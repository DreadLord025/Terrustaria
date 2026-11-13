//! Port of `Terraria/UI/CoinSlot.cs` — the coin cells' display-state machine
//! (the rank the coin icon/text animations ride). One instance owns the
//! three entry arrays (the vanilla's statics; the port's single-player app is
//! the one owner, S1-H-9).
//!
//! `ForceSlotState` runs after every transfer tail (`ItemSlot.cs:490/:774/
//! :796/:810/:848/:1016/:1119`) so a merged coin slot's rank follows the moved
//! coin; `UpdateDrawState` feeds each cell's draw (`ItemSlot.cs:2349`) and
//! `UpdateSlotAnims` steps the clocks per frame (`Main.cs:44936`).

/// `ItemID.Sets.CommonCoin` (`ItemID.cs:1318`) — the copper..platinum coins.
fn is_common_coin(kind: u16) -> bool {
    (71..=74).contains(&kind)
}

/// `CoinSlot.FadeAnimKeys` (`CoinSlot.cs:286-291`).
const FADE_ANIM_KEYS: [f32; 21] = [
    0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
    1.0, 1.0,
];
/// `CoinSlot.TextAnimKeys` (`CoinSlot.cs:292-296`).
const TEXT_ANIM_KEYS: [f32; 17] = [
    1.0, 1.0107, 1.0391, 1.0791, 1.125, 1.1709, 1.2109, 1.2393, 1.25, 1.2393, 1.2109, 1.1709,
    1.125, 1.0791, 1.0391, 1.0107, 1.0,
];
/// `CoinSlot.JumpAnimKeys` (`CoinSlot.cs:297-301`).
const JUMP_ANIM_KEYS: [f32; 25] = [
    0.0, 0.23748, 0.43408, 0.59366, 0.72007, 0.81717, 0.88881, 0.93885, 0.97115, 0.98955, 0.99793,
    1.0, 1.0, 1.0, 0.99793, 0.98955, 0.97115, 0.93885, 0.88881, 0.81717, 0.72007, 0.59366, 0.43408,
    0.23748, 0.0,
];
const JUMP_APEX: usize = 12;
const JUMP_TRIGGER_TEXT_ANIM_RANGE_START: usize = 9;
const JUMP_TRIGGER_TEXT_ANIM_RANGE_END: usize = 13;
const ITEM_SLOT_COIN_JUMP_SCALE: f32 = 10.0;
const SAVINGS_COIN_JUMP_SCALE: f32 = 10.0;
const JUMP_ANIM_HOLD_TIME: i32 = 12;
const SPIN_ANIM_RANGE_START: usize = 9;
const SPIN_ANIM_RANGE_END: usize = 13;

/// `CoinSlot.CoinDrawState` (`CoinSlot.cs:216-226`): the per-cell draw
/// parameters the vanilla icon/text draws consume.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CoinDrawState {
    pub coin_anim_frame: i32,
    pub coin_y_offset: f32,
    pub stack_text_scale: f32,
    pub stack_text_draw_fade_overload: f32,
    pub fade_item: u16,
    pub fade_scale: f32,
}

/// `CoinSlot.CoinEntry` (`CoinSlot.cs:9-212`): one cell's animation state.
#[derive(Clone, Copy, Debug, Default)]
struct CoinEntry {
    item_type: u16,
    stack: u32,
    text_anim_frame: i32,
    jump_anim_frame: i32,
    spin_anim_frame: i32,
    draw_active: i32,
    jump_anim_hold: i32,
    fade_item_type: u16,
}

impl CoinEntry {
    /// `CoinEntry.ForceState` (`CoinSlot.cs:29-38`).
    fn force_state(&mut self, item_type: u16, item_stack: u32) {
        self.item_type = item_type;
        self.stack = item_stack;
        self.text_anim_frame = 0;
        self.jump_anim_frame = 0;
        self.jump_anim_hold = 0;
        self.spin_anim_frame = 0;
        self.fade_item_type = 0;
    }

