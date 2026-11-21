# 91 — D22 ROSTER-GAP TABLE: the 123 unrowed C# aiStyle-3 types (d34-d43 lane input)

W4-d22 retire phase R2, branch `task/W4-d22-npc-ai-styles-3-6` (base DONE
`bcd42ddd`, R1 `fb99a047`). Code-only lane: no cargo/tests invoked. This
table is the **d37d-E6 input**: every C# `SetDefaults` `aiStyle = 3` type
with NO port kind at the d22 tip, its corridor lane, the lane's pushed-tip
claim status, and the AI_003 arms a faithful port must carry beyond the
shared core.

## 0. Census method + the corrected roster

Scripted sweep of `decompiled/Terraria/NPC.cs` `SetDefaults`
(`:8426-18093`): every `this.aiStyle = 3;` line stamps the enclosing
`this.type == N` / `|| this.type == M` / `case N:` / range gate. The
port side: `npc/data.rs::ai_style()`'s style-3 arms (68 kinds) resolved
through `cs_id()` (the hand rows + the bestiary `ENTRIES.net_id`
fallback). Result: **191 C# types, 68 ported, 123 gaps** — the same
totals as the phase-4 census (`bcd42ddd`), with two composition
corrections:

* **590 + 591 BOTH rowed** — the `:16666` gate is
  `this.type == 590 || this.type == 591` (one `aiStyle = 3` line stamps
  two types; a single-type parse drops 591 ArmedTorchZombie).
* **368 TravellingMerchant is NOT a SetDefaults style-3 type** — its
  `:53412` flip is the RUNTIME `CritterThatCanTurnOnPlayers` turn-on
  inside `AI_007_TownEntities` (`:53392-53413`, the town/critter
  machine), not a roster row. The port's TravellingMerchant rides
  aiStyle 7 (`npc/data.rs:3112`) — correct; the runtime flip belongs to
  the town lane (d120), not the fighter band.

The 68 ported kinds are ALL routed (phase-4 item-4 census, ANALYSIS
"W4-d22r"): 22 on the axis core (`npc_ai_style3::claims`), 17 zombie
variants on `update_zombie`, the shooters/frost siblings/martian
head-arms/pillar mobs/summoner on their machines. **No AI-arm gaps for
ported kinds — the 123 below are KIND-roster gaps.**

## 1. The gap table (123 types)

Columns: **type** (NPCID) | **name** (`NPCID.cs`) | **SetDefaults**
(the `aiStyle = 3` line) | **lane** (the d34-d43 corridor that owns the
id band; g8 where noted) | **tip** (claim status at the lane's PUSHED
tip, probed 2026-11-21: `ROWED` = cs_id/bestiary row + style-3 arm
present; `—` = unclaimed at tip) | **AI_003 extras** (the arms beyond
the shared core + ladder; `core` = the generic walker is the whole
machine; ladder rows live in `npc_ai3_speeds.rs`, W40 `bddc7e03`).

Lane tips probed: d34 `5f3435a8`, d35 `e5558b0a`, d36 `603c967a` /
d36x `396a35ce` (E4 re-pin in flight, kinds unpushed), d37 `a341daa1`,
d38 `1ba0395c`, d39 `ef22450f`, d40 `8fe1d6d3`, d41 `5d036721`,
d42 `2d7ac08c`, d43 `a725acb0` (AT BASE — no commits), g8 `82c93cec`.

