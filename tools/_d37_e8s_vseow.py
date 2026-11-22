import re, subprocess, sys
sys.stdout.reconfigure(encoding='utf-8', errors='replace')
def show(rev, path):
    return subprocess.run(['git', '-C', r'C:\Users\DreadLord\Desktop\TR-W4-d37', 'show', rev + ':' + path], capture_output=True).stdout.decode('utf8', 'replace')
src = show('43091b01c', 'crates/tr-app/src/npc/data.rs')
i = src.index('fn max_hp'); seg = src[i:i+150000]
m = re.search(r'NpcKind::VileSpitEoW\s*=>\s*([^,\n]+)', seg)
print('d23 VSEoW max_hp', m.group(1).strip() if m else 'NO')
# C# VileSpitEaterOfWorlds 666 SetDefaults
cs = open(r'C:\Users\DreadLord\Desktop\TR-W4-d37\decompiled\Terraria\NPC.cs', encoding='utf-8', errors='replace').read()
i = cs.find('case 666:')
print('C# case 666:', cs[i:i+400].replace('\n', ' | ')[:380] if i >= 0 else 'NOT FOUND')
