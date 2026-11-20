import sys, os
sys.path.insert(0, os.path.dirname(__file__))
from _d35_gen import rd, wr
ROOT = r"C:\Users\DreadLord\Desktop\TR-W4-d35"
ANALYSIS = r'docs\ANALYSIS.md'
ANCHORS = r'anchors.txt'

SECTION = """
## The W4-d35 kinds wave (NPC 71-140, data-only SetDefaults — code-only mode)

The f17 kinds-coverage census (`docs/research/65-kinds-coverage-1.md`,
commit `17c3f998`) lists **34 missing ids in the 71-140 band**; 76 is
`NPCID.None2` (a placeholder with NO `SetDefaults` arm — skipped), leaving
**33 real kinds**. This wave lands their **data only** — every exhaustive
`NpcKind` match gains arms (the Wyvern kind-wave precedent), but the AI is
**UNWIRED** (`update_kind_ai`/`find_frame` no-ops; the AI lands with its own
wave). Branch `task/W4-d35-npc-kinds-71-140`, NOT merged. Code-only: no
compiler invoked; the per-batch witness modules (`w4_d35_*_tests` in
`npc.rs`) are WRITTEN NOT RUN.

**Verbatim source**: every stat row pins to `NPC.cs` `SetDefaults`
(the per-kind citations ride the enum docs + the witness tables); the
debuff rows to `NPCID.cs` `DebuffImmunitySets`; the display names to the
en-US `NPCs.json` localization; the `NPCID` constants to `NPCID.cs`. A
scripted re-derivation (`tools/_d35_verbatim.py`) confirmed all 33 kinds'
width/height/aiStyle/damage/defense/lifeMax/knockBackResist/value/scale/
alpha/HitSound/DeathSound/rarity match the C# block exactly.

**Protocol band 429..461** (NOT the vanilla ids): the port-local `to_u16`
space already owns 250 (AngryNimbus) and `render_key` owns 261/263/264/265
(FungiSpore/PlanterasHook/PlanterasTentacle/Spore), so the in-flight plan's
237..269 band COLLIDES. The combined free band (to_u16 max 353, render_key
lunar ids <= 549) is 429..488; the 33 kinds take 429..461 (the pirate/wyvern
renumber precedent). `render_key` rides `to_u16`'s port band; the vanilla
`NPC.type` rides `cs_id`/`net_id`, NOT the protocol id.

**Bestiary Hide-folds** (C# `GetExclusions`,
`BestiaryDatabaseNPCsPopulator.cs:869-886` + `NPCID.cs` `NPCBestiaryDrawOffset`
`Hide = true`): the five hidden kinds — DiggerBody 96 / DiggerTail 97 ->
DiggerHead 95, SeekerBody 99 / SeekerTail 100 -> SeekerHead 98, VileSpit 112
-> Corruptor 94 — get **NO own `ENTRIES` row and NO `cs_id` hand-row** (the
`cs_id` fallback reads the folded head entry), and fold in BOTH
`try_entry_for_npc` + `entry_for_npc` (the Wyvern-segment precedent). Their
DRAW sheets stay their own `NPC_<id>` through explicit `texture_net_id` rows
(the DetonatingBubble/Sharkron2/ForceBubble precedent — `net_id()` folds to
the head, so the wildcard would mis-route the sheet). **TheHungryII 116 is
NOT hidden** (only a `Position` draw-modifier, `NPCID.cs:2035`): it keeps its
OWN entry (credit `TheHungryII`, net 116, name "The Hungry"); the populator's
`{116,113}` is a `UIInfoProvider` remap, not a credit fold (the Sharkron 372
precedent — its kill credit stays its own).

**Style-axis guards (data-only)**: the aiStyle dispatch's arms **5**
(`ai_005_hover_family`, whose inner match ends in `unreachable!()`) and **9**
(`ai_009_sphere_family`, which would RUN) are claimed unconditionally. The
new style-5 kind (Corruptor 94) and style-9 kind (VileSpit 112) would panic
or run AI, so those two arms gain `if !matches!(self.kind, ...)` guards (the
MourningWood/Everscream 57-guard precedent) that let the data-only kinds fall
through to the `update_kind_ai` no-op. `ai_005_hover_family` also gains a
Corruptor `=> {}` no-op (the Probe precedent) to keep its inner match
exhaustive. Every other aiStyle in the wave (2,3,6,13,14,16,18,22,23,24,25,26)
is UNCLAIMED in the axis, so those kinds fall through automatically. The
`CS_STYLES` style-5 roster pin (`s1_d_2` test) gains Corruptor after Probe.

**no_gravity**: the kb-0 worms (Digger/Seeker heads/bodies/tails) keep
`false` — the knockback gate refuses resist 0, so the static table models
only the knockback-relevant flyers (the WyvernHead-absent precedent). The
three aiStyle-14 bats (GiantBat 93, Slimer 121, IlluminantBat 137) carry
`noGravity = true` from the AI-head write (`NPC.cs:22851`), matching the
CaveBat/JungleBat/Hellbat/VoodooDemon rows — NOT a `SetDefaults` row.

**Documented deferrals** (the Wyvern precedent):
1. **Sheet deferral**: `NPC_<id>` are NOT packed (the atlas stands at the
   8192 strict-GPU bound); `npc_sheet` returns None and the draw skips.
   `BESTIARY_NPC_SHEET_IDS` is unchanged; the `portrait_data` arms rest the
   `SetDefaults` box (the wyvern portrait precedent). The packing +
   draw-modifier overrides ride the atlas-repack wave.
2. **AI wiring**: `update_kind_ai`/`find_frame` are no-ops; the family
   machines, the worm chain assembly (`shares_carrier_life` IS extended,
   inert until wiring), the spawner gates and the `npcSlots`/`netAlways`/
   `dontTakeDamageFromHostiles` live-state flags ride their own waves.
3. **Mimic 85 pre-hardmode branch** (`NPC.cs:9917-9923`: dmg 30 / def 12 /
   life 300 / value 200 when `!Main.hardMode`): the port has no runtime
   hardMode flag in the static table, so the hardmode row (80/30/500/100000)
   is the pinned default; the pre-hardmode downgrade rides the difficulty
   wiring (the ftw/expert static-table convention).

**Batches (5 kinds/commit)**: A=73,74,75,77,78; B=79,80,82,83,84;
C=85,86,93,94,95; D=96,97,98,99,100; E=101,102,103,104,109;
F=110,112,116,120,121; G=122,137,140.
"""

