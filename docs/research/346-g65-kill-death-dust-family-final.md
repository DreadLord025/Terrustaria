# 346 — g65 Kill death-dust-family bible FINAL (the 309→316-kinds census)

Status: FINAL / SEALED (W4-g65, 2026-11-21). Code-only — no
cargo/rustc/tests/rustfmt/graphify invoked; every witness WRITTEN NOT RUN.
C# cites are `decompiled/Terraria/Projectile.cs` (stable) unless named,
read in ≤100-line windows. Rust cites pin g65's tip `b0842907c`
(`crates/tr-app/src/projectile_death_dust.rs`, 7315 lines at that tip;
7333 with this seal's new witness) EXCEPT §2c (the g16 gore compose),
which pins the compose tip `37ec2f3aa` + its own commit (7549 lines:
+18 module-header seam lines, the 503 fix, +1 witness).

**Number chain: 245 → 308 → 346.** Registry-150 §8.7 AMENDMENT 3
(`origin/task/W4-d54` `9dddb32b4`, FROZEN TABLE): g65's post-§8
tips-only self-pick 308 is contested → **346 the §8.7 FINAL assignment**
(this file was formerly numbered 308; micro-rename,
content unchanged). Historical chain narrative below, preserved verbatim:
the prior session's uncommitted draft claimed
"245 first free" from a stale sweep; registry-150's LATEST refresh
(`origin/task/W4-d54-dust-sites-world` `0426245f2`,
`docs/research/150-research-number-registry.md` §7.4) declares 245 inside
the winner-consumed block 237–245 (g48's pre-publish 14:30:16 claim; g48
later self-renamed it 245→282 at `9c776ca2f`, but the registry's block
stands) and assigns 304–307 to d50/f4/g86/g43 — **308+ FREE, no
max+1/first-free-at-my-last-fetch picks**. This seal's fresh `ls-tree`
sweep of **all 363 origin refs** finds NOTHING numbered ≥308 (highest
filed: 298 g63, 293 d38) → **308 the first free key**. No g65 assignment
exists in §7.4; this is a fresh-sweep pick, not a rename-instruction.
26i (the keep-one receipt, `f6e07bb90`) is key-unique at every tip and
stays 26i; this FINAL is its companion.

## 1. Seal item 1 — the remainder census LANDED and CLOSED

`Kill-remainder-309-distinct-kinds-ALL-406-Dust.NewDust-sites-ported-or-accounted`
is landed: **22 commits** `7f0dc8649..b0842907c` = batches 11–31
(`b4ae9a4a4`…`ac33dd6d7`) + the final accounting (`b0842907c`, ANALYSIS
+77). `projectile_death_dust.rs` **+6200/−656** in that range (6856 diff
lines); the whole 22-commit range is +6914/−685 across 5 files.

**The number correction (309 → 316).** The owed-pool label's "309 distinct
kinds" was the drain census at **batch 28** (`4dc2df3d9`: 234 arms → 309
distinct kinds). Batches 29–31 added 711/636/686/502 + 932 + 681/1120 →
the FINAL tip carries **241 match arms → 316 distinct kinds** (mechanical
scan: every `N [ | N… | A..=B] =>` arm of `death_bursts`, ranges expanded;
zero duplicate kinds; the four range arms are `121..=126 | 597` `:3232`,
`411..=414` `:3423`, `424..=426` `:2730`, `625..=628` `:1020`). The
accounting's "309" (b0842907c) is the stale batch-28 figure; 26i's "317"
(f6e07bb90) is a +1 miscount. **316 is the FINAL number** — re-derived at
both `b0842907c` and the seal tip (code unchanged between them; re-scanned
mechanically again at this publish: 241 arms / 316 distinct).

