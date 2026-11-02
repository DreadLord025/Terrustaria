//! `TileDrawing.GetTileDrawData`: per-type sprite size/top/flip/frame.
//!
//! Ported from `Terraria.GameContent.Drawing.TileDrawing.GetTileDrawData`
//! (`decompiled/Terraria/GameContent/Drawing/TileDrawing.cs:4527`, type switch
//! at 4540-6032). The function is a pure table: given the tile kind, its
//! coordinates, its framing and the animation row `tile_frame`
//! (= `Main.tileFrame[type]`, 0 until the animator is ported — exactly the
//! vanilla t=0 state), it returns every metric `DrawSingleTile` needs to build
//! the source rectangle (`TileDrawing.cs:1007`). Branches that need engine
//! state the port does not have yet (wind, liquids, biome probes, cage-frame
//! tables, ...) keep only their unconditional static part and carry a
//! deviation comment; cases without any static part fall to the 16x16 default.
//!
//! The world-probing parts C# keeps in `GetTileDrawData` / `DrawSingleTile`
//! live here too: the palm biome row, the dye-plant frame rewrite, the cactus
//! ground shift and the cog parity shift, applied by
//! [`TileDrawData::apply_world_corrections`] (the caller owns the `World`).

use crate::tiledata::TILE_SOLID;
use crate::world::World;

/// `GetTileDrawData` metrics needed by the tile pass.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TileDrawData {
    pub width: u16,
    pub height: u16,
    /// Vertical offset of the sprite above the cell origin (`tileTop`).
    pub top: i16,
    /// Added to the source rectangle X/Y (`addFrX`/`addFrY`).
    pub add_fr_x: i16,
    pub add_fr_y: i16,
    /// `SpriteEffects.FlipHorizontally`.
    pub flip: bool,
    /// Final value of C#'s `ref tileFrameX` when the case rewrites it
    /// (`None` = keep the tile's frame). The caller builds the source
    /// rectangle from the overridden frame (`TileDrawing.cs:1007`).
    pub frame_x_override: Option<i16>,
    /// Final value of C#'s `ref tileFrameY` when the case rewrites it.
    pub frame_y_override: Option<i16>,
}

impl Default for TileDrawData {
    /// `GetTileDrawData` starts from `tileWidth = tileHeight = 16`,
    /// `tileTop = 0`, `addFrX = 0`, `addFrY = 0`, `SpriteEffects.None`
    /// (`TileDrawing.cs:4529-4535`); `draw_data` starts from the same shape
    /// with `add_fr_y = tile_frame * 38` (`TileDrawing.cs:4533`).
    fn default() -> Self {
        Self {
            width: 16,
            height: 16,
            top: 0,
            add_fr_x: 0,
            add_fr_y: 0,
            flip: false,
            frame_x_override: None,
            frame_y_override: None,
        }
    }
}

