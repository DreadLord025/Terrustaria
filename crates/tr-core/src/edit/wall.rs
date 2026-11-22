//! The background-wall edit family (S1-K-4 round 2): `WorldGen.KillWall` /
//! `WorldGen.PlaceWall` and the wall re-frame hook. Moved verbatim from
//! `edit.rs`; behaviour frozen (same revision bump, same `reframe_area` call).

use super::{touch, SoundCue, WALL_UNBREAKABLE};
use crate::framing;
use crate::world::World;

/// `WorldGen.KillWall_PlaySounds` (`WorldGen.cs:60423-60441`, S1-E-21
/// world-sounds wave): the four-arm wall-break cue table. The C# plays it
/// from `KillWall` (`:60385`) BEFORE the dust and the drop — and on the
/// `fail` path too (`KillWall_CheckFailure` `:60410-60421`: the dungeon
/// walls before Skeletron, wall 87 before Golem); the port's `break_wall`
/// has no fail path, so the cue rides the successful break only (ANALYSIS,
/// S1-E-21i).
///
/// The arms: the glass/stone family → the legacy id 13 (Shatter, no pitch
/// draw, `LegacySoundPlayer.cs:783-792`); the grass/jungle family → id 6
/// (Grass, one `Next(-30, 31) * 0.01`, `:706-716`); wall 357 →
/// `SoundID.NPCHit25` (`SoundID.cs:1281`, the id-3 style-25 half-volume
/// hit); everything else → id 0 (the Dig variation + pitch, `:440-452`).
pub fn kill_wall_sound(wall: u16) -> SoundCue {
    // `:60425-60428` — the Shatter arm.
    if wall == 241
        || (88..=93).contains(&wall)
        || matches!(
            wall,
            21 | 186 | 136 | 137 | 168 | 169 | 172 | 226 | 227 | 242 | 243 | 347 | 365
        )
    {
        return (13, 1);
    }
    // `:60429-60432` — the Grass arm.
    if (63..=70).contains(&wall) || matches!(wall, 264 | 268 | 265) {
        return (6, 1);
    }
    // `:60433-60436` — the NPCHit25 arm.
    if wall == 357 {
        return (3, 25);
    }
    // `:60437-60440` — the default Dig arm.
    (0, 1)
}

