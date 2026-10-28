# Roadmap & honest scope

## Why this is a long project

Terraria 1.4.5.8 is ~755 000 lines of decompiled C#. Most of it is *content behaviour*, not
engine: ~6 200 items, ~780 NPCs, ~1 100 projectiles and ~750 tile types, each with
hand-written logic, plus ~45 000 lines of world generation and ~35 000 lines of UI.

A faithful Rust port is therefore a **multi-year engineering effort** (a small team for
2–3 years; substantially longer for one developer). What exists now is the foundation: the
hard, low-level parts (save format, asset pipeline, GPU renderer) are working and measured.
The estimates below are in "focused agent-days" — a day of continuous work by an AI agent
with the decompiled sources at hand — and are honest order-of-magnitude figures with
uncertainty of roughly ±50 %.

## Milestone 0 — foundation ✅ (done)

| Deliverable | Result |
| --- | --- |
| Decompile game + ReLogic | 1 549 + 167 `.cs` files |
| XNB → PNG pipeline (LZX, DXT) | 15 100 / 15 123 textures |
| `.wld` parser | 11.5 M tiles in ~60 ms |
| Vulkan renderer (wgpu 30) | 2 draw calls, ~1 900 instances |
| Bench | 1.91 ms avg / 523 FPS @1600×900 |
| Debug overlay, screenshots, bench harness | done |

## Milestone 1 — visual fidelity (next)

Goal: a still frame that is hard to distinguish from the original at the same camera.

1. **Autotiling** — port `WorldGen.TileFrameCosmetic` + merge groups and per-type frame
   selection, including randomized variants (`frameNumber`) and `TileFrameImportant`.
   *3 600 lines of C# to port, ~10–20 agent-days.* ← biggest visual win
2. **Lighting** — tile light scanner, per-corner colours, `Lighting.GetColor`, torch/glow
   sources, day/night sky light. *~6–12 agent-days.*
3. **Liquids** — water/lava/honey/shimmer surfaces, waves, opacity blending.
   *~4–8 agent-days.*
4. **Backgrounds & sky** — parallax layers, sun/moon/stars/clouds, biome backgrounds.
   *~5–10 agent-days.*
5. **Tile objects** — `TileObjectData` (multi-tile furniture, doors, chests), paint,
   coatings, slopes/half-blocks, gemspark/glow. *~10–20 agent-days.*

Estimated total: **~35–70 agent-days**. Exit criterion: side-by-side screenshots of the same
world positions match texture-for-texture.

## Milestone 2 — player

Physics (`Player.cs` subset), collision, input mapping, animation state machine, draw layers
for the default outfit, camera follow, spawn/respawn, simple HUD. **~15–30 agent-days.**

## Milestone 3 — items, inventory, crafting, UI

Item use logic (subset first), inventory grid, hotbar, chests, crafting query, tooltips,
prefixes, equipment slots, mouse interaction, drag/drop. Full item parity is ~50 000 lines;
a playable subset (tools, weapons, blocks, potions, ~200 items) is **~40–80 agent-days**.

## Milestone 4 — NPCs, combat, projectiles

Damage model, knockback, immunities, buffs/debuffs; projectile system (subset), a first
batch of enemies (slimes, zombies, demon eyes, skeletons), spawn logic, town NPCs + shops.
**~60–120 agent-days.** Full 780-NPC / 1 100-projectile parity: several times that.

## Milestone 5 — world generation

Port generation passes and structures (`WorldGen.cs` + `Generation/*`). Alternatively keep
loading existing `.wld` files, which already works. **~40–90 agent-days** for a passable
generator; full parity much more.

## Milestone 6 — audio, map, networking, polish

XACT bank playback (`xsb`/`xwb`), music, 3D-ish attenuation; minimap/full map; exact
1.4.5.8 network protocol for multiplayer; localization; settings; save writing.
**~40–90 agent-days.**

## Aggregated

| Target | Approx. effort |
| --- | --- |
| Convincing single-player sandbox (M1–M3 subset) | 3–6 months of agent time |
| Broad single-player parity (M1–M6) | 1.5–3 years of agent time |
| Full 1:1 including multiplayer & all content | 3+ years, realistically a team |

These numbers assume the decompiled sources stay available and the port stays mechanical.
They do **not** assume the port is easy: the content surface is the bottleneck, not the
engine.

## Recommended order of attack

1. M1.1 autotiling → immediate, visible payoff.
2. M1.2 lighting → the world stops looking flat.
3. M2 player + M1.4 parallax → it starts to feel like a game.
4. M3 inventory + block placement/mining → the core loop.
5. M4 one enemy family + one boss → combat loop.
6. Everything else.

## Non-goals for now

- Modding API, tModLoader compatibility.
- Mobile/console parity.
- Exact network compatibility (single-player first).
