import json
d = json.load(open('g60_f10_overrun.json'))
over = [f for f in d['fns'] if f['min_cite'] >= 85560]
print(len(over))
for i, f in enumerate(over, 1):
    print(f"{i}|{f['name']}|{f['doc_start']}-{f['end']}|{f['min_cite']}-{f['max_cite']}")
