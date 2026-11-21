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

### c4 - 5 owned, 1 renumber, 8 drop

| Kind | old `to_u16`/`from_u16` | new id | cs_id |
|---|---|---|---|
| Arapaima | 237 | **762** | 157 |

Keep as pushed: CrimsonGoldfish@465, GiantFlyingAntlion@509, Pufferfish@688, Orca@692.

DROP duplicate ports (keep canonical owner's row+wiring):
- CorruptGoldfish: drop c4@229 -> keep d34@277
- BloodFeeder: drop c4@241 -> keep d37@373
- FlyingFish: drop c4@354 -> keep d37@359
- FlyingAntlion: drop c4@581 -> keep d42@581 (renumbered from 357)
- EyeballFlyingFish: drop c4@587 -> keep d42@587 (renumbered from 363)
- GoldGoldfish: drop c4@592 -> keep d42@592 (renumbered from 366)
- Pupfish: drop c4@607 -> keep d42@607 (renumbered from 381)
- Dolphin: drop c4@615 -> keep d42@615 (renumbered from 389)

### c7 - 4 owned, 2 renumber, 0 drop

| Kind | old `to_u16`/`from_u16` | new id | cs_id |
|---|---|---|---|
| Firefly | 355 | **735** | 355 |
| LightningBug | 358 | **736** | 358 |

Keep as pushed: Lavafly@654, Shimmerfly@677.

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

### c9 - 3 owned, 2 renumber, 1 drop

| Kind | old `to_u16`/`from_u16` | new id | cs_id |
|---|---|---|---|
| Snail | 359 | **740** | 359 |
| GlowingSnail | 360 | **741** | 360 |

Keep as pushed: MagmaSnail@655.

DROP duplicate ports (keep canonical owner's row+wiring):
- Shimmerfly: drop c9@677 -> keep c7@677

### d120 - 4 owned, 4 renumber, 1 drop

| Kind | old `to_u16`/`from_u16` | new id | cs_id |
|---|---|---|---|
| SantaClaus | 241 | **763** | 142 |
| Truffle | 242 | **764** | 160 |
| Steampunker | 243 | **765** | 178 |
| Cyborg | 244 | **766** | 209 |

DROP duplicate ports (keep canonical owner's row+wiring):
- BartenderUnconscious: drop d120@245 -> keep d42@579 (renumbered from 355)

### d125 - 2 owned, 2 renumber, 0 drop

| Kind | old `to_u16`/`from_u16` | new id | cs_id |
|---|---|---|---|
| EmpressOfLight | 229 | **636** | (fold/sentinel) |
| PrismaticLacewing | 237 | **742** | (fold/sentinel) |

### d23 - 1 owned, 1 renumber, 5 drop

| Kind | old `to_u16`/`from_u16` | new id | cs_id |
|---|---|---|---|
| AngryTrapper | 357 | **743** | (fold/sentinel) |

DROP duplicate ports (keep canonical owner's row+wiring):
- FungiBulb: drop d23@259 -> keep d37@385
- GiantFungiBulb: drop d23@260 -> keep d37@386
- ManEater: drop d23@354 -> keep d34@269
- Snatcher: drop d23@355 -> keep d34@276
- Clinger: drop d23@356 -> keep d35@718 (renumbered from 449)

### d33 - 7 owned, 0 renumber, 0 drop

### d34 - 15 owned, 0 renumber, 0 drop

### d35 - 33 owned, 33 renumber, 0 drop

| Kind | old `to_u16`/`from_u16` | new id | cs_id |
|---|---|---|---|
| GoblinScout | 429 | **698** | 73 |
| Bird | 430 | **699** | 74 |
| Pixie | 431 | **700** | 75 |
| ArmoredSkeleton | 432 | **701** | 77 |
| Mummy | 433 | **702** | 78 |
| DarkMummy | 434 | **703** | 79 |
| LightMummy | 435 | **704** | 80 |
| Wraith | 436 | **705** | 82 |
| CursedHammer | 437 | **706** | 83 |
| EnchantedSword | 438 | **707** | 84 |
| Mimic | 439 | **708** | 85 |
| Unicorn | 440 | **709** | 86 |
| GiantBat | 441 | **710** | 93 |
| Corruptor | 442 | **711** | 94 |
| DiggerHead | 443 | **712** | 95 |
| DiggerBody | 444 | **713** | (fold/sentinel) |
| DiggerTail | 445 | **714** | (fold/sentinel) |
| SeekerHead | 446 | **715** | 98 |
| SeekerBody | 447 | **716** | (fold/sentinel) |
| SeekerTail | 448 | **717** | (fold/sentinel) |
| Clinger | 449 | **718** | 101 |
| AnglerFish | 450 | **719** | 102 |
| GreenJellyfish | 451 | **720** | 103 |
| Werewolf | 452 | **721** | 104 |
| Clown | 453 | **722** | 109 |
| SkeletonArcher | 454 | **723** | 110 |
| VileSpit | 455 | **724** | (fold/sentinel) |
| TheHungryII | 456 | **725** | 116 |
| ChaosElemental | 457 | **726** | 120 |
| Slimer | 458 | **727** | 121 |
| Gastropod | 459 | **728** | 122 |
| IlluminantBat | 460 | **729** | 137 |
| PossessedArmor | 461 | **730** | 140 |

### d37 - 46 owned, 0 renumber, 0 drop

### d38 - 35 owned, 0 renumber, 0 drop

### d40 - 55 owned, 0 renumber, 1 drop

DROP duplicate ports (keep canonical owner's row+wiring):
- CrimsonGoldfish: drop d40@465 -> keep c4@465

### d42 - 44 owned, 44 renumber, 7 drop

| Kind | old `to_u16`/`from_u16` | new id | cs_id |
|---|---|---|---|
| DD2LightningBug | 354 | **578** | 578 |
| BartenderUnconscious | 355 | **579** | 579 |
| WalkingAntlion | 356 | **580** | 580 |
| FlyingAntlion | 357 | **581** | 581 |
| LarvaeAntlion | 358 | **582** | 582 |
| FairyCritterPink | 359 | **583** | 583 |
| FairyCritterGreen | 360 | **584** | 584 |
| FairyCritterBlue | 361 | **585** | 585 |
| ZombieMerman | 362 | **586** | 586 |
| EyeballFlyingFish | 363 | **587** | 587 |
| TorchZombie | 364 | **590** | 590 |
| ArmedTorchZombie | 365 | **591** | 591 |
| GoldGoldfish | 366 | **592** | 592 |
| GoldGoldfishWalker | 367 | **593** | 593 |
| WindyBalloon | 368 | **594** | 594 |
| Seagull | 376 | **602** | 602 |
| Seagull2 | 377 | **603** | 603 |
| LadyBug | 378 | **604** | 604 |
| GoldLadyBug | 379 | **605** | 605 |
| Maggot | 380 | **606** | 606 |
| Pupfish | 381 | **607** | 607 |
| Grebe | 382 | **608** | 608 |
| Grebe2 | 383 | **609** | 609 |
| Rat | 384 | **610** | 610 |
| Owl | 385 | **611** | 611 |
| WaterStrider | 386 | **612** | 612 |
| GoldWaterStrider | 387 | **613** | 613 |
| ExplosiveBunny | 388 | **614** | 614 |
| Dolphin | 389 | **615** | 615 |
| Turtle | 390 | **616** | 616 |
| TurtleJungle | 391 | **617** | 617 |
| BloodNautilus | 392 | **618** | 618 |
| BloodSquid | 393 | **619** | 619 |
| GoblinShark | 394 | **620** | 620 |
| BloodEelHead | 395 | **621** | 621 |
| BloodEelBody | 396 | **622** | 622 |
| BloodEelTail | 397 | **623** | 623 |
| Gnome | 398 | **624** | 624 |
| SeaTurtle | 399 | **625** | 625 |
| Seahorse | 400 | **626** | 626 |
| GoldSeahorse | 401 | **627** | 627 |
| Dandelion | 402 | **628** | 628 |
| IceMimic | 403 | **629** | 629 |
| BloodMummy | 404 | **630** | 630 |

DROP duplicate ports (keep canonical owner's row+wiring):
- BlackDragonfly: drop d42@369 -> keep d33@595
- BlueDragonfly: drop d42@370 -> keep d33@596
- GreenDragonfly: drop d42@371 -> keep d33@597
- OrangeDragonfly: drop d42@372 -> keep d33@598
- RedDragonfly: drop d42@373 -> keep d33@599
- YellowDragonfly: drop d42@374 -> keep d33@600
- GoldDragonfly: drop d42@375 -> keep d33@601

### e26 - 1 owned, 1 renumber, 3 drop

| Kind | old `to_u16`/`from_u16` | new id | cs_id |
|---|---|---|---|
| Frog | 361 | **744** | 361 |

DROP duplicate ports (keep canonical owner's row+wiring):
- Duck: drop e26@362 -> keep c10@731 (renumbered from 362)
- LadyBug: drop e26@604 -> keep d42@604 (renumbered from 378)
- GoldLadyBug: drop e26@605 -> keep d42@605 (renumbered from 379)

### g8 - 17 owned, 17 renumber, 17 drop

| Kind | old `to_u16`/`from_u16` | new id | cs_id |
|---|---|---|---|
| PinkJellyfish | 356 | **745** | 64 |
| Antlion | 357 | **746** | 69 |
| SpikeBall | 358 | **747** | 70 |
| WallCreeper | 366 | **748** | 164 |
| WallCreeperWall | 367 | **749** | 165 |
| IceElemental | 368 | **750** | 169 |
| Herpling | 369 | **751** | 174 |
| Derpling | 370 | **752** | 177 |
| CrimsonAxe | 371 | **753** | 179 |
| FloatyGross | 372 | **754** | 182 |
| LostGirl | 373 | **755** | 195 |
| Nymph | 374 | **756** | 196 |
| IceBat | 380 | **757** | 150 |
| Lavabat | 381 | **758** | 151 |
| GiantFlyingFox | 382 | **759** | 152 |
| VampireBat | 383 | **760** | 158 |
| RedDevil | 387 | **761** | 156 |

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
