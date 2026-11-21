//! The Journey powers menu's input half (W4-c24) — `CreativeUI`'s toggle
//! (`CreativeUI.cs:146-217`), `UICreativePowersMenu`'s category clicks
//! (`UICreativePowersMenu.cs:265-424`), the sacrifice slot's `ItemSlot.Handle`
//! context 30 (`UIItemSlot.cs:24-31`), the duplication grid's contexts 29/34
//! (`UICreativeItemGrid.cs:18-34`) and the power strips' toggles/buttons
//! (`CreativePowers.cs:160-191,417-445,501-533`).
//!
//! The handler runs inside `in_world_click_cluster` after the chest/crafting
//! handlers and before `handle_inventory_clicks` — the C# `UIElement` click
//! routing consumes the press before `DrawInventory`'s slot arms see it
//! (`Player.mouseInterface`).

use glam::Vec2;

use crate::app::App;
use crate::creative::{PowerId, PowerState};
use crate::item::{ItemSlot, EMPTY_SLOT};
use crate::ui::journey as j;

impl App {
    /// `CreativeUI.Draw`'s gate (`CreativeUI.cs:98-102`): the difficulty-3
    /// player, unblocked. The port's difficulty stand-in is the session
    /// draft (`active_player_difficulty`, `app/mod.rs`).
    pub(crate) fn journey_available(&self) -> bool {
        self.active_player_difficulty() == 3
            && !j::blocked(
                self.talk_npc.is_some(),
                self.open_container() >= 0,
                self.crafting_open,
            )
    }

    /// `CreativeUI.ToggleMenu` (`CreativeUI.cs:198-217`): the flip, sound 12,
    /// the open arm closes crafting/chest; the close arm returns the
    /// sacrifice slot's item (`GetOrDropItem`, `:211-216`).
    pub(crate) fn journey_toggle_menu(&mut self) {
        self.journey.enabled = !self.journey.enabled;
        self.audio.play("Menu_Tick"); // `SoundEngine.PlaySound(12)` (`:203`)
        if self.journey.enabled {
            // `:204-210`: `NewCraftingUI.Close(quiet, returnToInventory)`,
            // `chest = -1`, the TE anchor clear (no port surface).
            self.crafting_open = false;
            if let Some(p) = self.player.as_mut() {
                p.chest = crate::bank::CHEST_CLOSED;
            }
        } else {
            // The close arm stops the search input (`OnDeactivate`).
            self.journey.search_writing = false;
            if self.journey.sacrifice.count > 0 {
                self.return_sacrifice_slot();
                j::stop_animation(&mut self.journey);
            }
        }
    }

    /// `CreativeUI.CloseMenu` (`CreativeUI.cs:183-190`) — the inventory-close
    /// / chest-open rule (`Main.cs:45647,45664`).
    pub(crate) fn journey_close_menu(&mut self) {
        if self.journey.enabled {
            self.journey.enabled = false;
            j::stop_animation(&mut self.journey);
        }
        // The search input stops with the menu (`UISearchBar.OnDeactivate`,
        // `UISearchBar.cs:190-196`).
        self.journey.search_writing = false;
    }

    /// `Player.GetOrDropItem(sacrifice, ReturnItemFromSlot)`
    /// (`CreativeUI.cs:213`) — the `return_reforge_slot` shape.
    fn return_sacrifice_slot(&mut self) {
        let slot = self.journey.sacrifice;
        if slot.count == 0 {
            return;
        }
        if let Some(i) = crate::item::slot_for_pickup(&mut self.inventory, slot.kind, slot.prefix) {
            self.inventory[i].count = (self.inventory[i].count + slot.count)
                .min(crate::registry::max_stack(slot.kind));
        } else {
            let (cx, cy) = self.player.as_ref().map(|p| p.center()).unwrap_or((0.0, 0.0));
            let rng = self.rngs.global();
            let vx = (rng.next_float() * 2.0 - 1.0) * 1.2;
            let vy = -1.5 - rng.next_float();
            let phase = rng.next_float() * std::f32::consts::TAU;
            self.items.push(crate::item::ItemDrop::new(
                slot.kind, slot.count, slot.prefix, cx, cy, vx, vy, phase,
            ));
        }
        self.journey.sacrifice = EMPTY_SLOT;
    }

    /// `Player.ToggleCreativeMenu` (`Player.cs:5091-5105`) — the `C` key
    /// (`PlayerInput.cs:2087`): alive + Journey; with the inventory open and
    /// unblocked it toggles directly, else it opens the inventory first.
    pub(crate) fn toggle_creative_menu_key(&mut self) {
        let alive = self.player.as_ref().is_some_and(|p| !p.dead);
        if !alive || self.active_player_difficulty() != 3 {
            return;
        }
        if self.inventory_open && !j::blocked(
            self.talk_npc.is_some(),
            self.open_container() >= 0,
            self.crafting_open,
        ) {
            self.journey_toggle_menu();
            return;
        }
        // `IngameUIWindows.CloseAll(!playerInventory)` + `OpenInventory`
        // (`:5100-5103`): the port's windows close with the inventory open.
        self.inventory_open = true;
        self.journey_toggle_menu();
    }

