# 339 — W4-g17 research FINAL + the GORE-WORLD-family bible (registry 339, f5f seal)

> **Chain-banner (registry §9 UNIFIED FINAL, row 339): 304 → 339.**
> The lane's original 304 self-pick (`3dec710dc`, 16:32:44) landed AFTER the
> §7.4 publication that ASSIGNED 304 to d50 (`243-d50-dust-type-table-final.md`
> → `304`, published 16:07 `0426245f2`). Per §8.6: post-§7.4-contested;
> **d50 KEEPS 304**, g17 → **339**. Chain: 304 → 339. Renamed per §9.2
> (registry `docs/research/150` at d54 tip `b243c7086`); §9 is the single
> authority, NO SELF-PICKS.

Date: 2026-11-21. **Code-only mode** — no cargo/tests/rustfmt/graphify
invoked; every claim below was verified by re-reading the cited C# and the
landed Rust at this branch's tip `f0fe1bbdd`. Witnesses are WRITTEN, NOT
RUN (owed the merge-verify window). Branch: `task/W4-g17-gore-world-sites`.

## 0. The registry number — 339 (renamed from 304 per §8.6/§9.2; 304 = d50)

**SUPERSEDED claim (chain-history).** At authoring time the lane read the
research-number collision registry (`docs/research/150` at
`origin/task/W4-d54-dust-sites-world` tip `38bfd1d22`, §7 round-3 authority),
which assigned losers to **151–236 + 246–280 + 281–303** and declared
"Numbers 304+ are FREE" — and the fresh all-tip sweep found max 280, nothing
at 304+. The lane took 304 on that basis (`3dec710dc`, 16:32:44). That
sweep predated the landed §7.4 assignment: **304 was ASSIGNED to d50**
(published 16:07 `0426245f2`), so g17's push is post-§7.4-contested.

**Current verdict (§8.6 amendment + §9 UNIFIED FINAL row 339, registry at
d54 tip `b243c7086`): d50 KEEPS 304; this doc is 339.** Chain: 304 → 339.
No self-picks; §9 is the single authority. Sibling FINAL docs
read at authoring time: g58's 240–243 (happiness), g68's 247 (wind), d120's 143
(chatter retire) — all read for cross-lane contracts below.

## 1. SEAL item 1 — g17c landed (verified at tip)

The three g17c dispatches are LIVE at `f0fe1bbdd`, verified by direct read:

| Piece | Landing commit | Evidence at tip |
| --- | --- | --- |
| ShakeTree loot ladder | `51de86fda` (+ hardening `7197c4d28`, bee-guard `c0e6f07fe`) | `world_sim/editing/shake_loot.rs` — **613 lines**, the full genRand else-if chain (`WorldGen.cs:63163-63403`), `shake_tree_resolve`/`shake_tree_fx_at` split in `tr-core/src/tree.rs` |
| Hair-dye state + arm | `bd170d688` | `player.rs:1215` `pub hair_dye: u8`; the use arm `world_sim/editing.rs:2654-2677` (`Player.cs:42856-42866`); the tick-tail drain `world_sim/player.rs:1479-1500` behind the team-dye gate (item 1984 `hair_dye` 8) |
| 9-dust-only ItemCheck arms | `106854bdb` (+ order fix `f0fe1bbdd`) | `player_dust_use.rs` — 480 lines, `item_use_dust_pre`/`_post`/`_tail` (`Player.cs:44756-45023`), three interleaved halves around the gore lane's 65/3065/enchant-7/1304 arms; the enchant-8/magmaStone tail runs BEFORE the 1304 arm (the C# order, `:45009-45023` then `:45024`) |

Witness count at tip: **59** `fn w4_g17*` tests across 12 files
(player_gore_a..e 16, world_gore_a/b 14, editing.rs 12, tree.rs 6,
world_sim/player.rs 6, npcs.rs 3, growth.rs 2) — all WRITTEN NOT RUN.
The anchors.txt carries 15 g17 rows (7 module + 5 wiring + 3 wave-2).

## 2. SEAL item 2 — g17d verified (dispatched-earlier, executed here)

### 2a. The World.cs gore-family FINAL census — there IS no World.cs

