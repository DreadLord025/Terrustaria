# W4-d37 LAGGARD-EXEC probe 7: d38's 281-285 occupancy (d34 grew into it?),
# PartyBunny cs, c9/d125 new-tip decl/pin, d23/d34 new-kind wiring depth,
# e21 Bunny/PartyBunny nature, g8 current tip.
import re, subprocess, json

REPO = r"C:\Users\DreadLord\Desktop\TR-W4-d37"
DATA = "crates/tr-app/src/npc/data.rs"
NPC = "crates/tr-app/src/npc.rs"

def git(*a, repo=REPO):
    r = subprocess.run(["git", "-C", repo] + list(a), capture_output=True)
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

def arms(rev):
    return {m[0]: int(m[1]) for m in re.findall(
        r"NpcKind::(\w+) => (\d+),", fn_body(show(rev, DATA), "pub fn to_u16"))}

def akstate(rev):
    n = show(rev, NPC)
    m = re.search(r"const ALL_KINDS: \[NpcKind; (\d+)\] = \[(.*?)\n    \];", n, re.S)
    rows = re.findall(r"^\s*NpcKind::(\w+),\s*$", m.group(2), re.M)
    i = n.find("covered + sentinel")
    pin = int(re.search(r"(\d+),", n[i:]).group(1)) if i >= 0 else None
    return {"decl": int(m.group(1)), "rows": len(rows), "pin": pin,
            "decl_line": n[:m.start()].count("\n") + 1}

out = {}
# d38 at 281-285?
d38 = arms("origin/task/W4-d38-npc-kinds-281-350")
out["d38_281_290"] = {k: v for k, v in d38.items() if 281 <= v <= 290}
# d34 new five cs ids + wiring
d34rev = "origin/task/W4-d34-npc-kinds-1-70"
d34src = show(d34rev, DATA)
cs = fn_body(d34src, "fn cs_id")
out["d34_new_cs"] = {}
for k in ["Antlion", "Crab", "DungeonGuardian", "PinkJellyfish", "SpikeBall"]:
    m = re.search(rf"NpcKind::{k} => (\d+),", cs)
    out["d34_new_cs"][k] = int(m.group(1)) if m else "FALLBACK"
n34 = show(d34rev, NPC)
i = n34.find("fn update_kind_ai")
seg = n34[i:i+250000]
out["d34_new_update"] = {}
for k in ["Antlion", "Crab", "DungeonGuardian", "PinkJellyfish", "SpikeBall"]:
    m = re.search(rf"NpcKind::{k}\b[^\n]*=>([^\n]*)", seg)
    out["d34_new_update"][k] = m.group(0)[:90] if m else None
# e21 PartyBunny/Bunny cs + update
e21rev = "origin/task/W4-e21-wiring-device-arms-3"
e21src = show(e21rev, DATA)
cs21 = fn_body(e21src, "fn cs_id")
out["e21_cs"] = {}
for k in ["Bunny", "PartyBunny"]:
    m = re.search(rf"NpcKind::{k} => (\d+),", cs21)
    out["e21_cs"][k] = int(m.group(1)) if m else "FALLBACK"
# c9/d125/d36/d41/e26/d23/d34 ak state at new tips
for lane, rev in [("c9", "origin/task/W4-c9-style67-snails"),
                  ("d125", "origin/task/W4-d125-empress-r7-drain"),
                  ("d36", "origin/task/W4-d36-npc-kinds-141-210"),
                  ("d41", "origin/task/W4-d41-npc-kinds-491-560"),
                  ("e26", "origin/task/W4-e26-luck-system"),
                  ("d23", "origin/task/W4-d23-npc-ai-styles-9-14"),
                  ("d34", d34rev),
                  ("g8", "origin/task/W4-g8-npc-kind-wire-1")]:
    out[lane + "_ak"] = akstate(rev)
    rc, tip = git("rev-parse", "--short", rev)
    out[lane + "_ak"]["tip"] = tip.strip()
json.dump(out, open(REPO + r"\tools\_d37_lag_probe7.json", "w"), indent=1, sort_keys=True)
print(json.dumps(out, indent=1, sort_keys=True))
