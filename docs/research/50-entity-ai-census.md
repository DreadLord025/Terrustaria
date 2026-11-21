# 50 — Entity-AI census & re-audit baseline (full roster)

**Branch:** `docs/entity-ai-census` · **Base:** `origin/main` `c7fe175d` (merge phase complete, ~111 landed branches) · **Mode:** CODE-ONLY read audit — file reads + git only, **no compiler invoked**.

Scope: every ported `NpcKind` (roster = `npc/data.rs::to_u16`, port kind-ids), every NPC aiStyle dispatch path (`npc.rs` style axis + kind match + `world_sim/npcs.rs` passes), every projectile aiStyle arm + `projectile/data.rs` rows, the session-audit defect inventory (TR-SWARM-LEDGER / TR-MERGE-RESULTS), and the proposed next-wave audit slices.

Method: scripted extraction (regex over brace-balanced fn bodies; scripts kept in the worktree, not committed) cross-checked by hand-reads of the hot arms. C# anchors re-derived from `decompiled/Terraria/NPC.cs` (`AI()` at :19819, if-chain :19821-41132) and `Projectile.cs` (`AI()` at :23199, if-chain :23201-36806). All Rust line numbers are at `c7fe175d`.

## 0. Scoreboard

| Metric | Count |
| --- | --- |
| Ported NpcKind rows (roster) | 238 |
| Distinct NPC aiStyles in roster | 68 |
| NPC.AI() if-chain styles in C# | 128 (0-127) |
| Roster kinds WIRED-MODULE (faithful module in dispatch) | 9 |
| Roster kinds WIRED-DELEGATE (live fn delegates to landed faithful helper) | 51 |
| Roster kinds LIVE-INLINE (hand-ported arm, fidelity unverified this pass) | 75 |
| Roster kinds STAND-IN still live | 83 |
| Roster kinds SIM-PASS (worm chain, aiStyle 6) | 14 |
| Roster kinds EMPTY-OK (bound NPCs, aiStyle 0) | 6 |
| Landed NPC-AI modules with NO dispatch reference (UNWIRED) | 30 (list §2b) |
| Projectile dispatch arms (live, unique styles) | 65 styles / 72 arm entries |
| Projectile dead duplicate arms (unreachable) | 6 (styles 1,3,14,16,115,99) |
| proj_data rows resolved (kind → style) | 307 kinds / 71 styles (249 match arms) |
| Data styles with NO dispatch arm | 6 (31,120,147,181,193,194) |
| C# Projectile.AI() styles total | 210 |
| Session-audit real defects in "done" arms (ledger tally) | 32+ (2+10+6+14) + 2 wholesale stand-ins + 5 plantera gaps |

**Key structural fact:** the port dispatches NPC AI **kind-first** (`npc.rs::update_kind_ai` :3736-4251) with a thin aiStyle axis on top (`npc.rs` :3651-3689, claims styles 5/9/31/47/57-MourningWood/58-Pumpking only). The mass-landed `npc_ai_style*` modules are mostly **pure-function ports sitting UNWIRED** while the old stand-in arm keeps running. The next wave's job is **verify fidelity + claim dispatch**, not re-port.

## 1. NPC roster census (kind-id order)

Quality legend:
- **WIRED-MODULE** — dispatch calls a landed faithful module directly.
- **WIRED-DELEGATE** — live `update_*` fn delegates to a landed faithful helper (`npc_ai`, `npc_ai3_speeds`, `npc_ai_bat`, `npc_ai_golem`, `npc_ai_plantera`, `npc_target`); fidelity of the wrapper itself still needs the per-slice audit.
- **LIVE-INLINE** — hand-ported inline machine; not covered by a session audit yet (audit wave targets).
- **STAND-IN** — approximation still live (invented constants / simplified machine); a faithful UNWIRED module exists where noted.
- **SIM-PASS** — aiStyle-6 worm kinds ride `world_sim/npcs.rs::update_worms_in` (chain move stand-in for `AI_006_Worms` :50963-52273).
- **EMPTY-OK** — intentional no-op arm (bound/sleeping NPCs; rescue logic lives in `world_sim/npcs.rs` :544-616).

