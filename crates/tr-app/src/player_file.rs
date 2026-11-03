//! The `.plr` character file reader/writer — `Player.LoadPlayer`/
//! `Player.SavePlayer` (`Player.cs:55058`, `:54736`) at the release-279
//! section set, through the inventory.
//!
//! The vanilla files are AES-128-CBC encrypted with the fixed key
//! `"h3y_gUyZ"` in UTF-16-LE (`Player.cs:796-798` — the key doubles as the
//! IV), `PaddingMode.None` (RijndaelManaged with a 128-bit block and key is
//! exactly AES-128). The port writes the same section order so vanilla save
//! editors and the vanilla client can read the port's characters.

use std::path::Path;

use aes::cipher::{BlockCipherDecrypt, BlockCipherEncrypt, KeyInit};
use aes::Aes128;
use aes::Block;
use tr_core::binio::{Reader, Writer};

/// `Player.ENCRYPTION_KEY` (`Player.cs:796-798`): the UTF-16-LE bytes of
/// `"h3y_gUyZ"`; it doubles as the CBC IV (`CreateEncryptor(KEY, KEY)`).
pub const ENCRYPTION_KEY: [u8; 16] = [
    0x68, 0x00, 0x33, 0x00, 0x79, 0x00, 0x5F, 0x00, 0x67, 0x00, 0x55, 0x00, 0x79, 0x00, 0x5A,
    0x00,
];

/// `FileMetadata.MAGIC_NUMBER` (`FileMetadata.cs:8`): `"relogic"` in the low
/// 56 bits; the top byte is the `FileType`.
const FILE_METADATA_MAGIC: u64 = 0x6369_676F_6C65_72;
/// `FileType.Player` (`FileType.cs:8`).
const FILE_TYPE_PLAYER: u64 = 1;
/// The section set this port writes: the vanilla release 279
/// (`Player.cs` `release` gates; 279 predates the 322 favorite bits and the
/// 324 extra bool).
const RELEASE: i32 = 279;

const MAX_BUFFS: usize = 44;

/// The read `.plr` prefix — the sections the port consumes.
pub struct PlayerFile {
    pub name: String,
    pub difficulty: u8,
    pub skin_variant: u8,
    pub hair: i32,
    pub life: i32,
    pub life_max: i32,
    pub mana: i32,
    pub mana_max: i32,
    pub hair_color: [u8; 3],
    pub skin_color: [u8; 3],
    pub eye_color: [u8; 3],
    pub shirt_color: [u8; 3],
    pub undershirt_color: [u8; 3],
    pub pants_color: [u8; 3],
    pub shoe_color: [u8; 3],
    /// The 58 inventory slots as `(id, stack, prefix)`; `id == 0` = empty.
    pub inventory: [(i32, i32, u8); 58],
    pub is_favorite: bool,
    pub revision: u32,
}

#[derive(Debug)]
pub enum PlayerFileError {
    Io(std::io::Error),
    NotReLogicFormat,
    WrongFileType,
    LaterVersion(i32),
    Truncated(&'static str),
}

impl From<std::io::Error> for PlayerFileError {
    fn from(e: std::io::Error) -> Self {
        PlayerFileError::Io(e)
    }
}

impl std::fmt::Display for PlayerFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlayerFileError::Io(e) => write!(f, "io: {e}"),
            PlayerFileError::NotReLogicFormat => write!(f, "not a Re-Logic file"),
            PlayerFileError::WrongFileType => write!(f, "not a player file"),
            PlayerFileError::LaterVersion(v) => write!(f, "later version: {v}"),
            PlayerFileError::Truncated(w) => write!(f, "truncated at {w}"),
        }
    }
}

