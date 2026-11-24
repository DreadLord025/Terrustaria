//! W4-g56 — the inventory/chest slot glow (`ItemSlot.cs`'s glow arrays).
//!
//! The C# keeps four static arrays: `inventoryGlowTime`/`inventoryGlowHue`
//! (the player grid, contexts 0/2) and `inventoryGlowTimeChest`/
//! `inventoryGlowHueChest` (the open container, contexts 3/4/32). The pieces
//! ported here:
//! * `ItemSlot.SetGlowForChest` (`ItemSlot.cs:480-492`) — the open-time CLEAR:
//!   `PrepareForChest` resizes the chest arrays to `maxItems`, then
//!   `SetGlow(i, -1f, chest: true)` zeroes every slot's time/hue. The
//!   `CoinSlot.ForceSlotState(j, 3, ...)` half rides `coin_slot.rs` (the
//!   caller threads it).
//! * `ItemSlot.SetGlow` (`:494-514`) — the writer: a negative hue clears the
//!   slot, a non-negative hue arms `time = 300` + the hue. The chest flag
//!   selects the array pair.
//! * `ItemSorting.Sort`'s feedback (`ItemSorting.cs:1584-1587`) — the only
//!   non-clear writer: `SetGlow(num5, num4, Main.player[myPlayer].chest !=
//!   -1)` per written-back slot, `num4` the layer-band hue ramp.
//! * `ItemSlot.UpdateInterface`'s decay (`:522-544`) — per frame `time--`,
//!   the hue zeroes at 0 (the chest pair also on `forceClearGlowsOnChest`,
//!   set by `Player.ChestChangeEvents:28771` when the open chest changes to a
//!   world chest `>= 0`).
//! * `ItemSlot.Draw`'s tint (`:2305-2326`) — a glowing non-air non-favorited
//!   slot swaps to `Inventory_Back13` and lerps the base colour toward
//!   `hslToRgb(hue, 1, 0.5)` by `t²/2` (`t = time/300`).
//!
//! Out (recorded): `ItemSorting.FillAmmoFromInventory`'s ammo-row glow
//! (`:1695`, `SetGlow(i, 0.31f, chest: false)`) — the ammo-sort producer is
//! unported; the grid glow array exists for when it lands. The
//! `forceClearGlowsOnChest` flag is modelled on the chest pair only (the C#
//! reads it solely there, `:538`).

use crate::ui::color;

/// `ItemSlot.inventoryGlowTime`'s arm value (`ItemSlot.cs:505`/`:511`).
const GLOW_TIME: i32 = 300;
/// The player grid's glow-array length (`ItemSlot.cs:360-361`, `new int[58]`).
const GRID_LEN: usize = 58;

/// The four glow arrays (`ItemSlot.cs:302-304` + the static ctor `:360-361`).
/// The chest pair starts at the 58-slot default and grows to the open
/// container's `maxItems` (`PrepareForChest`, `:467-478`); the port keeps a
/// `Vec` so a 40-slot bank/vault and a resized world chest both fit.
#[derive(Clone, Debug)]
pub(crate) struct GlowState {
    /// `inventoryGlowTime[58]` — the player grid (contexts 0/2).
    pub(crate) grid_time: [i32; GRID_LEN],
    /// `inventoryGlowHue[58]`.
    pub(crate) grid_hue: [f32; GRID_LEN],
    /// `inventoryGlowTimeChest` — the open container (contexts 3/4/32).
    pub(crate) chest_time: Vec<i32>,
    /// `inventoryGlowHueChest`.
    pub(crate) chest_hue: Vec<f32>,
    /// `ItemSlot.forceClearGlowsOnChest` (`ItemSlot.cs:310`) — set by
    /// `ChestChangeEvents` (`Player.cs:28771`), consumed by the decay.
    pub(crate) force_clear_chest: bool,
}

impl Default for GlowState {
    fn default() -> Self {
        Self {
            grid_time: [0; GRID_LEN],
            grid_hue: [0.0; GRID_LEN],
            // The static ctor's `new int[58]`/`new float[58]` (`:360-361`).
            chest_time: vec![0; GRID_LEN],
            chest_hue: vec![0.0; GRID_LEN],
            force_clear_chest: false,
        }
    }
}

