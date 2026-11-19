//! S1-F-9 slice B — the SPECIAL-TILE `GetTileDrawData` cases whose C# bodies
//! need engine state the static table (`tr_core::drawdata::draw_data`) cannot
//! see: liquids, biome influence, wind, the local player, the party, pressed
//! plates, the invisible-block setting and the global animation clock.
//!
//! The static layer in `drawdata.rs` already carries every case's
//! unconditional part; this module completes the engine-driven remainder for
//! 16 case ids (disjoint from slice A `drawdata_cases_a.rs` — the merge
//! dedupe: A owns 518/529/530's first-switch world probes and 441/468's
//! second-switch glow rows, so B skips those five ids — and from wave B's
//! animator `drawdata_anim.rs` and wave C's cages/chests
//! `drawdata_furniture.rs`):
//!
//! - `719` RainbowCloud      — `Main.GlobalTimeWrappedHourly` (TileDrawing.cs:4562-4570)
//! - `579` RockGolemHead     — `Main.LocalPlayer.Center.X` (TileDrawing.cs:5104-5130)
//! - `453` SillyBalloonTile / `456` PartyBundleOfBalloonTile —
//!   `Main.tileFrameCounter` hard cycles (TileDrawing.cs:5412-5417)
//! - `507|508` StarryGlassBlocks — `Main.tileFrameCounter` (TileDrawing.cs:6007-6013)
//! - `541` EchoBlock         — `_shouldShowInvisibleBlocks` (TileDrawing.cs:6004-6006)
//! - `485` AntlionLarva      — `Main.tileFrameCounter` (TileDrawing.cs:4727-4730)
//! - `489` PinWheel          — `InAPlaceWithWind` + counter (TileDrawing.cs:4731-4741)
//! - `490` WeatherVane       — wind + `Main.weatherVaneBobframe` (TileDrawing.cs:4742-4818)
//! - `493` GolfCupFlag       — wind tiers + counter (TileDrawing.cs:5231-5257)
//! - `428` WeightedPressurePlate — `PressurePlateHelper.PressurePlatesPressed`
//!   (TileDrawing.cs:5605-5611)
//! - `455` PartyMonolith     — `BirthdayParty.PartyIsUp` (TileDrawing.cs:5397-5408)
//! - `349` MushroomStatue, `593|594` Volcanoes —
//!   the `Animation.GetTemporaryFrame` probe (TileDrawing.cs:5332-5341,
//!   5445-5487; Animation.cs:186-196).
//!
//! Seam: [`case_draw_data`] takes the static table's output as `base` and
//! returns `Some` only for the kinds above — the renderer
//! (`render/tiles.rs`) applies it right after `draw_data` +
//! `apply_world_corrections`. Every rewrite is the C# case body verbatim;
//! where the static arm already applied an idempotent absolute assignment
//! (`tileTop = 2`, the 529 sizes, the 593/594 fallbacks) the body re-states it
//! exactly once, and additive C# parts (`addFrY += ...`) add onto the same
//! `tile_frame * 38` starting value the static table carries.
//!
//! Deliberate deviations (docs/ANALYSIS.md "S1-F-9e slice B"):
//! - `tile_frame` / `counter` (`Main.tileFrame[kind]` /
//!   `Main.tileFrameCounter[kind]`) ride in as function parameters and
//!   `weather_vane_bobframe` as an input; WIRED since S1-G-1f — the
//!   production caller (`render/tiles.rs::draw_tiles`) feeds the
//!   `TileAnimator` state (`drawdata_anim.rs`), ticked by
//!   `App::animate_tiles_phase` (`Main.cs:17150`).
//! - `Animation.GetTemporaryFrame` reads `Animation._temporaryAnimations`
//!   (Animation.cs:186-196); the port has no temporary-animation registry
//!   yet, so [`temporary_frame`] returns `None` — the C# `false` path. The
//!   349/593/594 bodies are ported in full around that probe (441/468's
//!   first-switch half is static-only with the registry empty — slice A
//!   owns those ids' second-switch glow rows).
//! - `579` with no local player (`player_center_x = None`, menu frames)
//!   takes `flag2 = false`; in-game the caller always passes the player's
//!   center (`Main.LocalPlayer` always exists in C#).

