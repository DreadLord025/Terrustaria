# W4-d37 E8-SEAL: slot-gap itemization + stale-decl check at current tips.
import re, subprocess, json, collections
REPO = r"C:\Users\DreadLord\Desktop\TR-W4-d37"
def show(rev, path):
    r = subprocess.run(["git", "-C", REPO, "show", f"{rev}:{path}"], capture_output=True)
    return r.stdout.decode("utf8", "replace") if r.returncode == 0 else None
census = json.load(open(REPO + r"\tools\_d37_e8s_census.json"))["lanes"]
plan = json.load(open(REPO + r"\tools\_d37_e8s_plan.json"))
counts = json.load(open(REPO + r"\tools\_d37_e8s_counts.json"))
owners = {k: tuple(v) for k, v in plan["owners"].items()}
# canonical net per owned kind (from owner's ENTRIES)
net_of = {}
for k, (l, pid) in owners.items():
    n = census[l]["new_entries"].get(k)
    if n and n > 0: net_of[k] = n
# surviving slot set
def slots(rev):
    src = show(rev, "crates/tr-app/src/assets.rs")
    m = re.search(r"BESTIARY_NPC_SHEET_IDS:\s*\[u16;\s*(\d+)\]\s*=\s*\[(.*?)\];", src, re.S)
    return [int(x) for x in re.findall(r"\b(\d+)\b", re.sub(r"//[^\n]*", "", m.group(2)))]
surv = set(slots("a725acb04"))
for l, d in census.items():
    if l == "base": continue
    surv |= set(slots(d["rev"]))
gaps = {k: n for k, n in net_of.items() if n not in surv}
# deliberate non-gaps: d33's 5 hidden dragonfly colours (Hide folds onto Red 599,
# R3/d33 census) + d35's 6 documented sheet deferrals (research/113 lazy wave).
DELIBERATE = {"BlackDragonfly","BlueDragonfly","GreenDragonfly","OrangeDragonfly",
              "YellowDragonfly","Unicorn","Werewolf","Clown","SkeletonArcher",
              "ChaosElemental","PossessedArmor","OwlMimic"}
gaps = {k: n for k, n in gaps.items() if k not in DELIBERATE}
by_lane = collections.defaultdict(list)
for k, n in sorted(gaps.items(), key=lambda x: x[1]):
    by_lane[owners[k][0]].append((k, n))
print("REAL slot gaps:", len(gaps))
for l in sorted(by_lane):
    print(f"  {l}: {len(by_lane[l])} — " + ", ".join(f"{k} {n}" for k, n in by_lane[l]))
print("core slot target:", len(surv), "+", len(gaps), "=", len(surv) + len(gaps))
# stale decls at current tips
print("stale ALL_KINDS decls:", [(l, d["decl"], d["ak_len"]) for l, d in census.items()
                                  if l != "base" and (d["decl"] != d["ak_len"] or d["ak_missing_new"])])
json.dump({"gaps": gaps, "by_lane": {l: v for l, v in by_lane.items()},
           "slot_target": len(surv) + len(gaps)},
          open(REPO + r"\tools\_d37_e8s_gaps.json", "w"), indent=1, sort_keys=True)
