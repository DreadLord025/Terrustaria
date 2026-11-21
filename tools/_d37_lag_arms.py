# W4-d37 LAGGARD-EXEC: exact to_u16/from_u16 arm lines (function-scoped)
# for every owed move+drop kind per lane, at CURRENT origin tips.
import re, subprocess, json

REPO = r"C:\Users\DreadLord\Desktop\TR-W4-d37"
DATA = "crates/tr-app/src/npc/data.rs"

def show(rev, path):
    r = subprocess.run(["git", "-C", REPO, "show", f"{rev}:{path}"],
                       capture_output=True)
    return r.stdout.decode("utf8", "replace") if r.returncode == 0 else ""

def fn_span(src, sig):
    i = src.index(sig)
    start_ln = src[:i].count("\n")  # 0-based line of sig
    j = src.index("{", i); d = 0; k = j
    while True:
        c = src[k]
        if c == "{": d += 1
        elif c == "}":
            d -= 1
            if d == 0: break
        k += 1
    return start_ln, src[i:k+1]

def arm_lines(rev, kinds):
    src = show(rev, DATA)
    base_to, body_to = fn_span(src, "pub fn to_u16")
    base_fr, body_fr = fn_span(src, "pub fn from_u16")
    out = {}
    for k in kinds:
        t = [base_to + i + 1 for i, l in enumerate(body_to.split("\n"))
             if re.search(rf"NpcKind::{k} => (\d+),", l)]
        f = [base_fr + i + 1 for i, l in enumerate(body_fr.split("\n"))
             if re.search(rf"(\d+) => NpcKind::{k},", l)]
        tid = re.search(rf"NpcKind::{k} => (\d+),", body_to)
        out[k] = {"id": int(tid.group(1)) if tid else None, "to_line": t, "from_line": f}
    return out

receipts = json.load(open(REPO + r"\tools\_d37_seal_receipts.json"))
TIPS = {
 "c3": "c3-style10-cursed-skull", "c4": "c4-style44-flying-fish",
 "c6": "c6-style59-pumpking-blade", "c7": "c7-style64-fireflies",
 "c8": "c8-style66-crawling-critters", "c9": "c9-style67-snails",
 "c10": "c10-style68-waterfowl", "d23": "d23-npc-ai-styles-9-14",
 "d35": "d35-npc-kinds-71-140", "d120": "d120-town-chatter",
 "d125": "d125-empress-r7-drain", "d42": "d42-npc-kinds-561-630",
 "e26": "e26-luck-system", "g8": "g8-npc-kind-wire-1",
}
out = {}
for r in receipts:
    lane = r["lane"]
    if lane not in TIPS: continue
    kinds = [m[0] for m in r["move_owed"]] + [d[0] for d in r["drop_owed"]]
    if not kinds: continue
    out[lane] = arm_lines("origin/task/W4-" + TIPS[lane], kinds)

json.dump(out, open(REPO + r"\tools\_d37_lag_arms.json", "w"), indent=1, sort_keys=True)
for lane, ks in out.items():
    print("==", lane)
    for k, v in ks.items():
        print(f"  {k:22s} id={v['id']} to={v['to_line']} from={v['from_line']}")
