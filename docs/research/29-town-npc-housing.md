# 29 — Town NPCs: housing, happiness, pylons, relocation

> Wave 3 · researched 2026-11-12 · sources: decompiled (Terraria 1.4.5.8) + crates + ROADMAP/PORT_MAP + research/05, research/24

## Scope

Housing, happiness/mood, the teleport-pylon system and the town-NPC relocation
(home finding / recall) paths of the town-NPC layer. The files that carry them:

| C# file | lines | role |
| --- | --- | --- |
| `Terraria/WorldGen.cs` | ~87 k | the room check (`StartRoomCheck`/`CheckRoom`), the furniture needs (`RoomNeeds`), the room score (`ScoreRoom`), the home search (`QuickFindHome`), the move-in/relocation helpers (`MoveTownNPC`, `moveRoom`, `FindNPCLookingForHomeThatCanMoveIn`, `IsRoomConsideredAlreadyOccupied`) |
| `Terraria/GameContent/TownRoomManager.cs` | 167 | the room registry (one room per town-NPC type), the household status, the "can live together" rule |
| `Terraria/GameContent/ShopHelper.cs` | 372 | the happiness/mood system: the price multiplier + the happiness report |
| `Terraria/GameContent/Personalities/*.cs` | 21 files / ~950 | the per-NPC personality traits (biome + neighbour likes/dislikes) |
| `Terraria/GameContent/TeleportPylonsSystem.cs` | 394 | the pylon registry, the teleport gate ladder, the "enough NPCs" rule |
| `Terraria/GameContent/Tile_Entities/TETeleportationPylon.cs` | 179 | the pylon tile entity, the style→type map, the placement rules |
| `Terraria/GameContent/TeleportPylonType.cs` / `TeleportPylonInfo.cs` | 17 / 20 | the 11 pylon kinds, the network payload |
| `Terraria/GameContent/TownNPCProfiles.cs` + `ITownNPCProfile.cs` | 217 / 15 | the per-type profile (name variant, texture, head) |
| `Terraria/GameContent/NPCInteractions.cs` | ~530 | the dialogue menu: `ReportHappiness` (289), `RequestHome` (314), the pet chatter (341-353) |
| `Terraria/ID/HousingCategoryID.cs` | 8 | `CommonNPCs = 0`, `PetNPCs = 1` — the per-NPC housing flag |
| `Terraria/Enums/TownNPCRoomCheckFailureReason.cs` | 12 | the seven room-check failure reasons |

Rust side: `crates/tr-core/src/housing.rs` (the room check), `crates/tr-app/src/town.rs`
(499), `crates/tr-app/src/happiness.rs` (705), `crates/tr-core/src/pylons.rs` (528),
`crates/tr-app/src/ui/housing.rs` (468), `crates/tr-core/src/wld/sections/town_rooms.rs` (241).
`crates/tr-app/src/town_arrival.rs` (448) is the town-NPC *arrival* gates — owned by
research/05 and research/24, not by this module.

**Scope corrections (verified against the 1.4.5.8 source, not against the brief).**
The brief's framing does not match the shipped game:

1. There is **no happiness point scale** in 1.4.5.8 — no 250-point scale, no
   `NPC.GetGoodTown`/`GetBadTown`/`GetGoodWorld` happiness API, no "happiness item
   list". A repo-wide grep for `appiness` hits only `ShopHelper.cs` (26),
   `Main.cs` (13, the shop-HUD offsets), `NPCInteractions.cs` (5), `ShoppingSettings.cs` (2)
   and the localisation/achievement files. The single happiness-like system is
   the **mood**: a *price multiplier* (0.75 … 1.5) plus a localised *report string*.
   The `250` that does exist is the room-score reject threshold
   (`WorldGen.cs:5890`, `num2 <= -250`).
2. There is **no `NPC.house` field** in 1.4.5.8. The housing state is
   `NPC.homeless` (6543) + `NPC.homeTileX/homeTileY` (6551/6553) + the legacy pair
   `oldHomeless/oldHomeTileX/oldHomeTileY` (6557-6561) + the room registry
   (`TownRoomManager`). There is also **no "happiness requirement" for the pylons**:
   the pylon gate is *2 housed town NPCs* (`TeleportPylonsSystem.cs:304-311`).
