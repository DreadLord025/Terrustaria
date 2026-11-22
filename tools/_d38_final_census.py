# W4-d38 FINAL phase census - mechanical extraction for sheets 1/3/4.
# Reads ONLY direct file paths (no junction recursion): the Rust port files
# in this worktree + specific C# files under decompiled/ (git-show for the
# sibling tips happens in the shell, not here).
# Outputs tools/_d38_final_census.json consumed by the doc + witness work.
import json, re, subprocess, sys, io
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OUT = {}

# ---- the 35 kinds: name -> protocol id, from to_u16's d38 batch blocks ----
data_rs = (ROOT / 'crates/tr-app/src/npc/data.rs').read_text(encoding='utf-8')
def arms_of_fn(src, fn_sig):
    i = src.index(fn_sig)
    j = src.index('\n    }', i)  # fn bodies end at 4-space-indented close
    return src[i:j]
to_block = arms_of_fn(data_rs, 'pub fn to_u16')
kinds = {}
for m in re.finditer(r'// W4-d38 b\d[^\n]*\n((?:\s*NpcKind::\w+ => \d+,\n)+)', to_block):
    for k in re.finditer(r'NpcKind::(\w+) => (\d+),', m.group(1)):
        kinds[k.group(1)] = int(k.group(2))
assert len(kinds) == 35, f'expected 35 d38 kinds, got {len(kinds)}'
OUT['kinds'] = kinds

# protocol == vanilla check (sheet 4): cs_id arms; the S1-F-3 fallback
# resolves the un-handed kinds through the bestiary net ids (data.rs:3363-3370)
cs_block = arms_of_fn(data_rs, 'pub(crate) fn cs_id(self) -> u16')
cs = {}
for name in kinds:
    m = re.search(r'NpcKind::' + name + r' => (\d+)', cs_block)
    cs[name] = int(m.group(1)) if m else None
OUT['band'] = [min(kinds.values()), max(kinds.values())]
OUT['band_holes'] = sorted(set(range(min(kinds.values()), max(kinds.values()) + 1)) - set(kinds.values()))

# bestiary ENTRIES rows for the 35 kinds
best_rs = (ROOT / 'crates/tr-app/src/bestiary.rs').read_text(encoding='utf-8')
entries = []
for m in re.finditer(r'BestiaryEntryDef \{(.*?)\}', best_rs, re.S):
    b = m.group(1)
    km = re.search(r'kind: NpcKind::(\w+)', b)
    if km is None:
        continue  # struct decl / non-entry match
    k = km.group(1)
    if k in kinds:
        entries.append({
            'kind': k,
            'credit_id': re.search(r'credit_id: "(\w+)"', b).group(1),
            'net_id': int(re.search(r'net_id: (-?\d+)', b).group(1)),
            'quick_unlock': 'quick_unlock: true' in b,
        })
OUT['entries'] = entries
OUT['entries_count'] = len(entries)
net_by_kind = {e['kind']: e['net_id'] for e in entries}
# cs_id resolution: hand arm if present, else the bestiary net-id fallback
for name in kinds:
    if cs[name] is None and name in net_by_kind:
        cs[name] = net_by_kind[name]
OUT['cs_id'] = cs
OUT['cs_id_fallback_kinds'] = sorted(n for n in kinds
    if not re.search(r'NpcKind::' + n + r' => (\d+)', cs_block))
OUT['protocol_equals_vanilla'] = all(cs[n] == kinds[n] for n in kinds)

# ---- C# SetDefaults census: lifeMax / damage / rarity / boss per kind ----
npc_cs = (ROOT / 'decompiled/Terraria/NPC.cs').read_text(encoding='utf-8')
lines = npc_cs.split('\n')
SD_LO, SD_HI = 8426, 17859  # SetDefaults(int,...) :8426 .. getTenthAnniversary :17860
# chain links may be ORs (46||303||337||540) or ranges (>=305 && <=314)
starts = []
for i, ln in enumerate(lines):
    if not (SD_LO <= i + 1 <= SD_HI):
        continue
    if not re.match(r'\s*(?:else )?if \(.*this\.type', ln):
        continue
    ids = set(int(x) for x in re.findall(r'this\.type == (\d+)', ln))
    for a, b in re.findall(r'this\.type >= (\d+) && this\.type <= (\d+)', ln):
        ids |= set(range(int(a), int(b) + 1))
    if ids:
        starts.append((i, ids))
blocks = {}
for idx, (i, ids) in enumerate(starts):
    end = starts[idx + 1][0] if idx + 1 < len(starts) else SD_HI
    for tid in ids:
        blocks.setdefault(tid, []).append((i + 1, end, '\n'.join(lines[i:end])))
