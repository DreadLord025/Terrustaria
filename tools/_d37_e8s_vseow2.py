import re, subprocess, sys
sys.stdout.reconfigure(encoding='utf-8', errors='replace')
def show(rev, path):
    return subprocess.run(['git', '-C', r'C:\Users\DreadLord\Desktop\TR-W4-d37', 'show', rev + ':' + path], capture_output=True).stdout.decode('utf8', 'replace')
def fnbody(src, sig):
    i = src.index(sig); j = src.index('{', i); d = 0; k = j
    while True:
        c = src[k]
        if c == '{': d += 1
        elif c == '}':
            d -= 1
            if d == 0: return src[j:k+1]
        k += 1
src = show('43091b01c', 'crates/tr-app/src/npc/data.rs')
body = fnbody(src, 'fn max_hp')
# VileSpitEoW may share an arm with VileSpit
for l in body.splitlines():
    if 'VileSpit' in l:
        print('d23 max_hp line:', l.strip()[:120])
# find the shared arm value
m = re.search(r'NpcKind::VileSpit\s*\|\s*NpcKind::VileSpitEoW\s*=>\s*([^,\n]+)', body)
print('d23 VileSpit|VileSpitEoW max_hp:', m.group(1).strip() if m else 'NO')