    /// The per-frame update tail — `CreativeUI.Update` (`:84-90`) →
    /// `UICreativeInfiniteItemsDisplay_OnUpdate` (`:505-529`): the animation
    /// tick while the menu is live, the hover cues, the inventory-close /
    /// chest-open close rule (`Main.cs:45647,45664`).
    pub(crate) fn journey_tick(&mut self) {
        // `Draw`'s difficulty gate (`CreativeUI.cs:98-101`).
        if self.active_player_difficulty() != 3 {
            self.journey.enabled = false;
        }
        // `DrawInterface_27_Inventory` (`Main.cs:45643-45664`): a chest open
        // or the inventory closed force-closes the menu.
        if !self.inventory_open || self.open_container() >= 0 {
            self.journey_close_menu();
        }
        if !self.journey.enabled || !self.inventory_open {
            // The search input stops when the menu closes
            // (`UISearchBar.OnDeactivate`, `UISearchBar.cs:190-196`).
            self.journey.search_writing = false;
            return;
        }
        let research_page = self.journey.category == j::CATEGORY_RESEARCH;
        // The spark draws + the thunk's bank pick run on the bare-`Main.rand`
        // stand-in, in the C#'s order: sound 63 (its `Next(1, 4)` pick,
        // `LegacySoundPlayer.cs:482`) BEFORE the 15 spark draws
        // (`UICreativeInfiniteItemsDisplay.cs:349-364`).
        {
            let rng = self.rngs.global();
            let audio = &self.audio;
            let mut play = |cue: &str| audio.play(cue);
            j::tick_animation(&mut self.journey, research_page, rng, &mut play);
        }
        // `descriptionText_OnUpdate`'s forget (`:409-412`): a non-air cursor
        // clears the remembered numbers every update.
        if self.mouse_item.count > 0 {
            self.journey.remembered = (0, 0, 0);
        }
        // The toggle button's stateful tick (`DoStatefulTickSound`,
        // `CreativeUI.cs:171`): sound 12 on the hover-enter edge.
        let (sw, sh) = self.screen_size();
        let (ts, ps) = j::sheet_sizes(&self.lookups);
        let l = j::layout(Vec2::new(sw, sh), self.journey.enabled, ts, ps);
        let hovered = crate::ui::in_rect(self.mouse_pos, l.toggle);
        if hovered && !self.journey.toggle_hovered {
            self.audio.play("Menu_Tick");
        }
        self.journey.toggle_hovered = hovered;
        // `GroupOptionButton.DrawSelf`'s hover cue (`:212-223`): one latch —
        // the hovered button id changes → sound 12.
        let button = self.journey_button_at(&l, self.mouse_pos);
        if button != self.journey.hovered_button {
            if button >= 0 {
                self.audio.play("Menu_Tick");
            }
            self.journey.hovered_button = button;
        }
        // The duplication grid's hover (`UICreativeItemGrid.cs:22-29`): the
        // hovered cell clears its newly-researched status; the scrollbar
        // drag re-derives the view every held frame (`UIScrollbar.cs:109-112`).
        if self.journey.category == j::CATEGORY_DUPLICATION {
            let items = j::grid_items(
                &self.research,
                &self.journey.filters,
                &self.journey.search,
                self.player.as_ref(),
                self.active_player_difficulty(),
            );
            if let Some(i) = j::grid_hit(l.list, self.journey.scroll, items.len(), self.mouse_pos) {
                self.research.clear_newly_researched(items[i]);
            }
            if let Some(offset) = self.journey.scroll_drag {
                if self.mine_down {
                    let cols = j::grid_cols(l.list);
                    let view = l.list[3];
                    let max_view = j::grid_height(items.len(), cols).max(view);
                    self.journey.scroll = crate::ui::panel::scrollbar_drag_value(
                        l.scrollbar, view, max_view, offset, self.mouse_pos.y,
                    );
                } else {
                    self.journey.scroll_drag = None;
                }
            }
        }
        // The time slider's held drag (`UIVerticalSlider.cs:54-70`): while
        // the left button holds over the bar, the value follows the pointer.
        if self.journey.category == j::CATEGORY_TIME
            && self.journey.time_sub == 1
            && self.mine_down
        {
            let bar = j::time_slider_bar(j::time_slider_panel(&l));
            if crate::ui::in_rect(self.mouse_pos, j::slider_hit_rect(bar)) {
                let v = j::slider_value_at(bar, self.mouse_pos.y);
                if v != self.journey.time_rate_cache {
                    self.journey.time_rate_cache = v;
                    self.creative_powers.set(
                        PowerId::TimeRate,
                        PowerState::Slider(j::time_rate_from_cache(v)),
                    );
                }
            }
        }
        // The weather sliders' held drags — the same `UIVerticalSlider`
        // machine. The wind slider writes the target directly
        // (`ModifyWindDirectionAndStrength.UpdateInfoFromSliderValueCache`,
        // `CreativePowers.cs:1188-1191`); the rain slider runs the full
        // StartRain/StopRain ladder (`:1320-1330`).
        if self.journey.category == j::CATEGORY_WEATHER && self.mine_down {
            let panel = j::weather_slider_panel(&l);
            let bar = j::slider_bar(panel);
            if crate::ui::in_rect(self.mouse_pos, j::slider_hit_rect(bar)) {
                let v = j::slider_value_at(bar, self.mouse_pos.y);
                match self.journey.weather_sub {
                    1 => {
                        let target = crate::creative::wind_target_from_cache(v);
                        if target != self.wind.speed_target {
                            // `Main.windSpeedCurrent = Main.windSpeedTarget =
                            // Lerp(-0.8, 0.8, cache)` — both at once.
                            self.wind.speed_current = target;
                            self.wind.speed_target = target;
                            self.creative_powers
                                .set(PowerId::WindStrength, PowerState::Slider(target));
                        }
                    }
                    2 => {
                        if v != self.creative_powers.slider(PowerId::RainStrength) {
                            self.creative_powers
                                .set(PowerId::RainStrength, PowerState::Slider(v));
                            let luck = self.player.as_ref().map(|p| p.luck()).unwrap_or(0.0);
                            self.rain.set_rain_power(v, &self.world, self.wind.remix_world, luck);
                        }
                    }
                    _ => {}
                }
            }
        }
        // The spawn-rate slider's held drag — the per-player cache
        // (`APerPlayerSliderPower.PushChangeAndSetSlider`, `:300-309`,
        // clamped [0,1]).
        if self.journey.category == j::CATEGORY_PERSONAL
            && self.journey.personal_sub == 1
            && self.mine_down
        {
            let bar = j::slider_bar(j::spawn_slider_panel(&l));
            if crate::ui::in_rect(self.mouse_pos, j::slider_hit_rect(bar)) {
                let v = j::slider_value_at(bar, self.mouse_pos.y);
                if v != self.journey.spawn_rate_cache {
                    self.journey.spawn_rate_cache = v;
                    self.creative_powers
                        .set(PowerId::SpawnRate, PowerState::Slider(v));
                }
            }
        }
        // The difficulty slider's held drag — the cache + the derived
        // strength (`DifficultySliderPower.UpdateInfoFromSliderValueCache`,
        // `:1018-1028`).
        if self.journey.category == 5 && self.mine_down {
            let bar = j::slider_bar(j::difficulty_slider_panel(&l));
            if crate::ui::in_rect(self.mouse_pos, j::slider_hit_rect(bar)) {
                let v = j::slider_value_at(bar, self.mouse_pos.y);
                if v != self.journey.difficulty_cache {
                    self.journey.difficulty_cache = v;
                    self.creative_powers.set(
                        PowerId::Difficulty,
                        PowerState::Slider(crate::creative::difficulty_strength_from_cache(v)),
                    );
                }
            }
        }
        // The search box's outside-click stop (`UIWrappedSearchBar.Update`,
        // `UIWrappedSearchBar.cs:239-246`): while writing, a press outside
        // the bar stops the input.
        if self.journey.search_writing
            && (self.ui_left_press || self.ui_right_press)
            && !crate::ui::in_rect(self.mouse_pos, j::search_bar_rect(&l))
        {
            self.journey.search_writing = false;
        }
    }

    /// The hovered strip button's global id: main 0..6, time 10..15,
    /// weather 20..23, personal 30..32; -1 none. (One button can be hovered
    /// at a time — the strips never overlap.)
    fn journey_button_at(&self, l: &j::Layout, mouse: Vec2) -> i32 {
        if !self.journey.enabled || !self.inventory_open {
            return -1;
        }
        for i in 0..j::MAIN_BUTTONS {
            if crate::ui::in_rect(mouse, j::strip_button(l.main_strip, i)) {
                return i as i32;
            }
        }
        let (strip, count, base) = match self.journey.category {
            j::CATEGORY_TIME => (j::time_strip(l), j::TIME_BUTTONS, 10),
            j::CATEGORY_WEATHER => (j::weather_strip(l), j::WEATHER_BUTTONS, 20),
            j::CATEGORY_PERSONAL => (j::personal_strip(l), j::PERSONAL_BUTTONS, 30),
            _ => return -1,
        };
        for i in 0..count {
            if crate::ui::in_rect(mouse, j::strip_button(strip, i)) {
                return base + i as i32;
            }
        }
        -1
    }

