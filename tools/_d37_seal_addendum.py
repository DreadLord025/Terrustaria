# W4-d37 SEAL: addendum census for the post-receipts claimants (d39/d41/
# d23-new/g10-local) + the d40 38-vs-37 slot question. Git-show only.
import re, subprocess, json

REPO = r"C:\Users\DreadLord\Desktop\TR-W4-d37"
def show(rev, path):
    r = subprocess.run(["git","-C",REPO,"show",f"{rev}:{path}"],capture_output=True)
    return r.stdout.decode("utf-8","replace") if r.returncode==0 else None

def entries(rev):
    src=show(rev,"crates/tr-app/src/bestiary.rs")
    m=re.search(r"const ENTRIES[^=]*=\s*&?\[(.*?)\n\];",src,re.S)
    out=[]
    for blk in re.split(r"BestiaryEntryDef\s*\{",m.group(1))[1:]:
        k=re.search(r"kind:\s*NpcKind::(\w+)",blk)
        n=re.search(r"net_id:\s*(-?\d+)",blk)
        if k and n: out.append((k.group(1),int(n.group(1))))
    return out

def sheets(rev):
    src=show(rev,"crates/tr-app/src/assets.rs")
    m=re.search(r"STATIC_SHEET_NAMES[^=]*=\s*&?\[(.*?)\n\];",src,re.S)
    return re.findall(r'"(NPC_[^"]+)"',re.sub(r"//[^\n]*","",m.group(1)))

def slots(rev):
    src=show(rev,"crates/tr-app/src/assets.rs")
    m=re.search(r"BESTIARY_NPC_SHEET_IDS:\s*\[u16;\s*(\d+)\]\s*=\s*\[(.*?)\];",src,re.S)
    return [int(x) for x in re.findall(r"\b(\d+)\b",re.sub(r"//[^\n]*","",m.group(2)))]

def cues(rev):
    src=show(rev,"crates/tr-app/src/audio.rs")
    m=re.search(r"SFX_CUES[^=]*=\s*&?\[(.*?)\n\];",src,re.S)
    return re.findall(r'"(NPC_(?:Hit|Killed)_\d+)"',m.group(1))

census=json.load(open(REPO+r"\tools\_d37_seal_census.json"))["lanes"]
base_k=set(census["base"]["new"])  # empty; use base to arms instead
bsrc=show("a725acb04","crates/tr-app/src/npc/data.rs")
i=bsrc.index("pub fn to_u16")
base_names=set(m[0] for m in re.findall(r"NpcKind::(\w+)\s*=>\s*(\d+)",bsrc[i:bsrc.index("pub fn from_u16")]))
be=set(k for k,_ in entries("a725acb04"))
bs=set(sheets("a725acb04")); bsl=slots("a725acb04"); bc=set(cues("a725acb04"))

NEW={"d39":"907234441","d41":"5d036721d","d23":"43091b01c","g10L":"5dee28650"}
add={"entries":[],"sheets":set(),"slots":[],"cues":set()}
seen_e=set()
for lane,rev in NEW.items():
    e=[(k,n) for k,n in entries(rev) if k not in base_names]
    # d23: only the genuinely-new owned kinds count (VileSpit is a d35 dup)
    if lane=="d23":
        e=[(k,n) for k,n in e if k in ("SolarFlare","VileSpitEoW")]
    fresh=[(k,n,lane) for k,n in e if k not in seen_e]
    seen_e.update(k for k,_,_ in fresh)
    add["entries"]+=fresh
    sh=set(sheets(rev))-bs
    sl=slots(rev)[len(bsl):]
    cu=set(cues(rev))-bc
    print(f"{lane:5s} new ENTRIES {len(fresh)} {fresh}")
    print(f"      sheets +{len(sh)} slots +{len(sl)} {sl} cues +{len(cu)}")
    add["sheets"]|=sh; add["slots"]+=sl; add["cues"]|=cu
print("addendum totals: entries",len(add["entries"]),
      "sheets",len(add["sheets"]),"slots",len(add["slots"]),
      "cues",len(add["cues"]))
# slot dedupe within addendum + vs core? report raw; bible keeps core pins.
print("addendum slot ids distinct:",len(set(add["slots"])))

# d40: which of its 38 new ENTRIES rows is NOT in the 37-gap list?
counts=json.load(open(REPO+r"\tools\_d37_receipts_counts.json"))
d40rows=counts["per_lane_new_entries"]["d40"]
gaps=[('CultistTablet',437),('CultistDevote',438),('CultistBossClone',440),('GoldBird',442),('GoldBunny',443),('GoldButterfly',444),('GoldFrog',445),('GoldGrasshopper',446),('GoldMouse',447),('GoldWorm',448),('SkeletonMerchant',453),('CultistDragonHead',454),('Butcher',460),('CreatureFromTheDeep',461),('Fritz',462),('Nailhead',463),('CrimsonBunny',464),('Psycho',466),('DeadlySphere',467),('DrManFly',468),('ThePossessed',469),('CrimsonPenguin',470),('ShadowFlameApparition',472),('BigMimicCorruption',473),('BigMimicCrimson',474),('BigMimicHallow',475),('BigMimicJungle',476),('Mothron',477),('MothronSpawn',479),('Medusa',480),('GreekSkeleton',481),('GraniteGolem',482),('GraniteFlyer',483),('EnchantedNightcrawler',484),('Grubby',485),('Sluggy',486),('Buggy',487)]
gs=set(k for k,_ in gaps)
print("d40 rows w/o gap (already slotted or folded):",[(k,n) for k,n in d40rows if k not in gs])
print("d40 gap kinds not in rows:",[k for k in gs if k not in set(k2 for k2,_ in d40rows)])
# d40 slots on its own tip
sl40=slots("8fe1d6d31")
print("d40 tip slot count:",len(sl40),"adds:",sl40[len(bsl):])
json.dump({"entries":add["entries"],"sheets":sorted(add["sheets"]),
           "slots":add["slots"],"cues":sorted(add["cues"])},
          open(REPO+r"\tools\_d37_seal_addendum.json","w"),indent=1)
