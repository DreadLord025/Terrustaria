# W4-d37 receipts phase: extend the E-census to the CURRENT tips (post-E0
# movers), re-derive the count pins for the receipts doc. Git-show only.
import re, subprocess, json

REPO = r"C:\Users\DreadLord\Desktop\TR-W4-d37"
def show(rev, path):
    r = subprocess.run(["git","-C",REPO,"show",f"{rev}:{path}"],capture_output=True)
    if r.returncode: raise RuntimeError(f"{rev}:{path}: {r.stderr.decode()[:150]}")
    return r.stdout.decode("utf-8","replace")

DATA="crates/tr-app/src/npc/data.rs"; NPC="crates/tr-app/src/npc.rs"
BEST="crates/tr-app/src/bestiary.rs"; ASSETS="crates/tr-app/src/assets.rs"
AUDIO="crates/tr-app/src/audio.rs"

def fn_body(src,sig):
    i=src.index(sig); j=src.index("{",i); d=0; k=j
    while True:
        c=src[k]
        if c=="{": d+=1
        elif c=="}":
            d-=1
            if d==0: return src[j:k+1]
        k+=1

def arms_to(src):
    return {m[0]:int(m[1]) for m in re.findall(r"NpcKind::(\w+)\s*=>\s*(\d+)",fn_body(src,"pub fn to_u16"))}

TIPS={"base":"a725acb0","c3":"9299046f7","c4":"17a241f6a","c6":"5de68021",
 "c7":"c2cc6ec6","c8":"3705670d","c9":"2042d77f","c10":"9641e20db",
 "d23":"19109424","d120":"c5e68a80","d125":"cd2b1b1e","d33":"b08acf463",
 "d34":"5f3435a8","d35":"6a2a155a","d37":"bdd59afb","d38":"6f9dd02d",
 "d40":"8fe1d6d3","d42":"9b22c091","e26":"048cca6c","g8":"82c93cec"}

plan=json.load(open(REPO+r"\tools\_d37_receipts_plan_raw.json"))
owners={k:tuple(v) for k,v in plan["owners"].items()}
drops=plan["drops"]

# --- bestiary ENTRIES census per tip
def entries(rev):
    src=show(rev,BEST)
    m=re.search(r"const ENTRIES[^=]*=\s*&?\[(.*?)\n\];",src,re.S)
    body=m.group(1)
    rows=re.findall(r"kind:\s*NpcKind::(\w+).*?net_id:\s*(\d+)",body,re.S)
    # safer: split per BestiaryEntryDef
    out=[]
    for blk in re.split(r"BestiaryEntryDef\s*\{",body)[1:]:
        k=re.search(r"kind:\s*NpcKind::(\w+)",blk)
        n=re.search(r"net_id:\s*(-?\d+)",blk)
        if k and n: out.append((k.group(1),int(n.group(1))))
    return out

base_entries=entries("a725acb0")
base_kind_set=set(arms_to(show("a725acb0",DATA)))
print(f"base ENTRIES rows: {len(base_entries)}")
per={}
for lane,rev in TIPS.items():
    if lane=="base": continue
    e=entries(rev)
    new=[(k,n) for k,n in e if k not in base_kind_set]
    per[lane]=new
    print(f"{lane:5s} ENTRIES {len(e)} (new-kind rows {len(new)})")

# union new-entry rows deduped by kind (canonical owner keeps its row)
seen=set(); union_new=[]
for lane,rev in TIPS.items():
    if lane=="base": continue
    for k,n in per[lane]:
        if k in seen: continue
        if owners.get(k,(None,))[0]!=lane: continue
        seen.add(k); union_new.append((k,n,lane))
print(f"\nunion: base rows {len(base_entries)} + owned new-kind rows {len(union_new)} = {len(base_entries)+len(union_new)}")
missing=[k for k in owners if k not in seen]
print(f"owned kinds WITHOUT any new ENTRIES row: {len(missing)}")
print(sorted(missing))
# net_id distinctness in the union (positive rows)
nets=[n for _,n in base_entries]+[n for _,n,_ in union_new]
pos=[n for n in nets if n>0]
dups=[n for n in set(pos) if pos.count(n)>1]
print(f"union positive net ids: {len(pos)}, distinct {len(set(pos))}, dup ids {sorted(dups)}")

# --- static NPC sheets union
def sheets(rev):
    src=show(rev,ASSETS)
    m=re.search(r"STATIC_SHEET_NAMES[^=]*=\s*&?\[(.*?)\n\];",src,re.S)
    body=re.sub(r"//[^\n]*","",m.group(1))
    return re.findall(r'"(NPC_[^"]+)"',body)
base_sheets=set(sheets("a725acb0"))
u=set(base_sheets)
for lane,rev in TIPS.items():
    if lane=="base": continue
    u |= set(sheets(rev))
print(f"\nSTATIC NPC_ sheets: base {len(base_sheets)}, union {len(u)}")

# --- portrait slots union (as pushed)
def slots(rev):
    src=show(rev,ASSETS)
    m=re.search(r"BESTIARY_NPC_SHEET_IDS:\s*\[u16;\s*(\d+)\]\s*=\s*\[(.*?)\];",src,re.S)
    ids=[int(x) for x in re.findall(r"\b(\d+)\b",re.sub(r"//[^\n]*","",m.group(2)))]
    return int(m.group(1)),ids
tot=0
for lane,rev in TIPS.items():
    d,ids=slots(rev)
    assert d==len(ids),(lane,d,len(ids))
    if lane!="base": tot+=len(ids)-181
    print(f"{lane:5s} slots {d}")
print(f"naive slot union: 181 + {tot} = {181+tot}")

# --- SFX cues union
def cues(rev):
    src=show(rev,AUDIO)
    m=re.search(r"SFX_CUES[^=]*=\s*&?\[(.*?)\n\];",src,re.S)
    return re.findall(r'"(NPC_(?:Hit|Killed)_\d+)"',m.group(1))
base_cues=set(cues("a725acb0"))
uc=set(base_cues); naive=len(base_cues)
for lane,rev in TIPS.items():
    if lane=="base": continue
    c=set(cues(rev)); uc|=c; naive+=len(c-base_cues)
print(f"\nNPC cues: base {len(base_cues)}, naive append {naive}, unique union {len(uc)}")

json.dump({"union_new_entries":union_new,"missing_entry_kinds":sorted(missing),
           "static_union":len(u),"slot_naive":181+tot,"cue_unique":len(uc),
           "per_lane_new_entries":per},
          open(REPO+r"\tools\_d37_receipts_counts.json","w"),indent=1)
print("counts written")
