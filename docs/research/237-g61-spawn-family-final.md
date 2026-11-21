# 237 — W4-g61 NPC spawn / teleport-in dust: FINAL (the spawn-family declaration)

(Number per the d54 research/150 registry: 151-236 consumed, 237+ free
for future lanes — this doc takes the first free number.)

**STATUS: FINAL** — 2026-11-21, `task/W4-g61-npc-spawn-dust`, the
final-phase commits `e6fcb643` (keep-one fix) + `df15960a` (crimson +
activation census) + this doc. Code-only lane: no cargo/tests/rustfmt/
graphify; every witness WRITTEN NOT RUN (the merge-verify wave executes).

This note closes the lane. It records (1) the adjudicated keep-one
against g11's EXECUTED union, (2) the NightKey crimson wiring onto the
g69 `World.crimson` field, (3) the roster-gap activation census
re-derived from the pushed kind-lane tips, and (4) the spawn-family
declaration — what stays g61's, what the merger drops, and the co-lane
dup contracts.

## 1. The keep-one — g11 canonical (research/116 EXECUTED + FINAL)

The lane's earlier note (commit `1eedb9a3`: "supersedes g11 main_dust
styles 0-5/7 at merge") was the INVERSE direction of the agreed
keep-one and is RETIRED. Re-read from
`origin/task/W4-g11-dust-main-sites-1` tip `98283df0` (research/116
§3/§4/§5; union-exec `56afc0c0` + `8540a8ad`, final-phase folds
`cc8c767b`):

* **Body:** g11's `crates/tr-app/src/teleport_dust.rs` is the surviving
  copy of g61's widest-eleven `Main.TeleportEffect`
  (`Main.cs:13233-13464`) — adapted per-hunk from g61 `a9628928`
  (`TeleportDustEnv` = `SpawnVisualsEnv` minus the gore pool; no
  teleport arm spawns gores). Both branches always agreed the g61 BODY
  survives; g11 executed the fold on ITS branch.
* **Seams:** g11's `main_dust.rs` — the App-level `teleport_effect`
  dispatcher + the Entry-OLD→Exit-NEW `PlayerTeleportReq`/`NpcTeleportReq`
  replays (gap 2 `other_position` both legs, gap 3 the style-9 pylon
  latch, gap 4 the stored `App.water_style`, gap 5 the style-8 double
  cue + the replay-owned camera transition, gap 6 the grant dedupe).
* **Fold-source on THIS branch (marked DEPRECATED in-code, the merger
  drops):** `npc_spawn_visuals.rs`'s `teleport_effect` / `npc_teleport`
  env-level driver / `update_npc_teleport_visuals` triple shape /
  `portal_color` / `pylon_spawn_in_world_dust`; the 9 teleport
  witnesses (g11 union-adapted 8 as `w4_g11u_*` in `teleport_dust.rs`
  + the driver witness as
  `w4_g11_npc_teleport_style4_two_sided_burst_and_latch` in
  `main_dust.rs`); `world_sim/wiring.rs::npc_teleport_wiring` (the §5
  DUP RISK — the merger keeps the g11 QUEUE replay: the pad loop
  already rides it and the §5 Fold B queues the statue style 14
  through the SAME drain with the `NPC.cs:81052-81053` grants latched);
  the `world_sim/npcs.rs` copy-in/out fade drain (g11's surviving
  `update_npc_teleport_visuals` takes `&mut Npc` on the g11 fields —
  `teleport_style_id` naming, ONE field set — at the SAME pre-AI slot,
  `NPC.cs:90322` after the cast lights / BEFORE `AI()` `:90423`, fed
  `tfv = anim_time*60 % 216000`, the §5 Fold A of g61's seam-4 order).
* **Stays g61:** the poof/mimic/transform/alt-texture halves + their 8
  pure witnesses + ALL the wiring-seam witnesses (the §5 dup census:
  seams 2/3/5 NO-DUP). `Audio::play_teleport_cue` survives (the §5
  audio note — the merger may hoist the g11 dispatcher's inline
  resolution into the named helper, zero behaviour change).
  `make_transition_effects_for_texture_changes` stays (the town
  alt-texture seam is outside the union). The frog's style-13
  `NPC.Teleport` (`NPC.cs:79880`), when its lane lands, must ride
  `main_dust::npc_teleport` on the caller's pass — not a g61 driver.
* The g11 `snowing` site is separately adjudicated (g11 research/83 —
  d130-canonical, DEPRECATED on the g11 branch).

## 2. The NightKey crimson pick — LIVE

`NPC.BigMimicSummonCheck` (`NPC.cs:77517-77605`, re-read) picks
`num7 = 475` for the Light Key; the Night Key (`num3 == 1`) picks
`!WorldGen.crimson ? 473 : 474` (`:77594-77598`). The seam-6
constants-off stand-in (`let crimson = false`) is RETIRED:
`chest.rs::big_mimic_summon_check` reads `app.world.crimson`.

The field is **g69's** (`origin/task/W4-g69-weather-clouds-parallax`
tip `20d0380f`, field-shape read): `World.crimson: bool`
(`WorldGen.cs:4158` — true crimson, false corruption), declared after
`mushroom_bg_style`, ctor `false`, written `world.crimson =
opts.crimson` at BOTH worldgen entries (`generate_with_progress` +
`generate_with_chest_stock`; `GenOptions.crimson` / `roll_crimson(seed)`
already live on main). This branch carries g69's field/ctor/worldgen
hunks VERBATIM — `git diff --no-index` against the g69 blobs is EMPTY
for both files' crimson hunks (the merge dedupes to one, no semantic
conflict). The `.trwld` v18 tail + the `.wld` flags-byte read
(`WorldFile.cs:1281`/`:2080`) stay g69's lane — NOT duplicated here.
Until g69's save layer merges, a LOADED world keeps the ctor `false`
(corruption — the same standing default the constants-off had); a
GENERATED crimson world now picks 474 correctly. The shop.rs/loot.rs
`crimson_world` folds keep their own lanes' rows (g69 research/117 §2's
consumer census).

