# 298 — W4-g63 projectile TRAIL dust (kinds ≤400): FINAL + the spawn-dust-family declaration (SEALED)

> **NUMBER CHAIN: 240 → 281 (unpushed draft) → 298 (FINAL, EXECUTED).**

(Number: **240→298 RE-RENUMBER EXECUTED per registry-150 ROUND-3-FINAL
§7.3** — the d54 registry at `51a28d09e`, origin tip
`task/W4-d54-dust-sites-world`, §7 IS AUTHORITY. Chain: the original
seal-scan took 240 as "first free ≥237" (237=e23, 238/239=d42
CONFIRMED); the ROUND-2 refresh (`72226e500`) adjudicated
240 = **g58** `240-happiness-final.md` (first-add/self-rename
`cdfe3d108` 12:16:37, 240-243 CONFIRMED g58) and an interrupted
spawn-dust-seal round drafted a fresh-sweep 240→281 renumber — that
draft was NEVER pushed. ROUND-3-FINAL then assigned the 281 key to
f26's `68-findframe-census-addendum-7h.md` (§7.1: f26 7h/7i/7j →
281/282/283) and §7.3's g63 row EXPLICITLY assigns this doc
`240-proj-trail-dust-family-final.md` → **298** ("240 belongs to
g58's earlier rename (12:16:37); g63's 66 win + 199 EXECUTED stand").
This doc takes **298** per that explicit row — no fresh-sweep pick.
The d53/c42 losers of the same 240 key keep 275/280.)

