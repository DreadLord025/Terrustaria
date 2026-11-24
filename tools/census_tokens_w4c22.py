import json, glob, re, collections, sys

pat = re.compile(r'\{([?!]?[A-Za-z][\w\.]*)\}')
trail = re.compile(r',\s*([}\]])')
tot = 0
toks = collections.Counter()
cats = collections.Counter()
samples = {}
rows = []
for f in sorted(glob.glob('decompiled/Terraria.Localization.Content.en-US*.json')):
    raw = open(f, encoding='utf-8-sig').read()
    j = json.loads(trail.sub(r'\1', raw))
    for cat, vals in j.items():
        for k, v in vals.items():
            if isinstance(v, str):
                ms = pat.findall(v)
                if ms:
                    tot += 1
                    cats[cat] += 1
                    rows.append((cat + '.' + k, ','.join(ms), v))
                    for m in ms:
                        toks[m] += 1
                        samples.setdefault(m, (cat + '.' + k, v))
print('TOTAL VALUES:', tot)
print('CATEGORIES:', dict(cats))
print()
for t, c in toks.most_common():
    print('%-42s %4d  e.g. %s: %s' % (t, c, samples[t][0], samples[t][1][:80]))
if len(sys.argv) > 1 and sys.argv[1] == 'rows':
    print()
    for r in rows:
        print(r[0], '|', r[1])
