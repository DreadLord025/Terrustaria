import re, os, json
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
def rd(p): return open(os.path.join(ROOT,p),encoding='utf-8').read()
ids = [73,74,75,77,78,79,80,82,83,84,85,86,93,94,95,96,97,98,99,100,101,102,103,104,109,110,112,116,120,121,122,137,140]
npcid = rd(r'decompiled\Terraria\ID\NPCID.cs')
lines = npcid.split('\n')
out = []
# DebuffImmunitySets entries: pattern `{ id, new NPCDebuffImmunityData ... }` - find blocks keyed by id
for w in ids:
    hits = [i for i,l in enumerate(lines) if re.search(r'\{\s*%d\s*,' % w, l) and 'Immune' in '\n'.join(lines[max(0,i-2):i+8])]
    out.append(f"== id {w} debuff hits at {[h+1 for h in hits]}")
    for h in hits:
        for k in range(h, min(h+10, len(lines))):
            out.append(f"{k+1}: {lines[k].rstrip()}")
            if k>h and re.search(r'^\s*\}', lines[k]): break
open(os.path.join(ROOT,'tools','_d35_debuff.txt'),'w',encoding='utf-8').write('\n'.join(out))
# bestiary populator folds
pop = rd(r'decompiled\Terraria\GameContent\Bestiary\BestiaryDatabaseNPCsPopulator.cs')
pl = pop.split('\n')
out2=[]
for w in ids + [115,13]:
    for i,l in enumerate(pl):
        if re.search(r'\b%d\b' % w, l) and ('FindNPCEntryById' in l or 'Register' in l or '{' in l):
            out2.append(f"{i+1}: {l.strip()}")
open(os.path.join(ROOT,'tools','_d35_pop.txt'),'w',encoding='utf-8').write('\n'.join(out2))
# Hide draw offsets
out3=[]
for w in ids:
    for i,l in enumerate(lines):
        if re.search(r'\{\s*%d\s*,' % w, l):
            ctx = '\n'.join(lines[i:i+8])
            if 'Hide' in ctx or 'DrawOffset' in ctx or 'NPCBestiaryDrawOffset' in '\n'.join(lines[max(0,i-30):i]):
                out3.append(f"== {w} @ {i+1}")
                for k in range(i, min(i+8,len(lines))):
                    out3.append(f"{k+1}: {lines[k].rstrip()}")
                    if k>i and re.search(r'^\s*\}', lines[k]): break
open(os.path.join(ROOT,'tools','_d35_hide.txt'),'w',encoding='utf-8').write('\n'.join(out3))
print("debuff lines:", len(out), "pop lines:", len(out2), "hide lines:", len(out3))
