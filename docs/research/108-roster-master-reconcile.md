# 108 — Roster master reconcile (W4 kinds wave, merger playbook)

Status: RESEARCH / MERGE PLAN — no code changes on this branch.
Author lane: W4-d37 (executor, reconcile phase), branch
`task/W4-d37-npc-kinds-211-280`, parent `750d8e44`.

All numbers below were re-derived from the **pushed tips** by scripted
extraction of `to_u16`/`from_u16`/`cs_id`/`ALL_KINDS`/`ENTRIES`/
`BESTIARY_NPC_SHEET_IDS`/`STATIC_SHEET_NAMES`/`SFX_CUES` (git-show per
tip, regex arm census; every tip's `to_u16`↔`from_u16` bijection and
`ALL_KINDS` declared-vs-actual length verified equal, zero duplicate
arms).

## 0. Tips read (as of this commit)

| Lane | Branch | Tip read | State |
|---|---|---|---|
| base | (wave-3 land) | `a725acb0` | 270 kinds, the common ancestor of every kinds branch |
| d34 | `task/W4-d34-npc-kinds-1-70` | `4012a7f3` (phase 2; supersedes `30bfb41b` phase 1) | 10 kinds pushed, band 1-70 NOT complete |
| d35 | `task/W4-d35-npc-kinds-71-140` | `0668fba7` (sheets phase; contains `eb0f1005` evidence + `d6e4f9b8` ENTRIES splice repair) | 33 kinds, wave complete + 21 sheets packed |
| d36 | `task/W4-d36-npc-kinds-141-210` | `a725acb0` | **no work pushed** (tip == base) |
| d37 | `task/W4-d37-npc-kinds-211-280` | `750d8e44` | 46 kinds, band complete |
| d38 | `task/W4-d38-npc-kinds-281-350` | `8176c078` | 35 kinds, wave complete (vanilla-free ids) |
| d39 | `task/W4-d39-npc-kinds-351-420` | `a725acb0` | **no work pushed** (tip == base) |
| d40 | `task/W4-d40-npc-kinds-421-490` | `8fe1d6d3` | 56 kinds, band complete (vanilla-free ids) |
| d41/d42 | — | **branches do not exist on origin** | bands 491-560 / 561-630 unclaimed |
| d43 | `task/W4-d43-npc-kinds-631-697` | `a725acb0` | **no work pushed** (tip == base) |

The playbook's named hashes `eb0f1005` (d35 evidence) and `d6e4f9b8`
(d35 splice repair) are ancestors of `0668fba7`; `8176c078`/`8fe1d6d3`
are still the d38/d40 tips. d34 advanced mid-reconcile (phase 2 added
BoneSerpentHead/Body/Tail, CorruptBunny, Harpy at proto 271-275) — this
document covers the **phase-2** tip.

## 1. Per-band table

Protocol band = the `to_u16` ids the branch assigns its NEW kinds.
"vanilla-free" = protocol id == the vanilla `NPCID` (skipping ids the
base already owns); "port-local" = a reassigned band (S1-F-1 precedent).

