# W4-d37 LAGGARD-EXEC: d42 SEVEN-dragonfly-only drop ranges at bac408be0.
# research/239 sec1a covers 11 kinds, but the SEAL (fbb94817, published
# 12:49 - 239 read the PRE-seal a341daa12) keeps the 4 waterfowl on d42
# (c10 drops its dups). Split 239's ranges: dragonfly lines only.
import re, subprocess, json

REPO = r"C:\Users\DreadLord\Desktop\TR-W4-d37"
REV = "bac408be0"
DF = ["BlackDragonfly", "BlueDragonfly", "GreenDragonfly", "OrangeDragonfly",
      "RedDragonfly", "YellowDragonfly", "GoldDragonfly"]
WF = ["Seagull", "Seagull2", "Grebe", "Grebe2"]

def show(p):
    r = subprocess.run(["git", "-C", REPO, "show", f"{REV}:{p}"],
                       capture_output=True)
    return r.stdout.decode("utf8", "replace")

FILES = ["crates/tr-app/src/npc.rs", "crates/tr-app/src/npc/data.rs",
         "crates/tr-app/src/bestiary.rs", "crates/tr-app/src/npc_buffs.rs",
         "crates/tr-app/src/app/netglue.rs", "crates/tr-app/src/ui/bestiary.rs",
         "crates/tr-app/src/render/entities.rs"]
out = {}
for f in FILES:
    src = show(f)
    ls = src.split("\n")
    hits = []
    for i, l in enumerate(ls, 1):
        kinds = [k for k in DF if re.search(rf"\b{k}\b", l)]
        wf = [k for k in WF if re.search(rf"\b{k}\b", l)]
        if kinds:
            hits.append({"line": i, "kinds": kinds, "mixed_wf": wf,
                         "text": l.strip()[:96]})
    out[f] = hits

json.dump(out, open(REPO + r"\tools\_d37_lag_d42df.json", "w"), indent=1)
for f, hits in out.items():
    print("==", f, len(hits), "dragonfly-hit lines")
    mixed = [h for h in hits if h["mixed_wf"]]
    if mixed:
        print("   MIXED waterfowl lines (EDIT not DROP):",
              [(h["line"], h["kinds"], h["mixed_wf"]) for h in mixed])
    # contiguous ranges
    rng = []
    for h in hits:
        if rng and h["line"] == rng[-1][1] + 1:
            rng[-1][1] = h["line"]
        else:
            rng.append([h["line"], h["line"]])
    print("   ranges:", rng)