impl GlowState {
    /// `ItemSlot.PrepareForChest` (`ItemSlot.cs:467-478`): grow the chest
    /// arrays to `maxItems` (never shrink — the C# `Array.Resize` only fires
    /// when `Length < maxItems`).
    pub(crate) fn prepare_for_chest(&mut self, max_items: usize) {
        if self.chest_time.len() < max_items {
            self.chest_time.resize(max_items, 0);
            self.chest_hue.resize(max_items, 0.0);
        }
    }

    /// `ItemSlot.SetGlowForChest` (`ItemSlot.cs:480-492`): the open-time clear
    /// — `PrepareForChest` then `SetGlow(i, -1f, chest: true)` for every slot.
    /// The `CoinSlot.ForceSlotState` half is the caller's (it owns the coin
    /// state). `max_items` is the open container's slot count.
    pub(crate) fn set_glow_for_chest(&mut self, max_items: usize) {
        self.prepare_for_chest(max_items);
        for i in 0..max_items {
            self.set_glow(i, -1.0, true);
        }
    }

    /// `ItemSlot.SetGlow` (`ItemSlot.cs:494-514`): a negative hue clears the
    /// slot (`time = 0`, `hue = 0`), a non-negative hue arms `time = 300` +
    /// the hue. `chest` selects the array pair. Out-of-range indices are
    /// ignored (the C# would throw; the port's callers stay in bounds).
    pub(crate) fn set_glow(&mut self, index: usize, hue: f32, chest: bool) {
        if chest {
            if index >= self.chest_time.len() {
                return;
            }
            if hue < 0.0 {
                self.chest_time[index] = 0;
                self.chest_hue[index] = 0.0;
            } else {
                self.chest_time[index] = GLOW_TIME;
                self.chest_hue[index] = hue;
            }
        } else if index < GRID_LEN {
            // `:509-513` — the grid arm always arms 300 (no negative-hue
            // clear; the C# grid branch has no `hue < 0` test).
            self.grid_time[index] = GLOW_TIME;
            self.grid_hue[index] = hue;
        }
    }

    /// `ItemSlot.UpdateInterface`'s glow decay (`ItemSlot.cs:522-544`): per
    /// frame `time--` on every positive entry; the hue zeroes when the time
    /// hits 0 (the chest pair also on `forceClearGlowsOnChest`, which then
    /// clears). Runs once per game update (`Main.cs:17046`).
    pub(crate) fn update_interface(&mut self) {
        for i in 0..GRID_LEN {
            if self.grid_time[i] > 0 {
                self.grid_time[i] -= 1;
                if self.grid_time[i] == 0 {
                    self.grid_hue[i] = 0.0;
                }
            }
        }
        for j in 0..self.chest_time.len() {
            if self.chest_time[j] > 0 {
                self.chest_time[j] -= 1;
                if self.chest_time[j] == 0 || self.force_clear_chest {
                    self.chest_hue[j] = 0.0;
                }
            }
        }
        self.force_clear_chest = false;
    }

    /// The chest glow time at `index` (0 when out of range / not glowing).
    pub(crate) fn chest_time_at(&self, index: usize) -> i32 {
        self.chest_time.get(index).copied().unwrap_or(0)
    }

    /// The chest glow hue at `index`.
    pub(crate) fn chest_hue_at(&self, index: usize) -> f32 {
        self.chest_hue.get(index).copied().unwrap_or(0.0)
    }
}

