//! Sidecar save for the player state (`.trplayer` next to a `.trwld`).
//!
//! The world format stays tile-only; the player's position, HP and hotbar counts live in a
//! small versioned text file. Loading validates that the saved position is not inside a
//! solid tile.
//!
//! DEPRECATED-BY-SUPERSET (W4-g90 retire, research/135 §8.3/§11): this
//! branch's v7 writer/reader is the FOLD SOURCE, not the merge survivor.
//! g88's v11 superset (`a9f1c29d8`, research/103 §9 step 6) is the
//! canonical `.trplayer` writer; its v7 legacy arm loads THIS branch's
//! six-1-token-line block LOSSLESSLY (the 135 §2 recognition rule landed
//! verbatim at g88 `persist.rs:463-481`, witness
//! `g90bs_six_line_v7_loads_positionally` `:1198` — re-verified at the
//! retire probe). At the merge this writer retires and the `SavedPlayer`
//! flag fields + App restores dedup onto the superset's (103 §9 step 7);
//! the branch keeps its v7 shape until then (the standing decision).

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
    /// `Player.ateArtisanBread` (`Player.cs:2048`) — version 7 (W4-g90);
    /// older files load false (the ctor default, `Player.cs:56077`). The
    /// vanilla persists it in the `.plr` (`Serialize:54779`,
    /// `Deserialize:55245`); the port's live-session sidecar carries it so
    /// the permanent unlock survives a `.trwld` reload.
    pub ate_artisan_bread: bool,
    /// `Player.usedAegisCrystal` (`Player.cs:852`, save `:54781` / load
    /// `:55255`) — version 7's second unlock line (W4-g90 lane 7); older
    /// files load false (the ctor `:56078`).
    pub used_aegis_crystal: bool,
    /// `Player.usedArcaneCrystal` (`Player.cs:856`, `:54783`/`:55257`) —
    /// version 7 (W4-g90 lane 7); older files load false (`:56080`).
    pub used_arcane_crystal: bool,
    /// `Player.usedGalaxyPearl` (`Player.cs:858`, `:54784`/`:55258`) —
    /// version 7 (W4-g90 lane 7); older files load false (`:56081`).
    pub used_galaxy_pearl: bool,
    /// `Player.usedGummyWorm` (`Player.cs:860`, `:54785`/`:55259`) —
    /// version 7 (W4-g90 lane 7); older files load false (`:56082`).
    pub used_gummy_worm: bool,
    /// `Player.usedAmbrosia` (`Player.cs:862`, `:54786`/`:55260`) —
    /// version 7 (W4-g90 lane 7); older files load false (`:56083`).
    pub used_ambrosia: bool,
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
    // Version 7 (W4-g90): the artisan-bread flag line after the dye kinds
    // (`Player.ateArtisanBread`, `Player.cs:2048` — the permanent 5326
    // unlock, saved at `:54779` in the vanilla `.plr`).
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
    // Version 7 (W4-g90): the artisan-bread flag (0/1), then the five
    // shimmer-unlock flags in the .plr's own write order (`Player.cs:
    // 54781-54786`: crystal, arcane, pearl, gummy, ambrosia — the 5338
    // Aegis Fruit line is W4-g89's lane and joins at the merge). The g88
    // superset contract places the life-maxima pair line AHEAD of this
    // block at the merge; on this branch the flags follow the dye line.
    // DEPRECATED-BY-SUPERSET (retire): g88 v11 emits the bread flag on
    // its own line after the mana pair + the SIX-token unlock line after
    // dye (103 §2); this six-line block is the fold source its v7 arm
    // reads positionally (the module banner).
    s.push_str(if player.ate_artisan_bread { "1\n" } else { "0\n" });
    for flag in [
        player.used_aegis_crystal,
        player.used_arcane_crystal,
        player.used_galaxy_pearl,
        player.used_gummy_worm,
        player.used_ambrosia,
    ] {
        s.push_str(if flag { "1\n" } else { "0\n" });
    }
    std::fs::write(player_path(world), s)
}

