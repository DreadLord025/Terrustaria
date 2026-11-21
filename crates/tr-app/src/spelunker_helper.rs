//! W4-g78 DOC-SWEEP lane 4 — `Terraria.GameContent.SpelunkerProjectileHelper`
//! (`SpelunkerProjectileHelper.cs:6-67`), the Spelunker Glowstick (473) /
//! Spelunker Flare family's ore-spot dust helper. The 473 aiStyle-14 head
//! (`Projectile.cs:24836-24847`) keeps its 10-tick `localAI[0]` counter in
//! `projectile_ai_light_items.rs::glowstick_head` but documents the
//! `AddSpotToCheck` call as a skip ("no ore-highlight layer"); this module
//! ports the helper itself so the wiring lane has the real seam.
//!
//! **PORTED, ATTACH-CONTRACT PUBLISHED (code-only, W4-g78 wiring phase).**
//! The live dispatch seams (`projectile.rs`'s aiStyle-14/33 arms, the
//! `ProjectilePool` struct, `world_sim/projectiles.rs`'s pass head/tail) are
//! contested by the d9/d13/d16/g4/f5/g11/g16/g63/g64/g65 lanes — the exact
//! attach hunks (C# positions + the DropRng-stream verdict) are published in
//! **research/77 §1** for the c42d/95-union merge. Intended call sites:
//! * `Main.PreUpdateAllProjectiles` (`Main.cs:17535-17539`, called from
//!   `UpdateWorld_Projectiles` `:18001` BEFORE the 1000-slot walk) →
//!   [`SpelunkerProjectileHelper::on_pre_update_all_projectiles`] once per
//!   projectile-update pass (the 10-tick dedupe-clear cadence);
//! * the 473 head's counter wrap (`Projectile.cs:24841-24845`) AND the 1008
//!   Spelunker Flare's STUCK arm (`:27124-27127` — every stuck tick, NO
//!   counter) → [`SpelunkerProjectileHelper::add_spot_to_check`] with
//!   `base.Center`, gated on [`SpelunkerProjectileHelper::spot_in_range`] —
//!   the distance gate stays with the caller (the C# keeps it outside the
//!   helper);
//! * the HELD Spelunker Glowstick 3002 (`Player.cs:50094-50099`, the
//!   `spelunkerTimer` 10-tick counter inside `ItemCheck_EmitHeldItemLight`,
//!   NO distance gate, `!pulley`-gated) — the THIRD C# caller research/76
//!   missed; contract in research/77 §1.3.
//! The helper's `Main.rand` is the caller's stage stream (the `DropRng`
//! seam, the d60 particles convention): `UpdateProjectiles` for the two
//! projectile callers, `UpdatePlayers` for the held 3002. The dust spawns
//! are returned as [`SpelunkerDust`] requests the caller drains into
//! `app.dust` via `DustPool::new_dust_direct` (16×16 box, speed 0/0, alpha
//! 150, scale 0.3) + the fadeIn 0.75 / velocity ×0.1 / noLight post-arms;
//! this module never spawns.
//!
//! RNG census: ONE `Main.rand.Next(4)` per NEWLY-checked spelunkable tile
//! (`:54` — `_tilesChecked.Add(item) && Main.rand.Next(4) == 0`, the `&&`
//! short-circuits: a tile already in the set rolls NOTHING). The dedupe sets
//! clear every 10th `OnPreUpdateAllProjectiles` call (`:19-24`), so a parked
//! glowstick re-rolls its whole window every 10 ticks.
//!
//! The `Main.IsTileSpelunkable(Tile)` overload (`Main.cs:21778-21781`) reads
//! `t.type/frameX/frameY` — the same predicate d9 landed canonically in
//! `projectile_ai_style90::is_tile_spelunkable` (the `TileSet` bitset) and
//! d56 in `tr_render::animated_tiles`; this module now carries the d9
//! bitset VERBATIM (the keep-one intermediate — at the merge this copy
//! deletes and re-exports d9's, research/77 §4) and the census witness pins
//! every row against `Main.cs:9420-9467` + `:9651`.

