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
    /// `Player.extraAccessory` (`Player.cs:866`, the Demon Heart's permanent
    /// sixth accessory slot, use-grant `:45197`) — version 7 of the sidecar
    /// (W4-g87). The `.plr` half (`Serialize:54776`) is the f44 verify-sweep's
    /// lane; this is the live-session `.trplayer` carrier. Older files load
    /// false. Rides the self-identifying `dh` line (the g88 research/103 §6
    /// rule-5 contract — a sentinel prefix, never a bare numeric line).
    pub extra_accessory: bool,
    /// `Player.unlockedSuperCart` (`Player.cs:2052`, the Minecart Powerup's
    /// permanent Super Cart unlock, use-grant `:45209`) — version 7 (W4-g87),
    /// the second token of the `dh` line. The `.plr` half is the super-cart
    /// BitsByte (`Serialize:54914-54918`, f44's lane). Older files load false.
    pub unlocked_super_cart: bool,
    /// `Player.CurrentLoadoutIndex` (`Player.cs:3207`) — version 7; older
    /// files load on loadout 0. The active set rides the `armor`/`dye` lines
    /// above; this is which of the three sets is live.
    pub current_loadout_index: usize,
    /// `Player.Loadouts[3]` (`Player.cs:3200-3204`) — version 7; older files
    /// load three empty sets. Each stored set is 20 armor kinds + 10 dye
    /// kinds + 10 hide flags + the 30-bit share word (the port's sidecar
    /// keeps kinds, like the `armor`/`dye` lines — no prefix/stack).
    pub loadouts: [crate::loadout::EquipmentLoadout; 3],
    /// The LIVE equip set's share flags (`Item.favorited` over armor[0..20]
    /// then dye[0..10]) — version 7; older files load 0.
    pub equip_share_flags: u32,
}

pub fn player_path(world: &Path) -> PathBuf {
    world.with_extension("trplayer")
}

/// One stored loadout slot from its kind: a single-stack item (the sidecar
/// carries kinds only, like the `armor`/`dye` lines — no prefix/stack), or
/// air for kind 0.
fn loadout_item(kind: u16) -> crate::item::ItemSlot {
    if kind == 0 {
        crate::item::EMPTY_SLOT
    } else {
        crate::item::ItemSlot {
            kind,
            name: crate::registry::name(kind),
            count: 1,
            prefix: 0,
        }
    }
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
    equip_share_flags: u32,
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
    // Version 7 (W4-g87): the Demon Heart / Super Cart unlock line — a
    // SELF-IDENTIFYING `dh <extraAccessory> <unlockedSuperCart>` sentinel
    // line (the g88 research/103 §6 rule-5 contract: a new post-dye line
    // appends with a shape distinct from the numeric pair/flag/loadout
    // lines, so the superset reader's shape walk never misparses it). The
    // `.plr` halves are f44's lane; the pet `bought_*` flags are WORLD
    // globals (`.wld`/`.trwld`), never the sidecar. MERGER DEFAULT-DROP
    // (g87c retire, research/263 §6.2 — a6 138 §10.4 + f44 132 §2f): the
    // g88 v11 superset stays the sole L2 writer; this line + the v7 bump
    // + the reader arm + both round-trip witnesses DROP at merge (the v11
    // reader tolerates the 3-token line lossy-not-corrupt, both flags ride
    // f44's L1 walk + hydration). The branch keeps the line as the
    // in-session carrier until then.
    s.push_str(&format!(
        "dh {} {}\n",
        u8::from(player.extra_accessory),
        u8::from(player.unlocked_super_cart),
    ));
    // Version 7: the loadout block (`Player.cs:3200-3207`). The index, the
    // live set's share word, then each stored set as 20 armor kinds + 10 dye
    // kinds + 10 hide flags + its share word. Kinds only, like the armor/dye
    // lines above (the sidecar carries no prefix/stack).
    s.push_str(&format!("{} {}\n", player.current_loadout_index, equip_share_flags));
    for loadout in &player.loadouts {
        for item in loadout.armor {
            s.push_str(&format!("{} ", item.kind));
        }
        for item in loadout.dye {
            s.push_str(&format!("{} ", item.kind));
        }
        for hide in loadout.hide {
            s.push_str(&format!("{} ", hide as u8));
        }
        s.push_str(&format!("{}\n", loadout.favorited));
    }
    std::fs::write(player_path(world), s)
}

