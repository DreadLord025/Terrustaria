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
    /// `Player.usedAegisFruit` (`Player.cs:854`, save `:54782` / load
    /// `:55256`) — version 7 of the sidecar (W4-g89); older files load
    /// with the unlock absent. The permanent +4 defense
    /// (`UpdatePermanentBoosters` `:12853-12856`) survives world reloads.
    pub used_aegis_fruit: bool,
    /// `Player.usedGummyWorm` (`Player.cs:2074`, save `:54785` / load
    /// `:55259`) — version 8 of the sidecar (W4-g89), the SECOND token on
    /// the unlock line. The permanent +3 fishing skill (`:12857-12860`).
    pub used_gummy_worm: bool,
    /// `Player.usedAmbrosia` (`Player.cs:862`, save `:54786` / load
    /// `:55260`) — version 9 of the sidecar (W4-g89c), the THIRD token on
    /// the unlock line. The permanent mining/placement speed
    /// (`:12861-12866`).
    pub used_ambrosia: bool,
    /// `Player.usedAegisCrystal` (`Player.cs:852`, save `:54781` / load
    /// `:55255`) — version 10 of the sidecar (W4-g89i), the FOURTH token on
    /// the unlock line (the binding g88-v10 order, research/138 §6:
    /// crystal=4 / arcane=5 / pearl=6). The permanent +0.2 life-regen ramp
    /// (`:19113-19116`). Older files load false (ctor `:56078`). The
    /// arcane/pearl tokens 5/6 are written as `0` placeholders here — their
    /// Player fields + consumers stay the a6/g90 lanes' (research/138 §2);
    /// the g88 v10 superset reader hydrates them at the merge.
    pub used_aegis_crystal: bool,
}

pub fn player_path(world: &Path) -> PathBuf {
    world.with_extension("trplayer")
}

/// W4-g56 — the void-vault favorites sidecar: `<world>.trvault` beside the
/// `.trwld`/`.trplayer`. The C# rides `bank4.item[i].favorited` in the `.plr`
/// (`Player.cs:54856-54862`, the favourited bool at release >= 255,
/// `Deserialize:55495`; research/99 row 47) plus the `voidVaultInfo` BitsByte
/// (`:54863`, gate 199, row 48). The port's `.plr` writer (`player_file.rs`)
/// emits those rows at fresh-ctor defaults and its reader drops them — the
/// full bank4 byte-shape is W4-f44's lane (`task/W4-f44-save-plr-2`, the
/// `PlayerFile.bank4` model + the verify-sweep), and the live-session
/// `.trplayer` superset is W4-g88's lane (v7-v11). Until those land the vault
/// favorites persist in this self-contained sidecar (the `.trfav` marker
/// precedent, `player_file.rs:1058`): a 40-char `0`/`1` string, one per bank4
/// slot, index 0 first. The `.plr` contract is documented in
/// `docs/ANALYSIS.md` ("W4-g56 void-vault favorites sidecar contract") and
/// `docs/research/274-void-family-bible.md` §1 (the RETIRE verdict): the
/// sidecar COEXISTS through the f44/g88 folds — f44's `.plr` writers emit
/// bank4 at fresh defaults (no live-session re-save on any tip), so this
/// write retires only when character-load hydration AND the exit-save both
/// land; restore `.plr` first, sidecar second (f44 research/132 §2c).
pub fn vault_favorites_path(world: &Path) -> PathBuf {
    world.with_extension("trvault")
}

/// Write the vault favorites sidecar (`Player.vault_favorites`, the 40 bank4
/// favourited bools). A missing/empty vault writes all-`0` (the fresh default).
pub fn save_vault_favorites(world: &Path, favorites: &[bool; crate::bank::BANK_SLOTS]) -> std::io::Result<()> {
    let mut s = String::with_capacity(crate::bank::BANK_SLOTS);
    for f in favorites {
        s.push(if *f { '1' } else { '0' });
    }
    std::fs::write(vault_favorites_path(world), s)
}

