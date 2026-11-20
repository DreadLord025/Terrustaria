import os, re
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
def rd(p): return open(os.path.join(ROOT, p), encoding='utf-8').read()
npc = rd(r'crates\tr-app\src\npc.rs')

# find_frame: top-level (12-space) wildcard arms
i = npc.index('pub(crate) fn find_frame')
j = npc.index('fn update_slime')
seg = npc[i:j]
for m in re.finditer(r'^ {12}_[^\n]*$', seg, re.M):
    print('find_frame 12sp wildcard:', m.group(0))
for m in re.finditer(r'^ {12}\w+ =>[^\n]*$', seg, re.M):
    t = m.group(0).strip()
    if not t.startswith('NpcKind'):
        print('find_frame binding arm:', t)
# tail of match: last arms
print('--- last 60 lines of find_frame ---')
print('\n'.join(seg.split('\n')[-60:])[:3000])

# update_kind_ai wildcard
i = npc.index('fn update_kind_ai')
# find end: next '    fn ' at same indent
j = npc.index('\n    fn ', i+10)
seg = npc[i:j]
for m in re.finditer(r'^ {12}_[^\n]*$', seg, re.M):
    print('update_kind_ai wildcard:', m.group(0))
i2 = seg.index('NpcKind::Sharkron | NpcKind::Sharkron2 => self.ai_071_sharkron(world, player),')
print('--- sharkron ctx ---')
print(seg[i2-300:i2+700])
