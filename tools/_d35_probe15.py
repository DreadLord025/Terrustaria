import re, os
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
text = open(os.path.join(ROOT, r'decompiled\Terraria\ID\NPCID.cs'), encoding='utf-8').read()
lines = text.split('\n')
want = [73,74,75,77,78,79,80,82,83,84,85,86,93,94,95,96,97,98,99,100,101,102,103,104,109,110,112,116,120,121,122,137,140]
for m in re.finditer(r'\{\s*(\d+)\s*,\s*new NPCBestiaryDrawModifiers\(0\)\s*\{([^}]*)\}', text):
    i = int(m.group(1))
    if i in want:
        ln = text[:m.start()].count('\n') + 1
        print(i, '@', ln, re.sub(r'\s+', ' ', m.group(2)).strip())
# also Hide=false explicit? default Hide false. list all ids hidden in want
print('--- hidden ids in want ---')
hid = []
for m in re.finditer(r'\{\s*(\d+)\s*,\s*new NPCBestiaryDrawModifiers\(0\)\s*\{([^}]*)\}', text):
    if 'Hide = true' in m.group(2) and int(m.group(1)) in want:
        hid.append(int(m.group(1)))
print(hid)
