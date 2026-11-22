import re, subprocess, sys
sys.stdout.reconfigure(encoding='utf-8', errors='replace')
def show(rev, path):
    return subprocess.run(['git', '-C', r'C:\Users\DreadLord\Desktop\TR-W4-d37', 'show', rev + ':' + path], capture_output=True).stdout.decode('utf8', 'replace')
d33 = show('36aae03d0', 'crates/tr-app/src/npc.rs')
lines = d33.splitlines()
i = next(k for k, l in enumerate(lines) if 'fn update_kind_ai' in l)
seg = lines[i:i+3000]
for idx, l in enumerate(seg):
    if re.search(r'NpcKind::(FairyCritterPink|WindyBalloon)\b', l) and '=>' in l:
        print('d33:', ' | '.join(x.strip() for x in seg[idx:idx+3])[:150])
print('d33 style111/112/113 axis:', [l.strip()[:70] for l in d33.splitlines() if re.search(r'(111|112|113|114) =>', l)][:4])
d42 = show('adb304ab8', 'crates/tr-app/src/npc.rs')
lines = d42.splitlines()
i = next(k for k, l in enumerate(lines) if 'fn update_kind_ai' in l)
seg = lines[i:i+3000]
for idx, l in enumerate(seg):
    if re.search(r'NpcKind::(FairyCritterPink|WindyBalloon)\b', l) and '=>' in l:
        print('d42:', ' | '.join(x.strip() for x in seg[idx:idx+3])[:150])
