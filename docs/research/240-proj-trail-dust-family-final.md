# 240 — W4-g63 projectile TRAIL dust (kinds ≤400): FINAL + the spawn-dust-family declaration (SEALED)

(Number: the d54 research/150 registry's 237+ free block. Seal-scan of
every pushed tip's `docs/research/` found 237/238/239 ALL taken:
237 = e23 `237-bestiary-ui-final.md` (first-add 10:53:03, `7a7779690`)
+ the g48 loser `237-f11-tail-fix.md` (12:15:09, `34029437a` — owed a
renumber); 238 = d42 `238-d42-roster-sheets.md` (12:15:55, `9eadc3043`)
vs c8 `238-c8-crawler-final-census.md` (12:24:19, `2a0b5c701`) vs g61
`238-g61-spawn-family-final.md` (SEALED self-renumber 237→238, its
237 draft first-add 12:12:59 `fd406ecba`, the 238 filename add
12:55:22 `6e922fc60`); 239 = d42 `239-d42-final-keep-one-and-band.md`
(12:55:30, `bac408be0`) — also g48's expected renumber slot per g61's
note. This lane takes **240**, the first number NO tip carries —
no collision created. The 237/238/239 contention is the merger's to
adjudicate per registry-150 §1 first-pushed-wins; the evidence rows
above are this lane's scan, recorded not assigned.)

**STATUS: FINAL + SEALED** — 2026-11-21, `task/W4-g63-proj-trail-dust-1`
(base main `a725acb04`). The lane's work landed pre-timeout at
`2ea3e711` (batch 7 + the 12-anniversary re-pick fix + the f20b-357
seam cross-note); this retire round consumes the e3 census correction,
re-verifies the activation checklist against the moved tips, executes
the registry-150 67→199 rename, and closes the lane. Code-only: no
cargo/tests/rustfmt/graphify; all 50 witnesses WRITTEN NOT RUN (the
merge-verify wave executes them). No further g63 commits are planned —
merge execution belongs to the merge wave per research/199.

## 1. The declaration — what the g63 family IS

The **projectile per-kind TRAIL/ambient dust family, kinds 1-400** (the
`Dust.NewDust*` sites inside `Projectile.AI()` `:23199-36807` and
`AI_001` `:54002-57825`, NOT `Kill()`, NOT the draw path, NOT the
site-batch lanes). One module: `crates/tr-app/src/projectile_trail_dust.rs`
(3444 lines, 50 `#[test]` draw-mirror witnesses), fed through the
`update`/`update_dust` scratch-pool seam from `world_sim/projectiles.rs`
on the live `UpdateProjectiles` stream (the `Npc::update_rng` precedent).

| slice | kinds | state at seal |
| --- | --- | --- |
| batches 1-4 (AI() inline) | the rowed styles 2/4/5/8/11/12/14/21/22/23/24/27/28/30/36/49/50/51/52/59/60/65/66/68 kinds ≤400 | RE-ARMED end-to-end, WIRED (19 witnesses); the no-dust rowed kinds censused closed |
| batch 5 (AI_001 slices) | 41/120/207/265/270/282/325/337/350/355 + the 357 emitter | WIRED via the `step_dust` slice variants (11 witnesses); 357 UNWIRED (f20 slice D) |
| batch 6 (unrowed AI()) | 12/87/93/118/119/128/129/177/196/229/261/291/298/305/307/335/358 | emitters UNWIRED (14 witnesses); at retire 12/87/298/305/358 + NEWLY 128/129/229 have rows on unmerged tips → research/199 activation rows |
| batch 7 (owed AI_001) | 55/174/176/299/374/376 | emitters UNWIRED (6 witnesses); 55→d22, 299→c3 activate at merge; 174/176/374/376 wait on a row |
| 281 bunny | `bunny_burst` | UNWIRED — needs the row (unclaimed) + d5's merge + the tick-3 call (research/199 §281) |
| 267/275/276 | — | censused CLOSED (rowed, C# arm draws no dust ≤400) |

The claims/census record: **research/66** (number KEPT — registry-150
§1 winner, first-add `34bf98b39` 00:30:45; the g64 fork renumbered
→196 on its own branch, landed `9d9d89ac1`). The activation checklist:
**research/199** (was 67; the registry-150 §3 rename EXECUTED this
round — d54's `67-dust-orch-sites-claims.md` wins the 67 key).

## 2. The family boundaries (the co-lane contract, verified at the seal)

| lane | owns | tip at seal | boundary verdict |
| --- | --- | --- | --- |
| **g63 (this)** | per-kind TRAIL dust, kinds ≤400 | this branch | the declaration above |
| g64 `proj-trail-dust-2` | per-kind TRAIL dust, kinds ≥401 | `9d9d89ac1` (RETIRE FINAL, research/196) | disjoint by kind; the shared aiStyle-5/12/60 blocks split at 400 (9/92 vs 955; 22/280/288 vs the >400 streams; 358 vs 406/1042) |
| g65 `proj-death-dust` | `Kill()` death dust, all kinds | `b0842907c` (final completeness accounting) | disjoint by C# region (`:70642-77262`); the ONE overlap found: the 358/406/1042 gun Kill smoke puffs ported by BOTH g65 b18 and d6 residual 8 — §3 keep-one |
| e3 `dust-sites-proj-2` | the AI-inline-40 window + the AI_001 >400 edges + the coverage rows | `ac6392cf2` (research/125→231 owed per registry) | ported the doc-66 census-ERROR's two sites (`:29245/29284` = 1042's, §3); its 955 star block carries the anniversary re-pick error — §3 |
| d53 / f5 | the per-file dust SITE batches | `6d09fe61` / `55dd8f9f` | site-order census lanes; the per-kind arms are g63/g64's (the claim doc's scope rule) |
| g16 `gore-projectile-sites` | the gore BODIES at the AI sites | `f1c60ffe9` | g63 consumes the gore ARGUMENT draws, g16 the bodies — the 12/955 + 162/281 one-consumer reconciliations are the merger's (research/199) |
| d54/g11/g12/g13/e1/e4/f6/g14/g15 | Main/Player/World/WorldGen/Item dust sites | (the dust-site family, d54's research/131 merge order) | disjoint by source file — no Projectile.AI()-region claim |
| g61 `npc-spawn-dust` | NPC spawn/teleport-in dust | SEALED research/238 | disjoint by entity — the sibling spawn-family bible; this doc is its projectile-trail counterpart |

## 3. The corrections + keep-one contracts (the merger's inputs)

1. **The doc-66 census-ERROR — CONSUMED + FIXED this round.** The
   e3 round-2 census (research/125, its `8180b0d03`) found doc-66's
   "60 | 358" row listing `:29245/29284` under the 358: WRONG — the
   C# `type != 1042` return at `:29234` precedes them (re-read at the
   seal, `Projectile.cs:29234-29237` verbatim). They are the **1042
   ShimmerGunStream's**; `:29205/29229` are the **406**'s
   (`:29197-29233`). The 358's own block is `:29158-29196`
   (`:29167/29191` only) — exactly what the g63 `water_gun_stream`
   emitter ports (`projectile_trail_dust.rs:2134`), so NO port
   collision existed; only the census row was wrong. Doc-66's row is
   corrected; the e3 `style60_shimmer_gun_trail_1042` (STAGED, d6's
   `projectile_ai_guns` activates) owns the two sites.
2. **The 12/955 anniversary RE-PICK — three copies, one correct
   census.** The C# (`:23975-23978`) draws TWO `Next(4)` on
   anniversary worlds (the second SelectRandom wins). g63's emitter
   (fixed `036901a5e`) + g16's `falling_star_trail`
   (`projectile_gore_a.rs:199`, unchanged at `f1c60ffe9`) BOTH draw
   two. **e3's `style5_star_dust` (its 955 arm) draws ONE** — the
   same error g63 carried pre-fix. At the merge the surviving arm
   must take the two-draw form or 955 desyncs the stream on
   anniversary worlds. One consumer for the shared `:23973-23981`
   gore draws (research/199's 12 row).
3. **The gun Kill smoke puffs — DOUBLE-PORTED.** d6 residual 8
   (`1b54dcbbd`, `projectile_gun_splash.rs`) claims g65's kind list
   skips the guns; g65 batch 18 (`1bbeec5b5`) ported them anyway
   (`projectile_death_dust.rs` `smoke_puff`, the 358/406/1042 arms at
   `:2786`). Both faithful, both draw the same census — keep ONE at
   the d6+g65 merge (research/199's 358 row). g65's claim note is
   stale; flagged for its merger, not edited here (own-branch rule).
4. **The 357 f20b seam — STILL VALID at f20 tip `31386266`.** Slice D
   (`4bd2f496`) unchanged underneath F4/F6/F7/F3/F5; F7 widened
   `step`/`finish` to `(p, world, npcs, immune)` + the RNG-free
   `charm_tail` after the flag8 clamp. The emitter call seats BEFORE
   the fade-25, the charm tail AFTER — C# order preserved, no stream
   collision. The `step_dust` variant threads the F7 signature
   (research/199's 357 row, refreshed).
5. **The three newly-rowed kinds (retire re-probe).** 128→g8
   `0a4b76e3f` (no-draw skip → thread + call), 129→g5 `96f074959`
   (census block REPLACED — same 6×[Next(4)+Next(5)] draws), 229→g2
   `ee857a368` (skip-with-draws → the emitter re-arms both). Full
   call sites + seams: research/199's new rows.
6. **The 281 dead-twin gore** (g16 `36e03444` wired a 281 landing gore
   onto the DEAD aiStyle-2 twin — 281 rides style 49 per `:3680`):
   re-point at d5's tick-3 block or drop (research/199 §281,
   unchanged at the seal — d5's module byte-identical, no dispatch).

## 4. The residual debt (what the family still owes AFTER the merges)

- **14 unrowed kinds** (93/118/119/174/176/177/196/261/281/291/307/
  335/374/376): emitters written + witnessed, waiting on rows — the
  all-tip literal + const sweep at the seal found none. Any d-lane
  follow-up rows them; the wiring is the one-line kind-gated call
  each emitter's doc comment pins.
- **The `old_position` latch widening** (174/176): the port's latch
  is golf-gated (`projectile.rs:20308`, styles 149/150); the C#
  `:17013` latches EVERY projectile. The activating merge owns the
  widening (the g64 research/196 cross-note agrees — changes every
  kind's `old_pos` reads, the merge wave re-derives).
- **The 955 anniversary fix** (§3.2) rides e3's merge, not g63's.
- **The stream-parity verification**: the 50 witnesses exercise the
  emitter fns directly (pass unconditionally); the LIVE wiring's
  stream parity needs the merge-verify wave (`cargo test -p tr-app
  projectile_trail_dust` after each activating merge, research/199).

## 5. The seal verification (code-only, this round)

- `git fetch origin --prune` + per-tip `git ls-tree docs/research/`
  over all 358 `task/W4-*` refs (the number scan, §header) — direct
  paths only, no junction recursion.
- Every research/199 unlock tip re-probed: `merge-base --is-ancestor`
  (all cited commits ancestors of their tips) + the
  post-cited-commit diffs read for the unlock arms (d15
  `ai_falling_star` / d2 `light_pet_step` / d6 `ai_heal_orb` +
  `gun_stream` / d22 `ai_hornet_stinger` / c3 SHADOWFLAMES / d5
  style-49 / f20 `projectile_ai001_d` — verdicts in research/199's
  rows + Verification section).
- The C# re-reads: `:29150-29300` (the gun blocks + the `:29234`
  return), `:23960-23990` (the anniversary re-pick), `:26638-26647`
  (the 129 draws), `:27922-27941` (the 229 arm) — ≤100-line windows,
  direct file reads.
- The all-tip row sweep: the literal `NNN => (` pattern + the const
  names over every pushed tip's `projectile/data.rs` (the 14 unrowed
  + the 3 promotions, §3.5).
- The 67→199 rename executed (`git mv` + every own-branch ref
  updated: doc-66 ×6, the emitter comments ×2); doc-66's census row
  corrected; the f20b cross-note refreshed.
- No cargo/rustc/rustfmt/tests/graphify invoked. No other branch
  touched; publication on `task/W4-g63-proj-trail-dust-1` only.
