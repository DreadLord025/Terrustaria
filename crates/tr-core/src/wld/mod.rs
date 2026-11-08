//! Parser + writer for Terraria `.wld` files (format version 87+, sections-based
//! v2).
//!
//! Mirrors `Terraria.IO.WorldFile.LoadWorld_Version2` / `LoadWorldTiles` from the
//! original sources. The header+flags and tile sections are decoded in full;
//! the eight remaining sections live in [`sections`], whose registry drives both
//! the pointer verification on load and the prologue/sections/footer assembly
//! on save (`SaveWorld_Version2:1150`).

use std::path::Path;

use anyhow::{anyhow, bail, Context, Result};

use crate::binio::{Reader as BinReader, Writer};
use crate::tile::*;
use crate::world::World;

use sections::SectionData;

pub mod sections;

const FILE_METADATA_SIZE: usize = 20;
/// `SaveFileFormatHeader:1174`: the version the current C# writes.
const FORMAT_VERSION: i32 = 326;
/// `SaveWorld_Version2:1152`: eleven section end-offsets.
const SECTION_COUNT: usize = 11;
/// Offset of the pointer table inside the prologue: 4 (version) + 20
/// (metadata) + 2 (count) (`SaveHeaderPointers:1208-1217`).
const POINTER_TABLE_OFFSET: usize = 26;
/// `FileType.World` (`IO/FileType.cs`): the value stored in the metadata head.
const FILE_TYPE_WORLD: u8 = 2;
/// `FileMetadata.MAGIC_NUMBER` (`IO/FileMetadata.cs:8`).
const FILE_METADATA_MAGIC: u64 = 27981915666277746;

pub fn load(path: &Path) -> Result<World> {
    let data = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    parse(&data).with_context(|| format!("parsing {}", path.display()))
}

/// The 20-byte `Re-Logic` file header (`IO/FileMetadata.cs:22-27`): magic +
/// file type, revision counter, flags (bit 0 = favorite).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FileMetadata {
    pub ty: u8,
    pub revision: u32,
    pub favorite: bool,
}

impl FileMetadata {
    fn world(revision: u32) -> Self {
        Self {
            ty: FILE_TYPE_WORLD,
            revision,
            favorite: false,
        }
    }

    /// `FileMetadata.Write` (`IO/FileMetadata.cs:22-27`).
    fn to_bytes(self) -> [u8; FILE_METADATA_SIZE] {
        let mut b = [0u8; FILE_METADATA_SIZE];
        b[0..8].copy_from_slice(&(FILE_METADATA_MAGIC | (u64::from(self.ty) << 56)).to_le_bytes());
        b[8..12].copy_from_slice(&self.revision.to_le_bytes());
        b[12..20].copy_from_slice(&u64::from(self.favorite).to_le_bytes());
        b
    }
}

/// The `.wld` prologue: version, metadata, the section table (end offsets) and
/// the importance bitmap — everything a reader needs before the sections
/// (`LoadFileFormatHeader:1902`).
pub struct SectionTable<'a> {
    data: &'a [u8],
    version: i32,
    /// Section end offsets (`LoadFileFormatHeader:1925-1928`): `pointers[i]` is
    /// where section i ends (= where section i+1 starts).
    pointers: Vec<usize>,
    /// `Main.tileFrameImportant` (`LoadFileFormatHeader:1929-1942`).
    importance: Vec<bool>,
    metadata: FileMetadata,
}

