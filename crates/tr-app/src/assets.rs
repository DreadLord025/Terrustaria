//! Terraria asset loading: real game textures extracted from `.xnb` to PNG.

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use anyhow::{Context, Result};
use rayon::prelude::*;
use tr_core::World;
use tr_render::{AtlasImage, SheetLayout};

pub fn usage(world: &World) -> (BTreeSet<u16>, BTreeSet<u16>) {
    let mut tiles = BTreeSet::new();
    let mut walls = BTreeSet::new();
    for t in &world.tiles {
        // Tile id 0 is dirt, not air; only the active flag distinguishes air.
        if t.active() {
            tiles.insert(t.kind);
        }
        if t.wall != 0 {
            walls.insert(t.wall);
        }
    }
    (tiles, walls)
}

pub fn image_name_for_tile(kind: u16) -> String {
    format!("Tiles_{kind}")
}

pub fn image_name_for_wall(wall: u16) -> String {
    format!("Wall_{wall}")
}

/// Every `Item_<id>.xnb` present in the content `Images` dir, sorted ascending
/// and deduped so the atlas layout is reproducible. Non-item files (`Tiles_*`)
/// and non-numeric suffixes are ignored; a missing dir yields an empty list.
fn enumerate_item_icon_ids(images_dir: &Path) -> Vec<u16> {
    let Ok(entries) = std::fs::read_dir(images_dir) else {
        return Vec::new();
    };
    let mut ids: Vec<u16> = entries
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name();
            let name = name.to_str()?;
            let id = name.strip_prefix("Item_")?.strip_suffix(".xnb")?;
            id.parse::<u16>().ok()
        })
        .collect();
    ids.sort_unstable();
    ids.dedup();
    ids
}

