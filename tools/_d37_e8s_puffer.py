import re, subprocess, sys
sys.stdout.reconfigure(encoding='utf-8', errors='replace')
def show(rev, path):
    return subprocess.run(['git', '-C', r'C:\Users\DreadLord\Desktop\TR-W4-d37', 'show', rev + ':' + path], capture_output=True).stdout.decode('utf8', 'replace')
# c4 update_swimmer puffer re-puff
c4 = show('8abf0f1d8', 'crates/tr-app/src/npc.rs')
i = c4.find('fn update_swimmer'); j = c4.find('fn ', i + 10); seg = c4[i:j]
print('c4 update_swimmer puffer refs:', [l.strip()[:90] for l in seg.splitlines() if 'uffer' in l or '688' in l][:6])
# d43 update_swimmer puffer
d43 = show('31310b652', 'crates/tr-app/src/npc.rs')
i = d43.find('fn update_swimmer'); j = d43.find('fn ', i + 10); seg = d43[i:j]
print('d43 update_swimmer puffer refs:', [l.strip()[:90] for l in seg.splitlines() if 'uffer' in l or '688' in l][:6])
# c4 Orca wet-anchor
i = c4.find('fn update_swimmer'); j = c4.find('fn ', i + 10); seg = c4[i:j]
print('c4 update_swimmer Orca refs:', [l.strip()[:90] for l in seg.splitlines() if 'Orca' in l or '692' in l][:4])
# d43 Orca
i = d43.find('fn update_swimmer'); j = d43.find('fn ', i + 10); seg = d43[i:j]
print('d43 update_swimmer Orca refs:', [l.strip()[:90] for l in seg.splitlines() if 'Orca' in l or '692' in l][:4])
# C# pufferfish 688 justHit puff
cs = open(r'C:\Users\DreadLord\Desktop\TR-W4-d37\decompiled\Terraria\NPC.cs', encoding='utf-8', errors='replace').read()
i = cs.find('this.type == 688')
print('C# 688 SetDefaults:', cs[i:i+300].replace('\n', ' | ')[:280] if i >= 0 else 'none')