| id | kind | aiStyle | serving fn / module (npc.rs lines) | quality | note |
| --- | --- | --- | --- | --- | --- |
| 0 | Slime | 1 | `update_slime` :6652-6732 | STAND-IN |  |
| 1 | DemonEye | 2 | `update_eye` :6733-6774 | STAND-IN |  |
| 2 | WanderingEye | 2 | `update_eye` :6733-6774 | STAND-IN |  |
| 3 | Zombie | 3 | `update_zombie` :6775-6888 | WIRED-DELEGATE | npc_ai + npc_ai3_speeds + npc_target |
| 4 | KingSlime | 15 | `update_king_slime` :7290-7458 | LIVE-INLINE |  |
| 5 | Guide | 7 | `update_guide` :7471-7515 | STAND-IN |  |
| 6 | Skeleton | 3 | `update_skeleton` :7516-7568 | WIRED-DELEGATE | npc_ai3_speeds ladder |
| 7 | UndeadMiner | 3 | `update_skeleton` :7516-7568 | WIRED-DELEGATE | npc_ai3_speeds ladder |
| 8 | CaveBat | 14 | `update_cave_bat` :7743-7777 | STAND-IN | npc_ai::flyer_step; faithful npc_ai_style14 UNWIRED |
| 9 | JungleBat | 14 | `update_cave_bat` :7743-7777 | STAND-IN | npc_ai::flyer_step; faithful npc_ai_style14 UNWIRED |
| 10 | Hellbat | 14 | `update_cave_bat` :7743-7777 | STAND-IN | npc_ai::flyer_step; faithful npc_ai_style14 UNWIRED |
| 11 | VoodooDemon | 14 | `update_cave_bat` :7743-7777 | STAND-IN | npc_ai::flyer_step; faithful npc_ai_style14 UNWIRED |
| 12 | EaterOfSouls | 5 | `update_hover` :7778-7794 | WIRED-DELEGATE | npc_ai::hover_step |
| 13 | Hornet | 5 | `update_hover` :7778-7794 | WIRED-DELEGATE | npc_ai::hover_step |
| 14 | MeteorHead | 5 | `update_hover` :7778-7794 | WIRED-DELEGATE | npc_ai::hover_step |
| 15 | GoblinSorcerer | 8 | `update_caster` :8002-8018 | STAND-IN | npc_ai_bat::ai_style_8_step; faithful npc_ai_caster UNWIRED |
| 16 | Tim | 8 | `update_caster` :8002-8018 | STAND-IN | npc_ai_bat::ai_style_8_step; faithful npc_ai_caster UNWIRED |
| 17 | DarkCaster | 8 | `update_caster` :8002-8018 | STAND-IN | npc_ai_bat::ai_style_8_step; faithful npc_ai_caster UNWIRED |
| 18 | FireImp | 8 | `update_caster` :8002-8018 | STAND-IN | npc_ai_bat::ai_style_8_step; faithful npc_ai_caster UNWIRED |
| 19 | ChaosBall | 9 | `npc_ai_style9::sphere_family_step` | WIRED-MODULE | via style-9 axis :3659 (kind arm unreachable) |
| 20 | BurningSphere | 9 | `npc_ai_style9::sphere_family_step` | WIRED-MODULE | via style-9 axis :3659 (kind arm unreachable) |
| 21 | WaterSphere | 9 | `npc_ai_style9::sphere_family_step` | WIRED-MODULE | via style-9 axis :3659 (kind arm unreachable) |
| 22 | Goldfish | 16 | `update_swimmer` :14972-15048 | STAND-IN | npc_ai::swim_step; faithful npc_ai_style16 UNWIRED |
| 23 | Piranha | 16 | `update_swimmer` :14972-15048 | STAND-IN | npc_ai::swim_step; faithful npc_ai_style16 UNWIRED |
| 24 | Shark | 16 | `update_swimmer` :14972-15048 | STAND-IN | npc_ai::swim_step; faithful npc_ai_style16 UNWIRED |
| 25 | DevourerHead | 6 | `(empty)` | SIM-PASS | worm chain pass `world_sim/npcs.rs::update_worms_in` :1055-1315 (AI_006 stand-in) |
| 26 | DevourerBody | 6 | `(empty)` | SIM-PASS | worm chain pass `world_sim/npcs.rs::update_worms_in` :1055-1315 (AI_006 stand-in) |
| 27 | DevourerTail | 6 | `(empty)` | SIM-PASS | worm chain pass `world_sim/npcs.rs::update_worms_in` :1055-1315 (AI_006 stand-in) |
| 28 | GiantWormHead | 6 | `(empty)` | SIM-PASS | worm chain pass `world_sim/npcs.rs::update_worms_in` :1055-1315 (AI_006 stand-in) |
| 29 | GiantWormBody | 6 | `(empty)` | SIM-PASS | worm chain pass `world_sim/npcs.rs::update_worms_in` :1055-1315 (AI_006 stand-in) |
| 30 | GiantWormTail | 6 | `(empty)` | SIM-PASS | worm chain pass `world_sim/npcs.rs::update_worms_in` :1055-1315 (AI_006 stand-in) |
| 31 | EaterOfWorldsHead | 6 | `(empty)` | SIM-PASS | worm chain pass `world_sim/npcs.rs::update_worms_in` :1055-1315 (AI_006 stand-in) |
| 32 | EaterOfWorldsBody | 6 | `(empty)` | SIM-PASS | worm chain pass `world_sim/npcs.rs::update_worms_in` :1055-1315 (AI_006 stand-in) |
| 33 | EaterOfWorldsTail | 6 | `(empty)` | SIM-PASS | worm chain pass `world_sim/npcs.rs::update_worms_in` :1055-1315 (AI_006 stand-in) |
| 34 | Merchant | 7 | `update_guide` :7471-7515 | STAND-IN |  |
| 35 | Dryad | 7 | `update_guide` :7471-7515 | STAND-IN |  |
| 36 | Nurse | 7 | `update_guide` :7471-7515 | STAND-IN |  |
| 37 | ArmsDealer | 7 | `update_guide` :7471-7515 | STAND-IN |  |
| 38 | Demolitionist | 7 | `update_guide` :7471-7515 | STAND-IN |  |
| 39 | Painter | 7 | `update_guide` :7471-7515 | STAND-IN |  |
| 40 | WitchDoctor | 7 | `update_guide` :7471-7515 | STAND-IN |  |
| 41 | Clothier | 7 | `update_guide` :7471-7515 | STAND-IN |  |
| 42 | EyeOfCthulhu | 4 | `update_eye_of_cthulhu` :6889-7193 | LIVE-INLINE |  |
| 43 | QueenBee | 43 | `update_queen_bee` :8019-8099 | STAND-IN | faithful npc_ai_style43 UNWIRED |
| 44 | SkeletronHead | 11 | `update_skeletron` :8100-8300 | LIVE-INLINE |  |
| 45 | ServantOfCthulhu | 5 | `update_servant_of_cthulhu` :7212-7289 | LIVE-INLINE |  |
| 46 | WallOfFlesh | 27 | `update_wof` :8301-8487 | STAND-IN | faithful npc_ai_style27 UNWIRED |
| 47 | Hungry | 29 | `update_hungry` :8488-8596 | STAND-IN | faithful npc_ai_style29 UNWIRED |
| 48 | LeechHead | 6 | `(empty)` | SIM-PASS | worm chain pass `world_sim/npcs.rs::update_worms_in` :1055-1315 (AI_006 stand-in) |
| 49 | LeechBody | 6 | `(empty)` | SIM-PASS | worm chain pass `world_sim/npcs.rs::update_worms_in` :1055-1315 (AI_006 stand-in) |
| 50 | TheDestroyer | 37 | `(empty)` | STAND-IN | worm-pass stand-in; faithful AI_037 module npc_ai_style37 UNWIRED |
| 51 | SkeletronPrime | 32 | `update_prime` :12963-13117 | LIVE-INLINE |  |
| 52 | Retinazer | 30 | `update_retinazer` :13118-13335 | LIVE-INLINE |  |
| 53 | Spazmatism | 31 | `npc_ai_style31::spazmatism_ai` | WIRED-MODULE |  |
| 54 | Plantera | 51 | `update_plantera` :13410-13455 | WIRED-DELEGATE | npc_ai_plantera + anchor |
| 55 | BrainOfCthulhu | 54 | `update_brain` :14629-14835 | LIVE-INLINE |  |
| 56 | Creeper | 55 | `update_creeper` :14575-14628 | STAND-IN | faithful npc_ai_style55 UNWIRED |
| 57 | GoblinThief | 3 | `update_skeleton` :7516-7568 | WIRED-DELEGATE | npc_ai3_speeds ladder |
| 58 | GoblinWarrior | 3 | `update_skeleton` :7516-7568 | WIRED-DELEGATE | npc_ai3_speeds ladder |
| 59 | GoblinArcher | 3 | `update_archer` :14836-14944 | LIVE-INLINE |  |
| 60 | GoblinPeon | 3 | `update_skeleton` :7516-7568 | WIRED-DELEGATE | npc_ai3_speeds ladder |
| 61 | GoblinSummoner | 3 | `update_goblin_summoner` :7795-7945 | LIVE-INLINE |  |
| 62 | DesertDjinn | 8 | `update_caster` :8002-8018 | STAND-IN | npc_ai_bat::ai_style_8_step; faithful npc_ai_caster UNWIRED |
| 63 | BoundGoblin | 0 | `(empty)` | EMPTY-OK | bound NPC stands still until talk (rescue passes npcs.rs :544-616); AI_000 transform partial |
| 64 | GoblinTinkerer | 7 | `update_guide` :7471-7515 | STAND-IN |  |
| 65 | SnowmanGangsta | 38 | `update_frost_legion` :7569-7584 | STAND-IN |  |
| 66 | SnowBalla | 38 | `update_frost_legion` :7569-7584 | STAND-IN |  |
| 67 | MysterySnowman | 38 | `update_frost_legion` :7569-7584 | STAND-IN |  |
| 68 | MartianOfficer | 3 | `update_skeleton` :7516-7568 | WIRED-DELEGATE | npc_ai3_speeds ladder |
| 69 | GrayGrunt | 3 | `update_skeleton` :7516-7568 | WIRED-DELEGATE | npc_ai3_speeds ladder |
| 70 | MartianEngineer | 3 | `update_skeleton` :7516-7568 | WIRED-DELEGATE | npc_ai3_speeds ladder |
| 71 | GigaZapper | 3 | `update_skeleton` :7516-7568 | WIRED-DELEGATE | npc_ai3_speeds ladder |
| 72 | MartianWalker | 3 | `update_skeleton` :7516-7568 | WIRED-DELEGATE | npc_ai3_speeds ladder |
| 73 | Golem | 45 | `update_golem_rng` :14017-14574 | WIRED-DELEGATE | npc_ai_golem cores |
| 74 | GolemHead | 46 | `update_golem_rng` :14017-14574 | WIRED-DELEGATE | npc_ai_golem cores |
| 75 | GolemFistLeft | 47 | `update_golem_rng` :14017-14574 | WIRED-DELEGATE | npc_ai_golem cores |
| 76 | GolemFistRight | 47 | `update_golem_rng` :14017-14574 | WIRED-DELEGATE | npc_ai_golem cores |
| 77 | GolemHeadFree | 48 | `update_golem_rng` :14017-14574 | WIRED-DELEGATE | npc_ai_golem cores |
| 78 | Deerclops | 123 | `update_deerclops` :13807-13980 | LIVE-INLINE |  |
| 79 | SkeletronHand | 12 | `update_skeletron_hand` :13735-13806 | LIVE-INLINE |  |
| 80 | WallOfFleshEye | 28 | `update_wof_eye` :13616-13734 | STAND-IN | faithful npc_ai_style28 UNWIRED |
| 81 | PrimeCannon | 35 | `update_prime_limb` :13456-13615 | STAND-IN |  |
| 82 | PrimeSaw | 33 | `update_prime_limb` :13456-13615 | STAND-IN | faithful npc_ai_style33 UNWIRED |
| 83 | PrimeVice | 34 | `update_prime_limb` :13456-13615 | STAND-IN |  |
| 84 | PrimeLaser | 36 | `update_prime_limb` :13456-13615 | STAND-IN |  |
| 85 | BrainScrambler | 3 | `update_martian_shooter` :8890-9052 | LIVE-INLINE |  |
| 86 | RayGunner | 3 | `update_martian_shooter` :8890-9052 | LIVE-INLINE |  |
| 87 | ScutlixRider | 75 | `update_scutlix_rider` :9053-9132 | LIVE-INLINE |  |
| 88 | Scutlix | 3 | `update_scutlix` :9133-9186 | LIVE-INLINE |  |
| 89 | MartianSaucer | 75 | `update_saucer_mount` :9187-9440 | STAND-IN | DEAD duplicate arm (first arm wins) |
| 90 | MartianSaucerTurret | 75 | `update_saucer_mount` :9187-9440 | STAND-IN | DEAD duplicate arm (first arm wins) |
| 91 | MartianSaucerCannon | 75 | `update_saucer_mount` :9187-9440 | STAND-IN | DEAD duplicate arm (first arm wins) |
| 92 | MartianSaucerCore | 76 | `update_saucer_core` :12718-12962 | LIVE-INLINE |  |
| 93 | FlyingDutchman | 93 | `update_flying_dutchman` :9441-9525 | LIVE-INLINE |  |
| 94 | FlyingDutchmanCannon | 75 | `update_saucer_mount` :9187-9440 | STAND-IN |  |
| 95 | DD2GoblinT1 | 107 | `update_dd2_goblin` :9526-9583 | LIVE-INLINE |  |
| 96 | DD2GoblinT2 | 107 | `update_dd2_goblin` :9526-9583 | LIVE-INLINE |  |
| 97 | DD2GoblinT3 | 107 | `update_dd2_goblin` :9526-9583 | LIVE-INLINE |  |
| 98 | DD2GoblinBomberT1 | 107 | `update_dd2_bomber` :9584-9655 | LIVE-INLINE |  |
| 99 | DD2GoblinBomberT2 | 107 | `update_dd2_bomber` :9584-9655 | LIVE-INLINE |  |
| 100 | DD2GoblinBomberT3 | 107 | `update_dd2_bomber` :9584-9655 | LIVE-INLINE |  |
| 101 | DD2JavelinT1 | 107 | `update_dd2_thrower` :12656-12696 | LIVE-INLINE |  |
| 102 | DD2JavelinT2 | 107 | `update_dd2_thrower` :12656-12696 | LIVE-INLINE |  |
| 103 | DD2JavelinT3 | 107 | `update_dd2_thrower` :12656-12696 | LIVE-INLINE |  |
| 104 | DD2SkeletonT1 | 107 | `update_dd2_skeleton` :12697-12717 | LIVE-INLINE |  |
| 105 | DD2SkeletonT3 | 107 | `update_dd2_skeleton` :12697-12717 | LIVE-INLINE |  |
| 106 | DD2DarkMage | 109 | `update_dark_mage` :9656-10006 | LIVE-INLINE |  |
| 107 | DD2Betsy | 110 | `update_betsy` :10007-10286 | STAND-IN |  |
| 108 | DD2WitherBeast | 107 | `update_wither_beast` :12608-12655 | STAND-IN |  |
| 109 | DD2Bartender | 7 | `update_guide` :7471-7515 | STAND-IN | DEAD duplicate arm (first arm wins) |
| 110 | DD2EterniaCrystal | 105 | `update_eternia_crystal` :10287-10362 | LIVE-INLINE |  |
| 111 | TravellingMerchant | 7 | `update_guide` :7471-7515 | STAND-IN |  |
| 112 | AncientCultist | 84 | `update_lunatic_cultist` :7946-7980 | LIVE-INLINE |  |
| 113 | StardustTower | 94 | `update_ancient_cultist` :12553-12607 | LIVE-INLINE |  |
| 114 | NebulaTower | 94 | `update_ancient_cultist` :12553-12607 | LIVE-INLINE |  |
| 115 | SolarTower | 94 | `update_ancient_cultist` :12553-12607 | LIVE-INLINE |  |
| 116 | StardustCellSmall | 95 | `update_stardust_cell` :10363-10392 | LIVE-INLINE |  |
| 117 | StardustJellyfishBig | 96 | `update_stardust_jellyfish` :10393-10447 | LIVE-INLINE |  |
| 118 | NebulaBeast | 26 | `update_nebula_beast` :10540-10606 | LIVE-INLINE |  |
| 119 | VortexRifleman | 3 | `update_vortex_rifleman` :10607-10730 | STAND-IN |  |
| 120 | VortexSoldier | 3 | `npc_ai::walker_step_ladder` | WIRED-DELEGATE | npc_ai3_speeds ladder |
| 121 | StardustCellBig | 85 | `update_stardust_cell_big` :10448-10539 | LIVE-INLINE |  |
| 122 | MoonLordHead | 79 | `update_moon_lord_head` :11084-11172 | LIVE-INLINE |  |
| 123 | MoonLordHand | 78 | `update_moon_lord_hand` :10931-11083 | LIVE-INLINE |  |
| 124 | MoonLordCore | 77 | `update_moon_lord_core` :10786-10903 | LIVE-INLINE |  |
| 125 | MoonLordFreeEye | 81 | `update_true_eye` :11173-11301 | STAND-IN |  |
| 126 | MoonLordLeechBlob | 82 | `update_moon_leech_blob` :12543-12552 | LIVE-INLINE |  |
| 127 | VortexHunter | 3 | `update_vortex_hunter` :11967-12039 | STAND-IN |  |
| 128 | VortexAlien | 3 | `update_vortex_alien` :12040-12128 | LIVE-INLINE |  |
| 129 | VortexHeadcrab | 3 | `update_vortex_headcrab` :12129-12202 | LIVE-INLINE |  |
| 130 | NebulaFloater | 97 | `update_nebula_floater` :11734-11826 | LIVE-INLINE |  |
| 131 | BrainSuckler | 85 | `update_brain_suckler` :11827-11926 | LIVE-INLINE |  |
| 132 | Predictor | 3 | `update_nebula_predictor` :11927-11966 | LIVE-INLINE |  |
| 133 | StardustSpiderBig | 3 | `update_stardust_spider_big` :11302-11354 | LIVE-INLINE |  |
| 134 | StardustSpiderSmall | 26 | `update_stardust_spider_small` :11355-11408 | LIVE-INLINE |  |
| 135 | StardustSoldier | 3 | `update_stardust_soldier` :11409-11467 | LIVE-INLINE |  |
| 136 | SolarDrakomire | 3 | `update_solar_drakomire` :11468-11495 | LIVE-INLINE |  |
| 137 | SolarDrakomireRider | 75 | `update_solar_drakomire_rider` :11496-11519 | LIVE-INLINE |  |
| 138 | SolarSroller | 39 | `update_solar_sroller` :11520-11596 | STAND-IN | faithful npc_ai_style39 UNWIRED |
| 139 | SolarCorite | 74 | `update_solar_corite` :11597-11673 | LIVE-INLINE |  |
| 140 | SolarSelenian | 3 | `update_solar_selenian` :11674-11701 | LIVE-INLINE |  |
| 141 | SolarCrawltipedeHead | 6 | `(empty)` | SIM-PASS | worm chain pass `world_sim/npcs.rs::update_worms_in` :1055-1315 (AI_006 stand-in) |
| 142 | SolarCrawltipedeBody | 6 | `(empty)` | SIM-PASS | worm chain pass `world_sim/npcs.rs::update_worms_in` :1055-1315 (AI_006 stand-in) |
| 143 | SolarCrawltipedeTail | 6 | `(empty)` | SIM-PASS | worm chain pass `world_sim/npcs.rs::update_worms_in` :1055-1315 (AI_006 stand-in) |
| 144 | StardustWormHead | 6 | `update_stardust_worm_head` :11702-11733 | LIVE-INLINE |  |
| 145 | SolarSpearman | 3 | `npc_ai::walker_step_ladder` | WIRED-DELEGATE | npc_ai3_speeds ladder |
| 146 | VortexTower | 94 | `update_ancient_cultist` :12553-12607 | LIVE-INLINE |  |
| 147 | DD2LanePortal | 106 | `update_dd2_lane` :10731-10785 | LIVE-INLINE |  |
| 150 | Angler | 7 | `update_guide` :7471-7515 | STAND-IN |  |
| 151 | SleepingAngler | 0 | `(empty)` | EMPTY-OK | bound NPC stands still until talk (rescue passes npcs.rs :544-616); AI_000 transform partial |
| 156 | Wizard | 7 | `update_guide` :7471-7515 | STAND-IN |  |
| 157 | BoundWizard | 0 | `(empty)` | EMPTY-OK | bound NPC stands still until talk (rescue passes npcs.rs :544-616); AI_000 transform partial |
| 158 | Mechanic | 7 | `update_guide` :7471-7515 | STAND-IN |  |
| 159 | BoundMechanic | 0 | `(empty)` | EMPTY-OK | bound NPC stands still until talk (rescue passes npcs.rs :544-616); AI_000 transform partial |
| 160 | Stylist | 7 | `update_guide` :7471-7515 | STAND-IN |  |
| 161 | WebbedStylist | 0 | `(empty)` | EMPTY-OK | bound NPC stands still until talk (rescue passes npcs.rs :544-616); AI_000 transform partial |
| 162 | TaxCollector | 7 | `update_guide` :7471-7515 | STAND-IN |  |
| 163 | Golfer | 7 | `update_guide` :7471-7515 | STAND-IN |  |
| 164 | SleepingGolfer | 0 | `(empty)` | EMPTY-OK | bound NPC stands still until talk (rescue passes npcs.rs :544-616); AI_000 transform partial |
| 165 | Zoologist | 7 | `update_guide` :7471-7515 | STAND-IN |  |
| 166 | Princess | 7 | `update_guide` :7471-7515 | STAND-IN |  |
| 167 | DyeTrader | 7 | `update_guide` :7471-7515 | STAND-IN |  |
| 168 | PartyGirl | 7 | `update_guide` :7471-7515 | STAND-IN |  |
| 169 | DD2WyvernT1 | 108 | `update_diving_flyer` :12203-12333 | LIVE-INLINE |  |
| 170 | DD2WyvernT2 | 108 | `update_diving_flyer` :12203-12333 | LIVE-INLINE |  |
| 171 | DD2WyvernT3 | 108 | `update_diving_flyer` :12203-12333 | LIVE-INLINE |  |
| 172 | DD2DarkMageT3 | 109 | `update_dark_mage` :9656-10006 | LIVE-INLINE |  |
| 173 | DD2WitherBeastT3 | 107 | `update_wither_beast_t3` :12334-12352 | LIVE-INLINE |  |
| 174 | DD2DrakinT2 | 107 | `update_drakin` :12353-12414 | LIVE-INLINE |  |
| 175 | DD2DrakinT3 | 107 | `update_drakin` :12353-12414 | LIVE-INLINE |  |
| 176 | DD2KoboldWalkerT2 | 107 | `update_dd2_kobold_walker` :12415-12446 | LIVE-INLINE |  |
| 177 | DD2KoboldWalkerT3 | 107 | `update_dd2_kobold_walker` :12415-12446 | LIVE-INLINE |  |
| 178 | DD2KoboldFlyerT2 | 108 | `update_diving_flyer` :12203-12333 | LIVE-INLINE |  |
| 179 | DD2KoboldFlyerT3 | 108 | `update_diving_flyer` :12203-12333 | LIVE-INLINE |  |
| 180 | DD2OgreT2 | 107 | `update_ogre` :12447-12542 | LIVE-INLINE |  |
| 181 | DD2OgreT3 | 107 | `update_ogre` :12447-12542 | LIVE-INLINE |  |
| 182 | PirateDeckhand | 3 | `update_skeleton` :7516-7568 | WIRED-DELEGATE | npc_ai3_speeds ladder |
| 183 | PirateCaptainsMate | 3 | `update_skeleton` :7516-7568 | WIRED-DELEGATE | npc_ai3_speeds ladder |
| 184 | PirateArcher | 3 | `update_martian_shooter` :8890-9052 | LIVE-INLINE |  |
| 185 | PirateCorsair | 3 | `update_martian_shooter` :8890-9052 | LIVE-INLINE |  |
| 186 | PirateCaptain | 3 | `update_martian_shooter` :8890-9052 | LIVE-INLINE |  |
| 187 | MartianProbe | 80 | `update_martian_probe` :8597-8699 | LIVE-INLINE |  |
| 188 | MartianDrone | 74 | `update_martian_drone` :8700-8865 | LIVE-INLINE |  |
| 189 | MotherSlime | 1 | `update_slime` :6652-6732 | STAND-IN |  |
| 190 | LavaSlime | 1 | `update_slime` :6652-6732 | STAND-IN |  |
| 191 | DungeonSlime | 1 | `update_slime` :6652-6732 | STAND-IN |  |
| 192 | CorruptSlime | 1 | `update_slime` :6652-6732 | STAND-IN |  |
| 193 | IlluminantSlime | 1 | `update_slime` :6652-6732 | STAND-IN |  |
| 194 | IceSlime | 1 | `update_slime` :6652-6732 | STAND-IN |  |
| 195 | Crimslime | 1 | `update_slime` :6652-6732 | STAND-IN |  |
| 196 | SpikedIceSlime | 1 | `update_slime` :6652-6732 | STAND-IN |  |
| 197 | UmbrellaSlime | 1 | `update_slime` :6652-6732 | STAND-IN |  |
| 198 | CataractEye | 2 | `update_eye` :6733-6774 | STAND-IN |  |
| 199 | SleepyEye | 2 | `update_eye` :6733-6774 | STAND-IN |  |
| 200 | DialatedEye | 2 | `update_eye` :6733-6774 | STAND-IN |  |
| 201 | GreenEye | 2 | `update_eye` :6733-6774 | STAND-IN |  |
| 202 | PurpleEye | 2 | `update_eye` :6733-6774 | STAND-IN |  |
| 203 | DoctorBones | 3 | `update_zombie` :6775-6888 | WIRED-DELEGATE | npc_ai + npc_ai3_speeds + npc_target |
| 204 | TheGroom | 3 | `update_zombie` :6775-6888 | WIRED-DELEGATE | npc_ai + npc_ai3_speeds + npc_target |
| 205 | BaldZombie | 3 | `update_zombie` :6775-6888 | WIRED-DELEGATE | npc_ai + npc_ai3_speeds + npc_target |
| 206 | ZombieEskimo | 3 | `update_zombie` :6775-6888 | WIRED-DELEGATE | npc_ai + npc_ai3_speeds + npc_target |
| 207 | PincushionZombie | 3 | `update_zombie` :6775-6888 | WIRED-DELEGATE | npc_ai + npc_ai3_speeds + npc_target |
| 208 | SlimedZombie | 3 | `update_zombie` :6775-6888 | WIRED-DELEGATE | npc_ai + npc_ai3_speeds + npc_target |
| 209 | SwampZombie | 3 | `update_zombie` :6775-6888 | WIRED-DELEGATE | npc_ai + npc_ai3_speeds + npc_target |
| 210 | TwiggyZombie | 3 | `update_zombie` :6775-6888 | WIRED-DELEGATE | npc_ai + npc_ai3_speeds + npc_target |
| 211 | FemaleZombie | 3 | `update_zombie` :6775-6888 | WIRED-DELEGATE | npc_ai + npc_ai3_speeds + npc_target |
| 212 | ZombieRaincoat | 3 | `update_zombie` :6775-6888 | WIRED-DELEGATE | npc_ai + npc_ai3_speeds + npc_target |
| 305 | Scarecrow1 | 3 | `update_skeleton` :7516-7568 | WIRED-DELEGATE | npc_ai3_speeds ladder |
| 306 | Scarecrow2 | 3 | `update_skeleton` :7516-7568 | WIRED-DELEGATE | npc_ai3_speeds ladder |
| 307 | Scarecrow3 | 3 | `update_skeleton` :7516-7568 | WIRED-DELEGATE | npc_ai3_speeds ladder |
| 308 | Scarecrow4 | 3 | `update_skeleton` :7516-7568 | WIRED-DELEGATE | npc_ai3_speeds ladder |
| 309 | Scarecrow5 | 3 | `update_skeleton` :7516-7568 | WIRED-DELEGATE | npc_ai3_speeds ladder |
| 310 | Scarecrow6 | 3 | `update_skeleton` :7516-7568 | WIRED-DELEGATE | npc_ai3_speeds ladder |
| 311 | Scarecrow7 | 3 | `update_skeleton` :7516-7568 | WIRED-DELEGATE | npc_ai3_speeds ladder |
| 312 | Scarecrow8 | 3 | `update_skeleton` :7516-7568 | WIRED-DELEGATE | npc_ai3_speeds ladder |
| 313 | Scarecrow9 | 3 | `update_skeleton` :7516-7568 | WIRED-DELEGATE | npc_ai3_speeds ladder |
| 314 | Scarecrow10 | 3 | `update_skeleton` :7516-7568 | WIRED-DELEGATE | npc_ai3_speeds ladder |
| 315 | HeadlessHorseman | 26 | `npc_ai_unicorn::unicorn_step` | WIRED-MODULE |  |
| 325 | MourningWood | 57 | `npc_ai_pumpkin_boss::mourning_wood_step` | WIRED-MODULE | via style-57 axis :3682 (kind arm unreachable) |
| 326 | Splinterling | 3 | `update_skeleton` :7516-7568 | WIRED-DELEGATE | npc_ai3_speeds ladder |
| 327 | Pumpking | 58 | `npc_ai_pumpkin_boss::pumpking_step` | WIRED-MODULE | via style-58 axis :3685 (kind arm unreachable) |
| 329 | Hellhound | 26 | `npc_ai_unicorn::unicorn_step` | WIRED-MODULE |  |
| 330 | Poltergeist | 22 | `npc_ai_poltergeist::poltergeist_ai` | WIRED-MODULE |  |
| 338 | ZombieElf | 3 | `update_skeleton` :7516-7568 | WIRED-DELEGATE | npc_ai3_speeds ladder |
| 339 | ZombieElfBeard | 3 | `update_skeleton` :7516-7568 | WIRED-DELEGATE | npc_ai3_speeds ladder |
| 340 | ZombieElfGirl | 3 | `update_skeleton` :7516-7568 | WIRED-DELEGATE | npc_ai3_speeds ladder |
| 341 | PresentMimic | 25 | `update_skeleton` :7516-7568 | STAND-IN | npc_ai3_speeds ladder; faithful npc_ai_frost_mob UNWIRED |
| 342 | GingerbreadMan | 3 | `update_skeleton` :7516-7568 | WIRED-DELEGATE | npc_ai3_speeds ladder |
| 343 | Yeti | 3 | `update_skeleton` :7516-7568 | WIRED-DELEGATE | npc_ai3_speeds ladder |
| 344 | Everscream | 57 | `update_hover` :7778-7794 | STAND-IN | npc_ai::hover_step; faithful npc_ai_frost_boss_b UNWIRED |
| 345 | IceQueen | 60 | `update_hover` :7778-7794 | STAND-IN | npc_ai::hover_step; faithful npc_ai_frost_boss UNWIRED |
| 346 | SantaNK1 | 61 | `update_hover` :7778-7794 | STAND-IN | npc_ai::hover_step; faithful npc_ai_frost_boss_b UNWIRED |
| 347 | ElfCopter | 62 | `update_hover` :7778-7794 | STAND-IN | npc_ai::hover_step; faithful npc_ai_frost_mob UNWIRED |
| 348 | Nutcracker | 3 | `update_skeleton` :7516-7568 | STAND-IN | npc_ai3_speeds ladder; faithful npc_ai_frost_mob UNWIRED |
| 349 | NutcrackerSpinning | 3 | `update_skeleton` :7516-7568 | STAND-IN | npc_ai3_speeds ladder; faithful npc_ai_frost_mob UNWIRED |
| 350 | ElfArcher | 3 | `update_skeleton` :7516-7568 | STAND-IN | npc_ai3_speeds ladder; faithful npc_ai_frost_mob_b UNWIRED |
| 351 | Krampus | 3 | `update_skeleton` :7516-7568 | WIRED-DELEGATE | npc_ai3_speeds ladder |
| 352 | Flocko | 63 | `update_hover` :7778-7794 | STAND-IN | npc_ai::hover_step; faithful npc_ai_frost_mob UNWIRED |

