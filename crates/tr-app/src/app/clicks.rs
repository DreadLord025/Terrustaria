//! The App's HUD click handling — the slot/hotbar hit tests, the minimap
//! frame's buttons, the inventory stack moves and the legacy-menu action
//! dispatch. (A6: extracted from `main.rs`; the front screens' click tables
//! live in [`crate::app::flow`].)

use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;

use crate::achievements;
use crate::app::flow::{FlowClick, FrontScreen, next_flow_state};
use crate::app::{App, NetRole};
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
/// The favorite override's cursor id — `Main.cursorOverride = 3`
/// (`ItemSlot.cs:616-619`).
const CURSOR_FAVORITE: i32 = 3;
/// `AlternateClickAction.Trash`'s cursor id (`ItemSlot.cs:213`): the
/// Shift-click quick-trash (`GetAlternateClickAction`'s tail, `:679-682`).
const CURSOR_TRASH: i32 = 6;

/// `ItemSlot.ShiftInUse` (`ItemSlot.cs:328-338`): the physical Shift keys are
/// `Main.keyState.PressingShift()`'s port (`app.keys` tracks the pressed
/// codes, `app/loop.rs:117`/:402). The `ShiftForcedOn` gamepad/test static is
/// out.
fn shift_in_use(app: &App) -> bool {
    app.keys.contains(&KeyCode::ShiftLeft) || app.keys.contains(&KeyCode::ShiftRight)
}

