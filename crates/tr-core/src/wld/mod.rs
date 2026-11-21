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

/// A loaded `.wld` with every section the file carries beside its world —
/// `LoadWorld_Version2`'s two halves in one read.
#[derive(Clone, Debug)]
pub struct LoadedWorld {
    pub world: World,
    /// The eight sections past the tiles (`load_sections`'s output). Empty
    /// when the file's pointer table stops before them.
    pub sections: SectionData,
    /// `WorldFile._versionNumber` (`LoadFileFormatHeader:1902`): the format
    /// version the load-side fix-ups gate on (the chests' `< 115` dresser
    /// pass, `LoadChests:2801-2804`).
    pub version: i32,
}

/// [`load`] plus [`load_sections`] over the one read — the app's load entry
/// for a retail `.wld` (the native `.trwld` carries these states elsewhere).
pub fn load_with_sections(path: &Path) -> Result<LoadedWorld> {
    let data = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    parse_with_sections(&data).with_context(|| format!("parsing {}", path.display()))
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
    parse_table(&mut table).map(|loaded| loaded.world)
}

/// [`parse`] plus every section the file carries
/// (`LoadWorld_Version2:1781-1850`): the world first, then the section codecs
/// over the same bytes — one file read, in the C#'s order.
pub fn parse_with_sections(data: &[u8]) -> Result<LoadedWorld> {
    let mut table = SectionTable::open(data)?;
    parse_table(&mut table)
}

/// The shared body of [`parse`]/[`parse_with_sections`]: the world sections
/// (header, tiles), the pointer sanity walk, then the load side of the
/// section table. `importance` moves out of the table last, so it stays
/// available to the pointer checks.
fn parse_table(table: &mut SectionTable<'_>) -> Result<LoadedWorld> {
    let data = table.data;
    let version = table.version;

    // --- header section ----------------------------------------------------
    if table.pointers.is_empty() {
        bail!("world has no section table");
    }
    let mut r = Reader::new(data);
    r.seek(table.pointers[0])?;
    let name = r.string()?;
    let mut seed = 0u64;
    let mut generator_version = 0u64;
    let mut unique_id = [0u8; 16];
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
        generator_version = r.u64()?;
    }
    if version >= 181 {
        for b in &mut unique_id {
            *b = r.u8()?;
        }
    }
    let world_id = r.i32()?;
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
    world.generator_version = generator_version;
    world.unique_id = unique_id;
    world.world_id = world_id;
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
    // backwards is a hard reject); `load_sections` re-checks each pointer it
    // walks below (`LoadWorld_Version2:1781+`'s end-offset assertions).
    skip_sections(&r, &table)?;

    // --- the eight sections past the tiles (`LoadWorld_Version2:1781-1850`) --
    let sections = load_sections(table)?;

    world.frame_important = std::mem::take(&mut table.importance);
    // `WorldFile.cs:730`: `CheckSavedOreTiers` runs after a successful load —
    // a pre-216 file (or a -1 quartet) re-derives the four metals from the
    // tiles. `spawnTileY` keeps the file's value (`LoadWorldFlags:2069`); the
    // old `compute_spawn_y` override was a deviation, removed with S1-J-3b.
    check_saved_ore_tiers(&mut world);
    Ok(LoadedWorld {
        world,
        sections,
        version,
    })
}

