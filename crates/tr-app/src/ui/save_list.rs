//! The save listing behind the Select Player / Select World cards
//! (`Main.LoadPlayers`/`LoadWorlds`, `Main.ErasePlayer`/`EraseWorld`, plus the
//! `UICharacterSelect.UpdatePlayersList`/`UIWorldSelect.UpdateWorldsList`
//! orderings).
//!
//! Pure functions over injected directories — no `std::env` here (the App reads
//! the `TR_FAKE_CHARS`/`TR_FAKE_WORLDS` screenshot hooks and passes the counts
//! in, whole-UI P6 plan Task 5). The port has no `.plr` character format yet
//! (plan §9.4 deviation): characters are the session entry plus optional fakes,
//! while worlds are the real `Documents/My Games/Terrustaria/Worlds/*.trwld`
//! headers.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use tr_core::binio::Reader;

use crate::persist;

/// `PlayerFileData` as the cards need it (`UICharacterListItem` reads: Name,
/// IsFavorite, `Player.difficulty`, `statLifeMax`, `statManaMax`, loadStatus).
#[derive(Clone, Debug, PartialEq)]
pub struct PlayerEntry {
    /// `PlayerFileData.Name` — mirrors `Player.name` once the Player is set
    /// (`PlayerFileData.cs:17-27`).
    pub name: String,
    /// `Player.difficulty` (`Player.cs:1654`, 0-3; 0 fresh).
    pub difficulty: u8,
    /// `Player.statLifeMax` (`Player.cs:1924`, 100 fresh).
    pub life_max: i32,
    /// `Player.statManaMax` (`Player.cs:56075`, 20 fresh).
    pub mana_max: i32,
    /// `FileData.IsFavorite` (`FileData.cs:23`).
    pub is_favorite: bool,
    /// `loadStatus == Ok` (`StatusID.cs:16`, `WorldFileData.cs:75`); a
    /// sidecar-less session entry is playable.
    pub playable: bool,
    /// False for the implicit session entry (nothing on disk to erase,
    /// `Main.ErasePlayer` `Main.cs:4772-4791` needs a file).
    pub deletable: bool,
}

/// `WorldFileData` as the cards need it (`UIWorldListItem` reads: GetWorldName,
/// IsFavorite, IsValid, GameMode, WorldSizeName, CreationTime,
/// WorldGeneratorVersion).
#[derive(Clone, Debug, PartialEq)]
pub struct WorldEntry {
    /// `WorldFileData.Name` (`WorldFile.cs:207` reads it after the metadata
    /// block; the port's `tr_core::save` header keeps it after the seed).
    pub name: String,
    /// `WorldFileData.GameMode` (`WorldFileData.cs:53`); the port's writer
    /// stores none, so every entry is 0 (Classic) — documented deviation.
    pub difficulty: u8,
    /// 0/1/2 from the header width (`size_bucket`; `Main.cs:50990-51000`'s
    /// 4200/6400/8400 buckets).
    pub size_id: u8,
    /// `FileData.IsFavorite` (`FileData.cs:23`); the port reads a
    /// `<world>.trfav` marker (the stand-in for `Main.LocalFavoriteData`,
    /// `FileData.cs:22-24` — documented deviation).
    pub is_favorite: bool,
    /// The header parsed (`IsValid => LoadStatus == Ok`, `WorldFileData.cs:75`).
    pub valid: bool,
    /// The `<world>.map` sidecar exists (`WorldMap.TryGetMapPath`,
    /// `UIWorldSelect.cs:182-186`; the sidecar name is `persist.rs:23-28`).
    pub has_been_played: bool,
    /// The `.trwld` on disk; `None` for the fake entries (nothing to erase,
    /// `Main.EraseWorld` `Main.cs:4796-4813` needs a file).
    pub path: Option<PathBuf>,
}

/// `tr_core::save::MAGIC` (`save.rs:16`) — the port's own container format, not
/// a decompiled constant.
const MAGIC: &[u8; 8] = b"TRWLD01\0";
/// The header prefix the listing reads (magic, version, seed, name, size)
/// before the plausibility guard; names are short, so a small slice suffices.
const HEADER_READ_CAP: u64 = 8192;
/// `Main.maxLoadPlayer`/`maxLoadWorld` (`Main.cs:67065-67066`), the
/// `Directory.GetFiles` cap (`Main.cs:3969`, `:4006`).
const MAX_LOAD: usize = 1000;

