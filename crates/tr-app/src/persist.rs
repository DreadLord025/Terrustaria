//! Sidecar save for the player state (`.trplayer` next to a `.trwld`).
//!
//! The world format stays tile-only; the player's position, HP and hotbar counts live in a
//! small versioned text file. Loading validates that the saved position is not inside a
//! solid tile.
//!
//! ## The FINAL superset contract (W4-g88 fold — g89/g89b/g90/g55)
//!
//! Five lane writers collided on the post-dye block: g89 (`920014cb`,
//! v7 aegis flag), g88 (`f1eb9b60`, v7 life pair), g90 (`1f1b88f0`, v7
//! bread flag), g55 (`cde865fc`, v7 loadout block), g89b (`9bf03ede`,
//! v8 aegis+gummy unlock pair). The reconciled SUPERSET (this branch's
//! writer) is version **8**, g89b's unlock-line shape kept:
//!
//! 1. the unlock line — `usedAegisFruit usedGummyWorm` 0/1 (g89b's
//!    position, TWO tokens; token 3+ RESERVED for ambrosia/galaxy-pearl/
//!    arcane-crystal/aegis-crystal/DD2 — the layout doc's token map),
//! 2. the life-maxima pair — `statLifeMax statLifeMax2` (TWO tokens),
//! 3. the mana-maxima pair — `statManaMax statManaMax2` (TWO tokens),
//! 4. the artisan-bread flag line — `ateArtisanBread` 0/1 (ONE token,
//!    after the mana pair per the fold contract),
//! 5. the loadout index line — `currentLoadoutIndex equipShareFlags`
//!    (TWO tokens, g55's block after bread),
//! 6. three stored-set lines — each 20 armor kinds + 10 dye kinds +
//!    10 hide flags + the set's 30-bit share word (41 tokens each).
//!
//! The reader tolerates every predecessor. v8: the FIRST post-dye line
//! is positionally the unlock line (g89b's contract; 1 or 2 tokens read,
//! the rest reserved); the remaining lines dispatch on TOKEN COUNT/SHAPE
//! — a one-token line is the bread flag, the first two-token line the
//! life pair, the second the mana pair, the two-token line directly
//! BEFORE the first 41-token set line the loadout index line (g55's
//! block self-identifies by its set lines). v7: the FIRST one-token line
//! is the aegis flag, the SECOND the bread flag; pairs/loadouts as above.
//! So every single-lane v7, g89b's v8, the partial supersets and this
//! full superset all load; missing pieces keep their fresh defaults.
//! Any merge of the lanes keeps this reader (the superset). Full layout
//! + token reservations + back-compat matrix:
//! `docs/research/103-persist-v7-layout.md`.

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
    /// `Player.usedGummyWorm` (`Player.cs:2074`, save `:54785` / load
    /// `:55259`) — version 8 of the sidecar (W4-g89b `9bf03ede`), the
    /// SECOND token on the unlock line (the superset keeps g89b's
    /// position). The permanent +3 fishing skill (`:12857-12860`).
    pub used_gummy_worm: bool,
    /// `Player.ateArtisanBread` (`Player.cs:2048`) — version 7 (W4-g90,
    /// folded into the superset as the FOURTH post-dye line); older files
    /// load false (the ctor default, `Player.cs:56077`). The vanilla
    /// persists it in the `.plr` (`Serialize:54779`, `Deserialize:55245`,
    /// gate 256); the port's live-session sidecar carries it so the
    /// permanent +4 crafting reach survives a `.trwld` reload.
    pub ate_artisan_bread: bool,
    /// `Player.CurrentLoadoutIndex` (`Player.cs:3207`) — version 7
    /// (W4-g55 `cde865fc`, folded AFTER the bread line per the superset
    /// contract); older files load on loadout 0. The active set rides the
    /// `armor`/`dye` lines; this is which of the three sets is live.
    pub current_loadout_index: usize,
    /// `Player.Loadouts[3]` (`Player.cs:3200-3204`) — version 7 (W4-g55);
    /// older files load three empty sets. Each stored set is 20 armor
    /// kinds + 10 dye kinds + 10 hide flags + the 30-bit share word (the
    /// sidecar keeps kinds, like the `armor`/`dye` lines — no
    /// prefix/stack; `EquipmentLoadout.Serialize`,
    /// `EquipmentLoadout.cs:31-45`).
    pub loadouts: [crate::loadout::EquipmentLoadout; 3],
    /// The LIVE equip set's share flags (`Item.favorited` over armor[0..20]
    /// then dye[0..10]) — version 7 (W4-g55); older files load 0.
    pub equip_share_flags: u32,
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

