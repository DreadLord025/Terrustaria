//! Parser for Terraria `.wld` files (format version 87+, sections-based v2).
//!
//! Mirrors `Terraria.IO.WorldFile.LoadWorld_Version2` / `LoadWorldTiles` from the
//! original sources. Only the parts needed by the renderer are decoded in full;
//! later sections (chests, NPCs, bestiary...) are skipped via the section table.

use std::path::Path;

use anyhow::{anyhow, bail, Context, Result};

use crate::tile::*;
use crate::world::World;

const FILE_METADATA_SIZE: usize = 20;

pub fn load(path: &Path) -> Result<World> {
    let data = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    parse(&data).with_context(|| format!("parsing {}", path.display()))
}

pub fn parse(data: &[u8]) -> Result<World> {
    let mut r = Reader::new(data);

    // --- file format header -------------------------------------------------
    let version = r.i32()?;
    if version < 87 {
        bail!("unsupported world format {version} (expected >= 87)");
    }
    if version >= 135 {
        r.skip(FILE_METADATA_SIZE)?;
    }

    let section_count = r.i16()? as usize;
    let mut sections = Vec::with_capacity(section_count);
    for _ in 0..section_count {
        sections.push(r.i32()? as usize);
    }

    let tile_type_count = r.u16()? as usize;
    let mut importance = vec![false; tile_type_count];
    let mut bit: u8 = 0;
    let mut mask: u8 = 128;
    for i in 0..tile_type_count {
        if mask == 128 {
            bit = r.u8()?;
            mask = 1;
        } else {
            mask <<= 1;
        }
        importance[i] = bit & mask == mask;
    }

    // --- header section ----------------------------------------------------
    if sections.is_empty() {
        bail!("world has no section table");
    }
    r.seek(sections[0])?;
    let name = r.string()?;
    // M5-SEED-2: the seed TEXT rides the `.wld` header (`WorldFile.cs:1959`)
    // and is the C#'s canonical micro-seed store — `LoadWorldFlags` writes no
    // micro bytes at all, `WorldFileData.SetSeed` re-derives them on load.
    // The port mirrors that: parse the `|`-separated secret plaintexts into
    // `World.micro_seeds` (the classic booleans still come from the flags
    // block below, so the two sources never disagree).
    let mut seed_text = String::new();
    if version >= 179 {
        seed_text = r.string()?;
        let _generator_version = r.u64()?;
    }
    let (micro_seeds, _, _) = crate::seeds::resolve_seed_text(&seed_text);
    if version >= 181 {
        r.skip(16)?; // GUID
    }
    let _world_id = r.i32()?;
    let _left = r.i32()?;
    let _right = r.i32()?;
    let _top = r.i32()?;
    let _bottom = r.i32()?;
    let height = r.i32()?;
    let width = r.i32()?; // yes: Y before X in the file
    if width <= 0 || height <= 0 || width > 20_000 || height > 20_000 {
        bail!("implausible world size {width}x{height}");
    }

    let mut world = World::new(name, width, height);
    world.micro_seeds = micro_seeds;
    load_header_flags(&mut r, &mut world, version)?;

    // --- tile section ------------------------------------------------------
    if sections.len() < 2 {
        bail!("world has no tile section");
    }
    r.seek(sections[1])?;
    load_tiles(&mut r, &mut world, &importance, version)?;

    world.frame_important = importance;
    world.compute_spawn_y();
    Ok(world)
}

