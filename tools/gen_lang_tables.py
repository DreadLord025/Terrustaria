#!/usr/bin/env python3
"""Generate the localization id tables from the decompiled C# sources.

The C# builds its name caches by reflecting over the id classes and reading
the localization key `<Category>.<ConstName>` (`Lang.FillNameCacheArray`,
`Lang.cs:450-464`), so the key spelling IS the id class's constant name.
The port needs the same spelling for the id tables it does not carry yet
(`NPCID` and `BuffID`), plus the map legend table `Lang.BuildMapAtlas`
builds (`Lang.cs:554-...`).

Usage:
    python tools/gen_lang_tables.py            # write crates/tr-app/src/lang_tables.rs
    python tools/gen_lang_tables.py --check    # fail if the file is stale
"""
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DECOMPILED = os.path.join(ROOT, "decompiled", "Terraria")
OUT = os.path.join(ROOT, "crates", "tr-app", "src", "lang_tables.rs")

NPCID_CS = os.path.join(DECOMPILED, "ID", "NPCID.cs")
BUFFID_CS = os.path.join(DECOMPILED, "ID", "BuffID.cs")
LANG_CS = os.path.join(DECOMPILED, "Lang.cs")


def const_table(path, pattern):
    """The `(id, const name)` pairs of an id class, in the source order."""
    src = open(path, encoding="utf-8").read()
    pairs = []
    for m in re.finditer(pattern, src):
        pairs.append((int(m.group(2)), m.group(1)))
    return pairs


def map_legend():
    """The `Lang.BuildMapAtlas` writes: `(tile, option, key)` triples.

    The C# writes `MapHelper.TileToLookup(tile, option)` slots; the item-name
    rows point at the item cache (`itemNameCache[N]`), the rest at a
    `MapObject.*` key. The tree-trunk loop and the pot loop are expanded
    here (the tree set lives in `tr-core::tileobject::is_a_tree_trunk`).
    """
    src = open(LANG_CS, encoding="utf-8").read()
    start = src.index("public static void BuildMapAtlas()")
    body = src[start: src.index("\n\tpublic static", start + 10)]
    rows = []
    for m in re.finditer(
        r'_mapLegendCache\[MapHelper\.TileToLookup\((\d+), (\d+)\)\] = '
        r'(?:Language\.GetText\("([^"]+)"\)|itemNameCache\[(\d+)\]);',
        body,
    ):
        tile, option, key, item = m.groups()
        if key is not None:
            rows.append((int(tile), int(option), "key", key))
        else:
            rows.append((int(tile), int(option), "item", item))
    # The pot loop: nine options on tiles 28 and 653.
    for m in re.finditer(
        r'_mapLegendCache\[MapHelper\.TileToLookup\((\d+), k\)\] = Language\.GetText\("([^"]+)"\);',
        body,
    ):
        tile, key = m.groups()
        for option in range(9):
            rows.append((int(tile), option, "key", key))
    return rows


def net_id_map():
    """`NPCID.NetIdMap` (`NPCID.cs:12095-12104`) — the negative net id -> the
    positive NPC id table `NPCID.FromNetId` reads."""
    src = open(NPCID_CS, encoding="utf-8").read()
    m = re.search(r"NetIdMap = new int\[\d+\]\s*\{([^}]*)\}", src, re.S)
    if not m:
        raise SystemExit("NPCID.NetIdMap not found")
    return [int(x) for x in re.findall(r"-?\d+", m.group(1))]


def emit(pairs, rows, net_map):
    out = []
    out.append("//! The localization id tables (GENERATED — `tools/gen_lang_tables.py`).")
    out.append("//!")
    out.append("//! The C# builds its name caches by reflecting over the id classes and")
    out.append("//! reading the key `<Category>.<ConstName>` (`Lang.FillNameCacheArray`,")
    out.append("//! `Lang.cs:450-464`), so the constant spelling IS the localization key.")
    out.append("//! Do not hand-edit: run `python tools/gen_lang_tables.py`.")
    out.append("")
    out.append("/// `NPCID`'s constants verbatim — `(id, const name)`; the negative ids")
    out.append("/// are the `NPCID.FromNetId` slime/hornet aliases (`NPCID.cs`).")
    out.append("pub const NPC_ID: &[(i32, &str)] = &[")
    for id_, name in pairs[0]:
        out.append('    (%d, "%s"),' % (id_, name))
    out.append("];")
    out.append("")
    out.append("/// `BuffID`'s constants verbatim — `(id, const name)` (`BuffID.cs`).")
    out.append("pub const BUFF_ID: &[(i32, &str)] = &[")
    for id_, name in pairs[1]:
        out.append('    (%d, "%s"),' % (id_, name))
    out.append("];")
    out.append("")
    out.append("/// The map legend rows — `Lang.BuildMapAtlas` (`Lang.cs:554-...`).")
    out.append("///")
    out.append("/// `(tile, option, kind, value)`: `kind` is `\"key\"` for a")
    out.append("/// `MapObject.*` localization key and `\"item\"` for the C#'s")
    out.append("/// `itemNameCache[N]` row (the item id).")
    out.append("pub const MAP_LEGEND: &[(u16, u8, &str, &str)] = &[")
    for tile, option, kind, value in rows:
        out.append('    (%d, %d, "%s", "%s"),' % (tile, option, kind, value))
    out.append("];")
    out.append("")
    out.append("/// `NPCID.NetIdMap` (`NPCID.cs:12095-12104`) — the negative net id")
    out.append("/// to the positive NPC id map `NPCID.FromNetId` reads.")
    out.append("pub const NPC_NET_ID_MAP: &[i32] = &[")
    for i in range(0, len(net_map), 10):
        out.append("    " + ", ".join(str(x) for x in net_map[i : i + 10]) + ",")
    out.append("];")
    out.append("")
    return "\n".join(out)


def main():
    npc = const_table(NPCID_CS, r"public const short (\w+) = (-?\d+);")
    buff = const_table(BUFFID_CS, r"public const int (\w+) = (-?\d+);")
    rows = map_legend()
    net_map = net_id_map()
    text = emit((npc, buff), rows, net_map)
    if "--check" in sys.argv:
        current = open(OUT, encoding="utf-8").read() if os.path.exists(OUT) else ""
        if current != text:
            print("lang_tables.rs is stale — run python tools/gen_lang_tables.py")
            return 1
        print("lang_tables.rs is up to date")
        return 0
    with open(OUT, "w", encoding="utf-8", newline="\n") as f:
        f.write(text)
    print(
        "wrote %s (NPC_ID %d, BUFF_ID %d, MAP_LEGEND %d)"
        % (OUT, len(npc), len(buff), len(rows))
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