/// Per-type overrides of `TileDrawing.GetTileDrawData`.
///
/// `kind` is `Tile.type`, `x`/`y` the tile coordinates (`x` drives the
/// `x % 2 == 0` sprite flip, both feed case 751/207 arithmetic), `frame_x`/
/// `frame_y` the tile's framing, `tile_frame` the animation row
/// `Main.tileFrame[kind]` (0 until the animator is ported — the vanilla t=0
/// state), and `solid_above` is `WorldGen.SolidTile(x, y - 1)` (used by
/// torches). World/engine-dependent sub-branches keep their unconditional
/// static part; everything else is a documented deviation.
pub fn draw_data(
    kind: u16,
    x: i32,
    y: i32,
    frame_x: i16,
    frame_y: i16,
    tile_frame: i32,
    solid_above: bool,
) -> TileDrawData {
    let fx = frame_x as i32;
    let fy = frame_y as i32;
    let mut data = TileDrawData {
        // `addFrY = Main.tileFrame[typeCache] * 38` (TileDrawing.cs:4533).
        add_fr_y: (tile_frame * 38) as i16,
        ..TileDrawData::default()
    };
    // C#: `if (x % 2 == 0) tileSpriteEffect = SpriteEffects.FlipHorizontally;`
    // (per-case below, never global).
    let flip_x = x % 2 == 0;

    match kind {
        // `case 752` — PalworldChilletEgg: 36x38, top 2
        // (TileDrawing.cs:4542-4546).
        752 => {
            data.height = 38;
            data.width = 36;
            data.top = 2;
        }
        // `case 751` — PalworldDigtoiseSleeping: 56x46, the sleeping-cycle row
        // is `tileFrameY += (x + y * 2) % 7 * 46` (TileDrawing.cs:4547-4551).
        751 => {
            data.height = 46;
            data.width = 56;
            data.frame_y_override = Some((fy + (x + y * 2) % 7 * 46) as i16);
        }
        // `case 739 / 748` — JellyfishBlock / WaterBlock:
        // `addFrY = Main.tileFrame * 90` (TileDrawing.cs:4552-4555).
        739 | 748 => {
            data.add_fr_y = (tile_frame * 90) as i16;
        }
        // `case 726` — Tattered Cloth dummy / target dummy: resets the frame
        // to the sheet origin, 20x20 (TileDrawing.cs:4556-4561).
        726 => {
            data.frame_x_override = Some(0);
            data.frame_y_override = Some(0);
            data.width = 20;
            data.height = 20;
        }
        // `case 719` (RainbowCloud) needs `Main.GlobalTimeWrappedHourly`
        // (4562-4570): no static part in the port — default 16x16 (documented
        // deviation).
        // `case 443` — GeyserTrap: `tileTop` by the frame column
        // (TileDrawing.cs:4571-4580).
        443 => {
            if fx / 36 >= 2 {
                data.top = -2;
            } else {
                data.top = 2;
            }
        }
        // `case 571` — Bamboo: flip on even columns, top 2
        // (TileDrawing.cs:4581-4587).
        571 => {
            data.flip = flip_x;
            data.top = 2;
        }
        // `case 698` — DeadCellsDisplayJar: 36x44 and the frame X column is
        // remapped through `/18*38` (TileDrawing.cs:4588-4592).
        698 => {
            data.width = 36;
            data.frame_x_override = Some((fx / 18 * 38) as i16);
            data.height = 44;
        }
        // `case 136` — Switches: top 2 on the first frame column
        // (TileDrawing.cs:4593-4598).
        136 => {
            if fx == 0 {
                data.top = 2;
            }
        }
        // `case 561` — MarbleColumn: raised 2 px, 20 px tall,
        // `addFrY = tileFrameY / 18 * 4` (TileDrawing.cs:4599-4603).
        561 => {
            data.top -= 2;
            data.height = 20;
            data.add_fr_y = (fy / 18 * 4) as i16;
        }
        // `case 518` (LilyPad) needs `tile.liquid` + the block below
        // (4604-4630): no static part — default 16x16 (documented deviation).
        // `case 330..333` — candy cane blocks: top 2 (TileDrawing.cs:4631-4636).
        330..=333 => {
            data.top += 2;
        }
        // `case 129` — Crystals (Life Crystals): kills the default animation
        // row and walks the small-crystal variants by frame column
        // (TileDrawing.cs:4637-4644).
        129 => {
            data.add_fr_y = 0;
            if fx >= 324 {
                let n40 = (fx - 324) / 18;
                data.add_fr_x = (((n40 + tile_frame) % 6 - n40) * 18) as i16;
            }
        }
        // `case 5` (trees) is unreachable in the port: `tree::is_tree` routes
        // kind 5 through the dedicated tree path before `draw_data`, and its
        // frame rewrite needs `GetTreeBiome` (TileDrawing.cs:4645-4649).
        // `case 583..589 / 596 / 616 / 634` — tree variants: 20x20 sprites
        // (TileDrawing.cs:4650-4662). Dead in the app for the same reason;
        // the metrics themselves are complete.
        583..=589 | 596 | 616 | 634 => {
            data.width = 20;
            data.height = 20;
        }
        // `case 476` — GolfHole: 20x18 (TileDrawing.cs:4663-4666).
        476 => {
            data.width = 20;
            data.height = 18;
        }
        // `case 323` — palm tree: 20x20; C# replaces the frame row with
        // `tileFrameY = 22 * GetPalmTreeBiome(x, y)` (4670) — applied by
        // `apply_world_corrections` (the probe needs the world).
        323 => {
            data.width = 20;
            data.height = 20;
        }
        // `case 4` — torches: 20x20, tileTop 4 when the tile above is solid
        // (TileDrawing.cs:4672-4679).
        4 => {
            data.width = 20;
            data.height = 20;
            if solid_above {
                data.top = 4;
            }
        }
        // `case 78`-group — ClayPot / Tombstones / forges / anvils / candelabra
        // / LandMine / PlantDetritus / Pumpkins / HeavyWorkBench / TargetDummy
        // / PartyPresent / ElderCrystalStand / FoodPlatter / detritus echoes:
        // top 2 (TileDrawing.cs:4680-4696).
        78 | 85 | 133 | 134 | 173 | 210 | 233 | 254 | 283 | 378 | 457 | 466 | 520 | 651 | 652 => {
            data.top = 2;
        }
        // `case 100` — Candelabras: top 2 and the 2016 style stride is
        // unwound into addFrX/addFrY (TileDrawing.cs:4697-4704).
        100 => {
            data.top = 2;
            let n41 = fy / 2016;
            data.add_fr_y = (tile_frame * 38 - 2016 * n41) as i16;
            data.add_fr_x = (72 * n41) as i16;
        }
        // `case 530 / 485 / 489 / 490` (OasisPlants / AntlionLarva / PinWheel /
        // WeatherVane) need GetBiomeInfluence, Main.tileFrameCounter and the
        // wind system (4705-4817): no static part beyond the default —
        // documented deviations.
        // `case 705` — OasisPlantsEcho: top 2 (TileDrawing.cs:4724-4726).
        705 => {
            data.top = 2;
        }
        // `case 33`-group — candles: 20 px tall, raised 4 px
        // (TileDrawing.cs:4819-4826).
        33 | 49 | 174 | 372 | 646 => {
            data.height = 20;
            data.top = -4;
        }
        // `case 529` (SeaOats) needs GetBiomeInfluence (4827-4849): documented
        // deviation.
        // `case 3`-group — plants: 20 px tall, flip on even columns
        // (TileDrawing.cs:4850-4863).
        3 | 24 | 61 | 71 | 110 | 201 | 637 | 703 => {
            data.height = 20;
            data.flip = flip_x;
        }
        // `case 20 / 590 / 595 / 615` — saplings: 18 px tall, flip on even
        // columns (TileDrawing.cs:4864-4878).
        20 | 590 | 595 | 615 => {
            data.height = 18;
            data.flip = flip_x;
        }
        // Anvils / Furnaces / DemonAltar / thorns / MushroomTrees / Hellforge
        // / WoodenBeam / Traps / Boulder / DjinnLamp / PicnicTable /
        // FallenLog / beams / BouncyBoulder / altars / boulders: 18 px tall
        // (TileDrawing.cs:4880-4908).
        16
        | 17
        | 26
        | 32
        | 69
        | 72
        | 77
        | 124
        | 137
        | 138
        | 352
        | 462
        | 487
        | 488
        | 574..=578
        | 664
        | 695
        | 704
        | 712..=716 => {
            data.height = 18;
        }
        // `case 79` — beds: 18 px tall; the 2016 style stride unwinds into
        // addFrX/addFrY (TileDrawing.cs:4909-4916).
        79 => {
            data.height = 18;
            let n23 = fy / 2016;
            data.add_fr_y = (tile_frame * 38 - 2016 * n23) as i16;
            data.add_fr_x = (144 * n23) as i16;
        }
        // `case 90` — Bathtubs: 2016 style stride only
        // (TileDrawing.cs:4917-4923).
        90 => {
            let n22 = fy / 2016;
            data.add_fr_y = (tile_frame * 38 - 2016 * n22) as i16;
            data.add_fr_x = (144 * n22) as i16;
        }
        // `case 18` — WorkBenches: the 2016 style stride lives in frame X
        // here (TileDrawing.cs:4924-4930).
        18 => {
            let n21 = fx / 2016;
            data.add_fr_x = (-2016 * n21) as i16;
            data.add_fr_y = (tile_frame * 38 + 20 * n21) as i16;
        }
        // `case 711` — RainbowBoulder: 18 wide on later frame columns, 20 px
        // tall; the glow out-params are TILE-DRAW-2 (TileDrawing.cs:4931-4939).
        711 => {
            if fx > 0 {
                data.width = 18;
            }
            data.height = 20;
        }
        // `case 654` — TNTBarrel: top 2 (TileDrawing.cs:4940-4942).
        654 => {
            data.top += 2;
        }
        // `case 14`-group — Tables / Chests / Detonator / Containers2 /
        // Tables2: 18 px tall only on the `tileFrameY == 18` row
        // (TileDrawing.cs:4943-4952).
        14 | 21 | 411 | 467 | 469 => {
            if fy == 18 {
                data.height = 18;
            }
        }
        // `case 15`-group — chairs/toilets: 18 px tall when
        // `tileFrameY % 40 == 18` (TileDrawing.cs:4953-4959).
        15 | 497 => {
            if fy % 40 == 18 {
                data.height = 18;
            }
        }
        // `case 172 / 376` — Sinks / FishingCrate: 18 px tall when
        // `tileFrameY % 38 == 18` (TileDrawing.cs:4960-4966).
        172 | 376 => {
            if fy % 38 == 18 {
                data.height = 18;
            }
        }
        // `case 27` — Sunflower: 18 px tall when `tileFrameY % 74 == 54`
        // (TileDrawing.cs:4967-4972).
        27 => {
            if fy % 74 == 54 {
                data.height = 18;
            }
        }
        // `case 132 / 135` — Lever / PressurePlates: top 2, 18 px tall
        // (TileDrawing.cs:4973-4977).
        132 | 135 => {
            data.top = 2;
            data.height = 18;
        }
        // `case 82-84` — herbs: 20 px tall, raised 2 px, flip on even columns
        // (TileDrawing.cs:4978-4987).
        82 | 83 | 84 => {
            data.height = 20;
            data.top = -2;
            data.flip = flip_x;
        }
        // `case 324` — BeachPiles: 20x20, raised 2 px, flip on even columns
        // (TileDrawing.cs:4988-4996).
        324 => {
            data.width = 20;
            data.height = 20;
            data.top = -2;
            data.flip = flip_x;
        }
        // `case 494` — GolfTee: top 2 (TileDrawing.cs:4997-4999).
        494 => {
            data.top = 2;
        }
        // Vine family: raised 2 px, flip on even columns
        // (TileDrawing.cs:5000-5013).
        52 | 62 | 115 | 205 | 382 | 528 | 636 | 638 => {
            data.top = -2;
            data.flip = flip_x;
        }
        // `case 80 / 142 / 143` — cacti and pumps: top 2
        // (TileDrawing.cs:5014-5018). `DrawSingleTile` adds 54/108/162 to the
        // source Y for the corrupt / hallowed / crimson grounds — `case 80:`
        // only (TileDrawing.cs:1132-1147), applied by
        // `apply_world_corrections`.
        80 | 142 | 143 => {
            data.top = 2;
        }
        // `case 139` — MusicBoxes: top 2, 2016 stride unwound
        // (TileDrawing.cs:5019-5026).
        139 => {
            data.top = 2;
            let n3 = fy / 2016;
            data.add_fr_y = (tile_frame * 38 - 2016 * n3) as i16;
            data.add_fr_x = (72 * n3) as i16;
        }
        // `case 73 / 74 / 113` — jungle plants: 32 px tall, raised 12 px,
        // flip on even columns (TileDrawing.cs:5027-5036).
        73 | 74 | 113 => {
            data.top = -12;
            data.height = 32;
            data.flip = flip_x;
        }
        // `case 388 / 389` — TallGates: 18 px tall on the seam rows, top -2
        // except on the two lower rows (TileDrawing.cs:5037-5051).
        388 | 389 => {
            const N42: i32 = 94;
            data.top = -2;
            if fy == N42 - 20 || fy == N42 * 2 - 20 || fy == 0 || fy == N42 {
                data.height = 18;
            }
            if fy != 0 && fy != N42 {
                data.top = 0;
            }
        }
        // `case 227` — dye plants: 32x38; top -6 on the planted column
        // (`tileFrameX == 238`) else -20 (5052-5062); `tileFrameX == 204`
        // rewrites the frame per `WorldGen.GetCactusType` (5063-5078) —
        // applied by `apply_world_corrections`; flip on even columns
        // (5079-5082).
        227 => {
            data.width = 32;
            data.height = 38;
            data.top = if fx == 238 { -6 } else { -20 };
            data.flip = flip_x;
        }
        // `case 624 / 700` — AbigailsFlower family: 20x16, top 2, flip on
        // even columns (TileDrawing.cs:5084-5093).
        624 | 700 => {
            data.width = 20;
            data.height = 16;
            data.top += 2;
            data.flip = flip_x;
        }
        // `case 656 / 701` — GlowTulip family: 24x34, raised 16 px, flip on
        // even columns (TileDrawing.cs:5094-5103).
        656 | 701 => {
            data.width = 24;
            data.height = 34;
            data.top -= 16;
            data.flip = flip_x;
        }
        // `case 579` (RockGolemHead) needs `Main.LocalPlayer.Center.X`
        // (5104-5130): no static skeleton is ported — default 16x16
        // (documented deviation).
        // `case 567` — GardenGnome: 26x18, top -2 on the first row, flip on
        // even columns (TileDrawing.cs:5131-5142).
        567 => {
            data.width = 26;
            data.height = 18;
            if fy == 0 {
                data.top = -2;
            }
            data.flip = flip_x;
        }
        // `case 185-187` — SmallPiles / LargePiles / LargePiles2: top 2; the
        // `Main.tileShine2` writes are not ported (shine system); the
        // 185/187 strides unwind into addFrX/addFrY (TileDrawing.cs:5143-5183).
        185 | 186 | 187 => {
            data.top = 2;
            match kind {
                185 => {
                    if fy == 18 {
                        let n18 = fx / 1908;
                        data.add_fr_x = (-1908 * n18) as i16;
                        data.add_fr_y = (tile_frame * 38 + 18 * n18) as i16;
                    }
                }
                187 => {
                    let n17 = fx / 1890;
                    data.add_fr_x = (-1890 * n17) as i16;
                    data.add_fr_y = (tile_frame * 38 + 36 * n17) as i16;
                }
                _ => {}
            }
        }
        // `case 650` — top 2 (TileDrawing.cs:5184-5186).
        650 => {
            data.top = 2;
        }
        // `case 649` — SmallPiles2x1Echo: top 2, 1908 stride unwound
        // (TileDrawing.cs:5187-5194).
        649 => {
            data.top = 2;
            let n16 = fx / 1908;
            data.add_fr_x = (-1908 * n16) as i16;
            data.add_fr_y = (tile_frame * 38 + 18 * n16) as i16;
        }
        // `case 647 / 706` — LargePilesEcho / TerragrimShrineEcho: top 2
        // (TileDrawing.cs:5195-5198).
        647 | 706 => {
            data.top = 2;
        }
        // `case 648` — LargePiles2Echo: top 2, 1890 stride unwound
        // (TileDrawing.cs:5199-5206).
        648 => {
            data.top = 2;
            let n15 = fx / 1890;
            data.add_fr_x = (-1890 * n15) as i16;
            data.add_fr_y = (tile_frame * 38 + 36 * n15) as i16;
        }
        // `case 178` — ExposedGems: top 2 on the upper rows
        // (TileDrawing.cs:5207-5212).
        178 => {
            if fy <= 36 {
                data.top = 2;
            }
        }
        // `case 184` — LongMoss: 20 wide; top 2 upper rows, -2 mid rows
        // (TileDrawing.cs:5213-5223).
        184 => {
            data.width = 20;
            if fy <= 36 {
                data.top = 2;
            } else if fy <= 108 {
                data.top = -2;
            }
        }
        // `case 519` — Cattail: top 2, flip on even columns
        // (TileDrawing.cs:5224-5230).
        519 => {
            data.top = 2;
            data.flip = flip_x;
        }
        // `case 493` — GolfCupFlag: top 2; the addFrY animation needs
        // `Main.tileFrameCounter` + wind (5232-5255) — documented deviation.
        493 => {
            data.top = 2;
        }
        // `case 28`-group — Pots / Statues / DisplayDoll / HatRack / CatBast
        // and friends: top 2 (TileDrawing.cs:5258-5275).
        28 | 105 | 470 | 475 | 506 | 547 | 548 | 552 | 560 | 597 | 613 | 621 | 622 | 623 | 653
        | 699 => {
            data.top = 2;
        }
        // `case 617` — MasterTrophyBase: top 2, frames fold into the 54x144
        // block (TileDrawing.cs:5276-5280).
        617 => {
            data.top = 2;
            data.frame_y_override = Some((fy % 144) as i16);
            data.frame_x_override = Some((fx % 54) as i16);
        }
        // `case 614` — PottedLavaPlantTendrils: the animation row is addFrX
        // here, addFrY is cleared (TileDrawing.cs:5281-5285).
        614 => {
            data.add_fr_x = (tile_frame * 54) as i16;
            data.add_fr_y = 0;
            data.top = 2;
        }
        // `case 81` — Coral: 24x26, raised 8 px, flip on even columns
        // (TileDrawing.cs:5286-5294).
        81 => {
            data.top -= 8;
            data.height = 26;
            data.width = 24;
            data.flip = flip_x;
        }
        // `case 272` — Cog: clears the animation row; `DrawSingleTile` adds
        // the parity shift `*90` on top (TileDrawing.cs:5295-5297 and
        // 1125-1131) — applied by `apply_world_corrections`.
        272 => {
            data.add_fr_y = 0;
        }
        // `case 106` — Sawmill: animation row * 54 (TileDrawing.cs:5298-5300).
        106 => {
            data.add_fr_y = (tile_frame * 54) as i16;
        }
        // `case 300-308 / 354 / 355 / 499` — crafting stations (BoneWelder …
        // HoneyDispenser) / BewitchingTable / AlchemyTable / LesionStation:
        // animation row * 54, top 2 (TileDrawing.cs:5301-5315).
        300..=308 | 354 | 355 | 499 => {
            data.add_fr_y = (tile_frame * 54) as i16;
            data.top = 2;
        }
        // `case 377` — SharpeningStation: animation row * 38, top 2
        // (TileDrawing.cs:5316-5319).
        377 => {
            data.add_fr_y = (tile_frame * 38) as i16;
            data.top = 2;
        }
        // `case 463 / 464` — DefendersForge family: animation row * 72, top 2
        // (TileDrawing.cs:5320-5324).
        463 | 464 => {
            data.add_fr_y = (tile_frame * 72) as i16;
            data.top = 2;
        }
        // `case 491` — VoidVault: top 2, addFrX 54 (TileDrawing.cs:5325-5328).
        491 => {
            data.top = 2;
            data.add_fr_x = 54;
        }
        // `case 379` — Bubble: animation row * 90 (TileDrawing.cs:5329-5331).
        379 => {
            data.add_fr_y = (tile_frame * 90) as i16;
        }
        // `case 349` — MushroomStatue: top 2; the `tileFrameX` rewrite needs
        // `Animation.GetTemporaryFrame` (5332-5341) — documented deviation.
        349 => {
            data.top = 2;
        }
        // `case 441 / 468` — FakeContainers: 18 px tall on the
        // `tileFrameY == 18` row; the `tileFrameY` rewrite needs
        // `Animation.GetTemporaryFrame` (5342-5356) — documented deviation.
        441 | 468 => {
            if fy == 18 {
                data.height = 18;
            }
        }
        // `case 390` — LavaLamp: animation row * 36 (TileDrawing.cs:5357-5359).
        390 => {
            data.add_fr_y = (tile_frame * 36) as i16;
        }
        // `case 412` — LunarCraftingStation: clears the animation row, top 2
        // (TileDrawing.cs:5360-5363).
        412 => {
            data.add_fr_y = 0;
            data.top = 2;
        }
        // `case 36` — Presents: top 2 (TileDrawing.cs:5364-5366).
        36 => {
            data.top = 2;
        }
        // `case 406` — Chimney: 18 px tall on the lower frame rows; the
        // animation row folds per frame band (TileDrawing.cs:5367-5386).
        406 => {
            data.height = 16;
            if fy % 54 >= 36 {
                data.height = 18;
            }
            let mut n39 = tile_frame;
            if fy >= 108 {
                n39 = 6 - fy / 54;
            } else if fy >= 54 {
                n39 = tile_frame - 1;
            }
            data.add_fr_y = (n39 * 56 + fy / 54 * 2) as i16;
        }
        // `case 452` — SillyBalloonMachine: animation row * 54, dropped on
        // later frame columns (TileDrawing.cs:5387-5396).
        452 => {
            let mut n34 = tile_frame;
            if fx >= 54 {
                n34 = 0;
            }
            data.add_fr_y = (n34 * 54) as i16;
        }
        // `case 455` — PartyMonolith: addFrY 0, top 2; the extra row applies
        // only while `BirthdayParty.PartyIsUp` (5397-5408), which the port
        // treats as always false.
        455 => {
            data.add_fr_y = 0;
            data.top = 2;
        }
        // `case 454` — Pigronata: animation row * 54 (TileDrawing.cs:5409-5411).
        454 => {
            data.add_fr_y = (tile_frame * 54) as i16;
        }
        // `case 453 / 456` (SillyBalloonTile / PartyBundleOfBalloonTile) drive
        // off `Main.tileFrameCounter` (5412-5417): no static part — default
        // 16x16 (documented deviations).
        // `case 405` — Fireplace: 16 px tall default, 18 on framed rows; the
        // animation row drops on later frame columns
        // (TileDrawing.cs:5418-5432).
        405 => {
            data.height = 16;
            if fy > 0 {
                data.height = 18;
            }
            let mut n27 = tile_frame;
            if fx >= 54 {
                n27 = 0;
            }
            data.add_fr_y = (n27 * 38) as i16;
        }
        // `case 12`-group — Heart / ShadowOrbs / CookingPots / ManaCrystal /
        // LifeCrystalBoulder / ShadowOrbsReplica: animation row * 36
        // (TileDrawing.cs:5433-5440).
        12 | 31 | 96 | 639 | 665 | 696 => {
            data.add_fr_y = (tile_frame * 36) as i16;
        }
        // `case 238` — Plantera bulb: top 2, animation row * 36
        // (TileDrawing.cs:5441-5444).
        238 => {
            data.top = 2;
            data.add_fr_y = (tile_frame * 36) as i16;
        }
        // `case 593` — VolcanoSmall: addFrX -18 on later columns, top 2; the
        // animated addFrY needs `Animation.GetTemporaryFrame`, the static
        // fallback is `tileFrame * 18` below 18 and 0 otherwise (5445-5465).
        593 => {
            if fx >= 18 {
                data.add_fr_x = -18;
            }
            data.top = 2;
            data.add_fr_y = if fx < 18 { (tile_frame * 18) as i16 } else { 0 };
        }
        // `case 594` — VolcanoLarge: addFrX -36 on later columns, top 2; same
        // fallback shape with 36 (5466-5487).
        594 => {
            if fx >= 36 {
                data.add_fr_x = -36;
            }
            data.top = 2;
            data.add_fr_y = if fx < 36 { (tile_frame * 36) as i16 } else { 0 };
        }
        // `case 592` — BrazierSuspended: animation row * 54
        // (TileDrawing.cs:5488-5490).
        592 => {
            data.add_fr_y = (tile_frame * 54) as i16;
        }
        // `case 228 / 231 / 243 / 247` — DyeVat / Larva / ImbuingStation /
        // Autohammer: top 2, animation row * 54 (TileDrawing.cs:5491-5497).
        228 | 231 | 243 | 247 => {
            data.top = 2;
            data.add_fr_y = (tile_frame * 54) as i16;
        }
        // `case 244` — BubbleMachine: top 2, animation row * 36 only on the
        // first frame column (TileDrawing.cs:5498-5508).
        244 => {
            data.top = 2;
            data.add_fr_y = if fx < 54 { (tile_frame * 36) as i16 } else { 0 };
        }
        // `case 565` — FogMachine: top 2, animation row * 36 only on the
        // first frame column (TileDrawing.cs:5509-5519).
        565 => {
            data.top = 2;
            data.add_fr_y = if fx < 36 { (tile_frame * 36) as i16 } else { 0 };
        }
        // `case 235` — Teleporter: animation row * 18 (TileDrawing.cs:5520-5522).
        235 => {
            data.add_fr_y = (tile_frame * 18) as i16;
        }
        // `case 217 / 218` — Blendometer / MeatGrinder: animation row * 36,
        // top 2 (TileDrawing.cs:5523-5528).
        217 | 218 => {
            data.add_fr_y = (tile_frame * 36) as i16;
            data.top = 2;
        }
        // `case 564` — PlasmaLamp: shares the 217/218 arm in C# (5523-5528)
        // but the glow switch then unconditionally clears addFrY (6105-6113),
        // so the net result is top 2 with no animation offset.
        564 => {
            data.add_fr_y = 0;
            data.top = 2;
        }
        // `case 219 / 220 / 642` — Extractinator family: animation row * 54,
        // top 2 (TileDrawing.cs:5529-5534).
        219 | 220 | 642 => {
            data.add_fr_y = (tile_frame * 54) as i16;
            data.top = 2;
        }
        // `case 270 / 271 / 581` — Firefly / Lavafly-in-a-Bottle and friends:
        // parity chains mod 6 give the frame column, addFrY cleared, flip on
        // even columns (TileDrawing.cs:5535-5563).
        270 | 271 | 581 => {
            let mut n14 = tile_frame + x % 6;
            if x % 2 == 0 {
                n14 += 3;
            }
            if x % 3 == 0 {
                n14 += 3;
            }
            if x % 4 == 0 {
                n14 += 3;
            }
            while n14 > 5 {
                n14 -= 6;
            }
            data.add_fr_x = (n14 * 18) as i16;
            data.add_fr_y = 0;
            data.flip = flip_x;
        }
        // `case 660` — ShimmerflyinaBottle: parity chain mod 5
        // (TileDrawing.cs:5564-5590).
        660 => {
            let mut n11 = tile_frame + x % 5;
            if x % 2 == 0 {
                n11 += 3;
            }
            if x % 3 == 0 {
                n11 += 3;
            }
            if x % 4 == 0 {
                n11 += 3;
            }
            while n11 > 4 {
                n11 -= 5;
            }
            data.add_fr_x = (n11 * 18) as i16;
            data.add_fr_y = 0;
            data.flip = flip_x;
        }
        // `case 572` — SoulBottles: parity chain mod 4
        // (TileDrawing.cs:5591-5604).
        572 => {
            let mut n10 = tile_frame + x % 4;
            while n10 > 3 {
                n10 -= 4;
            }
            data.add_fr_x = (n10 * 18) as i16;
            data.add_fr_y = 0;
            data.flip = flip_x;
        }
        // `case 428` — WeightedPressurePlate: top 4;
        // `PressurePlateHelper.PressurePlatesPressed` would add addFrX 18 —
        // the port treats plates as not pressed (TileDrawing.cs:5605-5611).
        428 => {
            data.top += 4;
        }
        // `case 442` — ProjectilePressurePad: 20x20 (24 wide on the second
        // variant), top by frame column / 22 (TileDrawing.cs:5612-5628).
        442 => {
            data.width = 20;
            data.height = 20;
            match fx / 22 {
                1 => data.top = -4,
                2 => {
                    data.top = -2;
                    data.width = 24;
                }
                3 => data.top = -2,
                _ => {}
            }
        }
        // Big-animal cage group: top 2; the per-animal `Main.*CageFrame`
        // tables are animation state the port does not have — addFrY stays 0
        // (documented deviation; `Main.critterCage = true` side effect is not
        // ported either) (TileDrawing.cs:5629-5755).
        275..=281
        | 296
        | 297
        | 309
        | 358
        | 359
        | 413
        | 414
        | 542
        | 550
        | 551
        | 553
        | 554
        | 558
        | 559
        | 599..=612
        | 632
        | 640
        | 643..=645
        | 710 => {
            data.top = 2;
        }
        // Small-animal cage group: top 2, cage-frame tables not ported
        // (TileDrawing.cs:5756-5832).
        285
        | 286
        | 298
        | 299
        | 310
        | 339
        | 361..=364
        | 391..=394
        | 532
        | 533
        | 538
        | 544
        | 555
        | 556
        | 582
        | 619
        | 629 => {
            data.top = 2;
        }
        // Water-animal cage group (fish bowls, fairy jars, butterfly jars,
        // dragonfly jars): top 2, cage-frame tables not ported
        // (TileDrawing.cs:5833-5897).
        282 | 505 | 543 | 598 | 568..=570 | 288..=295 | 360 | 580 | 620 | 521..=527 => {
            data.top = 2;
        }
        // Jellyfish cage group: top 2, cage-frame tables not ported
        // (TileDrawing.cs:5898-5907).
        316 | 317 | 318 => {
            data.top = 2;
        }
        // `case 207` — WaterFountain: top 2; above frame row 72 the
        // animation folds with the column parity, below it is 0
        // (TileDrawing.cs:5908-5929).
        207 => {
            data.top = 2;
            if fy >= 72 {
                let mut n = tile_frame;
                let mut n2 = x;
                if fx % 36 != 0 {
                    n2 -= 1;
                }
                n += n2 % 6;
                if n >= 6 {
                    n -= 6;
                }
                data.add_fr_y = (n * 72) as i16;
            } else {
                data.add_fr_y = 0;
            }
        }
        // `case 410` — LunarMonolith: 18 px tall on frame row 36; the
        // animation row * 56 applies only from row 56 (TileDrawing.cs:5930-5944).
        410 => {
            if fy == 36 {
                data.height = 18;
            }
            if fy >= 56 {
                data.add_fr_y = (tile_frame * 56) as i16;
            } else {
                data.add_fr_y = 0;
            }
        }
        // `case 480`-group — BloodMoon / Void / Echo / CRT / Retro / Noir
        // Monoliths: top 2; animation row * 54 only from row 54
        // (TileDrawing.cs:5945-5961).
        480 | 509 | 657 | 720 | 721 | 725 => {
            data.top = 2;
            if fy >= 54 {
                data.add_fr_y = (tile_frame * 54) as i16;
            } else {
                data.add_fr_y = 0;
            }
        }
        // `case 658` — ShimmerMonolith: top 2; the animation row * 54 plus a
        // per-band constant by `tileFrameY / 54` (TileDrawing.cs:5962-5981).
        658 => {
            data.top = 2;
            data.add_fr_y = match fy / 54 {
                1 => (tile_frame * 54 + 486) as i16,
                2 => (tile_frame * 54 + 972) as i16,
                _ => (tile_frame * 54) as i16,
            };
        }
        // `case 733` — RadioThingMonolith: top 2; addFrX 54 below row 54,
        // frames fold into the 54x54 block afterwards
        // (TileDrawing.cs:5982-5990).
        733 => {
            data.top = 2;
            if fy < 54 {
                data.add_fr_x += 54;
            }
            data.frame_x_override = Some((fx % 54) as i16);
            data.frame_y_override = Some((fy % 54) as i16);
        }
        // `case 326`-group — fall blocks (Waterfall / Sandfall / Snowfall /
        // Honeyfall / ChimneySmoke / Shimmerfall) and conveyor belts:
        // animation row * 90 (TileDrawing.cs:5991-6003).
        326 | 327 | 328 | 329 | 345 | 351 | 421 | 422 | 458 | 459 | 708 => {
            data.add_fr_y = (tile_frame * 90) as i16;
        }
        // `case 541` (EchoBlock, invisible blocks, 6004-6006) and
        // `case 507 / 508` (StarryGlassBlocks, tileFrameCounter, 6007-6013):
        // no static part — documented deviations.
        // `case 336`-group — LivingFire family: animation row * 90, top 2
        // (TileDrawing.cs:6014-6022).
        336 | 340 | 341 | 342 | 343 | 344 => {
            data.add_fr_y = (tile_frame * 90) as i16;
            data.top = 2;
        }
        // `case 89 / 102 / 753` — Benches / Thrones / GiantTiki: top 2
        // (TileDrawing.cs:6023-6031).
        89 | 102 | 753 => {
            data.top = 2;
        }
        // 19 (platforms), 25, 40, 50, 51, 75 and 76 have no branch in
        // GetTileDrawData and use the generic 16x16 block metrics.
        _ => {}
    }

    // Campfire epilogue: `TileID.Sets.Campfires = CreateBoolSet(215)`
    // (TileID.cs:616) — runs after the switch, overriding the animation row
    // (TileDrawing.cs:6033-6044).
    if kind == 215 {
        data.add_fr_y = if frame_y < 36 {
            (tile_frame * 36) as i16
        } else {
            252
        };
        data.top = 2;
    }
    // The halfBrick epilogue (TileDrawing.cs:6045-6048) and the glow switch
    // (6049-, except 564's addFrY reset handled above) live in the tile pass
    // / TILE-DRAW-2 respectively.

    data
}

