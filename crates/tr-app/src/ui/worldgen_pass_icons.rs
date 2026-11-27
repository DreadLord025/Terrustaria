//! `GenPassElement.passIcons` — W4-f23 (census lane 4, research/18e): the
//! static icon dictionary `UIWorldGenDebug.GenPassElement.InitPassIcons`
//! fills (`GameContent/UI/States/UIWorldGenDebug.cs:306-416`, 108 rows) and
//! `GetPassIcon`'s lookup + fallback (`:418-429`). The keys are the
//! `GenPassNameID` string constants (`ID/GenPassNameID.cs:223-330`) — the
//! same vocabulary `tr-core`'s worldgen pipeline uses (`Step::vanilla`).
//! The element half (factories/fit/centring) lives in
//! `ui/worldgen_debug_elements.rs::PassIconEntry`; this is its data table.
//!
//! Wiring census (W4-f23 wiring pass): DEBUG-DEFERRED with its element half —
//! the only consumer is `UIWorldGenDebug.GenPassElement` (d86 scope-out, no
//! driver: the F5 arm is parked at `app/flow.rs`, the F3 workbench has no
//! pass-list tab). The name-mapping half IS ported (W4-f23 deviation fix):
//! [`get_step_icon`] maps the port's executed step display names
//! (`EXECUTED_PIPELINE`'s `name`) through `Step::vanilla` — the
//! `GenPassNameID` string keys — so the loading screen can look the table
//! up; the screen itself stays deferred.
#![allow(dead_code)] // dev-only screen (d86 scope-out); table parked for it

/// The three `PassIconEntry` factories as data (`UIWorldGenDebug.cs:184-232`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IconSpec {
    /// `FromBestiaryIcon(index)` (`:184-196`) — Icon_Tags_Shadow, 16×5 grid.
    Bestiary(i32),
    /// `FromItem(index)` (`:197-215`) — `Images/Item_<index>`, whole sheet.
    Item(i32),
    /// `FromImageFrame(image, index, rowCount, lineCount)` (`:216-232`).
    ImageFrame(&'static str, i32, i32, i32),
}

