# W4-d37 LAGGARD-EXEC: verify the ENTRIES regex captures every row shape.
import subprocess, re
REPO = r"C:\Users\DreadLord\Desktop\TR-W4-d37"
def show(rev, p):
    r = subprocess.run(["git", "-C", REPO, "show", rev + ":" + p], capture_output=True)
    return r.stdout.decode("utf8", "replace")
for rev in ["a725acb04", "origin/task/W4-d40-npc-kinds-421-490",
            "origin/task/W4-d35-npc-kinds-71-140"]:
    src = show(rev, "crates/tr-app/src/bestiary.rs")
    m = re.search(r"const ENTRIES[^=]*=\s*&?\[(.*?)\n\];", src, re.S)
    body = re.sub(r"//[^\n]*", "", m.group(1))
    n_credit = len(re.findall(r"credit_id:", body))
    strict = len(re.findall(
        r'credit_id:\s*"([^"]+)",\s*net_id:\s*(-?\d+),\s*kind:\s*NpcKind::(\w+)', body))
    print(rev.split("/")[-1], "credits", n_credit, "strict", strict)
    rows = re.split(r"BestiaryEntryDef \{", body)
    bad = [r[:150].replace("\n", " | ") for r in rows[1:]
           if not re.match(r'\s*credit_id:\s*"[^"]+",\s*net_id:\s*-?\d+,\s*kind:\s*NpcKind::\w+', r)]
    print("  non-strict:", len(bad), bad[:3])
