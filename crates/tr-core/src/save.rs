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
/// v12 appended the two secret-seed bitmasks (the classic `World.seeds` u16
/// — the `SaveWorldFlags` nine booleans, `WorldFile.cs:1238-1250` — and the
/// M5-SEED-2 `World.micro_seeds` u64); the C# `.wld` stores the nine
/// booleans in the flags block and re-derives the micro seeds from the seed
/// text, so the port carries both as its own tail block.
/// v13 appended the signs and the weighted pressure plates (`LoadSigns` /
/// `LoadWeightedPressurePlates`, `WorldFile.cs:1666-1685` / `:3373-3384`).
/// Both are `World` fields the port already loaded from the C# `.wld`
/// (M6-SAVE-1), so a `.trwld` round trip used to drop them.
/// v15 appended `NPC.downedQueenSlime` (`WorldFile.cs:1396`'s flags byte —
/// the Queen Slime's stock gate, `Chest.cs:3428`, reads it; S1-F-16); a
/// pre-v15 file loads with the flag down.
/// v14 appended the rain event's triple (`Main.raining`/`rainTime`/
/// `maxRaining` — the C#'s flag block writes the save-time snapshot
/// `_tempRaining`/`_tempRainTime`/`_tempMaxRain`, `WorldFile.cs:1312-1314`,
/// and `SetOngoingToTemps` `:159-162` restores it on load, `cloudAlpha` :=
/// `maxRaining`); a pre-v14 file loads with the event off, which is also
/// what the vanilla does for a world saved before the rain block existed.
/// v16 appended the five surface-background style bytes (`WorldGen.corruptBG`
/// / `jungleBG` / `hallowBG` / `crimsonBG` at `WorldFile.cs:1319-1323` and
/// `mushroomBG` at `:1375`) that the `DrawSurfaceBG_*` variants resolve their
/// `Background_<id>` sheets from; a pre-v16 file loads the `setBG` defaults.
/// v18 appended the three town-pet license world flags (`NPC.boughtCat` /
/// `boughtDog` / `boughtBunny`, `WorldFile.cs:1392-1394`) the 4829/4830/4910
/// Pet License use-unlocks raise (W4-g87); a pre-v18 file loads them false.
/// v18 appended the world-identity block (W4-c43): the generator version
/// (`WorldFile.cs:1225`), the 16-byte GUID (`:1226`), the world id (`:1227`,
/// footer `:1756`), `CreationTime` and `LastPlayed` as `DateTime.ToBinary`
/// longs (`:1250-1251`) — the fields the `.wld` header already round-trips
/// (S1-J-2), which the native format used to drop. A pre-v18 file loads the
/// `World::new` zeros; the save funnel re-stamps the identity on its first
/// re-save (`wld::stamp_created_identity` + the WorldId roll).
/// v18 appended the packed framing variants (`World::variants`, the runtime
/// `Tile.bTileHeader2` bits 4-7, `Tile.cs:464-482`) as a dense `width*height`
/// byte block (W4-d109, the save half of S1-A-3). The C# `.wld` never
/// serializes these bits (`SaveWorldTiles` `WorldFile.cs:1432-1604`), so this
/// is a port-only persistence: a v18 load restores them and sets
/// `World::variants_from_save`, and the full-world framing passes reuse the
/// stored values (`TileFrame`/`WallFrame` resetFrame:false) instead of
/// re-rolling. A pre-v18 file loads with the `World::new` zeros and re-rolls
/// (the C#'s own reset semantics).
/// v17 appended `NPC.downedFishron` (the case-370 death flag, S1-D-3 W-9).
/// v18 appended the banner claimable counts (`BannerSystem.Save`'s
/// `claimableBanners` half, `BannerSystem.cs:180-185`, the `version >= 289`
/// load arm `:203-211`) — the claimable-banner pool the milestone fills
/// (W4-e24); a pre-v18 file loads none (the pool starts empty).
/// v19 appended the combat-book-two + Peddler's-Satchel world flags
/// (`NPC.combatBookVolumeTwoWasUsed` / `peddlersSatchelWasUsed`, the C#
/// .wld flags bytes `WorldFile.cs:1407-1408`; W4-g90 lane 7, renumbered
/// from its v18 claim — v18 is the c43/d109/e24 union); a pre-v19 file
/// loads both false.
///
/// **Not** in this version, and why: the TownManager rooms
/// (`WorldFile.cs:1687-1701`), the bestiary sights/chats
/// (`Bestiary.Sights`/`Chats`) and the CreativePowers table live on the
/// App side in this port (`tr-app/src/town.rs`, `bestiary.rs`, `creative.rs`),
/// and `tr-core` cannot see them — they need a `WorldSide` twin on the app's
/// save path, which is the recorded M6-SAVE-1b remainder. The vanilla's own
/// file-absent temps (`_tempSandstorm*`/`_tempParty*`/`_tempLanternNight*` —
/// no writer or loader site in `WorldFile.cs`) reset on load by design, so
/// the port saves none of them either (S1-J-7).
pub const VERSION: u32 = 18;
///
/// W4-f40 RETIRE receipt (research/105 §9.3): this branch claims NO v18
/// tail bytes — the lantern quad persists through f45's `.wld` flags-block
/// sites (`wld/sections/flags.rs`, the v≥207 arm), NOT a TRWLD tail field.
/// The v18 UNION (c39 research/114 §8, tip `4f72847e`: the 9-way 106-byte
/// fixed head + d109 dense LAST, VERSION 18) carries no f40 row; this
/// constant stays 17 until that union lands at the merge.
pub const VERSION: u32 = 19;

