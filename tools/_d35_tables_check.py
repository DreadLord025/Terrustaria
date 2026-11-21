# W4-d35 sheets phase: BESTIARY_NPC_SHEET_IDS <-> bestiary_npc_sheets vec
# parallel-order witness + the STATIC_SHEET_NAMES / D35_SHEETS packing audit.
import io, re

ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
a = io.open(ROOT + r"\crates\tr-app\src\assets.rs", encoding="utf-8").read()
e = io.open(ROOT + r"\crates\tr-app\src\render\entities.rs", encoding="utf-8").read()

i = a.index("pub const BESTIARY_NPC_SHEET_IDS")
j = a.index("];", i)
# start AFTER the type annotation's closing bracket (the `= [` opener)
k = a.index("= [", i) + 3
body = a[k:j]
body = re.sub(r"//[^\n]*", "", body)
ids = [int(x) for x in re.findall(r"\d+", body)]
print("BESTIARY_NPC_SHEET_IDS rows:", len(ids), "| declared 197:", len(ids) == 197)
print("dups:", len(ids) - len(set(ids)))

i = a.index("bestiary_npc_sheets: vec![")
j = a.index("],", i)
vec = [int(x) for x in re.findall(r'atlas\.sheet\("NPC_(\d+)"\)', a[i:j])]
print("vec rows:", len(vec), "| order match:", ids == vec)

# D35_SHEETS vs STATIC_SHEET_NAMES
i = e.index("pub(crate) const D35_SHEETS")
j = e.index("];", i)
d35 = [int(x) for x in re.findall(r"\b\d+\b", re.sub(r"//[^\n]*", "", e[i:j]))]
print("D35_SHEETS:", len(d35), d35)
i = a.index("pub(crate) const STATIC_SHEET_NAMES")
j = a.index("];", i)
static = re.findall(r'"([^"]+)"', a[i:j])
missing = [f"NPC_{x}" for x in d35 if f"NPC_{x}" not in static]
print("D35 sheets missing from STATIC:", missing)
dups = [n for n in set(static) if static.count(n) > 1]
print("STATIC dups:", dups)
# the 16 portrait ids must be the visible subset of d35
hide = {96, 97, 99, 100, 112}
visible = [x for x in d35 if x not in hide]
print("portrait ids == visible d35:", ids[-16:] == visible, visible)
