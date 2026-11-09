//! Native save format (`TRWLD`) for generated worlds.
//!
//! Deliberately tiny and lossless for everything the renderer needs: header fields,
//! the frame-importance bitset and the full 14-byte tile struct, RLE-compressed per
//! row. The original `.wld` writer is a much larger job; this format exists so the
//! generator can persist a world under `Documents/My Games/Terrustaria`.

use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};

use crate::tile::Tile;
use crate::world::World;

const MAGIC: &[u8; 8] = b"TRWLD01\0";
/// v2 added `moon_type`/`moon_phase`; v1 files still load with 0/0. v3 added
/// the sparse paint/coating extras; v4 appended the two dial cooldown bytes;
/// v5 appended the bed spawn (the vanilla keeps it in the *player* file's
/// `spX/spY` arrays, `Player.cs:54877-54888` — the port's single-player world
/// save carries it world-side, `World::bed_spawn`); v6 appended the classic
/// secret-seed flags (`World.seeds` — the `SaveWorldFlags` byte block,
/// `WorldFile.cs:1238-1250`, as the port's own u16 bitmask; the C# `.wld`
/// stores the nine booleans and re-derives the seed from the stored seed
/// *text*).
pub const VERSION: u32 = 6;

pub fn default_world_dir() -> Option<PathBuf> {
    let home = std::env::var_os("USERPROFILE")?;
    Some(
        PathBuf::from(home)
            .join("Documents")
            .join("My Games")
            .join("Terrustaria")
            .join("Worlds"),
    )
}

pub fn default_world_path(name: &str) -> Option<PathBuf> {
    Some(default_world_dir()?.join(format!("{name}.trwld")))
}

/// `Main.PlayerPath` (`Main.cs:67065-67075`): `SavePath/Players` — the
/// `Worlds` dir's parent joined with `Players` (`Terrustaria/Players`).
pub fn default_players_dir() -> Option<PathBuf> {
    Some(default_world_dir()?.parent()?.join("Players"))
}

/// `AchievementManager`'s save path (`Main.SavePath/achievements.dat`) —
/// the port writes its own text format beside the vanilla name
/// (`achievements.trdat`; M6-ACH-1's documented deviation).
pub fn achievements_path() -> Option<PathBuf> {
    Some(default_world_dir()?.parent()?.join("achievements.trdat"))
}

/// `Main.GetPlayerPathFromName` (`Main.cs:4820-4841`): `.` and `*` become
/// `_`, the Windows invalid-filename set becomes `-`, spaces become `_`,
/// then `<name>.plr` under [`default_players_dir`].
pub fn default_player_path(name: &str) -> Option<PathBuf> {
    const INVALID: &[char] = &['"', '<', '>', '|', ':', '?', '\\', '/', '\u{0}'];
    let mut safe = String::with_capacity(name.len());
    for c in name.chars() {
        safe.push(if c == '.' || c == '*' {
            '_'
        } else if INVALID.contains(&c) {
            '-'
        } else if c == ' ' {
            '_'
        } else {
            c
        });
    }
    Some(default_players_dir()?.join(format!("{safe}.plr")))
}

pub fn save(path: &Path, world: &World) -> Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
    }
    let tmp = path.with_extension("trwld.tmp");
    {
        let file =
            std::fs::File::create(&tmp).with_context(|| format!("creating {}", tmp.display()))?;
        let mut w = std::io::BufWriter::new(file);
        write_world(&mut w, world)?;
        w.flush()?;
    }
    std::fs::rename(&tmp, path).with_context(|| format!("replacing {}", path.display()))?;
    Ok(())
}

/// Serialises a world to memory (multiplayer world transfer).
pub fn to_bytes(world: &World) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    write_world(&mut out, world)?;
    Ok(out)
}

