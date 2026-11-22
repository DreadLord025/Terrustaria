# 91 — DOC-SWEEP lane 15: the fighter AI (aiStyle 3) — d22 core probe, graveyard bash gate, door-open census, confused generic arm

W4-g4 branch `task/W4-g4-npc-style3-fighter`, DOC-SWEEP phase (base DONE
`53063dd1`). Code-only lane: witnesses WRITTEN NOT RUN (no cargo invoked).
Commits this phase: **P5 `743d3a02`** (graveyard bash gate), **P6
`1419f7d1`** (confused generic walker arm), this doc.

## 1. d22 style-3 CORE status — probe result: STARTED (local), UNPUSHED

Probe (read-only, 2026-11-21):

* `git ls-remote origin task/W4-d22-npc-ai-styles-3-6` →
  **`a725acb0`** — the wave-3 landing base only. The branch's own work is
  **not on origin**.
* Worktree `C:\Users\DreadLord\Desktop\TR-W4-d22`, branch
  `task/W4-d22-npc-ai-styles-3-6`, two local commits ahead of the pushed
  ref:
  * **`60712154`** — "W4-d22 style-3: the shared AI_003_Fighters core
    (NPC.cs:55856-60298) claims the axis for the 22 plain walkers — head
    flip/chase/despawn/wander-flip/ladder-every-tick/obstacle-scan/
    integration; scan generalised (flag8/wide-probe/charge-hop from live
    type), GoblinPeon door-smash drain, idle-counter gate extended; RNG
    draw-free (sim seams own counter+moans); witnesses written not run".
    Touches `npc.rs` (−187/+…), new **`npc_ai_style3.rs` (454 lines)**,
    `npc_ai_fighter_scan.rs` (+250), `ambient_sound.rs`, `main.rs`,
    `world_sim/npcs.rs`, `anchors.txt`, `docs/ANALYSIS.md`.
  * **`c1d26278`** — style-4 EoC RNG census+fix (a different style; not
    lane 15's concern).
  * The d22 worktree also carries **uncommitted** changes at probe time
    (`npc.rs` 711 lines, `npc_ai.rs` 385, `projectile.rs`/`data.rs`,
    `world_sim/npcs.rs` — 874 insertions): the lane is mid-flight.

**Gate decision:** the Lead's port condition was "if core UNSTARTED and
main's fighter arm is stand-in, port AI_003 core verbatim". The core IS
started (`60712154` covers the whole `:55856-60298` region, including the
walk/jump/fall/knockback phases inside `:58000-60500`), so the verbatim
re-port is **not triggered** — it would collide with d22's in-flight
`npc_ai_style3.rs` axis claim on merge. Lane 15 instead lands the four
sweep items below on the CURRENT main-shaped arms.

**Merge-risk note (for the Lead):** d22's `60712154` rewrites
`npc_ai_fighter_scan.rs` (+250 lines, the generalised scan) and
`update_zombie`'s dispatch region — exactly the files P5 touches. When
both branches land, the reconcile must keep (a) d22's axis claim + the
generalised scan and (b) P5's `target_zone_graveyard` pre-stamp + the
`rng`-threaded `fighter_obstacle_scan` signature + the graveyard chain
term. The two are complementary (d22's core is documented "RNG
draw-free (sim seams own counter+moans)" — P5's one conditional
`Next(60)` is the first AI-side draw the merged scan will carry; the
stream-parity witness `w4_g4_the_graveyard_bash_draws_on_the_update_npcs_stream`
pins its position).

## 2. Graveyard-zone bash gate — WIRED (P5 `743d3a02`)

The C# (`NPC.cs:59997`, inside the 60-tick door bash):

```csharp
bool flag25 = Main.player[this.target].ZoneGraveyard && Main.rand.Next(60) == 0;
```

flag25 feeds flag27 (`:60007`): a graveyard 1/60 roll cancels the
normal-night `ai[1]` reset, so even the flag24 door-bash types accumulate
toward the open on that tick. The port's S1-D-3z scan hardcoded
`flag25 = false` (the "unported zone" zero-draw short-circuit) while
`scene_metrics.zone_graveyard` has been LIVE since d120
(`tr-core/src/scene.rs:775`, `world_sim/scene_scan.rs:653`; the moan gate
already reads it, `world_sim/npcs.rs:4180`).

Wiring (the `wind_speed_current` pre-pass convention — the AI's `&World`
borrow cannot reach `app.scene_metrics`):

* `Npc::target_zone_graveyard: bool` — new pre-stamp field
  (`npc.rs`, ctor false). Single-player port: the session player's zone
  stands in for `Main.player[target].ZoneGraveyard`.
* `world_sim/npcs.rs::update_npcs` — a pre-pass stamps every slot from
  the captured `zone_graveyard` (the same value the S1-E-21e moan gate
  reads), right after the wind pre-pass.
* `npc_ai_fighter_scan::fighter_obstacle_scan(npc, world, rng)` — now
  takes the caller's `UpdateNPCs` stream; `flag25 =
  npc.target_zone_graveyard && rng.next_max(60) == 0`. Outside a
  graveyard the `&&` still short-circuits: **zero draws** (the legacy
  census holds for every non-graveyard tick). Inside one: exactly one
  `Next(60)` per 60-tick bash.
* `update_zombie` threads its dispatch `rng` into the scan; its inline
  discouraged fold gains the `&& !self.target_zone_graveyard` chain term
  (`:60302`), and `npc_ai::fighters_not_discouraged` reads the pre-stamp
  (retiring the `npc_ai.rs:280` false stand-in).

**Draw-position deviation (documented, extends S1-E-21e):** the C# rolls
the moans at the AI head (`:57340-57367`) BEFORE the scan's bash draw;
the port's moans ride the sim tail AFTER `update_rng`. In a graveyard
the bash `Next(60)` therefore precedes the moan `Next(1000)` — the same
moan-position remainder, extended by one conditional draw. The witness
replays that order explicitly.

Witnesses (written not run):
* `npc_ai_fighter_scan.rs` — `w4_g4_outside_a_graveyard_the_bash_draws_nothing`
  (stream-clone pristine), `w4_g4_in_a_graveyard_the_bash_spends_one_next_60`
  (mirror replay), `w4_g4_the_graveyard_roll_cancels_the_normal_night_reset`
  (seed-searched `Next(60)==0` → ai[1] 8+5→cap 10 open armed; non-zero
  control resets to 5).
* `npc_ai.rs` — `w4_g4_the_graveyard_stamp_cancels_the_day_discouragement`.
* `world_sim/npcs.rs` — `w4_g4_the_graveyard_bash_draws_on_the_update_npcs_stream`
  (end-to-end: pre-stamp → scan draw → drain; twin replays bash + moan;
  the outside-zone control carries only the moan roll).

## 3. Door-open census — town Next(10) vs fighter bash Next(60)

| # | Site | C# | Gate | Draw | Port |
| --- | --- | --- | --- | --- | --- |
| 1 | Town-NPC walk door-open | `NPC.cs:54043` | `townNPC && tileSafely5.nactive() && (type 10 \|\| 388) && (Next(10)==0 \| flag)` — the `\|` is NON-short-circuiting: the draw happens even when `flag` (the good-rest/home term) is true | one `Next(10)` per walk-tick at a door tile; on pass: OpenDoor(dir) → OpenDoor(−dir) → ShiftTallGate, first success latches `closeDoor` + `ai[1] += 80`, all-fail flips direction (`:54047-54078`) | `npc_ai_town.rs:2757` — FAITHFUL (`rng.next_max(10) == 0 \|\| c.flag`); witness `s1_d_3_town_the_door_open_attempts_and_the_next_10` (`:4219`) pins the draw + the 3-attempt order |
| 2 | Fighter door bash | `NPC.cs:59997` | `ai[2] >= 60` (the 60-tick bash counter) then `ZoneGraveyard && Next(60)==0` | one `Next(60)` per fired bash, ONLY in a graveyard; outside: zero (short-circuit) | `npc_ai_fighter_scan.rs` — P5 wired (was the false stand-in); witnesses §2 |
| 3 | `WorldGen.OpenDoor` itself | `WorldGen.cs:37630-37786` | — | NO rand draw; the only RNG consumer is `SoundEngine.PlaySound(8)`'s pitch (`:37727`) | `tr_core::edit::open_door` — draw-free; the drain plays the cue on `UpdateNPCs` (`world_sim/npcs.rs`) |
| 4 | `WorldGen.ShiftTallGate` | `WorldGen.cs:51723-51775` | — | NO rand draw; `PlaySound(8)` at `:51762` | `tr_core::edit::shift_tall_gate` (W4-g4 P4 `53063dd1`) — draw-free |
| 5 | The bash thud | `WorldGen.KillTile(fail: true)` `:60046` → `KillTile_PlaySounds` (`WorldGen.cs:63558`) | every fired bash | the dig cue's own rolls | the drain's `kill_tile_sound` + `kill_cue` (S1-D-3z) |

Reading: the two door mechanics are INDEPENDENT streams consumers — the
town walker opens doors on a 1/10 per-tick roll at the door tile (no
counter), the fighter bashes on a 60-tick counter and only rolls 1/60
inside a graveyard to bypass the normal-night reset. Neither open helper
draws; all door-related gameplay draws are the two gates above.

## 4. Confused walk-direction reversal — generic arm (P6 `1419f7d1`)

The C# flip is ONE site: `SetTargetTrackingValues`' tail
(`NPC.cs:77772-77775`, `if (this.confused) base.direction *= -1;`),
reached by every `TargetClosest()` call. `this.confused` latches from
`buffType[i] == 31` in the buff scan (`:92029-92032`); the port reads
`buffs.find_buff_index(31)` directly (the P3 convention).

Coverage before P6:
* `npc_target::target_closest` — the flip PORTED (`npc_target.rs:291`,
  witness `confused_flips_the_facing`). Every fighter-core caller
  (`update_zombie`, `walker_step_fighter`'s 10+ kinds) already reversed.
* The GENERIC archetypes `npc_ai::walker_step` / `walker_step_ladder`
  compute an ad-hoc dx-sign facing (no `TargetClosest`) — the flip was
  MISSING for their riders: `update_archer` (both the stand and walk
  calls), `update_martian_shooter`'s walk tail, `update_vortex_headcrab`,
  `update_dd2_skeleton`/`update_dd2_goblin`/`update_dd2_bomber`/
  `update_dd2_thrower`, `update_nebula_beast`,
  `update_stardust_spider_small`, `update_vortex_hunter`,
  `update_drakin`, `update_wither_beast`, the dead `update_goblin`
  archetype (no live caller since S1-D-3b).

P6 adds the flip in both archetypes' aggro arm, AFTER the dx-sign facing
and BEFORE the sprite lockstep + the velocity/ladder write — so the
ladder's accel (`run_ai3_ladder` reads `npc.facing`) walks the confused
NPC AWAY from the player, exactly the C# net effect. No double-flip:
none of the direct archetype callers also runs `target_closest`
(verified per-machine; the saucer core's `target_closest` is a different
machine, `update_saucer_core`, not a walker caller), and
`walker_step_fighter`'s non-style-3 fallback (the Present Mimic stand-in)
keeps the legacy step's single flip.

Witness (written not run): `npc_ai.rs`
`w4_g4_the_confused_generic_walkers_reverse_the_walk` — calm vs confused
twins on both archetypes (facing/sprite_dir/vx sign) + the one-flip-only
pin on `walker_step_fighter`.

## 5. Remainders (unchanged this phase)

* The full AI_003 core migration stays d22's (`60712154`, unpushed) —
  §1's merge-risk note applies.
* `flag26` (`player.insideUnbreakableWalls`, `:60002`) — still no Player
  field, false; the +6 bash bonus never fires (S1-D-3z stand-in).
* `Main.getGoodWorld` — unported, false; flag27 folds to `!bloodMoon`
  (the style-9 precedent).
* The moan-position deviation (§2) — the S1-E-21e remainder, now with
  one conditional draw ahead of it in graveyards.
* `Main.dontStarveWorld` in `fighters_not_discouraged` — still false
  (the `ambient_sound` twin reads the seed flag; the npc_ai fold does
  not — pre-existing, out of lane scope).