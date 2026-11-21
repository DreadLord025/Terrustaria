import re, subprocess, sys, json
sys.stdout.reconfigure(encoding='utf-8', errors='replace')
def show(rev, path):
    return subprocess.run(['git', '-C', r'C:\Users\DreadLord\Desktop\TR-W4-d37', 'show', rev + ':' + path], capture_output=True).stdout.decode('utf8', 'replace')
c = json.load(open(r'C:\Users\DreadLord\Desktop\TR-W4-d37\tools\_d37_e8s_census.json'))['lanes']
allids = set()
for l, d in c.items(): allids |= set(d['new'].values())
bsrc = show('a725acb04', 'crates/tr-app/src/npc/data.rs')
i = bsrc.index('pub fn to_u16'); j = bsrc.index('pub fn from_u16')
bids = set(int(m[1]) for m in re.findall(r'NpcKind::(\w+)\s*=>\s*(\d+)', bsrc[i:j]))
for q in (540, 546, 773, 774, 791, 792, 788, 789, 790):
    print(q, 'union-free:', q not in allids, 'base-free:', q not in bids)
# d23 SporeBat / QueenSlimeMinionPurple dispatch depth
d23 = show('b00e7626f', 'crates/tr-app/src/npc.rs')
lines = d23.splitlines()
i = next(k for k, l in enumerate(lines) if 'fn update_kind_ai' in l)
seg = lines[i:i+3000]
for idx, l in enumerate(seg):
    if re.search(r'NpcKind::(SporeBat|QueenSlimeMinionPurple)\b', l) and '=>' in l:
        print('d23:', ' | '.join(x.strip() for x in seg[idx:idx+3])[:150])
# d43 same
d43 = show('31310b652', 'crates/tr-app/src/npc.rs')
lines = d43.splitlines()
i = next(k for k, l in enumerate(lines) if 'fn update_kind_ai' in l)
seg = lines[i:i+3000]
for idx, l in enumerate(seg):
    if re.search(r'NpcKind::(SporeBat|QueenSlimeMinionPurple)\b', l) and '=>' in l:
        print('d43:', ' | '.join(x.strip() for x in seg[idx:idx+3])[:150])
