# 64 — NPC kinds 351–697 `SetDefaults` census + first-20 port (W4-f18)

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