impl<'a> SectionTable<'a> {
    /// Reads the prologue. Behavior-identical to the inline header parsing this
    /// replaced: same reads, same bails.
    pub fn open(data: &'a [u8]) -> Result<Self> {
        let mut r = Reader::new(data);

        // --- file format header ---------------------------------------------
        let version = r.i32()?;
        if version < 87 {
            bail!("unsupported world format {version} (expected >= 87)");
        }
        let mut metadata = FileMetadata::default();
        if version >= 135 {
            let at = r.0.pos();
            r.skip(FILE_METADATA_SIZE)?;
            // Recorded, not validated: the metadata check belongs to section
            // 0's reader (the file-format-header phase) — the split itself must
            // not change which files load.
            let raw = &data[at..at + FILE_METADATA_SIZE];
            metadata = FileMetadata {
                ty: (u64::from_le_bytes(raw[0..8].try_into().unwrap()) >> 56) as u8,
                revision: u32::from_le_bytes(raw[8..12].try_into().unwrap()),
                favorite: u64::from_le_bytes(raw[12..20].try_into().unwrap()) & 1 == 1,
            };
        }

        let section_count = r.i16()? as usize;
        let mut pointers = Vec::with_capacity(section_count);
        for _ in 0..section_count {
            pointers.push(r.i32()? as usize);
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

        // `LoadWorld_Version2:1767-1770`: the file-format header must end
        // exactly at pointer 0 (an empty table still bails in `parse`).
        if let Some(want) = pointers.first() {
            if *want != r.0.pos() {
                bail!(
                    "bad section pointer 0: header ends at {}, table says {want}",
                    r.0.pos()
                );
            }
        }

        Ok(Self {
            data,
            version,
            pointers,
            importance,
            metadata,
        })
    }

    /// The format version the file was written with (`_versionNumber`).
    pub fn version(&self) -> i32 {
        self.version
    }

    /// The file's `FileMetadata` (`Main.WorldFileMetadata`,
    /// `WorldFile.cs:1910`).
    pub fn metadata(&self) -> FileMetadata {
        self.metadata
    }
}

pub fn parse(data: &[u8]) -> Result<World> {
    let mut table = SectionTable::open(data)?;
    let version = table.version;

    // --- header section ----------------------------------------------------
    if table.pointers.is_empty() {
        bail!("world has no section table");
    }
    let mut r = Reader::new(data);
    r.seek(table.pointers[0])?;
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
    // `LoadWorld_Version2:1772-1775`: the header must end exactly at pointer 1.
    expect_section_end(&r, &table.pointers, 1)?;

    // --- tile section ------------------------------------------------------
    if table.pointers.len() < 2 {
        bail!("world has no tile section");
    }
    r.seek(table.pointers[1])?;
    load_tiles(&mut r, &mut world, &table.importance, version)?;
    expect_section_end(&r, &table.pointers, 2)?;
    // The sections past the tiles: pointer sanity (a pointer past EOF or going
    // backwards is a hard reject); the codecs land phase by phase.
    skip_sections(&r, &table)?;

    world.frame_important = std::mem::take(&mut table.importance);
    world.compute_spawn_y();
    Ok(world)
}

/// C#'s `Position != positions[i]` check (`LoadWorld_Version2:1767-1780`):
/// the section just read must end exactly at its recorded end offset.
fn expect_section_end(r: &Reader, pointers: &[usize], idx: usize) -> Result<()> {
    let want = pointers.get(idx).copied().unwrap_or(usize::MAX);
    let got = r.0.pos();
    if got != want {
        bail!("bad section pointer {idx}: section ends at {got}, table says {want}");
    }
    Ok(())
}

/// Walks the present pointers 3..min(count, 11) in order, rejecting one that
/// goes backwards or past EOF. Unimplemented sections are skip-read — the
/// pointer table is their length — which is what the port has always done with
/// them; a section phase turns its entry into a real read + end-offset check.
fn skip_sections(r: &Reader, table: &SectionTable<'_>) -> Result<()> {
    let mut cursor = r.0.pos();
    for idx in 3..table.pointers.len().min(SECTION_COUNT) {
        let end = table.pointers[idx];
        if end < cursor {
            bail!("bad section pointer {idx}: {end} is before the cursor {cursor}");
        }
        if end > table.data.len() {
            bail!("bad section pointer {idx}: {end} beyond EOF");
        }
        cursor = end;
    }
    Ok(())
}

/// Writes a `.wld` (`SaveWorld_Version2:1150-1168`): the 70-byte prologue, the
/// eleven sections in C# order, the footer, then the pointer backfill inside
/// the assembled buffer (`SaveHeaderPointers:1208` — no stream seeking).
/// Sections whose codec has not landed yet write their empty encoding; a
/// section that carries data without a codec is refused rather than written
/// corrupt.
pub fn save(path: &Path, world: &World, data: &SectionData) -> Result<()> {
    // `FileMetadata.IncrementAndWrite` (`IO/FileMetadata.cs:29-33`): the
    // previous file's revision + 1; a first save starts at 1.
    let revision = std::fs::read(path)
        .ok()
        .and_then(|d| SectionTable::open(&d).ok().map(|t| t.metadata.revision))
        .map_or(1, |r| r + 1);
    let buf = to_bytes(world, data, revision)?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
    }
    let tmp = path.with_extension("wld.tmp");
    std::fs::write(&tmp, &buf).with_context(|| format!("writing {}", tmp.display()))?;
    std::fs::rename(&tmp, path).with_context(|| format!("replacing {}", path.display()))?;
    Ok(())
}