3. The "housing flags per NPC type" is a single int, `NPC.housingCategory`
   (`NPC.cs:6555`), `0 = CommonNPCs` / `1 = PetNPCs` (`HousingCategoryID.cs`); it is
   used **only** by `TownRoomManager.CanNPCsLiveWithEachOther` (152-166).

## Key entities

### C# (source of truth)

- `Terraria/WorldGen.cs::StartRoomCheck` (6041-6111) — the room-check entry:
  the 10-tile world-edge rule (6060), the solid-start cut (6067), the 8-neighbour
  flood fill (6085-6094), the `numRoomTiles < 60` "room too small" rule (6100),
  and `roomCheckFailureReason` (the 7-reason enum).
- `Terraria/WorldGen.cs::CheckRoom` (6113-6237) — one flood-fill step: the
  `maxRoomSize`/`maxRoomTiles` window + the span check ("room too big", 6135/6158),
  the blocking tiles (solid wall, the open gates 11/389/386, 6169-6188), the stinkbugs
  630/631 (6189-6198) and the ±2 house-wall scan (`Main.wallHouse` /
  `TileID.Sets.HousingWalls`, 6200-6235 → `TooManyUnsafeWalls` / `HoleInWallIsTooBig`).
- `Terraria/WorldGen.cs::RoomNeeds` (5679-5730) — the furniture rule: a chair, a
  table, a torch and a door, each via `TileID.Sets.RoomNeeds.CountsAs*Types`.
- `Terraria/WorldGen.cs::ScoreRoom` (5858-5992) — the room **score**: base 50,
  the evil penalty (`TileScanGroup.TotalGoodEvil`, 5880), the `-250` reject (5890),
  the per-spot penalties (solid −5, non-solid +5, door −20, chest −30 each,
  the home spot −15, 5940-5955), the standing-space test (5961-5989) and the
  `bestX/bestY` pick (5983-5988).
- `Terraria/WorldGen.cs::QuickFindHome` (5732-5794) — the three-pass home search
  (the exact tile, the ±1 ring, the ±10 ring with stride 2), `RoomNeeds` + `ScoreRoom`,
  the occupancy check and the home assignment (`homeTileX/homeTileY`, `homeless = false`,
  the achievement 8).
- `Terraria/WorldGen.cs::FindNPCLookingForHomeThatCanMoveIn` (5617-5627) — the
  homeless-NPC lookup; gated on `lookForHomeTimeout == 0`.
- `Terraria/WorldGen.cs::IsRoomConsideredAlreadyOccupied` (5644-5657) and
  `::IsRoomConsideredOccupiedForNPCIndex` (5796-5815) — the "one tenant per house
  spot" rule, via `TownManager.CanNPCsLiveWithEachOther`.
- `Terraria/WorldGen.cs::MoveTownNPC` (4685) / `::moveRoom` (4843) — the mid-game
  relocation (a house is broken → the NPC is moved), the tail of `MoveTownNPC`
  (4848-4854) is the home assignment the Rust cites.
- `Terraria/GameContent/TownRoomManager.cs` — `HasRoomQuick` (40), `HasRoom` (44),
  `SetRoom` (57), `KickOut` (75/80), `Save`/`Load` (91/100), `GetHouseholdStatus`
  (139, the byte 0/1/2), `CanNPCsLiveWithEachOther` (152/158).
- `Terraria/GameContent/ShopHelper.cs::ProcessMood` (95-174) — the mood: the
  `loveStruck` ×0.9 (99), the exclusion block (103: remix world, 368, 453, the town
  pets, `IsNotReallyTownNPC` 37/368/453), the homelessness sentinel (107), the
  120-tile far-from-home sentinel (111), the evil-biome sentinel (115), the
  Princess 663 branch (122-131), the crowd ×1.05 (132-146), the LoveSpace ×0.95
  (147-151), the personality traits (164-168) and the clamp + banker's rounding
  (176-181: 0.75…1.5, then `Round(x*100)/100`).
- `Terraria/GameContent/ShopHelper.cs::GetNearbyResidentNPCs` (288-325) — the
  neighbour radii: **25 tiles = "house"**, **120 tiles = "village"**.
