# 103 — The `.trplayer` sidecar FINAL layout (the v7-collision superset, writer v10)

**Status: THE MERGER CONTRACT.** Any merge of W4-g88 / W4-g89 / W4-g89b /
W4-g89c / W4-g90 / W4-g90b / W4-a6 / W4-g55 (and the future booster lanes)
keeps the writer and reader described here — `crates/tr-app/src/persist.rs`
on `task/W4-g88-life-fruit-use`. This doc is the token order, the token
reservations and the back-compat matrix; the module header of
`persist.rs` is its short form. g89d's addendum
(`122-persist-v9-addendum.md`, `4344bd50`) is the g89-lane view of the
same line-8 contract + the v10 crystal reservation; its §6 fold
receipt records how this superset satisfies it (the writer stayed 9
until the crystal tokens landed, then bumped v10 — §7's receipt).

**v10 crystal fold inputs (read before folding):** g90b `e76ee133`
(`origin/task/W4-g90-artisan-bread-use` tip lineage `bb78974a` →
`e76ee133` → `0316b444` → `73367e0a`) landed the four shimmer Player
flags (`usedAegisCrystal`/`usedArcaneCrystal`/`usedGalaxyPearl`/
`usedAmbrosia`, ctor `:56078-56083`), their consumers, the `.plr` 260+
unlock bools both halves and a `.trplayer` v7 bread+five-flag-line
block; a6 `f23385e5` (`origin/task/W4-a6-crystals-powerups`) landed ALL
six shimmer use arms (`powerups::use_shimmer_permanent`, 5337-5342) +
the consumers (regen/luck/boosters) with NO sidecar writer — its pearl
flag rides `LuckState.used_galaxy_pearl` directly. The brief's
research/135 (g90c fold-input) and research/138 (a6b reconcile) were
NOT pushed on either branch at fold time (verified `git ls-tree` on
both tips) — this fold proceeds on the landed code shapes and records
the keep-own/dup adjudication in §5a pending 138.

## 1. The collision (why this doc exists)

Seven pushed lane writers independently extended the post-dye block of the
`.trplayer` text sidecar (`persist::save`/`load`, the `.trwld`-adjacent
live-session save — NOT the vanilla `.plr`; that is `player_file.rs`,
each lane's own scope); a6's lane added use arms only (no writer):

| Lane | Commit | Version | What it appended after the dye line |
| --- | --- | --- | --- |
| W4-g89 | `920014cb` | 7 | ONE line: `usedAegisFruit` 0/1 (1 token) |
| W4-g88 (pre-reconciliation) | `f1eb9b60` | 7 | ONE line: `statLifeMax statLifeMax2` (2 tokens) |
| W4-g90 | `1f1b88f0` | 7 | ONE line: `ateArtisanBread` 0/1 (1 token) |
| W4-g55 | `d2210b91` (tip `cde865fc`) | 7 | the loadout block: index/share line (2 tokens) + 3 stored-set lines (41 tokens each) |
| W4-g89b | `9bf03ede` | 8 | widened g89's flag line to `usedAegisFruit usedGummyWorm` (2 tokens) |
| W4-g89c | `78766306` | 9 | widened it again to `usedAegisFruit usedGummyWorm usedAmbrosia` (3 tokens) |
| W4-g90b | `e76ee133` | 7 | the bread line + FIVE one-token shimmer flag lines (crystal/arcane/pearl/gummy/ambrosia, the `.plr` order `:54781-54786` minus the g89 fruit) |
| W4-a6 | `f23385e5` | — | NO sidecar writer — the six use arms + consumers only; the pearl flag rides `LuckState` |

A naive merge misparses one lane's file as another's (a 2-token loadout
index line reads as a life pair; a lone bread flag reads as aegis; a
3-token v9 unlock line reads as a life pair under the pre-fold-3
token-count reader — g89d's research/122 §2 records that corruption
class; g90b's six one-token v7 lines collapse into the aegis/bread
slots under the v7 walk). The superset below loads ALL of them.

## 2. The FINAL layout (the superset writer — version 10)

Fixed prefix, unchanged since v6 (line numbers 1-based):

| Line | Content | Since |
| --- | --- | --- |
| 1 | `10` (the version) | v10 |
| 2 | `x y hp` (3 tokens, f32 f32 i32) | v1 |
| 3 | 50 × `kind count` inventory pairs (v1 wrote ≤10) | v2 |
| 4 | `coins` (u64) | v3-era |
| 5 | `armor[0] armor[1] armor[2]` kinds (3 tokens) | v4 |
| 6 | `count (npcId killCount)*` bestiary pairs | v5 |
| 7 | 10 dye kinds (10 tokens) | v6 |

Post-dye block (lines 8+, the superset contract, in order):

| # | Line | Tokens | Field(s) | C# anchor (`.plr` write / read) |
| --- | --- | --- | --- | --- |
| 8 | the unlock line | 6 (+1 RESERVED) | `usedAegisFruit usedGummyWorm usedAmbrosia usedAegisCrystal usedArcaneCrystal usedGalaxyPearl` 0/1 | `Player.cs:54782`/`:55256`, `:54785`/`:55259`, `:54786`/`:55260`, `:54781`/`:55255`, `:54783`/`:55257`, `:54784`/`:55258` (ctor `:56079`/`:56082`/`:56083`/`:56078`/`:56080`/`:56081`) |
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
records) and appends the crystal trio after ambrosia in the vanilla
RELATIVE order among themselves (verified at the v10 fold against
`Player.cs:54776-54787`: `:54781` usedAegisCrystal → `:54782`
usedAegisFruit → `:54783` usedArcaneCrystal → `:54784` usedGalaxyPearl
→ `:54785` usedGummyWorm → `:54786` usedAmbrosia → `:54787`
downedDD2EventAnyDifficulty). The pre-v10 reservation table listed
pearl/arcane/crystal — those tokens were RESERVED (never written by any
pushed superset writer), so the v10 fold corrects the map to the
verified vanilla relative order crystal/arcane/pearl with no
back-compat cost. A future lane appends its flag at its reserved
position (token 7 DD2) and bumps the writer to **v11** — the reader
already skips unknown tokens:

| Token | Field | C# write / read | Status |
| --- | --- | --- | --- |
| 1 | `usedAegisFruit` | `:54782` / `:55256` | LIVE (g89, folded) |
| 2 | `usedGummyWorm` | `:54785` / `:55259` | LIVE (g89b, folded) |
| 3 | `usedAmbrosia` (5342, `pickSpeed -= 0.05f; tileSpeed += 0.05f; wallSpeed += 0.05f`, `:12861-12866`; use arm `:45417-45422`) | `:54786` / `:55260` | LIVE (g89c `78766306`, folded — the v9 bump) |
| 4 | `usedAegisCrystal` (5337, +0.2 lifeRegenTime, `:19113-19116`; use arm `:45387-45392`) | `:54781` / `:55255` | LIVE (g90b `e76ee133` flag + a6 `f23385e5` use arm, folded — the v10 bump) |
| 5 | `usedArcaneCrystal` (5339, mana delay −0.05 `:19430-19433` + regen +max/50 `:19453-19456`; use arm `:45399-45404`) | `:54783` / `:55257` | LIVE (g90b/a6, folded — v10) |
| 6 | `usedGalaxyPearl` (5340, +0.03 luck `:29142-29145`; use arm `:45405-45410`) | `:54784` / `:55258` | LIVE (g90b/a6, folded — v10; the Player field is the persistence owner, `update_luck` mirrors into `LuckState`) |
| 7 | `downedDD2EventAnyDifficulty` | `:54787` / `:55265` (gate 182) | RESERVED — v11 |

(The vanilla also writes a dropped `false` at `:54780` between bread and
aegisCrystal — the port does NOT reserve a slot for it; the sidecar is
its own format. The combat-book vol-two flag — item **5336**,
`NPC.combatBookVolumeTwoWasUsed`, `Player.cs:45337-45349` — is a WORLD
flag (`.wld` `:1407`/`:2416-2423`), NEVER a line-8 token; census
`ItemID.cs:12355-12369`: 5336 CombatBookVolumeTwo / 5337 AegisCrystal /
5338 AegisFruit / 5339 ArcaneCrystal / 5340 GalaxyPearl / 5341 GummyWorm
/ 5342 Ambrosia / 5343 PeddlersSatchel; LifeFruit is **1291**,
`ItemID.cs:4265`.) A lane activating a reserved token MUST: read its
position in `load` (the v8/v9/v10 positional arm), emit it in `save`'s
unlock line, bump the writer version (v11 for token 7), keep the
short-line back-reads, restore it in `app/mod.rs` beside the
aegis/gummy/ambrosia/crystal flags, add a round-trip witness and update
this table. Tokens beyond 7 need a new row here first.

## 3. The reader (tolerant, version-dispatched)

`persist::load` accepts versions 1-10 (1/2 migrate kinds). For the
post-dye block:

- **v8/v9/v10**: the FIRST non-empty post-dye line is POSITIONALLY the
  unlock line (g89b/g89c's contract, widened at v10): token 1 → aegis,
  token 2 → gummy, token 3 → ambrosia, token 4 → aegis-crystal, token 5
  → arcane-crystal, token 6 → galaxy-pearl; short lines back-read (a
  1-token line loads worm/ambrosia/crystals false, a 2-token line loads
  ambrosia/crystals false — g89b's v7 / g89c's v8 back-read shapes; a
  3-token line loads the crystal trio false — the v9→v10 back-read);
  token 7+ is skipped (reserved, v11).
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
bread. The superset writer emits all six flags positionally (v10), so
files written after the fold are unambiguous. A g89-file misread is
impossible (its flag IS slot 1).

**g90b's v7 six-line block** (`e76ee133`: bread + crystal/arcane/pearl/
gummy/ambrosia, each its own 1-token line) loads under the v7 walk with
a bounded misread: the FIRST 1-token line (bread) reads the aegis slot,
EVERY later 1-token line overwrites the bread slot (last wins — the
ambrosia line's value), and the crystal trio + worm keep false (the v7
walk has no slots for them). Witness
`g90bs_bread_plus_flags_v7_loads_with_the_bounded_misread`. g90b's own
reader maps its lines to its own flags; the superset's v10 writer emits
all six positionally, so the loss is confined to pre-fold g90b files.

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
| g90b bread+five-flag-lines | 7 | yes* | *bounded misread: bread→aegis slot, last 1-token line→bread, crystals/worm false (the ambiguity table above) |
| the pre-v10 superset (the v9 writer: 3-token unlock + pairs + bread + loadouts) | 9 | yes | unlocks read, crystal trio false, block behind unchanged (`the_v9_superset_still_loads_without_the_crystals`) |
| v9 3-token back-read | 10 | yes | aegis+gummy+ambrosia read, crystal trio false (`a_three_token_unlock_line_reads_under_v10`) |
| v10 6-token unlock-only | 10 | yes | all six tokens read positionally (`the_six_token_unlock_line_reads_under_v10`) |
| future 7-token unlock line | 10 | yes | tokens 1-6 live, 7+ skipped; pairs/bread behind read (`reserved_unlock_tokens_are_skipped`) |
| corrupt loadout index 9 | 7/8/9/10 | yes | clamps to 2 (`the_loadout_index_clamps_on_read`) |
| THE FULL SUPERSET round-trip | 10 | yes | every field survives, exact line order asserted (`the_superset_round_trips_the_full_block`) |
| full-state round-trip | 10 | yes | position/inv/coins/armor/dye/kills + all superset fields (`round_trips_player_state`) |
| bread-only round-trip | 10 | yes | flag survives; fresh player loads false (`the_artisan_bread_flag_round_trips`) |
| loadout-block round-trip | 10 | yes | index/share/kinds/hide/word per set (`w4_g55_the_loadout_block_round_trips`) |

Code-only mode: all witnesses WRITTEN, NOT RUN (the lane's constraint).

## 5. The carriers (fields restored, consumers stay with their lanes)

`App::new`'s sidecar restore (`app/mod.rs`, the `persist::load` block)
applies, in order: position (collision-gated) → life pair → mana pair →
aegis → gummy → ambrosia → aegis-crystal → arcane-crystal → galaxy-pearl
→ bread → loadouts + index + live share word →
**hp clamp** (so a saved 405-life character keeps its 405 hp) →
coins/armor/dye/inv.

| Field | Declared at | Consumer (owner lane) |
| --- | --- | --- |
| `Player.life_max_base` / `max_hp` | g88 (`f1eb9b60`) | the fruit/crystal gates (g88, landed) |
| `Player.mana_max_base` / `mana_max` | pre-existing / g88 crystals | the mana gate (g88, landed) |
| `Player.used_aegis_fruit` | g89's exact spot (`player.rs`, after `used_gummy_worm`) | +4 defense `:12853-12856` + the 5338 use arm (g89; a6's `use_shimmer_permanent` 5338 arm dups it — §5a) |
| `Player.used_gummy_worm` | pre-existing (S1-P-i35) | +3 fishing `:12857-12860` (landed); the 5341 use arm (g89b; a6 dups — §5a) |
| `Player.used_ambrosia` | g89c's exact spot (`player.rs`, after `used_aegis_fruit`) | the three-speed row `:12861-12866` + the 5342 use arm (g89c; g90b's booster row + a6's use arm dup — §5a) |
| `Player.used_aegis_crystal` | g90b's exact spot (`player.rs`, after `used_ambrosia`, `e76ee133`) | the +0.2 life-regen ramp `:19113-19116` (g90b's `RegenInputs.aegis_crystal` / a6's `extra_time` fold) + the 5337 use arm (g90b `use_shimmer_permanent_item` / a6 `use_shimmer_permanent` — §5a) |
| `Player.used_arcane_crystal` | g90b's exact spot (after `used_aegis_crystal`) | the mana rows `:19430-19433`/`:19453-19456` (g90b `RegenInputs.arcane_crystal` / a6 `RegenInputs.used_arcane_crystal`) + the 5339 use arm (§5a) |
| `Player.used_galaxy_pearl` | g90b's exact spot (after `used_arcane_crystal`) | the +0.03 luck `:29142-29145` via the `update_luck` mirror into `LuckState.used_galaxy_pearl` (g90b's shape kept); a6's 5340 use arm sets the LuckState field directly (§5a) |
| `Player.ate_artisan_bread` | pre-existing (reach formula) | +4 reach `craft.rs::tile_reach` `:35671` (landed); shop gate `Chest.cs:2958` + the 5326 use arm (g90) |
| `Player.loadouts` / `current_loadout_index` | g55's exact spot (`player.rs`, after `hide_visible_accessory`) | the swap machine + loadout UI (g55 Ph1/3/4) |
| `App.equip_share_flags` | g55's exact spot (`app/mod.rs`, after `dye_swap_counter`) | the share mechanic (g55 Ph4) |