/// Read the vault favorites sidecar; `None` when absent (a fresh world keeps
/// the all-false default). A short/garbled line loads the slots it can and
/// defaults the rest (the back-compat shape the `.trplayer` reader uses).
pub fn load_vault_favorites(world: &Path) -> Option<[bool; crate::bank::BANK_SLOTS]> {
    let s = std::fs::read_to_string(vault_favorites_path(world)).ok()?;
    let mut out = [false; crate::bank::BANK_SLOTS];
    for (slot, ch) in out.iter_mut().zip(s.trim().chars()) {
        *slot = ch == '1';
    }
    Some(out)
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
    let mut s = format!("10\n{} {} {}\n", player.x, player.y, player.hp);
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
    // Version 7 (W4-g89): the permanent shimmer unlocks on their own line
    // — `Player.usedAegisFruit` (`Player.cs:54782`), 0/1. Version 8 appends
    // `Player.usedGummyWorm` (`:54785`); version 9 (W4-g89c) appends
    // `Player.usedAmbrosia` (`:54786`) as the third token. Version 10
    // (W4-g89i) widens to the SIX-token g88-v10 unlock line (research/138
    // §6, the binding written order): tokens 4/5/6 = the crystal trio in
    // the vanilla `.plr` relative order (`:54781` crystal / `:54783` arcane
    // / `:54784` pearl). This branch carries only the crystal flag (token
    // 4); arcane/pearl write `0` placeholders (their fields are a6/g90's).
    s.push_str(&format!(
        "{} {} {} {} 0 0\n",
        u8::from(player.used_aegis_fruit),
        u8::from(player.used_gummy_worm),
        u8::from(player.used_ambrosia),
        u8::from(player.used_aegis_crystal)
    ));
    std::fs::write(player_path(world), s)
}