- `Terraria/GameContent/Personalities/PersonalityDatabasePopulator.cs` (149) — the
  `Register(npcId)` rows (the per-NPC likes/dislikes: e.g. `Register(107)` 117-121,
  `Register(20)` 82-86, `Register(633)` 42-46) + `AllPersonalitiesModifier.cs` (507).
- `Terraria/GameContent/TeleportPylonsSystem.cs` — `HasPylonOfType` (41),
  `UpdatePylonsListAndBroadcastChanges` (60), `HandleTeleportRequest` (100-205, the
  gate ladder), `DoesPositionHaveEnoughNPCs` (221-238), `DoesPylonAcceptTeleportation`
  (245-302), `HowManyNPCsDoesPylonNeed` (304-311), `SpawnInWorldDust` (327).
- `Terraria/GameContent/Tile_Entities/TETeleportationPylon.cs` — the 3×4 footprint
  (9/10), the style = `frameX / 54` (110-113), the item map (115-131),
  `PlacementPreviewHook_CheckIfCanPlace` (153-161: **one pylon per type**) and
  `Framing_CheckTile` (58-108: the item drops back when the pylon breaks).
- `Terraria/NPC.cs` — `townNPC` (6537), `homeless` (6543), `homelessDespawn` (6545),
  `homeTileX/homeTileY` (6551/6553), `housingCategory` (6555), `isLikeATownNPC`
  (6856), `ShimmeredTownNPCs` (6071), `CheckDialogue` (90519) + `ConditionalDialogue`
  (6539) + `GetChat` (93766) + the per-NPC chatter tables (93976 Nurse, 94103
  Tinkerer, 94255 Mechanic, 94272 Guide, 94473 Cyborg, 94571 Pirate, 94684 Stylist,
  94760 Angler, 94771 Skeleton Merchant, 94794 Golfer).
- `Terraria/Main.cs::DrawNPCHousesInUI` (41993-42107) — the housing catalogue UI.

### Rust (the port)