### 5a. The use-arm dup adjudication (g88 keep-own; a6b research/138 pending)

The brief's a6b reconcile (research/138) was NOT pushed at fold time
(`origin/task/W4-a6-crystals-powerups` tip `8ed96ee5` carries no
research/138; `origin/task/W4-g90-artisan-bread-use` tip `73367e0a`
carries no research/135). This branch's position, recorded for the
merger:

- **g88 keeps its OWN use arms**: `life_fruit.rs` (1291 Life Fruit,
  `ItemCheck_UseLifeFruit` `:45290-45304` via the shared
  `permanent_upgrade_item_check` press-edge machine) and `crystals.rs`
  (29 Life Crystal `:45306-45320` / 109 Mana Crystal `:45274-45288`,
  the same machine). a6's `powerups.rs` carries parallel
  `use_life_fruit`/`use_life_crystal`/`use_mana_crystal` arms
  (`f23385e5`'s file, the `bb36e6ee`/`d46221b2` lineage) — a DUP of the
  same C# bodies on a different seam (`powerup_item_check` vs the
  `update_editing` permanent-item seam). **The merger drops a6's three
  dups per research/138 when it lands; until then g88's arms are the
  keep-own side** (they carry the achievement dispatch + the RAW-add
  witnesses + the sidecar round-trip the a6 arms lack).
