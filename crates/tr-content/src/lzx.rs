//! Xbox XMemCompress (LZX) decoder: faithful port of
//! `tools/XnbExtract/src/LzxDecoder.cs` (+ `LzxBuffer.cs`, `HuffTable.cs`),
//! 16-bit window, headered blocks. C# symbol names are referenced inline.

use anyhow::{anyhow, bail, Result};

// LzxDecoder constants
const MIN_MATCH: i32 = 2;
const NUM_CHARS: usize = 256;
const PRETREE_NUM_ELEMENTS: usize = 20;
const ALIGNED_NUM_ELEMENTS: usize = 8;
const NUM_PRIMARY_LENGTHS: i32 = 7;
const NUM_SECONDARY_LENGTHS: usize = 249;

const PRETREE_MAX_SYMBOLS: usize = PRETREE_NUM_ELEMENTS;
const PRETREE_TABLE_BITS: usize = 6;
const MAINTREE_MAX_SYMBOLS: usize = NUM_CHARS + 50 * 8;
const MAINTREE_TABLE_BITS: usize = 12;
const LENGTH_MAX_SYMBOLS: usize = NUM_SECONDARY_LENGTHS + 1;
const LENGTH_TABLE_BITS: usize = 12;
const ALIGNED_MAX_SYMBOLS: usize = ALIGNED_NUM_ELEMENTS;
const ALIGNED_TABLE_BITS: usize = 7;

// LzxDecoder.extraBits (static ctor)
fn extra_bits() -> [i32; 52] {
    let mut eb = [0i32; 52];
    let mut j = 0;
    for i in (0..=50).step_by(2) {
        eb[i] = j;
        eb[i + 1] = j;
        if i != 0 && j < 17 {
            j += 1;
        }
    }
    eb
}

// LzxDecoder.positionBase (static ctor)
fn position_base() -> [i32; 51] {
    let eb = extra_bits();
    let mut pb = [0i32; 51];
    let mut j = 0;
    for i in 0..=50 {
        pb[i] = j;
        j += 1 << eb[i];
    }
    pb
}

// LzxDecoder.LzxBlockType
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum LzxBlockType {
    Invalid = 0,
    Verbatim = 1,
    Aligned = 2,
    Uncompressed = 3,
}

/// Seekable byte input over a slice. Stands in for C#'s `BinaryReader` over a
/// `MemoryStream`; reads past the end yield 0 (they only occur on corrupt
/// data, where the C# tool would throw `EndOfStreamException`).
pub(crate) struct LzxInput<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> LzxInput<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }
    fn position(&self) -> usize {
        self.pos
    }
    fn set_position(&mut self, pos: usize) {
        self.pos = pos;
    }
    fn read_byte(&mut self) -> u8 {
        let b = self.data.get(self.pos).copied().unwrap_or(0);
        self.pos = (self.pos + 1).min(self.data.len());
        b
    }
    // BinaryReader.Read(buffer, index, count)
    fn read_into(&mut self, dst: &mut [u8]) {
        let n = dst.len().min(self.data.len().saturating_sub(self.pos));
        dst[..n].copy_from_slice(&self.data[self.pos..self.pos + n]);
        for b in &mut dst[n..] {
            *b = 0;
        }
        self.pos += n;
    }
}

/// LzxBuffer: MSB-first bit reader fed with little-endian 16-bit words.
struct LzxBuffer<'b, 'a> {
    input: &'b mut LzxInput<'a>,
    remaining_bits: i32,
    bit_buffer: u32,
}