fn write_world(w: &mut impl Write, world: &World) -> Result<()> {
    w.write_all(MAGIC)?;
    w.write_all(&VERSION.to_le_bytes())?;
    w.write_all(&world.seed.to_le_bytes())?;
    let name = world.name.as_bytes();
    w.write_all(&(name.len() as u32).to_le_bytes())?;
    w.write_all(name)?;
    w.write_all(&world.width.to_le_bytes())?;
    w.write_all(&world.height.to_le_bytes())?;
    w.write_all(&world.spawn_x.to_le_bytes())?;
    w.write_all(&world.spawn_y.to_le_bytes())?;
    w.write_all(&world.world_surface.to_le_bytes())?;
    w.write_all(&world.rock_layer.to_le_bytes())?;
    w.write_all(&world.time.to_le_bytes())?;
    w.write_all(&[u8::from(world.day_time)])?;
    w.write_all(&[world.moon_type, world.moon_phase])?;
    let imp = &world.frame_important;
    w.write_all(&(imp.len() as u32).to_le_bytes())?;
    let packed: Vec<u8> = imp.iter().map(|&b| u8::from(b)).collect();
    w.write_all(&packed)?;

    let width = world.width as usize;
    let mut run_tiles: Vec<Tile> = Vec::new();
    let mut run_counts: Vec<u16> = Vec::new();
    for y in 0..world.height as usize {
        run_tiles.clear();
        run_counts.clear();
        let row = &world.tiles[y * width..(y + 1) * width];
        for &t in row {
            if let Some(last) = run_tiles.last() {
                if *last == t && *run_counts.last().unwrap() < u16::MAX {
                    *run_counts.last_mut().unwrap() += 1;
                    continue;
                }
            }
            run_tiles.push(t);
            run_counts.push(1);
        }
        w.write_all(&(run_tiles.len() as u32).to_le_bytes())?;
        for (t, n) in run_tiles.iter().zip(run_counts.iter()) {
            write_tile(w, t)?;
            w.write_all(&n.to_le_bytes())?;
        }
    }

    // v3: the sparse paint/coating extras (`World.extras`) — wall paint,
    // invisible block/wall, fullbright wall. Sorted by index for determinism.
    let mut extras: Vec<(u32, u8)> = world.extras.iter().map(|(k, v)| (*k, *v)).collect();
    extras.sort_unstable();
    w.write_all(&(extras.len() as u32).to_le_bytes())?;
    for (idx, byte) in extras {
        w.write_all(&idx.to_le_bytes())?;
        w.write_all(&[byte])?;
    }
    // v4: the dial cooldowns (`Main.sundialCooldown`/`Main.moondialCooldown`,
    // `Main.cs:2161`/`:2165`); 0..=8 each. The fast-forward flags stay
    // runtime-only — the C# `.wld` has written `fastForwardTimeToDusk` since
    // v264, but the request is transient by nature.
    w.write_all(&[world.sundial_cooldown, world.moondial_cooldown])?;
    // v5: the bed spawn (`ChangeSpawn`'s cell — the CheckSpawn-validated
    // ground cell under the bed, `Player.cs:54635-54660`).
    match world.bed_spawn {
        Some((x, y)) => {
            w.write_all(&[1u8])?;
            w.write_all(&x.to_le_bytes())?;
            w.write_all(&y.to_le_bytes())?;
        }
        None => w.write_all(&[0u8])?,
    }
    // v6: the secret-seed flags (`WorldFile.SaveWorldFlags`'s nine
    // `Main.*` booleans, `WorldFile.cs:1238-1250`) as the port's own
    // `serializedSeedsSum` bitmask (`WorldFileData.cs:157-197`).
    w.write_all(&world.seeds.sum().to_le_bytes())?;
    Ok(())
}

fn write_tile(w: &mut impl Write, t: &Tile) -> Result<()> {
    w.write_all(&t.kind.to_le_bytes())?;
    w.write_all(&t.wall.to_le_bytes())?;
    w.write_all(&t.frame_x.to_le_bytes())?;
    w.write_all(&t.frame_y.to_le_bytes())?;
    w.write_all(&[
        t.liquid,
        t.flags,
        t.slope(),
        t.paint(),
        t.wall_col(),
        t.wall_row(),
    ])?;
    Ok(())
}

pub fn load(path: &Path) -> Result<World> {
    let data = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    parse(&data).with_context(|| format!("parsing {}", path.display()))
}

