"""Emit Rust match arms for the big frame->colour tables in TileLightScanner.

The five largest cases of `ApplyTileLight` (93 lamps, 33 candles, 34 chandeliers,
173/100 lanterns, 42 lanterns) are pure `switch (frame / N)` tables with a handful
of dynamic entries (paint, demon torch, shimmer). They are extracted mechanically
so the Rust port cannot drift from the decompiled source.
"""
import re

SRC = 'decompiled/Terraria/Graphics/Light/TileLightScanner.cs'
lines = open(SRC, encoding='utf-8').read().split('\n')
start = next(i for i, l in enumerate(lines) if 'private void ApplyTileLight' in l)

depth = 0
started = False
end = None
for i in range(start, len(lines)):
    for ch in lines[i]:
        if ch == '{':
            depth += 1
            started = True
        elif ch == '}':
            depth -= 1
    if started and depth <= 0:
        end = i
        break

depth = 0
started = False
labels = []
for i in range(start, end + 1):
    before = depth
    for ch in lines[i]:
        if ch == '{':
            depth += 1
        elif ch == '}':
            depth -= 1
    if re.match(r'^\s*case \d', lines[i]) and before == 3:
        labels.append(i)


def arm(case_id):
    for k, i in enumerate(labels):
        if re.match(r'^\s*case %d:' % case_id, lines[i]):
            nxt = labels[k + 1] if k + 1 < len(labels) else end
            return lines[i + 1:nxt]
    raise SystemExit('no case %d' % case_id)


def evalv(s):
    m = re.fullmatch(r'([\d.]+)f(?:\s*/\s*([\d.]+)f)?', s.strip())
    if not m:
        raise SystemExit('cannot evaluate %r' % s)
    v = float(m.group(1))
    if m.group(2):
        v /= float(m.group(2))
    return v


def parse(case_id):
    body = arm(case_id)
    text = '\n'.join(body)
    sw = re.search(r'switch \((.+?)\)', text)
    guard = re.search(r'if \((.+?)\)\s*\{\s*break;\s*\}', text)
    expr = sw.group(1).replace('tile.frameX', 'fx').replace('tile.frameY', 'fy')
    groups = []
    cur = None
    for line in body:
        m = re.match(r'^\s*case (\d+):\s*$', line)
        m2 = re.match(r'^\s*default:\s*$', line)
        if m or m2:
            cur = [int(m.group(1)) if m else 'default', []]
            groups.append(cur)
            continue
        if cur is None:
            continue
        t = line.strip()
        if t in ('{', '}', 'break;', '') or t.startswith('switch ('):
            continue
        cur[1].append(t)
    merged = []
    pending = []
    for lab, st in groups:
        if not st:
            pending.append(lab)
            continue
        merged.append((pending + [lab], st))
        pending = []
    if pending:
        raise SystemExit('trailing labels without body: %r' % pending)
    return (guard.group(1) if guard else None), expr, merged


def value(stmts):
    s = ' '.join(stmts)
    if 'tile.color() == 0' in s and 'flag = true' in s:
        return ('paint', [0.37, 0.8, 1.0])
    if 'hslToRgb(Main.demonTorch' in s:
        return ('demon_hsl', None)
    if 'GetShimmerBaseColor' in s:
        return ('shimmer', None)
    if 'Main.demonTorch' in s:
        return ('demon_torch', None)
    nums = re.findall(r'[RGB] = ([\d.]+f(?: / [\d.]+f)?);', s)
    if len(nums) == 3:
        kind = 'flag_const' if 'flag = true' in s else 'const'
        return (kind, [evalv(x) for x in nums])
    raise SystemExit('unknown arm %r' % s)


def fnum(v):
    return repr(round(v, 8))


for case_id in (93, 33, 34, 173, 42):
    guard, expr, groups = parse(case_id)
    print('        %d => {' % case_id)
    if guard:
        g = guard.replace('tile.frameX', 'fx').replace('tile.frameY', 'fy')
        if '!=' in g:
            g = g.replace('!=', '==')
        elif '>=' in g:
            g = g.replace('>=', '<')
        print('            if %s {' % g)
    print('            l = match %s {' % expr)
    dflt = None
    for labs, stmts in groups:
        kind, val = value(stmts)
        for lab in labs:
            if lab == 'default':
                dflt = val
                continue
            if kind == 'const':
                print('                %d => [%s, %s, %s],' % (lab, *[fnum(x) for x in val]))
            elif kind == 'paint':
                print('                %d => { if tile.paint() == 0 { [%s, %s, %s] } else { paint = true; [0.0; 3] } }'
                      % (lab, *[fnum(x) for x in val]))
            elif kind == 'flag_const':
                print('                %d => { paint = true; [%s, %s, %s] }'
                      % (lab, *[fnum(x) for x in val]))
            elif kind == 'demon_hsl':
                print('                %d => { let v = hsl_to_rgb(p.demon_torch * 0.12 + 0.69, 1.0, 0.75); [v[0] * 1.2, v[1] * 1.2, v[2] * 1.2] }' % lab)
            elif kind == 'demon_torch':
                print('                %d => torch_color(7, p),' % lab)
            elif kind == 'shimmer':
                print('                %d => { let v = shimmer_base_color(x, y, p.time_for_visuals); [(v[0] * 1.5).clamp(0.0, 1.0), (v[1] * 1.5).clamp(0.0, 1.0), (v[2] * 1.5).clamp(0.0, 1.0)] }' % lab)
    if dflt:
        print('                _ => [%s, %s, %s],' % tuple(fnum(x) for x in dflt))
    else:
        print('                _ => [0.0, 0.0, 0.0],')
    print('            };')
    if guard:
        print('            }')
    print('        }')
    print()
