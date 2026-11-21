# E8-RECONCILE-SEAL per-lane renumber tables (refreshed at current tips)

Each shifting lane executes its table on ITS OWN branch: change the listed
`to_u16` arms AND the mirrored `from_u16` arms, update witness pins/comments
naming the old protocol id, and DROP the listed duplicate ports (variant +
to/from arms + ALL_KINDS row + ENTRIES row + stat arms). `cs_id`, ENTRIES
`net_id`, sheets, render keys and every cs/net-keyed seam DO NOT CHANGE.

## c10 — 4 owned, 4 renumber, 16 drop

| Kind | old | new | cs_id | net_id |
|---|---|---|---|---|
| Duck | 362 | **731** | 362 | 362 |
| Duck2 | 363 | **732** | 363 | 363 |
| DuckWhite | 364 | **733** | 364 | 364 |
| DuckWhite2 | 365 | **734** | 365 | 365 |

DROP duplicate ports (keep canonical owner's row+wiring):
- Worm: drop c10@357 -> keep c8@737
- GoldBird: drop c10@442 -> keep d40@442
- GoldBunny: drop c10@443 -> keep d40@443
- GoldButterfly: drop c10@444 -> keep d40@444
- GoldFrog: drop c10@445 -> keep d40@445
- GoldGrasshopper: drop c10@446 -> keep d40@446
- GoldMouse: drop c10@447 -> keep d40@447
- GoldWorm: drop c10@448 -> keep d40@448
- EnchantedNightcrawler: drop c10@484 -> keep d40@484
- Grubby: drop c10@485 -> keep d40@485
- Sluggy: drop c10@486 -> keep d40@486
- Buggy: drop c10@487 -> keep d40@487
- Seagull: drop c10@602 -> keep d42@602
- Seagull2: drop c10@603 -> keep d42@603
- Grebe: drop c10@608 -> keep d42@608
- Grebe2: drop c10@609 -> keep d42@609

## c3 — 1 owned, 0 renumber, 2 drop

DROP duplicate ports (keep canonical owner's row+wiring):
- GiantCursedSkull: drop c3@289 -> keep d38@289
- CursedSkull: drop c3@355 -> keep d34@267

## c4 — 8 owned, 0 renumber, 5 drop

DROP duplicate ports (keep canonical owner's row+wiring):
- BloodFeeder: drop c4@241 -> keep d37@373
- FlyingFish: drop c4@359 -> keep d37@359
- CorruptGoldfish: drop c4@405 -> keep d34@277
- Pufferfish: drop c4@688 -> keep d43@688
- Orca: drop c4@692 -> keep d43@692

## c6 — 0 owned, 0 renumber, 1 drop

DROP duplicate ports (keep canonical owner's row+wiring):
- PumpkingBlade: drop c6@328 -> keep d38@328

## c7 — 4 owned, 2 renumber, 0 drop

| Kind | old | new | cs_id | net_id |
|---|---|---|---|---|
| Firefly | 355 | **735** | 355 | 355 |
| LightningBug | 358 | **736** | 358 | 358 |

Keep as pushed: Lavafly@654, Shimmerfly@677

## c7b — 0 owned, 0 renumber, 4 drop

DROP duplicate ports (keep canonical owner's row+wiring):
- Firefly: drop c7b@355 -> keep c7@735
- LightningBug: drop c7b@358 -> keep c7@736
- Lavafly: drop c7b@654 -> keep c7@654
- Shimmerfly: drop c7b@677 -> keep c7@677

## c8 — 4 owned, 3 renumber, 5 drop

| Kind | old | new | cs_id | net_id |
|---|---|---|---|---|
| Worm | 357 | **737** | 357 | 357 |
| TruffleWorm | 374 | **738** | 374 | 374 |
| TruffleWormDigger | 375 | **739** | 375 | 375 |

Keep as pushed: Maggot@606

DROP duplicate ports (keep canonical owner's row+wiring):
- GoldWorm: drop c8@448 -> keep d40@448
- EnchantedNightcrawler: drop c8@484 -> keep d40@484
- Grubby: drop c8@485 -> keep d40@485
- Sluggy: drop c8@486 -> keep d40@486
- Buggy: drop c8@487 -> keep d40@487

## c9 — 3 owned, 2 renumber, 6 drop

| Kind | old | new | cs_id | net_id |
|---|---|---|---|---|
| Snail | 359 | **740** | 359 | 359 |
| GlowingSnail | 360 | **741** | 360 | 360 |

Keep as pushed: MagmaSnail@655

DROP duplicate ports (keep canonical owner's row+wiring):
- GoldBird: drop c9@442 -> keep d40@442
- GoldBunny: drop c9@443 -> keep d40@443
- GoldFrog: drop c9@445 -> keep d40@445
- GoldGrasshopper: drop c9@446 -> keep d40@446
- GoldMouse: drop c9@447 -> keep d40@447
- Shimmerfly: drop c9@677 -> keep c7@677

## d103 — 0 owned, 0 renumber, 1 drop

DROP duplicate ports (keep canonical owner's row+wiring):
- Wraith: drop d103@360 -> keep d35@705

## d11 — 0 owned, 0 renumber, 1 drop

DROP duplicate ports (keep canonical owner's row+wiring):
- BloodSquid: drop d11@619 -> keep d42@619

## d120 — 4 owned, 4 renumber, 1 drop

| Kind | old | new | cs_id | net_id |
|---|---|---|---|---|
| SantaClaus | 241 | **763** | 142 | 142 |
| Truffle | 242 | **764** | 160 | 160 |
| Steampunker | 243 | **765** | 178 | 178 |
| Cyborg | 244 | **766** | 209 | 209 |

DROP duplicate ports (keep canonical owner's row+wiring):
- BartenderUnconscious: drop d120@245 -> keep d42@579

## d125 — 2 owned, 0 renumber, 0 drop (KEEP receipt)

## d23 — 4 owned, 3 renumber, 19 drop

| Kind | old | new | cs_id | net_id |
|---|---|---|---|---|
| AngryTrapper | 357 | **743** | None | 175 |
| SporeBat | 407 | **634** | None | 634 |
| QueenSlimeMinionPurple | 409 | **660** | None | 660 |

Keep as pushed: SolarFlare@408

DROP duplicate ports (keep canonical owner's row+wiring):
- FungiBulb: drop d23@259 -> keep d37@385
- GiantFungiBulb: drop d23@260 -> keep d37@386
- Harpy: drop d23@275 -> keep d34@275
- ManEater: drop d23@354 -> keep d34@269
- Snatcher: drop d23@355 -> keep d34@276
- Clinger: drop d23@356 -> keep d35@718
- FlyingSnake: drop d23@360 -> keep d37@360
- Demon: drop d23@400 -> keep d34@279
- IceBat: drop d23@401 -> keep g8@757
- Lavabat: drop d23@402 -> keep g8@758
- GiantFlyingFox: drop d23@403 -> keep g8@759
- RedDevil: drop d23@404 -> keep g8@761
- VampireBat: drop d23@405 -> keep g8@760
- Vampire: drop d23@406 -> keep g8@762
- VileSpitEoW: drop d23@411 -> keep d43 VileSpitEaterOfWorlds@666 (same vanilla 666)
- GiantBat: drop d23@441 -> keep d35@710
- VileSpit: drop d23@455 -> keep d35@724
- Slimer: drop d23@458 -> keep d35@727
- IlluminantBat: drop d23@460 -> keep d35@729

## d33 — 11 owned, 4 renumber, 0 drop

| Kind | old | new | cs_id | net_id |
|---|---|---|---|---|
| FairyCritterPink | 359 | **583** | 583 | 583 |
| FairyCritterGreen | 360 | **584** | 584 | 584 |
| FairyCritterBlue | 361 | **585** | 585 | 585 |
| WindyBalloon | 368 | **594** | 594 | 594 |

Keep as pushed: BlackDragonfly@595, BlueDragonfly@596, GreenDragonfly@597, OrangeDragonfly@598, RedDragonfly@599, YellowDragonfly@600, GoldDragonfly@601

## d34 — 17 owned, 2 renumber, 3 drop

| Kind | old | new | cs_id | net_id |
|---|---|---|---|---|
| Crab | 282 | **791** | 67 | 67 |
| DungeonGuardian | 283 | **792** | 68 | 68 |

Keep as pushed: AngryBones@266, CursedSkull@267, OldMan@268, ManEater@269, Bunny@270, BoneSerpentHead@271, BoneSerpentBody@272, BoneSerpentTail@273, CorruptBunny@274, Harpy@275, Snatcher@276, CorruptGoldfish@277, Vulture@278, Demon@279, BlueJellyfish@280

DROP duplicate ports (keep canonical owner's row+wiring):
- PinkJellyfish: drop d34@281 -> keep g8@745
- Antlion: drop d34@284 -> keep g8@746
- SpikeBall: drop d34@285 -> keep g8@747

## d35 — 33 owned, 33 renumber, 0 drop

| Kind | old | new | cs_id | net_id |
|---|---|---|---|---|
| GoblinScout | 429 | **698** | 73 | 73 |
| Bird | 430 | **699** | 74 | 74 |
| Pixie | 431 | **700** | 75 | 75 |
| ArmoredSkeleton | 432 | **701** | 77 | 77 |
| Mummy | 433 | **702** | 78 | 78 |
| DarkMummy | 434 | **703** | 79 | 79 |
| LightMummy | 435 | **704** | 80 | 80 |
| Wraith | 436 | **705** | 82 | 82 |
| CursedHammer | 437 | **706** | 83 | 83 |
| EnchantedSword | 438 | **707** | 84 | 84 |
| Mimic | 439 | **708** | 85 | 85 |
| Unicorn | 440 | **709** | 86 | 86 |
| GiantBat | 441 | **710** | 93 | 93 |
| Corruptor | 442 | **711** | 94 | 94 |
| DiggerHead | 443 | **712** | 95 | 95 |
| DiggerBody | 444 | **713** | None | None |
| DiggerTail | 445 | **714** | None | None |
| SeekerHead | 446 | **715** | 98 | 98 |
| SeekerBody | 447 | **716** | None | None |
| SeekerTail | 448 | **717** | None | None |
| Clinger | 449 | **718** | 101 | 101 |
| AnglerFish | 450 | **719** | 102 | 102 |
| GreenJellyfish | 451 | **720** | 103 | 103 |
| Werewolf | 452 | **721** | 104 | 104 |
| Clown | 453 | **722** | 109 | 109 |
| SkeletonArcher | 454 | **723** | 110 | 110 |
| VileSpit | 455 | **724** | None | None |
| TheHungryII | 456 | **725** | 116 | 116 |
| ChaosElemental | 457 | **726** | 120 | 120 |
| Slimer | 458 | **727** | 121 | 121 |
| Gastropod | 459 | **728** | 122 | 122 |
| IlluminantBat | 460 | **729** | 137 | 137 |
| PossessedArmor | 461 | **730** | 140 | 140 |

## d36 — 9 owned, 0 renumber, 0 drop (KEEP receipt)

## d37 — 46 owned, 0 renumber, 0 drop (KEEP receipt)

## d38 — 35 owned, 0 renumber, 0 drop (KEEP receipt)

## d39 — 7 owned, 0 renumber, 0 drop (KEEP receipt)

## d40 — 55 owned, 0 renumber, 1 drop

DROP duplicate ports (keep canonical owner's row+wiring):
- CrimsonGoldfish: drop d40@465 -> keep c4@465

## d41 — 36 owned, 0 renumber, 2 drop

DROP duplicate ports (keep canonical owner's row+wiring):
- GiantFlyingAntlion: drop d41@509 -> keep c4@509
- SolarFlare: drop d41@516 -> keep d23@408

## d42 — 32 owned, 0 renumber, 11 drop

DROP duplicate ports (keep canonical owner's row+wiring):
- FairyCritterPink: drop d42@583 -> keep d33@583
- FairyCritterGreen: drop d42@584 -> keep d33@584
- FairyCritterBlue: drop d42@585 -> keep d33@585
- WindyBalloon: drop d42@594 -> keep d33@594
- BlackDragonfly: drop d42@595 -> keep d33@595
- BlueDragonfly: drop d42@596 -> keep d33@596
- GreenDragonfly: drop d42@597 -> keep d33@597
- OrangeDragonfly: drop d42@598 -> keep d33@598
- RedDragonfly: drop d42@599 -> keep d33@599
- YellowDragonfly: drop d42@600 -> keep d33@600
- GoldDragonfly: drop d42@601 -> keep d33@601

## d43 — 42 owned, 0 renumber, 21 drop

DROP duplicate ports (keep canonical owner's row+wiring):
- SporeBat: drop d43@634 -> keep d23@634
- HallowBoss: drop d43@636 -> keep d125 EmpressOfLight@636 (same vanilla 636)
- TownCat: drop d43@637 -> keep g10@637
- TownDog: drop d43@638 -> keep g10@638
- Lavafly: drop d43@654 -> keep c7@654
- MagmaSnail: drop d43@655 -> keep c9@655
- TownBunny: drop d43@656 -> keep g10@656
- QueenSlimeMinionPurple: drop d43@660 -> keep d23@660
- EmpressButterfly: drop d43@661 -> keep d125 PrismaticLacewing@661 (same vanilla 661)
- TownSlimeBlue: drop d43@670 -> keep g10@670
- ShimmerSlime: drop d43@676 -> keep f47@676
- Shimmerfly: drop d43@677 -> keep c7@677
- TownSlimeGreen: drop d43@678 -> keep g10@678
- TownSlimeOld: drop d43@679 -> keep g10@679
- TownSlimePurple: drop d43@680 -> keep g10@680
- TownSlimeRainbow: drop d43@681 -> keep g10@681
- TownSlimeRed: drop d43@682 -> keep g10@682
- TownSlimeYellow: drop d43@683 -> keep g10@683
- TownSlimeCopper: drop d43@684 -> keep g10@684
- LibrarianSkeleton: drop d43@693 -> keep g5@693
- WaterBoltMimic: drop d43@694 -> keep c3@694

## d9 — 1 owned, 0 renumber, 6 drop

DROP duplicate ports (keep canonical owner's row+wiring):
- CultistDragonHead: drop d9@454 -> keep d40@454
- CultistDragonBody1: drop d9@455 -> keep d40@455
- CultistDragonBody2: drop d9@456 -> keep d40@456
- CultistDragonBody3: drop d9@457 -> keep d40@457
- CultistDragonBody4: drop d9@458 -> keep d40@458
- CultistDragonTail: drop d9@459 -> keep d40@459

## e21 — 1 owned, 1 renumber, 1 drop

| Kind | old | new | cs_id | net_id |
|---|---|---|---|---|
| PartyBunny | 237 | **540** | 540 | 540 |

DROP duplicate ports (keep canonical owner's row+wiring):
- Bunny: drop e21@229 -> keep d34@270

## e26 — 3 owned, 0 renumber, 0 drop (KEEP receipt)

## f18 — 0 owned, 0 renumber, 20 drop

DROP duplicate ports (keep canonical owner's row+wiring):
- Firefly: drop f18@237 -> keep c7@735
- Butterfly: drop f18@238 -> keep d39@817
- Worm: drop f18@239 -> keep c8@737
- LightningBug: drop f18@240 -> keep c7@736
- Snail: drop f18@241 -> keep c9@740
- GlowingSnail: drop f18@242 -> keep c9@741
- Frog: drop f18@243 -> keep e26@744
- Duck: drop f18@244 -> keep c10@731
- Duck2: drop f18@245 -> keep c10@732
- DuckWhite: drop f18@246 -> keep c10@733
- DuckWhite2: drop f18@247 -> keep c10@734
- ScorpionBlack: drop f18@248 -> keep d39@818
- Scorpion: drop f18@249 -> keep d39@819
- TruffleWorm: drop f18@252 -> keep c8@738
- TruffleWormDigger: drop f18@253 -> keep c8@739
- Grasshopper: drop f18@254 -> keep d39@820
- ChatteringTeethBomb: drop f18@255 -> keep d39@821
- CultistArcherBlue: drop f18@256 -> keep d39@822
- CultistArcherWhite: drop f18@257 -> keep d39@823
- ArmedZombie: drop f18@258 -> keep d40@430

## f42 — 1 owned, 1 renumber, 8 drop

| Kind | old | new | cs_id | net_id |
|---|---|---|---|---|
| Tumbleweed | 354 | **546** | None | 546 |

DROP duplicate ports (keep canonical owner's row+wiring):
- Vulture: drop f42@355 -> keep d34@278
- Antlion: drop f42@356 -> keep g8@746
- Mummy: drop f42@357 -> keep d35@702
- DarkMummy: drop f42@358 -> keep d35@703
- BloodMummy: drop f42@359 -> keep d42@630
- LightMummy: drop f42@360 -> keep d35@704
- WalkingAntlion: drop f42@361 -> keep d42@580
- FlyingAntlion: drop f42@362 -> keep c4@581

## f47 — 1 owned, 0 renumber, 0 drop (KEEP receipt)

## g10 — 11 owned, 0 renumber, 0 drop (KEEP receipt)

## g5 — 2 owned, 0 renumber, 0 drop (KEEP receipt)

## g62 — 0 owned, 0 renumber, 2 drop

DROP duplicate ports (keep canonical owner's row+wiring):
- GoblinShark: drop g62@620 -> keep d42@620
- Shimmerfly: drop g62@677 -> keep c7@677

## g8 — 18 owned, 0 renumber, 17 drop

DROP duplicate ports (keep canonical owner's row+wiring):
- Vulture: drop g8@354 -> keep d34@278
- BlueJellyfish: drop g8@355 -> keep d34@280
- Pixie: drop g8@359 -> keep d35@700
- Wraith: drop g8@360 -> keep d35@705
- CursedHammer: drop g8@361 -> keep d35@706
- EnchantedSword: drop g8@362 -> keep d35@707
- Mimic: drop g8@363 -> keep d35@708
- GreenJellyfish: drop g8@364 -> keep d35@720
- Gastropod: drop g8@365 -> keep d35@728
- CursedSkull: drop g8@375 -> keep d34@267
- Demon: drop g8@376 -> keep d34@279
- GiantBat: drop g8@377 -> keep d35@710
- Slimer: drop g8@378 -> keep d35@727
- IlluminantBat: drop g8@379 -> keep d35@729
- FlyingSnake: drop g8@384 -> keep d37@360
- Squid: drop g8@385 -> keep d37@358
- Harpy: drop g8@386 -> keep d34@275
