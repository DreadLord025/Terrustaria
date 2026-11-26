//! `Main.oldDrawWater`'s ambient-lava scan (`Main.cs:57615-57634` + the
//! `:57932-57936` store) — the RETRO liquid path's `Main.ambientLava*`
//! writer. `Main.DrawLiquid` routes `!Lighting.NotRetro` (Retro/Trippy,
//! `Lighting.cs:63-73`) to `oldDrawWater` (`Main.cs:57519-57522`); its lava
//! arm sums the near-screen radial strength of every lit, uncovered lava
//! cell and tracks the per-axis nearest cell. The port's liquid passes are
//! the `LiquidRenderer` path in every mode, so the DRAW halves of
//! `oldDrawWater` stay unported — this scan is the one piece with an
//! observable consumer (`Main.Ambience`'s lavafall arm reads the triple,
//! `Main.cs:13488-13497`) and rides the retro modes only (W4-g73 closeout).

use crate::tile::LiquidKind;
use crate::tiledata as td;
use crate::waterfall::{AmbientScan, ScreenInfo};
use crate::world::World;
use crate::lighting::{LightMap, GLOBAL_BRIGHTNESS};

/// `Lighting.Brightness` (`Lighting.cs:118-122`):
/// `GlobalBrightness * (r + g + b) / 3`.
fn brightness(light: &LightMap, x: i32, y: i32) -> f32 {
    let c = light.get(x, y);
    GLOBAL_BRIGHTNESS * (c[0] + c[1] + c[2]) / 3.0
}

