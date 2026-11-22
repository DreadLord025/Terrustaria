import re, subprocess, sys, json
sys.stdout.reconfigure(encoding='utf-8', errors='replace')
REPO = r'C:\Users\DreadLord\Desktop\TR-W4-d37'
def show(rev, path):
    return subprocess.run(['git', '-C', REPO, 'show', rev + ':' + path], capture_output=True).stdout.decode('utf8', 'replace')
def entries(rev):
    src = show(rev, 'crates/tr-app/src/bestiary.rs')
    m = re.search(r'const ENTRIES[^=]*=\s*&?\[(.*?)\n\];', src, re.S)
    out = []
    for blk in re.split(r'BestiaryEntryDef\s*\{', m.group(1))[1:]:
        k = re.search(r'kind:\s*NpcKind::(\w+)', blk)
        n = re.search(r'net_id:\s*(-?\d+)', blk)
        if k and n: out.append((k.group(1), int(n.group(1))))
    return out
# base nets
be = dict(entries('a725acb04'))
base_nets = set(n for n in be.values() if n > 0)
print('base positive nets:', len(base_nets))
# new-kind nets per claimant that COLLIDE with base nets
c = json.load(open(REPO + r'\tools\_d37_e8s_census.json'))['lanes']
TIPS = {l: c[l]['rev'] for l in c}
for l in ('c3','c4','c7','c8','c9','c10','d9','d11','d23','d120','d125','d33','d34','d35','d36','d37','d38','d39','d40','d41','d42','d43','e26','f18','f42','f47','g5','g8','g10'):
    ne = c[l]['new_entries']
    clash = {k: n for k, n in ne.items() if n > 0 and n in base_nets}
    if clash:
        print(l, 'NEW-ENTRY NET CLASHES WITH BASE:', clash)
# VileSpit d35 net
d35 = dict(entries('1afd327b9'))
print('d35 VileSpit net:', d35.get('VileSpit'))
print('d35 VileSpit in entries:', 'VileSpit' in d35)
# d23 VileSpit / VileSpitEoW / SolarFlare nets
d23 = dict(entries('43091b01c'))
for k in ('VileSpit','VileSpitEoW','SolarFlare'):
    print('d23', k, 'net', d23.get(k))
# d43 VileSpitEaterOfWorlds net
d43 = dict(entries('0253bde90'))
print('d43 VileSpitEaterOfWorlds net:', d43.get('VileSpitEaterOfWorlds'))
# f18 ArmedZombie net + hide
f18src = show('a832d38ea', 'crates/tr-app/src/bestiary.rs')
m = re.search(r'const ENTRIES[^=]*=\s*&?\[(.*?)\n\];', f18src, re.S)
for blk in re.split(r'BestiaryEntryDef\s*\{', m.group(1))[1:]:
    k = re.search(r'kind:\s*NpcKind::(\w+)', blk)
    if k and k.group(1) == 'ArmedZombie':
        print('f18 ArmedZombie entry block:', blk[:250].replace('\n',' '))
