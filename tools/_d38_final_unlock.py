# W4-d38 FINAL sheet 3 - bestiary unlock census per kind: C# vs port.
# C#: BannerSystem.NPCtoBanner (case stacks) -> BannerToItem (range ifs)
#     -> ItemID.Sets.KillsToBanner -> CommonEnemy GetUnlockStateByKillCount;
#     critters ride CritterUICollectionInfoProvider (sight -> tier 4).
# Port: tr-core bestiary.rs npc_to_banner/banner_to_item/kills_to_banner_item
#     + tr-app bestiary.rs unlock_state (kill-ladder for ALL entries).
import json, re
from pathlib import Path
ROOT = Path(__file__).resolve().parent.parent
C = json.load(open(ROOT / 'tools/_d38_final_census.json', encoding='utf-8'))
kinds = C['kinds']

# ---- C# NPCtoBanner: consecutive case labels share the next return ----
bs = (ROOT / 'decompiled/Terraria/GameContent/BannerSystem.cs').read_text(encoding='utf-8')
i = bs.index('public static int NPCtoBanner')
j = bs.index('public static int ', i + 10)
blk = bs[i:j]
n2b, pending = {}, []
for ln in blk.split('\n'):
    m = re.match(r'\s*case (\d+):', ln)
    if m:
        pending.append(int(m.group(1)))
        continue
    m = re.match(r'\s*return (\d+);', ln)
    if m:
        for t in pending:
            n2b[t] = int(m.group(1))
        pending = []
# default: return 0 (tail verified in the census probe)
assert 'default:' in blk

# ---- C# BannerToItem: the if-chain in order ----
i = bs.index('public static int BannerToItem')
j = bs.index('public static int ', i + 10)
bblk = bs[i:j]
rules = []
for m in re.finditer(r'if \(banner == (\d+)\)\s*\{\s*return (\d+);', bblk):
    rules.append(('eq', int(m.group(1)), int(m.group(2)), 0))
for m in re.finditer(r'if \(banner >= (\d+)\)\s*\{\s*return (\d+) \+ banner - (\d+);', bblk):
    rules.append(('ge', int(m.group(1)), int(m.group(2)), int(m.group(3))))
# keep source order
rules = []
for m in re.finditer(r'if \(banner (==|>=) (\d+)\)\s*\{\s*return (\d+)(?: \+ banner - (\d+))?;', bblk):
    op, thr, base, off = m.group(1), int(m.group(2)), int(m.group(3)), m.group(4)
    rules.append((op, thr, base, int(off) if off else 0))
tail = re.search(r'\n\t\treturn (\d+) \+ banner - (\d+);\s*\}', bblk)
assert tail, 'BannerToItem tail'
def cs_banner_to_item(b):
    for op, thr, base, off in rules:
        if (op == '==' and b == thr) or (op == '>=' and b >= thr):
            return base + (b - off if off else 0)
    return int(tail.group(1)) + b - int(tail.group(2))

# ---- C# KillsToBanner (ItemID.cs:1304) ----
iid = (ROOT / 'decompiled/Terraria/ID/ItemID.cs').read_text(encoding='utf-8')
m = re.search(r'Sets\.KillsToBanner = Sets\.Factory\.CreateIntSet\(([^;]+)\);', iid)
args = m.group(1)
default_m = re.match(r'\s*Sets\.DefaultKillsForBannerNeeded', args)
assert default_m
pairs = re.findall(r'(\d+),\s*(\d+)', args)
cs_k2b = {int(a): int(b) for a, b in pairs}
def cs_kills(item):
    return cs_k2b.get(item, 50)

# ---- port maps ----
core = (ROOT / 'crates/tr-core/src/bestiary.rs').read_text(encoding='utf-8')
def fn_block(sig):
    i = core.index(sig)
    j = core.index('\n}', i)
    return core[i:j]
port_n2b = {int(a): int(v) for a, v in
            re.findall(r'(-?\d+) => (-?\d+),', fn_block('pub fn npc_to_banner'))}
pblk = fn_block('pub fn banner_to_item')
port_rules = [(m.group(1), int(m.group(2)), int(m.group(3)),
               int(m.group(4)) if m.group(4) else 0)
              for m in re.finditer(
                  r'if banner (==|>=) (\d+) \{\s*return (\d+)(?: \+ banner - (\d+))?;', pblk)]
ptail = re.search(r'\n    (\d+) \+ banner - (\d+)\s*$', pblk)
def port_banner_to_item(b):
    for op, thr, base, off in port_rules:
        if (op == '==' and b == thr) or (op == '>=' and b >= thr):
            return base + (b - off if off else 0)
    return int(ptail.group(1)) + b - int(ptail.group(2))
port_k2b = {int(a): int(v) for a, v in
            re.findall(r'(\d+) => (\d+),', fn_block('pub fn kills_to_banner_item'))}

rows = {}
for name, t in sorted(kinds.items(), key=lambda kv: kv[1]):
    cb = n2b.get(t, 0)
    ci = cs_banner_to_item(cb)
    ck = cs_kills(ci)
    pb = port_n2b.get(t, 0)
    pi = port_banner_to_item(pb)
    pk = port_k2b.get(pi, 50)
    critter = name in C['critters']
    rows[name] = {
        'id': t, 'critter': critter,
        'cs': {'banner': cb, 'item': ci, 'kills_full': ck,
               'tier2': ck // 5, 'tier3': ck // 2},
        'port': {'banner': pb, 'item': pi, 'kills_full': pk,
                 'tier2': pk // 5, 'tier3': pk // 2},
        'agree': cb == pb and ci == pi and ck == pk,
        'cs_provider': 'Critter(sight->4)' if critter else 'CommonEnemy(kill-ladder)',
        'port_provider': 'kill-ladder (unlock_state, no critter branch)',
    }
OUT = {'rows': rows,
       'all_chains_agree': all(r['agree'] for r in rows.values()),
       'critter_gap': [n for n, r in rows.items() if r['critter']],
       'rare_spawns': C['rare_spawns'],
       'quick_unlock_any': any(e['quick_unlock'] for e in C['entries'])}
json.dump(OUT, open(ROOT / 'tools/_d38_final_unlock.json', 'w', encoding='utf-8'), indent=1)
print('chains agree (banner/item/kills):', OUT['all_chains_agree'])
dis = {n: (r['cs'], r['port']) for n, r in rows.items() if not r['agree']}
print('disagreements:', dis or 'NONE')
print('critter-provider gap (%d):' % len(OUT['critter_gap']), OUT['critter_gap'])
print('rare spawns:', OUT['rare_spawns'], '| quick_unlock rows:', OUT['quick_unlock_any'])
for n, r in rows.items():
    print(r['id'], n, 'kills', r['cs']['kills_full'], '(t2@%d t3@%d)' % (r['cs']['tier2'], r['cs']['tier3']),
          'CRITTER' if r['critter'] else '')
