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

        let section_count = r.i16()?;
        if section_count < 0 {
            // A crafted file can put a negative count here; `as usize` would
            // turn it into a huge capacity and panic.
            bail!("implausible section count {section_count}");
        }
        let mut pointers = Vec::with_capacity(section_count as usize);
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
    let mut seed = 0u64;
    if version >= 179 {
        // `Main.ActiveWorldFileData.SeedText` — the port's seed is the u64 its
        // decimal text parses to (0 when foreign, matching `World::seed`'s
        // ".wld-loaded worlds" note); `header::write_world_head` emits the same
        // text form, so the seed round-trips.
        seed = if version == 179 {
            // v179 stored the seed as an i32 (`LoadHeader:1956`).
            r.i32()?.to_string().parse().unwrap_or(0)
        } else {
            r.string()?.parse().unwrap_or(0)
        };
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
    world.seed = seed;
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

/// The world-header + flags block's read half — `LoadWorldFlags:1981`. The
/// full C#-order walk lives in [`sections::flags::read`] beside its write half
/// (the phase's stub lane); the parser keeps only this seam.
fn load_header_flags(r: &mut Reader, world: &mut World, version: i32) -> Result<()> {
    sections::flags::read(r, world, version)
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
            // The record's sparse `World.extras` byte, composed from the wall
            // paint and the coating bits. C#'s `Tile.CopyFrom` carries them
            // (`Tile.cs:142-153`), so the RLE-filled rows below get the same
            // byte the record's origin does (`LoadWorldTiles:2677-2682`).
            let mut extras = 0u8;

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
                    extras |= r.u8()? & 0x1f;
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
                    extras |= 0x20; // invisibleBlock
                }
                if b3 & 4 != 0 {
                    extras |= 0x40; // invisibleWall
                }
                if b3 & 0x10 != 0 {
                    extras |= 0x80; // fullbrightWall
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
                if extras != 0 {
                    world.set_extras_at(i, extras);
                }
                y += 1;
            }
        }
    }

    Ok(())
}

/// Thin `anyhow` adapter over `crate::binio::Reader`: a short read becomes the
/// parser's original `unexpected end of file at offset N` bail. All decoding
/// lives in `crate::binio`. `pub(crate)` so the section codecs (the flags
/// reader) can share it instead of re-wrapping.
pub(crate) struct Reader<'a>(crate::binio::Reader<'a>);

fn eof(pos: usize) -> anyhow::Error {
    anyhow!("unexpected end of file at offset {pos}")
}

impl<'a> Reader<'a> {
    pub(crate) fn new(data: &'a [u8]) -> Self {
        Self(crate::binio::Reader::new(data))
    }

    pub(crate) fn seek(&mut self, pos: usize) -> Result<()> {
        self.0
            .seek(pos)
            .ok_or_else(|| anyhow!("section pointer {pos} beyond EOF"))
    }

    pub(crate) fn skip(&mut self, n: usize) -> Result<()> {
        self.0.skip(n).ok_or_else(|| eof(self.0.pos()))
    }

    pub(crate) fn u8(&mut self) -> Result<u8> {
        self.0.u8().ok_or_else(|| eof(self.0.pos()))
    }

    pub(crate) fn bool(&mut self) -> Result<bool> {
        self.0.boolean().ok_or_else(|| eof(self.0.pos()))
    }

    pub(crate) fn i16(&mut self) -> Result<i16> {
        self.0.i16().ok_or_else(|| eof(self.0.pos()))
    }

    fn u16(&mut self) -> Result<u16> {
        self.0.u16().ok_or_else(|| eof(self.0.pos()))
    }

    pub(crate) fn i32(&mut self) -> Result<i32> {
        self.0.i32().ok_or_else(|| eof(self.0.pos()))
    }

    pub(crate) fn u64(&mut self) -> Result<u64> {
        self.0.u64().ok_or_else(|| eof(self.0.pos()))
    }

    pub(crate) fn f64(&mut self) -> Result<f64> {
        self.0.f64().ok_or_else(|| eof(self.0.pos()))
    }

    /// .NET `BinaryReader.ReadString`: 7-bit length prefix + UTF-8 payload,
    /// decoded lossily as before.
    pub(crate) fn string(&mut self) -> Result<String> {
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
        if version >= 179 {
            b.extend(cs_string("12345")); // seed text (v179 stored an i32)
            b.extend_from_slice(&7u64.to_le_bytes()); // generator version
        }
        if version >= 181 {
            b.extend_from_slice(&[0x22; 16]); // GUID
        }
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
            if version >= 112 {
                b.push(0); // GameMode as bool (absent below 112 — C#-faithful)
            }
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
        b.extend(flags_tail(version));
        b
    }

