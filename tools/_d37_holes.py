import subprocess, re
REPO = r"C:\Users\DreadLord\Desktop\TR-W4-d37"
def show(rev, path):
    return subprocess.run(["git","-C",REPO,"show",f"{rev}:{path}"],
                          capture_output=True).stdout.decode("utf8","replace")
src = show("a725acb0", "crates/tr-app/src/npc/data.rs")
i = src.index("pub fn to_u16")
body = src[i:src.index("pub fn from_u16")]
ids = {int(x) for x in re.findall(r"=>\s*(\d+)", body)}
# reserved cs-hole kinds' cs ids via bestiary entries of base
b = show("a725acb0", "crates/tr-app/src/bestiary.rs")
for name in ("Pirate","JungleCreeperWall","BloodFeeder","GoblinThief","BoundWizard","SantaClaus","Truffle","Steampunker","Cyborg","Angler","SkeletonMerchant"):
    m = re.search(r'credit_id: "%s",\s*net_id: (\d+)' % name, b)
    print(name, m.group(1) if m else "no base entry")
# cs 242/243/244 vanilla identity
nid = open(REPO+r"\decompiled\Terraria\ID\NPCID.cs", encoding="utf8", errors="replace").read()
for v in (229,237,241,242,243,244,245,142,160,178,209,157):
    mm = re.search(r"public const short (\w+) = %d;" % v, nid)
    print(f"NPCID {v}:", mm.group(1) if mm else "NONE")
# are cs 242-245 claimed by any union kind? check maps cs values
import json
maps = json.load(open(REPO+r"\tools\_d37_receipts_maps.json"))["lanes"]
plan = json.load(open(REPO+r"\tools\_d37_receipts_final.json"))
own = {k: tuple(v) for k,v in plan["owners"].items()}
cs_used=set()
for k,(l,p) in own.items():
    c=maps[l]["cs"].get(k)
    if c: cs_used.add(c)
print("cs 242/243/244/245 used by union?", {v: v in cs_used for v in (242,243,244,245)})
print("proto 242/243/244 in base?", {v: v in ids for v in (242,243,244)})
