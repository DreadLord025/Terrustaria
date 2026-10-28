//! DXT1/3/5 block decompression to RGBA8.
//! Port of `tools/XnbExtract/src/DtxUtil.cs` (FNA/MonoGame `DxtUtil`).

use anyhow::{anyhow, Result};

/// Local big-... little-endian reader over the compressed mip data
/// (stands in for the `BinaryReader` DxtUtil wraps around the block stream).
struct DxtReader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> DxtReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }
    fn read_u8(&mut self) -> Result<u8> {
        let b = *self.data.get(self.pos).ok_or_else(|| anyhow!("short DXT block data"))?;
        self.pos += 1;
        Ok(b)
    }
    fn read_u16(&mut self) -> Result<u16> {
        let lo = self.read_u8()? as u16;
        let hi = self.read_u16_high()?;
        Ok(lo | hi << 8)
    }
    fn read_u16_high(&mut self) -> Result<u16> {
        let b = self.read_u8()? as u16;
        Ok(b)
    }
    fn read_u32(&mut self) -> Result<u32> {
        let mut v = 0u32;
        for i in 0..4 {
            v |= (self.read_u8()? as u32) << (8 * i);
        }
        Ok(v)
    }
}

// DxtUtil.ConvertRgb565ToRgb888
fn rgb565_to_rgb888(color: u16) -> (u8, u8, u8) {
    let temp = ((color >> 11) as u32) * 255 + 16;
    let r = ((temp / 32 + temp) / 32) as u8;
    let temp = (((color & 0x07E0) >> 5) as u32) * 255 + 32;
    let g = ((temp / 64 + temp) / 64) as u8;
    let temp = ((color & 0x001F) as u32) * 255 + 16;
    let b = ((temp / 32 + temp) / 32) as u8;
    (r, g, b)
}

fn block_counts(width: i32, height: i32) -> (i32, i32) {
    ((width + 3) / 4, (height + 3) / 4)
}

fn check_block_bytes(data_len: usize, width: i32, height: i32, bytes_per_block: usize) -> Result<()> {
    let (bx, by) = block_counts(width, height);
    let need = bx as usize * by as usize * bytes_per_block;
    if data_len < need {
        bail_short(data_len, need);
    }
    Ok(())
}

fn bail_short(have: usize, need: usize) -> ! {
    panic!("short DXT data: have {have}, need {need}")
}

// DxtUtil.DecompressDxt1
pub(crate) fn decompress_dxt1(data: &[u8], width: i32, height: i32) -> Result<Vec<u8>> {
    check_block_bytes(data.len(), width, height, 8)?;
    let mut out = vec![0u8; (width as usize) * (height as usize) * 4];
    let mut r = DxtReader::new(data);
    let (block_count_x, block_count_y) = block_counts(width, height);
    for y in 0..block_count_y {
        for x in 0..block_count_x {
            dxt1_block(&mut r, x, y, width, height, &mut out)?;
        }
    }
    Ok(out)
}

// DxtUtil.DecompressDxt3
pub(crate) fn decompress_dxt3(data: &[u8], width: i32, height: i32) -> Result<Vec<u8>> {
    check_block_bytes(data.len(), width, height, 16)?;
    let mut out = vec![0u8; (width as usize) * (height as usize) * 4];
    let mut r = DxtReader::new(data);
    let (block_count_x, block_count_y) = block_counts(width, height);
    for y in 0..block_count_y {
        for x in 0..block_count_x {
            dxt3_block(&mut r, x, y, width, height, &mut out)?;
        }
    }
    Ok(out)
}

// DxtUtil.DecompressDxt5
pub(crate) fn decompress_dxt5(data: &[u8], width: i32, height: i32) -> Result<Vec<u8>> {
    check_block_bytes(data.len(), width, height, 16)?;
    let mut out = vec![0u8; (width as usize) * (height as usize) * 4];
    let mut r = DxtReader::new(data);
    let (block_count_x, block_count_y) = block_counts(width, height);
    for y in 0..block_count_y {
        for x in 0..block_count_x {
            dxt5_block(&mut r, x, y, width, height, &mut out)?;
        }
    }
    Ok(out)
}

fn put(out: &mut [u8], x: i32, y: i32, width: i32, height: i32, px: i32, py: i32, r: u8, g: u8, b: u8, a: u8) {
    let _ = (x, y);
    if px < width && py < height {
        let offset = (((py * width) + px) << 2) as usize;
        out[offset] = r;
        out[offset + 1] = g;
        out[offset + 2] = b;
        out[offset + 3] = a;
    }
}

