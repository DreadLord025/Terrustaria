import re, os, json
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d36"
raw = open(os.path.join(ROOT, r'decompiled\Terraria.Localization.Content.en-US.NPCs.json'), encoding='utf-8-sig').read()
raw = re.sub(r',\s*([}\]])', r'\1', raw)
loc = json.loads(raw)
key = 'NPCName' if 'NPCName' in loc else list(loc.keys())[0]
names = loc[key]
npcid = open(os.path.join(ROOT, r'decompiled\Terraria\ID\NPCID.cs'), encoding='utf-8').read().split('\n')
mine = json.load(open(r'C:\Users\DreadLord\AppData\Local\Temp\opencode\d36_mine.json'))
want = {t: mine['names'][str(t)] for t in mine['mine']}
out = {}
for i, n in sorted(want.items()):
    ln = next((j+1 for j, l in enumerate(npcid) if re.search(r'public const short %s = %d;' % (n, i), l)), None)
    out[i] = {'name': n, 'npcid_line': ln, 'display': names.get(n)}
    print(i, n, ln, repr(names.get(n)))
json.dump(out, open(os.path.join(ROOT, 'tools', '_d36_names.json'), 'w'), indent=0)
