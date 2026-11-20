import re, os, sys
sys.path.insert(0, os.path.dirname(__file__))
from _d35_model import KINDS
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
def rd(p): return open(os.path.join(ROOT, p), encoding='utf-8').read()
npc = rd(r'crates\tr-app\src\npc.rs')

# citations test
i = npc.index('fn test_DW_4_5_the_citations_are_present')
j = npc.index('\n    }', i)
print('--- citations test ---'); print(npc[i:j+6])

# findframe_a roster: are any of our kinds listed?
ffa = rd(r'crates\tr-app\src\npc_findframe_a.rs')
names = {k['v'] for k in KINDS}
listed = [n for n in names if ('NpcKind::'+n) in ffa]
print('our kinds in npc_findframe_a:', listed)

# any ALL_KINDS loop asserting entry kind == kind?
for m in re.finditer(r'for (?:&?kind|\(kind[^)]*\)) in ALL_KINDS', npc):
    start = m.start()
    end = npc.find('\n    }', start)
    body = npc[start:end]
    if 'entry_for_npc' in body or 'has_entry' in body:
        ln = npc[:start].count('\n')+1
        print('ALL_KINDS entry loop at', ln)

# bestiary ENTRIES count assertions anywhere
for dp, dn, fns in os.walk(os.path.join(ROOT,'crates')):
    dn[:] = [d for d in dn if d not in ('target','.git')]
    for fn in fns:
        if not fn.endswith('.rs'): continue
        s = open(os.path.join(dp,fn),encoding='utf-8').read()
        for m in re.finditer(r'ENTRIES\.len\(\)', s):
            ln = s[:m.start()].count('\n')+1
            line = s.split('\n')[ln-1].strip()
            if 'assert' in line or '==' in line:
                print(os.path.relpath(os.path.join(dp,fn),ROOT), ln, line[:110])