pub fn load(world: &Path) -> Option<SavedPlayer> {
    let s = std::fs::read_to_string(player_path(world)).ok()?;
    let mut lines = s.lines();
    let version = lines.next()?.trim();
    let migrate = match version {
        // Version 4 adds the equipped armor line; 5 the bestiary kills; 6 the
        // dye kinds; 7 the Demon Heart / Super Cart `dh` unlock line (W4-g87);
        // 3 stores vanilla `ItemID`s; 1 and 2 stored the ad-hoc
        // tile/wall/weapon/coin ranges and are converted on load.
        // dye kinds; 7 the loadout block; 3 stores vanilla `ItemID`s; 1 and 2
        // stored the ad-hoc tile/wall/weapon/coin ranges and are converted on
        // load.
        "3" | "4" | "5" | "6" | "7" => false,
        "1" | "2" => true,
        _ => return None,
    };
    let has_armor = matches!(version, "4" | "5" | "6" | "7");
    let has_kills = matches!(version, "5" | "6" | "7");
    let has_dye = matches!(version, "6" | "7");
    let has_loadouts = version == "7";
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
    if has_armor {
        let mut armor_line = lines.next()?.split_whitespace();
        for slot in &mut armor {
            *slot = armor_line.next()?.parse().ok()?;
        }
    }
    // Version 5 adds the bestiary kill pairs.
    let mut kills = Vec::new();
    if has_kills {
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
    if has_dye {
        let dye_line = lines.next()?;
        for (slot, kind) in dye.iter_mut().zip(dye_line.split_whitespace()) {
            *slot = kind.parse().ok()?;
        }
    }
    // Version 7 (W4-g87) adds the Demon Heart / Super Cart `dh` unlock line
    // (the self-identifying sentinel — `dh <extraAccessory> <unlockedSuperCart>`).
    // Older files load both false. The line is optional (a v7 file written
    // before this lane keeps the defaults), so a missing/non-`dh` line is
    // tolerated rather than fatal.
    let mut extra_accessory = false;
    let mut unlocked_super_cart = false;
    if version == "7" {
        if let Some(dh_line) = lines.next() {
            let mut parts = dh_line.split_whitespace();
            if parts.next() == Some("dh") {
                extra_accessory = parts.next().and_then(|v| v.parse::<u8>().ok()) == Some(1);
                unlocked_super_cart = parts.next().and_then(|v| v.parse::<u8>().ok()) == Some(1);
            }
        }
    }
    // Version 7 adds the loadout block; older files load on loadout 0 with
    // three empty sets and no live shares.
    let mut current_loadout_index = 0usize;
    let mut loadouts = [crate::loadout::EquipmentLoadout::default(); 3];
    let mut equip_share_flags = 0u32;
    if has_loadouts {
        let mut idx_line = lines.next()?.split_whitespace();
        current_loadout_index = idx_line
            .next()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(0)
            .min(2);
        equip_share_flags = idx_line.next().and_then(|v| v.parse().ok()).unwrap_or(0);
        for loadout in loadouts.iter_mut() {
            let mut parts = lines.next()?.split_whitespace();
            for item in loadout.armor.iter_mut() {
                let kind: u16 = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0);
                *item = loadout_item(kind);
            }
            for item in loadout.dye.iter_mut() {
                let kind: u16 = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0);
                *item = loadout_item(kind);
            }
            for hide in loadout.hide.iter_mut() {
                *hide = parts.next().and_then(|v| v.parse::<u8>().ok()).unwrap_or(0) != 0;
            }
            loadout.favorited = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0);
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
        extra_accessory,
        unlocked_super_cart,
        current_loadout_index,
        loadouts,
        equip_share_flags,
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
            0,
        )
        .unwrap();
        let s = load(&path).expect("sidecar");
        assert_eq!((s.x, s.y, s.hp), (123.5, 45.25, 73));
        assert_eq!(s.inventory[0], (2, 12));
        assert_eq!(s.inventory[3], (12, 1));
        assert_eq!(s.inventory[49], (0, 0));
        assert_eq!(s.armor, [89, 80, 76]);
        assert_eq!(s.dye, dye);
        // W4-g87: a fresh player carries neither permanent unlock.
        assert!(!s.extra_accessory, "fresh Demon Heart flag false");
        assert!(!s.unlocked_super_cart, "fresh Super Cart flag false");
        crate::test_world_helpers::remove_scratch(&dir);
    }

    /// W4-g87 — the Demon Heart / Super Cart `dh` unlock line round-trips
    /// (the v7 self-identifying sentinel, the g88 research/103 §6 rule-5
    /// contract). Both flags survive the save/load; the `.plr` halves are
    /// f44's lane, this is the live-session sidecar carrier.
    #[test]
    fn w4_g87_the_demon_heart_and_super_cart_unlocks_round_trip() {
        let world = World::new("t".into(), 32, 32);
        let mut p = Player::spawn(&world);
        p.extra_accessory = true; // the Demon Heart use-grant (`:45197`)
        p.unlocked_super_cart = true; // the Minecart Powerup grant (`:45209`)
        let dir = crate::test_world_helpers::test_scratch("persist-g87-dh");
        let path = dir.join("tr-persist-g87.trwld");
        save(&path, &p, [(0, 0); 50], 0, [0, 0, 0], &[0; 10], &[]).unwrap();
        // The writer emits the v7 `dh 1 1` sentinel line.
        let text = std::fs::read_to_string(player_path(&path)).unwrap();
        assert!(text.starts_with("7\n"), "the writer emits v7");
        assert!(text.lines().any(|l| l == "dh 1 1"), "the dh unlock line");
        let s = load(&path).expect("sidecar");
        assert!(s.extra_accessory, "the Demon Heart flag survives");
        assert!(s.unlocked_super_cart, "the Super Cart flag survives");
        crate::test_world_helpers::remove_scratch(&dir);
    }

    /// W4-g87 — a v6 sidecar (no `dh` line) loads both unlocks false (the
    /// back-compat default; the Demon Heart / Super Cart are new in v7).
    #[test]
    fn w4_g87_a_v6_sidecar_loads_the_unlocks_false() {
        let dir = crate::test_world_helpers::test_scratch("persist-g87-v6");
        let path = dir.join("tr-persist-g87-v6.trwld");
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
        assert!(!s.extra_accessory, "v6 has no Demon Heart flag");
        assert!(!s.unlocked_super_cart, "v6 has no Super Cart flag");
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

    /// W4-g55 — the v7 loadout block round-trips: the index, the live share
    /// word, and each stored set's armor/dye kinds + hide flags + share word.
    #[test]
    fn w4_g55_the_loadout_block_round_trips() {
        let world = World::new("t".into(), 32, 32);
        let mut p = Player::spawn(&world);
        p.current_loadout_index = 1;
        p.loadouts[2].armor[0] = loadout_item(90); // Iron Helmet stored in set 2
        p.loadouts[2].dye[1] = loadout_item(1007); // Red Dye
        p.loadouts[2].hide[4] = true;
        p.loadouts[2].set_shared(0, true);
        let dir = crate::test_world_helpers::test_scratch("persist-loadout");
        let path = dir.join("tr-persist-loadout.trwld");
        let inv = [(0u16, 0u32); 50];
        save(&path, &p, inv, 0, [0, 0, 0], &[0; 10], &[], 1 << 5).unwrap();
        let s = load(&path).expect("v7 sidecar");
        assert_eq!(s.current_loadout_index, 1);
        assert_eq!(s.equip_share_flags, 1 << 5);
        assert_eq!(s.loadouts[2].armor[0].kind, 90);
        assert_eq!(s.loadouts[2].dye[1].kind, 1007);
        assert!(s.loadouts[2].hide[4]);
        assert!(s.loadouts[2].is_shared(0));
        assert_eq!(s.loadouts[0], crate::loadout::EquipmentLoadout::default());
        assert_eq!(s.loadouts[1], crate::loadout::EquipmentLoadout::default());
        crate::test_world_helpers::remove_scratch(&dir);
    }

    /// W4-g55 — a v6 sidecar (no loadout block) loads on loadout 0 with three
    /// empty sets and no live shares — the version-7 read is gated.
    #[test]
    fn w4_g55_a_v6_sidecar_loads_empty_loadouts() {
        let dir = crate::test_world_helpers::test_scratch("persist-v6-loadout");
        let path = dir.join("tr-persist-v6-loadout.trwld");
        let mut line = String::from("6\n1.0 2.0 100\n");
        for _ in 0..50 {
            line.push_str("0 0 ");
        }
        line.push_str("\n0\n89 80 76\n0 \n1007 0 0 0 0 0 0 0 0 0 \n");
        std::fs::write(player_path(&path), line).unwrap();
        let s = load(&path).expect("v6 sidecar");
        assert_eq!(s.current_loadout_index, 0);
        assert_eq!(s.equip_share_flags, 0);
        assert_eq!(s.loadouts, [crate::loadout::EquipmentLoadout::default(); 3]);
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