/// The AES-128-CBC decrypt with the vanilla key/IV and no padding.
pub fn cbc_decrypt(data: &[u8]) -> Vec<u8> {
    let cipher = Aes128::new((&ENCRYPTION_KEY).into());
    let mut out = vec![0u8; data.len() - data.len() % 16];
    let mut prev = ENCRYPTION_KEY;
    for (i, chunk) in data.chunks_exact(16).enumerate() {
        let mut block = (&Block::clone_from_slice(chunk)).clone();
        cipher.decrypt_block(&mut block);
        for (b, p) in block.iter_mut().zip(prev.iter()) {
            *b ^= *p;
        }
        out[i * 16..(i + 1) * 16].copy_from_slice(&block);
        prev.copy_from_slice(chunk);
    }
    out
}

/// The AES-128-CBC encrypt: the plaintext is zero-padded to the 16-byte
/// alignment (`PaddingMode.None` needs full blocks; the reader stops at the
/// logical field end, so the tail padding is invisible).
pub fn cbc_encrypt(data: &[u8]) -> Vec<u8> {
    let cipher = Aes128::new((&ENCRYPTION_KEY).into());
    let aligned = data.len() - data.len() % 16 + if data.len() % 16 == 0 { 0 } else { 16 };
    let mut padded = data.to_vec();
    padded.resize(aligned, 0);
    let mut out = vec![0u8; aligned];
    let mut prev = ENCRYPTION_KEY;
    for (i, chunk) in padded.chunks_exact(16).enumerate() {
        let mut block = (&Block::clone_from_slice(chunk)).clone();
        for (b, p) in block.iter_mut().zip(prev.iter()) {
            *b ^= *p;
        }
        cipher.encrypt_block(&mut block);
        out[i * 16..(i + 1) * 16].copy_from_slice(&block);
        prev.copy_from_slice(&block);
    }
    out
}

/// `Player.LoadPlayer`'s prefix walk (`Player.cs:55058` + `Deserialize`
/// `Player.cs:55122+` at release 279): the version, the file metadata, the
/// name, the difficulty, the play time, the appearance ids, the stats.
pub fn read_player_file(path: &Path) -> Result<PlayerFile, PlayerFileError> {
    let raw = std::fs::read(path)?;
    let plain = cbc_decrypt(&raw);
    read_player_bytes(&plain)
}

fn read_player_bytes(plain: &[u8]) -> Result<PlayerFile, PlayerFileError> {
    let mut r = Reader::new(plain);
    let version = r.i32().ok_or(PlayerFileError::Truncated("version"))?;
    if version >= 135 {
        // `FileMetadata.Read` (`FileMetadata.cs:45-71`): the magic u64, the
        // revision u32, the favorites u64.
        let magic = r.u64().ok_or(PlayerFileError::Truncated("meta"))?;
        if (magic & 0x00FF_FFFF_FFFF_FFFF) != FILE_METADATA_MAGIC & 0x00FF_FFFF_FFFF_FFFF {
            return Err(PlayerFileError::NotReLogicFormat);
        }
        if ((magic >> 56) & 0xFF) != FILE_TYPE_PLAYER {
            return Err(PlayerFileError::WrongFileType);
        }
        let revision = r.u32().ok_or(PlayerFileError::Truncated("revision"))?;
        let favorite_bits = r.u64().ok_or(PlayerFileError::Truncated("fav"))?;
        return read_player_fields(&mut r, version, revision, favorite_bits == 1);
    }
    Err(PlayerFileError::Truncated("pre-135"))
}

