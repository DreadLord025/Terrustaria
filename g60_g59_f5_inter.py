import re, json

# g59's covered kinds (from its death_dust match arms)
g59 = [2,5,6,7,10,13,17,18,19,20,21,22,23,24,26,27,28,29,31,32,34,35,36,37,38,39,42,43,44,45,47,48,49,51,54,56,57,58,61,62,63,65,66,73,75,77,78,79,80,82,83,84,85,86,87,93,94,95,96,97,98,99,100,101,103,106,107,108,110,111,117,120,122,123,124,137,138,140,143,144,145,147,157,175,183,184,190,191,192,193,194,207,208,210,211,212,213,214,215,216,217,218,219,220,221,222,225,226,227,228,229]
g59 = set(g59)

# f5's dispatch: parse hit_effect_d gates (else-if / if chains) -> kinds + fn
import os
path = os.path.join(os.environ['TEMP'], 'f5_d.rs')
lines = open(path, encoding='utf-8', errors='replace').read().split('\n')
disp_start = None
for i, l in enumerate(lines):
    if l.startswith('pub fn hit_effect_d('):
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

gates = []
i = disp_start + 1
while i < disp_end:
    l = lines[i]
    m = re.match(r'\s*(?:else )?if (.+)', l)
    if m and ('npc_type' in l or 'matches!' in l):
        # the condition may continue onto the next line(s) before `{`
        cond = m.group(1)
        k = i
        while '{' not in cond and k + 1 < disp_end:
            k += 1
            cond += ' ' + lines[k].strip()
        cond = cond.split('{')[0]
        # find call in next lines
        j = i + 1
        call = None
        while j < disp_end and not lines[j].strip().startswith('}'):
            cm = re.match(r'\s*(\w+)\(', lines[j])
            if cm: call = cm.group(1); break
            j += 1
        # extract kinds from cond
        kinds = set()
        for km in re.finditer(r'npc_type == (\d+)', cond):
            kinds.add(int(km.group(1)))
        for km in re.finditer(r'\((\d+)\.\.=(\d+)\)', cond):
            kinds |= set(range(int(km.group(1)), int(km.group(2)) + 1))
        for km in re.finditer(r'\b(\d+)\.\.=(\d+)\b', cond):
            kinds |= set(range(int(km.group(1)), int(km.group(2)) + 1))
        # matches!(npc_type, 78 | 79 | ...) — may span lines WITHIN this gate
        # find this gate's closing brace first
        gd = 0; gs = False; ge = i
        while ge < disp_end:
            for ch in lines[ge]:
                if ch == '{': gd += 1; gs = True
                elif ch == '}': gd -= 1
            if gs and gd == 0: break
            ge += 1
        blob = '\n'.join(lines[i:ge + 1])
        mm = re.search(r'matches!\(npc_type,\s*([^)]+)\)', blob, re.S)
        if mm:
            for part in mm.group(1).split('|'):
                part = part.strip()
                rm = re.match(r'(\d+)\.\.=(\d+)', part)
                if rm: kinds |= set(range(int(rm.group(1)), int(rm.group(2)) + 1))
                elif part.isdigit(): kinds.add(int(part))
        gates.append({'line': i + 1, 'kinds': sorted(kinds), 'fn': call, 'cond': cond[:90]})
        i = j + 1
    else:
        i += 1

# intersect
inter = {}
for g in gates:
    hit = sorted(set(g['kinds']) & g59)
    if hit:
        inter[g['fn']] = {'line': g['line'], 'kinds': hit, 'all': g['kinds']}

allk = sorted({k for v in inter.values() for k in v['kinds']})
print(f"g59 kinds covered: {len(g59)}")
print(f"g59 INTERSECT f5 (latest tips): {len(allk)} kinds")
print(','.join(map(str, allk)))
print()
for fn, v in sorted(inter.items(), key=lambda x: x[1]['line']):
    print(f"{fn} @ {v['line']}: {v['kinds']}")
json.dump({'kinds': allk, 'fns': inter}, open('g60_g59_f5_inter.json', 'w'), indent=1)
