import re, os, sys
sys.path.insert(0, os.path.dirname(__file__))
from _d35_model import KINDS
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
def rd(p): return open(os.path.join(ROOT, p), encoding='utf-8').read()
npc=rd(r'crates\tr-app\src\npc.rs'); data=rd(r'crates\tr-app\src\npc\data.rs')
b=rd(r'crates\tr-app\src\bestiary.rs'); ent=rd(r'crates\tr-app\src\render\entities.rs')
buffs=rd(r'crates\tr-app\src\npc_buffs.rs'); net=rd(r'crates\tr-app\src\app\netglue.rs')
uib=rd(r'crates\tr-app\src\ui\bestiary.rs'); aud=rd(r'crates\tr-app\src\audio.rs')
bad=0
def chk(cond,msg):
    global bad
    if not cond: print('FAIL:',msg); bad+=1

# 1. to_u16/from_u16 bijection over all 303
i=data.index('fn to_u16');j=data.index('fn from_u16');seg=data[i:j]
tu={}
for m in re.finditer(r'((?:NpcKind::\w+\s*\|?\s*)+)=>\s*(\d+),',seg):
    for k in re.findall(r'NpcKind::(\w+)',m.group(1)):tu[k]=int(m.group(2))
i=data.index('fn from_u16');j=data.index('fn ai_style');seg=data[i:j]
fu={}
for m in re.finditer(r'^\s*(\d+)\s*=>\s*NpcKind::(\w+),',seg,re.M):fu[int(m.group(1))]=m.group(2)
chk(len(tu)==303,f'to_u16 count {len(tu)}')
chk(len(set(tu.values()))==303,'to_u16 injective')
for k,v in tu.items(): chk(fu.get(v)==k,f'roundtrip {k} {v}')
# 2. every kind's proto matches model
for k in KINDS: chk(tu.get(k['v'])==k['proto'],f"proto {k['v']}")
# 3. render_key = to_u16 for our kinds
i=npc.index('pub fn render_key');j=npc.index('} << 5;',i);rv={}
for m in re.finditer(r'((?:NpcKind::\w+\s*\|?\s*)+)=>\s*(\d+),',npc[i:j]):
    for k in re.findall(r'NpcKind::(\w+)',m.group(1)):rv[k]=int(m.group(2))
for k in KINDS: chk(rv.get(k['v'])==k['proto'],f"render_key {k['v']}")
# 4. bestiary: visible own entry, hidden fold, credit uniqueness
i=b.index('pub const ENTRIES');j=b.index('\n];',i);eseg=b[i:j]
credits=re.findall(r'credit_id: "([^"]+)"',eseg)
chk(len(credits)==len(set(credits)),'credit ids unique')
for k in KINDS:
    if k['hide']:
        chk(f'kind: NpcKind::{k["v"]},' not in eseg,f"{k['v']} no own entry")
        chk(f'NpcKind::{k["v"]} => NpcKind::{k["hide"]},' in b,f"{k['v']} fold")
    else:
        chk(f'kind: NpcKind::{k["v"]},' in eseg,f"{k['v']} own entry")
# 5. cs_id: visible hand row, hidden none
i=data.index('fn cs_id');j=data.index('_ => {',i);cseg=data[i:j]
for k in KINDS:
    if k['hide']: chk(f'NpcKind::{k["v"]} =>' not in cseg,f"{k['v']} no cs_id hand row")
    else: chk(f'NpcKind::{k["v"]} => {k["id"]},' in cseg,f"{k['v']} cs_id hand row")
# 6. texture_net_id: hidden explicit vanilla
i=ent.index('fn texture_net_id');j=ent.index('\n}',i);tseg=ent[i:j]
for k in KINDS:
    if k['hide']:
        m=re.search(r'NpcKind::'+k['v']+r' => (\d+)',tseg)
        chk(m and m.group(1)==str(k['id']),f"{k['v']} texture_net_id")
# 7. update_kind_ai single arm per kind (d4c)
i=npc.index('fn update_kind_ai');rest=npc[i:];end=rest.index('\n    }');body=rest[:end]
from collections import Counter
counts=Counter()
for line in body.split('\n'):
    t=line.strip()
    if t.startswith('NpcKind::') or t.startswith('| NpcKind::'):
        for nm in re.findall(r'NpcKind::(\w+)',t):counts[nm]+=1
chk(all(n==1 for n in counts.values()),'update_kind_ai single-arm')
for k in KINDS: chk(counts.get(k['v'])==1,f"{k['v']} one update arm")
# 8. axis guards present
chk('5 if !matches!(self.kind, NpcKind::Corruptor)' in npc,'axis-5 guard')
chk('9 if !matches!(self.kind, NpcKind::VileSpit)' in npc,'axis-9 guard')
chk('NpcKind::Corruptor => {}' in npc,'hover Corruptor no-op')
# 9. style-5 pin has Corruptor
i=npc.index('let five: Vec<_>');j=npc.index('];',i)
chk('NpcKind::Corruptor' in npc[i:j],'style-5 pin Corruptor')
# 10. audio bank rows for all our cues
for k in KINDS:
    if k['hit'] is not None: chk(f'("NPC_Hit_{k["hit"]}",' in aud,f"bank NPC_Hit_{k['hit']}")
    if k['death'] is not None: chk(f'("NPC_Killed_{k["death"]}",' in aud,f"bank NPC_Killed_{k['death']}")
# 11. witness modules A-G
for L in 'abcdefg': chk(f'mod w4_d35_{L}_tests' in npc,f'witness {L}')
# 12. no_gravity: worms false, flyers true
i=data.index('fn no_gravity');j=data.index('_ => false',i);ngseg=data[i:j]
for k in KINDS:
    expect = k['noGrav'] and (k['kb'] is None or k['kb']>0)
    present = f'NpcKind::{k["v"]}' in ngseg
    chk(present==expect,f"{k['v']} no_gravity {expect}")
print('FINAL comprehensive bad =',bad)
