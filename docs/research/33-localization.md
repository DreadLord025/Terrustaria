# 33 — Localization

> Wave 3 · researched 2026-11-12 · sources: decompiled (`Terraria/Localization/*`, `Terraria/Lang.cs`) + crates (`tr-core/src/localization/*`, `tr-app/src/lang.rs`) + ROADMAP/ATLAS/PORT_MAP

## Scope

| C# file | lines | responsibility |
| --- | --- | --- |
| `decompiled/Terraria/Localization/LanguageManager.cs` | 519 | the string table: load, categories, variations, copy commands, fallback chain |
| `decompiled/Terraria/Localization/LocalizedText.cs` | 214 | one key/value pair, `Value`/`UnformattedValue`/`EnglishValue`/`HasValue`, `Format` overloads |
| `decompiled/Terraria/Localization/VariableText.cs` | 123 | the `{Name}` / `{?Cond}` / `{!Cond}` substitution parser (`VariableText.TryCreate`) |
| `decompiled/Terraria/Localization/GameCulture.cs` | 138 | the 12 cultures, their legacy ids and `CultureInfo` names |
| `decompiled/Terraria/Localization/Language.cs` | 97 | the static facade (`Language.GetTextValue` overloads, `Exists`, `FindAll`, `SelectRandom`) |
| `decompiled/Terraria/Localization/NetworkText.cs` | 163 | netcode carrier for a localized key (not ported — module 17) |
| `decompiled/Terraria/Lang.cs` | 1230 | the name caches (`ItemName`/`NPCName`/`ProjectileName`/`BuffName`/`EmojiName`/`Prefix`), the 10 legacy index tables, the map-legend table, the ~60 global substitutions |

The subsystem owns every user-visible string in the game. The C# keeps the strings as
**embedded JSON resources** named `Terraria.Localization.Content.<culture>[.<Group>].json`
— 12 cultures × 7 groups = **84 files**, a flat two-level map `Category → { Name → text }`
(the flat key is `Category.Name`). `LanguageManager` merges the seven group files of one
culture into one `Dictionary<string, LocalizedText>`, tracks a per-category key list
(`_categoryGroupedKeys`, used by `IndexedFromCategory`/`RandomFromCategory`), a
variation table (`Key$Variant`) and the copy-command expansion (`{$Category.Key}`).

Rust side: `crates/tr-core/src/localization/` (`mod.rs` 613, `culture.rs` 154, `json.rs` 136,
`format.rs` 238) = the `LanguageManager`/`LocalizedText`/`VariableText`/`GameCulture` port;
`crates/tr-app/src/lang.rs` (236) = the process-global runtime facade (`Lang`/`Language`
statics), the source probe and the menu/tooltip accessors.

## Key entities

- `Terraria/Localization/LanguageManager.cs::LanguageManager` (L21) — the table holder:
  `_localizedTexts` (L25), `_categoryGroupedKeys` (L27), `_textVariations` (L29),
  `_fallbackCulture` (L31), `VariationSeparatorSign = '$'` (L35), `ActiveCulture` (L37).
- `LanguageManager.cs::LoadLanguageFromFileTextJson` (L273) — the JSON merge; builds the
  flat `Category.Name` key and appends to the category list **only for new keys**.
- `LanguageManager.cs::UpdateTextValue` (L294) — the `$` split → `AddVariant` (L491);
  returns `true` for a known key (the "override" path).
- `LanguageManager.cs::ReloadLanguage` (L101) — **the fallback chain**:
  `if (ActiveCulture != _fallbackCulture) { SetAllTextValuesToKeys(); if (target != _fallbackCulture) LoadLanguage(_fallbackCulture); } LoadLanguage(target);`
- `LanguageManager.cs::ProcessCopyCommandsInTexts` (L157) — the `{\$(\w+\.\w+)}` regex
  expansion, max 100 iterations, `Invariant.Assert` on convergence.
- `LanguageManager.cs::GetTextValue` overloads (L446 / L455 / L464 / L473 / L482) —
  `key`, `key+arg0`, `key+arg0+arg1`, `key+arg0+arg1+arg2`, `key+params object[]`;
  each one is `LocalizedText.Format` → `string.Format`, or the key itself when missing.