// DxtUtil.DecompressDxt1Block
fn dxt1_block(r: &mut DxtReader, x: i32, y: i32, width: i32, height: i32, out: &mut [u8]) -> Result<()> {
    let c0 = r.read_u16()?;
    let c1 = r.read_u16()?;
    let (r0, g0, b0) = rgb565_to_rgb888(c0);
    let (r1, g1, b1) = rgb565_to_rgb888(c1);
    let lookup_table = r.read_u32()?;

    for block_y in 0..4 {
        for block_x in 0..4 {
            let index = (lookup_table >> (2 * (4 * block_y + block_x))) & 0x03;
            let (rr, gg, bb, aa) = if c0 > c1 {
                match index {
                    0 => (r0, g0, b0, 255),
                    1 => (r1, g1, b1, 255),
                    2 => (
                        ((2 * r0 as u32 + r1 as u32) / 3) as u8,
                        ((2 * g0 as u32 + g1 as u32) / 3) as u8,
                        ((2 * b0 as u32 + b1 as u32) / 3) as u8,
                        255,
                    ),
                    _ => (
                        ((r0 as u32 + 2 * r1 as u32) / 3) as u8,
                        ((g0 as u32 + 2 * g1 as u32) / 3) as u8,
                        ((b0 as u32 + 2 * b1 as u32) / 3) as u8,
                        255,
                    ),
                }
            } else {
                match index {
                    0 => (r0, g0, b0, 255),
                    1 => (r1, g1, b1, 255),
                    2 => (
                        ((r0 as u32 + r1 as u32) / 2) as u8,
                        ((g0 as u32 + g1 as u32) / 2) as u8,
                        ((b0 as u32 + b1 as u32) / 2) as u8,
                        255,
                    ),
                    _ => (0, 0, 0, 0),
                }
            };
            put(out, x, y, width, height, (x << 2) + block_x, (y << 2) + block_y, rr, gg, bb, aa);
        }
    }
    Ok(())
}

// DxtUtil.DecompressDxt3Block
fn dxt3_block(r: &mut DxtReader, x: i32, y: i32, width: i32, height: i32, out: &mut [u8]) -> Result<()> {
    let mut alpha = [0u8; 8];
    for a in &mut alpha {
        *a = r.read_u8()?;
    }
    let c0 = r.read_u16()?;
    let c1 = r.read_u16()?;
    let (r0, g0, b0) = rgb565_to_rgb888(c0);
    let (r1, g1, b1) = rgb565_to_rgb888(c1);
    let lookup_table = r.read_u32()?;

    let mut alpha_index = 0usize;
    for block_y in 0..4 {
        for block_x in 0..4 {
            let index = (lookup_table >> (2 * (4 * block_y + block_x))) & 0x03;
            // 4-bit alpha per pixel, expanded to 8 bits by nibble replication
            let nibble_byte = alpha[alpha_index / 2];
            let a = if alpha_index % 2 == 0 {
                let n = nibble_byte & 0x0F;
                n | (n << 4)
            } else {
                let n = nibble_byte & 0xF0;
                n | (n >> 4)
            };
            alpha_index += 1;

            let (rr, gg, bb) = match index {
                0 => (r0, g0, b0),
                1 => (r1, g1, b1),
                2 => (((2 * r0 as u32 + r1 as u32) / 3) as u8, ((2 * g0 as u32 + g1 as u32) / 3) as u8, ((2 * b0 as u32 + b1 as u32) / 3) as u8),
                _ => (((r0 as u32 + 2 * r1 as u32) / 3) as u8, ((g0 as u32 + 2 * g1 as u32) / 3) as u8, ((b0 as u32 + 2 * b1 as u32) / 3) as u8),
            };
            put(out, x, y, width, height, (x << 2) + block_x, (y << 2) + block_y, rr, gg, bb, a);
        }
    }
    Ok(())
}

// DxtUtil.DecompressDxt5Block
fn dxt5_block(r: &mut DxtReader, x: i32, y: i32, width: i32, height: i32, out: &mut [u8]) -> Result<()> {
    let alpha0 = r.read_u8()?;
    let alpha1 = r.read_u8()?;
    let mut alpha_mask: u64 = 0;
    for i in 0..6 {
        alpha_mask += (r.read_u8()? as u64) << (8 * i);
    }
    let c0 = r.read_u16()?;
    let c1 = r.read_u16()?;
    let (r0, g0, b0) = rgb565_to_rgb888(c0);
    let (r1, g1, b1) = rgb565_to_rgb888(c1);
    let lookup_table = r.read_u32()?;

    for block_y in 0..4 {
        for block_x in 0..4 {
            let index = (lookup_table >> (2 * (4 * block_y + block_x))) & 0x03;
            let alpha_index = ((alpha_mask >> (3 * (4 * block_y + block_x))) & 0x07) as u32;
            let a: u8 = if alpha_index == 0 {
                alpha0
            } else if alpha_index == 1 {
                alpha1
            } else if alpha0 > alpha1 {
                (((8 - alpha_index) * alpha0 as u32 + (alpha_index - 1) * alpha1 as u32) / 7) as u8
            } else if alpha_index == 6 {
                0
            } else if alpha_index == 7 {
                0xff
            } else {
                (((6 - alpha_index) * alpha0 as u32 + (alpha_index - 1) * alpha1 as u32) / 5) as u8
            };

            let (rr, gg, bb) = match index {
                0 => (r0, g0, b0),
                1 => (r1, g1, b1),
                2 => (((2 * r0 as u32 + r1 as u32) / 3) as u8, ((2 * g0 as u32 + g1 as u32) / 3) as u8, ((2 * b0 as u32 + b1 as u32) / 3) as u8),
                _ => (((r0 as u32 + 2 * r1 as u32) / 3) as u8, ((g0 as u32 + 2 * g1 as u32) / 3) as u8, ((b0 as u32 + 2 * b1 as u32) / 3) as u8),
            };
            put(out, x, y, width, height, (x << 2) + block_x, (y << 2) + block_y, rr, gg, bb, a);
        }
    }
    Ok(())
}
