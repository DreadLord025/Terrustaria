import json, glob, re

trail = re.compile(r',\s*([}\]])')
files = {}
for f in sorted(glob.glob('decompiled/Terraria.Localization.Content.en-US*.json')):
    j = json.loads(trail.sub(r'\1', open(f, encoding='utf-8-sig').read()))
    files[f] = j

for cat in ('BartenderChatter', 'GolferChatter', 'PrincessChatter', 'BestiaryGirlChatter', 'AnglerChatter'):
    for f, j in files.items():
        if cat in j:
            print(cat, '->', f.split('.')[-2], sorted(j[cat].keys()))

# keys whose NAME contains a dot after the category join -> category parse check
# also show any category with 'Chatter' in name
print()
for f, j in files.items():
    for cat in j:
        if 'Chatter' in cat:
            print(f.split('Content.')[1], cat, len(j[cat]))
