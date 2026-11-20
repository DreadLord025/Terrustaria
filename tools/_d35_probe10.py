import os, re, sys
sys.path.insert(0, os.path.join(os.path.dirname(__file__)))
from _d35_model import KINDS
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
def rd(p): return open(os.path.join(ROOT, p), encoding='utf-8').read()
npc = rd(r'crates\tr-app\src\npc.rs')
ent = rd(r'crates\tr-app\src\render\entities.rs')
data = rd(r'crates\tr-app\src\npc\data.rs')
aud = rd(r'crates\tr-app\src\audio.rs')
buffs = rd(r'crates\tr-app\src\npc_buffs.rs')

# render_key arm values
i = npc.index('pub fn render_key')
j = npc.index('} << 5;', i)
seg = npc[i:j]
vals = {}
for m in re.finditer(r'((?:NpcKind::\w+\s*\|?\s*)+)=>\s*(\d+),', seg):
    v = int(m.group(2))
    for k in re.findall(r'NpcKind::(\w+)', m.group(1)):
        vals[k] = v
wild = re.findall(r'^\s*_[^\n]*', seg, re.M)
print('render_key arms:', len(vals), 'wildcards:', wild)
band = set(range(251, 284))
clash = {k: v for k, v in vals.items() if v in band}
print('render_key value clashes with 251..283:', clash)

# NPC_FRAME_COUNT rows for our ids
i = ent.index('NPC_FRAME_COUNT')
m = re.search(r'NPC_FRAME_COUNT[^=]*=\s*\[', ent)
j = ent.index('];', m.end())
arr = ent[m.end():j]
nums = [int(x) for x in re.findall(r'\d+', arr)]
print('NPC_FRAME_COUNT len', len(nums))
for k in KINDS:
    got = nums[k['id']] if k['id'] < len(nums) else None
    flag = '' if got == k['rows'] else '  <-- MISMATCH model rows=%s' % k['rows']
    print(k['id'], k['v'], 'framecount', got, flag)

# audio npc_sound_style
i = aud.index('fn npc_sound_style')
print(aud[i-400:i+900])

# npc_buffs fn name at 158
i = buffs.index('match kind')
print('--- buffs fn head ---')
print(buffs[max(0,i-900):i+50])

# CS_STYLES
i = npc.index('CS_STYLES')
print('--- CS_STYLES head ---')
print(npc[i-600:i+200])
