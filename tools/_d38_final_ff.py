# W4-d38 FINAL sheet 2 - port find_frame arm census vs the C# case census
# + the findframe-lane claims (f26/f27/d44 pushed tips, via git show).
import json, re, subprocess
from pathlib import Path
ROOT = Path(__file__).resolve().parent.parent
C = json.load(open(ROOT / 'tools/_d38_final_census.json', encoding='utf-8'))
kinds = C['kinds']

npc_rs = (ROOT / 'crates/tr-app/src/npc.rs').read_text(encoding='utf-8')
i = npc_rs.index('pub(crate) fn find_frame(&mut self)')
# fn end: next "\n    pub" / "\n    fn" at the same impl indent
j = re.search(r'\n    (?:pub(?:\(crate\))? )?fn ', npc_rs[i + 10:]).start() + i + 10
ff = npc_rs[i:j]
OUT = {'find_frame_range': [npc_rs[:i].count('\n') + 1, npc_rs[:j].count('\n') + 1]}

# match arms: collect the pattern text before each "=>" at arm level.
# Simple approach: split on lines, accumulate pattern lines until "=>".
arms = {}   # kind -> list of (line, snippet-head)
pat_lines, start_line = [], None
base = npc_rs[:i].count('\n') + 1
depth = 0
for off, ln in enumerate(ff.split('\n')):
    s = ln.strip()
    if s.startswith('//') or not s:
        continue
    if '=>' in ln and ('NpcKind::' in ln or pat_lines):
        pat = ' '.join(pat_lines + [ln.split('=>')[0]])
        for k in re.findall(r'NpcKind::(\w+)', pat):
            if k in kinds:
                arms.setdefault(k, []).append((base + off, pat.strip()[:110]))
        pat_lines, start_line = [], None
    elif re.match(r'NpcKind::', s) or (pat_lines and s.endswith('|')):
        pat_lines.append(s)
OUT['port_arms'] = arms
OUT['port_covered'] = sorted(arms)
OUT['port_missing'] = sorted(set(kinds) - set(arms))

# what does each arm DO? capture the arm body (from => to the next arm at
# the same brace depth) - classify: delegate / inline / empty / row0-pin
bodies = {}
for k, hits in arms.items():
    ln_no = hits[0][0]
    seg = ff.split('\n')[ln_no - base:]
    # take until braces balance after the arm's opening
    txt, d, started = [], 0, False
    for l in seg:
        txt.append(l)
        d += l.count('{') - l.count('}')
        if '{' in l:
            started = True
        if started and d <= 0:
            break
        if not started and l.rstrip().endswith(',') and '=>' not in l:
            break
    body = '\n'.join(txt)
    kind_cls = ('delegate' if re.search(r'crate::npc_(?:findframe|ai)', body)
                else 'empty' if re.search(r'=>\s*\{\s*\}', body)
                else 'row0' if re.search(r'self\.frame = 0;', body) and len(body.split('\n')) <= 4
                else 'inline')
    bodies[k] = {'class': kind_cls, 'head': body.strip()[:200]}
OUT['arm_bodies'] = bodies

# ---- sibling findframe-lane claims over OUR ids (pushed tips) ----
tips = {
    'f26': 'origin/task/W4-f26-findframe-extra-3',
    'f27': 'origin/task/W4-f27-findframe-extra-4',
    'd44': 'origin/task/W4-d44-findframe-low-1',
}
def show(tip, path):
    r = subprocess.run(['git', 'show', f'{tip}:{path}'], capture_output=True, cwd=ROOT)
    return r.stdout.decode('utf-8', 'replace') if r.returncode == 0 else None
claims = {}
for lane, tip in tips.items():
    r = subprocess.run(['git', 'ls-tree', '--name-only', tip, 'crates/tr-app/src/'],
                       capture_output=True, cwd=ROOT)
    files = [f for f in r.stdout.decode().split('\n') if 'findframe' in f]
    hit = {}
    for f in files:
        src = show(tip, f)
        if src is None:
            continue
        # claimed ids: "cs id NNN" comments + FindFrameKind arms + case cites
        for m in re.finditer(r'\bcase (\d+)\b', src):
            tid = int(m.group(1))
            if tid in kinds.values():
                hit.setdefault(tid, []).append(f.split('/')[-1])
        for m in re.finditer(r'NPC\.cs:(\d+)', src):
            pass  # line cites checked via the case ids above
    claims[lane] = {'tip': subprocess.run(['git', 'rev-parse', '--short', tip],
                     capture_output=True, cwd=ROOT).stdout.decode().strip(),
                    'our_ids_claimed': {str(k): sorted(set(v)) for k, v in hit.items()}}
OUT['lane_claims'] = claims

json.dump(OUT, open(ROOT / 'tools/_d38_final_ff.json', 'w', encoding='utf-8'), indent=1)
print('find_frame range:', OUT['find_frame_range'])
print('port arms (%d):' % len(arms), sorted(arms))
print('port MISSING (%d):' % len(OUT['port_missing']), OUT['port_missing'])
from collections import Counter
print('classes:', Counter(b['class'] for b in bodies.values()))
for lane, c in claims.items():
    print(lane, c['tip'], 'claims over our ids:', c['our_ids_claimed'] or 'NONE')
