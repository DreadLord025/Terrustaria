import re, os
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
def rd(p): return open(os.path.join(ROOT, p), encoding='utf-8').read()
data = rd(r'crates\tr-app\src\npc\data.rs')
npc = rd(r'crates\tr-app\src\npc.rs')
ent = rd(r'crates\tr-app\src\render\entities.rs')

# to_u16 used ids
i = data.index('pub fn to_u16')
j = data.index('pub fn from_u16')
seg = data[i:j]
used = set(int(m.group(1)) for m in re.finditer(r'=>\s*(\d+),', seg))
band = set(range(237, 270))
print('to_u16 collisions in 237..269:', sorted(used & band))
print('max used:', max(used))
# from_u16 decode ids
k = data.index('_ => return None', j)
seg2 = data[j:k]
dec = set(int(m.group(1)) for m in re.finditer(r'^\s*(\d+)\s*=>', seg2, re.M))
print('from_u16 collisions:', sorted(dec & band))

# ctor scale/alpha context
i = npc.index('.unwrap_or(if kind == NpcKind::BlazingWheel { 1.2 } else { 1.0 }),')
print('--- ctor scale ctx ---')
print(npc[i-1200:i+200])
i = npc.index('NpcKind::DetonatingBubble | NpcKind::Sharkron | NpcKind::Sharkron2 => {')
print('--- ctor alpha ctx ---')
print(npc[i-800:i+400])

# humanoid_draw + generic_anchor signatures
i = ent.index('fn humanoid_draw')
print(ent[i:i+400])
i = ent.index('fn generic_anchor<')
print(ent[i:i+400])
# the draw match context around DungeonSpirit arm
i = ent.index('NpcKind::DungeonSpirit => {')
print('--- draw ctx ---')
print(ent[i-1200:i+300])
