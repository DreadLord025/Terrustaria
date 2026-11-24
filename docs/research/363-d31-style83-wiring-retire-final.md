# 363 — W4-d31 aiStyle-83 WIRING + RETIRE/FINAL (the cultist-ritual family declaration)

> **CHAIN-BANNER (registry §9 rename, d54 tip b4c94432c):** this doc was self-picked
> as **352** (post-§9, tip `1f0bd6385`), colliding with d1's assigned 352
> (352 = d1, §9-EXECUTED `194c66ce6`). Renumbered **352 → 363** per the UNIFIED FINAL
> registry §9.2 row 363. Every reference below and in INDEX/ANALYSIS/PORT_MAP/ROADMAP/
> anchors.txt reads **363**; the original 352-claim narrative is kept as chain history.

Lane: **W4-d31** (`task/W4-d31-npc-ai-styles-75-83`), worktree TR-W4-d31.
Phase: the wiring wave + RETIRE/SEAL. **Code-only** — no cargo/tests/rustfmt/graphify
invoked; every witness below is **WRITTEN NOT RUN**.

Research number **363** per the registry-150 UNIFIED FINAL §9.2 assignment
(the original 352 self-pick below is CHAIN HISTORY, superseded by the §9 rename):

*Chain history — the original 352 claim:* Research number **352** per the registry-150 first-pushed-wins protocol
(`origin/task/W4-d54-dust-sites-world` `docs/research/150-research-number-registry.md`
§4-REFRESH: 281+ free). A fresh 362-tip `git ls-tree docs/research/` scan at the
RETIRE tip found the 281–351 block churning under parallel pushes (281/282/283/284/
293/298/304/305/308/309/338/341/342/343/345/346/349 taken at the scan; the high-water
mark 351). **352 is the first free number above the high-water mark** — the g58/d53
collision-margin precedent (a churning registry's first-free-at-scan can be claimed
by a parallel lane between scan and push; the above-max pick minimizes the race).

---

## 1. The git-log census — what landed in the proj-AI / NPC-style lane

The W4-d31 branch (`task/W4-d31-npc-ai-styles-75-83`) carries three arms over the
`a725acb04` wave-3 base:

| Commit | Arm | Scope |
| --- | --- | --- |
| `012eaa856` | arm 1 | claim aiStyles **76–82** on the NPC style axis (research/25 census): the ported machines (saucer core 395, Moon Lord core/hand/head, probe 399, True Eye 400, Leech Clot 401) move from the kind match to the axis arms (the S1-D-2 migration step, machines UNCHANGED verify-only); kind arms retired to `unreachable!` markers (the ScutlixRider precedent). 4 census witnesses. |
| `a31ff86ea` | arm 2 | port the MISSING aiStyle-**83** cultist ritual as unwired `npc_ai_style83.rs` (research/25 row 83): the verbatim `NPC.cs:36757-36925` arm — the 437 tablet's `localAI[3]` latch + `CultistRitual.CheckFloor`, the 379/438 ritual-four spawns, the struck-devotee release, the all-gone boss summon (ONE `Next(2)` side roll + `TabletDestroyed`), the 300-tick dissolve's three 526 bursts, the 438 devotee block. 8 witnesses. |
| `0487ea324` | arm 3 | docs + anchors: ANALYSIS "The cultist ritual" + "The aiStyle 75-83 axis census", PORT_MAP's style83 module row + the 76-82 axis-claim row, the ROADMAP W4-d31 catalog row, the two anchors.txt canaries. |

**This RETIRE commit (the wiring wave)** adds the style-83 dispatch + the sim
plumbing the d10 activation contract specifies (§2–3 below), re-pins the census
witness merge-safely, and adds the wiring witnesses (§5).