/// The ambient-lava half of `Main.oldDrawWater` (`Main.cs:57553-57634` +
/// `:57932-57936`). Returns the `Main.ambientLavaStrength/X/Y` triple in the
/// port's `(strength, x, y)` shape; the empty scan returns the C# init
/// `(0, -1, -1)` (`:57555-57559`).
///
/// `drew_lava` is `Main.drewLava` at entry — `DrawWaters` resets it false
/// (`Main.cs:57472`) before the pass sequence, and the last (non-waterOnly)
/// pass stores the triple only while it is still false (`:57932`), then
/// sets it true (`:57938`). The caller keeps that flag across the passes.
pub fn ambient_lava_scan(
    world: &World,
    light: &LightMap,
    screen: ScreenInfo,
    bg: bool,
    drew_lava: bool,
) -> AmbientScan {
    // `:57555-57559`: num = strength 0, num2/num3 = 99999 sentinels,
    // num4/num5 = -1 (the load-reset shape, `Main.cs:2213-2217`).
    let mut num = 0.0f32;
    let mut num2 = 99999.0f32;
    let mut num3 = 99999.0f32;
    let mut num4 = -1i32;
    let mut num5 = -1i32;
    // `:57560-57564`: the offScreenRange vector — 192 into the padded
    // targets (`Main.cs:58175`), zero on the direct-to-screen path. The
    // port draws into the padded targets, so the constant rides 192 (the
    // `render::bg::OFF_SCREEN_RANGE` convention).
    const OFF_SCREEN_RANGE: f32 = 192.0;
    // `:57573-57576`: the four (int) truncating casts, margins verbatim.
    let mut num7 = ((screen.position[0] - OFF_SCREEN_RANGE) / 16.0 - 1.0) as i32;
    let mut num8 = ((screen.position[0] + screen.width + OFF_SCREEN_RANGE) / 16.0) as i32 + 2;
    let mut num9 = ((screen.position[1] - OFF_SCREEN_RANGE) / 16.0 - 1.0) as i32;
    let mut num10 = ((screen.position[1] + screen.height + OFF_SCREEN_RANGE) / 16.0) as i32 + 5;
    // `:57577-57592`: the 5-tile world-edge clamps.
    num7 = num7.max(5);
    num8 = num8.min(world.width - 5);
    num9 = num9.max(5);
    num10 = num10.min(world.height - 5);
    // `:57593-57595`: row-major, +4 rows below / ±2 columns outside.
    for i in num9..num10 + 4 {
        for j in num7 - 2..num8 + 2 {
            let Some(tile) = world.tile(j, i) else {
                continue;
            };
            // `:57601`: dry, covered-by-solid, or unlit cells skip (the bg
            // pass ORs the brightness gate away). The covered test is the
            // RAW `nactive && tileSolid && !tileSolidTop` — no half-brick
            // or slope terms, unlike `WorldGen.SolidTile`.
            let covered = tile.active()
                && !tile.inactive()
                && td::TILE_SOLID.has(tile.kind)
                && !td::TILE_SOLID_TOP.has(tile.kind);
            if tile.liquid == 0 || covered || !(brightness(light, j, i) > 0.0 || bg) {
                continue;
            }
            // `:57609-57635`: only the lava arm feeds the triple (the
            // honey/shimmer/water arms set draw types the port's
            // LiquidRenderer path owns).
            if tile.liquid_kind() != LiquidKind::Lava {
                continue;
            }
            // `:57611-57614`: a prior pass already drew lava — skip.
            if drew_lava {
                continue;
            }
            // `:57615-57616`: per-axis distance to the INT-halved screen
            // centre (the `(int)(screenWidth / 2)` truncation).
            let num13 = ((j * 16 + 8) as f32
                - (screen.position[0] + (screen.width as i32 / 2) as f32))
                .abs();
            let num14 = ((i * 16 + 8) as f32
                - (screen.position[1] + (screen.height as i32 / 2) as f32))
                .abs();
            // `:57617-57624`: inside the 2x-screen box, the radial falloff
            // `1 - dist / (screenWidth * 0.75)` sums when positive.
            if num13 < screen.width * 2.0 && num14 < screen.height * 2.0 {
                let num15 =
                    1.0 - (num13 * num13 + num14 * num14).sqrt() / (screen.width * 0.75);
                if num15 > 0.0 {
                    num += num15;
                }
            }
            // `:57625-57629`: the nearest X tracks its own axis.
            if num13 < num2 {
                num2 = num13;
                num4 = j * 16 + 8;
            }
            // `:57630-57634`: the Y quirk VERBATIM — the test reads num14
            // but the store copies num13 (the X distance), so num3 is an
            // X-distance running minimum while num5 tracks the row.
            if num14 < num3 {
                num3 = num13;
                num5 = i * 16 + 8;
            }
        }
    }
    // `:57932-57936`: the store is gated on !drewLava; a gated pass leaves
    // the previous triple (the port returns the load-reset shape and the
    // caller keeps its live value).
    if drew_lava {
        return AmbientScan::default();
    }
    AmbientScan { x: num4 as f32, y: num5 as f32, strength: num }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lighting::LightParams;
    use crate::tile::{Tile, FLAG_ACTIVE, FLAG_LIQUID_LAVA};

    fn world_with_lava() -> World {
        let mut w = World::new("test".into(), 80, 60);
        w.world_surface = 10.0;
        let i = w.index(10, 10);
        let mut t = Tile::default();
        t.liquid = 255;
        t.flags |= FLAG_LIQUID_LAVA;
        w.tiles[i] = t;
        w
    }

    fn lit_map(w: &World) -> LightMap {
        let mut params = LightParams::default();
        params.sky = [1.0, 1.0, 1.0];
        LightMap::compute(w, 5, 5, 40, 40, &params)
    }

    fn screen_at(x: f32, y: f32) -> ScreenInfo {
        ScreenInfo { position: [x, y], width: 640.0, height: 640.0 }
    }

    /// `Main.cs:57615-57634` hand-replay: one lit lava cell at (10, 10),
    /// screen centred on its centre (168, 168) — distance 0, strength 1,
    /// the triple tracks the cell centre on both axes.
    #[test]
    fn one_lit_lava_cell_sums_full_strength_at_the_screen_centre() {
        let w = world_with_lava();
        let light = lit_map(&w);
        let scan = ambient_lava_scan(&w, &light, screen_at(168.0 - 320.0, 168.0 - 320.0), false, false);
        assert_eq!(scan.x, 168.0);
        assert_eq!(scan.y, 168.0);
        assert!((scan.strength - 1.0).abs() < 1e-6);
    }

    /// The `:57630-57634` Y quirk: the nearest-Y test reads the Y distance
    /// but stores the X distance into num3 — a second cell nearer in Y but
    /// FARTHER in X still wins the Y slot (its num14 < the stored X
    /// distance), while a cell nearer in X updates num3 to the smaller X.
    /// Two cells: (10, 10) centre-distance-wise at the screen centre, and
    /// (30, 11) — dx = 320, dy = 16. num3 after cell 1 = 0 (its X distance),
    /// so cell 2's 16 < 0 is false: the Y slot stays on row 10.
    #[test]
    fn the_y_slot_keeps_the_num3_equals_num13_quirk() {
        let mut w = world_with_lava();
        let i = w.index(30, 11);
        let mut t = Tile::default();
        t.liquid = 255;
        t.flags |= FLAG_LIQUID_LAVA;
        w.tiles[i] = t;
        let light = lit_map(&w);
        let scan = ambient_lava_scan(&w, &light, screen_at(168.0 - 320.0, 168.0 - 320.0), false, false);
        // Row-major order visits (10,10) first: num3 = 0 blocks every later
        // Y update (no Y distance can beat 0) — num5 stays 168.
        assert_eq!(scan.y, 168.0);
        assert_eq!(scan.x, 168.0);
        // Both cells contribute: cell 2 dist = sqrt(320² + 16²) ≈ 320.4,
        // strength 1 - 320.4/480 ≈ 0.3325.
        let d2 = (320.0f32 * 320.0 + 16.0 * 16.0).sqrt();
        assert!((scan.strength - (1.0 + 1.0 - d2 / 480.0)).abs() < 1e-4);
    }

    /// `:57601` gates: a dry world stores the init triple; a solid-covered
    /// lava cell skips; an unlit cell skips on the foreground pass but the
    /// bg pass ORs the brightness gate away. `:57611`: a prior drewLava
    /// pass skips the arm AND the store (`:57932`).
    #[test]
    fn the_dry_covered_unlit_and_drew_lava_gates_skip() {
        let mut dry = World::new("test".into(), 80, 60);
        dry.world_surface = 10.0;
        let light = lit_map(&dry);
        let screen = screen_at(0.0, 0.0);
        let empty = ambient_lava_scan(&dry, &light, screen, false, false);
        assert_eq!(empty, AmbientScan::default());
        // Covered: an active stone ABOVE-surface with lava in the same cell
        // (the raw covered test ignores the liquid) — the cell skips.
        let mut w = world_with_lava();
        let i = w.index(10, 10);
        w.tiles[i].kind = 1;
        w.tiles[i].flags |= FLAG_ACTIVE;
        let scan = ambient_lava_scan(&w, &lit_map(&w), screen, false, false);
        assert_eq!(scan, AmbientScan::default());
        // drewLava at entry: the arm skips and the store is gated.
        let w2 = world_with_lava();
        let scan = ambient_lava_scan(&w2, &lit_map(&w2), screen, false, true);
        assert_eq!(scan, AmbientScan::default());
        // Unlit foreground: with an EMPTY light map every cell reads black
        // (`LightMap::get` returns 0 outside its range) — the fg pass skips
        // the lava cell, the bg pass ORs the brightness gate away.
        let w3 = world_with_lava();
        let dark = LightMap::new();
        let fg = ambient_lava_scan(&w3, &dark, screen, false, false);
        let bg = ambient_lava_scan(&w3, &dark, screen, true, false);
        assert_eq!(fg, AmbientScan::default(), "the unlit cell skips the fg pass");
        assert!(bg.strength > 0.0, "the bg pass ORs the brightness gate away");
    }
}
