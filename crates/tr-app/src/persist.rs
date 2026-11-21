//! Sidecar save for the player state (`.trplayer` next to a `.trwld`).
//!
//! The world format stays tile-only; the player's position, HP and hotbar counts live in a
//! small versioned text file. Loading validates that the saved position is not inside a
//! solid tile.

use std::path::{Path, PathBuf};

use crate::player::Player;

pub struct SavedPlayer {
    pub x: f32,
    pub y: f32,
    pub hp: i32,
    pub inventory: [(u16, u32); 50],
    pub coins: u64,
    /// `Player.armor[0..3]` item ids — head, body, legs (0 = empty). Version 4
    /// of the sidecar; older files load with all three empty.
    pub armor: [u16; 3],
    /// `Player.dye[0..10]` item ids (0 = empty) — version 6; older files load
    /// with all ten empty. The vanilla writes the dye array straight after
    /// the armor, `type, prefix, favorited` per slot without a stack
    /// (`Player.cs:54818-54822`); the port keeps the kinds (the slot model
    /// carries no prefix/favorited) and restores each as a single stack.
    pub dye: [u16; 10],
    /// `Main.BestiaryTracker.Kills`' pairs — version 5; older files load
    /// with none. Vanilla keeps the same pairs in world-file section 11
    /// (`NPCKillsTracker.Save`, M6-SAVE-1's scope).
    pub kills: Vec<(String, i32)>,
    /// `Player.unlockedBiomeTorches` (`Player.cs:2046`, save `:54777` /
    /// load `:55241`) — version 7 of the sidecar (W4-f33); older files load
    /// with the unlock absent. The Torch God's Favor permanent biome-torch
    /// unlock survives world reloads.
    pub unlocked_biome_torches: bool,
    /// `Player.UsingBiomeTorches` (`Player.cs:3950-3963`, save `:54778` /
    /// load `:55242`) — the builder-toggle half of the pair (version 7).
    pub using_biome_torches: bool,
}

pub fn player_path(world: &Path) -> PathBuf {
    world.with_extension("trplayer")
}

/// The explored-map sidecar (`<world>.map`) next to the world file. Vanilla names
/// it `<worldName>.map` under the player directory (`MapHelper.cs:3038`); this
/// engine keeps it beside the `.trwld` like the player sidecar.
pub fn map_path(world: &Path) -> PathBuf {
    world.with_extension("map")
}

pub fn save(
    world: &Path,
    player: &Player,
    inventory: [(u16, u32); 50],
    coins: u64,
    armor: [u16; 3],
    dye: &[u16; 10],
    kills: &[(String, i32)],
) -> std::io::Result<()> {
    let mut s = format!("7\n{} {} {}\n", player.x, player.y, player.hp);
    for (kind, count) in inventory {
        s.push_str(&format!("{kind} {count} "));
    }
    s.push_str(&format!("\n{coins}\n"));
    // The equipped armor kinds (head/body/legs) on their own line.
    s.push_str(&format!("{} {} {}\n", armor[0], armor[1], armor[2]));
    // The bestiary kill pairs (`NPCKillsTracker.Save`'s `count (key value)*`
    // walk) on their own line.
    s.push_str(&format!("{} ", kills.len()));
    for (id, count) in kills {
        s.push_str(&format!("{id} {count} "));
    }
    s.push_str("\n");
    // Version 6: the dye slot kinds on their own line (the vanilla order —
    // dye directly after armor, `Player.cs:54818-54822`).
    for kind in dye {
        s.push_str(&format!("{kind} "));
    }
    s.push_str("\n");
    // Version 7 (W4-f33): the Torch God's Favor biome-torch pair on their own
    // line — `unlockedBiomeTorches` / `UsingBiomeTorches` (`Player.cs:54777-54778`),
    // 0/1 each.
    s.push_str(&format!(
        "{} {}\n",
        u8::from(player.unlocked_biome_torches),
        u8::from(player.using_biome_torches())
    ));
    std::fs::write(player_path(world), s)
}