impl<'b, 'a> LzxBuffer<'b, 'a> {
    fn new(input: &'b mut LzxInput<'a>) -> Self {
        Self { input, remaining_bits: 0, bit_buffer: 0 }
    }
    fn reset(&mut self) {
        self.remaining_bits = 0;
        self.bit_buffer = 0;
    }
    // LzxBuffer.EnsureBits; the C# int shift masks its count to 5 bits, so a
    // negative shift here behaves like wrapping_shl.
    fn ensure_bits(&mut self, bit_count: i32) -> Result<()> {
        if !(0..=32).contains(&bit_count) {
            bail!("LzxBuffer.EnsureBits: bad bit count {bit_count}");
        }
        while self.remaining_bits < bit_count {
            let lo = self.input.read_byte() as u32;
            let hi = self.input.read_byte() as u32;
            self.bit_buffer |= ((hi << 8) | lo).wrapping_shl((16 - self.remaining_bits) as u32);
            self.remaining_bits += 16;
        }
        Ok(())
    }
    fn peek_bits(&self, bit_count: i32) -> Result<u32> {
        if bit_count > self.remaining_bits {
            bail!("LzxBuffer.PeekBits: not enough bits: required {bit_count} has {}", self.remaining_bits);
        }
        Ok(self.bit_buffer >> (32 - bit_count))
    }
    fn remove_bits(&mut self, bit_count: i32) {
        self.bit_buffer = self.bit_buffer.wrapping_shl(bit_count as u32);
        self.remaining_bits -= bit_count;
    }
    fn read_bits(&mut self, bit_count: i32) -> Result<u32> {
        if bit_count > 0 {
            self.ensure_bits(bit_count)?;
            let result = self.peek_bits(bit_count)?;
            self.remove_bits(bit_count);
            Ok(result)
        } else {
            Ok(0)
        }
    }
    // direct (non-bit) access to the underlying stream, as C# uses `input.*`
    fn raw_read_byte(&mut self) -> u8 {
        self.input.read_byte()
    }
    fn raw_read_into(&mut self, dst: &mut [u8]) {
        self.input.read_into(dst);
    }
    fn position(&self) -> usize {
        self.input.position()
    }
    fn set_position(&mut self, pos: usize) {
        self.input.set_position(pos);
    }
}

/// HuffTable (tools/XnbExtract/src/HuffTable.cs).
struct HuffTable {
    table: Vec<i16>,
    length: Vec<u8>,
    max_symbols: usize,
    table_bits: usize,
}

const SAFETY: usize = 64; // HuffTable.Safety

impl HuffTable {
    fn new(max_symbols: usize, table_bits: usize) -> Self {
        Self {
            table: vec![0i16; (1 << table_bits) + (max_symbols << 1)],
            length: vec![0u8; max_symbols + SAFETY],
            max_symbols,
            table_bits,
        }
    }

    fn reset(&mut self) {
        self.table.fill(0);
        self.length.fill(0);
    }

    // HuffTable.MakeDecodeTable (liblzx make_decode_table); the C# version
    // silently returns on table overrun, which we reproduce with early returns.
    fn make_decode_table(&mut self) -> Result<()> {
        let mut bit_num: i32 = 1;
        let mut pos: i32 = 0;
        let mut table_mask: i32 = 1 << self.table_bits as i32;
        let mut bit_mask: i32 = ((table_mask as u32) >> 1) as i32; // don't do 0 length codes
        let mut next_symbol = bit_mask; // base of allocation for long codes
        let max_symbols = self.max_symbols as i32;

        // fill entries for codes short enough for a direct mapping
        while bit_num <= self.table_bits as i32 {
            for sym in 0..max_symbols {
                if self.length[sym as usize] == bit_num as u8 {
                    let mut leaf = pos;
                    pos += bit_mask;
                    if pos > table_mask {
                        return Ok(()); // table overrun
                    }
                    let mut fill = bit_mask;
                    while fill > 0 {
                        fill -= 1;
                        self.table[leaf as usize] = sym as i16;
                        leaf += 1;
                    }
                }
            }
            bit_mask = ((bit_mask as u32) >> 1) as i32;
            bit_num += 1;
        }

        // if there are any codes longer than tableBits
        if pos != table_mask {
            for sym in pos..table_mask {
                self.table[sym as usize] = 0;
            }
            // give ourselves room for codes to grow by up to 16 more bits
            pos <<= 16;
            table_mask <<= 16;
            bit_mask = 1 << 15;

            while bit_num <= 16 {
                for sym in 0..max_symbols {
                    if self.length[sym as usize] == bit_num as u8 {
                        let mut leaf = ((pos as u32) >> 16) as i32;
                        for fill in 0..(bit_num - self.table_bits as i32) {
                            // if this path hasn't been taken yet, 'allocate' two entries
                            if self.get(leaf)? == 0 {
                                let base = (next_symbol << 1) as usize;
                                if base + 1 >= self.table.len() {
                                    bail!("MakeDecodeTable: subtable overflow");
                                }
                                self.table[base] = 0;
                                self.table[base + 1] = 0;
                                *self.mut_entry(leaf)? = next_symbol as i16;
                                next_symbol += 1;
                            }
                            // follow the path and select either left or right for next bit
                            leaf = (self.get(leaf)? as i32) << 1;
                            if (((pos as u32) >> (15 - fill)) & 1) == 1 {
                                leaf += 1;
                            }
                        }
                        *self.mut_entry(leaf)? = sym as i16;
                        pos += bit_mask;
                        if pos > table_mask {
                            return Ok(());
                        }
                    }
                }
                bit_mask = ((bit_mask as u32) >> 1) as i32;
                bit_num += 1;
            }
        }
        Ok(()) // C# then only checks for a full/erroneous table, no side effects
    }