| type | name | SetDefaults | lane | tip | AI_003 extras |
| --- | --- | --- | --- | --- | --- |
| 31 | AngryBones | :9112 | d34 | ROWED | ladder :57606 (num83), moan 1000 (:57340), bash +6 (:60031), charge-hop (:60205) |
| 47 | CorruptBunny | :9352 | d34 | ROWED | flag8 false (:57072), ladder :57606, charge-hop (:60205) |
| 67 | Crab | :9644 | d34 | — | flag8 false (:57072), ladder :57881 |
| 73 | GoblinScout | :9737 | d35 | ROWED | NotDiscouraged-exempt (:60302), ladder :57606 |
| 77 | ArmoredSkeleton | :9790 | d35 | ROWED | moan 1000, flag24 bash (:59996), charge-hop |
| 78 | Mummy | :9803 | d35 | ROWED | moan 500 case 26 (:57348), ladder :57907 |
| 79 | DarkMummy | :9816 | d35 | ROWED | moan 500, ladder :57907/:57916 |
| 80 | LightMummy | :9829 | d35 | ROWED | moan 500, ladder :57907 |
| 104 | Werewolf | :10234 | d35 | ROWED | ladder :57554, charge-hop (:60205) |
| 109 | Clown | :10303 | d35 | ROWED | flag8 false, ladder :57580, bomb arm :59802, wide probe (:59953) |
| 110 | SkeletonArcher | :10317 | d35 | ROWED | SHOOTER block :59069+ (ai[1]/ai[2] own), moan 1000, flag10 freeze (:57085) |
| 120 | ChaosElemental | :10457 | d35 | ROWED | num57=180 + teleport arm (:57036-57064, 40 dust), ladder :57520, ×1.1 (:60258) |
| 140 | PossessedArmor | :10767 | d35 | ROWED | core + ladder :57606 |
| 159 | Vampire | :15485 | d36 | — | moan 500 case 29/7 (:57352), bat transform :58843, ±6 clamp (:57403-57409) |
| 162 | Frankenstein | :15428 | d36 | — | moan 1000 AND 500 case 29/6 (DOUBLE-match, :57340/:57356), ladder :57554 |
| 163 | BlackRecluse | :11034 | d36 | — | spider wall machine :58855-58902, step-up +8 (:59928), wide probe; d36x note: "163 no-entry" bestiary fold decision |
| 164 | WallCreeper | :11063 | g8 | ROWED | spider wall machine :58880-58902, step-up +8, wide probe; g8 keeps the `update_skeleton` kind arm (NOT axis-claimed) |
| 166 | SwampThing | :15441 | d36 | — | head block :55938, flag8 false, ladder :57520 |
| 167 | UndeadViking | :11092 | d36 | — | moan 1000, flag24 bash, ladder :57606 |
| 168 | CorruptPenguin | :11105 | d36 | — | flag8 false, charge-hop, ladder :57606 |
| 181 | FaceMonster | :11296 | d36 | — | moan 500 case 29/8 (:57360), ladder :57606/:57609 |
| 185 | SnowFlinx | :11365 | d36 | — | ladder :57606, :58820 |
| 196 | Nymph | :11510 | g8 | ROWED | ladder :57554, flag24 bash, charge-hop; g8 keeps the `update_skeleton` kind arm; the LostGirl transform is the kind lane's |
| 197 | ArmoredViking | :11524 | d36 | — | moan 1000, ladder :57554, flag24 bash |
| 198 | Lihzahrd | :11537 | d36 | — | :57027, ladder :57606; d36x note: "198->199 fold" decision |
| 199 | LihzahrdCrawler | :11550 | d36 | — | flag8 false, crawler ladder :57486, wide probe |
| 201 | HeadacheSkeleton | :11576 | d36 | — | moan 1000, ladder :57625/:57693, flag24 bash |
| 202 | MisassembledSkeleton | :11589 | d36 | — | moan 1000, ladder :57629/:57693, flag24 bash |
| 203 | PantlessSkeleton | :11602 | d36 | — | moan 1000, ladder :57633/:57693, flag24 bash |
| 206 | IcyMerman | :11644 | d36 | — | SHOOTER block :59069+ (:59131/:59219/:59284), flag8 false, flag10 freeze |
| 217 | CochinealBeetle | :11792 | d37 | ROWED | flag8 false, ladder :57606 |
| 218 | CyanBeetle | :11805 | d37 | ROWED | flag8 false, ladder :57606 |
| 219 | LacBeetle | :11818 | d37 | ROWED | flag8 false, ladder :57606 |
| 220 | SeaSnail | :11831 | d37 | ROWED | flag8 false, ladder :57881 |
| 236 | JungleCreeper | :12053 | d37 | ROWED | spider wall :58855-58902, step-up +8, wide probe |
| 239 | BloodCrawler | :12082 | d37 | ROWED | spider wall :58889-58902, step-up +8, wide probe, flag8 false |
| 243 | IceGolem | :12137 | d37 | ROWED | laser arm :58907, ladder :57972, flag8 false |
| 251 | Eyezor | :12238 | d37 | ROWED | laser arm :58944, ladder :58002, flag8 false |
| 254 | ZombieMushroom | :12267 | d37 | ROWED | core + ladder :57606 |
| 255 | ZombieMushroomHat | :12280 | d37 | ROWED | core (default ladder arm) |
| 257 | AnomuraFungus | :12308 | d37 | ROWED | flag8 false, ladder :57554, wide probe |
| 258 | MushiLadybug | :12322 | d37 | ROWED | flag8 false, ladder :57520/:58434, wide probe |
| 269 | RustyArmoredBonesAxe | :12488 | d37 | ROWED | moan 1000 (269-280 range :57364), ladder :57744 |
| 270 | RustyArmoredBonesFlail | :12501 | d37 | ROWED | moan 1000, ladder :57748 |
| 271 | RustyArmoredBonesSword | :12514 | d37 | ROWED | moan 1000, ladder :57752 |
| 272 | RustyArmoredBonesSwordNoArmor | :12527 | d37 | ROWED | moan 1000, ladder :57756 |
| 273 | BlueArmoredBones | :12540 | d37 | ROWED | moan 1000, ladder :57760 |
| 274 | BlueArmoredBonesMace | :12553 | d37 | ROWED | moan 1000, ladder :57764 |
| 275 | BlueArmoredBonesNoPants | :12566 | d37 | ROWED | moan 1000, ladder :57768 |
| 276 | BlueArmoredBonesSword | :12579 | d37 | ROWED | moan 1000, ladder :57772 |
| 277 | HellArmoredBones | :12592 | d37 | ROWED | moan 1000, ladder :57776 |
| 278 | HellArmoredBonesSpikeShield | :12605 | d37 | ROWED | moan 1000, ladder :57780 |
| 279 | HellArmoredBonesMace | :12618 | d37 | ROWED | moan 1000, ladder :57784 |
| 280 | HellArmoredBonesSword | :12631 | d37 | ROWED | moan 1000, ladder :57788 |
| 287 | BoneLee | :12728 | d38 | ROWED | dash ladder :57944, post-ladder dash :60262-60268 |
| 290 | Paladin | :12773 | d38 | ROWED | SHOOTER block (hammer :59147/:59223/:59288/:59490), flag8 false |
| 291 | SkeletonSniper | :12787 | d38 | ROWED | SHOOTER :59115/:59311, moan 1000, flag10 freeze |
| 292 | TacticalSkeleton | :12801 | d38 | ROWED | SHOOTER :59119/:59311, moan 1000, flag10 freeze |
| 293 | SkeletonCommando | :12815 | d38 | ROWED | SHOOTER :59123/:59227, moan 1000, flag10 freeze |
| 294 | AngryBonesBig | :12829 | d38 | ROWED | moan 1000, ladder :57613, bash +6, charge-hop |
| 295 | AngryBonesBigMuscle | :12842 | d38 | ROWED | moan 1000, ladder :57617, bash +6, charge-hop |
| 296 | AngryBonesBigHelmet | :12855 | d38 | ROWED | moan 1000, ladder :57621, bash +6, charge-hop |
| 322 | SkeletonTopHat | :13079 | d38 | ROWED | core (default ladder), flag24 bash |
| 323 | SkeletonAstonaut | :13092 | d38 | ROWED | core (default ladder), flag24 bash |
| 324 | SkeletonAlien | :13105 | d38 | ROWED | core (default ladder), flag24 bash |
| 379 | CultistArcherBlue | :13843 | d39 | ROWED | SHOOTER :59099/:59412, bow arm :56181, flag10 freeze |
| 380 | CultistArcherWhite | :13843 | d39 | — | SHOOTER :59099/:59412, bow arm :56181, flag10 freeze (the :13843 gate rows BOTH 379/380) |
| 430 | ArmedZombie | :8616 | d40 | — | charge family ladder (:58262-58321), flag24 bash; the stop :58291-58304 |
| 431 | ArmedZombieEskimo | :8629 | d40 | — | charge family ladder, NOT flag24 (accumulates every night) |
| 432 | ArmedZombiePincussion | :8642 | d40 | — | charge family ladder, NOT flag24 |
| 433 | ArmedZombieSlimed | :8655 | d40 | — | charge family ladder, NOT flag24 |
| 434 | ArmedZombieSwamp | :8668 | d40 | — | charge family ladder, NOT flag24 |
| 435 | ArmedZombieTwiggy | :8681 | d40 | — | charge family ladder, NOT flag24 |
| 436 | ArmedZombieCenx | :8694 | d40 | — | charge family ladder, NOT flag24 |
| 449 | BoneThrowingSkeleton | :14351 | d40 | — | SHOOTER-excluded ladder (:58322 list), moan 1000 (449-452), flag24 bash |
| 450 | BoneThrowingSkeleton2 | :14364 | d40 | ROWED | SHOOTER-excluded ladder, moan 1000, flag24 bash |
| 451 | BoneThrowingSkeleton3 | :14377 | d40 | ROWED | SHOOTER-excluded ladder, moan 1000, flag24 bash |
| 452 | BoneThrowingSkeleton4 | :14390 | d40 | ROWED | SHOOTER-excluded ladder, moan 1000, flag24 bash |
| 460 | Butcher | :15498 | d40 | ROWED | kb wet/dry head (:57219-57228), ladder :58070, ALWAYS-OPEN door (:60042), ×1.3 (:60273) |
| 461 | CreatureFromTheDeep | :15511 | d40 | ROWED | head :55972 (kb), ladder :57554 |
| 462 | Fritz | :15524 | d40 | ROWED | charge-jump head (:57282-57313), ladder :57657/:57736 |
| 463 | Nailhead | :15538 | d40 | ROWED | spike burst (:57151-57217, Next(3)+Next(3,6)+proj 498 — DRAWS), ladder :57661 |
| 464 | CrimsonBunny | :14475 | d40 | ROWED | flag8 false, charge-hop, ladder :57606 |
| 466 | Psycho | :15551 | d40 | ROWED | stealth head (:55901-55936, alpha fade), ladder :57665/:57706, flag10 freeze |
| 468 | DrManFly | :15591 | d40 | ROWED | SHOOTER :59267/:59332/:59420/:59466 (flask toss), flag8 false |
| 469 | ThePossessed | :15578 | d40 | ROWED | hover/kb head (:57230-57280, wall-scan + noGravity flight), ladder :57669 |
| 470 | CrimsonPenguin | :14500 | d40 | ROWED | flag8 false, charge-hop, ladder :57606 |
| 480 | Medusa | :14526 | d40 | ROWED | head :56356 (gaze), ladder :57673, flag8 false |
| 481 | GreekSkeleton | :14540 | d40 | ROWED | SHOOTER :59135/:59263/:59280, moan 1000, flag8 false |
| 482 | GraniteGolem | :14553 | d40 | ROWED | head :56252 (shield), ladder :57606, flag8 false |
| 494 | Crawdad | :14670 | d41 | ROWED | charge family ladder (:58262-58321, the 494/495 pair) |
| 495 | Crawdad2 | :14670 | d41 | — | charge family ladder (the :14670 gate rows BOTH 494/495) |
| 498 | Salamander | :14695 | d41 | ROWED | flag10 freeze (:57104), SHOOTER-excluded ladder; the :14695 gate rows 498-506 |
| 499 | Salamander2 | :14695 | d41 | ROWED | flag10 freeze (:57105), SHOOTER-excluded ladder |
| 500 | Salamander3 | :14695 | d41 | — | flag10 freeze (:57106), SHOOTER-excluded ladder |
| 501 | Salamander4 | :14695 | d41 | — | flag10 freeze (:57107), SHOOTER-excluded ladder |
| 502 | Salamander5 | :14695 | d41 | — | flag10 freeze (:57108), SHOOTER-excluded ladder |
| 503 | Salamander6 | :14695 | d41 | — | flag10 freeze (:57109), SHOOTER-excluded ladder |
| 504 | Salamander7 | :14695 | d41 | — | flag10 freeze (:57110), SHOOTER-excluded ladder |
| 505 | Salamander8 | :14695 | d41 | — | flag10 freeze (:57111), SHOOTER-excluded ladder |
| 506 | Salamander9 | :14695 | d41 | — | flag10 freeze (:57112), SHOOTER-excluded ladder |
| 508 | GiantWalkingAntlion | :14832 | d41 | — | ladder :58139, wide probe, flag8 false |
| 524 | DesertGhoul | :14886 | d41 | — | charge family (524-527 :58410-58418), charge-hop; the :14886 gate rows 524-527 |
| 525 | DesertGhoulCorruption | :14886 | d41 | — | charge :58410, charge-hop |
| 526 | DesertGhoulCrimson | :14886 | d41 | — | charge :58414, charge-hop |
| 527 | DesertGhoulHallow | :14886 | d41 | — | charge :58418, charge-hop |
| 528 | DesertLamiaLight | :14924 | d41 | — | ladder :57520, flag8 false; the :14924 gate rows 528/529 |
| 529 | DesertLamiaDark | :14924 | d41 | — | ladder :57520, flag8 false |
| 530 | DesertScorpionWalk | :14937 | d41 | — | spider wall :58892-58902, step-up +8, wide probe |
| 532 | DesertBeast | :14964 | d41 | — | ladder :58203/:58232, wide probe, flag8 false |
| 534 | DemonTaxCollector | :15817 | d41 | — | core (default ladder; the TaxCollector's demon form — the town lane's transform feeds it) |
| 536 | TheBride | :15847 | d41 | — | core (default ladder; the Groom's pair) |
| 580 | WalkingAntlion | :14858 | d42 | ROWED | ladder :58139, wide probe, directionY step-up gate (:59872), flag8 false |
| 582 | LarvaeAntlion | :16573 | d42 | ROWED | ladder :58139/:58144/:58183, wide probe, flag8 false |
| 586 | ZombieMerman | :16612 | d42 | ROWED | moan 800 (:57344, the 489 pair), wet kb head (:56055), ladder :57689/:57710/:57726, expert hop :60209-60241 (constants-off) |
| 590 | TorchZombie | :16670 | d42 | ROWED | torch head :56914, moan 1000, flag24 bash |
| 591 | ArmedTorchZombie | :16670 | d42 | ROWED | torch head :56948, moan 1000, charge family ladder (:58262), flag8 false, NOT flag24 |
| 624 | Gnome | :17030 | d42 | ROWED | STATUE TRANSFORM (:55863-55899, dust + PlaceTile 567 + Next(5) — DRAWS), chase-exempt head (:57315-57334), ladder :58325, gnome jump (:60113-60122) |
| 630 | BloodMummy | :17111 | d42 | ROWED | moan 500 case 26, ladder :57907/:57916 |
| 631 | RockGolem | :17125 | d43 | — | head :56291, ladder :57685, flag8 false, NotDiscouraged ai[2] arm (:60302) |
| 632 | MaggotZombie | :17138 | d43 | — | moan 1000, ladder :58357 |
| 635 | SporeSkeleton | :17179 | d43 | — | moan 1000, ladder :57693, flag24 bash |
| 691 | MossZombie | :17700 | d43 | — | moan 1000, ladder :58361, flag24 bash |

Tip-claim totals: **75 ROWED at pushed tips** (d34 2, d35 10, d37 24,
d38 11, d39 1, d40 15, d41 3, d42 7, g8 2), **48 unclaimed** — d34's 67;
d36's whole 159-206 band (15 types, the E4 re-pin in flight); d39's 380;
d40's 430-436+449 (8); d41's 495/500-506/508/524-532/534/536 (18); d43's
whole band (4, lane at base). The d36x tip carries two fold decisions
the E6 pass must read: **163 no-entry** (bestiary) and **198->199**.

## 2. Cross-notes for the kind lanes (the d37d-E6 work items)

1. **`fighter_walker_kind` MUST extend with each new walker kind**
   (`ambient_sound.rs:603`). The idle counter + the moan emitter ride
   that gate; a rowed kind missing from it never ticks `ai[3]` and
   never rolls its C# moan — a per-tick UpdateNPCs stream shortfall.
   The gap types WITH moan draws (the extension list):
   **31, 77, 78, 79, 80, 110, 159, 162 (DOUBLE-arm), 167, 181, 197,
   201, 202, 203, 269-280, 291, 292, 293, 449-452, 481, 586 (the 800
   arm), 590, 591, 630, 632, 635, 691**. The moan position is the
   PRE-SLOT head block (R1 `fb99a047`, the C# `:57338` order — BEFORE
   the machine's draws); the emitter table itself
   (`fighter_ambient_moans`) already names every id above, so the
   extension is the KIND GATE only.
2. **The axis claim is the faithful route for the `core` rows.** The
   d-lane tips dispatch their new style-3 kinds through
   `walker_step_fighter` KIND arms (the 80×40-tile aggro stand-in);
   d22's axis core (`npc_ai_style3::fighter_core_step`) is the faithful
   shared machine. On merge the kind is NOT double-claimed (the
   `claims()` gate names only the 22), but the E6 pass should move each
   plain-walker gap kind onto `claims()` + retire its kind arm (the
   ladder rows already exist — W40 ported ~155 types 1:1,
   `npc_ai3_speeds.rs`). The SHOOTER-block types (110/206/290-293/
   379/380/449-452/468/481/498-506) keep their own machines — the
   `:58322` ladder exclusion list names them; do NOT claim them.
3. **The draw-bearing head arms are per-kind ports, not core folds:**
   120's teleport (40 dust), 463's spike burst (proj 498), 624's statue
   transform (dust + PlaceTile + Next(5)), 109's bomb arm. Their kinds
   must thread the UpdateNPCs stream + the dust pool (the style-4/5
   census-fix shape, `c1d26278`/`204e0651`).
4. **The spider/wall family (163/164/236/239/530)** shares the
   :58855-58902 wall machine + the step-up +8 list (:59928) + the wide
   probe (:59953) — the scan generalisation (`fighter_obstacle_scan_g`)
   already computes all three from the live type; only the wall-climb
   head is per-kind.
5. **The flag24 bash list** (`DOOR_BASH_TYPES`, :59996) already names
   the gap ids 77/167/197/201-203/322-324/430/449-452/590/635/691 —
   the drain needs no extension when those kinds land (the blood-moon
   census correction `9bed77313` applies: they reset on a normal night,
   accumulate on a blood moon / graveyard roll). NOTE 431-436 and 591
   are NOT in flag24 (only the base 430/590 rows are) — like the Blood
   Zombie 489 they accumulate on EVERY night.
6. **368 TravellingMerchant's runtime flip** (:53412) is the TOWN
   lane's (d120), not a fighter-band row — see §0.

## 3. Provenance

Census scripts (read-only, local temp): the SetDefaults sweep (the
`||`-gate + range-gate parse, the :18094 method bound), the port
`ai_style()`/`cs_id()`+ENTRIES resolution, the 13-lane pushed-tip probe
(`git show` per ref, no checkout). Every table row re-readable at its
cited `NPC.cs` line; the lane claims re-derivable from the cited tips.
Code-only: no cargo/tests/rustfmt/graphify invoked.
