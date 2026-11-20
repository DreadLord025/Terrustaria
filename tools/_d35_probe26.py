import re, os
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
def rd(p): return open(os.path.join(ROOT, p), encoding='utf-8').read()
sid = rd(r'crates\tr-app\src\soundid.rs')
# SOUND_STYLES rows for sound_id 3 and 4
i = sid.index('SOUND_STYLES')
seg = sid[i:i+40000]
for m in re.finditer(r'SoundStyleDef\s*\{[^}]*\}', seg):
    blk = m.group(0)
    sm = re.search(r'sound_id:\s*(\d+)', blk); st = re.search(r'style:\s*(\d+)', blk)
    if sm and st and ((sm.group(1)=='3' and st.group(1) in ('5','54')) or (sm.group(1)=='4' and st.group(1) in ('7','9','28','52'))):
        print(re.sub(r'\s+',' ',blk)[:160])
# count styles per id
rows = re.findall(r'sound_id:\s*(\d+),\s*style:\s*(\d+)', seg)
if not rows:
    rows = [(m.group(1), m.group(2)) for m in re.finditer(r'sound_id:\s*(\d+)[^}]*?style:\s*(\d+)', seg)]
s3 = sorted(int(st) for sid_, st in rows if sid_=='3')
s4 = sorted(int(st) for sid_, st in rows if sid_=='4')
print('id3 styles:', s3)
print('id4 styles:', s4)
ent = rd(r'crates\tr-app\src\render\entities.rs')
i = ent.index('fn humanoid_draw'); print(ent[i:i+260])
i = ent.index('fn generic_anchor'); print(ent[i:i+320])
# draw_npcs match: wildcard?
i = ent.index('fn draw_npcs')
j = ent.index('NpcKind::DungeonSpirit => {', i)
k = ent.index('};', j)
seg = ent[i:k]
print('draw_npcs wildcards:', re.findall(r'^\s*_\s*=>[^\n]*', seg, re.M))
print('draw match head:', seg[seg.index('match'):seg.index('match')+300])
# spawner tests matches
sp = rd(r'crates\tr-app\src\spawner.rs')
lines = sp.split('\n')
print('=== spawner 3727 ==='); print('\n'.join(lines[3722:3740]))
ws = rd(r'crates\tr-app\src\world_sim\npcs.rs')
lines = ws.split('\n')
print('=== npcs 3540-3548 ==='); print('\n'.join(lines[3539:3548]))
print('=== npcs 4990-5000 ==='); print('\n'.join(lines[4989:5000]))
# dont_count / npc_slots modeled?
data = rd(r'crates\tr-app\src\npc\data.rs')
npc = rd(r'crates\tr-app\src\npc.rs')
for pat in ['dont_count', 'dontTake', 'npc_slots', 'npcSlots', 'catchable', 'catch_item']:
    print(pat, 'in data:', pat in data, 'in npc:', pat in npc)