pub fn parse(data: &[u8]) -> Result<World> {
    let mut r = Reader::new(data);
    if r.take(8)? != MAGIC {
        bail!("not a TRWLD save");
    }
    let version = r.u32()?;
    if version == 0 || version > VERSION {
        bail!("unsupported TRWLD version {version}");
    }
    let seed = r.u64()?;
    let name_len = r.u32()? as usize;
    let name = String::from_utf8_lossy(r.take(name_len)?).into_owned();
    let width = r.i32()?;
    let height = r.i32()?;
    if width <= 0 || height <= 0 || width > 20_000 || height > 20_000 {
        bail!("implausible world size {width}x{height}");
    }
    let mut world = World::new(name, width, height);
    world.seed = seed;
    world.spawn_x = r.i32()?;
    world.spawn_y = r.i32()?;
    world.world_surface = r.f64()?;
    world.rock_layer = r.f64()?;
    world.time = r.f64()?;
    world.day_time = r.u8()? != 0;
    if version >= 2 {
        world.moon_type = r.u8()?;
        world.moon_phase = r.u8()?;
    }
    let imp_len = r.u32()? as usize;
    if imp_len > 1 << 16 {
        bail!("bad importance table length {imp_len}");
    }
    world.frame_important = r.take(imp_len)?.iter().map(|&b| b != 0).collect();

    let w = width as usize;
    for y in 0..height as usize {
        let runs = r.u32()? as usize;
        let mut x = 0usize;
        for _ in 0..runs {
            let t = read_tile(&mut r)?;
            let n = r.u16()? as usize;
            if x + n > w {
                bail!("RLE run overflows row {y}");
            }
            let start = y * w + x;
            world.tiles[start..start + n].fill(t);
            x += n;
        }
        if x != w {
            bail!("row {y} has {x} of {w} tiles");
        }
    }
    if version >= 3 {
        let count = r.u32()? as usize;
        for _ in 0..count {
            let idx = r.u32()? as usize;
            let byte = r.u8()?;
            if idx >= world.tiles.len() {
                bail!("extras index {idx} out of range");
            }
            world.set_extras_at(idx, byte);
        }
    }
    // v4: the dial cooldowns; a v3 file (and older) loads with the 0s from
    // `World::new`.
    if version >= 4 {
        world.sundial_cooldown = r.u8()?;
        world.moondial_cooldown = r.u8()?;
    }
    // v5: the bed spawn. A corrupt/out-of-range cell degrades to `None` —
    // the payload rides disk reads (the load is a trust boundary) and a
    // dropped spawn point only falls back to the world spawn
    // (`Player.cs:37571-37575`'s `SpawnX = -1` arm).
    if version >= 5 && r.u8()? != 0 {
        let x = r.i32()?;
        let y = r.i32()?;
        if x >= 10 && x < world.width - 10 && y >= 10 && y < world.height - 10 {
            world.bed_spawn = Some((x, y));
        }
    }
    // v6: the secret-seed flags (`SaveWorldFlags`'s block, the port's bitmask
    // mirror). A v5 file loads with the classic (all-false) world, which is
    // what the pre-M5-SEED-1 saves were.
    if version >= 6 {
        world.seeds = crate::seeds::SeedFlags::from_sum(r.u16()?);
    }
    world.revision = 0;
    Ok(world)
}

