//! `.map` v326 persistence.
//!
//! Ports `MapHelper.InternalSaveMap` (`MapHelper.cs:3030-3322`) and
//! `MapHelper.LoadMapVersion2` (`:3749-4027`). The file is one contiguous stream:
//! an uncompressed header, the uncompressed option tables, then the row RLE body
//! wrapped in a raw deflate stream (`3124`, `3929`). `tr-core` owns every byte-
//! layout primitive; the deflate wrapping and file IO live in `tr-app`, which is
//! the only crate that depends on `flate2`.
//!
//! Not ported: `Main.ActivePlayerFileData.MapFileName` / cloud save paths,
//! `Main.statusText` progress text, `WorldMap.ClearEdges`, and the persistent
//! `MapFileMetadata` revision counter — a fresh metadata is written, so the
//! revision is always `1` and the output is deterministic.

use crate::mapdata::{TILE_COUNT, WALL_COUNT};
use crate::world::World;

use super::colors::color_table;
use super::MapTile;

/// `MapHelper` map format version for 1.4.5 (`MapHelper.cs:3050`).
pub const MAP_FORMAT_VERSION: u32 = 326;

/// `FileMetadata.MAGIC_NUMBER` (`FileMetadata.cs:8`).
const FILE_METADATA_MAGIC: u64 = 0x0063_6967_6F6C_6572;
/// `(1u64 << 56)` mask for the low 6 bytes (`FileMetadata.cs:59`).
const FILE_METADATA_MAGIC_MASK: u64 = 0x00FF_FFFF_FFFF_FFFF;
/// `FileType.Map` (`FileMetadata.cs:24`).
const FILE_TYPE_MAP: u64 = 1;

/// Fixed prefix (`InternalSaveMap` 3050-3061).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MapHeader {
    pub version: u32,
    pub seed: i32,
    pub height: i32,
    pub width: i32,
    pub tile_count: i16,   // TileID.Count, written
    pub wall_count: i16,   // WallID.Count
    pub liquid_count: i16, // 4
    pub sky_count: i16,    // 256
    pub dirt_count: i16,   // 256
    pub rock_count: i16,   // 256
}

/// The whole uncompressed prefix (`LoadMapVersion2` 3749-3838).
#[derive(Clone, Debug)]
pub struct MapPrefix {
    pub header: MapHeader,
    pub world_name: String,
    pub tile_options: Vec<u8>, // `array3`: len == tile_count
    pub wall_options: Vec<u8>, // `array4`: len == wall_count
    pub body_offset: usize,    // where the deflate stream starts
}

/// `BinaryWriter.Write(string)`: 7-bit-encoded byte length, then UTF-8.
fn write_cs_string(out: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    let mut len = bytes.len();
    while len >= 0x80 {
        out.push(((len & 0x7f) | 0x80) as u8);
        len >>= 7;
    }
    out.push(len as u8);
    out.extend_from_slice(bytes);
}

/// `InternalSaveMap` 3050-3061 — version, `FileMetadata`, name, seed, sizes, six i16.
pub fn write_map_header(world: &World) -> Vec<u8> {
    let mut out = Vec::with_capacity(64);
    out.extend_from_slice(&MAP_FORMAT_VERSION.to_le_bytes());
    // `Main.MapFileMetadata.IncrementAndWrite` from a fresh metadata (revision 1)
    // then `FileMetadata.Write`: magic|type, revision, favourite flag.
    out.extend_from_slice(&(FILE_METADATA_MAGIC | (FILE_TYPE_MAP << 56)).to_le_bytes());
    out.extend_from_slice(&1u32.to_le_bytes());
    out.extend_from_slice(&0u64.to_le_bytes());
    write_cs_string(&mut out, &world.name);
    out.extend_from_slice(&(world.seed as i32).to_le_bytes());
    out.extend_from_slice(&world.height.to_le_bytes());
    out.extend_from_slice(&world.width.to_le_bytes());
    for count in [
        TILE_COUNT as i16,
        WALL_COUNT as i16,
        4,
        256,
        256,
        256,
    ] {
        out.extend_from_slice(&count.to_le_bytes());
    }
    out
}