    fn get(&self, idx: i32) -> Result<i16> {
        if idx < 0 {
            bail!("HuffTable: negative index {idx}");
        }
        self.table
            .get(idx as usize)
            .copied()
            .ok_or_else(|| anyhow!("HuffTable: index {idx} out of range"))
    }

    fn mut_entry(&mut self, idx: i32) -> Result<&mut i16> {
        if idx < 0 {
            bail!("HuffTable: negative index {idx}");
        }
        self.table
            .get_mut(idx as usize)
            .ok_or_else(|| anyhow!("HuffTable: index {idx} out of range"))
    }

    // HuffTable.ReadHuffSym
    fn read_huff_sym(&self, buffer: &mut LzxBuffer) -> Result<i32> {
        buffer.ensure_bits(16)?;
        let mut i = self.get(buffer.peek_bits(self.table_bits as i32)? as i32)? as i32;
        let max_symbols = self.max_symbols as i32;
        if i >= max_symbols {
            let mut j: u32 = 1 << (32 - self.table_bits as u32);
            loop {
                j >>= 1;
                i = (i << 1) | if (buffer.bit_buffer & j) != 0 { 1 } else { 0 };
                if j == 0 {
                    bail!("ReadHuffSym: decode error");
                }
                i = self.get(i)? as i32;
                if i < max_symbols {
                    break;
                }
            }
        }
        let len = self.length.get(i as usize).copied().ok_or_else(|| anyhow!("ReadHuffSym: bad symbol {i}"))? as i32;
        buffer.remove_bits(len);
        Ok(i)
    }
}

/// LzxDecoder.
pub(crate) struct LzxDecoder {
    // LRU offset system
    r0: i32,
    r1: i32,
    r2: i32,
    // decoding window
    window: Vec<u8>,
    window_size: i32,
    window_pos: i32,
    main_element_count: i32,
    // current block information
    block_type: LzxBlockType,
    block_length: i32,
    block_remaining: i32,
    // Intel CALL instruction optimization
    intel_file_size: i32,
    intel_current_position: i32,
    intel_started: bool,
    // decoding tables
    pre_tree: HuffTable,
    main_tree: HuffTable,
    length_tree: HuffTable,
    aligned_tree: HuffTable,
    frames_read: i32,
    read_header: bool,
}

/// Decompresses one XNB LZX payload (`compressedSize - 14` bytes) and returns
/// at most `output_length` (the header's `decompressedSize`) bytes. Note: for
/// some assets (e.g. `Fonts/Mouse_Text.xnb`) the declared size overshoots the
/// real data; the surplus decodes to trailing garbage which we keep, exactly
/// like the C# tool does — consumers stop at the end of the parsed structure.
pub(crate) fn decompress(data: &[u8], output_length: usize) -> Result<Vec<u8>> {
    let mut input = LzxInput::new(data);
    let mut decoder = LzxDecoder::new();
    let mut output: Vec<u8> = Vec::with_capacity(output_length.min(64 << 20));
    decoder.decompress_into(&mut input, data.len(), &mut output)?;
    output.truncate(output_length);
    Ok(output)
}

