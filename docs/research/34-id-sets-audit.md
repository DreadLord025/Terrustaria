# 34 — ID sets completeness (Terraria.ID.*)

> Wave 3 · researched 2026-11-12 · sources: decompiled (`decompiled/Terraria/ID/*`), crates
> (`crates/tr-app`, `crates/tr-core`), ROADMAP/ANALYSIS/PORT_MAP, research/05-npc-core.md

## Scope

This module is a **mechanical census**, not a subsystem: it enumerates every id set the
C# declares under `decompiled/Terraria/ID/` (60 files), records the authoritative size
constant of each, and compares that size against the Rust port's id tables. The sets
carry the whole game's addressing: `ItemID`/`NPCID`/`ProjectileID`/`TileID`/`WallID`
are array indices in the C# (`Main.item[]`, `Main.npc[]`, `Main.projectile[]`,
`Main.tile[]`, `Main.wall[]`), so a missing row is not a cosmetic gap — it is an
off-by-N in every table that indexes by id. The audit therefore also audits the
**id-space translations** the port introduced (`NpcKind::to_u16`/`from_u16`, `net_id`,
`cs_id`), because a port-local protocol space that collides with itself loses kinds on
every save/load and net-message round-trip.

Files read: `decompiled/Terraria/ID/*.cs` (60 files, `Count` constants harvested by
grep), `crates/tr-app/src/{itemdata,npc,projectile,prefix,buffs,bestiary,audio,sound,
registry,variants,mount}.rs`, `crates/tr-core/src/{tiledata/mod.rs,tiledata/sets.rs,
armor_sets.rs,bestiary.rs}`, `crates/tr-app/src/app/{mod,netglue,saveglue}.rs`,
`crates/tr-app/src/world_sim/npcs.rs`. No cargo run, no graphify write.## Key entities

### C# — the `Count` census (declaration line → assignment line → value)

- `decompiled/Terraria/ID/ItemID.cs:14075` `public static readonly short Count;` → `:14861` `ItemID.Count = 6196;` — the item id space (0..6195); `ItemID.Sets` is sized by it (`ItemID.cs:329` `Sets.Factory = new SetFactory(ItemID.Count)`).
- `decompiled/Terraria/ID/NPCID.cs:12071` → `:12581` `NPCID.Count = 697;` — the NPC id space; **plus** `NPCID.cs:10545` `public const short NegativeIDCount = -66;` → the negative half of the space (-66..-1), so the C# NPC space is **763 slots**, not 697.
- `decompiled/Terraria/ID/ProjectileID.cs:2826` → `:2832` `ProjectileID.Count = 1136;`
- `decompiled/Terraria/ID/TileID.cs:2137` → `:2143` `TileID.Count = 754;`
- `decompiled/Terraria/ID/WallID.cs:839` → `:845` `WallID.Count = 367;`
- `decompiled/Terraria/ID/BuffID.cs:991` → `:997` `BuffID.Count = 401;`
- `decompiled/Terraria/ID/DustID.cs:373` → `:423` `DustID.Count = 332;`
- `decompiled/Terraria/ID/PrefixID.cs:212` → `:216` `PrefixID.Count = 98;` — the consts run 1..97 (`PrefixID.cs:210` `public const int Scraggling = 97;`).
- `decompiled/Terraria/ID/PlayerItemSlotID.cs:166` → `:192` `PlayerItemSlotID.Count = PlayerItemSlotID._nextSlotId;` — **dynamic**: the loadout/dye/bank slot registry grows at runtime, there is no fixed size.
- `decompiled/Terraria/ID/MusicID.cs:228` `public const int Count = 105;` — the only `const`-form `Count` in the directory.
- `decompiled/Terraria/ID/SoundID.cs` — **no `Count`**. The two size counters are `SoundID.cs:1383` `SoundID.NPCDeathCount = 69;` and `SoundID.cs:1595` `SoundID.ItemSoundCount = 211;`; the id space is the set of registered `SoundStyle`s, not a dense array.
- `decompiled/Terraria/GameContent/UI/ItemRarity.cs:7` — **not an id set**: a `Dictionary<int, Color>` with 12 entries (`:13`-`:25` — `-11` amber, `-1` trash, `1..9` blue→cyan). There is no `ItemRarityID` class in 1.4.5.8 and no `Count`.
- `decompiled/Terraria/ID/ArmorIDs.cs` — 15 per-slot sub-spaces, each with its own `Count`: Head `:633`→`:637` **293**, Body `:1129`→`:1133` **262**, Legs `:1622`→`:1626` **254**, HandOn `:1720` **33**, HandOff `:1783` **19**, Back `:1894` **42**, Front `:1963` **18**, Shoe `:2051` **31**, Waist `:2113` **18**, Wing `:2240` **52**, Shield `:2274` **13**, Neck `:2325` **15**, Face `:2415` **24**, Balloon `:2479` **20**, Beard `:2527` **5**.
- Remaining sets with a `Count` (declaration → assignment → value): `ChainID.cs:39`→`:43` **17**, `CloudID.cs:115`→`:119` **42**, `ExtrasID.cs:637`→`:641` **316**, `GlowMaskID.cs:777`→`:781` **384**, `GoreID.cs:1611`→`:1615` **1407**, `ImmunityCooldownID.cs:70`→`:81` **6**, `InvasionID.cs:23`→`:27` **5**, `LiquidID.cs:13`→`:17` **4**, `MessageID.cs:340`→`:344` **163**, `MountID.cs:185`→`:189` **66**, `NPCHeadID.cs:194`→`:198` **81**, `PlayerTeamID.cs:17`→`:21` **6**, `PlayerTextureID.cs:37`→`:41` **16**, `PlayerVariantID.cs:51`→`:55` **12**, `ProjectileSourceID.cs:45`→`:49` **23**, `TorchID.cs:133`→`:185` **24**, `WaterStyleID.cs:31`→`:35` **13**, `GameModeID.cs:13` **4**, `PlayerVoiceID.cs:30` **4**, `SurfaceBackgroundID.cs:55` **16**.
- **Not found**: no `NPCPaint*` id set and no `Recipe*` id set exist in 1.4.5.8's `Terraria.ID` — the paint ids live in `ID/PaintID.cs` / `ID/PaintCoatingID.cs` (no `Count` constant in either), and recipes are keyed by item id. "NPCPaintID" does not exist.
### Rust — the port's id tables