    /// The menu's click dispatch. Returns true when the press was consumed
    /// (`Player.mouseInterface` — the inventory handler stands down).
    pub(crate) fn handle_journey_clicks(&mut self) -> bool {
        if !self.journey_available() {
            return false;
        }
        let left = self.ui_left_press;
        let right = self.ui_right_press || self.place_down;
        let (sw, sh) = self.screen_size();
        let screen = Vec2::new(sw, sh);
        let (ts, ps) = j::sheet_sizes(&self.lookups);
        let l = j::layout(screen, self.journey.enabled, ts, ps);
        // The toggle button (`DrawToggleButton`, `CreativeUI.cs:153-170`):
        // the press edge flips the menu, hovered or not enabled.
        if left && crate::ui::in_rect(self.mouse_pos, l.toggle) {
            self.journey_toggle_menu();
            self.ui_left_press = false;
            return true;
        }
        if !self.journey.enabled || !self.inventory_open {
            return false;
        }
        // The main strip's category buttons (`MainCategoryButtonClick`,
        // `UICreativePowersMenu.cs:265-269`).
        if left {
            for i in 0..j::MAIN_BUTTONS {
                if !crate::ui::in_rect(self.mouse_pos, j::strip_button(l.main_strip, i)) {
                    continue;
                }
                self.ui_left_press = false;
                self.audio.play("Menu_Tick"); // `GroupOptionButton.LeftMouseDown` (`:255`)
                match i {
                    5 => {
                        // StopBiomeSpread's toggle (`ASharedTogglePower.
                        // button_OnClick`, `CreativePowers.cs:527-533`).
                        let on = !self.creative_powers.stop_biome_spread();
                        self.creative_powers.set(PowerId::StopBiomeSpread, PowerState::Toggle(on));
                    }
                    6 => {
                        // The difficulty slider's subcategory button
                        // (`CreateMainPowerStrip`, `UICreativePowersMenu.cs:
                        // 205`): the category opens the slider panel.
                        self.journey.category = j::toggle_category(self.journey.category, 5);
                    }
                    _ => {
                        let option = j::MAIN_OPTIONS[i];
                        self.journey.category = j::toggle_category(self.journey.category, option);
                    }
                }
                return true;
            }
        }
        match self.journey.category {
            j::CATEGORY_RESEARCH => self.journey_research_clicks(&l, left, right),
            j::CATEGORY_DUPLICATION => self.journey_duplication_clicks(&l, left, right),
            j::CATEGORY_TIME => self.journey_time_clicks(&l, left),
            j::CATEGORY_WEATHER => self.journey_weather_clicks(&l, left),
            j::CATEGORY_PERSONAL => self.journey_personal_clicks(&l, left),
            5 => self.journey_difficulty_clicks(&l, left),
            _ => false,
        }
    }

    /// The research page: the sacrifice slot (context 30) and the confirm
    /// button (`sacrificeButton_OnClick`, `UICreativeInfiniteItemsDisplay.cs:436-439`).
    fn journey_research_clicks(&mut self, l: &j::Layout, left: bool, right: bool) -> bool {
        if left && crate::ui::in_rect(self.mouse_pos, l.confirm) {
            self.ui_left_press = false;
            // `SacrificeWhatYouCan` (`:441-458`): the confirm plays no cue
            // of its own — the thunk/chime ride the animation.
            let excess = j::sacrifice_confirm(&mut self.journey, &mut self.research);
            if excess {
                // `CreativeUI.cs:320-323`: the leftover stack returns to the
                // inventory (`Player.GetItem`, ReturnItemFromSlot).
                self.return_sacrifice_slot();
            }
            return true;
        }
        // The slot element (`UIItemSlot.HandleItemSlotLogic`, `:24-31`):
        // context 30 forces movement action 0 (`ItemSlot.cs:1629-1632`) —
        // the whole-stack swap/merge (`LeftClick` case 0, `:934-1017`),
        // sound 7 when either side holds an item.
        if (left || right) && crate::ui::in_rect(self.mouse_pos, l.slot) {
            if left {
                self.ui_left_press = false;
                let mut held = self.mouse_item;
                let mut slot = self.journey.sacrifice;
                std::mem::swap(&mut held, &mut slot);
                // The merge (`:980-998`): same kind + prefix, both under max.
                if crate::bank::can_stack(&held, &slot) {
                    let max = crate::registry::max_stack(slot.kind);
                    if slot.count < max && held.count < max {
                        let total = slot.count + held.count;
                        slot.count = total.min(max);
                        held.count = total.saturating_sub(max);
                    }
                }
                if held.count == 0 {
                    held = EMPTY_SLOT;
                }
                if slot.count == 0 {
                    slot = EMPTY_SLOT;
                }
                if held.count > 0 || slot.count > 0 {
                    self.audio.play("Grab"); // `SoundEngine.PlaySound(7)` (`:1010`)
                }
                self.mouse_item = held;
                self.journey.sacrifice = slot;
            } else {
                // `RightClick` context 30: the paced one-per-frame pickup
                // (`ItemSlot.cs:1700-1713`), no drain on the source? — the
                // sacrifice slot DOES drain (`context != 29`, `:1739-1742`).
                if self.ui_right_press {
                    self.ui_right_press = false;
                }
                self.journey_take_one_from_slot();
            }
            return true;
        }
        // The panel's hover blocks the world (`Hover_OnUpdate`, `:131-137`)
        // but only a slot/button hit consumes the press.
        false
    }

    /// `ItemSlot.PickupItemIntoMouse` for the sacrifice slot (context 30):
    /// one item per press onto the cursor, the slot drains
    /// (`ItemSlot.cs:1717-1746`), sound 12 + the stack-split refresh
    /// (`:1710-1711`, `:1774-1784`).
    fn journey_take_one_from_slot(&mut self) {
        if self.stack_split > 1 || self.journey.sacrifice.count == 0 {
            return; // `:1700-1703`
        }
        let slot = self.journey.sacrifice;
        let can = self.mouse_item.count == 0
            || (crate::bank::can_stack(&self.mouse_item, &slot)
                && self.mouse_item.count < crate::registry::max_stack(slot.kind));
        if !can {
            return;
        }
        if self.mouse_item.count == 0 {
            self.mouse_item = slot;
            self.mouse_item.count = 0;
        }
        self.mouse_item.count += 1;
        self.journey.sacrifice.count -= 1;
        if self.journey.sacrifice.count == 0 {
            self.journey.sacrifice = EMPTY_SLOT;
        }
        self.audio.play("Menu_Tick");
        // `RefreshStackSplitCooldown` (`:1774-1784`).
        self.stack_split = if self.stack_split == 0 { 30 } else { self.stack_delay };
    }

