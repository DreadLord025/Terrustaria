//! Section 3 (pointer 2): tiles — `SaveWorldTiles` (`WorldFile.cs:1432-1604`),
//! the column-major RLE encoder and the exact inverse of the
//! reader `load_tiles` (`WorldFile.cs:2519-2721`); the batching equality is
//! `Tile.isTheSameAs` (`Tile.cs:155-197`) and its gate is
//! `TileID.Sets.AllowsSaveCompressionBatching` (`TileID.cs:484`).
//!
//! Each column is written top to bottom; one record covers one tile plus the
//! run of identical tiles below it. A record is the four chained header bytes
//! (each present only when the byte below sets bit 0 of the one before it)
//! followed by the payload, with the RLE count last. The four wire colours
//! ride the headers exactly as in the C# — `wire()`/`wire2()`/`wire3()` are
//! `sTileHeader` bits (`Tile.cs:623-672`) and `wire4()` is a `bTileHeader`
//! bit (`Tile.cs:437-452`) — and the port reads them from its sparse
//! `World.wires` table, where the C# keeps them inside the tile.

use crate::binio::Writer;
use crate::tile::{
    Tile, FLAG_ACTIVE, FLAG_ACTUATOR, FLAG_FULLBRIGHT, FLAG_HALF_BRICK, FLAG_INACTIVE,
    FLAG_LIQUID_HONEY, FLAG_LIQUID_LAVA, FLAG_LIQUID_SHIMMER,
};
use crate::tiledata::SET_ALLOWS_SAVE_COMPRESSION_BATCHING;
use crate::wiring::WireColor;
use crate::world::World;

/// The four wire colours of one tile — `Tile.wire()`/`wire2()`/`wire3()`
/// (`sTileHeader` bits 0x80/0x100/0x200, `Tile.cs:623-672`) and `wire4()`
/// (`bTileHeader` bit 0x80, `Tile.cs:437-452`). In the C# they ride the
/// tile's own headers; the port keeps them in the sparse `World.wires` table.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WireBits {
    pub red: bool,
    pub blue: bool,
    pub green: bool,
    pub yellow: bool,
}

impl WireBits {
    /// The four colours of the tile at (x, y) as the world's wire table holds
    /// them.
    pub fn at(world: &World, x: i32, y: i32) -> Self {
        Self {
            red: world.wires.get(x, y, WireColor::Red),
            blue: world.wires.get(x, y, WireColor::Blue),
            green: world.wires.get(x, y, WireColor::Green),
            yellow: world.wires.get(x, y, WireColor::Yellow),
        }
    }

    /// Writes the colours into the world's wire table at (x, y) — the
    /// reader's half (`LoadWorldTiles:2614-2632`/`:2651-2654`).
    pub fn apply_to(&self, world: &mut World, x: i32, y: i32) {
        for (on, color) in [
            (self.red, WireColor::Red),
            (self.blue, WireColor::Blue),
            (self.green, WireColor::Green),
            (self.yellow, WireColor::Yellow),
        ] {
            if on {
                world.wires.set(x, y, color, true);
            }
        }
    }
}

pub fn implemented() -> bool {
    true
}

/// Ports `SaveWorldTiles:1432-1604`: one record per (tile, run) pair, columns
/// in order, RLE counts one or two bytes.
pub fn write(w: &mut Writer, world: &World) {
    for x in 0..world.width {
        let mut y = 0;
        while y < world.height {
            let idx = world.index(x, y);
            let tile = world.tiles[idx];
            let extras = world.extras_at(idx);
            y += write_tile_record(w, world, x, y, tile, extras);
        }
    }
}