| Branch | New kinds | Protocol band | ALL_KINDS | bestiary ENTRIES | BESTIARY_NPC_SHEET_IDS (portrait slots) | STATIC NPC_ sheets |
|---|---|---|---|---|---|---|
| base `a725acb0` | — | 0-353 (270 ids, holes kept) | 270 | 241 | 181 | 248 |
| d34 `4012a7f3` | +10 | **266-275** port-local (vanilla 31/34/37/39/40/41/43/46/47/48) | 280 | 249 (+8; BoneSerpent body/tail Hide-fold onto head 39) | 191 (+10, incl. the 2 Hide segment draw sheets 40/41) | 258 (+10) |
| d35 `0668fba7` | +33 | **429-461** port-local (vanilla 73-140) | 303 | 269 (+28; 96/97/99/100/112 Hide-fold) | 197 (+16 visible) | 269 (+21 packed; 12 deferred: 79/80/84/85/86/103/104/109/110/120/122/140) |
| d37 `750d8e44` | +46 | **354-399** port-local (vanilla 217-280) | 316 | 284 (+43; 230/236/239 Hide-fold) | 224 (+43) | 294 (+46, incl. the 3 Hide folds' own sheets) |
| d38 `8176c078` | +35 | **281-337 vanilla-free** (skips 288/305-315/319-321/325-327/329-332 base-owned) | 305 | 276 (+35, all own) | 181 (**+0 — gap**, see §5) | 283 (+35) |
| d40 `8fe1d6d3` | +56 | **430-488 vanilla-free** (skips 439/441/471 base-owned; 421-429 base vortex family) | 326 | 279 (+38; 18 folds incl. TargetDummy 488 no-entry) | 181 (**+0 — gap**, see §5) | 248 (**+0** — all 56 sheets deferred to atlas-repack) |
| **UNION** | **+180** | see §3 | **450** | **393** | **250** as pushed (323 after the §5 gap close) | **360** as pushed |

Cross-checks run on the union: zero kind-name duplicates across
branches; zero bestiary `net_id`→multi-kind collisions (393 distinct
positive-net rows); every branch's `to_u16`/`from_u16` are exact
bijections over its own `ALL_KINDS` (270/280/303/316/305/326).

## 2. COLLISION matrix

Two numbering spaces exist and must not be conflated:
- **protocol space** (`to_u16`/`from_u16`) — the wire/save id. Collisions
  here are hard bugs (the S1-F-1 pirate/Deerclops class).
- **vanilla cs space** (`cs_id`/bestiary `net_id`) — the C# `NPCID`.
  Overlap with a protocol band is only a *constraint on future
  numbering*, never a wire bug by itself.

### 2a. Protocol-space collisions (wire bugs if merged naively)

Pairwise scan of all five branches' new-kind id maps:

| Pair | Colliding ids | Verdict |
|---|---|---|
| d34 × any | 0 | clean |
| d37 × any | 0 | clean — 354-399 is free on every other tip |
| d38 × any | 0 | clean — vanilla-free skips every base-owned id |
| d40 × d35 | **30 ids: 430-438, 440, 442-461** | **HARD COLLISION** |
| any × base | 0 | every branch verified against base's 270-id map |

The d35×d40 collision, id by id (d35 kind ↔ d40 kind):

```
430 Bird↔ArmedZombie            431 Pixie↔ArmedZombieEskimo
432 ArmoredSkeleton↔ArmedZombiePincussion  433 Mummy↔ArmedZombieSlimed
434 DarkMummy↔ArmedZombieSwamp  435 LightMummy↔ArmedZombieTwiggy
436 Wraith↔ArmedZombieCenx      437 CursedHammer↔CultistTablet
438 EnchantedSword↔CultistDevote 440 Unicorn↔CultistBossClone
442 Corruptor↔GoldBird          443 DiggerHead↔GoldBunny
444 DiggerBody↔GoldButterfly    445 DiggerTail↔GoldFrog
446 SeekerHead↔GoldGrasshopper  447 SeekerBody↔GoldMouse
448 SeekerTail↔GoldWorm         449 Clinger↔BoneThrowingSkeleton
450 AnglerFish↔BoneThrowingSkeleton2  451 GreenJellyfish↔BoneThrowingSkeleton3
452 Werewolf↔BoneThrowingSkeleton4    453 Clown↔SkeletonMerchant
454 SkeletonArcher↔CultistDragonHead  455 VileSpit↔CultistDragonBody1
456 TheHungryII↔CultistDragonBody2    457 ChaosElemental↔CultistDragonBody3
458 Slimer↔CultistDragonBody4         459 Gastropod↔CultistDragonTail
460 IlluminantBat↔Butcher             461 PossessedArmor↔CreatureFromTheDeep
```

d35's 429 (GoblinScout), 439 (Mimic), 441 (GiantBat) do NOT collide —
d40 skips 439/441 (base AncientCultist/TaxCollector cs ids) and 429
(base VortexSoldier cs id, proto 120). Both branches independently
picked "next free above the landed space" and landed on the same
429+/430+ corridor: d35 counted from 353 (BlazingWheel) up past the
then-free 400s; d40 kept protocol==vanilla. Neither saw the other.

### 2b. Vanilla ids sitting inside port-local bands (cs-space overlap)

These are the vanilla `NPCID`s whose *protocol* ids live elsewhere —
the reason each band was reassigned, and the constraint list for the
unpushed d36/d39/d41/d42/d43 lanes:

| Port-local band | Vanilla ids inside it | Owner of those vanilla types (kind → proto) |
|---|---|---|
| d34 266-275 | 266 BrainOfCthulhu, 267 Creeper (base); 268-275 = IchorSticker + the 7 rusty/blue armored bones | base BoC/Creeper; **d37** ports 268-275 at proto 387-394 |
| d35 429-461 | 429 VortexSoldier→120, 439 AncientCultist→112, 441 TaxCollector→162 (base); 430-438/440/442-461 = the armed-zombie/cultist/gold-critter/bone-thrower families | **d40** ports all of them at protocol==vanilla → §2a |
| d37 354-399 | 354 WebbedStylist→161, 368 TravellingMerchant→111, 369 Angler→150, 370 DukeFishron→219, 372 Sharkron→221, 376 SleepingAngler→151, 384 ForceBubble→213 (base); 355-367/371/373-375/377-380 etc. unported | **d39 (unpushed)** owns vanilla 351-420: 21 of its ~24 missing types fall inside 354-399 → d39 CANNOT use vanilla-free ids |
| d38 281-337 | protocol==vanilla; only 288 DungeonSpirit is base-owned *in the same space* (proto 288 == cs 288) and was skipped | none — clean by construction |
| d40 430-488 | protocol==vanilla; 439/441/471 skipped (base cs owners at proto 112/162/61) | d35's port-local band (§2a) is the only clash |

Also note the reverse direction: d37's own **vanilla** band 217-280
collides with base *protocol* ids 216-228 (destroyer/fishron/wyvern),
230-236 (zombies), 250 (nimbus) — the documented reason d37 rode
354-399. d35's vanilla band 73-140 collides with 68 base protocol ids
in 73-140 (golem/martian/DD2 families) — the reason d35 rode 429+.

### 2c. Future-lane pressure (unpushed bands)

Uncovered vanilla types per unpushed band (cs_id ∪ bestiary-net census
over the union): d36 141-210: **50**; d39 351-420: **24**; d41 491-560:
**49**; d42 561-630: **51**; d43 631-697: **64**. ~238 more kinds →
grand union ≈ **688 kinds** (≤ NPCID.Count 697; the 8 slime colours stay
`SlimeVariant`, not kinds).

- d36 (141-210): base protocol ids occupy that corridor densely → must
  take a port-local band.
- d39 (351-420): 21 uncovered types inside d37's 354-399, 3 inside
  400-428 (403/404/408) → must take a port-local band.
- d41/d42/d43 (491-697): **protocol space ≥489 is entirely free** on the
  union after the §3 plan (base max proto 353; d40 max 488; d35 moved
  out) → all three can ride protocol==vanilla, d38/d40-style, zero
  renumber.

## 3. FINAL union numbering plan (single consistent assignment)

Principle: **renumber exactly one branch, keep four untouched.** Every
existing save/witness pin on base/d34/d37/d38/d40 stays valid.

| Block | Kinds | Final protocol ids | Change |
|---|---|---|---|
| base roster | 270 | 0-353 as landed (holes 229/237-249/251-265/276-280/319-321/331-332 stay reserved — they are cs-owned, never reassign) | none |
| d34 | 10 | 266-275 | none |
| d38 | 35 | 281-337 vanilla-free (as pushed) | none |
| d37 | 46 | 354-399 | none |
| d40 | 56 | 430-488 vanilla-free (as pushed) | none |
| **d35** | **33** | **698-730** (contiguous, order-preserving: GoblinScout 429→698, Bird 430→699, … PossessedArmor 461→730) | **RENUMBER: +269 offset on all 33 `to_u16` arms + 33 `from_u16` arms + the band's witness pins/comments** |
| d36 (future) | ~50 | 731-780 port-local | guidance |
| d39 (future) | ~24 | 781-804 port-local | guidance |
| d41/d42/d43 (future) | ~164 | 491-697 vanilla-free | guidance |

Why move d35 and not d40:
1. d35's band is *already* port-local — no invariant is broken by
   moving it again. d40's protocol==vanilla invariant is documented
   across 12 batch commits and its witnesses; moving 30 of its 56
   kinds would split its band and break the invariant for half the
   wave.
2. 698+ is above `NPCID.Count` (697): **no future vanilla-matching
   branch can ever collide with it**, and it cannot be confused with a
   cs id. The 400-428 corridor (29 slots) is too small for 33 kinds and
   overlaps d39's future vanilla range anyway.
3. Save-compat: d35's 429-461 ids exist only on its unmerged branch —
   no landed save ever wrote them, so the renumber is free (the S1-F-1
   gate rule: only *landed* ids are frozen).

i16 safety: the `23` body's kind field is i16 (`net.rs:612`, written
`to_u16() as i16`). Final max id 730; grand-union max ≈804 ≪ 32767.
`the_protocol_id_space_round_trips_every_kind` already asserts
`id <= i16::MAX` per kind — it keeps guarding this.

