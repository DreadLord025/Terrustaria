# 360 — W4-g10 lane 3 pets FINAL (SEAL-VERIFY receipt + the pets-family bible)

Branch `task/W4-g10-npc-kind-wire-3`. Number **360** taken per registry-150
§8.7's new-doc rule ("New docs take ≥360 after a registry check"): a fresh
all-tip `git ls-tree docs/research/` sweep over every `origin/task/W4-*`
branch at seal time finds the highest taken number **349** (the §8/§8.7
frozen live set ends at 351; 352..359 unclaimed, 360 the rule's floor).
This doc is the numbered FINAL the g10b seal brief owed; the claims-table
stays a `tools/` claim doc (`tools/_g10_pet_claims.md`, phase 4 — the
`_d120_claim.md` convention) and the family bible's canonical copy is the
ANALYSIS "The pets-family declaration" paragraph (phase 5). No frozen-table
assignment is touched; no other lane's number is contested.

## 1. The commit ledger (SEAL-VERIFY item 1 — g10b landed)

The g10b-pets-final work was dispatched earlier and **LANDED on this
branch** (verified at seal time; the untracked-file state the brief
described was consumed by the phase commits):

| Phase | Commit | Scope |
| --- | --- | --- |
| batch 1 | `2d0403324` | the ten town pets 637/638/656/670/678-683 landed + wired |
| batch 2 | `5dee28650` | TownSlimeCopper 684 (the last `IsTownPet` member), ALL_KINDS 281 |
| FINAL 1 | `7bc156c40` | `npc_pet_exchange.rs` — the pet-license EXCHANGE NPC side (503 ln incl. 7 witnesses) |
| FINAL 2 | `91c8ed1eb` | `player_petting.rs` — the petting interaction (the e10 deferred host; 664 ln incl. 10 witnesses; the five folds go live) |
| FINAL 3 | `77ec42303` | `npc_chat.rs::slime_chat_colour` — the town-slime colour-per-kind mapping (Lang.GetSlimeType, 1 witness) |
| pin-fix | `4c4e56752` | the s1_f_3 coverage pin 270→281 (the d37 research/250 owed fix) |
| FINAL 4 | `f34bd2715` | `tools/_g10_pet_claims.md` — the d43r keep-one claims-table + notify-note |
| FINAL 5 | `197c777eb` | ANALYSIS "The FINAL phase" + the pets-family declaration + 3 PORT_MAP rows + the ROADMAP row + 3 anchors rows |
| lint-fix | `aebb1d824` | the `mut gore` drop (the Option is consumed by-value in the `if let`, no `mut` binding needed) |
| seal | (this commit) | this numbered FINAL + the INDEX row |

## 2. ALL-11 wired census (SEAL-VERIFY item 2 — VERIFIED)

The eleven `NPCID.Sets.IsTownPet` members (`NPCID.cs:10118`) —
637 TownCat / 638 TownDog / 656 TownBunny / 670 TownSlimeBlue /
678 Green / 679 Old / 680 Purple / 681 Rainbow / 682 Red / 683 Yellow /
684 Copper — are LANDED and WIRED at this tip:

- **Kind rows**: `npc/data.rs:594-604` cs_id + `:938-948` from_u16 at the
  VANILLA ids (cs==proto, the port-local bands stop at 353); the
  exhaustive-match sites (ai_style 7 / size 18x20, dog 18x28 / max_hp 250
  / contact_damage 10 / kb 0.5 / defense 15 / value 0 / hit NPC_Hit_1 /
  death NPC_Killed_6 / frame counts 28/28/27/14 / ExtraFramesCount
  18/11/20/6) all carry the eleven arms.
- **The 7-axis claim**: `npc.rs:5054` — `7 if self.kind.is_town_pet() =>
  self.ai_007_town_pet_family(...)` routes into the landed pet-aware
  `npc_ai_town::town_npc_ai` (`AI_007_TownEntities`,
  `NPC.cs:52965-55561`), kind-guarded so the 21 housed residents keep
  `update_guide` until W4-b3's unguarded `7 =>` arm lands (disjoint
  claims; the frost-mob `62 if ElfCopter` precedent).
- **npc_findframe_pet**: `npc_findframe_pet.rs::town_pet_find_frame`
  called at `npc.rs:7014` for all eleven kinds — the slime
  spriteDirection flip (`NPC.cs:68657-68660`), the state-20/21/22 idle
  poses, the cat sit row 19, the dog idle 0..7 cycle, the walk stride 12
  + start-row overrides, the airborne hop rows / dog air row 8.
