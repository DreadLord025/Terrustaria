#!/usr/bin/env python3
"""S1-F-7 wave A — generate the TileObjectData registry from the decompiled C#.

Parses `Terraria/ObjectData/TileObjectData.cs` `Initialize()` (:2110-5014) and
INTERPRETS the builder chain 1:1, then emits the resolved static registry as
`crates/tr-core/src/tileobject/registry.rs`. The emitted file is GENERATED —
never hand-edit; fix this generator and regenerate (`--check` verifies the
committed table, the AGENTS regen-emptiness pattern).

C# semantics reproduced (cite = TileObjectData.cs unless noted):

* `_baseObject` defaults — `SetupBaseObject` :1743-1803: Width/Height 1,
  Origin (0,0), Direction None, RandomStyleRange 0, FlattenAnchors false,
  anchors default, anchor lists null, WaterDeath/LavaDeath false,
  WaterPlacement Allowed / LavaPlacement NotAllowed, hooks default,
  CoordinateHeights {16}, CoordinateWidth 0, CoordinatePadding 0,
  CoordinatePaddingFix (0,0), Style 0, StyleHorizontal false,
  StyleWrapLimit 0, StyleMultiplier 1, StyleLineSkip 1 (module ctor default,
  TileObjectStyleModule.cs:19).
* `Initialize` prelude :2117-2139: newTile gets heights {16}, width 16,
  padding 2, horizontal, multiplier 27, wrap 27, LavaDeath=true; newSubTile
  copies it then LavaDeath=false + LavaPlacement=Allowed; the platform
  sub-tiles 13/43/47 and addTile(19) open the chain.
* `CopyFrom` :1698-1715 shares the module REFERENCES (and the two `Uses*`
  flags) — it does NOT touch the `_hasOwn*` flags. Every property setter is
  copy-on-write: when `!_hasOwnX` it first compares the current value and
  RETURNS UNCHANGED on equality (no clone, no write); otherwise it clones the
  module and sets `_hasOwnX`. `AnchorAlternateTiles`'s no-op compare reads
  `tileInvalid` (:449) — a C# bug, reproduced. `AnchorValidWalls` has no
  no-op compare (:486-490). `SpecificRandomStyles` compares by REFERENCE
  (:1366). Width/Height setters also clone the coords module when not owned
  (:1195-1200/:1234-1239). CoordinateHeights' clone passes the new heights
  into the module ctor (:1437, TileObjectCoordinatesModule.cs:33-77).
* `LinkedAlternates` :126-142 clones the alternates module (deep-copying the
  entries as fresh `TileObjectData(copy)` objects that SHARE the entries'
  modules, TileObjectAlternatesModule.cs:17-27); while linked, every
  value setter propagates to the alternates through their own setters
  (arrays cloned per alternate).
* `Calculate` :1805-1832: styleWidth = (cw+pad)*W + fix.x, styleHeight =
  sum(heights[i]+pad) + fix.y; when `_hasOwnLiquidDeath`, LavaDeath=true
  folds LavaPlacement=NotAllowed and WaterDeath=true folds
  WaterPlacement=NotAllowed (through the setters). Runs at addTile/addSubTile/
  addAlternate/addBaseTile time and lazily from the FullWidth/Height getters.
* `addTile` :2008-2013 / `addBaseTile` :2000-2006 / `addSubTile`
  :2015-2095 / `addSubTileRange` :2043-2070 / `addAlternate` :2097-2108:
  Calculate, register the cursor object, reset the cursor to a fresh
  `new TileObjectData(_baseObject)`. addSubTile pads the style list with null
  and registers the SAME object under every listed style; addAlternate sets
  `Style = baseStyle` AFTER Calculate and appends.
* `FullCopyFrom` :1722-1741 copies the modules AND `_subTiles = new
  TileObjectSubTilesModule(copy._subTiles)` — whose copy ctor loops
  `i < this.data.Count` over the FRESH list (Count 0, capacity != count,
  TileObjectSubTilesModule.cs:17-27): the copy's sub-tile list is EMPTY.
  A C# bug, reproduced — tiles 83/84 (FullCopyFrom 82/83, :4785-4788) and
  389 (FullCopyFrom 388, :2323) resolve no sub-tiles in vanilla.
* `ApplyNaturalObjectRules` :1847-1851: UsesCustomCanPlace=false,
  UsesGlobalLiquidChecks=true.
* The manual bed/bathtub/chair/toilet sub-tile alternate blocks
  (:2597-2603/:2637-2643/:4165-4171/:4198-4204): newAlternate copies the
  sub-tile, Calculate, Style=1, append to the sub-tile's OWN alternates list,
  reset the cursor.

Emitted per def: the static registry fields (width/height/origin/direction/
random style range/specific random styles/flatten anchors/the four anchors +
wall/anchor tile lists/liquid placement + death flags/coordinate rules/style
wrap-multiplier-line-skip-horizontal/alternates/sub-tiles/Uses* flags) plus
`frame_important` from `Main.tileFrameImportant` (Main.cs assignments +
`AddEchoFurnitureTile` :8412-8420/:8426-8431 + the 435..439 loop :9986-9994).

Documented callback seams (wave A — data generated, runtime hooks stay
callbacks in the port):
* `PlacementHook` entries emit the C# method NAME + badReturn/badResponse/
  processedCoordinates with `hook: None` — the hook bodies (Chest.FindEmptyChest,
  WorldGen.PlantCheck_CanPlaceHook, the TE* hooks, ...) are runtime-state
  consumers outside this task.
* `GetStyleOverride` emits the tag (Detritus/SmallPiles/Stalactite,
  :5294-5327) — the frameY-based resolution is the tile-based
  `GetTileData(Tile)` overload (:5196-5261), a framing/cascade consumer.
* Draw-only fields (DrawYOffset/DrawXOffset/DrawFlip*/DrawStepDown/
  DrawStyleOffset/DrawFrameOffsets/StyleWrapLimitVisualOverride/
  styleLineSkipVisualOverride) are parsed (they ride the same copy-on-write
  machinery) but NOT emitted — S1-F-9 (DrawData) scope.
* `interaction` is the port's right-click dispatch tag (edit.rs
  `interact_at`, Decision #3), not C# registry data: the mapping below keeps
  the wooden slice's tags verbatim (Player.cs citations per entry).

Usage:
    python tools/GenTileObjectData/gen.py [repo_root] [--check]
"""

import re
import sys
from pathlib import Path

TILE_COUNT = 754

# --------------------------------------------------------------------------
# The C# object model — modules, copy-on-write setters, the cursor machine.
# --------------------------------------------------------------------------

ANCHOR_TYPES = {
    "None": 0, "SolidTile": 1, "SolidWithTop": 2, "Table": 4, "SolidSide": 8,
    "Tree": 0x10, "AlternateTile": 0x20, "EmptyTile": 0x40, "SolidBottom": 0x80,
    "Platform": 0x100, "PlanterBox": 0x200, "PlatformNonHammered": 0x400,
    "AllFlatHeight": 0x800,
}
DIRECTIONS = {"None": 0, "PlaceLeft": 1, "PlaceRight": 2}
LIQUID = {"Allowed": 0, "NotAllowed": 1, "OnlyInLiquid": 2, "OnlyInFullLiquid": 3}