/// The in-memory half of [`save`]: assemble the prologue, walk the writer
/// table, append the footer, backfill the pointer table.
pub fn to_bytes(world: &World, data: &SectionData, revision: u32) -> Result<Vec<u8>> {
    let version = FORMAT_VERSION;

    // --- prologue (`SaveFileFormatHeader:1170-1180`) ------------------------
    let mut prologue = Vec::with_capacity(70);
    prologue.extend_from_slice(&version.to_le_bytes());
    prologue.extend_from_slice(&FileMetadata::world(revision).to_bytes());
    prologue.extend_from_slice(&(SECTION_COUNT as i16).to_le_bytes());
    prologue.extend_from_slice(&[0u8; 4 * SECTION_COUNT]); // backfilled below

    // --- sections (`SaveWorld_Version2:1152-1165`) --------------------------
    let mut bodies: Vec<Vec<u8>> = Vec::with_capacity(SECTION_COUNT);
    for def in sections::SAVE_DEFS {
        if !(def.implemented)() && !(def.empty)(data) {
            bail!(
                "section {} (pointer {}) has data but no writer yet",
                def.name,
                def.pointer
            );
        }
        let mut w = Writer::new();
        (def.write)(&mut w, world, data, version);
        bodies.push(w.into_inner());
    }

    // --- footer (`SaveFooter:1752`) -----------------------------------------
    let mut footer = Writer::new();
    footer.boolean(true);
    footer.cs_string(&world.name);
    footer.i32(0); // world id: the port keeps none (see `parse`'s `_world_id`)

    // --- assemble + backfill (`SaveHeaderPointers:1208-1218`) ---------------
    let mut pointers = [0usize; SECTION_COUNT];
    let mut offset = prologue.len();
    for (i, body) in bodies.iter().enumerate() {
        offset += body.len();
        pointers[i] = offset;
    }
    let mut file = prologue;
    for body in bodies {
        file.extend_from_slice(&body);
    }
    file.extend_from_slice(&footer.into_inner());
    for (i, p) in pointers.iter().enumerate() {
        let at = POINTER_TABLE_OFFSET + 4 * i;
        file[at..at + 4].copy_from_slice(&(*p as i32).to_le_bytes());
    }
    Ok(file)
}