/// The world-size bucket (`Main.cs:50990/50994/50998`'s `maxTilesX` 4200, 6400,
/// 8400 — Small, Medium, Large; the plan's `Main.cs:50975-50985` section).
/// Vanilla matches the exact widths and falls back to "Unknown"
/// (`WorldFileData.cs:329-347`); the port buckets instead so every width has a
/// size row — documented deviation.
pub fn size_bucket(width: i32) -> u8 {
    if width <= 4200 {
        0 // Main.cs:50990 Small
    } else if width <= 6400 {
        1 // Main.cs:50994 Medium
    } else {
        2 // Main.cs:50998 Large
    }
}

/// `Main.LoadWorlds` (`Main.cs:3959-3981`) over `*.trwld`: one `WorldEntry` per
/// file whose 8-byte magic + header (name/width) parse (`tr_core::save`
/// layout); unparsable files stay listed with `valid = false` and the file
/// stem for a name (`WorldFileData.FromInvalidWorld`, `WorldFileData.cs:360`).
/// A missing `Worlds` dir is empty (`TryCreatingDirectory`'s early return,
/// `Main.cs:3962-3965`). The list is name-sorted like
/// `Main.WorldListSortMethod` (`Main.cs:3986-4000`). `fake_count > 0`
/// prepends `World 1..n` entries (`TR_FAKE_WORLDS`, Task 5) so the cards are
/// screenshotable without a save directory.
pub fn list_worlds(worlds_dir: &Path, fake_count: usize) -> Vec<WorldEntry> {
    let mut reals = Vec::new();
    if let Ok(entries) = std::fs::read_dir(worlds_dir) {
        let mut paths: Vec<PathBuf> = entries
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|e| e == "trwld"))
            .collect();
        // `Directory.GetFiles` is alphabetical on Windows; sorting keeps the
        // name-sort ties deterministic regardless of host order.
        paths.sort();
        for path in paths.into_iter().take(MAX_LOAD) {
            reals.push(world_entry(&path));
        }
        // `Main.WorldList.Sort(Main.WorldListSortMethod)` (`Main.cs:3984`):
        // `data1.Name.CompareTo(data2.Name)` (`Main.cs:3999`).
        reals.sort_by(|a, b| a.name.cmp(&b.name));
    }
    let mut out = Vec::with_capacity(fake_count + reals.len());
    for i in 1..=fake_count {
        out.push(WorldEntry {
            name: format!("World {i}"),
            difficulty: 0,                      // Classic (`WorldFileData.cs:53` default)
            size_id: 1,                         // Medium, `Main.cs:50994`'s 6400 default
            is_favorite: false,
            valid: true,
            has_been_played: false,
            path: None,
        });
    }
    out.extend(reals);
    out
}

/// One directory entry: the header reader decides `valid`; the sidecars decide
/// the played/favourite flags.
fn world_entry(path: &Path) -> WorldEntry {
    let (name, width, valid) = read_header(path);
    let stem = path
        .file_stem()
        .map(OsStr::to_string_lossy)
        .map(|s| s.into_owned())
        .unwrap_or_default();
    WorldEntry {
        // A parsed name wins; otherwise the stem, like `FromInvalidWorld`'s
        // `GetFileName(path, includeExtension: false)` (`WorldFileData.cs:360`).
        name: name.unwrap_or(stem),
        // The port's writer stores no GameMode -> 0 (Classic); invalid entries
        // are 0 too, like `FromInvalidWorld` (`WorldFileData.cs:352`).
        difficulty: 0,
        // `FromInvalidWorld` reports the size of a 1x1 world
        // (`WorldFileData.cs:357`), which buckets to 0.
        size_id: if valid { size_bucket(width) } else { 0 },
        is_favorite: path.with_extension("trfav").exists(),
        valid,
        // `HasWorldBeenPlayedByActivePlayer` -> `WorldMap.TryGetMapPath`
        // (`UIWorldSelect.cs:182-186`); the port's map sidecar sits beside the
        // world (`persist.rs:23-28`).
        has_been_played: persist::map_path(path).exists(),
        path: Some(path.to_path_buf()),
    }
}

