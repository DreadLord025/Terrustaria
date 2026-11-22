import re, os
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
def rd(p): return open(os.path.join(ROOT, p), encoding='utf-8').read()
npc = rd(r'crates\tr-app\src\npc.rs')
i = npc.index('fn s1_d_2_the_ai_style_table_matches_the_csharp_set_defaults')
j = npc.index('\n    }', i)
print(npc[i:j+6])