/// Writes one tile's record (header chain + payload + RLE count) and returns
/// how many rows it covers: the tile itself plus its identical run below
/// (`SaveWorldTiles:1568-1580`'s `while` loop advancing `num2`).
fn write_tile_record(w: &mut Writer, world: &World, x: i32, y: i32, tile: Tile, extras: u8) -> i32 {
    let mut payload: Vec<u8> = Vec::with_capacity(16);
    // `b` (writer's first header) / `b2` / `b3` / `b4` follow the C# names; the
    // reader calls them b4/b1/b2/b3 in file order.
    let mut b = 0u8;
    let mut b2 = 0u8;
    let mut b3 = 0u8;
    let mut b4 = 0u8;

    // WorldFile.cs:1449-1481: the active tile — type (16-bit when above
    // 255), the frames when the kind is important, the tile paint.
    if tile.flags & FLAG_ACTIVE != 0 {
        b |= 2;
        payload.push(tile.kind as u8);
        if tile.kind > 255 {
            b |= 0x20;
            payload.push((tile.kind >> 8) as u8);
        }
        if frame_important(world, tile.kind) {
            payload.extend_from_slice(&tile.frame_x.to_le_bytes());
            payload.extend_from_slice(&tile.frame_y.to_le_bytes());
        }
        if tile.paint() != 0 {
            b3 |= 8;
            payload.push(tile.paint());
        }
    }

    // WorldFile.cs:1482-1493: the wall, plus the wall paint from the sparse
    // extras table.
    if tile.wall != 0 {
        b |= 4;
        payload.push(tile.wall as u8);
        let wall_paint = extras & 0x1f;
        if wall_paint != 0 {
            b3 |= 0x10;
            payload.push(wall_paint);
        }
    }

    // WorldFile.cs:1494-1507: the liquid amount and its kind. The two liquid
    // bits are a kind code, not independent flags: water is the plain `b |= 8`
    // (code 1), lava `b |= 0x10` (code 2), honey `b |= 0x18` (code 3), and
    // shimmer the water encoding plus `b3 |= 0x80`.
    if tile.liquid != 0 {
        if tile.flags & FLAG_LIQUID_SHIMMER != 0 {
            b3 |= 0x80;
            b |= 8;
        } else if tile.flags & FLAG_LIQUID_LAVA != 0 {
            b |= 0x10;
        } else if tile.flags & FLAG_LIQUID_HONEY != 0 {
            b |= 0x18;
        } else {
            b |= 8;
        }
        payload.push(tile.liquid);
    }

    // WorldFile.cs:1508-1519: the three `sTileHeader` wire colours —
    // `wire()` 0x02, `wire2()` 0x04, `wire3()` 0x08.
    let wires = WireBits::at(world, x, y);
    if wires.red {
        b2 |= 0x02;
    }
    if wires.blue {
        b2 |= 0x04;
    }
    if wires.green {
        b2 |= 0x08;
    }
    // WorldFile.cs:1520: the slope/half-brick field rides b2.
    if tile.flags & FLAG_HALF_BRICK != 0 {
        b2 |= 0x10;
    } else if tile.slope() != 0 {
        // C# is `tile.slope() + 1 << 4` — `(slope + 1) << 4`.
        b2 |= (tile.slope() + 1) << 4;
    }
    // WorldFile.cs:1521-1528: the actuator and the inactive flag ride b3.
    if tile.flags & FLAG_ACTUATOR != 0 {
        b3 |= 2;
    }
    if tile.flags & FLAG_INACTIVE != 0 {
        b3 |= 4;
    }
    // WorldFile.cs:1529-1532: the yellow wire — `wire4()`, a `bTileHeader`
    // bit — rides b3.
    if wires.yellow {
        b3 |= 0x20;
    }
    // WorldFile.cs:1533-1538: the wall's high byte comes **after** the liquid
    // byte — the reader reads it in its `b2 > 1` block, which follows the
    // liquid block.
    if tile.wall > 255 {
        b3 |= 0x40;
        payload.push((tile.wall >> 8) as u8);
    }

    // WorldFile.cs:1539-1554: the four coatings — three from the sparse
    // extras table, the tile fullbright from the packed flags.
    if extras & 0x20 != 0 {
        b4 |= 2; // invisibleBlock
    }
    if extras & 0x40 != 0 {
        b4 |= 4; // invisibleWall
    }
    if tile.flags & FLAG_FULLBRIGHT != 0 {
        b4 |= 8; // fullbrightBlock
    }
    if extras & 0x80 != 0 {
        b4 |= 0x10; // fullbrightWall
    }

    // WorldFile.cs:1574-1598: the run length — the next tiles down the
    // column while they encode identically **and** the kind allows batching,
    // never past the column's end (`num7 > 0`).
    let mut run = 0i32;
    let mut next = y + 1;
    let mut remaining = world.height - y - 1;
    while remaining > 0 && allows_batching(tile.kind) {
        let next_idx = world.index(x, next);
        let next_tile = world.tiles[next_idx];
        let next_extras = world.extras_at(next_idx);
        let next_wires = WireBits::at(world, x, next);
        if !encoded_same(
            world,
            &tile,
            extras,
            &wires,
            &next_tile,
            next_extras,
            &next_wires,
        ) {
            break;
        }
        run += 1;
        remaining -= 1;
        next += 1;
    }

    // WorldFile.cs:1586-1597: the RLE count is the payload's tail — one
    // byte when the run fits, low-then-high bytes when it does not.
    if run > 0 {
        payload.push(run as u8);
        if run > 255 {
            b |= 0x80;
            payload.push((run >> 8) as u8);
        } else {
            b |= 0x40;
        }
    }

    // WorldFile.cs:1555-1573: the header chain is filled backwards in the
    // C# (from index 3 down), so the file order is b, then b2/b3/b4 while each
    // carries the previous one's "present" bit.
    if b4 != 0 {
        b3 |= 1;
    }
    if b3 != 0 {
        b2 |= 1;
    }
    if b2 != 0 {
        b |= 1;
    }
    let mut record = Vec::with_capacity(payload.len() + 4);
    record.push(b);
    if b & 1 != 0 {
        record.push(b2);
    }
    if b2 & 1 != 0 {
        record.push(b3);
    }
    if b3 & 1 != 0 {
        record.push(b4);
    }
    record.append(&mut payload);
    w.bytes(&record);

    run + 1
}

