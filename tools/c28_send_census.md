# W4-c28 — the C# NetMessage.SendData trigger census (generated)

Total literal call sites: **918** across 68 files (decompiled/Terraria).
Regenerate: `python tools/c28_send_census.py` (read-only over the C#).

| id | MessageID name | sites | files (site counts) |
| --- | --- | --- | --- |
| 1 | Hello | 1 | Netplay.cs (1) |
| 2 | Kick | 13 | MessageBuffer.cs (10), Main.cs (2), NetMessage.cs (1) |
| 3 | PlayerInfo | 2 | MessageBuffer.cs (2) |
| 4 | SyncPlayer | 30 | Player.cs (15), UI/ItemSlot.cs (7), Main.cs (5), MessageBuffer.cs (2), NetMessage.cs (1) |
| 5 | SyncEquipment | 13 | MessageBuffer.cs (4), NetMessage.cs (4), Main.cs (3), GameContent/QuickStacking.cs (2) |
| 6 | RequestWorldData | 1 | MessageBuffer.cs (1) |
| 7 | WorldData | 58 | Main.cs (20), MessageBuffer.cs (13), NPC.cs (9), GameContent/Events/BirthdayParty.cs (3), Projectile.cs (3), WorldGen.cs (3), GameContent/Events/DD2Event.cs (2), GameContent/Events/LanternNight.cs (2), GameContent/Events/Sandstorm.cs (1), GameContent/TreeTopsInfo.cs (1), WorldItem.cs (1) |
| 8 | SpawnTileData | 1 | Netplay.cs (1) |
| 9 | StatusTextSize | 2 | MessageBuffer.cs (1), RemoteClient.cs (1) |
| 10 | TileSection | 2 | NetMessage.cs (2) |
| 12 | PlayerSpawn | 3 | MessageBuffer.cs (2), Player.cs (1) |
| 13 | PlayerControls | 40 | Player.cs (21), Main.cs (6), GameContent/DoorOpeningHelper.cs (2), GameContent/PlayerSittingHelper.cs (2), GameContent/PlayerSleepingHelper.cs (2), Mount.cs (2), Collision.cs (1), GameContent/PortalHelper.cs (1), MessageBuffer.cs (1), NetMessage.cs (1), Projectile.cs (1) |
| 14 | PlayerActive | 2 | NetMessage.cs (2) |
| 16 | PlayerLifeMana | 6 | Main.cs (2), MessageBuffer.cs (2), NetMessage.cs (1), Player.cs (1) |
| 17 | TileManipulation | 162 | Player.cs (59), WorldGen.cs (45), Projectile.cs (19), Wiring.cs (10), DelegateMethods.cs (5), GameContent/MinecartDiggerHelper.cs (3), Tile.cs (3), GameContent/Tile_Entities/TEDeadCellsDisplayJar.cs (2), GameContent/Tile_Entities/TEFoodPlatter.cs (2), GameContent/Tile_Entities/TEItemFrame.cs (2), GameContent/Tile_Entities/TEWeaponsRack.cs (2), Liquid.cs (2), MessageBuffer.cs (2), NPC.cs (2), Collision.cs (1), GameContent/Tile_Entities/TETeleportationPylon.cs (1), ObjectData/TileObjectData.cs (1), TileObject.cs (1) |
| 19 | ToggleDoorState | 25 | NPC.cs (9), GameContent/DoorOpeningHelper.cs (5), Player.cs (5), Wiring.cs (5), MessageBuffer.cs (1) |
| 20 | AreaTileChange | 1 | NetMessage.cs (1) |
| 21 | SyncItem | 4 | Item.cs (2), MessageBuffer.cs (1), WorldItem.cs (1) |
| 22 | ItemOwner | 3 | MessageBuffer.cs (2), WorldItem.cs (1) |
| 23 | SyncNPC | 76 | NPC.cs (54), GameContent/Events/DD2Event.cs (6), WorldGen.cs (6), MessageBuffer.cs (4), NetMessage.cs (2), GameContent/CoinLossRevengeSystem.cs (1), GameContent/Events/MysticLogFairiesEvent.cs (1), GameContent/PortalHelper.cs (1), Projectile.cs (1) |
| 27 | SyncProjectile | 21 | Projectile.cs (10), NPC.cs (6), MessageBuffer.cs (2), Main.cs (1), NetMessage.cs (1), Player.cs (1) |
| 28 | DamageNPC | 13 | NPC.cs (12), MessageBuffer.cs (1) |
| 29 | KillProjectile | 4 | MessageBuffer.cs (2), Projectile.cs (2) |
| 30 | TogglePVP | 3 | Main.cs (1), MessageBuffer.cs (1), NetMessage.cs (1) |
| 31 | RequestChestOpen | 2 | Player.cs (2) |
| 32 | SyncChestItem | 29 | UI/ChestUI.cs (14), UI/ItemSlot.cs (5), Projectile.cs (3), GameContent/CraftingRequests.cs (1), GameContent/Events/DD2Event.cs (1), GameContent/QuickStacking.cs (1), MessageBuffer.cs (1), NetMessage.cs (1), Player.cs (1), UI/ItemSorting.cs (1) |
| 33 | SyncPlayerChest | 6 | Main.cs (3), Player.cs (2), MessageBuffer.cs (1) |
| 34 | ChestUpdates | 19 | MessageBuffer.cs (9), Chest.cs (3), Player.cs (3), WorldGen.cs (3), NPC.cs (1) |
| 35 | PlayerHeal | 2 | MessageBuffer.cs (1), Player.cs (1) |
| 36 | SyncPlayerZone | 3 | Main.cs (2), MessageBuffer.cs (1) |
| 37 | RequestPassword | 1 | MessageBuffer.cs (1) |
| 38 | SendPassword | 2 | Main.cs (1), MessageBuffer.cs (1) |
| 39 | ReleaseItemOwnership | 2 | WorldItem.cs (2) |
| 40 | SyncTalkNPC | 4 | Main.cs (2), MessageBuffer.cs (1), NPC.cs (1) |
| 41 | ItemRotationAndAnimation | 4 | Player.cs (3), MessageBuffer.cs (1) |
| 42 | Unknown42 | 3 | Main.cs (1), MessageBuffer.cs (1), NetMessage.cs (1) |
| 43 | ManaEffect | 2 | MessageBuffer.cs (1), Player.cs (1) |
| 45 | TeamChange | 3 | MessageBuffer.cs (2), NetMessage.cs (1) |
| 46 | OpenSignRequest | 1 | Player.cs (1) |
| 47 | OpenSignResponse | 4 | MessageBuffer.cs (2), Main.cs (1), Projectile.cs (1) |
| 48 | LiquidUpdate | 3 | NetMessage.cs (2), MessageBuffer.cs (1) |
| 49 | InitialSpawn | 1 | MessageBuffer.cs (1) |
| 50 | PlayerBuffs | 4 | MessageBuffer.cs (2), Main.cs (1), NetMessage.cs (1) |
| 51 | MiscDataSync | 8 | Main.cs (2), Player.cs (2), GameContent/ConditionalDialogue.cs (1), GameContent/NPCInteractions.cs (1), MessageBuffer.cs (1), NPC.cs (1) |
| 52 | LockAndUnlock | 8 | Player.cs (5), MessageBuffer.cs (3) |
| 53 | AddNPCBuff | 1 | NPC.cs (1) |
| 54 | NPCBuffs | 6 | NPC.cs (4), MessageBuffer.cs (2) |
| 55 | AddPlayerBuffPvP | 2 | MessageBuffer.cs (1), Player.cs (1) |
| 56 | UniqueTownNPCInfoSyncRequest | 4 | NPC.cs (3), MessageBuffer.cs (1) |
| 57 | Unknown57 | 2 | MessageBuffer.cs (1), WorldGen.cs (1) |
| 58 | InstrumentSound | 6 | Player.cs (5), MessageBuffer.cs (1) |
| 59 | HitSwitch | 18 | Player.cs (5), Collision.cs (4), GameContent/Tile_Entities/TELogicSensor.cs (2), MessageBuffer.cs (2), WorldGen.cs (2), GameContent/Golf/GolfHelper.cs (1), GameContent/PressurePlateHelper.cs (1), Minecart.cs (1) |
| 60 | Unknown60 | 5 | NPC.cs (2), WorldGen.cs (2), NetMessage.cs (1) |
| 61 | SpawnBossUseLicenseStartEvent | 27 | Player.cs (24), NPC.cs (2), Main.cs (1) |
| 62 | SyncDodge | 5 | Player.cs (4), MessageBuffer.cs (1) |
| 63 | SyncTilePaintOrCoating | 3 | WorldGen.cs (2), MessageBuffer.cs (1) |
| 64 | SyncWallPaintOrCoating | 3 | WorldGen.cs (2), MessageBuffer.cs (1) |
| 65 | TeleportEntity | 18 | Player.cs (12), MessageBuffer.cs (3), GameContent/TeleportPylonsSystem.cs (1), NPC.cs (1), Wiring.cs (1) |
| 66 | Unknown66 | 2 | MessageBuffer.cs (1), Projectile.cs (1) |
| 68 | Unknown68 | 1 | MessageBuffer.cs (1) |
| 69 | ChestName | 2 | MessageBuffer.cs (2) |
| 70 | BugCatching | 1 | NPC.cs (1) |
| 71 | BugReleasing | 1 | NPC.cs (1) |
| 72 | TravelMerchantItems | 1 | NetMessage.cs (1) |
| 73 | RequestTeleportationByServer | 5 | Player.cs (5) |
| 74 | AnglerQuest | 2 | NetMessage.cs (2) |
| 75 | AnglerQuestFinished | 1 | Main.cs (1) |
| 76 | QuestsCountSync | 1 | MessageBuffer.cs (1) |
| 77 | TemporaryAnimation | 1 | NetMessage.cs (1) |
| 78 | InvasionProgressReport | 16 | GameContent/Events/DD2Event.cs (6), MessageBuffer.cs (4), Main.cs (3), NPC.cs (3) |
| 79 | PlaceObject | 1 | NetMessage.cs (1) |
| 80 | SyncPlayerChestIndex | 3 | MessageBuffer.cs (2), NetMessage.cs (1) |
| 81 | CombatTextInt | 1 | NPC.cs (1) |
| 84 | PlayerStealth | 7 | Player.cs (6), MessageBuffer.cs (1) |
| 85 | QuickStackChests | 2 | GameContent/QuickStacking.cs (2) |
| 86 | TileEntitySharing | 9 | GameContent/Tile_Entities/TETrainingDummy.cs (2), GameContent/Tile_Entities/TEWeaponsRack.cs (2), DataStructures/TileEntityType.cs (1), GameContent/Tile_Entities/TEDeadCellsDisplayJar.cs (1), GameContent/Tile_Entities/TEFoodPlatter.cs (1), GameContent/Tile_Entities/TEItemFrame.cs (1), GameContent/Tile_Entities/TELeashedEntityAnchor.cs (1) |
| 87 | TileEntityPlacement | 10 | GameContent/Tile_Entities/TEDeadCellsDisplayJar.cs (1), GameContent/Tile_Entities/TEDisplayDoll.cs (1), GameContent/Tile_Entities/TEFoodPlatter.cs (1), GameContent/Tile_Entities/TEHatRack.cs (1), GameContent/Tile_Entities/TEItemFrame.cs (1), GameContent/Tile_Entities/TELeashedEntityAnchor.cs (1), GameContent/Tile_Entities/TELogicSensor.cs (1), GameContent/Tile_Entities/TETeleportationPylon.cs (1), GameContent/Tile_Entities/TETrainingDummy.cs (1), GameContent/Tile_Entities/TEWeaponsRack.cs (1) |
| 88 | ItemTweaker | 6 | GameContent/ItemDropRules/CommonCode.cs (6) |
| 89 | ItemFrameTryPlacing | 1 | GameContent/Tile_Entities/TEItemFrame.cs (1) |
| 90 | SpawnInstancedItem | 1 | WorldItem.cs (1) |
| 91 | SyncEmoteBubble | 2 | GameContent/UI/EmoteBubble.cs (2) |
| 92 | SyncExtraValue | 3 | GameContent/CoinLossRevengeSystem.cs (1), MessageBuffer.cs (1), WorldItem.cs (1) |
| 94 | DevCommands | 10 | MessageBuffer.cs (3), Testing/ChatCommands/ToolkitDebugCommands.cs (3), Testing/DebugOptions.cs (3), Testing/QuickLoad.cs (1) |
| 95 | MurderSomeoneElsesPortal | 1 | GameContent/PortalHelper.cs (1) |
| 96 | TeleportPlayerThroughPortal | 2 | GameContent/PortalHelper.cs (1), MessageBuffer.cs (1) |
| 97 | AchievementMessageNPCKilled | 1 | GameContent/Achievements/AchievementsHelper.cs (1) |
| 98 | AchievementMessageEventHappened | 1 | GameContent/Achievements/AchievementsHelper.cs (1) |
| 99 | MinionRestTargetUpdate | 2 | Main.cs (1), MessageBuffer.cs (1) |
| 100 | TeleportNPCThroughPortal | 1 | GameContent/PortalHelper.cs (1) |
| 101 | UpdateTowerShieldStrengths | 3 | MessageBuffer.cs (1), Projectile.cs (1), WorldGen.cs (1) |
| 102 | NebulaLevelupRequest | 2 | MessageBuffer.cs (1), Player.cs (1) |
| 103 | MoonlordHorror | 2 | MessageBuffer.cs (1), WorldGen.cs (1) |
| 105 | GemLockToggle | 2 | Player.cs (2) |
| 106 | PoofOfSmoke | 7 | Wiring.cs (4), NPC.cs (2), Projectile.cs (1) |
| 107 | SmartTextMessage | 2 | Wiring.cs (2) |
| 108 | WiredCannonShot | 1 | WorldGen.cs (1) |
| 109 | MassWireOperation | 2 | Player.cs (1), Projectile.cs (1) |
| 110 | MassWireOperationPay | 2 | Wiring.cs (2) |
| 111 | ToggleParty | 1 | GameContent/Events/BirthdayParty.cs (1) |
| 112 | SpecialFX | 3 | WorldGen.cs (2), NPC.cs (1) |
| 113 | CrystalInvasionStart | 1 | GameContent/Events/DD2Event.cs (1) |
| 114 | CrystalInvasionWipeAllTheThingsss | 1 | GameContent/Events/DD2Event.cs (1) |
| 115 | MinionAttackTargetUpdate | 2 | Main.cs (1), MessageBuffer.cs (1) |
| 116 | CrystalInvasionSendWaitTime | 1 | GameContent/Events/DD2Event.cs (1) |
| 117 | PlayerHurtV2 | 1 | NetMessage.cs (1) |
| 118 | PlayerDeathV2 | 1 | NetMessage.cs (1) |
| 120 | Emoji | 3 | Chat/Commands/EmojiCommand.cs (1), Chat/Commands/RockPaperScissorsCommand.cs (1), GameContent/UI/EmoteBubble.cs (1) |
| 121 | TEDisplayDollDataSync | 9 | UI/ItemSlot.cs (6), GameContent/Tile_Entities/TEDisplayDoll.cs (3) |
| 122 | RequestTileEntityInteraction | 3 | DataStructures/TileEntity.cs (2), Main.cs (1) |
| 123 | WeaponsRackTryPlacing | 1 | GameContent/Tile_Entities/TEWeaponsRack.cs (1) |
| 124 | TEHatRackItemSync | 6 | UI/ItemSlot.cs (4), GameContent/Tile_Entities/TEHatRack.cs (2) |
| 125 | SyncTilePicking | 2 | MessageBuffer.cs (1), Player.cs (1) |
| 126 | SyncRevengeMarker | 1 | NetMessage.cs (1) |
| 127 | RemoveRevengeMarker | 3 | GameContent/CoinLossRevengeSystem.cs (3) |
| 128 | LandGolfBallInCup | 2 | GameContent/Golf/GolfHelper.cs (1), MessageBuffer.cs (1) |
| 129 | FinishedConnectingToServer | 1 | MessageBuffer.cs (1) |
| 130 | FishOutNPC | 1 | Player.cs (1) |
| 131 | TamperWithNPC | 1 | WorldGen.cs (1) |
| 132 | PlayLegacySound | 1 | NetMessage.cs (1) |
| 133 | FoodPlatterTryPlacing | 1 | GameContent/Tile_Entities/TEFoodPlatter.cs (1) |
| 134 | UpdatePlayerLuckFactors | 2 | MessageBuffer.cs (1), Player.cs (1) |
| 135 | DeadPlayer | 1 | NetMessage.cs (1) |
| 136 | SyncCavernMonsterType | 1 | MessageBuffer.cs (1) |
| 137 | RequestNPCBuffRemoval | 1 | NPC.cs (1) |
| 138 | ClientSyncedInventory | 1 | Main.cs (1) |
| 139 | SetCountsAsHostForGameplay | 1 | MessageBuffer.cs (1) |
| 140 | SetMiscEventValues | 4 | GameContent/Events/CreditsRollEvent.cs (2), NPC.cs (2) |
| 141 | RequestLucyPopup | 2 | GameContent/LucyAxeMessage.cs (1), MessageBuffer.cs (1) |
| 142 | SyncProjectileTrackers | 3 | Main.cs (1), MessageBuffer.cs (1), NetMessage.cs (1) |
| 143 | CrystalInvasionRequestedToSkipWaitTime | 1 | GameContent/Events/DD2Event.cs (1) |
| 144 | RequestQuestEffect | 1 | Main.cs (1) |
| 146 | ShimmerActions | 2 | WorldItem.cs (2) |
| 147 | SyncLoadout | 3 | MessageBuffer.cs (1), NetMessage.cs (1), Player.cs (1) |
| 149 | DeadCellsDisplayJarTryPlacing | 1 | GameContent/Tile_Entities/TEDeadCellsDisplayJar.cs (1) |
| 150 | SpectatePlayer | 3 | Player.cs (2), MessageBuffer.cs (1) |
| 151 | SyncItemDespawn | 3 | Item.cs (1), MessageBuffer.cs (1), WorldItem.cs (1) |
| 152 | ItemUseSound | 2 | MessageBuffer.cs (1), Player.cs (1) |
| 153 | NPCDebuffDamage | 2 | MessageBuffer.cs (1), NPC.cs (1) |
| 154 | Ping | 2 | MessageBuffer.cs (1), Net/Ping.cs (1) |
| 155 | SyncChestSize | 1 | NetMessage.cs (1) |
| 156 | TELeashedEntityAnchorPlaceItem | 2 | GameContent/Tile_Entities/TEKiteAnchor.cs (1), GameContent/Tile_Entities/TELeashedEntityAnchorWithItem.cs (1) |
| 157 | TeamChangeFromUI | 1 | Main.cs (1) |
| 158 | ExtraSpawnSectionLoaded | 1 | MessageBuffer.cs (1) |
| 159 | RequestSection | 2 | Player.cs (1), WorldItem.cs (1) |
| 160 | ItemPosition | 1 | WorldItem.cs (1) |
| 161 | HostToken | 1 | MessageBuffer.cs (1) |
| 162 | DamageNPCAck | 1 | MessageBuffer.cs (1) |
| relay | ? | 8 | MessageBuffer.cs (8) |

## NetMessage.Send* wrappers (ids they emit) + call-site counts

- `SendAnglerQuest` → ids 74 — 2 call sites
- `SendChestContentsTo` → ids 32,155 — 2 call sites
- `SendCoinLossRevengeMarker` → ids 126 — 2 call sites
- `SendObjectPlacement` → ids 79 — 4 call sites
- `SendPlayerDeath` → ids 118 — 2 call sites
- `SendPlayerHurt` → ids 117 — 5 call sites
- `SendSection` → ids 10,23 — 6 call sites
- `SendTemporaryAnimation` → ids 77 — 4 call sites
- `SendTileSquare` → ids 20 — 231 call sites
- `SendTravelShop` → ids 72 — 2 call sites

## netUpdate / netSpam / spawnNeedsSyncing write-site census

### `netImportant` — 196 write sites

| file | sites | methods (site counts) |
| --- | --- | --- |
| Projectile.cs | 195 | SetDefaults (195) |
| NewProjectileModifiers.cs | 1 | RainHazard (1) |

### `netSpam` — 9 write sites

| file | sites | methods (site counts) |
| --- | --- | --- |
| NPC.cs | 5 | UpdateNetworkCode (4), NetUpdateIgnoreSpamLimit (1) |
| Projectile.cs | 4 | Update (3), SetDefaults (1) |

### `netUpdate` — 940 write sites

| file | sites | methods (site counts) |
| --- | --- | --- |
| NPC.cs | 584 | AI (243), AI_007_TownEntities (69), AI_003_Fighters (38), AI_107_ImprovedWalkers (19), AI_121_QueenSlime (17), AI_069_DukeFishron (17), Vector2 (16), AI_084_LunaticCultist (12), AI_112_FairyCritter (11), AI_081_TrueEyeOfCthulhu (9), AI_005_EaterOfSouls (9), AI_077_MoonLordCore (7), … +42 more |
| Projectile.cs | 345 | AI (65), Damage_PVE_Inner (29), AI_067_FreakingPirates (17), AI_062 (16), AI_156_Think (14), AI_075 (14), HandleMovement (13), AI_019_Spears_Old (13), AI_003_Boomerang (10), AI_015_Flails (9), AI_120_StardustGuardian (9), AI_001 (8), … +57 more |
| Player.cs | 4 | TryUsingFoxsparksAbility (1), FreeUpPetsAndMinions (1), ItemCheck_CheckFishingBobber_PullBobber (1), CommandForbiddenStorm (1) |
| WorldGen.cs | 4 | ShakeTree (4) |
| GameContent/CoinLossRevengeSystem.cs | 1 | SpawnEnemy (1) |
| GameContent/UI/EmoteBubble.cs | 1 | CheckForNPCsToReactToEmoteBubble (1) |
| NetMessage.cs | 1 | SendData (1) |

### `netUpdate2` — 7 write sites

| file | sites | methods (site counts) |
| --- | --- | --- |
| Player.cs | 3 | ItemCheck_Shoot (1), Vector2 (1), ItemCheck_PullFishingBobbers (1) |
| Projectile.cs | 3 | Update (2), SetDefaults (1) |
| GameContent/Golf/GolfState.cs | 1 | ResetGolfBall (1) |

### `netUpdatePendingFullSpamCooldown` — 3 write sites

| file | sites | methods (site counts) |
| --- | --- | --- |
| NPC.cs | 2 | SetDefaults (1), NetUpdateLowPriority (1) |
| NetMessage.cs | 1 | SendData (1) |

### `netUpdatePendingSpamCooldown` — 3 write sites

| file | sites | methods (site counts) |
| --- | --- | --- |
| NPC.cs | 2 | SetDefaults (1), UpdateNetworkCode (1) |
| NetMessage.cs | 1 | SendData (1) |

### `spawnNeedsSyncing` — 2 write sites

| file | sites | methods (site counts) |
| --- | --- | --- |
| NetMessage.cs | 1 | SendData (1) |
| NPC.cs | 1 | NewNPC (1) |

## The netSpam priority-helper call sites

| helper | sites | lines (NPC.cs unless noted) |
| --- | --- | --- |
| `NetUpdateIgnoreSpamLimit` | 28 | NPC.cs:20215, NPC.cs:20281, NPC.cs:20522, NPC.cs:20572, NPC.cs:20588, NPC.cs:20687, NPC.cs:20720, NPC.cs:23980, NPC.cs:32481, NPC.cs:32507, NPC.cs:32525, NPC.cs:44047, NPC.cs:44081, NPC.cs:44089, NPC.cs:44124, NPC.cs:44137, NPC.cs:44154, NPC.cs:44164, NPC.cs:44175, NPC.cs:44186, NPC.cs:44199, NPC.cs:44215, NPC.cs:44231, NPC.cs:44281, NPC.cs:44319, NPC.cs:44339, NPC.cs:44347, NPC.cs:44402 |
| `NetUpdateLowPriority` | 4 | NPC.cs:25641, NPC.cs:81238, NPC.cs:92503, NPC.cs:94913 |