/// `InternalSaveMap` 3062-3109: one LSB-first bitmap, flushed every 8 bits.
fn write_option_bitmap(out: &mut Vec<u8>, counts: &[u8]) {
    let mut bit: u8 = 1;
    let mut acc: u8 = 0;
    for &c in counts {
        if c != 1 {
            acc |= bit;
        }
        if bit == 128 {
            out.push(acc);
            acc = 0;
            bit = 1;
        } else {
            bit <<= 1;
        }
    }
    if bit != 1 {
        out.push(acc);
    }
}

/// `InternalSaveMap` 3062-3123 — option bitmaps + the non-default option counts.
pub fn write_option_tables() -> Vec<u8> {
    let table = color_table();
    let mut out = Vec::new();
    write_option_bitmap(&mut out, &table.tile_option_counts);
    write_option_bitmap(&mut out, &table.wall_option_counts);
    for &c in &table.tile_option_counts {
        if c != 1 {
            out.push(c);
        }
    }
    for &c in &table.wall_option_counts {
        if c != 1 {
            out.push(c);
        }
    }
    out
}

/// `InternalSaveMap` 3125-3318 — the per-row RLE body, before deflate.
///
/// The source stages bytes in a 16384-byte `array` flushed to the `DeflateStream`
/// once `num >= 4096`; since every flush is appended to the same stream the
/// concatenated bytes are identical to appending directly here.
pub fn encode_tiles(map: &[MapTile], width: usize, height: usize) -> Vec<u8> {
    let table = color_table();
    let mut out = Vec::new();
    for j in 0..height {
        let mut x = 0usize;
        while x < width {
            let map_tile = map[j * width + x];
            let mut b4 = 0u8;
            let b5 = 0u8; // `3138`: never assigned in the source
            let mut b6 = 0u8;
            let mut ty = 0u16;
            let mut run = 0usize;
            let mut flag = true; // write the type
            let mut flag2 = true; // write the light
            let mut run_start = 0usize;
            let mut run_end = 0usize;
            let category: u8;

            if map_tile.light <= 18 {
                // `3147-3162`: an unrevealed skip-run.
                flag2 = false;
                flag = false;
                category = 0;
                let mut n9 = x + 1;
                let mut remaining = width - x - 1;
                while remaining > 0 && map[j * width + n9].light <= 18 {
                    run += 1;
                    remaining -= 1;
                    n9 += 1;
                }
            } else {
                b6 = map_tile.color();
                ty = map_tile.ty;
                if ty < table.wall_position {
                    category = 1;
                    ty = ty.wrapping_sub(table.tile_position);
                } else if ty < table.liquid_position {
                    category = 2;
                    ty = ty.wrapping_sub(table.wall_position);
                } else if ty < table.sky_position {
                    let mut liquid = ty.wrapping_sub(table.liquid_position);
                    if liquid == 3 {
                        b4 |= 0x40; // shimmer rides the water category
                        liquid = 0;
                    }
                    category = 3 + liquid as u8;
                    flag = false;
                } else if ty < table.dirt_position {
                    category = 6;
                    flag2 = false;
                    flag = false;
                } else if ty < table.hell_position {
                    category = 7;
                    ty = if ty >= table.rock_position {
                        ty.wrapping_sub(table.rock_position)
                    } else {
                        ty.wrapping_sub(table.dirt_position)
                    };
                } else {
                    category = 6;
                    flag = false;
                }
                if map_tile.light == 255 {
                    flag2 = false;
                }
                if flag2 {
                    // `3208-3226`: light != 255 keeps `EqualsWithoutLight` runs.
                    run = 0;
                    let mut n9 = x + 1;
                    let mut remaining = width - x - 1;
                    run_start = n9;
                    while remaining > 0 {
                        if map_tile.equals_without_light(&map[j * width + n9]) {
                            remaining -= 1;
                            run += 1;
                            n9 += 1;
                            continue;
                        }
                        break;
                    }
                    // The decompiled `num6 = 0` default would leave the run end at 0
                    // when the run reaches the last column and drop the run's light
                    // bytes, but `LoadMapVersion2:4017-4023` unconditionally reads
                    // `num27` lights whenever `light != 255`. End the run at
                    // `run_start + run` (exactly the C# `num6` whenever a mismatch
                    // sets it) so the reader stays in sync.
                    run_end = run_start + run;
                } else {
                    // `3227-3238`: light == 255 keeps exact `Equals` runs.
                    run = 0;
                    let mut n9 = x + 1;
                    let mut remaining = width - x - 1;
                    while remaining > 0 && map_tile.equals_all(&map[j * width + n9]) {
                        remaining -= 1;
                        run += 1;
                        n9 += 1;
                    }
                }
            }

            if b6 > 0 {
                b4 |= b6 << 1;
            }
            if b5 != 0 {
                b4 |= 1;
            }
            let mut b3 = 0u8;
            if b4 != 0 {
                b3 |= 1; // bit0: a second header byte follows
            }
            b3 |= category << 1;
            if flag && ty > 255 {
                b3 |= 0x10; // bit4: 16-bit type
            }
            if flag2 {
                b3 |= 0x20; // bit5: a light byte follows
            }
            if run > 0 {
                b3 |= if run <= 255 { 0x40 } else { 0x80 };
            }
            out.push(b3);
            if b4 != 0 {
                out.push(b4);
            }
            if b5 != 0 {
                out.push(b5);
            }
            if flag {
                out.push(ty as u8);
                if ty > 255 {
                    out.push((ty >> 8) as u8);
                }
            }
            if flag2 {
                out.push(map_tile.light);
            }
            if run > 0 {
                out.push(run as u8);
                if run > 255 {
                    out.push((run >> 8) as u8);
                }
            }
            // `3302-3306`: the run tiles' own light values.
            for k in run_start..run_end {
                out.push(map[j * width + k].light);
            }
            x += run + 1;
        }
    }
    out
}

