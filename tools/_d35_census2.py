import re, json, os
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
def rd(p): return open(os.path.join(ROOT,p),encoding='utf-8').read()
npcrs = rd(r'crates\tr-app\src\npc.rs')
en = npcrs[npcrs.index('pub enum NpcKind {'):]
en = en[:en.index('\n}')]
# walk lines, accumulate doc comments then variant
variant_ids = {}
doc = []
for line in en.split('\n'):
    s = line.strip()
    if s.startswith('///'):
        doc.append(s)
    elif re.match(r'^\w+,$', s):
        v = s[:-1]
        text = ' '.join(doc)
        ids = set()
        # NPCID.Name (123)  or  net 123  or `NPCID.X` (123)
        for m in re.finditer(r'NPCID\.\w+`?\s*\((\d+)\)', text): ids.add(int(m.group(1)))
        for m in re.finditer(r'\bnet (\d+)\b', text): ids.add(int(m.group(1)))
        variant_ids[v] = {'doc_ids': sorted(ids), 'doc': text[:200]}
        doc = []
    elif s and not s.startswith('//'):
        doc = []
census = json.load(open(os.path.join(ROOT,'tools','_d35_census.json')))
id2name = census['id2name']
covered = {int(k):v for k,v in census['covered'].items()}
# add doc-id coverage
for v,info in variant_ids.items():
    for i in info['doc_ids']:
        if 71<=i<=140: covered.setdefault(i,[]).append(v+'(doc)')
missing = [i for i in range(71,141) if i not in covered]
print("MISSING (no variant maps to id):")
for i in missing: print(f"  {i}: {'/'.join(id2name.get(str(i),['?']))}")
print("count:", len(missing))
json.dump({'missing':missing,'variant_ids':variant_ids}, open(os.path.join(ROOT,'tools','_d35_census2.json'),'w'), indent=0)
