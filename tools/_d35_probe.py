import re, os
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
def rd(p): return open(os.path.join(ROOT, p), encoding='utf-8').read()
s = rd(r'crates\tr-app\src\npc\data.rs')
lines = s.split('\n')
seg = '\n'.join(lines[1504:1806])
print('hit_sound wildcards:', re.findall(r'^\s*_.*$', seg, re.M))
seg2 = '\n'.join(lines[1805:2124])
print('death_sound wildcards:', re.findall(r'^\s*_.*$', seg2, re.M))
i = s.index('| NpcKind::WyvernTail => "NPC_Hit_7"')
print('--- hit_sound wyvern ctx ---')
print(s[i-700:i+300])
i = s.index('| NpcKind::WyvernTail => "NPC_Killed_8"')
print('--- death_sound wyvern ctx ---')
print(s[i-500:i+300])
# no_gravity: where is AngryNimbus => true relative to the wildcard?
i = s.index('NpcKind::AngryNimbus => true,')
j = s.index('fn no_gravity')
seg3 = s[j:j+9000]
w = seg3.index('_ => false')
a = seg3.index('NpcKind::AngryNimbus => true,')
print('no_gravity: angrynimbus at', a, 'wildcard at', w, 'OK' if a < w else 'BAD')
# behind_tiles + shares_carrier_life structure around BlazingWheel/WyvernTail
i = s.index('| NpcKind::BlazingWheel\n')
print('--- behind_tiles ctx ---')
print(s[i-700:i+500])
i = s.index('| NpcKind::WyvernTail\n')
print('--- shares_carrier_life ctx ---')
print(s[i-800:i+400])
