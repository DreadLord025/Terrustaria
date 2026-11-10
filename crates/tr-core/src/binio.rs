//! Shared little-endian binary cursor for the save-file formats.
//!
//! One cursor serves the three hand-rolled readers that used to live in
//! `save.rs` (`TRWLD`), `wld.rs` (`.wld`) and `map/file.rs` (`.map` prefix and
//! body). Every read primitive returns `Option` and leaves the cursor untouched
//! on a short read; each parser converts `None` into its own error type in one
//! place, so the original per-format error behaviour is preserved (`anyhow`
//! bail for `.wld`/`.trwld`, `Ok(None)` rejection for the `.map` prefix,
//! `io::Error` for the `.map` body).
//!
//! The `.trwld` **header prefix** reader ([`read_trwld_header`]) also lives
//! here: it is the header half of the `save.rs` parse, shared by the save
//! listing (`ui/save_list.rs`) and anyone that needs the world's name/size
//! without loading the tile body.

/// A little-endian cursor over a borrowed byte slice.
pub struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    /// Starts a cursor at offset 0.
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    /// Offset of the next byte. A failed read leaves it unchanged, so callers
    /// can name the position a short read stopped at.
    pub fn pos(&self) -> usize {
        self.pos
    }

    /// Borrows the next `n` bytes.
    pub fn take(&mut self, n: usize) -> Option<&'a [u8]> {
        let end = self.pos.checked_add(n)?;
        let bytes = self.data.get(self.pos..end)?;
        self.pos = end;
        Some(bytes)
    }

    /// Jumps to `pos`; `None` past the end of the buffer.
    pub fn seek(&mut self, pos: usize) -> Option<()> {
        if pos > self.data.len() {
            return None;
        }
        self.pos = pos;
        Some(())
    }

    /// Drops the next `n` bytes.
    pub fn skip(&mut self, n: usize) -> Option<()> {
        self.take(n).map(|_| ())
    }

    pub fn u8(&mut self) -> Option<u8> {
        Some(self.take(1)?[0])
    }

    pub fn u16(&mut self) -> Option<u16> {
        Some(u16::from_le_bytes(self.take(2)?.try_into().ok()?))
    }

    pub fn i16(&mut self) -> Option<i16> {
        Some(i16::from_le_bytes(self.take(2)?.try_into().ok()?))
    }

    pub fn u32(&mut self) -> Option<u32> {
        Some(u32::from_le_bytes(self.take(4)?.try_into().ok()?))
    }

    pub fn i32(&mut self) -> Option<i32> {
        Some(i32::from_le_bytes(self.take(4)?.try_into().ok()?))
    }

    pub fn u64(&mut self) -> Option<u64> {
        Some(u64::from_le_bytes(self.take(8)?.try_into().ok()?))
    }

    pub fn f32(&mut self) -> Option<f32> {
        Some(f32::from_le_bytes(self.take(4)?.try_into().ok()?))
    }

    pub fn f64(&mut self) -> Option<f64> {
        Some(f64::from_le_bytes(self.take(8)?.try_into().ok()?))
    }

    /// A `bool` stored as one byte (non-zero is true, `BinaryWriter` style).
    pub fn boolean(&mut self) -> Option<bool> {
        Some(self.u8()? != 0)
    }

    /// The `.wld` `BitsByte`: one byte holding up to eight flags
    /// (`Terraria.BitsByte`, `BitsByte.cs:7`), bit 0 first.
    pub fn bits_byte(&mut self) -> Option<BitsByte> {
        self.u8().map(BitsByte)
    }

    /// .NET `BinaryReader.ReadString`: 7-bit length prefix + UTF-8 payload,
    /// decoded lossily (the `.wld` reader's behaviour).
    pub fn cs_string_lossy(&mut self) -> Option<String> {
        Some(String::from_utf8_lossy(self.cs_string_bytes()?).into_owned())
    }

    /// [`Reader::cs_string_lossy`], but `None` on invalid UTF-8: the `.map`
    /// prefix rejects foreign files instead of lossy-decoding their names.
    pub fn cs_string(&mut self) -> Option<String> {
        String::from_utf8(self.cs_string_bytes()?.to_vec()).ok()
    }

    /// A 7-bit-encoded length prefix followed by that many payload bytes.
    fn cs_string_bytes(&mut self) -> Option<&'a [u8]> {
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
        self.take(len)
    }
}

/// The `.trwld` container magic (`save.rs:16` writes it) — the port's own
/// format, not a decompiled constant.
pub const TRWLD_MAGIC: &[u8; 8] = b"TRWLD01\0";

