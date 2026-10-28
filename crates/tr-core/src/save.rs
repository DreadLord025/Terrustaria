//! Native save format (`TRWLD`) for generated worlds.
//!
//! Deliberately tiny and lossless for everything the renderer needs: header fields,
//! the frame-importance bitset and the full 14-byte tile struct, RLE-compressed per
//! row. The original `.wld` writer is a much larger job; this format exists so the
//! generator can persist a world under `Documents/My Games/Terrustaria`.

use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use crate::tile::Tile;
use crate::world::World;

const MAGIC: &[u8; 8] = b"TRWLD01\0";
const VERSION: u32 = 1;

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
    let mut r = Reader { data, pos: 0 };
    if r.take(8)? != MAGIC {
        bail!("not a TRWLD save");
    }
    let version = r.u32()?;
    if version != VERSION {
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

struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        if self.pos + n > self.data.len() {
            bail!("unexpected end of file at offset {}", self.pos);
        }
        let s = &self.data[self.pos..self.pos + n];
        self.pos += n;
        Ok(s)
    }

    fn u8(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }

    fn i16(&mut self) -> Result<i16> {
        Ok(i16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }

    fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    fn i32(&mut self) -> Result<i32> {
        Ok(i32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    fn u64(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }

    fn f64(&mut self) -> Result<f64> {
        Ok(f64::from_le_bytes(self.take(8)?.try_into().unwrap()))
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
        assert!(back.frame_important[5]);
        assert_eq!(back.tiles, w.tiles);
    }
}
