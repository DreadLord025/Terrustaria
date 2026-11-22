//! The save files' wall-clock seam — .NET `DateTime` ticks / `ToBinary`
//! (`WorldFile.cs:1250-1251`, `Player.cs:54909`, `LoadPlayer:55098`).
//!
//! .NET ticks are 100 ns units since 0001-01-01T00:00:00; the Unix epoch sits
//! at tick 621355968000000000. `DateTime.ToBinary` (the form both file formats
//! store) packs the ticks with a 2-bit kind in bits 62-63: `Utc` = `01`
//! (bit 62 set), `Local` = `10` (bit 63 set, ticks offset-adjusted),
//! `Unspecified` = `00`. The port owns no time zone database, so every stamp
//! it writes is **Utc** — `CreationTime`'s C# source is `DateTime.Now`
//! (`WorldFile.CreateMetadata:600`, kind Local); the port's Utc stand-in is a
//! documented deviation (ANALYSIS "The save files' wall clock"). Reads mask
//! the kind bits off (`DateTime.FromBinary`'s Utc arm); a Local-kind value
//! from a foreign file is read as its stored ticks without the offset
//! round-trip (the same deviation, read side).

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// Ticks between 0001-01-01 and 1970-01-01 (`DateTime.UnixEpoch.Ticks`).
pub const TICKS_AT_UNIX_EPOCH: i64 = 621_355_968_000_000_000;
/// `DateTimeKind.Utc`'s `ToBinary` flag (bit 62).
const KIND_UTC: i64 = 0x4000_0000_0000_0000;
/// The two kind bits `ToBinary` may set (bits 62-63).
const KIND_MASK: i64 = 0xC000_0000_0000_0000u64 as i64;

/// Unix seconds + sub-second nanos -> .NET ticks.
pub fn unix_to_ticks(secs: u64, subsec_nanos: u32) -> i64 {
    TICKS_AT_UNIX_EPOCH + (secs as i64) * 10_000_000 + (subsec_nanos as i64) / 100
}

/// `DateTime.ToBinary` for a Utc-kind value: ticks with bit 62 set.
pub fn to_binary_utc(ticks: i64) -> i64 {
    ticks | KIND_UTC
}

/// `DateTime.FromBinary`'s tick extraction: the kind bits masked off (the
/// Local-offset arm is not modelled — see the module doc).
pub fn from_binary_ticks(binary: i64) -> i64 {
    binary & !KIND_MASK
}

/// `DateTime.UtcNow.ToBinary()` right now (`WorldFile.cs:1251`,
/// `Player.cs:54909`). A clock before the Unix epoch reads as the epoch
/// (the port's floor — no file format can carry a negative stamp meaningfully).
pub fn utc_now_binary() -> i64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    to_binary_utc(unix_to_ticks(now.as_secs(), now.subsec_nanos()))
}

/// `File.GetLastWriteTimeUtc(path).ToBinary()` — the load-side fallbacks
/// (`WorldFile.cs:317`, `Player.cs:55098`). `GetLastWriteTimeUtc` returns a
/// Utc-kind DateTime, so the stamp carries bit 62. `None` when the file's
/// mtime cannot be read.
pub fn file_mtime_binary(path: &Path) -> Option<i64> {
    let modified = std::fs::metadata(path).ok()?.modified().ok()?;
    let since = modified.duration_since(UNIX_EPOCH).ok()?;
    Some(to_binary_utc(unix_to_ticks(since.as_secs(), since.subsec_nanos())))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// W4-c43 witness: the tick conversion reproduces the fixture harness's
    /// pinned clock — `FixtureClock.UtcNow` = 2026-11-18T00:00:00Z, whose
    /// `ToBinary` is `0x48DF_4506_E391_C000` (the `golden.plr` timestamp,
    /// `player_file.rs`'s byte-parity test).
    #[test]
    fn w4_c43_the_fixture_clock_pins_the_conversion() {
        // 2026-11-18T00:00:00Z = Unix 1_794_960_000.
        let ticks = unix_to_ticks(1_794_960_000, 0);
        assert_eq!(ticks, 639_305_568_000_000_000);
        assert_eq!(to_binary_utc(ticks), 0x48DF_4506_E391_C000u64 as i64);
        assert_eq!(from_binary_ticks(0x48DF_4506_E391_C000u64 as i64), ticks);
    }

    #[test]
    fn w4_c43_the_epoch_and_sub_second_arms() {
        assert_eq!(unix_to_ticks(0, 0), TICKS_AT_UNIX_EPOCH);
        // 999_999_999 ns = 9_999_999 ticks (the sub-100ns remainder drops).
        assert_eq!(unix_to_ticks(0, 999_999_999), TICKS_AT_UNIX_EPOCH + 9_999_999);
        // The kind bits round-trip off, both Utc and a foreign Local stamp.
        let local = TICKS_AT_UNIX_EPOCH | (0x8000_0000_0000_0000u64 as i64);
        assert_eq!(from_binary_ticks(local), TICKS_AT_UNIX_EPOCH);
    }

    #[test]
    fn w4_c43_utc_now_carries_the_kind_bit_and_advances() {
        let a = utc_now_binary();
        assert_ne!(a & KIND_UTC, 0, "the Utc kind bit is set");
        assert!(from_binary_ticks(a) > TICKS_AT_UNIX_EPOCH);
    }
}