Roster gaps vs C# (kinds with NO NpcKind row, modules landed or styles known): DukeFishron 370 (`npc_ai_fishron` UNWIRED, no kind), PlanterasHook 263 / PlanterasTentacle 264 / Spore 265 / FungiSpore 261 (`npc_ai_style52/53/50` UNWIRED; sim drains Plantera spawn-requests to keep them from going stale, `world_sim/npcs.rs` :3654-3660), DungeonSpirit 288 (`npc_ai_style56`), CursedSkull 34 / GiantCursedSkull 289 / WaterBoltMimic 694 (`npc_ai_style10`), BlazingWheel 72 (`npc_ai_style21`), bird family 74/297/298/442/611/671-675/689 (`npc_ai_style24`), Herpling 174 / Derpling 177 / ChatteringTeethBomb 378 (`npc_ai_style41`), LostGirl 195 / Nymph 196 (`npc_ai_style42`), AngryNimbus 250 (`npc_ai_style49`), ForceBubble 384 (`npc_ai_style72`), Destroyer body/tail 135/136, Tortoise 153/154 + GiantShelly 496/497 (style-39 module covers them), Empress 636 (audit died, deferred), QueenSlime 657, plus the whole critter/pet style band 64-73/86-92/111-122/124-127 (mostly unported; census-first slices §5).