fn eof() -> std::io::Error {
    std::io::Error::new(
        std::io::ErrorKind::UnexpectedEof,
        "truncated .map body",
    )
}

/// A cursor over the uncompressed prefix; returns `None` past the end.
struct PrefixReader<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> PrefixReader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }
    fn u8(&mut self) -> Option<u8> {
        let b = *self.bytes.get(self.pos)?;
        self.pos += 1;
        Some(b)
    }
    fn u16(&mut self) -> Option<u16> {
        Some(self.u8()? as u16 | ((self.u8()? as u16) << 8))
    }
    fn i16(&mut self) -> Option<i16> {
        self.u16().map(|v| v as i16)
    }
    fn u32(&mut self) -> Option<u32> {
        let mut v = 0u32;
        for i in 0..4 {
            v |= (self.u8()? as u32) << (8 * i);
        }
        Some(v)
    }
    fn i32(&mut self) -> Option<i32> {
        self.u32().map(|v| v as i32)
    }
    fn u64(&mut self) -> Option<u64> {
        let mut v = 0u64;
        for i in 0..8 {
            v |= (self.u8()? as u64) << (8 * i);
        }
        Some(v)
    }
    fn cs_string(&mut self) -> Option<String> {
        let mut len = 0usize;
        let mut shift = 0u32;
        loop {
            let b = self.u8()?;
            len |= ((b & 0x7f) as usize) << shift;
            if b & 0x80 == 0 {
                break;
            }
            shift += 7;
            if shift > 28 {
                return None;
            }
        }
        let end = self.pos.checked_add(len)?;
        let bytes = self.bytes.get(self.pos..end)?;
        self.pos = end;
        String::from_utf8(bytes.to_vec()).ok()
    }
}

/// `LoadMapVersion2` 3773-3810: one `count`-bit LSB-first bitmap.
fn read_option_bitmap(r: &mut PrefixReader, count: usize) -> Option<Vec<bool>> {
    let mut set = vec![false; count];
    let mut b = 0u8;
    let mut bit = 128u8;
    for slot in set.iter_mut() {
        if bit == 128 {
            b = r.u8()?;
            bit = 1;
        } else {
            bit <<= 1;
        }
        if b & bit == bit {
            *slot = true;
        }
    }
    Some(set)
}

