// S1-J-3 golden-fixture harness — the pinned clock.
//
// The vanilla writers stamp `DateTime.UtcNow.ToBinary()` (SaveWorldFlags:1251,
// Player.Serialize:54909). A reproducible fixture cannot carry the wall clock,
// so extract.ps1 substitutes `DateTime.UtcNow` -> `FixtureClock.UtcNow` in the
// extracted bodies (the ONLY substitution). The value is pinned to a fixed
// instant; the Rust test reads the same instant back and re-writes it.

using System;

namespace Terraria
{
    public static class FixtureClock
    {
        // 2026-11-18T00:00:00Z, a fixed UTC instant (DateTimeKind.Utc so
        // ToBinary round-trips identically on read).
        public static readonly DateTime UtcNow =
            new DateTime(2026, 11, 18, 0, 0, 0, DateTimeKind.Utc);
    }
}
