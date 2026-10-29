# Task Status — Multi-Agent Coordination

**Last updated:** 2026-10-29T19:40Z  
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
| M1.5-LIQUID-2 | Water styles and shimmer | ❌ Pending | - | - |

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
| M6-MUSIC | XACT/WMA music playback | 🔄 In Progress | Agent-Audio | `Main.UpdateAudio`/`Main.GetMusicTrack` selection + `SoundEngine.PlayMusic`; MS-ADPCM wave bank decode (`tr-content/xact.rs`, `adpcm.rs`) and `audio::MusicSystem` fade/loop |
| M6-3DAUDIO | 3D audio attenuation | 🔄 In Progress | Agent-Audio | `SoundEngine.PlaySound` distance/pan math, `SoundEngine.Update` 3D re-positioning, music/sound/ambient volume application |
| M6-LOCALIZE | Localization system | ✅ Done | Agent-DeepSeek | `tr-core/src/localization.rs`: `GameCulture`/`LocalizedText`/`VariableText`/`LanguageManager` (JSON loader, `$`-variants, `{$Key}` copy commands, `{Name}`/conditions, `format_net`); 11 tests incl. the shipped `en-US` files. UI wiring still pending |

### Milestone 7 — UI (1:1 layout)

| ID | Task | Status | Assigned To | Notes |
|----|------|--------|-------------|-------|
| M7-UI-1 | Menu framework (DeathText, item scale, 5-pass shadow, hover) | 🔄 In Progress | Agent-DeepSeek | `Main.DrawMenu` geometry: `num2/num3/num4`, `menuItemScale`, `array4/5/7`, colors |
| M7-UI-2 | Settings screens (menuMode 11 / 112 / 1112 / 1111 / 26 / 1125 / 1127 / 1213) | 🔄 In Progress | Agent-DeepSeek | Labels, order, row spacing, toggles, sliders |
| M7-UI-3 | Main menu screen (logo, items, version, links) | ❌ Pending | - | - |
| M7-UI-4 | Inventory / ItemSlot cells exact sizes | ❌ Pending | - | 56 px cells, 0.85 scale, contexts |
| M7-UI-5 | Settings persistence (`config.json`, `Main.SaveSettings`) | ❌ Pending | - | - |

### Tile Objects & Special Tiles

| ID | Task | Status | Assigned To | Notes |
|----|------|--------|-------------|-------|
| TO-FURNITURE | TileObjectData (furniture, doors, chests) | ❌ Pending | - | - |
| TO-PAINT | Paint and coatings | ❌ Pending | - | - |
| TO-GEMSPARK | Gemspark/glow tiles | ❌ Pending | - | - |

---

## 🔥 Current Assignments (in progress)

```
M7-UI-1/2     Menu framework + settings screens — Agent-DeepSeek (started 2026-10-29T18:15Z)
M6-MUSIC      XACT/MS-ADPCM music playback — Agent-Audio (started 2026-10-29T18:25Z)
M6-3DAUDIO    SoundEngine 3D attenuation — Agent-Audio (started 2026-10-29T18:25Z)
```

---

## ✅ Recently Completed

```
d780b99  [M1.5-LIGHT-2] Wire the per-type tile light into the app  (Agent-DSV4)
4d42f34  [M1.5-LIGHT-2] Port the full per-type tile light switch   (Agent-DSV4)
74bcf10  [M6-LOCALIZE]   Port the localization core                  (Agent-DeepSeek)
077f0d6  [M1.5-LIQUID-1] Port the WaterfallManager overlay          (Agent-DeepSeek)
79680f2  [M3-TORCH]      Port WorldGen.CheckTorch attachment frames  (Agent-DeepSeek)
```

---

## ⚠️ Coordination notes (2026-10-29T18:40Z)

- **`tr-app` does not build in the working tree right now.** As of this note the last
  `cargo test --workspace` failed inside `crates/tr-app/src/audio.rs` (the uncommitted
  `impl Source for Panned<S>` uses `current_span_len`, which the pinned `rodio` does not
  declare) — that is Agent-Audio's in-flight M6-MUSIC work, not a committed regression.
  `tr-core` alone builds and its 67 tests pass. Earlier, the `HEAD` at `4d42f34` also did
  not build because it landed `tr-core/src/{lighting,tile_light}.rs` without the matching
  `tr-app/src/main.rs` `LightParams` wiring; M1.5-LIGHT-2's `d780b99` restored that.
- **`main.rs` was clobbered.** At ~17:56 an in-flight edit replaced `crates/tr-app/src/main.rs`
  with a pre-waterfall version: the whole `WaterfallManager` wiring and the committed
  M1.5-LIGHT-1 `mouseTextColor` flicker disappeared, and the file referenced
  `audio::Music::new()`, which does not exist in `audio.rs` yet (so it did not compile).
  The file was restored from `HEAD` and the waterfall/torch hooks re-applied. If you are
  mid-refactor, re-apply your hunk on top of the current file instead of saving an old
  buffer.
- **Uncommitted audio work** in `crates/tr-app/src/audio.rs`, `crates/tr-content/src/xact.rs`
  and `adpcm.rs` belongs to another agent (music/XACT decoding). It was left untouched and
  not committed.
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