/// `LoadMapVersion2` 3811-3838: `array3`/`array4`, one count per set bitmap bit.
fn read_option_counts(r: &mut PrefixReader, set: &[bool]) -> Option<Vec<u8>> {
    let mut options = Vec::with_capacity(set.len());
    for &is_set in set {
        options.push(if is_set { r.u8()? } else { 1 });
    }
    Some(options)
}

/// Reads the header + option tables. `Ok(None)` on a short / foreign / wrong-size
/// prefix (never panics).
pub fn read_map_prefix(bytes: &[u8]) -> std::io::Result<Option<MapPrefix>> {
    let mut r = PrefixReader::new(bytes);
    let Some(version) = r.u32() else {
        return Ok(None);
    };
    if version != MAP_FORMAT_VERSION {
        return Ok(None);
    }
    let Some(metadata) = r.u64() else {
        return Ok(None);
    };
    if metadata & FILE_METADATA_MAGIC_MASK != FILE_METADATA_MAGIC {
        return Ok(None);
    }
    if (metadata >> 56) & 0xFF != FILE_TYPE_MAP {
        return Ok(None);
    }
    let Some(_revision) = r.u32() else {
        return Ok(None);
    };
    let Some(_favourite) = r.u64() else {
        return Ok(None);
    };
    let Some(world_name) = r.cs_string() else {
        return Ok(None);
    };
    let Some(seed) = r.i32() else {
        return Ok(None);
    };
    let Some(height) = r.i32() else {
        return Ok(None);
    };
    let Some(width) = r.i32() else {
        return Ok(None);
    };
    let mut counts = [0i16; 6];
    for count in &mut counts {
        let Some(v) = r.i16() else {
            return Ok(None);
        };
        *count = v;
    }
    let [tile_count, wall_count, liquid_count, sky_count, dirt_count, rock_count] = counts;
    // Same-version files always carry 754/367/4/256/256/256. Reject any negative
    // count: `decode_tiles` sizes its offset tables from these, and a negative i16
    // cast to `usize` would panic in `vec![...]` with a capacity overflow.
    if counts.iter().any(|&c| c < 0) {
        return Ok(None);
    }
    if tile_count as usize != TILE_COUNT || wall_count as usize != WALL_COUNT {
        return Ok(None);
    }
    // `LoadMapVersion2` reads *both* bitmaps first, then both count lists.
    let Some(tile_set) = read_option_bitmap(&mut r, tile_count as usize) else {
        return Ok(None);
    };
    let Some(wall_set) = read_option_bitmap(&mut r, wall_count as usize) else {
        return Ok(None);
    };
    let Some(tile_options) = read_option_counts(&mut r, &tile_set) else {
        return Ok(None);
    };
    let Some(wall_options) = read_option_counts(&mut r, &wall_set) else {
        return Ok(None);
    };
    Ok(Some(MapPrefix {
        header: MapHeader {
            version,
            seed,
            height,
            width,
            tile_count,
            wall_count,
            liquid_count,
            sky_count,
            dirt_count,
            rock_count,
        },
        world_name,
        tile_options,
        wall_options,
        body_offset: r.pos,
    }))
}

/// A bounds-checked cursor over the inflated body; short reads become `Err`.
struct BodyReader<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> BodyReader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }
    fn u8(&mut self) -> std::io::Result<u8> {
        let b = *self.bytes.get(self.pos).ok_or_else(eof)?;
        self.pos += 1;
        Ok(b)
    }
    fn u16(&mut self) -> std::io::Result<u16> {
        Ok(self.u8()? as u16 | ((self.u8()? as u16) << 8))
    }
}

