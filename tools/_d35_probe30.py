import re, os
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
def rd(p): return open(os.path.join(ROOT, p), encoding='utf-8').read()
npc = rd(r'crates\tr-app\src\npc.rs')
for fn in ['fn ai_005_hover_family', 'fn ai_009_sphere_family']:
    i = npc.index(fn)
    print('======', fn, 'line', npc[:i].count('\n')+1)
    print(npc[i:i+1800])
# CS_STYLES test body
i = npc.index('const CS_STYLES')
j = npc.index('\n    ];', i)
print('CS_STYLES rows:', npc[i:j].count('(NpcKind::'))
k = npc.index('CS_STYLES', j)
print('--- CS_STYLES consumer ---')
print(npc[k-600:k+700])
