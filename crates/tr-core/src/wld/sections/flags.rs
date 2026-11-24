//! Section 2 (pointer 1), flags half: the world header + flags block —
//! `SaveWorldFlags` (`WorldFile.cs:1238-1430`, write) and `LoadWorldFlags`
//! (`WorldFile.cs:1981-2517`, read).
//!
//! The writer runs unconditionally: C# writes its current format (v326), no
//! version gates. The load half keeps the C# gates because the reader meets
//! any version, and it walks the block in the exact C# order — the port-owned
//! fields land in [`World`], everything else is skip-read ("unknowns keep
//! skipping", no state widening). Unowned mandatory fields are written as the
//! C# default; each such block carries its deviation line.

use anyhow::{bail, Result};

use crate::binio::Writer;
use crate::wld::Reader;
use crate::world::World;

/// `BannerSystem.MaxBannerTypes` (`BannerSystem.cs:1453`): both banner arrays
/// are this long, so the section carries two `i16 293` counts.
const MAX_BANNER_TYPES: i16 = 293;
/// `TreeTopsInfo.AreaId.Count` (`TreeTopsInfo.cs:22-45`): the tree-top
/// variations array length.
const TREE_TOP_VARIATIONS: i32 = 13;

pub fn implemented() -> bool {
    true
}

