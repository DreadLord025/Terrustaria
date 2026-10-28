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
    pub inventory: [(u16, u32); 10],
}

pub fn player_path(world: &Path) -> PathBuf {
    world.with_extension("trplayer")
}

pub fn save(world: &Path, player: &Player, inventory: [(u16, u32); 10]) -> std::io::Result<()> {
    let mut s = format!("1\n{} {} {}\n", player.x, player.y, player.hp);
    for (kind, count) in inventory {
        s.push_str(&format!("{kind} {count} "));
    }
    s.push('\n');
    std::fs::write(player_path(world), s)
}

pub fn load(world: &Path) -> Option<SavedPlayer> {
    let s = std::fs::read_to_string(player_path(world)).ok()?;
    let mut lines = s.lines();
    if lines.next()?.trim() != "1" {
        return None;
    }
    let mut pos = lines.next()?.split_whitespace();
    let x = pos.next()?.parse().ok()?;
    let y = pos.next()?.parse().ok()?;
    let hp = pos.next()?.parse().ok()?;
    // Version 1 originally stored four slots; accept any number up to ten and leave
    // the remaining slots empty.
    let mut inv_line = lines.next()?.split_whitespace();
    let mut inventory = [(0u16, 0u32); 10];
    for slot in &mut inventory {
        let Some(kind) = inv_line.next() else { break };
        *slot = (kind.parse().ok()?, inv_line.next()?.parse().ok()?);
    }
    Some(SavedPlayer {
        x,
        y,
        hp,
        inventory,
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
        save(
            &path,
            &p,
            [
                (0, 12),
                (1, 3),
                (4, 9),
                (7, 1),
                (6, 0),
                (0, 0),
                (0, 0),
                (0, 0),
                (0, 0),
                (0, 0),
            ],
        )
        .unwrap();
        let s = load(&path).expect("sidecar");
        assert_eq!((s.x, s.y, s.hp), (123.5, 45.25, 73));
        assert_eq!(
            s.inventory,
            [
                (0, 12),
                (1, 3),
                (4, 9),
                (7, 1),
                (6, 0),
                (0, 0),
                (0, 0),
                (0, 0),
                (0, 0),
                (0, 0)
            ]
        );
        let _ = std::fs::remove_file(player_path(&path));
    }
}
