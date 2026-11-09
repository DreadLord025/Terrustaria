//! Native save format (`TRWLD`) for generated worlds.
//!
//! Deliberately tiny and lossless for everything the renderer needs: header fields,
//! the frame-importance bitset and the full 14-byte tile struct, RLE-compressed per
//! row. The original `.wld` writer is a much larger job; this format exists so the
//! generator can persist a world under `Documents/My Games/Terrustaria`.

use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};

use crate::tile::Tile;
use crate::tile_entities::TileEntities;
use crate::wiring::WireState;
use crate::world::World;

const MAGIC: &[u8; 8] = b"TRWLD01\0";
/// v2 added `moon_type`/`moon_phase`; v1 files still load with 0/0. v3 added
/// the sparse paint/coating extras; v4 appended the two dial cooldown bytes;
/// v5 appended the bed spawn (the vanilla keeps it in the *player* file's
/// `spX/spY` arrays, `Player.cs:54877-54888` — the port's single-player world
/// save carries it world-side, `World::bed_spawn`); v6 appended the world
/// flags block (the C# `SaveWorldFlags` progression subset, `WorldFile.cs:1238-1430`),
/// the wire side table and the tile entities (`SaveTileEntities`, `WorldFile.cs:3302-3313`)
/// and the world chests (`SaveChests`, `WorldFile.cs:1606-1676`). The v6 tail on this
/// branch instead carries the classic secret-seed flags (`World.seeds` — the
/// `SaveWorldFlags` byte block, `WorldFile.cs:1238-1250`, as the port's own u16
/// bitmask; the C# `.wld` stores the nine booleans and re-derives the seed from the
/// stored seed *text*), so the v6 block below is re-assembled as v7. v8 appended the
/// banner kill counts (`BannerSystem.Save`, `BannerSystem.cs:172-186`); v9 appended the
/// bestiary kill counts (`NPCKillsTracker.Save`, `NPCKillsTracker.cs:59-70`); v10 appended
/// the town NPCs (`SaveNPCs`'s block B, `WorldFile.cs:1703-1750`); v11 appended the
/// saved ore tiers (`WorldGen.SavedOreTiers`, the C#'s world-flags bytes,
/// `WorldFile.cs:1315-1321` + `:1444-1450`).
pub const VERSION: u32 = 11;

/// The v7+ sections' payload — the world chests, the banner kill counts, the
/// bestiary kill counts and the town NPCs. New sections grow this struct
/// rather than the entry points' return tuples.
#[derive(Default)]
pub struct WorldSide {
    pub chests: Vec<ChestRecord>,
    pub banners: Vec<BannerKill>,
    pub bestiary: Vec<BestiaryKill>,
    pub npcs: Vec<NpcRecord>,
    /// `WorldGen.SavedOreTiers` (`WorldGen.cs:3283-3297`) — seven `i32` metal
    /// tiers; `None` for a file written before v11, whose World carries the
    /// static defaults (the C#'s own reset arm, `WorldFile.cs:796-814`).
    pub ore_tiers: Option<[i32; 7]>,
}

/// One town NPC — the C# `SaveNPCs` block-B record (`active`, `netID`,
/// `givenName`, `position.X/Y`, `homeless`, `homeTileX/Y`, the variation
/// `BitsByte`, `homelessDespawn`, `WorldFile.cs:1703-1750`). The port stores
/// its own internal kind id (not the netID), its `Npc` carries no variation
/// index beyond the slimes', and `homelessDespawn` is a runtime flag the C#
/// saves for its load validation only — all three recorded in the doc.
#[derive(Clone, Debug, PartialEq)]
pub struct NpcRecord {
    /// The port's internal `NpcKind` id (`npc::NpcKind::to_u16`) — the
    /// vanilla record carries the netID; the port-native file keeps its own
    /// identity, and the town roster's ids round-trip through the same table.
    pub kind: u16,
    /// `NPC.GivenName` (`NPC.cs:6497`); empty = the kind's static name.
    pub given_name: String,
    /// The pixel position (`NPC.position.X/Y`, the f32 pair).
    pub x: f32,
    pub y: f32,
    /// `NPC.homeless` inverted (the record is written once per live town NPC,
    /// which the C# addresses through its `Main.npc` array's townNPC flag).
    pub homeless: bool,
    /// `NPC.homeTileX/Y` while housed.
    pub home_x: i32,
    pub home_y: i32,
}

/// One banner's kill count — the C# `BannerSystem.Save` shape (`int32` per
/// banner id, `BannerSystem.cs:172-186`); the port stores the counts sparsely
/// (a `BTreeMap` by banner id) where the vanilla keeps a dense array, so each
/// record carries its id.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BannerKill {
    pub banner: i32,
    pub kills: i32,
}

/// One bestiary entry's kill count — the C# `NPCKillsTracker.Save` shape
/// (the persistent-id string + `int32` per entry, `NPCKillsTracker.cs:59-70`);
/// both stores key by the credit id, so each record carries its string.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BestiaryKill {
    pub credit_id: String,
    pub kills: i32,
}

/// One item inside a chest record — the C# `SaveChests` slot shape
/// (`int16 stack`, then `int32 itemType` + `byte prefix` while the stack is
/// positive, `WorldFile.cs:1606-1676`); the stack is widened to `i32` for
/// the port's `u32` counts (the vanilla cap is 9999, far inside `i16`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChestItem {
    pub stack: i32,
    pub kind: u16,
    pub prefix: u8,
}

/// One chest record — the anchor tile, the effective name and the slots.
///
/// The slots keep the port's implicit-air compression: a `stack == 0` entry
/// is an empty slot and trailing airs are dropped, where the C# always
/// writes `maxItems` slots (40) — the port's store materializes them on
/// open (`chest::open_chest`'s `resize`), so the compressed vec round-trips
/// exactly. `maxItems` itself is the port's frozen `DEFAULT_MAX_ITEMS` (40).
#[derive(Clone, Debug, PartialEq)]
pub struct ChestRecord {
    pub x: i32,
    pub y: i32,
    pub name: String,
    pub slots: Vec<ChestItem>,
}

pub fn default_world_dir() -> Option<PathBuf> {
    let home = std::env::var_os("USERPROFILE")?;
    Some(
        PathBuf::from(home)
            .join("Documents")
            .join("My Games")
            .join("Terrustaria")
            .join("Worlds"),
    )
}

pub fn default_world_path(name: &str) -> Option<PathBuf> {
    Some(default_world_dir()?.join(format!("{name}.trwld")))
}

/// `Main.PlayerPath` (`Main.cs:67065-67075`): `SavePath/Players` — the
/// `Worlds` dir's parent joined with `Players` (`Terrustaria/Players`).
pub fn default_players_dir() -> Option<PathBuf> {
    Some(default_world_dir()?.parent()?.join("Players"))
}

/// `AchievementManager`'s save path (`Main.SavePath/achievements.dat`) —
/// the port writes its own text format beside the vanilla name
/// (`achievements.trdat`; M6-ACH-1's documented deviation).
pub fn achievements_path() -> Option<PathBuf> {
    Some(default_world_dir()?.parent()?.join("achievements.trdat"))
}

/// `Main.GetPlayerPathFromName` (`Main.cs:4820-4841`): `.` and `*` become
/// `_`, the Windows invalid-filename set becomes `-`, spaces become `_`,
/// then `<name>.plr` under [`default_players_dir`].
pub fn default_player_path(name: &str) -> Option<PathBuf> {
    const INVALID: &[char] = &['"', '<', '>', '|', ':', '?', '\\', '/', '\u{0}'];
    let mut safe = String::with_capacity(name.len());
    for c in name.chars() {
        safe.push(if c == '.' || c == '*' {
            '_'
        } else if INVALID.contains(&c) {
            '-'
        } else if c == ' ' {
            '_'
        } else {
            c
        });
    }
    Some(default_players_dir()?.join(format!("{safe}.plr")))
}

pub fn save(path: &Path, world: &World, side: &WorldSide) -> Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
    }
    let tmp = path.with_extension("trwld.tmp");
    {
        let file =
            std::fs::File::create(&tmp).with_context(|| format!("creating {}", tmp.display()))?;
        let mut w = std::io::BufWriter::new(file);
        write_world(&mut w, world, side)?;
        w.flush()?;
    }
    std::fs::rename(&tmp, path).with_context(|| format!("replacing {}", path.display()))?;
    Ok(())
}

/// Serialises a world to memory (multiplayer world transfer).
pub fn to_bytes(world: &World, side: &WorldSide) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    write_world(&mut out, world, side)?;
    Ok(out)
}

