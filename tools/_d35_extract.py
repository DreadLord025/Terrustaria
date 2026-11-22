import re, os, json
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
lines = open(os.path.join(ROOT,r'decompiled\Terraria\NPC.cs'),encoding='utf-8').read().split('\n')
LO, HI = 8425, 17858  # SetDefaults body, 0-based inclusive
want = [73,74,75,76,77,78,79,80,82,83,84,85,86,93,94,95,96,97,98,99,100,101,102,103,104,109,110,112,116,120,121,122,137,140]
out = []
found = {}
for w in want:
    hits = []
    pat = re.compile(r'\bthis\.type == %d\b' % w)
    for i in range(LO, HI):
        l = lines[i]
        if re.search(r'(else\s+)?if\s*\(', l) and pat.search(l):
            hits.append(i)
    found[w] = [h+1 for h in hits]
    out.append(f"===== id {w} : arms at {[h+1 for h in hits]} =====")
    for h in hits:
        # capture from the if line to matching close brace
        depth = 0; started = False; j = h
        while j < HI+400:
            depth += lines[j].count('{') - lines[j].count('}')
            if '{' in lines[j]: started = True
            if started and depth <= 0: break
            j += 1
        for k in range(h, min(j+1, HI+400)):
            out.append(f"{k+1}: {lines[k].rstrip()}")
        out.append("")
open(os.path.join(ROOT,'tools','_d35_cases.txt'),'w',encoding='utf-8').write('\n'.join(out))
print(json.dumps(found))