use tr_core::drawdata::TileDrawData;
use tr_core::wind::Wind;
use tr_core::wiring::Wiring;
use tr_core::world::World;

// The 518/529/530 probes (`WorldGen.SolidTile`, `Tile.blockType`,
// `WorldGen.GetBiomeInfluence`, `WorldGen.beachDistance`) moved to slice A
// (`drawdata_cases_a.rs`) in the merge dedupe — A owns those ids.

/// The per-frame scalar engine inputs (everything that is not a world/wind/
/// wiring reference). `Default` is the neutral vanilla t=0 state.
#[derive(Clone, Copy, Debug, Default)]
pub struct CaseInputs {
    /// `Main.weatherVaneBobframe` (Main.cs:2237; fed from the wired
    /// `TileAnimator` since S1-G-1f).
    pub weather_vane_bobframe: i32,
    /// `Main.LocalPlayer.Center.X` (`player.x + width/2`); `None` = no
    /// local player (menu frames — the 579 deviation above).
    pub player_center_x: Option<f32>,
    /// `TileDrawing._shouldShowInvisibleBlocks`
    /// (`Main.ShouldShowInvisibleBlocksAndWalls`, TileDrawing.cs:379).
    pub show_invisible_blocks: bool,
    /// `BirthdayParty.PartyIsUp` (BirthdayParty.cs:25-32).
    pub party_is_up: bool,
    /// `Main.GlobalTimeWrappedHourly` — `TotalGameTime.TotalSeconds % 3600`
    /// (Main.cs:16836).
    pub global_time_wrapped_hourly: f32,
}

/// The full probe environment for one frame.
pub struct CaseEnv<'a> {
    pub world: &'a World,
    pub wind: &'a Wind,
    pub wiring: &'a Wiring,
    pub inputs: CaseInputs,
}

/// `Animation.GetTemporaryFrame` (Animation.cs:186-196): the temporary-
/// animation registry is not ported, so the lookup always takes the C#
/// `false` path (deviation above). When the registry lands, plug it in here.
fn temporary_frame(_x: i32, _y: i32) -> Option<i32> {
    None
}

