# -*- coding: utf-8 -*-
"""M7-WIRE-2: the special-device emissions (443 geyser, 210 mine, 531 barrel,
141 explosive lock) into tr-core/src/wiring.rs."""
fp = r"C:/Users/DreadLord/Desktop/TR-wire2/crates/tr-core/src/wiring.rs"
with open(fp, "rb") as f:
    s = f.read().decode("utf-8")
NL = "\r\n" if "\r\n" in s else "\n"

def sub(old, new, cnt=1):
    global s
    o = old.replace("\n", NL)
    n = new.replace("\n", NL)
    c = s.count(o)
    assert c == cnt, "expected %d, found %d of %r" % (cnt, c, o[:60])
    s = s.replace(o, n)

# 1. TrapRequest gains the spawn-family variants (the app drops what it
# cannot build yet).
sub("""pub enum TrapRequest {
    Shot {
        x: f32,
        y: f32,
        vx: f32,
        vy: f32,
        kind: u16,
        damage: i32,
    },""",
"""pub enum TrapRequest {
    Shot {
        x: f32,
        y: f32,
        vx: f32,
        vy: f32,
        kind: u16,
        damage: i32,
    },
    /// `Wiring.GeyserTrap` (`:3084-3112`): the trap wall (443) fires the
    /// tracer projectile 654 (20 dmg, kb 2) — downward when `frameX / 36 >=
    /// 2`, upward otherwise, from the anchor column.
    Geyser { x: f32, y: f32, vx: f32, vy: f32, kind: u16, damage: i32 },
    /// `Wiring.ExplodeMine` (`:3074-3082`): the trigger mine (210) kills
    /// its tile (no item) and spawns the land mine 164 (250 dmg, kb 10) at
    /// rest.
    Mine { x: f32, y: f32, kind: u16, damage: i32, knock_back: f32 },
    /// `HitWireSingle`'s case 531 (`:1994-2015`): the trap boulder — the
    /// 2-wide frame decodes the top-left anchor, `CheckMech(900)`, then the
    /// boulder 99 (70 dmg, kb 10) rests under the top row.
    Boulder { x: f32, y: f32, kind: u16, damage: i32, knock_back: f32 },
    /// `HitWireSingle`'s case 141 (`:2035-2040`): the explosive lock kills
    /// its tile (no item) and spawns the bomb 108 (500 dmg, kb 10) at rest.
    ExplosiveLock { x: f32, y: f32, kind: u16, damage: i32, knock_back: f32 },""")

with open(fp, "wb", encoding="f" and "utf-8") as f:
    f.write(s.encode("utf-8"))
print("TrapRequest extended")
