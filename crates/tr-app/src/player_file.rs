//! The `.plr` character file reader/writer — `Player.LoadPlayer`/
//! `Player.SavePlayer` (`Player.cs:55058`, `:54736`), stamped release **326**
//! (`InternalSavePlayerFile` writes the literal 326, `Player.cs:54738`).
//!
//! The vanilla files are AES-128-CBC encrypted with the fixed key
//! `"h3y_gUyZ"` in UTF-16-LE (`Player.cs:796-798` — the key doubles as the
//! IV), `PaddingMode.None` (RijndaelManaged with a 128-bit block and key is
//! exactly AES-128). The port writes the same section order so vanilla save
//! editors and the vanilla client can read the port's characters.
//!
//! **S1-J-5 increment 1: through `voidVaultInfo`.** `Player.Serialize`
//! (`Player.cs:54750-54933`) is ported from the name through the void-vault
//! info byte (`:54863`): the header fixes (`team` `:54757`, `hideMisc`
//! `:54770`, the dead 324 bool `:54780`, the 322 armour/dye favourited bools
//! `:54816`/`:54822`) plus the sections after the inventory — `miscEquips`/
//! `miscDyes` (`:54831-54837`), the four banks (`:54838-54862`) and
//! `voidVaultInfo` (`:54863`). The tail after `:54863` (buffs, spawn points,
//! hideInfo, loadouts, creative tracker, …) is the NEXT S1-J-5 round; a
//! vanilla reader of a port file stops cleanly at the vault byte until then.
//!
//! **Default-value policy.** The port models only the character-card state
//! (name, difficulty, appearance, life/mana, the 58-slot inventory and its
//! favourited bits). Every other field written so far is emitted with the
//! value a *fresh vanilla player* carries — the `Player` ctor
//! (`Player.cs:56036-56096`) and the field initializers — never omitted:
//! omission shifts every later section and breaks byte-comparability. The
//! reader walks the same span with the C# `Deserialize` version gates
//! (`Player.cs:55134-55504`), consuming (and dropping) unmodelled fields.
//!
//! **Backward compat.** The port's own pre-S1-J-5 files were a *subset* mirror
//! stamped release 279 (truncated after the inventory, missing `team` /
//! `hideMisc` / the 324 bool / the armour-dye favourites). They are not valid
//! vanilla 279 files, so the C# gates cannot read them;
//! [`read_legacy_port_subset`] keeps them loadable as a documented port-only
//! shim (see `docs/ANALYSIS.md`).

use std::path::{Path, PathBuf};

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
/// `FileType.Player` (`FileType.cs:8`): the enum is `None=0, Map=1, World=2,
/// Player=3`, so a player file's metadata top byte is **3**. S1-J-3's C#-written
/// `.plr` fixture exposed the old value `1` (which is `Map`): the reader rejected
/// every real vanilla character with `WrongFileType`, and the writer stamped the
/// wrong type. `FileMetadata.Read(reader, FileType.Player)` (`Player.cs:55082`)
/// validates exactly this byte.
const FILE_TYPE_PLAYER: u64 = 3;
/// The section set this port writes: the current vanilla release **326**
/// (`Player.InternalSavePlayerFile` writes the literal `326`,
/// `Player.cs:54738`). S1-J-5 raised this from the old subset's 279; the
/// emitted span currently runs through `voidVaultInfo` (`Serialize:54863`).
const RELEASE: i32 = 326;

/// The release the port's *pre-S1-J-5* subset writer stamped. Those files are
/// truncated after the inventory and miss `team`/`hideMisc`/the 324 bool, so
/// they are not valid vanilla 279 files; [`read_legacy_port_subset`] keeps
/// them loadable.
const LEGACY_PORT_RELEASE: i32 = 279;

/// `ItemID.Count` (`ItemID.cs:14861`): the inventory reader turns a type at or
/// past this into an empty slot (`Player.cs:55346`). The old subset reader used
/// 5455, which wrongly air-gapped the 5455..6195 band.
const ITEM_ID_COUNT: i32 = 6196;

/// `PlayerVariantID.Count` (`ID/PlayerVariantID.cs`): the load-time
/// `skinVariant` clamp ceiling (`LoadPlayer_LastMinuteFixes`, `Player.cs:55830`).
const PLAYER_VARIANT_COUNT: i32 = 12;

/// The read `.plr` — the character-card state the port models. The reader
/// walks `Deserialize` (`Player.cs:55134`) through `voidVaultInfo`
/// (`:55501-55504`) with the C# version gates, consuming and dropping every
/// field the port does not model; the tail past `:55504` is a later round.
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
    /// `Item.favorited`'s per-slot bits for the inventory — bit `i` is
    /// `inventory[i].favorited` (`Player.cs:54820-54824`'s write,
    /// `:55340-55362`'s read at release >= 114). The armour and dye
    /// favourites ride the same shape at release >= 322 (`:55315`/`:55336`)
    /// but have no port UI, so they are consumed and dropped.
    pub inv_favorites: u64,
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

