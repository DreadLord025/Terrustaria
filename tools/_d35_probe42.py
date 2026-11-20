import re, os
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
def rd(p): return open(os.path.join(ROOT, p), encoding='utf-8').read()
npc_cs = rd(r'decompiled\Terraria\NPC.cs')
# rotation writes in AI_023 (cursed hammer/enchanted sword) and AI_013 (clinger) and AI_006 worms
for fn in ['AI_023', 'AI_013', 'AI_006_Worms']:
    i = npc_cs.find('private void ' + fn)
    if i < 0:
        i = npc_cs.find(fn + '(')
    if i < 0:
        print(fn, 'NOT FOUND'); continue
    seg = npc_cs[i:i+9000]
    rots = re.findall(r'this\.rotation\s*=[^;]+;', seg)
    print(fn, 'rotation writes:', rots[:4])
# shares_carrier_life head inclusion
data = rd(r'crates\tr-app\src\npc\data.rs')
i = data.index('fn shares_carrier_life'); j = data.index('\n    }', i)
seg = data[i:j]
for k in ['WyvernHead','DevourerHead','GiantWormHead','LeechHead','TheDestroyer']:
    print('shares_carrier_life has', k, ':', ('NpcKind::'+k) in seg)
print('shares_carrier_life anchor | WyvernTail count:', data.count('                | NpcKind::WyvernTail\n'))
# texture_net_id ForceBubble anchor uniqueness
ent = rd(r'crates\tr-app\src\render\entities.rs')
print('texture_net_id ForceBubble anchor count:', ent.count('        NpcKind::ForceBubble => 384,\n        _ => kind.net_id() as u16,'))
# no_gravity bat citation line
i = npc_cs.find('aiStyle == 14')
print('aiStyle14 block near line:', npc_cs[:i].count('\n')+1 if i>=0 else '?')
# confirm the 5-axis arm and 9-axis arm exact text uniqueness (already 1 each)
npc = rd(r'crates\tr-app\src\npc.rs')
print('hover unreachable anchor count:', npc.count('            NpcKind::Probe => {}\n            _ => unreachable!("only aiStyle-5 kinds reach the family arm"),'))
# bestiary fold anchor count (both fns)
b = rd(r'crates\tr-app\src\bestiary.rs')
print('wyvern fold anchor count:', b.count('        | NpcKind::WyvernTail => NpcKind::WyvernHead,\n'))