/// Loads only the sheets referenced by the world (plus a dirt fallback).
pub fn load_world_images(
    dir: &Path,
    tiles: &BTreeSet<u16>,
    walls: &BTreeSet<u16>,
) -> Result<Vec<AtlasImage>> {
    let mut images = Vec::new();
    let mut missing = Vec::new();

    // C5: decode sheets on Rayon's pool. `par_iter().collect()` keeps iterator
    // order, so the atlas layout (and every UV) stays bit-identical to the
    // sequential loader.
    let tile_images: Vec<(u16, Result<AtlasImage>)> = tiles
        .par_iter()
        .map(|&kind| {
            (
                kind,
                load_png(dir, &image_name_for_tile(kind), SheetLayout::Tile),
            )
        })
        .collect();
    for (kind, result) in tile_images {
        match result {
            Ok(img) => images.push(img),
            Err(_) => missing.push(image_name_for_tile(kind)),
        }
    }

    let wall_images: Vec<(u16, Result<AtlasImage>)> = walls
        .par_iter()
        .map(|&wall| {
            (
                wall,
                load_png(dir, &image_name_for_wall(wall), SheetLayout::Wall),
            )
        })
        .collect();
    for (wall, result) in wall_images {
        match result {
            Ok(img) => images.push(img),
            Err(_) => missing.push(image_name_for_wall(wall)),
        }
    }

    // Fallback sheet so unknown ids still render something instead of vanishing.
    if !images.iter().any(|i| i.name == "Tiles_0") {
        if let Ok(img) = load_png(dir, "Tiles_0", SheetLayout::Tile) {
            images.push(img);
        }
    }

    // Whole-image sheets used by the sky/background pass.
    for name in [
        "Background_0",
        // Underground: `Background_1` is the surface transition row, `Background_2`
        // the tiled dirt backdrop (`_drawBackground_backTexture[0]/[1]`).
        "Background_1",
        "Background_2",
        // Rock layer / hell (`DrawBackground_DrawRockLayer`/`_DrawMagmaLayer`):
        // `Background_3` rock bands, `Background_4` the rock-transition row the dirt
        // sheet ends with, `Background_6` the magma transition band
        // (`_drawBackground_backTexture[2]/[3]/[4]`); `125+hellBackStyle` is the
        // animated magma field (three 96 px frames) and `185+hellBackStyle` the hell
        // floor row (`Main.cs:54000-54008`). `Black_Tile` fills the underworld box.
        "Background_3",
        "Background_4",
        "Background_6",
        "Background_125",
        "Background_126",
        "Background_127",
        "Background_185",
        "Background_186",
        "Background_187",
        "Black_Tile",
        // Surface-background layer sets: forest mountains 7/8, snow mountains 35/36,
        // forest tree lines 50..52 and snow tree lines 37..39.
        "Background_7",
        "Background_8",
        "Background_35",
        "Background_36",
        "Background_37",
        "Background_38",
        "Background_39",
        "Background_50",
        "Background_51",
        "Background_52",
        "Background_28",
        "Background_21",
        "Background_20",
        // Desert mountain layers behind the dunes (`DrawSurfaceBG_BackMountainsStep1/2`).
        "Background_23",
        "Background_22",
        "Background_112",
        "Background_113",
        "Sun",
        "Moon_0",
        "Moon_1",
        "Moon_2",
        "Moon_3",
        "Moon_4",
        "Moon_5",
        "Moon_6",
        "Moon_7",
        "Moon_8",
        "Star_0",
        "Star_1",
        "Star_2",
        "Star_3",
        // The ambient-sky creature sheets (`Images/Backgrounds/Ambience/*`, the
        // textures `Main.Assets.Request`ed by the `AmbientSky` entity classes).
        // `BoneSerpant.xnb` is skipped: `AmbientSky.Spawn` has no BoneSerpent
        // case and the entity class never sets a texture (`AmbientSky.cs:1054`).
        "Backgrounds/Ambience/ButterflySwarm1",
        "Backgrounds/Ambience/ButterflySwarm2",
        "Backgrounds/Ambience/LostKite",
        "Backgrounds/Ambience/Pegasus",
        "Backgrounds/Ambience/Vulture",
        "Backgrounds/Ambience/PixiePosse1",
        "Backgrounds/Ambience/PixiePosse2",
        "Backgrounds/Ambience/BirdsVShape",
        "Backgrounds/Ambience/Seagull",
        "Backgrounds/Ambience/Gastropod",
        "Backgrounds/Ambience/SlimeBalloons",
        "Backgrounds/Ambience/HellBat1",
        "Backgrounds/Ambience/HellBat2",
        "Backgrounds/Ambience/Bat1",
        "Backgrounds/Ambience/Bat2",
        "Backgrounds/Ambience/Bat3",
        "Backgrounds/Ambience/Wyvern",
        "Backgrounds/Ambience/FlyingShip",
        "Backgrounds/Ambience/AirBalloons_Large",
        "Backgrounds/Ambience/AirBalloons_Small",
        "Backgrounds/Ambience/EOC",
        "Backgrounds/Ambience/EOS1",
        "Backgrounds/Ambience/EOS2",
        "Backgrounds/Ambience/EOS3",
        "Backgrounds/Ambience/Crimera1",
        "Backgrounds/Ambience/Crimera2",
        "Backgrounds/Ambience/Crimera3",
        "Backgrounds/Ambience/Meteor",
        // The ambient leaf gores the generated world's trees can shed
        // (`WorldGen.GetTreeLeaf`: forest grass 910, palm sand 911).
        "Gore_910",
        "Gore_911",
        "Player_0_0",
        "Player_0_1",
        "Player_0_2",
        "Player_0_3",
        "Player_0_6",
        "Player_0_7",
        "Player_0_10",
        "Player_0_11",
        "Player_0_12",
        "Player_Hair_1",
        // The mount sheets of the M2-MOUNT-1 ground set (`MountData`
        // back/front textures, `Mount.cs:696-2567`): the named `Mount_*`
        // sheets plus the `Extra_*` files the C# binds through
        // `TextureAssets.Extra[...]` (horses 161-163, pogo 164, WoF goat
        // 169, Queen Slime 204) and Rudolph's three-part strip.
        "Rudolph_0",
        "Rudolph_1",
        "Rudolph_2",
        "Mount_Bunny",
        "Mount_Slime",
        "Mount_Scutlix",
        "Mount_ScutlixEyes",
        "Mount_ScutlixEyeGlow",
        "Mount_Unicorn",
        "Mount_Basilisk",
        "Extra_161",
        "Extra_162",
        "Extra_163",
        "Extra_164",
        "Extra_169",
        "Extra_204",
        // Armor sheets, slots 1..4 = copper/iron/silver/gold. `AssetInitializer.cs:
        // 165-185` indexes the texture arrays by the armor slot itself, so the file
        // number equals the slot: `Armor_Head_{n}`, `Armor_Legs_{n}` and the
        // 1.4.4+ body composite `Armor/Armor_{n}` (the torso + shoulders + arms
        // grid: torso (0,0)/(1,0), shoulders (0,1)/(1,1), arms cols 2-6, front
        // arm col 7, back arm col 8 — `PlayerDrawSet.cs:2241`).
        "Armor_Head_1",
        "Armor_Head_2",
        "Armor_Head_3",
        "Armor_Head_4",
        "Armor_Legs_1",
        "Armor_Legs_2",
        "Armor_Legs_3",
        "Armor_Legs_4",
        "Armor/Armor_1",
        "Armor/Armor_2",
        "Armor/Armor_3",
        "Armor/Armor_4",
        "NPC_1",
        "NPC_22",
        "NPC_21",
        "NPC_44",
        "NPC_49",
        "NPC_6",
        "NPC_42",
        "NPC_23",
        "NPC_29",
        "NPC_45",
        "NPC_32",
        "NPC_24",
        "NPC_30",
        "NPC_25",
        "NPC_33",
        "NPC_55",
        "NPC_58",
        "NPC_65",
        "NPC_7",
        "NPC_17",
        "NPC_20",
        "NPC_18",
        "NPC_19",
        "NPC_38",
        "NPC_227",
        "NPC_228",
        "NPC_54",
        "NPC_4",
        "NPC_222",
        "NPC_35",
        "NPC_113",
        "NPC_134",
        "NPC_127",
        "NPC_125",
        "NPC_126",
        "NPC_262",
        "NPC_266",
        "NPC_267",
        "NPC_26",
        "NPC_27",
        "NPC_28",
        "NPC_111",
        "NPC_245",
        "NPC_246",
        "NPC_247",
        "NPC_249",
        "NPC_105",
        "NPC_107",
        "NPC_8",
        "NPC_9",
        "NPC_10",
        "NPC_11",
        "NPC_12",
        "NPC_13",
        "NPC_14",
        "NPC_15",
        "NPC_51",
        "NPC_60",
        "NPC_66",
        "NPC_2",
        "NPC_3",
        "NPC_50",
        "Rain",
        "Dust",
        "Projectile_3",
        "Logo",
        "Inventory_Back",
        "Inventory_Back2",
        "Inventory_Back5",
        "Inventory_Back9",
        "Inventory_Back14",
        "Heart",
        "Bubble",
        "NPC_Head_Boss_7",
        "UI/UI_BossBar",
        "UI/Cursor_0",
        "UI/Cursor_1",
        "Mana",
        "Inventory_Back13",
        // P2 inventory panels: coins/ammo/misc/trash backgrounds, page tabs and
        // the PVP icon (`Main.DrawInventory`, `ItemSlot.Draw`).
        "Inventory_Back3",
        "Inventory_Back4",
        "Inventory_Back6",
        "Inventory_Back7",
        "Inventory_Back12",
        "Trash",
        "Extra_54",
        "Extra_58",
        "UI/DisplaySlots_0",
        "UI/DisplaySlots_1",
        "UI/DisplaySlots_2",
        "UI/DisplaySlots_3",
        "UI/DisplaySlots_4",
        "UI/DisplaySlots_5",
        "UI/DisplaySlots_6",
        "UI/DisplaySlots_7",
        "UI/DisplaySlots_8",
        "UI/DisplaySlots_9",
        "UI/DisplaySlots_10",
        "UI/PVP_0",
        "UI/PVP_1",
        // P3 modern crafting panel (`NewCraftingUI`): the `Inventory_Back18` craft
        // slot, the cyan panel 9-slice, the separator, the scrollbar, the craft
        // icon/toggles and the filter strip's icons.
        "Inventory_Back18",
        "UI/PanelBackground",
        "UI/PanelBorder",
        "UI/CharCreation/Separator1",
        "UI/Scrollbar2",
        "UI/ScrollbarInner",
        "UI/Craft",
        "UI/Craft_Toggle_0",
        "UI/Craft_Toggle_1",
        "UI/Craft_Toggle_2",
        "UI/Craft_Toggle_3",
        "UI/Craft_Toggle_4",
        "UI/Craft_Toggle_5",
        "UI/ChestCraft_0",
        "UI/ChestCraft_1",
        "UI/ChestCraft_2",
        "UI/ChestCraft_3",
        "UI/Creative/Infinite_Icons",
        "UI/Creative/Infinite_Tabs_B_2",
        "Text_Back",
        "Extra_48",
        "UI/Bestiary",
        "UI/Bestiary/Slot_Back",
        "UI/Bestiary/Slot_Front",
        "UI/Bestiary/Slot_Overlay",
        "UI/Bestiary/Slot_Selection",
        "UI/Bestiary/Icon_Locked",
        "UI/Bestiary/Portrait_Front",
        "UI/Bestiary/Button_Back",
        "UI/Bestiary/Stat_HP",
        "UI/Bestiary/Stat_Attack",
        "UI/Bestiary/Stat_Defense",
        "UI/Bestiary/Stat_Knockback",
        "UI/Bestiary/Stat_Copper",
        "UI/Bestiary/Button_Search_2",
        "UI/Bestiary/Button_Search_Border",
        "UI/SearchCancel",
        // P4 minimap chrome (`MinimapFrameTemplate.cs:30-35`) — 9 styles x 4 sheets.
        "UI/Minimap/Default/MinimapFrame",
        "UI/Minimap/Default/MinimapButton_Reset",
        "UI/Minimap/Default/MinimapButton_ZoomIn",
        "UI/Minimap/Default/MinimapButton_ZoomOut",
        "UI/Minimap/Golden/MinimapFrame",
        "UI/Minimap/Golden/MinimapButton_Reset",
        "UI/Minimap/Golden/MinimapButton_ZoomIn",
        "UI/Minimap/Golden/MinimapButton_ZoomOut",
        "UI/Minimap/Remix/MinimapFrame",
        "UI/Minimap/Remix/MinimapButton_Reset",
        "UI/Minimap/Remix/MinimapButton_ZoomIn",
        "UI/Minimap/Remix/MinimapButton_ZoomOut",
        "UI/Minimap/Sticks/MinimapFrame",
        "UI/Minimap/Sticks/MinimapButton_Reset",
        "UI/Minimap/Sticks/MinimapButton_ZoomIn",
        "UI/Minimap/Sticks/MinimapButton_ZoomOut",
        "UI/Minimap/StoneGold/MinimapFrame",
        "UI/Minimap/StoneGold/MinimapButton_Reset",
        "UI/Minimap/StoneGold/MinimapButton_ZoomIn",
        "UI/Minimap/StoneGold/MinimapButton_ZoomOut",
        "UI/Minimap/TwigLeaf/MinimapFrame",
        "UI/Minimap/TwigLeaf/MinimapButton_Reset",
        "UI/Minimap/TwigLeaf/MinimapButton_ZoomIn",
        "UI/Minimap/TwigLeaf/MinimapButton_ZoomOut",
        "UI/Minimap/Leaf/MinimapFrame",
        "UI/Minimap/Leaf/MinimapButton_Reset",
        "UI/Minimap/Leaf/MinimapButton_ZoomIn",
        "UI/Minimap/Leaf/MinimapButton_ZoomOut",
        "UI/Minimap/Retro/MinimapFrame",
        "UI/Minimap/Retro/MinimapButton_Reset",
        "UI/Minimap/Retro/MinimapButton_ZoomIn",
        "UI/Minimap/Retro/MinimapButton_ZoomOut",
        "UI/Minimap/Valkyrie/MinimapFrame",
        "UI/Minimap/Valkyrie/MinimapButton_Reset",
        "UI/Minimap/Valkyrie/MinimapButton_ZoomIn",
        "UI/Minimap/Valkyrie/MinimapButton_ZoomOut",
        // P4 map-style icon row (`AssetInitializer.cs:386-389`).
        "Map_0",
        "Map_1",
        "Map_2",
        "Map_3",
        "Map_4",
        "Map_5",
        "Map_6",
        "Map_7",
        // The wooden scroll frame behind the fullscreen map (`TextureAssets.Map`,
        // 928x248; `Main.cs:55434-55485`).
        "Map",
        // The biome backdrops behind the fullscreen map: `TextureAssets.MapBGs[i]
        // = "Images/MapBG" + (i + 1)` (`AssetInitializer.cs:341-343`). Only the
        // indices `map_bg_index` can select are listed; the unreachable `false`
        // guards (dungeon, glowshroom, corrupt/crimson/hallow/jungle, bloodMoon,
        // graveyard) can never pick theirs.
        "MapBG1",  // 0  surface default (forest)
        "MapBG2",  // 1  underground forest
        "MapBG3",  // 2  deepest layer
        "MapBG4",  // 3  snow (underground)
        "MapBG10", // 9  desert (surface)
        "MapBG11", // 10 beach
        "MapBG12", // 11 snow (surface)
        "MapBG14", // 13 Lihzahrd brick wall (WallID 87)
        "MapBG15", // 14 desert (underground)
        "MapBG16", // 15 Hive wall (WallID 86/108)
        "MapBG17", // 16 Granite wall (WallID 180/184)
        "MapBG18", // 17 Marble wall (WallID 178/183)
        "MapBG19", // 18 Spider wall (WallID 62/263)
        "MapBG32", // 31 rock layer
        "MapBG33", // 32 space
        // P5 settings sliders (`AssetInitializer.cs:345-349`): the
        // `IngameOptions.DrawValueBar` chrome and the `Main.DrawMenu` HSL colour
        // sliders. `ColorHighlight` is `Images/UI/Slider_Highlight`, NOT
        // `Images/ColorHighlight` (`AssetInitializer.cs:349`).
        "Hue",
        "ColorBar",
        "ColorBlip",
        "ColorSlider",
        "UI/Slider_Highlight",
        // P6 front-end (`UICharacterListItem.cs:52-60`, `UIWorldListItem.cs:210-221`,
        // `UICharacter.cs:45`, `AWorldListItem.cs:199`, `UICharacterCreation.cs`,
        // `UIWorldCreation.cs`, `UIVirtualKeyboard.cs:171-172`). Includes
        // `PreviewDifficultyExpert2` — the fourth bunny slot the Lookups table
        // carries (the plan's whitelist list omits it; `UIWorldCreationPreview.cs:53`).
        "UI/ButtonPlay",
        "UI/ButtonRename",
        "UI/ButtonDelete",
        "UI/ButtonSeed",
        "UI/ButtonFavoriteActive",
        "UI/ButtonFavoriteInactive",
        "UI/ButtonCloudActive",
        "UI/ButtonCloudInactive",
        "UI/Divider",
        "UI/InnerPanelBackground",
        "UI/PlayerBackground",
        "UI/IconCrimson",
        "UI/IconCompletion",
        "UI/IconMixedSeed",
        "UI/IconPlayedBefore",
        "UI/IconNewlyGenerated",
        "UI/CharCreation/CategoryPanel",
        "UI/CharCreation/CategoryPanelBorder",
        "UI/CharCreation/CategoryPanelHighlight",
        "UI/CharCreation/PanelGrayscale",
        "UI/CharCreation/SmallPanel",
        "UI/CharCreation/SmallPanelBorder",
        "UI/CharCreation/CharInfo",
        "UI/CharCreation/HairStyle_Hair",
        "UI/CharCreation/HairStyle_Arrow",
        "UI/CharCreation/ColorHair",
        "UI/CharCreation/ColorEye",
        "UI/CharCreation/ColorEyeBack",
        "UI/CharCreation/ColorSkin",
        "UI/CharCreation/ColorShirt",
        "UI/CharCreation/ColorUndershirt",
        "UI/CharCreation/ColorPants",
        "UI/CharCreation/ColorShoes",
        "UI/CharCreation/Copy",
        "UI/CharCreation/Paste",
        "UI/CharCreation/Randomize",
        "UI/CharCreation/ClothStyleMale",
        "UI/CharCreation/ClothStyleFemale",
        "UI/CharCreation/Voice",
        "UI/CharCreation/ColorCharacter",
        "UI/WorldCreation/PreviewBorder",
        "UI/WorldCreation/IconRandomName",
        "UI/WorldCreation/IconRandomSeed",
        "UI/WorldCreation/IconSizeSmall",
        "UI/WorldCreation/IconSizeMedium",
        "UI/WorldCreation/IconSizeLarge",
        "UI/WorldCreation/IconDifficultyCreative",
        "UI/WorldCreation/IconDifficultyNormal",
        "UI/WorldCreation/IconDifficultyExpert",
        "UI/WorldCreation/IconDifficultyMaster",
        "UI/WorldCreation/IconEvilRandom",
        "UI/WorldCreation/IconEvilCorruption",
        "UI/WorldCreation/IconEvilCrimson",
        "UI/WorldCreation/PreviewDifficultyNormal1",
        "UI/WorldCreation/PreviewDifficultyExpert1",
        "UI/WorldCreation/PreviewDifficultyMaster1",
        "UI/WorldCreation/PreviewDifficultyNormal2",
        "UI/WorldCreation/PreviewDifficultyCreative2",
        "UI/WorldCreation/PreviewDifficultyExpert2",
        "UI/WorldCreation/PreviewDifficultyMaster2",
        "UI/WorldCreation/PreviewEvilRandom",
        "UI/WorldCreation/PreviewEvilCorruption",
        "UI/WorldCreation/PreviewEvilCrimson",
        "UI/WorldCreation/PreviewSizeSmall",
        "UI/WorldCreation/PreviewSizeMedium",
        "UI/WorldCreation/PreviewSizeLarge",
        "UI/VK_Shift",
        "UI/VK_Backspace",
    ] {
        match load_png(dir, name, SheetLayout::Raw) {
            Ok(img) => images.push(img),
            Err(_) => log::warn!("missing background texture {name}"),
        }
    }

    // Every item icon the content carries: `AssetInitializer.cs:111-125` fills
    // `TextureAssets.Item[i]` with `Images/Item_<i>` for the whole `ItemID`
    // range, so the loader enumerates the files instead of a hand-list — an id
    // whose file is absent simply never registers and its draw code falls
    // through the coin/wall/tile fallbacks (the `item.rs` drop draw). Sorted
    // ids keep the atlas layout reproducible: `Atlas::build`'s stable shelf
    // sort breaks size ties by input order. C5: decoded on Rayon's pool,
    // `par_iter().collect()` keeps iterator order.
    let item_ids = enumerate_item_icon_ids(dir);
    let item_images: Vec<(u16, Result<AtlasImage>)> = item_ids
        .par_iter()
        .map(|&id| (id, load_png(dir, &format!("Item_{id}"), SheetLayout::Raw)))
        .collect();
    let mut failed_items = Vec::new();
    for (id, result) in item_images {
        match result {
            Ok(img) => images.push(img),
            Err(_) => failed_items.push(id),
        }
    }
    if !failed_items.is_empty() {
        log::warn!(
            "{} item icons failed to decode (first 10: {:?})",
            failed_items.len(),
            &failed_items[..failed_items.len().min(10)]
        );
    }

    // Tree canopies and branches are separate free-form textures (`Tree_Tops_*`,
    // `Tree_Branches_*`); the trunk itself lives in `Tiles_5.png`.
    for style in 0..32 {
        for prefix in ["Tree_Tops_", "Tree_Branches_"] {
            let name = format!("{prefix}{style}");
            if let Ok(img) = load_png(dir, &name, SheetLayout::Raw) {
                images.push(img);
            }
        }
    }

    // Sky clouds (`Cloud_0`..`Cloud_41`), drawn as a parallax layer.
    for i in 0..42 {
        let name = format!("Cloud_{i}");
        if let Ok(img) = load_png(dir, &name, SheetLayout::Raw) {
            images.push(img);
        }
    }

    // Liquid surfaces: `Images/Misc/water_0..14` (water styles, lava, honey,
    // shimmer). 48x1360 each: 3 columns of 16 px and 17 animation blocks of 80 px.
    for i in 0..15 {
        let name = format!("water_{i}");
        if let Ok(img) = load_misc_png(dir, &name) {
            images.push(img);
        }
    }

    // `DrawTile_LiquidBehindTile`: `Liquid_*` is the 16 px partial strip and
    // `LiquidSlope_*` the four diagonal masks drawn behind blocks that stand in
    // liquid.
    for i in 0..15 {
        for prefix in ["Liquid_", "LiquidSlope_"] {
            let name = format!("{prefix}{i}");
            if let Ok(img) = load_png(dir, &name, SheetLayout::Raw) {
                images.push(img);
            }
        }
    }

    // `WaterfallManager.waterfallTexture[0..27]`: the common sheet is 512x40
    // (sixteen 32x40 frames), the rain/dust variants are 144x18 (eight 18x18
    // frames) and the shimmer/disco companion sheets are 512x124.
    for i in 0..tr_core::waterfall::MAX_TYPES {
        let name = format!("Waterfall_{i}");
        if let Ok(img) = load_png(dir, &name, SheetLayout::Raw) {
            images.push(img);
        }
    }

    // `TextureAssets.Flames` (`AssetInitializer.cs:382-384`): the tile-flame
    // overlays `DrawSingleTile_Flames` draws over torches and furniture.
    for i in 0..18u16 {
        let name = format!("Flame_{i}");
        match load_png(dir, &name, SheetLayout::Raw) {
            Ok(img) => images.push(img),
            Err(_) => log::warn!("missing flame sheet {name}"),
        }
    }

    // `DrawWires`'s overlays (`Main.cs:58218+`): `TextureAssets.WireNew` is
    // the four-colour wire sheet (16 px mask columns × the 18 px colour rows)
    // and `TextureAssets.Actuator` the actuator plate drawn on every
    // actuator-carrying tile.
    for name in ["WiresNew", "Actuator"] {
        if let Ok(img) = load_png(dir, name, SheetLayout::Raw) {
            images.push(img);
        }
    }

    // The tile-side glow sheets of `DrawSingleTile_Flames`
    // (`TileDrawing.cs:2684-2825`, `:3463-3491`): `TextureAssets.GlowMask`
    // loads as `Images/Glow_{id}` (`AssetInitializer.cs:352-354`) and the
    // named single sheets keep their own names.
    for name in GLOW_SHEET_NAMES {
        match load_png(dir, name, SheetLayout::Raw) {
            Ok(img) => images.push(img),
            Err(_) => log::warn!("missing glow sheet {name}"),
        }
    }

    if !missing.is_empty() {
        log::warn!(
            "{} world sheets have no matching texture (first 10: {:?})",
            missing.len(),
            &missing[..missing.len().min(10)]
        );
    }

    Ok(images)
}

