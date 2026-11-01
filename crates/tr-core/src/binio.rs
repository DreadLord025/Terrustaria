//! Shared little-endian binary cursor for the save-file formats.
//!
//! One cursor serves the three hand-rolled readers that used to live in
//! `save.rs` (`TRWLD`), `wld.rs` (`.wld`) and `map/file.rs` (`.map` prefix and
//! body). Every read primitive returns `Option` and leaves the cursor untouched
//! on a short read; each parser converts `None` into its own error type in one
//! place, so the original per-format error behaviour is preserved (`anyhow`
//! bail for `.wld`/`.trwld`, `Ok(None)` rejection for the `.map` prefix,
//! `io::Error` for the `.map` body).

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
}
