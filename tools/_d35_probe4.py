import os, re
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
def rd(p): return open(os.path.join(ROOT, p), encoding='utf-8').read()
npc = rd(r'crates\tr-app\src\npc.rs')
i = npc.index('NpcKind::WyvernTail => 228,')
print('render_key ctx line', npc[:i].count('\n')+1)
print(npc[i-700:i+400])
aud = rd(r'crates\tr-app\src\audio.rs')
i = aud.index('("NPC_Killed_33", 4),')
print('--- audio ctx ---')
print(aud[i-1400:i+300])
# check hit bank rows already present
for h in [1,2,4,5,6,9,12,25,54]:
    print('NPC_Hit_%d present:'%h, ('("NPC_Hit_%d",' % h) in aud)
for d in [1,2,4,6,7,9,12,18,28,52]:
    print('NPC_Killed_%d present:'%d, ('("NPC_Killed_%d",' % d) in aud)