/// The `.trwld` header prefix — the part of the save header every consumer
/// needs before the tile body (magic, version, seed, name, size).
///
/// The port's own container format (`tr_core::save`), not a decompiled layout:
/// `save.rs:126-142` writes/reads it, `Main.cs` has no counterpart — the
/// listing stands in for `WorldFileData` (`WorldFileData.cs:75`'s `IsValid`
/// is `LoadStatus == Ok`, the port's `valid` below).
#[derive(Clone, Debug, PartialEq)]
pub struct TrwldHeader {
    /// The world name; `None` when the prefix is truncated before it.
    pub name: Option<String>,
    /// The world width; 1 when the prefix is truncated (`FromInvalidWorld`'s
    /// 1x1 world, `WorldFileData.cs:357`).
    pub width: i32,
    /// The prefix parses and passes the plausibility guard.
    pub valid: bool,
}

/// Reads the `.trwld` header prefix over `data`, applying the same guards
/// `tr_core::save::parse` applies (`save.rs:126-142`): the 8-byte magic, a
/// version of 0 or beyond `version_ceiling` refused (vanilla refuses later
/// versions the same way, `WorldFile.cs:190-194`, `StatusID.LaterVersion`),
/// and the width/height plausibility band. The tile body is never read.
pub fn read_trwld_header(data: &[u8], version_ceiling: u32) -> TrwldHeader {
    let mut r = Reader::new(data);
    // magic (`save.rs:128-130`)
    if r.take(8) != Some(TRWLD_MAGIC.as_slice()) {
        return TrwldHeader { name: None, width: 1, valid: false };
    }
    let Some(version) = r.u32() else {
        return TrwldHeader { name: None, width: 1, valid: false };
    };
    if version == 0 || version > version_ceiling {
        return TrwldHeader { name: None, width: 1, valid: false };
    }
    // seed (`save.rs:135`) — not needed past here.
    if r.take(8).is_none() {
        return TrwldHeader { name: None, width: 1, valid: false };
    }
    let Some(name_len) = r.u32() else {
        return TrwldHeader { name: None, width: 1, valid: false };
    };
    let name = r
        .take(name_len as usize)
        .map(String::from_utf8_lossy)
        .map(|s| s.into_owned());
    let Some(width) = r.i32() else {
        return TrwldHeader { name, width: 1, valid: false };
    };
    let Some(height) = r.i32() else {
        return TrwldHeader { name, width: 1, valid: false };
    };
    // The plausibility guard `save.rs:140-142` — a world the port cannot load
    // stays listed but not valid.
    if width <= 0 || height <= 0 || width > 20_000 || height > 20_000 {
        return TrwldHeader { name, width: 1, valid: false };
    }
    TrwldHeader { name, width, valid: true }
}

/// A little-endian writer over an owned buffer, the counterpart of [`Reader`]
/// (`BinaryWriter`-shaped: raw scalars and 7-bit-prefixed strings).
#[derive(Default)]
pub struct Writer {
    buf: Vec<u8>,
}

impl Writer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the written bytes.
    pub fn into_inner(self) -> Vec<u8> {
        self.buf
    }

    /// Appends raw bytes.
    pub fn bytes(&mut self, b: &[u8]) {
        self.buf.extend_from_slice(b);
    }

    /// `BinaryWriter.Write(string)`: 7-bit-encoded byte length, then UTF-8.
    pub fn cs_string(&mut self, s: &str) {
        let bytes = s.as_bytes();
        let mut len = bytes.len();
        while len >= 0x80 {
            self.buf.push(((len & 0x7f) | 0x80) as u8);
            len >>= 7;
        }
        self.buf.push(len as u8);
        self.buf.extend_from_slice(bytes);
    }

    pub fn boolean(&mut self, v: bool) {
        self.buf.push(u8::from(v));
    }

    pub fn u8(&mut self, v: u8) {
        self.buf.push(v);
    }

    pub fn i16(&mut self, v: i16) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub fn u16(&mut self, v: u16) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub fn i32(&mut self, v: i32) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub fn u32(&mut self, v: u32) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub fn u64(&mut self, v: u64) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub fn f32(&mut self, v: f32) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub fn f64(&mut self, v: f64) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    /// Writes a [`BitsByte`] as its one backing byte
    /// (`BinaryWriter.Write(BitsByte)` via the implicit byte conversion).
    pub fn bits_byte(&mut self, bb: BitsByte) {
        self.u8(bb.0);
    }
}

/// One byte addressed as eight LSB-first flags — the port of `Terraria.BitsByte`
/// (`BitsByte.cs:7-47`): `this[key]` is `value & (1 << key)`, so bit 0 is the
/// first flag and bit 7 the last. The C# indexer accepts only 0..7; reads
/// outside that range answer `false` (a byte has no such bit) and writes are
/// dropped, matching the C# byte shift results.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct BitsByte(pub u8);

