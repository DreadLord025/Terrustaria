import re, os, sys
sys.path.insert(0, os.path.dirname(__file__))
from _d35_model import KINDS
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
s = open(os.path.join(ROOT, r'decompiled\Terraria.Localization.Content.en-US.NPCs.json'), encoding='utf-8').read()
for k in KINDS:
    nm = k['v']
    m = re.search(r'"%s"\s*:\s*"([^"]*)"' % nm, s)
    loc = m.group(1) if m else 'MISSING'
    flag = '' if loc == k['name'] else '  <-- model name=%r' % k['name']
    print(k['id'], nm, '=', repr(loc), flag)
# also credit names for folds
for nm in ['DiggerHead', 'SeekerHead', 'Corruptor', 'TheHungry']:
    m = re.search(r'"%s"\s*:\s*"([^"]*)"' % nm, s)
    print('credit', nm, '=', repr(m.group(1) if m else 'MISSING'))