use std::collections::HashSet;

use glam::Vec2;
use tr_core::tiledata::TileSet;
use tr_core::World;

/// `Main.tileSpelunker[type]` (`Main.cs:9420-9467` + `:9651`): the 49 true
/// rows as a bitset. **Keep-one (d9 canonical):** these words are VERBATIM
/// d9's `projectile_ai_style90::TILE_SPELUNKER` (`TileSet::from_words`, the
/// `SWAYS_IN_WIND_BASIC` hand-transcription precedent) — at the wiring merge
/// this copy and d9's collapse to one (delete this, re-export d9's; the
/// d13 `projectile_ai_pets124` inline `[u16]` table and d56's tr-render copy
/// fold onto the same bitset, research/77 §4). The id list is kept below for
/// the census witness only.
static TILE_SPELUNKER: TileSet = TileSet::from_words(&[
    0x8000_0020_1020_13C0, // 6,7,8,9,12,21,28,37,63
    0x0000_9A00_0018_001F, // 64,65,66,67,68,83,84,105,107,108,111
    0x0004_03C0_0000_0000, // 166,167,168,169,178
    0x0065_1008_E008_0000, // 211,221,222,223,227,236,240,242,245,246
    0x0000_0000_0000_0000,
    0x0000_0000_2002_0000, // 337,349
    0x0200_0000_0090_0000, // 404,407,441
    0x0400_0000_0018_0000, // 467,468,506
    0x0040_0000_0008_0000, // 531,566
    0x8000_0000_0000_0000, // 639
    0x4000_0000_0000_0000, // 702
    0x0001_8000_0000_0000, // 751,752
]);

/// The 49 `tileSpelunker` ids in source order — the census witness's list
/// (the live lookup is the [`TILE_SPELUNKER`] bitset above).
const TILE_SPELUNKER_IDS: [u16; 49] = [
    6, 7, 8, 9, 12, 21, 467, 441, 468, 28, 107, 108, 111, 63, 64, 65, 66, 67, 68, 566, 166, 167,
    168, 169, 178, 211, 221, 222, 223, 236, 702, 37, 407, 227, 84, 83, 404, 242, 240, 246, 245,
    105, 349, 337, 531, 506, 751, 752, 639,
];

/// `Main.IsTileSpelunkable(ushort, short, short)` (`Main.cs:21783-21798`):
/// the `tileSpelunker` bitset plus the two 185/186 frame windows (the
/// grave-marker and statue treasure rows). Keep-one: d9's
/// `projectile_ai_style90::is_tile_spelunkable` is the canonical copy (the
/// same bitset + windows); d56's tr-render copy and d13's inline table fold
/// onto it at the wiring merges (research/77 §4).
pub fn is_tile_spelunkable(type_cache: u16, frame_x: i16, frame_y: i16) -> bool {
    if TILE_SPELUNKER.has(type_cache) {
        return true;
    }
    if type_cache == 185 && frame_y == 18 && (576..=882).contains(&frame_x) {
        return true;
    }
    if type_cache == 186 && (864..=1170).contains(&frame_x) {
        return true;
    }
    false
}

/// One `Dust.NewDustDirect` request the spot check emits
/// (`SpelunkerProjectileHelper.cs:58-61`): type 204, 16x16 at the tile's
/// top-left, alpha 150, scale 0.3, then `fadeIn = 0.75f`,
/// `velocity *= 0.1f`, `noLight = true`. `NewDustDirect` wraps `NewDust`
/// (`Dust.cs:65-73`) — the SAME spawn rolls; the NaN-velocity zeroing is
/// dust-layer business the caller owns.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpelunkerDust {
    /// The tile's top-left in world pixels (`i * 16, j * 16`).
    pub x: f32,
    pub y: f32,
    /// `Dust.NewDust` type — always 204 here.
    pub dust_type: i32,
    /// `Alpha` — 150.
    pub alpha: i32,
    /// `Scale` — 0.3f.
    pub scale: f32,
    /// `fadeIn = 0.75f` (NOT the vision-floor arms' 1f).
    pub fade_in: f32,
    /// `velocity *= 0.1f`.
    pub velocity_scale: f32,
    /// `noLight = true`.
    pub no_light: bool,
}

