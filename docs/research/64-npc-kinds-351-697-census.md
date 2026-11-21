# 64 — NPC kinds 351–697 `SetDefaults` census + first-20 port (W4-f18)

**Status: SEALED** (see the SEAL section at the end — the roster-collision
adjudication + the kinds-verify-2 family bible; code-only, witnesses WRITTEN
NOT RUN). Registry-150 FROZEN §9 voided the round-1 `64→249` rename (249=d15);
the doc keeps the `64-` filename pending a Lead §9 assignment ≥367.

Census of the vanilla `NPC.SetDefaults` type arms
(`decompiled/Terraria/NPC.cs`, the `else if (this.type == N)` chain)
against the port's kind roster, for the 347 ids 351–697, plus the
verbatim port of the first 20 missing arm-bearing kinds. Produced by
`task/W4-f18-npc-kinds-verify-2`; the d39–d43 co-lanes read the missing
list below as their remaining porting scope.

## Method

- **C# side:** every arm of the `SetDefaults` chain covering an id in
  351–697 was extracted mechanically, compound conditions
  (`this.type == 366 || this.type == 367`, `this.type >= 379 &&
  this.type <= 380`) evaluated per id: **343 arm-bearing ids**; the
  four ids without an arm are listed below.
- **Rust side (ported = has a kind):** the kind↔vanilla-type mapping
  rides the `cs_id` hand rows (`npc/data.rs:2992`), then the bestiary
  `net_id` rows (`bestiary.rs`), then the `render_key` type map
  (`npc.rs:18259`) — NOT `to_u16`/`from_u16`, which are the port-local
  protocol ids (gate-reassigned). A kind is "ported" when any of the
  three resolves its vanilla type into 351–697.
- **Effective-C# semantics:** `knockBackResist` class default **1f**
  (`NPC.cs:8533`) when the arm writes no row; `value` stock **0f**
  (`NPC.cs:8544`); `noGravity` stock false (`NPC.cs:8535`). The port's
  `knock_back_resist()` catch-all is 0.4, so every ported kind needs an
  explicit row; `value()` is an exhaustive match, so a no-row kind
  needs an explicit `0.0` arm.
- **Static analysis only** (no cargo per task gate): the witness tests
  are WRITTEN, NOT RUN — compiled/pinned for the next cargo-enabled pass.

## Scoreboard

The baseline (before this branch) and the post-port counts. This branch
lands the first 20 arm-bearing missing kinds (the 355–380 band + 430),
all in the d39 range except 430 (d40).

| wave | range | ported (before) | missing (before) | landed here | ported (after) | missing (after) | total |
| --- | --- | --- | --- | --- | --- | --- | --- |
| d39 | 351–420 | 48 | 22 | 19 | 67 | 3 | 70 |
| d40 | 421–490 | 14 | 56 | 1 | 15 | 55 | 70 |
| d41 | 491–560 | 21 | 49 | 0 | 21 | 49 | 70 |
| d42 | 561–630 | 19 | 51 | 0 | 19 | 51 | 70 |
| d43 | 631–697 | 3 | 64 | 0 | 3 | 64 | 67 |
| **all** | **351–697** | **105** | **242** | **20** | **125** | **222** | **347** |

Of the 242 baseline-missing, **238 carry a `SetDefaults` arm** (the waves'
porting scope) and **4 have none** (below). This pass ports the first
20 arm-bearing missing kinds (355–380 band + 430), leaving **218
arm-bearing kinds** for the waves. The d39 range's 3 remaining missing
are the no-arm ids 403/404/408 (the folded StardustWorm segments + the
unarmed JellyfishSmall).

### The four ids without a `SetDefaults` arm

- **403/404** (`StardustWormBody`/`Tail`): no arm — the segments ride
  the head's `realLife` pool; the port models the family through
  `StardustWormHead` (402) like every folded worm family.
- **408** (`StardustJellyfishSmall`): no arm in the chain; unported.
- **697**: not an NPC type — `NPCID.Count = 697` (`NPCID.cs:12581`);
  the last real kind is 696.

## First-20 port (this branch)

The first 20 arm-bearing missing kinds, ported verbatim with their
`SetDefaults` stat rows, protocol ids, bestiary entries, FindFrame arms
and AI routing. Each carries a stat witness in
`crates/tr-app/src/npc_w4_f18_witnesses.rs` (WRITTEN, NOT RUN).

| type | kind | C# arm | aiStyle | hp | dmg | def | value | kb | size |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 355 | Firefly | 13517-13530 | 64 | 5 | 0 | 0 | 0 | 1 | 10×10 |
| 356 | Butterfly | 13531-13544 | 65 | 5 | 0 | 0 | 0 | 1 | 10×10 |
| 357 | Worm | 13545-13558 | 66 | 5 | 0 | 0 | 0 | 1 | 10×4 |
| 358 | LightningBug | 13559-13572 | 64 | 5 | 0 | 0 | 0 | 1 | 12×12 |
| 359 | Snail | 13573-13586 | 67 | 5 | 0 | 0 | 0 | 1 | 12×12 |
| 360 | GlowingSnail | 13587-13600 | 67 | 5 | 0 | 0 | 0 | 1 | 14×14 |
| 361 | Frog | 13601-13612 | 7 | 5 | 0 | 0 | 0 | 1 | 12×10 |
| 362 | Duck | 13613-13624 | 7 | 5 | 0 | 0 | 0 | 1 | 22×26 |
| 363 | Duck2 | 13625-13636 | 68 | 5 | 0 | 0 | 0 | 1 | 28×22 |
| 364 | DuckWhite | 13637-13648 | 7 | 5 | 0 | 0 | 0 | 1 | 22×26 |
| 365 | DuckWhite2 | 13649-13660 | 68 | 5 | 0 | 0 | 0 | 1 | 28×22 |
| 366 | ScorpionBlack | 13661-13672 | 7 | 5 | 0 | 0 | 0 | 1 | 26×18 |
| 367 | Scorpion | 13661-13672 | 7 | 5 | 0 | 0 | 0 | 1 | 26×18 |
| 374 | TruffleWorm | 13762-13776 | 66 | 5 | 0 | 0 | 0 | 1 | 20×10 |
| 375 | TruffleWormDigger | 13777-13795 | 6 | 5 | 0 | 0 | 0 | 0 | 10×10 |
| 377 | Grasshopper | 13811-13824 | 1 | 5 | 0 | 0 | 0 | 1 | 14×10 |
| 378 | ChatteringTeethBomb | 13825-13838 | 41 | 200 | 120 | 30 | 450 | 0.5 | 20×26 |
| 379 | CultistArcherBlue | 13839-13855 | 3 | 210 | 45 | 14 | 1000 | 0.55 | 18×40 |
| 380 | CultistArcherWhite | 13839-13855 | 3 | 210 | 45 | 14 | 1000 | 0.55 | 18×40 |
| 430 | ArmedZombie | 8612-8624 | 3 | 45 | 14 | 6 | 60 | 0.5 | 18×40 |

`kb` = effective C# (class default 1f when the arm writes none). The
port's `no_gravity()` is the strike-time flag (C# noGravity AND kb>0,
W3-i11): 355/356/358/359/360 (noGravity, kb 1) read **true**; 375
(noGravity, kb 0) reads **false** BY DESIGN (the Y-pop selector never
reaches a kb-0 kind), not a gap.

### Landing details (the four batch commits)

The twenty kinds landed in four commits of five (the task gate's
5-kinds-per-commit cadence), each wiring EVERY exhaustive `NpcKind`
match (the stat tables, the protocol-id bijection, the bestiary, the
FindFrame arms, the draw + the AI routing):

- **Batch 1 (355–359):** the Firefly/Butterfly/Worm/LightningBug/Snail.
  The new `npc_ai_style65.rs` ports `AI_065_Butterflies`
  (`NPC.cs:44896-45119`, the colour roll / scale latch / rearm / threat
  flee); the style-axis claims 64/65/66/67 ride the existing
  `npc_ai_style64/66/67` modules. Protocol ids 237–241.
