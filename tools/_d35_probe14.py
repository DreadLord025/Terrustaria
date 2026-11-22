import re, os
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
lines = open(os.path.join(ROOT, r'decompiled\Terraria\ID\NPCID.cs'), encoding='utf-8').read().split('\n')
want = [73,74,75,77,78,79,80,82,83,84,85,86,93,94,95,96,97,98,99,100,101,102,103,104,109,110,112,116,120,121,122,137,140]
# find the DrawOffset dictionary region(s): NPCBestiaryDrawOffsetCreation + GetRedigitEntries etc
regions = [i for i, l in enumerate(lines) if 'NPCBestiaryDrawModifiers' in l and ('Dictionary' in l or 'Entries()' in l)]
print('regions at', [r+1 for r in regions])
for w in want:
    hits = []
    for i, l in enumerate(lines):
        if re.search(r'\{\s*%d\s*,' % w, l) and i > 390:
            block = '\n'.join(lines[i:i+12])
            if 'NPCBestiaryDrawModifiers' in block or 'Hide' in block or 'Scale' in block or 'Position' in block or 'Velocity' in block or 'PortraitScale' in block:
                hits.append((i+1, block.split('},')[0] + '}'))
    if hits:
        print('== %d ==' % w)
        for ln, blk in hits:
            print(ln, re.sub(r'\s+', ' ', blk)[:220])