/// `Player.LoadPlayer` (`Player.cs:55058`): decrypt, then walk the version +
/// `FileMetadata` + the full `Deserialize` section set.
pub fn read_player_file(path: &Path) -> Result<PlayerFile, PlayerFileError> {
    let raw = std::fs::read(path)?;
    let plain = cbc_decrypt(&raw);
    read_player_bytes(&plain)
}

fn read_player_bytes(plain: &[u8]) -> Result<PlayerFile, PlayerFileError> {
    let mut r = Reader::new(plain);
    let version = r.i32().ok_or(PlayerFileError::Truncated("version"))?;
    if version < 135 {
        // `LoadPlayer:55080-55087` synthesizes metadata for pre-135 files, but
        // their `Deserialize` body is the legacy string-item layout
        // (`:55506-55569`) the port does not model; refuse rather than misread.
        return Err(PlayerFileError::Truncated("pre-135"));
    }
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
    let is_favorite = favorite_bits == 1;
    match read_player_fields(&mut r, version, revision, is_favorite) {
        Ok(f) => Ok(f),
        // The port's own pre-S1-J-5 subset files are stamped 279 but are
        // truncated after the inventory and miss `hideMisc`; the C# gates
        // cannot read them. Retry with the port-only shim so old saves load.
        Err(PlayerFileError::Truncated(_)) if version == LEGACY_PORT_RELEASE => {
            let mut r2 = Reader::new(plain);
            r2.skip(4 + 20).ok_or(PlayerFileError::Truncated("legacy"))?;
            read_legacy_port_subset(&mut r2, revision, is_favorite)
        }
        Err(e) => Err(e),
    }
}

/// One `Item.DeserializeFrom(reader, readFavorite: false)` record
/// (`Item.cs:49921-49934`): `i32 type, i32 stack, u8 prefix`. Used for the
/// temporary-slot / refund items the port consumes and drops.
fn read_item_triple(r: &mut Reader, what: &'static str) -> Result<(), PlayerFileError> {
    r.i32().ok_or(PlayerFileError::Truncated(what))?;
    r.i32().ok_or(PlayerFileError::Truncated(what))?;
    r.u8().ok_or(PlayerFileError::Truncated(what))?;
    Ok(())
}

