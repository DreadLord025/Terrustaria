# W4-d37 LAGGARD-EXEC: FINAL v2 count pins, SEAL convention.
# ENTRIES = base numeric-positive-net distinct + surviving-owner owned rows.
# sheets/slots/cues = union over owner lanes (base included via every tip).
import re, subprocess, json

REPO = r"C:\Users\DreadLord\Desktop\TR-W4-d37"
def show(rev, p):
    r = subprocess.run(["git", "-C", REPO, "show", f"{rev}:{p}"], capture_output=True)
    return r.stdout.decode("utf8", "replace") if r.returncode == 0 else ""

def entry_rows(rev):
    src = show(rev, "crates/tr-app/src/bestiary.rs")
    m = re.search(r"const ENTRIES[^=]*=\s*&?\[(.*?)\n\];", src, re.S)
    if not m: return []
    body = re.sub(r"//[^\n]*", "", m.group(1)); rows = []
    for r in re.split(r"BestiaryEntryDef \{", body)[1:]:
        c = re.search(r'credit_id:\s*"([^"]+)"', r)
        n = re.search(r"net_id:\s*(-?\d+|crate::loot_db::\w+)", r)
        k = re.search(r"kind:\s*NpcKind::(\w+)", r)
        if c and n and k: rows.append((c.group(1), n.group(1), k.group(1)))
    return rows

# base numeric-positive-net distinct (SEAL's 241 convention)
base = entry_rows("a725acb04")
base_num = {(c, int(n)) for c, n, k in base if n.lstrip("-").isdigit() and int(n) > 0}
base_loot = {(c, n) for c, n, k in base if not n.lstrip("-").isdigit()}
print("base ENTRIES rows:", len(base), "numeric-positive distinct:", len(base_num),
      "loot_db-variant rows:", len(base_loot))

f2 = json.load(open(REPO + r"\tools\_d37_lag_final2.json"))
plan = f2["plan"]
p3 = json.load(open(REPO + r"\tools\_d37_lag_pins3.json"))
owned_entries = p3["entries"]
out = {
  "plan_kinds": len(plan),
  "all_kinds_to_from_s1f3": 270 + len(plan),
  "entries_base": len(base_num),
  "entries_owned": owned_entries,
  "entries_total": len(base_num) + owned_entries,
  "owned_no_entry": p3["owned_no_entry"],
  "sheets_npc_union": p3["sheets_union"],
  "slots_distinct": p3["slots_distinct"],
  "cues_npc": p3["cues"],
  "max_id": f2["max_id"],
  "moves_owed": f2["moves_total"],
  "drops_owed": f2["drops_total"],
}
json.dump(out, open(REPO + r"\tools\_d37_lag_pinsfinal.json", "w"), indent=1, sort_keys=True)
print(json.dumps(out, indent=1))
