# Roadmap — Terraria-Rust (1.4.5.8)

> **Rewrite of 2026-11-12, based on a full C#→Rust audit of the decompiled 1.4.5.8
> sources.** Every row in the task catalog below was produced by reading the
> decompiled C# and the Rust port side by side; nothing here is inherited from the
> previous roadmap's status marks. The previous file (the per-subsystem atlas and
> the legacy task rows) is preserved verbatim as [`docs/ATLAS.md`](ATLAS.md); where
> this file and the atlas disagree, **this file wins** — it is the audited view.

Audit base commit: `ad0af53` (research ran against `f0a4937`…`ad0af53`).
Research corpus: [`docs/research/`](research/) — 42 module reports, one per subsystem.

---

## How tasks work (branch-per-task, unchanged)

Tasks live in the **Task catalog** below. Coordination lives in **git branches**:
the branch list *is* the live record of who is working on what. Full loop:
sync → branch + worktree → work → re-sync → verify → merge by ref update →
refresh the graph → delete the branch. The rules, the incident history and the
mandatory safety checks live in [`AGENTS.md`](../AGENTS.md) — read it before taking
a row.

Every task in the catalog carries: a **size** (S/M/L), a **severity**, the
**evidence** (C# and Rust citations), its **dependencies**, and its
**verification** method. The evidence column is the audit's contribution: it says
*why* the task exists, with both-side line references, so the implementer never
has to re-derive the claim.

## Legend

| Mark | Meaning |
| --- | --- |
| ✅ | ported and **re-verified by the audit** against the C# source |
| 🟡 | partially ported — the audit found a concrete divergence |
| ❌ | missing / not ported |
| ⛔ | out of scope (see [Non-goals](#non-goals-updated-2026-11-12)) |
| S / M / L | task size — S ≈ ≤1 day, M ≈ 1–3 days, L ≈ 3–10 days |
| D / B / P / X | severity — D = determinism, B = behavioural, P = pixel, X = structural/coverage |

---

## Provenance of this roadmap

The previous roadmap's status marks were **claims**. On 2026-11-12 a full audit
ran over the decompiled C# (`decompiled/`, 1.4.5.8) and the Rust port
(`crates/`), module by module, with every module re-checked against source:

- **42 module reports** in `docs/research/NN-<slug>.md`, one per subsystem, each
  written from source reads (not from memory), each with: scope, key entities,
  dependencies (calls into / called by / shared data), a port-status table,
  the covering tests, severity-graded deviations, and sized task seeds.
- **One dedicated re-verification pass** (`docs/research/40-port-done-verification.md`)
  re-checked 32 roadmap rows marked DONE against the actual code:
  **27 PASS · 3 PARTIAL · 1 FAIL · 2 STALE**. No phantom ports were found —
  every cited Rust symbol exists — but several DONE rows were overstated.
- **A consolidation pass** (`docs/research/90-consolidated-findings.md`) merged
  every task seed into one register, separated the deviations that the existing
  `docs/ANALYSIS.md` / `docs/PORT_MAP.md` already admit from those they do not,
  and listed the doc claims the audit disproved.

Rules that were applied, and that the next auditor must keep applying:

1. **A DONE row is a claim, not a fact.** Re-read the Rust and the cited C#
   before trusting it; the four worst defects found (2× item pickup count,
   the wrong hover preset, the projectile kill timing, the `frameNumber`
   policy) were all *behind* a DONE mark.
2. **Citations are mandatory.** A task without a C# `file:line` and a Rust
   `file.rs::sym` citation is not ready to be taken.
3. **"Not found" is a result.** Subsystems with no Rust counterpart are recorded
   as MISSING with the C# call-site census — that census is the size estimate.
4. **Determinism is the hardest constraint.** Any change to RNG streams, the
   fixed-step order or the G6 `WorldFrameKey` needs byte-equality proof.

---

## Audit scoreboard — 42 subsystems

Verdict = the audit's judgement of the subsystem as a whole, not of a single row.

| # | Subsystem | Verdict | Headline gap (audit) | Report |
| --- | --- | --- | --- | --- |
| 01 | Main update loop / time / camera | 🟡 | three C# order inversions in the world branch; the clock runs with a menu open; camera is lerped in the player step instead of at draw time | [01](research/01-main-update-loop.md) |
| 02 | Main draw pipeline / targets / effects | 🟡 | no layer-target family, no `skyTarget`/capture gating, 5 screen-effect rows missing, `CaptureManager` absent | [02](research/02-main-draw-pipeline.md) |
| 03 | Player core (movement/life/mana/death) | 🟡 | in-liquid gravity uses ÷3 for *every* liquid instead of the C#'s per-liquid rebase; wet `jumpHeight`/`jumpSpeed` missing → water jumps 2× too weak | [03](research/03-player-core.md) |
| 04 | Player combat (hurt/damage/item use) | 🟡 | the whole dodge family is absent; projectile→NPC hits skip `DamageVar` **and** defence; the `num2 < 1 → 1` floor is missing | [04](research/04-player-combat.md) |
| 05 | NPC core (defaults/AI dispatch/net) | 🟡 | 182 kinds against 697 C# ids; `behindTiles` has no field; hover presets ship the wrong type's constants | [05](research/05-npc-core.md) |
| 06 | Projectile core | 🟡 | AI dispatch covers 32 of 210 `aiStyle`s; `timeLeft`/`penetrate` kills run once per update instead of per AI sub-step (extra-update projectiles live ~2× too long); PVE hit test is a point, not a rectangle | [06](research/06-projectile-core.md) |
| 07 | Item core (defaults/prefixes/tooltips) | ✅ | `SetDefaults` is complete (6 196 = `ItemID.Count`) and the prefix engine is faithful — but the pickup count is added **twice**, and the price math bypasses `GetStoreValue` | [07](research/07-item-core.md) |
| 08 | Tile / framing / TileObjectData | 🟡 | `frameNumber`/`wallFrameNumber` come from a position hash instead of `genRand`/the save (determinism + pixel); `SelfFrame8Way` and 4 `WallFrame` gates missing; 36 hand-written `TileObjectData` defs against a 5 328-line C# registry | [08](research/08-tile-framing.md) |
| 09 | TileDrawing / DrawData / paint | ❌ | 1 of 17 C# draw passes fully ported (6 partial, 10 missing); the `DrawData` table covers ~113 of ~376 cases; chests, vines, ropes, animated tiles and particles are not drawn | [09](research/09-tile-drawing.md) |
| 10 | Lighting / TileLightScanner / LightMap | 🟡 | the engine core and the scanner seeds are faithful, but **300+ C# `AddLight` entity-light call sites are unported** and the 1 442-line legacy engine is approximated | [10](research/10-lighting.md) |
| 11 | Liquid / buffer / types | 🟡 | water flow is 1:1, but the non-water surface is absent (lava/honey/shimmer delays, merge tiles, evaporation); `skipLiquid` missing; the pass window changed by ~10× for small pools | [11](research/11-liquid.md) |
| 12 | WorldGen / WorldBuilding passes | 🟡 | the ~110-pass pipeline is largely ported, but special-seed pruning, several biomes and the genRand interleave diverge | [12](research/12-worldgen.md) |
| 13 | Collision / HitTile / scans | 🟡 | the movement collision is 1:1, but `HitTiles`, `StickyTiles`, `SwitchTiles`, `AdvancedTileCollision` and `DrownCollision` are missing; wet flags read one tile instead of a box scan | [13](research/13-collision.md) |
| 14 | Map / MapHelper / minimap | 🟡 | colour tables and `.map` IO are ported, but the reveal/queue/edge logic and the overlay layers are only partial | [14](research/14-map-minimap.md) |
| 15 | Dust / Gore | ❌ | **the module does not exist**: 3 279 dust call sites across 36 C# files → 3 ported; 1 643 gore sites → 1; no `Dust`/`Gore` type, no pool, no per-type table | [15](research/15-dust-gore.md) |
| 16 | Weather / rain / clouds / scene metrics | 🟡 | rain and clouds are ported; the ambience/event scheduling and the scene-metric counters that gate spawning are partial | [16](research/16-weather.md) |
| 17 | Netcode (messages/sockets/Netplay) | 🟡 | the stack is real (TCP transport, frame codec, both state machines) but only 25 of 163 C# message ids are ported; `InitialSpawn` renamed with wrong semantics; prediction smoothing is port-invented | [17](research/17-netcode.md) |
| 18 | UI framework (UIElement/UIState) | 🟡 | layout + pointer machines are faithful pure ports, but ~40 element classes and 17 `UIState` screens have no counterpart; no event bubbling, no scissor, UI scale pinned to 1.0 | [18](research/18-ui-framework.md) |
| 19 | Front-end screens | ✅ | the front end is 1:1 (16 of 17 state hops PASS) — only the loading screen, the cloud UI and the creator's Randomize/Copy/Paste are missing | [19](research/19-ui-screens-frontend.md) |
| 20 | In-game UI (inventory/chest/craft) | 🟡 | right-click moves **half a stack** where the C# moves exactly one item per transfer; equip/dye/shop/Journey actions missing; the crafting UI is one click deep | [20](research/20-ui-ingame.md) |
| 21 | Recipes / crafting | 🟡 | see [21](research/21-recipe-crafting.md) | [21](research/21-recipe-crafting.md) |
| 22 | Chests / loot / drops / shops | 🟡 | containers and the shop dispatcher are ported, but world-gen chest loot uses a load-time splitmix seam instead of the C# genRand interleave; 13 drop rows skipped; the buy-price formula omits `buyOnce/5` and the Discount Card | [22](research/22-chest-loot-shops.md) |
| 23 | Wiring / tile entities | 🟡 | see [23](research/23-wiring-tile-entities.md) | [23](research/23-wiring-tile-entities.md) |
| 24 | Spawner / invasions / events | 🟡 | the spawn ladders are ported, but the town-NPC `Main.rand` block is unported, the `invaders` rate override is missing, and Pumpkin/Snow Moon have no wave rosters; Lantern Night, Birthday Party and the Sandstorm are absent | [24](research/24-spawner-invasions.md) |
| 25 | NPC AI families | 🟡 | the C# implements **128 `aiStyle` arms (0..127)** over 587 NPC types; the port has 44 ported, 9 partial, 6 approximate and **69 missing** — and only six vanilla families share a Rust archetype, each a core extraction rather than the family (`aiStyle 3` = 155 types and `aiStyle 7` = 52 types run as a generic walker/wander) | [25](research/25-npc-ai-audit.md) |
| 26 | Projectile AI families | 🟡 | 32 of 210 `aiStyle`s dispatched, with **6 duplicated unreachable arms** (module 06 counted 5); the missing styles cover every homing/spread/pierce/boomerang projectile | [26](research/26-projectile-ai-audit.md) |
| 27 | Mounts / pets / vehicles / grapple | 🟡 | the mount stat table is 1:1 for 50 of 66 ids and the minecart/grapple families are genuinely ported, but three C# activation rules are missing and `SetMount`/`Dismount` lose per-type extras | [27](research/27-mounts-pets-vehicles.md) |
| 28 | Buffs / debuffs | 🟡 | the slot engine is 1:1 (44 slots, the immunity table, the `BuffID.Sets` rules), but the **effect table is ~15 %** (≈40 of ~290 ids) and the buff HUD is entirely absent | [28](research/28-buffs-debuffs.md) |
| 29 | Town NPCs (housing/mood/pylon) | 🟡 | the room check, the mood and the pylon core are ported, but `ScoreRoom`/home picking, housing occupancy, mid-game relocation and the whole chatter/dialogue layer are missing — town NPCs are silent | [29](research/29-town-npc-housing.md) |
| 30 | Achievements / bestiary | 🟡 | 139 of 180 conditions (lossy `CreateMany`), only 4 of 10 condition kinds have notify sites, `achievements::save` has **no call site** (contradicting a DONE row), and the bestiary unlock ladder is wrong for 164 of 172 entries | [30](research/30-achievements-bestiary.md) |
| 31 | Golf / ball physics | ❌ | the subsystem is **100 % absent** — no ball physics, no course data, no UI; the `StockWhen::GolfScore` shop tiers can therefore never unlock | [31](research/31-golf-ball-physics.md) |
| 32 | Sound / audio engine | 🟡 | the XACT music engine, the attenuation maths and the music ladder are ported, but only **13 of 1 411** C# `PlaySound` call sites play, there is no pitch at all, no instance pool and no tracked-sound layer | [32](research/32-sound-audio.md) |
| 33 | Localization | 🟡 | see [33](research/33-localization.md) | [33](research/33-localization.md) |
| 34 | ID sets (`Terraria.ID.*`) | 🟡 | `ItemID`/`TileID`/`WallID`/`BuffID` counts match, but `PrefixID` covers 78 of 97 ids, `ProjectileID` 213 constants, and `DustID`/`SoundID`/`ArmorIDs` have no counterpart; **7 protocol-id collisions** cause round-trip loss on live save and netcode paths | [34](research/34-id-sets-audit.md) |
| 35 | RNG & utilities | 🟡 | the C# `UnifiedRandom` is .NET `System.Random` (not xoshiro) and has **no port**; the global stream is replaced by 7 ad-hoc `u64` fields; only `FastRandom` is an exact-stream port; the "FNV proof" pins self-consistency only, not C# parity | [35](research/35-rng-utils.md) |
| 36 | Cinematics / film / Old One's Army | 🟡 | the film framework is **dead code in vanilla** (0 call sites) — parity is trivial; the OOA wave rosters are byte-identical, but the loss path, victory scene, medals, crystal drops and tier persistence are missing | [36](research/36-cinematics-film.md) |
| 37 | ReLogic (content pipeline/XNA) | 🟡 | the Rust re-implements the XNB (LZX, not zlib) and XACT formats, but 23 retail files still fail to load; the asset-repository/async-loading model, content sources and text wrapping are not ported | [37](research/37-relogic-libs.md) |
| 38 | Out-of-scope confirmation | ✅ | `Social/*` (72 files) is correctly excluded and documented; the **undocumented** absences are `Terraria/Testing/**` (34 files, incl. the vanilla's `RecordReplay`/`LockstepDebug` harness), `TimeLogger`, the crash handlers and dedicated-server mode | [38](research/38-out-of-scope-audit.md) |
| 39 | Port architecture inventory | 🟡 | **the "god-files are gone" claim is false for the entity half**: 8 files exceed 5 000 lines; `App` has 257 fields (docs say 118); no spatial hash (15 linear scans), no SoA, no event buffers | [39](research/39-port-architecture.md) |
| 40 | DONE-row re-verification | ✅ | 32 rows re-checked: 27 PASS · 3 PARTIAL · 1 FAIL · 2 STALE. The one FAIL is `M4-NPC-4` (hover presets) | [40](research/40-port-done-verification.md) |
| 41 | Render pipeline vs the C# graphics layer | 🟡 | the atlas + instanced pipeline holds, but one blend state cannot express the C#'s additive/multiiply batches, `pow(tint,2.2)` is not the sRGB EOTF, there is no scissor, and `MoonLordShake` is inert | [41](research/41-render-pipeline.md) |
| 42 | Save/load (IO) + compression hooks | 🟡 | the 11-section `.wld` layout matches, but the writer **drops the four wire bits** (and diverges on RLE boundaries), the header writes placeholders, and no golden fixture compares against a C#-written file | [42](research/42-save-load-audit.md) |

**Scoreboard roll-up:** 4 subsystems ✅ (items, front end, out-of-scope, DONE-verification),
2 ❌ (tile drawing, dust/gore) plus golf ❌ in the catalogue, 36 🟡.
The 🟡 verdicts are mostly *small, located* gaps rather than absent systems —
which is why Stage 1 below is a catalogue of concrete, sized tasks rather than
a rebuild.

---

# Task catalog

The catalog is grouped by **theme**, not by milestone: the audit found that the
remaining work is dominated by a handful of cross-cutting problems (determinism,
tick order, AI dispatch, missing subsystems), and milestone buckets hid those.
Every row is claimable as one branch-per-task (AGENTS.md).

Severity: **D** determinism · **B** behavioural · **P** pixel · **X** structural/coverage.
`deps` = other ids in this catalog. `verify` names the proof the task must carry.

## Stage 1A — Determinism & RNG

The single largest risk area: the port's random streams are not the C#'s, and the
worldgen "proof" only pins self-consistency. Everything that touches RNG is
frozen until A1–A3 land.

| id | Task | Size | Sev | Evidence | deps | Verify |
| --- | --- | --- | --- | --- | --- | --- |
| S1-A-1 | Port the C# `UnifiedRandom` (a .NET `System.Random` — Knuth subtractive, 56-int seed array) as `tr_core::UnifiedRandom`; today it has **no port** (`tr-core/src/extract.rs:530`, `scene.rs:528` admit it) | L | D | research/35-rng-utils.md → RNG sources; `decompiled-libs/ReLogic/Utilities/UnifiedRandom.cs` | — | byte-identical stream vs the C# over a fixed seed vector (golden test) |
| S1-A-2 | Replace the 7 ad-hoc `u64` fields with the C#'s **named per-consumer streams** (`Main.SwapRandom` creates 13 keyed streams — `Main.cs:3204-3215`); `WorldGen.genRand` is an alias of `Main.rand` (`WorldGen.cs:4445`) | M | D | research/35; `Main.cs:3204-3215` | A1 | per-stream golden vectors; no shared stream consumed in a different order |
| S1-A-3 | Tile framing: take `frameNumber`/`wallFrameNumber` from `genRand`/the save, not from a **position hash** (`framing.rs:138/1097`) | M | D+P | research/08-tile-framing.md; `framing.rs:138` | A1, A2 | regenerate a world twice → identical tile frames; golden `.wld` frame comparison |
| S1-A-4 | Chest loot: roll at **generation** time on the worldgen RNG stream, not at load time from a splitmix seam (`chest_gen.rs`), restoring the vanilla genRand interleave and the cross-chest fairness flags | M | D | research/22-chest-loot-shops.md | A1, A2 | same seed → same chest contents as the C# (golden world) |
| S1-A-5 | Prefix rolls must use `UnifiedRandom`, not the splitmix stand-in (`prefix.rs`); also the loot/shop/particle streams that inherited the stand-in | S | D | research/07-item-core.md | A1 | prefix distribution matches the C# over many seeds |
| S1-A-6 | Lighting water-jitter must use `FastRandom.WithModifier(i).Next(98,100)` — the port hashes the index (`LightMap.cs:219` vs the Rust) | S | P | research/10-lighting.md | — | golden water-tile frame sequence |
| S1-A-7 | Worldgen parity proof: replace `deterministic_for_seed` (self-consistency) with a **comparison against a C#-generated world**; the FNV-ish hash is the port's own construction with constants re-pinned ~20× | L | D | research/35; `worldgen/mod.rs:259` | A1, A2 | byte-identical tile/chest/NPC sections vs a C# world file |
| S1-A-8 | Liquid: restore the C#'s 2 500-slot active-list pass window; the port slices `ceil(len/10)` → ~10× different update frequency for small pools | M | D | research/11-liquid.md | — | lake settling reaches the C#'s equilibrium in the same number of ticks |
| S1-A-9 | Projectile RNG/kill ordering: `timeLeft--` and `penetrate == 0` must run **per AI sub-step** (C# `:17201/:17206` inside the `while` at `:16676`), not once per update — today extra-update projectiles live ~2× too long | M | D+B | research/06-projectile-core.md; `projectile.rs:9079-9090` | — | golden projectile lifetimes per `extraUpdates` |
| S1-A-10 | Freeze the RNG-order contract: any task touching a stream must state which stream it consumes and in which order (add to the task template) | S | D | research/35 → task seeds | — | review checklist in `docs/ANALYSIS.md` |

## Stage 1B — Tick order & the main loop

The Rust world branch inverts three C# orders, runs the clock with a menu open,
and lerps the camera inside the player step. Order is the classic 1:1 trap: the
C#'s reference order is `Main.cs:17744-17822`.

| id | Task | Size | Sev | Evidence | deps | Verify |
| --- | --- | --- | --- | --- | --- | --- |
| S1-B-1 | Restore the C#'s update order in the Rust world branch: items **before** projectiles (`Main.cs:17977`/`18031`), `UpdateTime` after dust/floating text (`:18083`), and the WorldGen/wiring/growth/tile-entity neighbours (`:17744-17822`) | M | B+D | research/01; `app/frame.rs:328-557` | — | order-pinning test (a trace of phase names per tick) |
| S1-B-2 | Gate the clock: the port advances `Main.time` with a menu open; the C# returns early (`Main.cs:17166-17175`) | S | B | research/01; `world_sim/clock.rs::advance_time` | B1 | day/night does not advance on the front end |
| S1-B-3 | Port the pause gates: `CanPauseGame`, `DoUpdate_WhilePaused`, `gamePaused`, and the `ShouldUpdateEntities`/`WorldUpdateStepper` split — all absent | M | B | research/01 | B1 | paused world freezes entities but keeps the UI live |
| S1-B-4 | Move the camera update to draw time (`DoDraw_UpdateCameraPosition`, `Main.cs:61832`, `:62590-62649`); add `CurrentPan`, `cameraLerp` and the world clamp | M | B+P | research/01; `player.rs:334/666` | — | camera position identical to the C# at the same player state |
| S1-B-5 | Port `UpdateViewZoomKeys` (`Main.cs:16801-16815`) and keep the vanilla 1.0–2.0 clamp (today `TR_ZOOM` widens it) | S | B | research/01 | — | zoom clamp test; `TR_ZOOM` only in debug |
| S1-B-6 | `gfxQuality` budgets (dust/liquid/light) have no analogue — port the quality gates | S | B | research/01 | — | low-quality mode reduces the same counts as the C# |
| S1-B-7 | Draw phase order vs the C#'s 17 phases (`Main.cs:61707-62507`) — one phase function today (`frame.rs::frame`, 1 824 lines) | M | P+B | research/02; research/09 | — | phase-trace test + screenshot diff |
| S1-B-8 | Split `app/frame.rs` into named phases (it is a 1 824-line single function) — architecture, required before Stage-2 | M | X | research/39 | — | phase modules; no behaviour change (screenshots equal) |

## Stage 1C — Combat & damage

| id | Task | Size | Sev | Evidence | deps | Verify |
| --- | --- | --- | --- | --- | --- | --- |
| S1-C-1 | Port the **dodge family** — black belt / shadow dodge / mystic sash / brain / no-dodge-reductions — no dodge field exists anywhere in `crates/` | M | B | research/04; `Player.cs:38197-38567` vs `player.rs:2255-2295` | — | dodge chance test per source |
| S1-C-2 | Projectile→NPC damage must apply `DamageVar` **and** the defence term (`Projectile.cs:12938`); today `projectile.rs:9703`→`npc.rs:4062` skips both | S | B | research/04 | — | damage-parity test vs the C# formula |
| S1-C-3 | Restore the `num2 < 1 → 1` floor (`Player.cs:38378-38381`) and the resulting 40-vs-20 iframes; a current test asserts the divergence | S | B | research/04; research/40 | — | floor test; delete the divergence-pinning test |
| S1-C-4 | `GetWeaponDamageMultiplier`: port the bow/gun/specialist arms (`Player.cs:53396-53398`); and move the truncation back before the ammo fuse | M | B | research/04 | — | per-class damage table test |
| S1-C-5 | NPC→player contact damage must apply `DamageVar(..., -luck)` | S | B | research/04 | C2 | contact-damage test |
| S1-C-6 | `behindTiles` (`NPC.cs:10355`) — the field does not exist in `crates/` | S | B+P | research/40; research/05 | — | NPC draws behind walls correctly |
| S1-C-7 | Port the remaining hurt-pipeline modifiers: retaliate, magic cuffs, paladin/beetle/solar, dismount-on-hit, net, combat text, death-by-reason | L | B | research/04 | — | per-modifier test |
| S1-C-8 | Projectile PVE hit test must use `Damage_GetHitbox()`'s rectangle, not a slot-centre point (`Projectile.cs:12499` vs `projectile.rs:9574`) | M | B | research/06 | — | hitbox-overlap test (large targets) |
| S1-C-9 | Item pickup counts the stack **twice** (`world_sim/items.rs:191-194` adds `remaining` and then `drop.count`) — every stack pickup grants 2× | S | B | research/07-item-core.md | — | pickup test asserting the exact count |
| S1-C-10 | Right-click must move **exactly one item** per transfer, with the `stackSplit > 1` pacing and held-repeat (the port moves half the stack: `clicks.rs:911/238/813`) | M | B | research/20-ui-ingame.md | — | right-click transfer test + held-repeat |
| S1-C-11 | `GetStoreValue` price math (`Item.cs:49901`): the instance/prefix `value²` is bypassed and the `buyOnce`/discount arms are missing (`shop.rs::buy_price`/`sell_price`) | M | B | research/07; research/22 | — | price table vs the C# |
| S1-C-12 | Equipment/accessory stat modifiers: the `UpdateEquip` hook chain (`Player.cs:12948-12954`, `:18365-18366`) and the per-slot bonus table | L | B | research/04; research/27 | — | per-accessory modifier test |

## Stage 1D — Entity AI: the dispatch problem

The C# dispatches AI on `aiStyle`; the port dispatches on `NpcKind` and
reproduces the *effects* through six shared archetypes. That keeps behaviour
plausible but cannot express style-specific rules.

| id | Task | Size | Sev | Evidence | deps | Verify |
| --- | --- | --- | --- | --- | --- | --- |
| S1-D-1 | Fix the hover presets — `HoverParams::HORNET` ships type 205's 3.25/0.018 instead of type 42's 3.5/0.021, the `2f - scale` factor is skipped, and the ×0.97 damping is applied to every hover kind (`npc_ai.rs:177-181` vs `NPC.cs:50341-50344`) — **this is the one FAIL of the DONE-row re-verification** | S | B | research/40; research/25 | — | hover-parameter table test; repoint the four wrong C# anchors (`:50322`=type 619, `:50328`=type 252) |
| S1-D-2 | Introduce an `aiStyle` axis on the NPC side: dispatch on the C#'s style id first, then on the kind — so a style can be ported once for all kinds that use it | L | X+B | research/25 → "the family machine"; research/39 | — | per-style dispatch table matches the C#'s branch list |
| S1-D-3 | Port the missing NPC AI families — **69 MISSING + 6 APPROXIMATE + 9 PARTIAL arms of the C#'s 128** (research/25's centrepiece table). Highest leverage first: the `aiStyle 3` per-type speed table (155 types), the `aiStyle 7` family (52 types), the town-NPC family, and the Frost Legion trio (the port runs the skeleton's speed 0.8 where the C# uses 3/3.5/4) | L | B | research/25 → the 128-arm table | D2 | per-family behaviour test + screenshot |
| S1-D-3b | Fix the mis-routed NPC kinds: the still-shadowed goblin arm, `GoblinSummoner` (routed to the aiStyle-8 caster), and `AncientCultist` (claims `NPCID.CultistBoss` 439 but carries type 422's aiStyle-94 row) | S | B | research/25 → deviations | — | per-kind aiStyle table test |
| S1-D-4 | Projectile AI: port the missing `aiStyle`s (32 of 210 covered) and delete the **6 duplicated unreachable arms** (aiStyle 1, 3, 14, 16, 115 — module 06 counted 5, module 26 found the 6th) | L | B | research/26; `projectile.rs:8624/8631/8695/8696/8732` | — | `#[deny(unreachable_patterns)]` on the dispatch; per-style test |
| S1-D-5 | Projectile families that have no faithful AI (homing/spread/pierce/boomerang/exploding) — a task per family | L | B | research/26 → "which kinds therefore have no faithful AI" | D4 | per-family golden trajectory test |
| S1-D-6 | Boss part chains (`realLife`): the worm/body chains, the Moon Lord carrier and the DD2 anchors | M | B | research/05 | — | chain integrity test (no orphaned segments) |
| S1-D-7 | Replace the `npc_ai.rs::line_clear` stand-in (used by ~17 NPC sites) with the C#'s `CanHitLineOfSight` walk; the port's LOS path is dead code | M | B | research/13 | — | LOS parity test |
| S1-D-8 | NPC separation / target selection must read the **read-only world** (Stage-2 prerequisite: the port scans `npcs.iter()` 15 times per tick) | M | X | research/39 | — | spatial hash introduced; scan count constant |

## Stage 1E — Missing subsystems (the ❌ rows)

| id | Task | Size | Sev | Evidence | deps | Verify |
| --- | --- | --- | --- | --- | --- | --- |
| S1-E-1 | **Create the dust/gore module** (`tr-core::particles`): the C# has 3 279 `Dust.NewDust*` call sites across 36 files and 1 643 `Gore.NewGore*` sites across 13; the port has 3 files and 1 file, with no `Dust`/`Gore` type, no pool (6 000 dust / 600 gore slots), no per-type table | L | B+P | research/15-dust-gore.md → caller-coverage table | — | per-call-site coverage table; pool cap test |
| S1-E-2 | Dust per-type table: colour, gravity, fade, `noGravity`, light, velocity scaling — from `DustID` + the C#'s per-type if-chain (`Dust.cs:477` update order) | M | B+P | research/15 | E1 | per-type table test |
| S1-E-3 | Dust update order: guards → `maxDustToDraw` kill → per-type scale/light → `position += velocity` → `AddLight`/`customData` → collision → per-type chain → gravity tail → screen-bottom kill → `dCount` nibbles → scale floor | M | D+P | research/15 | E1 | golden dust frame sequence |
| S1-E-4 | Gore: `NewGore`, the sticky/timeleft/fade update, and the drop sites (NPC death 1 345 sites) | M | B+P | research/15 | E1 | gore lifecycle test |
| S1-E-5 | **Entity lights** (`AddLight`): 300+ C# call sites (Projectile 107, Dust 94, NPC 80, Player 46, WorldItem 69) are unported; the Rust only feeds the waterfall `DynamicLight` | L | P | research/10-lighting.md | — | per-entity light contribution test |
| S1-E-6 | Buff HUD: `Main.DrawBuffIcon` (`Main.cs:43307`) + `TryRemovingBuff` (`:43370`) — icons, alpha fade, duration text and right-click removal are entirely absent | M | P+B | research/28-buffs-debuffs.md | — | HUD screenshot + removal test |
| S1-E-7 | Buff **effect table**: ~40 of ~290 ids are ported (`Player.UpdateBuffs` has 251 arms over `Player.cs:9855-12069`) | L | B | research/28 | — | per-buff effect test; coverage table |
| S1-E-8 | Nurse rules: the protected-debuff list and the over-charge | S | B | research/28 | — | nurse test |
| S1-E-9 | **Golf subsystem**: ball physics (`Terraria/Physics/BallCollision.cs`, 368 lines; gravity 0.3, drag 0.99, `AbsoluteStrength = ovalLen*32`, hole capture on tile 476, score = distance/16 ÷ (hits+2)), the course data and the UI | L | B+X | research/31-golf-ball-physics.md | — | shot/golf-physics test; unlocks the `StockWhen::GolfScore` shop tiers |
| S1-E-10 | Town NPC chatter/dialogue: `NPC.GetChat`, `ConditionalDialogue` and the ten chatter tables — town NPCs are silent in the port | M | B | research/29 | — | chatter selection test |
| S1-E-11 | Town NPC housing: `ScoreRoom`/best-home picking (`scan_spot` returns the scan origin), `housingCategory` occupancy, `GetHouseholdStatus`, mid-game relocation (`MoveTownNPC`/`moveRoom`), `lookForHomeTimeout` | M | B | research/29 | — | relocation + occupancy test |
| S1-E-12 | Old One's Army: the **loss path**, the victory scene, wave-break skip, medals, crystal drops, arena rules, entity wipes, tier persistence and the per-tier kill-worth table (today a constant 1.0) | M | B | research/36-cinematics-film.md | — | OOA run test (both outcomes) |
| S1-E-13 | Collision family: `HitTiles` (`:3063-3108`), `StickyTiles` (`:3362-3435`), `AnyHurtingTiles`/`CanTileHurt`, `SwitchTiles`, `BuildTileContacts`, `noSlopeCollision`, `AdvancedTileCollision`, `DrownCollision`/`GetWaterLine` | M | B | research/13-collision.md | — | per-function parity test |
| S1-E-14 | Drop rules: 4 rule classes (`SlimeBody`, `StatueMimic`, `MechBossSpawners`, `FromOptionsWithoutRepeats`) + 2 conditions skipped of 26 classes / ~50 gates | M | B | research/22 | — | per-rule-class test; coverage table |
| S1-E-15 | `ReportDroprates` + the bestiary providers (town-NPC/critter chat/sight), filters, sorting, flavour and the MP kill-sync | M | B | research/30 | — | command output snapshot |
| S1-E-16 | Screen effects: `ScreenDarkness`, `ScreenObstruction`, `MoonlordDeathDrama`, `PaladinsShield`, `InfernoRings`, the lens flare — 20 inert filter rows | M | P | research/02; research/41 | — | effect screenshot per filter |
| S1-E-17 | `CaptureManager` + the loading screen (`UIWorldLoad`/`UIGenProgressBar`) | M | P+B | research/02; research/19 | — | capture + loading-screen screenshot |
| S1-E-18 | Achievements: wire `achievements::save` (it has **no call site** — contradicting the DONE row), port the lossy `NPCKilledCondition.CreateMany` conditions, and give all 10 condition kinds notify sites (only 4 have them) | M | B | research/30; research/40 | — | save/reload test; per-condition completion test |
| S1-E-19 | Legacy lighting engine (1 442 lines) — the White/Retro/Trippy styles are only style transforms today | L | P | research/10 | — | per-mode screenshot |
| S1-E-20 | Item tooltips: the per-item `ToolTip` line table (damage/speed/knockback text is still a placeholder), the player-side tooltip layers, the wider equipable gate and the stack-suffixed hover name | M | P | research/07 | — | tooltip table test + screenshot |
| S1-E-21 | Sound call sites: only 13 of 1 411 C# `PlaySound` sites play — extend the bank in waves (item use → NPC hits → tile break → footsteps → ambience) | L | B+P | research/32-sound-audio.md | — | per-category cue coverage table |
| S1-E-22 | Sound pitch: the legacy `Next(-10,11)*0.01` and `SoundStyle.GetRandomPitch` — there is **no pitch at all** today | M | B+P | research/32 | — | pitch distribution test |
| S1-E-23 | Sound instance pool + the tracked-sound layer (`SoundPlayer`/`ActiveSound`/`SoundStyle`); the C# stops and recreates per family | M | B | research/32 | — | instance-count test |
| S1-E-24 | Per-style volume (`WithVolumePerceived`), variation families, and the `play_sound` guards (worldgen/server) | S | B | research/32 | — | guard test |
| S1-E-25 | Ambience: the critter and liquid-merge tables have no caller; thunder pitch/slot gaps | S | B | research/32 | — | emitter wiring test |
| S1-E-26 | Chat/emote/achievement UI hooks that the framework gap leaves unreachable (module 18's ~40 element classes, ~17 `UIState` screens) — one task per screen family | L | X+B | research/18-ui-framework.md → gap list | — | per-screen parity checklist |
| S1-E-27 | MP: the missing message families — chest (31-33), sign (46/47), wiring (59), buffs (50/53/54), ping (154) and the `5 PlayerInventory` packet (silently dropped) | L | B | research/17-netcode.md → message-id map | — | per-id round-trip test |
| S1-E-28 | The `InitialSpawn` semantics (renamed `WORLD_COMPLETE` with the wrong meaning) and the ping relocation to NetModule 2 | M | B | research/17 | — | join sequence test |

## Stage 1F — Data, id spaces & tables

| id | Task | Size | Sev | Evidence | deps | Verify |
| --- | --- | --- | --- | --- | --- | --- |
| S1-F-1 | **Fix the 7 protocol-id collisions** (`to_u16`/`from_u16` ids 78..84: pirates/probe vs Deerclops/Skeletron-Prime) — they cause round-trip loss on the live **save** (`saveglue.rs:107` ↔ `mod.rs:1980`) and **netcode** (`netglue.rs`) paths | M | D+B | research/34-id-sets-audit.md | — | save→load round-trip test over all 182 kinds; netcode round-trip test |
| S1-F-2 | Delete the dead/contradictory arms the DONE row claims are gone: 29 dead + 2 contradictory arms in `to_u16`, 21 + 7 in `from_u16` — and **undo the pirate renumber** that created the 78..84 collision | M | D | research/34 (M4-EVENT-4b FAIL); research/26 (6 duplicated arms) | F1 | no dead arm; `#[deny(unreachable_patterns)]`; bijection test on the id map |
| S1-F-3 | `cs_id` covers only 43 of 182 kinds — complete the vanilla `NPC.type` mapping used by the statue/wiring rosters | M | X | research/34 | F1 | per-kind `cs_id` table test |
| S1-F-4 | `PrefixID`: 78 of 97 ids ported (62..80 missing) | M | B+X | research/34; research/07 | — | prefix id coverage test |
| S1-F-5 | `ProjectileID`: 213 constants ported — complete the table or generate it | M | X | research/34 | — | id coverage test |
| S1-F-6 | Add the missing id tables: `DustID` (332), `SoundID` (506 styles), `ArmorIDs`, `PlayerItemSlotID` | M | X | research/34; research/32 | — | table + coverage test |
| S1-F-7 | `TileObjectData`: generate the registry (36 hand-written defs vs a 5 328-line C# registry); style alternation, dynamic `GetStyle`, `RandomStyleRange`, coordinate pickup | L | B+P | research/08-tile-framing.md | — | per-style def test; `RandomStyleRange` RNG-order test |
| S1-F-8 | `tiledata` covers 46 of 152 `TileID.cs` set call sites — complete the sets/merge/emissive tables | M | X | research/08 | — | set-coverage test vs the C# table |
| S1-F-9 | `DrawData` table: ~113 of ~376 C# case ids ported; add the tree-biome/variant probes and the animator (`tile_frame ≡ 0` today) | L | P | research/09-tile-drawing.md | — | per-case id coverage test |
| S1-F-10 | `MAX_STACK` 999 vs the C#'s `CommonMaxStack` 9 999 for unregistered ids; add the item `Clone`/`Refresh` instance snapshot | S | B | research/07 | — | stack test |
| S1-F-11 | Spawner: the town-NPC `Main.rand` block (`NPC.cs:799-923`) is unported even though the code comment claims it rides the spawner RNG (the `rng` is discarded at `spawner.rs:203`) | S | D+B | research/24-spawner-invasions.md | A1 | town-spawn RNG test |
| S1-F-12 | Spawner: the `invaders` rate override (`NPC.cs:781-785`) is missing — invasion spawns fire once per tick, uncapped, at a fixed `(x-600, y-400)` offset instead of `FindSpawnTile` | M | B | research/24 | — | invasion spawn placement test |
| S1-F-13 | Pumpkin Moon / Snow Moon have no wave rosters (`spawn_npc_at` never reads `moon_event`) | M | B | research/24 | — | wave roster test |
| S1-F-14 | Missing world events: Lantern Night, Birthday Party, the Sandstorm event and the cultist delay (grep-verified absent) | M | B | research/24 | — | per-event test |
| S1-F-15 | Bestiary unlock ladder: `kills_needed_for` returns 0 outside an 11-pair banner table (0 = "≥ full" → tier 4 at zero kills) — wrong for 164 of 172 entries; the C# default is 50 | M | B | research/30 | — | per-entry ladder test |
| S1-F-16 | Chest icon tables (`chestTypeToIcon`/`chestTypeToIcon2`/`dresserTypeToIcon`) and the shop `StockWhen` gates hard-false | S | B | research/22 | — | icon/gate test |
| S1-F-17 | Item `SetDefaults` table: 6 196 rows ✅ — freeze it with a generator round-trip test so it cannot drift | S | D | research/07 | — | generator diff is empty (the AGENTS.md pattern) |

## Stage 1G — Rendering & pixels

| id | Task | Size | Sev | Evidence | deps | Verify |
| --- | --- | --- | --- | --- | --- | --- |
| S1-G-1 | Port the 10 missing tile-draw passes: vines, ropes, animated tiles, tile-entity furniture, pylons/trophies/void lenses, particles, tile cracks, `DrawSpecialTilesLegacy`, chest frames (+0/38/76), platform slope side-fills | L | P | research/09-tile-drawing.md → 18-row coverage table | — | per-pass screenshot vs the C# |
| S1-G-2 | Paint: the C# bakes recoloured render-target copies per (type, style, paint); the port recolours per pixel in `sprite.wgsl::paint_recolor` — prove or replace the equivalence (the current DONE claim is unverifiable) | L | P | research/09 | — | per-paint-colour pixel diff |
| S1-G-3 | Blend states: one blend state cannot express the C#'s additive and reverse-subtract ("multiply") batches (`Main.cs:27462-27490`) | L | P | research/41-render-pipeline.md (D-2) | — | per-batch screenshot |
| S1-G-4 | Colour space: `pow(tint,2.2)` is not the sRGB EOTF — the tint/recolour maths sits in the wrong space | M | P | research/41 (D-5) | — | tint value parity test vs the C# byte multiply |
| S1-G-5 | Scissor: no scissor anywhere — needed for clipped UI rows and map panels | M | P+B | research/41 (D-9); research/18 | — | clipped-row screenshot |
| S1-G-6 | `MoonLordShake` (the screen-shake filter, `Main.cs:64976-64984`) is inert — the C# does not shake the camera | S | P | research/41 (D-7) | — | shake filter test |
| S1-G-7 | `ForcedMinimumZoom` unported | S | P | research/41 (D-8) | — | zoom test |
| S1-G-8 | Wind gust family (`WindGrid`, `TileDrawing.cs:8554-9218`) absent — `Wind::cycle` itself is term-for-term faithful | M | B+P | research/09 | — | gust sequence test |
| S1-G-9 | 9-slice lighting: the sliced path skips the per-slice `GetFinalLight` (paint/actuation) | M | P | research/10 | — | 9-slice pixel diff |
| S1-G-10 | `NON_VISIBLE_PADDING` is clamped to `min(w,h)/3` instead of the fixed 18 | S | P | research/10 | — | padding test |
| S1-G-11 | Atlas composite order vs the XNA `Deferred` sort: merging textures into one atlas changes the composite order of cross-texture draws | L | P | research/41 (D-3/D-11) | G3 | pixel diff on a cross-texture scene |
| S1-G-12 | Correct the "2 draw calls" claim (`ROADMAP.md:64`): up to 6 per frame (3+3 with `TR_POST=1`) | S | X | research/41 | — | draw-call counter test |
| S1-G-13 | Render-target family: no layer-target family, no `skyTarget`, no capture gating | M | P | research/02 | — | target-layout screenshot |
| S1-G-14 | Water mask / waterfall rendering parity (`water_mask.wgsl`) | S | P | research/02; research/11 | — | waterfall screenshot |

## Stage 1H — UI

| id | Task | Size | Sev | Evidence | deps | Verify |
| --- | --- | --- | --- | --- | --- | --- |
| S1-H-1 | Event bubbling child→parent — the roadmap claimed it was ported; it is not | M | B | research/18 (ROADMAP:938-939 wrong) | — | bubbling test |
| S1-H-2 | `UIList` infinite height + the scissor/`OverflowHidden` break-gate | M | P+B | research/18 | G5 | scrolling-list screenshot |
| S1-H-3 | UI scale is pinned to 1.0 — port the vanilla scale factor | S | P | research/18 | — | scale test at 0.8/1.0/1.2 |
| S1-H-4 | The scroll wheel is outside the pointer machine | S | B | research/18 | — | wheel event test |
| S1-H-5 | ItemSlot movement-action table: actions 1/2/3/4/5 (equip/dye/shop/Journey), the `RightClick` equip/vanity/swap ladder, `retainFavoriteOnMoveAt`, the equip achievement hooks | M | B | research/20 | — | per-action test |
| S1-H-6 | `context_accepts` gates only contexts 1/2 — the equipment pages accept any held item | S | B | research/20 | H5 | context test |
| S1-H-7 | Crafting depth: Shift ×10 / craft-all, the `focusRecipe` select step, and the filter/search bars (draw-only today) | M | B | research/20 | — | craft-all test |
| S1-H-8 | Chest UI: the Sort button click is missing (the keyboard rename landed) | S | B | research/20 | — | sort test |
| S1-H-9 | Coin slot: `ForceSlotState` and the coin merge path | S | B | research/20 | — | coin merge test |
| S1-H-10 | World select: the `IsValid` + `TryMovingToRejectionMenuIfNeeded` gates (`UIWorldListItem.cs:362`/`:387-410`) — the roadmap claimed they are faithful; they are missing | S | B | research/19 | — | invalid-world click test |
| S1-H-11 | Character creator: Randomize / Copy / Paste buttons hit nothing (`Click_RandomizePlayer` unported) | S | B | research/19 | — | button test |
| S1-H-12 | World sort: the active-player key is hardcoded `false` | S | B | research/19 | — | sort order test |
| S1-H-13 | Journey mode: the power/research UI over the landed powers (M6-CREATIVE-1/2 rows) — see the atlas | M | B | docs/ATLAS.md → Milestone 6 | — | journey screen checklist |
| S1-H-14 | Settings rows: the invite row is `flag5`-gated (`IngameOptions.cs:178`) — keep the 8-row column 1:1 | S | X | research/38 | — | settings row test |
| S1-H-15 | Frame-cache key: the buff HUD, the chat and the emote wheel must participate in the G6 `WorldFrameKey` once they draw | S | D | docs/AGENTS.md rendering invariants | E6 | frame-key coverage test |

## Stage 1I — Multiplayer & netcode

The transport exists; the **protocol** does not. Module 17 maps 163 C# message
ids against 25 ported ones. These rows finish the protocol before the Stage-2
transport redesign (S2-4) starts.

| id | Task | Size | Sev | Evidence | deps | Verify |
| --- | --- | --- | --- | --- | --- | --- |
| S1-I-1 | Message-id remap: port the missing ids in dependency order (join bundle → entity state → container/wiring/sign → buffs → ping) | L | B | research/17 → message-id map | F1, F2 | per-id round-trip test |
| S1-I-2 | The `5 PlayerInventory` packet is silently dropped | S | B | research/17 | I1 | inventory sync test |
| S1-I-3 | Prediction smoothing (`net_interp_tick`) is port-invented with no C# counterpart — either port the C#'s behaviour or gate it behind the Stage-2 design | M | D+B | research/17 | I1 | two-client determinism test |
| S1-I-4 | The send path: the C# fans out 774 `SendData` sites across 68 files; the port replaces it with ~10 sends in `netglue.rs::net_tick` — map each subsystem's cadence | L | B | research/17 → SendData census | I1 | per-subsystem sync test |
| S1-I-5 | MP kill-sync for the bestiary/achievements | S | B | research/30 | I1 | two-client achievement test |
| S1-I-6 | Steam P2P is not ported (TCP only) — document as a deliberate divergence or port the transport | M | X | research/38 | — | documented in `docs/ANALYSIS.md` |

## Stage 1J — Save / load

| id | Task | Size | Sev | Evidence | deps | Verify |
| --- | --- | --- | --- | --- | --- | --- |
| S1-J-1 | The `.wld` tile writer **drops the four wire bits** — and because the C#'s `isTheSameAs` compares them, the RLE run boundaries diverge too | M | D+B | research/42-save-load-audit.md | — | byte-identical tile section vs a C#-written world |
| S1-J-2 | Header placeholders: GUID, world id, generator version and `LastPlayed` are written as placeholders | S | D | research/42 | — | header byte comparison |
| S1-J-3 | **Golden fixture**: a C#-written `.wld`/`.plr` in `assets/test/` compared byte-for-byte — today every fixture is Rust-internal | M | D | research/42 | J1 | fixture test in CI |
| S1-J-4 | `.bak` rolling backups + post-write validation (the port has neither) | M | B | research/42 | — | backup/restore test |
| S1-J-5 | The `.plr` writer is a subset mirror of the C# writer | M | B | research/42 | J3 | `.plr` byte comparison |
| S1-J-6 | Cloud save — ⛔ Steam-scoped (see Non-goals); keep the exclusion explicit in the docs | S | X | research/38 | — | documented |

## Stage 1K — World generation

| id | Task | Size | Sev | Evidence | deps | Verify |
| --- | --- | --- | --- | --- | --- | --- |
| S1-K-1 | Restore the genRand interleave and the special-seed pass pruning (`DisablePassesForSpecialSeeds`, `WorldGen.cs:22493`) | M | D | research/12-worldgen.md | A1 | per-seed pass-list test |
| S1-K-2 | The biomes/behaviours that the C# instantiates but the port does not (the `Terraria.GameContent.Biomes.*` set) | L | B | research/12 | K1 | per-biome presence test |
| S1-K-3 | `growth.rs` (5 305 lines) — split it; the AGENTS.md "no god-files" rule is violated here | M | X | research/39 | — | module split, behaviour unchanged |
| S1-K-4 | `edit.rs` (6 791 lines) and `world_sim/editing.rs` (7 738 lines) — split them | M | X | research/39 | — | module split, behaviour unchanged |
| S1-K-5 | The worldgen FNV constants were re-pinned ~20× — freeze them behind a golden test | S | D | research/35 | A7 | constant-pin test |

## Stage 1L — Weather, map, wiring, mounts, recipes, localization

| id | Task | Size | Sev | Evidence | deps | Verify |
| --- | --- | --- | --- | --- | --- | --- |
| S1-L-1 | Scene metrics: the counters that gate spawning (`SceneMetrics.Scan`/`ScanTiles`/`AggregateTileCounts`/`CalculateZones`) — the C# uses them for the music/zones and the spawn budget | M | B | research/16-weather.md | — | counter parity test |
| S1-L-2 | Rain/cloud: the `atmo` factor, the snow-tile suppression, the `waterStyle*3 + rand(3)` frame type and the `maxRain * cloudAlpha` slot budget | S | B | research/16 | — | rain count/position test |
| S1-L-3 | Map: the reveal/queue/edge logic (`WorldMap.UpdateLighting`/`UpdateType`/`QueueUpdate`/`UnlockMapSection`/`UnlockMapTilePretty`/`ClearEdges`) | M | B+P | research/14-map-minimap.md | — | reveal test + minimap screenshot |
| S1-L-4 | Map overlay layers: ping, spawn, team-spawn, teleport-pylon, boss-bag, digtoise | M | P | research/14 | — | per-layer screenshot |
| S1-L-5 | Map paint multiply (`MapColor`, `MapHelper.cs:1823-1874`) and the `.map` chunk compression parity | S | P | research/14 | — | `.map` byte comparison |
| S1-L-6 | Wiring: the signal-propagation order, the switch-type table, the statue RNG order and the `TruncatesWalls`/`invisibleWall` gates | M | D+B | research/23-wiring-tile-entities.md | — | propagation-order test |
| S1-L-7 | Tile entities: the missing subclasses (display doll, item frame, hat rack, logic sensor, weapon rack, food platter, training dummy) and their `.wld` bytes | M | B | research/23 | L6 | per-TE save/load test |
| S1-L-8 | Mount activation rules: the `MountID.Sets.Cart`/`CanUseHooks` grapple exemption (`Mount.cs:6599`), the bat-wet rule for id 56 (`:6595`), the Fishron halving (id 12, `Player.cs:26655`) | S | B | research/27-mounts-pets-vehicles.md | — | per-id activation test |
| S1-L-9 | Mount stat table: 50 of 66 ids ported; `SetMount`/`Dismount` per-type extras (type-44 velocity damp, `fullRotation` reset, the CC dismount gate, `noItems` clear); the SuperCart statics are `0` instead of 20/20/0.1/15/5.15 | M | B | research/27 | — | per-id stat table test |
| S1-L-10 | Grapple: the item-driven hook fire, the latch blacklist and the chain rendering (`Minecart.FrameTrack` too) | M | B | research/27 | — | hook lifecycle test |
| S1-L-11 | Recipes: the table-count parity with the C# recipe block, the `RecipeGroup` expansion rule, the crafting-station tile+range checks and the available-recipe filter | M | B | research/21-recipe-crafting.md | — | per-recipe craft test |
| S1-L-12 | Localization: the key coverage against the C# en-US set, the shipped languages and the fallback chain; the name tables must be data-driven | M | B+P | research/33-localization.md | — | key coverage test (no false "unknown key") |

---

# Stage 2 — engine & performance

Stage 1 closes the behavioural gap. Stage 2 changes the engine *underneath*
while keeping the player's experience 1:1: same visuals, same mechanics, same
pixels. The decisions below were taken with the owner on 2026-11-12 and are
frozen unless a new measurement contradicts them. Source: the Stage-2
agreements (`docs/superpowers/specs/` for the design docs; the numbers quoted
from the 2026-11-12 baseline are stale by the owner's instruction — see
**S2-0**).

The audit changed one thing in the plan: **Stage 2 cannot start until the
behavioural baseline is trustworthy.** The audit found order inversions in the
world branch (S1-B-1), a camera that lerps in the wrong place (S1-B-4), and
projectiles that live 2× too long (S1-A-9). Optimising a mis-ordered tick
optimises the wrong thing. So Stage 1A/1B are a **gate**, not a parallel track.

## S2-0 — Fresh measurements (first task of Stage 2)

The old numbers (present floor ~1.1 ms, light 0.01–0.03 ms) are stale. Before
any optimisation:

| id | Task | Size | Sev | Evidence | deps | Verify |
| --- | --- | --- | --- | --- | --- | --- |
| S2-0 | Re-measure the heavy scenes with `TR_BENCH=600` + `TR_STATS=1` (100 NPCs, 500 projectiles, 10 k dust, a large world, the boss fights) and publish the per-phase breakdown | M | D | Stage-2 agreements §"Контекст"; research/39 → concurrency readiness | Stage 1 | a `docs/PERF.md` baseline table with the commit hash |

## S2-1 — The entity engine (SoA + two-phase tick)

Decided: **manual data-oriented structures (SoA by archetype), no ECS
framework** (hecs/Bevy/Flecs/SkyECS/FreeCS/Shipyard were evaluated and
rejected: ~5 fixed archetypes, counts in the hundreds-to-thousands, and a
framework costs id-indirection on every access while disturbing the tick order
and RNG that 1:1 depends on).

| id | Task | Size | Sev | Evidence | deps | Verify |
| --- | --- | --- | --- | --- | --- | --- |
| S2-1a | Convert the entity storage to columnar layout (pos, vel, hp, `ai[4]`, flags) with a free-list of stable ids, replacing `Vec<Npc>` / `Vec<Projectile>` AoS. The tile world model stays as is (12-byte tile, 60 MB world) | L | X | agreements §"Движок сущностей"; research/39 (no SoA today) | S2-0 | identical simulation results; the free-list keeps ids stable across compaction |
| S2-1b | Split the tick into **Phase A (parallel)**: each entity reads the world (tiles, player, spatial hash) and the state at the *start* of the tick (double-buffered columns) and writes only its own slot — no shared mutable state, so the result is identical for any schedule | L | X | agreements §"Модель тика" | S2-1a | bit-exact comparison against the sequential tick over N seeds |
| S2-1c | **Phase B (sequential, in the C# order)**: inter-entity damage, deaths, slot compaction, spawn requests, worm chains. Deaths are *marked* immediately (visible to others in the same tick, as in vanilla) and reclaimed in a batch at the end of the tick in slot order | L | X+D | agreements §"Смерти" | S2-1b | bit-exact comparison; death visibility test |
| S2-1d | Event batches: per-thread append-only buffers (16–32 B per event) → merge → radix-sort by target → dense application in Phase B | M | X | agreements §"Батчи событий" | S2-1b | same damage outcomes as the sequential path |
| S2-1e | Spatial hash: a uniform grid rebuilt each tick in parallel (counting sort), read-only in Phase A; replaces the 15 linear `npcs.iter()` scans | M | X | agreements §"Spatial hash"; research/39 | S2-1b | scan count constant; nearest-neighbour parity |
| S2-1f | Projectiles are the first candidate for parallelisation (like NPCs, fewer cross-interactions) | S | X | agreements §"Модель тика" | S2-1b | as above, projectile-only |

**SAFE to parallelise (Phase A):** movement, gravity, tile collision (read-only),
AI machines, target search, NPC separation, animation, dust/gore.
**SERIAL (Phase B):** spawn RNG rolls (cheap), damage, deaths, compaction, spawns,
worm chains.

## S2-2 — CPU parallelism beyond entities

| id | Task | Size | Sev | Evidence | deps | Verify |
| --- | --- | --- | --- | --- | --- | --- |
| S2-2a | Tile draw-data / culling: parallel count over chunks → prefix-sum in raster order → parallel fill (each chunk knows its offset) → the buffer is byte-identical to the sequential output. This is the honest parallelisation of the draw stage (it removes the pixel-identity risk of the greedy-meshing idea) | M | X+P | agreements §"CPU-многопоточность Beyond сущностей" (1) | S2-0 | byte-identical draw buffer |
| S2-2b | Tile-framing cascades in waves (the BFS-frontier levels are independent) | M | X | agreements (2) | S1-A-3 | identical framing results |
| S2-2c | Liquid flow — a candidate **with a caveat**: the halo split is possible, but the C# order must be understood 1:1 first (it is not fully ported today) | L | X | agreements (3); research/11 | S1-A-8 | identical liquid state per tick |
| S2-2d | Lighting is already partly parallel (seeding + horizontal sweeps; the vertical pass is serial by design). Next: a diagonal wavefront, or re-open GPU lighting — the latter stays deferred (present-bound) until fresh measurements say otherwise | M | X | agreements (4) | S2-0 | identical light buffer |
| S2-2e | Save/load the world on a background thread with a double-buffered snapshot (the in-process world-install mechanism applies to saves) | M | X | agreements (5) | S1-J-1 | no frame hitch during a save |
| S2-2f | Cross-cutting "FPS floor" work: frame pipelining (simulate N+1 while the GPU draws N), thread affinity (simulation/network on their own cores), zero allocation on the hot path (per-phase arena/freelist, mimalloc as a candidate), SIMD over column chunks of 8, lock-free SPSC/MPSC queues, 64-byte-aligned per-thread buffers (false sharing), a warm rayon pool with priorities (render-prep above background) | L | X | agreements (6) | S2-1 | frame-time profile; no regressions in the pixel output |

**Inherently serial:** the fixed step, the spawn rolls, the UI input.

## S2-3 — GPU particles (dust)

The only layer with thousands of identical particles and a trivial per-particle
update → one compute dispatch instead of thousands of CPU updates. **Blocked on
Stage 1**: the dust layer is not ported at all (S1-E-1), and vanilla draws
particles from the shared RNG stream in slot order — a 1:1 check of the
per-particle RNG stream is mandatory before any GPU dispatch.

## S2-4 — Multiplayer architecture

The decided model (vanilla-like behaviour, our engine underneath):

1. The client predicts **its own player completely locally** (movement, swings,
   tile breaking) and sends the actions to the host immediately; the host owns
   every entity, simulates them and broadcasts to the connected clients.
2. **Interest management:** a spatial AOI grid on the host; a client's view rect
   = the screen + 2 screens of margin on each side. Only the tiles inside the
   rect that are *dirty* (per-column dirty flags) enter the client's delta.
   Vanilla sends every active NPC/projectile to every player per entity — we
   send a dense byte slice of the columns.
3. **Interpolation of remote entities:** the client keeps a 2–3 tick ring and
   draws the world "1.5 ticks in the past". Extrapolation is only a stop-gap
   (no updates for >3 ticks → ~50 ms of drift and a freeze); clean
   extrapolation does not exist in an action game. The added latency is ~25 ms,
   never more than vanilla's.
4. **Prediction correction:** a ring of predicted states over the RTT window;
   the error = confirmed − predicted; sub-pixel errors are ignored, larger ones
   converge over 1–2 ticks (Source-style). A hard revert only happens when the
   host refuses the action (occupied tile, dead target).
5. **Topology:** player-host by default; a dedicated server uses the same code
   path.

Packets: one packet per client per tick (all deltas + events batched), 16-bit
slot ids, position delta-coding. The goal is **≤ vanilla's traffic**.
Tile edits: applied locally at once → the host validates against its own state →
the delta goes to the nearby clients; conflicts resolve last-writer-wins by the
host's tick order. Damage travels as an **event** (target, amount, type), not
as "subtract hp" — only the host computes hp, in its deterministic batch, and
broadcasts the result. Input latency D = 1–3 ticks for remote actions;
local feedback is immediate. SP == MP behaviourally (not byte-identically,
because of latency — as in vanilla). Networking runs on its own thread with
per-client ring buffers of ticks; the simulation never blocks on ping; a
factorio-style lag indicator shows when a packet is late.

## S2-5 — Lossless world compression (save format)

JPEG/WebP logic (DCT/quantisation) is **lossy** and therefore inapplicable to a
1:1 port. Only lossless: the decompressed bytes must be identical, with zero
behavioural risk.

| id | Task | Size | Sev | Evidence | deps | Verify |
| --- | --- | --- | --- | --- | --- | --- |
| S2-5a | Insert the compression at `wld::to_bytes` (isolated per-section bodies + pointer backfill) — the safe insertion point found by the audit | M | X | agreements §"Сжатие"; research/42 → compression hooks | S1-J-1 | decompressed bytes identical to the uncompressed writer |
| S2-5b | Level 1: RLE/VLQ predictor over rows/columns (long dungeon runs of the same tile) | M | X | agreements | S2-5a | size win on a large world; byte-exact round trip |
| S2-5c | Level 2: deflate/zlib per component (tiles/walls/liquids/TE) via `flate2` (already a dependency) — expect 2–3× | M | X | agreements | S2-5a | as above |
| S2-5d | Level 3: the vanilla frame-important bitset — do not read/recompute the frames of an untouched section | M | X | agreements | S2-5a | frame-cache reuse preserved |
| S2-5e | A compressed file must bump the version past 326 (the vanilla rejects it as `LaterVersion`); the `.plr` must be compressed *inside* the AES layer | S | D | research/42 | S2-5a | the vanilla's reader rejects/accepts as designed |
| S2-5f | Document the optional in-memory compressed sections + a hot-section cache (242 MB for a large world) as a memory option, not an FPS one | S | X | agreements | S2-5b | memory profile |

The compression level is a size-vs-load-time trade-off; the file format must
stay readable by the vanilla when the compression is off.

## S2-6 — Process gates

1. S2-0 fresh measurements → 2. the **engine** design doc (SoA + phases +
   batches + GPU dust) — first, because it defines the interfaces → 3. the
   **network** design doc (the five mechanisms above) → 4. a builder-check
   review reads both before approval → 5. approval → `writing-plans` → the
   tasks land in this file.

Design docs live in `docs/superpowers/specs/YYYY-MM-DD-<topic>-design.md`.

**Deferred / closed rasterisation rows** (owner decision, 2026-11-12): B6 (merge
identical textured runs), B7 (texture array), B10 (atlas repack), D5/D6 (GPU
light, deferred lighting) and F5 (GPU culling) are **closed permanently** — the
greedy-meshing idea was rejected for five reasons (per-tile frames from the 8
neighbour merge sets, `RandomStyleRange`, per-tile tint, slopes/half-tiles,
actuated, wires, plants and wind sway break pixel identity; Terraria already
does a greedy merge in `GetTileDrawData`/`TileDrawing.cs`, which the port
reproduces; polygons are not the bottleneck — the pipeline is instanced with
2–80 k instances; atomicity of untouched blocks already exists via the G6
`WorldFrameKey` + the per-chunk generation cache at 72–98 % reuse; B6/B7/B10
were deferred for exactly the pixel-identity reason). A5 (zoom LOD), D4 (light
LOD), C6 (light one frame ahead) and F2 (temporal cycle) stay rejected for
their original reasons.

---

# Verification protocol

A task is done when its row's **verify** column is satisfied. The house rules
(AGENTS.md) apply in addition:

1. **C# first.** Read the C# implementation and its callers before writing any
   Rust; keep the C# line citations in the code you port.
2. **Tests:** `cargo test --workspace -- --skip migrates_version_2_kinds_to_item_ids`
   must be green **on the branch after the sync**. Note: cargo may exit 1 on
   success in this project — judge by the `test result: ok` lines only.
3. **Determinism:** any change to an RNG stream, the fixed-step order or the
   G6 `WorldFrameKey` needs byte-equality proof (a golden fixture, a two-seed
   comparison, or a golden stream vector).
4. **Visual work:** deterministic screenshots (`TR_SHOT`, `TR_CAM`, `TR_DAY`,
   `TR_NOPLAYER`, `TR_NO_BG`, `TR_UI=<screen>`, `TR_DEBUG=0`) compared with
   `tools/pixel_diff.py`; two identical runs must be SHA256-equal; check zoom
   1.0, 1.5 and 2.0 when background/parallax changes.
5. **Performance work:** A/B against the previous binary with `TR_BENCH=600`
   and `TR_STATS=1`; include before/after numbers.
6. **Docs:** update `docs/ARCHITECTURE.md` (if modules moved), `docs/UI.md`,
   `docs/ANALYSIS.md` (every deliberate deviation), `docs/PORT_MAP.md` and this
   file. `README.md` is player-facing — only for visible features.
7. **One task per branch**, imperative commit messages, and the graph refreshed
   (`graphify update .`) when code shape changed.

---

# Recommended order of attack

1. **Stage 1A (determinism) + 1B (tick order) — the gate.** Until the tick
   order matches the C# and the RNG streams are the C#'s, every measurement and
   every optimisation is measuring the wrong thing. Start with S1-B-1…B-3,
   S1-A-1…A-4, S1-C-3, S1-F-1/F-2 (the id collisions lose save data today).
2. **The one FAIL and the two STALE rows** — S1-D-1 (hover presets), the
   stale snapshot rows and the M1.5 measurement tables.
3. **The four cheap correctness bugs with player-visible impact** — S1-C-9
   (2× item pickup), S1-C-2 (projectile damage), S1-C-8 (hitbox), S1-C-10
   (right-click transfer).
4. **Missing subsystems with the largest call-site censuses** — S1-E-1…E-4
   (dust/gore: 4 922 C# call sites), S1-E-5 (entity lights: 300+), S1-E-9
   (golf), S1-E-6/E-7 (the buff HUD and the effect table).
5. **The AI dispatch refactor** — S1-D-2/D-4 first (the axis), then the
   families. This is the largest single behaviour block and it gates the boss
   content.
6. **Rendering** — S1-G-1 (the 10 missing draw passes), S1-G-2 (the paint
   proof), S1-G-3/G-4 (blend + colour space). Pixel work needs the
   screenshot rig, so it goes after the simulation is trustworthy.
7. **UI** — S1-H-1…H-9 (the framework and the slot rules), then the screen
   families.
8. **Netcode** — S1-I-1…I-4, in the order of the message-id map.
9. **Save/load** — S1-J-1…J-3 (byte parity + the golden fixture), then the
   Stage-2 compression.
10. **Stage 2** — S2-0 measurements, then the engine design doc, then the
    network design doc, then implementation.

Two-agent rule (AGENTS.md): `git fetch --prune` and check `git branch -a`
before claiming a row; push the task branch as the claim.

---

# Engineering debt

| Task | Branch | Scope |
| --- | --- | --- |
| ✅ REFACTOR-env | `refactor-app-env-flags` | centralize ~65 TR_* flags into app/env.rs — done 2026-11-09 |
| ✅ REFACTOR-rng | `refactor-xorshift-unify` | unify the worldgen/wind xorshift cores byte-exactly — done 2026-11-09 |
| ✅ REFACTOR-saveheader | `refactor-save-list-reader` | absorb HeaderReader into binio — done 2026-11-09 |
| ✅ REFACTOR-cleanup | `refactor-allow-cleanup` | remove inert `#[allow(dead_code)]` — done 2026-11-09 |
| S1-D-2 | — | the `aiStyle` dispatch axis (see Stage 1D) |
| S1-K-3 | — | split `growth.rs` (5 305 lines) |
| S1-K-4 | — | split `edit.rs` (6 791) and `world_sim/editing.rs` (7 738) |
| S1-B-8 | — | split `app/frame.rs` (1 824-line single function) |
| S1-G-1 | — | split `projectile.rs` (17 411 lines) and `npc.rs` (17 134) — the entity half's god-files, which the 2026-11-01 refactor did not remove |
| — | — | `App` has 257 fields (the docs say 118) and `World` mixes ~50 runtime event flags into the tile container — S2-1 needs both cleaned up first |
| E1 (paint byte) | — | paint byte landed with TILE-DRAW-4; further tile packing (12→8-10 B) stays deferred |

---

# Documentation corrections (from the audit)

These are the places where the current docs state something the audit disproved.
Each needs a one-line correction plus the task that fixes the underlying code.

| Doc | Claim | Reality | Fix |
| --- | --- | --- | --- |
| `docs/ROADMAP.md:64` (atlas) | "2 draw calls" | up to 6 per frame (3+3 with `TR_POST=1`) | S1-G-12 |
| `docs/ROADMAP.md:73` (atlas snapshot) | 🟡 on the M4-NPC-3c/3h..3r rows | the catalog marks them ✅ | S1-D-2 |
| `docs/ROADMAP.md:78` (atlas) | the audio bank is "8 cues" | 26 cues in the bank, 13 played | S1-E-21 |
| `docs/ROADMAP.md:938-939` (atlas) | UI event bubbling is ported | no bubbling exists | S1-H-1 |
| `docs/ROADMAP.md:1404` (atlas) | the hover preset citations | they point at the wrong C# branches (`:50322`=type 619, `:50328`=type 252) | S1-D-1 |
| `docs/ROADMAP.md:1658` (atlas) | M7-GOLF-1 | parked, and the golf subsystem is 100 % absent | S1-E-9 |
| `docs/ANALYSIS.md:2120-2124` | "no `.plr` reader/writer" | the code reads and writes real `.plr`s | docs fix |
| `docs/ANALYSIS.md:3024-3027` | "the duplicated dead arms removed" | 29+21 dead arms remain and the renumber created the id collision | S1-F-2 |
| `docs/PORT_MAP.md:167` | collision "partial" | confirmed PARTIAL (see the audit table) | — |
| `docs/PORT_MAP.md:180` | "no `.plr` reader/writer" | stale | docs fix |
| `docs/PORT_MAP.md:240` | "8-cue bank", "front-end music and UI sounds missing" | stale — both landed | docs fix |
| `docs/PORT_MAP.md` (ReLogic row) | the `ReLogic/Content/Lzx*` citation | the wrong file — ReLogic never parses XNB | docs fix |
| `docs/ARCHITECTURE.md` | the crate/module line counts and the graph node count | stale | docs fix |
| `docs/ARCHITECTURE.md` | "the god-files are gone" | false for the entity half (8 files > 5 000 lines) | S1-D-2, S1-K-3/4, S1-B-8 |
| `README.md` | multiplayer and the in-game options window are missing | both landed | docs fix |
| `docs/ROADMAP.md` non-goals | "exact network compatibility until M6-NET is complete — still single-player-first" | obsolete: M6-NET-1..5 landed 2026-11-09 | non-goals fix |
| `docs/ROADMAP.md` non-goals | the RGB non-goal names `GameContent/RGB/*` | it contains the virtual keyboard the port **did** port | non-goals fix |
| `docs/ANALYSIS.md` | the "XnbExtract PNG pipeline" text | stale | docs fix |
| `docs/ROADMAP.md` (atlas) | the mount table has "12 rows" | 50 of 66 ids are ported | docs fix |
| `docs/ROADMAP.md` (atlas) | the "Saves" status line | `.trwld v10` → v13; BUG-03 closed | docs fix |

---

# Non-goals (updated 2026-11-12)

Confirmed by the audit (research/38):

- **Modding API / tModLoader compatibility** — unchanged.
- **Mobile/console parity** — unchanged.
- **`Terraria/Social/**` (72 files, ~6 200 lines: Steam + WeGame modules, the
  Workshop, P2P, cloud, overlay, rich presence, join requests)** — excluded,
  and now enumerated so the exclusion is explicit. The Steam Workshop hub is
  the browser.
- **RGB peripheral lighting** (`GameContent/RGB/*`) — excluded, but the scope is
  narrowed: the virtual keyboard inside that folder **was** ported, so the
  non-goal must name the peripheral lighting only.
- **Steam cloud saves** — excluded with `Social/*`.
- **Network compatibility with the vanilla game** — the handshake is
  port-internal (two of our clients); the protocol is not the vanilla's. The
  previous wording ("until M6-NET is complete — still single-player-first") is
  obsolete and is replaced by the explicit statement above.

**Undocumented absences that must now be declared** (they were not in any
non-goal): `Terraria/Testing/**` (34 files, ~6 000 lines, including the
vanilla's deterministic `RecordReplay`/`StateSnapshot`/`LockstepDebug` harness),
`Terraria/TimeLogger.cs`, the crash dump/watcher handlers, the in-game
`UIBestiaryTest`/`UITextWrappingTest` screens, and dedicated-server mode
(169 `Main.dedServ` sites). `GameBanana`, `Terraria.Configuration` and
`Terraria.Mono` do not exist in 1.4.5.8 — nothing to exclude there.

---

# Appendix — research index

| Wave | Modules | Report |
| --- | --- | --- |
| 1 — god files & world core | 01 Main loop · 02 draw pipeline · 03 Player core · 04 Player combat · 05 NPC core · 06 Projectile core · 07 Item core · 08 Tile/framing · 09 TileDrawing · 10 Lighting · 11 Liquid · 12 WorldGen | `docs/research/01..12` |
| 2 — world & render systems | 13 Collision · 14 Map · 15 Dust/Gore · 16 Weather · 17 Netcode · 18 UI framework · 19 Front-end · 20 In-game UI · 21 Recipes · 22 Chests/loot/shops · 23 Wiring/TE · 24 Spawner/invasions | `docs/research/13..24` |
| 3 — entity deep dives | 25 NPC AI · 26 Projectile AI · 27 Mounts/pets/vehicles · 28 Buffs · 29 Town NPCs · 30 Achievements/bestiary · 31 Golf · 32 Sound · 33 Localization · 34 ID sets · 35 RNG/utils · 36 Cinematics/OOA | `docs/research/25..36` |
| 4 — infrastructure & port-side | 37 ReLogic · 38 Out-of-scope · 39 Port architecture · 40 DONE re-verification · 41 Render pipeline · 42 Save/load | `docs/research/37..42` |
| consolidation | the task-seed register, the risk register and the unadmitted-claims list | `docs/research/90-consolidated-findings.md` |

Each report carries the module's scope, key entities with line references,
dependencies (calls into / called by / shared data), the port-status table, the
covering tests, severity-graded deviations and sized task seeds. The atlas
(`docs/ATLAS.md`) keeps the per-subsystem description and the legacy task rows.



