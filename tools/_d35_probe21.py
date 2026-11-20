import re, os
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
def rd(p): return open(os.path.join(ROOT, p), encoding='utf-8').read()
npc = rd(r'crates\tr-app\src\npc.rs')
# the 0-arm matches: check what they match on
for ln in [20187, 32329, 32560]:
    lines = npc.split('\n')
    print('=== npc.rs', ln, '===')
    print('\n'.join(lines[ln-3:ln+8]))
data = rd(r'crates\tr-app\src\npc\data.rs')
i = data.index('fn cs_id')
print('--- cs_id head ---'); print(data[i-400:i+150])
j = data.index('}', data.index('match', i))
# find wildcard in cs_id
seg = data[i:i+20000]
print('cs_id wildcard:', re.findall(r'^\s*_ =>.*$', seg, re.M)[:2])
# hit_sound / death_sound wildcard
for fn in ['fn hit_sound', 'fn death_sound']:
    i = data.index(fn); seg = data[i:i+30000]
    w = re.findall(r'^\s*_ =>.*$', seg, re.M)[:2]
    print(fn, 'wildcard:', w)
# netglue npc_seed wildcard? probe20 said exhaustive with 30 arms
net = rd(r'crates\tr-app\src\app\netglue.rs')
i = net.index('fn npc_seed')
seg = net[i:i+16000]
print('npc_seed head:'); print(seg[:900])
print('npc_seed wildcard:', re.findall(r'^\s*_ =>.*$', seg, re.M)[:3])
# world_sim npcs update_npcs match at 4993
ws = rd(r'crates\tr-app\src\world_sim\npcs.rs')
lines = ws.split('\n')
print('=== npcs.rs 4993 ==='); print('\n'.join(lines[4988:5000]))
print('=== npcs.rs 3545 carrier_link_pre_pass ==='); print('\n'.join(lines[3540:3560]))