**The 406-site closure re-verified independently at this seal** (not just
26i's re-scan): every `Dust.NewDust*`/`CloneDust` line in
`Kill()` (`:70642-77262`) counted by direct scan = **406** (399 `NewDust*`
+ 7 `CloneDust` at `:71063/:71141/:71184/:71235/:71307/:71373/:71453`).
Each site attributed to its nearest preceding owning-kind header (329
`this.type` if/case headers): **393 sites** whose owning kind has a drain
arm, **6 other-lane-claimed** (442 `:73534` → i16
`dart_electro_firework_kill`; 608 `:72185-72212` (5) → d12
`solar_counter_kill`), **7 documented-deferred** (§4 below).
**Unclaimed: 0.** The 7 CloneDust owners: 1116 `:71063`, 932 `:71141`,
931|1039 `:71184`, 1047 `:71235`, 79 `:71307`, 16 `:71373`, 34 `:71453`
— all ride g65 drain arms (`:3907`/`:4346`/`:3980`/`:4023`/`:4035`/
`:4112`). POST-MERGE the 3 gun sites (`:74029/:74063/:74099`) move to
other-lane-claimed (d6, §2b): **390 + 9 + 7 = 406**, still 0 unclaimed.

## 2. Seal item 2 — d78c blasts keep-one, re-verified at CURRENT tips

26i §2/§3 pinned the adjudication at d78 `fd59c51a` / d77 `5f10560b` /
g3 `8568ef37` / g1 `e43c62879`. **Two tips moved since; re-read:**

- **d78 tip is now `7ea64ff75`** (`task/W4-d78-ui-sounds-menus`): the SEAL
  commit adds 26f §6 addenda ONLY — `projectile.rs`/`projectile_kill_hooks_a.rs`
  diff vs fd59c51a is EMPTY. `hostile_death_blast` (`projectile.rs:21612`)
  / `damage_evp_boxed` (`:21705`) carry ZERO dust draws ("dust/gore
  SKIPPED" at every resize, `:21602` doc). **g65's 452/454/448 bodies
  (`death_dust.rs:2378/:2401/:2663`) KEEP** — disjoint halves, no dup.
- **g1 tip is now `ef671cf2b`** (`task/W4-g1-proj-ai-missing-a`): g1
  EXECUTED the d78 fold itself — `phantasmal_eye_kill`/
  `phantasmal_sphere_kill`/`saucer_missile_kill` + `hostile_burst_colliding`
  + `death_burst_player_hit` + the three dispatcher arms + 7 witnesses
  DROPPED (drop-marker comments cite 26f at `projectile_kill_hooks_a.rs:107`
  / `projectile.rs:20767`). **The 26i "g1 drops 3 hostile hooks" hunks are
  CONSUMED — nothing left for the merger there.** g1 KEEPS its unique
  459|709/521/522/80 state hosts; its census calls now face g65's pushed
  bodies (the drop-hunks below).
- **d77 `5f10560b` unchanged**: the four pinned sites re-read line-exact —
  `betsy_arrow_death_draws` call `projectile.rs:20552`,
  `nebula_arcanum_death_draws` call `:20612`, the death-hook 618/620 census
  block `:20054-20065`, the swept-618 INLINE census `:20630` (the sweep's
  `pool.kill` `:15495` bare-deactivates — never reaches the snapshot hook).
- **g3 `8568ef37` unchanged**: the six census calls re-read line-exact —
  `projectile.rs:21836-21838` (16/34/79) + `:21894-21897` (634/635, 659,
  661); the fn bodies `projectile_ai_misc.rs:484/528/568/606/634/661`.

### 2a. Merger drop-hunks — the census table (execute in this order)

d77 (`5f10560b`), per 26i §3 (unchanged, re-verified):

1. `projectile.rs:20552` — `betsy_arrow_death_draws(rng)` → DROP (g65 710
   body `:1868` drains it). Keep the host.
2. `projectile.rs:20612` — `nebula_arcanum_death_draws(rng)` → DROP (g65
   617 body `:2121`). Keep the host + owner-gated tail.
3. `projectile.rs:20054-20065` — the death-hook 618/620 census block →
   DROP whole (g65 `:2160` folds the ai0=86 latch; the snapshot is
   pre-write).
4. `projectile.rs:20630` — the INLINE swept-618 census → **KEEP** until
   the sweep is rewired through the death-walk path (26i §3 hunk 4).
5. `projectile_ai_misc.rs:469/495` — `betsy_arrow_death_draws` /
   `nebula_arcanum_death_draws` retire with their calls;
   `nebula_shard_kill_draws` (`:532`) stays live while hunk 4 keeps its
   call.

g3 (`8568ef37`), per 26i §3 hunks 6–8 (unchanged, re-verified):

6. `projectile.rs:21836-21838` — the magic-missile/flamelash/rainbow-rod
   census calls (16/34/79) → DROP (g65 `:4035`/`:4112`/`:4023`).
7. `projectile.rs:21894-21897` — nebula-blaze/spirit-flame/onyx-bolt
   census calls (634/635, 659, 661) → DROP (g65 `:1986`/`:1593`/`:2069`).
8. `projectile_ai_misc.rs:484/528/568/606/634/661` — the six fn bodies
   retire with their calls. g3's STATE hosts KEEP.

**g1 (`ef671cf2b`) — NEW drop-hunks (the 26i table's g1 row was
"standalone census DROPS per 26f §1"; now pinned per-kind at the new tip):**

9. `projectile.rs:20047` — the `fairy_queen_death_draws(rng)` call (the
   932 gate at `:20046`) → DROP whole gate+call: g65's 932 body (`:4346`,
   batch 30) already deduped this census (g65's own `:20029` comment
   records the removal on its branch); keeping g1's call double-rolls
   the head NextFloat + the per-step counts + the 3-float chains.
10. `projectile.rs:20706` — `crystal_pulse2_kill_draws(rng)` (522) →
    DROP: g65's 522 arm (`:769`, `crystal_spark_burst` 10× dust-254)
    carries the same Next(3) gate + branch draws.
11. `projectile_kill_hooks_a.rs:183-191` — the 20-dust census loop INSIDE
    `crystal_pulse_kill` (521) → DROP the loop only: g65's 521 arm
    (`:2568`, `crystal_spark_burst` 20×) carries it. The shard-split
    velocity draws (`:196+`) are STATE-BEARING and STAY (the 26f rule).
12. `projectile.rs:20747` — `charged_blaster_orb_kill_draws(burst, rng)`
    (459|709) → DROP: g65's `459 | 709` arm (`:2578`) carries the 7/30/2
    vs 3/10/0 rings + gores. The resize host (`:20741`) + the
    `damage_pve_boxed` burst KEEP (state).
13. `projectile.rs:20752` — `ice_block_kill_draws` (80) → DROP (it rolls
    NOTHING; g65's 80 arm `:3668` carries the 10× dust-67). The tile-127
    break host KEEP.
14. `projectile_ai_misc.rs:443` (`fairy_queen_death_draws`) +
    `projectile_kill_hooks_a.rs:157/134/274` (`crystal_pulse2_kill_draws`
    / `charged_blaster_orb_kill_draws` / `ice_block_kill_draws`) retire
    with their calls; `crystal_pulse_kill` loses only hunk 11's loop.

d78 (`7ea64ff75`): NO g65-facing drops (zero dust draws, re-verified).

**Keep-one invariant (26i §2, re-verified):** the cue/state lanes draw
ZERO dust; the g65 bodies play ZERO cues. 79/16/34 + 634/635/661/659 keep
d77 `kill_tail_cue` alongside the g65 rings; 452/454/448 keep d78
state+EVP alongside the g65 dust/gore halves. The only double-roll risk
is the census stubs — every one is now pinned to a drop-hunk above.

### 2b. The gun Kill smoke-dup — d6 SURVIVES, g65 DROPS (g63-298 §3.3)

The g63 FINAL (`298-proj-trail-dust-family-final.md` §3.3, g63 tip
`10775b366`; corroborated by d6's RETIRE `174c0d541`, its research/247)
adjudicated the 358/406/1042 smoke puffs — DOUBLE-PORTED by g65 batch 18
(`1bbeec5b5`, 10:22:59) and d6 residual 8 (`1b54dcbbd`, 09:38:32,
`projectile_gun_splash.rs`); both port `Projectile.cs:74029-74147` with
the same per-dust draw census. **Verdict: d6-SURVIVES-g65-DROPS** —
first-claim AND the strictly faithful port: the TRUE `old_velocity` latch
+ the style-60 scale-death ×5 base (`:28999-29000` kicks velocity BEFORE
Kill; d6 `projectile.rs:20545` selects `p.vel` on `scale <= 0`) + the
corpse `p.vel = base * 0.2` write (`:20546`). g65's `DeathSnapshot::of`
sets `old_vel: p.vel` — a documented STAND-IN (`death_dust.rs:437-442`),
wrong for moving guns — and `smoke_puff` never writes the corpse. g65's
"kind list skips the guns" claim note is stale; 26i §5 now carries the
verdict. Merger DROP-hunks on g65's OWN file (ranges re-verified at
`f6e07bb90`, code unchanged below `:7480`):

15. **D1** `projectile_death_dust.rs:2786-2794` — the 358/406/1042 match
    arms DROP (the guns fall through to the default, no draws); replace
    with the 298 §3.3 ownership comment citing d6 + research/298 §3.3.
16. **D2** `:4834-4885` — the `smoke_puff` helper DROPs whole (doc
    comment included; its only callers were D1's arms).
17. **D3** `:6616-6634` — the `w4_g65_the_smoke_puffs` witness DROPs
    (d6's three gun-splash witnesses cover the surviving census).

d6 keeps EVERYTHING (`projectile_gun_splash.rs` module + 3 witnesses, the
`pending_gun_splashes` queue `projectile.rs:14752-14757`,
`drain_gun_splashes` `:15289-15294`, the death-hook snapshot + velocity
write `:20525-20556`, the pass-tail drain `world_sim/projectiles.rs:680-696`,
`main.rs:187`); g65's generic snapshot hook keeps snapshotting the gun
kinds — inert, no arm consumes them post-drop. One survivor, never both:
a double-feed desyncs the stream. Post-drop drain: 316−3 = **313 kinds**
(§3 roster minus 358/406/1042); the 406-site closure holds as 390+9+7 (§1).

**Total merger drop-hunks: 17 = 14 census (§2a) + 3 gun smoke-puff (§2b).**
The g16 gore compose adds 12 more (§2c) — **29 total**.

### 2c. Seal item 3 — the g16 gore compose: disjoint POOLS, shared STREAM (g16 tip `72e4cae20`)

g16 wired its 165 gore sites into real `GorePool` spawns across five batches
(`1deeb8ca`/`36e03444`/`69eaf77f`/`a05959ec`/`43a99ff1`, ANALYSIS "The
Projectile gore sites" WIRING record; tip `72e4cae20` = the 955 witness fix).
**Kill-side, once per death**: `gore_death_arms` (`projectile.rs:21482`,
dispatched `:20258`) plus the gore rows inside `special_arrow_kill`
(`:21130` the 41 pair, `:21139` the 91/92 triple), `special_bullet_kill`
(`:21360` the 284 confetti, `:21370` the 286 gore) and
`thrown_consumable_death` (`:21411` the 183 bee four) — ALL in the same
`!active` once-per-death block that g65's snapshot hook rides
(`:20319`/`:20326`/`:20341` dispatches; a dead slot never returns to active).

**The pools are disjoint; the stream is not.** g16 writes `GorePool`, g65
writes `DustPool`; neither reads the other, and no kind's dust loop is
duplicated (g16's comments pin "the dust loops … are the dust lane's" at
every arm). But BOTH consume `Main.rand` = `RngKey::UpdateProjectiles`
(g65 `world_sim/projectiles.rs:425`, g16 the pass `rng` threaded from
`:236`). g65's `gore_census` + inline gore loops are DRAWS-ONLY stand-ins
("the bodies stay the gore lane's — the beenade precedent",
`death_dust.rs:4774`); with g16's bodies live they DOUBLE-FEED the stream.

**Draw-equivalence verified per row** (C# re-read + fn-by-fn comparison):
every overlapping gore type (61-63, 16/17, 276-282) is in `SAFE_GORE`
(`gore.rs:144-164`), `special_ai = 0` (`gore_id.rs:26-37`) and non-drip
(`:94-99`) — so `GorePool::new_gore`'s only draws are the
`Next(10,31)`+`Next(-20,21)` kick pair (`gore.rs:415-416`), exactly what the
census replicates; the ChildSafety re-type (`:424-428`) and the SpecialAI
rolls (`:499-546`) never fire for these types. The type/position draws ride
BEFORE `new_gore` in both ports. Confirmed pairs: `celeb2_gore`
(`gore_c.rs:173`) = `gore_census(1,false,false)`; `fire_explosion_gores`
(`gore_b.rs:279`) = `gore_census(2,true,false)`; `dd2_explosion_gores`
(`gore_f.rs:100`) = `gore_census(ngore,true,true)` (the Next(2) lift gate
included); `kill_row_gores` ×N (`gore_g.rs:125`) = `gore_census(N,false,false)`;
`corner_gores` (`gore_j.rs:120`) / `goblin_bomb_gores` (`gore_f.rs:131`) =
4× `Next(61,64)`+kicks; `kill41_gores` (`gore_k.rs:117`) / `kill286_gore`
(`:83`) = the type+kicks+two `Next(-10,11)` nudges; `confetti_gores_vel`
(`gore_h.rs:157`) = the 10-draw confetti ladder; `star_kill_gores`
(`gore_i.rs:131`) = NextFloat + Next(8) + kicks ×7; `kill9_503_gores`
(`gore_j.rs:65`) = `Next(16,18)` (rolled then OVERRIDDEN for the 503 — the
draw is unconditional) + kicks.

**Merger drop-hunks 18–29 (execute at the g65×g16 merge; g16's bodies KEEP
— they carry the bodies AND the draws). Ranges at g65 tip `37ec2f3aa` +
this compose commit (`projectile_death_dust.rs`, 7549 lines):**

18. `:287` — `gore_census(rng, ngore, true, true)` in `dd2_explosion_dust`
    → **DROP for 664/666/668 ONLY**: g16's `FLAME_BURST_SHOT_1|2|3` arm
    (`:21528`) spawns the loop. The shared `664|666|668|706` arm
    (`death_dust.rs:1962`) keeps the census for the **706** (g16 has no 706
    arm — unrowed): gate the call on `s.kind == 706`.
19. `:2074` — `gore_census(rng, 1, false, false)` (645) → DROP whole
    (g16 `kill645_gore`, `:21567`; no resize/Damage in either port).
20. `:2170` — `gore_census(rng, 2, true, false)` (617) → DROP whole
    (g16 `kill608_617_gores`, `:21584`). The 608 half is d12's, not g65's.
21. `:2412` / `:2438` — `gore_census` ×1 (452) / ×3 (454) → DROP both
    (g16 `kill_row_gores`, `:21611`/`:21625`). The d78 EVP halves stay
    (§2a) — three-way disjoint: d78 state, g16 gore, g65 dust.
22. `:2617` — `gore_census(rng, ngore, false, false)` (459|709) → **DROP
    for 459 ONLY** (g16 `CHARGED_BLASTER_ORB`, `:21642`, scale≥1 count 2):
    gate on `s.kind == 709` (the 709 is unrowed in g16 — its `ngore` rides
    the same scale gate, so the census stays correct for it).
23. `:2697` — `gore_census(rng, 1, false, false)` (448) → DROP whole
    (g16 `kill448_616_424_gores`, `:21655`).
24. `:3386-3390` — the 240 inline 4-corner loop → DROP (g16
    `kill240_corner_gores`, `:21701`).
25. `:3716-3720` — the 9|503 inline gore loop → **DROP for 9 ONLY** (g16
    `STARFURY`, `:21690`, ×3): gate on `s.kind == 503` (the 503 is unrowed
    in g16; its 2 rows keep the census — INCLUDING the unconditional
    `Next(16,18)`, the fix below).
26. `:3760-3765` — the 12 inline 7-gore loop → **KEEP** (g16's 12 arm is
    the 955-only `STAR_CANNON_STAR`; `kill12_low_damage_gores` is NOT
    wired). `:3770-3774` — the damage<100 3-gore loop → **KEEP** likewise.
27. `:3821-3832` — the 284 inline 5-confetti loop → DROP (g16
    `kill284_confetti`, `:21360`).
28. `:3851-3855` — the 286 inline single-gore row → DROP (g16
    `kill286_gore`, `:21370`).
29. `:3875-3881` — the 41 inline 2-gore loop → DROP (g16 `kill41_gores`,
    `:21132`); `:3914-3919` — the 955 inline 7-gore loop → DROP (g16
    `kill12_star_gores`, `:21682`, screen-gated); `:4427-4431` — the 681
    inline 4-corner loop → DROP (g16 `goblin_bomb_gores`, `:21554`);
    `:4986-4997` — `confetti_burst`'s gore ladder → **DROP for the 178
    caller ONLY** (`:2913`, g16 `confetti_gores` ×40, `:21668`): pass
    `gores = 0` at `:2913`, keep `15` at the 289 caller (`:2915` — the 289
    is unrowed in g16, `kill289_confetti` NOT wired).

**KEEP (no g16 body — the census is the only stream consumer):** the 590
(`:2298`), 587 (`:2331`), 501 (`:2380`), 483 (`:2563`), 467/468
(`:2476`/`:2510` — `kill467_468_gores` NOT wired), 616 (`:2719`), 424-426
(`:2765`), 724/725/726/907/1133 (`:3061`), 1053 (`:3088`), 1020/1105
(`:3135`), 1085 (`:3154`), 281 (`:3316`/`:3334`), 162 (`:3365`), 712
(`:3528`), 370/371 (`:3564`), 1023 (`:3582`), 312 (`:3614`), 1078
(`:3676`), 246/249 (`:3961`/`:3984`), the 12's two loops (hunk 26), the
289 confetti (hunk 29), the 706/709/503 halves (hunks 18/22/25). g16's own
deferral list (ANALYSIS "Cited deferrals (unrowed hosts)") matches this
KEEP set kind-for-kind — the two lanes' deferral boundaries agree.

**The bomb family: NO drop, NO double-feed.** `explode_bomb`
(`projectile.rs:22496`) threads the pass `rng` + gore pool and adds ONLY
the `Kill_Bombs_DoUsualKillCode` gore rows (`:22570` bomb-corner ×4,
`:22582` dynamite boulder ×8) on the PASS stream AFTER the frozen zero-rng
`FastRandom::new(0)` `Damage()` walk (`:22544`, the M4-PROJ-9 deviation —
KEPT untouched, the gore draws do not feed it) and BEFORE
`Kill_ExplodeTiles`. None of the bomb kinds (28/29/30/37/397/516/517/519)
has a g65 drain arm (verified: zero matches in the 316-kind roster), and
the C# `Kill_Bombs` dust (`:77648-78500`, 51 sites) is OUTSIDE the
`Kill()`-body census window (`:70642-77262`) — it is the bomb lane's, not
g65's. Disjoint by construction.

**The 503 stream fix (this compose commit).** The `9 | 503` arm gated the
gore type draw on `s.kind != 503` ("503 → fixed 16, no draw") — WRONG: the
C# `:75729-75733` rolls `num726 = Main.rand.Next(16, 18)` UNCONDITIONALLY
and only overrides the RESULT to 16 for the 503 (g16's `kill9_503_gores`
`gore_j.rs:75-78` ports the roll-then-override correctly). The 503's two
gore rows were landing 2 draws behind the C# stream. Fixed: the draw is
now unconditional (`death_dust.rs:3717`), with the new witness
`w4_g65_the_503_gore_census_draws_the_type_it_overrides` (`:7279`) pinning
the 40-spawn + 2×(type+kicks) stream against a hand-built twin. The 9's
three rows were already correct. **117 witnesses** now (116 + this one).

## 3. The drain census — 316 kinds, the FINAL roster

241 arms, 316 distinct kinds (mechanical scan at `b0842907c`, ranges
expanded, zero duplicates). The roster in ascending order:

1, 2, 3, 4, 5, 9, 12, 15, 16, 17, 21, 24, 27, 31, 34, 38, 39, 40, 41,
42, 44, 45, 48, 51, 54, 55, 69, 70, 71, 76, 77, 78, 79, 80, 81, 82, 93,
95, 96, 98, 99, 103, 109, 114, 115, 116, 117, 118, 119, 120, 121, 122,
123, 124, 125, 126, 128, 132, 156, 157, 158, 159, 160, 161, 162, 166,
167, 168, 169, 170, 171, 172, 173, 177, 178, 179, 181, 184, 189, 195,
206, 225, 227, 229, 239, 240, 245, 246, 249, 253, 261, 263, 264, 265,
267, 270, 275, 276, 278, 281, 282, 283, 284, 286, 289, 304, 306, 308,
309, 311, 312, 316, 318, 321, 323, 330, 335, 336, 337, 343, 344, 345,
346, 349, 355, 358, 359, 370, 371, 377, 378, 379, 384, 385, 386, 399,
405, 406, 408, 410, 411, 412, 413, 414, 415, 416, 417, 418, 424, 425,
426, 435, 436, 440, 444, 448, 449, 452, 454, 459, 462, 467, 468, 471,
472, 474, 475, 483, 484, 485, 495, 497, 501, 502, 503, 505, 506, 507,
508, 514, 520, 521, 522, 523, 532, 539, 566, 572, 581, 583, 584, 585,
587, 589, 590, 596, 597, 598, 599, 601, 606, 614, 616, 617, 618, 619,
620, 621, 625, 626, 627, 628, 631, 634, 635, 636, 638, 639, 640, 645,
658, 659, 660, 661, 662, 664, 666, 668, 669, 671, 675, 676, 680, 681,
682, 684, 685, 686, 706, 709, 710, 711, 712, 719, 723, 724, 725, 726,
727, 728, 729, 731, 732, 756, 763, 772, 811, 814, 818, 819, 836, 837,
856, 873, 907, 909, 920, 921, 926, 928, 931, 932, 936, 937, 955, 961,
962, 967, 971, 980, 985, 1005, 1012, 1013, 1014, 1020, 1021, 1023,
1024, 1026, 1036, 1037, 1038, 1039, 1042, 1047, 1048, 1049, 1053, 1054,
1055, 1073, 1078, 1080, 1085, 1092, 1097, 1098, 1105, 1111, 1115, 1116,
1120, 1123, 1124, 1133.

**POST-MERGE roster: 313** — 358/406/1042 drop to d6 (§2b hunks 15–17).

Batch provenance: batches 1–10 (pre-`7f0dc8649`, 135 distinct kinds at
the base) + batches 11–31 (the ANALYSIS "Census completeness" per-batch
lists). The witness suite: **116 `w4_g65_*` fns** at the seal tip (115 +
this seal's inert-deferred witness) — all WRITTEN NOT RUN; post-merge 115
(the smoke-puff witness drops, §2b hunk 17).

## 4. Seal item 4 — the deferred set, task-seeded to owner lanes

(The Lead's seal numbering: item 3 = the g16 gore compose, §2c; item 4 =
this deferred set; item 5 = this FINAL + the §5 family bible.)

The 7 deferred dust sites are NOT free; each needs state the drain's
`World`/`SpawnCtx` does not carry. C# re-read whole at this seal
(≤100-line windows); the port-side counterpart cited per row.

| kind(s) | C# arm | dust | the blocking state | owner lane / seed |
| ------- | ------ | ---- | ------------------ | ----------------- |
| 1084 TorchGodHelper | `:70956-70975` | 40× `TorchID.Dust[(int)ai[0]]` alpha 100, Next(3)≠0 → noGravity, vel×2.5; type-66 → `Main.DiscoR/G/B` colour + noGravity | the 24-entry `TorchID.Dust` table (`TorchID.cs:179-184`, UNPORTED — only `TorchColor` is ported) + the App-frame disco cycle (`App.disco`/`advance_disco` exists, `app/mod.rs:840/1499`, but is not in `SpawnCtx`) | **S1-P-i58** (R-346-1): the torch-table + disco feed into the drain seam, then the 1084/949 arms |
| 949 TorchGod | `:70976-70995` | 20× `(int)ai[0]`-typed dust alpha 100, Next(3)≠0 → noGravity, vel×0.8; type-66 → disco colour + noGravity | same disco feed (no table — the type is ai[0] direct) | **S1-P-i58** (rides the same feed) |
| 868/869/996 Tree/World/MoonGlobe | `:73705-73728` | 15× dust-13 vel(0,−2) scale 1.5 (no explicit draws) | the AFTER-dust `WorldGen.TreeTops.RandomizeTreeStyleBasedOnWorldPosition(Main.rand, Center)` / `RandomizeBackgroundBasedOnPlayer(Main.rand, player[owner])` / `RandomizeMoonState(Main.rand, true)` (`:73717/:73720/:73723`) — UNPORTED state-bearing randomizers that consume the SAME `Main.rand` + `SendData(7)` | **S1-P-i59** (R-346-2): the three globe Kill arms ride the worldgen-randomizer lane (g70/g71/g72-adjacent); the 15-dust prefix ports with them (stream order: dust internals BEFORE the randomizer draws) |
| 655 BeeHive | `:75080-75124` | 30× dust-147, Next(2)==0 → scale×1.4, `velocity *= 1.9` PER ITERATION (the corpse velocity compounds ×1.9³⁰) | the wet-item `Item.NewItem(1124, Next(2,6))` / the 2+3×Next(3) NPC-count + `NPC.NewNPC(Next(210,212), Start:1)` burst with per-NPC velocity rolls — cross-seam spawn draws; the compounding velocity write mutates the corpse mid-arm | **S1-P-i60** (R-346-3): the hive death arm rides the NPC-spawn/item-drop seam owner; the dust prefix + the ×1.9 ladder port together (the velocity write is observable by later hooks) |
| 91/92 HolyArrow/HallowStar | `:75212-75249` | 10× dust-58 speed vel×0.1 alpha 150 scale 1.2 + 3 gores Next(16,18) | the star spawn's state-bearing draws (`Next(-400,400)`/`Next(600,900)` position, the 22/√ normalize, damage/3 for 91) ride `special_arrow_kill` inline (`projectile.rs:20849`, `special_arrows.rs:480/503` — main-landed) | the i17 special-arrow lane (main): the dust+gore prefix folds into `holy_arrow_kill`/`falling_star_kill` ahead of the spawn draws — **cross-seam dedupe note to i17, no new seed** (the arm is already owned) |
| 89 CrystalBullet | `:75250-75268` | 5× dust-68 noGravity vel×1.5 scale×0.9 | the 2-shard split's 8 state-bearing draws ride `special_bullet_kill` inline (`projectile.rs:20974`, the `:20986` "5 dust-68 (skipped)" comment) | the i18 special-bullet lane (main): the 5-dust prefix folds ahead of the shard draws — **cross-seam dedupe note to i18, no new seed** |
| 183 Beenade | `:76431-76464` | 20× dust-31 alpha 100 scale 1.5 vel×1 | the 4 gore rows + the bee burst's draws ride `beenade_death` (`projectile_ai_thrown.rs:461`, main-landed — the G3 census: 4×(Next(61,64)+kicks) + Next(15,25) + per-bee rolls) | the thrown lane (main): the 20-dust prefix folds ahead of the gore census — **cross-seam dedupe note, no new seed** |

The three cross-seam rows (89/91+92/183) need NO new lane — the owning
hooks are main-landed; the fold is a prefix-insert at the cited fn, drawn
BEFORE the existing state-bearing draws (the C# body order). The three
seeded rows (1084+949 / 868+869+996 / 655) are registered as S1-P-i58/59/60
(ROADMAP) + R-346-1/2/3 (research/90 §(d)) — id space verified free at
every tip.

## 5. The death-dust-family declaration (the bible)

ONE family, axis-disjoint, per the d54-bible convention:

- **g63/g64** — the per-tick TRAIL dust (g63 ≤400 / g64 ≥401, research/298
  FINAL + research/196) + e3's aiStyle-generic trail copies. NOT death dust.
- **g65 (this lane)** — the `Kill()` per-kind DEATH dust: the 316-kind
  drain (313 post-merge), the snapshot→pass-tail seam, the 393/406 sites
  (390 post-merge). CANONICAL for every kind in §3's roster MINUS the
  §2b gun trio.
- **d6** — the gun Kill smoke puffs (358/406/1042, `projectile_gun_splash.rs`):
  the keep-one survivor per g63-298 §3.3 (§2b); g65's copies DROP at merge.
- **d53→e3→f5** — the projectile per-tick/AI dust site batches
  (research/275 RETIRE, re-issued per registry-150 §7.4.3; chain
  240→244→275). NOT death dust.
- **i16/i17/i18/celeb2/floating-turret/d12** — the special-death hosts
  (442/478, the special arrows/bullets, 715-718, 644/641/643, 608): their
  Kill dust stays theirs (the 6 other-lane sites + the 3 cross-seam
  prefixes of §4).
- **g16** — the gore BODIES/draws (165 sites), WIRED at `72e4cae20`
  (`gore_death_arms` + the special-arrow/bullet/thrown gore rows, Kill-side
  once-per-death on the same `!active` seam). Disjoint POOLS (GorePool vs
  DustPool) but the SHARED `UpdateProjectiles` stream: g65's `gore_census`
  + inline gore loops are draws-only stand-ins that DROP per-kind where
  g16's bodies are live (§2c hunks 18–29); the unrowed-host halves
  (706/709/503/289/12 + the KEEP set) keep their census. The bomb family
  rides the pass stream with the frozen zero-rng Damage walk — no
  double-feed, no g65 arm.
- **d77/d78/g3/g1** — cues/state/census per §2; every census stub that
  duplicates a g65 body drops at the merge (§2a hunks 1–14 + §2b 15–17).

Gate command (post-merge, NOT run here): `cargo test --workspace --skip
migrates_version_2_kinds_to_item_ids` — judge by the `test result: ok`
lines.

## 6. Witnesses

WRITTEN NOT RUN (code-only). **117 `w4_g65_*` fns** at this compose tip:
the 116 at the seal (`37ec2f3aa`, incl.
`w4_g65_seal_the_deferred_kinds_are_inert` `:7538` — the 10 deferred kinds
89/91/92/183/655/868/869/949/996/1084 spawn nothing and draw nothing
through the drain: stream-inert until their owner lane ports them) plus
the §2c 503 fix's `w4_g65_the_503_gore_census_draws_the_type_it_overrides`
(`:7279`). The 26i §4 pins re-verified line-exact at this compose tip: 617
`:6175`, nebula sparks `:6191`, 710 `:6024`, 661 `:6159`, 452/454 `:6376`,
448/616 `:6543`. The smoke-puff witness (`:6638`) drops at merge with §2b
hunk 17; the gore-census witnesses that pin dropped rows (§2c) re-pin to
g16's body witnesses at the merge (post-merge 116).