    /// `CoinEntry.UpdateState` (`CoinSlot.cs:40-176`).
    fn update_state(&mut self, item_type: u16, item_stack: u32, jump_scale: f32) -> CoinDrawState {
        let mut draw_state = CoinDrawState {
            coin_anim_frame: 0,
            coin_y_offset: 0.0,
            stack_text_scale: 1.0,
            stack_text_draw_fade_overload: 0.0,
            fade_item: 0,
            fade_scale: 1.0,
        };
        if self.item_type != item_type || self.draw_active == 0 {
            let mut flag = true;
            if item_type != 0 && self.fade_item_type == item_type && self.draw_active != 0 {
                flag = false;
            }
            if item_type == 0 && self.draw_active != 0 && is_common_coin(self.item_type) {
                self.fade_item_type = self.item_type;
            } else {
                self.fade_item_type = 0;
            }
            if self.fade_item_type != 0 {
                flag = false;
            }
            self.item_type = item_type;
            if self.draw_active == 0 {
                self.stack = item_stack;
            }
            if flag {
                self.text_anim_frame = 0;
                self.jump_anim_frame = 0;
                self.jump_anim_hold = 0;
                self.spin_anim_frame = 0;
            }
        }
        self.draw_active = 2;
        if is_common_coin(self.item_type) || self.item_type == 3817 || self.fade_item_type != 0 {
            if self.stack != item_stack {
                self.stack = item_stack;
                if self.text_anim_frame == 0 {
                    self.text_anim_frame = TEXT_ANIM_KEYS.len() as i32 - 1;
                }
            }
            if self.text_anim_frame >= JUMP_TRIGGER_TEXT_ANIM_RANGE_START as i32
                && self.text_anim_frame <= JUMP_TRIGGER_TEXT_ANIM_RANGE_END as i32
            {
                self.jump_anim_hold = JUMP_ANIM_HOLD_TIME;
                if self.jump_anim_frame == 0 {
                    self.jump_anim_frame = JUMP_ANIM_KEYS.len() as i32 - 1;
                }
            }
        }
        draw_state.stack_text_scale = TEXT_ANIM_KEYS[self.text_anim_frame.clamp(0, 16) as usize];
        draw_state.coin_y_offset =
            JUMP_ANIM_KEYS[self.jump_anim_frame.clamp(0, 24) as usize] * jump_scale;
        draw_state.coin_anim_frame = self.spin_anim_frame / 2;
        draw_state.fade_item = self.fade_item_type;
        draw_state.fade_scale = 1.0;
        if self.fade_item_type != 0 {
            if self.text_anim_frame > 0
                || self.jump_anim_frame >= JUMP_APEX as i32
                || self.jump_anim_frame >= FADE_ANIM_KEYS.len() as i32
            {
                draw_state.stack_text_draw_fade_overload = 1.0;
            } else {
                draw_state.stack_text_draw_fade_overload =
                    FADE_ANIM_KEYS[self.jump_anim_frame.max(0) as usize];
            }
            draw_state.fade_scale = draw_state.stack_text_draw_fade_overload;
        } else if self.stack == 1 && (self.text_anim_frame > 0 || self.jump_anim_frame != 0) {
            if self.text_anim_frame > 0
                || self.jump_anim_frame >= JUMP_APEX as i32
                || self.jump_anim_frame >= FADE_ANIM_KEYS.len() as i32
            {
                draw_state.stack_text_draw_fade_overload = 1.0;
            } else {
                draw_state.stack_text_draw_fade_overload =
                    FADE_ANIM_KEYS[self.jump_anim_frame.max(0) as usize];
            }
        } else {
            draw_state.stack_text_draw_fade_overload = -1.0;
        }
        draw_state
    }

    /// `CoinEntry.UpdateAnim` (`CoinSlot.cs:178-210`).
    fn update_anim(&mut self) {
        if self.draw_active > 0 {
            self.draw_active -= 1;
        }
        if self.fade_item_type > 0 && self.jump_anim_frame == 0 && self.text_anim_frame == 0 {
            self.fade_item_type = 0;
        }
        if self.text_anim_frame > 0 {
            self.text_anim_frame -= 1;
        }
        if self.jump_anim_hold > 0 {
            self.jump_anim_hold -= 1;
        }
        if self.jump_anim_frame > 0 {
            if self.jump_anim_hold > 0 {
                if self.jump_anim_frame != JUMP_APEX as i32 {
                    if self.jump_anim_frame < JUMP_APEX as i32 {
                        self.jump_anim_frame =
                            JUMP_APEX as i32 + JUMP_APEX as i32 - self.jump_anim_frame;
                    }
                    self.jump_anim_frame -= 1;
                }
            } else {
                self.jump_anim_frame -= 1;
            }
        }
        if self.jump_anim_frame >= SPIN_ANIM_RANGE_START as i32
            && self.jump_anim_frame <= SPIN_ANIM_RANGE_END as i32
        {
            self.spin_anim_frame = (self.spin_anim_frame + 1) % 14;
        } else if self.spin_anim_frame != 0 {
            self.spin_anim_frame = (self.spin_anim_frame + 1) % 14;
        }
    }
}