/// `WorldGen.beachDistance` (WorldGen.cs:86914) — set once in the static
/// constructor, constant for every world size.
const BEACH_DISTANCE: i32 = 380;

/// `TileDrawing.GetPalmTreeBiome` (TileDrawing.cs:1725-1733): walks down the
/// palm trunk (kind 323) to the first non-palm tile, then classifies it. The
/// C# loop has no bounds check; a tile outside the world behaves like an empty
/// (inactive) one, which stops the walk (`Framing.GetTileSafely` semantics).
pub fn palm_tree_biome(world: &World, x: i32, y: i32) -> i32 {
    let mut i = y;
    while let Some(t) = world.tile(x, i) {
        if !(t.active() && t.kind == 323) {
            break;
        }
        i += 1;
    }
    palm_tree_variant(world, x, i)
}

/// `TileDrawing.GetPalmTreeVariant` (TileDrawing.cs:3491-3515): the ground
/// under the palm picks the style — 0 sand, 1 crimsand, 2 pearlsand,
/// 3 ebonsand, -1 anything else (the C# frame row then becomes -22).
/// `WorldGen.IsPalmOasisTree` (WorldGen.cs:27205-27210) adds 4 inside the
/// beach band.
fn palm_tree_variant(world: &World, x: i32, y: i32) -> i32 {
    let mut variant = -1;
    if let Some(t) = world.tile(x, y) {
        if t.active() {
            if t.kind == 53 {
                variant = 0;
            }
            if t.kind == 234 {
                variant = 1;
            }
            if t.kind == 116 {
                variant = 2;
            }
            if t.kind == 112 {
                variant = 3;
            }
        }
    }
    if x >= BEACH_DISTANCE && x <= world.width - BEACH_DISTANCE {
        variant += 4;
    }
    variant
}

