//! `NPC.HitEffect` — the per-NPC gore/dust table, wave C (W4-f10): the
//! THIRD batch of `Gore.NewGore*` death sites in source order, starting
//! where the second batch's nominal ~40 ends (`NPC.cs:82206` onward).
//!
//! The pool itself is already ported (`tr_core::gore::GorePool::new_gore`,
//! S1-E-4; the `GoreID.Sets` tables in `tr_core::gore_id`); this module is
//! the per-NPC spawn table the pool feeds. W4-f10 (code-only mode): the
//! module + witnesses land first; the dispatch claim (the `StrikeNPC_Inner`
//! `:81482`/`:81488` pair and the AI-arm `HitEffect()` calls) is the wiring
//! wave, the `npc_ai_style39.rs` / wave-A (`npc_hit_effect_a.rs`) precedent.
//!
//! ## Claim boundary (the pushed-claims probe)
//!
//! * wave A (`W4-d55`, `ec78d152`): the FIRST 30 sites, `NPC.cs:81660-81941`
//!   (the party-hat rows, the 686/594 bursts, daybreak, 668 Deerclops, the
//!   636 Empress first gore). PUSHED.
//! * wave B (`W4-e5-gore-sites-2`): the SECOND ~40 sites, `NPC.cs:81942-82202`
//!   (the 636 tail, 657/658-660/667, the DD2 goblin/bomber/javelinst rows and
//!   the 551 Betsy NON-flipped branch). Nominal range; no pushed commits at
//!   probe time.
//! * wave C (THIS module, `W4-f10-gore-sites-3`): the remainder from
//!   `NPC.cs:82206` — the 551 Betsy FLIPPED branch, the DD2 kobold/drakin/
//!   wither-beast/wyvern/skeleton/ogre/dark-mage rows, the 578 lightning bug,
//!   and the blood-moon sea monsters (620/619/618).
//! * `W4-g16` (projectile) / `W4-g17` (world) own the non-NPC files.
//!
//! ## Scope — this batch (source order, death halves only)
//!
//! | sites | C# lines | arm |
//! | --- | --- | --- |
//! | 71-80 | `:82206-82215` | the 551 Betsy FLIPPED branch (`spriteDirection == -1`) |
//! | 81-85 | `:82237-82241` | the 572/573 DD2 Kobold Walker death (5 gores) |
//! | 86-90 | `:82262-82266` | the 574/575 DD2 Kobold Flyer death (5 gores) |
//! | 91-95 | `:82285-82289` | the 570/571 DD2 Drakin death (5 gores, per-tier base) |
//! | 96-100 | `:82307-82311` | the 568/569 DD2 Wither Beast death (5 gores) |
//! | 101-107 | `:82338-82344` | the 558/559/560 DD2 Wyvern death (7 gores, per-tier base) |
//! | 108-110 | `:82362-82369` | the 566/567 DD2 Skeleton death: head + the first two coin-gates |
//! | 111-114 | `:82373-82385` | the 566/567 DD2 Skeleton death: the remaining four coin-gates |
//! | 115-120 | `:82406-82413` | the 576/577 DD2 Ogre FLIPPED death (6 gores, Next(5) type gate + π rotations) |
//! | 121-126 | `:82417-82422` | the 576/577 DD2 Ogre NON-flipped death (6 gores, Next(5) type gate) |
//! | 127-132 | `:82442-82447` | the 564/565 DD2 Dark Mage death: the first six gores (base 1009 ladder) |
//! | 133-137 | `:82448-82452` | the 564/565 DD2 Dark Mage death: the +6/+10/+7/+8/+9 tail |
//! | 138-141 | `:82470-82473` | the 578 DD2 Lightning Bug death (4 gores 999/998/998/997) |
//! | 142-145 | `:82496-82499` | the 620 Goblin Shark death (4 gores, base 1183 ladder, 150-dust burst) |
//! | 146-149 | `:82522-82525` | the 619 Blood Squid death (4 gores, base 1179 ladder, 150-dust fadeIn burst) |
//! | 150-155 | `:82548-82553` | the 618 Blood Nautilus death: the first six gores (base 1172, 200-dust burst) |
//! | 156-161 | `:82554-82559` | the 618 Blood Nautilus death: the +4/+4/+5/+5/+6/+6 tail |
//! | 162-165 | `:82600-82603` | the 541 Sand Elemental death (4 gores 960/961/963/962, tinted 60-dust burst) |
//! | 166-168 | `:82650-82652` | the 546 Tumbleweed death (3 gores 964/965/966, tinted 40-dust + 20 type-54 burst) |
//! | 169-174 | `:82676` | the 437 Cultist Tablet `dmg == 9999` burst (6 gores 728-733, RotatedByRandom velocity) |
//! | 175-177 | `:82743` | the 513-515 Tomb Crawler death (1 gore each, type-513+822, alpha copy) |
//! | 178-180 | `:82762` | the 510-512 Dune Splicer death (1 gore each, type-510+819, alpha copy) |
//! | 181-183 | `:82781` | the 621-623 Blood Eel death (1 gore each, type-621+1187, alpha copy, type-5 dust) |
//! | 184-188 | `:82816-82820` | the 518/416 Solar Spearman/Drakomire Rider death (5 gores, gated type-6 dust) |
//! | 189-194 | `:82854-82859` | the 419 Solar Solenian death (6 gores, gated type-6 dust) |
//! | 195-197 | `:82881` | the 412-414 Solar Crawltipede death (1 gore each, type-412+828) |
//! | 198-201 | `:82912-82915` | the 415 Solar Flare death (4 gores at Center, vel x0.8/x0.9) |
//! | 202-207 | `:82949-82954` | the 417 Solar Sroller death: the first six gores (837/838x3/839x2) |
//! | 208-209 | `:82955-82956` | the 417 Solar Sroller death: the 840/840 tail |
//! | 210-214 | `:82987-82991` | the 418 Solar Corite death (5 gores at Center, vel x0.8/x0.9) |
//! | 215-217 | `:83072-83074` | the 426 Nebula Floater death (3 gores, SelectRandom dust + NPC-428 spawn seam) |
//! | 218-221 | `:83119-83122` | the 427 Nebula Soldier death (4 gores at Center, vel x0.8/x0.9) |
//! | 222-226 | `:83156-83160` | the 429 Nebula Predictor death (5 gores, projectile-578 spawn seam) |
//! | 227-233 | `:83238-83244` | the 425 Nebula Beast death (7 gores, gated type-229 dust) |
//! | 234-235 | `:83270-83271` | the 428 Nebula Brain death (2 gores, vel x0.8, Next(2) type-229/240 dust) |
//! | 236-240 | `:83305-83309` | the 424 Vortex Soldier death (5 gores, gated type-242 dust) |
//! | 241-243 | `:83340-83342` | the 423 Vortex Hornet death (3 gores at Center, vel x0.8/x0.9) |
//! | 244-246 | `:83373-83375` | the 421 Vortex Larva death (3 gores at Center, vel x0.8/x0.9) |
//! | 247-249 | `:83406-83408` | the 420 Vortex Rider death (3 gores at Center, vel x0.8/x0.9) |
//! | 250-252 | `:83503-83505` | the 409 Stardust Spider death (3 gores at Center, vel x0.8/x0.9) |
//! | 253-258 | `:83557-83562` | the 407 Stardust Cell death (6 gores, Next(3)!=0 type-17 dust gate) |
//! | 259-263 | `:83657-83661` | the 411 Stardust Dragon body death (5 gores, Next(2)-gated type-229 dust) |
//! | 264-274 | `:83670-83676` | the 491 Flying Dutchman death: the head block (2 fixed + 3x3 loop gores) |
//! | 275-284 | `:83680-83684` | the 491 Flying Dutchman death: the RandomVector2 fan, iterations -1/0 |
//! | 285-289 | `:83680-83684` | the 491 Flying Dutchman death: the RandomVector2 fan, iteration 1 |
//! | 290-293 | `:83688-83689` | the 491 Flying Dutchman death: the 855 loop (2x2 gores) |
//! | 294-302 | `:83693-83695` | the 491 Flying Dutchman death: the 854/853/856 loop (3x3 gores) |
//! | 303-312 | `:83699` | the 491 Flying Dutchman death: the 861/862 scatter burst, gores 0-9 |
//! | 313-322 | `:83699` | the 491 Flying Dutchman death: the 861/862 scatter burst, gores 10-19 |
//! | 323-332 | `:83699` | the 491 Flying Dutchman death: the 861/862 scatter burst, gores 20-29 |
//! | 333-342 | `:83734` | the 492 Flying Dutchman (cannon) death: 7-dust type-31 burst + 10-gore Next(61,64) scatter |
//! | 343-348 | `:83789-83794` | the 524-527 Desert Ghoul (normal/corrupt/crimson/hallow) death (6 gores, per-tier base) |
//! | 349-352 | `:83818-83821` | the 528-529 Desert Lamia (light/dark) death (4 gores, per-tier head) |
//! | 353-357 | `:83839-83843` | the 530/531 Desert Scorpion death (5 gores 883-887, alpha-100 type-250 dust) |
//! | 358-361 | `:83861-83864` | the 532 Desert Beast death (4 gores 888-891, alpha-100 type-5 dust) |
//! | 362-366 | `:83904-83908` | the 460 Butcher death (5 gores 643/645/644/646/646) |
//! | 367-371 | `:83926-83930` | the 477 Mothron death (5 gores 687-691, type-237 dust) |
//! | 372-374 | `:83948-83950` | the 479 Mothron Spawn death (3 gores 681-683, type-237 dust) |
//! | 375-378 | `:83969-83972` | the 478 Mothron Egg death (4 gores, Next(2) dust + Next(3) type draws) |
//! | 379-383 | `:83990-83994` | the 466 Psycho death (5 gores 656/657/659/658/658) |
//! | 384-388 | `:84012-84016` | the 468 DrManFly death (5 gores 662/663/663/664/664) |
//! | 389-393 | `:84034-84038` | the 469 ThePossessed death (5 gores 665/666/666/667/667) |
//! | 394-398 | `:84056-84060` | the 463 Nailhead death (5 gores 653/654/654/655/655) |
//! | 399-403 | `:84078-84082` | the 461 CreatureFromTheDeep death (5 gores 647/648/648/649/649) |
//! | 404-408 | `:84100-84104` | the 462 Fritz death (5 gores 650/651/651/652/652) |
//! | 409-411 | `:84122-84124` | the 379/438 Cultist Archer/Devote death (3 gores 900/901/901) |
//! | 412-418+ | `:84153-84159` | the 439 Cultist Boss death (Next(1,4) bone loop + 4 fixed gores 902/903/901/901) |
//! | 419-421+ | `:84175` | the 440 Cultist Boss Clone death (10-dust type-16 burst + Next(1,4) bone loop) |
//! | 422-424+ | `:84202` | the 454-459 Cultist Dragon death (10-dust type-16 burst + Next(1,4) bone loop) |
//! | 425-428 | `:84223-84226` | the 372/373 Sharkron death (4 gores at Center, vel x0.8/x0.8/x0.9/x1) |
//! | 429-434 | `:84273-84278` | the 370 Duke Fishron death (6 gores, direction-spread anchors) |
//! | 435-438 | `:84296-84299` | the 381 BrainScrambler death (4 gores 587/590/588/589, type-227 dust) |
//! | 439-441 | `:84317-84319` | the 382/390 RayGunner/ScutlixRider death (3 gores 591/592/593) |
//! | 442-444 | `:84337-84339` | the 383 MartianOfficer death (3 gores 594/595/596) |
//! | 445-447 | `:84377-84379` | the 385 GrayGrunt death (3 gores 597/598/599, type-227 dust) |
//! | 448-450 | `:84397-84399` | the 386 MartianEngineer death (3 gores 600/601/602) |
//! | 451-453 | `:84420-84422` | the 387 MartianTurret death (3 gores 603/603/604, type-226 scale-0.5 dust) |
//! | 454-458 | `:84443-84447` | the 520 MartianWalker death (5 gores 851/852/848/849/850, +80 Y tail) |
//! | 459-461 | `:84465-84467` | the 389 GigaZapper death (3 gores 904/905/906, type-227 dust) |
//! | 462-463 | `:84498-84499` | the 388 MartianDrone death (2 gores 605/606, ai[0]==3 rotation-vector velocity) |
//! | 464-465 | `:84530-84531` | the 399 MartianProbe death (2 gores 898/899, same rotation-vector shape) |
//! | 466-468 | `:84549-84551` | the 391 Scutlix death (3 gores 607/608/609, width/2 X offset) |
//! | 469-476 | `:84575-84588` | the 392/393/394/395 Martian Saucer family death (395 core 6 gores, 393/394 one each; 392 none) |
//! | 477-480 | `:84609-84612` | the 482 GraniteGolem death (4 gores 718-721; 483 in gate but none) |
//! | 481-484 | `:84631-84634` | the 631 RockGolem death (4 gores 1210-1213, type-1 dust) |
//! | 485-489+ | `:84738` | the party Blue Slime (type 1, ai[1]==1345) confetti: Next(276,283) loop (5 living / 15 dead) |
//! | 490-494 | `:84810-84814` | the 305-314 Scarecrow death (5 gores 441-445, type-194 scale-1.1 dust) |
//! | 495-503 | `:84834` | the 327 Pumpking death (9-gore loop 476-484, vel x0, type-189 scale-1.1 dust) |
//! | 504-508 | `:84861-84865` | the 345 IceQueen death (5 gores 513/514/514/515/516, vel x0) |
//! | 509-515 | `:84855` | the 325 MourningWood death (7-gore loop 469-475, vel x0, type-191 scale-1.1 dust) |
//! | 516-524 | `:84884-84888` | the 344 Everscream death (two loops: 508-512 then 509-512, vel x0, type-196 dust) |
//! | 525 | `:84941` | the 302 SlimeMasked death (1 gore 440, type-192 gray alpha-100 dust) |
//! | 526-531 | `:84968-84973` | the 245 Golem body death: the 6 random-position 368/370 gores |
//! | 532-538 | `:84974-84980` | the 245 Golem body death: the fixed tail 360/361/362/363/364/367/366 |
//! | 539-545 | `:84984-84990` | the 249 GolemHeadFree death (4 random 368/370 + fixed 365/363/362) |
//! | 546-548 | `:84994-84996` | the 247/248 Golem fist death (3 gores 369/371/370) |
//! | 549 | `:85060` | the 243 IceGolem death (1 gore 359, 200-dust type-161 burst) |
//! | 550 | `:85042` | the 225 UmbrellaSlime death (1 gore 314, blue alpha-100 type-4 dust) |
//! | 551-552 | `:85133-85137` | the 604 LadyBug death (2 Next(2)-gated gores 1148/1149) |
//! | 553-554 | `:85157-85161` | the 669 Stinkbug death (2 Next(2)-gated gores 1275/1276) |
//! | 555-556 | `:85271-85272` | the 359 Snail hit/death (2 gores 547/548 EVERY hit; dust death-only) |
//! | 557-558 | `:85292-85293` | the 360 GlowingSnail hit/death (2 gores 549/550, same shape) |
//! | 559 | `:85313` | the 655 MagmaSnail hit/death (1 gore 1256, same shape) |
//! | 560 | `:85334` | the 154 IceTortoise death (1 gore 180, type-67 vel-scaled dust) |
//! | 561 | `:85451` | the 121 Slimer death (1 gore 94 + alpha copy, NPC-81 spawn seam; 81 spawns babies, no gore) |
//! | 562 | `:85684` | the 50 KingSlime death (1 gore 734 at Center anchor, baby-slimes spawn seam) |
//! | 563-566 | `:85718-85721` | the 153 GiantTortoise death (4 gores 177/178/179/179) |
//! | 567-570 | `:85737-85740` | the 177 Derpling death (4 gores 234/235/236/236) |
//! | 571-572 | `:85757-85758` | the 494/495 Crawdad death (2 gores, per-type offset (type-494)*2) |
//! | 573 | `:85775` | the 496/497 GiantShelly death (1 gore 739+(type-496)) |
//! | 574-576 | `:85793-85795` | the 498-506 Salamander death (3 gores, per-type offset type-498) |
//! | 577-586 | `:85813-85839` | the bat family 49/51/93/150/152/226 death (per-type gore lists) |
//! | 587-589 | `:85856-85858` | the 611/689 Owl/OwlMimic death (3 gores 1164/1164/1165) |
//! | 590-593 | `:85874-85877` | the 624 Gnome death (4 gores 1190-1193) |
//! | 594-596 | `:85893-85895` | the 628 Dandelion death: the first three gores (1205/1206/1206) |
//! | 597 | `:85896` | the 628 Dandelion death: the 1207 tail |
//! | 598-599 | `:85918-85919` | the 646-652 GemBunny death (2 gores, per-type offset (type-646)*2) |
//! | 600 | `:85940` | the 639-645 GemSquirrel death (1 gore 1227+(type-639)) |
//! | 601-610 | `:85958-85981` | the critter family: bunny group (46/303/337/540), crab 67, goldfish 55/230 |
//! | 611-620 | `:85985-86006` | the critter family: dolphin 615, bird 74, birds 297/298, scarlet macaw 671 |
//! | 621-629 | `:86010-86024` | the critter family: blue macaw 672, toucan 673, yellow cockatiel 674 |
//! | 630-638 | `:86028-86054` | the critter family: gray cockatiel 675, squirrel 299/538, mouse 300, rat 610, seahorse 626, anglerfish 102 |
//! | 639-646 | `:86058-86075` | the critter family: flying fish 224, eyeball flying fish 587, turtle 616 |
//! | 647-654 | `:86079-86089` | the critter family: jungle turtle 617, sea turtle 625 (completes the block) |
//! | 655-663 | `:86108-86125` | the penguin family 148/149/168/470 death (per-type gore pairs/triple) |
//! | 664-665 | `:86142-86143` | the 361 Frog death (2 gores 551/552) |
//! | 666-667 | `:86187-86188` | the 607 Pupfish death (2 gores 1152/1153) |
//! | 668-673 | `:86206-86214` | the 688 Pufferfish death (ai[2]==1 ? 4 gores : 2 gores) |
//! | 674-675 | `:86287-86288` | the 602/603 Seagull death (2 gores 1150/1151) |
//! | 676-681 | `:86233-86241` | the 366/367 Scorpion death (3 gores each, type-203 dust) |
//! | 682-684 | `:86307-86309` | the 608/609 Grebe death (3 gores 1154/1155/1156) |
//! | 685-690 | `:86260-86268` | the 362-365 Duck family death (3 gores each, per-type) |
//! | 691-694 | `:86326-86329` | the 348/349 Nutcracker death: the first four gores (528-531) |
//! | 695 | `:86330` | the 348/349 Nutcracker death: the 532 tail |
//! | 696-700 | `:86346-86350` | the 351 Krampus death (5 gores 538-542) |
//! | 701-704 | `:86366-86369` | the 350 death (4 gores 533-536) |
//! | 705-712 | `:86387-86406` | the corrupt/crimson critter family 47/57/58/464/465 death (per-type) |
//! | 713-722 | `:86425-86446` | the crimson monster family 173/174/181/182/268 death (per-type, alpha dust) |
//! | 723-732 | `:86465-86486` | the zombie family 190/191/192/193/194 death (2 gores each, +14 X offset) |
//! | 733-738 | `:86490-86501` | the demon-eye family 317/318/2 death (completes the block) |
//! | 739-740 | `:86518-86519` | the 157 Arapaima death (2 gores 186/187, +14 X offset) |
//! | 741-743 | `:86532-86542` | the 133 WanderingEye hit/death (hit-half <50% latch gore 1 + 2 death gores 155) |
//! | 744-749 | `:86568-86576` | the antlion 69/580 death (type-250 dust, per-type gores) |
//! | 750-758 | `:86580-86591` | the antlion 581/508 death (5 and 4 gores respectively) |
//! | 759-765 | `:86595-86604` | the antlion 509/582 death (completes the antlion block) |
//! | 766-768 | `:86621-86623` | the 61 death (3 gores 86/87/88, +14 X offset) |
//! | 769-770 | `:86639-86640` | the 301 Raven death (2 gores 438/439, +14 X offset) |
//! | 771-772 | `:86656-86657` | the 252 Parrot death (2 gores 373/374) |
//! | 773-777 | `:86675-86679` | the 489 BloodZombie death (5 gores 722/723/723/724/724) |
//! | 778-782 | `:86683-86687` | the 586 ZombieMerman death (5 gores 1106/1107/1107/1108/1108) |
//! | 783-787 | `:86704-86708` | the 534 DemonTaxCollector death (5 gores 894/895/895/896/896) |
//! | 788-792 | `:86724-86728` | the 490 Drippler death (5 gores, two Next(3) type draws) |
//! | 793-800 | `:86746-86756` | the shark family 65/692 death (4 gores each, vel x0.8, +14 X tails) |
//! | 801-808 | `:86760-86770` | the sandshark 542/543 death (4 gores each) |
//! | 809-816 | `:86774-86784` | the sandshark 544/545 death (completes the shark family) |
//! | 817-821 | `:86801-86805` | the 195/196 LostGirl/Nymph death (5 gores 254/255/255/256/256) |
//! | 822-826 | `:86842-86846` | the 206 IcyMerman death (5 gores 273/274/274/275/275, type-137 dust) |
//! | 827-832 | `:86821-86826` | the 198/199 Lihzahrd death (6 gores, all tails at +20 Y) |
//! | 833-837 | `:86862-86866` | the 342 GingerbreadMan death (5 gores 500/501/501/502/502, type-0 dust) |
//! | 838-843 | `:86884-86896` | the 338-340 ZombieElf death (per-type head + shared 497/498/499 tails) |
//! | 844-847 | `:86912-86915` | the 343 Yeti death (4 gores 503/504/505/506, distinct +20 pair) |
//! | 848-852 | `:86931-86935` | the 691 MossZombie death (5 gores 1391/1392/1392/1393/1393) |
//! | 853-862 | `:86982-86995` | the zombie block: 104 Werewolf + 632 MaggotZombie (5-gore ladders, 632 spawn seam) |
//!
//! Later commits extend the table through `:82559`. The hit (`life > 0`)
//! halves are dmg-scaled dust loops with NO gore — they ride the wiring
//! wave (wave A's "the death halves here use the flat counts" convention).
//!
//! ## RNG contract (the strike's stream)
//!
//! `HitEffect` runs inside `StrikeNPC_Inner` (`NPC.cs:81488`), under the
//! strike caller's `Main.rand` swap (the projectile/melee update's stage),
//! NOT the NPC pass's — the `npc_ai_style37.rs::probe_launch_on_hit` note.
//! The caller hands that stream in; every draw below hits it in C# order:
//!
//! * 551 Betsy death: the 20-dust type-5 loop first (each `NewDust`'s 6
//!   default-block draws + the type-5 tweak, `Dust.cs:153-252`, pool-side),
//!   then the 10 gores draw ONLY the `NewGore` kicks (2 each, `Gore.cs:1327-1328`,
//!   pool-side) — the types/anchors are data, no arm-level draw.
//!
//! The dust loops run through `DustPool::new_dust` (S1-E-1) on the SAME
//! stream, so a cull refusal or a pressure-band gate consumes exactly the
//! C#'s draws (the pool owns them). The post-spawn writes land on the
//! returned slot, the sentinel included — the C# writes `Main.gore[600]`
//! too (the `npc_ai_style56.rs` precedent).
//!
//! ## Documented drops / stand-ins (this wave)
//!
//! * `!this.active` gate (`:81656`): the caller's — the fn runs on an
//!   active NPC (the strike/death path's own check).
//! * The `this.type` axis rides an explicit `npc_type` parameter (the
//!   wave-A `HitEffectInputs` / `Style39Type` precedent — the DD2/blood-moon
//!   kinds have no `NpcKind` rows yet).
//! * The 551 Betsy NON-flipped branch (`:8193-82202`, `spriteDirection != -1`)
//!   is wave B's; this module ports ONLY the flipped (`== -1`) branch and
//!   gates on `sprite_dir == -1`. The shared 20-dust death loop is ported
//!   here; the merge keeps ONE copy (keep-one).
//! * `DD2Event.CheckProgress` (`:82176`) is a hit-half (`life > 0`) event
//!   seam — the wiring wave's, no draw here.
//! * `netMode` guards: single-player port (the `netMode != 1` arms run,
//!   the `== 2` arms drop), the pool's own convention.
//!
//! See docs/ANALYSIS.md "The NPC HitEffect gore table (W4-f10, wave C)".
#![allow(dead_code)] // wiring deferred (code-only mode) — the strike-path claim is the follow-up round

use crate::npc::Npc;
use glam::Vec2;
use tr_core::gore::GorePool;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::{UnifiedRandom, World};

/// `Entity.Center` (`Entity.cs:50-60`) — `(pos.X + width/2f, pos.Y + height/2f)`.
fn center(npc: &Npc) -> Vec2 {
    let (w, h) = npc.size();
    Vec2::new(npc.x + w / 2.0, npc.y + h / 2.0)
}

/// `Entity.Right` (`Entity.cs:104-114`) — `(pos.X + width, pos.Y + height/2f)`.
fn right_x(npc: &Npc) -> f32 {
    let (w, _) = npc.size();
    npc.x + w
}

/// `Entity.Top` (`Entity.cs:86-96`) — `(pos.X + width/2f, pos.Y)`.
fn top(npc: &Npc) -> Vec2 {
    let (w, _) = npc.size();
    Vec2::new(npc.x + w / 2.0, npc.y)
}

/// `Utils.SelectRandom<T>` (`Utils.cs:2459-2462`) — `choices[Next(len)]`.
fn select_random(rng: &mut UnifiedRandom, choices: &[i32]) -> i32 {
    choices[rng.next_max(choices.len() as i32) as usize]
}

/// `Utils.RandomVector2(Main.rand, min, max)` (`Utils.cs:2431-2434`): TWO
/// `NextDouble` draws, X then Y, each `(max-min)*NextDouble + min`.
fn random_vector2(rng: &mut UnifiedRandom, min: f32, max: f32) -> Vec2 {
    let x = (max - min) * rng.next_double() as f32 + min;
    let y = (max - min) * rng.next_double() as f32 + min;
    Vec2::new(x, y)
}

/// The side-effects the wiring caller owes (the roster seams this module
/// cannot reach — the wave-A `HitEffectOutcome` precedent).
#[derive(Default)]
pub struct HitEffectOutcomeC {
    /// The 426 Nebula Floater's brain-spawn seam (`NPC.cs:83075-83083`):
    /// when `CountNPCS(428) + CountNPCS(427) + CountNPCS(426)*3 < 20`, spawn
    /// 3 NPC-428s at Center with velocity
    /// `-UnitY.RotatedByRandom(2π) * Next(3,6) - UnitY*2`. `Some((cx,cy))` =
    /// the caller resolves the count gate + the spawns; the per-spawn
    /// RotatedByRandom/Next draws ride the wiring wave's roster pass.
    pub nebula_brain_spawn: Option<(f32, f32)>,
    /// The 429 Nebula Predictor's projectile-spawn seam (`NPC.cs:83161-83204`):
    /// a tile-scan loop (up to 100 tries, two `Next` draws per try) around
    /// the target player's tile that spawns ONE projectile-578 at a valid
    /// cell. `Some((cx,cy))` = the NPC Center the scan anchors on; the
    /// target-player read, the tile probes and the per-try draws are the
    /// wiring wave's (the roster/projectile seams).
    pub nebula_predictor_shot: Option<(f32, f32)>,
    /// The 491 Flying Dutchman's projectile-spawn seam (`NPC.cs:83713-83720`):
    /// 4 projectile-594s at Center, each velocity
    /// `RotatedBy((num191<2 ? +1 : -1) * (π/8 + π/4*NextFloat), (0, -NextFloat*0.5 - 0.5)) * 6`.
    /// `Some((cx,cy))` = the Center; the per-spawn NextFloat draws + the
    /// projectile spawns are the wiring wave's.
    pub dutchman_death_shots: Option<(f32, f32)>,
    /// The 121 Slimer's NPC-81 spawn seam (`NPC.cs:85447-85455`): on death
    /// the Slimer spawns one CorruptSlime (81) at `(pos.X + width/2,
    /// pos.Y + height)` with `SetDefaults(-2)` and the parent's velocity.
    /// `Some((x,y))` = the spawn anchor; the NPC spawn + the netUpdate are
    /// the wiring wave's. The 81 CorruptSlime's own baby-spawn seam
    /// (`:85457-85474`, scale>=1) is likewise the wiring wave's (no gore).
    pub slimer_spawn: Option<(f32, f32)>,
    /// The 50 KingSlime's baby-slimes spawn seam (`NPC.cs:85689-85702`):
    /// `Next(4)+4` (4-7) NPC-1s at random positions inside the body with
    /// random velocities. `Some((x,y,w,h))` = the position/box the spawns
    /// randomize within; the count draw + the per-spawn draws + the NPC
    /// spawns are the wiring wave's.
    pub king_slime_babies: Option<(f32, f32, f32, f32)>,
    /// The 632 MaggotZombie's maggot-spawn seam (`NPC.cs:86997-87006`):
    /// `Next(2)+2` (2-3) NPC-606s at `(pos.X + width/2, pos.Y +
    /// Next(height/2) + 10)` with per-spawn velocity draws. `Some((x,y,w,h))`
    /// = the position/box; the count draw + the per-spawn draws + the NPC
    /// spawns are the wiring wave's.
    pub maggot_zombie_spawns: Option<(f32, f32, f32, f32)>,
}

/// `Utils.RotatedBy` (`Utils.cs:1675-1684`), centre default (0,0): the
/// cos/sin are FLOAT casts of the double radians, the products float.
fn rotated_by(v: Vec2, radians: f64) -> Vec2 {
    let num = radians.cos() as f32;
    let num2 = radians.sin() as f32;
    Vec2::new(v.x * num - v.y * num2, v.x * num2 + v.y * num)
}

/// `Utils.RotatedByRandom` (`Utils.cs:1697-1700`): TWO `NextDouble` draws
/// (`NextDouble()*max - NextDouble()*max`), then `RotatedBy`. The draws hit
/// the strike stream in that order.
fn rotated_by_random(v: Vec2, max_radians: f64, rng: &mut UnifiedRandom) -> Vec2 {
    let a = rng.next_double() * max_radians;
    let b = rng.next_double() * max_radians;
    rotated_by(v, a - b)
}

/// The death-half dust loop shared by the DD2 rows (`NPC.cs` per-arm
/// `for (… < 20f) Dust.NewDust(position, width, height, Type, hitDirection, -1f)`).
/// `count` is the flat death count; the hit (`life > 0`) half's dmg-scaled
/// bound is the wiring wave's. The pool owns every draw.
fn death_dust_loop(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    rng: &mut UnifiedRandom,
    world: &World,
    dust_type: i32,
    hit_direction: f32,
    count: i32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    for _ in 0..count {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, dust_type, hit_direction, -1.0, 0,
            Color::default(), 1.0,
        );
    }
}

/// The 551 Betsy FLIPPED death branch (`NPC.cs:82206-82215`, sites 71-80):
/// the 10 body gores when `spriteDirection == -1`. The types/anchors are
/// data; each spawn draws only the `NewGore` kicks (pool-side). The shared
/// 20-dust type-5 death loop (`:82187-82190`) runs FIRST (the C# order).
fn betsy_flipped_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    // `:82187-82190` — the 20-dust type-5 death burst (shared with the
    // non-flipped branch; keep-one at merge).
    death_dust_loop(npc, dust, dust_ctx, rng, world, 5, hit_direction, 20);
    // `:82206-82215` — the flipped anchors, velocity = base.velocity,
    // scale = this.scale.
    let pos = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos + Vec2::new(176.0, 54.0), vel, 1079, s); // :82206
    gore.new_gore(rng, world, pos + Vec2::new(158.0, 90.0), vel, 1080, s); // :82207
    gore.new_gore(rng, world, pos + Vec2::new(128.0, 94.0), vel, 1080, s); // :82208
    gore.new_gore(rng, world, pos + Vec2::new(0.0, 0.0), vel, 1081, s); // :82209
    gore.new_gore(rng, world, pos + Vec2::new(60.0, 12.0), vel, 1082, s); // :82210
    gore.new_gore(rng, world, pos + Vec2::new(88.0, 18.0), vel, 1083, s); // :82211
    gore.new_gore(rng, world, pos + Vec2::new(86.0, 26.0), vel, 1084, s); // :82212
    gore.new_gore(rng, world, pos + Vec2::new(40.0, 16.0), vel, 1085, s); // :82213
    gore.new_gore(rng, world, pos + Vec2::new(20.0, 10.0), vel, 1085, s); // :82214
    gore.new_gore(rng, world, pos + Vec2::new(142.0, 36.0), vel, 1086, s); // :82215
}

/// The 572/573 DD2 Kobold Walker death (`NPC.cs:82232-82241`, sites 81-85):
/// the 20-dust type-5 burst, then 5 gores 993/996/995/994/994 at
/// `Center - (10,10)` (+0/+20/+20/+34/+34 Y). The two tracked-sound stops
/// (`:82230-82231`, `localAI[0]`/`localAI[1]`) draw NO random numbers — the
/// tracked-sound seam is the wiring wave's (the S1-D-4b convention).
fn kobold_walker_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    death_dust_loop(npc, dust, dust_ctx, rng, world, 5, hit_direction, 20);
    // `:82236` — vector6 = Center - (10,10).
    let v6 = center(npc) - Vec2::new(10.0, 10.0);
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, v6, vel, 993, s); // :82237
    gore.new_gore(rng, world, Vec2::new(v6.x, v6.y + 20.0), vel, 996, s); // :82238
    gore.new_gore(rng, world, Vec2::new(v6.x, v6.y + 20.0), vel, 995, s); // :82239
    gore.new_gore(rng, world, Vec2::new(v6.x, v6.y + 34.0), vel, 994, s); // :82240
    gore.new_gore(rng, world, Vec2::new(v6.x, v6.y + 34.0), vel, 994, s); // :82241
}

/// The 574/575 DD2 Kobold Flyer death (`NPC.cs:82256-82266`, sites 86-90):
/// the 20-dust type-5 burst, then 5 gores 1050/1051/1052/1053/1054 at
/// `Center - (30,10)` (+10/+30/+30/+0/+0 Y). The tracked-sound stop
/// (`:82255`, `localAI[2]`) draws no random numbers (the wiring seam).
fn kobold_flyer_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    death_dust_loop(npc, dust, dust_ctx, rng, world, 5, hit_direction, 20);
    // `:82260-82261` — num58 = 1050; vector7 = Center - (30,10).
    let v7 = center(npc) - Vec2::new(30.0, 10.0);
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, Vec2::new(v7.x, v7.y + 10.0), vel, 1050, s); // :82262
    gore.new_gore(rng, world, Vec2::new(v7.x, v7.y + 30.0), vel, 1051, s); // :82263
    gore.new_gore(rng, world, Vec2::new(v7.x, v7.y + 30.0), vel, 1052, s); // :82264
    gore.new_gore(rng, world, Vec2::new(v7.x, v7.y), vel, 1053, s); // :82265
    gore.new_gore(rng, world, Vec2::new(v7.x, v7.y), vel, 1054, s); // :82266
}

/// The 570/571 DD2 Drakin death (`NPC.cs:82280-82289`, sites 91-95): the
/// 20-dust type-5 burst, then 5 gores at the per-tier base
/// (`num61 = type == 570 ? 1075 : 979`, `:82284`) with the type order
/// `num61 / +2 / +2 / +3 / +1` at the +0/+20/+20/+20/+34 Y ladder.
fn drakin_death(
    npc: &Npc,
    npc_type: i32,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    death_dust_loop(npc, dust, dust_ctx, rng, world, 5, hit_direction, 20);
    // `:82284` — the per-tier base.
    let num61 = if npc_type == 570 { 1075 } else { 979 };
    let pos = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, num61, s); // :82285
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, num61 + 2, s); // :82286
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, num61 + 2, s); // :82287
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, num61 + 3, s); // :82288
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, num61 + 1, s); // :82289
}

/// The 568/569 DD2 Wither Beast death (`NPC.cs:82302-82311`, sites 96-100):
/// the 20-dust type-5 burst, then the 5 fixed gores 1020/1021/1023/1022/1022
/// at the +0/+20/+20/+34/+34 Y ladder (the type order is NOT a base+offset
/// ladder — 1023 precedes the 1022 pair, `:82309-82311`).
fn wither_beast_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    death_dust_loop(npc, dust, dust_ctx, rng, world, 5, hit_direction, 20);
    let pos = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, 1020, s); // :82307
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 1021, s); // :82308
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 1023, s); // :82309
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, 1022, s); // :82310
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, 1022, s); // :82311
}

/// The 558/559/560 DD2 Wyvern death (`NPC.cs:82325-82344`, sites 101-107):
/// the 20-dust type-5 burst, then 7 gores at the per-tier base
/// (`num66`: 558→974, 559→1040, 560→1045, `:82329-82337`) with the type
/// order `+2 / base / base / +4 / +1 / +3 / +3` at the +0/+10/+10/+10/+24/+24/+24
/// Y ladder.
fn wyvern_death(
    npc: &Npc,
    npc_type: i32,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    death_dust_loop(npc, dust, dust_ctx, rng, world, 5, hit_direction, 20);
    // `:82329-82337` — the per-tier base (two independent ifs, not else-if).
    let mut num66 = 974;
    if npc_type == 559 {
        num66 = 1040;
    }
    if npc_type == 560 {
        num66 = 1045;
    }
    let pos = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, num66 + 2, s); // :82338
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 10.0), vel, num66, s); // :82339
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 10.0), vel, num66, s); // :82340
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 10.0), vel, num66 + 4, s); // :82341
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 24.0), vel, num66 + 1, s); // :82342
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 24.0), vel, num66 + 3, s); // :82343
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 24.0), vel, num66 + 3, s); // :82344
}

/// The 566/567 DD2 Skeleton death (`NPC.cs:82358-82386`, sites 108-114):
/// the 20-dust type-236 burst, the 970 head, then SIX independent
/// `Main.rand.Next(2) == 0` coin-gates (972/972/973/973/971/971). Each gate
/// draws its `Next(2)` BEFORE the conditional spawn — a refused gate still
/// consumes the draw. Sites 108-114 (the head + all six gates).
fn skeleton_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    // `:82358-82361` — the dust is type 236 (NOT 5).
    death_dust_loop(npc, dust, dust_ctx, rng, world, 236, hit_direction, 20);
    let pos = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, 970, s); // :82362
    // `:82363-82366` — gate 1: Next(2) then the conditional 972.
    if rng.next_max(2) == 0 {
        gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 972, s); // :82365
    }
    // `:82367-82370` — gate 2: Next(2) then the conditional 972.
    if rng.next_max(2) == 0 {
        gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 972, s); // :82369
    }
    // `:82371-82374` — gate 3: Next(2) then the conditional 973.
    if rng.next_max(2) == 0 {
        gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 973, s); // :82373
    }
    // `:82375-82378` — gate 4: Next(2) then the conditional 973.
    if rng.next_max(2) == 0 {
        gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 973, s); // :82377
    }
    // `:82379-82382` — gate 5: Next(2) then the conditional 971 at +34.
    if rng.next_max(2) == 0 {
        gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, 971, s); // :82381
    }
    // `:82383-82386` — gate 6: Next(2) then the conditional 971 at +34.
    if rng.next_max(2) == 0 {
        gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, 971, s); // :82385
    }
}

/// The 576/577 DD2 Ogre FLIPPED death (`NPC.cs:82404-82413`, sites 115-120):
/// the 20-dust type-273 burst, then the 6 flipped gores. The head type is
/// `1005 + (Next(5) == 0 ? 1 : 0)` (`:82406`, the draw rides the strike
/// stream BEFORE the spawn); gores 1001 (`:82408`) and 1002 (`:82410`) get
/// `rotation = π` post-writes on the returned slot (sentinel included).
/// The non-flipped branch (`:82417-82422`) is the NEXT batch.
fn ogre_flipped_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    // `:82400-82403` — the dust is type 273.
    death_dust_loop(npc, dust, dust_ctx, rng, world, 273, hit_direction, 20);
    let pos = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    // `:82406` — the head's Next(5) type gate draws first.
    let head = 1005 + if rng.next_max(5) == 0 { 1 } else { 0 };
    gore.new_gore(rng, world, pos, vel, head, s);
    gore.new_gore(rng, world, pos + Vec2::new(18.0, 28.0), vel, 1000, s); // :82407
    // `:82408-82409` — the 1001 gore, rotation = π on the slot.
    let num71 = gore.new_gore(rng, world, pos + Vec2::new(-40.0, 70.0), vel, 1001, s);
    gore.slot_mut(num71).rotation = std::f32::consts::PI;
    // `:82410-82411` — the 1002 gore, rotation = π on the slot.
    let num71 = gore.new_gore(rng, world, pos + Vec2::new(4.0, 28.0), vel, 1002, s);
    gore.slot_mut(num71).rotation = std::f32::consts::PI;
    gore.new_gore(rng, world, pos + Vec2::new(36.0, 90.0), vel, 1003, s); // :82412
    gore.new_gore(rng, world, pos + Vec2::new(32.0, 90.0), vel, 1004, s); // :82413
}

/// The 576/577 DD2 Ogre NON-flipped death (`NPC.cs:82417-82422`, sites
/// 121-126): the 20-dust type-273 burst, then the 6 non-flipped gores. The
/// head type is `1005 + (Next(5) == 0 ? 1 : 0)` (`:82417`); unlike the
/// flipped branch, the 1001/1002 gores get NO rotation write here.
fn ogre_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    death_dust_loop(npc, dust, dust_ctx, rng, world, 273, hit_direction, 20);
    let pos = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    // `:82417` — the head's Next(5) type gate draws first.
    let head = 1005 + if rng.next_max(5) == 0 { 1 } else { 0 };
    gore.new_gore(rng, world, pos + Vec2::new(48.0, 0.0), vel, head, s);
    gore.new_gore(rng, world, pos + Vec2::new(22.0, 28.0), vel, 1000, s); // :82418
    gore.new_gore(rng, world, pos + Vec2::new(48.0, 70.0), vel, 1001, s); // :82419
    gore.new_gore(rng, world, pos + Vec2::new(4.0, 28.0), vel, 1002, s); // :82420
    gore.new_gore(rng, world, pos + Vec2::new(36.0, 90.0), vel, 1003, s); // :82421
    gore.new_gore(rng, world, pos + Vec2::new(32.0, 90.0), vel, 1004, s); // :82422
}

/// The 564/565 DD2 Dark Mage death (`NPC.cs:82437-82452`, sites 127-137):
/// the 20-dust type-5 burst, then 11 gores at base `num74 = 1009`
/// (`:82441`). The type order is `base/+1/+2/+3/+4/+5/+6/+10/+7/+8/+9`
/// (the +10 = 1019 gore precedes the +7/+8/+9 trio, `:82449-82452`) at the
/// Y ladder +0/+15/+15/+15/+35/+35/+35/+35/+25/+25/+25, the +4 gore shifted
/// -20 in X (`:82446`).
fn dark_mage_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    death_dust_loop(npc, dust, dust_ctx, rng, world, 5, hit_direction, 20);
    // `:82441` — num74 = 1009.
    let num74 = 1009;
    let pos = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, num74, s); // :82442
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 15.0), vel, num74 + 1, s); // :82443
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 15.0), vel, num74 + 2, s); // :82444
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 15.0), vel, num74 + 3, s); // :82445
    gore.new_gore(rng, world, Vec2::new(pos.x - 20.0, pos.y + 35.0), vel, num74 + 4, s); // :82446
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 35.0), vel, num74 + 5, s); // :82447
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 35.0), vel, num74 + 6, s); // :82448
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 35.0), vel, num74 + 10, s); // :82449
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 25.0), vel, num74 + 7, s); // :82450
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 25.0), vel, num74 + 8, s); // :82451
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 25.0), vel, num74 + 9, s); // :82452
}

/// The 578 DD2 Lightning Bug death (`NPC.cs:82466-82473`, sites 138-141):
/// the 20-dust type-5 burst, then the 4 gores 999/998/998/997 at the
/// +0/+10/+10/+10 Y ladder (the 999 head at position, the rest at +10).
fn lightning_bug_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    death_dust_loop(npc, dust, dust_ctx, rng, world, 5, hit_direction, 20);
    let pos = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, 999, s); // :82470
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 10.0), vel, 998, s); // :82471
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 10.0), vel, 998, s); // :82472
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 10.0), vel, 997, s); // :82473
}

/// The 620 Goblin Shark death (`NPC.cs:82489-82499`, sites 142-145): the
/// 150-dust `NewDustDirect` type-5 burst (velocity ×2.5, scale = 1.7 — the
/// post-writes draw nothing), then 4 gores at base `num79 = 1183`
/// (`:82495`) with the type order `base/+2/+3/+1` at the +0/+20/+20/+34 Y
/// ladder. The dust runs through `new_dust_direct` (the C# `NewDustDirect`,
/// `Dust.cs:65-73`) on the strike stream.
fn goblin_shark_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:82489-82494` — the 150-dust direct burst.
    for _ in 0..150 {
        let idx = dust.new_dust_direct(
            rng, dust_ctx, world, pos, w as i32, h as i32, 5, hit_direction, -1.0, 0,
            Color::default(), 1.0,
        );
        let d = dust.slot_mut(idx);
        d.velocity *= 2.5;
        d.scale = 1.7;
    }
    // `:82495` — num79 = 1183.
    let num79 = 1183;
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, num79, s); // :82496
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, num79 + 2, s); // :82497
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, num79 + 3, s); // :82498
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, num79 + 1, s); // :82499
}

/// The 619 Blood Squid death (`NPC.cs:82515-82525`, sites 146-149): the
/// 150-dust `NewDustDirect` type-5 burst (velocity ×2.5, fadeIn = 1.7 — the
/// post-writes draw nothing), then 4 gores at base `num82 = 1179`
/// (`:82521`) with the type order `base/+2/+3/+1` at the +0/+20/+20/+34 Y
/// ladder. Same shape as the Goblin Shark, but the dust post-write is
/// `fadeIn` (not `scale`) and the base is 1179.
fn blood_squid_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:82515-82520` — the 150-dust direct burst (fadeIn, not scale).
    for _ in 0..150 {
        let idx = dust.new_dust_direct(
            rng, dust_ctx, world, pos, w as i32, h as i32, 5, hit_direction, -1.0, 0,
            Color::default(), 1.0,
        );
        let d = dust.slot_mut(idx);
        d.velocity *= 2.5;
        d.fade_in = 1.7;
    }
    // `:82521` — num82 = 1179.
    let num82 = 1179;
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, num82, s); // :82522
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, num82 + 2, s); // :82523
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, num82 + 3, s); // :82524
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, num82 + 1, s); // :82525
}

/// The 618 Blood Nautilus death (`NPC.cs:82541-82559`, sites 150-161): the
/// 200-dust `NewDustDirect` type-5 burst (velocity ×2.5, scale = 2.5 — the
/// DEATH scale; the hit half uses 2.0, `:82536`), then 12 gores at base
/// `num85 = 1172` (`:82547`). The first two type pairs alternate the
/// `Right.X - 30` anchor with `position`; the rest sit at `position`.
/// Sites 150-161 (all twelve gores).
fn blood_nautilus_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:82541-82546` — the 200-dust direct burst (death scale 2.5).
    for _ in 0..200 {
        let idx = dust.new_dust_direct(
            rng, dust_ctx, world, pos, w as i32, h as i32, 5, hit_direction, -1.0, 0,
            Color::default(), 1.0,
        );
        let d = dust.slot_mut(idx);
        d.velocity *= 2.5;
        d.scale = 2.5;
    }
    // `:82547` — num85 = 1172.
    let num85 = 1172;
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    let rx = right_x(npc) - 30.0;
    gore.new_gore(rng, world, Vec2::new(rx, npc.y), vel, num85, s); // :82548
    gore.new_gore(rng, world, pos, vel, num85, s); // :82549
    gore.new_gore(rng, world, Vec2::new(rx, npc.y), vel, num85 + 1, s); // :82550
    gore.new_gore(rng, world, pos, vel, num85 + 1, s); // :82551
    gore.new_gore(rng, world, pos, vel, num85 + 2, s); // :82552
    gore.new_gore(rng, world, pos, vel, num85 + 3, s); // :82553
    gore.new_gore(rng, world, pos, vel, num85 + 4, s); // :82554
    gore.new_gore(rng, world, pos, vel, num85 + 4, s); // :82555
    gore.new_gore(rng, world, pos, vel, num85 + 5, s); // :82556
    gore.new_gore(rng, world, pos, vel, num85 + 5, s); // :82557
    gore.new_gore(rng, world, pos, vel, num85 + 6, s); // :82558
    gore.new_gore(rng, world, pos, vel, num85 + 6, s); // :82559
}

/// The 541 Sand Elemental death (`NPC.cs:82585-82603`, sites 162-165): the
/// 30-iteration burst, each spawning TWO type-268 dusts tinted
/// `new Color(222,108,48) * 0.7f` with noGravity, scale 1.5, fadeIn 0.7,
/// velocity ×3 (the post-writes draw nothing), then the 4 gores
/// 960/961/963/962 at the +0/+20/+20/+40 Y ladder (the 963 precedes the
/// 962, `:82602-82603`). The tint rides the XNA `Color * float` truncation
/// (`ui::color::mul`): (155,75,33,178).
fn sand_elemental_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    _hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:82564` — the tint `new Color(222,108,48) * 0.7f` (alpha 255 * 0.7).
    let t = crate::ui::color::mul([222, 108, 48, 255], 0.7);
    let tint = Color::new(t[0], t[1], t[2], t[3]);
    // `:82585-82599` — 30 iterations x 2 type-268 dusts. The C# NewDust
    // here passes NO hitDirection (the 4-arg overload, SpeedX/Y default 0).
    for _ in 0..30 {
        for _ in 0..2 {
            let idx = dust.new_dust(
                rng, dust_ctx, world, pos, w as i32, h as i32, 268, 0.0, 0.0, 0,
                Color::default(), 1.0,
            );
            let d = dust.slot_mut(idx);
            d.color = tint;
            d.no_gravity = true;
            d.scale = 1.5;
            d.fade_in = 0.7;
            d.velocity *= 3.0;
        }
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, 960, s); // :82600
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 961, s); // :82601
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 963, s); // :82602
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 40.0), vel, 962, s); // :82603
}

/// The 546 Tumbleweed death (`NPC.cs:82629-82652`, sites 166-168): the
/// 20-iteration burst, each spawning TWO type-268 dusts tinted
/// `new Color(150,108,78) * 0.7f` (noGravity, scale 1.5, fadeIn 0.7,
/// velocity ×3), then a 20-dust type-54 burst (alpha 50, scale 1.5,
/// velocity ×2, noGravity), then the 3 gores 964/965/966 at the +0/+20/+20
/// Y ladder. The tint rides the XNA truncation: (105,75,54,178).
fn tumbleweed_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    _hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:82608` — the tint `new Color(150,108,78) * 0.7f`.
    let t = crate::ui::color::mul([150, 108, 78, 255], 0.7);
    let tint = Color::new(t[0], t[1], t[2], t[3]);
    // `:82629-82643` — 20 iterations x 2 type-268 dusts (4-arg NewDust).
    for _ in 0..20 {
        for _ in 0..2 {
            let idx = dust.new_dust(
                rng, dust_ctx, world, pos, w as i32, h as i32, 268, 0.0, 0.0, 0,
                Color::default(), 1.0,
            );
            let d = dust.slot_mut(idx);
            d.color = tint;
            d.no_gravity = true;
            d.scale = 1.5;
            d.fade_in = 0.7;
            d.velocity *= 3.0;
        }
    }
    // `:82644-82649` — the 20-dust type-54 burst (alpha 50, scale 1.5).
    for _ in 0..20 {
        let idx = dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 54, 0.0, 0.0, 50,
            Color::default(), 1.5,
        );
        let d = dust.slot_mut(idx);
        d.velocity *= 2.0;
        d.no_gravity = true;
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, 964, s); // :82650
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 965, s); // :82651
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 966, s); // :82652
}

/// The 437 Cultist Tablet `dmg == 9999` burst (`NPC.cs:82672-82677`, sites
/// 169-174): 6 gores 728-733 at `Center - (10,10)`, each with velocity
/// `Vector2.UnitY.RotatedByRandom(2π) * 5f`. The `RotatedByRandom` draws TWO
/// `NextDouble` per gore BEFORE the spawn's kicks; the types are the loop
/// index (no type draw). Gated on `dmg == 9999.0` (the C# exact-double
/// compare) — NOT a `life <= 0` gate.
fn cultist_tablet_burst(
    npc: &Npc,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    let anchor = center(npc) - Vec2::new(10.0, 10.0);
    for num94 in 0..6 {
        // `:82676` — UnitY.RotatedByRandom(2π) * 5 (two NextDouble draws).
        let vel = rotated_by_random(Vec2::new(0.0, 1.0), 6.2831854820251465, rng) * 5.0;
        gore.new_gore(rng, world, anchor, vel, 728 + num94, 1.0);
    }
}

/// The 513-515 Tomb Crawler death (`NPC.cs:82739-82744`, sites 175-177):
/// the 50-dust type-250 burst (SpeedY -2, alpha/color/scale from the NPC),
/// then ONE gore of type `this.type - 513 + 822` at position with
/// `Main.gore[num].alpha = this.alpha` post-write. The three segments
/// (513/514/515) map to gores 822/823/824.
///
/// `this.color` stand-in: the worm kinds set NO color in `SetDefaults`
/// (`NPC.cs:8526` `this.color = default`), so the dust color is
/// `Color::default()` (all-zero); the port's `Npc` carries `alpha` but no
/// color field (the wave-A `HitEffectInputs` stand-in convention).
fn tomb_crawler_death(
    npc: &Npc,
    npc_type: i32,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    let alpha = npc.alpha as i32;
    // `:82739-82742` — the 50-dust type-250 burst (SpeedY -2).
    for _ in 0..50 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 250, hit_direction, -2.0, alpha,
            Color::default(), npc.scale,
        );
    }
    // `:82743-82744` — the single gore + the alpha copy.
    let ty = npc_type - 513 + 822;
    let idx = gore.new_gore(rng, world, pos, Vec2::new(npc.vx, npc.vy), ty, 1.0);
    gore.slot_mut(idx).alpha = alpha;
}

/// The 510-512 Dune Splicer death (`NPC.cs:82758-82763`, sites 178-180):
/// the 50-dust type-250 burst (SpeedY -2), then ONE gore of type
/// `this.type - 510 + 819` with the alpha copy. The three segments
/// (510/511/512) map to gores 819/820/821. Same shape as the Tomb Crawler,
/// different base/type offset.
fn dune_splicer_death(
    npc: &Npc,
    npc_type: i32,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    let alpha = npc.alpha as i32;
    // `:82758-82761` — the 50-dust type-250 burst (SpeedY -2).
    for _ in 0..50 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 250, hit_direction, -2.0, alpha,
            Color::default(), npc.scale,
        );
    }
    // `:82762-82763` — the single gore + the alpha copy.
    let ty = npc_type - 510 + 819;
    let idx = gore.new_gore(rng, world, pos, Vec2::new(npc.vx, npc.vy), ty, 1.0);
    gore.slot_mut(idx).alpha = alpha;
}

/// The 621-623 Blood Eel death (`NPC.cs:82777-82782`, sites 181-183): the
/// 50-dust type-5 burst (SpeedY -2, alpha/color/scale from the NPC), then
/// ONE gore of type `this.type - 621 + 1187` with the alpha copy. The three
/// segments (621/622/623) map to gores 1187/1188/1189. Same shape as the
/// other worm arms, but the dust is type 5 (not 250). The 621-623 family
/// sets `alpha = 255` in SetDefaults (`NPC.cs:16985`), carried by `npc.alpha`.
fn blood_eel_death(
    npc: &Npc,
    npc_type: i32,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    let alpha = npc.alpha as i32;
    // `:82777-82780` — the 50-dust type-5 burst (SpeedY -2).
    for _ in 0..50 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 5, hit_direction, -2.0, alpha,
            Color::default(), npc.scale,
        );
    }
    // `:82781-82782` — the single gore + the alpha copy.
    let ty = npc_type - 621 + 1187;
    let idx = gore.new_gore(rng, world, pos, Vec2::new(npc.vx, npc.vy), ty, 1.0);
    gore.slot_mut(idx).alpha = alpha;
}

/// The 518/416 Solar Spearman / Drakomire Rider death (`NPC.cs:82804-82820`,
/// sites 184-188): the 20-iteration burst, each a type-241 dust then a
/// `Main.rand.Next(3) == 0`-gated type-6 dust (noGravity, scale 1.5,
/// fadeIn 1, velocity ×3); then the 5 gores 844/845/847/835/836 at the
/// +0/+20/+20/+20/+20 Y ladder. The death gate is `Next(3)` (the hit half
/// uses `Next(4)`, `:82792`).
fn solar_spearman_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:82804-82815` — 20 iterations: type-241 dust + the Next(3)-gated type-6.
    for _ in 0..20 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 241, hit_direction, -1.0, 0,
            Color::default(), 1.0,
        );
        // `:82807` — the death gate is Next(3) == 0 (draws before the spawn).
        if rng.next_max(3) == 0 {
            let idx = dust.new_dust(
                rng, dust_ctx, world, pos, w as i32, h as i32, 6, 0.0, 0.0, 0,
                Color::default(), 1.0,
            );
            let d = dust.slot_mut(idx);
            d.no_gravity = true;
            d.scale = 1.5;
            d.fade_in = 1.0;
            d.velocity *= 3.0;
        }
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, 844, s); // :82816
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 845, s); // :82817
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 847, s); // :82818
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 835, s); // :82819
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 836, s); // :82820
}

/// The 419 Solar Solenian death (`NPC.cs:82842-82859`, sites 189-194): the
/// 20-iteration burst (type-241 dust + `Next(3) == 0`-gated type-6 dust,
/// same as the Spearman), then the 6 gores 844/845/846/846/847/847 at the
/// +0/+20/+20/+20/+20/+20 Y ladder.
fn solar_solenian_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:82842-82853` — 20 iterations: type-241 + the Next(3)-gated type-6.
    for _ in 0..20 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 241, hit_direction, -1.0, 0,
            Color::default(), 1.0,
        );
        if rng.next_max(3) == 0 {
            let idx = dust.new_dust(
                rng, dust_ctx, world, pos, w as i32, h as i32, 6, 0.0, 0.0, 0,
                Color::default(), 1.0,
            );
            let d = dust.slot_mut(idx);
            d.no_gravity = true;
            d.scale = 1.5;
            d.fade_in = 1.0;
            d.velocity *= 3.0;
        }
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, 844, s); // :82854
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 845, s); // :82855
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 846, s); // :82856
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 846, s); // :82857
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 847, s); // :82858
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 847, s); // :82859
}

/// The 412-414 Solar Crawltipede death (`NPC.cs:82873-82881`, sites
/// 195-197): the 6-dust type-241 burst (SpeedX `2.5*hitDirection`, SpeedY
/// -2.5) then the 2-dust type-6 burst (alpha 100, Color.Transparent, scale
/// 2), then ONE gore of type `this.type - 412 + 828` (412/413/414 ->
/// 828/829/830). No alpha copy here (unlike the worm arms).
fn solar_crawltipede_death(
    npc: &Npc,
    npc_type: i32,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:82873-82876` — the 6-dust type-241 burst.
    for _ in 0..6 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 241, 2.5 * hit_direction, -2.5, 0,
            Color::default(), 1.0,
        );
    }
    // `:82877-82880` — the 2-dust type-6 burst (alpha 100, transparent, scale 2).
    for _ in 0..2 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 6, 2.5 * hit_direction, -2.5, 100,
            Color::TRANSPARENT, 2.0,
        );
    }
    // `:82881` — the single gore (no alpha copy).
    let ty = npc_type - 412 + 828;
    gore.new_gore(rng, world, pos, Vec2::new(npc.vx, npc.vy), ty, 1.0);
}

/// The 415 Solar Flare death (`NPC.cs:82903-82915`, sites 198-201): the
/// 40-iteration burst, each a type-241 dust AND an UNGATED type-6 dust
/// (noGravity, scale 1.5, fadeIn 1, velocity ×3 — no Next gate here), then
/// the 4 gores 831/832/833/834 at Center with velocity ×0.8 (831/832) and
/// ×0.9 (833/834). Scale defaults to 1 (the 3-arg NewGore).
fn solar_flare_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:82903-82911` — 40 iterations: type-241 + ungated type-6.
    for _ in 0..40 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 241, hit_direction, -1.0, 0,
            Color::default(), 1.0,
        );
        let idx = dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 6, 0.0, 0.0, 0,
            Color::default(), 1.0,
        );
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.scale = 1.5;
        d.fade_in = 1.0;
        d.velocity *= 3.0;
    }
    let c = center(npc);
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, c, vel * 0.8, 831, 1.0); // :82912
    gore.new_gore(rng, world, c, vel * 0.8, 832, 1.0); // :82913
    gore.new_gore(rng, world, c, vel * 0.9, 833, 1.0); // :82914
    gore.new_gore(rng, world, c, vel * 0.9, 834, 1.0); // :82915
}

/// The 417 Solar Sroller death (`NPC.cs:82937-82956`, sites 202-209): the
/// 30-iteration burst (type-241 dust + `Next(2) == 0`-gated type-6 dust),
/// then the 8 gores 837/838/838/838/839/839/840/840 at the +0/+20/+38 Y
/// ladder. Sites 202-209 (all eight gores).
fn solar_sroller_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:82937-82948` — 30 iterations: type-241 + the Next(2)-gated type-6.
    for _ in 0..30 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 241, hit_direction, -1.0, 0,
            Color::default(), 1.0,
        );
        if rng.next_max(2) == 0 {
            let idx = dust.new_dust(
                rng, dust_ctx, world, pos, w as i32, h as i32, 6, 0.0, 0.0, 0,
                Color::default(), 1.0,
            );
            let d = dust.slot_mut(idx);
            d.no_gravity = true;
            d.scale = 1.5;
            d.fade_in = 1.0;
            d.velocity *= 3.0;
        }
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, 837, s); // :82949
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 838, s); // :82950
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 838, s); // :82951
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 838, s); // :82952
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 839, s); // :82953
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 839, s); // :82954
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 38.0), vel, 840, s); // :82955
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 38.0), vel, 840, s); // :82956
}

/// The 418 Solar Corite death (`NPC.cs:82978-82991`, sites 210-214): the
/// 20-iteration burst (type-241 dust + UNGATED type-6 dust, noGravity,
/// scale 1.5, fadeIn 1, velocity ×3), then the 5 gores 841/842/842/843/843
/// at Center with velocity ×0.8 (841/842/842) and ×0.9 (843/843). Scale
/// defaults to 1. Same shape as the Solar Flare, different types/counts.
fn solar_corite_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:82978-82986` — 20 iterations: type-241 + ungated type-6.
    for _ in 0..20 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 241, hit_direction, -1.0, 0,
            Color::default(), 1.0,
        );
        let idx = dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 6, 0.0, 0.0, 0,
            Color::default(), 1.0,
        );
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.scale = 1.5;
        d.fade_in = 1.0;
        d.velocity *= 3.0;
    }
    let c = center(npc);
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, c, vel * 0.8, 841, 1.0); // :82987
    gore.new_gore(rng, world, c, vel * 0.8, 842, 1.0); // :82988
    gore.new_gore(rng, world, c, vel * 0.8, 842, 1.0); // :82989
    gore.new_gore(rng, world, c, vel * 0.9, 843, 1.0); // :82990
    gore.new_gore(rng, world, c, vel * 0.9, 843, 1.0); // :82991
}

/// The 426 Nebula Floater death (`NPC.cs:83058-83083`, sites 215-217): the
/// 20-dust type-241 burst, then the 30-dust `SelectRandom(229,240,161)`
/// burst (noGravity, scale `1.25 + NextFloat`, fadeIn 0.25, velocity ×2,
/// noLight), then the 3 gores 799/800/801 at Top/Center/Center with
/// velocity ×0.7/×0.8/×0.9. The NPC-428 brain-spawn seam
/// (`:83075-83083`) is RETURNED ([`HitEffectOutcomeC::nebula_brain_spawn`])
/// — the roster count gate + the per-spawn draws are the wiring wave's.
fn nebula_floater_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
    outcome: &mut HitEffectOutcomeC,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:83058-83061` — the 20-dust type-241 burst.
    death_dust_loop(npc, dust, dust_ctx, rng, world, 241, hit_direction, 20);
    // `:83062-83071` — the 30-dust SelectRandom(229,240,161) burst.
    for _ in 0..30 {
        let ty = select_random(rng, &[229, 240, 161]);
        let idx = dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, ty, 0.0, 0.0, 0,
            Color::default(), 1.0,
        );
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.scale = 1.25 + rng.next_float();
        d.fade_in = 0.25;
        d.velocity *= 2.0;
        d.no_light = true;
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, top(npc), vel * 0.7, 799, 1.0); // :83072
    gore.new_gore(rng, world, center(npc), vel * 0.8, 800, 1.0); // :83073
    gore.new_gore(rng, world, center(npc), vel * 0.9, 801, 1.0); // :83074
    // `:83075-83083` — the brain-spawn seam (netMode != 1, single-player
    // runs); the count gate + the spawns are the caller's.
    let c = center(npc);
    outcome.nebula_brain_spawn = Some((c.x, c.y));
}

/// The 427 Nebula Soldier death (`NPC.cs:83105-83122`, sites 218-221): the
/// 10-dust type-241 burst, then the 20-dust `SelectRandom(229,229,240)`
/// burst (noGravity, scale `1.25 + NextFloat`, fadeIn 0.25, velocity ×3,
/// noLight), then the 4 gores 802/803/804/805 at Center with velocity
/// ×0.8/×0.9/×0.9/×0.8. No spawn seam here.
fn nebula_soldier_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:83105-83108` — the 10-dust type-241 burst.
    death_dust_loop(npc, dust, dust_ctx, rng, world, 241, hit_direction, 10);
    // `:83109-83118` — the 20-dust SelectRandom(229,229,240) burst.
    for _ in 0..20 {
        let ty = select_random(rng, &[229, 229, 240]);
        let idx = dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, ty, 0.0, 0.0, 0,
            Color::default(), 1.0,
        );
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.scale = 1.25 + rng.next_float();
        d.fade_in = 0.25;
        d.velocity *= 3.0;
        d.no_light = true;
    }
    let c = center(npc);
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, c, vel * 0.8, 802, 1.0); // :83119
    gore.new_gore(rng, world, c, vel * 0.9, 803, 1.0); // :83120
    gore.new_gore(rng, world, c, vel * 0.9, 804, 1.0); // :83121
    gore.new_gore(rng, world, c, vel * 0.8, 805, 1.0); // :83122
}

/// The 429 Nebula Predictor death (`NPC.cs:83144-83204`, sites 222-226):
/// the 20-iteration burst (type-241 dust + `Next(3) == 0`-gated type-229
/// dust, noGravity, scale 1.5, fadeIn 1, velocity ×3), then the 5 gores
/// 808/809/809/810/810 at the +0/+20/+20/+34/+34 Y ladder. The
/// projectile-578 tile-scan spawn seam (`:83161-83204`) is RETURNED
/// ([`HitEffectOutcomeC::nebula_predictor_shot`]) — the target-player read,
/// the tile probes and the per-try `Next` draws are the wiring wave's.
fn nebula_predictor_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
    outcome: &mut HitEffectOutcomeC,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:83144-83155` — 20 iterations: type-241 + the Next(3)-gated type-229.
    for _ in 0..20 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 241, hit_direction, -1.0, 0,
            Color::default(), 1.0,
        );
        if rng.next_max(3) == 0 {
            let idx = dust.new_dust(
                rng, dust_ctx, world, pos, w as i32, h as i32, 229, 0.0, 0.0, 0,
                Color::default(), 1.0,
            );
            let d = dust.slot_mut(idx);
            d.no_gravity = true;
            d.scale = 1.5;
            d.fade_in = 1.0;
            d.velocity *= 3.0;
        }
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, 808, s); // :83156
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 809, s); // :83157
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 809, s); // :83158
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, 810, s); // :83159
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, 810, s); // :83160
    // `:83161-83204` — the projectile-spawn seam (netMode != 1, single-player
    // runs); the scan + the spawn are the caller's.
    let c = center(npc);
    outcome.nebula_predictor_shot = Some((c.x, c.y));
}

/// The 425 Nebula Beast death (`NPC.cs:83226-83244`, sites 227-233): the
/// 20-iteration burst (type-241 dust + `Next(3) == 0`-gated type-229 dust,
/// noGravity, scale 1.5, fadeIn 1, velocity ×3), then the 7 gores
/// 796/797/798/809/809/810/810 — 796 at position, 797 at Center, 798/809/809
/// at +20 Y, 810/810 at +34 Y.
fn nebula_beast_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:83226-83237` — 20 iterations: type-241 + the Next(3)-gated type-229.
    for _ in 0..20 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 241, hit_direction, -1.0, 0,
            Color::default(), 1.0,
        );
        if rng.next_max(3) == 0 {
            let idx = dust.new_dust(
                rng, dust_ctx, world, pos, w as i32, h as i32, 229, 0.0, 0.0, 0,
                Color::default(), 1.0,
            );
            let d = dust.slot_mut(idx);
            d.no_gravity = true;
            d.scale = 1.5;
            d.fade_in = 1.0;
            d.velocity *= 3.0;
        }
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, 796, s); // :83238
    gore.new_gore(rng, world, center(npc), vel, 797, s); // :83239
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 798, s); // :83240
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 809, s); // :83241
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 809, s); // :83242
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, 810, s); // :83243
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, 810, s); // :83244
}

/// The 428 Nebula Brain death (`NPC.cs:83258-83271`, sites 234-235): the
/// 10-iteration burst, each a type-241 dust then a `Next(2) == 0` ? type-229
/// : type-240 dust (the gate picks the type, both branches spawn), then the
/// 2 gores 806/807 at position with velocity ×0.8. Scale defaults to 1.
fn nebula_brain_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:83258-83269` — 10 iterations: type-241 + the Next(2)-picked 229/240.
    for _ in 0..10 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 241, hit_direction, -1.0, 0,
            Color::default(), 1.0,
        );
        // `:83261-83268` — the gate picks the type; both branches spawn.
        let ty = if rng.next_max(2) == 0 { 229 } else { 240 };
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, ty, 0.0, 0.0, 0,
            Color::default(), 1.0,
        );
    }
    let vel = Vec2::new(npc.vx, npc.vy) * 0.8;
    gore.new_gore(rng, world, pos, vel, 806, 1.0); // :83270
    gore.new_gore(rng, world, pos, vel, 807, 1.0); // :83271
}

/// The Vortex death dust burst shared by the 424/423/421/420 arms
/// (`NPC.cs:83293-83304` et al.): `count` iterations, each a type-249 dust
/// then a type-242 dust (noGravity, scale 1.5, fadeIn 1, velocity ×3). When
/// `gate` is `Some(n)` the type-242 spawn is `Next(n) == 0`-gated (the draw
/// still fires); when `None` it is ungated. The 424 arm gates `Next(3)`;
/// the 423/421/420 death halves are ungated.
fn vortex_death_dust(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
    count: i32,
    gate: Option<i32>,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    for _ in 0..count {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 249, hit_direction, -1.0, 0,
            Color::default(), 1.0,
        );
        let take = match gate {
            Some(n) => rng.next_max(n) == 0,
            None => true,
        };
        if take {
            let idx = dust.new_dust(
                rng, dust_ctx, world, pos, w as i32, h as i32, 242, 0.0, 0.0, 0,
                Color::default(), 1.0,
            );
            let d = dust.slot_mut(idx);
            d.no_gravity = true;
            d.scale = 1.5;
            d.fade_in = 1.0;
            d.velocity *= 3.0;
        }
    }
}

/// The 424 Vortex Soldier death (`NPC.cs:83293-83309`, sites 236-240): the
/// 20-iteration type-249 + `Next(3) == 0`-gated type-242 burst, then the 5
/// gores 793/794/794/795/795 at the +0/+20/+20/+34/+34 Y ladder.
fn vortex_soldier_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    vortex_death_dust(npc, dust, dust_ctx, rng, world, hit_direction, 20, Some(3));
    let pos = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, 793, s); // :83305
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 794, s); // :83306
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 794, s); // :83307
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, 795, s); // :83308
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, 795, s); // :83309
}

/// The 423 Vortex Hornet death (`NPC.cs:83331-83342`, sites 241-243): the
/// 20-iteration UNGATED type-249 + type-242 burst, then the 3 gores
/// 790/791/792 at Center with velocity ×0.8/×0.8/×0.9. Scale defaults to 1.
fn vortex_hornet_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    vortex_death_dust(npc, dust, dust_ctx, rng, world, hit_direction, 20, None);
    let c = center(npc);
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, c, vel * 0.8, 790, 1.0); // :83340
    gore.new_gore(rng, world, c, vel * 0.8, 791, 1.0); // :83341
    gore.new_gore(rng, world, c, vel * 0.9, 792, 1.0); // :83342
}

/// The 421 Vortex Larva death (`NPC.cs:83364-83375`, sites 244-246): the
/// 10-iteration UNGATED type-249 + type-242 burst, then the 3 gores
/// 785/786/787 at Center with velocity ×0.8/×0.8/×0.9. Same shape as the
/// Hornet, 10 iterations and different types.
fn vortex_larva_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    vortex_death_dust(npc, dust, dust_ctx, rng, world, hit_direction, 10, None);
    let c = center(npc);
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, c, vel * 0.8, 785, 1.0); // :83373
    gore.new_gore(rng, world, c, vel * 0.8, 786, 1.0); // :83374
    gore.new_gore(rng, world, c, vel * 0.9, 787, 1.0); // :83375
}

/// The 420 Vortex Rider death (`NPC.cs:83397-83408`, sites 247-249): the
/// 10-iteration UNGATED type-249 + type-242 burst, then the 3 gores
/// 782/783/784 at Center with velocity ×0.8/×0.8/×0.9. Same shape as the
/// Larva, different types.
fn vortex_rider_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    vortex_death_dust(npc, dust, dust_ctx, rng, world, hit_direction, 10, None);
    let c = center(npc);
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, c, vel * 0.8, 782, 1.0); // :83406
    gore.new_gore(rng, world, c, vel * 0.8, 783, 1.0); // :83407
    gore.new_gore(rng, world, c, vel * 0.9, 784, 1.0); // :83408
}

/// The 409 Stardust Spider death (`NPC.cs:83496-83505`, sites 250-252): the
/// 20-iteration UNGATED type-17 + type-229 burst (the 229 gets noGravity,
/// velocity ×3 — NO scale/fadeIn writes here), then the 3 gores 775/776/777
/// at Center with velocity ×0.8/×0.8/×0.9. Scale defaults to 1.
fn stardust_spider_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:83496-83502` — 20 iterations: type-17 + ungated type-229.
    for _ in 0..20 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 17, hit_direction, -1.0, 0,
            Color::default(), 1.0,
        );
        let idx = dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 229, 0.0, 0.0, 0,
            Color::default(), 1.0,
        );
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.velocity *= 3.0;
    }
    let c = center(npc);
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, c, vel * 0.8, 775, 1.0); // :83503
    gore.new_gore(rng, world, c, vel * 0.8, 776, 1.0); // :83504
    gore.new_gore(rng, world, c, vel * 0.9, 777, 1.0); // :83505
}

/// The 407 Stardust Cell death (`NPC.cs:83547-83562`, sites 253-258): the
/// 20-iteration burst, each a `Next(3) != 0`-gated type-17 dust (the INVERSE
/// gate — the draw fires, the spawn is skipped on 0) then an UNGATED
/// type-229 dust (noGravity, velocity ×3); then the 6 gores 778/779 at Top
/// (vel ×0.8) and 780/781/780/781 at Center (vel ×0.9). Scale defaults to 1.
fn stardust_cell_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:83547-83556` — 20 iterations: the Next(3)!=0-gated type-17 + ungated 229.
    for _ in 0..20 {
        // `:83549` — the INVERSE gate: spawn when Next(3) != 0.
        if rng.next_max(3) != 0 {
            dust.new_dust(
                rng, dust_ctx, world, pos, w as i32, h as i32, 17, hit_direction, -1.0, 0,
                Color::default(), 1.0,
            );
        }
        let idx = dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 229, 0.0, 0.0, 0,
            Color::default(), 1.0,
        );
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.velocity *= 3.0;
    }
    let t = top(npc);
    let c = center(npc);
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, t, vel * 0.8, 778, 1.0); // :83557
    gore.new_gore(rng, world, t, vel * 0.8, 779, 1.0); // :83558
    gore.new_gore(rng, world, c, vel * 0.9, 780, 1.0); // :83559
    gore.new_gore(rng, world, c, vel * 0.9, 781, 1.0); // :83560
    gore.new_gore(rng, world, c, vel * 0.9, 780, 1.0); // :83561
    gore.new_gore(rng, world, c, vel * 0.9, 781, 1.0); // :83562
}

/// The 411 Stardust Dragon body death (`NPC.cs:83649-83661`, sites
/// 259-263): the 20-iteration burst (type-17 dust + `Next(2) == 0`-gated
/// type-229 dust, noGravity only — no velocity/scale write), then the 5
/// gores 772/773/773/774/774 at the +0/+20/+20/+34/+34 Y ladder.
fn stardust_dragon_body_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:83649-83656` — 20 iterations: type-17 + the Next(2)-gated type-229.
    for _ in 0..20 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 17, hit_direction, -1.0, 0,
            Color::default(), 1.0,
        );
        if rng.next_max(2) == 0 {
            let idx = dust.new_dust(
                rng, dust_ctx, world, pos, w as i32, h as i32, 229, 0.0, 0.0, 0,
                Color::default(), 1.0,
            );
            dust.slot_mut(idx).no_gravity = true;
        }
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, 772, s); // :83657
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 773, s); // :83658
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 773, s); // :83659
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, 774, s); // :83660
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, 774, s); // :83661
}

/// The 491 Flying Dutchman death's HEAD block (`NPC.cs:83668-83677`, sites
/// 264-274): the anchor `vector15 = Center - (60,60)` and the facing unit
/// `vector16 = ((spriteDirection != 1) ? 1 : -1, 0).RotatedBy(rotation)`,
/// then the 858 gore at `+ vector16 * (width/2)` and the 857 at
/// `+ vector16 * (width/3)` (the C# `base.width / 2` / `/ 3` are INTEGER
/// divisions — width is int), then the `num183 = -1..1` loop's 3×3 = 9 gores
/// of type 859 at `width * (0.45/0.4/0.35 + num183*0.1)`. No arm-level draws
/// (the types/anchors are data); each spawn draws only the NewGore kicks.
/// The rest of the 491 arm (the RandomVector2 fan, the 855/854/853/856
/// loops, the 30-gore 861/862 burst, the 120-dust burst and the projectile
/// seam) lands with the next batches.
fn dutchman_death_head(
    npc: &Npc,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    let (w, _h) = npc.size();
    // `:83668` — vector15 = Center - (60,60).
    let v15 = center(npc) - Vec2::new(60.0, 60.0);
    // `:83669` — vector16 = (dir, 0).RotatedBy(rotation); dir = +1 unless
    // spriteDirection == 1.
    let dir = if npc.sprite_dir != 1 { 1.0 } else { -1.0 };
    let v16 = rotated_by(Vec2::new(dir, 0.0), npc.rotation as f64);
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    // `:83670-83671` — the integer width/2 and width/3.
    let wi = w as i32;
    gore.new_gore(rng, world, v15 + v16 * (wi / 2) as f32, vel, 858, s);
    gore.new_gore(rng, world, v15 + v16 * (wi / 3) as f32, vel, 857, s);
    // `:83672-83677` — the num183 = -1..1 loop, 3 gores of 859 each.
    for num183 in -1..2 {
        let f = num183 as f32;
        gore.new_gore(rng, world, v15 + v16 * (w * (0.45 + f * 0.1)), vel, 859, s); // :83674
        gore.new_gore(rng, world, v15 + v16 * (w * (0.4 + f * 0.1)), vel, 859, s); // :83675
        gore.new_gore(rng, world, v15 + v16 * (w * (0.35 + f * 0.1)), vel, 859, s); // :83676
    }
}

/// The 491 Flying Dutchman death's RandomVector2 FAN (`NPC.cs:83678-83685`,
/// sites 275-289): the `num184 = -1..1` loop, each spawning 5 gores of type
/// `SelectRandom(860,892,893)` at `v15 + v16*(width*k) - (0,drop) +
/// RandomVector2(-40,40)`. The per-gore draw order is the C# argument
/// evaluation: the position's `RandomVector2` (TWO NextDouble, X then Y)
/// FIRST, then the type's `SelectRandom` (ONE Next(3)), then the NewGore
/// kicks. Sites 275-289 (all three iterations).
fn dutchman_death_fan(
    npc: &Npc,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    let (w, _h) = npc.size();
    let v15 = center(npc) - Vec2::new(60.0, 60.0);
    let dir = if npc.sprite_dir != 1 { 1.0 } else { -1.0 };
    let v16 = rotated_by(Vec2::new(dir, 0.0), npc.rotation as f64);
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    // `:83678` — num184 = -1..1 (all three iterations).
    for num184 in -1..2 {
        let _ = num184; // the anchors do not depend on the loop index here
        // `:83680` — width*0.35, drop 200.
        let p = v15 + v16 * (w * 0.35) - Vec2::new(0.0, 200.0) + random_vector2(rng, -40.0, 40.0);
        let ty = select_random(rng, &[860, 892, 893]);
        gore.new_gore(rng, world, p, vel, ty, s);
        // `:83681` — width*0.15, drop 200.
        let p = v15 + v16 * (w * 0.15) - Vec2::new(0.0, 200.0) + random_vector2(rng, -40.0, 40.0);
        let ty = select_random(rng, &[860, 892, 893]);
        gore.new_gore(rng, world, p, vel, ty, s);
        // `:83682` — width*-0.35, drop 200.
        let p = v15 + v16 * (w * -0.35) - Vec2::new(0.0, 200.0) + random_vector2(rng, -40.0, 40.0);
        let ty = select_random(rng, &[860, 892, 893]);
        gore.new_gore(rng, world, p, vel, ty, s);
        // `:83683` — width*-0.15, drop 300.
        let p = v15 + v16 * (w * -0.15) - Vec2::new(0.0, 300.0) + random_vector2(rng, -40.0, 40.0);
        let ty = select_random(rng, &[860, 892, 893]);
        gore.new_gore(rng, world, p, vel, ty, s);
        // `:83684` — width*-0.15, drop 300 (the repeat).
        let p = v15 + v16 * (w * -0.15) - Vec2::new(0.0, 300.0) + random_vector2(rng, -40.0, 40.0);
        let ty = select_random(rng, &[860, 892, 893]);
        gore.new_gore(rng, world, p, vel, ty, s);
    }
}

/// The 491 Flying Dutchman death's 855 loop (`NPC.cs:83686-83690`, sites
/// 290-293): the `num185 = 0..1` loop, each spawning 2 gores of type 855 at
/// `v15 + v16*(width*±0.15) - (0, 160 + 60*num185)`. No arm-level draws.
fn dutchman_death_855(
    npc: &Npc,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    let (w, _h) = npc.size();
    let v15 = center(npc) - Vec2::new(60.0, 60.0);
    let dir = if npc.sprite_dir != 1 { 1.0 } else { -1.0 };
    let v16 = rotated_by(Vec2::new(dir, 0.0), npc.rotation as f64);
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    // `:83686` — num185 = 0..1.
    for num185 in 0..2 {
        let drop = (160 + 60 * num185) as f32;
        gore.new_gore(rng, world, v15 + v16 * (w * 0.15) - Vec2::new(0.0, drop), vel, 855, s); // :83688
        gore.new_gore(rng, world, v15 + v16 * (w * -0.15) - Vec2::new(0.0, drop), vel, 855, s); // :83689
    }
}

/// The 491 Flying Dutchman death's 854/853/856 loop (`NPC.cs:83691-83696`,
/// sites 294-302): the `num186 = -1..1` loop, each spawning 3 gores — 854 at
/// `width*(0.25 + num186*0.1)`, 853 at `width*(-0.45 + num186*0.1)`, 856 at
/// `width*(0.05 + num186*0.1)`, all dropped by `(0, 20 + 40*num186)`. No
/// arm-level draws.
fn dutchman_death_854(
    npc: &Npc,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    let (w, _h) = npc.size();
    let v15 = center(npc) - Vec2::new(60.0, 60.0);
    let dir = if npc.sprite_dir != 1 { 1.0 } else { -1.0 };
    let v16 = rotated_by(Vec2::new(dir, 0.0), npc.rotation as f64);
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    // `:83691` — num186 = -1..1.
    for num186 in -1..2 {
        let f = num186 as f32;
        let drop = (20 + 40 * num186) as f32;
        gore.new_gore(rng, world, v15 + v16 * (w * (0.25 + f * 0.1)) - Vec2::new(0.0, drop), vel, 854, s); // :83693
        gore.new_gore(rng, world, v15 + v16 * (w * (-0.45 + f * 0.1)) - Vec2::new(0.0, drop), vel, 853, s); // :83694
        gore.new_gore(rng, world, v15 + v16 * (w * (0.05 + f * 0.1)) - Vec2::new(0.0, drop), vel, 856, s); // :83695
    }
}

/// The 491 Flying Dutchman death's 861/862 SCATTER burst (`NPC.cs:83697-83703`,
/// sites 303-332): 30 gores of type `Next(861,863)` at `Center - (60,60)`
/// with velocity Zero, then per-gore post-writes: `alpha = 100`, velocity =
/// `Vector2.Transform((4,0), rotZ(NextDouble*2π))` = `(4cosθ, 4sinθ)` (ONE
/// NextDouble), position = `(pos.X - 20 + (width+40)*NextFloat, pos.Y - 240
/// + (height+240-20)*NextFloat)` (TWO NextFloat). The per-gore draw order:
/// the type `Next(861,863)` FIRST (the C# named-arg Type evaluates before
/// Position/Velocity, but those are data here), then the NewGore kicks
/// (pool), then the post-write NextDouble + two NextFloat. Sites 303-332
/// (all 30 gores).
///
/// NOTE: the 861/862 types hit the pool's sticky=false arm (`Gore.cs:1343`)
/// and the SpecialAI==1 velocity roll is NOT in the 861/862 set, so the
/// post-write velocity overwrite is the final value.
fn dutchman_death_scatter(
    npc: &Npc,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    let (w, h) = npc.size();
    let anchor = center(npc) - Vec2::new(60.0, 60.0);
    // `:83697` — num187 = 0..30 (all 30 gores).
    for _ in 0..30 {
        // `:83699` — the type draw, then the spawn at the anchor, vel Zero.
        let ty = rng.next_range(861, 863);
        let idx = gore.new_gore(rng, world, anchor, Vec2::ZERO, ty, 1.0);
        // `:83700` — alpha = 100 (no draw).
        let g = gore.slot_mut(idx);
        g.alpha = 100;
        // `:83701` — velocity = Transform((4,0), rotZ(NextDouble*2π)).
        let theta = rng.next_double() * 6.2831854820251465;
        let (s, c) = (theta as f32).sin_cos();
        gore.slot_mut(idx).velocity = Vec2::new(4.0 * c, 4.0 * s);
        // `:83702` — the position re-roll (two NextFloat).
        let px = npc.x - 20.0 + (w + 40.0) * rng.next_float();
        let py = npc.y - 240.0 + (h + 240.0 - 20.0) * rng.next_float();
        gore.slot_mut(idx).position = Vec2::new(px, py);
    }
}

/// The 491 Flying Dutchman death's TAIL (`NPC.cs:83704-83720`): the
/// 120-iteration dust burst, each a type-6 dust (alpha 100, scale 2.5,
/// noGravity, velocity ×5) then a type-6 dust (alpha 100, scale 1.5,
/// velocity ×4, noGravity) — the post-writes draw nothing; then the
/// projectile-594 spawn seam (`:83713-83720`) is RETURNED
/// ([`HitEffectOutcomeC::dutchman_death_shots`]) — the per-spawn NextFloat
/// draws + the spawns are the wiring wave's. No gore sites here; this
/// completes the 491 arm's strike-stream census.
fn dutchman_death_tail(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    rng: &mut UnifiedRandom,
    world: &World,
    outcome: &mut HitEffectOutcomeC,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:83704-83712` — 120 iterations x 2 type-6 dusts.
    for _ in 0..120 {
        let idx = dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 6, 0.0, 0.0, 100,
            Color::default(), 2.5,
        );
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.velocity *= 5.0;
        let idx = dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 6, 0.0, 0.0, 100,
            Color::default(), 1.5,
        );
        let d = dust.slot_mut(idx);
        d.velocity *= 4.0;
        d.no_gravity = true;
    }
    // `:83713-83720` — the projectile-spawn seam (netMode != 1, single-player
    // runs); the per-spawn draws + the spawns are the caller's.
    let c = center(npc);
    outcome.dutchman_death_shots = Some((c.x, c.y));
}

/// The 492 Flying Dutchman cannon death (`NPC.cs:83727-83737`, sites
/// 333-342): the 7-dust type-31 burst (alpha 100, scale 1.5, velocity ×1.4
/// — the post-write draws nothing), then 10 gores of type `Next(61,64)` at
/// position with velocity Zero, then per-gore post-writes: `alpha = 100`,
/// velocity = `Vector2.Transform((1,0), rotZ(NextDouble*2π))` =
/// `(cosθ, sinθ)` (ONE NextDouble). The per-gore draw order: the type
/// `Next(61,64)` FIRST, then the NewGore kicks (pool), then the post-write
/// NextDouble. The 61/62/63 types hit the pool's sticky=false arm
/// (`Gore.cs:1341`), no SpecialAI roll.
fn dutchman_cannon_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:83727-83731` — the 7-dust type-31 burst.
    for _ in 0..7 {
        let idx = dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 31, 0.0, 0.0, 100,
            Color::default(), 1.5,
        );
        dust.slot_mut(idx).velocity *= 1.4;
    }
    // `:83732-83737` — the 10-gore scatter.
    for _ in 0..10 {
        let ty = rng.next_range(61, 64);
        let idx = gore.new_gore(rng, world, pos, Vec2::ZERO, ty, 1.0);
        let g = gore.slot_mut(idx);
        g.alpha = 100;
        let theta = rng.next_double() * 6.2831854820251465;
        let (s, c) = (theta as f32).sin_cos();
        gore.slot_mut(idx).velocity = Vec2::new(c, s);
    }
}

/// The 524-527 Desert Ghoul death (`NPC.cs:83742-83794`, sites 343-348):
/// the per-tier params (`num196` accent-dust type, `num197 = 5` base dust,
/// `num198` gore base): 524 → (0, 5, 863), 525 → (75, 5, 867), 526 →
/// (170, 5, 871), 527 → (67, 5, 879). The 20-iteration type-5 burst with a
/// `Next(3) == 0 && num196 > 0`-gated accent dust (noGravity, scale 1.5,
/// fadeIn 1, velocity ×3) — the C# `&&` evaluates `Next(3)` FIRST, so the
/// draw fires even for 524 (num196 == 0) but the spawn is skipped. Then the
/// 6 gores `num198 / +1 / +2 / +2 / +3 / +3` at the +0/+0/+20/+20/+34/+34 Y
/// ladder.
fn desert_ghoul_death(
    npc: &Npc,
    npc_type: i32,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    // `:83742-83759` — the per-tier params.
    let (num196, num198) = match npc_type {
        525 => (75, 867),
        526 => (170, 871),
        527 => (67, 879),
        _ => (0, 863), // 524
    };
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:83777-83788` — 20 iterations: type-5 + the gated accent dust.
    for _ in 0..20 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 5, hit_direction, -1.0, 0,
            Color::default(), 1.0,
        );
        // `:83780` — Next(3) draws FIRST (the && left operand), then the
        // num196 > 0 check gates the spawn.
        if rng.next_max(3) == 0 && num196 > 0 {
            let idx = dust.new_dust(
                rng, dust_ctx, world, pos, w as i32, h as i32, num196, 0.0, 0.0, 0,
                Color::default(), 1.0,
            );
            let d = dust.slot_mut(idx);
            d.no_gravity = true;
            d.scale = 1.5;
            d.fade_in = 1.0;
            d.velocity *= 3.0;
        }
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, num198, s); // :83789
    gore.new_gore(rng, world, pos, vel, num198 + 1, s); // :83790
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, num198 + 2, s); // :83791
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, num198 + 2, s); // :83792
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, num198 + 3, s); // :83793
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, num198 + 3, s); // :83794
}

/// The 528-529 Desert Lamia death (`NPC.cs:83799-83821`, sites 349-352):
/// the 20-dust type-5 burst (no accent), then the 4 gores — the head is
/// `num202` (528 → 875, 529 → 876, `:83800-83804`), then 877/877 at +20 Y
/// and 878 at +34 Y (the tail types are FIXED, not base-relative).
fn desert_lamia_death(
    npc: &Npc,
    npc_type: i32,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    // `:83799-83804` — the per-tier head.
    let num202 = if npc_type == 529 { 876 } else { 875 };
    death_dust_loop(npc, dust, dust_ctx, rng, world, 5, hit_direction, 20);
    let pos = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, num202, s); // :83818
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 877, s); // :83819
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 877, s); // :83820
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, 878, s); // :83821
}

/// The 530/531 Desert Scorpion death (`NPC.cs:83835-83843`, sites 353-357):
/// the 20-dust type-250 burst (SpeedX `2.5*hitDirection`, SpeedY -2.5, alpha
/// 100), then the 5 gores 883/884/885/886/887 at position. Scale defaults
/// to 1 (the 3-arg NewGore).
fn desert_scorpion_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:83835-83838` — the 20-dust type-250 burst (alpha 100).
    for _ in 0..20 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 250, 2.5 * hit_direction, -2.5, 100,
            Color::default(), 1.0,
        );
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, pos, vel, 883, 1.0); // :83839
    gore.new_gore(rng, world, pos, vel, 884, 1.0); // :83840
    gore.new_gore(rng, world, pos, vel, 885, 1.0); // :83841
    gore.new_gore(rng, world, pos, vel, 886, 1.0); // :83842
    gore.new_gore(rng, world, pos, vel, 887, 1.0); // :83843
}

/// The 532 Desert Beast death (`NPC.cs:83857-83864`, sites 358-361): the
/// 50-dust type-5 burst (SpeedX `2.5*hitDirection`, SpeedY -2.5, alpha 100),
/// then the 4 gores 888/889/890/891 at position. Scale defaults to 1.
fn desert_beast_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:83857-83860` — the 50-dust type-5 burst (alpha 100).
    for _ in 0..50 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 5, 2.5 * hit_direction, -2.5, 100,
            Color::default(), 1.0,
        );
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, pos, vel, 888, 1.0); // :83861
    gore.new_gore(rng, world, pos, vel, 889, 1.0); // :83862
    gore.new_gore(rng, world, pos, vel, 890, 1.0); // :83863
    gore.new_gore(rng, world, pos, vel, 891, 1.0); // :83864
}

/// The 460 Butcher death (`NPC.cs:83900-83908`, sites 362-366): the 50-dust
/// type-5 burst (SpeedX `2.5*hitDirection`, SpeedY -2.5), then the 5 gores
/// 643/645/644/646/646 — 643 at position, 645/644 at +20 Y (645 precedes
/// 644), 646/646 at +34 Y.
fn butcher_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:83900-83903` — the 50-dust type-5 burst.
    for _ in 0..50 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 5, 2.5 * hit_direction, -2.5, 0,
            Color::default(), 1.0,
        );
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, 643, s); // :83904
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 645, s); // :83905
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 644, s); // :83906
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, 646, s); // :83907
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, 646, s); // :83908
}

/// The 477 Mothron death (`NPC.cs:83922-83930`, sites 367-371): the 50-dust
/// type-237 burst (SpeedX `2.5*hitDirection`, SpeedY -2.5), then the 5 gores
/// 687/688/689/690/691 all at position.
fn mothron_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:83922-83925` — the 50-dust type-237 burst.
    for _ in 0..50 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 237, 2.5 * hit_direction, -2.5, 0,
            Color::default(), 1.0,
        );
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, 687, s); // :83926
    gore.new_gore(rng, world, pos, vel, 688, s); // :83927
    gore.new_gore(rng, world, pos, vel, 689, s); // :83928
    gore.new_gore(rng, world, pos, vel, 690, s); // :83929
    gore.new_gore(rng, world, pos, vel, 691, s); // :83930
}

/// The 479 Mothron Spawn death (`NPC.cs:83944-83950`, sites 372-374): the
/// 50-dust type-237 burst (SpeedX `2.5*hitDirection`, SpeedY -2.5), then the
/// 3 gores 681/682/683 at position.
fn mothron_spawn_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:83944-83947` — the 50-dust type-237 burst.
    for _ in 0..50 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 237, 2.5 * hit_direction, -2.5, 0,
            Color::default(), 1.0,
        );
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, 681, s); // :83948
    gore.new_gore(rng, world, pos, vel, 682, s); // :83949
    gore.new_gore(rng, world, pos, vel, 683, s); // :83950
}

/// The 478 Mothron Egg death (`NPC.cs:83965-83972`, sites 375-378): the
/// 50-dust burst of type `237 + Next(2)` (the type draw fires per dust,
/// SpeedX `2.5*hitDirection`, SpeedY -2.5), then the 4 gores 684/685/686 and
/// `684 + Next(3)` (the last gore's type draw fires before its spawn).
fn mothron_egg_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:83965-83968` — the 50-dust burst, type 237 + Next(2).
    for _ in 0..50 {
        let ty = 237 + rng.next_max(2);
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, ty, 2.5 * hit_direction, -2.5, 0,
            Color::default(), 1.0,
        );
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, 684, s); // :83969
    gore.new_gore(rng, world, pos, vel, 685, s); // :83970
    gore.new_gore(rng, world, pos, vel, 686, s); // :83971
    // `:83972` — the type draw fires before the spawn.
    let ty = 684 + rng.next_max(3);
    gore.new_gore(rng, world, pos, vel, ty, s);
}

/// The shared 5-gore death shape of the 466/468/469/463/461/462 arms
/// (`NPC.cs:83986-83994` et al.): the 50-dust type-5 burst (SpeedX
/// `2.5*hitDirection`, SpeedY -2.5), then 5 gores — the head at position,
/// two of the "middle" type at +20 Y, two of the "tail" type at +34 Y. The
/// per-arm type triple `(head, middle, tail)` is data; the 466 arm's middle
/// (659) precedes its tail (658) in the type ids but the SHAPE is identical.
fn five_gore_type5_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
    head: i32,
    middle: i32,
    tail: i32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // The 50-dust type-5 burst.
    for _ in 0..50 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 5, 2.5 * hit_direction, -2.5, 0,
            Color::default(), 1.0,
        );
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, head, s);
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, middle, s);
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, middle, s);
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, tail, s);
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, tail, s);
}

/// The 466 Psycho death (`NPC.cs:83986-83994`, sites 379-383): the shared
/// 5-gore shape with types 656/659/658 (the middle 659 at +20, the tail 658
/// at +34 — the C# order is 656/657/659/658/658, so the +20 pair is
/// 657 THEN 659, NOT a repeated middle). Ported explicitly to keep the
/// distinct +20 types.
fn psycho_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    for _ in 0..50 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 5, 2.5 * hit_direction, -2.5, 0,
            Color::default(), 1.0,
        );
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, 656, s); // :83990
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 657, s); // :83991
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 659, s); // :83992
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, 658, s); // :83993
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, 658, s); // :83994
}

/// The 468 DrManFly death (`NPC.cs:84008-84016`, sites 384-388): the shared
/// 5-gore shape with types 662/663/664 (the +20 pair is the repeated 663).
fn dr_man_fly_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    five_gore_type5_death(npc, dust, dust_ctx, gore, rng, world, hit_direction, 662, 663, 664);
}

/// The 469 ThePossessed death (`NPC.cs:84030-84038`, sites 389-393): the
/// shared 5-gore shape with types 665/666/667.
fn the_possessed_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    five_gore_type5_death(npc, dust, dust_ctx, gore, rng, world, hit_direction, 665, 666, 667);
}

/// The 463 Nailhead death (`NPC.cs:84052-84060`, sites 394-398): the shared
/// 5-gore shape with types 653/654/655.
fn nailhead_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    five_gore_type5_death(npc, dust, dust_ctx, gore, rng, world, hit_direction, 653, 654, 655);
}

/// The 461 CreatureFromTheDeep death (`NPC.cs:84074-84082`, sites 399-403):
/// the shared 5-gore shape with types 647/648/649.
fn creature_from_the_deep_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    five_gore_type5_death(npc, dust, dust_ctx, gore, rng, world, hit_direction, 647, 648, 649);
}

/// The 462 Fritz death (`NPC.cs:84096-84104`, sites 404-408): the shared
/// 5-gore shape with types 650/651/652.
fn fritz_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    five_gore_type5_death(npc, dust, dust_ctx, gore, rng, world, hit_direction, 650, 651, 652);
}

/// The 379/438 Cultist Archer / Devote death (`NPC.cs:84118-84124`, sites
/// 409-411): the 30-dust type-5 burst (SpeedX `2.5*hitDirection`, SpeedY
/// -2.5), then the 3 gores 900/901/901 — 900 at position, the 901 pair at
/// +20 Y.
fn cultist_archer_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:84118-84121` — the 30-dust type-5 burst.
    for _ in 0..30 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 5, 2.5 * hit_direction, -2.5, 0,
            Color::default(), 1.0,
        );
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, 900, s); // :84122
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 901, s); // :84123
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 901, s); // :84124
}

/// The shared cultist BONE-SPATTER loop (`NPC.cs:84150-84155` et al.):
/// `Next(1,4)` draws the count (1-3), then per gore a `Next(11,14)` type at
/// `(pos.X, pos.Y + height/2 - 10)` with velocity `(hitDirection, 0)`, then
/// the post-write `velocity *= 0.8`. The count draw fires ONCE before the
/// loop; the type draw fires per gore BEFORE its spawn. The `height/2` is
/// the C# integer division (height is int).
fn cultist_bone_spatter(
    npc: &Npc,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (_w, h) = npc.size();
    let count = rng.next_range(1, 4);
    let anchor = Vec2::new(npc.x, npc.y + (h as i32 / 2) as f32 - 10.0);
    let s = npc.scale;
    for _ in 0..count {
        let ty = rng.next_range(11, 14);
        let idx = gore.new_gore(rng, world, anchor, Vec2::new(hit_direction, 0.0), ty, s);
        gore.slot_mut(idx).velocity *= 0.8;
    }
}

/// The 439 Cultist Boss death (`NPC.cs:84140-84159`, sites 412-418+): the
/// 30-dust type-5 burst (SpeedX `2.5*hitDirection`, SpeedY -2.5), then the
/// 10-dust type-16 burst (scale 1.5, velocity ×2, noGravity), then the
/// `Next(1,4)` bone-spatter loop ([`cultist_bone_spatter`], 1-3 gores of
/// `Next(11,14)`), then the 4 fixed gores 902/903/901/901 at the
/// +0/+10/+20/+20 Y ladder. The bone loop's site count is variable (the
/// 4 fixed gores are sites 415-418 when the loop draws 1).
fn cultist_boss_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:84140-84143` — the 30-dust type-5 burst.
    for _ in 0..30 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 5, 2.5 * hit_direction, -2.5, 0,
            Color::default(), 1.0,
        );
    }
    // `:84144-84149` — the 10-dust type-16 burst.
    for _ in 0..10 {
        let idx = dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 16, 0.0, 0.0, 0,
            Color::default(), 1.5,
        );
        let d = dust.slot_mut(idx);
        d.velocity *= 2.0;
        d.no_gravity = true;
    }
    // `:84150-84155` — the bone-spatter loop.
    cultist_bone_spatter(npc, gore, rng, world, hit_direction);
    // `:84156-84159` — the 4 fixed gores.
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, 902, s); // :84156
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 10.0), vel, 903, s); // :84157
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 901, s); // :84158
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 901, s); // :84159
}

/// The shared cultist type-16 death dust burst (`NPC.cs:84166-84171` /
/// `:84193-84198`): `count` type-16 dusts (scale 1.5, velocity ×2,
/// noGravity — the post-writes draw nothing).
fn cultist_type16_dust(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    rng: &mut UnifiedRandom,
    world: &World,
    count: i32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    for _ in 0..count {
        let idx = dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 16, 0.0, 0.0, 0,
            Color::default(), 1.5,
        );
        let d = dust.slot_mut(idx);
        d.velocity *= 2.0;
        d.no_gravity = true;
    }
}

/// The 440 Cultist Boss Clone death (`NPC.cs:84166-84177`, sites 419-421+):
/// the 10-dust type-16 burst then the `Next(1,4)` bone-spatter loop (1-3
/// gores of `Next(11,14)`). No fixed gores — the clone is just the spatter.
fn cultist_clone_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    cultist_type16_dust(npc, dust, dust_ctx, rng, world, 10);
    cultist_bone_spatter(npc, gore, rng, world, hit_direction);
}

/// The 454-459 Cultist Dragon death (`NPC.cs:84193-84204`, sites 422-424+):
/// the 10-dust type-16 burst then the `Next(1,4)` bone-spatter loop. Same
/// shape as the clone (the hit half differs — dmg-scaled type-16, the
/// wiring wave's).
fn cultist_dragon_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    cultist_type16_dust(npc, dust, dust_ctx, rng, world, 10);
    cultist_bone_spatter(npc, gore, rng, world, hit_direction);
}

/// The 372/373 Sharkron death (`NPC.cs:84218-84226`, sites 425-428): the
/// 75-dust type-5 burst at `Center - (25,25)` with a 50x50 box (SpeedX
/// `2*hitDirection`, SpeedY -2, velocity ÷2 post-write), then the 4 gores
/// 583/577/578/579 at Center with velocity ×0.8/×0.8/×0.9/×1. Scale
/// defaults to 1.
fn sharkron_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    // `:84218-84222` — the 75-dust burst at Center-(25,25), 50x50 box.
    let dust_pos = center(npc) - Vec2::new(25.0, 25.0);
    for _ in 0..75 {
        let idx = dust.new_dust(
            rng, dust_ctx, world, dust_pos, 50, 50, 5, 2.0 * hit_direction, -2.0, 0,
            Color::default(), 1.0,
        );
        dust.slot_mut(idx).velocity /= 2.0;
    }
    let c = center(npc);
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, c, vel * 0.8, 583, 1.0); // :84223
    gore.new_gore(rng, world, c, vel * 0.8, 577, 1.0); // :84224
    gore.new_gore(rng, world, c, vel * 0.9, 578, 1.0); // :84225
    gore.new_gore(rng, world, c, vel, 579, 1.0); // :84226
}

/// The 370 Duke Fishron death (`NPC.cs:84269-84278`, sites 429-434): the
/// 150-dust type-5 burst (SpeedX `2*hitDirection`, SpeedY -2), then the 6
/// gores at the direction-spread anchors: 576 at `Center - UnitX*20*dir`,
/// 574 at `Center - UnitY*30`, 575 at Center, 573 at `Center + UnitX*20*dir`,
/// then 574/575 again at `Center - UnitY*30` / Center. `base.direction`
/// rides the port's `facing` (the direction/facing fold, npc.rs).
fn duke_fishron_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:84269-84272` — the 150-dust type-5 burst.
    for _ in 0..150 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 5, 2.0 * hit_direction, -2.0, 0,
            Color::default(), 1.0,
        );
    }
    let c = center(npc);
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    let dir = npc.facing as f32;
    gore.new_gore(rng, world, c - Vec2::new(20.0 * dir, 0.0), vel, 576, s); // :84273
    gore.new_gore(rng, world, c - Vec2::new(0.0, 30.0), vel, 574, s); // :84274
    gore.new_gore(rng, world, c, vel, 575, s); // :84275
    gore.new_gore(rng, world, c + Vec2::new(20.0 * dir, 0.0), vel, 573, s); // :84276
    gore.new_gore(rng, world, c - Vec2::new(0.0, 30.0), vel, 574, s); // :84277
    gore.new_gore(rng, world, c, vel, 575, s); // :84278
}

/// The 381 BrainScrambler death (`NPC.cs:84292-84299`, sites 435-438): the
/// 30-dust type-227 burst (SpeedX `2.5*hitDirection`, SpeedY -2.5), then the
/// 4 gores 587/590 at position, 588 at +20 Y, 589 at +34 Y (the 590 precedes
/// the 588 — NOT a base+offset ladder).
fn brain_scrambler_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:84292-84295` — the 30-dust type-227 burst.
    for _ in 0..30 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 227, 2.5 * hit_direction, -2.5, 0,
            Color::default(), 1.0,
        );
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, 587, s); // :84296
    gore.new_gore(rng, world, pos, vel, 590, s); // :84297
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 588, s); // :84298
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, 589, s); // :84299
}

/// The 382/390 RayGunner / ScutlixRider death (`NPC.cs:84313-84319`, sites
/// 439-441): the 30-dust type-227 burst, then the 3 gores 591/592/593 at the
/// +0/+20/+34 Y ladder.
fn ray_gunner_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    for _ in 0..30 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 227, 2.5 * hit_direction, -2.5, 0,
            Color::default(), 1.0,
        );
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, 591, s); // :84317
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 592, s); // :84318
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, 593, s); // :84319
}

/// The 383 MartianOfficer death (`NPC.cs:84333-84339`, sites 442-444): the
/// 30-dust type-227 burst, then the 3 gores 594/595/596 at the +0/+20/+34 Y
/// ladder. Same shape as the RayGunner, different types.
fn martian_officer_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    for _ in 0..30 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 227, 2.5 * hit_direction, -2.5, 0,
            Color::default(), 1.0,
        );
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, 594, s); // :84337
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 595, s); // :84338
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, 596, s); // :84339
}

/// The 385 GrayGrunt death (`NPC.cs:84373-84379`, sites 445-447): the
/// 30-dust type-227 burst, then the 3 gores 597/598/599 at the +0/+20/+34 Y
/// ladder. Same shape as the RayGunner.
fn gray_grunt_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    for _ in 0..30 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 227, 2.5 * hit_direction, -2.5, 0,
            Color::default(), 1.0,
        );
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, 597, s); // :84377
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 598, s); // :84378
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, 599, s); // :84379
}

/// The 386 MartianEngineer death (`NPC.cs:84393-84399`, sites 448-450): the
/// 30-dust type-227 burst, then the 3 gores 600/601/602 at the +0/+20/+34 Y
/// ladder.
fn martian_engineer_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    for _ in 0..30 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 227, 2.5 * hit_direction, -2.5, 0,
            Color::default(), 1.0,
        );
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, 600, s); // :84397
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 601, s); // :84398
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, 602, s); // :84399
}

/// The 387 MartianTurret death (`NPC.cs:84415-84422`, sites 451-453): the
/// 30-dust type-226 burst (SpeedX `-1.5*hitDirection`, SpeedY -1.5, scale
/// 0.5 post-write), then the 3 gores 603/603/604 at the +0/+20/+34 Y ladder
/// (the 603 repeats at +0 and +20).
fn martian_turret_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:84415-84419` — the 30-dust type-226 burst (scale 0.5).
    for _ in 0..30 {
        let idx = dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 226, -1.5 * hit_direction, -1.5, 0,
            Color::default(), 1.0,
        );
        dust.slot_mut(idx).scale = 0.5;
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, 603, s); // :84420
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 603, s); // :84421
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, 604, s); // :84422
}

/// The 520 MartianWalker death (`NPC.cs:84438-84447`, sites 454-458): the
/// 30-dust type-226 burst (SpeedX `-1.5*hitDirection`, SpeedY -1.5, scale
/// 0.5 post-write), then the 5 gores 851/852 at position and 848/849/850 at
/// +80 Y (the tall walker's leg gores).
fn martian_walker_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:84438-84442` — the 30-dust type-226 burst (scale 0.5).
    for _ in 0..30 {
        let idx = dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 226, -1.5 * hit_direction, -1.5, 0,
            Color::default(), 1.0,
        );
        dust.slot_mut(idx).scale = 0.5;
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, 851, s); // :84443
    gore.new_gore(rng, world, pos, vel, 852, s); // :84444
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 80.0), vel, 848, s); // :84445
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 80.0), vel, 849, s); // :84446
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 80.0), vel, 850, s); // :84447
}

/// The 389 GigaZapper death (`NPC.cs:84461-84467`, sites 459-461): the
/// 30-dust type-227 burst, then the 3 gores 904/905/906 at the +0/+20/+34 Y
/// ladder. Same shape as the RayGunner.
fn giga_zapper_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    for _ in 0..30 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 227, 2.5 * hit_direction, -2.5, 0,
            Color::default(), 1.0,
        );
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, 904, s); // :84465
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 905, s); // :84466
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, 906, s); // :84467
}

/// `Utils.ToRotationVector2` (`Utils.cs:1665-1668`): `((float)cos(f),
/// (float)sin(f))` — the f64 trig over the f32 angle.
fn to_rotation_vector2(f: f32) -> Vec2 {
    Vec2::new((f as f64).cos() as f32, (f as f64).sin() as f32)
}

/// The shared MartianDrone/Probe death velocity (`NPC.cs:84488-84497` /
/// `:84520-84529`): `base.velocity`, but when `ai[0] == 3` it becomes
/// `rotation.ToRotationVector2()` (X flipped when `spriteDirection == 1`)
/// ×5. No draws — the velocity is data.
fn martian_drone_velocity(npc: &Npc) -> Vec2 {
    let mut v = Vec2::new(npc.vx, npc.vy);
    if npc.ai[0] == 3.0 {
        v = to_rotation_vector2(npc.rotation);
        if npc.sprite_dir == 1 {
            v.x *= -1.0;
        }
        v *= 5.0;
    }
    v
}

/// The 388 MartianDrone death (`NPC.cs:84483-84499`, sites 462-463): the
/// 10-dust type-226 burst (SpeedX `-2.5*hitDirection`, SpeedY -2.5, scale
/// 0.5 post-write), then the 2 gores 605/606 at `Center - (10,10)` with the
/// [`martian_drone_velocity`].
fn martian_drone_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:84483-84487` — the 10-dust type-226 burst (scale 0.5).
    for _ in 0..10 {
        let idx = dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 226, -2.5 * hit_direction, -2.5, 0,
            Color::default(), 1.0,
        );
        dust.slot_mut(idx).scale = 0.5;
    }
    let v = martian_drone_velocity(npc);
    let anchor = center(npc) - Vec2::new(10.0, 10.0);
    let s = npc.scale;
    gore.new_gore(rng, world, anchor, v, 605, s); // :84498
    gore.new_gore(rng, world, anchor, v, 606, s); // :84499
}

/// The 399 MartianProbe death (`NPC.cs:84515-84531`, sites 464-465): the
/// 10-dust type-226 burst (scale 0.5), then the 2 gores 898/899 at
/// `Center - (10,10)` with the same [`martian_drone_velocity`].
fn martian_probe_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    for _ in 0..10 {
        let idx = dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 226, -2.5 * hit_direction, -2.5, 0,
            Color::default(), 1.0,
        );
        dust.slot_mut(idx).scale = 0.5;
    }
    let v = martian_drone_velocity(npc);
    let anchor = center(npc) - Vec2::new(10.0, 10.0);
    let s = npc.scale;
    gore.new_gore(rng, world, anchor, v, 898, s); // :84530
    gore.new_gore(rng, world, anchor, v, 899, s); // :84531
}

/// The 391 Scutlix death (`NPC.cs:84545-84551`, sites 466-468): the 30-dust
/// type-227 burst, then the 3 gores 607/608 at `position + UnitX*width/2`
/// (the C# `base.width / 2f` is a FLOAT division) and 609 at +20 Y.
fn scutlix_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    for _ in 0..30 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 227, 2.5 * hit_direction, -2.5, 0,
            Color::default(), 1.0,
        );
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    let half = Vec2::new(w / 2.0, 0.0);
    gore.new_gore(rng, world, pos + half, vel, 607, s); // :84549
    gore.new_gore(rng, world, pos + half, vel, 608, s); // :84550
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 609, s); // :84551
}

/// The 392/393/394/395 Martian Saucer family death (`NPC.cs:84567-84589`,
/// sites 469-476): the 30-dust type-228 burst (SpeedX `-1.5*hitDirection`,
/// SpeedY -1.5, scale 0.5 post-write), then the per-type gores at Center:
/// the 395 core spawns 6 (610/611/612/613/614/615 at the spread anchors),
/// the 393 turret spawns 616, the 394 cannon spawns 617; the 392 saucer
/// itself spawns NO gore (the C# has no 392 arm inside the block).
fn martian_saucer_death(
    npc: &Npc,
    npc_type: i32,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:84567-84571` — the 30-dust type-228 burst (scale 0.5).
    for _ in 0..30 {
        let idx = dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 228, -1.5 * hit_direction, -1.5, 0,
            Color::default(), 1.0,
        );
        dust.slot_mut(idx).scale = 0.5;
    }
    let c = center(npc);
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    // `:84573-84581` — the 395 core's 6 gores.
    if npc_type == 395 {
        gore.new_gore(rng, world, c + Vec2::new(-20.0, -10.0), vel, 610, s); // :84575
        gore.new_gore(rng, world, c + Vec2::new(20.0, -10.0), vel, 611, s); // :84576
        gore.new_gore(rng, world, c + Vec2::new(-50.0, 0.0), vel, 612, s); // :84577
        gore.new_gore(rng, world, c + Vec2::new(50.0, 0.0), vel, 613, s); // :84578
        gore.new_gore(rng, world, c + Vec2::new(-60.0, 10.0), vel, 614, s); // :84579
        gore.new_gore(rng, world, c + Vec2::new(0.0, 30.0), vel, 615, s); // :84580
    }
    // `:84582-84585` — the 393 turret's single gore.
    if npc_type == 393 {
        gore.new_gore(rng, world, c, vel, 616, s); // :84584
    }
    // `:84586-84589` — the 394 cannon's single gore.
    if npc_type == 394 {
        gore.new_gore(rng, world, c, vel, 617, s); // :84588
    }
}

/// The 482 GraniteGolem death (`NPC.cs:84603-84612`, sites 477-480): the
/// 30-dust type-240 burst (SpeedX `2.5*hitDirection`, SpeedY -2.5), then the
/// 4 gores 718/719/720/721 at the +0/+10/+20/+40 Y ladder. The 483
/// GraniteFlyer shares the gate but spawns NO gore (the C# `if (type==482)`
/// inside).
fn granite_golem_death(
    npc: &Npc,
    npc_type: i32,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:84603-84606` — the 30-dust type-240 burst.
    for _ in 0..30 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 240, 2.5 * hit_direction, -2.5, 0,
            Color::default(), 1.0,
        );
    }
    // `:84607-84613` — only the 482 spawns gores.
    if npc_type == 482 {
        let vel = Vec2::new(npc.vx, npc.vy);
        let s = npc.scale;
        gore.new_gore(rng, world, pos, vel, 718, s); // :84609
        gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 10.0), vel, 719, s); // :84610
        gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 720, s); // :84611
        gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 40.0), vel, 721, s); // :84612
    }
}

/// The 631 RockGolem death (`NPC.cs:84627-84634`, sites 481-484): the
/// 30-dust type-1 burst (SpeedX `2.5*hitDirection`, SpeedY -2.5), then the
/// 4 gores 1210/1211/1212/1213 at the +0/+10/+20/+40 Y ladder (the same
/// shape as the GraniteGolem, different dust/types).
fn rock_golem_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:84627-84630` — the 30-dust type-1 burst.
    for _ in 0..30 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 1, 2.5 * hit_direction, -2.5, 0,
            Color::default(), 1.0,
        );
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, 1210, s); // :84631
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 10.0), vel, 1211, s); // :84632
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 1212, s); // :84633
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 40.0), vel, 1213, s); // :84634
}

/// The party Blue Slime confetti (`NPC.cs:84723-84741`, sites 485-489+):
/// gated on `type == 1 && ai[1] == 1345` (the party variant). `num314 = 5`,
/// tripled to 15 when `life <= 0`. First the `num314*2`-dust confetti burst
/// (type `Next(139,143)`, SpeedY -10, scale 1.2, then scale ×`1+Next(-30,31)*0.01`
/// and velocity.X += `Next(-50,51)*0.05`), then the `num314`-gore loop: type
/// `Next(276,283)` at position with velocity `(0,-10)`, then scale
/// ×`1+Next(-20,21)*0.01` and velocity.X += `Next(-50,51)*0.05`. The per-gore
/// draw order: the type draw, the NewGore kicks (pool), then the two
/// post-write draws. This is a HIT-half arm too (the confetti fires on every
/// hit of the party slime, not just death) — the `life <= 0` only triples
/// the count.
fn party_slime_confetti(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:84725-84729` — num314 = 5, tripled on death.
    let mut num314 = 5;
    if npc.hp <= 0 {
        num314 *= 3;
    }
    // `:84730-84735` — the num314*2 dust confetti.
    for _ in 0..num314 * 2 {
        let ty = rng.next_range(139, 143);
        let idx = dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, ty, 0.0, -10.0, 0,
            Color::default(), 1.2,
        );
        let d = dust.slot_mut(idx);
        d.scale *= 1.0 + rng.next_range(-30, 31) as f32 * 0.01;
        d.velocity.x += rng.next_range(-50, 51) as f32 * 0.05;
    }
    // `:84736-84741` — the num314 gore confetti.
    for _ in 0..num314 {
        let ty = rng.next_range(276, 283);
        let idx = gore.new_gore(rng, world, pos, Vec2::new(0.0, -10.0), ty, 1.0);
        let g = gore.slot_mut(idx);
        g.scale *= 1.0 + rng.next_range(-20, 21) as f32 * 0.01;
        g.velocity.x += rng.next_range(-50, 51) as f32 * 0.05;
    }
}

/// The 305-314 Scarecrow death (`NPC.cs:84806-84814`, sites 490-494): the
/// 60-dust type-194 burst (SpeedX `2*hitDirection`, SpeedY -2, scale 1.1),
/// then the 5 gores 441/442/443/444/445 at the +0/+20/+20/+34/+34 Y ladder.
fn scarecrow_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:84806-84809` — the 60-dust type-194 burst (scale 1.1).
    for _ in 0..60 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 194, 2.0 * hit_direction, -2.0, 0,
            Color::default(), 1.1,
        );
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, 441, s); // :84810
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 442, s); // :84811
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 443, s); // :84812
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, 444, s); // :84813
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, 445, s); // :84814
}

/// The 327 Pumpking death (`NPC.cs:84828-84835`, sites 495-503): the
/// 60-dust type-189 burst (SpeedX `2*hitDirection`, SpeedY -2, scale 1.1),
/// then the 9-gore loop `num332 = 476..=484` at position with velocity ×0.
fn pumpking_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:84828-84831` — the 60-dust type-189 burst (scale 1.1).
    for _ in 0..60 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 189, 2.0 * hit_direction, -2.0, 0,
            Color::default(), 1.1,
        );
    }
    let s = npc.scale;
    // `:84832-84835` — the 476..=484 loop, velocity x0.
    for num332 in 476..=484 {
        gore.new_gore(rng, world, pos, Vec2::ZERO, num332, s);
    }
}

/// The 345 IceQueen death (`NPC.cs:84861-84865`, sites 504-508): NO dust
/// burst — the 5 gores 513/514/514/515/516 at position with velocity ×0
/// (the 514 repeats).
fn ice_queen_death(
    npc: &Npc,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    let pos = Vec2::new(npc.x, npc.y);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, Vec2::ZERO, 513, s); // :84861
    gore.new_gore(rng, world, pos, Vec2::ZERO, 514, s); // :84862
    gore.new_gore(rng, world, pos, Vec2::ZERO, 514, s); // :84863
    gore.new_gore(rng, world, pos, Vec2::ZERO, 515, s); // :84864
    gore.new_gore(rng, world, pos, Vec2::ZERO, 516, s); // :84865
}

/// The 325 MourningWood death (`NPC.cs:84849-84856`, sites 509-515): the
/// 60-dust type-191 burst (SpeedX `2*hitDirection`, SpeedY -2, scale 1.1),
/// then the 7-gore loop `num335 = 469..=475` at position with velocity ×0.
fn mourning_wood_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:84849-84852` — the 60-dust type-191 burst (scale 1.1).
    for _ in 0..60 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 191, 2.0 * hit_direction, -2.0, 0,
            Color::default(), 1.1,
        );
    }
    let s = npc.scale;
    // `:84853-84856` — the 469..=475 loop, velocity x0.
    for num335 in 469..=475 {
        gore.new_gore(rng, world, pos, Vec2::ZERO, num335, s);
    }
}

/// The 344 Everscream death (`NPC.cs:84878-84889`, sites 516-524): the
/// 60-dust type-196 burst (SpeedX `2*hitDirection`, SpeedY -2), then TWO
/// gore loops — `num338 = 508..=512` (5 gores) then `num339 = 509..=512`
/// (4 gores), all at position with velocity ×0.
fn everscream_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:84878-84881` — the 60-dust type-196 burst.
    for _ in 0..60 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 196, 2.0 * hit_direction, -2.0, 0,
            Color::default(), 1.0,
        );
    }
    let s = npc.scale;
    // `:84882-84885` — the 508..=512 loop, velocity x0.
    for num338 in 508..=512 {
        gore.new_gore(rng, world, pos, Vec2::ZERO, num338, s);
    }
    // `:84886-84889` — the 509..=512 loop, velocity x0.
    for num339 in 509..=512 {
        gore.new_gore(rng, world, pos, Vec2::ZERO, num339, s);
    }
}

/// The 302 SlimeMasked death (`NPC.cs:84937-84941`, site 525): the 50-dust
/// type-192 burst (SpeedX `2*hitDirection`, SpeedY -2, alpha 100, color
/// `(100,100,100,100)`), then the single gore 440 at position.
fn slime_masked_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:84937-84940` — the 50-dust type-192 burst (gray, alpha 100).
    for _ in 0..50 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 192, 2.0 * hit_direction, -2.0, 100,
            Color::new(100, 100, 100, 100), 1.0,
        );
    }
    gore.new_gore(rng, world, pos, Vec2::new(npc.vx, npc.vy), 440, npc.scale); // :84941
}

/// The 245 Golem body death's random-position gores (`NPC.cs:84968-84973`,
/// sites 526-531): 6 gores alternating 368/370, each at
/// `position + (Next(width), Next(height))` — the two position draws fire
/// per gore BEFORE its spawn (the C# argument evaluation). The 50-dust
/// type-148 burst runs first (shared with the 249/247/248 arms), then the
/// fixed tail 360/361/362/363/364/367/366 at position (sites 532-538).
/// Sites 526-538 (all 13 body gores).
fn golem_body_random_gores(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:84962-84965` — the 50-dust type-148 burst.
    for _ in 0..50 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 148, 2.0 * hit_direction, -2.0, 0,
            Color::default(), 1.0,
        );
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    let wi = w as i32;
    let hi = h as i32;
    // `:84968-84973` — the 6 alternating 368/370 random-position gores.
    for i in 0..6 {
        let px = pos.x + rng.next_max(wi) as f32;
        let py = pos.y + rng.next_max(hi) as f32;
        let ty = if i % 2 == 0 { 368 } else { 370 };
        gore.new_gore(rng, world, Vec2::new(px, py), vel, ty, s);
    }
    // `:84974-84980` — the fixed tail at position (the 367 precedes the 366).
    gore.new_gore(rng, world, pos, vel, 360, s); // :84974
    gore.new_gore(rng, world, pos, vel, 361, s); // :84975
    gore.new_gore(rng, world, pos, vel, 362, s); // :84976
    gore.new_gore(rng, world, pos, vel, 363, s); // :84977
    gore.new_gore(rng, world, pos, vel, 364, s); // :84978
    gore.new_gore(rng, world, pos, vel, 367, s); // :84979
    gore.new_gore(rng, world, pos, vel, 366, s); // :84980
}

/// The 249 GolemHeadFree death (`NPC.cs:84984-84990`, sites 539-545): the
/// 50-dust type-148 burst, then 4 random-position gores alternating 368/370
/// (Next(width)+Next(height) per gore), then the fixed 365/363/362 at
/// position.
fn golem_head_free_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:84962-84965` — the shared 50-dust type-148 burst.
    for _ in 0..50 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 148, 2.0 * hit_direction, -2.0, 0,
            Color::default(), 1.0,
        );
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    let wi = w as i32;
    let hi = h as i32;
    // `:84984-84987` — the 4 alternating 368/370 random-position gores.
    for i in 0..4 {
        let px = pos.x + rng.next_max(wi) as f32;
        let py = pos.y + rng.next_max(hi) as f32;
        let ty = if i % 2 == 0 { 368 } else { 370 };
        gore.new_gore(rng, world, Vec2::new(px, py), vel, ty, s);
    }
    // `:84988-84990` — the fixed tail at position.
    gore.new_gore(rng, world, pos, vel, 365, s); // :84988
    gore.new_gore(rng, world, pos, vel, 363, s); // :84989
    gore.new_gore(rng, world, pos, vel, 362, s); // :84990
}

/// The 247/248 Golem fist death (`NPC.cs:84994-84996`, sites 546-548): the
/// 50-dust type-148 burst, then the 3 gores 369/371/370 at position (the
/// `else` arm of the 245-249 block — the fists and any other non-245/246/249
/// member).
fn golem_fist_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    for _ in 0..50 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 148, 2.0 * hit_direction, -2.0, 0,
            Color::default(), 1.0,
        );
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, 369, s); // :84994
    gore.new_gore(rng, world, pos, vel, 371, s); // :84995
    gore.new_gore(rng, world, pos, vel, 370, s); // :84996
}

/// The 243 IceGolem death (`NPC.cs:85056-85060`, site 549): the 200-dust
/// type-161 burst (SpeedX `2*hitDirection`, SpeedY -2), then the single
/// gore 359 at position.
fn ice_golem_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:85056-85059` — the 200-dust type-161 burst.
    for _ in 0..200 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 161, 2.0 * hit_direction, -2.0, 0,
            Color::default(), 1.0,
        );
    }
    gore.new_gore(rng, world, pos, Vec2::new(npc.vx, npc.vy), 359, npc.scale); // :85060
}

/// The 225 UmbrellaSlime death (`NPC.cs:85038-85042`, site 550): the
/// 50-dust type-4 burst (SpeedX `2*hitDirection`, SpeedY -2, alpha 100,
/// color `(0,80,255,100)`), then the single gore 314 at position.
fn umbrella_slime_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:85038-85041` — the 50-dust type-4 burst (blue, alpha 100).
    for _ in 0..50 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 4, 2.0 * hit_direction, -2.0, 100,
            Color::new(0, 80, 255, 100), 1.0,
        );
    }
    gore.new_gore(rng, world, pos, Vec2::new(npc.vx, npc.vy), 314, npc.scale); // :85042
}

/// The shared critter gated-dust burst (`NPC.cs:85118-85130` /
/// `:85257-85269` et al.): 6 dusts of `dust_type` (SpeedX `2*hitDirection`,
/// SpeedY -2), each `Next(2) == 0` ? (noGravity, scale `hi*this.scale`) :
/// (scale `lo*this.scale`). The gate draw fires per dust; both branches
/// write the scale. The LadyBug/Stinkbug use (198, 1.5, 0.8); the snails
/// use (201/202/174, 1.2, 0.7).
fn critter_gated_dust(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
    dust_type: i32,
    hi: f32,
    lo: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    for _ in 0..6 {
        let idx = dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, dust_type, 2.0 * hit_direction, -2.0, 0,
            Color::default(), 1.0,
        );
        let d = dust.slot_mut(idx);
        if rng.next_max(2) == 0 {
            d.no_gravity = true;
            d.scale = hi * npc.scale;
        } else {
            d.scale = lo * npc.scale;
        }
    }
}

/// The 604 LadyBug death (`NPC.cs:85118-85138`, sites 551-552): the shared
/// critter gated-dust burst, then TWO independent `Next(2) == 0`-gated gores
/// 1148/1149 at position (each gate draws before its conditional spawn).
/// Scale defaults to 1.
fn ladybug_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    critter_gated_dust(npc, dust, dust_ctx, rng, world, hit_direction, 198, 1.5, 0.8);
    let pos = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    if rng.next_max(2) == 0 {
        gore.new_gore(rng, world, pos, vel, 1148, 1.0); // :85133
    }
    if rng.next_max(2) == 0 {
        gore.new_gore(rng, world, pos, vel, 1149, 1.0); // :85137
    }
}

/// The 669 Stinkbug death (`NPC.cs:85142-85162`, sites 553-554): the shared
/// critter gated-dust burst, then TWO independent `Next(2) == 0`-gated gores
/// 1275/1276 at position. Same shape as the LadyBug, different types.
fn stinkbug_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    critter_gated_dust(npc, dust, dust_ctx, rng, world, hit_direction, 198, 1.5, 0.8);
    let pos = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    if rng.next_max(2) == 0 {
        gore.new_gore(rng, world, pos, vel, 1275, 1.0); // :85157
    }
    if rng.next_max(2) == 0 {
        gore.new_gore(rng, world, pos, vel, 1276, 1.0); // :85161
    }
}

/// The 359 Snail / 360 GlowingSnail / 655 MagmaSnail hit/death
/// (`NPC.cs:85253-85314`, sites 555-559): the gated-dust burst runs ONLY on
/// death (`life <= 0`), but the gores spawn on EVERY hit (they sit OUTSIDE
/// the death block). The 359 spawns 547/548 (type-201 dust), the 360 spawns
/// 549/550 (type-202 dust), the 655 spawns 1256 (type-174 dust); all use the
/// 1.2/0.7 gated scales. `gore_types` is the per-arm list.
fn snail_hit(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
    dust_type: i32,
    gore_types: &[i32],
) {
    // The dust burst is death-only.
    if npc.hp <= 0 {
        critter_gated_dust(npc, dust, dust_ctx, rng, world, hit_direction, dust_type, 1.2, 0.7);
    }
    // The gores fire on every hit.
    let pos = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    for &ty in gore_types {
        gore.new_gore(rng, world, pos, vel, ty, npc.scale);
    }
}

/// The 154 IceTortoise death (`NPC.cs:85328-85334`, site 560): the 50-dust
/// type-67 burst (SpeedX/Y = `base.velocity`, alpha 90, scale 1.5, then
/// noGravity + velocity ×0.2 post-writes), then the single gore 180 at
/// position.
fn ice_tortoise_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:85328-85333` — the 50-dust type-67 burst (velocity-scaled).
    for _ in 0..50 {
        let idx = dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 67, npc.vx, npc.vy, 90,
            Color::default(), 1.5,
        );
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.velocity *= 0.2;
    }
    gore.new_gore(rng, world, pos, Vec2::new(npc.vx, npc.vy), 180, npc.scale); // :85334
}

/// The 121 Slimer death (`NPC.cs:85438-85455`, site 561): the 50-dust
/// type-14 burst (SpeedX `hitDirection`, SpeedY 0, alpha/color from the NPC,
/// velocity ×2 post-write), then the NPC-81 spawn seam (RETURNED) and the
/// single gore 94 via `NewGoreDirect` with `alpha = this.alpha` post-write.
/// The 81 CorruptSlime shares the dust burst but spawns baby slimes (no
/// gore) — its seam is the wiring wave's.
///
/// `this.color` stand-in: the 81/121 slimes set NO color in `SetDefaults`
/// (`NPC.cs:9838-9851` / `:10466`), so the dust color is `Color::default()`;
/// `alpha` rides `npc.alpha` (55 in SetDefaults).
fn slimer_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
    outcome: &mut HitEffectOutcomeC,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    let alpha = npc.alpha as i32;
    // `:85438-85442` — the 50-dust type-14 burst (velocity x2).
    for _ in 0..50 {
        let idx = dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 14, hit_direction, 0.0, alpha,
            Color::default(), 1.0,
        );
        dust.slot_mut(idx).velocity *= 2.0;
    }
    // `:85447-85450` — the NPC-81 spawn seam (the anchor is the C#'s
    // `(int)(pos.X + width/2), (int)(pos.Y + height)`).
    outcome.slimer_spawn = Some((npc.x + w / 2.0, npc.y + h));
    // `:85451` — the gore 94 + the alpha copy (NewGoreDirect == new_gore).
    let idx = gore.new_gore(rng, world, pos, Vec2::new(npc.vx, npc.vy), 94, npc.scale);
    gore.slot_mut(idx).alpha = alpha;
}

/// The 50 KingSlime death (`NPC.cs:85680-85702`, site 562): the 200-dust
/// type-4 burst (SpeedX `2*hitDirection`, SpeedY -2, alpha 175, color
/// `(0,80,255,100)`), then the single gore 734 at
/// `Center + (-40, -height/2)` (the C# `base.height / 2` is INTEGER
/// division), then the baby-slimes spawn seam (RETURNED). Scale defaults
/// to 1.
fn king_slime_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
    outcome: &mut HitEffectOutcomeC,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:85680-85683` — the 200-dust type-4 burst (blue, alpha 175).
    for _ in 0..200 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 4, 2.0 * hit_direction, -2.0, 175,
            Color::new(0, 80, 255, 100), 1.0,
        );
    }
    // `:85684` — the gore 734 at Center + (-40, -height/2).
    let anchor = center(npc) + Vec2::new(-40.0, -(h as i32 / 2) as f32);
    gore.new_gore(rng, world, anchor, Vec2::new(npc.vx, npc.vy), 734, 1.0);
    // `:85689-85702` — the baby-slimes spawn seam (the count + the per-spawn
    // draws + the spawns are the caller's).
    outcome.king_slime_babies = Some((npc.x, npc.y, w, h));
}

/// The 153 GiantTortoise death (`NPC.cs:85714-85721`, sites 563-566): the
/// 15-dust type-5 burst (SpeedX `2*hitDirection`, SpeedY -2), then the 4
/// gores 177/178/179/179 at position (the 179 repeats). Scale defaults to 1.
fn giant_tortoise_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    for _ in 0..15 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 5, 2.0 * hit_direction, -2.0, 0,
            Color::default(), 1.0,
        );
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, pos, vel, 177, 1.0); // :85718
    gore.new_gore(rng, world, pos, vel, 178, 1.0); // :85719
    gore.new_gore(rng, world, pos, vel, 179, 1.0); // :85720
    gore.new_gore(rng, world, pos, vel, 179, 1.0); // :85721
}

/// The 177 Derpling death (`NPC.cs:85733-85740`, sites 567-570): the
/// 15-dust type-5 burst, then the 4 gores 234/235/236/236 at position (the
/// 236 repeats). Same shape as the GiantTortoise, different types.
fn derpling_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    for _ in 0..15 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 5, 2.0 * hit_direction, -2.0, 0,
            Color::default(), 1.0,
        );
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, pos, vel, 234, 1.0); // :85737
    gore.new_gore(rng, world, pos, vel, 235, 1.0); // :85738
    gore.new_gore(rng, world, pos, vel, 236, 1.0); // :85739
    gore.new_gore(rng, world, pos, vel, 236, 1.0); // :85740
}

/// The 494/495 Crawdad death (`NPC.cs:85752-85758`, sites 571-572): the
/// 15-dust type-5 burst, then the 2 gores `736 + num454` / `735 + num454`
/// where `num454 = (type-494)*2` (494 → 736/735, 495 → 738/737). The 736
/// precedes the 735. Scale defaults to 1.
fn crawdad_death(
    npc: &Npc,
    npc_type: i32,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    for _ in 0..15 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 5, 2.0 * hit_direction, -2.0, 0,
            Color::default(), 1.0,
        );
    }
    // `:85756` — num454 = (type-494)*2.
    let num454 = (npc_type - 494) * 2;
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, pos, vel, 736 + num454, 1.0); // :85757
    gore.new_gore(rng, world, pos, vel, 735 + num454, 1.0); // :85758
}

/// The 496/497 GiantShelly death (`NPC.cs:85771-85775`, site 573): the
/// 15-dust type-5 burst, then the single gore `739 + (type-496)` (496 → 739,
/// 497 → 740). Scale defaults to 1.
fn giant_shelly_death(
    npc: &Npc,
    npc_type: i32,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    for _ in 0..15 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 5, 2.0 * hit_direction, -2.0, 0,
            Color::default(), 1.0,
        );
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, pos, vel, 739 + (npc_type - 496), 1.0); // :85775
}

/// The 498-506 Salamander death (`NPC.cs:85788-85795`, sites 574-576): the
/// 15-dust type-5 burst, then the 3 gores `750 + num459` at position,
/// `741 + num459` at +16 Y, `759 + num459` at +18 Y, where
/// `num459 = type-498` (the nine variants shift all three types). Scale
/// defaults to 1.
fn salamander_death(
    npc: &Npc,
    npc_type: i32,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    for _ in 0..15 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 5, 2.0 * hit_direction, -2.0, 0,
            Color::default(), 1.0,
        );
    }
    // `:85792` — num459 = type-498.
    let num459 = npc_type - 498;
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, pos, vel, 750 + num459, 1.0); // :85793
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 16.0), vel, 741 + num459, 1.0); // :85794
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 18.0), vel, 759 + num459, 1.0); // :85795
}

/// The bat family death (`NPC.cs:85807-85840`, sites 577-586): the 15-dust
/// type-5 burst (SpeedX `2*hitDirection`, SpeedY -2), then the per-type
/// gore list at position — 51 JungleBat → [83], 93 GiantBat → [107],
/// 150 IceBat → [164,165], 152 GiantFlyingFox → [175,176,176], 226
/// FlyingSnake → [317,318,318,319], and the `else` (49 CaveBat) → [82].
/// Scale defaults to 1.
fn bat_family_death(
    npc: &Npc,
    npc_type: i32,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:85807-85810` — the 15-dust type-5 burst.
    for _ in 0..15 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 5, 2.0 * hit_direction, -2.0, 0,
            Color::default(), 1.0,
        );
    }
    // `:85811-85840` — the per-type gore list.
    let gores: &[i32] = match npc_type {
        51 => &[83],
        93 => &[107],
        150 => &[164, 165],
        152 => &[175, 176, 176],
        226 => &[317, 318, 318, 319],
        _ => &[82], // 49 CaveBat (the else arm)
    };
    let vel = Vec2::new(npc.vx, npc.vy);
    for &ty in gores {
        gore.new_gore(rng, world, pos, vel, ty, 1.0);
    }
}

/// The 611/689 Owl / OwlMimic death (`NPC.cs:85852-85858`, sites 587-589):
/// the 15-dust type-5 burst, then the 3 gores 1164/1164/1165 at position
/// (the 1164 repeats). Scale defaults to 1.
fn owl_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    for _ in 0..15 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 5, 2.0 * hit_direction, -2.0, 0,
            Color::default(), 1.0,
        );
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, pos, vel, 1164, 1.0); // :85856
    gore.new_gore(rng, world, pos, vel, 1164, 1.0); // :85857
    gore.new_gore(rng, world, pos, vel, 1165, 1.0); // :85858
}

/// The 624 Gnome death (`NPC.cs:85870-85877`, sites 590-593): the 10-dust
/// type-5 burst, then the 4 gores 1190/1191/1192/1193 at position. Scale
/// defaults to 1.
fn gnome_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    for _ in 0..10 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 5, 2.0 * hit_direction, -2.0, 0,
            Color::default(), 1.0,
        );
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, pos, vel, 1190, 1.0); // :85874
    gore.new_gore(rng, world, pos, vel, 1191, 1.0); // :85875
    gore.new_gore(rng, world, pos, vel, 1192, 1.0); // :85876
    gore.new_gore(rng, world, pos, vel, 1193, 1.0); // :85877
}

/// The 628 Dandelion death (`NPC.cs:85889-85896`, sites 594-597): the
/// 10-dust type-3 burst, then the 4 gores 1205/1206/1206/1207 at position
/// (the 1206 repeats). Scale defaults to 1.
fn dandelion_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:85889-85892` — the 10-dust type-3 burst.
    for _ in 0..10 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 3, 2.0 * hit_direction, -2.0, 0,
            Color::default(), 1.0,
        );
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, pos, vel, 1205, 1.0); // :85893
    gore.new_gore(rng, world, pos, vel, 1206, 1.0); // :85894
    gore.new_gore(rng, world, pos, vel, 1206, 1.0); // :85895
    gore.new_gore(rng, world, pos, vel, 1207, 1.0); // :85896
}

/// The 646-652 GemBunny death (`NPC.cs:85900-85919`, sites 598-599): the
/// 10-dust burst of type `num468` (86+(type-646) for type<652, else 138 for
/// the Amber 652), then the 2 gores `1234 + num471` / `1235 + num471` where
/// `num471 = (type-646)*2`. Scale defaults to 1.
fn gem_bunny_death(
    npc: &Npc,
    npc_type: i32,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:85900-85904` — num468 = 86+(type-646), or 138 for the Amber 652.
    let num468 = if npc_type < 652 { 86 + (npc_type - 646) } else { 138 };
    for _ in 0..10 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, num468, 2.0 * hit_direction, -2.0, 0,
            Color::default(), 1.0,
        );
    }
    // `:85917` — num471 = (type-646)*2.
    let num471 = (npc_type - 646) * 2;
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, pos, vel, 1234 + num471, 1.0); // :85918
    gore.new_gore(rng, world, pos, vel, 1235 + num471, 1.0); // :85919
}

/// The 639-645 GemSquirrel death (`NPC.cs:85923-85940`, site 600): the
/// 10-dust burst of type `num472` (86+(type-639) for type<645, else 138 for
/// the Amber 645), then the single gore `1227 + (type-639)`. Scale defaults
/// to 1.
fn gem_squirrel_death(
    npc: &Npc,
    npc_type: i32,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:85923-85927` — num472 = 86+(type-639), or 138 for the Amber 645.
    let num472 = if npc_type < 645 { 86 + (npc_type - 639) } else { 138 };
    for _ in 0..10 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, num472, 2.0 * hit_direction, -2.0, 0,
            Color::default(), 1.0,
        );
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, pos, vel, 1227 + (npc_type - 639), 1.0); // :85940
}

/// The critter family death (`NPC.cs:85942-86090`, sites 601-654): the
/// 10-dust type-5 burst (SpeedX `2*hitDirection`, SpeedY -2), then the
/// per-type gore list at position. The bunny group (46/303/337/540 → 76/77,
/// the 303 adds a 10-dust type-193 burst, the 540 adds 926), the crab 67
/// (95/95/96), the goldfish 55/230 (553/554), the dolphin 615, the birds,
/// the macaws/toucan/cockatiels (head at position + wings at Center±4), the
/// single-gore critters, the flying fish (587 adds an extra 50-dust burst)
/// and the turtles. Scale defaults to 1.
fn critter_family_death(
    npc: &Npc,
    npc_type: i32,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:85952-85955` — the 10-dust type-5 burst.
    for _ in 0..10 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 5, 2.0 * hit_direction, -2.0, 0,
            Color::default(), 1.0,
        );
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    // `:85956-85971` — the bunny group.
    if npc_type == 46 || npc_type == 303 || npc_type == 337 || npc_type == 540 {
        gore.new_gore(rng, world, pos, vel, 76, 1.0); // :85958
        gore.new_gore(rng, world, pos, vel, 77, 1.0); // :85959
        // `:85960-85966` — the 303 BunnySlimed adds a 10-dust type-193 burst.
        if npc_type == 303 {
            for _ in 0..10 {
                dust.new_dust(
                    rng, dust_ctx, world, pos, w as i32, h as i32, 193, 2.0 * hit_direction, -2.0,
                    0, Color::default(), 1.0,
                );
            }
        }
        // `:85967-85970` — the 540 PartyBunny adds the 926 hat gore.
        if npc_type == 540 {
            gore.new_gore(rng, world, pos, vel, 926, 1.0); // :85969
        }
    }
    // `:85972-85977` — the crab 67.
    else if npc_type == 67 {
        gore.new_gore(rng, world, pos, vel, 95, 1.0); // :85974
        gore.new_gore(rng, world, pos, vel, 95, 1.0); // :85975
        gore.new_gore(rng, world, pos, vel, 96, 1.0); // :85976
    }
    // `:85978-85982` — the goldfish 55/230.
    else if npc_type == 55 || npc_type == 230 {
        gore.new_gore(rng, world, pos, vel, 553, 1.0); // :85980
        gore.new_gore(rng, world, pos, vel, 554, 1.0); // :85981
    }
    // `:85983-85989` — the dolphin 615.
    else if npc_type == 615 {
        gore.new_gore(rng, world, pos, vel, 1198, 1.0); // :85985
        gore.new_gore(rng, world, pos, vel, 1199, 1.0); // :85986
        gore.new_gore(rng, world, pos, vel, 1200, 1.0); // :85987
        gore.new_gore(rng, world, pos, vel, 1200, 1.0); // :85988
    }
    // `:85990-85993` — the bird 74.
    else if npc_type == 74 {
        gore.new_gore(rng, world, pos, vel, 100, 1.0); // :85992
    }
    // `:85994-85997` — the blue bird 297.
    else if npc_type == 297 {
        gore.new_gore(rng, world, pos, vel, 431, 1.0); // :85996
    }
    // `:85998-86001` — the red bird 298.
    else if npc_type == 298 {
        gore.new_gore(rng, world, pos, vel, 432, 1.0); // :86000
    }
    // `:86002-86007` — the scarlet macaw 671 (the head at position, the
    // wings at Center±4).
    else if npc_type == 671 {
        let c = center(npc);
        gore.new_gore(rng, world, pos, vel, 1277, 1.0); // :86004
        gore.new_gore(rng, world, c + Vec2::new(4.0, 0.0), vel, 1279, 1.0); // :86005
        gore.new_gore(rng, world, c + Vec2::new(-4.0, 0.0), vel, 1279, 1.0); // :86006
    }
    // `:86008-86013` — the blue macaw 672.
    else if npc_type == 672 {
        let c = center(npc);
        gore.new_gore(rng, world, pos, vel, 1280, 1.0); // :86010
        gore.new_gore(rng, world, c + Vec2::new(4.0, 0.0), vel, 1281, 1.0); // :86011
        gore.new_gore(rng, world, c + Vec2::new(-4.0, 0.0), vel, 1281, 1.0); // :86012
    }
    // `:86014-86019` — the toucan 673.
    else if npc_type == 673 {
        let c = center(npc);
        gore.new_gore(rng, world, pos, vel, 1282, 1.0); // :86016
        gore.new_gore(rng, world, c + Vec2::new(4.0, 0.0), vel, 1283, 1.0); // :86017
        gore.new_gore(rng, world, c + Vec2::new(-4.0, 0.0), vel, 1283, 1.0); // :86018
    }
    // `:86020-86025` — the yellow cockatiel 674.
    else if npc_type == 674 {
        let c = center(npc);
        gore.new_gore(rng, world, pos, vel, 1284, 1.0); // :86022
        gore.new_gore(rng, world, c + Vec2::new(4.0, 0.0), vel, 1285, 1.0); // :86023
        gore.new_gore(rng, world, c + Vec2::new(-4.0, 0.0), vel, 1285, 1.0); // :86024
    }
    // `:86026-86031` — the gray cockatiel 675.
    else if npc_type == 675 {
        let c = center(npc);
        gore.new_gore(rng, world, pos, vel, 1286, 1.0); // :86028
        gore.new_gore(rng, world, c + Vec2::new(4.0, 0.0), vel, 1287, 1.0); // :86029
        gore.new_gore(rng, world, c + Vec2::new(-4.0, 0.0), vel, 1287, 1.0); // :86030
    }
    // `:86032-86035` — the squirrel 299.
    else if npc_type == 299 {
        gore.new_gore(rng, world, pos, vel, 433, 1.0); // :86034
    }
    // `:86036-86039` — the red squirrel 538.
    else if npc_type == 538 {
        gore.new_gore(rng, world, pos, vel, 897, 1.0); // :86038
    }
    // `:86040-86043` — the mouse 300.
    else if npc_type == 300 {
        gore.new_gore(rng, world, pos, vel, 434, 1.0); // :86042
    }
    // `:86044-86047` — the rat 610.
    else if npc_type == 610 {
        gore.new_gore(rng, world, pos, vel, 1163, 1.0); // :86046
    }
    // `:86048-86051` — the seahorse 626.
    else if npc_type == 626 {
        gore.new_gore(rng, world, pos, vel, 1197, 1.0); // :86050
    }
    // `:86052-86055` — the anglerfish 102.
    else if npc_type == 102 {
        gore.new_gore(rng, world, pos, vel, 116, 1.0); // :86054
    }
    // `:86056-86060` — the flying fish 224.
    else if npc_type == 224 {
        gore.new_gore(rng, world, pos, vel, 312, 1.0); // :86058
        gore.new_gore(rng, world, pos, vel, 313, 1.0); // :86059
    }
    // `:86061-86069` — the eyeball flying fish 587 (an EXTRA 50-dust type-5
    // burst at SpeedX 2.5*dir, SpeedY -2.5 before the gores).
    else if npc_type == 587 {
        for _ in 0..50 {
            dust.new_dust(
                rng, dust_ctx, world, pos, w as i32, h as i32, 5, 2.5 * hit_direction, -2.5, 0,
                Color::default(), 1.0,
            );
        }
        gore.new_gore(rng, world, pos, vel, 1104, 1.0); // :86067
        gore.new_gore(rng, world, pos, vel, 1105, 1.0); // :86068
    }
    // `:86070-86076` — the turtle 616.
    else if npc_type == 616 {
        gore.new_gore(rng, world, pos, vel, 1166, 1.0); // :86072
        gore.new_gore(rng, world, pos, vel, 1167, 1.0); // :86073
        gore.new_gore(rng, world, pos, vel, 1168, 1.0); // :86074
        gore.new_gore(rng, world, pos, vel, 1168, 1.0); // :86075
    }
    // `:86077-86083` — the jungle turtle 617.
    else if npc_type == 617 {
        gore.new_gore(rng, world, pos, vel, 1169, 1.0); // :86079
        gore.new_gore(rng, world, pos, vel, 1170, 1.0); // :86080
        gore.new_gore(rng, world, pos, vel, 1171, 1.0); // :86081
        gore.new_gore(rng, world, pos, vel, 1171, 1.0); // :86082
    }
    // `:86084-86090` — the sea turtle 625.
    else if npc_type == 625 {
        gore.new_gore(rng, world, pos, vel, 1194, 1.0); // :86086
        gore.new_gore(rng, world, pos, vel, 1195, 1.0); // :86087
        gore.new_gore(rng, world, pos, vel, 1196, 1.0); // :86088
        gore.new_gore(rng, world, pos, vel, 1196, 1.0); // :86089
    }
}

/// The penguin family death (`NPC.cs:86102-86126`, sites 655-663): the
/// 10-dust type-5 burst (SpeedX `2*hitDirection`, SpeedY -2), then the
/// per-type gores at position — 148 Penguin → [160,161], 149 PenguinBlack →
/// [162,163], 168 CorruptPenguin → [215,216], 470 CrimsonPenguin →
/// [673,672,674] (the 673 precedes the 672). Scale defaults to 1.
fn penguin_family_death(
    npc: &Npc,
    npc_type: i32,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:86102-86105` — the 10-dust type-5 burst.
    for _ in 0..10 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 5, 2.0 * hit_direction, -2.0, 0,
            Color::default(), 1.0,
        );
    }
    // `:86106-86126` — the per-type gore list.
    let gores: &[i32] = match npc_type {
        148 => &[160, 161],
        149 => &[162, 163],
        168 => &[215, 216],
        _ => &[673, 672, 674], // 470 CrimsonPenguin
    };
    let vel = Vec2::new(npc.vx, npc.vy);
    for &ty in gores {
        gore.new_gore(rng, world, pos, vel, ty, 1.0);
    }
}

/// The 361 Frog death (`NPC.cs:86138-86143`, sites 664-665): the 10-dust
/// type-5 burst, then the 2 gores 551/552 at position. Scale defaults to 1.
fn frog_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    for _ in 0..10 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 5, 2.0 * hit_direction, -2.0, 0,
            Color::default(), 1.0,
        );
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, pos, vel, 551, 1.0); // :86142
    gore.new_gore(rng, world, pos, vel, 552, 1.0); // :86143
}

/// The 607 Pupfish death (`NPC.cs:86183-86188`, sites 666-667): the
/// 10-dust type-5 burst, then the 2 gores 1152/1153 at position. Scale
/// defaults to 1.
fn pupfish_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    for _ in 0..10 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 5, 2.0 * hit_direction, -2.0, 0,
            Color::default(), 1.0,
        );
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, pos, vel, 1152, 1.0); // :86187
    gore.new_gore(rng, world, pos, vel, 1153, 1.0); // :86188
}

/// The 688 Pufferfish death (`NPC.cs:86200-86215`, sites 668-673): the
/// 10-dust type-5 burst, then the `ai[2] == 1` (puffed) branch spawns 4
/// gores 1388/1388/1389/1390, else 2 gores 1386/1387. Scale defaults to 1.
fn pufferfish_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    for _ in 0..10 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 5, 2.0 * hit_direction, -2.0, 0,
            Color::default(), 1.0,
        );
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    // `:86204-86215` — the puffed branch.
    if npc.ai[2] == 1.0 {
        gore.new_gore(rng, world, pos, vel, 1388, 1.0); // :86206
        gore.new_gore(rng, world, pos, vel, 1388, 1.0); // :86207
        gore.new_gore(rng, world, pos, vel, 1389, 1.0); // :86208
        gore.new_gore(rng, world, pos, vel, 1390, 1.0); // :86209
    } else {
        gore.new_gore(rng, world, pos, vel, 1386, 1.0); // :86213
        gore.new_gore(rng, world, pos, vel, 1387, 1.0); // :86214
    }
}

/// The 602/603 Seagull death (`NPC.cs:86281-86288`, sites 674-675): the
/// 10-dust type-5 burst, then the 2 gores 1150/1151 at position. Scale
/// defaults to 1.
fn seagull_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    for _ in 0..10 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 5, 2.0 * hit_direction, -2.0, 0,
            Color::default(), 1.0,
        );
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, pos, vel, 1150, 1.0); // :86287
    gore.new_gore(rng, world, pos, vel, 1151, 1.0); // :86288
}

/// The 366/367 Scorpion death (`NPC.cs:86227-86241`, sites 676-681): the
/// 10-dust type-203 burst (SpeedX `2*hitDirection`, SpeedY -2), then the
/// per-type 3 gores — 366 ScorpionBlack → [561,562,563], 367 Scorpion →
/// [564,565,566]. Scale defaults to 1.
fn scorpion_death(
    npc: &Npc,
    npc_type: i32,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:86227-86230` — the 10-dust type-203 burst.
    for _ in 0..10 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 203, 2.0 * hit_direction, -2.0, 0,
            Color::default(), 1.0,
        );
    }
    // `:86231-86242` — the per-type gore list.
    let gores: &[i32] = if npc_type == 366 { &[561, 562, 563] } else { &[564, 565, 566] };
    let vel = Vec2::new(npc.vx, npc.vy);
    for &ty in gores {
        gore.new_gore(rng, world, pos, vel, ty, 1.0);
    }
}

/// The 608/609 Grebe death (`NPC.cs:86301-86309`, sites 682-684): the
/// 10-dust type-5 burst, then the 3 gores 1154/1155/1156 at position. Scale
/// defaults to 1.
fn grebe_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    for _ in 0..10 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 5, 2.0 * hit_direction, -2.0, 0,
            Color::default(), 1.0,
        );
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, pos, vel, 1154, 1.0); // :86307
    gore.new_gore(rng, world, pos, vel, 1155, 1.0); // :86308
    gore.new_gore(rng, world, pos, vel, 1156, 1.0); // :86309
}

/// The 362-365 Duck family death (`NPC.cs:86254-86268`, sites 685-690): the
/// 10-dust type-5 burst, then the per-type 3 gores — 362/363 Duck →
/// [555,556,557], 364/365 DuckWhite → [558,559,560]. Scale defaults to 1.
fn duck_family_death(
    npc: &Npc,
    npc_type: i32,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    for _ in 0..10 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 5, 2.0 * hit_direction, -2.0, 0,
            Color::default(), 1.0,
        );
    }
    // `:86258-86269` — the per-type gore list.
    let gores: &[i32] = if npc_type == 362 || npc_type == 363 {
        &[555, 556, 557]
    } else {
        &[558, 559, 560]
    };
    let vel = Vec2::new(npc.vx, npc.vy);
    for &ty in gores {
        gore.new_gore(rng, world, pos, vel, ty, 1.0);
    }
}

/// The 348/349 Nutcracker death (`NPC.cs:86322-86330`, sites 691-695): the
/// 10-dust type-5 burst, then the 5 gores 528/529/530/531/532 at position.
/// Scale defaults to 1.
fn nutcracker_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    for _ in 0..10 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 5, 2.0 * hit_direction, -2.0, 0,
            Color::default(), 1.0,
        );
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, pos, vel, 528, 1.0); // :86326
    gore.new_gore(rng, world, pos, vel, 529, 1.0); // :86327
    gore.new_gore(rng, world, pos, vel, 530, 1.0); // :86328
    gore.new_gore(rng, world, pos, vel, 531, 1.0); // :86329
    gore.new_gore(rng, world, pos, vel, 532, 1.0); // :86330
}

/// The 351 Krampus death (`NPC.cs:86342-86350`, sites 696-700): the
/// 10-dust type-5 burst, then the 5 gores 538/539/540/541/542 at position.
/// Scale defaults to 1.
fn krampus_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    for _ in 0..10 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 5, 2.0 * hit_direction, -2.0, 0,
            Color::default(), 1.0,
        );
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, pos, vel, 538, 1.0); // :86346
    gore.new_gore(rng, world, pos, vel, 539, 1.0); // :86347
    gore.new_gore(rng, world, pos, vel, 540, 1.0); // :86348
    gore.new_gore(rng, world, pos, vel, 541, 1.0); // :86349
    gore.new_gore(rng, world, pos, vel, 542, 1.0); // :86350
}

/// The 350 ElfArcher death (`NPC.cs:86362-86369`, sites 701-704): the
/// 10-dust type-5 burst, then the 4 gores 533/534/535/536 at position.
/// Scale defaults to 1.
fn elf_archer_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    for _ in 0..10 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 5, 2.0 * hit_direction, -2.0, 0,
            Color::default(), 1.0,
        );
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, pos, vel, 533, 1.0); // :86366
    gore.new_gore(rng, world, pos, vel, 534, 1.0); // :86367
    gore.new_gore(rng, world, pos, vel, 535, 1.0); // :86368
    gore.new_gore(rng, world, pos, vel, 536, 1.0); // :86369
}

/// The corrupt/crimson critter family death (`NPC.cs:86381-86407`, sites
/// 705-712): the 10-dust type-5 burst, then the per-type gores at position —
/// 57 CorruptGoldfish → [84], 58 Piranha → [85], 464 CrimsonBunny →
/// [668,669], 465 CrimsonGoldfish → [670,671], and the `else` (47
/// CorruptBunny) → [78,79]. Scale defaults to 1.
fn corrupt_critter_death(
    npc: &Npc,
    npc_type: i32,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    for _ in 0..10 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 5, 2.0 * hit_direction, -2.0, 0,
            Color::default(), 1.0,
        );
    }
    // `:86385-86407` — the per-type gore list.
    let gores: &[i32] = match npc_type {
        57 => &[84],
        58 => &[85],
        464 => &[668, 669],
        465 => &[670, 671],
        _ => &[78, 79], // 47 CorruptBunny (the else arm)
    };
    let vel = Vec2::new(npc.vx, npc.vy);
    for &ty in gores {
        gore.new_gore(rng, world, pos, vel, ty, 1.0);
    }
}

/// The crimson monster family death (`NPC.cs:86419-86446`, sites 713-722):
/// the 50-dust type-5 burst (SpeedX `2*hitDirection`, SpeedY -2, alpha from
/// the NPC), then the per-type gores at position — 173 Crimera → [223,224],
/// 174 Herpling → [225,226], 181 FaceMonster → [237], 182 FloatyGross →
/// [238,239], 268 IchorSticker → [403,404,405]. Scale defaults to 1.
fn crimson_monster_death(
    npc: &Npc,
    npc_type: i32,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    let alpha = npc.alpha as i32;
    // `:86419-86422` — the 50-dust type-5 burst (alpha from the NPC).
    for _ in 0..50 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 5, 2.0 * hit_direction, -2.0, alpha,
            Color::default(), 1.0,
        );
    }
    // `:86423-86447` — the per-type gore list.
    let gores: &[i32] = match npc_type {
        173 => &[223, 224],
        174 => &[225, 226],
        181 => &[237],
        182 => &[238, 239],
        _ => &[403, 404, 405], // 268 IchorSticker
    };
    let vel = Vec2::new(npc.vx, npc.vy);
    for &ty in gores {
        gore.new_gore(rng, world, pos, vel, ty, 1.0);
    }
}

/// The demon-eye family death (`NPC.cs:86459-86502`, sites 723-738): the
/// 50-dust type-5 burst (SpeedX `2*hitDirection`, SpeedY -2), then 2 gores —
/// the head at position and the tail at `position + (14, 0)`, both at
/// `this.scale`. The per-type (head, tail) pairs: 190 CataractEye → (249,2),
/// 191 SleepyEye → (248,2), 192 DialatedEye → (247,2), 193 GreenEye →
/// (252,253), 194 PurpleEye → (250,251), 317 DemonEyeOwl → (447,448),
/// 318 DemonEyeSpaceship → (449,450), and the `else` (2 DemonEye) → (1,2).
fn demon_eye_death(
    npc: &Npc,
    npc_type: i32,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:86459-86462` — the 50-dust type-5 burst.
    for _ in 0..50 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 5, 2.0 * hit_direction, -2.0, 0,
            Color::default(), 1.0,
        );
    }
    // `:86463-86502` — the per-type (head, tail) pair.
    let (head, tail) = match npc_type {
        190 => (249, 2),
        191 => (248, 2),
        192 => (247, 2),
        193 => (252, 253),
        194 => (250, 251),
        317 => (447, 448),
        318 => (449, 450),
        _ => (1, 2), // 2 DemonEye (the else arm)
    };
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, head, s);
    gore.new_gore(rng, world, pos + Vec2::new(14.0, 0.0), vel, tail, s);
}

/// The 157 Arapaima death (`NPC.cs:86514-86519`, sites 739-740): the
/// 50-dust type-5 burst, then the 2 gores 186 at position and 187 at
/// `position + (14, 0)`. Scale defaults to 1.
fn arapaima_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    for _ in 0..50 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 5, 2.0 * hit_direction, -2.0, 0,
            Color::default(), 1.0,
        );
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, pos, vel, 186, 1.0); // :86518
    gore.new_gore(rng, world, pos + Vec2::new(14.0, 0.0), vel, 187, 1.0); // :86519
}

/// The 133 WanderingEye hit/death (`NPC.cs:86523-86543`, sites 741-743):
/// on a HIT (`life > 0`) below half life with `localAI[0] == 0`, latch
/// `localAI[0] = 1` and spawn gore 1 (the one-time "eye pop"); on DEATH the
/// 50-dust type-5 burst then the 2 gores 155/155 at position and
/// `position + (0, 14)`. Scale defaults to 1. The hit-half dust loop is
/// dmg-scaled (the wiring wave's); the gore latch is ported here.
fn wandering_eye_hit(
    npc: &mut Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    let vel = Vec2::new(npc.vx, npc.vy);
    if npc.hp > 0 {
        // `:86529-86533` — the below-half-life eye-pop latch.
        if (npc.hp as f32) < (npc.max_hp as f32) * 0.5 && npc.local_ai[0] == 0.0 {
            npc.local_ai[0] = 1.0;
            gore.new_gore(rng, world, pos, vel, 1, 1.0); // :86532
        }
    } else {
        // `:86537-86540` — the 50-dust type-5 death burst.
        for _ in 0..50 {
            dust.new_dust(
                rng, dust_ctx, world, pos, w as i32, h as i32, 5, 2.0 * hit_direction, -2.0, 0,
                Color::default(), 1.0,
            );
        }
        gore.new_gore(rng, world, pos, vel, 155, 1.0); // :86541
        gore.new_gore(rng, world, pos + Vec2::new(0.0, 14.0), vel, 155, 1.0); // :86542
    }
}

/// The antlion family death (`NPC.cs:86545-86605`, sites 744-765): the
/// type-250 dust burst (SpeedX `2*hitDirection`, SpeedY -2; count 50, or 10
/// for the 582 larva), then the per-type gores at position — 69 Antlion →
/// [97,98], 580 WalkingAntlion → [1094,1095,1096,1097], 581 FlyingAntlion →
/// [1098,1099,1100,1101,1101], 508 GiantWalkingAntlion → [811,812,813,814],
/// 509 GiantFlyingAntlion → [815,816,817,818,818], 582 LarvaeAntlion →
/// [1102,1103]. Scale defaults to 1.
fn antlion_death(
    npc: &Npc,
    npc_type: i32,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:86547-86553` — the 582 larva uses a 10-dust burst, the rest 50.
    let count = if npc_type == 582 { 10 } else { 50 };
    for _ in 0..count {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 250, 2.0 * hit_direction, -2.0, 0,
            Color::default(), 1.0,
        );
    }
    // `:86566-86605` — the per-type gore list.
    let gores: &[i32] = match npc_type {
        69 => &[97, 98],
        580 => &[1094, 1095, 1096, 1097],
        581 => &[1098, 1099, 1100, 1101, 1101],
        508 => &[811, 812, 813, 814],
        509 => &[815, 816, 817, 818, 818],
        _ => &[1102, 1103], // 582 LarvaeAntlion
    };
    let vel = Vec2::new(npc.vx, npc.vy);
    for &ty in gores {
        gore.new_gore(rng, world, pos, vel, ty, 1.0);
    }
}

/// The 61 Vulture death (`NPC.cs:86617-86623`, sites 766-768): the 50-dust
/// type-5 burst, then the 3 gores 86 at position and 87/88 at
/// `position + (14, 0)`. Scale defaults to 1.
fn vulture_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    for _ in 0..50 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 5, 2.0 * hit_direction, -2.0, 0,
            Color::default(), 1.0,
        );
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, pos, vel, 86, 1.0); // :86621
    gore.new_gore(rng, world, pos + Vec2::new(14.0, 0.0), vel, 87, 1.0); // :86622
    gore.new_gore(rng, world, pos + Vec2::new(14.0, 0.0), vel, 88, 1.0); // :86623
}

/// The 301 Raven death (`NPC.cs:86635-86640`, sites 769-770): the 50-dust
/// type-5 burst, then the 2 gores 438 at position and 439 at
/// `position + (14, 0)`. Scale defaults to 1.
fn raven_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    for _ in 0..50 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 5, 2.0 * hit_direction, -2.0, 0,
            Color::default(), 1.0,
        );
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, pos, vel, 438, 1.0); // :86639
    gore.new_gore(rng, world, pos + Vec2::new(14.0, 0.0), vel, 439, 1.0); // :86640
}

/// The 252 Parrot death (`NPC.cs:86652-86657`, sites 771-772): the 50-dust
/// type-5 burst, then the 2 gores 373/374 at position. Scale defaults to 1.
fn parrot_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    for _ in 0..50 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 5, 2.0 * hit_direction, -2.0, 0,
            Color::default(), 1.0,
        );
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    gore.new_gore(rng, world, pos, vel, 373, 1.0); // :86656
    gore.new_gore(rng, world, pos, vel, 374, 1.0); // :86657
}

/// The shared 5-gore blood-zombie ladder (`NPC.cs:86669-86708`): the
/// 50-dust type-5 burst (SpeedX `2.5*hitDirection`, SpeedY -2.5), then 5
/// gores at the +0/+20/+20/+34/+34 Y ladder at `this.scale` — the head, the
/// middle pair, the tail pair. The per-type triple `(head, middle, tail)`:
/// 489 BloodZombie → (722,723,724), 586 ZombieMerman → (1106,1107,1108),
/// 534 DemonTaxCollector → (894,895,896).
fn blood_zombie_ladder(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
    head: i32,
    middle: i32,
    tail: i32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // The 50-dust type-5 burst (SpeedX 2.5*dir).
    for _ in 0..50 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 5, 2.5 * hit_direction, -2.5, 0,
            Color::default(), 1.0,
        );
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, head, s);
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, middle, s);
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, middle, s);
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, tail, s);
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, tail, s);
}

/// The 489 BloodZombie death (`NPC.cs:86675-86679`, sites 773-777): the
/// shared 5-gore ladder with (722,723,724).
fn blood_zombie_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    blood_zombie_ladder(npc, dust, dust_ctx, gore, rng, world, hit_direction, 722, 723, 724);
}

/// The 586 ZombieMerman death (`NPC.cs:86683-86687`, sites 778-782): the
/// shared 5-gore ladder with (1106,1107,1108).
fn zombie_merman_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    blood_zombie_ladder(npc, dust, dust_ctx, gore, rng, world, hit_direction, 1106, 1107, 1108);
}

/// The 534 DemonTaxCollector death (`NPC.cs:86704-86708`, sites 783-787):
/// the shared 5-gore ladder with (894,895,896).
fn demon_tax_collector_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    blood_zombie_ladder(npc, dust, dust_ctx, gore, rng, world, hit_direction, 894, 895, 896);
}

/// The 490 Drippler death (`NPC.cs:86720-86728`, sites 788-792): the
/// 50-dust type-5 burst (SpeedX `2.5*hitDirection`, SpeedY -2.5), then the
/// 5 gores at position at `this.scale` — 725/726/727 fixed, then TWO gores
/// of type `725 + Next(3)` (each type draw fires before its spawn).
fn drippler_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:86720-86723` — the 50-dust type-5 burst (SpeedX 2.5*dir).
    for _ in 0..50 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 5, 2.5 * hit_direction, -2.5, 0,
            Color::default(), 1.0,
        );
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, 725, s); // :86724
    gore.new_gore(rng, world, pos, vel, 726, s); // :86725
    gore.new_gore(rng, world, pos, vel, 727, s); // :86726
    // `:86727-86728` — the two Next(3) type draws.
    let ty = 725 + rng.next_max(3);
    gore.new_gore(rng, world, pos, vel, ty, s);
    let ty = 725 + rng.next_max(3);
    gore.new_gore(rng, world, pos, vel, ty, s);
}

/// The shark family death (`NPC.cs:86730-86785`, sites 793-816): the
/// 75-dust type-5 burst (SpeedX `2*hitDirection`, SpeedY -2), then 4 gores
/// at velocity ×0.8 — the head at position and the three tail gores at
/// `position + (14, 0)`. The per-type quad: 65 Shark → [89,90,91,92],
/// 692 Orca → [1403,1404,1405,1406], 542 SandShark → [944,945,946,947],
/// 543 SandsharkCorrupt → [948,949,950,951], 544 SandsharkCrimson →
/// [952,953,954,955], 545 SandsharkHallow → [956,957,958,959]. Scale
/// defaults to 1.
fn shark_family_death(
    npc: &Npc,
    npc_type: i32,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:86740-86743` — the 75-dust type-5 burst.
    for _ in 0..75 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 5, 2.0 * hit_direction, -2.0, 0,
            Color::default(), 1.0,
        );
    }
    // `:86744-86785` — the per-type gore quad.
    let gores: &[i32] = match npc_type {
        65 => &[89, 90, 91, 92],
        692 => &[1403, 1404, 1405, 1406],
        542 => &[944, 945, 946, 947],
        543 => &[948, 949, 950, 951],
        544 => &[952, 953, 954, 955],
        _ => &[956, 957, 958, 959], // 545 SandsharkHallow
    };
    let vel = Vec2::new(npc.vx, npc.vy) * 0.8;
    gore.new_gore(rng, world, pos, vel, gores[0], 1.0);
    gore.new_gore(rng, world, pos + Vec2::new(14.0, 0.0), vel, gores[1], 1.0);
    gore.new_gore(rng, world, pos + Vec2::new(14.0, 0.0), vel, gores[2], 1.0);
    gore.new_gore(rng, world, pos + Vec2::new(14.0, 0.0), vel, gores[3], 1.0);
}

/// The general 5-gore death ladder: a dust burst (`dust_count` dusts of
/// `dust_type`, SpeedX `speed_x*hitDirection`, SpeedY `speed_y`), then 5
/// gores at the +0/+20/+20/+34/+34 Y ladder at `this.scale` — the head, the
/// middle pair, the tail pair. Used by the nymph/icy-merman/gingerbread
/// arms (the blood-zombie ladder is the SpeedX-2.5 special case).
fn five_gore_ladder(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
    dust_type: i32,
    dust_count: i32,
    speed_x: f32,
    speed_y: f32,
    head: i32,
    middle: i32,
    tail: i32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    for _ in 0..dust_count {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, dust_type, speed_x * hit_direction,
            speed_y, 0, Color::default(), 1.0,
        );
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, head, s);
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, middle, s);
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, middle, s);
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, tail, s);
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, tail, s);
}

/// The 195/196 LostGirl/Nymph death (`NPC.cs:86797-86805`, sites 817-821):
/// the 75-dust type-5 burst (SpeedX `2*hitDirection`, SpeedY -2), then the
/// 5-gore ladder (254,255,256).
fn nymph_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    five_gore_ladder(npc, dust, dust_ctx, gore, rng, world, hit_direction, 5, 75, 2.0, -2.0, 254, 255, 256);
}

/// The 206 IcyMerman death (`NPC.cs:86838-86846`, sites 822-826): the
/// 75-dust type-137 burst (SpeedX `2*hitDirection`, SpeedY -2), then the
/// 5-gore ladder (273,274,275).
fn icy_merman_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    five_gore_ladder(npc, dust, dust_ctx, gore, rng, world, hit_direction, 137, 75, 2.0, -2.0, 273, 274, 275);
}

/// The 198/199 Lihzahrd death (`NPC.cs:86817-86826`, sites 827-832): the
/// 75-dust type-5 burst (SpeedX `2*hitDirection`, SpeedY -2), then the 6
/// gores at `this.scale` — 258 at position, then 259/259/260/260/261 ALL at
/// +20 Y (no +34 tier).
fn lihzahrd_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    for _ in 0..75 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 5, 2.0 * hit_direction, -2.0, 0,
            Color::default(), 1.0,
        );
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    let tail = Vec2::new(pos.x, pos.y + 20.0);
    gore.new_gore(rng, world, pos, vel, 258, s); // :86821
    gore.new_gore(rng, world, tail, vel, 259, s); // :86822
    gore.new_gore(rng, world, tail, vel, 259, s); // :86823
    gore.new_gore(rng, world, tail, vel, 260, s); // :86824
    gore.new_gore(rng, world, tail, vel, 260, s); // :86825
    gore.new_gore(rng, world, tail, vel, 261, s); // :86826
}

/// The 342 GingerbreadMan death (`NPC.cs:86858-86866`, sites 833-837): the
/// 50-dust type-0 burst (SpeedX `2.5*hitDirection`, SpeedY -2.5), then the
/// 5-gore ladder (500,501,502).
fn gingerbread_man_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    five_gore_ladder(npc, dust, dust_ctx, gore, rng, world, hit_direction, 0, 50, 2.5, -2.5, 500, 501, 502);
}

/// The 338-340 ZombieElf death (`NPC.cs:86878-86896`, sites 838-843): the
/// 50-dust type-5 burst (SpeedX `2.5*hitDirection`, SpeedY -2.5), then the
/// per-type head at position (338 → 494, 339 → 495, 340 → 496) and the
/// shared tails 497/498 at +20 Y and 499 at +34 Y, all at `this.scale`.
fn zombie_elf_death(
    npc: &Npc,
    npc_type: i32,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    for _ in 0..50 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 5, 2.5 * hit_direction, -2.5, 0,
            Color::default(), 1.0,
        );
    }
    // `:86882-86892` — the per-type head.
    let head = match npc_type {
        338 => 494,
        339 => 495,
        _ => 496, // 340 ZombieElfGirl
    };
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, head, s);
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 497, s); // :86894
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 498, s); // :86895
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, 499, s); // :86896
}

/// The 343 Yeti death (`NPC.cs:86908-86915`, sites 844-847): the 50-dust
/// type-5 burst (SpeedX `2.5*hitDirection`, SpeedY -2.5), then the 4 gores
/// 503 at position, 504/505 at +20 Y (the DISTINCT pair), 506 at +34 Y, all
/// at `this.scale`.
fn yeti_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    for _ in 0..50 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 5, 2.5 * hit_direction, -2.5, 0,
            Color::default(), 1.0,
        );
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    gore.new_gore(rng, world, pos, vel, 503, s); // :86912
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 504, s); // :86913
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 20.0), vel, 505, s); // :86914
    gore.new_gore(rng, world, Vec2::new(pos.x, pos.y + 34.0), vel, 506, s); // :86915
}

/// The 691 MossZombie death (`NPC.cs:86927-86935`, sites 848-852): the
/// 50-dust type-5 burst (SpeedX `2.5*hitDirection`, SpeedY -2.5), then the
/// 5-gore ladder (1391,1392,1393).
fn moss_zombie_death(
    npc: &Npc,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
) {
    five_gore_ladder(npc, dust, dust_ctx, gore, rng, world, hit_direction, 5, 50, 2.5, -2.5, 1391, 1392, 1393);
}

/// The big zombie block death (`NPC.cs:86937-87139`): the 50-dust type-5
/// burst (SpeedX `2.5*hitDirection`, SpeedY -2.5), then the per-type gores.
/// This commit ports the 104 Werewolf (sites 853-857: ladder 117/118/119)
/// and the 632 MaggotZombie (sites 858-862: ladder 1214/1215/1216 + the
/// maggot-spawn seam RETURNED). The remaining zombie types land with the
/// next batches. The 187/433 SlimedZombie adds a 25-dust type-4 blue burst
/// (the wiring wave's hit-half detail is noted per arm).
fn zombie_block_death(
    npc: &Npc,
    npc_type: i32,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
    hit_direction: f32,
    outcome: &mut HitEffectOutcomeC,
) {
    let (w, h) = npc.size();
    let pos = Vec2::new(npc.x, npc.y);
    // `:86969-86972` — the 50-dust type-5 burst (SpeedX 2.5*dir).
    for _ in 0..50 {
        dust.new_dust(
            rng, dust_ctx, world, pos, w as i32, h as i32, 5, 2.5 * hit_direction, -2.5, 0,
            Color::default(), 1.0,
        );
    }
    let vel = Vec2::new(npc.vx, npc.vy);
    let s = npc.scale;
    let mid = Vec2::new(pos.x, pos.y + 20.0);
    let low = Vec2::new(pos.x, pos.y + 34.0);
    // `:86980-86987` — the 104 Werewolf ladder.
    if npc_type == 104 {
        gore.new_gore(rng, world, pos, vel, 117, s); // :86982
        gore.new_gore(rng, world, mid, vel, 118, s); // :86983
        gore.new_gore(rng, world, mid, vel, 118, s); // :86984
        gore.new_gore(rng, world, low, vel, 119, s); // :86985
        gore.new_gore(rng, world, low, vel, 119, s); // :86986
        return;
    }
    // `:86989-87007` — the 632 MaggotZombie ladder + the maggot-spawn seam.
    if npc_type == 632 {
        gore.new_gore(rng, world, pos, vel, 1214, s); // :86991
        gore.new_gore(rng, world, mid, vel, 1215, s); // :86992
        gore.new_gore(rng, world, mid, vel, 1215, s); // :86993
        gore.new_gore(rng, world, low, vel, 1216, s); // :86994
        gore.new_gore(rng, world, low, vel, 1216, s); // :86995
        // `:86997-87006` — the maggot-spawn seam (the count + the per-spawn
        // draws + the spawns are the caller's).
        outcome.maggot_zombie_spawns = Some((npc.x, npc.y, w, h));
        return;
    }
}

/// `NPC.HitEffect(hitDirection, dmg)` (`NPC.cs:81654+`) — wave C: the third
/// batch of gore sites, `:82206` onward. `npc.hp` is `this.life` AT THE CALL
/// (the strike writes it before calling, `:81488`); the death (`life <= 0`)
/// halves carry gore here, plus the 437 tablet's `dmg == 9999.0` burst (a
/// dmg-gated arm, not life-gated). `hit_direction` is the C# `hitDirection`
/// (a velocity offset for the dust, no extra draw); `dmg` rides the tablet
/// gate (the dmg-scaled hit-half dust loops are the wiring wave's).
///
/// The dispatch grows one arm per commit; the wiring wave folds it into the
/// wave-A/B `StrikeNPC_Inner` claim. Returns the roster side-effects the
/// wiring caller owes ([`HitEffectOutcomeC`]).
pub fn hit_effect_c(
    npc: &mut Npc,
    npc_type: i32,
    hit_direction: f32,
    dmg: f64,
    gore: &mut GorePool,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    rng: &mut UnifiedRandom,
    world: &World,
) -> HitEffectOutcomeC {
    let mut outcome = HitEffectOutcomeC::default();
    // `:82172-82218` — the 551 Betsy arm; this module owns the FLIPPED
    // (`spriteDirection == -1`) death branch only (the non-flipped branch
    // is wave B's).
    if npc_type == 551 && npc.hp <= 0 && npc.sprite_dir == -1 {
        betsy_flipped_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:82219-82243` — the 572/573 DD2 Kobold Walker arm (sites 81-85).
    if (npc_type == 572 || npc_type == 573) && npc.hp <= 0 {
        kobold_walker_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:82244-82268` — the 574/575 DD2 Kobold Flyer arm (sites 86-90).
    if (npc_type == 574 || npc_type == 575) && npc.hp <= 0 {
        kobold_flyer_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:82269-82291` — the 570/571 DD2 Drakin arm (sites 91-95).
    if (npc_type == 570 || npc_type == 571) && npc.hp <= 0 {
        drakin_death(npc, npc_type, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:82292-82313` — the 568/569 DD2 Wither Beast arm (sites 96-100).
    if (npc_type == 568 || npc_type == 569) && npc.hp <= 0 {
        wither_beast_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:82314-82346` — the 558/559/560 DD2 Wyvern arm (sites 101-107).
    if (npc_type == 558 || npc_type == 559 || npc_type == 560) && npc.hp <= 0 {
        wyvern_death(npc, npc_type, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:82347-82388` — the 566/567 DD2 Skeleton arm (sites 108-114, all
    // six coin-gates).
    if (npc_type == 566 || npc_type == 567) && npc.hp <= 0 {
        skeleton_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:82389-82425` — the 576/577 DD2 Ogre arm: the FLIPPED branch
    // (`spriteDirection == -1`, sites 115-120) vs the non-flipped branch
    // (sites 121-126).
    if (npc_type == 576 || npc_type == 577) && npc.hp <= 0 {
        if npc.sprite_dir == -1 {
            ogre_flipped_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
        } else {
            ogre_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
        }
    }
    // `:82426-82454` — the 564/565 DD2 Dark Mage arm (sites 127-137, all
    // eleven gores).
    if (npc_type == 564 || npc_type == 565) && npc.hp <= 0 {
        dark_mage_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:82455-82475` — the 578 DD2 Lightning Bug arm (sites 138-141).
    if npc_type == 578 && npc.hp <= 0 {
        lightning_bug_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:82476-82501` — the 620 Goblin Shark arm (sites 142-145).
    if npc_type == 620 && npc.hp <= 0 {
        goblin_shark_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:82502-82527` — the 619 Blood Squid arm (sites 146-149).
    if npc_type == 619 && npc.hp <= 0 {
        blood_squid_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:82528-82561` — the 618 Blood Nautilus arm (sites 150-161, all
    // twelve gores).
    if npc_type == 618 && npc.hp <= 0 {
        blood_nautilus_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:82562-82605` — the 541 Sand Elemental arm (sites 162-165).
    if npc_type == 541 && npc.hp <= 0 {
        sand_elemental_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:82606-82654` — the 546 Tumbleweed arm (sites 166-168).
    if npc_type == 546 && npc.hp <= 0 {
        tumbleweed_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:82672-82678` — the 437 Cultist Tablet burst (sites 169-174), gated
    // on the exact `dmg == 9999.0` double compare (NOT life <= 0).
    if npc_type == 437 && dmg == 9999.0 {
        cultist_tablet_burst(npc, gore, rng, world);
    }
    // `:82728-82746` — the 513-515 Tomb Crawler arm (sites 175-177).
    if (513..=515).contains(&npc_type) && npc.hp <= 0 {
        tomb_crawler_death(npc, npc_type, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:82747-82765` — the 510-512 Dune Splicer arm (sites 178-180).
    if (510..=512).contains(&npc_type) && npc.hp <= 0 {
        dune_splicer_death(npc, npc_type, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:82766-82784` — the 621-623 Blood Eel arm (sites 181-183).
    if (621..=623).contains(&npc_type) && npc.hp <= 0 {
        blood_eel_death(npc, npc_type, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:82785-82822` — the 518/416 Solar Spearman / Drakomire Rider arm
    // (sites 184-188).
    if (npc_type == 518 || npc_type == 416) && npc.hp <= 0 {
        solar_spearman_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:82823-82861` — the 419 Solar Solenian arm (sites 189-194).
    if npc_type == 419 && npc.hp <= 0 {
        solar_solenian_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:82862-82883` — the 412-414 Solar Crawltipede arm (sites 195-197).
    if (412..=414).contains(&npc_type) && npc.hp <= 0 {
        solar_crawltipede_death(npc, npc_type, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:82884-82917` — the 415 Solar Flare arm (sites 198-201).
    if npc_type == 415 && npc.hp <= 0 {
        solar_flare_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:82918-82958` — the 417 Solar Sroller arm (sites 202-209, all eight
    // gores).
    if npc_type == 417 && npc.hp <= 0 {
        solar_sroller_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:82959-82993` — the 418 Solar Corite arm (sites 210-214).
    if npc_type == 418 && npc.hp <= 0 {
        solar_corite_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:83039-83085` — the 426 Nebula Floater arm (sites 215-217).
    if npc_type == 426 && npc.hp <= 0 {
        nebula_floater_death(npc, dust, dust_ctx, gore, rng, world, hit_direction, &mut outcome);
    }
    // `:83086-83124` — the 427 Nebula Soldier arm (sites 218-221).
    if npc_type == 427 && npc.hp <= 0 {
        nebula_soldier_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:83125-83206` — the 429 Nebula Predictor arm (sites 222-226).
    if npc_type == 429 && npc.hp <= 0 {
        nebula_predictor_death(npc, dust, dust_ctx, gore, rng, world, hit_direction, &mut outcome);
    }
    // `:83207-83246` — the 425 Nebula Beast arm (sites 227-233).
    if npc_type == 425 && npc.hp <= 0 {
        nebula_beast_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:83247-83273` — the 428 Nebula Brain arm (sites 234-235).
    if npc_type == 428 && npc.hp <= 0 {
        nebula_brain_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:83274-83311` — the 424 Vortex Soldier arm (sites 236-240).
    if npc_type == 424 && npc.hp <= 0 {
        vortex_soldier_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:83312-83344` — the 423 Vortex Hornet arm (sites 241-243).
    if npc_type == 423 && npc.hp <= 0 {
        vortex_hornet_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:83345-83377` — the 421 Vortex Larva arm (sites 244-246).
    if npc_type == 421 && npc.hp <= 0 {
        vortex_larva_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:83378-83410` — the 420 Vortex Rider arm (sites 247-249).
    if npc_type == 420 && npc.hp <= 0 {
        vortex_rider_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:83481-83507` — the 409 Stardust Spider arm (sites 250-252).
    if npc_type == 409 && npc.hp <= 0 {
        stardust_spider_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:83532-83564` — the 407 Stardust Cell arm (sites 253-258).
    if npc_type == 407 && npc.hp <= 0 {
        stardust_cell_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:83634-83663` — the 411 Stardust Dragon body arm (sites 259-263).
    if npc_type == 411 && npc.hp <= 0 {
        stardust_dragon_body_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:83664-83722` — the 491 Flying Dutchman arm, COMPLETE: the HEAD block
    // (sites 264-274) + the full fan (sites 275-289) + the 855 loop (sites
    // 290-293) + the 854/853/856 loop (sites 294-302) + the full scatter
    // (sites 303-332) + the 120-dust tail + the projectile seam.
    if npc_type == 491 && npc.hp <= 0 {
        dutchman_death_head(npc, gore, rng, world);
        dutchman_death_fan(npc, gore, rng, world);
        dutchman_death_855(npc, gore, rng, world);
        dutchman_death_854(npc, gore, rng, world);
        dutchman_death_scatter(npc, gore, rng, world);
        dutchman_death_tail(npc, dust, dust_ctx, rng, world, &mut outcome);
    }
    // `:83723-83739` — the 492 Flying Dutchman cannon arm (sites 333-342).
    if npc_type == 492 && npc.hp <= 0 {
        dutchman_cannon_death(npc, dust, dust_ctx, gore, rng, world);
    }
    // `:83740-83796` — the 524-527 Desert Ghoul arm (sites 343-348).
    if (524..=527).contains(&npc_type) && npc.hp <= 0 {
        desert_ghoul_death(npc, npc_type, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:83797-83823` — the 528-529 Desert Lamia arm (sites 349-352).
    if (npc_type == 528 || npc_type == 529) && npc.hp <= 0 {
        desert_lamia_death(npc, npc_type, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:83824-83845` — the 530/531 Desert Scorpion arm (sites 353-357).
    if (npc_type == 530 || npc_type == 531) && npc.hp <= 0 {
        desert_scorpion_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:83846-83866` — the 532 Desert Beast arm (sites 358-361).
    if npc_type == 532 && npc.hp <= 0 {
        desert_beast_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:83889-83910` — the 460 Butcher arm (sites 362-366).
    if npc_type == 460 && npc.hp <= 0 {
        butcher_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:83911-83932` — the 477 Mothron arm (sites 367-371).
    if npc_type == 477 && npc.hp <= 0 {
        mothron_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:83933-83952` — the 479 Mothron Spawn arm (sites 372-374).
    if npc_type == 479 && npc.hp <= 0 {
        mothron_spawn_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:83953-83974` — the 478 Mothron Egg arm (sites 375-378).
    if npc_type == 478 && npc.hp <= 0 {
        mothron_egg_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:83975-83996` — the 466 Psycho arm (sites 379-383).
    if npc_type == 466 && npc.hp <= 0 {
        psycho_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:83997-84018` — the 468 DrManFly arm (sites 384-388).
    if npc_type == 468 && npc.hp <= 0 {
        dr_man_fly_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:84019-84040` — the 469 ThePossessed arm (sites 389-393).
    if npc_type == 469 && npc.hp <= 0 {
        the_possessed_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:84041-84062` — the 463 Nailhead arm (sites 394-398).
    if npc_type == 463 && npc.hp <= 0 {
        nailhead_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:84063-84084` — the 461 CreatureFromTheDeep arm (sites 399-403).
    if npc_type == 461 && npc.hp <= 0 {
        creature_from_the_deep_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:84085-84106` — the 462 Fritz arm (sites 404-408).
    if npc_type == 462 && npc.hp <= 0 {
        fritz_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:84107-84126` — the 379/438 Cultist Archer / Devote arm (sites
    // 409-411).
    if (npc_type == 379 || npc_type == 438) && npc.hp <= 0 {
        cultist_archer_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:84127-84161` — the 439 Cultist Boss arm (sites 412-418+, the bone
    // loop's count is variable).
    if npc_type == 439 && npc.hp <= 0 {
        cultist_boss_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:84162-84179` — the 440 Cultist Boss Clone arm (sites 419-421+).
    if npc_type == 440 && npc.hp <= 0 {
        cultist_clone_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:84180-84206` — the 454-459 Cultist Dragon arm (sites 422-424+).
    if (454..=459).contains(&npc_type) && npc.hp <= 0 {
        cultist_dragon_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:84207-84228` — the 372/373 Sharkron arm (sites 425-428).
    if (npc_type == 372 || npc_type == 373) && npc.hp <= 0 {
        sharkron_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:84258-84280` — the 370 Duke Fishron arm (sites 429-434).
    if npc_type == 370 && npc.hp <= 0 {
        duke_fishron_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:84281-84301` — the 381 BrainScrambler arm (sites 435-438).
    if npc_type == 381 && npc.hp <= 0 {
        brain_scrambler_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:84302-84321` — the 382/390 RayGunner / ScutlixRider arm (sites
    // 439-441).
    if (npc_type == 382 || npc_type == 390) && npc.hp <= 0 {
        ray_gunner_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:84322-84341` — the 383 MartianOfficer arm (sites 442-444).
    if npc_type == 383 && npc.hp <= 0 {
        martian_officer_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:84362-84381` — the 385 GrayGrunt arm (sites 445-447).
    if npc_type == 385 && npc.hp <= 0 {
        gray_grunt_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:84382-84401` — the 386 MartianEngineer arm (sites 448-450).
    if npc_type == 386 && npc.hp <= 0 {
        martian_engineer_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:84402-84424` — the 387 MartianTurret arm (sites 451-453).
    if npc_type == 387 && npc.hp <= 0 {
        martian_turret_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:84425-84449` — the 520 MartianWalker arm (sites 454-458).
    if npc_type == 520 && npc.hp <= 0 {
        martian_walker_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:84450-84469` — the 389 GigaZapper arm (sites 459-461).
    if npc_type == 389 && npc.hp <= 0 {
        giga_zapper_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:84470-84501` — the 388 MartianDrone arm (sites 462-463).
    if npc_type == 388 && npc.hp <= 0 {
        martian_drone_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:84502-84533` — the 399 MartianProbe arm (sites 464-465).
    if npc_type == 399 && npc.hp <= 0 {
        martian_probe_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:84534-84553` — the 391 Scutlix arm (sites 466-468).
    if npc_type == 391 && npc.hp <= 0 {
        scutlix_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:84554-84591` — the 392/393/394/395 Martian Saucer family arm
    // (sites 469-476; the 392 saucer spawns no gore).
    if (npc_type == 392 || npc_type == 393 || npc_type == 394 || npc_type == 395) && npc.hp <= 0 {
        martian_saucer_death(npc, npc_type, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:84592-84615` — the 482/483 GraniteGolem/Flyer arm (sites 477-480;
    // the 483 flyer spawns no gore).
    if (npc_type == 482 || npc_type == 483) && npc.hp <= 0 {
        granite_golem_death(npc, npc_type, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:84616-84636` — the 631 RockGolem arm (sites 481-484).
    if npc_type == 631 && npc.hp <= 0 {
        rock_golem_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:84723-84742` — the party Blue Slime confetti (sites 485-489+); the
    // gate is type==1 && ai[1]==1345, and it fires on EVERY hit (the count
    // triples on death), so it is NOT life-gated here.
    if npc_type == 1 && npc.ai[1] == 1345.0 {
        party_slime_confetti(npc, dust, dust_ctx, gore, rng, world);
    }
    // `:84795-84816` — the 305-314 Scarecrow arm (sites 490-494).
    if (305..=314).contains(&npc_type) && npc.hp <= 0 {
        scarecrow_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:84817-84837` — the 327 Pumpking arm (sites 495-503).
    if npc_type == 327 && npc.hp <= 0 {
        pumpking_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:84859-84866` — the 345 IceQueen arm (sites 504-508).
    if npc_type == 345 && npc.hp <= 0 {
        ice_queen_death(npc, gore, rng, world);
    }
    // `:84838-84858` — the 325 MourningWood arm (sites 509-515).
    if npc_type == 325 && npc.hp <= 0 {
        mourning_wood_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:84867-84891` — the 344 Everscream arm (sites 516-524, both loops).
    if npc_type == 344 && npc.hp <= 0 {
        everscream_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:84926-84943` — the 302 SlimeMasked arm (site 525).
    if npc_type == 302 && npc.hp <= 0 {
        slime_masked_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:84944-84999` — the 245-249 Golem family arm. The 245 body (sites
    // 526-538), the 249 free head (sites 539-545) and the 247/248 fists
    // (sites 546-548) spawn gores; the 246 head spawns a 249 (the roster
    // seam, no gore — the wiring wave's).
    if npc.hp <= 0 {
        if npc_type == 245 {
            golem_body_random_gores(npc, dust, dust_ctx, gore, rng, world, hit_direction);
        } else if npc_type == 249 {
            golem_head_free_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
        } else if npc_type == 247 || npc_type == 248 {
            golem_fist_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
        }
    }
    // `:85045-85062` — the 243 IceGolem arm (site 549).
    if npc_type == 243 && npc.hp <= 0 {
        ice_golem_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:85027-85044` — the 225 UmbrellaSlime arm (site 550).
    if npc_type == 225 && npc.hp <= 0 {
        umbrella_slime_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:85116-85139` — the 604 LadyBug arm (sites 551-552).
    if npc_type == 604 && npc.hp <= 0 {
        ladybug_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:85140-85163` — the 669 Stinkbug arm (sites 553-554).
    if npc_type == 669 && npc.hp <= 0 {
        stinkbug_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:85253-85314` — the 359/360/655 snail arms (sites 555-559); the gores
    // fire on EVERY hit (the dust is death-only), so NOT life-gated here.
    if npc_type == 359 {
        snail_hit(npc, dust, dust_ctx, gore, rng, world, hit_direction, 201, &[547, 548]);
    }
    if npc_type == 360 {
        snail_hit(npc, dust, dust_ctx, gore, rng, world, hit_direction, 202, &[549, 550]);
    }
    if npc_type == 655 {
        snail_hit(npc, dust, dust_ctx, gore, rng, world, hit_direction, 174, &[1256]);
    }
    // `:85315-85336` — the 154 IceTortoise arm (site 560).
    if npc_type == 154 && npc.hp <= 0 {
        ice_tortoise_death(npc, dust, dust_ctx, gore, rng, world);
    }
    // `:85427-85477` — the 81/121 CorruptSlime/Slimer arm; only the 121
    // Slimer spawns a gore (site 561) + the NPC-81 seam; the 81's baby
    // spawn is the wiring wave's (no gore).
    if npc_type == 121 && npc.hp <= 0 {
        slimer_death(npc, dust, dust_ctx, gore, rng, world, hit_direction, &mut outcome);
    }
    // `:85670-85703` — the 50 KingSlime arm (site 562 + the babies seam).
    if npc_type == 50 && npc.hp <= 0 {
        king_slime_death(npc, dust, dust_ctx, gore, rng, world, hit_direction, &mut outcome);
    }
    // `:85704-85722` — the 153 GiantTortoise arm (sites 563-566).
    if npc_type == 153 && npc.hp <= 0 {
        giant_tortoise_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:85723-85741` — the 177 Derpling arm (sites 567-570).
    if npc_type == 177 && npc.hp <= 0 {
        derpling_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:85742-85759` — the 494/495 Crawdad arm (sites 571-572).
    if (npc_type == 494 || npc_type == 495) && npc.hp <= 0 {
        crawdad_death(npc, npc_type, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:85760-85777` — the 496/497 GiantShelly arm (site 573).
    if (npc_type == 496 || npc_type == 497) && npc.hp <= 0 {
        giant_shelly_death(npc, npc_type, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:85778-85796` — the 498-506 Salamander arm (sites 574-576).
    if (498..=506).contains(&npc_type) && npc.hp <= 0 {
        salamander_death(npc, npc_type, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:85797-85841` — the bat family arm (sites 577-586).
    if (npc_type == 49 || npc_type == 51 || npc_type == 93 || npc_type == 150 || npc_type == 152
        || npc_type == 226)
        && npc.hp <= 0
    {
        bat_family_death(npc, npc_type, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:85842-85859` — the 611/689 Owl/OwlMimic arm (sites 587-589).
    if (npc_type == 611 || npc_type == 689) && npc.hp <= 0 {
        owl_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:85860-85878` — the 624 Gnome arm (sites 590-593).
    if npc_type == 624 && npc.hp <= 0 {
        gnome_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:85879-85897` — the 628 Dandelion arm (sites 594-597, all four).
    if npc_type == 628 && npc.hp <= 0 {
        dandelion_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:85898-85920` — the 646-652 GemBunny arm (sites 598-599).
    if (646..=652).contains(&npc_type) && npc.hp <= 0 {
        gem_bunny_death(npc, npc_type, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:85921-85941` — the 639-645 GemSquirrel arm (site 600).
    if (639..=645).contains(&npc_type) && npc.hp <= 0 {
        gem_squirrel_death(npc, npc_type, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:85942-86091` — the critter family arm, COMPLETE (sites 601-654):
    // the bunny group / crab / goldfish, dolphin / bird / birds / macaws /
    // toucan / cockatiels, squirrels / mouse / rat / seahorse / anglerfish,
    // flying fish / eyeball flying fish / turtles.
    if (npc_type == 46 || npc_type == 303 || npc_type == 337 || npc_type == 540 || npc_type == 67
        || npc_type == 55 || npc_type == 230 || npc_type == 615 || npc_type == 74
        || npc_type == 297 || npc_type == 298 || npc_type == 671 || npc_type == 672
        || npc_type == 673 || npc_type == 674 || npc_type == 675 || npc_type == 299
        || npc_type == 538 || npc_type == 300 || npc_type == 610 || npc_type == 626
        || npc_type == 102 || npc_type == 224 || npc_type == 587 || npc_type == 616
        || npc_type == 617 || npc_type == 625)
        && npc.hp <= 0
    {
        critter_family_death(npc, npc_type, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:86092-86127` — the penguin family arm (sites 655-663).
    if (npc_type == 148 || npc_type == 149 || npc_type == 168 || npc_type == 470) && npc.hp <= 0 {
        penguin_family_death(npc, npc_type, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:86128-86144` — the 361 Frog arm (sites 664-665).
    if npc_type == 361 && npc.hp <= 0 {
        frog_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:86173-86189` — the 607 Pupfish arm (sites 666-667).
    if npc_type == 607 && npc.hp <= 0 {
        pupfish_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:86190-86216` — the 688 Pufferfish arm (sites 668-673).
    if npc_type == 688 && npc.hp <= 0 {
        pufferfish_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:86271-86290` — the 602/603 Seagull arm (sites 674-675).
    if (npc_type == 602 || npc_type == 603) && npc.hp <= 0 {
        seagull_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:86217-86243` — the 366/367 Scorpion arm (sites 676-681).
    if (npc_type == 366 || npc_type == 367) && npc.hp <= 0 {
        scorpion_death(npc, npc_type, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:86291-86311` — the 608/609 Grebe arm (sites 682-684).
    if (npc_type == 608 || npc_type == 609) && npc.hp <= 0 {
        grebe_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:86244-86270` — the 362-365 Duck family arm (sites 685-690).
    if (362..=365).contains(&npc_type) && npc.hp <= 0 {
        duck_family_death(npc, npc_type, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:86312-86331` — the 348/349 Nutcracker arm (sites 691-695, all five).
    if (npc_type == 348 || npc_type == 349) && npc.hp <= 0 {
        nutcracker_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:86332-86351` — the 351 Krampus arm (sites 696-700).
    if npc_type == 351 && npc.hp <= 0 {
        krampus_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:86352-86370` — the 350 ElfArcher arm (sites 701-704).
    if npc_type == 350 && npc.hp <= 0 {
        elf_archer_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:86371-86408` — the corrupt/crimson critter family arm (sites
    // 705-712).
    if (npc_type == 47 || npc_type == 57 || npc_type == 58 || npc_type == 464 || npc_type == 465)
        && npc.hp <= 0
    {
        corrupt_critter_death(npc, npc_type, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:86409-86448` — the crimson monster family arm (sites 713-722).
    if (npc_type == 173 || npc_type == 174 || npc_type == 181 || npc_type == 182 || npc_type == 268)
        && npc.hp <= 0
    {
        crimson_monster_death(npc, npc_type, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:86449-86503` — the demon-eye family arm, COMPLETE (sites 723-738):
    // the 190/191/192/193/194/317/318/2 variants.
    if (npc_type == 2 || npc_type == 190 || npc_type == 191 || npc_type == 192 || npc_type == 193
        || npc_type == 194 || npc_type == 317 || npc_type == 318)
        && npc.hp <= 0
    {
        demon_eye_death(npc, npc_type, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:86504-86520` — the 157 Arapaima arm (sites 739-740).
    if npc_type == 157 && npc.hp <= 0 {
        arapaima_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:86521-86544` — the 133 WanderingEye arm (sites 741-743); the hit-half
    // eye-pop latch fires on EVERY hit (not life-gated).
    if npc_type == 133 {
        wandering_eye_hit(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:86545-86606` — the antlion family arm, COMPLETE (sites 744-765):
    // the 69/580/581/508/509/582 variants.
    if (npc_type == 69 || npc_type == 580 || npc_type == 581 || npc_type == 508
        || npc_type == 509 || npc_type == 582)
        && npc.hp <= 0
    {
        antlion_death(npc, npc_type, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:86607-86624` — the 61 Vulture arm (sites 766-768).
    if npc_type == 61 && npc.hp <= 0 {
        vulture_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:86625-86641` — the 301 Raven arm (sites 769-770).
    if npc_type == 301 && npc.hp <= 0 {
        raven_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:86642-86658` — the 252 Parrot arm (sites 771-772).
    if npc_type == 252 && npc.hp <= 0 {
        parrot_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:86659-86689` — the 489/586 BloodZombie/ZombieMerman arm (sites
    // 773-782).
    if npc_type == 489 && npc.hp <= 0 {
        blood_zombie_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    if npc_type == 586 && npc.hp <= 0 {
        zombie_merman_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:86690-86709` — the 534 DemonTaxCollector arm (sites 783-787).
    if npc_type == 534 && npc.hp <= 0 {
        demon_tax_collector_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:86710-86729` — the 490 Drippler arm (sites 788-792).
    if npc_type == 490 && npc.hp <= 0 {
        drippler_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:86730-86786` — the shark family arm, COMPLETE (sites 793-816): the
    // 65/692/542/543/544/545 variants.
    if (npc_type == 65 || npc_type == 692 || npc_type == 542 || npc_type == 543
        || npc_type == 544 || npc_type == 545)
        && npc.hp <= 0
    {
        shark_family_death(npc, npc_type, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:86787-86806` — the 195/196 LostGirl/Nymph arm (sites 817-821).
    if (npc_type == 195 || npc_type == 196) && npc.hp <= 0 {
        nymph_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:86828-86847` — the 206 IcyMerman arm (sites 822-826).
    if npc_type == 206 && npc.hp <= 0 {
        icy_merman_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:86807-86827` — the 198/199 Lihzahrd arm (sites 827-832).
    if (npc_type == 198 || npc_type == 199) && npc.hp <= 0 {
        lihzahrd_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:86848-86867` — the 342 GingerbreadMan arm (sites 833-837).
    if npc_type == 342 && npc.hp <= 0 {
        gingerbread_man_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:86868-86897` — the 338-340 ZombieElf arm (sites 838-843).
    if (338..=340).contains(&npc_type) && npc.hp <= 0 {
        zombie_elf_death(npc, npc_type, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:86898-86916` — the 343 Yeti arm (sites 844-847).
    if npc_type == 343 && npc.hp <= 0 {
        yeti_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:86917-86936` — the 691 MossZombie arm (sites 848-852).
    if npc_type == 691 && npc.hp <= 0 {
        moss_zombie_death(npc, dust, dust_ctx, gore, rng, world, hit_direction);
    }
    // `:86937-87139` — the big zombie block; this commit owns the 104/632
    // (sites 853-862); the rest lands with the next batches.
    if (npc_type == 104 || npc_type == 632) && npc.hp <= 0 {
        zombie_block_death(npc, npc_type, dust, dust_ctx, gore, rng, world, hit_direction, &mut outcome);
    }
    outcome
}

// ---- witness tests (WRITTEN, NOT RUN — code-only mode, no compiler) ----
#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;

    fn air() -> World {
        World::new("t".into(), 400, 400)
    }

    fn ctx() -> SpawnCtx {
        SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: Vec2::ZERO,
            screen_size: Vec2::new(1920.0, 1080.0),
            max_dust_to_draw: 6000,
        }
    }

    /// A DD2-boss-shaped stand-in: the unported kinds ride a placeholder
    /// kind with the C# box through `size_override` (the wave-A `boss`
    /// precedent). Betsy's box is 232x156 (`NPC.cs` SetDefaults case 551).
    fn boss(x: f32, y: f32, w: f32, h: f32) -> Npc {
        let mut n = Npc::new(NpcKind::Zombie, x, y, 100, 0.0);
        n.size_override = Some((w, h));
        n.hp = 0;
        n.scale = 1.0;
        n.sprite_dir = -1;
        n
    }

    /// `NPC.cs:82206-82215` — the flipped Betsy death spawns the 10 body
    /// gores with the flipped anchors, in source order, at velocity =
    /// base.velocity (+ the NewGore kicks) and scale = this.scale.
    #[test]
    fn w4_f10_the_betsy_flipped_death_spawns_the_ten_gores() {
        let w = air();
        let mut n = boss(100.0, 100.0, 232.0, 156.0);
        n.vx = 2.0;
        n.vy = -1.0;
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(7);
        betsy_flipped_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..10].iter().map(|g| g.gore_type).collect();
        assert_eq!(
            types,
            vec![1079, 1080, 1080, 1081, 1082, 1083, 1084, 1085, 1085, 1086]
        );
        // The flipped anchors (position + offset), the first gore at (276,154).
        assert_eq!(gore.slot(0).position, Vec2::new(100.0 + 176.0, 100.0 + 54.0));
        assert_eq!(gore.slot(3).position, Vec2::new(100.0, 100.0));
        assert_eq!(gore.slot(9).position, Vec2::new(100.0 + 142.0, 100.0 + 36.0));
        // The 20-dust type-5 death burst ran first (the pool holds the dust).
        let dust_active = dust.slots().iter().filter(|d| d.active).count();
        assert!(dust_active >= 15, "the death burst spawned: {dust_active}");
    }

    /// `NPC.cs:82206-82215` — the flipped branch draws ONLY the NewGore
    /// kicks per gore (the types are data); the twin replay mirrors the
    /// 20-dust census then the 10 gores' two kicks each on the strike stream.
    #[test]
    fn w4_f10_the_betsy_flipped_death_draws_only_the_kicks() {
        let w = air();
        let n = boss(100.0, 100.0, 232.0, 156.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(3);
        betsy_flipped_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        // The twin: 20 type-5 dusts (6 default-block draws each; type 5 hits
        // no tweak, Dust.cs:214-252) then 10 gores x 2 kicks.
        let mut twin = UnifiedRandom::new(3);
        let (bw, bh) = (232, 156);
        for _ in 0..20 {
            let _px = twin.next_max(bw - 4);
            let _py = twin.next_max(bh - 4);
            let _vx = twin.next_range(-20, 21);
            let _vy = twin.next_range(-20, 21);
            let _row = twin.next_max(3);
            let _scale = twin.next_range(-20, 21);
        }
        for _ in 0..10 {
            let _ky = twin.next_range(10, 31); // NewGore Y kick
            let _kx = twin.next_range(-20, 21); // NewGore X kick
        }
        assert_eq!(rng.peek(), twin.peek(), "the flipped death mirrors the C# draws");
    }

    /// `NPC.cs:82237-82241` — the Kobold Walker death: 5 gores
    /// 993/996/995/994/994 at Center-(10,10) with the +0/+20/+20/+34/+34
    /// Y ladder, after the 20-dust burst.
    #[test]
    fn w4_f10_the_kobold_walker_death_spawns_the_five_gores() {
        let w = air();
        let mut n = boss(100.0, 100.0, 40.0, 46.0);
        n.vx = 1.5;
        n.vy = -0.5;
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(9);
        kobold_walker_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, -1.0);
        let types: Vec<i32> = gore.slots()[..5].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![993, 996, 995, 994, 994]);
        // Center = (120,123); vector6 = (110,113).
        assert_eq!(gore.slot(0).position, Vec2::new(110.0, 113.0));
        assert_eq!(gore.slot(1).position, Vec2::new(110.0, 133.0));
        assert_eq!(gore.slot(3).position, Vec2::new(110.0, 147.0));
        assert_eq!(gore.slot(4).position, Vec2::new(110.0, 147.0));
    }

    /// `NPC.cs:82262-82266` — the Kobold Flyer death: 5 gores
    /// 1050-1054 at Center-(30,10) with the +10/+30/+30/+0/+0 Y ladder.
    #[test]
    fn w4_f10_the_kobold_flyer_death_spawns_the_five_gores() {
        let w = air();
        let mut n = boss(100.0, 100.0, 54.0, 44.0);
        n.vx = -2.0;
        n.vy = 1.0;
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(13);
        kobold_flyer_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..5].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![1050, 1051, 1052, 1053, 1054]);
        // Center = (127,122); vector7 = (97,112).
        assert_eq!(gore.slot(0).position, Vec2::new(97.0, 122.0));
        assert_eq!(gore.slot(1).position, Vec2::new(97.0, 142.0));
        assert_eq!(gore.slot(3).position, Vec2::new(97.0, 112.0));
        assert_eq!(gore.slot(4).position, Vec2::new(97.0, 112.0));
    }

    /// `NPC.cs:82219-82268` — both kobold arms draw the 20-dust census then
    /// only the NewGore kicks (5 gores x 2); the twin replay pins the stream.
    #[test]
    fn w4_f10_the_kobold_deaths_mirror_the_csharp_draws() {
        let w = air();
        let n = boss(100.0, 100.0, 40.0, 46.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(21);
        kobold_walker_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let mut twin = UnifiedRandom::new(21);
        for _ in 0..20 {
            let _px = twin.next_max(36);
            let _py = twin.next_max(42);
            let _vx = twin.next_range(-20, 21);
            let _vy = twin.next_range(-20, 21);
            let _row = twin.next_max(3);
            let _scale = twin.next_range(-20, 21);
        }
        for _ in 0..5 {
            let _ky = twin.next_range(10, 31);
            let _kx = twin.next_range(-20, 21);
        }
        assert_eq!(rng.peek(), twin.peek(), "the walker death mirrors the C# draws");
    }

    /// `NPC.cs:82284-82289` — the Drakin death's per-tier base: 570 rides
    /// 1075, 571 rides 979; the type order is base/+2/+2/+3/+1.
    #[test]
    fn w4_f10_the_drakin_death_uses_the_per_tier_base() {
        let w = air();
        for (ty, base) in [(570, 1075), (571, 979)] {
            let n = boss(100.0, 100.0, 60.0, 48.0);
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(17);
            drakin_death(&n, ty, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
            let types: Vec<i32> = gore.slots()[..5].iter().map(|g| g.gore_type).collect();
            assert_eq!(
                types,
                vec![base, base + 2, base + 2, base + 3, base + 1],
                "tier {ty}"
            );
            assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
            assert_eq!(gore.slot(4).position, Vec2::new(100.0, 134.0));
        }
    }

    /// `NPC.cs:82307-82311` — the Wither Beast death: the fixed type order
    /// 1020/1021/1023/1022/1022 (1023 precedes the 1022 pair).
    #[test]
    fn w4_f10_the_wither_beast_death_keeps_the_fixed_type_order() {
        let w = air();
        let n = boss(100.0, 100.0, 66.0, 52.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(19);
        wither_beast_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, -1.0);
        let types: Vec<i32> = gore.slots()[..5].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![1020, 1021, 1023, 1022, 1022]);
        assert_eq!(gore.slot(2).position, Vec2::new(100.0, 120.0));
        assert_eq!(gore.slot(3).position, Vec2::new(100.0, 134.0));
        assert_eq!(gore.slot(4).position, Vec2::new(100.0, 134.0));
    }

    /// `NPC.cs:82329-82344` — the Wyvern death's per-tier base and the
    /// +2/base/base/+4/+1/+3/+3 type order at the +0/+10/+24 Y ladder.
    #[test]
    fn w4_f10_the_wyvern_death_uses_the_per_tier_base() {
        let w = air();
        for (ty, base) in [(558, 974), (559, 1040), (560, 1045)] {
            let n = boss(100.0, 100.0, 60.0, 60.0);
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(23);
            wyvern_death(&n, ty, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
            let types: Vec<i32> = gore.slots()[..7].iter().map(|g| g.gore_type).collect();
            assert_eq!(
                types,
                vec![base + 2, base, base, base + 4, base + 1, base + 3, base + 3],
                "tier {ty}"
            );
            assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
            assert_eq!(gore.slot(1).position, Vec2::new(100.0, 110.0));
            assert_eq!(gore.slot(4).position, Vec2::new(100.0, 124.0));
        }
    }

    /// `NPC.cs:82362-82386` — the Skeleton death: the 970 head always
    /// spawns; the six coin-gates draw Next(2) BEFORE the conditional spawn,
    /// so the stream advances even when a gate refuses. The twin replay pins
    /// the head kicks + all six gate draws regardless of the gate outcomes.
    #[test]
    fn w4_f10_the_skeleton_death_gates_draw_before_the_spawn() {
        let w = air();
        let n = boss(100.0, 100.0, 44.0, 56.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(29);
        skeleton_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        // The head always lands (site 108).
        assert_eq!(gore.slot(0).gore_type, 970);
        assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
        // The twin: 20 type-236 dusts, the head's 2 kicks, then the six
        // Next(2) gates; a taken gate adds its 2 kicks.
        let mut twin = UnifiedRandom::new(29);
        for _ in 0..20 {
            let _px = twin.next_max(40);
            let _py = twin.next_max(52);
            let _vx = twin.next_range(-20, 21);
            let _vy = twin.next_range(-20, 21);
            let _row = twin.next_max(3);
            let _scale = twin.next_range(-20, 21);
        }
        let _ky = twin.next_range(10, 31);
        let _kx = twin.next_range(-20, 21);
        for _ in 0..6 {
            if twin.next_max(2) == 0 {
                let _gy = twin.next_range(10, 31);
                let _gx = twin.next_range(-20, 21);
            }
        }
        assert_eq!(rng.peek(), twin.peek(), "the skeleton gates mirror the C# draws");
        // Every spawned coin-gore is one of the six gate types.
        for g in gore.slots()[1..7].iter().filter(|g| g.active) {
            assert!(matches!(g.gore_type, 971 | 972 | 973), "gate type {}", g.gore_type);
        }
    }

    /// `NPC.cs:82406-82413` — the Ogre FLIPPED death: the head's Next(5)
    /// type gate draws first, the 1001/1002 gores get rotation = π, and the
    /// twin replay mirrors the census + the head gate + the six gores' kicks.
    #[test]
    fn w4_f10_the_ogre_flipped_death_rotates_the_two_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 96.0, 104.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(31);
        ogre_flipped_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        // The head is 1005 or 1006 (the Next(5) gate).
        assert!(matches!(gore.slot(0).gore_type, 1005 | 1006));
        assert_eq!(gore.slot(1).gore_type, 1000);
        assert_eq!(gore.slot(2).gore_type, 1001);
        assert_eq!(gore.slot(2).rotation, std::f32::consts::PI);
        assert_eq!(gore.slot(3).gore_type, 1002);
        assert_eq!(gore.slot(3).rotation, std::f32::consts::PI);
        assert_eq!(gore.slot(4).gore_type, 1003);
        assert_eq!(gore.slot(5).gore_type, 1004);
        // The twin: 20 type-273 dusts, the Next(5) head gate, then 6 gores'
        // kicks (the rotation writes draw nothing).
        let mut twin = UnifiedRandom::new(31);
        for _ in 0..20 {
            let _px = twin.next_max(92);
            let _py = twin.next_max(100);
            let _vx = twin.next_range(-20, 21);
            let _vy = twin.next_range(-20, 21);
            let _row = twin.next_max(3);
            let _scale = twin.next_range(-20, 21);
        }
        let _head_gate = twin.next_max(5);
        for _ in 0..6 {
            let _ky = twin.next_range(10, 31);
            let _kx = twin.next_range(-20, 21);
        }
        assert_eq!(rng.peek(), twin.peek(), "the ogre flipped death mirrors the C# draws");
    }

    /// `NPC.cs:82417-82422` — the Ogre NON-flipped death: the Next(5) head
    /// gate, the 6 non-flipped anchors, and NO rotation write on 1001/1002.
    #[test]
    fn w4_f10_the_ogre_non_flipped_death_has_no_rotation() {
        let w = air();
        let mut n = boss(100.0, 100.0, 96.0, 104.0);
        n.sprite_dir = 1;
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(37);
        ogre_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        assert!(matches!(gore.slot(0).gore_type, 1005 | 1006));
        assert_eq!(gore.slot(0).position, Vec2::new(148.0, 100.0));
        assert_eq!(gore.slot(2).gore_type, 1001);
        assert_eq!(gore.slot(2).rotation, 0.0); // no π write in this branch
        assert_eq!(gore.slot(3).gore_type, 1002);
        assert_eq!(gore.slot(3).rotation, 0.0);
        assert_eq!(gore.slot(5).position, Vec2::new(132.0, 190.0));
    }

    /// `NPC.cs:82442-82452` — the Dark Mage death: 11 gores at base 1009,
    /// the +10 (1019) gore precedes the +7/+8/+9 trio, the +4 gore shifted
    /// -20 in X, and the Y ladder +0/+15/+35/+25.
    #[test]
    fn w4_f10_the_dark_mage_death_spawns_the_eleven_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 48.0, 64.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(41);
        dark_mage_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..11].iter().map(|g| g.gore_type).collect();
        assert_eq!(
            types,
            vec![1009, 1010, 1011, 1012, 1013, 1014, 1015, 1019, 1016, 1017, 1018]
        );
        // The +4 gore (index 4) is shifted -20 in X at +35 Y.
        assert_eq!(gore.slot(4).position, Vec2::new(80.0, 135.0));
        // The +10 gore (index 7) sits at +35 Y with no X shift.
        assert_eq!(gore.slot(7).position, Vec2::new(100.0, 135.0));
        // The +7/+8/+9 trio (indices 8-10) sits at +25 Y.
        assert_eq!(gore.slot(8).position, Vec2::new(100.0, 125.0));
        assert_eq!(gore.slot(10).position, Vec2::new(100.0, 125.0));
    }

    /// `NPC.cs:82470-82473` — the Lightning Bug death: the 999 head at
    /// position, then 998/998/997 at +10 Y.
    #[test]
    fn w4_f10_the_lightning_bug_death_spawns_the_four_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 34.0, 34.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(43);
        lightning_bug_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..4].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![999, 998, 998, 997]);
        assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
        assert_eq!(gore.slot(1).position, Vec2::new(100.0, 110.0));
        assert_eq!(gore.slot(3).position, Vec2::new(100.0, 110.0));
    }

    /// `NPC.cs:82496-82499` — the Goblin Shark death: base 1183, the
    /// base/+2/+3/+1 type order at +0/+20/+20/+34 Y, after the 150-dust
    /// direct burst (velocity ×2.5, scale 1.7).
    #[test]
    fn w4_f10_the_goblin_shark_death_spawns_the_four_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 64.0, 44.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(47);
        goblin_shark_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..4].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![1183, 1185, 1186, 1184]);
        assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
        assert_eq!(gore.slot(1).position, Vec2::new(100.0, 120.0));
        assert_eq!(gore.slot(3).position, Vec2::new(100.0, 134.0));
        // The 150-dust burst ran (the pool holds the dust); the post-writes
        // set scale 1.7 on the live slots.
        let dust_active = dust.slots().iter().filter(|d| d.active).count();
        assert!(dust_active >= 100, "the burst spawned: {dust_active}");
    }

    /// `NPC.cs:82522-82525` — the Blood Squid death: base 1179, the
    /// base/+2/+3/+1 type order, and the dust post-write is fadeIn = 1.7
    /// (NOT scale) — the shape differs from the Goblin Shark only there.
    #[test]
    fn w4_f10_the_blood_squid_death_uses_fade_in_not_scale() {
        let w = air();
        let n = boss(100.0, 100.0, 48.0, 48.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(53);
        blood_squid_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, -1.0);
        let types: Vec<i32> = gore.slots()[..4].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![1179, 1181, 1182, 1180]);
        assert_eq!(gore.slot(3).position, Vec2::new(100.0, 134.0));
        // The live dust slots carry fadeIn 1.7 (the post-write); the scale
        // stays the NewDust roll x the 1.0 Scale arg (in [0.8, 1.2], never
        // the forced 1.7 the Goblin Shark arm writes).
        let live = dust.slots().iter().find(|d| d.active).unwrap();
        assert_eq!(live.fade_in, 1.7);
        assert!(live.scale <= 1.2, "scale stays the roll: {}", live.scale);
    }

    /// `NPC.cs:82548-82559` — the Blood Nautilus death: all twelve gores at
    /// base 1172, the first two type pairs alternate the Right.X-30 anchor
    /// with position, then +2/+3/+4/+4/+5/+5/+6/+6 at position; after the
    /// 200-dust burst (death scale 2.5).
    #[test]
    fn w4_f10_the_blood_nautilus_death_spawns_the_twelve_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 90.0, 90.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(59);
        blood_nautilus_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..12].iter().map(|g| g.gore_type).collect();
        assert_eq!(
            types,
            vec![1172, 1172, 1173, 1173, 1174, 1175, 1176, 1176, 1177, 1177, 1178, 1178]
        );
        // Right.X = 190; the alternation anchor is (160, 100).
        assert_eq!(gore.slot(0).position, Vec2::new(160.0, 100.0));
        assert_eq!(gore.slot(1).position, Vec2::new(100.0, 100.0));
        assert_eq!(gore.slot(2).position, Vec2::new(160.0, 100.0));
        assert_eq!(gore.slot(3).position, Vec2::new(100.0, 100.0));
        // The tail (indices 4-11) all sit at position.
        for i in 4..12 {
            assert_eq!(gore.slot(i).position, Vec2::new(100.0, 100.0));
        }
        // The 200-dust burst ran; the death post-write is scale 2.5.
        let dust_active = dust.slots().iter().filter(|d| d.active).count();
        assert!(dust_active >= 150, "the burst spawned: {dust_active}");
        let live = dust.slots().iter().find(|d| d.active).unwrap();
        assert_eq!(live.scale, 2.5);
    }

    /// `NPC.cs:82600-82603` — the Sand Elemental death: the 4 gores
    /// 960/961/963/962 (963 precedes 962) at +0/+20/+20/+40 Y, after the
    /// 30x2 tinted type-268 burst (tint (155,75,33,178), noGravity, scale
    /// 1.5, fadeIn 0.7).
    #[test]
    fn w4_f10_the_sand_elemental_death_tints_the_burst() {
        let w = air();
        let n = boss(100.0, 100.0, 60.0, 60.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(61);
        sand_elemental_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..4].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![960, 961, 963, 962]);
        assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
        assert_eq!(gore.slot(2).position, Vec2::new(100.0, 120.0));
        assert_eq!(gore.slot(3).position, Vec2::new(100.0, 140.0));
        // The tinted burst: (222,108,48,255) * 0.7 truncates to (155,75,33,178).
        let live = dust.slots().iter().find(|d| d.active).unwrap();
        assert_eq!(live.color, Color::new(155, 75, 33, 178));
        assert!(live.no_gravity);
        assert_eq!(live.scale, 1.5);
        assert_eq!(live.fade_in, 0.7);
    }

    /// `NPC.cs:82650-82652` — the Tumbleweed death: the 3 gores
    /// 964/965/966 at +0/+20/+20 Y, after the 20x2 tinted type-268 burst
    /// (tint (105,75,54,178)) and the 20-dust type-54 burst (alpha 50,
    /// scale 1.5, noGravity).
    #[test]
    fn w4_f10_the_tumbleweed_death_spawns_the_three_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 48.0, 48.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(67);
        tumbleweed_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..3].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![964, 965, 966]);
        assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
        assert_eq!(gore.slot(1).position, Vec2::new(100.0, 120.0));
        assert_eq!(gore.slot(2).position, Vec2::new(100.0, 120.0));
        // The tinted type-268 dust: (150,108,78,255) * 0.7 = (105,75,54,178).
        let tinted = dust.slots().iter().find(|d| d.active && d.dust_type == 268).unwrap();
        assert_eq!(tinted.color, Color::new(105, 75, 54, 178));
        assert_eq!(tinted.scale, 1.5);
        // The type-54 dust carries alpha 50 and noGravity.
        let t54 = dust.slots().iter().find(|d| d.active && d.dust_type == 54).unwrap();
        assert_eq!(t54.alpha, 50);
        assert!(t54.no_gravity);
    }

    /// `NPC.cs:82676` — the Cultist Tablet burst: 6 gores 728-733 at
    /// Center-(10,10), each velocity UnitY.RotatedByRandom(2π)*5 (two
    /// NextDouble draws per gore BEFORE the kicks). The twin replay pins
    /// the 6x(2 rotation + 2 kick) draw sequence.
    #[test]
    fn w4_f10_the_cultist_tablet_burst_rotates_the_velocity() {
        let w = air();
        let n = boss(100.0, 100.0, 40.0, 60.0);
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(71);
        cultist_tablet_burst(&n, &mut gore, &mut rng, &w);
        let types: Vec<i32> = gore.slots()[..6].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![728, 729, 730, 731, 732, 733]);
        // Center = (120,130); the anchor is (110,120).
        for i in 0..6 {
            assert_eq!(gore.slot(i).position, Vec2::new(110.0, 120.0));
        }
        // The twin: per gore two NextDouble (the rotation) then the two kicks.
        let mut twin = UnifiedRandom::new(71);
        for _ in 0..6 {
            let _a = twin.next_double();
            let _b = twin.next_double();
            let _ky = twin.next_range(10, 31);
            let _kx = twin.next_range(-20, 21);
        }
        assert_eq!(rng.peek(), twin.peek(), "the tablet burst mirrors the C# draws");
    }

    /// `NPC.cs:82743-82744` — the Tomb Crawler death: one gore per segment
    /// (513/514/515 -> 822/823/824) with the alpha copy, after the 50-dust
    /// type-250 burst. The color stand-in is default (the worm kinds set no
    /// SetDefaults color).
    #[test]
    fn w4_f10_the_tomb_crawler_death_maps_the_segments() {
        let w = air();
        for (ty, gore_ty) in [(513, 822), (514, 823), (515, 824)] {
            let mut n = boss(100.0, 100.0, 22.0, 22.0);
            n.alpha = 120;
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(73);
            tomb_crawler_death(&n, ty, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
            assert_eq!(gore.slot(0).gore_type, gore_ty, "segment {ty}");
            assert_eq!(gore.slot(0).alpha, 120);
            assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
            // The 50-dust type-250 burst ran.
            let dust_active = dust.slots().iter().filter(|d| d.active).count();
            assert!(dust_active >= 40, "the burst spawned: {dust_active}");
        }
    }

    /// `NPC.cs:82762-82763` — the Dune Splicer death: one gore per segment
    /// (510/511/512 -> 819/820/821) with the alpha copy.
    #[test]
    fn w4_f10_the_dune_splicer_death_maps_the_segments() {
        let w = air();
        for (ty, gore_ty) in [(510, 819), (511, 820), (512, 821)] {
            let mut n = boss(100.0, 100.0, 34.0, 34.0);
            n.alpha = 0;
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(79);
            dune_splicer_death(&n, ty, &mut dust, &ctx(), &mut gore, &mut rng, &w, -1.0);
            assert_eq!(gore.slot(0).gore_type, gore_ty, "segment {ty}");
            assert_eq!(gore.slot(0).alpha, 0);
        }
    }

    /// `NPC.cs:82781-82782` — the Blood Eel death: one gore per segment
    /// (621/622/623 -> 1187/1188/1189) with the alpha copy (the family sets
    /// alpha 255), after the 50-dust type-5 burst.
    #[test]
    fn w4_f10_the_blood_eel_death_maps_the_segments() {
        let w = air();
        for (ty, gore_ty) in [(621, 1187), (622, 1188), (623, 1189)] {
            let mut n = boss(100.0, 100.0, 28.0, 28.0);
            n.alpha = 255;
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(83);
            blood_eel_death(&n, ty, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
            assert_eq!(gore.slot(0).gore_type, gore_ty, "segment {ty}");
            assert_eq!(gore.slot(0).alpha, 255);
            // The dust is type 5 (not 250).
            let d5 = dust.slots().iter().find(|d| d.active && d.dust_type == 5);
            assert!(d5.is_some(), "the type-5 burst ran");
        }
    }

    /// `NPC.cs:82816-82820` — the Solar Spearman death: the 5 gores
    /// 844/845/847/835/836 at +0/+20 Y, after the 20-iteration type-241 +
    /// Next(3)-gated type-6 burst. The twin replay pins the gate draws.
    #[test]
    fn w4_f10_the_solar_spearman_death_gates_the_type_six_dust() {
        let w = air();
        let n = boss(100.0, 100.0, 40.0, 56.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(89);
        solar_spearman_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..5].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![844, 845, 847, 835, 836]);
        assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
        assert_eq!(gore.slot(1).position, Vec2::new(100.0, 120.0));
        // The twin: 20 x (type-241 6-draw census + Next(3) gate; a taken gate
        // adds the type-6 census + its tweak draw), then the 5 gores' kicks.
        let mut twin = UnifiedRandom::new(89);
        let census = |r: &mut UnifiedRandom| {
            let _px = r.next_max(36);
            let _py = r.next_max(52);
            let _vx = r.next_range(-20, 21);
            let _vy = r.next_range(-20, 21);
            let _row = r.next_max(3);
            let _scale = r.next_range(-20, 21);
        };
        for _ in 0..20 {
            census(&mut twin); // type 241 (no tweak)
            if twin.next_max(3) == 0 {
                census(&mut twin); // type 6 default block
                let _tweak = twin.next_range(-10, 6); // type-6 tweak (Dust.cs:216)
            }
        }
        for _ in 0..5 {
            let _ky = twin.next_range(10, 31);
            let _kx = twin.next_range(-20, 21);
        }
        assert_eq!(rng.peek(), twin.peek(), "the spearman death mirrors the C# draws");
    }

    /// `NPC.cs:82854-82859` — the Solar Solenian death: the 6 gores
    /// 844/845/846/846/847/847 at +0/+20 Y (the 846/847 pairs).
    #[test]
    fn w4_f10_the_solar_solenian_death_spawns_the_six_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 44.0, 44.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(97);
        solar_solenian_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..6].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![844, 845, 846, 846, 847, 847]);
        assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
        for i in 1..6 {
            assert_eq!(gore.slot(i).position, Vec2::new(100.0, 120.0));
        }
    }

    /// `NPC.cs:82881` — the Solar Crawltipede death: one gore per segment
    /// (412/413/414 -> 828/829/830), no alpha copy, after the 6-dust
    /// type-241 + 2-dust type-6 (alpha 100, transparent, scale 2) bursts.
    #[test]
    fn w4_f10_the_solar_crawltipede_death_maps_the_segments() {
        let w = air();
        for (ty, gore_ty) in [(412, 828), (413, 829), (414, 830)] {
            let n = boss(100.0, 100.0, 30.0, 30.0);
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(101);
            solar_crawltipede_death(&n, ty, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
            assert_eq!(gore.slot(0).gore_type, gore_ty, "segment {ty}");
            assert_eq!(gore.slot(0).alpha, 0); // no alpha copy
            // The type-6 dust carries alpha 100 + transparent + the 2x scale.
            let d6 = dust.slots().iter().find(|d| d.active && d.dust_type == 6).unwrap();
            assert_eq!(d6.alpha, 100);
            assert_eq!(d6.color, Color::TRANSPARENT);
        }
    }

    /// `NPC.cs:82912-82915` — the Solar Flare death: the 4 gores
    /// 831/832/833/834 at Center, velocity x0.8 (831/832) and x0.9
    /// (833/834), after the 40x(type-241 + ungated type-6) burst.
    #[test]
    fn w4_f10_the_solar_flare_death_scales_the_velocity() {
        let w = air();
        let mut n = boss(100.0, 100.0, 40.0, 40.0);
        n.vx = 4.0;
        n.vy = 2.0;
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(103);
        solar_flare_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..4].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![831, 832, 833, 834]);
        // Center = (120,120).
        for i in 0..4 {
            assert_eq!(gore.slot(i).position, Vec2::new(120.0, 120.0));
        }
        // The velocity is x0.8 / x0.9 plus the NewGore kicks (the Y kick
        // lifts, so check the X term: 4*0.8=3.2 +/- the 0.1-step kick).
        assert!((gore.slot(0).velocity.x - 3.2).abs() <= 2.0);
        assert!((gore.slot(2).velocity.x - 3.6).abs() <= 2.0);
    }

    /// `NPC.cs:82949-82956` — the Solar Sroller death: all 8 gores
    /// 837/838x3/839x2/840x2 at +0/+20/+38 Y, after the 30x(type-241 +
    /// Next(2)-gated type-6) burst.
    #[test]
    fn w4_f10_the_solar_sroller_death_spawns_the_eight_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 46.0, 46.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(107);
        solar_sroller_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..8].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![837, 838, 838, 838, 839, 839, 840, 840]);
        assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
        for i in 1..6 {
            assert_eq!(gore.slot(i).position, Vec2::new(100.0, 120.0));
        }
        // The 840/840 tail sits at +38 Y.
        assert_eq!(gore.slot(6).position, Vec2::new(100.0, 138.0));
        assert_eq!(gore.slot(7).position, Vec2::new(100.0, 138.0));
    }

    /// `NPC.cs:82987-82991` — the Solar Corite death: the 5 gores
    /// 841/842/842/843/843 at Center, velocity x0.8 (841/842/842) and x0.9
    /// (843/843), after the 20x(type-241 + ungated type-6) burst.
    #[test]
    fn w4_f10_the_solar_corite_death_spawns_the_five_gores() {
        let w = air();
        let mut n = boss(100.0, 100.0, 36.0, 36.0);
        n.vx = 3.0;
        n.vy = -2.0;
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(109);
        solar_corite_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..5].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![841, 842, 842, 843, 843]);
        // Center = (118,118).
        for i in 0..5 {
            assert_eq!(gore.slot(i).position, Vec2::new(118.0, 118.0));
        }
        // The velocity x0.8 / x0.9 plus the kicks.
        assert!((gore.slot(0).velocity.x - 2.4).abs() <= 2.0);
        assert!((gore.slot(3).velocity.x - 2.7).abs() <= 2.0);
    }

    /// `NPC.cs:83072-83074` — the Nebula Floater death: the 3 gores
    /// 799/800/801 at Top/Center/Center, velocity x0.7/x0.8/x0.9, after the
    /// 20-dust type-241 + 30-dust SelectRandom(229,240,161) bursts; the
    /// brain-spawn seam is returned.
    #[test]
    fn w4_f10_the_nebula_floater_death_returns_the_brain_seam() {
        let w = air();
        let mut n = boss(100.0, 100.0, 44.0, 44.0);
        n.vx = 2.0;
        n.vy = 1.0;
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(113);
        let mut out = HitEffectOutcomeC::default();
        nebula_floater_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0, &mut out);
        let types: Vec<i32> = gore.slots()[..3].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![799, 800, 801]);
        // Top = (122,100); Center = (122,122).
        assert_eq!(gore.slot(0).position, Vec2::new(122.0, 100.0));
        assert_eq!(gore.slot(1).position, Vec2::new(122.0, 122.0));
        assert_eq!(gore.slot(2).position, Vec2::new(122.0, 122.0));
        // The brain-spawn seam carries the Center.
        assert_eq!(out.nebula_brain_spawn, Some((122.0, 122.0)));
        // The twin: 20 type-241 dusts, then 30 x (SelectRandom Next(3) +
        // 6-draw census + NextFloat scale), then the 3 gores' kicks.
        let mut twin = UnifiedRandom::new(113);
        let census = |r: &mut UnifiedRandom| {
            let _px = r.next_max(40);
            let _py = r.next_max(40);
            let _vx = r.next_range(-20, 21);
            let _vy = r.next_range(-20, 21);
            let _row = r.next_max(3);
            let _scale = r.next_range(-20, 21);
        };
        for _ in 0..20 {
            census(&mut twin);
        }
        for _ in 0..30 {
            let _sel = twin.next_max(3);
            census(&mut twin);
            let _sf = twin.next_float();
        }
        for _ in 0..3 {
            let _ky = twin.next_range(10, 31);
            let _kx = twin.next_range(-20, 21);
        }
        assert_eq!(rng.peek(), twin.peek(), "the floater death mirrors the C# draws");
    }

    /// `NPC.cs:83119-83122` — the Nebula Soldier death: the 4 gores
    /// 802/803/804/805 at Center, velocity x0.8/x0.9/x0.9/x0.8, after the
    /// 10-dust type-241 + 20-dust SelectRandom(229,229,240) bursts.
    #[test]
    fn w4_f10_the_nebula_soldier_death_spawns_the_four_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 40.0, 52.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(127);
        nebula_soldier_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..4].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![802, 803, 804, 805]);
        // Center = (120,126).
        for i in 0..4 {
            assert_eq!(gore.slot(i).position, Vec2::new(120.0, 126.0));
        }
    }

    /// `NPC.cs:83156-83160` — the Nebula Predictor death: the 5 gores
    /// 808/809/809/810/810 at +0/+20/+34 Y, after the 20x(type-241 +
    /// Next(3)-gated type-229) burst; the projectile-spawn seam is returned.
    #[test]
    fn w4_f10_the_nebula_predictor_death_returns_the_shot_seam() {
        let w = air();
        let n = boss(100.0, 100.0, 48.0, 60.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(131);
        let mut out = HitEffectOutcomeC::default();
        nebula_predictor_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0, &mut out);
        let types: Vec<i32> = gore.slots()[..5].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![808, 809, 809, 810, 810]);
        assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
        assert_eq!(gore.slot(1).position, Vec2::new(100.0, 120.0));
        assert_eq!(gore.slot(3).position, Vec2::new(100.0, 134.0));
        // The shot seam carries the Center = (124,130).
        assert_eq!(out.nebula_predictor_shot, Some((124.0, 130.0)));
    }

    /// `NPC.cs:83238-83244` — the Nebula Beast death: the 7 gores
    /// 796/797/798/809/809/810/810 (796 at position, 797 at Center, the
    /// 798/809 pair at +20, the 810 pair at +34).
    #[test]
    fn w4_f10_the_nebula_beast_death_spawns_the_seven_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 50.0, 50.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(137);
        nebula_beast_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..7].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![796, 797, 798, 809, 809, 810, 810]);
        assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
        // Center = (125,125).
        assert_eq!(gore.slot(1).position, Vec2::new(125.0, 125.0));
        assert_eq!(gore.slot(2).position, Vec2::new(100.0, 120.0));
        assert_eq!(gore.slot(5).position, Vec2::new(100.0, 134.0));
    }

    /// `NPC.cs:83270-83271` — the Nebula Brain death: the 2 gores 806/807
    /// at position, velocity x0.8, after the 10x(type-241 + Next(2)-picked
    /// 229/240) burst. The twin replay pins the gate draws (both branches
    /// spawn a dust).
    #[test]
    fn w4_f10_the_nebula_brain_death_picks_the_dust_type() {
        let w = air();
        let mut n = boss(100.0, 100.0, 36.0, 36.0);
        n.vx = 5.0;
        n.vy = 0.0;
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(139);
        nebula_brain_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..2].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![806, 807]);
        assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
        // The velocity x0.8 plus the kicks.
        assert!((gore.slot(0).velocity.x - 4.0).abs() <= 2.0);
        // The twin: 10 x (type-241 census + Next(2) gate + the picked dust's
        // census), then the 2 gores' kicks.
        let mut twin = UnifiedRandom::new(139);
        let census = |r: &mut UnifiedRandom| {
            let _px = r.next_max(32);
            let _py = r.next_max(32);
            let _vx = r.next_range(-20, 21);
            let _vy = r.next_range(-20, 21);
            let _row = r.next_max(3);
            let _scale = r.next_range(-20, 21);
        };
        for _ in 0..10 {
            census(&mut twin); // type 241
            let _gate = twin.next_max(2);
            census(&mut twin); // the picked 229/240 (neither hits a tweak)
        }
        for _ in 0..2 {
            let _ky = twin.next_range(10, 31);
            let _kx = twin.next_range(-20, 21);
        }
        assert_eq!(rng.peek(), twin.peek(), "the brain death mirrors the C# draws");
    }

    /// `NPC.cs:83305-83309` — the Vortex Soldier death: the 5 gores
    /// 793/794/794/795/795 at +0/+20/+34 Y, after the 20x(type-249 +
    /// Next(3)-gated type-242) burst.
    #[test]
    fn w4_f10_the_vortex_soldier_death_spawns_the_five_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 44.0, 52.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(149);
        vortex_soldier_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..5].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![793, 794, 794, 795, 795]);
        assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
        assert_eq!(gore.slot(1).position, Vec2::new(100.0, 120.0));
        assert_eq!(gore.slot(3).position, Vec2::new(100.0, 134.0));
    }

    /// `NPC.cs:83340-83342` — the Vortex Hornet death: the 3 gores
    /// 790/791/792 at Center, velocity x0.8/x0.8/x0.9, after the 20x
    /// UNGATED (type-249 + type-242) burst. The twin replay pins the
    /// ungated census (no gate draw).
    #[test]
    fn w4_f10_the_vortex_hornet_death_is_ungated() {
        let w = air();
        let mut n = boss(100.0, 100.0, 38.0, 38.0);
        n.vx = 2.0;
        n.vy = 1.0;
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(151);
        vortex_hornet_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..3].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![790, 791, 792]);
        // Center = (119,119).
        for i in 0..3 {
            assert_eq!(gore.slot(i).position, Vec2::new(119.0, 119.0));
        }
        // The twin: 20 x (type-249 census + type-242 census + its tweak
        // draw), no gate; then the 3 gores' kicks.
        let mut twin = UnifiedRandom::new(151);
        let census = |r: &mut UnifiedRandom| {
            let _px = r.next_max(34);
            let _py = r.next_max(34);
            let _vx = r.next_range(-20, 21);
            let _vy = r.next_range(-20, 21);
            let _row = r.next_max(3);
            let _scale = r.next_range(-20, 21);
        };
        for _ in 0..20 {
            census(&mut twin); // type 249 (no tweak)
            census(&mut twin); // type 242 default block
            let _tweak = twin.next_range(-10, 6); // type-242 tweak (Dust.cs:216)
        }
        for _ in 0..3 {
            let _ky = twin.next_range(10, 31);
            let _kx = twin.next_range(-20, 21);
        }
        assert_eq!(rng.peek(), twin.peek(), "the hornet death mirrors the C# draws");
    }

    /// `NPC.cs:83373-83375` / `:83406-83408` — the Vortex Larva / Rider
    /// deaths: 3 gores each at Center, velocity x0.8/x0.8/x0.9, after the
    /// 10x ungated (type-249 + type-242) burst. The types differ per arm.
    #[test]
    fn w4_f10_the_vortex_larva_and_rider_deaths_spawn_their_gores() {
        let w = air();
        for (ty, gores, f) in [
            (421, [785, 786, 787], vortex_larva_death as fn(&Npc, &mut DustPool, &SpawnCtx, &mut GorePool, &mut UnifiedRandom, &World, f32)),
            (420, [782, 783, 784], vortex_rider_death),
        ] {
            let n = boss(100.0, 100.0, 40.0, 40.0);
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(157);
            f(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
            let types: Vec<i32> = gore.slots()[..3].iter().map(|g| g.gore_type).collect();
            assert_eq!(types, gores, "kind {ty}");
            // Center = (120,120).
            for i in 0..3 {
                assert_eq!(gore.slot(i).position, Vec2::new(120.0, 120.0));
            }
        }
    }

    /// `NPC.cs:83503-83505` — the Stardust Spider death: the 3 gores
    /// 775/776/777 at Center, velocity x0.8/x0.8/x0.9, after the 20x
    /// ungated (type-17 + type-229) burst.
    #[test]
    fn w4_f10_the_stardust_spider_death_spawns_the_three_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 40.0, 30.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(163);
        stardust_spider_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..3].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![775, 776, 777]);
        // Center = (120,115).
        for i in 0..3 {
            assert_eq!(gore.slot(i).position, Vec2::new(120.0, 115.0));
        }
    }

    /// `NPC.cs:83557-83562` — the Stardust Cell death: the 6 gores
    /// 778/779 at Top (vel x0.8) and 780/781/780/781 at Center (vel x0.9),
    /// after the 20x(Next(3)!=0-gated type-17 + ungated type-229) burst. The
    /// twin replay pins the INVERSE gate (the draw fires even when skipped).
    #[test]
    fn w4_f10_the_stardust_cell_death_uses_the_inverse_gate() {
        let w = air();
        let n = boss(100.0, 100.0, 36.0, 36.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(167);
        stardust_cell_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..6].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![778, 779, 780, 781, 780, 781]);
        // Top = (118,100); Center = (118,118).
        assert_eq!(gore.slot(0).position, Vec2::new(118.0, 100.0));
        assert_eq!(gore.slot(1).position, Vec2::new(118.0, 100.0));
        assert_eq!(gore.slot(2).position, Vec2::new(118.0, 118.0));
        // The twin: 20 x (Next(3) gate; a taken gate adds the type-17
        // census; then the type-229 census), then the 6 gores' kicks.
        let mut twin = UnifiedRandom::new(167);
        let census = |r: &mut UnifiedRandom| {
            let _px = r.next_max(32);
            let _py = r.next_max(32);
            let _vx = r.next_range(-20, 21);
            let _vy = r.next_range(-20, 21);
            let _row = r.next_max(3);
            let _scale = r.next_range(-20, 21);
        };
        for _ in 0..20 {
            if twin.next_max(3) != 0 {
                census(&mut twin); // type 17 (no tweak)
            }
            census(&mut twin); // type 229 (no tweak)
        }
        for _ in 0..6 {
            let _ky = twin.next_range(10, 31);
            let _kx = twin.next_range(-20, 21);
        }
        assert_eq!(rng.peek(), twin.peek(), "the cell death mirrors the C# draws");
    }

    /// `NPC.cs:83657-83661` — the Stardust Dragon body death: the 5 gores
    /// 772/773/773/774/774 at +0/+20/+34 Y, after the 20x(type-17 +
    /// Next(2)-gated type-229) burst.
    #[test]
    fn w4_f10_the_stardust_dragon_body_death_spawns_the_five_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 40.0, 40.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(173);
        stardust_dragon_body_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..5].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![772, 773, 773, 774, 774]);
        assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
        assert_eq!(gore.slot(1).position, Vec2::new(100.0, 120.0));
        assert_eq!(gore.slot(3).position, Vec2::new(100.0, 134.0));
    }

    /// `NPC.cs:83670-83676` — the Dutchman death head block: the 858/857
    /// gores at the integer width/2 and width/3 anchors, then the 3x3 loop
    /// of 859 gores. The facing unit flips on spriteDirection == 1.
    #[test]
    fn w4_f10_the_dutchman_death_head_spawns_the_eleven_gores() {
        let w = air();
        let mut n = boss(100.0, 100.0, 350.0, 120.0);
        n.sprite_dir = -1; // dir = +1
        n.rotation = 0.0;
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(179);
        dutchman_death_head(&n, &mut gore, &mut rng, &w);
        let types: Vec<i32> = gore.slots()[..11].iter().map(|g| g.gore_type).collect();
        assert_eq!(
            types,
            vec![858, 857, 859, 859, 859, 859, 859, 859, 859, 859, 859]
        );
        // Center = (275,160); v15 = (215,100); v16 = (1,0).
        // width/2 = 175 -> (390,100); width/3 = 116 -> (331,100).
        assert_eq!(gore.slot(0).position, Vec2::new(390.0, 100.0));
        assert_eq!(gore.slot(1).position, Vec2::new(331.0, 100.0));
        // The num183=-1 first 859: width*(0.45-0.1)=350*0.35=122.5 -> (337.5,100).
        assert_eq!(gore.slot(2).position, Vec2::new(215.0 + 350.0 * 0.35, 100.0));
    }

    /// `NPC.cs:83669` — the facing unit flips when spriteDirection == 1
    /// (dir = -1), mirroring the head anchors across the anchor point.
    #[test]
    fn w4_f10_the_dutchman_death_head_flips_on_sprite_direction() {
        let w = air();
        let mut n = boss(100.0, 100.0, 350.0, 120.0);
        n.sprite_dir = 1; // dir = -1
        n.rotation = 0.0;
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(181);
        dutchman_death_head(&n, &mut gore, &mut rng, &w);
        // v16 = (-1,0); width/2 = 175 -> (215-175,100) = (40,100).
        assert_eq!(gore.slot(0).position, Vec2::new(40.0, 100.0));
        assert_eq!(gore.slot(1).position, Vec2::new(215.0 - 116.0, 100.0));
    }

    /// `NPC.cs:83680-83684` — the Dutchman fan (all three iterations): 15
    /// gores of type SelectRandom(860,892,893), each drawing RandomVector2
    /// (two NextDouble) THEN SelectRandom (Next(3)) THEN the kicks. The twin
    /// replay pins that per-gore order.
    #[test]
    fn w4_f10_the_dutchman_fan_draws_position_then_type() {
        let w = air();
        let mut n = boss(100.0, 100.0, 350.0, 120.0);
        n.sprite_dir = -1;
        n.rotation = 0.0;
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(191);
        dutchman_death_fan(&n, &mut gore, &mut rng, &w);
        // 3 iterations x 5 gores = 15.
        let active = gore.slots()[..15].iter().filter(|g| g.active).count();
        assert_eq!(active, 15);
        for g in &gore.slots()[..15] {
            assert!(matches!(g.gore_type, 860 | 892 | 893), "fan type {}", g.gore_type);
        }
        // The twin: per gore two NextDouble (RandomVector2) + Next(3)
        // (SelectRandom) + the two kicks.
        let mut twin = UnifiedRandom::new(191);
        for _ in 0..15 {
            let _rx = twin.next_double();
            let _ry = twin.next_double();
            let _sel = twin.next_max(3);
            let _ky = twin.next_range(10, 31);
            let _kx = twin.next_range(-20, 21);
        }
        assert_eq!(rng.peek(), twin.peek(), "the fan mirrors the C# argument order");
    }

    /// `NPC.cs:83688-83689` — the Dutchman 855 loop: 4 gores of type 855 at
    /// width*±0.15 with drops 160 and 220 (160 + 60*num185).
    #[test]
    fn w4_f10_the_dutchman_855_loop_spawns_the_four_gores() {
        let w = air();
        let mut n = boss(100.0, 100.0, 350.0, 120.0);
        n.sprite_dir = -1;
        n.rotation = 0.0;
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(193);
        dutchman_death_855(&n, &mut gore, &mut rng, &w);
        let types: Vec<i32> = gore.slots()[..4].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![855, 855, 855, 855]);
        // v15 = (215,100); width*0.15 = 52.5. num185=0 drop 160 -> y=-60.
        assert_eq!(gore.slot(0).position, Vec2::new(215.0 + 52.5, 100.0 - 160.0));
        assert_eq!(gore.slot(1).position, Vec2::new(215.0 - 52.5, 100.0 - 160.0));
        // num185=1 drop 220 -> y=-120.
        assert_eq!(gore.slot(2).position, Vec2::new(215.0 + 52.5, 100.0 - 220.0));
        assert_eq!(gore.slot(3).position, Vec2::new(215.0 - 52.5, 100.0 - 220.0));
    }

    /// `NPC.cs:83693-83695` — the Dutchman 854/853/856 loop: 9 gores, the
    /// per-iteration type order 854/853/856 at the width-fraction anchors
    /// with drops 20 + 40*num186 (-20/20/60).
    #[test]
    fn w4_f10_the_dutchman_854_loop_spawns_the_nine_gores() {
        let w = air();
        let mut n = boss(100.0, 100.0, 350.0, 120.0);
        n.sprite_dir = -1;
        n.rotation = 0.0;
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(197);
        dutchman_death_854(&n, &mut gore, &mut rng, &w);
        let types: Vec<i32> = gore.slots()[..9].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![854, 853, 856, 854, 853, 856, 854, 853, 856]);
        // v15 = (215,100). num186=-1: 854 at width*0.15=52.5, drop -20 -> y=120.
        assert_eq!(gore.slot(0).position, Vec2::new(215.0 + 350.0 * 0.15, 120.0));
        // 853 at width*(-0.55)=-192.5, drop -20.
        assert_eq!(gore.slot(1).position, Vec2::new(215.0 - 350.0 * 0.55, 120.0));
        // num186=0: 854 at width*0.25=87.5, drop 20 -> y=80.
        assert_eq!(gore.slot(3).position, Vec2::new(215.0 + 350.0 * 0.25, 80.0));
        // num186=1: 856 at width*0.15=52.5, drop 60 -> y=40.
        assert_eq!(gore.slot(8).position, Vec2::new(215.0 + 350.0 * 0.15, 40.0));
    }

    /// `NPC.cs:83699-83702` — the Dutchman scatter (all 30 gores): 30 gores
    /// of type Next(861,863), alpha 100, velocity (4cosθ,4sinθ) from one
    /// NextDouble, position re-rolled with two NextFloat. The twin replay
    /// pins the per-gore order: type Next(861,863) + 2 kicks + NextDouble +
    /// 2 NextFloat.
    #[test]
    fn w4_f10_the_dutchman_scatter_draws_type_kicks_then_post() {
        let w = air();
        let n = boss(100.0, 100.0, 350.0, 120.0);
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(199);
        dutchman_death_scatter(&n, &mut gore, &mut rng, &w);
        let active = gore.slots()[..30].iter().filter(|g| g.active).count();
        assert_eq!(active, 30);
        for g in &gore.slots()[..30] {
            assert!(matches!(g.gore_type, 861 | 862), "scatter type {}", g.gore_type);
            assert_eq!(g.alpha, 100);
            // The velocity magnitude is 4 (the (4,0) rotation).
            assert!((g.velocity.length() - 4.0).abs() < 0.01);
            // sticky=false for 861/862 (Gore.cs:1341-1343).
            assert!(!g.sticky);
        }
        // The twin: per gore Next(861,863) + 2 kicks + NextDouble + 2 NextFloat.
        let mut twin = UnifiedRandom::new(199);
        for _ in 0..30 {
            let _ty = twin.next_range(861, 863);
            let _ky = twin.next_range(10, 31);
            let _kx = twin.next_range(-20, 21);
            let _theta = twin.next_double();
            let _px = twin.next_float();
            let _py = twin.next_float();
        }
        assert_eq!(rng.peek(), twin.peek(), "the scatter mirrors the C# draw order");
    }

    /// `NPC.cs:83704-83712` — the Dutchman tail: the 120x2 type-6 dust
    /// burst (alpha 100, scales 2.5/1.5, velocity x5/x4, noGravity) then the
    /// projectile seam returned. The twin replay pins the 240 dust censuses
    /// (type 6 hits the tweak draw, Dust.cs:216).
    #[test]
    fn w4_f10_the_dutchman_tail_runs_the_dust_burst_and_returns_the_seam() {
        let w = air();
        let n = boss(100.0, 100.0, 350.0, 120.0);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(211);
        let mut out = HitEffectOutcomeC::default();
        dutchman_death_tail(&n, &mut dust, &ctx(), &mut rng, &w, &mut out);
        // Center = (275,160).
        assert_eq!(out.dutchman_death_shots, Some((275.0, 160.0)));
        // The twin: 240 type-6 dusts, each the 6-draw census + the tweak draw.
        let mut twin = UnifiedRandom::new(211);
        for _ in 0..240 {
            let _px = twin.next_max(346);
            let _py = twin.next_max(116);
            let _vx = twin.next_range(-20, 21);
            let _vy = twin.next_range(-20, 21);
            let _row = twin.next_max(3);
            let _scale = twin.next_range(-20, 21);
            let _tweak = twin.next_range(-10, 6); // type-6 tweak
        }
        assert_eq!(rng.peek(), twin.peek(), "the tail mirrors the C# dust draws");
    }

    /// `NPC.cs:83734-83736` — the Dutchman cannon death: 10 gores of type
    /// Next(61,64), alpha 100, velocity (cosθ,sinθ) from one NextDouble,
    /// after the 7-dust type-31 burst. The twin replay pins the per-gore
    /// order: type Next(61,64) + 2 kicks + NextDouble.
    #[test]
    fn w4_f10_the_dutchman_cannon_death_scatters_the_ten_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 30.0, 30.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(223);
        dutchman_cannon_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w);
        let active = gore.slots()[..10].iter().filter(|g| g.active).count();
        assert_eq!(active, 10);
        for g in &gore.slots()[..10] {
            assert!(matches!(g.gore_type, 61 | 62 | 63), "cannon type {}", g.gore_type);
            assert_eq!(g.alpha, 100);
            assert!((g.velocity.length() - 1.0).abs() < 0.01);
            assert!(!g.sticky); // 61/62/63 sticky=false
        }
        // The twin: 7 type-31 dusts (6-draw census, no tweak), then per gore
        // Next(61,64) + 2 kicks + NextDouble.
        let mut twin = UnifiedRandom::new(223);
        for _ in 0..7 {
            let _px = twin.next_max(26);
            let _py = twin.next_max(26);
            let _vx = twin.next_range(-20, 21);
            let _vy = twin.next_range(-20, 21);
            let _row = twin.next_max(3);
            let _scale = twin.next_range(-20, 21);
        }
        for _ in 0..10 {
            let _ty = twin.next_range(61, 64);
            let _ky = twin.next_range(10, 31);
            let _kx = twin.next_range(-20, 21);
            let _theta = twin.next_double();
        }
        assert_eq!(rng.peek(), twin.peek(), "the cannon death mirrors the C# draw order");
    }

    /// `NPC.cs:83789-83794` — the Desert Ghoul death: the per-tier gore base
    /// (524→863, 525→867, 526→871, 527→879) with the base/+1/+2/+2/+3/+3
    /// order. The accent-dust gate draws Next(3) even for 524 (num196==0).
    #[test]
    fn w4_f10_the_desert_ghoul_death_uses_the_per_tier_base() {
        let w = air();
        for (ty, base) in [(524, 863), (525, 867), (526, 871), (527, 879)] {
            let n = boss(100.0, 100.0, 40.0, 50.0);
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(227);
            desert_ghoul_death(&n, ty, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
            let types: Vec<i32> = gore.slots()[..6].iter().map(|g| g.gore_type).collect();
            assert_eq!(
                types,
                vec![base, base + 1, base + 2, base + 2, base + 3, base + 3],
                "ghoul {ty}"
            );
            assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
            assert_eq!(gore.slot(2).position, Vec2::new(100.0, 120.0));
            assert_eq!(gore.slot(4).position, Vec2::new(100.0, 134.0));
        }
    }

    /// `NPC.cs:83780` — the ghoul accent gate draws Next(3) BEFORE the
    /// num196>0 check, so the 524 arm (num196==0) still advances the stream
    /// 20 times. The twin replay pins that for both 524 and 525.
    #[test]
    fn w4_f10_the_desert_ghoul_gate_draws_even_when_accent_is_zero() {
        let w = air();
        for ty in [524, 525] {
            let n = boss(100.0, 100.0, 40.0, 50.0);
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(229);
            desert_ghoul_death(&n, ty, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
            let mut twin = UnifiedRandom::new(229);
            let census = |r: &mut UnifiedRandom| {
                let _px = r.next_max(36);
                let _py = r.next_max(46);
                let _vx = r.next_range(-20, 21);
                let _vy = r.next_range(-20, 21);
                let _row = r.next_max(3);
                let _scale = r.next_range(-20, 21);
            };
            for _ in 0..20 {
                census(&mut twin); // type 5
                if twin.next_max(3) == 0 && ty != 524 {
                    census(&mut twin); // the accent dust (no tweak for 75/170/67)
                }
            }
            for _ in 0..6 {
                let _ky = twin.next_range(10, 31);
                let _kx = twin.next_range(-20, 21);
            }
            assert_eq!(rng.peek(), twin.peek(), "ghoul {ty} mirrors the C# draws");
        }
    }

    /// `NPC.cs:83818-83821` — the Desert Lamia death: the per-tier head
    /// (528→875, 529→876) then the FIXED 877/877/878 tail.
    #[test]
    fn w4_f10_the_desert_lamia_death_uses_the_per_tier_head() {
        let w = air();
        for (ty, head) in [(528, 875), (529, 876)] {
            let n = boss(100.0, 100.0, 40.0, 60.0);
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(233);
            desert_lamia_death(&n, ty, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
            let types: Vec<i32> = gore.slots()[..4].iter().map(|g| g.gore_type).collect();
            assert_eq!(types, vec![head, 877, 877, 878], "lamia {ty}");
            assert_eq!(gore.slot(3).position, Vec2::new(100.0, 134.0));
        }
    }

    /// `NPC.cs:83839-83843` — the Desert Scorpion death: the 5 gores
    /// 883-887 at position, after the 20-dust type-250 alpha-100 burst.
    #[test]
    fn w4_f10_the_desert_scorpion_death_spawns_the_five_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 44.0, 30.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(239);
        desert_scorpion_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..5].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![883, 884, 885, 886, 887]);
        for i in 0..5 {
            assert_eq!(gore.slot(i).position, Vec2::new(100.0, 100.0));
        }
        // The dust carries alpha 100.
        let d = dust.slots().iter().find(|d| d.active).unwrap();
        assert_eq!(d.alpha, 100);
    }

    /// `NPC.cs:83861-83864` — the Desert Beast death: the 4 gores 888-891
    /// at position, after the 50-dust type-5 alpha-100 burst.
    #[test]
    fn w4_f10_the_desert_beast_death_spawns_the_four_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 60.0, 50.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(241);
        desert_beast_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, -1.0);
        let types: Vec<i32> = gore.slots()[..4].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![888, 889, 890, 891]);
        for i in 0..4 {
            assert_eq!(gore.slot(i).position, Vec2::new(100.0, 100.0));
        }
    }

    /// `NPC.cs:83904-83908` — the Butcher death: the 5 gores
    /// 643/645/644/646/646 (645 precedes 644 at +20 Y).
    #[test]
    fn w4_f10_the_butcher_death_spawns_the_five_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 46.0, 56.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(251);
        butcher_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..5].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![643, 645, 644, 646, 646]);
        assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
        assert_eq!(gore.slot(1).position, Vec2::new(100.0, 120.0));
        assert_eq!(gore.slot(3).position, Vec2::new(100.0, 134.0));
    }

    /// `NPC.cs:83926-83930` — the Mothron death: the 5 gores 687-691 all at
    /// position, after the 50-dust type-237 burst.
    #[test]
    fn w4_f10_the_mothron_death_spawns_the_five_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 60.0, 60.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(257);
        mothron_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..5].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![687, 688, 689, 690, 691]);
        for i in 0..5 {
            assert_eq!(gore.slot(i).position, Vec2::new(100.0, 100.0));
        }
    }

    /// `NPC.cs:83948-83950` — the Mothron Spawn death: the 3 gores 681-683
    /// at position, after the 50-dust type-237 burst.
    #[test]
    fn w4_f10_the_mothron_spawn_death_spawns_the_three_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 36.0, 36.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(263);
        mothron_spawn_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..3].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![681, 682, 683]);
        for i in 0..3 {
            assert_eq!(gore.slot(i).position, Vec2::new(100.0, 100.0));
        }
    }

    /// `NPC.cs:83967-83972` — the Mothron Egg death: the dust type draw
    /// (237+Next(2)) fires per dust, and the last gore's type draw
    /// (684+Next(3)) fires before its spawn. The twin replay pins both.
    #[test]
    fn w4_f10_the_mothron_egg_death_draws_the_dust_and_gore_types() {
        let w = air();
        let n = boss(100.0, 100.0, 40.0, 40.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(269);
        mothron_egg_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        // The first three gores are fixed; the fourth is 684-686.
        assert_eq!(gore.slot(0).gore_type, 684);
        assert_eq!(gore.slot(1).gore_type, 685);
        assert_eq!(gore.slot(2).gore_type, 686);
        assert!(matches!(gore.slot(3).gore_type, 684 | 685 | 686));
        // The twin: 50 x (Next(2) type draw + 6-draw census), then 3 gores'
        // kicks, then Next(3) + the fourth gore's kicks.
        let mut twin = UnifiedRandom::new(269);
        for _ in 0..50 {
            let _ty = twin.next_max(2);
            let _px = twin.next_max(36);
            let _py = twin.next_max(36);
            let _vx = twin.next_range(-20, 21);
            let _vy = twin.next_range(-20, 21);
            let _row = twin.next_max(3);
            let _scale = twin.next_range(-20, 21);
        }
        for _ in 0..3 {
            let _ky = twin.next_range(10, 31);
            let _kx = twin.next_range(-20, 21);
        }
        let _gty = twin.next_max(3);
        let _ky = twin.next_range(10, 31);
        let _kx = twin.next_range(-20, 21);
        assert_eq!(rng.peek(), twin.peek(), "the egg death mirrors the C# draws");
    }

    /// `NPC.cs:83990-83994` — the Psycho death: the 5 gores
    /// 656/657/659/658/658 (the +20 pair is the DISTINCT 657 then 659).
    #[test]
    fn w4_f10_the_psycho_death_spawns_the_five_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 44.0, 56.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(271);
        psycho_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..5].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![656, 657, 659, 658, 658]);
        assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
        assert_eq!(gore.slot(1).position, Vec2::new(100.0, 120.0));
        assert_eq!(gore.slot(3).position, Vec2::new(100.0, 134.0));
    }

    /// `NPC.cs:84012-84016` — the DrManFly death: the shared 5-gore shape
    /// with types 662/663/664 (the +20 pair is the repeated 663).
    #[test]
    fn w4_f10_the_dr_man_fly_death_uses_the_shared_shape() {
        let w = air();
        let n = boss(100.0, 100.0, 44.0, 56.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(277);
        dr_man_fly_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..5].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![662, 663, 663, 664, 664]);
    }

    /// `NPC.cs:84034-84038` / `:84056-84060` — the ThePossessed / Nailhead
    /// deaths: the shared 5-gore shape with types 665/666/667 and
    /// 653/654/655 respectively.
    #[test]
    fn w4_f10_the_possessed_and_nailhead_deaths_use_the_shared_shape() {
        let w = air();
        for (f, gores) in [
            (the_possessed_death as fn(&Npc, &mut DustPool, &SpawnCtx, &mut GorePool, &mut UnifiedRandom, &World, f32), [665, 666, 666, 667, 667]),
            (nailhead_death, [653, 654, 654, 655, 655]),
        ] {
            let n = boss(100.0, 100.0, 44.0, 56.0);
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(281);
            f(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
            let types: Vec<i32> = gore.slots()[..5].iter().map(|g| g.gore_type).collect();
            assert_eq!(types, gores);
            assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
            assert_eq!(gore.slot(1).position, Vec2::new(100.0, 120.0));
            assert_eq!(gore.slot(3).position, Vec2::new(100.0, 134.0));
        }
    }

    /// `NPC.cs:84078-84082` / `:84100-84104` — the CreatureFromTheDeep /
    /// Fritz deaths: the shared 5-gore shape with types 647/648/649 and
    /// 650/651/652 respectively.
    #[test]
    fn w4_f10_the_creature_and_fritz_deaths_use_the_shared_shape() {
        let w = air();
        for (f, gores) in [
            (creature_from_the_deep_death as fn(&Npc, &mut DustPool, &SpawnCtx, &mut GorePool, &mut UnifiedRandom, &World, f32), [647, 648, 648, 649, 649]),
            (fritz_death, [650, 651, 651, 652, 652]),
        ] {
            let n = boss(100.0, 100.0, 44.0, 56.0);
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(283);
            f(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
            let types: Vec<i32> = gore.slots()[..5].iter().map(|g| g.gore_type).collect();
            assert_eq!(types, gores);
        }
    }

    /// `NPC.cs:84122-84124` — the Cultist Archer death: the 3 gores
    /// 900/901/901 at +0/+20 Y, after the 30-dust type-5 burst.
    #[test]
    fn w4_f10_the_cultist_archer_death_spawns_the_three_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 40.0, 56.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(293);
        cultist_archer_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..3].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![900, 901, 901]);
        assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
        assert_eq!(gore.slot(1).position, Vec2::new(100.0, 120.0));
    }

    /// `NPC.cs:84150-84159` — the Cultist Boss death: the Next(1,4) bone
    /// loop (1-3 gores of Next(11,14), velocity x0.8) then the 4 fixed gores
    /// 902/903/901/901. The twin replay pins the count draw + per-bone type
    /// draw + kicks + the fixed gores.
    #[test]
    fn w4_f10_the_cultist_boss_death_runs_the_bone_loop_then_the_fixed_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 40.0, 60.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(307);
        cultist_boss_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        // The twin: 30 type-5 dusts + 10 type-16 dusts (each 6-draw census;
        // type 16 hits no tweak), then Next(1,4) count, then per bone
        // Next(11,14) + 2 kicks, then the 4 fixed gores' kicks.
        let mut twin = UnifiedRandom::new(307);
        let census = |r: &mut UnifiedRandom| {
            let _px = r.next_max(36);
            let _py = r.next_max(56);
            let _vx = r.next_range(-20, 21);
            let _vy = r.next_range(-20, 21);
            let _row = r.next_max(3);
            let _scale = r.next_range(-20, 21);
        };
        for _ in 0..30 {
            census(&mut twin);
        }
        for _ in 0..10 {
            census(&mut twin);
        }
        let count = twin.next_range(1, 4);
        for _ in 0..count {
            let _ty = twin.next_range(11, 14);
            let _ky = twin.next_range(10, 31);
            let _kx = twin.next_range(-20, 21);
        }
        for _ in 0..4 {
            let _ky = twin.next_range(10, 31);
            let _kx = twin.next_range(-20, 21);
        }
        assert_eq!(rng.peek(), twin.peek(), "the boss death mirrors the C# draws");
        // The last four gores are the fixed 902/903/901/901.
        let n_active = gore.slots().iter().filter(|g| g.active).count();
        let fixed: Vec<i32> = gore.slots()[n_active - 4..n_active].iter().map(|g| g.gore_type).collect();
        assert_eq!(fixed, vec![902, 903, 901, 901]);
        // The bone gores (if any) are 11-13 with velocity x0.8.
        for g in &gore.slots()[..n_active - 4] {
            assert!(matches!(g.gore_type, 11 | 12 | 13));
        }
    }

    /// `NPC.cs:84175` / `:84202` — the Cultist Clone / Dragon deaths: the
    /// 10-dust type-16 burst then the Next(1,4) bone loop (1-3 gores of
    /// Next(11,14), velocity x0.8), no fixed gores. The twin replay pins the
    /// count draw + per-bone type draw + kicks.
    #[test]
    fn w4_f10_the_cultist_clone_and_dragon_deaths_run_the_bone_loop() {
        let w = air();
        for f in [
            cultist_clone_death as fn(&Npc, &mut DustPool, &SpawnCtx, &mut GorePool, &mut UnifiedRandom, &World, f32),
            cultist_dragon_death,
        ] {
            let n = boss(100.0, 100.0, 40.0, 60.0);
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(311);
            f(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
            let mut twin = UnifiedRandom::new(311);
            for _ in 0..10 {
                let _px = twin.next_max(36);
                let _py = twin.next_max(56);
                let _vx = twin.next_range(-20, 21);
                let _vy = twin.next_range(-20, 21);
                let _row = twin.next_max(3);
                let _scale = twin.next_range(-20, 21);
            }
            let count = twin.next_range(1, 4);
            for _ in 0..count {
                let _ty = twin.next_range(11, 14);
                let _ky = twin.next_range(10, 31);
                let _kx = twin.next_range(-20, 21);
            }
            assert_eq!(rng.peek(), twin.peek(), "the clone/dragon death mirrors the C# draws");
            // Every gore is a bone (11-13).
            for g in gore.slots().iter().filter(|g| g.active) {
                assert!(matches!(g.gore_type, 11 | 12 | 13));
            }
        }
    }

    /// `NPC.cs:84223-84226` — the Sharkron death: the 4 gores
    /// 583/577/578/579 at Center, velocity x0.8/x0.8/x0.9/x1, after the
    /// 75-dust type-5 burst at Center-(25,25).
    #[test]
    fn w4_f10_the_sharkron_death_spawns_the_four_gores() {
        let w = air();
        let mut n = boss(100.0, 100.0, 50.0, 50.0);
        n.vx = 4.0;
        n.vy = 2.0;
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(313);
        sharkron_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..4].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![583, 577, 578, 579]);
        // Center = (125,125).
        for i in 0..4 {
            assert_eq!(gore.slot(i).position, Vec2::new(125.0, 125.0));
        }
        // The velocity scales: 4*0.8=3.2, 4*0.9=3.6, 4*1=4 (+/- kicks).
        assert!((gore.slot(0).velocity.x - 3.2).abs() <= 2.0);
        assert!((gore.slot(2).velocity.x - 3.6).abs() <= 2.0);
        assert!((gore.slot(3).velocity.x - 4.0).abs() <= 2.0);
    }

    /// `NPC.cs:84273-84278` — the Duke Fishron death: the 6 gores at the
    /// direction-spread anchors (576/573 at Center±UnitX*20*dir, 574 at
    /// Center-UnitY*30, 575 at Center, then 574/575 again).
    #[test]
    fn w4_f10_the_duke_fishron_death_spreads_the_six_gores() {
        let w = air();
        let mut n = boss(100.0, 100.0, 100.0, 100.0);
        n.facing = 1;
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(317);
        duke_fishron_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..6].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![576, 574, 575, 573, 574, 575]);
        // Center = (150,150); dir=1.
        assert_eq!(gore.slot(0).position, Vec2::new(130.0, 150.0));
        assert_eq!(gore.slot(1).position, Vec2::new(150.0, 120.0));
        assert_eq!(gore.slot(2).position, Vec2::new(150.0, 150.0));
        assert_eq!(gore.slot(3).position, Vec2::new(170.0, 150.0));
        assert_eq!(gore.slot(4).position, Vec2::new(150.0, 120.0));
        assert_eq!(gore.slot(5).position, Vec2::new(150.0, 150.0));
    }

    /// `NPC.cs:84296-84299` — the BrainScrambler death: the 4 gores
    /// 587/590/588/589 (590 at position precedes the 588 at +20).
    #[test]
    fn w4_f10_the_brain_scrambler_death_spawns_the_four_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 44.0, 50.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(331);
        brain_scrambler_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..4].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![587, 590, 588, 589]);
        assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
        assert_eq!(gore.slot(1).position, Vec2::new(100.0, 100.0));
        assert_eq!(gore.slot(2).position, Vec2::new(100.0, 120.0));
        assert_eq!(gore.slot(3).position, Vec2::new(100.0, 134.0));
    }

    /// `NPC.cs:84317-84319` / `:84337-84339` — the RayGunner / MartianOfficer
    /// deaths: the 3-gore +0/+20/+34 ladder with types 591/592/593 and
    /// 594/595/596 respectively.
    #[test]
    fn w4_f10_the_ray_gunner_and_officer_deaths_spawn_the_ladders() {
        let w = air();
        for (f, gores) in [
            (ray_gunner_death as fn(&Npc, &mut DustPool, &SpawnCtx, &mut GorePool, &mut UnifiedRandom, &World, f32), [591, 592, 593]),
            (martian_officer_death, [594, 595, 596]),
        ] {
            let n = boss(100.0, 100.0, 44.0, 50.0);
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(337);
            f(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
            let types: Vec<i32> = gore.slots()[..3].iter().map(|g| g.gore_type).collect();
            assert_eq!(types, gores);
            assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
            assert_eq!(gore.slot(1).position, Vec2::new(100.0, 120.0));
            assert_eq!(gore.slot(2).position, Vec2::new(100.0, 134.0));
        }
    }

    /// `NPC.cs:84377-84379` / `:84397-84399` — the GrayGrunt /
    /// MartianEngineer deaths: the 3-gore +0/+20/+34 ladder with types
    /// 597/598/599 and 600/601/602 respectively.
    #[test]
    fn w4_f10_the_gray_grunt_and_engineer_deaths_spawn_the_ladders() {
        let w = air();
        for (f, gores) in [
            (gray_grunt_death as fn(&Npc, &mut DustPool, &SpawnCtx, &mut GorePool, &mut UnifiedRandom, &World, f32), [597, 598, 599]),
            (martian_engineer_death, [600, 601, 602]),
        ] {
            let n = boss(100.0, 100.0, 44.0, 50.0);
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(347);
            f(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
            let types: Vec<i32> = gore.slots()[..3].iter().map(|g| g.gore_type).collect();
            assert_eq!(types, gores);
        }
    }

    /// `NPC.cs:84420-84422` — the MartianTurret death: the 3 gores
    /// 603/603/604 (603 repeats at +0 and +20), after the 30-dust type-226
    /// scale-0.5 burst.
    #[test]
    fn w4_f10_the_martian_turret_death_repeats_the_603() {
        let w = air();
        let n = boss(100.0, 100.0, 44.0, 50.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(349);
        martian_turret_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..3].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![603, 603, 604]);
        assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
        assert_eq!(gore.slot(1).position, Vec2::new(100.0, 120.0));
        assert_eq!(gore.slot(2).position, Vec2::new(100.0, 134.0));
        // The dust carries scale 0.5.
        let d = dust.slots().iter().find(|d| d.active).unwrap();
        assert_eq!(d.scale, 0.5);
    }

    /// `NPC.cs:84443-84447` — the MartianWalker death: the 5 gores
    /// 851/852 at position and 848/849/850 at +80 Y (the leg gores).
    #[test]
    fn w4_f10_the_martian_walker_death_spawns_the_five_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 60.0, 110.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(353);
        martian_walker_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..5].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![851, 852, 848, 849, 850]);
        assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
        assert_eq!(gore.slot(1).position, Vec2::new(100.0, 100.0));
        assert_eq!(gore.slot(2).position, Vec2::new(100.0, 180.0));
        assert_eq!(gore.slot(4).position, Vec2::new(100.0, 180.0));
    }

    /// `NPC.cs:84465-84467` — the GigaZapper death: the 3 gores 904/905/906
    /// at the +0/+20/+34 ladder.
    #[test]
    fn w4_f10_the_giga_zapper_death_spawns_the_ladder() {
        let w = air();
        let n = boss(100.0, 100.0, 44.0, 50.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(359);
        giga_zapper_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..3].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![904, 905, 906]);
        assert_eq!(gore.slot(2).position, Vec2::new(100.0, 134.0));
    }

    /// `NPC.cs:84488-84499` — the MartianDrone death: the 2 gores 605/606
    /// at Center-(10,10); the velocity is base.velocity normally, but
    /// rotation.ToRotationVector2()*5 (X flipped on spriteDirection==1)
    /// when ai[0]==3.
    #[test]
    fn w4_f10_the_martian_drone_death_uses_the_rotation_velocity() {
        let w = air();
        // Normal: velocity = base.velocity.
        let mut n = boss(100.0, 100.0, 40.0, 40.0);
        n.vx = 2.0;
        n.vy = 1.0;
        n.ai[0] = 0.0;
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(367);
        martian_drone_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        assert_eq!(gore.slot(0).gore_type, 605);
        assert_eq!(gore.slot(1).gore_type, 606);
        // Center-(10,10) = (110,110).
        assert_eq!(gore.slot(0).position, Vec2::new(110.0, 110.0));
        // ai[0]==3: velocity = rotation vector *5 (rotation 0 -> (5,0)).
        let mut n2 = boss(100.0, 100.0, 40.0, 40.0);
        n2.ai[0] = 3.0;
        n2.rotation = 0.0;
        n2.sprite_dir = -1;
        let mut gore2 = GorePool::new();
        let mut dust2 = DustPool::new();
        let mut rng2 = UnifiedRandom::new(367);
        martian_drone_death(&n2, &mut dust2, &ctx(), &mut gore2, &mut rng2, &w, 1.0);
        // The X term is 5 +/- the kick; the Y term is the kick only.
        assert!((gore2.slot(0).velocity.x - 5.0).abs() <= 2.0);
        // spriteDirection==1 flips X.
        let mut n3 = boss(100.0, 100.0, 40.0, 40.0);
        n3.ai[0] = 3.0;
        n3.rotation = 0.0;
        n3.sprite_dir = 1;
        let mut gore3 = GorePool::new();
        let mut dust3 = DustPool::new();
        let mut rng3 = UnifiedRandom::new(367);
        martian_drone_death(&n3, &mut dust3, &ctx(), &mut gore3, &mut rng3, &w, 1.0);
        assert!((gore3.slot(0).velocity.x + 5.0).abs() <= 2.0);
    }

    /// `NPC.cs:84530-84531` — the MartianProbe death: the 2 gores 898/899
    /// at Center-(10,10), same rotation-velocity shape as the drone.
    #[test]
    fn w4_f10_the_martian_probe_death_spawns_the_two_gores() {
        let w = air();
        let mut n = boss(100.0, 100.0, 30.0, 30.0);
        n.ai[0] = 0.0;
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(373);
        martian_probe_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        assert_eq!(gore.slot(0).gore_type, 898);
        assert_eq!(gore.slot(1).gore_type, 899);
        // Center-(10,10) = (105,105).
        assert_eq!(gore.slot(0).position, Vec2::new(105.0, 105.0));
    }

    /// `NPC.cs:84549-84551` — the Scutlix death: the 3 gores 607/608 at
    /// position+width/2 (float division) and 609 at +20 Y.
    #[test]
    fn w4_f10_the_scutlix_death_offsets_by_half_width() {
        let w = air();
        let n = boss(100.0, 100.0, 50.0, 40.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(379);
        scutlix_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..3].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![607, 608, 609]);
        // width/2 = 25 -> (125,100).
        assert_eq!(gore.slot(0).position, Vec2::new(125.0, 100.0));
        assert_eq!(gore.slot(1).position, Vec2::new(125.0, 100.0));
        assert_eq!(gore.slot(2).position, Vec2::new(100.0, 120.0));
    }

    /// `NPC.cs:84575-84588` — the Martian Saucer family death: the 395
    /// core spawns 6 gores at the spread anchors, the 393/394 spawn one
    /// each at Center, and the 392 saucer spawns none.
    #[test]
    fn w4_f10_the_martian_saucer_family_death_per_type() {
        let w = air();
        // The 395 core: 6 gores.
        let n = boss(100.0, 100.0, 100.0, 60.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(383);
        martian_saucer_death(&n, 395, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..6].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![610, 611, 612, 613, 614, 615]);
        // Center = (150,130); the 610 anchor is (130,120).
        assert_eq!(gore.slot(0).position, Vec2::new(130.0, 120.0));
        assert_eq!(gore.slot(5).position, Vec2::new(150.0, 160.0));
        // The 393 turret: one gore 616 at Center.
        let mut gore2 = GorePool::new();
        let mut dust2 = DustPool::new();
        let mut rng2 = UnifiedRandom::new(383);
        martian_saucer_death(&n, 393, &mut dust2, &ctx(), &mut gore2, &mut rng2, &w, 1.0);
        assert_eq!(gore2.slot(0).gore_type, 616);
        assert_eq!(gore2.slot(0).position, Vec2::new(150.0, 130.0));
        // The 394 cannon: one gore 617.
        let mut gore3 = GorePool::new();
        let mut dust3 = DustPool::new();
        let mut rng3 = UnifiedRandom::new(383);
        martian_saucer_death(&n, 394, &mut dust3, &ctx(), &mut gore3, &mut rng3, &w, 1.0);
        assert_eq!(gore3.slot(0).gore_type, 617);
        // The 392 saucer: no gore.
        let mut gore4 = GorePool::new();
        let mut dust4 = DustPool::new();
        let mut rng4 = UnifiedRandom::new(383);
        martian_saucer_death(&n, 392, &mut dust4, &ctx(), &mut gore4, &mut rng4, &w, 1.0);
        assert_eq!(gore4.slots().iter().filter(|g| g.active).count(), 0);
    }

    /// `NPC.cs:84609-84612` — the GraniteGolem death: the 4 gores
    /// 718/719/720/721 at +0/+10/+20/+40 Y; the 483 flyer spawns none.
    #[test]
    fn w4_f10_the_granite_golem_death_spawns_the_four_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 50.0, 60.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(389);
        granite_golem_death(&n, 482, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..4].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![718, 719, 720, 721]);
        assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
        assert_eq!(gore.slot(1).position, Vec2::new(100.0, 110.0));
        assert_eq!(gore.slot(2).position, Vec2::new(100.0, 120.0));
        assert_eq!(gore.slot(3).position, Vec2::new(100.0, 140.0));
        // The 483 flyer: no gore.
        let mut gore2 = GorePool::new();
        let mut dust2 = DustPool::new();
        let mut rng2 = UnifiedRandom::new(389);
        granite_golem_death(&n, 483, &mut dust2, &ctx(), &mut gore2, &mut rng2, &w, 1.0);
        assert_eq!(gore2.slots().iter().filter(|g| g.active).count(), 0);
    }

    /// `NPC.cs:84631-84634` — the RockGolem death: the 4 gores
    /// 1210/1211/1212/1213 at +0/+10/+20/+40 Y, after the 30-dust type-1
    /// burst.
    #[test]
    fn w4_f10_the_rock_golem_death_spawns_the_four_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 50.0, 60.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(397);
        rock_golem_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..4].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![1210, 1211, 1212, 1213]);
        assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
        assert_eq!(gore.slot(3).position, Vec2::new(100.0, 140.0));
    }

    /// `NPC.cs:84738-84740` — the party slime confetti: the gore loop draws
    /// Next(276,283) per gore + 2 kicks + the scale Next(-20,21) + the
    /// velocity.X Next(-50,51). The count is 5 living / 15 dead. The twin
    /// replay pins the full dust+gore draw sequence for the death count.
    #[test]
    fn w4_f10_the_party_slime_confetti_draws_the_full_sequence() {
        let w = air();
        let mut n = boss(100.0, 100.0, 40.0, 30.0);
        n.ai[1] = 1345.0;
        n.hp = 0; // death -> num314 = 15
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(401);
        party_slime_confetti(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w);
        // 15 gores of type 276-282.
        let active = gore.slots()[..15].iter().filter(|g| g.active).count();
        assert_eq!(active, 15);
        for g in &gore.slots()[..15] {
            assert!((276..283).contains(&g.gore_type), "confetti type {}", g.gore_type);
        }
        // The twin: 30 dusts x (Next(139,143) + 6-draw census + Next(-30,31)
        // + Next(-50,51)), then 15 gores x (Next(276,283) + 2 kicks +
        // Next(-20,21) + Next(-50,51)).
        let mut twin = UnifiedRandom::new(401);
        for _ in 0..30 {
            let _ty = twin.next_range(139, 143);
            let _px = twin.next_max(36);
            let _py = twin.next_max(26);
            let _vx = twin.next_range(-20, 21);
            let _vy = twin.next_range(-20, 21);
            let _row = twin.next_max(3);
            let _scale = twin.next_range(-20, 21);
            let _sj = twin.next_range(-30, 31);
            let _xj = twin.next_range(-50, 51);
        }
        for _ in 0..15 {
            let _ty = twin.next_range(276, 283);
            let _ky = twin.next_range(10, 31);
            let _kx = twin.next_range(-20, 21);
            let _sj = twin.next_range(-20, 21);
            let _xj = twin.next_range(-50, 51);
        }
        assert_eq!(rng.peek(), twin.peek(), "the confetti mirrors the C# draws");
    }

    /// `NPC.cs:84810-84814` — the Scarecrow death: the 5 gores
    /// 441/442/443/444/445 at +0/+20/+34 Y, after the 60-dust type-194
    /// scale-1.1 burst.
    #[test]
    fn w4_f10_the_scarecrow_death_spawns_the_five_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 44.0, 60.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(409);
        scarecrow_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..5].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![441, 442, 443, 444, 445]);
        assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
        assert_eq!(gore.slot(1).position, Vec2::new(100.0, 120.0));
        assert_eq!(gore.slot(3).position, Vec2::new(100.0, 134.0));
    }

    /// `NPC.cs:84834` — the Pumpking death: the 9-gore loop 476-484 at
    /// position with velocity x0, after the 60-dust type-189 scale-1.1
    /// burst.
    #[test]
    fn w4_f10_the_pumpking_death_spawns_the_nine_gore_loop() {
        let w = air();
        let n = boss(100.0, 100.0, 60.0, 60.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(419);
        pumpking_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..9].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![476, 477, 478, 479, 480, 481, 482, 483, 484]);
        for i in 0..9 {
            assert_eq!(gore.slot(i).position, Vec2::new(100.0, 100.0));
        }
    }

    /// `NPC.cs:84861-84865` — the IceQueen death: the 5 gores
    /// 513/514/514/515/516 at position with velocity x0, no dust burst.
    #[test]
    fn w4_f10_the_ice_queen_death_spawns_the_five_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 50.0, 60.0);
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(421);
        ice_queen_death(&n, &mut gore, &mut rng, &w);
        let types: Vec<i32> = gore.slots()[..5].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![513, 514, 514, 515, 516]);
        for i in 0..5 {
            assert_eq!(gore.slot(i).position, Vec2::new(100.0, 100.0));
        }
    }

    /// `NPC.cs:84855` — the MourningWood death: the 7-gore loop 469-475 at
    /// position with velocity x0, after the 60-dust type-191 scale-1.1
    /// burst.
    #[test]
    fn w4_f10_the_mourning_wood_death_spawns_the_seven_gore_loop() {
        let w = air();
        let n = boss(100.0, 100.0, 60.0, 80.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(431);
        mourning_wood_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..7].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![469, 470, 471, 472, 473, 474, 475]);
        for i in 0..7 {
            assert_eq!(gore.slot(i).position, Vec2::new(100.0, 100.0));
        }
    }

    /// `NPC.cs:84884-84888` — the Everscream death: the two gore loops
    /// 508-512 then 509-512 (9 gores) at position with velocity x0, after
    /// the 60-dust type-196 burst.
    #[test]
    fn w4_f10_the_everscream_death_spawns_both_loops() {
        let w = air();
        let n = boss(100.0, 100.0, 60.0, 80.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(433);
        everscream_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..9].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![508, 509, 510, 511, 512, 509, 510, 511, 512]);
        for i in 0..9 {
            assert_eq!(gore.slot(i).position, Vec2::new(100.0, 100.0));
        }
    }

    /// `NPC.cs:84941` — the SlimeMasked death: the single gore 440 at
    /// position, after the 50-dust type-192 gray alpha-100 burst.
    #[test]
    fn w4_f10_the_slime_masked_death_spawns_the_single_gore() {
        let w = air();
        let n = boss(100.0, 100.0, 40.0, 30.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(439);
        slime_masked_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        assert_eq!(gore.slot(0).gore_type, 440);
        assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
        let d = dust.slots().iter().find(|d| d.active).unwrap();
        assert_eq!(d.color, Color::new(100, 100, 100, 100));
        assert_eq!(d.alpha, 100);
    }

    /// `NPC.cs:84968-84980` — the Golem body's 13 gores: the 6 random-
    /// position 368/370 (Next(width)+Next(height) per gore before spawn)
    /// then the fixed tail 360/361/362/363/364/367/366 at position. The twin
    /// replay pins the per-gore position draws + kicks.
    #[test]
    fn w4_f10_the_golem_body_random_gores_draw_position_first() {
        let w = air();
        let n = boss(100.0, 100.0, 60.0, 80.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(443);
        golem_body_random_gores(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..13].iter().map(|g| g.gore_type).collect();
        assert_eq!(
            types,
            vec![368, 370, 368, 370, 368, 370, 360, 361, 362, 363, 364, 367, 366]
        );
        // The fixed tail sits at position.
        for i in 6..13 {
            assert_eq!(gore.slot(i).position, Vec2::new(100.0, 100.0));
        }
        // The twin: 50 type-148 dusts, then 6 x (Next(60) + Next(80) + 2
        // kicks), then the 7 fixed gores' kicks.
        let mut twin = UnifiedRandom::new(443);
        for _ in 0..50 {
            let _px = twin.next_max(56);
            let _py = twin.next_max(76);
            let _vx = twin.next_range(-20, 21);
            let _vy = twin.next_range(-20, 21);
            let _row = twin.next_max(3);
            let _scale = twin.next_range(-20, 21);
        }
        for _ in 0..6 {
            let _px = twin.next_max(60);
            let _py = twin.next_max(80);
            let _ky = twin.next_range(10, 31);
            let _kx = twin.next_range(-20, 21);
        }
        for _ in 0..7 {
            let _ky = twin.next_range(10, 31);
            let _kx = twin.next_range(-20, 21);
        }
        assert_eq!(rng.peek(), twin.peek(), "the golem body mirrors the C# draws");
    }

    /// `NPC.cs:84984-84990` — the GolemHeadFree death: the 4 random
    /// 368/370 gores then the fixed 365/363/362 at position.
    #[test]
    fn w4_f10_the_golem_head_free_death_spawns_the_seven_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 50.0, 50.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(449);
        golem_head_free_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..7].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![368, 370, 368, 370, 365, 363, 362]);
        for i in 4..7 {
            assert_eq!(gore.slot(i).position, Vec2::new(100.0, 100.0));
        }
    }

    /// `NPC.cs:84994-84996` — the Golem fist death: the 3 gores
    /// 369/371/370 at position.
    #[test]
    fn w4_f10_the_golem_fist_death_spawns_the_three_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 40.0, 40.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(457);
        golem_fist_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..3].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![369, 371, 370]);
        for i in 0..3 {
            assert_eq!(gore.slot(i).position, Vec2::new(100.0, 100.0));
        }
    }

    /// `NPC.cs:85060` — the IceGolem death: the single gore 359 at
    /// position, after the 200-dust type-161 burst.
    #[test]
    fn w4_f10_the_ice_golem_death_spawns_the_single_gore() {
        let w = air();
        let n = boss(100.0, 100.0, 50.0, 60.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(461);
        ice_golem_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        assert_eq!(gore.slot(0).gore_type, 359);
        assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
    }

    /// `NPC.cs:85042` — the UmbrellaSlime death: the single gore 314 at
    /// position, after the 50-dust type-4 blue alpha-100 burst.
    #[test]
    fn w4_f10_the_umbrella_slime_death_spawns_the_single_gore() {
        let w = air();
        let n = boss(100.0, 100.0, 40.0, 30.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(463);
        umbrella_slime_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        assert_eq!(gore.slot(0).gore_type, 314);
        let d = dust.slots().iter().find(|d| d.active).unwrap();
        assert_eq!(d.color, Color::new(0, 80, 255, 100));
    }

    /// `NPC.cs:85133-85137` / `:85157-85161` — the LadyBug / Stinkbug
    /// deaths: the critter gated-dust burst then two Next(2)-gated gores.
    /// The twin replay pins the gate draws (both branches spawn a dust; the
    /// gore gates draw even when refused).
    #[test]
    fn w4_f10_the_ladybug_and_stinkbug_deaths_gate_the_gores() {
        let w = air();
        for (f, gores) in [
            (ladybug_death as fn(&Npc, &mut DustPool, &SpawnCtx, &mut GorePool, &mut UnifiedRandom, &World, f32), [1148, 1149]),
            (stinkbug_death, [1275, 1276]),
        ] {
            let n = boss(100.0, 100.0, 20.0, 20.0);
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(467);
            f(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
            // The twin: 6 x (6-draw census + Next(2) gate), then 2 x (Next(2)
            // gate; a taken gate adds 2 kicks).
            let mut twin = UnifiedRandom::new(467);
            for _ in 0..6 {
                let _px = twin.next_max(16);
                let _py = twin.next_max(16);
                let _vx = twin.next_range(-20, 21);
                let _vy = twin.next_range(-20, 21);
                let _row = twin.next_max(3);
                let _scale = twin.next_range(-20, 21);
                let _gate = twin.next_max(2);
            }
            for _ in 0..2 {
                if twin.next_max(2) == 0 {
                    let _ky = twin.next_range(10, 31);
                    let _kx = twin.next_range(-20, 21);
                }
            }
            assert_eq!(rng.peek(), twin.peek(), "the critter death mirrors the C# draws");
            // Every spawned gore is one of the two gated types.
            for g in gore.slots().iter().filter(|g| g.active) {
                assert!(gores.contains(&g.gore_type), "critter type {}", g.gore_type);
            }
        }
    }

    /// `NPC.cs:85271-85272` / `:85292-85293` / `:85313` — the snail hit:
    /// the gores fire on EVERY hit (547/548, 549/550, 1256), the gated dust
    /// only on death. A living snail spawns gores but no dust.
    #[test]
    fn w4_f10_the_snail_hit_spawns_gores_even_when_living() {
        let w = air();
        // Living snail: gores but no dust.
        let mut n = boss(100.0, 100.0, 24.0, 20.0);
        n.hp = 10;
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(479);
        snail_hit(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0, 201, &[547, 548]);
        let types: Vec<i32> = gore.slots()[..2].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![547, 548]);
        assert_eq!(dust.slots().iter().filter(|d| d.active).count(), 0);
        // Dead snail: gores + the gated dust.
        let mut n2 = boss(100.0, 100.0, 24.0, 20.0);
        n2.hp = 0;
        let mut gore2 = GorePool::new();
        let mut dust2 = DustPool::new();
        let mut rng2 = UnifiedRandom::new(479);
        snail_hit(&n2, &mut dust2, &ctx(), &mut gore2, &mut rng2, &w, 1.0, 202, &[549, 550]);
        let types2: Vec<i32> = gore2.slots()[..2].iter().map(|g| g.gore_type).collect();
        assert_eq!(types2, vec![549, 550]);
        assert!(dust2.slots().iter().filter(|d| d.active).count() > 0);
        // The MagmaSnail: one gore 1256.
        let mut n3 = boss(100.0, 100.0, 24.0, 20.0);
        n3.hp = 0;
        let mut gore3 = GorePool::new();
        let mut dust3 = DustPool::new();
        let mut rng3 = UnifiedRandom::new(479);
        snail_hit(&n3, &mut dust3, &ctx(), &mut gore3, &mut rng3, &w, 1.0, 174, &[1256]);
        assert_eq!(gore3.slot(0).gore_type, 1256);
    }

    /// `NPC.cs:85334` — the IceTortoise death: the single gore 180 at
    /// position, after the 50-dust type-67 velocity-scaled burst.
    #[test]
    fn w4_f10_the_ice_tortoise_death_spawns_the_single_gore() {
        let w = air();
        let mut n = boss(100.0, 100.0, 50.0, 40.0);
        n.vx = 3.0;
        n.vy = 1.0;
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(487);
        ice_tortoise_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w);
        assert_eq!(gore.slot(0).gore_type, 180);
        assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
        // The dust carries noGravity and the velocity x0.2 post-write.
        let d = dust.slots().iter().find(|d| d.active).unwrap();
        assert!(d.no_gravity);
    }

    /// `NPC.cs:85451` — the Slimer death: the gore 94 with the alpha copy
    /// (55), after the 50-dust type-14 velocity-x2 burst; the NPC-81 spawn
    /// seam is returned.
    #[test]
    fn w4_f10_the_slimer_death_spawns_the_gore_and_returns_the_seam() {
        let w = air();
        let mut n = boss(100.0, 100.0, 40.0, 30.0);
        n.alpha = 55;
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(491);
        let mut out = HitEffectOutcomeC::default();
        slimer_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0, &mut out);
        assert_eq!(gore.slot(0).gore_type, 94);
        assert_eq!(gore.slot(0).alpha, 55);
        assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
        // The spawn anchor = (pos.X + width/2, pos.Y + height) = (120,130).
        assert_eq!(out.slimer_spawn, Some((120.0, 130.0)));
    }

    /// `NPC.cs:85684` — the KingSlime death: the gore 734 at
    /// Center+(-40,-height/2) (integer height/2), after the 200-dust type-4
    /// blue alpha-175 burst; the babies seam is returned.
    #[test]
    fn w4_f10_the_king_slime_death_spawns_the_gore_and_returns_the_seam() {
        let w = air();
        let n = boss(100.0, 100.0, 100.0, 80.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(499);
        let mut out = HitEffectOutcomeC::default();
        king_slime_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0, &mut out);
        assert_eq!(gore.slot(0).gore_type, 734);
        // Center = (150,140); height/2 = 40 -> anchor (110,100).
        assert_eq!(gore.slot(0).position, Vec2::new(110.0, 100.0));
        assert_eq!(out.king_slime_babies, Some((100.0, 100.0, 100.0, 80.0)));
    }

    /// `NPC.cs:85718-85721` / `:85737-85740` — the GiantTortoise / Derpling
    /// deaths: the 4-gore shape with the repeated tail (177/178/179/179 and
    /// 234/235/236/236).
    #[test]
    fn w4_f10_the_tortoise_and_derpling_deaths_spawn_the_four_gores() {
        let w = air();
        for (f, gores) in [
            (giant_tortoise_death as fn(&Npc, &mut DustPool, &SpawnCtx, &mut GorePool, &mut UnifiedRandom, &World, f32), [177, 178, 179, 179]),
            (derpling_death, [234, 235, 236, 236]),
        ] {
            let n = boss(100.0, 100.0, 50.0, 40.0);
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(503);
            f(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
            let types: Vec<i32> = gore.slots()[..4].iter().map(|g| g.gore_type).collect();
            assert_eq!(types, gores);
            for i in 0..4 {
                assert_eq!(gore.slot(i).position, Vec2::new(100.0, 100.0));
            }
        }
    }

    /// `NPC.cs:85757-85758` — the Crawdad death: the per-type offset
    /// (type-494)*2 shifts the 736/735 pair (494→736/735, 495→738/737).
    #[test]
    fn w4_f10_the_crawdad_death_uses_the_per_type_offset() {
        let w = air();
        for (ty, gores) in [(494, [736, 735]), (495, [738, 737])] {
            let n = boss(100.0, 100.0, 44.0, 30.0);
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(509);
            crawdad_death(&n, ty, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
            let types: Vec<i32> = gore.slots()[..2].iter().map(|g| g.gore_type).collect();
            assert_eq!(types, gores, "crawdad {ty}");
        }
    }

    /// `NPC.cs:85775` — the GiantShelly death: the single gore
    /// 739+(type-496) (496→739, 497→740).
    #[test]
    fn w4_f10_the_giant_shelly_death_uses_the_per_type_offset() {
        let w = air();
        for (ty, gore_ty) in [(496, 739), (497, 740)] {
            let n = boss(100.0, 100.0, 40.0, 30.0);
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(521);
            giant_shelly_death(&n, ty, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
            assert_eq!(gore.slot(0).gore_type, gore_ty, "shelly {ty}");
        }
    }

    /// `NPC.cs:85793-85795` — the Salamander death: the 3 gores shift by
    /// type-498 (750/741/759 + offset) at +0/+16/+18 Y.
    #[test]
    fn w4_f10_the_salamander_death_uses_the_per_type_offset() {
        let w = air();
        for ty in [498, 502, 506] {
            let off = ty - 498;
            let n = boss(100.0, 100.0, 44.0, 36.0);
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(523);
            salamander_death(&n, ty, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
            let types: Vec<i32> = gore.slots()[..3].iter().map(|g| g.gore_type).collect();
            assert_eq!(types, vec![750 + off, 741 + off, 759 + off], "salamander {ty}");
            assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
            assert_eq!(gore.slot(1).position, Vec2::new(100.0, 116.0));
            assert_eq!(gore.slot(2).position, Vec2::new(100.0, 118.0));
        }
    }

    /// `NPC.cs:85813-85839` — the bat family death: the per-type gore lists
    /// (51→[83], 93→[107], 150→[164,165], 152→[175,176,176],
    /// 226→[317,318,318,319], 49→[82]).
    #[test]
    fn w4_f10_the_bat_family_death_uses_the_per_type_list() {
        let w = air();
        let cases: [(i32, &[i32]); 6] = [
            (49, &[82]),
            (51, &[83]),
            (93, &[107]),
            (150, &[164, 165]),
            (152, &[175, 176, 176]),
            (226, &[317, 318, 318, 319]),
        ];
        for (ty, gores) in cases {
            let n = boss(100.0, 100.0, 36.0, 30.0);
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(541);
            bat_family_death(&n, ty, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
            let types: Vec<i32> = gore.slots()[..gores.len()].iter().map(|g| g.gore_type).collect();
            assert_eq!(types, gores, "bat {ty}");
            for i in 0..gores.len() {
                assert_eq!(gore.slot(i).position, Vec2::new(100.0, 100.0));
            }
        }
    }

    /// `NPC.cs:85856-85858` — the Owl death: the 3 gores 1164/1164/1165
    /// (the 1164 repeats).
    #[test]
    fn w4_f10_the_owl_death_spawns_the_three_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 36.0, 30.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(547);
        owl_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..3].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![1164, 1164, 1165]);
    }

    /// `NPC.cs:85874-85877` — the Gnome death: the 4 gores 1190-1193.
    #[test]
    fn w4_f10_the_gnome_death_spawns_the_four_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 30.0, 40.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(557);
        gnome_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..4].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![1190, 1191, 1192, 1193]);
    }

    /// `NPC.cs:85893-85896` — the Dandelion death: the 4 gores
    /// 1205/1206/1206/1207, after the 10-dust type-3 burst.
    #[test]
    fn w4_f10_the_dandelion_death_spawns_the_four_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 30.0, 40.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(563);
        dandelion_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..4].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![1205, 1206, 1206, 1207]);
    }

    /// `NPC.cs:85918-85919` — the GemBunny death: the per-type offset
    /// (type-646)*2 shifts the 1234/1235 pair; the Amber 652 uses dust 138.
    #[test]
    fn w4_f10_the_gem_bunny_death_uses_the_per_type_offset() {
        let w = air();
        for ty in [646, 649, 652] {
            let off = (ty - 646) * 2;
            let n = boss(100.0, 100.0, 30.0, 24.0);
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(569);
            gem_bunny_death(&n, ty, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
            let types: Vec<i32> = gore.slots()[..2].iter().map(|g| g.gore_type).collect();
            assert_eq!(types, vec![1234 + off, 1235 + off], "bunny {ty}");
        }
    }

    /// `NPC.cs:85940` — the GemSquirrel death: the single gore
    /// 1227+(type-639); the Amber 645 uses dust 138.
    #[test]
    fn w4_f10_the_gem_squirrel_death_uses_the_per_type_offset() {
        let w = air();
        for ty in [639, 642, 645] {
            let n = boss(100.0, 100.0, 30.0, 24.0);
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(571);
            gem_squirrel_death(&n, ty, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
            assert_eq!(gore.slot(0).gore_type, 1227 + (ty - 639), "squirrel {ty}");
        }
    }

    /// `NPC.cs:85958-85981` — the critter family: the bunny group (76/77,
    /// 303 adds dust, 540 adds 926), the crab (95/95/96), the goldfish
    /// (553/554).
    #[test]
    fn w4_f10_the_critter_family_bunny_crab_goldfish() {
        let w = air();
        // The plain bunny 46: 76/77.
        let n = boss(100.0, 100.0, 30.0, 24.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(577);
        critter_family_death(&n, 46, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..2].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![76, 77]);
        // The PartyBunny 540: 76/77/926.
        let mut gore2 = GorePool::new();
        let mut dust2 = DustPool::new();
        let mut rng2 = UnifiedRandom::new(577);
        critter_family_death(&n, 540, &mut dust2, &ctx(), &mut gore2, &mut rng2, &w, 1.0);
        let types2: Vec<i32> = gore2.slots()[..3].iter().map(|g| g.gore_type).collect();
        assert_eq!(types2, vec![76, 77, 926]);
        // The crab 67: 95/95/96.
        let mut gore3 = GorePool::new();
        let mut dust3 = DustPool::new();
        let mut rng3 = UnifiedRandom::new(577);
        critter_family_death(&n, 67, &mut dust3, &ctx(), &mut gore3, &mut rng3, &w, 1.0);
        let types3: Vec<i32> = gore3.slots()[..3].iter().map(|g| g.gore_type).collect();
        assert_eq!(types3, vec![95, 95, 96]);
        // The goldfish 55: 553/554.
        let mut gore4 = GorePool::new();
        let mut dust4 = DustPool::new();
        let mut rng4 = UnifiedRandom::new(577);
        critter_family_death(&n, 55, &mut dust4, &ctx(), &mut gore4, &mut rng4, &w, 1.0);
        let types4: Vec<i32> = gore4.slots()[..2].iter().map(|g| g.gore_type).collect();
        assert_eq!(types4, vec![553, 554]);
    }

    /// `NPC.cs:85985-86006` — the critter family: dolphin 615
    /// (1198/1199/1200/1200), bird 74 (100), birds 297/298 (431/432),
    /// scarlet macaw 671 (1277 at position + 1279 at Center±4).
    #[test]
    fn w4_f10_the_critter_family_dolphin_birds_macaw() {
        let w = air();
        let n = boss(100.0, 100.0, 30.0, 24.0);
        // The dolphin 615.
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(587);
        critter_family_death(&n, 615, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..4].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![1198, 1199, 1200, 1200]);
        // The bird 74.
        let mut gore2 = GorePool::new();
        let mut dust2 = DustPool::new();
        let mut rng2 = UnifiedRandom::new(587);
        critter_family_death(&n, 74, &mut dust2, &ctx(), &mut gore2, &mut rng2, &w, 1.0);
        assert_eq!(gore2.slot(0).gore_type, 100);
        // The scarlet macaw 671: Center = (115,112).
        let mut gore3 = GorePool::new();
        let mut dust3 = DustPool::new();
        let mut rng3 = UnifiedRandom::new(587);
        critter_family_death(&n, 671, &mut dust3, &ctx(), &mut gore3, &mut rng3, &w, 1.0);
        let types3: Vec<i32> = gore3.slots()[..3].iter().map(|g| g.gore_type).collect();
        assert_eq!(types3, vec![1277, 1279, 1279]);
        assert_eq!(gore3.slot(0).position, Vec2::new(100.0, 100.0));
        assert_eq!(gore3.slot(1).position, Vec2::new(119.0, 112.0));
        assert_eq!(gore3.slot(2).position, Vec2::new(111.0, 112.0));
    }

    /// `NPC.cs:86010-86024` — the blue macaw 672 / toucan 673 / yellow
    /// cockatiel 674: the head at position + the wings at Center±4, with
    /// the per-type pairs 1280/1281, 1282/1283, 1284/1285.
    #[test]
    fn w4_f10_the_critter_family_macaw_toucan_cockatiel() {
        let w = air();
        for (ty, head, wing) in [(672, 1280, 1281), (673, 1282, 1283), (674, 1284, 1285)] {
            let n = boss(100.0, 100.0, 30.0, 24.0);
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(593);
            critter_family_death(&n, ty, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
            let types: Vec<i32> = gore.slots()[..3].iter().map(|g| g.gore_type).collect();
            assert_eq!(types, vec![head, wing, wing], "critter {ty}");
            assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
            // Center = (115,112).
            assert_eq!(gore.slot(1).position, Vec2::new(119.0, 112.0));
            assert_eq!(gore.slot(2).position, Vec2::new(111.0, 112.0));
        }
    }

    /// `NPC.cs:86028-86054` — the gray cockatiel 675 (1286/1287 wings) and
    /// the single-gore critters: squirrel 299 (433), red squirrel 538 (897),
    /// mouse 300 (434), rat 610 (1163), seahorse 626 (1197), anglerfish 102
    /// (116).
    #[test]
    fn w4_f10_the_critter_family_cockatiel_and_single_gores() {
        let w = air();
        // The gray cockatiel 675.
        let n = boss(100.0, 100.0, 30.0, 24.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(599);
        critter_family_death(&n, 675, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..3].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![1286, 1287, 1287]);
        // The single-gore critters.
        for (ty, gore_ty) in [(299, 433), (538, 897), (300, 434), (610, 1163), (626, 1197), (102, 116)] {
            let mut g = GorePool::new();
            let mut d = DustPool::new();
            let mut r = UnifiedRandom::new(599);
            critter_family_death(&n, ty, &mut d, &ctx(), &mut g, &mut r, &w, 1.0);
            assert_eq!(g.slot(0).gore_type, gore_ty, "critter {ty}");
        }
    }

    /// `NPC.cs:86058-86075` — the flying fish 224 (312/313), eyeball flying
    /// fish 587 (extra 50-dust burst + 1104/1105), turtle 616
    /// (1166/1167/1168/1168).
    #[test]
    fn w4_f10_the_critter_family_flying_fish_turtle() {
        let w = air();
        let n = boss(100.0, 100.0, 30.0, 24.0);
        // The flying fish 224.
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(601);
        critter_family_death(&n, 224, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..2].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![312, 313]);
        // The eyeball flying fish 587: the extra dust burst runs.
        let mut gore2 = GorePool::new();
        let mut dust2 = DustPool::new();
        let mut rng2 = UnifiedRandom::new(601);
        critter_family_death(&n, 587, &mut dust2, &ctx(), &mut gore2, &mut rng2, &w, 1.0);
        let types2: Vec<i32> = gore2.slots()[..2].iter().map(|g| g.gore_type).collect();
        assert_eq!(types2, vec![1104, 1105]);
        // 10 base dusts + 50 extra = 60 (minus any cull refusals).
        let dust_active = dust2.slots().iter().filter(|d| d.active).count();
        assert!(dust_active >= 50, "the extra burst ran: {dust_active}");
        // The turtle 616.
        let mut gore3 = GorePool::new();
        let mut dust3 = DustPool::new();
        let mut rng3 = UnifiedRandom::new(601);
        critter_family_death(&n, 616, &mut dust3, &ctx(), &mut gore3, &mut rng3, &w, 1.0);
        let types3: Vec<i32> = gore3.slots()[..4].iter().map(|g| g.gore_type).collect();
        assert_eq!(types3, vec![1166, 1167, 1168, 1168]);
    }

    /// `NPC.cs:86079-86089` — the jungle turtle 617 (1169/1170/1171/1171)
    /// and sea turtle 625 (1194/1195/1196/1196).
    #[test]
    fn w4_f10_the_critter_family_jungle_sea_turtle() {
        let w = air();
        for (ty, gores) in [(617, [1169, 1170, 1171, 1171]), (625, [1194, 1195, 1196, 1196])] {
            let n = boss(100.0, 100.0, 40.0, 30.0);
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(607);
            critter_family_death(&n, ty, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
            let types: Vec<i32> = gore.slots()[..4].iter().map(|g| g.gore_type).collect();
            assert_eq!(types, gores, "turtle {ty}");
        }
    }

    /// `NPC.cs:86108-86125` — the penguin family: 148→[160,161],
    /// 149→[162,163], 168→[215,216], 470→[673,672,674] (673 before 672).
    #[test]
    fn w4_f10_the_penguin_family_death_uses_the_per_type_list() {
        let w = air();
        let cases: [(i32, &[i32]); 4] = [
            (148, &[160, 161]),
            (149, &[162, 163]),
            (168, &[215, 216]),
            (470, &[673, 672, 674]),
        ];
        for (ty, gores) in cases {
            let n = boss(100.0, 100.0, 30.0, 30.0);
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(613);
            penguin_family_death(&n, ty, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
            let types: Vec<i32> = gore.slots()[..gores.len()].iter().map(|g| g.gore_type).collect();
            assert_eq!(types, gores, "penguin {ty}");
        }
    }

    /// `NPC.cs:86142-86143` — the Frog death: the 2 gores 551/552 at
    /// position, after the 10-dust type-5 burst.
    #[test]
    fn w4_f10_the_frog_death_spawns_the_two_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 30.0, 24.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(617);
        frog_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..2].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![551, 552]);
    }

    /// `NPC.cs:86187-86188` — the Pupfish death: the 2 gores 1152/1153.
    #[test]
    fn w4_f10_the_pupfish_death_spawns_the_two_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 24.0, 20.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(619);
        pupfish_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..2].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![1152, 1153]);
    }

    /// `NPC.cs:86206-86214` — the Pufferfish death: the ai[2]==1 (puffed)
    /// branch spawns 1388/1388/1389/1390, else 1386/1387.
    #[test]
    fn w4_f10_the_pufferfish_death_branches_on_ai2() {
        let w = air();
        // Puffed.
        let mut n = boss(100.0, 100.0, 30.0, 30.0);
        n.ai[2] = 1.0;
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(631);
        pufferfish_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..4].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![1388, 1388, 1389, 1390]);
        // Not puffed.
        let mut n2 = boss(100.0, 100.0, 30.0, 30.0);
        n2.ai[2] = 0.0;
        let mut gore2 = GorePool::new();
        let mut dust2 = DustPool::new();
        let mut rng2 = UnifiedRandom::new(631);
        pufferfish_death(&n2, &mut dust2, &ctx(), &mut gore2, &mut rng2, &w, 1.0);
        let types2: Vec<i32> = gore2.slots()[..2].iter().map(|g| g.gore_type).collect();
        assert_eq!(types2, vec![1386, 1387]);
    }

    /// `NPC.cs:86287-86288` — the Seagull death: the 2 gores 1150/1151.
    #[test]
    fn w4_f10_the_seagull_death_spawns_the_two_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 30.0, 24.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(641);
        seagull_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..2].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![1150, 1151]);
    }

    /// `NPC.cs:86233-86241` — the Scorpion death: 366→[561,562,563],
    /// 367→[564,565,566], after the 10-dust type-203 burst.
    #[test]
    fn w4_f10_the_scorpion_death_uses_the_per_type_list() {
        let w = air();
        for (ty, gores) in [(366, [561, 562, 563]), (367, [564, 565, 566])] {
            let n = boss(100.0, 100.0, 36.0, 24.0);
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(643);
            scorpion_death(&n, ty, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
            let types: Vec<i32> = gore.slots()[..3].iter().map(|g| g.gore_type).collect();
            assert_eq!(types, gores, "scorpion {ty}");
        }
    }

    /// `NPC.cs:86307-86309` — the Grebe death: the 3 gores 1154/1155/1156.
    #[test]
    fn w4_f10_the_grebe_death_spawns_the_three_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 30.0, 24.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(647);
        grebe_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..3].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![1154, 1155, 1156]);
    }

    /// `NPC.cs:86260-86268` — the Duck family death: 362/363→[555,556,557],
    /// 364/365→[558,559,560].
    #[test]
    fn w4_f10_the_duck_family_death_uses_the_per_type_list() {
        let w = air();
        for (ty, gores) in [(362, [555, 556, 557]), (363, [555, 556, 557]), (364, [558, 559, 560]), (365, [558, 559, 560])] {
            let n = boss(100.0, 100.0, 30.0, 24.0);
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(653);
            duck_family_death(&n, ty, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
            let types: Vec<i32> = gore.slots()[..3].iter().map(|g| g.gore_type).collect();
            assert_eq!(types, gores, "duck {ty}");
        }
    }

    /// `NPC.cs:86326-86330` — the Nutcracker death: the 5 gores
    /// 528/529/530/531/532.
    #[test]
    fn w4_f10_the_nutcracker_death_spawns_the_five_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 40.0, 50.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(659);
        nutcracker_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..5].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![528, 529, 530, 531, 532]);
    }

    /// `NPC.cs:86346-86350` — the Krampus death: the 5 gores 538-542.
    #[test]
    fn w4_f10_the_krampus_death_spawns_the_five_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 40.0, 50.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(661);
        krampus_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..5].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![538, 539, 540, 541, 542]);
    }

    /// `NPC.cs:86366-86369` — the ElfArcher death: the 4 gores 533-536.
    #[test]
    fn w4_f10_the_elf_archer_death_spawns_the_four_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 40.0, 50.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(673);
        elf_archer_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..4].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![533, 534, 535, 536]);
    }

    /// `NPC.cs:86387-86406` — the corrupt/crimson critter family:
    /// 57→[84], 58→[85], 464→[668,669], 465→[670,671], 47 else→[78,79].
    #[test]
    fn w4_f10_the_corrupt_critter_death_uses_the_per_type_list() {
        let w = air();
        let cases: [(i32, &[i32]); 5] = [
            (47, &[78, 79]),
            (57, &[84]),
            (58, &[85]),
            (464, &[668, 669]),
            (465, &[670, 671]),
        ];
        for (ty, gores) in cases {
            let n = boss(100.0, 100.0, 30.0, 24.0);
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(677);
            corrupt_critter_death(&n, ty, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
            let types: Vec<i32> = gore.slots()[..gores.len()].iter().map(|g| g.gore_type).collect();
            assert_eq!(types, gores, "corrupt critter {ty}");
        }
    }

    /// `NPC.cs:86425-86446` — the crimson monster family: 173→[223,224],
    /// 174→[225,226], 181→[237], 182→[238,239], 268→[403,404,405], after
    /// the 50-dust type-5 alpha burst.
    #[test]
    fn w4_f10_the_crimson_monster_death_uses_the_per_type_list() {
        let w = air();
        let cases: [(i32, &[i32]); 5] = [
            (173, &[223, 224]),
            (174, &[225, 226]),
            (181, &[237]),
            (182, &[238, 239]),
            (268, &[403, 404, 405]),
        ];
        for (ty, gores) in cases {
            let mut n = boss(100.0, 100.0, 36.0, 30.0);
            n.alpha = 30;
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(683);
            crimson_monster_death(&n, ty, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
            let types: Vec<i32> = gore.slots()[..gores.len()].iter().map(|g| g.gore_type).collect();
            assert_eq!(types, gores, "crimson monster {ty}");
        }
    }

    /// `NPC.cs:86465-86501` — the demon-eye family: the (head, tail) pairs
    /// 190→(249,2), 191→(248,2), 192→(247,2), 193→(252,253), 194→(250,251),
    /// 317→(447,448), 318→(449,450), 2 else→(1,2), the tail at
    /// position+(14,0), both at this.scale.
    #[test]
    fn w4_f10_the_demon_eye_death_uses_the_per_type_pair() {
        let w = air();
        let cases: [(i32, i32, i32); 8] = [
            (190, 249, 2),
            (191, 248, 2),
            (192, 247, 2),
            (193, 252, 253),
            (194, 250, 251),
            (317, 447, 448),
            (318, 449, 450),
            (2, 1, 2),
        ];
        for (ty, head, tail) in cases {
            let mut n = boss(100.0, 100.0, 30.0, 30.0);
            n.scale = 1.2;
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(691);
            demon_eye_death(&n, ty, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
            assert_eq!(gore.slot(0).gore_type, head, "eye {ty} head");
            assert_eq!(gore.slot(1).gore_type, tail, "eye {ty} tail");
            assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
            assert_eq!(gore.slot(1).position, Vec2::new(114.0, 100.0));
            assert_eq!(gore.slot(0).scale, 1.2);
        }
    }

    /// `NPC.cs:86518-86519` — the Arapaima death: the 2 gores 186/187, the
    /// tail at position+(14,0).
    #[test]
    fn w4_f10_the_arapaima_death_spawns_the_two_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 40.0, 24.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(701);
        arapaima_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        assert_eq!(gore.slot(0).gore_type, 186);
        assert_eq!(gore.slot(1).gore_type, 187);
        assert_eq!(gore.slot(1).position, Vec2::new(114.0, 100.0));
    }

    /// `NPC.cs:86529-86542` — the WanderingEye: the hit-half eye-pop latch
    /// (below half life, localAI[0]==0 -> gore 1 + latch), and the death
    /// half's 2 gores 155/155 at position and +14 Y.
    #[test]
    fn w4_f10_the_wandering_eye_hit_latches_and_death_spawns() {
        let w = air();
        // Hit below half life: the eye-pop fires once and latches.
        let mut n = boss(100.0, 100.0, 30.0, 30.0);
        n.hp = 40;
        n.max_hp = 100;
        n.local_ai[0] = 0.0;
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(709);
        wandering_eye_hit(&mut n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        assert_eq!(gore.slot(0).gore_type, 1);
        assert_eq!(n.local_ai[0], 1.0);
        // A second hit: the latch blocks the eye-pop.
        let before = gore.slots().iter().filter(|g| g.active).count();
        wandering_eye_hit(&mut n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        assert_eq!(gore.slots().iter().filter(|g| g.active).count(), before);
        // Death: the 2 gores 155/155.
        let mut n2 = boss(100.0, 100.0, 30.0, 30.0);
        n2.hp = 0;
        let mut gore2 = GorePool::new();
        let mut dust2 = DustPool::new();
        let mut rng2 = UnifiedRandom::new(709);
        wandering_eye_hit(&mut n2, &mut dust2, &ctx(), &mut gore2, &mut rng2, &w, 1.0);
        let types: Vec<i32> = gore2.slots()[..2].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![155, 155]);
        assert_eq!(gore2.slot(1).position, Vec2::new(100.0, 114.0));
    }

    /// `NPC.cs:86568-86604` — the antlion family: 69→[97,98],
    /// 580→[1094-1097], 581→[1098,1099,1100,1101,1101],
    /// 508→[811,812,813,814], 509→[815,816,817,818,818], 582→[1102,1103]
    /// (10-dust burst), after the type-250 dust burst.
    #[test]
    fn w4_f10_the_antlion_death_uses_the_per_type_list() {
        let w = air();
        let cases: [(i32, &[i32]); 6] = [
            (69, &[97, 98]),
            (580, &[1094, 1095, 1096, 1097]),
            (581, &[1098, 1099, 1100, 1101, 1101]),
            (508, &[811, 812, 813, 814]),
            (509, &[815, 816, 817, 818, 818]),
            (582, &[1102, 1103]),
        ];
        for (ty, gores) in cases {
            let n = boss(100.0, 100.0, 40.0, 30.0);
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(719);
            antlion_death(&n, ty, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
            let types: Vec<i32> = gore.slots()[..gores.len()].iter().map(|g| g.gore_type).collect();
            assert_eq!(types, gores, "antlion {ty}");
        }
    }

    /// `NPC.cs:86621-86623` — the Vulture death: the 3 gores 86 at position
    /// and 87/88 at +14 X.
    #[test]
    fn w4_f10_the_vulture_death_spawns_the_three_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 40.0, 30.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(727);
        vulture_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..3].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![86, 87, 88]);
        assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
        assert_eq!(gore.slot(1).position, Vec2::new(114.0, 100.0));
        assert_eq!(gore.slot(2).position, Vec2::new(114.0, 100.0));
    }

    /// `NPC.cs:86639-86640` — the Raven death: the 2 gores 438/439, tail at
    /// +14 X.
    #[test]
    fn w4_f10_the_raven_death_spawns_the_two_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 36.0, 30.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(733);
        raven_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        assert_eq!(gore.slot(0).gore_type, 438);
        assert_eq!(gore.slot(1).gore_type, 439);
        assert_eq!(gore.slot(1).position, Vec2::new(114.0, 100.0));
    }

    /// `NPC.cs:86656-86657` — the Parrot death: the 2 gores 373/374.
    #[test]
    fn w4_f10_the_parrot_death_spawns_the_two_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 30.0, 24.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(739);
        parrot_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..2].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![373, 374]);
    }

    /// `NPC.cs:86675-86708` — the blood-zombie ladder: 489→(722,723,724),
    /// 586→(1106,1107,1108), 534→(894,895,896), the 5-gore +0/+20/+34
    /// ladder at this.scale.
    #[test]
    fn w4_f10_the_blood_zombie_ladder_uses_the_per_type_triple() {
        let w = air();
        let cases: [(fn(&Npc, &mut DustPool, &SpawnCtx, &mut GorePool, &mut UnifiedRandom, &World, f32), i32, i32, i32); 3] = [
            (blood_zombie_death, 722, 723, 724),
            (zombie_merman_death, 1106, 1107, 1108),
            (demon_tax_collector_death, 894, 895, 896),
        ];
        for (f, head, middle, tail) in cases {
            let mut n = boss(100.0, 100.0, 40.0, 56.0);
            n.scale = 1.1;
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(743);
            f(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
            let types: Vec<i32> = gore.slots()[..5].iter().map(|g| g.gore_type).collect();
            assert_eq!(types, vec![head, middle, middle, tail, tail]);
            assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
            assert_eq!(gore.slot(1).position, Vec2::new(100.0, 120.0));
            assert_eq!(gore.slot(3).position, Vec2::new(100.0, 134.0));
            assert_eq!(gore.slot(0).scale, 1.1);
        }
    }

    /// `NPC.cs:86724-86728` — the Drippler death: the 5 gores 725/726/727
    /// fixed + two 725+Next(3). The twin replay pins the two type draws
    /// before their spawns.
    #[test]
    fn w4_f10_the_drippler_death_draws_the_two_random_types() {
        let w = air();
        let mut n = boss(100.0, 100.0, 40.0, 40.0);
        n.scale = 1.0;
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(751);
        drippler_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        assert_eq!(gore.slot(0).gore_type, 725);
        assert_eq!(gore.slot(1).gore_type, 726);
        assert_eq!(gore.slot(2).gore_type, 727);
        assert!((725..728).contains(&gore.slot(3).gore_type));
        assert!((725..728).contains(&gore.slot(4).gore_type));
        // The twin: 50 type-5 dusts, then 3 fixed gores' kicks, then 2 x
        // (Next(3) + 2 kicks).
        let mut twin = UnifiedRandom::new(751);
        for _ in 0..50 {
            let _px = twin.next_max(36);
            let _py = twin.next_max(36);
            let _vx = twin.next_range(-20, 21);
            let _vy = twin.next_range(-20, 21);
            let _row = twin.next_max(3);
            let _scale = twin.next_range(-20, 21);
        }
        for _ in 0..3 {
            let _ky = twin.next_range(10, 31);
            let _kx = twin.next_range(-20, 21);
        }
        for _ in 0..2 {
            let _ty = twin.next_max(3);
            let _ky = twin.next_range(10, 31);
            let _kx = twin.next_range(-20, 21);
        }
        assert_eq!(rng.peek(), twin.peek(), "the drippler mirrors the C# draws");
    }

    /// `NPC.cs:86746-86784` — the shark family: 65→[89,90,91,92],
    /// 692→[1403,1404,1405,1406], 542→[944,945,946,947],
    /// 543→[948,949,950,951], 544→[952,953,954,955], 545→[956,957,958,959],
    /// the head at position and the three tails at +14 X, all at velocity
    /// x0.8.
    #[test]
    fn w4_f10_the_shark_family_death_uses_the_per_type_quad() {
        let w = air();
        let cases: [(i32, [i32; 4]); 6] = [
            (65, [89, 90, 91, 92]),
            (692, [1403, 1404, 1405, 1406]),
            (542, [944, 945, 946, 947]),
            (543, [948, 949, 950, 951]),
            (544, [952, 953, 954, 955]),
            (545, [956, 957, 958, 959]),
        ];
        for (ty, gores) in cases {
            let mut n = boss(100.0, 100.0, 50.0, 30.0);
            n.vx = 5.0;
            n.vy = 0.0;
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(757);
            shark_family_death(&n, ty, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
            let types: Vec<i32> = gore.slots()[..4].iter().map(|g| g.gore_type).collect();
            assert_eq!(types, gores, "shark {ty}");
            assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
            assert_eq!(gore.slot(1).position, Vec2::new(114.0, 100.0));
            assert_eq!(gore.slot(3).position, Vec2::new(114.0, 100.0));
            // The velocity x0.8 plus the kicks.
            assert!((gore.slot(0).velocity.x - 4.0).abs() <= 2.0);
        }
    }

    /// `NPC.cs:86801-86805` / `:86842-86846` — the Nymph (254/255/256,
    /// type-5 dust) and IcyMerman (273/274/275, type-137 dust) deaths: the
    /// 5-gore +0/+20/+34 ladder at this.scale after the 75-dust burst.
    #[test]
    fn w4_f10_the_nymph_and_icy_merman_deaths_use_the_ladder() {
        let w = air();
        for (f, gores) in [
            (nymph_death as fn(&Npc, &mut DustPool, &SpawnCtx, &mut GorePool, &mut UnifiedRandom, &World, f32), [254, 255, 255, 256, 256]),
            (icy_merman_death, [273, 274, 274, 275, 275]),
        ] {
            let mut n = boss(100.0, 100.0, 40.0, 56.0);
            n.scale = 1.0;
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(761);
            f(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
            let types: Vec<i32> = gore.slots()[..5].iter().map(|g| g.gore_type).collect();
            assert_eq!(types, gores);
            assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
            assert_eq!(gore.slot(1).position, Vec2::new(100.0, 120.0));
            assert_eq!(gore.slot(3).position, Vec2::new(100.0, 134.0));
        }
    }

    /// `NPC.cs:86821-86826` — the Lihzahrd death: the 6 gores 258 at
    /// position then 259/259/260/260/261 ALL at +20 Y (no +34 tier).
    #[test]
    fn w4_f10_the_lihzahrd_death_spawns_the_six_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 40.0, 56.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(769);
        lihzahrd_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..6].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![258, 259, 259, 260, 260, 261]);
        assert_eq!(gore.slot(0).position, Vec2::new(100.0, 100.0));
        for i in 1..6 {
            assert_eq!(gore.slot(i).position, Vec2::new(100.0, 120.0));
        }
    }

    /// `NPC.cs:86862-86866` — the GingerbreadMan death: the 5-gore ladder
    /// 500/501/502 after the 50-dust type-0 SpeedX-2.5 burst.
    #[test]
    fn w4_f10_the_gingerbread_man_death_spawns_the_ladder() {
        let w = air();
        let n = boss(100.0, 100.0, 40.0, 56.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(773);
        gingerbread_man_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..5].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![500, 501, 501, 502, 502]);
    }

    /// `NPC.cs:86884-86896` — the ZombieElf death: the per-type head
    /// (338→494, 339→495, 340→496) + the shared 497/498 at +20 and 499 at
    /// +34.
    #[test]
    fn w4_f10_the_zombie_elf_death_uses_the_per_type_head() {
        let w = air();
        for (ty, head) in [(338, 494), (339, 495), (340, 496)] {
            let n = boss(100.0, 100.0, 40.0, 56.0);
            let mut gore = GorePool::new();
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(787);
            zombie_elf_death(&n, ty, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
            let types: Vec<i32> = gore.slots()[..4].iter().map(|g| g.gore_type).collect();
            assert_eq!(types, vec![head, 497, 498, 499], "elf {ty}");
            assert_eq!(gore.slot(1).position, Vec2::new(100.0, 120.0));
            assert_eq!(gore.slot(3).position, Vec2::new(100.0, 134.0));
        }
    }

    /// `NPC.cs:86912-86915` — the Yeti death: the 4 gores 503/504/505/506
    /// (the DISTINCT 504/505 pair at +20).
    #[test]
    fn w4_f10_the_yeti_death_spawns_the_four_gores() {
        let w = air();
        let n = boss(100.0, 100.0, 50.0, 60.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(797);
        yeti_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..4].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![503, 504, 505, 506]);
        assert_eq!(gore.slot(1).position, Vec2::new(100.0, 120.0));
        assert_eq!(gore.slot(2).position, Vec2::new(100.0, 120.0));
        assert_eq!(gore.slot(3).position, Vec2::new(100.0, 134.0));
    }

    /// `NPC.cs:86931-86935` — the MossZombie death: the 5-gore ladder
    /// 1391/1392/1393 after the 50-dust type-5 SpeedX-2.5 burst.
    #[test]
    fn w4_f10_the_moss_zombie_death_spawns_the_ladder() {
        let w = air();
        let n = boss(100.0, 100.0, 40.0, 56.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(809);
        moss_zombie_death(&n, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0);
        let types: Vec<i32> = gore.slots()[..5].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![1391, 1392, 1392, 1393, 1393]);
    }

    /// `NPC.cs:86982-86995` — the zombie block: 104 Werewolf ladder
    /// 117/118/119, 632 MaggotZombie ladder 1214/1215/1216 + the maggot
    /// spawn seam returned.
    #[test]
    fn w4_f10_the_zombie_block_werewolf_and_maggot() {
        let w = air();
        // The Werewolf 104.
        let n = boss(100.0, 100.0, 40.0, 56.0);
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(811);
        let mut out = HitEffectOutcomeC::default();
        zombie_block_death(&n, 104, &mut dust, &ctx(), &mut gore, &mut rng, &w, 1.0, &mut out);
        let types: Vec<i32> = gore.slots()[..5].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![117, 118, 118, 119, 119]);
        assert!(out.maggot_zombie_spawns.is_none());
        // The MaggotZombie 632.
        let mut gore2 = GorePool::new();
        let mut dust2 = DustPool::new();
        let mut rng2 = UnifiedRandom::new(811);
        let mut out2 = HitEffectOutcomeC::default();
        zombie_block_death(&n, 632, &mut dust2, &ctx(), &mut gore2, &mut rng2, &w, 1.0, &mut out2);
        let types2: Vec<i32> = gore2.slots()[..5].iter().map(|g| g.gore_type).collect();
        assert_eq!(types2, vec![1214, 1215, 1215, 1216, 1216]);
        assert_eq!(out2.maggot_zombie_spawns, Some((100.0, 100.0, 40.0, 56.0)));
    }

    /// The wave-C dispatch gates the flipped Betsy branch on
    /// `spriteDirection == -1` and `life <= 0`; a non-flipped or living
    /// Betsy spawns nothing here (the non-flipped branch is wave B's).
    #[test]
    fn w4_f10_the_dispatch_gates_the_flipped_betsy_branch() {
        let w = air();
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(5);
        // Flipped + dead -> the 10 gores.
        let mut n = boss(100.0, 100.0, 232.0, 156.0);
        hit_effect_c(&mut n, 551, 1.0, 10.0, &mut gore, &mut dust, &ctx(), &mut rng, &w);
        assert_eq!(gore.slots().iter().filter(|g| g.active).count(), 10);
        // Non-flipped -> nothing here (wave B's branch).
        let mut gore2 = GorePool::new();
        let mut dust2 = DustPool::new();
        let mut n2 = boss(100.0, 100.0, 232.0, 156.0);
        n2.sprite_dir = 1;
        hit_effect_c(&mut n2, 551, 1.0, 10.0, &mut gore2, &mut dust2, &ctx(), &mut rng, &w);
        assert_eq!(gore2.slots().iter().filter(|g| g.active).count(), 0);
        // Living -> nothing (the hit half is dust-only, the wiring wave's).
        let mut gore3 = GorePool::new();
        let mut dust3 = DustPool::new();
        let mut n3 = boss(100.0, 100.0, 232.0, 156.0);
        n3.hp = 50;
        hit_effect_c(&mut n3, 551, 1.0, 10.0, &mut gore3, &mut dust3, &ctx(), &mut rng, &w);
        assert_eq!(gore3.slots().iter().filter(|g| g.active).count(), 0);
    }
}