/// The full `Player.Deserialize` walk (`Player.cs:55134-55764`) for a file at
/// `version` in `[135, 326]`. Every field the port does not model is consumed
/// and dropped so the cursor stays aligned through the tail. Gates below 135
/// are always true in the accepted range and are read unconditionally (the
/// gate is cited); gates that can vary within the range are kept.
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
    let difficulty = r.u8().ok_or(PlayerFileError::Truncated("difficulty"))?; // gate 17
    let _play_time = r.u64().ok_or(PlayerFileError::Truncated("playtime"))?; // gate 138
    let mut hair = r.i32().ok_or(PlayerFileError::Truncated("hair"))?;
    if hair >= 228 {
        hair = 0; // `:55159-55162`
    }
    let _hair_dye = r.u8().ok_or(PlayerFileError::Truncated("hairDye"))?; // gate 82
    if version >= 283 {
        let _team = r.u8().ok_or(PlayerFileError::Truncated("team"))?; // `:55167-55170`
    }
    // hideVisibleAccessory: two BitsBytes (`:55171-55183`, gate 124).
    r.u8().ok_or(PlayerFileError::Truncated("hide1"))?;
    r.u8().ok_or(PlayerFileError::Truncated("hide2"))?;
    // hideMisc (`:55192-55195`, gate 119) — the byte the old subset omitted.
    let _hide_misc = r.u8().ok_or(PlayerFileError::Truncated("hideMisc"))?;
    let mut skin_variant = r.u8().ok_or(PlayerFileError::Truncated("skin"))?; // gate 107
    if version < 161 && skin_variant == 7 {
        skin_variant = 9; // `:55215-55218`
    }
    let life = r.i32().ok_or(PlayerFileError::Truncated("life"))?;
    let mut life_max = r.i32().ok_or(PlayerFileError::Truncated("lifeMax"))?;
    life_max = life_max.min(500); // `:55221-55224`
    let mut mana = r.i32().ok_or(PlayerFileError::Truncated("mana"))?;
    let mut mana_max = r.i32().ok_or(PlayerFileError::Truncated("manaMax"))?;
    mana_max = mana_max.min(200); // `:55227-55230`
    mana = mana.min(400); // `:55231-55234`
    // The unlock/seed bools (`:55235-55262`).
    let _extra = r.boolean().ok_or(PlayerFileError::Truncated("extra"))?; // gate 125
    if version >= 229 {
        r.boolean().ok_or(PlayerFileError::Truncated("bt1"))?;
        r.boolean().ok_or(PlayerFileError::Truncated("bt2"))?;
        if version >= 256 {
            r.boolean().ok_or(PlayerFileError::Truncated("bread"))?;
        }
        if version >= 324 {
            r.boolean().ok_or(PlayerFileError::Truncated("extra324"))?; // `:55247-55250`
        }
        if version >= 260 {
            for _ in 0..6 {
                r.boolean().ok_or(PlayerFileError::Truncated("unlock"))?;
            }
        }
    }
    if version >= 182 {
        r.boolean().ok_or(PlayerFileError::Truncated("dd2"))?;
    }
    let _tax = r.i32().ok_or(PlayerFileError::Truncated("tax"))?; // gate 128
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
    // The armor: 20 slots x (id, prefix[, favorited >= 322]) (`:55306-55318`).
    for _ in 0..20 {
        r.i32().ok_or(PlayerFileError::Truncated("armor"))?;
        r.u8().ok_or(PlayerFileError::Truncated("armorPrefix"))?;
        if version >= 322 {
            r.boolean().ok_or(PlayerFileError::Truncated("armorFav"))?;
        }
    }
    // The dye: 10 slots x (id, prefix[, favorited >= 322]) (`:55319-55340`).
    for _ in 0..10 {
        r.i32().ok_or(PlayerFileError::Truncated("dye"))?;
        r.u8().ok_or(PlayerFileError::Truncated("dyePrefix"))?;
        if version >= 322 {
            r.boolean().ok_or(PlayerFileError::Truncated("dyeFav"))?;
        }
    }
    // The inventory: 58 slots (`:55341-55367`); a type at/above ItemID.Count
    // reads the empty-slot triple and drops it.
    let mut inventory = [(0i32, 0i32, 0u8); 58];
    let mut inv_favorites: u64 = 0;
    for (i, slot) in inventory.iter_mut().enumerate() {
        let id = r.i32().ok_or(PlayerFileError::Truncated("invId"))?;
        if id >= ITEM_ID_COUNT {
            r.i32().ok_or(PlayerFileError::Truncated("invSkip"))?;
            r.u8().ok_or(PlayerFileError::Truncated("invSkipPrefix"))?;
            r.boolean().ok_or(PlayerFileError::Truncated("invSkipFav"))?; // gate 114
            continue;
        }
        let stack = r.i32().ok_or(PlayerFileError::Truncated("invStack"))?;
        let prefix = r.u8().ok_or(PlayerFileError::Truncated("invPrefix"))?;
        if r.boolean().ok_or(PlayerFileError::Truncated("invFav"))? {
            // gate 114
            inv_favorites |= 1 << i;
        }
        *slot = (id, stack, prefix);
    }
    // --- the post-inventory tail the old subset never reached -------------
    // miscEquips + miscDyes: 5 x (equip id/prefix, dye id/prefix)
    // (`:55420-55447`, gates 117/136 both true here).
    for _ in 0..5 {
        r.i32().ok_or(PlayerFileError::Truncated("miscEquip"))?;
        r.u8().ok_or(PlayerFileError::Truncated("miscEquipPrefix"))?;
        r.i32().ok_or(PlayerFileError::Truncated("miscDye"))?;
        r.u8().ok_or(PlayerFileError::Truncated("miscDyePrefix"))?;
    }
    // bank + bank2: 40 slots x (id, stack, prefix) each (`:55449-55462`, gate 58).
    for _ in 0..80 {
        read_item_triple(r, "bank")?;
    }
    if version >= 182 {
        for _ in 0..40 {
            read_item_triple(r, "bank3")?; // `:55479-55487`
        }
    }
    if version >= 198 {
        for _ in 0..40 {
            read_item_triple(r, "bank4")?; // `:55488-55500`
            if version >= 255 {
                r.boolean().ok_or(PlayerFileError::Truncated("bank4Fav"))?;
            }
        }
    }
    if version >= 199 {
        let _void_vault = r.u8().ok_or(PlayerFileError::Truncated("voidVault"))?; // `:55501-55504`
    }
    // S1-J-5 increment 1 stops here: everything the port models sits at or
    // before `voidVaultInfo`, so the walk may end — the `Serialize` tail
    // (buffs `:54864`, spawn points `:54877`, … `:54933`) is a later round and
    // is simply left unconsumed. `LoadPlayer_LastMinuteFixes` (`:55830`)
    // clamps skinVariant to the variant set.
    let skin_variant = (skin_variant as i32).clamp(0, PLAYER_VARIANT_COUNT - 1) as u8;
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
        inv_favorites,
        is_favorite,
        revision,
    })
}

