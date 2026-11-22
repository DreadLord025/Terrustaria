import re, os
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
def rd(p): return open(os.path.join(ROOT, p), encoding='utf-8').read()
npc = rd(r'crates\tr-app\src\npc.rs')

def fn_match(src, sig):
    i = src.index(sig)
    # find the kind match (match self.kind)
    m = re.search(r'match self\.kind \{', src[i:i+3000])
    start = i + m.end()
    depth=0; j=start-1
    while j < len(src):
        c=src[j]
        if c=='{':depth+=1
        elif c=='}':
            depth-=1
            if depth==0:break
        j+=1
    return src[start:j]

for sig, anchor in [('fn defense(', 'NpcKind::DungeonSpirit => 30,'),
                    ('fn value(', 'NpcKind::DungeonSpirit => 500.0,'),
                    ('fn knock_back_resist(', 'NpcKind::DungeonSpirit => 0.2,')]:
    body = fn_match(npc, sig)
    a = body.find(anchor)
    wilds = [m.start() for m in re.finditer(r'^\s*_\s*=>', body, re.M)]
    print(sig, 'anchor at', a, 'wildcards at', wilds, 'OK' if (not wilds or a < wilds[0]) else 'BAD-anchor-after-wildcard')
    # is it exhaustive (no wildcard)?
    print('   exhaustive:', not wilds, 'arms:', body.count('NpcKind::'))

# data.rs size/max_hp/contact_damage/ai_style: exhaustive or wildcard, anchor position
data = rd(r'crates\tr-app\src\npc\data.rs')
for sig, anchor in [('fn ai_style(', 'NpcKind::DungeonSpirit => 56,'),
                    ('fn size(', 'NpcKind::DungeonSpirit => (20.0, 20.0),'),
                    ('fn max_hp(', 'NpcKind::DungeonSpirit => 200,'),
                    ('fn contact_damage(', 'NpcKind::DungeonSpirit => 70,')]:
    i = data.index(sig)
    m = re.search(r'match self \{|match self\.kind \{', data[i:i+2000])
    start = i + m.end()
    depth=0;j=start-1
    while j<len(data):
        c=data[j]
        if c=='{':depth+=1
        elif c=='}':
            depth-=1
            if depth==0:break
        j+=1
    body=data[start:j]
    a=body.find(anchor)
    wilds=[mm.start() for mm in re.finditer(r'^\s*_\s*=>',body,re.M)]
    print(sig,'anchor at',a,'wildcards',wilds,'OK' if (not wilds or a<wilds[0]) else 'BAD','exhaustive:',not wilds)