- `crates/tr-app/src/itemdata.rs:120` `pub static ITEMS: [ItemDef; 6196]` — the generated item table (6196 `ItemDef { … }` literals, `id: 0..6195`), the port's only complete id table.
- `crates/tr-core/src/tiledata/mod.rs:11` `pub const TILE_COUNT: usize = 754;` / `:14` `pub const WALL_COUNT: usize = 367;` — tile/wall id-space bounds; `crates/tr-core/src/tiledata/sets.rs` carries ~45 generated `TileSet` bitsets over the same space.
- `crates/tr-app/src/buffs.rs:23` `pub const BUFF_ID_COUNT: usize = 401;` + `:21` `MAX_BUFFS: usize = 44` (`Player.maxBuffs`, `Player.cs:57106`).
- `crates/tr-app/src/audio.rs:18` `pub const MAX_MUSIC: usize = 105;`.
- `crates/tr-app/src/npc.rs:31` `pub enum NpcKind` — 190 variants = **182 kinds** + 8 slime-colour variants (`Blue`/`Green`/`Pinky`/`Baby`/`Black`/`Purple`/`Red`/`Yellow`, `:694`-`:701`).
- `crates/tr-app/src/npc.rs:850` `to_u16` / `:1105` `from_u16` — the **port-local protocol id space** (0..181).
- `crates/tr-app/src/npc.rs:1512` `net_id` → `crates/tr-app/src/bestiary.rs:1482` `entry_for_npc(...).net_id` — the **vanilla** `NPC.type`.
- `crates/tr-app/src/npc.rs:1789` `cs_id` — a **partial** vanilla-id map (43 of 182 kinds; the rest return `u16::MAX`).
- `crates/tr-app/src/projectile.rs:1046` `proj_data(kind: u16) -> Option<ProjData>` — the per-projectile defaults; 213 `pub const …: u16` id constants in the file (max `KRAKEN_WAVE = 1135`).
- `crates/tr-app/src/prefix.rs:198` `stat_switch(prefix: u16)` — the prefix stat multipliers; 78 arms (ids 1..61, 81..97).
- `crates/tr-app/src/bestiary.rs` — 174 `BestiaryEntryDef` literals (the net-id → sheet/entry table); `crates/tr-app/src/registry.rs:88` `item(id: u16)` / `:95` `get(id: u16)` — the item lookup over `ITEMS`.

## Dependencies

- Calls into: `NpcKind::to_u16`/`from_u16` are consumed by the **netcode** (`crates/tr-app/src/app/netglue.rs:385`, `:1044`, `:1048`, `:1504`, `:2060`) and by the **app save** (`app/saveglue.rs:107` writes `kind: n.kind.to_u16()`; `app/mod.rs:1980` reads `NpcKind::from_u16(rec.kind)`); `world_sim/npcs.rs:1520`, `:1911` decode a kind from an id.
- Called by: every subsystem that indexes by id — `registry::item` (items), `tiledata::TILE_COUNT`/`WALL_COUNT` (framing, worldgen, lighting), `proj_data` (projectiles), `stat_switch` (prefixes), `net_id` (loot db, save/load, bestiary UI).
- Shared data structures: the **item id space** is shared verbatim between the C# and the Rust (`u16` id, 0..6195 — no translation layer); the **NPC id space is not shared** — the Rust has three parallel spaces (protocol `u16`, vanilla `net_id`, partial `cs_id`), and the world `.wld` (`crates/tr-core/src/wld/sections/npcs.rs`, `SAVES_AND_LOADS`) keys on `net_id` while the app's own save keys on the protocol `u16`.