- **Batch 2 (360–364):** the GlowingSnail/Frog/Duck/Duck2/DuckWhite.
  The new `npc_ai_town_critter.rs` ports the `AI_007_TownEntities`
  TownCritter subset (`NPC.cs:52965-55561`, the idle/walk/obstacle-jump
  machine, reusing `npc_ai_town.rs`'s shared helpers); the Duck2's
  aiStyle-68 arm (`npc_ai_style68`) queues the dry-landing
  `Transform(362)` on the sim's `wants_transform` drain. The Duck2 363
  folds its bestiary credit onto the Duck 362 (Hide=true). Protocol ids
  242–246.
- **Batch 3 (365–375):** the DuckWhite2/ScorpionBlack/Scorpion/
  TruffleWorm/TruffleWormDigger. The TruffleWorm's aiStyle-66 dig tail
  queues `Transform(375)`; the TruffleWormDigger 375 rides a focused
  `AI_006` lone-head machine (the INVERTED dig aim, `NPC.cs:51861-51865`,
  the 6/0.15 tunables `:51730`, the 1000-px dig list `:51598`) —
  `is_worm()` true but NOT `is_worm_head()` (the StardustWormHead
  lone-minion precedent, no chain). The DuckWhite2 365 / TruffleWormDigger
  375 fold their credits (Hide=true). Protocol ids 247–249, 252–253
  (250/288 are the AngryNimbus/DungeonSpirit's port-local ids).
- **Batch 4 (377–430):** the Grasshopper/ChatteringTeethBomb/
  CultistArcherBlue/CultistArcherWhite/ArmedZombie. The Grasshopper rides
  the faithful `npc_ai_slime::ai_001_slimes` (the 377 flee/friendly-latch/
  ceiling-clamp rows); the ChatteringTeethBomb rides
  `npc_ai_style41::style41_step` (self-integrating); the cultist archers
  ride a focused `AI_003` shooter block (the 80-tick cooldown, the 81
  arrow / 40 dmg / 11 speed, the ±40 jitter NO rescale, the |dx|*0.1 aim
  lift NOT zeroed, the centre spawn NO +6 y — the 381/382-only terms);
  the ArmedZombie rides the aiStyle-3 charge family
  (`npc_ai3_speeds::apply_ladder`'s 430-436 arm) via `walker_step_fighter`.
  The CultistArcherWhite 380 folds its credit onto 379 (Hide=true);
  379/380 carry `chaseable = false`. Protocol ids 254–258.

The folded kinds (363/365/375/380) keep their OWN render key + draw
sheet (`texture_net_id`); only the bestiary credit folds. The
`npc_w4_f18_witnesses.rs` file pins each kind's full stat row + the
roster-level round-trip/frame-strip invariants (WRITTEN, NOT RUN).

### Documented remainders (the wiring follow-ups)

- The town-critter / butterfly / firefly threat scans read the sibling
  NPC pool in the C#; the solo `update_rng` path passes an empty pool
  (the `npc_ai_style65` module doc's deviation — the sim's pooled seam
  lands with the wiring follow-up).
- The cultist archers' `ai[3] < 0` retreat block (`NPC.cs:56181-56220`,
  the sibling `Main.npc[num10]` cultist-leader link) is unported (the
  port has no leader-link seam).
- The TruffleWormDigger's crumble dust (`NPC.cs:51590`) rides the dust
  seam (the worm pass's documented deviation); the dig roar rides the
  sound seam.
- The Worm 357's FindFrame slope tail (`NPC.cs:72807-72826`, the ±0.785
  slope rotation) reads the tile under the centre — the port's
  `find_frame` takes no `&World` (the signature seam), so the no-slope
  path is byte-identical and the slope rotation is a documented
  remainder (the world-read migration lands with the FindFrame wave).

## Findings repaired on this branch (npc.rs `value()`)

1. **402 StardustWormHead value 700→0** — the C# arm writes
   `value = 0f` (`NPC.cs:15018`); the 700.0 row had no source.
2. **Moon Lord kb group spliced into `value()`** — the five-part
   `=> 0.0` group (a `knock_back_resist` row) shadowed the core's
   `1_000_000.0` arm (`NPC.cs:14130`). Purged; the real kb rows live in
   `knock_back_resist()`.
3. **Six shard kb rows spliced into `value()`** — 405/406/407/423/
   425/429 returned their kb (0.3/0.4/0.03/0.2/0.4/0.6) as coin value;
   the C# arms write no `value` row. Replaced with explicit `0.0` arms
   (the match is exhaustive — purging outright would not compile).
4. **Duplicate-arm dedupes (keep-one, values unchanged):** the 6×
   repeat of the SolarCrawltipede×3/StardustWormHead/SolarSpearman
   block, and the second `DD2LanePortal => 0.0` arm.

## Findings recorded, NOT repaired (out of scope)

- **Pillar-mob coin values:** the port's `value()` rows for the
  Stardust/Solar/Nebula/Vortex mobs (409–428) have **no C# source** —
  every one of those arms writes no `value` row (stock 0f,
  `NPC.cs:8544`). A coin-drop rebalance needs its own task.
- **Same-value shadowed duplicates** (compile warnings, no behaviour
  change): `DD2LanePortal` ×3, `VortexTower` ×3, the frost trio ×2,
  `EyeOfCthulhu` ×2 — all inside `value()`.

## Ported kinds (105) — the kind↔vanilla-type resolution

| type | port kind(s) | NPCID name |
| --- | --- | --- |
| 351 | Krampus | Krampus |
| 352 | Flocko | Flocko |
| 353 | Stylist | Stylist |
| 354 | WebbedStylist | WebbedStylist |
| 368 | TravellingMerchant | TravellingMerchant |
| 369 | Angler | Angler |
| 370 | DukeFishron | DukeFishron |
| 371 | DetonatingBubble | DetonatingBubble |
| 372 | Sharkron | Sharkron |
| 373 | Sharkron2 | Sharkron2 |
| 376 | SleepingAngler | SleepingAngler |
| 381 | BrainScrambler | BrainScrambler |
| 382 | RayGunner | RayGunner |
| 383 | MartianOfficer | MartianOfficer |
| 384 | ForceBubble | ForceBubble |
| 385 | GrayGrunt | GrayGrunt |
| 386 | MartianEngineer | MartianEngineer |
| 387 | MartianTurret | MartianTurret |
| 388 | MartianDrone | MartianDrone |
| 389 | GigaZapper | GigaZapper |
| 390 | ScutlixRider | ScutlixRider |
| 391 | Scutlix | Scutlix |
| 392 | MartianSaucer | MartianSaucer |
| 393 | MartianSaucerTurret | MartianSaucerTurret |
| 394 | MartianSaucerCannon | MartianSaucerCannon |
| 395 | MartianSaucerCore | MartianSaucerCore |
| 396 | MoonLordHead | MoonLordHead |
| 397 | MoonLordHand | MoonLordHand |
| 398 | MoonLordCore | MoonLordCore |
| 399 | MartianProbe | MartianProbe |
| 400 | MoonLordFreeEye | MoonLordFreeEye |
| 401 | MoonLordLeechBlob | MoonLordLeechBlob |
| 402 | StardustWormHead | StardustWormHead |
| 405 | StardustCellBig | StardustCellBig |
| 406 | StardustCellSmall | StardustCellSmall |
| 407 | StardustJellyfishBig | StardustJellyfishBig |
| 409 | StardustSpiderBig | StardustSpiderBig |
| 410 | StardustSpiderSmall | StardustSpiderSmall |
| 411 | StardustSoldier | StardustSoldier |
| 412 | SolarCrawltipedeHead | SolarCrawltipedeHead |
| 413 | SolarCrawltipedeBody | SolarCrawltipedeBody |
| 414 | SolarCrawltipedeTail | SolarCrawltipedeTail |
| 415 | SolarDrakomire | SolarDrakomire |
| 416 | SolarDrakomireRider | SolarDrakomireRider |
| 417 | SolarSroller | SolarSroller |
| 418 | SolarCorite | SolarCorite |
| 419 | SolarSelenian | SolarSolenian |
| 420 | NebulaFloater | NebulaBrain |
| 421 | BrainSuckler | NebulaHeadcrab |
| 422 | VortexTower | LunarTowerVortex |
| 423 | NebulaBeast | NebulaBeast |
| 424 | Predictor | NebulaSoldier |
| 425 | VortexRifleman | VortexRifleman |
| 426 | VortexHunter | VortexHornetQueen |
| 427 | VortexAlien | VortexHornet |
| 428 | VortexHeadcrab | VortexLarva |
| 429 | VortexSoldier | VortexSoldier |
| 439 | AncientCultist | CultistBoss |
| 441 | TaxCollector | TaxCollector |
| 471 | GoblinSummoner | GoblinSummoner |
| 489 | BloodZombie | BloodZombie |
| 490 | Drippler | Drippler |
| 491 | FlyingDutchman | PirateShip |
| 492 | FlyingDutchmanCannon | PirateShipCannon |
| 493 | StardustTower | LunarTowerStardust |
| 507 | NebulaTower | LunarTowerNebula |
| 517 | SolarTower | LunarTowerSolar |
| 518 | SolarSpearman | SolarSpearman |
| 520 | MartianWalker | MartianWalker |
| 533 | DesertDjinn | DesertDjinn |
| 548 | DD2EterniaCrystal | DD2EterniaCrystal |
| 549 | DD2LanePortal | DD2LanePortal |
| 550 | DD2Bartender | DD2Bartender |
| 551 | DD2Betsy | DD2Betsy |
| 552 | DD2GoblinT1 | DD2GoblinT1 |
| 553 | DD2GoblinT2 | DD2GoblinT2 |
| 554 | DD2GoblinT3 | DD2GoblinT3 |
| 555 | DD2GoblinBomberT1 | DD2GoblinBomberT1 |
| 556 | DD2GoblinBomberT2 | DD2GoblinBomberT2 |
| 557 | DD2GoblinBomberT3 | DD2GoblinBomberT3 |
| 558 | DD2WyvernT1 | DD2WyvernT1 |
| 559 | DD2WyvernT2 | DD2WyvernT2 |
| 560 | DD2WyvernT3 | DD2WyvernT3 |
| 561 | DD2JavelinT1 | DD2JavelinstT1 |
| 562 | DD2JavelinT2 | DD2JavelinstT2 |
| 563 | DD2JavelinT3 | DD2JavelinstT3 |
| 564 | DD2DarkMage | DD2DarkMageT1 |
| 565 | DD2DarkMageT3 | DD2DarkMageT3 |
| 566 | DD2SkeletonT1 | DD2SkeletonT1 |
| 567 | DD2SkeletonT3 | DD2SkeletonT3 |
| 568 | DD2WitherBeast | DD2WitherBeastT2 |
| 569 | DD2WitherBeastT3 | DD2WitherBeastT3 |
| 570 | DD2DrakinT2 | DD2DrakinT2 |
| 571 | DD2DrakinT3 | DD2DrakinT3 |
| 572 | DD2KoboldWalkerT2 | DD2KoboldWalkerT2 |
| 573 | DD2KoboldWalkerT3 | DD2KoboldWalkerT3 |
| 574 | DD2KoboldFlyerT2 | DD2KoboldFlyerT2 |
| 575 | DD2KoboldFlyerT3 | DD2KoboldFlyerT3 |
| 576 | DD2OgreT2 | DD2OgreT2 |
| 577 | DD2OgreT3 | DD2OgreT3 |
| 588 | Golfer | Golfer |
| 589 | SleepingGolfer | GolferRescue |
| 633 | Zoologist | BestiaryGirl |
| 663 | Princess | Princess |
| 668 | Deerclops | Deerclops |


## Unported kinds — the waves' remaining scope (222 after this branch)

`arm` = the C# `SetDefaults` line range the wave must port; `—` = no
arm (see above). The first 20 arm-bearing rows (355–380 + 430) are
**ported by THIS branch** (marked ✓ below — kept in the table so the
d39–d43 co-lanes see the full baseline census and the exact landing
boundary); the rest are the d39–d43 waves' scope.

| type | NPCID name | C# arm | aiStyle | wave |
| --- | --- | --- | --- | --- |
| 355 | Firefly | 13517-13530 | 64 | ✓ W4-f18 |
| 356 | Butterfly | 13531-13544 | 65 | ✓ W4-f18 |
| 357 | Worm | 13545-13558 | 66 | ✓ W4-f18 |
| 358 | LightningBug | 13559-13572 | 64 | ✓ W4-f18 |
| 359 | Snail | 13573-13586 | 67 | ✓ W4-f18 |
| 360 | GlowingSnail | 13587-13600 | 67 | ✓ W4-f18 |
| 361 | Frog | 13601-13612 | 7 | ✓ W4-f18 |
| 362 | Duck | 13613-13624 | 7 | ✓ W4-f18 |
| 363 | Duck2 | 13625-13636 | 68 | ✓ W4-f18 |
| 364 | DuckWhite | 13637-13648 | 7 | ✓ W4-f18 |
| 365 | DuckWhite2 | 13649-13660 | 68 | ✓ W4-f18 |
| 366 | ScorpionBlack | 13661-13672 | 7 | ✓ W4-f18 |
| 367 | Scorpion | 13661-13672 | 7 | ✓ W4-f18 |
| 374 | TruffleWorm | 13762-13776 | 66 | ✓ W4-f18 |
| 375 | TruffleWormDigger | 13777-13795 | 6 | ✓ W4-f18 |
| 377 | Grasshopper | 13811-13824 | 1 | ✓ W4-f18 |
| 378 | ChatteringTeethBomb | 13825-13838 | 41 | ✓ W4-f18 |
| 379 | CultistArcherBlue | 13839-13855 | 3 | ✓ W4-f18 |
| 380 | CultistArcherWhite | 13839-13855 | 3 | ✓ W4-f18 |
| 403 | StardustWormBody | — | — | d39 |
| 404 | StardustWormTail | — | — | d39 |
| 408 | StardustJellyfishSmall | — | — | d39 |
| 430 | ArmedZombie | 8612-8624 | 3 | ✓ W4-f18 |
| 431 | ArmedZombieEskimo | 8625-8637 | 3 | d40 |
| 432 | ArmedZombiePincussion | 8638-8650 | 3 | d40 |
| 433 | ArmedZombieSlimed | 8651-8663 | 3 | d40 |
| 434 | ArmedZombieSwamp | 8664-8676 | 3 | d40 |
| 435 | ArmedZombieTwiggy | 8677-8689 | 3 | d40 |
| 436 | ArmedZombieCenx | 8690-8702 | 3 | d40 |
| 437 | CultistTablet | 14176-14194 | 83 | d40 |
| 438 | CultistDevote | 14195-14211 | 83 | d40 |
| 440 | CultistBossClone | 14230-14246 | 84 | d40 |
| 442 | GoldBird | 14247-14261 | 24 | d40 |
| 443 | GoldBunny | 14262-14274 | 7 | d40 |
| 444 | GoldButterfly | 14275-14289 | 65 | d40 |
| 445 | GoldFrog | 14290-14302 | 7 | d40 |
| 446 | GoldGrasshopper | 14303-14317 | 1 | d40 |
| 447 | GoldMouse | 14318-14331 | 7 | d40 |
| 448 | GoldWorm | 14332-14346 | 66 | d40 |
| 449 | BoneThrowingSkeleton | 14347-14359 | 3 | d40 |
| 450 | BoneThrowingSkeleton2 | 14360-14372 | 3 | d40 |
| 451 | BoneThrowingSkeleton3 | 14373-14385 | 3 | d40 |
| 452 | BoneThrowingSkeleton4 | 14386-14398 | 3 | d40 |
| 453 | SkeletonMerchant | 14399-14413 | 7 | d40 |
| 454 | CultistDragonHead | 14414-14432 | 6 | d40 |
| 455 | CultistDragonBody1 | 14452-14470 | 6 | d40 |
| 456 | CultistDragonBody2 | 14452-14470 | 6 | d40 |
| 457 | CultistDragonBody3 | 14452-14470 | 6 | d40 |
| 458 | CultistDragonBody4 | 14452-14470 | 6 | d40 |
| 459 | CultistDragonTail | 14433-14451 | 6 | d40 |
| 460 | Butcher | 15494-15506 | 3 | d40 |
| 461 | CreatureFromTheDeep | 15507-15519 | 3 | d40 |
| 462 | Fritz | 15520-15533 | 3 | d40 |
| 463 | Nailhead | 15534-15546 | 3 | d40 |
| 464 | CrimsonBunny | 14471-14482 | 3 | d40 |
| 465 | CrimsonGoldfish | 14483-14495 | 16 | d40 |
| 466 | Psycho | 15547-15559 | 3 | d40 |
| 467 | DeadlySphere | 15560-15573 | 85 | d40 |
| 468 | DrManFly | 15587-15599 | 3 | d40 |
| 469 | ThePossessed | 15574-15586 | 3 | d40 |
| 470 | CrimsonPenguin | 14496-14507 | 3 | d40 |
| 472 | ShadowFlameApparition | 15742-15755 | 86 | d40 |
| 473 | BigMimicCorruption | 14508-14521 | 87 | d40 |
| 474 | BigMimicCrimson | 14508-14521 | 87 | d40 |
| 475 | BigMimicHallow | 14508-14521 | 87 | d40 |
| 476 | BigMimicJungle | 14508-14521 | 87 | d40 |
| 477 | Mothron | 15600-15615 | 88 | d40 |
| 478 | MothronEgg | 15616-15629 | 89 | d40 |
| 479 | MothronSpawn | 15630-15643 | 90 | d40 |
| 480 | Medusa | 14522-14535 | 3 | d40 |
| 481 | GreekSkeleton | 14536-14548 | 3 | d40 |
| 482 | GraniteGolem | 14549-14561 | 3 | d40 |
| 483 | GraniteFlyer | 14562-14575 | 91 | d40 |
| 484 | EnchantedNightcrawler | 14576-14589 | 66 | d40 |
| 485 | Grubby | 14576-14589 | 66 | d40 |
| 486 | Sluggy | 14576-14589 | 66 | d40 |
| 487 | Buggy | 14576-14589 | 66 | d40 |
| 488 | TargetDummy | 14590-14604 | 92 | d40 |
| 494 | Crawdad | 14666-14677 | 3 | d41 |
| 495 | Crawdad2 | 14666-14677 | 3 | d41 |
| 496 | GiantShelly | 14678-14690 | 39 | d41 |
| 497 | GiantShelly2 | 14678-14690 | 39 | d41 |
| 498 | Salamander | 14691-14703 | 3 | d41 |
| 499 | Salamander2 | 14691-14703 | 3 | d41 |
| 500 | Salamander3 | 14691-14703 | 3 | d41 |
| 501 | Salamander4 | 14691-14703 | 3 | d41 |
| 502 | Salamander5 | 14691-14703 | 3 | d41 |
| 503 | Salamander6 | 14691-14703 | 3 | d41 |
| 504 | Salamander7 | 14691-14703 | 3 | d41 |
| 505 | Salamander8 | 14691-14703 | 3 | d41 |
| 506 | Salamander9 | 14691-14703 | 3 | d41 |
| 508 | GiantWalkingAntlion | 14828-14840 | 3 | d41 |
| 509 | GiantFlyingAntlion | 14841-14853 | 44 | d41 |
| 510 | DuneSplicerHead | 14773-14789 | 6 | d41 |
| 511 | DuneSplicerBody | 14790-14808 | 6 | d41 |
| 512 | DuneSplicerTail | 14809-14827 | 6 | d41 |
| 513 | TombCrawlerHead | 14718-14734 | 6 | d41 |
| 514 | TombCrawlerBody | 14735-14753 | 6 | d41 |
| 515 | TombCrawlerTail | 14754-14772 | 6 | d41 |
| 516 | SolarFlare | 15394-15408 | 9 | d41 |
| 519 | SolarGoop | 15409-15423 | 99 | d41 |
| 521 | AncientCultistSquidhead | 15768-15779 | 86 | d41 |
| 522 | AncientLight | 15780-15794 | 100 | d41 |
| 523 | AncientDoom | 15795-15812 | 101 | d41 |
| 524 | DesertGhoul | 14882-14919 | 3 | d41 |
| 525 | DesertGhoulCorruption | 14882-14919 | 3 | d41 |
| 526 | DesertGhoulCrimson | 14882-14919 | 3 | d41 |
| 527 | DesertGhoulHallow | 14882-14919 | 3 | d41 |
| 528 | DesertLamiaLight | 14920-14932 | 3 | d41 |
| 529 | DesertLamiaDark | 14920-14932 | 3 | d41 |
| 530 | DesertScorpionWalk | 14933-14945 | 3 | d41 |
| 531 | DesertScorpionWall | 14946-14959 | 40 | d41 |
| 532 | DesertBeast | 14960-14973 | 3 | d41 |
| 534 | DemonTaxCollector | 15813-15828 | 3 | d41 |
| 535 | SlimeSpiked | 15829-15842 | 1 | d41 |
| 536 | TheBride | 15843-15856 | 3 | d41 |
| 537 | SandSlime | 15857-15872 | 1 | d41 |
| 538 | SquirrelRed | 15873-15884 | 7 | d41 |
| 539 | SquirrelGold | 15885-15897 | 7 | d41 |
| 540 | PartyBunny | 9336-9347 | 7 | d41 |
| 541 | SandElemental | 15898-15911 | 102 | d41 |
| 542 | SandShark | 15912-15926 | 103 | d41 |
| 543 | SandsharkCorrupt | 15927-15941 | 103 | d41 |
| 544 | SandsharkCrimson | 15942-15956 | 103 | d41 |
| 545 | SandsharkHallow | 15957-15971 | 103 | d41 |
| 546 | Tumbleweed | 15972-15984 | 26 | d41 |
| 547 | DD2AttackerTest | 15985-15997 | 104 | d41 |
| 578 | DD2LightningBugT3 | 16536-16553 | 111 | d42 |
| 579 | BartenderUnconscious | 16554-16568 | 0 | d42 |
| 580 | WalkingAntlion | 14854-14867 | 3 | d42 |
| 581 | FlyingAntlion | 14868-14881 | 44 | d42 |
| 582 | LarvaeAntlion | 16569-16582 | 3 | d42 |
| 583 | FairyCritterPink | 16583-16607 | 112 | d42 |
| 584 | FairyCritterGreen | 16583-16607 | 112 | d42 |
| 585 | FairyCritterBlue | 16583-16607 | 112 | d42 |
| 586 | ZombieMerman | 16608-16622 | 3 | d42 |
| 587 | EyeballFlyingFish | 16623-16637 | 44 | d42 |
| 590 | TorchZombie | 16666-16678 | 3 | d42 |
| 591 | ArmedTorchZombie | 16666-16678 | 3 | d42 |
| 592 | GoldGoldfish | 16679-16693 | 16 | d42 |
| 593 | GoldGoldfishWalker | 16694-16706 | 7 | d42 |
| 594 | WindyBalloon | 16707-16719 | 113 | d42 |
| 595 | BlackDragonfly | 16720-16738 | 114 | d42 |
| 596 | BlueDragonfly | 16720-16738 | 114 | d42 |
| 597 | GreenDragonfly | 16720-16738 | 114 | d42 |
| 598 | OrangeDragonfly | 16720-16738 | 114 | d42 |
| 599 | RedDragonfly | 16720-16738 | 114 | d42 |
| 600 | YellowDragonfly | 16720-16738 | 114 | d42 |
| 601 | GoldDragonfly | 16720-16738 | 114 | d42 |
| 602 | Seagull | 16739-16750 | 7 | d42 |
| 603 | Seagull2 | 16751-16762 | 68 | d42 |
| 604 | LadyBug | 16763-16780 | 115 | d42 |
| 605 | GoldLadyBug | 16763-16780 | 115 | d42 |
| 606 | Maggot | 16781-16794 | 66 | d42 |
| 607 | Pupfish | 16795-16808 | 16 | d42 |
| 608 | Grebe | 16809-16820 | 7 | d42 |
| 609 | Grebe2 | 16821-16832 | 68 | d42 |
| 610 | Rat | 16833-16845 | 7 | d42 |
| 611 | Owl | 16846-16859 | 24 | d42 |
| 612 | WaterStrider | 16860-16879 | 116 | d42 |
| 613 | GoldWaterStrider | 16860-16879 | 116 | d42 |
| 614 | ExplosiveBunny | 16880-16891 | 7 | d42 |
| 615 | Dolphin | 16892-16904 | 16 | d42 |
| 616 | Turtle | 16905-16917 | 7 | d42 |
| 617 | TurtleJungle | 16905-16917 | 7 | d42 |
| 618 | BloodNautilus | 16918-16933 | 117 | d42 |
| 619 | BloodSquid | 16934-16949 | 5 | d42 |
| 620 | GoblinShark | 16950-16968 | 107 | d42 |
| 621 | BloodEelHead | 16969-16987 | 6 | d42 |
| 622 | BloodEelBody | 16988-17006 | 6 | d42 |
| 623 | BloodEelTail | 17007-17025 | 6 | d42 |
| 624 | Gnome | 17026-17038 | 3 | d42 |
| 625 | SeaTurtle | 17039-17050 | 7 | d42 |
| 626 | Seahorse | 17051-17071 | 118 | d42 |
| 627 | GoldSeahorse | 17051-17071 | 118 | d42 |
| 628 | Dandelion | 17072-17084 | 119 | d42 |
| 629 | IceMimic | 17085-17106 | 25 | d42 |
| 630 | BloodMummy | 17107-17119 | 3 | d42 |
| 631 | RockGolem | 17120-17133 | 3 | d43 |
| 632 | MaggotZombie | 17134-17146 | 3 | d43 |
| 634 | SporeBat | 17161-17174 | 14 | d43 |
| 635 | SporeSkeleton | 17175-17187 | 3 | d43 |
| 636 | HallowBoss | 17188-17206 | 120 | d43 |
| 637 | TownCat | 17207-17225 | 7 | d43 |
| 638 | TownDog | 17207-17225 | 7 | d43 |
| 639 | GemSquirrelAmethyst | 17226-17237 | 7 | d43 |
| 640 | GemSquirrelTopaz | 17226-17237 | 7 | d43 |
| 641 | GemSquirrelSapphire | 17226-17237 | 7 | d43 |
| 642 | GemSquirrelEmerald | 17226-17237 | 7 | d43 |
| 643 | GemSquirrelRuby | 17226-17237 | 7 | d43 |
| 644 | GemSquirrelDiamond | 17226-17237 | 7 | d43 |
| 645 | GemSquirrelAmber | 17226-17237 | 7 | d43 |
| 646 | GemBunnyAmethyst | 17238-17249 | 7 | d43 |
| 647 | GemBunnyTopaz | 17238-17249 | 7 | d43 |
| 648 | GemBunnySapphire | 17238-17249 | 7 | d43 |
| 649 | GemBunnyEmerald | 17238-17249 | 7 | d43 |
| 650 | GemBunnyRuby | 17238-17249 | 7 | d43 |
| 651 | GemBunnyDiamond | 17238-17249 | 7 | d43 |
| 652 | GemBunnyAmber | 17238-17249 | 7 | d43 |
| 653 | HellButterfly | 17250-17264 | 65 | d43 |
| 654 | Lavafly | 17265-17279 | 64 | d43 |
| 655 | MagmaSnail | 17280-17294 | 67 | d43 |
| 656 | TownBunny | 17295-17309 | 7 | d43 |
| 657 | QueenSlimeBoss | 17310-17325 | 121 | d43 |
| 658 | QueenSlimeMinionBlue | 17326-17337 | 1 | d43 |
| 659 | QueenSlimeMinionPink | 17326-17337 | 1 | d43 |
| 660 | QueenSlimeMinionPurple | 17338-17350 | 14 | d43 |
| 661 | EmpressButterfly | 17351-17366 | 65 | d43 |
| 662 | PirateGhost | 17367-17383 | 122 | d43 |
| 664 | TorchGod | 17398-17402 | 0 | d43 |
| 665 | ChaosBallTim | 17403-17417 | 9 | d43 |
| 666 | VileSpitEaterOfWorlds | 17418-17433 | 9 | d43 |
| 667 | GoldenSlime | 17434-17448 | 1 | d43 |
| 669 | Stinkbug | 17467-17480 | 115 | d43 |
| 670 | TownSlimeBlue | 17481-17495 | 7 | d43 |
| 671 | ScarletMacaw | 17496-17509 | 24 | d43 |
| 672 | BlueMacaw | 17510-17523 | 24 | d43 |
| 673 | Toucan | 17524-17537 | 24 | d43 |
| 674 | YellowCockatiel | 17538-17551 | 24 | d43 |
| 675 | GrayCockatiel | 17552-17565 | 24 | d43 |
| 676 | ShimmerSlime | 17566-17578 | 1 | d43 |
| 677 | Shimmerfly | 17579-17593 | 64 | d43 |
| 678 | TownSlimeGreen | 17594-17608 | 7 | d43 |
| 679 | TownSlimeOld | 17594-17608 | 7 | d43 |
| 680 | TownSlimePurple | 17594-17608 | 7 | d43 |
| 681 | TownSlimeRainbow | 17594-17608 | 7 | d43 |
| 682 | TownSlimeRed | 17594-17608 | 7 | d43 |
| 683 | TownSlimeYellow | 17594-17608 | 7 | d43 |
| 684 | TownSlimeCopper | 17594-17608 | 7 | d43 |
| 685 | BoundTownSlimeOld | 17609-17623 | 1 | d43 |
| 686 | BoundTownSlimePurple | 17624-17639 | 125 | d43 |
| 687 | BoundTownSlimeYellow | 17640-17652 | 7 | d43 |
| 688 | Pufferfish | 17653-17666 | 16 | d43 |
| 689 | OwlMimic | 17667-17679 | 24 | d43 |
| 690 | StatueMimic | 17680-17695 | 126 | d43 |
| 691 | MossZombie | 17696-17708 | 3 | d43 |
| 692 | Orca | 17709-17722 | 16 | d43 |
| 693 | LibrarianSkeleton | 17723-17736 | 8 | d43 |
| 694 | WaterBoltMimic | 17737-17751 | 10 | d43 |
| 695 | PalworldCattivaDistressed | 17752-17765 | 127 | d43 |
| 696 | PalworldFoxsparksDistressed | 17766-17779 | 127 | d43 |
| 697 | ? | — | — | d43 |

## SEAL — the roster-collision adjudication + the kinds-verify-2 family bible (W4-f18)

Status: **SEALED** (code-only; the four seal items below are verified at the
pushed tip `a832d38ea`, branch `task/W4-f18-npc-kinds-verify-2`, base
`a725acb04` == the d39 base). This section is the f18-side receipt for the
d37-E8 roster collision; it consumes the d39 SEAL (`048c74f8f`, research/125
§6) and the d37 S4 merger bible (`fbb948172`, research/108). **Doc-only: the
f18 code rows stay on-branch so the exhaustive `NpcKind` matches compile (the
E8 dropped-lane rule); the drop-hunks below are EXECUTE-prep for the merger.**

> **Registry note (registry-150 FROZEN §9, d54 tip `880b9ecbe`).** This doc's
> round-1/§6.3 assignment `64 → 249` is **VOID**: §8.2/§9.1 give **249 to d15**
> (band-141-150, `869ac7edd`, sole claimant KEEPS) and §9 carries **no f18
> reassignment**. §9.3 forbids self-picks ("New docs require a registry check +
> Lead assignment; NO self-picks"). The doc therefore **keeps the `64-` filename
> on-branch** (a merge-time collision the registry must adjudicate — f18 requests
> a §9 number ≥367); it is NOT renamed to 249. This SEAL section rides the same
> census doc (no new numbered doc is created, so no new number is consumed).

### Seal item 1 — the 20-kind port landed (verified)

The twenty kinds 355–380 + 430 landed in four batch commits
(`e098cf888`/`89a8d9d6f`/`8e10f9c34`/`b05706702`) + the census/witness/compile-fix
commits. Re-verified at `a832d38ea` by static script (no cargo):

- **Protocol bijection:** `to_u16` 290 arms / `from_u16` 290 arms, **0 duplicate
  ids, 0 kind↔id mismatches, to-set == from-set**; `ALL_KINDS` decl 290 == 290
  rows == the to_u16 kind-set (0 dups, 0 missing, 0 extra); max proto 353
  (i16-safe). The twenty ride port-local **237–258** (250/288 skipped — the
  AngryNimbus/DungeonSpirit's base ids).
- **`npc_ai_style65`** (the Butterfly 356's `AI_065_Butterflies`,
  `NPC.cs:44896-45119`) + **`npc_ai_town_critter`** (the `AI_007_TownEntities`
  TownCritter subset, `NPC.cs:52965-55561`) modules landed; the style-axis
  64/66/67/68/1/41 claims ride the existing modules.
- **4 Hide-folds:** Duck2 363→362, DuckWhite2 365→364, TruffleWormDigger 375→374,
  CultistArcherWhite 380→379 (`bestiary.rs` `try_entry_for_npc`/`entry_for_npc`
  twins; each folded kind keeps its OWN render key + draw sheet).
- **290-bijection** is the `s1_f_3` coverage pin (`npc.rs:28916`) + the
  round-trip test (`npc.rs:25301`).

### Seal item 3 — the `value()` merge-scar repair landed + compile-fixed (verified)

`npc.rs::value()` (the exhaustive match, no wildcard): the 402 StardustWormHead
700→0 (`NPC.cs:15018`), the Moon Lord kb group purged (the core's 1_000_000 row
restored, `NPC.cs:14130`), the six shard kb rows replaced with explicit `0.0`
arms (405/406/407/423/425/429 write no C# value row, stock 0f `NPC.cs:8544`), the
6× repeat SolarCrawltipede×3/StardustWormHead/SolarSpearman block deduped to one,
the second `DD2LanePortal => 0.0` arm removed. Re-verified: each repaired kind
appears **once** in `value()`; the same-value shadowed duplicates
(`DD2LanePortal`×3, `VortexTower`×3, frost trio×2, `EyeOfCthulhu`×2) remain as
documented (compile warnings, no behaviour change — out of scope). The
compile-fix `a832d38ea` saturates the butterfly FindFrame colour offset at
`ai[2]=0` (the unreachable grid state; the live 1..8 style unchanged).

### Seal item 2 — ROSTER-COLLISION d37-E8: the keep-one verdict + exact drop-hunks

**The collision.** f18's twenty kinds ride port-local **237–258**, which is
**off-plan on two counts** (d39 SEAL §3, confirmed against research/108):
1. **237–249 are the reserved cs-holes** ("cs-owned, never reassign", 108 §3
   base-roster row + R4/R6). Verified owners at the d37 tip (ENTRIES net_id /
   cs_id scan): 237 JungleCreeper(Wall), 238 BlackRecluse, 239/240 the
   BloodCrawler pair, 241 BloodFeeder, 242 BloodJelly, 243 IceGolem, 244
   RainbowSlime (R6's list), 245 Golem (base cs), 246-249 the remaining d37
   cs ids — all owned by d37's pushed 354–399 protocol band kinds (their
   `cs_id`/`net_id`, NOT their protocol ids).
2. **250–265 are the base/d37 cs corridor**: 250 AngryNimbus (base kind, proto
   250), 251 Eyezor, 252 Parrot, 253 Reaper, 254/255 SporeZombie, 256 FungoFish,
   257 AnomuraFungus, 258 MushiLadybug, 259 FungiBulb, 260 GiantFungiBulb (d37
   ENTRIES nets), 261-265 the base Plantera family (FungiSpore/Plantera/
   PlanterasHook/PlanterasTentacle/Spore).

So **none of f18's 237–258 ids survives the merge**. The d39 SEAL (`048c74f8f`)
adjudicates the keep-one (consumed here as the f18-side receipt):

| f18 kind | f18 proto | verdict | canonical owner / id |
|---|---|---|---|
| Butterfly 356 | 238 | **keep-d39-roster + splice-f18-AI** | d39 @817 |
| ScorpionBlack 366 | 248 | keep-d39-roster + splice-f18-AI | d39 @818 |
| Scorpion 367 | 249 | keep-d39-roster + splice-f18-AI | d39 @819 |
| Grasshopper 377 | 254 | keep-d39-roster + splice-f18-AI | d39 @820 |
| ChatteringTeethBomb 378 | 255 | keep-d39-roster + splice-f18-AI | d39 @821 |
| CultistArcherBlue 379 | 256 | keep-d39-roster + splice-f18-AI | d39 @822 |
| CultistArcherWhite 380 | 257 | keep-d39-roster + splice-f18-AI | d39 @823 |
| Firefly 355 | 237 | drop → E8 owner | c7 @735 |
| LightningBug 358 | 240 | drop → E8 owner | c7 @736 |
| Worm 357 | 239 | drop → E8 owner | c8 @737 |
| TruffleWorm 374 | 252 | drop → E8 owner | c8 @738 |
| TruffleWormDigger 375 | 253 | drop → E8 owner | c8 @739 |
| Snail 359 | 241 | drop → E8 owner | c9 @740 |
| GlowingSnail 360 | 242 | drop → E8 owner | c9 @741 |
| Frog 361 | 243 | drop → E8 owner | e26 @744 |
| Duck 362 | 244 | drop → E8 owner | c10 @731 |
| Duck2 363 | 245 | drop → E8 owner | c10 @732 |
| DuckWhite 364 | 246 | drop → E8 owner | c10 @733 |
| DuckWhite2 365 | 247 | drop → E8 owner | c10 @734 |
| ArmedZombie 430 | 258 | drop → E8 owner | d40 @430 |

**f18 contributes ZERO roster rows at merge** (d39 SEAL §4): every `NpcKind`
variant + `to_u16`/`from_u16`/`ALL_KINDS`/`ENTRIES`/stat row drops. **f18's
faithful AI/draw/sound wiring SURVIVES, spliced onto the canonical kinds** (the
E3 dropped-lane rule): the 7 overlap kinds keep d39's roster rows @817–823 but
route through f18's machines (the scorpion pair's `npc_ai_town_critter` arm, the
grasshopper's `ai_001_grasshopper`, the archer pair's `update_cultist_archer`
shooter, the butterfly's `npc_ai_style65`); the 13 non-overlap kinds' wiring
rides the E8 owners' kinds. The stat rows are **byte-equal across lanes**
(d39 SEAL §3 scripted spot-check ALL AGREE; both port the same `NPC.cs` arms),
so the drop loses no behaviour — the owners' roster + f18's wiring is the
superset.

**The E8 target ids are the d37 plan, NOT yet executed at the owner tips**
(d37 S1 receipts: c7/c8/c9/c10/e26 "none executed their E8 tables"; c9's
ALL_KINDS array-missing-4 is the S2 fix-hunk `0466ecf81`). The table's
"canonical owner / id" column is the **merge-wave destination** per the d37 S4
merger bible (c10 ducks→731–734, c7→735/736, c8→737–739, c9→740/741,
e26 Frog→744, d40 ArmedZombie@430 as-pushed); the owners execute their own
renumbers on their own branches. f18's drop-hunks below are independent of
those renumbers (f18's rows drop wholesale; the owners' rows are already on
their branches at their current ids).

**The f18 ArmedZombie 430 × d40 note.** d40 (`f4355d380`, the sheets phase) owns
430 at protocol==vanilla (the 430–488 band, 108 §3). d40's ArmedZombie is
data-only + the grouped draw arm; its bestiary credit **folds onto Zombie 3**
(`ContentSamples.cs:1027`, Hide=true `NPCID.cs:427-432`) — f18's own `ArmedZombie`
ENTRIES row (net 430) is the deviation that drops. f18's `walker_step_fighter`
charge-family wiring (the `npc_ai3_speeds` 430–436 arm) splices onto d40's kind.

#### The exact f18 drop-hunks (EXECUTE-prep, pinned to `a832d38ea`)

Every range below was read from the working tree at `a832d38ea` (clean). DROP =
delete the lines (the canonical owner's row survives); the per-kind line map is
the appendix at the end of this section (scripted, whole-word kind match). The
hunks are grouped by file; **apply bottom-up** (descending line order) so earlier
ranges stay valid. `cs_id`/`net_id`/sheets/render-keys are cs/net-keyed and DO
NOT move (R1's seam evidence) — but f18's kinds ALL drop, so their cs/net rows
drop too; the owners' cs/net rows are already on their branches.

**`crates/tr-app/src/npc.rs`** (the enum + the stat tables + the AI dispatch +
FindFrame + render_key + ALL_KINDS + the test pins):
- 1089–1200 DROP — the 20 enum variants + doc-triples (Firefly…ArmedZombie).
- 1892–1896 + 3220 DROP — the `wants_transform_ai1` field + init (the aiStyle-68
  dry-landing seam; KEEP if the c10 waterfowl splice needs it — the merger
  decides; f18's Duck2/DuckWhite2 wiring rides c10).
- 3158–3162 EDIT — the `chaseable` gate: revert to `chaseable: true` (the
  cultist-archer false-writers ride d39's roster; d39's own gate carries them).
- 3651–3671 DROP — the kb block (20 rows).
- 4052–4066 DROP — the defense block (20 rows).
- 4418–4436 DROP — the value block (20 rows).
- 5077–5130 DROP — the style-axis arms 64/65/66/67/7/68/1/41 (the f18 claims).
  **SPLICE NOTE:** the 7 overlap kinds' arms (65 butterfly, 7 scorpion, 1
  grasshopper, 41 teeth-bomb) are the faithful machines d39 keeps — the merger
  re-points them at d39's kinds, NOT a blind drop. The 13 non-overlap arms drop
  to the E8 owners' wiring.
- 5882–6445 DROP — the nine family wrappers (`ai_064_firefly_family`,
  `ai_065_butterfly_family`, `ai_066_crawler_family`, `ai_067_snail_family`,
  `ai_007_town_critter`, `ai_068_waterfowl`, `ai_001_grasshopper`,
  `update_cultist_archer`, `update_truffle_worm_digger`). **SPLICE NOTE:** the
  butterfly/scorpion/grasshopper/archer wrappers survive keyed on d39's kinds.
- 6906–6983 DROP — the `update_kind_ai` arms (the unreachable! markers + the
  TruffleWormDigger/cultist-archer/ArmedZombie dispatch).
- 7502–7943 DROP — the FindFrame cases (355/358, 356, 357, 359/360, 361,
  362/364, 363/365, 366/367, 374, 375, 377, 378, 379/380, 430).
- 19895–19926 DROP — the render_key block (20 rows).
- 24995 EDIT — `ALL_KINDS: [NpcKind; 290]` → **270** (the base count; the merger
  reconciles to the union 583, not 270 — this is the f18-local revert).
- 25274–25297 DROP — the 20 ALL_KINDS rows + 4 batch comments.
- 26080–26086 DROP — the butterfly frame-strip test's 24-row colour-band arm.
- 28916–28917 EDIT — the `s1_f_3` pin 290 → **270** + the message (f18-local).
- 32559–32585 DROP — the `s1_d_2` CS_STYLES rows (20).

**`crates/tr-app/src/npc/data.rs`**:
- 589–619 DROP — the 20 `to_u16` arms + 4 batch comments.
- 950–978 DROP — the 20 `from_u16` arms + 4 batch comments.
- 1043–1049 + 1164–1166 + 3378–3380 DROP — the TruffleWormDigger `is_worm` /
  worm_tunables / behind_tiles rows (ride c8's kind).
- 1869–1880 DROP — the hit-sound block.
- 2185–2198 DROP — the death-sound block.
- 2299–2316 DROP — the no_gravity block.
- 2568–2591 DROP — the ai_style block.
- 2700–2705 DROP — the ai3_walk_speed shooter/charge rows.
- 3128–3157 DROP — the size block.
- 3802–3816 DROP — the max_hp block.
- 4181–4195 DROP — the contact_damage block.

**`crates/tr-app/src/bestiary.rs`**:
- 2174–2318 DROP — the 16 ENTRIES rows + comments (the 4 Hide-folds have no row).
- 2440–2451 DROP — the `try_entry_for_npc` folds (Duck2/DuckWhite2/
  TruffleWormDigger/CultistArcherWhite).
- 2523–2532 DROP — the `entry_for_npc` folds (the twins).

**`crates/tr-app/src/npc_buffs.rs`**: 487–518 DROP — the immunity block (20).

**`crates/tr-app/src/app/netglue.rs`**: 3569–3623 DROP — the npc_seed arms (20).

**`crates/tr-app/src/ui/bestiary.rs`**: 1598–1721 DROP — the portrait arms (20).

**`crates/tr-app/src/render/entities.rs`**:
- 518 DROP — the `.chain(CRITTER_SHEETS)`.
- 1117–1128 DROP — the texture_net_id Hide-fold rows (Duck2/DuckWhite2/
  TruffleWormDigger/CultistArcherWhite).
- 1263–1274 DROP — the `CRITTER_SHEETS` const.
- 3198–3274 DROP — the draw arms (20).
- 3424–3443 DROP — the rotation-whitelist rows.

**`crates/tr-app/src/assets.rs`**:
- 1278–1306 DROP — the STATIC_SHEET_NAMES rows (20).
- 1892 EDIT — `BESTIARY_NPC_SHEET_IDS: [u16; 197]` → **181** (f18-local).
- 1927–1939 DROP — the 16 sheet-id rows.
- 2702–2725 DROP — the 16 strip lookups.

**`crates/tr-app/src/main.rs`**: 78–80 DROP — the `npc_w4_f18_witnesses`
module decl + comment (the witnesses drop with the roster). 71–77 KEEP — the
`npc_ai_style65` + `npc_ai_town_critter` decls survive (the splice keepers, see
the style65 keep-one below).

**`crates/tr-app/src/npc_ai_town.rs`**: the `pub(crate)` visibility upgrades
(tile_at/raw_solid/raw_solid_top/nactive/top_slope/solid_tiles/
solid_tiles_versatile/drown_collision/step_down/town_step_up) — KEEP (the
town_critter module reads them; harmless if unused).

**`crates/tr-app/src/world_sim/npcs.rs`**: 6165–6181 DROP — the Transform drain
arms (362/364/375) + the ai1 write (ride c10/c8's kinds).

**Whole-file DROP:** `crates/tr-app/src/npc_w4_f18_witnesses.rs` (303 lines, 21
tests — the stat pins ride the owners' witnesses).

### Seal item 4 — research/64 SEALED + the kinds-verify-2 family bible

This census (research/64) is **SEALED**: the 347-id 351–697 `SetDefaults` census
(343 arm-bearing + 4 no-arm), the 105→125 ported scoreboard, the first-20 port,
the `value()` repair, and the roster-collision adjudication above are complete
and verified at `a832d38ea`. The remaining 222 unported kinds are the d39–d43
waves' scope (the table above).

**The kinds-verify-2 family bible (one paragraph).** W4-f18 is the **verify lane**
for the NPC kinds 351–697 band: it censused the `SetDefaults` chain, ported the
first 20 arm-bearing missing kinds (355–380 + 430) verbatim with full stat rows +
protocol bijection + bestiary + FindFrame + AI routing, repaired the `value()`
merge-scar, and wrote 21 stat witnesses (WRITTEN NOT RUN, code-only). Its twenty
kinds **all drop at merge** (the 237–258 band squats the reserved cs-holes +
d37's corridor): the 7 overlap kinds (Butterfly/ScorpionBlack/Scorpion/
Grasshopper/ChatteringTeethBomb/CultistArcherBlue/CultistArcherWhite) keep d39's
roster @817–823 and splice f18's faithful AI arms; the 13 non-overlap drop to
their E8 owners (c7 Firefly/LightningBug, c8 Worm/TruffleWorm/TruffleWormDigger,
c9 Snail/GlowingSnail, e26 Frog, c10 the four ducks, d40 ArmedZombie). f18's
enduring contribution is the **wiring superset** — the `npc_ai_style65` butterfly
module, the `npc_ai_town_critter` TownCritter module, the `ai_001_grasshopper`
faithful hop (the 377 flee/friendly-latch/+3-charge/0.9-0.6-damping/ceiling-clamp
rows, `NPC.cs:61080/:61440/:61588/:61684` — the d39 "AI_001 has no 377 branch"
claim was WRONG, corrected in both lanes), the `update_cultist_archer` AI_003
shooter (80-tick cooldown, 81 arrow / 40 dmg / 11 speed, ±40 jitter NO rescale,
|dx|*0.1 aim lift NOT zeroed, centre spawn NO +6 y), and the AI_006 lone-digger
(inverted aim). The merger splices these onto the canonical kinds; the bijection
gate (decodable == ALL_KINDS.len(), i16-safe) + the s1_f_3 pin guard every splice
(552 core / 583 grand per d37 S4). **f18 zero roster rows; the wiring is the
legacy.**

### The style65 keep-one (f18 × d39, same module name)

Both lanes add `npc_ai_style65.rs` (f18 629 lines / 7 tests; d39 625 lines / 8
tests). They are the **same `AI_065_Butterflies` port** with one behavioural
difference: **d39's tile scans carry `WorldGen.InWorld(x,y,2)` guards; f18's do
not.** The C# `AI_065` (`NPC.cs:45026/:45044`) reads `Main.tile[num28,j] != null`
with **NO InWorld gate** (unlike `AI_064`'s `:34197/:34215` which DO gate) — so
**f18's version is the faithful one** (the null-check is the port's
`world.tile()` `Option`). The merger keeps **f18's `npc_ai_style65`** (the
no-InWorld version) and drops d39's; the colour-roll/scale-latch/rearm/threat-flee
maths are byte-identical (both cite the same `Next` order). The d39 FindFrame
butterfly arm lacks f18's `ai[2]=0` saturation compile-fix (`a832d38ea`) — f18's
FindFrame arm is the keeper too.

### Appendix — the per-kind f18 line map (scripted, `a832d38ea`, whole-word match)

Each cell lists the f18 line numbers mentioning the kind (the drop-hunk atoms;
the grouped ranges above are the apply-ready hunks). Files abbreviated:
`npc`=npc.rs, `data`=npc/data.rs, `best`=bestiary.rs, `buff`=npc_buffs.rs,
`net`=app/netglue.rs, `ui`=ui/bestiary.rs, `ent`=render/entities.rs,
`ass`=assets.rs, `main`=main.rs, `sim`=world_sim/npcs.rs. Shared group rows
(the kb/def/value/hit/death/ai_style/size/hp/dmg blocks) list every member kind
on one line — drop the whole block, not per-kind.

| kind | npc | data | best | buff | net | ui | ent | ass | main | sim |
|---|---|---|---|---|---|---|---|---|---|---|
| Firefly | 1089,1093,1107,3654,4054,4421,5078,5883,6911,7506,19897,25275,32561 | 593,952,1871,2187,2304,2571,3132,3804,4183 | 2179,2181,2184 | 490 | 3574 | 1605 | 1266,3204 | — | — | — |
| Butterfly | 1094,1098,3654,4054,4421,5084,5915,6914,7525,19898,25276,26080,26086,32562 | 594,953,1871,2187,2304,2572,3133,3804,4183 | 2187,2189,2192 | 491 | 3575 | 1611 | 1266,3207,3429 | — | 71 | — |
| Worm | 1099,1103,3654,4054,4421,5088,5932,6917,7561,19899,25277,32563 | 595,954,1871,2187,2301,2573,3134,3804,4183 | 2195,2197,2200 | 492 | 3580 | 1617 | 1266,3210,3430 | — | — | — |
| LightningBug | 1104,1108,3654,4054,4421,6911,7506,19900,25278,32564 | 596,955,1871,2187,2304,2571,3135,3804,4183 | 2203,2205 | 493 | 3576 | 1605 | 3213 | — | — | — |
| Snail | 1109,1113,1117,3654,4054,4421,5095,5970,6920,7587,7597,19901,25279,32565 | 597,956,1871,2187,2302,2304,2574,3136,3804,4183 | 2211,2213,2216 | 494 | 3580 | 1623 | 1267,3216,3431,3433 | — | — | — |
| GlowingSnail | 1114,1118,3658,4057,4425,5095,5970,6923,6926,7599,19905,25281,32568 | 600,959,1874,2190,2305,2309,2577,3140,3807,4186 | 2218,2223,2225 | 498 | 3583,3587 | 1629,1632 | 3219,3223,3432,3435 | 1930 | — | — |
| Frog | 1119,1123,3658,4057,4425,5103,5108,5998,6005,6924,6929,7613,19906,25282,32569 | 601,960,1874,2190,2306,2578,3141,3807,4186 | 2231,2233,2236 | 499 | 3584,3591 | 1629,1638 | 3220,3226 | 1930 | 75 | — |
| Duck | 1124,1128,1132,3658,4057,4425,5103,5108,5998,6924,6929,7659,19907,25283,32570 | 602,961,1874,2190,2578,3142,3807,4186 | 2239,2241,2244 | 500 | 3584,3592 | 1630,1644,1650 | 1118,3229 | — | 75 | 6166,6168 |
| Duck2 | 1127,1129,1134,3658,4057,4425,5112,6021,6924,6932,7682,7717,19903,19908,25284,32571 | 603,962,1874,2190,2307,2579,3143,3807,4186 | 2440,2443,2523,2525 | 501 | 3585,3593 | 1630,1650,1653 | 1117,1120,1270,3221,3232,3433,3436 | 1286,1931 | — | 6165 |
| DuckWhite | 1135,1139,1143,3658,4057,4425,5103,5108,5999,6924,6929,7659,19909,25285,32572 | 604,963,1874,2190,2578,3142,3807,4186 | 2247,2249,2256 | 502 | 3584,3594 | 1644,1660 | 3235 | — | 75 | 6170,6172 |
| DuckWhite2 | 1138,1140,1145,3659,3663,4060,4429,6935,6938,7719,19911,19914,25287,32575 | 608,967,1877,2193,2310,2582,3147,3810,4189 | 2444,2447,2526,2528 | 506 | 3597,3602 | 1660,1675 | 1121,1124,3238,3242,3437,3441 | 1293,1934 | — | 6169 |
| ScorpionBlack | 1146,1150,3663,4060,4429,5108,6941,7757,19915,25288,32576 | 609,968,1877,2193,2583,3148,3810,4189 | 2260,2262 | 507 | 3603 | 1663 | 3245 | — | — | — |
| Scorpion | 1151,1155,3663,4060,4429,5108,6941,7757,19916,25289,32577 | 610,969,1877,2193,2583,3148,3810,4189 | 2265,2268,2270,2273 | 508 | 3604 | 1663 | 3248 | — | — | — |
| TruffleWorm | 1156,1161,1166,3659,3663,4060,4429,5088,5932,5952,6936,6944,7785,19917,25290,32578 | 611,970,1877,2193,2310,2584,3149,3810,4189 | 2276,2278,2446,2448,2529 | 509 | 3598,3605 | 1659,1661,1669 | 3239,3251,3438,3442 | 1933 | — | 6170 |
| TruffleWormDigger | 1160,1162,1169,3661,3664,4060,4429,5954,6248,6947,6951,7816,19911,19918,25291,32579 | 612,971,1043,1049,1164,1166,1877,2193,2311,2585,3150,3378,3380,3810,4189 | 2445,2448,2526,2529 | 510 | 3599,3608 | 1661,1681 | 1121,1125,3240,3254,3439,3443 | 1294,1934 | — | 6171,6173 |
| Grasshopper | 1170,1175,3668,4063,4433,5118,5123,6070,6952,6955,7829,19922,25293,32581 | 615,974,1880,2197,2588,3154,3813,4192 | 2288,2290,2293 | 514 | 3612,3617 | 1687,1692 | 3257,3260 | — | — | — |
| ChatteringTeethBomb | 1176,1181,3669,4064,4434,5127,6953,6958,7840,19923,25294,32582 | 616,975,1880,2197,2589,3155,3814,4193 | 2296,2298 | 515 | 3613,3618 | 1687,1698 | 3257,3263 | — | — | — |
| CultistArcherBlue | 1182,1187,1192,3162,3670,4065,4435,6113,6963,7862,19924,25295,32583 | 617,976,1880,2197,2590,2704,3156,3815,4194 | 2304,2306,2450,2451,2531,2532 | 516 | 3619 | 1688,1704 | 3266 | — | — | — |
| CultistArcherWhite | 1188,1194,3162,3670,4065,4435,6113,6963,7862,19920,19925,25296,32584 | 618,977,1880,2197,2590,2704,3156,3815,4194 | 2449,2451,2530,2532 | 517 | 3620 | 1689,1710 | 1126,1128,3269 | 1301,1937 | — | — |
| ArmedZombie | 1195,1200,3671,4066,4436,6966,6969,7899,19926,25297,32585 | 619,978,1880,2195,2198,2591,2701,2705,3157,3816,4195 | 2312,2314 | 512,518 | 3613,3621 | 1688,1716 | 3258,3272 | — | — | — |

(The `best` cells omit the shared batch-comment lines 2174/2218/2254/2283 and
the fold-comment lines; the `ent`/`ass` cells include the CRITTER_SHEETS /
STATIC_SHEET_NAMES / sheet-id / strip rows. The Worm/Snail `best`/`ent`/`ass`/
`sim` hits at 695/1919/823/2165/13806/etc. are the PRE-EXISTING base Worm-kind
references — NOT f18 additions; the scripted whole-word match catches the base
`Worm`/`Snail` substrings. The f18-added rows are the ones in the grouped
drop-hunks above; the appendix is the kind→line index, the hunks are the
authority.)

