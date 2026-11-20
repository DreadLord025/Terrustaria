import re, os
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
def rd(p): return open(os.path.join(ROOT, p), encoding='utf-8').read()
data = rd(r'crates\tr-app\src\npc\data.rs')
npc = rd(r'crates\tr-app\src\npc.rs')
i = data.index('fn no_gravity')
print('--- no_gravity head ---')
print(data[i-900:i+120])
seg = data[i:data.index('_ => false', i)]
print('worm rows in no_gravity:', re.findall(r'NpcKind::(\w*(?:Worm|Devourer|Destroyer|Leech|Wyvern|Digger|Seeker)\w*)', seg))
# enum bare DungeonSpirit count
print('bare enum DungeonSpirit count:', npc.count('    DungeonSpirit,\n'))
# tests iterating ALL_KINDS with entry lookups
for m in re.finditer(r'for (?:&?kind|\(kind[^)]*\)) in (?:ALL_KINDS|super::ALL_KINDS|crate::npc::ALL_KINDS)', npc):
    ln = npc[:m.start()].count('\n')+1
    print('ALL_KINDS loop at', ln, npc[m.start():m.start()+160].replace('\n',' ')[:160])
# behind_tiles fn head
i = data.index('fn behind_tiles')
print('--- behind_tiles head ---'); print(data[i-500:i+100])
# shares_carrier_life head
i = data.index('fn shares_carrier_life')
print('--- shares_carrier_life head ---'); print(data[i-600:i+100])
