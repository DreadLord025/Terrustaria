# 250 — ROSTER LAGGARD-EXEC: the laggard probes, the executed-order receipts, the FINAL v2 master tables (74 moves / 144 drops)

Lane: `task/W4-d37-npc-kinds-211-280`, worktree `TR-W4-d37`, parent SEAL
`fbb948172` (research/108 S0-S4). **Code-only mode — no cargo/tests/rustfmt/
graphify invoked; doc + witness scripts only, zero code diff.** Number
**250** per the registry-150 rule: a fresh all-`task/W4-*`-tip
`ls-tree docs/research/` sweep (360 refs) + a local-worktree scan find
248 taken (c27/d3/e26), **249 taken locally** (d15's unpushed
`249-projectile-ai-141-150-final.md`), 250 free at every tip and worktree.

Companion inputs read at their named tips (git-show / read-only worktree
probes, NO junction recursion): research/108 (this branch, SEAL
`fbb948172`), d42 research/239 (`bac408be0`), d43 research/244
(`6fbe06250`), c4 research/143/148 (`8abf0f1d8`), g8 research/108-input-g8
(`1752c70df`), d35 research/113 (`8fa3b3dbf`), d54 research/150.

**The wave moved DURING this phase.** The SEAL's laggard list was derived
at ~12:49; between 14:19 and 15:57 the lanes pushed 20+ commits that
consumed several E8 orders and created new dups. Every table below is
pinned to the tip hash in its header — re-verify with
`git grep -n <Kind> <tip> -- crates` if the tip advances (the d42/239
precedent: the SYMBOL is the durable key, lines drift).

---

## 1. Laggard status board (the six ROSTER items, probed live)

