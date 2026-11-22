import os
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
p = os.path.join(ROOT, 'tools', '_d35_model.py')
s = open(p, encoding='utf-8').read()
s = s.replace('def=', 'dfn=')
open(p, 'w', encoding='utf-8', newline='').write(s)
p2 = os.path.join(ROOT, 'tools', '_d35_gen.py')
s2 = open(p2, encoding='utf-8').read()
s2 = s2.replace("k['def']", "k['dfn']")
open(p2, 'w', encoding='utf-8', newline='').write(s2)
print('renamed def->dfn')
