import re, subprocess, sys
sys.stdout.reconfigure(encoding='utf-8', errors='replace')
def show(rev, path):
    return subprocess.run(['git', '-C', r'C:\Users\DreadLord\Desktop\TR-W4-d37', 'show', rev + ':' + path], capture_output=True).stdout.decode('utf8', 'replace')
d9 = show('7a418c636', 'crates/tr-app/src/render/entities.rs')
print('d9 entities CultistDragon/NPC_454 refs:', len([l for l in d9.splitlines() if 'CultistDragon' in l or 'NPC_454' in l]))
d9a = show('7a418c636', 'crates/tr-app/src/assets.rs')
m = re.search(r'STATIC_SHEET_NAMES[^=]*=\s*&?\[(.*?)\n\];', d9a, re.S)
npc = re.findall(r'"(NPC_[^"]+)"', re.sub(r'//[^\n]*', '', m.group(1)))
print('d9 NPC sheets total:', len(npc))
b = show('a725acb04', 'crates/tr-app/src/assets.rs')
m = re.search(r'STATIC_SHEET_NAMES[^=]*=\s*&?\[(.*?)\n\];', b, re.S)
bnpc = set(re.findall(r'"(NPC_[^"]+)"', re.sub(r'//[^\n]*', '', m.group(1))))
print('d9 new NPC sheets:', sorted(set(npc) - bnpc))
# d9 slots
m = re.search(r'BESTIARY_NPC_SHEET_IDS:\s*\[u16;\s*(\d+)\]', d9a)
print('d9 slots decl:', m.group(1) if m else 'NO')