fn read_player_fields(
    r: &mut Reader,
    version: i32,
    revision: u32,
    is_favorite: bool,
) -> Result<PlayerFile, PlayerFileError> {
    if version > 326 {
        return Err(PlayerFileError::LaterVersion(version));
    }
    let name = r.cs_string().ok_or(PlayerFileError::Truncated("name"))?;
    let difficulty = if version >= 17 {
        r.u8().ok_or(PlayerFileError::Truncated("difficulty"))?
    } else {
        0
    };
    let _play_time = if version >= 138 {
        r.u64().ok_or(PlayerFileError::Truncated("playtime"))? as i64
    } else {
        0
    };
    let mut hair = r.i32().ok_or(PlayerFileError::Truncated("hair"))?;
    if hair >= 228 {
        hair = 0;
    }
    let _hair_dye = if version >= 82 {
        r.u8().ok_or(PlayerFileError::Truncated("hairDye"))?
    } else {
        0
    };
    let _team = if version >= 283 {
        r.u8().ok_or(PlayerFileError::Truncated("team"))?
    } else {
        0
    };
    let _hide = if version >= 124 {
        r.take(2).map(|b| b.to_vec()).unwrap_or_default()
    } else {
        Vec::new()
    };
    let mut skin_variant = if version >= 107 {
        r.u8().ok_or(PlayerFileError::Truncated("skin"))?
    } else {
        0
    };
    if version < 161 && skin_variant == 7 {
        skin_variant = 9;
    }
    let life = r.i32().ok_or(PlayerFileError::Truncated("life"))?;
    let mut life_max = r.i32().ok_or(PlayerFileError::Truncated("lifeMax"))?;
    life_max = life_max.min(500);
    let mut mana = r.i32().ok_or(PlayerFileError::Truncated("mana"))?;
    mana = mana.min(400);
    let mut mana_max = r.i32().ok_or(PlayerFileError::Truncated("manaMax"))?;
    mana_max = mana_max.min(200);
    // The unlock/seed bools through release 279 (`Player.cs:55231-55261`):
    // extraAccessory [125], the biome torch pair [229], the artisan bread
    // [256], the six consumed-unlock items [260]. `release >= 324` reads one
    // more — 279 does not.
    if version >= 125 {
        r.boolean().ok_or(PlayerFileError::Truncated("extra"))?;
    }
    if version >= 229 {
        r.boolean().ok_or(PlayerFileError::Truncated("bt1"))?;
        r.boolean().ok_or(PlayerFileError::Truncated("bt2"))?;
    }
    if version >= 256 {
        r.boolean().ok_or(PlayerFileError::Truncated("bread"))?;
    }
    if version >= 260 {
        for _ in 0..6 {
            r.boolean().ok_or(PlayerFileError::Truncated("unlock"))?;
        }
    }
    if version >= 182 {
        r.boolean().ok_or(PlayerFileError::Truncated("dd2"))?;
    }
    if version >= 128 {
        r.i32().ok_or(PlayerFileError::Truncated("tax"))?;
    }
    if version >= 254 {
        r.i32().ok_or(PlayerFileError::Truncated("pve"))?;
        r.i32().ok_or(PlayerFileError::Truncated("pvp"))?;
    }
    let mut colors = [[0u8; 3]; 7];
    for color in colors.iter_mut() {
        for b in color.iter_mut() {
            *b = r.u8().ok_or(PlayerFileError::Truncated("color"))?;
        }
    }
    // The armor: 20 slots x (id, prefix) — the 322+ favorited bool is absent
    // at 279 (`Player.cs:55303-55316`).
    for _ in 0..20 {
        r.i32().ok_or(PlayerFileError::Truncated("armor"))?;
        r.u8().ok_or(PlayerFileError::Truncated("armorPrefix"))?;
    }
    // The dye: 10 slots x (id, prefix) (`:55323-55332`).
    for _ in 0..10 {
        r.i32().ok_or(PlayerFileError::Truncated("dye"))?;
        r.u8().ok_or(PlayerFileError::Truncated("dyePrefix"))?;
    }
    // The inventory: 58 slots (`:55340-55362`); an out-of-range id reads the
    // empty-slot triple and skips the favorited bool.
    let mut inventory = [(0i32, 0i32, 0u8); 58];
    for slot in inventory.iter_mut() {
        let id = r.i32().ok_or(PlayerFileError::Truncated("invId"))?;
        if id >= 5455 {
            r.i32().ok_or(PlayerFileError::Truncated("invSkip"))?;
            r.u8().ok_or(PlayerFileError::Truncated("invSkipPrefix"))?;
            r.boolean().ok_or(PlayerFileError::Truncated("invSkipFav"))?;
            continue;
        }
        let stack = r.i32().ok_or(PlayerFileError::Truncated("invStack"))?;
        let prefix = r.u8().ok_or(PlayerFileError::Truncated("invPrefix"))?;
        if version >= 114 {
            r.boolean().ok_or(PlayerFileError::Truncated("invFav"))?;
        }
        *slot = (id, stack, prefix);
    }
    Ok(PlayerFile {
        name,
        difficulty,
        skin_variant,
        hair,
        life,
        life_max,
        mana,
        mana_max,
        hair_color: colors[0],
        skin_color: colors[1],
        eye_color: colors[2],
        shirt_color: colors[3],
        undershirt_color: colors[4],
        pants_color: colors[5],
        shoe_color: colors[6],
        inventory,
        is_favorite,
        revision,
    })
}