pub fn load(world: &Path) -> Option<SavedPlayer> {
    let s = std::fs::read_to_string(player_path(world)).ok()?;
    let mut lines = s.lines();
    let version = lines.next()?.trim();
    let migrate = match version {
        // Version 4 adds the equipped armor line; 5 the bestiary kills; 6 the
        // dye kinds; 7 the permanent shimmer unlocks (W4-g89); 8 the Gummy
        // Worm token on that line; 9 the Ambrosia token (W4-g89c); 10 the
        // crystal trio tokens 4/5/6 (W4-g89i, the g88-v10 binding order);
        // 3 stores vanilla `ItemID`s; 1 and 2 stored the ad-hoc
        // tile/wall/weapon/coin ranges and are converted on load.
        "3" | "4" | "5" | "6" | "7" | "8" | "9" | "10" => false,
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
    if matches!(version, "4" | "5" | "6" | "7" | "8" | "9" | "10") {
        let mut armor_line = lines.next()?.split_whitespace();
        for slot in &mut armor {
            *slot = armor_line.next()?.parse().ok()?;
        }
    }
    // Version 5 adds the bestiary kill pairs.
    let mut kills = Vec::new();
    if matches!(version, "5" | "6" | "7" | "8" | "9" | "10") {
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
    if matches!(version, "6" | "7" | "8" | "9" | "10") {
        let dye_line = lines.next()?;
        for (slot, kind) in dye.iter_mut().zip(dye_line.split_whitespace()) {
            *slot = kind.parse().ok()?;
        }
    }
    // Version 7 (W4-g89) adds the permanent shimmer unlock flags
    // (`Player.usedAegisFruit`, `Player.cs:55256`); version 8 appends
    // `usedGummyWorm` (`:55259`), version 9 `usedAmbrosia` (`:55260`),
    // version 10 (W4-g89i) the crystal trio tokens 4/5/6 (the g88-v10
    // binding order, research/138 §6 — this branch reads token 4, the
    // crystal; tokens 5/6 are the a6/g90 lanes'). Older files load with
    // the unlocks absent; a v7 line has one token, a v8 line two, a v9
    // line three (the crystal back-reads false).
    let (used_aegis_fruit, used_gummy_worm, used_ambrosia, used_aegis_crystal) =
        if matches!(version, "7" | "8" | "9" | "10") {
            let line = lines.next()?;
            let mut tokens = line.split_whitespace();
            let fruit = tokens.next().is_some_and(|t| t == "1");
            let worm = tokens.next().is_some_and(|t| t == "1");
            let ambrosia = tokens.next().is_some_and(|t| t == "1");
            let crystal = tokens.next().is_some_and(|t| t == "1");
            (fruit, worm, ambrosia, crystal)
        } else {
            (false, false, false, false)
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
        used_aegis_fruit,
        used_gummy_worm,
        used_ambrosia,
        used_aegis_crystal,
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

    /// W4-g89 — the permanent Aegis Fruit unlock round-trips through the
    /// sidecar's unlock line (v7 wrote it; save `Player.cs:54782` / load
    /// `:55256`): the flag survives the world reload, so the +4 booster
    /// (`:12853-12856`) resumes with the character.
    #[test]
    fn w4_g89_the_aegis_fruit_unlock_round_trips() {
        let world = World::new("t".into(), 32, 32);
        let mut p = Player::spawn(&world);
        p.used_aegis_fruit = true;
        let dir = crate::test_world_helpers::test_scratch("persist-aegis");
        let path = dir.join("tr-persist-aegis.trwld");
        save(&path, &p, [(0, 0); 50], 0, [0, 0, 0], &[0; 10], &[]).unwrap();
        let s = load(&path).expect("the sidecar");
        assert!(s.used_aegis_fruit, "the unlock persisted");
        // The fresh player loads false.
        let fresh = Player::spawn(&world);
        save(&path, &fresh, [(0, 0); 50], 0, [0, 0, 0], &[0; 10], &[]).unwrap();
        assert!(!load(&path).expect("the sidecar").used_aegis_fruit);
        crate::test_world_helpers::remove_scratch(&dir);
    }

    /// W4-g89 — the Gummy Worm unlock rides the v8 unlock line's SECOND
    /// token (`Player.usedGummyWorm`, save `Player.cs:54785` / load
    /// `:55259`); a v7 file (one token) loads with the worm absent while
    /// the fruit half still reads.
    #[test]
    fn w4_g89_the_gummy_worm_unlock_round_trips_and_v7_back_reads() {
        let world = World::new("t".into(), 32, 32);
        let mut p = Player::spawn(&world);
        p.used_gummy_worm = true;
        let dir = crate::test_world_helpers::test_scratch("persist-gummy");
        let path = dir.join("tr-persist-gummy.trwld");
        save(&path, &p, [(0, 0); 50], 0, [0, 0, 0], &[0; 10], &[]).unwrap();
        let s = load(&path).expect("the v8 sidecar");
        assert!(s.used_gummy_worm, "the unlock persisted");
        assert!(!s.used_aegis_fruit, "the first token stayed false");
        // Both flags round-trip independently.
        p.used_aegis_fruit = true;
        save(&path, &p, [(0, 0); 50], 0, [0, 0, 0], &[0; 10], &[]).unwrap();
        let s = load(&path).expect("the v8 sidecar");
        assert!(s.used_aegis_fruit && s.used_gummy_worm);
        // A v7 file: the unlock line carries only the fruit token.
        let mut line = String::from("7\n1.0 2.0 100\n");
        for _ in 0..50 {
            line.push_str("0 0 ");
        }
        line.push_str("\n0\n89 80 76\n0 \n1007 0 0 0 0 1015 0 0 0 0 \n1\n");
        std::fs::write(player_path(&path), line).unwrap();
        let s = load(&path).expect("the v7 sidecar still reads");
        assert!(s.used_aegis_fruit, "the v7 fruit token reads");
        assert!(!s.used_gummy_worm, "a v7 line carries no worm token");
        crate::test_world_helpers::remove_scratch(&dir);
    }

    /// W4-g89c — the Ambrosia unlock rides the v9 unlock line's THIRD
    /// token (`Player.usedAmbrosia`, save `Player.cs:54786` / load
    /// `:55260`); a v8 two-token line loads with the ambrosia absent while
    /// both earlier halves still read.
    #[test]
    fn w4_g89c_the_ambrosia_unlock_round_trips_and_v8_back_reads() {
        let world = World::new("t".into(), 32, 32);
        let mut p = Player::spawn(&world);
        p.used_ambrosia = true;
        let dir = crate::test_world_helpers::test_scratch("persist-ambrosia");
        let path = dir.join("tr-persist-ambrosia.trwld");
        save(&path, &p, [(0, 0); 50], 0, [0, 0, 0], &[0; 10], &[]).unwrap();
        let s = load(&path).expect("the v9 sidecar");
        assert!(s.used_ambrosia, "the unlock persisted");
        assert!(!s.used_aegis_fruit && !s.used_gummy_worm);
        // All three flags round-trip independently.
        p.used_aegis_fruit = true;
        p.used_gummy_worm = true;
        save(&path, &p, [(0, 0); 50], 0, [0, 0, 0], &[0; 10], &[]).unwrap();
        let s = load(&path).expect("the v9 sidecar");
        assert!(s.used_aegis_fruit && s.used_gummy_worm && s.used_ambrosia);
        // A v8 file: the unlock line carries two tokens.
        let mut line = String::from("8\n1.0 2.0 100\n");
        for _ in 0..50 {
            line.push_str("0 0 ");
        }
        line.push_str("\n0\n89 80 76\n0 \n1007 0 0 0 0 1015 0 0 0 0 \n0 1\n");
        std::fs::write(player_path(&path), line).unwrap();
        let s = load(&path).expect("the v8 sidecar still reads");
        assert!(!s.used_aegis_fruit && s.used_gummy_worm, "the v8 tokens read");
        assert!(!s.used_ambrosia, "a v8 line carries no ambrosia token");
        crate::test_world_helpers::remove_scratch(&dir);
    }

    /// W4-g89i — the Aegis Crystal unlock rides the v10 unlock line's
    /// FOURTH token (`Player.usedAegisCrystal`, save `Player.cs:54781` /
    /// load `:55255`; the binding g88-v10 order crystal=4/arcane=5/pearl=6,
    /// research/138 §6); a v9 three-token line loads with the crystal
    /// absent while all three earlier halves still read.
    #[test]
    fn w4_g89i_the_aegis_crystal_unlock_round_trips_and_v9_back_reads() {
        let world = World::new("t".into(), 32, 32);
        let mut p = Player::spawn(&world);
        p.used_aegis_crystal = true;
        let dir = crate::test_world_helpers::test_scratch("persist-aegis-crystal");
        let path = dir.join("tr-persist-aegis-crystal.trwld");
        save(&path, &p, [(0, 0); 50], 0, [0, 0, 0], &[0; 10], &[]).unwrap();
        let s = load(&path).expect("the v10 sidecar");
        assert!(s.used_aegis_crystal, "the unlock persisted (token 4)");
        assert!(!s.used_aegis_fruit && !s.used_gummy_worm && !s.used_ambrosia);
        // All four flags round-trip independently.
        p.used_aegis_fruit = true;
        p.used_gummy_worm = true;
        p.used_ambrosia = true;
        save(&path, &p, [(0, 0); 50], 0, [0, 0, 0], &[0; 10], &[]).unwrap();
        let s = load(&path).expect("the v10 sidecar");
        assert!(
            s.used_aegis_fruit && s.used_gummy_worm && s.used_ambrosia && s.used_aegis_crystal
        );
        // A v9 file: the unlock line carries three tokens (no crystal).
        let mut line = String::from("9\n1.0 2.0 100\n");
        for _ in 0..50 {
            line.push_str("0 0 ");
        }
        line.push_str("\n0\n89 80 76\n0 \n1007 0 0 0 0 1015 0 0 0 0 \n1 1 1\n");
        std::fs::write(player_path(&path), line).unwrap();
        let s = load(&path).expect("the v9 sidecar still reads");
        assert!(
            s.used_aegis_fruit && s.used_gummy_worm && s.used_ambrosia,
            "the v9 tokens read"
        );
        assert!(!s.used_aegis_crystal, "a v9 line carries no crystal token");
        crate::test_world_helpers::remove_scratch(&dir);
    }

    /// W4-g89 — a v6 sidecar (no unlock line) loads with the Aegis Fruit
    /// unlock absent; the version gate keeps the old files readable.
    #[test]
    fn w4_g89_version_6_loads_without_the_unlock() {
        let dir = crate::test_world_helpers::test_scratch("persist-v6-aegis");
        let path = dir.join("tr-persist-v6.trwld");
        let mut line = String::from("6\n1.0 2.0 100\n");
        for _ in 0..50 {
            line.push_str("0 0 ");
        }
        line.push_str("\n0\n89 80 76\n0 \n1007 0 0 0 0 1015 0 0 0 0 \n");
        std::fs::write(player_path(&path), line).unwrap();
        let s = load(&path).expect("the v6 sidecar still reads");
        assert!(!s.used_aegis_fruit, "older files carry no unlock");
        assert!(!s.used_gummy_worm, "older files carry no worm either");
        assert!(!s.used_ambrosia, "nor ambrosia");
        assert_eq!(s.dye[0], 1007, "the v6 dye line still parses");
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

    /// W4-g56 — the void-vault favorites sidecar round-trips: the 40 bank4
    /// favourited bools write to `<world>.trvault` and read back; a fresh world
    /// (no sidecar) loads `None` (the all-false default). The `.plr` bank4 rows
    /// are f44's lane — this is the interim carrier (ANALYSIS contract).
    #[test]
    fn w4_g56_the_vault_favorites_sidecar_round_trips() {
        let dir = crate::test_world_helpers::test_scratch("persist-vault-fav");
        let path = dir.join("tr-vault-fav.trwld");
        assert_eq!(vault_favorites_path(&path), dir.join("tr-vault-fav.trvault"));
        // No sidecar yet: the load answers None (fresh default).
        assert!(load_vault_favorites(&path).is_none());
        let mut favs = [false; crate::bank::BANK_SLOTS];
        favs[0] = true;
        favs[7] = true;
        favs[39] = true;
        save_vault_favorites(&path, &favs).unwrap();
        let back = load_vault_favorites(&path).expect("the sidecar reads");
        assert_eq!(back, favs, "the 40 bools round-trip");
        // An all-false vault writes a readable all-0 line.
        save_vault_favorites(&path, &[false; crate::bank::BANK_SLOTS]).unwrap();
        assert_eq!(
            load_vault_favorites(&path).unwrap(),
            [false; crate::bank::BANK_SLOTS]
        );
        crate::test_world_helpers::remove_scratch(&dir);
    }
}
