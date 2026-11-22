# W4-d35 RETIRE phase 2: the NEW-claimant delta since the 490-name union.
# Compares the re-extracted pushed set (comment-tolerant regex) against the
# committed 424-name extraction, resolves dims, and reports the new union
# total + the named claimants (g66c/g84b/e23b/f42/g34e) explicitly.
import json, subprocess, os, re

HERE = os.path.dirname(os.path.abspath(__file__))
def git(*a):
    return subprocess.run(["git", *a], capture_output=True, text=True,
                          encoding="utf-8", errors="replace").stdout

old = json.loads(git("show", "HEAD:tools/_d35_pushed_sheets.json"))
new = json.load(open(os.path.join(HERE, "_d35_pushed_sheets.json")))
on = {n for v in old.values() for n in v}
nn = {n for v in new.values() for n in v}

D40_IDS = [430,431,432,433,434,435,436,437,438,440,442,443,444,445,446,
  447,448,449,450,451,452,453,454,455,456,457,458,459,460,461,462,463,464,465,
  466,467,468,469,470,472,473,474,475,476,477,478,479,480,481,482,483,484,485,
  486,487,488]
D35_DEFERRED = [79,80,84,85,86,103,104,109,110,120,122,140]
D125_TIP = ["NPC_636","NPC_661","Extra_188"]
D125_DEFERRED = ["Extra_157","Extra_158","Extra_159","Extra_160","Extra_187"]

forecast = {f"NPC_{i}" for i in D40_IDS}
d35def = {f"NPC_{i}" for i in D35_DEFERRED}
d125tip = set(D125_TIP); d125def = set(D125_DEFERRED)

# the 490-name union (committed): 424 pushed + forecast + d35def + d125tip + d125def
union490 = on | forecast | d35def | d125tip | d125def
print("committed 490-union size:", len(union490))
print("new pushed size:", len(nn), "| old pushed:", len(on))

newnames = sorted(nn - on)
print("\nNEW pushed names since 424:", len(newnames))
# which of the new names were ALREADY in the 490 union (forecast/deferred overlap)
already = [n for n in newnames if n in union490]
genuinely_new = [n for n in newnames if n not in union490]
print("  already in 490-union (forecast/deferred now real):", len(already), already)
print("  genuinely NEW to the union:", len(genuinely_new))

# owners for the genuinely-new names
def owners_of(n):
    return sorted({b.replace("origin/task/","") for b,v in new.items() if n in v})
print("\nGenuinely-new names + owners:")
for n in genuinely_new:
    print("  %-28s %s" % (n, ",".join(owners_of(n))))

# named claimants
print("\n=== named claimants ===")
for slug in ["g66-player-trail-dust","g84-plantera-bulb-draw","e23-bestiary-ui-grid","f42-sandstorm-2","g34-drawdata-wire-2"]:
    b = "origin/task/W4-"+slug
    v = new.get(b, [])
    ov = old.get(b, [])
    delta = sorted(set(v)-set(ov))
    print(f"{slug}: now {len(v)} rows (was {len(ov)}), delta {len(delta)}: {delta}")

# the NEW full union (pushed + forecast + d35def + d125tip + d125def, deduped)
newunion = nn | forecast | d35def | d125tip | d125def
print("\nNEW full union size:", len(newunion), "(was", len(union490), ")")
print("net new union names:", len(newunion - union490))
print("names dropped from union:", sorted(union490 - newunion))

# write the genuinely-new + already-real lists for the delta table
json.dump({
  "newPushed": newnames, "alreadyIn490": already, "genuinelyNew": genuinely_new,
  "newUnionSize": len(newunion), "oldUnionSize": len(union490),
  "owners": {n: owners_of(n) for n in newnames},
}, open(os.path.join(HERE, "_d35_retire_delta.json"), "w"), indent=1)
print("\nwrote _d35_retire_delta.json")
