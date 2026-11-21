//! Sidecar save for the player state (`.trplayer` next to a `.trwld`).
//!
//! The world format stays tile-only; the player's position, HP and hotbar counts live in a
//! small versioned text file. Loading validates that the saved position is not inside a
//! solid tile.
//!
//! ## The v7 line contract (W4-g88 ⨉ W4-g89 reconciliation)
//!
//! Both the aegis lane (g89, `920014cb`) and the life-fruit lane (g88,
//! `f1eb9b60`) independently bumped the sidecar to version 7, each
//! appending ONE line after the dye line. The reconciled SUPERSET v7
//! order (this branch's writer) is:
//!
//! 1. the aegis flag line — `usedAegisFruit` 0/1 (g89's position, ONE token),
//! 2. the life-maxima pair — `statLifeMax statLifeMax2` (TWO tokens),
//! 3. the mana-maxima pair — `statManaMax statManaMax2` (TWO tokens).
//!
//! The reader tolerates both single-lane predecessors: it walks the
//! post-dye lines and dispatches on TOKEN COUNT — a one-token line is
//! the aegis flag, the first two-token line is the life pair, the second
//! is the mana pair. So g89's flag-only v7, g88's life-only v7 and this
//! superset all load; missing pieces keep their fresh defaults. Any
//! merge of the two lanes keeps this reader (the superset).

use std::path::{Path, PathBuf};

use crate::player::Player;

pub struct SavedPlayer {
    pub x: f32,
    pub y: f32,
    pub hp: i32,
    /// `Player.statLifeMax` / `statLifeMax2` (`Player.cs:54773`'s write,
    /// `:55220-55224`'s read) — version 7 of the sidecar (W4-g88: the
    /// Life Fruit's permanent +5 and the crystals' +20, `life_fruit.rs`
    /// / `crystals.rs`). Older files load `None` and the loader keeps
    /// the fresh-default maxima. `(permanent, effective)`.
    pub life_max: Option<(i32, i32)>,
    /// `Player.statManaMax` / `statManaMax2` (`Player.cs:54775`'s write,
    /// `:55226-55229`'s read clamp 200) — the v7 superset's mana pair
    /// (W4-g88: the Mana Crystal's permanent +20). `None` on older
    /// files and on g89's flag-only v7. `(permanent, effective)`.
    pub mana_max: Option<(i32, i32)>,
    /// `Player.usedAegisFruit` (`Player.cs:854`, save `:54782` / load
    /// `:55256`) — version 7 of the sidecar (W4-g89's flag, g88's
    /// superset carries it at the SAME post-dye position); older files
    /// load with the unlock absent.
    pub used_aegis_fruit: bool,
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
    // Version 7 SUPERSET (the W4-g88 ⨉ W4-g89 reconciliation, the module
    // header's contract): the aegis flag FIRST (g89's post-dye position,
    // `Player.usedAegisFruit` save `Player.cs:54782`), then the life pair
    // (`statLifeMax statLifeMax2`, `:54773`), then the mana pair
    // (`statManaMax statManaMax2`, `:54775`). Vanilla rides all three in
    // the .plr; the port's live-session sidecar appends them after dye.
    s.push_str(&format!("{}\n", u8::from(player.used_aegis_fruit)));
    s.push_str(&format!("{} {}\n", player.life_max_base, player.max_hp));
    s.push_str(&format!("{} {}\n", player.mana_max_base, player.mana_max));
    std::fs::write(player_path(world), s)
}

