import re, os, sys, json
sys.path.insert(0, os.path.dirname(__file__))
from _d35_model import KINDS
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
text = open(os.path.join(ROOT, r'decompiled\Terraria\NPC.cs'), encoding='utf-8').read()
lines = text.split('\n')

# Collect every `else if (this.type == ...)` block in SetDefaults and its body.
# Blocks: from the condition line to the next condition at same indent.
conds = []
for m in re.finditer(r'\n\t\telse if \((this\.type == [^\n]+)\)\n\t\t\{\n', text):
    conds.append((m.start(1), m.group(1), m.end()))
blocks = []
for idx, (pos, cond, bodystart) in enumerate(conds):
    bodyend = conds[idx+1][0] if idx+1 < len(conds) else bodystart+4000
    # cut at the closing brace before next cond
    body = text[bodystart:bodyend]
    ln = text[:pos].count('\n') + 1
    ids = [int(x) for x in re.findall(r'this\.type == (-?\d+)', cond)]
    blocks.append((ln, ids, cond, body))

def field(body, name, cast='int'):
    # top-level assignment (not nested inside if(type==X) sub-blocks) - take ALL and report
    pat = r'(?:this|base)\.%s = ([^;]+);' % name
    return re.findall(pat, body)

report = {}
bad = 0
for k in KINDS:
    cid = k['id']
    hit = [b for b in blocks if cid in b[1]]
    if not hit:
        print(cid, k['v'], 'NO BLOCK'); bad += 1; continue
    ln, ids, cond, body = hit[0]
    def one(name):
        vals = field(body, name)
        return vals
    out = {}
    checks = [
        ('w', 'width', str(k['w'])),
        ('h', 'height', str(k['h'])),
        ('ai', 'aiStyle', str(k['ai'])),
        ('dmg', 'damage', str(k['dmg'])),
        ('dfn', 'defense', str(k['dfn'])),
        ('life', 'lifeMax', str(k['life'])),
        ('alpha', 'alpha', str(k['alpha'])),
    ]
    msgs = []
    for key, csname, expect in checks:
        vals = one(csname)
        if expect == '0' and not vals:
            continue  # default 0
        if not vals:
            msgs.append(f'{csname}: NO ROW (model {expect})'); continue
        if expect not in [v.strip() for v in vals]:
            msgs.append(f'{csname}: C# {vals} vs model {expect}')
    # float fields
    for key, csname, expect in [('kb','knockBackResist',k['kb']),('value','value',k['value']),('scale','scale',k['scale'])]:
        vals = [v.strip().rstrip('f') for v in one(csname)]
        if expect is None:
            if vals: msgs.append(f'{csname}: C# {vals} but model None')
            continue
        exp = ('%g' % expect)
        got = ['%g' % float(v) for v in vals if re.match(r'^[\d.]+f?$', v)]
        if not got:
            if expect in (0, 0.0, 1.0) and key=='scale' and not vals: continue
            if key=='value' and expect==0 and not vals: continue
            if key=='kb' and expect==0.0 and not vals: continue
            msgs.append(f'{csname}: NO ROW (model {expect})')
        elif exp not in got:
            msgs.append(f'{csname}: C# {vals} vs model {expect}')
    # sounds
    hitsound = re.findall(r'this\.HitSound = SoundID\.NPCHit(\d+);', body)
    deathsound = re.findall(r'this\.DeathSound = SoundID\.NPCDeath(\d+);', body)
    nullhit = 'this.HitSound = null;' in body
    nulldeath = 'this.DeathSound = null;' in body
    if k['hit'] is None:
        if hitsound and not nullhit: msgs.append(f'HitSound: C# {hitsound} vs model None')
    else:
        if str(k['hit']) not in hitsound: msgs.append(f'HitSound: C# {hitsound} null={nullhit} vs model {k["hit"]}')
    if k['death'] is None:
        if deathsound and not nulldeath: msgs.append(f'DeathSound: C# {deathsound} null={nulldeath} vs model None')
    else:
        if str(k['death']) not in deathsound: msgs.append(f'DeathSound: C# {deathsound} vs model {k["death"]}')
    # flags
    for flag, key in [('noGravity','noGrav'),('noTileCollide','noTile'),('behindTiles','behind'),('netAlways','netAlways'),('dontTakeDamageFromHostiles','dontCount')]:
        present = ('this.%s = true;' % flag) in body
        if key=='dontCount': continue
        if present != k[key]:
            msgs.append(f'{flag}: C# {present} vs model {k[key]}')
    # rarity
    rar = re.findall(r'this\.rarity = (\d+);', body)
    exprar = k.get('rarity', 0)
    if rar:
        if str(exprar) not in rar: msgs.append(f'rarity: C# {rar} vs model {exprar}')
    elif exprar not in (0,1):
        msgs.append(f'rarity: NO ROW vs model {exprar}')
    # cite check
    cite_start = int(k['cite'].split('-')[0])
    if not (cite_start-2 <= ln <= cite_start+2):
        msgs.append(f'cite: block at {ln} vs model cite {k["cite"]}')
    report[cid] = {'line': ln, 'cond': cond, 'issues': msgs}
    if msgs:
        bad += 1
        print(cid, k['v'], '@', ln)
        for m_ in msgs: print('   ', m_)
    else:
        print(cid, k['v'], '@', ln, 'OK')
print('kinds with issues:', bad)
json.dump(report, open(os.path.join(ROOT, 'tools', '_d35_verbatim.json'), 'w'), indent=0)