## Port status

### Centrepiece — coverage table

| Set | C# Count (file:line) | Rust rows | Delta | Ported ids (reachable) | Verdict / severity |
| --- | --- | --- | --- | --- | --- |
| ItemID | 6196 (`ItemID.cs:14861`) | 6196 (`itemdata.rs:120`) | **0** | 0..6195, every id | **PASS** — verified independently (6196 `ItemDef` literals, ids contiguous) |
| TileID | 754 (`TileID.cs:2143`) | 754 (`tiledata/mod.rs:11`) | **0** (space) | 0..753 bounds; ~45 `TileSet` bitsets generated | **PASS** for the space, **PARTIAL** for the `TileID.Sets.*` family |
| WallID | 367 (`WallID.cs:845`) | 367 (`tiledata/mod.rs:14`) | **0** (space) | 0..366 bounds | **PASS** for the space; wall sets partial |
| BuffID | 401 (`BuffID.cs:997`) | 401 (`buffs.rs:23`) | **0** (space) | id space only — no per-buff effect rows | **PASS** for the space, **MISSING** for the effects (`buffs.rs:17-19` says so) |
| MusicID | 105 (`MusicID.cs:228`) | 105 (`audio.rs:18`) | **0** | 0..104 | **PASS** |
| PrefixID | 98 (`PrefixID.cs:216`; consts 1..97) | 78 arms (`prefix.rs:199`-`:277`) | **19** (ids 62..80) | 1..61, 81..97 | **PARTIAL** — behavioural, ids 62..80 (Hard…Violent, the melee-prefix family) fall into the `_ =>` identity arm and get **no** vanilla bonuses |
| ProjectileID | 1136 (`ProjectileID.cs:2832`) | 213 id constants (`projectile.rs`), 174 `proj_data` arms | ~**960** ids without a Rust constant | a hand-picked subset (max id 1135) | **PARTIAL** — behavioural; `proj_data` returns `None` for every unlisted id |
| NPCID | 697 + 66 negative (`NPCID.cs:12581`, `:10545`) | 182 kinds (`npc.rs:31`-`:701`) | **515+66** unported kinds | 182 kinds, port-local protocol ids 0..181 | **PARTIAL** — structural (see deviations 4-6) |
| DustID | 332 (`DustID.cs:423`) | 0 | **332** | two hard-coded literals only (`rain.rs:96` `DustID.Snowflake` = 76; `tr-core/src/waterfall/mod.rs:54` `DustID.Sparkle`) | **MISSING** — no dust id table (module 15) |
| SoundID | no `Count`; `NPCDeathCount = 69` (`SoundID.cs:1383`), `ItemSoundCount = 211` (`:1595`) | 0 | n/a | — | **MISSING** — `sound.rs` has only attenuation/pan helpers; no sound-id registry (module 32) |
| ItemRarity (not an id set) | 12-entry `Dictionary<int,Color>` (`GameContent/UI/ItemRarity.cs:13-25`) | 0 | 12 colours | — | **MISSING** — cosmetic/UI; `ItemDef.rare` carries the number, not the colour |
| PlayerItemSlotID | dynamic (`PlayerItemSlotID.cs:192`) | fixed 20 equip slots (`player.rs:432`) | n/a | 20 | **MISSING** — the loadout/dye/bank slot registry is not ported; cosmetic |
| ArmorIDs (15 slots) | Head 293 / Body 262 / Legs 254 / HandOn 33 / HandOff 19 / Back 42 / Front 18 / Shoe 31 / Waist 18 / Wing 52 / Shield 13 / Neck 15 / Face 24 / Balloon 20 / Beard 5 (`ArmorIDs.cs:637`-`:2527`) | 0 (only the *set-bonus* tables in `tr-core/src/armor_sets.rs`: `METAL_TIER1/2`, `PLATINUM`, `WOOD`, `SET_STATS`) | **15 slot spaces** | — | **MISSING** — the per-slot id spaces and their `Sets` are absent |
| ChainID | 17 (`ChainID.cs:43`) | 0 | 17 | — | **MISSING** |
| CloudID | 42 (`CloudID.cs:119`) | 0 | 42 | — | **MISSING** (no cloud id table found) |
| ExtrasID | 316 (`ExtrasID.cs:641`) | 0 | 316 | — | **MISSING** |
| GlowMaskID | 384 (`GlowMaskID.cs:781`) | 4 named loads (`render/entities.rs:528-531` `Glow_132/143/149/162`) | 380 | — | **PARTIAL** — by-name lookups, no id table |
| GoreID | 1407 (`GoreID.cs:1615`) | leaf-gore behaviour only (`render/foliage.rs:5`) | ~1400 | — | **PARTIAL** — no gore id table |
| ImmunityCooldownID | 6 (`ImmunityCooldownID.cs:81`) | 0 | 6 | — | **MISSING** |
| InvasionID | 5 (`InvasionID.cs:27`) | 0 | 5 | — | **MISSING** (no id table; invasion state is separate) |
| LiquidID | 4 (`LiquidID.cs:17`) | liquid kinds exist in `tr-core/src/liquid.rs` (count **not verified** here) | ? | — | **PARTIAL** — count not re-verified in this pass |
| MessageID | 163 (`MessageID.cs:344`) | 0 | 163 | — | **MISSING** |
| MountID | 66 (`MountID.cs:189`) | `tr-app/src/mount.rs` (count **not verified** here) | ? | — | **PARTIAL** — count not re-verified |
| NPCHeadID | 81 (`NPCHeadID.cs:198`) | 0 | 81 | — | **MISSING** |
| PlayerTeamID | 6 (`PlayerTeamID.cs:21`) | 0 | 6 | — | **MISSING** |
| PlayerTextureID | 16 (`PlayerTextureID.cs:41`) | 0 | 16 | — | **MISSING** |
| PlayerVariantID | 12 (`PlayerVariantID.cs:55`) | `tr-app/src/variants.rs` (count not verified) | ? | — | **PARTIAL** |
| ProjectileSourceID | 23 (`ProjectileSourceID.cs:49`) | 0 | 23 | — | **MISSING** |
| TorchID | 24 (`TorchID.cs:185`) | `tiledata/sets.rs` `SET_TORCHES` (24 ids, ROADMAP M7-SMARTCURSOR-1) | 0 | 24 | **PASS** for the set |
| WaterStyleID | 13 (`WaterStyleID.cs:35`) | water-style state exists (not verified here) | ? | — | **PARTIAL** |
| GameModeID | 4 (`GameModeID.cs:13`) | 0 (no difficulty flags at all — module 05) | 4 | — | **MISSING** |
| PlayerVoiceID | 4 (`PlayerVoiceID.cs:30`) | 0 | 4 | — | **MISSING** (out of scope, voiced) |
| SurfaceBackgroundID | 16 (`SurfaceBackgroundID.cs:55`) | 0 | 16 | — | **MISSING** |
| PaintID / PaintCoatingID | no `Count` in either file | 0 | n/a | — | **MISSING** — the paint/coating id spaces have no Rust table |

