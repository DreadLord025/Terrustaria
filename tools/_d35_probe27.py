import re, os
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
def rd(p): return open(os.path.join(ROOT, p), encoding='utf-8').read()
ent = rd(r'crates\tr-app\src\render\entities.rs')
i = ent.index('fn draw_npcs')
j = ent.index('NpcKind::DungeonSpirit => {', i)
k = ent.index('};', j)
seg = ent[i:k]
# top-level arms: 12-space indent `            NpcKind::`
arms = re.findall(r'^            (NpcKind::\w+|_)[^\n]*', seg, re.M)
print('outer arm count:', len(arms), 'last 3:', arms[-3:])
print('outer wildcard?', any(a.strip().startswith('_') for a in arms))
# rotation whitelist ctx
i = ent.index('| NpcKind::WyvernTail\n')
print('=== rotation ctx ==='); print(ent[i-1600:i+200])
npc = rd(r'crates\tr-app\src\npc.rs')
for pat in ['catchable', 'dontTake', 'npcSlots']:
    idxs = [m.start() for m in re.finditer(pat, npc)]
    print(pat, 'hits:', len(idxs))
    for x in idxs[:3]:
        ln = npc[:x].count('\n')+1
        print('  ', ln, npc.split('\n')[ln-1].strip()[:120])