## 2. NPC aiStyle dispatch census

### 2a. The style axis (`npc.rs` :3651-3689)

| style | C# anchor (NPC.cs) | Rust arm | status |
| --- | --- | --- | --- |
| 5 | :20802-20806 → AI_005_EaterOfSouls :50244-50916 | `ai_005_hover_family` (Servant→`update_servant_of_cthulhu`; EoS/Hornet/MeteorHead→`update_hover`→`npc_ai::hover_step`+`hover_dust_wet`+`hover_flee`; Bee/BeeSmall→`update_bee_hover`; Probe→sim `update_destroyers`) | FULL-PORT-WIRED (W4-d22 style-5 census+fix: per-tick dust gates Next(20)/Next(40) :50695/:50708, Hornet stinger ladder Next(5,20)+±20 jitter pair + hostile 55 + Item17 :50754-50810, num8/flag4 ±0.023 drift :50448-50478, wet rows :50714-50753, dawn/dead flee :50906-50910, prev-tick collide-latch bounce :50656-50688, every-tick TargetClosest :50250, servant double-flap removed; 6 stream-parity witnesses written not run — ANALYSIS "The aiStyle-5 census fix (W4-d22)") |
| 9 | :21377-21571 inline | `ai_009_sphere_family` :3718-3731 → `npc_ai_style9::sphere_family_step` | FULL-PORT-WIRED (W42 ef07892e; ftw flags parked false, dust draws documented-skip) |
| 31 | :27015-27625 inline | `npc_ai_style31::spazmatism_ai` :3665-3671, CLASSIC_SOLO env | FULL-PORT-WIRED (twin-list/expert/getGood env parked) |
| 47 | :31347-31350 → AI_047_GolemFist :19231-19504 | `ai_047_golem_fist_family` :3678 → `update_golem_rng` fists arm (three-way reconcile TR-D3s48; `npc_ai_style47` stays test-only reference) | FULL-PORT-WIRED (W65+W77) |
| 57 (MourningWood only) | :32761-33043 inline | `npc_ai_pumpkin_boss::mourning_wood_step` :3682-3684 | FULL-PORT-WIRED (W98 audit faithful) |
| 58 (Pumpking only) | :33044-33197 inline | `npc_ai_pumpkin_boss::pumpking_step` :3685-3687 | FULL-PORT-WIRED (W101 audit faithful; 6 merger gaps recorded — FindFrame 327 etc, verify in slice B-4) |
| _ | — | `update_kind_ai` :3736-4251 | kind-first fallback for every other style |

### 2b. UNWIRED landed modules (30) — the wiring wave's worklist

All are `mod`-declared in `main.rs`, referenced NOWHERE outside their own file (+tests). Each header pins its C# range.

| module | ln | style / kinds (vanilla ids) | C# range (NPC.cs) | live routing today |
| --- | --- | --- | --- | --- |
| `npc_ai_caster` | 936 | 8 — 13 caster types | :20817-21376 | `update_caster` :8002-8018 delegates to `npc_ai_bat::ai_style_8_step` (WIRED but the caster module is the fuller port — reconcile, do not double-port) |
| `npc_ai_fishron` | 1601 | 69 — DukeFishron 370 | AI_069 :48774-49755 | MISSING kind; no dispatch |
| `npc_ai_frost_boss` | 815 | 60 — IceQueen 345 | :33374-33680 | `update_hover` stand-in |
| `npc_ai_frost_boss_b` | 1567 | 57-type Everscream 344 + 61 SantaNK1 346 | :32761-33043 (344 branches) / 33681-33921 | `update_hover` stand-in |
| `npc_ai_frost_mob` | 1150 | 63 Flocko 352 / 62 ElfCopter 347 / 25 PresentMimic 341 + AI_003 walk arms + Nutcracker transform | :33971-34018 / 33922-33969 / 25538-25630 | `update_hover`/`update_skeleton` stand-ins |
| `npc_ai_frost_mob_b` | 855 | 3-type ElfArcher 350 shooter | AI_003 :55856-60299 (350 rows) | `update_skeleton` stand-in |
| `npc_ai_style10` | 1042 | 10 — CursedSkull 34 / GiantCursedSkull 289 / WaterBoltMimic 694 | :21572-21918 | MISSING kinds |
| `npc_ai_style14` | 718 | 14 — bats 49/51/60/66 (4 of 17 types) | :22849-23486 | `update_cave_bat` :7743-7772 hash-jitter stand-in |
| `npc_ai_style16` | 1023 | 16 — fish 55/58/65 (13 members) | :23492-24021 | `update_swimmer` :14972-15048 approximation |
| `npc_ai_style21` | 545 | 21 — BlazingWheel 72 | :24615-24665 | MISSING kind |
| `npc_ai_style24` | 1020 | 24 — birds (11 types) | :25313-25537 | MISSING kinds |
| `npc_ai_style27` | 1252 | 27 — WallOfFlesh 113 | :25635-26052 | `update_wof` :8301-8479 defective stand-in (W96 ×14) |
| `npc_ai_style28` | 623 | 28 — WoF Eye 114 | :26053-26208 | `update_wof_eye` :13616-13734 approximation (W48) |
| `npc_ai_style29` | 732 | 29 — Hungry 115/116 | :26209-26404 | `update_hungry` :8488-8596 defective (W81 ×10) |
| `npc_ai_style33` | 891 | 33 — PrimeSaw 129 | :27943-28244 | `update_prime_limb` :13456-13615 invented-drift v1 (W82) |
| `npc_ai_style37` | 1068 | 37 — Destroyer 134/135/136 | AI_037 :49756-50243 | `TheDestroyer => {}` :3804 + worm pass; body/tail kinds missing |
| `npc_ai_style39` | 1270 | 39 — Tortoise 153/154, Shelly 496/497, Sroller 417 | :29145-29665 | `update_solar_sroller` :11520-11596 approximate (W83) |
| `npc_ai_style41` | 951 | 41 — Herpling 174 / Derpling 177 / TeethBomb 378 | :29899-30159 | MISSING kinds |
| `npc_ai_style42` | 476 | 42 — LostGirl 195 → Nymph 196 | :30160-30189 | MISSING kinds |
| `npc_ai_style43` | 1590 | 43 — QueenBee 222 | :30190-30860 | `update_queen_bee` :8019-8099 EoC-shaped v1 stand-in (W86) |
| `npc_ai_style46` | 804 | 46 — GolemHead 246 | :31129-31346 | `update_golem_rng` head arm WIRED by W77 audit — module kept as reference (reconciled, TR-D3s48) |
| `npc_ai_style49` | 580 | 49 — AngryNimbus 250 | :31587-31650 | MISSING kind |
| `npc_ai_style50` | 704 | 50 — FungiSpore 261 / Spore 265 | :31651-31716 | MISSING kinds; sim drains requests |
| `npc_ai_style52` | 925 | 52 — PlanterasHook 263 | :32076-32240 | MISSING kind; sim drains requests |
| `npc_ai_style53` | 818 | 53 — PlanterasTentacle 264 | :32241-32369 | MISSING kind; sim drains requests |
| `npc_ai_style55` | 742 | 55 — Creeper 267 | :32657-32741 | `update_creeper` :14575-14615 defective stand-in (W95 ×6, still live at c7fe175d) |
| `npc_ai_style56` | 633 | 56 — DungeonSpirit 288 | :32742-32760 | MISSING kind |
| `npc_ai_style72` | 528 | 72 — ForceBubble 384 | :35080-35101 | MISSING kind |
| `npc_ai_unicorn_b` | 564 | 26 tails — FindFrame 315/329, gallop dust, AddLight, gfxOffY, pumpkin scale | FindFrame + AI_026 tails | slice A (`npc_ai_unicorn`) WIRED; tails UNWIRED (W106) |
| `npc_ai_style47` | 417 | 47 — GolemFist 247/248 | AI_047 :19231-19504 | test-only reference after three-way reconcile (axis :3678 delegates to `update_golem_rng`) |

(30 files; `npc_ai_style46`/`47` are deliberate references post-reconcile, so the actionable wiring set is 28.)

### 2c. Per-style status table (roster styles, kind-match arms)

C# ranges are the `NPC.AI()` if-chain block (method-call arms cite the AI_ method too). "Live" = what actually runs at `c7fe175d`.