**Sets with no Rust counterpart at all** (no table, no constant, no symbol found by a
crate-wide grep for `pub static …(BUFF|DUST|GORE|GLOW|SOUND|MUSIC|PREFIX|RARITY|SLOT)`):
`DustID`, `SoundID`, `ItemRarity`, `PlayerItemSlotID`, `ChainID`, `CloudID`, `ExtrasID`,
`ImmunityCooldownID`, `InvasionID`, `MessageID`, `NPCHeadID`, `PlayerTeamID`,
`PlayerTextureID`, `ProjectileSourceID`, `GameModeID`, `PlayerVoiceID`,
`SurfaceBackgroundID`, `PaintID`, `PaintCoatingID`, and the 15 `ArmorIDs` sub-spaces.

### Subsystem rows

| C# subsystem | Status | Rust location | Evidence / notes |
| --- | --- | --- | --- |
| `ItemID` (6196) | PORTED | `crates/tr-app/src/itemdata.rs:120` | `ITEMS: [ItemDef; 6196]`, 6196 literals, ids 0..6195 contiguous — exact match with `ItemID.cs:14861`. Generated by `tools/GenItemData`; the header forbids hand-edits. |
| `TileID` / `WallID` (754 / 367) | PORTED (bounds) / PARTIAL (sets) | `crates/tr-core/src/tiledata/mod.rs:11`,`:14`; `tiledata/sets.rs` | `TILE_COUNT`/`WALL_COUNT` match exactly. `TileSet::has` (`mod.rs:29`) bounds-checks against `TILE_COUNT`, so an out-of-range tile id is silently false — a safe failure mode. The `TileID.Sets.*` family is only ~45 sets; the rest of `TileID.cs`'s 100+ sets are absent. |
| `BuffID` (401) | PARTIAL | `crates/tr-app/src/buffs.rs:23` | `BUFF_ID_COUNT = 401` matches; the per-buff *effect* chain (`Player.cs:9875-12000`) is absent (`buffs.rs:17-19`). The five `Main` tables (`debuff` 76, `lightPet` 12, `vanityPet` 74, `meleeBuff` 8, `pvpBuff` 20) are transcribed verbatim (`buffs.rs:28`/`:36`/`:39`/`:47`/`:52`). |
| `MusicID` (105) | PORTED | `crates/tr-app/src/audio.rs:18` | `MAX_MUSIC: usize = 105` == `MusicID.cs:228`. |
| `PrefixID` (98) | PARTIAL | `crates/tr-app/src/prefix.rs:198` | `stat_switch` has 78 arms: ids 1..61 and 81..97. **Ids 62..80 are absent** and fall to the `_ =>` identity arm (`:278`). The C# `PrefixID` consts 62..80 are the melee-only family (`Hard` 62 … `Violent` 80) and carry their own damage/knockback values in `Item.Prefix`. |
| `ProjectileID` (1136) | PARTIAL | `crates/tr-app/src/projectile.rs:1046` (`proj_data`) | 213 `pub const …: u16` id constants (max `KRAKEN_WAVE = 1135`); `proj_data`'s match has 174 const-name arms. Unlisted ids return `None` → the projectile spawns with vanilla defaults. The id *constants* are the vanilla `ProjectileID` values (verified: `WOODEN_ARROW = 1`, `SHURIKEN = 3`, `HOOK = 13` — matching `ProjectileID.cs`), so the space is shared, just sparsely populated. |
| `NPCID` (697 + 66) | PARTIAL | `crates/tr-app/src/npc.rs:31` | 182 kinds vs 763 vanilla slots. The roster is a **kind list**, not an id table: `NpcKind` carries the vanilla behaviour per kind but not the vanilla id, so id-indexed vanilla tables (`NPCID.Sets.*`, `Main.npcFrameCount[696]`) have no Rust carrier. |
| `DustID` (332) | MISSING | — | No table. Two literals only (`rain.rs:96`, `tr-core/src/waterfall/mod.rs:54`). |
| `SoundID` | MISSING | — | `crates/tr-app/src/sound.rs` holds only `SOUND_ATTENUATION_DISTANCE` (`:10`), `does_sound_scale_with_ambient_volume` (`:20`), `play_sound` (`:63`), `pan_gains` (`:134`). No sound-id registry; sound names are stringly-typed. |
| `ItemRarity` | MISSING | — | `ItemRarity.cs` is a colour dictionary, not an id set. The Rust carries `ItemDef.rare` (`itemdata.rs`, `rare: i32`) but no rarity→colour map. |
| `PlayerItemSlotID` | MISSING | `crates/tr-app/src/player.rs:432` (`equipped_prefix: [u16; 20]`) | The C# set is a runtime-growing slot registry (`_nextSlotId`); the port hard-codes 20 equip slots and has no loadout/dye/bank slot space. |
| `ArmorIDs` (15 slots) | MISSING | `crates/tr-core/src/armor_sets.rs` | Only the *set-bonus* tables are ported (`METAL_TIER1` `:8`, `METAL_TIER2` `:17`, `PLATINUM` `:25`, `WOOD` `:27`, `SET_STATS` `:149`). The per-slot id spaces (Head 293, Body 262, …) are absent. |
| `TorchID` (24) | PORTED | `crates/tr-core/src/tiledata/sets.rs` `SET_TORCHES` | 24 ids, matching the ROADMAP's M7-SMARTCURSOR-1 citation of `ItemID.cs:1371`. |
| `NPCPaint*` / `Recipe*` id sets | N/A | — | **Do not exist** in 1.4.5.8's `Terraria.ID`. Paint/coating ids live in `ID/PaintID.cs` / `ID/PaintCoatingID.cs` (no `Count`); recipes key by item id. |
| `GlowMaskID` (384) / `GoreID` (1407) | PARTIAL | `render/entities.rs:528-531`, `render/foliage.rs:5` | By-name sheet loads only; no id table. |

