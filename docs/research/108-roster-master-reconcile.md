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

---

# EXTEND — the full-wave re-census (supersedes §§0-7 where they conflict)

Status: RESEARCH / MERGE PLAN EXTENSION — still no code changes on this
branch. The 6-tip read above (§0) missed **every kind-claimant outside the
five d3x kinds lanes**. A scripted sweep of **all 354 `task/W4-*` refs**
(`git diff --name-only <merge-base> <tip> -- npc/data.rs npc.rs`, then
per-tip `to_u16`/`from_u16`/`cs_id`/`ALL_KINDS`/`ENTRIES`/
`BESTIARY_NPC_SHEET_IDS`/`STATIC_SHEET_NAMES`/`SFX_CUES` extraction) found
**18 pushed kind-claimants** plus **4 local-only (unpushed) kind-claimants**
(d33/g10 with uncommitted kinds, d39 with a squatting WIP, d125 with an
unpushed sheets tip) and several claim-only/wiring-only lanes (E0/E6).
The §2a "ONE hard collision" verdict is **wrong at wave scope**: there are
**77 contested protocol ids involving 184 kind-ports**, and the §3
"renumber ONLY d35" plan no longer clears the matrix.

## E0. Tips read (the extend census, all merge-base `a725acb0`)

Pushed kind-claimants (tip → new kinds, protocol band as pushed):

| Lane | Tip read | New kinds | Pushed protocol band | Notes |
|---|---|---|---|---|
| c3 | `9ea0ee90` | +3 | 289 / 355 / 694 | GiantCursedSkull 289, CursedSkull 355 (cs 34), WaterBoltMimic 694 |
| c4 | `22cec16f` | +13 | 229-692 scattered | FlyingFish 354 (cs 224); CorruptGoldfish 229 (cs 57), Arapaima 237 (cs 157), BloodFeeder 241; the rest vanilla-free |
| c6 | `5de68021` | +1 | 328 | PumpkingBlade (dup of d38) |
| c7 | `c2cc6ec6` | +4 | 355 / 358 / 654 / 677 | Firefly 355, LightningBug 358; Lavafly/Shimmerfly vanilla-free |
| c8 | `3705670d` | +9 | 357-606 scattered | Worm 357, TruffleWorm 374, TruffleWormDigger 375; GoldWorm/crawlers dup d40; Maggot 606 |
| c9 | `2042d77f` | +4 | 359 / 360 / 655 / 677 | Snail 359, GlowingSnail 360; MagmaSnail 655; Shimmerfly 677 dup c7. **ALL_KINDS decl 270 but 4 arms — stale pin** |
| c10 | `c48bbb1e` | +8 | 362-365 / 602-609 | Duck family 362-365; Seagull/Grebe 602-609 (dup d42) |
| d23 | `19109424` | +6 | 259 / 260 / 354-357 | FungiBulb 259, GiantFungiBulb 260; ManEater 354, Snatcher 355, Clinger 356, AngryTrapper 357 |
| d120 | `4b17a2cf` | +5 | 241-245 | SantaClaus/Truffle/Steampunker/Cyborg 241-244; BartenderUnconscious 245 (dup d42) |
| d125 | `d8047a10` | +2 | 229 / 237 | EmpressOfLight 229 (cs 636), PrismaticLacewing 237 (cs 661). **ALL_KINDS decl 270 but 272 arms — stale pin** |
| d34 | `5f3435a8` | +15 | 266-280 | phase 3 added Snatcher/CorruptGoldfish/Vulture/Demon/BlueJellyfish 276-280 (supersedes §0's +10) |
| d35 | `6a2a155a` | +33 | 429-461 | unchanged ids; atlas-repack doc phases on top |
| d37 | `08f7d25c` | +46 | 354-399 | this lane, unchanged |
| d38 | `6f9dd02d` | +35 | 281-337 | **portrait gap CLOSED** (slots 181→216, crossnote phase B) — §5's d38 half is done |
| d40 | `8fe1d6d3` | +56 | 430-488 | unchanged |
| d42 | `9b22c091` | +51 | **354-404** | band 561-630 ported PORT-LOCAL 354-404 (cs 578-630). **Squats d37's whole 354-399 corridor + 400-404** |
| e26 | `048cca6c` | +4 | 361 / 362 / 604 / 605 | Frog 361, Duck 362 (dup c10); LadyBug/GoldLadyBug 604-605 (dup d42) |
| g8 | `82c93cec` | +34 | **354-387** | the ≤230 machine-wiring lane ports 34 kinds port-local 354-387. **ALL_KINDS decl 270 but 304 arms — stale pin** |

Local-only (unpushed) claimants — probed read-only via `git -C <worktree>`,
NO file recursion (see E6):

| Lane | Worktree HEAD | Local state | Kinds / band |
|---|---|---|---|
| d33 | `dcb59b7f` (pushed) | **uncommitted** npc.rs+data.rs | +5 dragonflies 595-599 (vanilla-free, cs==proto) |
| d36 | `65e481f9` (pushed CLAIM) | clean | claim-only: port-local 731-780, skip c4b 157 / d23b 175 |
| d39 | `8e4b87c9` (pushed census) | **uncommitted WIP** 7 files | concurrent writer squats 355-359 (Firefly/Butterfly/Worm/LightningBug/Snail) — violates 108 + c7/c8/c9; census commit owns 10 unclaimed 351-420 → port-local 781-790 |
| d41 | `4ed22f0b` (pushed claim) | clean | claim-only: band 491-560, skip 521/541-545/547 |
| d43 | `a725acb0` (base) | clean | no work |
| g10 | `a725acb0` (base) | **uncommitted** 4 files | +10 town pets 637/638/656/670/678-683 (vanilla-free, cs==proto) |
| c10 | `c48bbb1e` (pushed) | **uncommitted** npc.rs+projectile + 2 untracked modules | style65/critter wiring (no new kinds) |
| d125 | `d8047a10` (pushed) | **uncommitted local tip `2dfe026d2`** sheets phase + empress_draw.rs | NPC_636/661 sheets, slots 181→183 (NOT pushed) |

f26 (`63417edd`, forced-update) / f27 (`0b8e225d`) are FindFrame-only —
**no new kinds**, but they own the `find_frame` seams for 463-509 and carry
keep-one verdicts against d40's inline arms (E5).

## E1. The corrected collision matrix (protocol space)

Pairwise scan of all 18 pushed claimants' new-kind id maps (base's 270-id
map subtracted; every lane verified `to_u16`↔`from_u16` bijective, zero
duplicate arms within a lane). **77 distinct protocol ids are contested by
2+ lanes; 184 kind-ports sit on them.** The hard (different-kind) pairs:

| Pair | Contested ids | Verdict |
|---|---|---|
| **d37 × d42** | **46: 354-399 (whole corridor)** | **HARD — d42 squats d37's entire band** |
| **d37 × g8** | **34: 354-387** | **HARD — g8 squats 354-387** |
| d37 × c10 | 4: 362-365 | HARD (Duck family ↔ GoldfishWalker/Hornets) |
| d37 × c8 | 3: 357/374/375 | HARD |
| d37 × c7 | 2: 355/358 | HARD |
| d37 × c9 | 2: 359/360 | HARD |
| d37 × e26 | 2: 361/362 | HARD |
| d37 × c3/c4/d23 | 1 each: 355/354/354-357 | HARD (d23 = 4 ids 354-357) |
| **d35 × d40** | **30: 430-461** | **HARD — the §2a collision, still present** |
| d42 × g8 | 34: 354-387 | HARD (both squat the same corridor) |
| d42 × c10/c7/c8/c9/e26/c3/c4/d23 | 1-4 each | HARD (all squat 354-404) |
| g8 × c10/c7/c8/c9/e26/c3/c4/d23 | 1-4 each | HARD (all squat 354-387) |
| c4 × d125 | 2: 229/237 | HARD (CorruptGoldfish↔Empress, Arapaima↔Lacewing) |
| c4 × d120 | 1: 241 | HARD (BloodFeeder↔SantaClaus) |
| c8 × d35 | 1: 448 | HARD (GoldWorm↔SeekerTail) |

Same-kind duplicate ports (49 ports across 48 kind-names — the same vanilla
type ported by 2+ lanes; these are NOT wire collisions but **merge dedupes**,
one canonical owner kept, the rest dropped): CursedSkull (c3/d34/g8),
ManEater+Snatcher (d23/d34), Clinger (d23/d35), FungiBulb+GiantFungiBulb
(d23/d37), FlyingFish (c4/d37), BloodFeeder (c4/d37), Squid+FlyingSnake
(d37/g8), the 17 g8↔d35 kinds (Pixie/Wraith/CursedHammer/EnchantedSword/
Mimic/GreenJellyfish/Gastropod/GiantBat/Slimer/IlluminantBat/…), the 5
c8↔d40 crawlers, the c10↔d42 waterfowl, the c4↔d42 fish, the e26↔d42
ladybugs, Duck (c10/e26), Shimmerfly (c7/c9), GiantCursedSkull (c3/d38),
PumpkingBlade (c6/d38), BartenderUnconscious (d120/d42), CorruptGoldfish
(c4/d34), BlueJellyfish/Vulture/Demon/Harpy (d34/g8), CrimsonGoldfish
(c4/d40). Full keep/drop table in E3.

**The §2a "d37 × any = 0, clean" row is false**: d37's 354-399 is the
single most-squatted corridor in the wave (d42 all 46, g8 34, plus 7 small
lanes). The 6-tip read never saw d42/g8/c3/c4/c7/c8/c9/c10/d23/d120/d125/e26.

## E2. Adjudication — d37 band-shift vs small-lanes-shift

Three candidate plans were computed and collision-checked (scripted, zero
dups, zero base-clash in all three). Renumber cost = owned kinds whose
final id ≠ pushed id:

| Plan | Kinds moved | Max id | d37 corridor | Verdict |
|---|---|---|---|---|
| (a) naive minimal (keep largest lane per id) | 107 | 804 | **broken** (46 moved, interleaved) | rejected — shatters d37 AND d42, non-contiguous |
| (b) principled 698+ (all port-local → 698+) | 184 | 881 | broken (46 moved) | rejected — moves d37/d42 wholesale, max id 881 |
| **(c) CANONICAL: keep d37/d38/d40/d34, d35→698-730, d42→vanilla-free, small lanes→698+** | **110** | **761** | **INTACT (354-399)** | **CHOSEN** |

**Why (c) — the 698+ principle applied to the REAL matrix:**

1. **d37 keeps 354-399.** This lane's band is the named anchor of the
   reconcile, it is contiguous, bijective, and already witness-pinned
   (`750d8e44`). Moving it (plans a/b) costs 46 renumbers AND breaks the
   one corridor the merger can verify against a landed tip. The task's
   "d37 band-shift vs small-lanes-shift" resolves to **small-lanes-shift**:
   d42 and g8 are the squatters, not d37.
2. **d42 returns to vanilla-free 578-630.** d42's pushed 354-404 is
   port-local, but its cs ids 578-630 are **entirely free in base protocol
   space** (base max 353; 491-697 unoccupied). So d42's 51 kinds move
   354-404 → 578-630 (protocol==vanilla, the d38/d40 invariant) — this
   clears the whole d37×d42 collision AND restores d42's own invariant.
   d42's WIP commit message already says the 354-363 numbering is
   "SUPERSEDED … by the 108 union plan's vanilla-free band" — this is that
   band, extended to the full 51.
3. **d35 → 698-730** (the §3 decision, unchanged): clears d35×d40's 30 ids
   and the g8↔d35 duplicates' corridor. d35 is already port-local, so the
   move breaks no invariant; 698+ is above NPCID.Count 697, future-proof.
4. **g8's 17 owned kinds → 745-761.** g8 ports 34 kinds but 17 are
   duplicates of d34/d35/d37 (dropped, E3); its 17 unique kinds (the
   164-226 machine-wiring set: WallCreeper/IceElemental/Herpling/Derpling/
   CrimsonAxe/FloatyGross/LostGirl/Nymph/IceBat/Lavabat/GiantFlyingFox/
   VampireBat/RedDevil/PinkJellyfish/Antlion/SpikeBall) squat 354-387 and
   move to 745-761.
5. **The small critter/town lanes shift only their contested kinds** to
   731-744: c10 Duck family 362-365→731-734, c7 Firefly/LightningBug
   355/358→735-736, c8 Worm/TruffleWorm/TruffleWormDigger 357/374/375→
   737-739, c9 Snail/GlowingSnail 359/360→740-741, d125 PrismaticLacewing
   237→742, d23 AngryTrapper 357→743, e26 Frog 361→744. Their vanilla-free
   kinds (Lavafly 654, MagmaSnail 655, Shimmerfly 677, Seagull/Grebe
   602-609→owned by d42, etc.) keep protocol==cs.
6. **d34/d38/d40 keep pushed** (266-280 / 281-337 / 430-488): pairwise
   clean once d35 moves out and d42 returns to 578-630.

Total: **110 owned kinds renumber, 170 keep pushed, 49 duplicate ports
dropped.** Final max id **761** (i16-safe; grand union with the unpushed
d33/g10 vanilla-free pets+dragonflies stays ≤697, d36/d39 port-local
731-790 overlaps the small-lane band — see E4 guidance).

Rejected: moving d37 (the anchor) to satisfy squatters that have NOT landed
a witness-pinned corridor; the naive per-id greedy (shatters two bands into
non-contiguous interleaves, harder to review than 5 clean block moves).

## E3. Canonical final numbering + dedupe table

Per-lane final id ranges (owned kinds only; duplicates dropped):

| Lane | Owned kinds | Final ids | Changed |
|---|---|---|---|
| d34 | 15 | 266-280 | 0 |
| d38 | 35 | 281-337 | 0 |
| d37 | 46 | 354-399 | 0 |
| d40 | 56 | 430-488 | 0 |
| d42 | 51 | **578-630** (vanilla-free) | 51 |
| d35 | 33 | **698-730** | 33 |
| c10 | 4 | 731-734 | 4 |
| c7 | 4 | 654/677 + 735-736 | 2 |
| c8 | 3 | 737-739 | 3 |
| c9 | 3 | 655 + 740-741 | 2 |
| d125 | 2 | 229 + 742 | 1 |
| d23 | 1 | 743 | 1 |
| e26 | 1 | 744 | 1 |
| g8 | 17 | 745-761 | 17 |
| c3 | 1 | 694 | 0 |
| c4 | 4 | 237/241/509/688/692 (vanilla-free) | 0 |
| d120 | 4 | 241-244 | 0 |
| c6 | 0 | (PumpkingBlade deduped to d38) | — |

**Duplicate ports to DROP at merge** (keep canonical owner@final id):