/// The created character's inventory: the vanilla ctor's starter items are
/// the copper shortsword/axe/pickaxe in slots 0-2 (`Player.cs:54356-54360`).
/// The writer serializes whatever the caller passes.
pub struct PlayerFileWrite {
    pub name: String,
    pub difficulty: u8,
    pub skin_variant: u8,
    pub hair: i32,
    pub life: i32,
    pub life_max: i32,
    pub mana: i32,
    pub mana_max: i32,
    pub hair_color: [u8; 3],
    pub skin_color: [u8; 3],
    pub eye_color: [u8; 3],
    pub shirt_color: [u8; 3],
    pub undershirt_color: [u8; 3],
    pub pants_color: [u8; 3],
    pub shoe_color: [u8; 3],
    pub inventory: [(i32, i32, u8); 58],
}

/// The `SavePlayer` mirror (`Player.cs:54736`) at release 279 through the
/// inventory, encrypted like `:54736`'s CryptoStream.
pub fn write_player_bytes(file: &PlayerFileWrite) -> Vec<u8> {
    let mut w = Writer::new();
    w.i32(RELEASE);
    // `FileMetadata.Write` (`FileMetadata.cs:19-27`).
    w.u64(FILE_METADATA_MAGIC | (FILE_TYPE_PLAYER << 56));
    w.u32(0); // the revision
    w.u64(0); // the favorites
    w.cs_string(&file.name);
    w.u8(file.difficulty);
    w.u64(0); // the play time
    w.i32(file.hair);
    w.u8(0); // the hair dye
    w.bytes(&[0, 0]); // the hide-visible bits (124+)
    w.u8(file.skin_variant);
    w.i32(file.life);
    w.i32(file.life_max);
    w.i32(file.mana);
    w.i32(file.mana_max);
    w.boolean(false); // extraAccessory
    w.boolean(false); // unlockedBiomeTorches
    w.boolean(false); // UsingBiomeTorches
    w.boolean(false); // ateArtisanBread
    for _ in 0..6 {
        w.boolean(false); // the consumed-unlock items
    }
    w.boolean(false); // downedDD2EventAnyDifficulty
    w.i32(0); // taxMoney
    w.i32(0); // numberOfDeathsPVE
    w.i32(0); // numberOfDeathsPVP
    for color in [
        file.hair_color,
        file.skin_color,
        file.eye_color,
        file.shirt_color,
        file.undershirt_color,
        file.pants_color,
        file.shoe_color,
    ] {
        for b in color {
            w.u8(b);
        }
    }
    for _ in 0..20 {
        w.i32(0);
        w.u8(0);
    }
    for _ in 0..10 {
        w.i32(0);
        w.u8(0);
    }
    for (id, stack, prefix) in file.inventory.iter() {
        w.i32(*id);
        w.i32(*stack);
        w.u8(*prefix);
        w.boolean(false);
    }
    cbc_encrypt(&w.into_inner())
}