# Module field tables: module key -> (hasOwn flag suffix, fields).
# Each field: (property name, module field, kind). kinds:
#   scalar / array (deepCompare no-op) / refarray (reference no-op) /
#   anchor (AnchorData no-op) / walls (no no-op compare) / hook (no compare)
MODULES = {
    "anchor": ("Anchor", [
        ("AnchorTop", "top", "anchor"), ("AnchorBottom", "bottom", "anchor"),
        ("AnchorLeft", "left", "anchor"), ("AnchorRight", "right", "anchor"),
        ("AnchorWall", "wall", "scalar"),
    ]),
    "anchorTiles": ("AnchorTiles", [
        ("AnchorValidTiles", "tileValid", "array"),
        ("AnchorInvalidTiles", "tileInvalid", "array"),
        ("AnchorAlternateTiles", "tileAlternates", "array_bug"),
        ("AnchorValidWalls", "wallValid", "walls"),
    ]),
    "liquidDeath": ("LiquidDeath", [
        ("WaterDeath", "water", "scalar"), ("LavaDeath", "lava", "scalar"),
    ]),
    "liquidPlacement": ("LiquidPlacement", [
        ("WaterPlacement", "water", "scalar"), ("LavaPlacement", "lava", "scalar"),
    ]),
    "hooks": ("PlacementHooks", [
        ("HookCheckIfCanPlace", "check", "hook"),
        ("HookPostPlaceEveryone", "postPlaceEveryone", "hook"),
        ("HookPostPlaceMyPlayer", "postPlaceMyPlayer", "hook"),
        ("HookPlaceOverride", "placeOverride", "hook"),
        ("GetStyleOverride", "getStyleMethod", "hook"),
    ]),
    "draw": ("TileObjectDraw", [
        ("DrawXOffset", "xOffset", "scalar"), ("DrawYOffset", "yOffset", "scalar"),
        ("DrawFlipHorizontal", "flipHorizontal", "scalar"),
        ("DrawFlipVertical", "flipVertical", "scalar"),
        ("DrawStepDown", "stepDown", "scalar"),
    ]),
    "style": ("TileObjectStyle", [
        ("Style", "style", "scalar"), ("StyleHorizontal", "horizontal", "scalar"),
        ("StyleWrapLimit", "styleWrapLimit", "scalar"),
        ("StyleMultiplier", "styleMultiplier", "scalar"),
        ("StyleLineSkip", "styleLineSkip", "scalar"),
        ("StyleWrapLimitVisualOverride", "styleWrapLimitVisualOverride", "scalar"),
        ("styleLineSkipVisualOverride", "styleLineSkipVisualoverride", "scalar"),
    ]),
    "base": ("TileObjectBase", [
        ("Width", "width", "scalar_size"), ("Height", "height", "scalar_size"),
        ("Origin", "origin", "scalar"), ("Direction", "direction", "scalar"),
        ("RandomStyleRange", "randomRange", "scalar"),
        ("FlattenAnchors", "flattenAnchors", "scalar"),
        ("SpecificRandomStyles", "specificRandomStyles", "refarray"),
    ]),
    "coords": ("TileObjectCoords", [
        ("CoordinateHeights", "heights", "heights"),
        ("CoordinateWidth", "width", "scalar_coord"),
        ("CoordinatePadding", "padding", "scalar_coord"),
        ("CoordinatePaddingFix", "paddingFix", "scalar_coord"),
        ("DrawStyleOffset", "drawStyleOffset", "scalar_coord_draw"),
        ("DrawFrameOffsets", "drawFrameOffsets", "frameoffs"),
    ]),
}

# Which setters propagate to linked alternates (all except the hooks module
# and DrawStyleOffset's module-mates — DrawStyleOffset DOES propagate :1661).
NO_LINK = {"check", "postPlaceEveryone", "postPlaceMyPlayer", "placeOverride",
           "getStyleMethod"}


def clone_module(key, mod):
    """The C# module copy constructors (deep array copies, shared object refs
    for alternates entries)."""
    if mod is None:
        return None
    out = dict(mod)
    if key == "anchorTiles":
        for f in ("tileValid", "tileInvalid", "tileAlternates", "wallValid"):
            if out[f] is not None:
                out[f] = list(out[f])
    elif key == "base":
        if out["specificRandomStyles"] is not None:
            out["specificRandomStyles"] = list(out["specificRandomStyles"])
    elif key == "coords":
        if out["heights"] is not None:
            out["heights"] = list(out["heights"])
        if out["drawFrameOffsets"] is not None:
            out["drawFrameOffsets"] = [list(r) for r in out["drawFrameOffsets"]]
    elif key == "alternates":
        if out["data"] is not None:
            out["data"] = [TOD(copy_from=e) for e in out["data"]]
    return out


def default_module(key):
    """The C# parameterless module constructors."""
    if key == "anchor":
        return {"top": (0, 0, 0), "bottom": (0, 0, 0), "left": (0, 0, 0),
                "right": (0, 0, 0), "wall": False}
    if key == "anchorTiles":
        return {"tileValid": None, "tileInvalid": None,
                "tileAlternates": None, "wallValid": None}
    if key == "liquidDeath":
        return {"water": False, "lava": False}
    if key == "liquidPlacement":
        return {"water": 0, "lava": 0}
    if key == "hooks":
        return {"check": None, "postPlaceEveryone": None,
                "postPlaceMyPlayer": None, "placeOverride": None,
                "getStyleMethod": None}
    if key == "draw":
        return {"xOffset": 0, "yOffset": 0, "flipHorizontal": False,
                "flipVertical": False, "stepDown": 0}
    if key == "style":
        return {"style": 0, "horizontal": False, "styleWrapLimit": 0,
                "styleMultiplier": 1, "styleLineSkip": 1,
                "styleWrapLimitVisualOverride": None,
                "styleLineSkipVisualoverride": None}
    if key == "base":
        return {"width": 1, "height": 1, "origin": (0, 0), "direction": 0,
                "randomRange": 0, "flattenAnchors": False,
                "specificRandomStyles": None}
    if key == "coords":
        return {"width": 0, "padding": 0, "paddingFix": (0, 0),
                "styleWidth": 0, "styleHeight": 0, "calculated": False,
                "heights": None, "drawStyleOffset": 0, "drawFrameOffsets": None}
    if key == "alternates":
        return {"data": None}
    if key == "subTiles":
        return {"data": None}
    raise KeyError(key)