```
BartenderUnconscious  keep d42@579   drop d120@245
BloodFeeder           keep d37@373   drop c4@241
BlueJellyfish         keep d34@280   drop g8@355
Buggy                 keep d40@487   drop c8@487
Clinger               keep d35@718   drop d23@356
CorruptGoldfish       keep d34@277   drop c4@229
CrimsonGoldfish       keep d40@465   drop c4@465
CursedHammer          keep d35@706   drop g8@361
CursedSkull           keep d34@267   drop c3@355, g8@375
Demon                 keep d34@279   drop g8@376
Dolphin               keep d42@615   drop c4@615
Duck                  keep c10@731   drop e26@362
EnchantedNightcrawler keep d40@484   drop c8@484
EnchantedSword        keep d35@707   drop g8@362
EyeballFlyingFish     keep d42@587   drop c4@587
FlyingAntlion         keep d42@581   drop c4@581
FlyingFish            keep d37@359   drop c4@354
FlyingSnake           keep d37@360   drop g8@384
FungiBulb             keep d37@385   drop d23@259
Gastropod             keep d35@728   drop g8@365
GiantBat              keep d35@710   drop g8@377
GiantCursedSkull      keep d38@289   drop c3@289
GiantFungiBulb        keep d37@386   drop d23@260
GoldGoldfish          keep d42@592   drop c4@592
GoldLadyBug           keep d42@605   drop e26@605
GoldWorm              keep d40@448   drop c8@448
Grebe / Grebe2        keep d42@608/609  drop c10@608/609
GreenJellyfish        keep d35@720   drop g8@364
Grubby                keep d40@485   drop c8@485
Harpy                 keep d34@275   drop g8@386
IlluminantBat         keep d35@729   drop g8@379
LadyBug               keep d42@604   drop e26@604
Maggot                keep d42@606   drop c8@606
ManEater              keep d34@269   drop d23@354
Mimic                 keep d35@708   drop g8@363
Pixie                 keep d35@700   drop g8@359
PumpkingBlade         keep d38@328   drop c6@328
Pupfish               keep d42@607   drop c4@607
Seagull / Seagull2    keep d42@602/603  drop c10@602/603
Shimmerfly            keep c7@677    drop c9@677
Slimer                keep d35@727   drop g8@378
Sluggy                keep d40@486   drop c8@486
Snatcher              keep d34@276   drop d23@355
Squid                 keep d37@358   drop g8@385
Vulture               keep d34@278   drop g8@354
Wraith                keep d35@705   drop g8@360
```

Ownership rule: a kinds-lane (d3x/d4x) that ports the kind owns it over a
wiring lane (c*/g8/e26/d120/d125); among kinds-lanes the lower band number
owns (d34<d35<d37<d38<d40<d42). The dropped lane keeps its AI/draw/sound
wiring keyed on the canonical kind — only the duplicate `NpcKind` variant +
`to_u16`/`from_u16`/`ALL_KINDS`/`ENTRIES` row is removed.

## E4. Count-pins (post-merge, post-renumber, deduped) — SUPERSEDES §6

| Pin | Final value | Derivation |
|---|---|---|
| `ALL_KINDS` len | **550** | 270 base + 280 distinct new kinds (329 ports − 49 dupes) |
| `to_u16`/`from_u16` arms | **550** each | bijection, 0 dups (deny-unreachable guards) |
| `s1_f_3` coverage pin | **550** | covered + sentinel walk |
| bestiary `ENTRIES` rows | **489** | 249 base + 240 new credits (287 new rows − 47 dup-credit rows; all net ids distinct) |
| `BESTIARY_NPC_SHEET_IDS` | **≤357** upper bound | 181 + 176 naive per-lane slot sum; dedupe to the 240 visible-entry kinds at merge (d38's +35 already landed; d40's +38 still a gap, §5) |
| `STATIC_SHEET_NAMES` NPC rows | **446** | 270 base + 176 packed-sheet-name union |
| `SFX_CUES` NPC hit/death cues | **103 unique** | 55 base + 48 new (naive append 151 rows; dedupe to 103) |
| max protocol id | **761** | i16-safe; round-trip test guards |

The §6 pins (450/393/323/360/98) were computed from the 6-tip read and are
**superseded**. The 40 union kinds WITHOUT a new bestiary entry (Hide folds,
segments, no-entry fixtures): the 7 ArmedZombies, 5 BoneThrowers, 5
CultistDragon bodies+tail, Digger/Seeker/BoneSerpent body+tail (6), VileSpit,
MothronEgg, TargetDummy, GoldfishWalker, GoldGoldfishWalker, JungleCreeper,
BloodCrawler, BartenderUnconscious, the 4 dragonflies (d33 local), Seagull2,
Grebe2, ArmedTorchZombie — these fold onto a visible twin or keep no entry.

**Stale ALL_KINDS declarations to fix at merge** (decl ≠ actual arms):
c9 `270`→274, d125 `270`→272, g8 `270`→304, d42 WIP `270`→321 (the
`9b22c091` tip already declares 321 correctly). These are compile-time
`[NpcKind; N]` length mismatches — the merge must reconcile each to the
550 union, not trust the pushed decl.

Future-lane guidance (unpushed): d33 dragonflies 595-599 + g10 town pets
637/638/656/670/678-683 are **vanilla-free, keep cs==proto** (no clash with
the canonical plan — all ≤697 and free). d36 port-local 731-780 and d39
port-local 781-790 **overlap the small-lane 731-761 band** — re-pin d36 to
**762-811** and d39 to **812-821** (above the canonical max 761) when they
land; d41/d43 vanilla-free 491-560 / 631-697 (skip the g10/d33 cs ids).

## E5. Exhaustive-match + wildcard union checklist additions

§4's checklist stands, plus the new claimants' sites:

- [ ] `to_u16`/`from_u16` — 550 arms; d35 block 698-730, d42 block 578-630,
      g8 block 745-761, small-lane blocks 731-744 (E3)
- [ ] `ALL_KINDS` — 550 entries; fix the 4 stale decls (E4)
- [ ] `cs_id` — d42's 51 kinds now protocol==cs (578-630); g8's 17 moved
      kinds keep their cs hand rows (61-226), only proto changes
- [ ] `bestiary.rs` `entry_for_npc`/`try_entry_for_npc` — the 49 dedupe
      folds (E3) plus d42's new folds (Seagull2/Grebe2/GoldGoldfishWalker/
      ArmedTorchZombie → visible twin)
- [ ] `find_frame` — f26 (`63417edd`) owns 494-509, f27 (`0b8e225d`) owns
      463-472/489-493 keep-one verdicts vs d40's inline arms (480/481/482
      keep d40 INLINE; 473-477/479/483-488 keep the extra7b MODULE arms;
      463/467/471/472/492 are f27 FIX kinds). g8's 34 machine-wiring kinds
      claim the style axis — their `update_kind_ai` arms retire to
      `unreachable!` markers (the d23/g9 precedent)
- [ ] `npc_ai.rs` / `main.rs` — c10's untracked `npc_ai_critter.rs` +
      `npc_ai_style65.rs`, d39's untracked `npc_ai_style65.rs` (SAME module
      name — a merge conflict; keep one, the c10 version is the waterfowl
      lane's, d39's is the concurrent-writer WIP), d125's untracked
      `render/empress_draw.rs`
- [ ] `render/entities.rs` `npc_sheets` chain — add `D125_SHEETS` (local
      tip `2dfe026d2`), the c3/c4/c7/c8/c9/c10/d23/d120/e26/g8 sheet chains;
      every branch edited the same `.chain(...)` site (§4 warning stands)
- [ ] `world_sim/npcs.rs` — d42's BloodEel worm family (621-623), the
      c8 TruffleWormDigger Transform chain

## E6. Local-only work — resume-dispatch list (read-only probe)

Probed via `git -C C:\Users\DreadLord\Desktop\TR-W4-<lane> log/status/diff`,
NO file recursion, NO writes. These lanes have work that is NOT on origin
and must be resumed/pushed before the merge wave can see it:

| Lane | Worktree | Unpushed work | Resume action |
|---|---|---|---|
| **d33** | TR-W4-d33 | uncommitted +5 dragonflies 595-599 (npc.rs/data.rs) | commit + push; vanilla-free, no renumber |
| **d39** | TR-W4-d39 | census `8e4b87c9` pushed; uncommitted WIP squats 355-359 (concurrent writer, violates 108) | **discard the 355-359 WIP** (c7/c8/c9 own those kinds); port the 10 unclaimed 351-420 to 812-821 (E4) |
| **d41** | TR-W4-d41 | claim `4ed22f0b` pushed, no code | resume band 491-560 vanilla-free |
| **d43** | TR-W4-d43 | base, no work | resume band 631-697 vanilla-free (skip g10 pets) |
| **g10** | TR-W4-g10 | uncommitted +10 town pets 637-683 (npc.rs/data.rs/saveglue/clock) | commit + push; vanilla-free, no renumber |
| **c10** | TR-W4-c10 | uncommitted style65/critter wiring + 2 untracked modules | commit + push (no new kinds) |
| **d125** | TR-W4-d125 | local tip `2dfe026d2` sheets phase (NPC_636/661, slots 181→183) + untracked empress_draw.rs | **push `2dfe026d2`** — the sheets/slots are NOT on origin tip `d8047a10` |
| d36 | TR-W4-d36 | claim `65e481f9` pushed, clean | resume port-local 762-811 (E4 re-pin) |
| d40 | TR-W4-d40 | 13 untracked atlas-sim tools (no code) | junk/scratch — leave |
| c8 | TR-W4-c8 | uncommitted npc.rs (sweep 6 in flight) | let the lane finish + push |
| c3/c4 | TR-W4-c3/c4 | uncommitted spawner.rs / npc_ai_style16.rs | let the lanes finish + push |

d36/d39/d43 are NOT "zero-push" anymore (d36/d39 pushed claims/census);
d41/d42 are NOT "no-branch" anymore (both exist; d42 pushed 51 kinds).
The §0 rows for these lanes are superseded by E0.

## E7. Merge ORDER (extend) — SUPERSEDES §7

**d34 → d38 → d37 → d40 → d42(vanilla-free 578-630) → d35(698-730, last) →
the small lanes (c3/c4/c6/c7/c8/c9/c10/d23/d120/d125/e26/g8) with their
731-761 shifts + dedupe drops.**

