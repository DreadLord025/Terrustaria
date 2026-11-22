import json
c = json.load(open(r'C:\Users\DreadLord\Desktop\TR-W4-d37\tools\_d37_e8s_census.json'))['lanes']
for l in ('c3', 'c7', 'c9', 'e26', 'd120', 'd125', 'd33', 'd34', 'd35', 'd37', 'd38', 'd40', 'g8', 'c6'):
    d = c[l]
    print('==', l, d['rev'])
    for k, v in sorted(d['new'].items(), key=lambda x: x[1]):
        print('  %-28s proto %4d cs %s net %s' % (k, v, d['cs'].get(k), d['new_entries'].get(k)))