class TOD:
    """One `TileObjectData` object: module references + `_hasOwn*` flags."""

    def __init__(self, copy_from=None):
        self.modules = {k: None for k in MODULES}
        self.modules["alternates"] = None
        self.modules["subTiles"] = None
        self.own = {k: False for k in MODULES}
        self.own["alternates"] = False
        self.own["subTiles"] = False
        self.linked = False
        self.uses_custom = False
        self.uses_global_liquid = False
        self.parent = None
        if copy_from is not None:
            self.copy_from(copy_from)

    def copy_from(self, other):
        # :1698-1715 — module refs + the two Uses flags; hasOwn untouched.
        self.uses_custom = other.uses_custom
        self.uses_global_liquid = other.uses_global_liquid
        for k in self.modules:
            self.modules[k] = other.modules[k]

    def full_copy_from(self, other):
        # :1722-1741 — plus a FRESH subTiles module whose copy ctor's loop
        # runs over the fresh list's Count (0): the data stays None/empty.
        self.copy_from(other)
        src = other.modules["subTiles"]
        fresh = {"data": None}
        if src is not None and src["data"] is not None:
            # new List<T>(count) then `for i < this.data.Count` — Count is 0.
            fresh["data"] = []
        self.modules["subTiles"] = fresh
        self.own["subTiles"] = True

    def setup_base_object(self):
        # :1743-1803 — every module owned, the documented defaults.
        for k in MODULES:
            self.modules[k] = default_module(k)
            self.own[k] = True
        self.modules["alternates"] = {"data": []}
        self.own["alternates"] = True
        self.modules["subTiles"] = {"data": []}
        self.own["subTiles"] = True
        self.modules["coords"]["heights"] = [16]
        self.modules["liquidPlacement"]["lava"] = 1  # NotAllowed (:1768)

    # -- property access ----------------------------------------------------

    def get(self, prop):
        for key, (_, fields) in MODULES.items():
            for p, f, _kind in fields:
                if p == prop:
                    return self.modules[key][f]
        if prop == "Alternates":
            return self.modules["alternates"]["data"]
        if prop == "SubTiles":
            return self.modules["subTiles"]["data"]
        raise KeyError(prop)

    def set(self, prop, value):
        if prop == "LinkedAlternates":
            # :126-142
            if value and not self.own["alternates"]:
                self.own["alternates"] = True
                self.modules["alternates"] = clone_module(
                    "alternates", self.modules["alternates"])
            self.linked = value
            return
        if prop == "Alternates":
            # :170-189 — clone the module, then replace the list.
            if not self.own["alternates"]:
                self.own["alternates"] = True
                self.modules["alternates"] = clone_module(
                    "alternates", self.modules["alternates"])
            self.modules["alternates"]["data"] = value
            return
        if prop == "UsesCustomCanPlace":
            self.uses_custom = value
            return
        if prop == "UsesGlobalLiquidChecks":
            self.uses_global_liquid = value
            return
        for key, (_, fields) in MODULES.items():
            for p, f, kind in fields:
                if p != prop:
                    continue
                self._set_field(key, f, kind, value)
                return
        raise KeyError(prop)

    def _set_field(self, key, f, kind, value):
        mod = self.modules[key]
        if not self.own[key]:
            # The no-op compare (:206/:239/... — per-kind semantics).
            cur = mod[f]
            if kind in ("scalar", "scalar_size", "scalar_coord",
                        "scalar_coord_draw", "anchor"):
                if cur == value:
                    return
            elif kind in ("array", "heights", "frameoffs"):
                if deep_compare(cur, value):
                    return
            elif kind == "array_bug":
                # :449 — compares against tileInvalid, a C# bug.
                if deep_compare(mod["tileInvalid"], value):
                    return
            elif kind == "refarray":
                if cur is value:
                    return
            # walls / hook: no compare.
            self.own[key] = True
            if kind == "heights":
                # :1437 — the ctor takes the new heights directly.
                new = clone_module(key, mod)
                new["heights"] = value
                self.modules[key] = new
            else:
                self.modules[key] = clone_module(key, mod)
            mod = self.modules[key]
            if kind == "scalar_size" and not self.own["coords"]:
                # Width/Height :1195-1200/:1234-1239 — clone coords too.
                self.own["coords"] = True
                self.modules["coords"] = clone_module("coords",
                                                      self.modules["coords"])
                self.modules["coords"]["calculated"] = False
        mod[f] = value
        if kind in ("scalar_coord", "scalar_coord_draw", "heights", "frameoffs"):
            mod["calculated"] = False
        if self.linked and f not in NO_LINK:
            for alt in self.modules["alternates"]["data"]:
                v = value
                if kind in ("array", "array_bug", "walls", "heights") and value is not None:
                    v = list(value)
                if kind == "frameoffs" and value is not None:
                    v = [list(r) for r in value]
                alt._set_field(key, f, kind, v)

    def calculate(self):
        # :1805-1832
        c = self.modules["coords"]
        if c["calculated"]:
            return
        c["calculated"] = True
        c["styleWidth"] = ((c["width"] + c["padding"]) * self.get("Width")
                           + c["paddingFix"][0])
        c["styleHeight"] = (sum(h + c["padding"] for h in c["heights"])
                            + c["paddingFix"][1])
        if self.own["liquidDeath"]:
            if self.modules["liquidDeath"]["lava"]:
                self.set("LavaPlacement", 1)
            if self.modules["liquidDeath"]["water"]:
                self.set("WaterPlacement", 1)

    def apply_natural_object_rules(self):
        # :1847-1851
        self.uses_custom = False
        self.uses_global_liquid = True


def deep_compare(a, b):
    # Utils.cs:2096-2118 (int[]) / :2120+ (Rectangle[,]).
    if a is None and b is None:
        return True
    if a is None or b is None:
        return False
    return list(a) == list(b)


# --------------------------------------------------------------------------
# The Initialize() statement interpreter.
# --------------------------------------------------------------------------