| style | C# block / method | kinds (port ids) | live routing | status |
| --- | --- | --- | --- | --- |
| 0 | :19821-19944 | 63,151,157,159,161,164 | `(empty)` | EMPTY-OK (bound; AI_000_TransformBoundNPC :44884-44895 rescue-side partial) |
| 1 | :19945-19949 → AI_001_Slimes :60341-61724 | 0,189,190,191,192,193,194,195,196,197 | `update_slime` | STAND-IN — sin-timer hop, not the C# ai[] machine (10 kinds) |
| 2 | :19950-19954 → AI_002_FloatingEye :52274-52732 | 1,2,198,199,200,201,202 | `update_eye` | STAND-IN — accel chase + sin bob vs C# day/despawn machine (7 kinds) |
| 3 | :19955-19959 → AI_003_Fighters :55856-60299 | 3,6,7,57,58,59,60,61,68,69,70,71,72,85,86,88,119,120,127,... | `npc_ai::walker_step_ladder, update_archer, update_goblin_summoner (+13)` | MIXED — zombie/skeleton on faithful speed ladder (W40 bddc7e03) + shared TargetClosest (npc_target); 62 kinds ride 16 different fns; per-family audits pending (slices B-4/B-5/B-6) |
| 4 | :19960-20801 | 42 | `update_eye_of_cthulhu` | LIVE-INLINE — EoC v1 machine :6889-7193, boss-audit deferred queue |
| 5 | :20802-20806 → AI_005_EaterOfSouls :50244-50916 | 12,13,14,45 | `update_hover, update_servant_of_cthulhu` | WIRED axis (see 2a) |
| 6 | :20807-20811 → AI_006_Worms :50963-52273 | 25,26,27,28,29,30,31,32,33,48,49,141,142,143,144 | `(empty), update_stardust_worm_head` | SIM-PASS worm chain :1055-1315; AI_006 fidelity unverified (slice A-2) |
| 7 | :20812-20816 → AI_007_TownEntities :52965-55562 | 5,34,35,36,37,38,39,40,41,64,109,111,150,156,158,160,162,... | `update_guide` | STAND-IN — 240-tick wander vs AI_007_TownEntities 2600-line machine; town wave deferred (W62 died ×2) |
| 8 | :20817-21376 | 15,16,17,18,62 | `update_caster` | WIRED-DELEGATE npc_ai_bat::ai_style_8_step (W44-family); fuller npc_ai_caster UNWIRED — reconcile |
| 9 | :21377-21571 | 19,20,21 | `npc_ai_style9::sphere_family_step` | WIRED axis (see 2a) |
| 11 | :21919-22299 | 44 | `update_skeletron` | LIVE-INLINE — update_skeletron :8100-8300 "verbatim"-claimed; distrust-verbatim rule applies (slice B-1) |
| 12 | :22300-22614 | 79 | `update_skeletron_hand` | LIVE-INLINE — update_skeletron_hand :13735-13806; hands-count ai[3] stand-in documented |
| 14 | :22849-23486 | 8,9,10,11 | `update_cave_bat` | STAND-IN — hash-jitter bat :7743-7772; faithful npc_ai_style14 UNWIRED (W44 d6bc30ed) |
| 15 | :23487-23491 → AI_015_KingSlime :43121-43415 | 4 | `update_king_slime` | LIVE-INLINE — update_king_slime :7290-7458 cites AI_015 rows; boss-audit deferred queue |
| 16 | :23492-24021 | 22,23,24 | `update_swimmer` | STAND-IN — swim_step approximation; faithful npc_ai_style16 UNWIRED (W45 8754ef9f) |
| 22 | :24666-25234 | 330 | `npc_ai_poltergeist::poltergeist_ai` | WIRED-MODULE npc_ai_poltergeist (ac9be176 + pin repair) |
| 25 | :25538-25630 | 341 | `update_skeleton` | STAND-IN — PresentMimic rides update_skeleton walker; faithful machine in npc_ai_frost_mob UNWIRED |
| 26 | :25631-25634 → AI_026_Unicorns :62366-62925 | 118,134,315,329 | `npc_ai_unicorn::unicorn_step, update_nebula_beast, update_stardust_spider_small` | WIRED-MODULE unicorn_step for 315/329 (W2 027ff3ff, verified by second instance); nebula-beast/stardust-spider-small LIVE-INLINE; unicorn_b tails UNWIRED |
| 27 | :25635-26052 | 46 | `update_wof` | STAND-IN — update_wof defective (W96 ×14 incl. every-tick eye respawn :3980-3990 STILL LIVE); faithful npc_ai_style27 UNWIRED |
| 28 | :26053-26208 | 80 | `update_wof_eye` | STAND-IN — update_wof_eye f32-band approximation; faithful npc_ai_style28 UNWIRED (W48) |
| 29 | :26209-26404 | 47 | `update_hungry` | STAND-IN — update_hungry ×10 defects (W81); faithful npc_ai_style29 UNWIRED |
| 30 | :26405-27014 | 52 | `update_retinazer` | LIVE-INLINE — update_retinazer :13118-13335; ledger flags "Retinazer-30 mislabel" for re-audit (slice B-1) |
| 31 | :27015-27625 | 53 | `npc_ai_style31::spazmatism_ai` | WIRED axis (see 2a) |
| 32 | :27626-27942 | 51 | `update_prime` | LIVE-INLINE — update_prime :12963-13117; limb-spawn drain fixed (:3622-3633 npcs.rs); audit pending |
| 33 | :27943-28244 | 82 | `update_prime_limb` | STAND-IN — update_prime_limb invented +0.07/+0.04 drift; faithful npc_ai_style33 UNWIRED (W82) |
| 34 | :28245-28521 | 83 | `update_prime_limb` | STAND-IN — same shared v1 limb fn; W109 prime-limbs port DIED (deferred-restart queue) |
| 35 | :28522-28756 | 81 | `update_prime_limb` | STAND-IN — same shared v1 limb fn; W109 deferred |
| 36 | :28757-28990 | 84 | `update_prime_limb` | STAND-IN — same shared v1 limb fn; W109 deferred |
| 37 | :28991-28994 → AI_037_Destroyer :49756-50243 | 50 | `(empty)` | STAND-IN — TheDestroyer {} + worm pass; faithful npc_ai_style37 UNWIRED (W50); body/tail kinds missing |
| 38 | :28995-29144 | 65,66,67 | `update_frost_legion` | STAND-IN — update_frost_legion :7569-7584 stub (:7570 `let _ = world`); W28 frost-legion port DIED (deferred) |
| 39 | :29145-29665 | 138 | `update_solar_sroller` | STAND-IN — update_solar_sroller invented speed/exit (W83); faithful npc_ai_style39 UNWIRED |
| 43 | :30190-30860 | 43 | `update_queen_bee` | STAND-IN — update_queen_bee EoC-shaped v1; faithful npc_ai_style43 UNWIRED (W86) |
| 45 | :31125-31128 → AI_045_Golem :19505-19781 | 73 | `update_golem_rng` | WIRED-DELEGATE update_golem_rng → npc_ai_golem (W77 55b35124, 14 witnesses) |
| 46 | :31129-31346 | 74 | `update_golem_rng` | WIRED-DELEGATE update_golem_rng head arm (W77); npc_ai_style46 reference kept |
| 47 | :31347-31350 → AI_047_GolemFist :19231-19504 | 75,76 | `update_golem_rng` | WIRED axis → update_golem_rng fists (W65+W77 reconcile) |
| 48 | :31351-31586 | 77 | `update_golem_rng` | WIRED-DELEGATE update_golem_rng free-head (W77 integer tiers) |
| 51 | :31717-32075 | 54 | `update_plantera` | WIRED-DELEGATE update_plantera → npc_ai_plantera::step + hook_anchor (W76+W92 G1-G5 fixes landed via P5; minions missing kinds) |
| 54 | :32370-32656 | 55 | `update_brain` | LIVE-INLINE — update_brain :14629-14835 with W66 ×2 surgical fixes VERIFIED landed (:14636-14642 despawn gate, :14724-14727 reciprocal-first) |
| 55 | :32657-32741 | 56 | `update_creeper` | STAND-IN — update_creeper still has W95 ×6 defects (no 700px leash, .max(1.0) floor :14603, hurt_cd vs justHit :14611); faithful npc_ai_style55 UNWIRED |
| 57 | :32761-33043 | 325,344 | `npc_ai_pumpkin_boss::mourning_wood_step, update_hover` | SPLIT — MourningWood WIRED axis; Everscream 344 update_hover STAND-IN (npc_ai_frost_boss_b UNWIRED) |
| 58 | :33044-33197 | 327 | `npc_ai_pumpkin_boss::pumpking_step` | WIRED axis Pumpking (W101 faithful; 6 merger gaps to verify) |
| 60 | :33374-33680 | 345 | `update_hover` | STAND-IN — IceQueen update_hover; npc_ai_frost_boss UNWIRED (41bc275f) |
| 61 | :33681-33921 | 346 | `update_hover` | STAND-IN — SantaNK1 update_hover; npc_ai_frost_boss_b UNWIRED (W80 fdd646fe) |
| 62 | :33922-33970 | 347 | `update_hover` | STAND-IN — ElfCopter update_hover; npc_ai_frost_mob UNWIRED (5ef9b24b); aistyle62 branch superseded (SKIP-parked) |
| 63 | :33971-34018 | 352 | `update_hover` | STAND-IN — Flocko update_hover; npc_ai_frost_mob flocko_ai audited FAITHFUL (W107 5528b7db) — pure wiring |
| 74 | :35218-35500 | 139,188 | `update_martian_drone, update_solar_corite` | LIVE-INLINE — martian_drone :8700-8865 / solar_corite :11597-11673; martian census W73 DIED (deferred) |
| 75 | :35501-36070 | 87,89,90,91,94,137 | `update_saucer_mount, update_scutlix_rider, update_solar_drakomire_rider` | LIVE-INLINE/STAND-IN — saucer_mount :9187-9440 (stand-in flags; +3 DEAD dup arms :4036-4038), scutlix_rider, drakomire_rider; martian census deferred |
| 76 | :36071-36519 | 92 | `update_saucer_core` | LIVE-INLINE — update_saucer_core :12718-12962 (cites :27626-27942 = style-32 range, citation smell — verify) |
| 77 | :36520-36523 → AI_077_MoonLordCore :41157-41576 | 124 | `update_moon_lord_core` | LIVE-INLINE — update_moon_lord_core :10786-10903; W79 moonlord audit DIED (896-ln partial backed up, deferred) |
| 78 | :36524-36527 → AI_078_MoonLordHands :41577-42083 | 123 | `update_moon_lord_hand` | LIVE-INLINE — update_moon_lord_hand :10931-11083 (cites AI_079 range — hand/head cite cross-check needed) |
| 79 | :36528-36531 → AI_079_MoonLordHead :42084-42458 | 122 | `update_moon_lord_head` | LIVE-INLINE — update_moon_lord_head :11084-11172 (cites :42459-42880 = AI_081 range — citation smell) |
| 80 | :36532-36624 | 187 | `update_martian_probe` | LIVE-INLINE — update_martian_probe :8597-8699 |
| 81 | :36625-36628 → AI_081_TrueEyeOfCthulhu :42459-42923 | 125 | `update_true_eye` | STAND-IN — update_true_eye :11173-11301 (stand-in flags; AI_081 :42459-42923) |
| 82 | :36629-36756 | 126 | `update_moon_leech_blob` | LIVE-INLINE — update_moon_leech_blob :12543-12552 (10-line stub) |
| 84 | :36926-36929 → AI_084_LunaticCultist :64414-65389 | 112 | `update_lunatic_cultist` | LIVE-INLINE — update_lunatic_cultist :7946-7980 (35 lines vs AI_084 976-line machine — high-risk stub) |
| 85 | :36930-37220 | 121,131 | `update_brain_suckler, update_stardust_cell_big` | LIVE-INLINE — brain_suckler :11827-11926 / stardust_cell_big :10448-10539 |
| 93 | :38858-38967 | 93 | `update_flying_dutchman` | LIVE-INLINE — update_flying_dutchman :9441-9525 |
| 94 | :38968-39383 | 113,114,115,146 | `update_ancient_cultist` | LIVE-INLINE — update_ancient_cultist :12553-12607 serves 4 towers (fn-name smell: towers ride a cultist-named fn) |
| 95 | :39384-39429 | 116 | `update_stardust_cell` | LIVE-INLINE — update_stardust_cell :10363-10392 |
| 96 | :39430-39472 | 117 | `update_stardust_jellyfish` | LIVE-INLINE — update_stardust_jellyfish :10393-10447 |
| 97 | :39473-39619 | 130 | `update_nebula_floater` | LIVE-INLINE — update_nebula_floater :11734-11826 |
| 105 | :40728-40969 | 110 | `update_eternia_crystal` | LIVE-INLINE — update_eternia_crystal :10287-10362 |
| 106 | :40970-41047 | 147 | `update_dd2_lane` | LIVE-INLINE — update_dd2_lane :10731-10785 (cites AI_077 range — citation smell) |
| 107 | :41048-41051 → AI_107_ImprovedWalkers :62926-64413 | 95,96,97,98,99,100,101,102,103,104,105,108,173,174,175,17... | `update_dd2_bomber, update_dd2_goblin, update_dd2_kobold_walker (+6)` | LIVE-INLINE — 8 dd2 fns + wither_beast stand-in flag; AI_107 :62926-64413 |
| 108 | :41052-41055 → AI_108_DivingFlyer :65390-65790 | 169,170,171,178,179 | `update_diving_flyer` | LIVE-INLINE — update_diving_flyer :12203-12333 (5 kinds) |
| 109 | :41056-41059 → AI_109_DarkMage :65791-66187 | 106,172 | `update_dark_mage` | LIVE-INLINE — update_dark_mage :9656-10006 |
| 110 | :41060-41063 → AI_110_Betsy :61850-62307 | 107 | `update_betsy` | STAND-IN flag — update_betsy :10007-10286 ("stand-in"+"verbatim" both present; audit slice B-5) |
| 123 | :41112-41115 → AI_123_Deerclops :44017-44408 (+helpers :44448-44727) | 78 | `update_deerclops` | LIVE-INLINE — update_deerclops :13807-13980 (classic-only fold-outs documented) |

