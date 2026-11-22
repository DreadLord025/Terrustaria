import re, os
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
SRC = os.path.join(ROOT, 'crates')

def match_blocks(text):
    # yield (start_index, body) for each `match ... {` with brace balancing
    for m in re.finditer(r'\bmatch\b[^{;]*\{', text):
        i = m.end() - 1
        depth = 0; j = i
        while j < len(text):
            c = text[j]
            if c == '{': depth += 1
            elif c == '}':
                depth -= 1
                if depth == 0: break
            j += 1
        yield m.start(), text[i+1:j]

results = []
for dirpath, dirnames, filenames in os.walk(SRC):
    dirnames[:] = [d for d in dirnames if d not in ('target', '.git')]
    for fn in filenames:
        if not fn.endswith('.rs'): continue
        p = os.path.join(dirpath, fn)
        text = open(p, encoding='utf-8').read()
        for start, body in match_blocks(text):
            if 'NpcKind::' not in body: continue
            # catch-all: `_ =>`, `_ if`, binding ident =>, or `..`
            arms = re.findall(r'^\s*([^\n=]*?)=>', body, re.M)
            has_wild = any(re.match(r'^\s*(_|_ .*|[a-z_]\w*)\s*$', a) and 'NpcKind' not in a and '::' not in a and '"' not in a and not a.strip().startswith('(') for a in arms)
            # also tuple wildcards like (NpcKind::X, _) are not catch-all; plain (_, _) is
            if not has_wild:
                has_wild = any(re.match(r'^\s*\(?_\s*(,\s*_)*\)?\s*$', a) for a in arms)
            if not has_wild:
                ln = text[:start].count('\n') + 1
                # find enclosing fn
                fns = list(re.finditer(r'fn (\w+)', text[:start]))
                fname = fns[-1].group(1) if fns else '?'
                results.append((os.path.relpath(p, ROOT), ln, fname, len(arms)))
for r in sorted(results):
    print(r)
print('total exhaustive NpcKind matches:', len(results))