fn write_world(w: &mut impl Write, world: &World, side: &WorldSide) -> Result<()> {
    w.write_all(MAGIC)?;
    w.write_all(&VERSION.to_le_bytes())?;
    w.write_all(&world.seed.to_le_bytes())?;
    let name = world.name.as_bytes();
    w.write_all(&(name.len() as u32).to_le_bytes())?;
    w.write_all(name)?;
    w.write_all(&world.width.to_le_bytes())?;
    w.write_all(&world.height.to_le_bytes())?;
    w.write_all(&world.spawn_x.to_le_bytes())?;
    w.write_all(&world.spawn_y.to_le_bytes())?;
    w.write_all(&world.world_surface.to_le_bytes())?;
    w.write_all(&world.rock_layer.to_le_bytes())?;
    w.write_all(&world.time.to_le_bytes())?;
    w.write_all(&[u8::from(world.day_time)])?;
    w.write_all(&[world.moon_type, world.moon_phase])?;
    let imp = &world.frame_important;
    w.write_all(&(imp.len() as u32).to_le_bytes())?;
    let packed: Vec<u8> = imp.iter().map(|&b| u8::from(b)).collect();
    w.write_all(&packed)?;

    let width = world.width as usize;
    let mut run_tiles: Vec<Tile> = Vec::new();
    let mut run_counts: Vec<u16> = Vec::new();
    for y in 0..world.height as usize {
        run_tiles.clear();
        run_counts.clear();
        let row = &world.tiles[y * width..(y + 1) * width];
        for &t in row {
            if let Some(last) = run_tiles.last() {
                if *last == t && *run_counts.last().unwrap() < u16::MAX {
                    *run_counts.last_mut().unwrap() += 1;
                    continue;
                }
            }
            run_tiles.push(t);
            run_counts.push(1);
        }
        w.write_all(&(run_tiles.len() as u32).to_le_bytes())?;
        for (t, n) in run_tiles.iter().zip(run_counts.iter()) {
            write_tile(w, t)?;
            w.write_all(&n.to_le_bytes())?;
        }
    }

    // v3: the sparse paint/coating extras (`World.extras`) — wall paint,
    // invisible block/wall, fullbright wall. Sorted by index for determinism.
    let mut extras: Vec<(u32, u8)> = world.extras.iter().map(|(k, v)| (*k, *v)).collect();
    extras.sort_unstable();
    w.write_all(&(extras.len() as u32).to_le_bytes())?;
    for (idx, byte) in extras {
        w.write_all(&idx.to_le_bytes())?;
        w.write_all(&[byte])?;
    }
    // v4: the dial cooldowns (`Main.sundialCooldown`/`Main.moondialCooldown`,
    // `Main.cs:2161`/`:2165`); 0..=8 each. The fast-forward flags stay
    // runtime-only — the C# `.wld` has written `fastForwardTimeToDusk` since
    // v264, but the request is transient by nature.
    w.write_all(&[world.sundial_cooldown, world.moondial_cooldown])?;
    // v5: the bed spawn (`ChangeSpawn`'s cell — the CheckSpawn-validated
    // ground cell under the bed, `Player.cs:54635-54660`).
    match world.bed_spawn {
        Some((x, y)) => {
            w.write_all(&[1u8])?;
            w.write_all(&x.to_le_bytes())?;
            w.write_all(&y.to_le_bytes())?;
        }
        None => w.write_all(&[0u8])?,
    }
    // v6 (re-assembled as v7): the classic secret-seed flags
    // (`WorldFile.SaveWorldFlags`'s nine `Main.*` booleans,
    // `WorldFile.cs:1238-1250`) as the port's own `serializedSeedsSum`
    // bitmask (`WorldFileData.cs:157-197`).
    w.write_all(&world.seeds.sum().to_le_bytes())?;
    // v7: the world flags block, then the wires, the tile entities and the
    // chests — the four port-native sections whose C# counterparts ride
    // `SaveWorldFlags` / the per-tile header bits / `SaveTileEntities` /
    // `SaveChests`.
    write_flags_v6(w, world)?;
    let mut sec = crate::binio::Writer::new();
    world.wires.write(&mut sec);
    w.write_all(&sec.into_inner())?;
    let mut sec = crate::binio::Writer::new();
    world.tile_entities.write_all(&mut sec);
    w.write_all(&sec.into_inner())?;
    // v7: the world chests — `SaveChests`'s shape (`WorldFile.cs:1606-1676`):
    // a u32 count, then per chest the anchor, the name and the slots.
    let chests = &side.chests;
    w.write_all(&(chests.len() as u32).to_le_bytes())?;
    for chest in chests {
        w.write_all(&chest.x.to_le_bytes())?;
        w.write_all(&chest.y.to_le_bytes())?;
        // The name rides a u32 length + UTF-8, the file's own string
        // convention (the world name at the stream head).
        let name = chest.name.as_bytes();
        w.write_all(&(name.len() as u32).to_le_bytes())?;
        w.write_all(name)?;
        w.write_all(&(chest.slots.len() as u32).to_le_bytes())?;
        for slot in &chest.slots {
            w.write_all(&slot.stack.to_le_bytes())?;
            if slot.stack != 0 {
                w.write_all(&slot.kind.to_le_bytes())?;
                w.write_all(&[slot.prefix])?;
            }
        }
    }
    // v8: the banner kill counts — a u32 count, then per banner the id and
    // the count (the port's sparse map where the vanilla writes a dense array).
    w.write_all(&(side.banners.len() as u32).to_le_bytes())?;
    for banner in &side.banners {
        w.write_all(&banner.banner.to_le_bytes())?;
        w.write_all(&banner.kills.to_le_bytes())?;
    }
    // v9: the bestiary kill counts — `NPCKillsTracker.Save`'s shape
    // (`NPCKillsTracker.cs:59-70`): a u32 count, then per entry the
    // persistent-id string (the file's u32 length + UTF-8 convention) and
    // the count. The vanilla rides them in the world file's section 10; the
    // port had them on the `.trplayer` sidecar (the recorded deviation), and
    // that line stays the back-compat fallback.
    w.write_all(&(side.bestiary.len() as u32).to_le_bytes())?;
    for entry in &side.bestiary {
        let id = entry.credit_id.as_bytes();
        w.write_all(&(id.len() as u32).to_le_bytes())?;
        w.write_all(id)?;
        w.write_all(&entry.kills.to_le_bytes())?;
    }
    // v10: the town NPCs — `SaveNPCs`'s block-B record shape
    // (`WorldFile.cs:1703-1750`): the kind, the given name, the pixel
    // position, the homeless flag and the home tile. The block-A shimmer
    // list and block-C `SavesAndLoads` passives ride the C#'s arrays the port
    // does not model (documented in ANALYSIS).
    w.write_all(&(side.npcs.len() as u32).to_le_bytes())?;
    for npc in &side.npcs {
        w.write_all(&npc.kind.to_le_bytes())?;
        let name = npc.given_name.as_bytes();
        w.write_all(&(name.len() as u32).to_le_bytes())?;
        w.write_all(name)?;
        w.write_all(&npc.x.to_le_bytes())?;
        w.write_all(&npc.y.to_le_bytes())?;
        w.write_all(&[u8::from(npc.homeless)])?;
        w.write_all(&npc.home_x.to_le_bytes())?;
        w.write_all(&npc.home_y.to_le_bytes())?;
    }
    // v11: the saved ore tiers — the seven `i32` tiers in the `SavedOreTiers`
    // field order (the C# rides them in the world-flags block,
    // `WorldFile.cs:1315-1321` for the hardmode trio + `:1444-1450` for the
    // pre-hardmode quartet; the port's own section, the same values).
    w.write_all(&[side.ore_tiers.is_some() as u8])?;
    if let Some(tiers) = &side.ore_tiers {
        for t in tiers {
            w.write_all(&t.to_le_bytes())?;
        }
    }
    Ok(())
}

/// The fixed size of the v6 flags block (16 + 16 bools, 14 i32s, 2 f64s, 1 f32) —
/// a back-compat test truncates exactly this many bytes.
const V6_FLAGS_LEN: usize = 16 + 16 + 14 * 4 + 2 * 8 + 4;

