"""Generate `crates/tr-core/src/mapdata.rs` from `MapHelper.Initialize()`.

The vanilla tile/wall map colours are a ~1450-line run of assignments in
`decompiled/Terraria/Map/MapHelper.cs` (`Color[][] array` for tiles,
`Color[][] array4` for walls). They are extracted mechanically here so the Rust
port can never drift from the decompiled source. `mapdata.rs` is generated:
never hand-edit it, re-run this script instead.

Supported statement forms (everything the source actually uses before the
`array5` gradient block; the parser fails loudly on anything it does not know so
a source change cannot be silently dropped):

  * `Color name = new Color(r, g, b);`          local
  * `name = new Color(r, g, b);`                local
  * `name = array[id][v];`                      local from a table cell
  * `array[id][v] = new Color(r, g, b);`        table cell
  * `array[id][v] = name;`                      table cell from a local
  * `array[id][v] = array[id2][v2];`            table cell copy
  * `array[id][v] = Color.Black|Gray|LightGray;` named XNA colours
  * `array[id][v] = MapHelper.MultiplyMapColor(cell, factor);`
  * `Color[] tmp = array[row]; ... for (...) tmp[i] = array[src][i];`  row copy
  * `for (int i = 0; i < array[row].Length; i++) array[dst][i] = array[src][i];`
  * `array4[...]` equivalents for wall colours
  * `float name = 0.5f;`                        multiplier scalar

Ignored: the `array`/`array4` declarations + allocation loops, the
`array3`/`array5` gradient blocks, and the trailing `Lang.BuildMapAtlas()`.
"""
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parent.parent
MAPHELPER = ROOT / "decompiled/Terraria/Map/MapHelper.cs"
TILEID = ROOT / "decompiled/Terraria/ID/TileID.cs"
WALLID = ROOT / "decompiled/Terraria/ID/WallID.cs"
OUT = ROOT / "crates/tr-core/src/mapdata.rs"

TILE_VARIATIONS = 13  # Color[] array = new Color[13]
WALL_VARIATIONS = 2  # Color[] array4 = new Color[2]

# XNA named colours used by the source (Alpha 255).
NAMED_COLORS = {
    "Black": (0, 0, 0, 255),
    "Gray": (128, 128, 128, 255),
    "LightGray": (211, 211, 211, 255),
}

TRANSPARENT = (0, 0, 0, 0)


def fail(msg):
    raise SystemExit("gen_map_colors: " + msg)


def read_lines(path):
    return path.read_text(encoding="utf-8").split("\n")


def read_count(path, prefix, label):
    """Read `Prefix.Count = <n>;` / `public const ushort Count = <n>;`."""
    text = path.read_text(encoding="utf-8")
    m = re.search(r"%s\.Count = (\d+);" % re.escape(prefix), text)
    if not m:
        m = re.search(r"public const ushort Count = (\d+);", text)
    if not m:
        fail("could not find %s count in %s" % (label, path))
    return int(m.group(1))


def multiply(color, factor):
    """Port of `MapHelper.MultiplyMapColor` (MapHelper.cs:1791).

    `color *= multiplier` is XNA's `Color * float` (each channel truncated to
    an int), then the original alpha is restored.
    """
    r, g, b, a = color
    return (int(r * factor), int(g * factor), int(b * factor), a)


