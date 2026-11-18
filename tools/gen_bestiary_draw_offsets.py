#!/usr/bin/env python3
"""Generate crates/tr-app/src/ui/bestiary_draw_offsets.rs from the decompiled
`NPCID.Sets.NPCBestiaryDrawOffset` dictionaries (Terraria 1.4.5.8).

Source of truth: decompiled/Terraria/ID/NPCID.cs
  * struct NPCBestiaryDrawModifiers defaults  (:15-42)
  * NPCBestiaryDrawOffsetCreation merge order (:393-412): grox, then leinfors,
    then redigit — later dictionaries overwrite earlier keys.
  * GetRedigitEntries (:414-770), GetGroxEntries (:772-786),
    GetLeinforsEntries (:788-...).

Run from the repo root:  python tools/gen_bestiary_draw_offsets.py
The output is a cited static table; never hand-edit it.
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SRC = ROOT / "decompiled" / "Terraria" / "ID" / "NPCID.cs"
OUT = ROOT / "crates" / "tr-app" / "src" / "ui" / "bestiary_draw_offsets.rs"

DEFAULTS = {
    "Position": "[0.0, 0.0]",
    "PortraitPositionXOverride": "None",
    "PortraitPositionYOverride": "None",
    "Rotation": "0.0",
    "Scale": "1.0",
    "PortraitScale": "Some(1.0)",  # the C# field default is `1f`, not null (:27)
    "Hide": "false",
    "IsWet": "false",
    "Frame": "None",
    "Direction": "None",
    "SpriteDirection": "None",
    "Velocity": "0.0",
    "CustomTexturePath": "None",
}
FIELD = {
    "Position": "position",
    "PortraitPositionXOverride": "portrait_position_x_override",
    "PortraitPositionYOverride": "portrait_position_y_override",
    "Rotation": "rotation",
    "Scale": "scale",
    "PortraitScale": "portrait_scale",
    "Hide": "hide",
    "IsWet": "is_wet",
    "Frame": "frame",
    "Direction": "direction",
    "SpriteDirection": "sprite_direction",
    "Velocity": "velocity",
    "CustomTexturePath": "custom_texture_path",
}
NULLABLE_FLOAT = {"PortraitPositionXOverride", "PortraitPositionYOverride", "PortraitScale"}
NULLABLE_INT = {"Frame", "Direction", "SpriteDirection"}


def func_body(text, name):
    """The brace-balanced body of `private static ... <name>()`."""
    m = re.search(r"private static Dictionary<int, NPCBestiaryDrawModifiers> " + name + r"\(\)", text)
    if not m:
        sys.exit(f"missing {name}")
    i = text.index("{", m.end())
    depth, j = 0, i
    while True:
        if text[j] == "{":
            depth += 1
        elif text[j] == "}":
            depth -= 1
            if depth == 0:
                break
        j += 1
    start_line = text.count("\n", 0, m.start()) + 1
    end_line = text.count("\n", 0, j) + 1
    return text[i : j + 1], start_line, end_line


def fnum(s):
    s = s.strip().rstrip(",").strip()
    s = s.replace("(float)Math.PI", "std::f32::consts::PI")
    s = re.sub(r"(\d+(?:\.\d+)?)f\b", r"\1", s)
    s = re.sub(r"\bMath\.PI\b", "std::f32::consts::PI", s)
    # every remaining bare integer literal in the expression is a float
    # operand (e.g. `(float)Math.PI * 3f / 4f` -> `PI * 3.0 / 4.0`)
    s = re.sub(r"(?<![\w.])(\d+)(?![\w.])", r"\1.0", s)
    return s


def parse_value(key, raw):
    raw = raw.strip().rstrip(",").strip()
    if key == "Position":
        m = re.fullmatch(r"new Vector2\(([^,]+),\s*([^)]+)\)", raw)
        if not m:
            sys.exit(f"bad Position: {raw}")
        return f"[{fnum(m.group(1))}, {fnum(m.group(2))}]"
    if key == "CustomTexturePath":
        m = re.fullmatch(r'"([^"]+)"', raw)
        if not m:
            sys.exit(f"bad CustomTexturePath: {raw}")
        return f'Some("{m.group(1)}")'
    if key in ("Hide", "IsWet"):
        return raw  # true / false
    if key in NULLABLE_INT:
        return f"Some({raw.rstrip('f')})"
    if key in NULLABLE_FLOAT:
        return f"Some({fnum(raw)})"
    return fnum(raw)


def parse_entries(body):
    """Yield (id, {field: rust_value}) for each dictionary initializer."""
    pat = re.compile(
        r"\{\s*(-?\d+),\s*new NPCBestiaryDrawModifiers\(0\)\s*(\{[^{}]*\})?\s*\}",
        re.S,
    )
    for m in pat.finditer(body):
        npc_id = int(m.group(1))
        fields = {}
        if m.group(2):
            for line in m.group(2)[1:-1].splitlines():
                line = line.strip().rstrip(",").strip()
                if not line:
                    continue
                km = re.match(r"(\w+)\s*=\s*(.+)$", line)
                if not km:
                    sys.exit(f"bad field line: {line!r}")
                key, raw = km.group(1), km.group(2)
                if key not in FIELD:
                    sys.exit(f"unknown field {key}")
                fields[key] = parse_value(key, raw)
        yield npc_id, fields


def main():
    text = SRC.read_text(encoding="utf-8-sig")
    merged = {}
    cites = []
    # NPCBestiaryDrawOffsetCreation (:393-412): grox first, redigit last wins.
    for name in ("GetGroxEntries", "GetLeinforsEntries", "GetRedigitEntries"):
        body, a, b = func_body(text, name)
        n = 0
        for npc_id, fields in parse_entries(body):
            merged[npc_id] = fields
            n += 1
        cites.append(f"//!   * {name} — NPCID.cs:{a}-{b} ({n} rows)")
    rows = []
    for npc_id in sorted(merged):
        fields = merged[npc_id]
        parts = [
            f"{FIELD[k]}: {v}"
            for k, v in fields.items()
            if v != DEFAULTS[k]
        ]
        if parts:
            lit = "BestiaryDrawModifiers { " + ", ".join(parts) + ", ..DEFAULT }"
        else:
            lit = "DEFAULT"
        rows.append(f"    ({npc_id}, {lit}),")
    out = (
        "//! GENERATED by tools/gen_bestiary_draw_offsets.py — do not hand-edit.\n"
        "//!\n"
        "//! `NPCID.Sets.NPCBestiaryDrawOffset` (Terraria 1.4.5.8): the per-NPC\n"
        "//! bestiary icon draw modifiers, applied by `UnlockableNPCEntryIcon.Update`\n"
        "//! (`UnlockableNPCEntryIcon.cs:52-82`). Merge order follows\n"
        "//! `NPCBestiaryDrawOffsetCreation` (`NPCID.cs:393-412`): grox, leinfors,\n"
        "//! redigit — later dictionaries overwrite earlier keys. Rows sorted by id.\n"
        "//!\n" + "\n".join(cites) + "\n\n"
        "/// `NPCID.Sets.NPCBestiaryDrawModifiers` (`NPCID.cs:15-42`) — the field\n"
        "/// defaults are the C# struct's own initializers (`PortraitScale` defaults\n"
        "/// to `Some(1.0)`, the nullable-with-`1f` field at `:27`).\n"
        "#[derive(Clone, Copy, Debug, PartialEq)]\n"
        "pub struct BestiaryDrawModifiers {\n"
        "    pub position: [f32; 2],\n"
        "    pub portrait_position_x_override: Option<f32>,\n"
        "    pub portrait_position_y_override: Option<f32>,\n"
        "    pub rotation: f32,\n"
        "    pub scale: f32,\n"
        "    pub portrait_scale: Option<f32>,\n"
        "    pub hide: bool,\n"
        "    pub is_wet: bool,\n"
        "    pub frame: Option<i32>,\n"
        "    pub direction: Option<i32>,\n"
        "    pub sprite_direction: Option<i32>,\n"
        "    pub velocity: f32,\n"
        "    pub custom_texture_path: Option<&'static str>,\n"
        "}\n\n"
        "/// The struct's default instance (`NPCID.cs:17-41`).\n"
        "pub const DEFAULT: BestiaryDrawModifiers = BestiaryDrawModifiers {\n"
        "    position: [0.0, 0.0],\n"
        "    portrait_position_x_override: None,\n"
        "    portrait_position_y_override: None,\n"
        "    rotation: 0.0,\n"
        "    scale: 1.0,\n"
        "    portrait_scale: Some(1.0),\n"
        "    hide: false,\n"
        "    is_wet: false,\n"
        "    frame: None,\n"
        "    direction: None,\n"
        "    sprite_direction: None,\n"
        "    velocity: 0.0,\n"
        "    custom_texture_path: None,\n"
        "};\n\n"
        "/// The merged `NPCBestiaryDrawOffset` dictionary, sorted by NPC net id.\n"
        f"pub static DRAW_OFFSETS: &[(i32, BestiaryDrawModifiers)] = &[\n"
        + "\n".join(rows)
        + "\n];\n\n"
        "/// `NPCID.Sets.NPCBestiaryDrawOffset.TryGetValue` (`UnlockableNPCEntryIcon.cs:52`).\n"
        "pub fn get(net_id: i32) -> Option<&'static BestiaryDrawModifiers> {\n"
        "    DRAW_OFFSETS\n"
        "        .binary_search_by_key(&net_id, |(id, _)| *id)\n"
        "        .ok()\n"
        "        .map(|i| &DRAW_OFFSETS[i].1)\n"
        "}\n\n"
        + no_gravity_table()
    )
    OUT.write_text(out, encoding="utf-8", newline="\n")
    print(f"wrote {OUT} ({len(rows)} rows)")


def no_gravity_table():
    """The `NPC.noGravity` set from `NPC.SetDefaults` (Terraria 1.4.5.8).

    `UnlockableNPCEntryIcon.UpdatePosition` (`UnlockableNPCEntryIcon.cs:112-123`)
    centres the icon when `_npcCache.noGravity` is set, else bottom-anchors it.
    The cache runs `SetDefaults` ONLY (no AI), so the floater set is exactly the
    types whose `SetDefaults` arm sets `this.noGravity = true`. Parsed from the
    `SetDefaults` method body of `decompiled/Terraria/NPC.cs` (brace-matched),
    walking every `if/else-if (this.type ...)` arm — including the compound
    `type == A || type == B` arms and the `type >= A && type <= B` range arms.
    There are NO aiStyle blocks inside SetDefaults (the runtime AI()'s aiStyle
    blocks set noGravity only at runtime, which the icon cache never runs), so
    no aiStyle inheritance is applied.
    """
    npc_src = (ROOT / "decompiled" / "Terraria" / "NPC.cs").read_text(encoding="utf-8")
    m = re.search(r"public void SetDefaults\(int Type.*?\)\s*\{", npc_src, re.S)
    j = npc_src.index("{", m.start())
    d = 0
    k = j
    while k < len(npc_src):
        if npc_src[k] == "{":
            d += 1
        elif npc_src[k] == "}":
            d -= 1
            if d == 0:
                break
        k += 1
    body = npc_src[j + 1 : k]
    ng = set()
    for mm in re.finditer(r"(?:else )?if \(([^)]*this\.type[^)]*)\)\s*\{", body):
        cond = mm.group(1)
        jj = body.index("{", mm.start())
        dd = 0
        kk = jj
        while kk < len(body):
            if body[kk] == "{":
                dd += 1
            elif body[kk] == "}":
                dd -= 1
                if dd == 0:
                    break
            kk += 1
        blk = body[jj + 1 : kk]
        if "this.noGravity = true" not in blk:
            continue
        ids = set(int(x) for x in re.findall(r"==\s*(\d+)", cond))
        for a in re.findall(r">=\s*(\d+)", cond):
            for b in re.findall(r"<=\s*(\d+)", cond):
                if int(a) <= int(b):
                    ids.update(range(int(a), int(b) + 1))
        ng |= ids
    rows = ", ".join(str(t) for t in sorted(ng))
    return (
        "/// The `NPC.noGravity` set from `NPC.SetDefaults` (NPC.cs): the types\n"
        "/// whose SetDefaults arm sets `noGravity = true` (compound `||` and\n"
        "/// `>=`/`<=` range arms included). The icon cache runs SetDefaults only,\n"
        "/// so runtime-AI noGravity (e.g. Demon Eye) does NOT count.\n"
        "/// `UnlockableNPCEntryIcon.UpdatePosition` (`:112-123`) centres these\n"
        "/// (floaters) and bottom-anchors the rest. GENERATED — do not hand-edit.\n"
        f"pub static NO_GRAVITY: &[i32] = &[{rows}];\n\n"
        "/// Whether the bestiary icon for this net id floats (centred anchor).\n"
        "pub fn no_gravity(net_id: i32) -> bool {\n"
        "    NO_GRAVITY.binary_search(&net_id).is_ok()\n"
        "}\n"
    )


if __name__ == "__main__":
    main()
