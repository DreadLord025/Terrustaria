//! `Terraria.WorldSections`' map-draw queue — the spiral section walk that
//! paints the map texture one 200x150 section at a time
//! (`WorldSections.cs:218-358` `GetNextMapDraw`, `:149-157` `ClearMapDraw`).
//!
//! The C# class also tracks the net section load/frame bits
//! (`BitIndex_SectionLoaded/Framed`, `:33-39`); the port is single-player
//! with the whole world resident, so `SectionLoaded` is always true (the
//! same ruling `map::update_type` records for the `TileLoaded` gate,
//! `WorldMap.cs:81`) and only the map-draw bit (bit 2) is modelled here.
//!
//! The driver is `Main.cs:61868-61884`: a 5 ms-budgeted
//! `while (GetNextMapDraw(player.position, out x, out y))` loop that calls
//! `MapRenderer.DrawToMap_Section(x, y)` (and `WorldMap.UnlockMapSection`
//! while `DebugOptions.unlockMap != 0`). `Main.refreshMap` re-arms the walk
//! through `ClearMapDraw` (`Main.cs:61888-61892`).

/// `WorldSections.IterationState` (`WorldSections.cs:8-31`): the spiral's
/// resume point. `Reset` pins the centre to `(-3200, -2400)` — far outside
/// any world, so the first call always takes the section-change arm
/// (`:235`).
#[derive(Clone, Copy, Debug)]
struct IterationState {
    center_pos: [f32; 2],
    x: i32,
    y: i32,
    leg: i32,
    x_dir: i32,
    y_dir: i32,
}

impl IterationState {
    /// `IterationState.Reset` (`WorldSections.cs:22-30`).
    fn reset() -> Self {
        Self {
            center_pos: [-3200.0, -2400.0],
            x: 0,
            y: 0,
            leg: 0,
            x_dir: 0,
            y_dir: 0,
        }
    }
}

/// The map-draw half of `Terraria.WorldSections` (`WorldSections.cs:6-359`).
/// Constructed with `Main.maxTilesX / 200` x `Main.maxTilesY / 150`
/// sections (`WorldGen.cs:7150` — INTEGER division, not ceil; every vanilla
/// world size is an exact multiple, so the grid covers the world).
pub struct MapDrawQueue {
    width: i32,
    height: i32,
    /// Bit 2 (`BitIndex_SectionMapDrawn`, `:37`) per section, row-major
    /// `y * width + x` (`:283`).
    drawn: Vec<bool>,
    /// `mapSectionsLeft` (`:47`) — the ctor arms it to the section count
    /// (`:66`); each drawn section decrements (`:350`).
    sections_left: i32,
    /// `prevMap` (`:55`) — the spiral resume state.
    prev_map: IterationState,
}

