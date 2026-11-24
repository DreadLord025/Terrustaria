# 331 — W4-d3 SEAL: the boulder-family bible + the five receipts (FINAL)

> Renumbered **248 → 331** per registry-150 **§9 UNIFIED FINAL** (d54 tip
> `b243c7086`, §9.2 row 331): g82's published round-2 reservation wins the
> 248 key (§6.3 `26e-f-rope-union-final.md`, publish `72226e500` 14:47:50);
> d3's add `740026af6` 16:10:40 is POST-publish → loses → **331**. Chain:
> 248 → 331.
>
> Sealed 2026-11-21 against the W4-d3 tip `5ff9fe983` (branch
> `task/W4-d3-proj-ai-21-30`, code-only mode). Original number claim per
> registry-150 (`docs/research/150-research-number-registry.md`, d54
> `38bfd1d22`): the 151–236 block consumed, 237–247 taken at pushed tips
> (244/246 carry sibling collisions), 248 first free at this SEAL-tip
> scan (every `origin/task/W4-*` tip's `docs/research/` ls-tree walked) —
> superseded by §9's reservation verdict above, kept for provenance. The
> registry-assigned rename `26f → 170` (21-30 band status) is executed on
> this branch in the same wave; 26b (the census) keeps its key —
> uncontested at the registry scan.
>
> C# cites are `decompiled/Terraria/Projectile.cs` (stable) unless named;
> Rust cites are at the d3 tip `5ff9fe983` unless a sibling hash is given.
> Witnesses on this branch stay **WRITTEN NOT RUN** (no cargo / tests /
> rustfmt / graphify invoked — the code-only lane).

## 1. Receipt — the d3 landing wave (SEALED)

The aiStyle-25 boulder family is COMPLETE on this branch. The commit
chain (base `a725acb04` → tip `5ff9fe983`):

| commit | scope |
| --- | --- |
| `57abe80a4` | the 21-30 census (research/26b): 9 arms verify-only, style 25 MISSING |
| `f9d2759e8` | the 13 kinds' AI arm + `HandleMovement` bounce ladder + RainbowBoulder smart-bounce + Shimmer float + Kill helpers (`projectile_ai_boulder.rs`) |
| `241cbae09` | the `proj_data` rows (13 kinds + the 763 spike) + `proj_frames` 1014/1055 + the 1047 `ignore_water` |
| `40f8cbe3d` | the `25 =>` dispatch arm, the `boulder_kill` death hook, the contact lane, the seams |
| `affd7fa90` | the world-sim drains (drops/bees/switches) + the boulder statue (`world_sim/wiring.rs`, `StatueSpawn::Boulder`) |
| `154beb131` | ANALYSIS "The boulder family (W4-d3)" + PORT_MAP row + anchors |
| `2d01693ca` | the PreventSpam precompute gated to the 1047 (no 1000-slot scan per sub-step for the other kinds) |
| `3097a930e` | the g64/g16 dust+gore RNG seams replicated (`DustDraw`, `clone_dust`/`new_dust_direct`/`new_dust_perfect`, `GoreRng for dyn DropRng`) |
| `986ab27bd` | the dust/gore BODIES wave + the 763 spike latch/kill |
| `5ff9fe983` | the band-status doc (26f, now 170) + the bodies-wave docs |

**The bodies landed** (ANALYSIS "The boulder family (W4-d3)", the bullets
at the d3 tip): the 1053 ember trail (`:26031-26049`) + the 1048 flight
speck (`:26395-26399`), the two movement sites (`:18947-18956` the 1053
bounce 10×2 burst, `:19242-19246` the 1048 bounce 10-dust ring), the
Kill-arm bodies (the family dust loop `:74938`/`:74973`/`:75083` with the
per-kind num599 type, the 1053 lava 25+80 dust + 4-gore `:74877-74913`,
the 1054 20-dust `:74921`, the 1055 50-dust `:74928`, the 1047 rainbow
ring + 14 hue dust `:71192-71255`), and the **763 spike latch**
(`AI_001`'s `:55518-55529` 5-dust spawn burst, wired kind-gated in the
style-1 dispatch, `projectile.rs:17064` → `spike_spawn_dust`
`projectile_ai_boulder.rs:2084`) **+ kill** (`:75067-75078`,
`spike_kill_dust` `:2107`, the hook arm `projectile.rs:21527-21529`).
Every explicit `Main.rand` gate/settings draw stays in C# order; the
`NewDust`/`CloneDust`/`NewGore` internal rolls land at their C# positions
inside the verbatim `DustPool`/`GorePool` calls on the SAME
`UpdateProjectiles` stream. Wiring: `ProjectilePool::update` is the
scratch wrapper over `update_dust`; the world-sim pass threads
`app.dust` + `spawn_ctx` + the gore pool (`set/take_gore_pool`).

**Rainbow-gore NONEXISTENT (the directive's census, re-verified against
the C#):** the 1047 Kill arm (`Projectile.cs:71192-71255`) spawns
**zero** `Gore.NewGore` — the "rainbow shards" are dust 267/66 +
RainbowRodHit `ParticleOrchestrator` particles, NOT gore. The family's
ONLY gore is the 1053 lava death's four gibs (`:74898-74913`, type
`Next(61,64)`, `lava_death_gore` `projectile_ai_boulder.rs:1980`). The
particle BODIES stay skipped (the shadowbeam precedent — the port has no
particle orchestration); their settings draws stay. Anchored:
`anchors.txt` `lava_death_gore` row (the module's gore census marker).

**The 21-30 band status** lives in `170-projectile-ai-21-30-band-status.md`
(the registry rename of 26f): every style 21-30 has a live dispatch arm;
25 was the only MISSING one (ported here), the other nine are
verify-only; the remaining effect-layer gaps belong to the dust-site
co-lanes (d53/e3/f5, g63/g64, g65, g16/g17). The style-30 907/335 rows
and the style-28 967/1026 frame machines stay g64's unmerged trail lane
— deferred, not re-ported (170 §"The small missing pieces").

Witnesses: 25 `w4_d3_*` in `projectile_ai_boulder.rs` (the rows, the
ladders, the bounce/chase machines, the dust/gore bodies incl.
`w4_d3_the_spike_spawn_latch_body` :2907 / `w4_d3_the_spike_kill_dust_body`
:2940 / `w4_d3_the_lava_death_gore_body` :2869) — WRITTEN NOT RUN.

## 2. Receipt — the d75 sound layer (SIBLING-CUED)

The d75 lane (`origin/task/W4-d75-proj-sounds-1`, tip **`b59e3a874`**,
parts 1-5) owns `Projectile.cs`'s PlaySound sites #1-#120. Its **part 4**
(`20b2cea61` code / `e1bed75cf` docs) re-probed d3 at `2d01693ca` and
classified the boulder family **SIBLING-CUED**: d3 queues every cue
itself; d75 landed the sound layer those intents need. **Constants
copied, machines NOT re-ported** — the split is clean.

**d3's queued intents (this branch, verbatim sites):**

| producer (d3 tip) | intent `(id, style, x, y, pitch)` | C# site |
| --- | --- | --- |
| `bounce_ladder`'s dig closure (`projectile_ai_boulder.rs:1076`) | `(0,0,pos,0)` | `:18959/:18978/:18994/:19067/:19090/:19178/:19270` |
| the 1048 bounce arm (`:1241`) | `(2,177,pos,0)` | `:19241` |
| `rainbow_check_bounce` (`:584`) | `(42,274,pos,0)` | `:20104` |
| `boulder_kill` family cue (`projectile.rs:21422-21433`) | 1048 `(2,16)` / 1053 `(2,14)` / 1054 `(3,29)` / 1055 `(4,6)` / rest `(0,0)` | `:74857-74876` |
| the 1047 death (`projectile.rs:21409`) | `(42,274,center,0)` | `:71192` head |
| the 1005 death (`:21520`) | `(0,0,pos,0)` | `:74970` head |
| the 655 death (`:21538`) | `(4,1,pos,0)` — the **int** overload `PlaySound(4, x, y)` (`:75082`, no style arg; the port's stock-style-1 stand-in) | `:75082` |

The AI cues park on `wants_sound` (the sub-step forward); the death cues
push STRAIGHT into `pending_proj_sounds` (the hook runs after the
forward — the golf-strike precedent, `projectile.rs:20306-20318`).

**d75's landed layer (at `20b2cea61`, re-read at `b59e3a874`):**
`rainbow_boulder` joins `ITEM_CUSTOM_CUES` (`audio.rs:561`, the
`SoundID.RainbowBoulder` trackable, `Sounds/Custom/rainbow_boulder.xnb`
verified on disk); `NPC_Hit_29` joins `SFX_CUES`; the drain
(`world_sim/projectiles.rs`) gains the **id-3 route** (`:502-511`,
`id == 3 && style == 29` → `play_npc_hit_sound("NPC_Hit_29", …)` — the
style overload's audio-only GetRandomPitch + case 3's one pitch draw),
the **id-4 route** (`:512-518` → `play_npc_death_style`, `audio.rs:1396`,
whose style table **gains the 6 row**), the id-37 `(2,-34)` route
(`:519-534`, the 502/1106 lanes') and the id-42 custom route (part 2,
`audio.rs:609`'s custom-cue membership). The `(42,274)` rainbow bounce +
death and the `(3,29)`/`(4,6)` deaths are **audible after the d3+d75
merge**; on d3 alone they drop at the drain (the module doc's sound-skip
note, `projectile_ai_boulder.rs:59-61` — refreshed by this SEAL to cite
d75's routes).

**The part-5 compose-receipt (d75 `b59e3a874`, ANALYSIS "Part 5 (d75)"),
binding for the merger:**

1. **The id-4 STYLE-vs-INT overload collision** — d3's 1055 `(4,6)` is
   the **style** call (`PlaySound(SoundID.NPCDeath6, x, y)`,
   `SoundEngine.cs:88-99`, ONE audio-only GetRandomPitch NextFloat);
   d77's drain routes id 4 → `play_npc_death_int` (its 385/454 sites are
   `PlaySound(4, x, y, style)`). f1's doc labels d3's `(3,29)`/`(4,6)`
   "the int facade" — **a MISREAD** (d75 part 5's correction). Keep BOTH
   overload paths: d75's id-4 style route for the boulder `(4,6)`,
   d77's int route for its kill-tail cues; the payload alone cannot
   distinguish them — a sentinel (the −34 precedent) or the
   producer-side façade call is the seam. The 655's `(4,1)` is the int
   overload on BOTH sides (C# `:75082` has no style arg) — d77's int
   route serves it; no collision.
2. **The boulder kill-map DOUBLE-PLAY warning** — f1's final owed wave
   (`bcdfcd62e`) rows the 1054/1055/1047/1048/1053/99/727/1013/1014/
   1021/1057/1005/655 kinds on `kill_sound_cue`: the SAME sites d3's
   `boulder_kill` already queues. With d3 merged those arms would
   DOUBLE-PLAY (d3 parks the intent mid-Kill; the death hook fires
   again). **Keep-one: drop the boulder-kind arms from the surviving
   f1/d77c kill map** (or gate them on d3's arm absence) — d75 part 5
   records it for the composer.
3. **The 6-tuple fold** — d3/d75 push the 5-tuple
   `(id, style, x, y, pitch)`; d77/f1 carry the `volumeScale` widening.
   Every d3 push folds to `volume = 1.0` (all boulder cues are
   stock-volume; the C# sites pass no volume scale).
4. **The id-29 gate-truncation wrinkle** — d77's drain-side x/y
   truncation before `play_fighter_moan` is the faithful gate probe;
   keep it at the union (draw-identical otherwise).

The `d75e-flag` item of the directive resolves to the **1023/1081
STILL-DEFERRED flag** in d75's part-4/part-5 census — see §4 (no `d75e`
branch or commit exists at any tip; the flag lives in d75's docs).

## 3. Receipt — the g65 Kill-309 compose (1048/1053/763)

The g65 lane (`origin/task/W4-g65-proj-death-dust`, tip **`b0842907c`**)
ports `Kill()`'s per-kind death-DUST arms as a snapshot→drain lane:
the death hook snapshots the corpse (`DeathSnapshot::of`,
`projectile.rs:19990/:19999` at the g65 tip), the world-sim pass drains
at its tail (`world_sim/projectiles.rs:419-429` →
`projectile_death_dust::drain` `:520`) onto the live `UpdateProjectiles`
stream. **Its boulder-family arms OVERLAP d3's landed bodies** — the
compose is a keep-one, NOT a keep-both:

| kind | g65 arm (tip `b0842907c`) | d3 body (tip `5ff9fe983`) | verdict |
| --- | --- | --- | --- |
| 1048 (+99/1013/727/1014/1021) | the 99-family default `projectile_death_dust.rs:2929-2944` (batch 20 `8bb32268d`): 30× kind-switch dust, `Next(2)` scale×1.4; the ×1.9 corpse compound recorded as a DEAD mutation | `family_dust_loop` (`projectile_ai_boulder.rs:1950`, hook `projectile.rs:21476-21486`): the same 30×/type-switch/gate **plus the compound applied to the live slot** | **keep-one = d3** — g65's arm DROPS at the union (d3's is the fuller body: the compound feeds the C# `base.velocity *= 1.9` corpse write; g65's snapshot cannot mutate the pool slot). The draw ORDERS agree (gate after spawn), so either side is stream-identical; d3 wins on state fidelity |
| 1053 | `:3054-3074` (batch 21 `dd1322a19`): the centred 64-box, 25× dust-31 + 40 pairs dust-6, the 4-gore CENSUS draws only (`next_range(61,64)/(10,31)/(-20,21)` ×4); the 31-restore + `LavaBoulderExplosion` unported | `boulder_kill` `projectile.rs:21439-21468`: the recentre+64 resize, `lava_death_dust` (25+80), `lava_death_gore` (the four gibs BODIED through the gore pool), the 31×31 restore, `lava_boulder_explosion` (`:21510`, the PlaceLiquid flood) | **keep-one = d3** — g65's arm DROPS (d3 bodies the gore + the restore + the flood; g65 keeps only the census draws). Stream-identical either way (the gore type/kick rolls sit at the same positions) |
| 763 | `:3004-3013` (batch 21): 4× dust-40, `Next(2)` scale×0.8, noGravity | `spike_kill_dust` (`projectile_ai_boulder.rs:2107`, hook `projectile.rs:21527-21529`) + the SPAWN latch `spike_spawn_dust` (`:2084`, dispatch `projectile.rs:17064`) which g65 does not carry | **keep-one = d3** — g65's arm DROPS; the latch is d3-unique |

The same keep-one extends to the rest of the family's g65 arms: the
1054 (`:2946-2950`) / 1055 (`:2954-2962`) / 1005 (`:2966-2973`) bursts
and the **1047 rainbow trail-ring** (`1047 | 79 => rainbow_trail_ring`,
`:4023`, fn `:5045` — the 79 half stays g65's, the 1047 half folds into
d3's `rainbow_boulder_death` `projectile_ai_boulder.rs:1678`; the union
arm must keep the 79 dispatch and drop only the 1047 key). The 655 arm
g65 explicitly DEFERRED (batch 21: "the 655/91/92 state-bearing spawn
arms stay deferred") — d3's bee/hive body (`projectile.rs:21533-21556`)
is the only one; no conflict.

**Why d3 wins every row:** d3's bodies run INSIDE the death hook on the
live slot (the corpse-velocity compounds, the 1053 resize/restore, the
gore pool, the item/bee/switch parks all read or write pool state);
g65's snapshot lane cannot see post-snapshot mutations. Both sides take
the explicit draws in the same C# order on the same stream, so the
keep-one is **RNG-neutral** — the merger drops g65's boulder-kind match
arms (the `99 | 1013 | 727 | 1014 | 1021 | 1048` key row, `1054`, `1055`,
`1005`, `763`, `1053`, and the `1047` half of the `1047 | 79` key) and
keeps d3's hook. g65's witnesses for those arms
(`w4_g65_the_99_family_default` :6778, `w4_g65_the_1005_763_bursts`
:6812, `w4_g65_the_1053_lava_boulder` :6872) re-pin onto d3's bodies or
drop with the arms (d3's 25 witnesses cover the same pins).

## 4. Receipt — 1023/1081: d3 stopped short (the row-gap task-seed)

**d3's boulder family stopped short of the 1023 DeadCellsBarrel and the
1081 MudBallPlayer** — d75's part-4/part-5 census flag (the directive's
"d75e-flag"; ANALYSIS "Part 4/5 (d75)" + research/32's unrowed table at
`b59e3a874`: "row + movement (d3's boulder family stopped short of
it)"). The two kinds are NOT aiStyle-25 — they only share
`HandleMovement`'s bounce-Dig neighbourhood:

- **1023 DeadCellsBarrel** — `SetDefaults` `:9681-9692` (alpha 255,
  26×26, **aiStyle 1**, friendly, ranged, timeLeft 180,
  usesLocalNPCImmunity, localNPCHitCooldown −1). Its bounce arm
  `:18752-18782` (the ±5 clamped X/Y reversals + the `localAI[1] = 1`
  latch + `HitTiles` + the `:18780` Dig), the AI_001 age/alpha arm
  `:57162-57173`, the rotation clamp `:57620-57623`, the spawn-latch
  25-dust fan `:55209-55229`, the `GetCollisionParams` 14×14 box
  `:19882-19886`, the damage multipliers `:14557-14560` (×1.33 after
  the latch) / `:14992-14995` (+8 hitbox — **already ported**,
  `projectile.rs:22942-22944` at the d3 tip), and the Kill arm
  `:76866-76889` (the `DeadCellsBarrelLauncherExplode` cue, the 192×192
  resize, 8 gore + 40 dust + `Damage()`).
- **1081 MudBallPlayer** — `SetDefaults` `:10087-10094` (penetrate −1,
  12×12, **aiStyle 10**, friendly). Its blocked-move arm
  `:19829-19848`: a Y-block while falling steps + `Kill()`s; any other
  block zeroes a negative vy and the `localAI[0]` one-shot latch plays
  the `:19845` Dig.

**The SEAL-tip re-probe (this doc's own scan, NOT d75's stale one):**

- **1081 is NOW ROWED + CUED on d1** — `origin/task/W4-d1-proj-ai-1-10`
  advanced `e271b5bc8` → **`212516a82`** (wave 3c `e47181798`,
  2026-11-21T12:29:04+02:00 — AFTER d75's part-5 probe of `e271b5bc8`,
  which is an ancestor of the wave). `MUD_BALL_PLAYER` lands in
  `projectile/data.rs:1137/:3437` and `projectile_ai_falling.rs` ports
  the `:19829-19847` blocked arm **with the Dig queued**
  (`wants_sound = Some((0,0,pos,0))`, `:370`). **d75's "1081 unrowed on
  every tip" verdict is STALE at this SEAL** — the merger re-classifies
  the 1081 row SIBLING-CUED (d1 queues, d75's id-0 Dig route serves);
  no d3 work owed.
- **1023 remains UNROWED at every pushed tip** (the SEAL scan: zero
  `1023` hits in any tip's `projectile/data.rs`; d1's style-1 waves did
  not take it — it is a Dead Cells crossover kind, not a falling/arrow
  family member). Its Kill death-dust IS staged on g65 (batch 26
  `73506403b`, the `1023 =>` arm `projectile_death_dust.rs:3558-3574`:
  the 8-gore census + the 40× dust-31 radial burst; the gore BODIES are
  g16's wave K `c5c69562a` `:76881` velocity-overwrite row) and its
  orchestrator particles on d54 (batch 7 `965e9612c`, the
  DeadCellsBarrel 2-head-draw + spark loop + rotated ring). **The row +
  the AI/movement halves are the gap** — task-seeded **S1-P-i58**
  (ROADMAP + research/90 R-331-1; i57 is d53's wetVelocity seed, taken
  at its pushed tip; i58 free at every tip at this scan).

## 5. The boulder-family bible (the family map, C# → port → owner)

ONE family, ONE owner per layer — the merger's fold table:

| layer | C# anchor | port (owner) | compose note |
| --- | --- | --- | --- |
| SetDefaults rows (13 kinds + 763) | `:1833-1870`/`:7038-7047`/`:9918-9937`/`:7855-7865` | `projectile/data.rs` (**d3** `241cbae09`) | no sibling rows the same kinds |
| AI arm + wall-nudge ladder | `:26015-26415` | `projectile_ai_boulder.rs` (**d3** `f9d2759e8`) | the 1047 PreventSpam precompute gated (`2d01693ca`) |
| bounce ladder + slope pass | `:18941-19277` (+`:17656-17984`) | `bounce_ladder` (**d3**) | the 1053/1048 movement dust d3's; the OTHER kinds' trail dust g63/g64's |
| RainbowBoulder machine | `:20024-20315` | `rainbow_*` (**d3**) | the particle bodies d54's orch lane (batch 7) |
| Shimmer float | `:22504-22515` (`:22363` exemption) | `boulder_step` head (**d3**) | — |
| trail/flight dust | `:26031-26049`/`:26395-26399` | the AI-arm bodies (**d3** `986ab27bd`) | g64's trail lane carries OTHER kinds |
| Kill death cues | `:74857-74876`/`:71192`/`:75082` | `boulder_kill` (**d3**) | audible via **d75**'s drain routes (§2); f1/d77 kill-map arms DROP (double-play) |
| Kill dust/gore bodies | `:74841-74981`/`:75067-75078`/`:71192-71255` | the kill helpers (**d3** `986ab27bd`) | **g65**'s same-kind arms DROP (§3 keep-one) |
| 763 spawn latch | `:55518-55529` | `spike_spawn_dust` (**d3**) | d3-unique |
| 1053 lava flood / 1048 pyramid | `:70538-70552`/`:74958-74968` | `lava_boulder_explosion`/`poulder_spread` (**d3**) | PlaceLiquid/tile writes, draw-free |
| item drops / bee burst / switches | `:74950-74957`/`:75100-75121`/`:17087` etc. | the park→drain seams (**d3** `affd7fa90`) | the 1054 spider spawn DROPS (unported WallCreeper 164/165) |
| boulder statue | `Wiring.cs:1994-2013` | `world_sim/wiring.rs` (**d3** `affd7fa90`) | — |
| sounds (bank + drain) | `SoundID.cs` rows + `SoundEngine.cs:88-99` | `audio.rs`/`world_sim/projectiles.rs` (**d75** `20b2cea61`) | d3 queues, d75 serves; the id-4 overload split §2.1 |
| 1023 barrel (row+AI owed) | `:9681`/`:18752`/`:55209`/`:57162`/`:76866` | UNROWED — **S1-P-i58** | Kill dust g65 b26 / gore g16 wave K / particles d54 b7 already staged |
| 1081 mud ball | `:10087`/`:19829-19847` | **d1** wave 3c (`e47181798`) | ROWED+CUED at `212516a82`; d75's stale flag retired (§4) |

**Deviations stay as recorded** (ANALYSIS "The boulder family (W4-d3)"):
the wet-flag tick-head probe, the 1047 fixed-10 trail ring, the 1054
spider drop, the bee-hive `100 − npcs.len()` cap fold, the contact-lane
no-kill proxy + the 1021 hostile-only exemption, the park→drain item/
bee/switch seams, the one-tick-lag plate scan. The particle bodies,
`Collision.HitTiles`, `Lighting.AddLight` stay their layers.

## 6. Verification record (code-only)

- The d3 wave re-read at the tip `5ff9fe983` (the module, the hook, the
  dispatch latch site, the 25 witnesses' names).
- d75 `b59e3a874` read via `git show` (ANALYSIS Parts 4/5, research/32's
  census table, the drain routes at `world_sim/projectiles.rs:502-534`,
  the bank rows at `audio.rs:561`); the part-4 code commit `20b2cea61`
  stat-checked.
- g65 `b0842907c` read via `git show` (`projectile_death_dust.rs` the
  99-family/1054/1055/1005/763/1053/1047/1023 arms + the drain + the
  snapshot hook; batches 20/21/26 commit messages).
- d1 `212516a82` re-probed (`MUD_BALL_PLAYER` row + the falling arm's
  `:19836-19847` port with the `(0,0)` cue); `e271b5bc8` confirmed an
  ancestor of wave 3c (`git merge-base --is-ancestor`, exit 0).
- The 1023/1081 C# blocks re-read whole (`:9681-9692`, `:10087-10094`,
  `:18752-18782`, `:19829-19848`, `:55209-55229`, `:57162-57173`,
  `:57620-57623`, `:76866-76889`, `:14557-14560`, `:14992-14995`,
  `:19882-19886`); the 655/1054/1055 cue overloads re-read (`:75082`,
  `:74857-74876`).
- The registry scan: every `origin/task/W4-*` tip's `docs/research/`
  ls-tree walked; 248 free at all tips (237-247 taken; 244/246 sibling
  collisions noted, not touched) — that SEAL-time claim SUPERSEDED by
  registry-150 §9 (g82's reservation wins 248; this doc renumbered 331).
  S1-P-i58 free at all tips (i57 taken
  by d53's pushed seed).
- **No cargo / rustc / tests / rustfmt / graphify invoked.** The
  branch's witnesses stay WRITTEN NOT RUN; this SEAL adds no new
  witnesses (doc + comment-only wave).