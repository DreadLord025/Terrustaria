import subprocess, re
REPO = r"C:\Users\DreadLord\Desktop\TR-W4-d37"
def show(rev, path):
    return subprocess.run(["git","-C",REPO,"show",f"{rev}:{path}"],
                          capture_output=True).stdout.decode("utf8","replace")
src = show("a725acb0", "crates/tr-app/src/assets.rs")
m = re.search(r"STATIC_SHEET_NAMES[^=]*=\s*&?\[(.*?)\n\];", src, re.S)
body = re.sub(r"//[^\n]*", "", m.group(1))
allnames = re.findall(r'"([^"]+)"', body)
npc = [n for n in allnames if n.startswith("NPC_")]
print("base STATIC total:", len(allnames), "NPC_ rows:", len(npc))
# d42 tip slots?
s = show("9b22c091", "crates/tr-app/src/assets.rs")
mm = re.search(r"BESTIARY_NPC_SHEET_IDS:\s*\[u16;\s*(\d+)\]", s)
print("d42 slots decl:", mm.group(1))
# d33 slot adds
s = show("b08acf463", "crates/tr-app/src/assets.rs")
mm = re.search(r"BESTIARY_NPC_SHEET_IDS:\s*\[u16;\s*(\d+)\]\s*=\s*\[(.*?)\];", s, re.S)
ids = [int(x) for x in re.findall(r"\b(\d+)\b", re.sub(r"//[^\n]*","",mm.group(2)))]
print("d33 slots:", mm.group(1), "adds:", ids[181:])
# d40 ENTRIES count sanity: 38 rows but 37 gaps + TargetDummy no-entry
# g8's 17 owned slots present in its 34 adds? yes all 34.
