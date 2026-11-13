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
    // ice/jungle/hell back styles. Only `hellBackStyle` is owned; the tree and
    // cave arrays are unowned (no port state — `net.rs`'s `WorldInfo` carries
    // the same fields as zeros).
    w.u8(world.moon_type);
    for _ in 0..3 {
        w.i32(0); // treeX
    }
    for _ in 0..4 {
        w.i32(0); // treeStyle
    }
    for _ in 0..3 {
        w.i32(0); // caveBackX
    }
    for _ in 0..4 {
        w.i32(0); // caveBackStyle
    }
    w.i32(0); // iceBackStyle — unowned
    w.i32(0); // jungleBackStyle — unowned
    w.i32(world.hell_back_style as i32);
    // WorldFile.cs:1270-1281: spawn, layer heights, the time snapshot, the
    // moon phase, the blood moon/eclipse, the dungeon anchor.
    w.i32(world.spawn_x);
    w.i32(world.spawn_y);
    w.f64(world.world_surface);
    w.f64(world.rock_layer);
    w.f64(world.time);
    w.boolean(world.day_time);
    w.i32(world.moon_phase as i32);
    w.boolean(world.blood_moon);
    w.boolean(world.eclipse);
    w.i32(0); // dungeonX — unowned
    w.i32(0); // dungeonY — unowned
    w.boolean(false); // WorldGen.crimson — the evil flavour is unowned state
    write_downed_flags(w, world);
    // WorldFile.cs:1299-1305: the orbs, the meteor, the altar count, the
    // hardmode switch and the post-plantera party.
    w.boolean(world.shadow_orb_smashed);
    w.boolean(world.spawn_meteor);
    w.u8(world.shadow_orb_count as u8); // C# truncates to a byte
    w.i32(world.altars_smashed);
    w.boolean(world.hard_mode);
    w.boolean(false); // afterPartyOfDoom — unowned
                      // WorldFile.cs:1306-1312: the invasion block.
    w.i32(world.invasion_delay);
    w.i32(world.invasion_size);
    w.i32(world.invasion_type);
    w.f64(world.invasion_x);
    w.f64(world.slime_rain_time);
    w.u8(world.sundial_cooldown);
    // WorldFile.cs:1313-1315: the rain snapshot — the port's rain/weather
    // state lives in tr-app, so the written values are the C# defaults.
    w.boolean(false); // _tempRaining
    w.i32(0); // _tempRainTime
    w.f32(0.0); // _tempMaxRain
                // WorldFile.cs:1316-1325: the hardmode ore tiers (unowned — no port
                // state) and the eight background styles (owned only for the renderer's
                // hell sheet, which rides `hellBackStyle`).
    w.i32(0); // Cobalt
    w.i32(0); // Mythril
    w.i32(0); // Adamantite
    for _ in 0..8 {
        w.u8(0); // treeBG1..oceanBG
    }
    // WorldFile.cs:1326-1332: clouds and wind — the cloud state is unowned
    // and the wind engine lives in tr-app.
    w.i32(0); // cloudBGActive
    w.i16(0); // numClouds
    w.f32(0.0); // windSpeedTarget
                // WorldFile.cs:1333-1338: today's fisherman list (empty — the angler
                // quest is unported), then the saved-NPC flags.
    w.i32(0); // anglerWhoFinishedToday.Count
    w.boolean(false); // savedAngler
    w.i32(0); // anglerQuest
    w.boolean(false); // savedStylist
    w.boolean(false); // savedTaxCollector
    w.boolean(false); // savedGolfer
    w.i32(world.invasion_size_start);
    w.i32(0); // _tempCultistDelay — the ritual is unported
              // WorldFile.cs:1341: `BannerSystem.Save` — the banner kill counts have
              // no port store, so both arrays are written at their C# length, all zero.
    w.i16(MAX_BANNER_TYPES);
    for _ in 0..MAX_BANNER_TYPES {
        w.i32(0); // killCount
    }
    w.i16(MAX_BANNER_TYPES);
    for _ in 0..MAX_BANNER_TYPES {
        w.u16(0); // claimableBanners
    }
    // WorldFile.cs:1342-1362: the dial fast-forward, the event bosses and
    // the lunar towers. `downedMartians` is owned; the rest are unported event
    // flags with no port state.
    w.boolean(world.fast_forward_to_dawn);
    w.boolean(false); // downedFishron
    w.boolean(world.downed_martians);
    w.boolean(false); // downedAncientCultist
    w.boolean(false); // downedMoonlord
    w.boolean(false); // downedHalloweenKing
    w.boolean(false); // downedHalloweenTree
    w.boolean(false); // downedChristmasIceQueen
    w.boolean(false); // downedChristmasSantank
    w.boolean(false); // downedChristmasTree
    for _ in 0..4 {
        w.boolean(false); // downedTower{Solar,Vortex,Nebula,Stardust}
    }
    for _ in 0..4 {
        w.boolean(false); // TowerActive{Solar,Vortex,Nebula,Stardust}
    }
    w.boolean(false); // LunarApocalypseIsUp
                      // WorldFile.cs:1363-1368: the party snapshot — the birthday party is
                      // unowned, so manual/genuine are false, the cooldown 0 and the celebrant
                      // list empty.
    w.boolean(false); // _tempPartyManual
    w.boolean(false); // _tempPartyGenuine
    w.i32(0); // _tempPartyCooldown
    w.i32(0); // TempPartyCelebratingNPCs.Count
              // WorldFile.cs:1369-1372: the sandstorm snapshot — the sandstorm lives
              // in tr-app (`scene::Sandstorm`), so defaults are written.
    w.boolean(false); // _tempSandstormHappening
    w.i32(0); // _tempSandstormTimeLeft
    w.f32(0.0); // _tempSandstormSeverity
    w.f32(0.0); // _tempSandstormIntendedSeverity
                // WorldFile.cs:1373-1375: `DD2Event.Save` — the tavern/old-one's army
                // progress is unported.
    w.boolean(false); // savedBartender
    w.boolean(false); // DownedInvasionT1
    w.boolean(false); // DownedInvasionT2
    w.boolean(false); // DownedInvasionT3
                      // WorldFile.cs:1376-1379: the remaining background styles and the
                      // combat book — unowned.
    w.u8(0); // mushroomBG
    w.u8(0); // underworldBG
    w.u8(0); // treeBG2
    w.u8(0); // treeBG3
    w.u8(0); // treeBG4
    w.boolean(false); // combatBookWasUsed
                      // WorldFile.cs:1380-1384: the lantern night snapshot — unowned.
    w.i32(0); // _tempLanternNightCooldown
    w.boolean(false); // _tempLanternNightGenuine
    w.boolean(false); // _tempLanternNightManual
    w.boolean(false); // _tempLanternNightNextNightIsGenuine
                      // WorldFile.cs:1385: `WorldGen.TreeTops.Save` — the tree-top variation
                      // table has no port state; written at its C# length, all zero.
    w.i32(TREE_TOP_VARIATIONS);
    for _ in 0..TREE_TOP_VARIATIONS {
        w.i32(0); // _variations
    }
    // WorldFile.cs:1386-1391: the forced seasonal events, the four ore
    // tiers and the pets — all unowned.
    w.boolean(false); // forceHalloweenForToday
    w.boolean(false); // forceXMasForToday
    w.i32(0); // Copper
    w.i32(0); // Iron
    w.i32(0); // Silver
    w.i32(0); // Gold
    w.boolean(false); // boughtCat
    w.boolean(false); // boughtDog
    w.boolean(false); // boughtBunny
                      // WorldFile.cs:1392-1397: the two post-1.4 bosses; `downedDeerclops`
                      // is owned.
    w.boolean(false); // downedEmpressOfLight
    w.boolean(false); // downedQueenSlime
    w.boolean(world.downed_deerclops);
    // WorldFile.cs:1398-1410: the unlocked town-NPC spawns. Only the
    // rainbow slime spawn has port state; the other unlocks are unowned.
    w.boolean(false); // unlockedSlimeBlueSpawn
    w.boolean(false); // unlockedMerchantSpawn
    w.boolean(false); // unlockedDemolitionistSpawn
    w.boolean(false); // unlockedPartyGirlSpawn
    w.boolean(false); // unlockedDyeTraderSpawn
    w.boolean(false); // unlockedTruffleSpawn
    w.boolean(false); // unlockedArmsDealerSpawn
    w.boolean(false); // unlockedNurseSpawn
    w.boolean(false); // unlockedPrincessSpawn
    w.boolean(false); // combatBookVolumeTwoWasUsed
    w.boolean(false); // peddlersSatchelWasUsed
    w.boolean(false); // unlockedSlimeGreenSpawn
    w.boolean(false); // unlockedSlimeOldSpawn
    w.boolean(false); // unlockedSlimePurpleSpawn
    w.boolean(world.unlocked_slime_rainbow_spawn); // unlockedSlimeRainbowSpawn
    w.boolean(false); // unlockedSlimeRedSpawn
    w.boolean(false); // unlockedSlimeYellowSpawn
    w.boolean(false); // unlockedSlimeCopperSpawn
                      // WorldFile.cs:1416-1423: the moondial, the forced seasons and the
                      // secret seeds — unowned; the two rain counters likewise.
    w.boolean(world.fast_forward_to_dusk);
    w.u8(world.moondial_cooldown);
    w.boolean(false); // forceHalloweenForever
    w.boolean(false); // forceXMasForever
    w.boolean(false); // vampireSeed
    w.boolean(false); // infectedSeed
    w.i32(0); // _tempMeteorShowerCount
    w.i32(0); // _tempCoinRain
              // WorldFile.cs:1424-1426: team-based spawns plus the extra spawn point
              // list (`ExtraSpawnPointManager.Write`) — unowned, so a zero count.
    w.boolean(false); // teamBasedSpawnsSeed
    w.u8(0); // extraSpawnPoints.Length
             // WorldFile.cs:1427-1429: the remaining secret seeds.
    w.boolean(false); // dualDungeonsSeed
    w.boolean(false); // moreLightningSeed
    w.boolean(false); // noLightningSeed
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
    w.i32(0); // Main.GameMode — no port concept
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
    w.u64(0); // CreationTime — unowned
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
    w.boolean(false); // savedWizard — unowned
    w.boolean(false); // savedMech — unowned
    w.boolean(world.downed_goblins);
    w.boolean(false); // downedClown — unowned
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
        r.skip(4)?; // GameMode
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
        if version >= 112 {
            r.skip(1)?; // GameMode as bool
        }
        if version == 208 {
            r.skip(1)?;
        }
    }
    // WorldFile.cs:2014-2020 — the two creation-time longs.
    if version >= 141 {
        r.skip(8)?; // CreationTime
    }
    if version >= 284 {
        world.last_played = r.u64()? as i64; // LastPlayed (`DateTime.ToBinary`)
    }
    // WorldFile.cs:2021-2068 — moon type, the tree/cave arrays, the back styles. Only the
    // moon type and `hellBackStyle` are owned.
    world.moon_type = r.u8()?.min(8);
    r.skip(3 * 4)?; // treeX
    r.skip(4 * 4)?; // treeStyle
    r.skip(3 * 4)?; // caveBackX
    r.skip(4 * 4)?; // caveBackStyle
    r.skip(2 * 4)?; // iceBackStyle, jungleBackStyle
    world.hell_back_style = r.i32()? as u8;
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
    r.skip(2 * 4)?; // dungeonX/Y
    r.skip(1)?; // WorldGen.crimson
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
    r.skip(2)?; // savedWizard, savedMech
    world.downed_goblins = r.bool()?;
    r.skip(1)?; // downedClown
    world.downed_frost = r.bool()?;
    world.downed_pirates = r.bool()?;
    // WorldFile.cs:2103-2117 — orbs, meteor, altars, hardmode, invasion.
    world.shadow_orb_smashed = r.bool()?;
    world.spawn_meteor = r.bool()?;
    world.shadow_orb_count = r.u8()? as i32;
    world.altars_smashed = r.i32()?;
    world.hard_mode = r.bool()?;
    if version >= 257 {
        r.skip(1)?; // afterPartyOfDoom
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
    r.skip(1)?; // _tempRaining
    r.skip(4)?; // _tempRainTime
    r.skip(4)?; // _tempMaxRain
    r.skip(3 * 4)?; // SavedOreTiers Cobalt/Mythril/Adamantite
    r.skip(8)?; // setBG 0..7
    r.skip(4)?; // cloudBGActive
    r.skip(2)?; // numClouds
    r.skip(4)?; // windSpeedTarget
    if version < 95 {
        return Ok(());
    }
    // WorldFile.cs:2131-2135 — today's angler list: a count then that many names.
    let anglers = r.i32()?;
    skip_strings(r, anglers)?;
    if version < 99 {
        return Ok(());
    }
    r.skip(1)?; // savedAngler
    if version < 101 {
        return Ok(());
    }
    r.skip(4)?; // anglerQuest
    if version < 104 {
        return Ok(());
    }
    r.skip(1)?; // savedStylist
    if version >= 129 {
        r.skip(1)?; // savedTaxCollector
    }
    if version >= 201 {
        r.skip(1)?; // savedGolfer
    }
    world.invasion_size_start = r.i32()?;
    r.skip(4)?; // _tempCultistDelay
                // WorldFile.cs:2157-2166 — `BannerSystem.Load` (the second array only from 289).
    let kill_counts = r.i16()? as i32;
    r.skip((kill_counts.max(0) as usize) * 4)?;
    if version >= 289 {
        let claimable = r.i16()? as i32;
        r.skip((claimable.max(0) as usize) * 2)?;
    }
    world.fast_forward_to_dawn = r.bool()?;
    if version < 131 {
        return Ok(());
    }
    r.skip(1)?; // downedFishron
    world.downed_martians = r.bool()?;
    r.skip(7)?; // downedAncientCultist .. downedChristmasTree
    if version < 140 {
        return Ok(());
    }
    r.skip(9)?; // the four tower downeds + four tower actives + LunarApocalypseIsUp
                // WorldFile.cs:2195-2206 — the party snapshot (cooldown i32, then a counted i32 list).
    if version >= 170 {
        r.skip(1)?; // _tempPartyManual
        r.skip(1)?; // _tempPartyGenuine
        r.skip(4)?; // _tempPartyCooldown
        let celebrants = r.i32()?;
        r.skip((celebrants.max(0) as usize) * 4)?;
    }
    // WorldFile.cs:2207-2213 — the sandstorm snapshot.
    if version >= 174 {
        r.skip(1)?; // happening
        r.skip(4)?; // timeLeft
        r.skip(4)?; // severity
        r.skip(4)?; // intendedSeverity
    }
    // WorldFile.cs:2214-2218 — `DD2Event.Load`: below 178 nothing is read at all.
    if version >= 178 {
        r.skip(1)?; // savedBartender
        r.skip(3)?; // DownedInvasionT1..T3
    }
    // WorldFile.cs:2219-2233 — the remaining background styles.
    if version > 194 {
        r.skip(1)?; // setBG(8) mushroomBG
    }
    if version >= 215 {
        r.skip(1)?; // setBG(9) underworldBG
    }
    if version > 195 {
        r.skip(3)?; // setBG(10..12) treeBG2..4
    }
    if version >= 204 {
        r.skip(1)?; // combatBookWasUsed
    }
    // WorldFile.cs:2234-2239 — the lantern night snapshot.
    if version >= 207 {
        r.skip(4)?; // _tempLanternNightCooldown
        r.skip(1)?; // _tempLanternNightGenuine
        r.skip(1)?; // _tempLanternNightManual
        r.skip(1)?; // _tempLanternNightNextNightIsGenuine
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
        r.skip(4 * 4)?; // Copper/Iron/Silver/Gold
    }
    if version >= 217 {
        r.skip(3)?; // boughtCat/Dog/Bunny
    }
    if version >= 223 {
        r.skip(2)?; // downedEmpressOfLight, downedQueenSlime
    }
    if version >= 240 {
        world.downed_deerclops = r.bool()?;
    }
    if version >= 250 {
        r.skip(1)?; // unlockedSlimeBlueSpawn
    }
    if version >= 251 {
        r.skip(8)?; // unlockedMerchantSpawn .. unlockedPrincessSpawn
    }
    if version >= 259 {
        r.skip(1)?; // combatBookVolumeTwoWasUsed
    }
    if version >= 260 {
        r.skip(1)?; // peddlersSatchelWasUsed
    }
    if version >= 261 {
        r.skip(3)?; // unlockedSlimeGreen/Old/Purple
        world.unlocked_slime_rainbow_spawn = r.bool()?; // unlockedSlimeRainbowSpawn
        r.skip(3)?; // unlockedSlimeRed/Yellow/Copper
    }
    // WorldFile.cs:2306-2319 — the moondial, the forced seasons, the secret seeds, the
    // meteor shower, the team spawns and the extra spawn points.
    if version >= 264 {
        world.fast_forward_to_dusk = r.bool()?;
        world.moondial_cooldown = r.u8()?;
    }
    if version >= 287 {
        r.skip(2)?; // forceHalloweenForever, forceXMasForever
    }
    if version >= 288 {
        r.skip(1)?; // vampireSeed
    }
    if version >= 296 {
        r.skip(1)?; // infectedSeed
    }
    if version >= 291 {
        r.skip(2 * 4)?; // _tempMeteorShowerCount, _tempCoinRain
    }
    if version >= 297 {
        r.skip(1)?; // teamBasedSpawnsSeed
        let extra = r.u8()? as i32;
        r.skip((extra.max(0) as usize) * 4)?; // ExtraSpawnPointManager.Read: (i16, i16) pairs
    }
    if version >= 304 {
        r.skip(1)?; // dualDungeonsSeed
    }
    if version >= 323 {
        r.skip(1)?; // moreLightningSeed
        r.skip(1)?; // noLightningSeed
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

/// Skips `count` .NET strings (the angler list).
fn skip_strings(r: &mut Reader, count: i32) -> Result<()> {
    for _ in 0..count.max(0) {
        skip_one_string(r)?;
    }
    Ok(())
}
