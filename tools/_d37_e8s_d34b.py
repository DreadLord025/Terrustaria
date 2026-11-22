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
for q in (772, 773, 774, 775):
    print(q, 'union-free:', q not in allids, 'base-free:', q not in bids)
# g8 PinkJellyfish wiring: style18 axis machine vs d34 update_jellyfish
g8 = show('dd3cb0572', 'crates/tr-app/src/npc.rs')
i = g8.find('fn update_rng_dust'); j = g8.find('fn ', i + 10); seg = g8[i:j]
k = seg.find('18 =>')
print('g8 style18 axis arm:', seg[k:k+200].replace('\n', ' | ')[:190] if k >= 0 else 'NONE')
print('g8 PinkJellyfish kind arm:', [l.strip()[:80] for l in g8.splitlines() if 'PinkJellyfish =>' in l][:2])
# d34 update_jellyfish vs g8 style18: same machine?
d34 = show('f0d92564e', 'crates/tr-app/src/npc.rs')
i = d34.find('fn update_jellyfish'); j = d34.find('fn ', i + 10)
print('d34 update_jellyfish lines:', len(d34[i:j].splitlines()))
