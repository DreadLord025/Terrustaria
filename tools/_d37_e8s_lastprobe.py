import re, subprocess, sys
sys.stdout.reconfigure(encoding='utf-8', errors='replace')
def show(rev, path):
    return subprocess.run(['git', '-C', r'C:\Users\DreadLord\Desktop\TR-W4-d37', 'show', rev + ':' + path], capture_output=True).stdout.decode('utf8', 'replace')
nid = open(r'C:\Users\DreadLord\Desktop\TR-W4-d37\decompiled\Terraria\ID\NPCID.cs', encoding='utf-8', errors='replace').read()
m = re.search(r'\{\s*478,\s*new NPCBestiaryDrawModifiers\(0\)\s*\{(.*?)\}\s*\}', nid, re.S)
print('C# 478 MothronEgg mods:', m.group(1).strip()[:120] if m else 'none')
# c10 style65 wired?
c10 = show('15f93d134', 'crates/tr-app/src/npc.rs')
i = c10.find('fn update_rng_dust'); j = c10.find('fn ', i + 10); seg = c10[i:j]
print('c10 style65 axis arm:', '65 =>' in seg)
print('c10 style65 calls:', [l.strip()[:80] for l in c10.splitlines() if 'style65::' in l][:3])
# d39 style65 calls
d39 = show('048c74f8f', 'crates/tr-app/src/npc.rs')
print('d39 style65 calls:', [l.strip()[:80] for l in d39.splitlines() if 'style65::' in l][:3])
# f18 style65 calls
f18 = show('a832d38ea', 'crates/tr-app/src/npc.rs')
print('f18 style65 calls:', [l.strip()[:80] for l in f18.splitlines() if 'style65::' in l][:3])
# d40 MothronEgg entry at current tip
d40 = show('78ee5647c', 'crates/tr-app/src/bestiary.rs')
m = re.search(r'const ENTRIES[^=]*=\s*&?\[(.*?)\n\];', d40, re.S)
for blk in re.split(r'BestiaryEntryDef\s*\{', m.group(1))[1:]:
    k = re.search(r'kind:\s*NpcKind::(\w+)', blk)
    if k and k.group(1) == 'MothronEgg':
        print('d40 MothronEgg entry:', blk[:150].replace('\n', ' '))