/// `InitPassIcons` (`:308-415`) in insertion order — `(GenPassNameID value,
/// spec)`. The names are the constants' string values
/// (`GenPassNameID.cs:223-330`), NOT the field identifiers ("Mount Caves",
/// "Shinies", "Guide"…). Rows cite their C# line.
pub const PASS_ICONS: &[(&str, IconSpec)] = &[
    ("Terrain", IconSpec::Bestiary(1)),            // :308
    ("Skyblock", IconSpec::Bestiary(26)),          // :309
    ("Dunes", IconSpec::Bestiary(4)),              // :310
    ("Ocean Sand", IconSpec::Bestiary(28)),        // :311
    ("Sand Patches", IconSpec::Item(169)),         // :312
    ("Tunnels", IconSpec::Item(4501)),             // :313
    ("Mount Caves", IconSpec::Item(4510)),         // :314
    ("Dirt Wall Backgrounds", IconSpec::Item(30)), // :315
    ("Rocks In Dirt", IconSpec::Item(3)),          // :316
    ("Dirt In Rocks", IconSpec::Item(2)),          // :317
    ("Clay", IconSpec::Item(133)),                 // :318
    ("Small Holes", IconSpec::Item(4538)),         // :319
    ("Dirt Layer Caves", IconSpec::Item(4510)),    // :320
    ("Rock Layer Caves", IconSpec::Item(4512)),    // :321
    ("Surface Caves", IconSpec::Item(4501)),       // :322
    ("Wavy Caves", IconSpec::Item(4537)),          // :323
    ("Generate Ice Biome", IconSpec::Bestiary(6)), // :324
    ("Grass", IconSpec::ImageFrame("Images/Tiles_3", 5, 45, 1)), // :325
    ("Jungle", IconSpec::Bestiary(22)),            // :326
    ("Mud Caves To Grass", IconSpec::Item(745)),   // :327
    ("Full Desert", IconSpec::Bestiary(3)),        // :328
    ("Mushroom Patches", IconSpec::Bestiary(24)),  // :329
    ("Marble", IconSpec::Bestiary(29)),            // :330
    ("Granite", IconSpec::Bestiary(30)),           // :331
    ("Floating Islands", IconSpec::Bestiary(26)),  // :332
    ("Dirt To Mud", IconSpec::Item(176)),          // :333
    ("Silt", IconSpec::Item(424)),                 // :334
    ("Shinies", IconSpec::Item(19)),               // :335
    ("Webs", IconSpec::Item(150)),                 // :336
    ("Underworld", IconSpec::Bestiary(33)),        // :337
    ("Corruption", IconSpec::Bestiary(7)),         // :338
    ("Lakes", IconSpec::Bestiary(28)),             // :339
    ("Slush", IconSpec::Item(1103)),               // :340
    ("Dual Dungeons Dither Snake", IconSpec::Bestiary(32)), // :341
    ("Dungeon", IconSpec::Bestiary(32)),           // :342
    ("Mountain Caves", IconSpec::Bestiary(2)),     // :343
    ("Beaches", IconSpec::Bestiary(27)),           // :344
    ("Gems", IconSpec::Item(178)),                 // :345
    ("Gravitating Sand", IconSpec::Item(169)),     // :346
    ("Create Ocean Caves", IconSpec::Bestiary(28)), // :347
    ("Shimmer", IconSpec::Item(5340)),             // :348
    ("Clean Up Dirt", IconSpec::Item(2)),          // :349
    ("Pyramids", IconSpec::Item(607)),             // :350
    ("Dirt Rock Wall Runner", IconSpec::Item(4501)), // :351
    ("Living Trees", IconSpec::Bestiary(0)),       // :352
    ("Wood Tree Walls", IconSpec::Item(1723)),     // :353
    ("Altars", IconSpec::Item(5467)),              // :354
    ("Wet Jungle", IconSpec::Bestiary(22)),        // :355
    ("Jungle Temple", IconSpec::Bestiary(31)),     // :356
    ("Hives", IconSpec::Item(1126)),               // :357
    ("Jungle Chests", IconSpec::Item(680)),        // :358
    ("Settle Liquids", IconSpec::Bestiary(28)),    // :359
    ("Remove Water From Sand", IconSpec::Item(169)), // :360
    ("Oasis", IconSpec::Bestiary(27)),             // :361
    ("Shell Piles", IconSpec::Item(4090)),         // :362
    ("Smooth World", IconSpec::Bestiary(1)),       // :363
    ("Waterfalls", IconSpec::Item(2169)),          // :364
    ("Ice", IconSpec::Item(664)),                  // :365
    ("Wall Variety", IconSpec::Item(4540)),        // :366
    ("Life Crystals", IconSpec::Item(29)),         // :367
    ("Statues", IconSpec::Item(52)),               // :368
    ("Buried Chests", IconSpec::Item(306)),        // :369
    ("Surface Chests", IconSpec::Item(48)),        // :370
    ("Jungle Chests Placement", IconSpec::Item(680)), // :371
    ("Water Chests", IconSpec::Item(1298)),        // :372
    ("Spider Caves", IconSpec::Bestiary(34)),      // :373
    ("Gem Caves", IconSpec::Item(4644)),           // :374
    ("Moss", IconSpec::Item(4496)),                // :375
    ("Temple", IconSpec::Bestiary(31)),            // :376
    ("Cave Walls", IconSpec::Item(4510)),          // :377
    ("Jungle Trees", IconSpec::Bestiary(23)),      // :378
    ("Floating Island Houses", IconSpec::Bestiary(26)), // :379
    ("Quick Cleanup", IconSpec::Bestiary(41)),     // :380
    ("Pots", IconSpec::Item(222)),                 // :381
    ("Hellforge", IconSpec::Item(221)),            // :382
    (
        "Spreading Grass",
        IconSpec::ImageFrame("Images/Tiles_3", 5, 45, 1),
    ), // :383
    ("Surface Ore and Stone", IconSpec::Item(19)), // :384
    ("Place Fallen Log", IconSpec::Bestiary(0)),   // :385
    ("Traps", IconSpec::Item(580)),                // :386
    ("Piles", IconSpec::Bestiary(1)),              // :387
    ("Spawn Point", IconSpec::Item(224)),          // :388
    ("Grass Wall", IconSpec::Item(745)),           // :389
    ("Guide", IconSpec::Item(867)),                // :390
    ("Sunflowers", IconSpec::Item(63)),            // :391
    ("Planting Trees", IconSpec::Bestiary(0)),     // :392
    ("Herbs", IconSpec::Item(3093)),               // :393
    ("Dye Plants", IconSpec::Item(1109)),          // :394
    ("Webs And Honey", IconSpec::Item(150)),       // :395
    ("Weeds", IconSpec::ImageFrame("Images/Tiles_3", 5, 45, 1)), // :396
    (
        "Glowing Mushrooms and Jungle Plants",
        IconSpec::Bestiary(25),
    ), // :397
    ("Jungle Plants", IconSpec::Bestiary(23)),     // :398
    ("Vines", IconSpec::Item(3005)),               // :399
    ("Flowers", IconSpec::ImageFrame("Images/Tiles_3", 33, 45, 1)), // :400
    ("Mushrooms", IconSpec::Item(5)),              // :401
    ("Gems In Ice Biome", IconSpec::Item(182)),    // :402
    ("Random Gems", IconSpec::Item(4400)),         // :403
    ("Moss Grass", IconSpec::Item(4496)),          // :404
    ("Muds Walls In Jungle", IconSpec::Item(4487)), // :405
    ("Larva", IconSpec::Item(2108)),               // :406
    ("Settle Liquids Again", IconSpec::Bestiary(28)), // :407
    ("Cactus, Palm Trees, & Coral", IconSpec::Bestiary(3)), // :408
    ("Tile Cleanup", IconSpec::Bestiary(41)),      // :409
    ("Lihzahrd Altars", IconSpec::Bestiary(31)),   // :410
    ("Micro Biomes", IconSpec::Bestiary(0)),       // :411
    ("Water Plants", IconSpec::Item(4564)),        // :412
    ("Stalac", IconSpec::Bestiary(6)),             // :413
    ("Remove Broken Traps", IconSpec::Item(580)),  // :414
    ("Final Cleanup", IconSpec::Bestiary(41)),     // :415
];

