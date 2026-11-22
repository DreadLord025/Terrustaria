# W4-d37 LAGGARD-EXEC: arm-line sites at the LIVE tips for every owed move
# + the stale-pin fix sites (g8/d125/c7/c10/g10). Function-scoped.
import re, subprocess, json

REPO = r"C:\Users\DreadLord\Desktop\TR-W4-d37"
DATA = "crates/tr-app/src/npc/data.rs"
NPC = "crates/tr-app/src/npc.rs"

def show(rev, p):
    r = subprocess.run(["git", "-C", REPO, "show", f"{rev}:{p}"], capture_output=True)
    return r.stdout.decode("utf8", "replace") if r.returncode == 0 else ""

def fnspan(src, sig):
    i = src.index(sig); j = src.index("{", i); d = 0; k = j
    while True:
        c = src[k]
        if c == "{": d += 1
        elif c == "}":
            d -= 1
            if d == 0: break
        k += 1
    return src[:i].count("\n"), src[i:k+1]

f2 = json.load(open(REPO + r"\tools\_d37_lag_final2.json"))
BR = {
 "c3":"c3-style10-cursed-skull","c4":"c4-style44-flying-fish","c6":"c6-style59-pumpking-blade",
 "c7":"c7-style64-fireflies","c8":"c8-style66-crawling-critters","c9":"c9-style67-snails",
 "c10":"c10-style68-waterfowl","d23":"d23-npc-ai-styles-9-14","d34":"d34-npc-kinds-1-70",
 "d35":"d35-npc-kinds-71-140","d36":"d36-npc-kinds-141-210","d40":"d40-npc-kinds-421-490",
 "d41":"d41-npc-kinds-491-560","d42":"d42-npc-kinds-561-630","d43":"d43-npc-kinds-631-697",
 "d120":"d120-town-chatter","d125":"d125-empress-r7-drain","e26":"e26-luck-system",
 "g8":"g8-npc-kind-wire-1","d9":"d9-proj-ai-81-90","d11":"d11-proj-ai-101-110",
 "g5":"g5-npc-style7-caster-doc","f42":"f42-sandstorm-2","f18":"f18-npc-kinds-verify-2",
 "f47":"f47-liquid-shimmer-2","e21":"e21-wiring-device-arms-3","g62":"g62-npc-hit-dust",
}
out = {}
lanes_needed = sorted(set(f2["moves"]) | set(f2["drops"]))
for lane in lanes_needed:
    rev = f2["tips"][lane]
    d = show(rev, DATA)
    if not d: continue
    bt, bo = fnspan(d, "pub fn to_u16")
    bf, fr = fnspan(d, "pub fn from_u16")
    ent = {}
    kinds = [m["kind"] for m in f2["moves"].get(lane, [])] + \
            [x["kind"] for x in f2["drops"].get(lane, [])]
    for k in set(kinds):
        t = [bt+i+1 for i, l in enumerate(bo.split("\n"))
             if re.search(rf"NpcKind::{k} => (\d+),", l)]
        f = [bf+i+1 for i, l in enumerate(fr.split("\n"))
             if re.search(rf"(\d+) => NpcKind::{k},", l)]
        ent[k] = {"to": t, "from": f}
    out[lane] = ent

# stale-pin fix sites
pins = {}
for lane in ["g8", "d125", "c7", "c10", "g10"]:
    rev = f2["tips"].get(lane) or "origin/task/W4-g10-npc-kind-wire-3"
    n = show(rev, NPC)
    m = re.search(r"const ALL_KINDS: \[NpcKind; (\d+)\]", n)
    i = n.find("covered + sentinel")
    pin_ln = n[:i].count("\n") + 2
    pin_val = re.search(r"(\d+),", n[i:]).group(1)
    rows_m = re.search(r"const ALL_KINDS: \[NpcKind; \d+\] = \[(.*?)\n    \];", n, re.S)
    rows = len(re.findall(r"^\s*NpcKind::\w+,\s*$", rows_m.group(1), re.M))
    pins[lane] = {"tip": rev if len(rev) < 12 else rev.split("/")[-1],
                  "decl_line": n[:m.start()].count("\n") + 1,
                  "decl": int(m.group(1)), "rows": rows,
                  "pin_line": pin_ln, "pin": int(pin_val)}
out["_pins"] = pins
json.dump(out, open(REPO + r"\tools\_d37_lag_sites2.json", "w"), indent=1, sort_keys=True)
print(json.dumps(pins, indent=1))
print("sites for", len(lanes_needed), "lanes")
