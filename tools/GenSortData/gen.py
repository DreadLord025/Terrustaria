#!/usr/bin/env python
"""Emit crates/tr-app/src/sortdata.rs from the decompiled Terraria sources.

The chest Sort button (`ChestUI.cs:227`, id 4) drives `ItemSorting.SortChest`
(`UI/ItemSorting.cs:1629-1653`), whose 52-layer engine reads a fixed set of
static lookup tables. This generator extracts exactly those tables so the port
can never drift from the 1.4.5.8 source:

  * the twenty `ItemID.Sets.SortingPriority*` int sets (`ID/ItemID.cs:1328-1347`),
  * `ItemID.Sets.IsFood` / `IsBasicFish` / `IsFishingCrate`
    (`ID/ItemID.cs:498`, `:1308-1309`),
  * `BuffID.Sets.SortingPriorityFoodBuffs` (`ID/BuffID.cs:45`),
  * `MountID.Sets.Cart` (`ID/MountID.cs:43`),
  * `Main.projHook` (`Main.cs:6612-6624`: projectile ids whose `SetDefaults`
    sets `aiStyle = 7` — the grapple hook),
  * `Main.tileFrameImportant` (`Main.cs:66894` decl, `:7232-10352`: 403
    `Main.tileFrameImportant[N] = true;` rows).

`CreateIntSet(default, k1, v1, ...)` (`SetFactory.cs:116-131`) and
`CreateBoolSet([default,] ids...)` (`SetFactory.cs:97-109`) apply their inputs
left to right, so a repeated id keeps the last value; the emitted tables are
de-duplicated keeping the last value.

`python tools/GenSortData/gen.py --check` fails if the committed file drifted.
"""

import re
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
DEC = ROOT / "decompiled" / "Terraria"
OUT = ROOT / "crates" / "tr-app" / "src" / "sortdata.rs"

ITEMID_CS = DEC / "ID" / "ItemID.cs"
BUFFID_CS = DEC / "ID" / "BuffID.cs"
MOUNTID_CS = DEC / "ID" / "MountID.cs"
PROJECTILE_CS = DEC / "Projectile.cs"

ITEM_COUNT = 6196   # ItemID.cs
BUFF_COUNT = 401    # BuffID.cs
MOUNT_COUNT = 66    # MountID.cs
PROJECTILE_COUNT = 1136  # ProjectileID.cs
TILE_COUNT = 754    # TileID.cs


def read_lines(path):
    return Path(path).read_text(encoding="utf-8", errors="replace").splitlines()


def fail(src, line, msg):
    print("GenSortData: %s:%s: %s" % (src, line, msg), file=sys.stderr)
    sys.exit(1)


def parse_int_set(text, name):
    """`Sets.NAME = Sets.Factory.CreateIntSet(default, k1, v1, ...);` → (default, pairs)."""
    m = re.search(r"Sets\." + re.escape(name) +
                  r" = Sets\.Factory\.CreateIntSet\(\s*(-?\d+)\s*(?:,\s*([0-9,\s]*))?\);", text)
    if not m:
        fail("ItemID.cs", 1, "cannot find int set " + name)
    default = int(m.group(1))
    nums = [int(x) for x in (m.group(2) or "").split(",") if x.strip()]
    if len(nums) % 2:
        fail("ItemID.cs", 1, "odd entry count in int set " + name)
    pairs = list(zip(nums[0::2], nums[1::2]))
    # Later pairs win (SetFactory.cs:127-130): keep the last value per key.
    last = {}
    for k, v in pairs:
        last[k] = v
    ordered = []
    seen = set()
    for k, _ in pairs:
        if k not in seen:
            seen.add(k)
            ordered.append((k, last[k]))
    return default, ordered


def parse_bool_set(text, name, src):
    """`Sets.NAME = Sets.Factory.CreateBoolSet([default,] ids...);` → (default, ids).

    `CreateBoolSet(types)` implies `default = false` (`SetFactory.cs:92-95`);
    `CreateBoolSet(defaultState, types)` sets `buffer[id] = !defaultState`
    (`:97-109`).
    """
    m = re.search(r"Sets\." + re.escape(name) +
                  r" = Sets\.Factory\.CreateBoolSet\(([^;]*)\);", text)
    if not m:
        fail(src, 1, "cannot find bool set " + name)
    args = [a.strip() for a in m.group(1).split(",") if a.strip()]
    default = False
    if args and args[0] in ("true", "false"):
        default = args[0] == "true"
        args = args[1:]
    ids = []
    seen = set()
    for a in args:
        i = int(a)
        if i not in seen:
            seen.add(i)
            ids.append(i)
    return default, ids