fn load_header_flags(r: &mut Reader, world: &mut World, version: i32) -> Result<()> {
    // WorldFile.LoadWorldFlags: mostly data we skip, but it also carries the spawn
    // point and the world layers, which the lighting needs.
    if version >= 209 {
        r.skip(4)?; // GameMode
        for (min_version, flag) in [
            (222, crate::seeds::SeedFlags::DRUNK),
            (227, crate::seeds::SeedFlags::FOR_THE_WORTHY),
            (238, crate::seeds::SeedFlags::TENTH_ANNIVERSARY),
            (239, crate::seeds::SeedFlags::DONT_STARVE),
            (241, crate::seeds::SeedFlags::NOT_THE_BEES),
            (249, crate::seeds::SeedFlags::REMIX),
            (266, crate::seeds::SeedFlags::NO_TRAPS),
            (267, crate::seeds::SeedFlags::ZENITH),
            (302, crate::seeds::SeedFlags::SKYBLOCK),
        ] {
            // `WorldFile.LoadWorldFlags` (`WorldFile.cs:1981-2041`) reads the
            // nine `Main.*` seed booleans in the `SaveWorldFlags` order
            // (`:1238-1250`), each gated on its introduction version — the
            // port kept them as `skip(1)` while no flag existed to fill.
            if version >= min_version && r.u8()? != 0 {
                world.seeds.insert(flag);
            }
        }
    } else {
        r.skip(1)?; // GameMode as bool
        if version == 208 {
            r.skip(1)?;
        }
    }
    // The pre-267 back-compat inference (`WorldFile.cs:2020`): a world that
    // carries remix && drunk but has no zenith byte is a get-fixed-boi world.
    if version < 267
        && world.seeds.contains(crate::seeds::SeedFlags::REMIX)
        && world.seeds.contains(crate::seeds::SeedFlags::DRUNK)
    {
        world.seeds.insert(
            crate::seeds::SeedFlags::ZENITH | crate::seeds::SeedFlags::EVERYTHING_DEPS,
        );
    }
    if version >= 141 {
        r.skip(8)?; // creation time
    }
    if version >= 284 {
        r.skip(8)?; // last played
    }
    world.moon_type = r.u8()?.min(8); // moon type
    r.skip(3 * 4)?; // treeX
    r.skip(4 * 4)?; // treeStyle
    r.skip(3 * 4)?; // caveBackX
    r.skip(4 * 4)?; // caveBackStyle
    r.skip(3 * 4)?; // ice / jungle / hell back style

    world.spawn_x = r.i32()?;
    world.spawn_y = r.i32()?;
    world.world_surface = r.f64()?;
    world.rock_layer = r.f64()?;
    world.time = r.f64()?;
    world.day_time = r.u8()? != 0;
    // `WorldFile._tempMoonPhase` follows `_tempDayTime` (`WorldFile.cs:2074`).
    world.moon_phase = (r.i32()? as u8) & 7;
    Ok(())
}