impl LzxDecoder {
    fn new() -> Self {
        Self {
            r0: 1,
            r1: 1,
            r2: 1,
            window: Vec::new(),
            window_size: 0,
            window_pos: 0,
            main_element_count: 0,
            block_type: LzxBlockType::Invalid,
            block_length: 0,
            block_remaining: 0,
            intel_file_size: 0,
            intel_current_position: 0,
            intel_started: false,
            pre_tree: HuffTable::new(PRETREE_MAX_SYMBOLS, PRETREE_TABLE_BITS),
            main_tree: HuffTable::new(MAINTREE_MAX_SYMBOLS, MAINTREE_TABLE_BITS),
            length_tree: HuffTable::new(LENGTH_MAX_SYMBOLS, LENGTH_TABLE_BITS),
            aligned_tree: HuffTable::new(ALIGNED_MAX_SYMBOLS, ALIGNED_TABLE_BITS),
            frames_read: 0,
            read_header: true,
        }
    }

    // LzxDecoder.Reset
    fn reset(&mut self) {
        self.r0 = 1;
        self.r1 = 1;
        self.r2 = 1;
        self.window_size = 1 << 16;
        self.window = vec![0xDCu8; self.window_size as usize];
        self.window_pos = 0;
        self.main_element_count = NUM_CHARS as i32 + (16 << 4);
        self.read_header = true;
        self.frames_read = 0;
        self.block_remaining = 0;
        self.block_type = LzxBlockType::Invalid;
        self.intel_current_position = 0;
        self.intel_started = false;
        self.pre_tree.reset();
        self.main_tree.reset();
        self.length_tree.reset();
        self.aligned_tree.reset();
    }

    // LzxDecoder.Decompress: headered-block framing (0xFF escape => 3-byte
    // frame size header), each block appends frameSize bytes to the output.
    fn decompress_into(&mut self, input: &mut LzxInput, input_length: usize, output: &mut Vec<u8>) -> Result<()> {
        self.reset();
        let end_position = input.position() + input_length;
        let mut pos = input.position();

        while input.position() < end_position {
            input.set_position(pos);

            let hi = input.read_byte() as i32;
            let lo = input.read_byte() as i32;
            let mut block_size = (hi << 8) | lo;
            // all blocks by default will output 32Kb of data
            let mut frame_size = 0x8000;
            // ... unless this block is special; its header is identified by a 0xFF byte
            if hi == 0xFF {
                let hi = lo;
                let lo = input.read_byte() as i32;
                frame_size = (hi << 8) | lo;
                let hi = input.read_byte() as i32;
                let lo = input.read_byte() as i32;
                block_size = (hi << 8) | lo;
                pos += 5;
            } else {
                pos += 2;
            }

            // either says there is nothing to decode
            if block_size == 0 || frame_size == 0 {
                break;
            }

            self.decompress_block(input, block_size, output, frame_size)?;
            pos += block_size as usize;
        }
        Ok(())
    }

