//! XNB v5 container parsing and dispatch.
//! Port of the container half of `tools/XnbExtract/Program.cs` (`Program.Extract`,
//! `Program.Read7BitEncodedInt`).

use anyhow::{anyhow, bail, Result};

use crate::lzx;

const HEADER_SIZE: usize = 14; // Program.HeaderSize

/// Little-endian binary reader over the decompressed payload (stands in for
/// `System.IO.BinaryReader`).
pub(crate) struct ByteReader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> ByteReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }
    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }
    pub fn read_u8(&mut self) -> Result<u8> {
        let b = *self.data.get(self.pos).ok_or_else(|| anyhow!("unexpected end of payload at {}", self.pos))?;
        self.pos += 1;
        Ok(b)
    }
    pub fn read_bytes(&mut self, n: usize) -> Result<&'a [u8]> {
        let end = self.pos.checked_add(n).ok_or_else(|| anyhow!("byte count overflow at {}", self.pos))?;
        let s = self
            .data
            .get(self.pos..end)
            .ok_or_else(|| anyhow!("unexpected end of payload: need {n} bytes at {}", self.pos))?;
        self.pos = end;
        Ok(s)
    }
    pub fn read_i32(&mut self) -> Result<i32> {
        Ok(i32::from_le_bytes(self.read_bytes(4)?.try_into().unwrap()))
    }
    // Program.Read7BitEncodedInt
    pub fn read_7bit_encoded_int(&mut self) -> Result<u32> {
        let mut result: u32 = 0;
        let mut shift = 0;
        loop {
            let b = self.read_u8()?;
            result |= ((b & 0x7F) as u32) << shift;
            if b & 0x80 == 0 {
                return Ok(result);
            }
            shift += 7;
            if shift > 28 {
                bail!("malformed 7-bit encoded int at {}", self.pos);
            }
        }
    }
}

/// A parsed XNB file: the reader table plus the asset payload that follows the
/// metadata (i.e. positioned exactly where the asset's ContentTypeReader starts).
pub(crate) struct XnbFile {
    pub readers: Vec<String>,
    pub type_id: u32,
    pub payload: Vec<u8>,
}

impl XnbFile {
    pub fn read(raw: &[u8]) -> Result<XnbFile> {
        if raw.len() < HEADER_SIZE || raw[0..3] != *b"XNB" {
            bail!("not an XNB file");
        }
        let _platform = raw[3]; // 'w' = Windows
        let version = raw[4];
        if version != 5 {
            bail!("unsupported XNB version {version}");
        }
        let compressed = raw[5] & 0x80 != 0;
        let compressed_size = i32::from_le_bytes(raw[6..10].try_into().unwrap());
        if compressed_size < 0 || compressed_size as usize > raw.len() {
            bail!("bad compressed size {compressed_size} for a {} byte file", raw.len());
        }
        let decompressed_size = if compressed {
            i32::from_le_bytes(raw[10..14].try_into().unwrap())
        } else {
            compressed_size
        };
        if decompressed_size < 0 {
            bail!("bad decompressed size {decompressed_size}");
        }

        let decompressed;
        let body: &[u8] = if compressed {
            decompressed = lzx::decompress(&raw[HEADER_SIZE..compressed_size as usize], decompressed_size as usize)?;
            &decompressed
        } else {
            &raw[HEADER_SIZE..compressed_size as usize]
        };

        let mut r = ByteReader::new(body);
        let reader_count = r.read_7bit_encoded_int()?;
        if reader_count > 64 {
            bail!("implausible reader count {reader_count}");
        }
        let mut readers = Vec::with_capacity(reader_count as usize);
        for _ in 0..reader_count {
            let len = r.read_7bit_encoded_int()? as usize;
            let bytes = r.read_bytes(len)?;
            let _reader_version = r.read_i32()?;
            readers.push(String::from_utf8_lossy(bytes).into_owned());
        }
        let shared_resources = r.read_7bit_encoded_int()?;
        if shared_resources != 0 {
            bail!("unexpected shared resources: {shared_resources}");
        }
        let type_id = r.read_7bit_encoded_int()?;
        let payload = r.read_bytes(r.remaining())?.to_vec();
        Ok(XnbFile { readers, type_id, payload })
    }

    /// Name of the ContentTypeReader selected by the file's type id.
    pub fn asset_reader(&self) -> Result<&str> {
        if self.type_id == 0 {
            bail!("asset is a shared resource (type id 0)");
        }
        if (self.type_id as usize) > self.readers.len() {
            bail!("bad type id {} (readers={})", self.type_id, self.readers.len());
        }
        Ok(&self.readers[(self.type_id - 1) as usize])
    }
}
