# W4-d37 LAGGARD-EXEC probe 4: f18 base-id collision, d125 cs ids, d41's
# +5 post-SEAL kinds, d43-vs-roster dup census, g8 Vampire clash, free ids.
import re, subprocess, json

REPO = r"C:\Users\DreadLord\Desktop\TR-W4-d37"
DATA = "crates/tr-app/src/npc/data.rs"

def show(rev, path):
    r = subprocess.run(["git", "-C", REPO, "show", f"{rev}:{path}"],
                       capture_output=True)
    return r.stdout.decode("utf8", "replace") if r.returncode == 0 else ""

def fn_body(src, sig):
    i = src.index(sig); j = src.index("{", i); d = 0; k = j
    while True:
        c = src[k]
        if c == "{": d += 1
        elif c == "}":
            d -= 1
            if d == 0: return src[j:k+1]
        k += 1

def arms(src):
    return {m[0]: int(m[1]) for m in
            re.findall(r"NpcKind::(\w+) => (\d+),", fn_body(src, "pub fn to_u16"))}

def cs(src, kind):
    b = fn_body(src, "fn cs_id")
    m = re.search(rf"NpcKind::{kind} => (\d+),", b)
    return int(m.group(1)) if m else None

base = arms(show("a725acb04", DATA))
base_ids = set(base.values())
out = {}

# f18: do its 20 new ids collide with BASE ids?
f18 = arms(show("origin/task/W4-f18-npc-kinds-verify-2", DATA))
f18new = {k: v for k, v in f18.items() if k not in base}
out["f18_base_clash"] = {k: [v, next(b for b, i in base.items() if i == v)]
                         for k, v in f18new.items() if v in base_ids}
out["f18_new"] = f18new

# d125 cs ids of the two owed-move kinds
d125 = show("origin/task/W4-d125-empress-r7-drain", DATA)
out["d125_cs"] = {k: cs(d125, k) for k in ["EmpressOfLight", "PrismaticLacewing"]}

# d41 post-SEAL +5
d41 = arms(show("origin/task/W4-d41-npc-kinds-491-560", DATA))
out["d41_new_all"] = {k: v for k, v in d41.items() if k not in base}

# d43 dup census vs the SEAL grand plan + g10 pushed
final = json.load(open(REPO + r"\tools\_d37_seal_final.json"))
grand = final["grand"]
d43 = arms(show("origin/task/W4-d43-npc-kinds-631-697", DATA))
d43new = {k: v for k, v in d43.items() if k not in base}
g10 = arms(show("origin/task/W4-g10-npc-kind-wire-3", DATA))
g10new = {k: v for k, v in g10.items() if k not in base}
out["d43_dup_of_g10"] = {k: v for k, v in d43new.items() if k in g10new}
out["d43_same_kind_other_id"] = {k: [v, grand[k]] for k, v in d43new.items()
                                 if k in grand and grand[k] != v}
inv = {v: k for k, v in grand.items()}
out["d43_same_id_other_kind"] = {k: [v, inv[v]] for k, v in d43new.items()
                                 if v in inv and inv[v] != k}
# name-alias dups: HallowBoss==EmpressOfLight(cs636), EmpressButterfly vs PrismaticLacewing(cs661)
out["d43_alias_candidates"] = {k: v for k, v in d43new.items()
                               if k in ["HallowBoss", "EmpressButterfly", "VileSpitEaterOfWorlds"]}
out["d43_free_new"] = {k: v for k, v in d43new.items()
                       if k not in grand and v not in inv and k not in g10new}

# d36 pushed five: cs ids + the exact to/from lines for the re-pin hunk
d36rev = "origin/task/W4-d36-npc-kinds-141-210"
d36 = show(d36rev, DATA)
d36a = arms(d36)
lines = d36.split("\n")
out["d36_five"] = {}
for k in ["ToxicSludge", "Penguin", "PenguinBlack", "GiantTortoise", "IceTortoise"]:
    t = [i+1 for i, l in enumerate(lines) if re.search(rf"NpcKind::{k} => \d+,", l)]
    f = [i+1 for i, l in enumerate(lines) if re.search(rf"\d+ => NpcKind::{k},", l)]
    out["d36_five"][k] = {"id": d36a.get(k), "cs": cs(d36, k), "to": t[:1], "from": f[:1]}

# free-id check around the grand max
used = set(grand.values()) | set(d43new.values()) | set(d36a[k] for k in out["d36_five"])
out["free_762_830"] = [i for i in range(762, 831) if i not in used]

json.dump(out, open(REPO + r"\tools\_d37_lag_probe4.json", "w"), indent=1, sort_keys=True)
print(json.dumps(out, indent=1, sort_keys=True))
