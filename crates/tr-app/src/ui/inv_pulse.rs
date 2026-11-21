//! `Main.invAlpha` / `Main.invDir` — the Interface-Logic-2 inventory pulse
//! (`Main.DrawInterface_24_InterfaceLogic2`, `Main.cs:45695-45711`), W4-f23
//! deviation fix: the crafting grid's selection overlay used the constant
//! midpoint 210 as a stand-in; this ports the exact curve.
//!
//! The C# statics start at `invAlpha = 1f`, `invDir = 1f`
//! (`Main.cs:66846-66847`); every draw-interface pass adds `invDir * 0.2f`
//! and clamps into `[180, 240]`, flipping the direction at each rail
//! (`:45697-45707`). `Main.inventoryBack` is then the byte-truncated
//! `(invAlpha, invAlpha, invAlpha, invAlpha)` (`:45708`) — the tint
//! `ItemSlot.Draw`'s context-41 `Inventory_Back24` selection overlay reads
//! (`ItemSlot.cs:2371-2374`). The first tick clamps 1.2 up to 180, so the
//! pulse enters the band immediately and rises.
//!
//! The pulse also tints the inventory slot glow arms (`ItemSlot.cs:2307`,
//! `:2316`); those stay unported (the glow timers are not modelled), so the
//! only consumer today is the crafting overlay — the App ticks the pair in
//! the same per-frame block as `mouse_text_color` and hands the byte colour
//! to `draw_hud`.

/// The `(Main.invAlpha, Main.invDir)` pair (`Main.cs:1287-1289`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InvPulse {
    /// `Main.invAlpha` — starts at 1 (`Main.cs:66846`), lives in
    /// `[180, 240]` after the first tick.
    pub alpha: f32,
    /// `Main.invDir` — starts at 1 (`Main.cs:66847`), flips at the rails.
    pub dir: f32,
}

impl Default for InvPulse {
    /// `Main.Initialize`'s `Main.invAlpha = 1f; Main.invDir = 1f`
    /// (`Main.cs:66846-66847`).
    fn default() -> Self {
        Self { alpha: 1.0, dir: 1.0 }
    }
}

impl InvPulse {
    /// One `DrawInterface_24_InterfaceLogic2` tick (`Main.cs:45697-45707`):
    /// `invAlpha += invDir * 0.2f`, then the two independent clamps — `> 240`
    /// pins 240 and flips down, `< 180` pins 180 and flips up. The order
    /// matters only in that both run every tick (a rail hit never skips the
    /// other check).
    pub fn tick(&mut self) {
        self.alpha += self.dir * 0.2;
        if self.alpha > 240.0 {
            self.alpha = 240.0;
            self.dir = -1.0;
        }
        if self.alpha < 180.0 {
            self.alpha = 180.0;
            self.dir = 1.0;
        }
    }

    /// `Main.inventoryBack = new Color((byte)invAlpha, ...)` (`Main.cs:45708`)
    /// as the renderer tint: the float truncated to a byte, all four
    /// channels, over 255. `(byte)` wraps mod 256 in C#, but the clamped
    /// band `[180, 240]` never leaves byte range.
    pub fn inventory_back(&self) -> [f32; 4] {
        let b = self.alpha as i64 as u8;
        super::color::tint([b, b, b, b])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The initial state is the C# statics' init (`Main.cs:66846-66847`),
    /// and the FIRST tick clamps 1.2 up to the 180 rail (`:45703-45706`) —
    /// the pulse enters the band immediately, not after 900 ticks.
    #[test]
    fn the_first_tick_clamps_into_the_band() {
        let mut p = InvPulse::default();
        assert_eq!((p.alpha, p.dir), (1.0, 1.0));
        p.tick();
        assert_eq!((p.alpha, p.dir), (180.0, 1.0));
        // The byte colour is (180,180,180,180)/255 (`:45708`).
        assert_eq!(p.inventory_back(), [180.0 / 255.0; 4]);
    }

    /// The exact curve (`Main.cs:45697-45707`): +0.2 per tick up to the 240
    /// rail (~300 ticks from 180), the rail tick pins 240 EXACTLY and flips,
    /// then -0.2 per tick down to 180 (~300 ticks), pin + flip, repeat — a
    /// ~600-tick triangle wave, NOT the midpoint-210 constant. The f32
    /// accumulation of 0.2 drifts by ULPs (the C# accumulates the same way),
    /// so the rise/fall compare with a tolerance; the rail pins are exact
    /// literals in both.
    #[test]
    fn the_pulse_is_a_600_tick_triangle_between_180_and_240() {
        let mut p = InvPulse::default();
        p.tick(); // enter the band at 180, dir +1
        // Rise: tick k lands ~180 + 0.2k until the rail clamp flips.
        let mut rise_ticks = 0;
        while p.dir > 0.0 {
            p.tick();
            rise_ticks += 1;
            assert!(p.alpha >= 180.0 && p.alpha <= 240.0, "in the band");
            assert!(rise_ticks <= 302, "the rise cannot outrun 300 + drift");
        }
        assert!(rise_ticks >= 300, "0.2 * 300 = the 60 px span");
        assert_eq!(p.alpha, 240.0, "the rail clamp pins the literal 240");
        assert!((180.0 + 0.2 * (rise_ticks - 1) as f32 - 240.0).abs() < 0.05);
        // Fall: 240 - 0.2k down to the lower rail.
        let mut fall_ticks = 0;
        while p.dir < 0.0 {
            let before = p.alpha;
            p.tick();
            fall_ticks += 1;
            if p.dir < 0.0 {
                assert!(p.alpha < before, "monotone fall");
                assert!((before - p.alpha - 0.2).abs() < 0.001, "the 0.2 step");
            }
            assert!(fall_ticks <= 302);
        }
        assert!(fall_ticks >= 300);
        assert_eq!(p.alpha, 180.0, "the lower clamp pins the literal 180");
        assert_eq!(p.dir, 1.0, "and flips back up");
        // The byte truncation: 180.6 -> 180, 239.8 -> 239 (`(byte)` cast).
        p.alpha = 180.6;
        assert_eq!(p.inventory_back(), [180.0 / 255.0; 4]);
        p.alpha = 239.8;
        assert_eq!(p.inventory_back(), [239.0 / 255.0; 4]);
    }
}
