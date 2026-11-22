# W4-d37 LAGGARD-EXEC probe 6: the live wave re-read. c9 pushed +9, d125
# EXECUTED moves, e26 executed, d23 GREW to 23, d34 GREW to 20, d36 re-pinned
# 767-793?, d41 batch-6, e21/g62 new claimants. Full new-kind maps at the
# CURRENT tips + clash gates vs the SEAL final + the lag union.
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

def arms(rev):
    src = show(rev, DATA)
    if not src: return {}
    return {m[0]: int(m[1]) for m in
            re.findall(r"NpcKind::(\w+) => (\d+),", fn_body(src, "pub fn to_u16"))}

base = set(arms("a725acb04"))
LANES = {
 "c9": "origin/task/W4-c9-style67-snails",
 "d125": "origin/task/W4-d125-empress-r7-drain",
 "e26": "origin/task/W4-e26-luck-system",
 "d23": "origin/task/W4-d23-npc-ai-styles-9-14",
 "d34": "origin/task/W4-d34-npc-kinds-1-70",
 "d36": "origin/task/W4-d36-npc-kinds-141-210",
 "d41": "origin/task/W4-d41-npc-kinds-491-560",
 "e21": "origin/task/W4-e21-wiring-device-arms-3",
 "g62": "origin/task/W4-g62-npc-hit-dust",
}
out = {}
for lane, rev in LANES.items():
    rc, tip = git("rev-parse", "--short", rev)
    rc, dt = git("log", "-1", "--format=%ci", rev)
    a = arms(rev)
    out[lane] = {"tip": tip.strip(), "date": dt.strip(),
                 "new": {k: v for k, v in a.items() if k not in base}}
json.dump(out, open(REPO + r"\tools\_d37_lag_probe6.json", "w"), indent=1, sort_keys=True)
print(json.dumps(out, indent=1, sort_keys=True))
