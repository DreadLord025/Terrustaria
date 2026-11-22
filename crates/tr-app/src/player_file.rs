//! The `.plr` character file reader/writer — `Player.LoadPlayer`/
//! `Player.SavePlayer` (`Player.cs:55058`, `:54736`), stamped release **326**
//! (`InternalSavePlayerFile` writes the literal 326, `Player.cs:54738`).
//!
//! The vanilla files are AES-128-CBC encrypted with the fixed key
//! `"h3y_gUyZ"` in UTF-16-LE (`Player.cs:796-798` — the key doubles as the
//! IV). `InternalSavePlayerFile` uses a bare `new RijndaelManaged()`
//! (`Player.cs:54734-54736`) — its default `Padding` is **PKCS7**, so
//! `cryptoStream.FlushFinalBlock()` (`:54742`) appends 1..16 bytes of value
//! `16 - len % 16` (a full block when already aligned). The `PaddingMode.None`
//! at `Player.cs:55074` is the *read* path only (it decrypts whole blocks and
//! lets `Deserialize` stop at the logical field end). S1-J-5 inc3 ports the
//! write-side PKCS7; the reader still ignores the padding like the C# does.
//! The port writes the same section order so vanilla save editors and the
//! vanilla client can read the port's characters.
//!
//! **S1-J-5 increment 3: the full `Serialize`.** `Player.Serialize`
//! (`Player.cs:54750-54933`) is now ported end-to-end: through `hideInfo`
//! (`:54893`, increments 1-2) plus the tail (`:54894-54933`) — angler
//! quests, the DpadRadial bindings, `builderAccStatus[12]`, the bartender
//! log, `dead` (+ the conditional respawn timer), the save timestamp, the
//! golfer score, the creative tracker, the temporary item slots, the
//! per-player creative powers, the super-cart BitsByte, the loadout index +
//! the three `EquipmentLoadout`s, the voice pair, the crafting refunds and
//! the one-time dialogues. The only unowned field is the wall-clock
//! `DateTime.UtcNow.ToBinary()` (`:54909`), carried on
//! [`PlayerFileWrite::last_saved_binary`] exactly like the `.wld` writer's
//! CreationTime policy (ANALYSIS "The C#-written golden fixture").
//!
//! **Default-value policy.** The port models only the character-card state
//! (name, difficulty, appearance, life/mana, the 58-slot inventory and its
//! favourited bits). Every other field written so far is emitted with the
//! value a *fresh vanilla player* carries — the `Player` ctor
//! (`Player.cs:56036-56096`) and the field initializers — never omitted:
//! omission shifts every later section and breaks byte-comparability. The
//! reader walks the same span with the C# `Deserialize` version gates
//! (`Player.cs:55134-55764`), consuming (and dropping) unmodelled fields.
//!
//! **Backward compat.** The port's own pre-S1-J-5 files were a *subset* mirror
//! stamped release 279 (truncated after the inventory, missing `team` /
//! `hideMisc` / the 324 bool / the armour-dye favourites), and its
//! increment-1 files end at `voidVaultInfo` with no buffs tail. Neither is a
//! valid vanilla file at its stamp, so the C# gates cannot read them;
//! [`read_legacy_port_subset`] and the increment-1/2 truncation retries in
//! [`read_player_bytes`] keep them loadable as documented port-only shims
//! (see `docs/ANALYSIS.md`).

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
/// emitted span currently runs through `hideInfo` (`Serialize:54893`).
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
/// walks `Deserialize` (`Player.cs:55134`) through `hideInfo`
/// (`:55616-55623`) with the C# version gates, consuming and dropping every
/// field the port does not model; the tail past `:55623` is a later round.
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
    /// `Player.creativeTracker.ItemSacrifices`' `(persistentId, count)` pairs
    /// (`ItemsSacrificedUnlocksTracker.cs:157-169`), the Journey research the
    /// `.plr` carries (`Player.cs:54911`/`:55696-55699`). The W4-f44 seam's
    /// shape — the tracker hydrates through `research::load_pairs`.
    pub research: Vec<(String, i32)>,
    /// The Journey per-player creative powers (`CreativePowerManager
    /// .LoadToPlayer`, `CreativePowerManager.cs:197-215`): Godmode (id 5),
    /// FarPlacementRange (id 11), SpawnRate (id 14). The W4-f44 seam's
    /// shape — the App hydrates them on Play.
    pub powers: CreativePowersSave,
}

/// The Journey per-player creative powers (`CreativePowerManager
/// .LoadToPlayer`, `CreativePowerManager.cs:197-215`): `GodmodePower` id 5
/// (`CreativePowers.cs:692-728`), `FarPlacementRangePower` id 11
/// (`:730-767`), `SpawnRateSliderPerPlayerPower` id 14 (`:1573-1737`).
/// The W4-f44 seam's shape.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CreativePowersSave {
    pub godmode: bool,
    pub far_placement: bool,
    /// The raw [0,1] `_sliderCurrentValueCache` (`:1719`).
    pub spawn_rate: f32,
}

impl Default for CreativePowersSave {
    /// The read-side reset state (`CreativePowerManager.cs:211-214` →
    /// `ResetDataForNewPlayer`): Godmode false, FarPlacement true
    /// (`CreativePowers.cs:736`), SpawnRate 0.5 (`:1580`'s
    /// `_sliderDefaultValue`).
    fn default() -> Self {
        CreativePowersSave { godmode: false, far_placement: true, spawn_rate: 0.5 }
    }
}