/// `LoadMapVersion2` 3930-4025 — decodes the already-inflated body.
///
/// `world_surface` / `rock_layer` are `Main.worldSurface` / `Main.rockLayer`,
/// which the source reads as globals in the sky (`3999`) and dirt/rock (`4002`)
/// cases; they are passed in so `tr-core` stays engine-agnostic.
#[allow(clippy::too_many_arguments)]
pub fn decode_tiles(
    body: &[u8],
    header: &MapHeader,
    tile_options: &[u8],
    wall_options: &[u8],
    width: usize,
    height: usize,
    world_surface: f64,
    rock_layer: f64,
) -> std::io::Result<Vec<MapTile>> {
    let table = color_table();
    let n4 = header.tile_count as usize;
    let n5 = header.wall_count as usize;
    let n6 = header.liquid_count as usize;
    let n7 = header.sky_count as usize;
    let n8 = header.dirt_count as usize;
    let n9 = header.rock_count as usize;

    // `array5` remap (`3839-3928`): file index -> our `colorLookup` index.
    let num10: usize = tile_options.iter().map(|&c| c as usize).sum();
    let num11: usize = wall_options.iter().map(|&c| c as usize).sum();
    let mut array5 = vec![0u16; num10 + num11 + n6 + n7 + n8 + n9 + 2];
    let mut num12: u16 = 1;
    let mut num13: usize = 1;
    let num14 = num13 as u16; // tile region start
    for i in 0..TILE_COUNT {
        if i < n4 {
            let file_count = tile_options.get(i).copied().unwrap_or(0) as usize;
            let our_count = table.tile_option_counts[i] as usize;
            for j in 0..our_count {
                if j < file_count {
                    *array5.get_mut(num13).ok_or_else(eof)? = num12;
                    num13 += 1;
                }
                num12 += 1;
            }
        } else {
            num12 += table.tile_option_counts.get(i).copied().unwrap_or(0) as u16;
        }
    }
    let num17 = num13 as u16; // wall region start
    for i in 0..WALL_COUNT {
        if i < n5 {
            let file_count = wall_options.get(i).copied().unwrap_or(0) as usize;
            let our_count = table.wall_option_counts[i] as usize;
            for j in 0..our_count {
                if j < file_count {
                    *array5.get_mut(num13).ok_or_else(eof)? = num12;
                    num13 += 1;
                }
                num12 += 1;
            }
        } else {
            num12 += table.wall_option_counts.get(i).copied().unwrap_or(0) as u16;
        }
    }
    let num20 = num13 as u16; // liquid region start
    for i in 0..4 {
        if i < n6 {
            *array5.get_mut(num13).ok_or_else(eof)? = num12;
            num13 += 1;
        }
        num12 += 1;
    }
    let num21 = num13 as u16; // sky region start
    for i in 0..256 {
        if i < n7 {
            *array5.get_mut(num13).ok_or_else(eof)? = num12;
            num13 += 1;
        }
        num12 += 1;
    }
    let num22 = num13 as u16; // dirt region start
    for i in 0..256 {
        if i < n8 {
            *array5.get_mut(num13).ok_or_else(eof)? = num12;
            num13 += 1;
        }
        num12 += 1;
    }
    let num23 = num13 as u16; // rock region start
    for i in 0..256 {
        if i < n9 {
            *array5.get_mut(num13).ok_or_else(eof)? = num12;
            num13 += 1;
        }
        num12 += 1;
    }
    let num24 = num13 as u16; // hell position
    *array5.get_mut(num13).ok_or_else(eof)? = num12;

    let mut reader = BodyReader::new(body);
    let mut map = vec![MapTile::default(); width * height];
    for l in 0..height {
        let mut m = 0usize;
        // The C# `for (m = 0; m < maxTilesX; m++)`; run tiles advance `m` inside
        // the body and the loop increment adds the final `+ 1` (kept as a `while`).
        while m < width {
            let b3 = reader.u8()?;
            let mut b4 = 0u8;
            if b3 & 1 == 1 {
                b4 = reader.u8()?;
            }
            if b4 & 1 == 1 {
                // Legacy format carried a third header byte; never written by us.
                let _ = reader.u8()?;
            }
            let category = (b3 & 0x0E) >> 1;
            let has_type = matches!(category, 1 | 2 | 7);
            let mut ty: u16 = if has_type {
                if b3 & 0x10 != 0 {
                    reader.u16()?
                } else {
                    reader.u8()? as u16
                }
            } else {
                0
            };
            let light = if b3 & 0x20 != 0 {
                reader.u8()?
            } else {
                255
            };
            let run = match (b3 & 0xC0) >> 6 {
                0 => 0usize,
                1 => reader.u8()? as usize,
                _ => reader.u16()? as usize,
            };

            match category {
                0 => {
                    // `3977-3979`: light <= 18 skip-run, left unrevealed.
                    m += run + 1;
                    continue;
                }
                1 => ty += num14,
                2 => ty += num17,
                3 | 4 | 5 => {
                    let mut liquid = (category - 3) as u16;
                    if b4 & 0x40 == 0x40 {
                        liquid = 3;
                    }
                    ty = num20 + liquid;
                }
                6 => {
                    if (l as f64) < world_surface {
                        // `CalcSkyGradient` (`2078-2081`), with the file's counts.
                        let gradient = ((n7.saturating_sub(1)) as f64
                            * (l as f64 / world_surface))
                            as u8 as u16;
                        ty = num21 + gradient;
                    } else {
                        ty = num24;
                    }
                }
                7 => {
                    ty += if (l as f64) < rock_layer { num22 } else { num23 };
                }
                _ => {}
            }

            let tile = MapTile::create(
                *array5.get(ty as usize).ok_or_else(eof)?,
                light,
                (b4 >> 1) & 0x1F,
            );
            *map.get_mut(l * width + m).ok_or_else(eof)? = tile;
            if light == 255 {
                // `4007-4016`: a 255-light run repeats the same tile.
                let mut remaining = run;
                while remaining > 0 {
                    m += 1;
                    *map.get_mut(l * width + m).ok_or_else(eof)? = tile;
                    remaining -= 1;
                }
            } else {
                // `4017-4023`: an `EqualsWithoutLight` run reads each light.
                let mut remaining = run;
                while remaining > 0 {
                    m += 1;
                    let run_tile = tile.with_light(reader.u8()?);
                    *map.get_mut(l * width + m).ok_or_else(eof)? = run_tile;
                    remaining -= 1;
                }
            }
            m += 1; // the `for` increment
        }
    }
    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Task 4: `.map` save/load ----

    fn map_test_world() -> World {
        let mut w = World::new("Terrustaria".into(), 40, 40);
        w.seed = 0x1234_5678_9abc_def0;
        w.world_surface = 10.0;
        w.rock_layer = 18.0;
        w
    }

    fn map_test_header(world: &World) -> MapHeader {
        MapHeader {
            version: MAP_FORMAT_VERSION,
            seed: world.seed as i32,
            height: world.height,
            width: world.width,
            tile_count: TILE_COUNT as i16,
            wall_count: WALL_COUNT as i16,
            liquid_count: 4,
            sky_count: 256,
            dirt_count: 256,
            rock_count: 256,
        }
    }

    /// The per-entry option counts a file we wrote carries (`array3`/`array4`).
    fn map_test_options() -> (Vec<u8>, Vec<u8>) {
        let t = color_table();
        (t.tile_option_counts.clone(), t.wall_option_counts.clone())
    }

    fn put(map: &mut [MapTile], width: usize, x: usize, y: usize, tile: MapTile) {
        map[y * width + x] = tile;
    }

    /// A 40x40 map exercising every RLE branch: a `Light <= 18` skip-run (default
    /// tiles), a `Light == 255` run, an equal type+colour run with differing light,
    /// a type offset `> 255`, a painted tile, and the wall/liquid regions.
    fn map_test_tiles(width: usize, height: usize) -> Vec<MapTile> {
        let t = color_table();
        let mut map = vec![MapTile::default(); width * height];
        // x0..x3 stay `default` (light 0 <= 18): one skip-run.
        for x in 4..=6 {
            put(&mut map, width, x, 0, MapTile::create(t.tile_position + 2, 255, 0));
        }
        for (x, light) in [(7usize, 30u8), (8, 60), (9, 90)] {
            put(&mut map, width, x, 0, MapTile::create(t.tile_position + 3, light, 0));
        }
        put(&mut map, width, 10, 0, MapTile::create(t.tile_position + 4, 100, 7));
        put(&mut map, width, 11, 0, MapTile::create(t.tile_position + 300, 255, 0));
        put(&mut map, width, 12, 0, MapTile::create(t.wall_position, 200, 0));
        put(&mut map, width, 13, 0, MapTile::create(t.liquid_position, 200, 0));
        put(&mut map, width, 14, 0, MapTile::create(t.liquid_position + 3, 200, 0));
        map
    }

    #[test]
    fn body_round_trips() {
        let world = map_test_world();
        let (w, h) = (40usize, 40usize);
        let original = map_test_tiles(w, h);
        let header = map_test_header(&world);
        let (tile_options, wall_options) = map_test_options();
        let body = encode_tiles(&original, w, h);
        let decoded = decode_tiles(
            &body,
            &header,
            &tile_options,
            &wall_options,
            w,
            h,
            world.world_surface,
            world.rock_layer,
        )
        .expect("body must decode");
        assert_eq!(decoded.len(), original.len());
        for (i, (a, b)) in original.iter().zip(decoded.iter()).enumerate() {
            assert_eq!(a.ty, b.ty, "ty at {i}");
            assert_eq!(a.light, b.light, "light at {i}");
            assert_eq!(a.color(), b.color(), "color at {i}");
            assert_eq!(a.is_changed(), b.is_changed(), "changed at {i}");
        }
        // Light <= 18 tiles are never saved: the skip-run decodes to `default`.
        assert_eq!(decoded[0], MapTile::default());
        assert_eq!(decoded[2], MapTile::default());
    }

    #[test]
    fn variable_light_run_flush_to_row_end_round_trips() {
        let world = map_test_world();
        let (w, h) = (6usize, 1usize);
        let t = color_table();
        // A width-6 row that is one type+colour with distinct lights in 19..=254,
        // so the `EqualsWithoutLight` run ends exactly on the last column (the
        // row-end case where the decompiled `num6` stays 0).
        let mut map = vec![MapTile::default(); w * h];
        for (x, light) in [(0usize, 20u8), (1, 30), (2, 40), (3, 50), (4, 60), (5, 70)] {
            put(&mut map, w, x, 0, MapTile::create(t.tile_position + 5, light, 0));
        }
        let header = map_test_header(&world);
        let (tile_options, wall_options) = map_test_options();
        let body = encode_tiles(&map, w, h);
        let decoded = decode_tiles(
            &body,
            &header,
            &tile_options,
            &wall_options,
            w,
            h,
            world.world_surface,
            world.rock_layer,
        )
        .expect("a row-end variable-light run must round-trip");
        assert_eq!(decoded, map);
    }

    #[test]
    fn background_categories_round_trip() {
        let world = map_test_world(); // world_surface 10, rock_layer 18
        let (w, h) = (40usize, 40usize);
        let t = color_table();
        let mut map = vec![MapTile::default(); w * h];
        let sky = t.sky_position + (255.0 * (2.0 / world.world_surface)) as u8 as u16;
        put(&mut map, w, 5, 2, MapTile::create(sky, 255, 0));
        put(&mut map, w, 5, 12, MapTile::create(t.dirt_position + 40, 255, 0));
        put(&mut map, w, 5, 25, MapTile::create(t.rock_position + 40, 255, 0));
        put(&mut map, w, 5, 30, MapTile::create(t.hell_position, 255, 0));

        let header = map_test_header(&world);
        let (tile_options, wall_options) = map_test_options();
        let body = encode_tiles(&map, w, h);
        let decoded = decode_tiles(
            &body,
            &header,
            &tile_options,
            &wall_options,
            w,
            h,
            world.world_surface,
            world.rock_layer,
        )
        .expect("body must decode");
        assert_eq!(decoded[2 * w + 5].ty, sky, "sky row");
        assert_eq!(decoded[12 * w + 5].ty, t.dirt_position + 40, "dirt row");
        assert_eq!(decoded[25 * w + 5].ty, t.rock_position + 40, "rock row");
        assert_eq!(decoded[30 * w + 5].ty, t.hell_position, "hell row");
    }

    #[test]
    fn header_is_version_326_and_world_size() {
        let world = map_test_world();
        let mut bytes = write_map_header(&world);
        bytes.extend_from_slice(&write_option_tables());
        let prefix = read_map_prefix(&bytes)
            .expect("read must not error")
            .expect("prefix must parse");
        assert_eq!(prefix.header.version, 326);
        assert_eq!(prefix.header.seed, world.seed as i32);
        assert_eq!(prefix.header.width, world.width);
        assert_eq!(prefix.header.height, world.height);
        assert_eq!(prefix.world_name, "Terrustaria");
        // The tables are the whole prefix, so the body starts right after them.
        assert_eq!(prefix.body_offset, bytes.len());
        assert_eq!(prefix.tile_options, color_table().tile_option_counts);
        // A fresh metadata revision is always 1, so the header is deterministic.
        assert_eq!(write_map_header(&world), write_map_header(&world));
    }

    #[test]
    fn foreign_version_is_rejected() {
        let world = map_test_world();
        let mut bytes = write_map_header(&world);
        bytes.extend_from_slice(&write_option_tables());
        bytes[0..4].copy_from_slice(&999u32.to_le_bytes());
        assert!(read_map_prefix(&bytes).expect("no error").is_none());
    }

    #[test]
    fn negative_table_count_is_rejected() {
        let world = map_test_world();
        let mut bytes = write_map_header(&world);
        bytes.extend_from_slice(&write_option_tables());
        // `sky_count` is the 4th of the six i16 counts. Its offset is the fixed
        // header layout: version(4) + FileMetadata(8) + revision(4) + favourite(8)
        // + name (7-bit length byte + ASCII) + seed(4) + height(4) + width(4), then
        // the counts. Patch it to -1; a negative i16 cast to `usize` would size
        // `decode_tiles`' `array5` from a huge value and panic on capacity overflow.
        let name_field = 1 + world.name.len();
        let sky_off = 4 + 8 + 4 + 8 + name_field + 4 + 4 + 4 + 3 * 2;
        bytes[sky_off..sky_off + 2].copy_from_slice(&(-1i16).to_le_bytes());

        let parsed = read_map_prefix(&bytes).expect("read must not error");
        assert!(parsed.is_none(), "a negative table count must be rejected");

        // The unpatched prefix still parses, so the test targets the count only.
        let mut ok = write_map_header(&world);
        ok.extend_from_slice(&write_option_tables());
        assert!(read_map_prefix(&ok).expect("no error").is_some());
    }

    #[test]
    fn truncated_prefix_is_rejected() {
        let short = vec![0u8; 39];
        assert!(read_map_prefix(&short).expect("no error").is_none());
    }

    #[test]
    fn truncated_body_is_rejected() {
        let world = map_test_world();
        let (w, h) = (40usize, 40usize);
        let original = map_test_tiles(w, h);
        let header = map_test_header(&world);
        let (tile_options, wall_options) = map_test_options();
        let mut body = encode_tiles(&original, w, h);
        body.truncate(body.len() / 2);
        let result = decode_tiles(
            &body,
            &header,
            &tile_options,
            &wall_options,
            w,
            h,
            world.world_surface,
            world.rock_layer,
        );
        assert!(result.is_err(), "a cut body must be rejected");
    }

    #[test]
    #[ignore]
    fn loads_a_real_vanilla_map() {
        // Manual verification: set `TR_VANILLA_MAP` to a vanilla `.map`. `tr-core`
        // can only check the uncompressed prefix (the body is deflate-wrapped and
        // `flate2` is deliberately not a `tr-core` dependency); the full decode is
        // the `#[ignore] loads_a_real_vanilla_map` test in `tr-app`, which counts
        // the revealed tiles.
        let Ok(path) = std::env::var("TR_VANILLA_MAP") else {
            return;
        };
        let bytes = std::fs::read(path).expect("read map");
        let prefix = read_map_prefix(&bytes)
            .expect("no error")
            .expect("vanilla prefix must parse");
        assert_eq!(prefix.header.version, MAP_FORMAT_VERSION);
        assert!(
            prefix.body_offset < bytes.len(),
            "vanilla map has an empty body"
        );
    }
}
