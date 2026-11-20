//! `Terraria.WorldSections` (`WorldSections.cs:6-358`) — the section-grid
//! state behind `Main.sectionManager` (`Main.cs:399`), the map half the
//! reveal queue walks (W4-e18, the first half of the `Main.cs:61863-61899`
//! map-draw machinery; the second half — the `ExportToMiniMap` reveal
//! window — landed in `map.rs::WorldMap::update`).
//!
//! The grid is `Main.maxTilesX / 200` x `Main.maxTilesY / 150`
//! (`WorldGen.cs:7150`, integer division — NOT rounded up; the port's
//! `net::sections_x/y` round up for the wire's coverage and are a different
//! quantity). The four `BitsByte` flags (`WorldSections.cs:33-39`) ride one
//! byte per section. `prevMap` is the spiral walk's resume state
//! (`IterationState`, `:8-31`); its `Reset` centre `(-3200, -2400)` (`:24`)
//! is deliberately off-grid so the first `GetNextMapDraw` starts at the
//! player's own section.
//!
//! The loaded/framed halves (`TileLoaded`, `SetSectionFramed`, the
//! `frameSectionsLeft`/`_sectionsNeedingRefresh` counters) are ported
//! alongside because `SetAllFramedSectionsAsNeedingRefresh` and
//! `UpdateType`'s `TileLoaded` gate (`WorldMap.cs:81`) read them; the port
//! keeps the whole world resident, so every section is armed loaded +
//! framed at construction (`WorldGen.cs:6744`/`:6821`'s
//! `SetAllSectionsLoaded` + `WorldGen.cs:80363`'s per-section
//! `SetSectionFramed`, collapsed into the constructor — documented in
//! `docs/ANALYSIS.md`, "The map section grid").

use crate::net::{SECTION_H, SECTION_W};

/// `Netplay.GetSectionX` (`Netplay.cs:770-777`): `x / 200` — C# integer
/// division TRUNCATES toward zero. `crate::net::section_x` uses
/// `div_euclid` for the wire's non-negative section coords; the map walk
/// reads the raw truncating form because a player position can sit
/// fractionally outside the world (negative tile → section 0, not -1).
fn section_x(tile_x: i32) -> i32 {
    tile_x / SECTION_W
}

/// `Netplay.GetSectionY` (`Netplay.cs:775-778`).
fn section_y(tile_y: i32) -> i32 {
    tile_y / SECTION_H
}

/// `WorldSections.BitIndex_*` (`WorldSections.cs:33-39`).
const BIT_LOADED: u8 = 1 << 0;
const BIT_FRAMED: u8 = 1 << 1;
const BIT_MAP_DRAWN: u8 = 1 << 2;
const BIT_NEEDS_REFRESH: u8 = 1 << 3;

/// `WorldSections.IterationState` (`WorldSections.cs:8-31`): the map-draw
/// spiral's resume state. `Reset` (`:22-30`) parks the centre at
/// `(-3200, -2400)` — tiles, pre-`/16` — with every walk field zeroed.
#[derive(Clone, Copy, Debug, PartialEq)]
struct IterationState {
    center: [f32; 2],
    x: i32,
    y: i32,
    leg: i32,
    x_dir: i32,
    y_dir: i32,
}

impl IterationState {
    /// `IterationState.Reset` (`WorldSections.cs:22-30`).
    fn reset(&mut self) {
        self.center = [-3200.0, -2400.0];
        self.x = 0;
        self.y = 0;
        self.leg = 0;
        self.x_dir = 0;
        self.y_dir = 0;
    }
}

impl Default for IterationState {
    fn default() -> Self {
        let mut s = Self {
            center: [0.0; 2],
            x: 0,
            y: 0,
            leg: 0,
            x_dir: 0,
            y_dir: 0,
        };
        s.reset();
        s
    }
}

/// `Terraria.WorldSections` — `Main.sectionManager`.
pub struct MapSections {
    width: i32,
    height: i32,
    /// `BitsByte[] data` (`WorldSections.cs:45`), row-major `y * width + x`.
    data: Vec<u8>,
    /// `mapSectionsLeft` (`:47`): sections not yet drawn this sweep.
    map_sections_left: i32,
    /// `frameSectionsLeft` (`:49`) — `AnyUnfinishedSections` (`:57`).
    frame_sections_left: i32,
    /// `_sectionsNeedingRefresh` (`:51`) — `AnyNeedRefresh` (`:59`).
    sections_needing_refresh: i32,
    /// `prevMap` (`:55`): the map-draw spiral's resume state. `prevFrame`
    /// (`:53`) belongs to the tile-framing walk, which is out of this
    /// module's scope (the framing pass frames the resident world at load).
    prev_map: IterationState,
}

