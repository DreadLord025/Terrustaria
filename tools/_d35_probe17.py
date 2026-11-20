import re, os
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
def rd(p): return open(os.path.join(ROOT, p), encoding='utf-8').read()
b = rd(r'crates\tr-app\src\bestiary.rs')
i = b.index('pub const ENTRIES')
j = b.index('\n];', i)
seg = b[i:j]
nets = re.findall(r'net_id: (?:crate::loot_db::(\w+)|(-?\d+))', seg)
loot = dict((m.group(1), int(m.group(2))) for m in re.finditer(r'pub const (\w+): i32 = (-?\d+);', rd(r'crates\tr-app\src\loot_db.rs')))
vals = [loot[n] if n else int(d) for n, d in nets]
print('ENTRIES count:', len(vals))
print('sorted ascending?', vals == sorted(vals))
print('last 12 net ids:', vals[-12:])
# order tests?
for m in re.finditer(r'fn (\w*order\w*|\w*sorted\w*)\(', b):
    print('order test:', m.group(1))
uib = rd(r'crates\tr-app\src\ui\bestiary.rs')
# grid capacity
i = uib.index('fn grid_metrics'); print(uib[i:i+600])
i = uib.index('const CELL'); print(uib[i:i+120])
# tests asserting ENTRIES order
for m in re.finditer(r'ENTRIES', uib):
    pass
i = uib.find('ascending')
print('ascending ctx:', uib[max(0,i-300):i+300] if i>=0 else 'none')
