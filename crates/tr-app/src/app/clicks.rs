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
/// `AlternateClickAction.Sell`'s cursor id (`ItemSlot.cs:218`): the
/// Shift-click sell while a shop is open (`GetSellOrTrash`, `:198-209`).
const CURSOR_SELL: i32 = 10;

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

/// The favorite key of a ctx-1/ctx-2 cell — the vanilla inventory numbering
/// (`crate::inv::coin_slot_index`/`ammo_slot_index`), so a row favorite and a
/// grid favorite live in one key space (`App::toggle_favorite`).
fn row_favorite_index(context: i32, k: usize) -> usize {
    if context == 1 {
        crate::inv::coin_slot_index(k)
    } else {
        crate::inv::ammo_slot_index(k)
    }
}

/// `ItemID.CopperCoin..PlatinumCoin` — the four kinds the ctx-1 row takes
/// (`ItemSlot.PickItemMovementAction` `:1369-1371`).
fn is_coin_kind(kind: u16) -> bool {
    (71..=74).contains(&kind)
}

/// The shop row as the `ItemSlot` the movement/stack gates read — the
/// stock instance's kind/prefix/stack (`ShopEntry`; the fresh
/// `SetDefaults` rows carry prefix 0, stack 1).
fn shop_row_slot(entry: &crate::shop::ShopEntry) -> ItemSlot {
    ItemSlot {
        kind: entry.item,
        name: crate::itemdata::ITEMS[entry.item as usize].name,
        count: entry.stack,
        prefix: entry.prefix,
    }
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

/// `ItemSlot.CanEquipInArmorSlot`'s port lives in `app::equip_swap`
/// (S1-P-i48) — the full gate incl. the `HasSameItemInSlot` duplicate half,
/// the `DualEquipArmor` bypass (`ItemID.cs:336`) and the accessory collision
/// set, all reading the live `Player.armor` arrays.

/// `ItemSlot.PickItemMovementAction` (`ItemSlot.cs:1484-1634`) — the action id
/// the C# assigns to the cursor's `check` item over `slot` of `context`: `-1`
/// refuses the move, `0` is the generic move, `1` an equip, `2` a dye, `3`/`4`
/// the shop arms, `5` the Journey duplicate. `target` is the C#'s `inv[slot]`,
/// read by the ctx 15/29 arms only. `equip` carries the C#'s
/// `Main.LocalPlayer` + `Player.armor` for the ctx 8-11
/// `CanEquipInArmorSlot` gate (S1-P-i48); `None` (the coin/ammo callers, which
/// never reach those arms) refuses them — there is no live player state to
/// check against.
///
/// Contexts whose C# arm reads data the port does not carry (`CanHavePrefixes`
/// for ctx 5, `Main.projHook` for ctx 16, `mountType` for 17/18/39, the
/// `vanityPet`/`lightPet` tables for 19/20, `TEDisplayDoll` for 24/38) keep the
/// C#'s `type == 0` shortcut and accept a non-air item — the same result the
/// old `_ => true` gave, recorded as the approximation in docs/ANALYSIS.md.
pub(crate) fn pick_item_movement_action(
    context: i32,
    check: &ItemSlot,
    slot: usize,
    target: &ItemSlot,
    equip: Option<(&crate::player::Player, &[ItemSlot; crate::inv::ARMOR])>,
) -> i32 {
    let air = check.kind == 0;
    let def = registry::get(check.kind);
    let mut result = -1;
    match context {
        // `:1490-1492`
        0 => result = 0,
        // `:1493-1498`
        1 => {
            if air || is_coin_kind(check.kind) {
                result = 0;
            }
        }
        // `:1499-1504`
        2 => {
            if fits_ammo_slot(check.kind) {
                result = 0;
            }
        }
        // `:1505-1507`
        3 => result = 0,
        // `:1508-1514` — `ChestUI.IsBlockedFromTransferIntoChest` only blocks a
        // favorited money-trough / piggy item; the port carries no `favorited`
        // flag yet, so the gate never blocks.
        4 | 32 => result = 0,
        // `:1515-1520` — `CanHavePrefixes` (`GetRollablePrefixes`) is not
        // modelled: the C# air branch is kept, a non-air item accepts.
        5 => result = 1,
        // `:1521-1523`
        6 => result = 0,
        // `:1524-1529`
        7 => {
            if def.is_some_and(|d| d.material) || air {
                result = 0;
            }
        }
        // `:1530-1538` — `CanEquipInArmorSlot` over the live player + armour
        // set (S1-P-i48: the duplicate/collision halves and the
        // `DualEquipArmor` bypass are modelled in `equip_swap`).
        8 | 9 | 10 | 11 => {
            if air
                || equip.is_some_and(|(p, armor)| {
                    crate::app::equip_swap::can_equip_in_armor_slot(check, slot, p, armor)
                })
            {
                result = 1;
            }
        }
        // `:1560-1565`
        12 | 25 | 27 | 33 => result = 2,
        // `:1566-1579` — the shop buy/sell ladder.
        15 => {
            let max = registry::max_stack(check.kind);
            if air && target.kind > 0 {
                result = 3;
            } else if check.kind == target.kind
                && check.kind > 0
                && check.count < max
                && target.count > 0
            {
                result = 3;
            } else if target.kind == 0 && check.kind > 0 && !is_coin_kind(check.kind) {
                result = 4;
            }
        }
        // `:1580-1585` — `Main.projHook[shoot]` not modelled: air branch kept.
        16 => result = 1,
        // `:1586-1597`, `:1610-1615` — `mountType` not modelled: air branch kept.
        17 | 18 | 39 => result = 1,
        // `:1604-1609`, `:1616-1621` — `vanityPet`/`lightPet` not modelled.
        19 | 20 => result = 1,
        // `:1539-1544`
        23 => {
            if air
                || (slot == 0 && def.is_some_and(|d| d.head_slot > 0))
                || (slot == 1 && def.is_some_and(|d| d.body_slot > 0))
                || (slot == 2 && def.is_some_and(|d| d.leg_slot > 0))
            {
                result = 1;
            }
        }
        // `:1551-1559` — `CanEquipAccessoryInSlot`'s collision set is not
        // modelled; the `accessory` flag is the whole port test.
        24 => {
            if air || def.is_some_and(|d| d.accessory) {
                result = 1;
            }
        }
        // `:1545-1550`
        26 => {
            if air || def.is_some_and(|d| d.head_slot > 0) {
                result = 1;
            }
        }
        // `:1622-1627`
        29 => {
            if air && target.kind > 0 {
                result = 5;
            }
        }
        // `:1629-1632`
        30 => result = 0,
        // `:1598-1603` — `TEDisplayDoll.AcceptedInWeaponSlot` not modelled.
        38 => result = 1,
        _ => {}
    }
    result
}

/// The accepted-move gate `PickItemMovementAction` gives a context before the
/// shared action-0 move runs: `result != -1`. The coin/ammo callers carry no
/// slot or target, so the slot-only arms (8-11, 23, 26) and the ctx 15/29
/// target arms answer for the empty slot — neither row reaches them.
pub(crate) fn context_accepts(context: i32, kind: u16) -> bool {
    let check = ItemSlot {
        kind,
        name: "",
        count: u32::from(kind != 0),
        prefix: 0,
    };
    pick_item_movement_action(context, &check, 0, &EMPTY_SLOT, None) != -1
}

/// `Main.DrawInventory`'s misc-equip context -> `Player.miscEquips` index
/// (`Main.cs:41196-41215`: 19 pet, 20 light, 18 cart, 17 mount, 16 grapple —
/// the same `MISC_CONTEXT` the draw lays out).
fn misc_index(context: i32) -> Option<usize> {
    Some(match context {
        19 => 0,
        20 => 1,
        18 => 2,
        17 => 3,
        16 => 4,
        _ => return None,
    })
}

/// `Item.IsNotTheSameAs` (`Item.cs:49847-49854`): different type, or the same
/// count and a different prefix.
fn is_not_the_same_as(a: &ItemSlot, b: &ItemSlot) -> bool {
    if a.kind == b.kind && a.count == b.count {
        return a.prefix != b.prefix;
    }
    true
}

/// `ItemSlot.LeftClick`'s action-1 body (`ItemSlot.cs:1019-1119`) — the equip
/// rows. The achievement hooks and the display-doll net packets are
/// constants-off (ANALYSIS). Returns whether a cue should play.
fn equip_left_click(held: &mut ItemSlot, target: &mut ItemSlot) -> bool {
    if held.count == 1 && held.kind > 0 && target.kind > 0 && is_not_the_same_as(target, held) {
        // `:1020-1023` — the one-for-one swap with something already worn.
        std::mem::swap(held, target);
        true
    } else if held.count == 0 && target.kind > 0 {
        // `:1044-1058` — taking the worn piece onto an air cursor.
        std::mem::swap(held, target);
        true
    } else if held.kind > 0 && target.count == 0 {
        // `:1060-1084` — placing onto an empty row: one at a time unless the
        // cursor holds a single.
        if held.count == 1 {
            std::mem::swap(held, target);
        } else {
            *target = ItemSlot {
                count: 1,
                ..*held
            };
            held.count -= 1;
        }
        true
    } else {
        false
    }
}

/// `ItemSlot.LeftClick`'s action-2 body (`ItemSlot.cs:1121-1218`) — the dye
/// rows, which additionally demand that a held dye is not the worn kind.
fn dye_left_click(held: &mut ItemSlot, target: &mut ItemSlot, held_dye: bool) -> bool {
    if held.count == 1 && held_dye && target.kind > 0 && target.kind != held.kind {
        // `:1122-1125`
        std::mem::swap(held, target);
        true
    } else if held.count == 0 && target.kind > 0 {
        // `:1148-1162`
        std::mem::swap(held, target);
        true
    } else if held_dye && target.count == 0 {
        // `:1164-1187` — the multi-stack path is `SetDefaults(type)`, which
        // resets the prefix (`Item.cs:49022` sets `stack = 1`).
        if held.count == 1 {
            std::mem::swap(held, target);
        } else {
            held.count -= 1;
            *target = ItemSlot {
                kind: held.kind,
                name: held.name,
                count: 1,
                prefix: 0,
            };
        }
        true
    } else {
        false
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

    /// The shop grid's cell under the cursor — `Main.DrawInventory`'s shop
    /// block (`Main.cs:41798-41818`): 10×4 cells at `(int)(73 + i*56*0.755)`,
    /// `(int)(invBottom + j*56*0.755)` with `Main.invBottom = 258`
    /// (`Main.cs:6608`), the hit rect the `InventoryBack` size × 0.755
    /// (`:41811`, inclusive on all four edges). `None` while no shop draws
    /// (`npcShop > 0 && playerInventory`, the `:41784-41788` gate).
    pub(crate) fn shop_slot_at_mouse(&self) -> Option<usize> {
        if self.shop_npc.is_none() || !self.inventory_open {
            return None;
        }
        let cell = self.lookups.ui_back.map_or(52.0, |b| b.width as f32);
        ui::inventory::shop_slot_at(self.mouse_pos, cell)
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
            // `:631-634`: a favorited slot answers null in every context, so a
            // favorited coin/ammo cell never lights the transfer cursor.
            if self.is_favorite(row_favorite_index(ctx, k)) {
                return None;
            }
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
    /// `PickItemMovementAction`'s gate, right the `:1700-1772` split, Alt the
    /// favorite toggle on the row's own key (`row_favorite_index`).
    fn coin_ammo_click(&mut self, context: i32, k: usize, left: bool, right_edge: bool) {
        if k >= crate::inv::COINS {
            return;
        }
        // `OverrideLeftClick`'s cursor-3 arm (`ItemSlot.cs:745-755`): the
        // favorite toggle eats the press before the movement arms.
        if left && self.cursor_override == CURSOR_FAVORITE {
            let idx = row_favorite_index(context, k);
            if self.toggle_favorite(idx) != 0 {
                self.audio.play("Menu_Tick"); // `SoundEngine.PlaySound(12)`
            }
            return;
        }
        let mut held = self.mouse_item;
        let mut target = if context == 1 {
            self.equip.coins[k]
        } else {
            self.equip.ammo[k]
        };
        // The `:1707` pickup gate: an air cursor, or the same kind/prefix with
        // cursor room.
        let can_take = held.count == 0
            || (crate::bank::can_stack(&held, &target)
                && held.count < registry::max_stack(target.kind));
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
            self.mouse_item = held;
        } else if can_take {
            // `RightClick`'s pickup block (`ItemSlot.cs:1700-1713`): the paced
            // one-per-transfer `PickupItemIntoMouse` split (held-repeat
            // included).
            target = self.right_click_split(target);
        } else if right_edge {
            // The port's press-edge place-one / merge-one carry (research/20
            // §5, the flat-`MAX_STACK` deviation) — not the C#'s pickup
            // direction, kept on the edge so a held button cannot spam it.
            if target.count == 0 {
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
        }
        if context == 1 {
            self.equip.coins[k] = target;
        } else {
            self.equip.ammo[k] = target;
        }
        // `CoinSlot.ForceSlotState(slot, context, inv[slot])` — the transfer
        // tails' rank reset (`ItemSlot.cs:1016`, S1-H-9); the row's vanilla
        // index rides `coin_slot_index`/`ammo_slot_index`.
        let vs = if context == 1 {
            crate::inv::coin_slot_index(k)
        } else {
            crate::inv::ammo_slot_index(k)
        } as i32;
        self.coin_slot
            .force_slot_state(vs, context, target.kind, target.count);
    }

    /// The equipment cell `(context, index)`'s item, through the split arrays
    /// (`App::equip`): armour 8/10 and vanity 9/11 both live in
    /// `Player.armor`, 12 in `Player.dye`, 16-20 in `miscEquips`, 33 in
    /// `miscDyes` (`ItemSlot.cs:42-110`'s context ids).
    fn equip_cell(&self, context: i32, idx: usize) -> ItemSlot {
        if let Some(m) = misc_index(context) {
            return self.equip.misc_equips[m];
        }
        match context {
            8 | 9 | 10 | 11 => self.equip.armor.get(idx).copied().unwrap_or(EMPTY_SLOT),
            12 => self.equip.dye.get(idx).copied().unwrap_or(EMPTY_SLOT),
            33 => self.equip.misc_dyes.get(idx).copied().unwrap_or(EMPTY_SLOT),
            _ => EMPTY_SLOT,
        }
    }

    fn set_equip_cell(&mut self, context: i32, idx: usize, v: ItemSlot) {
        if let Some(m) = misc_index(context) {
            self.equip.misc_equips[m] = v;
            return;
        }
        match context {
            8 | 9 | 10 | 11 => {
                if let Some(c) = self.equip.armor.get_mut(idx) {
                    *c = v;
                }
            }
            12 => {
                if let Some(c) = self.equip.dye.get_mut(idx) {
                    *c = v;
                }
            }
            33 => {
                if let Some(c) = self.equip.misc_dyes.get_mut(idx) {
                    *c = v;
                }
            }
            _ => {}
        }
    }

    /// One equipment cell's press — `ItemSlot.Handle` over the armour, vanity,
    /// dye and misc rows (`Main.cs:41288,41450,41491,41519`): left runs
    /// `PickItemMovementAction`'s action 1 (equip) / 2 (dye) body, right the
    /// `RightClick` ladder's vanity swap (`:1678-1684`) or the shared paced
    /// pickup split.
    fn equip_slot_click(&mut self, context: i32, idx: usize, left: bool, right_edge: bool) {
        // `LeftClick`'s busy gate (`ItemSlot.cs:922-926`): while an item
        // animation or the use cooldown runs, the press does nothing (the
        // `OverrideLeftClick` half rides the cursor-override arms above).
        // `App::swing` is `player.itemAnimation > 0`, `App::item_time` the
        // `player.itemTime` countdown (S1-P-i48).
        if left && (self.swing > 0.0 || self.item_time > 0.0) {
            return;
        }
        let mut held = self.mouse_item;
        let mut target = self.equip_cell(context, idx);
        if left {
            // `:927` — the action table; `:928-931` refuses everything but
            // action 3 off a press edge. The ctx 8-11 gate reads the live
            // player + armour set (`CanEquipInArmorSlot`, S1-P-i48).
            let equip_ctx = self
                .player
                .as_ref()
                .map(|p| (p, &self.equip.armor));
            match pick_item_movement_action(context, &held, idx, &target, equip_ctx) {
                1 => {
                    if equip_left_click(&mut held, &mut target) {
                        self.audio.play("Grab"); // `:1023,1057,1075,1083`
                    }
                }
                2 => {
                    let held_dye = registry::get(held.kind).is_some_and(|d| d.dye > 0);
                    if dye_left_click(&mut held, &mut target, held_dye) {
                        self.audio.play("Grab"); // `:1125,1161,1179,1186`
                    }
                }
                _ => {}
            }
            self.mouse_item = held;
        } else {
            // `RightClick`'s busy gate (`ItemSlot.cs:1644-1647`):
            // `player.itemAnimation > 0` returns before every arm.
            if self.swing > 0.0 {
                return;
            }
            // `RightClick` (`:1654-1698`): the swap arms need an air cursor or
            // one that cannot stack with the cell.
            let can_stack = crate::bank::can_stack(&held, &target);
            if held.count == 0 || !can_stack {
                // `:1668-1675` — the `HasItemSwap` variant cycle
                // (`ItemID.cs:1385`), on the press edge, for a single item in
                // a `canItemSwapAt` context (0/3/4/32/8/10, `:451-456`).
                // S1-P-i48: the equipment contexts 8/10 join the grid's arm.
                if matches!(context, 8 | 10)
                    && target.count == 1
                    && crate::sortdata::HAS_ITEM_SWAP.get(target.kind)
                {
                    if right_edge {
                        self.try_item_swap_click(context, idx);
                    }
                    return;
                }
                if context == 9 || context == 11 {
                    // `:1678-1684` — `SwapVanityEquip(inv, context, slot,
                    // player)` against the real counterpart `slot - 10`.
                    if right_edge {
                        self.vanity_swap(idx, context);
                    }
                    return;
                }
                // The C#'s `:1685-1697` `SwapEquip` arm covers contexts
                // 0/3/4/32 ONLY — an equip cell (8/10/12/16-20/33) right-
                // clicks straight into the pickup block below (unequip is
                // the SHIFT arm, `GetAlternateClickAction` `:700-715`).
            }
            // `:1700-1713` — the paced one-per-transfer split.
            let can_take = held.count == 0
                || (can_stack && held.count < registry::max_stack(target.kind));
            if can_take {
                target = self.right_click_split(target);
            } else if right_edge {
                // The port's press-edge place-one carry (research/20 §5).
                if target.count == 0 {
                    target = ItemSlot {
                        count: 1,
                        ..held
                    };
                    held.count -= 1;
                    if held.count == 0 {
                        held = EMPTY_SLOT;
                    }
                    self.mouse_item = held;
                }
            }
        }
        self.set_equip_cell(context, idx, target);
    }

    /// `ItemSlot.TryItemSwap` + `AfterItemSwap` (`ItemSlot.cs:1901-2009`)
    /// applied to an equipment cell (S1-P-i48): the variant cycle, the
    /// 22/7 cue split, `Main.stackSplit = 30` and the consumed press edge
    /// (`Main.mouseRightRelease = false`).
    fn try_item_swap_click(&mut self, context: i32, idx: usize) {
        let mut item = self.equip_cell(context, idx);
        let Some(new_kind) = crate::app::equip_swap::try_item_swap(&mut item) else {
            return;
        };
        self.set_equip_cell(context, idx, item);
        // `:1999-2006` — the fifteen kinds play 22 (Unlock), the rest 7.
        self.audio
            .play(crate::app::equip_swap::after_item_swap_cue(new_kind));
        self.stack_split = 30; // `:2007`
        self.ui_right_press = false; // `:2008`
    }

    /// `ItemSlot.SwapEquip` (`ItemSlot.cs:3283-3363`) applied to a container
    /// cell: the item is reassigned to its equipment slot and the destination's
    /// previous content lands back in `slot`. The favourites clear on both
    /// sides (`SwapClearFavorite`, `:349/:3635`; `EquipSwap`'s `:3644`), which
    /// for the port means dropping the grid cell's positional favourite — the
    /// equipment arrays are outside the favourite key space.
    fn swap_equip_click(&mut self, ctx0_index: Option<usize>, item: &mut ItemSlot) -> bool {
        if item.count == 0 {
            return false; // `:3286-3289`
        }
        use crate::app::equip_swap::{SwapTarget, swap_equip_target};
        // The unlock gates read the live player (S1-P-i48); no player means
        // no equipment state to swap into.
        let Some(p) = self.player.as_ref() else {
            return false;
        };
        let Some(target) = swap_equip_target(
            item,
            &self.equip.armor,
            &self.equip.dye,
            &mut self.dye_swap_counter,
            p,
        ) else {
            return false;
        };
        match target {
            SwapTarget::Armor(slot) => std::mem::swap(item, &mut self.equip.armor[slot]),
            SwapTarget::Dye(slot) => std::mem::swap(item, &mut self.equip.dye[slot]),
            SwapTarget::Misc(slot) => {
                // `EquipSwap` (`:3640-3650`): the source's favourite clears.
                std::mem::swap(item, &mut self.equip.misc_equips[slot]);
            }
        }
        if let Some(idx) = ctx0_index {
            self.favorite_slots.retain(|&f| f != idx);
        }
        // `Main.EquipPageSelected = 0/2` (`:3296,3306,3316,3326,3336,3344,3354`):
        // jump to the page the item just landed on.
        self.equip_page_selected = match target {
            SwapTarget::Misc(_) => crate::ui::inventory::EQUIP_PAGE_MISC,
            _ => 0,
        };
        self.audio.play("Grab"); // `SoundEngine.PlaySound(7)` on every arm
        true
    }

    /// `ItemSlot.SwapVanityEquip` (`ItemSlot.cs:1867-1899`): swap the vanity
    /// row `slot` with the real row `slot - 10`, both favourites cleared
    /// (`SwapClearFavorite`, `:1878`). `Main.npcShop > 0` stands down (`:1869`).
    fn vanity_swap(&mut self, slot: usize, context: i32) {
        if self.shop_npc.is_some() {
            return; // `:1869-1872`
        }
        let Some(real) = slot.checked_sub(10) else {
            return;
        };
        let (a, b) = (self.equip.armor[slot], self.equip.armor[real]);
        if !crate::app::equip_swap::vanity_swap_allowed(&a, &b, context, slot, &self.equip.armor)
        {
            return; // `:1874-1877`
        }
        self.equip.armor.swap(slot, real);
        self.audio.play("Grab"); // `:1879` — `SoundEngine.PlaySound(7)`
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

    /// `Main.DrawMap`'s fullscreen ping (`Main.cs:55486-55495`): a second
    /// press within 0.5 s and 2 px of the previous one pins the tile under the
    /// pointer, converted back through the map's scale/offset
    /// (`Main.cs:55491`: `(MouseScreen - (num - 10*num5, num2 - 10*num5)) / num5`).
    ///
    /// The port's fullscreen branch is the `map.open` draw
    /// (`app/frame.rs`), so this runs only while it is up; the press edge is
    /// the frame's `ui_left_press`.
    pub(crate) fn handle_fullmap_ping(&mut self) {
        if !self.ui_left_press || self.menu.is_some() || self.front.is_some() {
            return;
        }
        let Some((map_pos, map_scale)) = self.map.as_ref().and_then(|m| {
            m.open.then(|| (m.pos, m.scale))
        }) else {
            return;
        };
        if !self.settings.map_enabled {
            return;
        }
        // `(Main.screenPosition + screenWidth/2)/16` is the map origin's tile
        // (`Main.cs:55420-55428`); `num = -pos*scale + screen/2` becomes the
        // `num - 10*scale` inset the ping conversion subtracts.
        let (sw, sh) = self.screen_size();
        let screen = Vec2::new(sw, sh);
        let origin = crate::map::map_origin(map_pos, map_scale, screen);
        let now = self.anim_time as f64;
        let dx = self.mouse_pos.x - self.last_ping_pos.x;
        let dy = self.mouse_pos.y - self.last_ping_pos.y;
        if now - self.last_ping_time < 0.5 && (dx * dx + dy * dy) < 4.0 {
            let tile = [
                (self.mouse_pos.x - origin[0]) / map_scale,
                (self.mouse_pos.y - origin[1]) / map_scale,
            ];
            if let Some(m) = self.map.as_mut() {
                m.trigger_ping(tile);
            }
        }
        self.last_ping_time = now;
        self.last_ping_pos = self.mouse_pos;
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
        // `Main.HoverOverCraftingItemButton` runs every held frame
        // (`Main.cs:42252-42305`): the click edge selects the recipe, the
        // held frames craft through the stack-split pacing.
        if !self.crafting_open {
            return false;
        }
        // `:42301-42305`: the buttons releasing reset the focus click's
        // lock and the held-repeat's recipe pin — `flag2 = Main.mouseLeft
        // || Main.mouseRight` (`:42255`), so EITHER held button keeps them.
        let held = self.mine_down || self.place_down;
        if !held {
            self.prevent_craft_click = false;
            self.current_recipe_being_crafted = -1;
            return false;
        }
        // `flag = (mouseLeftRelease && mouseLeft) || (mouseRightRelease &&
        // mouseRight)` (`:42254`): either button's press edge selects.
        let press_edge = self.ui_left_press || self.ui_right_press;
        if !press_edge && self.stack_split > 1 {
            return true; // the held frame between repeats
        }
        let (sw, sh) = self.screen_size();
        let screen = Vec2::new(sw, sh);
        // The filter strip's tabs (`UICreativeItemsInfiniteFilteringOptions.cs:80-100`):
        // a click toggles the tab, exclusive across the strip, and queues the
        // MenuTick the hover cue plays (`:75-78`). `OnClickingOption` →
        // `ResetRecipes` (`NewCraftingUI.cs:391`) is the shown list itself,
        // which re-derives every frame, so nothing is cached here.
        if self.ui_left_press {
            if let Some(i) = ui::crafting::filter_hit(screen, self.mouse_pos) {
                self.craft_filters.click(i);
                self.audio.play("Menu_Tick"); // `SoundEngine.PlaySound(12)`
                self.ui_left_press = false;
                return true;
            }
        }
        // The toggles before the grid: the nearby-chests toggle flips the
        // setting (`CraftingUI.cs:336-355`), and a consumed press does not
        // reach the recipe cells. The toggle reads `Main.mouseLeft &&
        // Main.mouseLeftRelease` (`:346`) — LEFT only, unlike the recipe
        // cells' either-button arms.
        if self.mine_down && ui::crafting::nearby_toggle_hit(screen, self.mouse_pos) {
            self.settings.craft_from_nearby_chests = !self.settings.craft_from_nearby_chests;
            // `NewCraftingUI.RefreshGrid()` (`CraftingUI.cs:349`): the shown
            // list re-derives every frame in the port, so the refresh is a
            // no-op here (the draw rebuilds `craft_raw_list`).
            self.audio.play("Menu_Tick"); // `SoundEngine.PlaySound(12)` (`:350`)
            let _ = self.settings.save(); // `Main.SaveSettings()` (`:351`)
            self.ui_left_press = false;
            return true;
        }
        let Some(player) = self.player.as_ref() else {
            return false;
        };
        // `Player.tileRangeX/Y` + `TileReachCheckSettings.Simple` (the C#'s
        // reach, `Player.cs:18815-18826`/`:13040-13044`/`:14955-14958`/
        // `:35671`): the Extendo Grip / the Architect's Gizmo Pack, the held
        // Toolbox, the Journey mode's Far Placement Range power and the
        // artisan loaf.
        let reach = craft::tile_reach(
            player.equipped[..10].contains(&2215) || player.equipped[..10].contains(&3061),
            self.mouse_item.kind == 1923,
            self.creative_powers.far_placement_range(),
            player.ate_artisan_bread,
        );
        let adj = craft::scan_adjacent(&self.world, player, reach);
        let zones = craft::Zones {
            snow: self.scene_metrics.zone_snow,
            graveyard: self.scene_metrics.zone_graveyard,
        };
        // `Recipe.CanCraft`'s two gates (`Recipe.cs:349-356`).
        let gates = craft::Gates {
            unlocked_biome_torches: player.unlocked_biome_torches,
            // `SpecialSeedFeatures.Mechdusa` (`SpecialSeedFeatures.cs:119-127`)
            // — `Main.remixWorld && Main.getGoodWorld`, i.e. the Remix option
            // and the "for the worthy" option (`WorldGen.cs:11118-11121`).
            mechdusa: self.world.seeds.contains(tr_core::seeds::SeedFlags::REMIX)
                && self
                    .world
                    .seeds
                    .contains(tr_core::seeds::SeedFlags::FOR_THE_WORTHY),
        };
        let in_range = craft::chests_in_range(
            &self.world,
            player,
            &self.world_chests,
            self.settings.craft_from_nearby_chests,
        );
        // `:42250`'s `Main.availableRecipe[recipeIndex]` indexes the RAW list, so
        // the grid narrows only what is shown (`NewCraftingUI.cs:596-611`).
        let raw_available = craft::available_with_gates(
            &self.inventory,
            &self.equip,
            &self.world_chests,
            &in_range,
            &adj,
            &zones,
            self.craft_filter.as_ref(),
            None,
            &gates,
        );
        let available = self.craft_filters.shown(raw_available.clone());
        let Some(idx) = ui::crafting::hit_test(
            screen,
            &self.lookups,
            self.mouse_pos,
            &available,
            self.craft_scroll,
        )
        else {
            return false;
        };
        let recipe = &craft::RECIPES[idx];
        // `HandleCraftSlot` (`NewCraftingUI.cs:499-528`): the click on an
        // unfocused cell selects it — `_selectedRecipeIndex = entry.index`
        // (`:507`) and `Main.focusRecipe = entry.availableIndex` (`:510`),
        // the RAW `availableRecipe` position (`UpdateContents` `:610`), NOT
        // the recipe-table index (the W4-f23 index-space fix: with a filter
        // active the shown position and the raw position diverge).
        let available_index =
            ui::crafting_grid::available_position(&raw_available, idx);
        // `Main.cs:42256-42262` + `:42295-42299`: the first click on an
        // unfocused row selects it (`stackSplit = 15`, the click-consumed
        // lock, the MenuTick sound); the craft runs only on the focused row.
        // `HandleCraftSlot`'s select arm takes EITHER button
        // (`NewCraftingUI.cs:505`: `(mouseLeft && mouseLeftRelease) ||
        // (mouseRight && mouseRightRelease)`), and so does
        // `HoverOverCraftingItemButton`'s (`:42254`'s `flag`).
        if self.focus_recipe != available_index {
            if press_edge {
                self.focus_recipe = available_index;
                self.stack_split = 15;
                self.prevent_craft_click = true;
                self.audio.play("Menu_Tick"); // `SoundEngine.PlaySound(12)`
                self.ui_left_press = false;
                self.ui_right_press = false;
            }
            return true;
        }
        // `:42266-42290`: the focused row crafts. `flag4` blocks the
        // held-repeat when a DIFFERENT recipe is being crafted or the
        // product is non-stacking; `TryAllowingToCraftRecipe`'s
        // mouse-blocking fetch (`:42278`) has no port counterpart (the
        // mouse item never blocks the craft here). The
        // `UsingOrReusingItem`/`IsStackingItems` gates of `flag3`
        // (`Player.cs:23098`) are unmodelled item-use state.
        let flag3 = !self.prevent_craft_click;
        let flag4 = (self.current_recipe_being_crafted != -1
            && self.current_recipe_being_crafted != idx as i32)
            | (self.current_recipe_being_crafted != -1
                && crate::registry::max_stack(recipe.create.0) == 1);
        let mut num = (self.super_fast_stack + 1).max(1) as u32;
        if shift_in_use(self) {
            num *= 10; // `Main.cs:42272-42274`: the Shift x10
        }
        if self.stack_split > 1 {
            return true; // `:42280`: the held-repeat pacing
        }
        if !flag3 || flag4 {
            return true;
        }
        // `try_craft_qty` crafts as many of `num` as the ingredients allow
        // (the request loop's net effect); an empty result = the partial-set
        // row's no-op (`Main.cs:42250` hover only). Each craft grants its own
        // `CreateResult` — its own kind (after `OnCreated`'s swap) and its own
        // rolled prefix (`CraftingRequests.cs:139`).
        let crafted = craft::try_craft_qty(
            &mut self.inventory,
            &mut self.equip,
            &mut self.world_chests,
            &in_range,
            recipe,
            &adj,
            &zones,
            &gates,
            // The craft rolls ride the App's craft stream (S1-L-11e
            // territory); the crafted item's `Item.Prefix(-1)` roll
            // (`CraftingRequests.cs:139`) draws the bare `Main.rand` — the
            // crafting interaction runs in `UpdateInventory` outside every
            // `SwapRandom` block (`Main.cs` ~17180 vs `DoUpdateInWorld`
            // :17226) — so the port stands it in with `NamedRngs::global()`,
            // the stream reforge and char-creation randomize also draw
            // (S1-A-11's bare-`Main.rand` stand-in).
            &mut self.craft_rng,
            self.rngs.global(),
            num,
        );
        // `CraftingRequests.CraftItem`'s tail (`CraftingRequests.cs:119`):
        // `CraftingEffects.OnCraft(recipe, quickCraft)` runs AFTER the craft
        // loop, UNCONDITIONALLY — even when the loop crafted nothing (the
        // `i <= 0` first-iteration arm, `:102`, always runs once). `OnCraft`
        // → `SpawnEffects_BeforeGrantingItem` → `SoundEngine.PlaySound(7)`
        // (`CraftingEffects.cs:16-24`, `:49-52`): the craft-attempt cue, NOT
        // a per-item grant sound. The port played it only inside
        // `!crafted.is_empty()` (the grant), dropping the cue on a
        // focused-row attempt the ingredients no longer allow — the
        // g23c-menu-union co-lane census found the gap (W4-f23 closeout 3).
        // The cue is button-agnostic (`:42276`'s `flag2` feeds both arms).
        self.audio.play("Grab"); // `SoundEngine.PlaySound(7)` (`CraftingEffects.cs:51`)
        if !crafted.is_empty() {
            // The C# hands the result to the cursor (`Main.CraftItem_GrantItem`,
            // `Main.cs:42607-42612`); the port grants it into the inventory
            // directly (the recorded deviation) — the prefix rides the slot,
            // so two crafted items with different prefixes take two slots
            // (`Item.CanStack`, `Item.cs:49865-49872`).
            for made in &crafted {
                if let Some(i) =
                    crate::item::slot_for_pickup(&mut self.inventory, made.kind, made.prefix)
                {
                    self.inventory[i].count =
                        (self.inventory[i].count + made.count).min(crate::registry::max_stack(made.kind));
                }
            }
            // `AchievementsHelper.NotifyItemCraft(Recipe)` (`:73-79`) — the
            // crafted item's id rides the event, so `BENCHED`'s workbench set
            // matches the crafted item and not any craft.
            let _ = self.achievements.notify_craft(crafted[0].kind);
            // `:42282-42285`: the first craft pins the held-repeat's recipe
            // and refreshes the split cooldown (`ItemSlot.cs:1774-1784`).
            if self.current_recipe_being_crafted == -1 {
                self.current_recipe_being_crafted = idx as i32;
            }
            self.stack_split = if self.stack_split == 0 {
                30
            } else {
                self.stack_delay
            };
        }
        self.ui_left_press = false;
        // The right press edge is consumed by the craft the same way
        // (`:42254`'s `flag` feeds both arms); the HELD right button keeps
        // crafting through `flag2` (`:42255`) like the held left.
        self.ui_right_press = false;
        true
    }

    /// The craft grid's live shown list — the same inputs
    /// `handle_crafting_clicks` and the draw path feed
    /// (`Recipe.UpdateRecipeList` + `NewCraftingUI.UpdateContents`).
    fn craft_shown_list(&self) -> Vec<usize> {
        let Some(player) = self.player.as_ref() else {
            return Vec::new();
        };
        let reach = craft::tile_reach(
            player.equipped[..10].contains(&2215) || player.equipped[..10].contains(&3061),
            self.mouse_item.kind == 1923,
            self.creative_powers.far_placement_range(),
            player.ate_artisan_bread,
        );
        let adj = craft::scan_adjacent(&self.world, player, reach);
        let zones = craft::Zones {
            snow: self.scene_metrics.zone_snow,
            graveyard: self.scene_metrics.zone_graveyard,
        };
        let gates = craft::Gates {
            unlocked_biome_torches: player.unlocked_biome_torches,
            mechdusa: self.world.seeds.contains(tr_core::seeds::SeedFlags::REMIX)
                && self
                    .world
                    .seeds
                    .contains(tr_core::seeds::SeedFlags::FOR_THE_WORTHY),
        };
        let in_range = craft::chests_in_range(
            &self.world,
            player,
            &self.world_chests,
            self.settings.craft_from_nearby_chests,
        );
        let raw = craft::available_with_gates(
            &self.inventory,
            &self.equip,
            &self.world_chests,
            &in_range,
            &adj,
            &zones,
            self.craft_filter.as_ref(),
            None,
            &gates,
        );
        self.craft_filters.shown(raw)
    }

    /// `UIList.ScrollWheel` (`UIList.cs:107-114`) for the craft grid:
    /// `UserInterface.Update` hands the notch to the element under the
    /// pointer (`:219-221`) — the list rect or the attached bar — and
    /// zeroes the accumulator whatever the hit (`Main.cs:17453-17456`:
    /// `NewCraftingUI.Visible` sets `flag = false`, so the classic list's
    /// scroll never also runs). Returns whether the panel consumed the
    /// notch (the hotbar arm must not see it).
    pub(crate) fn craft_wheel(&mut self, delta: i32) -> bool {
        if !self.crafting_open {
            return false;
        }
        let (sw, sh) = self.screen_size();
        let l = ui::crafting::layout(
            Vec2::new(sw, sh),
            ui::crafting::title_width_for(&self.lookups),
            false,
        );
        let n = self.craft_shown_list().len();
        if ui::crafting::wheel_target(&l, n, self.mouse_pos) {
            self.craft_scroll = ui::crafting::scroll_wheel(self.craft_scroll, delta, &l, n);
        }
        true
    }

    /// `UIScrollbar.LeftMouseDown` (`UIScrollbar.cs:126-143`) for the craft
    /// grid's bar: the handle grab latches `_dragYOffset`, a track press
    /// jumps the view. Returns whether the bar took the press — the same
    /// press must not reach the recipe cells (`evt.Target == this`).
    pub(crate) fn craft_scrollbar_press(&mut self) -> bool {
        if !self.crafting_open || !self.ui_left_press {
            return false;
        }
        let (sw, sh) = self.screen_size();
        let l = ui::crafting::layout(
            Vec2::new(sw, sh),
            ui::crafting::title_width_for(&self.lookups),
            false,
        );
        let n = self.craft_shown_list().len();
        let Some((scroll, drag)) =
            ui::crafting::scrollbar_press(&l, n, self.craft_scroll, self.mouse_pos)
        else {
            return false;
        };
        self.craft_scroll = scroll;
        self.craft_scroll_drag = drag;
        self.ui_left_press = false;
        true
    }

    /// `UIScrollbar.LeftMouseUp` (`UIScrollbar.cs:145-149`): the drag latch
    /// clears on the release.
    pub(crate) fn craft_scrollbar_release(&mut self) {
        self.craft_scroll_drag = None;
    }

    /// The town-chatter panel's clicks (`NPCChatPanel.DrawButtons`,
    /// `NPCChatPanel.cs:88-162`): the hover edge plays sound 12
    /// (`:157-160`) and the Close button's press (`:150-154`) runs
    /// `CloseChat.Interact` → `Main.CloseNPCChatOrSign`
    /// (`NPCInteractions.cs:283-286`, `Main.cs:40680-40692`) — the talk
    /// latch clears, `npcChatText` empties, sound 11 plays and
    /// `NPCChatPanel.Close` resets `_lastHovered` (`:265-269`). The other
    /// interaction buttons (shop/happiness/housing/…) are deferred —
    /// docs/ANALYSIS.md "Town chatter UI (S1-E-10b)".
    pub(crate) fn handle_npc_chat_clicks(&mut self) {
        if self.talk_npc.is_none() {
            // `DrawButtons` only runs while the panel draws — `_lastHovered`
            // keeps its value across a close (the next open's hover-change
            // edge, `:157-160`, reads it).
            return;
        }
        let (sw, sh) = self.screen_size();
        let screen = Vec2::new(sw, sh);
        let rect = ui::npc_chat::close_button_rect(&self.lookups, screen, &self.npc_chat_text);
        let hovered = if ui::in_rect(self.mouse_pos, rect) { 0 } else { -1 };
        // `:157-160` — the hover-change tick (no gamepad in the port, so
        // the `!UsingGamepad || num2 != -1` gate always passes).
        if self.npc_chat_last_hovered != hovered {
            crate::chest::play_menu_sound(self, 12);
        }
        self.npc_chat_last_hovered = hovered;
        if hovered == 0 && self.ui_left_press {
            self.ui_left_press = false; // `:152` — `Main.mouseLeftRelease = false`
            // `Main.CloseNPCChatOrSign` (`Main.cs:40680-40692`): the sign
            // half is unported; `SetTalkNPC(-1)` + the text clear + sound
            // 11 + the panel's `Close` reset.
            self.talk_npc = None;
            self.npc_chat_text.clear();
            crate::chest::play_menu_sound(self, 11);
            self.npc_chat_last_hovered = -1;
        }
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
        } else if self.inventory_open && self.ingame_options.button_layout.hit(self.mouse_pos) {
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
        // The wheel: a pick fires `MakeLocalPlayerEmote` (`EmoteBubble.cs:278-292`)
        // and closes (`EmoteButton.LeftClick`, `:90-95`); the Back panel just
        // closes (`UIEmotesMenu.GoBackClick`, `:305-309`).
        if self.emotes.menu_open {
            let (sw, sh) = self.screen_size();
            let screen = Vec2::new(sw, sh);
            if let Some(id) = ui::emotes::card_at(screen, self.mouse_pos, self.emotes.scroll) {
                // `MakeLocalPlayerEmote`'s dead gate (`:280`):
                // `Main.netMode == 2 || !Main.LocalPlayer.dead` — the port's
                // single-player arm silences a dead player.
                let dead = self.player.as_ref().is_some_and(|p| p.dead);
                if !dead {
                    self.emotes.new_bubble(id);
                }
                // `IngameFancyUI.Close` (`:162-188`): the emotes arm keeps
                // the inventory closed (`flag` stays false for `UIEmotesMenu`).
                self.emotes.close();
                self.ui_left_press = false;
                return;
            }
            // The wheel's OWN Back (`UIEmotesMenu.cs:78-87`, `Top(-45)`),
            // not the bestiary's panel it used to borrow (`Top(-25)` on a
            // different container) — the same BUG-18 mis-box the draw side
            // fixed and the click side kept.
            let panel = ui::emotes::panel_rect(screen, true);
            if ui::emotes::back_hit(&panel, self.mouse_pos) {
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

    /// `IngameFancyUI.OpenAchievementsAndGoto` (`UI/IngameFancyUI.cs`):
    /// `OpenAchievements()` (the in-game arm is `IngameFancyUI.OpenUIState`)
    /// then `UIAchievementsMenu.GotoAchievement` — `UIList.Goto` sets
    /// `ViewPosition` to the row's `Top.Pixels`. The port opens the overlay
    /// with every category on (`OnActivate`'s `SetState(true)`) and scrolls
    /// to the row's index in the visible list.
    pub(crate) fn open_achievements_and_goto(&mut self, name: &str) {
        self.achievements_open = true;
        self.achievements_categories = [true; 4];
        let rows = ui::achievements::visible_rows(&self.achievements, &self.achievements_categories);
        let idx = rows
            .iter()
            .position(|&i| self.achievements.list[i].name == name)
            .unwrap_or(0);
        let (sw, sh) = self.screen_size();
        let panel = ui::achievements::panel_rect(Vec2::new(sw, sh), self.menu.is_some());
        let list = ui::achievements::list_rect(&panel);
        // `Goto`'s `ViewPosition = _items[i].Top.Pixels` — the row's top in
        // the inner list (`row_rect`'s lattice), clamped to the scroll range.
        let top = ui::achievements::row_rect(&list, idx)[1] - list[1];
        let max = ui::achievements::max_scroll(rows.len(), &list);
        self.achievements_scroll = top.clamp(0.0, max);
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
        self.achievements_scroll =
            (self.achievements_scroll - delta * (ui::achievements::ROW_H + 5.0)).clamp(0.0, max);
    }

    /// `DrawInterface_38_MouseCarriedObject`'s click tail
    /// (`Main.cs:45217-45246`): with a housing head on the cursor, the right
    /// release drops it (sound 12) and the left release runs the query or
    /// the relocation at the clicked world tile. The head suppresses the
    /// slot clicks (`Player.mouseInterface = true`, `:45221`).
    fn handle_carried_housing_head(&mut self, left: bool, right_edge: bool) {
        // `:45205-45211`: a carried NPC that died or changed type falls back
        // to the housing query before the click resolves.
        if self.mouse_npc_index >= 0 {
            let live = self
                .npcs
                .get(self.mouse_npc_index as usize)
                .is_some_and(|n| n.kind.net_id() == self.mouse_npc_type);
            if !live {
                self.mouse_npc_index = -1;
                self.mouse_npc_type = 0;
            }
        }
        // `:45223-45227`: the right release drops the head, sound 12.
        if right_edge {
            self.audio.play("Menu_Tick"); // `SoundEngine.PlaySound(12)`
            self.mouse_npc_index = -1;
            self.mouse_npc_type = -1;
        }
        // `:45228-45231`: only the left release acts.
        if !left {
            return;
        }
        // `:45232-45236`: the click's world tile — `Main.MouseScreen` through
        // the zoom, plus the screen position, over 16.
        let (sw, sh) = self.screen_size();
        let world_x = self.camera.x + (self.mouse_pos.x - sw * 0.5) / self.zoom;
        let world_y = self.camera.y + (self.mouse_pos.y - sh * 0.5) / self.zoom;
        let x = (world_x / tr_core::TILE_PX) as i32;
        let y = (world_y / tr_core::TILE_PX) as i32;
        // `:45237-45241`: the housing query (`mouseNPCType == 0`).
        if self.mouse_npc_type == 0 {
            let prioritized = self
                .town_arrival
                .prioritized
                .map_or(0, |k| k.net_id());
            crate::town::perform_housing_check(
                &self.world,
                &self.npcs,
                x,
                y,
                prioritized,
                false,
                &mut self.chat,
            );
            return;
        }
        // `:45242-45246`: the relocation of the carried NPC.
        let n = self.mouse_npc_index;
        if n >= 0 {
            let mut rooms = std::mem::take(&mut self.town_rooms);
            let moved = crate::town::try_moving_npc(
                &self.world,
                &mut self.npcs,
                &mut rooms,
                x,
                y,
                n as usize,
                false,
                &mut self.chat,
            );
            self.town_rooms = rooms;
            if moved {
                // `Main.TryMovingNPC:45176-45178`: clear the head, sound 12.
                self.mouse_npc_index = -1;
                self.mouse_npc_type = -1;
                self.audio.play("Menu_Tick"); // `SoundEngine.PlaySound(12)`
                // `SpawnTownNPC`'s relocation arm fires NO_HOBO (progression
                // 8, `WorldGen.cs:5460`).
                let _ = self.notify_progression_net(8);
                // `moveRoom:4850`'s transient prioritized write is consumed
                // and zeroed by the same `SpawnTownNPC` (`:5461`).
                self.town_arrival.prioritized = None;
            }
        }
    }

    pub(crate) fn handle_inventory_clicks(&mut self) {
        let left = self.ui_left_press;
        let right_edge = self.ui_right_press;
        // `ItemSlot.RightClick` reads `Main.mouseRight` (`:1654`), not the
        // press edge: the held state drives the paced one-per-transfer pickup
        // (`:1700-1713`). The port's own reversed place/merge arms stay on the
        // edge (`right_edge`) so a held button cannot spam them.
        let right = right_edge || self.place_down;
        self.ui_left_press = false;
        self.ui_right_press = false;
        if !left && !right {
            return;
        }
        // `DrawNPCHousesInUI`'s cell click (`Main.cs:42058-42070`): the
        // housing page's heads pick up onto the cursor — the "?" cell arms
        // the housing query, an NPC head arms its relocation. Runs inside
        // `DrawInventory`, i.e. before the carried-head pass below, and
        // consumes the press (`mouseLeftRelease = false`, `:42069`).
        if left
            && self.inventory_open
            && self.equip_page == 1
            && (self.mouse_item.kind == 0 || self.mouse_item.count == 0)
        {
            let (sw, sh) = self.screen_size();
            let screen = Vec2::new(sw, sh);
            let base_y = ui::inventory::equip_page_base_y(screen.y, self.main_m_h());
            let cells = ui::housing::layout(
                &ui::housing::heads_for(&self.npcs),
                [screen.x, screen.y],
                base_y,
                0.85,
            );
            if let Some((head, npc)) =
                ui::housing::hover(&cells, [self.mouse_pos.x, self.mouse_pos.y], 0.85)
            {
                self.audio.play("Menu_Tick"); // `SoundEngine.PlaySound(12)` (`:42060`)
                if head == ui::housing::HOUSING_QUERY {
                    // `SetMouseNPC_ToHousingQuery()` (`:42063`).
                    self.mouse_npc_index = -1;
                    self.mouse_npc_type = 0;
                } else if let Some(i) = npc {
                    // `SetMouseNPC(num7, Main.npc[num7].type)` (`:42067`).
                    self.mouse_npc_index = i as i32;
                    self.mouse_npc_type = self.npcs[i].kind.net_id();
                }
                return;
            }
        }
        // `DrawInterface_38_MouseCarriedObject` (`Main.cs:45182-45247`): the
        // carried housing head. A carried item drops it (`:45185-45187`);
        // otherwise the head rides the cursor and the world click runs the
        // housing query / relocation. Checked before every inventory arm —
        // the C# runs it after `DrawInventory`, so a carried head consumes
        // the click instead of the slots (`Player.mouseInterface`, `:45221`).
        if self.mouse_npc_type > -1 {
            if self.mouse_item.count > 0 {
                // `SetMouseNPC(-1, -1)` (`:45187`).
                self.mouse_npc_index = -1;
                self.mouse_npc_type = -1;
            } else {
                self.handle_carried_housing_head(left, right_edge);
                return;
            }
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
            // `Main.cs:41426-41460`: the accessory "hide the visuals" ticks beside
            // the accessory column. Checked before the slots because the C# tick
            // box sits over the slot's right edge, and gated on
            // `Main.mouseItem.IsAir` exactly like the draw.
            // The ticks live only on the equipment page: the misc page
            // (`EquipPage == 2`) and the housing page (`== 1`) draw no accessory
            // column (`Main.cs:41172`, `:41368`).
            if self.equip_page == 0 {
                let tick = self
                    .lookups
                    .ui_tick_on
                    .as_ref()
                    .map(|s| (s.width as f32, s.height as f32))
                    .unwrap_or((16.0, 16.0));
                let base_y = ui::inventory::equip_page_base_y(screen.y, m_h);
                let air = self.mouse_item.kind == 0 || self.mouse_item.count == 0;
                if let Some(i) =
                    ui::inventory::accessory_tick_click(screen, base_y, self.mouse_pos, tick, air)
                {
                    if let Some(p) = self.player.as_mut() {
                        p.hide_visible_accessory[i] = !p.hide_visible_accessory[i];
                    }
                    // `SoundEngine.PlaySound(12)` (`Main.cs:41438`).
                    self.audio.play("Menu_Tick");
                    // `SendData(4)` in MP — the player-info packet is not ported
                    // for this field (`net.rs`'s `hide_visible_accessory` is a
                    // decode-only `u16`), so the flip stays local (ANALYSIS).
                    return;
                }
            }
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
        // The reforge menu regions (`Main.cs:41639-41641` button, `:41673`
        // slot): the (50, 270) input slot and the (120, 310) button centre
        // (strict ±15). Neither rides `Main.mH` — the reforge block uses the
        // literal coordinates (`Main.cs:41581-41582`); a hit consumes the
        // press before the slot grid.
        if self.inventory_open && self.in_reforge_menu {
            let cell = self
                .lookups
                .ui_back
                .map(|b| b.width as f32)
                .unwrap_or(52.0);
            let (mx, my) = (self.mouse_pos.x, self.mouse_pos.y);
            let mouse = Vec2::new(mx, my);
            if left && ui::inventory::reforge_slot_hit(mouse, cell) {
                self.reforge_slot_click();
                return;
            }
            if left && ui::inventory::reforge_button_hit(mouse) {
                self.reforge_click();
                return;
            }
        }
        // The shop grid (`Main.cs:41799-41818`): the cells set
        // `mouseInterface` and run `ItemSlot.Handle(shop.item, 15, slot)`
        // for the hovered cell — the press is consumed even when the
        // stack-split gate transfers nothing this frame (the C#'s
        // `mouseInterface` suppresses the world use). The block draws
        // after the reforge inline block and before `ChestUI.Draw`
        // (`:41831`), so it takes the pointer ahead of the bank/trash/
        // equipment arms below (its (73, 258) rect overlaps the bank
        // panel's — the shop wins, exactly like the C#'s draw order).
        if let Some(slot) = self.shop_slot_at_mouse() {
            self.shop_slot_click(slot, left, right);
            return;
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
                        // S1-P-i43 — the transfer cues ride the bare-`Main.rand`
                        // stand-in (the click handler runs outside every
                        // SwapRandom block, the `quick_transfer_from_slot`
                        // precedent); snapshot it across the `p` borrow.
                        let mut rng = std::mem::replace(
                            self.rngs.global(),
                            tr_core::UnifiedRandom::default(),
                        );
                        match id {
                            // `ChestUI.LootAll`'s bank arm (:417-473) —
                            // `LootAllFromBank` is the default settings
                            // (`GetItemSettings.cs:55`): no vault arm, so
                            // `None` (the overflow stays in the bank).
                            0 => crate::bank::loot_all(
                                &mut p.banks[bi],
                                &mut self.inventory,
                                &mut self.equip.coins,
                                &mut self.equip.ammo,
                                None,
                                &mut self.audio,
                                &mut rng,
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
                                    &mut self.audio,
                                    &mut rng,
                                );
                                crate::bank::deposit_all(
                                    &mut p.banks[bi],
                                    &mut self.inventory,
                                    &fav,
                                    &mut self.audio,
                                    &mut rng,
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
                                    &mut self.audio,
                                    &mut rng,
                                );
                                crate::bank::quick_stack(
                                    &mut p.banks[bi],
                                    &mut self.inventory,
                                    &mut self.equip.coins,
                                    &fav,
                                    &mut self.audio,
                                    &mut rng,
                                );
                            }
                            _ => {
                                let _ = crate::bank::restock(
                                    &mut p.banks[bi],
                                    &mut self.inventory,
                                    &mut self.equip.ammo,
                                    &fav,
                                    &mut self.audio,
                                    &mut rng,
                                );
                            }
                        }
                        *self.rngs.global() = rng;
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
            if let Some(bank_idx) = (crate::bank::bank_for_chest(chest_open)).and_then(|_| {
                let m_h = (self.screen_size().1 as f32 - 600.0) / 2.0;
                ui::inventory::bank_slot_at(self.mouse_pos, m_h)
            }) {
                let bi = crate::bank::bank_for_chest(chest_open).unwrap();
                // Read the cell and the cursor up front so the `&mut self`
                // pickup helper never overlaps the player borrow; the cell is
                // written back once the branch settles.
                let mut target = self
                    .player
                    .as_ref()
                    .map(|p| p.banks[bi][bank_idx])
                    .unwrap_or(crate::item::EMPTY_SLOT);
                let mut held = self.mouse_item;
                // The `:1707` pickup gate: an air cursor, or the same
                // kind/prefix with cursor room.
                let can_take = held.count == 0
                    || (crate::bank::can_stack(&held, &target)
                        && held.count < crate::registry::max_stack(target.kind));
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
                    self.mouse_item = held;
                } else if can_take {
                    // `RightClick`'s pickup block (`ItemSlot.cs:1700-1713`):
                    // the paced one-per-transfer `PickupItemIntoMouse` split.
                    target = self.right_click_split(target);
                } else if right_edge {
                    // The port's press-edge place-one carry (research/20 §5)
                    // — not the C#'s pickup direction, kept on the edge so a
                    // held button cannot spam it.
                    if target.count == 0 {
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
                        self.mouse_item = held;
                    }
                }
                if let Some(p) = self.player.as_mut() {
                    p.banks[bi][bank_idx] = target;
                }
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
        if left
            && self.inventory_open
            && ui::inventory::trash_slot_at(self.mouse_pos, chest_or_shop)
        {
            self.trash_swap();
            if self.mouse_item.count > 0 || self.equip.trash.count > 0 {
                self.audio.play("Grab");
            }
            return;
        }
        // The equipment columns (`Main.cs:41288,41450,41491,41519`): armour,
        // vanity, dye and — on the misc page — the misc-equip/misc-dye pair.
        // `ItemSlot.Handle` runs for every cell `DrawInventory` draws, so the
        // press is resolved through the same `equip_cell_at` geometry the
        // hover uses.
        if self.inventory_open {
            let (sw, sh) = self.screen_size();
            let screen = Vec2::new(sw, sh);
            let m_h = self.main_m_h();
            let cell = self.lookups.ui_back.map_or(52.0, |b| b.width as f32);
            // The housing page owns its own cells — `equip_cell_at` answers
            // `None` there, so it falls through to the grid probe below.
            if let Some((ctx, idx)) =
                ui::inventory::equip_cell_at(self.mouse_pos, screen, m_h, self.equip_page, cell)
            {
                self.equip_slot_click(ctx, idx, left, right_edge);
                return;
            }
        }
        // The coin (ctx 1) and ammo (ctx 2) rows: `Main.DrawInventory` handles
        // every cell it draws (`Main.cs:41752-41782`), and the shared grid
        // probe below knows nothing about them (`slot_at_mouse` covers the
        // 10x5 grid and the closed hotbar).
        if let Some((ctx, k)) = self.coin_ammo_at_mouse() {
            self.coin_ammo_click(ctx, k, left, right_edge);
            return;
        }
        let Some(slot) = self.slot_at_mouse() else {
            return;
        };
        if !self.inventory_open {
            if left {
                // S1-P-i54 — `GUIHotbarDrawInner` (`Main.cs:43616-43623`):
                // the click latches `changeItem`, which `Player.Update`
                // feeds to `Select` (`Player.cs:25087-25091`) — buffered,
                // not immediate. The hover gate (`:43616`) refuses the
                // locked hotbar (`hbLocked` has no port state — the lock
                // icon is a recorded no-op, `ui/chrome.rs`), the torch
                // auto-select hold and an active channel; `Main.blockMouse`
                // (`:43620`) blocks the held button after a mid-use switch.
                let control_torch = self.keys.iter().any(|k| {
                    crate::input_map::is_bound(&self.keybinds, crate::input_map::Action::Torch, *k)
                });
                let channel = self.player.as_ref().is_some_and(|p| p.channel);
                if !control_torch && !channel && !self.block_mouse {
                    if self.selection.select(self.selected, slot, false) {
                        self.audio.play("Menu_Tick"); // `Select`, `:506`
                    }
                }
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
        if left && self.cursor_override == CURSOR_SELL {
            // `OverrideLeftClick`'s cursor-10 arm (`ItemSlot.cs:851-869`):
            // the Shift-click sell into the open shop.
            self.sell_slot_click(slot);
            return;
        }
        let mut held = self.mouse_item;
        let mut target = self.inventory[slot];
        // The busy gates (S1-P-i48): `LeftClick` refuses while
        // `itemAnimation != 0 || itemTime != 0` (`ItemSlot.cs:922-926`),
        // `RightClick` while `itemAnimation > 0` (`:1644-1647`).
        if (left && (self.swing > 0.0 || self.item_time > 0.0))
            || (!left && self.swing > 0.0)
        {
            return;
        }
        // The `:1707` pickup gate: an air cursor, or the same kind/prefix with
        // cursor room. This decides pickup vs the port's press-edge carry.
        let can_stack = crate::bank::can_stack(&held, &target);
        let can_take = held.count == 0 || (can_stack && held.count < registry::max_stack(target.kind));
        // `RightClick`'s `HasItemSwap` arm (`ItemSlot.cs:1668-1675`,
        // S1-P-i48): a single variant-swap kind (`ItemID.cs:1385`) cycles
        // in place on the press edge — BEFORE the `SwapEquip` ladder
        // (`:1685-1697`), so an Ammo Cycler right-click changes its mode
        // instead of equipping.
        if !left
            && (held.count == 0 || !can_stack)
            && target.count == 1
            && crate::sortdata::HAS_ITEM_SWAP.get(target.kind)
        {
            if right_edge {
                if let Some(new_kind) = crate::app::equip_swap::try_item_swap(&mut target) {
                    self.inventory[slot] = target;
                    // `AfterItemSwap` (`:1997-2009`).
                    self.audio
                        .play(crate::app::equip_swap::after_item_swap_cue(new_kind));
                    self.stack_split = 30; // `:2007`
                    self.ui_right_press = false; // `:2008`
                }
            }
            return;
        }
        // `RightClick`'s swap ladder (`ItemSlot.cs:1658,1685-1697`): an air
        // cursor or one that cannot stack with the cell enters the swap block
        // first, and a single wearable / dye jumps to its equipment row on the
        // press edge (`Main.mouseRightRelease`) instead of being picked up.
        if !left
            && (held.count == 0 || !can_stack)
            && target.count == 1
            && target.kind > 0
            && (crate::app::equip_swap::can_be_equipped(&target)
                || registry::get(target.kind).is_some_and(|d| d.dye > 0))
        {
            if right_edge {
                let mut item = target;
                if self.swap_equip_click(Some(slot), &mut item) {
                    self.inventory[slot] = item;
                }
            }
            return;
        }
        if left {
            // The action-0 movement shared with the chest grid
            // (`left_click_movement`); the C# plays 7 when an item sits on
            // either side after the move (`ItemSlot.cs:1004-1011`).
            if left_click_movement(&mut held, &mut target) {
                self.audio.play("Grab");
            }
            self.mouse_item = held;
            self.inventory[slot] = target;
            // `CoinSlot.ForceSlotState(slot, context, inv[slot])` — the
            // case-0 tail (`ItemSlot.cs:1016`, S1-H-9).
            self.coin_slot.force_slot_state(
                slot as i32,
                0,
                self.inventory[slot].kind,
                self.inventory[slot].count,
            );
        } else if can_take {
            // `RightClick`'s pickup block (`ItemSlot.cs:1700-1713`): the paced
            // one-per-transfer `PickupItemIntoMouse` split (held-repeat
            // included).
            self.inventory[slot] = self.right_click_split(target);
            self.coin_slot.force_slot_state(
                slot as i32,
                0,
                self.inventory[slot].kind,
                self.inventory[slot].count,
            );
        } else if right_edge {
            // The port's press-edge place-one / merge-one carry (research/20
            // §5, the flat-`MAX_STACK` deviation) — not the C#'s pickup
            // direction, kept on the edge so a held button cannot spam it.
            if target.count == 0 {
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
    }

    /// `ItemSlot.HandleShopSlot` (`ItemSlot.cs:2035-2096`) — one shop-cell
    /// press (context 15, `Main.cs:41814`'s `ItemSlot.Handle`): the left
    /// click buys through `PickItemMovementAction`'s action 3
    /// (`:1566-1574` — an air cursor over a stocked row, or a same-kind
    /// cursor with room), the right click buys paced
    /// (`rightClickIsValid`, `:1648-1651`), and a non-air cursor over an
    /// EMPTY cell sells the whole held stack (action 4, `:1575-1578` →
    /// `LeftClick`'s case 4, `:1222-1241`). The chat-share cursor stands
    /// down (`:2037-2040`, unported).
    fn shop_slot_click(&mut self, slot: usize, left: bool, right: bool) {
        // The shop chest carries 40 slots (`Chest.DefaultMaxItems`,
        // `Chest.cs:64`); the port's dense Vec ends at the last stocked
        // row, so a cell past the end reads as an AIR row (the sell arm
        // targets exactly those).
        let entry = self.shop_stock.get(slot).copied().unwrap_or(crate::shop::ShopEntry {
            item: 0,
            when: crate::shop::StockWhen::Always,
            shop_custom_price: None,
            buy_once: false,
            is_a_shop_item: false,
            prefix: 0,
            stack: 0,
        });
        if slot >= 40 {
            return; // past the chest's slot window
        }
        let held = self.mouse_item;
        let action = pick_item_movement_action(15, &held, slot, &shop_row_slot(&entry), None);
        if left && action == 3 {
            self.handle_shop_buy(slot);
        } else if left && action == 4 {
            // `LeftClick`'s case 4 (`ItemSlot.cs:1222-1241`): SellItem pays
            // out, AddItemToShop re-stocks the sold units, the cursor turns
            // to air; a value-0 item still enters the shop (sound 7). The
            // `stackSplit = 9999` tail (`:1239`) locks the split repeat.
            let item = self.mouse_item;
            let paid = self.sell_item(&item);
            if paid > 0 || crate::itemdata::ITEMS[item.kind as usize].value == 0 {
                crate::shop::add_item_to_shop(
                    &mut self.shop_stock,
                    &mut self.shop_sellback,
                    &item,
                );
                self.mouse_item = EMPTY_SLOT;
                // `SoundEngine.PlaySound(18)` on the sale, 7 on the giveaway.
                self.audio.play(if paid > 0 { "Coins" } else { "Grab" });
            }
            self.stack_split = 9999; // `:1239`
        } else if right {
            // `RightClick`'s ctx-15 arm (`:1648-1651`) — the paced buy.
            self.handle_shop_buy(slot);
        }
    }

    /// `ItemSlot.HandleShopSlot`'s buy loop (`ItemSlot.cs:2042-2095`): the
    /// `stackSplit <= 1` gate, the cursor stackability gate (`:2043` —
    /// `Item.CanStack` or an air cursor), then `superFastStack + 1` units
    /// (× the bulk amount on a Shift-held buyOnce/isAShopItem row,
    /// `:2047-2051`). Each unit re-prices the row, charges `BuyItem`,
    /// pushes onto the cursor and refreshes the split cooldown
    /// (`:2089`); the first successful unit plays 18 on a valued row,
    /// else 7 (`:2064-2073`).
    fn handle_shop_buy(&mut self, slot: usize) {
        if self.stack_split > 1 {
            return; // `:2043`
        }
        let Some(entry) = self.shop_stock.get(slot).copied() else {
            return;
        };
        if entry.item == 0 || entry.stack == 0 {
            return; // `:2043`'s `inv[slot].type <= 0`
        }
        let held = self.mouse_item;
        // `:2043` — `!Item.CanStack(mouseItem, inv[slot]) && mouseItem.type != 0`.
        if held.count > 0 && !crate::bank::can_stack(&held, &shop_row_slot(&entry)) {
            return;
        }
        let mut num2 = (self.super_fast_stack + 1) as u32; // `:2047`
        if crate::shop::can_bulk_buy(&entry, shift_in_use(self)) {
            num2 *= crate::shop::bulk_buy_amount(&entry); // `:2048-2051`
        }
        for i in 0..num2 {
            // `:2055-2058` — a full cursor skips the unit (the C#
            // `continue`s; the remaining units re-test the same room).
            if self.mouse_item.count > 0
                && self.mouse_item.count >= registry::max_stack(self.mouse_item.kind)
            {
                continue;
            }
            let value = crate::itemdata::ITEMS[entry.item as usize].value;
            if !self.buy_shop_item(slot) {
                continue; // `:2060-2063` — the purse gate / the depleted row
            }
            if i == 0 {
                // `:2064-2073` — sound 18 on a valued row, else 7.
                self.audio.play(if value > 0 { "Coins" } else { "Grab" });
            }
            // `RefreshStackSplitCooldown` (`:1774-1784`, `:2089`).
            self.stack_split = if self.stack_split == 0 {
                30
            } else {
                self.stack_delay
            };
        }
    }

    /// `ItemSlot.OverrideLeftClick`'s cursor-10 arm (`ItemSlot.cs:851-869`)
    /// — the Shift-click sell from an inventory slot: `SellItem` pays the
    /// stack out, `Chest.AddItemToShop` re-stocks it, the slot turns to
    /// air (sound 18); a value-0 item still enters the shop (sound 7).
    fn sell_slot_click(&mut self, slot: usize) {
        let item = self.inventory[slot];
        if item.count == 0 {
            return;
        }
        let paid = self.sell_item(&item);
        if paid > 0 || crate::itemdata::ITEMS[item.kind as usize].value == 0 {
            crate::shop::add_item_to_shop(&mut self.shop_stock, &mut self.shop_sellback, &item);
            self.inventory[slot] = EMPTY_SLOT;
            self.audio.play(if paid > 0 { "Coins" } else { "Grab" });
        }
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
            // The reforge menu's cursor-9 arm (`ItemSlot.cs:821-836`): the
            // stack splits one item into the empty reforge slot, a single
            // item swaps (`Utils.Swap`, sound 7).
            if self.in_reforge_menu {
                if let Some(slot) = self.slot_at_mouse() {
                    let item = self.inventory[slot];
                    if item.count > 1 {
                        if self.reforge_slot.count == 0 {
                            self.reforge_slot = ItemSlot {
                                count: 1,
                                ..item
                            };
                            self.inventory[slot].count -= 1;
                            self.audio.play("Grab"); // `SoundEngine.PlaySound(7)`
                        }
                    } else if item.count == 1 {
                        std::mem::swap(&mut self.inventory[slot], &mut self.reforge_slot);
                        self.audio.play("Grab");
                    }
                }
                self.ui_left_press = false;
                return true;
            }
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
        // panel-fidelity deviation). While a shop is open the shop grid owns
        // the (73, 258) rect (`Main.cs:41799-41818`), so the chest dispatch
        // stands down — the inventory handler's shop arm takes the pointer.
        if self.open_container() >= 0 && !self.crafting_open && self.shop_npc.is_none() {
            // `ChestUI.DrawButton`'s click dispatch (`ChestUI.cs:281-308`) —
            // the C# `DrawButtons` pass runs before `DrawSlots`, so the
            // buttons take the press before the slot grid sees it. IDs 0-3
            // are the M3-STORE-2 transfers, 4 Sort (`ItemSorting.SortChest`,
            // `:304-306`, S1-H-8), 5/6 the rename pair (ID 5 →
            // `RenameChest` :827-838, ID 6 → `RenameChestCancel` :863-869).
            // The port's
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
                        ui::chest::BUTTON_SORT => self.chest_sort_click(),
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
            // The dresser top-row storage open (`Player.cs:32864-32925`,
            // S1-P-i43): the same open-inventory right-click route.
            if chest::try_open_dresser_at_cursor(self, world_pos) {
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
            // `:607-609` — air returns before the key block; `:616-619` — the
            // canFavoriteAt gate (contexts 0/1/2, `ItemSlot.cs:386-389`).
            if let Some((ctx, k)) = self.coin_ammo_at_mouse() {
                let item = if ctx == 1 {
                    self.equip.coins[k]
                } else {
                    self.equip.ammo[k]
                };
                if item.count > 0 && chest::can_favorite_at(ctx) {
                    self.cursor_override = CURSOR_FAVORITE;
                    return;
                }
            }
            if let Some(slot) = self.slot_at_mouse() {
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
        // `Main.InReforgeMenu`'s ctx-0 arm (`ItemSlot.cs:666-672`): a
        // prefixable item lights the transfer-to-reforge cursor (9). The
        // port's reforge click routes through the same cursor id.
        if self.in_reforge_menu {
            if let Some(slot) = self.slot_at_mouse() {
                if slot < crate::item::INV
                    && self.inventory[slot].count > 0
                    && !self.is_favorite(slot)
                    && crate::prefix::can_have_prefixes(self.inventory[slot].kind)
                {
                    self.cursor_override = CURSOR_TO_CHEST;
                }
            }
            return;
        }
        // The ctx-3 arms are the world-chest panel's (`chest >= 0`) — banks
        // keep the STORE-1 panel and no cursor-8/9 shift route (the recorded
        // panel-fidelity deviation; `TryPlacingInChest` refuses banks too).
        // While a shop is open the shop grid owns the (73, 258) rect, so the
        // chest-cell probe stands down (the chest panel does not draw).
        if chest >= 0 && !self.crafting_open && self.shop_npc.is_none() {
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
        // quick-trash cursor — or the SELL cursor while a shop is open
        // (`GetSellOrTrash`, `:198-209`: `npcShop <= 0` → Trash, the coin
        // kinds 71-74 → null, everything else → Sell). The head guard
        // (`:631-634`): air or favorited slots answer null.
        if let Some(slot) = self.slot_at_mouse() {
            if slot < crate::item::INV
                && self.inventory[slot].count > 0
                && !self.is_favorite(slot)
            {
                if self.shop_npc.is_none() {
                    self.cursor_override = CURSOR_TRASH;
                } else if !is_coin_kind(self.inventory[slot].kind) {
                    self.cursor_override = CURSOR_SELL;
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
        // S1-P-i43 — the transfer cues ride the bare-`Main.rand` stand-in
        // (the click handler runs outside every SwapRandom block, the
        // `quick_transfer_from_slot` precedent); snapshot it across the
        // `wc`/`p` borrows.
        let mut rng = std::mem::replace(self.rngs.global(), tr_core::UnifiedRandom::default());
        match id {
            // `ChestUI.LootAll` (`:393-473`).
            0 => crate::bank::loot_all(
                &mut wc.slots,
                &mut self.inventory,
                &mut self.equip.coins,
                &mut self.equip.ammo,
                vault,
                &mut self.audio,
                &mut rng,
            ),
            // `ChestUI.DepositAll` (`:540-659`) — the MoveCoins pre-pass runs
            // for every container kind (`:543-562`).
            1 => {
                crate::bank::move_coins(
                    &mut wc.slots,
                    &mut self.inventory,
                    &mut self.equip.coins,
                    &fav,
                    &mut self.audio,
                    &mut rng,
                );
                crate::bank::deposit_all(
                    &mut wc.slots,
                    &mut self.inventory,
                    &fav,
                    &mut self.audio,
                    &mut rng,
                );
            }
            // `ChestUI.QuickStack` (`:661-825`) — NO MoveCoins for a world
            // chest (`:670-685`'s arms are the bank kinds only).
            2 => crate::bank::quick_stack(
                &mut wc.slots,
                &mut self.inventory,
                &mut self.equip.coins,
                &fav,
                &mut self.audio,
                &mut rng,
            ),
            // `ChestUI.Restock` (`:871-995`) — the single PlaySound(7) on a
            // moved item (`:991-994`), S1-P-i43's faithful-cue decision.
            _ => {
                let _ = crate::bank::restock(
                    &mut wc.slots,
                    &mut self.inventory,
                    &mut self.equip.ammo,
                    &fav,
                    &mut self.audio,
                    &mut rng,
                );
            }
        }
        *self.rngs.global() = rng;
    }

    /// `ChestUI.cs:304-306` → `ItemSorting.SortChest` (`ItemSorting.cs:1629-1653`)
    /// → `SortInventory(chest, withSync: true, withFeedback: true)` over the open
    /// world chest's slots. The damage-ranked layer order
    /// (`AddSortingPrioritiesBasedOnPlayerDamage`, `:1412-1425`) reads the local
    /// player's `melee/ranged/magic/minionDamage` multipliers; the net-sync half
    /// is a no-op outside multiplayer (`Main.netMode != 1`, `:1669`).
    fn chest_sort_click(&mut self) {
        let idx = self.open_container() as usize;
        // The open-container gate in `handle_chest_clicks` guarantees a live
        // store entry; the `get_mut` keeps the click path panic-free regardless.
        let Some(wc) = self.world_chests.get_mut(idx) else {
            return;
        };
        let mult = match self.player.as_ref() {
            Some(p) => [
                p.melee_damage,
                p.ranged_damage,
                p.magic_damage,
                p.minion_damage,
            ],
            None => [1.0; 4],
        };
        crate::item_sorting::sort_chest(&mut wc.slots, mult);
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
        // The busy gates (S1-P-i48): `LeftClick` `:922-926` /
        // `RightClick` `:1644-1647` — the same swing/itemTime refusals the
        // grid arm applies.
        if (left && (self.swing > 0.0 || self.item_time > 0.0))
            || (!left && self.swing > 0.0)
        {
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
            // `CoinSlot.ForceSlotState(slot, context, inv[slot])` — the
            // chest transfer's tail (`ItemSlot.cs:1119`, S1-H-9; ctx 3).
            self.coin_slot
                .force_slot_state(abs as i32, 3, target.kind, target.count);
        }
        if right_held {
            // `RightClick`'s ctx-3 swap block (`ItemSlot.cs:1658-1697`,
            // S1-P-i48): an air cursor or one that cannot stack with the
            // cell enters the `HasItemSwap` cycle then the `SwapEquip`
            // ladder before the paced pickup split.
            let target = chest::chest_slot(&self.world_chests, chest_idx, abs).unwrap_or(EMPTY_SLOT);
            let can_stack = crate::bank::can_stack(&self.mouse_item, &target);
            if self.mouse_item.count == 0 || !can_stack {
                if target.count == 1 && crate::sortdata::HAS_ITEM_SWAP.get(target.kind) {
                    // `:1668-1675` — the variant cycle on the press edge.
                    if self.ui_right_press {
                        let mut item = target;
                        if let Some(new_kind) = crate::app::equip_swap::try_item_swap(&mut item) {
                            chest::set_chest_slot(&mut self.world_chests, chest_idx, abs, item);
                            self.audio
                                .play(crate::app::equip_swap::after_item_swap_cue(new_kind));
                            self.stack_split = 30; // `:2007`
                            self.ui_right_press = false; // `:2008`
                        }
                    }
                    return;
                }
                if target.count == 1
                    && target.kind > 0
                    && (crate::app::equip_swap::can_be_equipped(&target)
                        || registry::get(target.kind).is_some_and(|d| d.dye > 0))
                {
                    // `:1685-1697` — `SwapEquip` on the press edge. The
                    // chest-source favourite clear has no port key space
                    // (the favourites are grid-positional); the
                    // `SendData(32)` MP tail (`:3359-3362`) is unported.
                    if self.ui_right_press {
                        let mut item = target;
                        if self.swap_equip_click(None, &mut item) {
                            chest::set_chest_slot(&mut self.world_chests, chest_idx, abs, item);
                        }
                    }
                    return;
                }
            }
            self.chest_slot_stack_split(abs);
        }
    }

    /// `ItemSlot.PickupItemIntoMouse` (`ItemSlot.cs:1717-1747`): move exactly
    /// one item from `slot` onto the cursor. An air cursor clones the slot at
    /// stack 0 — name and prefix included (`:1721`'s `inv[slot].Clone()`) —
    /// then `mouseItem.stack++` / `slot--` (`:1738-1742`); the emptied slot
    /// turns to air (`:1743-1746`). Returns false when the `:1707` gate
    /// refuses (an air slot, or an alien/full cursor). The `:1728-1735`
    /// favourite carry (a favourited stack of 1 moves its flag onto the
    /// cursor clone) is unrepresentable in the port's slot-positional
    /// favourite model — the grid slots CAN be favourited (`toggle_favorite`),
    /// but the cursor carries no flag; the recorded research/20 §4 deviation
    /// (favourites follow the slot, not the item).
    fn pickup_one_into_mouse(&mut self, slot: &mut ItemSlot) -> bool {
        if slot.kind == 0 || slot.count == 0 {
            return false;
        }
        let max = registry::max_stack(slot.kind);
        // `:1707` — `(CanStack(mouseItem, inv[slot]) && mouseItem.stack <
        // maxStack) || mouseItem.type == 0` (`Item.CanStack` = same kind AND
        // same prefix, `Item.cs:49865-49871`).
        let can_take = self.mouse_item.count == 0
            || (crate::bank::can_stack(&self.mouse_item, slot) && self.mouse_item.count < max);
        if !can_take {
            return false;
        }
        if self.mouse_item.count == 0 {
            self.mouse_item = ItemSlot { count: 0, ..*slot };
        }
        self.mouse_item.count += 1; // `:1738`
        slot.count -= 1; // `:1741`
        if slot.count == 0 {
            *slot = EMPTY_SLOT; // `:1743-1746`
        }
        true
    }

    /// `ItemSlot.RightClick`'s pickup block (`ItemSlot.cs:1700-1713`): the
    /// `stackSplit > 1` gate, then `superFastStack + 1` one-item
    /// `PickupItemIntoMouse` transfers, sound 12 (`:1710`) and
    /// `RefreshStackSplitCooldown` (`:1774-1784`) after each. Returns the
    /// (possibly drained) slot so the caller can write it back to its
    /// container. The ctx-0/3 `SwapEquip` ladder (`:1685-1697`) and the
    /// `HasItemSwap` cycle (`:1668-1675`) run in the callers BEFORE this split
    /// (S1-P-i48); the OpenableBag arm (`:1660-1667`) has no container-opening
    /// state in the port.
    fn right_click_split(&mut self, mut slot: ItemSlot) -> ItemSlot {
        if self.stack_split > 1 || slot.kind == 0 || slot.count == 0 {
            return slot; // `:1700`
        }
        let transfers = self.super_fast_stack + 1; // `:1704`
        for _ in 0..transfers {
            if !self.pickup_one_into_mouse(&mut slot) {
                break; // `:1707` refuses
            }
            self.audio.play("Menu_Tick"); // `:1710` — `SoundEngine.PlaySound(12)`
            // `RefreshStackSplitCooldown` (`:1774-1784`).
            self.stack_split = if self.stack_split == 0 {
                30 // `:1778`
            } else {
                self.stack_delay // `:1782`
            };
            if slot.count == 0 {
                break;
            }
        }
        slot
    }

    /// `ItemSlot.RightClick`'s stack-split for context 3 (`ItemSlot.cs:
    /// 1636-1715`) — the ctx-3 chest cell rides the shared paced
    /// one-per-transfer split (`right_click_split`).
    fn chest_slot_stack_split(&mut self, abs: usize) {
        if self.stack_split > 1 {
            return; // `:1700`
        }
        let chest_idx = self.open_container() as usize;
        let Some(s) = chest::chest_slot(&self.world_chests, chest_idx, abs) else {
            return;
        };
        if s.kind == 0 || s.count == 0 {
            return; // `:1700`'s `inv[slot].IsAir` — no write, no resize
        }
        let s = self.right_click_split(s);
        chest::set_chest_slot(&mut self.world_chests, chest_idx, abs, s);
        self.coin_slot
            .force_slot_state(abs as i32, 3, s.kind, s.count);
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
            // The in-world keybinds screen's Back/Escape
            // (`IngameFancyUI.Close`, `IngameFancyUI.cs:162-188`): the menu
            // closes with the settings sync (`close_menu`) and gameplay
            // resumes — `Main.inFancyUI = false`, no front-end page opens.
            // The close reopens the inventory for every state but the emotes
            // wheel (`:176-184`'s `flag` → `Main.playerInventory = true`).
            Action::ReturnToWorld => {
                self.close_menu();
                self.inventory_open = true;
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
                let click = if yes {
                    FlowClick::ConfirmYes
                } else {
                    FlowClick::ConfirmNo
                };
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
            Action::Host { port, password } => match net::Server::start(port, password) {
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
            },
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
    fn the_wheel_click_spawns_the_drawn_cells_emote() {
        // The break: `card_at` read the uncentred lattice while the draw
        // centred the roster (+31 px), so a click on the drawn cell spawned
        // its neighbour's emote. At the headless 1280x720 the grid origin is
        // (388, 160); the drawn centre of cell (3, 1) is (514, 214) = id 17.
        let mut app = inv_app();
        app.inventory_open = false;
        app.emotes.open();
        let screen = Vec2::new(1280.0, 720.0);
        let grid = ui::emotes::grid_rect(screen);
        let origin = ui::emotes::grid_origin(&grid);
        let mouse = Vec2::new(
            origin[0] + 3.0 * ui::emotes::CELL + ui::emotes::CELL * 0.5,
            origin[1] + ui::emotes::CELL + ui::emotes::CELL * 0.5,
        );
        assert_eq!(ui::emotes::card_at(screen, mouse, 0), Some(17));
        app.mouse_pos = mouse;
        app.ui_left_press = true;
        app.handle_bestiary_clicks();
        assert_eq!(app.emotes.bubbles.len(), 1, "the pick spawned a bubble");
        assert_eq!(app.emotes.bubbles[0].emote, 17, "the drawn cell's id");
        assert_eq!(app.emotes.bubbles[0].life_time, 360, "the vanilla lifetime");
        assert!(app.emotes.bubbles[0].anchored_to_player);
        // `EmoteButton.LeftClick` closes the wheel (`IngameFancyUI.Close`);
        // the emotes arm keeps the inventory closed (`:176-184`).
        assert!(!app.emotes.menu_open);
        assert!(!app.inventory_open);
        assert!(!app.ui_left_press, "the press was consumed");
    }

    #[test]
    fn a_dead_players_pick_spawns_nothing() {
        // `MakeLocalPlayerEmote`'s gate (`EmoteBubble.cs:280`):
        // `Main.netMode == 2 || !Main.LocalPlayer.dead`.
        let mut app = inv_app();
        app.inventory_open = false;
        app.player.as_mut().unwrap().dead = true;
        app.emotes.open();
        let screen = Vec2::new(1280.0, 720.0);
        let grid = ui::emotes::grid_rect(screen);
        let origin = ui::emotes::grid_origin(&grid);
        app.mouse_pos = Vec2::new(origin[0] + 18.0, origin[1] + 18.0);
        app.ui_left_press = true;
        app.handle_bestiary_clicks();
        assert!(app.emotes.bubbles.is_empty(), "the dead gate silences it");
        assert!(!app.emotes.menu_open, "the wheel still closes");
    }

    #[test]
    fn the_wheel_back_is_its_own_box_not_the_bestiarys() {
        // The click side kept BUG-18's mis-box: it hit-tested the
        // bestiary's Back (`UIBestiaryTest.cs:371`, Top -25 on the 900-wide
        // 180-top container) instead of the wheel's own (`UIEmotesMenu.cs:83`,
        // Top -45 on the 590-wide 120-top container). At 1280x720 the wheel
        // Back spans y 625..675, the bestiary's 605..655 — the 655..675 band
        // is wheel-only, and x < 495 inside the bestiary's box is not the
        // wheel's.
        let mut app = inv_app();
        app.inventory_open = false;
        app.emotes.open();
        app.mouse_pos = Vec2::new(640.0, 665.0);
        app.ui_left_press = true;
        app.handle_bestiary_clicks();
        assert!(!app.emotes.menu_open, "the wheel's own Back closes it");
        // And the bestiary-only strip does NOT close the wheel.
        app.emotes.open();
        app.mouse_pos = Vec2::new(450.0, 630.0);
        app.ui_left_press = true;
        app.handle_bestiary_clicks();
        assert!(app.emotes.menu_open, "the bestiary's box is not the wheel's");
        assert!(app.ui_left_press, "the press stays unconsumed");
    }

    #[test]
    fn s1_h_9_the_coin_cell_transfer_resets_the_display_rank() {
        // `ItemSlot.cs:1016`'s tail: after the coin row's transfer the
        // `CoinSlot.ForceSlotState(slot, context, inv[slot])` reset makes the
        // cell's display rank follow the moved coin — a gold into a copper
        // cell swaps (the case-0 ladder's `IsNotTheSameAs` arm) and the rank
        // re-derives to the gold; the row rides the vanilla indices
        // (`coin_slot_index` = `Player.inventory[50..53]`).
        let mut app = inv_app();
        app.equip.coins[0] = crate::item::ItemSlot {
            kind: 71,
            name: "Copper Coin",
            count: 99,
            prefix: 0,
        };
        app.mouse_item = crate::item::ItemSlot {
            kind: 73,
            name: "Gold Coin",
            count: 1,
            prefix: 0,
        };
        app.coin_ammo_click(1, 0, true, false);
        assert_eq!(
            app.equip.coins[0].kind, 73,
            "the case-0 swap moved the gold in"
        );
        assert_eq!(app.mouse_item.kind, 71, "the copper rides the cursor");
        assert_eq!(
            app.coin_slot
                .entry_rank(crate::inv::coin_slot_index(0) as i32, 1),
            (73, 1),
            "the display rank follows the moved coin"
        );
    }

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

    /// S1-H-6: `context_accepts` must reflect `ItemSlot.PickItemMovementAction`
    /// (`ItemSlot.cs:1484-1634`) instead of accepting every context past 1/2.
    #[test]
    fn context_accepts_matches_the_pick_item_movement_action_table() {
        // ctx 7 (GuideItem, `:1524-1529`): the material flag (or air) is the
        // whole gate.
        assert!(context_accepts(7, 0), "air is accepted");
        assert!(context_accepts(7, 2), "a material item is accepted");
        assert!(!context_accepts(7, 4), "a non-material item is refused");
    }

    /// S1-H-6: the rest of the `PickItemMovementAction` table — the slot- and
    /// target-dependent arms that `context_accepts` cannot express.
    #[test]
    fn pick_item_movement_action_ports_the_context_table() {
        let air = EMPTY_SLOT;
        // Fixtures: 89 Copper Helmet (head), 80 Copper Chainmail (body),
        // 76 Copper Greaves (leg), 15 Copper Watch (accessory),
        // 2 Dirt Block (material), 4 Iron Broadsword (plain), 71 Copper Coin,
        // 40 Wooden Arrow (ammo).
        let head = slot(89, 1);
        let body = slot(80, 1);
        let leg = slot(76, 1);
        let acc = slot(15, 1);
        let mat = slot(2, 1);
        let plain = slot(4, 1);
        // The ctx 8-11 gate reads the live player + armour set (S1-P-i48).
        let p = crate::player::Player::spawn(&tr_core::world::World::new("t".into(), 64, 800));
        let armor = [EMPTY_SLOT; crate::inv::ARMOR];
        let eq = Some((&p, &armor));

        // `:1490-1492`, `:1505-1507`, `:1521-1523`, `:1629-1632`: the
        // unconditional accept contexts (action 0).
        for ctx in [0, 3, 6, 30] {
            assert_eq!(pick_item_movement_action(ctx, &plain, 0, &air, None), 0);
        }
        // `:1493-1498`, `:1499-1504`: the coin/ammo rows.
        assert_eq!(pick_item_movement_action(1, &slot(71, 1), 0, &air, None), 0);
        assert_eq!(pick_item_movement_action(1, &air, 0, &air, None), 0);
        assert_eq!(pick_item_movement_action(1, &plain, 0, &air, None), -1);
        assert_eq!(pick_item_movement_action(2, &slot(40, 1), 0, &air, None), 0);
        assert_eq!(pick_item_movement_action(2, &plain, 0, &air, None), -1);
        // `:1524-1529`.
        assert_eq!(pick_item_movement_action(7, &mat, 0, &air, None), 0);
        assert_eq!(pick_item_movement_action(7, &plain, 0, &air, None), -1);
        // `:1530-1538`: the armour/vanity/accessory rows by `slot % 10`.
        assert_eq!(pick_item_movement_action(8, &head, 0, &air, eq), 1);
        assert_eq!(pick_item_movement_action(8, &body, 0, &air, eq), -1);
        assert_eq!(pick_item_movement_action(9, &body, 1, &air, eq), 1);
        assert_eq!(pick_item_movement_action(10, &acc, 3, &air, eq), 1);
        assert_eq!(pick_item_movement_action(10, &head, 3, &air, eq), -1);
        assert_eq!(pick_item_movement_action(11, &leg, 2, &air, eq), 1);
        // S1-P-i48 — the duplicate half: the vanity set's helmet blocks the
        // real row (`CanEquipInArmorSlot` `:1455`), and a second same-kind
        // accessory in another slot is refused (`:1471-1475`).
        let mut armor2 = armor;
        armor2[10] = head;
        let eq2 = Some((&p, &armor2));
        assert_eq!(pick_item_movement_action(8, &head, 0, &air, eq2), -1);
        let mut armor3 = armor;
        armor3[3] = acc;
        let eq3 = Some((&p, &armor3));
        assert_eq!(pick_item_movement_action(10, &acc, 4, &air, eq3), -1);
        assert_eq!(pick_item_movement_action(10, &acc, 3, &air, eq3), 1);
        // Air always passes, even with no player state (`:1534`).
        assert_eq!(pick_item_movement_action(10, &air, 3, &air, None), 1);
        // `:1560-1565`: the dye rows take anything (action 2).
        assert_eq!(pick_item_movement_action(12, &plain, 0, &air, None), 2);
        assert_eq!(pick_item_movement_action(33, &plain, 0, &air, None), 2);
        // `:1539-1544`: display-doll armour, strictly `> 0`.
        assert_eq!(pick_item_movement_action(23, &head, 0, &air, None), 1);
        assert_eq!(pick_item_movement_action(23, &body, 0, &air, None), -1);
        assert_eq!(pick_item_movement_action(23, &acc, 0, &air, None), -1);
        // `:1551-1559`: display-doll accessory.
        assert_eq!(pick_item_movement_action(24, &acc, 0, &air, None), 1);
        assert_eq!(pick_item_movement_action(24, &head, 0, &air, None), -1);
        // `:1545-1550`: the hat rack takes a head item.
        assert_eq!(pick_item_movement_action(26, &head, 0, &air, None), 1);
        assert_eq!(pick_item_movement_action(26, &body, 0, &air, None), -1);
        // `:1622-1627`: the creative arm (air over a filled slot, action 5).
        assert_eq!(pick_item_movement_action(29, &air, 0, &plain, None), 5);
        assert_eq!(pick_item_movement_action(29, &air, 0, &air, None), -1);
        assert_eq!(pick_item_movement_action(29, &plain, 0, &mat, None), -1);
        // `:1566-1579`: the shop sell (3) / buy (4) ladder.
        assert_eq!(pick_item_movement_action(15, &air, 0, &plain, None), 3);
        assert_eq!(pick_item_movement_action(15, &plain, 0, &air, None), 4);
        assert_eq!(pick_item_movement_action(15, &plain, 0, &mat, None), -1);
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
            ..Default::default()
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
            ..Default::default()
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
        assert!(
            app.handle_chest_clicks(),
            "the cancel button took the click"
        );
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
            app.equip.coins[3].count, 50,
            "the coins ride the coin rows first (row 53, the descending empty fill)"
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

    /// S1-H-8 — the Sort button (id 4) runs `ItemSorting.SortChest`
    /// (`ChestUI.cs:304-306`) over the open world chest: the layer engine
    /// reorders the slots so the `Weapons - Melee` sword sorts before the
    /// `Last - Tiles (Common)` block, compacting into the low slots.
    #[test]
    fn s1_h_8_sort_button_reorders_the_world_chest() {
        let mut app = stocked_chest_app(vec![slot(2, 10), slot(24, 1)]);
        ready_buttons(&mut app);
        app.mouse_pos = Vec2::new(510.0, 404.0); // the Sort row (id 4)
        app.ui_left_press = true;
        assert!(app.handle_chest_clicks(), "the Sort button took the click");
        assert!(!app.ui_left_press, "the press is consumed");
        assert_eq!(
            app.world_chests[0].slots[0].kind, 24,
            "the melee weapon sorts before the dirt block"
        );
        assert_eq!(app.world_chests[0].slots[1].kind, 2);
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
            app.equip.coins[3].count, 50,
            "the coins still ride the rows (row 53, the descending empty fill)"
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
        assert_eq!(bank[0].kind, 71, "the grid stacks parked as plain stacks");
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
        assert_eq!(
            app.mouse_item,
            slot(71, 37),
            "the cell emptied onto the cursor"
        );
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

    /// S1-C-10 — the coin/ammo rows' right click is the `RightClick` pickup
    /// block (`ItemSlot.cs:1700-1713`): one item per paced transfer
    /// (`PickupItemIntoMouse`, `:1717-1747`), not the old half-stack split.
    #[test]
    fn ui_2b_the_rows_take_one_on_right_click() {
        let mut app = inv_app();
        app.equip.coins[1] = slot(72, 11);
        app.mouse_pos = Vec2::new(505.0, 145.0); // coin row 1
        app.ui_right_press = true;
        app.handle_inventory_clicks();
        assert_eq!(app.mouse_item, slot(72, 1), "one coin onto the cursor");
        assert_eq!(app.equip.coins[1], slot(72, 10), "the cell keeps the rest");
        assert!(app.stack_split > 1, "the transfer armed the split cooldown");
    }

    /// S1-C-10 — the inventory grid's right click takes one item, arms the
    /// `RefreshStackSplitCooldown` pacing (`ItemSlot.cs:1774-1784`) and then
    /// repeats only after the cooldown drains (the held `Main.mouseRight`
    /// dispatch, `:1654`). This is the reported "moves half the stack" bug.
    #[test]
    fn s1_c_10_inventory_right_click_takes_one_and_repeats_when_paced() {
        let mut app = inv_app();
        app.inventory[5] = slot(9, 6);
        app.mouse_pos = INV_SLOT_5;
        // The press edge: exactly one item onto the cursor.
        app.ui_right_press = true;
        app.place_down = true;
        app.handle_inventory_clicks();
        assert_eq!(app.mouse_item, slot(9, 1), "one item onto the cursor");
        assert_eq!(app.inventory[5], slot(9, 5), "the slot keeps the rest");
        assert!(app.stack_split > 1, "the transfer armed the cooldown");

        // A held frame inside the cooldown moves nothing.
        app.ui_right_press = false;
        app.handle_inventory_clicks();
        assert_eq!(app.mouse_item.count, 1, "the cooldown paces the hold");
        assert_eq!(app.inventory[5].count, 5);

        // Once the cooldown drains, the held button takes one more.
        app.stack_split = 1;
        app.handle_inventory_clicks();
        assert_eq!(app.mouse_item.count, 2, "the held button repeats");
        assert_eq!(app.inventory[5].count, 4);
    }

    /// S1-C-10 — the accelerator half of `RightClick` (`ItemSlot.cs:1704`):
    /// one gated call runs `superFastStack + 1` one-item transfers.
    #[test]
    fn s1_c_10_inventory_right_click_honours_the_accelerator() {
        let mut app = inv_app();
        app.inventory[5] = slot(9, 10);
        app.mouse_pos = INV_SLOT_5;
        app.stack_delay = 7;
        app.super_fast_stack = 2; // `superFastStack + 1` = 3 transfers
        app.stack_split = 1;
        app.ui_right_press = true;
        app.place_down = true;
        app.handle_inventory_clicks();
        assert_eq!(app.mouse_item.count, 3, "three one-item transfers");
        assert_eq!(app.inventory[5].count, 7);
    }

    /// S1-C-10 — the bank panel's right click takes one item (the same
    /// `RightClick` pickup block, `ItemSlot.cs:1700-1713`) and is paced.
    #[test]
    fn s1_c_10_bank_right_click_takes_one_item() {
        let mut app = bank_app();
        app.player.as_mut().unwrap().banks[0][0] = slot(9, 7);
        app.mouse_item = EMPTY_SLOT;
        app.mouse_pos = Vec2::new(20.0, 330.0); // bank row 0 at m_h 60
        app.ui_right_press = true;
        app.place_down = true;
        app.handle_inventory_clicks();
        assert_eq!(app.mouse_item, slot(9, 1), "one item onto the cursor");
        assert_eq!(
            app.player.as_ref().unwrap().banks[0][0],
            slot(9, 6),
            "the bank cell keeps the rest"
        );
        assert!(app.stack_split > 1, "the transfer armed the split cooldown");
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
        assert!(
            app.handle_chest_clicks(),
            "the cursor-9 arm takes the click"
        );
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
        assert!(
            app.handle_chest_clicks(),
            "the cursor-9 arm takes the click"
        );
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
        assert_eq!(
            crate::chest::chest_slot(&app.world_chests, 0, 0),
            Some(slot(72, 4))
        );
        assert_eq!(app.equip.coins[0].count, 0, "the coin cell drained");
        assert_eq!(app.inventory[0], slot(9, 3), "the grid slot is untouched");
    }

    /// M7-UI-2d — Alt+left-click favorites a coin/ammo cell in the vanilla key
    /// space (`canFavoriteAt`'s contexts 1/2, `ItemSlot.cs:386-389`: the coins
    /// are `Player.inventory[50..53]`, the ammo `54..57`), and the `.plr`
    /// bitfield carries the row (`Player.cs:54820-54824`'s 58 per-slot bools).
    #[test]
    fn ui_2d_alt_click_favorites_the_rows() {
        let mut app = inv_app();
        app.equip.coins[1] = slot(71, 9);
        app.equip.ammo[0] = slot(40, 5);
        app.keys.insert(KeyCode::AltLeft);
        app.mouse_pos = Vec2::new(505.0, 145.0); // coin row 1
        app.ui_left_press = true;
        assert!(!app.handle_chest_clicks());
        app.handle_inventory_clicks();
        assert!(
            app.is_favorite(crate::inv::coin_slot_index(1)),
            "the coin cell took the favorite"
        );
        assert_eq!(app.equip.coins[1], slot(71, 9), "the stack stayed put");
        // The ammo row shares the key space, at 54..58.
        app.mouse_pos = AMMO_SLOT_0;
        app.ui_left_press = true;
        app.handle_inventory_clicks();
        assert!(app.is_favorite(crate::inv::ammo_slot_index(0)));
        assert_eq!(app.favorite_bits(), (1 << 51) | (1 << 54));
        // The round-trip reads the same bits back.
        let mut other = inv_app();
        other.hydrate_favorites((1 << 51) | (1 << 54));
        assert!(other.is_favorite(crate::inv::coin_slot_index(1)));
        assert!(other.is_favorite(crate::inv::ammo_slot_index(0)));
        // Alt again clears.
        app.ui_left_press = true;
        app.handle_inventory_clicks();
        assert!(!app.is_favorite(crate::inv::ammo_slot_index(0)));
    }

    /// M7-UI-2d — a favorited coin/ammo cell never lights the shift-transfer
    /// cursor (`GetAlternateClickAction`'s head guard, `ItemSlot.cs:631-634`).
    #[test]
    fn ui_2d_the_favorited_rows_refuse_the_shift_transfer() {
        let mut app = chest_app(vec![]);
        app.equip.coins[0] = slot(71, 20);
        app.favorite_slots.push(crate::inv::coin_slot_index(0));
        app.keys.insert(KeyCode::ShiftLeft);
        app.mouse_pos = COIN_SLOT_0;
        app.ui_left_press = true;
        assert!(!app.handle_chest_clicks(), "the guarded arm takes nothing");
        assert_eq!(app.cursor_override, -1, "the cursor stays dark");
        app.handle_inventory_clicks();
        // The press falls through to the plain pickup arm — the row's own
        // left-click — so the stack lands on the cursor, never in the chest.
        assert_eq!(app.mouse_item, slot(71, 20), "the pickup arm took it");
        assert_eq!(
            crate::chest::chest_slot(&app.world_chests, 0, 0),
            Some(crate::item::EMPTY_SLOT)
        );
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
        assert_eq!(app.mouse_item.count, 1, "one item onto the cursor");
        assert_eq!(app.inventory[5].count, 5);
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
        assert_eq!(
            app.inventory[5],
            slot(9, 4),
            "the C# movement places on air"
        );
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
        assert!(
            !chest::can_favorite_at(3),
            "chest slots cannot be favorited"
        );
        assert!(!chest::can_favorite_at(5));
        assert!(
            !chest::can_favorite_at(6),
            "the trash slot cannot be favorited"
        );
        assert!(
            !chest::can_favorite_at(13),
            "the closed hotbar cannot either"
        );

        let mut app = inv_app();
        assert_eq!(app.toggle_favorite(7), 12);
        assert!(app.is_favorite(7));
        assert_eq!(app.toggle_favorite(7), 12, "the C# plays 12 on both flips");
        assert!(!app.is_favorite(7));
        // M7-UI-2d moved the guard out to the vanilla array's end: the coin
        // row (50..54) and the ammo row (54..58) are favoritable, slot 58 and
        // beyond are not (`crate::inv::INV_SLOTS`).
        assert_eq!(app.toggle_favorite(crate::item::INV), 12, "coin row 0");
        assert_eq!(
            app.toggle_favorite(crate::inv::INV_SLOTS - 1),
            12,
            "ammo row 3"
        );
        assert_eq!(
            app.toggle_favorite(crate::inv::INV_SLOTS),
            0,
            "past the rows"
        );

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
        // S1-P-i54 — the click buffers the request (`Main.cs:43622`
        // `changeItem` -> `Player.cs:25087-25091` `Select`); the live slot
        // changes on the player tick (`selectedItemState.Update`,
        // `:25218`), not on the click.
        assert_eq!(app.selection.buffered, 3, "the select buffered");
        crate::world_sim::player::update_player(&mut app, 1.0 / 60.0);
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
        assert_eq!(
            app.favorite_slots,
            vec![5],
            "context 0 retains the favorite"
        );
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

    /// `Main.cs:41426-41460`: a press on an accessory's "hide the visuals" tick
    /// flips `Player.hideVisibleAccessory`, cues `PlaySound(12)` and is consumed
    /// before the slot grid — while a held stack leaves the tick inert.
    #[test]
    fn the_accessory_tick_toggles_the_visibility() {
        let mut app = inv_app();
        let (sw, sh) = app.screen_size();
        let screen = Vec2::new(sw, sh);
        let base_y = ui::inventory::equip_page_base_y(sh, app.main_m_h());
        let tick = ui::inventory::accessory_tick_rects(screen, base_y, (16.0, 16.0));
        let (idx, rect) = tick[0];
        assert_eq!(idx, 3, "the first tick belongs to the first accessory");
        app.mouse_pos = Vec2::new(rect[0] + 2.0, rect[1] + 2.0);
        let before = app.audio.played_count();
        app.ui_left_press = true;
        app.handle_inventory_clicks();
        assert!(
            app.player.as_ref().unwrap().hide_visible_accessory[idx],
            "the tick flipped"
        );
        assert_eq!(app.audio.played_count(), before + 1, "`PlaySound(12)`");
        // Pressing it again shows the accessory.
        app.mouse_pos = Vec2::new(rect[0] + 2.0, rect[1] + 2.0);
        app.ui_left_press = true;
        app.handle_inventory_clicks();
        assert!(!app.player.as_ref().unwrap().hide_visible_accessory[idx]);
        // `Main.mouseItem.IsAir` (`:41430`): with a stack on the cursor the tick
        // is dead and the press falls through to the slot behind it.
        app.mouse_item = slot(3, 5);
        app.mouse_pos = Vec2::new(rect[0] + 2.0, rect[1] + 2.0);
        app.ui_left_press = true;
        app.handle_inventory_clicks();
        assert!(!app.player.as_ref().unwrap().hide_visible_accessory[idx]);
    }

    /// `Main.cs:41426-41460` only exists on the equipment page: the misc page
    /// (`EquipPage == 2`) draws no accessory column, so no tick fires there.
    #[test]
    fn the_misc_page_has_no_accessory_ticks() {
        // Only the equipment page (`EquipPage == 0`) draws the accessory column:
        // the misc page (2, `Main.cs:41172`) and the housing page (1, `:41368`)
        // both replace it, so neither can flip a hide flag through the tick box.
        for page in [2i32, 1] {
            let mut app = inv_app();
            app.equip_page = page;
            let (sw, sh) = app.screen_size();
            let screen = Vec2::new(sw, sh);
            let base_y = ui::inventory::equip_page_base_y(sh, app.main_m_h());
            let (_, rect) = ui::inventory::accessory_tick_rects(screen, base_y, (16.0, 16.0))[0];
            app.mouse_pos = Vec2::new(rect[0] + 2.0, rect[1] + 2.0);
            app.ui_left_press = true;
            app.handle_inventory_clicks();
            assert!(
                !app.player.as_ref().unwrap().hide_visible_accessory[3],
                "page {page} must not toggle the tick"
            );
        }
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

    // ------------------------------------------------------------------
    // S1-H-5 — the equipment columns and the right-click equip ladder
    // ------------------------------------------------------------------

    /// Inside the inventory grid's cell (0, 0).
    const INV_SLOT_0: Vec2 = Vec2::new(48.0, 48.0);

    /// The pointer over the cell `(context, index)` of `page`, found through
    /// the same `equip_cell_at` the click dispatch resolves the press with.
    fn equip_mouse(app: &App, page: i32, context: i32, idx: usize) -> Vec2 {
        let (sw, sh) = app.screen_size();
        let screen = Vec2::new(sw, sh);
        let m_h = app.main_m_h();
        let cell = app.lookups.ui_back.map_or(52.0, |b| b.width as f32);
        for y in 0..sh as i32 {
            for x in sw as i32 - 240..sw as i32 {
                let p = Vec2::new(x as f32, y as f32);
                if ui::inventory::equip_cell_at(p, screen, m_h, page, cell) == Some((context, idx)) {
                    return p;
                }
            }
        }
        panic!("no cell ({context}, {idx}) on equip page {page}");
    }

    /// `ItemSlot.Handle` over `Main.DrawInventory`'s armour column
    /// (`Main.cs:41288`) -> `PickItemMovementAction` action 1 (`:1530-1538`)
    /// -> the `:1060-1084` place-one arm.
    #[test]
    fn s1_h_5_left_click_equips_into_the_armour_column() {
        let mut app = inv_app();
        app.mouse_item = slot(89, 1);
        app.mouse_pos = equip_mouse(&app, 0, 8, 0);
        app.ui_left_press = true;
        app.handle_inventory_clicks();
        assert_eq!(app.equip.armor[0], slot(89, 1), "the helmet lands in armor[0]");
        assert_eq!(app.mouse_item.count, 0, "the cursor drained");
    }

    /// Action 2 (`:1560-1565` + `:1121-1218`): the dye column takes a dye and
    /// refuses everything else.
    #[test]
    fn s1_h_5_the_dye_column_takes_dye_only() {
        let mut app = inv_app();
        app.mouse_item = slot(4, 1);
        app.mouse_pos = equip_mouse(&app, 0, 12, 0);
        app.ui_left_press = true;
        app.handle_inventory_clicks();
        assert_eq!(app.equip.dye[0].count, 0, "a sword is no dye");
        assert_eq!(app.mouse_item.kind, 4, "the cursor keeps it");
        app.mouse_item = slot(1007, 1);
        app.ui_left_press = true;
        app.handle_inventory_clicks();
        assert_eq!(app.equip.dye[0], slot(1007, 1), "the red dye goes in");
        assert_eq!(app.mouse_item.count, 0);
    }

    /// Right-press the grid's first cell with a single `kind` in it —
    /// `RightClick`'s `:1689-1696` arm.
    fn right_click_grid0(kind: u16) -> App {
        let mut app = inv_app();
        app.inventory[0] = slot(kind, 1);
        app.mouse_pos = INV_SLOT_0;
        app.ui_right_press = true;
        app.handle_inventory_clicks();
        app
    }

    /// `SwapEquip`'s arm ladder (`ItemSlot.cs:3291-3358`) through the grid
    /// right-click, and `Main.EquipPageSelected` following the item.
    #[test]
    fn s1_h_5_right_click_swaps_the_equipment_rows() {
        let a = right_click_grid0(89);
        assert_eq!(a.equip.armor[0], slot(89, 1), "helmet -> armor[0]");
        assert_eq!(a.inventory[0].count, 0, "the grid cell emptied");
        assert_eq!(a.equip_page_selected, 0, "`:3296`");

        let a = right_click_grid0(1007);
        assert_eq!(a.equip.dye[0], slot(1007, 1), "dye -> dye[0]");
        assert_eq!(a.dye_swap_counter, 1, "`:3564` advanced");

        let a = right_click_grid0(84);
        assert_eq!(a.equip.misc_equips[4], slot(84, 1), "grapple -> misc[4]");
        assert_eq!(
            a.equip_page_selected,
            crate::ui::inventory::EQUIP_PAGE_MISC,
            "`:3306`"
        );

        let a = right_click_grid0(1914);
        assert_eq!(a.equip.misc_equips[3], slot(1914, 1), "mount -> misc[3]");
        let a = right_click_grid0(2343);
        assert_eq!(a.equip.misc_equips[2], slot(2343, 1), "minecart -> misc[2]");
        let a = right_click_grid0(115);
        assert_eq!(a.equip.misc_equips[1], slot(115, 1), "light pet -> misc[1]");
        let a = right_click_grid0(603);
        assert_eq!(a.equip.misc_equips[0], slot(603, 1), "vanity pet -> misc[0]");

        // A plain sword has no arm: the ladder falls through to the paced
        // pickup (`:1700-1713`) and the sword rides the cursor.
        let a = right_click_grid0(4);
        assert_eq!(a.inventory[0].count, 0, "picked up, not equipped");
        assert_eq!(a.mouse_item, slot(4, 1));
        assert_eq!(a.equip.misc_equips[0].count, 0);
    }

    /// `SwapEquip`'s `SwapClearFavorite` (`:3635`): the port's positional
    /// favourite table drops the cell the item left.
    #[test]
    fn s1_h_5_the_swap_clears_the_grid_favourite() {
        let mut app = inv_app();
        app.inventory[0] = slot(89, 1);
        app.favorite_slots.push(0);
        app.mouse_pos = INV_SLOT_0;
        app.ui_right_press = true;
        app.handle_inventory_clicks();
        assert_eq!(app.equip.armor[0].kind, 89);
        assert!(!app.is_favorite(0), "the favourite cleared with the move");
    }

    /// `RightClick`'s context 9/11 arm (`:1678-1684`) -> `SwapVanityEquip`
    /// (`:1867-1899`): the vanity row and its real counterpart trade.
    #[test]
    fn s1_h_5_right_click_swaps_a_vanity_row() {
        let mut app = inv_app();
        app.equip.armor[0] = slot(89, 1);
        app.equip.armor[10] = slot(237, 1);
        app.mouse_pos = equip_mouse(&app, 0, 9, 10);
        app.ui_right_press = true;
        app.handle_inventory_clicks();
        assert_eq!(app.equip.armor[0], slot(237, 1), "the real row");
        assert_eq!(app.equip.armor[10], slot(89, 1), "the vanity row");
    }

    // ------------------------------------------------------------------
    // S1-P-i48 — the accessory equip interaction fixes.
    // ------------------------------------------------------------------

    /// `CanEquipInArmorSlot`'s duplicate half (`ItemSlot.cs:1469-1481`)
    /// through the click: a second Copper Watch refuses every OTHER real
    /// accessory slot, but its own occupied slot still takes the one-for-one
    /// swap (`LeftClick` action 1, `:1020-1023`).
    #[test]
    fn s1_p_i48_a_duplicate_accessory_refuses_the_other_slots() {
        let mut app = inv_app();
        app.equip.armor[3] = slot(15, 1);
        app.mouse_item = slot(15, 1);
        // Slot 4 is refused — the collision is at slot 3, not here.
        app.mouse_pos = equip_mouse(&app, 0, 10, 4);
        app.ui_left_press = true;
        app.handle_inventory_clicks();
        assert_eq!(app.equip.armor[4].count, 0, "the duplicate refused");
        assert_eq!(app.mouse_item, slot(15, 1), "the cursor keeps it");
        // The collision slot itself accepts the placement
        // (`CanEquipAccessoryInSlot` `:3535` — `slot == collisionSlot`); a
        // DIFFERENT prefix makes `IsNotTheSameAs` true (`Item.cs:49847-49854`)
        // so the one-for-one swap (`:1020-1023`) runs.
        app.mouse_item = prefixed(15, 1, 65);
        app.mouse_pos = equip_mouse(&app, 0, 10, 3);
        app.ui_left_press = true;
        app.handle_inventory_clicks();
        assert_eq!(app.equip.armor[3], prefixed(15, 1, 65), "the swap ran");
        assert_eq!(app.mouse_item, slot(15, 1), "the old watch came back");
    }

    /// The vanity column's duplicate half (`ItemSlot.cs:1477-1480`): a
    /// vanity placement refuses when the REAL column already wears the same
    /// kind (`HasSameItemInSlot(check, AccSlots)`). The show+stats pair is
    /// the RIGHT-click `ArmorSwap` path's business (`:3589` checks the
    /// opposite column only), not the left-click gate's.
    #[test]
    fn s1_p_i48_the_vanity_column_refuses_a_worn_real_accessory() {
        let mut app = inv_app();
        app.equip.armor[3] = slot(15, 1);
        app.mouse_item = slot(15, 1);
        app.mouse_pos = equip_mouse(&app, 0, 11, 13);
        app.ui_left_press = true;
        app.handle_inventory_clicks();
        assert_eq!(app.equip.armor[13].count, 0, "`:1479` HasSameItemInSlot(AccSlots)");
        assert_eq!(app.mouse_item, slot(15, 1));
    }

    /// `CanEquipBothAccessories`' two-wings arm (`ItemSlot.cs:3457-3464`)
    /// through the right-click ladder: with Demon Wings worn, a right-click
    /// on Angel Wings lands on the WORN wing's slot (the collision target,
    /// `ArmorSwap` `:3609-3614`) — it swaps the wings, never doubles them.
    #[test]
    fn s1_p_i48_the_second_wings_pair_swaps_the_worn_slot() {
        let mut app = inv_app();
        app.equip.armor[3] = slot(492, 1);
        app.inventory[0] = slot(493, 1);
        app.mouse_pos = INV_SLOT_0;
        app.ui_right_press = true;
        app.handle_inventory_clicks();
        assert_eq!(app.equip.armor[3], slot(493, 1), "the wings swapped");
        assert_eq!(app.inventory[0], slot(492, 1), "the old wings came back");
        assert_eq!(app.equip.armor[4].count, 0, "no second pair");
    }

    /// The left-click gate refuses the second wings pair in another slot
    /// (`CanEquipAccessoryInSlot` over the wings collision).
    #[test]
    fn s1_p_i48_the_left_click_refuses_a_second_wings_pair() {
        let mut app = inv_app();
        app.equip.armor[3] = slot(492, 1);
        app.mouse_item = slot(493, 1);
        app.mouse_pos = equip_mouse(&app, 0, 10, 4);
        app.ui_left_press = true;
        app.handle_inventory_clicks();
        assert_eq!(app.equip.armor[4].count, 0);
        assert_eq!(app.mouse_item, slot(493, 1));
    }

    /// `TryItemSwap` through the grid right-click (`ItemSlot.cs:1668-1675`):
    /// an Ammo Cycler cycles its mode IN PLACE instead of equipping, plays
    /// the Unlock cue (`AfterItemSwap` `:2001`), arms `stackSplit = 30`
    /// (`:2007`) and consumes the press edge (`:2008`).
    #[test]
    fn s1_p_i48_right_click_cycles_the_ammo_cycler_in_place() {
        let mut app = inv_app();
        app.inventory[0] = slot(6168, 1);
        app.mouse_pos = INV_SLOT_0;
        let before = app.audio.played_count();
        app.ui_right_press = true;
        app.handle_inventory_clicks();
        assert_eq!(app.inventory[0].kind, 6169, "6168 -> 6169 (`:1973-1976`)");
        assert_eq!(app.equip.armor[3].count, 0, "NOT equipped");
        assert_eq!(app.stack_split, 30, "`:2007`");
        assert!(!app.ui_right_press, "`:2008` consumed the edge");
        assert_eq!(app.audio.played_count(), before + 1, "one cue");
        // The full cycle returns to the start.
        for expect in [6193u16, 6194, 6168] {
            app.ui_right_press = true;
            app.handle_inventory_clicks();
            assert_eq!(app.inventory[0].kind, expect);
        }
    }

    /// The same cycle on a WORN cycler (ctx 10, `canItemSwapAt[10]`,
    /// `ItemSlot.cs:455`): the mode changes in the accessory slot.
    #[test]
    fn s1_p_i48_right_click_cycles_a_worn_cycler() {
        let mut app = inv_app();
        app.equip.armor[3] = slot(6168, 1);
        app.mouse_pos = equip_mouse(&app, 0, 10, 3);
        app.ui_right_press = true;
        app.handle_inventory_clicks();
        assert_eq!(app.equip.armor[3].kind, 6169);
        assert_eq!(app.stack_split, 30);
    }

    /// The busy gates (`ItemSlot.cs:922-926` / `:1644-1647`): while the
    /// swing runs (`App::swing` = `itemAnimation > 0`) neither button moves
    /// equipment; the use cooldown (`App::item_time`) blocks the left only.
    #[test]
    fn s1_p_i48_the_swing_blocks_the_equip_clicks() {
        let mut app = inv_app();
        app.mouse_item = slot(89, 1);
        app.mouse_pos = equip_mouse(&app, 0, 8, 0);
        app.swing = 0.2;
        app.ui_left_press = true;
        app.handle_inventory_clicks();
        assert_eq!(app.equip.armor[0].count, 0, "the swing refused the left");
        assert_eq!(app.mouse_item, slot(89, 1));
        // The right-click pickup is refused too (`:1644-1647`).
        app.mouse_item = EMPTY_SLOT;
        app.equip.armor[0] = slot(89, 1);
        app.ui_right_press = true;
        app.handle_inventory_clicks();
        assert_eq!(app.equip.armor[0], slot(89, 1), "the swing refused the right");
        assert_eq!(app.mouse_item.count, 0);
        // itemTime alone blocks the LEFT (`:923`'s `itemTime != 0`).
        app.swing = 0.0;
        app.item_time = 0.1;
        app.mouse_item = slot(80, 1);
        app.mouse_pos = equip_mouse(&app, 0, 8, 1);
        app.ui_left_press = true;
        app.handle_inventory_clicks();
        assert_eq!(app.equip.armor[1].count, 0, "itemTime refused the left");
        // ... but not the right (`:1644` reads itemAnimation only).
        app.mouse_item = EMPTY_SLOT;
        app.mouse_pos = equip_mouse(&app, 0, 8, 0);
        app.ui_right_press = true;
        app.handle_inventory_clicks();
        assert_eq!(app.mouse_item, slot(89, 1), "the right still picks up");
    }

    /// The Demon Heart unlock through the ladder (S1-P-i48): with rows 3..=7
    /// full, a new accessory falls back to row 3 while locked
    /// (`IsItemSlotUnlockedAndUsable` `:12585-12595`), and lands on row 8
    /// once `extraAccessory` + expert mode arm — the SAME gate the sim's
    /// grant loop reads (`Player.cs:12948`).
    #[test]
    fn s1_p_i48_the_demon_heart_opens_row_eight_to_the_ladder() {
        let mut app = inv_app();
        for i in 3..8 {
            app.equip.armor[i] = slot(16 + i as u16, 1);
        }
        app.inventory[0] = slot(212, 1);
        app.mouse_pos = INV_SLOT_0;
        app.ui_right_press = true;
        app.handle_inventory_clicks();
        assert_eq!(app.equip.armor[3], slot(212, 1), "locked: the :3618 fallback");
        assert_eq!(app.inventory[0].kind, 16 + 3, "the old row-3 item came back");
        // Unlock and retry with a fresh accessory.
        let p = app.player.as_mut().unwrap();
        p.extra_accessory = true;
        p.buffs.expert_mode = true;
        app.inventory[0] = slot(285, 1);
        app.ui_right_press = true;
        app.handle_inventory_clicks();
        assert_eq!(app.equip.armor[8], slot(285, 1), "row 8 joined the scan");
        assert_eq!(app.inventory[0].count, 0);
    }

    /// S1-P-i48 — the ctx-3 chest `RightClick` ladder (`ItemSlot.cs:1685-1697`
    /// with `canItemSwapAt[3]` `:454`): a single wearable in a chest cell
    /// right-clicks into its equipment row on the press edge (the `SwapEquip`
    /// arm, `:3348-3358` -> `ArmorSwap`), NOT onto the cursor.
    #[test]
    fn s1_p_i48_the_chest_cell_right_click_equips_a_wearable() {
        let mut app = chest_app(vec![slot(15, 1)]);
        app.mouse_pos = CHEST_SLOT_0;
        app.place_down = true;
        app.ui_right_press = true;
        assert!(app.handle_chest_clicks());
        assert_eq!(app.equip.armor[3], slot(15, 1), "the ladder equipped it");
        assert_eq!(chest::chest_slot(&app.world_chests, 0, 0), Some(EMPTY_SLOT));
        assert_eq!(app.mouse_item.count, 0, "not a pickup");
        // The `HasItemSwap` cycle comes FIRST (`:1668-1675`): a chest cycler
        // changes mode in place instead of equipping.
        let mut app = chest_app(vec![slot(6168, 1)]);
        app.mouse_pos = CHEST_SLOT_0;
        app.place_down = true;
        app.ui_right_press = true;
        assert!(app.handle_chest_clicks());
        assert_eq!(
            chest::chest_slot(&app.world_chests, 0, 0),
            Some(slot(6169, 1)),
            "cycled, not equipped"
        );
        assert_eq!(app.equip.armor[3].count, 0);
        assert_eq!(app.stack_split, 30, "`AfterItemSwap` `:2007`");
    }

    /// S1-P-i53 — the accessory vanity pair swaps through the ctx-11 arm
    /// too (`ItemSlot.cs:1678-1684`): a real accessory in `armor[3]` and a
    /// vanity one in `armor[13]` trade on the vanity cell's right-click —
    /// `CanEquipAccessoryInSlot` (`:3531-3538`) admits the swap while no
    /// OTHER slot carries an incompatible accessory.
    #[test]
    fn s1_p_i53_right_click_swaps_the_accessory_vanity_pair() {
        let mut app = inv_app();
        app.equip.armor[3] = slot(15, 1); // Copper Watch — real accessory
        app.equip.armor[13] = slot(16, 1); // Aglet — vanity accessory
        app.mouse_pos = equip_mouse(&app, 0, 11, 13);
        app.ui_right_press = true;
        app.handle_inventory_clicks();
        assert_eq!(app.equip.armor[3], slot(16, 1), "the real row took the aglet");
        assert_eq!(app.equip.armor[13], slot(15, 1), "the vanity row the watch");
    }

    /// S1-P-i53 — the ammo row (ctx 2) has NO `SwapEquip` arm: the C#'s
    /// `:1685-1697` switch covers contexts 0/3/4/32 only (`InventoryAmmo =
    /// 2`, `ItemSlot.cs:48`), so a single ammo stack right-clicks into the
    /// paced pickup — no "quick-equip" jump exists in the source.
    #[test]
    fn s1_p_i53_the_ammo_row_right_click_only_splits() {
        let mut app = inv_app();
        app.equip.ammo[0] = slot(40, 1); // Wooden Arrow
        app.mouse_pos = AMMO_SLOT_0;
        app.ui_right_press = true;
        app.handle_inventory_clicks();
        assert_eq!(app.mouse_item, slot(40, 1), "one onto the cursor (`:1707`)");
        assert_eq!(app.equip.ammo[0].count, 0, "the cell drained, nothing equipped");
        assert_eq!(app.equip_page_selected, 0, "no equip-page jump");
    }
}

#[cfg(test)]
mod s1_h_7_tests {
    use super::*;
    use crate::craft;

    fn craft_app() -> App {
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        app.crafting_open = true;
        app.inventory_open = true;
        app.player = Some(crate::player::Player::spawn(&app.world));
        app
    }

    /// The mouse over the TORCH recipe's cell (the grid origin + the 44 px
    /// pitch — `ui::crafting::hit_test`'s geometry).
    fn torch_mouse(app: &App) -> Vec2 {
        let (sw, sh) = app.screen_size();
        let screen = Vec2::new(sw, sh);
        let adj = craft::scan_adjacent(&app.world, app.player.as_ref().unwrap(), (5, 3));
        let zones = craft::Zones::default();
        let in_range = craft::chests_in_range(
            &app.world,
            app.player.as_ref().unwrap(),
            &app.world_chests,
            false,
        );
        let available = craft::available(
            &app.inventory,
            &app.equip,
            &app.world_chests,
            &in_range,
            &adj,
            &zones,
            None,
            None,
        );
        let pos = available
            .iter()
            .position(|&r| craft::RECIPES[r].create.0 == 8)
            .expect("the torch recipe is available");
        let l = ui::crafting::layout(screen, 0.0, false);
        let [x, y, _, _] = ui::crafting::recipe_cell(&l, pos, 0.0);
        Vec2::new(x + 22.0, y + 22.0)
    }

    #[test]
    fn the_first_click_focuses_and_the_next_crafts() {
        // `Main.cs:42295-42299`: the unfocused click selects
        // (`focusRecipe`, `stackSplit = 15`, the lock, MenuTick) and the
        // focused click crafts (`:42266-42290`).
        let mut app = craft_app();
        // Torch x3 = 1 gel + 1 wood, by hand (recipes_gen row 0).
        app.inventory[0] = crate::item::ItemSlot {
            kind: 9,
            name: "Wood",
            count: 30,
            prefix: 0,
        };
        app.inventory[1] = crate::item::ItemSlot {
            kind: 23,
            name: "Gel",
            count: 30,
            prefix: 0,
        };
        let mouse = torch_mouse(&app);
        app.mouse_pos = mouse;
        // The torch (row 0) starts UNfocused — `Main.focusRecipe` defaults
        // to 0 and the torch IS row 0, so the test parks the focus on
        // another row to exercise the select step (`Main.cs:42295-42299`).
        app.focus_recipe = 1;
        // The first press: the focus.
        app.ui_left_press = true;
        app.mine_down = true;
        assert!(app.handle_crafting_clicks());
        assert!(app.prevent_craft_click, "the focus click's lock");
        assert_eq!(app.stack_split, 15);
        assert!(
            !app.inventory.iter().any(|s| s.kind == 8),
            "nothing crafted yet"
        );
        // The second press: the release resets the lock, the held frames
        // decay `stackSplit` (`frame.rs`'s per-tick decrement) below the
        // pacing gate.
        app.mine_down = false;
        app.handle_crafting_clicks();
        app.stack_split = 0;
        app.mine_down = true;
        app.ui_left_press = true;
        assert!(app.handle_crafting_clicks());
        let torches = app
            .inventory
            .iter()
            .find(|s| s.kind == 8)
            .expect("the focused click crafted");
        assert_eq!(torches.count, 3, "one craft = three torches");
    }

    #[test]
    fn shift_crafts_ten_at_once() {
        // `Main.cs:42272-42274`: `num *= 10` behind `ItemSlot.ShiftInUse`.
        let mut app = craft_app();
        app.inventory[0] = crate::item::ItemSlot {
            kind: 9,
            name: "Wood",
            count: 300,
            prefix: 0,
        };
        app.inventory[1] = crate::item::ItemSlot {
            kind: 23,
            name: "Gel",
            count: 300,
            prefix: 0,
        };
        let mouse = torch_mouse(&app);
        app.mouse_pos = mouse;
        // The torch (row 0) is pre-focused (`Main.focusRecipe` defaults to
        // 0) — one shift-click crafts the x10
        // (`Main.cs:42272-42274`).
        app.keys.insert(KeyCode::ShiftLeft);
        app.ui_left_press = true;
        app.mine_down = true;
        app.handle_crafting_clicks();
        let torches = app
            .inventory
            .iter()
            .find(|s| s.kind == 8)
            .expect("the x10 craft");
        assert_eq!(torches.count, 30, "the shift x10 = 10 crafts of 3");
    }

    #[test]
    fn the_qty_craft_respects_the_ingredients() {
        // The request loop's net effect: a partial ingredient set crafts as
        // many as it can.
        let mut app = craft_app();
        app.inventory[0] = crate::item::ItemSlot {
            kind: 9,
            name: "Wood",
            count: 3,
            prefix: 0,
        };
        app.inventory[1] = crate::item::ItemSlot {
            kind: 23,
            name: "Gel",
            count: 300,
            prefix: 0,
        };
        let mouse = torch_mouse(&app);
        app.mouse_pos = mouse;
        app.keys.insert(KeyCode::ShiftLeft);
        app.ui_left_press = true;
        app.mine_down = true;
        app.handle_crafting_clicks();
        let torches = app
            .inventory
            .iter()
            .find(|s| s.kind == 8)
            .expect("the partial craft");
        assert_eq!(torches.count, 9, "only 3 wood -> 3 crafts of 3");
    }

    /// W4-f23 doc-sweep: the RIGHT button crafts exactly like the left —
    /// `HandleCraftSlot`'s select arm (`NewCraftingUI.cs:505`) and
    /// `HoverOverCraftingItemButton` (`Main.cs:42254-42255`) OR both
    /// buttons. There is no craft-max: the bulk arm is Shift x10 only
    /// (`:42272-42274`).
    #[test]
    fn the_right_button_selects_and_crafts_like_the_left() {
        let mut app = craft_app();
        app.inventory[0] = crate::item::ItemSlot {
            kind: 9,
            name: "Wood",
            count: 30,
            prefix: 0,
        };
        app.inventory[1] = crate::item::ItemSlot {
            kind: 23,
            name: "Gel",
            count: 30,
            prefix: 0,
        };
        app.mouse_pos = torch_mouse(&app);
        app.focus_recipe = 1; // park the focus off the torch row
        // The right press edge selects (`:505`'s right arm).
        app.ui_right_press = true;
        app.place_down = true;
        assert!(app.handle_crafting_clicks());
        assert_eq!(app.focus_recipe, 0, "the right click focused the torch");
        assert!(app.prevent_craft_click);
        assert_eq!(app.stack_split, 15);
        assert!(!app.ui_right_press, "the edge is consumed");
        assert!(
            !app.inventory.iter().any(|s| s.kind == 8),
            "the select step crafts nothing"
        );
        // Release + re-press: the focused right click crafts ONE recipe
        // (num = superFastStack + 1 = 1 — no craft-max).
        app.place_down = false;
        app.handle_crafting_clicks();
        assert!(!app.prevent_craft_click, "both buttons released reset the lock");
        app.stack_split = 0;
        app.place_down = true;
        app.ui_right_press = true;
        assert!(app.handle_crafting_clicks());
        let torches = app
            .inventory
            .iter()
            .find(|s| s.kind == 8)
            .expect("the right click crafted");
        assert_eq!(torches.count, 3, "one craft = three torches, not the max");
    }

    /// W4-f23 doc-sweep: the HELD right button keeps the held-repeat alive
    /// (`flag2 = mouseLeft || mouseRight`, `Main.cs:42255`) — the lock and
    /// the recipe pin survive while either button is down (`:42301-42305`).
    #[test]
    fn a_held_right_button_keeps_the_craft_locks() {
        let mut app = craft_app();
        app.prevent_craft_click = true;
        app.current_recipe_being_crafted = 5;
        app.place_down = true; // right held, left up
        app.mine_down = false;
        app.stack_split = 0;
        // No press edge and no pacing: the handler walks to the grid and
        // returns without resetting the locks (the `!flag2` arm never runs).
        app.mouse_pos = Vec2::new(-100.0, -100.0); // off every cell
        app.handle_crafting_clicks();
        assert!(app.prevent_craft_click, "the held right keeps the lock");
        assert_eq!(app.current_recipe_being_crafted, 5);
        // Both up: the reset arm fires.
        app.place_down = false;
        app.handle_crafting_clicks();
        assert!(!app.prevent_craft_click);
        assert_eq!(app.current_recipe_being_crafted, -1);
    }

    /// The mouse over the filter tab `i` — `ui::crafting::filter_button_rect`'s
    /// square (`UICreativeItemsInfiniteFilteringOptions.cs:56-57`).
    fn filter_mouse(app: &App, i: usize) -> Vec2 {
        let (sw, sh) = app.screen_size();
        let l = ui::crafting::layout(Vec2::new(sw, sh), 0.0, false);
        let b = ui::crafting::filter_button_rect(l.filter_strip, i);
        Vec2::new(b[0] + 20.0, b[1] + 20.0)
    }

    /// W4-f23 doc-sweep: the refocus-on-craft wiring — after a craft drains
    /// the ingredients, the draw-path rebuild (`frame/draw.rs`'s
    /// `try_refocusing_recipe` call, `Recipe.cs:232-233`/`:257`) moves the
    /// focus off the vanished recipe onto the last row. The witness runs
    /// the same capture + refocus sequence the draw path runs.
    #[test]
    fn crafting_the_last_ingredients_refocuses_to_the_last_row() {
        let mut app = craft_app();
        // Exactly one torch craft's worth: 1 wood + 1 gel.
        app.inventory[0] = crate::item::ItemSlot {
            kind: 9,
            name: "Wood",
            count: 1,
            prefix: 0,
        };
        app.inventory[1] = crate::item::ItemSlot {
            kind: 23,
            name: "Gel",
            count: 1,
            prefix: 0,
        };
        let adj = craft::scan_adjacent(&app.world, app.player.as_ref().unwrap(), (5, 3));
        let zones = craft::Zones::default();
        let in_range = craft::chests_in_range(
            &app.world,
            app.player.as_ref().unwrap(),
            &app.world_chests,
            false,
        );
        let before = craft::available(
            &app.inventory,
            &app.equip,
            &app.world_chests,
            &in_range,
            &adj,
            &zones,
            None,
            None,
        );
        let torch_pos = before
            .iter()
            .position(|&r| craft::RECIPES[r].create.0 == 8)
            .expect("the torch recipe is available");
        // The draw path's snapshot + focus before the craft.
        app.craft_last_available = before.clone();
        app.focus_recipe = torch_pos as i32;
        // Craft the torch (the focused click).
        app.mouse_pos = torch_mouse(&app);
        app.ui_left_press = true;
        app.mine_down = true;
        assert!(app.handle_crafting_clicks());
        assert!(app.inventory.iter().any(|s| s.kind == 8), "the craft ran");
        // The next frame's rebuild: the torch left the list (no gel/wood
        // left), the refocus tail captures the OLD recipe from the
        // snapshot and pins the focus to the new last row.
        let after = craft::available(
            &app.inventory,
            &app.equip,
            &app.world_chests,
            &in_range,
            &adj,
            &zones,
            None,
            None,
        );
        assert!(
            !after.contains(&before[torch_pos]),
            "the torch recipe vanished with its ingredients"
        );
        let old_recipe = app.craft_last_available[app.focus_recipe as usize];
        app.focus_recipe =
            craft::try_refocusing_recipe(&after, old_recipe, app.focus_recipe);
        assert_eq!(
            app.focus_recipe,
            (after.len() as i32 - 1).max(0),
            "the focus rides the last row (`Recipe.cs:487-494`)"
        );
    }

    /// The shown grid — the same call `handle_crafting_clicks` makes.
    fn shown(app: &App) -> Vec<usize> {
        let (sw, sh) = app.screen_size();
        let _ = Vec2::new(sw, sh);
        let adj = craft::scan_adjacent(&app.world, app.player.as_ref().unwrap(), (5, 3));
        let in_range = craft::chests_in_range(
            &app.world,
            app.player.as_ref().unwrap(),
            &app.world_chests,
            false,
        );
        app.craft_filters.shown(craft::available(
            &app.inventory,
            &app.equip,
            &app.world_chests,
            &in_range,
            &adj,
            &craft::Zones::default(),
            None,
            None,
        ))
    }

    #[test]
    fn a_filter_tab_click_narrows_the_grid_and_re_click_restores_it() {
        // `singleFilterButtonClick` (`UICreativeItemsInfiniteFilteringOptions.cs:80-100`):
        // the clicked tab becomes the only active one, `OnClickingOption` ->
        // `ResetRecipes` (`NewCraftingUI.cs:391`) re-derives the shown list, and
        // the grid then holds only the rows whose product passes
        // (`:596-611`, `:630-641`).
        let mut app = craft_app();
        app.inventory[0] = crate::item::ItemSlot {
            kind: 9,
            name: "Wood",
            count: 30,
            prefix: 0,
        };
        app.inventory[1] = crate::item::ItemSlot {
            kind: 23,
            name: "Gel",
            count: 30,
            prefix: 0,
        };
        let all = shown(&app);
        assert!(
            all.iter().any(|&r| craft::RECIPES[r].create.0 == 8),
            "the torch recipe starts shown"
        );
        // Tab 0 is `Weapon` (`NewCraftingUI.cs:284`) — a torch deals no damage.
        app.mouse_pos = filter_mouse(&app, 0);
        app.ui_left_press = true;
        app.mine_down = true;
        assert!(app.handle_crafting_clicks(), "the tab click is consumed");
        assert_eq!(app.craft_filters.active, Some(0));
        assert!(!app.ui_left_press, "the press was consumed");
        let weapons = shown(&app);
        assert!(weapons
            .iter()
            .all(|&r| craft::RECIPES[r].create.0 != 8));
        assert!(weapons.len() < all.len() || all.is_empty());
        // Re-clicking the active tab clears it (`:89-93` + `EntryFilterer.ToggleFilter`).
        app.mine_down = false;
        app.handle_crafting_clicks();
        app.ui_left_press = true;
        app.mine_down = true;
        assert!(app.handle_crafting_clicks());
        assert_eq!(app.craft_filters.active, None);
        assert_eq!(shown(&app).len(), all.len(), "the grid is restored");
    }

    #[test]
    fn a_filter_click_does_not_change_the_focused_recipe() {
        // `Main.focusRecipe` is an `availableRecipe` index (`NewCraftingUI.cs:610`,
        // `:622`), so narrowing the shown grid must leave it alone.
        let mut app = craft_app();
        app.inventory[0] = crate::item::ItemSlot {
            kind: 9,
            name: "Wood",
            count: 30,
            prefix: 0,
        };
        app.inventory[1] = crate::item::ItemSlot {
            kind: 23,
            name: "Gel",
            count: 30,
            prefix: 0,
        };
        app.focus_recipe = 0;
        app.mouse_pos = filter_mouse(&app, 3); // BuildingBlock
        app.ui_left_press = true;
        app.mine_down = true;
        app.handle_crafting_clicks();
        assert_eq!(app.craft_filters.active, Some(3));
        assert_eq!(app.focus_recipe, 0, "the focus index space is preserved");
    }

    /// W4-f23 deviation fix: with a filter active the shown position and
    /// the RAW available position diverge (`NewCraftingUI.cs:596-611`) —
    /// the focus click stores `entry.availableIndex` (`:510`), the raw
    /// position, NOT the recipe-table index nor the grid position.
    #[test]
    fn a_filtered_focus_click_stores_the_raw_available_position() {
        let mut app = craft_app();
        app.inventory[0] = crate::item::ItemSlot {
            kind: 9,
            name: "Wood",
            count: 30,
            prefix: 0,
        };
        app.inventory[1] = crate::item::ItemSlot {
            kind: 23,
            name: "Gel",
            count: 30,
            prefix: 0,
        };
        // The BuildingBlock tab hides the torch (a Consumable), so the raw
        // and shown lists differ from row 0 on. A work bench beside the
        // player opens the wooden-furniture rows too.
        let p = app.player.as_ref().unwrap();
        let (bx, by) = ((p.x / 16.0) as i32 + 2, (p.y / 16.0) as i32);
        let ti = app.world.index(bx, by);
        app.world.tiles[ti].kind = 18;
        app.world.tiles[ti].flags |= tr_core::tile::FLAG_ACTIVE;
        app.craft_filters.click(3);
        let adj = craft::scan_adjacent(&app.world, app.player.as_ref().unwrap(), (5, 3));
        let raw = craft::available(
            &app.inventory,
            &app.equip,
            &app.world_chests,
            &[],
            &adj,
            &craft::Zones::default(),
            None,
            None,
        );
        let grid = shown(&app);
        assert!(!grid.is_empty(), "some blocks are craftable from wood");
        assert!(grid.len() < raw.len(), "the torch rows are filtered out");
        assert_ne!(grid[0], raw[0], "the shown list starts past the torch");
        // Click the FIRST SHOWN cell: the focus must land on its RAW
        // position, not 0 (the grid position) nor the recipe-table index.
        let (sw, sh) = app.screen_size();
        let l = ui::crafting::layout(Vec2::new(sw, sh), 0.0, false);
        let [x, y, _, _] = ui::crafting::recipe_cell(&l, 0, 0.0);
        app.mouse_pos = Vec2::new(x + 22.0, y + 22.0);
        app.focus_recipe = -1; // nothing focused
        app.ui_left_press = true;
        app.mine_down = true;
        assert!(app.handle_crafting_clicks(), "the focus click is consumed");
        let expected = raw.iter().position(|&r| r == grid[0]).unwrap() as i32;
        assert_eq!(app.focus_recipe, expected, "the RAW available position");
        assert_ne!(app.focus_recipe, grid[0] as i32, "not the recipe index");
        assert_eq!(app.stack_split, 15);
        assert!(app.prevent_craft_click);
        // The draw-side derivation agrees: available[focus] is the clicked
        // recipe (`OnActivate` `:904`'s index space).
        assert_eq!(raw[app.focus_recipe as usize], grid[0]);
    }

    /// The mouse over a recipe's cell (the grid origin + the 44 px pitch —
    /// `ui::crafting::hit_test`'s geometry) for the given created item.
    fn recipe_mouse(app: &App, create: u16) -> Vec2 {
        let (sw, sh) = app.screen_size();
        let screen = Vec2::new(sw, sh);
        let adj = craft::scan_adjacent(&app.world, app.player.as_ref().unwrap(), (5, 3));
        let zones = craft::Zones::default();
        let in_range = craft::chests_in_range(
            &app.world,
            app.player.as_ref().unwrap(),
            &app.world_chests,
            false,
        );
        let available = craft::available(
            &app.inventory,
            &app.equip,
            &app.world_chests,
            &in_range,
            &adj,
            &zones,
            None,
            None,
        );
        let pos = available
            .iter()
            .position(|&r| craft::RECIPES[r].create.0 == create)
            .expect("the recipe is available");
        let l = ui::crafting::layout(screen, 0.0, false);
        let [x, y, _, _] = ui::crafting::recipe_cell(&l, pos, 0.0);
        Vec2::new(x + 22.0, y + 22.0)
    }

    #[test]
    fn s1_l_11d_a_crafted_weapon_keeps_its_rolled_prefix() {
        // `CraftingRequests.CreateResult` (`CraftingRequests.cs:122-131`): the
        // crafted item takes the natural prefix roll (`Item.Prefix(-1)`), and
        // the grant keeps it — two crafted items with different prefixes take
        // two slots (`Item.CanStack`, `Item.cs:49865-49872`).
        let mut app = craft_app();
        let p = app.player.as_ref().unwrap();
        let bx = (p.x / 16.0) as i32 + 2;
        let by = (p.y / 16.0) as i32;
        // The anvil (16) next to the player — the recipe's station.
        let idx = app.world.index(bx, by);
        app.world.tiles[idx].kind = 16;
        app.world.tiles[idx].flags |= tr_core::tile::FLAG_ACTIVE;
        // 10 copper bars (22) + 3 wood for the pickaxe (1).
        app.inventory[0] = crate::item::ItemSlot {
            kind: 22,
            name: "Copper Bar",
            count: 10,
            prefix: 0,
        };
        app.inventory[1] = crate::item::ItemSlot {
            kind: 9,
            name: "Wood",
            count: 3,
            prefix: 0,
        };
        // The engine's own roll, replayed from the same stream state: the
        // grant must carry exactly what `Item.Prefix(-1)` rolled. The craft
        // interaction is menu-contexted in the C# (`Main.cs` ~17180, outside
        // every swap), so the roll rides the bare-`Main.rand` stand-in
        // `global()` — the S1-A-11 re-point of the former `App::prefix_rng`.
        // The registry is seeded 11 so the roll is a non-zero prefix (the
        // world seed 0 lands `Prefix`'s 25% no-prefix arm for the pickaxe).
        app.rngs = crate::app::rng::NamedRngs::new(11);
        let mut expect_rng = app.rngs.global().clone();
        let expected = crate::prefix::roll_prefix(1, -1, &mut expect_rng);
        let mouse = recipe_mouse(&app, 1);
        app.mouse_pos = mouse;
        app.focus_recipe = 1;
        // The first press focuses, the second crafts.
        app.ui_left_press = true;
        app.mine_down = true;
        app.handle_crafting_clicks();
        app.mine_down = false;
        app.handle_crafting_clicks();
        app.stack_split = 0;
        app.mine_down = true;
        app.ui_left_press = true;
        app.handle_crafting_clicks();
        let pick = app
            .inventory
            .iter()
            .find(|s| s.kind == 1)
            .expect("the crafted pickaxe");
        assert_ne!(expected, 0, "the pinned roll is a prefix");
        assert_eq!(
            pick.prefix, expected,
            "the crafted item keeps the rolled prefix"
        );
    }

    /// W4-f23 closeout item 4 — the nearby-chests toggle's full click
    /// semantics (`CraftingUI.DrawCraftFromNearbyChestsToggle`,
    /// `CraftingUI.cs:346-352`): the LEFT press flips
    /// `Player.Settings.CraftFromNearbyChests`, plays sound 12 (`:350`) and
    /// persists (`Main.SaveSettings`, `:351`). The toggle is LEFT-only
    /// (`:346` reads `mouseLeft && mouseLeftRelease`), unlike the recipe
    /// cells' either-button arms.
    #[test]
    fn the_nearby_toggle_flips_the_setting_and_plays_the_tick() {
        let mut app = craft_app();
        app.settings.craft_from_nearby_chests = true;
        let (sw, sh) = app.screen_size();
        let l = ui::crafting::layout(Vec2::new(sw, sh), 0.0, false);
        app.mouse_pos = Vec2::new(l.nearby_toggle[0], l.nearby_toggle[1]);
        let before = app.audio.played_count();
        app.ui_left_press = true;
        app.mine_down = true;
        assert!(app.handle_crafting_clicks(), "the toggle click is consumed");
        assert!(!app.settings.craft_from_nearby_chests, "the setting flipped");
        assert!(!app.ui_left_press, "the press was consumed");
        assert_eq!(app.audio.played_count(), before + 1, "sound 12 (`:350`)");
        // A second click flips it back.
        app.mine_down = false;
        app.handle_crafting_clicks();
        app.ui_left_press = true;
        app.mine_down = true;
        assert!(app.handle_crafting_clicks());
        assert!(app.settings.craft_from_nearby_chests, "flipped back on");
    }

    /// The toggle is LEFT-only: a held RIGHT button over the toggle does
    /// NOT flip the setting (`CraftingUI.cs:346` reads `mouseLeft` only —
    /// the W4-f23 doc-sweep 4b parity must not let a held right flip it).
    #[test]
    fn a_held_right_does_not_flip_the_nearby_toggle() {
        let mut app = craft_app();
        app.settings.craft_from_nearby_chests = true;
        let (sw, sh) = app.screen_size();
        let l = ui::crafting::layout(Vec2::new(sw, sh), 0.0, false);
        app.mouse_pos = Vec2::new(l.nearby_toggle[0], l.nearby_toggle[1]);
        app.place_down = true; // right held, left NOT down
        app.ui_right_press = true;
        app.handle_crafting_clicks();
        assert!(
            app.settings.craft_from_nearby_chests,
            "the right button never flips the LEFT-only toggle"
        );
    }

    /// W4-f23 closeout item 3 — the craft cue is `CraftingEffects.OnCraft`'s
    /// `SoundEngine.PlaySound(7)` (`CraftingEffects.cs:51`), run
    /// UNCONDITIONALLY after `CraftingRequests.CraftItem`'s loop
    /// (`CraftingRequests.cs:119`) — even when the loop crafted nothing. A
    /// focused-row attempt the ingredients no longer allow still plays the
    /// cue (the old grant-gated port dropped it).
    #[test]
    fn the_craft_cue_plays_even_when_the_attempt_crafts_nothing() {
        let mut app = craft_app();
        // Stock the torch, focus it, then drain the gel so the focused
        // attempt can craft nothing.
        app.inventory[0] = crate::item::ItemSlot { kind: 9, name: "Wood", count: 30, prefix: 0 };
        app.inventory[1] = crate::item::ItemSlot { kind: 23, name: "Gel", count: 30, prefix: 0 };
        let mouse = torch_mouse(&app);
        app.mouse_pos = mouse;
        app.focus_recipe = 1;
        // Focus the torch (row 0).
        app.ui_left_press = true;
        app.mine_down = true;
        app.handle_crafting_clicks();
        app.mine_down = false;
        app.handle_crafting_clicks();
        // Drain the gel — the torch is no longer craftable, but the row
        // stays focused (the click path reads the live available list).
        app.inventory[1].count = 0;
        app.stack_split = 0;
        app.mine_down = true;
        app.ui_left_press = true;
        let before = app.audio.played_count();
        // The focused-row attempt: `try_craft_qty` returns empty (no gel),
        // but `OnCraft`'s cue still fires.
        app.handle_crafting_clicks();
        assert!(
            !app.inventory.iter().any(|s| s.kind == 8),
            "nothing crafted (no gel)"
        );
        assert_eq!(
            app.audio.played_count(),
            before + 1,
            "the craft-attempt cue plays even on a no-op (`CraftingRequests.cs:119`)"
        );
    }
}

#[cfg(test)]
mod housing_query_tests {
    use super::*;
    use crate::npc::NpcKind;

    fn slot(kind: u16, count: u32) -> ItemSlot {
        ItemSlot { kind, name: registry::name(kind), count, prefix: 0 }
    }

    fn inv_app() -> App {
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        app.inventory_open = true;
        app.player = Some(crate::player::Player::spawn(&app.world));
        app
    }

    // ------------------------------------------------------------------
    // S1-E-11d — the `Main.mouseNPC` housing query / relocation model.
    // ------------------------------------------------------------------

    /// The housing page's cells at the deterministic 1280x720 screen
    /// (`m_h = 0`): `equip_page_base_y` = 174, the query cell at
    /// `(1280 - 64 - 28, 174)` = `(1188, 174)`, 44.2 px wide (52 * 0.85).
    fn housing_cells(app: &App) -> Vec<ui::housing::Cell> {
        let (sw, sh) = app.screen_size();
        let base_y = ui::inventory::equip_page_base_y(sh, app.main_m_h());
        ui::housing::layout(
            &ui::housing::heads_for(&app.npcs),
            [sw, sh],
            base_y,
            0.85,
        )
    }

    /// An open housing page with one housed Guide and two furnished rooms.
    fn housing_app() -> App {
        let mut app = inv_app();
        let mut world = crate::test_world_helpers::ground();
        crate::test_world_helpers::build_furnished_room(&mut world, 60, 100, 12, 9);
        crate::test_world_helpers::build_furnished_room(&mut world, 90, 100, 12, 9);
        app.world = world;
        app.equip_page = 1;
        app.equip_page_selected = 1;
        let mut guide = crate::npc::Npc::new(NpcKind::Guide, 65.0 * 16.0, 106.0 * 16.0, 1, 0.0);
        guide.home = Some((65, 106));
        guide.hp = guide.max_hp;
        app.npcs.push(guide);
        app.town_rooms.set_room(22, 65, 106);
        app
    }

    /// Aim the pointer at a world tile through the camera + zoom
    /// (`Main.cs:45232-45235`'s inverse).
    fn aim_at_tile(app: &mut App, tx: i32, ty: i32) {
        let (sw, sh) = app.screen_size();
        let wx = tx as f32 * 16.0 + 8.0;
        let wy = ty as f32 * 16.0 + 8.0;
        app.mouse_pos = Vec2::new(
            (wx - app.camera.x) * app.zoom + sw * 0.5,
            (wy - app.camera.y) * app.zoom + sh * 0.5,
        );
    }

    #[test]
    fn the_housing_query_cell_arms_the_query_and_the_world_click_reports() {
        let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        crate::lang::init(tr_core::localization::GameCulture::English);
        let mut app = housing_app();
        // Click the "?" cell (`Main.cs:42061-42063`).
        let cells = housing_cells(&app);
        let query = cells.iter().find(|c| c.head == ui::housing::HOUSING_QUERY).unwrap();
        app.mouse_pos = Vec2::new(query.x + 10.0, query.y + 10.0);
        app.ui_left_press = true;
        app.handle_inventory_clicks();
        assert_eq!((app.mouse_npc_index, app.mouse_npc_type), (-1, 0));
        // The world click on the second (empty) furnished room: "This
        // housing is suitable." (`Main.PerformHousingCheck`,
        // `Main.cs:45162-45170`). The Guide's own room is occupied for the
        // query — `MoveTownNPC` passes `n = -1`, so `ScoreRoom(-1, type)`
        // counts his home as an occupant.
        app.camera = Vec2::new(95.0 * 16.0, 105.0 * 16.0);
        aim_at_tile(&mut app, 95, 105);
        app.ui_left_press = true;
        app.handle_inventory_clicks();
        assert_eq!(app.chat.messages[0].text, "This housing is suitable.");
        // The query head stays armed (the C# keeps it until right-click).
        assert_eq!((app.mouse_npc_index, app.mouse_npc_type), (-1, 0));
        // The Guide's own room reports the occupancy (`WorldGen.cs:4807-4809`).
        aim_at_tile(&mut app, 65, 105);
        app.ui_left_press = true;
        app.handle_inventory_clicks();
        assert_eq!(app.chat.messages[0].text, "This housing is already occupied.");
        // A solid-tile click reports the failure (`WorldGen.cs:4707-4708`).
        aim_at_tile(&mut app, 30, 130);
        app.ui_left_press = true;
        app.handle_inventory_clicks();
        assert_eq!(app.chat.messages[0].text, "This is a solid block!");
        // The right release drops the head (`Main.cs:45223-45227`).
        app.ui_right_press = true;
        app.handle_inventory_clicks();
        assert_eq!((app.mouse_npc_index, app.mouse_npc_type), (-1, -1));
    }

    #[test]
    fn the_npc_head_relocates_the_guide_to_the_clicked_room() {
        let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        crate::lang::init(tr_core::localization::GameCulture::English);
        let mut app = housing_app();
        // Click the Guide's head cell (`Main.cs:42067`).
        let cells = housing_cells(&app);
        let guide_cell = cells.iter().find(|c| c.npc == Some(0)).unwrap();
        app.mouse_pos = Vec2::new(guide_cell.x + 10.0, guide_cell.y + 10.0);
        app.ui_left_press = true;
        app.handle_inventory_clicks();
        assert_eq!((app.mouse_npc_index, app.mouse_npc_type), (0, 22));
        // Click into the second room (`Main.TryMovingNPC`, `:45172-45180`).
        app.camera = Vec2::new(95.0 * 16.0, 105.0 * 16.0);
        aim_at_tile(&mut app, 95, 105);
        app.npcs[0].look_for_home_timeout = 900;
        app.ui_left_press = true;
        app.handle_inventory_clicks();
        let (hx, hy) = app.npcs[0].home.expect("rehoused");
        assert!((91..=100).contains(&hx), "the scored spot: {hx}");
        assert!((101..=108).contains(&hy), "hy {hy}");
        assert_eq!(app.npcs[0].look_for_home_timeout, 0);
        assert_eq!(app.town_rooms.has_room(22), Some((hx, hy)));
        // The head cleared on success (`:45176`).
        assert_eq!((app.mouse_npc_index, app.mouse_npc_type), (-1, -1));
    }

    #[test]
    fn the_carried_head_drops_on_a_carried_item_and_a_closed_inventory() {
        let mut app = housing_app();
        app.mouse_npc_index = 0;
        app.mouse_npc_type = 22;
        // A carried item drops the head (`Main.cs:45185-45187`).
        app.mouse_item = slot(71, 5);
        app.refresh_mouse_npc();
        assert_eq!((app.mouse_npc_index, app.mouse_npc_type), (-1, -1));
        // A closed inventory drops it (`Main.cs:45666`).
        app.mouse_item = EMPTY_SLOT;
        app.mouse_npc_index = -1;
        app.mouse_npc_type = 0;
        app.inventory_open = false;
        app.refresh_mouse_npc();
        assert_eq!((app.mouse_npc_index, app.mouse_npc_type), (-1, -1));
        // A dead/changed NPC downgrades to the query (`Main.cs:45205-45211`).
        app.inventory_open = true;
        app.mouse_npc_index = 0;
        app.mouse_npc_type = 17; // the slot holds a Guide, not a Merchant
        app.refresh_mouse_npc();
        assert_eq!((app.mouse_npc_index, app.mouse_npc_type), (-1, 0));
    }
}

/// S1-P-i44 — the shop CLICK witnesses: the ctx-15 grid dispatch
/// (`Main.cs:41811-41814` → `ItemSlot.HandleShopSlot`), the cursor-10
/// Shift-sell (`ItemSlot.cs:851-869` + `GetSellOrTrash` `:198-209`) and
/// the shop grid's geometry (`Main.cs:41807-41811`). Written, not run.
#[cfg(test)]
mod s1_p_i44_shop_click_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;

    /// Inside the shop grid's cell (0, 0) — the drawn 52·0.755 px cell at
    /// (73, 258) (`Main.cs:41807-41808`, `ui::inventory::shop_slot_pos`).
    const SHOP_SLOT_0: Vec2 = Vec2::new(90.0, 270.0);
    /// Inside the shop grid's cell (1, 0) — x = (int)(73 + 56·0.755) = 115.
    const SHOP_SLOT_1: Vec2 = Vec2::new(130.0, 270.0);

    fn shop_app() -> App {
        let mut app = flow_app(None, vec![], vec![]);
        app.inventory_open = true;
        app.player = Some(crate::player::Player::spawn(&app.world));
        app.shop_npc = Some(0);
        app.shop_stock =
            crate::shop::resolve_stock_entries(1, &crate::shop::ShopFlags::default(), &[]);
        app.shop_mood = 1.0;
        app.coins = 1_000_000;
        app
    }

    #[test]
    fn the_shop_grid_geometry_matches_the_draw() {
        // `Main.cs:41807-41808`: `(int)(73 + i*56*0.755)`,
        // `(int)(258 + j*56*0.755)` — the same truncation the chest panel
        // uses (`ChestUI.cs:379-380`).
        assert_eq!(ui::inventory::shop_slot_pos(0, 0), Vec2::new(73.0, 258.0));
        assert_eq!(ui::inventory::shop_slot_pos(1, 0), Vec2::new(115.0, 258.0));
        assert_eq!(ui::inventory::shop_slot_pos(0, 1), Vec2::new(73.0, 300.0));
        // The hit test is inclusive on all four edges (`:41811`).
        assert_eq!(ui::inventory::shop_slot_at(Vec2::new(73.0, 258.0), 52.0), Some(0));
        assert_eq!(ui::inventory::shop_slot_at(SHOP_SLOT_1, 52.0), Some(1));
        // The last cell (9, 3): x = (int)(73 + 9·42.28) = 453, y = 384.
        assert_eq!(ui::inventory::shop_slot_pos(9, 3), Vec2::new(453.0, 384.0));
        assert_eq!(ui::inventory::shop_slot_at(Vec2::new(460.0, 390.0), 52.0), Some(39));
        // Off-grid: nothing.
        assert_eq!(ui::inventory::shop_slot_at(Vec2::new(60.0, 270.0), 52.0), None);
        assert_eq!(ui::inventory::shop_slot_at(Vec2::new(90.0, 440.0), 52.0), None);
    }

    #[test]
    fn the_left_click_buys_one_unit_onto_the_cursor() {
        // `ItemSlot.HandleShopSlot` (`:2053-2095`): one unit per press at
        // superFastStack 0; the charge is the row's buying value; the
        // stack-split cooldown arms (`RefreshStackSplitCooldown`, `:2089`).
        let mut app = shop_app();
        app.mouse_pos = SHOP_SLOT_0;
        app.ui_left_press = true;
        app.handle_inventory_clicks();
        assert_eq!((app.mouse_item.kind, app.mouse_item.count), (88, 1));
        assert_eq!(app.coins, 1_000_000 - 40000);
        assert_eq!(app.stack_split, 30, "the 0 → 30 arm (`:1776-1778`)");
        assert_eq!(app.shop_sellback.get_amount(88, 0), 1);
        // The cooldown gates the next press (`:2043`'s stackSplit <= 1).
        app.ui_left_press = true;
        app.handle_inventory_clicks();
        assert_eq!(app.mouse_item.count, 1, "the split gate holds the repeat");
        // Once the cooldown decays the buy repeats.
        app.stack_split = 0;
        app.ui_left_press = true;
        app.handle_inventory_clicks();
        assert_eq!(app.mouse_item.count, 2);
        assert_eq!(app.coins, 1_000_000 - 80000);
    }

    #[test]
    fn the_right_click_buys_paced() {
        // `RightClick`'s ctx-15 arm (`ItemSlot.cs:1648-1651`): the held
        // right button buys through the same HandleShopSlot gate.
        let mut app = shop_app();
        app.mouse_pos = SHOP_SLOT_0;
        app.place_down = true;
        app.handle_inventory_clicks();
        assert_eq!((app.mouse_item.kind, app.mouse_item.count), (88, 1));
        assert_eq!(app.coins, 1_000_000 - 40000);
    }

    #[test]
    fn the_full_cursor_refuses_the_buy() {
        // `:2055-2058`: a cursor at its maxStack skips the unit — the
        // purse and the memo stay untouched.
        let mut app = shop_app();
        app.mouse_item = ItemSlot {
            kind: 88,
            name: crate::registry::name(88),
            count: crate::registry::max_stack(88),
            prefix: 0,
        };
        app.mouse_pos = SHOP_SLOT_0;
        app.ui_left_press = true;
        app.handle_inventory_clicks();
        assert_eq!(app.coins, 1_000_000);
        assert_eq!(app.shop_sellback.get_amount(88, 0), 0);
    }

    #[test]
    fn the_alien_cursor_refuses_the_buy() {
        // `:2043`: a non-air cursor that cannot stack with the row refuses
        // (`Item.CanStack`, `Item.cs:49865-49871`).
        let mut app = shop_app();
        app.mouse_item = ItemSlot {
            kind: 9,
            name: crate::registry::name(9),
            count: 3,
            prefix: 0,
        };
        app.mouse_pos = SHOP_SLOT_0;
        app.ui_left_press = true;
        app.handle_inventory_clicks();
        assert_eq!(app.mouse_item.kind, 9, "the cursor keeps its own stack");
        assert_eq!(app.coins, 1_000_000);
    }

    #[test]
    fn the_held_stack_sells_into_an_empty_shop_cell() {
        // `PickItemMovementAction` ctx 15 action 4 (`:1575-1578`) →
        // `LeftClick`'s case 4 (`:1222-1241`): SellItem pays out,
        // AddItemToShop re-stocks, the cursor turns to air, stackSplit
        // locks at 9999 (`:1239`).
        let mut app = shop_app();
        // Cell 39 is past the Merchant's stock — an air cell.
        app.mouse_pos = Vec2::new(460.0, 390.0);
        app.mouse_item = ItemSlot {
            kind: 8, // Torch, value 50 → 10 per unit
            name: crate::registry::name(8),
            count: 5,
            prefix: 0,
        };
        let stock_len = app.shop_stock.len();
        app.ui_left_press = true;
        app.handle_inventory_clicks();
        assert_eq!(app.coins, 1_000_000 + 50);
        assert_eq!(app.mouse_item.count, 0, "the cursor turns to air");
        assert_eq!(app.stack_split, 9999);
        assert_eq!(app.shop_stock.len(), stock_len + 1);
        let e = app.shop_stock[stock_len];
        assert!(e.buy_once);
        assert_eq!((e.item, e.stack), (8, 5));
    }

    #[test]
    fn the_value_zero_item_still_enters_the_shop() {
        // `ItemSlot.cs:1232-1238`: a value-0 item pays nothing but still
        // re-stocks (sound 7). Wood 9 is value 0.
        let mut app = shop_app();
        app.mouse_pos = Vec2::new(460.0, 390.0);
        app.mouse_item = ItemSlot {
            kind: 9,
            name: crate::registry::name(9),
            count: 4,
            prefix: 0,
        };
        let stock_len = app.shop_stock.len();
        app.ui_left_press = true;
        app.handle_inventory_clicks();
        assert_eq!(app.coins, 1_000_000, "no payout on a value-0 item");
        assert_eq!(app.mouse_item.count, 0);
        assert_eq!(app.shop_stock.len(), stock_len + 1);
        assert_eq!(app.shop_stock[stock_len].stack, 4);
    }

    #[test]
    fn the_shift_click_sells_from_the_inventory_grid() {
        // `GetSellOrTrash` (`ItemSlot.cs:198-209`): with a shop open the
        // Shift-hover lights cursor 10; the click runs the cursor-10 arm
        // (`:851-869`) — the slot turns to air, the purse grows, the shop
        // re-stocks the sold units.
        let mut app = shop_app();
        app.inventory[0] = ItemSlot {
            kind: 8,
            name: crate::registry::name(8),
            count: 6,
            prefix: 0,
        };
        app.keys.insert(KeyCode::ShiftLeft);
        // Grid slot 0's centre — (20 + 26·0.85, 20 + 26·0.85) ≈ (42, 42).
        app.mouse_pos = Vec2::new(42.0, 42.0);
        // The hover evaluation runs in the chest handler's head
        // (`handle_chest_clicks` → `evaluate_cursor_override`); with no
        // button held it evaluates and stands down.
        app.handle_chest_clicks();
        assert_eq!(app.cursor_override, CURSOR_SELL);
        app.ui_left_press = true;
        app.handle_inventory_clicks();
        assert_eq!(app.inventory[0].count, 0, "the slot turns to air");
        assert_eq!(app.coins, 1_000_000 + 60);
        let last = app.shop_stock.last().unwrap();
        assert!(last.buy_once);
        assert_eq!((last.item, last.stack), (8, 6));
    }

    #[test]
    fn the_coin_kinds_never_light_the_sell_cursor() {
        // `GetSellOrTrash` (`:204-207`): types 71-74 answer null — the
        // coin stacks cannot be sold.
        let mut app = shop_app();
        app.inventory[0] = ItemSlot {
            kind: 71,
            name: crate::registry::name(71),
            count: 50,
            prefix: 0,
        };
        app.keys.insert(KeyCode::ShiftLeft);
        app.mouse_pos = Vec2::new(42.0, 42.0);
        app.handle_chest_clicks();
        assert_eq!(app.cursor_override, -1, "no sell cursor on coins");
        // Without a shop the same hover lights the quick-trash cursor.
        let mut app = shop_app();
        app.close_shop();
        app.inventory[0] = ItemSlot {
            kind: 8,
            name: crate::registry::name(8),
            count: 6,
            prefix: 0,
        };
        app.keys.insert(KeyCode::ShiftLeft);
        app.mouse_pos = Vec2::new(42.0, 42.0);
        app.handle_chest_clicks();
        assert_eq!(app.cursor_override, CURSOR_TRASH);
    }

    #[test]
    fn the_shift_buy_bulks_a_buy_once_row_ten_at_once() {
        // `HandleShopSlot` (`:2047-2051`): Shift + a buyOnce row buys
        // `GetBulkBuyAmount` = min(10, stack) units in one press.
        let mut app = shop_app();
        // A one-row stock: the buyOnce row sits at cell 0.
        app.shop_stock = vec![crate::shop::ShopEntry {
            item: 8,
            when: crate::shop::StockWhen::Always,
            shop_custom_price: None,
            buy_once: true,
            is_a_shop_item: false,
            prefix: 0,
            stack: 12,
        }];
        app.mouse_pos = SHOP_SLOT_0;
        app.keys.insert(KeyCode::ShiftLeft);
        app.ui_left_press = true;
        app.handle_inventory_clicks();
        assert_eq!(app.mouse_item.count, 10, "ten units in one press");
        assert_eq!(app.shop_stock[0].stack, 2);
        // The buy price of a buyOnce torch row: 50/5 = 10 per unit.
        assert_eq!(app.coins, 1_000_000 - 100);
    }
}