/// Ports `SaveWorldFlags:1238-1430` field-by-field. The snapshot rule
/// (`WorldFile.cs:1040`, `SetTempToOngoing`) needs no separate step here: C#'s
/// `_temp*` writes exist because its loader fills the statics from them, while
/// the port writes the live [`World`] fields directly.
pub fn write(w: &mut Writer, world: &World) {
    write_world_flavour(w, world);
    write_times(w, world);
    // WorldFile.cs:1252-1269: the moon type, tree/cave backgrounds, the
    // ice/jungle/hell back styles. W4-g71 wiring: the tree/cave arrays and the
    // ice/jungle/hell back styles are owned now (the wired tail rolls them).
    // The g70 treeStyle×forest_bg_styles conflation is corrected here per the
    // f45 fix-hunk pair (research/279 §3.3): treeStyle writes the merged
    // `tree_style: [i32; 4]`; `forest_bg_styles` keeps ONLY the setBG seats.
    w.u8(world.moon_type);
    for x in world.tree_x {
        w.i32(x); // treeX (`:1253-1255`)
    }
    for s in world.tree_style {
        w.i32(s); // treeStyle (`:1256-1259`)
    }
    for x in world.cave_back_x[..3] {
        // caveBackX (:1260-1262, W4-g72) — [3] is never persisted
        w.i32(x);
    }
    for s in world.cave_back_style {
        w.i32(s); // caveBackStyle (`:1263-1266`)
    }
    w.i32(world.ice_back_style as i32); // iceBackStyle (`:1267`)
    w.i32(world.jungle_back_style as i32); // jungleBackStyle (`:1268`)
    w.i32(world.hell_back_style as i32); // hellBackStyle (`:1269`)
    // WorldFile.cs:1270-1281: spawn, layer heights, the time snapshot, the
    // moon phase, the blood moon/eclipse, the dungeon anchor. W4-f45 claims
    // the dungeon anchor + the evil flavour (the generator stamps them).
    w.i32(world.spawn_x);
    w.i32(world.spawn_y);
    w.f64(world.world_surface);
    w.f64(world.rock_layer);
    w.f64(world.time);
    w.boolean(world.day_time);
    w.i32(world.moon_phase as i32);
    w.boolean(world.blood_moon);
    w.boolean(world.eclipse);
    w.i32(world.dungeon_x); // dungeonX (W4-f45)
    w.i32(world.dungeon_y); // dungeonY (W4-f45)
    w.boolean(world.crimson); // WorldGen.crimson (W4-f45)
    write_downed_flags(w, world);
    // WorldFile.cs:1299-1305: the orbs, the meteor, the altar count, the
    // hardmode switch and the post-plantera party.
    w.boolean(world.shadow_orb_smashed);
    w.boolean(world.spawn_meteor);
    w.u8(world.shadow_orb_count as u8); // C# truncates to a byte
    w.i32(world.altars_smashed);
    w.boolean(world.hard_mode);
    w.boolean(world.after_party_of_doom); // afterPartyOfDoom (W4-f45, `:1305`)
                      // WorldFile.cs:1306-1312: the invasion block.
    w.i32(world.invasion_delay);
    w.i32(world.invasion_size);
    w.i32(world.invasion_type);
    w.f64(world.invasion_x);
    w.f64(world.slime_rain_time);
    w.u8(world.sundial_cooldown);
    // WorldFile.cs:1312-1314: the rain snapshot. `SetTempToOngoing:1057-1059`
    // copies `Main.raining`/`rainTime`/`maxRaining` into the three temps before
    // the write (the port syncs them from `App.rain`, S1-J-7), and
    // `SetOngoingToTemps:159-162` restores them on load — `Main.cloudAlpha`
    // comes back as `maxRaining`.
    w.boolean(world.raining); // _tempRaining
    w.i32(world.rain_time); // _tempRainTime
    w.f32(world.max_raining); // _tempMaxRain
    // WorldFile.cs:1315-1317: the hardmode ore tiers — `World.ore_tiers`
    // owns them (M5-HARDFORCE-ores / M5-ORES-1); the eight background styles
    // follow. W4-g71 wiring: all eight `setBG` 0..7 styles ride their byte slots
    // (0 treeBG1, 1 corrupt, 2 jungle, 3 snow, 4 hallow, 5 crimson, 6 desert,
    // 7 ocean) — the wired tail rolls them.
    w.i32(world.ore_tiers.cobalt); // Cobalt
    w.i32(world.ore_tiers.mythril); // Mythril
    w.i32(world.ore_tiers.adamantite); // Adamantite
    w.u8(world.forest_bg_styles[0]); // setBG(0) treeBG1
    w.u8(world.corrupt_bg_style); // setBG(1) corruptBG
    w.u8(world.jungle_bg_style); // setBG(2) jungleBG
    w.u8(world.snow_bg_style); // setBG(3) snowBG
    w.u8(world.hallow_bg_style); // setBG(4) hallowBG
    w.u8(world.crimson_bg_style); // setBG(5) crimsonBG
    w.u8(world.desert_bg_style); // setBG(6) desertBG
    w.u8(world.ocean_bg_style); // setBG(7) oceanBG
    // WorldFile.cs:1326-1332: clouds and wind — W4-f45 claims the cloud pair
    // (the g69 CloudField lane's consumers) beside the wind target
    // (`App.wind.speed_target` snapshots into `World.wind_speed_target`).
    w.i32(world.cloud_bg_active); // cloudBGActive (`:1326`)
    w.i16(world.num_clouds as i16); // numClouds (`:1327`, the C# short cast)
    w.f32(world.wind_speed_target); // windSpeedTarget (W4-f45)
    // WorldFile.cs:1329-1338: today's fisherman list — W4-f45 claims the name
    // list (`Main.anglerWhoFinishedToday`); the port's single player collapses
    // its MEANING to the bool `angler_quest_turned_in`, but the file carries
    // the real names. Then the saved-NPC flags and the quest index.
    w.i32(world.angler_who_finished_today.len() as i32); // anglerWhoFinishedToday.Count
    for name in &world.angler_who_finished_today {
        w.cs_string(name);
    }
    w.boolean(world.saved_angler); // savedAngler
    w.i32(world.angler_quest as i32); // anglerQuest
    w.boolean(world.saved_stylist); // savedStylist
    w.boolean(world.saved_tax_collector); // savedTaxCollector
    w.boolean(world.saved_golfer); // savedGolfer
    w.i32(world.invasion_size_start);
    w.i32(world.cultist_delay); // _tempCultistDelay (W4-f45; App.world_events snapshots it)
    // WorldFile.cs:1341: `BannerSystem.Save` — W4-f45 claims the kill counts
    // (`App.banners` densifies into `World.banner_kill_counts` at save). The
    // claimable array stays unowned (the port spawns the banner item at the
    // death spot instead of queueing a claim), so it is written all zero.
    w.i16(MAX_BANNER_TYPES);
    for i in 0..MAX_BANNER_TYPES as usize {
        w.i32(world.banner_kill_counts.get(i).copied().unwrap_or(0)); // killCount
    }
    w.i16(MAX_BANNER_TYPES);
    for _ in 0..MAX_BANNER_TYPES {
        w.u16(0); // claimableBanners — unowned
    }
    // WorldFile.cs:1342-1362: the dial fast-forward, the event bosses and
    // the lunar towers. `downedMartians`, the cultist/Moon Lord kills, the
    // four tower downeds/actives and the apocalypse phase are World-owned;
    // the seasonal bosses stay unowned (no port state). S1-D-3 W-9:
    // `downedFishron` is World-owned now (the case-370 death flag).
    w.boolean(world.fast_forward_to_dawn);
    w.boolean(world.downed_fishron); // downedFishron
    w.boolean(world.downed_martians);
    w.boolean(world.downed_ancient_cultist); // downedAncientCultist
    w.boolean(world.downed_moon_lord); // downedMoonlord
    // W4-f45: the five seasonal bosses — the moon-event-gated kills
    // (`NPC.cs:79210-79239`); the death arms raise them in world_sim/npcs.rs.
    w.boolean(world.downed_halloween_king); // downedHalloweenKing
    w.boolean(world.downed_halloween_tree); // downedHalloweenTree
    w.boolean(world.downed_christmas_ice_queen); // downedChristmasIceQueen
    w.boolean(world.downed_christmas_santank); // downedChristmasSantank
    w.boolean(world.downed_christmas_tree); // downedChristmasTree
    w.boolean(world.downed_tower_solar); // downedTowerSolar
    w.boolean(world.downed_tower_vortex); // downedTowerVortex
    w.boolean(world.downed_tower_nebula); // downedTowerNebula
    w.boolean(world.downed_tower_stardust); // downedTowerStardust
    w.boolean(world.tower_active_solar); // TowerActiveSolar
    w.boolean(world.tower_active_vortex); // TowerActiveVortex
    w.boolean(world.tower_active_nebula); // TowerActiveNebula
    w.boolean(world.tower_active_stardust); // TowerActiveStardust
    w.boolean(world.lunar_apocalypse_is_up); // LunarApocalypseIsUp
    // WorldFile.cs:1363-1368: the party snapshot — W4-f45 claims it: the
    // birthday party state (`App.world_events`) snapshots into the World
    // fields at save (`SetTempToOngoing:1048-1052`). The celebrant list
    // carries the C#'s stable `Main.npc` slots (the port's live list indexes
    // the compacted vec — the recorded `world_events.rs` deviation).
    w.boolean(world.party_manual); // _tempPartyManual
    w.boolean(world.party_genuine); // _tempPartyGenuine
    w.i32(world.party_cooldown); // _tempPartyCooldown
    w.i32(world.party_celebrants.len() as i32); // TempPartyCelebratingNPCs.Count
    for &slot in &world.party_celebrants {
        w.i32(slot);
    }
              // WorldFile.cs:1369-1372: the sandstorm snapshot. `SetTempToOngoing:1053-1056`
              // copies `Sandstorm.Happening`/`TimeLeft`/`Severity`/`IntendedSeverity`
              // in before the write and `SetOngoingToTemps:169-172` restores them.
    w.boolean(world.sandstorm_happening); // _tempSandstormHappening
    w.i32(world.sandstorm_time_left); // _tempSandstormTimeLeft
    w.f32(world.sandstorm_severity); // _tempSandstormSeverity
    w.f32(world.sandstorm_intended_severity); // _tempSandstormIntendedSeverity
    // WorldFile.cs:1373-1375: `NPC.savedBartender` + `DD2Event.Save`'s three
    // tier flags (`DD2Event.cs:151-156`). The tiers are `World::dd2_downed_t*`
    // (S1-E-12c: the victory latches persist); W4-f45 claims `savedBartender`
    // (the rescue path is unported, the file round-trips the latch);
    // `dd2_latches` is the wave-boss death latches (never saved by the C#
    // either), NOT the tiers.
    w.boolean(world.saved_bartender); // savedBartender (W4-f45)
    w.boolean(world.dd2_downed_t1); // DownedInvasionT1
    w.boolean(world.dd2_downed_t2); // DownedInvasionT2
    w.boolean(world.dd2_downed_t3); // DownedInvasionT3
                      // WorldFile.cs:1376-1379: the remaining background styles and the
                      // combat book. W4-g71 wiring: `setBG(8) mushroomBG`, `setBG(9)
                      // underworldBG` and `setBG(10..12) treeBG2..4` are port-owned now.
    w.u8(world.mushroom_bg_style); // mushroomBG
    w.u8(world.underworld_bg_style); // underworldBG
    w.u8(world.forest_bg_styles[1]); // treeBG2
    w.u8(world.forest_bg_styles[2]); // treeBG3
    w.u8(world.forest_bg_styles[3]); // treeBG4
    w.boolean(world.combat_book_was_used); // combatBookWasUsed
    // WorldFile.cs:1380-1384: the lantern night snapshot — W4-f45 claims it
    // (`App.world_events` snapshots into the World fields at save,
    // `SetTempToOngoing:1060-1063`).
    w.i32(world.lantern_night_cooldown); // _tempLanternNightCooldown
    w.boolean(world.lantern_night_genuine); // _tempLanternNightGenuine
    w.boolean(world.lantern_night_manual); // _tempLanternNightManual
    w.boolean(world.lantern_night_next_night_is_genuine); // _tempLanternNightNextNightIsGenuine
                      // WorldFile.cs:1385: `WorldGen.TreeTops.Save` — the tree-top variation
                      // table has no port state; written at its C# length, all zero.
    w.i32(TREE_TOP_VARIATIONS);
    for _ in 0..TREE_TOP_VARIATIONS {
        w.i32(0); // _variations
    }
    // WorldFile.cs:1386-1391: the forced seasonal events (unowned — the f39
    // season lane), the four pre-hardmode ore tiers (`World.ore_tiers`,
    // `:1388-1391`) and the pet licenses (W4-f45, the f43 lane).
    w.boolean(false); // forceHalloweenForToday
    w.boolean(false); // forceXMasForToday
    w.i32(world.ore_tiers.copper); // Copper
    w.i32(world.ore_tiers.iron); // Iron
    w.i32(world.ore_tiers.silver); // Silver
    w.i32(world.ore_tiers.gold); // Gold
    w.boolean(world.bought_cat); // boughtCat (W4-g87 + W4-f45, `:1392`; the f43 license lane)
    w.boolean(world.bought_dog); // boughtDog (W4-g87 + W4-f45, `:1393`)
    w.boolean(world.bought_bunny); // boughtBunny (W4-g87 + W4-f45, `:1394`)
    // WorldFile.cs:1392-1397: the two post-1.4 bosses; `downedQueenSlime` and
    // `downedDeerclops` are owned; W4-f45 claims `downedEmpressOfLight` too
    // (the HallowBoss kind is unported, so the file round-trips the latch).
    w.boolean(world.downed_empress_of_light); // downedEmpressOfLight
    w.boolean(world.downed_queen_slime); // downedQueenSlime
    w.boolean(world.downed_deerclops);
    // WorldFile.cs:1398-1410: the unlocked town-NPC spawns — set on the NPC's
    // arrival (`WorldGen.cs:5577-5604`), read at the spawn gates
    // (`Main.cs:65785-65799` party girl, `:65949-65952` princess) and the
    // `SpawnAllowed_*` overrides (`NPC.cs:7069/7094/7119/7160/7177`). W4-f45
    // claims the six arrival unlocks + the King Slime's slime-blue unlock;
    // the party girl, princess and rainbow/red slimes were already owned.
    w.boolean(world.unlocked_slime_blue_spawn); // unlockedSlimeBlueSpawn (W4-f45)
    w.boolean(world.unlocked_merchant_spawn); // unlockedMerchantSpawn (W4-f45)
    w.boolean(world.unlocked_demolitionist_spawn); // unlockedDemolitionistSpawn (W4-f45)
    w.boolean(world.unlocked_party_girl); // unlockedPartyGirlSpawn
    w.boolean(world.unlocked_dye_trader_spawn); // unlockedDyeTraderSpawn (W4-f45)
    w.boolean(world.unlocked_truffle_spawn); // unlockedTruffleSpawn (W4-f45)
    w.boolean(world.unlocked_arms_dealer_spawn); // unlockedArmsDealerSpawn (W4-f45)
    w.boolean(world.unlocked_nurse_spawn); // unlockedNurseSpawn (W4-f45)
    w.boolean(world.unlocked_princess); // unlockedPrincessSpawn
    w.boolean(world.combat_book_volume_two_was_used); // combatBookVolumeTwoWasUsed (W4-f45, `:1407`; g89b co-claim)
    w.boolean(world.peddlers_satchel_was_used); // peddlersSatchelWasUsed (`:1408`; g89b co-claim)
    w.boolean(world.unlocked_slime_green_spawn); // unlockedSlimeGreenSpawn (W4-f45, `:1409`)
    w.boolean(world.unlocked_slime_old_spawn); // unlockedSlimeOldSpawn (`:1410`)
    w.boolean(world.unlocked_slime_purple_spawn); // unlockedSlimePurpleSpawn (`:1411`)
    w.boolean(world.unlocked_slime_rainbow_spawn); // unlockedSlimeRainbowSpawn
    w.boolean(world.unlocked_slime_red_spawn); // unlockedSlimeRedSpawn
    w.boolean(world.unlocked_slime_yellow_spawn); // unlockedSlimeYellowSpawn (`:1414`)
    w.boolean(world.unlocked_slime_copper_spawn); // unlockedSlimeCopperSpawn (`:1415`)
    // WorldFile.cs:1416-1423: the moondial, the forced seasons and the secret
    // seeds — W4-f45 claims the eight `Main.*` seed bytes: they are the
    // `InitializeSecretSeeds` mirrors of `World.micro_seeds` (`WorldGen.cs:
    // 553-587`), the same bits the `.trwld` v12 tail carries (`save.rs:388-390`).
    // W4-f45 also claims the two starfall counters (`:1422-1423`).
    w.boolean(world.fast_forward_to_dusk);
    w.u8(world.moondial_cooldown);
    w.boolean(world.micro_seeds.contains(crate::seeds::MicroSeedFlags::ENDLESS_HALLOWEEN)); // forceHalloweenForever (`WorldGen.cs:571-574`)
    w.boolean(world.micro_seeds.contains(crate::seeds::MicroSeedFlags::ENDLESS_CHRISTMAS)); // forceXMasForever (`:575-578`)
    w.boolean(world.micro_seeds.contains(crate::seeds::MicroSeedFlags::VAMPIRISM)); // vampireSeed (`:555-558`)
    w.boolean(world.micro_seeds.contains(crate::seeds::MicroSeedFlags::WORLD_IS_INFECTED)); // infectedSeed (`:559-562`)
    w.i32(world.meteor_shower_count); // _tempMeteorShowerCount (W4-f45, `:1422`)
    w.i32(world.coin_rain); // _tempCoinRain (W4-f45, `:1423`)
              // WorldFile.cs:1424-1426: team-based spawns plus the extra spawn point
              // list (`ExtraSpawnPointManager.Write`) — W4-f45 claims the list (the
              // c46 respawn lane); the seed byte is the micro-seed mirror.
    w.boolean(world.micro_seeds.contains(crate::seeds::MicroSeedFlags::TEAM_BASED_SPAWNS)); // teamBasedSpawnsSeed (`:563-566`)
    // `ExtraSpawnPointManager.Write` (`ExtraSpawnPointManager.cs:448-456`): a
    // count byte then that many (i16 X, i16 Y) pairs (W4-f45, the c46 lane).
    w.u8(world.extra_spawn_points.len() as u8); // extraSpawnPoints.Length (`:1425`)
    for &(x, y) in &world.extra_spawn_points {
        w.i16(x);
        w.i16(y);
    }
             // WorldFile.cs:1427-1429: the remaining secret seeds.
    w.boolean(world.micro_seeds.contains(crate::seeds::MicroSeedFlags::DUAL_DUNGEONS)); // dualDungeonsSeed (`:567-570`)
    w.boolean(world.micro_seeds.contains(crate::seeds::MicroSeedFlags::MORE_LIGHTNING)); // moreLightningSeed (`:579-582`)
    w.boolean(world.micro_seeds.contains(crate::seeds::MicroSeedFlags::NO_LIGHTNING)); // noLightningSeed (`:583-586`)
                      // WorldFile.cs:1429: the world-generation manifest. The port has no
                      // JSON manifest, so the empty .NET string is written (deviation: the
                      // generator's own metadata is not persisted).
    w.cs_string("");
}