Cross-lane census (the d10 contract's producer re-read, `649b18aef`):
- **W4-d10** `8814cb4e` — the 526 `CultistBossParticle` `proj_data` row + the
  aiStyle-98 seek arm are LIVE (the projectile family's obligation). The packed
  `ai0`/`ai1` boss-centre payload is exactly the seek's read.
- **W4-d40** `7fec14aa` — rowed the **437/438** SetDefaults kinds (CultistTablet/
  CultistDevote, aiStyle 83) + the 440 BossClone. The kinds ride the roster wave;
  until they merge no ported kind has `ai_style() == 83` and the dispatch is dormant.
- **W4-d30** (65–72) / **W4-d32** (85–93) do NOT touch 83 — no re-port, no overlap.

---

## 2. The d10 activation contract — DISCHARGED

The d10 CultistTablet-437 activation contract (`649b18aef`, docsweep 26d §98)
specifies four wiring obligations. This commit discharges the ones the d31 lane
owns (the style axis + the sim dispatch); the rest are cross-lane notes.

**(1) The wiring wave routes 437/438 to `style83_step` on UpdateNPCs (the style75
precedent) + resolves spawns/links/retargets/died/tablet_destroyed.** ✅ LANDED:
- `npc.rs::update_rng_dust`'s `83 =>` axis arm (the solo path,
  `update_rng_style83_solo` over a one-slot pool) + the pooled branch in
  `world_sim/npcs.rs`'s tick closure (the live route, in slot order on the
  `UpdateNPCs` stream).
- `ritual_link_prepass` (the Moon Lord carrier write-through precedent): the
  tablet's stable ritual uids (`ritual_uids[4]` + `carrier_uid` for the boss)
  re-derive the slot-aligned ai fields each tick (`ai[0..1]` archers,
  `local_ai[1..2]` devotees, `ai[2]` boss — all slot+1; the C#'s `ai[2]` RAW slot
  translated). The post-summon `ai[0] = -1` latch HOLDS (the archer links must not
  overwrite the dissolve counter). The devotee's owner link (`ai[3] = -(slot+1)`)
  re-derives from `carrier_uid`; a missing tablet → 0 → the module's orphan kill.
- `resolve_ritual_requests` (the style75 carrier-spawn one-tick fold): allocates
  the `NewNPC` requests IN ORDER (`Bottom = (X, Y)`, `NPC.cs:80770`), hands out the
  stable uids, writes the tablet's links, runs `CultistRitual.TabletDestroyed`
  (`world_events::tablet_destroyed`, delay 43200). The retargets ran in-step (the
  module calls `target_closest` on the pool slots); `died` rides the death retain;
  `light` is the draw-side feed's.
- The module's **0-link wiring guards**: a 0 link is an inactive slot (the port
  culls the dead); the release block + the summon scan skip it (no `as usize - 1`
  underflow, no write to a stranger slot) — behaviour-identical to the C#'s
  inactive-slot semantics (`SetDefaults` resets on reuse, `NPC.cs:80768`).

**(2) The 526 drain OWNER RE-PIN — the C# call sites pass `Main.myPlayer` NOT the
hostile default; the sim's generic `wants_hostile_shot_ai` drain hardcodes 255; the
wiring must spawn owner 0 (the fishron `spawn_hostile_ai_owned` precedent) for the
channel-track tail + owner semantics.** ✅ LANDED:
- The generic `ai_shots` drain (`world_sim/npcs.rs`) now branches on `shot.kind ==
  526`: the particle spawns via `spawn_hostile_ai_owned(kind, app.local_id as u8,
  …)`; every other kind keeps the stock 255 `spawn_hostile_ai` drain.
- The tablet's **death volley** rides a side channel (`ritual_death_shots`,
  captured in the tick closure): the tablet fires its death volley on the tick it
  dies (`NPC.cs:36852-36876` sets `life = 0` THEN spawns), so the shots cannot ride
  the dying NPC's `wants_hostile_shot_ai` (the death retain culls it before the
  generic drain). The side channel spawns them owner-re-pinned after the retain.
  The **trickle** (living tablet) rides the generic seam (the 526 re-pin applies).
- The owner-0 spawn arms the channel-track tail (`Projectile.cs:11097-11100`,
  `pending_channel_tracks`); the stock 255 drain leaves it inert — the observable
  difference the re-pin buys (witness §5.4).

**(3) The row + `ai_cultist_particle` are d10's live (`8814cb4e`); the packed payload
is exactly the seek's read.** ✅ CROSS-LANE NOTE: the 526 `proj_data` row + the
aiStyle-98 seek are d10's; this wiring only produces the `HostileShotAi` payloads
(`ai0`/`ai1` = the boss centre / the gated launch target). No stream conflict: the
tablet draws on UpdateNPCs, the flight draws on UpdateProjectiles, c1's boss-84
untouched.