def parse_projhook():
    """Projectile ids whose `SetDefaults` assigns `aiStyle = 7` — the grapple
    hooks `Main.projHook` marks (`Main.cs:6612-6624`)."""
    lines = read_lines(PROJECTILE_CS)
    lo, hi = 0, len(lines)
    for i, line in enumerate(lines):
        if "public void SetDefaults(int Type)" in line:
            lo = i
            break
    ids = []
    cur = None
    for i in range(lo, hi):
        s = lines[i].strip()
        m = re.match(r"(?:else )?if \(this\.type == (\d+)\)", s)
        if m:
            cur = int(m.group(1))
        if s == "this.aiStyle = 7;" and cur is not None:
            ids.append(cur)
    if len(ids) != 20:
        fail("Projectile.cs", lo + 1, "expected 20 aiStyle = 7 sites, got %d" % len(ids))
    return sorted(set(ids))


def parse_tile_frame_important():
    """`Main.tileFrameImportant[N] = true;` rows (`Main.cs:7232-10352`)."""
    lines = read_lines(DEC / "Main.cs")
    ids = set()
    for i, line in enumerate(lines):
        m = re.search(r"Main\.tileFrameImportant\[(\d+)\] = (true|false);", line)
        if not m:
            continue
        if m.group(2) == "true":
            ids.add(int(m.group(1)))
        else:
            fail("Main.cs", i + 1, "false tileFrameImportant row")
    # 403 `= true;` rows, 398 distinct ids (a handful are assigned twice).
    if len(ids) != 398:
        fail("Main.cs", 1, "expected 398 distinct tileFrameImportant ids, got %d" % len(ids))
    return sorted(ids)


def rust_int_pairs(pairs):
    """Format `(k, v)` pairs, ~12 per line for readability."""
    out = []
    row = []
    for k, v in pairs:
        row.append("(%d, %d)" % (k, v))
        if len(row) == 10:
            out.append(", ".join(row) + ",")
            row = []
    if row:
        out.append(", ".join(row) + ",")
    return out


def rust_ids(ids):
    out = []
    row = []
    for i in ids:
        row.append(str(i))
        if len(row) == 16:
            out.append(", ".join(row) + ",")
            row = []
    if row:
        out.append(", ".join(row) + ",")
    return out


def emit_int_set(parts, rust_name, cs_name, src, line, default, pairs):
    parts.append("/// `ItemID.Sets.%s` (`%s:%d`)." % (cs_name, src, line))
    parts.append("pub const %s: IntSet = IntSet {" % rust_name)
    parts.append("    default: %d," % default)
    parts.append("    pairs: &[")
    for row in rust_int_pairs(pairs):
        parts.append("        " + row)
    parts.append("    ],")
    parts.append("};")
    parts.append("")


def emit_bool_set(parts, rust_name, cs_name, src, line, default, ids):
    parts.append("/// `%s` (`%s:%d`)." % (cs_name, src, line))
    parts.append("pub const %s: BoolSet = BoolSet {" % rust_name)
    parts.append("    default: %s," % ("true" if default else "false"))
    parts.append("    ids: &[")
    for row in rust_ids(ids):
        parts.append("        " + row)
    parts.append("    ],")
    parts.append("};")
    parts.append("")


HEADER = '''// @generated by tools/GenSortData from the decompiled Terraria sources.
// Do not edit by hand.

//! Static lookup tables for `Terraria/UI/ItemSorting.cs` (1.4.5.8): the
//! `ItemID.Sets.SortingPriority*` int sets, the fishing/food bool sets, the
//! food-buff and cart sets, `Main.projHook` and `Main.tileFrameImportant`.
//! All of them are `SetFactory` products (`ID/SetFactory.cs:92-131`): later
//! inputs win, so a repeated id keeps its last value. Regenerate with
//! `python tools/GenSortData/gen.py`; `--check` freezes the file.
'''

INT_SET_DOC = '''/// One `SetFactory.CreateIntSet` lookup (`SetFactory.cs:116-131`).
#[derive(Debug)]
pub struct IntSet {
    /// The value every unlisted id takes.
    pub default: i32,
    /// `(id, value)` pairs, de-duplicated with the last value winning.
    pub pairs: &'static [(u16, i32)],
}

impl IntSet {
    /// The set's value for `id` — the default when unlisted. A repeated id
    /// reads its last pair (`SetFactory.cs:127-130` applies pairs left to
    /// right); the generated tables are already de-duplicated, so the scan
    /// cost stays negligible.
    #[inline]
    pub fn get(&self, id: u16) -> i32 {
        let mut found = None;
        for &(k, v) in self.pairs {
            if k == id {
                found = Some(v);
            }
        }
        found.unwrap_or(self.default)
    }
}
'''