/// The palm frame row: `tileFrameY = 22 * GetPalmTreeBiome(x, y)`
/// (TileDrawing.cs:4670).
pub fn palm_frame_y(biome: i32) -> i32 {
    22 * biome
}

/// `WorldGen.GetCactusType` (WorldGen.cs:47687-47737): walks down the cactus
/// column to the ground and reports which sand family it grows from —
/// (evil, good, crimson) for ebonsand 112 / pearlsand 116 / crimsand 234.
/// `frame_x`/`frame_y` select the probe column for the cactus sprite's side
/// arms. Tiles outside the world behave like empties; the 20-row walk limit
/// and the `InWorld` fluff of 2 are transcribed exactly.
pub fn cactus_type(
    world: &World,
    x: i32,
    y: i32,
    frame_x: i16,
    frame_y: i16,
) -> (bool, bool, bool) {
    let mut evil = false;
    let mut good = false;
    let mut crimson = false;
    // Side arms of the cactus sprite shift the probe column
    // (WorldGen.cs:47694-47701).
    let mut px = x;
    if frame_x == 36 {
        px -= 1;
    }
    if frame_x == 54 {
        px += 1;
    }
    if frame_x == 108 {
        px = if frame_y != 18 { px + 1 } else { px - 1 };
    }
    // `WorldGen.InWorld(x, y, fluff: 2)` (WorldGen.cs:9688-9695).
    let in_world =
        |px: i32, py: i32| px >= 2 && px < world.width - 2 && py >= 2 && py < world.height - 2;
    let mut py = y;
    if !in_world(px, py) {
        return (evil, good, crimson);
    }
    let mut seen_cactus = false;
    // C# reads the start tile after the InWorld check and bails on null
    // (WorldGen.cs:47706-47712); `world.tile` is `None` only out of bounds.
    let mut tile = match world.tile(px, py) {
        Some(t) => *t,
        None => return (evil, good, crimson),
    };
    if tile.kind == 80 && tile.active() {
        seen_cactus = true;
    }
    // The walk continues while the tile is not (active AND solid AND a cactus
    // was seen). `Main.tileSolid[80]` is false, so cactus segments never stop
    // it. The flag update, the increment and the break check keep the C#
    // order — the final check below sees exactly the tile the walk stopped
    // on.
    while !tile.active() || !TILE_SOLID.has(tile.kind) || !seen_cactus {
        if tile.kind == 80 && tile.active() {
            seen_cactus = true;
        }
        py += 1;
        if py > y + 20 || !in_world(px, py) {
            break;
        }
        match world.tile(px, py) {
            Some(t) => tile = *t,
            // C# `tile == null` exits the loop with no classification.
            None => break,
        }
    }
    if tile.active() {
        if tile.kind == 112 {
            evil = true;
        }
        if tile.kind == 116 {
            good = true;
        }
        if tile.kind == 234 {
            crimson = true;
        }
    }
    (evil, good, crimson)
}

/// Dye-plant frame rewrite: `tileFrameX += 238 (good) / +204 (evil) / +272
/// (crimson)` (TileDrawing.cs:5066-5077). Three independent `if`s — the flags
/// are mutually exclusive by construction, the sum is the literal form.
pub fn dye_frame_x_offset(evil: bool, good: bool, crimson: bool) -> i32 {
    204 * evil as i32 + 238 * good as i32 + 272 * crimson as i32
}

/// `DrawSingleTile` cactus shift: `rectangle.Y += 54 (evil) / +108 (good) /
/// +162 (crimson)` (TileDrawing.cs:1132-1147). The switch case is `80:` only —
/// 142/143 are pumps sharing just the `tileTop = 2` arm.
pub fn cactus_source_y_offset(evil: bool, good: bool, crimson: bool) -> i32 {
    54 * evil as i32 + 108 * good as i32 + 162 * crimson as i32
}

/// `DrawSingleTile` cog parity: `(Main.tileFrame[type] + x%2 + y%2 + x%3 +
/// y%3) % 2 * 90` (TileDrawing.cs:1125-1131).
pub fn cog_source_y_offset(x: i32, y: i32, tile_frame: i32) -> i32 {
    (tile_frame + x % 2 + y % 2 + x % 3 + y % 3) % 2 * 90
}