class Interp:
    def __init__(self):
        self.base = TOD()
        self.base.setup_base_object()
        self.data = [None] * TILE_COUNT
        self.styles = {}
        self.locals = {}
        self.cursor = {}
        self.read_only = False
        self.counts = {"addTile": 0, "addSubTile": 0, "addAlternate": 0,
                       "addBaseTile": 0}

    def fresh(self, name):
        self.cursor[name] = TOD(copy_from=self.base)

    def prelude(self):
        # :2117-2137
        self.fresh("newTile")
        self.fresh("newSubTile")
        self.fresh("newAlternate")
        nt = self.cursor["newTile"]
        nt.set("CoordinateHeights", [16])
        nt.set("CoordinateWidth", 16)
        nt.set("CoordinatePadding", 2)
        nt.set("StyleHorizontal", True)
        nt.set("StyleMultiplier", 27)
        nt.set("StyleWrapLimit", 27)
        nt.set("UsesCustomCanPlace", False)
        nt.set("LavaDeath", True)
        ns = self.cursor["newSubTile"]
        ns.copy_from(nt)
        ns.set("LavaDeath", False)
        ns.set("LavaPlacement", 0)

    # -- expression evaluation ----------------------------------------------

    def eval(self, expr):
        e = expr.strip()
        if e == "null":
            return None
        if e == "true":
            return True
        if e == "false":
            return False
        if e == "default" or e == "AnchorData.Empty":
            return (0, 0, 0)
        if e == "Point16.Zero":
            return (0, 0)
        if re.fullmatch(r"-?\d+", e):
            return int(e)
        if e in self.locals:
            return self.locals[e]
        m = re.fullmatch(r"new Point16\((.+),\s*(.+)\)", e)
        if m:
            return (self.eval(m.group(1)), self.eval(m.group(2)))
        m = re.fullmatch(r"new AnchorData\((.+)\)", e, re.S)
        if m:
            args = split_top(m.group(1))
            assert len(args) == 3, e
            return (self.eval_anchor_type(args[0]), self.eval(args[1]),
                    self.eval(args[2]))
        m = re.fullmatch(r"new int\[(\d+)\]\s*\{(.*)\}", e, re.S)
        if m:
            vals = [int(x) for x in split_top(m.group(2)) if x.strip()]
            assert len(vals) == int(m.group(1)), e
            return vals
        m = re.fullmatch(r"new int\[(\d+)\]", e)
        if m:
            return [0] * int(m.group(1))
        m = re.fullmatch(r"new Rectangle\[(\d+),\s*(\d+)\]\s*\{(.*)\}", e, re.S)
        if m:
            rows, cols = int(m.group(1)), int(m.group(2))
            rects = re.findall(r"new Rectangle\((-?\d+),\s*(-?\d+),\s*(-?\d+),\s*(-?\d+)\)",
                               m.group(3))
            assert len(rects) == rows * cols, e
            flat = [tuple(int(v) for v in r) for r in rects]
            return [flat[i * cols:(i + 1) * cols] for i in range(rows)]
        m = re.fullmatch(r"new Rectangle\[(\d+),\s*(\d+)\]", e)
        if m:
            return [[(0, 0, 0, 0)] * int(m.group(2)) for _ in range(int(m.group(1)))]
        m = re.fullmatch(r"new PlacementHook\((.+)\)", e, re.S)
        if m:
            args = split_top(m.group(1))
            assert len(args) == 4, e
            name = args[0].strip()
            bad_return = self.eval(args[1])
            bad_response = self.eval(args[2])
            pc = args[3].strip()
            m2 = re.fullmatch(r"processedCoordinates:\s*(true|false)", pc)
            assert m2, e
            return ("hook", name, bad_return, bad_response, m2.group(1) == "true")
        m = re.fullmatch(r"new List<TileObjectData>\(\)", e)
        if m:
            return []
        m = re.fullmatch(r"TileObjectData\.GetStyle_(\w+)", e)
        if m:
            return ("getstyle", m.group(1))
        m = re.fullmatch(r"TileObjectDirection\.(\w+)", e)
        if m:
            return DIRECTIONS[m.group(1)]
        m = re.fullmatch(r"LiquidPlacement\.(\w+)", e)
        if m:
            return LIQUID[m.group(1)]
        m = re.fullmatch(r"TileObjectData\.(newTile|newSubTile|newAlternate)", e)
        if m:
            return self.cursor[m.group(1)]
        m = re.fullmatch(r"TileObjectData\.(newTile|newSubTile|newAlternate)\.(\w+)", e)
        if m:
            return self.cursor[m.group(1)].get(m.group(2))
        m = re.fullmatch(r"(\d+)\s*\*\s*TileObjectData\.(newTile|newSubTile|newAlternate)\.(\w+)", e)
        if m:
            return int(m.group(1)) * self.cursor[m.group(2)].get(m.group(3))
        # `addSubTile(1 + newTile.StyleWrapLimit)` (:3343/:3347/:3351).
        m = re.fullmatch(r"(\d+)\s*\+\s*TileObjectData\.(newTile|newSubTile|newAlternate)\.(\w+)", e)
        if m:
            return int(m.group(1)) + self.cursor[m.group(2)].get(m.group(3))
        m = re.fullmatch(r"TileObjectData\.(Style\w+)", e)
        if m:
            return self.styles[m.group(1)]
        if e == "TileObjectData._baseObject":
            return self.base
        if e == "anchorInvalidTiles":
            return self.locals["anchorInvalidTiles"]
        raise ValueError(f"unhandled expression: {e!r}")

    def eval_anchor_type(self, expr):
        v = 0
        for tok in expr.split("|"):
            tok = tok.strip()
            m = re.fullmatch(r"AnchorType\.(\w+)", tok)
            assert m, expr
            v |= ANCHOR_TYPES[m.group(1)]
        return v

    # -- statement execution --------------------------------------------------

    def exec_block(self, stmts):
        for s in stmts:
            self.exec_stmt(s)

    def exec_stmt(self, s):
        s = s.strip()
        if not s:
            return
        # -- the Initialize prelude (:2117-2126), interpreted not hardcoded --
        if s == "TileObjectData._baseObject = new TileObjectData()":
            self.base = TOD()
            return
        if s == "TileObjectData._baseObject.SetupBaseObject()":
            self.base.setup_base_object()
            return
        if s == "TileObjectData._data = new List<TileObjectData>(TileID.Count)":
            self.data = [None] * TILE_COUNT
            return
        if s == "TileObjectData._data.Add(null)":
            return  # the null-fill loop; data already None
        m = re.fullmatch(r"TileObjectData\.(newTile|newSubTile|newAlternate) = "
                         r"new TileObjectData\(TileObjectData\._baseObject\)", s)
        if m:
            self.fresh(m.group(1))
            return
        m = re.fullmatch(r"int (\w+) = (-?\d+)", s)
        if m:
            self.locals[m.group(1)] = int(m.group(2))
            return
        m = re.fullmatch(r"int\[\] anchorInvalidTiles = new int\[11\]\s*\{(.*)\}", s, re.S)
        if m:
            self.locals["anchorInvalidTiles"] = [
                int(x) for x in split_top(m.group(1)) if x.strip()]
            return
        m = re.fullmatch(r"TileObjectData\.readOnlyData = true", s)
        if m:
            self.read_only = True
            return
        m = re.fullmatch(r"TileObjectData\.addTile\((.+)\)", s, re.S)
        if m:
            t = self.eval(m.group(1))
            nt = self.cursor["newTile"]
            nt.calculate()
            # `_data[tileType] = newTile` (:2011) — a second addTile for the
            # same kind overwrites (e.g. 287 at :2681 then :2716); last wins.
            self.data[t] = nt
            self.counts["addTile"] += 1
            self.fresh("newTile")
            return
        m = re.fullmatch(r"TileObjectData\.addBaseTile\(out TileObjectData\.(\w+)\)", s)
        if m:
            nt = self.cursor["newTile"]
            nt.calculate()
            nt.parent = self.base
            self.styles[m.group(1)] = nt
            self.counts["addBaseTile"] += 1
            self.fresh("newTile")
            return
        m = re.fullmatch(r"TileObjectData\.addSubTile\((.+)\)", s, re.S)
        if m:
            styles = [self.eval(x) for x in split_top(m.group(1))]
            self.add_sub_tile(styles)
            return
        m = re.fullmatch(r"TileObjectData\.addSubTileRange\((.+),\s*(.+)\)", s, re.S)
        if m:
            start, count = self.eval(m.group(1)), self.eval(m.group(2))
            self.add_sub_tile(list(range(start, start + count)))
            return
        m = re.fullmatch(r"TileObjectData\.addAlternate\((.+)\)", s, re.S)
        if m:
            base_style = self.eval(m.group(1))
            na = self.cursor["newAlternate"]
            nt = self.cursor["newTile"]
            na.calculate()
            if not nt.own["alternates"]:
                nt.set("Alternates", [])
            na.set("Style", base_style)
            na.parent = nt
            nt.modules["alternates"]["data"].append(na)
            self.counts["addAlternate"] += 1
            self.fresh("newAlternate")
            return
        m = re.fullmatch(r"TileObjectData\.(newTile|newSubTile|newAlternate)\.CopyFrom\((.+)\)", s, re.S)
        if m:
            src = self.eval(m.group(2))
            self.cursor[m.group(1)].copy_from(src)
            return
        m = re.fullmatch(r"TileObjectData\.(newTile|newSubTile|newAlternate)\.FullCopyFrom\((.+)\)", s, re.S)
        if m:
            arg = m.group(2).strip()
            if re.fullmatch(r"\d+", arg):
                # :1717-1720 — GetTileData(tileType, 0): the registered base
                # def (sub-tiles do not apply at style 0 unless registered).
                src = self.data[int(arg)]
                sub = src.modules["subTiles"]["data"]
                if sub and len(sub) > 0 and sub[0] is not None:
                    src = sub[0]
            else:
                src = self.eval(arg)
            self.cursor[m.group(1)].full_copy_from(src)
            return
        m = re.fullmatch(r"TileObjectData\.(newTile|newSubTile|newAlternate)\.ApplyNaturalObjectRules\(\)", s)
        if m:
            self.cursor[m.group(1)].apply_natural_object_rules()
            return
        m = re.fullmatch(r"TileObjectData\.(newTile|newSubTile|newAlternate)\.Calculate\(\)", s)
        if m:
            self.cursor[m.group(1)].calculate()
            return
        m = re.fullmatch(r"TileObjectData\.(newTile|newSubTile|newAlternate)\._parent = TileObjectData\.(newTile|newSubTile|newAlternate)", s)
        if m:
            self.cursor[m.group(1)].parent = self.cursor[m.group(2)]
            return
        # In-place DrawFrameOffsets cell writes (:4011-4014) — the array is
        # already owned at these sites (assigned at :4005); draw-only, but the
        # mutation is interpreted so the module state stays faithful.
        m = re.fullmatch(r"TileObjectData\.(newTile|newSubTile|newAlternate)\.DrawFrameOffsets\[(\d+), (\d+)\] = new Rectangle\((-?\d+), (-?\d+), (-?\d+), (-?\d+)\)", s)
        if m:
            grid = self.cursor[m.group(1)].get("DrawFrameOffsets")
            grid[int(m.group(2))][int(m.group(3))] = tuple(
                int(m.group(i)) for i in range(4, 8))
            return
        m = re.fullmatch(r"TileObjectData\.(newTile|newSubTile|newAlternate)\.Alternates\.Add\(TileObjectData\.(newTile|newSubTile|newAlternate)\)", s)
        if m:
            self.cursor[m.group(1)].modules["alternates"]["data"].append(
                self.cursor[m.group(2)])
            return
        m = re.fullmatch(r"TileObjectData\.newAlternate = new TileObjectData\(TileObjectData\._baseObject\)", s)
        if m:
            self.fresh("newAlternate")
            return
        m = re.fullmatch(r"TileObjectData\.(newTile|newSubTile|newAlternate)\.(\w+) = (.+)", s, re.S)
        if m:
            self.cursor[m.group(1)].set(m.group(2), self.eval(m.group(3)))
            return
        raise ValueError(f"unhandled statement: {s!r}")

    def add_sub_tile(self, styles):
        ns = self.cursor["newSubTile"]
        nt = self.cursor["newTile"]
        ns.calculate()
        for style in styles:
            if not nt.own["subTiles"]:
                nt.modules["subTiles"] = {"data": []}
                nt.own["subTiles"] = True
            lst = nt.modules["subTiles"]["data"]
            while len(lst) <= style:
                lst.append(None)
            ns.parent = nt
            lst[style] = ns
        self.counts["addSubTile"] += 1
        self.fresh("newSubTile")


