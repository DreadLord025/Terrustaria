import re, os
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
npc = open(os.path.join(ROOT, r'crates\tr-app\src\npc.rs'), encoding='utf-8').read()
i = npc.index('pub(crate) fn find_frame(')
j = npc.index('fn update_slime', i)
seg = npc[i:j]
print('find_frame len lines', seg.count('\n'))
for m in re.finditer(r'^\s*_\s*=>[^\n]*', seg, re.M):
    ln = npc[:i+m.start()].count('\n')+1
    print('wildcard at line', ln, ':', m.group(0))
print('--- find_frame tail ---')
print(seg[-1500:])
# deny attributes near find_frame / file head
print('deny lines:', re.findall(r'#!\[(?:deny|warn)[^\]]*\]', npc)[:5])
# knock_back_resist structure
i = npc.index('fn knock_back_resist(')
print('--- kb head ---'); print(npc[i:i+1500])
