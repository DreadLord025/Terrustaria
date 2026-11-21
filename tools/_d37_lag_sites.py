# W4-d37 LAGGARD-EXEC: locate exact DROP-hunk lines for c4's 3 owed drops
# (BloodFeeder@241, CorruptGoldfish@405, FlyingFish@359) + MOVE pins for
# d125/g8/c9/c7/c8/c10/d23/d35/e26/c3/c6/d120 at CURRENT origin tips.
# Read-only git-show; direct paths only.
import re, subprocess, json

REPO = r"C:\Users\DreadLord\Desktop\TR-W4-d37"
DATA = "crates/tr-app/src/npc/data.rs"
NPC = "crates/tr-app/src/npc.rs"

def show(rev, path):
    r = subprocess.run(["git", "-C", REPO, "show", f"{rev}:{path}"],
                       capture_output=True)
    return r.stdout.decode("utf8", "replace") if r.returncode == 0 else ""

def hits(src, pats):
    out = []
    for i, l in enumerate(src.split("\n"), 1):
        if any(re.search(p, l) for p in pats):
            out.append([i, l.strip()[:100]])
    return out

TIPS = {
 "c3":  "origin/task/W4-c3-style10-cursed-skull",
 "c4":  "origin/task/W4-c4-style44-flying-fish",
 "c6":  "origin/task/W4-c6-style59-pumpking-blade",
 "c7":  "origin/task/W4-c7-style64-fireflies",
 "c8":  "origin/task/W4-c8-style66-crawling-critters",
 "c9":  "origin/task/W4-c9-style67-snails",
 "c10": "origin/task/W4-c10-style68-waterfowl",
 "d23": "origin/task/W4-d23-npc-ai-styles-9-14",
 "d35": "origin/task/W4-d35-npc-kinds-71-140",
 "d120":"origin/task/W4-d120-town-chatter",
 "d125":"origin/task/W4-d125-empress-r7-drain",
 "e26": "origin/task/W4-e26-luck-system",
 "g8":  "origin/task/W4-g8-npc-kind-wire-1",
}
out = {}

# c4: full per-kind hit census across its changed files
kinds = ["BloodFeeder", "CorruptGoldfish", "FlyingFish"]
c4files = subprocess.run(["git", "-C", REPO, "diff", "--name-only",
    "a725acb04", TIPS["c4"], "--", "crates/"], capture_output=True
    ).stdout.decode().split()
out["c4_drops"] = {}
for k in kinds:
    per = {}
    for f in c4files:
        h = hits(show(TIPS["c4"], f),
                 [rf"NpcKind::{k}\b", rf"\b{k},", rf"\b{k}\b.*=>", rf"=>.*\b{k}\b"])
        if h: per[f] = h
    out["c4_drops"][k] = per

# moves: locate the to/from arms + ALL_KINDS decl + s1_f_3 pin per lane
MOVE = {
 "c7": ["Firefly", "LightningBug"],
 "c8": ["Worm", "TruffleWorm", "TruffleWormDigger"],
 "c9": ["Snail", "GlowingSnail"],
 "c10": ["Duck", "Duck2", "DuckWhite", "DuckWhite2"],
 "d23": ["AngryTrapper"],
 "d120": ["SantaClaus", "Truffle", "Steampunker", "Cyborg"],
 "d125": ["EmpressOfLight", "PrismaticLacewing"],
 "e26": ["Frog"],
 "g8": ["PinkJellyfish", "Antlion", "SpikeBall", "WallCreeper", "WallCreeperWall",
        "IceElemental", "Herpling", "Derpling", "CrimsonAxe", "FloatyGross",
        "LostGirl", "Nymph", "IceBat", "Lavabat", "GiantFlyingFox", "VampireBat",
        "RedDevil"],
}
for lane, ks in MOVE.items():
    rev = TIPS[lane]
    d = show(rev, DATA); n = show(rev, NPC)
    ent = {"data_arms": {}, "npc_decl": None, "s1f3_pin": None, "ak_rows": {}}
    for k in ks:
        ent["data_arms"][k] = hits(d, [rf"NpcKind::{k} => \d+,", rf"\d+ => NpcKind::{k},"])
        ent["ak_rows"][k] = hits(n, [rf"^\s*NpcKind::{k},$"])
    m = re.search(r"const ALL_KINDS: \[NpcKind; (\d+)\]", n)
    if m: ent["npc_decl"] = [n[:m.start()].count("\n") + 1, int(m.group(1))]
    i = n.find("covered + sentinel")
    if i >= 0:
        j = n.find("\n", n.find("\n", i) + 1)
        ent["s1f3_pin"] = [n[:i].count("\n") + 2, n[n.find("\n", i)+1:j].strip()]
    out.setdefault("moves", {})[lane] = ent

json.dump(out, open(REPO + r"\tools\_d37_lag_sites.json", "w"), indent=1, sort_keys=True)
for k, per in out["c4_drops"].items():
    print("== c4", k)
    for f, h in per.items():
        print("  ", f, "->", len(h), "hits", h[:3], "..." if len(h) > 3 else "")
for lane, e in out["moves"].items():
    print("==", lane, "decl", e["npc_decl"], "pin", e["s1f3_pin"])