/// WorldFile.cs:1240-1249: the game mode and the nine world-flavour bools
/// (drunk / get-good / tenth anniversary / don't starve / not the bees / remix /
/// no traps / zenith / skyblock) — `World.seeds` owns them (M5-SEED), each at its
/// introduction version (`SaveWorldFlags:1238-1250` / `LoadWorldFlags:1981-2041`);
/// the pre-267 zenith back-compat inference is the reader's job (`:2020`).
fn write_world_flavour(w: &mut Writer, world: &World) {
    w.i32(world.game_mode); // Main.GameMode (W4-f45; the e25 scaling lane reads it)
    for (min_version, flag) in [
        (222, crate::seeds::SeedFlags::DRUNK),
        (227, crate::seeds::SeedFlags::FOR_THE_WORTHY),
        (238, crate::seeds::SeedFlags::TENTH_ANNIVERSARY),
        (239, crate::seeds::SeedFlags::DONT_STARVE),
        (241, crate::seeds::SeedFlags::NOT_THE_BEES),
        (249, crate::seeds::SeedFlags::REMIX),
        (266, crate::seeds::SeedFlags::NO_TRAPS),
        (267, crate::seeds::SeedFlags::ZENITH),
        (302, crate::seeds::SeedFlags::SKYBLOCK),
    ] {
        // The writer always emits all nine (the modern file is v326); the flag's
        // own version gate is the reader's business.
        let _ = min_version;
        w.boolean(world.seeds.contains(flag));
    }
}