/// `GetPassIcon` (`:418-429`): the dictionary lookup by `pass.Name`; an
/// unmapped pass falls back to `FromBestiaryIcon(64)` (`:426`) — the 5th
/// row's first cell of Icon_Tags_Shadow (64 = col 0, row 4).
pub fn get_pass_icon(pass_name: &str) -> IconSpec {
    for (name, spec) in PASS_ICONS {
        if *name == pass_name {
            return *spec;
        }
    }
    IconSpec::Bestiary(64) // `:426`
}

/// The executed-step lookup — W4-f23 deviation fix: the port's worldgen
/// progress feed carries `EXECUTED_PIPELINE`'s display names
/// (`GenProgressHook::begin_step`, `tr_core::worldgen::pipeline`), NOT the
/// `GenPassNameID` strings this table keys on. `pipeline::pass_name_id`
/// maps display name → `Step::vanilla` (the `GenPassNameID` value,
/// `GenPassNameID.cs:223-330`); the icon then rides [`get_pass_icon`].
/// A step with no vanilla pass (the span planner, the world tail…) or an
/// unknown name takes the bestiary-64 fallback — the same arm an unmapped
/// `GenPass.Name` takes in the C# (`:426`).
pub fn get_step_icon(step: &str) -> IconSpec {
    match tr_core::worldgen::pipeline::pass_name_id(step) {
        Some(pass) => get_pass_icon(pass),
        None => IconSpec::Bestiary(64),
    }
}

#[cfg(test)]
mod tests {
    use super::super::worldgen_debug_elements::PassIconEntry;
    use super::*;