- `LanguageManager.cs::GetText` (L437) — unknown key ⇒ inserts `LocalizedText(key, key)`.
- `LanguageManager.cs::SetFallbackCulture` (L510) / `SetLanguage` (L91) — the culture switch.
- `LocalizedText.cs::LocalizedText` (L7) — `Value` (L17, VariableText-formatted),
  `UnformattedValue` (L29), `EnglishValue` (L31), `HasValue` (L33 = `EnglishValue != Key`),
  `SetValue` (L53, records `EnglishValue` only while the **default** culture is active).
- `VariableText.cs::VariableText.TryCreate` (L40) — regex `{(\?!?)?([a-zA-Z][\w\.]*)}` (L121);
  `TryFormat` (L101) returns false when a variable or a condition is missing.
- `GameCulture.cs::GameCulture` (L7) — the 12 cultures, `LegacyId` 1..12 (L72-118),
  `DefaultCulture` = English (L120), `FromLegacyId` (L49), `FromName` (L62).
- `Lang.cs::FillNameCacheArray` (L450) — the name-table builder: `ItemName.<ItemID const>`,
  `ProjectileName.<ProjectileID const>`, `NPCName.<NPCID const>`, `BuffName.<BuffID const>`,
  `EmojiName.<EmoteID const>`, `Prefix`; the const **field name** is the key suffix.
- `Lang.cs::InitializeLegacyLocalization` (L472) — the ten legacy index tables:
  `LegacyWorldGen` 94, `LegacyDialog` 392, `LegacyMenu` 254, `LegacyTooltip` 62,
  `LegacyMultiplayer` 27, `LegacyMisc` 106, `LegacyInterface` 126, `LegacyChestType` 52,
  `LegacyDresserType` 65, `LegacyChestType2` 38 (array sizes L1209-1218).
- `Lang.cs::InitGlobalSubstitutions` (L100) — ~60 named globals (`PlayerName`, `WorldName`,
  `Nurse`, `Guide`, `InputTrigger_*`, …).
- `Lang.cs::BuildMapAtlas` (L554-1010) — the **tile/map-legend** table: map tile → item
  name or `MapObject.*` key (79 `MapObject` keys in en-US).
- `tr-core/src/localization/mod.rs::LanguageManager` (L120) — the port of the table:
  `localized_texts` (L121), `category_grouped_keys` (L122), `text_variations` (L123),
  `fallback_culture` (L124), `active_culture` (L127), `substitutions` (L128), `sources` (L131).
- `tr-core/src/localization/mod.rs::reload_language` (L203) — the fallback chain;
  `load_language` (L215), `load_files_for_culture` (L224), `process_copy_commands_in_texts` (L282).
- `tr-core/src/localization/mod.rs::get_text_value` (L323) / `get_text_value_args` (L331) /
  `get_text` (L314) / `exists` (L308) / `category_size` (L303) / `try_get_variation` (L273) /
  `indexed_from_category` (L360) / `random_from_category` (L340).
- `tr-core/src/localization/culture.rs::GameCulture` (L5) + `CULTURES` (L21) — the 12 cultures,
  `name()` (L41), `legacy_id()` (L59), `from_legacy_id` (L78), `matches_resource` (L125).
- `tr-core/src/localization/json.rs::load_language_from_file_text_json` (L14) — the JSON merge
  + `strip_trailing_commas` (L49, the Newtonsoft trailing-comma shim) +
  `expand_copy_commands` (L93).
- `tr-core/src/localization/format.rs::VariableText` (L63) + `format_net` (L183).
- `tr-app/src/lang.rs::init` (L67) — the source probe + `set_language`;
  `source_dirs` (L37), `text` (L122), `text_args` (L127), `menu` (L141), `tip` (L148),
  `inter` (L154), `item_name` (L161), `game_ui_fmt` (L133).

## Dependencies

- Calls into: `Terraria.Localization.*` (the table), `Terraria/Lang.cs` (the name caches and
  the global substitutions), `Terraria.ID.{ItemID,NPCID,BuffID,EmoteID,ProjectileID,PrefixID}`
  (the const names that form the key suffixes), `Terraria/MapHelper.cs` (`TileToLookup` for
  the map legend), `Main.rand` (the category random).
