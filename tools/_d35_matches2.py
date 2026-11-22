import re, os, glob
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35\crates"
pat = re.compile(r'match\s+([^\{\n]+?)\s*\{')
report = []
for f in glob.glob(ROOT + r'\**\*.rs', recursive=True):
    src = open(f, encoding='utf-8').read()
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
        if body.count('NpcKind::') < 3: continue
        wild = re.search(r'^\s*(?:_|other|\w+)\s*(?:if [^=]+)?=>', body, re.M)
        if not wild:
            ln = src[:m.start()].count('\n')+1
            report.append(f"{os.path.relpath(f,ROOT)}:{ln}  scrutinee={m.group(1).strip()}")
print('\n'.join(report) if report else 'none')