impl MapDrawQueue {
    /// `WorldSections(int, int)` (`WorldSections.cs:61-69`): every section
    /// starts undrawn, `mapSectionsLeft = width * height`, both iteration
    /// states reset.
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            width,
            height,
            drawn: vec![false; (width.max(0) * height.max(0)) as usize],
            sections_left: width * height,
            prev_map: IterationState::reset(),
        }
    }

    /// `WorldSections.ClearMapDraw` (`WorldSections.cs:149-157`): every
    /// map-drawn bit clears, `prevMap` resets and `mapSectionsLeft` re-arms
    /// to the full count — the `Main.refreshMap` re-walk
    /// (`Main.cs:61891`).
    pub fn clear_map_draw(&mut self) {
        for d in self.drawn.iter_mut() {
            *d = false;
        }
        self.prev_map = IterationState::reset();
        self.sections_left = self.drawn.len() as i32;
    }

    /// `WorldSections.MapSectionDrawn` (`WorldSections.cs:136-147`): the
    /// out-of-bounds read is `false`.
    pub fn map_section_drawn(&self, x: i32, y: i32) -> bool {
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            return false;
        }
        self.drawn[(y * self.width + x) as usize]
    }

    /// `mapSectionsLeft` (`WorldSections.cs:47`) — the remaining count.
    pub fn sections_left(&self) -> i32 {
        self.sections_left
    }

    /// `WorldSections.GetNextMapDraw` (`WorldSections.cs:218-358`),
    /// verbatim: the outward spiral from the player's section. `player_pos`
    /// is `Main.player[myPlayer].position` in world PIXELS (`Main.cs:61871`);
    /// the C# scales it `*= 0.0625f` to tiles (`:229`) and sections ride
    /// `Netplay.GetSectionX/Y`'s `/ 200` / `/ 150` (`Netplay.cs:770-778`).
    /// Returns `None` when every section is drawn (`:220-225`); otherwise
    /// marks the next undrawn section and resumes the spiral from it.
    pub fn get_next_map_draw(&mut self, player_pos: [f32; 2]) -> Option<(i32, i32)> {
        // `:220-225`.
        if self.sections_left <= 0 {
            return None;
        }
        // `:226-233`: num/num2 are the resumed directions, vector the
        // previous centre. `(int)` casts truncate toward zero (C# and Rust
        // `as i32` agree); the section divides are C# integer division.
        let mut num = 0i32;
        let mut num2 = 0i32;
        let mut vector = self.prev_map.center_pos;
        let px = player_pos[0] * 0.0625;
        let py = player_pos[1] * 0.0625;
        let section_x = (px as i32) / 200;
        let section_y = (py as i32) / 150;
        let mut num3 = (vector[0] as i32) / 200;
        let mut num4 = (vector[1] as i32) / 150;
        let mut num5;
        let mut x;
        let mut y;
        // `:235-251`: a section change restarts the spiral at the player's
        // section with leg 4; otherwise resume the previous walk.
        if num3 != section_x || num4 != section_y {
            vector = [px, py];
            num3 = section_x;
            num4 = section_y;
            num5 = 4;
            x = section_x;
            y = section_y;
        } else {
            num5 = self.prev_map.leg;
            x = self.prev_map.x;
            y = self.prev_map.y;
            num = self.prev_map.x_dir;
            num2 = self.prev_map.y_dir;
        }
        // `:252-258`: the offset from the section centre picks the initial
        // spiral direction when none was resumed.
        let mut num6 = (px - (num3 as f32 + 0.5) * 200.0) as i32;
        let mut num7 = (py - (num4 as f32 + 0.5) * 150.0) as i32;
        if num == 0 {
            num = if num6 <= 0 { 1 } else { -1 };
            num2 = if num7 <= 0 { 1 } else { -1 };
        }
        let mut num8 = 0i32;
        // `:260-261`: both flags start false — `flag` stays false while the
        // walk is out of bounds (the C# initialises it before the loop).
        let mut flag = false;
        let mut flag2 = false;
        // `:262-348`: the spiral walk. The C# `while (true)` breaks on the
        // first in-bounds undrawn section (`:280-287`).
        loop {
            // `:264-279`: four axis steps without a turn restart from the
            // centre; a second restart with no progress is the C#'s
            // "Infinite loop" throw (`:268`) — unreachable on a valid grid
            // (every restart re-scans the axes), kept as the same panic.
            if num8 == 4 {
                if flag2 {
                    panic!("Infinite loop in WorldSections.GetNextMapDraw");
                }
                flag2 = true;
                x = num3;
                y = num4;
                num6 = (vector[0] - (num3 as f32 + 0.5) * 200.0) as i32;
                num7 = (vector[1] - (num4 as f32 + 0.5) * 150.0) as i32;
                num = if num6 <= 0 { 1 } else { -1 };
                num2 = if num7 <= 0 { 1 } else { -1 };
                num5 = 4;
                num8 = 0;
            }
            // `:280-287`.
            if y >= 0 && y < self.height && x >= 0 && x < self.width {
                flag = false;
                if !self.drawn[(y * self.width + x) as usize] {
                    break;
                }
            }
            let num9 = x - num3;
            let num10 = y - num4;
            // `:290-342`: on an axis, turn (leg 4 → the corner kick) or
            // step along the spiral; off-axis, keep walking the direction.
            if num9 == 0 || num10 == 0 {
                if num5 == 4 {
                    // `:294-315`: the first step away from the centre —
                    // kick along the SHORTER offset axis, then step out.
                    if num9 == 0 && num10 == 0 {
                        if num6.abs() > num7.abs() {
                            y -= num2;
                        } else {
                            x -= num;
                        }
                    } else {
                        if num9 != 0 {
                            x += num9 / num9.abs();
                        }
                        if num10 != 0 {
                            y += num10 / num10.abs();
                        }
                    }
                    num5 = 0;
                    num8 = -2;
                    flag = true;
                } else {
                    // `:322-332`: flip the direction of the axis we are
                    // on, step, advance the leg.
                    if num9 != 0 {
                        num = if num9 <= 0 { 1 } else { -1 };
                    } else {
                        num2 = if num10 <= 0 { 1 } else { -1 };
                    }
                    x += num;
                    y += num2;
                    num5 += 1;
                }
                // `:334-341`.
                if flag {
                    num8 += 1;
                } else {
                    flag = true;
                }
            } else {
                // `:345-346`.
                x += num;
                y += num2;
            }
        }
        // `:349-356`: mark drawn, decrement, persist the resume state.
        self.drawn[(y * self.width + x) as usize] = true;
        self.sections_left -= 1;
        self.prev_map.center_pos = [px, py];
        self.prev_map.x = x;
        self.prev_map.y = y;
        self.prev_map.leg = num5;
        self.prev_map.x_dir = num;
        self.prev_map.y_dir = num2;
        Some((x, y))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// W4-f46 witness (written, not run — code-only): the first call takes
    /// the section-change arm (`WorldSections.cs:235`, the reset centre
    /// (-3200, -2400) is section (-16, -16)) and yields the PLAYER's
    /// section; the walk then covers every section exactly once and
    /// `mapSectionsLeft` drains to zero (`:349-350`, `:220-225`).
    #[test]
    fn w4_f46_spiral_starts_at_the_player_section_and_covers_all() {
        let mut q = MapDrawQueue::new(4, 3);
        assert_eq!(q.sections_left(), 12);
        // Player pixel (500*16, 200*16) → tile (500, 200) → section (2, 1).
        let first = q.get_next_map_draw([500.0 * 16.0, 200.0 * 16.0]);
        assert_eq!(first, Some((2, 1)), "the player's section draws first");
        assert!(q.map_section_drawn(2, 1));
        let mut seen = vec![(2, 1)];
        while let Some(s) = q.get_next_map_draw([500.0 * 16.0, 200.0 * 16.0]) {
            assert!(!seen.contains(&s), "no section draws twice");
            seen.push(s);
        }
        assert_eq!(seen.len(), 12, "every section exactly once");
        assert_eq!(q.sections_left(), 0);
        for y in 0..3 {
            for x in 0..4 {
                assert!(q.map_section_drawn(x, y));
            }
        }
        assert_eq!(q.get_next_map_draw([0.0, 0.0]), None, "exhausted");
    }

    /// W4-f46 witness (written, not run — code-only): moving the player to
    /// a new section restarts the spiral there (`WorldSections.cs:235-243`)
    /// while the drawn bits persist — the already-drawn sections are
    /// skipped, not redrawn.
    #[test]
    fn w4_f46_section_change_restarts_the_spiral() {
        let mut q = MapDrawQueue::new(4, 3);
        // Drain three sections around (0, 0).
        let a = q.get_next_map_draw([100.0 * 16.0, 75.0 * 16.0]);
        assert_eq!(a, Some((0, 0)));
        let _ = q.get_next_map_draw([100.0 * 16.0, 75.0 * 16.0]);
        let _ = q.get_next_map_draw([100.0 * 16.0, 75.0 * 16.0]);
        assert_eq!(q.sections_left(), 9);
        // Jump to section (3, 2): the next draw is the new centre itself.
        let b = q.get_next_map_draw([700.0 * 16.0, 375.0 * 16.0]);
        assert_eq!(b, Some((3, 2)), "the restart yields the new section");
        // The remaining walk never revisits the three drawn sections.
        let drawn_before: Vec<(i32, i32)> = (0..4)
            .flat_map(|x| (0..3).map(move |y| (x, y)))
            .filter(|&(x, y)| q.map_section_drawn(x, y))
            .collect();
        while let Some(s) = q.get_next_map_draw([700.0 * 16.0, 375.0 * 16.0]) {
            assert!(!drawn_before.contains(&s));
        }
        assert_eq!(q.sections_left(), 0);
    }

    /// W4-f46 witness (written, not run — code-only): `ClearMapDraw`
    /// (`WorldSections.cs:149-157`) re-arms the full walk — every bit
    /// clears, the count restores and the reset centre forces the
    /// section-change arm again (`Main.refreshMap`'s re-walk,
    /// `Main.cs:61891`).
    #[test]
    fn w4_f46_clear_map_draw_rearms_the_walk() {
        let mut q = MapDrawQueue::new(2, 2);
        while q.get_next_map_draw([100.0 * 16.0, 75.0 * 16.0]).is_some() {}
        assert_eq!(q.sections_left(), 0);
        q.clear_map_draw();
        assert_eq!(q.sections_left(), 4);
        for y in 0..2 {
            for x in 0..2 {
                assert!(!q.map_section_drawn(x, y));
            }
        }
        // The reset centre (-3200, -2400) differs from the last player
        // section, so the first call yields the player's section again.
        assert_eq!(
            q.get_next_map_draw([300.0 * 16.0, 225.0 * 16.0]),
            Some((1, 1))
        );
    }

    /// W4-f46 witness (written, not run — code-only): the grid is
    /// `maxTilesX / 200` x `maxTilesY / 150` INTEGER division
    /// (`WorldGen.cs:7150`) — a world smaller than one section has an empty
    /// queue (`mapSectionsLeft == 0` → `GetNextMapDraw` returns false at
    /// `:220`), and `MapSectionDrawn`'s out-of-bounds read is false
    /// (`WorldSections.cs:136-145`).
    #[test]
    fn w4_f46_floor_grid_and_bounds() {
        let mut q = MapDrawQueue::new(200 / 200, 128 / 150);
        assert_eq!(q.sections_left(), 0, "128 / 150 == 0 rows");
        assert_eq!(q.get_next_map_draw([0.0, 0.0]), None);
        assert!(!q.map_section_drawn(-1, 0));
        assert!(!q.map_section_drawn(0, 5));
        let mut q = MapDrawQueue::new(1, 1);
        assert_eq!(q.get_next_map_draw([0.0, 0.0]), Some((0, 0)));
        assert_eq!(q.get_next_map_draw([0.0, 0.0]), None);
    }
}
