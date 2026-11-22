# W4-d35 — NPC kinds 71-140 (data-only SetDefaults wave)

Branch task/W4-d35-npc-kinds-71-140 @ a725acb0 (= origin/main). No in-flight sibling commits (d34/d36 both 0 ahead).
33 missing kinds (76 = NPCID.None2 has NO SetDefaults arm — skipped). f17 census (17c3f998, docs/research/65-kinds-coverage-1.md) list matches the model EXACTLY.

## VERIFIED corrections to the in-flight generator (probe3/10/15/16/33/39/41/42)
- **Protocol band 429..461** (NOT 237..269): to_u16 owns 250 (AngryNimbus), render_key owns 261/263/264/265 (FungiSpore/PlanterasHook/PlanterasTentacle/Spore). Combined free band 429..488; take first 33. Verified free in to_u16/from_u16/render_key. render_key = to_u16 (port band), NOT vanilla.
- **Stats verbatim-verified** vs NPC.cs SetDefaults (_d35_verbatim.py): all 33 match. The 3 bat noGravity (93/121/137) are aiStyle-14 AI-head writes (NPC.cs:22851), matching the CaveBat/JungleBat/Hellbat/VoodooDemon precedent in no_gravity(). Display names match en-US NPCs.json; NPCID constants match; debuff rows match NPCID.cs DebuffImmunitySets (_d35_debuff.json).
- **Bestiary Hide-folds** (C# GetExclusions, BestiaryDatabaseNPCsPopulator.cs:869-886 + NPCID.cs DrawOffset Hide=true): 96/97->95 DiggerHead, 99/100->98 SeekerHead, 112->94 Corruptor. These get NO own ENTRIES row, NO cs_id hand-row (fallback rides the head entry), and fold in BOTH try_entry_for_npc + entry_for_npc (the Wyvern-segment precedent). 116 TheHungryII is NOT hidden (only a Position draw-modifier) -> keeps its OWN entry (credit TheHungryII, net 116, name "The Hungry"); the populator {116,113} is a UIInfoProvider remap, not a credit fold (Sharkron precedent).
- **texture_net_id**: Hide kinds get explicit VANILLA rows (96=>96 etc.) because net_id() folds to the head (the DetonatingBubble/Sharkron2/ForceBubble precedent). Visible kinds ride the `_ => kind.net_id()` wildcard (= vanilla).
- **Style-axis guards (data-only)**: axis arms 5 (ai_005_hover_family, has unreachable!()) and 9 (ai_009_sphere_family, would RUN) are claimed unconditionally. Guard them with `if !matches!(self.kind, <batch Corruptor/VileSpit>)` so the new kinds fall through to the update_kind_ai no-op (the MourningWood/Everscream 57-guard precedent). ai_005_hover_family gets a Corruptor => {} no-op (Probe precedent) to keep its inner match exhaustive. All other our-styles (2,3,6,13,14,16,18,22,23,24,25,26) are UNCLAIMED in the axis -> fall through automatically.
- **CS_STYLES style-5 pin** (npc.rs:31218): add Corruptor to the `five` vec after Probe (batch B).
- **Roster counts**: ALL_KINDS 270->303 (+5/batch), s1_f_3 covered+sentinel 270->303 (+5/batch). No other roster pins (probe43).
- **no_gravity**: worms (kb 0) EXCLUDED (knockback gate refuses resist 0; WyvernHead absent precedent). Flyers kb>0 included.
- **Sheets deferred**: NPC_<id> NOT packed (atlas 8192 bound, wyvern precedent); npc_sheet returns None, draw skips. BESTIARY_NPC_SHEET_IDS unchanged (181 rows); portrait_data arms rest the SetDefaults box (wyvern precedent), entry_sheet None == tabled False holds.

## Kinds (vanilla id, variant, aiStyle, box, dmg/def/life, kb, value, scale, alpha, hit/death, debuff)
73 GoblinScout 3 18x40 20/6/80 .7 200 s.95 h1/d1 null
74 Bird 24 14x14 0/0/5 .8 0 h1/d1 [31] catchItem2015
75 Pixie 22 20x20 55/20/150 .6 350 h5/d7 noGrav null
77 ArmoredSkeleton 3 18x40 40/28/260 .4 400 h2/d2 [20]
78 Mummy 3 18x40 50/16/130 .6 600 h1/d6 null
79 DarkMummy 3 18x40 60/18/180 .5 700 h1/d6 null
80 LightMummy 3 18x40 55/18/200 .55 700 h1/d6 null
82 Wraith 22 24x44 65/16/160 .7 500 a100 h54/d52 noGrav+noTileCollide ImmuneAllNonWhip(->&[])
83 CursedHammer 23 40x40 80/18/200 .4 1000 h4/d6 [20,24,31,323]
84 EnchantedSword 23 40x40 80/18/200 .4 1000 h4/d6 [20,24,31,323]
85 Mimic 25 24x24 80/30/500 .3 100000 h4/d6 rarity4 [20,31] (pre-hardmode 30/12/300/200 unported-no-flag)
86 Unicorn 26 46x42 65/30/400 .3 1000 h12/d18 null
93 GiantBat 14 26x20 45/16/100 .75 400 h1/d4 noGrav null
94 Corruptor 5 44x44 60/32/230 .55 500 h1/d1 noGrav [31]
95 DiggerHead 6 22x22 45/10/200 0 300 s.9 h1/d1 noGrav+noTileCollide+behindTiles+netAlways [31]
96 DiggerBody 6 22x22 28/20/200 0 300 s.9 dontCountMe [31] (Hide)
97 DiggerTail 6 22x22 26/30/200 0 300 s.9 dontCountMe [31] (Hide)
98 SeekerHead 6 22x22 70/36/500 0 700 h1/d1 noGrav+noTileCollide+behindTiles+netAlways npcSlots3.5 [31]
99 SeekerBody 6 22x22 55/40/500 0 700 dontCountMe [31] (Hide)
100 SeekerTail 6 22x22 40/44/500 0 700 dontCountMe [31] (Hide)
101 Clinger 13 30x30 70/30/320 .2 600 h1/d1 noGrav+noTileCollide+behindTiles [39,31]
102 AnglerFish 16 18x20 80/22/90 (no kb->1) 500 h1/d1 noGrav [31]
103 GreenJellyfish 18 26x26 80/30/120 (no kb->1) 800 a20 h25/d28 noGrav [31]
104 Werewolf 3 18x40 70/38/350 .4 1000 h6/d1 null
109 Clown 3 34x78 60/25/800 .2 10000 h1/d2 rarity1 [31]
110 SkeletonArcher 3 18x40 45/14/210 .55 400 h2/d2 [20]
112 VileSpit 9 16x16 65/0/1 0 0 s.9 a80 hitNULL/d9 noGrav+noTileCollide [31] (Hide)
116 TheHungryII 2 30x32 30/6/80 .8 0 h9/d12 [31]
120 ChaosElemental 3 18x40 40/30/370 .4 600 h1/d6 [20]
121 Slimer 14 40x30 45/20/60 .8 0 s1.1 a55 h1/(no death) [20]
122 Gastropod 22 20x20 60/22/220 .8 600 h1/d1 noGrav [20,31]
137 IlluminantBat 14 26x20 75/30/200 .75 500 h1/d6 [20]
140 PossessedArmor 3 18x40 55/28/260 .4 400 h4/d6 [20,24,323]

## Touch-points per kind (Wyvern precedent, ANALYSIS "The Wyvern family")
npc.rs: enum variant(+doc); defense; value; knock_back_resist; update_kind_ai(no-op); find_frame(no-op); render_key(=to_u16); ALL_KINDS[270->303]; ctor scale/alpha rows.
npc/data.rs: to_u16/from_u16(237..269); ai_style; size; max_hp; contact_damage; hit_sound/death_sound; cs_id(hand row); behind_tiles(worms+clinger); no_gravity(kb>0 flyers); shares_carrier_life(worms).
npc_buffs.rs: debuff_immune_ids.
bestiary.rs: ENTRIES (own entry for visible; fold for Hide: 96/97->95,99/100->98,112->94,116->115Hungry); try_entry_for_npc+entry_for_npc folds.
app/netglue.rs: npc_seed (walker/hover ctor).
render/entities.rs: texture_net_id(=vanilla for folded); draw_npcs arm (deferred sheets, wyvern precedent); rotation whitelist (worms).
ui/bestiary.rs: portrait_data arm (SetDefaults box).
assets.rs: BESTIARY_NPC_SHEET_IDS + bestiary_npc_sheets (visible kinds, deferred None).
audio.rs: bank rows NPC_Hit_5/54, NPC_Killed_7/9/28/52.
npc.rs tests: CS_STYLES rows; s1_f_3 count 270->303; new stat-pin witness module.
anchors.txt: row. docs/ANALYSIS.md: deviation note.

## Batches (5/commit+push, the W4-d35 dispatch rule):
A=73,74,75,77,78; B=79,80,82,83,84; C=85,86,93,94,95; D=96,97,98,99,100; E=101,102,103,104,109; F=110,112,116,120,121; G=122,137,140.
Data-only: no AI wiring (update_kind_ai/find_frame no-op), sheets deferred (atlas headroom), no spawner.
NOTE: batch C carries Corruptor (axis-5 guard) + DiggerHead; batch D the four Hide worm segments; batch F VileSpit (axis-9 guard) + TheHungryII. The axis guards + style-5 pin land with their kind's batch.
