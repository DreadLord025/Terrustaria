# W4-d37 SEAL: AUTHORITATIVE current-tip slot accounting.
# Rule: a pushed slot id survives the dedupe iff it equals the CANONICAL
# net_id of some owned kind (the bestiary grid is net-keyed; a slot on a
# dropped duplicate port still serves the canonical kind when the net ids
# agree - e.g. d23's VileSpit slot 112 survives because d35 owns VileSpit
# at net 112). Gap = an owned kind with a positive canonical net that no
# surviving slot covers, minus the deliberate Hide folds + d35 deferrals.
import subprocess, re, json
REPO=r"C:\Users\DreadLord\Desktop\TR-W4-d37"
def show(rev,path):
    return subprocess.run(["git","-C",REPO,"show",f"{rev}:{path}"],
                          capture_output=True).stdout.decode("utf8","replace")
TIPS={"base":"a725acb04","c3":"9ea0ee90d","c4":"0df6964be","c6":"5de680219",
 "c7":"c2cc6ec61","c8":"3705670d3","c9":"2042d77f1","c10":"5d8387955",
 "d23":"43091b01c","d120":"2408233da","d125":"86e3adfd3","d33":"b08acf463",
 "d34":"5f3435a87","d35":"e5558b0a9","d37":"a341daa12","d38":"1ba0395c7",
 "d40":"8fe1d6d31","d42":"2d7ac08c3","e26":"048cca6cf","g8":"82c93cecf"}
def slots(rev):
    m=re.search(r"BESTIARY_NPC_SHEET_IDS:\s*\[u16;\s*(\d+)\]\s*=\s*\[(.*?)\];",
                show(rev,"crates/tr-app/src/assets.rs"),re.S)
    return [int(x) for x in re.findall(r"\b(\d+)\b",re.sub(r"//[^\n]*","",m.group(2)))]
def entries(rev):
    src=show(rev,"crates/tr-app/src/bestiary.rs")
    m=re.search(r"const ENTRIES[^=]*=\s*&?\[(.*?)\n\];",src,re.S)
    out=[]
    for blk in re.split(r"BestiaryEntryDef\s*\{",m.group(1))[1:]:
        k=re.search(r"kind:\s*NpcKind::(\w+)",blk); n=re.search(r"net_id:\s*(-?\d+)",blk)
        if k and n: out.append((k.group(1),int(n.group(1))))
    return out
seal=json.load(open(REPO+r"\tools\_d37_seal_final.json"))
own={k:tuple(v) for k,v in seal["owners"].items()}
bslots=slots("a725acb04")
bsrc=show("a725acb04","crates/tr-app/src/npc/data.rs"); i=bsrc.index("pub fn to_u16")
base_names=set(m[0] for m in re.findall(r"NpcKind::(\w+)\s*=>\s*(\d+)",bsrc[i:bsrc.index("pub fn from_u16")]))
per_e={}; adds={}
for l,r in TIPS.items():
    if l=="base": continue
    per_e[l]=[(k,n) for k,n in entries(r) if k not in base_names]
    s=slots(r); assert s[:len(bslots)]==bslots,l
    adds[l]=s[len(bslots):]
net_by={(l,k):n for l,rows in per_e.items() for k,n in rows}
canon={}
for k,(l,pid) in own.items():
    n=net_by.get((l,k))
    if n is None:
        for (ll,kk),nn in net_by.items():
            if kk==k: n=nn; break
    canon[k]=n
canon_nets=set(n for n in canon.values() if n and n>0)
# surviving slots = base + adds whose net is a canonical owned net
survive=set(bslots); dropped=[]
for l,ids in adds.items():
    for i in ids:
        if i in canon_nets: survive.add(i)
        else: dropped.append((l,i))
naive=181+sum(len(v) for v in adds.values())
print("naive append:",naive)
print("slot adds dropped (net not a canonical owned net):",len(dropped))
print("surviving distinct slots:",len(survive))
HIDE={("d33","BlackDragonfly"),("d33","BlueDragonfly"),("d33","GreenDragonfly"),
 ("d33","OrangeDragonfly"),("d33","YellowDragonfly"),("d42","Seagull2"),("d42","Grebe2")}
DEF={"DarkMummy","LightMummy","EnchantedSword","Mimic","Unicorn","GreenJellyfish",
 "Werewolf","Clown","SkeletonArcher","ChaosElemental","Gastropod","PossessedArmor"}
gaps=[]; folds=[]; deferred=[]
for k,(l,pid) in own.items():
    n=canon.get(k)
    if n and n>0 and n not in survive:
        if (l,k) in HIDE: folds.append((l,k,n))
        elif l=="d35" and k in DEF: deferred.append((l,k,n))
        else: gaps.append((l,k,n))
print("Hide folds:",len(folds),"d35 deferrals:",len(deferred),"REAL gaps:",len(gaps))
bylane={}
for l,k,n in sorted(gaps): bylane.setdefault(l,[]).append((k,n))
for l,v in sorted(bylane.items()): print(f"  {l}: {len(v)} -> {v}")
print("CORE slot target (surviving + gap close):",len(survive)+len(gaps))
json.dump({"naive":naive,"dropped_adds":len(dropped),"surviving":len(survive),
           "gaps":gaps,"folds":folds,"deferred":deferred,
           "target":len(survive)+len(gaps),"dropped_detail":dropped},
          open(REPO+r"\tools\_d37_seal_gapfinal.json","w"),indent=1)