impl TileDrawData {
    /// Applies the world-probing corrections C# makes for the palm biome row
    /// (323), the dye-plant frame rewrite (227, `tileFrameX == 204`), the
    /// cactus ground shift (80) and the cog parity shift (272). Must run right
    /// after [`draw_data`] — the caller builds the source rectangle from the
    /// resulting overrides and `add_fr_*` fields.
    pub fn apply_world_corrections(
        &mut self,
        world: &World,
        kind: u16,
        x: i32,
        y: i32,
        frame_x: i16,
        frame_y: i16,
        tile_frame: i32,
    ) {
        match kind {
            // `case 323`: `tileFrameY = 22 * GetPalmTreeBiome(x, y)` — a
            // replacement of the stored frame row (TileDrawing.cs:4670).
            323 => {
                self.frame_y_override = Some(palm_frame_y(palm_tree_biome(world, x, y)) as i16);
            }
            // `case 227`: only the `tileFrameX == 204` column is rewritten per
            // `GetCactusType`; every other column keeps the stored frame
            // (TileDrawing.cs:5063-5078).
            227 if frame_x == 204 => {
                let (evil, good, crimson) = cactus_type(world, x, y, frame_x, frame_y);
                self.frame_x_override =
                    Some((frame_x as i32 + dye_frame_x_offset(evil, good, crimson)) as i16);
            }
            // `DrawSingleTile` `case 80`: source-Y shift by the ground sand
            // (TileDrawing.cs:1132-1147). Only kind 80 — 142/143 are pumps.
            80 => {
                let (evil, good, crimson) = cactus_type(world, x, y, frame_x, frame_y);
                self.add_fr_y += cactus_source_y_offset(evil, good, crimson) as i16;
            }
            // `DrawSingleTile` `case 272`: parity shift; C# adds it to both
            // `addFrY` and the built rectangle, the port's source rectangle is
            // `frameY + add_fr_y`, so one addition lands the same pixels
            // (TileDrawing.cs:1125-1131).
            272 => {
                self.add_fr_y += cog_source_y_offset(x, y, tile_frame) as i16;
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEFAULT: TileDrawData = TileDrawData {
        width: 16,
        height: 16,
        top: 0,
        add_fr_x: 0,
        add_fr_y: 0,
        flip: false,
        frame_x_override: None,
        frame_y_override: None,
    };

    fn dd(kind: u16, x: i32, y: i32, fx: i16, fy: i16, tile_frame: i32) -> TileDrawData {
        draw_data(kind, x, y, fx, fy, tile_frame, false)
    }

    #[test]
    fn default_block_is_16x16() {
        assert_eq!(TileDrawData::default(), DEFAULT);
        // Generic blocks and the types with no switch case. The global
        // `addFrY = tileFrame * 38` default is the only field that moves with
        // `tile_frame` (TileDrawing.cs:4533).
        for kind in [0, 1, 19, 25, 40, 50, 51, 75, 76, 255, u16::MAX] {
            assert_eq!(draw_data(kind, 0, 0, 0, 0, 0, true), DEFAULT, "kind {kind}");
            assert_eq!(
                draw_data(kind, 7, 9, 90, 18, 0, false),
                DEFAULT,
                "kind {kind}"
            );
            let animated = draw_data(kind, 0, 0, 0, 0, 3, false);
            assert_eq!(
                animated,
                TileDrawData {
                    add_fr_y: 114,
                    ..DEFAULT
                },
                "kind {kind}"
            );
        }
    }

    #[test]
    fn torch_uses_solid_above() {
        let hanging = draw_data(4, 10, 0, 0, 0, 0, true);
        assert_eq!(
            hanging,
            TileDrawData {
                width: 20,
                height: 20,
                top: 4,
                ..DEFAULT
            }
        );
        let standing = draw_data(4, 10, 0, 0, 0, 0, false);
        assert_eq!(
            standing,
            TileDrawData {
                width: 20,
                height: 20,
                top: 0,
                ..DEFAULT
            }
        );
        // Torches do not care about column parity.
        assert_eq!(draw_data(4, 11, 0, 0, 0, 0, true), hanging);
        assert_eq!(draw_data(4, 11, 0, 0, 0, 0, false), standing);
    }

    #[test]
    fn plants_flip_on_even_columns() {
        for kind in [3, 24, 61, 71, 110, 201, 637, 703] {
            let even = dd(kind, 2, 0, 0, 0, 0);
            assert_eq!(
                (even.height, even.top, even.flip),
                (20, 0, true),
                "kind {kind}"
            );
            let odd = dd(kind, 3, 0, 0, 0, 0);
            assert_eq!(
                (odd.height, odd.top, odd.flip),
                (20, 0, false),
                "kind {kind}"
            );
        }
    }

    #[test]
    fn tall_plants_are_raised_and_flipped() {
        for kind in [73, 74, 113] {
            let even = dd(kind, 42, 0, 0, 0, 0);
            assert_eq!(
                (even.width, even.height, even.top, even.flip),
                (16, 32, -12, true),
                "kind {kind}"
            );
            let odd = dd(kind, 43, 0, 0, 0, 0);
            assert_eq!(
                (odd.width, odd.height, odd.top, odd.flip),
                (16, 32, -12, false),
                "kind {kind}"
            );
        }
    }

    #[test]
    fn herbs_are_raised_and_flipped() {
        for kind in [82, 83, 84] {
            let even = dd(kind, 0, 0, 0, 0, 0);
            assert_eq!(
                (even.height, even.top, even.flip),
                (20, -2, true),
                "kind {kind}"
            );
            let odd = dd(kind, 1, 0, 0, 0, 0);
            assert_eq!(
                (odd.height, odd.top, odd.flip),
                (20, -2, false),
                "kind {kind}"
            );
        }
    }

    #[test]
    fn candles_are_20px_high_and_raised() {
        for kind in [33, 49, 174, 372, 646] {
            let data = dd(kind, 0, 0, 0, 0, 0);
            assert_eq!(
                (data.width, data.height, data.top, data.flip),
                (16, 20, -4, false),
                "kind {kind}"
            );
        }
    }

    #[test]
    fn vines_family_is_raised_and_flipped() {
        for kind in [52, 62, 115, 205, 382, 528, 636, 638] {
            let even = dd(kind, 4, 0, 0, 0, 0);
            assert_eq!(
                (even.height, even.top, even.flip),
                (16, -2, true),
                "kind {kind}"
            );
            assert_eq!(dd(kind, 5, 0, 0, 0, 0).flip, false, "kind {kind}");
        }
    }

    #[test]
    fn thorns_beds_and_palms_use_their_extra_size() {
        for kind in [32, 77] {
            assert_eq!(
                (
                    dd(kind, 0, 0, 0, 0, 0).width,
                    dd(kind, 0, 0, 0, 0, 0).height
                ),
                (16, 18),
                "kind {kind}"
            );
        }
        let bed = dd(79, 0, 0, 0, 0, 0);
        assert_eq!(
            (bed.width, bed.height, bed.top, bed.flip),
            (16, 18, 0, false)
        );
    }

    #[test]
    fn cactus_family_and_plantera_bulb_sit_low() {
        for kind in [80, 142, 143] {
            let data = dd(kind, 0, 0, 0, 0, 0);
            assert_eq!(
                (data.width, data.height, data.top, data.flip),
                (16, 16, 2, false),
                "kind {kind}"
            );
        }
        let bulb = dd(238, 0, 0, 0, 0, 0);
        assert_eq!(
            (bulb.width, bulb.height, bulb.top, bulb.add_fr_y),
            (16, 16, 2, 0)
        );
    }

    #[test]
    fn dye_plants_are_32x38_and_flip() {
        // top -20 except the planted column (tileFrameX == 238 -> -6)
        let even = dd(227, 6, 0, 0, 0, 0);
        assert_eq!(
            (even.width, even.height, even.top, even.flip),
            (32, 38, -20, true)
        );
        assert_eq!(dd(227, 7, 0, 0, 0, 0).flip, false);
        assert_eq!(dd(227, 6, 0, 238, 0, 0).top, -6);
        // the frameX == 204 biome rewrite is a next-phase item
        assert_eq!(dd(227, 6, 0, 204, 0, 0).frame_x_override, None);
    }

    #[test]
    fn test_DW_1_4_conditional_heights() {
        // Tables / Chests / Detonator / Containers2 / Tables2: 18 px only on
        // the `tileFrameY == 18` row (TileDrawing.cs:4943-4952).
        for kind in [14, 21, 411, 467, 469] {
            assert_eq!(dd(kind, 0, 0, 0, 18, 0).height, 18, "kind {kind}");
            assert_eq!(dd(kind, 0, 0, 0, 0, 0).height, 16, "kind {kind}");
            assert_eq!(dd(kind, 0, 0, 0, 36, 0).height, 16, "kind {kind}");
        }
        // Chairs / toilets: `tileFrameY % 40 == 18` (TileDrawing.cs:4953-4959).
        for kind in [15, 497] {
            assert_eq!(dd(kind, 0, 0, 0, 18, 0).height, 18, "kind {kind}");
            assert_eq!(dd(kind, 0, 0, 0, 58, 0).height, 18, "kind {kind}");
            assert_eq!(dd(kind, 0, 0, 0, 40, 0).height, 16, "kind {kind}");
            assert_eq!(dd(kind, 0, 0, 0, 0, 0).height, 16, "kind {kind}");
        }
        // Sinks / FishingCrate: `tileFrameY % 38 == 18` (TileDrawing.cs:4960-4966).
        for kind in [172, 376] {
            assert_eq!(dd(kind, 0, 0, 0, 56, 0).height, 18, "kind {kind}");
            assert_eq!(dd(kind, 0, 0, 0, 18, 0).height, 18, "kind {kind}");
            assert_eq!(dd(kind, 0, 0, 0, 38, 0).height, 16, "kind {kind}");
            assert_eq!(dd(kind, 0, 0, 0, 0, 0).height, 16, "kind {kind}");
        }
        // Sunflower: `tileFrameY % 74 == 54` (TileDrawing.cs:4967-4972).
        assert_eq!(dd(27, 0, 0, 0, 128, 0).height, 18);
        assert_eq!(dd(27, 0, 0, 0, 54, 0).height, 18);
        assert_eq!(dd(27, 0, 0, 0, 74, 0).height, 16);
        assert_eq!(dd(27, 0, 0, 0, 0, 0).height, 16);
    }

    #[test]
    fn test_DW_1_4_stride_2016() {
        // 79 beds: addFrY -= 2016*n, addFrX += 144*n (TileDrawing.cs:4909-4916)
        let bed0 = dd(79, 0, 0, 0, 0, 0);
        assert_eq!((bed0.height, bed0.add_fr_x, bed0.add_fr_y), (18, 0, 0));
        let bed1 = dd(79, 0, 0, 0, 2016, 0);
        assert_eq!(
            (bed1.height, bed1.add_fr_x, bed1.add_fr_y),
            (18, 144, -2016)
        );
        let bed2 = dd(79, 0, 0, 0, 4032, 0);
        assert_eq!(
            (bed2.height, bed2.add_fr_x, bed2.add_fr_y),
            (18, 288, -4032)
        );
        // 100 candelabras: 144 -> 72 (TileDrawing.cs:4697-4704)
        let cand = dd(100, 0, 0, 0, 2016, 0);
        assert_eq!((cand.top, cand.add_fr_x, cand.add_fr_y), (2, 72, -2016));
        // 90 bathtubs: stride only, height stays 16 (TileDrawing.cs:4917-4923)
        let bath = dd(90, 0, 0, 0, 2016, 0);
        assert_eq!(
            (bath.height, bath.add_fr_x, bath.add_fr_y),
            (16, 144, -2016)
        );
        // 18 work benches: the stride lives in frame X (TileDrawing.cs:4924-4930)
        let bench1 = dd(18, 0, 0, 2016, 0, 0);
        assert_eq!((bench1.add_fr_x, bench1.add_fr_y), (-2016, 20));
        let bench2 = dd(18, 0, 0, 4032, 0, 0);
        assert_eq!((bench2.add_fr_x, bench2.add_fr_y), (-4032, 40));
        assert_eq!(bench2.height, 16);
        // 139 music boxes (TileDrawing.cs:5019-5026)
        let music = dd(139, 0, 0, 0, 2016, 0);
        assert_eq!((music.top, music.add_fr_x, music.add_fr_y), (2, 72, -2016));
        // pile echoes: 1908 / 1890 strides
        let piles1 = dd(649, 0, 0, 1908, 0, 0);
        assert_eq!(
            (piles1.top, piles1.add_fr_x, piles1.add_fr_y),
            (2, -1908, 18)
        );
        let piles2 = dd(648, 0, 0, 1890, 0, 0);
        assert_eq!(
            (piles2.top, piles2.add_fr_x, piles2.add_fr_y),
            (2, -1890, 36)
        );
        let small = dd(185, 0, 0, 1908, 18, 0);
        assert_eq!((small.top, small.add_fr_x, small.add_fr_y), (2, -1908, 18));
        let large = dd(187, 0, 0, 1890, 0, 0);
        assert_eq!((large.top, large.add_fr_x, large.add_fr_y), (2, -1890, 36));
        // 185 only strides on the `tileFrameY == 18` row (TileDrawing.cs:5158-5163)
        let no_stride = dd(185, 0, 0, 1908, 0, 0);
        assert_eq!((no_stride.add_fr_x, no_stride.add_fr_y), (0, 0));
        // 186 is top-2 only (the rest is tileShine2, not ported)
        assert_eq!(dd(186, 0, 0, 972, 0, 0), TileDrawData { top: 2, ..DEFAULT });
    }

    #[test]
    fn test_DW_1_4_banner_parity() {
        // Firefly-in-a-bottle family: chain mod 6 (TileDrawing.cs:5535-5563)
        let expect270 = [18, 90, 0, 72, 90, 0, 18, 36]; // x = 1..=8, tf 0
        for kind in [270, 271, 581] {
            for x in 1..=8i32 {
                let d = dd(kind, x, 0, 0, 0, 0);
                assert_eq!(d.add_fr_x, expect270[(x - 1) as usize], "kind {kind} x {x}");
                assert_eq!(d.add_fr_y, 0, "kind {kind} x {x}");
                assert_eq!(d.flip, x % 2 == 0, "kind {kind} x {x}");
                // the while-chain is periodic in tile_frame mod 6
                let d6 = dd(kind, x, 0, 0, 0, 6);
                assert_eq!(d6.add_fr_x, d.add_fr_x, "kind {kind} x {x} tf 6");
            }
        }
        // Shimmerfly-in-a-bottle: chain mod 5 (TileDrawing.cs:5564-5590)
        let expect660 = [18, 0, 18, 0, 0]; // x = 1..=5
        for x in 1..=5i32 {
            let d = dd(660, x, 0, 0, 0, 0);
            assert_eq!(d.add_fr_x, expect660[(x - 1) as usize], "x {x}");
            assert_eq!(d.add_fr_y, 0);
            assert_eq!(dd(660, x, 0, 0, 0, 5).add_fr_x, d.add_fr_x, "x {x} tf 5");
        }
        // SoulBottles: chain mod 4 (TileDrawing.cs:5591-5604)
        let expect572 = [18, 36, 54, 0]; // x = 1..=4
        for x in 1..=4i32 {
            let d = dd(572, x, 0, 0, 0, 0);
            assert_eq!(d.add_fr_x, expect572[(x - 1) as usize], "x {x}");
            assert_eq!(d.add_fr_y, 0);
            assert_eq!(d.flip, x % 2 == 0, "x {x}");
            assert_eq!(dd(572, x, 0, 0, 0, 4).add_fr_x, d.add_fr_x, "x {x} tf 4");
        }
    }

    #[test]
    fn test_DW_1_4_frame_overrides() {
        // 751: `tileFrameY += (x + y * 2) % 7 * 46` (TileDrawing.cs:4550)
        let a = dd(751, 3, 2, 0, 100, 0);
        assert_eq!(a.frame_y_override, Some(100)); // (3 + 4) % 7 == 0
        assert_eq!((a.width, a.height), (56, 46));
        assert_eq!(dd(751, 5, 2, 0, 100, 0).frame_y_override, Some(192)); // (5+4)%7 == 2
        assert_eq!(dd(751, 6, 3, 0, 0, 0).frame_y_override, Some(230)); // 12 % 7 == 5
        assert_eq!(dd(751, 3, 2, 0, 100, 0).frame_x_override, None);
        // 726: both frames pinned to the sheet origin (TileDrawing.cs:4556-4561)
        let dummy = dd(726, 0, 0, 48, 36, 0);
        assert_eq!(
            (dummy.frame_x_override, dummy.frame_y_override),
            (Some(0), Some(0))
        );
        assert_eq!((dummy.width, dummy.height), (20, 20));
        // 698: frame X remapped through /18*38 (TileDrawing.cs:4588-4592)
        assert_eq!(dd(698, 0, 0, 0, 0, 0).frame_x_override, Some(0));
        assert_eq!(dd(698, 0, 0, 18, 0, 0).frame_x_override, Some(38));
        assert_eq!(dd(698, 0, 0, 36, 0, 0).frame_x_override, Some(76));
        let jar = dd(698, 0, 0, 18, 0, 0);
        assert_eq!((jar.width, jar.height), (36, 44));
        assert_eq!(jar.frame_y_override, None);
        // 617: frames fold into the 54x144 block (TileDrawing.cs:5276-5280)
        let trophy = dd(617, 0, 0, 60, 150, 0);
        assert_eq!(
            (trophy.frame_x_override, trophy.frame_y_override, trophy.top),
            (Some(6), Some(6), 2)
        );
        assert_eq!(dd(617, 0, 0, 18, 144, 0).frame_y_override, Some(0));
        // 733: addFrX 54 below row 54, checked BEFORE the fold
        // (TileDrawing.cs:5982-5990)
        let mon1 = dd(733, 0, 0, 72, 36, 0);
        assert_eq!(
            (
                mon1.add_fr_x,
                mon1.frame_x_override,
                mon1.frame_y_override,
                mon1.top
            ),
            (54, Some(18), Some(36), 2)
        );
        let mon2 = dd(733, 0, 0, 18, 54, 0);
        assert_eq!(
            (
                mon2.add_fr_x,
                mon2.frame_x_override,
                mon2.frame_y_override,
                mon2.top
            ),
            (0, Some(18), Some(0), 2)
        );
    }

    #[test]
    fn test_DW_1_2_campfire_epilogue() {
        // Campfires = {215} (TileID.cs:616); the epilogue overrides the
        // animation row after the switch (TileDrawing.cs:6033-6044).
        let lit = dd(215, 0, 0, 0, 0, 0);
        assert_eq!(
            (lit.width, lit.height, lit.top, lit.add_fr_y),
            (16, 16, 2, 0)
        );
        // frameY >= 36 pins addFrY to the extinguished row, tileFrame or not.
        let off = dd(215, 0, 0, 0, 36, 3);
        assert_eq!((off.top, off.add_fr_y), (2, 252));
        // frameY < 36 animates: addFrY = tileFrame * 36.
        let anim = dd(215, 0, 0, 0, 18, 2);
        assert_eq!((anim.top, anim.add_fr_y), (2, 72));
        // Dirty: 216 is not in the Campfires set and keeps the global default.
        let not_campfire = dd(216, 0, 0, 0, 36, 3);
        assert_eq!((not_campfire.top, not_campfire.add_fr_y), (0, 114));
    }

    #[test]
    fn test_DW_1_4_tile_frame_passthrough() {
        // The animation-row formulas are ported verbatim over `tile_frame`
        // (= Main.tileFrame[kind]); 0 today, live when the animator lands.
        assert_eq!(dd(106, 0, 0, 0, 0, 2).add_fr_y, 108); // * 54
        assert_eq!(dd(106, 0, 0, 0, 0, 5).add_fr_y, 270);
        for kind in [12, 31, 96, 639, 665, 696] {
            // * 36
            assert_eq!(dd(kind, 0, 0, 0, 0, 3).add_fr_y, 108, "kind {kind}");
        }
        let bulb = dd(238, 0, 0, 0, 0, 5); // top 2, * 36
        assert_eq!((bulb.top, bulb.add_fr_y), (2, 180));
        let extr = dd(219, 0, 0, 0, 0, 4); // top 2, * 54
        assert_eq!((extr.top, extr.add_fr_y), (2, 216));
        // Kinds outside the switch keep the global default * 38.
        assert_eq!(dd(1, 0, 0, 0, 0, 3).add_fr_y, 114);
        // Dirty: 564's glow-switch reset (TileDrawing.cs:6105-6113) zeroes the
        // row that 217/218 keep.
        assert_eq!(dd(564, 0, 0, 0, 0, 3).add_fr_y, 0);
        assert_eq!(dd(564, 0, 0, 0, 0, 3).top, 2);
        assert_eq!(dd(217, 0, 0, 0, 0, 3).add_fr_y, 108);
        assert_eq!(dd(218, 0, 0, 0, 0, 3).add_fr_y, 108);
    }

    #[test]
    fn test_DW_1_1_group_lists_complete() {
        // `case 78`-group: top 2 (TileDrawing.cs:4680-4696)
        for kind in [
            78, 85, 133, 134, 173, 210, 233, 254, 283, 378, 457, 466, 520, 651, 652,
        ] {
            let d = dd(kind, 0, 0, 0, 0, 0);
            assert_eq!(
                (d.width, d.height, d.top, d.flip),
                (16, 16, 2, false),
                "kind {kind}"
            );
        }
        // 18 px group (TileDrawing.cs:4880-4908)
        for kind in [
            16, 17, 26, 32, 69, 72, 77, 124, 137, 138, 352, 462, 487, 488, 574, 575, 576, 577, 578,
            664, 695, 704, 712, 713, 714, 715, 716,
        ] {
            let d = dd(kind, 0, 0, 0, 0, 0);
            assert_eq!((d.width, d.height, d.top), (16, 18, 0), "kind {kind}");
        }
        // Vine family (TileDrawing.cs:5000-5013) — covered in
        // vines_family_is_raised_and_flipped; here just the count parity.
        // `case 28`-group: top 2 (TileDrawing.cs:5258-5275)
        for kind in [
            28, 105, 470, 475, 506, 547, 548, 552, 560, 597, 613, 621, 622, 623, 653, 699,
        ] {
            let d = dd(kind, 0, 0, 0, 0, 0);
            assert_eq!((d.width, d.height, d.top), (16, 16, 2), "kind {kind}");
        }
        // Cage groups: top 2, no animation offset (TileDrawing.cs:5629-5907)
        const CAGES: &[u16] = &[
            275, 276, 277, 278, 279, 280, 281, 296, 297, 309, 358, 359, 413, 414, 542, 550, 551,
            553, 554, 558, 559, 599, 600, 601, 602, 603, 604, 605, 606, 607, 608, 609, 610, 611,
            612, 632, 640, 643, 644, 645, 710, 285, 286, 298, 299, 310, 339, 361, 362, 363, 364,
            391, 392, 393, 394, 532, 533, 538, 544, 555, 556, 582, 619, 629, 282, 505, 543, 598,
            568, 569, 570, 288, 289, 290, 291, 292, 293, 294, 295, 360, 580, 620, 521, 522, 523,
            524, 525, 526, 527, 316, 317, 318,
        ];
        assert_eq!(CAGES.len(), 92);
        for kind in CAGES.iter().copied() {
            let d = dd(kind, 0, 0, 0, 0, 0);
            assert_eq!(
                (d.width, d.height, d.top, d.add_fr_y),
                (16, 16, 2, 0),
                "kind {kind}"
            );
        }
        // Dirty: ids of neighbouring arms do not give the other arm's metric.
        assert_eq!(dd(79, 0, 0, 0, 0, 0).top, 0); // beds != 78-group
        assert_eq!(dd(53, 0, 0, 0, 0, 0), DEFAULT); // outside the switch
        assert_eq!(dd(27, 0, 0, 0, 0, 0).top, 0); // sunflower != 28-group
        assert_eq!(dd(269, 0, 0, 0, 0, 0), DEFAULT); // != 270 parity chain
        assert_eq!(
            (dd(753, 0, 0, 0, 0, 0).width, dd(753, 0, 0, 0, 0, 0).top),
            (16, 2)
        );
    }

    #[test]
    fn test_DW_1_1_starred_static_skeletons() {
        // 349 MushroomStatue: top 2 only; the GetTemporaryFrame frame rewrite
        // is a documented deviation (TileDrawing.cs:5332-5341)
        let statue = dd(349, 0, 0, 18, 0, 0);
        assert_eq!((statue.top, statue.width, statue.height), (2, 16, 16));
        assert_eq!(statue.frame_x_override, None);
        // 441/468 FakeContainers: height on frameY == 18; the frameY rewrite
        // is a documented deviation (TileDrawing.cs:5342-5356)
        assert_eq!(dd(441, 0, 0, 0, 18, 0).height, 18);
        assert_eq!(dd(441, 0, 0, 0, 0, 0).height, 16);
        assert_eq!(dd(468, 0, 0, 0, 18, 0).height, 18);
        assert_eq!(dd(468, 0, 0, 0, 0, 0).frame_y_override, None);
        // 593 VolcanoSmall: addFrX -18 on later columns; static addFrY
        // fallback `tileFrame * 18` below 18 else 0 (TileDrawing.cs:5445-5465)
        let v1 = dd(593, 0, 0, 18, 0, 0);
        assert_eq!((v1.add_fr_x, v1.add_fr_y, v1.top), (-18, 0, 2));
        let v2 = dd(593, 0, 0, 0, 0, 2);
        assert_eq!((v2.add_fr_x, v2.add_fr_y, v2.top), (0, 36, 2));
        // 594 VolcanoLarge: same shape with 36 (TileDrawing.cs:5466-5487)
        let v3 = dd(594, 0, 0, 36, 0, 0);
        assert_eq!((v3.add_fr_x, v3.add_fr_y, v3.top), (-36, 0, 2));
        let v4 = dd(594, 0, 0, 0, 0, 2);
        assert_eq!((v4.add_fr_x, v4.add_fr_y, v4.top), (0, 72, 2));
        // 493 GolfCupFlag: top 2 only; the wind animation is a deviation
        // (TileDrawing.cs:5231-5257)
        let flag = dd(493, 0, 0, 0, 0, 0);
        assert_eq!((flag.top, flag.height), (2, 16));
        // 711 RainbowBoulder: 18 wide on later columns, 20 px tall; the glow
        // out-params are TILE-DRAW-2 (TileDrawing.cs:4931-4939)
        let rainbow = dd(711, 0, 0, 18, 0, 0);
        assert_eq!((rainbow.width, rainbow.height), (18, 20));
        let plain = dd(711, 0, 0, 0, 0, 0);
        assert_eq!((plain.width, plain.height), (16, 20));
        // 323 palm: 20x20 static part; the biome frame row lands next phase
        // (TileDrawing.cs:4667-4671)
        let palm = dd(323, 0, 0, 0, 0, 0);
        assert_eq!(
            (palm.width, palm.height, palm.top, palm.frame_y_override),
            (20, 20, 0, None)
        );
    }

    #[test]
    fn test_DW_1_1_static_switch_sweep() {
        let t = |kind: u16, fx: i16, fy: i16| dd(kind, 0, 0, fx, fy, 0);
        // Sizes / tops of the single-kind cases.
        let clock = t(752, 0, 0);
        assert_eq!((clock.width, clock.height, clock.top), (36, 38, 2));
        let hole = t(476, 0, 0);
        assert_eq!((hole.width, hole.height), (20, 18));
        assert_eq!(t(443, 0, 0).top, 2);
        assert_eq!(t(443, 36, 0).top, 2);
        assert_eq!(t(443, 72, 0).top, -2);
        let bamboo = t(571, 0, 0);
        assert_eq!((bamboo.top, bamboo.flip), (2, true));
        assert_eq!(t(136, 0, 0).top, 2);
        assert_eq!(t(136, 18, 0).top, 0);
        let column = t(561, 0, 18);
        assert_eq!((column.top, column.height, column.add_fr_y), (-2, 20, 4));
        assert_eq!(t(561, 0, 54).add_fr_y, 12);
        assert_eq!(t(561, 0, 0).add_fr_y, 0);
        for kind in [330, 331, 332, 333] {
            assert_eq!(t(kind, 0, 0).top, 2, "kind {kind}");
        }
        // 129 crystals: addFrY cleared; the giant-crystal chain
        // (TileDrawing.cs:4637-4644)
        assert_eq!(t(129, 0, 0).add_fr_y, 0);
        assert_eq!(t(129, 324, 0).add_fr_x, 0);
        assert_eq!(t(129, 342, 0).add_fr_x, 0);
        assert_eq!(t(129, 450, 0).add_fr_x, -108);
        // Tree variants: 20x20 (dead in the app, ported for completeness)
        for kind in [583, 596, 616, 634] {
            let d = t(kind, 0, 0);
            assert_eq!((d.width, d.height), (20, 20), "kind {kind}");
        }
        // Top-2 singles.
        for kind in [705, 494, 650, 647, 706, 89, 102, 753, 36] {
            assert_eq!(t(kind, 0, 0).top, 2, "kind {kind}");
            assert_eq!(t(kind, 0, 0).height, 16, "kind {kind}");
        }
        for kind in [132, 135] {
            let d = t(kind, 0, 0);
            assert_eq!((d.top, d.height), (2, 18), "kind {kind}");
        }
        // 388/389 tall gates: seam rows and tops (TileDrawing.cs:5037-5051)
        assert_eq!((t(388, 0, 0).height, t(388, 0, 0).top), (18, -2));
        assert_eq!((t(388, 0, 74).height, t(388, 0, 74).top), (18, 0));
        assert_eq!((t(388, 0, 94).height, t(388, 0, 94).top), (18, -2));
        assert_eq!((t(389, 0, 168).height, t(389, 0, 168).top), (18, 0));
        assert_eq!((t(389, 0, 40).height, t(389, 0, 40).top), (16, 0));
        // 624/700, 656/701, 567, 519
        let abigail = t(624, 0, 0);
        assert_eq!(
            (abigail.width, abigail.height, abigail.top, abigail.flip),
            (20, 16, 2, true)
        );
        let tulip = t(656, 0, 0);
        assert_eq!(
            (tulip.width, tulip.height, tulip.top, tulip.flip),
            (24, 34, -16, true)
        );
        let gnome = t(567, 0, 0);
        assert_eq!(
            (gnome.width, gnome.height, gnome.top, gnome.flip),
            (26, 18, -2, true)
        );
        assert_eq!(t(567, 0, 18).top, 0);
        assert_eq!((t(519, 0, 0).top, t(519, 0, 0).flip), (2, true));
        // 178 / 184
        assert_eq!(t(178, 0, 36).top, 2);
        assert_eq!(t(178, 0, 54).top, 0);
        assert_eq!(t(184, 0, 0).width, 20);
        assert_eq!(t(184, 0, 36).top, 2);
        assert_eq!(t(184, 0, 54).top, -2);
        assert_eq!(t(184, 0, 108).top, -2);
        assert_eq!(t(184, 0, 144).top, 0);
        // 81 coral
        let coral = t(81, 0, 0);
        assert_eq!(
            (coral.width, coral.height, coral.top, coral.flip),
            (24, 26, -8, true)
        );
        // 442 projectile pad variants by frameX / 22 (TileDrawing.cs:5612-5628)
        assert_eq!(
            (t(442, 0, 0).width, t(442, 0, 0).height, t(442, 0, 0).top),
            (20, 20, 0)
        );
        assert_eq!(t(442, 22, 0).top, -4);
        let pad2 = t(442, 44, 0);
        assert_eq!((pad2.width, pad2.top), (24, -2));
        assert_eq!(t(442, 66, 0).top, -2);
        assert_eq!(t(442, 88, 0).top, 0);
        // 428 weighted plate: top 4, never pressed in the port
        assert_eq!(t(428, 0, 0).top, 4);
        // 272 cog: explicit zero even when the animator runs
        assert_eq!(dd(272, 0, 0, 0, 0, 2).add_fr_y, 0);
        // 614 tendrils: animation row lives in addFrX
        let tendril = dd(614, 0, 0, 0, 0, 2);
        assert_eq!(
            (tendril.add_fr_x, tendril.add_fr_y, tendril.top),
            (108, 0, 2)
        );
        // Animation-row groups at tile_frame = 2.
        for kind in [
            326, 327, 328, 329, 345, 351, 421, 422, 458, 459, 708, 379, 739, 748,
        ] {
            assert_eq!(dd(kind, 0, 0, 0, 0, 2).add_fr_y, 180, "kind {kind}");
        }
        for kind in [336, 340, 341, 342, 343, 344] {
            let d = dd(kind, 0, 0, 0, 0, 2);
            assert_eq!((d.add_fr_y, d.top), (180, 2), "kind {kind}");
        }
        for kind in [300, 304, 308, 354, 355, 499] {
            let d = dd(kind, 0, 0, 0, 0, 2);
            assert_eq!((d.add_fr_y, d.top), (108, 2), "kind {kind}");
        }
        for kind in [463, 464] {
            let d = dd(kind, 0, 0, 0, 0, 2);
            assert_eq!((d.add_fr_y, d.top), (144, 2), "kind {kind}");
        }
        let sharp = dd(377, 0, 0, 0, 0, 2);
        assert_eq!((sharp.add_fr_y, sharp.top), (76, 2));
        for kind in [228, 231, 243, 247] {
            let d = dd(kind, 0, 0, 0, 0, 2);
            assert_eq!((d.add_fr_y, d.top), (108, 2), "kind {kind}");
        }
        assert_eq!(dd(592, 0, 0, 0, 0, 2).add_fr_y, 108);
        assert_eq!(dd(454, 0, 0, 0, 0, 2).add_fr_y, 108);
        assert_eq!(dd(390, 0, 0, 0, 0, 2).add_fr_y, 72);
        assert_eq!(dd(235, 0, 0, 0, 0, 2).add_fr_y, 36);
        // 244 / 565: only the first frame column animates
        assert_eq!(dd(244, 0, 0, 0, 0, 2).add_fr_y, 72);
        assert_eq!(dd(244, 0, 0, 54, 0, 2).add_fr_y, 0);
        assert_eq!(dd(565, 0, 0, 0, 0, 2).add_fr_y, 72);
        assert_eq!(dd(565, 0, 0, 36, 0, 2).add_fr_y, 0);
        // 452 / 405 / 455: frameX interplay
        assert_eq!(dd(452, 0, 0, 0, 0, 2).add_fr_y, 108);
        assert_eq!(dd(452, 0, 0, 54, 0, 2).add_fr_y, 0);
        let fireplace = dd(405, 0, 0, 0, 0, 2);
        assert_eq!((fireplace.height, fireplace.add_fr_y), (16, 76));
        assert_eq!(dd(405, 0, 0, 0, 18, 2).height, 18);
        assert_eq!(dd(405, 0, 0, 54, 0, 2).add_fr_y, 0);
        let party = dd(455, 0, 0, 0, 0, 2);
        assert_eq!((party.top, party.add_fr_y), (2, 0));
        // 406 chimney bands (TileDrawing.cs:5367-5386)
        let ch0 = dd(406, 0, 0, 0, 0, 0);
        assert_eq!((ch0.height, ch0.add_fr_y), (16, 0));
        let ch1 = dd(406, 0, 0, 0, 36, 0);
        assert_eq!((ch1.height, ch1.add_fr_y), (18, 0));
        let ch2 = dd(406, 0, 0, 0, 54, 0);
        assert_eq!((ch2.height, ch2.add_fr_y), (16, -54));
        let ch3 = dd(406, 0, 0, 0, 90, 0);
        assert_eq!((ch3.height, ch3.add_fr_y), (18, -54));
        let ch4 = dd(406, 0, 0, 0, 108, 0);
        assert_eq!((ch4.height, ch4.add_fr_y), (16, 228));
        // 410 lunar monolith
        assert_eq!(dd(410, 0, 0, 0, 36, 0).height, 18);
        assert_eq!(dd(410, 0, 0, 0, 0, 0).add_fr_y, 0);
        assert_eq!(dd(410, 0, 0, 0, 56, 2).add_fr_y, 112);
        assert_eq!(dd(410, 0, 0, 0, 36, 2).add_fr_y, 0);
        // 480-group monoliths: row * 54 only from frameY >= 54
        for kind in [480, 509, 657, 720, 721, 725] {
            let d0 = dd(kind, 0, 0, 0, 0, 0);
            assert_eq!((d0.top, d0.add_fr_y), (2, 0), "kind {kind}");
            let d1 = dd(kind, 0, 0, 0, 54, 2);
            assert_eq!((d1.top, d1.add_fr_y), (2, 108), "kind {kind}");
        }
        // 658 shimmer monolith bands (TileDrawing.cs:5962-5981)
        assert_eq!(dd(658, 0, 0, 0, 0, 1).add_fr_y, 54);
        assert_eq!(dd(658, 0, 0, 0, 54, 1).add_fr_y, 540);
        assert_eq!(dd(658, 0, 0, 0, 108, 1).add_fr_y, 1026);
        assert_eq!(dd(658, 0, 0, 0, 162, 1).add_fr_y, 54);
        // 207 water fountain (TileDrawing.cs:5908-5929)
        assert_eq!(dd(207, 0, 0, 0, 36, 0).add_fr_y, 0);
        assert_eq!(dd(207, 5, 0, 0, 72, 0).add_fr_y, 360);
        assert_eq!(dd(207, 6, 0, 0, 72, 0).add_fr_y, 0);
        assert_eq!(dd(207, 7, 0, 18, 72, 0).add_fr_y, 0); // fx%36!=0 shifts x
        assert_eq!(dd(207, 5, 0, 0, 72, 1).add_fr_y, 0); // 1+5 -> >= 6 -> 0
        assert_eq!(dd(207, 4, 0, 0, 72, 1).add_fr_y, 360); // 1+4 == 5
                                                           // 129 at tile_frame 1: (n40 + tf) % 6 walks the variants
        assert_eq!(dd(129, 0, 0, 324, 0, 1).add_fr_x, 18);
        assert_eq!(dd(129, 0, 0, 450, 0, 1).add_fr_x, -90); // (7+1)%6=2 -> 2-7=-5*18=-90
    }

    #[test]
    fn test_DW_1_1_deviation_kinds_stay_default() {
        // §3B kinds without an unconditional static part keep the 16x16
        // default at t=0 (documented deviations; every one cites its C#
        // branch in drawdata.rs).
        for kind in [
            453, 456, 485, 507, 508, 518, 529, 530, 489, 490, 719, 541, 579, 5,
        ] {
            assert_eq!(dd(kind, 3, 7, 90, 144, 0), DEFAULT, "kind {kind}");
        }
    }

    #[test]
    fn i16_domain_extremes_do_not_panic() {
        // fx/fy at the i16 extremes: arithmetic stays in i32, the final cast
        // wraps exactly like C#'s (short) cast.
        for kind in [
            79, 18, 100, 90, 139, 406, 452, 129, 207, 751, 617, 733, 698, 726, 270, 660, 572, 215,
            442, 658,
        ] {
            let d = dd(kind, 5, 3, 32767, 32767, 3);
            let _ = d;
        }
        // The wrap matches C#'s (short) cast for the additive 751.
        let wrapped = dd(751, 5, 2, 0, 32767, 0);
        assert_eq!(wrapped.frame_y_override, Some((32767i32 + 92) as i16));
        // Extreme strides stay inside i16 (32767 / 2016 == 16).
        let bed = dd(79, 0, 0, 0, 32767, 0);
        assert_eq!(bed.add_fr_y, -32256);
        let bench = dd(18, 0, 0, 32767, 0, 0);
        assert_eq!(bench.add_fr_x, -32256);
        // Zero frames everywhere: plain defaults.
        for kind in [79u16, 100, 90, 18, 139, 649, 648, 185, 187] {
            let d = dd(kind, 0, 0, 0, 0, 0);
            assert_eq!((d.add_fr_x, d.add_fr_y), (0, 0), "kind {kind}");
        }
    }

    // ---- Phase 2: biome helpers and world corrections ----

    use crate::tile::FLAG_ACTIVE;

    /// Places an active tile of `kind` (the `torch.rs` test pattern).
    fn put(w: &mut World, x: i32, y: i32, kind: u16) {
        let i = w.index(x, y);
        let mut t = crate::tile::Tile::default();
        t.kind = kind;
        t.flags |= FLAG_ACTIVE;
        w.tiles[i] = t;
    }

    #[test]
    fn test_DW_2_1_palm_biome() {
        // `GetPalmTreeVariant` (TileDrawing.cs:3491-3515): the ground under the
        // trunk picks the style — 53 sand → 0, 234 crimsand → 1, 116 pearlsand
        // → 2, 112 ebonsand → 3.
        for (ground, expect) in [(53u16, 0i32), (234, 1), (116, 2), (112, 3)] {
            let mut w = World::new("t".into(), 64, 64);
            for y in 30..34 {
                put(&mut w, 20, y, 323);
            }
            put(&mut w, 20, 34, ground);
            // The walk starts on any trunk cell and stops on the first
            // non-palm tile (TileDrawing.cs:1725-1733).
            assert_eq!(palm_tree_biome(&w, 20, 30), expect, "ground {ground} top");
            assert_eq!(palm_tree_biome(&w, 20, 33), expect, "ground {ground} base");
        }
        // Non-sand ground: variant -1 (the C# frame row would be -22).
        let mut w = World::new("t".into(), 64, 64);
        for y in 30..34 {
            put(&mut w, 20, y, 323);
        }
        put(&mut w, 20, 34, 2);
        assert_eq!(palm_tree_biome(&w, 20, 30), -1);
        // Regular sand far from the beaches: biome 0 (no oasis bonus).
        put(&mut w, 20, 34, 53);
        assert_eq!(palm_tree_biome(&w, 20, 30), 0);
        // An inactive tile stops the walk like a non-palm tile would.
        w.tile_mut(20, 32).unwrap().flags &= !FLAG_ACTIVE;
        assert_eq!(palm_tree_biome(&w, 20, 30), -1);
    }

    #[test]
    fn test_DW_2_1_palm_oasis_offset() {
        // `WorldGen.IsPalmOasisTree` (WorldGen.cs:27205-27210) adds 4 inside
        // `x >= 380 && x <= width - 380` (`beachDistance = 380`,
        // WorldGen.cs:86914).
        let mut w = World::new("t".into(), 1000, 40);
        for y in 10..14 {
            put(&mut w, 500, y, 323);
        }
        put(&mut w, 500, 14, 53);
        assert_eq!(palm_tree_biome(&w, 500, 10), 4, "oasis center");
        // Exact band edges are inside.
        for x in [380i32, 620i32] {
            for y in 10..14 {
                put(&mut w, x, y, 323);
            }
            put(&mut w, x, 14, 53);
            assert_eq!(palm_tree_biome(&w, x, 10), 4, "x {x}");
        }
        // Left of the band and one past the right edge: no bonus.
        for x in [200i32, 621i32] {
            for y in 10..14 {
                put(&mut w, x, y, 323);
            }
            put(&mut w, x, 14, 53);
            assert_eq!(palm_tree_biome(&w, x, 10), 0, "x {x}");
        }
        // The bonus also applies to the -1 ground variant (faithful C# `num += 4`).
        let mut w2 = World::new("t".into(), 1000, 40);
        for y in 10..14 {
            put(&mut w2, 500, y, 323);
        }
        put(&mut w2, 500, 14, 2);
        assert_eq!(palm_tree_biome(&w2, 500, 10), 3);
    }

    #[test]
    fn test_DW_2_2_cactus_type() {
        // The walk goes through the non-solid cactus (Main.tileSolid[80] is
        // false) to the ground; the sand family sets exactly one flag
        // (WorldGen.cs:47730-47736).
        for (ground, expect) in [
            (53u16, (false, false, false)),
            (112, (true, false, false)),
            (116, (false, true, false)),
            (234, (false, false, true)),
        ] {
            let mut w = World::new("t".into(), 64, 64);
            for y in 30..33 {
                put(&mut w, 30, y, 80);
            }
            put(&mut w, 30, 33, ground);
            assert_eq!(
                cactus_type(&w, 30, 32, 0, 0),
                expect,
                "ground {ground} base"
            );
            assert_eq!(cactus_type(&w, 30, 30, 0, 0), expect, "ground {ground} top");
        }
        // A dye plant (227) probing through the cactus below it: the flag is
        // only set once the walk reaches the cactus segment.
        let mut w = World::new("t".into(), 64, 64);
        put(&mut w, 30, 30, 227);
        put(&mut w, 30, 31, 80);
        put(&mut w, 30, 32, 234);
        assert_eq!(cactus_type(&w, 30, 30, 204, 0), (false, false, true));
    }

    #[test]
    fn test_DW_2_2_cactus_frame_columns() {
        // Side arms shift the probe column (WorldGen.cs:47694-47701):
        // frameX 36 → x-1, 54 → x+1, 108 → x+1 (frameY != 18) / x-1 (18).
        let mut w = World::new("t".into(), 64, 64);
        for x in 0..64 {
            put(&mut w, x, 40, 53);
        }
        put(&mut w, 30, 40, 112); // the only corrupt column
        put(&mut w, 30, 39, 80);
        assert_eq!(cactus_type(&w, 30, 39, 0, 0), (true, false, false));
        assert_eq!(
            cactus_type(&w, 31, 39, 36, 0),
            (true, false, false),
            "36 → x-1"
        );
        assert_eq!(
            cactus_type(&w, 29, 39, 54, 0),
            (true, false, false),
            "54 → x+1"
        );
        assert_eq!(
            cactus_type(&w, 29, 39, 108, 0),
            (true, false, false),
            "108, frameY != 18 → x+1"
        );
        assert_eq!(
            cactus_type(&w, 31, 39, 108, 18),
            (true, false, false),
            "108, frameY == 18 → x-1"
        );
        // Dirty: columns that do not resolve to 30 stay on plain sand.
        assert_eq!(cactus_type(&w, 28, 39, 0, 0), (false, false, false));
        assert_eq!(cactus_type(&w, 32, 39, 54, 0), (false, false, false));
        assert_eq!(cactus_type(&w, 29, 39, 108, 18), (false, false, false));
    }

    #[test]
    fn test_DW_2_2_cactus_walk_limit() {
        // The walk gives up 20 rows below the start (WorldGen.cs:47724).
        let mut w = World::new("t".into(), 64, 80);
        put(&mut w, 30, 10, 80);
        put(&mut w, 30, 30, 112); // exactly y + 20: still reached
        assert_eq!(cactus_type(&w, 30, 10, 0, 0), (true, false, false));
        let mut w2 = World::new("t".into(), 64, 80);
        put(&mut w2, 30, 10, 80);
        put(&mut w2, 30, 31, 112); // y + 21: the walk broke earlier
        assert_eq!(cactus_type(&w2, 30, 10, 0, 0), (false, false, false));
        // Vanilla quirk: the final check runs on the tile where the walk
        // broke, even when no cactus was ever seen — the corrupt sand at
        // y + 20 still reports evil.
        let mut w3 = World::new("t".into(), 64, 80);
        put(&mut w3, 30, 30, 112);
        assert_eq!(cactus_type(&w3, 30, 10, 0, 0), (true, false, false));
    }

    #[test]
    fn test_DW_2_3_wiring_arithmetic() {
        // Palm rows (TileDrawing.cs:4670): frame_y = 22 * biome.
        assert_eq!(palm_frame_y(0), 0);
        assert_eq!(palm_frame_y(1), 22);
        assert_eq!(palm_frame_y(2), 44);
        assert_eq!(palm_frame_y(3), 66);
        // Dye plants (TileDrawing.cs:5066-5077).
        assert_eq!(dye_frame_x_offset(false, true, false), 238);
        assert_eq!(dye_frame_x_offset(true, false, false), 204);
        assert_eq!(dye_frame_x_offset(false, false, true), 272);
        assert_eq!(dye_frame_x_offset(false, false, false), 0);
        // Cactus source-Y (TileDrawing.cs:1133-1146).
        assert_eq!(cactus_source_y_offset(true, false, false), 54);
        assert_eq!(cactus_source_y_offset(false, true, false), 108);
        assert_eq!(cactus_source_y_offset(false, false, true), 162);
        assert_eq!(cactus_source_y_offset(false, false, false), 0);
        // Cog parity (TileDrawing.cs:1126-1129) at t=0:
        // (x%2 + y%2 + x%3 + y%3) % 2 * 90.
        assert_eq!(cog_source_y_offset(0, 0, 0), 0);
        assert_eq!(cog_source_y_offset(1, 1, 0), 0); // 1+1+1+1
        assert_eq!(cog_source_y_offset(2, 2, 0), 0); // 0+0+2+2
        assert_eq!(cog_source_y_offset(3, 0, 0), 90); // 1+0+0+0
        assert_eq!(cog_source_y_offset(0, 3, 0), 90);
        assert_eq!(cog_source_y_offset(5, 0, 0), 90); // 1+0+2+0
                                                      // The animation row participates when the animator runs.
        assert_eq!(cog_source_y_offset(0, 0, 1), 90);
        assert_eq!(cog_source_y_offset(1, 1, 2), 0);
    }

    #[test]
    fn test_DW_2_3_apply_world_corrections() {
        // 323: the stored frame row is replaced by 22 * biome
        // (TileDrawing.cs:4670).
        let mut w = World::new("t".into(), 64, 64);
        for y in 30..34 {
            put(&mut w, 20, y, 323);
        }
        put(&mut w, 20, 34, 112);
        let mut palm = dd(323, 20, 30, 18, 0, 0);
        palm.apply_world_corrections(&w, 323, 20, 30, 18, 0, 0);
        assert_eq!(palm.frame_y_override, Some(66), "ebonsand → row 66");
        assert_eq!(palm.frame_x_override, None);
        // 227 with frameX 204: rewritten by the ground sand (5063-5078).
        let mut w2 = World::new("t".into(), 64, 64);
        put(&mut w2, 30, 30, 227);
        put(&mut w2, 30, 31, 80);
        put(&mut w2, 30, 32, 234);
        let mut dye = dd(227, 30, 30, 204, 0, 0);
        dye.apply_world_corrections(&w2, 227, 30, 30, 204, 0, 0);
        assert_eq!(dye.frame_x_override, Some(204 + 272), "crimson → +272");
        // 227 with frameX != 204: no frame mutation (plan edge case).
        for fx in [0i16, 238] {
            let mut dye = dd(227, 30, 30, fx, 0, 0);
            dye.apply_world_corrections(&w2, 227, 30, 30, fx, 0, 0);
            assert_eq!(dye.frame_x_override, None, "frameX {fx}");
        }
        // 80: source-Y shift on top of the cleared animation row; other
        // fields untouched (TileDrawing.cs:1132-1147).
        let mut w3 = World::new("t".into(), 64, 64);
        put(&mut w3, 30, 30, 80);
        put(&mut w3, 30, 31, 116);
        let mut cac = dd(80, 30, 30, 0, 0, 0);
        cac.apply_world_corrections(&w3, 80, 30, 30, 0, 0, 0);
        assert_eq!(cac.add_fr_y, 108, "pearlsand → +108");
        assert_eq!(cac.top, 2);
        // Dirty: 142/143 are pumps sharing only the top-2 arm — the
        // `DrawSingleTile` correction is `case 80:` only.
        for kind in [142u16, 143] {
            let mut pump = dd(kind, 30, 30, 0, 0, 0);
            pump.apply_world_corrections(&w3, kind, 30, 30, 0, 0, 0);
            assert_eq!(pump.add_fr_y, 0, "kind {kind}");
            assert_eq!(pump.top, 2, "kind {kind}");
        }
        // 272: the parity shift lands on add_fr_y (the arm cleared it).
        let mut cog = dd(272, 3, 0, 0, 0, 0);
        cog.apply_world_corrections(&w3, 272, 3, 0, 0, 0, 0);
        assert_eq!(cog.add_fr_y, 90);
        // Kinds outside the correction set are untouched.
        let mut sand = dd(53, 30, 30, 0, 0, 0);
        let before = sand;
        sand.apply_world_corrections(&w3, 53, 30, 30, 0, 0, 0);
        assert_eq!(sand, before);
    }

    #[test]
    fn test_DW_2_4_world_boundaries() {
        // `InWorld` fluff of 2 (WorldGen.cs:9688-9695): probes in the 2-tile
        // rim are rejected up front — neutral (false, false, false), no panic.
        let mut w = World::new("t".into(), 64, 64);
        for x in 0..64 {
            put(&mut w, x, 40, 112);
        }
        for x in [0i32, 1, 62, 63] {
            put(&mut w, x, 39, 80);
            assert_eq!(cactus_type(&w, x, 39, 0, 0), (false, false, false), "x {x}");
        }
        // Probes at the world bottom rim (y >= height-2): same neutral exit.
        put(&mut w, 30, 61, 80);
        assert_eq!(cactus_type(&w, 30, 61, 0, 0), (false, false, false));
        put(&mut w, 30, 63, 80);
        assert_eq!(cactus_type(&w, 30, 63, 0, 0), (false, false, false));
        // x = 0 / x = W-1 probes that stay inside the rim check still work
        // through `world.tile`'s None (palm walk off the world bottom).
        let mut w2 = World::new("t".into(), 64, 64);
        for y in 60..64 {
            put(&mut w2, 20, y, 323);
        }
        assert_eq!(palm_tree_biome(&w2, 20, 60), -1, "walk past the bottom");
        let mut w3 = World::new("t".into(), 64, 64);
        for y in 60..63 {
            put(&mut w3, 0, y, 323);
        }
        put(&mut w3, 0, 63, 53);
        assert_eq!(palm_tree_biome(&w3, 0, 60), 0, "palm at x = 0");
        let mut w4 = World::new("t".into(), 64, 64);
        for y in 60..63 {
            put(&mut w4, 63, y, 323);
        }
        put(&mut w4, 63, 63, 53);
        assert_eq!(palm_tree_biome(&w4, 63, 60), 0, "palm at x = W-1");
        // Palm walk stopping on the last in-world row (ground at y = H-1).
        let mut w5 = World::new("t".into(), 64, 64);
        for y in 60..63 {
            put(&mut w5, 30, y, 323);
        }
        put(&mut w5, 30, 63, 234);
        assert_eq!(palm_tree_biome(&w5, 30, 60), 1, "crimsand at the last row");
    }

    #[test]
    fn test_DW_2_4_cactus_no_ground() {
        // No solid tile within 20 rows: (false, false, false) → no shift.
        let mut w = World::new("t".into(), 64, 80);
        put(&mut w, 30, 10, 80);
        assert_eq!(cactus_type(&w, 30, 10, 0, 0), (false, false, false));
        // An entirely empty column too (the walk runs out of rows).
        assert_eq!(cactus_type(&w, 40, 10, 0, 0), (false, false, false));
        // Only inactive (actuated-off style) tiles below: same.
        let i = w.index(50, 30);
        w.tiles[i].kind = 112;
        assert_eq!(cactus_type(&w, 50, 10, 0, 0), (false, false, false));
    }

    #[test]
    fn corrections_i16_extremes_do_not_panic() {
        // Frame extremes flow through the corrections in the i32 domain.
        let mut w = World::new("t".into(), 64, 64);
        put(&mut w, 30, 39, 80);
        put(&mut w, 30, 40, 53);
        for fx in [0i16, 36, 54, 108, 204, 32767, -32768] {
            let mut cac = dd(80, 30, 39, fx, 32767, 0);
            cac.apply_world_corrections(&w, 80, 30, 39, fx, 32767, 0);
            let mut dye = dd(227, 30, 39, fx, 32767, 0);
            dye.apply_world_corrections(&w, 227, 30, 39, fx, 32767, 0);
        }
        // A palm trunk over corrupt sand: the biome row stays inside i16.
        for y in 20..24 {
            put(&mut w, 40, y, 323);
        }
        put(&mut w, 40, 24, 112);
        let mut palm = dd(323, 40, 20, 32767, 32767, 0);
        palm.apply_world_corrections(&w, 323, 40, 20, 32767, 32767, 0);
        assert_eq!(palm.frame_y_override, Some(66));
        // A probe column outside the world is neutral (variant -1 → row -22).
        let mut palm2 = dd(323, 9999, 20, 0, 0, 0);
        palm2.apply_world_corrections(&w, 323, 9999, 20, 0, 0, 0);
        assert_eq!(palm2.frame_y_override, Some(-22));
    }
}