/// The slice-B dispatch arm: `Some` for the 16 owned kinds (the C# case body
/// applied onto the static table's `base`), `None` for everything else.
///
/// `tile_frame` is `Main.tileFrame[kind]`, `counter` is
/// `Main.tileFrameCounter[kind]` (fed from the wired `TileAnimator` since
/// S1-G-1f; zero state = the vanilla t=0 draw).
pub fn case_draw_data(
    kind: u16,
    x: i32,
    y: i32,
    frame_x: i16,
    frame_y: i16,
    tile_frame: i32,
    counter: i32,
    base: TileDrawData,
    env: &CaseEnv<'_>,
) -> Option<TileDrawData> {
    let fx = frame_x as i32;
    let fy = frame_y as i32;
    let mut data = base;
    match kind {
        // `case 719` — RainbowCloud: the 14-step clock cycle walks a 4x4
        // block grid, 288 px columns / 270 px rows (TileDrawing.cs:4562-4570).
        // C# `addFrX/addFrY +=` on top of the `tileFrame * 38` default.
        719 => {
            let n24 = (x + y + (env.inputs.global_time_wrapped_hourly * 15.0) as i32) % 14;
            let n25 = n24 / 4;
            let n26 = n24 % 4;
            data.add_fr_x = (data.add_fr_x as i32 + 288 * n25) as i16;
            data.add_fr_y = (data.add_fr_y as i32 + 270 * n26) as i16;
        }
        // `case 579` — RockGolemHead: 20x20, raised 2 px; the eye row flips
        // to face the player (TileDrawing.cs:5104-5130).
        579 => {
            data.width = 20;
            data.height = 20;
            data.top -= 2;
            let flag2 = env
                .inputs
                .player_center_x
                .is_some_and(|cx| (x * 16 + 8) as f32 > cx);
            if fx > 0 {
                data.add_fr_y = if flag2 { 22 } else { 0 };
            } else {
                data.add_fr_y = if flag2 { 0 } else { 22 };
            }
        }
        // `case 453` — SillyBalloonTile: 3-row hard cycle off the counter
        // plus the cell's sheet-relative position (TileDrawing.cs:5412-5414).
        453 => {
            data.add_fr_y = ((counter / 20 + (y - fy / 18 + x)) % 3 * 54) as i16;
        }
        // `case 456` — PartyBundleOfBalloonTile: 4-row cycle, both axes
        // sheet-relative (TileDrawing.cs:5415-5417).
        456 => {
            data.add_fr_y = ((counter / 20 + (y - fy / 18 + (x - fx / 18))) % 4 * 54) as i16;
        }
        // `case 507 / 508` — StarryGlassBlocks: the 8-step twinkle band,
        // `num = 20` (TileDrawing.cs:6007-6013).
        507 | 508 => {
            let num = 20;
            data.add_fr_y = (90 * ((counter + x * 11 + y * 27) % (num * 8) / num)) as i16;
        }
        // `case 541` — EchoBlock: the hidden row unless invisible blocks
        // are shown (TileDrawing.cs:6004-6006).
        541 => {
            data.add_fr_y = if !env.inputs.show_invisible_blocks { 90 } else { 0 };
        }
        // `case 485` — AntlionLarva: top 2 (static arm), the 4-row cycle
        // (TileDrawing.cs:4727-4730).
        485 => {
            data.top = 2;
            data.add_fr_y = ((counter / 5 + (y - fy / 18 + (x - fx / 18))) % 4 * 36) as i16;
        }
        // `case 489` — PinWheel: top 2; spins only in a wind-blown place
        // (TileDrawing.cs:4731-4741). Outside wind the C# leaves addFrY at
        // the `tileFrame * 38` default — `base` already carries it.
        489 => {
            data.top = 2;
            let n32 = y - fy / 18;
            let n33 = x - fx / 18;
            if env.wind.in_a_place_with_wind(env.world, n33, n32, 2, 3) {
                data.add_fr_y = ((counter / 5 + (n32 + n33)) % 16 * 54) as i16;
            }
        }
        // `case 490` — WeatherVane: top 2; the bob table by wind strength
        // (TileDrawing.cs:4742-4818). `num12 = flag ? tileFrame : 0`, the
        // bob adds, negative wraps +12, `% 12 * 36`.
        490 => {
            data.top = 2;
            let flag = env
                .wind
                .in_a_place_with_wind(env.world, x - fx / 18, y - fy / 18, 2, 2);
            let mut n12 = if flag { tile_frame } else { 0 };
            let mut n13 = 0;
            if flag {
                // The two `Main.weatherVaneBobframe` switch tables
                // (:4752-4778 strong wind, :4782-4808 calm); a bobframe
                // outside 0..=7 matches no arm (n13 stays 0).
                let strong = env.wind.speed_current.abs() > 0.5;
                n13 = match env.inputs.weather_vane_bobframe {
                    0 => 0,
                    1 => 1,
                    2 => {
                        if strong {
                            2
                        } else {
                            0
                        }
                    }
                    3 => {
                        if strong {
                            1
                        } else {
                            -1
                        }
                    }
                    4 => 0,
                    5 => {
                        if strong {
                            -1
                        } else {
                            1
                        }
                    }
                    6 => {
                        if strong {
                            -2
                        } else {
                            0
                        }
                    }
                    7 => -1,
                    _ => 0,
                };
            }
            n12 += n13;
            if n12 < 0 {
                n12 += 12;
            }
            data.add_fr_y = (n12 % 12 * 36) as i16;
        }
        // `case 493` — GolfCupFlag: the flag waves only on frame row 0 and
        // only in wind; two speed tiers, direction picks the row side
        // (TileDrawing.cs:5231-5257). Below 0.1 wind the C# leaves addFrY
        // at the default — `base` carries it.
        493 => {
            if fy == 0 {
                let mut n4 = counter;
                let mut n5 = env.wind.speed_current.abs();
                let n6 = y - fy / 18;
                let n7 = x - fx / 18;
                // C# probes `InAPlaceWithWind(x, num6, 1, 1)` — the RAW x,
                // not num7 (:5238).
                if !env.wind.in_a_place_with_wind(env.world, x, n6, 1, 1) {
                    n5 = 0.0;
                }
                if !(n5 < 0.1) {
                    if n5 < 0.5 {
                        n4 = (n4 / 20 + (n6 + n7)) % 6;
                        data.add_fr_y = ((if !(env.wind.speed_current < 0.0) {
                            n4 + 1
                        } else {
                            6 - n4
                        }) * 36) as i16;
                    } else {
                        n4 = (n4 / 10 + (n6 + n7)) % 6;
                        data.add_fr_y = ((if !(env.wind.speed_current < 0.0) {
                            n4 + 7
                        } else {
                            12 - n4
                        }) * 36) as i16;
                    }
                }
            }
            data.top = 2;
        }
        // `case 428` — WeightedPressurePlate: top += 4 is the static arm's;
        // a pressed plate shifts one 18 px column (TileDrawing.cs:5605-5611).
        428 => {
            if env.wiring.plate_is_pressed(x, y) {
                data.add_fr_x += 18;
            }
        }
        // `case 455` — PartyMonolith: the extra row only while the party is
        // up (TileDrawing.cs:5397-5408).
        455 => {
            data.top = 2;
            let mut n31 = 1 + tile_frame;
            if !env.inputs.party_is_up {
                n31 = 0;
            }
            data.add_fr_y = (n31 * 54) as i16;
        }
        // `case 349` — MushroomStatue: top 2; the temporary-animation frame
        // rewrites tileFrameX (TileDrawing.cs:5332-5341).
        349 => {
            data.top = 2;
            let n45 = fx % 36;
            if let Some(f) = temporary_frame(x - n45 / 18, y - fy % 54 / 18) {
                data.frame_x_override = Some((36 * f + n45) as i16);
            }
        }
        // `case 593` — VolcanoSmall (TileDrawing.cs:5445-5465).
        593 => {
            if fx >= 18 {
                data.add_fr_x = -18;
            }
            data.top = 2;
            if let Some(f) = temporary_frame(x, y) {
                data.add_fr_y = (18 * f) as i16;
            } else if fx < 18 {
                data.add_fr_y = (tile_frame * 18) as i16;
            } else {
                data.add_fr_y = 0;
            }
        }
        // `case 594` — VolcanoLarge (TileDrawing.cs:5466-5487).
        594 => {
            if fx >= 36 {
                data.add_fr_x = -36;
            }
            data.top = 2;
            let n20 = fx % 36;
            if let Some(f) = temporary_frame(x - n20 / 18, y - fy % 36 / 18) {
                data.add_fr_y = (36 * f) as i16;
            } else if fx < 36 {
                data.add_fr_y = (tile_frame * 36) as i16;
            } else {
                data.add_fr_y = 0;
            }
        }
        _ => return None,
    }
    Some(data)
}

