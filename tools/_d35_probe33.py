import re, os, sys
sys.path.insert(0, os.path.dirname(__file__))
from _d35_model import KINDS
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
def rd(p): return open(os.path.join(ROOT, p), encoding='utf-8').read()
cs = rd(r'decompiled\Terraria\ID\ContentSamples.cs')
i = cs.index('void ModifyNPCIds'); seg = cs[i:i+40000]
for w in [95,96,97,98,99,100,112,115,116]:
    hits = re.findall(r'npcBestiaryCreditIdsByNpcNetIds\[%d\]\s*=\s*([^;]+);' % w, seg)
    print('credit remap', w, hits)
nid = rd(r'decompiled\Terraria\ID\NPCID.cs')
for nm in ['TheHungry','TheHungryII','VileSpit','Corruptor','DiggerHead','SeekerHead']:
    m = re.search(r'public const short %s = (\d+);' % nm, nid)
    print(nm, '=', m.group(1) if m else '?')
# 116 Hide? already confirmed not hidden. Confirm 112 hide + its natural credit name
print('--- 112 / 116 draw modifiers (probe15 already: 112 Hide, 116 Position) ---')
# Verify model stats vs C# SetDefaults for a sample: parse NPC.cs case blocks
npc_cs = rd(r'decompiled\Terraria\NPC.cs')
def case_block(cid):
    # find `case <cid>:` within SetDefaults region; grab until next case/break
    m = re.search(r'\n\t\t\tcase %d:\n' % cid, npc_cs)
    if not m: return None
    start = m.end()
    nxt = re.search(r'\n\t\t\tcase \d+:|\n\t\t\tdefault:', npc_cs[start:])
    end = start + nxt.start() if nxt else start+2000
    return npc_cs[start:end]
for k in KINDS[:3] + [x for x in KINDS if x['id'] in (82,95,112,116,121)]:
    blk = case_block(k['id'])
    if blk is None:
        print(k['id'], 'NO CASE'); continue
    def g(pat):
        m = re.search(pat, blk); return m.group(1) if m else None
    w = g(r'width = (\d+)'); h = g(r'height = (\d+)')
    dmg = g(r'damage = (\d+)'); dfn = g(r'defense = (\d+)'); life = g(r'lifeMax = (\d+)')
    kb = g(r'knockBackResist = ([\d.]+)f?'); val = g(r'value = ([\d.]+)f?')
    ai = g(r'aiStyle = (\d+)'); sc = g(r'scale = ([\d.]+)f?'); al = g(r'alpha = (\d+)')
    print(k['id'], k['v'], 'C# w/h',w,h,'dmg',dmg,'def',dfn,'life',life,'kb',kb,'val',val,'ai',ai,'scale',sc,'alpha',al)
    print('   model w/h',k['w'],k['h'],'dmg',k['dmg'],'def',k['dfn'],'life',k['life'],'kb',k['kb'],'val',k['value'],'ai',k['ai'],'scale',k['scale'],'alpha',k['alpha'])
