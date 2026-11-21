# W4-d35 ATLAS FINAL phase: the NPC-sheet lazy-load census inputs.
# Extracts the port's SPAWNABLE kind set (what a realistic world actually
# spawns -> the per-world NPC working set) from the landed code, WITHOUT
# running Rust:
#   1. NpcKind variant names (npc.rs enum body).
#   2. The kinds the ambient spawner can pick (spawner.rs production code,
#      `NpcKind::X` mentions before the test module).
#   3. The moon-roster ladder kinds (npc/data.rs pumpkin/frost_moon_type).
#   4. kind -> vanilla net id via bestiary.rs ENTRIES rows (the net_id()
#      path) + entities.rs texture_net_id overrides (the Hide-kind own-sheet
#      precedent).
# Writes _d35_npc_census.json {spawnKinds, nets, sheets, notes}; the .js
# measurement consumes it. Pure text extraction, mirrors what the code does.
import json, os, re

HERE = os.path.dirname(os.path.abspath(__file__))
SRC = os.path.join(HERE, "..", "crates", "tr-app", "src")

def read(p):
    return open(os.path.join(SRC, p), encoding="utf-8").read()

npc_src = read("npc.rs")
spawner_src = read("spawner.rs")
data_src = read(os.path.join("npc", "data.rs"))
best_src = read("bestiary.rs")
ent_src = read(os.path.join("render", "entities.rs"))

# 1. the enum variants (npc.rs `pub enum NpcKind {` .. closing brace at col 0)
m = re.search(r"pub enum NpcKind \{", npc_src)
body = npc_src[m.end():]
depth = 1
end = 0
for i, ch in enumerate(body):
    if ch == "{": depth += 1
    elif ch == "}":
        depth -= 1
        if depth == 0:
            end = i
            break
enum_body = body[:end]
variants = re.findall(r"^\s{4}([A-Z][A-Za-z0-9]+),", enum_body, re.M)
print("NpcKind variants:", len(variants))

# 2. spawner production mentions (cut at the test module). The world_sim
# spawn sites (invasions, dd2 waves, boss summons, shimmer release) join:
# same cut rule. This is the superset a long-lived world can spawn.
def prod_mentions(src):
    cut = src.find("#[cfg(test)]")
    prod = src[:cut] if cut > 0 else src
    return set(re.findall(r"NpcKind::([A-Z][A-Za-z0-9]+)", prod)) & set(variants)

sim_src = read(os.path.join("world_sim", "npcs.rs"))
ambient_kinds = sorted(prod_mentions(spawner_src))
sim_kinds = sorted(prod_mentions(sim_src))
spawn_kinds = sorted(set(ambient_kinds) | set(sim_kinds))
print("spawner production kinds:", len(ambient_kinds),
      "| world_sim spawn-site kinds:", len(sim_kinds),
      "| union:", len(spawn_kinds))

# 3. moon ladders: the *_moon_defaults arms (kind => vanilla_type). The
# spawned sheet id IS the vanilla_type (moon_wave_kind maps it back).
moon_net = {}
for fn in ("pumpkin_moon_defaults", "frost_moon_defaults"):
    m = re.search(r"pub fn " + fn + r"\(self\)", data_src)
    if not m:
        print("WARN: no", fn)
        continue
    seg = data_src[m.end():]
    depth = 0; end = None
    for i, ch in enumerate(seg):
        if ch == "{": depth += 1
        elif ch == "}":
            depth -= 1
            if depth == 0:
                end = i
                break
    for arm in re.finditer(r"NpcKind::([A-Z][A-Za-z0-9]+)\s*=>\s*\w+Defaults\s*\{\s*vanilla_type:\s*(\d+)", seg[:end]):
        if arm.group(1) in variants:
            moon_net[arm.group(1)] = int(arm.group(2))
print("moon ladder kinds:", len(moon_net))

# 4a. bestiary ENTRIES rows: `NpcKind::X => &BestiaryEntryDef { ... net_id: N`
# entry_for_npc's fold map (kind -> entry kind) parsed from its match arms.
fold = {}
m = re.search(r"pub fn entry_for_npc\(kind: NpcKind", best_src)
seg = best_src[m.end():]
depth = 0; end = None
for i, ch in enumerate(seg):
    if ch == "{": depth += 1
    elif ch == "}":
        depth -= 1
        if depth == 0:
            end = i
            break
fold_body = seg[:end]
for arm in re.finditer(r"((?:NpcKind::[A-Z][A-Za-z0-9]+\s*\|?\s*)+)\s*=>\s*(?:\{\s*)?NpcKind::([A-Z][A-Za-z0-9]+)", fold_body):
    for k in re.findall(r"NpcKind::([A-Z][A-Za-z0-9]+)", arm.group(1)):
        fold[k] = arm.group(2)
print("entry_for_npc fold arms:", len(fold))