pub fn load(world: &Path) -> Option<SavedPlayer> {
    let s = std::fs::read_to_string(player_path(world)).ok()?;
    let mut lines = s.lines();
    let version = lines.next()?.trim();
    let migrate = match version {
        // Version 4 adds the equipped armor line; 5 the bestiary kills; 6 the
        // dye kinds; 7 the Torch God biome-torch pair (W4-f33); 3 stores
        // vanilla `ItemID`s; 1 and 2 stored the ad-hoc tile/wall/weapon/coin
        // ranges and are converted on load.
        "3" | "4" | "5" | "6" | "7" => false,
        "1" | "2" => true,
        _ => return None,
    };
    let mut pos = lines.next()?.split_whitespace();
    let x = pos.next()?.parse().ok()?;
    let y = pos.next()?.parse().ok()?;
    let hp = pos.next()?.parse().ok()?;
    // Version 1 stored up to ten hotbar slots; version 2 stores all fifty. Accept
    // any number up to the version's cap and leave the remaining slots empty.
    let mut inv_line = lines.next()?.split_whitespace();
    let mut inventory = [(0u16, 0u32); 50];
    for slot in &mut inventory {
        let Some(kind) = inv_line.next() else { break };
        let mut kind: u16 = kind.parse().ok()?;
        if migrate {
            kind = crate::registry::migrate_kind(kind);
        }
        *slot = (kind, inv_line.next()?.parse().ok()?);
    }
    // The coin line was added after the original format; older files default to 0.
    let coins = lines
        .next()
        .and_then(|l| l.trim().parse::<u64>().ok())
        .unwrap_or(0);
    // The armor line is version 4+; older files load with every slot empty.
    let mut armor = [0u16; 3];
    if matches!(version, "4" | "5" | "6" | "7") {
        let mut armor_line = lines.next()?.split_whitespace();
        for slot in &mut armor {
            *slot = armor_line.next()?.parse().ok()?;
        }
    }
    // Version 5 adds the bestiary kill pairs.
    let mut kills = Vec::new();
    if matches!(version, "5" | "6" | "7") {
        let kill_line = lines.next()?;
        let mut parts = kill_line.split_whitespace();
        let count: usize = parts.next().and_then(|c| c.parse().ok()).unwrap_or(0);
        for _ in 0..count {
            let Some(id) = parts.next() else { break };
            let Some(value) = parts.next().and_then(|v| v.parse().ok()) else {
                break;
            };
            kills.push((id.to_string(), value));
        }
    }
    // Version 6 adds the dye slot kinds; older files load with every slot
    // empty.
    let mut dye = [0u16; 10];
    if matches!(version, "6" | "7") {
        let dye_line = lines.next()?;
        for (slot, kind) in dye.iter_mut().zip(dye_line.split_whitespace()) {
            *slot = kind.parse().ok()?;
        }
    }
    // Version 7 (W4-f33) adds the Torch God biome-torch pair
    // (`Player.unlockedBiomeTorches` / `UsingBiomeTorches`, `Player.cs:55241-55242`);
    // older files load with the unlock absent.
    let (unlocked_biome_torches, using_biome_torches) = if version == "7" {
        let line = lines.next()?;
        let mut parts = line.split_whitespace();
        let unlocked = parts.next().is_some_and(|v| v == "1");
        let using = parts.next().is_some_and(|v| v == "1");
        (unlocked, using)
    } else {
        (false, false)
    };
    Some(SavedPlayer {
        x,
        y,
        hp,
        inventory,
        coins,
        armor,
        dye,
        kills,
        unlocked_biome_torches,
        using_biome_torches,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::World;

    #[test]
    fn round_trips_player_state() {
        let world = World::new("t".into(), 32, 32);
        let mut p = Player::spawn(&world);
        p.x = 123.5;
        p.y = 45.25;
        p.hp = 73;
        let dir = crate::test_world_helpers::test_scratch("persist-player-state");
        let path = dir.join("tr-persist-test.trwld");
        let mut inv = [(0u16, 0u32); 50];
        inv[0] = (2, 12);
        inv[1] = (3, 3);
        inv[2] = (8, 9);
        inv[3] = (12, 1);
        inv[4] = (71, 40);
        let mut dye = [0u16; 10];
        dye[0] = 1007;
        dye[5] = 1015;
        save(
            &path,
            &p,
            inv,
            123_456,
            [89, 80, 76],
            &dye,
            &[("BlueSlime".into(), 7)],
        )
        .unwrap();
        let s = load(&path).expect("sidecar");
        assert_eq!((s.x, s.y, s.hp), (123.5, 45.25, 73));
        assert_eq!(s.inventory[0], (2, 12));
        assert_eq!(s.inventory[3], (12, 1));
        assert_eq!(s.inventory[49], (0, 0));
        assert_eq!(s.armor, [89, 80, 76]);
        assert_eq!(s.dye, dye);
        crate::test_world_helpers::remove_scratch(&dir);
    }

    /// W4-f33 — the Torch God's Favor biome-torch pair round-trips the v7
    /// sidecar line (`Player.unlockedBiomeTorches` / `UsingBiomeTorches`,
    /// save `Player.cs:54777-54778` / load `:55241-55242`): the permanent
    /// unlock survives the world reload, so the biome torches resume with
    /// the character.
    #[test]
    fn w4_f33_the_biome_torch_pair_round_trips_the_sidecar() {
        let world = World::new("t".into(), 32, 32);
        let mut p = Player::spawn(&world);
        p.unlocked_biome_torches = true;
        p.set_using_biome_torches(true);
        let dir = crate::test_world_helpers::test_scratch("persist-torch-favor");
        let path = dir.join("tr-persist-torch-favor.trwld");
        save(&path, &p, [(0, 0); 50], 0, [0, 0, 0], &[0; 10], &[]).unwrap();
        let s = load(&path).expect("the v7 sidecar");
        assert!(s.unlocked_biome_torches, "the unlock persisted");
        assert!(s.using_biome_torches, "the toggle persisted");
        // The fresh player loads false.
        let fresh = Player::spawn(&world);
        save(&path, &fresh, [(0, 0); 50], 0, [0, 0, 0], &[0; 10], &[]).unwrap();
        let s = load(&path).expect("the v7 sidecar");
        assert!(!s.unlocked_biome_torches);
        assert!(!s.using_biome_torches);
        crate::test_world_helpers::remove_scratch(&dir);
    }

    /// W4-f33 — a v6 sidecar (no biome-torch line) loads with the Torch God
    /// unlock absent; the version gate keeps the old files readable.
    #[test]
    fn w4_f33_version_6_loads_without_the_torch_unlock() {
        let dir = crate::test_world_helpers::test_scratch("persist-v6-torch");
        let path = dir.join("tr-persist-v6-torch.trwld");
        let mut line = String::from("6\n1.0 2.0 100\n");
        for _ in 0..50 {
            line.push_str("0 0 ");
        }
        line.push_str("\n0\n89 80 76\n1 (BlueSlime 7) \n");
        for _ in 0..10 {
            line.push_str("0 ");
        }
        line.push('\n');
        std::fs::write(player_path(&path), line).unwrap();
        let s = load(&path).expect("the v6 sidecar");
        assert_eq!(s.armor, [89, 80, 76]);
        assert!(!s.unlocked_biome_torches, "the unlock is absent pre-v7");
        assert!(!s.using_biome_torches);
        crate::test_world_helpers::remove_scratch(&dir);
    }

    /// W4-f33 — the g90 v7 collision tolerance: W4-g90's pushed branch writes
    /// a v7 line with ONE token (the artisan-bread flag). f33's reader is
    /// token-count tolerant — a 1-token v7 line loads the torch pair false
    /// (the correct default for a character that never used the favor), no
    /// error. The merge keeper reconciles the two v7 shapes into a v8 line
    /// (ANALYSIS "The sidecar version-7 collision").
    #[test]
    fn w4_f33_a_one_token_v7_line_loads_the_torch_pair_false() {
        let dir = crate::test_world_helpers::test_scratch("persist-v7-g90");
        let path = dir.join("tr-persist-v7-g90.trwld");
        let mut line = String::from("7\n1.0 2.0 100\n");
        for _ in 0..50 {
            line.push_str("0 0 ");
        }
        line.push_str("\n0\n89 80 76\n1 (BlueSlime 7) \n");
        for _ in 0..10 {
            line.push_str("0 ");
        }
        // The g90 bread flag (one token), NOT the torch pair.
        line.push_str("\n1\n");
        std::fs::write(player_path(&path), line).unwrap();
        let s = load(&path).expect("the g90 v7 sidecar loads");
        assert!(!s.unlocked_biome_torches, "no torch unlock in a bread-only file");
        assert!(!s.using_biome_torches);
        crate::test_world_helpers::remove_scratch(&dir);
    }

    #[test]
    fn version_3_loads_with_empty_armor() {
        let dir = crate::test_world_helpers::test_scratch("persist-v3");
        let path = dir.join("tr-persist-v3.trwld");
        let mut line = String::from("3\n1.0 2.0 100\n");
        for _ in 0..50 {
            line.push_str("0 0 ");
        }
        line.push_str("\n0\n");
        std::fs::write(player_path(&path), line).unwrap();
        let s = load(&path).expect("v3 sidecar");
        assert_eq!(s.armor, [0, 0, 0]);
        crate::test_world_helpers::remove_scratch(&dir);
    }

    /// DW-4.2 (dirty): a v5 sidecar (no dye line) loads with every dye slot
    /// empty — the version-6 dye read is gated on the file version.
    #[test]
    fn version_5_loads_with_empty_dye() {
        let dir = crate::test_world_helpers::test_scratch("persist-v5");
        let path = dir.join("tr-persist-v5.trwld");
        let mut line = String::from("5\n1.0 2.0 100\n");
        for _ in 0..50 {
            line.push_str("0 0 ");
        }
        line.push_str("\n0\n89 80 76\n1 (BlueSlime 7) \n");
        std::fs::write(player_path(&path), line).unwrap();
        let s = load(&path).expect("v5 sidecar");
        assert_eq!(s.armor, [89, 80, 76]);
        assert_eq!(s.dye, [0; 10]);
        crate::test_world_helpers::remove_scratch(&dir);
    }

    /// The same gate one version down: a v4 sidecar (armor, no kills, no dye).
    #[test]
    fn version_4_loads_with_empty_dye() {
        let dir = crate::test_world_helpers::test_scratch("persist-v4");
        let path = dir.join("tr-persist-v4.trwld");
        let mut line = String::from("4\n1.0 2.0 100\n");
        for _ in 0..50 {
            line.push_str("0 0 ");
        }
        line.push_str("\n0\n91 82 78\n");
        std::fs::write(player_path(&path), line).unwrap();
        let s = load(&path).expect("v4 sidecar");
        assert_eq!(s.armor, [91, 82, 78]);
        assert_eq!(s.dye, [0; 10]);
        crate::test_world_helpers::remove_scratch(&dir);
    }

    #[test]
    fn migrates_version_2_kinds_to_item_ids() {
        let dir = crate::test_world_helpers::test_scratch("persist-v2");
        let path = dir.join("tr-persist-v2.trwld");
        // Version 2: tile kinds and the wall/weapon/coin ranges. (The fixture
        // previously glued the version and position into one line — "210 20
        // 100" — so the version match failed and the test could never load.)
        let mut line = String::from("2\n10 20 100\n");
        for kind in [0u16, 1, 10_002, 20_001, 30_002] {
            line.push_str(&format!("{kind} 5 "));
        }
        line.push_str(
            "
0
",
        );
        std::fs::write(player_path(&path), line).unwrap();
        let s = load(&path).expect("v2 sidecar");
        assert_eq!(s.inventory[0], (2, 5));
        assert_eq!(s.inventory[1], (3, 5));
        assert_eq!(s.inventory[2], (30, 5));
        assert_eq!(s.inventory[3], (42, 5));
        assert_eq!(s.inventory[4], (73, 5));
        assert_eq!(s.armor, [0, 0, 0]);
        crate::test_world_helpers::remove_scratch(&dir);
    }

    /// Task 9: the `.map` sidecar shares the world's stem, one directory up from
    /// the player sidecar.
    #[test]
    fn map_path_sits_next_to_the_world() {
        assert_eq!(map_path(Path::new("a/b.trwld")), PathBuf::from("a/b.map"));
    }
}
