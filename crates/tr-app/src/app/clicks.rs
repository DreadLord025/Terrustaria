//! The App's HUD click handling — the slot/hotbar hit tests, the minimap
//! frame's buttons, the inventory stack moves and the legacy-menu action
//! dispatch. (A6: extracted from `main.rs`; the front screens' click tables
//! live in [`crate::app::flow`].)

use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;

use crate::achievements;
use crate::app::flow::{FlowClick, FrontScreen, next_flow_state};
use crate::app::netglue::compressed_world;
use crate::app::{App, NetRole, no_hud};
use crate::chest;
use crate::craft;
use crate::item::{EMPTY_SLOT, HOTBAR, ItemSlot, MAX_STACK};
use crate::menu;
use crate::net;
use crate::registry;
use crate::ui;
use glam::Vec2;

/// `AlternateClickAction.TransferFromChest`'s cursor id (`ItemSlot.cs:216`) —
/// the shift-click arrow on a hovered chest slot.
const CURSOR_FROM_CHEST: i32 = 8;
/// `AlternateClickAction.TransferToChest`'s cursor id (`ItemSlot.cs:217`) —
/// the shift-click arrow on a hovered inventory slot with an open chest.
const CURSOR_TO_CHEST: i32 = 9;

/// `ItemSlot.ShiftInUse` (`ItemSlot.cs:328-338`): the physical Shift keys are
/// `Main.keyState.PressingShift()`'s port (`app.keys` tracks the pressed
/// codes, `app/loop.rs:117`/:402). The `ShiftForcedOn` gamepad/test static is
/// out.
fn shift_in_use(app: &App) -> bool {
    app.keys.contains(&KeyCode::ShiftLeft) || app.keys.contains(&KeyCode::ShiftRight)
}

