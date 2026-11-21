//! W4-f35 — the expert/master boss drop trees: the census.
//!
//! Scope: the two loot trees that ride `Main.expertMode`/`Main.masterMode`
//! per boss —
//! 1. the DEATH side: `NPC.NPCLoot` (`NPC.cs:78830-78899`) →
//!    `NPCLoot_DropItems` (`:78925-78937`) → the `ItemDropDatabase`
//!    registrations (`ItemDropDatabase.cs:286-676`'s `RegisterBoss_*`),
//!    where `ItemDropRule.BossBag` (`ItemDropRule.cs:10-13`) is a
//!    `DropBasedOnExpertMode(DropNothing, DropLocalPerClientAndResetsNPCMoneyTo0)`
//!    — the bag ONLY in expert, and its `CommonCode` zeroes `npc.value`
//!    (`CommonCode.cs:29-39`); the master relics/pets ride
//!    `MasterModeCommonDrop`/`MasterModeDropOnAllPlayers` behind
//!    `Conditions.IsMasterMode`;
//! 2. the OPEN side: `Player.OpenBossBag(int type)`
//!    (`Player.cs:7202-7883`), the per-bag grant switch the inventory
//!    right-click reaches through `ItemSlot.TryOpenContainer`
//!    (`UI/ItemSlot.cs:1786-1811`, the `ItemID.Sets.BossBag` gate) —
//!    the expert-exclusive items (`Worm Scarf` 3224, `Brain of Confusion`
//!    3223, `Hive Backpack` 3333, `Bone Glove` 3245, `Demon Heart` 3335,
//!    the mechanical cart pieces 3354-3356, `Spore Sac` 3336,
//!    `Shiny Stone` 3337, `Shrimpy Truffle` 3367, the Empress/Queen
//!    Slime/Deerclops exclusives) plus the master-mode stack bumps
//!    (the EoW/BoC bags' ore/ingot counts) and the shared tails
//!    (`TryGettingDevArmor` `:7885-8026`, the bag money split
//!    `:7744-7882`).
//!
//! ## Port status BEFORE this task
//!
//! | Piece | C# | Port | Status |
//! | --- | --- | --- | --- |
//! | the rule engine + expert/master conditions | `GameContent/ItemDropRules/*` | `tr_core::loot` | done (S1-E-14/15) |
//! | the death-side boss registrations (bags, NotExpert weapons, master relics/pets, the mech/tower/moon rows) | `ItemDropDatabase.cs:286-676` | `loot_db_gen.rs` (gen_loot.py) | done (S1-E-14a, 1599 registrations) |
//! | the boss-bag rule + money reset | `DropLocalPerClientAndResetsNPCMoneyTo0.cs` | `loot.rs::DropLocalPerClientAndResetsMoney` | done |
//! | `Player.OpenBossBag` — ALL 19 openable bags | `Player.cs:7202-7742` | — | **MISSING** (this task) |
//! | `TryGettingDevArmor` (the 1/16 dev-set roll, 21 sets) | `Player.cs:7885-8026` | — | **MISSING** (this task) |
//! | the bag money tail (the fresh-`SetDefaults` value + the 4 bonus chains + the coin split) | `Player.cs:7744-7882` | — | **MISSING** (this task) |
//! | the inventory right-click open arm | `ItemSlot.cs:1660-1667`/`:1786-1811` | `clicks.rs` notes the gap | **MISSING** (this task) |
//! | `DoDeathEvents_DropBossPotionsAndHearts` (the boss potion + 5-9 hearts + Badger's Hat) | `NPC.cs:78945-79016` | — | missing, OUT of this task (the death-events lane) |
//! | `NPCLoot_DropHeals`/`DropCommonLifeAndMana` | `NPC.cs:79098-79195`/`:79531-79557` | — | missing, OUT of this task (same lane) |
//!
//! ## The bag set (`ItemID.cs:1383-1384`)
//!
//! `Sets.BossBag` (21 ids): 3318-3332, 3860, 3861, 3862, 4782, 4957,
//! 5111. `OpenBossBag`'s grant switch covers 19 of them — **3861 (Ogre)
//! and 3862 (Dark Mage) have NO case**: opening them draws nothing from
//! the switch, but the money tail still runs (`num9 = 576/564` at
//! `:7809-7816`; both bosses' `SetDefaults` value is 0f, `NPC.cs:16286`/
//! `:16456`, so the tail draws its 5 gates and mints nothing).
//!
//! ## The roll order (the RNG contract)
//!
//! `OpenBossBag` runs entirely on `Main.rand` — the inventory right-click
//! is a UI event OUTSIDE every `SwapRandom` block, so the port feeds
//! `NamedRngs::global()` (the bare-`Main.rand` stand-in, the reforge
//! precedent `clicks.rs:1027`). Per bag, in C# order:
//!
//! 1. the bag's own grant switch (each arm's draws verbatim — the
//!    discarded `num5`/`num6` draws of the EoW/BoC bags `:7257`/`:7287`,
//!    the King Slime do-while re-pick `:7218-7222`, the Queen Slime
//!    re-pick that redraws the FIRST slot `:7664-7669`, the Moon Lord
//!    `List.RemoveAt` bundle `:7734-7740`);
//! 2. the money tail: `num9` map → a FRESH `NPC.SetDefaults(num9)` (the
//!    CLASSIC base value — expert scaling rides `ScaleStats` at the
//!    spawn, e25's lane; the bag money uses the unscaled `value`) →
//!    `value * (1 + Next(-20,21)*0.01f)` → the 1/5, 1/10, 1/15, 1/20
//!    bonus chains → the platinum/gold/silver/copper split with the
//!    `(int)` truncations and the copper `< 1 → 1` floor;
//! 3. every grant rides `QuickSpawnItem` (`Player.cs:7170-7191`):
//!    `stack > 0` gate → `SetDefaults` → `Prefix(-1)` (the natural roll,
//!    `Item.cs:504-618`) → `GetOrDropItem(PickupItemFromWorld)` — the
//!    inventory fill, the overflow a world drop.
//!
//! `TryGettingDevArmor` runs FIRST inside the hardmode bags' arms
//! (3325-3332, 3860, 4782 — NOT 4957/5111): `Next(tenthAnniversary ? 8
//! : 16) == 0` → `Next(21)` picks the set → 3-5 `QuickSpawnItem` calls.
//!
//! ## The classic base values (`NPC.SetDefaults`, the money tail's input)
//!
//! | bag | boss type | value (copper) | cite |
//! | --- | --- | --- | --- |
//! | 3318 | 50 King Slime | 10000f | `NPC.cs:9400` |
//! | 3319 | 4 EoC | 30000f | `:8717` |
//! | 3320 | 13 EoW head | 800f | `:8919` |
//! | 3321 | 266 BoC | 50000f | `:12453` |
//! | 3322 | 222 Queen Bee | 50000f | `:11869` |
//! | 3323 | 35 Skeletron | 50000f | `:9178` |
//! | 3324 | 113 WoF | 80000f | `:10359` |
//! | 3325 | 134 Destroyer | 120000f | `:10674` |
//! | 3326 | 125 Retinazer | 120000f | `:10536` |
//! | 3327 | 127 Prime | 120000f | `:10568` |
//! | 3328 | 262 Plantera | `Item.buyPrice(0,15)` = 150000 | `:12391`, `Item.cs:49417-49420` |
//! | 3329 | 245 Golem | `Item.buyPrice(0,15)` = 150000 | `:12174` |
//! | 3330 | 370 Fishron | 250000f | `:13715` |
//! | 3331 | 439 Cultist | 100000f | `:14226` |
//! | 3332 | 398 Moon Lord | 1000000f | `:14130` |
//! | 3860 | 551 Betsy | 0f | `:16528` |
//! | 3861 | 576 Ogre | 0f | `:16286` |
//! | 3862 | 564 Dark Mage | 0f | `:16456` |
//! | 4782 | 636 Empress | 250000f | `:17200` |
//! | 4957 | 657 Queen Slime | 60000f | `:17322` |
//! | 5111 | 668 Deerclops | `Item.buyPrice(0,5)` = 50000 | `:17463` |
//!
//! ## The wiring seam
//!
//! `boss_bags.rs` (this task) carries the pure grant trees over a
//! `BagGrant` list + the `BagEnv` scalars (`master_mode`, `crimson`,
//! `remix`, `tenth_anniversary`, `extra_accessory`, `has_portal_gun`);
//! the `clicks.rs` right-click arm drains the grants through the
//! `Player.GetItem` fill (the bank `loot_all` shape) with the overflow
//! spawned as world drops. The e25 scaling module stays UNWIRED and
//! untouched — the bag money reads the classic `SetDefaults` value
//! exactly like the C# (the expert `value ×2.5` rides the DEATH-side
//! `ScaleStats`, not the bag).
//!
//! ## The widened scope (2026-11-21): the full OpenableBag set + the death heals
//!
//! The `ItemID.Sets.OpenableBag` set (`ItemID.cs:1384`) is 58 kinds:
//! 21 boss bags (above) + 26 fishing crates + 11 misc/present kinds.
//! The set diff after the boss-bag landing: EVERY remaining kind now
//! has a ported open-arm (the four family modules below).
//!
//! ### The crate open trees (`fishing_crates.rs`)
//!
//! `Player.OpenFishingCrate` (`Player.cs:8028-9250`), the OPEN side
//! only — the fishing-catch crate roll (`Projectile.cs:21052`) is
//! W4-f36's lane. `IsFishingCrate` (`ItemID.cs:1309`): 2334/2335/2336,
//! 3203-3208, 4405/4407, 4877, 5002 + the hardmode variants 3979-3987,
//! 4406/4408, 4878, 5003 (`IsFishingCrateHardmode` `:1310`, 13 ids —
//! the opener's `flag`).
//!
//! | Crate | ids | C# window | Shape |
//! | --- | --- | --- | --- |
//! | Wooden | 2334/3979 | `:8034-8262` | the retry while + the 1/3 heal + 1/3 bait tails; the Next(4)-over-six-cases dead bar/ore picks; the hardmode gate draws NO pick on a miss |
//! | Iron | 2335/3980 | `:8263-8441` | the retry while (5 accessory gates) + the 1/2 lesser-potion + 1/2 bait tails; the ore hardmode re-pick SUBTRACTS Next(2) from the stack |
//! | Golden | 2336/3981 | `:8442-8593` | the retry while (the else-branch Next(3)!=0 continue) + the 1/3 potion + 1/2 lesser + 2/3 master-bait tails + the TWO separate 989 gates (1/30 pre, 1/15 hard) |
//! | Corrupt | 3203/3982 | shared | the 5-way weapon (162/111/96/115/64); the hardmode souls tail 521/522 |
//! | Crimson | 3204/3983 | shared | the 5-way weapon (800/802/1256/1290/3062); the hardmode souls 521/1332 |
//! | Dungeon | 3205/3984 | shared | the Lock Box 3085 + the 1/2 Book 149 x5-15 + the 1/2 brick pick 134/137/139 x25-50; the 1/2 25-way painting tail |
//! | Sky | 3206/3985 | shared | the 4-way weapon (158/65/159/2219); the 1/2 6-way furniture + 1/2 Cloud 751 x50-100 + 1/40 + the lowTiles-gated 1/3 Sky Mill 2197 |
//! | Hallowed | 3207/3986 | shared | NO weapon arm; the 3986-only hardmode souls 520/502 (the second stack RE-DRAWN range(4,11)) |
//! | Jungle | 3208/3987 | shared | the 1/20 Leaf Blower 3017 (a DISCARDED Next(5)) else the 5-way (212/964/211/213/2292); the 1/2 4564 x20-50 + 1/20 753 tails |
//! | Frozen | 4405/4406 | shared | the 7-way SelectRandom (670/724/950/1319/987/1579/6153), the remix 1319->725 swap; the 1/20 669 tail |
//! | Oasis | 4407/4408 | shared | the 8-way weapon + the 1/4 Magic Conch 4423 + 1/2 Fossil 3380 + 1/35 Sandstorm 857 (every pass); the 1/2 14-way fossil tail |
//! | Lava | 4877/4878 | shared | the 1/20 Lava Charm 906 (a DISCARDED Next(5)) else the 5-way (4822/4828/4880/4881/4868) + the 1/4 4858 x2 INSIDE the weapon gate; the ALWAYS 4879 + the 1/3 4824 + 1/2 4902-4906; the bars/ores RE-TYPE to 174/175 AFTER the picks; the 1/2 12-way furniture + four 1/20 tails |
//! | Ocean | 5002/5003 | shared | the Next(4) BEFORE the 1/10 Water Bolt 863 gate (186/4404/277/187); the 1/5 5527 + 1/3 4090 x20-50 + two 1/10 tails |
//!
//! The shared loop (`:8595-8914`): `maxValue = 1` — the biome-weapon
//! gate always passes on the first pass; the shared 1/4 gold x5-12 +
//! 1/7 bars (8-way, hardmode 6-way re-pick) + the Next(4)!=0 continue +
//! the 1/4 ores (6-way, 6-16, the 2/3 hardmode re-pick subtracts
//! Next(2)). The shared tails (`:8915-9249`): the 1/4 potion + 1/2
//! lesser + 1/2 bait, then the per-biome tails in if order, then the
//! hardmode corrupt/crimson/hallowed souls.
//!
//! ### The present open trees (`presents.rs`)
//!
//! `OpenPresent` (`:7031-7150`, the Present 1869): the else-if chain —
//! 1/15+hardmode Snow Globe 602 (the draw ALWAYS happens), 1/30 Coal
//! 1922, 1/400 Dog Whistle 1927, 1/150 Red Ryder 1870 + 97 x30-60, five
//! 1/150 rows (1909/1917/1915/1918/1921), 1/300 Toolbox 1923, 1/40
//! Antlers 1907, 1/10 Holly 1908, 1/15 the five-way costume sets
//! (1932-1934/1935-1937/1940-1942/1938/1939), 1/7 the three-way food
//! (1911/1919/1920), 1/8 Eggnog 1912 x1-3, 1/9 Star Anise 1913 x20-40,
//! else the Next(3) block pick (1872/586/591 x20-49).
//! `OpenLegacyPresent` (`:7152-7168`, 599/600/601): ONE Next(14) —
//! 0+hardmode 602, <=7 (incl a pre-hardmode 0) 586 x20-49, else 591.
//! `OpenGoodieBag` (`:9502-9585`, 1774): 1/150 Unlucky Yarn 1810
//! (return), 1/150 Bat Hook 1800 (return), 1/4 Rotten Egg 1809 x10-40
//! (return), 1/10 Next(1846,1851) painting (return), else the 19-way
//! costume switch (case 10 = the Witch set 1766/1775/1776; case 15 =
//! Cat Ears 1824 alone; case 18 = 1851/1852).
//!
//! ### The misc-bag open trees (`misc_bags.rs`)
//!
//! `OpenHerbBag` (`:9367-9441`, 3093): Next(2,5) + 1/3 ++ herbs; each
//! Next(14) over 313/314/315/317/316/318/2358/307/308/309/311/310/312/
//! 2357, stack Next(2,5) + a 1/3-gated Next(1,5) add.
//! `OpenCanofWorms` (`:9443-9455`, 4345): 2002 x5-8 always; the
//! Next(10)<3 gate -> 3191 x1-2; 1/20 Gold Worm 2895.
//! `OpenOyster` (`:9457-9476`, 4410): the 1/4-gated nested chain
//! (1/10 Pink 4414, else 1/3 Black 4413, else White 4412); 4411 always.
//! `OpenChilletEgg` (`:9491-9500`, 6142): Next(2)==0 -> 5666 else 5665.
//! `OpenLockBox` (`:9330-9365`, 3085): the Next(7) pick (1->3317,
//! 2->155, 3->156, 4->157/remix 2623, 5->163, 6->113, 0+default->164),
//! 1/3 Shadow Key 329, 1/8 Ram Rune 5465, the ftw-SHORT-CIRCUIT 1/5
//! 5515, 1/4 Silver Bracer 6156. The dispatch CONSUMES a Golden Key 327
//! first (`ItemSlot.cs:1838`, `ConsumeItem` `:9269-9308`).
//! `OpenShadowLockbox` (`:9310-9328`, 4879): the Next(5) list
//! 274/220/112/218/3019 (remix 112->683), 1/5 Treasure Magnet 5010. The
//! dispatch needs a Shadow Key 329 PRESENT (`:1846`, not consumed).
//!
//! ### The death-heal feed (`npc_death_heals.rs`)
//!
//! The g60-unclaimed death ITEM lane (g60's census = the death dust).
//! `NPCLoot_DropCommonLifeAndMana` (`:79531-79557`): netID 16/121 skip
//! both rolls, 81 skips only the first; the RollLuck(6) gate +
//! lifeMax>1 + damage>0, then the mana-first Next(2) (the heart else-if
//! draws its OWN Next(2)); the second RollLuck(2) mana roll.
//! `NPCLoot_DropHeals` (`:79098-79195`): the preamble + the switch —
//! 305-314/329/330 RollLuck(4), 326 RollLuck(6), 315 always, 341
//! Next(5,11) hearts, 338-340 RollLuck(5), 342 Next(3)!=0, 325/327/344/
//! 345/346 Next(6)+6 hearts, 267 Next(2)+life-gate, 13-15
//! Next(4)+life-gate, 116-119 classic-always/expert-1/5, 139 Next(2).
//! `DoDeathEvents_DropBossPotionsAndHearts` (`:78945-79016`, the
//! `this.boss` tail `:79505-79507`): Next(5,16) potions — 28 default,
//! 188 for 113/35/668, 1134 for 222, 499 for 657 + the >113-and-<222
//! mech band + 636/245/262/370/395/439, 3544 for 398; Next(5)+5 hearts;
//! the EoC 4 / WoF 113 KilledToday latches -> the Badger's Hat 5004 +
//! ResetBadgerHatTime when both up (the dusk reset `Main.cs:65221`).
//! The boss set: the 22 `SetDefaults` rows (4, 35, 50, 113, 125, 126,
//! 127, 134, 222, 245, 262, 266, 370, 395, 396, 397, 398, 439, 636,
//! 657, 664, 668) + the EoW head 13's `DropEoWLoot` dynamic
//! last-segment latch (`:78414-78429`).
//!
//! ## LOOT FINAL (2026-11-21): the full NPCLoot death census — the wiring
//!
//! The death-side `NPC.NPCLoot` (`NPC.cs:78830-78899`) is now FULLY wired
//! in `world_sim/npcs.rs`'s death retain. The C# order is preserved:
//! entry gate (`:78832`) → bestiary/banner credit (`:78839-78847`) →
//! statue gate (`:78849`) → `DoDeathEvents_BeforeLoot` (`:78854`) →
//! `NPCLoot_DropItems` (`:78855`) → `DoDeathEvents` (`:78856`, the
//! Dungeon Spirit summon `:79200`) → `NPCLoot_DropMoney` (`:78897`) →
//! `NPCLoot_DropHeals` (`:78898`). The pre-final port ran coins BEFORE
//! loot (the recorded fold — retired).
//!
//! ### The death census table (every arm, C# lines, port, status)
//!
//! | Arm | C# | Port | Status |
//! | --- | --- | --- | --- |
//! | NPCLoot entry gate (netMode/type/no-surface dungeon/red-hat) | `:78832-78835` | `spawn_loot`'s entry gate (the no-surface `worldSurface<=50` pre-Skeletron nine-kind refusal + the red-hat Dark Caster `localAI[3]==1`; netMode==1/type>=Count dead in SP) | done (f35-final) |
//! | bestiary/banner kill credit | `:78839-78847` | the death retain's `register_kill` + `add_npc_kill_by` (M2-BUFF-8) | done (pre-existing) |
//! | statue loot gate (early-mode set + rarity float) | `:78849-78852` | `tr_core::loot::statue_loot_refused` + the two NPCID tables (`StatueSpawnedDropRarity` `:10445`, `NoEarlymodeLootWhenSpawnedFromStatue` `:10446`); refuses the WHOLE NPCLoot (None) | done (f35-final) |
//! | DoDeathEvents_BeforeLoot (Twins value/boss zero) | `:78916-78923` | `spawn_loot`/`spawn_coins`/`spawn_heals`' `twin_first_death` (the first eye zeroes value + clears boss; only the last eye loots) | done (f35-final) |
//! | NPCLoot_DropItems (the rule engine) | `:78925-78937` | `spawn_loot` → `tr_core::loot::try_dropping` over `loot_db_gen` (1599 registrations); returns `Some(resets_money)` | done (S1-E-14a + f35-final live feeds) |
//! | DoDeathEvents_SummonDungeonSpirit | `:79064-79078` | `summon_dungeon_spirit` (the post-Plantera hardmode dungeon RollLuck(13) + dungeon-wall spawn 288) | done (f35-final) |
//! | DoDeathEvents_AdvanceSlimeRain | `:79080-79095` | the death retain's slime-rain counter (BLUE slime only, `Main.slimeRainNPC[1]` `Main.cs:8864`) | done (f35-final fix) |
//! | DoDeathEvents_DropBossPotionsAndHearts | `:78945-79016` | `npc_death_heals::drop_boss_potions_and_hearts` (the `this.boss` tail) | done (f35 widened) |
//! | DoDeathEvents_CelebrateBossDeath | `:79018-79034` | `spawn_heals`' boss arm (the BossOrEvent announcement; NPCDamageTracker.BossKilled unported, no RNG) | done (f35-final) |
//! | NPCLoot_DropMoney (the coin math) | `:79619-79750` | `spawn_coins` → `tr_core::loot::drop_money` (midas `has_buff(72)` + blood_moon live; the `resets_money`/`twin_first_death` value zero) | done (f35-final, the g4c stand-ins wired) |
//! | NPCLoot_DropHeals / DropCommonLifeAndMana | `:79098-79195`/`:79531-79557` | `npc_death_heals::drop_heals`/`drop_common_life_and_mana` | done (f35 widened) |
//! | the boss/tower/event downed flags (DoDeathEvents switch) | `:79205-79504` | the death retain's `match npc.kind` (Golem/Fishron/Deerclops/towers/MoonLord/Cultist/EoW-BoC/GolemHead + KingSlime/EoC/QueenBee/mechs/Plantera/Skeletron/WoF) | done (pre-existing; the seasonal-boss flags stay unowned — no live field, the wld glue writes false) |
//! | the Skyblock recovery arms (DropAltar/DropDungeonStuff/DropHellforge/DropLihzahrdStuff/DropTempleTraps) | `:79559-79617` | — | OUT (the `WorldGen.Skyblock` latches have no port state — the Skyblock worldgen pass is unported, the gates are constants-off) |
//!
//! ### The rule-gate census fixes (f35-final)
//!
//! - `SoulOfWhateverConditionCanDrop` (`Conditions.cs:1579-1612`) is the
//!   FULL gate, not a bare depth flag: the boss refusal, the
//!   `DontDropDungeonKeysOrSouls` fold ({23}), the six exempt types
//!   (1/13/14/15/121/535), hardmode + lifeMax>1 + !friendly + value>=1,
//!   the classic depth band; the REMIX world drops ONLY the depth term.
//!   The pre-final port returned `flags.soul_depth_ok` alone (nothing set
//!   it — the SoulOfLight 520 / SoulOfNight 521 globals were dead).
//! - `DownedPlantera` (`Conditions.cs:322`) reads `NPC.downedPlantBoss` —
//!   the SAME flag `FirstTimeKillingPlantera` (`:358`) inverts. The phantom
//!   `LootFlags.downed_plantera` second flag is retired (nothing set it —
//!   the 460/463/466/467/468 post-Plantera rows stayed dead).
//!
//! ### The live LootFlags/LootNpc feeds (f35-final)
//!
//! `spawn_loot` now feeds every gate the registrations read: the biome-key
//! zones (jungle/hallow/desert-minus-beach), the soul NPC-depth band,
//! LivingFlames' underworld band, the yoyo depth bands, the pirate-map
//! surface/ocean band, blood_moon/windy/moon-waves/downed_boss2/3/
//! downed_plant_boss, the Skyblock sickle `HasItemInInventoryOrOpenVoidBag`
//! (1786) scan, the Mechdusa remix+ftw gate + four-part AnyNPCs scan,
//! MissingTwin's live sibling scan, the 23-kind boss table + EoW latch on
//! `is_boss` (was KingSlime-only), `HasPlayerTarget`'s [0,255) read,
//! damage/defense/friendly live (the holiday caps 500/40 classic).
//!
//! ### The coin math (f35-final, the g4c stand-ins wired)
//!
//! `spawn_coins`' two false stand-ins are LIVE (g4 `a9369ec38` cross-note
//! claimed them for f35): `midas = npc.buffs.has_buff(72)` (the
//! `UpdateNPC_BuffFlags` latch `NPC.cs:92017-92020`, reset `:92165`),
//! `blood_moon = app.world.blood_moon`. The `drop_money` math itself
//! (`tr_core::loot::drop_money`, `NPC.cs:79619-79750`) was already
//! faithful: the |luck| double-roll keeping best/worst, the midas
//! Next(30,51) bump, the ±20-75% spread, the six bonus chains
//! (1/2,1/4,1/8,1/16,1/32,1/64), the blood-moon Next(101) bump, the
//! extraValue add, the platinum/gold/silver/copper split with the >50
//! shrink rolls (copper's second roll Next(4)+1) + the copper floor.
//! The `resets_money` (boss bag / master pet CommonCode zero) and the
//! `twin_first_death` zero feed the value before DropMoney reads it.
//!
//! ## LOOT-FAMILY DECLARATION (W4-f35 FINAL)
//!
//! The NPC death-loot family is COMPLETE for the port's single-player
//! classic world: every `NPC.NPCLoot` arm is either ported+wired or
//! recorded OUT (the Skyblock recovery arms — the worldgen pass is
//! unported; the seasonal-boss downed flags — no live field, the wld glue
//! writes false; the expert/master difficulty — the e25 constants-off
//! seam). The death order is the C# sequence (loot → coins → heals), the
//! coin math's live inputs are wired, the rule gates are the full C#
//! bodies, and the statue/entry/Twins refusals skip the whole NPCLoot.
//! The OPEN side (the 58-kind OpenableBag set) landed in the widened
//! scope. Witnesses: 11 f35-final (the soul-gate refusal table, the
//! DownedPlantera complement, the statue table/draw-shape, the midas/
//! blood-moon stream deltas, the statue early-mode refusal, the
//! loot-coins-heals order twin, the Twins first-death zero, the entry-gate
//! table, the boss-death announcement, the Dungeon Spirit summon, the
//! blue-slime-only rain counter) + the 45+ widened-scope family replays —
//! all WRITTEN NOT RUN (code-only mode, no cargo/rustc/rustfmt/tests/
//! dotnet/graphify invoked).
