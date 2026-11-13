# 33 — Localization

> Wave 3 · researched 2026-11-12 · sources: decompiled + crates + ROADMAP/ANALYSIS/PORT_MAP

## Scope

`Terraria.Localization` in the decompiled tree is eight files under
`decompiled/Terraria/Localization/` — `LanguageManager.cs` (519 lines), `LocalizedText.cs` (214),
`VariableText.cs` (123), `NetworkText.cs` (163), `GameCulture.cs` (138), `Language.cs` (79),
`LanguageChangeCallback.cs` (3), `LanguageSearchFilter.cs` (3) — plus the legacy static facade
`decompiled/Terraria/Lang.cs` (1230 lines). The *content* is not under
`Terraria\Localization\Content\Language\`: in the retail build it ships as 84 **embedded resources**
named `Terraria.Localization.Content.<culture>[.<group>].json` (declared in
`decompiled/Terraria.csproj:133+`), which the decompiler dumped next to the csproj — 12 cultures ×
7 groups, 13 595 376 bytes total. The Rust side is `crates/tr-core/src/localization/` (`mod.rs` 613,
`culture.rs` 154, `format.rs` 238, `json.rs` 136) plus the runtime singleton
`crates/tr-app/src/lang.rs` (236). Responsibility: the key→text table, the culture set, the
`{Name}`/`{?Name}`/`{$Cat.Key}`/`{0}` substitution machinery, the culture fallback chain and the
twelve-row language picker.

## Key entities

- `Terraria/Localization/LanguageManager.cs::LanguageManager` (519) — the owner of
  `_localizedTexts` / `_categoryGroupedKeys` / `_textVariations`; `SetLanguage` (`:91`),
  `ReloadLanguage` (`:101`), `LoadLanguage` (`:118`), `LoadFilesForCulture` (`:127`),
  `ProcessCopyCommandsInTexts` (`:157`), `UpdateTextValue` (`:294`), `AddVariant` (`:491`),
  `GetTextValue` ×5 (`:446`–`:489`), `GetCategorySize` (`:46`), `SelectRandom` (`:392`),
  `RandomFromCategory` (`:413`), `IndexedFromCategory` (`:422`), `TryGetVariation` (`:500`).
- `Terraria/Localization/LocalizedText.cs::LocalizedText` (214) — `Key`, `Value` (`:17`, runs the
  `VariableText`), `UnformattedValue` (`:29`), `EnglishValue` (`:31`), `HasValue` (`:33`),
  `Format`/`FormatWith` (`:100`–`:169`), `EqualsCommand`/`ParseCommandPrefix` (`:176`–`:193`),
  `ToNetworkText` (`:136`).
- `Terraria/Localization/VariableText.cs::VariableText` (123) — `{Name}` variables, `{?Name}`/`{!Name}`
  conditions, regex `{(\?!?)?([a-zA-Z][\w\.]*)}` (`:121`); `TryFormat` (`:101`).
- `Terraria/Localization/GameCulture.cs::GameCulture` (138) — the twelve cultures with their
  `en-US`-style names and legacy ids 1..12 (`:69`–`:119`), `DefaultCulture` = English (`:120`),
  `FromLegacyId` (`:49`) / `FromName` (`:62`) / `FromCultureName` (`:40`).
- `Terraria/Localization/Language.cs::Language` (79) — the static facade: `GetText`, `GetTextValue`
  ×5, `GetTextValueWith` (`:40`), `Exists` (`:45`), `GetCategorySize` (`:50`), `FindAll` ×2,
  `SelectRandom` (`:65`), `RandomFromCategory` (`:70`), `TryGetVariation` (`:75`).
- `Terraria/Lang.cs::Lang` (1230) — the legacy caches (`menu`/`inter`/`tip`/`gen`/`misc`/`mp`/
  `chestType`/`dresserType`/`prefix`/`_itemNameCache`/`_npcNameCache`/`_projectileNameCache`/
  `_buffNameCache`/`_emojiNameCache`/`_mapLegendCache`), `InitializeLegacyLocalization` (`:472`),
  `FillNameCacheArray` (reflection over `ItemID`/`NPCID`/`ProjectileID`/`BuffID`/`EmoteID` static
  fields → key `ItemName.<FieldName>`), `GetItemNameValue` (`:220`), `GetNPCNameValue` (`:175`),
  `GetMapObjectName` (`:77`), `InitGlobalSubstitutions` (`:100`–`:151`, **65** registrations).
- `crates/tr-core/src/localization/mod.rs::LanguageManager` (613) — the port of the manager:
  `set_language` (`:196`), `reload_language` (`:203`), `load_language` (`:215`),
  `load_files_for_culture` (`:224`), `set_all_text_values_to_keys` (`:242`), `update_text_value`
  (`:249`), `process_copy_commands_in_texts` (`:282`), `get_text` (`:314`), `get_text_value` (`:323`),
  `get_text_value_args` (`:331`), `random_from_category` (`:340`), `indexed_from_category` (`:360`),
  `find_all` (`:380`), `try_get_variation` (`:273`), `category_size` (`:303`),
  `set_substitution`/`remove_substitution`/`substitution` (`:181`–`:193`).
- `crates/tr-core/src/localization/culture.rs::GameCulture` (154) — the twelve cultures, `CULTURES`
  (`:21`), `name` (`:41`), `legacy_id` (`:59`), `from_legacy_id` (`:78`), `language_key` (`:92`),
  `from_name` (`:110`), `from_culture_name` (`:119`), `matches_resource` (`:125`).
- `crates/tr-core/src/localization/format.rs::VariableText` + `format_net` (238) — the
  `{Name}`/`{?Name}` parser (`:80`, `:146`) and the `string.Format` clone (`:183`).
- `crates/tr-core/src/localization/json.rs::load_language_from_file_text_json` (136) — the JSON
  loader (`:14`), the Newtonsoft trailing-comma shim (`:49`), `expand_copy_commands` (`:93`).
- `crates/tr-app/src/lang.rs::init` (236) — the runtime: resource discovery (`:37` `source_dirs`),
  `add_source` per file (`:87`), `set_language` (`:111`), `text`/`text_args` (`:122`/`:127`),
  `menu`/`tip`/`inter` (`:141`/`:148`/`:154`), `item_name` (`:161`).

## Dependencies

- C# calls into: `LanguageManager` → `Utils.ReadEmbeddedResource`, `Newtonsoft.Json`,
  `CsvHelper`, `ReLogic.Content.IContentSource`; `Lang` → `LanguageManager`, `ItemID`/`NPCID`/
  `ProjectileID`/`BuffID`/`EmoteID` reflection, `NPC.GetFirstNPCNameOrNull`, `Main.worldName`.
- C# called by: `Main.cs` (360 references), `Lang.cs` (127), `WorldGen.cs` (46),
  `ItemDropRules/Conditions.cs` (41), `NPC.cs` (38), `IngameOptions.cs` (34),
  `UICharacterCreation.cs` (30), `UIWorldCreation.cs` (24) … 797 `Language.GetTextValue(` callsites
  in total.
- Rust: `tr-core::localization` → `serde_json` (workspace `preserve_order`) + `anyhow`;
  `tr-app::lang` → the manager singleton + `crate::itemdata::NAMES`; consumers: `menu/pages.rs`
  (the picker), `ui/tooltip.rs`, `ui/char_creation.rs`, `ui/keyboard.rs`, `ui/housing.rs`,
  `ui/emotes.rs`, `death.rs`, `chat.rs`, `app/frame.rs`.
- Shared data structures: the flat `HashMap<String, LocalizedText>` + per-category
  `Vec<String>` (order-sensitive for `RandomFromCategory`/`IndexedFromCategory`), the
  `text_variations` map (`$`-suffixed keys), the `Substitution` table.

## Port status

| C# subsystem | Status | Rust location | Evidence / notes |
| --- | --- | --- | --- |
| `LanguageManager` load/reload/fallback chain | PORTED | `tr-core/src/localization/mod.rs::reload_language` (`:203`) | 1:1 with `LanguageManager.cs:101-116`: reset-to-keys → load fallback (en-US) → load target. Test `switching_to_a_culture_reloads_fallback_then_target` (mod.rs:534). |
| `LocalizedText` value model | PORTED | `mod.rs::LocalizedText` (`:36`) | `Value`/`UnformattedValue`/`EnglishValue`/`HasValue` all present. Missing: `EqualsCommand`, `ParseCommandPrefix`, `ToNetworkText`, public `ConditionsMet` (LocalizedText.cs:70-193). |
| `VariableText` (`{Name}`, `{?Name}`, `{!Name}`) | PORTED | `format.rs::VariableText::try_create` (`:80`), `try_format` (`:126`) | Regex shape mirrored (`format.rs:146`). C# returns the *original* text when a variable is missing — the port does too (`mod.rs:103`). |
| `{$Category.Key}` copy commands | PORTED | `json.rs::expand_copy_commands` (`:93`), `mod.rs:282` | 2 184 tokens in en-US; 100-iteration cap kept (mod.rs:285). |
| `LanguageManager.GetTextValue` ×5 overloads | PARTIAL | `mod.rs::get_text_value` (`:323`), `get_text_value_args` (`:331`) | One `&[String]` shape instead of 5 overloads; **no culture-aware number formatting** (C# `string.Format` under `CurrentCulture`, LanguageManager.cs:95-96) and format specifiers are dropped (`format.rs:209-215`) — the content uses `{0:+0;-#}` in `ItemTooltip.PrefixArmorPenetration` and `ItemTooltip.PrefixTagDamage`. |
| `LanguageManager.GetText` placeholder insert | PORTED | `mod.rs::get_text` (`:314`) | Inserts key→key placeholder like C# `:437-444`. |
| `Language.Exists` / `GetCategorySize` | PORTED | `mod.rs::exists` (`:308`), `category_size` (`:303`) | Not surfaced in `tr-app` (no CLI port). |
| `Language.FindAll` / `SelectRandom` / `RandomFromCategory` | PARTIAL | `mod.rs::find_all` (`:380`), `random_from_category` (`:340`), `indexed_from_category` (`:360`) | `SelectRandom` is **missing**; `RandomFromCategory` takes an explicit index instead of `Main.rand` (deterministic, documented at mod.rs:338). No `Regex` overload of `FindAll`; `Lang.CreateDialogFilter` not ported. |
| `LanguageManager.TryGetVariation` / `$` variants | PORTED | `mod.rs::try_get_variation` (`:273`), `update_text_value` (`:249`) | `$` split mirrors `:294-308`; 1 274–1 468 variant rows exist in the non-English files. |
| `LanguageManager.LoadLanguageFromFileTextCsv` | MISSING | — | C# `:229-271`; mods-only CSV path, documented as skipped (`mod.rs:9-10`). |
| `LanguageManager.LoadFromContentSources` / `UseSources` / `HotReloadContentFile` | MISSING | — | C# `:190`, `:184`, `:310`; mod content-source path (`AssetSourceController.cs:59`). |
| `GameCulture` (12 cultures, ids 1..12) | PORTED | `culture.rs::GameCulture` (`:5`), `CULTURES` (`:21`) | 1:1 with `GameCulture.cs:69-119`. `CultureName.Unknown = 9999` has no Rust twin; `GameCulture.DefaultCulture` is a settable static in C# but a `const` in Rust (cosmetic, the setter is never used). |
| `Language` selection UI (12 rows) | PORTED | `tr-app/src/menu/pages.rs::language_items` / `language_initial_items` | Same 12 `Language.<Culture>` keys in the same order as `Main.cs:48466-48477` / `:48497-48508`; `Act::SetLanguage(legacy_id)` → `menu/actions.rs:206` → `set_language` + `settings.language` + save, mirroring `Main.cs:48519-48523`. |
| First-launch picker (`_needsLanguageSelect`) | MISSING | — | C# `Main.cs:4335` + `:10653-10656` open menuMode 1212 when the config has no `"Language"`; the port always boots `Page::Main` with `settings.language = "en-US"` (`settings.rs:243`). `Page::LanguageInitial` exists but nothing navigates to it. |
| "auto" language detection | NOT FOUND | — | Vanilla 1.4.5.8 has **no** OS-locale auto-detect. Selection is: config `"Language"` (`Main.cs:4335-4347`), `-lang`/`-language` CLI (`LaunchInitializer.cs:118-125`), `lang=`/`language=` server args (`Main.cs:5035-5044`), the save-file byte (`Main.cs:4729-4730`) and the picker. The port adds `TR_LANG` (`env.rs:226`), which *overrides* the saved value (`main.rs:134`) — opposite precedence to C#, where the config wins over `-lang`. |
| `Lang.InitGlobalSubstitutions` (65 entries) | MISSING | `mod.rs::set_substitution` exists, **no producer** | C# `Lang.cs:100-151` registers `{Nurse}`, `{Guide}`, `{WorldName}`, `{Day}`, `{BloodMoon}` … The port never calls `set_substitution`, so content using them renders the raw `{Nurse}` token. Affected families: `LegacyDialog` (392 keys, 8×`{Nurse}`, 5×`{Dryad}`, 5×`{ArmsDealer}`, 4×`{Guide}`, 3×`{Merchant}`, 3×`{WorldName}`), `AnglerQuestText`, `AnglerChatter`, `DyeTraderSpecialText`, `GuideHelpTextSpecific`, … (~14 categories). |
| `Lang` name caches (item/NPC/projectile/buff/emoji/prefix/map-object) | PARTIAL | `tr-app/src/lang.rs::item_name` (`:161`) | Item names **are** data-driven: key `ItemName.{itemdata::NAMES[id]}` (`:6320`, 6 196 const names) → JSON value, falling back to the hardcoded English `itemdata::ITEMS[id].name` ("Dirt Block"). NPC names are **hardcoded English** except 20 town kinds (`ui/housing.rs::name_key`, `:225`); projectile (1 036), buff (352/356), emoji (151), prefix (97) and map-object (79) tables are not wired at all; `Lang.GetMapObjectName`/`BuildMapAtlas` missing. |
| `NetworkText` | PARTIAL | `tr-app/src/net.rs` (literal mode only) | `NetworkText.cs` (163) key→net encoding not ported (`LocalizedText.ToNetworkText` missing). |
| Resource discovery / packaging | PARTIAL | `tr-app/src/lang.rs::source_dirs` (`:37`) | The port loads the **same 84 JSON files at runtime** from `TR_LOCALE` / `../decompiled` / cwd / `CARGO_MANIFEST_DIR/../../decompiled` / `<exe>/decompiled`. Nothing is embedded in the crate, and no language JSON exists under `assets/Content` — the strings live in the gitignored `decompiled/` junction. The documented "sibling of the content dir" probe is off by one level: `content_dir.parent()/decompiled` = `assets/decompiled` (does not exist) instead of `<repo>/decompiled` (`lang.rs:42-46`). |
| `LanguageManager.OnLanguageChanged` event | PARTIAL | `mod.rs:211` | No subscriber in the port (the event itself is C#-only, `LanguageManager.cs:39`). |

## Tests

- `crates/tr-core/src/localization/mod.rs` — 10 tests: `json_creates_categories_and_english_values`,
  `later_files_override_earlier_values`, `dollar_keys_become_variations`,
  `copy_commands_expand_into_the_referenced_value`, `format_substitutes_arguments_and_keeps_escapes`,
  `variable_text_uses_substitutions_and_conditions`, `unknown_key_becomes_its_own_placeholder_text`,
  `switching_to_a_culture_reloads_fallback_then_target`, `indexed_category_walks_file_order`,
  `shipped_english_resources_match_the_game` (mod.rs:573-612 — **skips** when `../../decompiled`
  is absent, mod.rs:594).
- `crates/tr-core/src/localization/culture.rs` — 1 test: `cultures_match_the_original_ids_and_names`.
- `crates/tr-core/src/localization/format.rs` / `json.rs` — **0 unit tests** (the trailing-comma shim,
  the copy-command parser and `format_net` are only covered indirectly through `mod.rs`).
- `crates/tr-app/src/lang.rs` — 2 tests: `russian_texts_and_item_names_resolve` (`:191`, skips
  without sources), `tip_and_inter_keys_follow_the_c_sharp_prefixes` (`:223`).
- `crates/tr-app/src/menu/mod.rs:2028-2110` — the `Page::Language` / `Page::LanguageInitial` layout
  and item-count tests.
- Resource-dependent tests that **hard-fail** (not skip) without the `decompiled` junction:
  `tr-app/src/chat.rs:301` (`"*Belsin was slain 3 times."`), `tr-app/src/ui/mod.rs:1133`
  (`"Space to skip"`), `tr-app/src/lang.rs:219` (`menu(14) == "Settings"`),
  `tr-app/src/ui/emotes.rs:335` (`"Back"` / `"Назад"`).

## Deviations & risks

| # | Deviation | Severity | Admitted by ROADMAP/ANALYSIS? |
| --- | --- | --- | --- |
| D1 | No global substitutions: the 65 `Lang.RegisterGlobalSubstitution` entries (`Lang.cs:100-151`) had no port-side producer, so `{Nurse}`/`{WorldName}`/`{Day}` rendered literally inside ~14 dialogue categories. | behavioural | **Fixed (2026-11-13, S1-L-12b, `S1-L-12b-global-substitutions`)**: `lang::refresh_global_substitutions` materializes all 65 entries from live app state each frame before text resolution (`Lang.cs:100-167`). Some sources remain best-effort/unported (e.g. no `talkNPC` for `Homeless`, no world-level crimson/fishron/pumpking/ice-queen downed fields yet, input-trigger tags use stable fallbacks). |
| D2 | The whole dialogue/chatter family is absent: `Language.SelectRandom`, `Lang.CreateDialogFilter`, `GetRandomGameTitle`, `BartenderHelpText`, `AnglerQuestChat`, `GolferChat`, `PrincessChat`, `CatChat`, `DogChat`, `BunnyChat`, `SlimeChat` (`Lang.cs:295-410`) — i.e. `AnglerChatter`, `BartenderChatter`, `GolferChatter`, `CatNames_*`, `GameTitle` (76 keys) etc. are loaded but never read. | behavioural | Partially — ROADMAP §25 marks the module 🟡 (line 77) and "sub-screen strings remain (BUG-18)". |
| D3 | First-launch language picker unreachable: C# opens menuMode 1212 when the config has no `"Language"` (`Main.cs:4335`, `:10653`); the port boots straight into `Page::Main` with `"en-US"`. `Page::LanguageInitial` is dead code. | behavioural | No. |
| D4 | `TR_LANG` (env) takes precedence over the saved `Settings.language` (`main.rs:134-136`); in C# the config value wins over `-lang` (the CLI sets the culture, `LoadSettings` then re-applies the config, `Main.cs:4340/4345`). | behavioural | No — the ROADMAP calls `TR_LANG` "= the `-lang` flag" (line 1014) without noting the inverted precedence. |
| D5 | Culture-aware number/date formatting is missing: C# sets `CurrentCulture`/`CurrentUICulture` on `SetLanguage` (`LanguageManager.cs:95-96`) and formats through `string.Format`; the port pre-stringifies args (`lang.rs:133-137`) and `format_net` discards format specifiers (`format.rs:209-215`), so `{0:+0;-#}` renders as a bare number (2 shipped keys: `CommonItemTooltip.PrefixArmorPenetration`, `CommonItemTooltip.PrefixTagDamage`). | behavioural | No. |
| D6 | Resource probe off by one directory level: `source_dirs()` pushes `content_dir.parent()/decompiled` = `assets/decompiled` (`lang.rs:42-46`), but the retail-derived JSON lives in the repo-root `decompiled/` sibling of `assets/`. The other probes mask it in the dev tree. | behavioural | No — ROADMAP line 1011 documents the probe as "the `../decompiled` sibling of the content dir", which is what the code intends but does not compute. |
| D7 | **No embedded fallback**: the crate ships zero language bytes and the resources are read from a gitignored junction at runtime, so a deployed build without `decompiled/` next to the exe has no strings at all (every label degrades to its key). The C# embeds them in the assembly. | behavioural (packaging) | No. |
| D8 | The `assets`-symlink note in AGENTS.md is mis-attributed for localization: no language JSON exists under `assets/` (verified — `assets/Content` holds only fonts/images/sounds/xnb). The strings live in the **`decompiled`** junction (`tr-core` mod.rs:575, `tr-app` lang.rs:49); `assets` matters for the fonts. Tests that skip vs. fail differ accordingly (`mod.rs:594`, `lang.rs:201` skip; `chat.rs:312`, `ui/mod.rs:1139` fail with false "unknown key" text). | cosmetic (docs) | No. |
| D9 | Per-culture **file order** differs: C# loads the embedded resources in manifest order (csproj order), the port registers them in `read_dir` order (`lang.rs:74-98`). Today no key is duplicated inside a culture (verified: 0 duplicates in en-US and de-DE), so values are unaffected, but the `category_grouped_keys` order — and hence any `RandomFromCategory`/`IndexedFromCategory` index → key mapping — can diverge from C#. | determinism (latent) | No. |
| D10 | Missing API surface: `Language.GetTextValueWith`, `Exists` (not surfaced in `tr-app`), `FindAll(Regex)`, `SelectRandom`, `RandomFromCategory`'s RNG form, `LocalizedText.FormatWith(object)`'s property-lookup (LocalizedText.cs:195-207), `LocalizedText.EqualsCommand`/`ParseCommandPrefix`/`ToNetworkText`, `LanguageManager.EstimateWordCount` (`:65`), `LoadLanguageFromFileTextCsv`, `LoadFromContentSources`, `HotReloadContentFile`. | behavioural / cosmetic | Partially — the module is 🟡 in ROADMAP line 77; none of these is listed individually. |
| D11 | Stale ANALYSIS text: §"Localization (M6)" still says "Wiring the strings into the HUD, item names and menus is still to do" (ANALYSIS.md:1646-1647) although `lang::menu`/`lang::item_name` now drive them, and ANALYSIS.md:1526-1527 still says "localized display names stay out of scope". | cosmetic (docs) | No — the docs contradict the landed code. |
| D12 | `assets/Content` has no language JSON, so the ROADMAP's "the same retail-derived content model as `assets/Content`" (line 1011) is inaccurate: the language files are the *decompiled embedded resources*, a different retail-derived tree from the game's own `.xnb` content. | cosmetic (docs) | No. |
| D13 | `format.rs` and `json.rs` have **no** unit tests; the trailing-comma shim (needed because the shipped files carry Newtonsoft trailing commas — confirmed by re-parsing them) and the copy-command parser are only exercised indirectly. | determinism (test gap) | No. |

ROADMAP re-verification of the DONE rows in the M6 area:

- **`M6-LOCALIZE-wire` (ROADMAP:1600, ✅ done 2026-11-08)** — **PASS for its stated scope.**
  `lang.rs` gained the `Lang.tip`/`Lang.inter` accessors (`LegacyTooltip.{i}` / `LegacyInterface.{i}`,
  `Lang.cs:497`/`:485`) and `ui/tooltip.rs` uses them for the real stat rows (tooltip.rs:185-289:
  damage suffix `tip[2]/[3]/[4]/[53]/[55]`, crit `tip[5]`, use-time ladder `tip[6..13]`, knockback
  `tip[14..22]`, tool rows `tip[25..28]`), matching `Main.cs:20631-20800`. The row's own exclusions
  (accessory layer) are honest.
- **`M6-LOCALIZE-2-settings-screens` / `-3-death-text` / `-wire-fonts`** (named in prose at
  ROADMAP:1008-1009, no separate catalog row) — **PASS.** `death.rs` uses `DeathText.*`/`Currency.*`
  (death.rs:76-97, `:282`), `ui/char_creation.rs` + `ui/keyboard.rs` are key-driven (≈30 `UI.*`
  keys + `menu[24..31]`, `menu[45]`, `menu[48]`), and the font fix lives in `tr-content/src/font.rs`.
  "No menu/front-end labels remain to localize" is **not** exhaustively re-verified here (spot
  checks of the front-end screens pass).
- **`BUG-18` (ROADMAP:1703, ✅ done 2026-11-11)** — **PASS.** `ui/emotes.rs` now reads
  `lang::text("UI.Back")` (`BACK_KEY`) and the ru-RU value is asserted as `"Назад"` (emotes.rs:336-354).
- **ROADMAP §25 summary (lines 1003-1020)** — **PARTIAL.** "84 shipped resources, 12 cultures ×
  7 groups" is verified exactly; "loaded from `TR_LOCALE`/the `../decompiled` sibling of the content
  dir" is inaccurate (D6); "the same retail-derived content model as `assets/Content`" is
  inaccurate (D12); the module status stays 🟡 (line 77) — correctly, given D1-D5.
- **PORT_MAP.md:63** ("wip - sub-screens pending") — consistent with the findings above.

## Task seeds

- **S** — Fix the `source_dirs()` probe: compute the *repo-root* `decompiled` sibling
  (`content_dir.parent().parent().join("decompiled")`) and drop the dead `assets/decompiled` path
  (`tr-app/src/lang.rs:42-46`).
- **S** — Add unit tests for `format.rs` and `json.rs`: the trailing-comma shim on a real shipped
  snippet, `expand_copy_commands` on a `{$Cat.Key}` chain, `format_net` on `{0}`/`{{`/`}}`/a missing
  index.
- **S** — Port the 65 `Lang.InitGlobalSubstitutions` entries (`Lang.cs:100-151`): a
  `Substitution` producer in `tr-app` (world name, day counter, blood moon/eclipse flags and the
  town-NPC given names) so `LegacyDialog`/`AnglerQuestText` stop showing `{Nurse}`.
- **S** — Implement the first-launch picker: set a `needs_language_select` flag when the saved
  config has no `"Language"` and boot into `Page::LanguageInitial` (`Main.cs:4335`, `:10653-10656`).
- **S** — Sort the registered sources by file name so the per-culture load order is deterministic and
  documented as the C# manifest order (`lang.rs:74-98`).
- **M** — Implement `Language.SelectRandom` + `Lang.CreateDialogFilter` and port the chatter helpers
  (`Lang.cs:295-410`), including the `Main.rand` stream discipline, so `AnglerChatter`,
  `BartenderChatter`, `GolferChatter`, `CatChatter`, `SlimeChatter` and `GameTitle` (76 keys) can be
  read.
- **M** — Wire the remaining name tables through the JSON: `NPCName.*` (703 keys, incl. the
  negative-net-id slime cache `Lang.cs:521-533`), `ProjectileName.*` (1 036), `BuffName.*`/`BuffDescription.*`
  (352/356), `EmojiName.*` (151), `Prefix.*` (97) and `MapObject.*` (79, `Lang.GetMapObjectName`).
- **M** — Culture-aware argument formatting: keep the `IFormatProvider` behaviour of C#
  `string.Format` (numbers, `CurrentCulture`) and honour format specifiers, at least `{0:+0;-#}`.
- **M** — Extend `LanguageManager` with the missing C# surface: `GetTextValueWith`,
  `FindAll(Regex)`, `LocalizedText.FormatWith(object)`'s property lookup, `EqualsCommand`,
  `ParseCommandPrefix`, `ToNetworkText`, `EstimateWordCount`.
- **L** — Package the 84 language resources into the build (or a shipped `locales/` tree) so a
  deployed binary is not dependent on a gitignored `decompiled/` junction (D7).
- **L** — Audit and localise the remaining hardcoded English UI strings across `crates/tr-app/src/ui`
  and `menu/` (the picker rows themselves are localised; the rest of the front-end screens are
  spot-checked only).
- **S** — Docs: refresh ANALYSIS.md §"Localization (M6)" (D11), the ROADMAP §25 resource-path wording
  (D6/D12) and the AGENTS.md symlink note (D8).