**(4) No stream conflict.** ✅ CONFIRMED: the ritual's draws (the ONE `Next(2)` side
roll + the dissolve's 3-draw ring particles + the 20 gated 4-draw launches + the
trickle) all ride the `UpdateNPCs` stream in-step, in C# textual order (the module's
RNG census, `npc_ai_style83.rs` §"RNG order"). The pre-pass / resolve / drain passes
are RNG-free by construction.

---

## 3. The movement tail + the light feed

- **`Npc::style83_movement_tail`** (`NPC.UpdateNPC`, `NPC.cs:90465-90498`): the
  tablet 437 is noGravity + noTileCollide (`:14189-14190`) → the dead-band X snap
  (`:90473-90476`) + the raw integration (`:90494-90498`, the style-56 arm's shape).
  The devotee 438 writes NEITHER flag (`:14195-14211`) → the 0.4 gravity + the 8 fall
  cap (`:90465-90472`, the zombie GRAVITY/MAX_FALL rows) + the tile collision
  (`UpdateCollision`, the port's `collides_rect` axis stand-in, the zombie
  integration's shape minus the fighter scan's jump fallback — the devotee has no
  obstacle scan). RNG-free.
- **The light feed** (`app/frame/draw.rs`): the tablet's constant
  `AddLight(Center, 0.8, 0.75, 0.55)` (`NPC.cs:36921-36924`). Keyed on `cs_id() ==
  437` (kind-agnostic — dormant until the roster wave), the `XAnchor::Center` form
  (`Lighting.cs:140-143`'s `(int)(v/16f)` quotient truncation, NOT the ForceBubble's
  truncate-first). Fed only while the tablet lives (the `hp <= 0` gate — the C#'s
  early-return paths skip the tail too). RNG-free.

---

## 4. The recorded folds / deviations (ANALYSIS cross-ref)

- **The 379 archer has no ported kind** (no bestiary row — the roster wave). Its
  spawn resolves to `None` in `resolve_ritual_requests` and its link stays 0; the
  summon gate reads it gone. The 438 devotee + 439 boss resolve through
  `kind_from_net_id` once W4-d40 rows them (439 → AncientCultist already live).
- **The devotee orphan kill**: the C# reads the inactive owner slot's stale centre
  (`Main.npc[-1]` is the C#'s own sentinel-guarded path); the port culls the dead
  tablet, so a missing owner → the module's `ai[3] = 0` → owner -1 → orphan kill
  (`:36897-36903`). The one-tick fold is the carrier write-through family's.
- **The death-volley side channel**: the dying tablet is culled by the death retain
  before the generic shot drain, so its death volley rides `ritual_death_shots`
  (captured in the tick closure, spawned owner-re-pinned after the retain). The
  trickle rides the generic seam. Both spawn owner 0 (the re-pin).
- **The summon-scan spawn-tick fold**: the C#'s `NewNPC` is synchronous, so the
  `:36805` scan on the latch tick reads the four fresh ritual NPCs (flag94 false);
  the port's links resolve after the tick, so the scan is skipped on the spawn tick
  (the module's `spawned_this_tick` guard — behaviour-identical, the style75
  carrier-spawn one-tick fold precedent).

---

## 5. The witnesses — WRITTEN NOT RUN (code-only)

**Module (`npc_ai_style83.rs`, +2 wiring guards over the 8 arm witnesses):**
1. `the_zero_link_guards_skip_the_inactive_slots` — all-0 links: the release latch
   advances, NO retarget runs, the summon arms (ONE `Next(2)`), no underflow panic.
2. `the_partial_links_release_only_the_living_slots` — one living archer link: only
   it retargets + drops its link; the living 379 blocks the summon; zero draws.

**Axis census (`npc.rs::w4_d31_axis_census_tests`, +2 movement-tail, 1 re-pin):**
3. `w4_d31_the_band_kinds_carry_their_csharp_styles` — RE-PINNED merge-safely: any
   style-83 kind must be 437/438 (was "no kind claims 83"; breaks when d40 merges).
4. `w4_d31_the_devotee_movement_tail_falls_and_lands` — the 0.4 gravity, the free-air
   integration, the floor snap (y = 440), on_ground, the 8 fall cap.
5. `w4_d31_the_movement_tail_dead_band_x_snap` — the ±0.005 dead-band snap; outside
   the band keeps vx.

**Sim wiring (`world_sim/npcs.rs::w4_d31_style83_wiring_tests`, +4):**
6. `the_prepass_is_dormant_until_the_ritual_kinds_land` — the dormancy gate: a
   non-83 placeholder carrying ritual uids is untouched; no roster kind claims 83.
7. `the_resolve_pass_spawns_the_boss_and_resets_the_delay` — the 439 boss spawn
   (Bottom placement, uid, facing, despawn 937), the carrier_uid link,
   TabletDestroyed → cultist_delay 43200.
8. `the_resolve_pass_folds_the_unported_ritual_four` — the 379/438 spawns fold to
   None pre-merge; ritual_uids stay 0; no TabletDestroyed.
9. `the_owner_zero_repin_arms_the_channel_track_tail` — owner 0 arms
   `drain_channel_tracks` (1 intent); the stock 255 drain leaves it inert (0).
   (Uses the rowed SHARKNADO_BOLT 385 to pin the OWNER mechanism kind-agnostically —
   the 526 proj_data row is d10's live; the re-pin's contract is the owner.)

The machine's own behaviour (the latch, the release, the summon's ONE side draw, the
dissolve's full RNG census, the devotee block) stays pinned by the module's 8 arm
witnesses (`a31ff86ea`), UNCHANGED by the wiring (the guards only skip inactive slots).

---

## 6. The cultist-ritual FAMILY DECLARATION

ONE family, three lanes, DISJOINT ownership (the spawn-family bible convention):

| Piece | C# anchor | Port | Owner | Status |
| --- | --- | --- | --- | --- |
| The aiStyle-83 machine (the tablet latch / release / summon / dissolve + the devotee block) | `NPC.cs:36757-36925` | `npc_ai_style83.rs::style83_step` | **d31** | LANDED `a31ff86ea`, WIRED this commit |
| The style-axis dispatch + the sim plumbing (pre-pass / resolve / 526 owner re-pin / movement tail / light feed) | `NPC.AI()`'s `aiStyle == 83` arm + `NPC.UpdateNPC:90465-90498` | `npc.rs` 83 arm + `world_sim/npcs.rs` + `app/frame/draw.rs` | **d31** | LANDED this commit |
| The 437/438 SetDefaults kinds + the 440 BossClone | `NPC.cs:14176-14246` | `npc/data.rs` rows + bestiary | **d40** | LANDED `7fec14aa` (roster wave) |
| The 526 CultistBossParticle proj_data row + the aiStyle-98 seek | `Projectile.cs` (the 526 row + AI_098) | `projectile/data.rs` + the seek arm | **d10** | LIVE `8814cb4e` |
| The 439 CultistBoss machine (AI_084) | `NPC.cs:64414-65388` | `npc_ai_cultist.rs` | **c1** | UNWIRED (its own lane) |
| The ritual delay machine (TabletDestroyed / CultistSlain / UpdateTime) | `CultistRitual.cs:21-59` | `world_events.rs` | main | LIVE (the delay 43200 reset wired here) |

**No two family branches own the same fn body.** The d31 machine + dispatch are
disjoint from d40's kind rows, d10's projectile row/seek, and c1's boss-84 machine.
The 379 archer (aiStyle 3) is the roster wave's kind obligation (no ported kind yet —
the recorded fold §4). The family activates end-to-end when d40's 437/438 kinds merge
(the dispatch is dormant until then); the 526 flight is already live (d10).

**Gate command** (the catalog command, judge by `test result: ok` lines):
`cargo test --workspace -- --skip migrates_version_2_kinds_to_item_ids`

**No further d31 commits planned** — the wiring + RETIRE/FINAL close the lane. The
merger composes d31 (machine + dispatch) + d40 (kinds) + d10 (526 row/seek); the
census witness is re-pinned merge-safe (§5.3), the anchors canaries stand.