impl BitsByte {
    /// An all-clear byte.
    pub fn new() -> Self {
        Self(0)
    }

    /// `this[key]` (`BitsByte.cs:13-20`).
    pub fn bit(self, key: usize) -> bool {
        key < 8 && self.0 & (1 << key) != 0
    }

    /// `this[key] = value` (`BitsByte.cs:20-29`).
    pub fn set(&mut self, key: usize, value: bool) {
        if key >= 8 {
            return;
        }
        if value {
            self.0 |= 1 << key;
        } else {
            self.0 &= !(1 << key);
        }
    }

    /// `ClearAll` (`BitsByte.cs:44`).
    pub fn clear_all(&mut self) {
        self.0 = 0;
    }

    /// `SetAll` (`BitsByte.cs:48`).
    pub fn set_all(&mut self) {
        self.0 = u8::MAX;
    }
}

impl From<u8> for BitsByte {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl From<BitsByte> for u8 {
    fn from(bb: BitsByte) -> Self {
        bb.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_every_primitive() {
        let mut w = Writer::new();
        w.boolean(true);
        w.u8(7);
        w.i16(-2);
        w.u16(65535);
        w.i32(-123456);
        w.u32(4_000_000_000);
        w.f32(1.5);
        w.f64(-0.25);
        w.u64(u64::MAX);
        w.bytes(b"raw");
        w.cs_string("");
        w.cs_string("Terrustaria");
        let long = "x".repeat(200); // a two-byte 7-bit length prefix
        w.cs_string(&long);
        let buf = w.into_inner();

        let mut r = Reader::new(&buf);
        assert_eq!(r.boolean(), Some(true));
        assert_eq!(r.u8(), Some(7));
        assert_eq!(r.i16(), Some(-2));
        assert_eq!(r.u16(), Some(65535));
        assert_eq!(r.i32(), Some(-123456));
        assert_eq!(r.u32(), Some(4_000_000_000));
        assert_eq!(r.f32(), Some(1.5));
        assert_eq!(r.f64(), Some(-0.25));
        assert_eq!(r.u64(), Some(u64::MAX));
        assert_eq!(r.take(3), Some(&b"raw"[..]));
        assert_eq!(r.cs_string(), Some(String::new()));
        assert_eq!(r.cs_string(), Some("Terrustaria".into()));
        assert_eq!(r.cs_string_lossy(), Some(long));
        assert_eq!(r.pos(), buf.len());
    }

    #[test]
    fn short_reads_return_none_and_hold_the_cursor() {
        let mut r = Reader::new(&[1, 2, 3, 4, 5]);
        assert_eq!(r.u32(), Some(0x0403_0201));
        assert_eq!(r.pos(), 4);
        assert_eq!(r.u32(), None); // one byte short
        assert_eq!(r.pos(), 4);
        assert_eq!(r.u8(), Some(5));
        assert_eq!(r.u8(), None);
        assert_eq!(r.pos(), 5);
        assert_eq!(r.seek(2), Some(()));
        assert_eq!(r.seek(99), None); // past the end
        assert_eq!(r.pos(), 2);
        assert_eq!(r.skip(2), Some(())); // pos 2 -> 4
        assert_eq!(r.skip(1), Some(())); // pos 4 -> 5
        assert_eq!(r.skip(1), None); // the buffer is exhausted
        assert_eq!(r.pos(), 5);
    }

    /// DW-1.2: every byte value survives a write/read round trip through
    /// `bits_byte`, and the accessors match the C# indexer bit-for-bit.
    #[test]
    fn bits_byte_round_trips_every_value() {
        for value in 0..=u8::MAX {
            let mut w = Writer::new();
            w.bits_byte(BitsByte(value));
            let buf = w.into_inner();
            assert_eq!(buf, vec![value]);
            let mut r = Reader::new(&buf);
            assert_eq!(r.bits_byte(), Some(BitsByte(value)));
        }
    }

    /// DW-1.2: bit order is LSB-first — bit `k` of the byte is flag `k`,
    /// exactly as `BitsByte.this[key]` computes `value & (1 << key)`.
    #[test]
    fn bits_byte_accessors_are_lsb_first() {
        for bit in 0..8usize {
            let mut bb = BitsByte::new();
            assert!(!bb.bit(bit));
            bb.set(bit, true);
            assert_eq!(u8::from(bb), 1 << bit);
            assert!(bb.bit(bit));
            for other in 0..8usize {
                assert_eq!(bb.bit(other), other == bit);
            }
            bb.set(bit, false);
            assert_eq!(u8::from(bb), 0);
        }
        let mut all = BitsByte::new();
        all.set_all();
        assert_eq!(u8::from(all), 0xFF);
        assert!((0..8).all(|b| all.bit(b)));
        all.clear_all();
        assert_eq!(u8::from(all), 0);
        assert!(!(0..8).any(|b| all.bit(b)));
        // Out-of-range keys answer false and are dropped (the C# byte shift).
        assert!(!BitsByte::new().bit(8));
        let mut bb = BitsByte(0xFF);
        bb.set(8, false);
        assert_eq!(u8::from(bb), 0xFF);
    }

    #[test]
    fn truncated_cs_strings_are_rejected() {
        // A 7-bit prefix that keeps going past five bytes.
        let bad_prefix = [0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
        assert_eq!(Reader::new(&bad_prefix).cs_string(), None);
        // A length prefix that promises more than the buffer holds: the prefix
        // is consumed, the missing payload is not.
        let short_payload = [3, b'a', b'b'];
        let mut r = Reader::new(&short_payload);
        assert_eq!(r.cs_string(), None);
        assert_eq!(r.pos(), 1);
        // Invalid UTF-8 is a hard reject for the strict variant only.
        let bad_utf8 = [2, 0xff, 0xfe];
        assert_eq!(Reader::new(&bad_utf8).cs_string(), None);
        assert_eq!(
            Reader::new(&bad_utf8).cs_string_lossy(),
            Some(String::from("\u{fffd}\u{fffd}"))
        );
    }

    /// Builds the `.trwld` header prefix for the reader tests (magic, version,
    /// seed, name, width, height) — the shape `ui/save_list.rs`'s test writer
    /// hands the listing.
    fn trwld_header_bytes(version: u32, name: &str, width: i32, height: i32) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(b"TRWLD01\0");
        buf.extend_from_slice(&version.to_le_bytes());
        buf.extend_from_slice(&0u64.to_le_bytes());
        buf.extend_from_slice(&(name.len() as u32).to_le_bytes());
        buf.extend_from_slice(name.as_bytes());
        buf.extend_from_slice(&width.to_le_bytes());
        buf.extend_from_slice(&height.to_le_bytes());
        buf
    }

    #[test]
    fn trwld_header_parses_the_prefix() {
        let data = trwld_header_bytes(2, "Alpha", 6400, 1800);
        let h = read_trwld_header(&data, 4);
        assert_eq!(h.name.as_deref(), Some("Alpha"));
        assert_eq!(h.width, 6400);
        assert!(h.valid, "the prefix passes every guard");
    }

    #[test]
    fn trwld_header_rejects_the_bad_guards() {
        // Garbage fails the magic (`WorldFileData.FromInvalidWorld`).
        assert!(!read_trwld_header(b"not a world at all", 4).valid);
        // Version 0 and versions past the ceiling are refused
        // (`save.rs:131-134`, `WorldFile.cs:190-194`'s `StatusID.LaterVersion`).
        assert!(!read_trwld_header(&trwld_header_bytes(0, "X", 4200, 1200), 4).valid);
        assert!(!read_trwld_header(&trwld_header_bytes(5, "X", 4200, 1200), 4).valid);
        // Implausible sizes fail the plausibility guard (`save.rs:140-142`).
        assert!(!read_trwld_header(&trwld_header_bytes(2, "X", 0, 1800), 4).valid);
        assert!(!read_trwld_header(&trwld_header_bytes(2, "X", 4200, 0), 4).valid);
        assert!(!read_trwld_header(&trwld_header_bytes(2, "X", 20_001, 1800), 4).valid);
    }

    #[test]
    fn trwld_header_truncation_reports_the_partial_state() {
        // Truncated before the size block: the name is known, the width falls
        // back to `FromInvalidWorld`'s 1x1 (`WorldFileData.cs:357`).
        let mut short = trwld_header_bytes(2, "Alpha", 4200, 1200);
        short.truncate(short.len() - 4);
        let h = read_trwld_header(&short, 4);
        assert_eq!(h.name.as_deref(), Some("Alpha"));
        assert_eq!(h.width, 1);
        assert!(!h.valid);
        // Truncated before the name: nothing is known.
        let mut shorter = trwld_header_bytes(2, "Alpha", 4200, 1200);
        shorter.truncate(20);
        let h = read_trwld_header(&shorter, 4);
        assert_eq!(h.name, None);
        assert_eq!(h.width, 1);
        assert!(!h.valid);
        // An empty buffer (an unreadable file in the listing).
        assert!(!read_trwld_header(&[], 4).valid);
    }
}