**STATUS: FINAL + SEALED** — 2026-11-21, `task/W4-g63-proj-trail-dust-1`
(base main `a725acb04`). The lane's work landed pre-timeout at
`2ea3e711` (batch 7 + the 12-anniversary re-pick fix + the f20b-357
seam cross-note); the retire round consumed the e3 census correction,
re-verified the activation checklist against the moved tips, executed
the registry-150 67→199 rename. THIS SPAWN-DUST SEAL round (post-seal)
executes the registry-150-ROUND-3-FINAL §7.3 **240→298 renumber**
(g58 wins 240; the unpushed round-2 281 draft superseded — §header
chain) and upgrades §3.2/§3.3 from flags to EXACT published hunks
(the e3 two-draw fix-hunk; the gun Kill keep-one drop-hunks — d6's
`projectile_gun_splash` SURVIVES, g65's `smoke_puff` drops) + re-probes
the 128/129/229 activation rows at the current g8/g5/g2 tips (§3.5).
Code-only: no cargo/tests/rustfmt/graphify; all 50 witnesses WRITTEN
NOT RUN (the merge-verify wave executes them). No further g63 commits
are planned — merge execution belongs to the merge wave per
research/199.

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
| g65 `proj-death-dust` | `Kill()` death dust, all kinds | `f6e07bb90` (the 26i keep-one receipt) | disjoint by C# region (`:70642-77262`); the ONE overlap found: the 358/406/1042 gun Kill smoke puffs ported by BOTH g65 b18 and d6 residual 8 — §3.3 keep-one, EXACT drop-hunks published |
| e3 `dust-sites-proj-2` | the AI-inline-40 window + the AI_001 >400 edges + the coverage rows | `143b8cb67` (AI-inline RETIRE completion; research/125→231 owed per registry) | ported the doc-66 census-ERROR's two sites (`:29245/29284` = 1042's, §3.1); its 955 star block STILL carries the anniversary re-pick error at this tip — §3.2 EXACT fix-hunk published |
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
   census. EXACT FIX-HUNK PUBLISHED (this round).** The C#
   (`Projectile.cs:23973-23981`, re-read verbatim at the seal-refresh)
   draws TWO `Next(4)` on anniversary worlds:
   `:23975` `Utils.SelectRandom<int>(Main.rand, 16, 17, 17, 17)` then
   `:23976-23978` `if (Main.tenthAnniversaryWorld) num47 =
   Utils.SelectRandom<int>(Main.rand, 16, 16, 16, 17)` — the second
   draw WINS the gore type, but BOTH stay on the `Main.rand` stream.
   g63's emitter (`natural_falling_star_sparkle`, fixed `036901a5e`,
   `projectile_trail_dust.rs:1767-1772`) + g16's `falling_star_trail`
   (`projectile_gore_a.rs:199`, unchanged at `f1c60ffe9`) BOTH draw
   two. **e3's `style5_star_dust` STILL draws ONE at tip `143b8cb67`**
   (`crates/tr-app/src/projectile_dust.rs:597-599` — re-read this
   round; the same error g63 carried pre-fix). The e3 lane (or the
   merger) applies this exact hunk:

   ```diff
   --- crates/tr-app/src/projectile_dust.rs (e3 tip 143b8cb67, :594-599)
        // `:23973-23981` — the gore gate: the on-screen test short-circuits the
        // Next(6); the taken branch's SelectRandom draws Next(4). Both go with
        // the gore (the gore lane's skip), but the GATE draws stay on the stream.
        if on_screen_wide(p, d.ctx) && d.rng.next_range(0, 6) == 0 {
   -        let _ = d.rng.next_range(0, 4); // SelectRandom's pick (gore skipped)
   +        let _ = d.rng.next_range(0, 4); // SelectRandom(16,17,17,17)
   +        if anniversary {
   +            // `:23976-23978` — the anniversary RE-PICK: the SECOND
   +            // SelectRandom(16,16,16,17) draws too; its value wins the gore
   +            // type, but BOTH Next(4) draws stay on the stream (the g63
   +            // natural_falling_star_sparkle / g16 falling_star_trail pin).
   +            let _ = d.rng.next_range(0, 4); // SelectRandom(16,16,16,17)
   +        }
        }
   ```

   The e3 witness `w4_e3_b3_star_dust_anniversary_second_operand`
   (`:3126-3128`) mirrors the ONE-draw form and must take the same
   fix or it desyncs its own mirror on the anniversary loop:

   ```diff
   --- crates/tr-app/src/projectile_dust.rs (e3 tip 143b8cb67, :3126-3128)
                    if on_screen_wide(&p, &c) && mirror.next_range(0, 6) == 0 {
   -                    let _ = mirror.next_range(0, 4);
   +                    let _ = mirror.next_range(0, 4); // SelectRandom(16,17,17,17)
   +                    if anniversary {
   +                        let _ = mirror.next_range(0, 4); // the :23978 re-pick
   +                    }
                    }
   ```

   Without the fix, 955 desyncs the stream on anniversary worlds
   (one missing `Next(4)` per taken gore gate). One consumer for the
   shared `:23973-23981` gore draws at the three-way merge
   (research/199's 12 row).
3. **The gun Kill smoke puffs — DOUBLE-PORTED. KEEP-ONE ADJUDICATED:
   d6 SURVIVES, g65 DROPS. EXACT DROP-HUNKS PUBLISHED (this round).**
   d6 residual 8 (`1b54dcbbd`, tip `7edfcd495`,
   `projectile_gun_splash.rs`) vs g65 batch 18 (`1bbeec5b5`, tip
   `f6e07bb90`, `projectile_death_dust.rs` `smoke_puff` + the
   358/406/1042 arms at `:2786-2794`). Both port
   `Projectile.cs:74029-74147` with the same per-dust draw census
   (3×`Next(2)` alpha gates → `Next(2)` scale/noGravity →
   `Next(-100,101)` → 2×`Next(-50,51)` → the 1042's trailing
   `Next(6)` palette). **d6's copy SURVIVES** — it is the strictly
   more faithful port on two counts re-verified against the C# this
   round:
   - **The velocity base.** C# `:74031`/`:74067`/`:74101` reads
     `base.oldVelocity`; the style-60 scale death (`:28999-29000`)
     does `velocity *= 5; oldVelocity = velocity` BEFORE `Kill()`.
     d6's death hook captures the true `old_velocity` latch and
     selects the kicked live velocity on `scale <= 0`
     (`d6 projectile.rs:20545`: `let base = if p.scale <= 0.0 {
     p.vel } else { old_velocity };`). g65's `DeathSnapshot::of`
     sets `old_vel: p.vel` — its own field doc calls it a STAND-IN
     ("Kinds that need the true last-sub-step velocity are deferred",
     `g65 projectile_death_dust.rs:437-442`) — WRONG for moving guns.
   - **The `p.vel` write.** C# writes `base.velocity = oldVelocity *
     0.2` inside Kill(); d6's hook performs it (`p.vel = base * 0.2`,
     `:20546`); g65's `smoke_puff` only computes a local `new_vel`
     (`:4855`) and never writes the corpse.
   d6 also owns the aiStyle-60 machine (`projectile_ai_guns`) — the
   death path's natural home. g65's claim note ("kind list skips the
   guns") is stale; its 26i receipt does not re-claim them. The
   merger applies these exact drops at the d6+g65 merge (line
   numbers at g65 tip `f6e07bb90`):

   ```diff
   --- crates/tr-app/src/projectile_death_dust.rs (:2786-2794) — the three match arms DROP
   -        // `:74029-74062` — the 358: the smoke puff, 50× dust-211 alpha 75
   -        // scale 1.2, small 0.6, vel += newVel.
   -        358 => smoke_puff(s, dust, rng, ctx, world, 50, 211, 75, Color::default(), 1.2, 0.6, 1.0, false),
   -        // `:74063-74098` — the 406: the smoke puff, 40× dust-4 alpha 175
   -        // (0,80,255,100) scale 1.6, small 0.6.
   -        406 => smoke_puff(s, dust, rng, ctx, world, 40, 4, 175, Color::new(0, 80, 255, 100), 1.6, 0.6, 1.0, false),
   -        // `:74099-74147` — the 1042: the smoke puff, 50× dust-324 alpha 75
   -        // scale 1.25, small 0.8, vel += newVel×0.8, + the Next(6) colour.
   -        1042 => smoke_puff(s, dust, rng, ctx, world, 50, 324, 75, Color::default(), 1.25, 0.8, 0.8, true),
   +        // `:74029-74147` — the 358/406/1042 gun Kill smoke puffs are
   +        // W4-d6 residual 8's (projectile_gun_splash.rs, the keep-one
   +        // survivor: the true oldVelocity latch + the scale-death ×5
   +        // base + the corpse p.vel write; research/298 §3.3). No arm
   +        // here — the guns fall through to the default (no draws).
   ```

   ```diff
   --- crates/tr-app/src/projectile_death_dust.rs (:4834-4885) — the helper DROPs
   -/// `Kill()`'s 358/406/1042 smoke puff (`:74029-74147`): ...
   -fn smoke_puff( ... ) { ... }   // the whole fn, doc comment included —
   -                               // its ONLY callers were the three dropped arms
   ```

   ```diff
   --- crates/tr-app/src/projectile_death_dust.rs (:6616-6634) — the witness DROPs
   -    /// The 358/406/1042 smoke puffs: 50/40/50 dust at the corpse centre, ...
   -    #[test]
   -    fn w4_g65_the_smoke_puffs() { ... }   // d6's three gun-splash witnesses
   -                                          // cover the surviving census
   ```

   d6 keeps EVERYTHING: `projectile_gun_splash.rs` (module + 3
   witnesses), the `pending_gun_splashes` queue
   (`d6 projectile.rs:14752-14757`), `drain_gun_splashes`
   (`:15289-15294`), the death-hook snapshot + velocity write
   (`:20525-20556`), the pass-tail drain
   (`d6 world_sim/projectiles.rs:680-696`), `main.rs:187`. g65's
   generic `DeathSnapshot` hook keeps snapshotting the gun kinds
   (inert — no arm consumes them after the drop); no other g65 arm
   reads 358/406/1042. One survivor, never both — a double-feed
   desyncs the stream (research/199's 358 row, refreshed).
4. **The 357 f20b seam — STILL VALID at f20 tip `31386266`.** Slice D
   (`4bd2f496`) unchanged underneath F4/F6/F7/F3/F5; F7 widened
   `step`/`finish` to `(p, world, npcs, immune)` + the RNG-free
   `charm_tail` after the flag8 clamp. The emitter call seats BEFORE
   the fade-25, the charm tail AFTER — C# order preserved, no stream
   collision. The `step_dust` variant threads the F7 signature
   (research/199's 357 row, refreshed).
5. **The three newly-rowed kinds — RE-PROBED CURRENT (this round).**
   128→g8 `0a4b76e3f` (the `FROST_BLAST_HOSTILE` proj_data row,
   `data.rs:876`/`:3226` aiStyle 28 — no-draw skip → thread + call),
   129→g5 `96f074959` (the `RUNE_BLAST` row `data.rs:932` + the
   `:26638-26646` census block at `projectile.rs:7957-7962` —
   REPLACED, same 6×[`Next(4)`+`Next(5)`] draws), 229→g2 `ee857a368`
   (the `CHLOROPHYTE_ORB` row `data.rs:917`/`:4002` aiStyle 44 + the
   `ai_spear_subshot_44` 229 arm, `projectile_ai_spear.rs:579-602`
   "skipped with the draws" — the emitter re-arms both). All three
   cited commits are ancestors of their CURRENT pushed tips
   (`merge-base --is-ancestor` exit 0: g8 `1752c70df`, g5
   `96f074959` = tip, g2 `81c4ab85a`); the row/arm sites re-grepped
   at those tips. The g63 emitters activate at those merges — full
   call sites + seams: research/199's rows.
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

## 6. The spawn-dust seal round (this round — the 240→298 renumber + the exact hunks)

- The d54 registry re-read: `origin/task/W4-d54-dust-sites-world`
  tip `51a28d09e` — **ROUND-3-FINAL, §7 IS AUTHORITY**. §7.1's 240 row:
  g58 wins (round 2, rename 12:16:37), g63's
  `240-proj-trail-dust-family-final.md` (`1c52d5ee3` 14:54:20) → **298**;
  §7.3's per-lane g63 row confirms the exact old/new filenames + ref
  file (`docs/research/INDEX.md`). The interrupted round-2 draft
  (fresh-sweep 281 pick) was NEVER pushed; round 3 assigned 281 to
  f26's 7h addendum, so the draft number was superseded before
  publication — the executed chain is 240→281(draft)→**298**.
- The rename: `git mv 240-… → 298-…` + all refs updated
  (ANALYSIS ×2, PORT_MAP ×1, research/199 ×4, doc-66 ×2, INDEX ×1,
  this doc's §3.3 hunk comment ×1); `git grep` zero stale old-name /
  old-number refs on this branch afterwards.
- The e3 re-read at tip `143b8cb67`: `projectile_dust.rs:582-617`
  (`style5_star_dust` — the ONE-draw gore gate confirmed still
  present) + `:3109-3142` (the mirror witness) → §3.2's exact
  two-draw fix-hunk + witness hunk published for the e3 lane/merger.
- The gun Kill keep-one re-read: C# `:74029-74084` + `:28994-29002`
  (the oldVelocity read + the scale-death ×5/oldVelocity overwrite);
  d6 tip `7edfcd495` (`projectile_gun_splash.rs` full read,
  `projectile.rs:20525-20556` hook, `world_sim/projectiles.rs:680-
  696` drain) vs g65 tip `f6e07bb90` (`smoke_puff :4840-4885`, arms
  `:2786-2794`, `DeathSnapshot::of :489-512` — the `old_vel: p.vel`
  stand-in) → §3.3's adjudication: **d6 SURVIVES** (the true latch
  + the corpse velocity write), g65's three arms + helper + witness
  DROP (exact hunks published).
- The 128/129/229 rows re-probed at the current g8/g5/g2 tips
  (§3.5 — ancestry + row/arm greps, all CURRENT).
- C# windows ≤100 lines, direct paths only (no junction recursion);
  no cargo/tests/rustfmt/graphify; edit-tool-only file changes.
