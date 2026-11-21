# 239 — W4-d42 FINAL: the 11-kind keep-one drop-hunks, the 593-fold re-point, the 592/607/615 re-adjudication + the band declaration

Status: MERGE CONTRACT + LANE-CLOSE DECLARATION (doc-only; the drop-hunks
are EXECUTE-prep for the merger — d42's own branch keeps all its rows so
the exhaustive matches compile, the E8 dropped-lane rule). Author lane:
W4-d42 (`task/W4-d42-npc-kinds-561-630`), parent `33b3cfe07` (the
BloodSquid hover gap-closure; the drop-hunk line ranges below are pinned
to the FINAL doc commit's tree — re-verify with git grep if the tip
advances). Number 239: free per registry-150's 237+ block
(the 355-tip 150-299 sweep finds only e23's local 237 taken; 238 is this
lane's renamed sheets doc).

Companion inputs read at their named tips (git-show, read-only):
- research/108 (d37 `a341daa12`, the RECEIPTS round) — E3/E8/R3/R9.
- research/145 (c4 local `ff8026327`, UNPUSHED at read time) — the
  keep-d42-verdicts withdrawal + the CorruptGoldfish/CrimsonGoldfish flags.
- research/148 (c4 local `0df6964be`, UNPUSHED) — the FISH-FAMILY
  declaration, 592/607/615 STAND + the 593-fold re-point request.
- d33 `b08acf463` (pushed, clean) — the canonical dragonfly family.
- c10 `5d8387955` (pushed; dirty ROADMAP row only) — the canonical
  waterfowl family + research/145-waterfowl-band-final.
- d54 `38bfd1d22` (pushed) — research/150, the number registry.

## 1. The 11-kind keep-one verdict (EXECUTE-prep)

Canonical owners (108-E3 + R3, re-verified at the pushed tips):

| Kind | cs/proto | Canonical owner | d42's duplicate |
|---|---|---|---|
| BlackDragonfly | 595 | **d33** `b08acf463` (aiStyle-114 axis WIRED) | drop at merge |
| BlueDragonfly | 596 | d33 | drop |
| GreenDragonfly | 597 | d33 | drop |
| OrangeDragonfly | 598 | d33 | drop |
| RedDragonfly | 599 | d33 | drop |
| YellowDragonfly | 600 | d33 | drop |
| GoldDragonfly | 601 | d33 | drop |
| Seagull | 602 | **c10** `5d8387955` (style-7/68 axes WIRED) | drop |
| Seagull2 | 603 | c10 | drop |
| Grebe | 608 | c10 | drop |
| Grebe2 | 609 | c10 | drop |

Why the owners win (not d42's rows): both owners are the **fully-wired
supersets** — d33 carries the aiStyle-114 machine + the dragonfly_neighbors
sim pre-pass + the catch/release chain 4334-4340 + the FindFrame cases
595-601 + DRAGONFLY_SHEETS + the netglue hover seed; c10 carries the
style-7 town walk + the style-68 fly machine + the takeoff gate + the
Transform drain + TransformVisuals + the sound census. d42's rows are the
data-only join (idle `{}` stand-ins, sheets deferred). The stats are
byte-equal (both port the same `NPC.cs:16720-16738` / `:16739-16762` /
`:16809-16832` rows — verified arm-by-arm at the tips), so the drop loses
no behaviour; the owners' wiring is the superset.

**d42 keeps its rows for the OTHER 32 kinds** (578/579/580/582/583-586/
590/591/593/594/610-614/616-630) — they are d42-owned, no rival port.

### 1a. The exact d42 drop-hunks (the FINAL doc commit's tree, 1-indexed inclusive)

Every line below was read from the working tree at `33b3cfe07` (clean;
the doc commit touches only docs/, so the code line numbers are
identical). Re-verify with `git grep -n <Kind> <tip> -- crates` if the
tip advances (the peer-session rotation commit `2f6b99947` already
shifted entities.rs — the ranges below are POST-that-commit). DROP =
delete the lines; EDIT = replace as stated. The canonical owner's
row+wiring survives; only d42's duplicate arms die (E3's rule).
`cs_id`/`net_id`/sheets/render-keys are cs/net-keyed and DO NOT move
(R1's seam evidence).

#### `crates/tr-app/src/npc.rs`

| Lines | Action | Content |
|---|---|---|
| 1151-1194 | DROP | the 11 `/// NPCID.<Kind>…` doc-triples + variants (BlackDragonfly…Grebe2); the WindyBalloon 1147-1150 and Rat 1195+ docs stay |
| 3746-3759 | DROP | the kb block: 3 batch comments + 11 `=> 1.0,` rows (595-599/600-604/605-609 groups) |
| 4180-4193 | DROP | the defense block: 3 comments + 11 `=> 0,` rows |
| 4606-4619 | DROP | the value block: 3 comments + 11 `=> 0.0,` rows |
| 6756-6799 | DROP | the update_kind_ai idle arms: the batch-6 comment 6756-6759 + Grebe/Grebe2 cites+arms 6760-6765, the batch-5 comment 6766-6768 + Yellow/Gold/Seagull/Seagull2 cites+arms 6769-6780, the batch-4 comment 6781-6784 + the five dragonfly cites+arms 6785-6799 (the batch-3 comment 6800+ stays) |
| 9791-9932 | DROP | the six FindFrame arms + their cites: five-colour dragonfly 9791-9801, Yellow/Gold 9802-9812, Seagull 9813-9834, Seagull2 9835-9872, Grebe 9873-9894, Grebe2 9895-9932 (the Rat case 9933+ stays) |
| 19959-19972 | DROP | the render_key block: 3 comments + 11 `=> <cs>,` rows |
| 25066 | EDIT | `pub(super) const ALL_KINDS: [NpcKind; 313]` → **302** |
| 25360-25373 | DROP | the ALL_KINDS entries: 3 batch comments + 11 `NpcKind::<Kind>,` rows |
| 25600-25634 | DROP | the whole `w4_d42_b4_the_defaults_match_the_csharp_rows` test (doc 25600-25602 + fn 25603-25634; all five rows are dragonflies — d33's stat pin covers them) |
| 25636-25669 | DROP | the whole `w4_d42_b5_…` test (all four rows Yellow/Gold/Seagull/Seagull2 — d33/c10 pins cover them) |
| 25671-25701 | DROP | the whole `w4_d42_b6_…` test (both rows Grebe/Grebe2 — c10's pin covers them) |
| 25736 | EDIT | b7's `assert_eq!(ALL_KINDS.len(), 298)` → **287** (the cumulative pin loses the 11) |
| 25771 | EDIT | b8's `302` → **291** |
| 25807 | EDIT | b9's `307` → **296** |
| 25843 | EDIT | b10's `312` → **301** |
| 25871 | EDIT | b11's `313` → **302** |
| 29388-29390 | EDIT | the s1_f_3 gate `covered + sentinel, 313` → **302**; the message's "W4-d42's 43 … kinds" → "32 … kinds (the 11 d33/c10-owned dragonflies+waterfowl drop at merge, research/239)" |

The b1/b2/b3 cumulative pins (274/278/282) are BEFORE the 11-kind batches
— unchanged. The round-trip test (`the_protocol_id_space_round_trips_every_kind`)
reads `ALL_KINDS.len()` dynamically — no edit; the bijection +
deny-unreachable guards force the to/from dedupe.

#### `crates/tr-app/src/npc/data.rs`

| Lines | Action | Content |
|---|---|---|
| 609-619 | DROP | the 11 `to_u16` arms (`NpcKind::<Kind> => <cs>,`) |
| 985-995 | DROP | the 11 `from_u16` arms (`<cs> => NpcKind::<Kind>,`) |
| 1910-1923 | DROP | the HitSound block: 3 comments + 11 `=> "NPC_Hit_1",` rows |
| 2271-2287 | DROP | the DeathSound rows: 11 `=> "NPC_Killed_1",` + the 2 interior batch comments 2278-2279/2284-2285 (KEEP the leading comment 2271-2272 — it also documents 614's null row, which stays) |
| 2427-2437 | DROP | the no_gravity groups: comment 2427 + five-colour group 2428-2433, comment 2434 + Yellow/Gold group 2435-2437 (the BloodNautilus/BloodSquid group 2438+ stays) |
| 2714-2727 | DROP | the ai_style block: 3 comments + 11 rows (114×7, 7/68/7/68) |
| 3315-3328 | DROP | the size block: 3 comments + 11 rows |
| 3477-3496 | DROP | the cs_id block: 3 "TRUE vanilla types" comments + 11 rows |
| 4093-4106 | DROP | the max_hp block: 3 comments + 11 `=> 5,` rows |
| 4511-4524 | DROP | the contact_damage block: 3 comments + 11 `=> 0,` rows |

#### `crates/tr-app/src/bestiary.rs`

| Lines | Action | Content |
|---|---|---|
| 2265-2302 | DROP | the 4 ENTRIES rows + their 3 batch comments: RedDragonfly 2267-2274, GoldDragonfly 2277-2284, Seagull 2285-2292, Grebe 2295-2302 (d33 carries all 7 dragonfly rows incl. the 5 hidden colours' own catch-chain rows; c10 carries all 4 waterfowl rows incl. Seagull2/Grebe2) |
| 2660-2672 | DROP | the entry_for_npc folds: comment 2660-2661 + the 4 colour folds 2662-2665, comment 2666-2667 + Yellow→Red 2668 + Seagull2→Seagull 2669, comment 2670-2671 + Grebe2→Grebe 2672 (d33's own fold group survives; see §1b for the c10 fold gap) |

#### `crates/tr-app/src/npc_buffs.rs`

| Lines | Action | Content |
|---|---|---|
| 511-524 | DROP | the 3 immunity groups + comments: five-colour `&[31]` 511-516, Yellow/Gold/Seagull/Seagull2 517-521, Grebe/Grebe2 522-524 (the owners' `{31}` rows survive — `NPCID.cs:9523-9577`) |

#### `crates/tr-app/src/app/netglue.rs`

| Lines | Action | Content |
|---|---|---|
| 3628-3662 | DROP | the 5 npc_seed spawn arms + comments: Grebe walker 3628-3631, Grebe2 hover 3632-3638, Seagull walker 3639-3642, Yellow/Gold/Seagull2 hover 3643-3651, five-colour hover 3652-3662 (d33/c10's own seeds survive) |

#### `crates/tr-app/src/ui/bestiary.rs`

| Lines | Action | Content |
|---|---|---|
| 1617-1629 | DROP | the 4 portrait arms + 3 comments: RedDragonfly 1620, GoldDragonfly 1624 + Seagull 1625, Grebe 1629 (the owners' portrait rows survive — d33's 10x10 dragonfly strips, c10's 22x26/28x22 waterfowl) |

#### `crates/tr-app/src/render/entities.rs`

| Lines | Action | Content |
|---|---|---|
| 559-560 | DROP | the rotation-gate members `\| NpcKind::Seagull2` + `\| NpcKind::Grebe2` (c10's own gate/wiring carries the fly tilt; d42's gate is the duplicate) |
| 1244-1259 | DROP | the texture_net_id rows: comment 1244-1246 + Grebe2=>609 1247, comment 1248-1250 + YellowDragonfly=>600 1251 + Seagull2=>603 1252, comment 1253-1255 + the four colour rows 1256-1259 (the owners' texture rows survive; the ArmedTorchZombie/GGF-walker comment 1260+ stays) |
| 3408-3452 | DROP | the 11 draw arms + 3 comments (Grebe/Grebe2 3412-3417, Yellow/Gold/Seagull/Seagull2 3422-3433, five colours 3438-3452; the owners' draw arms + sheets survive) |
| 3750-3761 | DROP | the rotation-gate witness's waterfowl block (the `for kind in [Seagull2, Grebe2]` fly-tilt loop — c10's own witnesses cover it) |
| 3792-3800 | DROP | the 9 OUT-list entries (the 7 dragonflies + Seagull + Grebe) in `w4_d42_the_rotation_gate_consumes_the_writers`; the OUT list shrinks 33→24 and the IN gate 10→8 (−Seagull2/Grebe2 at 559-560), total 32; update the doc's "ten IN"→"eight IN" (3707), "33 OUT"→"24 OUT" (3709/3779) |

### 1b. Two owner-side gaps the merger must close (NOT d42 drops)

1. **c10's Seagull2/Grebe2 credit fold is MISSING.** c10's `entry_for_npc`
   has NO `Seagull2 => Seagull` / `Grebe2 => Grebe` arm (read at
   `5d8387955`: the fold match ends at ForceBubble). The vanilla hides
   603/609 (`NPCID.cs:5256-5261`/`:5302-5307` `Hide = true`) and
   `ContentSamples.cs` has NO explicit 603/609 credit row — the automated
   populator folds the hidden draw onto the visible twin. d42's drop
   removes its (correct) folds; **c10 must add the two fold arms** (the
   d33 dragonfly-fold precedent) or the Seagull2/Grebe2 kills credit a
   hidden card. Flagged for c10/the merger.
2. **d33's hidden-colour ENTRIES rows are the superset** — d42 only ever
   carried the 2 visible dragonfly rows (Red/Gold); d33 carries all 7
   (the 5 hidden colours keep own rows for the catch/release chain, the
   c8 TruffleWormDigger precedent). The drop loses nothing; the union
   ENTRIES count uses d33's 7, not d42's 2.

### 1c. Count-pin deltas (d42 branch, post-drop)

| Pin | Before | After | Note |
|---|---|---|---|
| `ALL_KINDS` / to_u16 / from_u16 / s1_f_3 | 313 | **302** | −11 |
| bestiary `ENTRIES` (d42's own rows) | 281 | **277** | −4 visible rows (the 11 kinds' credits ride the owners' rows) |
| rotation-gate IN members | 10 | **8** | −Seagull2/Grebe2 (c10 owns the tilt) |
| rotation-gate OUT witness list | 33 | **24** | −9 (7 dragonflies + Seagull + Grebe) |

The 108-R9 UNION pins (ALL_KINDS 550, ENTRIES 486) already counted these
11 once (under d33/c10) — the drop is the dedupe that makes the union
match; no union re-derive is owed by d42.

## 2. The GoldGoldfishWalker-593 fold re-point (c4's 592 IS ported)

c4's research/145 + 148 confirm **GoldGoldfish 592 IS ported on c4**
(`bestiary.rs` `credit_id: "GoldGoldfish", net_id: 592`, pushed
`479d70ad2`). d42's interim OWN walker entry was the d40
BoneThrowingSkeleton2/3 unported-fold-target precedent — its condition
("592 is c4b's UNPORTED kind") is now FALSE. **Merge action (the c4
request, executed as a contract):**

- `bestiary.rs:2244-2256` — DROP the interim `GoldGoldfishWalker` ENTRIES
  row + its comment (net_id 593). The walker's credit folds onto c4's 592.
- `bestiary.rs:2655-2659` — REPLACE the comment-only note with the fold
  arm `NpcKind::GoldGoldfishWalker => NpcKind::GoldGoldfish,` (the
  `ContentSamples.cs:1049` `[593] = [592]` row, verbatim).
- `ui/bestiary.rs:1615` — DROP the walker's portrait arm (it renders the
  fold target's entry now; the SleepingGolfer precedent — no own portrait).
- `render/entities.rs:1264` — KEEP `GoldGoldfishWalker => 593`
  (texture_net_id: the credit folds, the DRAW sheet does not — the
  ForceBubble precedent). The walker keeps its own NPC_593 sheet.
- `npc/data.rs` cs_id 593 / to_u16 593 / from_u16 593 — KEEP (the walker
  is a real kind; only its bestiary CREDIT folds). `try_entry_for_npc`
  stays unfolded (returns the walker's own… now None → the s1_f_3
  cs_id==net invariant exempts it, the SleepingGolfer shape).
- ENTRIES pin: d42 277 → **276** (−1 interim row); the 108-R9 union 486
  was computed WITH the interim row → **485** (c4's 145 already flags
  "ENTRIES pin −1"; d37d re-derives).

This is a MERGE-time action (c4's 592 row must be present); d42 keeps the
interim row on-branch until then so `entry_for_npc`'s `.expect` never
fires for the walker. The contract is published here + in c4's 145/148.

## 3. The 592/607/615 re-adjudication (FINAL) + the 405/406 question

**The conflict:** 108-E3 (from d42's OLD tip `9b22c091`) said
`keep d42@592/607/615, drop c4`. d42's reconcile `49346dba6` then DROPPED
those three (ceding to c4b). c4's 145 withdrew its retire verdicts:
**c4 @592/607/615 STAND.** These are NOT actually in conflict once the
timeline is read: E3's "keep d42" assumed d42 still owned them; d42 ceded;
c4 is now the ONLY port. **FINAL: c4 owns GoldGoldfish 592 / Pupfish 607 /
Dolphin 615 (protocol==vanilla, pushed `479d70ad2`).** d42 has no rows for
them (already dropped) — nothing to reconcile on d42's side. The same
holds for FlyingAntlion 581 / EyeballFlyingFish 587 (c4 style-44 STANDS).

**The 405/406 question (d42-vanilla-free-ids vs c4-port-local-405/406):**
c4's renumber `91293a3d2` moved CorruptGoldfish 229→**405** and Arapaima
237→**406** (the "free-port band" above d37's 354-399). Per the 108-E plan
these are adjudicated as follows (d37d reads, re-verified here):

- **CorruptGoldfish: keep d34@277, retire c4@405 at merge.** d34's
  phase-3 `5f3435a87` ports the full kind at 277/cs 57 (kb 1.0, def 6,
  hp 100, dmg 30, value 500, `{31}`, vanilla_type 57, update_swimmer,
  inline case-57 FindFrame — behaviour-identical to c4's helper). E3's
  kinds-lane rule (d34 owns over wiring-lane c4) decides; c4's 145 §flags
  already concedes ("keep d34@277, retire c4@405"). c4's
  `style16_find_frame` case group is the FindFrame keeper (drop d34's
  inline copy). **405 is NOT a canonical id** — it is c4's branch-local
  proto for a kind d34 owns at 277.
- **Arapaima: 108-R5 says 237→762; c4 executed 237→406.** R5's rationale
  was "first free id above the canonical small-lane/g8 block" (762). c4's
  406 is ALSO free in the canonical plan (base proto max 353; d37 owns
  354-399; 400-428 unoccupied; d40 owns 430-488). **Both 406 and 762 are
  collision-free**; the question is which is canonical. Per the E-plan's
  principle (port-local kinds ride the 698+ block to stay above
  NPCID.Count and clear of every vanilla-free lane), **762 is the
  principled target**; but c4's 406 is pushed, witness-pinned, and
  collision-free, and Arapaima's cs 157 is base-owned (BoundWizard) so
  vanilla-free is impossible either way. **Verdict: ACCEPT c4@406 as
  canonical** (the pushed, verified, collision-free id; moving it to 762
  buys nothing — no lane contests 406, and the 698+ principle is a
  collision-avoidance heuristic, not a hard invariant). d37d's R5 "762"
  is superseded by the pushed-406 reality; flagged for d37d's final
  receipt. **406 is NOT a d42 id** — d42's vanilla-free band is 578-630,
  it never claimed 405/406.
- **The cs-space overlap is NOT a wire bug:** 405/406 are the vanilla
  StardustCellBig/Small cs ids, but those kinds ride base proto 121/116
  (verified `origin/main` data.rs:677/706) — proto 405/406 are FREE. c4's
  port-local use is the d35/d37 precedent (proto ≠ cs is fine for
  port-local kinds). No collision with d42's vanilla-free 578-630.

**Net for d42:** none of 405/406/592/607/615/581/587 are d42's. d42's
band is the vanilla-free 578-630 minus the 8 ceded + 11 keep-one-dropped
= **32 owned kinds** (§4).

## 4. The d42 band declaration (FINAL)

**Branch `task/W4-d42-npc-kinds-561-630` owns 32 kinds, protocol==NPCID
(the vanilla-free 578-630 band, the 108-E3/d38/d40 invariant):**

```
578 DD2LightningBug      579 BartenderUnconscious  580 WalkingAntlion
582 LarvaeAntlion        583 FairyCritterPink      584 FairyCritterGreen
585 FairyCritterBlue     586 ZombieMerman          590 TorchZombie
591 ArmedTorchZombie     593 GoldGoldfishWalker    594 WindyBalloon
610 Rat                  611 Owl                   612 WaterStrider
613 GoldWaterStrider     614 ExplosiveBunny        616 Turtle
617 TurtleJungle         618 BloodNautilus         619 BloodSquid
620 GoblinShark          621 BloodEelHead          622 BloodEelBody
623 BloodEelTail         624 Gnome                 625 SeaTurtle
626 Seahorse             627 GoldSeahorse          628 Dandelion
629 IceMimic             630 BloodMummy
```

Holes kept (cs-reserved / sibling-owned, never reassign):
- **581/587** → c4 style-44 (FlyingAntlion/EyeballFlyingFish).
- **588/589** → base Golfer/SleepingGolfer (proto 163/164, cs 588/589 —
  the cs-reservation rule, 108-E8 d42 note).
- **592/607/615** → c4 style-16 (GoldGoldfish/Pupfish/Dolphin).
- **595-601** → d33 (the 7 dragonflies, keep-one §1).
- **602/603/608/609** → c10 (the 4 waterfowl, keep-one §1).
- **604/605** → e26b (LadyBug/GoldLadyBug).
- **606** → c8 (Maggot).

The band is contiguous-free of every sibling claim; max proto 630 ≪
i16::MAX (the round-trip test guards). The 32 kinds' stats/AI/FindFrame/
bestiary/sounds/immunity are the data-only join (ANALYSIS "The NPCID
578-630 roster join"); the wired families (aiStyle-3 walkers, the
BloodSquid hover — now bob/boost/rise-complete, the GoblinShark spin)
ride their machines; the unwired styles 111-119/68/24/107/25-for-629/0
idle in place (the wyvern precedent). Sheets deferred to d35's
research/113 §10 lazy-load wave (research/238).

**Lane status: FINAL.** The roster join + reconcile + sheets inventory +
rotation census + BloodSquid hover gap-closure + this keep-one/fold/
re-adjudication contract are the lane's deliverables. The drop-hunks
(§1a) + the fold re-point (§2) are EXECUTE-prep for the merger; d42's
branch keeps its rows so the exhaustive matches compile until the merge
wave applies them.

## 5. Witness register (this phase)

- BloodSquid hover (Phase A, `33b3cfe07`): 3 tests in npc_ai.rs
  (`w4_d42_the_blood_squid_bob_drifts_every_tick_and_wraps`,
  `…_close_boost_and_rise_gates`, `…_hover_bob_gates_are_blood_squid_only`)
  — WRITTEN NOT RUN; a node simulation of the ported order re-verified
  every asserted value (ALL-WITNESS-ARITHMETIC-OK, 14 checks).
- This doc: the drop-hunk line ranges were read from the clean tree at
  `33b3cfe07` (git grep -n per kind, every exhaustive-match site walked);
  the owner rows were read at d33 `b08acf463` / c10 `5d8387955` / c4
  `479d70ad2` (git-show, read-only). No compiler/tests/rustfmt/graphify
  invoked (code-only mode). No foreign working-tree writes; no junction
  recursion (direct file paths only).

Code-only: every claim cites a pushed tip or a named local commit; the
peer-session rotation commit `2f6b99947` was absorbed (the entities.rs
ranges are post-it). The 143→238 rename (`9eadc3043`) follows
registry-150's late-push rule; 239 is this doc's free number.
