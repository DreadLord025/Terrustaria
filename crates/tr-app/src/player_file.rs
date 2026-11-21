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
//! **Default-value policy.** The port models the character-card state
//! (name, difficulty, appearance, life/mana, the 58-slot inventory and its
//! favourited bits) plus, since W4-f44, EVERY per-player payload the `.plr`
//! carries: phases 1-2 added the loadouts, the void vault, the research
//! pairs, the equip/dye tails, the hide flags and the creative powers;
//! phase 3 completes the set — playtime/hairDye/team, the three banks, the
//! buffs, the spawn points, the quest/death counters, the temp slots, the
//! super-cart byte, the voice pair, the refunds and the dialogues. The
//! writer's `Default` still emits the value a *fresh vanilla player*
//! carries — the `Player` ctor (`Player.cs:56036-56096`) and the field
//! initializers — never omitted: omission shifts every later section and
//! breaks byte-comparability (the golden.plr parity pins it). The reader
//! walks the same span with the C# `Deserialize` version gates
//! (`Player.cs:55134-55764`), capturing every field verbatim.
//!
//! **W4-f44 phase 2: the equip/dye tails + the Journey unlocks.** The three
//! equipment tails the reader used to drop are captured verbatim on BOTH
//! models: `armor[20]`/`dye[10]` as `(type, prefix[, favorited >= 322])`
//! pairs (`Serialize:54812-54823` / `Deserialize:55306-55340`), the
//! interleaved `miscEquips[5]`/`miscDyes[5]` `(type, prefix)` pairs
//! (`:54831-54837` / `:55420-55447`) and the accessory-hide tails —
//! `hideVisibleAccessory[10]`'s two BitsBytes (`:54758-54769` /
//! `:55171-55183`), `hideMisc` (`:54770` / `:55192-55195`) and `hideInfo[13]`
//! (`:54890-54893` / `:55616-55623`). The Journey side gains the
//! per-player creative powers `CreativePowerManager.SaveToPlayer` writes
//! (`:54913` / `:55706-55709`, ids 5/11/14 — `CreativePowers.cs:705-707`/
//! `:744-746`/`:1717-1719`), completing the research payload phase 1 started.
//!
//! **W4-f44 phase 3: the remainder.** Every field the writer used to emit
//! at fresh defaults now rides both models verbatim: playtime/hairDye/team
//! (`:54754-54757`), bank/bank2/bank3 (`:54838-54855`), the compacted buffs
//! with the full `Main.buffNoSave` table (`:54864-54876`), the spawn points
//! (`:54877-54888`), angler/dpad/builderAcc (with the < 210 / < 249 fixups)
//! /bartender/dead+respawnTimer/golfer (`:54894-54910`), the temp slots
//! (`:54912`), the super-cart byte with the < 253 cart-derive (`:54914-54918`),
//! the voice pair with the 1..4 clamp + the < 280 Male-derive
//! (`:54924-54925`), the refunds (`:54926`) and the deduped dialogues
//! (`:54927-54932`). See `docs/ANALYSIS.md` "W4-f44 phase 3".
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

/// `Main.buffNoSave` (`Main.cs:8969-9073` + the mount loop `:9206-9213`): the
/// buff types the save writes as (0, 0) instead of their live values
/// (`Serialize:54866-54870`). The explicit set, the 173..=181 range
/// (`:9070-9073`), and every `BuffID.Sets.MountType != -1` buff
/// (`ID/BuffID.cs:169`). `buffNoSave[0]` is false, so a padded empty slot
/// writes its (0, 0) unchanged.
fn buff_no_save(id: i32) -> bool {
    const SET: [i32; 91] = [
        20, 22, 23, 24, 28, 30, 31, 34, 35, 37, 38, 39, 43, 44, 46, 47, 156, 149, 48, 58, 59,
        60, 62, 63, 64, 67, 68, 69, 70, 395, 72, 80, 87, 158, 146, 147, 215, 88, 89, 94, 95,
        96, 97, 98, 99, 100, 103, 119, 120, 125, 126, 133, 134, 135, 139, 140, 137, 144, 161,
        163, 164, 170, 171, 172, 182, 187, 188, 194, 195, 196, 197, 198, 199, 205, 213, 214,
        263, 271, 322, 320, 321, 325, 335, 150, 93, 159, 29, 348, 366, 353, 355,
    ];
    // The tail of the explicit block (`:9060-9069`).
    const TAIL: [i32; 10] = [385, 386, 389, 390, 393, 394, 397, 398, 399, 400];
    // The mount buffs (`BuffID.Sets.MountType`, `ID/BuffID.cs:169` — the
    // CreateIntSet keys; `Main.cs:9206-9213` marks each no-save).
    const MOUNTS: [i32; 66] = [
        90, 118, 128, 129, 130, 131, 132, 141, 142, 143, 162, 166, 168, 184, 193, 208, 210,
        212, 220, 222, 224, 226, 228, 230, 231, 233, 235, 237, 239, 241, 243, 245, 247, 249,
        251, 253, 255, 265, 269, 272, 275, 276, 277, 278, 279, 280, 281, 282, 283, 305, 318,
        338, 342, 346, 370, 374, 377, 378, 379, 380, 381, 384, 387, 388, 391, 392,
    ];
    SET.contains(&id) || TAIL.contains(&id) || MOUNTS.contains(&id) || (173..=181).contains(&id)
}

/// The read `.plr` — the character-card state plus the per-player payloads
/// the port models. The reader walks the full `Deserialize`
/// (`Player.cs:55134-55762`) with the C# version gates, consuming and
/// dropping every field the port does not model.
pub struct PlayerFile {
    pub name: String,
    pub difficulty: u8,
    /// `PlayerFileData.GetPlayTime().Ticks` (`Serialize:54754` writes the
    /// TimeSpan ticks; `Deserialize:55150-55157` reads them at >= 138 into
    /// `data.SetPlayTime`, else `TimeSpan.Zero`). W4-f44 ph3 carries them.
    pub play_time_ticks: i64,
    /// `Player.hairDye` (`Serialize:54756` / `Deserialize:55163-55166`,
    /// gate 82) — the hair-dye item slot id, 0 = none.
    pub hair_dye: u8,
    /// `Player.team` (`Serialize:54757` writes `(byte)team` /
    /// `Deserialize:55167-55170`, gate 283) — 0 = no team.
    pub team: u8,
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
    /// favourites ride the per-slot bool at release >= 322
    /// (`:55315`/`:55336`) — W4-f44 ph2 captures them on `armor`/`dye`.
    pub inv_favorites: u64,
    pub is_favorite: bool,
    pub revision: u32,
    /// `Player.CurrentLoadoutIndex` (`Player.cs:3207`, read `:55722`, clamped
    /// to `0..Loadouts.Length-1`). The port has no loadout-switch UI, but the
    /// index rides the file so a vanilla character's selection survives.
    pub current_loadout: i32,
    /// `Player.Loadouts[3]` (`Player.cs:3200-3204`, `EquipmentLoadout.cs:8-18`),
    /// read at release >= 262 (`:55720-55727`). Persisted verbatim so the
    /// stored equipment is not lost on a port re-save.
    pub loadouts: [LoadoutData; 3],
    /// `Player.voidVaultInfo` (`Player.cs:1590`, a `BitsByte`; bit 0 is
    /// `IsVoidVaultEnabled`, `:3900-3910`). Read at >= 199 (`:55501-55504`).
    pub void_vault_info: u8,
    /// `Player.buffType`/`buffTime` (`Player.cs:1510-1512`, maxBuffs = 44
    /// `:57106`) — the COMPACTED live pairs: the `Deserialize` zero-type arm
    /// (`:55593-55597`) shrinks the target so only non-zero buffs remain,
    /// in file order. The write pads back to 44 with (0, 0) and zeroes the
    /// `Main.buffNoSave` types (`Serialize:54864-54876`).
    pub buffs: Vec<(i32, i32)>,
    /// `Player.spX/spY/spI/spN` (`Player.cs:2484-2488`, int[200]/string[200])
    /// — the bed spawn points, up to 200 `(x, y, worldId, worldName)` records
    /// (`Serialize:54877-54888` writes until the first null spN then the -1
    /// sentinel; `Deserialize:55600-55611` reads until spX == -1).
    pub spawn_points: Vec<(i32, i32, i32, String)>,
    /// `Player.bank` (the piggy bank's 40 slots, `Chest.cs:142-144` maxItems),
    /// each `(type, stack, prefix)` — read at >= 58 (`Deserialize:55449-55456`;
    /// the < 58 arm reads 20, `:55464-55471`). The read is
    /// `netDefaults`/`stack`/`Prefix` with NO `TurnToAir`: an out-of-range id
    /// airs id+prefix (`Item.cs:48670-48688` then `Prefix` no-ops on air
    /// `:515-518`) while the stack rides; the legacy -1..-48 ids ride verbatim
    /// like the equip tails.
    pub bank: [(i32, i32, u8); 40],
    /// `Player.bank2` (the safe), the same shape (`:55457-55462` / `:55472-55477`).
    pub bank2: [(i32, i32, u8); 40],
    /// `Player.bank3` (the defender's forge), read at >= 182 (`:55479-55487`).
    pub bank3: [(i32, i32, u8); 40],
    /// `Player.bank4` (the Void Bag's 40 slots, `Player.cs:1591`), each
    /// `(type, stack, prefix, favorited)` — read at >= 198 with the favourited
    /// bool gated >= 255 (`:55488-55500`).
    pub bank4: [(i32, i32, u8, bool); 40],
    /// `Player.creativeTracker.ItemSacrifices`' `(persistentId, count)` pairs
    /// (`ItemsSacrificedUnlocksTracker.cs:157-169`), the Journey research the
    /// `.plr` carries per-player (`:54911` save / `:55696-55699` load).
    pub research: Vec<(String, i32)>,
    /// `Player.armor[20]` (`Player.cs:1488`) as `(type, prefix, favorited)` —
    /// the write carries NO stack (`Serialize:54812-54817`); the favourited
    /// bool is gated >= 322 on read (`Deserialize:55313-55316`). An
    /// out-of-range type airs through `SetDefaults` (`Item.cs:48671-48674`)
    /// and `Prefix` no-ops on air (`Item.cs:515-518`) — id+prefix air, the
    /// favorited bit rides. Legacy ids -1..-48 (`Item.netDefaults`,
    /// `:1213-1360`) ride verbatim so a re-save round-trips the bytes.
    pub armor: [(i32, u8, bool); 20],
    /// `Player.dye[10]`, the same `(type, prefix, favorited)` shape
    /// (`Serialize:54818-54823` / `Deserialize:55330-55339`).
    pub dye: [(i32, u8, bool); 10],
    /// `Player.miscEquips[5]` + `Player.miscDyes[5]` interleaved per slot —
    /// `(equip type, equip prefix, dye type, dye prefix)`
    /// (`Serialize:54831-54837` / `Deserialize:55420-55447`, the >= 136 arm).
    /// No stack, no favorited — the pet/light-pet/mount/cart/hook pairs.
    pub misc: [(i32, u8, i32, u8); 5],
    /// `Player.hideVisibleAccessory[10]` (`Player.cs:1708`) — the two
    /// BitsBytes' bits flattened (`Serialize:54758-54769` /
    /// `Deserialize:55171-55183`, gate 124): bits 0..7 then bits 8..9.
    pub hide_visible_accessory: [bool; 10],
    /// `Player.hideMisc` (`Player.cs:1710`, a BitsByte; the pet/light-pet
    /// hide bits 0/1, `:17344`/`:17353`) — `Serialize:54770` /
    /// `Deserialize:55192-55195`, gate 119.
    pub hide_misc: u8,
    /// `Player.hideInfo[13]` (`Player.cs:812`) — the info-accessory visibility
    /// toggles (`Serialize:54890-54893` / `Deserialize:55616-55623`, gate 115).
    pub hide_info: [bool; 13],
    /// The Journey per-player creative powers (`CreativePowerManager
    /// .LoadToPlayer`, `CreativePowerManager.cs:197-215`): Godmode (id 5)
    /// and FarPlacementRange (id 11) toggles + the SpawnRate slider (id 14),
    /// read at >= 220 (`Deserialize:55706-55709`). A fresh non-Journey player
    /// has all three reset (`ResetPowersForPlayer`, `:211-214`).
    pub powers: CreativePowersSave,
    /// `Player.anglerQuestsFinished` (`Player.cs:1910`) — `Serialize:54894` /
    /// `Deserialize:55624-55627`, gate 98.
    pub angler_quests_finished: i32,
    /// `Player.DpadRadial.Bindings[4]` (`Player.cs:800`, the Dpad4 ctor sets
    /// all -1 `:618-621`) — `Serialize:54895-54898` / `Deserialize:55628-55634`,
    /// gate 162.
    pub dpad_bindings: [i32; 4],
    /// `Player.builderAccStatus[12]` (`Player.cs:814`,
    /// `BuilderAccToggleIDs.Count = 12`; the ctor's `[0] = 1` `:56092`) —
    /// `Serialize:54899-54902` / `Deserialize:55635-55673`: the count gates
    /// 8/10/11/12 at 164/167/197/230, then the < 210 `[0] = 1` and the
    /// < 249 inventory-3611 `[1] = 1` fixups (state-only, no extra bytes).
    pub builder_acc_status: [i32; 12],
    /// `Player.bartenderQuestLog` (`Player.cs:1914`) — `Serialize:54903` /
    /// `Deserialize:55675-55678`, gate 181.
    pub bartender_quest_log: i32,
    /// `Player.dead` (`Serialize:54904` / `Deserialize:55679-55686`, gate 200)
    /// — the respawnTimer rides ONLY when dead.
    pub dead: bool,
    /// `Player.respawnTimer` (`Player.cs:1636`) — read with the
    /// `Utils.Clamp(0, 60000)` (`:55684`) when dead; written verbatim only
    /// when dead (`:54905-54908`).
    pub respawn_timer: i32,
    /// `Player.golferScoreAccumulated` (`Player.cs:1912`) — `Serialize:54910`
    /// / `Deserialize:55692-55695`, gate 206.
    pub golfer_score: i32,
    /// `Player._temporaryItemSlots[4]` (`Player.cs:3223`, a null-able
    /// `Item[4]`) — the mouse / creative-menu / guide / reforge global slots
    /// (`SaveTemporaryItemSlotContents:54935-54959` /
    /// `LoadTemporaryItemSlotContents:54962-54985`): one BitsByte over the
    /// four `!IsAir` flags, then an `Item.DeserializeFrom` triple per set
    /// bit. `None` = the slot is air (no payload).
    pub temp_slots: [Option<(i32, i32, u8)>; 4],
    /// The super-cart BitsByte (`Serialize:54914-54918`): bit 0
    /// `unlockedSuperCart` (`Player.cs:2052`, fresh false), bit 1
    /// `enabledSuperCart` (`:2054`, fresh **true**) — so a fresh save is 2.
    /// Read at >= 253 (`Deserialize:55710-55715`); below 253 the unlock is
    /// derived from `HasItemInAnyInventory(3353)` (`:55716-55719`) and the
    /// enabled bit keeps its field default true.
    pub super_cart: u8,
    /// `Player.voiceVariant` (`Serialize:54924` writes `(byte)voiceVariant` /
    /// `Deserialize:55728-55731`, gate 280; the ctor sets 1 `Player.cs:56045`).
    /// `LoadPlayer_LastMinuteFixes` clamps it to 1..4 (`:55831`). Below 280
    /// it derives from `Male ? 1 : 2` (`:55735`, `PlayerVariantID.Sets.Male`
    /// = {0,1,2,3,8,10}, `ID/PlayerVariantID.cs:20`).
    pub voice_variant: u8,
    /// `Player.voicePitchOffset` (`Player.cs:1348`, `Serialize:54925` /
    /// `Deserialize:55738-55741`, gate 281; below 281 it is 0f `:55742-55745`).
    pub voice_pitch_offset: f32,
    /// `Player._pendingRefunds` (`Deserialize:55746-55754`, gate 300) /
    /// `CraftingRequests.SavePossibleRefunds` (`Serialize:54926` ->
    /// `CraftingRequests.cs:353-362`): the sum of consumed items over the
    /// pending crafts, as `(type, stack, prefix)` triples
    /// (`Item.Serialize`/`DeserializeFrom`, no favourited). A fresh queue is
    /// empty → count 0.
    pub refunds: Vec<(i32, i32, u8)>,
    /// `Player.oneTimeDialoguesSeen` (`Player.cs:3209`, a `HashSet<string>`)
    /// — `Serialize:54927-54932` writes count + the strings /
    /// `Deserialize:55755-55762` (gate 310) reads count + strings, `.Add`
    /// deduping. The port keeps first-occurrence order (the HashSet's
    /// practical enumeration order for an add-only set).
    pub dialogues: Vec<String>,
}