/// `Content` directory bundled with the project (the retail `.xnb` files copied into
/// `assets/Content`), set once at startup by `main`. The game reads them at runtime with
/// the original XNB readers; there is no pre-extracted fallback.
pub fn set_content_dir(dir: PathBuf) {
    let _ = IMAGES_DIR.set(dir.join("Images"));
    let _ = CONTENT_DIR.set(dir);
}

static CONTENT_DIR: OnceLock<PathBuf> = OnceLock::new();
static IMAGES_DIR: OnceLock<PathBuf> = OnceLock::new();

/// The bundled `Content` directory, if it was found at startup.
pub fn content_dir() -> Option<&'static Path> {
    CONTENT_DIR.get().map(|p| p.as_path())
}

fn content_images_dir() -> Option<&'static Path> {
    IMAGES_DIR.get().map(|p| p.as_path()).filter(|p| p.is_dir())
}

/// Decodes one sheet straight from the game's `.xnb` (the original pipeline).
fn load_sheet(name: &str, layout: SheetLayout) -> Result<AtlasImage> {
    let dir = content_images_dir().context("Content/Images not found")?;
    let path = dir.join(format!("{name}.xnb"));
    let img =
        tr_content::load_texture(&path).with_context(|| format!("decoding {}", path.display()))?;
    Ok(AtlasImage {
        name: name.to_string(),
        width: img.width,
        height: img.height,
        rgba: img.pixels,
        layout,
    })
}

pub fn load_png(_dir: &Path, name: &str, layout: SheetLayout) -> Result<AtlasImage> {
    load_sheet(name, layout)
}

fn load_misc_png(_dir: &Path, name: &str) -> Result<AtlasImage> {
    load_sheet(&format!("Misc/{name}"), SheetLayout::Raw)
}

/// One glyph of a ReLogic `DynamicSpriteFont`, in screen pixels.
#[derive(Clone, Copy, Debug)]
pub struct FontGlyph {
    pub uv: [f32; 4],
    pub w: f32,
    pub h: f32,
    /// `SpriteCharacterData.Padding`: draw offset inside the line box.
    pub pad: [f32; 2],
    /// `Kerning`: (left bearing, advance - z, z).
    pub kerning: [f32; 3],
}

/// A decoded `DynamicSpriteFont` bound to the atlas (`DynamicSpriteFont.DrawString`
/// walks these metrics exactly).
#[derive(Clone, Debug, Default)]
pub struct UiFont {
    pub glyphs: HashMap<char, FontGlyph>,
    pub line_spacing: f32,
    pub spacing: f32,
    pub fallback: char,
}

impl UiFont {
    pub fn glyph(&self, c: char) -> Option<&FontGlyph> {
        self.glyphs
            .get(&c)
            .or_else(|| self.glyphs.get(&self.fallback))
    }
}

/// Decodes `Fonts/Mouse_Text`, `Fonts/Item_Stack` and `Fonts/Death_Text` from the retail
/// content and returns their page textures as atlas images plus the raw metrics.
pub fn load_fonts() -> (
    Vec<AtlasImage>,
    Option<(String, tr_content::DynamicSpriteFont)>,
    Option<(String, tr_content::DynamicSpriteFont)>,
    Option<(String, tr_content::DynamicSpriteFont)>,
) {
    let mut images = Vec::new();
    let Some(images_dir) = content_images_dir() else {
        return (images, None, None, None);
    };
    let fonts_dir = images_dir.parent().map(|p| p.join("Fonts"));
    let mut ui = None;
    let mut item = None;
    let mut death = None;
    if let Some(dir) = fonts_dir {
        for (name, slot) in [("Mouse_Text", 0), ("Item_Stack", 1), ("Death_Text", 2)] {
            let path = dir.join(format!("{name}.xnb"));
            match tr_content::load_font(&path) {
                Ok(font) => {
                    let sheet = format!("Font_{name}");
                    for (page, pg) in font.pages.iter().enumerate() {
                        images.push(AtlasImage {
                            name: format!("{sheet}_{page}"),
                            width: pg.texture.width,
                            height: pg.texture.height,
                            rgba: pg.texture.pixels.clone(),
                            layout: SheetLayout::Raw,
                        });
                    }
                    log::info!(
                        "font {name}: {} glyphs, line spacing {}",
                        font.pages.first().map_or(0, |p| p.glyphs.len()),
                        font.line_spacing
                    );
                    match slot {
                        0 => ui = Some((sheet, font)),
                        1 => item = Some((sheet, font)),
                        _ => death = Some((sheet, font)),
                    }
                }
                Err(e) => log::warn!("font {name}: {e:#}"),
            }
        }
    }
    (images, ui, item, death)
}

/// Binds a decoded font to its atlas page UVs.
pub fn build_ui_font(
    atlas: &tr_render::Atlas,
    sheet: &str,
    font: &tr_content::DynamicSpriteFont,
) -> Option<UiFont> {
    let mut out = UiFont {
        line_spacing: font.line_spacing as f32,
        spacing: font.spacing,
        fallback: font.default_char,
        glyphs: HashMap::new(),
    };
    for (page, pg) in font.pages.iter().enumerate() {
        let rect = atlas.sheet(&format!("{sheet}_{page}"))?;
        for g in &pg.glyphs {
            let [sx, sy, w, h] = g.src;
            out.glyphs.insert(
                g.ch,
                FontGlyph {
                    uv: uv_rect(
                        rect,
                        (atlas.width, atlas.height),
                        sx,
                        sy,
                        w as u32,
                        h as u32,
                    ),
                    w: w as f32,
                    h: h as f32,
                    pad: [g.pad[0] as f32, g.pad[1] as f32],
                    kerning: g.kerning,
                },
            );
        }
    }
    Some(out)
}

/// One resolved frame: atlas UV plus whether it fully covers its 16x16 cell.
#[derive(Clone, Copy, Debug)]
pub struct FrameTex {
    pub uv: [f32; 4],
    pub opaque: bool,
}

/// The named glow sheets `DrawSingleTile_Flames` draws besides
/// `TextureAssets.GlowMask` (`AssetInitializer.cs:312-331`, `:576-577`).
const GLOW_SHEET_NAMES: &[&str] = &[
    // `TextureAssets.GlowMask` ids the flame/glow branches use
    // (`TileDrawing.cs:2686-2825`, `:3463-3491`).
    "Glow_131",
    "Glow_291",
    "Glow_293",
    "Glow_295",
    "Glow_296",
    "Glow_297",
    "Glow_298",
    "Glow_299",
    "Glow_300",
    "Glow_325",
    "Glow_334",
    "Glow_335",
    // The named single-sheet textures.
    "GlowSnail",
    "FireflyJar",
    "LightningBugJar",
    "XmasLight",
    "Timer",
    "SunAltar",
    "jellyfishBowl1",
    "jellyfishBowl2",
    "jellyfishBowl3",
];

/// The ambient-sky creature sheets with their `SpriteFrame(1, rows)` row
/// counts (`AmbientSky.cs`, one entry per `Main.Assets.Request`).
const AMBIENCE_SHEETS: &[(&str, u32)] = &[
    ("Backgrounds/Ambience/ButterflySwarm1", 17),
    ("Backgrounds/Ambience/ButterflySwarm2", 19),
    ("Backgrounds/Ambience/LostKite", 42),
    ("Backgrounds/Ambience/Pegasus", 11),
    ("Backgrounds/Ambience/Vulture", 10),
    ("Backgrounds/Ambience/PixiePosse1", 25),
    ("Backgrounds/Ambience/PixiePosse2", 25),
    ("Backgrounds/Ambience/BirdsVShape", 4),
    ("Backgrounds/Ambience/Seagull", 9),
    ("Backgrounds/Ambience/Gastropod", 1),
    ("Backgrounds/Ambience/SlimeBalloons", 7),
    ("Backgrounds/Ambience/HellBat1", 10),
    ("Backgrounds/Ambience/HellBat2", 10),
    ("Backgrounds/Ambience/Bat1", 10),
    ("Backgrounds/Ambience/Bat2", 10),
    ("Backgrounds/Ambience/Bat3", 10),
    ("Backgrounds/Ambience/Wyvern", 5),
    ("Backgrounds/Ambience/FlyingShip", 4),
    ("Backgrounds/Ambience/AirBalloons_Large", 5),
    ("Backgrounds/Ambience/AirBalloons_Small", 5),
    ("Backgrounds/Ambience/EOC", 3),
    ("Backgrounds/Ambience/EOS1", 4),
    ("Backgrounds/Ambience/EOS2", 4),
    ("Backgrounds/Ambience/EOS3", 4),
    ("Backgrounds/Ambience/Crimera1", 3),
    ("Backgrounds/Ambience/Crimera2", 3),
    ("Backgrounds/Ambience/Crimera3", 3),
    ("Backgrounds/Ambience/Meteor", 4),
];

/// Texture information for one tile type present in the world.
#[derive(Clone, Debug)]
pub struct TypeTex {
    pub sheet: String,
    /// UV rectangle of the "plain" 16x16 frame (fallback / no frame stored).
    pub plain_uv: [f32; 4],
    pub plain_opaque: bool,
    /// Dense 18 px frame grid: index `(fy / 18) * cols + fx / 18`.
    pub cols: u32,
    pub rows: u32,
    pub frames: Vec<FrameTex>,
}

/// Texture information for one wall type present in the world.
#[derive(Clone, Debug)]
pub struct WallTex {
    pub sheet: String,
    /// `Framing.wallFrameLookup` cells: `row * 13 + col`. Each entry is the 32x32
    /// sprite `WallDrawing.DrawWalls` draws 8 px up/left of the cell.
    /// `None` when the sheet has no such cell.
    pub frames: Vec<Option<FrameTex>>,
}

/// Wall autotile table dimensions (columns 0..=12, rows 0..=4).
pub const WALL_COLS: usize = 13;
pub const WALL_ROWS: usize = 5;

/// Tree canopy + branch sheets for one foliage style.
#[derive(Clone, Copy, Debug)]
pub struct FoliageTex {
    pub top: tr_render::SheetRect,
    pub branch: tr_render::SheetRect,
}

/// One cloud texture, already reduced to what the sky pass needs.
#[derive(Clone, Copy, Debug)]
pub struct CloudTex {
    pub uv: [f32; 4],
    pub w: f32,
    pub h: f32,
}

/// One `Images/Backgrounds/Ambience/*` creature sheet: the full-sheet UV plus
/// the 1×N row count the `SpriteFrame(1, rows)` grid addresses top-down.
#[derive(Clone, Copy)]
pub struct AmbienceTex {
    pub uv: [f32; 4],
    pub w: f32,
    pub h: f32,
    /// `SpriteFrame.RowCount`.
    pub frames: u32,
}