### 2d. Dead duplicate kind arms (compiler-warned, first arm wins)

- `DD2Bartender`: :3773 (town group → `update_guide`) vs :4033 (dead).
- `MartianSaucer | MartianSaucerTurret | MartianSaucerCannon`: :3888-3890 (live) vs :4036-4038 (dead). Same target today, but a wiring edit to the dead arm would silently no-op — the wiring wave must edit the FIRST arm.
- `npc/data.rs::to_u16` carries `#[deny(unreachable_patterns)]` after the 60-dead-arm incident (S1-F-1); `update_kind_ai` has no such guard — recommend adding it in the wiring wave.


## 3. Projectile aiStyle census

Dispatch: `ProjectilePool::update` (`projectile.rs` :12935+), per-slot sub-step loop; the style match is `match data.ai_style` **:13064-15109** (72 arm entries, 65 unique live styles + `_ => {}`), gated by `proj_data(kind)` (:13063). Pre-AI: out-of-world guard :12989-13008 (aiStyle-3 exempt), minion/sentry block :13012-13014. Post-AI: wind tail :15114-15116 (recorded no-wind deviation for old arms), `damage_pve` :15136, hostile contact :15145-15150, per-sub-step lifetime :15162-15169. C# chain: `Projectile.AI()` :23199, if-chain :23201-36806 (210 styles).

### 3a. Live arms (style → Rust arm lines → C#)

| style | Rust arm (projectile.rs) | serving fn(s) | C# dispatch / method | data kinds | status |
| --- | --- | --- | --- | --- | --- |
| 1 | :13075-13099 | `ai_book_of_skulls` :5355-5466 + `ai_style1_rider` :5279-5354 + `ai_arrow` :4353-4362 | AI_001 :54002-57795 | 1,2,14,20,81,83 (+41) | LIVE — kind-gated riders + slice A (259/270/325/389/436/438) + slice B (9 kinds) + generic arrow fallback; AI_001 3800-line machine only partially covered |
| 3 | :13100-13105 | `ai_boomerang` :4418-4512 | AI_003_Boomerang :38866-39640 | 6,52,113 | LIVE |
| 40 | :13106-13111 | `ai_leaf_blower` :5827-5903 | AI_040_LeafBlower :38003-38075 | 206 | LIVE |
| 7 | :13112-13195 | `ai_hook` :2012-2037 + `center` | AI_007_GrapplingHooks :52565-52867 | 13,32,73,74,165,230 (+26) | LIVE |
| 14 | :13196-13196 | `ai_spear` :4363-4417 | inline :24830-25307 + AI_019_Spears :45857-46074 | 186,277,326,327,328,378 (+2) | LIVE |
| 16 | :13197-13197 | `ai_bomb` :5134-5242 | AI_016_Bombs :46904-47539 | 28,29,37,102,108,144 (+7) | LIVE |
| 18 | :13198-13201 | `ai_demon_scythe` :5572-5602 | inline :25367-25476 (demon scythe) | 45 | LIVE |
| 22 | :13202-13210 | `ai_ice_block` :5603-5741 | inline :25607-25860 (ice block) | 80 | LIVE |
| 10 | :13211-13215 | `ai_dungeon_debris` :9944-10081 | AI_010 :57826-58101 | 42,65,68,354,736,737 (+1) | LIVE |
| 8 | :13216-13216 | `ai_ball_of_fire` :5467-5571 | inline :24233-24362 (Book of Skulls) | 15,27,95,96,253,258 | LIVE |
| 24 | :13217-13217 | `ai_crystal_storm` :5778-5826 | inline :25936-26014 (crystal storm) | 94,1134 | LIVE |
| 28 | :13218-13218 | `ai_frost_bolt` :5904-5921 | inline :26555-26675 (frost bolt) | 309,359,967,1026 | LIVE |
| 50 | :13219-13224 | `ai_inferno_fork` :5922-5964 | inline :28323-28401 (inferno fork hostile) | 295,296 | LIVE |
| 27 | :13225-13229 | `ai_unholy_trident` :3356-3440 | inline :26420-26554 (unholy trident/inferno fork) | 114,115,116,132,156,157 (+1) | LIVE |
| 51 | :13230-13235 | `ai_spectre_shot` :3496-3599 | inline :28402-28503 (spectre shot) | 293,297 | LIVE |
| 59 | :13236-13239 | `ai_spectre_wrath` :3741-3825 | inline :28938-28993 (spectre wrath) | 356 | LIVE |
| 157 | :13240-13247 | `ai_sharp_tears` :9877-9943 | AI_157_SharpTears :51358-51495 | 961 | LIVE |
| 2 | :13248-13254 | `ai_style2` :2085-2424 | inline :23205-23689 | 3,48,166,240,954 | LIVE |
| 4 | :13255-13258 | `vilethorn_step` :15517-15529 | inline :23694-23829 (vilethorn) | 7,8 | LIVE |
| 79 | :13259-13261 | `ai_saucer_deathray` :10117-10215 | inline :30791-30910 (saucer deathray) | 447 | LIVE |
| 80 | :13262-13265 | `ai_saucer_missile` :10216-10285 | inline :30911-31000 (saucer missile) | 448 | LIVE |
| 70 | :13266-13268 | `ai_flairon_bubble` :8237-8328 | inline :30304-30354 (flairon bubble) | 405 | LIVE |
| 133 | :13269-13272 | `ai_dark_mage_raise` :8329-8373 | inline :36435-36498 (dark mage raise) | 673 | LIVE |
| 187 | :13273-13276 | `ai_insanity_shadow` :8374-8478 | AI_187_ShadowHand :42502-42630 | 965 | LIVE |
| 188 | :13277-13280 | `ai_lights_bane` :8479-8509 | AI_188_LightsBane :42229-42263 | 974 | LIVE |
| 189 | :13281-13283 | `ai_volcano` :8510-8551 | AI_189_Volcano :42264-42371 | 1125 | LIVE |
| 71 | :13284-13287 | `ai_typhoon` :3600-3712 | inline :30355-30454 (typhoon) | 409 | LIVE |
| 23 | :13288-13293 | `ai_eye_fire` :8552-8589 | inline :25861-25935 (eye fire/flames trap) | 101,188 | LIVE |
| 158 | :13294-13311 | `ai_baby_bird` :8590-8740 | AI_158_BabyBird :48757-48989 | 759 | LIVE |
| 67 | :13312-13358 | `ai_freaking_pirates` :8792-9540 + `pirate_movement` :9573-9667 + `ai_freaking_pirates_pets` | AI_067_FreakingPirates :49121-51202 | 393,394,395,500,758,833 (+4) | LIVE — W36 slice B pets landed; 653/1090 deferred |
| 118 | :13359-13378 | `ai_nebula_arcanum` :3826-3988 | inline :34638-34808 (nebula arcanum) | 617 | LIVE |
| 119 | :13379-13387 | `ai_nebula_arm` :3989-4079 | inline :34809-34887 (nebula arm) | 618 | LIVE |
| 75 | :13388-13414 | inline arm | AI_075 :66130-67379 (last prism) | 615,630,633,705,714 | LIVE |
| 29 | :13415-13420 | `ai_gem_staff` :6190-6275 | AI_029_GemStaffs :37643-38002 | 121,122,123,124,125,126 (+1) | LIVE |
| 9 | :13421-13424 | `ai_magic_missile` :5965-6189 | AI_009_MagicMissiles :43195-43426 | 16,34,79 | LIVE |
| 47 | :13425-13426 | `magnet_sphere_step` :15530-15562 | AI_047_MagnetSphere :41403-41500 | 254 | LIVE |
| 48 | :13427-13429 | `ai_magnet_bolt` :6522-6556 | inline :28204-28242 (magnet bolt) | 255 | LIVE |
| 129 | :13430-13432 | `ai_spirit_flame` :6557-6687 | inline :36198-36309 (spirit flame) | 659 | LIVE |
| 115 | :13437-13437 | `ai_yoyo_beam` :2866-2903 | inline :34414-34437 (yoyo beam/counterweight) | 604 | LIVE |
| 210 | :13438-13438 | `ai_yoyo_shot` :2831-2865 | AI_210_YoyoShots :36809-36902 | 1130,1135 | LIVE |
| 82 | :13439-13445 | `ai_phasmal_eye` :2904-2956 | inline :31214-31321 (phasmal eye) | 452 | LIVE |
| 83 | :13446-13452 | `ai_phasmal_sphere` :2971-3009 | inline :31322-31376 (phasmal sphere) | 454 | LIVE |
| 84 | :13453-13487 | `ai_moonlord_turret_laser` :7629-7722 + `ai_phasmal_deathray` :3010-3083 | inline :31377-31757 (phasmal deathray/turret laser) | 455,632,642 | LIVE |
| 85 | :13488-13492 | `ai_moon_leech` :3084-3141 | inline :31758-31826 (moon leech) | 456 | LIVE |
| 102 | :13493-13497 | `ai_nebula_eye` :3288-3316 | inline :33222-33447 (nebula eye) | 574 | LIVE |
| 103 | :13498-13501 | `ai_stardust_mark` :3262-3287 | inline :33448-33477 (stardust mark) | 540 | LIVE |
| 122 | :13502-13521 | inline arm | inline :34896-34965 (tower damage bolt) | 629 | LIVE |
| 88 | :13522-13527 | `ai_vortex_bolt` :3212-3261 | inline :31937-32261 (vortex bolt) | 580 | LIVE |
| 108 | :13528-13535 | `ai_vortex_ritual` :3142-3211 | inline :33679-34025 (vortex ritual) | 578,579 | LIVE |
| 99 | :13536-13578 | `ai_yoyo` :2476-2830 | AI_099_2_Yoyos :67733-68106 | 534,541,542,543,544,545 (+15) | LIVE |
| 165 | :13579-13586 | `ai_whip` :4513-4570 | AI_165_Whip :47667-48271 | 841,847,848,849,912,913 (+12) | LIVE |
| 15 | :13587-13633 | `ai_flail` :4609-4865 | AI_015_Flails :43724-44170 | 25,26,35,63,154,247 (+4) | LIVE — flail machine + hydra sentry share the arm (kind switch) |
| 53 | :13634-13807 | `sentry_hydra_step` :6712-6732 | AI_053_FrostHydra :37266-37491 | 308,377,966 | LIVE — arm at :13634 has 48-space indent (merge artifact, sits between 15 and 197 arms); covers FrostHydra 308/QueenSpider 377/Houndius 966 |
| 197 | :13808-13962 | `center` + `sentry_turret_step` :6733-6771 | AI_197_CeilingAndHoverTurret :40307-40465 | 1025 | LIVE |
| 130 | :13963-14222 | `center` + `can_get_picked_up_by_sentry_backpack` + `dd2_sentry_step` :6772-6798 | AI_130_FlameBurstTower :68107-68408 | 663,665,667 | LIVE |
| 131 | :14223-14278 | `kill_step` :7810-7830 | AI_131_FlameBurstShot :68456-68509 | 664,666,668 | LIVE |
| 123 | :14279-14362 | `ai_floating_turrets` :7358-7628 | AI_123_FloatingTurrets :37051-37265 | 641,643 | LIVE |
| 112 | :14363-14386 | `ai_rainbow_crystal_shard` :7723-7809 | inline :34114-34363 (rainbow crystal shard) | 644 | LIVE |
| 134 | :14387-14564 | `center` + `can_get_picked_up_by_sentry_backpack` + `dd2_sentry_step` :6772-6798 | AI_134_Ballista :68510-68652 | 677,678,679 | LIVE |
| 137 | :14565-14732 | `can_get_picked_up_by_sentry_backpack` + `dd2_sentry_step` :6772-6798 | AI_137_LightningAura :68854-69024 | 688,689,690 | LIVE |
| 138 | :14733-14877 | `dd2_sentry_step` :6772-6798 | AI_138_ExplosiveTrap :69301-69375 | 691,692,693 | LIVE |
| 139 | :14878-14917 | inline arm | AI_139_ExplosiveTrapExplosion :69390-69415 | 694,695,696 | LIVE |
| 62 | :14919-14976 | `minion_separation` :772-822 + `ai_flying_minion` :823-1437 | AI_062 :65038-66129 (flying minion) | 373,375,407,423,613,963 | LIVE |
| 66 | :14977-15031 | `twins_separation` :1438-1478 + `ai_twins_minion` :1479-1862 | inline :29604-30161 (twins minion) | 387,388,533 | LIVE |
| 26 | :15090-15107 | inline arm | AI_026 :58102-65024 | 175,197,198,199,211,266 (+11) | LIVE — slices A+B chained (14 flyer pets + 885/889 boss-head pets); rest of AI_026 (ground pets slice C) NOT covered — non-slice kinds no-op |

