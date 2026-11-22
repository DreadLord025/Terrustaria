import re, os, sys
sys.path.insert(0, os.path.dirname(__file__))
from _d35_model import KINDS
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
styles = sorted({k['ai'] for k in KINDS})
print('our aiStyles:', styles)
hits = []
for dp, dn, fns in os.walk(os.path.join(ROOT, 'crates')):
    dn[:] = [d for d in dn if d not in ('target', '.git')]
    for fn in fns:
        if not fn.endswith('.rs'): continue
        p = os.path.join(dp, fn)
        s = open(p, encoding='utf-8').read()
        rel = os.path.relpath(p, ROOT)
        # style-filter roster pins
        for m in re.finditer(r'\.filter\(\|&&\(_, s\)\| s == (\d+)\)', s):
            if int(m.group(1)) in styles:
                hits.append((rel, s[:m.start()].count('\n')+1, 'CS_STYLES style-filter '+m.group(1)))
        # ai_style() == N assertions / counts
        for st in styles:
            for m in re.finditer(r'ai_style\(\)\s*==\s*%d\b' % st, s):
                hits.append((rel, s[:m.start()].count('\n')+1, 'ai_style()==%d'%st))
        # ALL_KINDS.len() numeric pins
        for m in re.finditer(r'ALL_KINDS\.len\(\)', s):
            ln = s[:m.start()].count('\n')+1
            line = s.split('\n')[ln-1]
            if re.search(r'==\s*\d|,\s*\d{3}', line):
                hits.append((rel, ln, 'ALL_KINDS.len pin: '+line.strip()[:80]))
        # hardcoded 270 / 303 roster counts in tests
        for m in re.finditer(r'\b(270|303)\b', s):
            ln = s[:m.start()].count('\n')+1
            line = s.split('\n')[ln-1].strip()
            if ('assert' in line or 'len' in line or 'count' in line or 'roster' in line.lower()) and 'NpcKind' not in line:
                hits.append((rel, ln, 'count270/303: '+line[:80]))
for h in sorted(set(hits)):
    print(h)
print('total potential pins:', len(set(hits)))