def split_top(s):
    """Split on top-level commas (parens/brackets/braces aware)."""
    parts, depth, cur = [], 0, []
    for ch in s:
        if ch in "([{":
            depth += 1
        elif ch in ")]}":
            depth -= 1
        if ch == "," and depth == 0:
            parts.append("".join(cur))
            cur = []
        else:
            cur.append(ch)
    if "".join(cur).strip():
        parts.append("".join(cur))
    return parts


FOR_HDR = re.compile(
    r"for \(int (\w+) = (-?\d+); \w+ (<=|<) (-?\d+|TileID\.Count); \w+\+\+\)")


def read_stmt(text, pos):
    """One statement from `text` at `pos`: returns (stmt, next_pos, for_body,
    for_header). `;` inside parens (a `for` header) does not terminate; a
    `{...}` block is kept inline UNLESS the pending text is a `for` header,
    in which case the block is returned as the loop body."""
    n = len(text)
    buf = []
    paren = 0
    i = pos
    while i < n:
        ch = text[i]
        if ch == "(":
            paren += 1
        elif ch == ")":
            paren -= 1
        if ch == ";" and paren == 0:
            return "".join(buf).strip(), i + 1, None, None
        if ch == "{" and paren == 0:
            depth, j = 1, i + 1
            while j < n and depth:
                if text[j] == "{":
                    depth += 1
                elif text[j] == "}":
                    depth -= 1
                j += 1
            pending = "".join(buf).strip()
            m = FOR_HDR.fullmatch(pending)
            if m:
                return pending, j, text[i + 1:j - 1], m
            buf.append(text[i:j])
            i = j
            continue
        buf.append(ch)
        i += 1
    return "".join(buf).strip(), n, None, None


def parse_initialize(src):
    """Extract the Initialize() body and interpret every statement in order,
    expanding the `for` loops (:2120/:2149/:2306/:2315) with the loop variable
    bound per iteration."""
    start = src.index("public static void Initialize()")
    brace = src.index("{", start)
    depth, j = 1, brace + 1
    while depth:
        if src[j] == "{":
            depth += 1
        elif src[j] == "}":
            depth -= 1
        j += 1
    body = re.sub(r"//[^\n]*", "", src[brace + 1:j - 1])

    interp = Interp()

    def run(text):
        pos = 0
        while pos < len(text):
            stmt, pos, for_body, hdr = read_stmt(text, pos)
            if hdr:
                var, lo, op = hdr.group(1), int(hdr.group(2)), hdr.group(3)
                hi = TILE_COUNT if hdr.group(4) == "TileID.Count" else int(hdr.group(4))
                end = hi + 1 if op == "<=" else hi
                for v in range(lo, end):
                    interp.locals[var] = v
                    run(for_body)
                interp.locals.pop(var, None)
            elif stmt:
                interp.exec_stmt(stmt)

    run(body)
    assert interp.read_only, "Initialize must end with readOnlyData = true"
    return interp


