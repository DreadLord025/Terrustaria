//! The tile-crack overlay — `Main.DrawTileCracks` (`Main.cs:61533-61626`):
//! the mining-progress crack drawn on damaged tiles from the local player's
//! `HitTile` books. The solid-layer draw sites are
//! `TileDrawing.cs:732-733` (`hitReplace` first, then `hitTile`, crack type
//! 1); the wall sites are `WallDrawing.cs:203-204` (crack type 2).
//!
//! S1-G-1 wave C; W4-g77 re-armed onto the i29 `HitTile` book. The draw
//! pass is the verbatim `DrawTileCracks` walk over the REAL per-player
//! book (`App.hit_tile`, `tr_core::hit_tile::HitTile` — the S1-P-i29
//! mining hit-buffer): every one of the 501 `data` slots in index order
//! (`Main.cs:61541`), so abandoned hits keep drawing their crack stage
//! while the 60-tick TTL decay (`HitTile.cs:285-333`) drains the damage
//! back under the 20 floor. Recorded deviations (ANALYSIS "The tile-crack
//! overlay (S1-G-1 wave C)" + "The book-driven crack draw (W4-g77)"):
//! - `hitReplace` draws nothing until replacement mining lands (the port
//!   has no second book); the wall-layer sites (`WallDrawing.cs:203-204`,
//!   crack type 2) await the wall-layer overlay wiring — the type-2 plan
//!   arm is ported and tested here;
//! - `HitTileObject.Clear`'s `crackStyle` roll (`HitTile.cs:29-46`, a static
//!   `UnifiedRandom` + the no-repeat-of-`lastCrack` loop) is seeded from the
//!   world seed instead of `DateTime.Now.Ticks` (the port's determinism
//!   rule, same deviation class as `NamedRngs::global`). The sim book is
//!   RNG-free (i29), so [`CrackStyles`] keeps the per-slot styles render-
//!   side and rolls when a slot takes a NEW occupancy (the C# rolls on
//!   `Clear`, i.e. just before the reuse — one roll per occupancy either
//!   way; a clear that is never reused draws no roll in the port);
//! - `Main.ShouldShowInvisibleBlocksAndWalls` (`Main.cs:58023-58030`) reads
//!   the echo monolith / `CanSeeInvisibleBlocks` — the port stands it in
//!   with the `TR_SHOW_INVISIBLE` pin (`env::show_invisible`, the pylons
//!   pass's convention); the echo-coating bits are real
//!   (`World::invisible_block`/`invisible_wall`, `Tile.cs:528`/`:545`);
//! - the draw rides the per-frame world range right after the cached tile
//!   pass (`frame/draw.rs`), the port's stand-in for the solid-layer site —
//!   the crack must redraw every frame while the damage decays, so it
//!   cannot live inside the G6-cached range (the `WorldFrameKey` is
//!   untouched).

use glam::Vec2;

use tr_core::hit_tile::{HitTile, MAX_HITTILES};
use tr_core::lighting::{LightMap, LightStyle};
use tr_core::tiledata::{SET_IS_A_TREE_TRUNK, TILE_SOLID, TILE_SOLID_TOP};
use tr_core::{TILE_PX, UnifiedRandom, World};
use tr_render::sprite::SpriteRenderer;

/// `HitTile.TILE` (`HitTile.cs:72`).
pub(crate) const CRACK_TILE: i32 = 1;
/// `HitTile.WALL` (`HitTile.cs:74`).
pub(crate) const CRACK_WALL: i32 = 2;

/// `HitTile.HitTileObject`'s draw inputs (`HitTile.cs:11-27`): the cell, the
/// accumulated damage, the book's hit type and the crack-style column.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct CrackRecord {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) damage: i32,
    /// `HitTileObject.type` — 1 = tile book, 2 = wall book.
    pub(crate) kind: i32,
    pub(crate) crack_style: i32,
}

/// `HitTileObject.Clear`'s style roll (`HitTile.cs:40-45`): `rand.Next(4)`
/// re-rolled while it equals `lastCrack`. The port seeds the stream from the
/// world seed (see the module deviations) and keeps the last style per book.
pub(crate) fn roll_crack_style(rng: &mut UnifiedRandom, last_crack: &mut i32) -> i32 {
    let mut style = rng.next_max(4);
    while style == *last_crack {
        style = rng.next_max(4);
    }
    *last_crack = style;
    style
}

