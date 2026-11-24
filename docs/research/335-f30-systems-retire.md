# 335 — W4-f30 SYSTEMS RETIRE: keep-one drop-hunks + compose meeting points + the FINAL notify census

**Renamed 240→335** per the collision registry (`150-research-number-registry.md`,
W4-d54, `b243c7086` §9.2 row 335): g58's `143→240` self-rename (`cdfe3d108`,
12:16:37) WINS key 240; f30's first-add (`973ec04c4`) loses → 335. Chain 240→335.
The original numbering paragraph (historical):
free block 237+; the tip scan of all `task/W4-*` remotes shows 237 (e23 + g48),
238 (c8 + d42 + g61), 239 (d42) already claimed — 240 was scanned as the first
free number (superseded by the §9 adjudication above). Branch `task/W4-f30-achievement-notify-2`, worktree TR-W4-f30,
code-only (no cargo/tests/rustfmt; witnesses WRITTEN NOT RUN).

The f30 owning-systems batch landed six systems (gravestone-36, sun-scorch-33,
hardcore-35, town-shimmer-43-core, Dryad-Joja-46, TRASH-45). Five sibling lanes
landed overlapping halves on their OWN unmerged branches. This doc is the
retire phase: the exact **drop-hunks** (keep-one c46/d70 vs f30), the **compose
meeting points** (c24/f47/d2), and the **FINAL systems-family census**.

**Retire mechanics (IMPORTANT):** none of the sibling tips is merged into
`origin/main` (`a725acb04` — the wave-3 magic+summon landing) yet. Deleting the
f30 hunks on this branch now would break this branch's anchors canary
(`anchors.txt` rows point at the f30 functions; the c46/d70/f47 survivors are
NOT in this tree). So the drops are PUBLISHED here as exact hunk tables — the
repo's retire-hunk precedent (`143-d40-crimsongoldfish-retire-hunk.md`). The
merger applies them at merge time, in the order given per section. All line
numbers are at the tips named below (f30 = `64c4ee578` + this doc commit;
c46 = `7330cd4ee`; d70 = `a7620eec3`; c24 = `ff99f1486`; f47 = `1997be092`;
d2 = `f430ccf8a`).

---

## 1. c46 hardcore DUP — KEEP c46, DROP f30 (progression 35 EXTRA_LIFE)

Both lanes port `HardcoreDeathPenalty` (`Player.cs:38775-38787`) +
`KillMeForGood` (`:38789-38805`) + the `KillMe` gate (`:38864-38866`) + the
`UpdateDead` ghost latch (`:17044-17066`). Verdict: **c46 wins every shared
hunk**; f30 keeps nothing of system 3 except the 35-notify concept (c46 fires
it identically).

### Why c46 (evidence)

1. **The file deletion.** C# `:38795-38802` deletes EXACTLY two files: the
   `.plr` then `path + ".bak"` (string concat). c46's
   `player_file::kill_player_files` + `backup_path` (4445a1459) is that pair
   verbatim (Exists-then-Delete, concat not extension-swap). f30's
   `kill_me_for_good` (death.rs:413-437) rides `save_list::erase_player` — the
   `Main.ErasePlayer` SUPERSET (also removes the map-sidecar folder). c46 is
   the faithful port; f30's own comment admits the superset.
