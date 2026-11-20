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
