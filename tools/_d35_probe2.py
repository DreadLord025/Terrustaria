import re, os
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
def rd(p): return open(os.path.join(ROOT, p), encoding='utf-8').read()

npc = rd(r'crates\tr-app\src\npc.rs')
data = rd(r'crates\tr-app\src\npc\data.rs')
net = rd(r'crates\tr-app\src\app\netglue.rs')

# knock_back_resist: anchor vs wildcard
i = npc.index('fn knock_back_resist')
seg = npc[i:i+30000]
a = seg.index('NpcKind::DungeonSpirit => 0.2,')
w = [m.start() for m in re.finditer(r'^\s*_\s*=>', seg, re.M)]
print('kb: anchor', a, 'wildcards', w[:3], 'OK' if not w or a < w[0] else 'BAD')
# find_frame top-level wildcard
i = npc.index('pub(crate) fn find_frame')
j = npc.index('fn update_slime')
seg = npc[i:j]
w = [m.group(0).strip() for m in re.finditer(r'^\s{8,12}_\s*=>[^\n]*', seg, re.M)]
print('find_frame wild candidates:', w[:6])

# npc_seed tail
i = net.index('fn npc_seed')
j = net.index('fn npc_from_net')
print('--- npc_seed tail ---')
print(net[j-900:j])

# cs_id BlazingWheel ctx + what follows
i = data.index('NpcKind::BlazingWheel => 72,')
print('--- cs_id ctx ---')
print(data[i-300:i+700])

# from_u16 tail around 228
i = data.index('228 => NpcKind::WyvernTail,')
print('--- from_u16 ctx ---')
print(data[i-200:i+400])
