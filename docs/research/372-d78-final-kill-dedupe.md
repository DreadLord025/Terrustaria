# 372 — d78 FINAL: the Kill death-effect dedupe adjudication + drop-vs-keep tables (W4-d78)

> Renumber 26f → 372 per registry §10.2 (`docs/research/150-research-number-registry.md`,
> d54 tip `08d21666e`): 26f belongs to W4-f20 (round-1 winner, 05:13:47); d78's
> `26f-d78-final-kill-dedupe.md` (fd59c51a0 12:50:43, pre-round-2 sweep miss)
> renumbers to **372**. Chain: 26f → 372. Every `research/26f` cite in this
> lane's docs/anchors/code-comments now reads `research/372`.

> Researched 2026-11-21 (final phase) against the LATEST pushed tips:
> g1 `5bd68ce2` (census-only push) + the TR-W4-g1 worktree's LOCAL state,
> g3 `8568ef37` (research/89 keep-one verdict), d77 `5f10560b` (the NEW
> Kill-host wave `0573b70f` — landed AFTER d78 batch 2c), g65 `7f0dc864`
> (pushed tip, batch 10) + TR-W4-g65 LOCAL `63c3d080` (batches 11-20,
> UNPUSHED), f1 `bcdfcd62`. C# cites are `decompiled/Terraria/Projectile.cs`
> (stable), re-read whole in ≤100-line windows. Rust cites pin
> `crates/tr-app/src/projectile.rs` / `projectile/data.rs` at d78's DONE tip
> `481d0bf4` (45 980 lines) unless a branch tip is named. This is the
> merger-facing publish: every d78 hunk gets a KEEP / DROP verdict with the
> surviving lane's exact fn-line-ranges.

## 0. The claim timeline (who pushed what, when)

| lane | tip | pushed at (2026-11-21) | content |
| --- | --- | --- | --- |
| d78 part 1 | `5fe6d940` | 00:52 | `magic_death_blast` (79/16/34/634/635/661) |
| g1 census | `5bd68ce2` | 01:02 | research/26d only — claims 452/454/459/521/448/80 |
| g3 aimed | `9ee2a7a1` | 03:09 | `aimed_spell_kill` (16/34/79 + cue + census) |
| g3 nebula | `778573ca` | 03:17 | `nebula_spirit_onyx_kill` (634/635/659/661 + cue + census) |
| d78 part 2a | `3fd3cc27` | 06:06 | `hostile_death_blast` + `damage_evp_boxed` (452/454/448) |
| g3 verdict | `0ae1b375` | 07:42 | research/89 §1 keep-one (g3 superset; 452/454/448 "g1 claim") |
| d78 part 2b | `cbae0aa3` | 08:04 | the child rows (246/249/620/1124) |
| g3 DONE | `8568ef37` | 09:15 | DOC-SWEEP (16 witnesses) |
| d78 part 2c | `481d0bf4` | 09:26 | `child_spawn_kill` (617/246/249/117) + child AI |
| d77 Kill-host | `0573b70f`/`5f10560b` | 12:01 | `betsy_arrow_kill` (710) + `nebula_arcanum_kill` (617 WHOLE) + the 620 row/AI — the 620 co-claim |
| g65 batches 11-20 | `63c3d080` (LOCAL) | unpushed | the 617/620/618/452/454/448/710 death-dust BODIES |

The g3 verdict's "452/454/448 are g1's claim, no 452/454 code exists in
d78's pushed tip" was true against `5fe6d940` ONLY — d78 part 2a
(`3fd3cc27`, 06:06) pushed the EVP trio BEFORE g3's 07:42 verdict commit,
which read the stale part-1 tip. The g1 code is STILL local-only today.

## 1. g1-dup adjudication (452/454/448) — d78 KEEPS, g1 folds

**Push-state probe (read-only, TR-W4-g1):** `git status --porcelain` shows
`M crates/tr-app/src/main.rs`, `M crates/tr-app/src/projectile.rs`
(+382 lines), `M crates/tr-app/src/projectile/data.rs` (+13 lines, the 522
row) and `?? crates/tr-app/src/projectile_kill_hooks_a.rs` (535 lines,
UNTRACKED). The pushed tip `5bd68ce2` carries ONLY
`docs/research/26d-projectile-kill-hooks-census.md` — **g1's port is
local-only, unpushed, uncommitted.** Per the Lead's rule the pushed-vs-pushed
compare does not apply; this is the local-probe path. The worktree was NOT
touched.

