import re, os
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
lines = open(os.path.join(ROOT, r'decompiled\Terraria\ID\NPCID.cs'), encoding='utf-8').read().split('\n')
for w in [95,96,97,98,99,100,112,115,116]:
    for i, l in enumerate(lines):
        if re.search(r'\{\s*%d\s*,' % w, l):
            ctx = '\n'.join(lines[max(0,i-3):i+8])
            print('== %d @ %d' % (w, i+1))
            print('\n'.join(lines[max(0,i-3):i+8]))
            print()
