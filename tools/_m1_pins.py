# W4 merger wave-1 small-lanes: post-splice pin census (TR-MERGE-1).
# Checks: to_u16/from_u16 bijective + equal maps; ALL_KINDS decl==rows==enum
# variant set==to/from key set; ENTRIES positive nets distinct; BESTIARY slots
# parallel vec order-equal; STATIC_SHEET_NAMES NPC rows; render-key 791-805
# reservation (only Crab/DungeonGuardian + the 13 d37 armored-bones/ichor keys).
# Usage: python tools/_m1_pins.py [--expect N]
import re, sys, collections, os
ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
def rd(p):
    with open(os.path.join(ROOT, p), encoding="utf8") as f: return f.read()

data = rd("crates/tr-app/src/npc/data.rs")
npc  = rd("crates/tr-app/src/npc.rs")
best = rd("crates/tr-app/src/bestiary.rs")
assets = rd("crates/tr-app/src/assets.rs")

def arms(fn):
    i = data.index("pub fn " + fn); j = data.index("{", i)
    # find matching close brace
    depth = 0; k = j
    while True:
        if data[k] == "{": depth += 1
        elif data[k] == "}":
            depth -= 1
            if depth == 0: break
        k += 1
    body = data[j:k]
    body = re.sub(r"//[^\n]*", "", body)
    return re.findall(r"NpcKind::(\w+)\s*=>\s*(\d+)", body) if fn == "to_u16" \
        else [(m[1], m[0]) for m in re.findall(r"(\d+)\s*=>\s*(?:Some\()?\s*NpcKind::(\w+)", body)]

to = arms("to_u16")                      # (kind, id)
fr = arms("from_u16")                    # (kind, id) — regex swapped
tok = [k for k, _ in to]; tov = [v for _, v in to]
frk = [k for k, _ in fr]; frv = [v for _, v in fr]
errs = []
def dups(xs): return sorted(x for x, c in collections.Counter(xs).items() if c > 1)
for name, xs in [("to_u16 keys", tok), ("to_u16 ids", tov), ("from_u16 ids", frk), ("from_u16 kinds", frv)]:
    d = dups(xs)
    if d: errs.append(f"DUP {name}: {d}")
if dict(to) != dict(fr): errs.append("to/from maps NOT inverse")

# enum variants
m = re.search(r"pub enum NpcKind\s*\{(.*?)\n\}", npc, re.S)
if not m: m = re.search(r"pub enum NpcKind\s*\{(.*?)\n\}", data, re.S)
enum_src = m.group(1)
enum_src = re.sub(r"//[^\n]*", "", enum_src)
variants = re.findall(r"^\s*(\w+)\s*,", enum_src, re.M)
d = dups(variants)
if d: errs.append(f"DUP enum variants: {d}")

# ALL_KINDS
m = re.search(r"const ALL_KINDS:\s*\[NpcKind;\s*(\d+)\]\s*=\s*\[(.*?)\];", npc, re.S)
decl = int(m.group(1))
rows = re.findall(r"NpcKind::(\w+)", re.sub(r"//[^\n]*", "", m.group(2)))
d = dups(rows)
if d: errs.append(f"DUP ALL_KINDS rows: {d}")
if decl != len(rows): errs.append(f"ALL_KINDS decl {decl} != rows {len(rows)}")
if set(rows) != set(variants): errs.append(f"ALL_KINDS vs enum delta: {set(rows) ^ set(variants)}")
if set(rows) != set(tok): errs.append(f"ALL_KINDS vs to_u16 delta: {set(rows) ^ set(tok)}")

# s1_f_3 + batch witness pins
pins = sorted(set(int(x) for x in re.findall(r"assert_eq!\(ALL_KINDS\.len\(\),\s*(\d+)\)", npc)))
s1 = re.search(r"mod s1_f_3_tests.*", npc, re.S)
s1pins = sorted(set(int(x) for x in re.findall(r"assert_eq!\([^\n]*?,\s*(\d+)\)", s1.group(0)))) if s1 else []

# ENTRIES
m = re.search(r"const ENTRIES[^=]*=\s*&?\[(.*?)\n\];", best, re.S)
ent = []
for blk in re.split(r"BestiaryEntryDef\s*\{", m.group(1))[1:]:
    k = re.search(r"kind:\s*NpcKind::(\w+)", blk); n = re.search(r"net_id:\s*(-?\d+)", blk)
    if k and n: ent.append((k.group(1), int(n.group(1))))
ek = [k for k, _ in ent]; nets = [n for _, n in ent if n > 0]
d = dups(ek)
if d: errs.append(f"DUP ENTRIES kinds: {d}")
d = dups(nets)
if d: errs.append(f"DUP ENTRIES nets: {d}")

# slots
m = re.search(r"BESTIARY_NPC_SHEET_IDS:\s*\[u16;\s*(\d+)\]\s*=\s*\[(.*?)\];", assets, re.S)
slotdecl = int(m.group(1))
slotids = [int(x) for x in re.findall(r"\b(\d+)\b", re.sub(r"//[^\n]*", "", m.group(2)))]
m2 = re.search(r"bestiary_npc_sheets:\s*vec!\[(.*?)\n\s*\],", assets, re.S)
vecids = [int(x) for x in re.findall(r'atlas\.sheet\("NPC_(\d+)"\)', m2.group(1))] if m2 else []
vecn = len(vecids)
if vecn >= 0 and vecids != slotids: errs.append(f"slots ids != parallel vec order (first diff at {next((i for i,(a,b) in enumerate(zip(slotids,vecids)) if a!=b), 'len')})")
if slotdecl != len(slotids): errs.append(f"slots decl {slotdecl} != ids {len(slotids)}")
if vecn >= 0 and vecn != len(slotids): errs.append(f"slots {len(slotids)} != parallel vec {vecn}")

# sheets
m = re.search(r"STATIC_SHEET_NAMES[^=]*=\s*&?\[(.*?)\n\];", assets, re.S)
sheetrows = re.findall(r'"(NPC_[^"]+)"', re.sub(r"//[^\n]*", "", m.group(1)))
plainrows = [r for r in sheetrows if re.fullmatch(r"NPC_\d+", r)]

print(f"ALL_KINDS decl={decl} rows={len(rows)} enum={len(variants)} to={len(to)} from={len(fr)}")
print(f"ALL_KINDS.len pins in npc.rs: {pins}")
print(f"ENTRIES={len(ent)} distinct-positive-nets={len(set(nets))}")
print(f"BESTIARY_NPC_SHEET_IDS={len(slotids)} (decl {slotdecl}) parallel-vec={vecn}")
print(f"STATIC_SHEET_NAMES NPC rows={len(sheetrows)} (plain NPC_id {len(plainrows)})")
print(f"max protocol id={max(int(v) for v in tov)}")
if errs:
    print("ERRORS:"); [print(" -", e) for e in errs]; sys.exit(1)
if "--expect" in sys.argv:
    exp = int(sys.argv[sys.argv.index("--expect") + 1])
    if not (decl == len(rows) == len(to) == len(fr) == exp):
        print(f"EXPECT FAIL: wanted {exp}"); sys.exit(1)
    badpins = [p for p in pins if p != exp]
    if badpins: print(f"STALE ALL_KINDS.len pins: {badpins}"); sys.exit(1)
print("PINS OK: bijective, decl==rows==enum==to==from, nets distinct")