impl MapSections {
    /// `new WorldSections(numSectionsX, numSectionsY)`
    /// (`WorldSections.cs:61-69`) at `WorldGen.cs:7150`'s dimensions
    /// (`maxTilesX / 200`, `maxTilesY / 150`, integer division). The
    /// constructor arms every section loaded + framed: the port's world is
    /// fully resident, the state the C# reaches via `SetAllSectionsLoaded`
    /// (`WorldGen.cs:6744`/`:6821`) + the framing walk's `SetSectionFramed`
    /// (`WorldGen.cs:80363`).
    pub fn new(world_w: i32, world_h: i32) -> Self {
        let width = world_w / SECTION_W;
        let height = world_h / SECTION_H;
        let n = (width.max(0) * height.max(0)) as usize;
        // `SetSectionLoaded` (`:181-193`) counts every first load into
        // `frameSectionsLeft`, then `SetSectionFramed` (`:159-171`) drains
        // it back — a fully framed resident world sits at zero.
        Self {
            width,
            height,
            data: vec![BIT_LOADED | BIT_FRAMED; n],
            map_sections_left: n as i32,
            frame_sections_left: 0,
            sections_needing_refresh: 0,
            prev_map: IterationState::default(),
        }
    }

    fn in_grid(&self, x: i32, y: i32) -> bool {
        x >= 0 && x < self.width && y >= 0 && y < self.height
    }

    fn bits(&self, x: i32, y: i32) -> u8 {
        self.data[(y * self.width + x) as usize]
    }

    /// `WorldSections.SectionLoaded` (`WorldSections.cs:110-121`) — the
    /// bounds-checked bit 0 read `WorldMap.UpdateType`'s `TileLoaded` gate
    /// (`WorldMap.cs:81`) resolves through.
    pub fn section_loaded(&self, x: i32, y: i32) -> bool {
        self.in_grid(x, y) && self.bits(x, y) & BIT_LOADED != 0
    }

    /// `WorldSections.TileLoaded` (`WorldSections.cs:105-108`): the tile's
    /// section, via `Netplay.GetSectionX/Y` (`Netplay.cs:770-777` — the
    /// truncating `/ 200` / `/ 150`).
    pub fn tile_loaded(&self, tile_x: i32, tile_y: i32) -> bool {
        self.section_loaded(section_x(tile_x), section_y(tile_y))
    }

    /// `WorldSections.SectionFramed` (`WorldSections.cs:123-134`).
    pub fn section_framed(&self, x: i32, y: i32) -> bool {
        self.in_grid(x, y) && self.bits(x, y) & BIT_FRAMED != 0
    }

    /// `WorldSections.MapSectionDrawn` (`WorldSections.cs:136-147`).
    pub fn map_section_drawn(&self, x: i32, y: i32) -> bool {
        self.in_grid(x, y) && self.bits(x, y) & BIT_MAP_DRAWN != 0
    }

    /// `WorldSections.SectionNeedsRefresh` (`WorldSections.cs:80-91`).
    pub fn section_needs_refresh(&self, x: i32, y: i32) -> bool {
        self.in_grid(x, y) && self.bits(x, y) & BIT_NEEDS_REFRESH != 0
    }

    /// `WorldSections.SetSectionAsRefreshed` (`WorldSections.cs:71-78`):
    /// the bounds-and-bit gate, then the counter decrement.
    pub fn set_section_as_refreshed(&mut self, x: i32, y: i32) {
        if self.section_needs_refresh(x, y) {
            self.data[(y * self.width + x) as usize] &= !BIT_NEEDS_REFRESH;
            self.sections_needing_refresh -= 1;
        }
    }

    /// `WorldSections.SetAllFramedSectionsAsNeedingRefresh`
    /// (`WorldSections.cs:93-103`): every framed section arms bit 3 and
    /// bumps the counter — verbatim, including the no-dedupe increment
    /// (a second call counts the same sections again; vanilla does too).
    pub fn set_all_framed_sections_as_needing_refresh(&mut self) {
        for b in self.data.iter_mut() {
            if *b & BIT_FRAMED != 0 {
                *b |= BIT_NEEDS_REFRESH;
                self.sections_needing_refresh += 1;
            }
        }
    }

