# W4-d37 E8-SEAL: per-lane move/keep/drop accounting + grand-union totals +
# the refreshed E8 old->new tables. Pure analysis over the plan + census.
import json, collections

REPO = r"C:\Users\DreadLord\Desktop\TR-W4-d37"
census = json.load(open(REPO + r"\tools\_d37_e8s_census.json"))["lanes"]
plan = json.load(open(REPO + r"\tools\_d37_e8s_plan.json"))
owners = {k: tuple(v) for k, v in plan["owners"].items()}
final = plan["final"]
dropped_names = set(plan["dropped_names"])
LANES = ["c3","c4","c6","c7","c8","c9","c10","d9","d11","d23","d120","d125",
         "d33","d34","d35","d36","d37","d38","d39","d40","d41","d42","d43",
         "e26","f18","f42","f47","g5","g8","g10","c7b","d103","e21","g62"]

# per-lane: owned kinds (move/keep), dropped ports (dup of another owner)
acct = {}
total_move = total_keep = total_drop = 0
for l in LANES:
    new = census[l]["new"]
    moves = []; keeps = []; drops = []
    for k, pid in sorted(new.items(), key=lambda x: x[1]):
        if k in dropped_names:
            drops.append((k, pid, "collapsed-name")); continue
        ow = owners.get(k)
        if ow is None:
            drops.append((k, pid, "UNOWNED?")); continue
        if ow[0] != l:
            drops.append((k, pid, ow[0] + "@" + str(final[k])))
        else:
            fid = final[k]
            (moves if fid != pid else keeps).append((k, pid, fid))
    acct[l] = {"moves": moves, "keeps": keeps, "drops": drops}
    total_move += len(moves); total_keep += len(keeps); total_drop += len(drops)
    print(f"{l:6s} owned {len(moves)+len(keeps):3d} (move {len(moves):3d} keep {len(keeps):3d}) drop {len(drops):3d}")
print(f"TOTAL: move {total_move} keep {total_keep} drop {total_drop}")
print(f"owned kinds: {len(owners)} | ALL_KINDS pin: {270+len(owners)} | max id: {max(final.values())}")

# grand union = the full plan (all lanes already in). Confirm no future lane outside.
print("grand-union owned:", len(owners), "max:", max(final.values()),
      "i16-safe:", max(final.values()) <= 32767)

json.dump(acct, open(REPO + r"\tools\_d37_e8s_acct.json", "w"), indent=1, sort_keys=True)
print("accounting written")
