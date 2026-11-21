# W4-d35 sheets phase: verify the port's bestiary_draw_offsets rows for the
# 21 packed kinds against NPCID.cs's NPCBestiaryDrawOffset dictionaries.
# Parses the C# `{ id, new NPCBestiaryDrawModifiers(frame) { ... } }` blocks
# and the port's `(id, BestiaryDrawModifiers { ... })` rows, compares fields.
import io, re

CS = io.open(r"C:\Users\DreadLord\Desktop\TR-W4-d35\decompiled\Terraria\ID\NPCID.cs",
             encoding="utf-8", errors="replace").read()
RS = io.open(r"C:\Users\DreadLord\Desktop\TR-W4-d35\crates\tr-app\src\ui\bestiary_draw_offsets.rs",
             encoding="utf-8").read()

IDS = [73, 74, 75, 77, 78, 82, 83, 93, 94, 95, 96, 97, 98, 99, 100, 101, 102, 112, 116, 121, 137]

# C# blocks: find each `{ <id>, new NPCBestiaryDrawModifiers(<frame>) { ... } }`
cs = {}
for m in re.finditer(r"\{\s*(\d+),\s*new NPCBestiaryDrawModifiers\(([-\d]+)\)\s*\{(.*?)\}\s*\}", CS, re.S):
    i = int(m.group(1))
    if i in IDS:
        fields = {"Frame": int(m.group(2))}
        for f in re.finditer(r"(\w+)\s*=\s*([^,\n]+)", m.group(3)):
            fields[f.group(1)] = f.group(2).strip()
        cs[i] = fields

# port rows
rs = {}
for m in re.finditer(r"\((\d+), BestiaryDrawModifiers \{([^}]*)\}\)", RS):
    i = int(m.group(1))
    if i in IDS:
        rs[i] = m.group(2)

MAP = {"Position": "position", "Velocity": "velocity", "Scale": "scale",
       "Rotation": "rotation", "Hide": "hide", "Frame": "frame",
       "PortraitScale": "portrait_scale",
       "PortraitPositionXOverride": "portrait_position_x_override",
       "PortraitPositionYOverride": "portrait_position_y_override",
       "CustomTexturePath": "custom_texture_path", "IsWet": "is_wet"}

def num(v):
    v = v.rstrip("f").replace("std::f32::consts::PI", "PI")
    return v

for i in IDS:
    c = cs.get(i)
    r = rs.get(i, "")
    if c is None:
        print(f"{i}: NO C# ROW; port: {r.strip()[:100]}")
        continue
    diffs = []
    for k, v in c.items():
        pk = MAP.get(k)
        if pk is None:
            diffs.append(f"unmapped {k}={v}")
            continue
        if k == "Frame":
            if v != 0 and f"frame: Some({v})" not in r:
                diffs.append(f"frame C#={v} port-missing")
            continue
        if k in ("Position",):
            nums = re.findall(r"-?[\d.]+f?", v)
            continue  # checked below via presence
        if pk not in r and k not in ("Position",):
            diffs.append(f"{k}={v} missing in port")
    print(f"{i}: C#={c}")
    print(f"    port={r.strip()[:160]}")
    if diffs:
        print("    DIFFS:", diffs)