# --------------------------------------------------------------------------
# Main.tileFrameImportant (the per-def frame_important flag + the full table).
# --------------------------------------------------------------------------

def parse_frame_important(main_src):
    table = [False] * TILE_COUNT
    for m in re.finditer(r"Main\.tileFrameImportant\[(\d+)\]\s*=\s*(true|false)\s*;", main_src):
        idx = int(m.group(1))
        if 0 <= idx < TILE_COUNT:
            table[idx] = m.group(2) == "true"
    # AddEchoFurnitureTile (:8426-8431) marks its argument frame-important.
    for m in re.finditer(r"Main\.AddEchoFurnitureTile\((\d+)\)\s*;", main_src):
        idx = int(m.group(1))
        if 0 <= idx < TILE_COUNT:
            table[idx] = True
    # The 435..439 loop (:9986-9994) — find `for` headers whose body assigns
    # tileFrameImportant[<var>].
    for m in re.finditer(
            r"for \(int (\w+) = (\d+); \w+ <= (\d+); \w+\+\+\)\s*\{([^}]*)\}",
            main_src):
        var, lo, hi, block = m.group(1), int(m.group(2)), int(m.group(3)), m.group(4)
        if re.search(rf"Main\.tileFrameImportant\[{var}\]\s*=\s*true\s*;", block):
            for idx in range(lo, hi + 1):
                if 0 <= idx < TILE_COUNT:
                    table[idx] = True
    return table


# --------------------------------------------------------------------------
# KillTile_GetItemDrops' empty case group (the cell_drops gate).
# --------------------------------------------------------------------------

def parse_no_cell_drop(wg_src):
    """`WorldGen.KillTile_GetItemDrops` (`WorldGen.cs:63964-66092`): the big
    case-label group that ends in a bare `break;` (:66090) — those kinds drop
    nothing per-cell (their loot rides the `Check*` cascade record / the pot
    event / RNG rolls). The port's `cell_drops` used `def_for_tile(...).is_some()`
    as the gate, which only matched the wooden slice; with the full registry it
    would swallow the torch/platform/orb drops, so the gate becomes this
    generated table (the C#'s actual empty group)."""
    start = wg_src.index("public static void KillTile_GetItemDrops")
    # The second `switch (tileCache.type)` in the body is the per-cell switch
    # (the first is the includeLargeObjectDrops chest/dresser/campfire probe).
    body = wg_src[start:]
    switches = [m.start() for m in re.finditer(r"switch \(tileCache\.type\)", body)]
    assert len(switches) >= 2, "KillTile_GetItemDrops switch layout drifted"
    sw = body[switches[1]:]
    # Walk to the matching close brace of this switch.
    brace = sw.index("{")
    depth, j = 1, brace + 1
    while depth:
        if sw[j] == "{":
            depth += 1
        elif sw[j] == "}":
            depth -= 1
        j += 1
    sw = sw[brace + 1:j - 1]
    # Only TOP-LEVEL case groups count (nested switches — case 324's frameY
    # switch, case 239's style chain — have their own labels/breaks). Track
    # brace depth; a group of top-level `case N:` labels closed by a bare
    # top-level `break;` with no intervening statement drops nothing.
    empty = set()
    pending = []
    depth = 0
    pos = 0
    n = len(sw)
    while pos < n:
        ch = sw[pos]
        if ch == "{":
            depth += 1
            pending = []  # a block statement voids the empty candidacy
            pos += 1
            continue
        if ch == "}":
            depth -= 1
            pos += 1
            continue
        if depth == 0:
            m = re.match(r"\s*case (\d+):", sw[pos:])
            if m:
                pending.append(int(m.group(1)))
                pos += m.end()
                continue
            m = re.match(r"\s*break;", sw[pos:])
            if m:
                if pending:
                    empty.update(pending)
                pending = []
                pos += m.end()
                continue
            m = re.match(r"\s*default:", sw[pos:])
            if m:
                pending = []
                pos += m.end()
                continue
        # Any other top-level statement voids the pending empty candidacy.
        if depth == 0 and not ch.isspace():
            pending = []
        pos += 1
    table = [False] * TILE_COUNT
    for k in empty:
        if 0 <= k < TILE_COUNT:
            table[k] = True
    return table


# --------------------------------------------------------------------------
# Emission.
# --------------------------------------------------------------------------

# The port's right-click dispatch tag (edit.rs `interact_at` routes on it —
# Decision #3). NOT C# registry data: the C# chain tests tile-kind literals.
# Kept verbatim from the wooden slice's hand defs (S1-F-7 wave A regression
# baseline); every entry cites the Player.cs arm it stands for.
INTERACTION = {
    10: "Door",            # Player.cs:32835-32856
    11: "Door",            # Player.cs:32857-32864
    15: "Chair",           # Player.cs:32313-32321
    29: "Bank(-2)",        # Player.cs:33056-33138
    97: "Bank(-3)",        # Player.cs:33056-33138
    463: "Bank(-4)",       # Player.cs:33056-33138
    491: "Bank(-5)",       # Player.cs:33056-33138
    79: "Bed",             # Player.cs:32654-32692
    104: "Clock",          # Player.cs:32725-32759
    33: "Candle",          # Player.cs:32378-32390
    219: "Extractinator",  # Player.cs:41665-41697
    642: "Extractinator",  # Player.cs:41665-41697
}

GET_STYLE_TAGS = {"Detritus": "Detritus", "SmallPiles": "SmallPiles",
                  "Stalactite": "Stalactite"}


def rust_bool(v):
    return "true" if v else "false"


def emit_anchor(a):
    ty, count, start = a
    if ty == 0 and count == 0 and start == 0:
        return "AnchorData::EMPTY"
    return f"AnchorData::new({ty}, {count}, {start})"


def emit_int_list(v):
    if v is None:
        return "&[]"
    return "&[" + ", ".join(str(x) for x in v) + "]"


def emit_hook(h):
    if h is None:
        return "PlacementHook::EMPTY"
    _, name, bad_return, bad_response, pc = h
    return (f'PlacementHook {{ hook: None, callback: "{name}", '
            f"bad_return: {bad_return}, bad_response: {bad_response}, "
            f"processed_coordinates: {rust_bool(pc)} }}")