Witness: `w4_g61_the_crimson_night_key_clears_and_reads_the_world_evil`
(the flag plumbs through, the kind-row refusal draws ZERO on the
`UpdatePlayers` stream, the Light Key ignores the evil flag). WRITTEN
NOT RUN.

## 3. The roster-gap activation census (pushed tips re-derived)

The seam-5/seam-6 "roster-gapped" notes were true at `8f545007`. The
kind lanes have since landed rows on their PUSHED branches:

| C# id | kind | landed | activation of the g61 seams |
| --- | --- | --- | --- |
| 473-476 | the big mimics | d40 `4860cd88b` (batch 9; own bestiary entries, `bestiary.rs:2376-2400`) | **AUTO** — `kind_from_net_id` (`npc/data.rs:3996`) scans the bestiary ENTRIES; at the d40 merge the key-summon gate resolves and the `NewNPC` + `BigMimicSpawnSmoke` arms fire with ZERO g61 edits. The aiStyle-87 machine is d32's (UNWIRED row-0 FindFrame per the d40 note) — the spawn + smoke are live regardless |
| 478/479 | MothronEgg / MothronSpawn | d40 `cfed253d0` (batch 10; 479 own row `:2417`, the egg Hide=true credit-folds onto Mothron `:2634`) | **SEMI-AUTO + GAP** — the egg's `Transform(479)` timer (`NPC.cs:38380`) is the aiStyle-89 machine (d32 module, g8 dispatch claim `out.transform = Some(479)`); the sim's `wants_transform` drain (`world_sim/npcs.rs:6195`) matches ONLY `382/518` — the merger adds the `479 => NpcKind::MothronSpawn` arm (the c10 research/149 §2.4 contract — renumbered from 145 at the c10 seal `5fcc68ae6` — names this the d40 wiring step). The g61 `transform_visuals(478, 479)` arm fires from the SAME drain's parity call once the arm lands. **478 RESOLUTION GAP:** the egg has NO own ENTRIES row (Hide=true), so `kind_from_net_id(478)` resolves `None` even after d40 — the fix is the d40 lane's (an own row — the g8 VampireBat precedent, `bestiary.rs:2353-2354` — or a fold-aware lookup). Declared, NOT fixed here |
| 158 | VampireBat | g8 `0f31e10e3` (batch 3; own row `:2421`, Hide=true folds onto Vampire 159 for credit) | **GATED** — the flip callers (`NPC.cs:23353` aiStyle-14 / `:58850` aiStyle-3) are cited machine skips on BOTH branches (g8's `npc_ai_style14.rs:122` still says "VampireBat unported" — the row landed, the Transform branch did not). The 158↔159 arm needs BOTH rows + the flip callers |
| 159 | Vampire | d36 CLAIMED, NOT landed (research/109 census; `bcf4ac551` supersede-note — 108-E `bdd59afb` authoritative, the 762-789 port-local band; d36/d36x tips are census-only) | as above — no auto-activation |
| 687 | MysticFrog | **NOWHERE** — d43's tip is bare main (`a725acb04`, no kind commits); e26's Frog is **361** (the luck critter, `048cca6cf`, `bestiary.rs:2182` on e26 — a DIFFERENT kind); d127's `SpawnFrog` gates are 361 too | the bug-net `CatchOutcome::MysticFrog` arm stays coded-for-parity (the poof fires when a 687 record arrives; `AI_AttemptToFindTeleportSpot` unported — the d43 kind lane's). The style-13 teleport rides `main_dust::npc_teleport` when it lands (§1) |
| 361 | Frog (the critter) | e26 `048cca6cf` (data-only aiStyle-7 walker) | NOT a g61 consumer — the catch CONVERTS (`catch_item(361) = 2121`, `NPC.cs:13611` — the same Frog item the 687 row uses, `:17651`), no poof arm |

