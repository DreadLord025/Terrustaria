# W4-d37 SEAL phase: re-census EVERY E8-addressed lane at its CURRENT origin
# tip (plus local-only tips), and measure which lanes CONSUMED their E8
# renumber tables (per-kind moved / kept / dropped / absent). Scratch tool.
# git-show only, direct file paths, no junction recursion.
import re, subprocess, json

REPO = r"C:\Users\DreadLord\Desktop\TR-W4-d37"

def show(rev, path):
    r = subprocess.run(["git", "-C", REPO, "show", f"{rev}:{path}"],
                       capture_output=True)
    if r.returncode != 0:
        return None
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

def all_kinds(rev):
    src = show(rev, NPC)
    m = re.search(r"const ALL_KINDS:\s*\[NpcKind;\s*(\d+)\]\s*=\s*\[(.*?)\];",
                  src, re.S)
    if not m: return None, []
    return int(m.group(1)), re.findall(r"NpcKind::(\w+)", m.group(2))

# current tips resolved by branch (origin unless local-only / local ahead)
TIPS = {
    "base": "a725acb04",
    "c3": "9ea0ee90d",        # origin; local ahead: 9299046f7, cb5cef764
    "c4": "0df6964be",
    "c6": "5de680219",
    "c7": "c2cc6ec61",
    "c8": "3705670d3",
    "c9": "2042d77f1",
    "c10": "5d8387955",
    "d23": "43091b01c",
    "d120": "2408233da",
    "d125": "d8047a109",      # origin REGRESSED vs cd2b1b1e3; local 86e3adfd3
    "d33": "b08acf463",
    "d34": "5f3435a87",
    "d35": "e5558b0a9",       # origin; local ahead 189c04aab (docs)
    "d37": "a341daa12",
    "d38": "1ba0395c7",
    "d40": "8fe1d6d31",
    "d42": "2d7ac08c3",
    "e26": "048cca6cf",
    "g8": "82c93cecf",
    # new claimants / future lanes (outside the R9 pins)
    "d36": "bcf4ac551",
    "d39": "907234441",
    "d41": "5d036721d",
    "d43": "a725acb04",
    # local-only probes
    "d125L": "86e3adfd3",
    "c3L": "cb5cef764",
    "g10L": "2d0403324",
    "d35L": "189c04aab",
}

base_src = show(TIPS["base"], DATA)
base_names = set(arms_to(base_src))
maps = json.load(open(REPO + r"\tools\_d37_receipts_maps.json"))["lanes"]
plan = json.load(open(REPO + r"\tools\_d37_receipts_plan_raw.json"))
final = plan["final"]                      # kind -> canonical final id
owners = {k: tuple(v) for k, v in plan["owners"].items()}
drops = plan["drops"]                      # (kind, lane, pushed_id, keeper)
drop_by_lane = {}
for k, l, pid, keep in drops:
    drop_by_lane.setdefault(l, {})[k] = pid

out = {"lanes": {}, "consumption": {}}
for lane, rev in TIPS.items():
    src = base_src if lane == "base" else show(rev, DATA)
    if src is None:
        out["lanes"][lane] = {"rev": rev, "error": "data.rs unreadable"}
        continue
    to = arms_to(src); fr = arms_from(src)
    decl, ak = all_kinds(rev)
    inv = {v: k for k, v in to.items()}
    bij = len(inv) == len(to) and inv == fr
    new = {k: v for k, v in to.items() if k not in base_names}
    out["lanes"][lane] = {"rev": rev, "total": len(to), "new": new,
                          "bijection": bij, "decl": decl, "ak_len": len(ak),
                          "ak_missing_new": sorted(set(new) - set(ak)),
                          "ak_stale": decl != len(ak)}
    # E8 consumption for lanes the plan addressed
    plain = lane.rstrip("L")
    if plain in maps and lane not in ("base",):
        old_new = maps[plain]["new"]       # kind -> id at receipts census
        exp_move, exp_keep = {}, {}
        for k, pid in old_new.items():
            if k in drop_by_lane.get(plain, {}): continue
            if owners.get(k, (None,))[0] != plain: continue
            (exp_move if final[k] != pid else exp_keep)[k] = final[k]
        dset = drop_by_lane.get(plain, {})
        st = {"moved": [], "move_pending": [], "moved_wrong": [],
              "kept_ok": [], "keep_broken": [],
              "dropped_ok": [], "drop_pending": [], "new_since": {}}
        for k, nid in exp_move.items():
            cur = new.get(k)
            if cur is None: st["move_pending"].append((k, nid, "ABSENT"))
            elif cur == nid: st["moved"].append((k, nid))
            elif cur == old_new[k]: st["move_pending"].append((k, nid, cur))
            else: st["moved_wrong"].append((k, old_new[k], cur, nid))
        for k, nid in exp_keep.items():
            cur = new.get(k)
            if cur == nid: st["kept_ok"].append(k)
            else: st["keep_broken"].append((k, nid, cur))
        for k, pid in dset.items():
            if k not in new: st["dropped_ok"].append(k)
            else: st["drop_pending"].append((k, pid, new[k]))
        st["new_since"] = {k: v for k, v in new.items() if k not in old_new}
        out["consumption"][lane] = st

json.dump(out, open(REPO + r"\tools\_d37_seal_census.json", "w"),
          indent=1, sort_keys=True)
for lane, d in out["lanes"].items():
    if "error" in d:
        print(lane, d["rev"], "ERROR"); continue
    print(f"{lane:6s} {d['rev']} total {d['total']:3d} new {len(d['new']):3d} "
          f"bij {'OK' if d['bijection'] else 'FAIL'} "
          f"AK {d['decl']}/{d['ak_len']}"
          f"{' STALE' if d['ak_stale'] else ''}"
          f" missing:{d['ak_missing_new'] if d['ak_missing_new'] else '-'}")
print("--- consumption ---")
for lane, st in sorted(out["consumption"].items()):
    print(f"{lane:6s} moved {len(st['moved'])} pending {len(st['move_pending'])} "
          f"wrong {len(st['moved_wrong'])} kept_ok {len(st['kept_ok'])} "
          f"keep_broken {len(st['keep_broken'])} dropped {len(st['dropped_ok'])} "
          f"drop_pending {len(st['drop_pending'])} new_since {len(st['new_since'])}")
    for k, a, b, c in st["moved_wrong"]:
        print(f"    WRONG {k}: was {a} now {b} plan {c}")
    if st["new_since"]:
        print(f"    NEW: {sorted(st['new_since'].items(), key=lambda x: x[1])}")
