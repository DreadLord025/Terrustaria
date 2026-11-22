import re, subprocess, sys
sys.stdout.reconfigure(encoding='utf-8', errors='replace')
def show(rev, path):
    return subprocess.run(['git', '-C', r'C:\Users\DreadLord\Desktop\TR-W4-d37', 'show', rev + ':' + path], capture_output=True).stdout.decode('utf8', 'replace')
src = show('f0d92564e', 'crates/tr-app/src/npc.rs')
i = src.find('fn update_rng_dust'); j = src.find('fn ', i + 10)
seg = src[i:j]
print('d34 style18 axis arm:', '18 =>' in seg)
print('d34 PinkJellyfish kind dispatch:', [l.strip()[:90] for l in src.splitlines() if 'PinkJellyfish =>' in l and 'self.' in l][:2])
# d34 Crab/DungeonGuardian entries + sheets
b = show('f0d92564e', 'crates/tr-app/src/bestiary.rs')
m = re.search(r'const ENTRIES[^=]*=\s*&?\[(.*?)\n\];', b, re.S)
for blk in re.split(r'BestiaryEntryDef\s*\{', m.group(1))[1:]:
    k = re.search(r'kind:\s*NpcKind::(\w+)', blk)
    if k and k.group(1) in ('Crab', 'DungeonGuardian', 'PinkJellyfish', 'Antlion', 'SpikeBall'):
        n = re.search(r'net_id:\s*(-?\d+)', blk)
        print('d34 entry', k.group(1), 'net', n.group(1) if n else None)
# free-id check for 772/773 against the whole current union
import json
c = json.load(open(r'C:\Users\DreadLord\Desktop\TR-W4-d37\tools\_d37_e8s_census.json'))['lanes']
allids = set()
for l, d in c.items():
    allids |= set(d['new'].values())
print('772 free:', 772 not in allids, '| 773 free:', 773 not in allids)
print('827 in union (g5 pushed):', 827 in allids)
# base ids
bsrc = show('a725acb04', 'crates/tr-app/src/npc/data.rs')
i = bsrc.index('pub fn to_u16'); j = bsrc.index('pub fn from_u16')
bids = set(int(m[1]) for m in re.findall(r'NpcKind::(\w+)\s*=>\s*(\d+)', bsrc[i:j]))
print('772/773 base-free:', 772 not in bids and 773 not in bids)
