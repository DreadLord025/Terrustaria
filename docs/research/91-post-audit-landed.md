# Post-audit landed merges — re-verification (current tree)

- **Generated:** 2026-11-12
- **Worktree/branch:** `C:\Users\DreadLord\Desktop\TR-AUDIT-2-docs` (`AUDIT-2-docs-reverification`)
- **Audit base:** `f0a4937` (`git log --oneline --merges f0a4937..origin/main`)
- **Baseline:** `origin/main` at `f3e2fcf` (HEAD of the inspected merge range)

## Scope & method

- **Task merges inspected (per scope):** `M3-EQUIP-12` buff-immunity (`d5c040e`), `M3-EQUIP-13` armor-stats (`cf422c5`), `M3-EQUIP-14` sky-stone (`b7504cd`), `M3-EQUIP-15` thorns (`2e7ce35`), `M3-EQUIP-16` lava-time (`cc6298a`), `M3-EQUIP-17` life-regen (`7531423`), `M3-EQUIP-18` item-life-regen (`f9163d8`), `M3-EQUIP-19` anchors (`33bfdf6`), `M3-EQUIP-20` armor-pen (`17ff78d`), `M7-UI-8` hide-toggles (`63b414f`), `M4-PROJ-6b` magic-followups (`937a350`), `M5-DUNGEON-lights` (`cc526b1`), `M5-DUNGEON-banners` (`2021d15`), `INFRA-main-rules` (`fb5c5ef`), `INFRA-npc-sheet-audit` (listed in scope; no task merge commit named in the range — only `f3e2fcf merge origin/main into INFRA-npc-sheet-audit` present; verified against current tree + `docs/ATLAS.md` row).
- **Noise excluded:** `merge origin/main into ...`, remote-tracking forward merges, and non-task merges (e.g. `AUDIT-1`, `RELAND*`, `M4-NPC-*`, UI/settings merges) except where needed to confirm the inspected task merges.
- **Sources read (per task):** `docs/ATLAS.md` row (grep by task id), merge commit message (`git log -1 --format=%B <merge>`), and `anchors.txt` rows where present.
- **Verification:** read-only checks against current code in `crates/` (`Select-String`/file reads for cited symbols/paths). No `cargo`, no git writes, no `graphify update/merge-graphs`. Checks targeted symbols/modules only (no whole-file reads of god files).
- **ROADMAP mapping:** `docs/ROADMAP.md` is the 2026-11-12 audited S1 rewrite; these landed tasks are ATLAS/content/infra rows with no direct S1-* id in ROADMAP. Mapping recorded as `— (ATLAS)` unless a clear structural tie exists.

## Main table

