# 99 — .plr complete audit (W4-f44 verify-sweep, 2026-11-21)

**Scope:** the FULL `Player.Serialize` walk (`Player.cs:54750-54933`, line-by-line)
vs `player_file.rs::serialize_plaintext`'s field order; the FULL
`Player.Deserialize` walk (`:55134-55764`) vs `read_player_fields`' version-gate
matrix; the legacy 279/inc1/inc2 shim coverage vs the W4-f44 families; the
golden-fixture divergence set. **Base:** `18c133ce` (PH3 DONE) on
`task/W4-f44-save-plr-2`. **Method:** direct file reads only (no junction
recursion), code-only — no cargo/tests run; every new witness is written, not run.

**Verdict:** the field ORDER and the section SET were already complete after
PH3 (no missing field, no `miscEquips2` in 1.4.5.8 — grep across `decompiled/`
returns nothing; nothing follows the dialogues). The sweep found **six semantic
gaps** (four read-policy, two version-gate) plus **one family of unmodelled
state fields** (the unlock/seed header block + hbLocked + the timestamp's read
side). All are ported in this sweep with witnesses; the loadouts remain g55's
family — this sweep only corrected their shared air policy (boundary noted
below).

---

## 1. Serialize offset table (release 326, fresh-default widths)

Offsets are PLAINTEXT (post-decrypt) byte positions for a fresh player with a
6-char name ("Golden"/"Belsin" = 1 length byte + 6 chars = 7). Variable-width
fields are marked; the golden fixture's total is **3827** bytes (3840 encrypted,
PKCS7 13×0x0D).

