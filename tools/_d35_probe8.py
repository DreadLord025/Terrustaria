import os, re
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
def rd(p): return open(os.path.join(ROOT, p), encoding='utf-8').read()
u = rd(r'crates\tr-app\src\ui\bestiary.rs')
lines = u.split('\n')
# portrait_data fn head
i = u.index('fn portrait_data')
print(u[i-600:i+1500])
print('=== wyvern arms in ui/bestiary ===')
for m in re.finditer(r'Wyvern\w*', u):
    ln = u[:m.start()].count('\n')+1
    print(ln, lines[ln-1].strip()[:110])
print('=== grid test ctx ===')
print('\n'.join(lines[1895:1935]))
print('=== pct test ctx ===')
print('\n'.join(lines[2295:2325]))