pub fn load(world: &Path) -> Option<SavedPlayer> {
    let s = std::fs::read_to_string(player_path(world)).ok()?;
    let mut lines = s.lines();
    let version = lines.next()?.trim();
    let migrate = match version {
        // Version 4 adds the equipped armor line; 5 the bestiary kills; 6 the
        // dye kinds; 7 the artisan-bread flag; 3 stores vanilla `ItemID`s;
        // 1 and 2 stored the ad-hoc tile/wall/weapon/coin ranges and are
        // converted on load.
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
    // Version 7 adds the artisan-bread flag; older files load false (the
    // ctor default, `Player.cs:56077`). W4-g90 lane 7 appends the five
    // shimmer-unlock flags after it (the .plr write order `:54781-54786`
    // minus the g89-owned 5338 fruit) — a v7 file written before this
    // extension carries the bread line only, so each flag reads its own
    // optional line and defaults false when absent.
    // DEPRECATED-BY-SUPERSET (retire): this reader rejects every g88/g89
    // file BY DESIGN (135 §1) — the superset reader wins at the merge;
    // g88 v11's v7 arm loads this branch's six-line block losslessly
    // (135 §2's rule, landed verbatim — the module banner).
    let mut ate_artisan_bread = false;
    let mut used_aegis_crystal = false;
    let mut used_arcane_crystal = false;
    let mut used_galaxy_pearl = false;
    let mut used_gummy_worm = false;
    let mut used_ambrosia = false;
    if version == "7" {
        let flag_line = lines.next()?;
        ate_artisan_bread = flag_line.trim() == "1";
        for slot in [
            &mut used_aegis_crystal,
            &mut used_arcane_crystal,
            &mut used_galaxy_pearl,
            &mut used_gummy_worm,
            &mut used_ambrosia,
        ] {
            if let Some(line) = lines.next() {
                *slot = line.trim() == "1";
            }
        }
    }
    Some(SavedPlayer {
        x,
        y,
        hp,
        inventory,
        coins,
        armor,
        dye,
        kills,
        ate_artisan_bread,
        used_aegis_crystal,
        used_arcane_crystal,
        used_galaxy_pearl,
        used_gummy_worm,
        used_ambrosia,
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
        assert!(!s.ate_artisan_bread, "the fresh player never ate one");
        crate::test_world_helpers::remove_scratch(&dir);
    }

    /// W4-g90: the v7 artisan-bread line round-trips — the permanent 5326
    /// unlock (`Player.ateArtisanBread`, `Player.cs:2048`) survives the
    /// sidecar save/load.
    #[test]
    fn the_artisan_bread_flag_round_trips() {
        let world = World::new("t".into(), 32, 32);
        let mut p = Player::spawn(&world);
        p.ate_artisan_bread = true;
        let dir = crate::test_world_helpers::test_scratch("persist-bread");
        let path = dir.join("tr-persist-bread.trwld");
        save(&path, &p, [(0, 0); 50], 0, [0, 0, 0], &[0; 10], &[]).unwrap();
        let s = load(&path).expect("v7 sidecar");
        assert!(s.ate_artisan_bread, "the flag survives the round trip");
        crate::test_world_helpers::remove_scratch(&dir);
    }

    /// W4-g90: a v6 sidecar (no bread line) loads false — the ctor default
    /// (`Player.cs:56077`), the same version-gate shape as the dye line.
    #[test]
    fn version_6_loads_with_no_bread() {
        let dir = crate::test_world_helpers::test_scratch("persist-v6-bread");
        let path = dir.join("tr-persist-v6.trwld");
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
        let s = load(&path).expect("v6 sidecar");
        assert!(!s.ate_artisan_bread);
        crate::test_world_helpers::remove_scratch(&dir);
    }

    /// W4-g90 lane 7: the five shimmer unlocks round-trip through the v7
    /// flag block (`Player.cs:54781-54786` minus the g89-owned fruit).
    #[test]
    fn w4_g90_the_shimmer_unlocks_round_trip() {
        let world = World::new("t".into(), 32, 32);
        let mut p = Player::spawn(&world);
        p.used_aegis_crystal = true;
        p.used_galaxy_pearl = true;
        p.used_ambrosia = true;
        let dir = crate::test_world_helpers::test_scratch("persist-shimmer");
        let path = dir.join("tr-persist-shimmer.trwld");
        save(&path, &p, [(0, 0); 50], 0, [0, 0, 0], &[0; 10], &[]).unwrap();
        let s = load(&path).expect("the v7 sidecar");
        assert!(s.used_aegis_crystal, "the crystal persisted");
        assert!(!s.used_arcane_crystal, "the unused arcane stays false");
        assert!(s.used_galaxy_pearl);
        assert!(!s.used_gummy_worm);
        assert!(s.used_ambrosia);
        crate::test_world_helpers::remove_scratch(&dir);
    }

    /// W4-g90 lane 7: a v7 sidecar written BEFORE the unlock extension (the
    /// bread line only, the g90 first-pass shape) loads with all five
    /// unlocks absent — each flag reads its own optional line.
    #[test]
    fn w4_g90_a_bread_only_v7_loads_without_the_unlocks() {
        let dir = crate::test_world_helpers::test_scratch("persist-v7-bread-only");
        let path = dir.join("tr-persist-v7.trwld");
        let mut line = String::from("7\n1.0 2.0 100\n");
        for _ in 0..50 {
            line.push_str("0 0 ");
        }
        line.push_str("\n0\n89 80 76\n0 \n1007 0 0 0 0 1015 0 0 0 0 \n1\n");
        std::fs::write(player_path(&path), line).unwrap();
        let s = load(&path).expect("the bread-only v7 still reads");
        assert!(s.ate_artisan_bread, "the bread line parses");
        assert!(!s.used_aegis_crystal, "the absent lines load false");
        assert!(!s.used_arcane_crystal);
        assert!(!s.used_galaxy_pearl);
        assert!(!s.used_gummy_worm);
        assert!(!s.used_ambrosia);
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
