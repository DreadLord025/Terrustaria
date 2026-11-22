import json, subprocess
REPO = r"C:\Users\DreadLord\Desktop\TR-MERGE-1"
plan = json.load(open(REPO+r"\tools\_d37_e8s_plan.json"))
census = json.load(open(REPO+r"\tools\_d37_e8s_census.json"))["lanes"]
scan = json.load(open(REPO+r"\tools\_m1_lanescan.json"))
print("plan keys:", list(plan.keys()))
owners = plan["owners"]; final = dict(plan["final"]); drops = plan["dropped_names"]
print("owners:", len(owners), "final:", len(final), "dropped_names:", len(drops))
# census tip vs current tip per lane
for lane, info in sorted(census.items()):
    cur = scan.get(lane, {}).get("rev")
    if cur and info.get("rev","")[:9] != cur:
        print(f"TIP-MOVED {lane}: census {info['rev'][:9]} -> now {cur}")
# drop structure sample
print("drop sample:", drops[:3])
# owner lanes for the contested kinds
for k in ["PartyBunny","Tumbleweed","SquirrelRed","SquirrelGold","GiantFlyingAntlion","SolarFlare","Worm","Shimmerfly","Frog"]:
    print(k, "owner:", owners.get(k), "final:", final.get(k), "drops:", [d for d in drops if (d[0] if isinstance(d,list) else d)==k])
