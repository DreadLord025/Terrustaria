import json, sys
sys.stdout.reconfigure(encoding='utf-8', errors='replace')
c = json.load(open(r'C:\Users\DreadLord\Desktop\TR-W4-d37\tools\_d37_e8s_census.json'))['lanes']
for l in ('c7b', 'd103', 'e21', 'g62', 'd33', 'd36'):
    d = c[l]
    print('==', l, d['rev'])
    for k, v in sorted(d['new'].items(), key=lambda x: x[1]):
        print('  %-24s proto %4d cs %s net %s' % (k, v, d['cs'].get(k), d['new_entries'].get(k)))