/// `WorldGen.KillWall_GetItemDrops` (`WorldGen.cs:60452-61343`): the item a
/// broken wall drops — the wall's own placing item for the safe walls, the
/// safe-conversion item for the dungeon/glass/brick unsafe walls, `0` for the
/// natural unsafe walls (dirt 2, mud 15, cave 54-59, …). The port's
/// `break_wall`/`replace_wall` callers use this instead of a reverse
/// `createWall` lookup, which misses the unsafe→safe drop conversions.
pub fn kill_wall_drop(wall: u16) -> u16 {
    // The leading switch (`:60454-60593`).
    match wall {
        237 => return 4233,
        238 => return 4234,
        239 => return 4235,
        240 => return 4236,
        246 => return 4486,
        247 => return 4487,
        248 => return 4488,
        249 => return 4489,
        250 => return 4490,
        251 => return 4491,
        252 => return 4492,
        253 => return 4493,
        254 => return 4494,
        255 => return 4495,
        314 => return 4647,
        256 => return 4496,
        257 => return 4497,
        258 => return 4498,
        259 => return 4499,
        260 => return 4500,
        261 => return 4501,
        262 => return 4502,
        263 => return 4503,
        264 => return 4504,
        265 => return 4505,
        266 => return 4506,
        267 => return 4507,
        268 => return 4508,
        269 => return 4509,
        270 => return 4510,
        271 => return 4511,
        274 => return 4512,
        275 => return 3273,
        276 => return 4513,
        277 => return 4514,
        278 => return 4515,
        279 => return 4516,
        280 => return 4517,
        281 => return 4518,
        282 => return 4519,
        283 => return 4520,
        284 => return 4521,
        285 => return 4522,
        286 => return 4523,
        287 => return 4524,
        288 => return 4525,
        289 => return 4526,
        290 => return 4527,
        291 => return 4528,
        292 => return 4529,
        293 => return 4530,
        294 => return 4531,
        295 => return 4532,
        296 => return 4533,
        297 => return 4534,
        298 => return 4535,
        299 => return 4536,
        300 => return 4537,
        301 => return 4538,
        302 => return 4539,
        303 => return 4540,
        304 => return 3340,
        305 => return 3341,
        306 => return 3342,
        307 => return 3343,
        308 => return 3344,
        309 => return 3345,
        310 => return 3346,
        311 => return 3348,
        _ => {}
    }
    // The default arm's sequential `if` writes (`:60594-61340`); later matches
    // overwrite earlier ones exactly as the C# does (the sets are disjoint, so
    // a plain match is equivalent — the two `else if` chains at `:61088-61123`
    // only gate within their own chain).
    let mut result = 0u16;
    match wall {
        168 => result = 2696,
        169 => result = 2698,
        226 => result = 3752,
        227 => result = 3753,
        228 => result = 3760,
        229 => result = 3761,
        230 => result = 3762,
        142 => result = 2263,
        143 => result = 2264,
        144 => result = 2271,
        149 => result = 2505,
        150 => result = 2507,
        151 => result = 2506,
        152 => result = 2508,
        316 => result = 5216,
        317 => result = 5217,
        245 => result = 4424,
        315 => result = 4667,
        1 => result = 26,
        4 => result = 93,
        5 => result = 130,
        6 => result = 132,
        7 => result = 135,
        8 => result = 138,
        9 => result = 140,
        10 => result = 142,
        11 => result = 144,
        12 => result = 146,
        14 => result = 330,
        224 => result = 3472,
        323 => result = 5409,
        324 => result = 5410,
        325 => result = 5411,
        326 => result = 5412,
        327 => result = 5413,
        328 => result = 5414,
        329 => result = 5415,
        330 => result = 5416,
        331 => result = 5418,
        332 => result = 5420,
        333 => result = 5422,
        334 => result = 5424,
        335 => result = 5426,
        336 => result = 5428,
        337 => result = 5434,
        338 => result = 5436,
        339 => result = 5430,
        340 => result = 5432,
        341 => result = 5445,
        342 => result = 5446,
        343 => result = 5447,
        344 => result = 5448,
        345 => result = 5449,
        346 => result = 5450,
        177 => result = 3067,
        167 => result = 2691,
        60 => result = 3584,
        231 => result = 3952,
        232 => result = 3954,
        225 => result = 3751,
        233 => result = 3956,
        234 => result = 4052,
        235 => result = 4053,
        236 => result = 4140,
        312 => result = 4565,
        313 => result = 4548,
        319 => result = 5307,
        318 => result = 5291,
        179 => result = 3083,
        183 => result = 3082,
        321 => result = 5397,
        322 => result = 5399,
        181 => result = 3089,
        184 => result = 3088,
        186 => result = 3238,
        320 => result = 5396,
        136 => result = 2169,
        137 => result = 2170,
        172 => result = 2788,
        242 => result = 4279,
        243 => result = 4280,
        347 => result = 5494,
        348 => result = 5623,
        351 => result = 5711,
        352 => result = 5734,
        353 => result = 5919,
        354 => result = 5921,
        355 => result = 5923,
        356 => result = 5925,
        357 => result = 5927,
        358 => result = 5931,
        145 => result = 2333,
        16 => result = 30,
        17 => result = 135,
        18 => result = 138,
        19 => result = 140,
        20 => result = 330,
        21 => result = 392,
        86 | 108 => result = 1126,
        173 => result = 2789,
        174 => result = 2790,
        175 => result = 2791,
        176 => result = 2861,
        182 => result = 3101,
        133 => result = 2158,
        134 => result = 2159,
        135 => result = 2160,
        113 => result = 1726,
        114 => result = 1728,
        115 => result = 1730,
        146 => result = 2432,
        147 => result = 2433,
        148 => result = 2434,
        22 => result = 417,
        23 => result = 418,
        24 => result = 419,
        25 => result = 420,
        26 => result = 421,
        29 => result = 587,
        30 => result = 592,
        31 => result = 595,
        32 => result = 605,
        33 => result = 606,
        34 => result = 608,
        35 => result = 610,
        36 => result = 615,
        37 => result = 616,
        38 => result = 617,
        39 => result = 618,
        41 => result = 622,
        42 => result = 623,
        43 => result = 624,
        44 => result = 663,
        45 => result = 720,
        46 => result = 721,
        47 => result = 722,
        66 => result = 745,
        67 => result = 746,
        68 => result = 747,
        84 => result = 884,
        72 => result = 750,
        73 => result = 752,
        74 => result = 764,
        85 => result = 927,
        75 => result = 768,
        76 => result = 769,
        77 => result = 770,
        82 => result = 825,
        27 => result = 479,
        106 => result = 1447,
        107 => result = 1448,
        109 => result = 1590,
        110 => result = 1592,
        111 => result = 1594,
        78 => result = 1723,
        87 | 112 => result = 1102,
        94 | 100 => result = 1378,
        95 | 101 => result = 1379,
        96 | 102 => result = 1380,
        97 | 103 => result = 1381,
        98 | 104 => result = 1382,
        99 | 105 => result = 1383,
        241 => result = 4260,
        _ => {}
    }
    // The range writes that follow the singles (`:60901-60948`, `:61124-61131`,
    // `:61332-61339`) and the trailing switch (`:61009-61035`).
    if (153..=166).contains(&wall) {
        result = match wall {
            154 => 2679,
            158 => 2680,
            166 => 2689,
            163 => 2690,
            165 => 2687,
            162 => 2688,
            156 => 2683,
            160 => 2684,
            164 => 2685,
            161 => 2686,
            155 => 2681,
            159 => 2682,
            153 => 2677,
            157 => 2678,
            _ => result,
        };
    }
    match wall {
        359 => result = 5954,
        360 => result = 5997,
        361 => result = 6020,
        362 => result = 6043,
        363 => result = 6066,
        364 => result = 6089,
        365 => result = 6110,
        366 => result = 6133,
        _ => {}
    }
    if (116..=125).contains(&wall) {
        result = 1948 + wall - 116;
    }
    if (126..=132).contains(&wall) {
        result = 2008 + wall - 126;
    }
    if (88..=93).contains(&wall) {
        result = 1267 + wall - 88;
    }
    if (138..=141).contains(&wall) {
        result = 2210 + wall - 138;
    }
    result
}