/// Precomputed texture information for every type present in the world.
#[derive(Default)]
pub struct Lookups {
    /// Dense per-type tables: one slot per `TileID`, `None` if the sheet is absent.
    pub tiles: Vec<Option<TypeTex>>,
    pub walls: Vec<Option<WallTex>>,
    /// `Tree_Tops_*` / `Tree_Branches_*` per foliage style.
    pub foliage: Vec<Option<FoliageTex>>,
    /// `Cloud_0`..`Cloud_41`.
    pub clouds: Vec<Option<CloudTex>>,
    /// The `Images/Backgrounds/Ambience/*` creature sheets, keyed by the
    /// `Main.Assets.Request` name (`AmbientSky.cs`).
    pub ambience: HashMap<&'static str, AmbienceTex>,
    /// `Misc/water_0..14` sheets (water styles, lava, honey, shimmer).
    pub liquids: Vec<Option<tr_render::SheetRect>>,
    /// `LiquidSlope_0..14`: 4 diagonal masks (18 px stride) drawn behind slopes.
    pub liquid_slopes: Vec<Option<tr_render::SheetRect>>,
    /// `Liquid_0..14`: 16 px partial strips (18 px stride) drawn behind blocks.
    pub liquid_parts: Vec<Option<tr_render::SheetRect>>,
    /// `Waterfall_0..27` (`WaterfallManager.waterfallTexture`).
    pub waterfalls: [Option<tr_render::SheetRect>; tr_core::waterfall::MAX_TYPES],
    /// `TextureAssets.Flames` (`Images/Flame_0..17`, `AssetInitializer.cs:382-384`).
    pub flames: [Option<tr_render::SheetRect>; 18],
    /// The tile-side glow sheets of `DrawSingleTile_Flames`, keyed by the C#
    /// texture name (`Images/Glow_{id}` and the named sheets).
    pub glow_sheets: HashMap<&'static str, tr_render::SheetRect>,
    /// `Rain.png`: 2x40 drop frames at 4 px strides.
    pub rain: Option<tr_render::SheetRect>,
    /// `Dust.png`: 10 px cells, 8 px frames.
    pub dust: Option<tr_render::SheetRect>,
    /// `Projectile_3.png`: the 22x22 shuriken frame.
    pub shuriken: Option<tr_render::SheetRect>,
    /// The ambient leaf gores the world's trees can shed: `Gore_910`
    /// (forest) and `Gore_911` (palm), 32-column (paint) x 8-row (variant)
    /// strips (`GoreID.cs:645-647`, `Gore.cs:1390-1401`).
    pub leaf_gores: [Option<tr_render::SheetRect>; 2],
    /// `Item_71..74`: copper/silver/gold/platinum coin sprites.
    pub coins: [Option<tr_render::SheetRect>; 4],
    /// Vanilla HUD textures (`Images/Inventory_Back`, `Heart`, ...).
    pub ui_back: Option<tr_render::SheetRect>,
    /// `Inventory_Back14`: the yellow frame of the selected hotbar slot.
    pub ui_back_sel: Option<tr_render::SheetRect>,
    /// `Inventory_Back9`: the hotbar row frame inside the open inventory.
    pub ui_back_hotbar: Option<tr_render::SheetRect>,
    /// `Item_<id>` sprites for the HUD/inventory icons, keyed by `ItemID`.
    pub item_icons: HashMap<u16, tr_render::SheetRect>,
    /// `Fonts/Mouse_Text` for UI labels and item names.
    pub ui_font: Option<UiFont>,
    /// `Fonts/Item_Stack` for stack counts and slot numbers.
    pub item_font: Option<UiFont>,
    /// `Fonts/Death_Text`: the menu items (`FontAssets.DeathText`).
    pub death_font: Option<UiFont>,
    /// `Logo` (menu title) as UV plus native size.
    pub logo: Option<([f32; 4], f32, f32)>,
    pub ui_heart: Option<tr_render::SheetRect>,
    pub ui_bubble: Option<tr_render::SheetRect>,
    pub ui_boss_bar: Option<tr_render::SheetRect>,
    /// `NPC_Head_Boss_7` — the boss-bar icon (`CommonBossBigProgressBar.cs:34`).
    /// King Slime (NPC 50) is head index 7 in `BossHeadTextures`
    /// (`NPCID.cs:10479`: `... 50, 7, ...`), which is the head sheet
    /// `GetBossHeadTextureIndex` resolves for it (`NPC.cs:7557-7585`).
    pub ui_boss_head: Option<tr_render::SheetRect>,
    /// `Images/UI/Cursor_0` / `Cursor_1`: the vanilla mouse cursor
    /// (`Main.DrawCursor`; `[0]` normal, `[1]` smart).
    pub cursor: [Option<tr_render::SheetRect>; 2],
    /// `Images/Mana`: the mana stars (`ClassicPlayerResourcesDisplaySet.DrawMana`).
    pub ui_mana: Option<tr_render::SheetRect>,
    /// `Inventory_Back13`: the tooltip panel 9-slice (`Utils.DrawInvBG`).
    pub ui_inv_bg13: Option<tr_render::SheetRect>,
    /// `Inventory_Back3`: misc-equip slot frame (`ItemSlot.cs:2280`).
    pub ui_back3: Option<tr_render::SheetRect>,
    /// `Inventory_Back4`: reforge/guide/crafting slot frame (`ItemSlot.cs:2286`).
    pub ui_back4: Option<tr_render::SheetRect>,
    /// `Inventory_Back5`: the world-chest slot frame (`ItemSlot.cs:2215-2221`).
    pub ui_back5: Option<tr_render::SheetRect>,
    /// `Inventory_Back6`: shop slot frame (`ItemSlot.cs:2292`).
    pub ui_back6: Option<tr_render::SheetRect>,
    /// `Inventory_Back7`: the trash slot frame (`ItemSlot.cs:2295`).
    pub ui_back7: Option<tr_render::SheetRect>,
    /// `Inventory_Back12`: misc-dye slot frame (`ItemSlot.cs:2310`).
    pub ui_back12: Option<tr_render::SheetRect>,
    /// `Trash`: the trash-can glyph (`TextureAssets.Trash`).
    pub ui_trash: Option<tr_render::SheetRect>,
    /// `Extra_54`: the empty-equipment slot glyphs (`TextureAssets.Extra[54]`,
    /// `ItemSlot.cs:2490-2498`).
    pub ui_equip_glyph: Option<tr_render::SheetRect>,
    /// `UI/DisplaySlots_0..10`: the equipment page tabs (`DrawPageIcons`,
    /// `Main.cs:42787`).
    pub page_icons: [Option<tr_render::SheetRect>; 11],
    /// `UI/PVP_0`/`PVP_1`: the PVP toggle and team icons (`Main.cs:42654`).
    pub pvp: [Option<tr_render::SheetRect>; 2],
    /// `Extra_58`: the defence shield (`DrawDefenseCounter`, `Main.cs:42148`).
    pub ui_defense_shield: Option<tr_render::SheetRect>,
    /// `Inventory_Back18`: the crafting-recipe slot frame (`ItemSlot.cs:2272`).
    pub ui_back18: Option<tr_render::SheetRect>,
    /// `UI/PanelBackground`: the panel background 9-slice (`UIPanel.cs:30`).
    pub ui_panel_bg: Option<tr_render::SheetRect>,
    /// `UI/PanelBorder`: the panel border 9-slice (`UIPanel.cs:26`).
    pub ui_panel_border: Option<tr_render::SheetRect>,
    /// `UI/CharCreation/Separator1`: the horizontal separator (`UIHorizontalSeparator.cs:21`).
    pub ui_separator: Option<tr_render::SheetRect>,
    /// `UI/Scrollbar2`: the cyan scrollbar track (`UIScrollbar.cs:64`).
    pub ui_scrollbar: Option<tr_render::SheetRect>,
    /// `UI/ScrollbarInner`: the scrollbar handle (`UIScrollbar.cs:67`).
    pub ui_scrollbar_inner: Option<tr_render::SheetRect>,
    /// `UI/Craft`: the detail-area craft icon (`NewCraftingUI.cs:460`).
    pub ui_craft_icon: Option<tr_render::SheetRect>,
    /// `UI/Craft_Toggle_0..5`: the grid-toggle button (`CraftingUI.cs:290`).
    pub craft_toggles: [Option<tr_render::SheetRect>; 6],
    /// `UI/ChestCraft_0..3`: the nearby-chests toggle (`CraftingUI.cs:341`).
    pub chest_craft: [Option<tr_render::SheetRect>; 4],
    /// `UI/Creative/Infinite_Icons`: the filter-strip icons (`ItemFilters.cs`).
    pub ui_infinite_icons: Option<tr_render::SheetRect>,
    /// `UI/Creative/Infinite_Tabs_B_2`: the filter-strip button frames.
    pub ui_infinite_tabs_b2: Option<tr_render::SheetRect>,
    /// `UI/Bestiary/Button_Search_2`: the search-bar icon button
    /// (`UIWrappedSearchBar.cs:100`).
    pub ui_search_icon: Option<tr_render::SheetRect>,
    /// `UI/Bestiary/Button_Search_Border`: the hovered search-bar border.
    pub ui_search_border: Option<tr_render::SheetRect>,
    /// `Images/UI/Bestiary`: the 2-frame inventory button
    /// (`DrawBestiaryIcon`, `Main.cs:42464`).
    pub ui_bestiary_button: Option<tr_render::SheetRect>,
    /// The bestiary card stack (`UIBestiaryEntryButton.cs:40-86`): the cell
    /// back, the hover frame, the 60 %-white overlay, the front border and
    /// the locked portrait.
    pub ui_bestiary_slot_back: Option<tr_render::SheetRect>,
    pub ui_bestiary_slot_selection: Option<tr_render::SheetRect>,
    pub ui_bestiary_slot_overlay: Option<tr_render::SheetRect>,
    pub ui_bestiary_slot_front: Option<tr_render::SheetRect>,
    pub ui_bestiary_locked: Option<tr_render::SheetRect>,
    /// `UI/Bestiary/Portrait_Front`: the info page's portrait backdrop.
    pub ui_bestiary_portrait: Option<tr_render::SheetRect>,
    /// The info page's stat icons (`UIBestiaryInfoItemLine`'s
    /// `Stat_HP`/`Stat_Attack`/`Stat_Defense`/`Stat_Knockback`/
    /// `Stat_Copper` sheets).
    pub ui_bestiary_stat_hp: Option<tr_render::SheetRect>,
    pub ui_bestiary_stat_attack: Option<tr_render::SheetRect>,
    pub ui_bestiary_stat_defense: Option<tr_render::SheetRect>,
    pub ui_bestiary_stat_knockback: Option<tr_render::SheetRect>,
    pub ui_bestiary_stat_copper: Option<tr_render::SheetRect>,
    /// The twenty-two NPC portrait sheets — the `NpcAssets` set restated
    /// for the UI draws: `[slime, demon eye, wandering eye, zombie, king
    /// slime, guide, skeleton, undead miner, cave bat, jungle bat, hellbat,
    /// voodoo demon, eater of souls, hornet, meteor head, goblin sorcerer,
    /// tim, dark caster, fire imp, chaos ball, burning sphere, water
    /// sphere, goldfish, piranha, shark, the Devourer's three, the Giant
    /// Worm's three, the Eater of Worlds' three]`.
    pub bestiary_npc_sheets: Vec<Option<tr_render::SheetRect>>,
    /// `UI/SearchCancel`: the search-bar cancel button (`UIWrappedSearchBar.cs:145`).
    pub ui_search_cancel: Option<tr_render::SheetRect>,
    /// `Images/Text_Back`: the chat input strip (`Main.cs:40947-40976`).
    pub ui_text_back: Option<tr_render::SheetRect>,
    /// `Images/Extra_48`: the emote sheet (`EmoteBubble.Draw`, `:209`).
    pub ui_emote_sheet: Option<tr_render::SheetRect>,
    /// `Images/Hue`: the hue gradient row of the colour menus (`Main.cs:51141`).
    pub ui_hue: Option<tr_render::SheetRect>,
    /// `Images/ColorBar`: the value-bar / sat-lum-alpha track
    /// (`IngameOptions.cs:1539`), 178x16.
    pub ui_color_bar: Option<tr_render::SheetRect>,
    /// `Images/ColorBlip`: the 2x8 gradient segment (`IngameOptions.cs:1552`).
    pub ui_color_blip: Option<tr_render::SheetRect>,
    /// `Images/ColorSlider`: the 11x25 slider knob (`IngameOptions.cs:1566`).
    pub ui_color_slider: Option<tr_render::SheetRect>,
    /// `Images/UI/Slider_Highlight`: the hover overlay — `TextureAssets.ColorHighlight`
    /// (`AssetInitializer.cs:349`; NOT `Images/ColorHighlight`), 178x16.
    pub ui_slider_highlight: Option<tr_render::SheetRect>,
    // --- P6 front-end sheets (the character/world select + creation screens) ---
    /// P6 card buttons (`UICharacterListItem.cs:52-60`): Play,
    /// FavoriteActive, FavoriteInactive, Rename, Delete, 22x22 each. The
    /// cloud pair is loaded separately for completeness but unused:
    /// `SocialAPI.Cloud == null` (plan §J.2).
    pub card_buttons: [Option<tr_render::SheetRect>; 5],
    /// `UI/ButtonCloudActive` / `ButtonCloudInactive` (unused — no cloud saves).
    pub card_cloud: [Option<tr_render::SheetRect>; 2],
    /// `UI/Divider` 8x4 (`UICharacterListItem.cs:52`).
    pub ui_divider: Option<tr_render::SheetRect>,
    /// `UI/InnerPanelBackground` 24x27 — the card info panels (`:53`, `DrawPanel`
    /// `:310-315` splices it 8/8/8/8).
    pub ui_inner_panel: Option<tr_render::SheetRect>,
    /// `UI/PlayerBackground` 59x58 — the card preview backdrop (`UICharacter.cs:45`).
    pub ui_player_background: Option<tr_render::SheetRect>,
    /// `UI/IconCrimson` 60x58 — the plain world-card icon (`AWorldListItem.cs:199`).
    pub ui_world_icon: Option<tr_render::SheetRect>,
    /// `UI/IconCompletion` 64x70 (`UIWorldListItem.cs:73`; unused — no Moon Lord data).
    pub ui_icon_completion: Option<tr_render::SheetRect>,
    /// `UI/IconMixedSeed` 22x18 (`:85`; unused — no mixed-seed data).
    pub ui_icon_mixed_seed: Option<tr_render::SheetRect>,
    /// `UI/IconPlayedBefore` 25x28 (`:220`).
    pub ui_icon_played_before: Option<tr_render::SheetRect>,
    /// `UI/IconNewlyGenerated` 24x28 (`:221`; unused — fresh entries only).
    pub ui_icon_newly_generated: Option<tr_render::SheetRect>,
    /// `UI/CharCreation/CategoryPanel` 44x44 (`UIColoredImageButton.cs:41`).
    pub cc_panel: Option<tr_render::SheetRect>,
    /// `UI/CharCreation/CategoryPanelBorder` 44x44 (`UIColoredImageButton.cs:52`).
    pub cc_panel_border: Option<tr_render::SheetRect>,
    /// `UI/CharCreation/CategoryPanelHighlight` 36x36 (`UIColoredImageButton.cs:45`).
    pub cc_panel_highlight: Option<tr_render::SheetRect>,
    /// `UI/CharCreation/PanelGrayscale` 44x44 (`GroupOptionButton.cs:126`,
    /// `UIDifficultyButton.cs:34`).
    pub cc_panel_gray: Option<tr_render::SheetRect>,
    /// `UI/CharCreation/SmallPanel` 32x32 (`UIColoredImageButton.cs:37`).
    pub cc_small_panel: Option<tr_render::SheetRect>,
    /// `UI/CharCreation/SmallPanelBorder` 32x32 (`UIColoredImageButton.cs:48`).
    pub cc_small_panel_border: Option<tr_render::SheetRect>,
    /// `UI/CharCreation/CharInfo` 30x30 (`UICharacterCreation.cs:765`).
    pub cc_char_info: Option<tr_render::SheetRect>,
    /// `UI/CharCreation/HairStyle_Hair` 30x30 (`UICharacterCreation.cs:760`).
    pub cc_hair_style: Option<tr_render::SheetRect>,
    /// `UI/CharCreation/HairStyle_Arrow` 30x30 (`UICharacterCreation.cs:762`).
    pub cc_hair_arrow: Option<tr_render::SheetRect>,
    /// The colour-picker icons 30x30 (`UICharacterCreation.cs:735-742`):
    /// ColorHair, ColorEye, ColorEyeBack, ColorSkin, ColorShirt,
    /// ColorUndershirt, ColorPants, ColorShoes.
    pub cc_colors: [Option<tr_render::SheetRect>; 8],
    /// The middle-texture buttons 20x20 (`UICharacterCreation.cs:406-429`
    /// categories bar, `:1006-1031` HSL menu): Copy, Paste, Randomize.
    pub cc_copy_paste_random: [Option<tr_render::SheetRect>; 3],
    /// `UI/CharCreation/ClothStyleMale` / `ClothStyleFemale` 34x36
    /// (`UICharacterCreation.cs:743,1289`).
    pub cc_cloth_styles: [Option<tr_render::SheetRect>; 2],
    /// `UI/CharCreation/Voice` 26x32 (`:482`).
    pub cc_voice: Option<tr_render::SheetRect>,
    /// `UI/CharCreation/ColorCharacter` 26x238 — the Clothing category's
    /// 7-frame body stack, one 26x34 frame per layer
    /// (`UICharacterCreation.cs:747-759`).
    pub cc_color_character: Option<tr_render::SheetRect>,
    /// `UI/WorldCreation/PreviewBorder` 84x84 (`UIWorldCreationPreview.cs:46`).
    pub wc_preview_border: Option<tr_render::SheetRect>,
    /// `UI/WorldCreation/IconRandomName` / `IconRandomSeed` 38x38
    /// (`UIWorldCreation.cs:223,250`).
    pub wc_icons: [Option<tr_render::SheetRect>; 2],
    /// `IconSizeSmall/Medium/Large` 32x32 (`:384`).
    pub wc_size_icons: [Option<tr_render::SheetRect>; 3],
    /// `IconDifficultyCreative/Normal/Expert/Master` 32x32 (`:433`).
    pub wc_difficulty_icons: [Option<tr_render::SheetRect>; 4],
    /// `IconEvilRandom/Corruption/Crimson` 32x32 (`:478`).
    pub wc_evil_icons: [Option<tr_render::SheetRect>; 3],
    /// `PreviewDifficultyNormal1/Expert1/Master1` 76x76 (`UIWorldCreationPreview.cs:48-50`).
    pub wc_preview_bg: [Option<tr_render::SheetRect>; 3],
    /// The bunny overlay 76x76 (`:51-54`): Normal2, Creative2, Expert2, Master2.
    pub wc_preview_bunny: [Option<tr_render::SheetRect>; 4],
    /// `PreviewEvilRandom/Corruption/Crimson` 76x76 (`:55-59`).
    pub wc_preview_evil: [Option<tr_render::SheetRect>; 3],
    /// `PreviewSizeSmall/Medium/Large` 76x76 (`:60-62`).
    pub wc_preview_size: [Option<tr_render::SheetRect>; 3],
    /// `UI/VK_Shift` 26x26 (`UIVirtualKeyboard.cs:171`).
    pub vk_shift: Option<tr_render::SheetRect>,
    /// `UI/VK_Backspace` 54x26 (`UIVirtualKeyboard.cs:172`).
    pub vk_backspace: Option<tr_render::SheetRect>,
    /// `UI/Minimap/<style>/MinimapFrame` per `ui::minimap::FRAME_STYLES`
    /// (`MinimapFrameTemplate.cs:30`), indexed by style.
    pub minimap_frames: [Option<tr_render::SheetRect>; 9],
    /// `UI/Minimap/<style>/MinimapButton_{ZoomIn,ZoomOut,Reset}` (`:33-35`), the
    /// index order of `MinimapFrame.cs:189-191` (`BUTTON_*`).
    pub minimap_buttons: [[Option<tr_render::SheetRect>; 3]; 9],
    /// `Images/Map_0..7` (`AssetInitializer.cs:386-389`).
    pub map_icons: [Option<tr_render::SheetRect>; 8],
    /// Frames that do not sit on the 18 px grid (furniture and other important
    /// tiles). Populated lazily; bounded by the number of distinct visible frames.
    pub frame_cache: HashMap<(u16, i16, i16), FrameTex>,
    pub atlas_size: (u32, u32),
}

