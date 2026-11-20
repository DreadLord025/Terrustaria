import re, os
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
def rd(p): return open(os.path.join(ROOT, p), encoding='utf-8').read()
npc = rd(r'crates\tr-app\src\npc.rs')

# find_frame: does it call find_frame_kind or have a fallback? Print head + first arms
i = npc.index('pub(crate) fn find_frame(')
print('--- find_frame first 1800 chars ---')
print(npc[i:i+1800])