/// `SaveWorldFlags:1250-1251`: the creation time and the last-played time as
/// `DateTime.ToBinary` longs.
fn write_times(w: &mut Writer, world: &World) {
    w.u64(world.creation_time as u64); // CreationTime (`DateTime.ToBinary`)
    w.u64(world.last_played as u64); // LastPlayed (`DateTime.UtcNow.ToBinary`)
}

/// `SaveWorldFlags:1282-1298`: the boss flags. `downedBoss1/2/3` are the Eye,
/// the Eater/Brain and Skeletron Prime's predecessor; `downedMechBossAny` is
/// the derived any-of-three (C# raises it with the individual kills).
fn write_downed_flags(w: &mut Writer, world: &World) {
    w.boolean(world.downed_eoc); // downedBoss1
    w.boolean(world.downed_eow); // downedBoss2
    w.boolean(world.downed_skeletron); // downedBoss3
    w.boolean(world.downed_queen_bee);
    w.boolean(world.downed_mech_boss1);
    w.boolean(world.downed_mech_boss2);
    w.boolean(world.downed_mech_boss3);
    w.boolean(world.downed_mech_boss1 || world.downed_mech_boss2 || world.downed_mech_boss3); // downedMechBossAny — derived
    w.boolean(world.downed_plant_boss);
    w.boolean(world.downed_golem);
    w.boolean(world.downed_slime_king);
    w.boolean(world.saved_goblin);
    w.boolean(world.saved_wizard); // savedWizard
    w.boolean(world.saved_mech); // savedMech
    w.boolean(world.downed_goblins);
    w.boolean(world.downed_clown); // downedClown (W4-f45; the kind is unported, the file round-trips)
    w.boolean(world.downed_frost);
    w.boolean(world.downed_pirates);
}