    /// `WorldSections.AnyNeedRefresh` (`WorldSections.cs:59`).
    pub fn any_need_refresh(&self) -> bool {
        self.sections_needing_refresh > 0
    }

    /// `WorldSections.AnyUnfinishedSections` (`WorldSections.cs:57`).
    pub fn any_unfinished_sections(&self) -> bool {
        self.frame_sections_left > 0
    }

    /// `WorldSections.ClearMapDraw` (`WorldSections.cs:149-157`): every
    /// drawn bit clears, the spiral resets, the budget refills — the
    /// `Main.refreshMap` consume (`Main.cs:61888-61892`).
    pub fn clear_map_draw(&mut self) {
        for b in self.data.iter_mut() {
            *b &= !BIT_MAP_DRAWN;
        }
        self.prev_map.reset();
        self.map_sections_left = self.data.len() as i32;
    }

    /// `WorldSections.GetNextMapDraw` (`WorldSections.cs:218-358`) —
    /// verbatim. `player_pos` is `Main.player[myPlayer].position` in px
    /// (`Main.cs:61871`); the walk runs in tile units (`*= 0.0625f`, `:229`)
    /// and spirals outward from the player's section, resuming across calls
    /// through `prevMap`. Returns the next undrawn section and marks it
    /// drawn (`:349-350`); `None` once the sweep is exhausted (`:220-225`).
    /// The C#'s infinite-loop guard throws (`:266-269`); the port panics
    /// with the same message — reachable only on corrupt state, never on
    /// the real walk.
    pub fn get_next_map_draw(&mut self, player_pos: [f32; 2]) -> Option<(i32, i32)> {
        if self.map_sections_left <= 0 {
            return None; // `:220-225` (the C# outs -1, -1)
        }
        // `:226-233`
        let mut num = 0;
        let mut num2 = 0;
        let mut vector = self.prev_map.center;
        let player = [player_pos[0] * 0.0625, player_pos[1] * 0.0625];
        let sec_x = section_x(player[0] as i32);
        let sec_y = section_y(player[1] as i32);
        let mut num3 = section_x(vector[0] as i32);
        let mut num4 = section_y(vector[1] as i32);
        // `:234-251`: a new centre section restarts the spiral there
        // (leg 4 = the corner kick-off); the same section resumes the walk.
        let (mut x, mut y, mut num5);
        if num3 != sec_x || num4 != sec_y {
            vector = player;
            num3 = sec_x;
            num4 = sec_y;
            num5 = 4;
            x = sec_x;
            y = sec_y;
        } else {
            num5 = self.prev_map.leg;
            x = self.prev_map.x;
            y = self.prev_map.y;
            num = self.prev_map.x_dir;
            num2 = self.prev_map.y_dir;
        }
        // `:252-258`: the quadrant pick — the offset from the centre
        // section's midpoint, truncating cast.
        let mut num6 = (player[0] - (num3 as f32 + 0.5) * 200.0) as i32;
        let mut num7 = (player[1] - (num4 as f32 + 0.5) * 150.0) as i32;
        if num == 0 {
            num = if num6 <= 0 { 1 } else { -1 };
            num2 = if num7 <= 0 { 1 } else { -1 };
        }
        let mut num8 = 0;
        let mut flag = false; // `:260`
        let mut flag2 = false;
        loop {
            // `:264-279`: the full-lap fallback — restart from the centre
            // with a fresh quadrant pick; a second full lap is the C#'s
            // "Infinite loop" exception.
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
            // `:280-287`: an in-grid undrawn section is the answer.
            if self.in_grid(x, y) {
                flag = false;
                if self.bits(x, y) & BIT_MAP_DRAWN == 0 {
                    break;
                }
            }
            // `:288-347`: the spiral step.
            let num9 = x - num3;
            let num10 = y - num4;
            if num9 == 0 || num10 == 0 {
                if num5 == 4 {
                    // `:292-319`: leg 4 kicks off the ring — at the centre
                    // step off the longer axis; on an axis extend outward.
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
                    // `:320-333`: on an axis, turn; then step diagonally.
                    if num9 != 0 {
                        num = if num9 <= 0 { 1 } else { -1 };
                    } else {
                        num2 = if num10 <= 0 { 1 } else { -1 };
                    }
                    x += num;
                    y += num2;
                    num5 += 1;
                }
                if flag {
                    num8 += 1;
                } else {
                    flag = true;
                }
            } else {
                x += num;
                y += num2;
            }
        }
        // `:349-357`: mark drawn, spend the budget, persist the resume.
        self.data[(y * self.width + x) as usize] |= BIT_MAP_DRAWN;
        self.map_sections_left -= 1;
        self.prev_map.center = player;
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

    /// `WorldGen.cs:7150`: `maxTilesX / 200` x `maxTilesY / 150`, integer
    /// division truncating — a 4200x1200 world is 21x8, and the 199x149
    /// remainder is NOT a section (the wire's round-up `net::sections_x/y`
    /// is a different quantity).
    #[test]
    fn w4_e18_grid_dims_truncate_and_every_section_arms_loaded_framed() {
        let s = MapSections::new(4200, 1200);
        assert_eq!((s.width, s.height), (21, 8));
        assert_eq!(s.map_sections_left, 21 * 8);
        assert_eq!(s.frame_sections_left, 0, "framed drains the load count");
        assert!(!s.any_unfinished_sections());
        assert!(!s.any_need_refresh());
        for y in 0..8 {
            for x in 0..21 {
                assert!(s.section_loaded(x, y));
                assert!(s.section_framed(x, y));
                assert!(!s.map_section_drawn(x, y));
            }
        }
        // The truncating remainder belongs to no section.
        let small = MapSections::new(199, 149);
        assert_eq!((small.width, small.height), (0, 0));
        assert_eq!(small.get_next_map_draw([0.0, 0.0]), None);
    }

    /// `IterationState.Reset` (`WorldSections.cs:22-30`) parks the centre at
    /// `(-3200, -2400)` — off-grid — so the first `GetNextMapDraw` takes the
    /// new-centre arm (`:235-243`) and answers the player's OWN section.
    #[test]
    fn w4_e18_the_first_draw_is_the_players_own_section() {
        let mut s = MapSections::new(4200, 1200);
        assert_eq!(s.prev_map.center, [-3200.0, -2400.0]);
        // Player at tile (1050, 620) → px (16800, 9920) → section (5, 4).
        let got = s.get_next_map_draw([16800.0, 9920.0]);
        assert_eq!(got, Some((5, 4)));
        assert!(s.map_section_drawn(5, 4));
        assert_eq!(s.map_sections_left, 21 * 8 - 1);
        // The resume state persists the scaled centre (`:351`, px * 0.0625).
        assert_eq!(s.prev_map.center, [1050.0, 620.0]);
    }

    /// The spiral covers every section exactly once across resumed calls
    /// (`prevMap`, `:244-250`), then outs `None` at the exhausted budget
    /// (`:220-225`). Same centre → the walk resumes, never restarts.
    #[test]
    fn w4_e18_the_spiral_covers_every_section_once_then_exhausts() {
        let mut s = MapSections::new(1000, 600); // 5x4 = 20 sections
        let pos = [400.0 * 16.0, 300.0 * 16.0]; // centre section (2, 2)
        let mut seen = std::collections::HashSet::new();
        for _ in 0..20 {
            let (x, y) = s.get_next_map_draw(pos).expect("budget remains");
            assert!(seen.insert((x, y)), "no section draws twice");
        }
        assert_eq!(seen.len(), 20);
        assert_eq!(s.map_sections_left, 0);
        assert_eq!(s.get_next_map_draw(pos), None, "`:220-225`");
        assert_eq!(s.get_next_map_draw(pos), None, "stays exhausted");
    }

    /// A new centre section restarts the spiral at the player's section
    /// (`:235-243`): the first answer after a move is the new own section,
    /// even mid-sweep.
    #[test]
    fn w4_e18_moving_to_a_new_section_restarts_the_spiral_there() {
        let mut s = MapSections::new(1000, 600);
        let a = [100.0 * 16.0, 100.0 * 16.0]; // section (0, 0)
        assert_eq!(s.get_next_map_draw(a), Some((0, 0)));
        assert_eq!(s.get_next_map_draw(a).is_some(), true); // resumed walk
        let b = [700.0 * 16.0, 400.0 * 16.0]; // section (3, 2)
        assert_eq!(s.get_next_map_draw(b), Some((3, 2)), "restart at (3,2)");
        // The first answer breaks before any step, so leg stays 4 (`:240`);
        // the kick-off zeroes it on the NEXT call's step (`:316`).
        assert_eq!(s.prev_map.leg, 4);
        assert!(s.get_next_map_draw(b).is_some());
        assert_eq!(s.prev_map.leg, 0, "leg 4 kicks off then zeroes (`:316`)");
    }

    /// `ClearMapDraw` (`WorldSections.cs:149-157`): every drawn bit clears,
    /// the spiral resets to the off-grid centre, the budget refills — the
    /// next draw is the player's own section again.
    #[test]
    fn w4_e18_clear_map_draw_refills_the_budget_and_resets_the_spiral() {
        let mut s = MapSections::new(600, 300); // 3x2
        let pos = [100.0 * 16.0, 100.0 * 16.0];
        while s.get_next_map_draw(pos).is_some() {}
        assert_eq!(s.map_sections_left, 0);
        s.clear_map_draw();
        assert_eq!(s.map_sections_left, 6);
        assert_eq!(s.prev_map.center, [-3200.0, -2400.0]);
        for y in 0..2 {
            for x in 0..3 {
                assert!(!s.map_section_drawn(x, y));
            }
        }
        assert_eq!(s.get_next_map_draw(pos), Some((0, 0)));
    }

    /// `SetAllFramedSectionsAsNeedingRefresh` (`:93-103`) counts WITHOUT
    /// dedupe — a second call counts the same sections again (vanilla does
    /// too); `SetSectionAsRefreshed` (`:71-78`) decrements only while the
    /// bit is armed, and bounds-checks.
    #[test]
    fn w4_e18_refresh_arming_double_counts_and_refreshing_decrements_once() {
        let mut s = MapSections::new(600, 300); // 6 framed sections
        s.set_all_framed_sections_as_needing_refresh();
        assert_eq!(s.sections_needing_refresh, 6);
        assert!(s.any_need_refresh());
        s.set_all_framed_sections_as_needing_refresh();
        assert_eq!(s.sections_needing_refresh, 12, "no dedupe, verbatim");
        s.set_section_as_refreshed(1, 1);
        assert_eq!(s.sections_needing_refresh, 11);
        assert!(!s.section_needs_refresh(1, 1));
        s.set_section_as_refreshed(1, 1);
        assert_eq!(s.sections_needing_refresh, 11, "the bit gate holds");
        s.set_section_as_refreshed(-1, 0);
        s.set_section_as_refreshed(99, 99);
        assert_eq!(s.sections_needing_refresh, 11, "out of grid is a no-op");
    }

    /// `TileLoaded` (`:105-108`) rides `Netplay.GetSectionX/Y`
    /// (`Netplay.cs:770-777`) — C# integer division TRUNCATES toward zero,
    /// so a fractionally-negative tile position reads section 0, not -1
    /// (the wire's `div_euclid` form would answer -1 → not loaded).
    #[test]
    fn w4_e18_tile_loaded_truncates_negative_positions_to_section_zero() {
        let s = MapSections::new(600, 300);
        assert!(s.tile_loaded(-1, -1), "(-1)/200 == 0 in C#");
        assert!(s.tile_loaded(-199, -149));
        assert!(s.tile_loaded(599, 299));
        assert!(!s.tile_loaded(600, 0), "section 3 is out of the 3-wide grid");
        assert!(!s.tile_loaded(0, 300));
        assert!(!s.section_loaded(-1, 0));
        assert!(!s.section_framed(0, -1));
        assert!(!s.map_section_drawn(3, 0));
        assert!(!s.section_needs_refresh(0, 2));
    }

    /// The quadrant pick (`:252-258`): the offset from the centre section's
    /// midpoint decides the spiral's first direction — a player right of
    /// the midpoint walks `xDir = -1` (back toward the centre first), left
    /// of it walks `+1`; the `<= 0` tie goes positive.
    #[test]
    fn w4_e18_the_quadrant_pick_reads_the_section_midpoint_offset() {
        // Section (0,0) midpoint is tile (100, 75). Player at tile 150
        // (right of it) → num6 > 0 → xDir = -1; the leg-4 kick-off steps
        // off the longer axis (|num6| > |num7| → y -= yDir).
        let mut s = MapSections::new(600, 300);
        let pos = [150.0 * 16.0, 75.0 * 16.0];
        let first = s.get_next_map_draw(pos).unwrap();
        assert_eq!(first, (0, 0));
        assert_eq!(s.prev_map.x_dir, -1, "right of midpoint → -1");
        assert_eq!(s.prev_map.y_dir, 1, "on the midpoint → +1 (`<= 0`)");
        let mut t = MapSections::new(600, 300);
        let pos = [50.0 * 16.0, 75.0 * 16.0];
        t.get_next_map_draw(pos).unwrap();
        assert_eq!(t.prev_map.x_dir, 1, "left of midpoint → +1");
    }
}