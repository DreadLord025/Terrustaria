import re, os
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
def rd(p): return open(os.path.join(ROOT, p), encoding='utf-8').read()
nid = rd(r'decompiled\Terraria\ID\NPCID.cs')
for w in range(113,120):
    m = re.search(r'public const short (\w+) = %d;' % w, nid)
    print(w, m.group(1) if m else '?')
# localization names for 115/116
import glob
loc = None
for p in [r'decompiled\Terraria\Localization\Content\NPCs.en-US.json',
          r'decompiled\Terraria\Localization\Content\NPCs.en-US.jsonl']:
    if os.path.exists(os.path.join(ROOT,p)): loc = p; break
print('loc file:', loc)
# search any localization for TheHungry
for dp,dn,fns in os.walk(os.path.join(ROOT,'decompiled','Terraria','Localization')):
    for fn in fns:
        if 'NPC' in fn and fn.endswith('.json'):
            s=open(os.path.join(dp,fn),encoding='utf-8').read()
            for key in ['TheHungryII','TheHungry"','"TheHungry']:
                i=s.find(key)
                if i>=0:
                    print(fn, key, '->', s[i:i+80].replace('\n',' '))
            break
    break
# populator: any manual 115/116 entry or credit fold
pop = rd(r'decompiled\Terraria\GameContent\Bestiary\BestiaryDatabaseNPCsPopulator.cs')
for m in re.finditer(r'.*\b(115|116)\b.*', pop):
    l=m.group(0).strip()
    if 'FindEntryByNPCID' in l or 'Register' in l or 'Credit' in l or '{ 11' in l:
        print('pop:', l[:140])
