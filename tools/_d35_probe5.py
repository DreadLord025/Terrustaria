import os, re
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
def rd(p): return open(os.path.join(ROOT, p), encoding='utf-8').read()

assets = rd(r'crates\tr-app\src\assets.rs')
i = assets.index('pub const BESTIARY_NPC_SHEET_IDS')
j = assets.index('];', i)
seg = assets[i:j]
ids = [int(x) for x in re.findall(r'\d+', seg.split('=',1)[1])]
print('bestiary sheet count', len(ids))
for w in [87,88,89,90,91,92,72,250,288,384,73,74,75,95,98,116]:
    print(w, 'in BESTIARY_NPC_SHEET_IDS:', w in ids)

# how bestiary_npc_sheets vec is built (deferred None precedent)
i = assets.index('bestiary_npc_sheets: vec![')
print(assets[i:i+1200])