/// `SpelunkerProjectileHelper` (`SpelunkerProjectileHelper.cs:6-67`): the
/// per-position / per-tile dedupe sets + the 10-tick clear cadence. The
/// `_clampBox` is rebuilt every pre-update pass from the world size.
#[derive(Default)]
pub struct SpelunkerProjectileHelper {
    /// `_positionsChecked` — `HashSet<Vector2>`: exact f32 bit equality
    /// (the C# hashset keys on the Vector2's bits; the port keys on the
    /// two `to_bits` words).
    positions_checked: HashSet<(u32, u32)>,
    /// `_tilesChecked` — `HashSet<Point>`.
    tiles_checked: HashSet<(i32, i32)>,
    /// `_frameCounter` — the 10-tick clear cadence.
    frame_counter: i32,
}

impl SpelunkerProjectileHelper {
    /// `OnPreUpdateAllProjectiles` (`:16-25`): the clamp box is
    /// `Rectangle(2, 2, maxTilesX - 2, maxTilesY - 2)` — Left/Top 2,
    /// Right `2 + (maxTilesX - 2) = maxTilesX`, Bottom `maxTilesY` (the
    /// C# `Rectangle.Right` is x+width); every 10th call clears BOTH sets.
    /// The box itself is derived per-check from the world size (the port
    /// carries no cached field — same value, same order).
    pub fn on_pre_update_all_projectiles(&mut self) {
        self.frame_counter += 1;
        if self.frame_counter >= 10 {
            self.frame_counter = 0;
            self.tiles_checked.clear();
            self.positions_checked.clear();
        }
    }

    /// `AddSpotToCheck` (`:27-33`): a NEW position runs `CheckSpot`; a
    /// repeat position is a no-op (no scan, no rolls). `rng` is the
    /// caller's projectile-stage `Main.rand` (the `DropRng` seam — the
    /// 473 head already carries one).
    pub fn add_spot_to_check(
        &mut self,
        world: &World,
        spot: Vec2,
        rng: &mut dyn tr_core::loot::DropRng,
        out: &mut Vec<SpelunkerDust>,
    ) {
        if self.positions_checked.insert((spot.x.to_bits(), spot.y.to_bits())) {
            self.check_spot(world, spot, rng, out);
        }
    }

    /// The CALLER-side distance gate the 473 glowstick head
    /// (`Projectile.cs:24842`) and the 1008 flare's stuck arm (`:27124`)
    /// both apply BEFORE `AddSpotToCheck`:
    /// `(Center - Main.player[myPlayer].Center).Length() <
    /// (float)(screenWidth + 30 * 16)`. The C# keeps it OUTSIDE the helper
    /// (the helper itself has no player read), so the port keeps it here as
    /// a free function the wiring lane calls at the dispatch — the
    /// `SpelunkerProjectileHelper` never sees the player. `screen_width` is
    /// `Main.screenWidth` (the swapchain pixel width, the d13
    /// `set_screen_width` seam); `player_center` is `None` when no local
    /// player exists (the gate fails closed — no scan).
    pub fn spot_in_range(spot: Vec2, player_center: Option<Vec2>, screen_width: f32) -> bool {
        let Some(pc) = player_center else {
            return false;
        };
        (spot - pc).length() < screen_width + 30.0 * 16.0
    }

