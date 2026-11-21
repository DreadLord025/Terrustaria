# 103 — The `.trplayer` sidecar FINAL layout (the v7-collision superset, writer v9)

**Status: THE MERGER CONTRACT.** Any merge of W4-g88 / W4-g89 / W4-g89b /
W4-g89c / W4-g90 / W4-g55 (and the future booster lanes) keeps the writer
and reader described here — `crates/tr-app/src/persist.rs` on
`task/W4-g88-life-fruit-use`. This doc is the token order, the token
reservations and the back-compat matrix; the module header of
`persist.rs` is its short form. g89d's addendum
(`122-persist-v9-addendum.md`, `4344bd50`) is the g89-lane view of the
same v9 line-8 contract + the v10 crystal reservation; its §6 fold
receipt records how this superset satisfies it (the writer stays 9
until the crystal tokens land, then bumps v10).

## 1. The collision (why this doc exists)

Six pushed lane writers independently extended the post-dye block of the
`.trplayer` text sidecar (`persist::save`/`load`, the `.trwld`-adjacent
live-session save — NOT the vanilla `.plr`; that is `player_file.rs`,
each lane's own scope):

| Lane | Commit | Version | What it appended after the dye line |
| --- | --- | --- | --- |
| W4-g89 | `920014cb` | 7 | ONE line: `usedAegisFruit` 0/1 (1 token) |
| W4-g88 (pre-reconciliation) | `f1eb9b60` | 7 | ONE line: `statLifeMax statLifeMax2` (2 tokens) |
| W4-g90 | `1f1b88f0` | 7 | ONE line: `ateArtisanBread` 0/1 (1 token) |
| W4-g55 | `d2210b91` (tip `cde865fc`) | 7 | the loadout block: index/share line (2 tokens) + 3 stored-set lines (41 tokens each) |
| W4-g89b | `9bf03ede` | 8 | widened g89's flag line to `usedAegisFruit usedGummyWorm` (2 tokens) |
| W4-g89c | `78766306` | 9 | widened it again to `usedAegisFruit usedGummyWorm usedAmbrosia` (3 tokens) |

A naive merge misparses one lane's file as another's (a 2-token loadout
index line reads as a life pair; a lone bread flag reads as aegis; a
3-token v9 unlock line reads as a life pair under the pre-fold-3
token-count reader — g89d's research/122 §2 records that corruption
class). The superset below loads ALL of them.

## 2. The FINAL layout (the superset writer — version 9)

Fixed prefix, unchanged since v6 (line numbers 1-based):

| Line | Content | Since |
| --- | --- | --- |
| 1 | `9` (the version) | v9 |
| 2 | `x y hp` (3 tokens, f32 f32 i32) | v1 |
| 3 | 50 × `kind count` inventory pairs (v1 wrote ≤10) | v2 |
| 4 | `coins` (u64) | v3-era |
| 5 | `armor[0] armor[1] armor[2]` kinds (3 tokens) | v4 |
| 6 | `count (npcId killCount)*` bestiary pairs | v5 |
| 7 | 10 dye kinds (10 tokens) | v6 |

Post-dye block (lines 8+, the superset contract, in order):

| # | Line | Tokens | Field(s) | C# anchor (`.plr` write / read) |
| --- | --- | --- | --- | --- |
| 8 | the unlock line | 3 (+4 RESERVED) | `usedAegisFruit usedGummyWorm usedAmbrosia` 0/1 | `Player.cs:54782`/`:55256`, `:54785`/`:55259`, `:54786`/`:55260` (ctor `:56079`/`:56082`/`:56083`) |
| 9 | the life pair | 2 | `statLifeMax statLifeMax2` (permanent, effective); permanent half clamps 500 on read | `:54773` / `:55220-55224` |
| 10 | the mana pair | 2 | `statManaMax statManaMax2`; permanent half clamps 200 on read | `:54775` / `:55226-55229` |
| 11 | the bread flag | 1 | `ateArtisanBread` 0/1 | `:54779` / `:55245` (gate 256; ctor `:56077`) |
| 12 | the loadout index | 2 | `currentLoadoutIndex equipShareFlags`; index clamps 0..2 on read | `:54919` / `:55722` (`Utils.Clamp`, gate 262) |
| 13-15 | 3 stored-set lines | 41 each | per set: 20 armor kinds + 10 dye kinds + 10 hide 0/1 + the 30-bit share word (`EquipmentLoadout.favorited`) | `:54920-54923` / `:55723-55726`; `EquipmentLoadout.cs:31-45` Serialize / `:47-61` Deserialize |

Kinds-only, like the armor/dye lines: the sidecar carries no
prefix/stack/favorited-per-item — a restored slot is a single stack,
prefixless (`loadout_item`, `persist.rs`). The live share word
(`App.equip_share_flags`) is the equip-context `Item.favorited` bitmap
over armor[0..20] then dye[0..10] (`ItemSlot.cs:395-399`), NOT the grid
favourites.

### 2a. The unlock line's token map (the booster lanes)

The vanilla `.plr` writes the unlock family as consecutive bools
(`Player.cs:54780-54787`, reads `:55249`/`:55255-55260`/`:55265`). The
superset's unlock line keeps the fruit-first order g89 established
(vanilla interleaves crystal/fruit — the deviation research/122 §3.1
records) and reserves the crystal tokens from position 4 on. A future
lane appends its flag at its reserved position and bumps the writer to
**v10** (research/122 §3: crystal bools append as tokens 4/5/6, never
their own line, never a reorder of the first three) — the reader
already skips unknown tokens:

| Token | Field | C# write / read | Status |
| --- | --- | --- | --- |
| 1 | `usedAegisFruit` | `:54782` / `:55256` | LIVE (g89, folded) |
| 2 | `usedGummyWorm` | `:54785` / `:55259` | LIVE (g89b, folded) |
| 3 | `usedAmbrosia` (5342, `pickSpeed -= 0.05f; tileSpeed += 0.05f; wallSpeed += 0.05f`, `:12861-12866`; use arm `:45417-45422`) | `:54786` / `:55260` | LIVE (g89c `78766306`, folded — the v9 bump) |
| 4 | `usedGalaxyPearl` (5340, luck, `:29142`) | `:54784` / `:55258` | RESERVED — v10 (g90b/a6 crystal lane pending; `luck.rs::used_galaxy_pearl` exists in play, no sidecar writer pushed) |
| 5 | `usedArcaneCrystal` (5339) | `:54783` / `:55257` | RESERVED — v10 (g90b/a6 pending) |
| 6 | `usedAegisCrystal` (5337) | `:54781` / `:55255` | RESERVED — v10 (g90b/a6 pending) |
| 7 | `downedDD2EventAnyDifficulty` | `:54787` / `:55265` (gate 182) | RESERVED |

(The vanilla also writes a dropped `false` at `:54780` between bread and
aegisCrystal — the port does NOT reserve a slot for it; the sidecar is
its own format. The combat-book vol-two flag — item **5336**,
`NPC.combatBookVolumeTwoWasUsed`, `Player.cs:45337-45349` — is a WORLD
flag (`.wld` `:1407`/`:2416-2423`), NEVER a line-8 token; census
`ItemID.cs:12355-12369`: 5336 CombatBookVolumeTwo / 5337 AegisCrystal /
5338 AegisFruit / 5339 ArcaneCrystal / 5340 GalaxyPearl / 5341 GummyWorm
/ 5342 Ambrosia / 5343 PeddlersSatchel; LifeFruit is **1291**,
`ItemID.cs:4265`.) A lane activating a reserved token MUST: read its
position in `load` (the v8/v9 positional arm), emit it in `save`'s
unlock line, bump the writer version (v10 for tokens 4-6), keep the
short-line back-reads, restore it in `app/mod.rs` beside the
aegis/gummy/ambrosia triple, add a round-trip witness and update this
table. Tokens beyond 7 need a new row here first.

## 3. The reader (tolerant, version-dispatched)

`persist::load` accepts versions 1-9 (1/2 migrate kinds). For the
post-dye block:

- **v8/v9**: the FIRST non-empty post-dye line is POSITIONALLY the
  unlock line (g89b/g89c's contract): token 1 → aegis, token 2 → gummy,
  token 3 → ambrosia; short lines back-read (a 1-token line loads
  worm/ambrosia false, a 2-token line loads ambrosia false — g89b's v7 /
  g89c's v8 back-read shapes); tokens 4+ are skipped (reserved, v10).
  The remaining lines dispatch on shape:
  - 1 token → the bread flag;
  - 2 tokens → the first is the life pair (clamp 500), the second the
    mana pair (clamp 200);
  - the 2-token line DIRECTLY BEFORE the first 41-token line → the
    loadout index line (g55's block self-identifies by its set lines;
    index clamps 0..2);
  - 41 tokens → stored sets 0,1,2 in order (kinds → `loadout_item`,
    hide flags, share word);
  - any other shape → tolerated, defaults kept.
- **v7**: the same shape walk, except the FIRST 1-token line → aegis
  and the SECOND → bread (no positional unlock line — g89/g90's
  single-flag writers both land here).
- **v3-v6**: no post-dye block; every superset field keeps its fresh
  default (`None` maxima, false flags, loadout 0, three empty sets,
  share word 0).

### Known ambiguity (bounded, ends at the fold)

A lone 1-token flag line cannot self-identify: g90's bread-only v7 file
reads into the AEGIS slot (slot 1 wins; witness
`g90s_bread_only_v7_loads_into_the_first_flag_slot`). Both single-flag
lanes' own readers map their line to their own flag, so a pre-fold
bread-only file loaded by the superset reader grants aegis instead of
bread. The superset writer emits all three flags positionally (v9), so
files written after the fold are unambiguous. A g89-file misread is
impossible (its flag IS slot 1).

## 4. The back-compat matrix (witnesses in `persist.rs::tests`)

| File shape (writer) | Version | Loads? | Result |
| --- | --- | --- | --- |
| v3 (no armor) | 3 | yes | empty armor/dye, no kills, all superset fields default (`version_3_loads_with_empty_armor`) |
| v4 (armor, no kills/dye) | 4 | yes | armor rides; dye empty (`version_4_loads_with_empty_dye`) |
| v5 (kills, no dye) | 5 | yes | dye empty, `None`/false (`version_5_loads_with_empty_dye`) |
| v6 (dye, no block) | 6 | yes | all superset fields default (`version_6_loads_without_life_maxima`, `w4_g55_a_v6_sidecar_loads_empty_loadouts`) |
| g89 flag-only | 7 | yes | aegis reads; pairs `None`, bread false (`w4_g89s_flag_only_v7_still_loads`) |
| g88 life-only | 7 | yes | life pair reads; mana `None`, flags false (`the_life_only_v7_still_loads`) |
| g90 bread-only | 7 | yes* | *lone flag → aegis slot (the bounded ambiguity above) |
| g55 loadout-only | 7 | yes | index/share + sets read; pairs `None`, flags false (`g55s_loadout_only_v7_still_loads`) |
| g88⨉g89 3-line superset (`717e0640`) | 7 | yes | flag + both pairs; bread false (`the_three_line_superset_still_loads_without_bread`) |
| v7 clamp probe (900/900 pairs) | 7 | yes | permanent halves clamp 500/200 (`version_7_clamps_the_permanent_caps`) |
| g89b unlock-only | 8 | yes | both tokens read; rest default (`g89bs_unlock_only_v8_still_loads`) |
| g89b 1-token back-read | 8 | yes | aegis reads, gummy false (`a_one_token_unlock_line_reads_under_v8`) |
| the pre-v9 superset (fold 3's writer: 2-token unlock + pairs + bread + loadouts) | 8 | yes | unlocks read, ambrosia false, block behind unchanged (`the_v8_superset_still_loads_without_ambrosia`) |
| g89c unlock-only | 9 | yes | all three tokens read; rest default (`g89cs_unlock_only_v9_still_loads`) |
| g89c 2-token back-read | 9 | yes | aegis+gummy read, ambrosia false (`a_two_token_unlock_line_reads_under_v9`) |
| future 7-token unlock line | 9 | yes | tokens 1-3 live, 4+ skipped; pairs/bread behind read (`reserved_unlock_tokens_are_skipped`) |
| corrupt loadout index 9 | 7/8/9 | yes | clamps to 2 (`the_loadout_index_clamps_on_read`) |
| THE FULL SUPERSET round-trip | 9 | yes | every field survives, exact line order asserted (`the_superset_round_trips_the_full_block`) |
| full-state round-trip | 9 | yes | position/inv/coins/armor/dye/kills + all superset fields (`round_trips_player_state`) |
| bread-only round-trip | 9 | yes | flag survives; fresh player loads false (`the_artisan_bread_flag_round_trips`) |
| loadout-block round-trip | 9 | yes | index/share/kinds/hide/word per set (`w4_g55_the_loadout_block_round_trips`) |

Code-only mode: all witnesses WRITTEN, NOT RUN (the lane's constraint).

## 5. The carriers (fields restored, consumers stay with their lanes)

`App::new`'s sidecar restore (`app/mod.rs`, the `persist::load` block)
applies, in order: position (collision-gated) → life pair → mana pair →
aegis → gummy → ambrosia → bread → loadouts + index + live share word →
**hp clamp** (so a saved 405-life character keeps its 405 hp) →
coins/armor/dye/inv.

| Field | Declared at | Consumer (owner lane) |
| --- | --- | --- |
| `Player.life_max_base` / `max_hp` | g88 (`f1eb9b60`) | the fruit/crystal gates (g88, landed) |
| `Player.mana_max_base` / `mana_max` | pre-existing / g88 crystals | the mana gate (g88, landed) |
| `Player.used_aegis_fruit` | g89's exact spot (`player.rs`, after `used_gummy_worm`) | +4 defense `:12853-12856` + the 5338 use arm (g89) |
| `Player.used_gummy_worm` | pre-existing (S1-P-i35) | +3 fishing `:12857-12860` (landed); the 5341 use arm (g89b) |
| `Player.used_ambrosia` | g89c's exact spot (`player.rs`, after `used_aegis_fruit`) | the three-speed row `:12861-12866` + the 5342 use arm (g89c) |
| `Player.ate_artisan_bread` | pre-existing (reach formula) | +4 reach `craft.rs::tile_reach` `:35671` (landed); shop gate `Chest.cs:2958` + the 5326 use arm (g90) |
| `Player.loadouts` / `current_loadout_index` | g55's exact spot (`player.rs`, after `hide_visible_accessory`) | the swap machine + loadout UI (g55 Ph1/3/4) |
| `App.equip_share_flags` | g55's exact spot (`app/mod.rs`, after `dye_swap_counter`) | the share mechanic (g55 Ph4) |

`loadout.rs` is byte-identical to g55's (blob `a647ae55`), so the merge
dedupes it cleanly. `persist::save`'s signature carries one extra arg
(`equip_share_flags: u32`, g55 Ph2's shape); all call sites pass it.

## 6. Merge guidance

1. Keep THIS `persist.rs` (writer v9 + the tolerant reader) over any
   lane's single-purpose version — it is the strict superset. g89d's
   research/122 §4 recipe (g89 v9 base whole, then g88 pairs/clamps,
   then g90 flag, then crystal tokens) resolves to this same reader:
   the superset already carries every layer; at merge, g89's use
   arms/booster rows/`.plr` halves land beside the carriers.
2. Keep the field declarations at the spots in §5 — every lane declared
   its fields at the same anchor, so the merge dedupes without semantic
   conflicts.
3. A lane's `.plr` (`player_file.rs`) changes are INDEPENDENT of this
   sidecar contract (different file, vanilla format); merge those per
   their own gates (256 bread, 260 unlocks, 262 loadouts, 322 favorited).
4. New unlock flags: §2a's reserved tokens — append, never renumber;
   the crystal tokens 4/5/6 bump the writer to v10 with the 3-token v9
   back-read (research/122 §3.1/§4.4). World globals (combat-book
   vol-two 5336, peddler's satchel 5343) NEVER ride line 8 — they are
   `.wld` flags, a separate lane.
5. New post-dye lines: append AFTER the loadout block, give the line a
   self-identifying shape (token count distinct from 1/2/41, or a
   sentinel), extend the reader's shape walk, add a back-compat row here
   and a predecessor witness.