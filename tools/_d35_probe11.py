import re, os
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
def rd(p): return open(os.path.join(ROOT, p), encoding='utf-8').read()
cs = rd(r'decompiled\Terraria\ID\ContentSamples.cs')
i = cs.index('NpcBestiaryCreditIdsByNpcNetIds')
print('--- credit table build head ---')
print(cs[i-200:i+1400])
b = rd(r'crates\tr-app\src\bestiary.rs')
i = b.index('fn entry_for_npc'); j = b.index('\n}', i)
seg = b[i:j]
k = seg.find('Wyvern')
print('--- entry_for_npc wyvern ctx ---')
print(seg[max(0,k-800):k+500])
i = b.index('fn try_entry_for_npc'); j = b.index('\n}', i)
seg = b[i:j]
k = seg.find('Wyvern')
print('--- try_entry_for_npc wyvern ctx ---')
print(seg[max(0,k-800):k+500])
