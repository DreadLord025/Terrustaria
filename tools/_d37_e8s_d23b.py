import re, subprocess, sys
sys.stdout.reconfigure(encoding='utf-8', errors='replace')
def show(rev, path):
    return subprocess.run(['git', '-C', r'C:\Users\DreadLord\Desktop\TR-W4-d37', 'show', rev + ':' + path], capture_output=True).stdout.decode('utf8', 'replace')
d23 = show('43091b01c', 'crates/tr-app/src/npc.rs')
hits = [l.strip()[:110] for l in d23.splitlines() if 'VileSpitEoW' in l]
for h in hits[:10]: print(h)
# d43 VileSpitEaterOfWorlds ai_style + dispatch
d43 = show('0253bde90', 'crates/tr-app/src/npc/data.rs')
i = d43.index('fn ai_style'); seg = d43[i:i+150000]
m = re.search(r'NpcKind::VileSpitEaterOfWorlds\s*=>\s*([^,\n]+)', seg)
print('d43 VileSpitEaterOfWorlds ai_style:', m.group(1).strip() if m else 'NO')