### 3b. Dead duplicate arms (unreachable — first arm wins)

| style | dead arm line | live arm | risk |
| --- | --- | --- | --- |
| 1 | :13433 | :13075-13099 | dead plain-arrow arm; edits there silently no-op |
| 3 | :13434 | :13100-13105 | same |
| 14 | :13435 | :13196 | same |
| 16 | :13436 | :13197 | same |
| 115 | :14918 | :13437 | same |
| 99 | :15032-15074 | :13536-13578 | **full 43-line duplicate yoyo machine** (flag/sweep/beam) — W67 already flagged "duplicate unreachable 82 arm" class; delete in wiring wave, add `#[deny(unreachable_patterns)]` |

### 3c. Data rows with NO dispatch arm (spawnable but AI-inert)

| style | kinds (proj ids) | C# method | impact |
| --- | --- | --- | --- |
| 31 | 145 PureSpray (pure spray/green spray solution) | inline :26786+ (spray family) | spray projectiles never convert tiles |
| 120 | 623 StardustGuardian | AI_120_StardustGuardian :44634-44983 | summon inert |
| 147 | 718 Celeb2RocketExplosiveLarge | AI_147_Celeb2Rocket :52881-53093 + AI_147_Explosion :53130-53400 | Celebration Mk2 rockets inert |
| 181 | 932 FairyQueenRangedItemShot | AI_181_FairyQueenRangedItemShot :43178-43194 | Empress drop-shot inert |
| 193 | 85 Flames (Flamethrower) | AI_193_Flamethrower :41337-41402 | documented at :20366 ("aiStyle 193 unported") — flamethrower flames fly generic/no AI |
| 194 | 1001 HorsemanPumpkin | AI_194_HorsemanPumpkin :41293-41336 | **spawned by WIRED unicorn machine** (`npc_ai_unicorn.rs` :137, scale roll W106) but the projectile has NO AI arm — lobbed pumpkins never home/explode. Highest-priority projectile gap: a live feature spawns a dead projectile. |

Also: `TRAP_SPIKY_BALL` 185 data row says aiStyle **1** but C# SetDefaults :2717-2727 says **14** (W67 finding, still present in `proj_data` at c7fe175d — W99 projdata-fix worker DIED, deferred-restart queue). Verify + fix in slice P-1.

### 3d. Coverage vs C#

- C# AI() chains **210** styles; the port arms **65**. Of the 145 unarmed styles, most belong to kinds with no `proj_data` row (never spawnable in the port — not defects). The actionable set is §3c (6 styles with live data rows) + the 185-row mislabel.
- `proj_data` resolves **307 kinds / 71 styles** (249 match arms); every data style except §3c has an arm.
- AI_001 (arrow family) is a 3 794-line type-switch; slices A+B cover 21 kinds + riders; the generic `ai_arrow` fallback (:4353-4362, cites :25143-25158) serves the rest — per-kind fidelity audits are slices P-2/P-3.
- AI_026 (pets) :58102-65024 is a 6 922-line type-switch; slices A+B cover 16 kinds; ground-pet slice C deferred (ledger board D).


## 4. Defect inventory (session audits, verified against c7fe175d where possible)

Ledger tally: **W66 ×2 + W81 ×10 + W95 ×6 + W96 ×14 = 32 real defects in "faithful/verbatim"-claimed landed arms**, plus Plantera/Fishron wholesale stand-ins and W92's 5 gaps. Status column = what THIS census verified at `c7fe175d`.

### 4a. Fixed-and-landed (verified in live code)

| source | defect | live evidence at c7fe175d |
| --- | --- | --- |
| W66 (f945e33a) style-54 Brain ×2 | missing 6000px Manhattan despawn gate (:32390-32393 strict >) | FIXED — `npc.rs` :14636-14642 gate present with audit comment |
| W66 | f32 reciprocal-first order (:32549-32551, 1-ulp) | FIXED — :14724-14727 `let k = speed / d` with cite |
| W77 (55b35124) golem v1 | swapped fist sides, flipped rotations, invented head tether, RAW-delta fireballs, flat-average beam triggers, hardcoded num=1 no enrage, body never fell | FIXED — `update_golem_rng` :14017-14574 delegates to `npc_ai_golem` cores; temple gate reads `downed_golem` |
| W65 (7482015d) style-47 fists | state-0 rotation arms SWAPPED; dash aims from stale pre-snap anchor | FIXED — axis claim :3678 → golem-audit fists (three-way reconcile TR-D3s48) |
| W2 (027ff3ff) unicorn | stuck latch after AI, flag7 fold, test geometry | FIXED + independently re-verified (second instance) |
| s52 gate (MERGER-L7) | dead-x4 test premise: witness expected 12, C# :32090 last operator non-short-circuit `\|` → flag58 always set → x4=24 | FIXED — port faithful, witness corrected to 24 (LEAD-S52-DIAG); module UNWIRED awaiting kind 263 |
| W40 (bddc7e03) style-3 speeds | ~155 ladder types ported 1:1 :57403-58401, WIRED 10 sites | LANDED (`npc_ai3_speeds.rs` 1244 ln) |
| prime limb-spawn | v1 set `wants_limbs` but never drained — no limb ever existed outside direct-call tests | FIXED — `world_sim/npcs.rs` :3622-3633 drain |

### 4b. Audited, faithful re-port landed but UNWIRED — defective stand-in STILL LIVE

These are the wiring wave's highest-value targets: the fix exists on main, the bug still runs.

