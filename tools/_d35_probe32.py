import re, os
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
def rd(p): return open(os.path.join(ROOT, p), encoding='utf-8').read()
npc = rd(r'crates\tr-app\src\npc.rs')

# 1. find_frame outer match subject
i = npc.index('pub(crate) fn find_frame(')
seg = npc[i:i+400]
print('--- find_frame head ---'); print(seg)

# 2. CS_STYLES other roster pins (filter by style)
for m in re.finditer(r'CS_STYLES\s*\.\s*iter\(\)\s*\.\s*filter\(\|&&\(_, s\)\| s == (\d+)\)', npc):
    ln = npc[:m.start()].count('\n')+1
    print('CS_STYLES style-filter pin at', ln, 'style', m.group(1))

# 3. entry_sheet + npc_sheet
uib = rd(r'crates\tr-app\src\ui\bestiary.rs')
i = uib.index('fn entry_sheet'); print('--- entry_sheet ---'); print(uib[i:i+500])
ast = rd(r'crates\tr-app\src\assets.rs')
m = re.search(r'fn npc_sheet\b[^\n]*', ast)
if m:
    i = m.start(); print('--- npc_sheet ---'); print(ast[i:i+300])
else:
    # find where npc_sheet is defined
    for dp, dn, fns in os.walk(os.path.join(ROOT, 'crates')):
        dn[:] = [d for d in dn if d not in ('target', '.git')]
        for fn in fns:
            if fn.endswith('.rs'):
                s = open(os.path.join(dp, fn), encoding='utf-8').read()
                mm = re.search(r'fn npc_sheet\b', s)
                if mm:
                    print('npc_sheet in', os.path.relpath(os.path.join(dp, fn), ROOT), s[mm.start():mm.start()+260])

# 4. boss_draw match subject
ent = rd(r'crates\tr-app\src\render\entities.rs')
i = ent.index('fn boss_draw'); print('--- boss_draw head ---'); print(ent[i:i+400])

# 5. try_entry_for_npc ForceBubble present?
b = rd(r'crates\tr-app\src\bestiary.rs')
i = b.index('fn try_entry_for_npc'); j = b.index('\n}', i)
print('try_entry has ForceBubble fold:', 'ForceBubble' in b[i:j])
print('try_entry has SleepingAngler fold:', 'SleepingAngler' in b[i:j])

# 6. C# 116 credit + entry
cs = rd(r'decompiled\Terraria\ID\ContentSamples.cs')
i = cs.index('void ModifyNPCIds'); seg = cs[i:i+30000]
for w in [115,116]:
    hits = re.findall(r'npcBestiaryCreditIdsByNpcNetIds\[%d\]\s*=\s*([^;]+);' % w, seg)
    print('C# credit remap', w, hits)
# NPCID name for 116
nid = rd(r'decompiled\Terraria\ID\NPCID.cs')
m = re.search(r'public const short TheHungryII = (\d+);', nid)
print('TheHungryII id:', m.group(1) if m else '?')
m = re.search(r'public const short Hungry = (\d+);', nid)
print('Hungry id:', m.group(1) if m else '?')