    /// The table is the full 108-row `InitPassIcons` (`:308-415`) with the
    /// GenPassNameID STRING values as keys, and the lookup falls back to
    /// bestiary icon 64 (`:426`).
    #[test]
    fn the_icon_table_is_complete_and_falls_back_to_64() {
        assert_eq!(PASS_ICONS.len(), 108);
        // No duplicate keys (the C# Dictionary.Add would throw).
        for (i, (a, _)) in PASS_ICONS.iter().enumerate() {
            assert!(!PASS_ICONS[..i].iter().any(|(b, _)| a == b), "dup key {a}");
        }
        assert_eq!(get_pass_icon("Terrain"), IconSpec::Bestiary(1));
        assert_eq!(
            get_pass_icon("Mount Caves"),
            IconSpec::Item(4510),
            "the string value, not MountainCaves"
        );
        assert_eq!(
            get_pass_icon("Mountain Caves"),
            IconSpec::Bestiary(2),
            "a DIFFERENT pass (:343)"
        );
        assert_eq!(get_pass_icon("Guide"), IconSpec::Item(867));
        assert_eq!(
            get_pass_icon("Grass"),
            IconSpec::ImageFrame("Images/Tiles_3", 5, 45, 1)
        );
        assert_eq!(
            get_pass_icon("Flowers"),
            IconSpec::ImageFrame("Images/Tiles_3", 33, 45, 1)
        );
        assert_eq!(get_pass_icon("Final Cleanup"), IconSpec::Bestiary(41));
        assert_eq!(
            get_pass_icon("Not A Pass"),
            IconSpec::Bestiary(64),
            "the fallback"
        );
        // The fallback frames to row 4, col 0 of the 16×5 sheet.
        let e = PassIconEntry::from_bestiary_icon(64, [416, 130]);
        assert_eq!(e.region, [0, 104, 26, 26]);
    }

    /// W4-f23 deviation fix: the executed-step lookup maps the port's
    /// display names through `Step::vanilla` (the `GenPassNameID` keys).
    /// Every executed step with a vanilla pass must resolve to a REAL
    /// table row (never the fallback) — the mapping is total over the
    /// pipeline's named passes.
    #[test]
    fn the_step_lookup_maps_display_names_to_pass_keys() {
        // The display names are NOT the keys (`EXECUTED_PIPELINE`).
        assert_eq!(get_step_icon("terrain fill"), IconSpec::Bestiary(1));
        assert_eq!(get_step_icon("cave fields"), IconSpec::Item(4501), "Tunnels");
        assert_eq!(get_step_icon("ores and gems"), IconSpec::Item(19), "Shinies");
        assert_eq!(get_step_icon("evil"), IconSpec::Bestiary(7), "Corruption");
        assert_eq!(get_step_icon("liquids"), IconSpec::Bestiary(28), "Lakes");
        assert_eq!(
            get_step_icon("marble"),
            IconSpec::Bestiary(29),
            "the Marble slot (:330) — the step's vanilla key"
        );
        assert_eq!(get_step_icon("granite"), IconSpec::Bestiary(30));
        assert_eq!(get_step_icon("slopes"), IconSpec::Bestiary(1), "Smooth World");
        assert_eq!(get_step_icon("trees"), IconSpec::Bestiary(0), "Planting Trees");
        assert_eq!(
            get_step_icon("surface flora"),
            IconSpec::ImageFrame("Images/Tiles_3", 5, 45, 1),
            "Weeds (:396)"
        );
        assert_eq!(get_step_icon("moss"), IconSpec::Item(4496));
        assert_eq!(get_step_icon("micro grass"), get_pass_icon("Grass"));
        // A step with NO vanilla pass ("" — the span planner, the world
        // tail, the seed finish) takes the bestiary-64 fallback, as does
        // an unknown name (`GetPassIcon` `:426`).
        assert_eq!(get_step_icon("plan biome spans"), IconSpec::Bestiary(64));
        assert_eq!(get_step_icon("world tail"), IconSpec::Bestiary(64));
        assert_eq!(get_step_icon("seed finish"), IconSpec::Bestiary(64));
        assert_eq!(get_step_icon("not a step"), IconSpec::Bestiary(64));
        // Totality: every executed step with a vanilla pass resolves to a
        // real PASS_ICONS row - no named step falls back. Driven by the
        // pipeline itself so a split/rename cannot leave a stale literal
        // (the `granite and marble` split into `marble`/`granite` did).
        for s in tr_core::worldgen::pipeline::executed_steps(Default::default()) {
            if s.vanilla.is_empty() {
                continue; // a port-only step ("" vanilla) takes the fallback
            }
            let pass = s.vanilla;
            assert!(
                PASS_ICONS.iter().any(|(k, _)| *k == pass),
                "'{}' -> '{pass}' has no icon row",
                s.name
            );
            assert_eq!(get_step_icon(s.name), get_pass_icon(pass), "{}", s.name);
        }
    }
}