/// The `flag3`/`flag4` tile-shape classification (`Main.cs:61571-61598`):
/// does the crack draw on this tile, and does it use the tree columns?
/// Returns `(draw, tree_style)`.
pub(crate) fn crack_tile_shape(kind: u16, frame_x: i32, frame_y: i32) -> (bool, bool) {
    let mut flag3 = false;
    let mut flag4 = false;
    if kind == 10 {
        // The door never cracks (`:61573-61576`).
        flag3 = false;
    } else if TILE_SOLID.has(kind) && !TILE_SOLID_TOP.has(kind) {
        flag3 = true;
    } else if SET_IS_A_TREE_TRUNK.has(kind) {
        // `WorldGen.IsTreeType` (`WorldGen.cs`, `TileID.Sets.IsATreeTrunk`):
        // the trunk cells crack, the branch/root cells do not — the 22 px
        // frame-grid exclusion windows (`:61584-61589`).
        flag4 = true;
        let num = frame_x / 22;
        let num2 = frame_y / 22;
        if num2 < 9 {
            flag3 = ((num != 1 && num != 2) || num2 < 6 || num2 > 8)
                && (num != 3 || num2 > 2)
                && (num != 4 || num2 < 3 || num2 > 5)
                && (num != 5 || num2 < 6 || num2 > 8);
        }
    } else if kind == 72 {
        // The giant glowing mushroom's stem (`:61591-61598`). Kept verbatim
        // even though 72 is in `IsATreeTrunk` (bits 71-77), so the tree arm
        // above claims it first and this arm is unreachable in 1.4.5.8.
        flag4 = true;
        if frame_x <= 34 {
            flag3 = true;
        }
    }
    (flag3, flag4)
}

/// The damage → crack row ladder (`Main.cs:61601-61617`): <20 draws nothing
/// (checked by the caller), 20..39 row 0, 40..59 row 1, 60..79 row 2, 80+
/// row 3.
pub(crate) fn crack_row(damage: i32) -> i32 {
    if damage >= 80 {
        3
    } else if damage >= 60 {
        2
    } else if damage >= 40 {
        1
    } else {
        0
    }
}

/// The crack source rect (`Main.cs:61618-61622`): `crackStyle * 18` column
/// on the `row * 18` row of the 16 px `TileCrack` sheet; the tree arm
/// shifts to column `4 + crackStyle / 2`.
pub(crate) fn crack_source_rect(crack_style: i32, row: i32, tree_style: bool) -> [f32; 4] {
    let mut x = crack_style * 18;
    if tree_style {
        x = (4 + crack_style / 2) * 18;
    }
    [x as f32, (row * 18) as f32, 16.0, 16.0]
}

/// One record's draw decision — every `continue` arm of
/// `Main.DrawTileCracks` (`Main.cs:61541-61622`) plus the source rect, so
/// the whole pass is unit-testable without a renderer. `None` = the record
/// draws nothing; `Some((sx, sy, sw, sh))` = the `TileCrack` source rect.
/// `show_invisible` stands in for `Main.ShouldShowInvisibleBlocksAndWalls`
/// (`Main.cs:58023-58030`, the `TR_SHOW_INVISIBLE` pin — module deviations).
pub(crate) fn crack_draw_plan(
    world: &World,
    hit: &CrackRecord,
    crack_type: i32,
    show_invisible: bool,
) -> Option<[f32; 4]> {
    // `:61543-61551`: the book-type filter and the <20 damage skip.
    if hit.kind != crack_type {
        return None;
    }
    let damage = hit.damage;
    if damage < 20 {
        return None;
    }
    let (x, y) = (hit.x, hit.y);
    // `WorldGen.InWorld(x, y)` (`WorldGen.cs:9688-9695`, fluff 0).
    if x < 0 || x >= world.width || y < 0 || y >= world.height {
        return None;
    }
    let tile = world.tile(x, y)?;
    // `:61558-61566`: the tile arm needs an active block that is not
    // echo-hidden (`flag || !invisibleBlock()`), the wall arm a wall
    // that is not echo-hidden (`flag || !invisibleWall()`).
    let visible = match crack_type {
        CRACK_TILE => tile.active() && (show_invisible || !world.invisible_block(x, y)),
        CRACK_WALL => tile.wall != 0 && (show_invisible || !world.invisible_wall(x, y)),
        _ => false,
    };
    if !visible {
        return None;
    }
    let (flag3, flag4) = crack_tile_shape(tile.kind, tile.frame_x as i32, tile.frame_y as i32);
    // `:61599`: slopes and half-bricks never crack.
    if flag3 && tile.slope() == 0 && !tile.half_brick() {
        let row = crack_row(damage);
        Some(crack_source_rect(hit.crack_style, row, flag4))
    } else {
        None
    }
}