/// The port's pre-S1-J-5 **subset** reader (release 279): the same prefix as
/// [`read_player_fields`] but *without* the `hideMisc` byte, stopping right
/// after the 58-slot inventory. Those files were never valid vanilla 279 files
/// (the C# would read `hideMisc` then run off the end), so this is a port-only
/// backward-compat shim, not a C# gate — see `docs/ANALYSIS.md`.
fn read_legacy_port_subset(
    r: &mut Reader,
    revision: u32,
    is_favorite: bool,
) -> Result<PlayerFile, PlayerFileError> {
    let name = r.cs_string().ok_or(PlayerFileError::Truncated("name"))?;
    let difficulty = r.u8().ok_or(PlayerFileError::Truncated("difficulty"))?;
    let _play_time = r.u64().ok_or(PlayerFileError::Truncated("playtime"))?;
    let mut hair = r.i32().ok_or(PlayerFileError::Truncated("hair"))?;
    if hair >= 228 {
        hair = 0;
    }
    let _hair_dye = r.u8().ok_or(PlayerFileError::Truncated("hairDye"))?;
    // The old writer emitted the two hide BitsBytes but no hideMisc.
    r.u8().ok_or(PlayerFileError::Truncated("hide1"))?;
    r.u8().ok_or(PlayerFileError::Truncated("hide2"))?;
    let mut skin_variant = r.u8().ok_or(PlayerFileError::Truncated("skin"))?;
    if skin_variant == 7 {
        skin_variant = 9; // release 279 < 161
    }
    let life = r.i32().ok_or(PlayerFileError::Truncated("life"))?;
    let mut life_max = r.i32().ok_or(PlayerFileError::Truncated("lifeMax"))?;
    life_max = life_max.min(500);
    let mut mana = r.i32().ok_or(PlayerFileError::Truncated("mana"))?;
    let mut mana_max = r.i32().ok_or(PlayerFileError::Truncated("manaMax"))?;
    mana_max = mana_max.min(200);
    mana = mana.min(400);
    // extraAccessory, biome-torch pair, artisan bread, six unlocks, dd2 —
    // the 11 bools the old subset writer emitted (no 324 dead slot).
    for _ in 0..11 {
        r.boolean().ok_or(PlayerFileError::Truncated("unlock"))?;
    }
    let _tax = r.i32().ok_or(PlayerFileError::Truncated("tax"))?;
    let _pve = r.i32().ok_or(PlayerFileError::Truncated("pve"))?;
    let _pvp = r.i32().ok_or(PlayerFileError::Truncated("pvp"))?;
    let mut colors = [[0u8; 3]; 7];
    for color in colors.iter_mut() {
        for b in color.iter_mut() {
            *b = r.u8().ok_or(PlayerFileError::Truncated("color"))?;
        }
    }
    for _ in 0..20 {
        r.i32().ok_or(PlayerFileError::Truncated("armor"))?;
        r.u8().ok_or(PlayerFileError::Truncated("armorPrefix"))?;
    }
    for _ in 0..10 {
        r.i32().ok_or(PlayerFileError::Truncated("dye"))?;
        r.u8().ok_or(PlayerFileError::Truncated("dyePrefix"))?;
    }
    let mut inventory = [(0i32, 0i32, 0u8); 58];
    let mut inv_favorites: u64 = 0;
    for (i, slot) in inventory.iter_mut().enumerate() {
        let id = r.i32().ok_or(PlayerFileError::Truncated("invId"))?;
        if id >= ITEM_ID_COUNT {
            r.i32().ok_or(PlayerFileError::Truncated("invSkip"))?;
            r.u8().ok_or(PlayerFileError::Truncated("invSkipPrefix"))?;
            r.boolean().ok_or(PlayerFileError::Truncated("invSkipFav"))?;
            continue;
        }
        let stack = r.i32().ok_or(PlayerFileError::Truncated("invStack"))?;
        let prefix = r.u8().ok_or(PlayerFileError::Truncated("invPrefix"))?;
        if r.boolean().ok_or(PlayerFileError::Truncated("invFav"))? {
            inv_favorites |= 1 << i;
        }
        *slot = (id, stack, prefix);
    }
    let skin_variant = (skin_variant as i32).clamp(0, PLAYER_VARIANT_COUNT - 1) as u8;
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
        inv_favorites,
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
    /// `Item.favorited`'s per-inventory-slot bits — bit `i` is slot `i`'s
    /// (`Player.cs:54820-54824`). The App's `favorite_slots` flatten to it.
    pub inv_favorites: u64,
}

/// The `SavePlayer` mirror (`Player.cs:54736-54740`): release 326, the
/// `FileMetadata`, then `Serialize` (`Player.cs:54750`) from the name through
/// `voidVaultInfo` (`:54863`) — S1-J-5 increment 1. Encrypted like `:54736`'s
/// CryptoStream.
pub fn write_player_bytes(file: &PlayerFileWrite) -> Vec<u8> {
    cbc_encrypt(&serialize_plaintext(file))
}

