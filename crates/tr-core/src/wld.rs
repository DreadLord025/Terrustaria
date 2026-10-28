//! Parser for Terraria `.wld` files (format version 87+, sections-based v2).
//!
//! Mirrors `Terraria.IO.WorldFile.LoadWorld_Version2` / `LoadWorldTiles` from the
//! original sources. Only the parts needed by the renderer are decoded in full;
//! later sections (chests, NPCs, bestiary...) are skipped via the section table.

use std::path::Path;

use anyhow::{Context, Result, bail};

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
    if version >= 179 {
        let _seed = r.string()?;
        let _generator_version = r.u64()?;
    }
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
        for min_version in [222, 227, 238, 239, 241, 249, 266, 267, 302] {
            if version >= min_version {
                r.skip(1)?;
            }
        }
    } else {
        r.skip(1)?; // GameMode as bool
        if version == 208 {
            r.skip(1)?;
        }
    }
    if version >= 141 {
        r.skip(8)?; // creation time
    }
    if version >= 284 {
        r.skip(8)?; // last played
    }
    r.skip(1)?; // moon type
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
                    r.u8()?; // wall paint
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

            if b3 > 1 && b3 & 8 != 0 {
                tile.flags |= FLAG_FULLBRIGHT;
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

struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn need(&self, n: usize) -> Result<()> {
        if self.pos + n > self.data.len() {
            bail!("unexpected end of file at offset {}", self.pos);
        }
        Ok(())
    }

    fn seek(&mut self, pos: usize) -> Result<()> {
        if pos > self.data.len() {
            bail!("section pointer {pos} beyond EOF");
        }
        self.pos = pos;
        Ok(())
    }

    fn skip(&mut self, n: usize) -> Result<()> {
        self.need(n)?;
        self.pos += n;
        Ok(())
    }

    fn u8(&mut self) -> Result<u8> {
        self.need(1)?;
        let v = self.data[self.pos];
        self.pos += 1;
        Ok(v)
    }

    fn i16(&mut self) -> Result<i16> {
        self.need(2)?;
        let v = i16::from_le_bytes([self.data[self.pos], self.data[self.pos + 1]]);
        self.pos += 2;
        Ok(v)
    }

    fn u16(&mut self) -> Result<u16> {
        Ok(self.i16()? as u16)
    }

    fn i32(&mut self) -> Result<i32> {
        self.need(4)?;
        let v = i32::from_le_bytes(self.data[self.pos..self.pos + 4].try_into().unwrap());
        self.pos += 4;
        Ok(v)
    }

    fn u64(&mut self) -> Result<u64> {
        self.need(8)?;
        let v = u64::from_le_bytes(self.data[self.pos..self.pos + 8].try_into().unwrap());
        self.pos += 8;
        Ok(v)
    }

    fn f64(&mut self) -> Result<f64> {
        self.need(8)?;
        let v = f64::from_le_bytes(self.data[self.pos..self.pos + 8].try_into().unwrap());
        self.pos += 8;
        Ok(v)
    }

    /// .NET `BinaryReader.ReadString`: 7-bit length prefix + UTF-8 payload.
    fn string(&mut self) -> Result<String> {
        let mut len = 0usize;
        let mut shift = 0;
        loop {
            let b = self.u8()?;
            len |= ((b & 0x7F) as usize) << shift;
            if b & 0x80 == 0 {
                break;
            }
            shift += 7;
            if shift > 28 {
                bail!("bad string length prefix");
            }
        }
        self.need(len)?;
        let s = String::from_utf8_lossy(&self.data[self.pos..self.pos + len]).into_owned();
        self.pos += len;
        Ok(s)
    }
}
