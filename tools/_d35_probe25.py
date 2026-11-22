import re, os
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
def rd(p): return open(os.path.join(ROOT, p), encoding='utf-8').read()
npc = rd(r'crates\tr-app\src\npc.rs')
# all loops over ALL_KINDS (any form)
for m in re.finditer(r'ALL_KINDS', npc):
    ln = npc[:m.start()].count('\n')+1
    line = npc.split('\n')[ln-1].strip()
    print(ln, line[:130])
