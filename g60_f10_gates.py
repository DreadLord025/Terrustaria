import re, json, sys

path = sys.argv[1]
lines = open(path, encoding='utf-8', errors='replace').read().split('\n')

overrun = json.load(open('g60_f10_overrun.json'))
over_names = {f['name'] for f in overrun['fns'] if f['min_cite'] >= 85560}
over_names |= {'five_gore_ladder'}

# dispatch region: hit_effect_c 5989..end (find its end)
disp_start = None
for i, l in enumerate(lines):
    if l.startswith('pub fn hit_effect_c('):
        disp_start = i
        break
depth = 0; started = False; e = disp_start
while e < len(lines):
    for ch in lines[e]:
        if ch == '{': depth += 1; started = True
        elif ch == '}': depth -= 1
    if started and depth == 0: break
    e += 1
disp_end = e
print(f"hit_effect_c: {disp_start+1}-{disp_end+1}")

# gates: comment-led blocks; a gate block = consecutive comment lines + if block
gates = []
i = disp_start + 1
while i < disp_end:
    l = lines[i]
    if re.match(r'\s*if ', l):
        # find start of preceding comment block
        s = i
        while s > disp_start and lines[s-1].strip().startswith('//'):
            s -= 1
        # find end of if block by brace depth
        depth = 0; started = False; j = i
        while j <= disp_end:
            for ch in lines[j]:
                if ch == '{': depth += 1; started = True
                elif ch == '}': depth -= 1
            if started and depth == 0: break
            j += 1
        block = '\n'.join(lines[s:j+1])
        called = sorted(set(re.findall(r'\b(\w+)\(', block)) & (over_names | {f['name'] for f in overrun['fns']}))
        cites = [int(x) for x in re.findall(r':(\d{5})', block)]
        gates.append({'start': s+1, 'end': j+1, 'calls': called,
                      'min_cite': min(cites) if cites else 0, 'max_cite': max(cites) if cites else 0,
                      'first': lines[i].strip()[:100]})
        i = j + 1
    else:
        i += 1

over_gates = [g for g in gates if g['min_cite'] >= 85560 or (set(g['calls']) & over_names and g['max_cite'] >= 85560)]
other_gates = [g for g in gates if g not in over_gates]
print(f"\n=== {len(over_gates)} OVERRUN dispatch gates ===")
for g in over_gates:
    print(f"{g['start']}-{g['end']} cites {g['min_cite']}-{g['max_cite']} calls={g['calls']} | {g['first']}")
print(f"\n=== other gates mentioning overrun fns (check!) ===")
for g in other_gates:
    if set(g['calls']) & over_names:
        print(f"{g['start']}-{g['end']} cites {g['min_cite']}-{g['max_cite']} calls={g['calls']} | {g['first']}")

# witnesses
tests = overrun['tests']
over_tests = []
legit_tests = []
for t in tests:
    cites_over = t['min_cite'] >= 85560
    calls_over = bool(set(t['body_fns']) & over_names)
    if cites_over or calls_over:
        over_tests.append(t)
    else:
        legit_tests.append(t)
print(f"\n=== {len(over_tests)} OVERRUN witnesses ===")
for t in over_tests:
    print(f"{t['name']}: {t['start']}-{t['end']} cites {t['min_cite']}-{t['max_cite']} fns={t['body_fns']}")
print(f"\n=== legit tests calling overrun fns (check!) ===")
for t in legit_tests:
    if set(t['body_fns']) & over_names:
        print(f"{t['name']}: {t['start']}-{t['end']} fns={t['body_fns']}")

json.dump({'gates': over_gates, 'tests': over_tests, 'disp': [disp_start+1, disp_end+1]},
          open('g60_f10_drops.json', 'w'), indent=1)
print(f"\ntotal tests: {len(tests)}; legit: {len(legit_tests)}")