    // LzxDecoder.DecompressBlock
    fn decompress_block(&mut self, input: &mut LzxInput, input_length: i32, output: &mut Vec<u8>, output_length: i32) -> Result<()> {
        let startpos = input.position();
        let endpos = startpos + input_length as usize;

        let mut buffer = LzxBuffer::new(input);

        if self.read_header {
            if buffer.read_bits(1)? == 1 {
                // Intel optimization header
                let hi = buffer.read_bits(16)?;
                let lo = buffer.read_bits(16)?;
                self.intel_file_size = ((hi << 16) | lo) as i32;
            }
            self.read_header = false;
        }

        let window_size = self.window_size;
        let mut window_posn = self.window_pos;
        let mut r0 = self.r0;
        let mut r1 = self.r1;
        let mut r2 = self.r2;

        let mut togo = output_length;

        while togo > 0 {
            if self.block_remaining == 0 {
                if self.block_type == LzxBlockType::Uncompressed {
                    // realign bitstream to word
                    if (self.block_length & 1) == 1 {
                        buffer.raw_read_byte();
                    }
                    buffer.reset();
                }

                let next_block_type = buffer.read_bits(3)?;
                if next_block_type > 3 {
                    bail!("Invalid block type: {next_block_type}");
                }
                self.block_type = match next_block_type {
                    1 => LzxBlockType::Verbatim,
                    2 => LzxBlockType::Aligned,
                    3 => LzxBlockType::Uncompressed,
                    _ => LzxBlockType::Invalid,
                };

                let a = buffer.read_bits(16)?;
                let b = buffer.read_bits(8)?;
                self.block_length = ((a << 8) | b) as i32;
                self.block_remaining = self.block_length;

                if self.block_type == LzxBlockType::Aligned {
                    for i in 0..8 {
                        self.aligned_tree.length[i] = buffer.read_bits(3)? as u8;
                    }
                    self.aligned_tree.make_decode_table()?;
                    // rest of aligned header is the same as verbatim (C#: goto case)
                }
                if self.block_type == LzxBlockType::Aligned || self.block_type == LzxBlockType::Verbatim {
                    let main_element_count = self.main_element_count as usize;
                    {
                        let Self { pre_tree, main_tree, .. } = self;
                        read_lengths(pre_tree, &mut main_tree.length[..], 0, NUM_CHARS, &mut buffer)?;
                        read_lengths(pre_tree, &mut main_tree.length[..], NUM_CHARS, main_element_count, &mut buffer)?;
                    }
                    self.main_tree.make_decode_table()?;
                    if self.main_tree.length[0xE8] != 0 {
                        self.intel_started = true;
                    }
                    {
                        let Self { pre_tree, length_tree, .. } = self;
                        read_lengths(pre_tree, &mut length_tree.length[..], 0, NUM_SECONDARY_LENGTHS, &mut buffer)?;
                    }
                    self.length_tree.make_decode_table()?;
                } else if self.block_type == LzxBlockType::Uncompressed {
                    self.intel_started = true; // because we can't assume otherwise
                    buffer.ensure_bits(16)?; // get up to 16 pad bits into the buffer
                    if buffer.remaining_bits > 16 {
                        buffer.set_position(buffer.position().wrapping_sub(2)); // and align the bitstream!
                    }
                    r0 = read_u32_raw(&mut buffer) as i32;
                    r1 = read_u32_raw(&mut buffer) as i32;
                    r2 = read_u32_raw(&mut buffer) as i32;
                } else if self.block_type == LzxBlockType::Invalid {
                    bail!("Unknown block type {:?}", self.block_type);
                }
            }

            // buffer exhaustion check (LzxDecoder.DecompressBlock)
            if buffer.position() > startpos + input_length as usize {
                // it's possible to have a file where the next run is less than 16 bits
                // in size; allow up to 2 bytes of overread with >= 16 bits buffered
                if buffer.position() > endpos + 2 || buffer.remaining_bits < 16 {
                    bail!("LZX block overran its input");
                }
            }

            while self.block_remaining > 0 && togo > 0 {
                let mut this_run = self.block_remaining;
                if this_run > togo {
                    this_run = togo;
                }
                togo -= this_run;
                self.block_remaining -= this_run;

                // apply 2^x-1 mask
                window_posn &= window_size - 1;

                // runs can't straddle the window wraparound
                if window_posn + this_run > window_size {
                    bail!("(window_posn + this_run) > window_size");
                }

                match self.block_type {
                    LzxBlockType::Verbatim => {
                        self.decode_matches(&mut buffer, &mut window_posn, this_run, &mut r0, &mut r1, &mut r2, false)?;
                    }
                    LzxBlockType::Aligned => {
                        self.decode_matches(&mut buffer, &mut window_posn, this_run, &mut r0, &mut r1, &mut r2, true)?;
                    }
                    LzxBlockType::Uncompressed => {
                        if buffer.position() + this_run as usize > endpos {
                            bail!("(input.position() + this_run) > endpos");
                        }
                        let end = (window_posn + this_run) as usize;
                        buffer.raw_read_into(&mut self.window[window_posn as usize..end]);
                        window_posn += this_run;
                    }
                    LzxBlockType::Invalid => bail!("Invalid block type: {:?}", self.block_type),
                }
            }
        }

        if togo != 0 {
            bail!("togo != 0");
        }

        let mut start_window_pos = window_posn;
        if start_window_pos == 0 {
            start_window_pos = window_size;
        }
        start_window_pos -= output_length;

        if start_window_pos >= 0 {
            let s = start_window_pos as usize;
            output.extend_from_slice(&self.window[s..s + output_length as usize]);
        } else {
            // window wrapped around the 64K boundary; copy the two parts
            // (liblzx does the same; the C# tool would throw on the negative index)
            let s = (window_size + start_window_pos) as usize;
            output.extend_from_slice(&self.window[s..window_size as usize]);
            output.extend_from_slice(&self.window[..window_posn as usize]);
        }

        self.window_pos = window_posn;
        self.r0 = r0;
        self.r1 = r1;
        self.r2 = r2;

        // intel E8 decoding (C#: "TODO finish intel E8 decoding") — the scan
        // loop never runs because the absolute output position is always
        // >= output_length - 10 at this point; keep the bookkeeping faithful.
        self.frames_read += 1;
        // `LzxDecoder.Decompress` applies Intel E8 call translation when the stream
        // sets an executable size; game assets never do, so only the position
        // bookkeeping is ported here.
        if self.frames_read <= 32768 && self.intel_file_size != 0 {
            self.intel_current_position += output_length;
        }
        Ok(())
    }