BOOL_SET_DOC = '''/// One `SetFactory.CreateBoolSet` lookup (`SetFactory.cs:97-109`): a listed id
/// reads `!default`, an unlisted id reads `default`.
#[derive(Debug)]
pub struct BoolSet {
    /// The value every unlisted id takes.
    pub default: bool,
    /// The listed ids (read as `!default`).
    pub ids: &'static [u16],
}

impl BoolSet {
    /// The set's value for `id`.
    #[inline]
    pub fn get(&self, id: u16) -> bool {
        if self.ids.iter().any(|&i| i == id) {
            !self.default
        } else {
            self.default
        }
    }
}
'''

# (rust_name, cs_name, source file label, line, kind)
INT_SETS = [
    ("SORTING_PRIORITY_MISC_ACORNS", "SortingPriorityMiscAcorns", "ID/ItemID.cs", 1328),
    ("SORTING_PRIORITY_MISC_GEMS", "SortingPriorityMiscGems", "ID/ItemID.cs", 1329),
    ("SORTING_PRIORITY_MISC_HERBS_AND_SEEDS", "SortingPriorityMiscHerbsAndSeeds", "ID/ItemID.cs", 1330),
    ("SORTING_PRIORITY_MISC_BOSS_BAGS", "SortingPriorityMiscBossBags", "ID/ItemID.cs", 1331),
    ("SORTING_PRIORITY_TOOLS_INSTRUMENTS", "SortingPriorityToolsInstruments", "ID/ItemID.cs", 1332),
    ("SORTING_PRIORITY_TOOLS_FISHING", "SortingPriorityToolsFishing", "ID/ItemID.cs", 1333),
    ("SORTING_PRIORITY_TOOLS_GOLF", "SortingPriorityToolsGolf", "ID/ItemID.cs", 1334),
    ("SORTING_PRIORITY_TOOLS_KITES", "SortingPriorityToolsKites", "ID/ItemID.cs", 1335),
    ("SORTING_PRIORITY_TOOLS_KEYS", "SortingPriorityToolsKeys", "ID/ItemID.cs", 1336),
    ("SORTING_PRIORITY_TOOLS_MISC", "SortingPriorityToolsMisc", "ID/ItemID.cs", 1337),
    ("SORTING_PRIORITY_WEAPONS_RANGED", "SortingPriorityWeaponsRanged", "ID/ItemID.cs", 1338),
    ("SORTING_PRIORITY_POTIONS_BUFFS", "SortingPriorityPotionsBuffs", "ID/ItemID.cs", 1339),
    ("SORTING_PRIORITY_POTIONS_DYE_MATERIAL", "SortingPriorityPotionsDyeMaterial", "ID/ItemID.cs", 1340),
    ("SORTING_PRIORITY_MISC_IMPORTANTS", "SortingPriorityMiscImportants", "ID/ItemID.cs", 1341),
    ("SORTING_PRIORITY_WIRING", "SortingPriorityWiring", "ID/ItemID.cs", 1342),
    ("SORTING_PRIORITY_MATERIALS", "SortingPriorityMaterials", "ID/ItemID.cs", 1343),
    ("SORTING_PRIORITY_EXTRACTIBLES", "SortingPriorityExtractibles", "ID/ItemID.cs", 1344),
    ("SORTING_PRIORITY_ROPES", "SortingPriorityRopes", "ID/ItemID.cs", 1345),
    ("SORTING_PRIORITY_PAINTING", "SortingPriorityPainting", "ID/ItemID.cs", 1346),
    ("SORTING_PRIORITY_TERRAFORMING", "SortingPriorityTerraforming", "ID/ItemID.cs", 1347),
]


