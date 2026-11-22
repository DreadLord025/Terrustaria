import re, json, sys

path = sys.argv[1]
lines = open(path, encoding='utf-8', errors='replace').read().split('\n')

# find all top-level fn signatures with their doc-comment start
fns = []
for i, l in enumerate(lines):
    m = re.match(r'^(?:pub(?:\(crate\))? )?fn (\w+)', l)
    if m:
        # walk back over doc comments / attrs
        s = i
        while s > 0 and (lines[s-1].lstrip().startswith('///') or lines[s-1].lstrip().startswith('#[') or lines[s-1].lstrip().startswith('//')):
            s -= 1
        fns.append({'name': m.group(1), 'sig': i+1, 'doc_start': s+1})

# end of each fn: closing brace at column 0
for k, f in enumerate(fns):
    e = f['sig']-1
    depth = 0
    started = False
    while e < len(lines):
        for ch in lines[e]:
            if ch == '{':
                depth += 1; started = True
            elif ch == '}':
                depth -= 1
        if started and depth == 0:
            break
        e += 1
    f['end'] = e+1
    body = '\n'.join(lines[f['doc_start']-1:f['end']])
    cites = [int(x) for x in re.findall(r':(\d{5})', body)]
    f['min_cite'] = min(cites) if cites else 0
    f['max_cite'] = max(cites) if cites else 0

# test fns too
tests = []
for i, l in enumerate(lines):
    m = re.match(r'^\s+fn (w4_f10\w+|test\w*)\(', l)
    if m:
        s = i
        while s > 0 and (lines[s-1].lstrip().startswith('///') or lines[s-1].lstrip().startswith('#[') or lines[s-1].lstrip().startswith('//')):
            s -= 1
        # end: matching brace at same indent (4 spaces typically)
        indent = len(l) - len(l.lstrip())
        e = i
        depth = 0; started = False
        while e < len(lines):
            for ch in lines[e]:
                if ch == '{': depth += 1; started = True
                elif ch == '}': depth -= 1
            if started and depth == 0: break
            e += 1
        body = '\n'.join(lines[s:e+1])
        cites = [int(x) for x in re.findall(r':(\d{5})', body)]
        tests.append({'name': m.group(1), 'start': s+1, 'end': e+1,
                      'min_cite': min(cites) if cites else 0,
                      'max_cite': max(cites) if cites else 0,
                      'body_fns': sorted(set(re.findall(r'\b(\w+_death|\w+_hit|\w+_burst|\w+_ladder|\w+_dust|\w+_confetti|\w+_gores|\w+_spatter)\(', body)))})

out = {'fns': fns, 'tests': tests, 'total_lines': len(lines)}
json.dump(out, open(sys.argv[2], 'w'), indent=1)

# summary: fns with min_cite >= 85560 (overrun) vs legit
print("=== fns with citations >= 85560 (f5-claimed window) ===")
for f in fns:
    if f['min_cite'] >= 85560:
        print(f"{f['name']}: {f['doc_start']}-{f['end']} cites {f['min_cite']}-{f['max_cite']}")
print("=== fns straddling or below ===")
for f in fns:
    if 0 < f['min_cite'] < 85560 and f['max_cite'] >= 85560:
        print(f"STRADDLE {f['name']}: {f['doc_start']}-{f['end']} cites {f['min_cite']}-{f['max_cite']}")