/// `WallID.Count` (`WallID.cs`, `Count = 367`) — the upper bound
/// `WorldGen.ReplaceWall` refuses (`WorldGen.cs:62532`).
pub const WALL_COUNT: u16 = 367;

/// `Main.wallDungeon` (`Main.cs:10594-10606` sets the nine members true): the
/// dungeon walls whose removal `KillWall_CheckFailure` refuses before Skeletron
/// (`WorldGen.cs:60412`). The same nine the caster/bat teleport gates use.
fn is_dungeon_wall(wall: u16) -> bool {
    matches!(wall, 7 | 8 | 9 | 94..=99)
}

/// `WorldGen.KillWall_CheckFailure` (`WorldGen.cs:60410-60421`): the dungeon
/// walls before Skeletron (`NPC.downedBoss3`) and the temple wall 87 before
/// Golem (`NPC.downedGolemBoss`) refuse the break. The port has no per-tick
/// `fail` path (S1-E-21i), so a refusal is a silent no-op rather than the C#'s
/// 3-dust + sound + re-frame tick.
fn kill_wall_check_failure(world: &World, wall: u16) -> bool {
    if is_dungeon_wall(wall) && !world.downed_skeletron {
        return true;
    }
    if wall == 87 && !world.downed_golem {
        return true;
    }
    false
}