impl Lookups {
    pub fn build(
        atlas: &tr_render::Atlas,
        tile_names: impl Iterator<Item = (u16, String)>,
        wall_names: impl Iterator<Item = (u16, String)>,
        ui_font: Option<UiFont>,
        item_font: Option<UiFont>,
        death_font: Option<UiFont>,
    ) -> Self {
        let mut me = Self {
            tiles: vec![None; tr_core::tiledata::TILE_COUNT],
            walls: vec![None; tr_core::tiledata::WALL_COUNT],
            foliage: vec![None; 32],
            clouds: vec![None; 42],
            liquids: vec![None; 15],
            liquid_slopes: vec![None; 15],
            liquid_parts: vec![None; 15],
            waterfalls: [None; tr_core::waterfall::MAX_TYPES],
            flames: std::array::from_fn(|i| atlas.sheet(&format!("Flame_{i}")).copied()),
            glow_sheets: GLOW_SHEET_NAMES
                .iter()
                .filter_map(|n| atlas.sheet(n).map(|r| (*n, *r)))
                .collect(),
            ambience: HashMap::new(),
            frame_cache: HashMap::new(),
            rain: atlas.sheet("Rain").copied(),
            dust: atlas.sheet("Dust").copied(),
            shuriken: atlas.sheet("Projectile_3").copied(),
            leaf_gores: [910, 911].map(|i| atlas.sheet(&format!("Gore_{i}")).copied()),
            coins: [71, 72, 73, 74].map(|i| atlas.sheet(&format!("Item_{i}")).copied()),
            ui_back: atlas.sheet("Inventory_Back").copied(),
            ui_back_sel: atlas.sheet("Inventory_Back14").copied(),
            ui_back_hotbar: atlas.sheet("Inventory_Back9").copied(),
            ui_font,
            item_font,
            death_font,
            logo: atlas.sheet("Logo").map(|r| {
                (
                    uv_rect(r, (atlas.width, atlas.height), 0, 0, r.width, r.height),
                    r.width as f32,
                    r.height as f32,
                )
            }),
            // Every `Item_<id>` sheet that made it into the atlas, keyed by the
            // parsed id (`AssetInitializer.cs:124` names them
            // `Images/Item_<i>`). An id without a texture file never registers,
            // so the draw code's coin/wall/tile fallbacks (`item.rs`) stay
            // reachable; the map content is iteration-order independent, so
            // building it from the sheets HashMap keeps runs reproducible.
            item_icons: atlas
                .sheets
                .iter()
                .filter_map(|(name, rect)| {
                    Some((name.strip_prefix("Item_")?.parse::<u16>().ok()?, *rect))
                })
                .collect(),
            ui_heart: atlas.sheet("Heart").copied(),
            ui_bubble: atlas.sheet("Bubble").copied(),
            ui_boss_bar: atlas.sheet("UI/UI_BossBar").copied(),
            ui_boss_head: atlas.sheet("NPC_Head_Boss_7").copied(),
            cursor: [0, 1].map(|i| atlas.sheet(&format!("UI/Cursor_{i}")).copied()),
            ui_mana: atlas.sheet("Mana").copied(),
            ui_inv_bg13: atlas.sheet("Inventory_Back13").copied(),
            ui_back3: atlas.sheet("Inventory_Back3").copied(),
            ui_back4: atlas.sheet("Inventory_Back4").copied(),
            ui_back5: atlas.sheet("Inventory_Back5").copied(),
            ui_back6: atlas.sheet("Inventory_Back6").copied(),
            ui_back7: atlas.sheet("Inventory_Back7").copied(),
            ui_back12: atlas.sheet("Inventory_Back12").copied(),
            ui_trash: atlas.sheet("Trash").copied(),
            ui_equip_glyph: atlas.sheet("Extra_54").copied(),
            ui_defense_shield: atlas.sheet("Extra_58").copied(),
            ui_back18: atlas.sheet("Inventory_Back18").copied(),
            ui_panel_bg: atlas.sheet("UI/PanelBackground").copied(),
            ui_panel_border: atlas.sheet("UI/PanelBorder").copied(),
            ui_separator: atlas.sheet("UI/CharCreation/Separator1").copied(),
            ui_scrollbar: atlas.sheet("UI/Scrollbar2").copied(),
            ui_scrollbar_inner: atlas.sheet("UI/ScrollbarInner").copied(),
            ui_craft_icon: atlas.sheet("UI/Craft").copied(),
            craft_toggles: std::array::from_fn(|i| {
                atlas.sheet(&format!("UI/Craft_Toggle_{i}")).copied()
            }),
            chest_craft: std::array::from_fn(|i| {
                atlas.sheet(&format!("UI/ChestCraft_{i}")).copied()
            }),
            ui_infinite_icons: atlas.sheet("UI/Creative/Infinite_Icons").copied(),
            ui_infinite_tabs_b2: atlas.sheet("UI/Creative/Infinite_Tabs_B_2").copied(),
            ui_bestiary_button: atlas.sheet("UI/Bestiary").copied(),
            ui_bestiary_slot_back: atlas.sheet("UI/Bestiary/Slot_Back").copied(),
            ui_bestiary_slot_selection: atlas.sheet("UI/Bestiary/Slot_Selection").copied(),
            ui_bestiary_slot_overlay: atlas.sheet("UI/Bestiary/Slot_Overlay").copied(),
            ui_bestiary_slot_front: atlas.sheet("UI/Bestiary/Slot_Front").copied(),
            ui_bestiary_locked: atlas.sheet("UI/Bestiary/Icon_Locked").copied(),
            ui_bestiary_portrait: atlas.sheet("UI/Bestiary/Portrait_Front").copied(),
            ui_bestiary_stat_hp: atlas.sheet("UI/Bestiary/Stat_HP").copied(),
            ui_bestiary_stat_attack: atlas.sheet("UI/Bestiary/Stat_Attack").copied(),
            ui_bestiary_stat_defense: atlas.sheet("UI/Bestiary/Stat_Defense").copied(),
            ui_bestiary_stat_knockback: atlas.sheet("UI/Bestiary/Stat_Knockback").copied(),
            ui_bestiary_stat_copper: atlas.sheet("UI/Bestiary/Stat_Copper").copied(),
            bestiary_npc_sheets: vec![
                atlas.sheet("NPC_1").copied(),
                atlas.sheet("NPC_2").copied(),
                atlas.sheet("NPC_133").copied(),
                atlas.sheet("NPC_3").copied(),
                atlas.sheet("NPC_50").copied(),
                atlas.sheet("NPC_22").copied(),
                atlas.sheet("NPC_21").copied(),
                atlas.sheet("NPC_44").copied(),
                atlas.sheet("NPC_49").copied(),
                atlas.sheet("NPC_51").copied(),
                atlas.sheet("NPC_60").copied(),
                atlas.sheet("NPC_66").copied(),
                atlas.sheet("NPC_6").copied(),
                atlas.sheet("NPC_42").copied(),
                atlas.sheet("NPC_23").copied(),
                atlas.sheet("NPC_29").copied(),
                atlas.sheet("NPC_45").copied(),
                atlas.sheet("NPC_32").copied(),
                atlas.sheet("NPC_24").copied(),
                atlas.sheet("NPC_30").copied(),
                atlas.sheet("NPC_25").copied(),
                atlas.sheet("NPC_33").copied(),
                atlas.sheet("NPC_55").copied(),
                atlas.sheet("NPC_58").copied(),
                atlas.sheet("NPC_65").copied(),
                atlas.sheet("NPC_7").copied(),
                atlas.sheet("NPC_17").copied(),
                atlas.sheet("NPC_8").copied(),
                atlas.sheet("NPC_9").copied(),
                atlas.sheet("NPC_10").copied(),
                atlas.sheet("NPC_11").copied(),
                atlas.sheet("NPC_12").copied(),
                atlas.sheet("NPC_13").copied(),
                atlas.sheet("NPC_14").copied(),
                atlas.sheet("NPC_15").copied(),
                atlas.sheet("NPC_20").copied(),
                atlas.sheet("NPC_18").copied(),
                atlas.sheet("NPC_19").copied(),
                atlas.sheet("NPC_38").copied(),
                atlas.sheet("NPC_227").copied(),
                atlas.sheet("NPC_228").copied(),
                atlas.sheet("NPC_54").copied(),
                atlas.sheet("NPC_4").copied(),
                atlas.sheet("NPC_222").copied(),
                atlas.sheet("NPC_35").copied(),
                atlas.sheet("NPC_5").copied(),
                atlas.sheet("NPC_113").copied(),
                atlas.sheet("NPC_115").copied(),
                atlas.sheet("NPC_143").copied(),
                atlas.sheet("NPC_144").copied(),
                atlas.sheet("NPC_145").copied(),
                atlas.sheet("NPC_383").copied(),
                atlas.sheet("NPC_520").copied(),
                atlas.sheet("NPC_212").copied(),
                atlas.sheet("NPC_216").copied(),
                atlas.sheet("NPC_117").copied(),
                atlas.sheet("NPC_118").copied(),
                atlas.sheet("NPC_134").copied(),
                atlas.sheet("NPC_127").copied(),
                atlas.sheet("NPC_125").copied(),
                atlas.sheet("NPC_126").copied(),
                atlas.sheet("NPC_262").copied(),
                atlas.sheet("NPC_266").copied(),
                atlas.sheet("NPC_267").copied(),
                atlas.sheet("NPC_26").copied(),
                atlas.sheet("NPC_27").copied(),
                atlas.sheet("NPC_28").copied(),
                atlas.sheet("NPC_111").copied(),
                atlas.sheet("NPC_471").copied(),
                atlas.sheet("NPC_105").copied(),
                atlas.sheet("NPC_107").copied(),
                atlas.sheet("NPC_245").copied(),
                atlas.sheet("NPC_246").copied(),
                atlas.sheet("NPC_247").copied(),
                atlas.sheet("NPC_249").copied(),
                atlas.sheet("NPC_399").copied(),
            ],
            ui_search_icon: atlas.sheet("UI/Bestiary/Button_Search_2").copied(),
            ui_search_border: atlas.sheet("UI/Bestiary/Button_Search_Border").copied(),
            ui_search_cancel: atlas.sheet("UI/SearchCancel").copied(),
            ui_text_back: atlas.sheet("Text_Back").copied(),
            ui_emote_sheet: atlas.sheet("Extra_48").copied(),
            ui_hue: atlas.sheet("Hue").copied(),
            ui_color_bar: atlas.sheet("ColorBar").copied(),
            ui_color_blip: atlas.sheet("ColorBlip").copied(),
            ui_color_slider: atlas.sheet("ColorSlider").copied(),
            ui_slider_highlight: atlas.sheet("UI/Slider_Highlight").copied(),
            // P6 card buttons (`UICharacterListItem.cs:52-60` — cloud sheets
            // loaded for completeness but unused: SocialAPI.Cloud is null, plan §J.2).
            card_buttons: [
                "UI/ButtonPlay",
                "UI/ButtonFavoriteActive",
                "UI/ButtonFavoriteInactive",
                "UI/ButtonRename",
                "UI/ButtonDelete",
            ]
            .map(|n| atlas.sheet(n).copied()),
            card_cloud: ["UI/ButtonCloudActive", "UI/ButtonCloudInactive"]
                .map(|n| atlas.sheet(n).copied()),
            ui_divider: atlas.sheet("UI/Divider").copied(),
            ui_inner_panel: atlas.sheet("UI/InnerPanelBackground").copied(),
            ui_player_background: atlas.sheet("UI/PlayerBackground").copied(),
            ui_world_icon: atlas.sheet("UI/IconCrimson").copied(),
            ui_icon_completion: atlas.sheet("UI/IconCompletion").copied(),
            ui_icon_mixed_seed: atlas.sheet("UI/IconMixedSeed").copied(),
            ui_icon_played_before: atlas.sheet("UI/IconPlayedBefore").copied(),
            ui_icon_newly_generated: atlas.sheet("UI/IconNewlyGenerated").copied(),
            cc_panel: atlas.sheet("UI/CharCreation/CategoryPanel").copied(),
            cc_panel_border: atlas.sheet("UI/CharCreation/CategoryPanelBorder").copied(),
            cc_panel_highlight: atlas
                .sheet("UI/CharCreation/CategoryPanelHighlight")
                .copied(),
            cc_panel_gray: atlas.sheet("UI/CharCreation/PanelGrayscale").copied(),
            cc_small_panel: atlas.sheet("UI/CharCreation/SmallPanel").copied(),
            cc_small_panel_border: atlas.sheet("UI/CharCreation/SmallPanelBorder").copied(),
            cc_char_info: atlas.sheet("UI/CharCreation/CharInfo").copied(),
            cc_hair_style: atlas.sheet("UI/CharCreation/HairStyle_Hair").copied(),
            cc_hair_arrow: atlas.sheet("UI/CharCreation/HairStyle_Arrow").copied(),
            cc_colors: [
                "UI/CharCreation/ColorHair",
                "UI/CharCreation/ColorEye",
                "UI/CharCreation/ColorEyeBack",
                "UI/CharCreation/ColorSkin",
                "UI/CharCreation/ColorShirt",
                "UI/CharCreation/ColorUndershirt",
                "UI/CharCreation/ColorPants",
                "UI/CharCreation/ColorShoes",
            ]
            .map(|n| atlas.sheet(n).copied()),
            cc_copy_paste_random: [
                "UI/CharCreation/Copy",
                "UI/CharCreation/Paste",
                "UI/CharCreation/Randomize",
            ]
            .map(|n| atlas.sheet(n).copied()),
            cc_cloth_styles: [
                "UI/CharCreation/ClothStyleMale",
                "UI/CharCreation/ClothStyleFemale",
            ]
            .map(|n| atlas.sheet(n).copied()),
            cc_voice: atlas.sheet("UI/CharCreation/Voice").copied(),
            cc_color_character: atlas.sheet("UI/CharCreation/ColorCharacter").copied(),
            wc_preview_border: atlas.sheet("UI/WorldCreation/PreviewBorder").copied(),
            wc_icons: [
                "UI/WorldCreation/IconRandomName",
                "UI/WorldCreation/IconRandomSeed",
            ]
            .map(|n| atlas.sheet(n).copied()),
            wc_size_icons: [
                "UI/WorldCreation/IconSizeSmall",
                "UI/WorldCreation/IconSizeMedium",
                "UI/WorldCreation/IconSizeLarge",
            ]
            .map(|n| atlas.sheet(n).copied()),
            wc_difficulty_icons: [
                "UI/WorldCreation/IconDifficultyCreative",
                "UI/WorldCreation/IconDifficultyNormal",
                "UI/WorldCreation/IconDifficultyExpert",
                "UI/WorldCreation/IconDifficultyMaster",
            ]
            .map(|n| atlas.sheet(n).copied()),
            wc_evil_icons: [
                "UI/WorldCreation/IconEvilRandom",
                "UI/WorldCreation/IconEvilCorruption",
                "UI/WorldCreation/IconEvilCrimson",
            ]
            .map(|n| atlas.sheet(n).copied()),
            wc_preview_bg: [
                "UI/WorldCreation/PreviewDifficultyNormal1",
                "UI/WorldCreation/PreviewDifficultyExpert1",
                "UI/WorldCreation/PreviewDifficultyMaster1",
            ]
            .map(|n| atlas.sheet(n).copied()),
            wc_preview_bunny: [
                "UI/WorldCreation/PreviewDifficultyNormal2",
                "UI/WorldCreation/PreviewDifficultyCreative2",
                "UI/WorldCreation/PreviewDifficultyExpert2",
                "UI/WorldCreation/PreviewDifficultyMaster2",
            ]
            .map(|n| atlas.sheet(n).copied()),
            wc_preview_evil: [
                "UI/WorldCreation/PreviewEvilRandom",
                "UI/WorldCreation/PreviewEvilCorruption",
                "UI/WorldCreation/PreviewEvilCrimson",
            ]
            .map(|n| atlas.sheet(n).copied()),
            wc_preview_size: [
                "UI/WorldCreation/PreviewSizeSmall",
                "UI/WorldCreation/PreviewSizeMedium",
                "UI/WorldCreation/PreviewSizeLarge",
            ]
            .map(|n| atlas.sheet(n).copied()),
            vk_shift: atlas.sheet("UI/VK_Shift").copied(),
            vk_backspace: atlas.sheet("UI/VK_Backspace").copied(),
            minimap_frames: std::array::from_fn(|i| {
                atlas
                    .sheet(&format!(
                        "UI/Minimap/{}/MinimapFrame",
                        crate::ui::minimap::FRAME_STYLES[i].name
                    ))
                    .copied()
            }),
            minimap_buttons: std::array::from_fn(|s| {
                let name = crate::ui::minimap::FRAME_STYLES[s].name;
                [
                    crate::ui::minimap::BUTTON_ZOOM_IN,
                    crate::ui::minimap::BUTTON_ZOOM_OUT,
                    crate::ui::minimap::BUTTON_RESET,
                ]
                .map(|b| {
                    atlas
                        .sheet(&format!(
                            "UI/Minimap/{}/MinimapButton_{}",
                            name,
                            ["ZoomIn", "ZoomOut", "Reset"][b]
                        ))
                        .copied()
                })
            }),
            map_icons: std::array::from_fn(|i| atlas.sheet(&format!("Map_{i}")).copied()),
            page_icons: std::array::from_fn(|i| {
                atlas.sheet(&format!("UI/DisplaySlots_{i}")).copied()
            }),
            pvp: [0, 1].map(|i| atlas.sheet(&format!("UI/PVP_{i}")).copied()),
            atlas_size: (atlas.width, atlas.height),
        };
        for (kind, name) in tile_names {
            let Some(rect) = atlas.sheet(&name) else {
                continue;
            };
            let plain = rect.plain_frame;
            let plain_uv = uv_of(atlas, rect, &name, plain.0, plain.1);
            let plain_opaque = atlas.frame_is_opaque(rect, plain.0, plain.1);
            let cols = rect.width / 18;
            let rows = rect.height / 18;
            let mut frames = Vec::with_capacity((cols * rows) as usize);
            for fy in 0..rows {
                for fx in 0..cols {
                    let px = (fx * 18) as i16;
                    let py = (fy * 18) as i16;
                    frames.push(FrameTex {
                        uv: uv_of(atlas, rect, &name, px, py),
                        opaque: atlas.frame_is_opaque(rect, px, py),
                    });
                }
            }
            if let Some(slot) = me.tiles.get_mut(kind as usize) {
                *slot = Some(TypeTex {
                    sheet: name,
                    plain_uv,
                    plain_opaque,
                    cols,
                    rows,
                    frames,
                });
            }
        }
        for (wall, name) in wall_names {
            let Some(rect) = atlas.sheet(&name) else {
                continue;
            };
            let mut frames = Vec::with_capacity(WALL_COLS * WALL_ROWS);
            for row in 0..WALL_ROWS {
                for col in 0..WALL_COLS {
                    // `WallDrawing.DrawWalls`: a 32x32 source at the 36 px cell origin,
                    // drawn 8 px up/left of the cell so neighbouring walls join.
                    let fx = col as i16 * 36;
                    let fy = row as i16 * 36;
                    if fx as u32 + 32 > rect.width || fy as u32 + 32 > rect.height {
                        frames.push(None);
                    } else {
                        frames.push(Some(FrameTex {
                            uv: uv_of_size(atlas, rect, &name, fx, fy, 32, 32),
                            opaque: atlas.rect_is_opaque(rect, fx as u32, fy as u32, 32, 32),
                        }));
                    }
                }
            }
            if let Some(slot) = me.walls.get_mut(wall as usize) {
                *slot = Some(WallTex {
                    sheet: name,
                    frames,
                });
            }
        }
        for style in 0..me.foliage.len() {
            let top = atlas.sheet(&format!("Tree_Tops_{style}"));
            let branch = atlas.sheet(&format!("Tree_Branches_{style}"));
            if let (Some(top), Some(branch)) = (top, branch) {
                me.foliage[style] = Some(FoliageTex {
                    top: *top,
                    branch: *branch,
                });
            }
        }
        for i in 0..me.clouds.len() {
            if let Some(rect) = atlas.sheet(&format!("Cloud_{i}")) {
                me.clouds[i] = Some(CloudTex {
                    uv: uv_rect(rect, me.atlas_size, 0, 0, rect.width, rect.height),
                    w: rect.width as f32,
                    h: rect.height as f32,
                });
            }
        }
        // The ambient-sky creature sheets (`AmbientSky.cs`'s requested
        // textures); keyed by the request name.
        for (name, frames) in AMBIENCE_SHEETS {
            if let Some(rect) = atlas.sheet(name) {
                me.ambience.insert(
                    name,
                    AmbienceTex {
                        uv: uv_rect(rect, me.atlas_size, 0, 0, rect.width, rect.height),
                        w: rect.width as f32,
                        h: rect.height as f32,
                        frames: *frames,
                    },
                );
            }
        }
        for i in 0..me.liquids.len() {
            if let Some(rect) = atlas.sheet(&format!("LiquidSlope_{i}")) {
                me.liquid_slopes[i] = Some(*rect);
            }
            if let Some(rect) = atlas.sheet(&format!("Liquid_{i}")) {
                me.liquid_parts[i] = Some(*rect);
            }
            if let Some(rect) = atlas.sheet(&format!("Misc/water_{i}")) {
                me.liquids[i] = Some(*rect);
            }
        }
        for i in 0..me.waterfalls.len() {
            if let Some(rect) = atlas.sheet(&format!("Waterfall_{i}")) {
                me.waterfalls[i] = Some(*rect);
            }
        }
        me
    }

