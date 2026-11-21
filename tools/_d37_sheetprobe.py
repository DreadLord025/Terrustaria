import subprocess, re
REPO = r"C:\Users\DreadLord\Desktop\TR-W4-d37"
def show(rev, path):
    return subprocess.run(["git","-C",REPO,"show",f"{rev}:{path}"],
                          capture_output=True).stdout.decode("utf8","replace")
for lane, rev, names in [
    ("c7","c2cc6ec6",["NPC_355","NPC_358","NPC_654","NPC_677"]),
    ("c8","3705670d",["NPC_357","NPC_374","NPC_375"]),
    ("d120","c5e68a80",["NPC_142","NPC_160","NPC_178","NPC_209"]),
    ("c4","17a241f6a",["NPC_509","NPC_157","NPC_465","NPC_688","NPC_692"]),
]:
    ent = show(rev, "crates/tr-app/src/render/entities.rs")
    ast = show(rev, "crates/tr-app/src/assets.rs")
    for n in names:
        in_ent = n in ent
        in_ast = n in ast
        print(f"{lane:5s} {n:9s} entities.rs:{in_ent} assets.rs:{in_ast}")
# what sheet-chain consts does c7 add?
ent = show("c2cc6ec6", "crates/tr-app/src/render/entities.rs")
print("c7 chain consts:", re.findall(r"\.chain\((\w+)\)", ent))
ent = show("3705670d", "crates/tr-app/src/render/entities.rs")
print("c8 chain consts:", re.findall(r"\.chain\((\w+)\)", ent))
