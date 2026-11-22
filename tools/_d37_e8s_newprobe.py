import re, subprocess, sys
sys.stdout.reconfigure(encoding='utf-8', errors='replace')
def show(rev, path):
    return subprocess.run(['git', '-C', r'C:\Users\DreadLord\Desktop\TR-W4-d37', 'show', rev + ':' + path], capture_output=True).stdout.decode('utf8', 'replace')
# d34 phase-4 new kinds dispatch
d34 = show('f0d92564e', 'crates/tr-app/src/npc.rs')
lines = d34.splitlines()
i = next(k for k, l in enumerate(lines) if 'fn update_kind_ai' in l)
seg = lines[i:i+2500]
for idx, l in enumerate(seg):
    if re.search(r'NpcKind::(PinkJellyfish|Crab|DungeonGuardian|Antlion|SpikeBall)\b', l) and '=>' in l:
        print('d34:', ' | '.join(x.strip() for x in seg[idx:idx+3])[:150])
# d34 ai_style for the 5
d34d = show('f0d92564e', 'crates/tr-app/src/npc/data.rs')
i = d34d.index('fn ai_style'); j = d34d.index('{', i); d = 0; k = j
while True:
    ch = d34d[k]
    if ch == '{': d += 1
    elif ch == '}':
        d -= 1
        if d == 0: break
    k += 1
body = d34d[j:k+1]
for n in ('PinkJellyfish','Crab','DungeonGuardian','Antlion','SpikeBall'):
    m = re.search(r'NpcKind::' + n + r'\s*=>\s*([^,\n]+)', body)
    print('d34 ai_style', n, m.group(1).strip() if m else 'NO')
# d41 SolarFlare stats + dispatch
d41 = show('065efdf97', 'crates/tr-app/src/npc/data.rs')
i = d41.index('fn max_hp'); seg = d41[i:i+200000]
m = re.search(r'NpcKind::SolarFlare\s*=>\s*([^,\n]+)', seg)
print('d41 SolarFlare max_hp', m.group(1).strip() if m else 'NO')
d41n = show('065efdf97', 'crates/tr-app/src/npc.rs')
print('d41 SolarFlare dispatch:', [l.strip()[:90] for l in d41n.splitlines() if 'SolarFlare' in l and '=>' in l][:3])
# g8 Vampire dispatch
g8 = show('dd3cb0572', 'crates/tr-app/src/npc.rs')
print('g8 Vampire dispatch:', [l.strip()[:90] for l in g8.splitlines() if 'Vampire' in l and '=>' in l and 'Bat' not in l][:3])