/// `Main.DrawTileCracks` (`Main.cs:61533-61626`) over one record list.
/// The `offScreenRange` vector (`:61535-61539`) is zero: the port draws the
/// world range straight to the screen-range coordinate space (the wires
/// pass's convention), so the C#'s `drawToScreen` arm is the one taken.
pub(crate) fn draw_tile_cracks(
    r: &mut SpriteRenderer,
    world: &World,
    light: &LightMap,
    style: &LightStyle,
    camera: Vec2,
    anim_time: f32,
    crack_type: i32,
    hitter: &[CrackRecord],
    show_invisible: bool,
) {
    for hit in hitter {
        let Some([sx, sy, sw, sh]) = crack_draw_plan(world, hit, crack_type, show_invisible) else {
            continue;
        };
        let Some(uv) = r.uv("TileCracks", sx, sy, sw, sh) else {
            continue;
        };
        // `Lighting.GetColor(x, y)` tint (`:61623`) — the port's light
        // style shade, the same seam every tile pass uses. The position is
        // the C#'s `x * 16 - (int)screenPosition.X` (`:61623`).
        let c = style.shade(light.get(hit.x, hit.y), hit.x, hit.y, anim_time);
        r.push_sprite_paint(
            [
                hit.x as f32 * TILE_PX - camera.x,
                hit.y as f32 * TILE_PX - camera.y,
            ],
            [TILE_PX, TILE_PX],
            uv,
            [c[0], c[1], c[2], 1.0],
            0.0,
            0,
        );
    }
}

/// The render-side per-slot `crackStyle` memory (`HitTile.cs:23`): the sim
/// book (`tr_core::hit_tile`) is RNG-free (i29), so the style rolls here
/// when a slot takes a NEW occupancy — the C# rolls on
/// `HitTileObject.Clear` (`HitTile.cs:40-45`), i.e. just before the reuse;
/// one roll per occupancy either way (a clear that is never reused draws
/// no roll in the port — the module deviation). Every new occupancy goes
/// through `Clear` → `Prepare` (`HitObject`, `HitTile.cs:159-179`), so the
/// `kind == 0` → `kind != 0` edge is the reuse detector; the style HOLDS
/// through `AddDamage`, the TTL decay and `UpdatePosition` (the
/// mannequin/weapon-rack redirects call `SetPosition` only, `:181-187` —
/// no Clear, no roll). `styles[i] == -1` = the slot is/was unoccupied.
#[derive(Clone, Debug)]
pub(crate) struct CrackStyles {
    pub(crate) styles: [i32; MAX_HITTILES + 1],
    /// `HitTile.lastCrack` (`HitTile.cs:82`, static ctor `:473-476`
    /// initialises it to -1).
    pub(crate) last_crack: i32,
}

impl Default for CrackStyles {
    fn default() -> Self {
        Self::new()
    }
}

impl CrackStyles {
    pub(crate) fn new() -> Self {
        Self {
            styles: [-1; MAX_HITTILES + 1],
            last_crack: -1,
        }
    }