/// The three `IPersistentPerPlayerContent` powers the `.plr` carries
/// (`CreativePowerManager.Initialize`, `CreativePowerManager.cs:86-105`):
/// `GodmodePower` id 5 (`CreativePowers.cs:692-728`), `FarPlacementRangePower`
/// id 11 (`:730-767`) and `SpawnRateSliderPerPlayerPower` id 14 (`:1573+`).
/// Each Save writes one bool / one bool / one f32 (`:705-707`/`:744-746`/
/// `:1717-1719`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CreativePowersSave {
    pub godmode: bool,
    pub far_placement: bool,
    pub spawn_rate: f32,
}

impl Default for CreativePowersSave {
    /// The reset defaults: Godmode `_defaultToggleState` false,
    /// FarPlacementRange true (`CreativePowers.cs:736`), SpawnRate
    /// `_sliderDefaultValue` 0.5 (`:1580`) — `ResetDataForNewPlayer`
    /// (`:710-713`/`:749-752`/`:1722-1725`) seeds these before the Load.
    /// The golden fixture's WRITTEN bytes are (false, false, 0f) — the
    /// harness's non-Journey `ResetPowersForPlayer` state — so the
    /// *writer's* default stays [`CreativePowersSave::written_fresh`].
    fn default() -> Self {
        CreativePowersSave { godmode: false, far_placement: true, spawn_rate: 0.5 }
    }
}

impl CreativePowersSave {
    /// The bytes a fresh non-Journey vanilla save writes: the harness pins
    /// (false, false, 0f) in `golden.plr` — `SaveToPlayer` runs on the power
    /// manager's live `_perPlayerIsEnabled`/`_sliderCurrentValueCache`
    /// state (`CreativePowers.cs:707`/`:746`/`:1719`), which a Classic
    /// character's session never touches (the toggles start disabled and
    /// the slider cache starts 0f). The *saved-field* reset defaults
    /// (false/true/0.5) are the read side's — see [`Default`].
    pub fn written_fresh() -> Self {
        CreativePowersSave { godmode: false, far_placement: false, spawn_rate: 0.0 }
    }
}

/// One `EquipmentLoadout`'s persisted slots (`EquipmentLoadout.cs:8-18`):
/// `Armor[20]` + `Dye[10]` as `Item.Serialize(writeFavorite: true)` records
/// `(type, stack, prefix, favorited)` (`Item.cs:49910-49919`), then `Hide[10]`.
#[derive(Clone, Debug, PartialEq)]
pub struct LoadoutData {
    pub armor: [(i32, i32, u8, bool); 20],
    pub dye: [(i32, i32, u8, bool); 10],
    pub hide: [bool; 10],
}

