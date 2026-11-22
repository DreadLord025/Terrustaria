# W4-d37 receipts-phase census: re-derive per-lane new-kind maps from pushed
# tips (git-show only), verify the 108-EXTEND canonical plan collision-free,
# emit per-lane old->new renumber tables + dedupe drops. Scratch tool.
import re, subprocess, sys, json

REPO = r"C:\Users\DreadLord\Desktop\TR-W4-d37"

def show(rev, path):
    r = subprocess.run(["git", "-C", REPO, "show", f"{rev}:{path}"],
                       capture_output=True)
    if r.returncode != 0:
        raise RuntimeError(f"git show {rev}:{path} failed: {r.stderr.decode()[:200]}")
    return r.stdout.decode("utf-8", "replace")

DATA = "crates/tr-app/src/npc/data.rs"
NPC = "crates/tr-app/src/npc.rs"

def fn_body(src, sig):
    i = src.index(sig)
    j = src.index("{", i)
    depth, k = 0, j
    while True:
        c = src[k]
        if c == "{": depth += 1
        elif c == "}":
            depth -= 1
            if depth == 0: return src[j:k+1]
        k += 1

def arms_to(src):
    body = fn_body(src, "pub fn to_u16")
    return {m[0]: int(m[1]) for m in
            re.findall(r"NpcKind::(\w+)\s*=>\s*(\d+)", body)}

def arms_from(src):
    body = fn_body(src, "pub fn from_u16")
    return {int(m[0]): m[1] for m in
            re.findall(r"(\d+)\s*=>\s*NpcKind::(\w+)", body)}

def arms_cs(src):
    body = fn_body(src, "fn cs_id")
    out = {}
    for m in re.finditer(r"((?:NpcKind::\w+\s*\|\s*)*NpcKind::\w+)\s*=>\s*(\d+)", body):
        for name in re.findall(r"NpcKind::(\w+)", m.group(1)):
            out[name] = int(m.group(2))
    return out

def all_kinds(rev):
    src = show(rev, NPC)
    m = re.search(r"const ALL_KINDS:\s*\[NpcKind;\s*(\d+)\]\s*=\s*\[(.*?)\];",
                  src, re.S)
    decl = int(m.group(1))
    names = re.findall(r"NpcKind::(\w+)", m.group(2))
    return decl, names

TIPS = {
    "base": "a725acb0",
    "c3": "9299046f7", "c4": "17a241f6a", "c6": "5de68021",
    "c7": "c2cc6ec6", "c8": "3705670d", "c9": "2042d77f",
    "c10": "9641e20db", "d23": "19109424", "d120": "c5e68a80",
    "d125": "cd2b1b1e", "d33": "b08acf463", "d34": "5f3435a8",
    "d35": "6a2a155a", "d37": "bdd59afb", "d38": "6f9dd02d",
    "d40": "8fe1d6d3", "d42": "9b22c091", "e26": "048cca6c",
    "g8": "82c93cec",
}

lanes = {}
base_src = show(TIPS["base"], DATA)
base_to = arms_to(base_src)
base_names = set(base_to)
for lane, rev in TIPS.items():
    src = base_src if lane == "base" else show(rev, DATA)
    to = arms_to(src); fr = arms_from(src); cs = arms_cs(src)
    decl, ak = all_kinds(rev)
    # bijection check
    inv = {v: k for k, v in to.items()}
    assert len(inv) == len(to), f"{lane}: duplicate to_u16 ids"
    assert inv == fr, f"{lane}: to/from not bijective"
    new = {k: v for k, v in to.items() if k not in base_names}
    lanes[lane] = {"rev": rev, "to": to, "cs": cs, "new": new,
                   "decl": decl, "ak_len": len(ak),
                   "ak_matches": set(ak) == set(to)}
    print(f"{lane:5s} {rev}: total {len(to)}, new {len(new)}, "
          f"ALL_KINDS decl {decl} actual {len(ak)} "
          f"{'OK' if decl == len(ak) else 'STALE'}, "
          f"ak==to_set {set(ak) == set(to)}")

out = {"lanes": {l: {"rev": d["rev"], "new": d["new"],
                     "cs": {k: d["cs"].get(k) for k in d["new"]}}
                 for l, d in lanes.items()}}
with open(r"C:\Users\DreadLord\Desktop\TR-W4-d37\tools\_d37_receipts_maps.json", "w") as f:
    json.dump(out, f, indent=1, sort_keys=True)
print("maps written")