/// `ItemSlot.Draw`'s chest-glow tint (`ItemSlot.cs:2314-2326`): a glowing
/// (`time > 0`) non-air non-favorited slot in context 3/4/32 swaps to
/// `Inventory_Back13` and lerps the base colour toward `hslToRgb(hue, 1, 0.5)`
/// by `t²/2` (`t = time/300`). The base is `(130, 62, 102)` for the bank/vault
/// contexts (4/32) and `(104, 52, 52)` for the world chest (3); both scale by
/// `Main.invAlpha/255` (the port keeps `invAlpha = 255`, the open-inventory
/// default). Returns `None` when the slot does not glow (the caller keeps its
/// own background/tint). `time`/`hue` are the slot's glow-array entries.
pub(crate) fn chest_glow_tint(context: i32, time: i32, hue: f32) -> Option<[f32; 4]> {
    if !matches!(context, 3 | 4 | 32) || time <= 0 {
        return None;
    }
    let base: [u8; 4] = if context == 3 {
        [104, 52, 52, 255]
    } else {
        [130, 62, 102, 255]
    };
    let hsl = color::hsl_to_rgb(hue as f64, 1.0, 0.5, 255);
    let t = time as f32 / GLOW_TIME as f32;
    let amount = t * t / 2.0;
    // `Color.Lerp(value4, value5, num9 * num9 / 2f)` — XNA's per-channel byte
    // lerp (`(byte)(a + (b - a) * amount)`, truncation toward zero).
    let lerp = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * amount) as u8;
    Some(color::tint([
        lerp(base[0], hsl[0]),
        lerp(base[1], hsl[1]),
        lerp(base[2], hsl[2]),
        lerp(base[3], hsl[3]),
    ]))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `SetGlowForChest` (`ItemSlot.cs:480-492`) clears every chest slot and
    /// grows the arrays to `maxItems`; a subsequent `SetGlow` arms 300 + hue.
    #[test]
    fn w4_g56_set_glow_for_chest_clears_and_arms() {
        let mut g = GlowState::default();
        // Arm a slot, then the open-time clear zeroes it.
        g.set_glow(2, 0.4, true);
        assert_eq!(g.chest_time_at(2), 300);
        g.set_glow_for_chest(40);
        assert_eq!(g.chest_time_at(2), 0, "the open clears the glow");
        assert_eq!(g.chest_hue_at(2), 0.0);
        assert!(g.chest_time.len() >= 40, "PrepareForChest grows to maxItems");
        // The sort-feedback writer arms a fresh glow.
        g.set_glow(5, 0.25, true);
        assert_eq!(g.chest_time_at(5), 300);
        assert_eq!(g.chest_hue_at(5), 0.25);
    }

    /// `UpdateInterface`'s decay (`ItemSlot.cs:533-544`): time counts down,
    /// the hue zeroes at 0; `forceClearGlowsOnChest` zeroes the hue early.
    #[test]
    fn w4_g56_the_glow_decays_and_force_clears() {
        let mut g = GlowState::default();
        g.set_glow(0, 0.5, true);
        g.update_interface();
        assert_eq!(g.chest_time_at(0), 299, "one tick of decay");
        assert_eq!(g.chest_hue_at(0), 0.5, "the hue survives while glowing");
        // The force-clear zeroes the hue on the next decay (time still > 0).
        g.force_clear_chest = true;
        g.update_interface();
        assert_eq!(g.chest_hue_at(0), 0.0, "forceClearGlowsOnChest zeroes the hue");
        assert!(!g.force_clear_chest, "the flag consumes");
        // Run the time to 0: the hue stays 0.
        for _ in 0..300 {
            g.update_interface();
        }
        assert_eq!(g.chest_time_at(0), 0);
        assert_eq!(g.chest_hue_at(0), 0.0);
    }

    /// `ItemSlot.Draw`'s chest-glow tint (`:2314-2326`): only contexts 3/4/32
    /// with a positive time glow; the base differs (world chest vs bank/vault)
    /// and the lerp amount is `t²/2`.
    #[test]
    fn w4_g56_the_chest_glow_tint_gates_and_lerps() {
        assert!(chest_glow_tint(0, 300, 0.5).is_none(), "the grid context never glows here");
        assert!(chest_glow_tint(4, 0, 0.5).is_none(), "time 0 does not glow");
        // Full time (t=1): amount = 0.5, the midpoint of base and hsl.
        let full = chest_glow_tint(4, 300, 0.0).expect("the vault glows");
        // hsl(0,1,0.5) = pure red (255,0,0); base (130,62,102).
        // lerp at 0.5: (192,31,51) truncated.
        assert_eq!(full, color::tint([192, 31, 51, 255]));
        // The world chest (3) uses the darker base (104,52,52).
        let chest = chest_glow_tint(3, 300, 0.0).expect("the world chest glows");
        assert_eq!(chest, color::tint([179, 26, 26, 255]));
    }
}