/// Runs the load side of the section table: each section whose codec has landed
/// is read and must end exactly at its recorded offset
/// (`LoadWorld_Version2:1781-1850`); the rest are skip-read. Ends with the
/// footer triple (`LoadFooter:2990`).
pub fn load_sections(table: &SectionTable<'_>) -> Result<SectionData> {
    let mut data = SectionData::default();
    let mut r = BinReader::new(table.data);
    let mut cursor = table.pointers.get(2).copied().unwrap_or(0);
    for def in sections::DEFS {
        if def.pointer >= table.pointers.len() || !(def.gate)(table.version) {
            continue;
        }
        let end = table.pointers[def.pointer];
        if end < cursor || end > table.data.len() {
            bail!(
                "bad section pointer {} ({}): {} is out of order or beyond EOF",
                def.pointer,
                def.name,
                end
            );
        }
        if (def.implemented)() {
            r.seek(cursor).ok_or_else(|| eof(cursor))?;
            let payload = (def.read)(&mut r, table.version)
                .ok_or_else(|| anyhow!("section {}: unexpected end of file", def.name))?;
            if r.pos() != end {
                bail!(
                    "bad section pointer {} ({}): section ends at {}, table says {end}",
                    def.pointer,
                    def.name,
                    r.pos()
                );
            }
            data.apply(payload);
        }
        cursor = end;
    }

    // --- footer (`LoadFooter:2990-3004`) ------------------------------------
    r.seek(cursor).ok_or_else(|| eof(cursor))?;
    if !r.boolean().ok_or_else(|| eof(r.pos()))? {
        bail!("bad footer");
    }
    // The name and id are C#'s self-description check; the port keeps no world
    // id to compare against, so the triple is parsed for shape only.
    let _name = r.cs_string_lossy().ok_or_else(|| eof(r.pos()))?;
    let _id = r.i32().ok_or_else(|| eof(r.pos()))?;
    Ok(data)
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Format version the current C# writes (`SaveFileFormatHeader:1174`).
    const VER: i32 = 326;

    /// A `.NET`-style string: 7-bit length prefix + UTF-8 payload.
    fn cs_string(s: &str) -> Vec<u8> {
        let mut v = vec![s.len() as u8];
        v.extend_from_slice(s.as_bytes());
        v
    }

    /// The importance bitmap in C# bit order (`SaveFileFormatHeader:1182-1204`):
    /// LSB-first, `ceil(count / 8)` bytes.
    fn importance_bitmap(types: &[usize], count: usize) -> Vec<u8> {
        let mut v = vec![0u8; count.div_ceil(8)];
        for &t in types {
            v[t / 8] |= 1 << (t % 8);
        }
        v
    }

    /// The world-header + flags block, mirroring `SaveWorldHeader:1221` /
    /// `SaveWorldFlags:1238` field-by-field with the load gates from
    /// `load_header_flags`.
    fn header_section(version: i32) -> Vec<u8> {
        let mut b = Vec::new();
        b.extend(cs_string("TileFixture"));
        b.extend(cs_string("12345")); // seed text
        b.extend_from_slice(&7u64.to_le_bytes()); // generator version
        b.extend_from_slice(&[0x22; 16]); // GUID
        b.extend_from_slice(&3i32.to_le_bytes()); // world id
        b.extend_from_slice(&(-10i32).to_le_bytes()); // left
        b.extend_from_slice(&1000i32.to_le_bytes()); // right
        b.extend_from_slice(&(-20i32).to_le_bytes()); // top
        b.extend_from_slice(&900i32.to_le_bytes()); // bottom
        b.extend_from_slice(&6i32.to_le_bytes()); // maxTilesY
        b.extend_from_slice(&8i32.to_le_bytes()); // maxTilesX
                                                  // flags: GameMode (+9 bools when >= 209), creation time, last played.
        if version >= 209 {
            b.extend_from_slice(&[0u8; 13]);
        } else {
            b.push(0); // GameMode as bool
            if version == 208 {
                b.push(0);
            }
        }
        if version >= 141 {
            b.extend_from_slice(&[0u8; 8]);
        }
        if version >= 284 {
            b.extend_from_slice(&[0u8; 8]);
        }
        b.push(3); // moon type
        b.extend_from_slice(&[0u8; 4 * 17]); // treeX/treeStyle/caveBackX/caveBackStyle/back styles
        b.extend_from_slice(&4i32.to_le_bytes()); // spawn x
        b.extend_from_slice(&5i32.to_le_bytes()); // spawn y
        b.extend_from_slice(&4.0f64.to_le_bytes()); // world surface
        b.extend_from_slice(&100.0f64.to_le_bytes()); // rock layer
        b.extend_from_slice(&1234.0f64.to_le_bytes()); // time
        b.push(0); // day time
        b.extend_from_slice(&5i32.to_le_bytes()); // moon phase
        b
    }

    /// The tile section for the 8x6 fixture world, hand-encoded as the inverse
    /// of `load_tiles` (column-major, chained b1/b2/b3 headers, RLE tail).
    /// Frame bytes follow an active tile only for important kinds.
    fn tile_section(importance: &[usize]) -> Vec<u8> {
        let mut b = Vec::new();
        let mut active = |b: &mut Vec<u8>, kind: i32, fx: i16, fy: i16| {
            b.push(0x02);
            b.push(kind as u8);
            if importance.contains(&(kind as usize)) {
                b.extend_from_slice(&fx.to_le_bytes());
                b.extend_from_slice(&fy.to_le_bytes());
            }
        };
        // Column 0: RLE 1-byte run, active tiles 2 + 144 (frame fix-up), a
        // painted wall, shimmered water.
        b.extend_from_slice(&[0x40, 0x01]); // air x2 (y 0..1)
        active(&mut b, 2, 90, 18);
        active(&mut b, 144, 10, 200);
        b.extend_from_slice(&[0x05, 0x01, 0x10, 5, 9]); // wall 5 painted 9
        b.extend_from_slice(&[0x09, 0x01, 0x80, 128]); // water 128, shimmer
                                                       // Column 1: painted tile, RLE 16-bit run, wall > 255 with coat, honey.
        b.extend_from_slice(&[0x03, 0x01, 0x08]); // active header with paint
        b.push(2);
        if importance.contains(&2) {
            b.extend_from_slice(&90i16.to_le_bytes());
            b.extend_from_slice(&18i16.to_le_bytes());
        }
        b.push(7); // tile paint
        b.extend_from_slice(&[0x80, 2, 0]); // air x2 (16-bit run, y 1..3)
        b.extend_from_slice(&[0x05, 0x01, 0x41, 0x02, 0x34, 0x01]); // wall 308, invisible block
        b.extend_from_slice(&[0x19, 0x01, 0x80, 200]); // honey 200, shimmer
                                                       // Columns 2..7: one 16-bit RLE run covering the whole column.
        for _ in 2..8 {
            b.extend_from_slice(&[0x80, 5, 0]);
        }
        b
    }

    /// The empty encodings of the sections a file carries (C# `Save*` of an
    /// empty store), one chunk per section so the fixture records each pointer.
    /// Version-gated exactly as the load side reads them.
    fn empty_section_chunks(version: i32, present: usize) -> Vec<Vec<u8>> {
        let mut v: Vec<Vec<u8>> = vec![
            vec![0, 0], // 3 chests: i16 count
            vec![0, 0], // 4 signs: i16 count
            // 5 npcs: shimmer count (>= 268), town + tower terminators (>= 140)
            [
                if version >= 268 {
                    vec![0; 4]
                } else {
                    Vec::new()
                },
                vec![0],
                if version >= 140 { vec![0] } else { Vec::new() },
            ]
            .concat(),
            0i32.to_le_bytes().to_vec(), // 6 tile entities: i32 count
            0i32.to_le_bytes().to_vec(), // 7 pressure plates: i32 count
            0i32.to_le_bytes().to_vec(), // 8 town rooms: i32 count
            vec![0u8; 12],               // 9 bestiary: 3 x i32 count
            vec![0],                     // 10 creative powers: terminator
        ];
        // Pre-294 files carry the shared slot count; pre-210/220 lack the last
        // two sections entirely — the caller says how many pointers exist.
        if version < 294 && !v.is_empty() {
            v[0].extend_from_slice(&[0, 0]);
        }
        v.truncate(present.saturating_sub(3));
        v
    }

    /// Builds a full synthetic file: prologue + sections 0..`present`-1 +
    /// footer.
    fn synthetic_file(
        version: i32,
        importance_types: &[usize],
        importance_count: usize,
        present: usize,
    ) -> Vec<u8> {
        let mut file = Vec::new();
        file.extend_from_slice(&version.to_le_bytes());
        file.extend_from_slice(&[0u8; 20]); // FileMetadata
        file.extend_from_slice(&(present as i16).to_le_bytes());
        let ptr_at = file.len();
        file.extend_from_slice(&vec![0u8; 4 * present]); // pointer placeholders
        file.extend_from_slice(&(importance_count as u16).to_le_bytes());
        file.extend(importance_bitmap(importance_types, importance_count));

        let mut pointers = [0usize; 11];
        pointers[0] = file.len();
        file.extend(header_section(version));
        pointers[1] = file.len();
        file.extend(tile_section(importance_types));
        pointers[2] = file.len();
        for (i, chunk) in empty_section_chunks(version, present)
            .into_iter()
            .enumerate()
        {
            file.extend(chunk);
            pointers[3 + i] = file.len();
        }
        for (i, p) in pointers.iter().take(present).enumerate() {
            file[ptr_at + 4 * i..ptr_at + 4 * i + 4].copy_from_slice(&(*p as i32).to_le_bytes());
        }
        file.push(1); // footer bool
        file.extend(cs_string("TileFixture"));
        file.extend_from_slice(&3i32.to_le_bytes());
        file
    }

    /// The default fixture: v326, 11 pointers.
    fn fixture() -> Vec<u8> {
        synthetic_file(VER, &[2, 144], 160, 11)
    }

    #[test]
    fn parses_the_synthetic_world_header() {
        let data = fixture();
        let world = parse(&data).expect("synthetic world parses");
        assert_eq!(world.name, "TileFixture");
        assert_eq!((world.width, world.height), (8, 6));
        assert_eq!((world.spawn_x, world.spawn_y), (4, 5));
        assert_eq!(world.world_surface, 4.0);
        assert_eq!(world.rock_layer, 100.0);
        assert_eq!(world.time, 1234.0);
        assert!(!world.day_time);
        assert_eq!(world.moon_type, 3);
        assert_eq!(world.moon_phase, 5);
    }

    /// The tile `load_tiles` builds for an empty cell: frames -1, everything
    /// else zero.
    fn air_tile() -> Tile {
        Tile {
            frame_x: -1,
            frame_y: -1,
            ..Default::default()
        }
    }

    #[test]
    fn parses_the_tile_section_column_major() {
        let data = fixture();
        let world = parse(&data).expect("synthetic world parses");
        let at = |x, y| world.tiles[world.index(x, y)];

        // RLE 1-byte run fills two air tiles.
        assert_eq!(at(0, 0), air_tile());
        assert_eq!(at(0, 1), air_tile());
        // An important active tile carries its frames.
        let t = at(0, 2);
        assert_eq!(t.kind, 2);
        assert_ne!(t.flags & FLAG_ACTIVE, 0);
        assert_eq!((t.frame_x, t.frame_y), (90, 18));
        // kind 144 has its frameY forced to 0 on load.
        let t = at(0, 3);
        assert_eq!(t.kind, 144);
        assert_eq!((t.frame_x, t.frame_y), (10, 0));
        // Wall paint rides the sparse extras table.
        assert_eq!(at(0, 4).wall, 5);
        assert_eq!(world.wall_paint(0, 4), 9);
        // Water + shimmer.
        assert_eq!(at(0, 5).liquid, 128);
        assert_ne!(at(0, 5).flags & FLAG_LIQUID_SHIMMER, 0);
        // Tile paint is packed into meta.
        let t = at(1, 0);
        assert_eq!(t.paint(), 7);
        assert_eq!((t.frame_x, t.frame_y), (90, 18));
        // A 16-bit wall id and the invisible-block coat.
        assert_eq!(at(1, 4).wall, 308);
        assert!(world.invisible_block(1, 4));
        // Honey + shimmer.
        assert_eq!(at(1, 5).liquid, 200);
        assert_ne!(at(1, 5).flags & FLAG_LIQUID_HONEY, 0);
        assert_ne!(at(1, 5).flags & FLAG_LIQUID_SHIMMER, 0);
        // The last RLE tile of a column also fills the run.
        assert_eq!(at(3, 3), air_tile());
    }

    #[test]
    fn the_importance_bitmap_round_trips_lsb_first() {
        let data = fixture();
        let world = parse(&data).expect("synthetic world parses");
        let imp = &world.frame_important;
        assert_eq!(imp.len(), 160);
        assert!(imp[2] && imp[144]);
        assert!(!imp[0] && !imp[159]);
    }

    /// The `% 8 == 0` corner of the bitpack: 754 tile ids mean a 95-byte bitmap
    /// whose last byte is full, and no partial trailing byte exists
    /// (`SaveFileFormatHeader:1201-1204`).
    #[test]
    fn the_full_byte_bitmap_corner_parses() {
        let data = synthetic_file(VER, &[0, 7, 753], 754, 11);
        let world = parse(&data).expect("synthetic world parses");
        assert_eq!(world.frame_important.len(), 754);
        assert!(world.frame_important[0]);
        assert!(world.frame_important[7]);
        assert!(world.frame_important[753]);
        assert!(!world.frame_important[8]);
        // 754 ids tile the bitmap exactly: 70-byte prologue + count + 95 bytes.
        let header_at = 70 + 2 + 95;
        assert_eq!(data[header_at], 11); // first byte of "TileFixture"
    }

    #[test]
    fn rejects_old_versions_and_broken_reads() {
        // Version below 87 is still a hard bail.
        let mut data = fixture();
        data[0..4].copy_from_slice(&86i32.to_le_bytes());
        assert!(parse(&data).is_err());
        // A truncated file never panics: it bails on the short read.
        let data = fixture();
        assert!(parse(&data[..80]).is_err());
        // A negative RLE run is rejected (16-bit run of -1).
        let mut data = fixture();
        let tiles_at = 70 + 2 + 20;
        data[tiles_at..tiles_at + 3].copy_from_slice(&[0x80, 0xFF, 0xFF]);
        assert!(parse(&data).is_err());
    }

    // --- DW-1.1: the split changed no behavior ------------------------------

    /// A per-test scratch dir under the system temp dir (tr-core has no
    /// tempfile dev-dependency).
    fn scratch(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("tr-wld-p1-{}-{name}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("scratch dir");
        dir
    }

    fn at(world: &World, x: i32, y: i32) -> Tile {
        world.tiles[world.index(x, y)]
    }

    #[test]
    fn test_DW_1_1_load_keeps_its_signature_and_reads_a_file() {
        let path = scratch("load-fixture").join("fixture.wld");
        std::fs::write(&path, fixture()).expect("write fixture");
        let world = load(&path).expect("load reads the synthetic file");
        assert_eq!(world.name, "TileFixture");
        assert_eq!((world.width, world.height), (8, 6));
        assert_eq!((world.spawn_x, world.spawn_y), (4, 5));
        assert_eq!(at(&world, 0, 2).kind, 2);
        let _ = std::fs::remove_dir_all(scratch("load-fixture"));
    }

    // --- DW-1.3: pointer verification ---------------------------------------

    #[test]
    fn test_DW_1_3_rejects_a_misaligned_header_pointer() {
        // pointers[1] shifted 3 bytes later: the header does not end there.
        let mut data = fixture();
        let at = 26 + 4;
        let bad = i32::from_le_bytes(data[at..at + 4].try_into().unwrap()) + 3;
        data[at..at + 4].copy_from_slice(&bad.to_le_bytes());
        assert!(parse(&data).is_err());
    }

    #[test]
    fn test_DW_1_3_rejects_a_misaligned_file_format_header() {
        // pointers[0] shifted: the file-format header ends mid-bitmap.
        let mut data = fixture();
        let at = 26;
        let bad = i32::from_le_bytes(data[at..at + 4].try_into().unwrap()) + 1;
        data[at..at + 4].copy_from_slice(&bad.to_le_bytes());
        assert!(parse(&data).is_err());
    }

    #[test]
    fn test_DW_1_3_rejects_a_tile_pointer_past_eof() {
        let mut data = fixture();
        let at = 26 + 8; // pointers[2]
        let past = data.len() as i32 + 10;
        data[at..at + 4].copy_from_slice(&past.to_le_bytes());
        assert!(parse(&data).is_err());
    }

    #[test]
    fn test_DW_1_3_rejects_a_backwards_section_pointer() {
        let mut data = fixture();
        let at = 26 + 12; // pointers[3], before the tile section's end
        data[at..at + 4].copy_from_slice(&0i32.to_le_bytes());
        assert!(parse(&data).is_err());
    }

    #[test]
    fn test_DW_1_3_rejects_a_section_pointer_past_eof() {
        let mut data = fixture();
        let at = 26 + 20; // pointers[5]
        data[at..at + 4].copy_from_slice(&i32::MAX.to_le_bytes());
        assert!(parse(&data).is_err());
    }

    #[test]
    fn test_DW_1_3_walks_a_legacy_six_pointer_table() {
        // A v200 file with six pointers: sections 0..5 exist, pointers 6..10
        // are absent (and the ≥268 shimmer block is not written) — the walk
        // takes only the pointers the table has.
        let data = synthetic_file(200, &[2, 144], 160, 6);
        let world = parse(&data).expect("legacy table loads");
        assert_eq!(world.name, "TileFixture");
        assert_eq!((world.width, world.height), (8, 6));
        assert_eq!(at(&world, 0, 2).kind, 2);
    }

    // --- DW-1.4: the save framing -------------------------------------------

    #[test]
    fn test_DW_1_4_the_prologue_and_pointers_are_hand_computed() {
        let world = World::new("SaveFixture".into(), 8, 6);
        let buf = to_bytes(&world, &SectionData::default(), 1).expect("framing writes");

        // Hand-computed: 70-byte prologue + 37 bytes of empty sections (2 tile
        // types word + 2+2+6+4+4+4+12+1) + footer (1 + 12 + 4).
        assert_eq!(buf.len(), 124);
        // Prologue: i32 326, 20-byte metadata, i16 11, 11 x i32.
        assert_eq!(&buf[0..4], &326i32.to_le_bytes());
        assert_eq!(
            &buf[4..12],
            &(27981915666277746u64 | (2u64 << 56)).to_le_bytes(),
            "magic | FileType.World"
        );
        assert_eq!(&buf[12..16], &1u32.to_le_bytes(), "revision");
        assert_eq!(&buf[16..24], &[0u8; 8], "favorite bit clear");
        assert_eq!(&buf[24..26], &11i16.to_le_bytes(), "section count");
        // Pointers are the section END offsets: the empty writers emit 2, 0,
        // 0, then 2, 2, 6, 4, 4, 4, 12, 1 bytes.
        let want: [i32; 11] = [72, 72, 72, 74, 76, 82, 86, 90, 94, 106, 107];
        for (i, p) in want.iter().enumerate() {
            let at = 26 + 4 * i;
            assert_eq!(&buf[at..at + 4], &p.to_le_bytes(), "pointer {i}");
        }
        // The empty section encodings themselves.
        assert_eq!(&buf[70..72], &[0, 0], "empty tile-type count");
        assert_eq!(&buf[72..74], &[0, 0], "chests count");
        assert_eq!(&buf[74..76], &[0, 0], "signs count");
        assert_eq!(&buf[76..80], &0i32.to_le_bytes(), "shimmered count");
        assert_eq!(&buf[80..81], &[0], "town terminator");
        assert_eq!(&buf[81..82], &[0], "tower terminator");
        assert_eq!(&buf[82..94], &[0u8; 12], "te + plates + rooms counts");
        assert_eq!(&buf[94..106], &[0u8; 12], "bestiary 3 counts");
        assert_eq!(&buf[106..107], &[0], "creative terminator");
        // The footer triple `(true, worldName, worldId)`.
        assert_eq!(&buf[107..108], &[1]);
        assert_eq!(buf[108], 11, "name length prefix");
        assert_eq!(&buf[109..120], b"SaveFixture");
        assert_eq!(&buf[120..124], &0i32.to_le_bytes());
    }

    #[test]
    fn test_DW_1_4_the_revision_reads_the_previous_file() {
        let dir = scratch("revision");
        let path = dir.join("rev.wld");
        let world = World::new("R".into(), 8, 6);
        save(&path, &world, &SectionData::default()).expect("first save");
        let saved = std::fs::read(&path).unwrap();
        let table = SectionTable::open(&saved).unwrap();
        assert_eq!(
            table.metadata().revision,
            1,
            "FromCurrentSettings(0) + IncrementAndWrite"
        );
        assert_eq!(table.metadata().ty, 2);
        assert!(!table.metadata().favorite);
        save(&path, &world, &SectionData::default()).expect("second save");
        let saved = std::fs::read(&path).unwrap();
        let table = SectionTable::open(&saved).unwrap();
        assert_eq!(table.metadata().revision, 2, "increments per save");
        let _ = std::fs::remove_dir_all(&dir);
    }

    // --- DW-1.5: the stub contract ------------------------------------------

    #[test]
    fn test_DW_1_5_every_section_stub_round_trips_its_empty_encoding() {
        let world = World::new("t".into(), 8, 6);
        let data = SectionData::default();
        for save_def in sections::SAVE_DEFS.iter().filter(|d| d.pointer >= 3) {
            let load_def = sections::DEFS
                .iter()
                .find(|d| d.pointer == save_def.pointer)
                .expect("symmetric registry");
            let mut w = Writer::new();
            (save_def.write)(&mut w, &world, &data, VER);
            let bytes = w.into_inner();
            let mut r = BinReader::new(&bytes);
            assert!(
                (load_def.read)(&mut r, VER).is_some(),
                "{} stub reads its empty encoding back",
                save_def.name
            );
            assert_eq!(
                r.pos(),
                bytes.len(),
                "{} stub consumes its bytes",
                save_def.name
            );
        }
    }

    #[test]
    fn the_stub_readers_refuse_non_empty_sections() {
        // A count of 1 is not an empty section: the stub must not silently drop
        // the records it cannot yet decode.
        assert!(sections::chests::read(&mut BinReader::new(&[1, 0]), VER).is_none());
        assert!(sections::bestiary::read(&mut BinReader::new(&2i32.to_le_bytes()), VER).is_none());
        assert!(sections::creative_powers::read(&mut BinReader::new(&[1]), VER).is_none());
    }

    #[test]
    fn load_sections_round_trips_the_framed_file() {
        let world = World::new("RoundTrip".into(), 8, 6);
        let buf = to_bytes(&world, &SectionData::default(), 1).unwrap();
        let table = SectionTable::open(&buf).unwrap();
        let data = load_sections(&table).expect("the framed file walks its sections");
        assert_eq!(data, SectionData::default());
    }

    #[test]
    fn save_refuses_a_section_that_has_data_but_no_writer() {
        let world = World::new("t".into(), 8, 6);
        let mut data = SectionData::default();
        data.chests.push(sections::WldChest::default());
        assert!(to_bytes(&world, &data, 1).is_err());
        data.chests.clear();
        data.bestiary.kills.push(("Slime".into(), 1));
        assert!(to_bytes(&world, &data, 1).is_err());
    }
}