**The auto-activation verdict:** YES for the key-summon seam (a table
read — the d40 rows activate the NPC + smoke at merge); NO for the
transform seams (the hardcoded `382/518` kind match + the missing flip
callers). The g61 witnesses pinning "no NPC spawns yet"
(`w4_g61_the_light_key_summon_clears_the_chest`'s `app.npcs.is_empty()`)
are TRUE on this branch and INVERT at the d40 merge — the merge-verify
wave re-pins them (the roster rows are not this lane's to add).

## 4. The spawn-family declaration

The g61 lane owns the SPAWN-time visual family. Final ownership:

**STAYS g61 (wired on this branch):**
* `poof_of_smoke` (`Utils.cs:1078-1093`) — the bug-net catch
  (`NPC.cs:79883`/`:79912`), the statue body-gate
  (`Wiring.cs:2275`/`:2329`/`:2397`), the lamp burn-out
  (`Wiring.cs:824-826`).
* `big_mimic_spawn_smoke` (`NPC.cs:77494-77515`) — the key-summon
  (`:77602`), the crimson pick now LIVE (§2).
* `transform_visuals` (`NPC.cs:80963-81033`) — the three live pillar
  flips (406→405 / 428→427 / 427→426, the `wants_transform_visuals`
  stamps) + the carrier-gone parity drain (390→382 / 416→518,
  zero-draw).
* `make_transition_effects_for_texture_changes`
  (`NPC.cs:90236-90263`) — module-ready; the town alt-texture seam is
  b3/e19's.
* The `Npc` teleport fields (ONE set at merge, g11 names),
  `wants_transform_visuals`, `Player.last_chest` +
  `chest_change_events` + `big_mimic_summon_check`,
  `Audio::play_teleport_cue`.
* The 8 pure poof/mimic/transform/alt-texture witnesses + the 12
  wiring-seam witnesses (+ the final-phase crimson witness).

**FOLD-SOURCE (DEPRECATED in-code, the merger drops):** the teleport
family + `npc_teleport_wiring` + the triple fade drain + the 9
teleport witnesses (§1).

**CO-LANE DUP DECLARED (c10):** `task/W4-c10-style68-waterfowl`
(`67191299b` + `1e336ee93`) carries `npc_transform_visuals.rs` — a
DRAW-CENSUS port of the SAME `NPC.TransformVisuals` body (the five
arms; the Item8 cue as an intent; dust/gore bodies unmodelled) with
the SAME three pillar flip sites wired. g61's `transform_visuals` is
the FULL pool-writing port (the d50/S1-E-4 contracts — the gores
land). At the c10×g61 merge keep ONE: **g61's full body** (c10's
census is a strict subset — it draws nothing into the pools), c10's
flip-site call shape re-points at the g61 module or retires (the g61
`wants_transform_visuals` stamp already covers the three sites),
c10's waterfowl-silence witness transfers (the eight takeoff/landing
pairs match no arm — zero draws, both ports agree). The merger
adjudicates; neither branch edits the other's module.

**CITED SKIPS (unchanged):** the town shimmer-state-25 teleport (b3),
the caster inline bursts (d52 batch3), the net-apply teleports
(g48-g52), `GasTrapCheck` (no gas-trap family), the `NewNPC`
getGoodWorld bunny/squirrel swap (constants-off at the sim's spawn
seams), the death dust (g59/g60) + the per-hit dust (g62) — separate
lanes, untouched. The `time_for_visuals` stand-in
(`anim_time*60 % 216000`, the draw.rs:374 convention) survives on the
g11 fade wiring (the §5 Fold A shape).

## 5. Verification

Code-only per the lane directive (no cargo/tests/rustfmt/graphify).
The C# re-read: `NPC.cs:77517-77605` (the summon), `:77494-77515`
(the smoke), `:79860-79924` (the catch/frog), `:80963-80992` (the
transform arms). The g69 field hunks diffed against `20d0380f`
(`git diff --no-index` — EMPTY for the crimson hunks). The
d40/g8/d36/d36x/e26/d43/c10/g11 tips re-read from origin
(`git log`/`git show`/`git grep` on the refs, no checkouts). The
new/updated witnesses are WRITTEN NOT RUN.