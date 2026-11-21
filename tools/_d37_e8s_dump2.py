import json, sys
sys.stdout.reconfigure(encoding='utf-8', errors='replace')
c = json.load(open(r'C:\Users\DreadLord\Desktop\TR-W4-d37\tools\_d37_e8s_census.json'))['lanes']
for l in ('c9', 'd23', 'd34', 'd36', 'd41', 'g8', 'e26', 'd40', 'd43', 'd9', 'd11', 'f47', 'g5', 'd39', 'd42', 'c4', 'c10', 'c8', 'd125'):
    d = c[l]
    print('==', l, d['rev'], 'new', len(d['new']))
    for k, v in sorted(d['new'].items(), key=lambda x: x[1]):
        print('  %-28s proto %4d cs %s net %s' % (k, v, d['cs'].get(k), d['new_entries'].get(k)))
