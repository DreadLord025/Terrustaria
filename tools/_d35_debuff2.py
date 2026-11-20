import re, os, json
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
lines = open(os.path.join(ROOT,r'decompiled\Terraria\ID\NPCID.cs'),encoding='utf-8').read().split('\n')
# locate DebuffImmunitySets dictionary start
start = next(i for i,l in enumerate(lines) if 'DebuffImmunitySets' in l and '=' in l and 'new' in l)
print('dict starts', start+1, lines[start].strip()[:80])
# parse entries: { <id>, null } or { <id>, new NPCDebuffImmunityData { ... } }
text = '\n'.join(lines[start:start+3200])
entries = {}
for m in re.finditer(r'\{\s*(\d+)\s*,\s*(null|new NPCDebuffImmunityData\s*\{[^{}]*(?:\{[^{}]*\}[^{}]*)*\})\s*\}', text):
    i = int(m.group(1)); body = m.group(2)
    ln = start + text[:m.start()].count('\n') + 1
    if body == 'null':
        entries[i] = {'line': ln, 'null': True}
    else:
        spec = re.search(r'SpecificallyImmuneTo = new int\[\d+\]\s*\{([^}]*)\}', body)
        allnw = 'ImmuneToAllBuffsThatAreNotWhips = true' in body
        whips = 'ImmuneToWhips = true' in body
        entries[i] = {'line': ln, 'spec': [int(x) for x in spec.group(1).split(',')] if spec else None, 'all_nonwhip': allnw, 'whip_immune': whips}
want = [73,74,75,77,78,79,80,82,83,84,85,86,93,94,95,96,97,98,99,100,101,102,103,104,109,110,112,116,120,121,122,137,140]
for w in want:
    print(w, entries.get(w, 'NO ROW'))
json.dump(entries, open(os.path.join(ROOT,'tools','_d35_debuff.json'),'w'), indent=0)