The brief's "World.cs-gore-family-FINAL-census" resolves against the
decompilation's actual shape: **this decompilation has no `World.cs`
file.** The `Terraria.World` class's statics live in `Main.cs`
(`maxTilesX` `Main.cs:1011` etc.); a full-tree scan
(`class World\b` over every `.cs` under `decompiled/Terraria`, junction
recursion excluded) finds only `WorldGen`/`WorldItem`/`WorldSections`/
`WorldSize`. The world-side gore census therefore reads **WorldGen.cs +
Main.cs**, and the FINAL count is:

| C# file | bare `NewGore` | `NewGorePerfect` | `NewGoreDirect` | total | owner lane |
| --- | --- | --- | --- | --- | --- |
| `NPC.cs` | 1338 | 4 | 3 | **1345** | d55/e5/f10 (`npc_hit_effect_a/b/c`, the HitEffect table COMPLETE per f10 `19e1139d6`) + g59/g60 death effects |
| `Projectile.cs` | 129 | 6 | 30 | **165** | g16 (`projectile_gore_a..o`, wiring batches 1-5) |
| `WorldGen.cs` | 35 | 0 | 1 | **36** | **g17** (`world_gore_a` 5 + `world_gore_b` 31) |
| `Player.cs` | 32 | 0 | 0 | **32** | **g17** (`player_gore_a..e`) |
| `TileDrawing.cs` | 16 | 3 | 1 | **20** | d60/g68 (leaf gores via GorePool, g68 seal `0f24c565e`) + tile-draw lanes |
| `ParticleOrchestrator.cs` | 2 | 1 | 9 | **12** | e4 (research/66 orch-family bible, keep-one resolved `eacaeb456`) |
| `Gore.cs` (self) | 2 | 0 | 2 | **4** | the pool itself (`Update` leaf-split `:440`, drip `:647`, the Perfect/Direct wrappers `:1275`/`:1283`) — g68/d50 pool lanes |
| `DelegateMethods.cs` | 3 | 0 | 0 | **3** | f6 (`SpawnFartCloud` `:258-306` STAGED, `573b7e7f2`, the c14-delegate activation seam) |
| `Mount.cs` | 3 | 0 | 0 | **3** | the mount lane (cart `k%10` `:6466`, drill/basilisk 40/41/42 `:6551`, type-23 `:6560`) — UNCLAIMED, task-seed below |
| `PlayerDrawSet.cs` | 3 | 0 | 0 | **3** | the player-draw lane (palladium 331 `:1318`, loveStruck 331 `:1326`, shimmer 1024-1026 `:1369`) — UNCLAIMED, task-seed below |
| `AmbientWindSystem.cs` | 0 | 2 | 0 | **2** | the wind/cloud lane (`SpawnAirborneCloud` `:111` 1091/1093, `SpawnFloorCloud` `:128` 1087-1090) — g68's seal covers the LEAF gores, NOT these clouds; candidate g69-clouds-parallax |
| `Utils.cs` | 1 | 0 | 0 | **1** | `PoofOfSmoke` `:1083` — bug_net documents the skip (`bug_net.rs:56-60`), the statue-poof lane's |
| `WorldItem.cs` | 1 | 0 | 0 | **1** | the fallen-star daytime burn `:653` — g15's `fallen_star_burn` ports the 10 dusts and documents the 3 gore siblings as "the gore lane's skip" (`item_dust.rs:387-405`); the gore half is UNCLAIMED (d55/f10 are NPC lanes), task-seed below |
| `Main.cs` | 0 | 0 | 0 | **0** | verified ZERO (only `Gore.goreTime` `:17075` + TimeLogger rows) |
| **TOTAL** | **1565** | **16** | **46** | **1627** | |

The g17 lane's own census is **68 = WorldGen 36 + Player 32** (Main 0),
unchanged from wave E's declaration (`ec6fe4e68`) — re-counted this round
by direct `Select-String` per file, every line number re-read. The
research/15 baseline (1643 sites) DOUBLE-COUNTED `NewGorePerfect`: its
"`NewGore` 1627" was the whole-family prefix match (`Gore.NewGore`
matches Perfect and Direct too — 1565 + 46 + 16 = 1627), then added the
16 Perfect sites again (1627 + 16 = 1643). The FINAL per-variant count
above (1565 bare / 16 Perfect / 46 Direct = **1627 distinct call sites**)
is the authority.

