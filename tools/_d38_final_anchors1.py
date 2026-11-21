# W4-d38 FINAL - append the phase canary rows to anchors.txt (one-shot).
import io
rows = [
    'tools/_d38_final_lazy.js|L91: the d38 35 + d40 56 NPC-page candidate ALONE|W4-d38 FINAL sheet 1 - the lazy-load re-projection (research/125 s6 vs 113 s10 at e5558b0a9): E35 HEAD-minus-35 reproduces the 8115 pre-d38 baseline exactly, the 35+56 NPC-page slice packs 1554 px ONE page (6638 headroom), verify()+double-run on every layout',
    'docs/research/125-d38-crossnote-atlas-addendum.md|FINAL alignment: the lazy-load census|W4-d38 FINAL - research/125 flipped FINAL: the eager-form record stands (both-agree BAF withdrawal), the lazy contract re-partitions the input (d38 rows leave STATIC_SHEET_NAMES for the per-world NPC page, texture_net_id draw path unchanged)',
]
p = 'anchors.txt'
s = io.open(p, encoding='utf-8').read()
assert s.endswith('\n')
for r in rows:
    path, marker, _ = r.split('|', 2)
    body = io.open(path, encoding='utf-8').read()
    assert marker in body, f'marker missing: {marker}'
io.open(p, 'a', encoding='utf-8', newline='').write('\n'.join(rows) + '\n')
print('appended', len(rows), 'canary rows; markers verified present')