## Tests

No test in the tree asserts an id-set *size*. The id-space invariants are only covered
indirectly, and the coverage is uneven:

- `crates/tr-app/src/world_sim/npcs.rs:7386`-`:7387`, `:7548`-`:7549`, `:7667`-`:7668`,
  `:7861`-`:7862`, `:8042`-`:8043`, `:8230` — the per-family round-trip assertions
  `assert_eq!(kind.to_u16(), id)` / `assert_eq!(NpcKind::from_u16(id), Some(kind))`.
  These cover the tower/worm/Moon-Lord families only — **not** the 78..84 collision band.
- `crates/tr-app/src/npc.rs:13197` `slime_variants_match_the_setdefaults_lines` — the
  slime-variant roster, the one place the 8 non-kind variants are exercised.
- `crates/tr-app/src/npc.rs:12730`-`:12853` — the bestiary `net_id` spot-checks
  (`assert_eq!(e.net_id, 6/42/23/21/44/49/51/60/66)`).
- `crates/tr-app/src/app/saveglue.rs:1023` `dw_2_5_a_town_npc_round_trips_through_the_app`
  — exercises the `to_u16` save path, but only for a town NPC (no collision-band kind).
- `crates/tr-app/src/app/netglue.rs` — the netcode kind codec, no exhaustive id test.
- **No test** asserts `ITEMS.len() == ItemID.Count`, `TILE_COUNT`, `WALL_COUNT`,
  `BUFF_ID_COUNT`, `MAX_MUSIC`, or the prefix table's coverage.