    /// The duplication page: the filter strip, the scrollbar and the grid
    /// cells (contexts 29/34).
    fn journey_duplication_clicks(&mut self, l: &j::Layout, left: bool, right: bool) -> bool {
        // The search bar (`UIWrappedSearchBar`, `UIWrappedSearchBar.cs:
        // 95-151`): the magnifier / box click toggles the input, the cancel
        // button clears (sound 11 with contents, 12 without, `:153-164`),
        // the box's right click clears + starts writing (`:230-237`).
        if left && crate::ui::in_rect(self.mouse_pos, j::search_cancel_rect(l)) {
            self.ui_left_press = false;
            if !self.journey.search.trim().is_empty() {
                self.journey.search.clear();
                self.audio.play("Menu_Close"); // `SoundEngine.PlaySound(11)` (`:158`)
            } else {
                self.audio.play("Menu_Tick"); // `:162`
            }
            return true;
        }
        if left
            && (crate::ui::in_rect(self.mouse_pos, j::search_button_rect(l))
                || crate::ui::in_rect(self.mouse_pos, j::search_box_rect(l)))
        {
            self.ui_left_press = false;
            // `Click_SearchArea` (`:222-228`): the toggle. The cancel button
            // is a child of the box but its own handler consumed the press
            // above (the C#'s `evt.Target.Parent != _searchBoxPanel` gate).
            self.journey.search_writing = !self.journey.search_writing;
            return true;
        }
        if right && crate::ui::in_rect(self.mouse_pos, j::search_box_rect(l)) {
            // `SearchBox_OnRightClick` (`:230-237`): clear + start writing.
            if self.ui_right_press {
                self.ui_right_press = false;
            }
            self.journey.search.clear();
            self.journey.search_writing = true;
            return true;
        }
        // The filter tabs (`singleFilterButtonClick`,
        // `UICreativeItemsInfiniteFilteringOptions.cs:80-100`).
        if left {
            if let Some(i) = j::filter_hit(l, self.mouse_pos) {
                self.journey.filters.click(i);
                self.audio.play("Menu_Tick");
                self.ui_left_press = false;
                return true;
            }
        }
        let items = j::grid_items(
            &self.research,
            &self.journey.filters,
            &self.journey.search,
            self.player.as_ref(),
            self.active_player_difficulty(),
        );
        let cols = j::grid_cols(l.list);
        let view = l.list[3];
        let max_view = j::grid_height(items.len(), cols).max(view);
        // The scrollbar (`UIScrollbar.LeftMouseDown`, `:127-145`): the handle
        // arms the drag, the track jumps.
        if left && crate::ui::in_rect(self.mouse_pos, l.scrollbar) {
            self.ui_left_press = false;
            let handle = crate::ui::panel::scrollbar_handle_at(
                l.scrollbar, view, max_view, self.journey.scroll,
            );
            if crate::ui::in_rect(self.mouse_pos, handle) {
                self.journey.scroll_drag = Some(self.mouse_pos.y - handle[1]);
            } else {
                self.journey.scroll = crate::ui::panel::scrollbar_jump_value(
                    l.scrollbar, view, max_view, handle[3], self.mouse_pos.y,
                );
            }
            return true;
        }
        // The grid cells (`UICreativeItemGrid.DrawSlot` → `ItemSlot.Handle`).
        if let Some(i) = j::grid_hit(l.list, self.journey.scroll, items.len(), self.mouse_pos) {
            let kind = items[i];
            let ctx = j::grid_context(&self.research, kind);
            if left && self.ui_left_press {
                self.ui_left_press = false;
                if ctx == 29 {
                    // Action 5 (`ItemSlot.cs:1622-1627`, `:1242-1251`): an
                    // air cursor takes a full stack (or the one-needed 1).
                    if self.mouse_item.count == 0 {
                        let max = if crate::bank::only_need_one_in_inventory(kind) {
                            1
                        } else {
                            crate::registry::max_stack(kind)
                        };
                        self.mouse_item = ItemSlot {
                            kind,
                            name: crate::registry::name(kind),
                            count: max,
                            prefix: 0,
                        };
                        self.audio.play("Grab"); // `SoundEngine.PlaySound(7)` (`:1245`)
                    }
                }
                // Context 34: action -1 — the partial cell refuses the left
                // click (`PickItemMovementAction`'s default).
                return true;
            }
            if right && ctx == 29 {
                // The paced one-per-frame pickup (`ItemSlot.cs:1700-1713`),
                // context 29 does NOT drain the cell (`:1739-1742`).
                if self.ui_right_press {
                    self.ui_right_press = false;
                }
                if self.stack_split <= 1 {
                    let can = self.mouse_item.count == 0
                        || (self.mouse_item.kind == kind
                            && self.mouse_item.prefix == 0
                            && self.mouse_item.count < crate::registry::max_stack(kind));
                    if can {
                        if self.mouse_item.count == 0 {
                            self.mouse_item = ItemSlot {
                                kind,
                                name: crate::registry::name(kind),
                                count: 0,
                                prefix: 0,
                            };
                        }
                        self.mouse_item.count += 1;
                        self.audio.play("Menu_Tick");
                        self.stack_split =
                            if self.stack_split == 0 { 30 } else { self.stack_delay };
                    }
                }
                return true;
            }
            if right {
                return true; // context 34 refuses right clicks (`:1654`)
            }
        }
        false
    }

    /// The time strip (`CreateTimePowerStrip`'s buttons +
    /// `TimeCategoryButtonClick`, `UICreativePowersMenu.cs:406-414`).
    fn journey_time_clicks(&mut self, l: &j::Layout, left: bool) -> bool {
        if !left {
            return false;
        }
        let strip = j::time_strip(l);
        for i in 0..j::TIME_BUTTONS {
            if !crate::ui::in_rect(self.mouse_pos, j::strip_button(strip, i)) {
                continue;
            }
            self.ui_left_press = false;
            self.audio.play("Menu_Tick");
            match i {
                0 => {
                    let on = !self.creative_powers.toggle(PowerId::FreezeTime);
                    self.creative_powers.set(PowerId::FreezeTime, PowerState::Toggle(on));
                }
                1..=4 => {
                    // The momentary buttons (`ASharedButtonPower.button_OnClick`,
                    // `CreativePowers.cs:439-445`): the clock jump.
                    let id = match i {
                        1 => PowerId::SetDay,
                        2 => PowerId::SetNoon,
                        3 => PowerId::SetNight,
                        _ => PowerId::SetMidnight,
                    };
                    self.set_time_of_day(id);
                }
                _ => {
                    self.journey.time_sub = if self.journey.time_sub == 1 { 0 } else { 1 };
                }
            }
            return true;
        }
        // The slider panel: the x24/x12/x1 labels
        // (`topText/middleText/bottomText_OnClick`, `CreativePowers.cs:926-942`)
        // and the bar's press-to-set.
        if self.journey.time_sub == 1 {
            let panel = j::time_slider_panel(l);
            let sizes: [[f32; 2]; 3] = std::array::from_fn(|k| {
                let label = ["x24", "x12", "x1"][k];
                let w = crate::ui::text::ui_font_width(&self.lookups, label, 1.0)
                    .unwrap_or(8.0 * label.len() as f32);
                let h = crate::ui::text::ui_font_height(&self.lookups, 1.0).unwrap_or(8.0);
                [w, h]
            });
            let rects = j::time_label_rects(panel, sizes);
            for (k, v) in [(0, 1.0f32), (1, 0.5), (2, 0.0)] {
                if crate::ui::in_rect(self.mouse_pos, rects[k]) {
                    self.ui_left_press = false;
                    self.audio.play("Menu_Tick");
                    self.journey.time_rate_cache = v;
                    self.creative_powers.set(
                        PowerId::TimeRate,
                        PowerState::Slider(j::time_rate_from_cache(v)),
                    );
                    return true;
                }
            }
            let bar = j::time_slider_bar(panel);
            if crate::ui::in_rect(self.mouse_pos, j::slider_hit_rect(bar)) {
                self.ui_left_press = false;
                self.audio.play("Menu_Tick");
                let v = j::slider_value_at(bar, self.mouse_pos.y);
                self.journey.time_rate_cache = v;
                self.creative_powers.set(
                    PowerId::TimeRate,
                    PowerState::Slider(j::time_rate_from_cache(v)),
                );
                return true;
            }
        }
        false
    }