census = {}
for name, pid in kinds.items():
    sd = blocks.get(pid, [])
    assert sd, f'{name} ({pid}): no SetDefaults block'
    lo, hi = sd[0][0], sd[-1][1]
    body = '\n'.join(b for _, _, b in sd)
    def grab(field, default=None):
        m = re.search(r'this\.' + field + r' = ([\d.]+)f?;', body)
        return float(m.group(1)) if m else default
    life = grab('lifeMax'); dmg = grab('damage')
    if life is None:
        m = re.search(r'base\.lifeMax = (\d+)', body)
        life = float(m.group(1)) if m else None
    census[name] = {
        'id': pid, 'setdefaults_lines': [lo, hi],
        'lifeMax': life, 'damage': dmg, 'rarity': grab('rarity', 0.0),
        'boss': bool(re.search(r'this\.boss = true', body)),
        'townNPC': bool(re.search(r'this\.townNPC = true', body)),
        'catchItem': grab('catchItem', 0.0),
        'noGravity': bool(re.search(r'this\.noGravity = true', body)),
        'aiStyle': grab('aiStyle'),
    }
OUT['setdefaults'] = census
# CountsAsACritter (NPC.cs:6887-6897): lifeMax<=5 && damage==0 && type!=594 && type!=686
OUT['critters'] = [n for n, c in census.items()
                   if c['lifeMax'] is not None and c['lifeMax'] <= 5 and c['damage'] == 0
                   and c['id'] not in (594, 686)]
OUT['rare_spawns'] = {n: c['rarity'] for n, c in census.items() if c['rarity']}
OUT['bosses'] = [n for n, c in census.items() if c['boss']]

# ---- C# FindFrame case census (sheet 2): which of the 35 have a case ----
ff_start = next(i for i, ln in enumerate(lines) if ln.startswith('\tpublic void FindFrame()'))
# the switch: find "switch (this.type)" after ff_start
sw = next(i for i in range(ff_start, len(lines)) if 'switch (this.type)' in lines[i])
# switch ends where the post-switch tail begins - scan case labels to the
# method end (next method definition at one-tab indent)
ff_end = next(i for i in range(sw + 1, len(lines))
              if re.match(r'\t(public|private|protected|internal) ', lines[i]))
ff_lines = lines[sw:ff_end]
case_ids = {}
for i, ln in enumerate(ff_lines):
    m = re.match(r'\s*case (\d+):', ln)
    if m:
        case_ids.setdefault(int(m.group(1)), []).append(sw + i + 1)
OUT['findframe_switch_range'] = [sw + 1, ff_end]
OUT['findframe_cases'] = {n: case_ids.get(c['id'], []) for n, c in census.items()}
OUT['findframe_covered'] = sorted(n for n, v in OUT['findframe_cases'].items() if v)
OUT['findframe_nocase'] = sorted(n for n, v in OUT['findframe_cases'].items() if not v)

# npcFrameCount rows for the 35: the Main.cs array LITERAL (Main.cs:67082
# `Main.npcFrameCount = new int[697]`, 10 values per line from index 0)
main_cs = (ROOT / 'decompiled/Terraria/Main.cs').read_text(encoding='utf-8')
mlines = main_cs.split('\n')
fc_start = next(i for i, ln in enumerate(mlines)
                if 'Main.npcFrameCount = new int[' in ln)
vals, val_lines = [], []
for i in range(fc_start + 1, len(mlines)):
    ln = mlines[i].strip()
    if ln.startswith('};'):
        break
    row = [int(x) for x in re.findall(r'\d+', ln.rstrip(','))]
    vals.extend(row)
    val_lines.extend([i + 1] * len(row))
assert len(vals) == 697, f'npcFrameCount literal: {len(vals)} values'
fc = {}
for c in census.values():
    tid = c['id']
    fc[tid] = {'count': vals[tid], 'line': val_lines[tid]}
OUT['npcFrameCount'] = fc
OUT['npcFrameCount_literal_start'] = fc_start + 1

json.dump(OUT, open(ROOT / 'tools/_d38_final_census.json', 'w', encoding='utf-8'), indent=1)
print('35 kinds:', len(kinds), '| band:', OUT['band'], '| protocol==vanilla:', OUT['protocol_equals_vanilla'])
print('holes:', OUT['band_holes'])
print('entries:', len(entries), '| critters:', OUT['critters'])
print('rare:', OUT['rare_spawns'], '| bosses:', OUT['bosses'])
print('FindFrame covered (%d):' % len(OUT['findframe_covered']), OUT['findframe_covered'])
print('FindFrame NO case (%d):' % len(OUT['findframe_nocase']), OUT['findframe_nocase'])
print('frameCount rows found:', len(fc))
missing = [c['id'] for c in census.values() if c['id'] not in fc]
print('frameCount missing ids:', missing)
