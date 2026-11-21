# W4-d37 LAGGARD-EXEC probe 3: post-SEAL movers the sweep surfaced.
# d39/d41 REGRESSED? d43 EXECUTED into g10's ids? d36 band clash? new
# claimants f18/f42/d9/d11/g5/f47 - what kinds, which ids? Read-only.
import re, subprocess, json

REPO = r"C:\Users\DreadLord\Desktop\TR-W4-d37"
DATA = "crates/tr-app/src/npc/data.rs"

def git(*a):
    r = subprocess.run(["git", "-C", REPO] + list(a), capture_output=True)
    return r.returncode, r.stdout.decode("utf8", "replace")

def show(rev, path):
    rc, out = git("show", f"{rev}:{path}")
    return out if rc == 0 else ""

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

base = arms(show("a725acb04", DATA))
sweep = json.load(open(REPO + r"\tools\_d37_e8s_sweep.json"))
final = json.load(open(REPO + r"\tools\_d37_seal_final.json"))
census = json.load(open(REPO + r"\tools\_d37_seal_census.json"))["lanes"]

out = {}
LANES = {
 "d36": "origin/task/W4-d36-npc-kinds-141-210",
 "d39": "origin/task/W4-d39-npc-kinds-351-420",
 "d41": "origin/task/W4-d41-npc-kinds-491-560",
 "d43": "origin/task/W4-d43-npc-kinds-631-697",
 "f18": "origin/task/W4-f18-npc-kinds-verify-2",
 "f42": "origin/task/W4-f42-sandstorm-2",
 "d9": "origin/task/W4-d9-proj-ai-81-90",
 "d11": "origin/task/W4-d11-proj-ai-101-110",
 "g5": "origin/task/W4-g5-npc-style7-caster-doc",
 "f47": "origin/task/W4-f47-liquid-shimmer-2",
}
for lane, rev in LANES.items():
    to = arms(show(rev, DATA))
    new = {k: v for k, v in to.items() if k not in base}
    ent = {"new": new}
    # regression check: seal-census kinds missing at current tip?
    if lane in census:
        sealnew = census[lane].get("new", {})
        ent["seal_new"] = sealnew
        ent["lost_since_seal"] = {k: v for k, v in sealnew.items() if k not in new}
        ent["moved_since_seal"] = {k: [sealnew[k], new[k]] for k in sealnew
                                   if k in new and new[k] != sealnew[k]}
    # clash vs the SEAL grand plan (kind -> id)
    ent["clash_grand"] = {k: v for k, v in new.items()
                          if k in final["grand"] and final["grand"][k] != v}
    inv = {v: k for k, v in final["grand"].items()}
    ent["id_taken_by_other"] = {k: [v, inv[v]] for k, v in new.items()
                                if v in inv and inv[v] != k}
    out[lane] = ent

json.dump(out, open(REPO + r"\tools\_d37_lag_probe3.json", "w"), indent=1, sort_keys=True)
print(json.dumps(out, indent=1, sort_keys=True))