    /// UV of one 2x40 raindrop frame (`DrawRain`: `frame.X = type * 4`).
    pub fn rain_uv(&self, kind: u8) -> Option<[f32; 4]> {
        let rect = self.rain.as_ref()?;
        let x = kind as i32 * 4;
        if x + 2 > rect.width as i32 {
            return None;
        }
        Some(uv_rect(rect, self.atlas_size, x, 0, 2, 40))
    }

    /// UV of a coin item sprite (0 copper .. 3 platinum).
    pub fn coin_uv(&self, kind: usize) -> Option<[f32; 4]> {
        let rect = self.coins.get(kind)?.as_ref()?;
        Some(uv_rect(
            rect,
            self.atlas_size,
            0,
            0,
            rect.width,
            rect.height,
        ))
    }

    /// The leaf gore sheet for `WorldGen.GetTreeLeaf`'s `passStyle` —
    /// `Gore_910` (forest) at index 0, `Gore_911` (palm) at index 1; the other
    /// biome styles load with their biomes.
    pub fn leaf_sheet(&self, style: u32) -> Option<&tr_render::SheetRect> {
        let idx = match style {
            910 => 0,
            911 => 1,
            _ => return None,
        };
        self.leaf_gores.get(idx).and_then(|s| s.as_ref())
    }

