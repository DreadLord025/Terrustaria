import re, os
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
def rd(p): return open(os.path.join(ROOT, p), encoding='utf-8').read()
b = rd(r'crates\tr-app\src\bestiary.rs')
for fn in ['fn try_entry_for_npc', 'fn entry_for_npc']:
    i = b.index(fn); j = b.index('\n}', i)
    print('======', fn, 'lines', b[:i].count('\n')+1)
    print(b[max(0,i-1400):j+2])