Rejected alternatives (recorded): (a) move d40's 30 colliding kinds to
489-518 — breaks the vanilla-free invariant, splits the band, and
squat on d41's future vanilla corridor; (b) move only d35's 30
colliding kinds, keeping 429/439/441 — splits d35's band for a 3-arm
saving; (c) interleave into the base holes — the holes are
non-contiguous and cs-reserved (the pirate-band comment's rule).

## 4. Exhaustive-match union checklist (every site × all 180 new kinds)

Hard sites (no wildcard — the compiler forces an arm per new kind;
`to_u16` additionally carries `#[deny(unreachable_patterns)]`, so a
duplicate arm is a hard error — the S1-F-1 lesson):

- [ ] `npc/data.rs` `to_u16` — 450 arms, d35 block renumbered to 698-730
- [ ] `npc/data.rs` `from_u16` — 450 arms, same renumber
- [ ] `npc/data.rs` `ai_style`, `size`, `max_hp`, `contact_damage`
- [ ] `npc.rs` `defense`, `value`, `render_key` (injective!), `update_kind_ai`
- [ ] `npc_buffs.rs` `debuff_immune_ids` + the `Default`-style bank match
- [ ] `app/netglue.rs` `npc_seed`, `can_relay`
- [ ] `ui/bestiary.rs` `portrait_data`
- [ ] `npc.rs` tests `ALL_KINDS` — 450 entries, declaration order

Wildcard sites (compile silently with a fallback — MUST be walked by
hand/script for each of the 180 kinds, the d37 scripted-scan precedent):

- [ ] `npc/data.rs` `cs_id` (hand rows; segments may keep the sentinel),
      `hit_sound`, `death_sound`, `no_gravity`, `ai3_walk_speed`,
      `is_town_resident` (d37 Pirate; d34 OldMan/Bunny; d38 critters no)
- [ ] `npc.rs` `knock_back_resist`, `find_frame` (d35/d37/d38 ported
      verbatim cases; d40 holds row 0 for the UNWIRED styles 87-92)
- [ ] `bestiary.rs` `entry_for_npc`/`try_entry_for_npc` (the Hide folds:
      d34 40/41→39; d35 96/97/99/100→95/98, 112→110; d37 230/236/239;
      d40 the 7 armed zombies fold onto their visible zombie twin, the
      4 dragon bodies + tail→head, MothronEgg→Mothron, the 5
      BoneThrowers→their skeleton twin, TargetDummy no entry)
- [ ] `render/entities.rs` `draw_npcs` arms + `texture_net_id` folds +
      the `npc_sheets` chain: `D35_SHEETS` + `D37_SHEETS` +
      `D38_B1..B7_SHEETS` must ALL be chained (each branch edited the
      same `.chain(...)` site — a merge that keeps one side silently
      unpakcs the other's sheets)
- [ ] `assets.rs` `STATIC_SHEET_NAMES` (union 360 NPC sheets),
      `BESTIARY_NPC_SHEET_IDS` + the parallel `bestiary_npc_sheets`
      vec (order-witnessed pairs)
- [ ] `audio.rs` `SFX_CUES` (see §6 dedupe)
- [ ] `ambient_sound.rs` `fighter_walker_kind`/`fighter_idle_counter`
- [ ] `world_sim/npcs.rs` worm families: d34 BoneSerpent (surface gate
      + segment-count second draw), d35 Digger/Seeker, d40 CultistDragon
- [ ] `main.rs` mod declarations: d38's `npc_ai_dungeon_shooter`,
      `npc_ai_critter`, `npc_ai_ghost` (only d38 touches main.rs)
- [ ] `npc_ai.rs` hover presets (d37 HORNET_FATTY/HORNET_VARIANT/PARROT)

## 5. Portrait-slot gap (d38/d40) — merge-wave work item

d38 added 35 bestiary entries and packed 35 draw sheets but did NOT
extend `BESTIARY_NPC_SHEET_IDS` (still 181); d40 added 38 entries with
no sheets and no slots. `portrait_data` derives
`sheet_i = bestiary_sheet_index(net_id).unwrap_or(0)` — until the slots
land, those 73 entries render the **slot-0 Blue Slime portrait** in the
bestiary (draw arms in-world are fine for d38; d40's world draw is
sheet-deferred). The merge wave should append the 35+38 slots + vec
rows in ENTRIES order: union pin 250 → **323**. (Base already carries
77 entry-less-slot deferrals — pirates etc. — and d35 has 12 deferred
sheets; those stay documented deferrals, not new gaps.)

Atlas warning (from d35's sheets-phase commit, re-verified): the
17-sibling-branch pushed sheet union **already overflows the 8192
strict-GPU bound at 8530 px (+338)** before d35's own 21; d35 packs at
8188/8188 on ITS base only. No merge order fixes this — the
**atlas-repack wave owns the cumulative reconciliation** and must run
after the roster merges (d40's 56 sheets + d35's 12 deferred ride it).

## 6. Count-pin FINAL values (post-merge, post-renumber)

| Pin | Final value | Derivation |
|---|---|---|
| `ALL_KINDS` len | **450** | 270+10+33+46+35+56 |
| `to_u16`/`from_u16` arms | **450** each | bijection, 0 dups (deny-unreachable guards) |
| `s1_f_3` coverage pin (`covered + sentinel`) | **450** | the wire-roster walk |
| bestiary `ENTRIES` rows | **393** | 241+8+28+43+35+38; all net ids distinct |
| `BESTIARY_NPC_SHEET_IDS` | **250** as-pushed union; **323** after the §5 gap close | 181+10+16+43 (+35+38) |
| `STATIC_SHEET_NAMES` NPC rows | **360** | 248+10+21+46+35 |
| `SFX_CUES` NPC hit/death cues | **98 unique** | base 55 + d34 5 + d35 6 + d37 17 + d40 18, minus cross-branch dups `NPC_Killed_28` (d34∩d35), `NPC_Hit_5`/`NPC_Killed_7` (d35∩d37). A naive append yields 104 rows; the table tolerates dups (`iter().any`) but dedupe to 98. Base itself carries 3 pre-existing dup rows (NPC_Hit_1/NPC_Hit_43/NPC_Killed_45) — leave or dedupe, pin the choice. |
| max protocol id | **730** (grand union ≈804) | i16-safe, round-trip test guards |

## 7. Merge ORDER recommendation

**d34 → d38 → d37 → d40 → d35 (last, with the 698-730 renumber).**

Rationale:
1. The first four are pairwise collision-free (§2a) and each merges
   with zero renumber — only the shared append sites (ALL_KINDS,
   to/from_u16, ENTRIES, sheet chains, SFX_CUES, portrait_data,
   netglue seeds) need splice care. Ascending vanilla-band order keeps
   the append-site diffs monotonic and reviewable.
2. d35 goes LAST because it is the only branch that must renumber:
   doing it against the final 450-kind roster means the renumber is
   one commit on one branch, verified once by the round-trip +
   s1_f_3 pins — never a double renumber if the order shifts.
3. d35's `ENTRIES` block must be taken WHOLESALE from `0668fba7`: the
   data wave's batch splices corrupted the shared ENTRIES tail and
   `d6e4f9b8` repaired it (277 heads = 277 closes at that point). A
   hunk-level merge that re-applies the pre-repair batches re-orphanes
   the six name tails (Mummy/Enchanted Sword/Digger/World Feeder/
   Clown/Slimer).
4. d38's three new `npc_ai_*` modules + main.rs rows merge cleanly at
   any position (no other branch touches main.rs).
5. After all five: run the §5 slot extension + the atlas-repack wave,
   then re-pin §6. Future lanes follow §3's guidance rows (d36/d39
   port-local 731+/781+; d41/d42/d43 vanilla-free 491-697).

No code was changed on this branch; this document is the deliverable.
Witness: extraction scripts ran against the exact tips in §0 (git-show,
no working-tree reads); all counts are reproducible from those tips.