| # | Field | C# write | Width | Offset (fresh) | Modelled | Version gate (read) |
|---|-------|----------|-------|----------------|----------|---------------------|
| 0 | release `326` | `:54738` | i32 | 0 | const | `LoadPlayer:55079` |
| 1 | FileMetadata magic+type | `:54739` (`FileMetadata.cs:22-27`) | u64 | 4 | const | `:55082` |
| 2 | revision | same | u32 | 12 | const 0 | same |
| 3 | favorites flags | same | u64 | 16 | const 0 (sidecar) | same |
| 4 | name | `:54752` | 7-bit len + UTF8 | 24 | yes | ungated |
| 5 | difficulty | `:54753` | u8 | 31 | yes | 17 (`:55139-55148`) |
| 6 | playTime.Ticks | `:54754` | i64 | 32 | yes (ph3) | **138** (`:55150-55157`) — sweep fix |
| 7 | hair | `:54755` | i32 | 40 | yes | ungated; `>=228→0` `:55159` |
| 8 | hairDye | `:54756` | u8 | 44 | yes (ph3) | 82 (`:55163`) |
| 9 | team | `:54757` | u8 | 45 | yes (ph3) | 283 (`:55167`) |
| 10 | hideVisibleAccessory[0..8] | `:54758-54763` | BitsByte | 46 | yes (ph2) | 124 (`:55171`) / 83 one-byte arm |
| 11 | hideVisibleAccessory[8..10] | `:54764-54769` | BitsByte | 47 | yes (ph2) | 124 |
| 12 | hideMisc | `:54770` | BitsByte | 48 | yes (ph2) | 119 (`:55192`) |
| 13 | skinVariant | `:54771` | u8 | 49 | yes | 107 (`:55207-55214`); `<161 && 7→9` `:55215` |
| 14 | statLife | `:54772` | i32 | 50 | yes | ungated |
| 15 | statLifeMax | `:54773` | i32 | 54 | yes | ungated; clamp 500 `:55221` |
| 16 | statMana | `:54774` | i32 | 58 | yes | ungated; clamp 400 `:55231` |
| 17 | statManaMax | `:54775` | i32 | 62 | yes | ungated; clamp 200 `:55227` |
| 18 | extraAccessory | `:54776` | bool | 66 | **sweep** (was fresh-false) | 125 (`:55235`) |
| 19 | unlockedBiomeTorches | `:54777` | bool | 67 | **sweep** | 229 (`:55239`) |
| 20 | UsingBiomeTorches | `:54778` | bool | 68 | **sweep** | 229 |
| 21 | ateArtisanBread | `:54779` | bool | 69 | **sweep** | 256 (`:55243`) |
| 22 | the dead 324 slot | `:54780` `Write(value: false)` | bool | 70 | const false (C# hardcodes) | 324 (`:55247-55250`, read-dropped) |
| 23 | usedAegisCrystal | `:54781` | bool | 71 | **sweep** (`used_upgrades[0]`) | 260 (`:55253`) |
| 24 | usedAegisFruit | `:54782` | bool | 72 | **sweep** (`[1]`) | 260 |
| 25 | usedArcaneCrystal | `:54783` | bool | 73 | **sweep** (`[2]`) | 260 |
| 26 | usedGalaxyPearl | `:54784` | bool | 74 | **sweep** (`[3]`) | 260 |
| 27 | usedGummyWorm | `:54785` | bool | 75 | **sweep** (`[4]`) | 260 |
| 28 | usedAmbrosia | `:54786` | bool | 76 | **sweep** (`[5]`) | 260 |
| 29 | downedDD2EventAnyDifficulty | `:54787` | bool | 77 | **sweep** | 182 (`:55263`) |
| 30 | taxMoney | `:54788` | i32 | 78 | **sweep** | 128 (`:55267`) |
| 31 | numberOfDeathsPVE | `:54789` | i32 | 82 | **sweep** | 254 (`:55271`) |
| 32 | numberOfDeathsPVP | `:54790` | i32 | 86 | **sweep** | 254 (`:55275`) |
| 33 | hairColor RGB | `:54791-54793` | 3×u8 | 90 | yes | ungated |
| 34 | skinColor RGB | `:54794-54796` | 3×u8 | 93 | yes | ungated |
| 35 | eyeColor RGB | `:54797-54799` | 3×u8 | 96 | yes | ungated |
| 36 | shirtColor RGB | `:54800-54802` | 3×u8 | 99 | yes | ungated |
| 37 | underShirtColor RGB | `:54803-54805` | 3×u8 | 102 | yes | ungated |
| 38 | pantsColor RGB | `:54806-54808` | 3×u8 | 105 | yes | ungated |
| 39 | shoeColor RGB | `:54809-54811` | 3×u8 | 108 | yes | ungated |
| 40 | armor[20] (type,prefix,favorited) | `:54812-54817` | 20×(4+1+1)=120 | 111 | yes (ph2) | 38/124 count arms; fav 322 (`:55313`) |
| 41 | dye[10] (type,prefix,favorited) | `:54818-54823` | 10×6=60 | 231 | yes (ph2) | 47/81/124 count arms; fav 322 (`:55335`) |
| 42 | inventory[58] (type,stack,prefix,favorited) | `:54824-54830` | 58×10=580 | 291 | yes | 58 (`:55341`); fav 114 (`:55351`) |
| 43 | miscEquips[5]+miscDyes[5] interleaved (type,prefix ×2) | `:54831-54837` | 5×10=50 | 871 | yes (ph2) | 117 (`:55387`); **<136 skips slot 1** (`:55393`) — sweep fix |
| 44 | bank[40] (type,stack,prefix) | `:54838-54843` | 40×9=360 | 921 | yes (ph3) | 58 (`:55449`), <58 20-slot arm |
| 45 | bank2[40] | `:54844-54849` | 360 | 1281 | yes (ph3) | 58 (`:55457`) |
| 46 | bank3[40] | `:54850-54855` | 360 | 1641 | yes (ph3) | 182 (`:55479`) |
| 47 | bank4[40] (type,stack,prefix,favorited) | `:54856-54862` | 40×10=400 | 2001 | yes (ph1) | 198 (`:55488`); fav 255 (`:55495`) |
| 48 | voidVaultInfo | `:54863` | BitsByte | 2401 | yes (ph1) | 199 (`:55501`) |
| 49 | buffType/buffTime[44] | `:54864-54876` | 44×8=352 | 2402 | yes (ph3, compacted) | 11 (`:55578`); count 10/22/44 at 74/252 |
| 50 | spawn points spX/spY/spI/spN | `:54877-54888` | var; fresh = lone -1 i32 | 2754 | yes (ph3) | ungated; -1 sentinel, 200 cap |
| 51 | hbLocked | `:54889` | bool | 2758 | **sweep** (was fresh-false) | 16 (`:55612`) |
| 52 | hideInfo[13] | `:54890-54893` | 13×bool=13 | 2759 | yes (ph2) | 115 (`:55616`) |
| 53 | anglerQuestsFinished | `:54894` | i32 | 2772 | yes (ph3) | 98 (`:55624`) |
| 54 | DpadRadial.Bindings[4] | `:54895-54898` | 4×i32=16 | 2776 | yes (ph3) | 162 (`:55628`) |
| 55 | builderAccStatus[12] | `:54899-54902` | 12×i32=48 | 2792 | yes (ph3) | 164; count 8/10/11/12 at 164/167/197/230; <210/<249 fixups |
| 56 | bartenderQuestLog | `:54903` | i32 | 2840 | yes (ph3) | 181 (`:55675`) |
| 57 | dead | `:54904` | bool | 2844 | yes (ph3) | 200 (`:55679`) |
| 58 | respawnTimer (only when dead) | `:54905-54908` | i32 conditional | — | yes (ph3) | 200; read clamp 0..60000 (`:55684`) |
| 59 | DateTime.UtcNow.ToBinary() | `:54909` | i64 | 2845 | write: caller stamp; **read: sweep** | 202 (`:55688`) |
| 60 | golferScoreAccumulated | `:54910` | i32 | 2853 | yes (ph3) | 206 (`:55692`) |
| 61 | creativeTracker.Save | `:54911` (`ItemsSacrificedUnlocksTracker.cs:157-169`) | bool false + i32 count + pairs | 2857 | yes (ph1) | 218 (`:55696`); flag 282 (`Tracker.cs:173`) |
| 62 | temp slots BitsByte + triples | `:54912` (`:54935-54959`) | u8 + per-set-bit 9 bytes | 2862 | yes (ph3) | 214 (`:55700`) |
| 63 | creative powers | `:54913` (`CreativePowerManager.cs:183-195`) | (true,u16,bool)×2 + (true,u16,f32) + false = 16 | 2863 | yes (ph2) | 220 (`:55706`); unknown-id BREAK `:203-206` |
| 64 | super-cart BitsByte | `:54914-54918` | u8 (fresh 2) | 2879 | yes (ph3) | 253 (`:55710`); <253 cart-derive `:55716` |
| 65 | CurrentLoadoutIndex | `:54919` | i32 | 2880 | yes (ph1, **g55 family**) | 262 (`:55720`); clamp 0..2 |
| 66 | Loadouts[3].Serialize | `:54920-54923` (`EquipmentLoadout.cs:31-45`) | 3×(30×10+10)=930 | 2884 | yes (ph1, **g55 family**) | 262; fav 322 (`EquipmentLoadout.cs:51/:55`) |
| 67 | voiceVariant | `:54924` | u8 (fresh 1) | 3814 | yes (ph3) | 280 (`:55728`); <280 Male-derive; clamp 1..4 `:55831` |
| 68 | voicePitchOffset | `:54925` | f32 | 3815 | yes (ph3) | 281 (`:55738`); <281 → 0f |
| 69 | refunds count + triples | `:54926` (`CraftingRequests.cs:353-362`) | i32 + n×9 | 3819 | yes (ph3) | 300 (`:55746`) |
| 70 | dialogues count + strings | `:54927-54932` | i32 + n×string | 3823 | yes (ph3) | 310 (`:55755`) |
| — | END | | | **3827** | | `LoadPlayer_LastMinuteFixes:55763` |

**Boundary (g55):** rows 65-66 (the loadout index + the three `EquipmentLoadout`
payloads) are g55's family — PH1 modelled their byte transport; this sweep did
not touch their structure, only the shared `Item.DeserializeFrom` air policy
they consume (gap G1 below applies to every `DeserializeFrom` caller, g55's
included). The loadout *semantics* (swap, fix-loaded-data dedup) stay g55's.

**No unmodelled field remains.** Every `fileIO.Write` in `Serialize:54752-54932`
maps to a row above; the only deliberately-constant writes are the dead 324
slot (row 22 — the C# itself writes `value: false`, `:54780`) and the metadata
revision/favorites (rows 2-3 — `FileMetadata.Write`, the star lives in the
`.trfav` sidecar). `miscEquips2` does not exist in this decompile (grep: 0 hits);
the 1.4.5-new fields (artisan bread, the six used-upgrades, the 324 dead slot,
the refunds, the dialogues) are all present and now modelled.

## 2. Deserialize version-gate matrix (accepted range [135, 326])

Every gate in `Deserialize:55134-55764`, its status in the port BEFORE the
sweep, and after:

| Gate | C# | Field(s) | Before | After |
|------|----|----------|--------|-------|
| >=10/>=17 | `:55139-55148` | difficulty (bool→2 below 17) | always-true in range, cited | same (correct) |
| **>=138** | `:55150-55157` | playTime i64 | **MISSING — unconditional read desynced 135-137** | **fixed** (G5) |
| >=82 | `:55163` | hairDye | always-true, cited | same |
| >=283 | `:55167` | team | gated ✓ | ✓ |
| >=124 / >=83 | `:55171-55191` | hide BitsBytes (2 vs 1) | 124 arm (83 unreachable in range) | same |
| >=119 | `:55192` | hideMisc | always-true, cited | same |
| <=17 / <107 / else | `:55196-55214` | Male bool vs skinVariant | 107 arm ✓ | ✓ |
| <161 && skin==7 | `:55215` | →9 | ✓ | ✓ |
| >=125 | `:55235` | extraAccessory | always-true, cited | ✓ now captured |
| >=229 | `:55239` | torch pair | gated ✓ | ✓ captured |
| >=256 | `:55243` | artisan bread | gated ✓ | ✓ captured |
| >=324 | `:55247` | dead slot | gated ✓ (read-dropped, correct — C# discards) | ✓ |
| >=260 | `:55253` | six upgrades | gated ✓ | ✓ captured |
| >=182 | `:55263` | dd2 downed | gated ✓ | ✓ captured |
| >=128 | `:55267` | taxMoney | always-true, cited | ✓ captured |
| >=254 ×2 | `:55271-55278` | deaths PVE/PVP | gated ✓ | ✓ captured |
| >=38 / <124 vs >=124 | `:55286-55318` | armor 11/16-slot legacy vs 20 | 124 arm ✓ (legacy arms out of range, cited) | ✓ |
| >=322 | `:55313`/`:55335` | armor/dye favourited | gated ✓ | ✓ |
| >=47/81/124 | `:55319-55329` | dye count 3/8/10 | 124 arm ✓ | ✓ |
| >=58 / <58 | `:55341-55386` | inventory 58 vs 48 | 58 arm ✓ | ✓ |
| >=114 | `:55351`/`:55361` | inventory favourited | always-true, cited | ✓ |
| >=117 / **<136** | `:55387-55447` | misc pairs; **<136 skips slot 1** | **<136 arm MISSING (G6)** | **fixed** |
| >=58 / <58 | `:55449-55478` | banks 40 vs 20 | 58 arm ✓ | ✓ |
| >=182 | `:55479` | bank3 | gated ✓ | ✓ |
| >=198 | `:55488` | bank4 | gated ✓ | ✓ |
| >=255 | `:55495` | bank4 favourited | gated ✓ | ✓ |
| >=199 | `:55501` | voidVaultInfo | gated ✓ | ✓ |
| <38 | `:55506-55569` | legacy string-item layout | refused (pre-135 reject) ✓ | ✓ |
| <58 | `:55570-55577` | inventory 40-48 clone shift | out of range, cited | ✓ |
| >=11 / <74 / >=252 | `:55578-55599` | buffs 10/22/44 + compaction | gated ✓ | ✓ |
| ungated | `:55600-55611` | spawn points, -1 break, 200 cap | ✓ | ✓ |
| >=16 | `:55612` | hbLocked | always-true, cited | ✓ now captured |
| >=115 | `:55616` | hideInfo[13] | always-true, cited | ✓ |
| >=98 | `:55624` | angler | always-true, cited | ✓ |
| >=162 | `:55628` | dpad[4] | gated ✓ | ✓ |
| >=164/167/197/230 | `:55635-55653` | builderAcc count 8/10/11/12 | gated ✓ | ✓ |
| <210 / <249 | `:55654-55673` | builderAcc fixups (state-only) | ✓ | ✓ |
| >=181 | `:55675` | bartender | gated ✓ | ✓ |
| >=200 | `:55679-55686` | dead + conditional respawnTimer clamp | gated ✓ | ✓ |
| >=202 | `:55688` | lastTimePlayerWasSaved | gated ✓ (read-dropped) | ✓ now captured |
| >=206 | `:55692` | golfer | gated ✓ | ✓ |
| >=218 (+282 inside) | `:55696`/`Tracker.cs:173` | research flag + pairs | gated ✓ | ✓ |
| >=214 | `:55700` | temp slots | gated ✓ | ✓ |
| >=220 | `:55706` | powers loop + unknown-id BREAK | gated ✓ | ✓ |
| >=253 / else | `:55710-55719` | super-cart byte vs cart-derive | gated ✓ | ✓ |
| >=262 | `:55720` | loadout idx + 3 loadouts | gated ✓ | ✓ |
| >=280 / else | `:55728-55737` | voice byte vs Male-derive | gated ✓ | ✓ |
| >=281 / else | `:55738-55745` | pitch vs 0f | gated ✓ | ✓ |
| >=300 | `:55746` | refunds | gated ✓ | ✓ |
| >=310 | `:55755` | dialogues | gated ✓ | ✓ |
| post | `:55763`→`:55828-55860` | LastMinuteFixes: skin clamp, voice clamp, equip-derived stats, FixLoadedData | clamps ✓; the equip-stat derivations + FixAgainstExploit are the live player-equip pass's job, not the card reader's (documented) | same |

**Matrix verdict:** complete after the sweep. The two genuinely-variable gates
inside [135,326] that were missing — 138 (playtime) and 136 (misc slot-1 skip) —
are now implemented; every other gate was already correct or provably constant
in the accepted range.

## 3. The found gaps (ported this sweep)

- **G1 — bank4/loadout over-count dropped stack+favorited.** C#
  `Item.DeserializeFrom` (`Item.cs:49921-49933`): `netDefaults(id)` airs an
  out-of-band id FIRST (`:49923` → `SetDefaults:48671-48674` sets `type = 0`,
  `:48685-48688` zeroes the stack), then `stack = ReadInt32()` OVERWRITES the
  aired stack with the file's value (`:49924`), `Prefix` no-ops on air
  (`:49925`→`:515-518`), and the favourited bool rides (`:49928`). The final
  `if (this.type >= ItemID.Count) TurnToAir()` check (`:49930-49933`) sees
  **type 0** — already zeroed inside `netDefaults` — and does NOT fire
  (`TurnToAir:49884-49890` is never reached). Net result:
  **(0, file-stack, 0, file-favorited)**. The PH1 whole-slot drop
  `(0,0,0,false)` lost the stack and the star; fixed.
- **G2 — temp/refund over-count kept the prefix.** Same `DeserializeFrom`
  path: `Prefix` no-ops on the aired item, so the prefix must air while the
  stack rides. PH3 kept `(0, stack, prefix)`; corrected to `(0, stack, 0)`.
- **G3 — negative ids below -48 rode verbatim** in the triple families.
  `netDefaults` (`Item.cs:1213-1361`) has cases ONLY for -1..-48; any other
  negative leaves the fresh `new Item()` (type 0, stack 0) untouched, then
  the stack read overwrites and `Prefix` no-ops → `(0, file-stack, 0)`
  (+ the favourited bool for bank4/loadouts). The old `id < ITEM_ID_COUNT`
  guard kept every negative verbatim; fixed.
- **G4 — type-0-with-prefix kept the prefix** in the bank/inventory reads.
  `Prefix` no-ops on ANY air item (`CanHavePrefixes` false for type 0,
  `Item.cs:493-496`/`:1101-1137` — no prefix set matches type 0), so a
  `(0, n, p≠0)` file slot reads `(0, n, 0)`. Fixed via `air_checked_triple`.
- **G5 — playtime gate 138 missing** (see matrix). Fixed.
- **G6 — misc <136 slot-1 skip missing** (see matrix). Fixed.
- **G7 — the unlock/seed header block + hbLocked + the timestamp read side
  were unmodelled state** (read-dropped, written fresh). A vanilla character's
  Demon Heart slot, biome-torch unlock, artisan bread, the six used-upgrade
  flags, the DD2 downed flag, tax coin, death counters, hotbar lock and
  last-saved stamp were LOST on a port re-save. Now ride both models
  (`extra_accessory`, `unlocked_biome_torches`, `using_biome_torches`,
  `ate_artisan_bread`, `used_upgrades[6]`, `downed_dd2_any_difficulty`,
  `tax_money`, `deaths_pve`, `deaths_pvp`, `hb_locked`, `last_saved_binary`
  on `PlayerFile`; the write side on `PlayerFileWrite`).

### G1/G2 resolution (the DeserializeFrom air policy, final)

The decisive line is `SetDefaults:48671-48674`: `netDefaults(over-count)` runs
`ResetStats(Type)` (which sets `this.type = Type`, `:49050`) and THEN
`if (this.type >= ItemID.Count) this.type = 0;` — the type is zeroed INSIDE
`netDefaults`, before the stack read. So `DeserializeFrom`'s final
`if (this.type >= ItemID.Count) TurnToAir()` (`:49930-49933`) tests **0** and
never fires; `TurnToAir` is dead code on this path for over-count ids. The
final state is **(0, file-stack, 0, file-favorited)** — the stack read
(`:49924`) overwrites the aired stack, `Prefix` no-ops on air, the favourited
bool (`:49928`) rides. The same holds for the below-(-48) negatives
(`netDefaults`' switch has no case → the fresh air item stays, type 0).

The BANK family (bank/bank2/bank3, read inline at `Player.cs:55453-55455`
with netDefaults/stack/Prefix and NO final check at all) behaves identically:
**(0, file-stack, 0)**. The inventory's >= Count arm (`:55346-55355`) is the
one exception: the C# reads-and-DISCARDS the whole record (stack, prefix,
favourited all dropped) and `netDefaults(0)` leaves the slot air —
**(0, 0, 0)** + no star. Its < Count arm goes through netDefaults/stack/Prefix
like the banks: (0, file-stack, 0) + the star rides.

**Port shape:** one shared `air_checked_triple` (live id 1..Count-1 or
-48..-1 rides verbatim; anything else → (0, file-stack, 0)) for the banks,
bank4, loadouts, temp slots, refunds and the inventory's < Count arm; the
inventory's >= Count arm keeps its read-and-drop. `air_checked_pair` is the
equip flavour (no stack): out-of-band → (0, 0), favorited rides.

## 4. Legacy shim coverage vs the new families

| Shim | Shape | Families it carries | Families it defaults | New arms needed? |
|------|-------|--------------------|----------------------|------------------|
| 279 subset (`read_legacy_port_subset`) | ends after the 58-slot inventory; no hideMisc, no team, no 324 bool, no favourites | name/diff/playtime/hair/hairDye, hide BitsBytes, skin/stats, **the 11 unlock bools + tax + deaths (sweep: now captured)**, colors, armor/dye (no fav), inventory + favs | everything past the inventory: banks, buffs, spawn, quest counters, temp slots, superCart (2), voice (1), refunds, dialogues, loadouts, vault, research, powers, hbLocked, timestamp | **NO** — the subset never wrote any ph1-3 family; the sweep only extended its EXISTING header capture (the 11 bools + 3 i32 it already consumed). No new section arms. |
| inc1 (depth 1, cut at voidVaultInfo `:55504`) | full 326 prefix through the vault byte | header (incl. the sweep's new fields), armor/dye/misc, banks 1-3, bank4, vault | buffs onward | **NO** — the cut is before every ph3 family except the header; the shared `read_player_fields` prefix carries the new fields automatically. |
| inc2 (depth 2, cut at hideInfo `:55623`) | through hideInfo | + buffs, spawn points, hbLocked (sweep) | angler onward | **NO** — same shared-prefix argument. |

The retry ladder (`read_player_bytes:505-529`) is unchanged: 279 → subset shim;
326 truncation → depth 1 → depth 2. The sweep's new header fields sit BEFORE
both cuts, so all three shims capture them with zero new arms.

## 5. Golden-fixture divergence set

`golden.plr` (C#-written, fresh player, FixtureClock 2026-11-18T00:00:00Z):
**every field the port models is fresh-default-identical** — the fixture's
player is the vanilla ctor state, and every `PlayerFileWrite::default` emits
exactly that byte (the module's default-value policy). The sweep's new fields
default to the same fresh values the old hardcoded writes emitted
(false×11, 0×3, hbLocked false), so:

- `golden_plr_writer_matches_the_csharp_fixture_byte_for_byte` — unchanged
  expectation: 3827-byte plaintext + PKCS7 13 = 3840 bytes, byte-identical.
- `golden_layout_through_hide_info` — unchanged: the offset table above is
  the fixture's layout; the tail block 2402..3827 compares with the timestamp
  spliced (the one caller-owned value).
- `the_golden_fixture_tail_reads_fresh` — the new fields read fresh too
  (extra_accessory false, used_upgrades all-false, tax/deaths 0, hb_locked
  false, last_saved_binary = the FixtureClock value on the READ side — the
  fixture carries 0x48DF4506E391C000 at 2845, so the read now surfaces it
  instead of dropping it; the fresh-WRITE default stays 0).

**The only deliberate divergences** (all pre-existing, documented in ANALYSIS;
**re-verified by the crosspersist pass — see `132-persist-master-map.md` §6**:
#1 NARROWED by the `d331ea7c` timestamp fix — the creation save stamps UtcNow
and the disk read ports the `LoadPlayer:55096-55099` mtime fallback, the
byte-level codec stays caller-stamped for golden parity; #2/#3 now provably
C#-identical — `InternalSavePlayerFile:54739` calls `Metadata.Write`, NOT
`IncrementAndWrite` (whose only call site is `WorldFile.cs:1175`), and vanilla
`IsFavorite` is never set true, the star lives in `favorites.json`; #4/#5/#6
still deliberate):
1. the write timestamp is caller-stamped (0 = unknown) vs the C#'s UtcNow —
   the golden test pins the FixtureClock value;
2. the metadata favorites u64 is always 0 (the star lives in the `.trfav`
   sidecar, `favorite_marker_path`);
3. the revision u32 is always 0 (`FileMetadata.Write`, not Increment);
4. the legacy -1..-48 ids ride verbatim (no item-defaults tables in the card
   model) — a re-save round-trips the bytes, the mapping is deferred;
5. the <253 cart-derive treats a non-air equip slot as stack>0 (no stack in
   the equip card model);
6. the dialogues keep first-occurrence order (the HashSet's practical add-only
   enumeration order).

**No NEW divergence was introduced by the sweep** — every corrected read
policy moves the port TOWARD the C#, and every new write field defaults to
the byte the old writer hardcoded.

## 6. Witnesses (written, code-only — not run)

New: `the_unlock_and_counter_header_fields_round_trip` (the G7 block, byte
offsets 66-90 + hbLocked 2758 pinned), `a_pre_138_file_carries_no_playtime_i64`
(hand-built 135/137/138 bodies, the hair alignment probe),
`the_aired_slot_keeps_its_stack_and_favorited_from_the_file_bytes` (hand-spliced
file bytes at the table's offsets: bank4 2001, bank 921/930, loadout 2884,
temp 2862, refunds 3819, inventory 291/301, armor 111 — the G1-G4 policies).
Updated: `over_count_tail_items_air_without_desyncing` (bank4/loadout aired
slots now pin (0, stack, 0, fav)), `the_refunds_round_trip_as_count_and_triples`
and `the_temp_slots_round_trip_through_the_bits_byte` (the prefix airs).

**Verification performed:** line-by-line re-read of both C# walks against the
port (this doc's tables), offset arithmetic cross-checked against the existing
`golden_layout_through_hide_info` witness (3827 total), and a full `git diff`
re-read. No cargo/tests/graphify per the task constraints.