/// Reads the `tr_core::save` header prefix: `(name, width, valid)`. The whole
/// tile body is never read — only the first [`HEADER_READ_CAP`] bytes.
/// `valid` mirrors `IsValid => LoadStatus == Ok` (`WorldFileData.cs:75`)
/// against the same guards `tr_core::save::parse` applies (`save.rs:126-142`).
fn read_header(path: &Path) -> (Option<String>, i32, bool) {
    use std::io::Read;
    let mut raw = Vec::new();
    let read = std::fs::File::open(path)
        .and_then(|f| f.take(HEADER_READ_CAP).read_to_end(&mut raw));
    if read.is_err() {
        return (None, 1, false);
    }
    let mut r = Reader::new(&raw);
    // magic (`save.rs:128-130`)
    if r.take(8) != Some(MAGIC.as_slice()) {
        return (None, 1, false);
    }
    // version: 0 or beyond the writer's is unsupported (`save.rs:131-134`);
    // vanilla refuses later versions the same way (`WorldFile.cs:190-194`,
    // `StatusID.LaterVersion`).
    let Some(version) = r.u32() else {
        return (None, 1, false);
    };
    if version == 0 || version > 3 {
        return (None, 1, false);
    }
    // seed (`save.rs:135`) — not needed for the listing.
    if r.take(8).is_none() {
        return (None, 1, false);
    }
    let Some(name_len) = r.u32() else {
        return (None, 1, false);
    };
    let name = r
        .take(name_len as usize)
        .map(String::from_utf8_lossy)
        .map(|s| s.into_owned());
    let Some(width) = r.i32() else {
        return (name, 1, false);
    };
    let Some(height) = r.i32() else {
        return (name, 1, false);
    };
    // The plausibility guard `save.rs:140-142` — a world the port cannot load
    // stays listed but not valid.
    if width <= 0 || height <= 0 || width > 20_000 || height > 20_000 {
        return (name, 1, false);
    }
    (name, width, true)
}

/// `Main.LoadPlayers` (`Main.cs:4001-4030`). The port has no `.plr` format yet
/// (spec §9.4 deviation, plan Task 9): `dir == None` yields the session entry
/// when one is supplied. `fake_count > 0` prepends `Player 1..n` entries
/// (`TR_FAKE_CHARS`, difficulty 0, 100/20, deletable) so the cards are
/// screenshotable.
pub fn list_players(
    dir: Option<&Path>,
    session: Option<PlayerEntry>,
    fake_count: usize,
) -> Vec<PlayerEntry> {
    let mut out = Vec::new();
    // The fakes rebuild on every re-list: a Yes-deleted one returns in memory
    // until `TR_FAKE_CHARS` clears (the erase is in-memory only).
    for i in 1..=fake_count {
        out.push(PlayerEntry {
            name: format!("Player {i}"),
            difficulty: 0,   // `Player.cs:1654` (fresh `byte difficulty` = 0)
            life_max: 100,   // `Player.cs:1924`
            mana_max: 20,    // `Player.cs:56075`
            is_favorite: false,
            playable: true,
            deletable: true,
        });
    }
    if let Some(session) = session {
        out.push(session);
    }
    if let Some(dir) = dir {
        // The `Directory.GetFiles(PlayerPath, "*.plr")` scan (`Main.cs:4007`),
        // each through `Player.GetFileData` (`Main.cs:4010-4017`). Files whose
        // data cannot be loaded return null there and are skipped
        // (`Player.cs:55862-55881`) — and with no `.plr` reader every file
        // fails, so the port lists none of them. The scan stays so a future
        // reader slots into the same shape.
        if let Ok(entries) = std::fs::read_dir(dir) {
            let mut paths: Vec<PathBuf> = entries
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| p.extension().is_some_and(|e| e == "plr"))
                .collect();
            paths.sort();
            let _ = paths.into_iter().take(MAX_LOAD).count();
        }
    }
    out
}

/// `Main.EraseWorld` (`Main.cs:4796-4813`; the mode-9 Yes branch calls it,
/// `Main.cs:49681-49683`): removes `<path>`, `<path>.trplayer` and
/// `<path>.map` (the port's sidecars, `persist.rs:19-28`). Missing files are
/// ignored. Vanilla moves the `.wld` and its `.bak`/`.bak2..9` files to the
/// recycle bin (`Main.cs:4799-4808`) — the port has neither, documented
/// deviation.
pub fn erase_world(path: &Path) -> std::io::Result<()> {
    delete_if_exists(path)?;
    delete_if_exists(&persist::player_path(path))?;
    delete_if_exists(&persist::map_path(path))?;
    Ok(())
}