/// The unencrypted release-326 prologue (`InternalSavePlayerFile:54738-54739`:
/// the `i32 326` + `FileMetadata`) plus `Player.Serialize` (`Player.cs:54750`)
/// from the name through `voidVaultInfo` (`:54863`).
///
/// Every field the port does not model is written at its fresh-vanilla default
/// (the `Player` ctor `:56036-56096` + the field initializers), never omitted —
/// see the module-level default-value policy. The `Serialize` tail
/// (`:54864-54933`) is the next S1-J-5 round.
pub fn serialize_plaintext(file: &PlayerFileWrite) -> Vec<u8> {
    let mut w = Writer::new();
    w.i32(RELEASE); // `InternalSavePlayerFile:54738` writes the literal 326.
    // `FileMetadata.Write` (`FileMetadata.cs:22-27`).
    w.u64(FILE_METADATA_MAGIC | (FILE_TYPE_PLAYER << 56));
    w.u32(0); // the revision (`FromCurrentSettings:40` = 0; `Write`, not Increment)
    // `FileMetadata.Write`'s favourites flags (`FileMetadata.cs:26`): the bit
    // is dead in the vanilla too — `IsFavorite` is only ever set `false`
    // (`FileMetadata.cs:41`) and the live state sits in `favorites.json`
    // (`IO/FavoritesFile.cs:27-36`). The port's carrier is the `<stem>.trfav`
    // sidecar beside this file ([`favorite_marker_path`]), which this writer
    // never opens — so re-saving a character cannot clear its star.
    w.u64(0); // the favorites (see the sidecar above)

    // --- Player.Serialize (`Player.cs:54752-54863`) ------------------------
    w.cs_string(&file.name); // `:54752`
    w.u8(file.difficulty); // `:54753`
    w.u64(0); // `:54754` playTime.Ticks — the port tracks no play time (Zero)
    w.i32(file.hair); // `:54755`
    w.u8(0); // `:54756` hairDye (default 0)
    w.u8(0); // `:54757` (byte)team (`Player.cs:1430`, default 0 = none)
    w.u8(0); // `:54758-54763` hideVisibleAccessory[0..8] BitsByte
    w.u8(0); // `:54764-54769` hideVisibleAccessory[8..10] BitsByte
    w.u8(0); // `:54770` hideMisc BitsByte (`Player.cs:1710` = `(byte)0`)
    w.u8(file.skin_variant); // `:54771`
    w.i32(file.life); // `:54772`
    w.i32(file.life_max); // `:54773`
    w.i32(file.mana); // `:54774`
    w.i32(file.mana_max); // `:54775`
    w.boolean(false); // `:54776` extraAccessory (ctor `:56076`)
    w.boolean(false); // `:54777` unlockedBiomeTorches
    w.boolean(false); // `:54778` UsingBiomeTorches (unlocked=false -> false)
    w.boolean(false); // `:54779` ateArtisanBread (ctor `:56077`)
    w.boolean(false); // `:54780` the hardcoded `false` (the dead 324 slot)
    w.boolean(false); // `:54781` usedAegisCrystal (ctor `:56078`)
    w.boolean(false); // `:54782` usedAegisFruit (`:56079`)
    w.boolean(false); // `:54783` usedArcaneCrystal (`:56080`)
    w.boolean(false); // `:54784` usedGalaxyPearl (`:56081`)
    w.boolean(false); // `:54785` usedGummyWorm (`:56082`)
    w.boolean(false); // `:54786` usedAmbrosia (`:56083`)
    w.boolean(false); // `:54787` downedDD2EventAnyDifficulty
    w.i32(0); // `:54788` taxMoney
    w.i32(0); // `:54789` numberOfDeathsPVE
    w.i32(0); // `:54790` numberOfDeathsPVP
    for color in [
        file.hair_color, // `:54791-54793`
        file.skin_color, // `:54794-54796`
        file.eye_color, // `:54797-54799`
        file.shirt_color, // `:54800-54802`
        file.undershirt_color, // `:54803-54805`
        file.pants_color, // `:54806-54808`
        file.shoe_color, // `:54809-54811`
    ] {
        for b in color {
            w.u8(b);
        }
    }
    // armor[20]: type, prefix, favorited (`:54812-54817`) — no stack. Fresh
    // armor is all `new Item()` (type 0, prefix 0, favorited false); the
    // favourited bool is unconditional at 326 (reader gate >= 322, `:55313`).
    for _ in 0..20 {
        w.i32(0);
        w.u8(0);
        w.boolean(false);
    }
    // dye[10]: type, prefix, favorited (`:54818-54823`).
    for _ in 0..10 {
        w.i32(0);
        w.u8(0);
        w.boolean(false);
    }
    // inventory[58]: type, stack, prefix, favorited (`:54824-54830`).
    for (i, (id, stack, prefix)) in file.inventory.iter().enumerate() {
        w.i32(*id);
        w.i32(*stack);
        w.u8(*prefix);
        w.boolean(file.inv_favorites & (1 << i) != 0);
    }
    // miscEquips[5] + miscDyes[5] interleaved: type, prefix each
    // (`:54831-54837`) — no stack, no favorited. Fresh `Item()`s are air
    // (ctor `:56064-56071`).
    for _ in 0..5 {
        w.i32(0);
        w.u8(0);
        w.i32(0);
        w.u8(0);
    }
    // bank / bank2 / bank3: 40 x (type, stack, prefix) each
    // (`:54838-54855`); `Chest.CreateBank`'s maxItems = 40 (`Chest.cs:142-144`,
    // `:191`), filled with empty `Item`s (ctor `:56056-56058`).
    for _ in 0..3 {
        for _ in 0..40 {
            w.i32(0);
            w.i32(0);
            w.u8(0);
        }
    }
    // bank4 (the void bag): 40 x (type, stack, prefix, favorited)
    // (`:54856-54862`); the favourited bool reads at >= 255 (`:55495`).
    for _ in 0..40 {
        w.i32(0);
        w.i32(0);
        w.u8(0);
        w.boolean(false);
    }
    // voidVaultInfo: one BitsByte (`:54863`), default 0 (`Player.cs:1590`);
    // read at >= 199 (`Deserialize:55501-55504`).
    w.u8(0);
    // S1-J-5 increment 1 ends here. The `Serialize` tail (`:54864-54933`:
    // buffs, spawn points, hbLocked, hideInfo, angler, DpadRadial,
    // builderAccStatus, bartender, dead, lastTimePlayerWasSaved, golfer,
    // creativeTracker, temp slots, creative powers, super cart, loadouts,
    // voice, refunds, dialogues) is the next round.
    w.into_inner()
}