## Deviations & risks

1. **The `NpcKind` protocol id space collides with itself on 7 ids (78..84).**
   **RESOLVED 2026-11-12 by S1-F-1 — see Task seeds.**
   `to_u16` (`npc.rs:850`) maps **two** kinds onto each of these ids:
   `78 ← {PirateDeckhand, Deerclops}`, `79 ← {PirateCaptainsMate, SkeletronHand}`,
   `80 ← {PirateArcher, WallOfFleshEye}`, `81 ← {PirateCorsair, PrimeCannon}`,
   `82 ← {PirateCaptain, PrimeSaw}`, `83 ← {MartianProbe, PrimeVice}`,
   `84 ← {MartianDrone, PrimeLaser}` (the pirate band at `:887`-`:893` vs the
   Deerclops/Skeletron-Prime band at `:1044`-`:1050`).
   `from_u16` (`:1105`) resolves each id to its **first** arm (the pirate/probe
   family, `:1138`-`:1144`), so an exhaustive `to_u16 → from_u16` sweep yields
   **7 round-trip failures**: `Deerclops→PirateDeckhand`,
   `SkeletronHand→PirateCaptainsMate`, `WallOfFleshEye→PirateArcher`,
   `PrimeCannon→PirateCorsair`, `PrimeSaw→PirateCaptain`, `PrimeVice→MartianProbe`,
   `PrimeLaser→MartianDrone`.
   This is **not cosmetic**: `to_u16`/`from_u16` are on the live **save** path
   (`app/saveglue.rs:107` writes, `app/mod.rs:1980` reads) and the live **netcode**
   path (`app/netglue.rs:385`, `:1044`, `:1504`, `:2060`). A saved Deerclops comes
   back as a Pirate Deckhand; a remote Deerclops arrives as a Pirate Deckhand.
   Severity: **behavioural / determinism, high**. Not admitted by ROADMAP or ANALYSIS
   (see deviation 3).

2. **`to_u16` still carries 31 conflicting arms; `from_u16` 7.**
   **RESOLVED 2026-11-12 by S1-F-1/S1-F-2 — see Task seeds.** `to_u16` has 29 dead
   duplicate arms (`SolarCrawltipedeHead` ×4 at `:953`/`:958`/`:963`/`:968`,
   `SolarSpearman` ×4, `StardustWormHead` ×4, `StardustCellSmall` ×2 at `:942`/`:991`,
   `VortexHunter`/`VortexAlien`/`VortexHeadcrab` ×2 at `:987`-`:989`/`:1051`-`:1053`, …)
   plus 2 arms whose value contradicts the surviving one (`DD2LanePortal ⇒ 549` at
   `:990` vs `⇒ 147` at `:204`; `StardustSpiderBig ⇒ 409` at `:1062` vs `⇒ 133` at
   `:946`). `from_u16` has 21 dead duplicate literal arms (`:1207`-`:1226`) and 7
   contradictory ones (`:1285`-`:1291`). Rust resolves first-arm-wins, so behaviour is
   currently the "early" value — but the code is unreachable and the file's own comment
   ("the unreachable dead copies are gone", `npc.rs:881-886`) is **false**. Severity:
   **cosmetic** today, a trap for the next id merge; the *id-collision* it hides
   (deviation 1) is behavioural.