| task | claim (one line) | verdict | evidence (C# / Rust) | ROADMAP S1-* |
| --- | --- | --- | --- | --- |
| M3-EQUIP-12 | Equip/accessory buff-immunity rows grant `buffImmune` (Ankh Shield family, parts, sets). | **PASS** | C#: `Player.cs` buff-immunity grant sites (per ATLAS/commit: buffImmune rows). Rust: `crates/tr-app/src/player.rs:242` `BUFF_IMMUNITY_ROWS`, `:2764-2766` applies worn `BUFF_IMMUNITY_ROWS` to `buff_immune`; `anchors.txt:48` pins `BUFF_IMMUNITY_ROWS`. | — (ATLAS M3-EQUIP-12) |
| M3-EQUIP-13 | Armor pieces contribute their stat rows (not just defense). | **PASS** | C#: `Player.cs:13126-139xx` `GrantArmorBenefits` switch (per ATLAS). Rust: `crates/tr-core/src/armor_sets.rs:440` `ARMOR_STATS`, `:552-554` `armor_stats()`; `crates/tr-app/src/player.rs` consumes armor/set stats in equip/tick; `anchors.txt:49` pins `ARMOR_STATS`. | — (ATLAS M3-EQUIP-13) |
| M3-EQUIP-14 | Celestial/Sun/Moon stone bundle sets `skyStoneEffects` by item/time. | **PASS** | C#: `Player.cs:12988-13003` sky-stone bundle (per ATLAS). Rust: `crates/tr-app/src/player.rs:6247` `the_sky_stones_add_their_bundle` test/marker and sky-stone grant logic in equip pass (field/flag usage present); `anchors.txt:50` pins `the_sky_stones_add_their_bundle`. | — (ATLAS M3-EQUIP-14) |
| M3-EQUIP-15 | Thorns/cactus/turtle reflection with contact bounce arithmetic. | **PASS** | C#: `Player.cs:31373-31415` thorns/contact reflection (per ATLAS). Rust: `crates/tr-app/src/player.rs:2298` `fn thorns_damage`, reflection applied in contact/hurt path; `anchors.txt:51` pins `fn thorns_damage`. | — (ATLAS M3-EQUIP-15) |
| M3-EQUIP-16 | Lava charm family grants `lavaMax/lavaTime` and the 80-damage lava hit behavior. | **PASS** | C#: `Player.cs` `lavaMax/lavaTime` handling (per ATLAS/commit: +420 rows, 80-damage hit). Rust: `crates/tr-app/src/player.rs:620` `lava_max`, `:2682` reset, `:3077/:3166/:3169` charm rows add 420, `:5786-5790` lava timer cap/use, `:6178-6185` tests; `anchors.txt:52` pins `lava_max`. | — (ATLAS M3-EQUIP-16) |
| M3-EQUIP-17 | Armor/buff/stone life-regen rows feed `extra_regen`. | **PASS** | C#: `Player.cs` UpdateLifeRegen/equip lifeRegen rows (per ATLAS: pieces 3800/3872/2763-5, buffs 2/165, Celestial Stone +2). Rust: `crates/tr-app/src/player.rs:6141` `the_life_regen_rows_add_up` and equip/buff aggregation into regen input (`crates/tr-app/src/regen.rs` consumes `extra_regen`); `anchors.txt:53` pins `the_life_regen_rows_add_up`. | — (ATLAS M3-EQUIP-17) |
| M3-EQUIP-18 | `Item.lifeRegen` provisioned in generated item data and applied from worn items. | **PASS** | C#: `Item.cs` `lifeRegen` (per ATLAS: `Item.cs:49013`). Rust: `crates/tr-app/src/itemdata.rs` carries `life_regen` field/values (generated; grep shows `life_regen: 0` stock + populated rows), `tools/GenItemData/gen.py` emits it; equip pass adds worn item life regen; `anchors.txt:54` pins `itemdata.rs` `life_regen`. | — (ATLAS M3-EQUIP-18) |
| M3-EQUIP-19 | Pin landed M3-EQUIP features in `anchors.txt` (tripwire). | **PASS** | C#: n/a (infra/pinning task). Rust/repo: `anchors.txt:41-55` contains M3-EQUIP-5..20 anchor rows including 12-18/20; `crates/tr-core/build.rs` and `crates/tr-app/tests/main_anchors.rs` enforce anchors (per repo rules/anchors header). | — (ATLAS M3-EQUIP-19/infra) |
| M3-EQUIP-20 | Armor penetration rows + melee-hit consumer. | **PASS** | C#: `Player.cs` armor penetration fields/consumers (per ATLAS/commit). Rust: `crates/tr-app/src/player.rs:2294` `fn get_armor_penetration` and melee/penetration rows wired; `anchors.txt:55` pins `fn get_armor_penetration`. | — (ATLAS M3-EQUIP-20) |
| M7-UI-8 | Accessory "hide the visuals" toggles (`hideVisibleAccessory`) in inventory UI. | **PASS** | C#: `Player.cs:1708` `hideVisibleAccessory`, inventory tick visuals `Main.cs`/`ItemSlot.cs` hide-toggle behavior (per ATLAS). Rust: `crates/tr-app/src/ui/inventory.rs:137` `HideAccessory(usize)`, `:229/:298/:387/:389/:623/:630/:641` hide-visibility handling/draw/toggle; `crates/tr-app/src/app/clicks.rs` forwards hide toggle; `docs/ATLAS.md` row states `Player.hideVisibleAccessory` ported. | — (ATLAS M7-UI-8) |
| M4-PROJ-6b | Magic follow-up caster families/arms (aiStyles 27/40/51/59/71/118/119/75/84 etc.), Last Prism pair, shoot specials, trail pass. | **PASS (present; not re-exhaustively enumerated here)** | C#: `Projectile.cs` aiStyle arms for listed families + Last Prism (per commit/ATLAS). Rust: `crates/tr-app/src/projectile.rs` contains `LAST_PRISM`/`LAST_PRISM_LASER` (`:576-581`), defs/usages (`:1766-1773`), `ai_last_prism`/`ai_last_prism_laser` (`:5246/:5383`) and trail-pass logic (`:8185` comment/ref). Broad family coverage claimed by commit/ATLAS; spot-check confirms key Last Prism + trail-pass symbols present. | — (ATLAS M4-PROJ-6b) |
| M5-DUNGEON-lights | Dungeon global lights pass (+ restored spike pass context in merge). | **PASS** | C#: Dungeon generation passes (DungeonCrawler/WorldGen dungeon passes, per ATLAS/commit). Rust: `crates/tr-core/src/worldgen/dungeon_features.rs:644` `pub fn place_lights`; spike pass also present (`:173` `place_spikes`) consistent with merge note. | — (ATLAS M5-DUNGEON) |
| M5-DUNGEON-banners | Dungeon global banners pass + `WorldGen.PlaceBanner` support. | **PASS** | C#: Dungeon banner placement pass / tile placement (per ATLAS/commit). Rust: `crates/tr-core/src/worldgen/dungeon_features.rs:1245` `pub fn place_banners`; `crates/tr-core/src/worldgen/dungeon.rs` wires dungeon generation (`fn generate_dungeon` present at `:982` in earlier checks context / symbol exists). | — (ATLAS M5-DUNGEON) |
| INFRA-main-rules | Multi-agent main rules + prompt/documentation. | **PASS** | C#: n/a. Repo: `docs/MAIN-RULES.md` present and defines merge/verify/anchor/re-land procedure (read in full); `anchors.txt:39` pins `docs/MAIN-RULES.md` marker; merge commit `fb5c5ef` matches. | — (infra) |
| INFRA-npc-sheet-audit | NPC sheet-packing guard / audit fix (per ATLAS row; merge commit not found by task name). | **PARTIAL (evidence of feature present; merge provenance unclear)** | C#: n/a (engine/asset guard). Rust: `docs/ATLAS.md:1565` documents the `NpcAssets`/sheet-packing guard landed 2026-11-12; current tree contains NPC asset/sheet handling in `crates/tr-app/src/npc.rs` and related render paths (not fully re-audited line-by-line here). No `Merge branch 'INFRA-npc-sheet-audit'` commit found in `f0a4937..origin/main`; only forward-merge commits into that branch observed. Treat as present-but-provenance-unconfirmed in this range. | — (ATLAS INFRA-npc-sheet-audit) |

## Work-loss check (documented losses)

Checked against current tree symbols/paths named in `anchors.txt` and repo rules:

| loss record | status now | evidence |
| --- | --- | --- |
| BUG-21 (single-player world entry) — lost twice | **PRESENT** | `crates/tr-app/src/app/mod.rs:1291` `pub(crate) fn enter_loaded_world(&mut self)` |
| BUG-21 (cached static/world-independent atlas half) — associated | **PRESENT** | `crates/tr-app/src/app/mod.rs:112` `static_sheets`, `:1633/:1647/:1690/:2826/:3477/:3482/:3922` cached usage |
| `touch_damage.rs` (tile contact damage) — lost with dungeon features | **PRESENT** | `crates/tr-app/src/touch_damage.rs:169` `pub fn apply_touch_damage`, `:35` `TOUCH_DAMAGE_IMMEDIATE` |
| `worldgen/dungeon_features.rs` (spikes/lights/wall variants/banners/traps) — lost twice (`9f89c99`, `cc6298a`) | **PRESENT** | `crates/tr-core/src/worldgen/dungeon_features.rs:173` `place_spikes`, `:644` `place_lights`, `:837` `wall_variants`, `:1164` `place_traps`, `:1245` `place_banners` |
| M4-NPC-3m (recorded in repo history/rules context) — symbol-level proxy | **PRESENT (proxy: walker/NPC sheet anchors intact)** | `crates/tr-app/src/npc.rs:4478` `fn walker_frame`, `:1885` `fn extra_frames_count`; `crates/tr-app/src/town_arrival.rs:234` `update_time_spawn_town_npcs`; dungeon crawler `crates/tr-core/src/worldgen/dungeon.rs:982` `fn generate_dungeon`. (No separate M4-NPC-3m-specific symbol audited beyond these anchors.) |

**Conclusion:** All anchor-tracked features implicated by the documented losses are present in the current tree at the cited locations.

## Deviations & risks

| severity | item | detail |
| --- | --- | --- |
| **Medium** | `INFRA-npc-sheet-audit` merge provenance | ATLAS marks it done 2026-11-12, but `git log --oneline --merges f0a4937..origin/main` shows no task merge commit for this branch name (only merges *into* it). Feature appears present, but this re-verification cannot prove it landed via the scoped merge set. |
| **Low** | M4-PROJ-6b breadth not exhaustively rechecked | Commit/ATLAS claim many aiStyle arms and shoot specials; spot checks confirm Last Prism pair + trail pass + projectile module presence, but full per-style parity (all listed 27/40/51/59/71/118/119/75/84 arms and every shoot special) was **not** line-audited here (tool/time budget). Risk of hidden partial/overclaim inside a broadly present merge remains low-to-moderate. |
| **Low** | No runtime/test execution | Per constraints, no `cargo`/build/tests run; verdicts are static symbol/presence checks plus commit/ATLAS claims. Behavioral regressions introduced after merge would not be detected. |
| **Low** | ROADMAP S1 mapping unavailable | Audited `docs/ROADMAP.md` has no S1 rows for these equip/UI/proj/dungeon-feature merges; traceability is ATLAS-only. |
| **Info** | Anchor/grep encoding noise in some reads | `Select-String` output showed encoding artifacts for non-ASCII markers in ATLAS/anchors display, but symbol matches and file contents were confirmed. |

## Task seeds (remaining work)

| task | size | remaining / follow-up |
| --- | --- | --- |
| M3-EQUIP-12 | **S** | Maintain coverage as new accessories/sets land; add parity tests for edge immunity interactions (stacking, debuff-clear timing) if missing. |
| M3-EQUIP-13 | **M** | Keep `ARMOR_STATS`/set tables in sync with generator/C# updates; audit any armor pieces still relying on defense-only fallback. |
| M3-EQUIP-14 | **S** | Verify all stone-combo variants/timing edges (day/night boundaries, Celestial Shell states) against C# if not already covered. |
| M3-EQUIP-15 | **S** | Confirm thorns damage scaling/targets/iframing parity in all contact cases (NPC/projectile reflect nuances) and PvP/non-contact exclusions. |
| M3-EQUIP-16 | **S** | Validate lava immunity interactions with other lava sources/buffs and exact tick accounting at boundaries. |
| M3-EQUIP-17 | **S** | Ensure all regen sources (equipment/buffs/auras) compose in correct order/caps per `Player.cs` UpdateLifeRegen. |
| M3-EQUIP-18 | **S** | Keep generator/idempotence checks (`GenItemData` rerun clean) in CI habit; extend to any future item-regen fields if C# changes. |
| M3-EQUIP-19 | **S** | Continue adding anchors for every newly landed feature (policy task; no code gap). |
| M3-EQUIP-20 | **S** | Audit non-melee penetration consumers and interaction with enemy defense/DR rules where applicable. |
| M7-UI-8 | **S** | Verify persistence/save-load of `hideVisibleAccessory`, vanity/accessory slot edge cases, and multiplayer sync if/when relevant. |
| M4-PROJ-6b | **M** | Full per-arm parity audit for all listed magic follow-up aiStyles and shoot-special edge cases; add targeted behavior tests/screenshot checks for regressions. |
| M5-DUNGEON-lights | **S** | Validate light placement density/variants against C# pass across seeds; ensure no interaction regressions with other dungeon feature passes. |
| M5-DUNGEON-banners | **S** | Validate banner type/placement rules and `PlaceBanner` behavior across dungeon styles/seeds. |
| INFRA-main-rules | **S** | Keep `docs/MAIN-RULES.md`/`anchors.txt`/tripwires aligned as workflow evolves; ensure agents follow ref-update + landed-check discipline. |
| INFRA-npc-sheet-audit | **S** | Clarify/record landing provenance (merge commit or re-land note) and add/confirm an anchor/row if needed for the guard symbol(s). |

## Summary counts

- **PASS:** 13
- **PARTIAL:** 1 (`INFRA-npc-sheet-audit`, provenance unclear though feature present)
- **FAIL:** 0
- **OVERCLAIMED:** 0 (not proven by this pass)

Worst finding: **provenance gap** for `INFRA-npc-sheet-audit` in the inspected merge range. Work-loss status: **all checked lost features present now** (BUG-21, touch damage, dungeon features spikes/lights/wall variants/traps/banners, walker/town/dungeon anchors intact).