| source | defects in live arm | faithful module (UNWIRED) | live arm at c7fe175d |
| --- | --- | --- | --- |
| W96 (b1f2e8a3) WoF-27 **×14** | **CATASTROPHIC every-tick respawn**: sim spawns 2 eyes + 11 Hungries EVERY tick unconditionally (eyes never gated at all; `wants_hungry` never cleared); roar-RNG skip desyncs Main.rand; missing band clamps/160px/TargetClosest_WOF/expert ladder/getGood demon/INT Y pin/Bottom placement | `npc_ai_style27` (1252 ln, 15 witnesses) | VERIFIED STILL LIVE — `world_sim/npcs.rs` :3980-3990 eye spawn loop ungated; :3991-3999 `if wof.wants_hungry` never resets the flag (`npc.rs` :8474 sets it once, no clearer anywhere) |
| W81 (f1701eb0) Hungry-29 **×10** | wall-death-before-justHit order, no TargetClosest, missing INTEGER half-width anchor (~50px), f32-vs-f64 hp ladder, no per-tick damage reset, no difficulty scaling, INVENTED defense clear, missing negative-ai[1] reset, no spriteDirection/+3.14f/dead-centre hold, float-vs-int centre halves, no expert arms | `npc_ai_style29` (732 ln) | `update_hungry` :8488-8596 still the v1 |
| W95 (32e4100e+cd4a55a6) Creeper-55 **×6** | missing 700px leash (lunges forever), missing TargetClosest, invented `.max(1.0)` floor (C# unguarded NaN), f32 `dx/d*8` vs reciprocal-first `dx*(8/d)`, hurt_cd vs one-tick justHit, missing netUpdate/expert/ftw/kb-0 | `npc_ai_style55` (742 ln) | VERIFIED STILL LIVE — `update_creeper` :14575-14615: no leash, `.max(1.0)` at :14603, `hurt_cd` at :14611 |
| W48 (040df4c5) WoF-Eye-28 | f32 band midpoint vs C# INTEGER, ai[] vs localAI, missing rotation/expert arms | `npc_ai_style28` (623 ln) | `update_wof_eye` :13616-13734 still v1 |
| W82 (e5faaf42) PrimeSaw-33 | INVENTED +0.07/+0.04 drift, no state machine/hysteresis, dock float-vs-INT half-width | `npc_ai_style33` (891 ln) | `update_prime_limb` :13456-13615 still shared v1 (all 4 limbs) |
| W83 (634af5e8) roller-39 | invented 5+ai[2]*0.6 speed, 60*ai[2] exit, +0.5 wind-up vs C# ++, no collapse/restore, vx*0.05 vs ±/10 clamp | `npc_ai_style39` (1270 ln) | `update_solar_sroller` :11520-11596 still v1 |
| W86 (a616788f) QueenBee-43 | entire arm EoC-shaped stand-in (invented cycle); real machine states 5/-1/0/2/1/3/4 + expert ladders + RNG (Next(3) do-while, Next(20)/tick, gated Next(210,212)) | `npc_ai_style43` (1590 ln) | `update_queen_bee` :8019-8099 still v1 |
| W44 (d6bc30ed) bat-14 | missing ungated second generic pass :23265-23331 (dry-bat 0.2/tick accel); RNG X-before-Y shot + short-circuit Next(300) | `npc_ai_style14` (718 ln) | `update_cave_bat` :7743-7772 still hash-jitter stand-in |
| W45 (8754ef9f) fish-16 | Pufferfish puff latch, Dolphin leap (128px CanHitLine/300-tick dive), Arapaima 7/4 caps + 157 damp exemption, Orca π-fold rotation, Dolphin Next(300,1200) BEFORE ++ai[3], flop triple order | `npc_ai_style16` (1023 ln) | `update_swimmer` :14972-15048 approximation |
| W50 (be63c688) Destroyer-37 | head rides worm stand-in; real machine: Next(4)+Next(1400,26000) EVERY tick, 4×Next(-20,21) inside CanHit, 42-step fade, muzzle float-X/INT-Y, +1.57f literal, integer spacing, 1000px dive, daytime double-sink | `npc_ai_style37` (1068 ln) | `TheDestroyer => {}` :3804 + worm pass; body/tail kinds absent |
| W76+W92 Plantera-51 | wholesale INVENTED EoC-shaped 480-tick/7.5px cycle → faithful core landed WIRED; W92 found 5 gaps in sibling (atan2 swapped-arg witness, spore lift gate INVERTED, gore 6-of-12 draws, first-tick-NaN doc, f32 sqrt seam) — G1-G5 fix wave landed via P5 | `npc_ai_plantera` WIRED :13415 | minions 263/264/265/261 still MISSING kinds; modules `npc_ai_style52/53/50` UNWIRED; sim drains requests :3654-3660; world-bottom despawn tail kept as recorded deviation :13419-13421 |
| W78 (ad41a646) Fishron-69 | ENTIRELY MISSING boss (census-confirmed) → whole AI_069 ported | `npc_ai_fishron` (1601 ln, 22 tests) | still no NpcKind row, no dispatch |
| W80 (fdd646fe) + 41bc275f frost bosses | Everscream/SantaNK1/IceQueen machines | `npc_ai_frost_boss(_b)` | kinds 344/345/346 ride `update_hover` stand-in |
| 5ef9b24b + W-elf frost mobs | Flocko (W107 audit FAITHFUL), ElfCopter, PresentMimic, Nutcracker transform, ElfArcher (mob_b) | `npc_ai_frost_mob(_b)` | 341/347/348/349/350/352 ride walker/hover stand-ins |
| W106 (90aeff6c) unicorn tails | FindFrame 315/329, gallop dust in-step (Next(3) always), AddLight, gfxOffY decay, proj-1001 scale Next(51) | `npc_ai_unicorn_b` (564 ln) | tails unwired; **proj 1001 has NO AI arm (§3c) — the lobbed pumpkin is inert** |
| W93/W94/W91 plantera minions | Hook X/X client-latch quirk, int lifeMax ladder, always-drawn jitters; Tentacle ftw Next(10) BEFORE orphan return, rearm triple BEFORE anchor-death; Spore contact-death early return + expert double-damp | `npc_ai_style52/53/50` | no kinds; s52 dead-x4 witness fixed to 24 at merge |

### 4c. Open defects found by THIS census (new)

1. **WoF eye/Hungry respawn loop still live** (§4b row 1) — the single worst live defect; every tick with a living WoF pushes 2 eyes (+11 Hungries once, but `wants_hungry` never clears so any re-set respawns). Fix = wire `npc_ai_style27` + gate the sim spawn block on the module's one-shot init.
2. **6 dead duplicate projectile arms** (§3b) incl. the full 43-line yoyo machine at :15032-15074.
3. **4 dead duplicate kind arms** (§2d).
4. **proj 185 TrapSpikyBall aiStyle 1 vs C# 14** (SetDefaults :2717-2727 verified) — W99 fix worker died; still wrong.
5. **6 data-rowed projectile styles with no arm** (§3c), notably 194 HorsemanPumpkin spawned by a WIRED NPC machine.
6. **Citation smells** (wrong-range cites suggest copied scaffolding, verify in slices): `update_saucer_core` cites :27626-27942 (style-32 Prime range) for a style-76 machine; `update_moon_lord_hand` cites AI_079's range; `update_moon_lord_head` cites AI_081's range; `update_dd2_lane` cites AI_077's range; `update_ancient_cultist` serves the 4 towers (style 94) under a cultist name.
7. **`update_kind_ai` lacks `#[deny(unreachable_patterns)]`** — the guard that saved `to_u16` after S1-F-1.

### 4d. Deferred-restart queue (work died, partials safe — from ledger)

D3moon moonlord audit (896-ln partial + 135KB patch backup) > W109 Prime limbs 34/35/36 > W99 projdata fixes > W117 style-73 (×2 deaths) > W120 style-76 > W103 style-61 > W115 style-71 > W28 frost-legion > W73 martian census > W62 town AI (×2) > W105 empress audit > W53 proj-sounds > W52 NPC-sounds. Boss audits queued: Retinazer-30 mislabel, EoC, KingSlime, QueenSlime, Deerclops, Cultist.

## 5. Proposed audit slices — next wave (~12 min each, code-only verify + WIRE)

Priority order. Each slice: read the C# range WHOLE, diff the landed module (or live arm), then claim dispatch (style axis or kind arm — FIRST arm, not the dead dup) + sim-side gates; witnesses written-not-run per code-only mode. All slices disjoint by kind/style; hot-file edits (`npc.rs` axis, `world_sim/npcs.rs`) serialize through the merger.

| # | slice | scope (port kinds / proj styles) | C# ranges (exact) | job | risk |
| --- | --- | --- | --- | --- | --- |
| W-1 | **WoF family wiring** | kinds 46/47/80 + sim spawn block | NPC.cs :25635-26052 (27), :26053-26208 (28), :26209-26404 (29) | WIRE `npc_ai_style27/28/29`; replace `update_wof/_eye/_hungry`; gate+clear the sim's eye/Hungry init (:3943-3999) on the module's one-shot; restore roar RNG draws | **P0 — catastrophic live respawn loop + 30 audited defects** |
| W-2 | **Creeper/Brain verify+wire** | kind 56 (55 kind 54 already fixed) | :32657-32741 (55); re-verify :32370-32656 fixes | WIRE `npc_ai_style55` over `update_creeper`; A/B the W66 fixes | P0 — 6 live defects |
| W-3 | **QueenBee wiring** | kind 43 | :30190-30860 | WIRE `npc_ai_style43`; RNG replay witnesses (Next(3) do-while etc.) | P1 — boss, wholesale stand-in |
| W-4 | **Prime limbs** | kinds 81/82/83/84 | :27943-28244 (33), :28245-28521 (34), :28522-28756 (35), :28757-28990 (36) | WIRE style33 module; PORT 34/35/36 (W109 died — check worktree partial first) | P1 — 4 boss limbs on invented v1 |
| W-5 | **Destroyer** | kind 50 + new body/tail kinds | AI_037 :49756-50243, dispatch :28991-28994 | WIRE `npc_ai_style37`; add 135/136 kinds; RNG every-tick draws | P1 — boss on worm stand-in |
| W-6 | **Frost bosses wiring** | kinds 344/345/346 | :32761-33043 (344 branches), :33374-33680 (60), :33681-33921 (61) | WIRE `npc_ai_frost_boss(_b)`; Everscream ALWAYS Next(1,3) idle draw | P1 — 3 bosses on hover stand-in |
| W-7 | **Frost mobs wiring** | kinds 341/347/348/349/350/352 | :25538-25630 (25), :33922-33969 (62), :33971-34018 (63), AI_003 350-rows :55856-60299 | WIRE `npc_ai_frost_mob(_b)`; Nutcracker transform; Flocko already audited FAITHFUL | P1 — 6 kinds, mostly pure wiring |
| W-8 | **Plantera minions** | new kinds 261/263/264/265 | :31651-31716 (50), :32076-32240 (52), :32241-32369 (53) | add kinds + WIRE `npc_ai_style50/52/53`; replace sim request-drain :3654-3660 with real spawns; hook centroid seam already landed | P1 — boss fight incomplete |
| W-9 | **Fishron** | new kind 370 | AI_069 :48774-49755 | add kind + WIRE `npc_ai_fishron`; 371 sharknado scale draw rides spawn | P1 — missing boss |
| W-10 | **bat/fish/caster reconcile** | kinds 8-11, 22-24, 15-18+62 | :22849-23486 (14), :23492-24021 (16), :20817-21376 (8) | WIRE style14/16; reconcile `npc_ai_caster` vs live `npc_ai_bat::ai_style_8_step` (do NOT double-port) | P2 |
| W-11 | **roller + unicorn tails** | kind 138 + 315/329 tails | :29145-29665 (39); AI_026 tails + FindFrame | WIRE `npc_ai_style39` + `npc_ai_unicorn_b`; **add proj-1001 AI_194 arm :41293-41336** | P2 |
| W-12 | **small-kind landing batch** | new kinds 34/289/694, 72, 174/177/378, 195/196, 250, 288, 384 | :21572-21918 (10), :24615-24665 (21), :29899-30159 (41), :30160-30189 (42), :31587-31650 (49), :32742-32760 (56), :35080-35101 (72) | add kinds + WIRE the 7 single-family modules (all landed, audited) | P2 — pure wiring, 7 modules |
| B-1 | **Twins/Skeletron boss audit** | kinds 44/79/51/52/53 | :21919-22299 (11), :22300-22614 (12), :26405-27014 (30), :27015-27625 (31 env), :27626-27942 (32) | verify "verbatim" claims (distrust rule); Retinazer-30 mislabel check; hands-count ai[3] stand-in | P2 audit |
| B-2 | **MoonLord audit resume** | kinds 122/123/124/125/126 | AI_077 :41157-41576, AI_078 :41577-42083, AI_079 :42084-42458, AI_081 :42459-42923, :36629-36756 (82) | resume D3moon partial (896-ln backup); fix the hand/head citation cross-swap; leech-blob 10-line stub | P2 audit |
| B-3 | **Cultist/Deerclops/EoC/KingSlime audit** | kinds 112/78/42/4 | AI_084 :64414-65389, AI_123 :44017-44727, :19960-20801 (4), AI_015 :43087-43415 | 35-line cultist stub vs 976-line machine = highest gap; boss-audit deferred queue | P2 audit |
| B-4 | **Pumpkin gaps + martian census** | kinds 325/327 gaps; 85-94/187/188 | :33044-33197 gaps (FindFrame 327, rotation whitelist, blade 328/scythe 329, :33059 light, cloak ghosts 18 GLOBAL draws/frame); :35218-36519 (74/75/76), :36532-36624 (80) | W101's 6 merger gaps; resume W73 martian census; saucer dup-arm cleanup | P2 |
| B-5 | **DD2 family audit** | kinds 95-110/147/169-181 | AI_107 :62926-64413, AI_108 :65390-65790, AI_109 :65791-66187, AI_110 :61850-62307, :40728-41047 (105/106) | verify 8 dd2 fns + betsy stand-in flag + wither_beast; lane/crystal citation smells | P3 |
| B-6 | **pillars audit** | kinds 113-146 towers/mobs | :38968-39619 (94/95/96/97), :36930-37220 (85), style-3 pillar rows in AI_003 | tower fn-name smell; 4 towers one fn; per-mob type rows | P3 |
| A-1 | **slime/eye families** | kinds 0/189-197, 1/2/198-202 | AI_001_Slimes :60341-61849, AI_002 :52274-52732 | replace sin-timer stand-ins with the real ai[] machines (17 kinds, one slice each style) | P2 — most-spawned mobs |
| A-2 | **worm pass fidelity** | 14 worm kinds + 144 | AI_006 :50963-52273 | diff `update_worms_in` :1055-1315 vs C# (spawn counts, chain move, head dig) | P2 |
| A-3 | **town AI** | 22 style-7 kinds | AI_007 :52765-55786 (whole helper set) | resume W62 (died ×2); housing/shimmer/teleport already partly in npcs.rs — census first | P3 — big machine, slice into 3 |
| P-1 | **projdata fixes + dead-arm purge** | proj rows 185 + 6 dup arms + 4 kind dups | Projectile.cs :2717-2727 | resume W99; fix 185→14; delete dead arms; add deny(unreachable_patterns) to both dispatches | P1 — small, unblocks clean wiring |
| P-2 | **AI_001 coverage census** | style-1 kinds beyond slices A/B | AI_001 :54002-57795 | enumerate which rowed kinds still ride generic `ai_arrow`; slice C/D proposals | P2 |
| P-3 | **unarmed data styles** | proj styles 31/120/147/181/193 (+194 in W-11) | :26786+ (31), AI_120 :44634-44983, AI_147 :52881-53400, AI_181 :43178-43194, AI_193 :41337-41402 | port 5 small arms (181 is 17 lines; 193 is 66) | P2 |
| P-4 | **AI_026 ground pets slice C** | remaining style-26 pets | :60254-65024 (slice C range per ledger board D) | ledger-queued; RNG-light flag13 family | P3 |

Sequencing note: W-1…W-9 are **wiring** (faithful code already on main — verify then claim dispatch); B-* are **audits** (distrust-verbatim rule + W66/W81 defect-class checklist: missing gates / f32 reciprocal order / int-vs-float division / strict bounds / RNG draw order / short-circuit operators); P-1 should land FIRST (cheap, removes the dead-arm trap for every wiring slice).

## 6. Verification performed

- `git fetch origin --prune` + `git merge origin/main` → fast-forward `bfe68db8..c7fe175d` (clean, no conflicts).
- Scripted extraction re-run against the synced tree (npc roster/axis/kind-match, fn ranges, C# anchors/method tables, proj arms/data rows); hand-reads of `update_creeper`, `update_brain`, `update_wof`, the sim WoF spawn block, the style axis, the proj dispatch head/tail, and the 53-arm indentation artifact.
- **No compiler invoked** (code-only mode): no cargo/rustc/dotnet runs; all claims are file-read evidence at `c7fe175d`.