/// `Tile.ClearWallPaintAndCoating` (`Tile.cs:911-916`): the wall paint and both
/// coating bits come off. `KillWall` (`WorldGen.cs:60402`) and `ReplaceWall`
/// (`:62553`) run it so a removed/replaced wall never leaks its paint onto the
/// next wall placed in the cell.
fn clear_wall_paint_and_coating(world: &mut World, x: i32, y: i32) {
    world.set_wall_paint(x, y, 0);
    world.set_invisible_wall(x, y, false);
    world.set_fullbright_wall(x, y, false);
}

/// `Main.wallHouse` — the FULL table (`Main.cs:9741-9789`, `:10353-10577`,
/// incl. the `for (num4 = 153; num4 < 167; num4++)` loop `:10367-10370`): the
/// 279 "safe" walls a house may be enclosed by, and the walls the hammer /
/// replacement / friendly-wall gates treat as player-made. This is the faithful
/// wall-edit table; `housing::is_house_wall` now DELEGATES to this table
/// (the W4-g44 full-table fix — `CheckRoom`'s ±2 scan reads `Main.wallHouse`
/// directly, `WorldGen.cs:6204`/`:6212`; the old narrow subset refused the
/// disc wall 82). Unsafe worldgen walls (dirt 2, stone-unsafe
/// variants, dungeon 7/8/9/94-99, biome walls, …) are deliberately absent.
pub fn wall_house(wall: u16) -> bool {
    matches!(
        wall,
        1 | 4..=6
            | 10..=12
            | 16..=27
            | 29..=39
            | 41..=47
            | 60
            | 66..=68
            | 72..=78
            | 82
            | 84..=85
            | 88..=93
            | 100..=169
            | 172..=177
            | 179
            | 181..=184
            | 186
            | 224..=243
            | 245..=348
            | 351..=366
    )
}

/// `Player.CanPlayerSmashWall` (`Player.cs:45917-45940`): the hammer may break
/// the wall unless it is the unbreakable 350; a non-house (unsafe) wall is only
/// smashable while some cell of its 3×3 neighbourhood is open (`wall == 0`) or
/// itself a house wall — a fully enclosed unsafe wall refuses the hammer.
pub fn can_player_smash_wall(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    if t.wall == WALL_UNBREAKABLE {
        return false;
    }
    if wall_house(t.wall) {
        return true;
    }
    for i in (x - 1)..=(x + 1) {
        for j in (y - 1)..=(y + 1) {
            if world
                .tile(i, j)
                .is_some_and(|n| n.wall == 0 || wall_house(n.wall))
            {
                return true;
            }
        }
    }
    false
}

/// `WorldGen.NearFriendlyWall` (`WorldGen.cs:52258-52291`): true at the world
/// edge, or when the cell or any of its four neighbours is open or a house
/// wall — the gate `PlaceThing_TryReplacingWalls` reads before replacing an
/// unsafe wall with a safe one.
pub fn near_friendly_wall(world: &World, x: i32, y: i32) -> bool {
    if x < 2 || x >= world.width - 2 || y < 2 || y >= world.height - 2 {
        return true;
    }
    let open_or_house = |wx: i32, wy: i32| {
        world
            .tile(wx, wy)
            .is_some_and(|t| t.wall == 0 || wall_house(t.wall))
    };
    open_or_house(x, y)
        || open_or_house(x - 1, y)
        || open_or_house(x + 1, y)
        || open_or_house(x, y - 1)
        || open_or_house(x, y + 1)
}

/// Removes the background wall at `(x, y)`. Returns the wall id.
///
/// `WorldGen.KillWall` (`WorldGen.cs:60368-60408`): the break refuses the
/// unbreakable 350 (the port folds `Player.CanPlayerSmashWall`'s gate in here,
/// ANALYSIS) and the `KillWall_CheckFailure` progression walls; on success it
/// clears the wall paint/coating (`:60402`) before the re-frame. The sound
/// (`KillWall_PlaySounds`), dust and item drop are app-side.
pub fn break_wall(world: &mut World, x: i32, y: i32) -> Option<u16> {
    let t = *world.tile(x, y)?;
    if t.wall == 0 || t.wall == WALL_UNBREAKABLE {
        return None;
    }
    if kill_wall_check_failure(world, t.wall) {
        return None;
    }
    let wall = t.wall;
    let i = world.index(x, y);
    world.tiles[i].wall = 0;
    world.tiles[i].set_wall_frame(0, 0);
    clear_wall_paint_and_coating(world, x, y);
    after_wall_edit(world, x, y);
    Some(wall)
}

