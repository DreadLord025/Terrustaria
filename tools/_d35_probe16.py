import re, os, sys
sys.path.insert(0, os.path.dirname(__file__))
from _d35_model import KINDS
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
def rd(p): return open(os.path.join(ROOT, p), encoding='utf-8').read()
npc = rd(r'crates\tr-app\src\npc.rs')
data = rd(r'crates\tr-app\src\npc\data.rs')
net = rd(r'crates\tr-app\src\app\netglue.rs')
aud = rd(r'crates\tr-app\src\audio.rs')
ent = rd(r'crates\tr-app\src\render\entities.rs')
uib = rd(r'crates\tr-app\src\ui\bestiary.rs')

# from_u16 id set vs 429..461
i = data.index('pub fn from_u16'); j = data.index('pub fn ai_style') if 'pub fn ai_style' in data else data.index('fn ai_style', i)
seg = data[i:j]
dec = set(int(m.group(1)) for m in re.finditer(r'^\s*(\d+)\s*=>', seg, re.M))
print('from_u16 count', len(dec), 'max', max(dec), 'clash 429..461:', sorted(dec & set(range(429,462))))

# net_id fn
i = data.index('fn net_id') if 'fn net_id' in data else npc.index('fn net_id')
src = data if 'fn net_id' in data else npc
print('--- net_id ---'); print(src[i-300:i+500])

# audio rows present
for h in sorted({k['hit'] for k in KINDS if k['hit'] is not None}):
    print('NPC_Hit_%d present:' % h, ('("NPC_Hit_%d",' % h) in aud)
for d in sorted({k['death'] for k in KINDS if k['death'] is not None}):
    print('NPC_Killed_%d present:' % d, ('("NPC_Killed_%d",' % d) in aud)
i = aud.index('("NPC_Killed_33", 4),')
print('--- audio ctx ---'); print(aud[i-900:i+200])

# npc_seed tail + ctor signatures
i = net.index('fn npc_seed'); j = net.index('fn npc_from_net')
print('--- npc_seed tail ---'); print(net[j-1100:j])
print('spawn_cave_walker sig:', re.search(r'fn spawn_cave_walker\([^)]*\)', npc).group(0))
print('spawn_hover sig:', re.search(r'fn spawn_hover\([^)]*\)', npc).group(0))

# s1_f_3 count anchor
m = re.search(r'covered \+ sentinel,\n            (\d+),', npc)
print('s1_f_3 count:', m.group(1) if m else 'MISSING')

# ui/bestiary test around 2040 (tabled)
i = uib.index('let tabled =')
print('--- tabled test ctx ---'); print(uib[i-1500:i+900])
