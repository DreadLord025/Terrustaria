"""d38 crossnote scratch: structural checks on the edited files.
1. npc_rotation_allowed membership == the pre-edit inline matches! list.
2. brace/bracket/paren balance of each edited file vs its HEAD version.
3. BESTIARY_NPC_SHEET_IDS const len == 216, vec rows == 216, parallel order.
"""
import re, subprocess, pathlib
ROOT = pathlib.Path(__file__).resolve().parent.parent

def head(path):
    return subprocess.run(['git', 'show', f'HEAD:{path}'], cwd=ROOT,
                          capture_output=True, text=True, encoding='utf-8').stdout

def balance(s):
    # strip line comments and strings crudely (Rust: // and "...")
    s = re.sub(r'//[^\n]*', '', s)
    s = re.sub(r'"(?:[^"\\]|\\.)*"', '""', s)
    return {c: s.count(c) for c in '{}[]()'}

# 1. gate membership
old = head('crates/tr-app/src/render/entities.rs')
m = re.search(r'if n\.rotation != 0\.0\s*&& matches!\(\s*n\.kind,(.*?)\)\s*\{', old, re.S)
assert m, 'old inline gate not found'
old_kinds = re.findall(r'NpcKind::(\w+)', m.group(1))
new = (ROOT / 'crates/tr-app/src/render/entities.rs').read_text(encoding='utf-8')
m2 = re.search(r'pub\(crate\) fn npc_rotation_allowed\(kind: NpcKind\) -> bool \{\s*matches!\(\s*kind,(.*?)\)\s*\}', new, re.S)
assert m2, 'new gate fn not found'
new_kinds = re.findall(r'NpcKind::(\w+)', m2.group(1))
assert old_kinds == new_kinds, f'membership drift: {set(old_kinds) ^ set(new_kinds)}'
print(f'gate membership identical: {len(new_kinds)} kinds, order preserved')
assert 'if n.rotation != 0.0 && npc_rotation_allowed(n.kind) {' in new, 'draw call site'
assert new.count('fn npc_rotation_allowed') == 1

# 2. balance deltas
for p in ['crates/tr-app/src/render/entities.rs', 'crates/tr-app/src/assets.rs',
          'crates/tr-app/src/ui/bestiary.rs']:
    b_old, b_new = balance(head(p)), balance((ROOT / p).read_text(encoding='utf-8'))
    d = {c: b_new[c] - b_old[c] for c in b_old}
    paired = all(b_new[c] == b_new[c2] for c, c2 in [('{','}'),('[',']'),('(',')')])
    print(p, 'delta', d, 'balanced:', paired)
    assert paired

# 3. bestiary table
a = (ROOT / 'crates/tr-app/src/assets.rs').read_text(encoding='utf-8')
m3 = re.search(r'pub const BESTIARY_NPC_SHEET_IDS: \[u16; (\d+)\] = \[(.*?)\];', a, re.S)
assert m3
declared = int(m3.group(1))
ids = [int(x) for x in re.findall(r'\d+', re.sub(r'//[^\n]*', '', m3.group(2)))]
assert declared == len(ids) == 216, (declared, len(ids))
assert len(set(ids)) == 216, 'duplicate slot ids'
m4 = re.search(r'bestiary_npc_sheets: vec!\[(.*?)\n            \],', a, re.S)
rows = re.findall(r'atlas\.sheet\("([^"]+)"\)', m4.group(1))
assert len(rows) == 216, len(rows)
# parallel: last 35 rows are NPC_<id> in the same order as the const tail
tail_ids = ids[-35:]
tail_rows = rows[-35:]
assert tail_rows == [f'NPC_{i}' for i in tail_ids], 'parallel order drift'
# the first 181 rows unchanged vs HEAD
a_old = head('crates/tr-app/src/assets.rs')
m5 = re.search(r'bestiary_npc_sheets: vec!\[(.*?)\n            \],', a_old, re.S)
rows_old = re.findall(r'atlas\.sheet\("([^"]+)"\)', m5.group(1))
assert rows[:181] == rows_old, 'earlier slots drifted'
print(f'BESTIARY_NPC_SHEET_IDS: {declared} declared == {len(ids)} ids == {len(rows)} vec rows; first 181 unchanged; tail 35 parallel')
# ENTRIES order check
b = (ROOT / 'crates/tr-app/src/bestiary.rs').read_text(encoding='utf-8')
order = [int(x) for x in re.findall(r'net_id: (\d+),', b) if int(x) in set(tail_ids)]
assert order == tail_ids, f'ENTRIES order mismatch: {order}'
print('tail 35 in ENTRIES order: OK')
print('ALL STRUCTURAL CHECKS PASSED')
