# W4-d37 E8-SEAL census: per-lane new-kind maps at CURRENT tips (origin +
# named local probes), cs_id rows, ENTRIES nets, ALL_KINDS decls, bijection.
# Git-show only, direct paths, no junction recursion.
import re, subprocess, json

REPO = r"C:\Users\DreadLord\Desktop\TR-W4-d37"

def show(rev, path):
    r = subprocess.run(["git", "-C", REPO, "show", f"{rev}:{path}"],
                       capture_output=True)
    return r.stdout.decode("utf8", "replace") if r.returncode == 0 else None

DATA = "crates/tr-app/src/npc/data.rs"
NPC = "crates/tr-app/src/npc.rs"
BEST = "crates/tr-app/src/bestiary.rs"

def fn_body(src, sig):
    i = src.index(sig); j = src.index("{", i); d = 0; k = j
    while True:
        c = src[k]
        if c == "{": d += 1
        elif c == "}":
            d -= 1
            if d == 0: return src[j:k+1]
        k += 1

def arms_to(src):
    return {m[0]: int(m[1]) for m in
            re.findall(r"NpcKind::(\w+)\s*=>\s*(\d+)", fn_body(src, "pub fn to_u16"))}

def arms_from(src):
    return {int(m[0]): m[1] for m in
            re.findall(r"(\d+)\s*=>\s*NpcKind::(\w+)", fn_body(src, "pub fn from_u16"))}

def cs_rows(src):
    body = fn_body(src, "fn cs_id")
    return {m[0]: int(m[1]) for m in
            re.findall(r"NpcKind::(\w+)\s*=>\s*(\d+)", body)}

def entries(rev):
    src = show(rev, BEST)
    m = re.search(r"const ENTRIES[^=]*=\s*&?\[(.*?)\n\];", src, re.S)
    out = []
    for blk in re.split(r"BestiaryEntryDef\s*\{", m.group(1))[1:]:
        k = re.search(r"kind:\s*NpcKind::(\w+)", blk)
        n = re.search(r"net_id:\s*(-?\d+)", blk)
        if k and n: out.append((k.group(1), int(n.group(1))))
    return out

def all_kinds(rev):
    src = show(rev, NPC)
    m = re.search(r"const ALL_KINDS:\s*\[NpcKind;\s*(\d+)\]\s*=\s*\[(.*?)\];",
                  src, re.S)
    if not m: return None, []
    return int(m.group(1)), re.findall(r"NpcKind::(\w+)", m.group(2))

# CURRENT origin tips (E8-SEAL re-fetch, live wave) + local probes
TIPS = {
    "base": "a725acb04",
    "c3": "dd0ca4727", "c4": "8abf0f1d8", "c6": "5de680219",
    "c7": "c2cc6ec61", "c8": "10192602a", "c9": "222ad0a9d",
    "c10": "15f93d134", "d9": "88677f9de", "d11": "2894f6f7d",
    "d23": "b00e7626f", "d120": "2408233da", "d125": "5a26ae348",
    "d33": "b08acf463", "d34": "f0d92564e", "d35": "8fa3b3dbf",
    "d36": "8c5839b03", "d37": "7bea5c835", "d38": "b888a9575",
    "d39": "048c74f8f", "d40": "78ee5647c", "d41": "065efdf97",
    "d42": "adb304ab8", "d43": "31310b652", "e26": "a23c64933",
    "f18": "a832d38ea", "f42": "425d4ff9d", "f47": "057f44a68",
    "g5": "f044bd315", "g8": "dd3cb0572", "g10": "5dee28650",
    "c7b": "90abc1edf", "d103": "882be0ec1", "e21": "2fb0a36ee",
    "g62": "428e93ecb", "d33": "36aae03d0", "d36": "a11f61850",
}

base_names = set(arms_to(show("a725acb04", DATA)))
base_ents = entries("a725acb04")
out = {}
for lane, rev in TIPS.items():
    src = show(rev, DATA)
    if src is None:
        print(f"{lane:6s} {rev} ERROR data.rs"); continue
    to = arms_to(src); fr = arms_from(src)
    inv = {v: k for k, v in to.items()}
    bij = len(inv) == len(to) and inv == fr
    cs = cs_rows(src)
    new = {k: v for k, v in to.items() if k not in base_names}
    ents = entries(rev)
    new_ents = {k: n for k, n in ents if k not in base_names}
    decl, ak = all_kinds(rev)
    out[lane] = {"rev": rev, "total": len(to), "bijection": bij,
                 "new": new, "cs": {k: cs.get(k) for k in new},
                 "new_entries": new_ents, "entries_total": len(ents),
                 "decl": decl, "ak_len": len(ak),
                 "ak_missing_new": sorted(set(new) - set(ak))}
    print(f"{lane:6s} {rev} total {len(to):3d} new {len(new):3d} "
          f"bij {'OK' if bij else 'FAIL'} ents {len(ents):3d} "
          f"(new {len(new_ents):3d}) AK {decl}/{len(ak)}"
          f"{' MISSING:' + str(sorted(set(new) - set(ak))) if set(new) - set(ak) else ''}")
print(f"base entries: {len(base_ents)}")
json.dump({"lanes": out, "base_entries": len(base_ents)},
          open(REPO + r"\tools\_d37_e8s_census.json", "w"),
          indent=1, sort_keys=True)