/// Places a background wall into a cell that has none. The caller checks reach and
/// support; a wall may sit behind an active tile.
///
/// `WorldGen.PlaceWall` (`WorldGen.cs:75125-75144`): the boundary guard
/// (`i <= 1 || j <= 1 || i >= maxTilesX - 2 || j >= maxTilesY - 2`, `:75127`)
/// refuses the outer two-cell ring, then the write lands only on an empty wall.
/// The sound (`:75141`) is app-side.
pub fn place_wall(world: &mut World, x: i32, y: i32, wall: u16) -> bool {
    if x <= 1 || y <= 1 || x >= world.width - 2 || y >= world.height - 2 {
        return false;
    }
    let Some(t) = world.tile(x, y).copied() else {
        return false;
    };
    if t.wall != 0 || wall == 0 {
        return false;
    }
    let i = world.index(x, y);
    world.tiles[i].wall = wall;
    world.tiles[i].set_wall_frame(0, 0);
    after_wall_edit(world, x, y);
    true
}

/// `WorldGen.ReplaceWall` (`WorldGen.cs:62530-62556`): swaps an existing wall
/// for `target` (the unsafe→safe conversion the placement route uses). Refuses
/// `target >= WallID.Count`, an empty cell, `target == 0`, the unbreakable 350
/// and the `KillWall_CheckFailure` progression walls; on success it writes the
/// new wall, clears the old paint/coating (`:62553`) and re-frames. Returns the
/// OLD wall id so the caller can play its break sound and drop its item
/// (`KillWall_PlaySounds`/`KillWall_DropItems`, `:62550-62551`); the 10-dust
/// burst is app-side. `None` = refused.
pub fn replace_wall(world: &mut World, x: i32, y: i32, target: u16) -> Option<u16> {
    if target >= WALL_COUNT {
        return None;
    }
    let t = *world.tile(x, y)?;
    if t.wall == 0 || target == 0 || t.wall == WALL_UNBREAKABLE {
        return None;
    }
    if kill_wall_check_failure(world, t.wall) {
        return None;
    }
    let old = t.wall;
    let i = world.index(x, y);
    world.tiles[i].wall = target;
    clear_wall_paint_and_coating(world, x, y);
    after_wall_edit(world, x, y);
    Some(old)
}