    /// The Verbatim / Aligned inner loops of LzxDecoder.DecompressBlock, which
    /// are identical except for the match-offset decoding (`aligned` selects
    /// the aligned-offset variant).
    fn decode_matches(
        &mut self,
        buffer: &mut LzxBuffer,
        window_posn: &mut i32,
        mut this_run: i32,
        r0: &mut i32,
        r1: &mut i32,
        r2: &mut i32,
        aligned: bool,
    ) -> Result<()> {
        let eb = extra_bits();
        let pb = position_base();
        let window_size = self.window_size;

        while this_run > 0 {
            let mut main_element = self.main_tree.read_huff_sym(buffer)?;
            if main_element < NUM_CHARS as i32 {
                // literal: 0 to NUM_CHARS-1
                self.put(*window_posn, main_element as u8)?;
                *window_posn += 1;
                this_run -= 1;
            } else {
                // match: NUM_CHARS + ((slot<<3) | length_header (3 bits))
                main_element -= NUM_CHARS as i32;

                let mut match_length = main_element & NUM_PRIMARY_LENGTHS;
                if match_length == NUM_PRIMARY_LENGTHS {
                    let length_footer = self.length_tree.read_huff_sym(buffer)?;
                    match_length += length_footer;
                }
                match_length += MIN_MATCH;

                let mut match_offset = main_element >> 3;

                if match_offset > 2 {
                    // not repeated offset
                    if !aligned {
                        if match_offset != 3 {
                            let extra = eb[match_offset as usize];
                            let verbatim_bits = buffer.read_bits(extra)? as i32;
                            match_offset = pb[match_offset as usize] - 2 + verbatim_bits;
                        } else {
                            match_offset = 1;
                        }
                    } else {
                        let mut extra = eb[match_offset as usize];
                        match_offset = pb[match_offset as usize] - 2;
                        if extra > 3 {
                            // verbatim and aligned bits
                            extra -= 3;
                            let verbatim_bits = buffer.read_bits(extra)? as i32;
                            match_offset += verbatim_bits << 3;
                            let aligned_bits = self.aligned_tree.read_huff_sym(buffer)?;
                            match_offset += aligned_bits;
                        } else if extra == 3 {
                            // aligned bits only
                            let aligned_bits = self.aligned_tree.read_huff_sym(buffer)?;
                            match_offset += aligned_bits;
                        } else if extra > 0 {
                            // verbatim bits only (extra==1, extra==2)
                            let verbatim_bits = buffer.read_bits(extra)? as i32;
                            match_offset += verbatim_bits;
                        } else {
                            // extra == 0: ???
                            match_offset = 1;
                        }
                    }

                    // update repeated offset LRU queue
                    *r2 = *r1;
                    *r1 = *r0;
                    *r0 = match_offset;
                } else if match_offset == 0 {
                    match_offset = *r0;
                } else if match_offset == 1 {
                    match_offset = *r1;
                    *r1 = *r0;
                    *r0 = match_offset;
                } else {
                    // match_offset == 2
                    match_offset = *r2;
                    *r2 = *r0;
                    *r0 = match_offset;
                }

                let mut rundest = *window_posn;
                this_run -= match_length;

                // copy any wrapped around source data
                let mut runsrc;
                if *window_posn >= match_offset {
                    // no wrap
                    runsrc = rundest - match_offset;
                } else {
                    runsrc = rundest + (window_size - match_offset);
                    let mut copy_length = match_offset - *window_posn;
                    if copy_length < match_length {
                        match_length -= copy_length;
                        *window_posn += copy_length;
                        while copy_length > 0 {
                            copy_length -= 1;
                            let b = self.get(runsrc)?;
                            self.put(rundest, b)?;
                            rundest += 1;
                            runsrc += 1;
                        }
                        runsrc = 0;
                    }
                }
                *window_posn += match_length;

                // copy match data - no worries about destination wraps
                while match_length > 0 {
                    match_length -= 1;
                    let b = self.get(runsrc)?;
                    self.put(rundest, b)?;
                    rundest += 1;
                    runsrc += 1;
                }
            }
        }
        Ok(())
    }