impl Default for LoadoutData {
    /// A fresh `EquipmentLoadout` (`EquipmentLoadout.cs:14-19`): every slot a
    /// `new Item()` = `(0, 0, 0, false)`, every hide false.
    fn default() -> Self {
        LoadoutData {
            armor: [(0, 0, 0, false); 20],
            dye: [(0, 0, 0, false); 10],
            hide: [false; 10],
        }
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

/// The bank-slot triple, captured as `(type, stack, prefix)` — the bank slots
/// (`Deserialize:55449-55487`'s `netDefaults(id); stack; Prefix(byte)` read
/// order). For an out-of-range id `netDefaults`→`SetDefaults` airs the item
/// (`Item.cs:48670-48688`: type 0, stack 0, prefix 0); the read then overwrites
/// `stack` with the file's value and `Prefix` no-ops on air (`CanHavePrefixes`
/// false, `:515-518`) so the prefix stays 0 — id+prefix air, stack rides. The
/// legacy -1..-48 ids ride verbatim (the card model carries no item-defaults
/// tables); anything outside those bands airs like `SetDefaults` does (the
/// ph2 equip policy). There is no `TurnToAir` in the bank read path.
fn read_bank_item(r: &mut Reader, what: &'static str) -> Result<(i32, i32, u8), PlayerFileError> {
    let id = r.i32().ok_or(PlayerFileError::Truncated(what))?;
    let stack = r.i32().ok_or(PlayerFileError::Truncated(what))?;
    let prefix = r.u8().ok_or(PlayerFileError::Truncated(what))?;
    if (0..ITEM_ID_COUNT).contains(&id) || (-48..0).contains(&id) {
        Ok((id, stack, prefix))
    } else {
        Ok((0, stack, 0))
    }
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
    // The per-player tail the walk captures as it goes — created up-front so
    // the header fields land on it too; defaults for the sections a shallower
    // port-only depth (or an older release gate) never reaches.
    let mut tail = PlayerTail::default();
    let name = r.cs_string().ok_or(PlayerFileError::Truncated("name"))?;
    let difficulty = r.u8().ok_or(PlayerFileError::Truncated("difficulty"))?; // gate 17
    // playTime ticks (`:55150-55157`, gate 138) — W4-f44 ph3 rides them.
    tail.play_time_ticks = r.u64().ok_or(PlayerFileError::Truncated("playtime"))? as i64;
    let mut hair = r.i32().ok_or(PlayerFileError::Truncated("hair"))?;
    if hair >= 228 {
        hair = 0; // `:55159-55162`
    }
    // hairDye (`:55163-55166`, gate 82).
    tail.hair_dye = r.u8().ok_or(PlayerFileError::Truncated("hairDye"))?;
    if version >= 283 {
        tail.team = r.u8().ok_or(PlayerFileError::Truncated("team"))?; // `:55167-55170`
    }
    // hideVisibleAccessory: two BitsBytes (`:55171-55183`, gate 124) —
    // W4-f44 ph2 captures the ten bits (`Serialize:54758-54769`'s order).
    let hide1 = r.u8().ok_or(PlayerFileError::Truncated("hide1"))?;
    let hide2 = r.u8().ok_or(PlayerFileError::Truncated("hide2"))?;
    let mut hide_visible_accessory = [false; 10];
    for (i, slot) in hide_visible_accessory.iter_mut().enumerate() {
        *slot = if i < 8 { hide1 & (1 << i) != 0 } else { hide2 & (1 << (i - 8)) != 0 };
    }
    // hideMisc (`:55192-55195`, gate 119) — the byte the old subset omitted.
    let hide_misc = r.u8().ok_or(PlayerFileError::Truncated("hideMisc"))?;
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
    // W4-f44 ph2: captured verbatim — an out-of-range type airs through
    // `SetDefaults` (`Item.cs:48671-48674`); `Prefix` then no-ops on the air
    // item (`CanHavePrefixes` false, `Item.cs:515-518`) so the prefix airs
    // too, but favorited is set unconditionally (`:55315`) and rides.
    let mut armor = [(0i32, 0u8, false); 20];
    for slot in armor.iter_mut() {
        let id = r.i32().ok_or(PlayerFileError::Truncated("armor"))?;
        let prefix = r.u8().ok_or(PlayerFileError::Truncated("armorPrefix"))?;
        let mut fav = false;
        if version >= 322 {
            fav = r.boolean().ok_or(PlayerFileError::Truncated("armorFav"))?;
        }
        // The legacy negative ids -1..-48 map to real items (`Item
        // .netDefaults`, `Item.cs:1213-1360`); the port keeps the id verbatim
        // so a re-save round-trips the bytes (a legacy file's mapping needs
        // the item-defaults tables the card model does not carry). Anything
        // outside those bands airs like `SetDefaults` does.
        if (0..ITEM_ID_COUNT).contains(&id) || (-48..0).contains(&id) {
            *slot = (id, prefix, fav);
        } else {
            *slot = (0, 0, fav);
        }
    }
    // The dye: 10 slots x (id, prefix[, favorited >= 322]) (`:55319-55340`).
    let mut dye = [(0i32, 0u8, false); 10];
    for slot in dye.iter_mut() {
        let id = r.i32().ok_or(PlayerFileError::Truncated("dye"))?;
        let prefix = r.u8().ok_or(PlayerFileError::Truncated("dyePrefix"))?;
        let mut fav = false;
        if version >= 322 {
            fav = r.boolean().ok_or(PlayerFileError::Truncated("dyeFav"))?;
        }
        if (0..ITEM_ID_COUNT).contains(&id) || (-48..0).contains(&id) {
            *slot = (id, prefix, fav);
        } else {
            *slot = (0, 0, fav);
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
    // (`:55420-55447`, gates 117/136 both true here) — W4-f44 ph2 captures
    // the interleaved pairs verbatim (`Serialize:54831-54837`'s order).
    let mut misc = [(0i32, 0u8, 0i32, 0u8); 5];
    for slot in misc.iter_mut() {
        let eq_id = r.i32().ok_or(PlayerFileError::Truncated("miscEquip"))?;
        let eq_prefix = r.u8().ok_or(PlayerFileError::Truncated("miscEquipPrefix"))?;
        let dye_id = r.i32().ok_or(PlayerFileError::Truncated("miscDye"))?;
        let dye_prefix = r.u8().ok_or(PlayerFileError::Truncated("miscDyePrefix"))?;
        let eq_ok = (eq_id >= 0 && eq_id < ITEM_ID_COUNT) || (-48..0).contains(&eq_id);
        let dye_ok = (dye_id >= 0 && dye_id < ITEM_ID_COUNT) || (-48..0).contains(&dye_id);
        *slot = (
            if eq_ok { eq_id } else { 0 },
            if eq_ok { eq_prefix } else { 0 },
            if dye_ok { dye_id } else { 0 },
            if dye_ok { dye_prefix } else { 0 },
        );
    }
    // bank + bank2: 40 slots x (id, stack, prefix) each (`:55449-55462`,
    // gate 58; the < 58 arm reads 20 — outside the accepted [135, 326]
    // range, cited). W4-f44 ph3: the slots ride verbatim.
    for slot in tail.bank.iter_mut() {
        *slot = read_bank_item(r, "bank")?;
    }
    for slot in tail.bank2.iter_mut() {
        *slot = read_bank_item(r, "bank2")?;
    }
    if version >= 182 {
        for slot in tail.bank3.iter_mut() {
            *slot = read_bank_item(r, "bank3")?; // `:55479-55487`
        }
    }
    // `Player.bank4` — the Void Bag's 40 slots (`:55488-55500`); the C#
    // `netDefaults(type)` airs a type >= ItemID.Count AFTER reading the whole
    // record (`Item.DeserializeFrom`, `Item.cs:49930-49933`), so the port
    // airs the slot but consumes the same bytes.
    let mut bank4 = [(0i32, 0i32, 0u8, false); 40];
    if version >= 198 {
        for slot in bank4.iter_mut() {
            let id = r.i32().ok_or(PlayerFileError::Truncated("bank4"))?;
            let stack = r.i32().ok_or(PlayerFileError::Truncated("bank4Stack"))?;
            let prefix = r.u8().ok_or(PlayerFileError::Truncated("bank4Prefix"))?;
            let mut fav = false;
            if version >= 255 {
                fav = r.boolean().ok_or(PlayerFileError::Truncated("bank4Fav"))?;
            }
            if id < ITEM_ID_COUNT {
                *slot = (id, stack, prefix, fav);
            }
        }
    }
    let mut void_vault_info = 0u8;
    if version >= 199 {
        void_vault_info = r.u8().ok_or(PlayerFileError::Truncated("voidVault"))?;
        // `:55501-55504`
    }
    // Fold the fields captured after the tail's creation point into it.
    tail.void_vault_info = void_vault_info;
    tail.bank4 = bank4;
    tail.armor = armor;
    tail.dye = dye;
    tail.misc = misc;
    tail.hide_visible_accessory = hide_visible_accessory;
    tail.hide_misc = hide_misc;
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
            tail,
        ));
    }
    // --- S1-J-5 increment 2: buffs through hideInfo (`:55578-55623`) ------
    // buffs (`:55578-55599`, gate 11): the slot count is 10 below 74, 22,
    // and 44 at >= 252 (`Player.maxBuffs`, `Player.cs:57106`). The zero-type
    // compaction (`num28--; num27--`, `:55593-55597`) still consumes exactly
    // `count` (type, time) pairs — each iteration advances either the read
    // index or shrinks the target. W4-f44 ph3: the compacted live pairs ride.
    if version >= 11 {
        let mut count: i32 = if version < 74 { 10 } else { 22 };
        if version >= 252 {
            count = 44;
        }
        let mut i: i32 = 0;
        while i < count {
            let buff_type = r.i32().ok_or(PlayerFileError::Truncated("buffType"))?;
            let buff_time = r.i32().ok_or(PlayerFileError::Truncated("buffTime"))?;
            if buff_type == 0 {
                i -= 1;
                count -= 1;
            } else {
                tail.buffs.push((buff_type, buff_time));
            }
            i += 1;
        }
    }
    // spawn points (`:55600-55611`, ungated): up to 200 records, each
    // (spX, spY, spI, spN); spX == -1 is the end sentinel. W4-f44 ph3: the
    // records ride verbatim (an spX of -1 inside a record ends the read,
    // exactly like the C# loop — no special case).
    for _ in 0..200 {
        let sp_x = r.i32().ok_or(PlayerFileError::Truncated("spawnX"))?;
        if sp_x == -1 {
            break;
        }
        let sp_y = r.i32().ok_or(PlayerFileError::Truncated("spawnY"))?;
        let sp_i = r.i32().ok_or(PlayerFileError::Truncated("spawnI"))?;
        let sp_n = r.cs_string().ok_or(PlayerFileError::Truncated("spawnN"))?;
        tail.spawn_points.push((sp_x, sp_y, sp_i, sp_n));
    }
    if version >= 16 {
        r.boolean().ok_or(PlayerFileError::Truncated("hbLocked"))?; // `:55612-55615`
    }
    if version >= 115 {
        for hide in tail.hide_info.iter_mut() {
            *hide = r.boolean().ok_or(PlayerFileError::Truncated("hideInfo"))?; // `:55616-55623`
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
            tail,
        ));
    }
    // --- S1-J-5 increment 3: the Deserialize tail (`:55624-55762`) --------
    if version >= 98 {
        tail.angler_quests_finished =
            r.i32().ok_or(PlayerFileError::Truncated("angler"))?; // `:55624-55627`
    }
    if version >= 162 {
        for binding in tail.dpad_bindings.iter_mut() {
            *binding = r.i32().ok_or(PlayerFileError::Truncated("dpad"))?; // `:55628-55634`
        }
    }
    if version >= 164 {
        // The builderAccStatus count gate (`:55635-55653`): 8, then 10 at
        // 167, 11 at 197, 12 at 230. W4-f44 ph3: the slots ride; the
        // < 210 / < 249 fixups (`:55654-55673`) mutate state only — no
        // extra bytes.
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
        for slot in tail.builder_acc_status[..count].iter_mut() {
            *slot = r.i32().ok_or(PlayerFileError::Truncated("builderAcc"))?;
        }
        if version < 210 {
            tail.builder_acc_status[0] = 1; // `:55654-55657`
        }
        if version < 249 {
            // `:55658-55673`: the Terraprisma-in-inventory check (3611).
            if inventory.iter().any(|(id, _, _)| *id == 3611) {
                tail.builder_acc_status[1] = 1;
            }
        }
    }
    if version >= 181 {
        tail.bartender_quest_log =
            r.i32().ok_or(PlayerFileError::Truncated("bartender"))?; // `:55675-55678`
    }
    if version >= 200 {
        // dead + the conditional respawnTimer (`:55679-55686`); the timer
        // reads through `Utils.Clamp(0, 60000)` (`:55684`).
        tail.dead = r.boolean().ok_or(PlayerFileError::Truncated("dead"))?;
        if tail.dead {
            let timer = r.i32().ok_or(PlayerFileError::Truncated("respawn"))?;
            tail.respawn_timer = timer.clamp(0, 60000);
        }
    }
    if version >= 202 {
        r.u64().ok_or(PlayerFileError::Truncated("lastSaved"))?; // `:55688-55691`
    }
    if version >= 206 {
        tail.golfer_score = r.i32().ok_or(PlayerFileError::Truncated("golfer"))?; // `:55692-55695`
    }
    if version >= 218 {
        // creativeTracker.Load -> CreativeUnlocksTracker.Load
        // (`CreativeUnlocksTracker.cs:14-17`) -> ItemsSacrificedUnlocksTracker
        // .Load (`ItemsSacrificedUnlocksTracker.cs:171-186`): the >= 282 bool
        // then the (string, i32) sacrifice pairs. Consumed and dropped.
        if version >= 282 {
            r.boolean().ok_or(PlayerFileError::Truncated("sacrFlag"))?;
        }
        let n = r.i32().ok_or(PlayerFileError::Truncated("sacrCount"))?;
        for _ in 0..n {
            let key = r.cs_string().ok_or(PlayerFileError::Truncated("sacrKey"))?;
            let val = r.i32().ok_or(PlayerFileError::Truncated("sacrVal"))?;
            tail.research.push((key, val));
        }
    }
    if version >= 214 {
        // LoadTemporaryItemSlotContents (`Player.cs:54962-54985`): one
        // BitsByte, then an Item.DeserializeFrom triple per set bit 0..3.
        // W4-f44 ph3: the slots ride; the triple airs a type >= ItemID.Count
        // AFTER the whole record (`Item.cs:49930-49933` TurnToAir — the
        // bank4/loadout precedent keeps the read stack/prefix).
        let bits = r.u8().ok_or(PlayerFileError::Truncated("tempSlots"))?;
        for (i, slot) in tail.temp_slots.iter_mut().enumerate() {
            if bits & (1 << i) != 0 {
                let id = r.i32().ok_or(PlayerFileError::Truncated("tempItem"))?;
                let stack = r.i32().ok_or(PlayerFileError::Truncated("tempItem"))?;
                let prefix = r.u8().ok_or(PlayerFileError::Truncated("tempItem"))?;
                *slot = Some((if id < ITEM_ID_COUNT { id } else { 0 }, stack, prefix));
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
        // W4-f44 ph2: the loaded values ride the tail; the C# then resets
        // them for non-Journey characters (`difficulty != 3`, `:211-214`)
        // after the whole walk — mirrored in `finish_player_file`.
        loop {
            let more = r.boolean().ok_or(PlayerFileError::Truncated("powerFlag"))?;
            if !more {
                break;
            }
            let id = r.u16().ok_or(PlayerFileError::Truncated("powerId"))?;
            match id {
                5 => {
                    tail.powers.godmode =
                        r.boolean().ok_or(PlayerFileError::Truncated("powerBool"))?;
                }
                11 => {
                    tail.powers.far_placement =
                        r.boolean().ok_or(PlayerFileError::Truncated("powerBool"))?;
                }
                14 => {
                    tail.powers.spawn_rate =
                        r.f32().ok_or(PlayerFileError::Truncated("powerSlider"))?;
                }
                _ => break, // `CreativePowerManager.cs:203-206`
            }
        }
    }
    if version >= 253 {
        tail.super_cart = r.u8().ok_or(PlayerFileError::Truncated("superCart"))?; // `:55710-55715`
    } else {
        // `:55716-55719`: the unlock derives from `HasItemInAnyInventory(3353)`
        // (`:55931-55970`, `HasItem` needs stack > 0 `:55906-55916`); the
        // enabled bit keeps its field default true (`:2054`). The equip
        // arrays carry no stack in the card model, so a non-air equip slot
        // stands in for stack > 0 (documented in ANALYSIS).
        let has_cart = inventory.iter().any(|(id, stack, _)| *id == 3353 && *stack > 0)
            || tail.armor.iter().any(|(id, _, _)| *id == 3353)
            || tail.dye.iter().any(|(id, _, _)| *id == 3353)
            || tail.misc.iter().any(|(eq, _, dye, _)| *eq == 3353 || *dye == 3353)
            || tail.bank.iter().chain(tail.bank2.iter()).chain(tail.bank3.iter())
                .any(|(id, stack, _)| *id == 3353 && *stack > 0)
            || tail.bank4.iter().any(|(id, stack, _, _)| *id == 3353 && *stack > 0);
        tail.super_cart = u8::from(has_cart) | 2;
    }
    if version >= 262 {
        // CurrentLoadoutIndex + the three loadouts (`:55720-55727`);
        // EquipmentLoadout.Deserialize (`EquipmentLoadout.cs:47-61`):
        // 20 armor + 10 dye x (type, stack, prefix[, favorited >= 322])
        // then Hide[10] bools. `Item.DeserializeFrom` airs a type >=
        // ItemID.Count after the whole record is consumed (`Item.cs:49930`).
        let idx = r.i32().ok_or(PlayerFileError::Truncated("loadoutIdx"))?;
        // `Utils.Clamp(idx, 0, Loadouts.Length - 1)` (`:55722`).
        tail.current_loadout = idx.clamp(0, 2);
        for loadout in tail.loadouts.iter_mut() {
            for slot in loadout.armor.iter_mut().chain(loadout.dye.iter_mut()) {
                let id = r.i32().ok_or(PlayerFileError::Truncated("loadoutItem"))?;
                let stack = r
                    .i32()
                    .ok_or(PlayerFileError::Truncated("loadoutStack"))?;
                let prefix = r
                    .u8()
                    .ok_or(PlayerFileError::Truncated("loadoutPrefix"))?;
                let mut fav = false;
                if version >= 322 {
                    fav = r.boolean().ok_or(PlayerFileError::Truncated("loadoutFav"))?;
                }
                if id < ITEM_ID_COUNT {
                    *slot = (id, stack, prefix, fav);
                }
            }
            for hide in loadout.hide.iter_mut() {
                *hide = r.boolean().ok_or(PlayerFileError::Truncated("loadoutHide"))?;
            }
        }
    }
    if version >= 280 {
        tail.voice_variant = r.u8().ok_or(PlayerFileError::Truncated("voice"))?; // `:55728-55731`
    } else {
        // `:55732-55737`: the skinVariant clamp (done in finish_player_file),
        // then voiceVariant = Male ? 1 : 2. `Male` reads
        // `PlayerVariantID.Sets.Male[skinVariant]` (`Player.cs:3405-3410`,
        // the set {0,1,2,3,8,10}, `ID/PlayerVariantID.cs:20`); the variant is
        // clamped first exactly like `:55734`.
        let variant = (skin_variant as i32).clamp(0, PLAYER_VARIANT_COUNT - 1);
        let male = matches!(variant, 0 | 1 | 2 | 3 | 8 | 10);
        tail.voice_variant = if male { 1 } else { 2 };
        tail.voice_pitch_offset = 0.0;
    }
    if version >= 281 {
        tail.voice_pitch_offset =
            r.f32().ok_or(PlayerFileError::Truncated("voicePitch"))?; // `:55738-55741`
    } else {
        tail.voice_pitch_offset = 0.0; // `:55742-55745`
    }
    if version >= 300 {
        // _pendingRefunds (`:55746-55754`): count + Item.DeserializeFrom
        // triples. W4-f44 ph3: the triples ride; an over-count id airs after
        // the whole record (`Item.cs:49930-49933`, the bank4 precedent).
        let n = r.i32().ok_or(PlayerFileError::Truncated("refundCount"))?;
        for _ in 0..n {
            let id = r.i32().ok_or(PlayerFileError::Truncated("refundItem"))?;
            let stack = r.i32().ok_or(PlayerFileError::Truncated("refundItem"))?;
            let prefix = r.u8().ok_or(PlayerFileError::Truncated("refundItem"))?;
            tail.refunds.push((if id < ITEM_ID_COUNT { id } else { 0 }, stack, prefix));
        }
    }
    if version >= 310 {
        // oneTimeDialoguesSeen (`:55755-55762`): count + strings. W4-f44
        // ph3: the strings ride; `.Add` on the HashSet dedupes, so the port
        // keeps first occurrences only.
        let n = r.i32().ok_or(PlayerFileError::Truncated("dialogueCount"))?;
        for _ in 0..n {
            let key = r.cs_string().ok_or(PlayerFileError::Truncated("dialogue"))?;
            if !tail.dialogues.contains(&key) {
                tail.dialogues.push(key);
            }
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
        tail,
    ))
}

/// The per-player payloads the walk captures past the character-card state —
/// the void vault (bank4 + the info byte), the Journey research pairs and the
/// three equipment loadouts, plus (W4-f44 ph2) the armor/dye/misc equip tails,
/// the hide flags and the creative powers. Defaults for the sections a
/// shallower port-only depth (or an older release gate) never reaches.
struct PlayerTail {
    play_time_ticks: i64,
    hair_dye: u8,
    team: u8,
    buffs: Vec<(i32, i32)>,
    spawn_points: Vec<(i32, i32, i32, String)>,
    current_loadout: i32,
    loadouts: [LoadoutData; 3],
    void_vault_info: u8,
    bank: [(i32, i32, u8); 40],
    bank2: [(i32, i32, u8); 40],
    bank3: [(i32, i32, u8); 40],
    bank4: [(i32, i32, u8, bool); 40],
    research: Vec<(String, i32)>,
    armor: [(i32, u8, bool); 20],
    dye: [(i32, u8, bool); 10],
    misc: [(i32, u8, i32, u8); 5],
    hide_visible_accessory: [bool; 10],
    hide_misc: u8,
    hide_info: [bool; 13],
    powers: CreativePowersSave,
    angler_quests_finished: i32,
    dpad_bindings: [i32; 4],
    builder_acc_status: [i32; 12],
    bartender_quest_log: i32,
    dead: bool,
    respawn_timer: i32,
    golfer_score: i32,
    temp_slots: [Option<(i32, i32, u8)>; 4],
    super_cart: u8,
    voice_variant: u8,
    voice_pitch_offset: f32,
    refunds: Vec<(i32, i32, u8)>,
    dialogues: Vec<String>,
}

impl Default for PlayerTail {
    /// The fresh defaults — a manual impl because `bank4` is a 40-element
    /// array and std's derived array `Default` stops at 32
    /// (`core::array::array_impl_default!{32, ...}`).
    fn default() -> Self {
        PlayerTail {
            play_time_ticks: 0,
            hair_dye: 0,
            team: 0,
            buffs: Vec::new(),
            spawn_points: Vec::new(),
            current_loadout: 0,
            loadouts: Default::default(),
            void_vault_info: 0,
            bank: [(0, 0, 0); 40],
            bank2: [(0, 0, 0); 40],
            bank3: [(0, 0, 0); 40],
            bank4: [(0, 0, 0, false); 40],
            research: Vec::new(),
            armor: [(0, 0, false); 20],
            dye: [(0, 0, false); 10],
            misc: [(0, 0, 0, 0); 5],
            hide_visible_accessory: [false; 10],
            hide_misc: 0,
            hide_info: [false; 13],
            powers: CreativePowersSave::default(),
            angler_quests_finished: 0,
            dpad_bindings: [-1; 4], // the Dpad4 ctor (`Player.cs:618-621`)
            // The ctor's `builderAccStatus[0] = 1` (`Player.cs:56092`).
            builder_acc_status: [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            bartender_quest_log: 0,
            dead: false,
            respawn_timer: 0,
            golfer_score: 0,
            temp_slots: [None; 4],
            // `enabledSuperCart = true` (`Player.cs:2054`) → 0b10 = 2.
            super_cart: 2,
            voice_variant: 1, // the ctor (`Player.cs:56045`)
            voice_pitch_offset: 0.0,
            refunds: Vec::new(),
            dialogues: Vec::new(),
        }
    }
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
    tail: PlayerTail,
) -> PlayerFile {
    let skin_variant = (skin_variant as i32).clamp(0, PLAYER_VARIANT_COUNT - 1) as u8;
    // `LoadPlayer_LastMinuteFixes` (`:55831`): voiceVariant clamps to 1..4.
    let voice_variant = (tail.voice_variant as i32).clamp(1, 4) as u8;
    // CreativePowerManager.LoadToPlayer (`CreativePowerManager.cs:211-214`):
    // a non-Journey character (difficulty != 3) has the loaded powers RESET
    // after the walk — Godmode false / FarPlacement true / SpawnRate 0.5
    // (`ResetDataForNewPlayer`, `CreativePowers.cs:710-713`/`:749-752`/
    // `:1722-1725` -> the ctor defaults). Only Journey keeps the loaded state.
    let powers = if difficulty != 3 { CreativePowersSave::default() } else { tail.powers };
    PlayerFile {
        name,
        difficulty,
        play_time_ticks: tail.play_time_ticks,
        hair_dye: tail.hair_dye,
        team: tail.team,
        buffs: tail.buffs,
        spawn_points: tail.spawn_points,
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
        current_loadout: tail.current_loadout,
        loadouts: tail.loadouts,
        void_vault_info: tail.void_vault_info,
        bank: tail.bank,
        bank2: tail.bank2,
        bank3: tail.bank3,
        bank4: tail.bank4,
        research: tail.research,
        armor: tail.armor,
        dye: tail.dye,
        misc: tail.misc,
        hide_visible_accessory: tail.hide_visible_accessory,
        hide_misc: tail.hide_misc,
        hide_info: tail.hide_info,
        powers,
        angler_quests_finished: tail.angler_quests_finished,
        dpad_bindings: tail.dpad_bindings,
        builder_acc_status: tail.builder_acc_status,
        bartender_quest_log: tail.bartender_quest_log,
        dead: tail.dead,
        respawn_timer: tail.respawn_timer,
        golfer_score: tail.golfer_score,
        temp_slots: tail.temp_slots,
        super_cart: tail.super_cart,
        voice_variant,
        voice_pitch_offset: tail.voice_pitch_offset,
        refunds: tail.refunds,
        dialogues: tail.dialogues,
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
    // W4-f44 ph3: the subset's playtime/hairDye ride (it has no team byte).
    let play_time_ticks = r.u64().ok_or(PlayerFileError::Truncated("playtime"))? as i64;
    let mut hair = r.i32().ok_or(PlayerFileError::Truncated("hair"))?;
    if hair >= 228 {
        hair = 0;
    }
    let hair_dye = r.u8().ok_or(PlayerFileError::Truncated("hairDye"))?;
    // The old writer emitted the two hide BitsBytes but no hideMisc.
    // W4-f44 ph2: the ten hide bits ride (same order as the full walk).
    let hide1 = r.u8().ok_or(PlayerFileError::Truncated("hide1"))?;
    let hide2 = r.u8().ok_or(PlayerFileError::Truncated("hide2"))?;
    let mut hide_visible_accessory = [false; 10];
    for (i, slot) in hide_visible_accessory.iter_mut().enumerate() {
        *slot = if i < 8 { hide1 & (1 << i) != 0 } else { hide2 & (1 << (i - 8)) != 0 };
    }
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
    // The subset's armour/dye pairs (no favourited bool at 279) — W4-f44
    // ph2 captures them with the same air policy as the full walk.
    let mut armor = [(0i32, 0u8, false); 20];
    for slot in armor.iter_mut() {
        let id = r.i32().ok_or(PlayerFileError::Truncated("armor"))?;
        let prefix = r.u8().ok_or(PlayerFileError::Truncated("armorPrefix"))?;
        if (0..ITEM_ID_COUNT).contains(&id) || (-48..0).contains(&id) {
            *slot = (id, prefix, false);
        }
    }
    let mut dye = [(0i32, 0u8, false); 10];
    for slot in dye.iter_mut() {
        let id = r.i32().ok_or(PlayerFileError::Truncated("dye"))?;
        let prefix = r.u8().ok_or(PlayerFileError::Truncated("dyePrefix"))?;
        if (0..ITEM_ID_COUNT).contains(&id) || (-48..0).contains(&id) {
            *slot = (id, prefix, false);
        }
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
        play_time_ticks,
        hair_dye,
        team: 0, // the subset shape has no team byte
        buffs: Vec::new(), // the subset files end after the inventory
        spawn_points: Vec::new(),
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
        // The subset files end after the inventory — the per-player tail was
        // never written; the fresh defaults stand in (the back-compat gate).
        current_loadout: 0,
        loadouts: Default::default(),
        void_vault_info: 0,
        bank: [(0, 0, 0); 40],
        bank2: [(0, 0, 0); 40],
        bank3: [(0, 0, 0); 40],
        bank4: [(0, 0, 0, false); 40],
        research: Vec::new(),
        // W4-f44 ph2: the subset carries armour/dye + the hide BitsBytes;
        // everything past the inventory is fresh defaults.
        armor,
        dye,
        misc: [(0, 0, 0, 0); 5],
        hide_visible_accessory,
        hide_misc: 0,
        hide_info: [false; 13],
        powers: CreativePowersSave::default(),
        angler_quests_finished: 0,
        dpad_bindings: [-1; 4],
        builder_acc_status: [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        bartender_quest_log: 0,
        dead: false,
        respawn_timer: 0,
        golfer_score: 0,
        temp_slots: [None; 4],
        super_cart: 2,
        voice_variant: 1,
        voice_pitch_offset: 0.0,
        refunds: Vec::new(),
        dialogues: Vec::new(),
    })
}

/// The created character's inventory: the vanilla ctor's starter items are
/// the copper shortsword/axe/pickaxe in slots 0-2 (`Player.cs:54356-54360`).
/// The writer serializes whatever the caller passes.
pub struct PlayerFileWrite {
    pub name: String,
    pub difficulty: u8,
    /// `PlayerFileData.GetPlayTime().Ticks` (`:54754`) — the play time the
    /// caller carries (0 = a fresh character, `TimeSpan.Zero`).
    pub play_time_ticks: i64,
    /// `Player.hairDye` (`:54756`), default 0 = no dye.
    pub hair_dye: u8,
    /// `(byte)Player.team` (`:54757`), default 0 = no team.
    pub team: u8,
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
    /// `Player.CurrentLoadoutIndex` (`:54919`) — clamped to 0..2 on write
    /// like the load clamp (`:55722`).
    pub current_loadout: i32,
    /// `Player.Loadouts[3]` (`:54920-54923`, `EquipmentLoadout.Serialize`,
    /// `EquipmentLoadout.cs:31-45`).
    pub loadouts: [LoadoutData; 3],
    /// `Player.voidVaultInfo` (`:54863`) — bit 0 the vault-enabled flag.
    pub void_vault_info: u8,
    /// `Player.buffType`/`buffTime`'s live pairs (`:54864-54876`) — the write
    /// pads to maxBuffs = 44 with (0, 0) and zeroes the `Main.buffNoSave`
    /// types; a fresh player has none.
    pub buffs: Vec<(i32, i32)>,
    /// `Player.spX/spY/spI/spN`'s bed spawn points (`:54877-54888`) — the
    /// write emits up to 200 records then the -1 sentinel; a fresh player's
    /// spN[200] is all null, so only the sentinel is written.
    pub spawn_points: Vec<(i32, i32, i32, String)>,
    /// `Player.bank`'s 40 slots (`:54838-54843`) — the piggy bank.
    pub bank: [(i32, i32, u8); 40],
    /// `Player.bank2`'s 40 slots (`:54844-54849`) — the safe.
    pub bank2: [(i32, i32, u8); 40],
    /// `Player.bank3`'s 40 slots (`:54850-54855`) — the defender's forge.
    pub bank3: [(i32, i32, u8); 40],
    /// `Player.bank4`'s 40 slots (`:54856-54862`).
    pub bank4: [(i32, i32, u8, bool); 40],
    /// The Journey research pairs (`:54911` -> `ItemsSacrificedUnlocksTracker
    /// .Save`, `ItemsSacrificedUnlocksTracker.cs:157-169`).
    pub research: Vec<(String, i32)>,
    /// `Player.armor[20]` as `(type, prefix, favorited)` (`:54812-54817`) —
    /// no stack; the favourited bool is unconditional at 326.
    pub armor: [(i32, u8, bool); 20],
    /// `Player.dye[10]`, the same shape (`:54818-54823`).
    pub dye: [(i32, u8, bool); 10],
    /// `Player.miscEquips[5]` + `Player.miscDyes[5]` interleaved per slot —
    /// `(equip type, equip prefix, dye type, dye prefix)` (`:54831-54837`).
    pub misc: [(i32, u8, i32, u8); 5],
    /// `Player.hideVisibleAccessory[10]` — the two BitsBytes (`:54758-54769`).
    pub hide_visible_accessory: [bool; 10],
    /// `Player.hideMisc` BitsByte (`:54770`).
    pub hide_misc: u8,
    /// `Player.hideInfo[13]` (`:54890-54893`).
    pub hide_info: [bool; 13],
    /// The Journey per-player creative powers (`:54913` ->
    /// `CreativePowerManager.SaveToPlayer`, `CreativePowerManager.cs:183-195`).
    /// Defaults to [`CreativePowersSave::written_fresh`] — the non-Journey
    /// live state the golden fixture pinned.
    pub powers: CreativePowersSave,
    /// `Player.anglerQuestsFinished` (`:54894`), default 0.
    pub angler_quests_finished: i32,
    /// `Player.DpadRadial.Bindings[4]` (`:54895-54898`), default all -1.
    pub dpad_bindings: [i32; 4],
    /// `Player.builderAccStatus[12]` (`:54899-54902`), default `[1, 0..]`.
    pub builder_acc_status: [i32; 12],
    /// `Player.bartenderQuestLog` (`:54903`), default 0.
    pub bartender_quest_log: i32,
    /// `Player.dead` (`:54904`) — the respawnTimer is written only when dead.
    pub dead: bool,
    /// `Player.respawnTimer` (`:54905-54908`, written verbatim when dead).
    pub respawn_timer: i32,
    /// `Player.golferScoreAccumulated` (`:54910`), default 0.
    pub golfer_score: i32,
    /// `(byte)Player.voiceVariant` (`:54924`), default 1 (the ctor `:56045`).
    pub voice_variant: u8,
    /// `Player.voicePitchOffset` (`:54925`), default 0f.
    pub voice_pitch_offset: f32,
    /// `Player._temporaryItemSlots[4]` (`:54912` ->
    /// `SaveTemporaryItemSlotContents:54935-54959`): `Some(triple)` writes
    /// the slot's bit + payload, `None` = air. A fresh save is all-`None`.
    pub temp_slots: [Option<(i32, i32, u8)>; 4],
    /// The super-cart BitsByte (`:54914-54918`): bit 0 unlockedSuperCart,
    /// bit 1 enabledSuperCart — a fresh save is 2 (enabled true, `:2054`).
    pub super_cart: u8,
    /// `CraftingRequests.SavePossibleRefunds` (`:54926` ->
    /// `CraftingRequests.cs:353-362`): the consumed-item triples over the
    /// pending crafts. A fresh queue is empty → count 0, no payloads.
    pub refunds: Vec<(i32, i32, u8)>,
    /// `Player.oneTimeDialoguesSeen` (`:54927-54932`): count + the strings.
    /// A fresh HashSet is empty → count 0. The write dedupes like the set.
    pub dialogues: Vec<String>,
}

impl Default for PlayerFileWrite {
    /// The fresh-vanilla defaults for every field — the same values the
    /// pre-model writer hardcoded (the module-level default-value policy).
    fn default() -> Self {
        PlayerFileWrite {
            name: String::new(),
            difficulty: 0,
            play_time_ticks: 0,
            hair_dye: 0,
            team: 0,
            buffs: Vec::new(),
            spawn_points: Vec::new(),
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
            current_loadout: 0,
            loadouts: Default::default(),
            void_vault_info: 0,
            bank: [(0, 0, 0); 40],
            bank2: [(0, 0, 0); 40],
            bank3: [(0, 0, 0); 40],
            bank4: [(0, 0, 0, false); 40],
            research: Vec::new(),
            armor: [(0, 0, false); 20],
            dye: [(0, 0, false); 10],
            misc: [(0, 0, 0, 0); 5],
            hide_visible_accessory: [false; 10],
            hide_misc: 0,
            hide_info: [false; 13],
            powers: CreativePowersSave::written_fresh(),
            angler_quests_finished: 0,
            dpad_bindings: [-1; 4],
            builder_acc_status: [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            bartender_quest_log: 0,
            dead: false,
            respawn_timer: 0,
            golfer_score: 0,
            temp_slots: [None; 4],
            super_cart: 2,
            voice_variant: 1, // the ctor (`Player.cs:56045`)
            voice_pitch_offset: 0.0,
            refunds: Vec::new(),
            dialogues: Vec::new(),
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
    w.u64(file.play_time_ticks as u64); // `:54754` playTime.Ticks (fresh Zero)
    w.i32(file.hair); // `:54755`
    w.u8(file.hair_dye); // `:54756` hairDye (default 0)
    w.u8(file.team); // `:54757` (byte)team (`Player.cs:1430`, default 0 = none)

    // `:54758-54763` hideVisibleAccessory[0..8] BitsByte — the caller's bits
    // (W4-f44 ph2); a fresh player's are all false.
    let mut hide1 = 0u8;
    for (i, hide) in file.hide_visible_accessory[..8].iter().enumerate() {
        hide1 |= u8::from(*hide) << i;
    }
    w.u8(hide1);
    // `:54764-54769` hideVisibleAccessory[8..10] BitsByte.
    let mut hide2 = 0u8;
    for (i, hide) in file.hide_visible_accessory[8..].iter().enumerate() {
        hide2 |= u8::from(*hide) << i;
    }
    w.u8(hide2);
    w.u8(file.hide_misc); // `:54770` hideMisc BitsByte (`Player.cs:1710` = 0 fresh)
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
    // W4-f44 ph2: the caller's slots ride verbatim.
    for (id, prefix, fav) in file.armor {
        w.i32(id);
        w.u8(prefix);
        w.boolean(fav);
    }
    // dye[10]: type, prefix, favorited (`:54818-54823`).
    for (id, prefix, fav) in file.dye {
        w.i32(id);
        w.u8(prefix);
        w.boolean(fav);
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
    // (ctor `:56064-56071`). W4-f44 ph2: the caller's pairs ride verbatim.
    for (eq_id, eq_prefix, dye_id, dye_prefix) in file.misc {
        w.i32(eq_id);
        w.u8(eq_prefix);
        w.i32(dye_id);
        w.u8(dye_prefix);
    }
    // bank / bank2 / bank3: 40 x (type, stack, prefix) each
    // (`:54838-54855`); `Chest.CreateBank`'s maxItems = 40 (`Chest.cs:142-144`,
    // `:191`), filled with empty `Item`s (ctor `:56056-56058`). W4-f44 ph3:
    // the caller's slots ride verbatim (a fresh player's are all-air).
    for bank in [&file.bank, &file.bank2, &file.bank3] {
        for (id, stack, prefix) in bank.iter() {
            w.i32(*id);
            w.i32(*stack);
            w.u8(*prefix);
        }
    }
    // bank4 (the void bag): 40 x (type, stack, prefix, favorited)
    // (`:54856-54862`); the favourited bool reads at >= 255 (`:55495`).
    // The caller's slots ride verbatim (a fresh player's are all-air).
    for (id, stack, prefix, fav) in file.bank4 {
        w.i32(id);
        w.i32(stack);
        w.u8(prefix);
        w.boolean(fav);
    }
    // voidVaultInfo: one BitsByte (`:54863`), default 0 (`Player.cs:1590`);
    // read at >= 199 (`Deserialize:55501-55504`). Bit 0 is IsVoidVaultEnabled
    // (`Player.cs:3900-3910`).
    w.u8(file.void_vault_info);
    // --- S1-J-5 increment 2: buffs through hideInfo (`:54864-54893`) ------
    // buffs: maxBuffs = 44 (`Player.cs:57106`) x (buffType, buffTime)
    // (`:54864-54876`). W4-f44 ph3: the caller's live pairs ride in order;
    // `Main.buffNoSave` types write (0, 0) (`:54866-54870`), the remaining
    // slots pad with the fresh all-zero buffType/buffTime (`:1510-1512`).
    for i in 0..44 {
        let (buff_type, buff_time) = file.buffs.get(i).copied().unwrap_or((0, 0));
        if buff_no_save(buff_type) {
            w.i32(0);
            w.i32(0);
        } else {
            w.i32(buff_type);
            w.i32(buff_time);
        }
    }
    // spawn points (`:54877-54888`): up to 200 (spX, spY, spI, spN) records;
    // the -1 sentinel is written at the first null spN and the loop breaks.
    // With all 200 slots non-null the loop writes 200 records and NO sentinel
    // (it exits on the bound, `:54877`). A fresh spN[200] is all null
    // (`Player.cs:2488`), so only the sentinel is written. W4-f44 ph3: the
    // caller's records ride verbatim.
    let spawn_count = file.spawn_points.len().min(200);
    for (x, y, world_id, world_name) in file.spawn_points.iter().take(200) {
        w.i32(*x);
        w.i32(*y);
        w.i32(*world_id);
        w.cs_string(world_name);
    }
    if spawn_count < 200 {
        w.i32(-1);
    }
    // hbLocked (`:54889`): default false (`Player.cs:1436`).
    w.boolean(false);
    // hideInfo[13] (`:54890-54893`): a fresh `bool[13]` (`Player.cs:812`),
    // all false — W4-f44 ph2: the caller's toggles ride verbatim.
    for hide in file.hide_info {
        w.boolean(hide);
    }
    // --- S1-J-5 increment 3: the Serialize tail (`:54894-54933`) ----------
    w.i32(file.angler_quests_finished); // `:54894` anglerQuestsFinished (`:1910`)
    // `:54895-54898` DpadRadial.Bindings[4] (`Player.cs:800`): the Dpad4
    // ctor sets all four to -1 (`Player.cs:618-621`) — the caller's ride.
    for binding in file.dpad_bindings {
        w.i32(binding);
    }
    // `:54899-54902` builderAccStatus[12] (`Player.cs:814`,
    // `BuilderAccToggleIDs.Count = 12`, `Player.cs:47-75`): a fresh int[12]
    // with the ctor's `builderAccStatus[0] = 1` (`:56092`) — the caller's ride.
    for status in file.builder_acc_status {
        w.i32(status);
    }
    w.i32(file.bartender_quest_log); // `:54903` bartenderQuestLog (`:1914`)
    // `:54904-54908` dead; the respawnTimer is written ONLY when dead —
    // a fresh save emits the single bool.
    w.boolean(file.dead);
    if file.dead {
        w.i32(file.respawn_timer);
    }
    // `:54909` DateTime.UtcNow.ToBinary() — the wall-clock field, carried
    // by the caller (see `PlayerFileWrite::last_saved_binary`).
    w.u64(file.last_saved_binary as u64);
    w.i32(file.golfer_score); // `:54910` golferScoreAccumulated (`:1912`)
              // `:54911` creativeTracker.Save -> CreativeUnlocksTracker.Save
              // (`CreativeUnlocksTracker.cs:9-12`) -> ItemsSacrificedUnlocksTracker.Save
              // (`ItemsSacrificedUnlocksTracker.cs:157-169`): the literal `false` flag
              // (`AnyNewUnlocksFromTeammates`, always written false) then the
              // sacrifice-pair count + the (persistentId, count) pairs — the
              // Journey research the caller carries (a fresh tracker has none).
    w.boolean(false);
    w.i32(file.research.len() as i32);
    for (key, count) in &file.research {
        w.cs_string(key);
        w.i32(*count);
    }
    // `:54912` SaveTemporaryItemSlotContents (`Player.cs:54935-54959`): one
    // BitsByte over mouseItem / CreativeMenu.GetItemByIndex(0) / guideItem /
    // reforgeItem IsAir, then an Item.Serialize triple per set bit. W4-f44
    // ph3: the caller's slots ride; a fresh save is all-air → 0, no payloads.
    let mut temp_bits = 0u8;
    for (i, slot) in file.temp_slots.iter().enumerate() {
        if slot.is_some() {
            temp_bits |= 1 << i;
        }
    }
    w.u8(temp_bits);
    for slot in file.temp_slots.iter().flatten() {
        w.i32(slot.0);
        w.i32(slot.1);
        w.u8(slot.2);
    }
    // `:54913` CreativePowerManager.SaveToPlayer
    // (`CreativePowerManager.cs:183-195`): for each registered power that is
    // IPersistentPerPlayerContent, `true` + the u16 id + its Save. The
    // registration order (`Initialize`, `:86-105`) gives GodmodePower id 5
    // (`IsEnabledForPlayer(Main.myPlayer)`, `CreativePowers.cs:705-707` —
    // fresh false), FarPlacementRangePower id 11 (`:744-746` — fresh false),
    // SpawnRateSliderPerPlayerPower id 14 (`_sliderCurrentValueCache`,
    // `:1717-1719` — fresh 0f). Then the `false` terminator. W4-f44 ph2:
    // the caller's live power state rides (the default is `written_fresh`).
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
    // (`Player.cs:2054`, default **true**) → a fresh save is 0b10 = 2.
    // W4-f44 ph3: the caller's byte rides verbatim.
    w.u8(file.super_cart);
    // `:54919` CurrentLoadoutIndex (`Player.cs:3207`, default 0) — the load
    // clamp is 0..Loadouts.Length-1 (`:55722`); the writer clamps the same.
    w.i32(file.current_loadout.clamp(0, 2));
    // `:54920-54923` Loadouts[3].Serialize (`Player.cs:3200-3204`;
    // `EquipmentLoadout.cs:31-45`): Armor[20] + Dye[10] each
    // Item.Serialize(writeFavorite: true) = (type, stack, prefix, favorited)
    // — fresh `Item()`s are (0, 0, 0, false) (`EquipmentLoadout.cs:14-19`) —
    // then Hide[10] bools. The caller's slots ride verbatim.
    for loadout in &file.loadouts {
        for (id, stack, prefix, fav) in loadout.armor.iter().chain(loadout.dye.iter()) {
            w.i32(*id);
            w.i32(*stack);
            w.u8(*prefix);
            w.boolean(*fav);
        }
        for hide in loadout.hide {
            w.boolean(hide);
        }
    }
    w.u8(file.voice_variant); // `:54924` (byte)voiceVariant (ctor 1, `:56045`)
    w.f32(file.voice_pitch_offset); // `:54925` voicePitchOffset (`:1348`)
    // `:54926` CraftingRequests.SavePossibleRefunds
    // (`CraftingRequests.cs:353-362`): the sum of consumed items over
    // _pendingCrafts, each Item.Serialize (no favourited). W4-f44 ph3: the
    // caller's triples ride; a fresh queue is empty → count 0, no payloads.
    w.i32(file.refunds.len() as i32);
    for (id, stack, prefix) in &file.refunds {
        w.i32(*id);
        w.i32(*stack);
        w.u8(*prefix);
    }
    // `:54927-54932` oneTimeDialoguesSeen (`Player.cs:3209`): a HashSet →
    // count + the strings. W4-f44 ph3: the caller's strings ride, deduped
    // first-occurrence like the set; a fresh HashSet is empty → count 0.
    let mut seen: Vec<&str> = Vec::new();
    for key in &file.dialogues {
        if !seen.contains(&key.as_str()) {
            seen.push(key);
        }
    }
    w.i32(seen.len() as i32);
    for key in seen {
        w.cs_string(key);
    }
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
            last_saved_binary: 0,
            ..Default::default()
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

    /// W4-f44 ph3: playTime ticks / hairDye / team round-trip and sit at the
    /// `Serialize:54754-54757` offsets (right after difficulty, before hair).
    #[test]
    fn the_header_fields_round_trip() {
        let written = PlayerFileWrite {
            play_time_ticks: 123_456_789_012,
            hair_dye: 7,
            team: 3,
            ..sample()
        };
        let plain = serialize_plaintext(&written);
        // The byte offsets: 24 prologue + name(1 len + 6 chars) + difficulty.
        let mut r = Reader::new(&plain);
        r.skip(24).unwrap();
        assert_eq!(r.cs_string().unwrap(), "Belsin");
        assert_eq!(r.u8().unwrap(), 0); // difficulty `:54753`
        assert_eq!(r.u64().unwrap(), 123_456_789_012); // playTime `:54754`
        assert_eq!(r.i32().unwrap(), 0); // hair `:54755`
        assert_eq!(r.u8().unwrap(), 7); // hairDye `:54756`
        assert_eq!(r.u8().unwrap(), 3); // team `:54757`
        let file = read_player_bytes(&cbc_decrypt(&write_player_bytes(&written))).unwrap();
        assert_eq!(file.play_time_ticks, 123_456_789_012);
        assert_eq!(file.hair_dye, 7);
        assert_eq!(file.team, 3);
        // A fresh write keeps the golden zeros (`:54754-54757`).
        let fresh = read_player_bytes(&cbc_decrypt(&write_player_bytes(&sample()))).unwrap();
        assert_eq!((fresh.play_time_ticks, fresh.hair_dye, fresh.team), (0, 0, 0));
    }

    /// W4-f44 ph3: the one-time dialogues round-trip as count + strings,
    /// deduped first-occurrence on both sides (the HashSet `.Add`); a fresh
    /// set is count 0.
    #[test]
    fn the_dialogues_round_trip_deduped() {
        let dialogues = vec!["First".to_string(), "Second".to_string(), "Third".to_string()];
        let written = PlayerFileWrite { dialogues: dialogues.clone(), ..sample() };
        let plain = serialize_plaintext(&written);
        // The dialogue count is the 4 bytes before the strings, at the end.
        let mut total = 4usize;
        for key in &dialogues {
            total += 1 + key.len(); // the 7-bit length prefix + ASCII bytes
        }
        let count_at = plain.len() - total;
        assert_eq!(
            i32::from_le_bytes(plain[count_at..count_at + 4].try_into().unwrap()),
            3
        );
        let file = read_player_bytes(&plain).unwrap();
        assert_eq!(file.dialogues, dialogues);
        // Duplicates collapse on write (the HashSet cannot hold them).
        let written = PlayerFileWrite {
            dialogues: vec!["A".into(), "B".into(), "A".into(), "C".into(), "B".into()],
            ..sample()
        };
        let file = read_player_bytes(&cbc_decrypt(&write_player_bytes(&written))).unwrap();
        assert_eq!(file.dialogues, vec!["A", "B", "C"]);
        // A file that somehow carries duplicates: the read `.Add` dedupes.
        let mut w = Writer::new();
        let base = serialize_plaintext(&sample());
        // Replace the trailing dialogue count 0 with 3 + "A","A","B".
        w.bytes(&base[..base.len() - 4]);
        w.i32(3);
        w.cs_string("A");
        w.cs_string("A");
        w.cs_string("B");
        let file = read_player_bytes(&w.into_inner()).unwrap();
        assert_eq!(file.dialogues, vec!["A", "B"]);
        // Fresh: count 0 (golden parity).
        let fresh = read_player_bytes(&cbc_decrypt(&write_player_bytes(&sample()))).unwrap();
        assert!(fresh.dialogues.is_empty());
    }

    /// W4-f44 ph3: the crafting refunds round-trip as count + triples; an
    /// over-count id airs after the whole record; a fresh queue is count 0.
    #[test]
    fn the_refunds_round_trip_as_count_and_triples() {
        let refunds = vec![(757, 1, 81), (2, 999, 0), (3507, 1, 0)];
        let written = PlayerFileWrite { refunds: refunds.clone(), ..sample() };
        let plain = serialize_plaintext(&written);
        // The refunds sit after the voice pair, before the dialogues. Walk
        // to them: the tail's fixed prefix is 1425 - 4 dialogues - (4 + 9*n)
        // refunds... simpler: read back and also pin the count bytes by
        // locating the dialogue count at the very end.
        let file = read_player_bytes(&plain).unwrap();
        assert_eq!(file.refunds, refunds);
        // The last 4 bytes are the dialogue count (0); the 4 before the
        // refund payloads are the refund count.
        let n = refunds.len();
        let refund_count_at = plain.len() - 4 - (9 * n) - 4;
        assert_eq!(
            i32::from_le_bytes(plain[refund_count_at..refund_count_at + 4].try_into().unwrap()),
            n as i32
        );
        // An over-count id airs id only, stack/prefix ride (`Item.cs:49930`).
        let written = PlayerFileWrite {
            refunds: vec![(ITEM_ID_COUNT + 2, 7, 9), (8, 99, 0)],
            ..sample()
        };
        let file = read_player_bytes(&cbc_decrypt(&write_player_bytes(&written))).unwrap();
        assert_eq!(file.refunds, vec![(0, 7, 9), (8, 99, 0)]);
        // Fresh: count 0, no payloads (golden parity).
        let fresh = read_player_bytes(&cbc_decrypt(&write_player_bytes(&sample()))).unwrap();
        assert!(fresh.refunds.is_empty());
    }

    /// W4-f44 ph3: the voice pair round-trips; the read clamps the variant
    /// to 1..4 (`:55831`); a pre-280 file derives the variant from the
    /// skinVariant's Male set and zeroes the pitch.
    #[test]
    fn the_voice_pair_round_trips_clamps_and_derives() {
        let written = PlayerFileWrite {
            voice_variant: 3,
            voice_pitch_offset: -0.25,
            ..sample()
        };
        let file = read_player_bytes(&cbc_decrypt(&write_player_bytes(&written))).unwrap();
        assert_eq!(file.voice_variant, 3);
        assert_eq!(file.voice_pitch_offset, -0.25);
        // The clamp: 0 → 1, 9 → 4 (`Utils.Clamp(1, 4)`, `:55831`).
        for (raw, want) in [(0u8, 1u8), (5, 4), (255, 4), (2, 2)] {
            let written = PlayerFileWrite { voice_variant: raw, ..sample() };
            let file = read_player_bytes(&cbc_decrypt(&write_player_bytes(&written))).unwrap();
            assert_eq!(file.voice_variant, want);
        }
        // A pre-280 body: hand-built at 279's gates (no team < 283, no 324
        // bool, no armour/dye favourites < 322, no bank4 favourites < 255,
        // no tracker flag < 282, no voice bytes) — ends after the loadouts.
        fn body_279(skin_variant: u8) -> Vec<u8> {
            let mut w = Writer::new();
            w.i32(279);
            w.u64(FILE_METADATA_MAGIC | (FILE_TYPE_PLAYER << 56));
            w.u32(0);
            w.u64(0);
            w.cs_string("Belsin");
            w.u8(0); // difficulty
            w.u64(0); // playtime
            w.i32(0); // hair
            w.u8(0); // hairDye — no team at 279
            w.bytes(&[0, 0]); // hide BitsBytes
            w.u8(0); // hideMisc
            w.u8(skin_variant);
            for v in [100, 100, 20, 20] {
                w.i32(v);
            }
            for _ in 0..11 {
                w.boolean(false); // extra + torch pair + bread + 6 unlocks + dd2
            }
            w.i32(0); // tax
            w.i32(0); // deathsPVE (254)
            w.i32(0); // deathsPVP (254)
            w.bytes(&[0; 21]); // colors
            for _ in 0..30 {
                w.i32(0); // armor + dye pairs, no favourited (< 322)
                w.u8(0);
            }
            for _ in 0..58 {
                w.bytes(&[0; 10]); // inventory triples + favourited
            }
            for _ in 0..5 {
                w.bytes(&[0; 10]); // misc pairs
            }
            for _ in 0..120 {
                w.bytes(&[0; 9]); // bank + bank2 + bank3 triples
            }
            for _ in 0..40 {
                w.bytes(&[0; 10]); // bank4 with favourited (>= 255)
            }
            w.u8(0); // voidVaultInfo (199)
            for _ in 0..44 {
                w.i32(0); // buffs (>= 252)
                w.i32(0);
            }
            w.i32(-1); // spawn sentinel
            w.boolean(false); // hbLocked
            w.bytes(&[0; 13]); // hideInfo
            w.i32(0); // angler
            for _ in 0..4 {
                w.i32(-1); // dpad
            }
            for i in 0..12 {
                w.i32(i32::from(i == 0)); // builderAcc (>= 230)
            }
            w.i32(0); // bartender
            w.boolean(false); // dead
            w.u64(0); // timestamp
            w.i32(0); // golfer (206)
            w.i32(0); // tracker count (218) — no flag (< 282)
            w.u8(0); // temp-slot BitsByte (214)
            w.boolean(false); // powers terminator (220)
            w.u8(2); // superCart (253)
            w.i32(0); // CurrentLoadoutIndex (262)
            for _ in 0..3 {
                for _ in 0..30 {
                    w.bytes(&[0; 9]); // loadout items, no favourited (< 322)
                }
                w.bytes(&[0; 10]); // Hide[10]
            }
            w.into_inner() // the 279 file ends before the voice bytes
        }
        // Male variants {0,1,2,3,8,10} → 1; female {4,5,6,7,9,11} → 2.
        for variant in [0u8, 1, 2, 3, 8, 10] {
            let file = read_player_bytes(&body_279(variant)).unwrap();
            assert_eq!(file.voice_variant, 1, "variant {variant} is Male");
            assert_eq!(file.voice_pitch_offset, 0.0);
        }
        for variant in [4u8, 5, 6, 7, 9, 11] {
            let file = read_player_bytes(&body_279(variant)).unwrap();
            assert_eq!(file.voice_variant, 2, "variant {variant} is female");
            assert_eq!(file.voice_pitch_offset, 0.0);
        }
    }

    /// W4-f44 ph3: the super-cart BitsByte round-trips verbatim; a pre-253
    /// file derives the unlock from a 3353 in any inventory (stack > 0) and
    /// keeps the enabled default true.
    #[test]
    fn the_super_cart_byte_rides_and_the_legacy_unlock_derives() {
        for byte in [0u8, 1, 2, 3, 0xFF] {
            let written = PlayerFileWrite { super_cart: byte, ..sample() };
            let file = read_player_bytes(&cbc_decrypt(&write_player_bytes(&written))).unwrap();
            assert_eq!(file.super_cart, byte);
        }
        // The fresh default is 2 (enabled true, unlocked false).
        let fresh = read_player_bytes(&cbc_decrypt(&write_player_bytes(&sample()))).unwrap();
        assert_eq!(fresh.super_cart, 2);
        // A pre-253 body, hand-built at its own gates (no team < 283, no
        // bread/unlock-block < 256/260, no 324 bool, no deaths < 254, no
        // armour/dye favourites < 322, no bank4 favourites < 255, no
        // tracker flag < 282) — ends right before the superCart byte.
        fn body_252(inventory: [(i32, i32, u8); 58]) -> Vec<u8> {
            let mut w = Writer::new();
            w.i32(252);
            w.u64(FILE_METADATA_MAGIC | (FILE_TYPE_PLAYER << 56));
            w.u32(0);
            w.u64(0);
            w.cs_string("Belsin");
            w.u8(0); // difficulty
            w.u64(0); // playtime
            w.i32(0); // hair
            w.u8(0); // hairDye — no team at 252
            w.bytes(&[0, 0]); // hide BitsBytes
            w.u8(0); // hideMisc
            w.u8(0); // skinVariant
            for v in [100, 100, 20, 20] {
                w.i32(v);
            }
            w.boolean(false); // extraAccessory (125)
            w.boolean(false); // unlockedBiomeTorches (229)
            w.boolean(false); // UsingBiomeTorches (229)
            w.boolean(false); // dd2 (182)
            w.i32(0); // tax (128) — no deaths at 252
            w.bytes(&[0; 21]); // colors
            for _ in 0..30 {
                w.i32(0); // armor + dye pairs, no favourited (< 322)
                w.u8(0);
            }
            for (id, stack, prefix) in inventory {
                w.i32(id);
                w.i32(stack);
                w.u8(prefix);
                w.boolean(false);
            }
            for _ in 0..5 {
                w.bytes(&[0; 10]); // misc pairs
            }
            for _ in 0..120 {
                w.bytes(&[0; 9]); // bank + bank2 + bank3 triples
            }
            for _ in 0..40 {
                w.bytes(&[0; 9]); // bank4, no favourited (< 255)
            }
            w.u8(0); // voidVaultInfo (199)
            for _ in 0..44 {
                w.i32(0); // buffs: 44 at 252 (>= 252)
                w.i32(0);
            }
            w.i32(-1); // spawn sentinel
            w.boolean(false); // hbLocked
            w.bytes(&[0; 13]); // hideInfo
            w.i32(0); // angler (98)
            for _ in 0..4 {
                w.i32(-1); // dpad (162)
            }
            for i in 0..12 {
                w.i32(i32::from(i == 0)); // builderAcc: 12 at 252 (>= 230)
            }
            w.i32(0); // bartender (181)
            w.boolean(false); // dead (200)
            w.u64(0); // timestamp (202)
            w.i32(0); // golfer (206)
            w.i32(0); // tracker count (218) — no flag (< 282)
            w.u8(0); // temp-slot BitsByte (214)
            w.boolean(false); // powers terminator (220) — the 252 file ends here
            w.into_inner()
        }
        let file = read_player_bytes(&body_252([(0, 0, 0); 58])).unwrap();
        assert_eq!(file.super_cart, 2); // no 3353 anywhere → unlock false
        // With a 3353 (stack 1) in inventory slot 10: unlock true → 3.
        let mut inv = [(0i32, 0i32, 0u8); 58];
        inv[10] = (3353, 1, 0);
        let file = read_player_bytes(&body_252(inv)).unwrap();
        assert_eq!(file.super_cart, 3);
        // A zero-stack 3353 does NOT count (`HasItem` needs stack > 0).
        inv[10] = (3353, 0, 0);
        let file = read_player_bytes(&body_252(inv)).unwrap();
        assert_eq!(file.super_cart, 2);
    }

    /// W4-f44 ph3: the four temporary item slots round-trip through the
    /// BitsByte + per-set-bit triples; air slots carry no payload.
    #[test]
    fn the_temp_slots_round_trip_through_the_bits_byte() {
        // Bits 0 and 2 set (mouse + guide), 1 and 3 air.
        let written = PlayerFileWrite {
            temp_slots: [Some((757, 1, 81)), None, Some((2, 999, 0)), None],
            ..sample()
        };
        let plain = serialize_plaintext(&written);
        let file = read_player_bytes(&plain).unwrap();
        assert_eq!(file.temp_slots, [Some((757, 1, 81)), None, Some((2, 999, 0)), None]);
        // All four set: the BitsByte is 0x0F and four triples follow.
        let written = PlayerFileWrite {
            temp_slots: [Some((1, 1, 0)), Some((2, 2, 0)), Some((3, 3, 0)), Some((4, 4, 0))],
            ..sample()
        };
        let file = read_player_bytes(&cbc_decrypt(&write_player_bytes(&written))).unwrap();
        assert_eq!(file.temp_slots, [Some((1, 1, 0)), Some((2, 2, 0)), Some((3, 3, 0)), Some((4, 4, 0))]);
        // An over-count id airs after the whole record (`Item.cs:49930`).
        let written = PlayerFileWrite {
            temp_slots: [Some((ITEM_ID_COUNT + 1, 5, 9)), None, None, None],
            ..sample()
        };
        let file = read_player_bytes(&cbc_decrypt(&write_player_bytes(&written))).unwrap();
        assert_eq!(file.temp_slots[0], Some((0, 5, 9)));
        // Fresh: the lone 0 BitsByte, no payloads (golden parity).
        let fresh = read_player_bytes(&cbc_decrypt(&write_player_bytes(&sample()))).unwrap();
        assert_eq!(fresh.temp_slots, [None; 4]);
    }

    /// W4-f44 ph3: angler/dpad/builderAcc/bartender/dead/golfer round-trip;
    /// the respawnTimer rides only when dead and clamps on read.
    #[test]
    fn the_quest_and_death_fields_round_trip() {
        let written = PlayerFileWrite {
            angler_quests_finished: 42,
            dpad_bindings: [3, -1, 17, 0],
            builder_acc_status: [0, 1, 1, 0, 1, 0, 0, 1, 0, 0, 1, 1],
            bartender_quest_log: 7,
            dead: true,
            respawn_timer: 1234,
            golfer_score: -50,
            ..sample()
        };
        let file = read_player_bytes(&cbc_decrypt(&write_player_bytes(&written))).unwrap();
        assert_eq!(file.angler_quests_finished, 42);
        assert_eq!(file.dpad_bindings, [3, -1, 17, 0]);
        assert_eq!(file.builder_acc_status, [0, 1, 1, 0, 1, 0, 0, 1, 0, 0, 1, 1]);
        assert_eq!(file.bartender_quest_log, 7);
        assert!(file.dead);
        assert_eq!(file.respawn_timer, 1234);
        assert_eq!(file.golfer_score, -50);
        // Alive: no respawnTimer bytes — the read keeps the default 0.
        let alive = PlayerFileWrite { dead: false, respawn_timer: 999, ..sample() };
        let plain = serialize_plaintext(&alive);
        let file = read_player_bytes(&plain).unwrap();
        assert!(!file.dead);
        assert_eq!(file.respawn_timer, 0);
        // The clamp: `Utils.Clamp(0, 60000)` (`:55684`).
        let over = PlayerFileWrite { dead: true, respawn_timer: 99999, ..sample() };
        let file = read_player_bytes(&cbc_decrypt(&write_player_bytes(&over))).unwrap();
        assert_eq!(file.respawn_timer, 60000);
        let under = PlayerFileWrite { dead: true, respawn_timer: -5, ..sample() };
        let file = read_player_bytes(&cbc_decrypt(&write_player_bytes(&under))).unwrap();
        assert_eq!(file.respawn_timer, 0);
        // The fresh defaults keep the golden bytes (builderAcc [0]=1, dpad -1).
        let fresh = read_player_bytes(&cbc_decrypt(&write_player_bytes(&sample()))).unwrap();
        assert_eq!(fresh.dpad_bindings, [-1; 4]);
        assert_eq!(fresh.builder_acc_status, [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert!(!fresh.dead);
    }

    /// The < 210 builderAcc[0] = 1 fixup and the < 249 inventory-3611
    /// builderAcc[1] = 1 fixup (`Deserialize:55654-55673`): state-only, the
    /// byte stream stays aligned. The 205 body is hand-built at its own
    /// gates (no team/unlock-block/deaths/favourites, 22 buffs, 11 builder
    /// slots, ends after the timestamp) — the same pattern as the legacy
    /// subset witness.
    #[test]
    fn the_builder_acc_legacy_fixups_apply_without_extra_bytes() {
        fn body_205(inventory: [(i32, i32, u8); 58]) -> Vec<u8> {
            let mut w = Writer::new();
            w.i32(205);
            w.u64(FILE_METADATA_MAGIC | (FILE_TYPE_PLAYER << 56));
            w.u32(0);
            w.u64(0);
            w.cs_string("Belsin");
            w.u8(0); // difficulty
            w.u64(0); // playtime
            w.i32(0); // hair
            w.u8(0); // hairDye — no team at 205 (< 283)
            w.bytes(&[0, 0]); // hide BitsBytes
            w.u8(0); // hideMisc
            w.u8(0); // skinVariant
            for v in [100, 100, 20, 20] {
                w.i32(v);
            }
            w.boolean(false); // extraAccessory (125) — no 229 block at 205
            w.boolean(false); // dd2 (182)
            w.i32(0); // tax (128) — no deaths at 205 (< 254)
            w.bytes(&[0; 21]); // colors
            for _ in 0..30 {
                w.i32(0); // armor + dye pairs, no favourited (< 322)
                w.u8(0);
            }
            for (id, stack, prefix) in inventory {
                w.i32(id);
                w.i32(stack);
                w.u8(prefix);
                w.boolean(false);
            }
            for _ in 0..5 {
                w.bytes(&[0; 10]); // misc pairs
            }
            for _ in 0..120 {
                w.bytes(&[0; 9]); // bank + bank2 + bank3 triples
            }
            for _ in 0..40 {
                w.bytes(&[0; 9]); // bank4, no favourited (< 255)
            }
            w.u8(0); // voidVaultInfo (199)
            for _ in 0..22 {
                w.i32(0); // buffs: 22 at 205 (< 252)
                w.i32(0);
            }
            w.i32(-1); // spawn sentinel
            w.boolean(false); // hbLocked
            w.bytes(&[0; 13]); // hideInfo
            w.i32(0); // angler (98)
            for _ in 0..4 {
                w.i32(-1); // dpad (162)
            }
            for _ in 0..11 {
                w.i32(0); // builderAcc: 11 at 205 (>= 197, < 230)
            }
            w.i32(0); // bartender (181)
            w.boolean(false); // dead (200)
            w.u64(0); // timestamp (202) — the 205 file ends here
            w.into_inner()
        }
        // builderAcc[0] reads 0 from the file; 205 < 210 forces it to 1.
        let file = read_player_bytes(&body_205([(0, 0, 0); 58])).unwrap();
        assert_eq!(file.builder_acc_status[0], 1);
        // 205 < 249 and no 3611 in the inventory: [1] stays 0.
        assert_eq!(file.builder_acc_status[1], 0);
        assert_eq!(file.builder_acc_status[11], 0); // the 12th slot never read
        // The same body with a 3611 in inventory slot 5: [1] forced to 1.
        let mut inv = [(0i32, 0i32, 0u8); 58];
        inv[5] = (3611, 1, 0);
        let file = read_player_bytes(&body_205(inv)).unwrap();
        assert_eq!(file.builder_acc_status[0], 1);
        assert_eq!(file.builder_acc_status[1], 1);
        assert_eq!(file.inventory[5], (3611, 1, 0));
    }

    /// W4-f44 ph3: spawn points round-trip in order, the write emits the
    /// records then the -1 sentinel, and the 200-record cap holds.
    #[test]
    fn the_spawn_points_round_trip_with_the_sentinel() {
        let points = vec![
            (100, 200, 7, "Alpha".to_string()),
            (0, 0, 0, String::new()), // an empty world name still rides
            (4000, 1200, 3, "Beta".to_string()),
        ];
        let written = PlayerFileWrite { spawn_points: points.clone(), ..sample() };
        let plain = serialize_plaintext(&written);
        // The spawn block starts at 2402 + 352 buffs = 2754.
        let mut r = Reader::new(&plain);
        r.skip(2754).unwrap();
        assert_eq!(r.i32().unwrap(), 100);
        assert_eq!(r.i32().unwrap(), 200);
        assert_eq!(r.i32().unwrap(), 7);
        assert_eq!(r.cs_string().unwrap(), "Alpha");
        assert_eq!(r.i32().unwrap(), 0);
        assert_eq!(r.i32().unwrap(), 0);
        assert_eq!(r.i32().unwrap(), 0);
        assert_eq!(r.cs_string().unwrap(), "");
        assert_eq!(r.i32().unwrap(), 4000);
        assert_eq!(r.i32().unwrap(), 1200);
        assert_eq!(r.i32().unwrap(), 3);
        assert_eq!(r.cs_string().unwrap(), "Beta");
        assert_eq!(r.i32().unwrap(), -1); // the sentinel `:54881`
        let file = read_player_bytes(&plain).unwrap();
        assert_eq!(file.spawn_points, points);
        // The 200 cap: record 201 never reaches the file (`:54877`).
        let many: Vec<_> = (0..205).map(|i| (i, i, 0, format!("W{i}"))).collect();
        let written = PlayerFileWrite { spawn_points: many.clone(), ..sample() };
        let file = read_player_bytes(&cbc_decrypt(&write_player_bytes(&written))).unwrap();
        assert_eq!(file.spawn_points.len(), 200);
        assert_eq!(file.spawn_points[199], many[199]);
        // A fresh write is the lone sentinel (golden parity).
        let fresh = read_player_bytes(&cbc_decrypt(&write_player_bytes(&sample()))).unwrap();
        assert!(fresh.spawn_points.is_empty());
    }

    /// W4-f44 ph3: buffs round-trip through the compaction read and the
    /// padded write; `Main.buffNoSave` types zero on write; an over-long
    /// list truncates to maxBuffs = 44 (the C# array bound).
    #[test]
    fn the_buffs_round_trip_compacted_and_no_save_types_zero() {
        // Interleaved zero padding: the read compaction keeps only the live
        // pairs in file order (`:55589-55598`).
        let buffs = vec![(26, 3600), (113, 60), (1, 999)];
        let written = PlayerFileWrite { buffs: buffs.clone(), ..sample() };
        let file = read_player_bytes(&cbc_decrypt(&write_player_bytes(&written))).unwrap();
        assert_eq!(file.buffs, buffs);
        // A no-save type (20, `Main.cs:8969`) writes (0, 0) — the read drops
        // it via the zero-type compaction; its neighbours survive.
        let written = PlayerFileWrite { buffs: vec![(26, 60), (20, 500), (113, 7)], ..sample() };
        let plain = serialize_plaintext(&written);
        // The buffs start at 2402 (24 + 87 header + 120 + 60 + 580 + 50 +
        // 1080 + 400 + 1 vault).
        let mut r = Reader::new(&plain);
        r.skip(2402).unwrap();
        assert_eq!((r.i32().unwrap(), r.i32().unwrap()), (26, 60));
        assert_eq!((r.i32().unwrap(), r.i32().unwrap()), (0, 0)); // no-save 20
        assert_eq!((r.i32().unwrap(), r.i32().unwrap()), (113, 7));
        let file = read_player_bytes(&plain).unwrap();
        assert_eq!(file.buffs, vec![(26, 60), (113, 7)]);
        // The 173..=181 range is no-save too (`Main.cs:9070-9073`).
        // The range's neighbours outside the set stay savable (172/182 are
        // explicit no-save entries themselves, `Main.cs:9032-9033`).
        assert!(buff_no_save(173) && buff_no_save(181) && !buff_no_save(169) && !buff_no_save(183));
        // Over-long list: only the first 44 slots exist (`:54864`).
        let long: Vec<(i32, i32)> = (0..50).map(|i| (i + 500, i)).collect();
        let written = PlayerFileWrite { buffs: long.clone(), ..sample() };
        let file = read_player_bytes(&cbc_decrypt(&write_player_bytes(&written))).unwrap();
        assert_eq!(file.buffs.len(), 44);
        assert_eq!(file.buffs[43], long[43]);
    }

    /// W4-f44 ph3: the piggy bank / safe / forge slots round-trip, the write
    /// span is byte-faithful at the `:54838-54855` offsets, and an over-count
    /// id airs id+prefix while the stack rides (no desync).
    #[test]
    fn the_banks_round_trip_and_stay_byte_faithful() {
        let mut bank = [(0i32, 0i32, 0u8); 40];
        bank[0] = (757, 1, 81); // a prefixed sword
        bank[39] = (2, 999, 0); // dirt at the last slot
        let mut bank2 = [(0i32, 0i32, 0u8); 40];
        bank2[7] = (-15, 3, 0); // a legacy id rides verbatim
        let mut bank3 = [(0i32, 0i32, 0u8); 40];
        bank3[12] = (3507, 1, 57);
        let written = PlayerFileWrite { bank, bank2, bank3, ..sample() };
        let plain = serialize_plaintext(&written);
        // The bank span: 24 prologue + 87 header + 120 armor + 60 dye +
        // 580 inventory + 50 misc = 921; 3 x 40 x 9 = 1080 bytes.
        let mut r = Reader::new(&plain);
        r.skip(921).unwrap();
        for (id, stack, prefix) in bank.iter().chain(bank2.iter()).chain(bank3.iter()) {
            assert_eq!(r.i32().unwrap(), *id);
            assert_eq!(r.i32().unwrap(), *stack);
            assert_eq!(r.u8().unwrap(), *prefix);
        }
        // bank4 (all-air here) follows immediately.
        assert_eq!(r.take(10).unwrap(), &[0; 10]);
        let file = read_player_bytes(&cbc_decrypt(&write_player_bytes(&written))).unwrap();
        assert_eq!(file.bank, bank);
        assert_eq!(file.bank2, bank2);
        assert_eq!(file.bank3, bank3);
        // An over-count id airs id+prefix, the stack rides (`SetDefaults`
        // `Item.cs:48670-48688` then `Prefix` no-ops on air `:515-518`); the
        // later slots stay aligned.
        let mut over = [(0i32, 0i32, 0u8); 40];
        over[0] = (ITEM_ID_COUNT + 5, 42, 9);
        over[1] = (8, 99, 0);
        let written = PlayerFileWrite { bank: over, ..sample() };
        let file = read_player_bytes(&cbc_decrypt(&write_player_bytes(&written))).unwrap();
        assert_eq!(file.bank[0], (0, 42, 0));
        assert_eq!(file.bank[1], (8, 99, 0));
        assert_eq!(file.bank3, [(0, 0, 0); 40]);
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
            // The harness's player is fresh — the per-player tail stays at
            // the vanilla defaults (all-air loadouts/bank4, vault off, no
            // research), which is what the C# fixture wrote.
            ..Default::default()
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

    // --- W4-f44: the per-player payloads (loadouts / void vault / research) --

    /// A file carrying all three per-player payloads round-trips them: the
    /// loadout index + slots (`:54919-54923` / `:55720-55727`), the void
    /// vault's bank4 + info byte (`:54856-54863` / `:55488-55504`) and the
    /// Journey research pairs (`:54911` / `:55696-55699`).
    #[test]
    fn the_per_player_tail_round_trips() {
        let mut file = sample();
        file.current_loadout = 2;
        file.loadouts[2].armor[3] = (908, 1, 81, true); // Lava Waders, warding, starred
        file.loadouts[2].dye[0] = (1007, 1, 0, false);
        file.loadouts[2].hide[4] = true;
        file.loadouts[0].armor[0] = (89, 1, 0, false); // a second loadout keeps its own
        file.void_vault_info = 1; // IsVoidVaultEnabled (`Player.cs:3900-3910`)
        file.bank4[0] = (71, 250, 0, true); // copper coins, starred
        file.bank4[39] = (2, 999, 0, false);
        file.research = vec![("Torch".into(), 100), ("DirtBlock".into(), 37)];
        let bytes = write_player_bytes(&file);
        let read = read_player_bytes(&cbc_decrypt(&bytes)).expect("the tail file must read");
        assert_eq!(read.current_loadout, 2);
        assert_eq!(read.loadouts[2].armor[3], (908, 1, 81, true));
        assert_eq!(read.loadouts[2].dye[0], (1007, 1, 0, false));
        assert!(read.loadouts[2].hide[4]);
        assert!(!read.loadouts[2].hide[3]);
        assert_eq!(read.loadouts[0].armor[0], (89, 1, 0, false));
        assert_eq!(read.loadouts[1], LoadoutData::default(), "the untouched loadout stays fresh");
        assert_eq!(read.void_vault_info, 1);
        assert_eq!(read.bank4[0], (71, 250, 0, true));
        assert_eq!(read.bank4[39], (2, 999, 0, false));
        assert_eq!(read.bank4[1], (0, 0, 0, false));
        assert_eq!(read.research, vec![("Torch".into(), 100), ("DirtBlock".into(), 37)]);
        // The card state is unaffected by the tail.
        assert_eq!(read.inventory, file.inventory);
        assert_eq!(read.inv_favorites, file.inv_favorites);
    }

    /// The write side is byte-faithful: with a non-default tail the plaintext
    /// differs from the fresh file ONLY inside the three payload spans, and
    /// the fresh-default write still reproduces the C# golden fixture
    /// byte-for-byte (the default-value policy holds).
    #[test]
    fn the_tail_write_is_byte_faithful() {
        let fresh = serialize_plaintext(&sample());
        let mut file = sample();
        file.void_vault_info = 1;
        let with_vault = serialize_plaintext(&file);
        assert_eq!(fresh.len(), with_vault.len(), "the info byte swaps in place");
        // voidVaultInfo's offset: hideInfo ends at 2772 (the golden-layout
        // witness), the inc2 tail is 1055 bytes and the buffs..hideInfo block
        // 370 — so the inc1 cut is 2402 and the info byte sits at 2401.
        let diffs: Vec<usize> = fresh
            .iter()
            .zip(with_vault.iter())
            .enumerate()
            .filter(|(_, (a, b))| a != b)
            .map(|(i, _)| i)
            .collect();
        assert_eq!(diffs, vec![2401], "exactly the voidVaultInfo byte moved");
        // The load clamp: an out-of-range index writes clamped (`:55722`).
        file = sample();
        file.current_loadout = 9;
        let clamped = read_player_bytes(&cbc_decrypt(&write_player_bytes(&file))).unwrap();
        assert_eq!(clamped.current_loadout, 2);
        file.current_loadout = -3;
        let clamped = read_player_bytes(&cbc_decrypt(&write_player_bytes(&file))).unwrap();
        assert_eq!(clamped.current_loadout, 0);
    }

    /// Back-compat: the C# golden fixture (a fresh player) reads the tail as
    /// the fresh defaults — all-air loadouts/bank4, vault off, no research —
    /// through the same walk that now captures them.
    #[test]
    fn the_golden_fixture_tail_reads_fresh() {
        let plain = cbc_decrypt(include_bytes!("../../../assets/test/golden.plr"));
        let file = read_player_bytes(&plain).expect("the C# fixture must read");
        assert_eq!(file.current_loadout, 0);
        assert_eq!(file.loadouts, Default::default());
        assert_eq!(file.void_vault_info, 0);
        assert_eq!(file.bank4, [(0, 0, 0, false); 40]);
        assert!(file.research.is_empty());
        // W4-f44 ph2: the equip/dye/hide tails + the powers read fresh too.
        assert_eq!(file.armor, [(0, 0, false); 20]);
        assert_eq!(file.dye, [(0, 0, false); 10]);
        assert_eq!(file.misc, [(0, 0, 0, 0); 5]);
        assert_eq!(file.hide_visible_accessory, [false; 10]);
        assert_eq!(file.hide_misc, 0);
        assert_eq!(file.hide_info, [false; 13]);
        // A Classic fixture (difficulty 0): the loaded (false, false, 0f)
        // is RESET by `LoadToPlayer` (`CreativePowerManager.cs:211-214`).
        assert_eq!(file.powers, CreativePowersSave::default());
    }

    /// Back-compat gates: the port-only shallow retries (inc1 at the vault
    /// byte, inc2 at hideInfo) still load, and carry the sections they reached
    /// — inc1 has bank4 + the vault byte but no research/loadouts; inc2 the
    /// same; the legacy 279 subset has none of the tail.
    #[test]
    fn the_shallow_retries_keep_their_reached_tail() {
        let mut file = sample();
        file.void_vault_info = 1;
        file.bank4[7] = (8, 99, 0, true);
        file.research = vec![("Torch".into(), 100)];
        file.loadouts[1].armor[2] = (90, 1, 0, false);
        file.armor[0] = (89, 81, true);
        file.hide_info[2] = true;
        let plain = serialize_plaintext(&file);
        // inc1: cut after the voidVaultInfo byte (offset 2402).
        let inc1 = read_player_bytes(&plain[..2402]).expect("inc1 reads");
        assert_eq!(inc1.void_vault_info, 1);
        assert_eq!(inc1.bank4[7], (8, 99, 0, true));
        assert!(inc1.research.is_empty(), "inc1 never reached the research");
        assert_eq!(inc1.loadouts, Default::default());
        // W4-f44 ph2: the armor/hide BitsBytes sit BEFORE the cut — captured.
        assert_eq!(inc1.armor[0], (89, 81, true));
        assert_eq!(inc1.hide_info, [false; 13], "inc1 never reached hideInfo");
        // inc2: cut after hideInfo (offset 2772).
        let inc2 = read_player_bytes(&plain[..2772]).expect("inc2 reads");
        assert_eq!(inc2.void_vault_info, 1);
        assert_eq!(inc2.bank4[7], (8, 99, 0, true));
        assert!(inc2.research.is_empty());
        assert_eq!(inc2.loadouts, Default::default());
        assert_eq!(inc2.armor[0], (89, 81, true));
        assert!(inc2.hide_info[2], "inc2 reached hideInfo");
        assert_eq!(
            inc2.powers,
            CreativePowersSave::default(),
            "inc2 never reached the powers; the non-Journey reset stands"
        );
    }

    /// An over-count item type in bank4/loadouts airs the slot after the full
    /// record is consumed (`Item.DeserializeFrom`, `Item.cs:49930-49933`) —
    /// the walk stays aligned and the later sections still read.
    #[test]
    fn over_count_tail_items_air_without_desyncing() {
        let mut file = sample();
        file.bank4[0] = (ITEM_ID_COUNT, 5, 0, false); // >= Count → air
        file.loadouts[0].dye[9] = (ITEM_ID_COUNT + 10, 1, 3, true); // → air
        file.loadouts[2].armor[19] = (89, 1, 0, false); // a real item AFTER the aired one
        file.research = vec![("Torch".into(), 100)];
        let read = read_player_bytes(&cbc_decrypt(&write_player_bytes(&file))).unwrap();
        assert_eq!(read.bank4[0], (0, 0, 0, false));
        assert_eq!(read.loadouts[0].dye[9], (0, 0, 0, false));
        assert_eq!(read.loadouts[2].armor[19], (89, 1, 0, false));
        assert_eq!(
            read.research,
            vec![("Torch".into(), 100)],
            "the tail past the loadouts still reads"
        );
    }

    // --- W4-f44 phase 2: the equip/dye tails + the Journey powers -----------

    /// The three equipment tails + the hide flags round-trip: armor[20] /
    /// dye[10] `(type, prefix, favorited)` (`:54812-54823` / `:55306-55340`),
    /// the interleaved misc pairs (`:54831-54837` / `:55420-55447`), the two
    /// hide BitsBytes (`:54758-54769` / `:55171-55183`), hideMisc (`:54770`)
    /// and hideInfo[13] (`:54890-54893` / `:55616-55623`).
    #[test]
    fn the_equip_and_hide_tails_round_trip() {
        let mut file = sample();
        file.armor[0] = (89, 81, true); // Gold Helmet, warding, starred
        file.armor[19] = (2326, 0, false); // the last slot rides too
        file.dye[0] = (1007, 0, true);
        file.dye[9] = (3041, 0, false);
        file.misc[0] = (3060, 0, 1121, 0); // pet + its dye
        file.misc[4] = (3224, 0, 0, 0); // the hook slot, no dye
        file.hide_visible_accessory =
            [true, false, true, false, false, false, false, true, true, false];
        file.hide_misc = 0b11; // pet + light-pet hidden (`:17344`/`:17353`)
        file.hide_info[0] = true;
        file.hide_info[12] = true;
        let read = read_player_bytes(&cbc_decrypt(&write_player_bytes(&file))).unwrap();
        assert_eq!(read.armor[0], (89, 81, true));
        assert_eq!(read.armor[19], (2326, 0, false));
        assert_eq!(read.armor[1], (0, 0, false));
        assert_eq!(read.dye[0], (1007, 0, true));
        assert_eq!(read.dye[9], (3041, 0, false));
        assert_eq!(read.misc[0], (3060, 0, 1121, 0));
        assert_eq!(read.misc[4], (3224, 0, 0, 0));
        assert_eq!(read.misc[1], (0, 0, 0, 0));
        assert_eq!(read.hide_visible_accessory, file.hide_visible_accessory);
        assert_eq!(read.hide_misc, 0b11);
        assert!(read.hide_info[0] && read.hide_info[12] && !read.hide_info[1]);
        // The card state + the phase-1 tail are unaffected.
        assert_eq!(read.inventory, file.inventory);
        assert_eq!(read.loadouts, Default::default());
    }

    /// The write side is byte-faithful: non-default equip/hide values move
    /// ONLY their documented byte spans, and the fresh-default write still
    /// reproduces the C# golden fixture byte-for-byte.
    #[test]
    fn the_equip_tail_write_is_byte_faithful() {
        let fresh = serialize_plaintext(&sample());
        let mut file = sample();
        file.hide_visible_accessory[9] = true; // the second BitsByte, bit 1
        let with_hide = serialize_plaintext(&file);
        // Offset walk (golden-layout witness): 24 prologue + 7 name + 1 diff
        // + 8 playtime + 4 hair + 1 hairDye + 1 team = 46, hide1 at 46,
        // hide2 at 47.
        let diffs: Vec<usize> = fresh
            .iter()
            .zip(with_hide.iter())
            .enumerate()
            .filter(|(_, (a, b))| a != b)
            .map(|(i, _)| i)
            .collect();
        assert_eq!(diffs, vec![47], "exactly hideVisibleAccessory[8..10]'s byte");
        assert_eq!(with_hide[47], 0b10);
        // hideMisc at 48.
        file = sample();
        file.hide_misc = 3;
        let with_misc = serialize_plaintext(&file);
        let diffs: Vec<usize> = fresh
            .iter()
            .zip(with_misc.iter())
            .enumerate()
            .filter(|(_, (a, b))| a != b)
            .map(|(i, _)| i)
            .collect();
        assert_eq!(diffs, vec![48]);
        // The golden parity is untouched (the defaults emit the old bytes).
        let golden = serialize_plaintext(&PlayerFileWrite {
            name: "Golden".into(),
            hair: 1,
            hair_color: [215, 90, 55],
            skin_color: [255, 125, 90],
            eye_color: [105, 90, 75],
            shirt_color: [175, 165, 140],
            undershirt_color: [160, 180, 215],
            pants_color: [255, 230, 175],
            shoe_color: [160, 105, 60],
            inventory: {
                let mut inv = [(0, 0, 0); 58];
                inv[0] = (3507, 1, 0);
                inv[1] = (3509, 1, 0);
                inv[2] = (3506, 1, 0);
                inv[3] = (2, 50, 0);
                inv
            },
            inv_favorites: 1 << 3,
            last_saved_binary: 0x48DF_4506_E391_C000,
            ..Default::default()
        });
        let fixture = cbc_decrypt(include_bytes!("../../../assets/test/golden.plr"));
        assert_eq!(golden.as_slice(), &fixture[..3827]);
    }

    /// Out-of-range equip ids air WITHOUT desyncing: the C# `netDefaults`
    /// airs the id, `Prefix` no-ops on air (`Item.cs:515-518`) but favorited
    /// is set unconditionally (`:55315`); the misc `>= ItemID.Count` arm
    /// drops the prefix byte (`:55425-55439`). The legacy -1..-48 ids ride
    /// verbatim (`Item.netDefaults`, `Item.cs:1213-1360`).
    #[test]
    fn over_count_equip_items_air_without_desyncing() {
        let mut file = sample();
        file.armor[0] = (ITEM_ID_COUNT, 81, true); // id+prefix air, fav rides
        file.armor[1] = (-48, 0, false); // the legacy band rides verbatim
        file.armor[2] = (-49, 5, false); // below the band → air
        file.dye[0] = (ITEM_ID_COUNT + 7, 0, false);
        file.misc[0] = (ITEM_ID_COUNT, 9, 1121, 0); // equip airs, dye rides
        file.armor[19] = (89, 0, false); // a real item AFTER the aired ones
        file.research = vec![("Torch".into(), 100)];
        let read = read_player_bytes(&cbc_decrypt(&write_player_bytes(&file))).unwrap();
        assert_eq!(read.armor[0], (0, 0, true), "air id+prefix, favorited rides");
        assert_eq!(read.armor[1], (-48, 0, false));
        assert_eq!(read.armor[2], (0, 0, false));
        assert_eq!(read.dye[0], (0, 0, false));
        assert_eq!(read.misc[0], (0, 0, 1121, 0), "the misc arm drops the aired prefix");
        assert_eq!(read.armor[19], (89, 0, false));
        assert_eq!(read.research, vec![("Torch".into(), 100)], "the walk stayed aligned");
    }

    /// The Journey powers: a Journey character (difficulty 3) keeps the
    /// loaded Godmode/FarPlacement/SpawnRate (`CreativePowerManager.cs:197-210`);
    /// a non-Journey one has them RESET to the ctor defaults after the load
    /// (`:211-214` — `ResetDataForNewPlayer`, `CreativePowers.cs:710-713`/
    /// `:749-752`/`:1722-1725`).
    #[test]
    fn the_creative_powers_load_and_reset_by_difficulty() {
        let mut file = sample();
        file.difficulty = 3; // Journey
        file.powers = CreativePowersSave { godmode: true, far_placement: false, spawn_rate: 0.25 };
        let read = read_player_bytes(&cbc_decrypt(&write_player_bytes(&file))).unwrap();
        assert_eq!(read.powers, file.powers, "Journey keeps the loaded powers");
        file.difficulty = 0; // Classic — the same bytes reset on load
        let read = read_player_bytes(&cbc_decrypt(&write_player_bytes(&file))).unwrap();
        assert_eq!(read.powers, CreativePowersSave::default());
        assert_eq!(
            read.powers,
            CreativePowersSave { godmode: false, far_placement: true, spawn_rate: 0.5 }
        );
    }

    /// An unknown power id BREAKS the load loop (`CreativePowerManager.cs:203-206`)
    /// — the remaining bytes belong to the next section, so the super-cart
    /// byte + loadouts still read at their own offsets.
    #[test]
    fn an_unknown_power_id_breaks_the_loop_like_the_csharp() {
        // Hand-splice the powers span: (true, 99) then STOP — no payload for
        // the unknown id; the next byte is the super-cart BitsByte (2).
        let plain = serialize_plaintext(&sample());
        // The powers span starts after the temp-slot byte. Offsets from the
        // golden-layout witness: buffs at 2402, the inc3 tail 1425 bytes;
        // powers sit 370+4+16+48+4+1+8+4+5+1 = 461 into it → 2863.
        let powers_at = 2402 + 461;
        let mut w = Writer::new();
        w.bytes(&plain[..powers_at]);
        w.boolean(true);
        w.u16(99); // an id the manager never registered → break
        w.bytes(&plain[powers_at + 16..]); // skip the old 16-byte span, keep superCart onward
        let read = read_player_bytes(&w.into_inner()).expect("the break keeps the walk aligned");
        assert_eq!(read.powers, CreativePowersSave::default(), "nothing loaded, the reset stands");
        assert_eq!(read.current_loadout, 0);
        assert_eq!(read.loadouts, Default::default());
        assert_eq!(read.inventory, sample().inventory);
    }

    /// The legacy 279 subset shim captures the armour/dye pairs + the hide
    /// BitsBytes it carries (W4-f44 ph2), with the fresh defaults for the
    /// sections the subset never wrote.
    #[test]
    fn the_legacy_subset_keeps_its_equip_and_hide_state() {
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
        w.u8(0);
        w.bytes(&[0b101, 0b10]); // hide bits 0/2 + bit 9
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
        // armour[0] = (89, 81), the rest air; dye[3] = (1007, 0).
        w.i32(89);
        w.u8(81);
        for _ in 0..19 {
            w.i32(0);
            w.u8(0);
        }
        for i in 0..10 {
            w.i32(if i == 3 { 1007 } else { 0 });
            w.u8(0);
        }
        for (i, (id, stack, prefix)) in s.inventory.iter().enumerate() {
            w.i32(*id);
            w.i32(*stack);
            w.u8(*prefix);
            w.boolean(s.inv_favorites & (1 << i) != 0);
        }
        let file = read_player_bytes(&w.into_inner()).expect("the legacy subset reads");
        assert_eq!(file.armor[0], (89, 81, false), "no favourited bool at 279");
        assert_eq!(file.dye[3], (1007, 0, false));
        assert!(file.hide_visible_accessory[0] && file.hide_visible_accessory[2]);
        assert!(file.hide_visible_accessory[9] && !file.hide_visible_accessory[8]);
        assert_eq!(file.hide_misc, 0, "the subset never wrote hideMisc");
        assert_eq!(file.hide_info, [false; 13]);
        assert_eq!(file.powers, CreativePowersSave::default());
        assert_eq!(file.inventory, s.inventory);
    }
}