    /// Refresh the per-slot styles against the book and return the draw
    /// records in the C#'s `hitter.data` INDEX order (`Main.cs:61541` —
    /// NOT the LRU order walk; the draw is index-order and the overlap of
    /// two records on one cell paints in slot order).
    pub(crate) fn records(&mut self, book: &HitTile, rng: &mut UnifiedRandom) -> Vec<CrackRecord> {
        let mut out = Vec::new();
        for i in 0..=MAX_HITTILES {
            let obj = &book.data[i];
            if obj.kind == 0 {
                // `HitTileObject.Clear` zeroed the slot — the next
                // `Prepare` rolls again (the C# rolls on the Clear itself).
                self.styles[i] = -1;
                continue;
            }
            if self.styles[i] < 0 {
                self.styles[i] = roll_crack_style(rng, &mut self.last_crack);
            }
            out.push(CrackRecord {
                x: obj.x,
                y: obj.y,
                damage: obj.damage,
                kind: obj.kind,
                crack_style: self.styles[i],
            });
        }
        out
    }
}

/// The frame hook: run both draw sites in the C# order — `hitReplace` then
/// `hitTile` (`TileDrawing.cs:732-733`) — over the local player's real
/// `HitTile` book (i29). The port has no replacement-mining book yet, so
/// the first site is empty (module deviations).
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_mining_cracks(
    r: &mut SpriteRenderer,
    world: &World,
    light: &LightMap,
    style: &LightStyle,
    camera: Vec2,
    anim_time: f32,
    book: &HitTile,
    crack_styles: &mut CrackStyles,
    rng: &mut UnifiedRandom,
    show_invisible: bool,
) {
    let records = crack_styles.records(book, rng);
    // `DrawTileCracks(1, hitReplace)` — empty in the port (module note).
    draw_tile_cracks(
        r,
        world,
        light,
        style,
        camera,
        anim_time,
        CRACK_TILE,
        &[],
        show_invisible,
    );
    // `DrawTileCracks(1, hitTile)`.
    draw_tile_cracks(
        r,
        world,
        light,
        style,
        camera,
        anim_time,
        CRACK_TILE,
        &records,
        show_invisible,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::tile::{FLAG_ACTIVE, FLAG_HALF_BRICK};

    fn world_with(kind: u16, x: i32, y: i32) -> World {
        let mut w = World::new("cracks".into(), 40, 40);
        let t = w.tile_mut(x, y).unwrap();
        t.kind = kind;
        t.flags |= FLAG_ACTIVE;
        w
    }

    fn rec(x: i32, y: i32, damage: i32, kind: i32, style: i32) -> CrackRecord {
        CrackRecord {
            x,
            y,
            damage,
            kind,
            crack_style: style,
        }
    }

    /// `Main.cs:61601-61617`: the four damage bands and the <20 skip.
    #[test]
    fn s1_g_1c_the_damage_ladder_picks_the_four_rows() {
        assert_eq!(crack_row(20), 0);
        assert_eq!(crack_row(39), 0);
        assert_eq!(crack_row(40), 1);
        assert_eq!(crack_row(59), 1);
        assert_eq!(crack_row(60), 2);
        assert_eq!(crack_row(79), 2);
        assert_eq!(crack_row(80), 3);
        assert_eq!(crack_row(100), 3);
    }

    /// `Main.cs:61618-61622`: the 18 px grid rects and the tree column shift.
    #[test]
    fn s1_g_1c_the_source_rects_ride_the_18px_grid() {
        assert_eq!(crack_source_rect(0, 0, false), [0.0, 0.0, 16.0, 16.0]);
        assert_eq!(crack_source_rect(3, 3, false), [54.0, 54.0, 16.0, 16.0]);
        // The tree arm: column 4 + style/2 (styles 0/1 share column 4,
        // styles 2/3 column 5).
        assert_eq!(crack_source_rect(0, 1, true), [72.0, 18.0, 16.0, 16.0]);
        assert_eq!(crack_source_rect(1, 1, true), [72.0, 18.0, 16.0, 16.0]);
        assert_eq!(crack_source_rect(2, 1, true), [90.0, 18.0, 16.0, 16.0]);
        assert_eq!(crack_source_rect(3, 1, true), [90.0, 18.0, 16.0, 16.0]);
    }

    /// `Main.cs:61573-61598`: the door skip, the solid/solid-top split, the
    /// tree frame windows and the glowing-mushroom stem.
    #[test]
    fn s1_g_1c_the_tile_shape_classification_is_verbatim() {
        // Door (10): never.
        assert_eq!(crack_tile_shape(10, 0, 0), (false, false));
        // Dirt (0) / stone (1): solid, not solid-top.
        assert_eq!(crack_tile_shape(0, 0, 0), (true, false));
        assert_eq!(crack_tile_shape(1, 0, 0), (true, false));
        // Platform (19): solid-top — no crack.
        assert_eq!(crack_tile_shape(19, 0, 0), (false, false));
        // Tree trunk (5): the trunk cells crack with the tree columns.
        assert_eq!(crack_tile_shape(5, 0, 0), (true, true));
        assert_eq!(crack_tile_shape(5, 66, 66), (true, true)); // col 3 row 3
        // The branch/root exclusion windows (`:61588`).
        assert_eq!(crack_tile_shape(5, 66, 0), (false, true)); // col 3 row 0
        assert_eq!(crack_tile_shape(5, 22, 132), (false, true)); // col 1 row 6
        assert_eq!(crack_tile_shape(5, 44, 176), (false, true)); // col 2 row 8
        assert_eq!(crack_tile_shape(5, 66, 44), (false, true)); // col 3 row 2
        assert_eq!(crack_tile_shape(5, 88, 66), (false, true)); // col 4 row 3
        assert_eq!(crack_tile_shape(5, 88, 110), (false, true)); // col 4 row 5
        assert_eq!(crack_tile_shape(5, 110, 132), (false, true)); // col 5 row 6
        assert_eq!(crack_tile_shape(5, 110, 176), (false, true)); // col 5 row 8
        // Outside the windows the same columns crack.
        assert_eq!(crack_tile_shape(5, 22, 110), (true, true)); // col 1 row 5
        assert_eq!(crack_tile_shape(5, 88, 132), (true, true)); // col 4 row 6
        // row >= 9: the C# leaves flag3 false.
        assert_eq!(crack_tile_shape(5, 0, 198), (false, true));
        // Giant glowing mushroom (72): it is in `TileID.Sets.IsATreeTrunk`
        // (bits 71-77), so the tree arm claims it BEFORE the C#'s
        // `type == 72` arm (`Main.cs:61591`, unreachable in 1.4.5.8 — the
        // port keeps the arm verbatim). The tree windows apply.
        assert_eq!(crack_tile_shape(72, 0, 0), (true, true));
        assert_eq!(crack_tile_shape(72, 36, 0), (true, true)); // col 1 row 0
        assert_eq!(crack_tile_shape(72, 22, 132), (false, true)); // col 1 row 6
        // A non-solid furniture tile (torch 4): no crack.
        assert_eq!(crack_tile_shape(4, 0, 0), (false, false));
    }

    /// `HitTile.cs:40-45`: the style is 0..3 and never repeats the last one.
    #[test]
    fn s1_g_1c_the_style_roll_never_repeats_the_last_crack() {
        let mut rng = UnifiedRandom::new(42);
        let mut last = -1;
        for _ in 0..200 {
            let prev = last;
            let style = roll_crack_style(&mut rng, &mut last);
            assert!((0..4).contains(&style));
            assert_ne!(style, prev, "the no-repeat loop must reroll");
            assert_eq!(style, last);
        }
        // Deterministic for the seed (the world-seeded deviation).
        let mut a = UnifiedRandom::new(7);
        let mut b = UnifiedRandom::new(7);
        let (mut la, mut lb) = (-1, -1);
        for _ in 0..20 {
            assert_eq!(
                roll_crack_style(&mut a, &mut la),
                roll_crack_style(&mut b, &mut lb)
            );
        }
    }

    /// W4-g77 — the book-driven records: every occupied slot draws a
    /// record in `data` INDEX order (`Main.cs:61541`), the style holds
    /// across the damage climb, and the WALL book keeps its own slot.
    #[test]
    fn w4_g77_the_records_walk_the_book_in_index_order() {
        let mut book = HitTile::new();
        let mut styles = CrackStyles::new();
        let mut rng = UnifiedRandom::new(3);
        // Two tile hits + one wall hit (i29's accumulation).
        let a = book.hit_object(5, 6, CRACK_TILE);
        book.add_damage(a, 40, true);
        let b = book.hit_object(7, 6, CRACK_TILE);
        book.add_damage(b, 85, true);
        let wid = book.hit_object(2, 2, CRACK_WALL);
        book.add_damage(wid, 60, true);
        let recs = styles.records(&book, &mut rng);
        assert_eq!(recs.len(), 3);
        // INDEX order (`Main.cs:61541`), NOT the LRU order: the i29
        // SortSlots rotations put the three occupancies at data slots
        // 0 (first hit), 500 (second), 499 (third) — the LRU order is
        // [499(buffer), wid, b, a] but the draw walks 0, 499, 500.
        assert_eq!(a, 0);
        assert_eq!(b, 500);
        assert_eq!(wid, 499);
        assert_eq!((recs[0].x, recs[0].y), (5, 6));
        assert_eq!(recs[0].damage, 40);
        assert_eq!(recs[0].kind, CRACK_TILE);
        assert_eq!(recs[1].kind, CRACK_WALL);
        assert_eq!(recs[1].damage, 60);
        assert_eq!((recs[2].x, recs[2].y), (7, 6));
        assert_eq!(recs[2].damage, 85);
        // Every style is distinct from its predecessor (the no-repeat
        // chain) and stable across frames of the same occupancy. The
        // rolls fire in index order (0, 499, 500), not hit order.
        let s0 = recs[0].crack_style;
        let s1 = recs[1].crack_style;
        let s2 = recs[2].crack_style;
        assert_ne!(s0, s1);
        assert_ne!(s1, s2);
        book.add_damage(a, 45, true); // 85 now — the ladder climbs
        let recs2 = styles.records(&book, &mut rng);
        assert_eq!(recs2[0].crack_style, s0, "the style holds through AddDamage");
        assert_eq!(recs2[0].damage, 85);
        assert_eq!(recs2[1].crack_style, s1);
        assert_eq!(recs2[2].crack_style, s2);
        // UpdatePosition (the mannequin redirect, `HitTile.cs:181-187`)
        // moves the cell WITHOUT a Clear — the style holds.
        book.update_position(a, 9, 6);
        let recs3 = styles.records(&book, &mut rng);
        assert_eq!(recs3[0].crack_style, s0);
        assert_eq!((recs3[0].x, recs3[0].y), (9, 6));
    }

    /// W4-g77 — the TTL decay keeps drawing: an abandoned hit's damage
    /// drains through the ladder (`HitTile.cs:297-313`) and the record
    /// still draws every stage until the <20 floor, then vanishes when
    /// the slot clears — the multi-tile decay the single-accumulator
    /// projection could never show.
    #[test]
    fn w4_g77_the_abandoned_hit_decays_through_the_crack_stages() {
        let mut book = HitTile::new();
        let mut styles = CrackStyles::new();
        let mut rng = UnifiedRandom::new(9);
        let id = book.hit_object(4, 4, CRACK_TILE);
        book.add_damage(id, 95, true);
        let mut seen_rows = Vec::new();
        for _ in 0..60 {
            let recs = styles.records(&book, &mut rng);
            if let Some(r) = recs.first() {
                if r.damage >= 20 {
                    seen_rows.push(crack_row(r.damage));
                }
                assert_eq!(r.crack_style, styles.styles[id], "one style for the decay");
            }
            book.prune(|_, _| true, |_, _| 0);
        }
        // 95 damage: rows 3 (95..80), 2 (78..60), 1 (58..40), 0 (38..20)
        // as the 2/5/7/10 ladder drains it — all four stages drew.
        assert!(seen_rows.contains(&3));
        assert!(seen_rows.contains(&2));
        assert!(seen_rows.contains(&1));
        assert!(seen_rows.contains(&0));
        // The slot cleared: no records left.
        assert!(styles.records(&book, &mut rng).is_empty());
        // A re-Prepare rolls a FRESH style (the C#'s Clear roll,
        // `HitTile.cs:40-45`) — never the immediately-previous roll (the
        // no-repeat loop), and the old slot's memory reset to -1.
        assert_eq!(styles.styles[id], -1, "the cleared slot forgot its style");
        let last = styles.last_crack;
        let id2 = book.hit_object(4, 4, CRACK_TILE);
        book.add_damage(id2, 50, true);
        let recs = styles.records(&book, &mut rng);
        assert_eq!(recs[0].damage, 50);
        assert!((0..4).contains(&recs[0].crack_style));
        assert_ne!(recs[0].crack_style, last, "the reuse rerolls");
    }

    /// The per-pass draw decision — every `continue` arm of
    /// `Main.DrawTileCracks` (`Main.cs:61541-61622`) in order, plus the
    /// emitted source rect for the drawing arm.
    #[test]
    fn s1_g_1c_the_draw_pass_gates_match_the_csharp_continues() {
        let mut w = world_with(1, 3, 3); // stone, active
        // The drawing arm: damage 80+, style 2 → row 3, column 36.
        assert_eq!(
            crack_draw_plan(&w, &rec(3, 3, 85, CRACK_TILE, 2), CRACK_TILE, false),
            Some([36.0, 54.0, 16.0, 16.0])
        );
        // The wall book draws on the same cell only via crack type 2 when
        // a wall exists (`:61563-61566`).
        assert_eq!(
            crack_draw_plan(&w, &rec(3, 3, 85, CRACK_WALL, 2), CRACK_WALL, false),
            None,
            "no wall in the cell"
        );
        w.tile_mut(3, 3).unwrap().wall = 4;
        assert_eq!(
            crack_draw_plan(&w, &rec(3, 3, 85, CRACK_WALL, 2), CRACK_WALL, false),
            Some([36.0, 54.0, 16.0, 16.0])
        );
        // `:61543`: the book-type filter.
        assert_eq!(
            crack_draw_plan(&w, &rec(3, 3, 85, CRACK_WALL, 2), CRACK_TILE, false),
            None
        );
        // `:61548`: the <20 damage floor (19 skips, 20 draws row 0).
        assert_eq!(
            crack_draw_plan(&w, &rec(3, 3, 19, CRACK_TILE, 0), CRACK_TILE, false),
            None
        );
        assert_eq!(
            crack_draw_plan(&w, &rec(3, 3, 20, CRACK_TILE, 0), CRACK_TILE, false),
            Some([0.0, 0.0, 16.0, 16.0])
        );
        // `:61554` InWorld (fluff 0): every out-of-bounds cell skips.
        for (x, y) in [(-1, 3), (40, 3), (3, -1), (3, 40)] {
            assert_eq!(
                crack_draw_plan(&w, &rec(x, y, 85, CRACK_TILE, 0), CRACK_TILE, false),
                None
            );
        }
        // `:61561`: an inactive block skips the tile arm.
        w.tile_mut(3, 3).unwrap().flags &= !FLAG_ACTIVE;
        assert_eq!(
            crack_draw_plan(&w, &rec(3, 3, 85, CRACK_TILE, 0), CRACK_TILE, false),
            None
        );
        // …but the wall arm still draws (the wall survives).
        assert_eq!(
            crack_draw_plan(&w, &rec(3, 3, 85, CRACK_WALL, 0), CRACK_WALL, false),
            Some([0.0, 54.0, 16.0, 16.0])
        );
        w.tile_mut(3, 3).unwrap().flags |= FLAG_ACTIVE;
        // `:61599`: slope 1 skips…
        w.tile_mut(3, 3).unwrap().meta = 1;
        assert_eq!(
            crack_draw_plan(&w, &rec(3, 3, 85, CRACK_TILE, 0), CRACK_TILE, false),
            None
        );
        // …and so does a half-brick.
        w.tile_mut(3, 3).unwrap().meta = 0;
        w.tile_mut(3, 3).unwrap().flags |= FLAG_HALF_BRICK;
        assert_eq!(
            crack_draw_plan(&w, &rec(3, 3, 85, CRACK_TILE, 0), CRACK_TILE, false),
            None
        );
        w.tile_mut(3, 3).unwrap().flags &= !FLAG_HALF_BRICK;
        // `:61573`: the door never cracks; `:61577`: solid-top never cracks.
        w.tile_mut(3, 3).unwrap().kind = 10;
        assert_eq!(
            crack_draw_plan(&w, &rec(3, 3, 85, CRACK_TILE, 0), CRACK_TILE, false),
            None
        );
        w.tile_mut(3, 3).unwrap().kind = 19; // platform
        assert_eq!(
            crack_draw_plan(&w, &rec(3, 3, 85, CRACK_TILE, 0), CRACK_TILE, false),
            None
        );
        // The tree arm shifts the column (`:61619-61622`): trunk 5, style 3
        // → column (4 + 3/2) * 18 = 90.
        w.tile_mut(3, 3).unwrap().kind = 5;
        assert_eq!(
            crack_draw_plan(&w, &rec(3, 3, 85, CRACK_TILE, 3), CRACK_TILE, false),
            Some([90.0, 54.0, 16.0, 16.0])
        );
    }

    /// W4-g77 — the echo-coating gates (`Main.cs:61561`/`:61565`): an
    /// invisible block/wall skips its crack unless
    /// `ShouldShowInvisibleBlocksAndWalls` (the `TR_SHOW_INVISIBLE` pin
    /// stand-in) is on. The bits are real now (`Tile.cs:528`/`:545`).
    #[test]
    fn w4_g77_the_echo_coating_gates_the_crack() {
        let mut w = world_with(1, 3, 3);
        w.tile_mut(3, 3).unwrap().wall = 4;
        w.set_invisible_block(3, 3, true);
        assert_eq!(
            crack_draw_plan(&w, &rec(3, 3, 85, CRACK_TILE, 0), CRACK_TILE, false),
            None,
            "the echo block hides its crack"
        );
        assert_eq!(
            crack_draw_plan(&w, &rec(3, 3, 85, CRACK_TILE, 0), CRACK_TILE, true),
            Some([0.0, 54.0, 16.0, 16.0]),
            "show-invisible reveals it"
        );
        // The wall gate is independent (`:61565`).
        assert_eq!(
            crack_draw_plan(&w, &rec(3, 3, 85, CRACK_WALL, 0), CRACK_WALL, false),
            Some([0.0, 54.0, 16.0, 16.0]),
            "the wall is not echo-coated"
        );
        w.set_invisible_wall(3, 3, true);
        assert_eq!(
            crack_draw_plan(&w, &rec(3, 3, 85, CRACK_WALL, 0), CRACK_WALL, false),
            None
        );
        assert_eq!(
            crack_draw_plan(&w, &rec(3, 3, 85, CRACK_WALL, 0), CRACK_WALL, true),
            Some([0.0, 54.0, 16.0, 16.0])
        );
    }

    /// The hook's book walk (`TileDrawing.cs:732-733`): `hitReplace` first
    /// (empty in the port), then `hitTile` — one dig's hits climb the
    /// damage ladder through the book without rerolling the style, and
    /// the determinism holds for the world-seeded stream.
    #[test]
    fn s1_g_1c_the_hook_walks_one_digs_damage_ladder() {
        let w = world_with(1, 5, 6);
        let mut book = HitTile::new();
        let mut styles = CrackStyles::new();
        let mut rng = UnifiedRandom::new(11);
        let id = book.hit_object(5, 6, CRACK_TILE);
        // Hit 1: damage 10, below the floor → the plan skips.
        book.add_damage(id, 10, true);
        let r = styles.records(&book, &mut rng)[0];
        assert_eq!(crack_draw_plan(&w, &r, CRACK_TILE, false), None);
        let prev_style = r.crack_style;
        // Later hits climb the ladder without rerolling the style
        // (25 → row 0, 50 → row 1, 70 → row 2, 90 → row 3).
        for (add, row) in [(15, 0.0), (25, 18.0), (20, 36.0), (20, 54.0)] {
            book.add_damage(id, add, true);
            let r = styles.records(&book, &mut rng)[0];
            assert_eq!(r.crack_style, prev_style);
            let plan = crack_draw_plan(&w, &r, CRACK_TILE, false).expect("draws");
            assert_eq!(plan[1], row, "the damage ladder's row at {}", r.damage);
            assert_eq!(plan[0], (r.crack_style * 18) as f32);
        }
        // Determinism: the same seed + the same hit sequence roll the
        // same styles (the world-seeded deviation).
        let mut book2 = HitTile::new();
        let mut styles2 = CrackStyles::new();
        let mut rng2 = UnifiedRandom::new(11);
        let id2 = book2.hit_object(5, 6, CRACK_TILE);
        book2.add_damage(id2, 10, true);
        assert_eq!(
            styles2.records(&book2, &mut rng2)[0].crack_style,
            prev_style
        );
    }
}