# ENTRIES table: kind label + net_id (numeric or loot_db const)
loot_src = read("loot_db.rs")
loot_consts = dict(re.findall(r"pub const ([A-Z_0-9]+): (?:u16|i32) = (-?\d+)", loot_src))
entries = {}
m = re.search(r"pub const ENTRIES", best_src)
seg = best_src[m.end():]
# rows: BestiaryEntryDef { credit_id, net_id, kind: NpcKind::X, ... }
for row in re.finditer(r"BestiaryEntryDef\s*\{(.{0,400}?)\}", seg, re.S):
    body_r = row.group(1)
    km = re.search(r"kind:\s*NpcKind::([A-Z][A-Za-z0-9]+)", body_r)
    nm = re.search(r"net_id:\s*(?:crate::loot_db::)?([A-Z_0-9]+|-?\d+)", body_r)
    if not km or not nm:
        continue
    tok = nm.group(1)
    net = int(tok) if tok.lstrip("-").isdigit() else (int(loot_consts[tok]) if tok in loot_consts else None)
    # negative net ids are the slime variants: their DRAW sheet is NPC_1
    # (the variant is scale+color on the shared strip, Lang.cs:520-533).
    if net is not None and km.group(1) not in entries:
        entries[km.group(1)] = net if net > 0 else 1
print("ENTRIES kind rows with net ids:", len(entries))

# 4b. texture_net_id overrides (entities.rs)
m = re.search(r"pub\(crate\) fn texture_net_id\(kind: NpcKind\)", ent_src)
seg = ent_src[m.end():]
depth = 0; end = None
for i, ch in enumerate(seg):
    if ch == "{": depth += 1
    elif ch == "}":
        depth -= 1
        if depth == 0:
            end = i
            break
tex_over = dict((k, int(v)) for k, v in re.findall(r"NpcKind::([A-Z][A-Za-z0-9]+)\s*=>\s*(\d+)", seg[:end]))
print("texture_net_id overrides:", len(tex_over))

def sheet_net(kind):
    if kind in tex_over:
        return tex_over[kind]
    ek = fold.get(kind, kind)
    return entries.get(ek)

unresolved = []
spawn_nets = {}
for k in spawn_kinds:
    n = moon_net.get(k) or sheet_net(k)
    if n is None:
        unresolved.append(k)
    else:
        spawn_nets[k] = n
ambient_nets = {k: (moon_net.get(k) or sheet_net(k)) for k in ambient_kinds
                if (moon_net.get(k) or sheet_net(k)) is not None}
moon_nets = dict(moon_net)

# town roster: the vanilla ids whose SetDefaults arm writes townNPC = true
# (NPC.cs). Walk the case labels: each `this.townNPC = true;` belongs to
# the nearest preceding `case N:` in the same switch.
npc_cs = open(os.path.join(HERE, "..", "decompiled", "Terraria", "NPC.cs"),
              encoding="utf-8", errors="replace").read()
lines = npc_cs.split("\n")
town_nets = set()
cur_type = None
for ln in lines:
    # only the `this.type == N [|| this.type == M ...]` shape (no >=/&&
    # range arms — those set OTHER kinds' flags, not the town roster)
    cm = re.search(r"(?:else )?if \((this\.type == -?\d+(?:\s*\|\|\s*this\.type == -?\d+)*)\)", ln)
    if cm:
        cur_type = [int(t) for t in re.findall(r"this\.type == (-?\d+)", cm.group(1))]
    elif re.search(r"(?:else )?if \(this\.type", ln):
        cur_type = None
    if "this.townNPC = true" in ln and cur_type is not None:
        town_nets.update(cur_type)
town_nets = sorted(n for n in town_nets if n > 0)
print("townNPC=true vanilla ids:", len(town_nets), town_nets)

out = {
    "variants": len(variants),
    "spawnKinds": spawn_kinds,
    "spawnNets": spawn_nets,
    "ambientKinds": ambient_kinds,
    "ambientNets": ambient_nets,
    "moonNets": moon_nets,
    "unresolvedSpawnKinds": unresolved,
    "townNets": town_nets,
    "entryNets": sorted(set(entries.values())),
    "texOverrideNets": sorted(set(tex_over.values())),
    # Main.cs LoadNPC call sites outside DrawNPCDirect (the cross-draws):
    # 403/404 (:24278-79 stardust), 492 (:24683 empress), 391 (:25227
    # bestiary scutlix), 136 (:25509 mech-queen), 139 (:31967 probe proj),
    # 4 (:33179 eye proj), 594 (:34410 lunatic proj), 550/199 (credits).
    "drawExtraNets": [403, 404, 492, 391, 136, 139, 4, 594, 550, 199],
    "notes": [
        "spawnKinds = NpcKind mentions in spawner.rs production code (pre-#[cfg(test)])",
        "sheet id = texture_net_id override else bestiary entry_for_npc fold + ENTRIES net_id",
        "townNets = the C# townNPC=true roster ids (NPC.cs SetDefaults townNPC writes)",
    ],
}
with open(os.path.join(HERE, "_d35_npc_census.json"), "w") as f:
    json.dump(out, f, indent=1)
print("spawn sheets:", len(set(spawn_nets.values())), "distinct nets;",
      "moon adds:", len(set(moon_nets.values()) - set(spawn_nets.values())))
print("unresolved:", unresolved)
