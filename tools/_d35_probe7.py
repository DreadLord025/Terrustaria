import os, re
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
def rd(p): return open(os.path.join(ROOT, p), encoding='utf-8').read()
npc = rd(r'crates\tr-app\src\npc.rs')
lines = npc.split('\n')
for ln in [19037, 31740, 23964]:
    print('=== npc.rs around', ln)
    print('\n'.join(lines[ln-30:ln+40]))
    print()
