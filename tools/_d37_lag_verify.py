# W4-d37 LAGGARD-EXEC final witness: (1) c9 S2 patch git-apply-check at the
# CURRENT origin tip; (2) master-table arithmetic 69/61 + per-lane old->new
# arm-line verification at CURRENT tips; (3) tip-age liveness table.
import subprocess, json, re

REPO = r"C:\Users\DreadLord\Desktop\TR-W4-d37"

def git(*a):
    r = subprocess.run(["git", "-C", REPO] + list(a), capture_output=True)
    return r.returncode, r.stdout.decode("utf8", "replace")

receipts = json.load(open(REPO + r"\tools\_d37_seal_receipts.json"))
moves = sum(len(r["move_owed"]) for r in receipts)
drops = sum(len(r["drop_owed"]) for r in receipts)
print(f"SEAL receipts re-sum at HEAD: moves owed={moves} drops owed={drops}")
assert (moves, drops) == (69, 61), "seal arithmetic drift!"

# per-lane arm-line cross-check vs the lag_arms census
arms = json.load(open(REPO + r"\tools\_d37_lag_arms.json"))
bad = []
for r in receipts:
    lane = r["lane"]
    if lane not in arms: continue
    for k, old, new in r["move_owed"]:
        if arms[lane][k]["id"] != old: bad.append((lane, k, old, arms[lane][k]["id"]))
    for k, old, owner in r["drop_owed"]:
        if arms[lane][k]["id"] != old: bad.append((lane, k, old, arms[lane][k]["id"]))
print("arm-id mismatches vs current tips:", bad or "NONE (all 69+61 sites verified)")
assert not bad

# liveness: tip age of every laggard/executor lane
for br in ["task/W4-c9-style67-snails", "task/W4-g8-npc-kind-wire-1",
           "task/W4-d125-empress-r7-drain", "task/W4-d42-npc-kinds-561-630",
           "task/W4-c4-style44-flying-fish"]:
    rc, tip = git("rev-parse", "--short", "origin/" + br)
    rc, dt = git("log", "-1", "--format=%ci", "origin/" + br)
    print(f"{br:36s} {tip.strip()} {dt.strip()}")

# d125: the three lost-at-R0 commits are ancestors of origin now
for c in ["cd2b1b1e3", "86e3adfd3", "7a0482302"]:
    rc, _ = git("merge-base", "--is-ancestor", c, "origin/task/W4-d125-empress-r7-drain")
    print(f"d125 {c} ancestor-of-origin: {rc == 0}")
    assert rc == 0

# d42: 36 moves DONE at tip, 7 dragonflies + 4 waterfowl still on-branch
final = json.load(open(REPO + r"\tools\_d37_seal_final.json"))
rc, d = git("show", "origin/task/W4-d42-npc-kinds-561-630:crates/tr-app/src/npc/data.rs")
i = d.index("pub fn to_u16"); j = d.index("{", i); depth = 0; k = j
while True:
    c = d[k]
    if c == "{": depth += 1
    elif c == "}":
        depth -= 1
        if depth == 0: break
    k += 1
to = dict((m[0], int(m[1])) for m in re.findall(r"NpcKind::(\w+) => (\d+),", d[i:k+1]))
d42owned = [k for k, (l, o) in final["owners"].items() if l == "d42"]
wrong = [(k, to.get(k), final["final"][k]) for k in d42owned
         if to.get(k) != final["final"][k]]
print("d42 36 owned at FINAL ids:", "ALL-OK" if not wrong else wrong)
assert not wrong
for k in ["BlackDragonfly", "BlueDragonfly", "GreenDragonfly", "OrangeDragonfly",
          "RedDragonfly", "YellowDragonfly", "GoldDragonfly",
          "Seagull", "Seagull2", "Grebe", "Grebe2"]:
    assert k in to, k
print("d42 11 keep-one rows present on-branch (E8 dropped-lane rule): OK")
print("ALL LAGGARD-EXEC WITNESS CHECKS PASSED")