    /// The weather strip (`CreateWeatherPowerStrip` +
    /// `WeatherCategoryButtonClick`, `UICreativePowersMenu.cs:384-404`):
    /// the wind/rain subcategory buttons, the two freeze toggles, the
    /// sliders' label clicks and press-to-set.
    fn journey_weather_clicks(&mut self, l: &j::Layout, left: bool) -> bool {
        if !left {
            return false;
        }
        let strip = j::weather_strip(l);
        for i in 0..j::WEATHER_BUTTONS {
            if !crate::ui::in_rect(self.mouse_pos, j::strip_button(strip, i)) {
                continue;
            }
            self.ui_left_press = false;
            self.audio.play("Menu_Tick");
            match i {
                0 => self.journey.weather_sub = if self.journey.weather_sub == 1 { 0 } else { 1 },
                1 => {
                    // FreezeWindDirectionAndStrength (`ASharedTogglePower.
                    // button_OnClick`, `CreativePowers.cs:527-533`).
                    let on = !self.creative_powers.freeze_wind();
                    self.creative_powers.set(PowerId::FreezeWind, PowerState::Toggle(on));
                }
                2 => self.journey.weather_sub = if self.journey.weather_sub == 2 { 0 } else { 2 },
                _ => {
                    let on = !self.creative_powers.freeze_rain();
                    self.creative_powers.set(PowerId::FreezeRain, PowerState::Toggle(on));
                }
            }
            return true;
        }
        // The open slider's labels + bar.
        if self.journey.weather_sub == 1 || self.journey.weather_sub == 2 {
            let panel = j::weather_slider_panel(l);
            let wind = self.journey.weather_sub == 1;
            // The label clicks: wind West/None/East → 1/0.5/0
            // (`topText/middleText/bottomText_OnClick`, `CreativePowers.cs:
            // 1247-1263`); rain Monsoon/Drizzle/ClearSky → 1/0.5/0
            // (`:1386-1402`).
            let labels: [String; 3] = if wind {
                [
                    crate::lang::text("CreativePowers.WindWest"),
                    crate::lang::text("CreativePowers.WindNone"),
                    crate::lang::text("CreativePowers.WindEast"),
                ]
            } else {
                [
                    crate::lang::text("CreativePowers.WeatherMonsoon"),
                    crate::lang::text("CreativePowers.WeatherDrizzle"),
                    crate::lang::text("CreativePowers.WeatherClearSky"),
                ]
            };
            let sizes: [[f32; 2]; 3] = std::array::from_fn(|k| {
                let w = crate::ui::text::ui_font_width(&self.lookups, &labels[k], 1.0)
                    .unwrap_or(8.0 * labels[k].len() as f32);
                let h = crate::ui::text::ui_font_height(&self.lookups, 1.0).unwrap_or(8.0);
                [w, h]
            });
            let rects = j::slider_label_rects(panel, sizes);
            for (k, v) in [(0, 1.0f32), (1, 0.5), (2, 0.0)] {
                if crate::ui::in_rect(self.mouse_pos, rects[k]) {
                    self.ui_left_press = false;
                    self.audio.play("Menu_Tick");
                    self.journey_set_weather_slider(wind, v);
                    return true;
                }
            }
            let bar = j::slider_bar(panel);
            if crate::ui::in_rect(self.mouse_pos, j::slider_hit_rect(bar)) {
                self.ui_left_press = false;
                self.audio.play("Menu_Tick");
                let v = j::slider_value_at(bar, self.mouse_pos.y);
                self.journey_set_weather_slider(wind, v);
                return true;
            }
        }
        false
    }

    /// The weather sliders' shared set arm — the wind target write
    /// (`CreativePowers.cs:1188-1191`) or the rain power's Start/Stop ladder
    /// (`:1320-1330`).
    fn journey_set_weather_slider(&mut self, wind: bool, v: f32) {
        if wind {
            let target = crate::creative::wind_target_from_cache(v);
            self.wind.speed_current = target;
            self.wind.speed_target = target;
            self.creative_powers
                .set(PowerId::WindStrength, PowerState::Slider(target));
        } else {
            self.creative_powers
                .set(PowerId::RainStrength, PowerState::Slider(v));
            let luck = self.player.as_ref().map(|p| p.luck()).unwrap_or(0.0);
            self.rain.set_rain_power(v, &self.world, self.wind.remix_world, luck);
        }
    }

    /// The personal strip (`CreatePersonalPowerStrip` +
    /// `PersonalCategoryButtonClick`, `UICreativePowersMenu.cs:416-424`):
    /// the toggles, the spawn-rate subcategory button, the slider's
    /// x10/x1/x0 labels and press-to-set.
    fn journey_personal_clicks(&mut self, l: &j::Layout, left: bool) -> bool {
        if !left {
            return false;
        }
        let strip = j::personal_strip(l);
        for i in 0..j::PERSONAL_BUTTONS {
            if !crate::ui::in_rect(self.mouse_pos, j::strip_button(strip, i)) {
                continue;
            }
            self.ui_left_press = false;
            self.audio.play("Menu_Tick");
            match i {
                0 => {
                    let on = !self.creative_powers.god_mode();
                    self.creative_powers.set(PowerId::Godmode, PowerState::Toggle(on));
                }
                1 => {
                    let on = !self.creative_powers.far_placement_range();
                    self.creative_powers.set(PowerId::FarPlacementRange, PowerState::Toggle(on));
                }
                _ => {
                    self.journey.personal_sub = if self.journey.personal_sub == 1 { 0 } else { 1 };
                }
            }
            return true;
        }
        // The spawn-rate slider's labels (`topText/middleText/bottomText_
        // OnClick`, `CreativePowers.cs:1672-1688`): x10/x1/x0 → 1/0.5/0.
        if self.journey.personal_sub == 1 {
            let panel = j::spawn_slider_panel(l);
            let sizes: [[f32; 2]; 3] = std::array::from_fn(|k| {
                let label = ["x10", "x1", "x0"][k];
                let w = crate::ui::text::ui_font_width(&self.lookups, label, 1.0)
                    .unwrap_or(8.0 * label.len() as f32);
                let h = crate::ui::text::ui_font_height(&self.lookups, 1.0).unwrap_or(8.0);
                [w, h]
            });
            let rects = j::slider_label_rects(panel, sizes);
            for (k, v) in [(0, 1.0f32), (1, 0.5), (2, 0.0)] {
                if crate::ui::in_rect(self.mouse_pos, rects[k]) {
                    self.ui_left_press = false;
                    self.audio.play("Menu_Tick");
                    self.journey.spawn_rate_cache = v;
                    self.creative_powers
                        .set(PowerId::SpawnRate, PowerState::Slider(v));
                    return true;
                }
            }
            let bar = j::slider_bar(panel);
            if crate::ui::in_rect(self.mouse_pos, j::slider_hit_rect(bar)) {
                self.ui_left_press = false;
                self.audio.play("Menu_Tick");
                let v = j::slider_value_at(bar, self.mouse_pos.y);
                self.journey.spawn_rate_cache = v;
                self.creative_powers
                    .set(PowerId::SpawnRate, PowerState::Slider(v));
                return true;
            }
        }
        false
    }

    /// The difficulty slider's clicks (`DifficultySliderPower.ProvideSlider`'s
    /// indication icons, `CreativePowers.cs:1049-1071`, + the bar's
    /// press-to-set): the four icons set the caches 1/0.66/0.33/0, the bar
    /// sets the drag value; both derive the strength.
    fn journey_difficulty_clicks(&mut self, l: &j::Layout, left: bool) -> bool {
        if !left {
            return false;
        }
        let panel = j::difficulty_slider_panel(l);
        let icon_size = self
            .lookups
            .wc_difficulty_icons
            .iter()
            .find_map(|s| s.map(|s| [s.width as f32, s.height as f32]))
            .unwrap_or([32.0, 32.0]);
        let rects = j::difficulty_icon_rects(panel, icon_size);
        for (i, &v) in j::DIFFICULTY_ICON_CACHES.iter().enumerate() {
            if crate::ui::in_rect(self.mouse_pos, rects[i]) {
                self.ui_left_press = false;
                self.audio.play("Menu_Tick"); // `SoundEngine.PlaySound(12)` (`:1052`)
                self.journey_set_difficulty(v);
                return true;
            }
        }
        let bar = j::slider_bar(panel);
        if crate::ui::in_rect(self.mouse_pos, j::slider_hit_rect(bar)) {
            self.ui_left_press = false;
            self.audio.play("Menu_Tick");
            let v = j::slider_value_at(bar, self.mouse_pos.y);
            self.journey_set_difficulty(v);
            return true;
        }
        false
    }