    /// The flags block after `_tempMoonPhase`, in C# order
    /// (`LoadWorldFlags:2076-2517`), every value the C# default (zero / false)
    /// and the counted lists at their C# lengths — the exact byte shape
    /// `sections::flags::read` walks. This is the table the reader is
    /// characterized against; the writer must produce the identical shape for
    /// an all-default world.
    fn flags_tail(version: i32) -> Vec<u8> {
        let mut b = Vec::new();
        // A run of `n` zero bytes.
        let z = |b: &mut Vec<u8>, n: usize| b.resize(b.len() + n, 0);

        z(&mut b, 2); // _tempBloodMoon, _tempEclipse
        z(&mut b, 8); // dungeonX, dungeonY
        z(&mut b, 1); // WorldGen.crimson
        z(&mut b, if version >= 118 { 11 } else { 10 }); // downedBoss1..downedSlimeKing
        z(&mut b, 7); // savedGoblin..downedPirates
        z(&mut b, 8); // shadowOrbSmashed, spawnMeteor, shadowOrbCount, altarCount, hardMode
        if version >= 257 {
            z(&mut b, 1); // afterPartyOfDoom
        }
        z(&mut b, 20); // invasionDelay, invasionSize, invasionType, invasionX
        if version >= 118 {
            z(&mut b, 8); // slimeRainTime
        }
        if version >= 113 {
            z(&mut b, 1); // sundialCooldown
        }
        z(&mut b, 9); // _tempRaining, _tempRainTime, _tempMaxRain
        z(&mut b, 12); // SavedOreTiers Cobalt/Mythril/Adamantite
        z(&mut b, 8); // setBG 0..7
        z(&mut b, 10); // cloudBGActive, numClouds, windSpeedTarget
        if version < 95 {
            return b;
        }
        z(&mut b, 4); // anglerWhoFinishedToday.Count — empty
        if version < 99 {
            return b;
        }
        z(&mut b, 1); // savedAngler
        if version < 101 {
            return b;
        }
        z(&mut b, 4); // anglerQuest
        if version < 104 {
            return b;
        }
        z(&mut b, 1); // savedStylist
        if version >= 129 {
            z(&mut b, 1); // savedTaxCollector
        }
        if version >= 201 {
            z(&mut b, 1); // savedGolfer
        }
        z(&mut b, 4); // invasionSizeStart
        z(&mut b, 4); // _tempCultistDelay
                      // BannerSystem.Load: the kill counts (the second array only from 289).
        b.extend_from_slice(&293i16.to_le_bytes()); // killCount.Length
        z(&mut b, 293 * 4);
        if version >= 289 {
            b.extend_from_slice(&293i16.to_le_bytes()); // claimableBanners.Length
            z(&mut b, 293 * 2);
        }
        z(&mut b, 1); // fastForwardTimeToDawn
        if version < 131 {
            return b;
        }
        z(&mut b, 9); // downedFishron..downedChristmasTree
        if version < 140 {
            return b;
        }
        z(&mut b, 9); // the four tower downeds, the four actives, LunarApocalypseIsUp
        if version >= 170 {
            z(&mut b, 10); // party manual/genuine/cooldown + celebrant count
        }
        if version >= 174 {
            z(&mut b, 13); // sandstorm happening/timeLeft/severity/intended
        }
        if version >= 178 {
            z(&mut b, 4); // savedBartender, DownedInvasionT1..T3
        }
        if version > 194 {
            z(&mut b, 1); // setBG(8) mushroomBG
        }
        if version >= 215 {
            z(&mut b, 1); // setBG(9) underworldBG
        }
        if version > 195 {
            z(&mut b, 3); // setBG(10..12) treeBG2..4
        }
        if version >= 204 {
            z(&mut b, 1); // combatBookWasUsed
        }
        if version >= 207 {
            z(&mut b, 7); // the lantern night snapshot
        }
        if version >= 211 {
            b.extend_from_slice(&13i32.to_le_bytes()); // TreeTops.Load: _variations.Length
            z(&mut b, 13 * 4);
        }
        if version >= 212 {
            z(&mut b, 2); // forceHalloweenForToday, forceXMasForToday
        }
        if version >= 216 {
            z(&mut b, 16); // Copper/Iron/Silver/Gold
        }
        if version >= 217 {
            z(&mut b, 3); // boughtCat/Dog/Bunny
        }
        if version >= 223 {
            z(&mut b, 2); // downedEmpressOfLight, downedQueenSlime
        }
        if version >= 240 {
            z(&mut b, 1); // downedDeerclops
        }
        if version >= 250 {
            z(&mut b, 1); // unlockedSlimeBlueSpawn
        }
        if version >= 251 {
            z(&mut b, 8); // unlockedMerchantSpawn..unlockedPrincessSpawn
        }
        if version >= 259 {
            z(&mut b, 1); // combatBookVolumeTwoWasUsed
        }
        if version >= 260 {
            z(&mut b, 1); // peddlersSatchelWasUsed
        }
        if version >= 261 {
            z(&mut b, 7); // unlockedSlimeGreen/Old/Purple/Rainbow/Red/Yellow/Copper
        }
        if version >= 264 {
            z(&mut b, 2); // fastForwardTimeToDusk, moondialCooldown
        }
        if version >= 287 {
            z(&mut b, 2); // forceHalloweenForever, forceXMasForever
        }
        if version >= 288 {
            z(&mut b, 1); // vampireSeed
        }
        if version >= 296 {
            z(&mut b, 1); // infectedSeed
        }
        if version >= 291 {
            z(&mut b, 8); // _tempMeteorShowerCount, _tempCoinRain
        }
        if version >= 297 {
            z(&mut b, 2); // teamBasedSpawnsSeed, extraSpawnPoints.Length
        }
        if version >= 304 {
            z(&mut b, 1); // dualDungeonsSeed
        }
        if version >= 323 {
            z(&mut b, 2); // moreLightningSeed, noLightningSeed
        }
        if version >= 299 {
            z(&mut b, 1); // WorldGen.Manifest — the empty .NET string
        }
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
        if version >= 135 {
            file.extend_from_slice(&[0u8; 20]); // FileMetadata
        }
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

        // Hand-computed from the C# table (all-default world, seed text "0"):
        // 70-byte prologue + 97 (u16 754 + the 95-byte importance bitmap) +
        // 66 (world head: name 12 + seed 2 + generator 8 + GUID 16 + world id 4
        // + bounds 16 + maxTilesY 4 + maxTilesX 4) + 2200 (the flags block) +
        // 16 tiles (8 columns x one air record [0x40, 5]) + 35 (the eight stub
        // sections: 2+2+6+4+4+4+12+1) + footer (1 + 12 + 4).
        assert_eq!(buf.len(), 2501);
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
        // Pointers are the section END offsets: the bitmap ends at 167, the
        // header+flags at 2433, the tiles at 2449 (each tile record is
        // [0x40, 5] — air, RLE 1-byte run of 5), then the eight stubs.
        let want: [i32; 11] = [
            167, 2433, 2449, 2451, 2453, 2459, 2463, 2467, 2471, 2483, 2484,
        ];
        for (i, p) in want.iter().enumerate() {
            let at = 26 + 4 * i;
            assert_eq!(&buf[at..at + 4], &p.to_le_bytes(), "pointer {i}");
        }
        // The file-format header tail: 754 tile types and the all-clear bitmap
        // (no world frame-important bits on a fresh `World::new`).
        assert_eq!(&buf[70..72], &754u16.to_le_bytes());
        assert_eq!(&buf[72..167], &[0u8; 95], "importance bitmap");
        // The world head: name, the seed text "0", the generator version, the
        // zero GUID, world id 0, the derived bounds and the dims.
        assert_eq!(&buf[167..168], &[11], "name length");
        assert_eq!(&buf[168..179], b"SaveFixture");
        assert_eq!(&buf[179..181], &[1, b'0'], "seed text \"0\"");
        assert_eq!(&buf[181..189], &0u64.to_le_bytes(), "generator version");
        assert_eq!(&buf[189..205], &[0u8; 16], "empty GUID");
        assert_eq!(&buf[205..209], &0i32.to_le_bytes(), "world id");
        assert_eq!(&buf[209..213], &0i32.to_le_bytes(), "leftWorld");
        assert_eq!(&buf[213..217], &(8 * 16i32).to_le_bytes(), "rightWorld");
        assert_eq!(&buf[217..221], &0i32.to_le_bytes(), "topWorld");
        assert_eq!(&buf[221..225], &(6 * 16i32).to_le_bytes(), "bottomWorld");
        assert_eq!(&buf[225..229], &6i32.to_le_bytes(), "maxTilesY");
        assert_eq!(&buf[229..233], &8i32.to_le_bytes(), "maxTilesX");
        // The flags block opens with the classic GameMode, the nine flavour
        // bools and the two creation-time longs (all defaults).
        assert_eq!(&buf[233..237], &0i32.to_le_bytes(), "GameMode");
        assert_eq!(&buf[237..246], &[0u8; 9], "flavour bools");
        assert_eq!(&buf[246..254], &[0u8; 8], "CreationTime");
        assert_eq!(&buf[254..262], &[0u8; 8], "LastPlayed");
        // The tile section: one [0x40, 5] record per column (b with the 1-byte
        // RLE bit, run 5). `pointers[1] = 2433` is the section's end, and the
        // header it ends is 66 + 2200 bytes, so the tiles start at 167 + that.
        let tiles_at = 2433;
        for col in 0..8 {
            assert_eq!(
                &buf[tiles_at + 2 * col..tiles_at + 2 * col + 2],
                &[0x40, 5],
                "column {col}"
            );
        }
        // The empty section encodings themselves.
        assert_eq!(&buf[2449..2451], &[0, 0], "chests count");
        assert_eq!(&buf[2451..2453], &[0, 0], "signs count");
        assert_eq!(&buf[2453..2457], &0i32.to_le_bytes(), "shimmered count");
        assert_eq!(&buf[2457..2458], &[0], "town terminator");
        assert_eq!(&buf[2458..2459], &[0], "tower terminator");
        assert_eq!(&buf[2459..2471], &[0u8; 12], "te + plates + rooms counts");
        assert_eq!(&buf[2471..2483], &[0u8; 12], "bestiary 3 counts");
        assert_eq!(&buf[2483..2484], &[0], "creative terminator");
        // The footer triple `(true, worldName, worldId)`.
        assert_eq!(&buf[2484..2485], &[1]);
        assert_eq!(buf[2485], 11, "name length prefix");
        assert_eq!(&buf[2486..2497], b"SaveFixture");
        assert_eq!(&buf[2497..2501], &0i32.to_le_bytes());
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
    fn save_writes_every_section_that_has_a_writer() {
        // Phase 6 landed the last two stub lanes (bestiary, creative powers),
        // so all eleven sections now have writers and `to_bytes`'s "data but no
        // writer" bail cannot trigger through the registry any more — this test
        // is the flip's record: the bag below used to be refused, one section
        // at a time. The bail itself stays in `to_bytes` as the guard for a
        // section added without a codec.
        let world = World::new("t".into(), 8, 6);
        let mut data = SectionData::default();
        data.creative_powers
            .records
            .push((0, sections::WldCreativeValue::Bool(true)));
        data.bestiary.kills.push(("Slime".into(), 1));
        data.bestiary.sights.push("Slime".into());
        let file = to_bytes(&world, &data, 1).expect("every section has a writer now");
        let table = SectionTable::open(&file).expect("opens");
        let loaded = load_sections(&table).expect("walks its sections");
        assert_eq!(loaded.bestiary.kills, data.bestiary.kills);
        assert_eq!(loaded.bestiary.sights, data.bestiary.sights);
        assert_eq!(loaded.creative_powers.records, data.creative_powers.records);
    }

    // --- the Phase 1 review's carried probes ---------------------------------

    #[test]
    fn load_sections_walks_a_legacy_six_pointer_table() {
        // The walk takes only the pointers the table has
        // (`def.pointer >= table.pointers.len()` → skip); the review verified
        // this by a throwaway probe, and it now has a committed home.
        let data = synthetic_file(200, &[2, 144], 160, 6);
        let table = SectionTable::open(&data).expect("opens");
        let sections = load_sections(&table).expect("a legacy table walks");
        assert_eq!(sections, SectionData::default());
    }

    #[test]
    fn save_over_a_pre_135_file_starts_the_revision_at_one() {
        // `FileMetadata` only exists from version 135, so a previous file
        // without one grants no revision: `FromCurrentSettings(0)` +
        // `IncrementAndWrite` = 1.
        let dir = scratch("pre-135");
        let path = dir.join("old.wld");
        let data = synthetic_file(100, &[2, 144], 160, 11);
        std::fs::write(&path, data).expect("write the old file");
        let world = World::new("O".into(), 8, 6);
        save(&path, &world, &SectionData::default()).expect("save over an old file");
        let saved = std::fs::read(&path).expect("read back");
        let table = SectionTable::open(&saved).expect("opens");
        assert_eq!(table.metadata().revision, 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    // --- the carried Phase 1 note: the negative section count ---------------

    #[test]
    fn the_negative_section_count_is_rejected_not_a_capacity_overflow() {
        // A crafted `i16 -1` count used to reach `Vec::with_capacity` and panic
        // with a capacity overflow; it is a clean bail now.
        let mut data = fixture();
        data[24..26].copy_from_slice(&(-1i16).to_le_bytes());
        assert!(parse(&data).is_err());
        assert!(SectionTable::open(&data).is_err());
    }

    // --- DW-2.1: the file-format header bytes -------------------------------

    #[test]
    fn test_DW_2_1_the_file_format_header_bytes_are_hand_computed() {
        let mut world = World::new("H".into(), 8, 6);
        world.frame_important = vec![false; crate::tiledata::TILE_COUNT];
        for kind in [2usize, 5, 753] {
            world.frame_important[kind] = true;
        }
        let buf = to_bytes(&world, &SectionData::default(), 1).expect("writes");

        // `SaveFileFormatHeader:1174-1178`: i32 326, the 20-byte metadata,
        // `i16 11` and the 44 pointer placeholder bytes — 70 bytes of prologue.
        assert_eq!(&buf[0..4], &VER.to_le_bytes());
        assert_eq!(
            &buf[4..12],
            &(27981915666277746u64 | (2u64 << 56)).to_le_bytes(),
            "magic | FileType.World"
        );
        assert_eq!(&buf[24..26], &11i16.to_le_bytes(), "section count");
        // `SaveFileFormatHeader:1181-1204`: `TileID.Count` = 754 and the
        // 95-byte bitmap — LSB-first, one bit per kind.
        assert_eq!(&buf[70..72], &754u16.to_le_bytes());
        let bitmap = &buf[72..167];
        assert_eq!(bitmap.len(), 95, "ceil(754 / 8)");
        assert_eq!(bitmap[0], (1 << 2) | (1 << 5), "kinds 2 and 5");
        // Kind 753 lands in byte 94, bit 1 — the partial trailing byte the `% 8
        // == 0` suppression only skips for an evenly-dividing count.
        assert_eq!(bitmap[94], 1 << (753 % 8));
        for (i, byte) in bitmap.iter().enumerate() {
            if i != 0 && i != 94 {
                assert_eq!(*byte, 0, "bitmap byte {i}");
            }
        }
        // The header section is exactly the bitmap: pointer 0 is its end.
        let ptr0 = i32::from_le_bytes(buf[26..30].try_into().unwrap());
        assert_eq!(ptr0 as usize, 167);
        // The reader recovers the same table.
        let table = SectionTable::open(&buf).expect("opens");
        assert_eq!(table.importance.len(), 754);
        assert!(table.importance[2] && table.importance[5] && table.importance[753]);
        assert!(!table.importance[0] && !table.importance[6]);
    }

    #[test]
    fn test_DW_2_1_the_importance_bitmap_is_empty_without_a_frame_table() {
        // A `World::new` world carries no frame table: the bitmap is still
        // C#'s 95 bytes, all clear.
        let world = World::new("H".into(), 8, 6);
        let buf = to_bytes(&world, &SectionData::default(), 1).expect("writes");
        assert_eq!(&buf[70..72], &754u16.to_le_bytes());
        assert_eq!(&buf[72..167], &[0u8; 95]);
    }

    // --- DW-2.2: the header + flags round-trip ------------------------------

    /// Every owned header/flags field at a distinct value.
    fn owned_world() -> World {
        let mut world = World::new("OwnedFields".into(), 300, 200);
        world.seed = 0xDEAD_BEEF;
        world.spawn_x = 71;
        world.spawn_y = 62;
        world.world_surface = 123.5;
        world.rock_layer = 205.25;
        world.time = 4321.0;
        world.day_time = false;
        world.moon_type = 5;
        world.moon_phase = 3;
        world.hell_back_style = 2;
        world.blood_moon = true;
        world.eclipse = true;
        world.downed_eoc = true;
        world.downed_eow = true;
        world.downed_skeletron = true;
        world.downed_queen_bee = true;
        world.downed_mech_boss1 = true;
        world.downed_mech_boss2 = true;
        world.downed_mech_boss3 = true;
        world.downed_plant_boss = true;
        world.downed_golem = true;
        world.downed_slime_king = true;
        world.saved_goblin = true;
        world.downed_goblins = true;
        world.downed_frost = true;
        world.downed_pirates = true;
        world.downed_martians = true;
        world.downed_deerclops = true;
        world.shadow_orb_smashed = true;
        world.spawn_meteor = true;
        world.shadow_orb_count = 7;
        world.altars_smashed = 3;
        world.hard_mode = true;
        world.invasion_delay = 5;
        world.invasion_size = 60;
        world.invasion_type = 1;
        world.invasion_x = 1234.5;
        world.invasion_size_start = 60;
        world.slime_rain_time = 123.0;
        world.sundial_cooldown = 4;
        world.moondial_cooldown = 2;
        world.fast_forward_to_dawn = true;
        world.fast_forward_to_dusk = true;
        world.unlocked_slime_rainbow_spawn = true;
        world
    }

    /// Asserts every owned header/flags field survives the write→read cycle
    /// (the field list is DW-2.2's).
    fn assert_owned_fields(want: &World, got: &World, context: &str) {
        assert_eq!(got.name, want.name, "{context}: name");
        assert_eq!(got.seed, want.seed, "{context}: seed");
        assert_eq!(
            (got.width, got.height),
            (want.width, want.height),
            "{context}: bounds"
        );
        assert_eq!(
            (got.spawn_x, got.spawn_y),
            (want.spawn_x, want.spawn_y),
            "{context}: spawn"
        );
        assert_eq!(
            got.world_surface, want.world_surface,
            "{context}: world surface"
        );
        assert_eq!(got.rock_layer, want.rock_layer, "{context}: rock layer");
        assert_eq!(got.time, want.time, "{context}: time");
        assert_eq!(got.day_time, want.day_time, "{context}: day time");
        assert_eq!(got.moon_type, want.moon_type, "{context}: moon type");
        assert_eq!(got.moon_phase, want.moon_phase, "{context}: moon phase");
        assert_eq!(got.blood_moon, want.blood_moon, "{context}: blood moon");
        assert_eq!(got.eclipse, want.eclipse, "{context}: eclipse");
        assert_eq!(
            got.hell_back_style, want.hell_back_style,
            "{context}: hell back style"
        );
        assert_eq!(got.downed_eoc, want.downed_eoc, "{context}: downedBoss1");
        assert_eq!(got.downed_eow, want.downed_eow, "{context}: downedBoss2");
        assert_eq!(
            got.downed_skeletron, want.downed_skeletron,
            "{context}: downedBoss3"
        );
        assert_eq!(
            got.downed_queen_bee, want.downed_queen_bee,
            "{context}: queen bee"
        );
        assert_eq!(
            got.downed_mech_boss1, want.downed_mech_boss1,
            "{context}: mech 1"
        );
        assert_eq!(
            got.downed_mech_boss2, want.downed_mech_boss2,
            "{context}: mech 2"
        );
        assert_eq!(
            got.downed_mech_boss3, want.downed_mech_boss3,
            "{context}: mech 3"
        );
        assert_eq!(
            got.downed_plant_boss, want.downed_plant_boss,
            "{context}: plant boss"
        );
        assert_eq!(got.downed_golem, want.downed_golem, "{context}: golem");
        assert_eq!(
            got.downed_slime_king, want.downed_slime_king,
            "{context}: slime king"
        );
        assert_eq!(
            got.saved_goblin, want.saved_goblin,
            "{context}: saved goblin"
        );
        assert_eq!(
            got.downed_goblins, want.downed_goblins,
            "{context}: goblins"
        );
        assert_eq!(
            got.downed_frost, want.downed_frost,
            "{context}: frost legion"
        );
        assert_eq!(
            got.downed_pirates, want.downed_pirates,
            "{context}: pirates"
        );
        assert_eq!(
            got.downed_martians, want.downed_martians,
            "{context}: martians"
        );
        assert_eq!(
            got.downed_deerclops, want.downed_deerclops,
            "{context}: deerclops"
        );
        assert_eq!(
            got.shadow_orb_smashed, want.shadow_orb_smashed,
            "{context}: orbs"
        );
        assert_eq!(got.spawn_meteor, want.spawn_meteor, "{context}: meteor");
        assert_eq!(
            got.shadow_orb_count, want.shadow_orb_count,
            "{context}: orb count"
        );
        assert_eq!(got.altars_smashed, want.altars_smashed, "{context}: altars");
        assert_eq!(got.hard_mode, want.hard_mode, "{context}: hard mode");
        assert_eq!(
            got.invasion_delay, want.invasion_delay,
            "{context}: invasion delay"
        );
        assert_eq!(
            got.invasion_size, want.invasion_size,
            "{context}: invasion size"
        );
        assert_eq!(
            got.invasion_type, want.invasion_type,
            "{context}: invasion type"
        );
        assert_eq!(got.invasion_x, want.invasion_x, "{context}: invasion x");
        assert_eq!(
            got.invasion_size_start, want.invasion_size_start,
            "{context}: invasion start"
        );
        assert_eq!(
            got.slime_rain_time, want.slime_rain_time,
            "{context}: slime rain time"
        );
        assert_eq!(
            got.sundial_cooldown, want.sundial_cooldown,
            "{context}: sundial"
        );
        assert_eq!(
            got.moondial_cooldown, want.moondial_cooldown,
            "{context}: moondial"
        );
        assert_eq!(
            got.fast_forward_to_dawn, want.fast_forward_to_dawn,
            "{context}: fast-forward dawn"
        );
        assert_eq!(
            got.fast_forward_to_dusk, want.fast_forward_to_dusk,
            "{context}: fast-forward dusk"
        );
        assert_eq!(
            got.unlocked_slime_rainbow_spawn, want.unlocked_slime_rainbow_spawn,
            "{context}: rainbow slime"
        );
    }

    #[test]
    fn test_DW_2_2_the_header_and_flags_round_trip_owned_fields() {
        let world = owned_world();
        let buf = to_bytes(&world, &SectionData::default(), 1).expect("writes");
        let back = parse(&buf).expect("the saved world parses");
        assert_owned_fields(&world, &back, "cycle");
    }

    /// The v326 flags block straight from the C# table, independent of the
    /// writer: the owned fields from the world, every unowned field its C#
    /// default (zeros / false / the counted lists at their C# lengths).
    fn expected_flags_block(world: &World) -> Vec<u8> {
        let mut b = Vec::new();
        let z = |b: &mut Vec<u8>, n: usize| b.resize(b.len() + n, 0);
        b.extend_from_slice(&0i32.to_le_bytes()); // GameMode — unowned
        z(&mut b, 9); // the nine world-flavour bools — unowned
        z(&mut b, 16); // CreationTime, LastPlayed — no wall clock
        b.push(world.moon_type);
        b.extend(flags_tail_prefix(world));
        // The tail opens with the two owned event flags; the fixture table
        // carries their C# defaults, so the world's values go in first.
        let mut tail = flags_tail(VER);
        tail[0] = u8::from(world.blood_moon);
        tail[1] = u8::from(world.eclipse);
        b.extend(tail);
        b
    }

    /// The owned middle of the flags table: the back styles (hell only), the
    /// spawn/layers/time block — written between the 17 zero i32s of
    /// `treeX`..`jungleBackStyle` and the `_tempBloodMoon` tail.
    fn flags_tail_prefix(world: &World) -> Vec<u8> {
        let mut b = Vec::new();
        b.resize(3 * 4 + 4 * 4 + 3 * 4 + 4 * 4 + 2 * 4, 0); // tree/cave arrays + ice/jungle
        b.extend_from_slice(&(world.hell_back_style as i32).to_le_bytes());
        b.extend_from_slice(&world.spawn_x.to_le_bytes());
        b.extend_from_slice(&world.spawn_y.to_le_bytes());
        b.extend_from_slice(&world.world_surface.to_le_bytes());
        b.extend_from_slice(&world.rock_layer.to_le_bytes());
        b.extend_from_slice(&world.time.to_le_bytes());
        b.push(u8::from(world.day_time));
        b.extend_from_slice(&(world.moon_phase as i32).to_le_bytes());
        b
    }

    #[test]
    fn test_DW_2_2_the_unowned_flag_fields_are_csharp_defaults() {
        // A world whose owned flag values are all at their fresh-world state:
        // every other byte of the block must be the C# default the table
        // spells. (The owned values themselves are pinned by the round-trip
        // test above.)
        let world = World::new("Defaults".into(), 300, 200);
        let buf = to_bytes(&world, &SectionData::default(), 1).expect("writes");

        // The flags block sits inside pointer 1's section, after the world
        // head: name (1 + 8) + seed text "0" (2) + 8 + 16 + 4 + 16 + 4 + 4.
        let ptr0 = i32::from_le_bytes(buf[26..30].try_into().unwrap()) as usize;
        let ptr1 = i32::from_le_bytes(buf[30..34].try_into().unwrap()) as usize;
        let head = (1 + 8) + 2 + 8 + 16 + 4 + 16 + 4 + 4;
        let flags_at = ptr0 + head;
        let want = expected_flags_block(&world);
        assert_eq!(want.len(), 2200, "the v326 flags block's size");
        assert_eq!(flags_at + want.len(), ptr1, "the block fills pointer 1");
        assert_eq!(
            &buf[flags_at..ptr1],
            &want[..],
            "the flags block matches the C# table byte for byte"
        );
    }

    #[test]
    fn test_DW_2_2_a_fresh_world_round_trips_through_wld() {
        // The never-loaded world entering `.wld` for the first time: a plain
        // `World::new` with no frame table still writes and reads back every
        // owned field (the derived spawn/layer defaults included).
        let world = World::new("Fresh".into(), 400, 300);
        let buf = to_bytes(&world, &SectionData::default(), 1).expect("writes");
        let back = parse(&buf).expect("the saved world parses");
        assert_owned_fields(&world, &back, "fresh");
    }

    #[test]
    fn test_DW_2_2_the_old_version_gates_walk_the_flags_block() {
        // A v100 file (< 112: no GameMode byte, < 141: no creation time, < 95:
        // the block stops after the wind target) still parses, and the owned
        // fields land.
        let data = synthetic_file(100, &[2, 144], 160, 11);
        let world = parse(&data).expect("v100 loads");
        assert_eq!((world.spawn_x, world.spawn_y), (4, 5));
        assert_eq!(world.moon_type, 3);
        assert_eq!(world.time, 1234.0);
    }

    // --- DW-2.3: the tile section round-trip --------------------------------

    /// A tile at (x, y) with an optional frame: non-important kinds carry
    /// `-1` frames, the form the reader leaves them in (`LoadWorldTiles:2605`'s
    /// `frameX = frameY = -1` branch).
    fn tile(kind: u16, frame_x: i16, frame_y: i16) -> Tile {
        let mut t = Tile {
            kind,
            frame_x,
            frame_y,
            ..Default::default()
        };
        if kind != 0 {
            t.flags |= FLAG_ACTIVE;
        }
        t
    }

    /// A world whose tile section exercises every payload branch: RLE runs of
    /// one byte, two bytes and none, 16-bit tiles and walls, tile and wall
    /// paints, all four liquid kinds, slopes, the actuator, the coatings and
    /// the non-batchable kinds.
    fn tile_fixture_world() -> World {
        let mut world = World::new("Tiles".into(), 2, 300);
        world.frame_important = vec![false; crate::tiledata::TILE_COUNT];
        for kind in [2usize, 144, 658] {
            world.frame_important[kind] = true;
        }
        let set = |w: &mut World, x: i32, y: i32, mut t: Tile| {
            let idx = w.index(x, y);
            if !w
                .frame_important
                .get(t.kind as usize)
                .copied()
                .unwrap_or(false)
            {
                t.frame_x = -1;
                t.frame_y = -1;
            }
            w.tiles[idx] = t;
        };

        // Column 0.
        set(&mut world, 0, 0, tile(0, -1, -1)); // air
        for y in 1..10 {
            let mut t = tile(2, 90, 18);
            t.set_paint(3);
            set(&mut world, 0, y, t);
        }
        // Kind 144: the load fix-up forces its frameY to 0, so the fixture
        // carries 0 there (`LoadWorldTiles:2593-2595`).
        set(&mut world, 0, 10, tile(144, 10, 0));
        for y in 11..14 {
            let mut t = tile(658, 36, 54); // a 16-bit tile id
            t.wall = 308; // a 16-bit wall id
            t.liquid = 128;
            t.flags |= FLAG_LIQUID_SHIMMER;
            set(&mut world, 0, y, t);
            world.set_wall_paint(0, y, 5);
            let idx = world.index(0, y);
            world.tiles[idx].set_paint(12);
        }
        let mut t = tile(2, -1, -1); // non-important: frames -1
        t.flags |= FLAG_ACTUATOR | FLAG_INACTIVE;
        t.set_slope(3);
        set(&mut world, 0, 14, t);
        // A wall with its paint and the invisible-block coating.
        let mut t = tile(0, -1, -1);
        t.wall = 5;
        set(&mut world, 0, 15, t);
        world.set_wall_paint(0, 15, 9);
        world.set_invisible_block(0, 15, true);
        // The three liquid kinds and water.
        let mut t = tile(0, -1, -1);
        t.liquid = 200;
        t.flags |= FLAG_LIQUID_HONEY;
        set(&mut world, 0, 16, t);
        let mut t = tile(0, -1, -1);
        t.liquid = 100;
        t.flags |= FLAG_LIQUID_LAVA;
        set(&mut world, 0, 17, t);
        let mut t = tile(0, -1, -1);
        t.liquid = 77;
        set(&mut world, 0, 18, t);
        let mut t = tile(0, -1, -1);
        t.liquid = 128;
        t.flags |= FLAG_LIQUID_SHIMMER;
        set(&mut world, 0, 19, t);
        // Fullbright + the two wall coatings.
        let mut t = tile(2, 90, 18);
        t.flags |= FLAG_FULLBRIGHT;
        set(&mut world, 0, 20, t);
        world.set_invisible_wall(0, 20, true);
        world.set_fullbright_wall(0, 20, true);
        // Column 0's rows 21..300 stay air: one two-byte RLE run.

        // Column 1: the non-batchable kinds write no run at all.
        for y in 0..6 {
            set(&mut world, 1, y, tile(423, 18 * y as i16, 0));
        }
        // A half brick on a solid kind, then air for the rest of the column.
        let mut t = tile(1, -1, -1); // stone: non-important in the fixture
        t.flags |= FLAG_HALF_BRICK;
        set(&mut world, 1, 6, t);
        // The reader's `frameX = frameY = -1` branch (`LoadWorldTiles:2605`)
        // applies to every non-important kind, including the untouched cells;
        // the framer recomputes those frames from -1, so the reset is lossless.
        for t in &mut world.tiles {
            let important = world
                .frame_important
                .get(t.kind as usize)
                .copied()
                .unwrap_or(false);
            if !important {
                t.frame_x = -1;
                t.frame_y = -1;
            }
        }
        world
    }

    #[test]
    fn test_DW_2_3_the_tile_section_round_trips_every_payload_branch() {
        let world = tile_fixture_world();
        let buf = to_bytes(&world, &SectionData::default(), 1).expect("writes");
        let back = parse(&buf).expect("the saved world parses");
        assert_eq!(back.tiles, world.tiles, "the tile array survives the cycle");
        assert_eq!(
            back.extras, world.extras,
            "the sparse extras survive the cycle"
        );
        // The branches the fixture promised, spot-checked on the way back: the
        // 16-bit tile and wall pair, the two water shimmer kinds, the painted
        // wall and the three coatings.
        let rich = world.index(0, 11);
        assert_eq!(back.tiles[rich].kind, 658, "a 16-bit tile id");
        assert_eq!(back.tiles[rich].wall, 308, "a 16-bit wall id");
        assert_ne!(back.tiles[rich].flags & FLAG_LIQUID_SHIMMER, 0);
        assert_ne!(
            back.tiles[world.index(0, 19)].flags & FLAG_LIQUID_SHIMMER,
            0
        );
        assert_eq!(back.wall_paint(0, 15), 9, "the wall paint");
        assert!(back.invisible_block(0, 15));
        assert!(back.invisible_wall(0, 20) && back.fullbright_wall(0, 20));
        // Column 0's tail is one 278-tile run: its members read back identical,
        // and the 2-byte RLE encoding of such a run has its own test.
        assert_eq!(
            back.tiles[world.index(0, 21)],
            back.tiles[world.index(0, 22)]
        );
        assert_eq!(
            back.tiles[world.index(0, 299)],
            back.tiles[world.index(0, 21)]
        );
    }

    #[test]
    fn test_DW_2_3_an_air_column_encodes_as_one_short_rle_record() {
        let world = World::new("a".into(), 1, 8);
        let mut w = Writer::new();
        sections::tiles::write(&mut w, &world);
        // One record: b = 0x40 (the 1-byte RLE bit) and the run 7.
        assert_eq!(w.into_inner(), vec![0x40, 7]);
    }

    #[test]
    fn test_DW_2_3_a_long_air_column_encodes_the_two_byte_rle() {
        let world = World::new("a".into(), 1, 300);
        let mut w = Writer::new();
        sections::tiles::write(&mut w, &world);
        // b = 0x80 (the 2-byte RLE bit), the run 299 as low-then-high bytes.
        assert_eq!(w.into_inner(), vec![0x80, 43, 1]);
    }

    #[test]
    fn test_DW_2_3_a_non_batchable_kind_writes_no_rle_run() {
        let mut world = World::new("a".into(), 1, 4);
        world.frame_important = vec![false; crate::tiledata::TILE_COUNT];
        world.frame_important[423] = true;
        for y in 0..4 {
            let idx = world.index(0, y);
            world.tiles[idx] = tile(423, 18 * y as i16, 0);
        }
        let mut w = Writer::new();
        sections::tiles::write(&mut w, &world);
        // Four standalone records: b = 0x22 (active + the 16-bit-kind bit,
        // because 423 > 255), the kind in two bytes, the frames — and no RLE
        // count, because 423 is not batchable (`TileID.cs:484`).
        let mut want = Vec::new();
        for y in 0..4i16 {
            let kind_bytes = [423u16 as u8, (423u16 >> 8) as u8];
            want.extend_from_slice(&[0x22, kind_bytes[0], kind_bytes[1]]);
            want.extend_from_slice(&(18 * y).to_le_bytes());
            want.extend_from_slice(&0i16.to_le_bytes());
        }
        assert_eq!(w.into_inner(), want);
    }

    #[test]
    fn test_DW_2_3_the_rle_run_detection_matches_is_the_same_as() {
        let mut world = World::new("a".into(), 1, 4);
        world.frame_important = vec![false; crate::tiledata::TILE_COUNT];
        world.frame_important[2] = true;
        for y in 0..4 {
            let idx = world.index(0, y);
            world.tiles[idx] = tile(2, 90, 18);
        }
        let idx = world.index(0, 3);
        world.tiles[idx].set_slope(1);
        let mut w = Writer::new();
        sections::tiles::write(&mut w, &world);
        // Rows 0..2 batch (the slope difference is `sTileHeader`, which
        // `isTheSameAs` compares first); row 3 is its own record with the
        // chained header: b = 3 (active + "b2 follows"), b2 = 0x20 (slope 1).
        let want = vec![0x42, 2, 90, 0, 18, 0, 2, 0x03, 0x20, 2, 90, 0, 18, 0];
        assert_eq!(w.into_inner(), want);
    }

    #[test]
    fn test_DW_2_3_kind_144_frame_y_is_forced_to_zero() {
        // The C# load fix-up (`LoadWorldTiles:2593-2595`) zeroes a kind-144
        // frameY: the bytes round-trip through the reader with 0 there — the
        // documented asymmetry, not a writer bug.
        let mut world = World::new("a".into(), 1, 1);
        world.frame_important = vec![false; crate::tiledata::TILE_COUNT];
        world.frame_important[144] = true;
        world.tiles[0] = tile(144, 10, 200);
        let buf = to_bytes(&world, &SectionData::default(), 1).expect("writes");
        let mut w = Writer::new();
        sections::tiles::write(&mut w, &world);
        assert_eq!(
            w.into_inner(),
            vec![0x02, 144, 10, 0, 200, 0],
            "the byte form"
        );
        let back = parse(&buf).expect("parses");
        assert_eq!(back.tiles[0].frame_y, 0);
    }

    #[test]
    fn test_DW_2_3_a_non_important_kind_comes_back_without_frames() {
        // The reader's `frameX = frameY = -1` branch (the C# mirrors it): the
        // writer emits no frame bytes for a non-important kind, so its frames
        // do not survive the cycle.
        let mut world = World::new("a".into(), 1, 1);
        world.frame_important = vec![false; crate::tiledata::TILE_COUNT];
        world.tiles[0] = tile(3, 90, 18);
        let buf = to_bytes(&world, &SectionData::default(), 1).expect("writes");
        let back = parse(&buf).expect("parses");
        assert_eq!(back.tiles[0].kind, 3);
        assert_eq!((back.tiles[0].frame_x, back.tiles[0].frame_y), (-1, -1));
    }

    #[test]
    fn test_DW_2_3_the_last_row_of_a_column_never_starts_a_run() {
        // A one-tall world: the run scan has nothing below it, so the record
        // carries no RLE bytes at all.
        let world = World::new("a".into(), 1, 1);
        let mut w = Writer::new();
        sections::tiles::write(&mut w, &world);
        assert_eq!(
            w.into_inner(),
            vec![0],
            "a single air tile is one zero byte"
        );
    }

    #[test]
    fn a_truncated_tile_section_is_rejected_not_a_panic() {
        let world = tile_fixture_world();
        let buf = to_bytes(&world, &SectionData::default(), 1).expect("writes");
        let tiles_at = i32::from_le_bytes(buf[30..34].try_into().unwrap()) as usize;
        for cut in [tiles_at, tiles_at + 5, tiles_at + 100] {
            let cut = cut.min(buf.len());
            assert!(parse(&buf[..cut]).is_err(), "truncated at {cut}");
        }
    }

    // --- DW-2.4: the batching set -------------------------------------------

    #[test]
    fn test_DW_2_4_the_batching_set_is_pinned_to_the_csharp_default() {
        use crate::tiledata::SET_ALLOWS_SAVE_COMPRESSION_BATCHING;
        // `TileID.cs:484`: `CreateBoolSet(true, 520, 423, 723, 724)` — the
        // default is true and the four listed kinds are the exceptions.
        for kind in [423u16, 520, 723, 724] {
            assert!(
                !SET_ALLOWS_SAVE_COMPRESSION_BATCHING.has(kind),
                "kind {kind} does not batch"
            );
        }
        for kind in [0u16, 2, 144, 658, 753] {
            assert!(
                SET_ALLOWS_SAVE_COMPRESSION_BATCHING.has(kind),
                "kind {kind} batches"
            );
        }
        assert!(
            !SET_ALLOWS_SAVE_COMPRESSION_BATCHING.has(754),
            "past TileID.Count"
        );
    }

    // --- DW-2.5: citations + the full suite ---------------------------------

    #[test]
    fn test_DW_2_5_every_writer_block_carries_its_citation() {
        let flags = include_str!("sections/flags.rs");
        assert!(flags.contains("WorldFile.cs:1238"), "flags write anchor");
        assert!(flags.contains("WorldFile.cs:1981"), "flags read anchor");
        assert!(
            flags.contains("WorldFile.cs:1040"),
            "the SetTempToOngoing snapshot rule reference"
        );
        let header = include_str!("sections/header.rs");
        assert!(header.contains("WorldFile.cs:1181"), "bitmap anchor");
        assert!(header.contains("WorldFile.cs:1221"), "world head anchor");
        let tiles = include_str!("sections/tiles.rs");
        assert!(tiles.contains("WorldFile.cs:1432"), "tiles write anchor");
        assert!(tiles.contains("WorldFile.cs:2519"), "the reader's anchor");
        assert!(tiles.contains("Tile.cs:155"), "the isTheSameAs anchor");
        assert!(tiles.contains("TileID.cs:484"), "the batching set anchor");
    }
}