- **The shimmer use arms (5337-5342) are NOT g88's**: this branch never
  wrote them (the carrier precedent — g89/g89b/g89c own 5338/5341/5342,
  g90b `0316b444`'s `use_shimmer_permanent_item` and a6 `f23385e5`'s
  `use_shimmer_permanent` both port the five non-g89 arms + dup the
  g89 ones). The g90b-vs-a6 keep-one is research/138's call; the
  sidecar contract is agnostic (it carries the FLAGS, whichever use arm
  sets them). The pearl carrier divergence (g90b's Player field +
  mirror vs a6's direct LuckState write) resolves to g90b's shape here
  because the sidecar needs a Player-owned field; at merge, a6's 5340
  arm must set `Player.used_galaxy_pearl` (or the mirror must run after
  the use) — flagged for 138.
- **The `.plr` halves** (g90b's 260+ unlock bools both halves +
  legacy-279 indices) stay g90b's scope — this branch's
  `player_file.rs` is untouched by the fold (the sidecar is a different
  file, §6 rule 3).

`loadout.rs` is byte-identical to g55's (blob `a647ae55`), so the merge
dedupes it cleanly. `persist::save`'s signature carries one extra arg
(`equip_share_flags: u32`, g55 Ph2's shape); all call sites pass it.

## 6. Merge guidance

1. Keep THIS `persist.rs` (writer v10 + the tolerant reader) over any
   lane's single-purpose version — it is the strict superset. g89d's
   research/122 §4 recipe (g89 v9 base whole, then g88 pairs/clamps,
   then g90 flag, then crystal tokens) resolves to this same reader:
   the superset already carries every layer (the crystal tokens landed
   at v10); at merge, g89's/g90b's/a6's use arms/booster rows/`.plr`
   halves land beside the carriers (the dup adjudication §5a).
2. Keep the field declarations at the spots in §5 — every lane declared
   its fields at the same anchor, so the merge dedupes without semantic
   conflicts.
3. A lane's `.plr` (`player_file.rs`) changes are INDEPENDENT of this
   sidecar contract (different file, vanilla format); merge those per
   their own gates (256 bread, 260 unlocks, 262 loadouts, 322 favorited).
4. New unlock flags: §2a's reserved tokens — append, never renumber;
   the crystal tokens 4/5/6 LANDED at v10 with the 3-token v9 back-read
   (research/122 §3.1/§4.4, §7's receipt); token 7 (DD2) bumps v11 with
   the 6-token v10 back-read. World globals (combat-book
   vol-two 5336, peddler's satchel 5343) NEVER ride line 8 — they are
   `.wld` flags, a separate lane (g90b `bb78974a`'s `.trwld` v18 tail).
5. New post-dye lines: append AFTER the loadout block, give the line a
   self-identifying shape (token count distinct from 1/2/41, or a
   sentinel), extend the reader's shape walk, add a back-compat row here
   and a predecessor witness.

## 7. The v10 crystal fold receipt (this branch)

Folded on `task/W4-g88-life-fruit-use` atop the v9 superset (`6998a4ec`).
Inputs read: g90b `e76ee133` (the four Player flags + consumers + the
`.plr` halves + the v7 five-line block) and a6 `f23385e5` (the six use
arms + consumers, no sidecar). research/135/138 NOT pushed at fold time
(§5a records the keep-own adjudication pending 138).

- **The writer bumps 9 → 10**: the unlock line widens to SIX tokens —
  `usedAegisFruit usedGummyWorm usedAmbrosia usedAegisCrystal
  usedArcaneCrystal usedGalaxyPearl`. Tokens 1-3 keep g89c's fruit-first
  positions verbatim; tokens 4/5/6 append in the verified vanilla
  relative order (`Player.cs:54781`/`:54783`/`:54784` — read at
  `:54776-54787`). The pre-v10 reservation map's pearl/arcane/crystal
  listing is corrected to crystal/arcane/pearl (the tokens were never
  written, so no back-compat cost).
- **The reader**: the positional arm extends to `"8" | "9" | "10"` with
  `unlock.get(3/4/5)` → the crystal trio; the v9 three-token line
  back-reads all three false (witness
  `a_three_token_unlock_line_reads_under_v10` +
  `the_v9_superset_still_loads_without_the_crystals`); token 7+ stays
  skipped (`reserved_unlock_tokens_are_skipped`, re-scoped to v10).
- **The carriers**: `Player.used_aegis_crystal`/`used_arcane_crystal`/
  `used_galaxy_pearl` declared at g90b's exact spots (after
  `used_ambrosia`, ctor `:56078`/`:56080`/`:56081` false), restored in
  `app/mod.rs` beside the aegis/gummy/ambrosia triple BEFORE the bread
  line; the pearl mirror (`update_luck` seeds
  `LuckState.used_galaxy_pearl`, g90b's shape) lands with its witness
  (`w4_g88_v10_the_galaxy_pearl_feeds_the_luck_recalculation`). The
  consumers (regen rows, booster rows, use arms) stay g90b/a6's — this
  branch carries the flags only (the carrier precedent, §5).
- **Witnesses added/updated** (`persist.rs::tests`, written not run —
  code-only): `the_v9_superset_still_loads_without_the_crystals`,
  `a_three_token_unlock_line_reads_under_v10`,
  `the_six_token_unlock_line_reads_under_v10`,
  `g90bs_bread_plus_flags_v7_loads_with_the_bounded_misread`; the full
  round-trip now asserts the `10\n` header + the `1 1 1 1 1 1` unlock
  line; `round_trips_player_state` carries all six flags; the v5/v6
  back-compat assertions extend to the crystal trio.
- **Cross-refs**: research/122 §7 (the g89-addendum fold receipt),
  §5a (the a6 dup adjudication pending research/138), the brief's
  research/135 (g90c fold-input, unpushed).