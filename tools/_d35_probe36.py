import re, os
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
def rd(p): return open(os.path.join(ROOT, p), encoding='utf-8').read()
npc = rd(r'crates\tr-app\src\npc.rs')

# find_frame outer match: subject + top-level (12-space) arms + wildcard + deny
i = npc.index('pub(crate) fn find_frame(')
# find the outer `match self.kind {`
m = re.search(r'match self\.kind \{', npc[i:i+6000])
print('find_frame outer match subject found:', bool(m), 'at offset', m.start() if m else None)
if m:
    start = i + m.end()
    # top-level arms at 12 spaces
    seg = npc[start:start+40000]
    top = re.findall(r'^            (NpcKind::\w+|_)([^\n]*)', seg, re.M)
    print('find_frame top-level arm count:', len(top))
    print('find_frame top-level wildcard:', [t for t in top if t[0]=='_'])
    print('find_frame last 3 top arms:', top[-3:])
print('find_frame deny attr:', 'deny' in npc[i-100:i+200])

# axis-shape tests scanning npc.rs include_str
for m in re.finditer(r'include_str!\("npc\.rs"\)', npc):
    ln = npc[:m.start()].count('\n')+1
    # find enclosing test fn
    fns = list(re.finditer(r'fn (\w+)\(', npc[:m.start()]))
    print('include_str npc.rs test:', fns[-1].group(1), 'at', ln)

# sphere launch_speed default
s9 = rd(r'crates\tr-app\src\npc_ai_style9.rs')
i = s9.index('fn launch_speed')
print('--- launch_speed ---'); print(s9[i:i+700])

# C# rotation for 83/84/101 (Main.cs DrawNPC type gates)
main = rd(r'decompiled\Terraria\Main.cs')
for cid in [83,84,101]:
    # find DrawNPC arms referencing type==cid with rotation
    hits = [m.start() for m in re.finditer(r'rCurrentNPC\.type == %d\b' % cid, main)]
    rot_near = 0
    for x in hits:
        ctx = main[x:x+1500]
        if 'rotation' in ctx: rot_near += 1
    print('C# type', cid, 'DrawNPC hits', len(hits), 'with rotation nearby', rot_near)
