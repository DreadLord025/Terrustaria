# Task Status — Multi-Agent Coordination

**Last updated:** 2026-10-29T19:50Z  
**Purpose:** Track all porting tasks, assign to agents, avoid duplicate work

---

## 📋 Master Task Matrix

### Milestone 1.x — Rendering & Core Systems

| ID | Task | Status | Assigned To | Notes |
|----|------|--------|-------------|-------|
| M1.4-BG-1 | Background gradients (forest, mountain) | 🔄 In Progress | Agent-DeepSeek | ... |
| M1.4-BG-2 | Cloud generation and parallax | ❌ Pending | - | - |
| M1.4-BG-3 | Sun, moon, stars rendering | ❌ Pending | - | - |
| M1.4-BG-4 | Day-night cycle and sky colors | ❌ Pending | - | - |
| M1.4-BG-5 | Rain and snow systems | ❌ Pending | - | - |
| M1.4-BG-6 | Biome background variants | ❌ Pending | - | - |
| M1.5-LIGHT-1 | Star/Moon/Light flicker | ✅ Done | Agent-Nemotron | Ported mouseTextColor flicker (torch/lava/shimmer/crystal-wall light oscillation), committed |
| M1.5-LIGHT-2 | Per-type tile lighting | ✅ Done | Agent-DSV4 | Full `TileLightScanner.ApplyTileLight`/`ApplyWallLight` port: all 147 cases, paint-coloured light, disco/demon/shimmer dynamics, per-tile `FastRandom`. Scanner core in `4d42f34`; app wiring (`demonTorch` ramp, `DiscoRGB`, per-frame scanner seed, `LightParams`) in `d780b99` |
| M1.5-LIQUID-1 | Waterfall overlays | ✅ Done | Agent-DeepSeek | `waterfall.rs` ports `FindWaterfalls`/`UpdateFrame`/`DrawWaterfall`; wired between the non-solid and solid passes; lake A/B pixel-identical; see ANALYSIS.md "Waterfall overlay" |
| M1.5-LIQUID-2 | Water styles and shimmer | 🔄 In Progress | Agent-DeepSeek | Finish the liquid draw: honey/shimmer waterfall passes, shimmer sparkle, ambient light; check water style selection |

### Milestone 2 — Player

| ID | Task | Status | Assigned To | Notes |
|----|------|--------|-------------|-------|
| M2-DRAW-1 | Player draw layers (armor/accessories) | ❌ Pending | - | - |
| M2-DRAW-2 | Composite arm rotation | ❌ Pending | - | - |
| M2-ANIM | Full itemAnimation/use timings | ❌ Pending | - | - |
| M2-BUFF | Buffs, hooks, mounts, gravity flips | ❌ Pending | - | - |

### Milestone 3 — Items & Crafting

| ID | Task | Status | Assigned To | Notes |
|----|------|--------|-------------|-------|
| M3-TOOL | Tool tiers and durability | ❌ Pending | - | - |
| M3-TORCH | Torch attachment frames (`CheckTorch`) | ✅ Done | Agent-DeepSeek | `tr-core/src/torch.rs`: frames 0/22/44, tall 66 offset, removal when unsupported; run from `place_tile` and `reframe_area` |
| M3-EQUIP | Equipment slots and paint | ❌ Pending | - | - |
| M3-ACTUATOR | Actuators | ❌ Pending | - | - |
| M3-CRAFT | Recipe.cs crafting menu | ❌ Pending | - | Port ItemSlot contexts 4/33 |
| M3-PREFIX | Item prefixes system | ❌ Pending | - | - |
| M3-INV | Full inventory UI | ❌ Pending | - | - |

### Milestone 4 — NPCs & Combat

| ID | Task | Status | Assigned To | Notes |
|----|------|--------|-------------|-------|
| M4-NPC-1 | Remaining NPC types (780 total) | ❌ Pending | - | ~780 AI implementations |
| M4-SPAWN | NPC spawn/biome/night rules | ❌ Pending | - | - |
| M4-LOOT | Full loot tables/item drops | ❌ Pending | - | - |
| M4-PROJ | Projectile system (~1100 types) | ❌ Pending | - | - |
| M4-DAMAGE | Damage types/defense/crits | ❌ Pending | - | - |
| M4-BOSS | All bosses | ❌ Pending | - | - |
| M4-TOWN | Town NPCs and shops | ❌ Pending | - | - |

### Milestone 5 — World Generation

| ID | Task | Status | Assigned To | Notes |
|----|------|--------|-------------|-------|
| M5-BIOME | Corruption/Crimson/Jungle/Hallow | ❌ Pending | - | - |
| M5-OCEAN | Full ocean biome | ❌ Pending | - | - |
| M5-DESERT | Desert structures and loot | ❌ Pending | - | - |
| M5-DUNGEON | Dungeons | ❌ Pending | - | - |
| M5-TEMPLE | Jungle temples | ❌ Pending | - | - |
| M5-LIVING | Living trees | ❌ Pending | - | - |
| M5-HARDFORCE | Hardmode spread | ❌ Pending | - | - |
| M5-CHEST | Chest generation and loot | ❌ Pending | - | - |

### Milestone 6 — Audio & Polish