/// `Main.FavoriteKey` (`Main.cs:67196` — `Keys.LeftAlt`; the binding string
/// `cFavoriteKey = "LeftAlt"` sits at `Main.cs:67177`), read live like the
/// C#'s `Main.keyState.IsKeyDown(Main.FavoriteKey)` (`ItemSlot.cs:611`) —
/// the same pressed-codes set `shift_in_use` reads. The
/// `KeyFavoriteModifier` rebind setting has no port surface.
fn favorite_key_held(app: &App) -> bool {
    app.keys.contains(&KeyCode::AltLeft)
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
    } else if target.kind == held.kind && target.prefix == held.prefix && target.count < max {
        // The merge gate is `Item.CanStack` (`ItemSlot.cs:1000`'s case-0
        // merge, `Item.cs:49865-49871`) — same type AND same prefix; a
        // prefixed stack never blends into an unprefixed one.
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

/// `ItemID.CopperCoin..PlatinumCoin` — the four kinds the ctx-1 row takes
/// (`ItemSlot.PickItemMovementAction` `:1369-1371`).
fn is_coin_kind(kind: u16) -> bool {
    (71..=74).contains(&kind)
}

/// `Item.FitsAmmoSlot` (`Item.cs:1369-1376`) over the flags the port carries:
/// air, an ammo item (`ammo > 0`) or bait. The C# also takes the paint-and-
/// coating arm and the three literal exceptions (353/530/849), and honours
/// `notAmmo`; none of those has a port counterpart yet.
fn fits_ammo_slot(kind: u16) -> bool {
    if kind == 0 {
        return true;
    }
    registry::get(kind).is_some_and(|d| d.ammo > 0 || d.bait > 0)
}

/// The accepted-move gate `PickItemMovementAction` gives a context before the
/// shared action-0 move runs (`:1341-1379`): context 0/3 always allow, the
/// ctx-1 coin row takes coins, the ctx-2 ammo row takes the ammo set.
pub(crate) fn context_accepts(context: i32, kind: u16) -> bool {
    match context {
        1 => kind == 0 || is_coin_kind(kind),
        2 => fits_ammo_slot(kind),
        _ => true,
    }
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

    /// The coin (ctx 1) or ammo (ctx 2) cell under the cursor —
    /// `Main.DrawInventory`'s rows (`Main.cs:41752-41782`) through the shared
    /// `ui::inventory::coin_ammo_hit` geometry. Closed inventory: nothing.
    pub(crate) fn coin_ammo_at_mouse(&self) -> Option<(i32, usize)> {
        if !self.inventory_open || self.bestiary_open {
            return None;
        }
        let cell = self.lookups.ui_back.map_or(52.0, |b| b.width as f32);
        ui::inventory::coin_ammo_hit(self.mouse_pos, cell)
    }

    /// The [`chest::Source`] under the cursor for the shift-transfer arms: the
    /// coin/ammo rows resolve through their own cells, everything else through
    /// the grid. `None` when the pointer is over neither.
    fn chest_transfer_source(&self) -> Option<chest::Source> {
        // `GetAlternateClickAction`'s head guard (`ItemSlot.cs:631-634`): air or
        // a favorited slot answers null for every context. The rows keep no
        // favorites yet, so only the air half is theirs (`try_placing_from`
        // refuses an empty source).
        if let Some((ctx, k)) = self.coin_ammo_at_mouse() {
            return Some(if ctx == 1 {
                chest::Source::Coin(k)
            } else {
                chest::Source::Ammo(k)
            });
        }
        let slot = self.slot_at_mouse()?;
        if self.is_favorite(slot) {
            return None;
        }
        Some(chest::Source::Inv(slot))
    }

    /// One coin/ammo cell's press — `ItemSlot.Handle` over the ctx-1/ctx-2
    /// cells: left runs the shared action-0 move behind
    /// `PickItemMovementAction`'s gate, right the `:1700-1772` split. The Alt
    /// favorite arm stays out: the port keeps favorites as inventory indices
    /// (`App.favorite_slots`), so the row's `canFavoriteAt[1]/[2]` has no key
    /// space yet (the recorded gap in ANALYSIS).
    fn coin_ammo_click(&mut self, context: i32, k: usize, left: bool) {
        if k >= crate::inv::COINS {
            return;
        }
        let mut held = self.mouse_item;
        let mut target = if context == 1 {
            self.equip.coins[k]
        } else {
            self.equip.ammo[k]
        };
        if left {
            // `PickItemMovementAction`: a refused move never runs, and the
            // held stack stays on the cursor (the C# `result == -1` falls to
            // the sound-only tail, `:1004-1011`).
            if !context_accepts(context, held.kind) {
                return;
            }
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
            if !context_accepts(context, held.kind) {
                return;
            }
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
        } else if target.kind == held.kind && target.count < registry::max_stack(target.kind) {
            target.count += 1;
            held.count -= 1;
            if held.count == 0 {
                held = EMPTY_SLOT;
            }
        }
        self.mouse_item = held;
        if context == 1 {
            self.equip.coins[k] = target;
        } else {
            self.equip.ammo[k] = target;
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
            && !crate::env::no_hud()
            && self.map.as_ref().is_some_and(|m| !m.open)
            && self.map_style == 1;
        if !live {
            self.minimap_hover = None;
            return;
        }
        let (sw, sh) = self.screen_size();
        let screen = Vec2::new(sw, sh);
        // `Main.MapScale` (`Main.cs:67297`, clamp `:4494`) — the same value the
        // draw branch reads (`app/frame.rs`): the panel is laid out in the scaled
        // batch's transform space (`Main.cs:55309-55329`), so the pointer is
        // divided by it here exactly as `Main.cs:55512-55513` does
        // (`mouseX = (int)(mouseX * num16)`, `num16 = 1 / MapScale`).
        let map_scale = self.settings.map_scale;
        // The frame position from the same formula as the draw branch
        // (`Main.cs:55546`'s `DrawTo` position + `_frameOffset`).
        let pos = ui::minimap::draw_to_position(ui::minimap::minimap_origin_raw(screen, map_scale));
        let s = ui::minimap::style_index(&self.settings.minimap_frame);
        let frame_pos = ui::minimap::frame_position(pos, &ui::minimap::FRAME_STYLES[s]);
        let sizes = ui::minimap::button_sizes(&self.lookups, s);
        // `Main.mouseLeft` is the held state (`MinimapFrame.cs:169`).
        let action = ui::minimap::update(
            self.mouse_pos / map_scale,
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
        // `AchievementsHelper.NotifyItemCraft(Recipe)` (`:73-79`) — the
        // crafted item's id rides the event, so `BENCHED`'s workbench set
        // matches the crafted item and not any craft.
        let _ = self.achievements.notify_craft(kind);
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
        // `Main.DrawSettingButton`'s click arm (`Main.cs:45532-45580`,
        // BUG-11): the Settings button opens (and, while the window is open,
        // closes) the in-game options window — `IngameOptions.Open` /
        // `Close` (`IngameOptions.cs:56`/`:84`).
        if self.ingame_options.open {
            let (sw, sh) = self.screen_size();
            let screen = Vec2::new(sw, sh);
            if ui::ingame_options::click(self, screen, self.mouse_pos) {
                self.ui_left_press = false;
                self.inventory_open = false;
                let _ = self.settings.save();
                return;
            }
        } else if self.inventory_open
            && self.ingame_options.button_layout.hit(self.mouse_pos)
        {
            self.ingame_options.open = true;
            self.ui_left_press = false;
            return;
        }
        // `DrawEmoteBubblesButton`'s click arm (`Main.cs:42400-42405`):
        // open the emotes wheel.
        if self.inventory_open && ui::bestiary::emote_button_hit(self.mouse_pos) {
            self.inventory_open = false;
            // `UIEmotesMenu.OnActivate` rebuilds the page, so the wheel opens
            // with its scroll at 0 (`EmoteSystem::open`).
            self.emotes.open();
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
                self.emotes.close();
                self.ui_left_press = false;
                return;
            }
            if ui::bestiary::back_hit(screen, self.mouse_pos) {
                self.emotes.close();
                self.ui_left_press = false;
            }
        }
    }

    /// The achievements menu's clicks (`UIAchievementsMenu`'s
    /// `GoBackClick`/`FilterList`/`FadedMouseOver`): the Back panel closes
    /// the overlay, a category toggle flips the filter, the wheel scrolls
    /// the list. Consumes the press on every hit.
    pub(crate) fn handle_achievements_clicks(&mut self) {
        let left = self.ui_left_press;
        if !left {
            return;
        }
        if self.achievements_open {
            let (sw, sh) = self.screen_size();
            let screen = Vec2::new(sw, sh);
            // `OnActivate`'s `Main.gameMenu` arm decides the container's Top.
            let game_menu = self.menu.is_some();
            let panel = ui::achievements::panel_rect(screen, game_menu);
            // The Back panel closes the state (`GoBackClick` -> `IngameFancyUI.Close()`).
            if ui::achievements::back_hit(&panel, self.mouse_pos) {
                self.achievements_open = false;
                self.achievements_categories = [true; 4];
                self.achievements_scroll = 0.0;
                self.ui_left_press = false;
                return;
            }
            // `FilterList` — a toggle flips and the list rebuilds.
            if let Some(j) = ui::achievements::category_at(&panel, self.mouse_pos) {
                self.achievements_categories[j] = !self.achievements_categories[j];
                self.achievements_scroll = 0.0;
                self.ui_left_press = false;
                return;
            }
            self.ui_left_press = false;
        }
    }

    /// The achievements list's wheel scroll (`UIList`'s scrollbar, driven by
    /// the mouse wheel here — `UIScrollbar.SetView(100f, 1000f)` is the
    /// vanilla control; the port's wheel is the documented stand-in).
    pub(crate) fn scroll_achievements(&mut self, delta: f32) {
        if !self.achievements_open {
            return;
        }
        let (sw, sh) = self.screen_size();
        let screen = Vec2::new(sw, sh);
        let game_menu = self.menu.is_some();
        let panel = ui::achievements::panel_rect(screen, game_menu);
        let list = ui::achievements::list_rect(&panel);
        let rows = self
            .achievements
            .list
            .iter()
            .filter(|a| ui::achievements::filter_visible(&self.achievements_categories, a.category))
            .count();
        let max = ui::achievements::max_scroll(rows, &list);
        self.achievements_scroll = (self.achievements_scroll - delta * (ui::achievements::ROW_H + 5.0))
            .clamp(0.0, max);
    }

    pub(crate) fn handle_inventory_clicks(&mut self) {
        let left = self.ui_left_press;
        let right = self.ui_right_press;
        self.ui_left_press = false;
        self.ui_right_press = false;
        if !left && !right {
            return;
        }
        // `Main.DrawPVPIcons`'s click handling (`Main.cs:42678-42694`, `:42724-42767`):
        // the PVP toggle and the six team cells. Both sit inside the open
        // inventory (`DrawInventory`'s tail, `:41003`) and consume the press
        // before the slot grid. The gates, in the C# order: the
        // `hostile`/`team` change needs `teamCooldown == 0`; the team change
        // also needs the cell to hold a different team and
        // `Player.TeamChangeAllowed()` (a `return true` stub in 1.4.5.8,
        // `Player.cs:4517-4520`, kept as the gate it is).
        if left && self.inventory_open {
            let (sw, sh) = self.screen_size();
            let screen = Vec2::new(sw, sh);
            // `Main.mH` as `frame()` hands it to the draw (`Main.cs:45771-45783`).
            let m_h = self.main_m_h();
            let state = self.pvp_icon_state();
            if let Some(hit) = ui::chrome::pvp_hit(
                screen,
                self.equip_page,
                m_h,
                &state,
                self.mouse_pos,
                ui::chrome::pvp_team_cell(&self.lookups),
            ) {
                // The gate chain (`:42689-42694`, `:42753-42767`) is
                // `ui::chrome::pvp_click`'s job — this arm is the effect.
                match ui::chrome::pvp_click(&state, &hit) {
                    Some(ui::chrome::PvpClick::ToggleHostile) => {
                        self.team_cooldown = ui::chrome::TEAM_COOLDOWN_LEN;
                        self.audio.play("Menu_Tick"); // `SoundEngine.PlaySound(12)`
                        if let Some(p) = self.player.as_mut() {
                            p.hostile = !p.hostile;
                        }
                        // `NetMessage.SendData(30, -1, -1, null, myPlayer)` —
                        // the PVP-state broadcast. M6-NET's message table has
                        // no 30 (`net.rs`); the local flip is the whole effect
                        // in SP (ANALYSIS).
                    }
                    Some(ui::chrome::PvpClick::SetTeam(i)) => {
                        self.team_cooldown = ui::chrome::TEAM_COOLDOWN_LEN;
                        self.audio.play("Menu_Tick"); // `SoundEngine.PlaySound(12)`
                        if let Some(p) = self.player.as_mut() {
                            p.team = i;
                        }
                        // `:42765-42770`: `SendData(157)` in MP, else the
                        // team-based-spawns respawn in SP (`teamBasedSpawnsSeed`
                        // is not ported — no team spawn seeds yet).
                    }
                    None => {}
                }
                return;
            }
            // `Main.DrawPageIcons`' click handling (`Main.cs:42828-42858`): the
            // three equipment-page tabs. Pressing the selected tab falls back to
            // page 0; pressing another selects it. The effect is scoped to
            // `Main.EquipPageSelected` — `Main.EquipPage` is re-derived every
            // frame in `frame()` (`:42864-42867`).
            let num20 = 174.0 + m_h;
            if let Some(tab) = ui::chrome::page_tab_hit(
                screen,
                num20 - 32.0,
                self.mouse_pos,
                (self.mouse_item.kind, self.mouse_item.count),
                false,
                ui::chrome::page_tab_sizes(&self.lookups),
            ) {
                if let Some(sel) = ui::chrome::page_tab_click(tab, self.equip_page_selected) {
                    self.equip_page_selected = sel;
                    // `SoundEngine.PlaySound(10)` (`:42857`) — the port's cue.
                    self.audio.play("Menu_Open");
                }
                return;
            }
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
                            // `ChestUI.LootAll`'s bank arm (:417-473) —
                            // `LootAllFromBank` is the default settings
                            // (`GetItemSettings.cs:55`): no vault arm, so
                            // `None` (the overflow stays in the bank).
                            0 => crate::bank::loot_all(
                                &mut p.banks[bi],
                                &mut self.inventory,
                                &mut self.equip.coins,
                                None,
                            ),
                            // `ChestUI.DepositAll` (`:540-659`) — the
                            // MoveCoins pre-pass runs for every container
                            // kind (:543-562).
                            1 => {
                                crate::bank::move_coins(
                                    &mut p.banks[bi],
                                    &mut self.inventory,
                                    &mut self.equip.coins,
                                    &fav,
                                );
                                crate::bank::deposit_all(
                                    &mut p.banks[bi],
                                    &mut self.inventory,
                                    &fav,
                                );
                            }
                            // `ChestUI.QuickStack`'s bank arm (:670-685) —
                            // the MoveCoins pre-pass rides the dispatch
                            // (hoisted out of `quick_stack`, M3-CHEST-INT).
                            2 => {
                                crate::bank::move_coins(
                                    &mut p.banks[bi],
                                    &mut self.inventory,
                                    &mut self.equip.coins,
                                    &fav,
                                );
                                crate::bank::quick_stack(
                                    &mut p.banks[bi],
                                    &mut self.inventory,
                                    &mut self.equip.coins,
                                    &fav,
                                );
                            }
                            _ => {
                                let _ = crate::bank::restock(
                                    &mut p.banks[bi],
                                    &mut self.inventory,
                                    &mut self.equip.ammo,
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
        // The trash slot's left click (`ItemSlot.LeftClick`'s context-6 arm,
        // `ItemSlot.cs:937-941`): `MouseItemSwap(ref trashItem, 6)` — the
        // plain held/trash swap (no favorited-held guard exists in the C#
        // swap; the can't-drop rule lives in `DropSelectedItem`,
        // `Player.cs:5170-5184`, the sell/drop pass's seam). The swap plays
        // 7 when an item sits on either side after it (`:1008-1011`, the
        // same cue the grid movement uses); the separate `TrashItem` cue
        // (`:1007`) has no port cue — the recorded gap. Right-click stays
        // unported (the Journey research arm `:940-943`).
        let chest_or_shop = self.open_container() >= 0 || self.shop_npc.is_some();
        if left && self.inventory_open && ui::inventory::trash_slot_at(self.mouse_pos, chest_or_shop) {
            self.trash_swap();
            if self.mouse_item.count > 0 || self.equip.trash.count > 0 {
                self.audio.play("Grab");
            }
            return;
        }
        // The coin (ctx 1) and ammo (ctx 2) rows: `Main.DrawInventory` handles
        // every cell it draws (`Main.cs:41752-41782`), and the shared grid
        // probe below knows nothing about them (`slot_at_mouse` covers the
        // 10x5 grid and the closed hotbar).
        if let Some((ctx, k)) = self.coin_ammo_at_mouse() {
            self.coin_ammo_click(ctx, k, left);
            return;
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
        // `OverrideLeftClick`'s cursor-3 arm (`ItemSlot.cs:745-755`): the
        // favorite toggle consumes the left click before the movement arms
        // (`LeftClick` returns on `OverrideLeftClick`'s `true`,
        // `:915-918`). `cursor_override == 3` is the Alt+hover gate
        // `evaluate_cursor_override` set this frame, so Alt held never falls
        // through to the pickup arm.
        if left && self.cursor_override == CURSOR_FAVORITE {
            if self.toggle_favorite(slot) != 0 {
                self.audio.play("Menu_Tick"); // `SoundEngine.PlaySound(12)`
            }
            return;
        }
        if left && self.cursor_override == CURSOR_TRASH {
            // `OverrideLeftClick`'s cursor-6 arm (`ItemSlot.cs:759-775`): the
            // stack moves to the trash (REPLACING its contents) and the slot
            // turns to air. The research exception (`:762`) rides the Journey
            // research (unported); the cues are `TrashItem` (`:764`, no port
            // cue — the recorded gap) then `Grab` (`:765`).
            self.equip.trash = self.inventory[slot].clone();
            self.inventory[slot] = EMPTY_SLOT;
            self.audio.play("TrashItem"); // no port cue — the recorded gap
            self.audio.play("Grab");
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
        // `:629-728`): the favorite and shift arms evaluate for the hovered
        // slot with or without a click.
        self.evaluate_cursor_override();
        let left = self.ui_left_press;
        let right_held = self.place_down; // `Main.mouseRight` (`ItemSlot.cs:1654`)
        let right_edge = self.ui_right_press;
        if !left && !right_held {
            return false;
        }
        // The cursor-9 shift-click — `OverrideLeftClick` (`ItemSlot.cs:813-850`)
        // → `ChestUI.TryPlacingInChest(justCheck: false)` (`:846`).
        if left && self.cursor_override == CURSOR_TO_CHEST {
            // The ctx-1/ctx-2 rows ride the same cursor (`ItemSlot.cs:680-686`
            // covers 0/1/2), so the source resolves before the grid probe.
            if let Some(src) = self.chest_transfer_source() {
                chest::try_placing_from(self, src, false);
                self.ui_left_press = false;
                return true;
            }
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
        // stack-split gate transfers nothing this frame. The dispatch is
        // the world-chest panel's (the ctx-3 grid) — bank values keep the
        // STORE-1 panel and its own handler, so the gate is `>= 0` (the C#'s
        // single `ChestUI.Draw` covers all kinds — the recorded STORE-1
        // panel-fidelity deviation).
        if self.open_container() >= 0 && !self.crafting_open {
            // `ChestUI.DrawButton`'s click dispatch (`ChestUI.cs:281-308`) —
            // the C# `DrawButtons` pass runs before `DrawSlots`, so the
            // buttons take the press before the slot grid sees it. IDs 0-3
            // are the M3-STORE-2 transfers, 5/6 the rename pair (ID 5 →
            // `RenameChest` :827-838, ID 6 → `RenameChestCancel` :863-869);
            // Sort (4) stays label-less and takes no clicks. The port's
            // modal rename keyboard parks the world routes, so the
            // editing-state arms (`Save`/`Cancel` clicks) are the overlay
            // shape's revive point.
            if left {
                if let Some(id) = ui::chest::chest_button_hit(
                    &self.lookups,
                    self.mouse_pos,
                    &self.chest_button_scales,
                    &self.chest_button_hovered,
                    self.open_container(),
                    self.chest_editing(),
                ) {
                    match id {
                        0..=3 => self.chest_transfer_click(id),
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
            if let Some(abs) = ui::chest::slot_at(self.mouse_pos, self.chest_scroll_row) {
                self.chest_slot_click(abs, left, right_held);
                self.ui_left_press = false;
                self.ui_right_press = false;
                return true;
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

    /// `ItemSlot.OverrideHover` (`ItemSlot.cs:602-625`) — the FavoriteKey arm
    /// first: a non-air favoritable slot under Alt lights `cursorOverride = 3`
    /// and the shift arms never evaluate (the C# `return`s inside the key
    /// block; the canShareAt/chat arm `:613-615` and the loadout arm are out —
    /// no chat-draw share, no loadout system). Then
    /// `GetAlternateClickAction`'s shift arms (`ItemSlot.cs:644-726`), over
    /// the two ported surfaces: a hovered chest cell (context 3 → cursor 8
    /// when the inventory can take the stack, `:692-698`) and a hovered
    /// inventory grid slot (context 0 → cursor 9 when the open chest can take
    /// it, `:680-686` via `TryPlacingInChest(justCheck: true)`). The
    /// Control-in-use trash arm (`:637-643`) and the gamepad arms are out.
    fn evaluate_cursor_override(&mut self) {
        // `ItemSlot.cs:611-620`: `Main.keyState.IsKeyDown(Main.FavoriteKey)`
        // + `canFavoriteAt[context]` → `Main.cursorOverride = 3`. The port's
        // favoritable click surface is the open inventory's 10x5 grid
        // (context 0; the closed hotbar is context 13 — not in the table).
        if favorite_key_held(self) && self.inventory_open {
            if let Some(slot) = self.slot_at_mouse() {
                // `:607-609` — air returns before the key block; `:616-619`
                // — the canFavoriteAt gate (the grid is context 0; the
                // coin/ammo rows are not clickable here, the recorded
                // coin-row gap).
                if slot < crate::item::INV
                    && self.inventory[slot].count > 0
                    && chest::can_favorite_at(0)
                {
                    self.cursor_override = CURSOR_FAVORITE;
                    return;
                }
            }
        }
        let chest = self.open_container();
        if !shift_in_use(self) {
            return;
        }
        // The ctx-3 arms are the world-chest panel's (`chest >= 0`) — banks
        // keep the STORE-1 panel and no cursor-8/9 shift route (the recorded
        // panel-fidelity deviation; `TryPlacingInChest` refuses banks too).
        if chest >= 0 && !self.crafting_open {
            if let Some(abs) = ui::chest::slot_at(self.mouse_pos, self.chest_scroll_row) {
                if let Some(s) = chest::chest_slot(&self.world_chests, chest as usize, abs) {
                    if s.kind != 0 && s.count > 0 && chest::inventory_has_room_for(self, s.kind) {
                        self.cursor_override = CURSOR_FROM_CHEST;
                    }
                }
                return;
            }
        }
        if chest >= 0 {
            // `ItemSlot.cs:657-665`: the ctx-1/ctx-2 arms reach the same
            // `TryPlacingInChest` check, so a hovered coin/ammo cell lights the
            // transfer cursor exactly like a grid slot.
            if let Some(src) = self.chest_transfer_source() {
                if chest::try_placing_from(self, src, true) {
                    self.cursor_override = CURSOR_TO_CHEST;
                }
            }
            return;
        }
        // The tail arm — `GetAlternateClickAction`'s else-chain close
        // (`ItemSlot.cs:679-682`): Shift held and no crafting/reforge/chest
        // arm matched, so a hovered inventory slot with an item lights the
        // quick-trash cursor (`GetSellOrTrash` → `AlternateClickAction.Trash`
        // while no shop is open, `:198-209`; the port has no sell cursor, so
        // the shop case stands down — the recorded gap). The head guard
        // (`:631-634`): air or favorited slots answer null.
        if self.shop_npc.is_none() {
            if let Some(slot) = self.slot_at_mouse() {
                if slot < crate::item::INV
                    && self.inventory[slot].count > 0
                    && !self.is_favorite(slot)
                {
                    self.cursor_override = CURSOR_TRASH;
                }
            }
        }
    }

    /// The chest panel's transfer-button dispatch (`ChestUI.cs:281-308`'s
    /// cases 0-3, M3-STORE-2) against the open world chest's slots, with the
    /// C# world/bank deltas: LootAll rides the void vault when enabled, the
    /// DepositAll MoveCoins pre-pass runs for every container kind
    /// (`:543-562`) and QuickStack gets none (the pre-pass arms cover the
    /// bank kinds only, `:670-685`).
    fn chest_transfer_click(&mut self, id: usize) {
        let idx = self.open_container() as usize;
        // The open-container gate in `handle_chest_clicks` guarantees a live
        // store entry (`open_chest` opened it); the `get_mut` keeps the click
        // path panic-free regardless.
        let Some(wc) = self.world_chests.get_mut(idx) else {
            return;
        };
        let favs = self.favorite_slots.clone();
        let fav = move |i: usize| favs.contains(&i);
        let Some(p) = self.player.as_mut() else {
            return;
        };
        // `LootAllFromChest`'s `CanGoIntoVoidVault` (`GetItemSettings.cs:55-56`)
        // + `IsVoidVaultEnabled` (`Player.cs:24550-24553`): the vault is
        // `GetItem`'s per-item last resort (`Player.cs:39395-39398`). Banks
        // ride the default `LootAllFromBank` and never pass a vault.
        let vault = if p.void_vault_enabled {
            Some(&mut p.banks[3])
        } else {
            None
        };
        match id {
            // `ChestUI.LootAll` (`:393-473`).
            0 => crate::bank::loot_all(
                &mut wc.slots,
                &mut self.inventory,
                &mut self.equip.coins,
                vault,
            ),
            // `ChestUI.DepositAll` (`:540-659`) — the MoveCoins pre-pass runs
            // for every container kind (`:543-562`).
            1 => {
                crate::bank::move_coins(
                    &mut wc.slots,
                    &mut self.inventory,
                    &mut self.equip.coins,
                    &fav,
                );
                crate::bank::deposit_all(&mut wc.slots, &mut self.inventory, &fav);
            }
            // `ChestUI.QuickStack` (`:661-825`) — NO MoveCoins for a world
            // chest (`:670-685`'s arms are the bank kinds only).
            2 => crate::bank::quick_stack(
                &mut wc.slots,
                &mut self.inventory,
                &mut self.equip.coins,
                &fav,
            ),
            // `ChestUI.Restock` (`:871-995`) — the single PlaySound(7) on a
            // moved item (`:991-994`) stays out with the existing cue stance.
            _ => {
                let _ =
                    crate::bank::restock(&mut wc.slots, &mut self.inventory, &mut self.equip.ammo);
            }
        }
    }

    /// One ctx-3 cell's click pair (`ItemSlot.Handle`'s `LeftClick` +
    /// `RightClick` order, `ItemSlot.cs:588-597`): the left press runs the
    /// cursor-8 shift transfer or the action-0 movement, the held right
    /// button runs the stack-split.
    fn chest_slot_click(&mut self, abs: usize, left: bool, right_held: bool) {
        let chest_idx = self.open_container() as usize;
        if left && self.cursor_override == CURSOR_FROM_CHEST {
            // `OverrideLeftClick`'s cursor-8 arm (`ItemSlot.cs:800-812`):
            // `inv[slot] = player.GetItem(inv[slot], QuickTransferFromSlot)`;
            // consumes the click (the action-0 movement does not run).
            chest::quick_transfer_from_slot(self, abs);
            return;
        }
        if left {
            let mut held = self.mouse_item;
            // The chest slot IS an `ItemSlot` now (M3-CHEST-INT) — the
            // fabricated view is gone and the slot's prefix rides the whole
            // movement (take / place / CanStack merge / swap).
            let mut target =
                chest::chest_slot(&self.world_chests, chest_idx, abs).unwrap_or(EMPTY_SLOT);
            // `PickItemMovementAction` ctx 3 → action 0 (`ItemSlot.cs:1505-1507`)
            // — the shared movement.
            if left_click_movement(&mut held, &mut target) {
                self.audio.play("Grab"); // `:1010` — `SoundEngine.PlaySound(7)`
            }
            // `:976-979` — the emptied slot turns to air (the air-normalizing
            // write; the moved-in item keeps its prefix).
            chest::set_chest_slot(&mut self.world_chests, chest_idx, abs, target);
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
        let chest_idx = self.open_container() as usize;
        let transfers = self.super_fast_stack + 1; // `:1704`
        for _ in 0..transfers {
            let Some(s) = chest::chest_slot(&self.world_chests, chest_idx, abs) else {
                break;
            };
            if s.kind == 0 || s.count == 0 {
                break; // `:1700`'s `inv[slot].IsAir`
            }
            let max = registry::max_stack(s.kind);
            // `:1707` — `CanStack(mouseItem, inv[slot]) && mouseItem.stack <
            // maxStack`, or an air cursor (`Item.CanStack` = same type AND
            // same prefix, `Item.cs:49865-49871`).
            let can_take = (crate::bank::can_stack(&self.mouse_item, &s)
                && self.mouse_item.count < max)
                || self.mouse_item.count == 0;
            if !can_take {
                break;
            }
            // `PickupItemIntoMouse` (`:1717-1747`): an air cursor clones the
            // slot at stack 0 — name and prefix included (`:1721`'s
            // `inv[slot].Clone()`) — then +1/−1; the emptied slot turns to
            // air (`:1743-1746`). (The `:1728-1735` favorite carry is
            // unreachable — chest slots cannot be favorited.)
            if self.mouse_item.count == 0 {
                self.mouse_item = s;
                self.mouse_item.count = 0;
            }
            self.mouse_item.count += 1;
            let left_in_slot = s.count - 1;
            chest::set_chest_slot(
                &mut self.world_chests,
                chest_idx,
                abs,
                ItemSlot {
                    count: left_in_slot,
                    ..s
                },
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
                match net::Server::start(port, password) {
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
            // `Lang.menu[131]`'s row (`Main.cs:48928-48935`): `menuMode = 888`
            // + `Main.MenuUI.SetState(Main.AchievementsMenu)`. The port keeps
            // the legacy menu live behind the overlay (the vanilla state
            // draws over the menu's world backdrop) and opens
            // `UIAchievementsMenu`'s `OnActivate` state: every category
            // toggle on, the scroll at the top.
            Action::Achievements => {
                self.achievements_open = true;
                self.achievements_scroll = 0.0;
                self.achievements_categories = [true; 4];
                // The menu row's `SoundEngine.PlaySound(10)` already fired in
                // `Menu::click`.
            }
            Action::Join { addr, password } => match net::Client::connect(&addr) {
                Ok(client) => {
                    self.net = NetRole::Client(client);
                    self.join_target = Some((addr, password));
                    self.net_state = net::client_state::NOT_CONNECTED;
                    self.net_last_recv = std::time::Instant::now();
                    self.net_dropped = false;
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
            // The settings screen's bind edits (`Main.cs:4602`'s `KeyStatus`
            // re-seed): the menu owns the rebind UI, `Settings` owns the table.
            self.settings.keybinds = m.keybinds;
            self.settings.map_binds = m.map_binds;
        }
        self.apply_settings();
    }

    /// Takes the legacy menu into the App without the launch semantics — the
    /// settings sync of [`App::close_menu`] for every hop where a front screen
    /// takes over or the world launches.
    pub(crate) fn park_menu(&mut self) {
        if let Some(m) = self.menu.take() {
            self.settings = m.settings;
            self.settings.keybinds = m.keybinds;
            self.settings.map_binds = m.map_binds;
        }
        self.apply_settings();
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn bug20_shift_click_moves_the_stack_to_the_trash() {
        // Bug 20: `GetAlternateClickAction`'s tail + `OverrideLeftClick`'s
        // cursor-6 arm (`ItemSlot.cs:679-682`/`:759-775`) — Shift-clicking an
        // inventory slot REPLACES the trash with the stack and leaves the slot
        // empty; the cues fire.
        let mut app = inv_app();
        app.inventory[0] = crate::item::ItemSlot {
            kind: 3,
            name: "Dirt",
            count: 7,
            prefix: 0,
        };
        app.equip.trash = crate::item::ItemSlot {
            kind: 9,
            name: "Wood",
            count: 2,
            prefix: 0,
        };
        // Shift held over the grid's first cell.
        app.keys.insert(KeyCode::ShiftLeft);
        app.mouse_pos = Vec2::new(20.0 + 28.0, 20.0 + 28.0);
        let before = app.audio.played_count();
        app.ui_left_press = true;
        assert!(!app.handle_chest_clicks(), "no chest surface takes it");
        // The click pass evaluates the cursor then runs the arms.
        app.evaluate_cursor_override();
        assert_eq!(app.cursor_override, CURSOR_TRASH, "the trash cursor lights");
        app.handle_inventory_clicks();
        assert_eq!(app.equip.trash.kind, 3, "the trash takes the stack");
        assert_eq!(app.equip.trash.count, 7);
        assert_eq!(app.inventory[0].count, 0, "the slot is emptied");
        assert!(app.audio.played_count() > before, "the cues fired");
    }

    #[test]
    fn bug20_death_clears_the_trash() {
        // `Player.DropItems`'s first line (`Player.cs:54304`): the trash is
        // emptied on death.
        let mut app = inv_app();
        app.equip.trash = crate::item::ItemSlot {
            kind: 3,
            name: "Dirt",
            count: 5,
            prefix: 0,
        };
        crate::death::on_player_death(&mut app, 100.0, 100.0, "text");
        assert_eq!(app.equip.trash.count, 0, "the trash cleared");
    }

    #[test]
    fn bug20_the_trash_shifts_with_the_chest_open() {
        // `Main.DrawTrashItemSlot`'s `ChestOrShopUIVisible` arm
        // (`Main.cs:42351-42356`): the slot moves to (453, 426) at 0.755.
        let (rect, scale) = crate::ui::inventory::trash_rect(true);
        assert_eq!(rect[0], 453.0, "the chest-open x");
        assert_eq!(rect[1], 426.0, "the chest-open y");
        assert_eq!(scale, 0.755);
        // The hit test follows the drawn rect.
        assert!(crate::ui::inventory::trash_slot_at(
            Vec2::new(453.0, 426.0),
            true
        ));
        assert!(!crate::ui::inventory::trash_slot_at(
            Vec2::new(448.0, 258.0),
            true
        ));
    }

    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::chest::{DEFAULT_MAX_ITEMS, WorldChest};
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

    /// A prefixed fixture slot (the M3-PREFIX rolled-prefix shape).
    fn prefixed(kind: u16, count: u32, prefix: u16) -> ItemSlot {
        ItemSlot {
            prefix,
            ..slot(kind, count)
        }
    }

    /// A headless app with the inventory open and an open chest (index 0,
    /// tile origin (20, 10)) whose store holds `slots`. The open state is
    /// the player's (`Player.chest`, the M3-CHEST-INT unify).
    fn chest_app(slots: Vec<ItemSlot>) -> App {
        let mut app = flow_app(None, vec![], vec![]);
        app.inventory_open = true;
        app.world_chests.push(WorldChest {
            tile: (20, 10),
            slots,
            name: String::new(),
            max_items: crate::chest::DEFAULT_MAX_ITEMS as usize,
        });
        app.player = Some(crate::player::Player::spawn(&app.world));
        let p = app.player.as_mut().unwrap();
        p.chest = 0;
        p.chest_x = 20;
        p.chest_y = 10;
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
        let mut app = chest_app(vec![slot(9, 6)]);
        // Take: an empty cursor takes the whole stack.
        app.mouse_pos = CHEST_SLOT_0;
        app.ui_left_press = true;
        assert!(app.handle_chest_clicks());
        assert_eq!(app.mouse_item.kind, 9);
        assert_eq!(app.mouse_item.count, 6);
        assert_eq!(chest::chest_slot(&app.world_chests, 0, 0), Some(EMPTY_SLOT));
        // Place: the cursor stack lands on an empty chest cell.
        app.mouse_item = slot(9, 3);
        app.mouse_pos = CHEST_SLOT_1;
        app.ui_left_press = true;
        assert!(app.handle_chest_clicks());
        assert_eq!(chest::chest_slot(&app.world_chests, 0, 1), Some(slot(9, 3)));
        assert_eq!(app.mouse_item.count, 0);
        // Swap: a different kind exchanges with the cursor.
        app.mouse_item = slot(5, 2);
        app.ui_left_press = true;
        assert!(app.handle_chest_clicks());
        assert_eq!(chest::chest_slot(&app.world_chests, 0, 1), Some(slot(5, 2)));
        assert_eq!(app.mouse_item.kind, 9);
        assert_eq!(app.mouse_item.count, 3);
        // Stack-merge: the same kind pours into the partial slot.
        app.mouse_item = slot(5, 4);
        app.ui_left_press = true;
        assert!(app.handle_chest_clicks());
        assert_eq!(chest::chest_slot(&app.world_chests, 0, 1), Some(slot(5, 6)));
        assert_eq!(app.mouse_item.count, 0);
    }

    /// DW-4.1 — the merge caps at the kind's `registry::max_stack` (Wood's
    /// `Item.maxStack` is 9999, itemdata) and a full slot swaps instead of
    /// merging (`ItemSlot.cs:980-998`).
    #[test]
    fn dw_4_1_stack_merge_respects_max_stack_boundary() {
        let mut app = chest_app(vec![slot(9, 9990)]);
        app.mouse_pos = CHEST_SLOT_0;
        app.mouse_item = slot(9, 20);
        app.ui_left_press = true;
        assert!(app.handle_chest_clicks());
        assert_eq!(
            chest::chest_slot(&app.world_chests, 0, 0),
            Some(slot(9, 9999))
        );
        assert_eq!(app.mouse_item.count, 11);
        // The now-full slot exchanges with the cursor stack.
        app.ui_left_press = true;
        assert!(app.handle_chest_clicks());
        assert_eq!(
            chest::chest_slot(&app.world_chests, 0, 0),
            Some(slot(9, 11))
        );
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
        let mut app = chest_app(vec![slot(3, 9999); full_count]);
        app.inventory[3] = slot(9, 10);
        assert!(!chest::try_placing_in_chest(&mut app, 3, true));
        assert!(!chest::try_placing_in_chest(&mut app, 3, false));
        assert_eq!(app.inventory[3].count, 10);
        assert_eq!(app.world_chests[0].slots, vec![slot(3, 9999); full_count]);
        // Same kind, every stack full: the merge skip rule (`:1225`).
        let mut app = chest_app(vec![slot(9, 9999); full_count]);
        app.inventory[3] = slot(9, 10);
        assert!(!chest::try_placing_in_chest(&mut app, 3, false));
        assert_eq!(app.inventory[3].count, 10);
    }

    /// DW-4.2 — `just_check` answers placeability without mutating the
    /// inventory, the store or the audio state (`ChestUI.cs:1234-1237`,
    /// `:1271-1274`).
    #[test]
    fn dw_4_2_just_check_mutates_nothing() {
        let mut app = chest_app(vec![slot(9, 5)]);
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
        let mut app = chest_app(vec![slot(9, 5)]);
        app.inventory[3] = slot(9, 12);
        assert!(!chest::try_placing_in_chest(&mut app, 3, false));
        assert_eq!(app.world_chests[0].slots[0], slot(9, 17));
        assert_eq!(app.inventory[3].count, 0);
        // A remainder overflows into the first empty slot only.
        let mut app = chest_app(vec![slot(9, 9990)]);
        app.inventory[3] = slot(9, 20);
        assert!(!chest::try_placing_in_chest(&mut app, 3, false));
        assert_eq!(app.world_chests[0].slots[0], slot(9, 9999));
        assert_eq!(app.world_chests[0].slots[1], slot(9, 11));
        assert_eq!(app.inventory[3].count, 0);
        // Two partials soak their room in slot order before the empty pass.
        let mut app = chest_app(vec![slot(9, 9995), EMPTY_SLOT, slot(9, 9995)]);
        app.inventory[3] = slot(9, 12);
        assert!(!chest::try_placing_in_chest(&mut app, 3, false));
        assert_eq!(app.world_chests[0].slots[0], slot(9, 9999));
        assert_eq!(app.world_chests[0].slots[1], slot(9, 4));
        assert_eq!(app.world_chests[0].slots[2], slot(9, 9999));
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
        let mut app = chest_app(vec![slot(9, 1)]);
        app.mouse_pos = CHEST_SLOT_0;
        app.place_down = true;
        app.ui_right_press = true;
        assert!(app.handle_chest_clicks());
        assert_eq!(app.mouse_item.kind, 9);
        assert_eq!(app.mouse_item.count, 1);
        assert_eq!(chest::chest_slot(&app.world_chests, 0, 0), Some(EMPTY_SLOT));
        assert_eq!(app.stack_split, 30);
    }

    /// DW-4.3 — the held repeat is paced: no transfer while `stackSplit > 1`
    /// (`:1700`), then one per frame and `RefreshStackSplitCooldown` arms
    /// `stackDelay` (`:1774-1784`).
    #[test]
    fn dw_4_3_right_click_repeat_paced_by_stack_split() {
        let mut app = chest_app(vec![slot(9, 5)]);
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
        let mut app = chest_app(vec![slot(9, 5)]);
        app.mouse_pos = CHEST_SLOT_0;
        app.mouse_item = slot(9, 9999);
        app.place_down = true;
        app.handle_chest_clicks();
        assert_eq!(app.mouse_item.count, 9999);
        assert_eq!(chest::chest_slot(&app.world_chests, 0, 0), Some(slot(9, 5)));
        // An alien cursor never splits.
        let mut app = chest_app(vec![slot(9, 5)]);
        app.mouse_pos = CHEST_SLOT_0;
        app.mouse_item = slot(3, 1);
        app.place_down = true;
        app.handle_chest_clicks();
        assert_eq!(app.mouse_item.kind, 3);
        assert_eq!(chest::chest_slot(&app.world_chests, 0, 0), Some(slot(9, 5)));
    }

    /// DW-2.1 — the ctx-3 left-click take carries the chest slot's prefix
    /// into the cursor and back (`PickItemMovementAction` ctx 3 → action 0
    /// through the shared movement; the C# moves whole `Item`s, so the
    /// prefix rides every swap).
    #[test]
    fn test_DW_2_1_prefix_survives_the_left_click_take() {
        let mut app = chest_app(vec![prefixed(48, 7, 3)]);
        app.mouse_pos = CHEST_SLOT_0;
        app.ui_left_press = true;
        assert!(app.handle_chest_clicks());
        assert_eq!(app.mouse_item.prefix, 3, "the take carried the prefix");
        assert_eq!(app.mouse_item.kind, 48);
        assert_eq!(app.mouse_item.count, 7);
        assert_eq!(chest::chest_slot(&app.world_chests, 0, 0), Some(EMPTY_SLOT));
        // Placing it back keeps the prefix in the store.
        app.mouse_pos = CHEST_SLOT_1;
        app.ui_left_press = true;
        assert!(app.handle_chest_clicks());
        assert_eq!(
            chest::chest_slot(&app.world_chests, 0, 1),
            Some(prefixed(48, 7, 3))
        );
        assert_eq!(app.mouse_item.count, 0);
    }

    /// DW-2.1 (dirty) — the right-click stack-split's cursor clone carries
    /// the chest slot's prefix (`PickupItemIntoMouse`'s `inv[slot].Clone()`,
    /// `ItemSlot.cs:1721`), and the take gate is CanStack — a different
    /// prefix never splits into the cursor's stack (`ItemSlot.cs:1707`).
    #[test]
    fn test_DW_2_1_stack_split_carries_the_prefix_and_refuses_cross_prefix() {
        let mut app = chest_app(vec![prefixed(9, 3, 2)]);
        app.mouse_pos = CHEST_SLOT_0;
        app.place_down = true;
        app.ui_right_press = true;
        assert!(app.handle_chest_clicks());
        assert_eq!(app.mouse_item.prefix, 2, "the clone carried the prefix");
        assert_eq!(app.mouse_item.count, 1);
        assert_eq!(
            chest::chest_slot(&app.world_chests, 0, 0),
            Some(prefixed(9, 2, 2))
        );
        // An unprefixed cursor never splits a prefixed chest stack.
        let mut app = chest_app(vec![prefixed(9, 3, 2)]);
        app.mouse_item = slot(9, 5);
        app.mouse_pos = CHEST_SLOT_0;
        app.place_down = true;
        app.ui_right_press = true;
        app.handle_chest_clicks();
        assert_eq!(app.mouse_item.count, 5, "the cross-prefix split refused");
        assert_eq!(
            chest::chest_slot(&app.world_chests, 0, 0),
            Some(prefixed(9, 3, 2))
        );
    }

    /// DW-4.5 — the shift hover evaluation (`GetAlternateClickAction`,
    /// `ItemSlot.cs:644-726`): cursor 8 on a chest cell with inventory room,
    /// 9 on an inventory slot with a placeable stack, −1 without Shift and
    /// −1 when nothing has room. (The Cursors_8/9 texture draw is the
    /// recorded follow-up — the state is what the click arms consume.)
    #[test]
    fn dw_4_5_cursor_override_eval_per_hover_and_shift() {
        let mut app = chest_app(vec![slot(9, 5)]);
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
        let mut app = chest_app(vec![slot(9, 5)]);
        app.inventory[5] = slot(9, 10);
        app.mouse_pos = INV_SLOT_5;
        app.keys.insert(KeyCode::ShiftLeft);
        app.ui_left_press = true;
        assert!(app.handle_chest_clicks());
        assert_eq!(app.world_chests[0].slots[0], slot(9, 15));
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
        let mut app = chest_app(vec![slot(9, 500)]);
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
            Some(EMPTY_SLOT),
            "the whole stack moved"
        );
    }

    /// The batch-review carry — the open-inventory chest-tile click: the same
    /// chest's right-click toggles closed (`Player.cs:33230-33252`) and the
    /// press is consumed.
    #[test]
    fn re_click_on_the_open_chest_tile_closes_it() {
        let mut app = chest_app(vec![slot(9, 1)]);
        app.world = World::new("t".into(), 120, 120);
        place_tile_chest(&mut app, 20, 10);
        app.mouse_pos = tile_click(20, 10);
        app.ui_right_press = true;
        app.place_down = true;
        assert!(app.handle_chest_clicks());
        assert_eq!(
            app.player.as_ref().unwrap().chest,
            -1,
            "the re-click closes (:33236)"
        );
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
            name: String::new(),
            max_items: crate::chest::DEFAULT_MAX_ITEMS as usize,
        });
        app.player.as_mut().unwrap().chest = 0;
        app.mouse_pos = tile_click(40, 10);
        app.ui_right_press = true;
        app.place_down = true;
        assert!(app.handle_chest_clicks());
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.chest, 1, "the store index of the (40, 10) chest");
        assert_eq!((p.chest_x, p.chest_y), (40, 10));
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
        let p = app.player.as_mut().unwrap();
        p.chest_x = 2;
        p.chest_y = 2;
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
        assert!(
            app.world_chests.iter().all(|c| c.name.is_empty()),
            "the cancel never wrote"
        );
    }

    /// The editing-state Save arm (`ChestUI.RenameChest`'s else, `:834-837`):
    /// clicking ID 5 while the editor is live submits the live text.
    #[test]
    fn rename_button_click_submits_while_editing() {
        let mut app = chest_app(vec![]);
        place_tile_chest(&mut app, 2, 2);
        let p = app.player.as_mut().unwrap();
        p.chest_x = 2;
        p.chest_y = 2;
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

    // --- Phase 3 (M3-CHEST-INT): the STORE-2 transfer buttons ------------

    /// A headless app with the inventory open and a piggy bank open
    /// (`Player.chest = -2`, bank index 0) — the M3-STORE-1 dispatch
    /// fixture.
    fn bank_app() -> App {
        let mut app = flow_app(None, vec![], vec![]);
        app.inventory_open = true;
        app.player = Some(crate::player::Player::spawn(&app.world));
        app.player.as_mut().unwrap().chest = -2;
        app
    }

    /// `chest_app` with the open chest's slots materialized to the browsed
    /// 40-slot window (`open_chest`'s resize, Phase-2 decision #8) — the
    /// transfers' free-slot walks need the real window shape.
    fn stocked_chest_app(slots: Vec<ItemSlot>) -> App {
        let mut app = chest_app(slots);
        app.world_chests[0]
            .slots
            .resize(DEFAULT_MAX_ITEMS as usize, EMPTY_SLOT);
        app
    }

    /// The button row at the drawn idle state — the dispatch reads the same
    /// scales the draw pass animates (`ChestUI.ButtonScale`, the static
    /// ctor's 0.0 decays to `buttonScaleMinimum` after one frame).
    fn ready_buttons(app: &mut App) {
        app.chest_button_scales = [ui::chest::BUTTON_SCALE_MIN; ui::chest::BUTTON_COUNT];
    }

    /// DW-3.1 — LootAll on a world chest drains it into the player: the
    /// coins ride the coin rows first (`Player.GetItem`'s isACoin arm,
    /// `Player.cs:39340-39364`), the overflow stays in the chest with the
    /// vault disabled (`LootAllFromBank`'s default settings — the port's
    /// `void_vault_enabled` gates `CanGoIntoVoidVault`).
    #[test]
    fn test_DW_3_1_loot_all_drains_a_world_chest_coins_first_and_keeps_overflow() {
        let mut app = stocked_chest_app(vec![slot(9, 5), slot(71, 50), slot(3, 500)]);
        for s in app.inventory.iter_mut() {
            *s = slot(3, 9999); // stone everywhere: the wood/stone overflow
        }
        ready_buttons(&mut app);
        app.mouse_pos = Vec2::new(510.0, 300.0); // the Loot All row (id 0)
        app.ui_left_press = true;
        assert!(app.handle_chest_clicks(), "the button took the click");
        assert!(!app.ui_left_press, "the press is consumed");
        assert_eq!(
            app.mouse_item.count, 0,
            "the slot handler never saw the click"
        );
        assert_eq!(
            app.equip.coins[0].count, 50,
            "the coins ride the coin rows first"
        );
        assert_eq!(
            app.world_chests[0].slots[1].count, 0,
            "the coin slot drained"
        );
        assert_eq!(
            app.world_chests[0].slots[0].count, 5,
            "the wood overflow stayed in the chest"
        );
        assert_eq!(
            app.world_chests[0].slots[2].count, 500,
            "the stone overflow stayed in the chest"
        );
    }

    /// DW-3.1 — with the void vault enabled (`IsVoidVaultEnabled`,
    /// `Player.cs:24550-24553`) the overflow fills the vault first
    /// (`GetItem`'s per-item last resort, `Player.cs:39395-39398`).
    #[test]
    fn test_DW_3_1_loot_all_overflow_fills_the_void_vault_when_enabled() {
        let mut app = stocked_chest_app(vec![slot(9, 5), slot(71, 50), slot(3, 500)]);
        for s in app.inventory.iter_mut() {
            *s = slot(3, 9999);
        }
        {
            let p = app.player.as_mut().unwrap();
            p.void_vault_enabled = true;
        }
        ready_buttons(&mut app);
        app.mouse_pos = Vec2::new(510.0, 300.0);
        app.ui_left_press = true;
        assert!(app.handle_chest_clicks());
        assert_eq!(
            app.equip.coins[0].count, 50,
            "the coins still ride the rows"
        );
        assert!(
            app.world_chests[0].slots.iter().all(|s| s.count == 0),
            "the vault took the whole overflow"
        );
        let vault = &app.player.as_ref().unwrap().banks[3];
        assert_eq!(vault[0].kind, 9);
        assert_eq!(vault[0].count, 5, "the wood overflowed into the vault");
        assert_eq!(vault[1].kind, 3);
        assert_eq!(vault[1].count, 500, "the stone overflowed into the vault");
    }

    /// DW-3.2 — DepositAll on a world chest = MoveCoins + the rows 49..=10
    /// descending scan, the favorited rows stay, the leftovers park in the
    /// first empty slot (`ChestUI.cs:540-659`). A coin-less chest is the
    /// `:1048-1051` MoveCoins no-op — the coin rows stay put and the grid
    /// stacks park as plain stacks; with a coin in the chest the pre-pass
    /// consolidates into it.
    #[test]
    fn test_DW_3_2_deposit_all_moves_coins_then_pours_rows_descending() {
        let mut app = stocked_chest_app(vec![slot(9, 500)]);
        app.inventory[0] = slot(9, 50); // the hotbar: untouched
        app.inventory[20] = slot(9, 200); // merges 500 -> 700
        app.inventory[22] = slot(71, 100); // a grid coin stack
        app.inventory[25] = slot(3, 100); // favorited: stays
        app.inventory[30] = slot(41, 1); // a non-merge kind: the first empty slot
        app.favorite_slots.push(25);
        app.equip.coins[0] = slot(71, 60); // the coin rows
        ready_buttons(&mut app);
        app.mouse_pos = Vec2::new(510.0, 326.0); // the Deposit All row (id 1)
        app.ui_left_press = true;
        assert!(app.handle_chest_clicks());
        let chest = &app.world_chests[0].slots;
        assert_eq!(chest[0].count, 700, "the wood merged");
        assert!(
            !chest.iter().any(|s| s.kind == 72),
            "no consolidation into a coin-less chest (`:1048-1051`)"
        );
        assert!(
            chest.iter().any(|s| s.kind == 71 && s.count == 100),
            "the grid coin stack parked as a plain stack"
        );
        assert!(
            chest.iter().any(|s| s.kind == 41 && s.count == 1),
            "the leftover parked in the first empty slot"
        );
        assert_eq!(app.inventory[25].count, 100, "the favorited row stayed");
        assert_eq!(app.inventory[0].count, 50, "the hotbar untouched");
        assert_eq!(app.inventory[20].count, 0);
        assert_eq!(app.inventory[22].count, 0, "the grid stack deposited");
        assert_eq!(app.inventory[30].count, 0);
        assert_eq!(app.equip.coins[0].count, 60, "the coin rows untouched");
        // The pre-pass with a coin in the chest: 40 + 35 + 5 = 80 copper
        // consolidates into the chest's coin slot (no carry at 80).
        let mut app = stocked_chest_app(vec![slot(71, 40)]);
        app.inventory[20] = slot(71, 35);
        app.inventory[30] = slot(71, 5);
        ready_buttons(&mut app);
        app.mouse_pos = Vec2::new(510.0, 326.0);
        app.ui_left_press = true;
        assert!(app.handle_chest_clicks());
        assert_eq!(
            app.world_chests[0].slots[0].kind, 71,
            "the pre-pass consolidated into the chest's coin slot"
        );
        assert_eq!(app.world_chests[0].slots[0].count, 80);
        assert_eq!(app.inventory[20].count, 0);
        assert_eq!(app.inventory[30].count, 0);
        // The descending order: one free slot — row 40 scans before row 20
        // (`ChestUI.cs:563`'s `for num = 49; num >= 10; num--`).
        let mut slots = vec![slot(3, 9999); DEFAULT_MAX_ITEMS as usize - 1];
        slots.push(EMPTY_SLOT);
        let mut app = stocked_chest_app(slots);
        app.inventory[20] = slot(41, 1); // the torch: scanned last
        app.inventory[40] = slot(43, 5); // the arrows: scanned first
        ready_buttons(&mut app);
        app.mouse_pos = Vec2::new(510.0, 326.0);
        app.ui_left_press = true;
        assert!(app.handle_chest_clicks());
        assert_eq!(
            app.world_chests[0].slots[DEFAULT_MAX_ITEMS as usize - 1].kind,
            43,
            "row 40 won the free slot"
        );
        assert_eq!(app.inventory[20].count, 1, "row 20 found no free slot");
    }

    /// DW-3.3 — QuickStack on a world chest performs NO MoveCoins and only
    /// merges types the chest already holds (`ChestUI.cs:661-825` — the
    /// MoveCoins arms :670-685 cover the bank kinds only).
    #[test]
    fn test_DW_3_3_quick_stack_world_merges_only_held_types_without_move_coins() {
        let mut app = stocked_chest_app(vec![slot(9, 100)]);
        app.inventory[20] = slot(9, 150); // wood merges (the chest holds wood)
        app.inventory[21] = slot(3, 40); // stone: the chest holds none — stays
        app.inventory[22] = slot(71, 200); // grid coins: no MoveCoins — stay
        app.equip.coins[0] = slot(71, 5);
        ready_buttons(&mut app);
        app.mouse_pos = Vec2::new(510.0, 352.0); // the Quick Stack row (id 2)
        app.ui_left_press = true;
        assert!(app.handle_chest_clicks());
        let chest = &app.world_chests[0].slots;
        assert_eq!(chest[0].count, 250, "the wood merged");
        assert_eq!(app.inventory[21].count, 40, "only held types merge");
        assert_eq!(app.inventory[22].count, 200, "no MoveCoins ran");
        assert_eq!(app.equip.coins[0].count, 5, "the coin rows untouched");
        assert!(
            !chest.iter().any(|s| s.kind == 71 || s.kind == 72),
            "no coin consolidation into the chest"
        );
    }

    /// DW-3.3 — the bank regression: the bank QuickStack keeps its MoveCoins
    /// pre-pass (`ChestUI.cs:670-685`) — with a coin in the bank it pulls the
    /// player's coins and consolidates into the bank's coin slot; with a
    /// coin-less bank it is the `:1048-1051` no-op and nothing moves.
    #[test]
    fn test_DW_3_3_quick_stack_bank_keeps_the_move_coins_pre_pass() {
        let mut app = bank_app();
        app.player.as_mut().unwrap().banks[0][0] = slot(9, 100);
        app.player.as_mut().unwrap().banks[0][1] = slot(71, 10); // the seed coin
        app.inventory[20] = slot(9, 150); // merges into the wood target
        app.inventory[22] = slot(71, 85); // MoveCoins pulls the grid coins
        app.mouse_pos = Vec2::new(510.0, 390.0); // the bank's Quick Stack row
        app.ui_left_press = true;
        app.handle_inventory_clicks();
        let bank = &app.player.as_ref().unwrap().banks[0];
        assert_eq!(bank[0].count, 250, "the wood merged");
        assert_eq!(bank[1].kind, 71);
        assert_eq!(
            bank[1].count, 95,
            "the pre-pass consolidated 10 + 85 copper (no carry at 95)"
        );
        assert_eq!(bank[2].count, 0);
        assert_eq!(app.inventory[20].count, 0);
        assert_eq!(app.inventory[22].count, 0, "the grid coins drained");
        // The coin-less bank: the pre-pass is the `:1048-1051` no-op — the
        // player's coins stay put.
        let mut app = bank_app();
        app.player.as_mut().unwrap().banks[0][0] = slot(9, 100);
        app.inventory[20] = slot(9, 150);
        app.inventory[22] = slot(71, 200);
        app.equip.coins[0] = slot(71, 5);
        app.mouse_pos = Vec2::new(510.0, 390.0);
        app.ui_left_press = true;
        app.handle_inventory_clicks();
        let bank = &app.player.as_ref().unwrap().banks[0];
        assert_eq!(bank[0].count, 250, "the wood merged");
        assert_eq!(bank[1].count, 0, "the bank took no coins");
        assert_eq!(
            app.inventory[22].count, 200,
            "no consolidation into a coin-less bank"
        );
        assert_eq!(app.equip.coins[0].count, 5, "the coin rows untouched");
    }

    /// DW-3.4 — Restock from a world chest fills unprefixed demanded stacks
    /// only (`ChestUI.cs:871-995`); prefixed chest stacks never move
    /// (:921's `prefix != 0` gate).
    #[test]
    fn test_DW_3_4_restock_fills_unprefixed_demanded_stacks_only() {
        let mut app = stocked_chest_app(vec![prefixed(9, 10, 2), slot(9, 30), slot(40, 20)]);
        app.inventory[20] = slot(9, 50); // partial wood: the demand
        app.inventory[21] = slot(40, 10); // partial arrows: the demand
        ready_buttons(&mut app);
        app.mouse_pos = Vec2::new(510.0, 378.0); // the Restock row (id 3)
        app.ui_left_press = true;
        assert!(app.handle_chest_clicks());
        assert_eq!(
            app.inventory[20].count, 80,
            "the unprefixed supply filled it"
        );
        assert_eq!(app.inventory[21].count, 30);
        assert_eq!(
            app.world_chests[0].slots[0],
            prefixed(9, 10, 2),
            "the prefixed stack never moves"
        );
        assert_eq!(app.world_chests[0].slots[1].count, 0);
        assert_eq!(app.world_chests[0].slots[2].count, 0);
    }

    /// DW-3.5 — the button hit region swallows the click before
    /// `chest_slot_click` sees it (`ChestUI.DrawButtons` runs before
    /// `DrawSlots`): the transfer runs, the press is consumed and the cursor
    /// never picked the slot up.
    #[test]
    fn test_DW_3_5_button_hit_consumes_the_press_before_the_slot_grid() {
        let mut app = stocked_chest_app(vec![slot(40, 20)]);
        app.inventory[21] = slot(40, 10); // partial arrows: the demand
        ready_buttons(&mut app);
        app.mouse_pos = Vec2::new(510.0, 378.0); // the Restock row (id 3)
        app.ui_left_press = true;
        assert!(app.handle_chest_clicks(), "the button took the click");
        assert!(!app.ui_left_press, "the press is consumed");
        assert_eq!(app.mouse_item.count, 0, "the slot handler never ran");
        assert_eq!(app.inventory[21].count, 30, "the transfer ran instead");
        assert_eq!(app.world_chests[0].slots[0].count, 0);
    }

    /// DW-3.5 — a bank value never dispatches the chest panel's row (the
    /// two-panel ownership): the click falls through unconsumed and the bank
    /// keeps its contents.
    #[test]
    fn test_DW_3_5_bank_values_dispatch_no_chest_panel_row() {
        let mut app = bank_app();
        app.player.as_mut().unwrap().banks[0][0] = slot(9, 100);
        ready_buttons(&mut app);
        app.mouse_pos = Vec2::new(510.0, 300.0); // the chest panel's Loot All row
        app.ui_left_press = true;
        assert!(
            !app.handle_chest_clicks(),
            "the chest panel's dispatch is bank-hidden"
        );
        assert!(
            app.ui_left_press,
            "the press fell through to the bank panel"
        );
        assert_eq!(
            app.player.as_ref().unwrap().banks[0][0].count,
            100,
            "the bank untouched by the chest row"
        );
    }

    /// DW-3.6 — the bank DepositAll call site gains the MoveCoins pre-pass
    /// per `ChestUI.cs:543-562` (it runs for every container kind): with a
    /// coin in the bank the scattered coins consolidate into the bank on
    /// deposit; with a coin-less bank the pre-pass is the `:1048-1051`
    /// no-op — the grid stacks park as plain stacks (:563-655) and the coin
    /// rows stay put.
    #[test]
    fn test_DW_3_6_bank_deposit_all_consolidates_coins_into_the_bank() {
        let mut app = bank_app();
        app.player.as_mut().unwrap().banks[0][0] = slot(71, 40); // the seed coin
        app.inventory[20] = slot(71, 35);
        app.inventory[30] = slot(71, 5);
        app.mouse_pos = Vec2::new(510.0, 360.0); // the bank's Deposit All row
        app.ui_left_press = true;
        app.handle_inventory_clicks();
        let bank = &app.player.as_ref().unwrap().banks[0];
        assert_eq!(bank[0].kind, 71);
        assert_eq!(
            bank[0].count, 80,
            "the pre-pass consolidated 40 + 35 + 5 (no carry at 80)"
        );
        assert_eq!(app.inventory[20].count, 0);
        assert_eq!(app.inventory[30].count, 0);
        // The coin-less bank: no consolidation — the grid stacks park as
        // plain stacks (row 30 scans first, row 20 merges into it) and the
        // coin rows stay untouched.
        let mut app = bank_app();
        app.inventory[20] = slot(71, 35);
        app.inventory[30] = slot(71, 5);
        app.equip.coins[0] = slot(71, 60);
        app.mouse_pos = Vec2::new(510.0, 360.0);
        app.ui_left_press = true;
        app.handle_inventory_clicks();
        let bank = &app.player.as_ref().unwrap().banks[0];
        assert_eq!(
            bank[0].kind,
            71,
            "the grid stacks parked as plain stacks"
        );
        assert_eq!(bank[0].count, 40, "5 parked first, then 35 merged in");
        assert_eq!(app.equip.coins[0].count, 60, "the coin rows untouched");
        assert_eq!(app.inventory[20].count, 0);
        assert_eq!(app.inventory[30].count, 0);
    }

    /// Dirty — an empty chest's LootAll is a no-op (the drain loop sees only
    /// air slots).
    #[test]
    fn loot_all_empty_chest_is_a_noop() {
        let mut app = stocked_chest_app(vec![]);
        app.inventory[20] = slot(9, 10);
        ready_buttons(&mut app);
        app.mouse_pos = Vec2::new(510.0, 300.0);
        app.ui_left_press = true;
        assert!(app.handle_chest_clicks());
        assert!(
            app.world_chests[0].slots.iter().all(|s| s.count == 0),
            "the empty chest stays empty"
        );
        assert_eq!(app.inventory[20].count, 10, "the inventory untouched");
    }

    /// Dirty — a chest holding only coin types merges nothing on QuickStack
    /// (the coin exclusion rides the target side, `ChestUI.cs:703-714`), and
    /// no MoveCoins runs for a world chest.
    #[test]
    fn quick_stack_with_a_chest_holding_only_coins_merges_nothing() {
        let mut app = stocked_chest_app(vec![slot(71, 30)]);
        app.inventory[20] = slot(9, 50);
        app.equip.coins[0] = slot(71, 5);
        ready_buttons(&mut app);
        app.mouse_pos = Vec2::new(510.0, 352.0);
        app.ui_left_press = true;
        assert!(app.handle_chest_clicks());
        assert_eq!(
            app.world_chests[0].slots[0],
            slot(71, 30),
            "the chest's coin stack is not a target"
        );
        assert_eq!(app.inventory[20].count, 50, "nothing merged");
        assert_eq!(app.equip.coins[0].count, 5, "no MoveCoins ran");
    }

    // ------------------------------------------------------------------
    // M3-CHEST-INT phase 4 — STORE-3: the favorite toggle + the trash swap.
    // ------------------------------------------------------------------

    /// Inside the trash slot's cell — `Main.cs:42352-42353`'s (448, 258) at
    /// scale 0.85 (`ui::inventory::trash_slot_at`).
    const TRASH_CELL: Vec2 = Vec2::new(470.0, 280.0);

    /// A plain open-inventory app with no container open (the favorite/trash
    /// surfaces; the player exists so the open-container reads are live).
    fn inv_app() -> App {
        let mut app = flow_app(None, vec![], vec![]);
        app.inventory_open = true;
        app.player = Some(crate::player::Player::spawn(&app.world));
        app
    }

    /// M7-UI-2b — the coin (ctx 1) and ammo (ctx 2) rows became click
    /// surfaces. `Main.cs:41752-41782` draws them through `ItemSlot.Handle`,
    /// so a left click runs the shared action-0 move behind
    /// `PickItemMovementAction`'s gate (`:1369-1371` coins, `:1372-1374` via
    /// `FitsAmmoSlot`) and a right click the `:1700-1772` split.
    const COIN_SLOT_0: Vec2 = Vec2::new(505.0, 110.0);
    const AMMO_SLOT_0: Vec2 = Vec2::new(540.0, 110.0);

    #[test]
    fn ui_2b_the_coin_row_takes_coins_and_refuses_everything_else() {
        let mut app = inv_app();
        // The vanilla move is two clicks: pick the stack up off the grid, then
        // drop it on the coin cell (the C# has no grid→row shortcut either).
        app.mouse_item = slot(71, 37);
        app.mouse_pos = COIN_SLOT_0;
        app.ui_left_press = true;
        app.handle_inventory_clicks();
        assert_eq!(app.equip.coins[0], slot(71, 37), "the stack moved in");
        assert_eq!(app.mouse_item.count, 0, "the cursor drained");
        // And back out: a click on the full cell lifts it.
        app.ui_left_press = true;
        app.handle_inventory_clicks();
        assert_eq!(app.mouse_item, slot(71, 37), "the cell emptied onto the cursor");
        assert_eq!(app.equip.coins[0].count, 0);
        app.equip.coins[0] = slot(71, 37);
        // `PickItemMovementAction`'s ctx-1 arm: only air and 71..=74 pass, so a
        // held wood stack stays on the cursor and the cell keeps its coins.
        app.mouse_item = slot(9, 5);
        app.ui_left_press = true;
        app.handle_inventory_clicks();
        assert_eq!(app.mouse_item, slot(9, 5), "the refused stack stayed held");
        assert_eq!(app.equip.coins[0], slot(71, 37), "the cell was untouched");
    }

    #[test]
    fn ui_2b_the_ammo_row_takes_the_ammo_set() {
        let mut app = inv_app();
        app.mouse_item = slot(40, 120);
        app.mouse_pos = AMMO_SLOT_0;
        app.ui_left_press = true;
        app.handle_inventory_clicks();
        assert_eq!(app.equip.ammo[0], slot(40, 120), "the arrows moved in");
        // A wood stack fails `FitsAmmoSlot` (`ammo`/`bait` both 0).
        app.mouse_item = slot(9, 5);
        app.ui_left_press = true;
        app.handle_inventory_clicks();
        assert_eq!(app.mouse_item, slot(9, 5), "the wood was refused");
        assert_eq!(app.equip.ammo[0], slot(40, 120));
    }

    #[test]
    fn ui_2b_the_rows_split_on_right_click() {
        let mut app = inv_app();
        app.equip.coins[1] = slot(72, 11);
        app.mouse_pos = Vec2::new(505.0, 145.0); // coin row 1
        app.ui_right_press = true;
        app.handle_inventory_clicks();
        assert_eq!(app.mouse_item, slot(72, 6), "the half rounded up");
        assert_eq!(app.equip.coins[1], slot(72, 5), "the cell keeps the rest");
    }

    /// M7-UI-2c — shift+click on a coin/ammo cell transfers into the open
    /// chest: `GetAlternateClickAction`'s ctx-1/ctx-2 arms (`ItemSlot.cs:657-665`)
    /// reach the same `TryPlacingInChest` check and the cursor-9 click arm
    /// carries the context's array into the merge/empty passes.
    #[test]
    fn ui_2c_shift_click_sends_the_coin_row_to_the_chest() {
        let mut app = chest_app(vec![slot(71, 5)]);
        app.equip.coins[0] = slot(71, 20);
        app.keys.insert(KeyCode::ShiftLeft);
        app.mouse_pos = COIN_SLOT_0;
        app.ui_left_press = true;
        // The chest handler owns the cursor-9 arm (`OverrideLeftClick`
        // `ItemSlot.cs:813-850`) and consumes the press.
        assert!(app.handle_chest_clicks(), "the cursor-9 arm takes the click");
        assert_eq!(app.cursor_override, CURSOR_TO_CHEST, "the cursor lit");
        assert_eq!(
            crate::chest::chest_slot(&app.world_chests, 0, 0),
            Some(slot(71, 25)),
            "the merge pass poured the coins in"
        );
        assert_eq!(app.equip.coins[0].count, 0, "the cell drained");
    }

    #[test]
    fn ui_2c_shift_click_sends_the_ammo_row_to_the_chest() {
        let mut app = chest_app(vec![]);
        app.equip.ammo[1] = slot(40, 60);
        app.keys.insert(KeyCode::ShiftLeft);
        app.mouse_pos = Vec2::new(540.0, 145.0); // ammo row 1
        app.ui_left_press = true;
        assert!(app.handle_chest_clicks(), "the cursor-9 arm takes the click");
        assert_eq!(app.cursor_override, CURSOR_TO_CHEST);
        assert_eq!(
            crate::chest::chest_slot(&app.world_chests, 0, 0),
            Some(slot(40, 60)),
            "the empty pass took the whole stack"
        );
        assert_eq!(app.equip.ammo[1].count, 0);
    }

    /// The placer's `Source` keeps the two arrays apart: a coin transfer writes
    /// the coin cell, never the grid slot with the same index.
    #[test]
    fn ui_2c_the_transfer_source_keeps_the_arrays_apart() {
        let mut app = chest_app(vec![]);
        app.inventory[0] = slot(9, 3);
        app.equip.coins[0] = slot(72, 4);
        assert!(crate::chest::try_placing_from(
            &mut app,
            crate::chest::Source::Coin(0),
            true
        ));
        assert!(!crate::chest::try_placing_from(
            &mut app,
            crate::chest::Source::Coin(0),
            false
        ));
        assert_eq!(crate::chest::chest_slot(&app.world_chests, 0, 0), Some(slot(72, 4)));
        assert_eq!(app.equip.coins[0].count, 0, "the coin cell drained");
        assert_eq!(app.inventory[0], slot(9, 3), "the grid slot is untouched");
    }

    #[test]
    fn ui_2b_the_rows_are_dead_while_the_inventory_is_closed() {
        let mut app = inv_app();
        app.inventory_open = false;
        app.equip.coins[0] = slot(71, 9);
        app.mouse_pos = COIN_SLOT_0;
        app.ui_left_press = true;
        app.handle_inventory_clicks();
        assert_eq!(app.equip.coins[0], slot(71, 9), "no lane off-screen");
        assert_eq!(app.mouse_item.count, 0);
    }

    /// DW-4.1 — Alt+left-click on an inventory slot toggles the favorite
    /// (`OverrideHover`'s cursor-3 arm `ItemSlot.cs:616-620` +
    /// `OverrideLeftClick`'s `:745-755` → `ToggleFavorited`): the click is
    /// consumed before the movement arms, so a held item neither swaps nor
    /// merges (`ItemSlot.cs:915-918` returns on the override's `true`).
    #[test]
    fn test_DW_4_1_alt_left_click_toggles_the_favorite() {
        let mut app = inv_app();
        app.inventory[5] = slot(9, 6);
        app.keys.insert(KeyCode::AltLeft);
        app.mouse_pos = INV_SLOT_5;
        app.mouse_item = slot(3, 1); // held: the override must eat the click
        app.ui_left_press = true;
        assert!(!app.handle_chest_clicks(), "no chest surface takes it");
        app.handle_inventory_clicks();
        assert!(app.is_favorite(5), "the toggle fired");
        assert_eq!(app.mouse_item, slot(3, 1), "the held item stayed");
        assert_eq!(app.inventory[5], slot(9, 6), "the slot item stayed");
        // The second Alt+click clears it — the toggle is single-click.
        app.ui_left_press = true;
        app.handle_chest_clicks();
        app.handle_inventory_clicks();
        assert!(!app.is_favorite(5), "the second click un-favorited");
    }

    /// DW-4.1 — without Alt the same click picks the stack up (the movement
    /// arm), and a right-click with Alt held runs the half-take ladder and
    /// never toggles (`OverrideLeftClick`'s cursor-3 arm is left-only; the
    /// right path is `RightClick`, a different entry).
    #[test]
    fn test_DW_4_1_without_alt_picks_up_and_right_click_never_toggles() {
        let mut app = inv_app();
        app.inventory[5] = slot(9, 6);
        app.mouse_pos = INV_SLOT_5;
        app.ui_left_press = true;
        app.handle_chest_clicks();
        app.handle_inventory_clicks();
        assert!(app.favorite_slots.is_empty(), "no favorite without Alt");
        assert_eq!(app.mouse_item, slot(9, 6), "the pickup ran");
        assert_eq!(app.inventory[5], EMPTY_SLOT);

        let mut app = inv_app();
        app.inventory[5] = slot(9, 6);
        app.keys.insert(KeyCode::AltLeft);
        app.mouse_pos = INV_SLOT_5;
        app.ui_right_press = true;
        app.handle_chest_clicks();
        app.handle_inventory_clicks();
        assert!(app.favorite_slots.is_empty(), "right-click never toggles");
        assert_eq!(app.mouse_item.count, 3, "the half-take ran");
        assert_eq!(app.inventory[5].count, 3);
    }

    /// DW-4.1 (dirty) — Alt+click on an empty slot never toggles: air returns
    /// before the key block (`ItemSlot.cs:607-609`), so the cursor never
    /// lights and the click falls to the movement arm — with an empty cursor
    /// that is a full no-op; with a held item the C# movement still places
    /// (the override never engages on air).
    #[test]
    fn test_DW_4_1_alt_click_on_empty_slot_never_toggles() {
        let mut app = inv_app();
        app.keys.insert(KeyCode::AltLeft);
        app.mouse_pos = INV_SLOT_5; // slot 5 empty
        app.ui_left_press = true;
        app.handle_chest_clicks();
        assert_eq!(app.cursor_override, -1, "air returns before the key block");
        app.handle_inventory_clicks();
        assert!(app.favorite_slots.is_empty());
        assert_eq!(app.mouse_item, EMPTY_SLOT, "nothing moved");

        let mut app = inv_app();
        app.keys.insert(KeyCode::AltLeft);
        app.mouse_pos = INV_SLOT_5;
        app.mouse_item = slot(9, 4);
        app.ui_left_press = true;
        app.handle_chest_clicks();
        app.handle_inventory_clicks();
        assert!(app.favorite_slots.is_empty());
        assert_eq!(app.inventory[5], slot(9, 4), "the C# movement places on air");
        assert_eq!(app.mouse_item, EMPTY_SLOT);
    }

    /// DW-4.2 — the `canFavoriteAt` context table holds (`ItemSlot.cs:386-389`)
    /// and the toggle returns `PlaySound(12)`'s legacy id on every flip
    /// (`ToggleFavorited`, `ItemSlot.cs:873-877` — the `toggle_open` pinning
    /// precedent); the coin-row guard refuses with no sound.
    #[test]
    fn test_DW_4_2_can_favorite_gate_holds_and_toggle_plays_sound_12() {
        assert!(chest::can_favorite_at(0));
        assert!(chest::can_favorite_at(1));
        assert!(chest::can_favorite_at(2));
        assert!(chest::can_favorite_at(32));
        assert!(!chest::can_favorite_at(3), "chest slots cannot be favorited");
        assert!(!chest::can_favorite_at(5));
        assert!(!chest::can_favorite_at(6), "the trash slot cannot be favorited");
        assert!(!chest::can_favorite_at(13), "the closed hotbar cannot either");

        let mut app = inv_app();
        assert_eq!(app.toggle_favorite(7), 12);
        assert!(app.is_favorite(7));
        assert_eq!(app.toggle_favorite(7), 12, "the C# plays 12 on both flips");
        assert!(!app.is_favorite(7));
        assert_eq!(app.toggle_favorite(crate::item::INV), 0, "the coin-row guard");
        assert_eq!(app.toggle_favorite(53), 0);

        // The hover gate refuses air — the cursor never lights (the click-path
        // half of the gate; the table asserts above are the static half).
        app.keys.insert(KeyCode::AltLeft);
        app.mouse_pos = INV_SLOT_5; // empty
        app.handle_chest_clicks();
        assert_eq!(app.cursor_override, -1);
    }

    /// DW-4.2 — the closed hotbar is context 13, not in `canFavoriteAt`: an
    /// Alt+click selects the slot like the C# and never toggles.
    #[test]
    fn test_DW_4_2_alt_click_closed_hotbar_selects_never_toggles() {
        let mut app = inv_app();
        app.inventory[3] = slot(9, 6);
        app.inventory_open = false;
        app.keys.insert(KeyCode::AltLeft);
        // Inside closed-hotbar slot 3 (x 20 + 3·43 = 149..188, y 25..64 at
        // the 0.75 idle scale — `ui::slot::closed_hotbar_metrics`).
        app.mouse_pos = Vec2::new(160.0, 40.0);
        app.ui_left_press = true;
        app.handle_chest_clicks();
        app.handle_inventory_clicks();
        assert_eq!(app.selected, 3, "the select ran");
        assert!(app.favorite_slots.is_empty(), "context 13 cannot favorite");
    }

    /// DW-4.3 — a shift-click on a favorited slot is a no-op as the alternate
    /// action: `GetAlternateClickAction`'s head guard (`ItemSlot.cs:631-634`)
    /// keeps cursor 9 dark, so the cursor-9 transfer never fires. The click
    /// then falls through to the C# movement arm (the override returned
    /// false) — a plain pickup, with the favorite retained on the slot
    /// (context 0 is in `retainFavoriteOnMoveAt`).
    #[test]
    fn test_DW_4_3_shift_click_on_favorited_slot_is_noop() {
        let mut app = stocked_chest_app(vec![slot(9, 5)]);
        app.inventory[5] = slot(3, 10);
        app.favorite_slots.push(5);
        app.keys.insert(KeyCode::ShiftLeft);
        app.mouse_pos = INV_SLOT_5;
        app.ui_left_press = true;
        assert!(!app.handle_chest_clicks());
        assert_eq!(
            app.cursor_override, -1,
            "the favorited guard keeps cursor 9 dark"
        );
        assert_eq!(
            app.world_chests[0].slots[0],
            slot(9, 5),
            "nothing transferred into the chest"
        );
        app.handle_inventory_clicks();
        assert_eq!(app.mouse_item, slot(3, 10), "the fall-through pickup ran");
        assert_eq!(app.favorite_slots, vec![5], "context 0 retains the favorite");
    }

    /// DW-4.3 — direct moves on favorited slots work (the C# has no
    /// immovability rule for direct clicks) and the retain table leaves the
    /// favorite in place: context 0 retains, so nothing clears on the move
    /// (the slot-positional v1 model — the `ItemSlot.cs:1728-1734` flag-carry
    /// is unrepresentable, Phase 5's ANALYSIS note).
    #[test]
    fn test_DW_4_3_direct_moves_work_and_favorite_stays_on_the_slot() {
        let mut app = inv_app();
        app.inventory[5] = slot(9, 6);
        app.favorite_slots.push(5);
        app.mouse_pos = INV_SLOT_5;
        app.ui_left_press = true;
        app.handle_chest_clicks();
        app.handle_inventory_clicks();
        assert_eq!(app.mouse_item, slot(9, 6), "the direct move worked");
        assert_eq!(app.favorite_slots, vec![5], "retain[0]: nothing cleared");
    }

    /// DW-4.4 — the trash slot's left click is the plain `MouseItemSwap`
    /// (`ItemSlot.cs:937-941` + `:1255-1263`): all four arms.
    #[test]
    fn test_DW_4_4_left_click_trash_swaps_held_and_trash() {
        // held ↔ trash: both sides move.
        let mut app = inv_app();
        app.mouse_item = slot(9, 4);
        app.equip.trash = slot(3, 2);
        app.mouse_pos = TRASH_CELL;
        app.ui_left_press = true;
        app.handle_inventory_clicks();
        assert_eq!(app.equip.trash, slot(9, 4), "the held item was trashed");
        assert_eq!(app.mouse_item, slot(3, 2), "the old trash returned");

        // held + empty trash: the item is trashed, the cursor empties.
        let mut app = inv_app();
        app.mouse_item = slot(9, 4);
        app.mouse_pos = TRASH_CELL;
        app.ui_left_press = true;
        app.handle_inventory_clicks();
        assert_eq!(app.equip.trash, slot(9, 4));
        assert_eq!(app.mouse_item, EMPTY_SLOT);

        // empty held + trash: the trash returns to the cursor.
        let mut app = inv_app();
        app.equip.trash = slot(3, 2);
        app.mouse_pos = TRASH_CELL;
        app.ui_left_press = true;
        app.handle_inventory_clicks();
        assert_eq!(app.mouse_item, slot(3, 2));
        assert_eq!(app.equip.trash, EMPTY_SLOT);

        // empty-on-empty: nothing moves anywhere.
        let mut app = inv_app();
        app.mouse_pos = TRASH_CELL;
        app.ui_left_press = true;
        app.handle_inventory_clicks();
        assert_eq!(app.mouse_item, EMPTY_SLOT);
        assert_eq!(app.equip.trash, EMPTY_SLOT);
    }

    /// The trash hit region sits outside the inventory grid's rows — a click
    /// at the trash cell must not also resolve as a grid slot (the dispatch's
    /// trash arm precedes the grid, but the geometry should not overlap).
    #[test]
    fn trash_cell_is_outside_the_inventory_grid() {
        let mut app = inv_app();
        app.mouse_pos = TRASH_CELL;
        assert!(ui::inventory::trash_slot_at(TRASH_CELL, false));
        assert_eq!(app.slot_at_mouse(), None, "the grid rows end at y 258");
    }

    // ------------------------------------------------------------------
    // The PVP toggle / team select cells (`Main.DrawPVPIcons`).
    // ------------------------------------------------------------------

    /// The screen point of the PVP toggle's hover box and of one team cell, at
    /// the test app's 1600x900 frame with the minimap live (`Main.mH = 256`).
    fn pvp_points(app: &App) -> (Vec2, Vec2) {
        let (sw, sh) = app.screen_size();
        let screen = Vec2::new(sw, sh);
        let m_h = app.main_m_h();
        let (x, y) = ui::chrome::pvp_icon_xy(screen.x, app.equip_page, m_h);
        // `num2 = x + 10`, and the box is `num2 - 7 .. num2 + 25` (`:42678`).
        let toggle = Vec2::new(x + 10.0, y + 10.0);
        // The team grid starts at `num2 -= 10; num3 += 60` (`:42718`).
        let team0 = Vec2::new(x + 6.0, y + 66.0);
        (toggle, team0)
    }

    /// A headless app with the PVP icons' draw/click gates forced open (the
    /// `TR_PVP` pin, held in a field so the test does not race the `OnceLock`).
    fn pvp_app() -> App {
        let mut app = inv_app();
        app.pvp_icons_forced = true;
        app
    }

    /// `Main.cs:42828-42858` + `:42864-42867`: a press on a page tab sets
    /// `EquipPageSelected`, the drawn page follows it on the next frame, and the
    /// next frame's `SelectEquipPage` joins in without an item held.
    #[test]
    fn page_tab_click_switches_the_equipment_page() {
        let mut app = inv_app();
        let (sw, _sh) = app.screen_size();
        let screen = Vec2::new(sw, app.screen_size().1);
        let y = 174.0 + app.main_m_h() - 32.0;
        // The test harness loads no atlas, so stand in the native tab sheet
        // (the `UI/DisplaySlots_*` cells are 22x18 in the retail XNB).
        for i in [2usize, 4, 8] {
            app.lookups.page_icons[i] = Some(tr_render::SheetRect {
                x: 0,
                y: 0,
                width: 22,
                height: 18,
                plain_frame: (0, 0),
            });
        }
        let sizes = ui::chrome::page_tab_sizes(&app.lookups);
        assert_eq!(sizes[0], (22.0, 18.0));
        // Tab index 0 is `EquipPageSelected = 2` (the misc page).
        let tab2 = Vec2::new(ui::chrome::page_tab_x(screen.x, 0) + 4.0, y + 4.0);
        app.mouse_pos = tab2;
        let before = app.audio.played_count();
        app.ui_left_press = true;
        app.handle_inventory_clicks();
        assert_eq!(app.equip_page_selected, 2);
        assert_eq!(app.audio.played_count(), before + 1, "`PlaySound(10)`");
        // The frame tail resolves `EquipPage` from the empty cursor stack.
        app.resolve_equip_page();
        assert_eq!(app.equip_page, 2, "`:42866` falls back to the selection");
        // Pressing the same tab again falls back to page 0 (`:42853-42857`).
        app.mouse_pos = tab2;
        app.ui_left_press = true;
        app.handle_inventory_clicks();
        assert_eq!(app.equip_page_selected, 0);
        app.resolve_equip_page();
        assert_eq!(app.equip_page, 0);
    }

    /// `ItemSlot.SelectEquipPage` (`ItemSlot.cs:4647-4650`): holding a piece of
    /// armour forces page 0 even when the misc page was selected.
    #[test]
    fn held_armour_forces_the_equipment_page() {
        let mut app = inv_app();
        app.equip_page_selected = 2;
        let armour = (0u16..6196)
            .find(|&k| crate::itemdata::ITEMS[k as usize].head_slot != -1)
            .expect("a helmet exists");
        app.mouse_item = slot(armour, 1);
        app.resolve_equip_page();
        assert_eq!(app.equip_page, 0, "`:4647-4650`");
        // Dropping the stack restores the selection.
        app.mouse_item = crate::item::EMPTY_SLOT;
        app.resolve_equip_page();
        assert_eq!(app.equip_page, 2);
    }

    /// `Main.cs:42793`: with the `dye > 0` half of tab 2's gate unported a held
    /// dye is refused like any other stack — the documented deviation.
    #[test]
    fn a_held_stack_refuses_the_page_tabs() {
        let mut app = inv_app();
        let (sw, sh) = app.screen_size();
        let screen = Vec2::new(sw, sh);
        let y = 174.0 + app.main_m_h() - 32.0;
        let tab2 = Vec2::new(ui::chrome::page_tab_x(screen.x, 0) + 4.0, y + 4.0);
        app.mouse_item = slot(3, 5);
        app.mouse_pos = tab2;
        app.ui_left_press = true;
        app.handle_inventory_clicks();
        assert_eq!(app.equip_page_selected, 0, "the press was refused");
    }

    /// `Main.cs:42689-42694`: a press on the toggle flips `hostile`, arms the
    /// 300-tick cooldown and plays `SoundEngine.PlaySound(12)`. A second press
    /// inside the cooldown does nothing.
    #[test]
    fn pvp_toggle_click_flips_hostile_under_the_cooldown() {
        let mut app = pvp_app();
        let (toggle, _) = pvp_points(&app);
        app.mouse_pos = toggle;
        let before = app.audio.played_count();
        app.ui_left_press = true;
        app.handle_inventory_clicks();
        assert!(app.player.as_ref().unwrap().hostile, "the toggle flipped");
        assert_eq!(app.team_cooldown, ui::chrome::TEAM_COOLDOWN_LEN);
        assert_eq!(app.audio.played_count(), before + 1, "the Menu_Tick cue");
        // Inside the cooldown the gate refuses (`:42689`'s `teamCooldown == 0`).
        app.ui_left_press = true;
        app.handle_inventory_clicks();
        assert!(app.player.as_ref().unwrap().hostile, "still flipped once");
    }

    /// `Main.cs:42753-42767`: a press on a team cell sets `team`, and pressing
    /// the player's own team is refused by `player.team != i`.
    #[test]
    fn team_cell_click_sets_the_team_and_refuses_the_same_one() {
        let mut app = pvp_app();
        let (_, team0) = pvp_points(&app);
        // Team 0 is where the player already stands — refused, no cooldown.
        app.mouse_pos = team0;
        app.ui_left_press = true;
        app.handle_inventory_clicks();
        assert_eq!(app.player.as_ref().unwrap().team, 0);
        assert_eq!(app.team_cooldown, 0, "nothing happened, no cooldown armed");
        // The next cell along (`i = 1`, +20 px on x) is Red.
        app.mouse_pos = team0 + Vec2::new(20.0, 0.0);
        app.ui_left_press = true;
        app.handle_inventory_clicks();
        assert_eq!(app.player.as_ref().unwrap().team, 1, "joined the Red team");
        assert_eq!(app.team_cooldown, ui::chrome::TEAM_COOLDOWN_LEN);
    }

    /// The cooldown decays once per frame (`Main.cs:17048-17051`) and never
    /// below zero.
    #[test]
    fn team_cooldown_decays_per_frame() {
        let mut app = pvp_app();
        app.team_cooldown = 2;
        app.frame();
        assert_eq!(app.team_cooldown, 1);
        app.frame();
        assert_eq!(app.team_cooldown, 0);
        app.frame();
        assert_eq!(app.team_cooldown, 0, "clamped at zero");
    }

    /// The icons are inert while the gates are closed (`Main.netMode != 1`
    /// without the pin, `Main.cs:2909`).
    #[test]
    fn pvp_click_is_inert_without_the_gates() {
        let mut app = inv_app();
        let (toggle, team0) = pvp_points(&app);
        app.mouse_pos = toggle;
        app.ui_left_press = true;
        app.handle_inventory_clicks();
        assert!(!app.player.as_ref().unwrap().hostile);
        app.mouse_pos = team0 + Vec2::new(20.0, 0.0);
        app.ui_left_press = true;
        app.handle_inventory_clicks();
        assert_eq!(app.player.as_ref().unwrap().team, 0);
    }
}
