import re, os
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
def rd(p): return open(os.path.join(ROOT, p), encoding='utf-8').read()
npc = rd(r'crates\tr-app\src\npc.rs')

# find_frame closing anchor
i = npc.index('pub(crate) fn find_frame(')
j = npc.index('fn update_slime', i)
print('--- find_frame -> update_slime junction ---')
print(repr(npc[j-60:j+80]))

# ai_005_hover_family Probe arm exact
i = npc.index('NpcKind::Probe => {}')
print('--- hover Probe arm ---'); print(repr(npc[i-40:i+60]))

# axis arm 9 exact
i = npc.index('9 => self.ai_009_sphere_family(world, player, rng),')
print('--- axis 9 ---'); print(repr(npc[i-20:i+70]))

# style-5 pin exact
i = npc.index('let five: Vec<_> = CS_STYLES')
j = npc.index('    }', i)
print('--- style5 pin ---'); print(npc[i:j])

# has_entry tests / all-kinds-entry assertions
for m in re.finditer(r'has_entry|entry_for_npc\(kind|every ported NPC kind', npc):
    ln = npc[:m.start()].count('\n')+1
    print('entry-assert', ln, npc.split('\n')[ln-1].strip()[:110])

# bestiary fold anchors: the wyvern fold block in try and entry
b = rd(r'crates\tr-app\src\bestiary.rs')
for fn in ['fn try_entry_for_npc','fn entry_for_npc']:
    i = b.index(fn)
    k = b.index('| NpcKind::WyvernTail => NpcKind::WyvernHead,', i)
    print('---', fn, 'wyvern fold tail ---'); print(repr(b[k:k+80]))
