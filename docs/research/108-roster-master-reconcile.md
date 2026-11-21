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