3. **ROADMAP `M4-EVENT-4b` and ANALYSIS claim the dead arms were removed — they were
   not.**
   **RESOLVED 2026-11-12 by S1-F-1 — the claims are corrected (ROADMAP/ANALYSIS/ATLAS)
   and the dead arms are gone.** `docs/ROADMAP.md:1469` ("done 2026-11-07 … the pirate/martian id collision
   68-72 renumbered to 78-82/83 (plus the render_key + from_u16 tables), **the
   duplicated dead arms removed**") and `docs/ANALYSIS.md:3024-3027` ("**the duplicated
   dead arms removed**") both assert the cleanup. Verified: the renumber landed
   (pirates 78-82, probe 83) but **31 conflicting / 29 dead arms remain in `to_u16`
   and 7 contradictory / 21 dead arms remain in `from_u16`**, and the renumber
   *created* the 78..84 collision with the Deerclops/Skeletron-Prime band.
   Verdict on the DONE row: **FAIL** (the claim is false and the renumber introduced
   a regression). The research file `docs/research/05-npc-core.md:172-183` already
   flagged the dead arms — as "cosmetic" — but did **not** detect the resulting
   round-trip loss, and no ROADMAP/ANALYSIS text admits the collision.

4. **`cs_id` covers 43 of 182 kinds.** `npc.rs:1789` maps Slime…Shark
   (`:1791`-`:1835`) and returns `u16::MAX` for the remaining 139. The doc comment
   (`:1784-1788`) says the map is what "the statue spawns' ids, the `MechSpawn` caps
   (`NPC.cs:7420`) and the teleporting statues' rosters (`Wiring.cs:2424`/`:2452`)"
   read — so for 139 kinds those vanilla-id consumers can never match.
   Severity: **behavioural**, medium; the affected consumers are statue spawns and
   the mech-spawn caps.

5. **11 kinds are unreachable through `from_u16`.**
   **RESOLVED 2026-11-12 by S1-F-1 — all 182 kinds decode (the roster test proves it).** The protocol ids 85-88
   (`BrainScrambler`, `RayGunner`, `ScutlixRider`, `Scutlix`) and 162-168
   (`TaxCollector`, `Golfer`, `SleepingGolfer`, `Zoologist`, `Princess`, `DyeTrader`,
   `PartyGirl`) have **no** `from_u16` arm (`to_u16` assigns them, `from_u16` omits
   them). A net message or save record carrying one of those ids decodes to `None` →
   the NPC is dropped. Severity: **behavioural**, medium.

6. **Three parallel NPC id spaces.** The vanilla `NPC.type` is reachable through
   `net_id` (`npc.rs:1512`, via the bestiary table), the port's protocol id through
   `to_u16`, and a third, partial map through `cs_id`. The `.wld` save keys on
   `net_id` (`crates/tr-core/src/wld/sections/npcs.rs`), the app save on the protocol
   `u16`. Any future consumer that picks the wrong one silently addresses the wrong
   id. Severity: **structural**, high (this is the root cause of deviation 1).

7. **The bestiary table (174 entries) is smaller than the kind roster (182).**
   `net_id` is resolved through `bestiary::entry_for_npc` (`bestiary.rs:1482`), so at
   least 8 kinds have no entry row and their `net_id` resolution path is not covered
   by the port's own tests. Not verified in this pass whether `entry_for_npc` panics,
   falls back, or mis-maps for those kinds. Severity: **behavioural**, medium.

8. **Prefix ids 62..80 are unmodelled.** `stat_switch` (`prefix.rs:199`-`:278`) covers
   1..61 and 81..97; ids 62..80 (`Hard` … `Violent`, `PrefixID.cs:130`-`:200`) hit the
   `_ =>` identity arm. A melee weapon rolled with, say, `Hard` (62) gets the vanilla
   melee damage bonus in C# and **nothing** in Rust. Severity: **behavioural**, medium
   (item stat divergence, visible in tooltips and hit damage). Not admitted anywhere.

9. **`ProjectileID` is sparsely populated.** 213 id constants for 1136 vanilla ids;
   `proj_data` has 174 arms. An unlisted projectile id spawns with no defaults (no
   size, no `aiStyle`, no `timeLeft`, no `penetrate`) — a silent behavioural
   difference for ~950 ids. Severity: **behavioural**, medium-high; the ROADMAP
   tracks the *families* (M4-PROJ-*), not the id table.

10. **`DustID` (332) has no Rust carrier at all.** Only two literals
    (`rain.rs:96`, `tr-core/src/waterfall/mod.rs:54`). Severity: **behavioural**,
    medium (module 15's scope).

11. **The `TileID.Sets.*` family is only partly generated.** ~45 sets in
    `tiledata/sets.rs` against 100+ in `TileID.cs`. The bitset type bounds-checks
    against `TILE_COUNT` (`tiledata/mod.rs:29`), so a missing set degrades to
    "not in the set" rather than a crash — safe, but wrong. Severity: **behavioural**,
    low-medium.

12. **`GameModeID` (4) and the whole difficulty system are absent** (no expert/hardmode
    flags — module 05's deviation). Every `GameModeID`-keyed vanilla table has no
    Rust counterpart. Severity: **behavioural**, high (cross-cutting), tracked by 05.

13. **Cosmetic: `ItemRarity`'s 12 colours and `SurfaceBackgroundID`'s 16 backgrounds
    have no Rust tables.** Severity: **cosmetic / pixel**.

## Task seeds

- **LANDED 2026-11-12 (S1-F-1/F-2, `S1-F-1-npc-kind-map`) — Delete the dead `to_u16`/`from_u16` arms and fix the 78..84 collision.**
  (`npc.rs:953-990`, `:1054-1085`, `:1207-1226`, `:1285-1291`.) Remove the 29/21 dead
  duplicates, reconcile the 2/7 contradictory values, and move the pirate/probe
  family out of the Deerclops/Skeletron-Prime band (or give the Prime family its own
  free ids). Add `#[deny(unreachable_patterns)]` to both fns (the pattern already used
  by `render/entities.rs:1428` after BUG-22). Depends on nothing. Closes deviations 1,
  2, 3.
  *Resolution:* the pirates went to the contiguous 182..188 (the free 0..181 holes are
  non-contiguous); every dead/duplicate/contradictory arm deleted from both tables; the
  deny landed on both fns.
- **LANDED 2026-11-12 (same commit) — Add the exhaustive id-round-trip test.** A table-driven test over all 182
  kinds asserting `from_u16(to_u16(k)) == Some(k)` and that the protocol id space is
  injective (no two kinds share an id). This is the test that would have caught
  deviation 1. Add it next to the existing round-trip assertions in
  `world_sim/npcs.rs:7386`-`:8230`. Depends on the previous task.
  *Resolution:* `npc::tests::the_protocol_id_space_round_trips_every_kind` (it also
  asserts `from_u16` decodes exactly 182 ids and that no id exceeds the `23` body's
  i16 `kind` field), plus the `netglue` wire-seam tests
  (`npc_kind_wire_tests::*`).
- **LANDED 2026-11-12 (same commit) — Correct the ROADMAP/ANALYSIS claims.** `docs/ROADMAP.md:1469` and
  `docs/ANALYSIS.md:3024-3027` assert "the duplicated dead arms removed"; amend both
  (and the `M4-EVENT-4b` status) to the verified state. Depends on the fix task.
  *Resolution:* ROADMAP rows S1-F-1/S1-F-2 marked landed, the ANALYSIS probe section
  and the `ATLAS.md` M4-EVENT-4b row both carry the correction block, and
  `90-consolidated-findings`' U-01/R-34-3 are closed.
- **LANDED 2026-11-12 (same commit) — Close the `from_u16` gap for ids 85-88 and 162-168** (11 kinds currently
  undecodable, deviation 5) and add the missing `from_u16` arms. Depends on the
  dead-arm cleanup.
  *Resolution:* 85..88 decode (`BrainScrambler`/`RayGunner`/`ScutlixRider`/`Scutlix`);
  the roster test proves all 182 kinds are decodable (162..168's arms were live at
  the time of the fix; the audit's 11-kind count dropped to 4 in-tree).
- **M — Extend `cs_id` to the full roster** (43 → 182 kinds) so the statue spawns
  (`Wiring.cs:2424`/`:2452`) and the `MechSpawn` caps (`NPC.cs:7420`) can match every
  kind. Depends on the id-space decision (should `cs_id` be replaced by a single
  `vanilla_id` table that also feeds `net_id`?). Coordinate with module 05/29.
- **M — Unify the three NPC id spaces.** Replace the protocol `u16` with the vanilla
  `NPC.type` (or make the protocol space a generated table derived from one source of
  truth) so the save/net/`.wld` paths share one mapping. This is the structural fix
  for deviations 1, 4, 5, 6. Depends on the S tasks.
- **M — Port the missing prefix ids 62..80** into `stat_switch` (`prefix.rs`), with
  the C#'s melee-branch values from `Item.Prefix` (`Item.cs`'s prefix switch), and
  remove the identity `_ =>` fallback in favour of an explicit "unknown prefix"
  branch. Cross-check with module 07.
- **M — Generate the `ProjectileID` id-constant table** (all 1136 ids, or at least a
  `PROJ_IDS` const per family) and give `proj_data` a documented default arm instead
  of `None`, so an unlisted id fails loudly rather than silently. Coordinate with
  module 06 and the M4-PROJ family.
- **M — Port the `DustID` id table** (332 ids) with the dust-type → texture/frame
  mapping, so `Dust` emissions can name a vanilla dust type. Depends on module 15.
- **L — Port the `ArmorIDs` per-slot id spaces** (15 slots, Head 293 … Beard 5) and
  the `ArmorIDs.Sets.*` families, so armor sets, dyes and the vanity slots can be
  addressed by vanilla id. Cross-check with modules 07/20/29.
- **L — Port the remaining "no Rust counterpart" sets** — `SoundID` (the style
  registry behind `NPCDeathCount = 69` / `ItemSoundCount = 211`), `PlayerItemSlotID`
  (the dynamic slot registry), `ExtrasID` (316), `GlowMaskID` (384), `GoreID` (1407),
  `NPCHeadID` (81), `MessageID` (163), `CloudID` (42), `ChainID` (17),
  `ProjectileSourceID` (23), `ImmunityCooldownID` (6), `SurfaceBackgroundID` (16),
  `PlayerTextureID` (16), `PaintID`/`PaintCoatingID`, `ItemRarity`'s 12 colours. Each
  needs a generated table (`tools/GenTileData` is the house pattern); split per set.
- **L — Add the "id census" test harness.** A test that parses the C# `Count`
  constants out of `decompiled/Terraria/ID/*.cs` and compares them against the Rust
  constants (`TILE_COUNT`, `WALL_COUNT`, `BUFF_ID_COUNT`, `MAX_MUSIC`, `ITEMS.len()`,
  the prefix arm list), so any future id drift fails the suite instead of the audit.
  This is the mechanical guard the whole module exists to provide.
