import re, subprocess, sys
sys.stdout.reconfigure(encoding='utf-8', errors='replace')
def show(rev, path):
    return subprocess.run(['git', '-C', r'C:\Users\DreadLord\Desktop\TR-W4-d37', 'show', rev + ':' + path], capture_output=True).stdout.decode('utf8', 'replace')
d23 = show('43091b01c', 'crates/tr-app/src/npc.rs')
lines = d23.splitlines()
i = next(k for k, l in enumerate(lines) if 'fn update_kind_ai' in l)
seg = lines[i:i+2000]
for idx, l in enumerate(seg):
    if re.search(r'NpcKind::(SolarFlare|VileSpitEoW|VileSpit)\b', l) and '=>' in l:
        print('d23:', ' | '.join(x.strip() for x in seg[idx:idx+3])[:150])
# d23 VileSpitEoW vs d43 VileSpitEaterOfWorlds: same vanilla type 666.
# d23 named it VileSpitEoW, d43 named it VileSpitEaterOfWorlds (the C# name).
# Both aiStyle 9. d43 wires style axis; d23?
print('d23 VileSpitEoW ai_style:')
d23d = show('43091b01c', 'crates/tr-app/src/npc/data.rs')
i = d23d.index('fn ai_style'); seg = d23d[i:i+150000]
for n in ('SolarFlare', 'VileSpitEoW', 'VileSpit'):
    m = re.search(r'NpcKind::' + n + r'\s*=>\s*([^,\n]+)', seg)
    print(' ', n, m.group(1).strip() if m else 'NO')