/// `ItemSlot.LeftClick`'s action-0 movement (`ItemSlot.cs:934-1017`) — the
/// one copy behind the inventory grid (context 0) and the chest grid
/// (context 3, `PickItemMovementAction` `:1505-1507`'s action 0): take into
/// an empty cursor, place onto an empty slot, swap different kinds, stack
/// into a same-kind partial up to the kind's `maxStack`
/// (`registry::max_stack`, the C#'s `Item.maxStack`). The C#'s
/// swap-then-`CanStack`-merge order (`MouseItemSwap` `:1255-1263` + the merge
/// `:980-998`) produces these same four outcomes at every boundary, equal
/// stacks included. Returns the `SoundEngine.PlaySound(7)` condition
/// (`:1004-1011` — an item sits on either side after the move).
///
/// `MouseItemSwap`'s favorite half (`:1255-1263`: both sides clear
/// `favorited` unless `retainFavoriteOnMoveAt[context]` — a chest move is a
/// clear, `chest::retain_favorite_on_move_at`) has nothing to clear yet: the
/// port's `ItemSlot` carries no favorite field (item.rs:52-57) — the no-op is
/// recorded for M7-UI-2, which owns the flag.
fn left_click_movement(held: &mut ItemSlot, target: &mut ItemSlot) -> bool {
    let max = registry::max_stack(target.kind);
    if held.count == 0 {
        *held = *target;
        *target = EMPTY_SLOT;
    } else if target.count == 0 {
        *target = *held;
        *held = EMPTY_SLOT;
    } else if target.kind == held.kind && target.count < max {
        let moved = (max - target.count).min(held.count);
        target.count += moved;
        held.count -= moved;
        if held.count == 0 {
            *held = EMPTY_SLOT;
        }
    } else {
        std::mem::swap(held, target);
    }
    held.count > 0 || target.count > 0
}

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
    /// crafts it — `try_craft` consumes the ingredients (the inventory, then
    /// the in-range chests) and the result goes to the inventory via
    /// `slot_for_pickup` (a full inventory drops on the ground in the
    /// original; the drop bridge is not wired yet, so a full inventory stacks
    /// only what fits — see docs/ANALYSIS.md). Consumes the press only when a
    /// recipe was crafted.
    pub(crate) fn handle_crafting_clicks(&mut self) -> bool {
        if !self.crafting_open || !self.ui_left_press {
            return false;
        }
        let (sw, sh) = self.screen_size();
        let screen = Vec2::new(sw, sh);
        // The toggles before the grid: the nearby-chests toggle flips the
        // setting (`CraftingUI.cs:336-355`), and a consumed press does not
        // reach the recipe cells.
        if ui::crafting::nearby_toggle_hit(screen, self.mouse_pos) {
            self.settings.craft_from_nearby_chests = !self.settings.craft_from_nearby_chests;
            self.ui_left_press = false;
            return true;
        }
        let Some(player) = self.player.as_ref() else { return false; };
        let adj = craft::scan_adjacent(&self.world, player);
        let zones = craft::Zones {
            snow: self.scene_metrics.zone_snow,
            graveyard: self.scene_metrics.zone_graveyard,
        };
        let in_range = craft::chests_in_range(
            &self.world,
            player,
            &self.world_chests,
            self.settings.craft_from_nearby_chests,
        );
        let available = craft::available(
            &self.inventory,
            &self.world_chests,
            &in_range,
            &adj,
            &zones,
        );
        let Some(idx) = ui::crafting::hit_test(screen, &self.lookups, self.mouse_pos, &available)
        else {
            return false;
        };
        let recipe = &craft::RECIPES[idx];
        let Some((kind, count)) =
            craft::try_craft(
                &mut self.inventory,
                &mut self.world_chests,
                &in_range,
                recipe,
                &adj,
                &zones,
                &mut self.craft_rng,
            )
        else {
            // The recipe row shows with a partial ingredient set; a click
            // without the full set does nothing (`Main.cs:42250` hover only).
            return false;
        };
        if let Some(i) = crate::item::slot_for_pickup(&mut self.inventory, kind, 0) {
            self.inventory[i].count = (self.inventory[i].count + count).min(crate::registry::max_stack(kind));
        }
        // `SoundEngine.PlaySound(7)` — the grant sound
        // (`CraftingEffects.SpawnEffects_BeforeGrantingItem`, `CraftingEffects.cs:51`).
        self.audio.play("Grab");
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
        // The reforge menu regions (`Main.cs:41639-41661`): the (50, 270)
        // input slot and the (120, 310) button (30x30), both + m_h. A hit
        // consumes the press before the slot grid.
        if self.inventory_open && self.in_reforge_menu {
            let (sw, sh) = self.screen_size();
            let m_h = (sh as f32 - 600.0) / 2.0;
            let (mx, my) = (self.mouse_pos.x, self.mouse_pos.y);
            let in_slot = mx >= 50.0 && mx <= 106.0 && my >= 270.0 + m_h && my <= 326.0 + m_h;
            let in_button =
                mx >= 120.0 && mx <= 150.0 && my >= 310.0 + m_h && my <= 340.0 + m_h;
            if left && in_slot {
                self.reforge_slot_click();
                return;
            }
            if left && in_button {
                self.reforge_click();
                return;
            }
        }
        // The transfer buttons (M3-STORE-2, `ChestUI.cs:282-307`'s click
        // dispatch): LootAll/DepositAll/QuickStack/Restock over the open
        // bank. A hit consumes the press before the slot grid.
        if self.inventory_open {
            let chest_open = self
                .player
                .as_ref()
                .map(|p| p.chest)
                .unwrap_or(crate::bank::CHEST_CLOSED);
            if let Some(bi) = crate::bank::bank_for_chest(chest_open) {
                let m_h = (self.screen_size().1 as f32 - 600.0) / 2.0;
                if let Some(id) = ui::inventory::bank_button_at(self.mouse_pos, m_h) {
                    if left {
                        let Some(p) = self.player.as_mut() else {
                            return;
                        };
                        let favs = self.favorite_slots.clone();
                        let fav = move |i: usize| favs.contains(&i);
                        match id {
                            0 => crate::bank::loot_all(
                                &mut p.banks[bi],
                                &mut self.inventory,
                                &mut self.equip.coins,
                            ),
                            1 => crate::bank::deposit_all(
                                &mut p.banks[bi],
                                &mut self.inventory,
                                &fav,
                            ),
                            2 => crate::bank::quick_stack(
                                &mut p.banks[bi],
                                &mut self.inventory,
                                &mut self.equip.coins,
                                &fav,
                            ),
                            _ => {
                                let _ = crate::bank::restock(
                                    &mut p.banks[bi],
                                    &mut self.inventory,
                                );
                            }
                        }
                        return;
                    }
                }
            }
        }
        // The bank panel's slots (M3-STORE-1, `ChestUI.Draw`'s context-4
        // grid): the same held/target merge as the grid rows below —
        // `ItemSlot.Handle`'s swap/stack/split semantics.
        if self.inventory_open {
            let chest_open = self
                .player
                .as_ref()
                .map(|p| p.chest)
                .unwrap_or(crate::bank::CHEST_CLOSED);
            if let Some(bank_idx) = (crate::bank::bank_for_chest(chest_open))
                .and_then(|_| {
                    let m_h = (self.screen_size().1 as f32 - 600.0) / 2.0;
                    ui::inventory::bank_slot_at(self.mouse_pos, m_h)
                })
            {
                let Some(p) = self.player.as_mut() else {
                    return;
                };
                let mut held = std::mem::replace(&mut self.mouse_item, crate::item::EMPTY_SLOT);
                let bi = crate::bank::bank_for_chest(chest_open).unwrap();
                let mut target = p.banks[bi][bank_idx];
                if left {
                    if held.count == 0 {
                        held = target;
                        target = crate::item::EMPTY_SLOT;
                    } else if target.count == 0 {
                        target = held;
                        held = crate::item::EMPTY_SLOT;
                    } else if target.kind == held.kind
                        && target.prefix == held.prefix
                        && target.count < crate::registry::max_stack(target.kind)
                    {
                        let moved = (crate::registry::max_stack(target.kind) - target.count)
                            .min(held.count);
                        target.count += moved;
                        held.count -= moved;
                        if held.count == 0 {
                            held = crate::item::EMPTY_SLOT;
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
                            prefix: target.prefix,
                        };
                        target.count -= take;
                        if target.count == 0 {
                            target = crate::item::EMPTY_SLOT;
                        }
                    }
                } else if target.count == 0 {
                    target = ItemSlot {
                        kind: held.kind,
                        name: held.name,
                        count: 1,
                        prefix: held.prefix,
                    };
                    held.count -= 1;
                    if held.count == 0 {
                        held = crate::item::EMPTY_SLOT;
                    }
                }
                self.mouse_item = held;
                p.banks[bi][bank_idx] = target;
                return;
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
            // The action-0 movement shared with the chest grid
            // (`left_click_movement`); the C# plays 7 when an item sits on
            // either side after the move (`ItemSlot.cs:1004-1011`).
            if left_click_movement(&mut held, &mut target) {
                self.audio.play("Grab");
            }
        } else if held.count == 0 {
            if target.count > 0 {
                let take = (target.count + 1) / 2;
                held = ItemSlot {
                    kind: target.kind,
                    name: target.name,
                    count: take,
                    prefix: target.prefix,
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
                prefix: held.prefix,
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

    /// The chest panel's slot dispatch (M3-CHEST-UI phase 4) — the ctx-3 half
    /// of `ChestUI.DrawSlots`' per-cell `ItemSlot.Handle` calls (`ChestUI.cs:
    /// 382-385`, every drawn frame for the hovered cell), the shift transfer
    /// into the chest from the inventory grid (`GetAlternateClickAction`'s
    /// ctx-0 arm, `ItemSlot.cs:680-686`), and the open-inventory complement of
    /// the editing.rs chest interception: with the panel up, the C# open
    /// branch (`Player.cs:33056-33260`) still runs on a world right-click that
    /// no UI slot holds, so the re-click toggle (`:33230-33252`) and the
    /// switch-to-another-chest click route here (the batch-review carry).
    /// `true` consumes the press and stops the click chain.
    pub(crate) fn handle_chest_clicks(&mut self) -> bool {
        // `Main.cursorOverride = -1` at the interface's head (`Main.cs:44941`).
        self.cursor_override = -1;
        if !self.inventory_open || self.bestiary_open {
            return false;
        }
        // `OverrideHover` → `GetAlternateClickAction` (`ItemSlot.cs:599-627`,
        // `:629-728`): the shift arms evaluate for the hovered slot with or
        // without a click.
        self.evaluate_shift_cursor_override();
        let left = self.ui_left_press;
        let right_held = self.place_down; // `Main.mouseRight` (`ItemSlot.cs:1654`)
        let right_edge = self.ui_right_press;
        if !left && !right_held {
            return false;
        }
        // The cursor-9 shift-click — `OverrideLeftClick` (`ItemSlot.cs:813-850`)
        // → `ChestUI.TryPlacingInChest(justCheck: false)` (`:846`).
        if left && self.cursor_override == CURSOR_TO_CHEST {
            let Some(slot) = self.slot_at_mouse() else {
                return false;
            };
            chest::try_placing_in_chest(self, slot, false);
            self.ui_left_press = false;
            return true;
        }
        // The chest cells own the pointer while the panel draws
        // (`player.mouseInterface = true`, `ChestUI.cs:86`'s visibility gate
        // + `:382-384`); a cell hit consumes both buttons even when the
        // stack-split gate transfers nothing this frame.
        if self.chest != -1 && !self.crafting_open {
            if let Some(abs) = ui::chest::slot_at(self.mouse_pos, self.chest_scroll_row) {
                self.chest_slot_click(abs, left, right_held);
                self.ui_left_press = false;
                self.ui_right_press = false;
                return true;
            }
            // `ChestUI.DrawButton`'s click dispatch (`ChestUI.cs:282-308`),
            // the rename pair: ID 5 → `RenameChest` (`:827-838` — open the
            // keyboard, or submit while the editor is live), ID 6 →
            // `RenameChestCancel` (`:863-869`). Buttons 0-4 are
            // M3-STORE-2's. The dispatch only exists inside `ChestUI.Draw`
            // (the panel's visibility gate above); the port's modal rename
            // keyboard parks the world routes, so the editing-state arms
            // (`Save`/`Cancel` clicks) are the overlay shape's revive point.
            if left {
                if let Some(id) = ui::chest::rename_button_hit(
                    &self.lookups,
                    self.mouse_pos,
                    &self.chest_button_scales,
                    &self.chest_button_hovered,
                    self.chest,
                    self.chest_editing(),
                ) {
                    match id {
                        ui::chest::BUTTON_RENAME => chest::rename_button_click(self),
                        ui::chest::BUTTON_RENAME_CANCEL => {
                            chest::rename_cancel(self);
                        }
                        _ => {}
                    }
                    self.ui_left_press = false;
                    return true;
                }
            }
        }
        // The open-inventory chest-tile click: only when no UI slot holds the
        // pointer (the C#'s `mouseInterface` rule — the drawn cell rects
        // `ChestUI.cs:382`, the inventory grid's hit test) and on the right
        // press. The left button stays the mine input.
        if right_edge && self.slot_at_mouse().is_none() {
            let (sw, sh) = self.screen_size();
            let world_pos = Vec2::new(
                self.camera.x + (self.mouse_pos.x - sw * 0.5) / self.zoom,
                self.camera.y + (self.mouse_pos.y - sh * 0.5) / self.zoom,
            );
            if chest::try_open_chest_at_cursor(self, world_pos) {
                self.ui_right_press = false;
                return true;
            }
        }
        false
    }

    /// `ItemSlot.OverrideHover` → `GetAlternateClickAction`'s shift arms
    /// (`ItemSlot.cs:644-726`), over the two ported surfaces: a hovered chest
    /// cell (context 3 → cursor 8 when the inventory can take the stack,
    /// `:692-698`) and a hovered inventory grid slot (context 0 → cursor 9
    /// when the open chest can take it, `:680-686` via
    /// `TryPlacingInChest(justCheck: true)`). The Control-in-use trash arm
    /// (`:637-643`) and the gamepad arms are out.
    fn evaluate_shift_cursor_override(&mut self) {
        if !shift_in_use(self) {
            return;
        }
        if self.chest != -1 && !self.crafting_open {
            if let Some(abs) = ui::chest::slot_at(self.mouse_pos, self.chest_scroll_row) {
                if let Some((kind, stack)) =
                    chest::chest_slot(&self.world_chests, self.chest as usize, abs)
                {
                    if kind != 0 && stack > 0 && chest::inventory_has_room_for(self, kind) {
                        self.cursor_override = CURSOR_FROM_CHEST;
                    }
                }
                return;
            }
        }
        if self.chest != -1 {
            if let Some(slot) = self.slot_at_mouse() {
                if chest::try_placing_in_chest(self, slot, true) {
                    self.cursor_override = CURSOR_TO_CHEST;
                }
            }
        }
    }

    /// One ctx-3 cell's click pair (`ItemSlot.Handle`'s `LeftClick` +
    /// `RightClick` order, `ItemSlot.cs:588-597`): the left press runs the
    /// cursor-8 shift transfer or the action-0 movement, the held right
    /// button runs the stack-split.
    fn chest_slot_click(&mut self, abs: usize, left: bool, right_held: bool) {
        if left && self.cursor_override == CURSOR_FROM_CHEST {
            // `OverrideLeftClick`'s cursor-8 arm (`ItemSlot.cs:800-812`):
            // `inv[slot] = player.GetItem(inv[slot], QuickTransferFromSlot)`;
            // consumes the click (the action-0 movement does not run).
            chest::quick_transfer_from_slot(self, abs);
            return;
        }
        if left {
            let mut held = self.mouse_item;
            let (kind, stack) =
                chest::chest_slot(&self.world_chests, self.chest as usize, abs).unwrap_or((0, 0));
            let mut target = if kind != 0 {
                ItemSlot {
                    kind,
                    name: registry::name(kind),
                    count: stack,
                    prefix: 0,
                }
            } else {
                EMPTY_SLOT
            };
            // `PickItemMovementAction` ctx 3 → action 0 (`ItemSlot.cs:1505-1507`)
            // — the shared movement.
            if left_click_movement(&mut held, &mut target) {
                self.audio.play("Grab"); // `:1010` — `SoundEngine.PlaySound(7)`
            }
            // `:976-979` — the emptied slot turns to air; the store keeps its
            // `(kind, stack)` encoding (the air-normalizing write).
            chest::set_chest_slot(
                &mut self.world_chests,
                self.chest as usize,
                abs,
                (target.kind, target.count),
            );
            self.mouse_item = held;
        }
        if right_held {
            self.chest_slot_stack_split(abs);
        }
    }

    /// `ItemSlot.RightClick`'s stack-split for context 3 (`ItemSlot.cs:
    /// 1636-1715` + `PickupItemIntoMouse` `:1717-1747`): one item per gated
    /// drawn frame while the right button is held — `superFastStack + 1` once
    /// the accelerator bottoms out — sound 12 per transfer and
    /// `RefreshStackSplitCooldown` (`:1774-1784`) after each. The ctx-0/3
    /// `SwapEquip` arm (`:1685-1697`) and the OpenableBag/HasItemSwap arms
    /// (`:1660-1675`) have no equip/swap state in the port.
    fn chest_slot_stack_split(&mut self, abs: usize) {
        if self.stack_split > 1 {
            return; // `:1700`
        }
        let transfers = self.super_fast_stack + 1; // `:1704`
        for _ in 0..transfers {
            let Some((kind, stack)) =
                chest::chest_slot(&self.world_chests, self.chest as usize, abs)
            else {
                break;
            };
            if kind == 0 || stack == 0 {
                break; // `:1700`'s `inv[slot].IsAir`
            }
            let max = registry::max_stack(kind);
            // `:1707` — `CanStack(mouseItem, inv[slot]) && mouseItem.stack <
            // maxStack`, or an air cursor (`Item.CanStack` is the type match,
            // `Item.cs:49865-49872`; the port has no prefix, decision #2).
            let can_take = (self.mouse_item.kind == kind && self.mouse_item.count < max)
                || self.mouse_item.count == 0;
            if !can_take {
                break;
            }
            // `PickupItemIntoMouse` (`:1717-1747`): an air cursor clones the
            // slot at stack 0, then +1/−1; the emptied slot turns to air
            // (`:1743-1746`). (The `:1728-1735` favorite carry is
            // unreachable — chest slots cannot be favorited.)
            if self.mouse_item.count == 0 {
                self.mouse_item = ItemSlot {
                    kind,
                    name: registry::name(kind),
                    count: 0,
                    prefix: 0,
                };
            }
            self.mouse_item.count += 1;
            let left_in_slot = stack - 1;
            chest::set_chest_slot(
                &mut self.world_chests,
                self.chest as usize,
                abs,
                (kind, left_in_slot),
            );
            self.audio.play("Menu_Tick"); // `:1710` — `SoundEngine.PlaySound(12)`
            // `RefreshStackSplitCooldown` (`:1774-1784`).
            self.stack_split = if self.stack_split == 0 {
                30
            } else {
                self.stack_delay
            };
            if left_in_slot == 0 {
                break;
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::chest::{ChestSlot, DEFAULT_MAX_ITEMS, WorldChest};
    use tr_core::World;

    /// Inside the chest grid's cell (0, 0) — the drawn 52·0.755 px cell at
    /// (73, 258) (`ui::chest::slot_at`).
    const CHEST_SLOT_0: Vec2 = Vec2::new(90.0, 270.0);
    /// Inside the chest grid's cell (1, 0).
    const CHEST_SLOT_1: Vec2 = Vec2::new(130.0, 270.0);
    /// Inside the inventory grid's cell (5, 0) — the 56·0.85 px pitch from
    /// (20, 20) (`App::slot_at_mouse`).
    const INV_SLOT_5: Vec2 = Vec2::new(280.0, 40.0);

    fn slot(kind: u16, count: u32) -> ItemSlot {
        ItemSlot {
            kind,
            name: registry::name(kind),
            count,
            prefix: 0,
        }
    }

    /// A headless app with the inventory open and an open chest (index 0,
    /// tile origin (20, 10)) whose store holds `slots`.
    fn chest_app(slots: Vec<ChestSlot>) -> App {
        let mut app = flow_app(None, vec![], vec![]);
        app.inventory_open = true;
        app.world_chests.push(WorldChest {
            tile: (20, 10),
            slots,
        });
        app.chest = 0;
        app.chest_x = 20;
        app.chest_y = 10;
        app
    }

    /// A 2×2 chest tile at `(x, y)` in the app's world (the phase-2
    /// `place_chest` shape).
    fn place_tile_chest(app: &mut App, x: i32, y: i32) {
        for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
            let i = app.world.index(x + dx, y + dy);
            app.world.tiles[i].kind = 21;
            app.world.tiles[i].frame_x = if dx == 1 { 18 } else { 0 };
            app.world.tiles[i].frame_y = if dy == 1 { 18 } else { 0 };
            app.world.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        }
    }

    /// The screen point whose camera mapping lands on tile `(x, y)`'s centre
    /// (the `update_editing` mapping with the fixture's camera 0/zoom 1 and
    /// the 1280×720 headless screen).
    fn tile_click(x: i32, y: i32) -> Vec2 {
        Vec2::new(x as f32 * 16.0 + 8.0 + 640.0, y as f32 * 16.0 + 8.0 + 360.0)
    }

    /// DW-4.1 — the ctx-3 left-click movements (`PickItemMovementAction`
    /// ctx 3 → action 0, `ItemSlot.cs:1505-1507`) match the inventory grid's
    /// semantics: both drive the shared [`left_click_movement`].
    #[test]
    fn dw_4_1_take_place_swap_stack_on_chest_slots() {
        let mut app = chest_app(vec![(9, 6)]);
        // Take: an empty cursor takes the whole stack.
        app.mouse_pos = CHEST_SLOT_0;
        app.ui_left_press = true;
        assert!(app.handle_chest_clicks());
        assert_eq!(app.mouse_item.kind, 9);
        assert_eq!(app.mouse_item.count, 6);
        assert_eq!(chest::chest_slot(&app.world_chests, 0, 0), Some((0, 0)));
        // Place: the cursor stack lands on an empty chest cell.
        app.mouse_item = slot(9, 3);
        app.mouse_pos = CHEST_SLOT_1;
        app.ui_left_press = true;
        assert!(app.handle_chest_clicks());
        assert_eq!(chest::chest_slot(&app.world_chests, 0, 1), Some((9, 3)));
        assert_eq!(app.mouse_item.count, 0);
        // Swap: a different kind exchanges with the cursor.
        app.mouse_item = slot(5, 2);
        app.ui_left_press = true;
        assert!(app.handle_chest_clicks());
        assert_eq!(chest::chest_slot(&app.world_chests, 0, 1), Some((5, 2)));
        assert_eq!(app.mouse_item.kind, 9);
        assert_eq!(app.mouse_item.count, 3);
        // Stack-merge: the same kind pours into the partial slot.
        app.mouse_item = slot(5, 4);
        app.ui_left_press = true;
        assert!(app.handle_chest_clicks());
        assert_eq!(chest::chest_slot(&app.world_chests, 0, 1), Some((5, 6)));
        assert_eq!(app.mouse_item.count, 0);
    }

    /// DW-4.1 — the merge caps at the kind's `registry::max_stack` (Wood's
    /// `Item.maxStack` is 9999, itemdata) and a full slot swaps instead of
    /// merging (`ItemSlot.cs:980-998`).
    #[test]
    fn dw_4_1_stack_merge_respects_max_stack_boundary() {
        let mut app = chest_app(vec![(9, 9990)]);
        app.mouse_pos = CHEST_SLOT_0;
        app.mouse_item = slot(9, 20);
        app.ui_left_press = true;
        assert!(app.handle_chest_clicks());
        assert_eq!(chest::chest_slot(&app.world_chests, 0, 0), Some((9, 9999)));
        assert_eq!(app.mouse_item.count, 11);
        // The now-full slot exchanges with the cursor stack.
        app.ui_left_press = true;
        assert!(app.handle_chest_clicks());
        assert_eq!(chest::chest_slot(&app.world_chests, 0, 0), Some((9, 11)));
        assert_eq!(app.mouse_item.count, 9999);
    }

    /// DW-4.2 — a full chest refuses the shift-in transfer: the merge pass
    /// finds no room (alien kinds at full stack, or same-kind full stacks)
    /// and the empty pass finds no empty slot, so both the `just_check`
    /// probe and the real pass are no-ops (`ChestUI.cs:1221-1287`).
    #[test]
    fn dw_4_2_shift_transfer_full_chest_is_noop() {
        let full_count = DEFAULT_MAX_ITEMS as usize;
        // Alien kinds at full stack.
        let mut app = chest_app(vec![(3, 9999); full_count]);
        app.inventory[3] = slot(9, 10);
        assert!(!chest::try_placing_in_chest(&mut app, 3, true));
        assert!(!chest::try_placing_in_chest(&mut app, 3, false));
        assert_eq!(app.inventory[3].count, 10);
        assert_eq!(app.world_chests[0].slots, vec![(3, 9999); full_count]);
        // Same kind, every stack full: the merge skip rule (`:1225`).
        let mut app = chest_app(vec![(9, 9999); full_count]);
        app.inventory[3] = slot(9, 10);
        assert!(!chest::try_placing_in_chest(&mut app, 3, false));
        assert_eq!(app.inventory[3].count, 10);
    }

    /// DW-4.2 — `just_check` answers placeability without mutating the
    /// inventory, the store or the audio state (`ChestUI.cs:1234-1237`,
    /// `:1271-1274`).
    #[test]
    fn dw_4_2_just_check_mutates_nothing() {
        let mut app = chest_app(vec![(9, 5)]);
        app.inventory[3] = slot(9, 10);
        let store_before = app.world_chests[0].slots.clone();
        let inv_before: Vec<(u16, u32)> = app.inventory.iter().map(|s| (s.kind, s.count)).collect();
        assert!(chest::try_placing_in_chest(&mut app, 3, true));
        assert_eq!(app.world_chests[0].slots, store_before);
        let inv_after: Vec<(u16, u32)> = app.inventory.iter().map(|s| (s.kind, s.count)).collect();
        assert_eq!(inv_after, inv_before);
    }

    /// `TryPlacingInChest`'s happy path (`ChestUI.cs:1221-1287`): the merge
    /// pass pours into every partial same-kind slot in order, then the first
    /// empty slot takes the whole remainder. The C# `flag2` return is only
    /// set by the `justCheck` branches — the real pass always answers false
    /// (its only caller ignores the value, `ItemSlot.cs:846`).
    #[test]
    fn try_placing_in_chest_merges_then_fills_the_first_empty_slot() {
        // Everything merges into the one partial slot.
        let mut app = chest_app(vec![(9, 5)]);
        app.inventory[3] = slot(9, 12);
        assert!(!chest::try_placing_in_chest(&mut app, 3, false));
        assert_eq!(app.world_chests[0].slots[0], (9, 17));
        assert_eq!(app.inventory[3].count, 0);
        // A remainder overflows into the first empty slot only.
        let mut app = chest_app(vec![(9, 9990)]);
        app.inventory[3] = slot(9, 20);
        assert!(!chest::try_placing_in_chest(&mut app, 3, false));
        assert_eq!(app.world_chests[0].slots[0], (9, 9999));
        assert_eq!(app.world_chests[0].slots[1], (9, 11));
        assert_eq!(app.inventory[3].count, 0);
        // Two partials soak their room in slot order before the empty pass.
        let mut app = chest_app(vec![(9, 9995), (0, 0), (9, 9995)]);
        app.inventory[3] = slot(9, 12);
        assert!(!chest::try_placing_in_chest(&mut app, 3, false));
        assert_eq!(app.world_chests[0].slots[0], (9, 9999));
        assert_eq!(app.world_chests[0].slots[1], (9, 4));
        assert_eq!(app.world_chests[0].slots[2], (9, 9999));
        assert_eq!(app.inventory[3].count, 0);
    }

    /// `TryPlacingInChest` answers false with no chest open
    /// (`GetCurrentContainer`'s world-chest arm, `ChestUI.cs:1213`; the banks
    /// are M3-STORE-1).
    #[test]
    fn try_placing_in_chest_refuses_without_an_open_chest() {
        let mut app = flow_app(None, vec![], vec![]);
        app.inventory_open = true;
        app.inventory[3] = slot(9, 10);
        assert!(!chest::try_placing_in_chest(&mut app, 3, false));
    }

    /// DW-4.3 — the accelerator ladder (`Main.cs:61781-61807`): the first
    /// step consumes 30 frames (delay 7 → 6), the ladder bottoms at 2, and
    /// `superFastStack` starts accumulating on the 11th decay (~2 s in).
    #[test]
    fn dw_4_3_stack_split_timing_constants() {
        let mut app = flow_app(None, vec![], vec![]);
        app.tick_stack_split(); // the idle arm installs stackDelay 7
        app.stack_split = 1;
        for _ in 0..30 {
            app.tick_stack_split();
        }
        assert_eq!(app.stack_delay, 6, "the first step is 30 frames");
        assert_eq!(app.stack_counter, 0);
        for _ in 0..100 {
            app.tick_stack_split();
        }
        // 25+20+15+10 = 70 frames bottom the delay at 2 (steps 2-5, whose
        // superFast the `timesTriedToFastStack < 10` guard keeps resetting),
        // then six 5-frame steps land on the 11th decay — the first one that
        // accumulates.
        assert_eq!(app.stack_delay, 2);
        assert_eq!(app.super_fast_stack, 1);
        for _ in 0..5 {
            app.tick_stack_split();
        }
        assert_eq!(app.super_fast_stack, 2, "one more per 5 frames");
        // The idle reset re-installs the constants (`:61782-61787`).
        app.stack_split = 0;
        app.tick_stack_split();
        assert_eq!(app.stack_delay, 7);
        assert_eq!(app.super_fast_stack, 0);
        assert_eq!(app.times_tried_to_fast_stack, 0);
    }

    /// DW-4.3 — a stack of 1 right-clicks whole into the cursor and arms the
    /// first 30-frame cooldown (`PickupItemIntoMouse` `:1717-1747`).
    #[test]
    fn dw_4_3_right_click_stack_one_takes_one() {
        let mut app = chest_app(vec![(9, 1)]);
        app.mouse_pos = CHEST_SLOT_0;
        app.place_down = true;
        app.ui_right_press = true;
        assert!(app.handle_chest_clicks());
        assert_eq!(app.mouse_item.kind, 9);
        assert_eq!(app.mouse_item.count, 1);
        assert_eq!(chest::chest_slot(&app.world_chests, 0, 0), Some((0, 0)));
        assert_eq!(app.stack_split, 30);
    }

    /// DW-4.3 — the held repeat is paced: no transfer while `stackSplit > 1`
    /// (`:1700`), then one per frame and `RefreshStackSplitCooldown` arms
    /// `stackDelay` (`:1774-1784`).
    #[test]
    fn dw_4_3_right_click_repeat_paced_by_stack_split() {
        let mut app = chest_app(vec![(9, 5)]);
        // The accelerator's idle arm has installed `stackDelay` 7 by the time
        // any repeat runs (`Main.cs:61782-61787`).
        app.stack_delay = 7;
        app.mouse_pos = CHEST_SLOT_0;
        app.place_down = true;
        app.ui_right_press = true;
        assert!(app.handle_chest_clicks());
        assert_eq!(app.mouse_item.count, 1);
        assert_eq!(app.stack_split, 30);
        // The gate blocks the next frames while the cooldown runs.
        assert!(app.handle_chest_clicks());
        assert_eq!(app.mouse_item.count, 1);
        for _ in 0..29 {
            app.decay_stack_split();
        }
        assert_eq!(app.stack_split, 1);
        app.ui_right_press = true;
        assert!(app.handle_chest_clicks());
        assert_eq!(app.mouse_item.count, 2);
        assert_eq!(app.stack_split, 7, "the repeat arms stackDelay");
    }

    /// The split honours the cursor's room and kind: a full same-kind cursor,
    /// an alien cursor or a drained slot transfers nothing
    /// (`ItemSlot.cs:1707`).
    #[test]
    fn right_click_split_respects_the_cursor_room_and_kind() {
        // A full cursor of the same kind cannot take more.
        let mut app = chest_app(vec![(9, 5)]);
        app.mouse_pos = CHEST_SLOT_0;
        app.mouse_item = slot(9, 9999);
        app.place_down = true;
        app.handle_chest_clicks();
        assert_eq!(app.mouse_item.count, 9999);
        assert_eq!(chest::chest_slot(&app.world_chests, 0, 0), Some((9, 5)));
        // An alien cursor never splits.
        let mut app = chest_app(vec![(9, 5)]);
        app.mouse_pos = CHEST_SLOT_0;
        app.mouse_item = slot(3, 1);
        app.place_down = true;
        app.handle_chest_clicks();
        assert_eq!(app.mouse_item.kind, 3);
        assert_eq!(chest::chest_slot(&app.world_chests, 0, 0), Some((9, 5)));
    }

    /// DW-4.5 — the shift hover evaluation (`GetAlternateClickAction`,
    /// `ItemSlot.cs:644-726`): cursor 8 on a chest cell with inventory room,
    /// 9 on an inventory slot with a placeable stack, −1 without Shift and
    /// −1 when nothing has room. (The Cursors_8/9 texture draw is the
    /// recorded follow-up — the state is what the click arms consume.)
    #[test]
    fn dw_4_5_cursor_override_eval_per_hover_and_shift() {
        let mut app = chest_app(vec![(9, 5)]);
        app.keys.insert(KeyCode::ShiftLeft);
        app.mouse_pos = CHEST_SLOT_0;
        app.handle_chest_clicks();
        assert_eq!(app.cursor_override, CURSOR_FROM_CHEST);
        // An inventory grid slot with a placeable stack → 9.
        app.inventory[5] = slot(9, 10);
        app.mouse_pos = INV_SLOT_5;
        app.handle_chest_clicks();
        assert_eq!(app.cursor_override, CURSOR_TO_CHEST);
        // Without Shift the evaluation stays off.
        app.keys.remove(&KeyCode::ShiftLeft);
        app.mouse_pos = CHEST_SLOT_0;
        app.handle_chest_clicks();
        assert_eq!(app.cursor_override, -1);
        // No room anywhere in the inventory → no cursor 8.
        app.keys.insert(KeyCode::ShiftRight);
        for s in app.inventory.iter_mut() {
            *s = slot(3, 9999);
        }
        app.handle_chest_clicks();
        assert_eq!(app.cursor_override, -1);
    }

    /// The cursor-9 shift-click moves the stack into the chest
    /// (`OverrideLeftClick` `ItemSlot.cs:813-850` → `TryPlacingInChest(justCheck:
    /// false)` `:846`) and consumes the press.
    #[test]
    fn shift_click_transfers_inventory_into_the_chest() {
        let mut app = chest_app(vec![(9, 5)]);
        app.inventory[5] = slot(9, 10);
        app.mouse_pos = INV_SLOT_5;
        app.keys.insert(KeyCode::ShiftLeft);
        app.ui_left_press = true;
        assert!(app.handle_chest_clicks());
        assert_eq!(app.world_chests[0].slots[0], (9, 15));
        assert_eq!(app.inventory[5].count, 0);
        assert!(!app.ui_left_press, "the press is consumed");
    }

    /// The cursor-8 shift-click drains a chest cell into the inventory
    /// (`OverrideLeftClick` `ItemSlot.cs:800-812` → `GetItem`): the partial
    /// grid stack soaks what it has room for, the remainder lands on the
    /// reverse-scanned empty slot (49 — the hotbar pass finds none with
    /// everything else full), and nothing stays behind.
    #[test]
    fn shift_click_from_chest_slot_transfers_to_the_inventory() {
        let mut app = chest_app(vec![(9, 500)]);
        for s in app.inventory.iter_mut() {
            *s = slot(3, 9999);
        }
        app.inventory[0] = slot(9, 9995); // room for 4
        app.inventory[49] = EMPTY_SLOT; // the reverse pass's first hit
        app.mouse_pos = CHEST_SLOT_0;
        app.keys.insert(KeyCode::ShiftLeft);
        app.ui_left_press = true;
        assert!(app.handle_chest_clicks());
        assert_eq!(app.inventory[0].count, 9999);
        assert_eq!(app.inventory[49].kind, 9);
        assert_eq!(app.inventory[49].count, 496);
        assert_eq!(
            chest::chest_slot(&app.world_chests, 0, 0),
            Some((0, 0)),
            "the whole stack moved"
        );
    }

    /// The batch-review carry — the open-inventory chest-tile click: the same
    /// chest's right-click toggles closed (`Player.cs:33230-33252`) and the
    /// press is consumed.
    #[test]
    fn re_click_on_the_open_chest_tile_closes_it() {
        let mut app = chest_app(vec![(9, 1)]);
        app.world = World::new("t".into(), 120, 120);
        place_tile_chest(&mut app, 20, 10);
        app.mouse_pos = tile_click(20, 10);
        app.ui_right_press = true;
        app.place_down = true;
        assert!(app.handle_chest_clicks());
        assert_eq!(app.chest, -1, "the re-click closes (:33236)");
        assert!(!app.ui_right_press, "the press is consumed");
    }

    /// The carry's switch half — a right-click on another chest's tile opens
    /// it in place of the open one (`:33248-33251`, the re-open sound 12).
    #[test]
    fn right_click_on_another_chest_switches_while_open() {
        let mut app = chest_app(vec![]);
        app.world = World::new("t".into(), 120, 120);
        place_tile_chest(&mut app, 20, 10);
        place_tile_chest(&mut app, 40, 10);
        app.world_chests.push(WorldChest {
            tile: (40, 10),
            slots: vec![],
        });
        app.chest = 0;
        app.mouse_pos = tile_click(40, 10);
        app.ui_right_press = true;
        app.place_down = true;
        assert!(app.handle_chest_clicks());
        assert_eq!(app.chest, 1, "the store index of the (40, 10) chest");
        assert_eq!((app.chest_x, app.chest_y), (40, 10));
    }

    /// The inventory grid's left click still moves items through the shared
    /// helper — the extraction's characterization pin (DW-4.5's "inventory
    /// clicks still green" half).
    #[test]
    fn inventory_left_clicks_move_through_the_shared_helper() {
        let mut app = flow_app(None, vec![], vec![]);
        app.inventory_open = true;
        app.inventory[0] = slot(9, 6);
        app.mouse_pos = Vec2::new(30.0, 30.0); // grid (0, 0)
        app.ui_left_press = true;
        app.handle_inventory_clicks();
        assert_eq!(app.mouse_item.count, 6);
        assert_eq!(app.inventory[0].count, 0);
    }

    // --- Phase 5: the rename button dispatch -----------------------------

    /// `ChestUI.DrawButton`'s click dispatch (`ChestUI.cs:282-308`): the ID-5
    /// click opens the ctx-2 keyboard (`RenameChest` :827-838 →
    /// `IngameFancyUI.OpenVirtualKeyboard(2)`), the ID-6 click (the editor is
    /// live) cancels it (`RenameChestCancel` :863-869).
    #[test]
    fn dw_5_rename_button_click_routes_to_the_keyboard_and_cancel() {
        let mut app = chest_app(vec![]);
        // A real style-0 chest tile at the open origin, so the prefill
        // decodes the tile default (`IngameFancyUI.cs:103-107`).
        place_tile_chest(&mut app, 2, 2);
        app.chest_x = 2;
        app.chest_y = 2;
        // The rename button's row anchor (506, invBottom + 40 + 5*26) at the
        // idle scale (the static-ctor 0.0 only precedes the first drawn
        // frame), probed a few px in (the headless font estimate).
        app.chest_button_scales = [ui::chest::BUTTON_SCALE_MIN; ui::chest::BUTTON_COUNT];
        app.mouse_pos = Vec2::new(510.0, 428.0);
        app.ui_left_press = true;
        assert!(app.handle_chest_clicks(), "the button took the click");
        assert!(
            app.chest_editing(),
            "the rename keyboard opened (the ChestRename tail session)"
        );
        assert!(
            matches!(
                app.front.as_ref(),
                Some(crate::app::flow::FrontScreen::Keyboard(s))
                    if s.tail == crate::app::flow::KeyboardTail::ChestRename
                        && s.request.context == ui::keyboard::Context::Chest
            ),
            "the session is the chest-rename keyboard"
        );
        assert_eq!(app.chest_edit_text(), "Chest", "the tile default prefills");
        // The cancel button's row (ID 6, y = 454) is only live while editing;
        // the click cancels the editor back into gameplay.
        app.mouse_pos = Vec2::new(510.0, 454.0);
        app.ui_left_press = true;
        assert!(app.handle_chest_clicks(), "the cancel button took the click");
        assert!(!app.chest_editing(), "the editor closed");
        assert_eq!(app.front, None, "back into gameplay");
        assert!(app.chest_names.is_empty(), "the cancel never wrote");
    }

    /// The editing-state Save arm (`ChestUI.RenameChest`'s else, `:834-837`):
    /// clicking ID 5 while the editor is live submits the live text.
    #[test]
    fn rename_button_click_submits_while_editing() {
        let mut app = chest_app(vec![]);
        place_tile_chest(&mut app, 2, 2);
        app.chest_x = 2;
        app.chest_y = 2;
        app.chest_button_scales = [ui::chest::BUTTON_SCALE_MIN; ui::chest::BUTTON_COUNT];
        app.mouse_pos = Vec2::new(510.0, 428.0);
        app.ui_left_press = true;
        assert!(app.handle_chest_clicks());
        assert!(app.chest_editing());
        // Type, then click the Save-labelled button again.
        if let Some(crate::app::flow::FrontScreen::Keyboard(s)) = app.front.as_mut() {
            s.state.text.clear();
            ui::keyboard::write_str(&mut s.state, "Saved");
        }
        app.mouse_pos = Vec2::new(510.0, 428.0);
        app.ui_left_press = true;
        assert!(app.handle_chest_clicks());
        assert!(!app.chest_editing(), "the Save click submitted");
        assert_eq!(chest::chest_name(&app, 0), "Saved");
    }
}