/// One stored loadout slot from its kind: a single-stack item (the sidecar
/// carries kinds only, like the `armor`/`dye` lines — no prefix/stack), or
/// air for kind 0 (W4-g55 `d2210b91`'s helper, folded verbatim).
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
    // Version 8 SUPERSET (the module header's contract): g89b's unlock
    // line first (`usedAegisFruit usedGummyWorm`, `Player.cs:54782`/
    // `:54785`), then the pairs, the bread flag and the loadout block.
    let mut s = format!("8\n{} {} {}\n", player.x, player.y, player.hp);
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
    // Version 8 SUPERSET (the module header's contract, the
    // g88⨉g89⨉g89b⨉g90⨉g55 fold): the unlock line FIRST (g89b's post-dye
    // position, `usedAegisFruit usedGummyWorm`, `Player.cs:54782`/
    // `:54785`; token 3+ reserved — 103-persist-v7-layout.md), then the
    // life pair (`statLifeMax statLifeMax2`, `:54773`), then the mana pair
    // (`statManaMax statManaMax2`, `:54775`), then the artisan-bread flag
    // (`Player.ateArtisanBread`, `:54779`, W4-g90). Vanilla rides them all
    // in the .plr; the port's live-session sidecar appends them after dye.
    s.push_str(&format!(
        "{} {}\n",
        u8::from(player.used_aegis_fruit),
        u8::from(player.used_gummy_worm)
    ));
    s.push_str(&format!("{} {}\n", player.life_max_base, player.max_hp));
    s.push_str(&format!("{} {}\n", player.mana_max_base, player.mana_max));
    s.push_str(&format!("{}\n", u8::from(player.ate_artisan_bread)));
    // Version 7 SUPERSET, the g55 fold (`d2210b91`): the loadout block
    // AFTER the bread line (`Player.cs:3200-3207`). The index, the live
    // set's share word, then each stored set as 20 armor kinds + 10 dye
    // kinds + 10 hide flags + its share word. Kinds only, like the
    // armor/dye lines above (the sidecar carries no prefix/stack).
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
        // dye kinds; 7 the SUPERSET post-dye block (the module header's
        // contract — W4-g88 ⨉ W4-g89 ⨉ W4-g90 ⨉ W4-g55); 8 the g89b
        // unlock line (aegis + gummy, `9bf03ede`) atop the same block;
        // 3 stores vanilla `ItemID`s; 1 and 2 stored the ad-hoc
        // tile/wall/weapon/coin ranges and are converted on load.
        "3" | "4" | "5" | "6" | "7" | "8" => false,
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
    if matches!(version, "4" | "5" | "6" | "7" | "8") {
        let mut armor_line = lines.next()?.split_whitespace();
        for slot in &mut armor {
            *slot = armor_line.next()?.parse().ok()?;
        }
    }
    // Version 5 adds the bestiary kill pairs.
    let mut kills = Vec::new();
    if matches!(version, "5" | "6" | "7" | "8") {
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
    if matches!(version, "6" | "7" | "8") {
        let dye_line = lines.next()?;
        for (slot, kind) in dye.iter_mut().zip(dye_line.split_whitespace()) {
            *slot = kind.parse().ok()?;
        }
    }
    // Version 7/8 SUPERSET (the module header's contract).
    // v8 (the current writer, g89b's shape kept): the FIRST post-dye line
    // is POSITIONALLY the unlock line — token 1 the aegis flag
    // (`Player.cs:55256`), token 2 the gummy worm (`:55259`); a one-token
    // line still reads (g89b's v7 back-read), tokens 3+ are RESERVED
    // (103-persist-v7-layout.md) and skipped. The remaining lines dispatch
    // on TOKEN COUNT/SHAPE as below.
    // v7 (the tolerant predecessor walk): the FIRST one-token line is the
    // aegis flag (g89's line), the SECOND is the artisan-bread flag
    // (g90's line, `:55245`).
    // Both versions: the FIRST two-token line after the unlock walk is
    // the life pair (`:55220-55224`, permanent half clamps to 500), the
    // SECOND the mana pair (`:55226-55229`, clamps to 200); the g55
    // loadout block adds a THIRD two-token shape — the index/share line —
    // identified by the 41-token stored-set lines that follow it (20
    // armor + 10 dye + 10 hide + the share word,
    // `EquipmentLoadout.cs:31-45`): the two-token line directly BEFORE
    // the first set line is the index line, NOT a maxima pair
    // (`Utils.Clamp` index to 0..2, `Player.cs:55720-55727`).
    // So every single-lane v7, g89b's v8, the partial supersets AND the
    // full superset load; missing pieces keep None/false/empty defaults.
    let mut life_max = None;
    let mut mana_max = None;
    let mut used_aegis_fruit = false;
    let mut used_gummy_worm = false;
    let mut ate_artisan_bread = false;
    let mut current_loadout_index = 0usize;
    let mut loadouts = [crate::loadout::EquipmentLoadout::default(); 3];
    let mut equip_share_flags = 0u32;
    if matches!(version, "7" | "8") {
        // One stored-set line: ARMOR + DYE + HIDE kinds/flags + the word.
        const SET_TOKENS: usize = crate::inv::ARMOR + crate::inv::DYE + crate::loadout::HIDE + 1;
        let mut post: Vec<Vec<&str>> = lines
            .map(|l| l.split_whitespace().collect::<Vec<_>>())
            .collect();
        // v8: consume the unlock line positionally (g89b's contract).
        // Its tokens 3+ are reserved future unlock flags — skip them.
        let v7_flags = version == "7";
        if version == "8" {
            while post.first().is_some_and(|t| t.is_empty()) {
                post.remove(0);
            }
            if let Some(unlock) = post.first() {
                used_aegis_fruit = unlock.first().is_some_and(|t| *t == "1");
                used_gummy_worm = unlock.get(1).is_some_and(|t| *t == "1");
                post.remove(0);
            }
        }
        let set_start = post.iter().position(|t| t.len() == SET_TOKENS);
        let idx_line = match set_start {
            Some(i) if i > 0 && post[i - 1].len() == 2 => Some(i - 1),
            _ => None,
        };
        let mut flag_seen = false;
        let mut set_no = 0usize;
        for (n, toks) in post.iter().enumerate() {
            if Some(n) == idx_line {
                current_loadout_index = toks[0].parse::<usize>().unwrap_or(0).min(2);
                equip_share_flags = toks[1].parse().unwrap_or(0);
                continue;
            }
            match toks.len() {
                0 => continue,
                1 => {
                    if !v7_flags || flag_seen {
                        ate_artisan_bread = toks[0] == "1";
                    } else {
                        used_aegis_fruit = toks[0] == "1";
                        flag_seen = true;
                    }
                }
                2 => {
                    let base: i32 = toks[0].parse().ok()?;
                    let effective: i32 = toks[1].parse().ok()?;
                    if life_max.is_none() {
                        life_max = Some((base.min(500), effective));
                    } else if mana_max.is_none() {
                        mana_max = Some((base.min(200), effective));
                    }
                }
                SET_TOKENS => {
                    if set_no < loadouts.len() {
                        let l = &mut loadouts[set_no];
                        for (i, item) in l.armor.iter_mut().enumerate() {
                            *item = loadout_item(toks[i].parse().unwrap_or(0));
                        }
                        for (j, item) in l.dye.iter_mut().enumerate() {
                            *item = loadout_item(toks[crate::inv::ARMOR + j].parse().unwrap_or(0));
                        }
                        for (k, hide) in l.hide.iter_mut().enumerate() {
                            *hide = toks[crate::inv::ARMOR + crate::inv::DYE + k]
                                .parse::<u8>()
                                .unwrap_or(0)
                                != 0;
                        }
                        l.favorited = toks[SET_TOKENS - 1].parse().unwrap_or(0);
                        set_no += 1;
                    }
                }
                _ => {} // an unknown shape: tolerate, keep the defaults
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
        used_gummy_worm,
        ate_artisan_bread,
        current_loadout_index,
        loadouts,
        equip_share_flags,
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
        p.used_gummy_worm = true;
        p.ate_artisan_bread = true;
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
        assert_eq!(s.life_max, Some((405, 405)), "the v7 life-maxima pair");
        assert_eq!(s.mana_max, Some((40, 40)), "the v7 mana-maxima pair");
        assert!(s.used_aegis_fruit, "the g89 flag at its post-dye position");
        assert!(s.used_gummy_worm, "the g89b token on the unlock line");
        assert!(s.ate_artisan_bread, "the g90 flag after the mana pair");
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
        assert!(!s.used_gummy_worm, "pre-v8 files carry no worm unlock");
        assert!(!s.ate_artisan_bread, "pre-v7 files carry no loaf unlock");
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
        assert!(!s.used_gummy_worm);
        assert!(!s.ate_artisan_bread, "v6 loads false — the ctor default");
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
        assert!(!s.ate_artisan_bread, "no bread line in g89's writer");
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
        assert!(!s.ate_artisan_bread);
        crate::test_world_helpers::remove_scratch(&dir);
    }

    /// The fold witness: g90's BREAD-ONLY v7 (`1f1b88f0`'s writer — one
    /// post-dye flag line) loads into the FIRST flag slot (aegis), because
    /// a single flag line cannot self-identify; the bread half keeps false.
    /// The g90 lane's own reader maps that line to bread — after the merge
    /// the superset writer emits both flags, so the ambiguity only affects
    /// pre-fold single-flag files (documented in 103-persist-v7-layout.md).
    #[test]
    fn g90s_bread_only_v7_loads_into_the_first_flag_slot() {
        let dir = crate::test_world_helpers::test_scratch("persist-v7-g90");
        let path = dir.join("tr-persist-v7-g90.trwld");
        let mut line = String::from("7\n1.0 2.0 100\n");
        for _ in 0..50 {
            line.push_str("0 0 ");
        }
        line.push_str("\n0\n0 0 0\n0 \n");
        for _ in 0..10 {
            line.push_str("0 ");
        }
        line.push_str("\n1\n"); // g90's single flag line
        std::fs::write(player_path(&path), line).unwrap();
        let s = load(&path).expect("g90's v7 sidecar");
        assert!(s.used_aegis_fruit, "the lone flag rides slot 1 (g89's)");
        assert!(!s.ate_artisan_bread, "no second flag line to read");
        assert_eq!(s.life_max, None);
        assert_eq!(s.mana_max, None);
        crate::test_world_helpers::remove_scratch(&dir);
    }

    /// The fold witness: the three-line g88⨉g89 superset (`717e0640`'s
    /// writer — flag, life pair, mana pair, NO bread line) still loads;
    /// the bread half keeps its ctor default.
    #[test]
    fn the_three_line_superset_still_loads_without_bread() {
        let dir = crate::test_world_helpers::test_scratch("persist-v7-3line");
        let path = dir.join("tr-persist-v7-3line.trwld");
        let mut line = String::from("7\n1.0 2.0 420\n");
        for _ in 0..50 {
            line.push_str("0 0 ");
        }
        line.push_str("\n0\n0 0 0\n0 \n");
        for _ in 0..10 {
            line.push_str("0 ");
        }
        line.push_str("\n1\n420 420\n60 80\n");
        std::fs::write(player_path(&path), line).unwrap();
        let s = load(&path).expect("the three-line superset sidecar");
        assert!(s.used_aegis_fruit);
        assert_eq!(s.life_max, Some((420, 420)));
        assert_eq!(s.mana_max, Some((60, 80)));
        assert!(!s.ate_artisan_bread, "the pre-fold writer had no bread line");
        crate::test_world_helpers::remove_scratch(&dir);
    }

    /// The FULL superset round-trip: all post-dye lines survive a
    /// save/load, in the contract order (v8 unlock line, life pair, mana
    /// pair, bread flag, loadout index + 3 stored sets).
    #[test]
    fn the_superset_round_trips_the_full_block() {
        let world = World::new("t".into(), 32, 32);
        let mut p = Player::spawn(&world);
        p.used_aegis_fruit = true;
        p.used_gummy_worm = true;
        p.life_max_base = 420;
        p.max_hp = 420;
        p.mana_max_base = 60;
        p.mana_max = 80; // an Arcane-inflated effective half rides verbatim
        p.ate_artisan_bread = true;
        p.current_loadout_index = 1;
        p.loadouts[2].armor[0] = loadout_item(90);
        let dir = crate::test_world_helpers::test_scratch("persist-v8-superset");
        let path = dir.join("tr-persist-v8-superset.trwld");
        save(&path, &p, [(0, 0); 50], 0, [0; 3], &[0; 10], &[], 1 << 5).unwrap();
        let text = std::fs::read_to_string(player_path(&path)).unwrap();
        assert!(text.starts_with("8\n"), "the superset writer emits v8");
        let post_dye: Vec<&str> = text.lines().skip(7).collect();
        assert_eq!(post_dye.len(), 8, "unlock/life/mana/bread/index + 3 sets");
        assert_eq!(&post_dye[..5], ["1 1", "420 420", "60 80", "1", "1 32"]);
        let s = load(&path).expect("the superset v8 sidecar");
        assert!(s.used_aegis_fruit);
        assert!(s.used_gummy_worm);
        assert_eq!(s.life_max, Some((420, 420)));
        assert_eq!(s.mana_max, Some((60, 80)));
        assert!(s.ate_artisan_bread);
        assert_eq!(s.current_loadout_index, 1);
        assert_eq!(s.equip_share_flags, 1 << 5);
        assert_eq!(s.loadouts[2].armor[0].kind, 90);
        crate::test_world_helpers::remove_scratch(&dir);
    }

    /// The g89b fold witness: g89b's OWN v8 (`9bf03ede`'s writer — the
    /// two-token unlock line ONLY, no pairs/bread/loadouts) loads: both
    /// unlocks read positionally, everything else keeps fresh defaults.
    #[test]
    fn g89bs_unlock_only_v8_still_loads() {
        let dir = crate::test_world_helpers::test_scratch("persist-v8-g89b");
        let path = dir.join("tr-persist-v8-g89b.trwld");
        let mut line = String::from("8\n1.0 2.0 100\n");
        for _ in 0..50 {
            line.push_str("0 0 ");
        }
        line.push_str("\n0\n0 0 0\n0 \n");
        for _ in 0..10 {
            line.push_str("0 ");
        }
        line.push_str("\n0 1\n"); // g89b's unlock line: no fruit, worm eaten
        std::fs::write(player_path(&path), line).unwrap();
        let s = load(&path).expect("g89b's v8 sidecar");
        assert!(!s.used_aegis_fruit);
        assert!(s.used_gummy_worm, "the second token reads");
        assert_eq!(s.life_max, None);
        assert_eq!(s.mana_max, None);
        assert!(!s.ate_artisan_bread);
        assert_eq!(s.current_loadout_index, 0);
        assert_eq!(s.loadouts, [crate::loadout::EquipmentLoadout::default(); 3]);
        crate::test_world_helpers::remove_scratch(&dir);
    }

    /// The g89b fold witness: a v7 ONE-token unlock line under version 8
    /// (g89b's own back-read shape) still reads the fruit half; the worm
    /// keeps false.
    #[test]
    fn a_one_token_unlock_line_reads_under_v8() {
        let dir = crate::test_world_helpers::test_scratch("persist-v8-1tok");
        let path = dir.join("tr-persist-v8-1tok.trwld");
        let mut line = String::from("8\n1.0 2.0 100\n");
        for _ in 0..50 {
            line.push_str("0 0 ");
        }
        line.push_str("\n0\n0 0 0\n0 \n");
        for _ in 0..10 {
            line.push_str("0 ");
        }
        line.push_str("\n1\n");
        std::fs::write(player_path(&path), line).unwrap();
        let s = load(&path).expect("v8 sidecar");
        assert!(s.used_aegis_fruit, "the first token reads");
        assert!(!s.used_gummy_worm, "no second token — the worm absent");
        crate::test_world_helpers::remove_scratch(&dir);
    }

    /// The reservation witness: unlock-line tokens 3+ are RESERVED
    /// (ambrosia/pearl/arcane/crystal/DD2 — the layout doc's token map)
    /// and skipped, never misparsed as maxima pairs; the block behind
    /// them still reads.
    #[test]
    fn reserved_unlock_tokens_are_skipped() {
        let dir = crate::test_world_helpers::test_scratch("persist-v8-reserved");
        let path = dir.join("tr-persist-v8-reserved.trwld");
        let mut line = String::from("8\n1.0 2.0 420\n");
        for _ in 0..50 {
            line.push_str("0 0 ");
        }
        line.push_str("\n0\n0 0 0\n0 \n");
        for _ in 0..10 {
            line.push_str("0 ");
        }
        // A future writer's unlock line: 7 tokens (the full vanilla set).
        line.push_str("\n1 1 1 1 1 1 1\n420 420\n60 80\n1\n");
        std::fs::write(player_path(&path), line).unwrap();
        let s = load(&path).expect("v8 sidecar");
        assert!(s.used_aegis_fruit);
        assert!(s.used_gummy_worm);
        assert_eq!(s.life_max, Some((420, 420)), "the pairs read past the line");
        assert_eq!(s.mana_max, Some((60, 80)));
        assert!(s.ate_artisan_bread);
        crate::test_world_helpers::remove_scratch(&dir);
    }

    /// W4-g90 fold: the artisan-bread flag round-trips alone (the fresh
    /// player never ate one; the flag survives, everything else defaults).
    #[test]
    fn the_artisan_bread_flag_round_trips() {
        let world = World::new("t".into(), 32, 32);
        let mut p = Player::spawn(&world);
        p.ate_artisan_bread = true;
        let dir = crate::test_world_helpers::test_scratch("persist-bread");
        let path = dir.join("tr-persist-bread.trwld");
        save(&path, &p, [(0, 0); 50], 0, [0, 0, 0], &[0; 10], &[], 0).unwrap();
        let s = load(&path).expect("v8 sidecar");
        assert!(s.ate_artisan_bread, "the flag survives the round trip");
        assert!(!s.used_aegis_fruit, "the aegis slot stays clear");
        let fresh = Player::spawn(&world);
        save(&path, &fresh, [(0, 0); 50], 0, [0, 0, 0], &[0; 10], &[], 0).unwrap();
        assert!(!load(&path).expect("v8 sidecar").ate_artisan_bread);
        crate::test_world_helpers::remove_scratch(&dir);
    }

    /// W4-g55 fold — the loadout block round-trips: the index, the live
    /// share word, and each stored set's armor/dye kinds + hide flags +
    /// share word (`d2210b91`'s witness, the superset's block order).
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
        let s = load(&path).expect("v8 sidecar");
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

    /// W4-g55 fold — a v6 sidecar (no loadout block) loads on loadout 0
    /// with three empty sets and no live shares — the v7 read is gated.
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

    /// The fold witness: g55's LOADOUT-ONLY v7 (`cde865fc`'s writer — the
    /// index line + 3 set lines directly after dye, no flag/pair lines)
    /// still loads: the index line is recognised by its set lines, NOT
    /// misparsed as the life pair; the maxima keep None, flags false.
    #[test]
    fn g55s_loadout_only_v7_still_loads() {
        let dir = crate::test_world_helpers::test_scratch("persist-v7-g55");
        let path = dir.join("tr-persist-v7-g55.trwld");
        let mut line = String::from("7\n1.0 2.0 100\n");
        for _ in 0..50 {
            line.push_str("0 0 ");
        }
        line.push_str("\n0\n0 0 0\n0 \n");
        for _ in 0..10 {
            line.push_str("0 ");
        }
        line.push_str("\n2 4\n"); // g55's index/share line
        for _ in 0..3 {
            for _ in 0..41 {
                line.push_str("0 ");
            }
            line.push('\n');
        }
        std::fs::write(player_path(&path), line).unwrap();
        let s = load(&path).expect("g55's v7 sidecar");
        assert_eq!(s.current_loadout_index, 2, "the index rides, NOT the life pair");
        assert_eq!(s.equip_share_flags, 4);
        assert_eq!(s.life_max, None, "no maxima in g55's writer");
        assert_eq!(s.mana_max, None);
        assert!(!s.used_aegis_fruit);
        assert!(!s.ate_artisan_bread);
        assert_eq!(s.loadouts, [crate::loadout::EquipmentLoadout::default(); 3]);
        crate::test_world_helpers::remove_scratch(&dir);
    }

    /// The fold witness: the index read clamps to 0..2 (`Utils.Clamp`,
    /// `Player.cs:55720-55727`'s .plr read) — a corrupt index 9 loads 2.
    #[test]
    fn the_loadout_index_clamps_on_read() {
        let dir = crate::test_world_helpers::test_scratch("persist-loadout-clamp");
        let path = dir.join("tr-persist-loadout-clamp.trwld");
        let mut line = String::from("7\n1.0 2.0 100\n");
        for _ in 0..50 {
            line.push_str("0 0 ");
        }
        line.push_str("\n0\n0 0 0\n0 \n");
        for _ in 0..10 {
            line.push_str("0 ");
        }
        line.push_str("\n9 0\n");
        for _ in 0..3 {
            for _ in 0..41 {
                line.push_str("0 ");
            }
            line.push('\n');
        }
        std::fs::write(player_path(&path), line).unwrap();
        let s = load(&path).expect("v7 sidecar");
        assert_eq!(s.current_loadout_index, 2, "the index clamps to 2");
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