/// v6: the flags block — the port's `World` fields that the C#
/// `SaveWorldFlags` persists (`WorldFile.cs:1238-1430`), grouped with the C#
/// line citations. The port's extra state that the C# does not save (the
/// slime-rain flag/kill counter, the DD2 wave counter, the tower shields as
/// live values) rides its group with the deviation noted; the runtime-only
/// fields (the WoF/Golem/boss write-through anchors, `travel_shop`, the
/// App-side weather block) stay out.
fn write_flags_v6(w: &mut impl Write, world: &World) -> Result<()> {
    // `_tempBloodMoon`/`_tempEclipse` (`:1277-1278`), the downed-boss ladder
    // (`:1282-1294`: Boss1/2/3, QueenBee, MechBoss1/2/3, PlantBoss, GolemBoss,
    // SlimeKing, savedGoblin — the port has no MechBossAny/savedWizard/savedMech
    // state), the invasion downed flags (`:1296-1299`) and
    // `unlockedSlimeBlueSpawn` (`:1398`).
    w.write_all(&[
        u8::from(world.blood_moon),
        u8::from(world.eclipse),
        u8::from(world.downed_eoc),
        u8::from(world.downed_eow),
        u8::from(world.downed_skeletron),
        u8::from(world.downed_queen_bee),
        u8::from(world.downed_mech_boss1),
        u8::from(world.downed_mech_boss2),
        u8::from(world.downed_mech_boss3),
        u8::from(world.downed_plant_boss),
        u8::from(world.downed_golem),
        u8::from(world.downed_slime_king),
        u8::from(world.saved_goblin),
        u8::from(world.downed_goblins),
        u8::from(world.downed_frost),
        u8::from(world.downed_pirates),
    ])?;
    // `shadowOrbSmashed`/`spawnMeteor`/`hardMode` (`:1300-1304`), the DD2
    // event flag (`DD2Event.Save`, `DD2Event.cs:151-156` — the port carries the
    // live event state instead of only the tier flags), `downedMoonlord`
    // (`:1346`), `slimeRainTime`'s companion flag and kill counter (port-only),
    // `TowerActive*` (`:1357-1360`) and `downedTower*` (`:1352-1355`).
    w.write_all(&[
        u8::from(world.shadow_orb_smashed),
        u8::from(world.spawn_meteor),
        u8::from(world.hard_mode),
        u8::from(world.dd2_event_ongoing),
        u8::from(world.dd2_lost),
        u8::from(world.downed_moon_lord),
        u8::from(world.slime_rain),
        u8::from(world.tower_active_solar),
        u8::from(world.tower_active_vortex),
        u8::from(world.tower_active_nebula),
        u8::from(world.tower_active_stardust),
        u8::from(world.downed_tower_solar),
        u8::from(world.downed_tower_vortex),
        u8::from(world.downed_tower_nebula),
        u8::from(world.downed_tower_stardust),
        0, // reserved — keeps the two bool groups 16 bytes each
    ])?;
    // `shadowOrbCount`/`altarCount` (`:1302-1303`), the invasion block
    // (`:1306-1309` + `invasionSizeStart` `:1339`), the slime-rain counter
    // (port-only), the DD2 wave counters (port-only) and the tower shields —
    // the C# load restores them from `ShieldStrengthTowerMax` while
    // `LunarApocalypseIsUp` (`:2237-2249`); the port persists the live values.
    for v in [
        world.shadow_orb_count,
        world.altars_smashed,
        world.invasion_delay,
        world.invasion_size,
        world.invasion_type,
        world.invasion_size_start,
        world.slime_rain_kills,
        world.dd2_wave,
        world.dd2_spawn_hold,
        world.tower_shield_solar,
        world.tower_shield_vortex,
        world.tower_shield_nebula,
        world.tower_shield_stardust,
        world.lunar_shield_power_normal,
    ] {
        w.write_all(&v.to_le_bytes())?;
    }
    w.write_all(&world.invasion_x.to_le_bytes())?;
    w.write_all(&world.slime_rain_time.to_le_bytes())?;
    w.write_all(&world.dd2_wave_kills.to_le_bytes())?;
    Ok(())
}

/// The v6 flags block reader — the exact inverse of `write_flags_v6`.
fn read_flags_v6(r: &mut Reader<'_>, world: &mut World) -> Result<()> {
    let a = r.take(16)?;
    world.blood_moon = a[0] != 0;
    world.eclipse = a[1] != 0;
    world.downed_eoc = a[2] != 0;
    world.downed_eow = a[3] != 0;
    world.downed_skeletron = a[4] != 0;
    world.downed_queen_bee = a[5] != 0;
    world.downed_mech_boss1 = a[6] != 0;
    world.downed_mech_boss2 = a[7] != 0;
    world.downed_mech_boss3 = a[8] != 0;
    world.downed_plant_boss = a[9] != 0;
    world.downed_golem = a[10] != 0;
    world.downed_slime_king = a[11] != 0;
    world.saved_goblin = a[12] != 0;
    world.downed_goblins = a[13] != 0;
    world.downed_frost = a[14] != 0;
    world.downed_pirates = a[15] != 0;
    let b = r.take(16)?;
    world.shadow_orb_smashed = b[0] != 0;
    world.spawn_meteor = b[1] != 0;
    world.hard_mode = b[2] != 0;
    world.dd2_event_ongoing = b[3] != 0;
    world.dd2_lost = b[4] != 0;
    world.downed_moon_lord = b[5] != 0;
    world.slime_rain = b[6] != 0;
    world.tower_active_solar = b[7] != 0;
    world.tower_active_vortex = b[8] != 0;
    world.tower_active_nebula = b[9] != 0;
    world.tower_active_stardust = b[10] != 0;
    world.downed_tower_solar = b[11] != 0;
    world.downed_tower_vortex = b[12] != 0;
    world.downed_tower_nebula = b[13] != 0;
    world.downed_tower_stardust = b[14] != 0;
    for field in [
        &mut world.shadow_orb_count,
        &mut world.altars_smashed,
        &mut world.invasion_delay,
        &mut world.invasion_size,
        &mut world.invasion_type,
        &mut world.invasion_size_start,
        &mut world.slime_rain_kills,
        &mut world.dd2_wave,
        &mut world.dd2_spawn_hold,
        &mut world.tower_shield_solar,
        &mut world.tower_shield_vortex,
        &mut world.tower_shield_nebula,
        &mut world.tower_shield_stardust,
        &mut world.lunar_shield_power_normal,
    ] {
        *field = r.i32()?;
    }
    world.invasion_x = r.f64()?;
    world.slime_rain_time = r.f64()?;
    world.dd2_wave_kills = r.f32()?;
    Ok(())
}

fn write_tile(w: &mut impl Write, t: &Tile) -> Result<()> {
    w.write_all(&t.kind.to_le_bytes())?;
    w.write_all(&t.wall.to_le_bytes())?;
    w.write_all(&t.frame_x.to_le_bytes())?;
    w.write_all(&t.frame_y.to_le_bytes())?;
    w.write_all(&[
        t.liquid,
        t.flags,
        t.slope(),
        t.paint(),
        t.wall_col(),
        t.wall_row(),
    ])?;
    Ok(())
}

pub fn load(path: &Path) -> Result<(World, WorldSide)> {
    let data = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    parse(&data).with_context(|| format!("parsing {}", path.display()))
}

/// The side sections of a parsed save — the chests and the banners alone
/// (the world is parsed and dropped, so callers that only walk the stores
/// keep one entry point).
pub fn parse_side(data: &[u8]) -> Result<WorldSide> {
    parse(data).map(|(_, side)| side)
}

pub fn parse(data: &[u8]) -> Result<(World, WorldSide)> {
    let mut r = Reader::new(data);
    if r.take(8)? != MAGIC {
        bail!("not a TRWLD save");
    }
    let version = r.u32()?;
    if version == 0 || version > VERSION {
        bail!("unsupported TRWLD version {version}");
    }
    let seed = r.u64()?;
    let name_len = r.u32()? as usize;
    let name = String::from_utf8_lossy(r.take(name_len)?).into_owned();
    let width = r.i32()?;
    let height = r.i32()?;
    if width <= 0 || height <= 0 || width > 20_000 || height > 20_000 {
        bail!("implausible world size {width}x{height}");
    }
    let mut world = World::new(name, width, height);
    world.seed = seed;
    world.spawn_x = r.i32()?;
    world.spawn_y = r.i32()?;
    world.world_surface = r.f64()?;
    world.rock_layer = r.f64()?;
    world.time = r.f64()?;
    world.day_time = r.u8()? != 0;
    if version >= 2 {
        world.moon_type = r.u8()?;
        world.moon_phase = r.u8()?;
    }
    let imp_len = r.u32()? as usize;
    if imp_len > 1 << 16 {
        bail!("bad importance table length {imp_len}");
    }
    world.frame_important = r.take(imp_len)?.iter().map(|&b| b != 0).collect();

    let w = width as usize;
    for y in 0..height as usize {
        let runs = r.u32()? as usize;
        let mut x = 0usize;
        for _ in 0..runs {
            let t = read_tile(&mut r)?;
            let n = r.u16()? as usize;
            if x + n > w {
                bail!("RLE run overflows row {y}");
            }
            let start = y * w + x;
            world.tiles[start..start + n].fill(t);
            x += n;
        }
        if x != w {
            bail!("row {y} has {x} of {w} tiles");
        }
    }
    if version >= 3 {
        let count = r.u32()? as usize;
        for _ in 0..count {
            let idx = r.u32()? as usize;
            let byte = r.u8()?;
            if idx >= world.tiles.len() {
                bail!("extras index {idx} out of range");
            }
            world.set_extras_at(idx, byte);
        }
    }
    // v4: the dial cooldowns; a v3 file (and older) loads with the 0s from
    // `World::new`.
    if version >= 4 {
        world.sundial_cooldown = r.u8()?;
        world.moondial_cooldown = r.u8()?;
    }
    // v5: the bed spawn. A corrupt/out-of-range cell degrades to `None` —
    // the payload rides disk reads (the load is a trust boundary) and a
    // dropped spawn point only falls back to the world spawn
    // (`Player.cs:37571-37575`'s `SpawnX = -1` arm).
    if version >= 5 && r.u8()? != 0 {
        let x = r.i32()?;
        let y = r.i32()?;
        if x >= 10 && x < world.width - 10 && y >= 10 && y < world.height - 10 {
            world.bed_spawn = Some((x, y));
        }
    }
    // v6 (re-assembled as v7): the secret-seed flags. A v5 file loads with
    // the classic (all-false) world, which is what the pre-M5-SEED-1 saves
    // were.
    if version >= 6 {
        world.seeds = crate::seeds::SeedFlags::from_sum(r.u16()?);
    }
    // v7: the flags block, the wires, the tile entities and the chests. A v6
    // file (and older) keeps the `World::new` defaults — the same contract as
    // the earlier tails. The wire/TE/chest sections are self-delimiting
    // (count + records), so a truncated one bails like every other structural
    // failure.
    let mut side = WorldSide::default();
    if version >= 7 {
        read_flags_v6(&mut r, &mut world)?;
        world.wires = WireState::read(&mut r.0).ok_or_else(|| eof(r.0.pos()))?;
        let tes = TileEntities::read_all(&mut r.0, &world).ok_or_else(|| eof(r.0.pos()))?;
        world.tile_entities = tes;
        // v7: the world chests (`LoadChests`'s shape `WorldFile.cs:2723-2805` —
        // the port keeps the store's implicit-air compression instead of the
        // vanilla fixed `maxItems`, and drops the vanilla's pre-count validity
        // sweep: the anchors are re-checked by the app when the store is built).
        side.chests = read_chests_v7(&mut r)?;
    }
    // v8: the banner kill counts (`BannerSystem.Save`'s `int32` per banner id,
    // `BannerSystem.cs:172-186` — the vanilla rides the world-flags block; the
    // port keeps the counts sparse, so each record carries its banner id).
    if version >= 8 {
        side.banners = read_banners_v8(&mut r)?;
    }
    // v9: the bestiary kill counts (the C#'s world-file section 10 — the
    // side sections' counting order is the C#'s `SaveWorld_Version2` order
    // wherever the port has the state).
    if version >= 9 {
        side.bestiary = read_bestiary_v9(&mut r)?;
    }
    // v10: the town NPCs (the C#'s block-B records; the load-side
    // `ValidateLoadNPCs` skip of unknown ids the app re-checks).
    if version >= 10 {
        side.npcs = read_npcs_v10(&mut r)?;
    }
    // v11: the saved ore tiers; a pre-v11 file leaves them at the World's
    // defaults (the C#'s reset arm).
    if version >= 11 {
        if r.u8()? != 0 {
            let mut tiers = [0i32; 7];
            for t in &mut tiers {
                *t = r.i32()?;
            }
            side.ore_tiers = Some(tiers);
            world.ore_tiers = crate::ores::SavedOreTiers {
                copper: tiers[0],
                iron: tiers[1],
                silver: tiers[2],
                gold: tiers[3],
                cobalt: tiers[4],
                mythril: tiers[5],
                adamantite: tiers[6],
            };
        }
    }
    world.revision = 0;
    Ok((world, side))
}

