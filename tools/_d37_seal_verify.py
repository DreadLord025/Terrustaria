# W4-d37 SEAL final verify: amend the canonical plan for the executions
# that already happened (d42's 8 ownership-flip drops, c4's 405/406), add
# the new claimants, re-derive the core + grand count pins. Git-show only.
import re, subprocess, json
REPO=r"C:\Users\DreadLord\Desktop\TR-W4-d37"
def show(rev,path):
    return subprocess.run(["git","-C",REPO,"show",f"{rev}:{path}"],
                          capture_output=True).stdout.decode("utf8","replace")
def entries(rev):
    src=show(rev,"crates/tr-app/src/bestiary.rs")
    m=re.search(r"const ENTRIES[^=]*=\s*&?\[(.*?)\n\];",src,re.S)
    out=[]
    for blk in re.split(r"BestiaryEntryDef\s*\{",m.group(1))[1:]:
        k=re.search(r"kind:\s*NpcKind::(\w+)",blk)
        n=re.search(r"net_id:\s*(-?\d+)",blk)
        if k and n: out.append((k.group(1),int(n.group(1))))
    return out
census=json.load(open(REPO+r"\tools\_d37_seal_census.json"))["lanes"]
plan=json.load(open(REPO+r"\tools\_d37_receipts_final.json"))
owners={k:list(v) for k,v in plan["owners"].items()}
final=dict(plan["final"])
# --- amendments (seal adjudication)
for k in ("FlyingAntlion","EyeballFlyingFish","GoldGoldfish","Pupfish","Dolphin"):
    owners[k]=["c4",census["c4"]["new"][k]]      # d42 dropped -> c4 keeps
for k in ("LadyBug","GoldLadyBug"):
    owners[k]=["e26",census["e26"]["new"][k]]    # d42 dropped -> e26 keeps
owners["Maggot"]=["c8",census["c8"]["new"]["Maggot"]]
final["Arapaima"]=406                            # c4 executed 406 (free)
# c10's 12 roster-tail dup ports drop to d40/c8 (owners already correct)
# d23's VileSpit dup drops to d35; its 2 new kinds are owned:
owners["SolarFlare"]=["d23",408]; final["SolarFlare"]=408
owners["VileSpitEoW"]=["d23",411]; final["VileSpitEoW"]=411
# --- core collision check
ids={}
clash=[]
for k,(l,pid) in owners.items():
    v=final[k]
    if v in ids: clash.append((v,ids[v],k))
    ids[v]=k
print("core owned:",len(owners),"distinct final ids:",len(ids),"clashes:",clash)
base_ids=set()
bsrc=show("a725acb04","crates/tr-app/src/npc/data.rs")
i=bsrc.index("pub fn to_u16")
base_to={m[0]:int(m[1]) for m in re.findall(r"NpcKind::(\w+)\s*=>\s*(\d+)",bsrc[i:bsrc.index("pub fn from_u16")])}
print("base clash:",[ (k,v) for k,v in final.items() if v in set(base_to.values())])
print("core ALL_KINDS pin:",270+len(owners),"max id:",max(ids))
# --- ENTRIES union at CURRENT tips with amended owners
CUR={"c3":"9ea0ee90d","c4":"0df6964be","c6":"5de680219","c7":"c2cc6ec61",
 "c8":"3705670d3","c9":"2042d77f1","c10":"5d8387955","d23":"43091b01c",
 "d120":"2408233da","d125":"86e3adfd3","d33":"b08acf463","d34":"5f3435a87",
 "d35":"e5558b0a9","d37":"a341daa12","d38":"1ba0395c7","d40":"8fe1d6d31",
 "d42":"2d7ac08c3","e26":"048cca6cf","g8":"82c93cecf"}
be=entries("a725acb04")
base_names=set(base_to)
per={l:[(k,n) for k,n in entries(r) if k not in base_names] for l,r in CUR.items()}
seen=set(); union=[]
for l in CUR:
    for k,n in per[l]:
        if k in seen: continue
        if owners.get(k,[None])[0]!=l: continue
        seen.add(k); union.append((k,n,l))
missing=[k for k in owners if k not in seen]
nets=[n for _,n in be]+[n for _,n,_ in union]
pos=[n for n in nets if n>0]
dups=sorted(n for n in set(pos) if pos.count(n)>1)
print("core ENTRIES:",len(be),"+",len(union),"=",len(be)+len(union),
      "| no-entry owned:",len(missing),"| pos nets:",len(pos),
      "distinct:",len(set(pos)),"dups:",dups)
print("no-entry list:",sorted(missing))
print("VileSpit entry on d23:",[x for x in per["d23"] if x[0]=="VileSpit"])
print("d42 GoldGoldfishWalker entry:",[x for x in per["d42"] if x[0]=="GoldGoldfishWalker"])
# --- grand union (future lanes + local g10)
grand=dict(final)
for l in ("d39","d41"):
    for k,v in census[l]["new"].items(): grand[k]=v
g10=show("5dee28650","crates/tr-app/src/npc/data.rs")
i=g10.index("pub fn to_u16")
g10to={m[0]:int(m[1]) for m in re.findall(r"NpcKind::(\w+)\s*=>\s*(\d+)",g10[i:g10.index("pub fn from_u16")])}
for k,v in g10to.items():
    if k not in base_names: grand[k]=v
gid={}
gclash=[]
for k,v in grand.items():
    if v in gid: gclash.append((v,gid[v],k))
    gid[v]=k
print("grand owned:",len(grand),"ALL_KINDS pin:",270+len(grand),
      "max:",max(gid),"clashes:",gclash,
      "base clash:",[k for k,v in grand.items() if v in set(base_to.values())])
ge=len(be)+len(union)
for l in ("d39","d41"):
    rows=[(k,n) for k,n in entries(CUR.get(l,{"d39":"907234441","d41":"5d036721d"}[l])) if k not in base_names and k not in seen]
    ge+=len(rows); seen.update(k for k,_ in rows)
g10e=[(k,n) for k,n in entries("5dee28650") if k not in base_names and k not in seen]
ge+=len(g10e)
print("grand ENTRIES:",ge)
json.dump({"owners":owners,"final":final,"grand":grand,
           "core_entries":len(be)+len(union),"grand_entries":ge,
           "core_no_entry":sorted(missing)},
          open(REPO+r"\tools\_d37_seal_final.json","w"),indent=1,sort_keys=True)
