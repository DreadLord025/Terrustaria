import re, os
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
def rd(p): return open(os.path.join(ROOT, p), encoding='utf-8').read()

def fn_body(src, sig):
    i = src.index(sig)
    m = re.search(r'\bmatch\b[^{;]*\{', src[i:i+4000])
    if not m: return None
    start = i + m.end() - 1
    depth = 0; j = start
    while j < len(src):
        c = src[j]
        if c == '{': depth += 1
        elif c == '}':
            depth -= 1
            if depth == 0: break
        j += 1
    return src[start+1:j]

checks = [
    (r'crates\tr-app\src\npc.rs', 'fn defense(', 'defense'),
    (r'crates\tr-app\src\npc.rs', 'fn value(', 'value'),
    (r'crates\tr-app\src\npc.rs', 'fn knock_back_resist(', 'kb'),
    (r'crates\tr-app\src\npc.rs', 'pub fn render_key(&self)', 'render_key'),
    (r'crates\tr-app\src\npc.rs', 'fn update_kind_ai(', 'update_kind_ai'),
    (r'crates\tr-app\src\npc.rs', 'pub(crate) fn find_frame(', 'find_frame'),
    (r'crates\tr-app\src\npc\data.rs', 'fn ai_style(', 'ai_style'),
    (r'crates\tr-app\src\npc\data.rs', 'fn size(', 'size'),
    (r'crates\tr-app\src\npc\data.rs', 'fn max_hp(', 'max_hp'),
    (r'crates\tr-app\src\npc\data.rs', 'fn contact_damage(', 'contact_damage'),
    (r'crates\tr-app\src\npc_buffs.rs', 'fn debuff_immune_ids(', 'debuff'),
    (r'crates\tr-app\src\ui\bestiary.rs', 'fn portrait_data(', 'portrait'),
    (r'crates\tr-app\src\app\netglue.rs', 'fn npc_seed(', 'npc_seed'),
]
for path, sig, name in checks:
    src = rd(path)
    body = fn_body(src, sig)
    if body is None:
        print(name, 'NO MATCH FOUND'); continue
    wilds = re.findall(r'^\s*(_|[a-z]\w*)\s*(?:if [^\n]*)?=>[^\n]*', body, re.M)
    print(name, 'arms~', body.count('=>'), 'wildcards:', wilds[:4])
