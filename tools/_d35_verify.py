import re, os, sys
sys.path.insert(0, os.path.dirname(__file__))
from _d35_model import KINDS
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
def rd(p): return open(os.path.join(ROOT, p), encoding='utf-8').read()

# Collect all kinds landed so far (enum variants beyond the original 270)
npc = rd(r'crates\tr-app\src\npc.rs')
i = npc.index('pub enum NpcKind {'); en = npc[i:npc.index('\n}', i)]
variants = re.findall(r'^\s{4}(\w+),', en, re.M)
landed = set(k['v'] for k in KINDS) & set(variants)
print('landed kinds:', len(landed), sorted(landed))

def match_body(src, sig):
    i = src.index(sig)
    m = re.search(r'match (?:self\.kind|self|kind|\(entry\.kind, entry\.variant\)) \{', src[i:i+3000])
    start = i + m.end()
    depth=0;j=start-1
    while j<len(src):
        c=src[j]
        if c=='{':depth+=1
        elif c=='}':
            depth-=1
            if depth==0:break
        j+=1
    return src[start:j]

# exhaustive matches that must contain every landed kind
checks = [
    (r'crates\tr-app\src\npc.rs', 'fn defense('),
    (r'crates\tr-app\src\npc.rs', 'fn value('),
    (r'crates\tr-app\src\npc.rs', 'pub fn render_key(&self)'),
    (r'crates\tr-app\src\npc.rs', 'fn update_kind_ai('),
    (r'crates\tr-app\src\npc\data.rs', 'fn ai_style('),
    (r'crates\tr-app\src\npc\data.rs', 'fn size('),
    (r'crates\tr-app\src\npc\data.rs', 'fn max_hp('),
    (r'crates\tr-app\src\npc\data.rs', 'fn contact_damage('),
    (r'crates\tr-app\src\npc_buffs.rs', 'fn debuff_immune_ids('),
    (r'crates\tr-app\src\ui\bestiary.rs', 'fn portrait_data('),
    (r'crates\tr-app\src\app\netglue.rs', 'fn npc_seed('),
]
bad=0
for path, sig in checks:
    src=rd(path)
    body=match_body(src,sig)
    present=set(re.findall(r'NpcKind::(\w+)',body))
    missing=landed-present
    if missing:
        print('MISSING in',sig,':',sorted(missing)); bad+=1
    else:
        print('OK',sig)
# to_u16/from_u16/render_key collision check
data=rd(r'crates\tr-app\src\npc\data.rs')
def arms(src,start,end):
    i=src.index(start);j=src.index(end,i);seg=src[i:j];vals={}
    for m in re.finditer(r'((?:NpcKind::\w+\s*\|?\s*)+)=>\s*(\d+),',seg):
        v=int(m.group(2))
        for k in re.findall(r'NpcKind::(\w+)',m.group(1)):vals[k]=v
    return vals
tu=arms(data,'fn to_u16','fn from_u16')
i=npc.index('pub fn render_key');j=npc.index('} << 5;',i);rv={}
for m in re.finditer(r'((?:NpcKind::\w+\s*\|?\s*)+)=>\s*(\d+),',npc[i:j]):
    v=int(m.group(2))
    for k in re.findall(r'NpcKind::(\w+)',m.group(1)):rv[k]=v
# duplicates within to_u16
from collections import Counter
c=Counter(tu.values()); dups=[v for v,n in c.items() if n>1]
print('to_u16 duplicate ids:',dups)
c=Counter(rv.values()); dups=[v for v,n in c.items() if n>1]
print('render_key duplicate ids:',dups)
# landed proto ids match model
for k in KINDS:
    if k['v'] in landed:
        if tu.get(k['v'])!=k['proto']: print('PROTO MISMATCH',k['v'],tu.get(k['v']),k['proto']); bad+=1
        if rv.get(k['v'])!=k['proto']: print('RENDER MISMATCH',k['v'],rv.get(k['v']),k['proto']); bad+=1
print('structural bad =',bad)