/// `Main.ErasePlayer` (`Main.cs:4772-4791`) for a sidecar-backed entry:
/// deletes the file and its `.bak` (`:4775-4778`) and the sibling directory
/// named by the extensionless path (`:4782-4786`); the caller re-lists
/// (`Main.LoadPlayers`, `:4787`). The session/fake entries have nothing on
/// disk — `None` drops them from the list in memory only.
pub fn erase_player(path: Option<&Path>) -> std::io::Result<()> {
    let Some(path) = path else {
        return Ok(());
    };
    delete_if_exists(path)?;
    let bak = {
        let mut s = path.as_os_str().to_os_string();
        s.push(".bak");
        PathBuf::from(s)
    };
    delete_if_exists(&bak)?;
    // `text = Path.Substring(0, Path.Length - 4)` — the path minus its
    // 4-character extension (`Main.cs:4783`); delete it only when it exists
    // and does not end in '.' or a separator (`:4784-4785`, with '/' added for
    // the port's separators).
    let text = path.to_string_lossy();
    if text.chars().count() >= 4 {
        let stripped: String = text.chars().take(text.chars().count() - 4).collect();
        if let Some(last) = stripped.chars().last() {
            if last != '.' && last != '\\' && last != '/' {
                let dir = Path::new(&stripped);
                if dir.is_dir() {
                    std::fs::remove_dir_all(dir)?;
                }
            }
        }
    }
    Ok(())
}

/// `FileUtilities.Delete`'s port-side shape: a missing file is already gone.
fn delete_if_exists(path: &Path) -> std::io::Result<()> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    }
}

/// `UICharacterSelect.UpdatePlayersList` (`UICharacterSelect.cs:146-157`):
/// `orderby x.IsFavorite descending, x.LastPlayed descending, x.Name,
/// x.GetFileName()`. The port has no play times and `PlayerEntry` carries no
/// filename — the list's own order (the `list_players` construction order)
/// stands in for both keys, so the stable sort reduces to favourite-first,
/// then name.
pub fn sort_players(players: &mut [PlayerEntry]) {
    players.sort_by(|a, b| {
        b.is_favorite
            .cmp(&a.is_favorite)
            .then_with(|| a.name.cmp(&b.name))
    });
}

/// `UIWorldSelect.UpdateWorldsList` (`UIWorldSelect.cs:153-166`):
/// `OrderByDescending(CanWorldBeJoinedByActivePlayer)
/// .ThenByDescending(IsNewlyGenerated).ThenByDescending(x.IsFavorite)
/// .ThenByDescending(HasWorldBeenPlayedByActivePlayer)
/// .ThenByDescending(x.LastPlayed).ThenBy(x.Name).ThenBy(x.GetFileName())`.
/// The port has no newly-generated tracking (always false,
/// `UIWorldSelect.cs:168-175` — the constant key is omitted) and no play
/// times — the list order stands in for `LastPlayed`. The filename tie-break
/// compares the path's file name (`FileData.GetFileName`, `FileData.cs:32-34`);
/// `None` (a fake entry) sorts first, keeping the prepend order on ties.
pub fn sort_worlds(worlds: &mut [WorldEntry], active_is_creative: bool) {
    worlds.sort_by(|a, b| {
        joinable_by_creative(active_is_creative, b.difficulty)
            .cmp(&joinable_by_creative(active_is_creative, a.difficulty))
            .then_with(|| b.is_favorite.cmp(&a.is_favorite))
            .then_with(|| b.has_been_played.cmp(&a.has_been_played))
            .then_with(|| a.name.cmp(&b.name))
            .then_with(|| file_name(&a.path).cmp(&file_name(&b.path)))
    });
}

/// `UIWorldSelect.CanWorldBeJoinedByActivePlayer`
/// (`UIWorldSelect.cs:177-180`): `Main.ActivePlayerFileData.Player.difficulty
/// == 3 == (file.GameMode == 3)` — the active player's Journey flag must match
/// the world's. Private helper of [`sort_worlds`]; the card-facing
/// `world_select::joinable` lands with Task 4.
fn joinable_by_creative(active_is_creative: bool, world_game_mode: u8) -> bool {
    active_is_creative == (world_game_mode == 3)
}