/// The machine's state — the vanilla's static arrays (`CoinSlot.cs:340-344`)
/// owned by the App. `UpdateSavings`/`UpdateCustom` have no port consumers
/// (the piggy-bank HUD and the custom-currency displays are unported), so the
/// `Savings`/`Custom` arrays stay out.
#[derive(Clone, Debug, Default)]
pub struct CoinSlotState {
    chest_entries: Vec<CoinEntry>,
    inventory_entries: Vec<CoinEntry>,
}

impl CoinSlotState {
    /// The static constructor's array sizes (`CoinSlot.cs:342-343`).
    pub fn new() -> Self {
        Self {
            chest_entries: vec![CoinEntry::default(); 200],
            inventory_entries: vec![CoinEntry::default(); 59],
        }
    }

    /// `CoinSlot.UpdateSlotAnims` (`CoinSlot.cs:358-372`) — the vanilla runs
    /// it per frame after the interface layers draw (`Main.cs:44936`).
    pub fn update_slot_anims(&mut self) {
        for e in &mut self.chest_entries {
            e.update_anim();
        }
        for e in &mut self.inventory_entries {
            e.update_anim();
        }
    }

    /// `CoinSlot.ForceSlotState` (`CoinSlot.cs:374-390`): the rank reset the
    /// transfer tails call (`ItemSlot.cs:490/:774/:796/:810/:848/:1016/:1119`).
    /// Contexts 0/1/2 ride the inventory array (the grid's slot indices and
    /// the coin row's vanilla numbering share it), 3/4 the chest array.
    pub fn force_slot_state(&mut self, slot: i32, context: i32, kind: u16, count: u32) {
        match context {
            0 | 1 | 2 => {
                if let Some(e) = self.inventory_entries.get_mut(slot as usize) {
                    e.force_state(kind, count);
                }
            }
            3 | 4 => {
                if let Some(e) = self.chest_entries.get_mut(slot as usize) {
                    e.force_state(kind, count);
                }
            }
            _ => {}
        }
    }

    /// `CoinSlot.UpdateDrawState` (`CoinSlot.cs:392-422`): the cell's draw
    /// parameters; unmapped contexts get the neutral state (`:405-419`).
    pub fn update_draw_state(
        &mut self,
        slot: i32,
        context: i32,
        kind: u16,
        count: u32,
    ) -> CoinDrawState {
        match context {
            0 | 1 | 2 => {
                if let Some(e) = self.inventory_entries.get_mut(slot as usize) {
                    return e.update_state(kind, count, ITEM_SLOT_COIN_JUMP_SCALE);
                }
                neutral()
            }
            3 | 4 => {
                if let Some(e) = self.chest_entries.get_mut(slot as usize) {
                    return e.update_state(kind, count, ITEM_SLOT_COIN_JUMP_SCALE);
                }
                neutral()
            }
            _ => neutral(),
        }
    }
}

impl CoinSlotState {
    /// The test seam: an entry's stored rank (`item_type`, `stack`).
    #[cfg(test)]
    pub(crate) fn entry_rank(&self, slot: i32, context: i32) -> (u16, u32) {
        match context {
            0 | 1 | 2 => self
                .inventory_entries
                .get(slot as usize)
                .map(|e| (e.item_type, e.stack))
                .unwrap_or((0, 0)),
            _ => self
                .chest_entries
                .get(slot as usize)
                .map(|e| (e.item_type, e.stack))
                .unwrap_or((0, 0)),
        }
    }
}

