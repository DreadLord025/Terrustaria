# Final slot accounting: naive union, dedupe drops, remaining gaps.
import subprocess, re, json
REPO = r"C:\Users\DreadLord\Desktop\TR-W4-d37"
def show(rev, path):
    return subprocess.run(["git","-C",REPO,"show",f"{rev}:{path}"],
                          capture_output=True).stdout.decode("utf8","replace")
TIPS={"base":"a725acb0","c3":"9299046f7","c4":"17a241f6a","c6":"5de68021",
 "c7":"c2cc6ec6","c8":"3705670d","c9":"2042d77f","c10":"9641e20db",
 "d23":"19109424","d120":"c5e68a80","d125":"cd2b1b1e","d33":"b08acf463",
 "d34":"5f3435a8","d35":"6a2a155a","d37":"bdd59afb","d38":"6f9dd02d",
 "d40":"8fe1d6d3","d42":"9b22c091","e26":"048cca6c","g8":"82c93cec"}
def slots(rev):
    src=show(rev,"crates/tr-app/src/assets.rs")
    m=re.search(r"BESTIARY_NPC_SHEET_IDS:\s*\[u16;\s*(\d+)\]\s*=\s*\[(.*?)\];",src,re.S)
    return [int(x) for x in re.findall(r"\b(\d+)\b",re.sub(r"//[^\n]*","",m.group(2)))]
bslots=slots("a725acb0")
maps=json.load(open(REPO+r"\tools\_d37_receipts_maps.json"))["lanes"]
plan=json.load(open(REPO+r"\tools\_d37_receipts_plan_raw.json"))
own={k:tuple(v) for k,v in plan["owners"].items()}
drops=plan["drops"]  # (kind, dropped_lane, pushed_id, keeper)
counts=json.load(open(REPO+r"\tools\_d37_receipts_counts.json"))
# per-lane slot adds (beyond base 181, in order)
adds={}
for l,rev in TIPS.items():
    if l=="base": continue
    s=slots(rev)
    assert s[:len(bslots)]==bslots, f"{l} reordered base slots!"
    adds[l]=s[len(bslots):]
# map dropped kind -> its bestiary net_id on the DROPPING lane's tip
# (slots are net_id-keyed; cs_id can be a folded sentinel)
net_by={ (l,k):n for l,rows in counts["per_lane_new_entries"].items() for k,n in rows }
drop_net={(l, net_by.get((l,k))) for k,l,pid,keep in drops}
naive=181+sum(len(v) for v in adds.values())
dupslots=0
for l,ids in adds.items():
    for i in ids:
        if (l,i) in drop_net: dupslots+=1
print("naive slot union:",naive)
print("slots sitting on DROPPED duplicate ports:",dupslots)
print("deduped as-pushed slots:",naive-dupslots)
# which owned visible-entry kinds lack a slot anywhere?
slot_ids=set(bslots)
for l,ids in adds.items():
    for i in ids:
        if (l,i) not in drop_net: slot_ids.add(i)
# c4's tip added NO slots but its kinds are fully wired with packed sheets
# (STYLE44/STYLE16_SHEETS) -> its 5 visible entries are KNOWN slot-gap
# closes, not unknowns; same for c7's FIREFLY_SHEETS (4) and c8's
# TruffleWormDigger (packed in CRAWLER_SHEETS).
missing=[]
HIDE_FOLDS={  # deliberate credit folds: own ENTRIES row, no grid slot
 ("d33","BlackDragonfly"),("d33","BlueDragonfly"),("d33","GreenDragonfly"),
 ("d33","OrangeDragonfly"),("d33","YellowDragonfly"),
 ("d42","Seagull2"),("d42","Grebe2"),
}
DEFERRED_D35={  # d35's documented 12 deferred draw sheets (atlas repack)
 "DarkMummy","LightMummy","EnchantedSword","Mimic","Unicorn",
 "GreenJellyfish","Werewolf","Clown","SkeletonArcher","ChaosElemental",
 "Gastropod","PossessedArmor"}
gaps=[]; folds=0; deferred=0
for k,n,l in counts["union_new_entries"]:
    if n>0 and n not in slot_ids:
        if (l,k) in HIDE_FOLDS: folds+=1
        elif l=="d35" and k in DEFERRED_D35: deferred+=1
        else: gaps.append((l,k,n))
print("owned entries w/o slot: deliberate Hide folds",folds,
      "| d35 documented deferrals",deferred,"| REAL gaps",len(gaps))
bylane={}
for l,k,n in gaps: bylane.setdefault(l,[]).append((k,n))
for l,v in sorted(bylane.items()): print(f"  {l}: {len(v)} -> {v}")
print("final slot target (deduped as-pushed + real gap close):",naive-dupslots+len(gaps))
