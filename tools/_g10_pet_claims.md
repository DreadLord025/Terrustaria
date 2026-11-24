# W4-g10 town-pet claims-table + d43r keep-one receipt (FINAL phase 4)

Branch `task/W4-g10-npc-kind-wire-3`, tip after the FINAL phases
(`5dee28650` batches 1/2 + the FINAL exchange/petting/colour/pin commits).
Code-only wave: no cargo/rustc/tests/rustfmt/graphify invoked; witnesses
WRITTEN NOT RUN. This doc is the g10-side claims-table for the d37d-E3
merge adjudication + the notify-note to d43r / d37d. It is a `tools/`
claim doc (the `tools/_d120_claim.md` convention), NOT a numbered
research doc — the research-number registry (research/150 §8.7, FROZEN,
NO-SELF-PICKS, new docs ≥360 after a round-5 re-scan) is respected by
not taking a number.

## 1. The conflict (the Lead's item 4)

d43r (`task/W4-d43-npc-kinds-631-697`, the R3-dispatched 631-697 band)
ported the town-pet kinds TOO — its batches 2/5/9/10/11 rowed
637/638 (TownCat/TownDog), 656 (TownBunny), 670 (TownSlimeBlue),
678-683 (Green/Old/Purple/Rainbow/Red/Yellow), 684 (TownSlimeCopper).
g10's eleven kinds INCLUDE all of these. **The overlap is ELEVEN, not
the brief's two (637/638)** — g10 grew to TownSlimeCopper 684 in batch 2.

## 2. The verdict — KEEP g10 (already adjudicated, this is the receipt)

The keep-one is ALREADY decided by BOTH sibling lanes; this table is the
g10-side confirmation + the exact keeper evidence, not a new adjudication:

- **d43 research/304 §2c** (renumbered **343** per registry-150 §8.7;
  tip `31310b652`): "The 11 g10 town pets → keep g10". d43 self-adjudicated
  18 drops, the 11 pets among them. d43's pets ride `update_guide` (the
  town-NPC stand-in) + `{}` FindFrame; g10 wires the pet-aware machine.
- **d37 research/250** (tip `7bea5c835`): "g10 | 11 pets PUSHED |
  `5dee28650` | 637/638/656/670/678-684 as the SEAL accepted". The FINAL
  v2 plan counts g10's 11 in the 389-kind union; d43's drop count is 20
  (its 18 + the SporeBat/QueenSlimeMinionPurple stand-ins).

**KEEP g10's wired rows; DROP d43's eleven pet kinds.** The deciding
factor (the keep-wired-superset rule, applied symmetrically across the
roster reconcile): g10's pets are WIRED to the landed pet-aware town
machine; d43's are bare `update_guide` stand-ins.

## 3. The claims-table (per-kind, g10 keeper evidence)

All eleven ride their VANILLA ids in the protocol space (cs==proto, the
pumpkin/frost precedent — 637/638/656/670/678-684 are free in
to_u16/from_u16; the port-local bands stop at 353). `data.rs:594-604`
carries all eleven cs_id rows; `to_u16`/`from_u16` at the same ids.

| vanilla | kind | g10 keeper wiring | d43 row (DROP) |
| --- | --- | --- | --- |
| 637 | TownCat | axis `7 if is_town_pet` → `ai_007_town_pet_family`; `npc_findframe_pet.rs`; bestiary "Town Cat" (Friends.TownCat) | `update_guide` + `{}` FindFrame; bestiary "Cat" (NO Friends override) |
| 638 | TownDog | same axis; FindFrame dog idle 0..7 + air row 8; bestiary "Town Dog" | `update_guide` + `{}`; bestiary "Dog" |
| 656 | TownBunny | same axis; FindFrame bunny walk stride 12; bestiary "Town Bunny" | `update_guide` + `{}`; bestiary "Bunny" |
| 670 | TownSlimeBlue | same axis; FindFrame slime flip + hop rows; bestiary "Nerdy Slime" | `update_guide` + `{}`; "Nerdy Slime" |
| 678 | TownSlimeGreen | same; "Cool Slime" | `update_guide` + `{}`; "Cool Slime" |
| 679 | TownSlimeOld | same; "Elder Slime" | `update_guide` + `{}`; "Elder Slime" |
| 680 | TownSlimePurple | same; "Clumsy Slime" | `update_guide` + `{}`; "Clumsy Slime" |
| 681 | TownSlimeRainbow | same; "Diva Slime" | `update_guide` + `{}`; "Diva Slime" |
| 682 | TownSlimeRed | same; "Surly Slime" | `update_guide` + `{}`; "Surly Slime" |
| 683 | TownSlimeYellow | same; "Mystic Slime" | `update_guide` + `{}`; "Mystic Slime" |
| 684 | TownSlimeCopper | same; "Squire Slime" (batch 2) | `update_guide` + `{}`; "Squire Slime" |

### The keeper wiring d43 does NOT have (the superset)

- The kind-guarded axis arm `7 if self.kind.is_town_pet()` (`npc.rs:5054`)
  routing into `ai_007_town_pet_family` → the landed pet-aware
  `npc_ai_town::town_npc_ai` (the solo TownEnv + NoOpTownPetDoors + the
  generic UpdateNPC collision tail). d43's pets ride `update_guide`.
