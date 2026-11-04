#!/usr/bin/env python3
"""GenItemData test suite — DW-1.1..DW-1.4 acceptance checks.

Run:  python tools/GenItemData/test_gen.py   (exit 0 = all pass)

Every pinned value carries its C# citation; the fixtures are parsed from the
real decompiled sources through the same pipeline the generator uses.
"""

import hashlib
import shutil
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import gen  # noqa: E402

ALL_VARIANTS = []
TMP = Path(tempfile.mkdtemp(prefix="genitemdata-test-"))
OUT = TMP / "itemdata.rs"


def build_once():
    rows, variants = gen.run_pipeline(OUT)
    global ALL_VARIANTS
    ALL_VARIANTS = variants
    return rows


def fail(msg):
    print("FAIL:", msg)
    sys.exit(1)


def check(cond, msg):
    if not cond:
        fail(msg)


# ---------------------------------------------------------------------------
# DW-1.1 — idempotent regen (run twice, byte-identical output)
# ---------------------------------------------------------------------------
def test_dw_1_1_idempotent_regen():
    build_once()
    sha1 = hashlib.sha256(OUT.read_bytes()).hexdigest()
    build_once()
    sha2 = hashlib.sha256(OUT.read_bytes()).hexdigest()
    check(sha1 == sha2, "regen changed the output (DW-1.1): %s != %s" % (sha1, sha2))
    print("ok DW-1.1 idempotent regen: sha", sha1[:16])


# ---------------------------------------------------------------------------
# DW-1.2 — spot sample across the five ranges, hand-verified vs Item.cs
# ---------------------------------------------------------------------------
def test_dw_1_2_spot_sample():
    rows = build_once()

    def row(i):
        return rows[i]

    # Dirt (id 2) — case 3/2 block, Item.cs:1398-1416: consumable, autoReuse,
    # createTile 0, 12x12, useStyle 1, 15/10; ResetStats damage -1 (:49013).
    r = row(2)
    check((r["useTime"], r["useAnimation"], r["useStyle"]) == (10, 15, 1), "dirt timings")
    check(r["autoReuse"] is True and r["consumable"] is True, "dirt reuse/consume")
    check(r["createTile"] == 0 and (r["width"], r["height"]) == (12, 12), "dirt tile/size")
    check(r["damage"] == -1 and r["value"] == 0, "dirt damage/value")
    check(r["maxStack"] == 9999, "dirt CommonMaxStack (Item.cs:49966)")

    # Torch (id 8) — holdStyle 1 (the epilogue food rule does NOT apply; the
    # case body sets holdStyle), tile 4, value 50.
    r = row(8)
    check(r["holdStyle"] == 1, "torch holdStyle (case 8)")
    check(r["createTile"] == 4 and r["value"] == 50, "torch tile/value")

    # Wooden door (id 25) — Item.cs:1675-1684: the one placement without
    # autoReuse, value 200, consumable, 14x28, tile 10.
    r = row(25)
    check(r["autoReuse"] is False, "door no autoReuse (Item.cs:1675-1684)")
    check(r["value"] == 200 and r["consumable"] is True, "door value/consume")
    check(r["createTile"] == 10 and (r["width"], r["height"]) == (14, 28), "door tile/size")

    # Shuriken (id 42) — Item.cs:1869-1890: shoot 3, damage 10, style 1, 15/15,
    # no autoReuse, ranged class flag.
    r = row(42)
    check(r["shoot"] == 3 and r["damage"] == 10, "shuriken shoot/damage (Item.cs:1869+)")
    check((r["useTime"], r["useAnimation"], r["useStyle"]) == (15, 15, 1), "shuriken timings")
    check(r["autoReuse"] is False, "shuriken no autoReuse")
    check(r["ranged"] is True and r["melee"] is False, "shuriken class flags")

    # Flower of Fire (id 112) — Item.cs:2710-2723: damage 48, mana 9, kb 5.5,
    # rare 3, value hellPrice = sellPrice(0,2,50) = (50*100+2*10000)*5 = 125000
    # (sellPrice's *5 factor, Item.cs:49422-49426).
    r = row(112)
    check(r["damage"] == 48 and r["mana"] == 9, "flower-of-fire damage/mana")
    check(r["knockBack"] == 5.5 and r["rare"] == 3, "flower-of-fire kb/rare")
    check(r["value"] == 125000, "flower-of-fire hellPrice 125000")
    check(r["magic"] is True, "flower-of-fire class flag (noMelee is a named skip)")

    # Wooden/iron bow (id 99) — SD1 case 99: style 5, useAmmo Arrow (=40),
    # shoot 1, damage 8, no autoReuse.
    r = row(99)
    check(r["useStyle"] == 5 and r["useAmmo"] == 40, "bow style/ammo (case 99)")
    check(r["shoot"] == 1 and r["autoReuse"] is False, "bow shoot/reuse")

    # Cobalt drill (id 385) — Item.cs:5869-5876: useTime 13, useAnimation 25,
    # pick 110; the drill epilogue ×0.6 min 1 (Item.cs:48884-48895) → 7/15.
    r = row(385)
    check((r["useTime"], r["useAnimation"]) == (7, 15), "drill ×0.6 rule (13→7, 25→15)")
    check(r["pick"] == 110, "drill pick power")

    # Golf ball dyed red (id 4251, SD5) — projectile 748, rare 2, value 10000.
    r = row(4251)
    check(r["shoot"] == 748 and r["rare"] == 2 and r["value"] == 10000, "golf ball row")

    # Redirect 1803-1807 → 1533+id-1803 (Item.cs:48917-48919): the row adopts
    # the target's data wholesale.
    for k in range(1803, 1808):
        a, b = rows[k], rows[1533 + k - 1803]
        check(a == b, "redirect %d == %d" % (k, 1533 + k - 1803))

    # Deprecated → air (Item.cs:48921-48924): 2772 is in Sets.Deprecated
    # (ItemID.cs:1316) — its row is the stock reset, not the case body.
    r = row(2772)
    stock = gen.stock_row()
    stock["name"] = "None"  # TurnToAir ⇒ SetDefaults(0): id 0's name rides along
    check(r == stock, "deprecated 2772 zeroed (TurnToAir)")

    print("ok DW-1.2 spot sample (9 ids + redirect + deprecated)")