fn after_wall_edit(world: &mut World, x: i32, y: i32) {
    touch(world, x, y);
    framing::reframe_area(world, x, y, 2, true);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edit::tests::ground_world;

    #[test]
    fn walls_place_and_break_and_reframe_neighbours() {
        let mut w = ground_world();
        assert!(place_wall(&mut w, 10, 15, 2));
        assert_eq!(w.tile(10, 15).unwrap().wall, 2);
        assert!(!place_wall(&mut w, 10, 15, 1), "already walled");
        assert!(place_wall(&mut w, 11, 15, 2));
        // The shared edge reframes to a connecting cell.
        assert_ne!(w.tile(10, 15).unwrap().wall_col(), 0);
        assert_eq!(break_wall(&mut w, 10, 15), Some(2));
        assert_eq!(w.tile(10, 15).unwrap().wall, 0);
        assert_eq!(break_wall(&mut w, 10, 15), None);
    }

    /// S1-E-21i — the `KillWall_PlaySounds` table
    /// (`WorldGen.cs:60423-60441`): every arm's membership, verbatim.
    #[test]
    fn s1_e_21i_kill_wall_sound_table_arms() {
        // Arm 1 (`:60425`): Shatter (13) — 241, 88..93, and the 13 singles.
        for wall in [
            241, 88, 89, 90, 91, 92, 93, 21, 186, 136, 137, 168, 169, 172, 226, 227, 242,
            243, 347, 365,
        ] {
            assert_eq!(kill_wall_sound(wall), (13, 1), "wall {wall} shatters");
        }
        // Arm 2 (`:60429`): Grass (6) — 63..70 plus 264/268/265.
        for wall in [63, 64, 65, 66, 67, 68, 69, 70, 264, 268, 265] {
            assert_eq!(kill_wall_sound(wall), (6, 1), "wall {wall} is grass");
        }
        // Arm 3 (`:60433`): wall 357 → SoundID.NPCHit25 (`SoundID.cs:1281`).
        assert_eq!(kill_wall_sound(357), (3, 25));
        // Arm 4 (`:60437`): everything else → the Dig 0.
        for wall in [1, 2, 4, 20, 22, 62, 71, 87, 94, 185, 240, 244, 263, 266, 356, 358] {
            assert_eq!(kill_wall_sound(wall), (0, 1), "wall {wall} digs");
        }
        // The arm boundaries: 87 and 94 sit OUTSIDE the 88..93 window,
        // 62/71 outside 63..70, 240/244 outside the singles.
        assert_eq!(kill_wall_sound(87), (0, 1));
        assert_eq!(kill_wall_sound(94), (0, 1));
    }

    // ---- S1-P-i33: the wall placement/removal census fixes ----------------

    /// `WorldGen.PlaceWall`'s boundary guard (`WorldGen.cs:75127`): the outer
    /// two-cell ring refuses the write (`i <= 1 || j <= 1 || i >= w-2 || j >= h-2`).
    #[test]
    fn s1_p_i33_place_wall_refuses_the_outer_ring() {
        let mut w = ground_world(); // 32x32
        // Inside the guard.
        assert!(place_wall(&mut w, 2, 2, 4));
        assert!(place_wall(&mut w, 29, 29, 4));
        // The refused ring: x/y <= 1 and x/y >= 30.
        for (x, y) in [(1, 5), (5, 1), (30, 5), (5, 30), (0, 5), (31, 5)] {
            assert!(!place_wall(&mut w, x, y, 4), "({x},{y}) is the outer ring");
            assert_eq!(w.tile(x, y).unwrap().wall, 0);
        }
    }

    /// `Main.wallHouse` (`Main.cs:9741-9789`, `:10353-10577`): the full 279-wall
    /// table — the safe walls the hammer/replacement gates treat as player-made,
    /// and the unsafe worldgen walls that are absent.
    #[test]
    fn s1_p_i33_wall_house_is_the_full_main_table() {
        // Safe members across every band (incl. the 153..166 loop and the
        // 129..152 / 246..311 / 351..366 bands the housing subset drops).
        for wall in [
            1, 4, 5, 6, 10, 16, 21, 27, 60, 74, 75, 78, 82, 88, 100, 128, 129, 133, 142,
            145, 152, 153, 166, 167, 168, 172, 177, 179, 186, 224, 243, 245, 246, 272, 311,
            314, 348, 351, 366,
        ] {
            assert!(wall_house(wall), "wall {wall} is a house wall");
        }
        // Unsafe walls: the natural/biome/dungeon walls the table omits.
        for wall in [
            0, 2, 3, 7, 8, 9, 13, 14, 15, 28, 40, 48, 54, 62, 63, 71, 79, 80, 83, 86, 87,
            94, 99, 170, 171, 178, 180, 185, 187, 244, 349, 350,
        ] {
            assert!(!wall_house(wall), "wall {wall} is unsafe");
        }
        // The exact count of the C# table (279 distinct members).
        let count = (0..WALL_COUNT).filter(|&w| wall_house(w)).count();
        assert_eq!(count, 279, "Main.wallHouse has 279 members");
    }

    /// `Player.CanPlayerSmashWall` (`Player.cs:45917-45940`): 350 refuses; a
    /// house wall always smashes; an unsafe wall smashes only while some 3×3
    /// neighbour is open or a house wall.
    #[test]
    fn s1_p_i33_can_player_smash_wall_gates() {
        let mut w = ground_world();
        // A house wall (wood 4) smashes even fully enclosed by unsafe walls.
        let i = w.index(10, 10);
        w.tiles[i].wall = 4;
        for x in 9..=11 {
            for y in 9..=11 {
                if (x, y) != (10, 10) {
                    let j = w.index(x, y);
                    w.tiles[j].wall = 2; // unsafe dirt
                }
            }
        }
        assert!(can_player_smash_wall(&w, 10, 10));
        // The unbreakable 350 always refuses.
        w.tiles[i].wall = WALL_UNBREAKABLE;
        assert!(!can_player_smash_wall(&w, 10, 10));
        // An unsafe wall fully enclosed by the SAME unsafe wall refuses.
        w.tiles[i].wall = 2;
        assert!(!can_player_smash_wall(&w, 10, 10));
        // Open one neighbour → the unsafe wall becomes smashable.
        let j = w.index(9, 9);
        w.tiles[j].wall = 0;
        assert!(can_player_smash_wall(&w, 10, 10));
        // A house-wall neighbour also opens it.
        w.tiles[j].wall = 4;
        assert!(can_player_smash_wall(&w, 10, 10));
    }

    /// `WorldGen.NearFriendlyWall` (`WorldGen.cs:52258-52291`): the edge returns
    /// true; an unsafe wall surrounded by unsafe walls returns false; an open
    /// or house-wall neighbour returns true.
    #[test]
    fn s1_p_i33_near_friendly_wall_gates() {
        let mut w = ground_world();
        // Edge cells return true regardless.
        assert!(near_friendly_wall(&w, 1, 5));
        assert!(near_friendly_wall(&w, 30, 5));
        // An unsafe wall enclosed by unsafe walls: false.
        for x in 9..=11 {
            for y in 9..=11 {
                let j = w.index(x, y);
                w.tiles[j].wall = 2;
            }
        }
        assert!(!near_friendly_wall(&w, 10, 10));
        // Open the cell itself → true.
        let i = w.index(10, 10);
        w.tiles[i].wall = 0;
        assert!(near_friendly_wall(&w, 10, 10));
        // Restore, then a house-wall neighbour → true.
        w.tiles[i].wall = 2;
        let j = w.index(11, 10);
        w.tiles[j].wall = 4;
        assert!(near_friendly_wall(&w, 10, 10));
    }

    /// `WorldGen.ReplaceWall` (`WorldGen.cs:62530-62556`): swaps an existing
    /// wall, refuses empty/0-target/350/out-of-count, clears the paint/coating,
    /// and returns the OLD wall id.
    #[test]
    fn s1_p_i33_replace_wall_swaps_and_clears_paint() {
        let mut w = ground_world();
        let i = w.index(10, 10);
        w.tiles[i].wall = 2; // unsafe dirt
        w.set_wall_paint(10, 10, 7);
        w.set_invisible_wall(10, 10, true);
        w.set_fullbright_wall(10, 10, true);
        // The unsafe→safe conversion returns the old wall and clears coating.
        assert_eq!(replace_wall(&mut w, 10, 10, 16), Some(2));
        assert_eq!(w.tile(10, 10).unwrap().wall, 16);
        assert_eq!(w.wall_paint(10, 10), 0);
        assert!(!w.invisible_wall(10, 10));
        assert!(!w.fullbright_wall(10, 10));
        // Refusals: empty cell, target 0, target >= Count, unbreakable 350.
        let j = w.index(12, 12);
        assert_eq!(replace_wall(&mut w, 12, 12, 16), None, "empty cell");
        w.tiles[j].wall = 4;
        assert_eq!(replace_wall(&mut w, 12, 12, 0), None, "target 0");
        assert_eq!(replace_wall(&mut w, 12, 12, WALL_COUNT), None, "out of count");
        w.tiles[j].wall = WALL_UNBREAKABLE;
        assert_eq!(replace_wall(&mut w, 12, 12, 16), None, "unbreakable");
    }

    /// `KillWall_CheckFailure` (`WorldGen.cs:60410-60421`): the dungeon walls
    /// refuse before Skeletron, the temple wall 87 before Golem; both open
    /// after the boss. `break_wall` and `replace_wall` honour it.
    #[test]
    fn s1_p_i33_progression_walls_refuse_until_the_boss() {
        let mut w = ground_world();
        let i = w.index(10, 10);
        w.tiles[i].wall = 7; // blue dungeon unsafe
        assert_eq!(break_wall(&mut w, 10, 10), None, "pre-Skeletron");
        assert_eq!(replace_wall(&mut w, 10, 10, 17), None, "pre-Skeletron replace");
        w.downed_skeletron = true;
        assert_eq!(break_wall(&mut w, 10, 10), Some(7), "post-Skeletron");
        // The temple wall 87 before Golem.
        let j = w.index(12, 12);
        w.tiles[j].wall = 87;
        assert_eq!(break_wall(&mut w, 12, 12), None, "pre-Golem");
        w.downed_golem = true;
        assert_eq!(break_wall(&mut w, 12, 12), Some(87), "post-Golem");
    }

    /// `break_wall` clears the wall paint/coating (`WorldGen.cs:60402` →
    /// `Tile.ClearWallPaintAndCoating`, `Tile.cs:911-916`) so a removed wall
    /// never leaks its paint onto the next placement.
    #[test]
    fn s1_p_i33_break_wall_clears_paint_and_coating() {
        let mut w = ground_world();
        let i = w.index(10, 10);
        w.tiles[i].wall = 4;
        w.set_wall_paint(10, 10, 9);
        w.set_invisible_wall(10, 10, true);
        w.set_fullbright_wall(10, 10, true);
        assert_eq!(break_wall(&mut w, 10, 10), Some(4));
        assert_eq!(w.wall_paint(10, 10), 0);
        assert!(!w.invisible_wall(10, 10));
        assert!(!w.fullbright_wall(10, 10));
    }

    /// `WorldGen.KillWall_GetItemDrops` (`WorldGen.cs:60452-61343`): the drop
    /// conversions the reverse `createWall` lookup misses — unsafe dungeon →
    /// safe item, stone 1 → StoneBlock 26, glass 21 → 392, natural unsafe → 0.
    #[test]
    fn s1_p_i33_kill_wall_drop_conversions() {
        // The unsafe→safe dungeon conversions (`:60685-60700`, `:61304-61327`).
        assert_eq!(kill_wall_drop(7), 135);
        assert_eq!(kill_wall_drop(8), 138);
        assert_eq!(kill_wall_drop(9), 140);
        assert_eq!(kill_wall_drop(94), 1378);
        assert_eq!(kill_wall_drop(99), 1383);
        // The leading switch (`:60456-60593`).
        assert_eq!(kill_wall_drop(237), 4233);
        assert_eq!(kill_wall_drop(275), 3273);
        assert_eq!(kill_wall_drop(304), 3340);
        assert_eq!(kill_wall_drop(311), 3348);
        // The default-arm singles.
        assert_eq!(kill_wall_drop(1), 26); // stone → StoneBlock
        assert_eq!(kill_wall_drop(16), 30); // dirt → DirtWall item
        assert_eq!(kill_wall_drop(21), 392); // glass → GlassWall item
        assert_eq!(kill_wall_drop(168), 2696);
        assert_eq!(kill_wall_drop(87), 1102); // temple → LihzahrdBrickWall
        // The range writes (`:61124-61131`, `:61332-61339`).
        assert_eq!(kill_wall_drop(116), 1948);
        assert_eq!(kill_wall_drop(125), 1957);
        assert_eq!(kill_wall_drop(126), 2008);
        assert_eq!(kill_wall_drop(88), 1267);
        assert_eq!(kill_wall_drop(93), 1272);
        assert_eq!(kill_wall_drop(138), 2210);
        // The 153..166 inner switch (`:60901-60948`).
        assert_eq!(kill_wall_drop(153), 2677);
        assert_eq!(kill_wall_drop(154), 2679);
        // Natural unsafe walls drop nothing.
        assert_eq!(kill_wall_drop(2), 0); // dirt unsafe
        assert_eq!(kill_wall_drop(15), 0); // mud unsafe
        assert_eq!(kill_wall_drop(63), 0); // grass unsafe
    }
}