class Emitter:
    def __init__(self, frame_important):
        self.fi = frame_important
        self.lines = []
        self.names = {}   # id(obj) -> static name (shared sub-tile objects)
        self.order = []   # emission order (deterministic)
        self.owner = {}   # id(obj) -> owning tile kind (fi/interaction overlay)

    def name_for(self, obj, name):
        key = id(obj)
        if key in self.names:
            return self.names[key]
        self.names[key] = name
        self.order.append((name, obj))
        return name

    def collect(self, obj, name, is_top=True):
        """Register the object graph depth-first; children get stable names.

        Only TOP-LEVEL defs (the `_data[type]` entries) recurse into their
        alternates/sub-tiles. An alternate/sub-tile node's OWN nested lists are
        not collected: in C# they are shared-reference cycles (CopyFrom shares
        the alternates/sub-tile list, then addAlternate/addSubTile appends to
        it — an alternate can contain itself), and `GetTileData`
        (:5126-5161) resolves only ONE level (base -> sub-tile -> alternate),
        never reading a child's nested lists. Rust statics cannot be cyclic, so
        the children are emitted with their scalar fields and EMPTY nested
        lists — behaviourally identical for every consumer (can_place/place/
        get_tile_data read only the base's lists and one resolution level)."""
        if id(obj) in self.names:
            return self.names[id(obj)]
        self.names[id(obj)] = name
        if is_top:
            alts = obj.modules["alternates"]["data"] or []
            for i, alt in enumerate(alts):
                self.collect(alt, f"{name}_A{i + 1}", is_top=False)
            subs = obj.modules["subTiles"]["data"] or []
            for style, sub in enumerate(subs):
                if sub is not None:
                    self.collect(sub, f"{name}_S{style}", is_top=False)
        self.order.append((name, obj))
        return name

    def emit_def(self, name, obj, kind, owner):
        g = obj.get
        # Nested lists are emitted only for top-level defs (kind is not None);
        # alternates/sub-tiles carry empty lists (the C# shared-reference cycles
        # GetTileData never recurses into — see collect()).
        is_top = kind is not None
        alts = (obj.modules["alternates"]["data"] or []) if is_top else []
        subs = (obj.modules["subTiles"]["data"] or []) if is_top else []
        alt_refs = ", ".join(self.names[id(a)] for a in alts)
        sub_entries = []
        for style, sub in enumerate(subs):
            if sub is not None:
                sub_entries.append(f"({style}, &{self.names[id(sub)]})")
        gs = obj.modules["hooks"]["getStyleMethod"]
        gs_tag = "None" if gs is None else GET_STYLE_TAGS[gs[1]]
        # `frame_important` is `Main.tileFrameImportant[type]` — a per-TYPE
        # static in C#, not per-def: nested alternates/sub-tiles inherit the
        # owning tile's flag (the hand defs inherited it through `..BASE`).
        # `interaction` is the port's per-type dispatch overlay, likewise.
        eff_kind = kind if kind is not None else owner
        interact = INTERACTION.get(eff_kind, "None")
        fi = self.fi[eff_kind] if eff_kind is not None else False
        self.lines.append(f"static {name}: TileObjectDef = TileObjectDef {{")
        self.lines.append(f"    kind: {kind if kind is not None else 0},")
        self.lines.append(f"    width: {g('Width')},")
        self.lines.append(f"    height: {g('Height')},")
        ox, oy = g("Origin")
        self.lines.append(f"    origin: Point16::new({ox}, {oy}),")
        self.lines.append(f"    direction: {['TileObjectDirection::None', 'TileObjectDirection::PlaceLeft', 'TileObjectDirection::PlaceRight'][g('Direction')]},")
        self.lines.append(f"    random_style_range: {g('RandomStyleRange')},")
        self.lines.append(f"    specific_random_styles: {emit_int_list(g('SpecificRandomStyles'))},")
        self.lines.append(f"    flatten_anchors: {rust_bool(g('FlattenAnchors'))},")
        self.lines.append(f"    anchor_top: {emit_anchor(g('AnchorTop'))},")
        self.lines.append(f"    anchor_bottom: {emit_anchor(g('AnchorBottom'))},")
        self.lines.append(f"    anchor_left: {emit_anchor(g('AnchorLeft'))},")
        self.lines.append(f"    anchor_right: {emit_anchor(g('AnchorRight'))},")
        self.lines.append(f"    anchor_wall: {rust_bool(g('AnchorWall'))},")
        self.lines.append(f"    anchor_valid_tiles: {emit_int_list(g('AnchorValidTiles'))},")
        self.lines.append(f"    anchor_invalid_tiles: {emit_int_list(g('AnchorInvalidTiles'))},")
        self.lines.append(f"    anchor_alternate_tiles: {emit_int_list(g('AnchorAlternateTiles'))},")
        self.lines.append(f"    anchor_valid_walls: {emit_int_list(g('AnchorValidWalls'))},")
        self.lines.append(f"    water_placement: {['LiquidPlacement::Allowed', 'LiquidPlacement::NotAllowed', 'LiquidPlacement::OnlyInLiquid', 'LiquidPlacement::OnlyInFullLiquid'][g('WaterPlacement')]},")
        self.lines.append(f"    lava_placement: {['LiquidPlacement::Allowed', 'LiquidPlacement::NotAllowed', 'LiquidPlacement::OnlyInLiquid', 'LiquidPlacement::OnlyInFullLiquid'][g('LavaPlacement')]},")
        self.lines.append(f"    water_death: {rust_bool(g('WaterDeath'))},")
        self.lines.append(f"    lava_death: {rust_bool(g('LavaDeath'))},")
        self.lines.append(f"    uses_custom_can_place: {rust_bool(obj.uses_custom)},")
        self.lines.append(f"    uses_global_liquid_checks: {rust_bool(obj.uses_global_liquid)},")
        self.lines.append(f"    get_style_override: GetStyleOverride::{gs_tag},")
        self.lines.append(f"    hook_check_if_can_place: {emit_hook(obj.modules['hooks']['check'])},")
        self.lines.append(f"    hook_place_override: {emit_hook(obj.modules['hooks']['placeOverride'])},")
        self.lines.append(f"    hook_post_place_my_player: {emit_hook(obj.modules['hooks']['postPlaceMyPlayer'])},")
        heights = g("CoordinateHeights")
        self.lines.append(f"    coordinate_heights: {emit_int_list(heights)},")
        self.lines.append(f"    coordinate_width: {g('CoordinateWidth')},")
        self.lines.append(f"    coordinate_padding: {g('CoordinatePadding')},")
        fx, fy = g("CoordinatePaddingFix")
        self.lines.append(f"    coordinate_padding_fix: Point16::new({fx}, {fy}),")
        self.lines.append(f"    style: {g('Style')},")
        self.lines.append(f"    style_horizontal: {rust_bool(g('StyleHorizontal'))},")
        self.lines.append(f"    style_wrap_limit: {g('StyleWrapLimit')},")
        self.lines.append(f"    style_multiplier: {g('StyleMultiplier')},")
        self.lines.append(f"    style_line_skip: {g('StyleLineSkip')},")
        self.lines.append(f"    alternates: &[{alt_refs}],")
        self.lines.append(f"    subtiles: &[{', '.join(sub_entries)}],")
        self.lines.append(f"    frame_important: {rust_bool(fi)},")
        self.lines.append(f"    interaction: Interaction::{interact},")
        self.lines.append("};")
        self.lines.append("")