- `npc_findframe_pet.rs` (489 ln, `town_pet_find_frame` called at
  `npc.rs:7014`): the slime spriteDirection flip, the state-20/21/22 idle
  poses, the cat sit row 19, the dog idle 0..7 cycle, the dog/bunny/slime
  walk stride 12 + start-row overrides, the airborne slime hop / dog air
  row 8. d43's FindFrame arms are `{}`.
- The `is_town_pet`/`is_town_slime` kind predicates (`data.rs:1213`), the
  `is_town_npc`/`is_town_npc_like`/`is_friendly` joins + the
  `is_town_resident` EXCLUSION (tax/arrival/party), the ExtraFramesCount /
  AttackFrameCount / humanoid_frame_count rows, the sitting draw shift
  (`render/entities.rs::npc_add_height`), the `world_sim/clock.rs`
  pet-spawn partyable filter, the `app/saveglue.rs` pet town-block rows.
- The bestiary Friends.* name-plate override for cat/dog/bunny
  (`bestiary.rs:2178-2205`, "Town Cat"/"Town Dog"/"Town Bunny" — the C#
  `ModifyEntriesThatNeedIt_NameOverride(637/638/656, "Friends.Town*")`,
  `BestiaryDatabaseNPCsPopulator.cs:646-648`). d43's "Cat"/"Dog"/"Bunny"
  miss the override. The slime names agree on both lanes.

### The d43 drop-hunks (d43 research/304 §2c, the merger executes)

d43 drops, per its own published hunks: the 11 enum variants; the
kb/defense/value rows; the `update_guide` kind-AI arms; the `{}` FindFrame
arms; render_key rows; ALL_KINDS entries; the batch stat pins;
`data.rs` to_u16/from_u16/cs_id/ai_style(7)/size/max_hp/damage/sound rows;
`npc_buffs.rs` `&[31]` rows; `app/netglue.rs` spawn_cave_walker arms (g10
uses spawn_guide — keep g10's); `bestiary.rs` ENTRIES; `ui/bestiary.rs`
portraits; `render/entities.rs` draws. **g10's gates + findframe + clock +
saveglue are the keeper wiring — d43 has none of them.**

## 4. The FINAL-phase additions (this lane, post-batch-2)

These ride ON TOP of the eleven keeper kinds and are g10-owned (no d43
counterpart — they are the activation consumers, not kind rows):

- **Pet-license exchange NPC side** (`npc_pet_exchange.rs`): the
  `RerollVariationForNPCType`/`RerollVariation` chain over the live kinds
  (the a6/g87 use-arm's Success dispatch, research/138 §12.2 activation
  contract step 1 consumed). Keep-one vs f43's `town_arrival_pets.rs`
  pure fns recorded in the module docs (f43 owns the loop/roster shape;
  g10 is its live-Npc mapping).
- **Petting** (`player_petting.rs`): the PlayerPettingInfo +
  GetPettingInfo + PetAnimal/PetMount/UpdatePettingAnimal + the hold-arm
  pat cycle (the e10 deferred host). g10-owned; no sibling claims it.
- **Slime colour-per-kind** (`npc_chat.rs::slime_chat_colour`): the
  Lang.GetSlimeType 8-word switch over the kinds. Keep-one vs d120's
  type-keyed `slime_chat_colour(ty)` + its GetChat arms (d120 owns the
  dispatch; its arms key on `cs_id` and go LIVE when these kinds merge —
  zero d120 edits, its research/143 retire item 2). At the merge the
  kind-keyed and type-keyed fns dedup (`kind.cs_id()` bridges them).
- **The s1_f_3 coverage pin 270→281** (the d37 research/250 owed pin-fix,
  executed this phase).

## 5. Notify-note (to d43r + d37d-E3)

- **To d43r**: your eleven pet kinds (637/638/656/670/678-684) are
  CONFIRMED dropped onto g10 per your own research/304 §2c (→343). g10's
  wired superset is the keeper. No g10 action needed on your side beyond
  the drop-hunks you already published. The g10 FINAL additions
  (exchange/petting/colour) are activation consumers, not kind rows —
  they do not change the drop set.
- **To d37d-E3**: the g10 column of the master reconcile is ELEVEN pets
  cs==proto at `5dee28650` + the FINAL commits, the s1_f_3 pin now 281
  (the owed fix executed). The keep-one verdict (g10 over d43) stands as
  research/250 recorded. The g10 bestiary Friends.* override is the
  C#-faithful name plate (d43's plain names lose). The merger splices g10
  as-pushed (the research/250 merger-bible order: …d43/g10… splice
  as-pushed); the bijection gate stays 281 for g10's standalone roster.

## 6. Verification

Code-only: every keeper row re-read against the cited C# windows
(NPC.cs:17207-17225/:17295-17309/:17481-17495/:17594-17608 SetDefaults,
:55750-55785 pet idle, :68622-69655 FindFrame, NPCID.cs:10118-10119 the
sets, BestiaryDatabaseNPCsPopulator.cs:646-648/:833-843). The d43/d37
verdicts re-read at their live tips (d43 `31310b652`, d37 `7bea5c835`).
The witnesses (8 batch wiring + 7 exchange + 10 petting + 1 colour) are
WRITTEN NOT RUN. No cargo/rustc/tests/rustfmt/graphify invoked.