import re, subprocess, sys
sys.stdout.reconfigure(encoding='utf-8', errors='replace')
def show(rev, path):
    return subprocess.run(['git', '-C', r'C:\Users\DreadLord\Desktop\TR-W4-d37', 'show', rev + ':' + path], capture_output=True).stdout.decode('utf8', 'replace')
for rev, name in (('8abf0f1d8', 'c4'), ('31310b652', 'd43')):
    src = show(rev, 'crates/tr-app/src/npc.rs')
    i = src.find('fn update_swimmer'); j = src.find('fn ', i + 10); seg = src[i:j]
    print(name, 'swimmer just_hit puff:', 'just_hit' in seg, '| style16 module:', 'npc_ai_style16' in seg)
    print(name, 'swimmer lines:', len(seg.splitlines()))
# c4 style16 module exists? d43 style16 module?
import subprocess as sp
for rev, name in (('8abf0f1d8', 'c4'), ('31310b652', 'd43')):
    r = sp.run(['git', '-C', r'C:\Users\DreadLord\Desktop\TR-W4-d37', 'ls-tree', '--name-only', rev, 'crates/tr-app/src/'], capture_output=True)
    mods = [l for l in r.stdout.decode().splitlines() if 'style16' in l]
    print(name, 'style16 module:', mods)
