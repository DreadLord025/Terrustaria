import re, os, sys

def read(p):
    return open(p, encoding='utf-8').read().split('\n')

mine = read(r'crates\tr-app\src\npc.rs')
g62 = read(os.path.join(os.environ['TEMP'], 'g62_npc.rs'))

def extract(lines, start_pat, end_pat):
    s = e = -1
    for i, l in enumerate(lines):
        if s < 0 and re.search(start_pat, l):
            s = i
        if s >= 0 and re.search(end_pat, l):
            e = i
            break
    return lines[s:e+1]

pairs = [
    ('fields', r'NPC\.oldPos.*NPC\.cs:6119.*position trail$', r'pub trail_len: usize,'),
    ('init', r"NPC\.oldPos.*s field init", r'trail_len: kind\.trail_cache_length\(\),'),
]
ok = True
for name, sp, ep in pairs:
    a = extract(mine, sp, ep)
    b = extract(g62, sp, ep)
    eq = a == b
    ok &= eq
    print(f'{name}: mine={len(a)} g62={len(b)} equal={eq}')
    if not eq:
        for k in range(max(len(a), len(b))):
            x = a[k] if k < len(a) else '<missing>'
            y = b[k] if k < len(b) else '<missing>'
            if x != y:
                print(f'  DIFF {k}:\n  MINE: {x!r}\n  G62 : {y!r}')
                break

def extract_fn(lines):
    s = e = -1
    for i, l in enumerate(lines):
        if re.search(r'UpdateNPC_UpdateTrails.*90779-90875.*per-type', l):
            s = i
            break
    for j in range(s, len(lines)):
        if re.search(r'self\.old_rot\[0\] = self\.rotation;', lines[j]):
            e = j + 1
            break
    return lines[s:e+1]

a = extract_fn(mine); b = extract_fn(g62)
eq = a == b; ok &= eq
print(f'update_trails: mine={len(a)} g62={len(b)} equal={eq}')
if not eq:
    for k in range(max(len(a), len(b))):
        x = a[k] if k < len(a) else '<missing>'
        y = b[k] if k < len(b) else '<missing>'
        if x != y:
            print(f'  DIFF {k}:\n  MINE: {x!r}\n  G62 : {y!r}')
            break

def extract_transform(lines):
    for i, l in enumerate(lines):
        if 'trail resize + zero loop' in l:
            return lines[i:i+7]
    return []

a = extract_transform(mine); b = extract_transform(g62)
eq = a == b; ok &= eq
print(f'transform: mine={len(a)} g62={len(b)} equal={eq}')

def extract_wit(lines):
    s = e = -1
    for i, l in enumerate(lines):
        if s < 0 and re.search(r'W4-g60 wiring .*UpdateNPC_UpdateTrails.*402 mode-7 arm', l):
            s = i
        if s >= 0 and 'pub(super) fn bare' in l:
            e = i - 1
            break
    return lines[s:e+1]

a = extract_wit(mine); b = extract_wit(g62)
eq = a == b; ok &= eq
print(f'witnesses: mine={len(a)} g62={len(b)} equal={eq}')
if not eq:
    for k in range(max(len(a), len(b))):
        x = a[k] if k < len(a) else '<missing>'
        y = b[k] if k < len(b) else '<missing>'
        if x != y:
            print(f'  DIFF {k}:\n  MINE: {x!r}\n  G62 : {y!r}')
            break

mineD = read(r'crates\tr-app\src\npc\data.rs')
g62D = read(os.path.join(os.environ['TEMP'], 'g62_data.rs'))

def extract_table(lines):
    s = e = -1
    for i, l in enumerate(lines):
        if s < 0 and re.search(r'NPCID\.Sets\.TrailCacheLength.*NPCID\.cs:10450.*FULL', l):
            s = i
        if s >= 0 and 'pub fn trail_cache_length' in l:
            for j in range(i, len(lines)):
                if lines[j] == '    }':
                    e = j
                    break
            break
    return lines[s:e+1]

a = extract_table(mineD); b = extract_table(g62D)
eq = a == b; ok &= eq
print(f'table: mine={len(a)} g62={len(b)} equal={eq}')
if not eq:
    for k in range(max(len(a), len(b))):
        x = a[k] if k < len(a) else '<missing>'
        y = b[k] if k < len(b) else '<missing>'
        if x != y:
            print(f'  DIFF {k}:\n  MINE: {x!r}\n  G62 : {y!r}')
            break

print('ALL BYTE-IDENTICAL' if ok else 'MISMATCH FOUND')
sys.exit(0 if ok else 1)