/// The v8 banners reader — the exact inverse of the v8 write.
fn read_banners_v8(r: &mut Reader<'_>) -> Result<Vec<BannerKill>> {
    let count = r.u32()? as usize;
    let mut out = Vec::with_capacity(count.min(4096));
    for _ in 0..count {
        let banner = r.i32()?;
        let kills = r.i32()?;
        out.push(BannerKill { banner, kills });
    }
    Ok(out)
}

/// The v9 bestiary reader — the exact inverse of the v9 write.
fn read_bestiary_v9(r: &mut Reader<'_>) -> Result<Vec<BestiaryKill>> {
    let count = r.u32()? as usize;
    let mut out = Vec::with_capacity(count.min(4096));
    for _ in 0..count {
        let id_len = r.u32()? as usize;
        let credit_id = String::from_utf8_lossy(r.take(id_len)?).into_owned();
        let kills = r.i32()?;
        out.push(BestiaryKill { credit_id, kills });
    }
    Ok(out)
}

/// The v10 NPC reader — the exact inverse of the v10 write.
fn read_npcs_v10(r: &mut Reader<'_>) -> Result<Vec<NpcRecord>> {
    let count = r.u32()? as usize;
    let mut out = Vec::with_capacity(count.min(1024));
    for _ in 0..count {
        let kind = r.u16()?;
        let name_len = r.u32()? as usize;
        let given_name = String::from_utf8_lossy(r.take(name_len)?).into_owned();
        let x = r.f32()?;
        let y = r.f32()?;
        let homeless = r.u8()? != 0;
        let home_x = r.i32()?;
        let home_y = r.i32()?;
        out.push(NpcRecord { kind, given_name, x, y, homeless, home_x, home_y });
    }
    Ok(out)
}

/// The v7 chests reader — the exact inverse of the v7 write.
fn read_chests_v7(r: &mut Reader<'_>) -> Result<Vec<ChestRecord>> {
    let count = r.u32()? as usize;
    let mut out = Vec::with_capacity(count.min(8192));
    for _ in 0..count {
        let x = r.i32()?;
        let y = r.i32()?;
        let name_len = r.u32()? as usize;
        let name = String::from_utf8_lossy(r.take(name_len)?).into_owned();
        let slots = r.u32()? as usize;
        let mut items = Vec::with_capacity(slots.min(64));
        for _ in 0..slots {
            let stack = r.i32()?;
            if stack == 0 {
                items.push(ChestItem { stack: 0, kind: 0, prefix: 0 });
                continue;
            }
            items.push(ChestItem {
                stack,
                kind: r.u16()?,
                prefix: r.u8()?,
            });
        }
        out.push(ChestRecord { x, y, name, slots: items });
    }
    Ok(out)
}

fn read_tile(r: &mut Reader) -> Result<Tile> {
    let mut tile = Tile {
        kind: r.u16()?,
        wall: r.u16()?,
        frame_x: r.i16()?,
        frame_y: r.i16()?,
        liquid: r.u8()?,
        flags: r.u8()?,
        ..Default::default()
    };
    let slope = r.u8()?;
    let paint = r.u8()?;
    let col = r.u8()?;
    let row = r.u8()?;
    tile.set_slope(slope);
    tile.set_paint(paint);
    tile.set_wall_frame(col, row);
    Ok(tile)
}

/// Thin `anyhow` adapter over `crate::binio::Reader`: a short read becomes the
/// parser's original `unexpected end of file at offset N` bail. All decoding
/// lives in `crate::binio`.
struct Reader<'a>(crate::binio::Reader<'a>);

fn eof(pos: usize) -> anyhow::Error {
    anyhow!("unexpected end of file at offset {pos}")
}

