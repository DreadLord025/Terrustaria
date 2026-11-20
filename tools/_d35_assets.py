import re, os, sys
sys.path.insert(0, os.path.dirname(__file__))
from _d35_model import KINDS
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
# audio assets
cues=set()
for k in KINDS:
    if k['hit'] is not None: cues.add(f'NPC_Hit_{k["hit"]}')
    if k['death'] is not None: cues.add(f'NPC_Killed_{k["death"]}')
missing=[]
for c in sorted(cues):
    p=os.path.join(ROOT,'assets','Content','Sounds',c+'.xnb')
    if not os.path.exists(p): missing.append(c)
print('audio cues:',len(cues),'missing assets:',missing)
# bestiary ENTRIES count + grid capacity
b=open(os.path.join(ROOT,r'crates\tr-app\src\bestiary.rs'),encoding='utf-8').read()
i=b.index('pub const ENTRIES');j=b.index('\n];',i)
n=b[i:j].count('BestiaryEntryDef {')
print('ENTRIES count:',n)
# grid: panel 900 wide / CELL 72 = 12 cols; rows grow with need
cols=int(900/72); need=(n+cols-1)//cols
print('grid cols:',cols,'rows needed:',need,'capacity:',cols*need,'>=',n, cols*need>=n)
# soundid styles for our cues exist
sid=open(os.path.join(ROOT,r'crates\tr-app\src\soundid.rs'),encoding='utf-8').read()
for k in KINDS:
    if k['hit'] is not None:
        ok=re.search(r'sound_id: 3, style: %d,'%k['hit'],sid)
        if not ok: print('MISSING style id3',k['hit'])
    if k['death'] is not None:
        ok=re.search(r'sound_id: 4, style: %d,'%k['death'],sid)
        if not ok: print('MISSING style id4',k['death'])
print('soundid style check done')