Rationale: the first four are pairwise clean and keep pushed ids; d42
returns to its vanilla-free invariant (clearing the d37 squat); d35 goes
last-with-renumber against the final roster (§7 rule 2 stands); the small
lanes merge after, each dropping its duplicate ports (E3) and shifting only
its contested kinds. The unpushed lanes (E6) must push first. Atlas-repack
wave still owns the cumulative sheet overflow (§5; the union is now 446 NPC
sheets, worse than the §5 360 estimate — research/113's two-page form is
mandatory, d38's crossnote `6f9dd02d` confirms BAF-alone dies at 8278).

No code was changed on this branch; this extension is the deliverable.
Witness: the extend census ran against the exact tips in E0 (git-show +
`git -C` worktree probes, no working-tree writes, no junction recursion);
the collision matrix, all three adjudication plans, and the canonical
id-map are reproducible from those tips (scripted extraction, zero-dup
verified). Counts: 354 refs swept, 77 touched npc.rs/data.rs, 18 pushed
kind-claimants, 280 distinct new kinds, 77 contested ids / 184 ports,
49 duplicate ports, canonical max id 761.

---

# RECEIPTS — the verification round (supersedes E2-E6 counts where they conflict)

Status: RESEARCH / RECEIPTS — still no code changes on this branch.
Author lane: W4-d37 (executor, receipts phase). This round (a) answers
the c9 × d37 359/360 challenge with source evidence, (b) receipts d37's
ZERO-move, (c) re-reads every origin tip + local worktree (E6 refresh —
five lanes moved since the E-census), (d) applies seven corrections
(R1-R7) to the canonical plan, re-verifies it collision-free by script,
and (e) publishes the per-lane renumber-INSTRUCTION tables (§E8) each
shifting lane executes against its own tip.

## R0. Tips re-read (this round; changes vs E0 in bold)

| Lane | E0 tip | Receipts tip | Delta |
|---|---|---|---|
| c3 | `9ea0ee90` | **`9299046f7`** | docs/witness only — kinds unchanged (273 arms, bijection OK) |
| c4 | `22cec16f` | **`17a241f6a`** | +dup-check verdicts `318e28709`, catch census, Pufferfish justHit fix — kinds unchanged (283 arms) |
| c10 | `c48bbb1e` | **`9641e20db`** | sweep items 1-4 (BirdDroppings keep-one vs d20, TransformVisuals module, sound census) — kinds unchanged (278 arms); still dirty (style65/critter wiring + untracked `npc_ai_critter.rs`) |
| d120 | `4b17a2cf` | **`c5e68a80`** | shop-index/chatter wiring — kinds unchanged (275 arms) |
| d125 | `d8047a10` (+local `2dfe026d2`) | **`cd2b1b1e`** | the sheets phase AND the draw phase (`render/empress_draw.rs`, Extra_188 packed) are now PUSHED — E6's "push 2dfe026d2" action is DONE |
| d33 | `dcb59b7f` + uncommitted | **`b08acf463`** | the dragonflies are PUSHED and GREW: `ba8b9b5c6` five colours 595-599 + `e71010d3c` Yellow 600/Gold 601 + `b08acf463` net-fold witness fix. 7 kinds, ALL_KINDS 277, aiStyle-114 axis fully wired. E6's d33 action DONE |
| d39 | `8e4b87c9` + WIP | **`024c9877f`** | the squatter WIP was CHECKPOINTED VERBATIM and self-declared OFF-PLAN/SUPERSEDED (commit message cites 108 + research/125); working tree now carries the census-correct start (Butterfly at 781 in the dirty diff) |
| f26/f27 | `63417edd`/`0b8e225d` | **`f1a0eaa9c`/`0e1874280`** | FindFrame-only still — zero data.rs delta |
| d34/d35/d37/d38/d40/d42/g8/c6/c7/c8/c9/d23/e26 | as E0 | unchanged | d34 `5f3435a8`, d35 `6a2a155a` (dirty: atlas-repack docs+tools), d37 `bdd59afb`, d38 `6f9dd02d`, d40 `8fe1d6d3` (untracked atlas-sim scratch), d42 `9b22c091` clean, g8 `82c93cec` clean |

Local-only probes (read-only `git -C`, no junction recursion):
- **g10**: STILL base + uncommitted, and the pets are now TEN files dirty
  (+ untracked `npc_findframe_pet.rs`): TownCat 637/TownDog 638/TownBunny
  656/TownSlimeBlue 670/TownSlime{Green,Old,Purple,Rainbow,Red,Yellow}
  678-683, vanilla-free cs==proto. Zero clash with the canonical plan
  (scripted). E6 action stands: commit + push, no renumber.
- **d41**: claim pushed, dirty `npc_buffs.rs` only (no kinds). Band
  491-560 vanilla-free stands; skip c4's pushed 509.
- **d43**: base, clean. 631-697 vanilla-free; skip g10's ten pets, d125's
  636/661 (R4 makes those protocol==cs), c4's 688/692, c3's 694.
- **d36**: claim pushed + untracked census doc
  (`109-d36-npc-kinds-141-210-census.md`). Its claimed 731-780 band is
  superseded twice over — see R7 (767-816).
- No `d33b`/`d42b`/`g8b`/`c4b`/`d23b`/`c4c` branch or worktree exists on
  this machine or origin (the Lead's names are the *successor phases* of
  d33/d42/g8/c4/d23 — the tables below address the existing branches).

Scripted re-census (git-show per tip, `to_u16`↔`from_u16` bijection +
ALL_KINDS-set equality per lane): all 20 tips bijective, zero duplicate
arms. ALL_KINDS declaration drift: **c9 `270` declared / 270 listed but
the four snail kinds are MISSING from the array entirely** (worse than
E4's "decl stale": the array omits Snail/GlowingSnail/MagmaSnail/
Shimmerfly, so its round-trip test never walks them), d125 `270`/272,
g8 `270`/304. d42's tip declares 321 correctly.

## R1. The c9 × d37 359/360 adjudication — d37 KEEPS, c9 shifts (receipt)

The Lead's question: c9's research/109 (§3, tip `2042d77f`) says
359/360 are "squatted by d37 FlyingFish/FlyingSnake" while c9's
Snail/GlowingSnail claim them "vanilla-free" — and c4's verdict
(`318e28709`) already ruled keep-d37-359 for FlyingFish. Are the c9
snails a DIFFERENT-kind-same-proto collision the E-plan missed?

**The E-plan did NOT miss it.** E1 already carries the row
`d37 × c9 | 2: 359/360 | HARD` and E3 shifts Snail 359→740,
GlowingSnail 360→741 (MagmaSnail 655 keeps). The matrix covers it;
no correction of the collision record is needed. What the c9 lane
disputes is the *direction* — its 109 §3 guidance argues d37's band
should move instead. That guidance is REJECTED, with evidence:

1. **c9's "vanilla-free" premise is false for 359/360.** Vanilla-free
   means *the protocol id is free AND equals the cs id*. c9's phase 1
   `37baf0d1c` (01:08) picked "next free above 353" **against the landed
   base only** — 109 §1 admits the scan predates the kinds-wave pushes;
   d37's `750d8e44` (04:00) claimed the same corridor for FlyingFish/
   FlyingSnake the same night. Neither lane saw the other; commit order
   is NOT the tie-break (both are unlanded branch-only ids). Two lanes
   picking the same corridor is exactly the d35×d40 pattern — the E2
   principle decides.
2. **The kinds-lane owns the corridor (E2 rule 1).** d37's 354-399 is
   the contiguous, witness-pinned anchor band of a kinds-coverage lane;
   c9 is a wiring lane (style-67) whose two contested kinds are
   incidental census additions. E2 already weighed moving d37 (plans
   a/b): 46 renumbers + a shattered anchor vs 2 renumbers on c9.
3. **c9's renumber is FREE of its feared breakage — the invariant it
   cites does not exist in the seam it fears.** 109 §4d warns "the f47
   table keys would break" if c9 renumbers. Read the actual code:
   `world_sim/npcs.rs:3554` (f47 tip `a2561d6d9`) calls
   `resolve_npc_shimmer(app.npcs[i].kind.net_id() as u16, …)` — and
   `net_id()` (`npc/data.rs:1186`) reads **`bestiary::ENTRIES.net_id`,
   NOT `to_u16`** (the doc comment says exactly this: "**Not**
   `NpcKind::to_u16` — that is the port's internal protocol id space").
   The snails' ENTRIES rows carry net_id 359/360/655 (cs ids) and DO NOT
   CHANGE under a protocol renumber. `SHIMMER_TRANSFORM_TO_NPC`
   (359→677, 360→677, 655→677), `kind_from_net_id`, the catch/release
   chain (`bug_net.rs` keys `cs_id()`), the shimmer-immunity set (keys
   `cs_id`), `snail_light` — every seam c9 names is cs/net-keyed. Only
   the two `to_u16`/`from_u16` arms move: 359→740, 360→741.
4. **The c4 verdict composes.** c4's `318e28709` already conceded
   FlyingFish to d37@359 ("KEEP d37's 359 + c4's world-sim wiring").
   With c9's snails at 740/741 and c4's FlyingFish port dropped, proto
   359/360 have exactly ONE owner each — the E3 table stands as written.

**Receipt: c9 executes the two-arm shift (E8 c9 table); its 109 §3
guidance items 1-2 are superseded by this section.** c9's 655/677 rows,
all wiring, all witnesses keyed on cs/net ids stay untouched. The
Snail/GlowingSnail protocol ids were never landed (branch-only), so the
S1-F-1 freeze does not apply.

## R2. The c4 verdict's CrimsonGoldfish exception (E3 correction)

E3's drop table said `CrimsonGoldfish keep d40@465 drop c4@465`. The c4
dup-check (`318e28709`, read in full this round) PROVED d40's row
defective: kb 0.4 vs the C# no-row reset 1f (`NPC.cs:14483-14495`) and a
missing `vanilla_type` extension that makes `update_swimmer` hit
`unreachable!` at runtime. **Correction: keep c4's IMPLEMENTATION at the
shared id 465, retire d40's duplicate row** (kind + kb arm + inline
FindFrame; d40's band keeps every other id). The id does not move — 465
stays inside d40's frozen corridor; only the surviving row's provenance
flips. E8's d40 table carries the drop.

Same-round composition check of c4's other verdicts: BloodFeeder
keep-d37@373 and FlyingFish keep-d37@359 match E3 exactly. The fish five
(581/587/592/607/615) "c4 stands" — under the FINAL plan d42's renumber
lands its owned rows at exactly those ids (protocol==cs), so both
verdicts agree on the wire; c4's duplicate ports drop per the E3
kinds-lane rule and c4 keeps its style-16/44 wiring.

## R3. d33's dragonflies are PUSHED — d42 drops seven more rows

E0 saw d33's five dragonflies as uncommitted local work and gave d42
ownership of cs 595-601 inside its 578-630 band. d33 has since pushed
the FULL family (`b08acf463`): seven kinds 595-601 (Yellow 600 + Gold
601 added), the aiStyle-114 axis arm, the Hide-fold credit census (five
colours fold onto Red 599; Gold 601 keeps its own visible card), the
catch/release chain 4334-4340, slots 181→183. d42's tip ports the same
seven at 369-375 → its planned renumber would land them at 595-601 —
a same-kind duplicate, not a wire collision.

**Correction: canonical owner d33 (the fully-wired superset), d42 DROPS
its seven dragonfly rows** (variant + to/from arms + ALL_KINDS + ENTRIES
+ stat arms; its BloodEel-style wiring is untouched — the dragonflies
have none on d42). d42's renumber table shrinks 51→44 moves. d33 keeps
595-601 vanilla-free, zero renumber. Final ids unchanged (595-601 were
already the plan's values via d42's cs mapping).

## R4. d125's EmpressOfLight 229 — the reserved-hole rule applies to it too

E3 kept d125's EmpressOfLight@229 and moved only PrismaticLacewing
237→742. But 229 is the Pirate's vanilla cs id, owned by d37's Pirate
(proto 361, ENTRIES net_id 229) — the SAME reserved-cs-hole squat the c4
verdict flagged for CorruptGoldfish/Arapaima (`NPCID.cs`: 229 Pirate,
237 JungleCreeperWall, 241 BloodFeeder). The hole rule ("cs-owned,
never reassign") cannot exempt d125 while condemning c4.

**Correction: EmpressOfLight 229→636 — its OWN vanilla cs id**
(`NPCID.cs:11949` HallowBoss=636; free in base protocol space, free in
the canonical plan, below NPCID.Count). This RESTORES the vanilla-free
invariant for d125's boss instead of parking it port-local; its ENTRIES
net_id 636 and sheets NPC_636 already key the cs id, so (per R1's
net_id-seam evidence) only the two protocol arms move. PrismaticLacewing
237→742 stands (E3; 661 would also be free, but 742 keeps the small-lane
block contiguous and the Lacewing's release chain is net-keyed either
way — no invariant prefers 661).

## R5. c4's Arapaima 237 — execute the flagged renumber

The c4 verdict's secondary flag ("merge-wave action: renumber
CorruptGoldfish/Arapaima to genuinely free port-local ids") is now
scheduled, not just flagged: CorruptGoldfish drops to d34@277 (E3), and
**Arapaima 237→762** (first free id above the canonical small-lane/g8
block; cs 157 is base-owned BoundWizard so vanilla-free is impossible).
Its style-16 wiring, sheet NPC_157 and ENTRIES net_id 157 are untouched.

## R6. d120's town-NPC ids 241-244 squat d37-owned cs holes

E3 kept d120@241-244 "vanilla-free" — but they are NOT: proto 241-244
sit in the reserved hole 237-249, and those cs ids belong to d37's
pushed kinds (241 BloodFeeder@373, 242 BloodJelly@374, 243 IceGolem@375,
244 RainbowSlime@376 — ENTRIES rows verified on `bdd59afb`). d120's
SantaClaus/Truffle/Steampunker/Cyborg have cs 142/160/178/209, all
base-protocol-owned (proto==cs impossible). **Correction: 241→763,
242→764, 243→765, 244→766.** BartenderUnconscious 245 already drops to
d42@579 (E3; 245 is Golem's cs id — the drop clears the last hole-squat).
The town wiring (shop indices, chatter, housing) is kind-keyed — the
`c5e68a80` delta touched no id arms.

## R7. Future-lane guidance re-pin (supersedes E4's guidance paragraph)

The R4-R6 moves extend the canonical max from 761 to **766**. Re-pin:
- d36 (141-210, ~50 kinds): port-local **767-816** (was 762-811).
- d39 (351-420, 10 unclaimed per its pushed census research/125):
  port-local **817-826** (was 812-821). Its dirty-tree Butterfly@781 is
  OFF-PLAN again — re-pin to 817 when it resumes; the checkpointed
  squatter WIP (355-359) stays discarded per E6.
- d41: 491-560 vanilla-free (skip 521/541-545/547 per its claim; also
  skip c4's pushed 509 GiantFlyingAntlion).
- d43: 631-697 vanilla-free, skip g10's 637/638/656/670/678-683, d125's
  636/661 (636 now proto-owned per R4), c4's 688/692, c3's 694.
- g10/d33: keep cs==proto as pushed/local (zero clash, scripted).
Grand-union max ≈ **826** ≪ i16::MAX; the round-trip test keeps guard.

## R8. d37 ZERO-MOVE receipt (the Lead's item 2)

Verified two ways:
1. `git diff 750d8e44 bdd59afb --stat -- crates anchors.txt` → **EMPTY**
   (the only delta on this branch since the port commit is
   `docs/research/108-roster-master-reconcile.md` itself). Every to_u16/
   from_u16/cs_id/ALL_KINDS/ENTRIES arm is byte-identical to the
   witness-pinned port commit.
2. The scripted final-plan check: all 46 d37 kinds have
   `final == pushed` (354-399 contiguous, bijective, zero base clash,
   zero clash with any other lane's FINAL ids). The canonical plan
   moves 114 kinds across 12 lanes — **d37 contributes 0**. No
   renumber-hunks are owed by this lane; the E8 d37 table is the
   keep-receipt.

Also received: d37's FlyingFish@359/FlyingSnake@360/Squid@358 are the
canonical survivors of the c4/g8 duplicate ports (E3 + R1/R2); the
dropped lanes keep their wiring keyed on these kinds (c4's style-44
world-sim pre-pass, g8's axis arms).

## R9. Final count pins (supersedes E4 where they conflict)

Re-derived by script from the R0 tips + the corrected plan
(`tools/_d37_receipts_*.py`, git-show extraction, reproducible):

| Pin | E4 said | RECEIPTS value | Derivation |
|---|---|---|---|
| `ALL_KINDS` len | 550 | **550** | 270 base + 280 owned (336 ports − 56 drops; d33's +2 and the recounted dup set change E4's 329−49 arithmetic, same total) |
| `to_u16`/`from_u16` arms | 550 | **550** each | bijection verified per lane; deny-unreachable guards the union |
| `s1_f_3` coverage pin | 550 | **550** | covered + sentinel walk |
| bestiary `ENTRIES` rows | 489 | **486** | 241 base + 245 owned new-kind rows (294 naive − 49 dup-credit rows); all 486 positive net ids DISTINCT (scripted). E4's 249 base was wrong — base has 241 (the §1 table's own value) |
| owned kinds WITHOUT a new entry | 40 | **35** | the Hide folds/segments/no-entry fixtures (list in the census JSON); d33's five hidden colours DO carry own rows (catch chain), so E4's "4 dragonflies no-entry" overcounted |
| `BESTIARY_NPC_SHEET_IDS` | ≤357 | **323 as-pushed deduped; 411 after gap close** | 181 + 180 naive adds − 38 slots on dropped dup ports = 323; + 88 real gaps (c4 5, c7 4, c8 1, d120 4, d40 37, d42 37) = 411. d35's 12 deferred sheets + 5 deliberate Hide folds (d33) are NOT gaps. d38's +35 landed; d33's +2 landed; d125's +2 landed |
| `STATIC_SHEET_NAMES` NPC rows | 446 | **455** | base re-censused at **270** NPC_ rows on `a725acb0` (the §0/§1 "248" undercounted; scripted, zero dups) + 185 distinct new NPC_ names across the 20 tips |
| `SFX_CUES` NPC hit/death cues | 103 | **103 unique** | 55 base + 71 naive adds → 103 distinct (naive append 126 rows; dedupe 23) |
| max protocol id | 761 | **766** (grand union ≈826) | R4-R6 extend the small-lane block; i16-safe |
| renumber count | 110 | **114 owned kinds move, 166 keep, 56 ports drop** | per-lane: d35 33, d42 44, g8 17, c10 4, d120 4, c8 3, c7 2, c9 2, d125 2, c4 1, d23 1, e26 1 |

Stale ALL_KINDS declarations to fix at merge (re-censused): **c9 270
declared AND 270 listed — the 4 snail kinds are absent from the array**
(add them, then reconcile to 550), d125 270→272, g8 270→304. d42's tip
is correct (321). The merge reconciles every decl to the 550 union.

## E6-R. Resume-dispatch table — CURRENT statuses (supersedes E6)

| Lane | Status now | Remaining action |
|---|---|---|
| d33 | **DONE** — 7 dragonflies pushed `b08acf463`, clean | none (R3: d42 drops its dups) |
| d125 | **DONE** — sheets `2dfe026d2` + draw `cd2b1b1e3` pushed, clean | execute R4 renumber (229→636, 237→742) at merge |
| d39 | census pushed; WIP checkpointed OFF-PLAN `024c9877f`; dirty tree starts Butterfly@781 | re-pin to **817-826** (R7); discard the checkpointed 355-359 blob at merge |
| g10 | STILL LOCAL — 10 dirty files + untracked `npc_findframe_pet.rs`, base HEAD | commit + push the 10 pets (637/638/656/670/678-683, cs==proto, zero clash) |
| d41 | claim pushed; dirty npc_buffs.rs (no kinds) | resume 491-560 vanilla-free (skip c4 509 + claim skips) |
| d42 | **51 kinds pushed** `9b22c091`, clean | execute the E8 44-move renumber to 578-630 + 7 dragonfly drops (R3) |
| d43 | base, clean | resume 631-697 vanilla-free (R7 skip list) |
| d36 | claim pushed + untracked census doc | commit the census; band re-pinned **767-816** (R7) |
| c10 | sweep pushed `9641e20db`; dirty style65/critter wiring + untracked npc_ai_critter.rs | commit + push wiring; execute E8 4-move + 4 drops |
| c8 | tip unchanged; only untracked scratch tool | execute E8 3-move + 6 drops |
| c3 | docs pushed `9299046f7`; dirty spawner.rs | let the lane finish; E8: 2 drops, WaterBoltMimic 694 keeps |
| c4 | verdicts pushed `17a241f6a`; only untracked scratch | execute E8 1-move (Arapaima 762) + 8 drops; keep CrimsonGoldfish impl (R2) |
| d40 | tip unchanged; untracked atlas-sim scratch | E8: 1 drop (CrimsonGoldfish row, R2); 37-slot gap rides the merge wave |
| d35 | atlas-repack docs dirty on `6a2a155a` | let the lane finish; E8 33-move +269 renumber LAST (E7) |

The Lead's "d33b-R4-active/d42b-active/g10-local" read: no `*b` branches
exist — d33's successor phase IS `b08acf463` on the SAME branch (pushed,
done), d42's active phase IS `9b22c091` (pushed, clean), g10 is local as
E6 said. Statuses above are the origin-read truth.

## E8. Per-lane renumber INSTRUCTIONS (the shifting lanes' work orders)

Each shifting lane executes its table against ITS OWN tip: change the
listed `to_u16` arms AND the mirrored `from_u16` arms (the bijection +
deny-unreachable tests guard the splice), update the band's witness
pins/comments that name the old protocol id, and DROP the listed
duplicate ports (variant + to/from arms + ALL_KINDS row + ENTRIES row +
stat arms; keep the canonical owner's row; the dropped lane's AI/draw/
sound wiring stays, keyed on the canonical kind — E3's rule). `cs_id`,
ENTRIES `net_id`, sheets, render keys and every cs/net-keyed seam DO NOT
CHANGE (R1's seam evidence). No lane touches another lane's file.

### c10 - 4 owned, 4 renumber, 4 drop

| Kind | old `to_u16`/`from_u16` | new id | cs_id |
|---|---|---|---|
| Duck | 362 | **731** | 362 |
| Duck2 | 363 | **732** | 363 |
| DuckWhite | 364 | **733** | 364 |
| DuckWhite2 | 365 | **734** | 365 |

DROP duplicate ports (keep canonical owner's row+wiring):
- Seagull: drop c10@602 -> keep d42@602 (renumbered from 376)
- Seagull2: drop c10@603 -> keep d42@603 (renumbered from 377)
- Grebe: drop c10@608 -> keep d42@608 (renumbered from 382)
- Grebe2: drop c10@609 -> keep d42@609 (renumbered from 383)

### c3 - 1 owned, 0 renumber, 2 drop

DROP duplicate ports (keep canonical owner's row+wiring):
- GiantCursedSkull: drop c3@289 -> keep d38@289
- CursedSkull: drop c3@355 -> keep d34@267

WaterBoltMimic 694 keeps (vanilla-free, `NPCID.cs:12065`).

### c4 - 5 owned, 1 renumber, 8 drop

| Kind | old `to_u16`/`from_u16` | new id | cs_id |
|---|---|---|---|
| Arapaima | 237 | **762** | 157 |

Keep as pushed: CrimsonGoldfish@465 (R2: YOUR impl survives),
GiantFlyingAntlion@509, Pufferfish@688, Orca@692.

DROP duplicate ports (keep canonical owner's row+wiring):
- CorruptGoldfish: drop c4@229 -> keep d34@277
- BloodFeeder: drop c4@241 -> keep d37@373
- FlyingFish: drop c4@354 -> keep d37@359
- FlyingAntlion: drop c4@581 -> keep d42@581 (renumbered from 357)
- EyeballFlyingFish: drop c4@587 -> keep d42@587 (renumbered from 363)
- GoldGoldfish: drop c4@592 -> keep d42@592 (renumbered from 366)
- Pupfish: drop c4@607 -> keep d42@607 (renumbered from 381)
- Dolphin: drop c4@615 -> keep d42@615 (renumbered from 389)

Your style-16/44 world-sim wiring, FindFrame helpers and sheets stay —
the merge routes the canonical kinds through them (your own verdict's
compose rule). The five dropped fish land at the SAME ids you pushed,
so no wiring id ever changes.

### c6 - 0 owned, 1 drop

- PumpkingBlade: drop c6@328 -> keep d38@328 (the whole lane's kind
  contribution dedupes; its style-59 wiring stays).

### c7 - 4 owned, 2 renumber, 0 drop

| Kind | old `to_u16`/`from_u16` | new id | cs_id |
|---|---|---|---|
| Firefly | 355 | **735** | 355 |
| LightningBug | 358 | **736** | 358 |

Keep as pushed: Lavafly@654, Shimmerfly@677 (canonical Shimmerfly owner;
c9's duplicate drops to you). Your pooled `style == 64` sim arm subsumes
c9's kind-gated arm at merge (c9's 109 §4e compose stands).

### c8 - 3 owned, 3 renumber, 6 drop

| Kind | old `to_u16`/`from_u16` | new id | cs_id |
|---|---|---|---|
| Worm | 357 | **737** | 357 |
| TruffleWorm | 374 | **738** | 374 |
| TruffleWormDigger | 375 | **739** | 375 |

DROP duplicate ports (keep canonical owner's row+wiring):
- GoldWorm: drop c8@448 -> keep d40@448
- EnchantedNightcrawler: drop c8@484 -> keep d40@484
- Grubby: drop c8@485 -> keep d40@485
- Sluggy: drop c8@486 -> keep d40@486
- Buggy: drop c8@487 -> keep d40@487
- Maggot: drop c8@606 -> keep d42@606 (renumbered from 380)

The TruffleWormDigger Transform chain + own ENTRIES row (net 375) stay;
its slot is one of the 88 merge-wave gap closes (R9).

### c9 - 3 owned, 2 renumber, 1 drop

| Kind | old `to_u16`/`from_u16` | new id | cs_id |
|---|---|---|---|
| Snail | 359 | **740** | 359 |
| GlowingSnail | 360 | **741** | 360 |

Keep as pushed: MagmaSnail@655.

DROP duplicate ports (keep canonical owner's row+wiring):
- Shimmerfly: drop c9@677 -> keep c7@677

R1 is your adjudication receipt: the f47/catch/light/immunity seams all
key cs/net ids, which do not move. ALSO fix your ALL_KINDS array: the
four kinds are missing from it entirely (R0) — add them (post-drop: the
three snails) before the union reconcile.

### d120 - 4 owned, 4 renumber, 1 drop

| Kind | old `to_u16`/`from_u16` | new id | cs_id |
|---|---|---|---|
| SantaClaus | 241 | **763** | 142 |
| Truffle | 242 | **764** | 160 |
| Steampunker | 243 | **765** | 178 |
| Cyborg | 244 | **766** | 209 |

DROP duplicate ports (keep canonical owner's row+wiring):
- BartenderUnconscious: drop d120@245 -> keep d42@579 (renumbered from 355)

R6: 241-244 are d37-owned cs holes (BloodFeeder/BloodJelly/IceGolem/
RainbowSlime); your town wiring is kind-keyed and untouched. Your four
ENTRIES rows (net 142/160/178/209) need merge-wave slots (R9 gap list).

### d125 - 2 owned, 2 renumber, 0 drop

| Kind | old `to_u16`/`from_u16` | new id | cs_id |
|---|---|---|---|
| EmpressOfLight | 229 | **636** | 636 (via ENTRIES; cs_id rides the folded net row) |
| PrismaticLacewing | 237 | **742** | 661 (same) |

R4: 636 is your boss's OWN vanilla id — the move RESTORES protocol==cs.
Sheets NPC_636/NPC_661, slots 183, empress_draw.rs: all net-keyed, all
stay. Fix the stale ALL_KINDS decl 270→272 (R0).

### d23 - 1 owned, 1 renumber, 5 drop

| Kind | old `to_u16`/`from_u16` | new id | cs_id |
|---|---|---|---|
| AngryTrapper | 357 | **743** | 175 (via ENTRIES) |

DROP duplicate ports (keep canonical owner's row+wiring):
- FungiBulb: drop d23@259 -> keep d37@385
- GiantFungiBulb: drop d23@260 -> keep d37@386
- ManEater: drop d23@354 -> keep d34@269
- Snatcher: drop d23@355 -> keep d34@276
- Clinger: drop d23@356 -> keep d35@718 (renumbered from 449)

Your style-9-14 axis arms stay (the d23/g9 unreachable-marker precedent
for the kind-match arms).

### d33 - 7 owned, 0 renumber, 0 drop (KEEP receipt)

BlackDragonfly@595, BlueDragonfly@596, GreenDragonfly@597,
OrangeDragonfly@598, RedDragonfly@599, YellowDragonfly@600,
GoldDragonfly@601 — all keep, vanilla-free cs==proto. You are the
canonical dragonfly owner (R3); d42 drops its seven duplicates.

### d34 - 15 owned, 0 renumber, 0 drop (KEEP receipt)

266-280 as pushed (`5f3435a8` phase 3). Canonical survivor of the
CursedSkull/ManEater/Snatcher/CorruptGoldfish/Vulture/Demon/Harpy/
BlueJellyfish duplicate ports.

### d35 - 33 owned, 33 renumber, 0 drop (+269, order-preserving)

| Kind | old | new | | Kind | old | new |
|---|---|---|---|---|---|---|
| GoblinScout | 429 | **698** | | SkeletonArcher | 454 | **723** |
| Bird | 430 | **699** | | VileSpit | 455 | **724** |
| Pixie | 431 | **700** | | TheHungryII | 456 | **725** |
| ArmoredSkeleton | 432 | **701** | | ChaosElemental | 457 | **726** |
| Mummy | 433 | **702** | | Slimer | 458 | **727** |
| DarkMummy | 434 | **703** | | Gastropod | 459 | **728** |
| LightMummy | 435 | **704** | | IlluminantBat | 460 | **729** |
| Wraith | 436 | **705** | | PossessedArmor | 461 | **730** |
| CursedHammer | 437 | **706** | | Clinger | 449 | **718** |
| EnchantedSword | 438 | **707** | | AnglerFish | 450 | **719** |
| Mimic | 439 | **708** | | GreenJellyfish | 451 | **720** |
| Unicorn | 440 | **709** | | Werewolf | 452 | **721** |
| GiantBat | 441 | **710** | | Clown | 453 | **722** |
| Corruptor | 442 | **711** | | DiggerHead | 443 | **712** |
| DiggerBody | 444 | **713** | | DiggerTail | 445 | **714** |
| SeekerHead | 446 | **715** | | SeekerBody | 447 | **716** |
| SeekerTail | 448 | **717** | | | | |

Mechanical rule: every arm +269. Execute LAST against the final roster
(E7); ENTRIES wholesale from the splice-repaired tip (§7 rule 3).

### d37 - 46 owned, 0 renumber, 0 drop (KEEP receipt — R8)

354-399 byte-identical to `750d8e44` (empty code diff verified). This
lane owes no renumber-hunks.

### d38 - 35 owned, 0 renumber, 0 drop (KEEP receipt)

281-337 as pushed; portrait gap already closed (`6f9dd02d`).

### d40 - 55 owned, 0 renumber, 1 drop

430-488 keep, EXCEPT:
- CrimsonGoldfish: drop d40@465 -> keep c4@465 (R2: your kb-0.4 row +
  missing vanilla_type arm are the defects; c4's impl survives at the
  same id inside your band).
Your 37 portrait slots are the merge wave's gap close (R9).

### d42 - 44 owned, 44 renumber, 7 drop (354-404 -> vanilla-free cs)

Mechanical rule: **new id = the kind's own cs_id** (protocol==vanilla,
the d38/d40 invariant your WIP message already deferred to). Exact arms:

| Kind | old | new | | Kind | old | new |
|---|---|---|---|---|---|---|
| DD2LightningBug | 354 | **578** | | Rat | 384 | **610** |
| BartenderUnconscious | 355 | **579** | | Owl | 385 | **611** |
| WalkingAntlion | 356 | **580** | | WaterStrider | 386 | **612** |
| FlyingAntlion | 357 | **581** | | GoldWaterStrider | 387 | **613** |
| LarvaeAntlion | 358 | **582** | | ExplosiveBunny | 388 | **614** |
| FairyCritterPink | 359 | **583** | | Dolphin | 389 | **615** |
| FairyCritterGreen | 360 | **584** | | Turtle | 390 | **616** |
| FairyCritterBlue | 361 | **585** | | TurtleJungle | 391 | **617** |
| ZombieMerman | 362 | **586** | | BloodNautilus | 392 | **618** |
| EyeballFlyingFish | 363 | **587** | | BloodSquid | 393 | **619** |
| TorchZombie | 364 | **590** | | GoblinShark | 394 | **620** |
| ArmedTorchZombie | 365 | **591** | | BloodEelHead | 395 | **621** |
| GoldGoldfish | 366 | **592** | | BloodEelBody | 396 | **622** |
| GoldGoldfishWalker | 367 | **593** | | BloodEelTail | 397 | **623** |
| WindyBalloon | 368 | **594** | | Gnome | 398 | **624** |
| Seagull | 376 | **602** | | SeaTurtle | 399 | **625** |
| Seagull2 | 377 | **603** | | Seahorse | 400 | **626** |
| LadyBug | 378 | **604** | | GoldSeahorse | 401 | **627** |
| GoldLadyBug | 379 | **605** | | Dandelion | 402 | **628** |
| Maggot | 380 | **606** | | IceMimic | 403 | **629** |
| Pupfish | 381 | **607** | | BloodMummy | 404 | **630** |
| Grebe | 382 | **608** | | Grebe2 | 383 | **609** |

DROP duplicate ports (R3 — d33's pushed family is the canonical owner):
- BlackDragonfly: drop d42@369 -> keep d33@595
- BlueDragonfly: drop d42@370 -> keep d33@596
- GreenDragonfly: drop d42@371 -> keep d33@597
- OrangeDragonfly: drop d42@372 -> keep d33@598
- RedDragonfly: drop d42@373 -> keep d33@599
- YellowDragonfly: drop d42@374 -> keep d33@600
- GoldDragonfly: drop d42@375 -> keep d33@601

(The 588/589 holes in your new band: Golfer/SleepingGolfer — base owns
those kinds at proto 163/164 with cs 588/589 (`NPCID.cs:11853/:11855`;
arms verified on your own tip); the cs-reservation rule keeps 588/589
out of your band. Keep the gaps.)
Your 37 portrait slots + the Seagull2/Grebe2/GoldGoldfishWalker/
ArmedTorchZombie folds ride the merge wave (R9).

### e26 - 1 owned, 1 renumber, 3 drop

| Kind | old `to_u16`/`from_u16` | new id | cs_id |
|---|---|---|---|
| Frog | 361 | **744** | 361 |

DROP duplicate ports (keep canonical owner's row+wiring):
- Duck: drop e26@362 -> keep c10@731 (renumbered from 362)
- LadyBug: drop e26@604 -> keep d42@604 (renumbered from 378)
- GoldLadyBug: drop e26@605 -> keep d42@605 (renumbered from 379)

Your luck wiring is kind-keyed; the aiStyle-115 boundary stays yours.

### g8 - 17 owned, 17 renumber, 17 drop (354-387 -> 745-761)

| Kind | old | new | cs | | Kind | old | new | cs |
|---|---|---|---|---|---|---|---|---|
| PinkJellyfish | 356 | **745** | 64 | | LostGirl | 373 | **755** | 195 |
| Antlion | 357 | **746** | 69 | | Nymph | 374 | **756** | 196 |
| SpikeBall | 358 | **747** | 70 | | IceBat | 380 | **757** | 150 |
| WallCreeper | 366 | **748** | 164 | | Lavabat | 381 | **758** | 151 |
| WallCreeperWall | 367 | **749** | 165 | | GiantFlyingFox | 382 | **759** | 152 |
| IceElemental | 368 | **750** | 169 | | VampireBat | 383 | **760** | 158 |
| Herpling | 369 | **751** | 174 | | RedDevil | 387 | **761** | 156 |
| Derpling | 370 | **752** | 177 | | CrimsonAxe | 371 | **753** | 179 |
| FloatyGross | 372 | **754** | 182 | | | | | |

DROP duplicate ports (keep canonical owner's row+wiring):
- Vulture: drop g8@354 -> keep d34@278
- BlueJellyfish: drop g8@355 -> keep d34@280
- Pixie: drop g8@359 -> keep d35@700 (renumbered from 431)
- Wraith: drop g8@360 -> keep d35@705 (renumbered from 436)
- CursedHammer: drop g8@361 -> keep d35@706 (renumbered from 437)
- EnchantedSword: drop g8@362 -> keep d35@707 (renumbered from 438)
- Mimic: drop g8@363 -> keep d35@708 (renumbered from 439)
- GreenJellyfish: drop g8@364 -> keep d35@720 (renumbered from 451)
- Gastropod: drop g8@365 -> keep d35@728 (renumbered from 459)
- CursedSkull: drop g8@375 -> keep d34@267
- Demon: drop g8@376 -> keep d34@279
- GiantBat: drop g8@377 -> keep d35@710 (renumbered from 441)
- Slimer: drop g8@378 -> keep d35@727 (renumbered from 458)
- IlluminantBat: drop g8@379 -> keep d35@729 (renumbered from 460)
- FlyingSnake: drop g8@384 -> keep d37@360
- Squid: drop g8@385 -> keep d37@358
- Harpy: drop g8@386 -> keep d34@275

Your 34 axis wirings ALL stay (the machine-wiring lane's whole point);
only the 17 duplicate kind rows drop. Fix the stale ALL_KINDS decl
270→304 (R0), then the union reconcile. Your 34 pushed portrait slots
dedupe to the 17 owned kinds at merge (the other 17 sit on dropped
ports; their canonical owners' slots land via the owners' lanes or the
R9 gap/deferral lists — e.g. d35's Mimic/EnchantedSword/GreenJellyfish/
Gastropod sheets are documented deferrals).

### E8 witness

Tables generated mechanically from the verified final plan
(`tools/_d37_receipts_final.json` ← `_d37_receipts_plan.py` +
`_d37_final_verify.py`): 280 owned kinds, 280 distinct final ids, zero
clash with base's 270-id map, max 766; guidance bands 767-826 + g10/d33
cs==proto verified free. 114 moves / 166 keeps / 56 drops. Every count
in R9 re-derived by `_d37_receipts_counts.py` + `_d37_slot_account.py`
from the R0 tips (git-show, no working-tree reads, no junction
recursion). No code was changed on this branch; this extension is the
deliverable.

# SEAL — the broadcast-receipt round (supersedes E8/R9 where they conflict)

Status: RESEARCH / SEAL — still no code changes on this branch (empty
`git diff 750d8e44 HEAD -- crates anchors.txt` re-verified this round).
Author lane: W4-d37 (executor, seal phase). This round (S1) reads every
E8-addressed lane's CURRENT origin tip and receipts which lanes CONSUMED
their renumber tables, (S2) publishes the exact c9 ALL_KINDS fix-hunk
(the bonus finding — still missing at c9's tip), (S3) re-items the
portrait-slot gaps at current tips with fill ownership, and (S4) seals
the merger bible with the amended count pins. All numbers re-derived by
`tools/_d37_seal_*.py` (git-show only, direct paths, no junction
recursion); witnesses `_d37_seal_census.json`, `_d37_seal_final.json`,
`_d37_seal_receipts.json`, `_d37_seal_gapfinal.json`.

## S0. Tips read (this round; changes vs R0 in bold)

The Lead's phase aliases (d42c/d35f/g8b-R3/c4e/d36e/d39r/c9b) are NOT
separate branches — `git branch -a` shows no such refs (E6-R's finding
stands); each is the successor phase ON the same lane branch, read below.
d120-done = its RETIRE tip; d125-done is UNDONE on origin (regression).

| Lane | R0 tip | SEAL tip | Delta |
|---|---|---|---|
| c3 | `9299046f7` | **`9ea0ee90d`** | origin tip is BEHIND R0's read: `9299046f7` (gap-1 witness) + `cb5cef764` (gaps 2+3 dungeon spawn band) are LOCAL-ONLY again — unpushed; kinds unchanged either way (3 new) |
| c4 | `17a241f6a` | **`0df6964be`** | EXECUTED a renumber (`91293a3d2`): CorruptGoldfish 229→405, Arapaima 237→**406**, FlyingFish 354→359 (adopts d37's id); then WITHDREW its fish-five verdicts (`ff8026327`, after d42's drop) and retired (`0df6964be`, research/148) |
| c10 | `9641e20db` | **`5d8387955`** | +12 roster-tail kinds PUSHED (`d7f422049` + flip-site wiring `1e336ee93` + FINAL docs): Worm@357, GoldBird..GoldMouse@442-447, GoldWorm@448, EnchantedNightcrawler/Grubby/Sluggy/Buggy@484-487 — ALL duplicate ports of d40/c8 rows (see S1) |
| d23 | `19109424` | **`43091b01c`** | +3 kinds: SolarFlare@408, VileSpitEoW@411 (new, free), VileSpit@455 (dup of d35's row — d23 ADOPTED d35's pushed id, same compose as c4's FlyingFish) |
| d120 | `c5e68a80` | **`2408233da`** | RETIRE docs (research/143 hand-offs) — kinds unchanged (5) |
| d125 | `cd2b1b1e` | **`d8047a109`** | origin REGRESSED to the phase-5-docs tip: `2dfe026d2` sheets + `cd2b1b1e3` draw + `86e3adfd3` hitbox are LOCAL-ONLY (3 ahead, 0 behind; `merge-base --is-ancestor cd2b1b1e3 origin/...` exit 1). R0's "PUSHED" receipt is UNDONE on origin — the local tip carries everything; **push before merge** |
| d35 | `6a2a155a` | **`e5558b0a9`** | ATLAS FINAL docs (research/113 §10-12 + Appendix B); local ahead `189c04aab` (RETIRE §15, docs) — kinds unchanged (33), renumber NOT executed |
| d38 | `6f9dd02d` | **`1ba0395c7`** | FINAL sheets 1-4 (research/125 §7-10, unlock census, 108 cross-ref) — kinds unchanged (35) |
| d42 | `9b22c091` | **`2d7ac08c3`** | **RENUMBER EXECUTED** (`49346dba6`) + ANALYSIS note + SHEETS census (research/143): 36 moves to 578-630 landed, 8 kinds dropped (see S1 — ownership flips vs E8), 7 dragonfly drops NOT executed |
| d36 | claim | **`bcf4ac551`** | supersede-note: E4 band 762-789, 28 kinds — R7 re-pin (767-816) NOT yet absorbed; exec branch `d36x` at `396a35ceb` carries the same stale E4 band |
| d39 | `024c9877f` | **`907234441`** | batch 3 PUSHED: 7 kinds at 781-787 (Butterfly/ScorpionBlack/Scorpion/Grasshopper/ChatteringTeethBomb/CultistArcherBlue/CultistArcherWhite) — OFF-PLAN vs R7's 817-826 re-pin, but collision-free (scripted) |
| d41 | claim | **`5d036721d`** | batches 1-2 PUSHED: 13 kinds at 494-506 (crawdads/shellies/salamanders), vanilla-free, collision-free |
| g10 | local | **local `5dee28650`** | NOW COMMITTED locally (2 commits: 10 pets + TownSlimeCopper@684 — grew to ELEVEN), still unpushed; cs==proto, zero clash |
| c6/c7/c8/c9/d33/d34/d37/d40/e26/g8 | as R0 | unchanged | c9 `2042d77f1`, g8 `82c93cecf`, d40 `8fe1d6d31`, e26 `048cca6cf` — none executed their E8 tables |

## S1. E8-broadcast receipts — who CONSUMED their tables

Measured per kind at each lane's current tip against the amended plan
(`_d37_seal_receipts.py` → `_d37_seal_receipts.json`). "Consumed" = every
owned kind at its final id AND every non-owned kind absent.

| Lane | E8 work order | Receipt status | Remaining |
|---|---|---|---|
| **d42** | 44 moves + 7 drops | **PARTIAL-EXECUTED** `49346dba6` — 36/36 owned moves DONE (578-630, bijection 313/313, ALL_KINDS 313 correct); it ALSO dropped 8 kinds E8 had assigned to it (see adjudication below) | **7 dragonfly drops** (595-601, keeper d33) — R3's rows still present |
| **c4** | 1 move (Arapaima→762) + 8 drops | **PARTIAL-EXECUTED, own variant** `91293a3d2` — Arapaima→**406** (not 762; free, collision-free — ACCEPTED, see S4), CorruptGoldfish→405 (still owed as a DROP to d34@277), FlyingFish→359 (identical-arm splice ready); fish-five verdicts WITHDRAWN `ff8026327` after d42's drop — c4 now KEEPS them (ownership flip accepted) | **3 drops**: BloodFeeder@241→d37@373, CorruptGoldfish@405→d34@277, FlyingFish@359→d37@359 |
| d33 | KEEP receipt | **CONSUMED** (7/7 at 595-601) | none |
| d34 | KEEP receipt | **CONSUMED** (15/15 at 266-280) | none |
| d37 | KEEP receipt (R8) | **CONSUMED** (46/46 at 354-399, zero-move re-verified) | none |
| d38 | KEEP receipt | **CONSUMED** (35/35 at 281-337) | none |
| d40 | 1 drop (CrimsonGoldfish) | **NOT EXECUTED** — 55/55 keeps intact | 1 drop (CrimsonGoldfish@465, keeper c4 — c4's research/143 `90fd242d9` publishes the exact hunk) |
| d35 | 33 moves +269 | **NOT EXECUTED** (by design — E7 orders it LAST) | 33 moves 429-461→698-730 |
| **g8** | 17 moves + 17 drops | **NOT EXECUTED — LAGGARD** (tip unchanged since E0; ALL_KINDS decl still stale 270/304) | 17 moves 354-387→745-761, 17 drops, decl fix |
| **c9** | 2 moves + 1 drop + ALL_KINDS fix | **NOT EXECUTED — LAGGARD** (tip unchanged; array still missing all 4 snails — S2) | Snail 359→740, GlowingSnail 360→741, Shimmerfly drop, S2 hunk |
| c10 | 4 moves + 4 drops | **NOT EXECUTED — GREW** (+12 dup ports pushed instead) | 4 Duck moves→731-734, **16 drops** (4 waterfowl + 12 new roster-tail dups) |
| c3 | 2 drops | **NOT EXECUTED** (local-ahead work is spawn-band, not kinds) | CursedSkull@355, GiantCursedSkull@289 drops |
| c6 | 1 drop | **NOT EXECUTED** | PumpkingBlade@328 drop |
| c7 | 2 moves | **NOT EXECUTED** | Firefly 355→735, LightningBug 358→736 |
| c8 | 3 moves + 6 drops | **NOT EXECUTED** | 3 moves→737-739; 5 drops to d40 (Maggot@606 drop RETIRED — d42 dropped its row, c8 now KEEPS Maggot, ownership flip) |
| d23 | 1 move + 5 drops | **NOT EXECUTED — GREW** (+3 kinds: 2 new free, 1 dup) | AngryTrapper 357→743; **6 drops** (5 + VileSpit@455→d35@724) |
| d120 | 4 moves + 1 drop | **NOT EXECUTED** (retired with the work order unexecuted) | 241-244→763-766, BartenderUnconscious@245 drop |
| d125 | 2 moves + decl fix | **NOT EXECUTED** (and origin regressed — S0) | EmpressOfLight 229→636, PrismaticLacewing 237→742, decl 270→272, PUSH the 3 local commits |
| e26 | 1 move + 3 drops | **NOT EXECUTED** | Frog 361→744; 1 drop (Duck@362→c10@731); LadyBug/GoldLadyBug drops RETIRED — d42 dropped its rows, e26 now KEEPS them at 604/605 (ownership flip) |

**The d42 ownership-flip adjudication (E8 amendment).** d42's executed
reconcile (`49346dba6`) dropped EIGHT kinds E8 had assigned to it:
FlyingAntlion/EyeballFlyingFish/GoldGoldfish/Pupfish/Dolphin (to c4's
pushed rows at 581/587/592/607/615), LadyBug/GoldLadyBug (to e26's pushed
rows at 604/605), Maggot (to c8's row at 606). E8's rule was "kinds-lane
owns, wiring-lane drops" — d42 executed the INVERSE for these eight,
citing c4's original verdicts. c4 then withdrew its five fish verdicts
(`ff8026327`) and retired, so the rows now exist on exactly ONE tip each.
**The seal ACCEPTS the executed state** (re-opening it would force 8
re-ports + 3 lanes' re-drops for zero wire difference — the final ids are
identical either way): owners flip to c4×5, e26×2, c8×1; the E8 c4/e26/c8
drop lists shrink accordingly (receipts table above). d42's interim
GoldGoldfishWalker@593 OWN entry stays until the merge re-points its fold
onto c4's GoldGoldfish@592 (d42's commit message already schedules this).
The 7 dragonfly drops are NOT covered by this flip — d33's fully-wired
superset stands (R3), d42 still owes those rows' removal.

**Laggards flagged: g8, c9** (zero movement since E0, both with stale
ALL_KINDS declarations — g8 270/304, c9's array missing 4 kinds entirely),
plus **d120/d125 retired/regressed with work orders unexecuted** (their
E8 hunks transfer to the merge wave), and **c10/d23 grew NEW duplicate
ports** instead of executing (their drop lists grew 4→16 and 5→6).

## S2. c9's ALL_KINDS array — STILL missing all 4 snails; exact fix-hunk

Re-read at c9's current tip `2042d77f1` (unchanged since R0): the array
declares `[NpcKind; 270]` and lists exactly 270 rows, but `to_u16`/
`from_u16` carry **274** bijective arms — Snail/GlowingSnail/MagmaSnail/
Shimmerfly are absent from the array. This is not a cosmetic stale decl:
`the_protocol_id_space_round_trips_every_kind` (npc.rs:24254) asserts
`decodable == ALL_KINDS.len()` — **274 != 270, the test FAILS on c9's own
tip**; `s1_f_3` (npc.rs:27828) pins `covered + sentinel == 270` — also
fails (274 decodable). No `c9b` branch exists (local or origin); c9's
successor phase is the same branch.

**Fix-hunk published: `tools/_d37_c9_allkinds_fix.patch`** — a unified
diff against `2042d77f1:crates/tr-app/src/npc.rs`, three sites:
1. npc.rs:23972 — decl `[NpcKind; 270]` → `[NpcKind; 274]`;
2. npc.rs:24250 (after `NpcKind::WyvernTail,`, before `];`) — insert the
   four rows `NpcKind::Snail, NpcKind::GlowingSnail, NpcKind::MagmaSnail,
   NpcKind::Shimmerfly,` with a citation comment;
3. npc.rs:27862 — the s1_f_3 literal pin `270,` → `274,`.

Verified: `git apply --check` exit 0 against the byte-exact tip blob;
post-apply scripted census: decl 274, rows 274, array-set == to_u16-set,
all four snails present. Apply with
`git apply tools/_d37_c9_allkinds_fix.patch` from the c9 worktree. At the
108 merge the Shimmerfly row drops (keeper c7) and the union reconcile
re-pins to the S4 value; c9 should land the hunk NOW so its own witnesses
run green.

## S3. The portrait-slot gaps, re-itemized at current tips

R9's "88 gaps / 323 deduped / 411 target" was derived at the R0 tips.
Re-derived at the SEAL tips (`_d37_seal_gapfinal.py` →
`_d37_seal_gapfinal.json`), with the S1 ownership flips applied and the
slot-survival rule made explicit: **a pushed slot survives iff its id is
the canonical net_id of an owned kind** (the grid is net-keyed; a slot
pushed on a duplicate port still serves the canonical kind when the net
ids agree — e.g. d23's VileSpit slot 112 survives for d35's VileSpit).

- Naive append at current tips: **376** (was 361 — c10 +12, d23 +3).
- Surviving distinct slots: **347** (base 181 + 166 distinct net-covered
  adds; of the 195 adds, 27 are intra-union duplicates of nets already
  slotted and only d34's 40/41 adds are net-orphaned — BoneSerpentBody/
  Tail segments whose credit folds; they ride the fold rule, not the gap
  list).
- Deliberate non-gaps: **5 Hide folds** (d33's five hidden dragonfly
  colours) + **8 d35 deferrals still open** (DarkMummy/LightMummy/
  Unicorn/Werewolf/Clown/SkeletonArcher/ChaosElemental/PossessedArmor —
  EnchantedSword/Mimic/GreenJellyfish/Gastropod slots now arrive via g8's
  pushed rows at 84/85/103/122, so those 4 deferrals are slot-covered
  though their draw sheets stay deferred).
- **REAL gaps: 72**, itemized with fill ownership:

| Owner lane | Count | Kinds (canonical net id) | Fill vehicle |
|---|---|---|---|
| c4 | 10 | GiantFlyingAntlion 509, Arapaima 157, CrimsonGoldfish 465, Pufferfish 688, Orca 692 + the five flipped fish: FlyingAntlion 581, EyeballFlyingFish 587, GoldGoldfish 592, Pupfish 607, Dolphin 615 | c4 packed STYLE16/44 sheets but never appended grid slots; the five fish gaps were d42's in R9 — they FOLLOW THE OWNERSHIP FLIP to c4 (or the merge wave appends all 10 in one hunk) |
| c7 | 3 | Firefly 355, LightningBug 358, Lavafly 654 | FIREFLY_SHEETS packed, no grid slots (Shimmerfly 677's slot arrives via c9's pushed row — net-covered) |
| c8 | 1 | TruffleWormDigger 375 | CRAWLER_SHEETS packed, no grid slot |
| d120 | 4 | SantaClaus 142, Truffle 160, Steampunker 178, Cyborg 209 | lane RETIRED without slots — merge-wave append |
| d40 | 26 | CultistTablet 437, CultistDevote 438, CultistBossClone 440, SkeletonMerchant 453, CultistDragonHead 454, Butcher 460, CreatureFromTheDeep 461, Fritz 462, Nailhead 463, CrimsonBunny 464, Psycho 466, DeadlySphere 467, DrManFly 468, ThePossessed 469, CrimsonPenguin 470, ShadowFlameApparition 472, BigMimic{Corruption,Crimson,Hallow,Jungle} 473-476, Mothron 477, MothronSpawn 479, Medusa 480, GreekSkeleton 481, GraniteGolem 482, GraniteFlyer 483 | **d40b's 38-entry half-open question ANSWERED**: d40's tip has 38 new ENTRIES rows but 0 slot adds; 11 of the 38 are now net-covered by c10's pushed roster-tail slots (GoldBird..GoldMouse 442-447, GoldWorm 448, EnchantedNightcrawler/Grubby/Sluggy/Buggy 484-487) and CrimsonGoldfish 465's slot rides c4's gap row — leaving 26. The 38-entry roster is COMPLETE; only the grid append is open. Merge-wave append (d40's lane is data-only by design) |
| d42 | 28 | DD2LightningBug 578, WalkingAntlion 580, LarvaeAntlion 582, FairyCritter{Pink,Green,Blue} 583-585, ZombieMerman 586, TorchZombie 590, GoldGoldfishWalker 593 (interim entry), WindyBalloon 594, Seagull 602, Grebe 608, Rat 610, Owl 611, WaterStrider 612, GoldWaterStrider 613, ExplosiveBunny 614, Turtle 616, TurtleJungle 617, BloodNautilus 618, BloodSquid 619, GoblinShark 620, BloodEelHead 621, Gnome 624, SeaTurtle 625, Seahorse 626, GoldSeahorse 627, Dandelion 628, IceMimic 629, BloodMummy 630 | d42's SHEETS phase (`2d7ac08c3`, research/143) inventoried all 43 dims but ruled NO-PACK (8382 > 8192) — slots ride d35's lazy-load wave (research/113 §10); the 8 dropped kinds' gaps left with the ownership flip; Seagull2/Grebe2 stay deliberate folds |

- **Core slot target: 347 + 72 = 419** (R9's 411 re-derived: c10's 12
  pushed adds net-cover 11 of d40's 37 gaps, Shimmerfly 677's slot
  arrives via c9's row, 4 of d35's 12 deferrals are net-covered by g8's
  pushed slots, the ownership flip moves 5 fish gaps from d42 to c4 and
  GoldGoldfishWalker's interim entry adds 1; every delta scripted).
  Grand union adds d39's 6 pushed slots (356/366/367/377/378/
  379), g10's 11 local pet slots (637-684), d41's 3 entry-nets needing
  slots (494/496/499) → **grand target ≈ 439** when those lanes merge.

## S4. The SEAL — amended canonical plan + merger bible

**Amendments to the receipts plan** (all scripted-verified,
`_d37_seal_verify.py` → `_d37_seal_final.json`):
1. S1's ownership flips: c4 owns the fish five at 581/587/592/607/615,
   e26 owns LadyBug/GoldLadyBug at 604/605, c8 owns Maggot at 606.
2. **Arapaima final id 406, not 762** — c4 executed 406 (`91293a3d2`),
   verified free on every current tip AND in the final plan (405/406 sit
   in the 400-428 corridor above d37's frozen band, below d35/d40; cs
   405/406 are base-owned StardustCellBig/Small under other protocol ids
   — the 108 §2b constraint class, not a reserved hole). Accepting the
   executed id avoids a second renumber of a retired lane. CorruptGoldfish
   @405 still DROPS to d34@277, so 405 frees at merge.
3. d23's SolarFlare@408 + VileSpitEoW@411 are NEW owned kinds (free,
   collision-free); VileSpit@455 is a dup port dropping to d35@724.
4. c10's 12 roster-tail kinds are ALL dup ports (d40 owns the gold
   critters + the four d40 crawlers; c8 owns Worm) — drop list 4→16.
5. Future-lane reality: d39 pushed 7 at **781-787** and d41 pushed 13 at
   **494-506**, both collision-free with the core plan and each other —
   ACCEPTED as-pushed (R7's 817-826 guidance is superseded for these 7;
   d39's remaining 3 unclaimed kinds take 788-790). d36's E4 band
   762-789 is now HALF-SQUATTED (763-766 = d120's towns per R6, 781-787
   = d39's pushed seven; only 762 and 788-789 remain free in it) —
   **d36 re-pins to 767-780 + 791-804**
   (28 kinds, both sub-bands verified free); its `d36x` exec branch must
   absorb this before porting. g10's ELEVEN pets (637/638/656/670/
   678-684) stay cs==proto, zero clash — push.

**Count pins (SEAL, supersede R9):**

| Pin | R9 | SEAL core | Grand (d39/d41/g10 in) |
|---|---|---|---|
| owned kinds | 280 | **282** (+SolarFlare/VileSpitEoW) | **313** (+7 d39, +13 d41, +11 g10) |
| `ALL_KINDS` / `to_u16` / `from_u16` / `s1_f_3` | 550 | **552** | **583** |
| bestiary `ENTRIES` | 486 | **489** (241 base + 248 owned; all 489 positive nets DISTINCT, scripted) | **509** |
| owned kinds w/o entry | 35 | **34** (VileSpit's d23 row now counted; GoldGoldfishWalker interim entry in) | — |
| `STATIC_SHEET_NAMES` NPC rows | 455 | **463** (+6 c10 gold-critter sheets, +2 d23 NPC_516/666) | **494** (+7 d39, +13 d41, +11 g10) |
| `BESTIARY_NPC_SHEET_IDS` | 323/411 | **347 surviving / 419 target** (S3) | **≈439** |
| `SFX_CUES` NPC cues | 103 | **103** (unchanged) | **109** (+6 d41) |
| max protocol id | 766 | **766** | **787** (grand guidance max 804 with d36) |
| renumber owed | 114 | **69** (d42's 36 + c4's 1 executed, 8 E8 moves retired by the ownership flip; d35 33, g8 17, c10 4, d120 4, c8 3, c7 2, c9 2, d125 2, d23 1, e26 1 remain; 61 drops owed) | — |

Stale ALL_KINDS declarations at merge: **c9 array-missing-4 (S2 hunk,
test-breaking)**, d125 270→272, g8 270→304. d42's 313 is correct.

**Merger bible (one paragraph).** Merge in the order **d34 → d38 → d37 →
d40 → d42 → d35-renumber-last → small lanes**, per E7: the first four are
pairwise-clean keep-receipts (d34 266-280, d38 281-337, d37 354-399
zero-move, d40 430-488 minus the CrimsonGoldfish row c4's research/143
hunk retires); d42's executed 578-630 band splices in clean once its 7
dragonfly rows drop to d33's pushed superset; d35 executes its +269
renumber (429-461→698-730) LAST against the settled roster with ENTRIES
wholesale from its splice-repaired tip; then the small lanes splice with
their S1 receipt tables — c4 keeps its ten at 406/465/509/581/587/592/
607/615/688/692 and drops three, e26 keeps LadyBug/GoldLadyBug and moves
Frog→744, c8 keeps Maggot and moves three→737-739, c9 applies the S2
patch then moves two→740/741 and drops Shimmerfly, c10 moves four ducks
→731-734 and drops SIXTEEN, d23 moves AngryTrapper→743 and drops six,
d120's four towns→763-766 and d125's Empress→636/Lacewing→742 ride the
merge wave (both lanes closed), g8 moves seventeen→745-761 and drops
seventeen; every splice is guarded by the **bijection gate** — the
round-trip test (`decodable == ALL_KINDS.len()`, i16-safe) plus the
s1_f_3 coverage pin must equal **552** (core) after each lane lands and
**583** after d39/d41/g10, with the count pins **ENTRIES 489/509, sheets
463/494, slots 419/≈439, cues 103/109** re-asserted at each step; the
atlas union (494 NPC sheets) exceeds one 8192 page, so d35's research/113
lazy-load contract (NPC rows leave STATIC_SHEET_NAMES, per-world
resident+spawnable packing, texture_net_id keying) is MANDATORY at the
merge, and the S3 gap appends ride it.

### SEAL witness

`_d37_seal_census.py` (28 tips incl. local probes, bijection + ALL_KINDS
per tip), `_d37_seal_receipts.py` (per-lane consumption), `_d37_seal_
verify.py` (amended plan: 282 owned / 282 distinct ids / zero base clash
/ max 766; grand 313 / 583 / max 787 zero clash), `_d37_seal_gapfinal.py`
(347 surviving / 72 gaps / 419 target), `_d37_seal_c9patch.py` (the S2
hunk, `git apply --check` exit 0 + post-apply set-equality). No code was
changed on this branch; this seal is the deliverable.

# LV — ROSTER-LAGGARD-VERIFY receipt (supersedes S0/S1 statuses where they moved)

Status: RESEARCH / verify-receipt — zero code diff `750d8e44..HEAD` re-
verified (`git diff --stat -- crates anchors.txt` EMPTY, scripted). This
round probes the flagged lanes at CURRENT origin tips (post-fetch
14:52-14:55 local) and receipts what landed since the SEAL. Witness:
`tools/_d37_lv_receipt.py` → `_d37_lv_receipt.json` (git-show only,
direct paths, no junction recursion). The earlier d37f-dispatched
laggard-exec phase IS ALIVE and ADVANCING (untracked `_d37_lag_*` /
`_d37_e8s_*` witnesses still being written, newest 14:59; this receipt
does NOT clobber them — its own files are `_d37_lv_*`).

## LV1. Landed since the SEAL (receipts)

| Lane | SEAL tip | LV tip | Receipt |
|---|---|---|---|
| **d125** | `d8047a109` (REGRESSED) | **`7a0482302`** | **REGRESSION RESOLVED** — the 3 local commits are pushed: `merge-base --is-ancestor cd2b1b1e3 origin/...` exit 0 (sheets + draw `cd2b1b1e3` + hitbox `86e3adfd3` + BossNoCheese `7a0482302` all on origin). E8 kinds orders STILL unexecuted: EmpressOfLight 229, PrismaticLacewing 237, decl 270/rows 272 |
| **d36** | `bcf4ac551` (claim) | **`8c5839b03`** (14:28) | **R7 RE-PIN EXECUTED** — b0 five re-pinned 762-766 → **767-771** (ToxicSludge 767, Penguin 768, PenguinBlack 769, GiantTortoise 770, IceTortoise 771, scripted all-match); commit cites the d37-R7 receipt (762 reserved, 763-766 = d120 towns R6). Its batch-0 `603c967aa` at the E4 762-766 band is SUPERSEDED. NEW hover batch landed OFF the re-pin plan: **Crimera 781 / MossHornet 782 / Moth 793** — all three ids FREE in the lag-union witness (d39 moved off 781-787 to 817-823, so 781/782 are vacated; 793 sits in d36's S4 sub-band 791-804) — ACCEPT as-pushed pending the live executor's union re-derive. d36 is an ACTIVE re-pinning writer (clean tree at push time) |
| **d39** | `907234441` (781-787 OFF-PLAN) | `a0d49dc1d` | **R7 SELF-CONSUMED** — the seven re-pinned 817-823 (Butterfly 817, ScorpionBlack 818, Scorpion 819, Grasshopper 820, ChatteringTeethBomb 821, CultistArcherBlue 822, CultistArcherWhite 823, scripted) — supersedes the SEAL's 781-787 acceptance |
| **g8** | `82c93cecf` (LAGGARD) | **`1752c70df`** (14:37) | ALIVE, machine rounds landed (`e1d9578a4` RedDevil trident / `5767ca7c7` VampireBat flee + Vampire ROW / `058453096` FlyingSnake facing) + the **34-id claims-table published** (`docs/research/108-input-g8.md`, band 354-387 + Vampire 388). E8 move-table STILL unexecuted (PinkJellyfish still 356; Vampire@388 squats d37's RustyArmoredBonesAxe → 762 owed); ALL_KINDS decl STILL stale **270 / rows 305** (Vampire rowed, +1 vs SEAL's 304). Site-level re-derivation at the new tip: the live phase's `_d37_lag_g8_newtip.json` |
| **c9** | `2042d77f1` (LAGGARD) | `2042d77f1` origin / **`c830aec3d` local** (1 ahead, 8 dirty) | **S2 FIX ABSORBED LOCALLY, NOT PUSHED** — local tip "gold-critter wave 1" carries the latent ALL_KINDS fix: decl **277 / rows 277**, array-set == to_u16-set TRUE, all four snails present, s1_f_3 pin **277** (270 + snail four + GoldBird/GoldGrasshopper/GoldMouse). Origin tip still the broken 270/274. The three gold critters at **442/446/447 duplicate c10's pushed rows** (`5fcc68ae6`) — c9's drop list grows by 3 at merge (kinds-lane rule: c10 keeps, c9 drops; c9's wiring composes). E8 moves still owed: Snail 359→740, GlowingSnail 360→741, Shimmerfly drop. The S2 patch is superseded by the local commit for the LOCAL lane; origin still needs the push |
| **d42** | `2d7ac08c3` | **`bac408be0`** | research/239 FINAL (`bac408be0`) — the **11-kind keep-one drop-hunks** (7 dragonflies 595-601 + 4 waterfowl 602/603/608/609) published as EXECUTE-prep with node-verified ranges; **rows still present at tip** (scripted: all 7 dragonflies 595-601 in to_u16) — d42 keeps its rows on-branch per the E8 dropped-lane rule, the drops ride the merge wave. BloodSquid-619 hover gap CLOSED (`33b3cfe07`); research/143→238 renamed per the registry rule |
| d41 | `5d036721d` | **`db12805be`** (14:20) | batches 3+4 PUSHED: Dune Splicer chain 510-512 + desert antlions 508/509 + Tomb Crawler 513-515 + solar fragments 516/519 (ALL_KINDS 283→293). 509 GiantFlyingAntlion duplicates c4's pushed row (same id — keep c4, d41 drops); 516 SolarFlare duplicates d23's SEAL-owned kind (keep d23, d41 drops) |
| d43 | claim | **`0253bde90`** (12:35) | 63 kinds PUSHED at 631-697: 43 ACCEPT (cs==proto), 11 g10 dups + 6 grand-owned dups + 3 aliases (HallowBoss/EmpressButterfly/VileSpitEaterOfWorlds) drop — the live phase's lag-union adjudication |
| g10 | local `5dee28650` | **origin `5dee28650`** | ELEVEN pets PUSHED (637/638/656/670/678-684), cs==proto, zero clash — SEAL's push order CONSUMED |
| c4 | `0df6964be` | `0df6964be` | unchanged, retired; 3 drops still owed (BloodFeeder→d37@373, CorruptGoldfish→d34@277, FlyingFish splice) |

## LV2. The lag-union witness (live phase, cited not superseded)

The live laggard-exec phase's `_d37_lag_union.py` → `_d37_lag_union.json`
(14:32) re-derives the grand union on top of the SEAL's 282-core plan:
**core 282 (max 766) / grand 373 (max 823)** — +91 additions (d43 43,
d41 21, g10 11, d39 7 at 817-823, d36 5 at 767-771, g8 Vampire 762,
stranded Tumbleweed 788 / RuneWizard 789 / AncientCultistSquidhead 790),
20 d43 drops; ALL UNION GATES PASSED (unique ids, zero base clash,
i16-safe). d36's NEW hover batch (781/782/793) is NOT yet in that witness
but all three ids verify FREE against it (LV1) — the live executor's next
re-derive absorbs them; grand max stays 823 (d39's CultistArcherWhite).
The SEAL's "grand ~439 slots / 583 kinds" pins predate the d43/d41-b3b4
waves and are superseded by that re-derive when the live phase lands it.

## LV3. Alive-status + remaining laggards

- **d37f-laggard-exec: ALIVE** — witness writes at 14:44→14:59 continuous
  (`_d37_lag_probe2/g8new/d42df/d42win`, `_d37_e8s_netprobe/d23probe/
  vseow/basecs`); do NOT clobber; this receipt is additive.
- **d36-writer: ALIVE** — pushed 14:28, clean tree, re-pinning active
  (its foreign batch-1 WIP preserved as stash `d36-inflight-4`).
- **g8: ALIVE** — pushed 14:37 (claims-table), 1 dirty file.
- **c9: ALIVE local** — 1 unpushed commit + 8 dirty (gold-critter wave
  in flight); origin stale since 07:13.
- Still-laggard work orders (unchanged from S1 unless noted): g8 17
  moves + 18 drops (17 + Vampire renumber 388→762) + decl fix 270→305;
  c9 2 moves + 4 drops (Shimmerfly + 3 gold-critter dups) + PUSH; d120
  4 towns→763-766 (retired, merge-wave); d125 2 moves + decl 270→272
  (regression resolved, orders open); d42 11 merge-wave drops (hunks
  published); c10 4 moves + 16 drops; d35 33 moves LAST; d40 1 drop;
  c3/c6/c7/c8/d23/e26 per S1. d37 contributes ZERO moves (R8/LV
  re-verified).

### LV witness

`_d37_lv_receipt.py` (read-only git-show probes at the LV1 tips: c9
origin-vs-local ALL_KINDS census, g8 new-tip state, d36 R7 all-match +
hover-batch free-id check vs the lag-union grand set, d39 817-823
all-match, d125 ancestor check exit 0, d42 dragonfly presence, d37
zero-code-diff). No code changed on this branch.

---

# E8-RECONCILE-SEAL — the final dedupe + grand-union re-derive (SUPERSEDES the LV/lag-union counts and the SEAL S4 pins where they conflict)

Status: RESEARCH / FINAL-SEAL — zero code diff `750d8e44..HEAD` re-verified
(`git diff --stat -- crates anchors.txt` EMPTY). Author lane: W4-d37
(executor, E8-reconcile-seal phase), branch `task/W4-d37-npc-kinds-211-280`.
This round re-derives the ENTIRE roster at the CURRENT origin tips (the
wave pushed 30+ commits mid-phase — the Lead's named hashes d39-a0d49dc1 /
d41-5d036721 / d42-2d7ac08c / d43-0253bde9 / f18-a832d38e / g10-5dee2865
are all superseded except f18/g10), adjudicates every duplicate port with
source-read depth evidence, COMPOSES with the concurrent LAGGARD-EXEC
(research/250) and d43-retire (research/304) lane-published verdicts rather
than re-opening them, absorbs the SIX new claimants those rounds missed,
and seals the merger bible. Witnesses: `tools/_d37_e8s_sweep.py` (362-ref
sweep), `_d37_e8s_census.py` (34-lane census), `_d37_e8s_dups.py` (123 dup
names / 66 contested protos), `_d37_e8s_plan.py` (395-owned collision
check), `_d37_e8s_counts.py`, `_d37_e8s_acct.py`, `_d37_e8s_gaps.py`,
`_d37_e8s_emit.py` → `_d37_e8s_tables.md`. All git-show only, direct paths,
no junction recursion. Witnesses WRITTEN NOT RUN (code-only lane).

## X0. Tips read (the E8-SEAL re-fetch; the live-wave truth)

| Lane | Tip read | New kinds | Note |
|---|---|---|---|
| base | `a725acb04` | — | 270 kinds, ENTRIES 241, slots 181, NPC sheets 270, cues 55 |
| c3 | `ccb981eb3` | 3 | kinds unchanged (docs rename 247→329); WaterBoltMimic 694 canonical |
| c4 | `8abf0f1d8` | 13 | retired; 8 owned after drops |
| c6 | `5de680219` | 1 | PumpkingBlade dup |
| c7 | `c2cc6ec61` | 4 | Firefly/LightningBug/Lavafly/Shimmerfly |
| c7b | `90abc1edf` | 4 | **NEW claimant** (c7-style65-critters): the SAME four firefly kinds at the SAME ids — one canonical c7 owner, c7b drops all 4 |
| c8 | `10192602a` | 9 | retire phase 4 |
| c9 | `2267c93e8` | 9 | +5 gold-critter dups (442/443/445/446/447); ALL_KINDS fixed 279/279 |
| c10 | `15f93d134` | 20 | retire 4/4 SEALED |
| d9 | `88677f9de` | 7 | CultistDragon fam (d40 dups) + AncientCultistSquidhead 521 NEW |
| d11 | `2894f6f7d` | 1 | BloodSquid 619 dup of d42 |
| d23 | `b00e7626f` | 23 | style-14 wave +14 (12 dups of g8/d34/d35/d37 + SporeBat/QueenSlimeMinionPurple) |
| d103 | `882be0ec1` | 1 | **NEW claimant**: Wraith 360 dup of d35 |
| d120 | `2408233da` | 5 | retired, orders unexecuted |
| d125 | `5a26ae348` | 2 | **EXECUTED** EmpressOfLight 229→636, PrismaticLacewing 237→742 |
| d33 | `36aae03d0` | 11 | +4 fairy/balloon kinds (359/360/361/368) NOW WIRED (axis 112/113) |
| d34 | `f0d92564e` | 20 | **phase 4** +5 at 281-285 — SQUATS d38's anchor (new hard collision) |
| d35 | `8fa3b3dbf` | 33 | ATLAS RETIRE; renumber NOT executed (LAST by design) |
| d36 | `a11f61850` | 9 | R7 re-pin EXECUTED 767-771 + hover batch Crimera 781/MossHornet 782/Moth 793 + **Wolf 772 NEW** |
| d37 | `7bea5c835` | 46 | this lane; ZERO-MOVE (354-399 byte-identical to 750d8e44) |
| d38 | `b888a9575` | 35 | RETIRE sheet 4 SEALED |
| d39 | `048c74f8f` | 7 | R7 self-consumed 817-823 |
| d40 | `78ee5647c` | 56 | SEAL research/285: MothronEgg 478 own-entry fix + 479 drain + co-lane sheets phase (22 slots, unpushed) |
| d41 | `065efdf97` | 38 | batches 3-6: 494-537 (TombCrawler/solar/desert ghoul-lamia-poacher) |
| d42 | `adb304ab8` | 43 | executed 578-630; 11 drops owed (7 dragonfly + 4 fairy/balloon) |
| d43 | `715cf5f91` | 63 | retire research/304: 21 drops published; band 631-696 complete |
| e21 | `2fb0a36ee` | 2 | **NEW claimant**: Bunny 229 (d34 dup) + PartyBunny 237 NEW (cs 540) |
| e26 | `a23c64933` | 3 | ALL DONE (Frog 744 executed, Duck dropped) |
| f18 | `a832d38ea` | 20 | verify lane — ALL 20 are dups, drop wholesale |
| f42 | `425d4ff9d` | 9 | Tumbleweed 546 NEW + 8 dups |
| f47 | `057f44a68` | 1 | ShimmerSlime 676 (shimmer-dust wired) |
| g5 | `f044bd315` | 2 | LibrarianSkeleton 693 + RuneWizard **827** (executed re-pin) |
| g62 | `428e93ecb` | 2 | **NEW claimant**: GoblinShark 620 + Shimmerfly 677 dups |
| g8 | `dd3cb0572` | 35 | **EXECUTED** 18-move renumber 745-762 (incl. Vampire 762); 17 dups remain |
| g10 | `5dee28650` | 11 | PUSHED, cs==proto, town-pet machine wired |

Sweep totals: **362 refs, 39 data.rs-changed, 34 kind-claimants, 398
distinct new kind-names, 123 duplicate kind-names (150 extra ports), 66
contested protocol ids.** Every lane's `to_u16`↔`from_u16` bijection
verified; **zero stale ALL_KINDS declarations remain** (c9 fixed its
array, g8 fixed 305/305, d125 fixed 272/272 — the S2/R0 laggard decls are
all consumed).

## X1. DEDUPE adjudications (the Lead's five named contests + the new ones)

Ownership rule (E2/E3 stands): a kinds-lane owns over a wiring lane; among
kinds-lanes the lower band owns; **the wired implementation wins over a
data-only stub at the same vanilla type** (the R2/R3 depth rule); an
EXECUTED lane-published verdict (LAGGARD-EXEC research/250, d43-retire
research/304) is COMPOSED, not re-opened, unless a source read disproves it.

### X1a. 693-LibrarianSkeleton — d43 × g5 → **KEEP g5** (g5-caster-wired canonical: YES)

Both rows identical stats (hp 70, dmg 18, def 2, kb 0.6, value 160,
aiStyle 8, 18x40 — `NPC.cs:17725-17735`). The depth difference is the
**book shot**: g5 phase 5 (`96f074959`) ports projectile 1092
(`LIBRARIAN_BOOK`, `projectile/data.rs`), the 54-frame sheet, the
`AI_FindNearbyBook` feed (`npc_ai_bat.rs` gates on
`NpcKind::LibrarianSkeleton`), and the RuneBlast 129 timeLeft-300 arm
(`world_sim/npcs.rs`). d43's row **explicitly skips it** — its
`npc_ai_bat.rs:437-440` comment: "the LibrarianSkeleton (693) is ported,
but its book shot (projectile 1092 via AI_FindNearbyBook, NPC.cs:62308)
is a documented skip — the book-search feed is unplumbed (the kind casts
silently)". d43-retire research/304 concurs (693→g5). **DROP d43@693**
(variant + to/from arms + ALL_KINDS + ENTRIES + stat arms); d43's cs_id-693
hand row and update_caster dispatch compose onto g5's kind. g5 keeps 693
(vanilla-free, cs==proto).

### X1b. 636-HallowBoss — d43 × d125 → **KEEP d125 EmpressOfLight@636** (same-vanilla name collapse)

d43's `HallowBoss` and d125's `EmpressOfLight` are the SAME vanilla type
636 (`NPCID.cs:11949`), identical stats (hp 70000, dmg 80, def 50, kb 0,
value 250000, aiStyle 120). d125's is the **live-wired** implementation:
`ai_120_empress` dispatch (`npc.rs:5774`), `render/empress_draw.rs` (the
dedicated draw, Extra_188 halo), the BossNoCheese counter routing
(`7a0482302`), the hitbox phase. d43's `HallowBoss` dispatch is `{}` (the
`npc_ai_empress.rs` module is UNWIRED — its own doc says "the live dispatch
wiring [is] the documented [gap]"). d125 EXECUTED 229→636 (`5a26ae348`),
restoring protocol==cs. **The name collapses to EmpressOfLight** (d125's,
the wired owner); **DROP d43 HallowBoss@636** wholesale. d43's
projectile_ai_empress.rs (the five projectile AIs) composes at merge.

### X1c. 637/638 (+ the other 9 pets) — d43 × g10 → **KEEP g10** (g10-wired canonical)

d43 ports all eleven pets at cs==proto (637/638/656/670/678-684) but its
dispatch is `update_guide` (a stand-in) or `{}`. g10's are the **faithful
town-pet machine**: `ai_007_town_pet_family` + `integrate_town_pet_collision`
+ `npc_findframe_pet::town_pet_find_frame` wired at `npc.rs:7014`, the
unreachable kind arms (the style-7 axis claims them), the town flags +
exclusions. d43-retire research/304 concurs (11 pets→g10). **DROP d43's
eleven pet rows**; g10 keeps cs==proto, zero renumber. (The Lead named
637/638; the verdict extends to all eleven — d43's 656/670/678-684 are the
same contest.)

### X1d. 361/362 — e26 × c4 × c10 × f18 → resolved by the executed state

- **361 Frog**: e26 EXECUTED 361→744 (`a23c64933`); f18's Frog@243 drops
  to e26@744. c4 never ported Frog (the Lead's "c4" here is the f18
  verify-lane overlap). e26 owns Frog@744 (cs 361).
- **362 Duck**: c10 owns the Duck family (362-365→731-734 owed); e26
  DROPPED its Duck@362 (executed); f18's Duck@244/Duck2@245/DuckWhite@246/
  DuckWhite2@247 drop to c10@731-734. c4 never ported Duck.
- The 361/362 protocol ids themselves are d37's Pirate@361 /
  GoldfishWalker@362 — the small-lane kinds were ALWAYS port-local squatters
  on d37's anchor, now all shifted off (744/731-734).

### X1e. 355-380 — f18 × d39 × d42 → f18 drops wholesale, d39 keeps 817-823

f18 is a VERIFY lane that re-ported 20 already-owned kinds at 237-258
(the reserved-hole corridor). **ALL 20 f18 ports DROP** (X3 f18 table):
the 7 critters to c7/c8/c9/e26/c10, the 7 d39-band kinds to d39@817-823,
ArmedZombie to d40@430. d39's seven (Butterfly/ScorpionBlack/Scorpion/
Grasshopper/ChatteringTeethBomb/CultistArcherBlue/CultistArcherWhite) keep
817-823 (R7 self-consumed, `048c74f8f`). d42 never ported the 355-380
critters (its band is 578-630); the Lead's "d42" here is the f18×d39
overlap. f18's style modules (npc_ai_style65/41/etc.) are keep-one
candidates vs c10/d39's at merge — the style65 module: c10's
`ai_065_butterflies` (the d30 module, dust+lerp wired) vs d39/f18's
`ai_065_step` — **keep c10's** (the superset; d39/f18's are the pure
velocity variant), the Butterfly kind routes through it.

### X1f. The NEW claimants + the d34-phase-4 collision (this round's additions)

- **d34 phase 4 × d38 (281-285 HARD)**: d34's PinkJellyfish/Crab/
  DungeonGuardian/Antlion/SpikeBall at 281-285 squat d38's witness-pinned
  RaggedCaster..DiabolistRed. d38 KEEPS (anchor). d34's PinkJellyfish/
  Antlion/SpikeBall are g8 machine dups → DROP to g8@745/746/747.
  **Crab/DungeonGuardian re-pin 282→791, 283→792** (LAGGARD-EXEC award,
  both free; d36's Wolf took 772). d34 keeps 266-280 + 791/792.
- **d33 fairy/balloon × d42 (583-585/594)**: d33 NOW wires the machines
  (axis 112 `ai_112_fairy`, 113 `ai_113_balloon`); d42's rows are data-only
  `{}` stubs. **d33 owns** (wired superset, the R3 dragonfly precedent
  extends); d33 re-pins 359/360/361/368 → **583/584/585/594** (d42's
  executed vanilla-free ids, freed by its stub drop — cs==proto restored).
  **DROP d42's four stubs** (its 11-drop list = 7 dragonfly + 4 fairy/balloon).
- **d23 SporeBat/QueenSlimeMinionPurple × d43 (634/660)**: C# confirms
  both aiStyle 14 (`NPC.cs` type 634/660). d43's dispatch is
  `update_cave_bat` (faithful); d23's is a style-1/slime stand-in. BUT
  d43-retire research/304 + LAGGARD-EXEC both award d23 (the style-14 axis
  `update_cave_bat` is the keeper-fold, d23's row re-pins to the vanilla
  id). **COMPOSE: d23 owns, re-pins 407→634, 409→660; DROP d43's two
  rows.** (The axis machine is identical either way; the lane-published
  verdict stands.)
- **d23 SolarFlare × d41 (516)**: d23 has the own visible ENTRIES row
  (net 516); d41's is entry-less. C# `GetExclusions` excludes Hide=true
  ids from entry registration, BUT the populator's
  `FindEntryByNPCID(516).Info` call registers spawn-conditions — the C#
  truth is 516 HAS an entry (the Hide modifier only hides the DRAW).
  LAGGARD-EXEC awards d23. **COMPOSE: d23 keeps SolarFlare@408; DROP
  d41@516.** (d41's entry-less witness is the deviation; d23's row is
  C#-faithful.)
- **c4 GiantFlyingAntlion × d41 (509)**: LAGGARD-EXEC final awards c4
  (style44 axis + the 509 overrides). **COMPOSE: c4 keeps 509; DROP
  d41@509** (its dedicated `update_giant_flying_antlion` composes at merge).
- **c4 Pufferfish/Orca × d43 (688/692)**: d43-retire's conditional FIRED —
  d43 owns, having ABSORBED c4's just_hit puff-fix + the Orca wet-anchor
  (both swimmers now carry the fix, verified). **COMPOSE: d43 keeps
  688/692; DROP c4's two rows** (c4's style16 wiring composes).
- **c3 WaterBoltMimic × d43 (694)**: d43-retire awards c3 (the style-10
  skull lane, the 694 anchor entities.rs:966/:1018). **COMPOSE: c3 keeps
  694; DROP d43@694.**
- **e21 PartyBunny (237 NEW)**: cs 540, free in protocol space. 237 is a
  d37-owned cs hole (JungleCreeperWall). **Re-pin 237→540** (vanilla-free,
  cs==proto). e21's Bunny@229 drops to d34@270.
- **f42 Tumbleweed (354→546)**: cs 546 free; 354 squats d37's
  CochinealBeetle. **Re-pin 354→546** (vanilla-free). f42's 8 dups drop.
- **g5 RuneWizard (354→827)**: cs 172 is base-owned (DD2DarkMageT3 proto
  172) so vanilla-free impossible; g5 EXECUTED the re-pin to 827 (free,
  above d39's 823). **Accept 827 as-pushed.**
- **d9 AncientCultistSquidhead (521)**: NEW, vanilla-free, free. Keep 521.
  d9's CultistDragon fam (454-459) drops to d40.
- **d36 Wolf (772)**: NEW hover-batch kind, cs 155, 772 free. Keep 772.
- **c7b/d103/g62**: all-dup claimants (c7b's 4 fireflies = c7's; d103's
  Wraith = d35's; g62's GoblinShark = d42's, Shimmerfly = c7's). Drop
  wholesale; their wiring composes.

## X2. Grand-union recount (CURRENT tips, deduped, collision-free)

Scripted-verified (`_d37_e8s_plan.py`): **395 owned kinds, 395 distinct
final ids, ZERO id clashes, ZERO base clash, max id 827 ≪ i16::MAX
(32767) — i16-SAFE.** The `23` body's kind field is i16 (`net.rs:612`,
`to_u16() as i16`); `the_protocol_id_space_round_trips_every_kind` asserts
`id <= i16::MAX` per kind and keeps guarding.

| Pin | E8-SEAL value | Derivation |
|---|---|---|
| owned new kinds | **395** | 398 distinct names − 3 same-vanilla collapses (HallowBoss→EmpressOfLight, EmpressButterfly→PrismaticLacewing, VileSpitEoW→VileSpitEaterOfWorlds) |
| `ALL_KINDS` / `to_u16` / `from_u16` / `s1_f_3` | **665** | 270 base + 395 owned |
| bestiary `ENTRIES` rows | **583** | 241 base + 342 owned new-kind rows; **all 583 positive net ids DISTINCT** (scripted, zero dups) |
| owned kinds WITHOUT entry | **53** | the Hide folds / segments / no-entry fixtures (list in `_d37_e8s_counts.json`) |
| `STATIC_SHEET_NAMES` NPC rows | **593** | 270 base + 323 distinct new NPC_ names across the 34 tips |
| `BESTIARY_NPC_SHEET_IDS` | **440 surviving / 517 target** | 440 distinct net-covered slots as pushed; + 77 real gaps (X4) |
| `SFX_CUES` NPC hit/death cues | **117 unique** | 55 base + 62 new (naive append 165; dedupe 48) |
| max protocol id | **827** | g5 RuneWizard (grand union); i16-safe |
| renumber owed | **59 moves / 336 keeps / 153 drops** | per-lane X3 tables |

The LV/lag-union "grand 373 / max 823" and LAGGARD-EXEC "389 / max 823 /
659-577-581-432-117" pins are **superseded**: this round adds the six new
claimants (c7b/d103/e21/g62 + d33's fairy four + d36 Wolf), the d41
batch-6 desert wave, and re-derives ENTRIES/sheets/slots at the newest
tips (d40's MothronEgg entry fix, d43's 689 Hide-fold, d125's executed
636/742). The 3-name collapse (636/661/666) is the same-vanilla dedupe the
earlier rounds counted as drops — here it is explicit.

## X3. E8 per-lane old→new tables — REFRESHED (lanes execute on own branches)

The full mechanical tables (every move arm + every drop with keeper) are
in **`tools/_d37_e8s_tables.md`** (generated by `_d37_e8s_emit.py` from the
verified plan). Summary of the shifting lanes:

| Lane | Owned | Move | Drop | Renumber instruction |
|---|---|---|---|---|
| c10 | 4 | 4 | 16 | Duck family 362-365→731-734; drop 12 roster-tail + 4 waterfowl dups |
| c3 | 1 | 0 | 2 | keep WaterBoltMimic 694; drop GiantCursedSkull/CursedSkull |
| c4 | 8 | 0 | 5 | keep 406/465/509/581/587/592/607/615; drop BloodFeeder/FlyingFish/CorruptGoldfish/Pufferfish/Orca |
| c6 | 0 | 0 | 1 | drop PumpkingBlade |
| c7 | 4 | 2 | 0 | Firefly 355→735, LightningBug 358→736; keep 654/677 |
| c7b | 0 | 0 | 4 | drop all 4 (c7 canonical) |
| c8 | 4 | 3 | 5 | Worm/TruffleWorm/TruffleWormDigger 357/374/375→737-739; keep Maggot 606; drop 5 d40 dups |
| c9 | 3 | 2 | 6 | Snail/GlowingSnail 359/360→740/741; keep MagmaSnail 655; drop 5 golds + Shimmerfly |
| d9 | 1 | 0 | 6 | keep AncientCultistSquidhead 521; drop CultistDragon fam |
| d11 | 0 | 0 | 1 | drop BloodSquid |
| d23 | 4 | 3 | 19 | AngryTrapper 357→743, SporeBat 407→634, QueenSlimeMinionPurple 409→660; keep SolarFlare 408; drop 19 |
| d103 | 0 | 0 | 1 | drop Wraith |
| d120 | 4 | 4 | 1 | towns 241-244→763-766; drop BartenderUnconscious |
| d125 | 2 | 0 | 0 | KEEP (executed 636/742) |
| d33 | 11 | 4 | 0 | fairy/balloon 359/360/361/368→583/584/585/594; keep 7 dragonflies 595-601 |
| d34 | 17 | 2 | 3 | Crab 282→791, DungeonGuardian 283→792; keep 266-280; drop PinkJellyfish/Antlion/SpikeBall |
| d35 | 33 | 33 | 0 | +269 order-preserving 429-461→698-730 (LAST, E7) |
| d36 | 9 | 0 | 0 | KEEP (executed 767-771 + 781/782/793 + Wolf 772) |
| d37 | 46 | 0 | 0 | KEEP (ZERO-MOVE, R8) |
| d38 | 35 | 0 | 0 | KEEP |
| d39 | 7 | 0 | 0 | KEEP (executed 817-823) |
| d40 | 55 | 0 | 1 | drop CrimsonGoldfish (c4 impl survives @465) |
| d41 | 36 | 0 | 2 | keep 494-537 minus 509/516; drop GiantFlyingAntlion/SolarFlare |
| d42 | 32 | 0 | 11 | keep 578-630 minus 11; drop 7 dragonfly + 4 fairy/balloon |
| d43 | 42 | 0 | 21 | keep 631-696 minus 21; drop per X1 (pets/empress/aliases/693/694/654/655/676/677/634/660) |
| e21 | 1 | 1 | 1 | PartyBunny 237→540; drop Bunny |
| e26 | 3 | 0 | 0 | KEEP (executed Frog 744) |
| f18 | 0 | 0 | 20 | drop ALL 20 (verify lane) |
| f42 | 1 | 1 | 8 | Tumbleweed 354→546; drop 8 dups |
| f47 | 1 | 0 | 0 | KEEP ShimmerSlime 676 |
| g5 | 2 | 0 | 0 | KEEP (LibrarianSkeleton 693 + RuneWizard 827 executed) |
| g62 | 0 | 0 | 2 | drop GoblinShark/Shimmerfly |
| g8 | 18 | 0 | 17 | KEEP (executed 745-762); drop 17 dups |
| g10 | 11 | 0 | 0 | KEEP (cs==proto) |

**Total: 59 moves, 336 keeps, 153 drops.** Every move changes the listed
`to_u16` arms AND the mirrored `from_u16` arms + the band's witness
pins/comments; every drop removes variant + to/from arms + ALL_KINDS row +
ENTRIES row + stat arms, keeping the canonical owner's row and the dropped
lane's AI/draw/sound wiring (keyed on the canonical kind). `cs_id`,
ENTRIES `net_id`, sheets, render keys and every cs/net-keyed seam DO NOT
CHANGE (R1's seam evidence). The bijection gate +
`the_protocol_id_space_round_trips_every_kind` + s1_f_3 guard each splice.

## X4. Portrait-slot gaps (CURRENT tips, fill ownership)

`_d37_e8s_gaps.py`: 440 surviving distinct slots; **77 REAL gaps** (the
deliberate non-gaps — d33's 5 hidden dragonfly colours, d35's 6 sheet
deferrals, d43's OwlMimic 689 Hide-fold onto Owl 611 — excluded):

| Owner | Count | Kinds (canonical net id) | Fill vehicle |
|---|---|---|---|
| c4 | 7 | Arapaima 157, CrimsonGoldfish 465, GiantFlyingAntlion 509, EyeballFlyingFish 587, GoldGoldfish 592, Pupfish 607, Dolphin 615 | merge-wave append (c4 packed STYLE16/44 sheets, no grid slots) |
| c8 | 1 | TruffleWormDigger 375 | merge-wave append |
| d120 | 4 | SantaClaus 142, Truffle 160, Steampunker 178, Cyborg 209 | merge-wave append (lane retired) |
| d33 | 1 | WindyBalloon 594 | d33's fairy/balloon phase appends |
| d40 | 26 | CultistTablet 437 … GraniteFlyer 483 (the 26-list in `_d37_e8s_gaps.json`) | **d40's co-lane sheets phase** (research/285: 22 slots packed 181→203, unpushed) + merge-wave remainder |
| d41 | 17 | Crawdad 494 … SandSlime 537 (the 17-list) | d41's portraits phase / merge-wave append |
| d42 | 21 | DD2LightningBug 578 … IceMimic 629 (the 21-list) | d35's lazy-load wave (research/113 §10; d42 ruled NO-PACK 8382>8192) |

**Core slot target: 440 + 77 = 517.** d40's co-lane sheets phase (22 slots)
is unpushed — it rides the merge wave; the 26-gap list is the pre-phase
state, the phase closes 22 of them.

## X5. The FINAL merger bible (one paragraph)

Merge in the order **d34 → d38 → d37 → d40 → d42 → d35-renumber-last →
the small lanes + the new claimants**, per E7: the first four are
pairwise-clean keep-receipts (d34 266-280 + Crab/DungeonGuardian 791/792,
d38 281-337, d37 354-399 ZERO-move, d40 430-488 minus the CrimsonGoldfish
row c4's impl retires); d42's executed 578-630 band splices clean once its
11 drops (7 dragonfly → d33, 4 fairy/balloon → d33) land; d35 executes its
+269 renumber (429-461→698-730) LAST against the settled roster with
ENTRIES wholesale from its splice-repaired tip; then the small lanes
splice with their X3 tables — c4 keeps its eight at 406/465/509/581/587/
592/607/615 and drops five, c3 keeps WaterBoltMimic 694, c7/c8/c9/c10 move
their contested kinds to 735-741/731-734 and drop their dups, d23 moves
AngryTrapper→743 + SporeBat→634 + QueenSlimeMinionPurple→660 and drops
nineteen, d120's four towns→763-766 ride the merge wave (lane retired),
d125's executed 636/742 keep, g5's LibrarianSkeleton 693 + RuneWizard 827
keep, g8's executed 745-762 keep and drop seventeen, d33's fairy/balloon
re-pin to 583-585/594, d34's Crab/DungeonGuardian→791/792, e21's
PartyBunny→540, f42's Tumbleweed→546, d9's AncientCultistSquidhead 521
keeps, d36/d39/d41/e26/g10/f47 keep their executed/pushed ids, f18/c7b/
d103/g62/d11 drop wholesale; every splice is guarded by the **bijection
gate** — the round-trip test (`decodable == ALL_KINDS.len()`, i16-safe)
plus the s1_f_3 coverage pin must equal **665** after all lanes land, with
the count pins **ENTRIES 583, sheets 593, slots 517, cues 117** re-asserted
at each step; the atlas union (593 NPC sheets) exceeds one 8192 page, so
d35's research/113 lazy-load contract (NPC rows leave STATIC_SHEET_NAMES,
per-world resident+spawnable packing, texture_net_id keying) is MANDATORY
at the merge, and the X4 gap appends ride it. **Grand-union max id 827,
i16-safe; 395 owned kinds; zero collision; zero base clash.**

### E8-SEAL witness

`_d37_e8s_sweep.py` (362 refs → 34 claimants), `_d37_e8s_census.py`
(34-lane bijection + ALL_KINDS + cs_id + ENTRIES census, zero stale decls),
`_d37_e8s_dups.py` (123 dup names / 66 contested protos), `_d37_e8s_plan.py`
(395 owned / 395 distinct ids / zero clash / zero base clash / max 827
i16-safe), `_d37_e8s_counts.py` (ENTRIES 583 all-distinct-nets / sheets 593
/ slots 440 / cues 117), `_d37_e8s_acct.py` (59 moves / 336 keeps / 153
drops), `_d37_e8s_gaps.py` (77 gaps / 517 target), `_d37_e8s_emit.py` →
`_d37_e8s_tables.md` (the per-lane work orders). Depth probes:
`_d37_e8s_newprobe/d34probe/d34b/dispute/fairy/puffer/puffer2/lastprobe/
netprobe/vseow/vseow2/d23probe/d23b/d9probe/f42probe/basecs/lastprobe.py`
(git-show source reads at the cited tips). No code was changed on this
branch; this seal is the deliverable. Witnesses WRITTEN NOT RUN.

