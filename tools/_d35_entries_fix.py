# W4-d35 sheets phase: ENTRIES structural repair.
# The data-wave batch splices inserted each new batch's rows INSIDE the
# previous batch's last entry (before its `name:` + `},` tail), leaving the
# tails orphaned after PossessedArmor. This rewrites the W4-d35 block
# (GoblinScout .. the orphan tails) as well-formed entries in id order.
# Names come from the existing rows (the en-US NPCs.json values the data
# wave verified); nothing else in the file is touched.
import io, re

P = r"C:\Users\DreadLord\Desktop\TR-W4-d35\crates\tr-app\src\bestiary.rs"
src = io.open(P, encoding="utf-8").read()

start = src.index("    // W4-d35 (A): the 73/74/75/77/78 data-wave entries.")
endmark = '        name: "Mummy",\n    },\n];'
end = src.index(endmark) + len(endmark)
block = src[start:end]

# Parse every entry head (credit_id .. quick_unlock) and every name tail.
heads = re.findall(
    r'BestiaryEntryDef \{\n'
    r'        credit_id: "(\w+)",\n'
    r'        net_id: (\d+),\n'
    r'        kind: NpcKind::(\w+),\n'
    r'        variant: None,\n'
    r'        quick_unlock: false,\n', block)
# (the orphan tails' names are mapped via ORPHAN below)
# map credit_id -> name from the ORIGINAL well-formed pairs inside the block
pairs = re.findall(
    r'credit_id: "(\w+)",\n'
    r'        net_id: \d+,\n'
    r'        kind: NpcKind::\w+,\n'
    r'        variant: None,\n'
    r'        quick_unlock: false,\n'
    r'        name: "([^"]+)",', block)
name_by_credit = dict(pairs)
# the orphan tails at the end carry the names of the entries whose heads
# were split: Slimer/Clown/World Feeder(94 Corruptor? no - Corruptor has its
# own name inline)/Digger/Enchanted Sword/Mummy. Map them by net id:
ORPHAN = {"Slimer": "Slimer", "Clown": "Clown", "SeekerHead": "World Feeder",
          "DiggerHead": "Digger", "EnchantedSword": "Enchanted Sword",
          "Mummy": "Mummy"}
for credit, net, kind in heads:
    if credit not in name_by_credit and credit in ORPHAN:
        name_by_credit[credit] = ORPHAN[credit]

missing = [h for h in heads if h[0] not in name_by_credit]
assert not missing, f"no name for {missing}"
assert len(heads) == 28, f"expected 28 heads (33 - 5 Hide), got {len(heads)}"

out = ["    // W4-d35: the 33 data-wave kinds' entries (73..140), id order.\n"
       "    // The Hide kinds (96/97/99/100/112) have NO own row - their\n"
       "    // credit folds onto the head (try_entry_for_npc/entry_for_npc).\n"]
for credit, net, kind in sorted(heads, key=lambda h: int(h[1])):
    out.append(
        "    BestiaryEntryDef {\n"
        f'        credit_id: "{credit}",\n'
        f"        net_id: {net},\n"
        f"        kind: NpcKind::{kind},\n"
        "        variant: None,\n"
        "        quick_unlock: false,\n"
        f'        name: "{name_by_credit[credit]}",\n'
        "    },\n")
new = "".join(out) + "];"
src = src[:start] + new + src[end:]
io.open(P, "w", encoding="utf-8", newline="").write(src)
print("rewrote", len(heads), "entries; names:", name_by_credit)
