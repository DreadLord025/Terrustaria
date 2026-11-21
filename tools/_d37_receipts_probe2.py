import subprocess, re, json
REPO = r"C:\Users\DreadLord\Desktop\TR-W4-d37"
def show(rev, path):
    return subprocess.run(["git","-C",REPO,"show",f"{rev}:{path}"],
                          capture_output=True).stdout.decode("utf8","replace")
# c9 ALL_KINDS vs to_u16 set
src = show("2042d77f", "crates/tr-app/src/npc.rs")
m = re.search(r"const ALL_KINDS:\s*\[NpcKind;\s*(\d+)\]\s*=\s*\[(.*?)\];", src, re.S)
ak = set(re.findall(r"NpcKind::(\w+)", m.group(2)))
d = show("2042d77f", "crates/tr-app/src/npc/data.rs")
i = d.index("pub fn to_u16"); j = d.index("{", i)
body = d[j:d.index("pub fn from_u16")]
to = set(re.findall(r"NpcKind::(\w+)\s*=>", body))
print("c9 ALL_KINDS decl", m.group(1), "len", len(ak))
print("c9 kinds in to_u16 but MISSING from ALL_KINDS:", sorted(to - ak))
print("c9 in ALL_KINDS but not to_u16:", sorted(ak - to))
# d39 dirty proto assignments (read-only diff)
r = subprocess.run(["git","-C",r"C:\Users\DreadLord\Desktop\TR-W4-d39","diff","--","crates/tr-app/src/npc/data.rs"],capture_output=True).stdout.decode("utf8","replace")
print("\nd39 dirty to_u16 adds:")
for line in r.splitlines():
    if re.match(r"\+\s*(\| )?NpcKind::\w+ => \d+,", line): print(" ", line.strip())
# NPCID 694 / 688 / 692 identity (direct file, no recursion)
nid = open(r"C:\Users\DreadLord\Desktop\TR-W4-d37\decompiled\Terraria\ID\NPCID.cs", encoding="utf8", errors="replace").read()
for v in (688, 692, 694, 636, 661):
    mm = re.search(r"public const ushort (\w+) = %d;" % v, nid)
    print(f"NPCID {v}:", mm.group(1) if mm else "NONE")