/// Ports `LoadWorldFlags:1981-2517` as the exact inverse walk: every field in
/// C# order, the C#'s version gates, the port-owned fields read into [`World`]
/// and the rest skip-read. The function returns early at the same points the
/// C# does (its `version < X` guards), so an old-version block ends where C#
/// would end it.
pub(crate) fn read(r: &mut Reader, world: &mut World, version: i32) -> Result<()> {
    // WorldFile.cs:1982-2012 — game mode and the nine flavour bools.
    if version >= 209 {
        world.game_mode = r.i32()?; // GameMode (W4-f45, `:1985`)
        for (min_version, flag) in [
            (222, crate::seeds::SeedFlags::DRUNK),
            (227, crate::seeds::SeedFlags::FOR_THE_WORTHY),
            (238, crate::seeds::SeedFlags::TENTH_ANNIVERSARY),
            (239, crate::seeds::SeedFlags::DONT_STARVE),
            (241, crate::seeds::SeedFlags::NOT_THE_BEES),
            (249, crate::seeds::SeedFlags::REMIX),
            (266, crate::seeds::SeedFlags::NO_TRAPS),
            (267, crate::seeds::SeedFlags::ZENITH),
            (302, crate::seeds::SeedFlags::SKYBLOCK),
        ] {
            // `LoadWorldFlags:1981-2041` reads the nine `Main.*` seed booleans in
            // `SaveWorldFlags` order, each gated on its introduction version — the
            // M5-SEED flags ride here (main's ANALYSIS "Persistence" entry).
            if version >= min_version && r.u8()? != 0 {
                world.seeds.insert(flag);
            }
        }
        // The pre-267 back-compat inference (`WorldFile.cs:2020`): a world that
        // carries remix && drunk but has no zenith byte is a get-fixed-boi world.
        if version < 267
            && world.seeds.contains(crate::seeds::SeedFlags::REMIX)
            && world.seeds.contains(crate::seeds::SeedFlags::DRUNK)
        {
            world.seeds.insert(
                crate::seeds::SeedFlags::ZENITH | crate::seeds::SeedFlags::EVERYTHING_DEPS,
            );
        }
    } else {
        // WorldFile.cs:2029-2040 — the pre-209 game mode: a bool (expert) from
        // 112, else classic; the v208 master byte upgrades it to 2.
        if version >= 112 {
            world.game_mode = if r.u8()? != 0 { 1 } else { 0 };
        } else {
            world.game_mode = 0;
        }
        if version == 208 && r.u8()? != 0 {
            world.game_mode = 2;
        }
    }
    // WorldFile.cs:2042-2049 — the two creation-time longs.
    if version >= 141 {
        world.creation_time = r.u64()? as i64; // CreationTime (`DateTime.FromBinary`)
    }
    if version >= 284 {
        world.last_played = r.u64()? as i64; // LastPlayed (`DateTime.ToBinary`)
    }
    // WorldFile.cs:2021-2068 — moon type, the tree/cave arrays, the back styles.
    // W4-g71 wiring: the moon type, the tree/cave arrays and the ice/jungle/hell
    // back styles are owned now (the wired tail rolls them). The g70
    // treeStyle×forest_bg_styles conflation corrected per the f45 fix-hunk
    // pair (research/279 §3.3): treeStyle reads the merged `tree_style`.
    world.moon_type = r.u8()?.min(8);
    for x in &mut world.tree_x {
        *x = r.i32()?; // treeX (`:2051-2053`)
    }
    for s in &mut world.tree_style {
        *s = r.i32()?; // treeStyle (`:2054-2057`)
    }
    for x in &mut world.cave_back_x[..3] {
        *x = r.i32()?; // caveBackX (`:2058-2060` — [3] keeps its new int[4] zero)
    }
    for s in &mut world.cave_back_style {
        *s = r.i32()?; // caveBackStyle (`:2061-2064`)
    }
    world.ice_back_style = r.i32()? as u8; // iceBackStyle (`:2065`)
    world.jungle_back_style = r.i32()? as u8; // jungleBackStyle (`:2066`)
    world.hell_back_style = r.i32()? as u8; // hellBackStyle (`:2067`)
    // WorldFile.cs:2069-2083 — spawn, layers, time snapshot, moon phase, blood moon,
    // eclipse, dungeon.
    world.spawn_x = r.i32()?;
    world.spawn_y = r.i32()?;
    world.world_surface = r.f64()?;
    world.rock_layer = r.f64()?;
    world.time = r.f64()?;
    world.day_time = r.u8()? != 0;
    // `_tempMoonPhase` follows `_tempDayTime` (`WorldFile.cs:2074`).
    world.moon_phase = (r.i32()? as u8) & 7;
    world.blood_moon = r.u8()? != 0;
    world.eclipse = r.u8()? != 0;
    world.dungeon_x = r.i32()?; // dungeonX (W4-f45, `:2078`)
    world.dungeon_y = r.i32()?; // dungeonY (W4-f45, `:2079`)
    world.crimson = r.bool()?; // WorldGen.crimson (W4-f45, `:2080`)
                // WorldFile.cs:2084-2102 — the boss flags (with `downedSlimeKing` gated at 118).
    world.downed_eoc = r.bool()?; // downedBoss1
    world.downed_eow = r.bool()?; // downedBoss2
    world.downed_skeletron = r.bool()?; // downedBoss3
    world.downed_queen_bee = r.bool()?;
    world.downed_mech_boss1 = r.bool()?;
    world.downed_mech_boss2 = r.bool()?;
    world.downed_mech_boss3 = r.bool()?;
    r.skip(1)?; // downedMechBossAny — derived, not stored
    world.downed_plant_boss = r.bool()?;
    world.downed_golem = r.bool()?;
    if version >= 118 {
        world.downed_slime_king = r.bool()?;
    }
    world.saved_goblin = r.bool()?;
    world.saved_wizard = r.bool()?; // savedWizard
    world.saved_mech = r.bool()?; // savedMech
    world.downed_goblins = r.bool()?;
    world.downed_clown = r.bool()?; // downedClown (W4-f45)
    world.downed_frost = r.bool()?;
    world.downed_pirates = r.bool()?;
    // WorldFile.cs:2103-2117 — orbs, meteor, altars, hardmode, invasion.
    world.shadow_orb_smashed = r.bool()?;
    world.spawn_meteor = r.bool()?;
    world.shadow_orb_count = r.u8()? as i32;
    world.altars_smashed = r.i32()?;
    world.hard_mode = r.bool()?;
    if version >= 257 {
        world.after_party_of_doom = r.bool()?; // afterPartyOfDoom (W4-f45, `:2109`)
    }
    world.invasion_delay = r.i32()?;
    world.invasion_size = r.i32()?;
    world.invasion_type = r.i32()?;
    world.invasion_x = r.f64()?;
    if version >= 118 {
        world.slime_rain_time = r.f64()?;
    }
    if version >= 113 {
        world.sundial_cooldown = r.u8()?;
    }
    // WorldFile.cs:2127-2129: the rain snapshot, into `Main.raining` /
    // `rainTime` / `maxRaining` through `_temp*` (restored by
    // `SetOngoingToTemps:159-162`).
    world.raining = r.bool()?; // _tempRaining
    world.rain_time = r.i32()?; // _tempRainTime
    world.max_raining = r.f32()?; // _tempMaxRain
    // WorldFile.cs:2131-2133 — the hardmode ore tiers into `World.ore_tiers`.
    world.ore_tiers.cobalt = r.i32()?;
    world.ore_tiers.mythril = r.i32()?;
    world.ore_tiers.adamantite = r.i32()?;
    // `setBG` slots 0..7 (`WorldFile.cs:2211-2218`): W4-g71 wiring — all eight
    // styles are port-owned now (slot 0 treeBG1 included).
    world.forest_bg_styles[0] = r.u8()?; // setBG(0) treeBG1
    world.corrupt_bg_style = r.u8()?; // setBG(1) corruptBG
    world.jungle_bg_style = r.u8()?; // setBG(2) jungleBG
    world.snow_bg_style = r.u8()?; // setBG(3) snowBG
    world.hallow_bg_style = r.u8()?; // setBG(4) hallowBG
    world.crimson_bg_style = r.u8()?; // setBG(5) crimsonBG
    world.desert_bg_style = r.u8()?; // setBG(6) desertBG
    world.ocean_bg_style = r.u8()?; // setBG(7) oceanBG
    // stored value, then `:2144` re-rolls cloudBGActive (`-genRand.Next(8640,
    // 86400)`) — the load does NOT restore the stored timer (the g69 lane's
    // live machine re-rolls it), so the port reads it into the World mirror
    // for the round-trip but the App keeps its own re-roll. numClouds (`:2145`)
    // restores as-is.
    world.cloud_bg_active = r.i32()?; // cloudBGActive (`:2142`)
    world.num_clouds = r.i16()? as i32; // numClouds (`:2145`)
    // W4-f45: the wind target lands; `:2147` sets `windSpeedCurrent` to it
    // (the App restores both from this one field, `app/mod.rs`).
    world.wind_speed_target = r.f32()?; // windSpeedTarget (`:2146`)
    if version < 95 {
        return Ok(());
    }
    // WorldFile.cs:2152-2156 — today's angler list: a count then that many
    // names (W4-f45 claims the list; the C# clears it first).
    let anglers = r.i32()?;
    world.angler_who_finished_today.clear();
    for _ in 0..anglers.max(0) {
        world.angler_who_finished_today.push(r.string()?);
    }
    if version < 99 {
        return Ok(());
    }
    world.saved_angler = r.bool()?; // savedAngler
    if version < 101 {
        return Ok(());
    }
    world.angler_quest = r.i32()? as u8; // anglerQuest (the 0..40 pool index)
    if version < 104 {
        return Ok(());
    }
    world.saved_stylist = r.bool()?; // savedStylist
    if version >= 129 {
        world.saved_tax_collector = r.bool()?; // savedTaxCollector
    }
    if version >= 201 {
        world.saved_golfer = r.bool()?; // savedGolfer
    }
    // WorldFile.cs:2180-2198 — the invasion-start + cultist-delay gates the
    // S1-J-3b walk flattened: below 107 `invasionSizeStart` is not read at
    // all (`FakeLoadInvasionStart` re-derives it from the invasion pair,
    // `Main.cs:64488-64514`); below 108 the cultist delay defaults to 86400
    // (`CultistRitual.delayStart`). W4-f45 restores both gates.
    if version < 107 {
        if world.invasion_type > 0 && world.invasion_size > 0 {
            fake_load_invasion_start(world);
        }
    } else {
        world.invasion_size_start = r.i32()?;
    }
    if version < 108 {
        world.cultist_delay = 86400; // `CultistRitual.delayStart` (`:2193`)
    } else {
        world.cultist_delay = r.i32()?; // _tempCultistDelay (`:2197`, W4-f45)
    }
    if version < 109 {
        return Ok(());
    }
    // WorldFile.cs:2203 — `BannerSystem.Load` (`BannerSystem.cs:188-212`):
    // the kill counts land in the dense array (only the entries the array
    // carries, `i < killCount.Length`); the claimable array (289+) stays
    // unowned, skip-read. W4-f45 claims the kill counts.
    let kill_counts = r.i16()? as i32;
    for i in 0..kill_counts.max(0) as usize {
        let count = r.i32()?;
        if i < world.banner_kill_counts.len() {
            world.banner_kill_counts[i] = count;
        }
    }
    if version >= 289 {
        let claimable = r.i16()? as i32;
        r.skip((claimable.max(0) as usize) * 2)?; // claimableBanners — unowned
    }
    if version < 128 {
        return Ok(());
    }
    world.fast_forward_to_dawn = r.bool()?;
    if version < 131 {
        return Ok(());
    }
    world.downed_fishron = r.bool()?; // downedFishron (S1-D-3 W-9)
    world.downed_martians = r.bool()?;
    world.downed_ancient_cultist = r.bool()?; // downedAncientCultist
    world.downed_moon_lord = r.bool()?; // downedMoonlord
    // W4-f45: the five seasonal bosses (`LoadWorldFlags:2217-2221`, v131+).
    world.downed_halloween_king = r.bool()?; // downedHalloweenKing
    world.downed_halloween_tree = r.bool()?; // downedHalloweenTree
    world.downed_christmas_ice_queen = r.bool()?; // downedChristmasIceQueen
    world.downed_christmas_santank = r.bool()?; // downedChristmasSantank
    world.downed_christmas_tree = r.bool()?; // downedChristmasTree
    if version < 140 {
        return Ok(());
    }
    world.downed_tower_solar = r.bool()?;
    world.downed_tower_vortex = r.bool()?;
    world.downed_tower_nebula = r.bool()?;
    world.downed_tower_stardust = r.bool()?;
    world.tower_active_solar = r.bool()?;
    world.tower_active_vortex = r.bool()?;
    world.tower_active_nebula = r.bool()?;
    world.tower_active_stardust = r.bool()?;
    world.lunar_apocalypse_is_up = r.bool()?;
    // WorldFile.cs:2235-2250: an active tower comes back at full shield —
    // `ShieldStrengthTowerMax` (`NPC.cs:6795-6806`: the normal power, halved
    // once the Moon Lord is downed), the tr-app `tower_shield_max` twin.
    let shield_max = if world.downed_moon_lord {
        world.lunar_shield_power_normal / 2
    } else {
        world.lunar_shield_power_normal
    };
    if world.tower_active_solar {
        world.tower_shield_solar = shield_max;
    }
    if world.tower_active_vortex {
        world.tower_shield_vortex = shield_max;
    }
    if world.tower_active_nebula {
        world.tower_shield_nebula = shield_max;
    }
    if world.tower_active_stardust {
        world.tower_shield_stardust = shield_max;
    }
    // WorldFile.cs:2251-2269 — the party snapshot (W4-f45): below 170 the
    // four temps reset (`:2253-2256` — the World defaults already match),
    // else manual/genuine/cooldown + a counted i32 slot list.
    if version >= 170 {
        world.party_manual = r.bool()?; // _tempPartyManual
        world.party_genuine = r.bool()?; // _tempPartyGenuine
        world.party_cooldown = r.i32()?; // _tempPartyCooldown
        let celebrants = r.i32()?;
        world.party_celebrants.clear();
        for _ in 0..celebrants.max(0) {
            world.party_celebrants.push(r.i32()?);
        }
    }
    // WorldFile.cs:2277-2283 — the sandstorm snapshot: below 174 the four
    // temps are zeroed (`:2272-2275`), else read back.
    if version >= 174 {
        world.sandstorm_happening = r.bool()?; // happening
        world.sandstorm_time_left = r.i32()?; // timeLeft
        world.sandstorm_severity = r.f32()?; // severity
        world.sandstorm_intended_severity = r.f32()?; // intendedSeverity
    }
    // WorldFile.cs:2214-2218 — `DD2Event.Load`: below 178 nothing is read at all
    // (the C# clears the tiers below the gate, `DD2Event.cs:174` — the World
    // defaults already false).
    if version >= 178 {
        world.saved_bartender = r.bool()?; // savedBartender (W4-f45)
        world.dd2_downed_t1 = r.bool()?; // DownedInvasionT1
        world.dd2_downed_t2 = r.bool()?; // DownedInvasionT2
        world.dd2_downed_t3 = r.bool()?; // DownedInvasionT3
    }
    // WorldFile.cs:2219-2233 — the remaining background styles. W4-g71 wiring:
    // the underworld + treeBG2..4 styles are port-owned now.
    if version > 194 {
        world.mushroom_bg_style = r.u8()?; // setBG(8) mushroomBG
    }
    if version >= 215 {
        world.underworld_bg_style = r.u8()?; // setBG(9) underworldBG
    }
    if version > 195 {
        for s in &mut world.forest_bg_styles[1..] {
            *s = r.u8()?; // setBG(10..12) treeBG2..4
        }
    }
    if version >= 204 {
        world.combat_book_was_used = r.bool()?; // combatBookWasUsed
    }
    // WorldFile.cs:2317-2330 — the lantern night snapshot (W4-f45): below
    // 207 the four temps reset (`:2319-2322` — the World defaults match).
    if version >= 207 {
        world.lantern_night_cooldown = r.i32()?; // _tempLanternNightCooldown
        world.lantern_night_genuine = r.bool()?; // _tempLanternNightGenuine
        world.lantern_night_manual = r.bool()?; // _tempLanternNightManual
        world.lantern_night_next_night_is_genuine = r.bool()?; // _tempLanternNightNextNightIsGenuine
    }
    // WorldFile.cs:2240-2243 — `TreeTops.Load`: below 211 nothing is read. The C# reads
    // `min(count, 13)` entries; skipping the full count keeps the byte cursor
    // where a C#-written file (count == 13) ends it.
    if version >= 211 {
        let tops = r.i32()?;
        r.skip((tops.max(0) as usize) * 4)?;
    }
    // WorldFile.cs:2244-2278 — forced seasons, ore tiers, pets, post-1.4 bosses, the
    // unlocked spawns and the combat books.
    if version >= 212 {
        r.skip(2)?; // forceHalloweenForToday, forceXMasForToday
    }
    if version >= 216 {
        // WorldFile.cs:2344-2347 — the pre-hardmode quartet.
        world.ore_tiers.copper = r.i32()?;
        world.ore_tiers.iron = r.i32()?;
        world.ore_tiers.silver = r.i32()?;
        world.ore_tiers.gold = r.i32()?;
    } else {
        // WorldFile.cs:2351-2354 — the pre-216 reset arm; the post-load
        // `CheckSavedOreTiers` (`:730`) then re-derives them from the tiles.
        world.ore_tiers.copper = -1;
        world.ore_tiers.iron = -1;
        world.ore_tiers.silver = -1;
        world.ore_tiers.gold = -1;
    }
    if version >= 217 {
        // WorldFile.cs:2358-2360 — the three town-pet licenses (W4-g87).
        // W4-f45: the pet-license trio lands (`:2358-2360`); the f43 lane's
        // `TailFlags.bought_*` consumers read them.
        world.bought_cat = r.bool()?; // boughtCat
        world.bought_dog = r.bool()?; // boughtDog
        world.bought_bunny = r.bool()?; // boughtBunny
    }
    if version >= 223 {
        world.downed_empress_of_light = r.bool()?; // downedEmpressOfLight (W4-f45)
        world.downed_queen_slime = r.bool()?; // downedQueenSlime
    }
    if version >= 240 {
        world.downed_deerclops = r.bool()?;
    }
    if version >= 250 {
        world.unlocked_slime_blue_spawn = r.bool()?; // unlockedSlimeBlueSpawn (W4-f45)
    }
    if version >= 251 {
        // WorldFile.cs:2396-2403 — the eight unlocks in write order; W4-f45
        // claims the six arrival unlocks beside the party girl/princess.
        world.unlocked_merchant_spawn = r.bool()?; // unlockedMerchantSpawn
        world.unlocked_demolitionist_spawn = r.bool()?; // unlockedDemolitionistSpawn
        world.unlocked_party_girl = r.bool()?; // unlockedPartyGirlSpawn
        world.unlocked_dye_trader_spawn = r.bool()?; // unlockedDyeTraderSpawn
        world.unlocked_truffle_spawn = r.bool()?; // unlockedTruffleSpawn
        world.unlocked_arms_dealer_spawn = r.bool()?; // unlockedArmsDealerSpawn
        world.unlocked_nurse_spawn = r.bool()?; // unlockedNurseSpawn
        world.unlocked_princess = r.bool()?; // unlockedPrincessSpawn
    }
    if version >= 259 {
        world.combat_book_volume_two_was_used = r.bool()?; // combatBookVolumeTwoWasUsed (W4-f45, `:2418`)
    }
    if version >= 260 {
        world.peddlers_satchel_was_used = r.bool()?; // peddlersSatchelWasUsed (`:2426`)
    }
    if version >= 261 {
        // W4-f45: the five remaining slime-colour unlocks land beside the
        // rainbow/red pair (`:2434-2440`). The town-slime kinds are unported,
        // so the latches round-trip (the green arrival setter rides the f43
        // town-slime lane; the doom party clears green, `Main.cs:17940`).
        world.unlocked_slime_green_spawn = r.bool()?; // unlockedSlimeGreenSpawn
        world.unlocked_slime_old_spawn = r.bool()?; // unlockedSlimeOldSpawn
        world.unlocked_slime_purple_spawn = r.bool()?; // unlockedSlimePurpleSpawn
        world.unlocked_slime_rainbow_spawn = r.bool()?; // unlockedSlimeRainbowSpawn
        world.unlocked_slime_red_spawn = r.bool()?; // unlockedSlimeRedSpawn
        world.unlocked_slime_yellow_spawn = r.bool()?; // unlockedSlimeYellowSpawn
        world.unlocked_slime_copper_spawn = r.bool()?; // unlockedSlimeCopperSpawn
    }
    // WorldFile.cs:2306-2319 — the moondial, the forced seasons, the secret seeds, the
    // meteor shower, the team spawns and the extra spawn points.
    if version >= 264 {
        world.fast_forward_to_dusk = r.bool()?;
        world.moondial_cooldown = r.u8()?;
    }
    if version >= 287 {
        // W4-f45: the two forced-season bytes are the endlessHalloween/
        // endlessChristmas micro-seed mirrors (`WorldGen.cs:571-578`).
        if r.bool()? {
            world.micro_seeds.insert(crate::seeds::MicroSeedFlags::ENDLESS_HALLOWEEN);
        }
        if r.bool()? {
            world.micro_seeds.insert(crate::seeds::MicroSeedFlags::ENDLESS_CHRISTMAS);
        }
    }
    if version >= 288 {
        if r.bool()? {
            world.micro_seeds.insert(crate::seeds::MicroSeedFlags::VAMPIRISM); // vampireSeed
        }
    }
    if version >= 296 {
        if r.bool()? {
            world.micro_seeds.insert(crate::seeds::MicroSeedFlags::WORLD_IS_INFECTED); // infectedSeed
        }
    }
    if version >= 291 {
        // W4-f45: the starfall pair lands (`SetOngoingToTemps:177-178` restores
        // them to `WorldGen.meteorShowerCount` / `Main.coinRain`).
        world.meteor_shower_count = r.i32()?; // _tempMeteorShowerCount (`:2490`)
        world.coin_rain = r.i32()?; // _tempCoinRain (`:2491`)
    }
    if version >= 297 {
        if r.bool()? {
            world.micro_seeds.insert(crate::seeds::MicroSeedFlags::TEAM_BASED_SPAWNS); // teamBasedSpawnsSeed
        }
        // `ExtraSpawnPointManager.Read` (`ExtraSpawnPointManager.cs:436-446`):
        // a count byte then that many (i16 X, i16 Y) pairs (W4-f45).
        let extra = r.u8()? as i32;
        world.extra_spawn_points.clear();
        for _ in 0..extra.max(0) {
            world.extra_spawn_points.push((r.i16()?, r.i16()?));
        }
    }
    if version >= 304 {
        if r.bool()? {
            world.micro_seeds.insert(crate::seeds::MicroSeedFlags::DUAL_DUNGEONS); // dualDungeonsSeed
        }
    }
    if version >= 323 {
        if r.bool()? {
            world.micro_seeds.insert(crate::seeds::MicroSeedFlags::MORE_LIGHTNING); // moreLightningSeed
        }
        if r.bool()? {
            world.micro_seeds.insert(crate::seeds::MicroSeedFlags::NO_LIGHTNING); // noLightningSeed
        }
    }
    // WorldFile.cs:2499-2501 — the legacy u32 only 299..312 read.
    if (299..313).contains(&version) {
        r.skip(4)?;
    }
    // WorldFile.cs:2503 — `WorldGen.Manifest` as a .NET string.
    if version >= 299 {
        skip_one_string(r)?;
    }
    Ok(())
}