fn load_tiles(r: &mut Reader, world: &mut World, importance: &[bool], version: i32) -> Result<()> {
    let width = world.width;
    let height = world.height;

    for x in 0..width {
        let mut y = 0;
        while y < height {
            let mut tile = Tile {
                frame_x: -1,
                frame_y: -1,
                ..Default::default()
            };

            let b4 = r.u8()?;
            let mut b1 = 0u8;
            let mut b2 = 0u8;
            let mut b3 = 0u8;

            if b4 & 1 != 0 {
                b1 = r.u8()?;
            }
            if b1 & 1 != 0 {
                b2 = r.u8()?;
            }
            if b2 & 1 != 0 {
                b3 = r.u8()?;
            }

            if b4 & 2 != 0 {
                tile.flags |= FLAG_ACTIVE;
                let kind = if b4 & 0x20 != 0 {
                    let lo = r.u8()? as i32;
                    let hi = r.u8()? as i32;
                    (hi << 8) | lo
                } else {
                    r.u8()? as i32
                };
                tile.kind = kind as u16;

                if importance.get(kind as usize).copied().unwrap_or(false) {
                    tile.frame_x = r.i16()?;
                    tile.frame_y = r.i16()?;
                    if tile.kind == 144 {
                        tile.frame_y = 0;
                    }
                }
                if b2 & 8 != 0 {
                    let paint = r.u8()?;
                    tile.set_paint(paint);
                }
            }

            if b4 & 4 != 0 {
                tile.wall = r.u8()? as u16;
                if b2 & 0x10 != 0 {
                    // `WorldFile.cs:2589-2592`: `tile.wallColor(reader.ReadByte())`
                    // — the wall paint lives in the sparse `World.extras` byte
                    // (`Tile` keeps 12 bytes; see `World::set_wall_paint`).
                    let wall_paint = r.u8()?;
                    if wall_paint != 0 {
                        world.set_wall_paint(x, y, wall_paint);
                    }
                }
            }

            let liquid_kind = (b4 & 0x18) >> 3;
            if liquid_kind != 0 {
                tile.liquid = r.u8()?;
                match liquid_kind {
                    1 => {}
                    2 => tile.flags |= FLAG_LIQUID_LAVA,
                    _ => tile.flags |= FLAG_LIQUID_HONEY,
                }
                if b2 & 0x80 != 0 {
                    tile.flags |= FLAG_LIQUID_SHIMMER;
                }
            }

            if b1 > 1 {
                let slope = (b1 & 0x70) >> 4;
                if slope == 1 {
                    tile.flags |= FLAG_HALF_BRICK;
                } else if slope > 1 {
                    tile.set_slope(slope - 1);
                }
            }

            if b2 > 1 {
                if b2 & 2 != 0 {
                    tile.flags |= FLAG_ACTUATOR;
                }
                if b2 & 4 != 0 {
                    tile.flags |= FLAG_INACTIVE;
                }
                if b2 & 0x40 != 0 {
                    let hi = r.u8()? as u16;
                    tile.wall |= hi << 8;
                }
            }

            if b3 > 1 {
                if b3 & 8 != 0 {
                    tile.flags |= FLAG_FULLBRIGHT;
                }
                // `WorldFile.cs:2664-2682`: the Echo/Glow wall coatings.
                if b3 & 2 != 0 {
                    world.set_invisible_block(x, y, true);
                }
                if b3 & 4 != 0 {
                    world.set_invisible_wall(x, y, true);
                }
                if b3 & 0x10 != 0 {
                    world.set_fullbright_wall(x, y, true);
                }
            }

            if version >= 105 {
                // nothing to fix up for v0.1 renderer
            }

            let repeat = (b4 & 0xC0) >> 6;
            let extra = match repeat {
                0 => 0,
                1 => r.u8()? as i32,
                _ => r.i16()? as i32,
            };
            if extra < 0 {
                bail!("negative RLE run at ({x}, {y})");
            }

            let end = (y + extra).min(height - 1);
            while y <= end {
                let i = world.index(x, y);
                world.tiles[i] = tile;
                y += 1;
            }
        }
    }

    Ok(())
}

/// Thin `anyhow` adapter over `crate::binio::Reader`: a short read becomes the
/// parser's original `unexpected end of file at offset N` bail. All decoding
/// lives in `crate::binio`.
struct Reader<'a>(crate::binio::Reader<'a>);

fn eof(pos: usize) -> anyhow::Error {
    anyhow!("unexpected end of file at offset {pos}")
}

impl<'a> Reader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self(crate::binio::Reader::new(data))
    }

    fn seek(&mut self, pos: usize) -> Result<()> {
        self.0
            .seek(pos)
            .ok_or_else(|| anyhow!("section pointer {pos} beyond EOF"))
    }

    fn skip(&mut self, n: usize) -> Result<()> {
        self.0.skip(n).ok_or_else(|| eof(self.0.pos()))
    }

    fn u8(&mut self) -> Result<u8> {
        self.0.u8().ok_or_else(|| eof(self.0.pos()))
    }

    fn i16(&mut self) -> Result<i16> {
        self.0.i16().ok_or_else(|| eof(self.0.pos()))
    }

    fn u16(&mut self) -> Result<u16> {
        self.0.u16().ok_or_else(|| eof(self.0.pos()))
    }

    fn i32(&mut self) -> Result<i32> {
        self.0.i32().ok_or_else(|| eof(self.0.pos()))
    }

    fn u64(&mut self) -> Result<u64> {
        self.0.u64().ok_or_else(|| eof(self.0.pos()))
    }

    fn f64(&mut self) -> Result<f64> {
        self.0.f64().ok_or_else(|| eof(self.0.pos()))
    }

    /// .NET `BinaryReader.ReadString`: 7-bit length prefix + UTF-8 payload,
    /// decoded lossily as before.
    fn string(&mut self) -> Result<String> {
        self.0.cs_string_lossy().ok_or_else(|| eof(self.0.pos()))
    }
}
