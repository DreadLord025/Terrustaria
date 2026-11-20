import re, os
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
def rd(p): return open(os.path.join(ROOT, p), encoding='utf-8').read()
npc = rd(r'crates\tr-app\src\npc.rs')
i = npc.index('fn update_kind_ai(')
print('--- update_kind_ai head ---')
print(npc[i:i+2600])
# who calls update_kind_ai
for m in re.finditer(r'update_kind_ai\(', npc):
    ln = npc[:m.start()].count('\n')+1
    print('call at', ln, npc.split('\n')[ln-1].strip()[:120])