    /// The difficulty cache + derived strength write
    /// (`UpdateInfoFromSliderValueCache`, `CreativePowers.cs:1018-1028`).
    fn journey_set_difficulty(&mut self, v: f32) {
        self.journey.difficulty_cache = v;
        self.creative_powers.set(
            PowerId::Difficulty,
            PowerState::Slider(crate::creative::difficulty_strength_from_cache(v)),
        );
    }

    /// The duplication grid's wheel scroll (`UIList.ScrollWheel`,
    /// `UIList.cs:107-114`: `ViewPosition -= ScrollWheelValue`). Returns
    /// true when the notch was consumed.
    pub(crate) fn journey_wheel(&mut self, notches: i32) -> bool {
        if !self.journey.enabled
            || !self.inventory_open
            || self.journey.category != j::CATEGORY_DUPLICATION
        {
            return false;
        }
        let (sw, sh) = self.screen_size();
        let (ts, ps) = j::sheet_sizes(&self.lookups);
        let l = j::layout(Vec2::new(sw, sh), true, ts, ps);
        if !crate::ui::in_rect(self.mouse_pos, l.dup_panel) {
            return false;
        }
        let items = j::grid_items(
            &self.research,
            &self.journey.filters,
            &self.journey.search,
            self.player.as_ref(),
            self.active_player_difficulty(),
        );
        let cols = j::grid_cols(l.list);
        let view = l.list[3];
        let max_view = j::grid_height(items.len(), cols).max(view);
        // The C# accumulator is 120 per notch (`Mouse.GetState`'s wheel
        // delta); the port's achievements scroll uses the row pitch — the
        // grid's pitch is 44.
        self.journey.scroll =
            (self.journey.scroll - notches as f32 * 44.0).clamp(0.0, (max_view - view).max(0.0));
        true
    }

    /// The shift-click research route (`GetAlternateClickAction`,
    /// `ItemSlot.cs:659-665`): while the research menu shows, a hovered
    /// non-air non-favorited grid slot lights the transfer cursor (9).
    pub(crate) fn journey_research_shift_active(&self) -> bool {
        self.journey.enabled
            && self.inventory_open
            && self.journey.category == j::CATEGORY_RESEARCH
    }