/// `Main.tileFrameImportant[tile.type]` — the world's own bitmap; a kind past
/// the table's end is not important.
fn frame_important(world: &World, kind: u16) -> bool {
    world
        .frame_important
        .get(kind as usize)
        .copied()
        .unwrap_or(false)
}

/// `TileID.Sets.AllowsSaveCompressionBatching[tile.type]` (`TileID.cs:484`):
/// everything batches except 423/520/723/724.
fn allows_batching(kind: u16) -> bool {
    SET_ALLOWS_SAVE_COMPRESSION_BATCHING.has(kind)
}

/// `Tile.isTheSameAs:155-197` over the fields this writer encodes — the RLE
/// batching equality. The C# compares the packed `sTileHeader` (paint, active,
/// inactive, the three `sTileHeader` wires, half-brick, actuator, slope,
/// fullbright-wall) first and the `bTileHeader` (wall paint, liquid kind, the
/// yellow wire) after it; the port reads the same fields from its own layout.
fn encoded_same(
    world: &World,
    a: &Tile,
    a_extras: u8,
    a_wires: &WireBits,
    b: &Tile,
    b_extras: u8,
    b_wires: &WireBits,
) -> bool {
    // `sTileHeader`: paint + slope packed in `meta`, the active/inactive/
    // half-brick/actuator flags, fullbright-wall in the extras.
    if a.meta != b.meta {
        return false;
    }
    const HEADER_FLAGS: u8 = FLAG_ACTIVE | FLAG_INACTIVE | FLAG_HALF_BRICK | FLAG_ACTUATOR;
    if a.flags & HEADER_FLAGS != b.flags & HEADER_FLAGS {
        return false;
    }
    // The three `sTileHeader` wire colours (`Tile.cs:623-672`) ride the same
    // header, so they break the batch just like a slope does.
    if a_wires.red != b_wires.red || a_wires.blue != b_wires.blue || a_wires.green != b_wires.green
    {
        return false;
    }
    if a.flags & FLAG_ACTIVE != 0 {
        if a.kind != b.kind {
            return false;
        }
        if frame_important(world, a.kind) && (a.frame_x != b.frame_x || a.frame_y != b.frame_y) {
            return false;
        }
    }
    if a.wall != b.wall || a.liquid != b.liquid {
        return false;
    }
    // `bTileHeader` (wall paint + liquid kind + wire4) is compared whole when
    // the tile carries liquid; without liquid only the wall paint is.
    if a.liquid != 0 {
        const LIQUID_KIND: u8 = FLAG_LIQUID_LAVA | FLAG_LIQUID_HONEY | FLAG_LIQUID_SHIMMER;
        if a.flags & LIQUID_KIND != b.flags & LIQUID_KIND {
            return false;
        }
    }
    if a_extras & 0x1f != b_extras & 0x1f {
        return false; // wallColor
    }
    // The yellow wire — `wire4()`, a `bTileHeader` bit (`Tile.cs:437-452`) —
    // is compared in both C# branches (`:186` without liquid, `:191` with it).
    if a_wires.yellow != b_wires.yellow {
        return false;
    }
    // The four coatings, unconditional in the C#.
    if a_extras & 0xe0 != b_extras & 0xe0 {
        return false; // invisibleBlock | invisibleWall | fullbrightWall
    }
    if a.flags & FLAG_FULLBRIGHT != b.flags & FLAG_FULLBRIGHT {
        return false; // fullbrightBlock
    }
    true
}