    /// One leaf frame: the 32-column (paint) x 8-row (variant) strip
    /// (`Gore.cs:1390-1401`); the port draws column 0 (no paint).
    pub fn leaf_uv(&self, style: u32, row: u8) -> Option<[f32; 4]> {
        let sheet = self.leaf_sheet(style)?;
        let fw = sheet.width / 32;
        let fh = sheet.height / 8;
        if fw == 0 || fh == 0 || row as u32 >= 8 {
            return None;
        }
        Some(uv_rect(
            sheet,
            self.atlas_size,
            0,
            row as i32 * fh as i32,
            fw,
            fh,
        ))
    }

    /// The pixel frame size of one leaf cell.
    pub fn leaf_frame(&self, style: u32) -> Option<(f32, f32)> {
        let sheet = self.leaf_sheet(style)?;
        Some((sheet.width as f32 / 32.0, sheet.height as f32 / 8.0))
    }

    /// UV of the shuriken projectile/item sprite.
    pub fn shuriken_uv(&self) -> Option<[f32; 4]> {
        let rect = self.shuriken.as_ref()?;
        Some(uv_rect(
            rect,
            self.atlas_size,
            0,
            0,
            rect.width,
            rect.height,
        ))
    }

    /// UV of a representative wall cell, used as the inventory/drop icon.
    pub fn wall_icon_uv(&self, wall: u16) -> Option<[f32; 4]> {
        let tex = self.walls.get(wall as usize)?.as_ref()?;
        // Row 2, column 6 is an interior cell without exposed edges. Frames hold the
        // full 32x32 wall sprite; the icon is its central 16x16.
        let idx = 2 * WALL_COLS + 6;
        let uv = tex.frames.get(idx).and_then(|f| f.as_ref())?.uv;
        let du = (uv[2] - uv[0]) * 0.25;
        let dv = (uv[3] - uv[1]) * 0.25;
        Some([uv[0] + du, uv[1] + dv, uv[2] - du, uv[3] - dv])
    }

    /// UV of an 8x8 `Dust.png` cell.
    pub fn dust_uv(&self, sx: i32, sy: i32, w: u32, h: u32) -> Option<[f32; 4]> {
        let rect = self.dust.as_ref()?;
        Some(uv_rect(rect, self.atlas_size, sx, sy, w, h))
    }

    /// UV of a 16x16 source cell inside `Misc/water_{index}`. The sheet is a
    /// vertical strip of 80 px animation blocks, 3 cells wide.
    pub fn liquid_uv(&self, index: usize, sx: i32, sy: i32, w: u32, h: u32) -> Option<[f32; 4]> {
        let rect = self.liquids.get(index)?.as_ref()?;
        Some(uv_rect(rect, self.atlas_size, sx, sy, w, h))
    }

    /// UV of a `WaterfallManager` source rectangle inside `Waterfall_{texture}`.
    pub fn waterfall_uv(&self, texture: usize, source: [i32; 4]) -> Option<[f32; 4]> {
        let rect = self.waterfalls.get(texture)?.as_ref()?;
        if source[2] <= 0 || source[3] <= 0 {
            return None;
        }
        Some(uv_rect(
            rect,
            self.atlas_size,
            source[0],
            source[1],
            source[2] as u32,
            source[3] as u32,
        ))
    }

    /// UV of a source rectangle inside `Images/Flame_{sheet}` — the tile-flame
    /// sheets share the tile's own frame coordinates
    /// (`TextureAssets.Flames`, `TileDrawing.cs:2722` and peers).
    pub fn flame_uv(&self, sheet: usize, sx: i32, sy: i32, w: u32, h: u32) -> Option<[f32; 4]> {
        let rect = self.flames.get(sheet)?.as_ref()?;
        Some(uv_rect(rect, self.atlas_size, sx, sy, w, h))
    }

    /// UV of a pixel rect inside a named glow sheet (`Images/Glow_{id}` or one
    /// of the named single sheets the `DrawSingleTile_Flames` branches use).
    pub fn glow_uv(&self, name: &str, sx: i32, sy: i32, w: u32, h: u32) -> Option<[f32; 4]> {
        let rect = self.glow_sheets.get(name)?;
        Some(uv_rect(rect, self.atlas_size, sx, sy, w, h))
    }

    /// UV of a whole glow sheet plus its pixel size (the `value.Frame()`
    /// draws, e.g. the teleporter pulse on `Glow_334`).
    pub fn glow_frame_uv(&self, name: &str) -> Option<([f32; 4], f32, f32)> {
        let rect = self.glow_sheets.get(name)?;
        Some((
            uv_rect(rect, self.atlas_size, 0, 0, rect.width, rect.height),
            rect.width as f32,
            rect.height as f32,
        ))
    }

    /// UV of a partial `Liquid_*` strip (16 px tall, 18 px frame stride).
    pub fn liquid_part_uv(
        &self,
        index: usize,
        sx: i32,
        sy: i32,
        w: u32,
        h: u32,
    ) -> Option<[f32; 4]> {
        let rect = self.liquid_parts.get(index)?.as_ref()?;
        Some(uv_rect(rect, self.atlas_size, sx, sy, w, h))
    }

    /// UV of a `LiquidSlope_*` mask: 18 px stride, `slope` in 1..=4.
    pub fn liquid_slope_uv(&self, index: usize, slope: u8, sy: i32, h: u32) -> Option<[f32; 4]> {
        let rect = self.liquid_slopes.get(index)?.as_ref()?;
        let sx = 18 * (slope.clamp(1, 4) as i32 - 1);
        Some(uv_rect(rect, self.atlas_size, sx, sy, 16, h))
    }

    /// UV of a whole HUD sheet.
    pub fn hud_uv(&self, rect: Option<&tr_render::SheetRect>) -> Option<[f32; 4]> {
        let rect = rect?;
        Some(uv_rect(
            rect,
            self.atlas_size,
            0,
            0,
            rect.width,
            rect.height,
        ))
    }

    /// UV and native size of the original `Item_<id>` sprite for a slot kind
    /// (`ItemSlot.Draw` uses the item's own texture at inventory scale).
    pub fn item_icon(&self, id: u16) -> Option<([f32; 4], f32, f32)> {
        let rect = self.item_icons.get(&id)?;
        Some((
            uv_rect(rect, self.atlas_size, 0, 0, rect.width, rect.height),
            rect.width as f32,
            rect.height as f32,
        ))
    }

