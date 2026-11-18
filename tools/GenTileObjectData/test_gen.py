#!/usr/bin/env python3
"""GenTileObjectData test suite — S1-F-7 wave A acceptance checks.

Run:  python tools/GenTileObjectData/test_gen.py   (exit 0 = all pass)

Every pinned value carries its C# citation; the fixtures are interpreted from
the real decompiled `TileObjectData.cs Initialize` through the same pipeline the
generator uses.
"""

import hashlib
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
sys.path.insert(0, str(HERE))
import gen  # noqa: E402

SRC = ROOT / "decompiled" / "Terraria" / "ObjectData" / "TileObjectData.cs"
COMMITTED = ROOT / "crates" / "tr-core" / "src" / "tileobject" / "registry.rs"


def fail(msg):
    print("FAIL:", msg)
    sys.exit(1)


def check(cond, msg):
    if not cond:
        fail(msg)


def interp():
    return gen.parse_initialize(SRC.read_text(encoding="utf-8"))


# ---------------------------------------------------------------------------
# S1-F-7.1 — idempotent regen (generate twice, byte-identical output)
# ---------------------------------------------------------------------------
def test_idempotent_regen():
    a, _ = gen.generate(ROOT)
    b, _ = gen.generate(ROOT)
    ha = hashlib.sha256(a.encode("utf-8")).hexdigest()
    hb = hashlib.sha256(b.encode("utf-8")).hexdigest()
    check(ha == hb, "regen changed the output: %s != %s" % (ha, hb))
    print("ok S1-F-7.1 idempotent regen: sha", ha[:16])


# ---------------------------------------------------------------------------
# S1-F-7.2 — the committed table is current (the --check round-trip)
# ---------------------------------------------------------------------------
def test_committed_table_is_current():
    check(COMMITTED.exists(), "committed registry.rs is missing: %s" % COMMITTED)
    before = COMMITTED.read_text(encoding="utf-8")
    fresh, _ = gen.generate(ROOT)
    check(before == fresh, "the committed registry.rs drifted from a fresh generation")
    # The shipping --check path agrees (read-only).
    rc = subprocess.call([sys.executable, str(HERE / "gen.py"), str(ROOT), "--check"])
    check(rc == 0, "gen.py --check reported drift (rc=%d)" % rc)
    after = COMMITTED.read_text(encoding="utf-8")
    check(before == after, "--check rewrote the committed file (must be read-only)")
    print("ok S1-F-7.2 committed registry.rs is current (%d bytes)" % len(before))


# ---------------------------------------------------------------------------
# S1-F-7.3 — the registry covers every addTile kind
# ---------------------------------------------------------------------------
def test_coverage():
    i = interp()
    registered = [k for k in range(gen.TILE_COUNT) if i.data[k] is not None]
    check(len(registered) == 389, "389 registered tiles, got %d" % len(registered))
    # The platform prelude + the 435..439 loop.
    for k in [19, 427, 435, 436, 437, 438, 439]:
        check(i.data[k] is not None, "platform %d registered" % k)
    # A few unregistered kinds.
    for k in [0, 1, 2, 221, 380]:
        check(i.data[k] is None, "kind %d is not a tile object" % k)
    print("ok S1-F-7.3 coverage (389 tiles)")


# ---------------------------------------------------------------------------
# S1-F-7.4 — per-family spot values, hand-verified vs the C# chains
# ---------------------------------------------------------------------------
def test_spot_families():
    i = interp()
    d = i.data

    # Platforms (prelude :2127-2160): multiplier/wrap 27, LavaDeath fold.
    p = d[19]
    check((p.get("Width"), p.get("Height")) == (1, 1), "platform size")
    check(p.get("StyleMultiplier") == 27 and p.get("StyleWrapLimit") == 27, "platform 27/27")
    check(p.get("LavaPlacement") == 1, "platform LavaDeath fold (:2134)")
    check(p.uses_custom is False, "platform UsesCustomCanPlace=false (:2133)")

    # Torches (StyleTorch :4577-4628): 20-px cell, 6/6, flatten, water/lava death.
    t = d[4]
    check(t.get("CoordinateWidth") == 20 and t.get("CoordinateHeights") == [20], "torch 20-px cell")
    check(t.get("StyleMultiplier") == 6 and t.get("StyleWrapLimit") == 6, "torch 6/6")
    check(t.get("FlattenAnchors") is True, "torch FlattenAnchors (:4580)")
    check(t.get("WaterDeath") is True and t.get("LavaDeath") is True, "torch deaths (:4590-4591)")
    check(len(t.modules["alternates"]["data"]) == 3, "torch 3 alternates (:4596-4606)")
    check(t.modules["alternates"]["data"][2].get("AnchorWall") is True, "torch wall alternate")

    # Closed door (:2223-2247): wrap 36, line skip 3, 2 click-point alternates.
    c = d[10]
    check(c.get("StyleWrapLimit") == 36 and c.get("StyleLineSkip") == 3, "door 36/3")
    check(c.get("StyleHorizontal") is False, "door vertical styles (:2223)")
    check(len(c.modules["alternates"]["data"]) == 2, "door 2 alternates")

    # Open door (:2248-2291): 2 wide, line skip 2, PlaceRight, 5 alternates.
    o = d[11]
    check((o.get("Width"), o.get("Height")) == (2, 3), "open door 2x3")
    check(o.get("StyleLineSkip") == 2, "open door line skip 2 (:2260)")
    check(o.get("Direction") == 2, "open door PlaceRight (:2261)")
    check(len(o.modules["alternates"]["data"]) == 5, "open door 5 alternates")

    # Chair (Style1x2 + overrides :4141-4173): wrap/mult 2, padding fix (0,2).
    ch = d[15]
    check(ch.get("StyleWrapLimit") == 2 and ch.get("StyleMultiplier") == 2, "chair 2/2")
    check(ch.get("CoordinatePaddingFix") == (0, 2), "chair padding fix (:4146)")
    check(ch.get("Direction") == 1, "chair PlaceLeft (:4143)")
    check(ch.modules["alternates"]["data"][0].get("Style") == 1, "chair PlaceRight alternate style 1")
    # Sub-tile OWN alternates (GetTileData indexes the resolved sub-tile's
    # alternates, TileObjectData.cs:5148-5157): chair sub16 (Obsidian) and
    # torch sub8 carry their PlaceRight / wall-anchor alternates.
    check(ch.modules["subTiles"]["data"][16].modules["alternates"]["data"][0].get("Direction") == 2,
          "chair sub16 PlaceRight alternate (:4151-4155)")
    check(ch.modules["subTiles"]["data"][16].modules["alternates"]["data"][0].get("Style") == 1,
          "chair sub16 alternate style 1")
    check(len(t.modules["subTiles"]["data"][8].modules["alternates"]["data"]) == 3,
          "torch sub8 3 alternates (:4596-4606 via CopyFrom)")
    check(t.modules["subTiles"]["data"][8].modules["alternates"]["data"][2].get("AnchorWall") is True,
          "torch sub8 wall-anchor alternate")

    # Chest (Style2x2 + overrides :3461-3470): 18-px bottom, no Table anchor.
    chest = d[21]
    check(chest.get("CoordinateHeights") == [16, 18], "chest 18-px bottom (:3463)")
    check(chest.get("AnchorBottom") == (1 | 2 | 8, 2, 0), "chest bottom anchor without Table (:3469)")
    check(chest.get("LavaPlacement") == 1, "chest LavaDeath=false keeps NotAllowed (:3468)")
    check(chest.modules["hooks"]["check"][1] == "Chest.FindEmptyChest", "chest hook name (:3464)")

    print("ok S1-F-7.4 spot families (platforms/torches/doors/chairs/chests)")