- `crates/tr-core/src/housing.rs::check_room` / `room_needs` / `start_room_check` —
  the room-check trio the app calls (the M4-TOWN-1 row's home).
- `crates/tr-app/src/town.rs::quick_find_home` (21-47) — the three-pass scan;
  `::scan_spot` (50-57) — `check_room` + `room_needs`.
- `crates/tr-app/src/town.rs::TownRooms` (64-160) — the `TownRoomManager` port
  (`has_room`, `has_room_quick`, `set_room`, `kick_out`, `clear`, the wld round-trip).
- `crates/tr-app/src/town.rs::ShimmeredTownNpcs` (162-216) — `NPC.ShimmeredTownNPCs`.
- `crates/tr-app/src/town.rs::fix_town_npc_homes` (219-244) — the load-path
  re-validation (`WorldFile.FixAgainstExploits:1864-1896`): re-check the home, else
  the stored room, else kick out and go homeless.
- `crates/tr-app/src/happiness.rs::process_mood` (308-448) + `Mood`/`MoodFlag`
  (283-301) + `biome_preferences` (64-272) + `neighbor_preferences` (274-281) +
  `classify_biome` (450-471) — the `ShopHelper` port.
- `crates/tr-core/src/pylons.rs` — `PylonType` (33) + `from_tile_style` (67) +
  `item_id` (72), `npcs_needed` (99), `does_position_have_enough_npcs` (113),
  `does_pylon_accept_teleportation` (148), `handle_teleport_request` (234),
  `scan_pylons` (306).
- `crates/tr-app/src/ui/housing.rs` — the `Main.DrawNPCHousesInUI` port
  (`HEAD_ORDER` 24, `type_to_head_index` 48, `layout` 159, `hover_text` 255, `draw` 308).
- `crates/tr-core/src/wld/sections/town_rooms.rs` — the `.wld` town-rooms codec
  (the C# `TownRoomManager.Save/Load`).
- `crates/tr-app/src/npc.rs` — the housing state: `home: Option<(i32,i32)>`
  (the collapsed `homeTileX/homeTileY` + `homeless`, documented at 12632-12636),
  `homeless_despawn` (2527), `is_town_npc` (1444) / `is_town_npc_like` (1470).
- `crates/tr-app/src/world_sim/npcs.rs` — the town home look-up call site
  (2280, per research/05).

## Dependencies

- **C# calls into:** `NPC.homeless/homeTileX/homeTileY/housingCategory`,
  `Main.wallHouse`, `TileID.Sets.RoomNeeds.CountsAs*Types`, `TileID.Sets.HousingWalls`,
  `TileID.Sets.IgnoredInHouseScore`, `Main.tileSolid[379]`, `SceneMetrics` (the pylon
  biome gates), `TileEntity.ByPosition` (the pylon scan), `Player.currentShoppingSettings`
  (the mood report), `Language.GetTextValue` (the report strings).
- **Called by:** `WorldGen.SpawnTownNPC` (5409), `Main`'s 7200-tick move-in evaluation
  (`Main.cs:65542`), `WorldFile.FixAgainstExploits` (1864-1896), `Main.OpenShop`
  (`Main.cs:40459`), `Main.DrawNPCHousesInUI` (41993), `NPCInteractions.Actions`.
- **Rust calls into:** `tr_core::housing` (the room check), `tr_core::scene::SceneMetrics`
  (the pylon biome gates + the mood's biome classification), `tr_core::wld::sections`
  (the room records), `crate::npc::Npc` (the home/homeless state).
- **Shared data structures:** the room registry (`TownRooms` ↔ `TownRoomManager`),
  the `home`/`homeless` pair (`Option<(i32,i32)>` ↔ `homeTileX/homeTileY` + `homeless`),
  the town head-count (`townNPCs`, `NPC.cs:165`; the music ladder at `Main.cs:12199`),
  the pylon list (`Vec<PylonInfo>` ↔ `List<TeleportPylonInfo>`).

## Port status

| C# rule | C# location | Rust | Verdict |
| --- | --- | --- | --- |
| Room check: edge rule, solid cut, 8-neighbour flood fill, 750-tile cap, span < 100, < 60 tiles → too small, the 7 failure reasons | `WorldGen.cs:6041-6237` | `tr-core/src/housing.rs::start_room_check` / `check_room` (called from `town.rs:51`) | **PORTED** (row M4-TOWN-1; the internals of `housing.rs` were not re-read line by line in this pass — see the task seed) |
| Furniture: chair + table + torch + door (`CountsAs*Types`) | `WorldGen.cs:5679-5730` | `tr_core::housing::room_needs` (`town.rs:52`) | **PORTED** |
| Room **score**: base 50, the evil penalty, the −250 reject, the chest/door/home penalties, the standing-space test | `WorldGen.cs:5858-5992` | — | **MISSING** — no score function; `town.rs::scan_spot` returns the *scan origin* as the home (50-57) instead of the score-best `bestX/bestY` |
| Room occupancy: one tenant per spot, per `housingCategory` | `WorldGen.cs:5644-5657`, `5796-5815`, `TownRoomManager.cs:152-166` | — | **MISSING** — no `housing_category` on `Npc` (`npc.rs`), no `can_npcs_live_with_each_other` |
| Room registry (`_roomLocationPairs` + `_hasRoom`), `SetRoom`/`KickOut`/`HasRoom` | `TownRoomManager.cs:11-90` | `town.rs::TownRooms` (64-160) | **PORTED** |
| `GetHouseholdStatus` (the 0/1/2 byte) | `TownRoomManager.cs:139-150` | — | **MISSING** (no consumer found) |
| Room save/load (`.wld` section) | `TownRoomManager.cs:91-112` | `tr-core/src/wld/sections/town_rooms.rs` (241) | **PORTED** |
| Home search: the three-pass scan + `RoomNeeds` + `ScoreRoom` + the assignment | `WorldGen.cs:5732-5794` | `town.rs::quick_find_home` (21-47) + `scan_spot` (50-57) | **PARTIAL** — the scan passes and the furniture gate match; the score gate (`hiScore > 0`) and the score-best home tile do not |
| `lookForHomeTimeout == 0` before a homeless NPC may move in | `WorldGen.cs:5621` | — | **MISSING** (no timeout field) |
| Load-path home re-validation (`FixAgainstExploits`) | `WorldFile.cs:1864-1896` | `town.rs::fix_town_npc_homes` (219-244) | **PORTED** |
| Mid-game relocation (`MoveTownNPC` / `moveRoom`) | `WorldGen.cs:4685`, `4843` | — | **MISSING** (no equivalent in `town.rs`; only the load-time healing exists) |
| "Request a home" dialogue (`NPCInteractions.RequestHome`) | `NPCInteractions.cs:314-340` | — | **MISSING / not located** (the dialogue-action table is not ported) |
| `NPC.housingCategory` (common vs pet) | `NPC.cs:6555`, `HousingCategoryID.cs` | — | **MISSING** |
| Mood: the price multiplier, the 0.75-1.5 clamp + 2-decimal rounding | `ShopHelper.cs:95-181` | `happiness.rs::process_mood` (308-448) | **PORTED** (row M4-TOWN-2) |
| Mood: the homelessness / far-from-home / evil-biome 1000 sentinel | `ShopHelper.cs:107-118` | `happiness.rs:343+` | **PORTED** |
| Mood: the crowd ×1.05 past 3, the LoveSpace ×0.95, the 25/120-tile radii | `ShopHelper.cs:132-151`, `288-325` | `happiness.rs` (the crowd/LoveSpace arms; the tests `the_crowd_multiplies_past_three`, `the_love_space_gate_needs_the_village_room`) | **PORTED** |
| Mood: the remix-world / 368 / 453 / town-pet exclusion | `ShopHelper.cs:103-106` | `happiness.rs:318-328` | **PARTIAL** — only the Travelling Merchant (368) returns early; the remix-world and the 37/453 exclusions are missing while the comment claims them |
| Mood: the Princess (663) branch (`HateLonely`, the ≥2/≥2 gate) | `ShopHelper.cs:122-131` | — | **MISSING** (the port's town set has no Princess) |
| Mood: the per-NPC personality rows (biome + neighbour likes) | `PersonalityDatabasePopulator.cs` (149), `AllPersonalitiesModifier.cs` (507) | `happiness.rs::biome_preferences` (64-272) + `neighbor_preferences` (274-281) | **PARTIAL** — 17 NPC rows ported; the un-ported NPCs (incl. the Princess) have no rows |
| Mood: the report string (`TownNPCMood_<Name>.<key>`) | `ShopHelper.cs:188-196`, `NPCInteractions.cs:289-313` | `happiness.rs::MoodFlag` (283) | **PARTIAL** — the flags exist, the localised report text does not |
| Mood: the shop HUD (the happiness icon + the `P0` price) | `Main.cs:41792-41797`, `Main.cs:2339-2345` | `tr-app/src/ui` (the shop row, row M4-TOWN-6) | **PARTIAL** — the price rides the mood; the icon/offset block was not located |
| Pylon: the 11 kinds, the style = `frameX / 54`, the item map | `TeleportPylonType.cs`, `TETeleportationPylon.cs:110-131` | `pylons.rs::PylonType` (33), `from_tile_style` (67), `item_id` (72) | **PORTED** |
| Pylon: the registry + the TE-book scan | `TeleportPylonsSystem.cs:60-85` | `pylons.rs::scan_pylons` (306) | **PORTED** (the net broadcast is not) |
| Pylon: `HowManyNPCsDoesPylonNeed` = 2 (Victory 0) | `TeleportPylonsSystem.cs:304-311` | `pylons.rs::npcs_needed` (99-104) | **PORTED** |
| Pylon: `DoesPositionHaveEnoughNPCs` (the `ZoneScanSize` rect, `!homeless`, the < 100-tile home-to-centre distance) | `TeleportPylonsSystem.cs:221-238` | `pylons.rs::does_position_have_enough_npcs` (113-141) | **PORTED** (verbatim) |
| Pylon: `DoesPylonAcceptTeleportation` (the purity exclusivity, the 380-tile ocean bands, the depth bands, the biome gates) | `TeleportPylonsSystem.cs:245-302` | `pylons.rs::does_pylon_accept_teleportation` (148+) | **PARTIAL** — the non-remix branch only (the remix arms 252-254/276-279/284-285 are skipped; the row admits it) |
| Pylon: the teleport gate ladder + the failure keys | `TeleportPylonsSystem.cs:100-205` | `pylons.rs::handle_teleport_request` (234-298) | **PORTED** |
| Pylon: **one pylon per type** (the duplicate placement is refused) | `TETeleportationPylon.cs:13-27`, `153-161` | — | **MISSING / not located** (no duplicate-type gate in `pylons.rs`) |
| Pylon: the item drop when a pylon breaks (`Framing_CheckTile`) | `TETeleportationPylon.cs:58-108` | — | **MISSING** (the TE row M7-TE-2 ports only the pylon's `ValidTile`) |
| Pylon: the net module (`NetTeleportPylonModule`), the map layer | `NetModules/NetTeleportPylonModule.cs` (81), `Map/TeleportPylonsMapLayer.cs` (120) | — | **MISSING** (net + map layers are out of this module) |
| The housing catalogue UI (the NPC heads, the hover names) | `Main.cs:41993-42107` | `tr-app/src/ui/housing.rs` (468) | **PORTED** |
| Chatter: `NPC.GetChat`, `ConditionalDialogue`, the 10 chatter tables | `NPC.cs:93766`, `6539`, `90519-90529`, `93976-94794` | — | **MISSING** (research/05 lists `CheckDialogue` among the un-ported `UpdateNPC` phases) |
| The pet chatter lines (Dog/Cat/Bunny/Slime) | `NPCInteractions.cs:341-353` | — | **MISSING** |
| The town head-count (`townNPCs`) used by the music ladder | `NPC.cs:165`, `Main.cs:12199` | the town roster in `world_sim/npcs.rs` | **PARTIAL** (the count's consumers live in the audio module) |

## Tests

- `crates/tr-app/src/town.rs` (247-498): `the_homeless_guide_finds_a_furnished_room`,
  `an_unfurnished_room_rejects`, `test_DW_4_1_the_rooms_round_trip_through_the_wld_seam`,
  `set_room_replaces_the_npcs_previous_room`, `kick_out_clears_the_pair_and_the_fast_table`,
  `an_out_of_range_room_npc_id_is_dropped`, `test_DW_4_3_the_shimmered_bitmap_round_trips`,
  `test_DW_4_3_an_invalid_home_with_no_room_kicks_the_npc_out`,
  `test_DW_4_3_an_invalid_home_with_a_stored_room_teleports_in`,
  `test_DW_4_3_an_unusable_stored_room_still_kicks_out`,
  `test_DW_4_3_a_valid_home_is_left_alone`,
  `test_DW_4_3_a_homeless_npc_is_never_revalidated`,
  `test_DW_4_3_a_home_outside_the_world_bounds_kicks_out`,
  `a_non_town_npc_is_never_revalidated`, `test_DW_4_3_the_load_path_heals_a_bogus_home`,
  `test_DW_4_5_the_citations_are_present`.
- `crates/tr-app/src/happiness.rs` (473-705): `the_travelling_merchant_ignores_the_mood`,
  `the_forest_guide_alone_is_content`, `the_homeless_guide_prices_at_the_ceiling`,
  `the_evil_biome_prices_at_the_ceiling`, `the_crowd_multiplies_past_three`,
  `the_love_space_gate_needs_the_village_room`,
  `the_dryad_judges_the_jungle_and_the_desert`,
  `the_tinkerer_judges_the_underground_and_the_jungle`.
- `crates/tr-core/src/pylons.rs` (332-528): `the_type_map_and_items_are_verbatim`,
  `the_npc_home_check_uses_the_zone_and_100px`, `the_biome_gates_match_the_types`,
  `the_teleport_ladder_walks_the_gates`, `the_network_scan_reads_the_styles`.
- `crates/tr-app/src/ui/housing.rs` (336-467): `the_head_table_matches_the_vanilla_switch`,
  `heads_are_resolved_per_type_in_first_appearance_order`,
  `the_page_wraps_a_column_before_the_bottom_band`, `the_hover_probe_reads_a_cell`,
  `a_third_column_hides_the_pvp_icons`, `the_hover_text_names_the_cell`,
  `the_head_icon_shrinks_to_thirty_six_pixels`.
- `crates/tr-core/src/wld/sections/town_rooms.rs` (84-240): `test_DW_5_5_the_section_is_implemented`,
  `test_DW_5_6_the_rooms_round_trip_byte_equal`,
  `test_DW_5_6_the_hand_computed_rooms_match_the_c_sharp_layout`,
  `test_DW_5_6_the_codec_round_trips_what_the_town_rooms_seam_carries`,
  `a_room_npc_id_outside_the_table_is_refused`, `a_negative_count_is_refused`,
  `a_truncated_room_is_refused`, `a_hostile_count_does_not_allocate`,
  `duplicate_rooms_for_one_npc_are_kept`, `test_DW_5_5_the_citations_are_present`.
- `crates/tr-core/src/housing.rs` — the row M4-TOWN-1 owns the room-check tests;
  their names were not re-read in this pass.

## Deviations & risks

1. **The room score (`ScoreRoom`) is not ported** — *behavioural, medium*. The C# only
   accepts a home when `hiScore > 0` and picks the **best-scoring** standing spot
   (`bestX/bestY`, `WorldGen.cs:5983-5988`); the Rust's `town.rs::scan_spot` (50-57)
   returns the **scan origin** and ignores the score, so a town NPC can be homed on a
   spot the vanilla would reject (no standing space / an evil room) and always homes
   on a different tile than the vanilla. Not admitted by ROADMAP/ANALYSIS.
2. **The remix-world mood exclusion is missing** — *behavioural, medium*. The C# returns
   early for `Main.remixWorld` (`ShopHelper.cs:103`), so every remix-world NPC sells at
   `PriceAdjustment = 1`. `happiness.rs:318-328` documents the exclusion but only
   implements the Travelling Merchant arm → remix worlds get real mood prices.
3. **The 37/453 mood exclusions are missing** — *behavioural, low*. `IsNotReallyTownNPC`
   (`ShopHelper.cs:363-371`) covers the Old Man (37) and type 453; the Rust checks
   only the Travelling Merchant.
4. **The "one tenant per house" rule is missing** — *behavioural, medium*.
   `CanNPCsLiveWithEachOther` / `IsRoomConsideredAlreadyOccupied` (the `housingCategory`
   comparison) is absent, and `Npc` has no `housing_category` field → in the port two
   incompatible NPCs can be assigned the same home spot. The room registry itself is
   one-room-per-NPC (`TownRoomManager.cs:57-64`) and that half *is* ported.
5. **The mid-game relocation is missing** — *behavioural, medium*. `MoveTownNPC`
   (`WorldGen.cs:4685`) / `moveRoom` (4843) re-home an NPC when its house is destroyed;
   the Rust only has the **load-time** healing (`town.rs::fix_town_npc_homes`). A player
   who breaks a house mid-session keeps a stale home (the C# would move or unhouse him).
6. **`lookForHomeTimeout` is missing** — *behavioural, low*. The C# only lets a homeless
   NPC move in when its timeout has expired (`WorldGen.cs:5621`); the Rust's
   `quick_find_home` has no cooldown, so the move-in cadence differs from the vanilla.
7. **The pylon duplicate-type gate is missing** — *behavioural, medium*. The vanilla
   refuses a second pylon of the same type at placement and kills a net-placed
   duplicate (`TETeleportationPylon.cs:13-27`, `153-161`); the Rust's `pylons.rs` has
   no equivalent check in the reviewed functions.
8. **The pylon's remix branches are skipped** — *behavioural, low-medium*. The C# has
   three remix arms in `DoesPylonAcceptTeleportation` (252-254, 276-279, 284-285);
   the Rust ports only the non-remix branch. **Admitted** by the ROADMAP row M7-PYLON-1
   ("the remix branches skipped").
9. **The chatter/dialogue layer is missing** — *behavioural, high (feature gap)*.
   `NPC.GetChat` (93766), `ConditionalDialogue` (6539/90519) and the ten chatter tables
   (93976-94794) have no Rust counterpart (research/05 also lists `CheckDialogue` as an
   un-ported `UpdateNPC` phase). The town NPCs are silent.
10. **The happiness report text is missing** — *behavioural/cosmetic, medium*. The C#
    shows a localised `TownNPCMood_<Name>.<key>` string plus the shop-hud icon
    (`Main.cs:41792-41797`); the Rust carries `MoodFlag`s but no report text, so the
    "why is this NPC expensive" feedback is absent.
11. **The Princess's mood branch is missing** — *behavioural, low*. `ShopHelper.cs:122-131`
    gives the Princess a hard gate (≥2 house & ≥2 village neighbours, else the 1000
    sentinel + `HateLonely`); the port has no Princess kind.
12. **The `home`/`homeless` collapse** — *cosmetic/determinism, low*. The C# keeps
    `homeTileX/homeTileY` **and** `homeless` **and** the legacy `oldHomeTile*` pair
    (`NPC.cs:6551-6561`); the Rust models the first two as one `Option<(i32,i32)>`
    (`npc.rs:12632-12636`). The legacy `old*` pair (the 1.4.3 → 1.4.4 migration
    fallback) is not ported — a 1.4.3 world loaded into the port keeps no fallback home.
13. **Citation drift in the ROADMAP** — *cosmetic, low*. The M4-TOWN-1 row cites
    `RoomNeeds` as `:5717-5756` and the housing flood fill as `MoveTownNPC 4675 →
    StartRoomCheck 6041`; in the current decompilation `RoomNeeds` is at
    `WorldGen.cs:5679-5730` and `MoveTownNPC` at 4685. The row's other cites
    (`ShopHelper.cs:95-174`, `TeleportPylonsSystem.cs:62-84`, `:203-219`, `:240-306`,
    `:93-186`) match the source.

## Task seeds

- **M — Port `WorldGen.ScoreRoom` + the `hiScore > 0` gate** (`WorldGen.cs:5858-5992`)
  and make `town::scan_spot` return the score-best `bestX/bestY` instead of the scan
  origin. *Depends on:* nothing (the room check already exists in `tr-core::housing`).
  Unblocks deviation 1 and the home-tile fidelity.
- **M — Add the housing occupancy rule**: a `housing_category` on `Npc`
  (0 = common, 1 = pet), a `can_npcs_live_with_each_other` helper and the
  `IsRoomConsideredAlreadyOccupied` / `IsRoomConsideredOccupiedForNPCIndex` gates
  (`WorldGen.cs:5644-5657`, `5796-5815`). *Depends on:* the `ScoreRoom` task (both
  pick the home tile).
- **M — Port the mid-game relocation** (`WorldGen.MoveTownNPC` 4685 / `moveRoom` 4843):
  re-home or unhouse a town NPC when its home is destroyed, plus the
  `lookForHomeTimeout` gate (`WorldGen.cs:5621`). *Depends on:* the `ScoreRoom` task
  (the new home must be a scored spot).
- **S — Fix the mood exclusion block** (`ShopHelper.cs:103-106`): add the
  `Main.remixWorld` early return and the 37/453 exclusions to
  `happiness.rs::process_mood`. *Depends on:* nothing.
- **M — Port the happiness report**: the `TownNPCMood_*` localisation keys
  (`ShopHelper.cs:188-196`), the `NPCInteractions.ReportHappiness` dialogue (289-313)
  and the shop-hud icon/offsets (`Main.cs:2339-2345`, `41792-41797`).
  *Depends on:* M6-LOCALIZE for the keys.
- **M — Port the town chatter**: `NPC.GetChat` (93766), `ConditionalDialogue`
  (6539/90519) and the ten per-NPC chatter tables, wired into the NPC right-click
  dialogue. *Depends on:* M6-LOCALIZE.
- **S — Add the pylon duplicate-type gate** (`TETeleportationPylon.cs:13-27`,
  `153-161`) and the `Framing_CheckTile` item drop (58-108).
  *Depends on:* M7-TE-2 for the TE lifecycle.
- **S — Port the remix arms of `DoesPylonAcceptTeleportation`**
  (`TeleportPylonsSystem.cs:252-254`, `276-279`, `284-285`) — already an admitted
  follow-up of M7-PYLON-1.
- **S — Re-read `tr-core/src/housing.rs` against `WorldGen.cs:6041-6237`** and record
  the room-check fidelity in ANALYSIS (this pass verified the call sites and the API
  shape, not the flood-fill internals).
- **S — Fix the ROADMAP citation drift** on the M4-TOWN-1 row (`RoomNeeds` 5679-5730,
  `MoveTownNPC` 4685).