/// `WorldFile.CheckSavedOreTiers` (`WorldFile.cs:794-849`): when any of the
/// four pre-hardmode tiers is -1, count the ore tiles
/// (`WorldGen.CountTileTypesInWorld(7, 166, 6, 167, 9, 168, 8, 169)`,
/// `WorldGen.cs:86885-86907` — active tiles only, first matching kind wins)
/// and pick per pair: the classic metal when its count is strictly greater,
/// the alternate otherwise. All four pairs are re-derived together, as the
/// C# does.
fn check_saved_ore_tiers(world: &mut World) {
    {
        let t = &world.ore_tiers;
        if t.copper != -1 && t.iron != -1 && t.silver != -1 && t.gold != -1 {
            return;
        }
    }
    const KINDS: [u16; 8] = [7, 166, 6, 167, 9, 168, 8, 169];
    let mut counts = [0i32; 8];
    for tile in &world.tiles {
        if !tile.active() {
            continue;
        }
        for (k, &kind) in KINDS.iter().enumerate() {
            if tile.kind == kind {
                counts[k] += 1;
                break;
            }
        }
    }
    let t = &mut world.ore_tiers;
    t.copper = if counts[0] > counts[1] { 7 } else { 166 };
    t.iron = if counts[2] > counts[3] { 6 } else { 167 };
    t.silver = if counts[4] > counts[5] { 9 } else { 168 };
    t.gold = if counts[6] > counts[7] { 8 } else { 169 };
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

/// The save side of one of the eleven sections.
/// `SaveWorld_Version2`'s caller contract: a section that claims records it
/// has no writer for would emit a section whose pointer cannot be honoured,
/// so it is refused rather than written corrupt. Extracted from [`to_bytes`]
/// so the guard stays exercisable when the frozen [`sections::SAVE_DEFS`]
/// registry has no unimplemented lane left to trip it — a future section
/// phase (or a registry test) passes its own def list here.
fn refuse_unwritten(defs: &[sections::SaveDef], data: &SectionData) -> Result<()> {
    for def in defs {
        if !(def.implemented)() && !(def.empty)(data) {
            bail!(
                "section {} (pointer {}) has data but no writer yet",
                def.name,
                def.pointer
            );
        }
    }
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
    refuse_unwritten(sections::SAVE_DEFS, data)?;
    let mut bodies: Vec<Vec<u8>> = Vec::with_capacity(SECTION_COUNT);
    for def in sections::SAVE_DEFS {
        let mut w = Writer::new();
        (def.write)(&mut w, world, data, version);
        bodies.push(w.into_inner());
    }

    // --- footer (`SaveFooter:1752`) -----------------------------------------
    let mut footer = Writer::new();
    footer.boolean(true);
    footer.cs_string(&world.name);
    footer.i32(world.world_id); // `Main.ActiveWorldFileData.WorldId` (`WorldFile.cs:1756`)

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
            // byte the record's origin does (`LoadWorldTiles:2708-2713`).
            let mut extras = 0u8;
            // The four wire colours the record carries — `CopyFrom`
            // (`LoadWorldTiles:2711`) copies them into every row of the run.
            let mut wires = sections::tiles::WireBits::default();

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
                // `WorldFile.cs:2614-2639`: the three `sTileHeader` wire
                // colours (`wire()`/`wire2()`/`wire3()`) and the slope.
                if b1 & 2 != 0 {
                    wires.red = true;
                }
                if b1 & 4 != 0 {
                    wires.blue = true;
                }
                if b1 & 8 != 0 {
                    wires.green = true;
                }
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
                // `WorldFile.cs:2651-2654`: `wire4()` — the yellow wire, a
                // `bTileHeader` bit.
                if b2 & 0x20 != 0 {
                    wires.yellow = true;
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
                // `WorldFile.cs:2711`: `Main.tile[i, j].CopyFrom(tile)` — the
                // wire colours travel with the copied header into every row
                // of the RLE run.
                wires.apply_to(world, x, y);
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

    pub(crate) fn f32(&mut self) -> Result<f32> {
        self.0.f32().ok_or_else(|| eof(self.0.pos()))
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
        // flags: GameMode (i32 from 209) + the nine seed bools, EACH gated on
        // its own introduction version (`LoadWorldFlags:1983-2024`: 222/227/
        // 238/239/241/249/266/267/302 — not all nine from 209), creation
        // time, last played.
        if version >= 209 {
            b.extend_from_slice(&[0u8; 4]); // GameMode i32
            for min_version in [222, 227, 238, 239, 241, 249, 266, 267, 302] {
                if version >= min_version {
                    b.push(0);
                }
            }
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
        // at a time.
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

    /// F5 (Phase-6 review N5): the "section has data but no writer" guard,
    /// restored as a direct test. `SAVE_DEFS` is frozen and fully implemented,
    /// so the guard is exercised through the function `to_bytes` calls with a
    /// synthetic def list — exactly what a future section phase (a new stub
    /// lane, or a codec temporarily flipped off) hits.
    #[test]
    fn test_DW_7_5_a_section_with_data_and_no_writer_is_refused() {
        // A lane over the chests bag: `implemented` and the message's name are
        // the only knobs, so one builder covers both arms (the struct is not
        // `Copy`, hence a function rather than a value).
        fn lane(implemented: fn() -> bool) -> sections::SaveDef {
            sections::SaveDef {
                pointer: 3,
                name: "future section",
                write: |_, _, _, _| {},
                implemented,
                empty: |d: &SectionData| d.chests.is_empty(),
            }
        }
        // Empty: the lane is allowed to stay unwritten (the empty encoding).
        assert!(
            refuse_unwritten(&[lane(|| false)], &SectionData::default()).is_ok(),
            "an empty section needs no writer"
        );
        // One chest record: the save must refuse rather than emit a section
        // whose pointer cannot be honoured.
        let mut data = SectionData::default();
        data.chests.push(sections::WldChest::default());
        let err = refuse_unwritten(&[lane(|| false)], &data)
            .expect_err("a data-carrying section without a writer is refused");
        let text = format!("{err:#}");
        assert!(
            text.contains("future section") && text.contains("pointer 3"),
            "the message names the section: {text}"
        );
        // A written lane with the same data is fine (the implemented check is
        // what makes the guard one-sided).
        assert!(refuse_unwritten(&[lane(sections::chests::implemented)], &data).is_ok());
        // And the real registry — every lane implemented — writes the same bag
        // through `to_bytes` without the guard firing.
        let world = World::new("t".into(), 8, 6);
        let file = to_bytes(&world, &data, 1).expect("every real lane has a writer");
        let table = SectionTable::open(&file).expect("opens");
        let loaded = load_sections(&table).expect("walks its sections");
        assert_eq!(loaded.chests.len(), 1, "the chest record rides the file");
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
        // S1-D-3 W-9: the fishron flag round-trips too.
        world.downed_fishron = true;
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
        // S1-J-3b: the creation clock and a non-default tier sweep.
        world.creation_time = 5250991586427387904i64;
        world.last_played = 5250991586427387905i64;
        world.ore_tiers = crate::ores::SavedOreTiers {
            copper: 166,
            iron: 167,
            silver: 168,
            gold: 169,
            cobalt: 221,
            mythril: 222,
            adamantite: 223,
        };
        // S1-J-3b: the newly-owned saved-NPC / quest / event / tower / unlock
        // flags, each at a distinct non-default value.
        world.saved_wizard = true;
        world.saved_mech = true;
        world.saved_angler = true;
        world.angler_quest = 17;
        world.saved_stylist = true;
        world.saved_tax_collector = true;
        world.saved_golfer = true;
        world.downed_ancient_cultist = true;
        world.downed_moon_lord = true;
        world.downed_queen_slime = true;
        world.downed_tower_solar = true;
        world.downed_tower_vortex = true;
        world.downed_tower_nebula = true;
        world.downed_tower_stardust = true;
        world.tower_active_solar = true;
        world.tower_active_vortex = true;
        world.tower_active_nebula = true;
        world.tower_active_stardust = true;
        world.lunar_apocalypse_is_up = true;
        world.unlocked_party_girl = true;
        world.unlocked_princess = true;
        // Gate V3 (S1-P-i35): the two newly-owned flag bytes — the combat
        // book (WorldFile.cs:1379, v204) and the red slime unlock
        // (`:1413`, v261) — ride the round-trip at non-default values.
        world.combat_book_was_used = true;
        world.unlocked_slime_red_spawn = true;
        // W4-g71: the three new setBG style bytes at non-default values.
        world.snow_bg_style = 7;
        world.desert_bg_style = 51;
        world.ocean_bg_style = 6;
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
        assert_eq!(got.downed_fishron, want.downed_fishron, "{context}: fishron");
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
        // S1-J-3b: the newly-closed header fields.
        assert_eq!(got.creation_time, want.creation_time, "{context}: creation");
        assert_eq!(got.last_played, want.last_played, "{context}: last played");
        assert_eq!(got.ore_tiers, want.ore_tiers, "{context}: ore tiers");
        // S1-J-3b: the saved-NPC flags and the quest index.
        assert_eq!(got.saved_wizard, want.saved_wizard, "{context}: savedWizard");
        assert_eq!(got.saved_mech, want.saved_mech, "{context}: savedMech");
        assert_eq!(got.saved_angler, want.saved_angler, "{context}: savedAngler");
        assert_eq!(got.angler_quest, want.angler_quest, "{context}: anglerQuest");
        assert_eq!(
            got.saved_stylist, want.saved_stylist,
            "{context}: savedStylist"
        );
        assert_eq!(
            got.saved_tax_collector, want.saved_tax_collector,
            "{context}: savedTaxCollector"
        );
        assert_eq!(got.saved_golfer, want.saved_golfer, "{context}: savedGolfer");
        // S1-J-3b: the event bosses, the towers and the apocalypse phase.
        assert_eq!(
            got.downed_ancient_cultist, want.downed_ancient_cultist,
            "{context}: downedAncientCultist"
        );
        assert_eq!(
            got.downed_moon_lord, want.downed_moon_lord,
            "{context}: downedMoonlord"
        );
        assert_eq!(
            got.downed_queen_slime, want.downed_queen_slime,
            "{context}: downedQueenSlime"
        );
        assert_eq!(
            got.downed_tower_solar, want.downed_tower_solar,
            "{context}: downedTowerSolar"
        );
        assert_eq!(
            got.downed_tower_vortex, want.downed_tower_vortex,
            "{context}: downedTowerVortex"
        );
        assert_eq!(
            got.downed_tower_nebula, want.downed_tower_nebula,
            "{context}: downedTowerNebula"
        );
        assert_eq!(
            got.downed_tower_stardust, want.downed_tower_stardust,
            "{context}: downedTowerStardust"
        );
        assert_eq!(
            got.tower_active_solar, want.tower_active_solar,
            "{context}: TowerActiveSolar"
        );
        assert_eq!(
            got.tower_active_vortex, want.tower_active_vortex,
            "{context}: TowerActiveVortex"
        );
        assert_eq!(
            got.tower_active_nebula, want.tower_active_nebula,
            "{context}: TowerActiveNebula"
        );
        assert_eq!(
            got.tower_active_stardust, want.tower_active_stardust,
            "{context}: TowerActiveStardust"
        );
        assert_eq!(
            got.lunar_apocalypse_is_up, want.lunar_apocalypse_is_up,
            "{context}: LunarApocalypseIsUp"
        );
        // S1-J-3b: the two owned town-NPC spawn unlocks.
        assert_eq!(
            got.unlocked_party_girl, want.unlocked_party_girl,
            "{context}: unlockedPartyGirlSpawn"
        );
        assert_eq!(
            got.unlocked_princess, want.unlocked_princess,
            "{context}: unlockedPrincessSpawn"
        );
        // Gate V3 (S1-P-i35): the combat book + red slime unlock bytes.
        assert_eq!(
            got.combat_book_was_used, want.combat_book_was_used,
            "{context}: combatBookWasUsed"
        );
        assert_eq!(
            got.unlocked_slime_red_spawn, want.unlocked_slime_red_spawn,
            "{context}: unlockedSlimeRedSpawn"
        );
        // W4-g71: the three new setBG style bytes (slots 3/6/7).
        assert_eq!(got.snow_bg_style, want.snow_bg_style, "{context}: snowBG");
        assert_eq!(
            got.desert_bg_style, want.desert_bg_style,
            "{context}: desertBG"
        );
        assert_eq!(got.ocean_bg_style, want.ocean_bg_style, "{context}: oceanBG");
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
        b.extend_from_slice(&(world.creation_time as u64).to_le_bytes());
        b.extend_from_slice(&(world.last_played as u64).to_le_bytes());
        b.push(world.moon_type);
        b.extend(flags_tail_prefix(world));
        // The tail opens with the two owned event flags; the fixture table
        // carries their C# defaults, so the world's values go in first.
        let mut tail = flags_tail(VER);
        tail[0] = u8::from(world.blood_moon);
        tail[1] = u8::from(world.eclipse);
        // The hardmode trio at tail offset 76 (2+8+1+11+7+8+1+20+8+1+9 bytes
        // of flags before it) — `SaveWorldFlags:1315-1317`.
        let mut at = 76;
        for v in [
            world.ore_tiers.cobalt,
            world.ore_tiers.mythril,
            world.ore_tiers.adamantite,
        ] {
            tail[at..at + 4].copy_from_slice(&v.to_le_bytes());
            at += 4;
        }
        // The pre-hardmode quartet at tail offset 2005 (8 setBG + 10 cloud +
        // 4 angler count + 1+4+1+1+1 saved NPCs + 4+4 invasion/cultist +
        // 2+1172+2+586 banner arrays + 1 dawn + 9+9 event/tower + 10 party +
        // 13 sandstorm + 4 DD2 + 1+1+3 BG + 1 book + 7 lantern + 4+52 tree
        // tops + 2 seasons past the trio's end at 88) —
        // `SaveWorldFlags:1388-1391`.
        let mut at = 2005;
        for v in [
            world.ore_tiers.copper,
            world.ore_tiers.iron,
            world.ore_tiers.silver,
            world.ore_tiers.gold,
        ] {
            tail[at..at + 4].copy_from_slice(&v.to_le_bytes());
            at += 4;
        }
        // S1-J-3b: the newly-owned single-byte flags, each at its C# tail
        // offset (the running sum of the field widths in `flags_tail`).
        // savedWizard/savedMech sit in the 7-byte savedGoblin..downedPirates
        // run that starts at 22 (2+8+1+11): goblin@22, wizard@23, mech@24.
        tail[23] = u8::from(world.saved_wizard); // savedWizard (`:1294`)
        tail[24] = u8::from(world.saved_mech); // savedMech (`:1295`)
        // The OLDER owned flags the writer also emits (DW-2.2's set) — the
        // table must carry the world's values for every owned field, not
        // just the S1-J-3b close set.
        // The 11-byte downed run at 11 (`SaveWorldFlags:1283-1293`), incl.
        // the derived downedMechBossAny (`:1290`).
        for (i, v) in [
            world.downed_eoc,
            world.downed_eow,
            world.downed_skeletron,
            world.downed_queen_bee,
            world.downed_mech_boss1,
            world.downed_mech_boss2,
            world.downed_mech_boss3,
            world.downed_mech_boss1 || world.downed_mech_boss2 || world.downed_mech_boss3,
            world.downed_plant_boss,
            world.downed_golem,
            world.downed_slime_king,
        ]
        .into_iter()
        .enumerate()
        {
            tail[11 + i] = u8::from(v);
        }
        tail[22] = u8::from(world.saved_goblin); // savedGoblin (`:1293`)
        tail[25] = u8::from(world.downed_goblins); // downedGoblins (`:1296`)
        tail[27] = u8::from(world.downed_frost); // downedFrost (`:1297`)
        tail[28] = u8::from(world.downed_pirates); // downedPirates (`:1298`)
        // The orbs/meteor/counts/hardmode block at 29..36 (`:1299-1305`).
        tail[29] = u8::from(world.shadow_orb_smashed);
        tail[30] = u8::from(world.spawn_meteor);
        tail[31] = world.shadow_orb_count as u8; // C# truncates to a byte
        tail[32..36].copy_from_slice(&world.altars_smashed.to_le_bytes());
        tail[36] = u8::from(world.hard_mode);
        // The invasion block at 38..57 + slime rain + sundial (`:1306-1312`).
        tail[38..42].copy_from_slice(&world.invasion_delay.to_le_bytes());
        tail[42..46].copy_from_slice(&world.invasion_size.to_le_bytes());
        tail[46..50].copy_from_slice(&world.invasion_type.to_le_bytes());
        tail[50..58].copy_from_slice(&world.invasion_x.to_le_bytes());
        tail[58..66].copy_from_slice(&world.slime_rain_time.to_le_bytes());
        tail[66] = world.sundial_cooldown;
        // The rain snapshot at 67..75 (`SetTempToOngoing:1057-1059`).
        tail[67] = u8::from(world.raining);
        tail[68..72].copy_from_slice(&world.rain_time.to_le_bytes());
        tail[72..76].copy_from_slice(&world.max_raining.to_le_bytes());
        // The owned background styles inside setBG 0..7 at 88..95
        // (`:1318-1325`): corrupt@89, jungle@90, snow@91, hallow@92,
        // crimson@93, desert@94, ocean@95.
        tail[89] = world.corrupt_bg_style;
        tail[90] = world.jungle_bg_style;
        tail[91] = world.snow_bg_style;
        tail[92] = world.hallow_bg_style;
        tail[93] = world.crimson_bg_style;
        tail[94] = world.desert_bg_style;
        tail[95] = world.ocean_bg_style;
        // savedAngler..savedGolfer follow the 4-byte angler-list count at
        // 106..110 (88 trio-end + 8 setBG + 10 clouds/wind = 106).
        tail[110] = u8::from(world.saved_angler); // savedAngler (`:1334`)
        tail[111..115].copy_from_slice(&(world.angler_quest as i32).to_le_bytes()); // anglerQuest (`:1335`)
        tail[115] = u8::from(world.saved_stylist); // savedStylist (`:1336`)
        tail[116] = u8::from(world.saved_tax_collector); // savedTaxCollector (`:1337`)
        tail[117] = u8::from(world.saved_golfer); // savedGolfer (`:1338`)
        tail[118..122].copy_from_slice(&world.invasion_size_start.to_le_bytes()); // (`:1339`)
        // The event bosses past the banner arrays (which end at 1888):
        // dawn@1888, fishron@1889, martians@1890, cultist@1891, moonlord@1892.
        tail[1891] = u8::from(world.downed_ancient_cultist); // downedAncientCultist (`:1345`)
        tail[1892] = u8::from(world.downed_moon_lord); // downedMoonlord (`:1346`)
        tail[1888] = u8::from(world.fast_forward_to_dawn); // fastForwardTimeToDawn (`:1342`)
        // S1-D-3 W-9: downedFishron is World-owned now (`:1343`).
        tail[1889] = u8::from(world.downed_fishron); // downedFishron
        tail[1890] = u8::from(world.downed_martians); // downedMartians (`:1344`)
        // The four tower downeds + four actives + the apocalypse phase, after
        // the five seasonal bosses (1893..1897): 1898..1906.
        tail[1898] = u8::from(world.downed_tower_solar); // (`:1352`)
        tail[1899] = u8::from(world.downed_tower_vortex); // (`:1353`)
        tail[1900] = u8::from(world.downed_tower_nebula); // (`:1354`)
        tail[1901] = u8::from(world.downed_tower_stardust); // (`:1355`)
        tail[1902] = u8::from(world.tower_active_solar); // (`:1356`)
        tail[1903] = u8::from(world.tower_active_vortex); // (`:1357`)
        tail[1904] = u8::from(world.tower_active_nebula); // (`:1358`)
        tail[1905] = u8::from(world.tower_active_stardust); // (`:1359`)
        tail[1906] = u8::from(world.lunar_apocalypse_is_up); // (`:1360`)
        // downedQueenSlime: 2021 quartet-end + 3 pets = 2024 empress, +1.
        tail[2025] = u8::from(world.downed_queen_slime); // (`:1396`)
        // The unlocks: 2026 deerclops + 1 slimeBlue = 2028 merchant, +1 demo,
        // +1 partyGirl; +4 dyeTrader/truffle/armsDealer/nurse, +1 princess.
        tail[2030] = u8::from(world.unlocked_party_girl); // (`:1401`)
        tail[2035] = u8::from(world.unlocked_princess); // (`:1406`)
        tail[2026] = u8::from(world.downed_deerclops); // downedDeerclops (`:1397`)
        // The rainbow slime in the 7-slime run at 2038 (`:1410-1414`):
        // green/old/purple, then rainbow@2041, red@2042 (gate V3, i35).
        tail[2041] = u8::from(world.unlocked_slime_rainbow_spawn);
        tail[2042] = u8::from(world.unlocked_slime_red_spawn);
        // combatBookWasUsed (`WorldFile.cs:1379`): trio-end 88 + 8 setBG +
        // 10 clouds/wind + 4 angler count + 8 saved-NPC/quest + 8 invasion/
        // cultist + 2+1172+2+586 banners + 1 dawn + 9+9 event/tower + 10
        // party + 13 sandstorm + 4 DD2 + 5 BG (mushroom/underworld/tree2-4)
        // = tail offset 1939 (gate V3, i35).
        tail[1939] = u8::from(world.combat_book_was_used);
        tail[2045] = u8::from(world.fast_forward_to_dusk); // fastForwardTimeToDusk (`:1416`)
        tail[2046] = world.moondial_cooldown; // moondialCooldown (`:1417`)
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
        // fields land. Six pointers, the shape that build's `SaveWorld_Version2`
        // actually wrote: sections 0..5 (header, tiles, chests, signs, NPCs —
        // the NPC list just its terminator, `SaveNPCs`'s shimmer block and the
        // tower list are `>= 268`/`>= 140`) and the footer right behind, so
        // `load_sections` walks the three gated sections it may and stops.
        let data = synthetic_file(100, &[2, 144], 160, 6);
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
    fn s1_j_2_world_header_identity_roundtrips_in_wld() {
        // `IO/WorldFile.cs`: `SaveWorldHeader` writes generator version/UniqueId/
        // WorldId (`:1225-1227`), `SaveWorldFlags` writes LastPlayed
        // (`DateTime.UtcNow.ToBinary`, `:1251`), footer writes WorldId (`:1756`),
        // and `LoadWorld_Version2`/`LoadWorldFlags` read them back.
        let mut world = World::new("HeaderId".into(), 8, 6);
        world.generator_version = 1400159338497u64; // `CreateMetadata:603`
        world.unique_id = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        world.world_id = 0x1234_5678;
        world.last_played = 638000000000000000i64; // arbitrary DateTime.ToBinary-like ticks
        let buf = to_bytes(&world, &SectionData::default(), 1).expect("writes");

        // Header bytes: generator version, GUID, world id at fixed offsets for
        // this default-layout world (name "HeaderId" = 8 chars, seed text "0").
        // Prologue 70 + bitmap tail 97 = 167; header starts at 167:
        // [len+name(9)] [len+seed(2)] [gen(8)] [guid(16)] [world id(4)]
        let gen_at = 167 + 9 + 2;
        assert_eq!(
            &buf[gen_at..gen_at + 8],
            &1400159338497u64.to_le_bytes(),
            "generator version"
        );
        assert_eq!(
            &buf[gen_at + 8..gen_at + 24],
            &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
            "unique id"
        );
        assert_eq!(
            &buf[gen_at + 24..gen_at + 28],
            &0x1234_5678i32.to_le_bytes(),
            "world id"
        );

        let loaded = parse(&buf).expect("parses");
        assert_eq!(loaded.generator_version, world.generator_version);
        assert_eq!(loaded.unique_id, world.unique_id);
        assert_eq!(loaded.world_id, world.world_id);
        assert_eq!(loaded.last_played, world.last_played);
    }

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

    // --- S1-J-1: the four wire bits in the `.wld` tile section ---------------

    use crate::wiring::WireColor;

    /// The tile section of [`wire_fixture_world`], byte for byte, as produced
    /// by a `dotnet` run of the decompiled `WorldFile.SaveWorldTiles`
    /// (`WorldFile.cs:1432-1604`) driving the decompiled `Tile` accessors
    /// (`Tile.cs:437-452/528-733`). The oracle's source is the decompile
    /// itself — only the three global lookups (`Main.tile`,
    /// `Main.tileFrameImportant`, `TileID.Sets.AllowsSaveCompressionBatching`)
    /// became parameters.
    const CSHARP_WIRE_TILE_SECTION: [u8; 75] = [
        0x00, 0x43, 0x02, 0x02, 0x5a, 0x00, 0x12, 0x00, 0x01, 0x03, 0x04, 0x02, 0x5a, 0x00, 0x12,
        0x00, 0x03, 0x08, 0x02, 0x5a, 0x00, 0x12, 0x00, 0x03, 0x01, 0x20, 0x02, 0x5a, 0x00, 0x12,
        0x00, 0x03, 0x0f, 0x20, 0x02, 0x5a, 0x00, 0x12, 0x00, 0x03, 0x22, 0x02, 0x5a, 0x00, 0x12,
        0x00, 0x07, 0x01, 0x19, 0x02, 0x02, 0x5a, 0x00, 0x12, 0x00, 0x03, 0x05, 0x09, 0x1a, 0x02,
        0x5a, 0x00, 0x12, 0x00, 0x80, 0x13, 0x01, 0x20, 0x02, 0x5a, 0x00, 0x12, 0x00, 0x64, 0x00,
    ];

    /// The oracle's fixture: one column of twelve rows that covers the four
    /// wire colours (`Tile.wire()`/`wire2()`/`wire3()`/`wire4()`), the RLE
    /// batch boundary, the slope, the tile and wall paint, the coatings and
    /// the liquid kinds. Rows 1 and 2 are identical (the red wire) — the C#'s
    /// `isTheSameAs` batches them into one record.
    fn wire_fixture_world() -> World {
        let mut world = World::new("WireFixture".into(), 1, 12);
        world.frame_important = vec![false; crate::tiledata::TILE_COUNT];
        world.frame_important[2] = true;
        let solid = || {
            let mut t = Tile {
                kind: 2,
                frame_x: 90,
                frame_y: 18,
                ..Default::default()
            };
            t.flags |= FLAG_ACTIVE;
            t
        };
        for y in [1, 2, 3, 4, 5, 6, 7, 8, 9, 10] {
            let idx = world.index(0, y);
            world.tiles[idx] = solid();
        }
        // The four colours: red (`sTileHeader` 0x80), blue (0x100), green
        // (0x200) and yellow (`bTileHeader` 0x80).
        world.wires.set(0, 1, WireColor::Red, true);
        world.wires.set(0, 2, WireColor::Red, true);
        world.wires.set(0, 3, WireColor::Blue, true);
        world.wires.set(0, 4, WireColor::Green, true);
        world.wires.set(0, 5, WireColor::Yellow, true);
        for color in [
            WireColor::Red,
            WireColor::Blue,
            WireColor::Green,
            WireColor::Yellow,
        ] {
            world.wires.set(0, 6, color, true);
        }
        // A slope plus the red wire.
        let idx = world.index(0, 7);
        world.tiles[idx].set_slope(1);
        world.wires.set(0, 7, WireColor::Red, true);
        // The wall, the wall paint, the tile paint and the invisible block.
        let idx = world.index(0, 8);
        world.tiles[idx].set_paint(3);
        world.tiles[idx].wall = 5;
        world.set_wall_paint(0, 8, 9);
        world.set_invisible_block(0, 8, true);
        // Honey, then lava with the yellow wire.
        let idx = world.index(0, 9);
        world.tiles[idx].liquid = 128;
        world.tiles[idx].flags |= FLAG_LIQUID_HONEY;
        let idx = world.index(0, 10);
        world.tiles[idx].liquid = 100;
        world.tiles[idx].flags |= FLAG_LIQUID_LAVA;
        world.wires.set(0, 10, WireColor::Yellow, true);
        world
    }

    #[test]
    fn s1_j_1_the_tile_section_matches_the_csharp_bytes() {
        // The C#'s own encoder (`SaveWorldTiles:1432-1604`) writes the four
        // wire bits: `b2` 0x02/0x04/0x08 for `wire()`/`wire2()`/`wire3()` and
        // `b3` 0x20 for `wire4()`. The port dropped them, so its tile section
        // was never byte-identical.
        let world = wire_fixture_world();
        let mut w = Writer::new();
        sections::tiles::write(&mut w, &world);
        assert_eq!(w.into_inner(), CSHARP_WIRE_TILE_SECTION.to_vec());
    }

    #[test]
    fn s1_j_1_the_rle_run_breaks_when_the_wire_state_differs() {
        // `Tile.isTheSameAs:161` compares the whole `sTileHeader`, and the
        // wire bits live in it — so two tiles that differ only by a wire are
        // **not** batched (`WorldFile.cs:1577`).
        let mut world = World::new("a".into(), 1, 3);
        world.frame_important = vec![false; crate::tiledata::TILE_COUNT];
        world.frame_important[2] = true;
        for y in 0..3 {
            let idx = world.index(0, y);
            world.tiles[idx] = tile(2, 90, 18);
        }
        world.wires.set(0, 1, WireColor::Red, true);
        let mut w = Writer::new();
        sections::tiles::write(&mut w, &world);
        // Three standalone records: the middle one carries the chained header
        // (b = 0x03 active + "b2 follows", b2 = 0x02 the red wire).
        let want = vec![
            0x02, 0x02, 0x5a, 0x00, 0x12, 0x00, 0x03, 0x02, 0x02, 0x5a, 0x00, 0x12, 0x00, 0x02,
            0x02, 0x5a, 0x00, 0x12, 0x00,
        ];
        assert_eq!(w.into_inner(), want);
    }

    #[test]
    fn s1_j_1_the_wire_state_round_trips_through_a_wld() {
        // `LoadWorldTiles:2614-2632` restores the three `sTileHeader` wires
        // and `:2651-2654` the `bTileHeader` one; `CopyFrom` (`:2711`) copies
        // them into every row of the RLE run.
        let world = wire_fixture_world();
        let buf = to_bytes(&world, &SectionData::default(), 1).expect("writes");
        let back = parse(&buf).expect("the saved world parses");
        for (y, red, blue, green, yellow) in [
            (0, false, false, false, false),
            (1, true, false, false, false),
            (2, true, false, false, false),
            (3, false, true, false, false),
            (4, false, false, true, false),
            (5, false, false, false, true),
            (6, true, true, true, true),
            (7, true, false, false, false),
            (8, false, false, false, false),
            (9, false, false, false, false),
            (10, false, false, false, true),
            (11, false, false, false, false),
        ] {
            assert_eq!(
                back.wires.get(0, y, WireColor::Red),
                red,
                "the red wire at row {y}"
            );
            assert_eq!(
                back.wires.get(0, y, WireColor::Blue),
                blue,
                "the blue wire at row {y}"
            );
            assert_eq!(
                back.wires.get(0, y, WireColor::Green),
                green,
                "the green wire at row {y}"
            );
            assert_eq!(
                back.wires.get(0, y, WireColor::Yellow),
                yellow,
                "the yellow wire at row {y}"
            );
        }
    }

    // --- round-trip lane: save -> load keeps everything the port models ------
    //
    // `SaveWorld_Version2:1150-1168` assembles the sections in C# order and
    // `LoadWorld_Version2:1760-1851` walks them back; these four tests are the
    // whole-file contract rather than one section's codec: the tile grid, the
    // header/flags metadata, the eight payload sections, and the weather-event
    // snapshot (the newest slice).

    /// One important solid tile: kind 2 (`Main.tileFrameImportant[2]`) so the
    /// frames ride the wire (`SaveWorldTiles:1449-1459`).
    fn solid(frame_x: i16, frame_y: i16) -> Tile {
        let mut t = Tile {
            kind: 2,
            frame_x,
            frame_y,
            ..Default::default()
        };
        t.flags |= FLAG_ACTIVE;
        t
    }

    /// A 6x5 grid whose cells cover every state `SaveWorldTiles:1432-1604`
    /// writes: the frames, the paint, the wall (8- and 16-bit), the four
    /// liquids, the slope/half-brick pair, the actuator/inactive pair, the two
    /// invisible coats, the block/wall fullbright pair and the four wires.
    fn tile_zoo() -> World {
        let mut world = World::new("TileZoo".into(), 6, 5);
        world.frame_important = vec![false; crate::tiledata::TILE_COUNT];
        world.frame_important[2] = true;
        for y in 0..world.height {
            for x in 0..world.width {
                let idx = world.index(x, y);
                world.tiles[idx] = solid(18 * (x as i16), 18 * (y as i16));
            }
        }
        // Tile paint + wall paint (`b3` 0x08 / 0x10). The wall paint rides the
        // `if (tile.wall != 0)` arm (`SaveWorldTiles:1478-1486`), so the cell
        // needs a wall for it to reach the file.
        let i = world.index(0, 0);
        world.tiles[i].set_paint(7);
        world.tiles[i].wall = 1;
        world.set_wall_paint(0, 0, 9);
        // An 8-bit and a 16-bit wall (`b3` 0x40 carries the high byte).
        let i = world.index(0, 1);
        world.tiles[i].wall = 5;
        let i = world.index(0, 2);
        world.tiles[i].wall = 308;
        // The four liquid kinds and their amount byte.
        let i = world.index(1, 0);
        world.tiles[i].liquid = 128;
        let i = world.index(1, 1);
        world.tiles[i].liquid = 200;
        world.tiles[i].flags |= FLAG_LIQUID_LAVA;
        let i = world.index(1, 2);
        world.tiles[i].liquid = 255;
        world.tiles[i].flags |= FLAG_LIQUID_HONEY;
        let i = world.index(1, 3);
        world.tiles[i].liquid = 64;
        world.tiles[i].flags |= FLAG_LIQUID_SHIMMER;
        // Slopes 1..=4 and the half-brick alternative (`b2` 0x10 / 0x20..).
        let i = world.index(2, 0);
        world.tiles[i].set_slope(1);
        let i = world.index(2, 1);
        world.tiles[i].set_slope(3);
        let i = world.index(2, 2);
        world.tiles[i].flags |= FLAG_HALF_BRICK;
        // Actuator + inactive (`b3` 0x02 / 0x04).
        let i = world.index(3, 0);
        world.tiles[i].flags |= FLAG_ACTUATOR;
        let i = world.index(3, 1);
        world.tiles[i].flags |= FLAG_ACTUATOR | FLAG_INACTIVE;
        // The two invisible coats (`b4` 0x02 / 0x04) and the two fullbrights
        // (`b4` 0x08 / 0x10).
        world.set_invisible_block(4, 0, true);
        world.set_invisible_wall(4, 1, true);
        let i = world.index(4, 2);
        world.tiles[i].flags |= FLAG_FULLBRIGHT;
        world.set_fullbright_wall(4, 3, true);
        // The four wire colours (`b2` 0x02/0x04/0x08, `b3` 0x20).
        for (n, color) in [
            WireColor::Red,
            WireColor::Blue,
            WireColor::Green,
            WireColor::Yellow,
        ]
        .into_iter()
        .enumerate()
        {
            world.wires.set(5, n as i32, color, true);
        }
        // Everything at once, to pin the header chain's byte order.
        let i = world.index(5, 4);
        world.tiles[i].wall = 300;
        world.tiles[i].liquid = 100;
        world.tiles[i].flags |= FLAG_LIQUID_LAVA | FLAG_ACTUATOR | FLAG_HALF_BRICK;
        world.tiles[i].set_paint(31);
        world.set_wall_paint(5, 4, 31);
        world.set_invisible_block(5, 4, true);
        world.set_fullbright_wall(5, 4, true);
        world.wires.set(5, 4, WireColor::Red, true);
        world.wires.set(5, 4, WireColor::Green, true);
        world
    }

    #[test]
    fn wld_roundtrip_tiles_identical() {
        // `SaveWorldTiles:1432-1604` writes (chained headers, payload, RLE) and
        // `LoadWorldTiles:2519-2721` reads it back; the grid must come out
        // identical, tile for tile, wire for wire.
        let world = tile_zoo();
        let buf = to_bytes(&world, &SectionData::default(), 1).expect("writes");
        let back = parse(&buf).expect("the saved world parses");
        assert_eq!((back.width, back.height), (world.width, world.height));
        assert_eq!(back.frame_important, world.frame_important);
        for y in 0..world.height {
            for x in 0..world.width {
                let idx = world.index(x, y);
                assert_eq!(back.tiles[idx], world.tiles[idx], "tile ({x}, {y})");
                assert_eq!(
                    back.wall_paint(x, y),
                    world.wall_paint(x, y),
                    "wall paint ({x}, {y})"
                );
                assert_eq!(
                    back.invisible_block(x, y),
                    world.invisible_block(x, y),
                    "invisible block ({x}, {y})"
                );
                assert_eq!(
                    back.invisible_wall(x, y),
                    world.invisible_wall(x, y),
                    "invisible wall ({x}, {y})"
                );
                assert_eq!(
                    back.fullbright_wall(x, y),
                    world.fullbright_wall(x, y),
                    "fullbright wall ({x}, {y})"
                );
                for color in [
                    WireColor::Red,
                    WireColor::Blue,
                    WireColor::Green,
                    WireColor::Yellow,
                ] {
                    assert_eq!(
                        back.wires.get(x, y, color),
                        world.wires.get(x, y, color),
                        "{color:?} wire ({x}, {y})"
                    );
                }
            }
        }
    }

    #[test]
    fn wld_roundtrip_metadata() {
        // `SaveWorldHeader:1221` + `SaveWorldFlags:1238`: every header field the
        // port owns, and the footer's name (`SaveFooter:1752`). `spawn_y` is
        // the file's own value since S1-J-3b (`LoadWorldFlags:2069`).
        let mut world = World::new("Metadata — ünïcode".into(), 8, 6);
        world.seed = 424242;
        world.generator_version = 140_015_933_8497;
        world.unique_id = [0x11; 16];
        world.world_id = 4242;
        world.spawn_x = 3;
        world.time = 1234.0;
        world.day_time = false;
        world.moon_type = 3;
        world.moon_phase = 5;
        world.world_surface = 4.5;
        world.rock_layer = 100.25;
        world.slime_rain_time = 999.0;
        world.sundial_cooldown = 2;
        world.moondial_cooldown = 3;
        world.invasion_delay = 7;
        world.invasion_size = 8;
        world.invasion_type = 9;
        world.invasion_x = 10.5;
        world.corrupt_bg_style = 2;
        world.jungle_bg_style = 3;
        world.hallow_bg_style = 4;
        world.crimson_bg_style = 5;
        world.mushroom_bg_style = 6;
        world.snow_bg_style = 21;
        world.desert_bg_style = 52;
        world.ocean_bg_style = 5;
        world.hard_mode = true;
        world.downed_eoc = true;
        world.downed_plant_boss = true;
        world.spawn_meteor = true;
        world.altars_smashed = 3;
        world.shadow_orb_smashed = true;
        let buf = to_bytes(&world, &SectionData::default(), 1).expect("writes");
        let back = parse(&buf).expect("the saved world parses");
        assert_eq!(back.name, world.name);
        assert_eq!(back.seed, world.seed);
        assert_eq!(back.generator_version, world.generator_version);
        assert_eq!(back.unique_id, world.unique_id);
        assert_eq!(back.world_id, world.world_id);
        // `SaveWorldFlags:1270-1271`: the spawn column and row (`spawnTileX/Y`).
        assert_eq!(back.spawn_x, world.spawn_x);
        assert_eq!(back.spawn_y, world.spawn_y);
        assert_eq!(back.time, world.time);
        assert_eq!(back.day_time, world.day_time);
        assert_eq!(back.moon_type, world.moon_type);
        assert_eq!(back.moon_phase, world.moon_phase);
        assert_eq!(back.world_surface, world.world_surface);
        assert_eq!(back.rock_layer, world.rock_layer);
        assert_eq!(back.slime_rain_time, world.slime_rain_time);
        assert_eq!(back.sundial_cooldown, world.sundial_cooldown);
        assert_eq!(back.moondial_cooldown, world.moondial_cooldown);
        assert_eq!(back.invasion_delay, world.invasion_delay);
        assert_eq!(back.invasion_size, world.invasion_size);
        assert_eq!(back.invasion_type, world.invasion_type);
        assert_eq!(back.invasion_x, world.invasion_x);
        assert_eq!(back.corrupt_bg_style, world.corrupt_bg_style);
        assert_eq!(back.jungle_bg_style, world.jungle_bg_style);
        assert_eq!(back.hallow_bg_style, world.hallow_bg_style);
        assert_eq!(back.crimson_bg_style, world.crimson_bg_style);
        assert_eq!(back.mushroom_bg_style, world.mushroom_bg_style);
        assert_eq!(back.snow_bg_style, world.snow_bg_style);
        assert_eq!(back.desert_bg_style, world.desert_bg_style);
        assert_eq!(back.ocean_bg_style, world.ocean_bg_style);
        assert!(back.hard_mode);
        assert!(back.downed_eoc && back.downed_plant_boss);
        assert!(back.spawn_meteor);
        assert_eq!(back.altars_smashed, world.altars_smashed);
        assert_eq!(back.shadow_orb_smashed, world.shadow_orb_smashed);
    }

    /// Every one of the eight sections past the tiles, filled with records the
    /// C# wire shape can carry back (`SaveChests`..`SaveCreativePowers`).
    fn section_zoo() -> SectionData {
        let mut data = SectionData::default();
        data.chests.push(sections::WldChest {
            x: 3,
            y: 4,
            name: "Loot — ünïcode".into(),
            max_items: 2,
            slots: vec![
                sections::WldChestSlot {
                    stack: 5,
                    kind: 22,
                    prefix: 3,
                },
                sections::WldChestSlot {
                    stack: 0,
                    kind: 0,
                    prefix: 0,
                },
            ],
        });
        data.signs.push(sections::WldSign {
            text: "Read me".into(),
            x: 6,
            y: 7,
        });
        data.npcs.shimmered = vec![17, 22];
        data.npcs.town.push(sections::WldTownNpc {
            net_id: 17,
            legacy_name: None,
            given_name: "Hal".into(),
            x: 120.5,
            y: 64.25,
            homeless: false,
            home_x: 3,
            home_y: 4,
            town_npc: true,
            variation_index: 2,
            homeless_despawn: true,
        });
        data.tile_entities.push(sections::WldTileEntity {
            id: 0,
            ty: 0, // `TETrainingDummy`
            x: 5,
            y: 6,
            extra: sections::WldTeExtra::Dummy { npc: 7 },
        });
        data.tile_entities.push(sections::WldTileEntity {
            id: 1,
            ty: 1, // `TEItemFrame`
            x: 7,
            y: 8,
            extra: sections::WldTeExtra::Item {
                item: sections::WldTeItem {
                    kind: 42,
                    prefix: 4,
                    stack: 1,
                },
            },
        });
        data.pressure_plates.push(sections::WldPressurePlate { x: 9, y: 10 });
        data.town_rooms.push(sections::WldTownRoom {
            npc_id: 17,
            x: 3,
            y: 4,
        });
        data.bestiary.kills.push(("BlueSlime".into(), 7));
        data.bestiary.sights.push("GreenSlime".into());
        data.bestiary.chats.push("Pinky".into());
        data.creative_powers
            .records
            .push((0, sections::WldCreativeValue::Bool(true)));
        data.creative_powers
            .records
            .push((13, sections::WldCreativeValue::Bool(false)));
        data
    }

    #[test]
    fn wld_roundtrip_every_section_identical() {
        // The eight payload sections, both directions, in one file:
        // `SaveChests:1606` (3), `SaveSigns:1678` (4), `SaveNPCs:1703` (5),
        // `SaveTileEntities:3302` (6), `SaveWeightedPressurePlates:3359` (7),
        // `SaveTownManager:3385` (8), `SaveBestiary:3396` (9),
        // `SaveCreativePowers:3412` (10).
        let world = World::new("Sections".into(), 8, 6);
        let data = section_zoo();
        let buf = to_bytes(&world, &data, 1).expect("writes");
        let back = parse_with_sections(&buf).expect("the saved world parses");
        assert_eq!(back.sections.chests, data.chests, "chests");
        assert_eq!(back.sections.signs, data.signs, "signs");
        assert_eq!(back.sections.npcs, data.npcs, "npcs");
        assert_eq!(
            back.sections.tile_entities, data.tile_entities,
            "tile entities"
        );
        assert_eq!(
            back.sections.pressure_plates, data.pressure_plates,
            "pressure plates"
        );
        assert_eq!(back.sections.town_rooms, data.town_rooms, "town rooms");
        assert_eq!(back.sections.bestiary, data.bestiary, "bestiary");
        assert_eq!(
            back.sections.creative_powers, data.creative_powers,
            "creative powers"
        );
        // The reload is itself stable: a second save is byte-identical.
        let again = to_bytes(&back.world, &back.sections, 1).expect("re-writes");
        assert_eq!(again, buf, "save -> load -> save is byte-stable");
    }

    #[test]
    fn wld_roundtrip_weather_snapshot() {
        // `SaveWorldFlags:1312-1314` writes the rain snapshot
        // (`_tempRaining`/`_tempRainTime`/`_tempMaxRain`) and `:1369-1372` the
        // sandstorm one (`_tempSandstorm*`); `SetTempToOngoing:1053-1059`
        // snapshots the live state into them before the write and
        // `SetOngoingToTemps:159-172` restores it on load. Both events are
        // ported (`App.rain`, `scene::Sandstorm`), so both must round-trip.
        let mut world = World::new("Weather".into(), 8, 6);
        world.raining = true;
        world.rain_time = 4321;
        world.max_raining = 0.75;
        world.sandstorm_happening = true;
        world.sandstorm_time_left = 1234;
        world.sandstorm_severity = 0.5;
        world.sandstorm_intended_severity = 0.625;
        let buf = to_bytes(&world, &SectionData::default(), 1).expect("writes");
        let back = parse(&buf).expect("the saved world parses");
        assert!(back.raining, "the rain event survives the round-trip");
        assert_eq!(back.rain_time, world.rain_time);
        assert_eq!(back.max_raining, world.max_raining);
        assert!(
            back.sandstorm_happening,
            "the sandstorm event survives the round-trip"
        );
        assert_eq!(back.sandstorm_time_left, world.sandstorm_time_left);
        assert_eq!(back.sandstorm_severity, world.sandstorm_severity);
        assert_eq!(
            back.sandstorm_intended_severity,
            world.sandstorm_intended_severity
        );
    }

    // --- S1-J-3: the C#-written golden fixture -------------------------------
    //
    // `assets/test/golden.wld` is written by the REAL decompiled save path
    // (`WorldFile.SetTempToOngoing` + `SaveWorld_Version2`, the eleven sections)
    // through `tools/WldFixtureHarness` — a `dotnet` harness that extracts the
    // verbatim method bodies from `decompiled/` and shims only the global
    // statics (the S1-J-1 / S1-A-1 oracle pattern). The tile array is a small
    // hand-built deterministic scene (no worldgen RNG); the only substitution in
    // the extracted bodies is `DateTime.UtcNow` -> a pinned instant, because a
    // reproducible fixture cannot carry the wall clock. See the harness README
    // and ANALYSIS "The C#-written golden fixture (S1-J-3)".
    //
    // These tests pin the port against that real C# output: every section the
    // port models re-writes byte-for-byte; the header/flags block's unowned
    // fields are the documented divergences (not faked).

    /// The C#-written `.wld` fixture (`tools/WldFixtureHarness`, regen with
    /// `dotnet run --project tools/WldFixtureHarness -- assets/test`).
    const GOLDEN_WLD: &[u8] = include_bytes!("../../../../assets/test/golden.wld");

    /// Load the fixture the way the app does (`app/mod.rs:3511-3522`): the
    /// parser plus the three world-state fix-ups, then materialize the save bag
    /// from the loaded state (`saveglue.rs::wld_sections`'s `from_world` halves).
    fn golden_loaded() -> LoadedWorld {
        let mut loaded = parse_with_sections(GOLDEN_WLD).expect("the C# fixture parses");
        sections::signs::apply_loaded(&mut loaded.world, &loaded.sections.signs);
        sections::tile_entities::apply_loaded(&mut loaded.world, &loaded.sections.tile_entities);
        sections::pressure_plates::apply_loaded(&mut loaded.world, &loaded.sections.pressure_plates);
        loaded
    }

    fn golden_bag(loaded: &LoadedWorld) -> SectionData {
        let mut data = loaded.sections.clone();
        data.signs = sections::signs::from_world(&loaded.world);
        data.tile_entities = sections::tile_entities::from_world(&loaded.world);
        data.pressure_plates = sections::pressure_plates::from_world(&loaded.world);
        data
    }

    /// The eleven section bodies of a file, split at its pointer table (the
    /// prologue is `[0..pointers[0]]`, section i is `[pointers[i-1]..pointers[i]]`).
    fn section_bodies(bytes: &[u8]) -> Vec<&[u8]> {
        let table = SectionTable::open(bytes).expect("table");
        let mut out = Vec::with_capacity(11);
        let mut start = 0usize;
        for &end in &table.pointers {
            out.push(&bytes[start..end]);
            start = end;
        }
        out
    }

    #[test]
    fn s1_j_3_the_csharp_fixture_decodes_every_section() {
        let loaded = golden_loaded();
        let w = &loaded.world;
        // The world head (SaveWorldHeader) — the identity fields S1-J-2 owns.
        assert_eq!(w.name, "S1J3Golden");
        assert_eq!(w.seed, 12345);
        assert_eq!(w.generator_version, 1400159338497);
        assert_eq!(w.world_id, 123456789);
        assert_eq!(
            w.unique_id,
            [0x67, 0x45, 0x23, 0x01, 0xab, 0x89, 0xef, 0xcd, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef],
            "Guid.ToByteArray's mixed-endian layout"
        );
        assert_eq!((w.width, w.height), (40, 30));
        assert_eq!(loaded.version, 326);
        // S1-J-3b: the closed header fields — the pinned creation clock, the
        // file's spawn row (`LoadWorldFlags:2069`, no derivation) and the
        // seven ore tiers (`:2131-2133`/`:2344-2347`).
        assert_eq!(w.creation_time, 5250991586427387904);
        assert_eq!(w.last_played, 5250991586427387904);
        assert_eq!(w.spawn_y, 18);
        assert_eq!(
            w.ore_tiers,
            crate::ores::SavedOreTiers {
                copper: 7,
                iron: 6,
                silver: 9,
                gold: 8,
                cobalt: 107,
                mythril: 108,
                adamantite: 111,
            }
        );
        // The flags block's owned scalars (SaveWorldFlags).
        assert_eq!(w.spawn_x, 20);
        assert_eq!(w.world_surface, 12.0);
        assert_eq!(w.rock_layer, 20.0);
        assert_eq!(w.time, 12345.0);
        assert!(w.day_time);
        assert_eq!(w.moon_phase, 3);
        assert_eq!(w.moon_type, 2);
        assert_eq!(w.hell_back_style, 1);
        assert!(w.blood_moon && w.hard_mode);
        assert!(w.downed_eoc && w.downed_slime_king && w.downed_goblins && w.saved_goblin);
        assert!(w.downed_deerclops && w.unlocked_slime_rainbow_spawn);
        assert!(w.shadow_orb_smashed);
        assert_eq!(w.shadow_orb_count, 2);
        assert_eq!(w.altars_smashed, 1);
        assert!(w.raining);
        assert_eq!(w.rain_time, 500);
        assert_eq!(w.max_raining, 0.5);
        assert_eq!(w.sundial_cooldown, 3);
        assert_eq!(w.moondial_cooldown, 2);
        assert!(w.fast_forward_to_dawn);
        assert_eq!(w.corrupt_bg_style, 1);
        assert_eq!(w.jungle_bg_style, 2);
        assert_eq!(w.hallow_bg_style, 3);
        assert_eq!(w.crimson_bg_style, 4);
        assert_eq!(w.mushroom_bg_style, 5);
        // The eight payload sections (LoadChests..LoadCreativePowers).
        let s = &loaded.sections;
        assert_eq!(s.chests.len(), 2);
        assert_eq!(s.chests[0].name, "Golden Chest");
        assert_eq!(s.chests[0].slots[0], sections::WldChestSlot { stack: 50, kind: 2, prefix: 0 });
        assert_eq!(s.chests[0].slots[1], sections::WldChestSlot { stack: 99, kind: 8, prefix: 81 });
        // The banned item (type 58, ItemsThatShouldNotBeInInventory) wrote as air.
        assert_eq!(s.chests[0].slots[2], sections::WldChestSlot::default());
        assert_eq!(s.signs.len(), 2);
        assert_eq!(s.signs[0].text, "Hello golden world");
        assert_eq!(s.npcs.town.len(), 2);
        assert_eq!(s.npcs.town[0].net_id, 22); // Guide
        assert_eq!(s.npcs.town[0].given_name, "Guido");
        assert_eq!(s.npcs.town[0].variation_index, 2);
        assert_eq!(s.npcs.towers.len(), 1);
        assert_eq!(s.npcs.towers[0].net_id, 422); // Vortex tower
        assert_eq!(s.npcs.shimmered, vec![22]);
        assert_eq!(s.tile_entities.len(), 11); // one per TE kind
        assert_eq!(s.pressure_plates.len(), 2);
        assert_eq!(s.town_rooms.len(), 1);
        assert_eq!(s.town_rooms[0], sections::WldTownRoom { npc_id: 22, x: 6, y: 20 });
        assert_eq!(s.bestiary.kills, vec![("BlueSlime".into(), 3), ("Guide".into(), 1)]);
        assert_eq!(s.bestiary.sights, vec!["GreenSlime".to_string(), "Merchant".to_string()]);
        assert_eq!(s.bestiary.chats, vec!["Guide".to_string()]);
        assert_eq!(s.creative_powers.records.len(), 6);
        // The tile grid: the four wire colours, the liquids, the coatings.
        assert!(w.wires.get(16, 24, crate::wiring::WireColor::Red));
        assert!(w.wires.get(17, 24, crate::wiring::WireColor::Blue));
        assert!(w.wires.get(18, 24, crate::wiring::WireColor::Green));
        assert!(w.wires.get(19, 24, crate::wiring::WireColor::Yellow));
        assert_eq!(w.tile(28, 22).unwrap().liquid, 200);
        assert!(w.tile(28, 22).unwrap().flags & crate::tile::FLAG_LIQUID_LAVA != 0);
        assert!(w.tile(30, 22).unwrap().flags & crate::tile::FLAG_LIQUID_SHIMMER != 0);
        assert_eq!(w.tile(23, 23).unwrap().wall, 300); // the >255 wall
    }

    #[test]
    fn s1_j_3_the_modeled_sections_rewrite_byte_identical_to_the_csharp() {
        // The core S1-J-3 contract: the port reads the real C# file and re-writes
        // every section it models byte-for-byte. Sections 2..10 (tiles, chests,
        // signs, NPCs, tile entities, plates, town rooms, bestiary, creative
        // powers) and the footer are byte-identical; the prologue's pointer table
        // and the header/flags block carry the documented unowned-field
        // divergences (the next test pins those exactly).
        let loaded = golden_loaded();
        let bag = golden_bag(&loaded);
        let rewritten = to_bytes(&loaded.world, &bag, 1).expect("re-write");
        let f = section_bodies(GOLDEN_WLD);
        let r = section_bodies(&rewritten);
        // Sections 2..10 are byte-identical (the nine payload sections).
        for i in 2..11 {
            assert_eq!(f[i], r[i], "section {i} must re-write byte-identical to the C#");
        }
        // The footer (SaveFooter: bool true, name, WorldId) is byte-identical.
        let ffooter = &GOLDEN_WLD[SectionTable::open(GOLDEN_WLD).unwrap().pointers[10]..];
        let rfooter = &rewritten[SectionTable::open(&rewritten).unwrap().pointers[10]..];
        assert_eq!(ffooter, rfooter, "the footer must re-write byte-identical");
        // The prologue's framing (version, metadata, count, tile-count+bitmap)
        // matches; only the backfilled pointer values differ (the flags block's
        // length differs by the unowned fields). Compare the prologue with the
        // pointer table zeroed on both sides.
        let mut fp = f[0].to_vec();
        let mut rp = r[0].to_vec();
        fp[26..70].fill(0);
        rp[26..70].fill(0);
        assert_eq!(fp, rp, "the prologue framing + importance bitmap match");
    }

    #[test]
    fn s1_j_3_the_header_flags_divergences_are_exactly_the_unowned_fields() {
        // The header/flags block is NOT byte-identical: the port writes its
        // unowned fields at the C# default. This test pins the divergence set
        // exactly (a field-by-field walk in SaveWorldFlags order), so a future
        // change that owns one of these fields — or breaks an owned one — is
        // caught. The documented divergences (ANALYSIS "The C#-written golden
        // fixture"): the banner kill/claimable counts (no .wld store), the
        // tree-top variations (no store) and the world-gen manifest JSON (the
        // port writes the empty string). S1-J-3b closed CreationTime,
        // spawnTileY, the seven ore tiers (`World.creation_time`, the
        // file-faithful spawn read, `World.ore_tiers`), the saved-NPC flags
        // (wizard/mech/angler/stylist/taxCollector/golfer), the angler quest
        // index, the cultist/Moon Lord/Queen Slime kills, the four tower
        // downeds/actives + the apocalypse phase, and the party-girl/princess
        // spawn unlocks. Every OTHER field re-writes byte-identical.
        let loaded = golden_loaded();
        let bag = golden_bag(&loaded);
        let rewritten = to_bytes(&loaded.world, &bag, 1).expect("re-write");
        let f = section_bodies(GOLDEN_WLD);
        let r = section_bodies(&rewritten);
        let mut fr = crate::binio::Reader::new(f[1]);
        let mut rr = crate::binio::Reader::new(r[1]);
        let mut diffs: Vec<String> = Vec::new();
        macro_rules! cmp {
            ($name:expr, $m:ident) => {{
                let fv = fr.$m();
                let rv = rr.$m();
                if fv != rv { diffs.push($name.to_string()); }
            }};
        }
        // world head — all owned, all identical.
        cmp!("name", cs_string); cmp!("seed", cs_string); cmp!("generatorVersion", u64);
        for _ in 0..16 { cmp!("guid", u8); }
        cmp!("worldId", i32);
        for n in ["left","right","top","bottom","maxTilesY","maxTilesX"] { cmp!(n, i32); }
        // flags
        cmp!("GameMode", i32);
        for n in ["drunk","getGood","tenth","dontStarve","notTheBees","remix","noTraps","zenith","skyblock"] { cmp!(n, boolean); }
        cmp!("CreationTime", u64); cmp!("LastPlayed", u64);
        cmp!("moonType", u8);
        for _ in 0..3 { cmp!("treeX", i32); }
        for _ in 0..4 { cmp!("treeStyle", i32); }
        for _ in 0..3 { cmp!("caveBackX", i32); }
        for _ in 0..4 { cmp!("caveBackStyle", i32); }
        cmp!("iceBackStyle", i32); cmp!("jungleBackStyle", i32); cmp!("hellBackStyle", i32);
        cmp!("spawnTileX", i32); cmp!("spawnTileY", i32);
        cmp!("worldSurface", f64); cmp!("rockLayer", f64); cmp!("time", f64);
        cmp!("dayTime", boolean); cmp!("moonPhase", i32); cmp!("bloodMoon", boolean); cmp!("eclipse", boolean);
        cmp!("dungeonX", i32); cmp!("dungeonY", i32); cmp!("crimson", boolean);
        for n in ["downedBoss1","downedBoss2","downedBoss3","downedQueenBee","downedMech1","downedMech2","downedMech3","downedMechAny","downedPlant","downedGolem","downedSlimeKing","savedGoblin","savedWizard","savedMech","downedGoblins","downedClown","downedFrost","downedPirates"] { cmp!(n, boolean); }
        cmp!("shadowOrbSmashed", boolean); cmp!("spawnMeteor", boolean); cmp!("shadowOrbCount", u8); cmp!("altarCount", i32);
        cmp!("hardMode", boolean); cmp!("afterPartyOfDoom", boolean);
        cmp!("invasionDelay", i32); cmp!("invasionSize", i32); cmp!("invasionType", i32); cmp!("invasionX", f64);
        cmp!("slimeRainTime", f64); cmp!("sundialCooldown", u8);
        cmp!("raining", boolean); cmp!("rainTime", i32); cmp!("maxRain", f32);
        cmp!("Cobalt", i32); cmp!("Mythril", i32); cmp!("Adamantite", i32);
        for n in ["treeBG1","corruptBG","jungleBG","snowBG","hallowBG","crimsonBG","desertBG","oceanBG"] { cmp!(n, u8); }
        cmp!("cloudBGActive", i32); cmp!("numClouds", i16); cmp!("windSpeedTarget", f32);
        let fa = fr.i32().unwrap(); let ra = rr.i32().unwrap();
        assert_eq!(fa, ra, "angler count");
        for _ in 0..fa { fr.cs_string(); } for _ in 0..ra { rr.cs_string(); }
        cmp!("savedAngler", boolean); cmp!("anglerQuest", i32); cmp!("savedStylist", boolean);
        cmp!("savedTaxCollector", boolean); cmp!("savedGolfer", boolean);
        cmp!("invasionSizeStart", i32); cmp!("cultistDelay", i32);
        // BannerSystem.Save: the counts differ at index 1 (the fixture's banner).
        let fkc = fr.i16().unwrap(); let rkc = rr.i16().unwrap();
        assert_eq!(fkc, rkc);
        for i in 0..fkc {
            let a = fr.i32().unwrap(); let b = rr.i32().unwrap();
            if a != b { diffs.push(format!("killCount[{i}]")); }
        }
        let fcc = fr.i16().unwrap(); let rcc = rr.i16().unwrap();
        assert_eq!(fcc, rcc);
        for i in 0..fcc {
            let a = fr.u16().unwrap(); let b = rr.u16().unwrap();
            if a != b { diffs.push(format!("claimable[{i}]")); }
        }
        cmp!("fastForwardDawn", boolean);
        for n in ["downedFishron","downedMartians","downedCultist","downedMoonlord","downedHalloweenKing","downedHalloweenTree","downedChristmasIceQueen","downedChristmasSantank","downedChristmasTree","downedTowerSolar","downedTowerVortex","downedTowerNebula","downedTowerStardust","TowerActiveSolar","TowerActiveVortex","TowerActiveNebula","TowerActiveStardust","LunarApocalypse"] { cmp!(n, boolean); }
        cmp!("partyManual", boolean); cmp!("partyGenuine", boolean); cmp!("partyCooldown", i32);
        let fp = fr.i32().unwrap(); let rp = rr.i32().unwrap();
        assert_eq!(fp, rp);
        for _ in 0..fp { fr.i32(); } for _ in 0..rp { rr.i32(); }
        cmp!("sandstormHappening", boolean); cmp!("sandstormTimeLeft", i32); cmp!("sandstormSeverity", f32); cmp!("sandstormIntended", f32);
        cmp!("savedBartender", boolean); cmp!("DD2T1", boolean); cmp!("DD2T2", boolean); cmp!("DD2T3", boolean);
        for n in ["mushroomBG","underworldBG","treeBG2","treeBG3","treeBG4"] { cmp!(n, u8); }
        cmp!("combatBook", boolean);
        cmp!("lanternCooldown", i32); cmp!("lanternGenuine", boolean); cmp!("lanternManual", boolean); cmp!("lanternNextGenuine", boolean);
        let ft = fr.i32().unwrap(); let rt = rr.i32().unwrap();
        assert_eq!(ft, rt);
        for i in 0..ft {
            let a = fr.i32().unwrap(); let b = rr.i32().unwrap();
            if a != b { diffs.push(format!("treeTop[{i}]")); }
        }
        cmp!("forceHalloweenToday", boolean); cmp!("forceXMasToday", boolean);
        cmp!("Copper", i32); cmp!("Iron", i32); cmp!("Silver", i32); cmp!("Gold", i32);
        cmp!("boughtCat", boolean); cmp!("boughtDog", boolean); cmp!("boughtBunny", boolean);
        cmp!("downedEmpress", boolean); cmp!("downedQueenSlime", boolean); cmp!("downedDeerclops", boolean);
        for n in ["unlockedSlimeBlue","unlockedMerchant","unlockedDemo","unlockedPartyGirl","unlockedDyeTrader","unlockedTruffle","unlockedArmsDealer","unlockedNurse","unlockedPrincess"] { cmp!(n, boolean); }
        cmp!("combatBookTwo", boolean); cmp!("peddlersSatchel", boolean);
        for n in ["unlockedSlimeGreen","unlockedSlimeOld","unlockedSlimePurple","unlockedSlimeRainbow","unlockedSlimeRed","unlockedSlimeYellow","unlockedSlimeCopper"] { cmp!(n, boolean); }
        cmp!("fastForwardDusk", boolean); cmp!("moondialCooldown", u8);
        cmp!("forceHalloweenForever", boolean); cmp!("forceXMasForever", boolean);
        cmp!("vampireSeed", boolean); cmp!("infectedSeed", boolean);
        cmp!("meteorShowerCount", i32); cmp!("coinRain", i32);
        cmp!("teamBasedSpawns", boolean);
        let fe = fr.u8().unwrap(); let re = rr.u8().unwrap();
        assert_eq!(fe, re);
        for _ in 0..fe { fr.i16(); fr.i16(); } for _ in 0..re { rr.i16(); rr.i16(); }
        cmp!("dualDungeons", boolean); cmp!("moreLightning", boolean); cmp!("noLightning", boolean);
        cmp!("manifest", cs_string);
        // Both walks consume their whole section.
        assert_eq!(fr.pos(), f[1].len());
        assert_eq!(rr.pos(), r[1].len());
        // The exact divergence set — the unowned fields, nothing else.
        assert_eq!(
            diffs,
            vec!["killCount[1]", "claimable[1]", "treeTop[0]", "manifest",],
            "the header/flags divergences must be exactly the documented unowned fields"
        );
    }

    #[test]
    fn s1_j_3b_check_saved_ore_tiers_rederives_the_quartet_from_the_tiles() {
        // `WorldFile.cs:794-849`: any -1 quartet member triggers the tile
        // count; per pair the classic metal wins only on a STRICTLY greater
        // count (`num > num2`), a tie lands the alternate. All four pairs
        // re-derive together even when only one member was -1.
        let mut world = World::new("OreCheck".into(), 4, 8);
        world.ore_tiers = crate::ores::SavedOreTiers {
            copper: -1,
            iron: 6,
            silver: 9,
            gold: 8,
            cobalt: 107,
            mythril: 108,
            adamantite: 111,
        };
        // Two copper (7) vs one tin (166); one iron (6) vs one lead (167) —
        // the tie; no silver/gold tiles at all (0 vs 0 — also ties).
        for (x, y, kind) in [(0, 4, 7u16), (1, 4, 7), (2, 4, 166), (0, 5, 6), (1, 5, 167)] {
            let i = world.index(x, y);
            world.tiles[i].kind = kind;
            world.tiles[i].flags |= FLAG_ACTIVE;
        }
        // An INACTIVE copper tile does not count (`CountTileTypesInWorld`'s
        // `!tile.active()` continue, `WorldGen.cs:86892-86895`).
        let inactive = world.index(3, 4);
        world.tiles[inactive].kind = 7;
        check_saved_ore_tiers(&mut world);
        assert_eq!(world.ore_tiers.copper, 7, "2 > 1 keeps the classic");
        assert_eq!(world.ore_tiers.iron, 167, "the tie lands the alternate");
        assert_eq!(world.ore_tiers.silver, 168, "0 vs 0 lands the alternate");
        assert_eq!(world.ore_tiers.gold, 169);
        // The hardmode trio is untouched by the check.
        assert_eq!(world.ore_tiers.cobalt, 107);
        assert_eq!(world.ore_tiers.mythril, 108);
        assert_eq!(world.ore_tiers.adamantite, 111);
        // A fully-set quartet is a no-op (`:796-799`'s early return) even when
        // the tiles would say otherwise.
        world.ore_tiers.copper = 166;
        check_saved_ore_tiers(&mut world);
        assert_eq!(world.ore_tiers.copper, 166, "no -1: the check returns");
    }

    #[test]
    fn s1_j_3b_a_pre_216_wld_resets_and_rederives_the_ore_tiers() {
        // `LoadWorldFlags:2349-2354`: below 216 the quartet is not in the file
        // and resets to -1; the post-load `CheckSavedOreTiers`
        // (`WorldFile.cs:730`) then re-derives it from the tiles. The v150
        // synthetic fixture carries no ore tiles, so every pair ties and all
        // four alternates land. v150 carries seven sections (tile entities
        // exist from 116; pressure plates only from 170).
        let data = synthetic_file(150, &[2, 144], 160, 7);
        let world = parse(&data).expect("v150 loads");
        assert_eq!(world.ore_tiers.copper, 166);
        assert_eq!(world.ore_tiers.iron, 167);
        assert_eq!(world.ore_tiers.silver, 168);
        assert_eq!(world.ore_tiers.gold, 169);
        // The trio came from the file's zeros (the synthetic flags table).
        assert_eq!(world.ore_tiers.cobalt, 0);
        assert_eq!(world.ore_tiers.mythril, 0);
        assert_eq!(world.ore_tiers.adamantite, 0);
    }

    #[test]
    fn s1_j_3b_the_closed_header_fields_rewrite_byte_identical_to_the_fixture() {
        // The S1-J-3b contract slice: the fixture's world head plus the flags
        // block UP TO the first remaining unowned field (the banner kill
        // counts) re-writes byte-for-byte — CreationTime, spawnTileY and the
        // Cobalt/Mythril/Adamantite trio included. (The quartet after the tree
        // tops is pinned by the divergence walk above: Copper..Gold no longer
        // appear in its diff list.)
        let loaded = golden_loaded();
        let bag = golden_bag(&loaded);
        let rewritten = to_bytes(&loaded.world, &bag, 1).expect("re-write");
        let f = section_bodies(GOLDEN_WLD);
        let r = section_bodies(&rewritten);
        // The same field walk as the divergence test, stopping after
        // `_tempCultistDelay` — the last owned field before `BannerSystem.Save`.
        let mut fr = crate::binio::Reader::new(f[1]);
        let mut rr = crate::binio::Reader::new(r[1]);
        macro_rules! both {
            ($m:ident) => {{
                assert_eq!(fr.$m(), rr.$m());
            }};
        }
        both!(cs_string); both!(cs_string); both!(u64); // name, seed, genVer
        for _ in 0..16 { both!(u8); } // GUID
        for _ in 0..7 { both!(i32); } // worldId + bounds + dims
        both!(i32); // GameMode
        for _ in 0..9 { both!(boolean); } // flavour bools
        both!(u64); both!(u64); // CreationTime, LastPlayed
        both!(u8); // moonType
        for _ in 0..17 { both!(i32); } // tree/cave arrays + back styles
        both!(i32); both!(i32); // spawnTileX/Y
        both!(f64); both!(f64); both!(f64); // surface, rock, time
        both!(boolean); both!(i32); // dayTime, moonPhase
        for _ in 0..2 { both!(boolean); } // bloodMoon, eclipse
        both!(i32); both!(i32); both!(boolean); // dungeonX/Y, crimson
        for _ in 0..18 { both!(boolean); } // downed/saved flags
        both!(boolean); both!(boolean); both!(u8); both!(i32); // orbs..altars
        both!(boolean); both!(boolean); // hardMode, afterPartyOfDoom
        for _ in 0..3 { both!(i32); } // invasion delay/size/type
        both!(f64); both!(f64); both!(u8); // invasionX, slimeRain, sundial
        both!(boolean); both!(i32); both!(f32); // the rain triple
        for _ in 0..3 { both!(i32); } // the ore trio — now owned
        for _ in 0..8 { both!(u8); } // setBG 0..7
        both!(i32); both!(i16); both!(f32); // clouds + wind
        let n = fr.i32().unwrap();
        assert_eq!(n, rr.i32().unwrap());
        for _ in 0..n { both!(cs_string); } // angler list
        both!(boolean); both!(i32); // savedAngler, anglerQuest
        for _ in 0..3 { both!(boolean); } // savedStylist..savedGolfer
        both!(i32); both!(i32); // invasionSizeStart, _tempCultistDelay
        let cut = fr.pos();
        assert_eq!(cut, rr.pos());
        assert_eq!(
            &f[1][..cut],
            &r[1][..cut],
            "the header through the ore trio must re-write byte-identical"
        );
    }

    /// Locates the flags tail inside a synthetic file's pointer-1 section:
    /// the block ends the section, so the tail is its last `flags_tail(v)`
    /// bytes — independent of the world-head width.
    fn tail_offset_in_synthetic(data: &[u8], version: i32) -> usize {
        // The pointer table starts after the version i32 (+ the 20-byte
        // FileMetadata from v135) + the i16 section count; pointer 1 is the
        // second entry (`SaveFileFormatHeader:1174-1180`).
        let ptr_at = 4 + if version >= 135 { 20 } else { 0 } + 2;
        let ptr1 = i32::from_le_bytes(data[ptr_at + 4..ptr_at + 8].try_into().unwrap()) as usize;
        ptr1 - flags_tail(version).len()
    }

    #[test]
    fn s1_j_3b_the_newly_owned_flags_land_at_their_csharp_byte_offsets() {
        // The byte-layout contract for the S1-J-3b close set: patch each
        // newly-owned flag into the independent `flags_tail` table at its C#
        // offset (`SaveWorldFlags:1294-1406`), then the reader must land every
        // value in its World field. The offsets are the running sum of the
        // field widths — the same ones `expected_flags_block` pins for the
        // writer, here checked against the reader.
        let mut data = fixture();
        let tail_at = tail_offset_in_synthetic(&data, VER);
        let patch = |data: &mut Vec<u8>, off: usize, v: u8| data[tail_at + off] = v;
        patch(&mut data, 23, 1); // savedWizard (`:1294`)
        patch(&mut data, 24, 1); // savedMech (`:1295`)
        patch(&mut data, 110, 1); // savedAngler (`:1334`)
        data[tail_at + 111..tail_at + 115].copy_from_slice(&17i32.to_le_bytes()); // anglerQuest (`:1335`)
        patch(&mut data, 115, 1); // savedStylist (`:1336`)
        patch(&mut data, 116, 1); // savedTaxCollector (`:1337`)
        patch(&mut data, 117, 1); // savedGolfer (`:1338`)
        patch(&mut data, 1891, 1); // downedAncientCultist (`:1345`)
        patch(&mut data, 1892, 1); // downedMoonlord (`:1346`)
        for off in 1898..=1901 {
            patch(&mut data, off, 1); // downedTower{Solar,Vortex,Nebula,Stardust} (`:1352-1355`)
        }
        for off in 1902..=1905 {
            patch(&mut data, off, 1); // TowerActive{Solar,Vortex,Nebula,Stardust} (`:1356-1359`)
        }
        patch(&mut data, 1906, 1); // LunarApocalypseIsUp (`:1360`)
        patch(&mut data, 2025, 1); // downedQueenSlime (`:1396`)
        patch(&mut data, 2030, 1); // unlockedPartyGirlSpawn (`:1401`)
        patch(&mut data, 2035, 1); // unlockedPrincessSpawn (`:1406`)
        let w = parse(&data).expect("the patched fixture parses");
        assert!(w.saved_wizard, "savedWizard at tail 23");
        assert!(w.saved_mech, "savedMech at tail 24");
        assert!(w.saved_angler, "savedAngler at tail 110");
        assert_eq!(w.angler_quest, 17, "anglerQuest at tail 111 (i32 -> u8)");
        assert!(w.saved_stylist, "savedStylist at tail 115");
        assert!(w.saved_tax_collector, "savedTaxCollector at tail 116");
        assert!(w.saved_golfer, "savedGolfer at tail 117");
        assert!(w.downed_ancient_cultist, "downedAncientCultist at tail 1891");
        assert!(w.downed_moon_lord, "downedMoonlord at tail 1892");
        assert!(w.downed_tower_solar && w.downed_tower_vortex);
        assert!(w.downed_tower_nebula && w.downed_tower_stardust);
        assert!(w.tower_active_solar && w.tower_active_vortex);
        assert!(w.tower_active_nebula && w.tower_active_stardust);
        assert!(w.lunar_apocalypse_is_up, "LunarApocalypseIsUp at tail 1906");
        assert!(w.downed_queen_slime, "downedQueenSlime at tail 2025");
        assert!(w.unlocked_party_girl, "unlockedPartyGirlSpawn at tail 2030");
        assert!(w.unlocked_princess, "unlockedPrincessSpawn at tail 2035");
        // The writer's own block carries the same values at the same offsets
        // (`expected_flags_block` is the independent table): the owned-world
        // sweep round-trips through the whole block, not just the fields.
        let world = owned_world();
        let buf = to_bytes(&world, &SectionData::default(), 1).expect("writes");
        let ptr1 = i32::from_le_bytes(buf[30..34].try_into().unwrap()) as usize;
        let want = expected_flags_block(&world);
        // The block ends pointer 1's section; anchor from its end so the
        // world-head width (name/seed text lengths) does not matter.
        let flags_at = ptr1 - want.len();
        assert_eq!(
            &buf[flags_at..ptr1],
            &want[..],
            "the owned flags block matches the C# table byte for byte"
        );
    }

    #[test]
    fn s1_j_3b_active_towers_restore_their_shields_on_load() {
        // `LoadWorldFlags:2235-2250`: an active tower comes back at
        // `ShieldStrengthTowerMax` (`NPC.cs:6795-6806` — the normal power,
        // halved once the Moon Lord is downed). The `.wld` does not serialize
        // the shield strengths, so the restore is the only source.
        let mut data = fixture();
        let tail_at = tail_offset_in_synthetic(&data, VER);
        // Solar + nebula active, vortex + stardust downed but inactive; the
        // Moon Lord still up -> the full normal power (100, the World default).
        data[tail_at + 1902] = 1; // TowerActiveSolar
        data[tail_at + 1904] = 1; // TowerActiveNebula
        data[tail_at + 1899] = 1; // downedTowerVortex (inactive)
        data[tail_at + 1901] = 1; // downedTowerStardust (inactive)
        let w = parse(&data).expect("the tower fixture parses");
        assert_eq!(w.tower_shield_solar, 100, "active solar: full shield");
        assert_eq!(w.tower_shield_nebula, 100, "active nebula: full shield");
        assert_eq!(w.tower_shield_vortex, 0, "inactive vortex keeps 0");
        assert_eq!(w.tower_shield_stardust, 0, "inactive stardust keeps 0");
        // With the Moon Lord downed the max halves (100 / 2 = 50).
        data[tail_at + 1892] = 1; // downedMoonlord
        let w = parse(&data).expect("the post-Moon-Lord fixture parses");
        assert!(w.downed_moon_lord);
        assert_eq!(w.tower_shield_solar, 50, "the halved max");
        assert_eq!(w.tower_shield_nebula, 50, "the halved max");
    }

    #[test]
    fn s1_j_3b_the_saved_npc_and_unlock_gates_follow_the_csharp() {
        // The C# read gates (`LoadWorldFlags`): savedAngler @99 (`:2161`),
        // anglerQuest @101 (`:2166`), savedStylist @104 (`:2171`),
        // savedTaxCollector @129 (`:2174`), savedGolfer @201 (`:2178`),
        // downedQueenSlime @223 (`:2371`), the eight unlocks @251 (`:2396-2403`).
        // Below a gate the field keeps its fresh-world default.
        // v98: the block ends at the angler-list count (`flags_tail` < 99).
        let w = parse(&synthetic_file(98, &[2, 144], 160, 6)).expect("v98 loads");
        assert!(!w.saved_angler, "below 99: savedAngler stays false");
        // v100: savedAngler lands, the quest index does not exist yet. The
        // pre-118 tail lacks the slimeKing byte, afterPartyOfDoom, the slime
        // rain double and the sundial byte (11 bytes), so savedAngler sits at
        // 110 - 11 = 99.
        let mut data = synthetic_file(100, &[2, 144], 160, 6);
        let tail_at = tail_offset_in_synthetic(&data, 100);
        data[tail_at + 99] = 1; // savedAngler
        let w = parse(&data).expect("v100 loads");
        assert!(w.saved_angler, "savedAngler reads from 99");
        assert_eq!(w.angler_quest, 0, "below 101: the quest index stays 0");
        // v102: the quest index lands (i32), the stylist does not exist yet.
        let mut data = synthetic_file(102, &[2, 144], 160, 6);
        let tail_at = tail_offset_in_synthetic(&data, 102);
        data[tail_at + 99] = 1; // savedAngler
        data[tail_at + 100..tail_at + 104].copy_from_slice(&23i32.to_le_bytes());
        let w = parse(&data).expect("v102 loads");
        assert!(w.saved_angler);
        assert_eq!(w.angler_quest, 23, "anglerQuest reads from 101");
        assert!(!w.saved_stylist, "below 104: savedStylist stays false");
        // v104: the stylist lands; the tax collector needs 129.
        let mut data = synthetic_file(104, &[2, 144], 160, 6);
        let tail_at = tail_offset_in_synthetic(&data, 104);
        data[tail_at + 104] = 1; // savedStylist
        let w = parse(&data).expect("v104 loads");
        assert!(w.saved_stylist, "savedStylist reads from 104");
        assert!(!w.saved_tax_collector, "below 129: stays false");
        // v250: the slime-blue byte exists (250) but the eight unlocks need
        // 251 — the party girl/princess stay false. Eleven pointers: bestiary
        // (210) and creative powers (220) both exist by now.
        let w = parse(&synthetic_file(250, &[2, 144], 160, 11)).expect("v250 loads");
        assert!(!w.unlocked_party_girl, "below 251: the unlocks stay false");
        assert!(!w.unlocked_princess);
        // v251: the unlock block lands — party girl (3rd) and princess (8th).
        let mut data = synthetic_file(251, &[2, 144], 160, 11);
        let tail_at = tail_offset_in_synthetic(&data, 251);
        // The v251 tail lacks the 257 afterPartyOfDoom byte and the 289
        // claimable array (2 + 586) relative to v326 — 589 bytes before the
        // unlocks: partyGirl@2030-589=1441, princess@2035-589=1446.
        data[tail_at + 1441] = 1; // unlockedPartyGirlSpawn
        data[tail_at + 1446] = 1; // unlockedPrincessSpawn
        let w = parse(&data).expect("v251 loads");
        assert!(w.unlocked_party_girl, "unlockedPartyGirlSpawn reads from 251");
        assert!(w.unlocked_princess, "unlockedPrincessSpawn reads from 251");
    }

    /// Gate V3 (S1-P-i35) — the two newly-owned flag bytes follow their C#
    /// version gates (`WorldFile.LoadWorldFlags`): `combatBookWasUsed` reads
    /// from v204 (`:2233`), `unlockedSlimeRedSpawn` from v261 (`:2404-2410`).
    /// Below a gate the field keeps its fresh-world false; at/above it the
    /// byte lands. Offsets derive from `flags_tail(version)`'s own width, so
    /// the check is independent of the v326 layout.
    #[test]
    fn gate_v3_i35_the_combat_book_and_red_slime_flags_follow_their_version_gates() {
        // v203: the combat book byte does not exist yet.
        let w = parse(&synthetic_file(203, &[2, 144], 160, 9)).expect("v203 loads");
        assert!(!w.combat_book_was_used, "below 204: combatBookWasUsed stays false");
        // v204: it lands. The book is the LAST byte of the v204 tail (the
        // lantern snapshot needs 207, tree tops 211).
        let mut data = synthetic_file(204, &[2, 144], 160, 9);
        let tail_at = tail_offset_in_synthetic(&data, 204);
        let book_at = flags_tail(204).len() - 1;
        data[tail_at + book_at] = 1;
        let w = parse(&data).expect("v204 loads");
        assert!(w.combat_book_was_used, "combatBookWasUsed reads from 204");
        assert!(!w.unlocked_slime_red_spawn, "below 261: the red slime stays false");
        // v260: the slime block does not exist yet (the satchel byte is last).
        let mut data = synthetic_file(260, &[2, 144], 160, 11);
        let tail_at = tail_offset_in_synthetic(&data, 260);
        data[tail_at + flags_tail(260).len() - 1] = 1; // peddlersSatchelWasUsed — skipped
        let w = parse(&data).expect("v260 loads");
        assert!(!w.unlocked_slime_red_spawn, "below 261: red slime stays false");
        // v261: green/old/purple(3) rainbow(1) red(1) yellow/copper(2) — the
        // red byte sits 3 from the tail end (2 skip + 1 red).
        let mut data = synthetic_file(261, &[2, 144], 160, 11);
        let tail_at = tail_offset_in_synthetic(&data, 261);
        let red_at = flags_tail(261).len() - 3;
        data[tail_at + red_at] = 1;
        let w = parse(&data).expect("v261 loads");
        assert!(w.unlocked_slime_red_spawn, "unlockedSlimeRedSpawn reads from 261");
        assert!(!w.unlocked_slime_rainbow_spawn, "the rainbow byte before it stays 0");
        // v326 (the writer's version): both round-trip at their pinned tail
        // offsets — the DW-2.2 owned-fields sweep covers the values; here the
        // raw bytes prove the offsets (book@1939, red@2042).
        let world = owned_world();
        let buf = to_bytes(&world, &SectionData::default(), 1).expect("writes");
        let ptr1 = i32::from_le_bytes(buf[30..34].try_into().unwrap()) as usize;
        let tail_at = ptr1 - flags_tail(VER).len();
        assert_eq!(buf[tail_at + 1939], 1, "combatBookWasUsed at tail 1939");
        assert_eq!(buf[tail_at + 2042], 1, "unlockedSlimeRedSpawn at tail 2042");
    }

    // --- S1-A-7: the C#-generated world (real genRand) -----------------------
    //
    // `assets/test/parity_real.wld` is written by tools/WorldGenParityHarness in
    // REAL mode: the verbatim C# WorldGen.TileRunner driven by the REAL Knuth
    // UnifiedRandom(42) (WorldGen.genRand), then the real save path. This is a
    // genuine C#-generated world — the artifact the ROADMAP row asks for. It is
    // NOT byte-comparable with the port's generator (the port draws xorshift64,
    // not Knuth; research/35 #5), so this test pins what IS comparable: the port
    // decodes every section, and re-writes the TILE section byte-identical (the
    // tile codec is stream-independent — S1-J-3 proved the same for the golden
    // fixture). The algorithm-level parity proof (the port's tile_runner vs the
    // verbatim C# TileRunner fed the port's stream) lives in
    // worldgen/tile_runner.rs::s1_a_7_the_tile_runner_matches_the_csharp_differential.

    /// The C#-generated world (real genRand, seed 42), written by
    /// tools/WorldGenParityHarness REAL mode.
    const PARITY_REAL_WLD: &[u8] = include_bytes!("../../../../assets/test/parity_real.wld");

    #[test]
    fn s1_a_7_the_csharp_generated_world_decodes_and_rewrites_tiles() {
        let loaded = parse_with_sections(PARITY_REAL_WLD).expect("the C#-generated world parses");
        let w = &loaded.world;
        assert_eq!((w.width, w.height), (200, 400));
        assert_eq!(w.name, "S1A7Parity");
        assert_eq!(loaded.version, 326);
        // The runner painted ore (7), mud (59) and carved pockets — the world is
        // not the all-stone input.
        assert!(w.tiles.iter().any(|t| t.active() && t.kind == 7), "ore placed");
        assert!(w.tiles.iter().any(|t| t.active() && t.kind == 59), "mud placed");
        assert!(w.tiles.iter().any(|t| !t.active()), "pockets carved");
        // Re-write: the tile section (index 2) is byte-identical — the tile codec
        // is stream-independent, exactly as S1-J-3 proved for the golden fixture.
        let bag = loaded.sections.clone();
        let rewritten = to_bytes(w, &bag, 1).expect("re-write");
        let f = section_bodies(PARITY_REAL_WLD);
        let r = section_bodies(&rewritten);
        assert_eq!(f[2], r[2], "the tile section must re-write byte-identical to the C#");
    }
}