**Arm-by-arm (g1 local `projectile_kill_hooks_a.rs` + its pool diff vs
d78's pushed `hostile_death_blast`/`damage_evp_boxed`):**

| piece | g1 local | d78 `481d0bf4` | C# truth | verdict |
| --- | --- | --- | --- | --- |
| 452 resize pair | `resize(p,144)` + `resize(p,30)` (hooks_a:97-103), int-half recentre | `hostile_death_blast` (proj.rs:21612-21654) `(144.0, 30.0)` double-resize | `:72990`/`:73011` both centre-keep; 144/30 EVEN → int-half == float-half | same state; KEEP d78 |
| 454 recentre 208 | hooks_a:110-132 | same fn `(208.0, 208.0)` | `:73017-73020` | same; KEEP d78 |
| 448 recentre 112 | hooks_a:168-179 | same fn `(112.0, 112.0)` | `:73606-73609` | same; KEEP d78 |
| gore/ring-dust census draws | `gore()` 7 ranges/gore (hooks_a:60-68), `ring_dust()` 2 doubles (73-76); 452×1, 454×3+127 rings, 448×1 | SKIPPED with their draws (the g65/g16 lanes') | `:73004-73010`/`:73021-73043`/`:73610-73629` | g65 LOCAL batch-15/17 ports the same censuses INSIDE the bodies (`gore_census(rng,1/3/1,false,false)` + `double_angle_ring_pos`, death_dust.rs:2368/:2391/:2653) — g1's standalone census DROPS (keeping both double-rolls the stream) |
| EVP gate chain | `death_burst_player_hit` (g1 proj.rs diff @20661): hostile+damage gate, `hostile_burst_colliding` (hooks_a:295-301), truncated-rect overlap, `hurt_with_source(p.damage, ...)` | `damage_evp_boxed` (proj.rs:21705-21779): the same gates PLUS DamageVar(−owner.luck) `:14831`, hostileDamageScaling `:14842`, the EVP ×2 `:14844`, BossNoCheese-vs-General cooldown `:14779-14797` through `hurt_inner_cooldown` | `Damage_EVP` `:14773-14900` | **g1's hit is RAW damage — no DamageVar, no difficulty scale, no ×2, no cooldown lane** (its own doc calls these "STANDING single-lane deviations, M4-HAZARD-1"). d78 ports them verbatim. KEEP d78 |
| immune gate | none (hurt_with_source's General lane refuses while immune — the 452/454 BossNoCheese bypass is LOST) | `:14799` `!dead && (!immune \|\| num != General)` — BossNoCheese lands THROUGH spawn i-frames | `:14799` | KEEP d78 (g1 would refuse the boss blasts during i-frames — a behaviour bug vs C#) |
| Colliding order | gate BEFORE the overlap | overlap gates in C# order (state gates then rect) | `Colliding` runs the state arms first, then `Intersects` — both refuse identically | equivalent |
| 459/521/522/80 | hooks_a:140-162 (459 scale-gate), 190-237 (521 split), 259-275 (522 AI), 284-288 (80 census) + the pool tile-break + the 522 row | NOT in d78 (never claimed) | `:73338-73371`/`:73292-73327`/`:37742-37773`/`:75533-75553` | **g1-UNIQUE — g1 keeps them** |

**Verdict: keep d78's `hostile_death_blast` + `damage_evp_boxed` +
`player.rs`'s `hurt_cooldowns[6]`/`hurt_inner_cooldown` machinery; g1 drops
its 452/454/448 hooks (`phantasmal_eye_kill`, `phantasmal_sphere_kill`,
`saucer_missile_kill`, `hostile_burst_colliding`, the pool's
`death_burst_player_hit` + the three dispatcher arms) and keeps
459/521/522/80.** g1's dispatcher shrinks to the four unique kinds; the
C# body order holds (459 `:73338` < 521 `:73292`… the pool arms are
kind-disjoint from d78's, order unobservable). The 452/454/448 death CUES
already ride d77's `kill_tail_cue` (cue-first `:72989`/`:73016`, pre-snap
`:73605`) — no sound work is owed by either side.

## 2. g3 keep-g3 verdict (79/16/34/634/635/661) — CONFIRMED, d78 drops batch 1

g3 research/89 §1 (`0ae1b375`, tip `8568ef37`) ruled g3's two fns strict
supersets of d78's `magic_death_blast`. Re-verified arm-by-arm against both
pushed tips — the verdict STANDS:

| kind | C# arm | d78 `magic_death_blast` | g3 survivor (tip `8568ef37` line ranges) | extra in g3 |
| --- | --- | --- | --- | --- |
| 79 | `:71257-71328` | 128-box+pen−1+Damage+restore | `aimed_spell_kill` proj.rs:21789-21841 (+`rainbow_rod_death_draws` misc.rs:568-605) | Item10 @Center `:71266` + trail census |
| 16 | `:71329-71408` | same | same fn (+`magic_missile_death_draws` misc.rs:484-527) | Item10 @Center `:71338` + trail census |
| 34 | `:71409-71489` | 96-box, same | same fn (+`flamelash_death_draws` misc.rs:528-567) | Item10 @pos `:71418` + trail census |
| 634/635 | `:71995-72050` | 50-box+pen−1+Damage, no restore | `nebula_spirit_onyx_kill` proj.rs:21853-21900 (+`nebula_blaze_death_draws` misc.rs:606-633) | Item14 `:72015` + census |
| 661 | `:72226-72277` | 160-box, same | same fn (+`onyx_bolt_death_draws` misc.rs:661-687) | Item14 `:72234` + census |
| 659 | `:72503-72542` | NOT ported by d78 | same fn, ai0≥0 gate, pen kept (+`spirit_flame_death_draws` misc.rs:634-660) | g3-UNIQUE |

Resize helper: `aimed_spell_kill_resize` misc.rs:466-471 — byte-identical
maths to d78's inline centre-keep (`Entity.Center` get/set, `Entity.cs`:
`pos = oldCentre − new/2`; g1's int-half variant is equivalent for the even
blast sides). Both ride the SAME landed `damage_pve_boxed` seam (main
`a725acb0`:23261; d78 did NOT touch it — verified `git grep` at `5fe6d940^`).
The Magic Missile 8-tick immunity (`:14032`) rides the seam on both sides.

**Two reconciliation notes for the merger (g3-side fixes, d78 publishes):**

1. **The 634/635 cue position.** g3's fn queues Item14 at the POST-recentre
   position for all three kinds (comment cites `:72017`). The C# blaze arm
   plays at `:72015` — BEFORE `position = Center` (`:72016`) — i.e. the
   PRE-snap top-left; only the 659 (`:72511`) and 661 (`:72234`) play
   post-recentre. d77's `kill_tail_cue` 634/635 arm reads the pre-hook pos
   (correct). **Keep-one the CUE: drop the 634/635 (and 659/661/79/16/34)
   cue pushes from g3's two fns and let d77's `kill_tail_cue` own every
   one** — its positions are all correct (79/16 centre-invariant, 34
   post-restore == pre-hook pos, 634/635 pre-snap, 661 centre−80, 659
   centre−40 with the ai0 gate). Keeping both plays each cue TWICE.
2. **The census draws double-roll with g65.** g3's `*_death_draws` fns keep
   the explicit draws census-style; g65's LOCAL batches 12/14 port the SAME
   draws inside the dust bodies (`634|635` death_dust.rs:1976, `661`:2059,
   `659`:1583; the 79/16/34 trails are g65's remaining waves). At merge,
   when g65's body lands for a kind, the g3 census call for that kind drops
   (the g65-wins convention, the d8/g64 precedent). Until then g3's census
   is the only draw-keeper — keep it, drop it per-kind as g65 pushes.

**d78 batch-1 DROP hunks (tip `481d0bf4`, projectile.rs):**
- `magic_death_blast` fn + doc: **:21513-21586**.
- the death-hook call + comment: **:20102-20114** — KEEP the `let luck = …`
  line **:20106** (the 2a/2c calls read it).
- witness mod `w4_d78_magic_death_blast_tests` + doc: **:26004-26201**
  (5 tests; g3's `w4_g3_aimed_spell_kill_tests` :44897 +
  `w4_g3_nebula_spirit_onyx_kill_tests` :45002 cover the same state plus
  the cue/census).
- anchors.txt **:899** (`fn magic_death_blast` row) — replace with g3's
  rows 899-901 at merge (the build.rs canary reads the CURRENT tree; the
  row must go in the SAME commit as the fn drop).
- docs: the part-1 ANALYSIS section (:39248-39319) + PORT_MAP :758 row +
  the ROADMAP part-1 sentence get the keep-one addendum (done in this
  commit); the ROADMAP/PORT_MAP entries stay as the historical record.

**KEEP from batch 1: nothing code-side.** The seam (`damage_pve_boxed`)
was already on main; the hook placement argument survives as g3's calls
sit at the same death hook.

## 3. The 620-fans d77 co-claim — d77 wins the 617 whole arm, d78 keeps 246/249/117

d77's Kill-host wave (`0573b70f`, pushed 12:01, AFTER d78 2c's 09:26) ports
the 617 arm WHOLE — the same C# site (`:72278-72373`) as d78's
`child_spawn_kill` NEBULA_ARCANUM arm. **Same-C#-site: YES.** It is a
strict superset (the cue + the dust/gore census + the swept-618 inline
Kill), so the g3-style keep-one applies: **KEEP d77's
`nebula_arcanum_kill`, DROP d78's 617 arm.**

**The dedupe table (d77 tip `5f10560b` vs d78 tip `481d0bf4`):**

| piece | d77 survivor (tip lines) | d78 duplicate (tip lines) | verdict |
| --- | --- | --- | --- |
| 617 Kill arm (176-box+pen+Damage+618 sweep+two 620 fans) | `nebula_arcanum_kill` proj.rs:20577-20684 + hook :20046-20053 | `child_spawn_kill`'s NEBULA_ARCANUM arm :21832-21941 | KEEP d77 (adds Item14 @centre−88 `:72286` + `nebula_arcanum_death_draws` misc.rs:495-531 + the swept-618 inline `ai[0]=86` latch `:72398` + `nebula_shard_kill_draws` misc.rs:532-543) |
| 620 row (8×8 st29 α255 pen1 magic) | `NEBULA_ARCANUM_SHARD` data.rs:1058 + :2833 | `NEBULA_ARCANUM_EXPLOSION_SHOT_SHARD` data.rs:1061 + :2849-2852 | byte-identical rows (`(8,8,29,1,3600,0,true,false,false,(false,-2,false,-1,255,1.0))`); KEEP ONE — d77's name is shorter, d78's is the verbatim ProjectileID name; either survives, rename the other's uses (d78's 2c-witness/617-arm uses drop anyway; the dispatch gate :17891 renames) |
| 620 AI_029 arm + bounce | `ai_nebula_shard` proj.rs:7125-7153 (the >40 kill no-return + 0.2/18/0.98 + the :18796-18806 full bounce inline) | `ai_nebula_shard` proj.rs:7125-7159 (same maths; bounce identical, d78 adds the !tile_collide early-out) | same behaviour; KEEP d77's (its dispatch :17815 already routes it); d78's :7112-7159 drops |
| style-29 dispatch kind-gate | :17814-17815 | :17885-17892 | KEEP d77's |
| 618/620 own Kill census (`:72394-72417`) | the death-hook block :20054-20064 (ai0=86 latch + `nebula_shard_kill_draws`) | none (d78 skipped it — g65's lane) | KEEP d77's; g65's LOCAL `620|618` body (death_dust.rs:2146) SUPERSEDES the census at its push — drop d77's census block then (the g65-wins convention) |
| 617 fan witness | d77's :25040-25140 (`shards[0].damage == 65` etc.) | `w4_d78_b2c_the_arcanum_bolt_bursts_into_two_shard_fans` :26516-26573 + `…sweeps_its_keyed_subshots` :26575-26590 | DROP d78's two (d77's cover the same state + the cue/census) |
| 620 AI witness | d77's :25260-25280 | `…the_arcanum_shard_ages_and_falls` :26776-26798 | DROP d78's |
| magic-census entries for 620 | data.rs:4318 (`NEBULA_ARCANUM_SHARD` in the magic set) | data.rs:4366-4367 + the flags pin :4835-4836 + the row-witness 620 block :4892-4899 + :4914 + the spawnable loop member :4916 | KEEP ONE set (rename to the surviving const) |

**d78 2c surgery (the fn stays, the 617 arm goes):**
- `child_spawn_kill` :21821-22095 KEEPS the STYNGER (:21942-22009),
  STYNGER_SHRAPNEL (:22010-22032) and BONE_ARROW (:22033-22094) arms —
  **d78-UNIQUE** (d77/g3/g1 port none of them; f1 arms only their cues).
  The NEBULA_ARCANUM arm :21832-21941 DROPS; the fn doc's 617 paragraph
  (:21786-21794) rewords to cite d77's host.
- the hook call :20129-20147 stays (the comment's 617 mention rewords).
- the foreign-owner witness :26711-26729 drops NEBULA_ARCANUM from its
  kind list (d77's own foreign-owner coverage: its 617 fn gates at :72327
  AFTER the resize — see the deviation note below).
- **Owner-gate deviation (recorded):** d78 folded the `:12504` owner gate
  AHEAD of the 617 resize (a foreign corpse never resizes); d77 (and the
  C#) resize + Damage + cue + census FIRST, then gate only the child block
  (`:72327`). The corpse box of a foreign-owned 617 differs (176 vs row).
  d77's shape is the C#'s — it survives; d78's fold was the
  `magic_death_blast` convention and dies with the arm.
- **Key-sweep note:** d78 matched `ai[1] == f32::from_bits(key)`; d77
  matches `ai[1].to_bits() == key_bits`. Equal for every non-NaN key (the
  C# `(float)key` union); d77's bit-compare is the stricter NaN-safe form.
  d77 also skips the `j != slot` guard — harmless (the dying 617 is not a
  618) — and runs the swept 618's Kill INLINE (the C# `Kill()` call at
  `:72333` runs immediately, not at the pool's next death pass): d77 is
  MORE faithful here; d78's `pool.kill(j)` deferred the 618's dust census
  to the slot's own hook. Another reason d77's arm survives.

## 4. The batch-1 census question — does keep-g3 drop 2b/2c too? NO.

The Lead's question: g3's superset verdict covers 79/16/34/634/635/661 —
the batch-1 kinds. Batch 2b's rows are 246/249/620/1124 and batch 2c's
arms are 617/246/249/117 — **disjoint from the g3 verdict except the 620
row/AI (the d77 co-claim, §3)**. Full d78 drop-vs-keep FINAL table:

| d78 hunk (tip `481d0bf4`) | verdict | surviving lane |
| --- | --- | --- |
| `magic_death_blast` :21513-21586 + call :20102-20114 (keep :20106) + tests :26004-26201 + anchors :899 | **DROP** | g3 `aimed_spell_kill`/`nebula_spirit_onyx_kill` (§2) |
| `hostile_death_blast` :21588-21654 | **KEEP** | d78 (g1 local-only + raw-damage EVP, §1) |
| `damage_evp_boxed` :21656-21779 | **KEEP** | d78 (the verbatim Damage_EVP port) |
| `player.rs` hurt_cooldowns[6] (:1398-1408, ctor :2757), update_immunity decay (:4223/:4280), `hurt_inner_cooldown` (:4345-4488+), set_immune_time array write | **KEEP** | d78 (g1's hurt_with_source cannot carry the BossNoCheese lane) |
| hostile hook call :20115-20128 + tests `w4_d78_hostile_death_blast_tests` :26203-26446 + anchors :900 | **KEEP** | d78 |
| 2b rows: STYNGER :3817-3825, STYNGER_SHRAPNEL :3826-3830, BONE_SHARD :3295-3302 + consts :1062-1073 + ranged census :4802-4806 + proj_frames :4722-4729 + row-witness 246/249/1124 blocks | **KEEP** | d78-UNIQUE (no other tip rows them; f1's 246/249/1124 cue arms read these rows) |
| 2b row: 620 const :1058-1061 + row :2843-2852 + magic census :4366-4367 + flags pin :4835-4836 + witness 620 blocks | **DROP/RENAME** | d77 `NEBULA_ARCANUM_SHARD` (§3 — byte-identical row; keep one name) |
| `child_spawn_kill` 617 arm :21832-21941 + its two witnesses :26516-26590 | **DROP** | d77 `nebula_arcanum_kill` (§3) |
| `child_spawn_kill` 246/249/117 arms :21942-22094 + fn shell/doc + hook call :20129-20147 + witnesses :26592-26775, :26799-26818 + anchors :902 (reworded) | **KEEP** | d78-UNIQUE |
| `ai_nebula_shard` :7112-7159 + dispatch :17885-17892 + witness :26776-26798 | **DROP** | d77's same-named fn (§3) |
| `ai_stynger` :8804-8828, `ai_bone_shard` :8830-8864 + dispatch :17025-17038 + witnesses :26731-26775 | **KEEP** | d78-UNIQUE |
| plain-walk 1124 Colliding gate :23416-23423 | **KEEP** | d78-UNIQUE (`:15413-15418`) |
| 246/249 7-tick immunity: plain :23836-23839 + boxed :24156-24157 + witness :26800-26818 | **KEEP** | d78-UNIQUE (`:14024-14031`; d77's boxed ladder has NO 246/249 arm — verified at its tip :24034+) |
| 2c witnesses :26448-26819 mod shell | **KEEP** (minus the three 617/620 tests above) | d78 |
| anchors :901 (the 2b rows row) | **KEEP** (reword: the 620 clause cites d77's const) | d78 |

Net: d78 keeps parts 2a/2b/2c minus the batch-1 fn and the 617/620
duplicates — roughly 1 100 of its 1 596 projectile.rs added lines survive
verbatim; the rest folds into g3/d77 supersets that carry MORE (cues +
censuses) than d78's skipped-with-their-draws shapes.

## 5. Research FINAL — the lane ledger after this adjudication

- **79/16/34/634/635/659/661**: g3 owns state+census; d77 owns the cues
  (drop g3's inline pushes, §2 note 1); g65 owns the bodies when pushed.
- **452/454/448**: d78 owns state+EVP; d77 owns the cues (armed); g65
  LOCAL owns the dust/gore bodies (unpushed — the merge waits or re-arms);
  g1 drops its three hooks, keeps 459/521/522/80.
- **617/618/620**: d77 owns the whole Kill host + row + AI + census; g65
  LOCAL owns the bodies (supersedes d77's census at push); d78 drops its
  617 arm/620 row/AI.
- **710**: d77 owns the Kill host (`betsy_arrow_kill` :20520-20551) +
  census; g65 LOCAL :1868 owns the body; g3's DOC-SWEEP owns the 710
  ON-HIT 5-shard fan (`:14121`/`:53964`) — a DIFFERENT C# site, no dup.
- **246/249/117/1124**: d78 owns rows+Kill+AI+gates (UNIQUE); f1's cue
  arms (246/249 `:76946`/`:76989`, 1124 `:74754`, 117 `:74736` on d77)
  are cue-first pre-resize reads — correct against d78's post-cue resize
  order, no position conflict.
- **521/522/459/80**: g1 (local) — unpushed; d77's 521 Item110 cue
  (`:73294`, cue-first) and 80 Item27 gate (`:75537`) are already armed
  and read pre-hook state — compatible.
- **f1 vs d77**: d77's superset verdict re-verified at `5f10560b` vs f1
  `bcdfcd62` (d77's own table) — f1's `kill_sound_cue` +
  `rocket_kill_census` drop, its numeric owed arms fold into d77 (already
  armed by d77's re-probe wave: 949/171-family/590/539/523/12/321/576/
  728/729/1115/1120). f1-UNIQUE remainder: the Flame Burst 664/666/668
  Item100 `:71848` (apprentice_t3 thread) + the part-7 second-owed arms
  d77's re-probe did NOT take (931/1039/1047/1037/1049/1048/1053/1054/
  1055/99-family/1005/655/723/724-726/1133/128/503/31/584/514) — those
  fold into `kill_tail_cue` per f1's published fold-hunks (its ANALYSIS
  part-7), converting to the 6-tuple.

**Verification.** Code-only: every arm re-read against its C# block in
≤100-line windows (the 452/454/448 resizes + Damage_EVP chain, the 617
whole arm, the 246/249 cue-first order, the 634/635 pre-snap cue, the
Colliding gates, the Resize/Entity.Center maths); every branch tip
re-fetched and re-probed at adjudication time (the d77 12:01 push was
caught by the final re-probe — the §3 verdict reflects it); the g1/g65
worktrees probed READ-ONLY (`git status`/`git diff`/direct file reads, no
writes, no junction recursion). No cargo/rustc/rustfmt/tests/graphify
invoked (the Lead's constraint). The 620-row equivalence witness
(`w4_d78_final_the_shard_row_matches_the_d77_co_claim`) is WRITTEN NOT RUN.

## 6. SEAL addenda (2026-11-21, the resume pass — this document is SEALED)

The FINAL adjudication (`fd59c51a0`) landed AND pushed before the prior
session died (ls-remote: `refs/heads/task/W4-d78-ui-sounds-menus` ==
`fd59c51a0527…`). This section pins the three coordination receipts the
merger and the sibling lanes consume. All pins re-probed READ-ONLY at seal
time; no sibling worktree was written.

### 6a. g1-drop-hunks — REFRESHED against g1's LIVE local state

**State-change note:** at adjudication g1's untracked
`projectile_kill_hooks_a.rs` was 535 lines; at seal time it is **602
lines** (R3-g1-resume is actively committing it — the file gained the
witness suite). The §1 verdict is UNCHANGED (re-read arm-by-arm); the
line pins below REPLACE the §1 hooks_a cites. g1's pushed tip is still
`5bd68ce2` (census-only) — the port remains LOCAL-ONLY.

**DROP from `projectile_kill_hooks_a.rs` (602-line state):**
- `phantasmal_eye_kill` + doc **:98-109** (the 452 resize pair + gore census).
- `phantasmal_sphere_kill` + doc **:111-138** (the 454 208-box + 7/120 ring
  dusts + 3 gores).
- `saucer_missile_kill` + doc **:177-192** (the 448 112-box + gore census).
- `hostile_burst_colliding` + doc **:324-335** (d78's `damage_evp_boxed`
  carries the same gates in C# Colliding order).
- witnesses: 452 **:400-413**, 454 **:417-427**, 448 **:460-469**,
  the colliding-gates test **:572-590**.
- the shared `gore()` :66-74 / `ring_dust()` :79-82 / `resize()` :89-96
  helpers **STAY** — the kept 459/521 arms consume them; only the three
  hostile hooks' census CALLS drop (g65's LOCAL batch-15/17 bodies own
  those rolls; keeping both double-rolls the stream).

**DROP from g1's local `projectile.rs` diff:**
- dispatcher arms in `kill_hooks_a` (:20677-20767): the PHANTASMAL_EYE arm
  **:20689-20694**, PHANTASMAL_SPHERE **:20695-20700**, SAUCER_MISSILE
  **:20745-20750** (comments included).
- `death_burst_player_hit` + doc **:20769-20806** — its :20805
  `player.hurt_with_source(damage, cx, rng, dodgeable, kind)` is the RAW
  damage hit (§1): no DamageVar(−luck) `:14831`, no hostileDamageScaling
  `:14842`, no EVP ×2 `:14844`, no BossNoCheese cooldown `:14782-14797`,
  and the General immune lane would REFUSE the 452/454 boss blasts the
  C# `:14799` gate lands through i-frames. **This is the
  g1-raw-damage-no-BossNoCheese deviation** — g1's own doc calls it
  M4-HAZARD-1; d78's `damage_evp_boxed` + `hurt_inner_cooldown` port it
  verbatim, so the deviation DIES with the fold, it does not survive.

**KEEP (g1-UNIQUE, kind-disjoint from d78):** `charged_blaster_orb_kill`
:147-154 + `…_draws` :161-175 (459, gate widens to 459|709 at merge per
the f20 cross-note), `crystal_pulse2_kill_draws` :201-211,
`crystal_pulse_kill` :222-269 + `PulseShard` :275-282 (521),
`crystal_pulse2_step` :291-309 (522 AI), `ice_block_kill_draws` :318-322
(80); dispatcher arms :20704-20706 (522), :20710-20728 (521 split spawns),
:20736-20744 (459 PVE burst), :20753-20764 (80 tile break); the hook call
:20125; the 522 const + row in data.rs; witnesses :433-456 (459),
:477-524 (521), :529-551/:555-567 (522), :594-601 (80) + the pool-side
522-row/split tests :44623/:44636.

### 6b. d77-617-drop — the d78-side MUTUAL RECEIPT (for d77d to mirror)

d78 RECEIPTS d77's superset at tip `5f10560b` (Kill-host wave `0573b70f`,
pushed 12:01): `nebula_arcanum_kill` proj.rs **:20577-20684** (hook
:20052) ports the SAME 617 C# site `:72278-72373` WHOLE and adds the
Item14 @centre−88 cue, the dust/gore census, the swept-618 INLINE Kill
(the C# `Kill()` at `:72333` runs immediately) and the C#-order `:72327`
owner gate. **d78 DROP list (its own tip `481d0bf4` line space, executed
by the merger — d78's branch stays frozen as the historical record):**
the `child_spawn_kill` NEBULA_ARCANUM arm **:21832-21941**; the 620 const
data.rs **:1058-1061** + row **:2843-2852** + magic census **:4366-4367**
+ flags pin **:4835-4836** + row-witness blocks **:4892-4899/:4914/:4916**
(byte-identical to d77's `NEBULA_ARCANUM_SHARD` — pinned by the
`w4_d78_final_the_shard_row_matches_the_d77_co_claim` witness, written
not run); `ai_nebula_shard` **:7112-7159** + style-29 dispatch
**:17885-17892**; the three witnesses **:26516-26573**, **:26575-26590**,
**:26776-26798**; the foreign-owner witness :26711-26729 drops
NEBULA_ARCANUM from its kind list. d78's ahead-of-resize owner-gate fold
and its deferred `pool.kill(j)` 618 sweep both die with the arm — d77's
shapes are the C#'s. **KEEP: the 246/249/117 arms + rows + child AI +
gates (§4) — d77's boxed ladder has NO 246/249 arm (re-verified at
`5f10560b`).** d77d mirrors this receipt on its side; the 617/618/620
lane ledger (§5) is the single ownership record.

### 6c. g3-cue-position-fix — exact hunks (g3-side fix, d78 publishes)

Re-verified by direct C# read at seal time: `Projectile.cs:72015`
`SoundEngine.PlaySound(SoundID.Item14, base.position)` precedes `:72016`
`base.position = base.Center` — the 634/635 blaze cue plays at the
**PRE-snap top-left**. g3 tip `8568ef37` `nebula_spirit_onyx_kill` queues
its cue at **:21884-21890** AFTER `aimed_spell_kill_resize` (:21875) —
post-recentre, WRONG for 634/635 (right for 659 `:72511` / 661 `:72234`,
but d77's map derives those correctly too). d77's `kill_tail_cue`
(tip `5f10560b`) reads the pre-hook pos: **:20250** `634 | 635 =>
Some((2, 14, pos.x, pos.y, 1.0))` — CORRECT; :20222 (79/16 centre),
:20226 (34 post-restore), :20255 (661 centre−80), :20260 (659 centre−40,
ai0 gate) all correct.

**The fix (g3 executes on its branch; keeping both plays every cue TWICE):**
- DROP the cue block in `aimed_spell_kill` **:21823-21832** (the `(sx, sy)`
  read + the `pending_proj_sounds.push((2, 10, …))`).
- DROP the cue block in `nebula_spirit_onyx_kill` **:21884-21890** (the
  `(sx, sy)` read + the `pending_proj_sounds.push((2, 14, …))`).
- KEEP d77's `kill_tail_cue` arms as the single cue owner —
  **keep-d77-kill_tail_cue**, including its 634/635 pre-snap arm.
- g3's census calls (:21833-21840, :21891-21899) stay until g65's bodies
  push per-kind (§2 note 2) — the cue drop does NOT touch them.
- g3's cue witnesses (`w4_g3_*` cue assertions) reword to cite d77's map.

### 6d. The blasts-family bible (the sealed ownership table)

| family | STATE host | CUE owner | census now | bodies at push |
| --- | --- | --- | --- | --- |
| 79/16/34 | g3 `aimed_spell_kill` | d77 `kill_tail_cue` :20222/:20226 | g3 `*_death_draws` | g65 (trails pending) |
| 634/635/659/661 | g3 `nebula_spirit_onyx_kill` | d77 :20250/:20255/:20260 (634/635 PRE-snap) | g3 census | g65 LOCAL 12/14 |
| 452/454/448 | **d78** `hostile_death_blast` + `damage_evp_boxed` | d77 (cue-first 29/4, pre-snap Item14) | g65 LOCAL 15/17 (inside bodies) | g65 LOCAL |
| 617/618/620 | **d77** `nebula_arcanum_kill` + row + AI | d77 (Item14 @centre−88, inline) | d77 census block :20054-20064 | g65 LOCAL :2146 supersedes |
| 710 | d77 `betsy_arrow_kill` | d77 (Item14 @centre−20) | d77 census | g65 LOCAL :1868 |
| 246/249/117/1124 | **d78** rows + `child_spawn_kill` + child AI + gates | f1 cue arms (cue-first, compatible) | — | g65/g16 lanes |
| 459/521/522/80 | **g1** (local, unpushed) | d77 (521 Item110, 80 Item27 gate — armed) | g1 module censuses | g65/g16 at wiring |

**SEAL.** The four-way adjudication is final; §1-§5 verdicts stand as
pushed at `fd59c51a0`; §6a-c are the merger/sibling receipts with
seal-time line pins. d78's branch is frozen — every DROP executes at
merge (or on the sibling's own branch for 6c), never by rewriting this
record. Code-only: no cargo/tests/rustfmt/graphify invoked; sibling
worktrees probed READ-ONLY (direct paths, no junction recursion).