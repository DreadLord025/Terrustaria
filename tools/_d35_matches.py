import re, os, glob
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35\crates"
# find match expressions whose scrutinee looks like an NpcKind, then check for wildcard arm
pat = re.compile(r'match\s+((?:self\.)?kind|(?:\w+\.)?kind|self)\s*\{')
report = []
for f in glob.glob(ROOT + r'\**\*.rs', recursive=True):
    src = open(f, encoding='utf-8').read()
    lines = src.split('\n')
    for m in pat.finditer(src):
        start = m.end()-1
        depth = 0; i = start
        while i < len(src):
            if src[i] == '{': depth += 1
            elif src[i] == '}':
                depth -= 1
                if depth == 0: break
            i += 1
        body = src[start:i+1]
        ln = src[:m.start()].count('\n')+1
        has_npckind = 'NpcKind::' in body
        if not has_npckind: continue
        # wildcard or catch-all binding arm?
        wild = re.search(r'^\s*(?:_\s*=>|_ if .+=>|\w+ =>)', body, re.M)
        # count arms
        if not wild:
            rel = os.path.relpath(f, ROOT)
            report.append(f"{rel}:{ln}  scrutinee={m.group(1)}  EXHAUSTIVE? arms~{len(re.findall(chr(61)+chr(62), body))}")
print('\n'.join(report) if report else 'no exhaustive NpcKind matches found')