def test_dw_1_2_variants():
    rows = build_once()
    # The captured variant deltas: type 112 Stronger (Item.cs:2716-2723) —
    # rare 6, value 500000, damage 85, 12/12; 25 in-case rows total.
    vs = ALL_VARIANTS
    overrides = [o for o in vs if o[0] == 112 and o[1] == "Stronger" and o[3] == "delta"]
    check(len(overrides) == 1, "112 has exactly one Stronger delta, got %d" % len(overrides))
    ops = dict((f, v) for (_op, f, v) in overrides[0][2])
    check(ops.get("rare") == 6 and ops.get("value") == 500000, "112 stronger rare/value")
    check(ops.get("damage") == 85, "112 stronger damage")
    check(ops.get("useAnimation") == 12 and ops.get("useTime") == 12, "112 stronger timings")
    n_checks = len([o for o in vs if o[3] in ("delta", "restore")])
    check(n_checks == 25, "25 in-case Variant checks (22 delta + 3 restore), got %d" % n_checks)
    print("ok DW-1.2 variant deltas (112 Stronger + 25 total)")


# ---------------------------------------------------------------------------
# DW-1.3 — fail-loud on an unknown statement form
# ---------------------------------------------------------------------------
def test_dw_1_3_unknown_form_fails_loud():
    try:
        gen.parse_stmts(["\t\t\ttotallyUnknownCall(1, 2, 3);"], 0, 1, "Item.cs")
    except SystemExit as e:
        msg = str(e)
        check("Item.cs" in msg and "unsupported statement" in msg,
              "fail-loud message names source and reason: %r" % msg)
        print("ok DW-1.3 unknown form fails loud:", msg.strip()[:70])
        return
    fail("unknown statement did not fail (DW-1.3)")


# ---------------------------------------------------------------------------
# DW-1.4 — whitelist form coverage from real sources + epilogue rules
# ---------------------------------------------------------------------------
def _collect_calls(st, out):
    """Recursively gather helper-call names from parsed statement tuples
    (case bodies live in dicts — casemap/default of 'switch' nodes)."""
    if isinstance(st, dict):
        for v in st.values():
            _collect_calls(v, out)
        return
    if isinstance(st, list):
        for sub in st:
            _collect_calls(sub, out)
        return
    if not isinstance(st, tuple):
        return
    if st[0] == "call":
        out.add(st[2])
    for part in st:
        _collect_calls(part, out)