/// The `UpdateDrawState` tail's neutral state (`CoinSlot.cs:405-419`).
fn neutral() -> CoinDrawState {
    CoinDrawState {
        fade_item: 0,
        fade_scale: 1.0,
        coin_anim_frame: 0,
        coin_y_offset: 0.0,
        stack_text_scale: 1.0,
        stack_text_draw_fade_overload: -1.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn force_state_resets_the_rank_to_the_moved_coin() {
        // `CoinEntry.ForceState` (`CoinSlot.cs:29-38`): the transfer tails'
        // reset — the entry's rank (Type/Stack) follows the moved coin and
        // every animation clock zeroes.
        let mut st = CoinSlotState::new();
        // A copper (71) sits in the coin row's first cell; a gold (73) moves
        // in — the entry re-derives to the gold (`ItemSlot.cs:1016`'s tail).
        st.force_slot_state(crate::inv::coin_slot_index(0) as i32, 1, 71, 99);
        st.force_slot_state(crate::inv::coin_slot_index(0) as i32, 1, 73, 1);
        let d = st.update_draw_state(crate::inv::coin_slot_index(0) as i32, 1, 73, 1);
        assert_eq!(d.fade_item, 0, "the force clears the fade state");
        // The vanilla index mapping: the coin row lives at inventory
        // `Player.inventory[50..53]` (`coin_slot_index`), the chest array is
        // its own (`CoinSlot.cs:342-343`).
        assert_eq!(crate::inv::coin_slot_index(0), crate::item::INV);
    }

    #[test]
    fn update_state_fades_a_common_coin_out_to_air() {
        // `UpdateState`'s fade arm (`CoinSlot.cs:44-56`): a CommonCoin
        // (`ItemID.cs:1318` — 71..74) vacating a DRAWN cell (`DrawActive !=
        // 0`) keeps the old type as `FadeItemType` so the icon fades out; a
        // non-coin does not. A `ForceState`-fresh entry has `DrawActive == 0`
        // — the fade arms on the second vacate.
        let mut e = CoinEntry::default();
        e.force_state(71, 5);
        let _ = e.update_state(71, 5, ITEM_SLOT_COIN_JUMP_SCALE); // drawn: active 2
        let _ = e.update_state(0, 0, ITEM_SLOT_COIN_JUMP_SCALE);
        assert_eq!(e.fade_item_type, 71, "the copper fades out");
        let mut e = CoinEntry::default();
        e.force_state(3601, 5); // the Celestial Sigil — not a CommonCoin
        let _ = e.update_state(3601, 5, ITEM_SLOT_COIN_JUMP_SCALE);
        let _ = e.update_state(0, 0, ITEM_SLOT_COIN_JUMP_SCALE);
        assert_eq!(e.fade_item_type, 0, "a non-coin does not fade");
    }

    #[test]
    fn update_anim_steps_the_clocks_and_bounces_the_stack_change() {
        // `UpdateState`'s stack-change arm (`CoinSlot.cs:60-77`): a stack
        // change while drawn restarts the text pulse at the key table's tail
        // (`TextAnimKeys[16]` = 1.0 — the pulse reads the reversed keys and
        // grows as `UpdateAnim` drains the frame); the jump only fires once
        // the text frame walks the 9..13 trigger window.
        let mut e = CoinEntry::default();
        e.force_state(71, 5);
        let _ = e.update_state(71, 5, ITEM_SLOT_COIN_JUMP_SCALE); // the draw warm-up — a fresh cell absorbs its stack (`DrawActive == 0` arm)
        let d = e.update_state(71, 6, ITEM_SLOT_COIN_JUMP_SCALE);
        assert!(
            (d.stack_text_scale - 1.0).abs() < f32::EPSILON,
            "the pulse starts at the tail key"
        );
        assert!(
            (d.coin_y_offset - 0.0).abs() < f32::EPSILON,
            "no jump before the trigger window"
        );
        e.update_anim();
        let d = e.update_state(71, 6, ITEM_SLOT_COIN_JUMP_SCALE);
        assert!(
            d.stack_text_scale > 1.0,
            "the pulse grows: {}",
            d.stack_text_scale
        );
        // `UpdateAnim` drained the draw-active latch (2 → 1) before the
        // second draw re-armed it.
        assert_eq!(e.draw_active, 2, "the draw re-arms per frame");
    }

    #[test]
    fn the_chest_array_is_its_own_rank_space() {
        // `CoinSlot.cs:342-343`: `ChestEntries[200]` vs `InventoryEntries[59]`
        // — the same slot number in the chest row does not touch the
        // inventory's entry.
        let mut st = CoinSlotState::new();
        st.force_slot_state(0, 3, 72, 33);
        let d = st.update_draw_state(0, 3, 72, 33);
        assert_eq!(d.coin_anim_frame, 0);
        let inv = st.update_draw_state(0, 0, 0, 0);
        assert_eq!(
            inv.stack_text_draw_fade_overload, -1.0,
            "the grid entry stays neutral"
        );
    }
}