    /// UV of one `UI_BossBar` row (516x58, six vertical frames).
    pub fn boss_bar_uv(&self, row: u32, sx: u32, sy: u32, w: u32, h: u32) -> Option<[f32; 4]> {
        let rect = self.ui_boss_bar.as_ref()?;
        Some(uv_rect(
            rect,
            self.atlas_size,
            sx as i32,
            (row * 58 + sy) as i32,
            w,
            h,
        ))
    }

    /// UV rectangle covering a whole raw (non-tiled) sheet, e.g. the sky gradient.
    pub fn sheet_uv(atlas: &tr_render::Atlas, name: &str) -> Option<[f32; 4]> {
        let rect = atlas.sheet(name)?;
        Some([
            rect.x as f32 / atlas.width as f32,
            rect.y as f32 / atlas.height as f32,
            (rect.x + rect.width) as f32 / atlas.width as f32,
            (rect.y + rect.height) as f32 / atlas.height as f32,
        ])
    }

    /// Texture for a tile frame. Non-important tiles almost always sit on the 18 px
    /// grid and hit the dense table; free-form important frames use a small cache.
    pub fn frame_tex(
        &mut self,
        atlas: &tr_render::Atlas,
        kind: u16,
        fx: i16,
        fy: i16,
    ) -> Option<FrameTex> {
        let tt = self.tiles.get(kind as usize)?.as_ref()?;
        if fx >= 0 && fy >= 0 && fx % 18 == 0 && fy % 18 == 0 {
            let cx = (fx / 18) as u32;
            let cy = (fy / 18) as u32;
            if cx < tt.cols && cy < tt.rows {
                return tt.frames.get((cy * tt.cols + cx) as usize).copied();
            }
        }
        if let Some(f) = self.frame_cache.get(&(kind, fx, fy)) {
            return Some(*f);
        }
        let rect = atlas.sheet(&tt.sheet)?;
        let frame = FrameTex {
            uv: uv_of(atlas, rect, &tt.sheet, fx, fy),
            opaque: atlas.frame_is_opaque(rect, fx, fy),
        };
        self.frame_cache.insert((kind, fx, fy), frame);
        Some(frame)
    }

    /// Fallback texture for a tile type (plain interior frame).
    pub fn plain_tex(&self, kind: u16) -> Option<FrameTex> {
        let tt = self.tiles.get(kind as usize)?.as_ref()?;
        Some(FrameTex {
            uv: tt.plain_uv,
            opaque: tt.plain_opaque,
        })
    }

    /// UV of any source rectangle inside a tile type's sheet. Tree trunks are 20x20
    /// sprites on a 22 px grid, so they cannot use the 16x16 block-frame table.
    pub fn rect_uv(
        &self,
        atlas: &tr_render::Atlas,
        kind: u16,
        fx: i16,
        fy: i16,
        w: u32,
        h: u32,
    ) -> Option<FrameTex> {
        let tt = self.tiles.get(kind as usize)?.as_ref()?;
        let rect = atlas.sheet(&tt.sheet)?;
        let uv = uv_rect(rect, self.atlas_size, fx as i32, fy as i32, w, h);
        let opaque = atlas.rect_is_opaque(rect, fx.max(0) as u32, fy.max(0) as u32, w, h);
        Some(FrameTex { uv, opaque })
    }

    /// UV rectangle of a canopy (`top = true`) or branch sprite for a foliage style.
    pub fn foliage_uv(
        &self,
        style: usize,
        top: bool,
        sx: i32,
        sy: i32,
        w: u32,
        h: u32,
    ) -> Option<[f32; 4]> {
        let f = self.foliage.get(style)?.as_ref()?;
        let rect = if top { &f.top } else { &f.branch };
        Some(uv_rect(rect, self.atlas_size, sx, sy, w, h))
    }

    /// Texture for a wall autotile cell.
    ///
    /// Wall sheets use 36 px cells with the 16x16 art inset by 8 px, and cells are
    /// addressed by column/row rather than by pixel offset (`Framing.wallFrameLookup`).
    pub fn wall_tex(&self, wall: u16, col: u8, row: u8) -> Option<FrameTex> {
        let wt = self.walls.get(wall as usize)?.as_ref()?;
        if col as usize >= WALL_COLS || row as usize >= WALL_ROWS {
            return None;
        }
        wt.frames[row as usize * WALL_COLS + col as usize]
    }
}

fn uv_of(
    atlas: &tr_render::Atlas,
    rect: &tr_render::SheetRect,
    name: &str,
    fx: i16,
    fy: i16,
) -> [f32; 4] {
    uv_of_size(atlas, rect, name, fx, fy, 16, 16)
}

fn uv_of_size(
    atlas: &tr_render::Atlas,
    rect: &tr_render::SheetRect,
    name: &str,
    fx: i16,
    fy: i16,
    w: u32,
    h: u32,
) -> [f32; 4] {
    let x0 = rect.x as f32 + fx.max(0) as f32;
    let y0 = rect.y as f32 + fy.max(0) as f32;
    if fx.max(0) as u32 + w > rect.width || fy.max(0) as u32 + h > rect.height {
        log::warn!(
            "sheet {name}: frame ({fx},{fy}) outside rect {}x{}",
            rect.width,
            rect.height
        );
    }
    [
        x0 / atlas.width as f32,
        y0 / atlas.height as f32,
        (x0 + w as f32) / atlas.width as f32,
        (y0 + h as f32) / atlas.height as f32,
    ]
}

fn uv_rect(
    rect: &tr_render::SheetRect,
    atlas_size: (u32, u32),
    sx: i32,
    sy: i32,
    w: u32,
    h: u32,
) -> [f32; 4] {
    let (aw, ah) = (atlas_size.0 as f32, atlas_size.1 as f32);
    let x0 = rect.x as f32 + sx.max(0) as f32;
    let y0 = rect.y as f32 + sy.max(0) as f32;
    [x0 / aw, y0 / ah, (x0 + w as f32) / aw, (y0 + h as f32) / ah]
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]

    use super::*;
    use std::collections::BTreeSet;
    use std::sync::OnceLock;

    fn raw_image(name: &str, w: u32, h: u32) -> tr_render::AtlasImage {
        tr_render::AtlasImage {
            name: name.to_string(),
            width: w,
            height: h,
            rgba: vec![0; (w * h * 4) as usize],
            layout: tr_render::SheetLayout::Raw,
        }
    }

    /// The real content atlas + Lookups, built once per process. Decodes every
    /// `Item_*.xnb` the junction carries (~6500 sheets), exactly like startup.
    fn content_lookups() -> &'static (tr_render::Atlas, Lookups) {
        static CACHE: OnceLock<Box<(tr_render::Atlas, Lookups)>> = OnceLock::new();
        CACHE.get_or_init(|| {
            let content = crate::app::find_content_dir()
                .expect("assets/Content not found: the asset tests need the game content junction");
            set_content_dir(content.clone());
            let images_dir = content.join("Images");
            let tiles = BTreeSet::new();
            let walls = BTreeSet::new();
            let images =
                load_world_images(&images_dir, &tiles, &walls).expect("world image decode");
            let atlas = tr_render::Atlas::build(images, 8192);
            let lookups = Lookups::build(
                &atlas,
                std::iter::empty(),
                std::iter::empty(),
                None,
                None,
                None,
            );
            Box::new((atlas, lookups))
        })
    }

    #[test]
    fn test_DW_4_1_enumeration_sorted_deduped_deterministic() {
        let dir = std::env::temp_dir().join(format!("tr_item_enum_{}", std::process::id()));
        let images = dir.join("Images");
        std::fs::create_dir_all(&images).unwrap();
        for name in [
            "Item_20.xnb",
            "Item_2.xnb",
            "Item_5.xnb",
            "Tiles_0.xnb",
            "Item_x.xnb",
        ] {
            std::fs::write(images.join(name), b"x").unwrap();
        }
        // Read_dir order is filesystem-dependent; the enumeration must not leak
        // it into the atlas (the plan's determinism edge case).
        let ids = enumerate_item_icon_ids(&images);
        assert_eq!(
            ids,
            vec![2, 5, 20],
            "sorted ascending, non-Item/non-numeric ignored"
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn enumeration_missing_dir_yields_empty_list() {
        let dir = std::env::temp_dir().join(format!("tr_item_enum_absent_{}", std::process::id()));
        assert_eq!(enumerate_item_icon_ids(&dir), Vec::<u16>::new());
    }

    #[test]
    fn test_DW_4_3_fallback_lookups_survive_without_item_sheets() {
        // An atlas carrying only the fallback sheets: `item_icons` must hold
        // exactly the Item_ sheets it has, a file-less id must miss (so the
        // `item.rs:176-191` chain proceeds), and the coin/shuriken/tile
        // fallback lookups the chain ends at must still resolve.
        let images = vec![
            raw_image("Item_2", 16, 16),
            raw_image("Item_71", 16, 16),
            raw_image("Projectile_3", 22, 22),
            raw_image("Tiles_0", 18, 18),
        ];
        let atlas = tr_render::Atlas::build(images, 8192);
        let lookups = Lookups::build(
            &atlas,
            [(0u16, "Tiles_0".to_string())].into_iter(),
            std::iter::empty(),
            None,
            None,
            None,
        );
        assert_eq!(lookups.item_icons.len(), 2, "Item_2 and Item_71 only");
        assert!(lookups.item_icon(2).is_some());
        assert!(
            lookups.item_icon(3).is_none(),
            "id without a sheet misses so the fallback chain engages"
        );
        assert!(
            lookups.coin_uv(0).is_some(),
            "Item_71 feeds the coin fallback"
        );
        assert!(
            lookups.shuriken_uv().is_some(),
            "Projectile_3 feeds the shuriken fallback"
        );
        assert!(
            lookups.plain_tex(0).is_some(),
            "Tiles_0 feeds the tile fallback"
        );
    }

    #[test]
    fn test_DW_4_1_item_icons_cover_enumerated_content_files() {
        let (atlas, lookups) = content_lookups();
        let content = crate::app::find_content_dir().unwrap();
        let enumerated: BTreeSet<u16> = enumerate_item_icon_ids(&content.join("Images"))
            .into_iter()
            .collect();
        assert!(!enumerated.is_empty(), "content carries Item_*.xnb files");
        let registered: BTreeSet<u16> = lookups.item_icons.keys().copied().collect();
        assert_eq!(
            registered.len(),
            enumerated.len(),
            "one icon per present Item_ file (atlas {}x{})",
            atlas.width,
            atlas.height
        );
        assert_eq!(
            registered, enumerated,
            "item_icons == the enumerated file set"
        );
    }

    #[test]
    fn test_DW_4_3_fileless_id_stays_out_of_item_icons() {
        let (_, lookups) = content_lookups();
        // 3665 opens the C# `TextureCopyLoad` block (`ItemID.cs:1314`) and has
        // no `Item_3665.xnb` in content — the dirty boundary. It must miss so
        // the draw code's fallback chain stays the path for it.
        assert!(lookups.item_icon(3665).is_none());
        assert!(
            lookups.item_icon(2).is_some(),
            "dirt block (textured) registers"
        );
        // The former allow-list ids survive the swap.
        for id in [71, 74, 89, 593] {
            assert!(lookups.item_icon(id).is_some(), "Item_{id} registered");
        }
        assert!(
            lookups
                .item_icons
                .values()
                .all(|r| r.width > 0 && r.height > 0),
            "every registered rect is sane"
        );
    }

    #[test]
    fn coin_and_shuriken_fallbacks_resolve_from_enumerated_sheets() {
        let (_, lookups) = content_lookups();
        // `coins` and `shuriken` are atlas-sheet lookups, untouched by the swap;
        // their sheets now arrive through the enumeration instead of the list.
        assert!(lookups.coin_uv(0).is_some());
        assert!(lookups.shuriken_uv().is_some());
    }

    #[test]
    fn item_icon_uvs_are_normalized_and_atlas_fits() {
        let (atlas, lookups) = content_lookups();
        assert!(
            atlas.height <= 8192,
            "atlas {} px tall must stay within the strict-GPU bound",
            atlas.height
        );
        for id in [2, 42, 71, 3664, 6195] {
            let Some((uv, w, h)) = lookups.item_icon(id) else {
                continue; // file-less ids legitimately miss
            };
            assert!(
                uv.iter().all(|c| (0.0..=1.0).contains(c)),
                "id {id} uv {uv:?}"
            );
            assert!(w > 0.0 && h > 0.0);
        }
    }
}
