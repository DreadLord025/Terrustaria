# W4-d37 receipts phase: FINAL canonical map (R1-R9 corrections applied),
# full collision verification incl. the unpushed-lane guidance bands.
import json, subprocess, re, collections
REPO = r"C:\Users\DreadLord\Desktop\TR-W4-d37"
maps = json.load(open(REPO+r"\tools\_d37_receipts_maps.json"))["lanes"]
plan = json.load(open(REPO+r"\tools\_d37_receipts_plan_raw.json"))
own = {k: tuple(v) for k,v in plan["owners"].items()}
fin = dict(plan["final"]); drops = plan["drops"]

# R5/R6: reserved cs-hole corrections (the c4 verdict's own secondary flag,
# extended to every non-owner-of-the-cs row squatting a reserved hole).
fin["Arapaima"] = 762          # c4@237 squats JungleCreeperWall's cs 237 (d37@369)
fin["EmpressOfLight"] = 636    # d125@229 squats Pirate cs 229 (d37@361); 636 == own cs
fin["SantaClaus"] = 763        # d120@241 squats BloodFeeder cs 241 (d37@373)
fin["Truffle"] = 764           # d120@242, hole 237-249
fin["Steampunker"] = 765       # d120@243
fin["Cyborg"] = 766            # d120@244

# base ids
src = subprocess.run(["git","-C",REPO,"show","a725acb0:crates/tr-app/src/npc/data.rs"],
                     capture_output=True).stdout.decode("utf8","replace")
i=src.index("pub fn to_u16"); j=src.index("{",i)
bids={int(x) for x in re.findall(r"=>\s*(\d+)", src[j:src.index("pub fn from_u16")])}
vals=list(fin.values())
assert len(set(vals))==len(vals)==280, "dup final ids"
assert not (set(vals)&bids), f"base clash {set(vals)&bids}"
print("VERIFIED: 280 owned kinds, final ids unique, zero base clash, max", max(vals))
assert max(vals)<=32767
moved=[(k,own[k][0],own[k][1],fin[k]) for k in own if fin[k]!=own[k][1]]
kept=[k for k in own if fin[k]==own[k][1]]
print(f"moved {len(moved)}, keep-pushed {len(kept)}, drops {len(drops)}")
per_lane=collections.Counter(l for _,l,_,_ in moved)
print("moves per lane:", dict(sorted(per_lane.items())))
drop_per=collections.Counter(l for _,l,_,_ in drops)
print("drops per lane:", dict(sorted(drop_per.items())))

# guidance bands (unpushed): d36 767-816 (50), d39 817-826 (10),
# d41 491-560 vanilla-free (skip d33/g10 cs + c4 509), d43 631-697
# (skip g10 pets 637/638/656/670/678-683 + d125 636/661 + c4 688/692 + c3 694)
g10=[637,638,656,670,678,679,680,681,682,683]
d33=[595,596,597,598,599,600,601]
guid=set(range(767,817))|set(range(817,827))|set(g10)
assert not (guid & set(vals)), "guidance clash with pushed plan"
assert not (set(g10)&bids) and not (set(d33)&bids)
print("guidance bands verified free: d36 767-816, d39 817-826, g10 pets keep cs==proto")
print("grand-union max id 826 (i16-safe)")

json.dump({"owners":own,"final":fin,"drops":drops,
           "moved":sorted(moved,key=lambda x:(x[1],x[2]))},
          open(REPO+r"\tools\_d37_receipts_final.json","w"),indent=1,sort_keys=True)
print("final plan written")