def main():
    tile_count = read_count(TILEID, "TileID", "tile")
    wall_count = read_count(WALLID, "WallID", "wall")
    if tile_count != 754:
        fail("TileID.Count is %d, expected 754" % tile_count)
    if wall_count != 367:
        fail("WallID.Count is %d, expected 367" % wall_count)

    lines = read_lines(MAPHELPER)
    start = next(
        i for i, l in enumerate(lines) if l.strip().startswith("public static void Initialize()")
    )
    stop = next(i for i, l in enumerate(lines) if "Color[] array5 = new Color[256];" in l)

    tables = {
        "array": [[list(TRANSPARENT) for _ in range(TILE_VARIATIONS)] for _ in range(tile_count)],
        "array4": [[list(TRANSPARENT) for _ in range(WALL_VARIATIONS)] for _ in range(wall_count)],
    }
    widths = {"array": TILE_VARIATIONS, "array4": WALL_VARIATIONS}
    locals_ = {}
    scalars = {}
    aliases = {}

    def cell(name, row, v):
        return tuple(tables[name][row][v])

    def set_cell(name, row, v, value):
        tables[name][row][v] = list(value)

    def resolve_local(name):
        if name not in locals_:
            fail("local %r used before assignment" % name)
        return locals_[name]

    def loop_count(bound):
        m = re.fullmatch(r"(array\d*)\[\d+\]\.Length", bound)
        if m:
            return widths[m.group(1)]
        m = re.fullmatch(r"(\w+)\.Length", bound)
        if m and m.group(1) in aliases:
            return widths[aliases[m.group(1)][0]]
        if re.fullmatch(r"\d+", bound):
            return int(bound)
        if bound == "TileID.Count":
            return tile_count
        if bound == "WallID.Count":
            return wall_count
        fail("cannot evaluate loop bound %r" % bound)

    def run_loop_body(body, var, idx):
        for stmt in body:
            m = re.fullmatch(
                r"(array\d*)\[(\d+)\]\[%s\] = (array\d*)\[(\d+)\]\[%s\];" % (var, var), stmt
            )
            if m:
                dst, drow, src, srow = m.group(1), int(m.group(2)), m.group(3), int(m.group(4))
                set_cell(dst, drow, idx, cell(src, srow, idx))
                continue
            m = re.fullmatch(r"(\w+)\[%s\] = (array\d*)\[(\d+)\]\[%s\];" % (var, var), stmt)
            if m:
                alias, src, srow = m.group(1), m.group(2), int(m.group(3))
                if alias not in aliases:
                    fail("unknown row alias %r in loop body" % alias)
                dst, drow = aliases[alias]
                set_cell(dst, drow, idx, cell(src, srow, idx))
                continue
            fail("unsupported loop body statement: %r" % stmt)

    i = start + 1
    while i < stop:
        t = lines[i].strip()
        if t == "" or t in ("{", "}"):
            i += 1
            continue

        # Gradient palettes (array3 at 1272, then array5/array6/array7 later).
        if t.startswith("Color[] array3"):
            while i < stop and lines[i].strip() != "};":
                i += 1
            i += 1
            continue

        # Declarations and allocation loops: ignore.
        if re.fullmatch(r"Color\[\]\[\] array\d* = new Color\[[^\]]+\]\[\];", t):
            i += 1
            continue
        if re.fullmatch(r"array\d*\[num\d*\] = new Color\[\d+\];", t):
            i += 1
            continue

        m = re.fullmatch(r"float (\w+) = ([\d.]+)f;", t)
        if m:
            scalars[m.group(1)] = float(m.group(2))
            i += 1
            continue

        m = re.fullmatch(r"(\w+) = (array\d*)\[(\d+)\]\[(\d+)\];", t)
        if m:
            locals_[m.group(1)] = cell(m.group(2), int(m.group(3)), int(m.group(4)))
            i += 1
            continue

        m = re.fullmatch(r"Color (\w+) = new Color\((\d+), (\d+), (\d+)\);", t)
        if m:
            locals_[m.group(1)] = (int(m.group(2)), int(m.group(3)), int(m.group(4)), 255)
            i += 1
            continue

        m = re.fullmatch(r"(\w+) = new Color\((\d+), (\d+), (\d+)\);", t)
        if m:
            locals_[m.group(1)] = (int(m.group(2)), int(m.group(3)), int(m.group(4)), 255)
            i += 1
            continue

        m = re.fullmatch(r"Color\[\] (\w+) = (array\d*)\[(\d+)\];", t)
        if m:
            aliases[m.group(1)] = (m.group(2), int(m.group(3)))
            i += 1
            continue

        m = re.fullmatch(r"(\w+) = (array\d*)\[(\d+)\];", t)
        if m:
            # Row-alias reassignment (`array2 = array[648];`).
            if m.group(1) not in aliases:
                fail("unknown row alias %r at line %d" % (m.group(1), i + 1))
            aliases[m.group(1)] = (m.group(2), int(m.group(3)))
            i += 1
            continue

        m = re.fullmatch(r"for \(int (\w+) = 0; \w+ < (.+?); \w+\+\+\)", t)
        if m:
            var, bound = m.group(1), m.group(2)
            if lines[i + 1].strip() != "{":
                fail("loop at line %d has no braced body" % (i + 1))
            j = i + 2
            body = []
            while lines[j].strip() != "}":
                body.append(lines[j].strip())
                j += 1
            j += 1
            # Allocation loop (`array[i] = new Color[13];`) -> ignore.
            if len(body) == 1 and re.fullmatch(
                r"array\d*\[%s\] = new Color\[\d+\];" % re.escape(var), body[0]
            ):
                i = j
                continue
            for idx in range(loop_count(bound)):
                run_loop_body(body, var, idx)
            i = j
            continue

        # Table cell assignments.
        m = re.fullmatch(
            r"(array\d*)\[(\d+)\]\[(\d+)\] = new Color\((\d+), (\d+), (\d+)\);", t
        )
        if m:
            set_cell(
                m.group(1),
                int(m.group(2)),
                int(m.group(3)),
                (int(m.group(4)), int(m.group(5)), int(m.group(6)), 255),
            )
            i += 1
            continue

        m = re.fullmatch(
            r"(array\d*)\[(\d+)\]\[(\d+)\] = (array\d*)\[(\d+)\]\[(\d+)\];", t
        )
        if m:
            set_cell(m.group(1), int(m.group(2)), int(m.group(3)),
                     cell(m.group(4), int(m.group(5)), int(m.group(6))))
            i += 1
            continue

        m = re.fullmatch(r"(array\d*)\[(\d+)\]\[(\d+)\] = Color\.(\w+);", t)
        if m:
            named = m.group(4)
            if named not in NAMED_COLORS:
                fail("unknown named colour Color.%s at line %d" % (named, i + 1))
            set_cell(m.group(1), int(m.group(2)), int(m.group(3)), NAMED_COLORS[named])
            i += 1
            continue

        m = re.fullmatch(
            r"(array\d*)\[(\d+)\]\[(\d+)\] = MapHelper\.MultiplyMapColor\("
            r"(array\d*)\[(\d+)\]\[(\d+)\], ([\d.]+)f\);",
            t,
        )
        if m:
            set_cell(m.group(1), int(m.group(2)), int(m.group(3)),
                     multiply(cell(m.group(4), int(m.group(5)), int(m.group(6))), float(m.group(7))))
            i += 1
            continue

        m = re.fullmatch(
            r"(array\d*)\[(\d+)\]\[(\d+)\] = MapHelper\.MultiplyMapColor\("
            r"(array\d*)\[(\d+)\]\[(\d+)\], (\w+)\);",
            t,
        )
        if m:
            factor = m.group(7)
            if factor not in scalars:
                fail("unknown multiplier %r at line %d" % (factor, i + 1))
            set_cell(m.group(1), int(m.group(2)), int(m.group(3)),
                     multiply(cell(m.group(4), int(m.group(5)), int(m.group(6))), scalars[factor]))
            i += 1
            continue

        m = re.fullmatch(r"(array\d*)\[(\d+)\]\[(\d+)\] = (\w+);", t)
        if m:
            set_cell(m.group(1), int(m.group(2)), int(m.group(3)), resolve_local(m.group(4)))
            i += 1
            continue

        fail("unhandled statement at line %d: %r" % (i + 1, t))

    defined_tiles = sum(
        1 for row in tables["array"] for c in row if tuple(c) != TRANSPARENT
    )
    defined_walls = sum(
        1 for row in tables["array4"] for c in row if tuple(c) != TRANSPARENT
    )

    def rgba(c):
        return "[%d, %d, %d, %d]" % (c[0], c[1], c[2], c[3])

    out = []
    out.append("// @generated by tools/gen_map_colors.py — do not edit\n")
    out.append("//! Vanilla map colour tables, extracted mechanically from\n")
    out.append("//! `decompiled/Terraria/Map/MapHelper.cs` (`MapHelper.Initialize()`).\n")
    out.append("//!\n")
    out.append("//! Each entry is `[r, g, b, a]`. `a == 0` marks a slot the source never\n")
    out.append("//! assigned (transparent), mirroring `Color.Transparent`; every defined\n")
    out.append("//! entry has `a == 255`.\n\n")
    out.append(
        "/// Tile map colours, `TILE_COLORS[tileId][variation]` "
        "(`TileID.Count` x %d).\n" % TILE_VARIATIONS
    )
    out.append("/// Source: `MapHelper.Initialize`, `Color[][] array`.\n")
    out.append("pub const TILE_COLORS: [[[u8; 4]; %d]; %d] = [\n" % (TILE_VARIATIONS, tile_count))
    for row in tables["array"]:
        out.append("    [%s],\n" % ", ".join(rgba(c) for c in row))
    out.append("];\n\n")
    out.append(
        "/// Wall map colours, `WALL_COLORS[wallId][variation]` "
        "(`WallID.Count` x %d).\n" % WALL_VARIATIONS
    )
    out.append("/// Source: `MapHelper.Initialize`, `Color[][] array4`.\n")
    out.append("pub const WALL_COLORS: [[[u8; 4]; %d]; %d] = [\n" % (WALL_VARIATIONS, wall_count))
    for row in tables["array4"]:
        out.append("    [%s],\n" % ", ".join(rgba(c) for c in row))
    out.append("];\n\n")
    out.append("/// Canonical definitions live in [`crate::tiledata`] (`TileID.Count` / `WallID.Count`).\n")
    out.append("pub use crate::tiledata::{TILE_COUNT, WALL_COUNT};\n\n")
    out.append(TESTS)
    OUT.write_text("".join(out), encoding="utf-8", newline="\n")

    print("defined tile entries: %d" % defined_tiles)
    print("defined wall entries: %d" % defined_walls)
    print("wrote %s (%d tile rows, %d wall rows)" % (OUT, tile_count, wall_count))