impl CreativePowersSave {
    /// The *writer's* fresh default — the live manager state a Classic
    /// session saves (`CreativePowers.cs:707`/`:746`/`:1719`'s
    /// `_sliderCurrentValueCache` at its ctor 0, not the reset default).
    /// The W4-f44 seam's `written_fresh`.
    pub fn written_fresh() -> Self {
        CreativePowersSave { godmode: false, far_placement: false, spawn_rate: 0.0 }
    }
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

/// The AES-128-CBC decrypt with the vanilla key/IV — the read-side envelope
/// (`Player.cs:55073-55076`): `Padding = PaddingMode.None`, whole blocks
/// only; any trailing PKCS7 padding stays in the output and is simply never
/// read (`Deserialize` stops at the logical field end).
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

/// The AES-128-CBC encrypt with **PKCS7** padding — the write-side envelope
/// of `InternalSavePlayerFile` (`Player.cs:54734-54742`): a bare
/// `RijndaelManaged` defaults to `PaddingMode.PKCS7`, and
/// `cryptoStream.FlushFinalBlock()` appends `n = 16 - len % 16` bytes of
/// value `n` (a FULL extra block when the plaintext is already aligned).
/// The read path's `PaddingMode.None` (`:55074`) never strips anything —
/// `Deserialize` stops at the logical field end — so [`cbc_decrypt`] leaves
/// the padding in place, exactly like the C# reader.
pub fn cbc_encrypt(data: &[u8]) -> Vec<u8> {
    let cipher = Aes128::new((&ENCRYPTION_KEY).into());
    let pad = (16 - data.len() % 16) as u8; // 1..=16, never 0 (PKCS7)
    let mut padded = data.to_vec();
    padded.resize(data.len() + pad as usize, pad);
    let mut out = vec![0u8; padded.len()];
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
    let type_byte = (magic >> 56) & 0xFF;
    if type_byte != FILE_TYPE_PLAYER {
        // Pre-S1-J-3 port saves stamped the type byte 1 (Map) by mistake;
        // accept that ONLY for the port's own legacy release 279 so old
        // dev characters still load. Vanilla files (and the corrected
        // writer) stamp 3 (FileType.cs:8); anything else is WrongFileType.
        if !(type_byte == 1 && version == LEGACY_PORT_RELEASE) {
            return Err(PlayerFileError::WrongFileType);
        }
    }
    let revision = r.u32().ok_or(PlayerFileError::Truncated("revision"))?;
    let favorite_bits = r.u64().ok_or(PlayerFileError::Truncated("fav"))?;
    let is_favorite = favorite_bits == 1;
    match read_player_fields(&mut r, version, revision, is_favorite, 3) {
        Ok(f) => Ok(f),
        // The port's own pre-S1-J-5 subset files are stamped 279 but are
        // truncated after the inventory and miss `hideMisc`; the C# gates
        // cannot read them. Retry with the port-only shim so old saves load.
        Err(PlayerFileError::Truncated(_)) if version == LEGACY_PORT_RELEASE => {
            let mut r2 = Reader::new(plain);
            r2.skip(4 + 20)
                .ok_or(PlayerFileError::Truncated("legacy"))?;
            read_legacy_port_subset(&mut r2, revision, is_favorite)
        }
        // The port's own S1-J-5 increment-1/2 files are stamped 326 but end
        // early: inc1 at `voidVaultInfo` (`Serialize:54863`), inc2 at
        // `hideInfo` (`:54893`). The C# `Deserialize` would run off the end;
        // retry at each shallower depth so those dev characters still load
        // (port-only shims, same pattern as the 279 one above — see
        // `docs/ANALYSIS.md`).
        Err(PlayerFileError::Truncated(_)) if version == RELEASE => {
            let mut r2 = Reader::new(plain);
            r2.skip(4 + 20).ok_or(PlayerFileError::Truncated("inc1"))?;
            match read_player_fields(&mut r2, version, revision, is_favorite, 1) {
                Ok(f) => Ok(f),
                Err(PlayerFileError::Truncated(_)) => {
                    let mut r3 = Reader::new(plain);
                    r3.skip(4 + 20).ok_or(PlayerFileError::Truncated("inc2"))?;
                    read_player_fields(&mut r3, version, revision, is_favorite, 2)
                }
                Err(e) => Err(e),
            }
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
/// `depth` is the port-only truncation-retry selector for the port's own
/// pre-inc3 files — never below 3 for genuine vanilla files:
/// 1 = stop at `voidVaultInfo` (`:55504`, the inc1 files),
/// 2 = stop at `hideInfo` (`:55623`, the inc2 files),
/// 3 = the full `Deserialize` walk through the dialogues (`:55762`).
fn read_player_fields(
    r: &mut Reader,
    version: i32,
    revision: u32,
    is_favorite: bool,
    depth: u8,
) -> Result<PlayerFile, PlayerFileError> {
    if version > 326 {
        return Err(PlayerFileError::LaterVersion(version));
    }
    // The Journey research pairs (`Player.cs:55696-55699`'s creativeTracker
    // block) — collected here, carried on the PlayerFile below.
    let mut research: Vec<(String, i32)> = Vec::new();
    // The per-player creative powers (`CreativePowerManager.LoadToPlayer`,
    // `:55706-55709`) — the W4-f44 seam's `powers` field.
    let mut powers = CreativePowersSave::default();
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
        let _void_vault = r.u8().ok_or(PlayerFileError::Truncated("voidVault"))?;
        // `:55501-55504`
    }
    if depth <= 1 {
        // The port-only retry for increment-1 files (see `read_player_bytes`).
        return Ok(finish_player_file(
            name,
            difficulty,
            skin_variant,
            hair,
            life,
            life_max,
            mana,
            mana_max,
            colors,
            inventory,
            inv_favorites,
            is_favorite,
            revision,
            research,
            powers,
        ));
    }
    // --- S1-J-5 increment 2: buffs through hideInfo (`:55578-55623`) ------
    // buffs (`:55578-55599`, gate 11): the slot count is 10 below 74, 22,
    // and 44 at >= 252 (`Player.maxBuffs`, `Player.cs:57106`). The zero-type
    // compaction (`num28--; num27--`, `:55593-55597`) still consumes exactly
    // `count` (type, time) pairs — each iteration advances either the read
    // index or shrinks the target. The port models no buffs; drop them.
    if version >= 11 {
        let mut count: i32 = if version < 74 { 10 } else { 22 };
        if version >= 252 {
            count = 44;
        }
        let mut i: i32 = 0;
        while i < count {
            let buff_type = r.i32().ok_or(PlayerFileError::Truncated("buffType"))?;
            r.i32().ok_or(PlayerFileError::Truncated("buffTime"))?;
            if buff_type == 0 {
                i -= 1;
                count -= 1;
            }
            i += 1;
        }
    }
    // spawn points (`:55600-55611`, ungated): up to 200 records, each
    // (spX, spY, spI, spN); spX == -1 is the end sentinel.
    for _ in 0..200 {
        let sp_x = r.i32().ok_or(PlayerFileError::Truncated("spawnX"))?;
        if sp_x == -1 {
            break;
        }
        r.i32().ok_or(PlayerFileError::Truncated("spawnY"))?;
        r.i32().ok_or(PlayerFileError::Truncated("spawnI"))?;
        r.cs_string().ok_or(PlayerFileError::Truncated("spawnN"))?;
    }
    if version >= 16 {
        r.boolean().ok_or(PlayerFileError::Truncated("hbLocked"))?; // `:55612-55615`
    }
    if version >= 115 {
        for _ in 0..13 {
            r.boolean().ok_or(PlayerFileError::Truncated("hideInfo"))?; // `:55616-55623`
        }
    }
    if depth <= 2 {
        // The port-only retry for increment-2 files (see `read_player_bytes`).
        return Ok(finish_player_file(
            name,
            difficulty,
            skin_variant,
            hair,
            life,
            life_max,
            mana,
            mana_max,
            colors,
            inventory,
            inv_favorites,
            is_favorite,
            revision,
            research,
            powers,
        ));
    }
    // --- S1-J-5 increment 3: the Deserialize tail (`:55624-55762`) --------
    if version >= 98 {
        r.i32().ok_or(PlayerFileError::Truncated("angler"))?; // `:55624-55627`
    }
    if version >= 162 {
        for _ in 0..4 {
            r.i32().ok_or(PlayerFileError::Truncated("dpad"))?; // `:55628-55634`
        }
    }
    if version >= 164 {
        // The builderAccStatus count gate (`:55635-55653`): 8, then 10 at
        // 167, 11 at 197, 12 at 230. The < 210 / < 249 fixups (`:55654-55673`)
        // mutate state only — no extra bytes.
        let mut count = 8;
        if version >= 167 {
            count = 10;
        }
        if version >= 197 {
            count = 11;
        }
        if version >= 230 {
            count = 12;
        }
        for _ in 0..count {
            r.i32().ok_or(PlayerFileError::Truncated("builderAcc"))?;
        }
    }
    if version >= 181 {
        r.i32().ok_or(PlayerFileError::Truncated("bartender"))?; // `:55675-55678`
    }
    if version >= 200 {
        // dead + the conditional respawnTimer (`:55679-55686`).
        if r.boolean().ok_or(PlayerFileError::Truncated("dead"))? {
            r.i32().ok_or(PlayerFileError::Truncated("respawn"))?;
        }
    }
    if version >= 202 {
        r.u64().ok_or(PlayerFileError::Truncated("lastSaved"))?; // `:55688-55691`
    }
    if version >= 206 {
        r.i32().ok_or(PlayerFileError::Truncated("golfer"))?; // `:55692-55695`
    }
    if version >= 218 {
        // creativeTracker.Load -> CreativeUnlocksTracker.Load
        // (`CreativeUnlocksTracker.cs:14-17`) -> ItemsSacrificedUnlocksTracker
        // .Load (`ItemsSacrificedUnlocksTracker.cs:171-186`): the >= 282 bool
        // then the (string, i32) sacrifice pairs — the Journey research
        // counts (`Player.cs:55696-55699`). The pairs ride the PlayerFile
        // (the W4-f44 seam's `research` field); the tracker hydrates from
        // them on Play (`research::load_pairs`).
        if version >= 282 {
            r.boolean().ok_or(PlayerFileError::Truncated("sacrFlag"))?;
        }
        let n = r.i32().ok_or(PlayerFileError::Truncated("sacrCount"))?;
        for _ in 0..n {
            let key = r.cs_string().ok_or(PlayerFileError::Truncated("sacrKey"))?;
            let val = r.i32().ok_or(PlayerFileError::Truncated("sacrVal"))?;
            research.push((key, val));
        }
    }
    if version >= 214 {
        // LoadTemporaryItemSlotContents (`Player.cs:54962-54985`): one
        // BitsByte, then an Item.DeserializeFrom triple per set bit 0..3.
        let bits = r.u8().ok_or(PlayerFileError::Truncated("tempSlots"))?;
        for i in 0..4 {
            if bits & (1 << i) != 0 {
                read_item_triple(r, "tempItem")?;
            }
        }
    }
    if version >= 220 {
        // CreativePowerManager.LoadToPlayer (`CreativePowerManager.cs:197-215`):
        // while(bool) { u16 id; the power's Load }. The three per-player
        // powers: Godmode (5) / FarPlacementRange (11) read one bool
        // (`CreativePowers.cs:705-707`/`:744-746` shape), SpawnRate (14) one
        // f32 (`:1717-1719` shape). An unknown id BREAKS the loop in the C#
        // (`:203-206`) — mirror that instead of guessing a payload size.
        // The values ride the PlayerFile's `powers` field (the W4-f44 seam);
        // the App hydrates them on Play.
        loop {
            let more = r.boolean().ok_or(PlayerFileError::Truncated("powerFlag"))?;
            if !more {
                break;
            }
            let id = r.u16().ok_or(PlayerFileError::Truncated("powerId"))?;
            match id {
                5 => {
                    powers.godmode = r.boolean().ok_or(PlayerFileError::Truncated("powerBool"))?;
                }
                11 => {
                    powers.far_placement =
                        r.boolean().ok_or(PlayerFileError::Truncated("powerBool"))?;
                }
                14 => {
                    powers.spawn_rate =
                        r.f32().ok_or(PlayerFileError::Truncated("powerSlider"))?;
                }
                _ => break, // `CreativePowerManager.cs:203-206`
            }
        }
    }
    if version >= 253 {
        r.u8().ok_or(PlayerFileError::Truncated("superCart"))?; // `:55710-55715`
    }
    if version >= 262 {
        // CurrentLoadoutIndex + the three loadouts (`:55720-55727`);
        // EquipmentLoadout.Deserialize (`EquipmentLoadout.cs:47-61`):
        // 20 armor + 10 dye x (type, stack, prefix[, favorited >= 322])
        // then Hide[10] bools.
        r.i32().ok_or(PlayerFileError::Truncated("loadoutIdx"))?;
        for _ in 0..3 {
            for _ in 0..30 {
                read_item_triple(r, "loadoutItem")?;
                if version >= 322 {
                    r.boolean()
                        .ok_or(PlayerFileError::Truncated("loadoutFav"))?;
                }
            }
            for _ in 0..10 {
                r.boolean()
                    .ok_or(PlayerFileError::Truncated("loadoutHide"))?;
            }
        }
    }
    if version >= 280 {
        r.u8().ok_or(PlayerFileError::Truncated("voice"))?; // `:55728-55731`
    }
    if version >= 281 {
        r.f32().ok_or(PlayerFileError::Truncated("voicePitch"))?; // `:55738-55741`
    }
    if version >= 300 {
        // _pendingRefunds (`:55746-55754`): count + Item.DeserializeFrom triples.
        let n = r.i32().ok_or(PlayerFileError::Truncated("refundCount"))?;
        for _ in 0..n {
            read_item_triple(r, "refundItem")?;
        }
    }
    if version >= 310 {
        // oneTimeDialoguesSeen (`:55755-55762`): count + strings.
        let n = r.i32().ok_or(PlayerFileError::Truncated("dialogueCount"))?;
        for _ in 0..n {
            r.cs_string()
                .ok_or(PlayerFileError::Truncated("dialogue"))?;
        }
    }
    // `LoadPlayer_LastMinuteFixes` (`:55830`) clamps skinVariant to the
    // variant set (done in `finish_player_file`).
    Ok(finish_player_file(
        name,
        difficulty,
        skin_variant,
        hair,
        life,
        life_max,
        mana,
        mana_max,
        colors,
        inventory,
        inv_favorites,
        is_favorite,
        revision,
        research,
        powers,
    ))
}

/// The shared tail of both walk depths: the `LoadPlayer_LastMinuteFixes`
/// skinVariant clamp (`Player.cs:55830`) and the `PlayerFile` assembly.
#[allow(clippy::too_many_arguments)]
fn finish_player_file(
    name: String,
    difficulty: u8,
    skin_variant: u8,
    hair: i32,
    life: i32,
    life_max: i32,
    mana: i32,
    mana_max: i32,
    colors: [[u8; 3]; 7],
    inventory: [(i32, i32, u8); 58],
    inv_favorites: u64,
    is_favorite: bool,
    revision: u32,
    research: Vec<(String, i32)>,
    powers: CreativePowersSave,
) -> PlayerFile {
    let skin_variant = (skin_variant as i32).clamp(0, PLAYER_VARIANT_COUNT - 1) as u8;
    // `CreativePowerManager.LoadToPlayer` (`CreativePowerManager.cs:211-214`):
    // after the load the C# RESETS the powers for non-Journey characters
    // (`difficulty != 3` → `ResetDataForNewPlayer`, the ctor defaults).
    let powers = if difficulty != 3 { CreativePowersSave::default() } else { powers };
    PlayerFile {
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
        research,
        powers,
    }
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
        // The legacy subset stops before the creativeTracker block — no
        // research pairs (the port-only shim, ANALYSIS).
        research: Vec::new(),
        powers: CreativePowersSave::default(),
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
    /// `DateTime.UtcNow.ToBinary()` at save time (`Player.cs:54909`) — the
    /// one wall-clock field in `Serialize`. The port does not track a clock,
    /// so the caller stamps it (0 = the port's historical "unknown" stamp);
    /// the golden-fixture test pins the harness's `FixtureClock` value
    /// (2026-11-18T00:00:00Z, kind Utc → kind bits 01, no offset).
    pub last_saved_binary: i64,
    /// The Journey research pairs (`:54911` -> `ItemsSacrificedUnlocksTracker
    /// .Save`, `ItemsSacrificedUnlocksTracker.cs:157-169`): the
    /// `(persistentId, count)` strings the creativeTracker block carries.
    /// The W4-f44 seam's shape — a fresh character has none (the default).
    pub research: Vec<(String, i32)>,
    /// The Journey per-player creative powers (`:54913` ->
    /// `CreativePowerManager.SaveToPlayer`, `CreativePowerManager.cs:183-195`).
    /// Defaults to [`CreativePowersSave::written_fresh`] — the live manager
    /// state a Classic session saves (the W4-f44 seam's writer default).
    pub powers: CreativePowersSave,
}

impl Default for PlayerFileWrite {
    /// The fresh-vanilla defaults (`Player` ctor + the field initializers) —
    /// the W4-f44 seam's default split: the writer's fresh state carries no
    /// research pairs.
    fn default() -> Self {
        Self {
            name: String::new(),
            difficulty: 0,
            skin_variant: 0,
            hair: 0,
            life: 100,
            life_max: 100,
            mana: 20,
            mana_max: 20,
            hair_color: [0; 3],
            skin_color: [0; 3],
            eye_color: [0; 3],
            shirt_color: [0; 3],
            undershirt_color: [0; 3],
            pants_color: [0; 3],
            shoe_color: [0; 3],
            inventory: [(0, 0, 0); 58],
            inv_favorites: 0,
            last_saved_binary: 0,
            research: Vec::new(),
            powers: CreativePowersSave::written_fresh(),
        }
    }
}

/// The `SavePlayer` mirror (`Player.cs:54736-54742`): release 326, the
/// `FileMetadata`, then the FULL `Serialize` (`Player.cs:54750-54933`) —
/// S1-J-5 increment 3. Encrypted like `:54736`'s CryptoStream with the
/// default PKCS7 padding flushed by `FlushFinalBlock` (`:54742`).
pub fn write_player_bytes(file: &PlayerFileWrite) -> Vec<u8> {
    cbc_encrypt(&serialize_plaintext(file))
}

/// The unencrypted release-326 prologue (`InternalSavePlayerFile:54738-54739`:
/// the `i32 326` + `FileMetadata`) plus the FULL `Player.Serialize`
/// (`Player.cs:54750-54933`).
///
/// Every field the port does not model is written at its fresh-vanilla default
/// (the `Player` ctor `:56036-56096` + the field initializers), never omitted —
/// see the module-level default-value policy. The single caller-supplied
/// unmodelled value is the save timestamp ([`PlayerFileWrite::last_saved_binary`]).
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

    // --- Player.Serialize (`Player.cs:54752-54893`) ------------------------
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
    // --- S1-J-5 increment 2: buffs through hideInfo (`:54864-54893`) ------
    // buffs: maxBuffs = 44 (`Player.cs:57106`) x (buffType, buffTime)
    // (`:54864-54876`). Fresh buffType/buffTime are all-zero (`:1510-1512`)
    // and `Main.buffNoSave[0]` is false, so the else arm writes the (0, 0)
    // pair — byte-identical to the no-save arm.
    for _ in 0..44 {
        w.i32(0); // buffType[i]
        w.i32(0); // buffTime[i]
    }
    // spawn points (`:54877-54888`): spN is a fresh null string[200]
    // (`Player.cs:2488`), so the first iteration writes the -1 sentinel
    // and breaks — one i32, no coordinates.
    w.i32(-1);
    // hbLocked (`:54889`): default false (`Player.cs:1436`).
    w.boolean(false);
    // hideInfo[13] (`:54890-54893`): a fresh `bool[13]` (`Player.cs:812`),
    // all false.
    for _ in 0..13 {
        w.boolean(false);
    }
    // --- S1-J-5 increment 3: the Serialize tail (`:54894-54933`) ----------
    w.i32(0); // `:54894` anglerQuestsFinished (`Player.cs:1910`, default 0)
              // `:54895-54898` DpadRadial.Bindings[4] (`Player.cs:800`): the Dpad4
              // ctor sets all four to -1 (`Player.cs:618-621`).
    for _ in 0..4 {
        w.i32(-1);
    }
    // `:54899-54902` builderAccStatus[12] (`Player.cs:814`,
    // `BuilderAccToggleIDs.Count = 12`, `Player.cs:47-75`): a fresh int[12]
    // with the ctor's `builderAccStatus[0] = 1` (`:56092`, the ruler-line
    // toggle starts ON).
    w.i32(1);
    for _ in 0..11 {
        w.i32(0);
    }
    w.i32(0); // `:54903` bartenderQuestLog (`Player.cs:1914`, default 0)
              // `:54904-54908` dead (default false); the respawnTimer is written ONLY
              // when dead — a fresh save emits the single bool.
    w.boolean(false);
    // `:54909` DateTime.UtcNow.ToBinary() — the wall-clock field, carried
    // by the caller (see `PlayerFileWrite::last_saved_binary`).
    w.u64(file.last_saved_binary as u64);
    w.i32(0); // `:54910` golferScoreAccumulated (`Player.cs:1912`, default 0)
              // `:54911` creativeTracker.Save -> CreativeUnlocksTracker.Save
              // (`CreativeUnlocksTracker.cs:9-12`) -> ItemsSacrificedUnlocksTracker.Save
              // (`ItemsSacrificedUnlocksTracker.cs:157-169`): the literal `false` flag,
              // the sacrifice-pair count, then the (persistentId, count) pairs — the
              // Journey research the caller carries (a fresh tracker has none).
    w.boolean(false);
    w.i32(file.research.len() as i32);
    for (key, count) in &file.research {
        w.cs_string(key);
        w.i32(*count);
    }
    // `:54912` SaveTemporaryItemSlotContents (`Player.cs:54935-54959`): one
    // BitsByte over mouseItem / CreativeMenu.GetItemByIndex(0) / guideItem /
    // reforgeItem IsAir — the port models none of the four global slots, so
    // all air → 0, no item payloads.
    w.u8(0);
    // `:54913` CreativePowerManager.SaveToPlayer
    // (`CreativePowerManager.cs:183-195`): for each registered power that is
    // IPersistentPerPlayerContent, `true` + the u16 id + its Save. The
    // registration order (`Initialize`, `:86-105`) gives GodmodePower id 5
    // (`IsEnabledForPlayer(Main.myPlayer)`, `CreativePowers.cs:705-707`),
    // FarPlacementRangePower id 11 (`:744-746`), SpawnRateSliderPerPlayerPower
    // id 14 (`_sliderCurrentValueCache`, `:1717-1719`). Then the `false`
    // terminator. The values ride the caller's `powers` field.
    w.boolean(true);
    w.u16(5);
    w.boolean(file.powers.godmode);
    w.boolean(true);
    w.u16(11);
    w.boolean(file.powers.far_placement);
    w.boolean(true);
    w.u16(14);
    w.f32(file.powers.spawn_rate);
    w.boolean(false);
    // `:54914-54918` the super-cart BitsByte: bit0 unlockedSuperCart
    // (`Player.cs:2052`, default false), bit1 enabledSuperCart
    // (`Player.cs:2054`, default **true**) → 0b10 = 2.
    w.u8(2);
    w.i32(0); // `:54919` CurrentLoadoutIndex (`Player.cs:3207`, default 0)
              // `:54920-54923` Loadouts[3].Serialize (`Player.cs:3200-3204`;
              // `EquipmentLoadout.cs:31-45`): Armor[20] + Dye[10] each
              // Item.Serialize(writeFavorite: true) = (type, stack, prefix, favorited)
              // — fresh `Item()`s are (0, 0, 0, false) (`EquipmentLoadout.cs:14-19`) —
              // then Hide[10] bools, all false.
    for _ in 0..3 {
        for _ in 0..30 {
            w.i32(0);
            w.i32(0);
            w.u8(0);
            w.boolean(false);
        }
        for _ in 0..10 {
            w.boolean(false);
        }
    }
    w.u8(1); // `:54924` (byte)voiceVariant — the ctor sets 1 (`Player.cs:56045`)
    w.f32(0.0); // `:54925` voicePitchOffset (`Player.cs:1348`, default 0f)
                // `:54926` CraftingRequests.SavePossibleRefunds
                // (`CraftingRequests.cs:353-362`): the sum of consumed items over _pendingCrafts
                // — a fresh queue is empty → count 0, no item payloads.
    w.i32(0);
    // `:54927-54932` oneTimeDialoguesSeen (`Player.cs:3209`): a fresh empty
    // HashSet → count 0, no strings.
    w.i32(0);
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

/// `PlayerFileData.Rename` (`IO/PlayerFileData.cs:207-214`) for a card the
/// port has no live `Player` for: the trimmed name lands on the stored
/// character and `Player.SavePlayer` re-writes the file. The vanilla mutates
/// `Player.name` then serializes the WHOLE live player; the port models only
/// the card state, so the write is a read-modify-write splice: decrypt
/// (`LoadPlayer:55073-55076`), replace ONLY `Serialize`'s first field
/// (`Player.cs:54752`'s name string — the byte right after the 4-byte
/// release + the 20-byte `FileMetadata`), keep every other byte (including
/// the unmodelled tail the writer would default), re-encrypt with the
/// write-side PKCS7 envelope (`:54734-54742`). The `.bak` copy is
/// `InternalSavePlayerFile:54730-54733` — the previous file before the
/// overwrite. `Player.GetFileData` skips files that cannot load
/// (`Player.cs:55862-55881`), so the read gate runs first.
pub fn rename_player_file(path: &Path, new_name: &str) -> Result<(), PlayerFileError> {
    let raw = std::fs::read(path)?;
    let plain = cbc_decrypt(&raw);
    // The load gate: a file the listing would skip cannot be renamed either.
    read_player_bytes(&plain)?;
    let spliced = rename_plaintext_name(&plain, new_name)
        .ok_or(PlayerFileError::Truncated("rename name field"))?;
    // `InternalSavePlayerFile:54730-54733`: `FileUtilities.Copy(path, path +
    // ".bak")` when the file exists — it does (the read above succeeded).
    let mut bak = path.as_os_str().to_os_string();
    bak.push(".bak");
    std::fs::write(PathBuf::from(bak), &raw)?;
    std::fs::write(path, cbc_encrypt(&spliced))?;
    Ok(())
}

/// The name-field splice [`rename_player_file`] runs: `Serialize`'s first
/// field (`Player.cs:54752`) sits at offset 24 — the `i32` release
/// (`InternalSavePlayerFile:54738`) plus the 20-byte `FileMetadata`
/// (`FileMetadata.cs:22-27`: u64 magic + u32 revision + u64 favorites). The
/// 7-bit-length-prefixed string is replaced whole; every later byte keeps
/// its value. `None` when the field cannot be walked (a truncated file).
fn rename_plaintext_name(plain: &[u8], new_name: &str) -> Option<Vec<u8>> {
    let mut r = Reader::new(plain);
    r.skip(4 + 20)?;
    let name_start = r.pos();
    r.cs_string()?;
    let name_end = r.pos();
    let mut w = Writer::new();
    w.bytes(&plain[..name_start]);
    w.cs_string(new_name);
    w.bytes(&plain[name_end..]);
    Some(w.into_inner())
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
            last_saved_binary: 0,
            research: Vec::new(),
            powers: crate::player_file::CreativePowersSave::written_fresh(),
        }
    }

    // --- The card rename (research/19 M1, W4-f22) -----------------------------

    #[test]
    fn rename_rewrites_only_the_name_field_and_baks_the_previous_file() {
        // `PlayerFileData.Rename` (`IO/PlayerFileData.cs:207-214`): the name
        // lands, `Player.SavePlayer` re-writes, `InternalSavePlayerFile:54730-
        // 54733` copies the previous file to `.bak`. Every OTHER field keeps
        // its value — the splice never re-defaults the unmodelled tail.
        let dir = crate::test_world_helpers::test_scratch("plr-rename");
        let path = dir.join("Belsin.plr");
        write_player_file(&path, &sample()).unwrap();
        let before = read_player_file(&path).unwrap();
        rename_player_file(&path, "Zephyra").unwrap();
        let after = read_player_file(&path).unwrap();
        assert_eq!(after.name, "Zephyra");
        // The rest of the card state is untouched.
        assert_eq!(after.difficulty, before.difficulty);
        assert_eq!(after.hair, before.hair);
        assert_eq!(after.inventory, before.inventory);
        assert_eq!(after.inv_favorites, before.inv_favorites);
        assert_eq!(after.hair_color, before.hair_color);
        assert_eq!(after.revision, before.revision);
        // The `.bak` is the pre-rename file (`:54732`).
        let bak = dir.join("Belsin.plr.bak");
        assert_eq!(read_player_file(&bak).unwrap().name, "Belsin");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn rename_splice_keeps_every_byte_past_the_name_field() {
        // The plaintext witness: only `Serialize`'s first field changes; the
        // bytes before it (release + FileMetadata) and after it (the whole
        // remaining Serialize walk) are byte-identical.
        let plain = serialize_plaintext(&sample());
        let spliced = rename_plaintext_name(&plain, "A Much Longer Name Than Before").unwrap();
        let mut r = Reader::new(&plain);
        r.skip(4 + 20).unwrap();
        let name_start = r.pos();
        r.cs_string().unwrap();
        let name_end = r.pos();
        assert_eq!(&spliced[..name_start], &plain[..name_start]);
        let mut r2 = Reader::new(&spliced);
        r2.skip(4 + 20).unwrap();
        assert_eq!(r2.cs_string().unwrap(), "A Much Longer Name Than Before");
        let tail_start = r2.pos();
        assert_eq!(&spliced[tail_start..], &plain[name_end..]);
        // A truncated buffer refuses instead of mis-splicing.
        assert!(rename_plaintext_name(&plain[..25], "X").is_none());
    }

    #[test]
    fn rename_refuses_a_file_the_listing_would_skip() {
        // `Player.GetFileData`'s null-on-failure skip (`Player.cs:55862-55881`)
        // gates the rename too — garbage is not renamed, nothing is written.
        let dir = crate::test_world_helpers::test_scratch("plr-rename-bad");
        let path = dir.join("junk.plr");
        std::fs::write(&path, b"unreadable").unwrap();
        assert!(rename_player_file(&path, "X").is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"unreadable");
        assert!(!dir.join("junk.plr.bak").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn rename_round_trips_a_multibyte_name_through_the_7bit_length() {
        // The keyboard allows any text (`Lang.menu[45]`, no charset gate); a
        // >127-byte name exercises the multi-byte 7-bit length prefix on both
        // the write (`Writer::cs_string`) and the read walk.
        let long = "Ы".repeat(80); // 160 UTF-8 bytes -> a 2-byte length prefix
        let plain = serialize_plaintext(&sample());
        let spliced = rename_plaintext_name(&plain, &long).unwrap();
        let file = read_player_bytes(&spliced).unwrap();
        assert_eq!(file.name, long);
        assert_eq!(file.inventory, sample().inventory);
    }

    #[test]
    fn round_trip_through_the_crypto_and_the_sections() {
        // The write -> the read: the name/stats/inventory round-trip through
        // the AES-CBC layer and the release-326 section walk.
        let bytes = write_player_bytes(&sample());
        assert_eq!(bytes.len() % 16, 0, "the CBC padding aligns the file");
        // `read_player_bytes` takes the decrypted body — the CBC layer is
        // `read_player_file`'s job, so peel it here.
        let file = match read_player_bytes(&cbc_decrypt(&bytes)) {
            Ok(f) => f,
            Err(e) => panic!("read err: {e:?}"),
        };
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
        let file = match read_player_bytes(&cbc_decrypt(&bytes)) {
            Ok(f) => f,
            Err(e) => panic!("read err: {e:?}"),
        };
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
        let file = match read_player_bytes(&plain) {
            Ok(f) => f,
            Err(e) => panic!("read err: {e:?}"),
        };
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
        let file321 = match read_player_bytes(&lo) {
            Ok(f) => f,
            Err(e) => panic!("read err: {e:?}"),
        };
        assert_eq!(file321.name, file.name);
        assert_eq!(file321.inventory, file.inventory);
        assert_eq!(file321.inv_favorites, file.inv_favorites);
    }

    /// S1-J-5 golden layout: the plaintext through `hideInfo` matches the
    /// C# `Serialize` field order byte-for-byte at the documented offsets
    /// (`Player.cs:54738-54893`), with every unmodelled field at its fresh
    /// default. The inventory is the sample's; everything else is pinned.
    #[test]
    fn golden_layout_through_hide_info() {
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
        // --- increment 2: buffs through hideInfo (`:54864-54893`) ---------
        // buffs `:54864-54876`: 44 (maxBuffs, `:57106`) x (0, 0) pairs.
        assert_eq!(r.take(44 * 8).unwrap(), vec![0u8; 44 * 8].as_slice());
        // spawn points `:54877-54888`: fresh spN[0] == null (`:2488`) writes
        // the -1 sentinel and breaks — exactly one i32.
        assert_eq!(r.i32().unwrap(), -1);
        // hbLocked `:54889`: false (`:1436`).
        assert_eq!(r.u8().unwrap(), 0);
        // hideInfo[13] `:54890-54893`: fresh bool[13] (`:812`), all false.
        assert_eq!(r.take(13).unwrap(), &[0; 13]);
        // --- increment 3: the Serialize tail (`:54894-54933`) -------------
        assert_eq!(r.i32().unwrap(), 0); // anglerQuestsFinished `:54894`
        assert_eq!(r.take(16).unwrap(), &[0xFF; 16]); // DpadRadial -1 x4 `:54895-54898`
        assert_eq!(r.i32().unwrap(), 1); // builderAccStatus[0] `:56092`
        assert_eq!(r.take(44).unwrap(), &[0; 44]); // builderAccStatus[1..12]
        assert_eq!(r.i32().unwrap(), 0); // bartenderQuestLog `:54903`
        assert_eq!(r.u8().unwrap(), 0); // dead `:54904` (no respawnTimer)
        assert_eq!(r.u64().unwrap(), 0); // the sample's last_saved_binary `:54909`
        assert_eq!(r.i32().unwrap(), 0); // golferScoreAccumulated `:54910`
        assert_eq!(r.u8().unwrap(), 0); // creativeTracker flag `:54911`
        assert_eq!(r.i32().unwrap(), 0); // sacrifice count
        assert_eq!(r.u8().unwrap(), 0); // temp-slot BitsByte `:54912`
                                        // creative powers `:54913`: (true, 5, false) (true, 11, false)
                                        // (true, 14, 0f) then the false terminator.
        assert_eq!(r.u8().unwrap(), 1);
        assert_eq!(r.u16().unwrap(), 5);
        assert_eq!(r.u8().unwrap(), 0);
        assert_eq!(r.u8().unwrap(), 1);
        assert_eq!(r.u16().unwrap(), 11);
        assert_eq!(r.u8().unwrap(), 0);
        assert_eq!(r.u8().unwrap(), 1);
        assert_eq!(r.u16().unwrap(), 14);
        assert_eq!(r.f32().unwrap(), 0.0);
        assert_eq!(r.u8().unwrap(), 0);
        assert_eq!(r.u8().unwrap(), 2); // superCart BitsByte `:54914-54918`
        assert_eq!(r.i32().unwrap(), 0); // CurrentLoadoutIndex `:54919`
        for _ in 0..3 {
            // EquipmentLoadout.Serialize `:54920-54923`: 30 x 10 zero bytes
            // + Hide[10] false = 310 zero bytes per loadout.
            assert_eq!(r.take(310).unwrap(), vec![0u8; 310].as_slice());
        }
        assert_eq!(r.u8().unwrap(), 1); // voiceVariant `:54924` (ctor `:56045`)
        assert_eq!(r.f32().unwrap(), 0.0); // voicePitchOffset `:54925`
        assert_eq!(r.i32().unwrap(), 0); // refunds `:54926`
        assert_eq!(r.i32().unwrap(), 0); // dialogues `:54927-54932`
        assert_eq!(r.pos(), plain.len());
        // The C#-written fixture (tools/WldFixtureHarness, real Serialize)
        // is byte-identical to the port over the owned tail block. In the
        // fixture the buffs start at 2402 (24 prologue + 87 header + 120
        // armor + 60 dye + 580 inventory + 50 misc + 1080 banks + 400 bank4
        // + 1 vault) and hideInfo ends at 2772 (+ 352 buffs + 4 sentinel
        // + 1 hbLocked + 13 hideInfo).
        let fixture = cbc_decrypt(include_bytes!("../../../assets/test/golden.plr"));
        // buffs..hideInfo 370 + angler 4 + dpad 16 + builder 48 + bartender 4
        // + dead 1 + timestamp 8 + golfer 4 + tracker 5 + temp slots 1
        // + powers 16 + supercart 1 + loadout idx 4 + loadouts 930 + voice 5
        // + refunds 4 + dialogues 4 = 1425.
        let tail_len = 370 + 4 + 16 + 48 + 4 + 1 + 8 + 4 + 5 + 1 + 16 + 1 + 4 + 930 + 5 + 4 + 4;
        let mut port_tail = plain[plain.len() - tail_len..].to_vec();
        assert_eq!(port_tail.len(), 1425);
        // The one unowned field: the sample's timestamp is 0, the fixture's
        // is the pinned FixtureClock — splice the fixture's 8 bytes in at
        // tail offset 443 (370 buffs..hideInfo + 4 angler + 16 dpad +
        // 48 builder + 4 bartender + 1 dead) before comparing.
        port_tail[443..451].copy_from_slice(&fixture[2845..2853]);
        assert_eq!(
            port_tail.as_slice(),
            &fixture[2402..3827],
            "buffs..dialogues byte parity with the C# fixture"
        );
    }

    /// S1-J-5 inc3: with the fixture's player values and the harness's pinned
    /// clock, the port's writer reproduces the C#-written `golden.plr`
    /// BYTE-FOR-BYTE — the full 3827-byte plaintext AND the PKCS7 envelope
    /// (13 bytes of 0x0D, `FlushFinalBlock`, `Player.cs:54742`).
    #[test]
    fn golden_plr_writer_matches_the_csharp_fixture_byte_for_byte() {
        // The harness's player (tools/WldFixtureHarness/Program.cs:289-307).
        let mut inventory = [(0i32, 0i32, 0u8); 58];
        inventory[0] = (3507, 1, 0); // copper shortsword
        inventory[1] = (3509, 1, 0); // copper axe
        inventory[2] = (3506, 1, 0); // copper pickaxe
        inventory[3] = (2, 50, 0); // dirt x50, favorited
        let file = PlayerFileWrite {
            name: "Golden".into(),
            difficulty: 0,
            skin_variant: 0,
            hair: 1,
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
            inv_favorites: 1 << 3,
            // FixtureClock.UtcNow.ToBinary() (Shims/FixtureClock.cs):
            // 2026-11-18T00:00:00Z, DateTimeKind.Utc → kind bits 01, no
            // offset: ticks 639305568000000000 | 0x4000000000000000.
            last_saved_binary: 0x48DF_4506_E391_C000,
            research: Vec::new(),
            powers: CreativePowersSave::written_fresh(),
        };
        let plain = serialize_plaintext(&file);
        let fixture_raw = include_bytes!("../../../assets/test/golden.plr");
        let fixture = cbc_decrypt(fixture_raw);
        assert_eq!(plain.len(), 3827, "the full Serialize plaintext length");
        assert_eq!(
            plain.as_slice(),
            &fixture[..3827],
            "the full plaintext matches the C# Serialize"
        );
        // The envelope: PKCS7 with n = 16 - 3827 % 16 = 13.
        assert_eq!(
            &fixture[3827..],
            &[13u8; 13][..],
            "the fixture carries real PKCS7 padding"
        );
        let written = write_player_bytes(&file);
        assert_eq!(written.len(), fixture_raw.len());
        assert_eq!(
            written.as_slice(),
            &fixture_raw[..],
            "the encrypted .plr is byte-identical to the C# fixture"
        );
        // And it reads back through the full Deserialize walk.
        let read = read_player_bytes(&cbc_decrypt(&written)).expect("the full-tail file must read");
        assert_eq!(read.name, "Golden");
        assert_eq!(read.inventory[3], (2, 50, 0));
        assert_eq!(read.inv_favorites, 1 << 3);
    }

    /// PKCS7 itself: the pad is 1..16 bytes of value n, and an aligned
    /// plaintext gains a FULL block of 0x10 (`RijndaelManaged` default,
    /// `Player.cs:54734`; `FlushFinalBlock`, `:54742`).
    #[test]
    fn the_write_envelope_is_pkcs7() {
        for len in [0usize, 1, 15, 16, 17, 31, 32, 3827] {
            let data = vec![0xABu8; len];
            let enc = cbc_encrypt(&data);
            let dec = cbc_decrypt(&enc);
            let n = 16 - len % 16;
            assert_eq!(dec.len(), len + n);
            assert_eq!(&dec[..len], &data[..]);
            assert_eq!(&dec[len..], vec![n as u8; n].as_slice());
        }
    }

    /// The port's own S1-J-5 increment-1 files (release 326, ending at
    /// `voidVaultInfo`, no buffs tail) still load through the retry in
    /// `read_player_bytes`.
    #[test]
    fn the_increment_one_files_still_load() {
        // The increment-1 layout is the current one minus the final 1425
        // bytes (the 370-byte buffs..hideInfo block + the 1055-byte inc3 tail).
        let plain = serialize_plaintext(&sample());
        let inc1 = &plain[..plain.len() - 1425];
        let file = match read_player_bytes(inc1) {
            Ok(f) => f,
            Err(e) => panic!("inc1 read err: {e:?}"),
        };
        assert_eq!(file.name, "Belsin");
        assert_eq!(file.inventory, sample().inventory);
        assert_eq!(file.inv_favorites, sample().inv_favorites);
        assert_eq!(file.life_max, 100);
    }

    /// The port's own S1-J-5 increment-2 files (release 326, ending at
    /// `hideInfo`, no angler/loadouts tail) still load through the depth-2
    /// retry in `read_player_bytes`.
    #[test]
    fn the_increment_two_files_still_load() {
        // The increment-2 layout is the current one minus the final 1055
        // bytes (the inc3 Serialize tail, `:54894-54933`).
        let plain = serialize_plaintext(&sample());
        let inc2 = &plain[..plain.len() - 1055];
        let file = match read_player_bytes(inc2) {
            Ok(f) => f,
            Err(e) => panic!("inc2 read err: {e:?}"),
        };
        assert_eq!(file.name, "Belsin");
        assert_eq!(file.inventory, sample().inventory);
        assert_eq!(file.inv_favorites, sample().inv_favorites);
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
        let file = match read_player_bytes(&w.into_inner()) {
            Ok(f) => f,
            Err(e) => panic!("legacy read err: {e:?}"),
        };
        assert_eq!(file.name, "Belsin");
        assert_eq!(file.inventory, s.inventory);
        assert_eq!(file.inv_favorites, s.inv_favorites);
        assert_eq!(file.hair_color, s.hair_color);
        assert_eq!(file.shoe_color, s.shoe_color);
        assert_eq!(file.life_max, 100);
    }

    #[test]
    fn a_genuine_legacy_type_byte_one_file_still_loads() {
        // Pre-S1-J-3 port saves stamped the type byte 1 (Map) by mistake.
        // The corrected constant is 3 (FileType.cs:8), but real on-disk old
        // dev characters carry 1; accept type-byte 1 ONLY for release 279 so
        // they still load (the legacy shim path). Build the old layout with
        // the WRONG type byte 1 and confirm it reads.
        let s = sample();
        let mut w = Writer::new();
        w.i32(279);
        w.u64(FILE_METADATA_MAGIC | (1u64 << 56)); // the old mistaken Map byte
        w.u32(0);
        w.u64(0);
        w.cs_string(&s.name);
        w.u8(s.difficulty);
        w.u64(0);
        w.i32(s.hair);
        w.u8(0);
        w.bytes(&[0, 0]);
        w.u8(s.skin_variant);
        w.i32(s.life);
        w.i32(s.life_max);
        w.i32(s.mana);
        w.i32(s.mana_max);
        for _ in 0..11 {
            w.boolean(false);
        }
        w.i32(0);
        w.i32(0);
        w.i32(0);
        for color in [
            s.hair_color, s.skin_color, s.eye_color, s.shirt_color,
            s.undershirt_color, s.pants_color, s.shoe_color,
        ] {
            w.bytes(&color);
        }
        for _ in 0..30 {
            w.i32(0);
            w.u8(0);
        }
        for (i, (id, stack, prefix)) in s.inventory.iter().enumerate() {
            w.i32(*id);
            w.i32(*stack);
            w.u8(*prefix);
            w.boolean(s.inv_favorites & (1 << i) != 0);
        }
        let file = match read_player_bytes(&w.into_inner()) {
            Ok(f) => f,
            Err(e) => panic!("legacy type-1 read err: {e:?}"),
        };
        assert_eq!(file.name, "Belsin");
        assert_eq!(file.inventory, s.inventory);
        // A non-279 file with the wrong type byte must STILL be rejected.
        let mut w2 = Writer::new();
        w2.i32(326);
        w2.u64(FILE_METADATA_MAGIC | (1u64 << 56));
        assert!(matches!(
            read_player_bytes(&w2.into_inner()),
            Err(PlayerFileError::WrongFileType)
        ));
    }

    #[test]
    fn the_metadata_carries_the_player_type() {
        // FileMetadata.Read: the top byte of the magic word is the FileType —
        // the player files carry 3 (FileType.cs:8; the port once stamped 1).
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
