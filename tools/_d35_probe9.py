import os, re
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
def rd(p): return open(os.path.join(ROOT, p), encoding='utf-8').read()
data = rd(r'crates\tr-app\src\npc\data.rs')
# cs_id tail (fallback)
i = data.index('pub(crate) fn cs_id')
j = data.index('\n    }', i)
seg = data[i:j]
print('--- cs_id tail ---')
print(seg[-1500:])
# wyvern cs_id rows?
for m in re.finditer(r'NpcKind::Wyvern\w+[^\n]*', seg):
    print('WYVERN CS_ID:', m.group(0))
# which functions exist
for fn in ['no_tile_collide', 'dont_count_me', 'net_always', 'npc_slots', 'rarity', 'catch_item', 'behind_tiles', 'no_gravity', 'shares_carrier_life', 'taken_damage_multiplier']:
    print(fn, 'fn exists:', ('fn ' + fn) in data or ('fn ' + fn) in rd(r'crates\tr-app\src\npc.rs'))
