import re, os
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
def rd(p): return open(os.path.join(ROOT, p), encoding='utf-8').read()

# projectile.rs 23780 match
pr = rd(r'crates\tr-app\src\projectile.rs')
lines = pr.split('\n')
print('=== projectile 23775-23800 ==='); print('\n'.join(lines[23774:23800]))

# behind_tiles: locate BlazingWheel anchor
data = rd(r'crates\tr-app\src\npc\data.rs')
i = data.index('fn behind_tiles')
j = data.index('\n    }', i)
seg = data[i:j]
print('--- behind_tiles has BlazingWheel:', '| NpcKind::BlazingWheel' in seg)
print('behind_tiles body tail:', seg[-400:])

# CS_STYLES: any len() or other pins
npc = rd(r'crates\tr-app\src\npc.rs')
for m in re.finditer(r'CS_STYLES\.len\(\)|CS_STYLES\.iter\(\)', npc):
    ln = npc[:m.start()].count('\n')+1
    print('CS_STYLES use', ln, npc.split('\n')[ln-1].strip()[:100])

# C# rotation: Main.cs DrawNPCs rotation gate - find which types rotate
main = rd(r'decompiled\Terraria\Main.cs')
# search for npc.rotation usage in DrawNPC context with type checks
idxs = [m.start() for m in re.finditer(r'\.rotation != 0', main)]
print('Main.cs rotation!=0 hits:', len(idxs))
for x in idxs[:6]:
    ln = main[:x].count('\n')+1
    print('  @', ln, main[x-60:x+120].replace('\n',' '))