| ID | Task | Status | Assigned To | Notes |
|----|------|--------|-------------|-------|
| M6-MUSIC | XACT/WMA music playback | ✅ Done | Agent-Audio | `918b5a5`: XWB/XSB reader + MS-ADPCM decode (`tr-content/xact.rs`, `adpcm.rs`) and `audio::MusicSystem` (`Main.UpdateAudio`/`LegacyAudioSystem.UpdateCommonTrack` fade/selection/loop). See ANALYSIS.md "Audio (M6)" |
| M6-3DAUDIO | 3D audio attenuation | ✅ Done | Agent-Audio | `918b5a5`: `sound.rs` ports `LegacySoundPlayer.PlaySound` (2500 px falloff, pan, ambient volume); positional `play_at` + listener wiring in `main.rs` |
| M6-LOCALIZE | Localization system | ✅ Done | Agent-DeepSeek | `tr-core/src/localization.rs`: `GameCulture`/`LocalizedText`/`VariableText`/`LanguageManager` (JSON loader, `$`-variants, `{$Key}` copy commands, `{Name}`/conditions, `format_net`); 11 tests incl. the shipped `en-US` files. UI wiring still pending |

### Milestone 7 — UI (1:1 layout)

| ID | Task | Status | Assigned To | Notes |
|----|------|--------|-------------|-------|
| M7-UI-1 | Menu framework (DeathText, item scale, 5-pass shadow, hover) | ✅ Done | Agent-DeepSeek | `menu.rs`: `Main.DrawMenu` geometry + colours, 9 tests; see docs/UI.md |
| M7-UI-2 | Settings screens (menuMode 11 / 112 / 1112 / 1111) | ✅ Done | Agent-DeepSeek | General/Interface/Video rows, labels and toggles; volume sliders, cursor, controls, language left as notices |
| M7-UI-3 | Main menu screen (logo, items, version, links) | ✅ Done | Agent-DeepSeek | Logo at (w/2,100) with scale/rotation easing, seven vanilla rows, `v1.4.5.8` in Mouse_Text |
| M7-UI-4 | Inventory / ItemSlot cells exact sizes | ❌ Pending | - | 56 px cells, 0.85 scale, contexts |
| M7-UI-5 | Settings persistence (`config.json`, `Main.SaveSettings`) | ✅ Done | Agent-DeepSeek | `settings.rs`: same keys as vanilla, round-trip test; LightingMode/Background/MapEnabled applied |

### Tile Objects & Special Tiles

| ID | Task | Status | Assigned To | Notes |
|----|------|--------|-------------|-------|
| TO-FURNITURE | TileObjectData (furniture, doors, chests) | ❌ Pending | - | - |
| TO-PAINT | Paint and coatings | ❌ Pending | - | - |
| TO-GEMSPARK | Gemspark/glow tiles | ❌ Pending | - | - |

---

## 🔥 Current Assignments (in progress)

```
M7-UI-1/2/3/5 Menu framework + settings — done 2026-10-29T18:45Z (Agent-DeepSeek)
M6-MUSIC      XACT/MS-ADPCM music playback — done 2026-10-29T19:50Z (Agent-Audio)
M6-3DAUDIO    SoundEngine 3D attenuation — done 2026-10-29T19:50Z (Agent-Audio)
```

---

## ✅ Recently Completed

```
918b5a5  [M6-MUSIC]      XACT/MS-ADPCM music engine + 3D attenuation (Agent-Audio)
6b8e36e  [M7-UI]         Main.DrawMenu menu + settings screens     (Agent-DeepSeek)
d780b99  [M1.5-LIGHT-2] Wire the per-type tile light into the app  (Agent-DSV4)
4d42f34  [M1.5-LIGHT-2] Port the full per-type tile light switch   (Agent-DSV4)
74bcf10  [M6-LOCALIZE]   Port the localization core                  (Agent-DeepSeek)
077f0d6  [M1.5-LIQUID-1] Port the WaterfallManager overlay          (Agent-DeepSeek)
79680f2  [M3-TORCH]      Port WorldGen.CheckTorch attachment frames  (Agent-DeepSeek)
```

---

## ⚠️ Coordination notes (2026-10-29T19:50Z)

- **The `rodio` `current_span_len` build error is fixed** in `918b5a5` (`current_frame_len`),
  and `cargo test --workspace` now passes (130 tests). The shared tree's remaining
  uncommitted work (M1.5-LIGHT-2 `mouseTextColor` relocation in `main.rs`,
  `lighting/tile_light/tiledata/liquid_draw/save`, `ROADMAP.md`) is Agent-DSV4's and was
  deliberately left uncommitted.

## ⚠️ Notes retained from 2026-10-29T18:40Z

- **`main.rs` was clobbered earlier.** At ~17:56 an in-flight edit replaced
  `crates/tr-app/src/main.rs` with a pre-waterfall version and it was restored. If you are
  mid-refactor, re-apply your hunk on top of the current file instead of saving an old
  buffer.
- `tools/GenTileData` no longer matches the current decompilation (`Main.`-prefixed
  assignments, `decompiled/Terraria/ID/…`); running it emits empty tables and overwrites
  `tiledata.rs`. See `docs/ANALYSIS.md` risk 7. Do not regenerate until it is fixed.
- Verify `cargo test --workspace` and take a screenshot **before** committing; the shared
  working tree is being edited by several agents at once.

---

## 📝 Usage Instructions for Agents

### Before Starting Work
1. Read this file completely
2. Find an unassigned task (marked "❌ Pending")
3. Verify no other agent is working on similar task
4. Check related documentation in `docs/`

### Starting a Task
1. Update task status to **"🔄 In Progress"**
2. Add your name/identifier in "Assigned To" column
3. Begin implementation following `agents.md` golden rule (port C# first)

### Completing a Task
1. Mark task as **"✅ Done"**
2. Add commit hash/reference
3. Update `docs/ROADMAP.md` if milestones shift
4. Commit with clear message referencing task ID (e.g., "[M1.4-BG-1] Background gradients")

### Important Rules
- NEVER work on same task simultaneously
- If uncertain about behavior → search C# source first
- Document non-obvious decisions in `docs/ANALYSIS.md`
- Verify against original game before committing

---

**File auto-generated for multi-agent coordination**  
**Last modification timestamp: 2026-10-29T00:00:00Z**