/// `PlayerFileData.CreateAndSave` (`Player.cs:54560`): the write to
/// `Players/<name>.plr`.
pub fn write_player_file(path: &Path, file: &PlayerFileWrite) -> Result<(), PlayerFileError> {
    // `Player.SavePlayer` (`Player.cs:54740-54744`) streams through
    // `FileUtilities.WriteAllBytes` after the path's directory exists — the
    // Players dir may not exist yet on a first run.
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(path, write_player_bytes(file))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> PlayerFileWrite {
        let mut inventory = [(0i32, 0i32, 0u8); 58];
        inventory[0] = (2, 50, 0); // Dirt Block x50
        inventory[1] = (3, 50, 0); // Stone Block x50
        inventory[2] = (8, 99, 0); // Torch x99
        inventory[3] = (42, 44, 0); // Shuriken x44
        PlayerFileWrite {
            name: "Belsin".into(),
            difficulty: 0,
            skin_variant: 0,
            hair: 0,
            life: 100,
            life_max: 100,
            mana: 20,
            mana_max: 20,
            hair_color: [215, 90, 55],
            skin_color: [255, 125, 90],
            eye_color: [105, 90, 75],
            shirt_color: [175, 165, 140],
            undershirt_color: [160, 180, 215],
            pants_color: [255, 230, 175],
            shoe_color: [160, 105, 60],
            inventory,
        }
    }

    #[test]
    fn round_trip_through_the_crypto_and_the_sections() {
        // The write -> the read: the name/stats/inventory round-trip through
        // the AES-CBC layer and the release-279 section walk.
        let bytes = write_player_bytes(&sample());
        assert_eq!(bytes.len() % 16, 0, "the CBC padding aligns the file");
        // `read_player_bytes` takes the decrypted body — the CBC layer is
        // `read_player_file`'s job, so peel it here.
        let file = match read_player_bytes(&cbc_decrypt(&bytes)) { Ok(f) => f, Err(e) => panic!("read err: {e:?}") };
        assert_eq!(file.name, "Belsin");
        assert_eq!(file.difficulty, 0);
        assert_eq!(file.life, 100);
        assert_eq!(file.life_max, 100);
        assert_eq!(file.mana, 20);
        assert_eq!(file.mana_max, 20);
        assert_eq!(file.skin_variant, 0);
        assert_eq!(file.hair, 0);
        assert_eq!(file.inventory[0], (2, 50, 0));
        assert_eq!(file.inventory[2], (8, 99, 0));
        assert_eq!(file.inventory[57], (0, 0, 0));
        assert!((file.hair_color, file.skin_color) == ([215, 90, 55], [255, 125, 90]));
    }

    #[test]
    fn the_metadata_carries_the_player_type() {
        // FileMetadata.Read: the top byte of the magic word is the FileType —
        // the player files carry 1.
        let bytes = write_player_bytes(&sample());
        let plain = cbc_decrypt(&bytes);
        let magic = u64::from_le_bytes(plain[4..12].try_into().unwrap());
        assert_eq!((magic >> 56) & 0xFF, FILE_TYPE_PLAYER);
        assert_eq!(magic & 0x00FF_FFFF_FFFF_FFFF, FILE_METADATA_MAGIC & 0x00FF_FFFF_FFFF_FFFF);
    }

    #[test]
    fn the_caps_clamp_on_read() {
        // LoadPlayer clamps: statLifeMax 500, statManaMax 200, statMana 400
        // (`Player.cs:55217-55229`).
        let mut file = sample();
        file.life_max = 900;
        file.mana_max = 900;
        file.mana = 900;
        let bytes = write_player_bytes(&file);
        let read = read_player_bytes(&cbc_decrypt(&bytes)).unwrap();
        assert_eq!(read.life_max, 500);
        assert_eq!(read.mana_max, 200);
        assert_eq!(read.mana, 400);
    }

    #[test]
    fn the_vanilla_key_is_the_utf16_of_h3y_guyz() {
        // Player.cs:798: `UnicodeEncoding().GetBytes("h3y_gUyZ")` — UTF-16-LE,
        // 8 chars x 2 bytes = the 128-bit key.
        let expected: [u8; 16] = [
            b'h', 0, b'3', 0, b'y', 0, b'_', 0, b'g', 0, b'U', 0, b'y', 0, b'Z', 0,
        ];
        assert_eq!(ENCRYPTION_KEY, expected);
    }
}
