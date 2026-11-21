# 26i — g65 Kill-bodies keep-one receipt (the d78c blasts cross-lane publish)

Status: RECEIPT PUBLISHED (W4-g65 push-rescue session, 2026-11-21).
Code-only — no cargo/rustc/tests/rustfmt/graphify invoked. C# cites are
`decompiled/Terraria/Projectile.cs` (stable), read in ≤80-line windows.
Rust cites pin g65's PUSHED tip `b0842907c`
(`crates/tr-app/src/projectile_death_dust.rs`, 7315 lines) unless a branch
tip is named. Companion to d78c's `26f-d78-final-kill-dedupe.md` (fd59c51a)
— this is g65's side of the same adjudication: the g65-wins convention
executed now that the bodies are PUSHED.

## 0. Push-rescue + the local-commit census

- The Lead's briefing saw `ac33dd6d7` (21 local ahead) + 1 dirty file.
  The prior session finished: it committed the dirty `docs/ANALYSIS.md`
  as `b0842907c` ("publish the final Kill dust census-completeness
  accounting", +77 lines) → 22 local ahead, tree CLEAN.
- Two mtime probes 7 minutes apart (11:13:06Z / 11:20:11Z, >90s) over all
  679 tracked files: SHA256-identical snapshots
  (`4438A105…7FDD32`), newest mtime 11:03:12Z frozen → IDLE, safe to act.
- PUSH: attempts 1-3 failed (curl 55 send-abort / connection reset / DNS
  resolve failure — the flaky network), attempt 4 LANDED:
  `7f0dc8649..b0842907c  task/W4-g65-proj-death-dust`.
  `git ls-remote` confirms origin at `b0842907c`; `rev-list --count` = 0.
- The 22 commits: batches 11-31 (`b4ae9a4a4`…`ac33dd6d7`, the re-census
  remainder arms) + the final accounting `b0842907c`. Every Kill dust arm
  the re-census listed is now on origin.

## 1. The remaining-234 census — CLOSED, zero genuinely unclaimed

Independent re-scan (not just the prior session's accounting): every
`Dust.NewDust*`/`CloneDust` line in `Kill()` (`:70642-77262`) attributed to
its owning kind arm (318 arm headers → 292 dust-owning kinds), matched
against the drain's match arms (317 distinct kinds incl. the `121..=126`
/`411..=414` ranges).

| bucket | sites | detail |
| ------ | ----- | ------ |
| total Kill dust sites | **406** | 399 `NewDust*` + 7 `CloneDust` (the e3/125 "399" count was NewDust-only; the 7 clones at :71063/:71141/:71184/:71235/:71307/:71373/:71453 ride g65's 1116/932/931+1039/1047/79/16/34 trail-ring arms) |
| g65 drain covers | **393** | every site whose owning kind has a drain arm |
| other-lane claimed | **6** | 442 `:73534` → i16 `dart_electro_firework_kill` (in-tree `projectile.rs:20129`, the Next(3,7) count draw + sphere spawn); 608 `:72185-72212` (5) → d12 `solar_counter_kill` (`projectile_ai_explosions.rs:225`, cites `:72177-72225`) |
| g65-deferred, documented | **7** | 1084 `:70963` + 949 `:70983` (TorchID.Dust table + Main.DiscoR/G/B App-frame state); 868/869/996 share `:73710` (WorldGen.RandomizeTreeStyle/Background/MoonState state-bearing draws); 655 `:75085` (wet-item/NPC-spawn cross-seam); 91/92 share `:75217` (star-spawn draws ride `special_arrow_kill`); 89 `:75255` (shard-split draws ride `special_bullet_kill`); 183 `:76436` (bee-burst draws ride `beenade_death`) |

**Remaining unclaimed: 0.** The ledger's owed-pool `g65-Kill-remainder-234`
is fully discharged: 234 = the e3/125 uncited count at `7f0dc864` (batch
10); batches 11-31 + the accounting closed it. The 7 deferred sites are
NOT free — each needs a cross-seam dedupe or App-frame state the drain's
`World`/`SpawnCtx` does not carry (the ANALYSIS "Census completeness"
section, `b0842907c`). No task-seed owed; no continuation dispatch needed.

## 2. Keep-one: g65 bodies vs d78 `damage_evp_boxed` / d77 `kill_tail_cue`

The d78c verdict (26f §5) assigned: d78 owns 452/454/448 state+EVP; d77
owns the cues + the 617 whole Kill host; g65 owns the dust/gore BODIES
"when pushed — the merge waits or re-arms". **The bodies are now pushed.**
Verified arm-by-arm (C# re-read vs the g65 drain arms):

| kind(s) | C# arm | g65 body (death_dust.rs @ b0842907c) | d78/d77 counterpart | verdict |
| ------- | ------ | ------------------------------------ | ------------------- | ------- |
| 452 | `:72987-73013` | `:2378` — 4× dust-31 + 40 pairs dust-229 + `gore_census(1,false,false)` | d78 `hostile_death_blast`/`damage_evp_boxed` (proj.rs:21612/21705 @ fd59c51a): resize+Damage, "dust/gore SKIPPED" | KEEP BOTH — disjoint halves, no dup |
| 454 | `:73014-73045` | `:2401` — 7× dust-31 + 60 pairs dust-229 `double_angle_ring_pos` + `gore_census(3,…)` | same d78 fns | KEEP BOTH |
| 448 | `:73603-73631` | `:2663` — 4× dust-31 + 40 pairs dust-228 + `gore_census(1,…)` (C# re-read: no scatter, gravity ON, 1 gore — matches) | same d78 fns | KEEP BOTH |
| 617 | `:72278-72326` | `:2121` — 4+30×2+10+10 dust (scatter_y/x) + `gore_census(2,true,false)`; draw-count identical to d77 `nebula_arcanum_death_draws` (26f-pinned) | d77 `nebula_arcanum_kill` (proj.rs:20577 @ 5f10560b) owns resize/pen/Damage/Item14/owner-gate/618-sweep/620-fans | KEEP d77 host + g65 body; DROP d77's census call (below) |
| 620/618 | `:72394-72417` | `:2160` — `crystal_spark_burst` 10× (618 folds ai0=86, the pre-write latch); Next(3) gate + Next(-10,11) pair/else-branch = d77 `nebula_shard_kill_draws` draw-for-draw | d77 death-hook census block + the inline swept-618 census | DROP the death-hook block; KEEP the inline (below) |
| 710 | `:71652-71679` | `:1868` — 2× dust-31 Y-scatter + 10× dust-6 + 5× dust-31 X-scatter (RotatedByRandom pair + NextDouble radius per scatter; X-tail pair-only) = d77 `betsy_arrow_death_draws` draw-for-draw | d77 `betsy_arrow_kill` (proj.rs:20520) owns ai[1]=-1/40-box/Damage/Item14 | KEEP d77 host + g65 body; DROP d77's census call |
| 79/16/34 | `:71257/:71329/:71409` | `:4023` (1047\|79 rainbow_trail_ring) / `:4035` (16) / `:4112` (34) — the CloneDust trail rings | d77 `kill_tail_cue` Item10 cues (centre-invariant / post-restore positions, 26f §2) | KEEP BOTH — g65 draws no cue, d77 draws no dust |
| 634/635 | `:71995-72093` | `:1986` — the 5-set/3-set rolls + 40-pair scatter + kick | d77 cue Item14 PRE-snap (`:72015`); g3 `nebula_blaze_death_draws` census | KEEP g65 body + d77 cue; DROP g3 census call per-kind |
| 661 | `:72226-72277` | `:2069` | d77 cue centre−80; g3 `onyx_bolt_death_draws` | same |
| 659 | `:72503-72564` | `:1593` | d77 cue centre−40 ai0-gated; g3 `spirit_flame_death_draws` | same |

**The cue/state lanes draw ZERO dust; the g65 bodies play ZERO cues.**
The only double-roll risk is the census stubs (d77's three `*_draws` fns,
g3's six) — pure draw-count keepers that the g65 bodies now supersede.

## 3. Merger drop-hunks (execute at the g65×d77×g3 merge, in this order)

d77 tip `5f10560b` (`task/W4-d77-proj-sounds-3`):

1. `projectile.rs:20552` — the `betsy_arrow_death_draws(rng)` call in
   `betsy_arrow_kill` → DROP (g65's 710 body drains it). Keep the host.
2. `projectile.rs:20612` — the `nebula_arcanum_death_draws(rng)` call in
   `nebula_arcanum_kill` → DROP (g65's 617 body). Keep the host + the
   owner-gated tail.
3. `projectile.rs:20054-20065` — the death-hook 618/620 census block
   (the ai0=86 latch + `nebula_shard_kill_draws`) → DROP whole: g65's
   death-walk snapshot covers every naturally-dying 618/620, and the
   `620 | 618` arm folds the 86 latch itself (the snapshot is pre-write).
4. `projectile.rs:20630` — the INLINE swept-618 census inside
   `nebula_arcanum_kill`'s sweep → **KEEP, do not drop**: d77's sweep
   calls `pool.kill(j)` (`:15495`, a bare deactivate) which never reaches
   the death-walk snapshot hook — the swept 618's draws exist ONLY here.
   Merger option (cleaner, optional): route the sweep through the
   death-walk path so g65's drain owns it, then drop this call too; the
   ai0=86 write at `:20629` must then precede the snapshot. Until that
   rewire, keeping the inline census is the stream-correct form.
5. `projectile_ai_misc.rs` — retire the three now-unused census fns:
   `betsy_arrow_death_draws` (`:469`), `nebula_arcanum_death_draws`
   (`:495`), `nebula_shard_kill_draws` (`:532` — stays live while hunk 4
   keeps its call; retire only after the sweep rewire).

g3 tip `8568ef37` (`task/W4-g3-proj-ai-missing-c`), per 26f §2 note 2
("drop per-kind as g65 pushes" — all six are now pushed):

6. `projectile.rs:21836-21838` — the `magic_missile_death_draws` /
   `flamelash_death_draws` / `rainbow_rod_death_draws` census calls
   (16/34/79) → DROP (g65 `:4035`/`:4112`/`:4023`).
7. `projectile.rs:21894-21897` — `nebula_blaze_death_draws` (634/635) /
   `spirit_flame_death_draws` (659) / `onyx_bolt_death_draws` (661) →
   DROP (g65 `:1986`/`:1593`/`:2069`).
8. `projectile_ai_misc.rs:484/528/568/606/634/661` — the six census fn
   bodies retire with their calls. g3's STATE hosts (`aimed_spell_kill`,
   `nebula_spirit_onyx_kill`) KEEP per 26f — only the census calls drop.

d78 tip `fd59c51a`: NO g65-facing drops — `damage_evp_boxed` /
`hostile_death_blast` carry no dust draws (verified: "dust/gore SKIPPED"
at every resize). g1's standalone 452/454/448 census DROPS per 26f §1
(g65's `gore_census` + ring bodies carry the same draws; keeping both
double-rolls the stream).

## 4. Witnesses

The g65 bodies' witnesses are WRITTEN NOT RUN (code-only): the 617
84-dust pin (`w4_g65_the_617_nebula_arcanum`, death_dust.rs:6157), the
620/618/619 sparks (`:6173`), the 710 ballista (`:6006`), the 661 blaze
(`:6141`), the 452/454 (`:6358`) + 448/616 (`:6525`) bursts. The
draw-equivalence claims above were verified by C# re-read + census-fn
line-by-line comparison, NOT by running the tests.

## 5. ADDENDUM — the gun Kill smoke-dup verdict (g63-298 §3.3): d6 SURVIVES, g65 DROPS

The g63 FINAL (`298-proj-trail-dust-family-final.md` §3.3, g63 tip
`10775b366`; corroborated by d6's RETIRE `174c0d541`, research/247 there)
adjudicated the 358/406/1042 smoke puffs — DOUBLE-PORTED by g65 batch 18
(`1bbeec5b5`) and d6 residual 8 (`1b54dcbbd`, `projectile_gun_splash.rs`);
both port `Projectile.cs:74029-74147` with the same per-dust draw census.
**Verdict: d6-SURVIVES-g65-DROPS.** d6's hook is the strictly faithful port:
the TRUE `old_velocity` latch + the style-60 scale-death ×5 base
(`:28999-29000` kicks velocity BEFORE Kill; d6 `projectile.rs:20545` selects
`p.vel` on `scale <= 0`) + the corpse `p.vel = base * 0.2` write (`:20546`).
g65's `DeathSnapshot::of` sets `old_vel: p.vel` — a documented STAND-IN
(`death_dust.rs:437-442`), wrong for moving guns — and `smoke_puff` never
writes the corpse. Merger DROP-hunks on g65's file (ranges re-verified at
this receipt's tip `f6e07bb90`, code unchanged below `:7480`):

- **D1** `projectile_death_dust.rs:2786-2794` — the 358/406/1042 match arms
  DROP (guns fall through to the default, no draws); replace with the
  298 §3.3 ownership comment citing d6 + research/298 §3.3.
- **D2** `:4834-4885` — the `smoke_puff` helper DROPs whole (doc comment
  included; its only callers were D1's arms).
- **D3** `:6616-6634` — the `w4_g65_the_smoke_puffs` witness DROPs (d6's
  three gun-splash witnesses cover the surviving census).

d6 keeps EVERYTHING (module + queue + death-hook + pass-tail drain); g65's
generic snapshot hook keeps snapshotting the gun kinds — inert, no arm
consumes them post-drop. One survivor, never both: a double-feed desyncs
the stream. Post-drop drain: 316−3 = **313 kinds**; the 406-site closure
holds with the 3 gun sites moving to other-lane-claimed (390+9+7 = 406).
The FINAL bible (research/308 §2b) carries the same hunks.