fn read_tile(r: &mut Reader) -> Result<Tile> {
    let mut tile = Tile {
        kind: r.u16()?,
        wall: r.u16()?,
        frame_x: r.i16()?,
        frame_y: r.i16()?,
        liquid: r.u8()?,
        flags: r.u8()?,
        ..Default::default()
    };
    let slope = r.u8()?;
    let paint = r.u8()?;
    let col = r.u8()?;
    let row = r.u8()?;
    tile.set_slope(slope);
    tile.set_paint(paint);
    tile.set_wall_frame(col, row);
    Ok(tile)
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

    fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        self.0.take(n).ok_or_else(|| eof(self.0.pos()))
    }

    fn u8(&mut self) -> Result<u8> {
        self.0.u8().ok_or_else(|| eof(self.0.pos()))
    }

    fn u16(&mut self) -> Result<u16> {
        self.0.u16().ok_or_else(|| eof(self.0.pos()))
    }

    fn i16(&mut self) -> Result<i16> {
        self.0.i16().ok_or_else(|| eof(self.0.pos()))
    }

    fn u32(&mut self) -> Result<u32> {
        self.0.u32().ok_or_else(|| eof(self.0.pos()))
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::FLAG_ACTIVE;

    #[test]
    fn round_trips_tiles_and_header() {
        let mut w = World::new("Test".into(), 64, 48);
        w.seed = 99;
        w.world_surface = 12.5;
        w.rock_layer = 30.0;
        w.spawn_x = 32;
        w.spawn_y = 10;
        w.time = 27000.0;
        w.moon_type = 5;
        w.moon_phase = 3;
        w.frame_important = vec![false; 754];
        w.frame_important[5] = true;
        for x in 0..64 {
            let i = w.index(x, 20);
            w.tiles[i] = Tile {
                kind: 2,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        let i = w.index(5, 30);
        w.tiles[i] = Tile {
            kind: 0,
            wall: 2,
            frame_x: 18,
            frame_y: 54,
            liquid: 200,
            flags: FLAG_ACTIVE | crate::tile::FLAG_LIQUID_LAVA,
            ..Default::default()
        };
        w.tiles[i].set_slope(2);
        w.tiles[i].set_paint(3);
        w.tiles[i].set_wall_frame(4, 2);
        // v3 extras: wall paint + the Echo bit on (3, 0); the zero byte at
        // (7, 7) collapses back to absent.
        w.set_wall_paint(3, 0, 4);
        w.set_invisible_block(3, 0, true);
        w.set_extras_at(w.index(7, 7), 0);
        let mut buf = Vec::new();
        write_world(&mut buf, &w).unwrap();
        let back = parse(&buf).unwrap();
        assert_eq!(back.width, 64);
        assert_eq!(back.name, "Test");
        assert_eq!(back.seed, 99);
        let t = back.tiles[back.index(5, 30)];
        assert_eq!(
            (t.slope(), t.paint(), t.wall_col(), t.wall_row()),
            (2, 3, 4, 2)
        );
        assert_eq!(back.world_surface, 12.5);
        assert_eq!((back.moon_type, back.moon_phase), (5, 3));
        assert!(back.frame_important[5]);
        assert_eq!(back.tiles, w.tiles);
        // v3: the sparse paint/coating extras round-trip (the zero byte
        // collapses back to absent).
        assert_eq!(back.extras.len(), 1);
        assert_eq!(back.extras_at(back.index(3, 0)), 0x24);
        assert_eq!(back.wall_paint(3, 0), 4, "wall paint rides the low 5 bits");
        assert!(back.invisible_block(3, 0), "the Echo bit rides bit 5");
        assert!(!back.invisible_wall(3, 0));
        assert!(!back.invisible_block(7, 7), "the zero byte collapses");
    }

    #[test]
    fn dw_1_4_dial_cooldowns_round_trip_v4() {
        // DW-1.4: the v4 tail carries both cooldown bytes (`Main.cs:2161`/`:2165`).
        let mut w = World::new("Test".into(), 32, 32);
        w.sundial_cooldown = 5;
        w.moondial_cooldown = 8;
        let mut buf = Vec::new();
        write_world(&mut buf, &w).unwrap();
        assert_eq!(&buf[8..12], &[6, 0, 0, 0], "the writer emits the current v6");
        let back = parse(&buf).unwrap();
        assert_eq!(back.sundial_cooldown, 5);
        assert_eq!(back.moondial_cooldown, 8);
    }

    #[test]
    fn dw_1_4_v3_loads_zero_cooldowns() {
        // DW-1.4: a pristine v3 file (the current stream stripped of the
        // v4 tail (2 bytes) and the v5 tail (1 presence byte — the world has
        // no bed spawn), the version field restored) loads with both
        // cooldowns at 0.
        let mut w = World::new("Test".into(), 32, 32);
        w.sundial_cooldown = 7;
        let mut buf = Vec::new();
        write_world(&mut buf, &w).unwrap();
        buf.truncate(buf.len() - 3);
        buf[8..12].copy_from_slice(&3u32.to_le_bytes());
        let back = parse(&buf).unwrap();
        assert_eq!(back.sundial_cooldown, 0);
        assert_eq!(back.moondial_cooldown, 0);
    }

    #[test]
    fn dw_1_4_version_above_5_rejected() {
        // DW-1.4 (dirty): the existing loader rejects a version above the
        // current one (`save.rs` version gate). v6 (the secret-seed flags)
        // is now the current format, so the gate pins 7.
        let w = World::new("Test".into(), 32, 32);
        let mut buf = Vec::new();
        write_world(&mut buf, &w).unwrap();
        buf[8..12].copy_from_slice(&7u32.to_le_bytes());
        assert!(parse(&buf).is_err(), "version 7 is unsupported");
    }

    #[test]
    fn m5_seed_1_the_secret_seed_flags_round_trip_in_v6() {
        // M5-SEED-1: the v6 tail carries the classic secret-seed flags
        // (`WorldFile.SaveWorldFlags`'s nine booleans as the port's
        // `serializedSeedsSum` bitmask); a classic world keeps all bits 0
        // and a seed world survives the save/load.
        let mut classic = World::new("Test".into(), 32, 32);
        let mut buf = Vec::new();
        write_world(&mut buf, &classic).unwrap();
        assert_eq!(back_flags(&buf), 0, "a classic world writes no seed bits");
        let back = parse(&buf).unwrap();
        assert_eq!(back.seeds, crate::seeds::SeedFlags::none());
        classic.seeds = crate::seeds::SeedFlags::from_seed_text("for the worthy");
        buf.clear();
        write_world(&mut buf, &mut classic).unwrap();
        assert_eq!(
            back_flags(&buf),
            crate::seeds::SeedFlags::FOR_THE_WORTHY
        );
        let back = parse(&buf).unwrap();
        assert_eq!(back.seeds, classic.seeds);
    }

    /// The v6 tail's u16 bitmask, read back out of the written buffer.
    fn back_flags(buf: &[u8]) -> u16 {
        u16::from_le_bytes([buf[buf.len() - 2], buf[buf.len() - 1]])
    }

    #[test]
    fn the_bed_spawn_round_trips_in_v5() {
        // DW-3.1 — the v5 tail field: `World::bed_spawn` survives the save
        // (both arms), and a `None` world stays `None`.
        let mut w = World::new("Test".into(), 64, 48);
        w.bed_spawn = Some((30, 20));
        let mut buf = Vec::new();
        write_world(&mut buf, &w).unwrap();
        let back = parse(&buf).unwrap();
        assert_eq!(back.bed_spawn, Some((30, 20)));
        w.bed_spawn = None;
        buf.clear();
        write_world(&mut buf, &w).unwrap();
        assert_eq!(parse(&buf).unwrap().bed_spawn, None);
    }

    #[test]
    fn a_v4_payload_still_loads_without_the_bed_spawn() {
        // DW-3.1 — the back-compat read: the v5 payload with the version
        // field patched to 4 parses as v4 — the tail sits unread and the bed
        // spawn degrades to `None`.
        let mut w = World::new("Test".into(), 64, 48);
        w.spawn_x = 32;
        w.spawn_y = 20;
        w.bed_spawn = Some((30, 22));
        let mut buf = Vec::new();
        write_world(&mut buf, &w).unwrap();
        buf[8..12].copy_from_slice(&4u32.to_le_bytes());
        let back = parse(&buf).unwrap();
        assert_eq!(back.bed_spawn, None);
        assert_eq!((back.spawn_x, back.spawn_y), (32, 20));
    }

    #[test]
    fn a_corrupt_bed_spawn_degrades_to_none() {
        // DW-3.1 (dirty, the deserialization trust boundary): an
        // out-of-range bed cell degrades to `None` instead of erroring or
        // poisoning the spawn (`Player.cs:37571-37575`'s fallback arm).
        let mut w = World::new("Test".into(), 64, 48);
        w.bed_spawn = Some((30, 20));
        let mut buf = Vec::new();
        write_world(&mut buf, &w).unwrap();
        // The v5 tail is the last 9 bytes: [presence][x: i32][y: i32] —
        // poison the x.
        let tail = buf.len() - 9;
        buf[tail + 1..tail + 5].copy_from_slice(&99999i32.to_le_bytes());
        assert_eq!(parse(&buf).unwrap().bed_spawn, None);
    }
}