pub fn load(world: &Path) -> Option<SavedPlayer> {
    let s = std::fs::read_to_string(player_path(world)).ok()?;
    let mut lines = s.lines();
    let version = lines.next()?.trim();
    let migrate = match version {
        // Version 4 adds the equipped armor line; 5 the bestiary kills; 6 the
        // dye kinds; 7 the SUPERSET post-dye block (the aegis flag ⨉ the
        // life pair ⨉ the mana pair, the module header's contract —
        // W4-g88 ⨉ W4-g89); 3 stores vanilla `ItemID`s; 1 and 2 stored the
        // ad-hoc tile/wall/weapon/coin ranges and are converted on load.
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
    // Version 7 SUPERSET (the module header's contract): walk the
    // post-dye lines and dispatch on TOKEN COUNT — ONE token is the
    // aegis flag (g89's line, `Player.cs:55256`'s read), the FIRST
    // two-token line is the life pair (`:55220-55224`, permanent half
    // clamps to 500), the SECOND is the mana pair (`:55226-55229`,
    // permanent half clamps to 200). This reads g89's flag-only v7, the
    // pre-reconciliation life-only v7 AND the full superset; missing
    // pieces keep None/false and the loader keeps the fresh defaults.
    let mut life_max = None;
    let mut mana_max = None;
    let mut used_aegis_fruit = false;
    if version == "7" {
        while let Some(line) = lines.next() {
            let mut toks = line.split_whitespace();
            let Some(first) = toks.next() else { continue };
            match toks.next() {
                None => used_aegis_fruit = first == "1",
                Some(second) => {
                    let base: i32 = first.parse().ok()?;
                    let effective: i32 = second.parse().ok()?;
                    if life_max.is_none() {
                        life_max = Some((base.min(500), effective));
                    } else if mana_max.is_none() {
                        mana_max = Some((base.min(200), effective));
                    }
                }
            }
        }
    }
    Some(SavedPlayer {
        x,
        y,
        hp,
        life_max,
        mana_max,
        used_aegis_fruit,
        inventory,
        coins,
        armor,
        dye,
        kills,
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
        // W4-g88: a fruit-upgraded character's permanent + effective caps.
        p.life_max_base = 405;
        p.max_hp = 405;
        // W4-g88 crystals: a crystal-upgraded mana pair + the g89 flag.
        p.mana_max_base = 40;
        p.mana_max = 40;
        p.used_aegis_fruit = true;
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
        assert_eq!(s.life_max, Some((405, 405)), "the v7 life-maxima pair");
        assert_eq!(s.mana_max, Some((40, 40)), "the v7 mana-maxima pair");
        assert!(s.used_aegis_fruit, "the g89 flag at its post-dye position");
        assert_eq!(s.inventory[0], (2, 12));
        assert_eq!(s.inventory[3], (12, 1));
        assert_eq!(s.inventory[49], (0, 0));
        assert_eq!(s.armor, [89, 80, 76]);
        assert_eq!(s.dye, dye);
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
        assert_eq!(s.life_max, None, "pre-v7 files carry no life maxima");
        assert_eq!(s.mana_max, None, "pre-v7 files carry no mana maxima");
        assert!(!s.used_aegis_fruit, "pre-v7 files carry no unlock");
        crate::test_world_helpers::remove_scratch(&dir);
    }

    /// W4-g88: a v6 sidecar (dye, no v7 block) loads with `None`/false —
    /// the loader keeps the fresh-default maxima for pre-upgrade saves.
    #[test]
    fn version_6_loads_without_life_maxima() {
        let dir = crate::test_world_helpers::test_scratch("persist-v6");
        let path = dir.join("tr-persist-v6.trwld");
        let mut line = String::from("6\n1.0 2.0 100\n");
        for _ in 0..50 {
            line.push_str("0 0 ");
        }
        line.push_str("\n0\n89 80 76\n0 \n");
        for _ in 0..10 {
            line.push_str("0 ");
        }
        line.push('\n');
        std::fs::write(player_path(&path), line).unwrap();
        let s = load(&path).expect("v6 sidecar");
        assert_eq!(s.armor, [89, 80, 76]);
        assert_eq!(s.dye, [0; 10]);
        assert_eq!(s.life_max, None);
        assert_eq!(s.mana_max, None);
        assert!(!s.used_aegis_fruit);
        crate::test_world_helpers::remove_scratch(&dir);
    }

    /// W4-g88: the v7 read clamps the permanent life cap at 500
    /// (`Player.cs:55221-55224`) and the permanent mana cap at 200
    /// (`:55227-55229`) — the superset block's both pairs.
    #[test]
    fn version_7_clamps_the_permanent_caps() {
        let dir = crate::test_world_helpers::test_scratch("persist-v7-clamp");
        let path = dir.join("tr-persist-v7.trwld");
        let mut line = String::from("7\n1.0 2.0 500\n");
        for _ in 0..50 {
            line.push_str("0 0 ");
        }
        line.push_str("\n0\n0 0 0\n0 \n");
        for _ in 0..10 {
            line.push_str("0 ");
        }
        line.push_str("\n0\n900 900\n900 900\n");
        std::fs::write(player_path(&path), line).unwrap();
        let s = load(&path).expect("v7 sidecar");
        assert_eq!(s.life_max, Some((500, 900)), "the life permanent half clamps");
        assert_eq!(s.mana_max, Some((200, 900)), "the mana permanent half clamps");
        crate::test_world_helpers::remove_scratch(&dir);
    }

    /// The reconciliation witness: g89's FLAG-ONLY v7 (one post-dye line,
    /// `920014cb`'s writer) still loads — the token-count dispatch reads
    /// the unlock and keeps the maxima at None.
    #[test]
    fn w4_g89s_flag_only_v7_still_loads() {
        let dir = crate::test_world_helpers::test_scratch("persist-v7-g89");
        let path = dir.join("tr-persist-v7-g89.trwld");
        let mut line = String::from("7\n1.0 2.0 100\n");
        for _ in 0..50 {
            line.push_str("0 0 ");
        }
        line.push_str("\n0\n0 0 0\n0 \n");
        for _ in 0..10 {
            line.push_str("0 ");
        }
        line.push_str("\n1\n"); // g89's single flag line
        std::fs::write(player_path(&path), line).unwrap();
        let s = load(&path).expect("g89's v7 sidecar");
        assert!(s.used_aegis_fruit, "the flag rides its own position");
        assert_eq!(s.life_max, None, "no life pair in g89's writer");
        assert_eq!(s.mana_max, None, "no mana pair in g89's writer");
        crate::test_world_helpers::remove_scratch(&dir);
    }

    /// The reconciliation witness: the pre-reconciliation LIFE-ONLY v7
    /// (`f1eb9b60`'s writer — the first post-dye line is a TWO-token
    /// pair) still loads: the pair dispatches to life, mana stays None,
    /// the flag stays false.
    #[test]
    fn the_life_only_v7_still_loads() {
        let dir = crate::test_world_helpers::test_scratch("persist-v7-life-only");
        let path = dir.join("tr-persist-v7-life.trwld");
        let mut line = String::from("7\n1.0 2.0 405\n");
        for _ in 0..50 {
            line.push_str("0 0 ");
        }
        line.push_str("\n0\n0 0 0\n0 \n");
        for _ in 0..10 {
            line.push_str("0 ");
        }
        line.push_str("\n405 405\n"); // f1eb9b60's single life line
        std::fs::write(player_path(&path), line).unwrap();
        let s = load(&path).expect("the life-only v7 sidecar");
        assert_eq!(s.life_max, Some((405, 405)));
        assert_eq!(s.mana_max, None);
        assert!(!s.used_aegis_fruit);
        crate::test_world_helpers::remove_scratch(&dir);
    }

    /// The superset round-trip: all three post-dye lines survive a
    /// save/load, in the contract order (flag, life pair, mana pair).
    #[test]
    fn the_v7_superset_round_trips_all_three_lines() {
        let world = World::new("t".into(), 32, 32);
        let mut p = Player::spawn(&world);
        p.used_aegis_fruit = true;
        p.life_max_base = 420;
        p.max_hp = 420;
        p.mana_max_base = 60;
        p.mana_max = 80; // an Arcane-inflated effective half rides verbatim
        let dir = crate::test_world_helpers::test_scratch("persist-v7-superset");
        let path = dir.join("tr-persist-v7-superset.trwld");
        save(&path, &p, [(0, 0); 50], 0, [0; 3], &[0; 10], &[]).unwrap();
        let text = std::fs::read_to_string(player_path(&path)).unwrap();
        let post_dye: Vec<&str> = text.lines().skip(7).collect();
        assert_eq!(post_dye, ["1", "420 420", "60 80"], "the contract order");
        let s = load(&path).expect("the superset v7 sidecar");
        assert!(s.used_aegis_fruit);
        assert_eq!(s.life_max, Some((420, 420)));
        assert_eq!(s.mana_max, Some((60, 80)));
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