impl<'a> Reader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self(crate::binio::Reader::new(data))
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        self.0.take(n).ok_or_else(|| eof(self.0.pos()))
    }

    fn u8(&mut self) -> Result<u8> {
        self.0.u8().ok_or_else(|| eof(self.0.pos()))
    }

    fn u16(&mut self) -> Result<u16> {
        self.0.u16().ok_or_else(|| eof(self.0.pos()))
    }

    fn i16(&mut self) -> Result<i16> {
        self.0.i16().ok_or_else(|| eof(self.0.pos()))
    }

    fn u32(&mut self) -> Result<u32> {
        self.0.u32().ok_or_else(|| eof(self.0.pos()))
    }

    fn i32(&mut self) -> Result<i32> {
        self.0.i32().ok_or_else(|| eof(self.0.pos()))
    }

    fn u64(&mut self) -> Result<u64> {
        self.0.u64().ok_or_else(|| eof(self.0.pos()))
    }

    fn f64(&mut self) -> Result<f64> {
        self.0.f64().ok_or_else(|| eof(self.0.pos()))
    }

    fn f32(&mut self) -> Result<f32> {
        self.0.f32().ok_or_else(|| eof(self.0.pos()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::FLAG_ACTIVE;
    use crate::tile_entities::{TeData, TeItem, TeKind};
    use crate::wiring::WireColor;

    #[test]
    fn round_trips_tiles_and_header() {
        let mut w = World::new("Test".into(), 64, 48);
        w.seed = 99;
        w.world_surface = 12.5;
        w.rock_layer = 30.0;
        w.spawn_x = 32;
        w.spawn_y = 10;
        w.time = 27000.0;
        w.moon_type = 5;
        w.moon_phase = 3;
        w.frame_important = vec![false; 754];
        w.frame_important[5] = true;
        for x in 0..64 {
            let i = w.index(x, 20);
            w.tiles[i] = Tile {
                kind: 2,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        let i = w.index(5, 30);
        w.tiles[i] = Tile {
            kind: 0,
            wall: 2,
            frame_x: 18,
            frame_y: 54,
            liquid: 200,
            flags: FLAG_ACTIVE | crate::tile::FLAG_LIQUID_LAVA,
            ..Default::default()
        };
        w.tiles[i].set_slope(2);
        w.tiles[i].set_paint(3);
        w.tiles[i].set_wall_frame(4, 2);
        // v3 extras: wall paint + the Echo bit on (3, 0); the zero byte at
        // (7, 7) collapses back to absent.
        w.set_wall_paint(3, 0, 4);
        w.set_invisible_block(3, 0, true);
        w.set_extras_at(w.index(7, 7), 0);
        let mut buf = Vec::new();
        write_world(&mut buf, &w, &WorldSide::default()).unwrap();
        let (back, _chests) = parse(&buf).unwrap();
        assert_eq!(back.width, 64);
        assert_eq!(back.name, "Test");
        assert_eq!(back.seed, 99);
        let t = back.tiles[back.index(5, 30)];
        assert_eq!(
            (t.slope(), t.paint(), t.wall_col(), t.wall_row()),
            (2, 3, 4, 2)
        );
        assert_eq!(back.world_surface, 12.5);
        assert_eq!((back.moon_type, back.moon_phase), (5, 3));
        assert!(back.frame_important[5]);
        assert_eq!(back.tiles, w.tiles);
        // v3: the sparse paint/coating extras round-trip (the zero byte
        // collapses back to absent).
        assert_eq!(back.extras.len(), 1);
        assert_eq!(back.extras_at(back.index(3, 0)), 0x24);
        assert_eq!(back.wall_paint(3, 0), 4, "wall paint rides the low 5 bits");
        assert!(back.invisible_block(3, 0), "the Echo bit rides bit 5");
        assert!(!back.invisible_wall(3, 0));
        assert!(!back.invisible_block(7, 7), "the zero byte collapses");
    }

    #[test]
    fn dw_1_4_dial_cooldowns_round_trip_v4() {
        // DW-1.4: the v4 tail carries both cooldown bytes (`Main.cs:2161`/`:2165`).
        let mut w = World::new("Test".into(), 32, 32);
        w.sundial_cooldown = 5;
        w.moondial_cooldown = 8;
        let mut buf = Vec::new();
        write_world(&mut buf, &w, &WorldSide::default()).unwrap();
        assert_eq!(&buf[8..12], &[11, 0, 0, 0], "the writer emits the current v11");
        let (back, _chests) = parse(&buf).unwrap();
        assert_eq!(back.sundial_cooldown, 5);
        assert_eq!(back.moondial_cooldown, 8);
    }

    #[test]
    fn dw_1_4_v3_loads_zero_cooldowns() {
        // DW-1.4: a pristine v3 file (the current stream stripped of the
        // v4 tail (2 bytes) and the v5 tail (1 presence byte — the world has
        // no bed spawn), the version field restored) loads with both
        // cooldowns at 0.
        let mut w = World::new("Test".into(), 32, 32);
        w.sundial_cooldown = 7;
        let mut buf = Vec::new();
        write_world(&mut buf, &w, &WorldSide::default()).unwrap();
        buf.truncate(buf.len() - 3);
        buf[8..12].copy_from_slice(&3u32.to_le_bytes());
        let (back, _chests) = parse(&buf).unwrap();
        assert_eq!(back.sundial_cooldown, 0);
        assert_eq!(back.moondial_cooldown, 0);
    }

    #[test]
    fn dw_1_4_version_above_4_rejected() {
        // DW-1.4 (dirty): the existing loader rejects a version above the
        // current one (`save.rs` version gate). v9 (the flags/wires/TE/chests/banners/
        // bestiary/npc/ore-tiers tail) is now the current format, so the gate pins 12.
        let w = World::new("Test".into(), 32, 32);
        let mut buf = Vec::new();
        write_world(&mut buf, &w, &WorldSide::default()).unwrap();
        buf[8..12].copy_from_slice(&12u32.to_le_bytes());
        assert!(parse(&buf).is_err(), "version 12 is unsupported");
    }

    #[test]
    fn the_bed_spawn_round_trips_in_v5() {
        // DW-3.1 — the v5 tail field: `World::bed_spawn` survives the save
        // (both arms), and a `None` world stays `None`.
        let mut w = World::new("Test".into(), 64, 48);
        w.bed_spawn = Some((30, 20));
        let mut buf = Vec::new();
        write_world(&mut buf, &w, &WorldSide::default()).unwrap();
        let (back, _chests) = parse(&buf).unwrap();
        assert_eq!(back.bed_spawn, Some((30, 20)));
        w.bed_spawn = None;
        buf.clear();
        write_world(&mut buf, &w, &WorldSide::default()).unwrap();
        assert_eq!(parse(&buf).unwrap().0.bed_spawn, None);
    }

    #[test]
    fn a_v4_payload_still_loads_without_the_bed_spawn() {
        // DW-3.1 — the back-compat read: the v5 payload with the version
        // field patched to 4 parses as v4 — the tail sits unread and the bed
        // spawn degrades to `None`.
        let mut w = World::new("Test".into(), 64, 48);
        w.spawn_x = 32;
        w.spawn_y = 20;
        w.bed_spawn = Some((30, 22));
        let mut buf = Vec::new();
        write_world(&mut buf, &w, &WorldSide::default()).unwrap();
        buf[8..12].copy_from_slice(&4u32.to_le_bytes());
        let (back, _chests) = parse(&buf).unwrap();
        assert_eq!(back.bed_spawn, None);
        assert_eq!((back.spawn_x, back.spawn_y), (32, 20));
    }

    #[test]
    fn a_corrupt_bed_spawn_degrades_to_none() {
        // DW-3.1 (dirty, the deserialization trust boundary): an
        // out-of-range bed cell degrades to `None` instead of erroring or
        // poisoning the spawn (`Player.cs:37571-37575`'s fallback arm).
        let mut w = World::new("Test".into(), 64, 48);
        w.bed_spawn = Some((30, 20));
        let mut buf = Vec::new();
        write_world(&mut buf, &w, &WorldSide::default()).unwrap();
        // The v5 tail is the last 9 bytes AHEAD of the v6 sections: the
        // fixed flags block + four empty colour counts + one empty entity
        // count sit after it — [presence][x: i32][y: i32] — poison the x.
        let tail = buf.len() - (2 + V6_FLAGS_LEN + 4 * 4 + 4 + 4 + 4 + 4 + 1) - 9;
        buf[tail + 1..tail + 5].copy_from_slice(&99999i32.to_le_bytes());
        assert_eq!(parse(&buf).unwrap().0.bed_spawn, None);
    }

    #[test]
    fn dw_2_1_a_v6_payload_loads_the_seeds_but_not_the_v7_tail() {
        // DW-2.1: a v6 payload (the seed flags only — the v7 block truncated
        // and the version field patched) parses with the seeds and with the
        // `World::new` defaults for the v7 sections — the tail sits unread.
        let mut w = World::new("Test".into(), 32, 32);
        w.seeds = crate::seeds::SeedFlags::from_seed_text("for the worthy");
        let mut buf = Vec::new();
        write_world(&mut buf, &w, &WorldSide::default()).unwrap();
        // The v7 block of an empty-wire/entity/chest/banner/bestiary world: the
        // fixed flags block + four empty colour counts + one empty entity count +
        // one empty chest count + one empty banner count + one empty bestiary count.
        buf.truncate(buf.len() - (V6_FLAGS_LEN + 4 * 4 + 4 + 4 + 4 + 4 + 4 + 1));
        buf[8..12].copy_from_slice(&6u32.to_le_bytes());
        let (back, side) = parse(&buf).unwrap();
        let chests = side.chests;
        assert_eq!(back.seeds, w.seeds);
        assert!(!back.downed_eoc, "the v7 flags stay default");
        assert!(!back.wires.get(1, 1, WireColor::Red));
        assert!(back.tile_entities.get_at(2, 2).is_none());
        assert!(chests.is_empty());
    }

    #[test]
    fn m5_seed_1_the_secret_seed_flags_round_trip_in_v6() {
        // M5-SEED-1: the v6 block carries the classic secret-seed flags
        // (`WorldFile.SaveWorldFlags`'s nine booleans as the port's
        // `serializedSeedsSum` bitmask); a classic world keeps all bits 0 and
        // a seed world survives the save/load.
        let mut classic = World::new("Test".into(), 32, 32);
        let mut buf = Vec::new();
        write_world(&mut buf, &classic, &WorldSide::default()).unwrap();
        assert_eq!(back_flags(&buf), 0, "a classic world writes no seed bits");
        let (back, _) = parse(&buf).unwrap();
        assert_eq!(back.seeds, crate::seeds::SeedFlags::none());
        classic.seeds = crate::seeds::SeedFlags::from_seed_text("for the worthy");
        buf.clear();
        write_world(&mut buf, &classic, &WorldSide::default()).unwrap();
        assert_eq!(back_flags(&buf), crate::seeds::SeedFlags::FOR_THE_WORTHY);
        let (back, _) = parse(&buf).unwrap();
        assert_eq!(back.seeds, classic.seeds);
    }

    /// A `WorldSide` carrying only the chest records (the v7 tests).
    fn chest_side(chests: &[ChestRecord]) -> WorldSide {
        WorldSide { chests: chests.to_vec(), banners: Vec::new(), bestiary: Vec::new(), npcs: Vec::new(), ore_tiers: None}
    }

    /// The v6 block's u16 bitmask, read back out of the written buffer — the
    /// two bytes ahead of the fixed v7 flags block.
    fn back_flags(buf: &[u8]) -> u16 {
        let tail = buf.len() - (V6_FLAGS_LEN + 4 * 4 + 4 + 4 + 4 + 4 + 4 + 1) - 2;
        u16::from_le_bytes([buf[tail], buf[tail + 1]])
    }

    #[test]
    fn dw_2_1_the_v6_flags_wires_and_tes_round_trip() {
        // DW-2.1: the v6 tail carries the `SaveWorldFlags` progression subset
        // (`WorldFile.cs:1238-1430`), the wire side table and the tile-entity
        // books (`SaveTileEntities`, `WorldFile.cs:3302-3313`) across a
        // save/load.
        let mut w = World::new("Test".into(), 64, 48);
        // One flag from every group.
        w.blood_moon = true;
        w.downed_eoc = true;
        w.downed_golem = true;
        w.saved_goblin = true;
        w.downed_pirates = true;
        w.shadow_orb_smashed = true;
        w.spawn_meteor = true;
        w.hard_mode = true;
        w.dd2_event_ongoing = true;
        w.downed_moon_lord = true;
        w.slime_rain = true;
        w.tower_active_nebula = true;
        w.downed_tower_stardust = true;
        // The counter block.
        w.shadow_orb_count = 7;
        w.altars_smashed = 12;
        w.invasion_delay = 3;
        w.invasion_size = 80;
        w.invasion_type = 2;
        w.invasion_size_start = 120;
        w.slime_rain_kills = 41;
        w.dd2_wave = 3;
        w.dd2_spawn_hold = 90;
        w.tower_shield_nebula = 4_000;
        w.lunar_shield_power_normal = 100;
        w.invasion_x = 1234.5;
        w.slime_rain_time = 9876.25;
        w.dd2_wave_kills = 12.5;
        // The wires — all four colours, the origin and negative coords.
        w.wires.set(3, 4, WireColor::Red, true);
        w.wires.set(0, 0, WireColor::Blue, true);
        w.wires.set(-5, 7, WireColor::Green, true);
        w.wires.set(9, 9, WireColor::Yellow, true);
        w.wires.set(9, 9, WireColor::Yellow, false); // the clear round-trips too
        // The tile entities — a dummy with its npc, an item frame, a logic
        // sensor and a pylon. The load-side re-validation (`LoadTileEntities`
        // `WorldFile.cs:3315+`) reads the anchor tiles, so the entities sit on
        // their real kinds/frames.
        for &(x, y, kind, frame_y) in &[
            (5, 5, 378, 0i16),  // TrainingDummy
            (6, 6, 395, 0),     // ItemFrame
            (7, 7, 423, 36),    // LogicSensor (check 2 rides frameY/18)
            (8, 8, 597, 0),     // TeleportationPylon
        ] {
            let i = w.index(x, y);
            w.tiles[i].kind = kind;
            w.tiles[i].frame_x = 0;
            w.tiles[i].frame_y = frame_y;
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        let dummy = w.tile_entities.place_unchecked(TeKind::TrainingDummy, 5, 5);
        w.tile_entities.by_id_mut(dummy).unwrap().data = TeData::Dummy(7);
        let frame = w.tile_entities.place_unchecked(TeKind::ItemFrame, 6, 6);
        w.tile_entities.by_id_mut(frame).unwrap().data =
            TeData::Item(Some(TeItem { kind: 75, prefix: 3, stack: 12 }));
        let sensor = w.tile_entities.place_unchecked(TeKind::LogicSensor, 7, 7);
        w.tile_entities.by_id_mut(sensor).unwrap().data =
            TeData::Logic { check: 2, on: true, counted: 15 };
        let pylon = w.tile_entities.place_unchecked(TeKind::TeleportationPylon, 8, 8);

        let mut buf = Vec::new();
        write_world(&mut buf, &w, &WorldSide::default()).unwrap();
        let (back, _chests) = parse(&buf).unwrap();
        // The set flags round-trip and the unset ones stay default.
        assert!(back.blood_moon && back.downed_eoc && back.downed_golem && back.saved_goblin);
        assert!(back.downed_pirates && back.shadow_orb_smashed && back.spawn_meteor && back.hard_mode);
        assert!(back.dd2_event_ongoing && back.downed_moon_lord && back.slime_rain);
        assert!(back.tower_active_nebula && back.downed_tower_stardust);
        assert!(!back.downed_eow && !back.downed_frost && !back.eclipse && !back.downed_goblins);
        // The counters and the float fields.
        assert_eq!(back.shadow_orb_count, 7);
        assert_eq!(back.altars_smashed, 12);
        assert_eq!(back.invasion_delay, 3);
        assert_eq!(back.invasion_size, 80);
        assert_eq!(back.invasion_type, 2);
        assert_eq!(back.invasion_size_start, 120);
        assert_eq!(back.slime_rain_kills, 41);
        assert_eq!(back.dd2_wave, 3);
        assert_eq!(back.dd2_spawn_hold, 90);
        assert_eq!(back.tower_shield_nebula, 4_000);
        assert_eq!(back.lunar_shield_power_normal, 100);
        assert_eq!(back.invasion_x, 1234.5);
        assert_eq!(back.slime_rain_time, 9876.25);
        assert_eq!(back.dd2_wave_kills, 12.5);
        // The wires.
        assert!(back.wires.get(3, 4, WireColor::Red));
        assert!(back.wires.get(0, 0, WireColor::Blue));
        assert!(back.wires.get(-5, 7, WireColor::Green));
        assert!(!back.wires.get(9, 9, WireColor::Yellow), "the clear round-trips");
        assert!(!back.wires.get(3, 4, WireColor::Blue), "no cross-colour bleed");
        // The tile entities — the stored ids and payloads.
        assert_eq!(back.tile_entities.get_by_id(dummy).unwrap().data, TeData::Dummy(7));
        assert_eq!(
            back.tile_entities.get_by_id(frame).unwrap().data,
            TeData::Item(Some(TeItem { kind: 75, prefix: 3, stack: 12 }))
        );
        assert_eq!(
            back.tile_entities.get_by_id(sensor).unwrap().data,
            TeData::Logic { check: 2, on: true, counted: 15 }
        );
        assert_eq!(back.tile_entities.get_at(8, 8).map(|e| e.id), Some(pylon));
        assert_eq!(back.tile_entities.get_at(5, 5).map(|e| e.id), Some(dummy));
    }

    #[test]
    fn dw_2_1_a_v5_payload_loads_defaults() {
        // DW-2.1: the v6 payload truncated to the v5 length (the version
        // field patched) parses with the `World::new` defaults — the flags
        // false, the counters 0, no wires, no entities.
        let mut w = World::new("Test".into(), 32, 32);
        w.hard_mode = true;
        w.wires.set(1, 1, WireColor::Red, true);
        w.tile_entities.place_unchecked(TeKind::ItemFrame, 2, 2);
        let mut buf = Vec::new();
        write_world(&mut buf, &w, &WorldSide::default()).unwrap();
        // The v6 tail of an empty-wire/entity world: the fixed flags block +
        // four empty colour counts + one empty entity count.
        buf.truncate(buf.len() - (2 + V6_FLAGS_LEN + 4 * 4 + 4));
        buf[8..12].copy_from_slice(&5u32.to_le_bytes());
        let (back, _chests) = parse(&buf).unwrap();
        assert!(!back.hard_mode);
        assert!(!back.downed_eoc);
        assert_eq!(back.invasion_size, 0);
        assert!(!back.wires.get(1, 1, WireColor::Red));
        assert!(back.tile_entities.get_at(2, 2).is_none());
    }

    #[test]
    fn dw_2_1_a_truncated_v6_tail_bails() {
        // DW-2.1 (dirty): the wire/TE sections are self-delimiting; a payload
        // cut mid-record is a structural error, not a silent default.
        let mut w = World::new("Test".into(), 32, 32);
        w.hard_mode = true;
        w.wires.set(1, 1, WireColor::Red, true);
        w.tile_entities.place_unchecked(TeKind::ItemFrame, 2, 2);
        let mut buf = Vec::new();
        write_world(&mut buf, &w, &WorldSide::default()).unwrap();
        buf.truncate(buf.len() - 2); // inside the TE record
        assert!(parse(&buf).is_err());
    }

    #[test]
    fn dw_2_1_the_wire_and_te_sections_are_deterministic() {
        // DW-2.1: the same world written twice is byte-equal — the HashSet
        // iteration order must not leak into the stream.
        let mut w = World::new("Test".into(), 32, 32);
        for (x, y) in [(4, 1), (1, 4), (2, 2), (9, 0)] {
            w.wires.set(x, y, WireColor::Green, true);
        }
        for (x, y) in [(3, 3), (5, 1)] {
            w.wires.set(x, y, WireColor::Yellow, true);
        }
        w.tile_entities.place_unchecked(TeKind::ItemFrame, 2, 2);
        w.tile_entities.place_unchecked(TeKind::TeleportationPylon, 7, 3);
        let mut a = Vec::new();
        let mut b = Vec::new();
        write_world(&mut a, &w, &WorldSide::default()).unwrap();
        write_world(&mut b, &w, &WorldSide::default()).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn dw_2_2_the_chests_round_trip_in_v7() {
        // DW-2.2: the v7 tail carries `SaveChests`'s shape (`WorldFile.cs:1606-1676`)
        // — the anchor, the name (the effective one, renamed or default) and the
        // slots, including interior airs (the implicit-air compression) and the
        // prefixed items.
        let mut w = World::new("Test".into(), 64, 48);
        let chests = vec![
            ChestRecord {
                x: 12,
                y: 7,
                name: "Gold Chest".into(),
                slots: vec![
                    ChestItem { stack: 35, kind: 19, prefix: 0 },
                    ChestItem { stack: 0, kind: 0, prefix: 0 }, // interior air
                    ChestItem { stack: 1, kind: 1541, prefix: 81 },
                ],
            },
            ChestRecord { x: 40, y: 9, name: String::new(), slots: Vec::new() },
        ];
        let mut buf = Vec::new();
        write_world(&mut buf, &w, &chest_side(&chests)).unwrap();
        assert_eq!(&buf[8..12], &[11, 0, 0, 0], "the writer emits the current v11");
        let (back, side) = parse(&buf).unwrap();
        let back_chests = side.chests;
        assert_eq!(back_chests, chests);
    }

    #[test]
    fn dw_2_2_a_v6_payload_loads_no_chests() {
        // DW-2.2: a v6 payload (the version field patched) parses with no chests —
        // the v7 tail sits unread, the same back-compat contract as the earlier
        // tails.
        let mut w = World::new("Test".into(), 32, 32);
        let chests = vec![ChestRecord { x: 3, y: 4, name: "Wooden Chest".into(), slots: vec![ChestItem { stack: 99, kind: 9, prefix: 0 }] }];
        let mut buf = Vec::new();
        write_world(&mut buf, &w, &chest_side(&chests)).unwrap();
        buf[8..12].copy_from_slice(&6u32.to_le_bytes());
        let (back, side) = parse(&buf).unwrap();
        let back_chests = side.chests;
        assert!(back_chests.is_empty());
        assert!(!back_chests.iter().any(|c| c.x == 3 && c.y == 4));
    }

    #[test]
    fn dw_2_2_a_truncated_chest_section_bails() {
        // DW-2.2 (dirty): the chests section is self-delimiting; a payload cut
        // mid-record is a structural error.
        let mut w = World::new("Test".into(), 32, 32);
        let chests = vec![ChestRecord {
            x: 3,
            y: 4,
            name: "Wooden Chest".into(),
            slots: vec![ChestItem { stack: 99, kind: 9, prefix: 0 }],
        }];
        let mut buf = Vec::new();
        write_world(&mut buf, &w, &chest_side(&chests)).unwrap();
        buf.truncate(buf.len() - 3); // inside the item record
        assert!(parse(&buf).is_err());
    }

    #[test]
    fn dw_2_2_the_chest_section_is_deterministic() {
        // DW-2.2: the same records written twice are byte-equal.
        let mut w = World::new("Test".into(), 32, 32);
        let chests = vec![
            ChestRecord { x: 5, y: 6, name: "Ivy Chest".into(), slots: vec![ChestItem { stack: 20, kind: 166, prefix: 3 }] },
            ChestRecord { x: 7, y: 8, name: "Water Chest".into(), slots: Vec::new() },
        ];
        let mut a = Vec::new();
        let mut b = Vec::new();
        write_world(&mut a, &w, &chest_side(&chests)).unwrap();
        write_world(&mut b, &w, &chest_side(&chests)).unwrap();
        assert_eq!(a, b);
        let (_, back) = parse(&a).unwrap();
        assert_eq!(back.chests[0].name, "Ivy Chest");
        assert_eq!(back.chests[1].name, "Water Chest");
    }

    #[test]
    fn dw_2_3_the_banner_counts_round_trip_in_v8() {
        // DW-2.3: the v8 tail carries `BannerSystem.Save`'s kill counts
        // (`int32` per banner id, `BannerSystem.cs:172-186`) as the port's
        // sparse records — including a zeroed banner and the 999999999-range
        // counts the bestiary tracker caps at.
        let mut w = World::new("Test".into(), 32, 32);
        let side = WorldSide {
            chests: Vec::new(),
            banners: vec![
                BannerKill { banner: 3331, kills: 50 },
                BannerKill { banner: 71, kills: 0 },
                BannerKill { banner: 1569, kills: 999_999_999 },
            ],
            bestiary: Vec::new(),
        npcs: Vec::new(),
        ore_tiers: None,
        };
        let mut buf = Vec::new();
        write_world(&mut buf, &w, &side).unwrap();
        let (back, back_side) = parse(&buf).unwrap();
        assert_eq!(back_side.banners, side.banners);
        assert!(back_side.chests.is_empty());
    }

    #[test]
    fn dw_2_3_a_v7_payload_loads_no_banners() {
        // DW-2.3: a v7 payload (the version field patched) parses with the
        // chests intact and no banners — the v8 tail sits unread, the same
        // back-compat contract as the earlier tails.
        let mut w = World::new("Test".into(), 32, 32);
        let side = WorldSide {
            chests: vec![ChestRecord {
                x: 4,
                y: 5,
                name: "Gold Chest".into(),
                slots: vec![ChestItem { stack: 3, kind: 19, prefix: 0 }],
            }],
            banners: vec![BannerKill { banner: 3331, kills: 7 }],
            bestiary: Vec::new(),
        npcs: Vec::new(),
        ore_tiers: None,
        };
        let mut buf = Vec::new();
        write_world(&mut buf, &w, &side).unwrap();
        buf[8..12].copy_from_slice(&7u32.to_le_bytes());
        let (back, back_side) = parse(&buf).unwrap();
        assert_eq!(back_side.chests, side.chests, "the chests still load");
        assert!(back_side.banners.is_empty(), "no banners below v8");
    }

    #[test]
    fn dw_2_3_a_truncated_banner_section_bails() {
        // DW-2.3 (dirty): the banners section is self-delimiting; a payload
        // cut mid-record is a structural error.
        let mut w = World::new("Test".into(), 32, 32);
        let side = WorldSide {
            chests: Vec::new(),
            banners: vec![BannerKill { banner: 3331, kills: 7 }],
            bestiary: Vec::new(),
        npcs: Vec::new(),
        ore_tiers: None,
        };
        let mut buf = Vec::new();
        write_world(&mut buf, &w, &side).unwrap();
        buf.truncate(buf.len() - 2); // inside the banner record
        assert!(parse(&buf).is_err());
    }

    #[test]
    fn dw_2_3_the_banner_section_is_deterministic() {
        // DW-2.3: the same records written twice are byte-equal.
        let mut w = World::new("Test".into(), 32, 32);
        let side = WorldSide {
            chests: Vec::new(),
            banners: vec![
                BannerKill { banner: 3331, kills: 7 },
                BannerKill { banner: 71, kills: 0 },
                BannerKill { banner: 1569, kills: 12 },
            ],
            bestiary: Vec::new(),
        npcs: Vec::new(),
        ore_tiers: None,
        };
        let mut a = Vec::new();
        let mut b = Vec::new();
        write_world(&mut a, &w, &side).unwrap();
        write_world(&mut b, &w, &side).unwrap();
        assert_eq!(a, b);
        let (_, back) = parse(&a).unwrap();
        assert_eq!(back.banners.len(), 3);
        assert_eq!(back.banners[0].banner, 3331);
    }

    #[test]
    fn dw_2_4_the_bestiary_counts_round_trip_in_v9() {
        // DW-2.4: the v9 tail carries `NPCKillsTracker.Save`'s counts
        // (the credit-id string + int32, `NPCKillsTracker.cs:59-70`) — the
        // vanilla's world-file section; the port had them on the `.trplayer`
        // sidecar, and that line stays the back-compat fallback.
        let mut w = World::new("Test".into(), 32, 32);
        let side = WorldSide {
            chests: Vec::new(),
            banners: Vec::new(),
            bestiary: vec![
                BestiaryKill { credit_id: "BlueSlime".into(), kills: 137 },
                BestiaryKill { credit_id: "KingSlime".into(), kills: 0 },
                BestiaryKill { credit_id: " EyeofCthulhu".into(), kills: 1 },
            ],
        npcs: Vec::new(),
        ore_tiers: None,
        };
        let mut buf = Vec::new();
        write_world(&mut buf, &w, &side).unwrap();
        let (back, back_side) = parse(&buf).unwrap();
        assert_eq!(back_side.bestiary, side.bestiary);
    }

    #[test]
    fn dw_2_4_a_v8_payload_loads_no_bestiary() {
        // DW-2.4: a v8 payload (the version field patched) parses with the
        // chests and banners intact and no bestiary — the sidecar fallback is
        // the load path's decision (app-side).
        let mut w = World::new("Test".into(), 32, 32);
        let side = WorldSide {
            chests: Vec::new(),
            banners: vec![BannerKill { banner: 69, kills: 50 }],
            bestiary: vec![BestiaryKill { credit_id: "BlueSlime".into(), kills: 3 }],
        npcs: Vec::new(),
        ore_tiers: None,
        };
        let mut buf = Vec::new();
        write_world(&mut buf, &w, &side).unwrap();
        buf[8..12].copy_from_slice(&8u32.to_le_bytes());
        let (back, back_side) = parse(&buf).unwrap();
        assert_eq!(back_side.banners, side.banners, "the banners still load");
        assert!(back_side.bestiary.is_empty(), "no bestiary below v9");
    }

    #[test]
    fn dw_2_4_a_truncated_bestiary_section_bails() {
        // DW-2.4 (dirty): the bestiary section is self-delimiting; a payload
        // cut mid-record is a structural error.
        let mut w = World::new("Test".into(), 32, 32);
        let side = WorldSide {
            chests: Vec::new(),
            banners: Vec::new(),
            bestiary: vec![BestiaryKill { credit_id: "BlueSlime".into(), kills: 3 }],
        npcs: Vec::new(),
        ore_tiers: None,
        };
        let mut buf = Vec::new();
        write_world(&mut buf, &w, &side).unwrap();
        buf.truncate(buf.len() - 2); // inside the record
        assert!(parse(&buf).is_err());
    }

    #[test]
    fn dw_2_4_the_bestiary_section_is_deterministic() {
        // DW-2.4: the same records written twice are byte-equal.
        let mut w = World::new("Test".into(), 32, 32);
        let side = WorldSide {
            chests: Vec::new(),
            banners: Vec::new(),
            bestiary: vec![
                BestiaryKill { credit_id: "BlueSlime".into(), kills: 7 },
                BestiaryKill { credit_id: "KingSlime".into(), kills: 0 },
            ],
        npcs: Vec::new(),
        ore_tiers: None,
        };
        let mut a = Vec::new();
        let mut b = Vec::new();
        write_world(&mut a, &w, &side).unwrap();
        write_world(&mut b, &w, &side).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn dw_2_5_the_town_npcs_round_trip_in_v10() {
        // DW-2.5: the v10 tail carries `SaveNPCs`'s block-B record shape
        // (`WorldFile.cs:1703-1750`): the kind, the given name, the pixel
        // position, the homeless flag and the home tile — the vanilla's
        // per-world town roster (the port's internal kind ids).
        let mut w = World::new("Test".into(), 32, 32);
        let side = WorldSide {
            chests: Vec::new(),
            banners: Vec::new(),
            bestiary: Vec::new(),
            npcs: vec![
                NpcRecord {
                    kind: 5, // the Guide
                    given_name: "Andrew".into(),
                    x: 96.5,
                    y: 128.25,
                    homeless: false,
                    home_x: 6,
                    home_y: 7,
                },
                NpcRecord {
                    kind: 17, // the Merchant
                    given_name: String::new(),
                    x: 320.0,
                    y: 64.0,
                    homeless: true,
                    home_x: -1,
                    home_y: -1,
                },
            ],
        ore_tiers: None,
        };
        let mut buf = Vec::new();
        write_world(&mut buf, &w, &side).unwrap();
        let (back, back_side) = parse(&buf).unwrap();
        assert_eq!(back_side.npcs, side.npcs);
    }

    #[test]
    fn dw_2_5_a_v9_payload_loads_no_npcs() {
        // DW-2.5: a v9 payload (the version field patched) parses with the
        // bestiary intact and no town NPCs — the app keeps its
        // spawn-the-Guide path for such a world.
        let mut w = World::new("Test".into(), 32, 32);
        let side = WorldSide {
            chests: Vec::new(),
            banners: Vec::new(),
            bestiary: vec![BestiaryKill { credit_id: "BlueSlime".into(), kills: 2 }],
            npcs: vec![NpcRecord {
                kind: 5,
                given_name: "Andrew".into(),
                x: 96.0,
                y: 64.0,
                homeless: false,
                home_x: 1,
                home_y: 1,
            }],
        ore_tiers: None,
        };
        let mut buf = Vec::new();
        write_world(&mut buf, &w, &side).unwrap();
        buf[8..12].copy_from_slice(&9u32.to_le_bytes());
        let (back, back_side) = parse(&buf).unwrap();
        assert_eq!(back_side.bestiary, side.bestiary, "the bestiary still loads");
        assert!(back_side.npcs.is_empty(), "no NPCs below v10");
    }

    #[test]
    fn dw_2_5_a_truncated_npc_section_bails() {
        // DW-2.5 (dirty): the NPC section is self-delimiting; a payload cut
        // mid-record is a structural error.
        let mut w = World::new("Test".into(), 32, 32);
        let side = WorldSide {
            chests: Vec::new(),
            banners: Vec::new(),
            bestiary: Vec::new(),
            npcs: vec![NpcRecord {
                kind: 5,
                given_name: "Andrew".into(),
                x: 96.0,
                y: 64.0,
                homeless: false,
                home_x: 1,
                home_y: 1,
            }],
        ore_tiers: None,
        };
        let mut buf = Vec::new();
        write_world(&mut buf, &w, &side).unwrap();
        buf.truncate(buf.len() - 2); // inside the record
        assert!(parse(&buf).is_err());
    }

    #[test]
    fn dw_2_5_the_npc_section_is_deterministic() {
        // DW-2.5: the same records written twice are byte-equal.
        let mut w = World::new("Test".into(), 32, 32);
        let side = WorldSide {
            chests: Vec::new(),
            banners: Vec::new(),
            bestiary: Vec::new(),
            npcs: vec![
                NpcRecord { kind: 5, given_name: "A".into(), x: 1.0, y: 2.0, homeless: true, home_x: -1, home_y: -1 },
                NpcRecord { kind: 17, given_name: "B".into(), x: 3.0, y: 4.0, homeless: false, home_x: 2, home_y: 3 },
            ],
        ore_tiers: None,
        };
        let mut a = Vec::new();
        let mut b = Vec::new();
        write_world(&mut a, &w, &side).unwrap();
        write_world(&mut b, &w, &side).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn dw_2_6_the_ore_tiers_round_trip_in_v11() {
        // DW-2.6: the v11 tail carries `WorldGen.SavedOreTiers`
        // (`WorldGen.cs:3283-3297`) — the seven i32 tiers in the field order,
        // including the -1 hardmode sentinels and the alternate-metal flips
        // (the tin/lead/tungsten/platinum kinds).
        let mut w = World::new("Test".into(), 32, 32);
        let side = WorldSide {
            chests: Vec::new(),
            banners: Vec::new(),
            bestiary: Vec::new(),
            npcs: Vec::new(),
            ore_tiers: Some([166, 12, 169, 172, -1, -1, -1]),
        };
        let mut buf = Vec::new();
        write_world(&mut buf, &w, &side).unwrap();
        let (back, back_side) = parse(&buf).unwrap();
        assert_eq!(back_side.ore_tiers, side.ore_tiers);
        // The World that comes back carries them too (the app reads
        // `World.ore_tiers`).
        assert_eq!(back.ore_tiers.copper, 166);
        assert_eq!(back.ore_tiers.gold, 172);
        assert_eq!(back.ore_tiers.cobalt, -1);
    }

    #[test]
    fn dw_2_6_a_v10_payload_loads_default_tiers() {
        // DW-2.6: a v10 payload (the version field patched) carries no ore
        // section — the World keeps the static defaults (the C#'s reset arm,
        // `WorldFile.cs:796-814`), which is exactly the pre-fix state.
        let mut w = World::new("Test".into(), 32, 32);
        let side = WorldSide {
            chests: Vec::new(),
            banners: Vec::new(),
            bestiary: Vec::new(),
            npcs: Vec::new(),
            ore_tiers: Some([166, 6, 9, 8, 107, -1, -1]),
        };
        let mut buf = Vec::new();
        write_world(&mut buf, &w, &side).unwrap();
        buf[8..12].copy_from_slice(&10u32.to_le_bytes());
        let (back, back_side) = parse(&buf).unwrap();
        assert!(back_side.ore_tiers.is_none(), "no section below v11");
        assert_eq!(back.ore_tiers, crate::ores::SavedOreTiers::new());
    }

    #[test]
    fn dw_2_6_a_truncated_ore_section_bails() {
        // DW-2.6 (dirty): the ore section is self-delimiting (the presence
        // byte + seven i32s); a payload cut mid-section bails.
        let mut w = World::new("Test".into(), 32, 32);
        let side = WorldSide {
            chests: Vec::new(),
            banners: Vec::new(),
            bestiary: Vec::new(),
            npcs: Vec::new(),
            ore_tiers: Some([7, 6, 9, 8, -1, -1, -1]),
        };
        let mut buf = Vec::new();
        write_world(&mut buf, &w, &side).unwrap();
        buf.truncate(buf.len() - 3); // inside the tiers
        assert!(parse(&buf).is_err());
    }

    #[test]
    fn dw_2_6_the_ore_section_is_deterministic() {
        // DW-2.6: the same tiers written twice are byte-equal.
        let mut w = World::new("Test".into(), 32, 32);
        let side = WorldSide {
            chests: Vec::new(),
            banners: Vec::new(),
            bestiary: Vec::new(),
            npcs: Vec::new(),
            ore_tiers: Some([13, 6, 9, 8, -1, -1, -1]),
        };
        let mut a = Vec::new();
        let mut b = Vec::new();
        write_world(&mut a, &w, &side).unwrap();
        write_world(&mut b, &w, &side).unwrap();
        assert_eq!(a, b);
    }
}

