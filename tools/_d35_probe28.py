import re, os
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
def rd(p): return open(os.path.join(ROOT, p), encoding='utf-8').read()
npc = rd(r'crates\tr-app\src\npc.rs')
i = npc.index('pub fn update(')
print('--- update head ---')
print(npc[i:i+3000])