    /// `OverrideLeftClick`'s cursor-9 research arm (`ItemSlot.cs:815-820`):
    /// `SwapItem` into the sacrifice slot, sound 7, then
    /// `SacrificeItemInSacrificeSlot` — the whole stack sacrifices at once.
    pub(crate) fn journey_shift_sacrifice(&mut self, slot: usize) {
        std::mem::swap(&mut self.inventory[slot], &mut self.journey.sacrifice);
        self.audio.play("Grab"); // `SoundEngine.PlaySound(7)` (`:818`)
        let excess = j::sacrifice_confirm(&mut self.journey, &mut self.research);
        if excess {
            self.return_sacrifice_slot();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::ui::char_creation::NewPlayerDraft;

    /// A Journey session with the inventory open and the menu enabled.
    fn journey_app() -> App {
        let mut app = flow_app(None, vec![], vec![]);
        app.session_draft = Some(NewPlayerDraft { difficulty: 3, ..Default::default() });
        app.player = Some(crate::player::Player::spawn(&app.world));
        app.inventory_open = true;
        app.journey.enabled = true;
        app
    }

    fn lay(app: &App) -> j::Layout {
        let (sw, sh) = app.screen_size();
        let (ts, ps) = j::sheet_sizes(&app.lookups);
        j::layout(Vec2::new(sw, sh), app.journey.enabled, ts, ps)
    }

    fn centre(rect: [f32; 4]) -> Vec2 {
        Vec2::new(rect[0] + rect[2] * 0.5, rect[1] + rect[3] * 0.5)
    }

    /// `Player.ToggleCreativeMenu` (`Player.cs:5091-5105`): the C key needs
    /// difficulty 3.
    #[test]
    fn the_c_key_toggles_only_for_journey() {
        let mut app = journey_app();
        app.toggle_creative_menu_key();
        assert!(!app.journey.enabled);
        // A Classic session refuses.
        app.session_draft = Some(NewPlayerDraft { difficulty: 0, ..Default::default() });
        app.journey.enabled = false;
        app.toggle_creative_menu_key();
        assert!(!app.journey.enabled);
    }

    /// `ToggleMenu`'s open arm closes crafting + the chest
    /// (`CreativeUI.cs:204-210`); the close arm returns the sacrifice item.
    #[test]
    fn the_toggle_closes_crafting_and_returns_the_slot() {
        let mut app = journey_app();
        app.journey.enabled = false;
        app.crafting_open = true;
        app.journey_toggle_menu();
        assert!(app.journey.enabled);
        assert!(!app.crafting_open, "NewCraftingUI.Close (:206)");
        // The close arm returns the slot's stack to the inventory.
        app.journey.sacrifice = ItemSlot { kind: 8, name: "Torch", count: 7, prefix: 0 };
        app.journey_toggle_menu();
        assert!(!app.journey.enabled);
        assert_eq!(app.journey.sacrifice.count, 0);
        assert!(app.inventory.iter().any(|s| s.kind == 8 && s.count == 7));
    }

    /// The category buttons open/close exclusively (`ToggleCategory`,
    /// `UICreativePowersMenu.cs:301-312`) and the press is consumed.
    #[test]
    fn the_main_strip_opens_the_research_page() {
        let mut app = journey_app();
        let l = lay(&app);
        app.mouse_pos = centre(j::strip_button(l.main_strip, 1)); // research
        app.ui_left_press = true;
        assert!(app.handle_journey_clicks());
        assert_eq!(app.journey.category, j::CATEGORY_RESEARCH);
        assert!(!app.ui_left_press);
        // Re-click closes.
        app.ui_left_press = true;
        assert!(app.handle_journey_clicks());
        assert_eq!(app.journey.category, j::CATEGORY_NONE);
    }

    /// The confirm button sacrifices what fits (`SacrificeWhatYouCan`,
    /// `UICreativeInfiniteItemsDisplay.cs:441-458`): the counts clamp, the
    /// animation starts, the excess returns to the inventory.
    #[test]
    fn the_confirm_sacrifices_and_returns_the_excess() {
        let mut app = journey_app();
        app.journey.category = j::CATEGORY_RESEARCH;
        app.research.register_sacrifice(8, 90);
        app.journey.sacrifice = ItemSlot { kind: 8, name: "Torch", count: 25, prefix: 0 };
        let l = lay(&app);
        app.mouse_pos = centre(l.confirm);
        app.ui_left_press = true;
        assert!(app.handle_journey_clicks());
        assert_eq!(app.research.sacrificed(8), 100);
        assert!(app.research.is_fully_researched(8));
        assert_eq!(app.journey.anim_left, 60);
        assert!(app.journey.research_complete);
        // The 15 excess torches returned (`CreativeUI.cs:320-323`).
        assert!(app.inventory.iter().any(|s| s.kind == 8 && s.count == 15));
        assert_eq!(app.journey.sacrifice.count, 0);
    }

    /// The duplication grid's left click fills the cursor with a max stack
    /// (action 5, `ItemSlot.cs:1242-1251`); the partial cell refuses.
    #[test]
    fn the_grid_duplicates_a_full_stack() {
        let mut app = journey_app();
        app.journey.category = j::CATEGORY_DUPLICATION;
        app.research.register_sacrifice(8, 100); // Torch, fully
        app.research.register_sacrifice(2, 5); // Dirt, partial
        let l = lay(&app);
        let items = j::grid_items(&app.research, &app.journey.filters, "", None, 3);
        // Torch sorts first (fully researched beats partial).
        assert_eq!(items[0], 8);
        app.mouse_pos = centre(j::grid_cell(l.list, 0.0, 0));
        app.ui_left_press = true;
        assert!(app.handle_journey_clicks());
        assert_eq!(app.mouse_item.kind, 8);
        assert_eq!(app.mouse_item.count, crate::registry::max_stack(8));
        // The partial dirt cell refuses the left click.
        app.mouse_item = EMPTY_SLOT;
        app.mouse_pos = centre(j::grid_cell(l.list, 0.0, 1));
        app.ui_left_press = true;
        assert!(app.handle_journey_clicks());
        assert_eq!(app.mouse_item.count, 0);
    }

    /// The time strip: FreezeTime flips, the clock buttons jump, the rate
    /// labels set the derived 1..24 value (`CreativePowers.cs:877-880`).
    #[test]
    fn the_time_strip_drives_the_powers() {
        let mut app = journey_app();
        app.journey.category = j::CATEGORY_TIME;
        let l = lay(&app);
        let strip = j::time_strip(&l);
        // FreezeTime.
        app.mouse_pos = centre(j::strip_button(strip, 0));
        app.ui_left_press = true;
        assert!(app.handle_journey_clicks());
        assert!(app.creative_powers.toggle(PowerId::FreezeTime));
        // Noon (`StartNoonImmediately`, dayTime true, 27000).
        app.mouse_pos = centre(j::strip_button(strip, 2));
        app.ui_left_press = true;
        assert!(app.handle_journey_clicks());
        assert!(app.world.day_time);
        assert_eq!(app.world.time, 27000.0);
        // The rate subcategory opens.
        app.mouse_pos = centre(j::strip_button(strip, 5));
        app.ui_left_press = true;
        assert!(app.handle_journey_clicks());
        assert_eq!(app.journey.time_sub, 1);
        // x12 sets 0.5 → 12 (banker's rounding of 12.5).
        let panel = j::time_slider_panel(&l);
        let w = crate::ui::text::ui_font_width(&app.lookups, "x12", 1.0)
            .unwrap_or(8.0 * 3.0);
        let h = crate::ui::text::ui_font_height(&app.lookups, 1.0).unwrap_or(8.0);
        let rects = j::time_label_rects(panel, [[w, h]; 3]);
        app.mouse_pos = centre(rects[1]);
        app.ui_left_press = true;
        assert!(app.handle_journey_clicks());
        assert_eq!(app.journey.time_rate_cache, 0.5);
        assert_eq!(app.creative_powers.slider(PowerId::TimeRate), 12.0);
    }

    /// The personal strip's Godmode + Far Placement toggles
    /// (`APerPlayerTogglePower.button_OnClick`, `CreativePowers.cs:185-191`).
    #[test]
    fn the_personal_strip_toggles_the_powers() {
        let mut app = journey_app();
        app.journey.category = j::CATEGORY_PERSONAL;
        let l = lay(&app);
        let strip = j::personal_strip(&l);
        app.mouse_pos = centre(j::strip_button(strip, 0));
        app.ui_left_press = true;
        assert!(app.handle_journey_clicks());
        assert!(app.creative_powers.god_mode());
        app.mouse_pos = centre(j::strip_button(strip, 1));
        app.ui_left_press = true;
        assert!(app.handle_journey_clicks());
        assert!(app.creative_powers.far_placement_range());
    }

    /// The inventory-close rule force-closes the menu
    /// (`Main.cs:45664` → `CreativeUI.CloseMenu`).
    #[test]
    fn the_closed_inventory_closes_the_menu() {
        let mut app = journey_app();
        app.journey.anim_left = 30;
        app.inventory_open = false;
        app.journey_tick();
        assert!(!app.journey.enabled);
        assert_eq!(app.journey.anim_left, 0, "StopPlayingSacrificeAnimations");
    }

    /// The shift-click research route (`ItemSlot.cs:815-820`): the swap +
    /// the immediate sacrifice, the excess back to the inventory.
    #[test]
    fn the_shift_click_sacrifices_the_whole_slot() {
        let mut app = journey_app();
        app.journey.category = j::CATEGORY_RESEARCH;
        app.inventory[0] = ItemSlot { kind: 8, name: "Torch", count: 120, prefix: 0 };
        app.journey_shift_sacrifice(0);
        assert_eq!(app.research.sacrificed(8), 100);
        assert_eq!(app.journey.sacrifice.count, 0);
        // The 20 excess returned; the source slot swapped to air then refilled.
        assert_eq!(app.inventory[0].count, 20);
    }

    /// The wheel scrolls the duplication list, clamped
    /// (`UIList.ScrollWheel`, `UIList.cs:107-114`).
    #[test]
    fn the_wheel_scrolls_the_duplication_list() {
        let mut app = journey_app();
        app.journey.category = j::CATEGORY_DUPLICATION;
        for k in 0..200u16 {
            // 200 kinds with progress — enough rows to scroll.
            app.research.register_sacrifice(k + 1, 1);
        }
        let l = lay(&app);
        app.mouse_pos = centre(l.list);
        assert!(app.journey_wheel(-3));
        assert_eq!(app.journey.scroll, 132.0);
        assert!(app.journey_wheel(10));
        assert_eq!(app.journey.scroll, 0.0, "clamped at the top");
    }

    // ---- W4-c24 phase 4: the weather / spawn-rate / difficulty sliders ------

    /// The weather strip's freeze toggles latch the powers
    /// (`ASharedTogglePower.button_OnClick`, `CreativePowers.cs:527-533`).
    #[test]
    fn the_weather_freezes_toggle_the_powers() {
        let mut app = journey_app();
        app.journey.category = j::CATEGORY_WEATHER;
        let l = lay(&app);
        let strip = j::weather_strip(&l);
        // The wind freeze (button 1).
        app.mouse_pos = centre(j::strip_button(strip, 1));
        app.ui_left_press = true;
        assert!(app.handle_journey_clicks());
        assert!(app.creative_powers.freeze_wind());
        // The rain freeze (button 3).
        app.mouse_pos = centre(j::strip_button(strip, 3));
        app.ui_left_press = true;
        assert!(app.handle_journey_clicks());
        assert!(app.creative_powers.freeze_rain());
        // Re-click clears.
        app.mouse_pos = centre(j::strip_button(strip, 1));
        app.ui_left_press = true;
        assert!(app.handle_journey_clicks());
        assert!(!app.creative_powers.freeze_wind());
    }

    /// The wind slider's label clicks set the target
    /// (`ModifyWindDirectionAndStrength`, `CreativePowers.cs:1247-1263`):
    /// West → 1 → +0.8, No Wind → 0.5 → 0, East → 0 → -0.8.
    #[test]
    fn the_wind_slider_labels_set_the_target() {
        let mut app = journey_app();
        app.journey.category = j::CATEGORY_WEATHER;
        app.journey.weather_sub = 1;
        let l = lay(&app);
        let panel = j::weather_slider_panel(&l);
        let labels = [
            crate::lang::text("CreativePowers.WindWest"),
            crate::lang::text("CreativePowers.WindNone"),
            crate::lang::text("CreativePowers.WindEast"),
        ];
        let sizes: [[f32; 2]; 3] = std::array::from_fn(|k| {
            let w = crate::ui::text::ui_font_width(&app.lookups, &labels[k], 1.0)
                .unwrap_or(8.0 * labels[k].len() as f32);
            let h = crate::ui::text::ui_font_height(&app.lookups, 1.0).unwrap_or(8.0);
            [w, h]
        });
        let rects = j::slider_label_rects(panel, sizes);
        // West → +0.8.
        app.mouse_pos = centre(rects[0]);
        app.ui_left_press = true;
        assert!(app.handle_journey_clicks());
        assert!((app.wind.speed_target - 0.8).abs() < 1e-6);
        assert!((app.wind.speed_current - 0.8).abs() < 1e-6, "both at once (:1190)");
        // East → -0.8.
        app.mouse_pos = centre(rects[2]);
        app.ui_left_press = true;
        assert!(app.handle_journey_clicks());
        assert!((app.wind.speed_target + 0.8).abs() < 1e-6);
        // No Wind → 0.
        app.mouse_pos = centre(rects[1]);
        app.ui_left_press = true;
        assert!(app.handle_journey_clicks());
        assert!(app.wind.speed_target.abs() < 1e-6);
    }

    /// The rain slider's Monsoon click starts the rain at the override
    /// strength (`ModifyRainPower.UpdateInfoFromSliderValueCache`,
    /// `CreativePowers.cs:1320-1330` → `Main.StartRain(instant, override)`).
    #[test]
    fn the_rain_slider_monsoon_starts_the_rain() {
        let mut app = journey_app();
        app.world.world_surface = 200.0; // isThereAWorldSurface
        app.journey.category = j::CATEGORY_WEATHER;
        app.journey.weather_sub = 2;
        let l = lay(&app);
        let panel = j::weather_slider_panel(&l);
        let labels = [
            crate::lang::text("CreativePowers.WeatherMonsoon"),
            crate::lang::text("CreativePowers.WeatherDrizzle"),
            crate::lang::text("CreativePowers.WeatherClearSky"),
        ];
        let sizes: [[f32; 2]; 3] = std::array::from_fn(|k| {
            let w = crate::ui::text::ui_font_width(&app.lookups, &labels[k], 1.0)
                .unwrap_or(8.0 * labels[k].len() as f32);
            let h = crate::ui::text::ui_font_height(&app.lookups, 1.0).unwrap_or(8.0);
            [w, h]
        });
        let rects = j::slider_label_rects(panel, sizes);
        // Monsoon → cache 1 → StartRain(instant, 1).
        app.mouse_pos = centre(rects[0]);
        app.ui_left_press = true;
        assert!(app.handle_journey_clicks());
        assert!(app.rain.is_raining());
        assert!((app.rain.max_raining() - 1.0).abs() < 1e-6, "the override strength");
        assert!((app.rain.cloud_alpha() - 1.0).abs() < 1e-6, "instant snaps cloudAlpha");
        // Clear Sky → cache 0 → StopRain(instant).
        app.mouse_pos = centre(rects[2]);
        app.ui_left_press = true;
        assert!(app.handle_journey_clicks());
        assert!(!app.rain.is_raining());
        assert_eq!(app.rain.max_raining(), 0.0);
        assert_eq!(app.rain.cloud_alpha(), 0.0, "instant snaps cloudAlpha to 0");
    }

    /// The spawn-rate slider's x0 label disables the spawns
    /// (`SpawnRateSliderPerPlayerPower`, `CreativePowers.cs:1684-1688` →
    /// `GetShouldDisableSpawnsFor`).
    #[test]
    fn the_spawn_slider_x0_disables_the_spawns() {
        let mut app = journey_app();
        app.journey.category = j::CATEGORY_PERSONAL;
        app.journey.personal_sub = 1;
        let l = lay(&app);
        let panel = j::spawn_slider_panel(&l);
        let sizes: [[f32; 2]; 3] = std::array::from_fn(|k| {
            let label = ["x10", "x1", "x0"][k];
            let w = crate::ui::text::ui_font_width(&app.lookups, label, 1.0)
                .unwrap_or(8.0 * label.len() as f32);
            let h = crate::ui::text::ui_font_height(&app.lookups, 1.0).unwrap_or(8.0);
            [w, h]
        });
        let rects = j::slider_label_rects(panel, sizes);
        // x10 → cache 1 → x10.
        app.mouse_pos = centre(rects[0]);
        app.ui_left_press = true;
        assert!(app.handle_journey_clicks());
        assert!((app.creative_powers.spawn_rate_multiplier() - 10.0).abs() < 1e-5);
        assert!(!app.creative_powers.spawn_rate_disabled());
        // x0 → cache 0 → disabled.
        app.mouse_pos = centre(rects[2]);
        app.ui_left_press = true;
        assert!(app.handle_journey_clicks());
        assert!(app.creative_powers.spawn_rate_disabled());
        assert_eq!(app.journey.spawn_rate_cache, 0.0);
        // x1 → cache 0.5 → x1, enabled.
        app.mouse_pos = centre(rects[1]);
        app.ui_left_press = true;
        assert!(app.handle_journey_clicks());
        assert!(!app.creative_powers.spawn_rate_disabled());
        assert!((app.creative_powers.spawn_rate_multiplier() - 1.0).abs() < 1e-6);
    }

    /// The difficulty slider's indication icons set the caches
    /// (`DifficultySliderPower.Click_Master/Expert/Normal/Journey`,
    /// `CreativePowers.cs:1049-1071`): 1/0.66/0.33/0 → strengths 3/2/1/0.5.
    #[test]
    fn the_difficulty_icons_set_the_strength() {
        let mut app = journey_app();
        app.journey.category = 5;
        let l = lay(&app);
        let panel = j::difficulty_slider_panel(&l);
        let rects = j::difficulty_icon_rects(panel, [32.0, 32.0]);
        // Master (top) → cache 1 → strength 3.
        app.mouse_pos = centre(rects[0]);
        app.ui_left_press = true;
        assert!(app.handle_journey_clicks());
        assert_eq!(app.journey.difficulty_cache, 1.0);
        assert!((app.creative_powers.difficulty_strength() - 3.0).abs() < 1e-6);
        // Expert → cache 0.66 → strength 2.
        app.mouse_pos = centre(rects[1]);
        app.ui_left_press = true;
        assert!(app.handle_journey_clicks());
        assert!((app.creative_powers.difficulty_strength() - 2.0).abs() < 1e-6);
        // Normal → cache 0.33 → strength 1.
        app.mouse_pos = centre(rects[2]);
        app.ui_left_press = true;
        assert!(app.handle_journey_clicks());
        assert!((app.creative_powers.difficulty_strength() - 1.0).abs() < 1e-6);
        // Journey (bottom) → cache 0 → strength 0.5.
        app.mouse_pos = centre(rects[3]);
        app.ui_left_press = true;
        assert!(app.handle_journey_clicks());
        assert!((app.creative_powers.difficulty_strength() - 0.5).abs() < 1e-6);
        // The override seam answers the strength in a Journey session.
        assert_eq!(
            app.creative_powers.game_mode_difficulty_override(true),
            Some(0.5)
        );
    }

    /// The search box's click toggles the input, the cancel clears
    /// (`UIWrappedSearchBar`, `UIWrappedSearchBar.cs:153-164,222-228`).
    #[test]
    fn the_search_box_toggles_and_clears() {
        let mut app = journey_app();
        app.journey.category = j::CATEGORY_DUPLICATION;
        app.journey.search = "torch".into();
        let l = lay(&app);
        // The box click toggles the writing flag.
        app.mouse_pos = centre(j::search_box_rect(&l));
        app.ui_left_press = true;
        assert!(app.handle_journey_clicks());
        assert!(app.journey.search_writing);
        // The cancel clears the contents (sound 11).
        app.mouse_pos = centre(j::search_cancel_rect(&l));
        app.ui_left_press = true;
        assert!(app.handle_journey_clicks());
        assert!(app.journey.search.is_empty());
        // An empty cancel plays sound 12 (the Menu_Tick), keeps the flag.
        app.mouse_pos = centre(j::search_cancel_rect(&l));
        app.ui_left_press = true;
        assert!(app.handle_journey_clicks());
        assert!(app.journey.search.is_empty());
    }
}