- **Predicates + joins**: `is_town_pet`/`is_town_slime`
  (`data.rs:1213`+), `is_town_npc`/`is_town_npc_like`/`is_friendly`
  joins, the `is_town_resident` EXCLUSION (tax/arrival/party), the
  bestiary entries + Friends.* name-plate override (cat/dog/bunny),
  TOWN_PET_SHEETS + BESTIARY_NPC_SHEET_IDS 192, the sitting draw shift
  (`render/entities.rs::npc_add_height`, `Main.cs:22178-22181`), the
  saveglue town-block rows, the clock partyable filter.
- **Witnesses**: `npc.rs::w4_g10_town_pet_wiring` (8, the PETS table
  carries all eleven) + the s1_f_3 coverage pin now 281.

## 3. The f43b splice contract (SEAL-VERIFY item 3 — g10's leg DONE, receipt)

The a6 Success-leg activation contract (research/138 §12.2 at the a6 tip
`0766c8df9`, re-read at seal time) is:

**g10-kinds-DONE → f43-splice (the f43b lane) → a6 else-arm one-line
dispatch.**

- **g10's leg is DONE**: the kinds exist (item 2 above) AND the exchange
  chain's live-Npc mapping landed (`npc_pet_exchange.rs`,
  `reroll_variation_for_kind` — the first-active-resident scan, the
  ≤100-draw `Main.rand.Next(6)` loop / the legacy constant-0 no-draw
  roll, the genRand name draw, the heal, the 13-gore/104-draw
  PetExchange burst). The module IS the contract's step-1 consumer; the
  POSITIVE Success witnesses the a6/g87 boundary said could not exist
  pre-kinds are written (7, incl. the two-stream cat roll/heal/name and
  the burst stream census).
- **The f43b splice is NOT g10's to execute**: f43
  (`79a037f5e`, `town_arrival_pets.rs` 819 ln UNWIRED) owns the pure-fn
  shape (`reroll_variation`/`reroll_variation_for_npc_type` over
  `PetVariationSlot`) + the arrival gates (`Main.cs:65900-65943`); the
  f43b lane splices them into `town_arrival.rs` and lands the live-Npc
  adapter. Keep-one recorded in the module docs + the claims-table: at
  the merge the shared loop + rosters dedup onto f43's shape; g10 keeps
  the live-`Npc` scan, the heal/name writes and the gore burst f43
  leaves to the caller. The variant rosters are byte-identical on both
  lanes (`TownNPCProfiles.cs:139-147` re-read: Cat
  Siamese/Black/OrangeTabby/RussianBlue/Silver/White, Dog
  Labrador/PitBull/Beagle/Corgi/Dalmation/Husky, Bunny
  White/Angora/Dutch/Flemish/Lop/Silver; the eight slimes are
  LegacyNPCProfile head ids 46-53, `:150-180`).
- **The exact hunk's anchor VERIFIED unchanged** at the a6 tip:
  `powerups.rs:441-450` is still the hardcoded-Fail else-arm
  (`use_pet_license`, the `npc_type` local at `:412-417`); the published
  one-line dispatch (`roll_rng` on `RngKey::UpdatePlayers` — the §10.2
  precedent — + the `if success` PetExchangeSuccess/Fail key select,
  World colour, consume either way `Player.cs:45442`) applies as
  published. g10's `reroll_variation_for_kind` is the drop-in the
  merger may call directly instead of f43's adapter (same semantics,
  the Success/Fail/NoResident → chat-key mapping already carries
  `chat_key()`); either shape satisfies the contract's CHAT CONTRACT.
