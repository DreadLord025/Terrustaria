import re, subprocess, sys
sys.stdout.reconfigure(encoding='utf-8', errors='replace')
def show(rev, path):
    return subprocess.run(['git', '-C', r'C:\Users\DreadLord\Desktop\TR-W4-d37', 'show', rev + ':' + path], capture_output=True).stdout.decode('utf8', 'replace')
src = show('a725acb04', 'crates/tr-app/src/npc/data.rs')
i = src.index('pub fn to_u16'); j = src.index('pub fn from_u16')
to = {m[0]: int(m[1]) for m in re.findall(r'NpcKind::(\w+)\s*=>\s*(\d+)', src[i:j])}
for n in ('DesertDjinn', 'Golfer', 'SleepingGolfer', 'Zoologist', 'Princess'):
    print(n, 'proto', to.get(n))
# d43 skips: which cs ids in 631-697 does d43 NOT port?
import json
c = json.load(open(r'C:\Users\DreadLord\Desktop\TR-W4-d37\tools\_d37_e8s_census.json'))['lanes']
d43cs = set(v for v in c['d43']['cs'].values() if v)
missing = [v for v in range(631, 698) if v not in d43cs]
print('d43 unported cs 631-697:', missing)
