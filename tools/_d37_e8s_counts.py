# W4-d37 E8-SEAL counts: ENTRIES / sheets / slots / cues at CURRENT tips under
# the E8S plan. Git-show only, direct paths, no junction recursion.
import re, subprocess, json, collections

REPO = r"C:\Users\DreadLord\Desktop\TR-W4-d37"
def show(rev, path):
    r = subprocess.run(["git", "-C", REPO, "show", f"{rev}:{path}"], capture_output=True)
    return r.stdout.decode("utf8", "replace") if r.returncode == 0 else None
BEST = "crates/tr-app/src/bestiary.rs"; ASSETS = "crates/tr-app/src/assets.rs"; AUDIO = "crates/tr-app/src/audio.rs"
def entries(rev):
    src = show(rev, BEST)
    m = re.search(r"const ENTRIES[^=]*=\s*&?\[(.*?)\n\];", src, re.S)
    out = []
    for blk in re.split(r"BestiaryEntryDef\s*\{", m.group(1))[1:]:
        k = re.search(r"kind:\s*NpcKind::(\w+)", blk); n = re.search(r"net_id:\s*(-?\d+)", blk)
        if k and n: out.append((k.group(1), int(n.group(1))))
    return out
def sheets(rev):
    src = show(rev, ASSETS)
    m = re.search(r"STATIC_SHEET_NAMES[^=]*=\s*&?\[(.*?)\n\];", src, re.S)
    return re.findall(r'"(NPC_[^"]+)"', re.sub(r"//[^\n]*", "", m.group(1)))
def slots(rev):
    src = show(rev, ASSETS)
    m = re.search(r"BESTIARY_NPC_SHEET_IDS:\s*\[u16;\s*(\d+)\]\s*=\s*\[(.*?)\];", src, re.S)
    ids = [int(x) for x in re.findall(r"\b(\d+)\b", re.sub(r"//[^\n]*", "", m.group(2)))]
    return int(m.group(1)), ids
def cues(rev):
    src = show(rev, AUDIO)
    m = re.search(r"SFX_CUES[^=]*=\s*&?\[(.*?)\n\];", src, re.S)
    return re.findall(r'"(NPC_(?:Hit|Killed)_\d+)"', m.group(1))

census = json.load(open(REPO + r"\tools\_d37_e8s_census.json"))["lanes"]
plan = json.load(open(REPO + r"\tools\_d37_e8s_plan.json"))
owners = {k: tuple(v) for k, v in plan["owners"].items()}
final = plan["final"]
LANES = ["c3","c4","c6","c7","c8","c9","c10","d9","d11","d23","d120","d125",
         "d33","d34","d35","d36","d37","d38","d39","d40","d41","d42","d43",
         "e26","f18","f42","f47","g5","g8","g10","c7b","d103","e21","g62"]
TIPS = {l: census[l]["rev"] for l in LANES}

base_entries = entries("a725acb04")
base_names = set(k for k, _ in base_entries) | set(census["base"]["new"])  # base kinds
# base kind set from data.rs:
import re as _re
bsrc = show("a725acb04", "crates/tr-app/src/npc/data.rs")
i = bsrc.index("pub fn to_u16"); j = bsrc.index("pub fn from_u16")
base_kind_set = set(_re.findall(r"NpcKind::(\w+)\s*=>\s*\d+", bsrc[i:j]))

# --- ENTRIES union: owned new-kind rows, canonical owner's row
per = {l: [(k, n) for k, n in entries(TIPS[l]) if k not in base_kind_set] for l in LANES}
seen = set(); union = []
for l in LANES:
    for k, n in per[l]:
        if k in seen: continue
        if owners.get(k, (None,))[0] != l: continue
        seen.add(k); union.append((k, n, l))
missing = [k for k in owners if k not in seen]
nets = [n for _, n in base_entries] + [n for _, n, _ in union]
pos = [n for n in nets if n > 0]
dups = sorted(n for n in set(pos) if pos.count(n) > 1)
print("ENTRIES: base", len(base_entries), "+ owned new", len(union), "=", len(base_entries) + len(union))
print("owned kinds WITHOUT entry:", len(missing), sorted(missing))
print("positive nets:", len(pos), "distinct:", len(set(pos)), "DUP nets:", dups)

# --- sheets union
base_sheets = set(sheets("a725acb04")); u = set(base_sheets)
for l in LANES: u |= set(sheets(TIPS[l]))
print("STATIC NPC sheets: base", len(base_sheets), "union", len(u))

# --- slots: naive append + surviving distinct (net-covered)
naive = 181; surv = set(slots("a725acb04")[1])
for l in LANES:
    d, ids = slots(TIPS[l])
    naive += len(ids) - 181
    surv |= set(ids)
print("slots naive append:", naive, "| surviving distinct:", len(surv))

# --- cues
base_cues = set(cues("a725acb04")); uc = set(base_cues); naive_c = len(base_cues)
for l in LANES:
    cc = set(cues(TIPS[l])); uc |= cc; naive_c += len(cc - base_cues)
print("NPC cues: base", len(base_cues), "naive", naive_c, "unique union", len(uc))

json.dump({"entries": len(base_entries) + len(union), "no_entry": sorted(missing),
           "net_dups": dups, "sheets": len(u), "slots_naive": naive,
           "slots_surviving": len(surv), "cues": len(uc),
           "union_new_entries": union},
          open(REPO + r"\tools\_d37_e8s_counts.json", "w"), indent=1, sort_keys=True)
print("counts written")