    fn get(&self, idx: i32) -> Result<u8> {
        self.window
            .get(idx as usize)
            .copied()
            .ok_or_else(|| anyhow!("LZX window index {idx} out of range"))
    }

    fn put(&mut self, idx: i32, b: u8) -> Result<()> {
        if idx < 0 || idx as usize >= self.window.len() {
            bail!("LZX window index {idx} out of range");
        }
        self.window[idx as usize] = b;
        Ok(())
    }
}

// reads R0/R1/R2 in uncompressed blocks: 4 raw little-endian bytes each
fn read_u32_raw(buffer: &mut LzxBuffer) -> u32 {
    let lo = buffer.raw_read_byte() as u32;
    let ml = buffer.raw_read_byte() as u32;
    let mh = buffer.raw_read_byte() as u32;
    let hi = buffer.raw_read_byte() as u32;
    lo | ml << 8 | mh << 16 | hi << 24
}

// LzxDecoder.ReadLengths
fn read_lengths(pre_tree: &mut HuffTable, lens: &mut [u8], first: usize, last: usize, buffer: &mut LzxBuffer) -> Result<()> {
    for x in 0..20 {
        pre_tree.length[x] = buffer.read_bits(4)? as u8;
    }
    pre_tree.make_decode_table()?;

    let mut x = first;
    while x < last {
        let z = pre_tree.read_huff_sym(buffer)?;
        if z == 17 {
            let mut y = buffer.read_bits(4)? as i32 + 4;
            while y != 0 {
                y -= 1;
                lens[x] = 0;
                x += 1;
            }
        } else if z == 18 {
            let mut y = buffer.read_bits(5)? as i32 + 20;
            while y != 0 {
                y -= 1;
                lens[x] = 0;
                x += 1;
            }
        } else if z == 19 {
            let mut y = buffer.read_bits(1)? as i32 + 4;
            let mut z = pre_tree.read_huff_sym(buffer)? as i32;
            z = lens[x] as i32 - z;
            if z < 0 {
                z += 17;
            }
            while y != 0 {
                y -= 1;
                lens[x] = z as u8;
                x += 1;
            }
        } else {
            let mut z = lens[x] as i32 - z;
            if z < 0 {
                z += 17;
            }
            lens[x] = z as u8;
            x += 1;
        }
    }
    Ok(())
}