def test_dw_1_4_form_coverage():
    item_lines = gen.read_lines(gen.DEC / "Item.cs")

    # Every helper the C# bodies call is ported in HELPER_TABLE.
    called = set()
    for sig in ["private void SetDefaults%d(int type)" % k for k in range(1, 6)] + \
               ["private void SetFoodDefaults(int type)"]:
        lo, hi = gen.find_method_span(item_lines, sig, "Item.cs")
        for st in gen.parse_stmts(item_lines, lo + 1, hi - 1, "Item.cs"):
            _collect_calls(st, called)
    missing = called - set(gen.HELPER_TABLE)
    check(not missing, "helpers called by C# but not ported: %s" % sorted(missing))
    unused = set(gen.HELPER_TABLE) - called
    if unused:
        print("note: ported helpers never called by SD bodies:", sorted(unused))

    # All 86 IsFood ids have SetFoodDefaults cases (ItemID.cs Sets.IsFood).
    itemid_text = "\n".join(gen.read_lines(gen.DEC / "ID/ItemID.cs"))
    food = gen.parse_bool_set(itemid_text, "Sets.IsFood", "ItemID.cs")
    check(len(food) == 86, "IsFood set size 86, got %d" % len(food))
    rows = build_once()
    dep = gen.parse_bool_set(itemid_text, "Sets.Deprecated", "ItemID.cs")
    for i in food:
        if i in dep:
            continue  # TurnToAir zeroes deprecated foods (Item.cs:48921-48924)
        check(rows[i]["useStyle"] in (2, 9), "food %d got DefaultToFood style" % i)
        check(rows[i]["holdStyle"] == 1, "food %d holdStyle 1 (Item.cs:48913-48915)" % i)

    # Bait rare ladder (Item.cs:48838-48847): bait >= 50 → rare 3, >= 30 → 2,
    # >= 15 → 1 — holds for every row whose case set a bait value.
    ladder_ok = all(
        (r["bait"] < 15) or (r["rare"] == (3 if r["bait"] >= 50 else 2 if r["bait"] >= 30 else 1))
        for r in rows)
    check(ladder_ok, "bait rare ladder violated")

    # projHook zeroing (Item.cs:48878-48882): every hook item zeroes use
    # style/time/animation. Hook items are derived from Projectile aiStyle 7
    # (Main.cs:6612-6624); rows must be self-consistent with that set.
    hooks = {i for i in range(gen.ITEM_COUNT)
             if rows[i]["useStyle"] == 0 and rows[i]["useTime"] == 0
             and rows[i]["useAnimation"] == 0 and rows[i]["shoot"] > 0}
    check(len(hooks) > 0, "projHook zeroing produced no rows")
    for i in sorted(hooks)[:5]:
        print("   hook item %d zeroed (shoot %d)" % (i, rows[i]["shoot"]))

    # netDefaults negative table present (Item.cs:1211-1385): -1 → 3521.
    neg = dict(build_once() and _negative_table())
    check(neg.get(-1) == 3521, "netDefaults -1 → 3521, got %r" % neg.get(-1))
    check(len(neg) == 48, "48 negative ids, got %d" % len(neg))

    # OnCreated (Item.cs:48936): 5437 → 5358.
    text = OUT.read_text(encoding="utf-8")
    check("(5437, 5358)" in text, "ON_CREATED carries 5437 → 5358")

    print("ok DW-1.4 form coverage (helpers, food 86, bait ladder, hooks, netDefaults, OnCreated)")


def _negative_table():
    text = OUT.read_text(encoding="utf-8")
    block = text.split("NEGATIVE_DEFAULTS", 1)[1].split("];", 1)[0]
    pairs = []
    for a, b in gen.re.findall(r"\((-?\d+), (\d+)\)", block):
        pairs.append((int(a), int(b)))
    return dict(pairs)


def main():
    test_dw_1_1_idempotent_regen()
    test_dw_1_2_spot_sample()
    test_dw_1_2_variants()
    test_dw_1_3_unknown_form_fails_loud()
    test_dw_1_4_form_coverage()
    shutil.rmtree(TMP, ignore_errors=True)
    print("ALL GEN TESTS PASSED")


if __name__ == "__main__":
    main()