- **RNG census stands**: the variation roll is the ONLY `Main.rand`
  slot before the burst (`Profiles.cs:216`); the name draw rides
  `WorldGen.genRand` (`LanguageManager.cs:419`) — two streams, injected
  separately; the burst draws 8/gore × 13 = 104
  (`ParticleOrchestrator.cs:2691-2700`, the raw-`Next()` rotation
  outside the sentinel guard — the C# draws even on a refused spawn).

## 4. The d43 dup keep-one (SEAL-VERIFY item 4 — VERIFIED, g10 canonical)

d43's 631-697 band rowed the SAME eleven pets (batches 2/5/9/10/11).
The keep-one is adjudicated on BOTH sides and re-verified at the live
tips at seal time:

- **d43 research/343** (renumbered 304→343 per registry-150 §8.7, tip
  `715cf5f91`; the retire `6fbe06250` published the drop-hunks):
  "11 town pets → g10" among its 18→20 drops. d43's pets ride
  `update_guide` + `{}` FindFrame — bare stand-ins.
- **d37 research/250** (tip `7bea5c835`): "g10 | 11 pets PUSHED |
  `5dee28650`" in the FINAL v2 389-kind union; the g10 owed s1_f_3 pin
  270→281 is EXECUTED (`4c4e56752`).
- **KEEP g10's wired superset; DROP d43's eleven** (the
  keep-wired-superset rule, the d37d-E8 adjudication). The per-kind
  keeper evidence + the d43 drop-hunk list + the notify-note are in
  `tools/_g10_pet_claims.md` (phase 4). The g10 bestiary Friends.*
  override ("Town Cat"/"Town Dog"/"Town Bunny",
  `BestiaryDatabaseNPCsPopulator.cs:646-648`) is the C#-faithful plate
  d43's plain names lose.

## 5. The pets-family bible (one paragraph, canonical in ANALYSIS)

The town-pet family seals as a multi-lane machine with one owner per
seam: **g10** owns the eleven kinds (637/638/656/670/678-684, cs==proto)
+ the kind-guarded `7 if is_town_pet` axis arm + `npc_findframe_pet.rs`
+ the `is_town_pet`/`is_town_slime` gates + the bestiary Friends.*
override + the FINAL activation consumers (the exchange NPC side
`npc_pet_exchange.rs`, the petting machine `player_petting.rs`, the
colour-per-kind `slime_chat_colour`); **d43** drops its eleven bare
`update_guide` pet stand-ins onto g10 (research/343, the
keep-wired-superset rule); **f43** owns the pure-fn exchange chain shape
(`town_arrival_pets.rs`) + the arrival gates (`Main.cs:65900-65943`) —
g10's exchange module is its live-Npc mapping, the loop/rosters dedup at
the merge; **a6/g87** own the use-arm cluster
(`powerups.rs::use_pet_license`, the bought flags + gate + chat) — the
Success dispatch is the one-line conditional onto the reroll result
(research/138 §12.2 step 4); **d120** owns the GetChat dispatch + the
dialog-filter machinery — its type-keyed pet/slime chat arms activate
when g10's kinds merge (zero d120 edits, its research/143 retire item
2); **e10**'s deferred petting folds went live on g10's `Player.petting`
field (phase 2). Activation order: kinds (g10, DONE) → arrival (f43b) →
the use-arm Success swap (a6/g87). The petting + colour consumers are
g10-owned with no sibling kind claim. The petting census stands: the
town-pet path draws NO particle (the heart emote is the Pal-projectile
path's, `Main.cs:34270-34275`) and NO sound (the PalChilletJoy cue is
PetMount's, types 62-65); the only petting pose is the composite back
arm's 14-tick pat cycle; the whole petting chain draws ZERO sim RNG.

## 6. Verification record (code-only)

- Every FINAL row re-read against the cited C# windows at seal time:
  `NPC.cs:80858-80961` (GetPettingInfo / UnlockOrExchangePet /
  RerollVariationForNPCType / RerollVariation),
  `Player.cs:33339-33490` (PetMount/PetAnimal/StopVanityActions/
  StopPettingAnimal/UpdatePettingAnimal) + `:50174`/`:50204`/
  `:50240-50274` (the light/hold/pat arms),
  `PlayerPettingInfo.cs:5-77`, `Profiles.cs:83-86`/`:214-222`,
  `TownNPCProfiles.cs:136-181`, `ParticleOrchestrator.cs:2691-2700`,
  `Gore.cs:1286-1354` (the burst's allocator census),
  `Utils.cs:1261-1269`/`:1665-1668` (NextVector2Circular /
  ToRotationVector2), `Lang.cs:375-411`,
  `NetMessage.cs:470-479`/`MessageBuffer.cs:1039-1045` (the
  petting/petSmall bits), `NPCID.cs:10118-10119`,
  `NPC.cs:17207-17225`/`:17295-17309`/`:17481-17495`/`:17594-17608`.
- The sibling verdicts re-read at their fetched tips: a6 `0766c8df9`
  (the §12.2 contract + the `powerups.rs:441-450` anchor unchanged),
  f43 `79a037f5e`, d43 `715cf5f91` (research/343), d37 `7bea5c835`
  (research/250), registry-150 at the d54 tip (§8.7 FROZEN + the ≥360
  new-doc rule).
- The registry check for this doc's number: an all-tip
  `git ls-tree docs/research/` sweep — taken numbers ≥330 are
  {338, 341-346, 348, 349}; nothing ≥350 exists on any tip; 360 is the
  §8.7 floor and FREE.
- Witness ledger: 8 batch wiring + 7 exchange + 10 petting + 1 colour
  = **26 witnesses WRITTEN NOT RUN**. The seal's compile-fix (`mut
  gore` → `gore`) is behaviour-neutral (the binding was never
  re-assigned; the Option moves into the `if let`).
- **No cargo/rustc/tests/rustfmt/graphify invoked** (the lane
  constraint). Commit + push only; never merge/primary.