TESTS = '''#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tile_colors_are_vanilla() {
        assert_eq!(TILE_COLORS[0][0], [151, 107, 75, 255]);
        assert_eq!(TILE_COLORS[1][0], [128, 128, 128, 255]);
        assert_eq!(TILE_COLORS[2][0], [28, 216, 94, 255]);
        assert_eq!(TILE_COLORS[7][0], [150, 67, 22, 255]);
        assert_eq!(TILE_COLORS[8][0], [185, 164, 23, 255]);
        assert_eq!(TILE_COLORS[9][0], [185, 194, 195, 255]);
    }

    #[test]
    fn absent_entries_are_zero_alpha() {
        assert_eq!(TILE_COLORS[0][1][3], 0);
    }

    #[test]
    fn aliased_row_copies_match_their_source() {
        // MapHelper.cs:961-981 — `Color[] array2 = array[647];` style copies.
        assert_eq!(TILE_COLORS[647][0], TILE_COLORS[186][0]);
        assert_eq!(TILE_COLORS[648][0], TILE_COLORS[187][0]);
    }

    #[test]
    fn direct_row_copies_match_their_source() {
        // MapHelper.cs:462-469 and :898-901.
        assert_eq!(TILE_COLORS[628][0], TILE_COLORS[627][0]);
        assert_eq!(TILE_COLORS[692][0], TILE_COLORS[627][0]);
        assert_eq!(TILE_COLORS[693][0], TILE_COLORS[165][0]);
    }

    #[test]
    fn multiplied_tile_colors_are_scaled() {
        // MapHelper.cs:1269 — MultiplyMapColor(array[138][0] = (128,128,128), 0.95f).
        assert_eq!(TILE_COLORS[749][0], [121, 121, 121, 255]);
    }

    #[test]
    fn multiplied_wall_colors_are_scaled() {
        // MapHelper.cs:1659 and :1648 — MultiplyMapColor(..., 0.5f).
        assert_eq!(WALL_COLORS[366][0], [60, 60, 60, 255]);
        assert_eq!(WALL_COLORS[353][0], [13, 54, 34, 255]);
    }

    #[test]
    fn table_dimensions() {
        assert_eq!(TILE_COUNT, 754);
        assert_eq!(WALL_COUNT, 367);
    }
}
'''


if __name__ == "__main__":
    main()
