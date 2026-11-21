#!/usr/bin/env python3
"""Generate crates/tr-app/src/bestiary_tags.rs from the decompiled
BestiaryDatabaseNPCsPopulator (Terraria 1.4.5.8) — W4-e23 data half.

Source of truth: decompiled/Terraria/GameContent/Bestiary/BestiaryDatabaseNPCsPopulator.cs
  * CommonTags.SpawnConditions static ctors (Biomes/Events/Invasions/Times)
    — the (language key, filter icon frame) pairs of FilterProviderInfoElement.
  * CommonTags.GetCommonInfoElementsForFilters (:327-388) — the filter order.
  * AddNPCBiomeRelationships_Automated (:888-1746) +
    AddNPCBiomeRelationships_Manual (:1747-2027) — the per-entry tag walk,
    AddRange in order + the eleven Info.Remove rows.
  * ModifyEntriesThatNeedIt (:601-748) — the preference-wrapped tags
    (BestiaryPortraitBackgroundProviderPreferenceInfoElement) + search aliases.
Also: decompiled/Terraria/ID/NPCID.cs
  * Sets.TownNPCBestiaryPriority / NormalGoldCritterBestiaryPriority /
    BossBestiaryPriority (:10088-10113) — the sorting-id priority lists.

Run from the repo root:  python tools/gen_bestiary_tags.py [--check]
The output is a cited static table; never hand-edit it.
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SRC = ROOT / "decompiled" / "Terraria" / "GameContent" / "Bestiary" / "BestiaryDatabaseNPCsPopulator.cs"
NPCID = ROOT / "decompiled" / "Terraria" / "ID" / "NPCID.cs"
OUT = ROOT / "crates" / "tr-app" / "src" / "bestiary_tags.rs"

text = SRC.read_text(encoding="utf-8")
lines = text.split("\n")


def lineno(pos):
    return text.count("\n", 0, pos) + 1


# --- the tag catalog: Family.Name -> (key, frame) ---------------------------
tags = {}  # "Biomes.Surface" -> (key, frame, ctor_line)
for m in re.finditer(
    r"(Biomes|Events|Invasions|Times)\.(\w+) = new SpawnCondition\w+\(\"([^\"]+)\", (\d+)",
    text,
):
    fam, name, key, frame = m.groups()
    tags[f"{fam}.{name}"] = (key, int(frame), lineno(m.start()))
assert len(tags) >= 60, f"tag catalog too small: {len(tags)}"

# --- the portrait background providers (W4-e23 remainder) --------------------
# `SpawnConditionBestiaryInfoElement`'s optional backgroundImagePath /
# backgroundColor ctor args (SpawnConditionBestiaryInfoElement.cs:15-20):
# the biome backdrop the info-page portrait draws
# (UIBestiaryNPCEntryPortrait.cs:30-41). Overlay elements
# (SpawnConditionBestiaryOverlayInfoElement) carry an OVERLAY path, not a
# background — they are not background providers, so they stay None here.
tag_bg = {}  # "Biomes.Surface" -> (path_or_None, color_or_None)
for m in re.finditer(
    r"(Biomes|Events|Invasions|Times)\.(\w+) = new (SpawnCondition\w+)\("
    r"\"([^\"]+)\", (\d+)"
    r"(?:, \"([^\"]+)\")?"
    r"(?:, new Color\((\d+), (\d+), (\d+)\))?",
    text,
):
    fam, name, cls, key, frame, path, cr, cg, cb = m.groups()
    full = f"{fam}.{name}"
    if cls != "SpawnConditionBestiaryInfoElement":
        tag_bg[full] = (None, None)  # overlay elements: no background image
        continue
    # Strip the "Images/" prefix — the atlas sheet names are bare ("MapBG1").
    sheet = path[len("Images/"):] if path else None
    color = f"[{cr}, {cg}, {cb}, 255]" if cr is not None else None
    tag_bg[full] = (sheet, color)

# --- the filter order (GetCommonInfoElementsForFilters) ---------------------
m = re.search(r"GetCommonInfoElementsForFilters\(\)\s*\{.*?return new List<IBestiaryInfoElement>\s*\{(.*?)\};", text, re.S)
assert m, "filter list not found"
filter_order = re.findall(r"SpawnConditions\.(\w+)\.(\w+)", m.group(1))
filter_order = [f"{a}.{b}" for a, b in filter_order]
assert len(filter_order) == 57, f"filter count {len(filter_order)} != 57"
filter_list_line = lineno(m.start())

# --- the per-entry tag walk --------------------------------------------------
# The relationship passes span :888-2027; ModifyEntriesThatNeedIt :601-748
# adds preference-wrapped tags. Both are AddRange/AddTags of CommonTags rows
# plus eleven Info.Remove rows. We replay them in SOURCE ORDER per entry.
events = []  # (phase, line, net_id, op, tag, preferred)
pat_add = re.compile(
    r"this\.FindEntryByNPCID\((-?\d+)\)\.Info\.AddRange\(new IBestiaryInfoElement\[\d+\]\s*\{(.*?)\}\)",
    re.S,
)
pat_addtags = re.compile(
    r"this\.FindEntryByNPCID\((-?\d+)\)\.AddTags\(new BestiaryPortraitBackgroundProviderPreferenceInfoElement\(CommonTags\.SpawnConditions\.(\w+)\.(\w+)\)\)",
)
pat_remove = re.compile(
    r"this\.FindEntryByNPCID\((-?\d+)\)\.Info\.Remove\(CommonTags\.SpawnConditions\.(\w+)\.(\w+)\)",
)


def in_ranges(ln, ranges):
    return any(a <= ln <= b for a, b in ranges)


REL_RANGES = [(888, 2027)]
MOD_RANGES = [(601, 748)]
for m in pat_add.finditer(text):
    ln = lineno(m.start())
    if not in_ranges(ln, REL_RANGES):
        continue
    net = int(m.group(1))
    for a, b in re.findall(r"CommonTags\.SpawnConditions\.(\w+)\.(\w+)", m.group(2)):
        events.append((0, ln, net, "add", f"{a}.{b}", False))
for m in pat_addtags.finditer(text):
    ln = lineno(m.start())
    if not in_ranges(ln, MOD_RANGES):
        continue
    events.append((1, ln, int(m.group(1)), "add", f"{m.group(2)}.{m.group(3)}", True))
for m in pat_remove.finditer(text):
    ln = lineno(m.start())
    if not in_ranges(ln, REL_RANGES + MOD_RANGES):
        continue
    phase = 1 if in_ranges(ln, MOD_RANGES) else 0
    events.append((phase, ln, int(m.group(1)), "remove", f"{m.group(2)}.{m.group(3)}", False))
# Populate (:544-555) runs the relationship passes BEFORE ModifyEntriesThatNeedIt,
# so the replay order is (phase, source line) — not raw line order.
events.sort(key=lambda e: (e[0], e[1]))

# --- search aliases (ModifyEntriesThatNeedIt :731-748) -----------------------
aliases = []  # (net_id, alias, line)
for m in re.finditer(
    r"this\.FindEntryByNPCID\((-?\d+)\)\.AddTags\(new SearchAliasInfoElement\(\"([^\"]+)\"\)\)", text
):
    aliases.append((int(m.group(1)), m.group(2), lineno(m.start())))

# --- the priority lists (NPCID.cs) -------------------------------------------
npcid = NPCID.read_text(encoding="utf-8")


def prio_list(name):
    m = re.search(r"Sets\." + name + r" = new List<int>\s*\{(.*?)\};", npcid, re.S)
    assert m, name
    ids = [int(x) for x in re.findall(r"-?\d+", m.group(1))]
    return ids, npcid.count("\n", 0, m.start()) + 1


town, town_ln = prio_list("TownNPCBestiaryPriority")
gold, gold_ln = prio_list("NormalGoldCritterBestiaryPriority")
boss, boss_ln = prio_list("BossBestiaryPriority")

# --- replay per entry ---------------------------------------------------------
# NOTE: the preference wraps (ModifyEntriesThatNeedIt's AddTags of
# BestiaryPortraitBackgroundProviderPreferenceInfoElement) add a DIFFERENT
# object to Info — not the tag singleton. Filters.ByInfoElement's
# `Info.Contains(element)` (Filters.cs:161-164) and GetLowestBiomeGroupIndex's
# `info.IndexOf` (ContentSamples.cs:780) are reference checks, so a wrapped
# tag does NOT count for filter membership / the sorting-id biome index; it
# only feeds the flavor-text preference walk. The `preferred` flag keeps the
# row for that future consumer; membership helpers skip it. `Info.Remove(tag)`
# likewise never removes a wrap.
per_entry = {}  # net -> list[(tag, preferred)]
for ln, net, op, tag, pref in ((e[1], e[2], e[3], e[4], e[5]) for e in events):
    lst = per_entry.setdefault(net, [])
    if op == "add":
        lst.append((tag, pref))
    else:
        # List.Remove removes the FIRST occurrence only (C# semantics), and
        # only a plain (non-wrapped) row can match the tag reference.
        for i, (t, p) in enumerate(lst):
            if t == tag and not p:
                del lst[i]
                break

# tag id assignment: filter order first (the C# GetLowestBiomeGroupIndex reads
# the common-filter list), then any remaining catalog tags in ctor order.
tag_names = list(filter_order)
for t in sorted(tags, key=lambda t: tags[t][2]):
    if t not in tag_names:
        tag_names.append(t)
tag_id = {t: i for i, t in enumerate(tag_names)}

# --- emit ---------------------------------------------------------------------
out = []
w = out.append
w("//! GENERATED by tools/gen_bestiary_tags.py — do not hand-edit.")
w("//!")
w("//! The bestiary spawn-condition tag data (W4-e23, the filters/sort data")
w("//! half): `CommonTags.SpawnConditions`' (language key, filter icon frame)")
w("//! catalog, the `GetCommonInfoElementsForFilters` filter order, the")
w("//! per-entry tag walk (`AddNPCBiomeRelationships_Automated`/`_Manual` +")
w("//! `ModifyEntriesThatNeedIt`'s preference wraps and `Info.Remove` rows")
w("//! replayed in source order), the search aliases and the three")
w("//! `NPCID.Sets.*BestiaryPriority` lists the sorting-id order reads")
w("//! (`ContentSamples.BestiaryHelper.GetSortedBestiaryEntriesList`,")
w("//! `ContentSamples.cs:765-773`).")
w("//!")
w("//! Source: BestiaryDatabaseNPCsPopulator.cs + NPCID.cs (Terraria 1.4.5.8).")
w("")
w("/// One `SpawnConditionBestiaryInfoElement` tag")
w("/// (`FilterProviderInfoElement.cs:24-29`): the language key + the")
w("/// `Icon_Tags_Shadow` frame index (`frame % 16`, `frame / 16` on the")
w("/// 16x5 sheet), plus the portrait background provider's image sheet +")
w("/// tint (`SpawnConditionBestiaryInfoElement.cs:15-20` — the optional")
w("/// `backgroundImagePath`/`backgroundColor` ctor args; overlay elements")
w("/// carry no background). W4-e23 remainder: the biome portrait backdrops.")
w("#[derive(Clone, Copy, Debug, PartialEq, Eq)]")
w("pub struct TagDef {")
w("    pub key: &'static str,")
w("    pub frame: u32,")
w("    /// `GetBackgroundImage`'s sheet name (the `Images/` prefix stripped),")
w("    /// `None` when the tag draws no backdrop.")
w("    pub bg: Option<&'static str>,")
w("    /// `GetBackgroundColor`'s tint, `None` = `Color.White`.")
w("    pub bg_color: Option<[u8; 4]>,")
w("}")
w("")
w(f"/// The tag catalog in id order. Ids 0..{len(filter_order)-1} are the")
w(f"/// `GetCommonInfoElementsForFilters` list (`:{filter_list_line}`), the")
w("/// order `GetLowestBiomeGroupIndex` scans (`ContentSamples.cs:775-786`);")
w("/// the remaining ids are catalog tags outside the filter list (Oasis,")
w("/// SlimeRain, Blizzard), ctor order.")
w("pub const TAGS: &[TagDef] = &[")
for t in tag_names:
    key, frame, ln = tags[t]
    bg, bg_color = tag_bg.get(t, (None, None))
    bg_lit = f'Some("{bg}")' if bg else "None"
    color_lit = f"Some({bg_color})" if bg_color else "None"
    w(f'    TagDef {{ key: "{key}", frame: {frame}, bg: {bg_lit}, bg_color: {color_lit} }}, // {t} (:{ln})')
w("];")
w("")
w(f"/// The filter-list length — `AvailableFilters` carries the three named")
w(f"/// filters (ByUnlockState/ByBoss/ByRareCreature, `:583-585`) plus these")
w(f"/// {len(filter_order)} tag filters (`:586-590`).")
w(f"pub const COMMON_FILTER_COUNT: usize = {len(filter_order)};")
w("")
w("/// One entry's tag: the tag id + whether the populator wrapped it in a")
w("/// `BestiaryPortraitBackgroundProviderPreferenceInfoElement`")
w("/// (`ModifyEntriesThatNeedIt:603-636` — the preferred background wins the")
w("/// portrait pick, `UIBestiaryEntryButton.cs:104-129`).")
w("#[derive(Clone, Copy, Debug, PartialEq, Eq)]")
w("pub struct EntryTag {")
w("    pub tag: u32,")
w("    pub preferred: bool,")
w("}")
w("")
w("/// The per-entry tag lists, net-id ascending — the relationship passes'")
w("/// AddRange/AddTags replayed in source order with the eleven")
w("/// `Info.Remove` rows applied (first-occurrence removal, C# `List.Remove`).")
w("/// Entries without tags carry no row.")
w("pub static ENTRY_TAGS: &[(i32, &[EntryTag])] = &[")


def et(tag, pref):
    return f"EntryTag {{ tag: {tag_id[tag]}, preferred: {'true' if pref else 'false'} }}"


for net in sorted(per_entry):
    lst = per_entry[net]
    if not lst:
        continue
    row = ", ".join(et(t, p) for t, p in lst)
    w(f"    ({net}, &[{row}]),")
w("];")
w("")
w("/// `SearchAliasInfoElement` rows (`ModifyEntriesThatNeedIt:731-748`):")
w("/// the raw alias string searches match once the entry is known")
w("/// (`SearchAliasInfoElement.cs:14-21`).")
w("pub static SEARCH_ALIASES: &[(i32, &str)] = &[")
for net, alias, ln in sorted(aliases, key=lambda a: (a[0], a[2])):
    w(f'    ({net}, "{alias}"), // :{ln}')
w("];")
w("")
for name, ids, ln in [
    ("TOWN_NPC_BESTIARY_PRIORITY", town, town_ln),
    ("NORMAL_GOLD_CRITTER_BESTIARY_PRIORITY", gold, gold_ln),
    ("BOSS_BESTIARY_PRIORITY", boss, boss_ln),
]:
    w(f"/// `NPCID.Sets.{name.title().replace('_','')}` (`NPCID.cs:{ln}`) —")
    w("/// the sorting-id priority list; `IndexOf` gives the rank, -1 maps to")
    w("/// `int.MaxValue` for the town/critter lists (`ContentSamples.cs:802-820`)")
    w("/// but stays -1 for the boss list (`:822-825`).")
    w(f"pub const {name}: &[i32] = &[")
    for i in range(0, len(ids), 10):
        w("    " + ", ".join(str(x) for x in ids[i : i + 10]) + ",")
    w("];")
    w("")
w("/// The entry's tag list (empty when the populator adds none).")
w("pub fn entry_tags(net_id: i32) -> &'static [EntryTag] {")
w("    ENTRY_TAGS")
w("        .binary_search_by_key(&net_id, |&(id, _)| id)")
w("        .map(|i| ENTRY_TAGS[i].1)")
w("        .unwrap_or(&[])")
w("}")
w("")
w("/// `GetLowestBiomeGroupIndex` (`ContentSamples.cs:775-786`): scan the")
w("/// common-filter list BACKWARD, return the first index whose tag the")
w("/// entry carries; `int.MaxValue` when none matches. Only ids below")
w("/// `COMMON_FILTER_COUNT` count (the non-filter catalog tags never match);")
w("/// preference-wrapped rows never match (the wrap is a different object,")
w("/// `info.IndexOf` is a reference check).")
w("pub fn lowest_biome_group_index(net_id: i32) -> i32 {")
w("    let tags = entry_tags(net_id);")
w("    for i in (0..COMMON_FILTER_COUNT).rev() {")
w("        if tags.iter().any(|t| t.tag == i as u32 && !t.preferred) {")
w("            return i as i32;")
w("        }")
w("    }")
w("    i32::MAX")
w("}")
w("")
w("/// `Filters.ByInfoElement.FitsFilter` (`Filters.cs:161-164`): the entry's")
w("/// Info contains the tag singleton — preference wraps do not count.")
w("pub fn has_tag(net_id: i32, tag: u32) -> bool {")
w("    entry_tags(net_id)")
w("        .iter()")
w("        .any(|t| t.tag == tag && !t.preferred)")
w("}")
w("")
w("/// The `IndexOf`-or-`int.MaxValue` rank of the town/critter priority")
w("/// lists (`ContentSamples.cs:802-820`).")
w("fn priority_or_max(list: &[i32], net_id: i32) -> i32 {")
w("    list.iter()")
w("        .position(|&x| x == net_id)")
w("        .map(|i| i as i32)")
w("        .unwrap_or(i32::MAX)")
w("}")
w("")
w("/// `GetBestiaryTownPriority` (`ContentSamples.cs:802-810`).")
w("pub fn town_priority(net_id: i32) -> i32 {")
w("    priority_or_max(TOWN_NPC_BESTIARY_PRIORITY, net_id)")
w("}")
w("")
w("/// `GetBestiaryNormalGoldCritterPriority` (`ContentSamples.cs:812-820`).")
w("pub fn gold_critter_priority(net_id: i32) -> i32 {")
w("    priority_or_max(NORMAL_GOLD_CRITTER_BESTIARY_PRIORITY, net_id)")
w("}")
w("")
w("/// `GetBestiaryBossPriority` (`ContentSamples.cs:822-825`): the raw")
w("/// `IndexOf` — -1 stays -1 (bosses sort before everything else).")
w("pub fn boss_priority(net_id: i32) -> i32 {")
w("    BOSS_BESTIARY_PRIORITY")
w("        .iter()")
w("        .position(|&x| x == net_id)")
w("        .map(|i| i as i32)")
w("        .unwrap_or(-1)")
w("}")
w("")
w("/// The entry's search aliases (empty when none).")
w("pub fn search_aliases(net_id: i32) -> impl Iterator<Item = &'static str> {")
w("    SEARCH_ALIASES")
w("        .iter()")
w("        .filter(move |&&(id, _)| id == net_id)")
w("        .map(|&(_, a)| a)")
w("}")
w("")

rendered = "\n".join(out) + "\n"
if "--check" in sys.argv:
    old = OUT.read_text(encoding="utf-8") if OUT.exists() else ""
    if old.replace("\r\n", "\n") != rendered:
        sys.exit(f"{OUT} drifted — regenerate with tools/gen_bestiary_tags.py")
    print("bestiary_tags.rs is up to date")
else:
    OUT.write_text(rendered, encoding="utf-8", newline="\n")
    print(f"wrote {OUT}: {len(tag_names)} tags, {len(per_entry)} tagged entries, {len(aliases)} aliases")
