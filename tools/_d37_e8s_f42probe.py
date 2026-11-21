import re, subprocess, sys
sys.stdout.reconfigure(encoding='utf-8', errors='replace')
def show(rev, path):
    return subprocess.run(['git', '-C', r'C:\Users\DreadLord\Desktop\TR-W4-d37', 'show', rev + ':' + path], capture_output=True).stdout.decode('utf8', 'replace')
f42 = show('425d4ff9d', 'crates/tr-app/src/npc.rs')
lines = f42.splitlines()
i = next(k for k, l in enumerate(lines) if 'fn update_kind_ai' in l)
seg = lines[i:i+1500]
for idx, l in enumerate(seg):
    if re.search(r'NpcKind::(Mummy|DarkMummy|BloodMummy|LightMummy)\b', l) and '=>' in l:
        print('f42:', ' | '.join(x.strip() for x in seg[idx:idx+3])[:150])
f42a = show('425d4ff9d', 'crates/tr-app/src/assets.rs')
m = re.search(r'STATIC_SHEET_NAMES[^=]*=\s*&?\[(.*?)\n\];', f42a, re.S)
npc = re.findall(r'"(NPC_[^"]+)"', re.sub(r'//[^\n]*', '', m.group(1)))
b = show('a725acb04', 'crates/tr-app/src/assets.rs')
m = re.search(r'STATIC_SHEET_NAMES[^=]*=\s*&?\[(.*?)\n\];', b, re.S)
bnpc = set(re.findall(r'"(NPC_[^"]+)"', re.sub(r'//[^\n]*', '', m.group(1))))
print('f42 new NPC sheets:', sorted(set(npc) - bnpc))
# f42 ENTRIES nets for the 9
f42b = show('425d4ff9d', 'crates/tr-app/src/bestiary.rs')
m = re.search(r'const ENTRIES[^=]*=\s*&?\[(.*?)\n\];', f42b, re.S)
for blk in re.split(r'BestiaryEntryDef\s*\{', m.group(1))[1:]:
    k = re.search(r'kind:\s*NpcKind::(\w+)', blk)
    if k and k.group(1) in ('Tumbleweed','Vulture','Antlion','Mummy','DarkMummy','BloodMummy','LightMummy','WalkingAntlion','FlyingAntlion'):
        n = re.search(r'net_id:\s*(-?\d+)', blk)
        print('f42 entry', k.group(1), 'net', n.group(1) if n else None)
