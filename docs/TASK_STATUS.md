# Task Status — Multi-Agent Coordination

**Last updated:** 2026-10-29  
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
| M1.5-LIGHT-2 | Per-type tile lighting | ❌ Pending | - | - |
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
| M6-MUSIC | XACT/WMA music playback | ❌ Pending | - | - |
| M6-3DAUDIO | 3D audio attenuation | ❌ Pending | - | - |
| M6-LOCALIZE | Localization system | ❌ Pending | - | - |

### Tile Objects & Special Tiles

| ID | Task | Status | Assigned To | Notes |
|----|------|--------|-------------|-------|
| TO-FURNITURE | TileObjectData (furniture, doors, chests) | ❌ Pending | - | - |
| TO-PAINT | Paint and coatings | ❌ Pending | - | - |
| TO-GEMSPARK | Gemspark/glow tiles | ❌ Pending | - | - |

---

## 🔥 Current Assignments (in progress)

```
No tasks currently assigned
```

---

## ✅ Recently Completed

```
See individual commits for details
```

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