def add_analysis():
    src = rd(ANALYSIS)
    if '## The W4-d35 kinds wave' in src:
        print('analysis section already present'); return
    anchor = "\n## The Destroyer machine (aiStyle-37, S1-D-3, 2026-11-18 — code-only mode)"
    assert src.count(anchor) == 1, src.count(anchor)
    wr(ANALYSIS, src.replace(anchor, SECTION + anchor))
    print('analysis section added')

def add_anchor():
    src = rd(ANCHORS)
    marker = 'mod w4_d35_a_tests'
    if marker in src:
        print('anchor already present'); return
    row = ('crates/tr-app/src/npc.rs|mod w4_d35_a_tests|W4-d35 NPC kinds 71-140 data wave - '
           'the 33 missing SetDefaults kinds (f17 census 65-kinds-coverage-1) landed data-only '
           '(verbatim NPC.cs rows, port-local protocol band 429..461, bestiary Hide-folds '
           '96/97->95 99/100->98 112->94, style-5/9 axis guards for Corruptor/VileSpit, '
           'sheets deferred); 7 batches A-G, per-batch witness modules written not run\n')
    if not src.endswith('\n'): src += '\n'
    wr(ANCHORS, src + row)
    print('anchor row added')

if __name__ == '__main__':
    add_analysis()
    add_anchor()