/// The v7+ sections' payload — the world chests, the banner kill counts, the
/// bestiary kill counts and the town NPCs. New sections grow this struct
/// rather than the entry points' return tuples.
#[derive(Default)]
pub struct WorldSide {
    pub chests: Vec<ChestRecord>,
    pub banners: Vec<BannerKill>,
    /// `BannerSystem.Save`'s `claimableBanners` half (`BannerSystem.cs:180-185`,
    /// the `version >= 289` load arm `:203-211`) — the claimable-banner pool
    /// the milestone fills (W4-e24). Sparse like `banners`; a pre-v18 file
    /// loads none (the pool starts empty, the C#'s pre-289 reset).
    pub banner_claimable: Vec<BannerClaimable>,
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

/// One banner's claimable count — the C# `BannerSystem.Save`'s
/// `claimableBanners` half (`ushort` per banner id, `BannerSystem.cs:180-185`);
/// the port stores the counts sparsely like `BannerKill`, so each record
/// carries its id (W4-e24).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BannerClaimable {
    pub banner: i32,
    pub count: u16,
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
    // v12: the secret-seed bitmasks — the classic u16 (`World.seeds`, the
    // `SaveWorldFlags` nine booleans) then the micro u64
    // (`World.micro_seeds`, the 37 `SecretSeed` flags).
    w.write_all(&world.seeds.sum().to_le_bytes())?;
    w.write_all(&world.micro_seeds.sum().to_le_bytes())?;
    // v13: the signs and the weighted pressure plates. Both are `World`
    // fields (loaded from the C# `.wld` in M6-SAVE-1), so the `.trwld` used
    // to lose them on reload. Sorted by position for a deterministic file —
    // the two vectors are load-order dependent (`apply_loaded` appends).
    let mut signs: Vec<&crate::wld::sections::WldSign> = world.signs.iter().collect();
    signs.sort_by_key(|s| (s.x, s.y));
    w.write_all(&(signs.len() as u32).to_le_bytes())?;
    for sign in signs {
        let text = sign.text.as_bytes();
        w.write_all(&(text.len() as u32).to_le_bytes())?;
        w.write_all(text)?;
        w.write_all(&sign.x.to_le_bytes())?;
        w.write_all(&sign.y.to_le_bytes())?;
    }
    let mut plates: Vec<&crate::wld::sections::WldPressurePlate> =
        world.pressure_plates.iter().collect();
    plates.sort_by_key(|p| (p.x, p.y));
    w.write_all(&(plates.len() as u32).to_le_bytes())?;
    for plate in plates {
        w.write_all(&plate.x.to_le_bytes())?;
        w.write_all(&plate.y.to_le_bytes())?;
    }
    // v14: the rain event's triple — the C#'s flag block writes the
    // save-time snapshot of `Main.raining`/`Main.rainTime`/`Main.maxRaining`
    // (`WorldFile.cs:1312-1314`), and the loader reads them back into the
    // temps (`:2127-2130`) for `SetOngoingToTemps` (`:159-162`) to apply.
    // The save path syncs the App's live `Rain` into these fields first
    // (the C#'s `SetTempToOngoing`, `:1058-1060`).
    w.write_all(&[u8::from(world.raining)])?;
    w.write_all(&world.rain_time.to_le_bytes())?;
    w.write_all(&world.max_raining.to_le_bytes())?;
    // v15: `NPC.downedQueenSlime` (`WorldFile.cs:1396`'s flags byte).
    w.write_all(&[u8::from(world.downed_queen_slime)])?;
    // v16: the five surface-background style bytes the renderer resolves ids
    // from — the vanilla rides them in the flags block (`WorldFile.cs:
    // 1319-1323`, `:1375`); the port's own block, same values, same order.
    w.write_all(&[
        world.corrupt_bg_style,
        world.jungle_bg_style,
        world.hallow_bg_style,
        world.crimson_bg_style,
        world.mushroom_bg_style,
    ])?;
    // v17: `NPC.downedFishron` (`NPC.cs:6369`, the case-370 death flag
    // `:79349`) — S1-D-3 W-9 fishron wiring; the v15 queen-slime byte's
    // shape.
    w.write_all(&[u8::from(world.downed_fishron)])?;
    // v18: the town-pet license world flags (W4-g87) — the C#'s
    // `NPC.boughtCat`/`boughtDog`/`boughtBunny` (`WorldFile.cs:1392-1394`),
    // the 4829/4830/4910 Pet License use-unlocks. At the g71c union merge
    // these 3 bytes seat [57..60] of the SEALED 107-byte v18 head (g88
    // research/103 §L3-ARBITER A3: after crimson/moon/underworld, before
    // empress [60] + the g90 books pair [61..63]; identity 44 B closes the
    // head, W×H dense LAST — research/263 §6.4; this branch keeps its own
    // 3-byte tail until then, forward-only).
    w.write_all(&[
        u8::from(world.bought_cat),
        u8::from(world.bought_dog),
        u8::from(world.bought_bunny),
    // v18: the world-identity block (W4-c43) — the `.wld` header's identity
    // fields (`WorldFile.cs:1225-1227` + `SaveWorldFlags:1250-1251`), the
    // same values the retail writer emits; a `.trwld` round trip used to
    // drop them.
    w.write_all(&world.generator_version.to_le_bytes())?; // `:1225`
    w.write_all(&world.unique_id)?; // `:1226`
    w.write_all(&world.world_id.to_le_bytes())?; // `:1227`
    w.write_all(&world.creation_time.to_le_bytes())?; // `:1250`
    w.write_all(&world.last_played.to_le_bytes())?; // `:1251`
    // v18: the packed framing variants (`World::variants`, the runtime
    // `Tile.bTileHeader2` bits 4-7, `Tile.cs:464-482`) as a dense
    // `width*height` byte block, row-major like the tiles (W4-d109, the save
    // half of S1-A-3). The C# `.wld` never writes these bits
    // (`SaveWorldTiles` `WorldFile.cs:1432-1604`) — a port-only persistence so
    // a `.trwld` load-after-quit keeps the rolled `frameNumber`/
    // `wallFrameNumber` instead of re-rolling. The reader restores them and
    // sets `variants_from_save`, which the framing passes read.
    w.write_all(&world.variants)?;
    // v18: the banner claimable counts (`BannerSystem.Save`'s
    // `claimableBanners` half, `BannerSystem.cs:180-185`) — a u32 count, then
    // per banner the id (i32) and the count (u16). The port's sparse map
    // where the vanilla writes a dense `ushort[293]` array (W4-e24).
    w.write_all(&(side.banner_claimable.len() as u32).to_le_bytes())?;
    for c in &side.banner_claimable {
        w.write_all(&c.banner.to_le_bytes())?;
        w.write_all(&c.count.to_le_bytes())?;
    }
    // v19: the combat-book-two + Peddler's-Satchel world flags (W4-g90
    // lane 7) — `NPC.combatBookVolumeTwoWasUsed` / `peddlersSatchelWasUsed`
    // (`NPC.cs:6341/6343`; the C# .wld flags bytes `WorldFile.cs:1407-1408`,
    // read gates 259/260). The v15 queen-slime byte's shape; book ONE
    // (`combat_book_was_used`) rides the .wld flags half only (its .trwld
    // carrier is the recorded remainder — the i35 fishing context reads
    // the .wld-loaded field).
    // UNION SEAT (research/135 §10g, the 107-DEFINITIVE receipt — g88
    // research/103 §L3-DEFINITIVE D3, fc5cf11ae): at the merge this pair
    // rides offsets [61]/[62] of the 107-byte fixed head — the LAST flags
    // block (STRICT C# row order: crimson [50] → moon [51..56] → underworld
    // [56] → pets [57..60] → empress [60] → books [61..63]), identity
    // [63..107] closes the head, d109's dense variants LAST ([107..]);
    // journey 19/20 (135 §4, 21-byte) → 22/23 (138 §5a, 68-byte) → 104/105
    // (114 §8, 106-byte) → 61/62 (103 D3, 107-byte). This branch keeps its
    // own 2-byte tail until the union lands (the standing decision).
    w.write_all(&[
        u8::from(world.combat_book_volume_two_was_used),
        u8::from(world.peddlers_satchel_was_used),
    ])?;
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
    // v12: the secret-seed bitmasks (the classic u16, then the micro u64).
    if version >= 12 {
        world.seeds = crate::seeds::SeedFlags::from_sum(r.u16()?);
        world.micro_seeds = crate::seeds::MicroSeedFlags::from_sum(r.u64()?);
    }
    // v13: the signs and the weighted pressure plates.
    if version >= 13 {
        let count = r.u32()? as usize;
        world.signs = Vec::with_capacity(count.min(4096));
        for _ in 0..count {
            let len = r.u32()? as usize;
            // The file's string convention (u32 length + UTF-8), the same one
            // the world name and the chest names use.
            let text = String::from_utf8(r.take(len)?.to_vec())
                .map_err(|e| anyhow!("sign text at {len} bytes is not UTF-8: {e}"))?;
            world.signs.push(crate::wld::sections::WldSign {
                text,
                x: r.i32()?,
                y: r.i32()?,
            });
        }
        let count = r.u32()? as usize;
        world.pressure_plates = Vec::with_capacity(count.min(65536));
        for _ in 0..count {
            world
                .pressure_plates
                .push(crate::wld::sections::WldPressurePlate {
                    x: r.i32()?,
                    y: r.i32()?,
                });
        }
    }
    // v14: the rain event's triple (see the writer's citation) — a pre-v14
    // file loads with the event off, the vanilla's own reset behavior.
    if version >= 14 {
        world.raining = r.u8()? != 0;
        world.rain_time = r.i32()?;
        world.max_raining = r.f32()?;
    }
    // v15: the Queen Slime's downed flag (see the writer's citation).
    if version >= 15 {
        world.downed_queen_slime = r.u8()? != 0;
    }
    // v16: the five surface-background style bytes (see the writer's citation).
    if version >= 16 {
        world.corrupt_bg_style = r.u8()?;
        world.jungle_bg_style = r.u8()?;
        world.hallow_bg_style = r.u8()?;
        world.crimson_bg_style = r.u8()?;
        world.mushroom_bg_style = r.u8()?;
    } else {
        // A pre-v16 file predates the variants' style tracking: fall back to
        // the `setBG` defaults (`WorldGen.corruptBG` etc. start at 0, whose
        // arm is the classic `12/13/14`-style row set).
        world.corrupt_bg_style = 0;
        world.jungle_bg_style = 0;
        world.hallow_bg_style = 0;
        world.crimson_bg_style = 0;
        world.mushroom_bg_style = 0;
    }
    // v17: Duke Fishron's downed flag (see the writer's citation).
    if version >= 17 {
        world.downed_fishron = r.u8()? != 0;
    }
    // v18: the world-identity block (see the writer's citation) — a pre-v18
    // file keeps the `World::new` zeros, exactly the values the pre-W4-c43
    // saves carried.
    if version >= 18 {
        world.generator_version = r.u64()?;
        for b in &mut world.unique_id {
            *b = r.u8()?;
        }
        world.world_id = r.i32()?;
        world.creation_time = r.u64()? as i64;
        world.last_played = r.u64()? as i64;
    }
    // v18: the packed framing variants (see the writer's citation). A dense
    // `width*height` block, row-major like the tiles; the load restores them
    // and arms `variants_from_save` so the full-world framing passes reuse the
    // stored values instead of re-rolling. A pre-v18 file keeps the
    // `World::new` zeros and re-rolls (the C#'s reset semantics).
    if version >= 18 {
        let n = world.tiles.len();
        world.variants = r.take(n)?.to_vec();
        world.variants_from_save = true;
    }
    // v18: the banner claimable counts (`BannerSystem.Load`'s `version >= 289`
    // claimable half, `BannerSystem.cs:203-211`); a pre-v18 file loads none
    // (the pool starts empty, the C#'s pre-289 reset). W4-e24.
    if version >= 18 {
        side.banner_claimable = read_banner_claimable_v18(&mut r)?;
    }
    // v19: the combat-book-two + Peddler's-Satchel world flags (W4-g90
    // lane 7 — the v15 queen-slime byte's shape). A pre-v18 file loads
    // both false (the fresh-world default; the C# reads them from .wld
    // v259/v260, `WorldFile.cs:2404-2405`). UNION SEAT: [61]/[62] of the
    // 107-byte DEFINITIVE head at the merge (research/135 §10g / g88
    // research/103 §L3-DEFINITIVE D3, fc5cf11ae — the writer's seat note).
    if version >= 19 {
        world.combat_book_volume_two_was_used = r.u8()? != 0;
        world.peddlers_satchel_was_used = r.u8()? != 0;
    }
    world.revision = 0;
    Ok((world, side))
}

/// The v18 banner-claimable reader — the exact inverse of the v18 write.
fn read_banner_claimable_v18(r: &mut Reader<'_>) -> Result<Vec<BannerClaimable>> {
    let count = r.u32()? as usize;
    let mut out = Vec::with_capacity(count.min(4096));
    for _ in 0..count {
        let banner = r.i32()?;
        let count = r.u16()?;
        out.push(BannerClaimable { banner, count });
    }
    Ok(out)
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
        out.push(NpcRecord {
            kind,
            given_name,
            x,
            y,
            homeless,
            home_x,
            home_y,
        });
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
                items.push(ChestItem {
                    stack: 0,
                    kind: 0,
                    prefix: 0,
                });
                continue;
            }
            items.push(ChestItem {
                stack,
                kind: r.u16()?,
                prefix: r.u8()?,
            });
        }
        out.push(ChestRecord {
            x,
            y,
            name,
            slots: items,
        });
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
        assert_eq!(
            &buf[8..12],
            &[18, 0, 0, 0],
            "the writer emits the current v18"
        );
        let (back, _chests) = parse(&buf).unwrap();
        assert_eq!(back.sundial_cooldown, 5);
        assert_eq!(back.moondial_cooldown, 8);
    }

    #[test]
    fn dw_1_4_v3_loads_zero_cooldowns() {
        // DW-1.4: a v3 payload (the version field patched) loads with both
        // cooldowns at 0 — the v3 parse stops after the paint extras and
        // never reads the v4 cooldown bytes (`World::new`'s 0s stand). The
        // end-trim only has to stay inside the post-extras tail (the v18
        // banner-claimable count lengthens it, W4-e24); 3 bytes is safely
        // within it.
        let mut w = World::new("Test".into(), 32, 32);
        w.sundial_cooldown = 7;
        let mut buf = Vec::new();
        write_world(&mut buf, &w, &WorldSide::default()).unwrap();
        // Peel the v18 tail (identity 44 + variants w*h, v18_tail_len
        // covers both), then the v4 (2) + v5 (1) tails.
        let tail = v18_tail_len(&buf) + 3;
        buf.truncate(buf.len() - tail);
        buf[8..12].copy_from_slice(&3u32.to_le_bytes());
        let (back, _chests) = parse(&buf).unwrap();
        assert_eq!(back.sundial_cooldown, 0);
        assert_eq!(back.moondial_cooldown, 0);
    }

    #[test]
    fn dw_1_4_version_above_4_rejected() {
        // DW-1.4 (dirty): the existing loader rejects a version above the
        // current one (`save.rs` version gate). v18 (the town-pet license
        // flags, W4-g87) is now the current format, so the gate pins 18.
        // current one (`save.rs` version gate). v19 (the book-two/satchel
        // pair, W4-g90 lane 7, after the v18 c43/d109/e24 union) is now
        // the current format, so the gate pins 19.
        let w = World::new("Test".into(), 32, 32);
        let mut buf = Vec::new();
        write_world(&mut buf, &w, &WorldSide::default()).unwrap();
        buf[8..12].copy_from_slice(&19u32.to_le_bytes());
        assert!(parse(&buf).is_err(), "version 19 is unsupported");
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
        // The v5 tail is the last 9 bytes AHEAD of the v6..v12 sections:
        // the fixed flags block + the four empty colour counts + one empty
        // entity count + the chests + banners + bestiary + NPC counts and
        // the v12 bitmasks (2 + 8) sit after it — poison the x.
        // Locate the v5 tail by its own bytes — [presence][x][y] = 1, 30,
        // 20 — rather than by counting the v6..v12 sections behind it (that
        // count changes every time a section lands).
        let pattern = {
            let mut p = vec![1u8];
            p.extend_from_slice(&30i32.to_le_bytes());
            p.extend_from_slice(&20i32.to_le_bytes());
            p
        };
        let at = buf
            .windows(pattern.len())
            .position(|w| w == pattern.as_slice())
            .expect("the bed-spawn tail is in the payload");
        buf[at + 1..at + 5].copy_from_slice(&99999i32.to_le_bytes());
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
        // (+1: the v17 downedFishron byte rides the flags block's tail;
        // the v18 tail — identity 44 + variants — peels via v18_tail_len.)
        let tail = v19_tail_len(&buf) + V6_FLAGS_LEN + 1 + 4 * 4 + 4 + 4 + 4 + 4 + 4 + 1;
        buf.truncate(buf.len() - tail);
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
    fn m5_seed_1_the_secret_seed_flags_round_trip_in_v12() {
        // M5-SEED-1: the v12 block carries the classic secret-seed flags
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

    #[test]
    fn m5_seed_2_the_micro_seed_flags_round_trip_in_v12() {
        // M5-SEED-2: the v12 block also carries the 37 micro-seed flags as
        // the u64 bitmask; a classic world keeps all bits 0 and a seeded
        // world survives the save/load.
        let mut w = World::new("Test".into(), 32, 32);
        let mut buf = Vec::new();
        write_world(&mut buf, &w, &WorldSide::default()).unwrap();
        assert_eq!(back_micro(&buf), 0, "a classic world writes no micro bits");
        let (back, _) = parse(&buf).unwrap();
        assert_eq!(back.micro_seeds, crate::seeds::MicroSeedFlags::none());
        w.micro_seeds = crate::seeds::micro_flags_for_texts(["pumpkinseason", "arachnophobia"]);
        buf.clear();
        write_world(&mut buf, &w, &WorldSide::default()).unwrap();
        assert_eq!(back_micro(&buf), w.micro_seeds.sum());
        let (back, _) = parse(&buf).unwrap();
        assert_eq!(back.micro_seeds, w.micro_seeds);
        // A v11 file (the same stream minus the v12 block and the v13..v16
        // tails, the version byte restored) loads as the classic world it was.
        // The v18 tail (identity 44 + variants) peels via v18_tail_len.
        let tail = v18_tail_len(&buf) + 10 + EMPTY_V16_TAIL;
        buf.truncate(buf.len() - tail);
        buf[8..12].copy_from_slice(&11u32.to_le_bytes());
        let (back, _) = parse(&buf).unwrap();
        assert_eq!(back.micro_seeds, crate::seeds::MicroSeedFlags::none());
    }

    #[test]
    fn m6_save_1b_the_signs_and_plates_round_trip_in_v13() {
        // M6-SAVE-1b: the signs and the weighted pressure plates are `World`
        // fields the C# `.wld` already loads (`LoadSigns`,
        // `WorldFile.cs:1666-1685`; `LoadWeightedPressurePlates`, `:3373-3384`),
        // but the `.trwld` had no twin, so a native save dropped them.
        let mut w = World::new("Test".into(), 32, 32);
        let mut buf = Vec::new();
        write_world(&mut buf, &w, &WorldSide::default()).unwrap();
        let (back, _) = parse(&buf).unwrap();
        assert!(back.signs.is_empty(), "an empty world writes no signs");
        assert!(back.pressure_plates.is_empty());

        w.signs = vec![
            crate::wld::sections::WldSign {
                text: "second".into(),
                x: 9,
                y: 4,
            },
            // Out of order on purpose: the writer sorts by position so the
            // file is deterministic, and the reader keeps that order.
            crate::wld::sections::WldSign {
                text: "first — ünïcode".into(),
                x: 3,
                y: 7,
            },
        ];
        w.pressure_plates = vec![
            crate::wld::sections::WldPressurePlate { x: 12, y: 5 },
            crate::wld::sections::WldPressurePlate { x: 2, y: 1 },
        ];
        buf.clear();
        write_world(&mut buf, &w, &WorldSide::default()).unwrap();
        let (back, _) = parse(&buf).unwrap();
        assert_eq!(back.signs.len(), 2);
        assert_eq!(back.signs[0].text, "first — ünïcode", "sorted by x");
        assert_eq!((back.signs[0].x, back.signs[0].y), (3, 7));
        assert_eq!(back.signs[1].text, "second");
        assert_eq!((back.signs[1].x, back.signs[1].y), (9, 4));
        assert_eq!(
            back.pressure_plates
                .iter()
                .map(|p| (p.x, p.y))
                .collect::<Vec<_>>(),
            vec![(2, 1), (12, 5)],
            "the plates sort by position too"
        );

        // A v12 file (the same stream minus the v13 block and the v14 tail)
        // loads as the sign-less world it was — the version gate, not a
        // length guess.
        let cur_len = buf.len();
        let mut v12 = buf.clone();
        // The v13 block is the tail: 4 + (4+len+8) + 4 + 8 bytes, then the
        // v14 rain block (9 bytes), the v15 flag (1), the v16 styles (5),
        // the v17 fishron flag (1) and the v18 pet-license flags (3, W4-g87).
        let tail = 4 + (4 + "first — ünïcode".len() + 8) + (4 + "second".len() + 8) + 4 + 8 + 9 + 1 + 5 + 1 + 3;
        v12.truncate(cur_len - tail - 1);
        // the v17 fishron flag (1) and the v18 identity block (44).
        let tail = 4 + (4 + "first — ünïcode".len() + 8) + (4 + "second".len() + 8) + 4 + 8 + 9 + 1 + 5 + 1 + EMPTY_V18_TAIL;
        // Peel the v18 variants block first (W4-d109); the -1 is the v13
        // block's leading u32 sign count.
        v12.truncate(cur_len - v19_tail_len(&v12) - tail - 1);
        v12[8..12].copy_from_slice(&12u32.to_le_bytes());
        let (back, _) = parse(&v12).unwrap();
        assert!(back.signs.is_empty(), "a v12 file has no sign block");
        assert!(back.pressure_plates.is_empty());
    }

    /// A `WorldSide` carrying only the chest records (the v7 tests).
    fn chest_side(chests: &[ChestRecord]) -> WorldSide {
        WorldSide {
            chests: chests.to_vec(),
            banners: Vec::new(),
            bestiary: Vec::new(),
            npcs: Vec::new(),
            ore_tiers: None,
            ..Default::default()
        }
    }

    /// The v6 block's u16 bitmask, read back out of the written buffer — the
    /// two bytes ahead of the fixed v7 flags block.
    /// The empty-world v13 tail: the signs' u32 count + the plates' u32 count
    /// (0 + 0). The v12 fixtures below write a world with no signs or plates,
    /// so the two blocks ahead of it sit this many bytes from the end.
    const EMPTY_V13_TAIL: usize = 8;
    /// The v14 rain block: 1 bool + 4 i32 + 4 f32 (the constants-off defaults
    /// for an empty rain event), the v15 Queen Slime flag byte, the v16
    /// five style bytes, the v17 downedFishron byte (S1-D-3 W-9) and the
    /// v18 town-pet license flags (3 bytes, W4-g87).
    const EMPTY_V16_TAIL: usize = EMPTY_V13_TAIL + 9 + 1 + 5 + 1 + 3;
    /// five style bytes and the v17 downedFishron byte (S1-D-3 W-9).
    const EMPTY_V16_TAIL: usize = EMPTY_V13_TAIL + 9 + 1 + 5 + 1;
    /// The v18 identity block (W4-c43): u64 generator version + 16 GUID bytes
    /// + i32 world id + two i64 ToBinary stamps.
    const EMPTY_V18_TAIL: usize = 8 + 16 + 4 + 8 + 8;

    /// The v18 framing-variants block's size: a dense `width*height` byte
    /// array (W4-d109). The end-relative fixtures below peel it off before
    /// truncating the older tails; the version is read from the header.
    fn v18_tail_len(buf: &[u8]) -> usize {
        let version = u32::from_le_bytes(buf[8..12].try_into().unwrap());
        if version < 18 {
            return 0;
        }
        // The v18 identity block (W4-c43, 44 bytes) precedes the variants.
        // Header layout: magic(8) + version(4) + seed(8) + name_len(4) + name
        // + width(4) + height(4). The name length sits at offset 20.
        let name_len = u32::from_le_bytes(buf[20..24].try_into().unwrap()) as usize;
        let dims = 24 + name_len;
        let width = i32::from_le_bytes(buf[dims..dims + 4].try_into().unwrap()) as usize;
        let height = i32::from_le_bytes(buf[dims + 4..dims + 8].try_into().unwrap()) as usize;
        // v18 layout: identity (44) + variants (w*h) + the banner
        // claimable block (u32 count + 6 bytes per entry, W4-e24). The
        // end-relative fixtures below all carry EMPTY pools (count 0),
        // so the tail is 44 + w*h + 4; a non-empty pool's exact length
        // rides the sequential parse, never this helper.
        44 + width * height + 4
    }

    /// The v19 tail: the v18 union (identity 44 + variants + claimable)
    /// plus the book-two/satchel pair (2 bytes, W4-g90 lane 7). The
    /// end-relative fixtures carry empty pools (the v18 helper's note).
    fn v19_tail_len(buf: &[u8]) -> usize {
        let version = u32::from_le_bytes(buf[8..12].try_into().unwrap());
        if version < 19 {
            return v18_tail_len(buf);
        }
        v18_tail_len(buf) + 2
    }

    fn back_flags(buf: &[u8]) -> u16 {
        let end = buf.len() - EMPTY_V16_TAIL - v18_tail_len(buf);
        u16::from_le_bytes([buf[end - 10], buf[end - 9]])
    }

    /// The v12 block's micro u64 bitmask (the eight bytes ahead of the v13
    /// tail).
    fn back_micro(buf: &[u8]) -> u64 {
        let end = buf.len() - EMPTY_V16_TAIL - v18_tail_len(buf);
        u64::from_le_bytes(buf[end - 8..end].try_into().unwrap())
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
            (5, 5, 378, 0i16), // TrainingDummy
            (6, 6, 395, 0),    // ItemFrame
            (7, 7, 423, 36),   // LogicSensor (check 2 rides frameY/18)
            (8, 8, 597, 0),    // TeleportationPylon
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
        w.tile_entities.by_id_mut(frame).unwrap().data = TeData::Item(Some(TeItem {
            kind: 75,
            prefix: 3,
            stack: 12,
        }));
        let sensor = w.tile_entities.place_unchecked(TeKind::LogicSensor, 7, 7);
        w.tile_entities.by_id_mut(sensor).unwrap().data = TeData::Logic {
            check: 2,
            on: true,
            counted: 15,
        };
        let pylon = w
            .tile_entities
            .place_unchecked(TeKind::TeleportationPylon, 8, 8);

        let mut buf = Vec::new();
        write_world(&mut buf, &w, &WorldSide::default()).unwrap();
        let (back, _chests) = parse(&buf).unwrap();
        // The set flags round-trip and the unset ones stay default.
        assert!(back.blood_moon && back.downed_eoc && back.downed_golem && back.saved_goblin);
        assert!(
            back.downed_pirates && back.shadow_orb_smashed && back.spawn_meteor && back.hard_mode
        );
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
        assert!(
            !back.wires.get(9, 9, WireColor::Yellow),
            "the clear round-trips"
        );
        assert!(
            !back.wires.get(3, 4, WireColor::Blue),
            "no cross-colour bleed"
        );
        // The tile entities — the stored ids and payloads.
        assert_eq!(
            back.tile_entities.get_by_id(dummy).unwrap().data,
            TeData::Dummy(7)
        );
        assert_eq!(
            back.tile_entities.get_by_id(frame).unwrap().data,
            TeData::Item(Some(TeItem {
                kind: 75,
                prefix: 3,
                stack: 12
            }))
        );
        assert_eq!(
            back.tile_entities.get_by_id(sensor).unwrap().data,
            TeData::Logic {
                check: 2,
                on: true,
                counted: 15
            }
        );
        assert_eq!(back.tile_entities.get_at(8, 8).map(|e| e.id), Some(pylon));
        assert_eq!(back.tile_entities.get_at(5, 5).map(|e| e.id), Some(dummy));
    }

    #[test]
    fn s1_j_7_rain_round_trips_load_after_quit() {
        // The C#'s save-time snapshot rides the flags block
        // (`_tempRaining`/`_tempRainTime`/`_tempMaxRain`, `WorldFile.cs:1312-1314`;
        // `SetTempToOngoing` `:1058-1060` refreshes the temps before the write) and
        // `SetOngoingToTemps` (`:159-162`) restores it on load — load-after-quit
        // keeps the rain event (`Main.cloudAlpha` comes back as `maxRaining`).
        // Watched RED against the v13 writer: load returned the defaults.
        let mut w = World::new("Test".into(), 64, 48);
        w.raining = true;
        w.rain_time = 12_345;
        w.max_raining = 0.75;
        let mut buf = Vec::new();
        write_world(&mut buf, &w, &WorldSide::default()).unwrap();
        let (back, _) = parse(&buf).unwrap();
        assert!(back.raining, "the rain event survives the reload");
        assert_eq!(back.rain_time, 12_345);
        assert!((back.max_raining - 0.75).abs() < f32::EPSILON);
    }

    #[test]
    fn s1_f_16_the_queen_slime_flag_round_trips() {
        // `NPC.downedQueenSlime` rides the vanilla `.wld` flags block
        // (`WorldFile.cs:1396`) and the Skeleton Merchant's rank gate reads
        // it (`Chest.cs:3428`); the port's v15 tail carries the same bit.
        let mut w = World::new("Test".into(), 64, 48);
        w.downed_queen_slime = true;
        let mut buf = Vec::new();
        write_world(&mut buf, &w, &WorldSide::default()).unwrap();
        let (back, _) = parse(&buf).unwrap();
        assert!(back.downed_queen_slime, "the flag survives the reload");
        // A pre-v15 file (the tail truncated, the version patched) loads
        // with the flag down — the vanilla's own reset arm (`:2376`).
        // The v16 style bytes + the v17 fishron byte + the v18 pet-license
        // flags ride behind the v15 flag (9 bytes).
        let mut v14 = buf.clone();
        v14.truncate(v14.len() - 1 - 5 - 1 - 3);
        // The v16 style bytes + the v17 fishron byte ride behind the v15
        // flag (6 bytes); the v18 identity block (44) rides the stream end
        // after the v18 variants block (W4-d109) — peel the variants first.
        let mut v14 = buf.clone();
        let tail = v19_tail_len(&v14) + 1 + 5 + 1;
        v14.truncate(v14.len() - tail);
        v14[8..12].copy_from_slice(&14u32.to_le_bytes());
        let (back, _) = parse(&v14).unwrap();
        assert!(!back.downed_queen_slime, "a pre-v15 file resets the flag");
    }

    /// W4-g90 lane 7: the book-two + satchel world flags round-trip through
    /// the v18 tail (`NPC.combatBookVolumeTwoWasUsed` /
    /// `peddlersSatchelWasUsed`, the C# .wld bytes `WorldFile.cs:1407-1408`);
    /// a pre-v18 file loads both false (the fresh-world default).
    #[test]
    fn w4_g90_the_book_two_and_satchel_flags_round_trip_in_v18() {
        let mut w = World::new("Test".into(), 64, 48);
        w.combat_book_volume_two_was_used = true;
        w.peddlers_satchel_was_used = true;
        let mut buf = Vec::new();
        write_world(&mut buf, &w, &WorldSide::default()).unwrap();
        let (back, _) = parse(&buf).unwrap();
        assert!(back.combat_book_volume_two_was_used, "book two survives");
        assert!(back.peddlers_satchel_was_used, "the satchel survives");
        // A pre-v18 file (the pair truncated, the version patched) loads
        // both false — the version gate, not a length guess. Union re-pin:
        // the -2 becomes -107 (research/135 §10g / g88 103 D3, the
        // 107-byte definitive head).
        let mut v17 = buf.clone();
        v17.truncate(v17.len() - 2);
        v17[8..12].copy_from_slice(&17u32.to_le_bytes());
        let (back, _) = parse(&v17).unwrap();
        assert!(!back.combat_book_volume_two_was_used, "pre-v18: false");
        assert!(!back.peddlers_satchel_was_used);
        // The mixed state: one flag up, one down — each byte is its own.
        w.peddlers_satchel_was_used = false;
        buf.clear();
        write_world(&mut buf, &w, &WorldSide::default()).unwrap();
        let (back, _) = parse(&buf).unwrap();
        assert!(back.combat_book_volume_two_was_used);
        assert!(!back.peddlers_satchel_was_used);
    }

    #[test]
    fn w4_c43_the_world_identity_round_trips_in_v18() {
        // W4-c43: the v18 tail carries the `.wld` header's identity fields
        // (`WorldFile.cs:1225-1227` + `SaveWorldFlags:1250-1251`) — a
        // `.trwld` round trip used to drop them (the audit's placeholder
        // finding, research/42).
        let mut w = World::new("Test".into(), 32, 32);
        w.generator_version = 140_015_933_8497; // `CreateMetadata:603`
        w.unique_id = [1, 2, 3, 4, 5, 6, 7, 0x48, 0x89, 10, 11, 12, 13, 14, 15, 16];
        w.world_id = 0x1234_5678;
        w.creation_time = 0x48DF_4506_E391_C000u64 as i64; // the fixture clock
        w.last_played = w.creation_time + 10_000_000; // +1 s of ticks
        let mut buf = Vec::new();
        write_world(&mut buf, &w, &WorldSide::default()).unwrap();
        let (back, _) = parse(&buf).unwrap();
        assert_eq!(back.generator_version, w.generator_version);
        assert_eq!(back.unique_id, w.unique_id);
        assert_eq!(back.world_id, w.world_id);
        assert_eq!(back.creation_time, w.creation_time);
        assert_eq!(back.last_played, w.last_played);
        // A pre-v18 file (the block truncated, the version patched) loads the
        // `World::new` zeros — the values the pre-W4-c43 saves carried.
        let mut v17 = buf.clone();
        v17.truncate(v17.len() - EMPTY_V18_TAIL);
        v17[8..12].copy_from_slice(&17u32.to_le_bytes());
        let (back, _) = parse(&v17).unwrap();
        assert_eq!(back.generator_version, 0);
        assert_eq!(back.unique_id, [0u8; 16]);
        assert_eq!(back.world_id, 0);
        assert_eq!(back.creation_time, 0);
        assert_eq!(back.last_played, 0);
    }

    #[test]
    fn w4_c43_the_created_identity_stamp_follows_create_metadata() {
        // `WorldFile.CreateMetadata` (`WorldFile.cs:600-604`): the stamp sets
        // the version literal, a v4-shaped GUID and a Utc-kind CreationTime.
        let mut w = World::new("Test".into(), 32, 32);
        let mut rng = crate::unified_random::UnifiedRandom::new(1234);
        crate::wld::stamp_created_identity(&mut w, &mut rng);
        assert_eq!(w.generator_version, crate::wld::WORLD_GENERATOR_VERSION);
        assert_eq!(w.unique_id[7] & 0xF0, 0x40, "the GUID version nibble is 4");
        assert_eq!(w.unique_id[8] & 0xC0, 0x80, "the RFC-4122 variant bits");
        assert_ne!(w.creation_time & (0x4000_0000_0000_0000u64 as i64), 0, "Utc kind");
        assert!(w.creation_time > crate::clock::TICKS_AT_UNIX_EPOCH);
        // The WorldId is NOT part of the stamp — the caller rolls it (the
        // `WorldGen.cs:11337` stand-in), so the stamp leaves it untouched.
        assert_eq!(w.world_id, 0);
        // A different stream draws a different GUID (uniqueness per world).
        let mut w2 = World::new("Test2".into(), 32, 32);
        let mut rng2 = crate::unified_random::UnifiedRandom::new(4321);
        crate::wld::stamp_created_identity(&mut w2, &mut rng2);
        assert_ne!(w.unique_id, w2.unique_id);
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
        // (+1: the v17 downedFishron byte rides the flags block's tail;
        // the v18 tail — identity 44 + variants — peels via v18_tail_len.)
        let tail = v19_tail_len(&buf) + 2 + V6_FLAGS_LEN + 1 + 4 * 4 + 4;
        buf.truncate(buf.len() - tail);
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
        // Peel the v18 variants block, then cut inside the TE record (W4-d109).
        buf.truncate(buf.len() - v18_tail_len(&buf) - 2);
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
        w.tile_entities
            .place_unchecked(TeKind::TeleportationPylon, 7, 3);
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
                    ChestItem {
                        stack: 35,
                        kind: 19,
                        prefix: 0,
                    },
                    ChestItem {
                        stack: 0,
                        kind: 0,
                        prefix: 0,
                    }, // interior air
                    ChestItem {
                        stack: 1,
                        kind: 1541,
                        prefix: 81,
                    },
                ],
            },
            ChestRecord {
                x: 40,
                y: 9,
                name: String::new(),
                slots: Vec::new(),
            },
        ];
        let mut buf = Vec::new();
        write_world(&mut buf, &w, &chest_side(&chests)).unwrap();
        assert_eq!(
            &buf[8..12],
            &[18, 0, 0, 0],
            "the writer emits the current v18"
        );
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
        let chests = vec![ChestRecord {
            x: 3,
            y: 4,
            name: "Wooden Chest".into(),
            slots: vec![ChestItem {
                stack: 99,
                kind: 9,
                prefix: 0,
            }],
        }];
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
            slots: vec![ChestItem {
                stack: 99,
                kind: 9,
                prefix: 0,
            }],
        }];
        let mut buf = Vec::new();
        write_world(&mut buf, &w, &chest_side(&chests)).unwrap();
        // Peel the v18 variants block, then cut inside the item record (W4-d109).
        buf.truncate(buf.len() - v18_tail_len(&buf) - 3);
        assert!(parse(&buf).is_err());
    }

    #[test]
    fn dw_2_2_the_chest_section_is_deterministic() {
        // DW-2.2: the same records written twice are byte-equal.
        let mut w = World::new("Test".into(), 32, 32);
        let chests = vec![
            ChestRecord {
                x: 5,
                y: 6,
                name: "Ivy Chest".into(),
                slots: vec![ChestItem {
                    stack: 20,
                    kind: 166,
                    prefix: 3,
                }],
            },
            ChestRecord {
                x: 7,
                y: 8,
                name: "Water Chest".into(),
                slots: Vec::new(),
            },
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
                BannerKill {
                    banner: 3331,
                    kills: 50,
                },
                BannerKill {
                    banner: 71,
                    kills: 0,
                },
                BannerKill {
                    banner: 1569,
                    kills: 999_999_999,
                },
            ],
            bestiary: Vec::new(),
            npcs: Vec::new(),
            ore_tiers: None,
            ..Default::default()
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
                slots: vec![ChestItem {
                    stack: 3,
                    kind: 19,
                    prefix: 0,
                }],
            }],
            banners: vec![BannerKill {
                banner: 3331,
                kills: 7,
            }],
            bestiary: Vec::new(),
            npcs: Vec::new(),
            ore_tiers: None,
            ..Default::default()
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
            banners: vec![BannerKill {
                banner: 3331,
                kills: 7,
            }],
            bestiary: Vec::new(),
            npcs: Vec::new(),
            ore_tiers: None,
            ..Default::default()
        };
        let mut buf = Vec::new();
        write_world(&mut buf, &w, &side).unwrap();
        // Peel the v18 variants block, then cut inside the banner record (W4-d109).
        buf.truncate(buf.len() - v18_tail_len(&buf) - 2);
        assert!(parse(&buf).is_err());
    }

    #[test]
    fn dw_2_3_the_banner_section_is_deterministic() {
        // DW-2.3: the same records written twice are byte-equal.
        let mut w = World::new("Test".into(), 32, 32);
        let side = WorldSide {
            chests: Vec::new(),
            banners: vec![
                BannerKill {
                    banner: 3331,
                    kills: 7,
                },
                BannerKill {
                    banner: 71,
                    kills: 0,
                },
                BannerKill {
                    banner: 1569,
                    kills: 12,
                },
            ],
            bestiary: Vec::new(),
            npcs: Vec::new(),
            ore_tiers: None,
            ..Default::default()
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
    fn w4_e24_the_banner_claimable_counts_round_trip_in_v18() {
        // W4-e24: the v18 tail carries `BannerSystem.Save`'s claimable half
        // (`ushort` per banner id, `BannerSystem.cs:180-185`) as the port's
        // sparse records — including a zeroed banner (a fully claimed pool
        // entry survives, the `BannerKill` zero precedent) and the 9999 cap
        // (`Item.CommonMaxStack`, `Item.cs:49966`).
        let w = World::new("Test".into(), 32, 32);
        let side = WorldSide {
            chests: Vec::new(),
            banners: vec![BannerKill {
                banner: 69,
                kills: 150,
            }],
            banner_claimable: vec![
                BannerClaimable {
                    banner: 69,
                    count: 3,
                },
                BannerClaimable {
                    banner: 151,
                    count: 0,
                },
                BannerClaimable {
                    banner: 119,
                    count: 9999,
                },
            ],
            bestiary: Vec::new(),
            npcs: Vec::new(),
            ore_tiers: None,
            ..Default::default()
        };
        let mut buf = Vec::new();
        write_world(&mut buf, &w, &side).unwrap();
        assert_eq!(&buf[8..12], &[18, 0, 0, 0], "the writer emits v18");
        let (_, back_side) = parse(&buf).unwrap();
        assert_eq!(back_side.banner_claimable, side.banner_claimable);
        assert_eq!(back_side.banners, side.banners, "the kill counts still ride");
    }

    #[test]
    fn w4_e24_a_pre_v18_payload_loads_no_claimable_banners() {
        // W4-e24: a v17 payload (the version field patched — the v18 tail
        // sits unread) parses with the kill counts intact and no claimable
        // pool: the C# `Load`'s pre-289 reset (`BannerSystem.cs:203-211`
        // guards the claimable half on the version).
        let w = World::new("Test".into(), 32, 32);
        let side = WorldSide {
            chests: Vec::new(),
            banners: vec![BannerKill {
                banner: 69,
                kills: 50,
            }],
            banner_claimable: vec![BannerClaimable {
                banner: 69,
                count: 1,
            }],
            bestiary: Vec::new(),
            npcs: Vec::new(),
            ore_tiers: None,
            ..Default::default()
        };
        let mut buf = Vec::new();
        write_world(&mut buf, &w, &side).unwrap();
        buf[8..12].copy_from_slice(&17u32.to_le_bytes());
        let (_, back_side) = parse(&buf).unwrap();
        assert_eq!(back_side.banners, side.banners, "the v8 kills still load");
        assert!(
            back_side.banner_claimable.is_empty(),
            "no claimable pool below v18"
        );
    }

    #[test]
    fn w4_e24_a_truncated_claimable_section_bails() {
        // W4-e24 (dirty): the claimable section is self-delimiting; a payload
        // cut mid-record is a structural error. The tail is the u32 count +
        // one (i32, u16) record; -2 cuts into the record's u16 count.
        let w = World::new("Test".into(), 32, 32);
        let side = WorldSide {
            chests: Vec::new(),
            banners: Vec::new(),
            banner_claimable: vec![BannerClaimable {
                banner: 69,
                count: 1,
            }],
            bestiary: Vec::new(),
            npcs: Vec::new(),
            ore_tiers: None,
            ..Default::default()
        };
        let mut buf = Vec::new();
        write_world(&mut buf, &w, &side).unwrap();
        buf.truncate(buf.len() - 2); // inside the claimable record's u16
        assert!(parse(&buf).is_err());
    }

    #[test]
    fn w4_e24_the_claimable_section_is_deterministic() {
        // W4-e24: the same records written twice are byte-equal.
        let w = World::new("Test".into(), 32, 32);
        let side = WorldSide {
            chests: Vec::new(),
            banners: Vec::new(),
            banner_claimable: vec![
                BannerClaimable {
                    banner: 69,
                    count: 2,
                },
                BannerClaimable {
                    banner: 151,
                    count: 7,
                },
            ],
            bestiary: Vec::new(),
            npcs: Vec::new(),
            ore_tiers: None,
            ..Default::default()
        };
        let mut a = Vec::new();
        let mut b = Vec::new();
        write_world(&mut a, &w, &side).unwrap();
        write_world(&mut b, &w, &side).unwrap();
        assert_eq!(a, b);
        let (_, back) = parse(&a).unwrap();
        assert_eq!(back.banner_claimable.len(), 2);
        assert_eq!(back.banner_claimable[0].banner, 69);
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
                BestiaryKill {
                    credit_id: "BlueSlime".into(),
                    kills: 137,
                },
                BestiaryKill {
                    credit_id: "KingSlime".into(),
                    kills: 0,
                },
                BestiaryKill {
                    credit_id: " EyeofCthulhu".into(),
                    kills: 1,
                },
            ],
            npcs: Vec::new(),
            ore_tiers: None,
            ..Default::default()
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
            banners: vec![BannerKill {
                banner: 69,
                kills: 50,
            }],
            bestiary: vec![BestiaryKill {
                credit_id: "BlueSlime".into(),
                kills: 3,
            }],
            npcs: Vec::new(),
            ore_tiers: None,
            ..Default::default()
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
            bestiary: vec![BestiaryKill {
                credit_id: "BlueSlime".into(),
                kills: 3,
            }],
            npcs: Vec::new(),
            ore_tiers: None,
            ..Default::default()
        };
        let mut buf = Vec::new();
        write_world(&mut buf, &w, &side).unwrap();
        // Peel the v18 variants block, then cut inside the bestiary record (W4-d109).
        buf.truncate(buf.len() - v18_tail_len(&buf) - 2);
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
                BestiaryKill {
                    credit_id: "BlueSlime".into(),
                    kills: 7,
                },
                BestiaryKill {
                    credit_id: "KingSlime".into(),
                    kills: 0,
                },
            ],
            npcs: Vec::new(),
            ore_tiers: None,
            ..Default::default()
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
            ..Default::default()
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
            bestiary: vec![BestiaryKill {
                credit_id: "BlueSlime".into(),
                kills: 2,
            }],
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
            ..Default::default()
        };
        let mut buf = Vec::new();
        write_world(&mut buf, &w, &side).unwrap();
        buf[8..12].copy_from_slice(&9u32.to_le_bytes());
        let (back, back_side) = parse(&buf).unwrap();
        assert_eq!(
            back_side.bestiary, side.bestiary,
            "the bestiary still loads"
        );
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
            ..Default::default()
        };
        let mut buf = Vec::new();
        write_world(&mut buf, &w, &side).unwrap();
        // Peel the v18 variants block, then cut inside the NPC record (W4-d109).
        buf.truncate(buf.len() - v18_tail_len(&buf) - 2);
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
                NpcRecord {
                    kind: 5,
                    given_name: "A".into(),
                    x: 1.0,
                    y: 2.0,
                    homeless: true,
                    home_x: -1,
                    home_y: -1,
                },
                NpcRecord {
                    kind: 17,
                    given_name: "B".into(),
                    x: 3.0,
                    y: 4.0,
                    homeless: false,
                    home_x: 2,
                    home_y: 3,
                },
            ],
            ore_tiers: None,
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
        };
        let mut buf = Vec::new();
        write_world(&mut buf, &w, &side).unwrap();
        // Peel the v18 variants block, then cut inside the tiers (W4-d109).
        buf.truncate(buf.len() - v18_tail_len(&buf) - 3);
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
            ..Default::default()
        };
        let mut a = Vec::new();
        let mut b = Vec::new();
        write_world(&mut a, &w, &side).unwrap();
        write_world(&mut b, &w, &side).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn the_five_surface_bg_styles_round_trip() {
        // `SaveWorldFlags` writes `WorldGen.corruptBG`/`jungleBG`/`hallowBG`/
        // `crimsonBG` at `WorldFile.cs:1319-1323` and `mushroomBG` at `:1375`;
        // `LoadWorldFlags` reads them back at `:2211-2218` and `:2219-2221`.
        let mut w = World::new("Test".into(), 32, 32);
        w.corrupt_bg_style = 3;
        w.jungle_bg_style = 4;
        w.hallow_bg_style = 5;
        w.crimson_bg_style = 6;
        w.mushroom_bg_style = 2;
        let mut buf = Vec::new();
        write_world(&mut buf, &w, &WorldSide::default()).unwrap();
        let (back, _) = parse(&buf).unwrap();
        assert_eq!(back.corrupt_bg_style, 3);
        assert_eq!(back.jungle_bg_style, 4);
        assert_eq!(back.hallow_bg_style, 5);
        assert_eq!(back.crimson_bg_style, 6);
        assert_eq!(back.mushroom_bg_style, 2);
        // The style bytes resolve to the `setBG` texture-id rows.
        assert_eq!(
            crate::world::surface_bg_texture_ids(1, 3),
            Some([225, 226, 227])
        );
        assert_eq!(
            crate::world::surface_bg_texture_ids(8, 2),
            Some([234, 235, 236])
        );
    }

    // ---- W4-d109: the v18 framing-variants block (the save half of S1-A-3) ----

    /// The packed variants round-trip byte-for-byte and arm `variants_from_save`
    /// so the framing passes reuse them. Both 2-bit fields survive, including a
    /// wall variant of 3 (the lazure fourth lookup column).
    #[test]
    fn w4_d109_the_framing_variants_round_trip_in_v18() {
        let mut w = World::new("Test".into(), 16, 12);
        for y in 0..12 {
            for x in 0..16 {
                let i = w.index(x, y);
                w.set_frame_number(i, ((x + y) % 3) as u8);
                w.set_wall_frame_number(i, ((x * 2 + y) % 4) as u8); // reaches 3
            }
        }
        let mut buf = Vec::new();
        write_world(&mut buf, &w, &WorldSide::default()).unwrap();
        assert_eq!(&buf[8..12], &[18, 0, 0, 0], "the writer emits v18");
        let (back, _) = parse(&buf).unwrap();
        assert!(back.variants_from_save, "a v18 load arms the reuse seam");
        assert_eq!(back.variants, w.variants, "the packed bits round-trip");
        for y in 0..12 {
            for x in 0..16 {
                let i = back.index(x, y);
                assert_eq!(back.frame_number(i), w.frame_number(i));
                assert_eq!(back.wall_frame_number(i), w.wall_frame_number(i));
            }
        }
        // The block is dense width*height and rides the file's tail.
        assert_eq!(v18_tail_len(&buf), 16 * 12);
        assert_eq!(&buf[buf.len() - 16 * 12..], w.variants.as_slice());
    }

    /// A pre-v18 file (the v18 block truncated, the version patched to 17) loads
    /// with the `World::new` zero variants and `variants_from_save` false — the
    /// C#'s own reset semantics (the runtime bits are not in the `.wld`, so they
    /// reset and re-roll). The older tails still load.
    #[test]
    fn w4_d109_a_pre_v18_file_resets_the_variants() {
        let mut w = World::new("Test".into(), 16, 12);
        for i in 0..w.tiles.len() {
            w.set_frame_number(i, 2);
            w.set_wall_frame_number(i, 3);
        }
        w.downed_fishron = true;
        let mut buf = Vec::new();
        write_world(&mut buf, &w, &WorldSide::default()).unwrap();
        // Peel the dense v18 block and patch the version to 17.
        buf.truncate(buf.len() - v18_tail_len(&buf));
        buf[8..12].copy_from_slice(&17u32.to_le_bytes());
        let (back, _) = parse(&buf).unwrap();
        assert!(!back.variants_from_save, "a pre-v18 file re-rolls");
        assert!(back.variants.iter().all(|&v| v == 0), "the zeros of World::new");
        assert!(back.downed_fishron, "the v17 flag still loads");
    }

    /// A v18 payload cut mid-variants-block is a structural error (the dense
    /// block is self-delimiting by `width*height`), not a silent default.
    #[test]
    fn w4_d109_a_truncated_variants_block_bails() {
        let mut w = World::new("Test".into(), 16, 12);
        for i in 0..w.tiles.len() {
            w.set_frame_number(i, 1);
        }
        let mut buf = Vec::new();
        write_world(&mut buf, &w, &WorldSide::default()).unwrap();
        buf.truncate(buf.len() - 5); // inside the dense variants block
        assert!(parse(&buf).is_err());
    }

    /// The same world written twice is byte-equal — the variants block is a
    /// dense row-major array, so no iteration order can leak into the stream.
    #[test]
    fn w4_d109_the_variants_block_is_deterministic() {
        let mut w = World::new("Test".into(), 16, 12);
        for i in 0..w.tiles.len() {
            w.set_frame_number(i, (i % 3) as u8);
            w.set_wall_frame_number(i, (i % 4) as u8);
        }
        let mut a = Vec::new();
        let mut b = Vec::new();
        write_world(&mut a, &w, &WorldSide::default()).unwrap();
        write_world(&mut b, &w, &WorldSide::default()).unwrap();
        assert_eq!(a, b);
    }
}