### 2b. shake-loot i30-compose — ONE FastRandom throw (witness added)

The compose seam is verified at `world_sim/editing.rs:1870-1901`
(`axe_tile_hit`'s fail tick): `shake_tree_resolve` records the shake
ONCE, then **ONE** `UpdatePlayers` throw seeds the genRand `FastRandom`
stand-in (`:1876-1880`), the ladder draws (genRand gates on the stand-in,
`Main.rand` item/NPC rolls on `UpdatePlayers`), then the FX tail
(`shake_tree_fx_at` → `tree_grow_fx(hitTree:true)`) on the same stream —
the C# order (`WorldGen.cs:63131-63418`). The i30 chop lane owns the
break path (per-cell BREAK drops via `break_tile`); a fail tick never
breaks, so this compose is the tick's whole draw.

**New witness** (this round): `w4_g17_the_axe_fail_tick_compose_draws_one_seed_throw`
(`world_sim/editing.rs`) — the live `axe_tile_hit` path vs a manual
resolve → ONE-throw → ladder → FX twin, seeded identically; asserts
stream equality + item/NPC/gore pool equality + the dedupe book. A second
seed throw or a reordering diverges the streams. WRITTEN NOT RUN.
Witness total: **60**.

### 2c. The 6 unported chat buttons — DEAD DrawNPCChatButtons confirmed

`Main.DrawNPCChatButtons` (`Main.cs:40474-40673`) has **ZERO callers in
the whole decompilation** (a full-tree `DrawNPCChatButtons` scan finds
only the definition). It is DEAD CODE — the `NPCChatPanel.DrawButtons`
path (`NPCChatPanel.cs:88-162`) replaced it; g58's research/240 row 8
already pinned this ("NOT ported, correctly"). The `npcChatFocus1..4`
statics (`Main.cs:1757-1763`) are written ONLY inside that dead function
and the hair-window (`:43868`/`:43925`, unported M4-TOWN-17) — no live
chat-panel reader exists.

The **6 unported chat buttons** (the `NPCInteractions.All` roster,
`NPCInteractions.cs:486-531`, minus the landed ones) are:

| # | Action | C# | Status |
| --- | --- | --- | --- |
| 1 | `StardewValleyBit` (Dryad cola) | `:79-103` | UNPORTED (the Dryad animation system, g58 research/240 cross-note) |
| 2 | `DryadPurification` | `:105-131` | UNPORTED (`GetDryadWorldStatusDialog` + achievement 27) |
| 3 | `GuideTip` | `:201-219` | UNPORTED (`Main.HelpText`) |
| 4 | `RequestHome` (housing) | `:314-368` | UNPORTED (the `TownNPCMood_*.NoHome` + `HousingText.HousingRequirements` text; f38 owns the chatter, NOT this button) |
| 5 | `PartyGirlMusicSwap` | `:371-387` | UNPORTED (`NPCChatText_PartyGirlSwapMusic`) |
| 6 | `TavernkeepAdvice` | `:465-481` | UNPORTED (`NPCChatText_TavernkeepAdvice`) |

The 6 are exactly g58's research/240 UNPORTED cross-note table (the
Dryad pair, GuideTip, RequestHome, the Tavernkeep advice, the music
swap). Landed for contrast: `CloseChat` (S1-E-10b), `OpenShop` (the
25-row registry, d120), `ReportHappiness` (g58), `OpenSign` (f22),
`AnglerQuest` (angler.rs), `NurseHeal` (app/mod.rs), `TinkererReforge`
(the reforge menu). The remaining roster entries
(`TaxCollectorCollectTaxes`/`PetAnimal`/`OldManCurse`/
`GuideReverseCrafting`/`StylistHairWindow`/`DyeTraderRarePlant`) ride
their system lanes — their button entry points are the panel's
`interaction_rects` walk (f22's), not this lane's to claim. The 6 above
are the residue the gore-world lane does NOT own — they are recorded
here so the merger sees the full roster boundary. **No code change**:
the dead function stays unported (correctly), the 6 buttons stay with
their lanes.

## 3. SEAL item 3 — f5f-gore-seal (the 68-site census FINAL)

The f5f seal is the g17 lane's own census closure: **Main 0 / WorldGen
36 / Player 32 = 68** `Gore.NewGore*` sites, ported across the 9 module
commits (wave A `9f574c4d0`, B `a342a6c8b`/`91c23d090`/`e3c71fb94`,
player A-E `556754eeb`/`14dedf496`/`3ff14a0ab`/`f3e554d5a`/`ec6fe4e68`)
and wired across the 10 wiring commits (wave 1 `355380f9c`..`7277d8550`,
wave 2 `51de86fda`/`bd170d688`/`106854bdb` + fixes). Every site is
re-counted and re-read this round (§2a); the census is FINAL. The
keep-one contracts (sandstorm → f42 `4bf7b2e88`, msg-112 net arm → g50)
are published in ANALYSIS "The gore wiring wave 2" and hold at the
co-lane tips probed this round.

## 4. The GORE-WORLD-family bible — SEALED

**The gore-world family is CLOSED for porting on this lane.** The one
canonical shape: the world/player gore sites (WorldGen 36 + Player 32)
live in `world_gore_a/b.rs` + `player_gore_a..e.rs` + `player_dust_use.rs`,
drawn on the caller's stage stream in C# order with the gates ALWAYS
drawing and the POST-write snapshots riding the state structs; the
genRand stand-in is ONE `UpdatePlayers` throw per event (the
`spawn_things_from_pot` convention), seeded BEFORE the event's own
draws. The dispatch seams are the 5 wiring-wave drains (pot, player-tick
queues, death, tree-FX, ItemCheck) + the 3 wave-2 drains (shake-loot,
hair-dye, dust-only arms). The NPC death sites are d55/e5/f10's, the
projectile sites g16's, the leaf/cloud gores g68/d60's, the orchestrator
e4's — **no double-feed**: each family file has exactly one owner lane
(the §2a table), and the keep-one twins (sandstorm, msg-112) retire at
merge per their published contracts.

**Task-seeds for the merger (the UNCLAIMED residue, NOT this lane's):**

- `Mount.cs:6466/:6551/:6560` (3 sites) — the mount visual lane (the
  cart/drill/basilisk/type-23 gore bursts; the `MountDustRequest` drain
  precedent, e4 wiring batch 8).
- `PlayerDrawSet.cs:1318/:1326/:1369` (3 sites) — the player-draw buff
  VFX lane (palladium/loveStruck/shimmer; the `GoreCache` follow).
- `AmbientWindSystem.cs:111/:128` (2 sites) — the cloud-parallax lane
  (g69 candidate; g68's seal covers leaves only).
- `WorldItem.cs:653` (1 site) — the fallen-star burn's 3 gore siblings
  (g15 ports the dusts; the gore half awaits a world-item gore lane).
- `Utils.cs:1083` `PoofOfSmoke` (1 site) — the statue-poof lane
  (bug_net documents the skip).

These 10 sites are the family's open edge; they are recorded, not
claimed. The g17 lane's 68 are SEALED.

## 5. Verification record (code-only)

- Census: per-file `Select-String 'Gore\.NewGore'` (+ the Perfect/Direct
  variant split) over `decompiled/Terraria` and its subdirectories
  (junction recursion excluded — direct paths only); every g17 site's
  line number re-read against the C#.
- `DrawNPCChatButtons`: full-tree caller scan (zero hits beyond the
  definition); the `npcChatFocus` writer census.
- Registry: registry-150 §7 read at d54's tip + a fresh all-tip
  `git ls-tree docs/research/` sweep (max 280, 304 free — the authoring-time
  read, chain-history; SUPERSEDED by §8.6: 304 = d50's assignment, this doc
  renamed 304 → 339 per §9.2 row 339).
- The compose seam: `axe_tile_hit` `:1870-1901` re-read line-by-line
  against `WorldGen.cs:63131-63418`; the new witness written.
- Co-lane contracts: g58 research/240/242/243, g68 research/247, d120
  research/143, e4 research/66, f6 `573b7e7f2`, g15 `item_dust.rs` read
  at their origin tips.
- **No cargo/tests/rustfmt/graphify invoked.** The 60 witnesses are
  WRITTEN NOT RUN, owed the merge-verify window.
