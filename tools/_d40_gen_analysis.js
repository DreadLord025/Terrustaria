// W4-d40 sheets phase: generate the ANALYSIS.md section.
'use strict';
const fs = require('fs');
const path = require('path');
const w = JSON.parse(fs.readFileSync(path.join(__dirname, '_d40_wiring.json'), 'utf8'));
const packed = w.filter(o => o.packed);
const deferred = w.filter(o => !o.packed);
const rot = w.filter(o => o.rotates);
const tex = w.filter(o => o.needsTexRow);
const packedOwn = w.filter(o => o.packed && o.ownEntry);

const sec = `
## The W4-d40 sheets phase (NPC 421-490 band, 2026-11-21 — code-only mode)

The data wave (\`8fe1d6d3\`, 12 batches) landed the 56 kinds' identity rows
(enum, to_u16/from_u16, cs_id/net_id, SetDefaults stats, bestiary entries /
Hide-folds, debuff immunities, sound cues) but left the **draw match
non-exhaustive** (the 56 kinds had no \`draw_npcs\` arm) and the sheets
unpacked. This sheets phase wires the draw, packs the sheets that fit the
8192 strict-GPU bound, upgrades the portraits, and adds the rotation
whitelist + anchor nudges + frame-count witnesses. Branch
\`task/W4-d40-npc-kinds-421-490\`, NOT merged. Code-only: no cargo/tests run;
the witness tests (\`the_d40_rows_match_between_the_draw_and_the_machine\`,
\`the_d40_strips_divide_into_their_vanilla_rows\`, the packing-audit
extension) are WRITTEN NOT RUN.

### The atlas crisis, re-measured (the decisive fact)

The real retail \`.xnb\` dims for all 56 \`NPC_<id>\` sheets were measured by
the pinned c4 LZX harness (\`tools/_d40_atlas_sim_pinned.js\`, the b4 copy
whose L-8b fallback survives the wave-2 purge; \`tools/_d40_dims.js\` →
\`_d40_dims.json\`). The cumulative headroom
(\`tools/_d40_headroom.js\`, skyline waste tie-break, \`verify()\` +
double-run determinism on every scenario):

| scenario | sheets | skyline px | vs 8192 |
|---|---|---|---|
| base (HEAD a725acb0, gore landed) | 9497 | **8115** | headroom 77 |
| base + boot font | 9498 | **8119** | headroom 73 |
| + pushed union (35 branches, 403 names) | 9899 | **9361** | **OVERFLOW +1169** |
| + pushed + all 56 d40 | 9953 | **9772** | **OVERFLOW +1580** |

The pushed union (\`tools/_d40_pushed_extract.py\` →
\`_d40_pushed_sheets.json\`, re-extracted fresh so it includes d35's own 21
sheets-phase packs and d38's 35) **already overflows without this wave**.
The d35 ATLAS-REPACK plan (\`docs/research/113-atlas-repack-plan.md\`,
commits \`519c8035\`/\`7d131be3\`/\`239108c2\`/\`ed3c08d4\`/\`6a2a155a\`)
measured the same crisis including the d40 56-sheet forecast: the union's
padded **AREA alone (72.34 Mpx → 8835 px lower bound) exceeds 8192**, so
**no single-page packer — skyline, maxrects, or any other — can fit the
merged W4 wave. A second atlas page is mandatory, not optional.** The plan
explicitly says "d40's 56 land in its own sheets phase as normal" (§3 step
4) and "the d35 deferred 12 + d40's 56 simply join the input list". This
phase does exactly that: it packs the solo-fitting subset on HEAD and
documents the cumulative overflow for the atlas-repack merger wave.

### The solo pack: 36 of 56 (the d35 id-order greedy method)

On the HEAD base alone (no pushed union), the id-order greedy
(\`tools/_d40_headroom.js\` first pass + \`tools/_d40_pass2.js\` second pass,
keep iff BOTH main and font stay ≤ 8192) packs **36 sheets at 8185 px main
/ 8186 px boot-font** — 7/6 px under the bound, \`verify()\` + determinism
on the final layout (\`tools/_d40_final_pack.js\`). The 36:
430-438 (the ArmedZombie seven + CultistTablet/Devote), 442-448 (the gold
critters), 454-459 (the Cultist Dragon chain), 462/463/464/465 (Fritz /
Nailhead / CrimsonBunny / CrimsonGoldfish), 469/470/472 (ThePossessed /
CrimsonPenguin / ShadowFlameApparition), 478 (MothronEgg), 482
(GraniteGolem), 484-487 (EnchantedNightcrawler / Grubby / Sluggy / Buggy),
488 (TargetDummy).

The **20 deferred** (each would overflow at its greedy turn):
440 (CultistBossClone, +103), 449-452 (BoneThrowingSkeleton ×4, +118..172),
453 (SkeletonMerchant, +157), 460 (Butcher, +40), 461
(CreatureFromTheDeep, +172), 466 (Psycho, +40), 467 (DeadlySphere, +36),
468 (DrManFly, +157), 473-476 (Big Mimics ×4, +40), 477 (Mothron, +103),
479 (MothronSpawn, +2), 480 (Medusa, +107), 481 (GreekSkeleton, +60),
483 (GraniteFlyer, +166). They stay sheet-deferred (the wyvern precedent):
\`npc_sheet\` returns None and the draw skips; their portraits rest the
SetDefaults box. The atlas-repack wave admits them once page 1 exists.

### The wiring (the d35 / small-kinds precedent)

- **\`D40_SHEETS\`** (36 ids, \`render/entities.rs\`) chained into
  \`NpcAssets::npc_sheets\`; the 36 names appended to \`STATIC_SHEET_NAMES\`
  (784 → 820). The deferred 20 are NOT in the chain, so \`npc_tex\`'s
  \`debug_assert\` never fires for them (the draw reads the HashMap miss).
- **The draw arm**: ONE grouped arm for all 56 kinds (the blood-moon /
  small-kinds shape), \`let ty = texture_net_id(n.kind);
  pumpkin_moon_draw(assets.npc_sheet(ty), npc_rows(ty), n, w, h)\`.
  \`pumpkin_moon_draw\` uses \`vanilla_row_h\` (INTEGER division, the
  \`NPC.SetFrameSize\` form \`NPC.cs:66666\`), so the non-dividing NPC_444
  (24×74, 3 rows) loses the leftover 2 px at the bottom instead of bleeding
  them into the last frame. The generic centred anchor
  (\`Main.cs:25511-25516\`) + \`generic_anchor_shift_y\`. The draw match is
  now exhaustive (326 kinds, was 270 + the 56 missing).
- **\`texture_net_id\`**: 18 explicit vanilla rows for the Hide-fold /
  no-entry kinds (ArmedZombie ×7 → 430-436, BoneThrowingSkeleton ×4 →
  449-452, CultistDragonBody ×5 → 455-459, MothronEgg → 478, TargetDummy →
  488). The fold kinds' \`net_id()\` returns the HEAD id (the bestiary
  credit fold), so the wildcard would mis-route the sheet — the d35
  DiggerBody/SeekerBody/VileSpit fishron-hazard precedent. The four
  no-entry kinds (BoneThrowingSkeleton2/3/4, TargetDummy) have cs_id hand
  rows but NO bestiary entry, so \`net_id()\` would panic — the explicit row
  bypasses it (the DetonatingBubble/Sharkron2 precedent). The 38 own-entry
  kinds ride the \`_ => kind.net_id()\` wildcard (= their vanilla id).
- **\`npc_anchor_nudge\`**: 6 rows for the \`Main.cs:23003-23046\`
  num46/num47 else-if overrides — DeadlySphere 467 num46=7 (\`:23003\`),
  EnchantedNightcrawler 484 num46=2 (\`:23019\`), GraniteFlyer 483 num46=14
  (\`:23023\`), Mothron 477 num47=22 (\`:23027\`), MothronEgg 478 num46-=2
  (\`:23031\`), ThePossessed 469 ai[2]==1 num46=14 (\`:23043\`, a match
  guard). None of the 56 appear in the \`:23048\` halfSize switch or
  NPCAddHeight, so the rows stand alone. (The pre-existing DD2Ogre
  num47=22 / DD2KoboldWalker num46=14 / DD2WitherBeastT3 num46=2 rows are a
  misattribution of the type==477/483/484 overrides — the DD2 kinds have
  dedicated draw arms, \`Main.cs:23788\`; left untouched, out of scope.)
- **The rotation whitelist**: 22 kinds whose C# AI / FindFrame writes
  \`npc.rotation\` (the census): GoldBird 442 (\`NPC.cs:73634\` bird group),
  GoldButterfly 444 (\`:72745\`), GoldWorm 448 + EnchantedNightcrawler 484
  (\`:72804\`/\`:72818\`/\`:72823\` crawler tilt + slope ±0.785), Grubby /
  Sluggy / Buggy 485-487 (\`:72859\`/\`:72873\`/\`:72878\`), the Cultist
  Dragon chain 454-459 (\`AI_006\` \`:51878\` follower atan2+1.57 /
  \`:52201\` head atan2+PI/2), CreatureFromTheDeep 461 (\`:75328\`/
  \`:75343\` wet atan2 blend), CrimsonGoldfish 465 (\`AI_016\` \`:24010\`
  ±0.2-clamped), DeadlySphere 467 (\`AI_085\` \`:36991\` spin / \`:37037\`),
  ThePossessed 469 (\`:75668\` atan2+1.57 lunge), ShadowFlameApparition 472
  (\`:71386\` atan2), Mothron 477 (\`AI_088\` \`:37886\` blend), MothronEgg
  478 (\`AI_089\` \`:38357\`/\`:38362\`), MothronSpawn 479 (\`AI_090\`
  \`:38399\` blend), GraniteFlyer 483 (\`:75185\`/\`:75199\`). The other 34
  kinds' AI (the aiStyle-3 fighter, aiStyle-7 town, aiStyle-1 hop,
  aiStyle-83/84 cultist, aiStyle-87 mimic, aiStyle-92 dummy) writes NO
  rotation — they stay out of the whitelist (the C# generic draw rotates
  every NPC by \`npc.rotation\`, \`Main.cs:26632\`, but a zero rotation is a
  no-op).
- **The bestiary portraits**: \`BESTIARY_NPC_SHEET_IDS\` 181 → 203 (+22
  packed own-entry ids), the parallel \`bestiary_npc_sheets\` vec +22
  \`atlas.sheet\` rows (order witnessed). The 22 packed own-entry kinds'
  \`portrait_data\` arms upgraded to the sheet-row shape
  (\`entry_sheet.height / npcFrameCount\`, fallback SetDefaults box — the
  d35/c4 swimmers' shape); the 4 mixed data-wave arms split
  (CultistDragonHead from Body1/2, Nailhead from Psycho, CrimsonBunny from
  CrimsonGoldfish, EnchantedNightcrawler from Grubby/Sluggy/Buggy; the
  latter three share dims and stay grouped). The 20 deferred + 14
  Hide-fold kinds keep the SetDefaults box rest. The CultistDragonHead 454
  custom portrait texture (\`UI/Bestiary/NPCs/NPC_454\`) was already packed;
  the sheet-row upgrade sizes the layout frame (the d35 DiggerHead
  precedent).
- **The frame counts**: all 697 \`NPC_FRAME_COUNT\` slots verified ==
  \`Main.cs:67082\` verbatim (0 mismatches); the 56 kinds' counts ride the
  table unchanged. The \`the_d40_rows_match\` witness pins
  \`texture_net_id(kind) == vanilla id\` + \`npc_rows == Main.npcFrameCount\`
  for all 56; the \`the_d40_strips_divide\` witness pins the 36 packed
  strips' integer-division row heights over the real \`.xnb\` dims + the 20
  deferred staying OUT of \`D40_SHEETS\`; the packing audit
  (\`every_sheet_a_draw_arm_asks_for_is_packed\`) extended for
  \`D40_SHEETS\`, and its vestigial lunar loop bound fixed 430 → 429 (the
  data wave's \`from_u16(430) = ArmedZombie\` broke the old \`..=430\` bound —
  430 is the 421-490 band's first kind, NOT a lunar minion; VortexSoldier
  429 is the lunar band's last).

### Documented deferrals / latent issues (out of scope)

1. **The 20 deferred sheets** + the cumulative pushed-union overflow: the
   atlas-repack merger wave (d35 research/113) owns the second-page
   reconciliation. This phase packs the solo-fitting 36 and documents the
   rest; it does NOT force-pack or swap the landed skyline packer.
2. **The dedicated draw overlays** (CultistTablet 437 GlowMask 108/109
   \`Main.cs:25994\`, Mothron 477 velocity>9 trail \`:26014\`, MothronSpawn
   479 velocity>6.5 trail \`:26034\`, ShadowFlameApparition 472 GlowMask 110
   \`:26054\`, CultistBossClone 440 Extra[30] shadow \`:24520\`): the generic
   sheet draw lands here; the glow/trail/shadow overlays ride their own
   wiring waves (the glow-mask / extra-sheet census).
3. **The \`net_id()\` latent panic** for BoneThrowingSkeleton2/3/4 +
   TargetDummy (no bestiary entry, no fold → \`entry_for_npc\` \`.expect()\`):
   the data wave gave them cs_id hand rows but no entry. The draw bypasses
   \`net_id()\` via the explicit \`texture_net_id\` rows; any OTHER system
   calling \`net_id()\` on them would panic. A bestiary/data follow-up (a
   fold or a net_id hand row) owns that — out of this sheets phase's scope.
4. **The pre-existing DD2 anchor-nudge misattribution** (DD2Ogre num47=22 /
   DD2KoboldWalker num46=14 / DD2WitherBeastT3 num46=2 are the type==477 /
   483 / 484 overrides, but the DD2 kinds have dedicated draw arms): left
   untouched (out of scope); the correct d40 rows are added alongside.
`;
fs.writeFileSync(path.join(__dirname, '_d40_analysis_section.md'), sec);
console.log('section lines:', sec.split('\n').length);