| Lane | SEAL flag | Probe verdict (15:57 read) | Status |
|---|---|---|---|
| c9 | zero movement, stale ALL_KINDS, S2 patch unapplied | **ALIVE.** Local commit `c830aec3d` (12:49) fixed ALL_KINDS itself (277 rows incl the 4 snails); PUSHED as `9252d6f31` (15:57) with GoldBunny/GoldFrog on top → decl/rows/pin **279/279/279 bijective**. The S2 patch is **SUPERSEDED — DO NOT APPLY** (git-apply-check exit 1 at the new tip; exit 0 only at the stale origin `2042d77f1`). | S2 CLOSED by the lane; 2 moves + 6 drops owed (§3) |
| g8 | zero movement, stale ALL_KINDS | **ALIVE.** Machine rounds 3+4 landed (`5767ca7c7` VampireBat/Vampire, `058453096` FlyingSnake) + claims-table `1752c70df` PUSHED (12:20→15:0x). Worktree CLEAN. **ALL_KINDS decl 270 vs 305 rows — DOES NOT COMPILE** (array-length mismatch, not just a test failure). New kind **Vampire@388 squats d37's RustyArmoredBonesAxe**. | 18 moves + 17 drops owed; compile-fix hunk §4 |
| d125 | origin REGRESSED (d8047a109 lost the sheets/draw commits) | **RESOLVED — no rescue needed.** The lane pushed everything itself: origin `7a0482302` (12:22) contains `cd2b1b1e3`/`86e3adfd3` (merge-base --is-ancestor exit 0, scripted), then EXECUTED both E8 moves → origin `5a26ae348` (15:44): EmpressOfLight **229→636**, PrismaticLacewing **237→742**. Local == origin, clean. **Remaining defect: s1_f_3 literal pin 270 vs 272 arms** (decl fixed to 272; the test fails). | moves DONE; pin-fix §4; 0 drops |
| d42 | 7 dragonfly drops owed | **ALIVE.** `bac408be0` (12:55) = research/239 FINAL doc: 11-kind keep-one drop-hunks + the 593-fold re-point + the 592/607/615 re-adjudication. **239's 4-waterfowl-drop half is MOOT** — it read the PRE-SEAL 108 (`a341daa12`); the SEAL (`fbb94817`, 12:49) flipped the waterfowl to d42 (c10 drops its dups). Only the **7 dragonflies** drop. 36/36 moves verified DONE at tip (scripted). | 7 drops owed; exact 7-only hunks §5 |
| c4 | 3 drops owed | **ALIVE.** `8abf0f1d8` (15:21) SEAL-VERIFY receipt. The 148 conditional **FIRED**: d43 rowed 688/692 (`fd6a544f`/`8fc608bc`) before the merge → per c4's own published contract + d43's research/244, **Pufferfish/Orca flip to d43; c4's drops grow 3→5**. c4's keeper implementations (the `just_hit` one-tick puff fix `17a241f6a`, the Orca wet-anchor, research/144's NPC_692 census) fold onto d43's rows at merge. | 5 drops owed (§3); fold-contract §6 |
| d37 (this lane) | 69-moves/61-drops master table owed | **DELIVERED** — re-derived at LIVE tips: **74 moves / 144 drops** (§3, the full per-lane tables in `tools/_d37_lag_tables2.md`). The SEAL's 69/61 grew because d23/d34/c9 pushed new dup ports and f18/f42/e21/g62/d9/d11/g5/f47 surfaced as claimants the SEAL census had not itemized. | this doc + the witness scripts |

**108 §S4 SEALED-receipt:** the SEAL commit `fbb948172` is confirmed at
origin (`origin/task/W4-d37-npc-kinds-211-280` == local HEAD, clean); its
S2 patch artifact `tools/_d37_c9_allkinds_fix.patch` re-verified
`git apply --check` **exit 0** against the byte-exact `2042d77f1` npc.rs
blob (temp-dir check, scripted) — and **exit 1** against c9's new tip,
which is the supersession proof. The S4 pins are superseded by §7 below
(the wave added 107 kinds since the SEAL's 282-core/313-grand).

## 2. The executed-order receipts (consumed since the SEAL)

| Lane | Order | Executed at | Evidence |
|---|---|---|---|
| d42 | 36 moves 354-404→578-630 | `49346dba6` (pre-SEAL) | re-verified at `bac408be0`: all 36 at FINAL ids, scripted (`_d37_lag_verify.py`) |
| c4 | Arapaima→406 | `91293a3d2` | at tip `8abf0f1d8` |
| d125 | EmpressOfLight 229→636, PrismaticLacewing 237→742 | `5a26ae348` (15:44) | to_u16 arms read at tip |
| e26 | Frog 361→744; Duck drop | `a23c64933` (15:56) | tip has Frog@744, LadyBug/GoldLadyBug@604/605, NO Duck |
| d36 | 5 re-pins 762-766→767-771 | `8c5839b03` (14:28) | ToxicSludge 767/Penguin 768/PenguinBlack 769/GiantTortoise 770/IceTortoise 771 — exactly the ids this phase's first-pass adjudication assigned; PLUS new Crimera 781/MossHornet 782/Moth 793 (free, ACCEPT) |
| d39 | 7 kinds 781-787→817-823 (R7) | `3caf10cff`+`a0d49dc1d` (12:03) | ACCEPT as-pushed; supersedes the SEAL's 781-787 acceptance (d36's Crimera/MossHornet now sit at 781/782 — collision-free only BECAUSE d39 moved) |
| c9 | ALL_KINDS 4-snail fix (S2) | `c830aec3d`→pushed `9252d6f31` | decl/rows/pin 279/279/279, snails+golds all in array (scripted) |
| d41 | 494-506 band + batch 4-6 | `1a1b07ff2` (15:39) | 33 kinds at 494-531; ALL_KINDS 303/303/303 |
| d43 | full 631-697 band + retire doc | `0253bde90`→`6fbe06250` (15:55) | research/244: 18 drops self-adjudicated (matches this doc's verdicts except 666, §6) |
| g10 | 11 pets PUSHED | `5dee28650` (10:24) | 637/638/656/670/678-684 as the SEAL accepted |

**KEEPS re-confirmed at live tips:** d33 (7 dragonflies 595-601,
`b08acf463`), d34 phase-3 (276-280), d37 (46 kinds 354-399, zero-move),
d38 (35 kinds 281-337), d40 (56 kinds 430-488), d35 (33 kinds still at
429-461 — the +269 renumber NOT executed, owed), c10 (ducks still at
362-365 — owed), c7/c8 (moves owed), d120 (retired lane, 4 town moves +
1 drop ride the merge wave), d23's SolarFlare@408/VileSpitEoW@411
(EoW now drops, §6).

## 3. The FINAL v2 master tables — 74 moves / 144 drops

Derivation: `_d37_lag_final2.py` compares EVERY claimant's current-tip
`to_u16` map against the adjudicated plan (kind at target id on its owner
lane = DONE; at old id = move owed; on a non-owner lane = drop owed).
**Gates (scripted, ALL PASS): 389 plan kinds, 389 distinct ids, zero
base-270 clash, max id 823, i16-safe.** The full per-lane old→new +
drop tables WITH to_u16/from_u16 line numbers at the pinned tips:
**`tools/_d37_lag_tables2.md`** (generated by `_d37_lag_emit2.py` from
`_d37_lag_final2.json` + `_d37_lag_sites2.json`). Summary:

| Lane | tip | moves | drops | note |
|---|---|---|---|---|
| c3 | `9c54f4c0e` | 0 | 2 | CursedSkull→d34@267, GiantCursedSkull→d38@289 |
| c4 | `8abf0f1d8` | 0 | 5 | +Pufferfish/Orca→d43 (the 148 conditional FIRED) |
| c6 | `5de680219` | 0 | 1 | PumpkingBlade→d38@328 |
| c7 | `c2cc6ec61` | 2 | 0 | Firefly 355→735, LightningBug 358→736; pin-fix §4 |
| c8 | `10192602a` | 3 | 5 | Worm/TruffleWorm/TruffleWormDigger→737-739 |
| c9 | `9252d6f31` | 2 | 6 | Snail/GlowingSnail→740/741; drops Shimmerfly + **5 gold-critter dups** (GoldBird/Grasshopper/Mouse/Bunny/Frog→d40) |
| c10 | `15f93d134` | 4 | 16 | ducks→731-734; 12 d40-dups + 4 d42-waterfowl; pin-fix §4 |
| d9 | `88677f9de` | 1 | 6 | AncientCultistSquidhead 521→790; 6 CultistDragon dups→d40 |
| d11 | `121672908` | 0 | 1 | BloodSquid@619 dup→d42 |
| d23 | `b00e7626f` | 3 | 19 | AngryTrapper→743 + **SporeBat 407→634, QueenSlimeMinionPurple 409→660** (cs==proto, d43 stand-ins drop); 19 drops incl VileSpitEoW (§6) + the style-14 wave's 12 dups of g8/d34/d35/d37 kinds |
| d34 | `f0d92564e` | 2 | 3 | **Crab 282→791, DungeonGuardian 283→792** (d38 owns 282/283); Antlion/PinkJellyfish/SpikeBall dups→g8 (the wired axis machines win over d34's `{}`/stand-in rows) |
| d35 | `8fa3b3dbf` | 33 | 0 | the +269 renumber 429-461→698-730, LAST in the merge order |
| d40 | `8fe1d6d31` | 0 | 1 | CrimsonGoldfish@465→c4 (research/143 exact hunk) |
| d41 | `1a1b07ff2` | 0 | 2 | GiantFlyingAntlion@509→c4, SolarFlare@516→d23@408 |
| d42 | `bac408be0` | 0 | 7 | the 7 dragonflies→d33 (§5); the 4 waterfowl STAY (SEAL flip) |
| d43 | `6fbe06250` | 0 | 20 | research/244's 18 + SporeBat/QueenSlimeMinionPurple stand-ins (§6) |
| d120 | `2408233da` | 4 | 1 | towns 241-244→763-766, BartenderUnconscious→d42@579 (retired lane; merger executes) |
| d125 | `5a26ae348` | 0 | 0 | moves DONE; pin-fix §4 |
| e21 | `4782d6631` | 0 | 1 | Bunny@229 dup→d34@270; PartyBunny@237 ACCEPT (free after d125's move) |
| e26 | `a23c64933` | 0 | 0 | ALL ORDERS EXECUTED |
| f18 | `a832d38ea` | 0 | 20 | the verify-lane's 20 kinds 237-258 are ALL dup ports of canonical lanes (c7/c8/c9/c10/d39/d40/e26); its style-axis claims dedupe onto the owners' machines at merge |
| f42 | `425d4ff9d` | 1 | 8 | Tumbleweed 354→788 (unique, cs 546); 8 dups of d34/d35/g8/c4/d42 |
| f47 | `18fe75f2e` | 0 | 1 | ShimmerSlime@676 dup→d43 (f47's light/dust wiring folds) |
| g5 | `96f074959` | 1 | 0 | RuneWizard 354→789 (unique, cs 172); LibrarianSkeleton@693 KEEP (d43/244 verdict) |
| g62 | `7ec3df62b` | 0 | 2 | GoblinShark@620→d42, Shimmerfly@677→c7 |
| g8 | `1752c70df` | 18 | 17 | 17 kinds→745-761 + **Vampire 388→762**; compile-fix §4 |

**Move mechanics (every lane):** edit ONLY the `to_u16`/`from_u16` arms
(the line numbers in `_d37_lag_tables2.md`); `cs_id`/`net_id`/bestiary
entries/`texture_net_id`/render keys/sheets are cs/net-keyed and DO NOT
move (R1's seam evidence; the Lacewing 742/cs-661 precedent). The
`ALL_KINDS` decl/rows and the s1_f_3 literal pin are COUNT-keyed — they
change only when kinds are added/dropped, never on a renumber. Branch-only
ids: no landed save wrote them (the S1-F-1 gate rule).

**Drop mechanics (every lane):** the E8 dropped-lane rule — a lane whose
session is alive executes its drops ON ITS OWN BRANCH (variant + every
arm + entry + portrait + draw + seed + witness rows; the exhaustive
matches force the census); a retired lane's drops (d120) execute AT
MERGE. The keeper's row+wiring survives; where the loser carries a
keeper-grade implementation detail (c4's puff fix, f47's shimmer wiring,
f18's axis claims), the merger FOLDS it onto the keeper (§6).

## 4. The stale-pin / compile-fix hunks (exact, at the pinned tips)

| Lane | tip | file:line | now | fix to | class |
|---|---|---|---|---|---|
| **g8** | `1752c70df` | `npc.rs:25412` | `pub(super) const ALL_KINDS: [NpcKind; 270] = [` (305 rows) | **`[NpcKind; 305]`** | **COMPILE ERROR** (array length ≠ initializer count) |
| **g8** | `1752c70df` | `npc.rs:29342` | `270,` (the s1_f_3 `covered + sentinel` literal) | **`305,`** | test pin |
| **d125** | `5a26ae348` | `npc.rs:27902` | `270,` (s1_f_3 literal; decl already 272) | **`272,`** | test pin |
| **c7** | `c2cc6ec61` | `npc.rs:27888` | `270,` (s1_f_3; decl 274) | **`274,`** | test pin |
| **c10** | `15f93d134` | `npc.rs:29211` | `270,` (s1_f_3; decl 290) | **`290,`** | test pin |
| **g10** | `5dee28650` | `npc.rs:28244` | `270,` (s1_f_3; decl 281) | **`281,`** | test pin |

g8's fix is URGENT: the pushed tip does not compile, so every g8 witness
is unrunnable until the decl lands. After g8 executes its 17 drops + the
Vampire renumber the count stays 305−17=288 → the decl/pin become **288**
(the drops remove 17 rows; the renumber changes no count). d125/c7/c10/
g10: execute the pin fix in the same commit as the owed moves/drops where
possible (c10 post-drops: 290−16=274; c7 unchanged 274; g10 unchanged 281;
d125 unchanged 272).

**c9's S2 patch is RETIRED** — `tools/_d37_c9_allkinds_fix.patch` applies
only to the dead `2042d77f1` tip. c9's own `c830aec3d` (pushed
`9252d6f31`) already carries the four snail rows + the decl/pin bumps,
extended to 279 with the gold-critter wave. The merger takes c9's array
as-pushed and drops the 5 gold dups + Shimmerfly at merge (279−6=273).

## 5. d42's SEVEN-dragonfly drop-hunks (exact, at `bac408be0`)

research/239 §1a published 11-kind ranges against the PRE-SEAL plan. The
SEAL flipped the 4 waterfowl to d42 — **execute ONLY the dragonfly lines
below** (7 kinds: Black/Blue/Green/Orange/Red/Yellow/Gold, 595-601;
keeper d33 `b08acf463`, the fully-wired aiStyle-114 superset). The
waterfowl rows (Seagull/Seagull2/Grebe/Grebe2 602/603/608/609) STAY.
Every range below was read at the tip (windows ≤100 lines,
`_d37_lag_d42win.py`); DROP = delete, EDIT = replace as stated.

### `crates/tr-app/src/npc.rs`
| Lines | Action | Content |
|---|---|---|
| 1151-1178 | DROP | the 7 doc-triples + variants (BlackDragonfly…GoldDragonfly; WindyBalloon 1147-1150 and Seagull 1179+ stay) |
| 3746-3751 | DROP | kb comment + 5 rows; 3753-3754 DROP Yellow/Gold kb rows (KEEP the 3752 comment — Seagull/Seagull2 3755-3756 ride it) |
| 4180-4185 | DROP | defense comment + 5 rows; 4187-4188 DROP Yellow/Gold (keep 4186 comment) |
| 4606-4611 | DROP | value comment + 5 rows; 4613-4614 DROP Yellow/Gold (keep 4612 comment) |
| 6769-6774 | DROP | the Yellow/Gold idle cites+arms (the 6766-6768 batch-5 comment STAYS — Seagull/Seagull2 6775-6780 ride it) |
| 6781-6799 | DROP | the whole batch-4 comment + 5 dragonfly cites+arms |
| 9791-9812 | DROP | both FindFrame arms + cites (five-colour 9791-9801, Yellow/Gold 9802-9812; the Seagull case 9813+ stays) |
| 19959-19964 | DROP | render-key comment + 5 rows; 19966-19967 DROP Yellow/Gold (keep 19965 comment) |
| 25066 | EDIT | `ALL_KINDS: [NpcKind; 313]` → **306** |
| 25360-25365 | DROP | ALL_KINDS comment + 5 rows; 25367-25368 DROP Yellow/Gold (keep 25366 comment) |
| 25600-25634 | DROP | the whole `w4_d42_b4_…` test (all five rows are dragonflies; d33's stat pin covers them) |
| 25644-25645 | DROP | b5's Yellow/Gold stat rows (KEEP the test — Seagull/Seagull2 rows stay) |
| 25664-25665 | DROP | b5's Yellow/Gold to_u16 asserts |
| 25668 | EDIT | b5's `ALL_KINDS.len(), 291` → **289** |
| 25736 | EDIT | b7's `298` → **291** |
| 25771 | EDIT | b8's `302` → **295** |
| 25807 | EDIT | b9's `307` → **300** |
| 25843 | EDIT | b10's `312` → **305** |
| 25871 | EDIT | b11's `313` → **306** |
| 29389 | EDIT | s1_f_3 `313,` → **306,** (+ the message's kind-count note) |

b6 (25671-25701, Grebe/Grebe2) UNCHANGED — the waterfowl stay.

### `crates/tr-app/src/npc/data.rs`
| Lines | Action | Content |
|---|---|---|
| 609-615 | DROP | the 7 to_u16 arms |
| 985-991 | DROP | the 7 from_u16 arms |
| 1910-1915 | DROP | HitSound comment + 5 rows; 1917-1918 DROP Yellow/Gold (keep 1916 comment — Seagull/Seagull2 ride it) |
| 2273-2277 | DROP | DeathSound 5 rows (keep 2271-2272 comment); 2278-2281 DROP comment+Yellow/Gold (keep 2284-2285 comment for Grebe) |
| 2427-2437 | DROP | both no_gravity groups + comments (5-colour 2427-2433, Yellow/Gold 2434-2437; BloodNautilus 2438+ stays) |
| 2714-2719 | DROP | ai_style comment + 5 rows; 2721-2722 DROP Yellow/Gold (keep 2720 comment) |
| 3315-3320 | DROP | size comment + 5 rows; 3322-3323 DROP Yellow/Gold (keep 3321 comment) |
| 3480-3484 | DROP | cs_id 5 rows (keep 3477-3479 comment); 3485-3489 DROP comment+Yellow/Gold (keep 3492-3494 comment for Grebe) |
| 4093-4098 | DROP | max_hp comment + 5 rows; 4100-4101 DROP Yellow/Gold (keep 4099 comment) |
| 4511-4516 | DROP | contact_damage comment + 5 rows; 4518-4519 DROP Yellow/Gold (keep 4517 comment) |

### `crates/tr-app/src/bestiary.rs`
| Lines | Action | Content |
|---|---|---|
| 2265-2284 | DROP | the 2 comments + RedDragonfly/GoldDragonfly ENTRIES rows (d33 carries all 7 incl the hidden colours; Seagull 2285+ stays) |
| 2660-2665 | DROP | the fold comment + 4 colour folds; 2666-2668 DROP comment + Yellow→Red fold (KEEP 2669 Seagull2→Seagull — d42 now owns it; re-attach it under the 2670-2671 comment or keep its own) |

### `crates/tr-app/src/npc_buffs.rs`
| Lines | Action | Content |
|---|---|---|
| 511-516 | DROP | the 5-colour immunity group + comment |
| 518-519 | DROP | Yellow/Gold alternants — EDIT 517-521: keep the comment, the group becomes `NpcKind::Seagull \| NpcKind::Seagull2 => &[31],` |

### `crates/tr-app/src/app/netglue.rs`
| Lines | Action | Content |
|---|---|---|
| 3645-3646 | DROP | Yellow/Gold alternants — EDIT 3643-3651: the hover group keeps only `crate::npc::NpcKind::Seagull2 => { … }` |
| 3652-3662 | DROP | the whole 5-colour hover seed + comment |

### `crates/tr-app/src/ui/bestiary.rs`
| Lines | Action | Content |
|---|---|---|
| 1617-1620 | DROP | comment + RedDragonfly portrait; 1621-1624 DROP comment + GoldDragonfly portrait (Seagull 1625 stays) |

### `crates/tr-app/src/render/entities.rs`
| Lines | Action | Content |
|---|---|---|
| 1248-1251 | DROP | comment + YellowDragonfly texture row (KEEP 1252 Seagull2 — re-attach under the 1253-1255 comment) |
| 1253-1259 | DROP | comment + 4 colour texture rows (the 1260+ comment stays) |
| 3418-3427 | DROP | comment + Yellow/Gold draw arms (Seagull 3428+ stays) |
| 3434-3452 | DROP | comment + 5-colour draw arms |
| 3792-3798 | DROP | the 7 dragonfly OUT-list entries (KEEP 3799-3800 Seagull/Grebe — d42 owns them); the witness doc's "33 OUT" → **26 OUT** (3779), the IN gate unchanged (10) |

**Owner-side gap (from 239 §1b, now d42's OWN obligation):** c10's
`entry_for_npc` lacks the Seagull2→Seagull / Grebe2→Grebe folds — but the
SEAL flip means **d42 keeps its own correct folds** (2669/2672) and c10
drops its waterfowl rows entirely. NO c10 fold-append is owed anymore;
239 §1b item 1 is MOOT. The GGF-walker-593 fold re-point (239 §2) stands
as published (c4's 592 IS ported).

## 6. The new-wave adjudications (this doc's verdicts, with rationale)

1. **688/692 → d43** (c4 research/148's conditional FIRED; d43
   research/244 §1 concur). c4 retires its variants AT MERGE and folds:
   the `just_hit` one-tick puff fix (d43's `npc.rs:20167` still reads
   `just_hit: self.hurt_cd > 0.0` — the stand-in c4's `17a241f6a` witness
   pins as WRONG; the merger must carry c4's `Npc.just_hit` field +
   strike-path write into d43's shape), the Orca wet-anchor, the
   catchItem 5511 row, research/144's NPC_692 lazy-page admission.
2. **666 VileSpitEoW vs VileSpitEaterOfWorlds** — d43/244 said "not a dup,
   keep both"; that is WRONG against d23's pushed `VileSpitEoW@411`
   (same C# kind, `NPCID.cs:12009`, same bestiary credit
   "VileSpitEaterOfWorlds"). ONE kind survives: **d43's full-name variant
   at cs==proto 666** (kinds-lane rule; the name matches the C#); d23
   drops its abbreviation (its style-9 machine arm keys cs 666 — the
   axis dispatch survives the kind-name dedupe; the merger re-keys
   d23's `sphere_family_step` 666 branch onto the surviving variant).
3. **634 SporeBat / 660 QueenSlimeMinionPurple → d23** at cs==proto
   (re-pin 407→634, 409→660). d23's style-14 wave WIRES both (the 634
   flap num292--, the 660 eye-group localAI[0] seed + the dead-rotation
   documentation); d43's rows are `{}` stand-ins → drop (the 244
   keep-wired-superset rule, applied symmetrically). d43's drop count
   18→20. d23's G6 render-key note (the 660→433 hash slot) stays valid —
   render keys are net-keyed, the proto renumber does not touch them.
4. **d23's style-14 dup wave** (Demon 400/IceBat 401/Lavabat 402/
   GiantFlyingFox 403/VampireBat 405/Vampire 406/RedDevil 404/GiantBat
   441/IlluminantBat 460/Slimer 458/Harpy 275/FlyingSnake 360): all 12
   are dups of the SEAL-roster kinds (g8's wired machines own the bat/
   devil family; d34 owns Harpy/Demon; d35 owns GiantBat/Slimer/
   IlluminantBat; d37 owns FlyingSnake). d23 drops the 12 duplicate
   ROWS at merge but its **style-14 axis machine + the shot/dust/
   transform seams are the keeper implementation** — the merger folds
   the surviving kinds' dispatch onto d23's `npc_ai_style14` (g8's
   per-kind machine ports and d23's axis port are the same C#
   `NPC.cs:22849-23486`; the E5 keep-one rule picks the axis form, the
   g8 witnesses re-key). g8's owed 17 moves still execute (the ids are
   the protocol contract); the machine dedupe is a merge-wave action.
5. **d34 phase-4** (281-285): PinkJellyfish/Antlion/SpikeBall are dups of
   g8's kinds (g8 wires the faithful style-18/19/20 machines; d34's
   Antlion/SpikeBall are `{}` stand-ins, its PinkJellyfish rides
   `update_jellyfish` — the same machine g8's axis arm claims). d34
   drops 3. **Crab/DungeonGuardian are UNIQUE** but 282/283 are d38's
   pushed RaggedCasterOpenCoat/Necromancer → re-pin **791/792** (free
   above d39's 823? NO — 791/792 sit in the SEAL's d36 guidance band,
   verified free at every live tip by the union gate; d36's live tip
   uses 767-771/781/782/793 only).
6. **f18** (verify-lane, 20 kinds at 237-258): every kind is a canonical
   lane's (scripted: all 20 in the plan under c7/c8/c9/c10/d39/d40/e26).
   Its ids 237-258 collide with d125's OLD 229/237 (now moved) and the
   base-free band, but the kinds themselves are dups → all 20 drop at
   merge; f18's FindFrame butterfly fix (`a832d38ea`, the ai[2]=0
   saturation) folds onto d39's Butterfly.
7. **e21** Bunny@229 dup→d34@270; **PartyBunny@237 ACCEPT** (cs 540,
   unique; 237 freed by d125's executed move; f18's Firefly@237 drops).
8. **g62** GoblinShark@620/Shimmerfly@677 dups→d42/c7 (the hit-dust
   wiring folds onto the keepers).
9. **d9** CultistDragon 6 dups→d40 (same ids 454-459); the unique
   AncientCultistSquidhead 521→**790** (521 is inside d41's claimed
   491-560 band; cs/net/texture stay 521 — the bestiary-fallthrough
   shape, the E8 convention).
10. **g8 Vampire@388** squats d37's RustyArmoredBonesAxe → **762**
    (cs 159 stays; 762 verified free after d36's executed re-pin).

## 7. Count pins (FINAL v2, supersede the SEAL S4)

| Pin | SEAL core/grand | **FINAL v2** | derivation |
|---|---|---|---|
| plan kinds | 282 / 313 | **389** | `_d37_lag_final2.py` (SEAL 282 + 108 new − 1 alias-merge: d41 31, d43 41, g10 11, d36 8, d39 7, d23 2, d34 2, g5 2, d9/e21/f42/g8 1 each; VileSpitEoW folds into d43's VileSpitEaterOfWorlds; Pufferfish/Orca flip c4→d43 without count change) |
| ALL_KINDS/to/from/s1_f_3 | 552 / 583 | **659** (270 base + 389) | scripted |
| bestiary ENTRIES | 489 / 509 | **577** (241 base numeric + 336 surviving-owner rows) | `_d37_lag_pins3.py`/`_pinsfinal.py`, per-owner dedupe incl the 3 aliases |
| owned kinds w/o entry | 34 | **53** (the fold-target segments + the d41 colour variants + SolarGoop/AncientLight/AncientDoom — the d41 lane's own entries are owed, flagged) | scripted list in `_d37_lag_pins3.json` |
| STATIC_SHEET_NAMES NPC rows | 463 / 494 | **581 union** (pre-repack; the lazy-load wave owns the resident set) | `_d37_lag_pins3.py` |
| BESTIARY_NPC_SHEET_IDS | 347/419 | **432 distinct** (the S3 gap-fill target re-derives at merge against the 389-kind roster) | scripted |
| SFX_CUES NPC cues | 103 / 109 | **117** | scripted |
| max protocol id | 766 / 787 | **823** (d39's executed R7 re-pin) | union gate |
| owed | 69 moves / 61 drops | **74 moves / 144 drops** | `_d37_lag_final2.json` |

**Merger bible (unchanged order, updated gates):** d34 → d38 → d37 →
d40 → d42 → d35-renumber-last → small lanes → the new-wave lanes
(d36/d39/d41/d43/g10/d9/g5/f42/e21 splice as-pushed; f18/f47/g62/d11
drop-only). The bijection gate (`decodable == ALL_KINDS.len()`,
i16-safe) + the s1_f_3 coverage pin must equal **659** after every lane
lands, ENTRIES **577**, cues **117**; the atlas union (581 NPC sheets)
exceeds one 8192 page — d35 research/113's lazy-load contract is
MANDATORY (its §17 + Appendix C at `8fa3b3dbf` carry the 707-name census
union the repack wave reads). The S3 gap appends ride it.

## 8. Witness register (all scripted, ALL PASS; code-only lane)

- `_d37_lag_probe.py`/`_probe2.py` — the 5-lane liveness + ALL_KINDS
  census (tip ages, dirty states, local-vs-origin ancestry).
- `_d37_lag_probe3.py`/`_probe4.py` — the post-SEAL mover clash gates
  (d39/d41/d43/d36/f18/f42/d9/d11/g5/f47 vs the SEAL grand plan).
- `_d37_lag_probe6.py`/`_probe7.py` — the LIVE-wave re-read (c9/d125/
  e26/d23/d34/d36/d41/e21/g62 new tips; d38's 281-290 occupancy; the
  stale-pin compile/test classification).
- `_d37_lag_arms.py`/`_sites2.py`/`_g8new.py` — function-scoped
  to_u16/from_u16 arm-line censuses (the broad-regex trap documented:
  a naive `NpcKind::X => N` scan hits ai_style/cs_id/render tables —
  every site here is fn-body-scoped).
- `_d37_lag_verify.py` — the SEAL receipts re-sum (69/61 at HEAD), the
  130 arm-id cross-check vs tips (NONE mismatched), d125 ancestry
  (3/3 exit 0), d42's 36/36 FINAL-id verification.
- `_d37_lag_final2.py` — the FINAL v2 plan + owed derivation: **389
  kinds / 389 distinct ids / zero base clash / max 823 — ALL GATES
  PASSED**; 74 moves / 144 drops; UNPLANNED: none.
- `_d37_lag_pins3.py`/`_pinsfinal.py` — the §7 count pins (per-owner
  ENTRIES dedupe, base 241 convention verified).
- `_d37_lag_d42df.py`/`_d42win.py` — the §5 dragonfly-only ranges
  (every window ≤100 lines, read at `bac408be0`).
- `_d37_lag_emit2.py` → `_d37_lag_tables2.md` — the per-lane master
  tables with arm-line sites at the pinned tips.
- c9 patch check: `git apply --check` **exit 0** at `2042d77f1`
  (byte-exact blob, temp dir), **exit 1** at `9252d6f31`/`c830aec3d`
  (the supersession proof).

No code was changed on this branch; this doc + the witness scripts are
the deliverable. The lanes execute their own tables on their own
branches; the merger executes the retired-lane rows (d120) + the
fold-contracts (§6) at the merge wave.
