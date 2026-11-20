import re, json, os, sys
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
def rd(p): return open(os.path.join(ROOT,p),encoding='utf-8').read()

loot = dict((m.group(1), int(m.group(2))) for m in re.finditer(r'pub const (\w+): i32 = (-?\d+);', rd(r'crates\tr-app\src\loot_db.rs')))
bes = rd(r'crates\tr-app\src\bestiary.rs')
kind_net = {}
for m in re.finditer(r'BestiaryEntryDef \{(.*?)\}', bes, re.S):
    b = m.group(1)
    k = re.search(r'kind: NpcKind::(\w+)', b)
    n = re.search(r'net_id: (?:crate::loot_db::(\w+)|(-?\d+))', b)
    if k and n:
        nid = loot.get(n.group(1)) if n.group(1) else int(n.group(2))
        if nid is not None:
            kind_net.setdefault(k.group(1), set()).add(nid)

data = rd(r'crates\tr-app\src\npc\data.rs')
cs = data[data.index('pub(crate) fn cs_id'):data.index('pub fn humanoid_frame_count')]
kind_cs = {}
for m in re.finditer(r'NpcKind::(\w+)(?:\s*\|\s*NpcKind::(\w+))*\s*=>\s*(\d+)', cs):
    # handle alternation: find full arm text
    pass
for m in re.finditer(r'((?:NpcKind::\w+\s*\|?\s*)+)=>\s*(\d+)', cs):
    for k in re.findall(r'NpcKind::(\w+)', m.group(1)):
        kind_cs[k] = int(m.group(2))

npcrs = rd(r'crates\tr-app\src\npc.rs')
en = npcrs[npcrs.index('pub enum NpcKind {'):]
en = en[:en.index('\n}')]
variants = re.findall(r'^\s{4}(\w+),', en, re.M)

covered = {}
for k in variants:
    ids = set()
    if k in kind_cs: ids.add(kind_cs[k])
    ids |= kind_net.get(k,set())
    for i in ids:
        if 71 <= i <= 140: covered.setdefault(i, []).append(k)

# NPCID.cs constants
npcid = rd(r'decompiled\Terraria\ID\NPCID.cs')
id2name = {}
for m in re.finditer(r'public const short (\w+) = (-?\d+);', npcid):
    id2name.setdefault(int(m.group(2)), []).append(m.group(1))

missing = [i for i in range(71,141) if i not in covered]
print("COVERED 71-140:")
for i in sorted(covered): print(f"  {i}: {','.join(covered[i])} ({'/'.join(id2name.get(i,['?']))})")
print("MISSING 71-140:")
for i in missing: print(f"  {i}: {'/'.join(id2name.get(i,['?']))}")
print("total missing:", len(missing))
json.dump({'missing':missing,'covered':{str(k):v for k,v in covered.items()},'id2name':{str(k):v for k,v in id2name.items() if 0<=k<=700}}, open(os.path.join(ROOT,'tools','_d35_census.json'),'w'), indent=0)