def build():
    itemid = "\n".join(read_lines(ITEMID_CS))
    buffid = "\n".join(read_lines(BUFFID_CS))
    mountid = "\n".join(read_lines(MOUNTID_CS))

    int_sets = [(rust, cs, src, line) + parse_int_set(itemid, cs)
                for (rust, cs, src, line) in INT_SETS]
    is_food = parse_bool_set(itemid, "IsFood", "ItemID.cs")
    is_basic_fish = parse_bool_set(itemid, "IsBasicFish", "ItemID.cs")
    is_fishing_crate = parse_bool_set(itemid, "IsFishingCrate", "ItemID.cs")
    food_buffs = parse_int_set(buffid, "SortingPriorityFoodBuffs")
    mount_cart = parse_bool_set(mountid, "Cart", "ID/MountID.cs")
    proj_hook = parse_projhook()
    tile_frame_important = parse_tile_frame_important()

    parts = [HEADER, "", INT_SET_DOC, "", BOOL_SET_DOC, ""]
    for entry in int_sets:
        rust, cs, src, line, default, pairs = entry
        emit_int_set(parts, rust, cs, src, line, default, pairs)

    # BuffID.Sets.SortingPriorityFoodBuffs — the one non-ItemID int set the
    # sort reads (Potions - Food, ItemSorting.cs:820).
    d, pairs = food_buffs
    parts.append("/// `BuffID.Sets.SortingPriorityFoodBuffs` (`ID/BuffID.cs:45`) — keyed")
    parts.append("/// by buff id (`ItemSorting.cs:820`).")
    parts.append("pub const SORTING_PRIORITY_FOOD_BUFFS: IntSet = IntSet {")
    parts.append("    default: %d," % d)
    parts.append("    pairs: &[")
    for row in rust_int_pairs(pairs):
        parts.append("        " + row)
    parts.append("    ],")
    parts.append("};")
    parts.append("")

    emit_bool_set(parts, "IS_FOOD", "IsFood", "ID/ItemID.cs", 498, *is_food)
    emit_bool_set(parts, "IS_BASIC_FISH", "IsBasicFish", "ID/ItemID.cs", 1308, *is_basic_fish)
    emit_bool_set(parts, "IS_FISHING_CRATE", "IsFishingCrate", "ID/ItemID.cs", 1309, *is_fishing_crate)
    emit_bool_set(parts, "MOUNT_CART", "Sets.Cart", "ID/MountID.cs", 43, *mount_cart)
    emit_bool_set(parts, "PROJ_HOOK", "Main.projHook", "Main.cs", 6622,
                  False, proj_hook)
    emit_bool_set(parts, "TILE_FRAME_IMPORTANT", "Main.tileFrameImportant", "Main.cs", 7232,
                  False, tile_frame_important)

    parts.append(emit_tests(is_food[1], proj_hook, tile_frame_important))
    parts.append("")
    return "\n".join(parts)


def emit_tests(food_ids, proj_hook, tile_ids):
    return '''#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn int_set_later_inputs_win() {
        // SetFactory.cs:127-130: the builder applies pairs left to right.
        const S: IntSet = IntSet { default: -1, pairs: &[(3, 5), (3, 9), (7, 1)] };
        assert_eq!(S.get(3), 9);
        assert_eq!(S.get(7), 1);
        assert_eq!(S.get(4), -1);
    }

    #[test]
    fn bool_set_reads_not_default_for_listed_ids() {
        const S: BoolSet = BoolSet { default: false, ids: &[2, 4] };
        assert!(S.get(2));
        assert!(!S.get(3));
    }

    #[test]
    fn counts_match_the_source_tables() {
        assert_eq!(IS_FOOD.ids.len(), %(food)d);
        assert_eq!(PROJ_HOOK.ids.len(), %(hooks)d);
        assert_eq!(TILE_FRAME_IMPORTANT.ids.len(), %(tiles)d);
        assert!(SORTING_PRIORITY_MISC_ACORNS.get(5518) == 1);
    }
}
''' % {"food": len(food_ids), "hooks": len(proj_hook), "tiles": len(tile_ids)}


def run_pipeline():
    text = build()
    OUT.write_text(text, encoding="utf-8", newline="\n")
    return text


def check():
    with tempfile.TemporaryDirectory(prefix="gensortdata-check-") as tmp:
        global OUT
        saved = OUT
        try:
            OUT = Path(tmp) / "sortdata.rs"
            want = run_pipeline()
        finally:
            OUT = saved
    current = OUT.read_text(encoding="utf-8") if OUT.exists() else ""
    if current == want:
        print("check ok (%d bytes)" % len(want))
        return 0
    want_lines = want.splitlines()
    cur_lines = current.splitlines()
    for i in range(max(len(want_lines), len(cur_lines))):
        a = cur_lines[i] if i < len(cur_lines) else "<missing>"
        b = want_lines[i] if i < len(want_lines) else "<missing>"
        if a != b:
            print("%s is out of date - rerun tools/GenSortData/gen.py" % OUT)
            print("first difference at line %d:" % (i + 1))
            print("  committed: %s" % a[:160])
            print("  generated: %s" % b[:160])
            break
    return 1


def main():
    for src, count, name in ((ITEMID_CS, ITEM_COUNT, "ItemID"), (BUFFID_CS, BUFF_COUNT, "BuffID"),
                             (MOUNTID_CS, MOUNT_COUNT, "MountID")):
        text = "\n".join(read_lines(src))
        m = re.search(r"\b%s\.Count = (\d+);" % name, text)
        if not m or int(m.group(1)) != count:
            fail(str(src), 1, "%s.Count mismatch" % name)
    if "--check" in sys.argv[1:]:
        sys.exit(check())
    text = run_pipeline()
    print("wrote %s (%d bytes)" % (OUT, len(text)))


if __name__ == "__main__":
    main()
