# W4-d35 SEAL-VERIFY: the NEW-claimant delta since the RETIRE 562-union.
# Compares the re-extracted pushed set (65 branches / 630 names) against the
# committed retire extraction (54 / 504), reports the new full union size and
# owners for every genuinely-new name. Read-only vs git; writes seal json.
import json, subprocess, os

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
# d42's FINAL 32-kind band (research/239 §4): NO-PACK verdict (research/238
# §4, eager 8382 +190 OVER) - the 29 names no other lane packs are FORECAST
# rows exactly like d40's 56 (580/630 pushed by f42, 619 by d11).
D42_FORECAST = [578,579,582,583,584,585,586,590,591,593,594,610,611,612,613,
  614,616,617,618,620,621,622,623,624,625,626,627,628,629]
forecast = {f"NPC_{i}" for i in D40_IDS}
d35def = {f"NPC_{i}" for i in D35_DEFERRED}
d42fc = {f"NPC_{i}" for i in D42_FORECAST}
extras = forecast | d35def | set(D125_TIP) | set(D125_DEFERRED)

union562 = on | extras
newunion = nn | extras | d42fc
print("committed 562-union size:", len(union562))
print("old pushed:", len(on), "| new pushed:", len(nn),
      "| branches:", len(new), "(was", len(old), ")")
newnames = sorted(nn - on)
already = [n for n in newnames if n in union562]
genuinely = [n for n in newnames if n not in union562]
print("NEW pushed names since 504:", len(newnames),
      "| already in 562-union:", len(already), already)
print("genuinely NEW to the union:", len(genuinely))
print("NEW full union size:", len(newunion), "(was", len(union562), ")")
print("dropped from union:", sorted(union562 - newunion))
print("d42 FORECAST additions (NO-PACK verdict, not pushed rows):",
      len(sorted(d42fc - nn)), sorted(d42fc - nn))

def owners_of(n):
    return sorted({b.replace("origin/task/","") for b,v in new.items() if n in v})
print("\nGenuinely-new names + owners:")
for n in genuinely:
    print("  %-28s %s" % (n, ",".join(owners_of(n))))
print("\nNEW branches (no rows in retire extraction):")
for b in sorted(set(new) - set(old)):
    print("  %s: %d rows" % (b, len(new[b])))
print("\nBranch row-count changes (existing branches):")
for b in sorted(set(new) & set(old)):
    if set(new[b]) != set(old[b]):
        d = sorted(set(new[b]) - set(old[b])); g = sorted(set(old[b]) - set(new[b]))
        print("  %s: %d -> %d, +%s -%s" % (b, len(old[b]), len(new[b]), d, g))
json.dump({
  "newPushed": newnames, "alreadyIn562": already, "genuinelyNew": genuinely,
  "d42Forecast": sorted(d42fc - nn),
  "newUnionSize": len(newunion), "oldUnionSize": len(union562),
  "owners": {n: owners_of(n) for n in newnames},
  "newUnion": sorted(newunion),
}, open(os.path.join(HERE, "_d35_seal_delta.json"), "w"), indent=1)
print("\nwrote _d35_seal_delta.json")