/// `PlayerFileData.CreateAndSave` (`Player.cs:54560`): the write to
/// `Players/<name>.plr`.
/// The favourite marker beside a character's `.plr`: `<stem>.trfav` in the
/// same directory — the port's stand-in for `Main.LocalFavoriteData`
/// (`Main.cs:66548`), keyed by the file name like `IO/FavoritesFile.cs:34`.
/// The same carrier the world uses (`ui/save_list.rs:219`), so the star never
/// rides a file the engine re-writes wholesale (`writer` below rebuilds the
/// metadata from defaults, which is why the in-file bit could not survive).
pub fn favorite_marker_path(path: &Path) -> PathBuf {
    path.with_extension("trfav")
}

/// `FavoritesFile.IsFavorite` (`IO/FavoritesFile.cs:48`) for a `.plr`.
pub fn is_favorite_marker(path: &Path) -> bool {
    favorite_marker_path(path).exists()
}

/// `FileData.SetFavorite` -> `FavoritesFile.SaveFavorite`
/// (`IO/FileData.cs:48-54`, `IO/FavoritesFile.cs:27-36`): the click saves the
/// store at once, outside the character file. A zero-byte marker, exactly like
/// the world's (`app/flow.rs:904-911`).
pub fn set_favorite_marker(path: &Path, favorite: bool) -> std::io::Result<()> {
    let marker = favorite_marker_path(path);
    if favorite {
        return std::fs::write(&marker, b"");
    }
    match std::fs::remove_file(&marker) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e),
        _ => Ok(()),
    }
}

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

    // --- S1-J-3: the C#-written .plr fixture reads ----------------------------
    // `assets/test/golden.plr` is written by the REAL decompiled
    // `Player.Serialize` (Player.cs:54750+) through `tools/WldFixtureHarness`,
    // with the real AES-CBC envelope. The fixture exposed FILE_TYPE_PLAYER=1
    // (Map) where the C# enum says Player=3 — the reader rejected every real
    // vanilla character. Read parity only; the .plr writer stays S1-J-5's.
    #[test]
    fn s1_j_3_the_csharp_plr_fixture_reads() {
        let raw = include_bytes!("../../../assets/test/golden.plr");
        let plain = cbc_decrypt(raw);
        let version = i32::from_le_bytes(plain[0..4].try_into().unwrap());
        assert_eq!(version, 326);
        let magic = u64::from_le_bytes(plain[4..12].try_into().unwrap());
        assert_eq!((magic >> 56) & 0xFF, 3, "FileType.Player (FileType.cs:8)");
        let file = read_player_bytes(&plain).expect("the C# .plr fixture must read");
        assert_eq!(file.name, "Golden");
        assert_eq!(file.difficulty, 0);
        assert_eq!(file.hair, 1);
        assert_eq!((file.life, file.life_max), (100, 100));
        assert_eq!((file.mana, file.mana_max), (20, 20));
        assert_eq!(file.inventory[0], (3507, 1, 0)); // copper shortsword
        assert_eq!(file.inventory[1], (3509, 1, 0)); // copper axe
        assert_eq!(file.inventory[2], (3506, 1, 0)); // copper pickaxe
        assert_eq!(file.inventory[3], (2, 50, 0));
        assert_eq!(file.inv_favorites, 1 << 3);
        assert_eq!(file.hair_color, [215, 90, 55]);
        assert_eq!(file.skin_color, [255, 125, 90]);
    }

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
            inv_favorites: 1 << 0 | 1 << 2, // slots 0 and 2 starred
        }
    }

    #[test]
    fn round_trip_through_the_crypto_and_the_sections() {
        // The write -> the read: the name/stats/inventory round-trip through
        // the AES-CBC layer and the release-326 section walk.
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
        assert_eq!(
            file.inv_favorites,
            1 << 0 | 1 << 2,
            "the per-slot favourited bits ride the release-326 inventory walk"
        );
    }

    /// `Item.favorited`'s bitfield round-trips (`Player.cs:54820-54824`'s
    /// write order — type, stack, prefix, favorited): toggling two slots,
    /// saving and loading leaves the same two starred (BUG-03). Before this
    /// the writer emitted a hardcoded `false` and the reader dropped it, so
    /// every reload lost the stars.
    #[test]
    fn the_favorited_bits_survive_a_save_and_a_load() {
        // The App-side toggle pair through the flattening helper.
        let bits = {
            let file = sample();
            // slot 1 and slot 57 starred, the other two of the sample unstarred.
            (1u64 << 1) | (1u64 << 57)
        };
        let _ = bits;
        let written = PlayerFileWrite {
            inv_favorites: (1 << 1) | (1 << 57),
            ..sample()
        };
        let bytes = write_player_bytes(&written);
        let file = match read_player_bytes(&cbc_decrypt(&bytes)) { Ok(f) => f, Err(e) => panic!("read err: {e:?}") };
        assert_eq!(file.inv_favorites, (1 << 1) | (1 << 57));
        // The set bit lands on the slot the C# reads it for.
        assert_eq!(file.inventory[1], (3, 50, 0));
        // The empty slots read false — their favourited byte is consumed with
        // the rest of the triple (`:55363`).
        assert_eq!(file.inventory[30], (0, 0, 0));
    }

    /// The 322+ armour/dye favourited bytes are version-gated on read: a
    /// vanilla file below 322 carries no favourited bool per armour/dye pair,
    /// a >= 322 one does — the walk must stay aligned either way.
    #[test]
    fn the_armor_and_dye_favorites_are_version_gated() {
        // The port's own release-326 file carries the 30 favourited bytes.
        let bytes = write_player_bytes(&sample());
        let plain = cbc_decrypt(&bytes);
        assert_eq!(i32::from_le_bytes(plain[..4].try_into().unwrap()), 326);
        let file = match read_player_bytes(&plain) { Ok(f) => f, Err(e) => panic!("read err: {e:?}") };
        assert_eq!(file.inventory[1], (3, 50, 0));
        // The same body restamped at 321 (below the favourite gate) with the
        // 30 armour/dye favourited bytes stripped must read to the same state.
        let mut r = Reader::new(&plain);
        r.skip(24).unwrap(); // version + FileMetadata
        let name = r.cs_string().unwrap();
        let mut w = Writer::new();
        w.i32(321);
        w.bytes(&plain[4..24]);
        w.cs_string(&name);
        // difficulty..hairDye: 1 + 8 + 4 + 1 = 14 bytes; then the 283+ team
        // byte is absent at 321? No — 321 >= 283, so keep it: 15 bytes.
        let head = r.take(15).unwrap();
        w.bytes(head);
        // hide BitsBytes (2) + hideMisc (1) + skinVariant (1) + stats (16) +
        // the unlock block: 12 bools incl. the 324 slot? 321 < 324 — strip it.
        w.bytes(r.take(2 + 1 + 1 + 16).unwrap());
        // extra, bt1, bt2, bread (4 bools) — then the 324 dead bool is absent
        // at 321, then the six unlocks + dd2 (7 bools).
        w.bytes(r.take(4).unwrap());
        r.u8().unwrap(); // drop the 324 dead bool
        w.bytes(r.take(7).unwrap());
        // tax + deaths + colours: 12 + 21 bytes.
        w.bytes(r.take(12 + 21).unwrap());
        // armour 20 x (id, prefix[, fav]) + dye 10 x (id, prefix[, fav]):
        // strip the favourited byte of each pair.
        for _ in 0..30 {
            w.bytes(r.take(5).unwrap());
            r.u8().unwrap();
        }
        // The rest (inventory onward) is identical.
        w.bytes(r.take(plain.len() - r.pos()).unwrap());
        let lo = w.into_inner();
        let file321 = match read_player_bytes(&lo) { Ok(f) => f, Err(e) => panic!("read err: {e:?}") };
        assert_eq!(file321.name, file.name);
        assert_eq!(file321.inventory, file.inventory);
        assert_eq!(file321.inv_favorites, file.inv_favorites);
    }

    /// S1-J-5 golden layout: the plaintext through `voidVaultInfo` matches the
    /// C# `Serialize` field order byte-for-byte at the documented offsets
    /// (`Player.cs:54738-54863`), with every unmodelled field at its fresh
    /// default. The inventory is the sample's; everything else is pinned.
    #[test]
    fn golden_layout_through_void_vault_info() {
        let plain = serialize_plaintext(&sample());
        // prologue: i32 release, u64 magic, u32 revision, u64 favorites.
        assert_eq!(&plain[0..4], &326i32.to_le_bytes()); // `:54738`
        assert_eq!(
            u64::from_le_bytes(plain[4..12].try_into().unwrap()),
            FILE_METADATA_MAGIC | (FILE_TYPE_PLAYER << 56)
        );
        assert_eq!(&plain[12..16], &[0; 4]);
        assert_eq!(&plain[16..24], &[0; 8]);
        let mut r = Reader::new(&plain);
        r.skip(24).unwrap();
        assert_eq!(r.cs_string().unwrap(), "Belsin"); // `:54752`
        assert_eq!(r.u8().unwrap(), 0); // difficulty `:54753`
        assert_eq!(r.u64().unwrap(), 0); // playtime `:54754`
        assert_eq!(r.i32().unwrap(), 0); // hair `:54755`
        assert_eq!(r.u8().unwrap(), 0); // hairDye `:54756`
        assert_eq!(r.u8().unwrap(), 0); // team `:54757`
        assert_eq!(r.u8().unwrap(), 0); // hideVisible[0..8] `:54758-54763`
        assert_eq!(r.u8().unwrap(), 0); // hideVisible[8..10] `:54764-54769`
        assert_eq!(r.u8().unwrap(), 0); // hideMisc `:54770`
        assert_eq!(r.u8().unwrap(), 0); // skinVariant `:54771`
        assert_eq!(r.i32().unwrap(), 100); // statLife `:54772`
        assert_eq!(r.i32().unwrap(), 100); // statLifeMax `:54773`
        assert_eq!(r.i32().unwrap(), 20); // statMana `:54774`
        assert_eq!(r.i32().unwrap(), 20); // statManaMax `:54775`
        for _ in 0..12 {
            // `:54776-54787`: extra, torch pair, bread, dead-324, 6 unlocks, dd2.
            assert_eq!(r.u8().unwrap(), 0);
        }
        assert_eq!(r.i32().unwrap(), 0); // taxMoney `:54788`
        assert_eq!(r.i32().unwrap(), 0); // deathsPVE `:54789`
        assert_eq!(r.i32().unwrap(), 0); // deathsPVP `:54790`
        for expected in [
            [215, 90, 55],
            [255, 125, 90],
            [105, 90, 75],
            [175, 165, 140],
            [160, 180, 215],
            [255, 230, 175],
            [160, 105, 60],
        ] {
            assert_eq!(r.take(3).unwrap(), expected); // `:54791-54811`
        }
        for _ in 0..20 {
            // armor `:54812-54817`: (0, 0, false).
            assert_eq!(r.i32().unwrap(), 0);
            assert_eq!(r.u8().unwrap(), 0);
            assert_eq!(r.u8().unwrap(), 0);
        }
        for _ in 0..10 {
            // dye `:54818-54823`.
            assert_eq!(r.i32().unwrap(), 0);
            assert_eq!(r.u8().unwrap(), 0);
            assert_eq!(r.u8().unwrap(), 0);
        }
        let sample = sample();
        for (i, (id, stack, prefix)) in sample.inventory.iter().enumerate() {
            // inventory `:54824-54830`.
            assert_eq!(r.i32().unwrap(), *id);
            assert_eq!(r.i32().unwrap(), *stack);
            assert_eq!(r.u8().unwrap(), *prefix);
            assert_eq!(r.u8().unwrap(), u8::from(sample.inv_favorites & (1 << i) != 0));
        }
        for _ in 0..5 {
            // miscEquips/miscDyes `:54831-54837`: (0, 0, 0, 0).
            assert_eq!(r.take(10).unwrap(), &[0; 10]);
        }
        // banks 1-3 `:54838-54855`: 120 x (0, 0, 0) = 1080 zero bytes.
        assert_eq!(r.take(1080).unwrap(), vec![0u8; 1080].as_slice());
        for _ in 0..40 {
            // bank4 `:54856-54862`: (0, 0, 0, false) = 10 zero bytes.
            assert_eq!(r.take(10).unwrap(), &[0; 10]);
        }
        assert_eq!(r.u8().unwrap(), 0); // voidVaultInfo `:54863`
        // Increment 1 ends exactly at the vault byte — the Serialize tail is
        // the next round, so nothing follows.
        assert_eq!(r.pos(), plain.len());
    }

    /// The port's own pre-S1-J-5 subset files (release 279, truncated after
    /// the inventory, no team/hideMisc/324-bool/armour-favourites) still load
    /// through [`read_legacy_port_subset`].
    #[test]
    fn the_legacy_port_subset_still_loads() {
        // Rebuild the old writer's exact byte layout (the pre-S1-J-5 shape).
        let s = sample();
        let mut w = Writer::new();
        w.i32(279);
        w.u64(FILE_METADATA_MAGIC | (FILE_TYPE_PLAYER << 56));
        w.u32(0);
        w.u64(0);
        w.cs_string(&s.name);
        w.u8(s.difficulty);
        w.u64(0);
        w.i32(s.hair);
        w.u8(0); // hairDye — no team byte at the old shape
        w.bytes(&[0, 0]); // hide BitsBytes — no hideMisc
        w.u8(s.skin_variant);
        w.i32(s.life);
        w.i32(s.life_max);
        w.i32(s.mana);
        w.i32(s.mana_max);
        for _ in 0..11 {
            w.boolean(false); // extra, torch pair, bread, 6 unlocks, dd2
        }
        w.i32(0);
        w.i32(0);
        w.i32(0);
        for color in [
            s.hair_color,
            s.skin_color,
            s.eye_color,
            s.shirt_color,
            s.undershirt_color,
            s.pants_color,
            s.shoe_color,
        ] {
            w.bytes(&color);
        }
        for _ in 0..30 {
            w.i32(0); // armour + dye pairs, no favourited bool
            w.u8(0);
        }
        for (i, (id, stack, prefix)) in s.inventory.iter().enumerate() {
            w.i32(*id);
            w.i32(*stack);
            w.u8(*prefix);
            w.boolean(s.inv_favorites & (1 << i) != 0);
        }
        let file = match read_player_bytes(&w.into_inner()) { Ok(f) => f, Err(e) => panic!("legacy read err: {e:?}") };
        assert_eq!(file.name, "Belsin");
        assert_eq!(file.inventory, s.inventory);
        assert_eq!(file.inv_favorites, s.inv_favorites);
        assert_eq!(file.hair_color, s.hair_color);
        assert_eq!(file.shoe_color, s.shoe_color);
        assert_eq!(file.life_max, 100);
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