/// `FileData.GetFileName` (`FileData.cs:32-34`) — the path's final component;
/// `None` for the port's disk-less entries.
fn file_name(path: &Option<PathBuf>) -> Option<&OsStr> {
    path.as_deref().and_then(Path::file_name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::World;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(name);
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn player(name: &str, favorite: bool, life_max: i32) -> PlayerEntry {
        PlayerEntry {
            name: name.into(),
            difficulty: 0,
            life_max,
            mana_max: 20,
            is_favorite: favorite,
            playable: true,
            deletable: true,
        }
    }

    fn world(name: &str, difficulty: u8, favorite: bool, played: bool) -> WorldEntry {
        WorldEntry {
            name: name.into(),
            difficulty,
            size_id: 0,
            is_favorite: favorite,
            valid: true,
            has_been_played: played,
            path: None,
        }
    }

    /// Writes a real `.trwld` through the port's own saver.
    fn write_world(path: &Path, name: &str, width: i32, height: i32) {
        tr_core::save::save(path, &World::new(name.into(), width, height)).unwrap();
    }

    /// Hand-writes a `tr_core::save` header (magic, version, seed, name,
    /// width, height) followed by junk, without building the tile body —
    /// exactly what the listing reads.
    fn write_header_only(path: &Path, name: &str, width: i32, height: i32) {
        let mut buf = Vec::new();
        buf.extend_from_slice(MAGIC);
        buf.extend_from_slice(&2u32.to_le_bytes());
        buf.extend_from_slice(&0u64.to_le_bytes());
        buf.extend_from_slice(&(name.len() as u32).to_le_bytes());
        buf.extend_from_slice(name.as_bytes());
        buf.extend_from_slice(&width.to_le_bytes());
        buf.extend_from_slice(&height.to_le_bytes());
        buf.extend_from_slice(b"junk tail");
        std::fs::write(path, buf).unwrap();
    }

    #[test]
    fn size_buckets_match_the_vanilla_widths() {
        // `Main.cs:50990` (4200 -> Small), `:50994` (6400 -> Medium),
        // `:50998` (8400 -> Large); the plan's oracles: 4200/6400/8400/1200.
        assert_eq!(size_bucket(4200), 0);
        assert_eq!(size_bucket(6400), 1);
        assert_eq!(size_bucket(8400), 2);
        assert_eq!(size_bucket(1200), 0);
        // The bucket bounds (vanilla matches exact widths instead — the
        // documented plan deviation).
        assert_eq!(size_bucket(4201), 1);
        assert_eq!(size_bucket(6401), 2);
        // `FromInvalidWorld`'s 1x1 size (`WorldFileData.cs:357`).
        assert_eq!(size_bucket(1), 0);
    }

    #[test]
    fn list_worlds_reads_the_trwld_headers() {
        let dir = temp_dir("tr-savelist-worlds");
        write_world(&dir.join("alpha.trwld"), "Alpha", 64, 48);
        // A garbage `.trwld` stays listed with `valid = false` and the stem
        // for a name (`WorldFileData.cs:360`).
        std::fs::write(dir.join("Beta.trwld"), b"not a world at all").unwrap();
        // Non-`.trwld` files are not listed (`Main.cs:3968`'s "*.wld" filter).
        std::fs::write(dir.join("notes.txt"), b"ignore").unwrap();
        // The atomic-write temp suffix (`save.rs:39`) is not a world either.
        std::fs::write(dir.join("alpha.trwld.tmp"), b"partial").unwrap();

        let worlds = list_worlds(&dir, 0);
        let names: Vec<&str> = worlds.iter().map(|w| w.name.as_str()).collect();
        assert_eq!(names, vec!["Alpha", "Beta"], "name-sorted per :3999");
        let alpha = &worlds[0];
        assert!(alpha.valid);
        assert_eq!(alpha.name, "Alpha");
        assert_eq!(alpha.size_id, size_bucket(64));
        assert_eq!(alpha.path.as_deref(), Some(dir.join("alpha.trwld").as_path()));
        assert_eq!(alpha.difficulty, 0, "the port writes no GameMode");
        let beta = &worlds[1];
        assert!(!beta.valid, "garbage fails the header reader");
        assert_eq!(beta.name, "Beta", "the stem stands in for the name");
        assert_eq!(beta.size_id, 0, "FromInvalidWorld's 1x1 size buckets to 0");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn list_worlds_flags_the_sidecars() {
        let dir = temp_dir("tr-savelist-flags");
        // A 6400-wide header parses without building the tile body.
        write_header_only(&dir.join("mid.trwld"), "Mid", 6400, 1800);
        std::fs::write(dir.join("mid.map"), b"explored").unwrap();
        std::fs::write(dir.join("mid.trfav"), b"").unwrap();

        let worlds = list_worlds(&dir, 0);
        assert_eq!(worlds.len(), 1);
        let mid = &worlds[0];
        assert!(mid.valid);
        assert_eq!(mid.size_id, 1, "6400 is the Medium bucket (`:50994`)");
        assert!(mid.has_been_played, "the `.map` sidecar exists");
        assert!(mid.is_favorite, "the `.trfav` marker exists");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn list_worlds_missing_dir_is_empty() {
        let missing = std::env::temp_dir().join("tr-savelist-nope");
        let _ = std::fs::remove_dir_all(&missing);
        assert!(list_worlds(&missing, 0).is_empty());
    }

    #[test]
    fn list_worlds_prepends_the_fakes() {
        let dir = temp_dir("tr-savelist-fakeworlds");
        write_world(&dir.join("zeta.trwld"), "Zeta", 64, 48);
        let worlds = list_worlds(&dir, 2);
        let names: Vec<&str> = worlds.iter().map(|w| w.name.as_str()).collect();
        assert_eq!(names, vec!["World 1", "World 2", "Zeta"]);
        assert!(worlds[0].path.is_none() && worlds[0].valid);
        assert_eq!(worlds[0].size_id, 1, "the Medium default (`Main.cs:50994`)");
        assert!(!worlds[0].has_been_played && !worlds[0].is_favorite);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn list_players_session_fakes_and_garbage() {
        // `list_players(None, None, 0)` is the empty-list screenshot case.
        assert!(list_players(None, None, 0).is_empty());
        // The session entry passes through unchanged.
        let session = PlayerEntry {
            name: "Hero".into(),
            difficulty: 0,
            life_max: 100,
            mana_max: 20,
            is_favorite: false,
            playable: true,
            deletable: false,
        };
        assert_eq!(list_players(None, Some(session.clone()), 0), vec![session.clone()]);
        // `fake_count > 0` prepends `Player 1..n`, deletable per the plan.
        let fakes = list_players(None, None, 2);
        assert_eq!(
            fakes.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(),
            vec!["Player 1", "Player 2"]
        );
        assert!(fakes.iter().all(|p| p.deletable && p.playable));
        assert_eq!(fakes[0].difficulty, 0);
        assert_eq!((fakes[0].life_max, fakes[0].mana_max), (100, 20));
        // Fakes go in front of the session entry.
        let mixed = list_players(None, Some(session.clone()), 1);
        assert_eq!(mixed.len(), 2);
        assert_eq!(mixed[0].name, "Player 1");
        assert_eq!(mixed[1], session);
        // `.plr` files exist but cannot be read, so none are listed
        // (`Player.cs:55862-55881`'s null skip).
        let dir = temp_dir("tr-savelist-players");
        std::fs::write(dir.join("somebody.plr"), b"unreadable").unwrap();
        assert!(list_players(Some(&dir), None, 0).is_empty());
        // A missing player directory is not an error either.
        let missing = std::env::temp_dir().join("tr-savelist-noplayers");
        let _ = std::fs::remove_dir_all(&missing);
        assert!(list_players(Some(&missing), None, 0).is_empty());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn erase_world_removes_the_world_and_its_sidecars() {
        let dir = temp_dir("tr-savelist-erase");
        let world = dir.join("gone.trwld");
        write_world(&world, "Gone", 64, 48);
        std::fs::write(dir.join("gone.trplayer"), b"3\n1 1 100\n").unwrap();
        std::fs::write(dir.join("gone.map"), b"").unwrap();
        std::fs::write(dir.join("gone.trfav"), b"").unwrap();

        erase_world(&world).unwrap();
        assert!(!world.exists());
        assert!(!dir.join("gone.trplayer").exists());
        assert!(!dir.join("gone.map").exists());
        // The favourite marker stands in for `Main.LocalFavoriteData`
        // (`FileData.cs:22-24`), which `EraseWorld` does not touch.
        assert!(dir.join("gone.trfav").exists());
        // Missing files are ignored.
        erase_world(&world).unwrap();
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn erase_player_removes_the_file_bak_and_sibling_dir() {
        let dir = temp_dir("tr-savelist-eraseplayer");
        let plr = dir.join("bob.plr");
        std::fs::write(&plr, b"player").unwrap();
        std::fs::write(dir.join("bob.plr.bak"), b"backup").unwrap();
        std::fs::create_dir(dir.join("bob")).unwrap();
        std::fs::write(dir.join("bob").join("client-side"), b"data").unwrap();

        erase_player(Some(&plr)).unwrap();
        assert!(!plr.exists());
        assert!(!dir.join("bob.plr.bak").exists());
        assert!(!dir.join("bob").exists(), "the sibling directory is removed");
        // Missing files are already gone -> still Ok.
        erase_player(Some(&plr)).unwrap();
        // The session/fake entries are dropped in memory only.
        erase_player(None).unwrap();
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn sort_players_favourites_first_then_name() {
        // `orderby IsFavorite descending, LastPlayed descending, Name,
        // GetFileName()` (`UICharacterSelect.cs:148-150`) with the port's
        // insertion order standing in for the last two keys.
        let mut players = vec![
            player("Same", false, 100),
            player("Zed", true, 100),
            player("Same", false, 50),
            player("Art", false, 100),
            player("Cal", true, 100),
        ];
        sort_players(&mut players);
        let names: Vec<&str> = players.iter().map(|p| p.name.as_str()).collect();
        // Name orders within each favourite group: with no play times every
        // `LastPlayed` ties, so the `Name` key applies (`:148-150`).
        assert_eq!(names, vec!["Cal", "Zed", "Art", "Same", "Same"]);
        // The equal-key pair keeps its insertion order (the stable stand-in
        // for `LastPlayed descending`): life 100 before life 50.
        assert_eq!(players[3].life_max, 100);
        assert_eq!(players[4].life_max, 50);
    }

    #[test]
    fn sort_worlds_follows_the_vanilla_key_chain() {
        // Joinable desc, favourite desc, played desc, name asc
        // (`UIWorldSelect.cs:153-162`); newly-generated is the constant false
        // key and `LastPlayed` is the insertion order.
        let mut worlds = vec![
            world("a", 0, false, false),
            world("b", 3, false, false),
            world("c", 3, true, false),
            world("d", 0, false, true),
        ];
        // A Journey active player joins Journey worlds first.
        sort_worlds(&mut worlds, true);
        let names: Vec<&str> = worlds.iter().map(|w| w.name.as_str()).collect();
        assert_eq!(names, vec!["c", "b", "d", "a"]);
        // A Classic active player joins Classic worlds first.
        sort_worlds(&mut worlds, false);
        let names: Vec<&str> = worlds.iter().map(|w| w.name.as_str()).collect();
        assert_eq!(names, vec!["d", "a", "c", "b"]);
    }

    #[test]
    fn sort_worlds_tie_breaks_on_the_file_name() {
        // Equal keys: `ThenBy(GetFileName())` (`UIWorldSelect.cs:163-165`).
        // A disk-less fake (no filename) sorts first, keeping the prepend
        // order; two files compare by their final path component.
        let mut worlds = vec![
            world("same", 0, false, false),
            WorldEntry {
                name: "same".into(),
                difficulty: 0,
                size_id: 0,
                is_favorite: false,
                valid: true,
                has_been_played: false,
                path: Some(PathBuf::from("saves/z.trwld")),
            },
            WorldEntry {
                name: "same".into(),
                difficulty: 0,
                size_id: 0,
                is_favorite: false,
                valid: true,
                has_been_played: false,
                path: Some(PathBuf::from("saves/a.trwld")),
            },
        ];
        sort_worlds(&mut worlds, false);
        assert!(worlds[0].path.is_none());
        assert_eq!(
            worlds[1].path.as_deref(),
            Some(Path::new("saves/a.trwld"))
        );
        assert_eq!(
            worlds[2].path.as_deref(),
            Some(Path::new("saves/z.trwld"))
        );
    }
}