- Called by: everything user-visible — `Main.cs` menu/option draws, `ItemSlot`/`Item` tooltips,
  `NPC` dialogue, the bestiary (`BestiaryEntry.cs:46` builds
  `Bestiary_FlavorText.npc_<NPCName key>`), the achievements menu, the shop, the chat.
- Shared data structures: the flat key string `Category.Name` is the shared currency between
  the table, the name caches and the legacy index tables — the port's `itemdata::NAMES`
  (6196 entries) is the item half of it, `ui/housing.rs` the NPC half.
- Rust call sites: 482 hits for `lang::text` / `game_ui_fmt` / `lang::menu` / `lang::tip` /
  `lang::inter` across `tr-app` — the runtime is genuinely wired, not a stub.

## Port status

| C# subsystem | Status | Rust location | Evidence / notes |
| --- | --- | --- | --- |
| `LanguageManager` table + JSON merge | PORTED | `tr-core/src/localization/mod.rs::LanguageManager`, `json.rs::load_language_from_file_text_json` | Same flat key, same "later file overrides earlier", same category-list growth only for new keys (`json.rs:31`). The Newtonsoft trailing-comma shim (`json.rs:49`) is an addition the C# gets free from Newtonsoft. |
| 12 cultures + legacy ids | PORTED | `culture.rs::GameCulture`, `CULTURES` | `GameCulture.cs:72-118` → `culture.rs:41-74`: identical names (`en-US`…`zh-Hant`) and ids (1..12); `from_legacy_id` clamps `<1` to 1 and falls back to English (`GameCulture.cs:49-60` → `culture.rs:78`). |
| `GetTextValue` overload set | PORTED | `mod.rs::get_text_value` (L323) / `get_text_value_args` (L331) | C# has 5 overloads (L446-489); Rust folds the arity variants into one `&[String]`. The missing-key answer is the key itself in both (`LanguageManager.cs:452`). |
| Unknown-key placeholder | PARTIAL | `mod.rs::get_text` (L314) | Value matches (`key → key`), but `HasValue` diverges: C# `GetText` creates `LocalizedText(key,key)` whose `SetValue` records `EnglishValue = key` under the default culture ⇒ `HasValue == false` (`LocalizedText.cs:33`); Rust's `LocalizedText::new(key, key, false)` leaves `english_value = ""` ⇒ `has_value() == true` (`mod.rs:81`). No vanilla consumer of `LocalizedText.HasValue` was found, so the impact is theoretical. |
| Fallback chain (en-US → target) | PORTED | `mod.rs::reload_language` (L203) | Verbatim structure of `LanguageManager.cs:101-116`, incl. the `SetAllTextValuesToKeys` reset and the "load the fallback culture first" step. Pinned by `switching_to_a_culture_reloads_fallback_then_target` (`mod.rs:534`). The two keys every non-English culture lacks (`CreditsRollCategory_Playtesting.78/79`) resolve only through this chain — the port inherits the behaviour. |
| `$` variations | PORTED | `mod.rs::update_text_value` (L249) / `try_get_variation` (L273) / `add_variant` (L265) | Same split rule and same "a `$` key is not a normal text" behaviour (`LanguageManager.cs:296-301`). The 1274-1468 `ItemName.X$Gender` variants in the Latin-script cultures are loaded. |
| `{$Category.Key}` copy commands | PORTED | `json.rs::expand_copy_commands` (L93), `mod.rs::process_copy_commands_in_texts` (L282) | Hand-rolled scan instead of the C# `Regex`, same `\w+\.\w+` shape, same 100-iteration cap. The C#'s `Invariant.Assert` on convergence is not reproduced (a non-converging file loops out silently). |
| `VariableText` `{Name}` / `{?Cond}` | PORTED | `format.rs::VariableText::try_create` (L80) / `try_format` (L126) | Same regex shape `{(\?!?)?([a-zA-Z][\w\.]*)}` (`VariableText.cs:121`); a missing variable ⇒ `None` ⇒ the original text is shown (`VariableText.cs:84-90` → `format.rs:138`). |
| `string.Format` clone | PARTIAL | `format.rs::format_net` (L183) | Handles `{0}`, `{{`/`}}` escapes and ignores the format specifier; an out-of-range index **keeps the token** where C# throws `FormatException` (`LanguageManager.cs:455` → `string.Format`). Deliberate robustness deviation, documented in the code comment (L178-182). |
| Global substitutions (`Lang.InitGlobalSubstitutions`) | MISSING | `mod.rs::set_substitution` (L181) exists, **no production caller** | The C# registers ~60 named globals (`Lang.cs:100-167`): `PlayerName`, `WorldName`, `Nurse`, `Guide`, `InputTrigger_*`, … **204 en-US values** contain such a token. Rust registers substitutions only in unit tests (`mod.rs:507`), so every `{PlayerName}`-style value renders with the literal token. The vanilla call site of `InitGlobalSubstitutions` was not traced in this pass. |
| `LanguageManager.SelectRandom` / `FindAll(Regex)` | PARTIAL | `mod.rs::find_all` (L380, closure filter only), `random_from_category` (L340, explicit index) | `Language.cs:71` `FindAll(Regex)` has no Rust counterpart (closure filter only). `RandomFromCategory` takes an explicit index instead of `Main.rand` — deterministic-friendly, but the call shape differs. |
| CSV path (`LoadLanguageFromFileTextCsv`, L229) | MISSING (out of scope) | — | Mod-only path; the module doc states it explicitly (`mod.rs:9-10`). |
| Content sources (`UseSources`/`LoadFromContentSources`, L184/190) | MISSING (out of scope) | — | Mod-only; the `canCreateCategories: false` branch has no analogue. |
| `NetworkText` (163 lines) | MISSING | net.rs:186 documents the literal wire mode | Belongs to module 17 (netcode). |
| Name tables: **items** | PORTED (data-driven) | `tr-app/src/lang.rs::item_name` (L161) + `itemdata::NAMES` (L6320, 6196 entries) | `itemdata::NAMES` is **generated** by `tools/GenItemData/gen.py::emit_names` (L2369-2372) from `ID/ItemID.cs` const names — exactly the C#'s `FillNameCacheArray<ItemID,short>("ItemName",…)` key shape (`Lang.cs:515` + `Lang.cs:463`). The key is data-driven, not hardcoded. Fallback to the baked English name when the key is absent (`lang.rs:164-167`). |
| Name tables: **NPCs** | PARTIAL (hardcoded) | `tr-app/src/ui/housing.rs:227-247` (21 keys), `tr-app/src/npc.rs::SlimeVariant::name` (L803) | C# builds the full `NPCName` cache from `NPCID` consts (`Lang.cs:517`, 703 en-US keys) + the negative cache (L521-536). The port hardcodes 21 town-NPC keys and returns **hardcoded English** for the slimes (`"Blue Slime"`), i.e. `NPCName.*` is *not* data-driven and not localized for the 682 other NPCs. |
| Name tables: **tiles** | MISSING | — | The C# has **no** tile-name category; the tile display names live in the map-legend table (`Lang.cs:554-1010`) fed by `MapObject.*` (79 en-US keys) and item names. The port has **no map legend at all** (0 hits for `MapObject`/`map_legend`) — so those 79 keys are unreachable and the minimap legend is not localized. |
| Other name tables (`ProjectileName` 1036, `BuffName` 352, `EmojiName` 151, `ItemTooltip` 2790, `CommonItemTooltip` 78) | MISSING | — | 0 hits for `ProjectileName`/`BuffName`; `EmojiName` appears only in a comment (`emotes.rs:10` "lands with M6-LOCALIZE") and the port uses a hardcoded `EMOTE_NAMES` table (`emotes.rs:19`). `ItemTooltip`/`CommonItemTooltip` are only touched in comments — the per-item tooltip text table is not localized. |
| Legacy index tables | PARTIAL | `lang.rs::menu` (L141), `tip` (L148), `inter` (L154) | 3 of 10 ported. `LegacyMisc` has 2 hits, and `LegacyMultiplayer`, `LegacyDialog`, `LegacyWorldGen`, `LegacyChestType`, `LegacyDresserType`, `LegacyChestType2` have **zero** Rust accessors — those strings cannot be localized at all today (`Lang.cs:477-513`). |
| `Lang.GetPrefixedItemName` / `Lang.prefix` | MISSING | — | `CombineFormat.Prefix` has 0 hits; the prefix display names + the `Gender` variation are absent (ATLAS's M3-PREFIX row already lists them as "out: the localization seam"). |
| Source discovery (the data location) | PORTED | `tr-app/src/lang.rs::source_dirs` (L37) + `init` (L67) | Probe order: `TR_LOCALE` → `<content dir>/../decompiled` → `./decompiled` → `CARGO_MANIFEST_DIR/../../decompiled` → `<exe dir>/decompiled`. **The JSON data is NOT under `assets/` and NOT in the crate**: the 84 `Terraria.Localization.Content.*.json` files live in the gitignored **`decompiled/`** tree (junction → `Terraria-Rust\decompiled`); `assets/Content` holds only the `.xnb` content. `tr-core`'s test reads `CARGO_MANIFEST_DIR/../../decompiled` (`mod.rs:575`). So AGENTS.md's warning applies to the **`decompiled` junction** (all three are junctions; `assets` is the wrong one for localization). A worktree without it ⇒ 0 keys loaded ⇒ the "unknown key" failures. |
| Culture switch at startup + menu | PORTED | `tr-app/src/main.rs:130-138`, `menu/actions.rs:213`, `settings.rs:11-16` | `TR_LANG` mirrors `-lang` (`LaunchInitializer.cs:120`), else `settings.language` (the vanilla `"Language"` key, `Main.cs:4099`/`:4345`); the menu click writes the culture name back (`actions.rs:213`). |
| Font coverage for the localized glyphs | PORTED (claimed) | the `Mouse_Text.xnb` parse + the `covers_the_scripts_the_localizations_use` test | The ATLAS claims 159 pages / 38 636 glyphs (Cyrillic, CJK, Kana, Hangul); the test exists but the 159/38 636 figures were **not re-verified** here. |

## Coverage table (the shipped resources)

Key counts are measured from the 84 shipped JSON files (the exact input the C# embeds):
`texts` = the flat keys that become `LocalizedText` entries, `variations` = the `$`-keys
that become `TryGetVariation` entries. Rust loads the **same files**, so the Rust column is
identical whenever the `decompiled/` junction is present — and 0 without it.

| language | C# keys (texts) | C# variations | Rust keys (texts) | Rust variations | delta | verdict |
| --- | --- | --- | --- | --- | --- | --- |
| en-US | 20 279 | 0 | 20 279 | 0 | 0 | PASS |
| de-DE | 20 277 | 1 468 | 20 277 | 1 468 | 0 | PASS |
| it-IT | 20 277 | 1 274 | 20 277 | 1 274 | 0 | PASS |
| fr-FR | 20 277 | 1 274 | 20 277 | 1 274 | 0 | PASS |
| es-ES | 20 277 | 1 274 | 20 277 | 1 274 | 0 | PASS |
| ru-RU | 20 277 | 1 274 | 20 277 | 1 274 | 0 | PASS |
| zh-Hans | 20 277 | 0 | 20 277 | 0 | 0 | PASS |
| pt-BR | 20 277 | 1 274 | 20 277 | 1 274 | 0 | PASS |
| pl-PL | 20 277 | 1 371 | 20 277 | 1 371 | 0 | PASS |
| ja-JP | 20 277 | 0 | 20 277 | 0 | 0 | PASS |
| ko-KR | 20 277 | 0 | 20 277 | 0 | 0 | PASS |
| zh-Hant | 20 277 | 0 | 20 277 | 0 | 0 | PASS |

- en-US per group file (the 7 files): `Game` 35 cats / 2 337 keys, `Items` 8 / 9 209,
  base `` 54 / 3 699, `Legacy` 10 / 1 216, `NPCs` 2 / 730, `Projectiles` 1 / 1 036,
  `Town` 146 / 2 052 — 255 categories in total.
- Every non-English culture is **missing 2 en-US keys** (`CreditsRollCategory_Playtesting.78`
  and `.79`) and adds its own extra keys; the Latin-script cultures add the
  `ItemName.X$Gender` variations, the CJK ones add none. The missing pair survives only
  through the fallback chain — the port implements that chain correctly.
- The 7 Latin-script cultures (de-DE, it-IT, fr-FR, es-ES, ru-RU, pt-BR, pl-PL) have
  1 274-1 468 **extra** keys relative to en-US, all `$`
  variations (e.g. `ItemName.AbigailsFlower$Gender`), so they are *not* extra texts.
- **Consumption** coverage is the real gap: the table holds all 20 279 keys, but the port
  only has accessors for 3 of the 10 legacy tables and no accessor at all for
  `ProjectileName`, `BuffName`, `EmojiName`, `ItemTooltip`, `CommonItemTooltip`,
  `MapObject` and `Prefix`.

## Tests

- `crates/tr-core/src/localization/mod.rs` — 10 tests: `json_creates_categories_and_english_values`
  (L402), `later_files_override_earlier_values` (L421), `dollar_keys_become_variations` (L438),
  `copy_commands_expand_into_the_referenced_value` (L458), `format_substitutes_arguments_and_keeps_escapes`
  (L469), `variable_text_uses_substitutions_and_conditions` (L498),
  `unknown_key_becomes_its_own_placeholder_text` (L524),
  `switching_to_a_culture_reloads_fallback_then_target` (L534),
  `indexed_category_walks_file_order` (L554),
  `shipped_english_resources_match_the_game` (L574 — **skips silently when `decompiled/` is
  absent**, so it can pass green with zero keys loaded).
- `crates/tr-core/src/localization/culture.rs` — `cultures_match_the_original_ids_and_names` (L136).
- `crates/tr-app/src/lang.rs` — `russian_texts_and_item_names_resolve` (L191),
  `tip_and_inter_keys_follow_the_c_sharp_prefixes` (L223).
- `crates/tr-app/src/menu/mod.rs` — `language_screen_lists_the_twelve_vanilla_rows` (L2022),
  `language_initial_picker_differs_from_the_language_screen` (L2065).
- `crates/tr-app/src/ui/ingame_options.rs` — `the_general_preferences_are_localised_and_clickable` (L2005 area).
- `crates/tr-app/src/ui/tooltip.rs` — `tooltip_table_mirrors_the_draw_chain`,
  `m7_ui_6_the_tooltip_rows_follow_their_gates` (the M6-LOCALIZE-wire slice).
- A font test named `covers_the_scripts_the_localizations_use` probes Cyrillic / CJK / Kana /
  Hangul glyphs against the shipped `Mouse_Text.xnb` (ATLAS attributes it to slice 1).
- Gaps: **no** test asserts the key coverage against the C# en-US set (the ROADMAP S1-L-12
  exit criterion "key coverage test (no false 'unknown key')" is still open), and no test
  pins the per-culture variation counts.

## Deviations & risks

| # | Deviation | Severity | Admitted by ROADMAP/ATLAS? |
| --- | --- | --- | --- |
| D1 | **No global substitutions registered** — `Lang.InitGlobalSubstitutions` (`Lang.cs:100-167`, ~60 names) has no port; 204 en-US values contain such a token and render it literally (`{PlayerName}`). | behavioural | No — not in ATLAS §25 nor in ANALYSIS. |
| D2 | **No NPC-name table** — 21 hardcoded `NPCName.*` keys (`ui/housing.rs:227-247`) instead of the generated `NPCID`-const table (703 en-US keys), and the slime names are hardcoded English (`npc.rs:803`). | behavioural | Partially — ATLAS §25 claims the name tables are wired; only items are. |
| D3 | **No map legend / tile names** — `Lang.BuildMapAtlas` (`Lang.cs:554-1010`) and the 79 `MapObject.*` keys are absent; the minimap legend is not localized. | behavioural | No. |
| D4 | **Legacy tables 3 of 10** — `LegacyMultiplayer`, `LegacyDialog`, `LegacyWorldGen`, `LegacyChestType`, `LegacyDresserType`, `LegacyChestType2` have no accessor; `LegacyMisc` is nearly unused. ATLAS §25's "No menu/front-end labels remain to localize" is therefore **too strong**. | behavioural | No — the claim contradicts the code. |
| D5 | **Projectile / buff / emoji / per-item tooltip names not localized** (1 036 + 352 + 151 + 2 790 + 78 keys unreachable). `emotes.rs:10` still says the `EmojiName` table "lands with M6-LOCALIZE". | behavioural | Partially — the emote comment admits it; the rest does not. |
| D6 | **Prefix display names missing** (`Lang.prefix` + `CombineFormat.Prefix` + the `Gender` variation). | cosmetic | Yes — ATLAS's M3-PREFIX row lists them as out-of-scope "the localization seam". |
| D7 | **Failed-file semantics differ** — the C# `catch` **breaks** the file loop (`LanguageManager.cs:145-153`); Rust logs and **continues** (`mod.rs:235-237`). Only matters for a corrupted resource file. | behavioural | Yes — the module doc calls the C# `break` a "silent skip" (the comment at `mod.rs:222-223` states the opposite of the code, which continues). |
| D8 | **`HasValue` divergence on unknown keys** — C# `false` (EnglishValue = key under the default culture), Rust `true` (`english_value == ""`). | cosmetic | No. No vanilla consumer found. |
| D9 | **`format_net` keeps an out-of-range `{n}` token** where C# `string.Format` throws `FormatException`. | cosmetic | Yes — documented in `format.rs:178-182`. |
| D10 | **File registration order is filesystem order** — `lang::init` registers the 84 files in `std::fs::read_dir` order (`lang.rs:78`), while the C# uses the assembly's manifest-resource order. 256 category slots vs 255 unique categories in en-US ⇒ at least one category is split across two files, so its `IndexedFromCategory`/`RandomFromCategory` index mapping depends on the read order ⇒ **non-deterministic across filesystems**, and not the C#'s mapping either. | determinism | No. |
| D11 | **Data location** — the strings live in the gitignored `decompiled/` junction, not `assets/` and not in the crate. A fresh worktree without the junction silently degrades every string to its key (and `shipped_english_resources_match_the_game` skips instead of failing). | determinism (test hygiene) | Partially — AGENTS.md warns about the symlinks but names `assets`; the localization-relevant one is `decompiled`. ATLAS §25 says "the same retail-derived content model as `assets/Content`" — the JSON is in `decompiled/`, `assets/Content` holds the `.xnb` files. |
| D12 | `NetworkText` (163 lines) not ported; the netcode path resolves literals instead (`net.rs:186`, `chat.rs:243`). | behavioural | Out of scope (module 17). |
| D13 | The copy-command loop's C# `Invariant.Assert` on convergence is not reproduced (`mod.rs:282-300`). | cosmetic | No. |

## ROADMAP/ATLAS DONE rows re-verified (M6 section, `docs/ATLAS.md`)

| Claim (ATLAS) | Verdict | Evidence |
| --- | --- | --- |
| §25: "the `lang` runtime (84 shipped `Terraria.Localization.Content.*` resources, 12 cultures × 7 groups, loaded from `TR_LOCALE`/the `../decompiled` sibling of the content dir)" | **PASS** | 84 files on disk (12 × 7); `lang.rs::source_dirs` (L37) probes `TR_LOCALE` then the content dir's `../decompiled` sibling, exactly as described. |
| §25: "the `LanguageManager.SetLanguage` menu flow (`Main.cs:48519-48523`), `Settings.language` under the vanilla `"Language"` key (`Main.cs:4099`) applied at startup (`Main.cs:4345`, `TR_LANG` = the `-lang` flag)" | **PASS** | `main.rs:130-138` (`TR_LANG` → else `settings.language` → `lang::init`), `settings.rs:11-16` documents the vanilla `"Language"` key, `menu/actions.rs:213` writes the culture back on the menu click. |
| §25: "Slice 3 localized the death texts (`DeathText.*`, `Currency.*`, `Game.DroppedCoins`), slice 4 the front-end UIState screens … **No menu/front-end labels remain to localize**" | **PARTIAL** | The death/front-end rows are localized, but 6 of the 10 legacy tables have no accessor (D4) and 4 name tables are unreachable (D2/D3/D5) ⇒ strings do remain. |
| §25: font slice — "159 pages / 38 636 glyphs" | **PARTIAL (unverified)** | The `covers_the_scripts_the_localizations_use` test exists and probes the four scripts, but the page/glyph counts were not re-measured here. |
| `M6-LOCALIZE-wire` (ATLAS:1607, done 2026-11-08) — `Lang.tip`/`Lang.inter` accessors + the real tooltip stat rows, 11 tests | **PASS** | `lang.rs:141-156` has `menu`/`tip`/`inter`; `ui/tooltip.rs` carries the damage/crit/use-time/knockback/tool rows; the tooltips' tests exist. The row's own exclusions (the accessory layer) are respected. |
| `M6-LOCALIZE-wire-fonts` / `-2-settings-screens` / `-3-death-text` (named only in the ATLAS prose, no catalog row) | **PARTIAL** | The three slices' *effects* are visible (font parse, settings labels, death texts); they inherit D1/D4/D5, so "done" only holds for the strings they touch. |
| ATLAS:84 summary row — "menus + tooltips (M6-LOCALIZE-wire); sub-screen strings remain (BUG-18)" | **PASS** | Consistent with the code; BUG-18's `UI.Back` label is resolved through `lang::text` (`ui/emotes.rs`). |
| ROADMAP `S1-L-12` exit criterion — "key coverage test (no false 'unknown key')" | **FAIL (not met)** | No test compares the port's key set against the C# en-US set; `shipped_english_resources_match_the_game` *skips* when the resources are absent. This is the module's own open item. |

## Task seeds

- **S** — Register the vanilla global substitutions (`Lang.InitGlobalSubstitutions`, `Lang.cs:100-167`):
  add `lang::set_global_substitutions(&App)` at startup with at least `PlayerName`, `WorldName`,
  the 21 town-NPC names and the `InputTrigger_*` set; pins D1 and the 204 affected strings.
- **S** — Add the remaining legacy accessors (`lang::misc`, `lang::mp`, `lang::dialog`,
  `lang::gen`, `lang::chest_type`, `lang::dresser_type`, `lang::chest_type2`) mirroring
  `Lang.cs:477-513`; unblocks `Lang.mp[9]` (the chat roll line) and the death/respawn strings.
  Dependency: none.
- **S** — Sort the source registration by file name in `lang::init` (`lang.rs:78`) and add a
  test that pins the per-category key order; closes D10 (determinism).
- **S** — Turn `shipped_english_resources_match_the_game` into a real key-coverage test:
  fail (not skip) when the resources are missing, and assert the en-US key count against the
  shipped file set — the ROADMAP S1-L-12 exit criterion. Dependency: the `decompiled` junction.
- **M** — Data-driven NPC names: extend `tools/GenItemData` (or add `tools/GenNpcData`) to emit
  `npcdata::NAMES` from `ID/NPCID.cs` consts, add `lang::npc_name(net_id)` with the negative
  cache (`Lang.cs:521-536`), and replace the 21 hardcoded `ui/housing.rs` keys and the hardcoded
  `SlimeVariant::name` English strings. Dependency: S (substitutions) for the town-NPC names.
- **M** — Map legend: port `Lang.BuildMapAtlas` (`Lang.cs:554-1010`) into the minimap legend
  and resolve the 79 `MapObject.*` keys; closes D3.
- **M** — Remaining name tables: `ProjectileName` (1 036), `BuffName` (352), `EmojiName` (151),
  `ItemTooltip` (2 790) + `CommonItemTooltip` (78) accessors and call sites. Dependency: the
  generator work from the NPC-name task (same shape).
- **L** — Prefix display names: `Lang.prefix` + `CombineFormat.Prefix` + the `Gender` variation
  (`Lang.cs:225-239`); pins D6 and the M3-PREFIX "localization seam" follow-up.
- **L** — `NetworkText` (`Terraria/Localization/NetworkText.cs`, 163 lines) for the netcode
  path; hand over to module 17.