    /// `CheckSpot` (`:35-66`): the ±30-tile window around the spot's tile
    /// (clamped to the box), x outer / y inner; an active spelunkable tile
    /// within 30 tiles (`!(Length() > 30f)`) that is NEW to `_tilesChecked`
    /// rolls `Next(4)` — a 0 spawns the 204 dust request.
    fn check_spot(
        &mut self,
        world: &World,
        center: Vec2,
        rng: &mut dyn tr_core::loot::DropRng,
        out: &mut Vec<SpelunkerDust>,
    ) {
        let num = center.x as i32 / 16;
        let num2 = center.y as i32 / 16;
        // `Utils.Clamp(num ± 30, _clampBox.Left/Top, Right/Bottom)` — the
        // box's Right/Bottom are maxTilesX/maxTilesY (module doc). An
        // edge-clamped index past the last tile reads None in the port
        // (the C# `tile != null` guard's stand-in).
        let num3 = (num - 30).clamp(2, world.width);
        let num4 = (num + 30).clamp(2, world.width);
        let num5 = (num2 - 30).clamp(2, world.height);
        let num6 = (num2 + 30).clamp(2, world.height);
        for i in num3..=num4 {
            for j in num5..=num6 {
                let Some(tile) = world.tile(i, j) else {
                    continue;
                };
                if !tile.active()
                    || !is_tile_spelunkable(tile.kind, tile.frame_x, tile.frame_y)
                {
                    continue;
                }
                let dx = (num - i) as f32;
                let dy = (num2 - j) as f32;
                if (dx * dx + dy * dy).sqrt() > 30.0 {
                    continue;
                }
                // `:54` — the Add && Next(4) short-circuit: only a NEW tile
                // rolls; a repeat tile draws nothing.
                if self.tiles_checked.insert((i, j)) && rng.next_range(0, 4) == 0 {
                    out.push(SpelunkerDust {
                        x: (i * 16) as f32,
                        y: (j * 16) as f32,
                        dust_type: 204,
                        alpha: 150,
                        scale: 0.3,
                        fade_in: 0.75,
                        velocity_scale: 0.1,
                        no_light: true,
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::tile::FLAG_ACTIVE;

    /// A scripted `DropRng` (the light-items `Recorder` precedent): the
    /// `Next(4)` gate draws are handed back in order, so the witness pins
    /// the exact roll count + the spawn-on-0 arm without a seed hunt.
    struct Scripted {
        draws: Vec<i32>,
        pos: usize,
    }
    impl tr_core::loot::DropRng for Scripted {
        fn next_range(&mut self, min: i32, max: i32) -> i32 {
            assert_eq!((min, max), (0, 4), "the helper rolls only Next(4)");
            let d = self.draws[self.pos];
            self.pos += 1;
            d
        }
        fn next_float(&mut self) -> f32 {
            unreachable!("the helper draws no floats")
        }
    }

    fn world_with_ore() -> World {
        let mut w = World::new("t".into(), 80, 80);
        // A copper ore (6) at (40, 40), a chest (21) at (41, 40), plain
        // dirt (1) at (42, 40) — one spelunkable pair + one refusal.
        for (x, kind) in [(40, 6u16), (41, 21), (42, 1)] {
            let i = w.index(x, 40);
            w.tiles[i].kind = kind;
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        w
    }

    /// `Main.cs:9420-9467` + `:9651`: every listed id is spelunkable; the
    /// unlisted dirt/stone are not. The 49-row census pins the id list AND
    /// that the d9-canonical bitset agrees with it exactly (the keep-one
    /// dedupe's equivalence proof, research/77 §4).
    #[test]
    fn w4_g78_the_spelunker_table_census() {
        assert_eq!(TILE_SPELUNKER_IDS.len(), 49);
        for k in TILE_SPELUNKER_IDS {
            assert!(is_tile_spelunkable(k, 0, 0), "spelunker {k}");
            assert!(TILE_SPELUNKER.has(k), "the bitset carries {k}");
        }
        // The bitset holds EXACTLY the 49 ids (no stray bits): every id in
        // 0..TILE_COUNT is in the set iff it is in the census list.
        for k in 0..tr_core::tiledata::TILE_COUNT as u16 {
            assert_eq!(
                TILE_SPELUNKER.has(k),
                TILE_SPELUNKER_IDS.contains(&k),
                "bitset vs census at {k}"
            );
        }
        assert!(!is_tile_spelunkable(0, 0, 0));
        assert!(!is_tile_spelunkable(1, 0, 0));
        assert!(!is_tile_spelunkable(185, 0, 0));
        assert!(!is_tile_spelunkable(186, 0, 0));
    }

    /// `Projectile.cs:24842` / `:27124` — the caller-side distance gate:
    /// strictly `< screenWidth + 480`; exactly-at-the-boundary fails; no
    /// player fails closed. screenWidth 1280 → the boundary is 1760 px.
    #[test]
    fn w4_g78_the_caller_distance_gate_is_strict() {
        let pc = Vec2::new(1000.0, 1000.0);
        // 1759 px east: inside.
        assert!(SpelunkerProjectileHelper::spot_in_range(
            Vec2::new(2759.0, 1000.0),
            Some(pc),
            1280.0
        ));
        // Exactly 1760: the C# `<` refuses.
        assert!(!SpelunkerProjectileHelper::spot_in_range(
            Vec2::new(2760.0, 1000.0),
            Some(pc),
            1280.0
        ));
        // Euclidean, not per-axis: (1200 east, 1300 south) ≈ 1769.2 > 1760.
        assert!(!SpelunkerProjectileHelper::spot_in_range(
            Vec2::new(2200.0, 2300.0),
            Some(pc),
            1280.0
        ));
        // No player: no scan.
        assert!(!SpelunkerProjectileHelper::spot_in_range(pc, None, 1280.0));
    }

    /// `Main.cs:21789-21796`: the 185 window needs frameY 18 AND frameX
    /// 576..=882 (inclusive); the 186 window is frameX 864..=1170 at ANY
    /// frameY.
    #[test]
    fn w4_g78_the_spelunker_frame_windows_are_inclusive() {
        assert!(is_tile_spelunkable(185, 576, 18));
        assert!(is_tile_spelunkable(185, 882, 18));
        assert!(!is_tile_spelunkable(185, 575, 18));
        assert!(!is_tile_spelunkable(185, 883, 18));
        assert!(!is_tile_spelunkable(185, 600, 0));
        assert!(is_tile_spelunkable(186, 864, 999));
        assert!(is_tile_spelunkable(186, 1170, 0));
        assert!(!is_tile_spelunkable(186, 863, 0));
        assert!(!is_tile_spelunkable(186, 1171, 0));
    }

    /// `:27-33` + `:54`: a new spot scans; the SAME spot again is a no-op
    /// (no second scan, no rolls). Two 0 draws spawn one dust per new
    /// spelunkable tile (x-then-y order); the dirt never rolls.
    #[test]
    fn w4_g78_the_spot_dedupe_and_the_next4_gate() {
        let w = world_with_ore();
        let mut h = SpelunkerProjectileHelper::default();
        let spot = Vec2::new(40.0 * 16.0 + 8.0, 40.0 * 16.0 + 8.0);
        let mut rng = Scripted { draws: vec![0, 0], pos: 0 };
        let mut out = Vec::new();
        h.add_spot_to_check(&w, spot, &mut rng, &mut out);
        assert_eq!(rng.pos, 2, "one Next(4) per new spelunkable tile");
        assert_eq!(out.len(), 2, "the ore + the chest each spawn once");
        assert_eq!(out[0].x, 40.0 * 16.0);
        assert_eq!(out[1].x, 41.0 * 16.0);
        assert_eq!(out[0].dust_type, 204);
        assert_eq!(out[0].alpha, 150);
        assert_eq!(out[0].scale, 0.3);
        assert_eq!(out[0].fade_in, 0.75);
        assert_eq!(out[0].velocity_scale, 0.1);
        assert!(out[0].no_light);
        // The same spot: deduped, no new requests, no rolls.
        h.add_spot_to_check(&w, spot, &mut rng, &mut out);
        assert_eq!(out.len(), 2, "the repeat spot scans nothing");
        assert_eq!(rng.pos, 2, "the repeat spot rolls nothing");
    }

    /// `:54` — a non-zero Next(4) spawns nothing but STILL marks the tile
    /// checked (the Add runs before the roll): a later re-scan of the same
    /// tile (after the 10-tick clear) rolls again.
    #[test]
    fn w4_g78_a_nonzero_roll_marks_the_tile_checked() {
        let w = world_with_ore();
        let mut h = SpelunkerProjectileHelper::default();
        let spot = Vec2::new(648.0, 648.0);
        let mut rng = Scripted { draws: vec![3, 3], pos: 0 };
        let mut out = Vec::new();
        h.add_spot_to_check(&w, spot, &mut rng, &mut out);
        assert_eq!(rng.pos, 2);
        assert!(out.is_empty(), "both rolls non-zero: no dust");
        // Same spot again: deduped (no rolls), even though nothing spawned.
        h.add_spot_to_check(&w, spot, &mut rng, &mut out);
        assert_eq!(rng.pos, 2, "the tiles stay checked after a miss");
    }

    /// `:19-24`: the 10th pre-update pass clears both sets — the same spot
    /// then re-scans (and the tiles roll again).
    #[test]
    fn w4_g78_the_ten_tick_clear_reopens_the_dedupe() {
        let w = world_with_ore();
        let mut h = SpelunkerProjectileHelper::default();
        let spot = Vec2::new(648.0, 648.0);
        let mut rng = Scripted { draws: vec![0, 0, 0, 0], pos: 0 };
        let mut out = Vec::new();
        h.add_spot_to_check(&w, spot, &mut rng, &mut out);
        assert_eq!(out.len(), 2);
        for _ in 0..9 {
            h.on_pre_update_all_projectiles();
        }
        // 9 passes: counter 9, no clear — the spot stays deduped.
        h.add_spot_to_check(&w, spot, &mut rng, &mut out);
        assert_eq!(out.len(), 2, "no clear yet");
        assert_eq!(rng.pos, 2);
        // The 10th pass clears; the spot re-scans and re-rolls.
        h.on_pre_update_all_projectiles();
        h.add_spot_to_check(&w, spot, &mut rng, &mut out);
        assert_eq!(out.len(), 4, "the clear reopens the spot");
        assert_eq!(rng.pos, 4);
        assert_eq!(h.frame_counter, 0, "the counter wrapped");
    }

    /// `:50`: the 30-tile radius gate — a spelunkable tile INSIDE the ±30
    /// window but at Euclidean distance > 30 (dx 30, dy 1 → √901 ≈ 30.017)
    /// is skipped (no roll, no dust); the exactly-30 tile passes.
    #[test]
    fn w4_g78_the_thirty_tile_radius_gate() {
        let mut w = World::new("t".into(), 100, 100);
        // Ore at exactly 30 tiles east of the spot's tile, and one at
        // (30 east, 1 south) — inside the window, outside the radius.
        for (x, y) in [(70, 50), (70, 51)] {
            let i = w.index(x, y);
            w.tiles[i].kind = 6;
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        let mut h = SpelunkerProjectileHelper::default();
        let spot = Vec2::new(40.0 * 16.0 + 8.0, 50.0 * 16.0 + 8.0);
        let mut rng = Scripted { draws: vec![0], pos: 0 };
        let mut out = Vec::new();
        h.add_spot_to_check(&w, spot, &mut rng, &mut out);
        assert_eq!(rng.pos, 1, "only the in-radius ore rolls");
        assert_eq!(out.len(), 1, "only the 30-tile ore passes");
        assert_eq!(out[0].x, 70.0 * 16.0);
        assert_eq!(out[0].y, 50.0 * 16.0);
    }
}
