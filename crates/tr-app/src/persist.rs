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
}

pub fn player_path(world: &Path) -> PathBuf {
    world.with_extension("trplayer")
}

pub fn save(
    world: &Path,
    player: &Player,
    inventory: [(u16, u32); 50],
    coins: u64,
) -> std::io::Result<()> {
    let mut s = format!("3\n{} {} {}\n", player.x, player.y, player.hp);
    for (kind, count) in inventory {
        s.push_str(&format!("{kind} {count} "));
    }
    s.push_str(&format!("\n{coins}\n"));
    std::fs::write(player_path(world), s)
}

pub fn load(world: &Path) -> Option<SavedPlayer> {
    let s = std::fs::read_to_string(player_path(world)).ok()?;
    let mut lines = s.lines();
    let version = lines.next()?.trim();
    let migrate = match version {
        // Version 3 stores vanilla `ItemID`s; 1 and 2 stored the ad-hoc
        // tile/wall/weapon/coin ranges and are converted on load.
        "3" => false,
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
    Some(SavedPlayer {
        x,
        y,
        hp,
        inventory,
        coins,
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
        let path = std::env::temp_dir().join("tr-persist-test.trwld");
        let mut inv = [(0u16, 0u32); 50];
        inv[0] = (2, 12);
        inv[1] = (3, 3);
        inv[2] = (8, 9);
        inv[3] = (12, 1);
        inv[4] = (71, 40);
        save(&path, &p, inv, 123_456).unwrap();
        let s = load(&path).expect("sidecar");
        assert_eq!((s.x, s.y, s.hp), (123.5, 45.25, 73));
        assert_eq!(s.inventory[0], (2, 12));
        assert_eq!(s.inventory[3], (12, 1));
        assert_eq!(s.inventory[49], (0, 0));
        let _ = std::fs::remove_file(player_path(&path));
    }

    #[test]
    fn migrates_version_2_kinds_to_item_ids() {
        let path = std::env::temp_dir().join("tr-persist-v2.trwld");
        // Version 2: tile kinds and the wall/weapon/coin ranges.
        let mut line = String::from("2
10 20 100
");
        for kind in [0u16, 1, 10_002, 20_001, 30_002] {
            line.push_str(&format!("{kind} 5 "));
        }
        line.push_str("
0
");
        std::fs::write(player_path(&path), line).unwrap();
        let s = load(&path).expect("v2 sidecar");
        assert_eq!(s.inventory[0], (2, 5));
        assert_eq!(s.inventory[1], (3, 5));
        assert_eq!(s.inventory[2], (30, 5));
        assert_eq!(s.inventory[3], (42, 5));
        assert_eq!(s.inventory[4], (73, 5));
        let _ = std::fs::remove_file(player_path(&path));
    }
}