/// The case ids slice B owns (the coverage test pins the dispatch). Kept
/// visible outside tests so the sibling slices can check disjointness.
/// The merge dedupe removed A's five ids (518/529/530 first-switch probes,
/// 441/468 second-switch glow rows) — `drawdata_cases_a.rs` owns them.
#[allow(dead_code)]
pub const OWNED_KINDS: [u16; 16] = [
    349, 428, 453, 455, 456, 485, 489, 490, 493, 507, 508, 541, 579, 593, 594, 719,
];

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::tile::FLAG_ACTIVE;
    use tr_core::tiledata::TILE_COUNT;

    fn world() -> World {
        World::new("t".into(), 60, 60)
    }

    fn set_tile(w: &mut World, x: i32, y: i32, kind: u16, liquid: u8) {
        let i = w.index(x, y);
        w.tiles[i] = tr_core::tile::Tile {
            kind,
            liquid,
            flags: FLAG_ACTIVE,
            ..Default::default()
        };
    }

    fn wind() -> Wind {
        let mut wind = Wind::new(0);
        // `Wind::new` defaults world_surface to +inf (every height blows);
        // pin a finite surface so the altitude gate is exercisable.
        wind.world_surface = 30.0;
        // The seeded roll for seed 0 is -0.49 (Xorshift64::from_seed(0),
        // range(-800, 801)); pin calm so the tier tests state their wind
        // explicitly instead of leaning on the seed.
        wind.speed_current = 0.0;
        wind
    }

    fn env<'a>(
        w: &'a World,
        wind: &'a Wind,
        wiring: &'a Wiring,
        inputs: CaseInputs,
    ) -> CaseEnv<'a> {
        CaseEnv {
            world: w,
            wind,
            wiring,
            inputs,
        }
    }

    /// The static table's output — the same `base` the renderer passes.
    fn base(kind: u16, x: i32, y: i32, fx: i16, fy: i16, tf: i32) -> TileDrawData {
        tr_core::drawdata::draw_data(kind, x, y, fx, fy, tf, false)
    }

    fn dd(
        kind: u16,
        x: i32,
        y: i32,
        fx: i16,
        fy: i16,
        tf: i32,
        counter: i32,
        e: &CaseEnv<'_>,
    ) -> TileDrawData {
        case_draw_data(kind, x, y, fx, fy, tf, counter, base(kind, x, y, fx, fy, tf), e)
            .expect("owned kind")
    }

    #[test]
    fn dispatch_covers_exactly_the_owned_kinds() {
        let w = world();
        let wind = wind();
        let wiring = Wiring::new();
        let e = env(&w, &wind, &wiring, CaseInputs::default());
        for kind in 0..TILE_COUNT as u16 {
            let got = case_draw_data(
                kind,
                3,
                7,
                18,
                36,
                0,
                0,
                base(kind, 3, 7, 18, 36, 0),
                &e,
            );
            assert_eq!(
                got.is_some(),
                OWNED_KINDS.contains(&kind),
                "kind {kind}"
            );
        }
    }

    #[test]
    fn case_719_rainbow_cloud_clock_grid() {
        // TileDrawing.cs:4562-4570: n24 = (x + y + (int)(time*15)) % 14.
        let w = world();
        let wind = wind();
        let wiring = Wiring::new();
        let mut inputs = CaseInputs::default();
        // time 0: (3+7)%14 = 10 -> col 2, row 2 -> +576 / +540.
        let e = env(&w, &wind, &wiring, inputs);
        let d = dd(719, 3, 7, 0, 0, 0, 0, &e);
        assert_eq!((d.add_fr_x, d.add_fr_y), (576, 540));
        // time 0.2s -> (int)(0.2*15)=3: (3+7+3)%14=13 -> col 3, row 1.
        inputs.global_time_wrapped_hourly = 0.2;
        let e = env(&w, &wind, &wiring, inputs);
        let d = dd(719, 3, 7, 0, 0, 0, 0, &e);
        assert_eq!((d.add_fr_x, d.add_fr_y), (864, 270));
        // The += rides the tileFrame*38 default: tf=2 -> 76 + 270*row.
        inputs.global_time_wrapped_hourly = 0.0;
        let e = env(&w, &wind, &wiring, inputs);
        let d = dd(719, 0, 0, 0, 0, 2, 0, &e);
        assert_eq!((d.add_fr_x, d.add_fr_y), (0, 76));
    }

    // case_518_lily_pad_liquid_and_ground moved to slice A's tests in the
    // merge dedupe (drawdata_cases_a.rs pins the same C# body).

    #[test]
    fn case_579_golem_head_faces_the_player() {
        // TileDrawing.cs:5104-5130: flag2 = (x*16+8) > Center.X.
        let w = world();
        let wind = wind();
        let wiring = Wiring::new();
        let mut inputs = CaseInputs::default();
        // Cell x=10 -> probe 168. Player right of it (center 200): flag2 false.
        inputs.player_center_x = Some(200.0);
        let e = env(&w, &wind, &wiring, inputs);
        let d = dd(579, 10, 5, 0, 0, 0, 0, &e);
        assert_eq!((d.width, d.height, d.top, d.add_fr_y), (20, 20, -2, 22));
        let d = dd(579, 10, 5, 18, 0, 0, 0, &e);
        assert_eq!(d.add_fr_y, 0);
        // Player left (center 100): flag2 true — the rows swap.
        inputs.player_center_x = Some(100.0);
        let e = env(&w, &wind, &wiring, inputs);
        assert_eq!(dd(579, 10, 5, 0, 0, 0, 0, &e).add_fr_y, 0);
        assert_eq!(dd(579, 10, 5, 18, 0, 0, 0, &e).add_fr_y, 22);
        // No player: flag2 false (documented deviation).
        inputs.player_center_x = None;
        let e = env(&w, &wind, &wiring, inputs);
        assert_eq!(dd(579, 10, 5, 0, 0, 0, 0, &e).add_fr_y, 22);
    }

    #[test]
    fn cases_453_456_balloon_counter_cycles() {
        // TileDrawing.cs:5412-5417.
        let w = world();
        let wind = wind();
        let wiring = Wiring::new();
        let e = env(&w, &wind, &wiring, CaseInputs::default());
        // 453: (counter/20 + (y - fy/18 + x)) % 3 * 54.
        assert_eq!(dd(453, 2, 4, 0, 0, 0, 0, &e).add_fr_y, (6 % 3) * 54);
        assert_eq!(dd(453, 2, 4, 0, 36, 0, 0, &e).add_fr_y, (4 % 3) * 54);
        assert_eq!(dd(453, 2, 4, 0, 0, 0, 40, &e).add_fr_y, ((2 + 6) % 3) * 54);
        // 456: (counter/20 + (y - fy/18 + (x - fx/18))) % 4 * 54.
        assert_eq!(dd(456, 5, 3, 36, 18, 0, 0, &e).add_fr_y, ((3 - 1 + 3) % 4) * 54);
        assert_eq!(dd(456, 5, 3, 36, 18, 0, 60, &e).add_fr_y, ((3 + 5) % 4) * 54);
    }

    #[test]
    fn cases_507_508_starry_glass_twinkle() {
        // TileDrawing.cs:6007-6013: 90 * ((counter + x*11 + y*27) % 160 / 20).
        let w = world();
        let wind = wind();
        let wiring = Wiring::new();
        let e = env(&w, &wind, &wiring, CaseInputs::default());
        for kind in [507u16, 508] {
            assert_eq!(dd(kind, 3, 2, 0, 0, 0, 0, &e).add_fr_y, 90 * ((33 + 54) % 160 / 20));
            assert_eq!(
                dd(kind, 3, 2, 0, 0, 0, 100, &e).add_fr_y,
                90 * ((100 + 33 + 54) % 160 / 20)
            );
            // The default tileFrame*38 row is REPLACED, not added.
            assert_eq!(dd(kind, 0, 0, 0, 0, 3, 0, &e).add_fr_y, 0);
        }
    }

    #[test]
    fn case_541_echo_block_invisible_row() {
        // TileDrawing.cs:6004-6006.
        let w = world();
        let wind = wind();
        let wiring = Wiring::new();
        let mut inputs = CaseInputs::default();
        let e = env(&w, &wind, &wiring, inputs);
        assert_eq!(dd(541, 1, 1, 0, 0, 0, 0, &e).add_fr_y, 90);
        inputs.show_invisible_blocks = true;
        let e = env(&w, &wind, &wiring, inputs);
        assert_eq!(dd(541, 1, 1, 0, 0, 0, 0, &e).add_fr_y, 0);
    }

    #[test]
    fn case_485_antlion_larva_cycle() {
        // TileDrawing.cs:4727-4730: (counter/5 + (y - fy/18 + (x - fx/18))) % 4 * 36.
        let w = world();
        let wind = wind();
        let wiring = Wiring::new();
        let e = env(&w, &wind, &wiring, CaseInputs::default());
        let d = dd(485, 6, 4, 18, 36, 0, 0, &e);
        assert_eq!((d.top, d.add_fr_y), (2, ((4 - 2 + 5) % 4) * 36));
        let d = dd(485, 6, 4, 18, 36, 0, 25, &e);
        assert_eq!(d.add_fr_y, ((5 + 7) % 4) * 36);
    }

    #[test]
    fn case_489_pinwheel_needs_wind_place() {
        // TileDrawing.cs:4731-4741.
        let mut w = world();
        let mut wind = wind();
        let wiring = Wiring::new();
        let e = env(&w, &wind, &wiring, CaseInputs::default());
        // Open air above the surface: spins. n32 = 4-2 = 2, n33 = 6-1 = 5.
        let d = dd(489, 6, 4, 18, 36, 0, 0, &e);
        assert_eq!((d.top, d.add_fr_y), (2, ((2 + 5) % 16) * 54));
        // Below the surface: no wind -> addFrY keeps tileFrame*38.
        let d = dd(489, 6, 40, 18, 36, 2, 0, &e);
        assert_eq!((d.top, d.add_fr_y), (2, 76));
        // Liquid in the 2x3 probe region blocks the wind.
        set_tile(&mut w, 5, 2, 1, 10);
        let e = env(&w, &wind, &wiring, CaseInputs::default());
        assert_eq!(dd(489, 6, 4, 18, 36, 0, 0, &e).add_fr_y, 0);
        wind.world_surface = 100.0;
        let e = env(&w, &wind, &wiring, CaseInputs::default());
        assert_eq!(dd(489, 6, 4, 18, 36, 0, 0, &e).add_fr_y, 0);
    }

    #[test]
    fn case_490_weather_vane_bob_tables() {
        // TileDrawing.cs:4742-4818.
        let w = world();
        let mut wind = wind();
        let wiring = Wiring::new();
        let mut inputs = CaseInputs::default();
        // No wind place (below surface): num12 = 0 -> addFrY 0 even at tf 5.
        let e = env(&w, &wind, &wiring, inputs);
        assert_eq!(dd(490, 2, 40, 0, 0, 5, 0, &e).add_fr_y, 0);
        // In wind, calm: bob 3 -> -1; tf 0 -> num12 = -1 -> +12 = 11 -> 396.
        inputs.weather_vane_bobframe = 3;
        let e = env(&w, &wind, &wiring, inputs);
        assert_eq!(dd(490, 2, 4, 0, 0, 0, 0, &e).add_fr_y, 11 * 36);
        // Strong wind (>0.5): bob 3 -> +1 -> 36.
        wind.speed_current = 0.8;
        let e = env(&w, &wind, &wiring, inputs);
        assert_eq!(dd(490, 2, 4, 0, 0, 0, 0, &e).add_fr_y, 36);
        // Strong wind bob 6 -> -2; tf 1 -> -1 -> 11 -> 396.
        inputs.weather_vane_bobframe = 6;
        let e = env(&w, &wind, &wiring, inputs);
        assert_eq!(dd(490, 2, 4, 0, 0, 1, 0, &e).add_fr_y, 11 * 36);
        // Calm bob 5 -> +1; tf 11 -> 12 % 12 = 0.
        wind.speed_current = 0.2;
        inputs.weather_vane_bobframe = 5;
        let e = env(&w, &wind, &wiring, inputs);
        assert_eq!(dd(490, 2, 4, 0, 0, 11, 0, &e).add_fr_y, 0);
        // The probe origin is sheet-relative: fx 36, fy 36 at (6,4) -> (4,2).
        inputs.weather_vane_bobframe = 2;
        let e = env(&w, &wind, &wiring, inputs);
        assert_eq!(dd(490, 6, 4, 36, 36, 0, 0, &e).add_fr_y, 0); // calm bob 2 -> 0
    }

    #[test]
    fn case_493_golf_flag_wind_tiers() {
        // TileDrawing.cs:5231-5257.
        let w = world();
        let mut wind = wind();
        let wiring = Wiring::new();
        let e = env(&w, &wind, &wiring, CaseInputs::default());
        // Calm (<0.1): addFrY keeps the default; top 2.
        let d = dd(493, 3, 4, 18, 0, 2, 0, &e);
        assert_eq!((d.top, d.add_fr_y), (2, 76));
        // Row != 0: no animation at all.
        wind.speed_current = 0.3;
        let e = env(&w, &wind, &wiring, CaseInputs::default());
        assert_eq!(dd(493, 3, 4, 18, 18, 0, 0, &e).add_fr_y, 0);
        // Tier 1 (0.1..0.5), wind >= 0: n4 = (0/20 + (4 + 2)) % 6 = 0 -> (0+1)*36.
        let d = dd(493, 3, 4, 18, 0, 0, 0, &e);
        assert_eq!((d.top, d.add_fr_y), (2, 36));
        // Negative wind: (6 - n4) * 36.
        wind.speed_current = -0.3;
        let e = env(&w, &wind, &wiring, CaseInputs::default());
        assert_eq!(dd(493, 3, 4, 18, 0, 0, 0, &e).add_fr_y, 6 * 36);
        // Tier 2 (>=0.5): n4 = (counter/10 + 6) % 6; counter 30 -> 9%6=3 -> (3+7)*36.
        wind.speed_current = 0.9;
        let e = env(&w, &wind, &wiring, CaseInputs::default());
        assert_eq!(dd(493, 3, 4, 18, 0, 0, 30, &e).add_fr_y, 10 * 36);
        // Below the surface the wind probe fails -> num5 = 0 -> default row.
        assert_eq!(dd(493, 3, 40, 18, 0, 0, 30, &e).add_fr_y, 0);
    }

    // case_529_sea_oats_biome_and_beach / case_530_oasis_plant_biome_row
    // moved to slice A's tests in the merge dedupe (drawdata_cases_a.rs
    // pins the same C# bodies).

    #[test]
    fn case_428_pressed_plate_shifts_a_column() {
        // TileDrawing.cs:5605-5611.
        let mut w = world();
        let wind = wind();
        let mut wiring = Wiring::new();
        set_tile(&mut w, 5, 5, 428, 0);
        let e = env(&w, &wind, &wiring, CaseInputs::default());
        let d = dd(428, 5, 5, 0, 0, 0, 0, &e);
        assert_eq!((d.top, d.add_fr_x), (4, 0));
        wiring.press_plate(&mut w, 5, 5);
        let e = env(&w, &wind, &wiring, CaseInputs::default());
        assert_eq!(dd(428, 5, 5, 0, 0, 0, 0, &e).add_fr_x, 18);
        wiring.release_plate(&mut w, 5, 5);
        let e = env(&w, &wind, &wiring, CaseInputs::default());
        assert_eq!(dd(428, 5, 5, 0, 0, 0, 0, &e).add_fr_x, 0);
    }

    #[test]
    fn case_455_party_monolith_row() {
        // TileDrawing.cs:5397-5408.
        let w = world();
        let wind = wind();
        let wiring = Wiring::new();
        let mut inputs = CaseInputs::default();
        let e = env(&w, &wind, &wiring, inputs);
        let d = dd(455, 1, 1, 0, 0, 3, 0, &e);
        assert_eq!((d.top, d.add_fr_y), (2, 0));
        inputs.party_is_up = true;
        let e = env(&w, &wind, &wiring, inputs);
        assert_eq!(dd(455, 1, 1, 0, 0, 3, 0, &e).add_fr_y, 4 * 54);
        assert_eq!(dd(455, 1, 1, 0, 0, 0, 0, &e).add_fr_y, 54);
    }

    #[test]
    fn temp_animation_cases_match_the_static_fallback() {
        // 349/593/594: the Animation registry is empty (the C#
        // `GetTemporaryFrame == false` path), so the full bodies reproduce
        // the static arms byte-for-byte at any tile_frame. (441/468 moved
        // to slice A in the merge dedupe.)
        let w = world();
        let wind = wind();
        let wiring = Wiring::new();
        let e = env(&w, &wind, &wiring, CaseInputs::default());
        for kind in [349u16, 593, 594] {
            for (fx, fy) in [(0i16, 0i16), (18, 18), (36, 36), (54, 72)] {
                for tf in [0, 2, 7] {
                    assert_eq!(
                        dd(kind, 3, 7, fx, fy, tf, 0, &e),
                        base(kind, 3, 7, fx, fy, tf),
                        "kind {kind} fx {fx} fy {fy} tf {tf}"
                    );
                }
            }
        }
    }

    #[test]
    fn neutral_inputs_reproduce_the_vanilla_t0_static_state() {
        // With counter 0, no wind, no party, no plates, time 0 and no
        // player, the wind/party/plate/temp-animation cases equal the
        // static table; the position-driven cycles (453/456/485/507/508/
        // 541/579/719) intentionally differ — they are the ported
        // engine parts (per-case tests above pin their exact values).
        let w = world();
        let mut wind = wind();
        wind.world_surface = 0.0; // nothing blows -> 489/490/493 take the calm path
        let wiring = Wiring::new();
        let e = env(&w, &wind, &wiring, CaseInputs::default());
        for kind in [489u16, 490, 493, 428, 455, 349, 593, 594] {
            assert_eq!(
                dd(kind, 3, 7, 18, 36, 0, 0, &e),
                base(kind, 3, 7, 18, 36, 0),
                "kind {kind}"
            );
        }
    }
}