# ---------------------------------------------------------------------------
# S1-F-7.5 — the C# bugs are reproduced (FullCopyFrom empties sub-tiles)
# ---------------------------------------------------------------------------
def test_csharp_bugs_reproduced():
    i = interp()
    d = i.data
    # FullCopyFrom's sub-tile copy ctor loops over the FRESH list's Count (0):
    # 83/84 (FullCopyFrom 82/83, :4785-4788) and 389 (FullCopyFrom 388, :2323)
    # resolve NO sub-tiles in vanilla.
    for k in [83, 84, 389]:
        subs = d[k].modules["subTiles"]["data"] or []
        check(all(s is None for s in subs), "kind %d FullCopyFrom empties sub-tiles" % k)
    # 82 itself keeps its 6 sub-tiles.
    sub82 = [s for s in d[82].modules["subTiles"]["data"] if s is not None]
    check(len(sub82) == 6, "herb 82 keeps 6 sub-tiles")
    # The AnchorAlternateTiles setter compares against tileInvalid (:449) — a
    # no-op write when the alternate list equals the INVALID list. The herb
    # chain sets AnchorAlternateTiles {78} after a CopyFrom whose tileInvalid is
    # null, so the write lands (78 != null). Verify the herb alternate list.
    check(d[82].get("AnchorAlternateTiles") == [78], "herb alternate tiles {78}")
    print("ok S1-F-7.5 C# bugs reproduced (FullCopyFrom sub-tiles)")


# ---------------------------------------------------------------------------
# S1-F-7.6 — the emitted table is acyclic (Rust statics) and well-shaped
# ---------------------------------------------------------------------------
def test_acyclic_and_shape():
    text, stats = gen.generate(ROOT)
    check(stats["tiles"] == 389, "389 tiles emitted")
    # Every top-level def's and sub-tile's alternates/sub-tiles reference
    # emitted statics; alternate nodes are leaves (their deeper nested lists
    # are the C# shared-reference cycles no consumer reads). Verify the
    # emitted reference graph is acyclic.
    import re
    # Parse each static block and its alternates/subtiles refs.
    blocks = re.split(r"\nstatic (D_\w+): TileObjectDef = TileObjectDef \{", text)
    # blocks[0] is the header; then alternating name, body.
    refs = {}
    for j in range(1, len(blocks), 2):
        name = blocks[j]
        body = blocks[j + 1].split("\n};", 1)[0]
        alts = re.search(r"alternates: &\[([^\]]*)\]", body)
        subs = re.search(r"subtiles: &\[([^\]]*)\]", body)
        r = set()
        if alts:
            r.update(x.strip() for x in alts.group(1).split(",") if x.strip())
        if subs:
            r.update(m.group(1) for m in re.finditer(r"&(\w+)", subs.group(1)))
        refs[name] = r
    # DFS cycle check over the reference graph.
    WHITE, GRAY, BLACK = 0, 1, 2
    color = {n: WHITE for n in refs}
    cyc = []

    def dfs(n):
        color[n] = GRAY
        for c in refs.get(n, ()):
            if color.get(c) == GRAY:
                cyc.append((n, c))
            elif color.get(c) == WHITE:
                dfs(c)
        color[n] = BLACK

    sys.setrecursionlimit(100000)
    for n in refs:
        if color[n] == WHITE:
            dfs(n)
    check(not cyc, "the emitted registry has a reference cycle: %s" % cyc[:5])
    print("ok S1-F-7.6 acyclic registry (%d defs)" % len(refs))


def main():
    test_idempotent_regen()
    test_committed_table_is_current()
    test_coverage()
    test_spot_families()
    test_csharp_bugs_reproduced()
    test_acyclic_and_shape()
    print("ALL GEN TESTS PASSED")


if __name__ == "__main__":
    main()