2. **The `:38803` reset.** C# `Main.ActivePlayerFileData = new
   PlayerFileData()` swaps the file-data singleton; `Main.player[myPlayer]`
   still points at the OLD Player so `difficulty` stays 2 and the ghost
   latches. c46 documents NOT porting the reset as a draft clear (its
   `Player.difficulty` field syncs from `session_draft` per tick —
   world_sim/player.rs:638 on its tip — clearing the draft would reset
   difficulty to 0 and break the ghost). f30 DOES clear
   (`app.session_draft = None`, death.rs:436): on f30's own base
   `active_player_difficulty()` (app/mod.rs:2404-2406) then answers 0 and the
   world_sim hardcore gate (world_sim/player.rs:651) never fires — f30's
   draft-clear is self-defeating on its own branch. c46's ruling is correct.
3. **The stat half.** Identical maths both lanes (`max(statLifeMax-60, 0)`
   ONLY inside the crossover; >0 survives + fires 35). c46 splits it
   Player-method (`Player::hardcore_death_penalty`, player.rs:3852-3862 tip)
   + App glue (death.rs:515-531 tip, the 35 notify at :527) — the house seam
   shape; f30's tuple-returning `death.rs::hardcore_death_penalty` (388-403)
   duplicates it.
4. **The ghost family.** c46 owns the FULL family: `Player.difficulty` field,
   `update_dead`/`update_dead_seeded` (the seeds-threaded form, player.rs:3866/
   3878 tip), `ghost_tick` (`Player.Ghost` :6715-6849), `draw_ghost`, the
   ghost-HUD gates, the breath/lava fidelity rows. f30's `update_dead` gained
   a 5th `hardcore_permadeath: bool` param (player.rs:3813-3823) — a DIFFERENT
   signature for the same C# arm; keeping both would not compile. c46's
   seeded form is the surviving seam (its anchors row
   `player.rs|fn update_dead_seeded` is live on its tip).
5. **The seed predicate.** Both are faithful to `Main.cs:2638-2648`
   (DST && Anniversary && !FTW). c46's private
   `Player::survive_hardcore_death` (player.rs:3837-3842 tip) is the survivor;
   f30's pub free fn (death.rs:375-381) drops with its callers.

### DROP table — f30 hunks the merger removes (keep c46's)

| f30 file | lines (f30 tip) | hunk | c46 survivor |
| --- | --- | --- | --- |
| `death.rs` | 369-381 | `survive_hardcore_death` free fn + doc | `Player::survive_hardcore_death` (player.rs:3837) |
| `death.rs` | 383-403 | `hardcore_death_penalty(app) -> (bool,bool)` | `Player::hardcore_death_penalty` stat half + `death.rs::hardcore_death_penalty` glue (c46 tip :515) |
| `death.rs` | 405-437 | `kill_me_for_good` (erase_player superset + draft clear) | `death.rs::kill_me_for_good` (c46 tip :553) + `player_file::kill_player_files`/`backup_path` |
| `death.rs` | 554-564 | `on_player_death`'s `if difficulty == 2 { let (notify35, _) = …; notify }` block | c46's `if difficulty == 2 { hardcore_death_penalty(app); }` AFTER DropItems (same slot; c46's glue fires 35 itself) |
| `death.rs` | 1035-1113 | tests `the_survive_hardcore_death_seed_predicate`, `the_hardcore_penalty_drops_sixty_and_fires_35`, `the_hardcore_death_fires_extra_life` | c46's `w4_c46_the_hardcore_death_deletes_the_plr_and_its_bak` + `w4_c46_the_hardcore_penalty_drains_60_only_inside_the_crossover`. **WITNESS UNION:** f30's `the_hardcore_death_fires_extra_life` asserts the achievement ROW completes through the real notify seam (c46's witnesses pin the files/stats, not the row) — re-pin it onto c46's glue (call `on_player_death` with the crossover seeds + difficulty-2 draft, assert `EXTRA_LIFE.completed`) and keep it as the 35 end-to-end witness |
| `player.rs` | 729-735 | `pub ghost: bool` field + doc | c46 `player.rs:743` (same field; dedupe to one) |
| `player.rs` | 2523 | `ghost: false` init | c46 `player.rs:2536` |
| `player.rs` | 3789-3807 | `allows_respawn_timer_skip` doc rewrite + `ghost_like` reading `self.ghost` | c46 player.rs:3819-3833 (identical bodies — dedupe) |
| `player.rs` | 3813-3872 | `update_dead`'s 5th `hardcore_permadeath` param + the gate block (3851-3864) + the unconditional pre-gate timer decrement (3841) | c46's `update_dead`/`update_dead_seeded` pair — the C# drains the timer ONLY inside the arm it takes (`:17047-17050` vs `:17057`); c46's shape is verbatim, f30's pre-decrement is behaviour-identical but the signature collides |
| `player.rs` | 13457, 13477, 13493, 13498, 13793-13828, 19965-19971 | the 4-arg `update_dead` test call sites f30 widened to 5 args (`, false`) | revert to c46's 4-arg calls |
| `player.rs` | 19979-20007 | test `the_hardcore_ghost_never_respawns` (5-arg form) | c46's ghost witnesses; re-pin this one onto `update_dead_seeded(…, seeds)` with a non-crossover seed set + `p.difficulty = 2` (it pins the expiry→ghost + skip-refusal, which c46's own tests cover — drop if redundant at merge) |
| `world_sim/player.rs` | 649-653 | the `hardcore_permadeath` fold (`active_player_difficulty() == 2 && …`) | c46's `p.difficulty` sync (:638 tip) + `update_dead_seeded(…, app.world.seeds)` call (:689 tip) |
| `world_sim/player.rs` | 674-680 | the 5-arg `p.update_dead(…, hardcore_permadeath)` call | c46's `p.update_dead_seeded(input.jump, boss_near, danger_near, moon_event_live, app.world.seeds)` |
| `anchors.txt` | f30 row `death.rs\|pub fn hardcore_death_penalty\|W4-f30 system 3 …` | retire the row (the marker dies with the drop) | c46's 3 rows (`player.rs\|fn hardcore_death_penalty`, `player_file.rs\|fn kill_player_files`, `death.rs\|fn kill_me_for_good`) — keep |

**KEEP from f30 in this family:** nothing structural. The 35 census row
(research/30) stays, re-attributed to c46. `Player.death_source_projectile`
(player.rs:736-740) is NOT part of this dup — it is system 1's (see §5).

**Merge order note:** c46's `on_player_death` glue inserts AFTER `drop_items`
BEFORE the coin bookkeeping; f30's block sits AFTER the cursor-return BEFORE
the tombstone. C# `:38861-38866` runs DropItems → HardcoreDeathPenalty →
PlayDeathSound → … → DropCoins/DropTombstone — **c46's slot is the C# slot**;
the merger keeps c46's position and drops f30's.

---

## 2. d70 sun-scorch DUP — KEEP f30's scan/33, DROP d70's copies; KEEP d70's render feeds (progression 33 FEAR_THE_SUN)

Both lanes port `Player.cs:28580-28719` (`UpdateSunScorchValues` /
`UpdateSunScorch` / `VampireSeedSunlightExposure`). d70 (`67fb6d4c5`) ports it
as Player methods for the lens-flare/HeatDistortion counter feed; f30
(`1a6d4b07d`) ports it as `sun_scorch.rs` free fns WITH the progression-33
wire. **d70 does NOT fire 33** (its `update_sun_scorch` has no vampire-set
check). Verdict: **f30's `sun_scorch.rs` is the ONE scan+ramp+burn**; d70's
three method copies drop; d70's unique render-side halves survive and re-feed
from the f30 field.

### Why f30 (evidence)

1. **The 33 wire.** Only f30 ports the crossing-tick vampire-set check
   (`:28639-28652`, effective head 124 / body 85 / legs 72 →
   `NotifyProgressionEvent(33)`); sun_scorch.rs:209-222 + the
   world_sim/player.rs:1255-1268 call site. Dropping f30's scan would unwire
   the progression — the whole point of the lane.
2. **The exposure scan fidelity.** f30 keeps the C#'s invisible-block/wall
   terms (`:28707-28717`: `!ShouldShowInvisibleBlocksAndWalls() &&
   tile.invisibleWall()` opens the sky; the 54/541 exceptions gate on
   `invisibleBlock()`/show) — sun_scorch.rs:111-143, with the STALE-tile quirk
   (the type/invisible terms read the PRE-decrement cell, `SolidTile3` the
   new one). d70 folds the invisible bits to false ("no port bit, deviation
   recorded") — but the port HAS the bits (`World::invisible_block/
   invisible_wall`, f30's witnesses drive them). f30 is strictly more
   faithful.
3. **The Molten order seam.** f30 alone carries `fire_immune_pre_molten`
   (player.rs:2098-2105, capture at :6945, restore at world_sim/player.rs:
   1269-1276) — the C# `UpdateSunScorch` (:25634) runs BEFORE `UpdateArmorSets`
   (:25685) so the warm-up burn tick keeps Molten's grant. d70's call site
   (player.rs:7028 tip, inside `update_equips` after the fold) has no restore
   — on a warm-up tick d70's Molten immunity would be wrongly withheld.
4. **The dead-tick ramp.** f30 wires `update_sun_scorch_values` on the dead
   branch (world_sim/player.rs:681-685, `Player.cs:17079`); d70 has no dead
   call site. KEEP f30's (it calls the sun_scorch.rs free fn — unaffected by
   the d70 method drops).

### DROP table — d70 hunks the merger removes (keep f30's)

| d70 file | lines (d70 tip) | hunk | f30 survivor |
| --- | --- | --- | --- |
| `player.rs` | 4787-4855 | `Player::vampire_seed_sunlight_exposure` (the invisible-folded copy) | `sun_scorch.rs::vampire_seed_sunlight_exposure` (66-144) |
| `player.rs` | 4857-4872 | `Player::update_sun_scorch_values` | `sun_scorch.rs::update_sun_scorch_values` (149-157) |
| `player.rs` | 4874-4895 | `Player::update_sun_scorch` (no 33, no Molten restore) | `sun_scorch.rs::update_sun_scorch` (177-241) |
| `player.rs` | 7025-7028 | the `self.update_sun_scorch(world, raining)` call at the `update_equips` tail | f30's world_sim/player.rs:1244-1277 block (the same C# slot, with the 33 notify + the order-seam restore) |
| `player.rs` | 2110-2114 | `pub sun_scorch_counter` field + doc | f30 player.rs:2091-2097 (dedupe to ONE field — both feed the same renders) |
| `player.rs` | 12166-12228 | tests `the_sun_scorch_counter_ramps_and_clamps` (:12172) + `the_vampire_exposure_probe_gates_on_seed_sky_and_shelter` (:12199) | f30's sun_scorch.rs witnesses (7 tests incl. the invisible-wall/bubble arms d70 cannot express). Drop both d70 twins — f30's `the_counter_ramp_and_cap_follow_the_source` + `the_weather_and_zone_gates_refuse_the_burn` cover the same pins |
| `anchors.txt` | d70 row `player.rs\|pub fn update_sun_scorch\|W4-d70 DOC-SWEEP …` | retire (marker dies) | f30 row `sun_scorch.rs\|pub fn update_sun_scorch` — keep |

### KEEP from d70 (unique halves — NOT duplicated by f30)

* `Player::update_advanced_shadows` / `get_advanced_shadow` /
  `reset_advanced_shadows` (player.rs:4756-4785 tip) + the ring fields + the
  teleport/spawn/TR_XY clears — the lens-flare shadow offset
  (`LensFlareElement.cs:34-37`). f30 never touches these.
* `render/screen_fx.rs` + `app/filters.rs` + `app/frame/draw.rs` — the live
  `sun_scorch_counter` feed into `SceneVisualInputs` (the HeatDistortion
  branches `SceneState.cs:98,146` + the lens-flare AdjustIntensity override
  `NextHorizonRenderer.cs:392-397`). These read the FIELD, which survives
  (f30's copy) — no rewire needed beyond the field dedupe.
* `world_sim/scene_scan.rs:665` — `sun_scorch_counter: player…` feed. Keep.
* `PlayerNet` team/shield/dead + the paladin peer loops (`ee4905939`) —
  unrelated to the dup, keep whole.

### ADAPT table — d70 hunks the merger keeps but re-points

| d70 file | lines (d70 tip) | hunk | adaptation |
| --- | --- | --- | --- |
| `player.rs` | 2115-2123 | `pub zone_graveyard` + `pub moon_lord_sky_intensity` fields | KEEP the fields (the render/scan feeds may read them) but f30's scan takes them as ARGS — the fields become write-only telemetry unless another consumer lands; simplest merge: keep d70's fields + feed, f30's call site keeps reading `app.scene_metrics` directly (both derive the same value) |
| `world_sim/player.rs` | 586-601 | the `p.zone_graveyard` / `p.moon_lord_sky_intensity` scene-scan feed | KEEP (feeds the kept fields). Note the perspective difference: d70 derives the Moon-Lord distance from `scene_metrics.center` (the screen centre), f30's scan computes it at the PLAYER centre from the live NPC list (the `SceneMetrics` perspective player — the C# reads `Main.LocalPlayer`). f30's scan is authoritative for the burn gate; d70's field is display-only |

**Net:** ONE scan (f30), ONE counter field, ONE 33 wire (f30), d70's
lens-flare/shadow/HeatDistortion halves ride the surviving field.

---

## 3. c24 Journey sacrifice — COMPOSE (progression 45 TRASH_COMPACTOR goes live)

f30 wired the CHECK (`research.rs::check_research_achievement`, the
`LastEditId` latch + `count_fully_researched` + both call sites) against an
inert tracker. c24 (`ff99f1486`) lands the LIVE sacrifice consumer
(`app/journey_clicks.rs` + `ui/journey.rs`), the faithful
`RegisterItemSacrifice` (cap-clamp + newly-unlocked), the unlock chain
(`is_fully_researched` → the duplication grid) and the `.plr` persistence
(`save_pairs`/`load_pairs` through the f44 seam). **c24 does NOT port
`CheckResearchAchievement` and never fires 45** (its tip has zero
`notify_progression_net(45)` / `check_research_achievement` hits) — no
double-fire. The two compose into the full C# chain:
sacrifice → count → unlock → (per-tick/forced) half-catalogue check → 45.

### Meeting point: `crates/tr-app/src/research.rs` (both lanes edit the same regions)

| region | f30 (tip) | c24 (tip) | compose verdict |
| --- | --- | --- | --- |
| `ResearchTracker` fields | `counts` + `last_edit_id: i32` | `counts` + `newly: Vec<u16>` | KEEP BOTH fields (disjoint C# state: `LastEditId` :23 vs `_newlyUnlocked` :19) |
| `register_sacrifice` | running total + `last_edit_id += 1` per call | cap-clamp (`Math.Min(have+amount, cap)` :131), capless/full-row early return (:122-130), `newly` arm (:135-137) | **c24's body wins** (the faithful clamp) **+ f30's `self.last_edit_id += 1`** at the C# `MarkContentsDirty` slot (:134 — AFTER the count write, BEFORE the newly arm; the early-return paths do NOT bump, matching the C#). f30's unclamped accumulation drops |
| `reset` | `counts.clear()` + bump | `counts.clear()` + `newly.clear()` | union: both clears + f30's bump (C# `Reset` :208-216 calls `MarkContentsDirty` :215) |
| `last_edit_id()` / `count_fully_researched` / `check_research_achievement` | f30-only | absent | KEEP f30 verbatim |
| `is_newly_researched` / `clear_newly_researched` / `sacrifice_numbers` / `items_with_progress` / `save_pairs` / `load_pairs` | absent | c24-only | KEEP c24 verbatim |
| tests | `the_tracker_accumulates_and_unlocks` asserts unclamped merge (`register_sacrifice(8,50)` twice → 100 — still passes under the clamp) + `register_sacrifice(6196,999)` no-op (passes) | adds the capless-count-0 assert + clamp/newly/persistence tests | union; f30's `f30_the_edit_id_latches_the_research_check` uses `register_sacrifice(8,1)` twice — under c24's clamp both register (1 then 2 < 100), each bumps — the latch pins hold unchanged |

### Meeting point: `App.research` (app/mod.rs)

Both lanes add `pub(crate) research: crate::research::ResearchTracker` +
a `ResearchTracker::default()` init — SAME name/type, dedupe to one field.
f30's doc comment ("the live sacrifice UI is unported… inert") is STALE after
compose — replace with c24's (the `.plr` creativeTracker block, Player.cs:
54911/:55696-55699). f30's `last_research_version: i32` + init stays (c24 has
no latch). c24's `journey`/`game_mode_difficulty_override` fields are unique —
keep.

### The load-path subtlety (verified against the C#)

`ItemsSacrificedUnlocksTracker.Load` (:171-196) does **NOT** call
`MarkContentsDirty` — a loaded tracker keeps `LastEditId` 0. c24's
`load_pairs` correctly does not bump. Consequence: after a `.plr` load the
per-tick check skips (version 0 == edit id 0) — exactly the C# — and the
**forced `EnterWorld` check is the load-path 45 site** (f30's
`enter_loaded_world` wire, app/mod.rs:2343-2351). Compose rule: the `.plr`
research load (c24's saveglue/player_file seam) must land BEFORE
`enter_loaded_world`'s forced check runs, or a past-half saved catalogue
misses its entry notify until the next sacrifice. The merger orders the load
first (the C# loads the player file before `Player.Hooks.EnterWorld`).

**45 double-fire audit:** f30's two call sites are the ONLY 45 sites in either
lane; `notify_progression` is idempotent per row (the manager latches
`completed`) — even a forced+per-tick double call completes the row once. No
drop needed.

---

## 4. f47 town shimmer — COMPOSE at the merge-meeting point (progression 43 NEW_DIGS)

f30's `town_shimmer.rs::town_shimmer_step(npc, wet, net_mode) ->
TownShimmerOutcome` is the RNG-free decision core (ai2 hold, climb, ≥30
transparency decay pinned while wet, (0,-4t) rise, ≥75&&t≤0 completion +
variant flip; the 43 gate rides `out.completed && out.variation_index == 1`).
f47 (`e5aebe3ed` + `6f126bd9e`) lands the FULL machine
(`npc_ai_town.rs::town_shimmer_state25` :1704-1839 tip — the same state writes
PLUS the landing-spot teleport, the hue light/309 dust/1-15 particle triple on
the UpdateNPCs stream, the TargetClosest read) and its own seam ALSO NAMED
`town_shimmer_step` (npc_ai_town.rs:1865-1948 tip, 8-arg, returns the 43 bool
via `TownOutputs::shimmer_town_progression`), WIRED at world_sim/npcs.rs:4641
(the slot loop, `shimmer_progression43` → the post-pass
`notify_progression_net(43)` at :6417).

**The name collision is cross-module (no compile clash), but the DRIVER must
be ONE.** f47's machine is a strict superset of f30's core (every state write
matches line-for-line; f47 additionally clears `shimmer_wet/honey_wet` — the
C# `:53194-53197` clears all four wet flags — and runs the visuals/teleport).
f47's doc-sweep 36 already wrote the merge contract (npc_ai_town.rs:1849-1864
tip + research/121): *"the merger keeps ONE driver — either this seam folds
into the [b3] adapter's TownOutputs drain or the adapter's state-25 call is
dropped in favour of this pre-pass… the b3 adapter's `shimmering: false` fold
must become the NPC's live buff-353 flag."*

### Compose verdict

1. **KEEP f47's `town_shimmer_state25` + its `town_shimmer_step` seam + the
   world_sim/npcs.rs:4641 wiring + the :6417 post-pass 43 notify.** It is the
   live, wired, visual-complete driver; the trigger (buff-353 ramp →
   `drain_shimmer_transforms`' town-set arm, world_sim/npcs.rs:3540 tip) is
   f47's too.
2. **DROP f30's `town_shimmer.rs` module + the `mod town_shimmer;` line
   (main.rs:263) + the anchors row `town_shimmer.rs|pub fn town_shimmer_step`
   + the npc_ai_town.rs:39-44 doc-note hunk** (the cited-skip note pointing at
   the core — f47's tip rewrites the same note to describe its own machine).
   The core's unique value was the conflict-free 43 seam; f47's seam supersedes
   it. **WITNESS UNION:** f30's `f30_the_client_never_completes` pins the
   `netMode != 1` completion gate (`NPC.cs:53251`) — f47's state25 reads
   `Main.netMode != 1`? **NO — f47's completion arm (npc_ai_town.rs:1816 tip)
   omits the net-mode gate** (its solo seam hardcodes `net_mode: 0` in the
   TownEnv but the completion `if` never reads it). At merge, add
   `&& env.net_mode != 1` to f47's completion condition (the C# has it) and
   re-pin f30's client witness onto `town_shimmer_state25` with
   `env.net_mode = 1`. f30's other four witnesses (ai2 hold / climb-decay /
   completion-flip / both-gates) are covered by f47's own state25 witnesses
   (npc_ai_town.rs:4963-5045 tip) — drop as redundant after a line-by-line
   compare at merge.
3. **43 double-fire audit:** f30's core has NO live caller on its own branch
   (the witnesses only) — its 43 gate is inert. f47's :6417 is the only live
   43 site. Keeping ONE driver means ONE notify. No further drop.
4. **f30's `npc_ai_town.rs` hunk is doc-only** (the module-header cited-skip
   note) — no code collision with f47's machine in the same file.

---

## 5. d2 epitaph/tombstone — COMPOSE (progression 36 GRAVE_MISTAKE; the aiStyle-17 flight)

Both lanes port the aiStyle-17 arm (`Projectile.cs:25316-25366`) + the
`DropTombstone` death spawn (`Player.cs:39108-39137`). d2 (`61c47ed79`,
earlier: 03:58 vs f30's 12:0x) ports the arm VERBATIM into
`projectile_ai_tombstone.rs::tombstone_step` (the mutable-world carve-out):
the x−0.75 / y−0.7 (last.y>1.5) bounce (`:19310-19320`), the blocked-move
prologue `Next(3)` (`:17948` — the FIRST operand, drawn even when the cut
gates fail), the `TileObject.CanPlace/Place` pair (place_rng taken/restored),
the Sign epitaph + Dig thud + Kill INLINE, the direction flip
(`AutomaticallyChangesDirection` :17283), wind-eligibility (`:22118` lists
17). f30's `ai_gravestone` (projectile.rs:7479-7528) is the SIMPLER stand-in:
no bounce (zeroes the blocked axis), no Next(3), placement via the
`wants_gravestone_place` intent drained in world_sim/projectiles.rs:489-505.

**d2's flight wins** (strictly more faithful; the RNG draw `Next(3)` per
blocked move is a determinism invariant f30's copy misses). **f30's 36
machinery + the epitaph date + the FTW hostile pair win** (d2 has none of
them — its tip has zero GRAVE_MISTAKE/OnKillNPC hits).

### Compose table

| piece | f30 (tip) | d2 (tip) | verdict |
| --- | --- | --- | --- |
| aiStyle-17 dispatch arm | projectile.rs:17401-17406 (`17 => ai_gravestone`) | projectile.rs:17044-17066 (`17 => TOMB_KINDS gate → tombstone_step`, mutable-world reborrow) | **KEEP d2's arm**; DROP f30's arm + `ai_gravestone` (7468-7528) + the `wants_gravestone_place` field (508-514) + init (811) + `drain_gravestone_places`/`kill_gravestone` (16170-16193) + the world_sim/projectiles.rs:483-505 drain block + the two test-rig field inits (30711, 31124) |
| `misc_text` field | projectile.rs:504-507 + init 810 | same field, same shape | dedupe to ONE (identical semantics) |
| `uses_wind_physics` | unchanged (no 17) | adds 17 | KEEP d2's |
| proj_data family row | data.rs `43 \| 201..=205 \| 527..=531 => (24,24,17,-1,…)` (the f30 block also carries the Junimo 995 row — KEEP that, system 5's) | identical tombstone row at a different match position | dedupe to ONE row (identical values); keep f30's 995 row |
| death spawn (`on_player_death`) | death.rs:565-609: roll → launch draws → spawn centred `cx−12` → `friendly=hostile=get_good` → `ai[0]=local_id` → `misc_text = death_text + "\n" + tombstone_date()`; the OLD `tomb_falls` visual push KEPT (double-visual deviation) | death.rs:484-548: same roll+draws order, INTEGER box halves, `friendly=true` only (FTW), `ai0=0`, `misc_text = death_text` (date dropped as non-deterministic), `tomb_falls` reduced to a bookkeeping record (flight rides the pool) | **d2's spawn shape wins** (the stand-in replacement is the correct retirement of `tick_tomb_falls` — verified DEAD: zero callers on either tip) **+ three f30 folds:** (a) `p.hostile = get_good` beside d2's `friendly` (the C# `:1265-1269` SetDefaults sets BOTH on FTW — d2's friendly-only is a miss; f30's `Projectile.hostile` field + the player-contact gate `!friendly \|\| hostile` at projectile.rs:19987 survive), (b) `ai0 = app.local_id as f32` (the C# `num4 = whoAmI` ai[0] — the ByProjectile attribution key; d2's 0 is the single-player stand-in, f30's is the faithful value), (c) the epitaph date line — see below |
| the epitaph date | `tombstone_date()` (death.rs:617-644, the civil-from-days `MMMM d, yyy` English-culture port of `:39128-39133`) | dropped ("wall-clock, non-deterministic") | **KEEP f30's date line** (`misc_text = format!("{}\n{}", death_text, tombstone_date())`). The C# DOES write it; the wall-clock read is the same class of deviation as the port's other `DateTime.Now` reads (the save timestamps) — dropping it changes the sign text every player sees. The determinism note moves to ANALYSIS (the date is display-only, never feeds RNG/sim). f30's witness `f30_the_death_spawns_the_gravestone_projectile` (death.rs:1121-1156) re-pins onto the composed spawn (its `ai[0]`/hostile/date asserts are exactly the three folds) |
| the 36 machinery | `grave_mistake_kill` latch (projectile.rs:515-518, 812, 23168-23184) + `drain_grave_mistake` (16195-16209) + the world_sim/projectiles.rs:506-510 notify + `Player.death_source_projectile` (player.rs:736-740, 2524, 3648, 4442-4448) + `hurt_with_source` (4230-4248) + the KillMe teammate arm (world_sim/npcs.rs:6567-6582) + the `is_gravestone` refuses_friendly bypass (22746, 22805) + the 4 f30 witnesses (25398-25545) | absent | **KEEP ALL f30** — d2 never fires 36. The latch/drain survive the flight swap unchanged (they hook `damage_pve`, not the AI arm). The two AI-arm witnesses (`f30_the_gravestone_falls_spins_and_scans_the_landing`, `f30_the_ftw_rest_zeroes_the_damage`) re-pin onto `tombstone_step` (d2's own witnesses cover the same head constants — drop as redundant after compare); the town-kill + killing-blow witnesses stay verbatim |
| anchors | f30 row `projectile.rs\|fn ai_gravestone` | d2 row `projectile_ai_tombstone.rs\|pub(crate) fn tombstone_step` | retire the f30 row's marker → re-point it at the 36 machinery that survives (`projectile.rs\|pub fn drain_grave_mistake`), keep d2's row |

**RNG order note (determinism):** both lanes draw the launch pair
(`Next(10,30)`, `Next(-40,-20)`) AFTER the kind roll on UpdatePlayers — the
streams agree. d2's per-blocked-move `Next(3)` rides UpdateProjectiles (the
C# stream) — the composed flight ADDS those draws vs f30's stand-in; that is
the faithful stream, and f30's three death-RNG witness twins already expect
the launch pair (no further re-pin).

---

## 6. The FINAL systems-family census (the f30 lane closes)

### Progression notify sites — the union at the f30 tip + the sibling retires

| id | row | site (surviving lane) | status |
| --- | --- | --- | --- |
| 1/3/5/25 | clock family | world_sim/clock.rs (base) | wired (base) |
| 6 | ITS_GETTING_HOT_IN_HERE | world_sim/editing.rs:1312 (base) | wired (base) |
| 8 | INTO_ORBIT | clicks.rs:1386 + npcs.rs (base) | wired (base) |
| 9 | WHERES_MY_HONEY | growth.rs:55 + npcs.rs:5414 (base) | wired (base) |
| 10/11/12/13 | boss kills | npcs.rs:2423-2435 (base) | wired (base) |
| 16 | BULLDOZER | npcs.rs:5373 (base) | wired (base) |
| 21 | MECHA_MAYHEM | npcs.rs:5313 + the editing.rs bookends (f30 batch 1) | **WIRED (f30)** |
| 23 | DEAD_MANS_CHEST | npcs.rs:2777 (base) | wired (base) |
| 27 | THE_GREAT_SHIMMER_TRANSFORMATION | shimmer.rs:463 (base) | wired (base) |
| 29 | BOOK_WORM | netglue.rs:2990 (base) | wired (base) |
| 33 | FEAR_THE_SUN | sun_scorch.rs + world_sim/player.rs:1267 (**f30 keep**, §2) | **WIRED (f30)** |
| 34 | ITS_SHALING_OUTSIDE | WorldGen.cs:71694 | CITED — boulder-rain unported (worldgen lane) |
| 35 | EXTRA_LIFE | c46 death.rs:527 (**c46 keep**, §1) | **WIRED (c46 after retire)** |
| 36 | GRAVE_MISTAKE | world_sim/projectiles.rs:509 + npcs.rs:6581 (**f30 keep**, §5) | **WIRED (f30)** |
| 37 | SPICY_LICKS | editing.rs:3848 (f30 batch 2, the 4715 arm) | **WIRED-partial (f30)** — the Axe/Carbon/Ivy chord arms CITED |
| 38 | ON_FLEEK | world_sim/player.rs:1720 (f30 batch 2) | **WIRED (f30)** |
| 39 | FORTUNE_FAVORS_THE_BOULD | Main.cs:61828 | CITED — rainbow boulder unported (d54 orch) |
| 40 | TRAINING_DAY | Player.cs:25697 | CITED — combat-book-v2 unported (g89b) |
| 41 | MINI_ME | world_sim/player.rs:1725 (f30 batch 2) | **WIRED (f30)** |
| 42 | (editing family) | editing.rs:7828 (base) | wired (base) |
| 43 | NEW_DIGS | f47 world_sim/npcs.rs:6417 (**f47 keep**, §4) | **WIRED (f47 after retire)** |
| 44 | MY_PEOPLE_NEED_ME | Player.cs:24076 | CITED — BordersMovement unported |
| 45 | TRASH_COMPACTOR | app/mod.rs:2350 + world_sim/player.rs:1743 (**f30 keep**, live after §3 compose) | **WIRED (f30 + c24 compose)** |
| 46 | INTERDIMENSIONAL_RECYCLING | dryad_stardew.rs (f30 system 5) | **WIRED (f30)** |
| 31 | QUEEN_MACHINE | NPC.cs:78857-78869 | CITED — the Mechdusa summon unported (the `seeds::mechdusa` gate landed, f30) |

### The systems-family declaration

**The W4-f30 achievement-notify lane is CENSUS-COMPLETE for its reserved
band** (progression 21, 33-41, 43-46 + the CheckResearchAchievement family):
every id in the band is either WIRED (its owning system ported, the notify
fires through `notify_progression_net`) or CITED with its C# anchor and its
owning lane named. After the §1-§5 retires/composes land at merge:

* f30 uniquely owns: 21 (the MechaMayhem machine), 33 (the sun-scorch scan +
  the Molten order seam), 36 (the GRAVE_MISTAKE latch + the killing-blow
  source + the epitaph date + the FTW hostile pair), 37-partial/38/41 (the
  armor/guitar tail), 45 (the check + both call sites), 46 (the Dryad Joja
  bit, verified LIVE), the `seeds::mechdusa` gate.
* c46 owns 35 + the whole hardcore ghost family; d70 owns the lens-flare
  shadow trail + the screen-shader feeds; f47 owns 43 + the whole shimmer
  transform family; c24 owns the sacrifice consumer + the unlock chain + the
  research persistence; d2 owns the aiStyle-17 flight.
* The band's remaining CITED rows (31, 34, 39, 40, 44) belong to active
  lanes (boss-summon / worldgen / d54 / g89b / the border-movement lane) —
  they are NOT f30 residuals.

No f30 witness runs were performed (code-only mode, user order); every
witness named above is WRITTEN NOT RUN, on its lane's tip. The retire drops
remove only f30/d70 code whose survivors carry their own witnesses; the
three witness re-pins (§1 EXTRA_LIFE row, §4 client-gate, §5 spawn folds) are
the merger's to write.

## Verification record

- `git fetch origin --prune` + per-tip `git ls-tree`/`git show` reads of
  c46 `7330cd4ee`, d70 `a7620eec3`, c24 `ff99f1486`, f47 `1997be092`,
  d2 `f430ccf8a`, d54's registry `150` (the number source).
- C# re-reads: `Player.cs:38775-38805/:38825-38866/:39108-39137/
  :28580-28719/:17044-17079`, `Main.cs:2638-2648`,
  `ItemsSacrificedUnlocksTracker.cs:116-225` (Load does NOT MarkContentsDirty
  — the §3 load-path rule), `NPC.cs:53187-53277`, `Projectile.cs:1258-1270/
  :2872-2936/:10876-10877/:25316-25366`.
- The free-number scan: all `task/W4-*` remote tips, `docs/research/` filenames
  ≥150 — 237/238/239 claimed, 240 scanned free at the time (superseded: g58's
  earlier rename wins 240; registry §9.2 assigns this doc **335**, chain 240→335).
- No cargo/tests/rustfmt/graphify (code-only order). No code edited by this
  retire phase — the drop/compose tables ARE the deliverable (the sibling
  tips are unmerged; on-branch drops would break this branch's anchors).