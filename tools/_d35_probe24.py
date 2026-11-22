import re, os
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
def rd(p): return open(os.path.join(ROOT, p), encoding='utf-8').read()
net = rd(r'crates\tr-app\src\app\netglue.rs')
i = net.index('fn npc_from_net')
print('--- npc_from_net head ---')
print(net[i:i+2200])
# how save writes npc kind
sg = rd(r'crates\tr-app\src\app\saveglue.rs')
for m in re.finditer(r'(cs_id|to_u16|net_id)\(\)', sg):
    ln = sg[:m.start()].count('\n')+1
    print('saveglue', ln, sg[max(0,m.start()-120):m.end()+60].replace('\n',' ')[-200:])
# npc.rs 18711 ctx
npc = rd(r'crates\tr-app\src\npc.rs')
lines = npc.split('\n')
print('--- npc.rs 18700-18725 ---')
print('\n'.join(lines[18699:18725]))