/// Skips one 7-bit-prefixed string without decoding it.
fn skip_one_string(r: &mut Reader) -> Result<()> {
    let mut len = 0usize;
    let mut shift = 0u32;
    loop {
        let b = r.u8()?;
        len |= ((b & 0x7f) as usize) << shift;
        if b & 0x80 == 0 {
            break;
        }
        shift += 7;
        if shift > 28 {
            bail!("bad string length prefix in the flags block");
        }
    }
    r.skip(len)
}

/// `Main.FakeLoadInvasionStart` (`Main.cs:64488-64514`): the pre-107 load's
/// `invasionSizeStart` re-derivation from the invasion pair — the per-type
/// (base, step) ladder, then `base + ceil((size - base) / step) * step` when
/// the ceiling is positive. W4-f45 (the `LoadWorldFlags:2180-2186` gate).
fn fake_load_invasion_start(world: &mut World) {
    let (base, step) = match world.invasion_type {
        1 | 2 => (80, 40),
        3 => (120, 60),
        4 => (160, 40),
        _ => (0, 0),
    };
    // The C# divides by `num2` unconditionally; the port guards the unknown
    // type's zero step (the C# would throw there — an unreachable file shape).
    let waves = if step > 0 {
        ((world.invasion_size - base) as f32 / step as f32).ceil() as i32
    } else {
        0
    };
    world.invasion_size_start = base;
    if waves > 0 {
        world.invasion_size_start += waves * step;
    }
}