def generate(root: Path):
    src = (root / "decompiled" / "Terraria" / "ObjectData" / "TileObjectData.cs").read_text(
        encoding="utf-8")
    main_src = (root / "decompiled" / "Terraria" / "Main.cs").read_text(encoding="utf-8")
    tile_id_src = (root / "decompiled" / "Terraria" / "ID" / "TileID.cs").read_text(
        encoding="utf-8")
    m = re.search(r"TileID\.Count\s*=\s*(\d+)\s*;", tile_id_src)
    assert m and int(m.group(1)) == TILE_COUNT, "TileID.Count drift"

    interp = parse_initialize(src)
    fi = parse_frame_important(main_src)
    wg_src = (root / "decompiled" / "Terraria" / "WorldGen.cs").read_text(encoding="utf-8")
    no_cell_drop = parse_no_cell_drop(wg_src)

    em = Emitter(fi)
    top = []
    for kind in range(TILE_COUNT):
        obj = interp.data[kind]
        if obj is None:
            continue
        name = em.collect(obj, f"D_{kind}")
        top.append((kind, name))
        # Propagate the owning kind to every reachable child (alternates and
        # sub-tiles): fi/interaction are per-TYPE overlays in the port.
        stack = [obj]
        seen = set()
        while stack:
            o = stack.pop()
            if id(o) in seen:
                continue
            seen.add(id(o))
            em.owner[id(o)] = kind
            stack.extend(o.modules["alternates"]["data"] or [])
            stack.extend(s for s in (o.modules["subTiles"]["data"] or []) if s)

    # Sanity: the C# place math reads coordinate_heights[j] for j < Height
    # (TileObject.cs:94-110); flag any def whose rows do not cover the height.
    short = []
    for name, obj in em.order:
        if len(obj.get("CoordinateHeights")) < obj.get("Height"):
            short.append(name)

    out = []
    out.append("// @generated by tools/GenTileObjectData/gen.py from the decompiled Terraria")
    out.append("// sources (TileObjectData.cs Initialize :2110-5014, interpreted 1:1 — see the")
    out.append("// generator doc for the C# semantics). Do not edit by hand.")
    out.append("")
    out.append("//! The full `TileObjectData` registry: every `addTile` of `Initialize`")
    out.append("//! (:2110-5014), resolved through the copy-on-write builder chain.")
    out.append("//! `frame_important` rides `Main.tileFrameImportant`; `interaction` is the")
    out.append("//! port's dispatch overlay (gen.py INTERACTION, Player.cs citations).")
    out.append("")
    out.append("use super::{")
    out.append("    AnchorData, GetStyleOverride, Interaction, LiquidPlacement, PlacementHook,")
    out.append("    Point16, TileObjectDef, TileObjectDirection,")
    out.append("};")
    out.append("use crate::tiledata::TILE_COUNT;")
    out.append("")
    em.lines = out
    for name, obj in em.order:
        kind = None
        for k, n in top:
            if n == name:
                kind = k
                break
        em.emit_def(name, obj, kind, em.owner.get(id(obj)))

    out.append("/// Every registered tile object, ascending tile kind (`_data` :2119-2123")
    out.append("/// with the null slots removed).")
    out.append("pub static TABLE: &[TileObjectDef] = &[")
    for kind, name in top:
        out.append(f"    {name},")
    out.append("];")
    out.append("")
    out.append("/// `_data[type]` (:5136) — the per-kind lookup, `None` = unregistered.")
    out.append("pub static BY_KIND: [Option<&'static TileObjectDef>; TILE_COUNT] = [")
    by_kind = {k: n for k, n in top}
    for kind in range(TILE_COUNT):
        if kind in by_kind:
            out.append(f"    Some(&{by_kind[kind]}),")
        else:
            out.append("    None,")
    out.append("];")
    out.append("")
    out.append("/// `Main.tileFrameImportant` (Main.cs assignments + AddEchoFurnitureTile")
    out.append("/// :8412-8431 + the 435..439 loop :9986-9994).")
    out.append("pub static FRAME_IMPORTANT: [bool; TILE_COUNT] = [")
    for i in range(0, TILE_COUNT, 16):
        row = ", ".join(rust_bool(v) for v in fi[i:i + 16])
        out.append(f"    {row},")
    out.append("];")
    out.append("")
    out.append("/// `WorldGen.KillTile_GetItemDrops`' empty case group (`WorldGen.cs:63988-66090`,")
    out.append("/// the labels closed by a bare `break;`): these kinds drop nothing per-cell —")
    out.append("/// their loot rides the `Check*` cascade record / the pot event / the RNG rolls.")
    out.append("/// The `cell_drops` gate (the wooden slice used `def_for_tile(...).is_some()`).")
    out.append("pub static KILL_TILE_NO_CELL_DROP: [bool; TILE_COUNT] = [")
    for i in range(0, TILE_COUNT, 16):
        row = ", ".join(rust_bool(v) for v in no_cell_drop[i:i + 16])
        out.append(f"    {row},")
    out.append("];")
    out.append("")

    stats = {
        "tiles": len(top),
        "defs": len(em.order),
        "alternates": interp.counts["addAlternate"],
        "subtile_calls": interp.counts["addSubTile"],
        "base_styles": interp.counts["addBaseTile"],
        "short_height_rows": short,
        "frame_important": sum(fi),
        "no_cell_drop": sum(no_cell_drop),
    }
    return "\n".join(out), stats


def main():
    args = [a for a in sys.argv[1:]]
    check = "--check" in args
    args = [a for a in args if a != "--check"]
    root = Path(args[0]) if args else Path(__file__).resolve().parents[2]
    text, stats = generate(root)
    out_path = root / "crates" / "tr-core" / "src" / "tileobject" / "registry.rs"
    if check:
        if not out_path.exists():
            print(f"drift: {out_path} is missing", file=sys.stderr)
            return 1
        # Universal-newline read (the repo is autocrlf=true: the working tree
        # may be CRLF while the generator emits LF — compare content, not EOL).
        current = out_path.read_text(encoding="utf-8")
        if current != text:
            cur, fresh = current.split("\n"), text.split("\n")
            for i in range(max(len(cur), len(fresh))):
                a = cur[i] if i < len(cur) else "<eof>"
                b = fresh[i] if i < len(fresh) else "<eof>"
                if a != b:
                    print(f"drift: registry.rs:{i + 1}", file=sys.stderr)
                    print(f"  committed: {a.strip()}", file=sys.stderr)
                    print(f"  generated: {b.strip()}", file=sys.stderr)
                    break
            return 1
        print(f"check: registry.rs is current ({stats['tiles']} tiles, "
              f"{stats['defs']} defs)")
        return 0
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(text, encoding="utf-8", newline="\n")
    print(f"registry: {stats['tiles']} tiles, {stats['defs']} defs "
          f"({stats['alternates']} addAlternate, {stats['subtile_calls']} addSubTile, "
          f"{stats['base_styles']} base styles), "
          f"frame_important {stats['frame_important']}/{TILE_COUNT}")
    if stats["short_height_rows"]:
        print(f"note: coordinate_heights shorter than Height (C# reads past the "
              f"array): {', '.join(stats['short_height_rows'])}")
    print(f"output: {out_path}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
