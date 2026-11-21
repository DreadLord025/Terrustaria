# 132 — The persist MASTER MAP: three layers, one owner per field

**Status: THE CROSS-LAYER CONTRACT.** Written by W4-f44 crosspersist
(branch `task/W4-f44-save-plr-2`, base `180b7430` audit DONE + the
`d331ea7c` timestamp residual fix). This doc sits ABOVE the per-layer
contracts — research/99 (`.plr`, this lane), research/103 v9 + research/122
(`.trplayer`, g88/g89), research/92-doc-sweep-bg + research/135 (`.trwld`
union, g70×g71×g90) — and fixes the **field ownership** between them:
which layer carries which field, the merge order per layer, and the
back-compat matrix across all three. Code-only lane: witnesses written,
not run.

Inputs read at probe (2026-11-21, second fetch — the branches moved
mid-probe; these are the tips this doc describes):

| Ref | Tip read | Persist state |
| --- | --- | --- |
| `origin/task/W4-g89-aegis-fruit-use` | `598642563` (g89f over g89e `e9ec8dcfc` / g89d `4344bd50`) | `.trplayer` **v9** 3-token unlock line (research/122); `.plr` fruit/gummy/ambrosia bools both halves; `.wld` book-two byte (g89e); `.trwld` stays **v17** |
| `origin/task/W4-g88-life-fruit-use` | `6998a4ecb` (fold 5 docs over `afa371f16`) | `.trplayer` **v9 SUPERSET** (research/103 v9 — g89c folded, tokens 4+ reserved); carries research/122 blob-verbatim |
| `origin/task/W4-g90-artisan-bread-use` | `b3b82d071` (reconcile-input over lane 7 `73367e0af`) | `.trplayer` **v7** six 1-token lines + `.plr` bools 1/3/4/5/6 + `.trwld` **v18** 2-byte tail; research/**135** is its mechanical fold-input |
| `origin/task/W4-g71-bg-layers-2` | `e2d085ae4` | `.trwld` **v18** 3-byte surface tail; research/**92-doc-sweep-bg** §1 = the 19-byte g70×g71 union |
| `origin/task/W4-g70-bg-layers-1` | `6b3049e19` (persist at `bbe47378` lineage) | `.trwld` **v18** 16-byte forest tail |
| `origin/task/W4-g55-ui-loadout-save` | `cde865fc2` | `.plr` loadout family + `.trplayer` v7 loadout block (folded into g88's superset) |
| `origin/task/W4-a6-crystals-powerups` | `d46221b2b` | NO persist writer (`.plr` crystal slots stay hardcoded false, no `persist.rs` change); `.wld` book-two/satchel/bought-pets bytes — overlaps g90 lane 7 / g89e |
| `origin/task/W4-e10-player-sounds-wave` | `2c024e444` | NO persist change — `voice_variant`/`voice_pitch_offset` are Player fields only (§4 below) |
| this branch | `d331ea7c` | `.plr` complete (research/99) + the timestamp fix; `.trplayer` stays main's **v6**; `.trwld` stays **v17** |

---

## 1. The three layers (what each file IS)

| Layer | File | Format | Writer / reader | Job |
| --- | --- | --- | --- | --- |
| **L1 `.plr`** | `Players/<name>.plr` | vanilla binary, AES-128-CBC, release **326** | `crates/tr-app/src/player_file.rs` | the CHARACTER: identity, appearance, stats, inventory/banks/loadouts, every per-player unlock/counter, the full `Player.Serialize` field set (`Player.cs:54750-54933`). Vanilla-interchangeable both ways. **Canonical field map: research/99's offset table** (71 rows, 3827-byte fresh layout). |
| **L2 `.trplayer`** | `<world>.trplayer` | port-invented versioned TEXT | `crates/tr-app/src/persist.rs` | the LIVE SESSION next to a `.trwld`: position/hp/coins + the fields the live App mutates and must restore on world reload. NOT a `.plr` mirror — a lossy projection (kinds-only equips, 50-slot inventory). **Canonical contract: research/103 (v9 superset) + research/122 (line-8 addendum); g90's fold-input: research/135.** |
| **L3 world** | `<world>.trwld` (native) / `<name>.wld` (vanilla) | port binary v17→v18 union / vanilla sections release 326 | `tr-core/src/save.rs` / `tr-core/src/wld/` | the WORLD: tiles, chests, NPCs, world-global flags. **`.trwld` union contract: research/92 §1 (19 bytes) extended by research/135 §4 (21 bytes).** `.wld` flags codec: `wld/sections/flags.rs` (the C# gates). |

Side satellites (not layers): `<stem>.trfav` (the character star —
vanilla's `favorites.json` store, `IO/FavoritesFile.cs:27-36`; the `.plr`
metadata bit is dead in vanilla too), `<world>.map` (explored map),
`achievements.trdat`.

**The routing rule that keeps the layers apart** (`saveglue::save_world`,
`app/saveglue.rs:88-296`): a `.trwld` world saves native + the
`.trplayer`/`.map` sidecars; a `.wld` world saves vanilla-shaped and
writes NO player sidecar (`.wld` player state has no port format —
recorded ANALYSIS deviation). The `.plr` is written ONLY by the
character-creation path (`session.rs::save_created_character`) — there is
no in-session `.plr` re-save yet (the vanilla `Player.SavePlayer` per
world-exit is the recorded remainder). Consequence for this map: today
**no code path writes the same field to two layers in one save** — but
the g-lanes' sidecar extensions overlap the `.plr`'s field set, so the
ownership table below is what the merged save funnel must honour.

## 2. Field ownership — which fields live where (no dup-writes)

### 2a. L1-only (the `.plr` carries them; the sidecar must NOT grow lines for them)

Everything in research/99's table EXCEPT the §2b rows. The families the
sidecar has no business mirroring: name/difficulty/appearance/colors
(the character card), playtime, hairDye/team, the hide flags, banks
1-4 + voidVaultInfo, buffs, spawn points, the quest/death counters
(angler/dpad/builderAcc/bartender/golfer), dead+respawnTimer, the
research pairs, creative powers, temp slots, super-cart, refunds,
dialogues, hbLocked, the save timestamp, the metadata block. Also
**L1-only among the unlock family**: `extraAccessory` (`:54776`, g87's
demon-heart lane — research/122 §4.4 already pins "no sidecar line"),
the biome-torch pair (`:54777-54778`), `downedDD2EventAnyDifficulty`
(`:54787` — 103 §2a token 7 stays RESERVED; if a DD2 lane ever needs it
live, the token is there, but no pushed lane claims it), taxMoney, the
death counters.

### 2b. The contested fields — BOTH layers carry them today (the boundary)

The g-lanes extended the sidecar with fields the `.plr` also carries.
**This is a deliberate dual-carrier, not an accident** — the two files
have different lifetimes: the `.plr` is written at character creation
(and, once the exit-save lands, at world exit), the sidecar at every
world autosave. A field the live session mutates (an upgrade eaten
mid-world) must ride the sidecar or it is lost between `.plr` writes;
the same field must ride the `.plr` or a vanilla round-trip loses it.

| Field | `.plr` slot (research/99 row) | `.trplayer` carrier (union) | Live owner |
| --- | --- | --- | --- |
| `statLifeMax`/`statLifeMax2` | rows 15 (+ effective hp in L2 line 2) | line 9 life pair (g88) | g88 |
| `statManaMax`/`statManaMax2` | row 17 | line 10 mana pair (g88) | g88 |
| `usedAegisFruit` | row 24 (`used_upgrades[1]`) | line 8 token 1 (g89) | g89 |
| `usedGummyWorm` | row 27 (`[4]`) | line 8 token 2 (g89b) | g89b (field pre-existed i35) |
| `usedAmbrosia` | row 28 (`[5]`) | line 8 token 3 (g89c) | g89c |
| `usedGalaxyPearl` | row 26 (`[3]`) | line 8 token 4 RESERVED→g90 fold | g90 lane 7 |
| `usedArcaneCrystal` | row 25 (`[2]`) | line 8 token 5 RESERVED→g90 fold | g90 lane 7 |
| `usedAegisCrystal` | row 23 (`[0]`) | line 8 token 6 RESERVED→g90 fold | g90 lane 7 |
| `ateArtisanBread` | row 21 | line 11 (g90, folded g88) | g90 |
| `CurrentLoadoutIndex` + `Loadouts[3]` | rows 65-66 | lines 12-15 (g55, folded g88) | g55 |
| inventory/armor/dye/coins/hp/pos | rows 40-42 (+ stats) | lines 2-7 (v1-v6 prefix) | pre-existing |

**The no-dup-write rules for the merged save funnel:**

1. **One writer per file per save.** `player_file.rs` writes L1;
   `persist.rs` writes L2; `save.rs`/`wld` write L3. No lane's writer
   opens another layer's file (today: `persist.rs` never touches the
   `.plr`; `player_file.rs` never touches the sidecar — verified by the
   module reads). Keep it that way.
2. **Within L1, one field slot per field.** The g89/g90 `.plr` halves
   and this lane's G7 `used_upgrades[6]` array model THE SAME SIX BYTES
   (`:54781-54786`). At the merge, keep ONE model: this lane's
   `used_upgrades[6]` (file-order array, both models, all three shim
   depths ride it) is the superset — g89's three named bools map to
   indices 1/4/5, g90's five to 0/2/3/4/5. The named-field lanes'
   `PlayerFile`/`PlayerFileWrite` fields dedup onto the array (or the
   array onto six named fields — either way ONE set, never both).
   Same for `ate_artisan_bread` (g90's field == this lane's G7 field,
   same name, same slot — a clean textual dedup).
3. **Within L2, one token per field.** 103 §2a's reserved positions are
   the single registry; g90's six-line v7 block folds into tokens per
   research/135 §2-3 (its lines are a LEGACY READ shape, never a second
   write shape). The 103-vs-122 token-order conflict for 4/5/6 is
   resolved by 135 §3: **103 §2a wins** (pearl=4/arcane=5/crystal=6) —
   no pushed writer ever emitted token 4+, so no back-compat obligation
   attaches to either order.
4. **L3 world globals never ride L2.** `combatBookWasUsed` /
   `combatBookVolumeTwoWasUsed` / `peddlersSatchelWasUsed` /
   bought-pets are WORLD flags (`NPC.cs:6339-6343`, `WorldFile.cs:1392-1394`/
   `:1407-1408`) — `.wld` flags block + the `.trwld` union tail only.
   Never a sidecar token (122 §5 already pins this; g90's use arms write
   the World fields, not Player fields).

### 2c. The G7-vs-sidecar semantics census (the brief's question)

**Is the sidecar a port-invention for pre-unlock state?** No — the
sidecar bools and the `.plr` G7 bools are THE SAME vanilla fields
(`Player.usedAegisFruit` etc., ctor `:56078-56083`), captured at
different times. The semantic difference is the WRITE MOMENT, not the
meaning:

- **G7 (this lane)** models the `.plr` slots so a VANILLA character's
  already-used upgrades survive a port read→re-save (the audit's loss
  case). It invents no state; `PlayerFile.used_upgrades` is a verbatim
  byte carrier. The live `Player` on THIS branch has only
  `used_gummy_worm` (i35) + `ate_artisan_bread` (reach formula) — the
  other four G7 slots have no live consumer here; the g-lanes own them.
- **g89/g90 sidecar bools** carry the LIVE session's unlock state
  (the use arms set `Player.used_*`, the booster rows read them every
  tick) across world reloads, because the `.plr` is not re-saved
  in-session. Same field, same meaning — a second carrier for a
  different save cadence.
- **The merge boundary:** the live `Player.used_*` fields (g89/g90's
  declarations) are the single live truth; the sidecar restores them at
  world load (`app/mod.rs`'s `persist::load` block); the `.plr` G7
  carrier restores them at CHARACTER load (once the character-load path
  hydrates a live Player — today the `.plr` reader feeds only the card
  list, `save_list.rs:294`). When both restores exist, the character
  load runs FIRST (App::new spawns the Player), the sidecar restore
  second (world load) — the sidecar wins for a world that was saved
  after the character file, which is the correct precedence (it is the
  newer state). A `.plr`-only field (extraAccessory etc.) has no sidecar
  line, so no precedence question arises.

## 3. Per-layer merge order

### L1 `.plr` (player_file.rs — hot file, four lanes touch it)

1. **Base = THIS branch** (`d331ea7c`): the complete 71-row walk, the
   G1-G7 fixes, the three shim depths, the timestamp fix. It is the
   only branch whose `player_file.rs` models EVERY field; the g-lanes'
   versions are main-base + their own slots.
2. **Fold g55's loadout semantics** (the swap machine's
   `set_shared`/fix-loaded-data hooks ride this branch's
   `LoadoutData` transport; byte layout identical — g55's
   `EquipmentLoadout.cs:31-45` walk == research/99 rows 65-66).
3. **Dedup g89's + g90's named unlock fields onto `used_upgrades[6]`**
   (§2b rule 2). Their read arms (`:55255-55260`) and write arms
   (`:54781-54786`) are already covered by the G7 array walk — the
   fold DELETES their duplicate reads/writes, keeps their live
   `Player.used_*` fields + use arms + booster rows. g90's legacy-279
   indices 4/6/7/8/9 already ride this branch's subset capture.
4. **Keep the timestamp fix whole** (`read_player_file`'s mtime
   fallback + `utc_now_binary`); g89/g90's `last_saved_binary: 0`
   literals in their session/test diffs dedup onto this branch's
   `utc_now_binary()` (session) / stay 0 (fixtures).
5. Anchors: this lane's rows stay; the g-lanes' `.plr` witnesses
   re-pin onto the array fields (their assertions are slot-value
   assertions — mechanically portable).

### L2 `.trplayer` (persist.rs — the g88 superset wins)

Per research/103 §6 + research/122 §4 + research/135 §6, in order:

1. **Base = g88 tip `6998a4ecb`** (writer v9 superset, the tolerant
   v7/v8/v9 reader, the loadout block, research/103+122 in-tree).
2. **Extend the v7 legacy arm** with 135 §2's six-1-token-line rule
   (g90 lane-7 files load positionally; every other v7 shape keeps
   103 §3's walk).
3. **Activate tokens 4/5/6** (pearl/arcane/crystal, 103 §2a order per
   135 §3) → writer **v10**; short-line back-reads for v7/v8/v9 kept.
4. **Dedup** g90's `SavedPlayer` flag fields + App restores onto the
   superset's (gummy/ambrosia already folded; bread already folded).
5. This lane contributes NOTHING to L2 (main's v6 reader/writer is a
   strict prefix of the superset; the merge takes g88's file whole).

### L3 world (save.rs `.trwld` + wld/ `.wld`)

1. **`.trwld` base = the 21-byte v18 union** (research/92 §1's 19
   forest-first bytes + research/135 §4's book-two/satchel pair at
   offsets 19/20). VERSION stays 18; forward-only (no released v18).
   The six truncation-constant moves per 135 §4's table + 92 §1's list.
   g70/g71/g90's round-trip witnesses all re-pin 16/3/2 → 21.
2. **`.wld` flags dedup**: g89e, g90 lane 7 and a6 ALL write
   `combatBookVolumeTwoWasUsed` (gate 259) — keep ONE (g90's pair also
   owns the satchel byte, gate 260; a6 adds the bought-pets trio,
   gates 217, tail 2021-2023 — independent bytes, no conflict).
3. This lane contributes NOTHING to L3 (no save.rs/wld change).

## 4. The e10b voice co-lane (research/99 row 67 cross-note)

**`origin/task/W4-e10-player-sounds-wave` tip `2c024e444`: NO `.plr`
write landed.** The branch's diff against main touches neither
`player_file.rs` nor `persist.rs` (verified `git diff --stat` — empty
for both). What it DOES carry:

- `Player.voice_variant: i32` (ctor 1, `Player.cs:1346`/`:56045`),
  `voice_pitch_offset: f32` (`:1348`), `voice_override: i32` (`:1706`)
  — live fields feeding the ported `PlayHurtSound`/`PlayDeathSound`
  decision trees (`player.rs:1933-1948` at its tip).
- Its ANALYSIS deviation note: "**No voice-picker / save-load
  surface.** `voiceVariant` stays at the ctor's 1 ... until a
  character-creation/save surface lands."

**Row 67 status (this lane):** COMPLETE on the `.plr` side —
`voice_variant: u8` + `voice_pitch_offset: f32` ride both models
(`player_file.rs:361/364`, write `:54924-54925` slots, read gates
280/281 with the <280 Male-derive `{0,1,2,3,8,10}` and the
`LastMinuteFixes` 1..4 clamp `:55831`). The golden fixture's fresh
voice byte 1 + pitch 0f are pinned.

**The merge seam (clean, no conflict):** e10's live field is `i32`,
this lane's file field is `u8` — the hydration is
`p.voice_variant = file.voice_variant as i32` (the 1..4 clamp already
applied on the read side) and `p.voice_pitch_offset =
file.voice_pitch_offset`, at the future character-load restore block.
The save direction casts back (`file.voice_variant = p.voice_variant
as u8`). e10 owns the CONSUMERS (the voice trees, the pitch remap
`Utils.Remap(-1,1,-0.7,0.7)`); this lane owns the TRANSPORT. Neither
branch needs a change for the other to land — the fields are declared
at different layers with no textual overlap. The character-creation
voice picker (S1-H-11's clipboard note: the creator writes the gender
default `Male ? 1 : 2`) is the remaining UI seam, not a persist gap.

## 5. The back-compat matrix (cross-layer)

| File shape | Layer | Reader that loads it | Result |
| --- | --- | --- | --- |
| vanilla `.plr` 135-326 | L1 | this branch `read_player_fields` | full gate matrix (research/99 §2); 135-137 no playtime, <136 misc slot-1 skip |
| vanilla `.plr` <135 | L1 | refused (`pre-135`) | the legacy string-item layout is unmodelled (documented) |
| port 279 subset | L1 | `read_legacy_port_subset` | header + armor/dye/inv; tail defaults; G7 bools ride |
| port 326 inc1/inc2 | L1 | depth-1/2 retries | reached sections ride, rest defaults (research/99 §4) |
| port `.plr` 0-stamp | L1 | `read_player_file` | **mtime fallback fills the stamp** (`d331ea7c`, `LoadPlayer:55096-55099`); byte-level read keeps 0 |
| g89/g90 `.plr` (main-base + their slots) | L1 | this branch's walk | their slots ARE G7 slots — loads complete; their own readers load this branch's files too (same bytes) |
| `.trplayer` v1/v2 | L2 | any lane reader | kinds migrate (`migrate_kind`) |
| `.trplayer` v3-v6 | L2 | any lane reader | prefix rides, post-dye defaults |
| `.trplayer` v7 (5 historical shapes + g90 six-line) | L2 | g88 superset + 135 §2 rule | all load (103 §4 matrix + the six-line row) |
| `.trplayer` v8/v9 short lines | L2 | g88 superset / g89 reader | tokens 1-3 back-read, rest false |
| `.trplayer` v9 g88-superset | L2 | g89's reader | line 8 reads; pairs/bread/loadout lines IGNORED (g89's reader stops after line 8 — lossy but aligned; the superset reader is the merge winner) |
| `.trplayer` v9 g89-file | L2 | g88 superset | unlock line positional, rest defaults |
| `.trplayer` v7 g90-six-line | L2 | g89's reader | **CORRUPTS** (bread→aegis; 135 §2) — the fold's recognition rule is mandatory |
| `.trwld` ≤17 | L3 | any v18 reader | union tail unread, Reset defaults |
| `.trwld` v18 g70-only (16B) / g71-only (3B) / g90-only (2B) | L3 | the 21-byte union reader | **EOF/garbage — not back-compat** (92 §1: forward-only, no released v18; all three are unmerged task work) |
| `.wld` any release | L3 | `wld::parse` gates | per-field gates; book-two 259 / satchel 260 / pets 217 ride after the g90/a6/g89e dedup |

## 6. The six pre-existing divergences — re-verified (C# re-read)

Research/99 §5's list, each re-checked against the decompile this pass:

1. **Write timestamp caller-stamped vs UtcNow** — **NARROWED, partially
   fixed (`d331ea7c`).** C# re-read: `Serialize:54909` writes
   `DateTime.UtcNow.ToBinary()` unconditionally; `LoadPlayer:55096-55099`
   falls back to the file mtime on a zero stamp. The port now: (a) the
   creation save stamps the real clock (`utc_now_binary`), (b) the disk
   read ports the mtime fallback. REMAINING deliberate residue: the
   byte-level writer stays caller-stamped (the golden fixture pins
   FixtureClock — a clock inside the codec would break byte-parity),
   and the stamps carry whole seconds, not the C#'s sub-second ticks
   (no consumer reads finer than a date). Witnesses:
   `the_utc_binary_encoding_pins_the_epoch_and_the_fixture_clock`,
   `a_zero_stamp_falls_back_to_the_file_mtime_on_the_disk_read`.
2. **Metadata favorites u64 always 0** — **STILL DELIBERATE, now
   provably C#-identical for the port's write path.** C# re-read:
   `FileMetadata.Write` (`FileMetadata.cs:22-27`) writes
   `IsFavorite.ToInt()`, but `IsFavorite` is only ever set false
   (`FromCurrentSettings:41`); the live star lives in `favorites.json`
   (`FileData.cs:35` reads `LocalFavoriteData.IsFavorite`, NOT the
   metadata). `CreateAndSave` (`PlayerFileData.cs:53`) builds fresh
   false metadata. So a vanilla-created `.plr` ALSO carries 0 here —
   the port's `.trfav` sidecar is the faithful analogue of
   `favorites.json`, not a divergence from the file bytes. The reader
   still ORs the header bit (`save_list.rs:300`) for foreign tools.
3. **Revision u32 always 0** — **STILL DELIBERATE, now provably
   C#-identical.** C# re-read: `InternalSavePlayerFile:54739` calls
   `playerFile.Metadata.Write` — NOT `IncrementAndWrite`. The ONLY
   `IncrementAndWrite` call in the decompile is `WorldFile.cs:1175`
   (the WORLD file). A vanilla `.plr` re-save keeps its revision
   forever (0 for a created character). The port's constant 0 matches
   every vanilla `.plr` the port can produce. (The `.wld` writer DOES
   increment — `wld/mod.rs`, witnessed by
   `test_DW_7_3_..._increments_the_revision`.)
4. **Legacy -1..-48 ids ride verbatim** — **STILL DELIBERATE.** C#
   re-read: `netDefaults` (`Item.cs:1211-1361`) maps -1..-48 to real
   items (3521..3480 etc.); the port keeps the negative id so a re-save
   round-trips the bytes. The mapping needs the item-defaults tables
   in the card model (the card carries no item stats) — deferred, the
   byte transport is lossless. The below-(-48) band airs correctly
   (G3, `Item.cs:1213`'s switch has no case).
5. **<253 cart-derive equip stand-in** — **STILL DELIBERATE.** C#
   re-read: `HasItemInAnyInventory(3353)` (`Player.cs:55931-55970`)
   walks inventory/armor/dye/miscEquips/miscDyes/banks via `HasItem`
   (`:55906-55916`, `type == && stack > 0`). The port's derive
   (`player_file.rs:1062-1069`) covers ALL nine collections in the C#
   order; the equip arrays carry no stack in the card model, so a
   non-air equip slot stands in for stack>0. A file that actually
   parks a cart in an equip slot with stack 0 is the only divergence
   case — vanishingly rare, documented.
6. **Dialogues first-occurrence order** — **STILL DELIBERATE.** C#
   re-read: `oneTimeDialoguesSeen` is a `HashSet<string>`
   (`Player.cs:3209`); `Serialize:54927` snapshots it into a List
   (insertion-order in practice for an add-only set), `Deserialize:55760`
   `.Add`s (dedupes). The port's Vec keeps first occurrences on both
   sides — the practical HashSet enumeration order, byte-identical for
   every add-only history. A remove-then-readd history could reorder
   the C# set; no port code removes dialogues (`OneTimeDialogue.cs:23`
   is the only writer, add-only).

**No new divergence introduced by the crosspersist pass** — the
timestamp fix moves the port TOWARD the C# on both the write and read
sides; the golden-fixture witnesses are untouched (the codec's
caller-stamped contract is preserved).

## 7. Witnesses (this pass — written, code-only, not run)

- `the_utc_binary_encoding_pins_the_epoch_and_the_fixture_clock` —
  the DateTime binary encoding pins, independently derived via .NET
  `DateTime.ToBinary` (epoch `0x489F7FF5F7B58000`, FixtureClock
  `0x48DF4506E391C000` = Unix 1794960000).
- `a_zero_stamp_falls_back_to_the_file_mtime_on_the_disk_read` — the
  0-stamp file surfaces its mtime through `read_player_file`, the raw
  byte walk keeps 0, a nonzero stamp rides verbatim.

**Verification performed:** direct C# re-reads (`Player.cs:54680-54939`,
`:55060-55134`, `:55216-55339`, `:55686-55974`; `Item.cs:1211-1364`,
`:48660-49069`, `:49875-49939`; `IO/FileMetadata.cs` whole;
`IO/FavoritesFile.cs:1-60`; `IO/PlayerFileData.cs:1-75`), the g-lane
persist diffs at their fetched tips, and a full `git diff` re-read of
this pass's two-file change. No cargo/tests/rustfmt/graphify per the
lane constraint.