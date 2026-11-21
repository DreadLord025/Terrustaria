# 150 — Research-number COLLISION REGISTRY (W4-d54 doc-integrity, code-only)

Scan date: **2026-11-21** (worktree TR-W4-d54, branch `task/W4-d54-dust-sites-world`).
Method: fetched **all 355 `task/W4-*` remote tips**, `git ls-tree docs/research/`
per tip (20 071 file rows), grouped by leading number key (`NN` or `NNx`),
blob-identity checked, and first-add commit per filename per branch via
`git log --diff-filter=A --format=%H %cI --name-only`. Timestamps below are
**committer dates (%cI, +02:00)** of the doc's first add on the branch — the
first-push proxy; ties broken by lane name.

**Result (round 1, commit `38bfd1d22`): the 7-collision brief undercounts. The
full scan found 32 colliding number keys** (26 bare numbers 08, 61–71, 90–95,
97, 101, 109, 125, 133–136 plus 6 letter-suffixed keys 12g, 12i, 12j, 26d, 26e,
26f), **86 rename assignments (81 distinct losing docs + 5 same-path fork
copies) across 64 lanes**. Free renumber block assigned: **151–236, sequential**
(150 = this registry; max number seen at any tip then was 144).

> ## 🔄🔄🔄🔄 §8 (round 4, APPEND-ONLY) — READ THIS FIRST — §8 IS AUTHORITY for 244–248/281–284
>
> Round 4 adjudicates the **post-§7.4 SELF-PICK wave on the assigned keys**
> (§8, lean claimant-tip re-check, NO full sweep — §7.2/§7.4 assignments are
> the authority). The 281–303 block was ASSIGNED by §7.2/§7.3 to specific
> lanes; **self-pickers who took 281–303 WITHOUT assignment COLLIDE and
> renumber 308+**. Winners of the contested keys: **281/282/283 = f26
> 7h/7i/7j (KEEP their §7.3 assignments)**, **284 = g15**, **244/245 = g48**
> (pre-publish WINS stand), **246 = e23 reservation**, **247 = d10 reservation
> (EXECUTED `72431e61f`)**, **248 = g82's published round-2 reservation**
> (e26's 15:44:26 direct add is POST-publish → loses, §7.4 rule; brief's
> "e26-KEEPS?" verified NO). g48's executed 244→281 / 245→282 self-renames
> and f39's 244→281 rename are CONTESTED → NOT honored → **308/309** and
> **312** (f39 chain 244→281→312, supersedes its unexecuted 299 row → 299
> FREE); f4's 240→283 rename was contested but f4 then self-corrected to
> **305 EXECUTED** (§7.4 row satisfied). **26 numbers 308–333 (25 assigned
> + 323 RESERVED g65 per dispatch, claim unverified at tip); LIVE total
> 173 = 151–236 (86) + 246–280 (35) + 281–303 (22, 299 freed) + 304–307 (4)
> + 308–333 (26). Numbers 334+ are FREE.**

> ## 🔄🔄🔄 §4-REFRESH (round 3 EXTENSION, same scan date) — §7.4 authority extended by §8 where noted
>
> Round 3 landed as `51a28d09e` (15:35:39). A lean post-cutoff re-check of the
> ~12 claimant tips for keys **240/243/244/246/247** (§7.4) finds FIVE claimants
> the round-3 sweep missed or that landed after it: **243 +d50** (14:59:24,
> g58 wins → d50 **304**), **240 +f4** (15:00:53, g58 wins → f4 **305**),
> **244 +d53's EXECUTED 240→244 rename** (`a106d13e8` 14:56:57, contested by
> g48's earlier 244 → d53 re-issued its round-2 **275**, old name now
> `244-proj-dust-retire.md`) **+g86** (15:05:34 → **306**), **246 +g43**
> (15:10:56, e23's published reservation wins → **307**), **247 +g67's
> EXECUTED 146→247 rename** (`8387af9ac` 15:13:18, contested by d10's
> published round-2 247 reservation → g67 re-issued its round-2 **267**, chain
> 134→146→247→267). Executed self-renames are honored ONLY if uncontested —
> both of these were contested. **4 NEW assignments 304–307; live total 148 =
> 151–236 + 246–280 + 281–303 + 304–307 (§5 machine check). Numbers 308+ are
> FREE.** §7.4 supersedes §7.1–§7.3 wherever they disagree.
>
> ## 🔄🔄 §4-REFRESH (round 3, commit `51a28d09e`) — extended by §7.4 where noted
>
> Round 2 landed as commit `72226e500` (14:47:50). A round-3 re-scan of **all
> 360 `task/W4-*` remote tips** (21 172 ls-tree rows; two new lanes:
> `W4-f12-buff-effects` base-carry only, `W4-g11-teleport-dust` = g11 successor
> branch) finds **53 colliding keys** (45 bare + 8 letter-suffixed incl. the
> 18b fork family). **§7 supersedes §6 wherever they disagree.** Highlights:
>
> - **Round-1/2 WINNER CORRECTIONS (missed earlier claimants, §7.1):**
>   **125 = d125** empress-sheets-ledger (06:07:29, missed in round 1) — d38's
>   round-1 win is REVOKED → **293**; **144 = e19** splash-keep-one (08:45:32,
>   missed in round 2) — c4's round-2 win REVOKED → **294**; **145 = e19**
>   16box-cross-lane-repins (09:32:12, missed in round 2) — g89's round-2 win
>   REVOKED → **295**. All other round-1/2 winner calls re-verified unchanged.
> - **Executed-before-publish pushes beat round-2 draft reservations** (the
>   round-2 principle, applied at its 14:47:50 publish cutoff): **244 = g48**
>   c28-18-send-switch + **245 = g48** 39-item-ownership-seed (both 14:30:16,
>   pre-publish) — c31's 244 reservation → **300**, d81's 245 reservation →
>   **301** (g53 rides 301). d10's 246 take (14:52:30) is POST-publish → loses
>   to e23's published 246 reservation → **247** (round-2 assignment stands).
> - **§7 assigns 23 losers → 281–303** + re-issues 2 round-2 rows with updated
>   old filenames (g48 237→239→**272**, d10 26d→246→**247**). New collisions:
>   77 (g78 wins, g15 →284), 91 ×7 new losers (d22/g4 fighter-family wave →
>   285–291), 108 +g8 (→292), 68 +f26 7h/7i/7j (→281/282/283), 65 +f3 (→303),
>   146/148 +e19 (→296/297), 240 +g63 (→298), 244 +f39 (→299), 18b d83 fork
>   DIVERGED from d82 (→302). **116** = same-lane successor fork
>   (g11-dust-main → g11-teleport-dust, merge-base = dust-main tip): NO
>   renumber, merger keep-one the SEAL blob (§7.1).
> - **Executions CONFIRMED at tips:** d10 165/166, g63 199, g64 196, f11
>   233/234/235. **237** now single-claimant (e23; g48 renamed away to 239).
>   **26i** (g65) single-claimant FREE. f45's 137→279 still owed.
> - **Live assignment total: 144 = 151–236 (86) + 246–280 (35) + 281–303 (23),
>   machine-checked unique, no gaps within each block (§5 round 3).**
>   Winner-consumed: **237–245** (e23/d42×2/g58×4/g48×2). ~~Numbers 304+ are
>   FREE.~~ **⚠ §7.4 consumed 304–307; 308+ FREE.** Do NOT pick max-on-origin+1; check this registry AND a fresh
>   all-tip sweep first.
> - **Claim-time convention (pinned §7):** a directly-added doc contests its
>   number at first-add `%cI`; a SELF-RENAMED doc contests the NEW number at
>   its rename-commit `%cI` (not its original content birth). This is the
>   convention round 2 applied (g24 146 / e20 149 / d42 238 / g58 240) and
>   round 3 keeps — it prevents ancient-content squatting on fresh numbers.
>
> ## 🔄 §4-REFRESH (round 2, commit `72226e500`, six fetches) — superseded by §7 where noted
>
> Re-scanned **all 358 `task/W4-*` remote tips** (`git ls-tree docs/research/`
> per tip, **21 038 rows** at the final sweep). The lane wave since round 1
> landed new collisions and several lanes self-renamed mid-flight —
> including INTO each other (c8/d42/g61 all picked 238; d53/c42 both picked
> 240). Current state:
>
> - **47 colliding number keys** at the tips (40 bare + 7 letter-suffixed:
>   12g, 12i, 12j, 12k, 26d, 26e, 26f).
>   New since round 1: **12k, 96, 108, 111, 129, 130, 137, 143, 144, 145,
>   146, 147, 148, 149, 237, 238, 239, 240**; extended: **26d, 26e, 64, 68**
>   (key 70 gained a same-path fork). New same-path fork families (single
>   filename, diverged blobs): **18b, 83, 84, 122** (+ the key-70 fork).
>   Resolved by self-rename: **134/135/136** (f11 executed 134→151→**233**,
>   135→152→**234**, 136→153→**235**; interim 151/152/153 collided with this
>   registry's block, corrected at f11 `14ca0a8d5` — **CONFIRMED at tips**).
>   g64 executed round-1's 66-fork→**196** — CONFIRMED.
> - **Consumed free numbers at tips (winners, NOT assignments): 237 = e23
>   bestiary-ui-final (10:53:03), 238 = d42 roster-sheets (12:15:55), 239 =
>   d42 final-keep-one-and-band (12:55:30), 240/241/242/243 = g58
>   happiness-final (143 self-rename 12:16:37) / f37-latch-repoint-hunks /
>   chat-portrait-seed / seal-verification. Executed pushes beat unpublished
>   registry drafts.**
> - **§6 assigns 37 new losers → 244–280** (31 new losing docs + 6 new
>   same-path fork copies). Total assignments: **123 (151–236 + 244–280),
>   machine-checked unique, no gaps within each block**. **Numbers 281+ are
>   FREE.** 237–243 are taken; do not reuse them.
> - Self-renames reconciled (§6.2): g24 107→**146 KEEPS**; g67 134→146→**267**;
>   e20's c42-repick 146→**266**; c10 145→149→**270**; e23 **237 KEEPS**;
>   g89 139→**145 KEEPS**; c4 **143/144/148 KEEP**, 145-addendum→**265**;
>   e20 **149/147/71 KEEP**, 148→**269**; f11 →233/234/235 **CONFIRMED**;
>   **d42 143→238 EXECUTED and CONFIRMED (238+239 winner)**; **g58 143→240
>   EXECUTED and CONFIRMED (240/241/242/243 winner)**; **c8 143→238 EXECUTED
>   but 238 collides → 262**; **g61 237→238 EXECUTED but both collide →
>   273**; g48's fresh 237 → **272**; g56's fresh 239 → **274**; f21's fresh
>   239 (the round-1 brief's unpushed f21b doc!) → **276**; d53's fresh 240 →
>   **275**; c42's fresh 240 → **280**; f47's two fresh 144 docs →
>   **277/278**; f45's fresh 137 → **279**; g50's fresh 154 (registry-
>   reserved for c33) → **271**. Authoritative numbers are ONLY the §6
>   tables.
> - **150 present at origin tip**: `origin/task/W4-d54-dust-sites-world` =
>   `38bfd1d22` carried `docs/research/150-research-number-registry.md`
>   (blob `cf077631f`) at the refresh fetch — the g67 report of tip
>   `1d16ef4ea` WITHOUT 150 was a stale fetch on their side (`1d16ef4ea` is
>   3 commits behind `38bfd1d22` on the same branch); no re-publish from blob
>   needed. This refresh commit rides the same branch.
> - Round-1 tables (§1–§3) remain valid for every row NOT superseded by §6.
>   107 is now EMPTY (g24 renamed to 146); 139 stays RESERVED for
>   g52c-net-master (g89's doc self-renamed 139→145 at `41dabfbc8`, g50's
>   cross-note explicitly does not take 139); if g52c pushes a 139 doc it
>   keeps 139 (single claimant).
> - **Cutoff:** pushes after the final sweep (fetch 6) are not covered; see
>   the §6.3 cutoff note.

Rules:
- **First-pushed timestamp wins** the contested number. Losers rename on their
  OWN branch (this lane cannot touch other branches) per §3.
- Same filename on several branches is NOT a number collision: identical blobs
  merge cleanly (listed in §4); diverged blobs are same-path forks — merger
  keeps the later-timestamped copy under the number and the earlier copy moves
  to its assigned free number (§1 fork rows + §3).
- Letter-suffixed addenda (`26d-…`) collide with each other on the full key;
  a renumbered loser drops the letter and takes a bare free number.
- Renames are content-preserving: keep the doc body, update the `# NN` header
  to the new number, and update every file in the refs column (§3).

## 1. Per-number collision table (winner keeps, losers renumber)

Branch names abbreviated: `W4-x` = `task/W4-x`. ts = first-add committer date.

| Key | Winner (keeps number) | Winner doc / ts | Losing claimants → FREE number |
| --- | --- | --- | --- |
| 08 | base trunk (all pre-W4 branches) | `08-tile-framing.md` 2026-11-12T12:23:05 `28bdd09fa` | W4-d113 `08-d113-tileobjectdata-third.md` 11-20T19:00:15 → **151** |
| 12g | W4-f25 | `12g-f25-settle-liquids.md` 11-20T22:08:23 `fd0985287` | W4-g36 `12g-g36-worldgen-missing-a.md` 22:09:57 → **152**; W4-f25 (2nd doc, same lane) `12g-f25-worldgen-helpers.md` 11-21T03:55:47 → **153** |
| 12i | W4-g38 | `12i-g38-worldgen-missing-c.md` 11-20T22:18:34 `d32b13cc9` | W4-c33 `12i-c33-underwater-chests-oasis.md` 22:59:16 → **154** |
| 12j | W4-g39 | `12j-g39-worldgen-helpers.md` 11-20T22:30:10 `e84dde505` | W4-g42 `12j-g42-biome-mushroom.md` 11-21T01:15:00 → **155** |
| 26d | W4-d12 | `26d-projectile-ai-111-120-census.md` 11-21T00:06:06 `64a296c68` | W4-d5 41-50-census 00:24:13 → **156**; W4-g2 106-210-census 00:27:07 → **157**; W4-d14 131-140-census 00:44:16 → **158**; W4-d17 161-170-census 00:55:48 → **159**; W4-g1 kill-hooks-census 01:02:53 → **160**; W4-d4 31-40-census 01:41:25 → **161**; W4-d21 201-210-census 02:40:39 → **162**; W4-g2 rowless-kinds 04:01:40 → **163**; W4-d11 101-110-census 04:17:27 → **164**; W4-d10 91-100-crosschecks 08:36:27 → **165**; W4-d10 91-100-docsweep 08:46:30 → **166** |
| 26e | W4-d7 | `26e-projectile-ai-61-70-census.md` 11-21T01:30:11 `e96523733` | W4-f20 106-210-census 02:10:20 → **167**; W4-g3 kill-onhit-census 02:48:55 → **168**; W4-d4 rope-coil-keep-one 06:37:47 → **169** |
| 26f | W4-f20 | `26f-projectile-ai001-overlays-round2.md` 11-21T05:13:47 `5aa190e3c` | W4-d3 21-30-band-status 09:28:41 → **170** |
| 61 | base trunk | `61-weapon-data-census.md` 11-19T23:36:41 `a7d9d26af` | W4-d61 `61-drawdata-cases-1.md` 11-20T20:16:47 → **171** |
| 62 | base trunk | `62-ammo-potion-census.md` 11-19T23:44:29 `4132bc050` | W4-d62 `62-drawdata-cases-2.md` 11-20T20:02:30 → **172** |
| 63 | base trunk | `63-special-bullets-census.md` 11-20T02:25:29 `648633183` | W4-d63 `63-drawdata-cases-3.md` 11-20T19:39:02 → **173** |
| 64 | W4-d64 | `64-drawdata-cases-4.md` 11-20T18:54:02 `161c63898` | W4-e25 expert-master-scaling-census 18:56:14 → **174**; W4-d80 tracked-sound-census 20:48:20 → **175**; W4-f43 town-spawn-census 21:14:54 → **176**; W4-g12 dust-main-sites-claims 22:10:33 → **177**; W4-e1 dust-sites-player-2 11-21T01:20:41 → **178**; W4-g78 tile-echo-draw-census 02:57:05 → **179**; W4-e16 item-tooltip-line-census 03:08:01 → **180**; W4-g72 cave-bg-rng-shift-census 04:11:59 → **181**; W4-g72 v18-union-addendum 07:35:19 → **182** |
| 65 | W4-f17 | `65-kinds-coverage-1.md` 11-20T20:49:39 `17c3f9986` | W4-g14 dust-worldgen-sites-claims 11-21T00:15:05 → **183**; W4-f35 expert-drop-trees-census 00:35:44 → **184**; W4-g33 drawdata-wiring-lane1 00:39:12 → **185**; W4-d54 dust-world-sites-claims 00:43:15 → **186**; W4-g13 dust-main-sites-3 01:04:48 → **187**; W4-g11 dust-main-sites-g11 01:09:11 → **188**; W4-c19 addlight-proj-a 01:54:05 → **189**; W4-g15 dust-item-sites-claims 02:55:24 → **190**; FORK: W4-f6 copy of `65-dust-world-sites-claims.md` (first add 00:48:39, diverged blob) → **191** (d54 copy keeps 65 until d54 itself renames →186) |
| 66 | W4-g63 | `66-proj-trail-dust-claims.md` 11-21T00:30:45 `34bf98b39` | W4-e4 dust-world-sites-2-claims 00:32:48 → **192**; W4-g34 drawdata-unwired-second-half 01:31:54 → **193**; W4-d54 world-dust-reconcile 04:10:05 → **194**; W4-f35 bag-open-master-census 09:12:43 → **195**; FORK: W4-g64 copy of `66-proj-trail-dust-claims.md` (first add 01:53:25, diverged blob) → **196** |
| 67 | W4-d54 (own) | `67-dust-orch-sites-claims.md` 11-21T01:23:05 `48343a4c8` | W4-g34 cage-animator-reconcile 03:06:13 → **197**; W4-f6 dust-world-wiring-keep-one 07:02:43 → **198**; W4-g63 proj-trail-dust-activation 07:50:44 → **199** |
| 68 | W4-f37 | `68-mood-reconcile.md` 11-21T02:48:30 `908482d52` | W4-d44 findframe-wire-status 02:52:49 → **200**; W4-f26 addendum-7c 06:09:33 → **201**; W4-g13 dust-main-residual-sweep 06:27:34 → **202**; W4-f26 addendum-7e 07:41:46 → **203**; W4-g34 cage-critter-items-census 08:04:37 → **204**; W4-f26 addendum-7f 08:52:36 → **205** |
| 69 | W4-g46 | `69-liquid-reconcile.md` 11-21T02:58:06 `f3b33b928` | W4-g13 dust-second-chain-lights 08:01:44 → **206**; FORK: W4-g45 copy of `69-liquid-reconcile.md` (first add 08:13:20, diverged blob) → **207** |
| 70 | W4-d53 | `70-proj-dust-site-batches-claims.md` 11-21T03:49:21 `8a9cb90ba` | W4-f41 party-reconcile 04:02:54 → **208**; W4-g47 liquid-merge-order 05:56:14 → **209**; W4-g13 dust-draw-tint-final 08:56:52 → **210** |
| 71 | W4-e20 | `71-wiring-device-reconcile.md` 11-21T03:50:05 `8791f07fe` | W4-f4 npc-gore-wave-e-claims 05:08:03 → **211**; FORK: W4-c42 copy of `71-wiring-device-reconcile.md` (first add 05:35:32, diverged blob) → **212** |
| 90 | base trunk | `90-consolidated-findings.md` 11-12T14:52:25 `6ef2f3929` | W4-g22 doc-sweep-menu-sounds 11-21T07:07:16 → **213** |
| 91 | base trunk | `91-post-audit-landed.md` 11-12T16:39:09 `0ed7d39a4` | W4-g4 doc-sweep-fighter 11-21T05:27:41 → **214**; W4-g22 g23-refold-g22-doc-sweep 08:04:27 → **215**; W4-g4 addendum-d22r-fold-list 08:09:05 → **216**; W4-g22 g23-branch-deprecated 08:50:40 → **217** |
| 92 | base trunk | `92-roadmap-reverification.md` 11-12T16:39:09 `0ed7d39a4` | W4-g71 doc-sweep-bg 11-21T05:12:58 → **218** |
| 93 | W4-g41 | `93-spider-cave-census.md` 11-21T00:45:07 `e74d1da97` | W4-f33 torch-god-949-keep-one 06:14:53 → **219**; W4-d17 doc-sweep-melee-proj 07:46:57 → **220**; W4-f33 torch-god-fold-input-addendum 08:26:53 → **221** |
| 94 | W4-g41 | `94-not-the-bees-host-census.md` 11-21T03:24:48 `4317e2ba5` | W4-g40 w4g40-biome-residual-census 04:15:49 → **222**; W4-f2 npc-sound-reconcile 05:38:00 → **223** |
| 95 | W4-c42 | `95-wiring-master-reconcile.md` 11-21T04:18:15 `ec568b3ea` | W4-c29 gem-moss-census 05:10:30 → **224**; W4-g41 finish-no-traps-remix-census 05:25:31 → **225** |
| 97 | W4-d115 | `97-doc-sweep-dodge.md` 11-21T05:18:36 `7c4fcb755` | W4-d8 final-phase-proj-ai-71-80 08:44:51 → **226** |
| 101 | W4-g80 | `101-g80-task-seeds.md` 11-21T06:09:53 `7be2d8eed` | W4-g43 doc-sweep-temple 08:41:18 → **227** |
| 109 | W4-c9 | `109-snail-band-claims.md` 11-21T07:13:30 `2042d77f1` | W4-d20 doc-sweep-proj-191 08:03:04 → **228**; FORK: W4-d36x copy of `109-d36-npc-kinds-141-210-census.md` (same add commit `0eb4fee95` 08:20:07 as d36, diverged blob later; lane-name tiebreak d36 < d36x) → **229**; W4-d36's own copy is ALSO a loser vs c9 → **236** (see §4 correction row) |
| 125 | ~~W4-d38~~ **W4-d125 (§7.1 CORRECTION)** | ~~`125-d38-crossnote-atlas-addendum.md` 07:00:21~~ **`125-empress-sheets-ledger.md` 06:07:29 `2dfe026d2` — earlier, missed in round 1; d38 WIN REVOKED → 293** | W4-d39 d39-band-351-420-census 07:18:50 → **230**; W4-e3 proj-dust-coverage 07:42:27 → **231**; W4-d38 crossnote-atlas-addendum → **293** (§7.3) |
| 133 | W4-f11 | `133-f11-unique-folds.md` 11-21T05:53:19 `7f307d171` | W4-c42 fold-receipt 07:24:20 → **232** |
| 134 | W4-g67 | `134-g67-post-union-folds.md` 11-21T07:50:35 `30b39fceb` | W4-f11 light-family-unification 08:33:23 → **233** |
| 135 | W4-g90 | `135-g90-persist-folds.md` 11-21T06:39:48 `b3b82d071` | W4-f11 wires-radial 09:03:29 → **234** |
| 136 | W4-g31 | `136-tilepass-d-hunks.md` 11-21T06:47:18 `5b8899541` | W4-f11 grand-design-activation 09:21:26 → **235** |

**Free block actually consumed (round 1): 151–236 (86 unique assignments = 81
distinct losing docs + 5 same-path fork copies).** ~~Numbers 237+ stay FREE~~
**SUPERSEDED by the refresh: 237–243 consumed by winner pushes
(e23/d42/d42/g58×4); the assignment block now runs 151–236 + 244–280 (§6),
numbers 281+ are FREE.** 150 is this registry.

Doc titles of every loser are in §3 (header line quoted where present; two f35
docs use `//!` banner comments instead of a `#` header).

## 2. Known-collision brief reconciliation (7 reported vs. scan)

| Brief entry | Scan verdict |
| --- | --- |
| 107 f21b-net-apply × g24c-wg-sound | **ONE-SIDED at tips**: only W4-g24 carries `107-wg-sounds-final.md`; W4-f21 tip has NO 107/139-class doc (58 files, all base). f21b sub-round doc was never pushed. No live collision — 107 stays with g24. |
| 139 g52c-net-master × g89d-rod | **ONE-SIDED**: only W4-g89 carries `139-rod-teleport-union-composition.md`; W4-g52 tip has no such doc. 139 stays with g89. |
| 143 g87b-demonheart × c4d-d40-retire | **ONE-SIDED**: only W4-c4 carries `143-d40-crimsongoldfish-retire-hunk.md`; W4-g87 tip has none. 143 stays with c4. |
| 144 f47c-splash × c4d-orca | **ONE-SIDED**: only W4-c4 carries `144-orca-npc692-census-input.md`; W4-f47 tip has none (its only non-base doc is `121-doc-sweep-shimmer.md`). 144 stays with c4. |
| 134 g67-rain-folds × f11d-forced-state | **CONFIRMED** — key 134 row above; f11 `134-light-family-unification.md` → 233. |
| 135 g90c-persist × f11d-radial-draw | **CONFIRMED** — key 135 row; f11 `135-wires-radial.md` → 234. |
| 136 g31e-tilepass-dhunks × f11d-kite | **CONFIRMED** — key 136 row; f11 `136-grand-design-activation.md` → 235. |

The four one-sided entries are unpushed sub-round docs; if those lanes later
push them, they MUST take free numbers ~~237+~~ **281+ (refresh: 237–243
consumed by winner pushes; assignment block now ends at 280)**. The scan
found **29 additional colliding keys** the brief missed.

**REFRESH update (§6):** 143 became a FULL 6-way collision (c4 keeps; d42
self-renamed →238 and KEEPS 238; g58 self-renamed →240 and KEEPS 240; d120/g87
→ 261/263; c8's 143→238 rename lost 238 to d42 → 262); 144 became 3-way (c4
keeps; f22 → 264; f47's two mid-round docs → 277/278); the f21b doc FINALLY
pushed — on 239 (14:11:45), losing to d42 → 276; 107 and 139 are now EMPTY at
tips (g24 renamed 107→146 and KEEPS 146; g89 renamed 139→145 and KEEPS 145 —
139 stays RESERVED for g52c). f11's 134/135/136 losses are EXECUTED as
233/234/235 (via an interim 151/152/153 wave that this registry superseded).

## 3. Per-lane rename-INSTRUCTION table (lanes execute on OWN branches)

Each lane: `git mv` old → new under `docs/research/`, update the doc's number
header, then update EVERY listed ref file on the same branch (replace the old
filename string; refs were found by `git grep -l -F <oldname> <tip> -- docs/`,
excluding the doc itself). `(none)` = no external refs found at tip — rename
only the file + its own header. Do NOT touch this registry or other branches.

| Lane (branch `task/W4-…`) | old filename → new filename | doc title (first header) | refs-to-update on own branch |
| --- | --- | --- | --- |
| d113-tileobjectdata-3 | `08-d113-tileobjectdata-third.md` → `151-d113-tileobjectdata-third.md` | 08-d113 — TileObjectData third slice: TileID 240–359 census + witnesses | `docs/ANALYSIS.md` |
| g36-worldgen-missing-a | `12g-g36-worldgen-missing-a.md` → `152-g36-worldgen-missing-a.md` | 12g — W4-g36: worldgen missing-a (mop-up lane, first quarter) | (none) |
| f25-worldgen-passes-8 | `12g-f25-worldgen-helpers.md` → `153-f25-worldgen-helpers.md` | 12g (helpers) — W4-f25: the WorldGen helper census, round 2 | (none) |
| c33-underwater-chests-oasis | `12i-c33-underwater-chests-oasis.md` → `154-c33-underwater-chests-oasis.md` | 12i — W4-c33: UnderwaterChests + PyrX consumption + Oasis family remainder | (none) |
| g42-biome-mushroom | `12j-g42-biome-mushroom.md` → `155-g42-biome-mushroom.md` | 12j — W4-g42: the glowing mushroom biome lane (census + claim) | `docs/ANALYSIS.md`, `docs/PORT_MAP.md` |
| d5-proj-ai-41-50 | `26d-projectile-ai-41-50-census.md` → `156-projectile-ai-41-50-census.md` | 26d — Projectile aiStyles 41-50 dispatch census (W4-d5) | (none) |
| g2-proj-ai-missing-b | `26d-projectile-ai-106-210-census.md` → `157-projectile-ai-106-210-census.md` | 26d — aiStyles 106-210 COVERAGE census + the 211+ void (W4-g2) | (none) |
| g2-proj-ai-missing-b | `26d-projectile-rowless-kinds-w4-g2.md` → `163-projectile-rowless-kinds-w4-g2.md` | 26d — the rowless proj kinds of d5's F1-F6 (W4-g2 lane b) | `docs/ANALYSIS.md` |
| d14-proj-ai-131-140 | `26d-projectile-ai-131-140-census.md` → `158-projectile-ai-131-140-census.md` | W4-d14 census — projectile aiStyles 131-140 | `docs/PORT_MAP.md` |
| d17-proj-ai-161-170 | `26d-projectile-ai-161-170-census.md` → `159-projectile-ai-161-170-census.md` | 26d — Projectile aiStyles 161-170 census (W4-d17) | (none) |
| d17-proj-ai-161-170 | `93-doc-sweep-melee-proj.md` → `220-doc-sweep-melee-proj.md` | 93 — DOC-SWEEP lane 17 (melee-proj): item use-paths behind aiStyles 161/163/166/168/182 + whip tag census | (none) |
| g1-proj-ai-missing-a | `26d-projectile-kill-hooks-census.md` → `160-projectile-kill-hooks-census.md` | 26d — Projectile Kill()/on-hit/tile-collide hook census (W4-g1, lane a) | (none) |
| d4-proj-ai-31-40 | `26d-projectile-ai-31-40-census.md` → `161-projectile-ai-31-40-census.md` | 26d — Projectile aiStyles 31-40 COVERAGE census (W4-d4) | `docs/ANALYSIS.md` |
| d4-proj-ai-31-40 | `26e-rope-coil-keep-one.md` → `169-rope-coil-keep-one.md` | 26e — Rope-coil DUPLICATE keep-one (W4-d4 DOC-SWEEP) | `docs/ANALYSIS.md`, `docs/research/26d-projectile-ai-31-40-census.md` (becomes `161-…` — same-lane rename) |
| d21-proj-ai-201-210 | `26d-projectile-ai-201-210-census.md` → `162-projectile-ai-201-210-census.md` | 26d — Projectile aiStyles 201-210 COVERAGE census (W4-d21) | `docs/ANALYSIS.md`, `docs/PORT_MAP.md` |
| d11-proj-ai-101-110 | `26d-projectile-ai-101-110-census.md` → `164-projectile-ai-101-110-census.md` | Projectile aiStyles 101-110 — census + port (W4-d11) | (none) |
| d10-proj-ai-91-100 | `26d-projectile-ai-91-100-crosschecks.md` → `165-projectile-ai-91-100-crosschecks.md` | 26d — W4-d10 parked-seam cross-checks (1128 spore row, 536 Medusa rays) | (none) |
| d10-proj-ai-91-100 | `26d-projectile-ai-91-100-docsweep.md` → `166-projectile-ai-91-100-docsweep.md` | 26d — W4-d10 DOC-SWEEP: the 94/97/98 spawn-source censuses | (none) |
| f20-proj-ai-verify-2 | `26e-projectile-ai-106-210-census.md` → `167-projectile-ai-106-210-census.md` | 26e — aiStyles 106-210 COVERAGE census + AI_001 overlay tail (W4-f20) | (none) |
| f20-proj-ai-verify-2 | `26f-projectile-ai001-overlays-round2.md` → **KEEP** (26f winner) | 26f — AI_001 OVERLAYS round 2 (W4-f20) | — |
| g3-proj-ai-missing-c | `26e-projectile-kill-onhit-census.md` → `168-projectile-kill-onhit-census.md` | 26e — Projectile Kill()/OnHit hook COVERAGE census (W4-g3, lane c) | (none) |
| d3-proj-ai-21-30 | `26f-projectile-ai-21-30-band-status.md` → `170-projectile-ai-21-30-band-status.md` | 26f — Projectile aiStyles 21-30 band status (W4-d3, the OTHER styles) | (none) |
| d61-drawdata-cases-1 | `61-drawdata-cases-1.md` → `171-drawdata-cases-1.md` | 61 — DrawData case census, slice 1 (W4-d61) | (none) |
| d62-drawdata-cases-2 | `62-drawdata-cases-2.md` → `172-drawdata-cases-2.md` | 62 — DrawData case census, slice 2 (W4-d62) | (none) |
| d63-drawdata-cases-3 | `63-drawdata-cases-3.md` → `173-drawdata-cases-3.md` | 63 — DrawData case census, slice 3 (W4-d63) | (none) |
| e25-expert-master-scaling | `64-expert-master-scaling-census.md` → `174-expert-master-scaling-census.md` | 64 — Expert/Master mode scaling census (W4-e25) | `docs/ANALYSIS.md`, `docs/PORT_MAP.md`, `docs/ROADMAP.md` |
| d80-tracked-sound-layer | `64-tracked-sound-census.md` → `175-tracked-sound-census.md` | 64 — Tracked-sound entity-loop census (W4-d80) | `docs/ANALYSIS.md`, `docs/PORT_MAP.md`, `docs/ROADMAP.md` |
| f43-town-spawn-2 | `64-town-spawn-census.md` → `176-town-spawn-census.md` | 64 — Town-spawn census (W4-f43, parallel to W4-c41) | `docs/ANALYSIS.md`, `docs/PORT_MAP.md`, `docs/research/INDEX.md` |
| g12-dust-main-sites-2 | `64-dust-main-sites-claims.md` → `177-dust-main-sites-claims.md` | 64 — Main.cs dust-site batches (W4-g11/g12/g13): claims + census | `docs/ANALYSIS.md` |
| e1-dust-sites-player-2 | `64-dust-sites-player-2.md` → `178-dust-sites-player-2.md` | W4-e1 — Player.cs Dust.NewDust sites, SECOND batch (claim ledger) | `docs/ANALYSIS.md` |
| g78-tile-echo-draw | `64-tile-echo-draw-census.md` → `179-tile-echo-draw-census.md` | 64 — Echo-coating tile/wall draw census (W4-g78) | `docs/ANALYSIS.md`, `docs/research/INDEX.md` |
| e16-item-tooltip-lines | `64-item-tooltip-line-census.md` → `180-item-tooltip-line-census.md` | 64 — Item-tooltip line census (W4-e16, code-only) | `docs/ANALYSIS.md`, `docs/PORT_MAP.md`, `docs/ROADMAP.md`, `docs/research/07-item-core.md` |
| g72-bg-layers-3 | `64-cave-bg-rng-shift-census.md` → `181-cave-bg-rng-shift-census.md` | 64 — RandomizeCaveBackgrounds worldgen RNG-shift census (W4-g72 lane 3, phase 6) | `docs/research/INDEX.md` |
| g72-bg-layers-3 | `64-v18-union-addendum.md` → `182-v18-union-addendum.md` | 64-addendum — the W4-g72 31-byte v18 claim, input to the g71c UNION | `docs/research/INDEX.md` |
| g14-dust-worldgen-sites | `65-dust-worldgen-sites-claims.md` → `183-dust-worldgen-sites-claims.md` | 65 — WorldGen.cs dust-site census + claims (W4-g14, adopting d54/e4/f6) | check own tip refs (d54's copy refs `docs/ANALYSIS.md`, `docs/PORT_MAP.md`, `docs/research/66-world-dust-reconcile.md`; blobs identical, see §4) |
| f35-expert-drop-trees | `65-expert-drop-trees-census.md` → `184-expert-drop-trees-census.md` | //! W4-f35 — the expert/master boss drop trees: the census | `docs/ANALYSIS.md`, `docs/PORT_MAP.md`, `docs/ROADMAP.md`, `docs/research/66-bag-open-master-census.md` (becomes `195-…`), `docs/research/INDEX.md` |
| g33-drawdata-wire-1 | `65-drawdata-wiring-lane1.md` → `185-drawdata-wiring-lane1.md` | 65 — DrawData wiring lane 1 (W4-g33): critter-cage animator + draw wire | (none) |
| **d54-dust-sites-world (OWN)** | `65-dust-world-sites-claims.md` → `186-dust-world-sites-claims.md` | 65 — World-sim dust-site batches (W4-d54/e4/f6): claims + census | own-branch refs incl. `docs/research/66-world-dust-reconcile.md` (becomes `194-…`) |
| g13-dust-main-sites-3 | `65-dust-main-sites-3.md` → `187-dust-main-sites-3.md` | 65 — Main.cs dust-site batch 3 (W4-g13): census completion + witnesses | `docs/ANALYSIS.md` |
| g11-dust-main-sites-1 | `65-dust-main-sites-g11.md` → `188-dust-main-sites-g11.md` | 65 — Main.cs dust sites, batch 1 (W4-g11): ledger + witnesses | `docs/research/INDEX.md` |
| c19-addlight-projectiles | `65-addlight-proj-a.md` → `189-addlight-proj-a.md` | 65 — Projectile.cs AddLight first wave (sites 1-67, W4-c19) — LANDED | (none) |
| g15-dust-item-sites | `65-dust-item-sites-claims.md` → `190-dust-item-sites-claims.md` | 65 — World-item dust sites (W4-g15): claim + census correction | (none) |
| f6-dust-world-3 | `65-dust-world-sites-claims.md` (f6 fork copy) → `191-dust-world-sites-claims-f6.md` | 65 — World-sim dust-site batches (W4-d54/e4/f6): claims + census | `docs/ANALYSIS.md` |
| e4-dust-sites-world-2 | `66-dust-world-sites-2-claims.md` → `192-dust-world-sites-2-claims.md` | 66 — World dust sites SECOND batch (W4-e4): claims + census | (none) |
| g34-drawdata-wire-2 | `66-drawdata-unwired-second-half.md` → `193-drawdata-unwired-second-half.md` | 66 — DrawData unwired-arm census, SECOND half + wiring (W4-g34) | `docs/ANALYSIS.md`, `docs/research/67-cage-animator-reconcile.md` (becomes `197-…`) |
| **d54-dust-sites-world (OWN)** | `66-world-dust-reconcile.md` → `194-world-dust-reconcile.md` | 66 — World-dust RECONCILE (W4-d54 merger assist): arm diffs, KEEP/DROP, witness union | (none outside self at tip) |
| f35-expert-drop-trees | `66-bag-open-master-census.md` → `195-bag-open-master-census.md` | //! W4-f35 DOC-SWEEP — bag OPEN mechanics + master-mode bag drop rates | `docs/research/INDEX.md` |
| g64-proj-trail-dust-2 | `66-proj-trail-dust-claims.md` (g64 fork copy) → `196-proj-trail-dust-claims-g64.md` | 66 — Projectile.cs per-kind TRAIL dust lanes (W4-g63/g64/g65): claims + census | `docs/ANALYSIS.md`, `docs/PORT_MAP.md` |
| g34-drawdata-wire-2 | `67-cage-animator-reconcile.md` → `197-cage-animator-reconcile.md` | 67 — Cage-animator triple-dup reconcile (merger assist) | `docs/research/66-drawdata-unwired-second-half.md` (becomes `193-…`) |
| f6-dust-world-3 | `67-dust-world-wiring-keep-one.md` → `198-dust-world-wiring-keep-one.md` | 67 — World-lane dust WIRING (W4-f6): keep-one + seam census | `docs/ANALYSIS.md` |
| g63-proj-trail-dust-1 | `67-proj-trail-dust-activation.md` → `199-proj-trail-dust-activation.md` | 67 — Projectile TRAIL dust activation checklist (W4-g63, for the merger) | (none) |
| d44-findframe-low-1 | `68-findframe-wire-status.md` → `200-findframe-wire-status.md` | 68 — FindFrame master wire-census (W4-d44 census refresh, code-only) | `docs/ANALYSIS.md`, `docs/PORT_MAP.md`, `docs/research/INDEX.md` |
| f26-findframe-extra-3 | `68-findframe-census-addendum-7c.md` → `201-findframe-census-addendum-7c.md` | 68 addendum — FindFrame census after f26 slice 7c (code-only) | `docs/research/68-findframe-census-addendum-7e.md` (→`203-…`), `docs/research/68-findframe-census-addendum-7f.md` (→`205-…`) |
| g13-dust-main-sites-3 | `68-dust-main-residual-sweep.md` → `202-dust-main-residual-sweep.md` | 68 — Main dust residuals doc-sweep (W4-g13, lane 41) | `docs/ANALYSIS.md`, `docs/research/65-dust-main-sites-3.md` (becomes `187-…`), `docs/research/INDEX.md` |
| f26-findframe-extra-3 | `68-findframe-census-addendum-7e.md` → `203-findframe-census-addendum-7e.md` | 68 addendum — FindFrame census after f26 slice 7e (code-only) | `docs/ANALYSIS.md`, `docs/PORT_MAP.md` |
| g34-drawdata-wire-2 | `68-cage-critter-items-census.md` → `204-cage-critter-items-census.md` | 68 — Cage critter items: place/release/light census (W4-g34) | (none) |
| f26-findframe-extra-3 | `68-findframe-census-addendum-7f.md` → `205-findframe-census-addendum-7f.md` | 68 addendum — FindFrame census after f26 slice 7f (code-only) | `docs/ANALYSIS.md`, `docs/PORT_MAP.md` |
| g13-dust-main-sites-3 | `69-dust-second-chain-lights.md` → `206-dust-second-chain-lights.md` | 69 — The 52 orphaned second-chain dust lights (W4-g13, flag-a closure) | `docs/research/INDEX.md` |
| g45-liquid-lava-honey-delays | `69-liquid-reconcile.md` (g45 fork copy) → `207-liquid-reconcile-g45.md` | 69 — Liquid 4-way reconcile (W4-g46 merger assist) | `docs/ANALYSIS.md`, `docs/research/104-doc-sweep-liquid.md`, `docs/research/INDEX.md` |
| f41-birthday-party-2 | `70-party-reconcile.md` → `208-party-reconcile.md` | 70 — Birthday-party 3-way reconcile (merger assist) | (none) |
| g47-liquid-skip-settle | `70-liquid-merge-order.md` → `209-liquid-merge-order.md` | 70 — FINAL liquid merge order (W4-g47, the closing verdict) | `docs/ANALYSIS.md` |
| g13-dust-main-sites-3 | `70-dust-draw-tint-final.md` → `210-dust-draw-tint-final.md` | 70 — Dust DRAW-tint census FINAL + GetAlpha 157/327 drop closure (W4-g13) | `docs/research/INDEX.md` |
| f4-dust-npc-3 | `71-npc-gore-wave-e-claims.md` → `211-npc-gore-wave-e-claims.md` | NPC gore wave E — HitEffect-remainder census + claim (W4-f4) | `docs/ANALYSIS.md` |
| c42-wiring-device-arms | `71-wiring-device-reconcile.md` (c42 fork copy) → `212-wiring-device-reconcile-c42.md` | 71 — Wiring device-arm duplicate reconcile (W4-e20 + W4-f11) + e21 census | (none outside self at tip) |
| g22-sounds-main-2 | `90-doc-sweep-menu-sounds.md` → `213-doc-sweep-menu-sounds.md` | 90 — DOC-SWEEP lane 14: menu/UI sounds (W4-g22 continuation) | `docs/research/91-g23-refold-g22-doc-sweep.md` (becomes `215-…`) |
| g4-npc-style3-fighter | `91-doc-sweep-fighter.md` → `214-doc-sweep-fighter.md` | 91 — DOC-SWEEP lane 15: the fighter AI (aiStyle 3) | `docs/research/INDEX.md` |
| g22-sounds-main-2 | `91-g23-refold-g22-doc-sweep.md` → `215-g23-refold-g22-doc-sweep.md` | 91 — g23 re-fold list: g22's post-e0f81135 doc-sweep (57 sites) | `docs/research/90-doc-sweep-menu-sounds.md` (→`213-…`), `docs/research/91-g23-branch-deprecated.md` (→`217-…`) |
| g4-npc-style3-fighter | `91-addendum-d22r-fold-list.md` → `216-addendum-d22r-fold-list.md` | 91 — ADDENDUM A: the d22r RESUME fold-list (g4 lane 15 residuals) | `docs/ANALYSIS.md`, `docs/research/INDEX.md` |
| g22-sounds-main-2 | `91-g23-branch-deprecated.md` → `217-g23-branch-deprecated.md` | 91 — g23-branch-DEPRECATED: merge-direction recommendation (MENU RETIRE) | `docs/research/90-doc-sweep-menu-sounds.md` (→`213-…`), `docs/research/91-g23-refold-g22-doc-sweep.md` (→`215-…`) |
| g71-bg-layers-2 | `92-doc-sweep-bg.md` → `218-doc-sweep-bg.md` | 92 — DOC-SWEEP: background variants (lane 16, W4-g71) | `docs/ANALYSIS.md`, `docs/research/INDEX.md` |
| f33-torch-god-favor | `93-torch-god-949-keep-one.md` → `219-torch-god-949-keep-one.md` | 93 — The 949 Torch God duplicate: W4-f33 vs W4-d19 keep-one | `docs/ANALYSIS.md`, `docs/research/93-torch-god-fold-input-addendum.md` (becomes `221-torch-god-fold-input-addendum.md`), `docs/research/INDEX.md` |
| f33-torch-god-favor | `93-torch-god-fold-input-addendum.md` → `221-torch-god-fold-input-addendum.md` | 93 addendum — W4-f33 persist fold-input + the d19 949 drop-hunks (closeout) | `docs/ANALYSIS.md`, `docs/research/93-torch-god-949-keep-one.md` (becomes `219-…` — same-lane rename), `docs/research/INDEX.md` — round-1 §1 assigned 221 but the §3 row was missing; refresh backfill |
| g40-biome-marble-granite | `94-w4g40-biome-residual-census.md` → `222-w4g40-biome-residual-census.md` | 94 — Biome-residual census (W4-g40 continuation, code-only) | `docs/ANALYSIS.md`, `docs/research/INDEX.md` |
| f2-npc-sounds-3 | `94-npc-sound-reconcile.md` → `223-npc-sound-reconcile.md` | 94 — NPC-sound dup-reconcile: W4-e9 (3793fef8) vs W4-f2 (9de95697) | `docs/ANALYSIS.md`, `docs/ROADMAP.md` |
| c29-deadmans-chest-biome | `95-gem-moss-census.md` → `224-gem-moss-census.md` | 95 — Gem-moss biome gen census + fidelity port (W4-c29) | `docs/research/INDEX.md` |
| g41-biome-spider-cave | `95-finish-no-traps-remix-census.md` → `225-finish-no-traps-remix-census.md` | 95 — FinishNoTraps + FinishRemixWorld census (W4-g41 finish gaps) | `docs/ANALYSIS.md`, `docs/PORT_MAP.md`, `docs/research/INDEX.md` |
| d8-proj-ai-71-80 | `97-final-phase-proj-ai-71-80.md` → `226-final-phase-proj-ai-71-80.md` | 97 — FINAL-PHASE lane 18 (proj AI 71-80): drill split, 434 verdict, band census | `docs/research/INDEX.md` |
| g43-biome-jungle-temple | `101-doc-sweep-temple.md` → `227-doc-sweep-temple.md` | 101 — DOC-SWEEP lane 22: the temple/jungle USE-side census | `docs/ANALYSIS.md`, `docs/ROADMAP.md` |
| d20-proj-ai-191-200 | `109-doc-sweep-proj-191.md` → `228-doc-sweep-proj-191.md` | 109 — DOC-SWEEP lane 28: proj aiStyle 191-200 skip census | `docs/research/INDEX.md` |
| d36x-npc-kinds-141-210-exec | `109-d36-npc-kinds-141-210-census.md` (d36x fork copy) → `229-d36x-npc-kinds-141-210-census.md` | 109 — W4-d36 NPC kinds 141-210: census, claims, id-band plan | (none outside self at tip) |
| d36-npc-kinds-141-210 | `109-d36-npc-kinds-141-210-census.md` → `236-d36-npc-kinds-141-210-census.md` | 109 — W4-d36 NPC kinds 141-210: census, claims, id-band plan | (none outside self at tip) |
| d39-npc-kinds-351-420 | `125-d39-band-351-420-census.md` → `230-d39-band-351-420-census.md` | 125 — W4-d39 band 351-420 census + wave-A draft disposition | (none) |
| e3-dust-sites-proj-2 | `125-proj-dust-coverage.md` → `231-proj-dust-coverage.md` | 125 — PROJ-DUST master coverage census (W4-e3, coverage phase) | `docs/ANALYSIS.md` |
| c42-wiring-device-arms | `133-fold-receipt.md` → `232-fold-receipt.md` | 133-fold-receipt — the f11 unique folds EXECUTED on the c42 union | `docs/research/95-wiring-master-reconcile.md` |
| f11-wiring-arms-4 | `134-light-family-unification.md` → `233-light-family-unification.md` | 134 — The light-family unification: final hunks for c42c | `docs/research/INDEX.md` |
| f11-wiring-arms-4 | `135-wires-radial.md` → `234-wires-radial.md` | 135 — WiresRadial: the logic half landed, the draw contract | `docs/research/INDEX.md` |
| f11-wiring-arms-4 | `136-grand-design-activation.md` → `235-grand-design-activation.md` | 136 — The Grand Design 3611 drag / proj-651: landed-verification + apply activation | `docs/research/INDEX.md` |

Note on d54 (this branch): the two OWN-branch rows above (65→186, 66→194) are
recorded for completeness; d54's live dust-lane docs (`65-dust-world-sites-claims`,
`66-world-dust-reconcile`, `67-dust-orch-sites-claims`, `131-dust-master-status`)
are mid-flight inputs to the dust merge order documented in `131`; renaming them
here would break every sibling lane's citations before the dust union lands.
**Merger instruction: when merging d54, apply the 186/194 renames at merge time
(or after the dust union), and dedupe per §4.** The registry itself (150) and
the INDEX note are the published artifacts of this lane.

## 4. Same-name shared docs (NOT number collisions) — merger dedupe rules

| File | Branches | Blob verdict | Merger action |
| --- | --- | --- | --- |
| `65-dust-worldgen-sites-claims.md` | d54, g14 | **IDENTICAL** (`git rev-parse` equal) | keep one; g14 renames its copy →183 per §3, d54's copy rides the dust-union merge (then →183 or drop as duplicate) |
| `65-dust-world-sites-claims.md` | d54 (00:43:15), f6 (00:48:39) | DIVERGED (`d7ec020e3` vs `a644d7988`) | d54 copy earlier → number 65 lineage until d54 rename →186; f6 fork →191 |
| `66-proj-trail-dust-claims.md` | g63 (00:30:45), g64 (01:53:25) | DIVERGED (`2937a5c3a` vs `f8523add0`) | g63 keeps 66; g64 fork →196 |
| `69-liquid-reconcile.md` | g46 (02:58:06), g45 (08:13:20) | DIVERGED (`e3d76d679` vs `955b62dbc`) | g46 keeps 69; g45 fork →207 |
| `71-wiring-device-reconcile.md` | e20 (03:50:05), c42 (05:35:32) | DIVERGED (`b37112c78` vs `bfb5ee53e`) | e20 keeps 71; c42 fork →212 |
| `109-d36-npc-kinds-141-210-census.md` | d36, d36x (same add `0eb4fee95`) | DIVERGED at tips (`69a7015d6` vs `82813937b`) | d36 keeps 109-slot lineage vs c9 winner (c9 earlier 07:13:30 → c9 keeps 109; BOTH d36/d36x copies are late — d36 copy should ALSO take a free number 236 at rename time; d36x fork →229) |
| `28c/28d/28e/28f-wave-*-claim.md` | c17/c18/d71/d72 + f12 | **IDENTICAL** (all four) | keep one each; no renumber |
| `90/91/92` + `01–63` base census docs | ~all branches | identical inherited base | normal git merge dedupe; only the NEW docs in §1 collide |

**Correction row (109 three-way):** c9 `109-snail-band-claims.md` (07:13:30)
wins 109 outright; d20 →228, d36x →229 per §3, and **d36's own
`109-d36-npc-kinds-141-210-census.md` → 236** (first free number after the
block). d36 refs-to-update: (none outside self at tip).

## 6. REFRESH (round 2) — new collisions, self-rename reconciliation, block 244–280

Re-scan: **358 tips, 21 038 ls-tree rows, 47 colliding keys** (40 bare + 7
letter-suffixed), six fetches (lanes pushed mid-round; the final sweep is
the authority). Same rules as round 1: first-add `%cI` per filename per
branch wins; fork tiebreak by lane name; letter-suffixed losers drop the
letter. All timestamps 2026-11-21 +02:00 unless noted.

**Free numbers consumed by WINNER pushes before this refresh published (not
assignments): 237 (e23), 238 (d42), 239 (d42), 240/241/242/243 (g58).**
Executed pushes beat unpublished registry drafts — g58's 143→240 self-rename
(`cdfe3d108` 12:16:37) and its new 241/242/243 docs are VALID takes (240 wins
vs d53 14:08:36 / c42 14:17:05; 243 single claimant). The round-2 assignment
block therefore starts at **244**.

### 6.1 New / extended collision keys (winner keeps, losers → 244–280)

| Key | Winner (keeps number) | Winner doc / ts | Losing claimants → FREE number |
| --- | --- | --- | --- |
| 12k | W4-f24 | `12k-f24-worldgen-gated.md` 00:09:38 `f353e13f6` | W4-c31 `12k-c31-jungle-residuals.md` 05:10:24 → **244** |
| 18b (FORK) | W4-d82 (**§7.1 CORRECTION: d83 is NOT identical** — blob `0c9ea47d2` diverged, d83 fork → **302**) | `18b-element-census.md` 11-20T18:38:12 `d4a4e7fda` blob `21808dd38` | FORK: W4-d81 copy (add `44744efa5` 11-20T19:41:08, blob `ba85736f1`) → ~~245~~ **301 (§7.3 RE-ISSUE)**; W4-g53 copy is IDENTICAL to d81's (same add commit+blob) → rides 301, merger keep-one, NO extra number; FORK: W4-e23 copy (same add commit 19:41:08, diverged blob `fc715ca4f`; lane tiebreak d81 < e23) → **246**; FORK: W4-d83 copy (same add, diverged blob `0c9ea47d2`; lane tiebreak d82 < d83) → **302** (§7.3) |
| 26d (+1) | W4-d12 (round 1) | 00:06:06 | W4-d10 `26d-projectile-ai-91-100-final.md` 12:07:11 (NEW third d10 doc) → **247** |
| 26e (+1) | W4-d7 (round 1) | 01:30:11 | W4-g82 `26e-f-rope-union-final.md` 09:31:06 → **248** |
| 64 (+2) | W4-d64 (round 1) | 11-20T18:54:02 | W4-f18 `64-npc-kinds-351-697-census.md` 05:02:54 → **249**; W4-e2 `64-npc-dust-window-coverage.md` 10:48:45 → **250** |
| 68 (+1) | W4-f37 (round 1) | 02:48:30 | W4-f26 `68-findframe-census-addendum-7g.md` 11:12:33 → **251** |
| 70 (FORK) | W4-d53 (round 1) keeps 70; g47 keeps its **209** assignment | `70-liquid-merge-order.md` g47 first add 05:56:14 `fdeb897ca` blob `266e2d011` | FORK: W4-g46 copy of `70-liquid-merge-order.md` (first add 10:00:03 `9bc722e5f`, diverged blob `e29d0864d`) → **252** |
| 83 (FORK) | W4-d130 | `83-weather-reconcile.md` 04:12:25 `50c845e5e` (tip blob advanced to `f0c1c80c9` mid-round; first-add unchanged) | FORK: W4-g11 copy (first add 05:25:53 `d892d8d43`, blob `555ccff03`) → **253** |
| 84 (FORK) | W4-f46 | `84-map-reconcile.md` 03:54:03 `38c283a0d` blob `92ef798cf` | FORK: W4-e18 copy (first add 05:57:51 `0bd346e78`; tip blob moved `052859ca4`→`2a6b3a231`→`a0d6956a8` mid-round — first-add unchanged) → **254** |
| 96 | W4-d8 | `96-doc-sweep-proj-misc.md` 06:17:01 `35bacbb6e` | W4-g41 `96-biome-retire-receipts.md` 10:33:31 → **255** |
| 108 | W4-d37 | `108-roster-master-reconcile.md` 05:31:28 `08f7d25cf` | W4-f42 `108-input-f42.md` 12:02:51 → **256** |
| 111 | W4-d70 | `111-doc-sweep-screenfx.md` 10:38:01 `a7620eec3` | W4-f5 `111-gore-final-verification.md` 12:37:04 → **257** |
| 122 (FORK) | W4-g89 | `122-persist-v9-addendum.md` 05:21:18 `4344bd50e` blob `f92eaec98` | FORK: W4-g88 copy (first add 06:16:30 `6998a4ecb`, diverged blob `4eeaad66d`) → **258** |
| 129 | W4-f28 | `129-f28-deferred-families.md` 08:33:30 `9ca48c496` | W4-d105 `129-doc-sweep-wg-37.md` 12:32:10 → **259** |
| 130 | W4-f28 | `130-f28-retire.md` 09:45:35 `922e4fd25` | W4-c29 `130-doc-sweep-gem-moss.md` 12:04:03 → **260** |
| 137 | W4-g84 | `137-g84-g31-keep-one.md` 09:37:33 `e6c2dcd0d` | W4-f45 `137-wld-final-audit.md` 14:14:09 `d2a8276a2` → **279** |
| 143 | W4-c4 | `143-d40-crimsongoldfish-retire-hunk.md` 09:02:12 `90fd242d9` (round-1 one-sided entry became a FULL 6-way collision; d42/c8/g58 have since renamed away, 3 claimants remain at tips) | W4-d120 chatter-retire 09:46:26 → **261**; W4-g87 fold-input 11:08:00 → **263**; W4-c8 crawler-final-census 10:03:55 (self-renamed 143→238 at `2a0b5c701`, 238 collides → chain) → **262**; W4-d42 roster-sheets 09:50:22 self-renamed 143→238 at `9eadc3043` 12:15:55 and **WINS 238 — CONFIRMED**; W4-g58 happiness-final 10:26:59 self-renamed 143→240 at `cdfe3d108` 12:16:37 and **WINS 240 — CONFIRMED** |
| 144 | ~~W4-c4~~ **W4-e19 (§7.1 CORRECTION: `144-splash-keep-one.md` 08:45:32 `cd2fbfa10` predates the orca — c4 WIN REVOKED → 294)** | `144-orca-npc692-census-input.md` 09:15:20 `479d70ad2` (now 3-way: f47 pushed TWO 144 docs mid-round) | W4-f22 `144-f22-ui-screens-retire.md` 12:04:59 → **264**; W4-f47 `144-item-splash-keep-one.md` 12:39:05 `f80490bcf` → **277**; W4-f47 `144-ARBITER-FINAL.md` 13:52:51 `18fe75f2e` → **278**; W4-c4 orca → **294** (§7.3) |
| 145 | ~~W4-g89~~ **W4-e19 (§7.1 CORRECTION: `145-16box-cross-lane-repins.md` 09:32:12 `c1586b9cb` predates the rod doc — g89 WIN REVOKED → 295)** | `145-rod-teleport-union-composition.md` 09:48:08 `41dabfbc8` (g89 SELF-RENAMED 139→145; 145 free at that moment) | W4-c4 `145-d42-keep-d42-verdicts-addendum.md` 10:12:10 → **265**; W4-g89 rod → **295** (§7.3) |
| 146 | W4-g24 | `146-wg-sounds-final.md` 10:02:44 `2678cfae4` (g24 SELF-RENAMED 107→146) | W4-e20 `146-c42-repick-critical.md` 10:22:27 → **266**; W4-g67 `146-g67-post-union-folds.md` 10:32:09 (g67 SELF-RENAMED 134→146) → **267** |
| 147 | W4-e20 | `147-c42e-session-probe.md` 10:25:00 `038af46cf` | W4-e19 `147-quicksell-ui-hint-census.md` 12:52:51 → **268** |
| 148 | W4-c4 | `148-fish-family-declaration.md` 10:16:16 `0df6964be` | W4-e20 `148-brokentrapcleanup-wg-seed.md` 10:28:49 → **269** |
| 149 | W4-e20 | `149-devices-family-bible.md` 10:31:10 `46fdcfd8e` | W4-c10 `149-waterfowl-band-final.md` 10:49:53 (c10 SELF-RENAMED 145→149) → **270** |
| 154 (reserved) | W4-c33 (round-1 ASSIGNMENT `12i-c33…`→154, not yet executed) | registry reservation `38bfd1d22` 10:22:14 | W4-g50 `154-g52c-cross-input-40-statuses.md` 12:01:55 (picked "max-on-origin 153 + 1" from f11's INTERIM wave — the registry block 151–236 was already reserved) → **271** |
| 237 | W4-e23 | `237-bestiary-ui-final.md` 10:53:03 `7a7779690` (took 237 per round-1 free-block guidance — VALID, 237 was free) | W4-g48 `237-f11-tail-fix.md` 12:15:09 `34029437a` → **272**; W4-g61's 237 doc self-renamed →238 mid-round (chain below) |
| 238 | W4-d42 | `238-d42-roster-sheets.md` 12:15:55 `9eadc3043` (self-rename from 143, first to land 238) | W4-c8 `238-c8-crawler-final-census.md` 12:24:19 `2a0b5c701` (chain 143→238) → **262** (counted in the 143 row); W4-g61 `238-g61-spawn-family-final.md` 12:55:22 `6e922fc60` (chain 237→238) → **273** |
| 239 | W4-d42 | `239-d42-final-keep-one-and-band.md` 12:55:30 `bac408be0` | W4-g56 `239-void-family-bible.md` 13:57:53 `54c63d8d4` → **274**; W4-f21 `239-f21b-net-apply-audit.md` 14:11:45 `f7bc25471` → **276** (the long-awaited f21b sub-round doc from the round-1 brief — pushed late, loses) |
| 240 | W4-g58 | `240-happiness-final.md` 12:16:37 `cdfe3d108` (143 self-rename; 241/242/243 g58 docs all single-claimant, **KEEP** — **⚠ §7.4: 243 now contested by d50 →304; 240 also claimed by g63 →298 / f4 →305**) | W4-d53 `240-proj-dust-retire.md` 14:08:36 `df3355c36` → **275** (**⚠ RE-ISSUED §7.4.3: d53 executed 240→244, old name now `244-proj-dust-retire.md`**); W4-c42 `240-c42-wiring-seal.md` 14:17:05 `2910a7209` → **280** |

Single-claimant new numbers verified FREE (no action): **140** (f29
map-overlay-reconcile), **141** (e9 npc-sound-final-census), **142** (d122
savings-keep-one), **121** (f47 doc-sweep-shimmer, unchanged). Round-1
execution spotted: **g64 landed `196-proj-trail-dust-claims-g64.md`** per §3
— CONFIRMED. **107** and **139** are now EMPTY at all tips (g24/g89 renamed
away); 139 stays RESERVED for g52c-net-master per the g50 cross-note — a
future g52c 139 doc keeps 139.

### 6.2 Self-rename reconciliation (rename chains)

| Lane | Chain | Verdict |
| --- | --- | --- |
| g24-sounds-worldgen | 107 → 146 | **146 KEEPS** — first-add winner 10:02:44 vs e20 10:22:27 / g67 10:32:09. No further rename. |
| g67-weather-rain-splash | 134 → 146 → **267** | 146 collides, g67 is LAST of the three → rename again to 267 (§6.3). **⚠ §7.4: g67 executed 146→247 (`8387af9ac` 15:13:18) — 247 contested by d10's published reservation → 267 RE-ISSUED §7.4.3, chain 134→146→247→267.** |
| e20-wiring-device-arms-2 | (c42-repick) 146 → **266** | second of the three → 266. e20's **149 KEEPS** (10:31:10 < c10 10:49:53); e20's **147 KEEPS** (10:25:00 < e19 12:52:51); e20's **148 → 269** (c4 earlier); e20's `71-wiring-device-reconcile.md` keeps 71 (round-1 winner). |
| c10-style68-waterfowl | 145 → 149 → **270** | 149 collides with e20's earlier devices-bible → rename again to 270. |
| e23-bestiary-ui-grid | → 237 | **237 KEEPS** (10:53:03 < g61 12:12:59 < g48 12:15:09). e23's `18b-element-census.md` fork copy is a SEPARATE loss → 246. |
| g61-npc-spawn-dust | 237 → 238 → **273** | 237 taken by e23; the self-rename to 238 (`6e922fc60` 12:55:22) landed AFTER d42's 238 (12:15:55) and c8's 238 (12:24:19) → rename again to 273. |
| c8-style66-crawling-critters | 143 → 238 → **262** | 143 belongs to c4; the self-rename to 238 (`2a0b5c701` 12:24:19) lost to d42's earlier 238 (12:15:55) → rename again to 262. |
| d42-npc-kinds-561-630 | 143 → **238 KEEPS**; new doc **239 KEEPS** | d42's rename (`9eadc3043` 12:15:55) is the FIRST 238 at any tip → wins the key vs c8/g61. `239-d42-final-keep-one-and-band.md` (12:55:30) beats g56's 239 (13:57:53) and f21's 239 (14:11:45). **CONFIRMED — no further action.** |
| g58-ui-shop-happiness | 143 → **240 KEEPS**; new docs **241/242/243 KEEP** | `cdfe3d108` 12:16:37 renamed 143→240; beats d53's 240 (14:08:36) and c42's 240 (14:17:05). `241-f37-latch-repoint-hunks.md` 12:25:26, `242-chat-portrait-seed-happiness-sealed.md` 13:12:35, `243-seal-verification.md` 14:08:18 single claimants (**⚠ §7.4: 243 later contested by d50 14:59:24 — g58 still WINS, d50 →304; 240 also contested by g63/f4, g58 WINS both**). Executed pushes beat this refresh's unpublished draft reservations → **CONFIRMED, no further action** (the draft's c31/d81/e23 shifted to 244/245/246). |
| d53-dust-sites-projectile | (new) 240 → **275** | pushed 14:08:36, after g58's 240 (12:16:37) → loser, rename to 275. d53's round-1 70 win is unaffected. **⚠ §7.4: d53 executed 240→244 (`a106d13e8` 14:56:57) — 244 contested by g48 → 275 RE-ISSUED §7.4.3, chain 240→244→275.** |
| c42-wiring-device-arms | (new) 240 → **280** | pushed 14:17:05, last of the three 240s → rename to 280. c42's round-1 assignments (71-fork→212, 133→232) still owed. |
| f21-net-ids-verify-tail | (new) 239 → **276** | the f21b sub-round doc the round-1 brief flagged as unpushed FINALLY landed (14:11:45) — on 239, taken by d42 at 12:55:30 → rename to 276. |
| g56-ui-void-vault | (new) 239 → **274** | pushed 13:57:53, after d42's 239 (12:55:30) → loser, rename to 274. |
| g48-net-apply-arms-1 | (new) 237 → **272** | pushed 237 at 12:15:09 citing the round-1 free block, but e23's 237 (10:53:03) was already at its tip → loser, rename to 272. |
| g84-plantera-bulb-draw | 137 pushed first | **137 KEEPS** (09:37:33 vs f45 14:14:09). |
| f45-wld-header-2 | (new) 137 → **279** | pushed 14:14:09, after g84's 137 → rename to 279. |
| g89-aegis-fruit-use | 139 → 145 | ~~**145 KEEPS**~~ **§7.1 REVOKED: e19's 16box (09:32:12) predates g89 (09:48:08) → 295 (chain 139→145→295).** g89's `122-persist-v9-addendum.md` keeps 122 (fork winner vs g88). |
| c4-style44-flying-fish | 143 / 144 / 148 pushed first | **143/148 KEEP**; ~~144 KEEP~~ **§7.1 REVOKED: e19's splash (08:45:32) predates the orca (09:15:20) → 294**; c4's later `145-d42-keep-d42-verdicts-addendum.md` loses 145 → **265**. |
| f47-liquid-shimmer-2 | (new) 144 ×2 → **277/278** | both 144 docs pushed after c4's orca (09:15:20): splash-keep-one 12:39:05 → 277, ARBITER-FINAL 13:52:51 → 278. |
| f11-wiring-arms-4 | 134/135/136 → 151/152/153 → **233/234/235** | Interim pick (`b03d53f1d` 10:32:33, "150 not yet landed") collided with the registry block (151=d113, 152=g36, 153=f25). f11 SELF-CORRECTED at `14ca0a8d5` 12:07:51 citing this registry: 233/234/235 now at tip, all refs re-pointed (wiring.rs comments, INDEX, ANALYSIS/ROADMAP, anchors.txt). **CONFIRMED — matches round-1 §3 assignments; no further action.** |
| g50-net-apply-arms-3 | (new) 154 → **271** | picked max-on-origin+1 from f11's interim wave; 154 belongs to c33 per round-1 §3. Rename to 271. Its "139 RESERVED for g52c" note is honored by this registry. |
| g64-proj-trail-dust-2 | 66-fork → **196 EXECUTED** | round-1 §3 assignment landed at tip (`196-proj-trail-dust-claims-g64.md`). **CONFIRMED.** |

### 6.3 Per-lane rename-INSTRUCTION table (round 2; lanes execute on OWN branches)

Same procedure as §3: `git mv`, update the doc's number header, update every
listed ref file (`git grep -l -F <oldname> <tip> -- docs/`, self excluded).
Old filenames are the CURRENT tip names (c8/g61 already carry their interim
238 names).

| Lane (branch `task/W4-…`) | old filename → new filename | doc title (first header) | refs-to-update on own branch |
| --- | --- | --- | --- |
| c31-hive-rewrite | `12k-c31-jungle-residuals.md` → `244-c31-jungle-residuals.md` **⚠ RE-ISSUED §7.3 → 300** (244 taken by g48's pre-publish push) | 12k — W4-c31 jungle residuals census + gap-fill | `docs/ANALYSIS.md` |
| d81-ui-elements-1 | `18b-element-census.md` (d81 fork copy) → `245-element-census-d81.md` **⚠ RE-ISSUED §7.3 → 301** (245 taken by g48's pre-publish push) | 18b — UIElement class census + the d81/d82/d83 range split | (none) |
| g53-ui-achievement-screen | `18b-element-census.md` — IDENTICAL blob to d81's copy | (same header) | rename to the SAME `245-element-census-d81.md` **⚠ RE-ISSUED §7.3 → 301** (or drop the copy); merger keep-one with d81. No separate number. |
| e23-bestiary-ui-grid | `18b-element-census.md` (e23 fork copy) → `246-element-census-e23.md` | 18b — UIElement class census + the d81/d82/d83 range split | (none) — e23's `237-bestiary-ui-final.md` KEEPS 237 |
| d10-proj-ai-91-100 | `26d-projectile-ai-91-100-final.md` → `247-projectile-ai-91-100-final.md` **⚠ RE-ISSUED §7.3: old name now `246-projectile-ai-91-100-final.md`** (d10 executed 26d→246 post-publish; 246=e23's reservation; target stays 247) | 26d — W4-d10 FINAL: the projectile aiStyles 91-100 band table | (none) — d10's OTHER two 26d docs keep their round-1 assignments 165/166 (EXECUTED `d89380ca1`) |
| g82-rope-coil-draw | `26e-f-rope-union-final.md` → `248-rope-union-final.md` | 26e-f — Rope-coil FINAL union (W4-g82 final phase) | `docs/ANALYSIS.md`, `docs/PORT_MAP.md`, `docs/research/90-consolidated-findings.md` |
| f18-npc-kinds-verify-2 | `64-npc-kinds-351-697-census.md` → `249-npc-kinds-351-697-census.md` | 64 — NPC kinds 351–697 SetDefaults census + first-20 port (W4-f18) | (none) |
| e2-dust-sites-npc-2 | `64-npc-dust-window-coverage.md` → `250-npc-dust-window-coverage.md` | 64 — NPC.cs dust-site window coverage | (none) |
| f26-findframe-extra-3 | `68-findframe-census-addendum-7g.md` → `251-findframe-census-addendum-7g.md` | 68 addendum — FindFrame census after f26 slice 7g (code-only) | `docs/ANALYSIS.md`, `docs/PORT_MAP.md` — f26's 7c/7e/7f keep round-1 assignments 201/203/205 |
| g46-liquid-evaporation | `70-liquid-merge-order.md` (g46 fork copy) → `252-liquid-merge-order-g46.md` | 70 — FINAL liquid merge order (W4-g47, the closing verdict) | `docs/ANALYSIS.md`; `docs/research/69-liquid-reconcile.md` stays (g46 keeps 69) |
| g11-dust-main-sites-1 | `83-weather-reconcile.md` (g11 fork copy) → `253-weather-reconcile-g11.md` | 83 — Weather reconcile: the g11-side addendum (snowing keep-one) | `docs/research/INDEX.md` |
| e18-minimap-reveal-queue | `84-map-reconcile.md` (e18 fork copy) → `254-map-reconcile-e18.md` | 84 — Map reveal dup-reconcile: W4-e18 vs W4-f46 | `docs/ANALYSIS.md` |
| g41-biome-spider-cave | `96-biome-retire-receipts.md` → `255-biome-retire-receipts.md` | 96 — W4-g41 BIOME RETIRE: cross-lane receipts + the FINAL declaration | `docs/research/INDEX.md` — g41's 93/94 wins and 95→225 assignment unchanged |
| f42-sandstorm-2 | `108-input-f42.md` → `256-input-f42.md` | 108-input-f42 — W4-f42 kind claims (input for the d37 roster matrix) | (none) |
| f5-dust-proj-3 | `111-gore-final-verification.md` → `257-gore-final-verification.md` | 111 — Gore-family FINAL verification + completion receipt (W4-f5) | `docs/research/INDEX.md` |
| g88-life-fruit-use | `122-persist-v9-addendum.md` (g88 fork copy) → `258-persist-v9-addendum-g88.md` | 122 — .trplayer v9 addendum: the sidecar contract AFTER dye | `docs/ANALYSIS.md`, `docs/PORT_MAP.md`, `docs/research/103-persist-v7-layout.md`, `docs/research/INDEX.md` |
| d105-worldgen-passes-5 | `129-doc-sweep-wg-37.md` → `259-doc-sweep-wg-37.md` | 129 — Doc-sweep lane 39: worldgen #37-45 residuals (W4-d105 sweep phase) | `docs/research/12e-worldgen-passes-5.md` |
| c29-deadmans-chest-biome | `130-doc-sweep-gem-moss.md` → `260-doc-sweep-gem-moss.md` | 130 — DOC-SWEEP lane 40: the gem/moss residuals (W4-c29 sweep phase) | `docs/ANALYSIS.md`, `docs/research/INDEX.md` — c29's 95→224 assignment unchanged |
| d120-town-chatter | `143-d120-chatter-retire.md` → `261-d120-chatter-retire.md` | 143 — W4-d120 town chatter RETIRE | (none) |
| c8-style66-crawling-critters | `238-c8-crawler-final-census.md` → `262-c8-crawler-final-census.md` | 238 — W4-c8 crawler FINAL census | `docs/ANALYSIS.md`, `docs/research/INDEX.md` — rename-chain note: 143→238→262 (238 belongs to d42's earlier rename); update the doc's RENUMBERED provenance block |
| g87-demon-heart-use | `143-g87-fold-input.md` → `263-g87-fold-input.md` | 143 - W4-g87 closeout fold-input | `docs/ANALYSIS.md`, `docs/research/INDEX.md` |
| f22-ui-screens-4 | `144-f22-ui-screens-retire.md` → `264-f22-ui-screens-retire.md` | 144 — W4-f22 UI screens RETIRE | `docs/research/18d-ui-screens-4-census.md` |
| c4-style44-flying-fish | `145-d42-keep-d42-verdicts-addendum.md` → `265-d42-keep-d42-verdicts-addendum.md` | 145 — Claims-table addendum: the d42 keep-d42 verdicts are SUPERSEDED | `docs/ANALYSIS.md`, `docs/research/INDEX.md`; `docs/research/143-d40-crimsongoldfish-retire-hunk.md` stays (c4 keeps 143) |
| e20-wiring-device-arms-2 | `146-c42-repick-critical.md` → `266-c42-repick-critical.md` | 146 — c42 re-pick CRITICAL: device-arm halves CANNOT MERGE AS-IS | `docs/research/149-devices-family-bible.md`, `docs/research/71-wiring-device-reconcile.md` (both KEEP their numbers on e20 — refs stay, only this doc's own name/header changes) |
| g67-weather-rain-splash | `146-g67-post-union-folds.md` → `267-g67-post-union-folds.md` **⚠ RE-ISSUED §7.4.3: old name now `247-g67-post-union-folds.md`** (g67 executed 146→247; 247 contested by d10's reservation; target stays 267) | 146 — W4-g67 post-union fold-list (for W4-d130f) | `docs/research/INDEX.md` — rename-chain note: 134→146→247→267; record the chain in the header like f11 did |
| e19-liquid-quicksell-shimmer | `147-quicksell-ui-hint-census.md` → `268-quicksell-ui-hint-census.md` | 147 — The quicksell SHIFT-click UI-hint census (W4-e19 final, SHIMMER item 4) | `docs/research/INDEX.md` |
| e20-wiring-device-arms-2 | `148-brokentrapcleanup-wg-seed.md` → `269-brokentrapcleanup-wg-seed.md` | 148 — BrokenTrapCleanup wg-seed: census + dedupe | `docs/research/149-devices-family-bible.md`, `docs/research/71-wiring-device-reconcile.md` (both keep) |
| c10-style68-waterfowl | `149-waterfowl-band-final.md` → `270-waterfowl-band-final.md` | 149 — W4-c10 waterfowl band FINAL | `docs/ANALYSIS.md`, `docs/research/INDEX.md` — rename-chain note: 145→149→270 |
| g50-net-apply-arms-3 | `154-g52c-cross-input-40-statuses.md` → `271-g52c-cross-input-40-statuses.md` | 154 — g52c cross-input: research/40's DONE statuses × the g50 net lane | `docs/research/INDEX.md` — keep the doc's "139 RESERVED for g52c" note; fix its numbering paragraph (154 was registry-reserved for c33) |
| g48-net-apply-arms-1 | `237-f11-tail-fix.md` → `272-f11-tail-fix.md` **⚠ RE-ISSUED §7.3: old name now `239-f11-tail-fix.md`** (g48 self-renamed 237→239; 239=d42's; target stays 272) | 237 — W4-g48: the f11 minecart fail-tile tail-fix hunk (upstream publish) | `docs/research/INDEX.md` (also re-check the PORT_MAP minecart row cited in the add commit) — 237 belongs to e23's earlier push; g48's NEW 244/245 docs WIN their keys (§7.1) |
| g61-npc-spawn-dust | `238-g61-spawn-family-final.md` → `273-g61-spawn-family-final.md` | 238 — W4-g61 NPC spawn / teleport-in dust: FINAL (SEALED) | `docs/research/INDEX.md` — rename-chain note: 237→238→273 (237=e23, 238=d42) |
| g56-ui-void-vault | `239-void-family-bible.md` → `274-void-family-bible.md` | 239 — The VOID-FAMILY BIBLE + the g56 RETIRE verdicts | `docs/research/INDEX.md` — 239 belongs to d42's earlier push (12:55:30) |
| d53-dust-sites-projectile | `240-proj-dust-retire.md` → `275-proj-dust-retire.md` **⚠ RE-ISSUED §7.4.3: old name now `244-proj-dust-retire.md`** (d53 executed 240→244; 244 contested by g48; target stays 275) | 240 — PROJ-DUST RETIRE (W4-d53): wetVelocity census + family declaration | `docs/research/90-consolidated-findings.md`, `docs/research/INDEX.md` — 240 belongs to g58's earlier rename (12:16:37) |
| f21-net-ids-verify-tail | `239-f21b-net-apply-audit.md` → `276-f21b-net-apply-audit.md` | 239 — Net APPLY audit (W4-f21b, code-only) | `docs/research/17-netcode.md`, `docs/research/INDEX.md` — the round-1 brief's unpushed f21b doc landed late on a taken number |
| f47-liquid-shimmer-2 | `144-item-splash-keep-one.md` → `277-item-splash-keep-one.md` | 144 — the item entry/exit splash KEEP-ONE (W4-f47 × W4-e19, 2-way) | `docs/ANALYSIS.md`, `docs/research/121-doc-sweep-shimmer.md`, `docs/research/144-ARBITER-FINAL.md` (becomes `278-…` — same-lane rename), `docs/research/INDEX.md` |
| f47-liquid-shimmer-2 | `144-ARBITER-FINAL.md` → `278-ARBITER-FINAL.md` | 144 ARBITER FINAL — the splash keep-one, definitive (merger-blocker resolved) | `docs/research/144-item-splash-keep-one.md` (becomes `277-…`), `docs/research/INDEX.md` — 144 belongs to c4's orca (09:15:20) |
| f45-wld-header-2 | `137-wld-final-audit.md` → `279-wld-final-audit.md` | 137 — WLD FINAL: the full SaveWorldFlags/LoadWorldFlags walk audit (W4-f45) | `docs/ANALYSIS.md`, `docs/research/INDEX.md` — 137 belongs to g84 (09:37:33) |
| c42-wiring-device-arms | `240-c42-wiring-seal.md` → `280-c42-wiring-seal.md` | 240 — W4-c42 WIRING SEAL: research/146 re-pick verdict + wiring-family bible | (none outside self at tip) — 240 belongs to g58; c42's round-1 renames (71-fork→212, 133→232) still owed |

**Round-2 block consumed: 244–280 (37 assignments = 31 distinct losing docs +
6 same-path fork copies; 237–243 consumed by winner pushes
e23/d42/d42/g58×4). Combined with round 1: 123 assignments (151–236 +
244–280), machine-checked unique, no gaps within each block (§5). Numbers
281+ are FREE.** New lanes must NOT pick "max-on-origin + 1" or "first free
at my last fetch" — take the next number ≥281 and re-check this registry AND
a fresh all-tip sweep first (the f11/151-153, g50/154, g48/237, g56+f21/239,
d53+c42/240 and c8-d42-g61/238 incidents all came from racing picks).

**Cutoff note:** this refresh adjudicates every push visible at the final
sweep (fetch 6, 21 038 rows). Docs pushed AFTER that sweep are NOT covered —
their lanes must self-adjudicate per the rules (first-add wins; losers take
≥281 after a fresh sweep) or request a round-3 refresh.
**ROUND 3 LANDED (§7): the post-cutoff wave + three missed earlier claimants
are adjudicated there; the free floor is now 304.**

## 7. REFRESH (round 3) — winner corrections, pre-publish pushes, block 281–303

Re-scan after round 2 landed (`72226e500`, 14:47:50): **360 tips** (two new
branches: `W4-f12-buff-effects` — base docs + non-numbered `W4-*` files only,
no claims; `W4-g11-teleport-dust` — the g11 successor lane, merge-base with
`W4-g11-dust-main-sites-1` = its tip `2e978b279`), **21 172 ls-tree rows**,
**53 colliding keys** (45 bare + 8 letter-suffixed incl. the 18b fork family).
Same rules as rounds 1–2, plus one convention PINNED (it was already the
de-facto round-2 rule): **a directly-added doc contests its number at
first-add `%cI`; a self-RENAMED doc contests the NEW number at its
rename-commit `%cI`** — original content birth does not squat a fresh number
(g24 146 @10:02:44 not 08:51:01; d42 238 @12:15:55 not 09:50:22; g58 240
@12:16:37 not 10:26:59; c10 149 @10:49:53 not 10:12:03). All timestamps
2026-11-21 +02:00.

**Pre-publish executed pushes beat round-2 draft reservations** (round 2's own
principle, applied at its 14:47:50 publish): g48's `244-c28-18-send-switch.md`
+ `245-39-item-ownership-seed.md` (both add 14:30:16 `2b61530b4`) WIN 244/245;
the round-2 reservations c31→244 and d81(+g53)→245 re-issue to **300/301**.
d10's post-publish take of 246 (rename `d89380ca1` 14:52:30) LOSES to the
published 246 reservation (e23's 18b fork) → d10 takes its round-2 number
**247** (re-issued row, old filename now `246-…`).

### 7.1 New / extended / CORRECTED collision keys

| Key | Winner (keeps number) | Winner doc / ts | Losing claimants → FREE number |
| --- | --- | --- | --- |
| 65 (+1 missed) | W4-f17 (round 1) | 11-20T20:49:39 | W4-f3 `65-heightoffset-stool-seam.md` 12:36:05 `63fbeaa4f` (missed in rounds 1–2) → **303** |
| 68 (+3) | W4-f37 (round 1) | 02:48:30 | W4-f26 `68-findframe-census-addendum-7h.md` 13:03:04 `7901cd6e9` → **281**; `…-7i.md` 13:08:05 `5af99f1cc` → **282**; `…-7j.md` 14:18:55 `b89812bf7` → **283** |
| 77 (NEW) | W4-g78 | `77-wiring-attach-contracts.md` 10:35:08 `82bc71ea8` | W4-g15 `77-doc-sweep-items.md` 14:52:48 `af7d97534` → **284** |
| 91 (+7) | base trunk | `91-post-audit-landed.md` 11-12 | W4-d22 `91-d22-roster-gap-table.md` 12:33:34 `28866c7c8` → **285**; W4-d22 `91-g4-deprecated-keep-drop.md` 12:39:15 `78386ab14` → **286**; W4-d22 `91-addendum-final-receipt.md` 12:40:49 `a1a21235f` → **287**; W4-g4 `91-final-fighter-census.md` 12:56:07 `69d7737ba` → **288**; W4-d22 `91-fighter-family-bible.md` 13:58:41 `3ec231f98` → **289**; W4-d22 `91-fighter-walker-kind-obligation.md` 13:58:41 `3ec231f98` (same commit, filename tiebreak) → **290**; W4-g4 `91-seal-keep-one.md` 14:24:28 `e5915bfab` → **291** |
| 108 (+1) | W4-d37 (round 2) | 05:31:28 | W4-g8 `108-input-g8.md` 14:37:06 `1752c70df` → **292** |
| 116 (SAME-LANE FORK) | W4-g11-dust-main-sites-1 lineage | `116-teleport-reconcile.md` add `d892d8d43` 05:25:53 blob `de46cba4b` | W4-g11-teleport-dust (SUCCESSOR branch, merge-base = dust-main tip `2e978b279`) carries the SEALED superset blob `59002ecb0` (`238ccec07` 14:52:03). Same lane continuing its own doc — **NO renumber**; merger keep-one the successor SEAL blob. Its identical carries `65-dust-main-sites-g11.md` (→188) and `83-weather-reconcile.md` (→253) ride the round-1/2 g11 assignments — **execute them on the LIVE successor branch**. |
| 125 (WINNER CORRECTED) | **W4-d125** | `125-empress-sheets-ledger.md` 06:07:29 `2dfe026d2` (missed in round 1 — it predates d38) | **W4-d38 `125-d38-crossnote-atlas-addendum.md` 07:00:21 `d728002aa` — round-1 WIN REVOKED → 293**; d39→230 and e3→231 (round 1) unchanged — both later than d38 |
| 144 (WINNER CORRECTED) | **W4-e19** | `144-splash-keep-one.md` 08:45:32 `cd2fbfa10` (missed in round 2 — it predates c4's orca) | **W4-c4 `144-orca-npc692-census-input.md` 09:15:20 `479d70ad2` — round-2 WIN REVOKED → 294**; f22→264, f47→277/278 (round 2) unchanged |
| 145 (WINNER CORRECTED) | **W4-e19** | `145-16box-cross-lane-repins.md` 09:32:12 `c1586b9cb` (missed in round 2) | **W4-g89 `145-rod-teleport-union-composition.md` 09:48:08 `41dabfbc8` — round-2 WIN REVOKED → 295**; c4's 145-addendum→265 (round 2) unchanged |
| 146 (+1 missed) | W4-g24 (round 2, rename 10:02:44) | `146-wg-sounds-final.md` | W4-e19 `146-aether-biome-census.md` 12:08:42 `6627efd6b` (missed in round 2) → **296**; e20→266 / g67→267 unchanged |
| 148 (+1 missed) | W4-c4 (round 2) | 10:16:16 | W4-e19 `148-e19-research-final.md` 12:56:49 `c7a38bea5` (missed in round 2) → **297**; e20→269 unchanged |
| 18b (FORK, d83 DIVERGED) | W4-d82 | blob `21808dd38` | **d83's copy is NOT identical** (round-2 claim wrong): blob `0c9ea47d2`, same add `d4a4e7fda`, lane tiebreak d82 < d83 → **302**; e23 fork→246 (round 2) unchanged; d81+g53 pair re-issued → **301** (245 taken by g48) |
| 240 (+1) | W4-g58 (round 2, rename 12:16:37) | `240-happiness-final.md` | W4-g63 `240-proj-trail-dust-family-final.md` 14:54:20 `1c52d5ee3` → **298**; d53→275 / c42→280 unchanged. **⚠ EXTENDED §7.4: +f4 15:00:53 → 305; d53's claim moved to 244 via executed rename (re-issued 275).** |
| 244 (NEW, pre-publish) | **W4-g48** | `244-c28-18-send-switch.md` 14:30:16 `2b61530b4` (pushed BEFORE round-2 publish 14:47:50) | W4-f39 `244-f39-v18-claim-input.md` 14:32:01 `8704691a3` → **299**; W4-c31 (round-2 244 reservation, unexecuted) re-issued → **300**. **⚠ EXTENDED §7.4: +d53 executed rename 14:56:57 (contested → 275 re-issued) +g86 15:05:34 → 306.** |
| 245 (NEW, pre-publish) | **W4-g48** | `245-39-item-ownership-seed.md` 14:30:16 `2b61530b4` | W4-d81 18b-fork (round-2 245 reservation) re-issued → **301**; W4-g53 identical copy rides 301, merger keep-one, NO extra number |

### 7.2 Reconciliations, executions, single claimants

| Item | Verdict |
| --- | --- |
| g48 237→239 self-rename (`2b61530b4` 14:30:16) | 239 belongs to d42 (12:55:30) — the round-2 loss STANDS, chain 237→239→**272**; §7.3 re-issues the row with the CURRENT old filename `239-f11-tail-fix.md`. |
| d10 26d→246 self-rename (`d89380ca1` 14:52:30, POST-publish) | 246 was the published e23-fork reservation → d10 takes round-2's **247**; §7.3 re-issues the row with old filename `246-projectile-ai-91-100-final.md`. d10's **165/166 EXECUTED — CONFIRMED** (same commit). **⚠ §7.4: g43 also took 246 (15:10:56 → 307); g67's executed 146→247 rename (15:13:18) is contested by d10's published 247 reservation → d10's 247 WINS, g67 re-issued 267.** |
| g63 67-activation→199 | round-1 §3 assignment **EXECUTED — CONFIRMED** (`1c52d5ee3` 14:54:20, `199-proj-trail-dust-activation.md` at tip). |
| 237 | now SINGLE-CLAIMANT (e23 `237-bestiary-ui-final.md`) — g48 renamed away. e23 KEEPS. |
| 26i | `26i-g65-kill-keep-one.md` (g65, 14:20:10 `f6e07bb90`) single claimant — **FREE, keeps 26i**. |
| f45 137→279 | NOT yet executed at tip `6370dbd82` (still `137-wld-final-audit.md`, blob advanced) — round-2 row owed, unchanged. |
| g11 successor lane | `W4-g11-teleport-dust` is the LIVE g11 branch (dust-main tip is its merge-base). All owed g11 renames (65→188, 83-fork→253) execute THERE. 116 needs no renumber (§7.1). |
| e10 / f12 non-numbered docs | `W4-e10-voice-seal.md`, `W4-f12-closeout.md`, `W4-f12-sweep-census.md`, `W4-c16-buff-effects-wave-b.md`, `W4-g27-orb-keep-one.md` — lane-prefixed names outside the NN-key space; no collision, no action (merger dedupes the g27 copy by blob). |
| 139 / 107 | still EMPTY at all 360 tips; 139 stays RESERVED for g52c-net-master. |
| Round-2 winners re-verified | 12k f24, 18b d82, 26d d12, 26e d7, 26f f20, 64 d64, 68 f37, 70 d53, 83 d130, 84 f46, 96 d8, 108 d37, 111 d70, 122 g89, 129/130 f28, 137 g84, 143 c4, 146 g24, 147 e20, 148 c4, 149 e20, 237 e23, 238 d42, 239 d42, 240 g58, 241/242/243 g58 — ALL unchanged at the round-3 tips (only 125/144/145 corrected above). |

### 7.3 Per-lane rename-INSTRUCTION table (round 3; lanes execute on OWN branches)

Same procedure as §3/§6.3: `git mv`, update the doc's number header, update
every listed ref file (`git grep -l -F <oldname> <tip> -- docs/`, self
excluded). Old filenames are the CURRENT tip names. Rows marked RE-ISSUE
replace the named §6.3 row (do not execute both).

| Lane (branch `task/W4-…`) | old filename → new filename | doc title (first header) | refs-to-update on own branch |
| --- | --- | --- | --- |
| f26-findframe-extra-3 | `68-findframe-census-addendum-7h.md` → `281-findframe-census-addendum-7h.md` | 68 addendum — FindFrame census after f26 slice 7h (code-only) | `docs/ANALYSIS.md`, `docs/PORT_MAP.md` |
| f26-findframe-extra-3 | `68-findframe-census-addendum-7i.md` → `282-findframe-census-addendum-7i.md` | 68 addendum — FindFrame census after f26 slice 7i (code-only) | (none) |
| f26-findframe-extra-3 | `68-findframe-census-addendum-7j.md` → `283-findframe-census-addendum-7j.md` | 68 addendum — FindFrame census after f26 slice 7j (code-only) | (none) — f26's 7c/7e/7f/7g keep 201/203/205/251 |
| g15-dust-item-sites | `77-doc-sweep-items.md` → `284-doc-sweep-items.md` | 77 — DOC-SWEEP lane 5: WorldItem / world-drop items census + port | `docs/research/INDEX.md` — g15's 65→190 assignment unchanged |
| d22-npc-ai-styles-3-6 | `91-d22-roster-gap-table.md` → `285-d22-roster-gap-table.md` | 91 — D22 ROSTER-GAP TABLE: the 123 unrowed C# aiStyle-3 types | `docs/research/91-addendum-final-receipt.md` (→`287-…`), `docs/research/91-fighter-family-bible.md` (→`289-…`), `docs/research/91-fighter-walker-kind-obligation.md` (→`290-…`), `docs/research/91-g4-deprecated-keep-drop.md` (→`286-…`), `docs/research/INDEX.md` |
| d22-npc-ai-styles-3-6 | `91-g4-deprecated-keep-drop.md` → `286-g4-deprecated-keep-drop.md` | 91 — G4 DEPRECATED: the fighter-family canonical record (d22) | `docs/research/91-addendum-final-receipt.md` (→`287-…`), `docs/research/91-fighter-family-bible.md` (→`289-…`), `docs/research/INDEX.md` |
| d22-npc-ai-styles-3-6 | `91-addendum-final-receipt.md` → `287-addendum-final-receipt.md` | 91 — ADDENDUM FINAL RECEIPT: the d22r keep-both fold-list EXECUTED | `docs/research/91-fighter-family-bible.md` (→`289-…`), `docs/research/INDEX.md` |
| g4-npc-style3-fighter | `91-final-fighter-census.md` → `288-final-fighter-census.md` | 91 — FINAL: the fighter (aiStyle 3) census + the heart/star seam | `docs/ANALYSIS.md`, `docs/research/91-seal-keep-one.md` (→`291-…` — same-lane rename), `docs/research/INDEX.md` — g4's 214/216 unchanged |
| d22-npc-ai-styles-3-6 | `91-fighter-family-bible.md` → `289-fighter-family-bible.md` | 91 — FIGHTER-FAMILY BIBLE (SEALED) | `docs/ANALYSIS.md`, `docs/research/INDEX.md` |
| d22-npc-ai-styles-3-6 | `91-fighter-walker-kind-obligation.md` → `290-fighter-walker-kind-obligation.md` | 91 — THE fighter_walker_kind EXTENSION OBLIGATION (exact hunk) | `docs/ANALYSIS.md`, `docs/research/91-fighter-family-bible.md` (→`289-…`), `docs/research/INDEX.md` |
| g4-npc-style3-fighter | `91-seal-keep-one.md` → `291-seal-keep-one.md` | 91 — SEAL: the corrected keep-one (g4 vs d22 vs f35) | `docs/ANALYSIS.md`, `docs/research/91-final-fighter-census.md` (→`288-…`), `docs/research/INDEX.md` |
| g8-npc-kind-wire-1 | `108-input-g8.md` → `292-input-g8.md` | 108 — W4-g8 proto-band claims table (354–387) for the d37c/d37d matrix | `docs/research/INDEX.md` |
| d38-npc-kinds-281-350 | `125-d38-crossnote-atlas-addendum.md` → `293-d38-crossnote-atlas-addendum.md` | 125 — d38 crossnote ADDENDUM to the atlas-repack plan (research/113) | `docs/ANALYSIS.md`, `docs/research/INDEX.md` — **round-1 WIN REVOKED**: d125's empress ledger (06:07:29) predates this doc (07:00:21); fix the header's "wins 125" claim |
| c4-style44-flying-fish | `144-orca-npc692-census-input.md` → `294-orca-npc692-census-input.md` | 144 — The Orca NPC_692 census-input (W4-c4 → d35e atlas repack) | `docs/ANALYSIS.md`, `docs/research/INDEX.md` — **round-2 WIN REVOKED**: e19's splash-keep-one (08:45:32) predates the orca (09:15:20); c4's 143/148 wins + 145→265 unchanged |
| g89-aegis-fruit-use | `145-rod-teleport-union-composition.md` → `295-rod-teleport-union-composition.md` | 145 — the RoD teleport_style(1) ⨉ TeleportEffect-union composition (W4-g89h) | `docs/ANALYSIS.md`, `docs/research/INDEX.md` — **round-2 WIN REVOKED**: e19's 16box (09:32:12) predates the rod doc (09:48:08); rename-chain note 139→145→295; g89's 122 win unchanged |
| e19-liquid-quicksell-shimmer | `146-aether-biome-census.md` → `296-aether-biome-census.md` | 146 — The AETHER biome census (W4-e19 final, SHIMMER item 3) | `docs/research/INDEX.md` — e19's 144/145 WINS stand; 147→268 unchanged |
| e19-liquid-quicksell-shimmer | `148-e19-research-final.md` → `297-e19-research-final.md` | 148 — W4-e19 research FINAL (the liquid/quicksell/shimmer lane closes) | `docs/research/INDEX.md` |
| g63-proj-trail-dust-1 | `240-proj-trail-dust-family-final.md` → `298-proj-trail-dust-family-final.md` | 240 — W4-g63 projectile TRAIL dust (kinds ≤400): FINAL + family declaration (SEALED) | `docs/research/INDEX.md` — 240 belongs to g58's earlier rename (12:16:37); g63's 66 win + 199 EXECUTED stand |
| f39-weather-slimerain | `244-f39-v18-claim-input.md` → `299-f39-v18-claim-input.md` | 244 — W4-f39 v18-claim INPUT for g71c (the closeout receipt, code-only) | `docs/ANALYSIS.md`, `docs/ROADMAP.md`, `docs/research/INDEX.md` — g48's 244 (14:30:16) predates this doc (14:32:01) |
| c31-hive-rewrite | `12k-c31-jungle-residuals.md` → `300-c31-jungle-residuals.md` | 12k — W4-c31 jungle residuals census + gap-fill | `docs/ANALYSIS.md` — **RE-ISSUE of §6.3's c31→244 row** (244 taken by g48's pre-publish push) |
| d81-ui-elements-1 | `18b-element-census.md` (d81 fork copy) → `301-element-census-d81.md` | 18b — UIElement class census + the d81/d82/d83 range split | (none) — **RE-ISSUE of §6.3's d81→245 row** (245 taken by g48's pre-publish push) |
| g53-ui-achievement-screen | `18b-element-census.md` — IDENTICAL blob to d81's copy | (same header) | rename to the SAME `301-element-census-d81.md` (or drop the copy); merger keep-one with d81. No separate number. **RE-ISSUE of the §6.3 g53 ride row.** |
| d83-ui-elements-3 | `18b-element-census.md` (d83 fork copy) → `302-element-census-d83.md` | 18b — UIElement class census + the d81/d82/d83 range split | (none) — d83's blob `0c9ea47d2` DIVERGED from d82's `21808dd38` (round-2 "identical" claim corrected); same add commit, lane tiebreak d82 < d83 |
| f3-dust-player-3 | `65-heightoffset-stool-seam.md` → `303-heightoffset-stool-seam.md` | 65 — The HeightOffset family seam + the portableStool deferral | `docs/ANALYSIS.md`, `docs/research/INDEX.md` |
| d10-proj-ai-91-100 | `246-projectile-ai-91-100-final.md` → `247-projectile-ai-91-100-final.md` | 246 — W4-d10 FINAL: the projectile aiStyles 91-100 band table | (none outside self at tip) — **RE-ISSUE of §6.3's d10 26d→247 row** with the CURRENT old name; 246 is e23's published fork reservation; 165/166 EXECUTED stand |
| g48-net-apply-arms-1 | `239-f11-tail-fix.md` → `272-f11-tail-fix.md` | 239 — W4-g48: the f11 minecart fail-tile tail-fix hunk (upstream publish) | `docs/research/INDEX.md` — **RE-ISSUE of §6.3's g48 237→272 row** with the CURRENT old name; chain 237→239→272 (237=e23, 239=d42); fix the doc's numbering paragraph (its "239 = this lane's" claim is wrong — d42 owns 239); g48's 244/245 WINS stand |

**Round-3 block consumed: 281–303 (23 assignments = 21 distinct losing docs +
2 same-path fork copies [d81→301, d83→302]; g53 rides 301). Re-issued rows
carry round-2 numbers 247/272 and re-map 244/245→300/301 — no new numbers.
LIVE assignment total: 144 = 151–236 (86) + 246–280 (35, round 2 minus the
g48-consumed 244/245) + 281–303 (23), machine-checked unique, no gaps within
each block (§5 round 3). Winner-consumed numbers: 237–245 (e23 / d42×2 /
g58×4 / g48×2). Numbers 304+ are FREE.** ⚠ **EXTENDED by §7.4: 304–307
consumed (d50/f4/g86/g43), 308+ FREE.** New lanes must NOT pick
max-on-origin+1 or first-free-at-my-last-fetch — check this registry AND a
fresh all-tip sweep first.

**Cutoff note (round 3):** adjudicates every push visible at the round-3
sweep (360 tips, 21 172 rows). Later pushes self-adjudicate per the rules
(first-add wins; renamed docs contest at rename-commit time; losers take
≥304 after a fresh sweep) or request a round-4 refresh. **⚠ EXTENDED by §7.4:
the post-cutoff claimants on keys 240/243/244/246/247 are adjudicated there;
304–307 consumed, 308+ FREE.**

### 7.4 EXTENSION (round 3, post-cutoff claimants) — keys 240/243/244/246/247, block 304–307

Lean targeted re-check after round 3 landed (`51a28d09e`, 15:35:39): `git fetch
origin --prune` + `git ls-tree docs/research/` at the ~12 claimant tips ONLY
(g58 `c9af393dd`, d53 `c0dc7d37f`, g63 `1c52d5ee3`, f4 `e7ea8b24b`, f39
`8704691a3`, g48 `2b61530b4`, g86 `1bb11b0f5`, d10 `d89380ca1`, g43 `682f1b504`,
g67 `8387af9ac`, d50 `7474ce3d9`, + reservation holders e23/c31/d81/g53/d42).
Rules unchanged (§7 claim-time convention): direct adds contest at first-add
`%cI`; EXECUTED self-renames contest the NEW number at rename-commit `%cI` and
are honored ONLY IF UNCONTESTED; published registry reservations contest at
their publish time. All timestamps 2026-11-21 +02:00.

#### 7.4.1 Adjudication per key

| Key | Winner (keeps number) | Winner claim / ts | Losing claimants → FREE number |
| --- | --- | --- | --- |
| 240 | W4-g58 (round 2/3 CONFIRMED) | `240-happiness-final.md` rename `cdfe3d108` 12:16:37 | W4-g63 `240-proj-trail-dust-family-final.md` 14:54:20 `1c52d5ee3` → **298** (§7.3 row STANDS, unexecuted at tip); **W4-f4 `240-npc-dust-final.md` 15:00:53 `e7ea8b24b` (missed by the round-3 sweep) → 305**; d53's 240 claim is GONE — executed 240→244 rename `a106d13e8` 14:56:57 (adjudicated under 244 below); c42→280 (§6.3) unchanged |
| 243 | W4-g58 (round 2/3 CONFIRMED) | `243-seal-verification.md` first-add `c9af393dd` 14:08:18 | **W4-d50 `243-d50-dust-type-table-final.md` 14:59:24 `7474ce3d9` (missed by the round-3 sweep; its commit cites a "243 first free" scan that predated g58's push landing) → 304** — the §6.2 "243 single claimant" note is SUPERSEDED |
| 244 | W4-g48 (round 3 CONFIRMED) | `244-c28-18-send-switch.md` first-add `2b61530b4` 14:30:16 (pre-publish) | W4-f39 `244-f39-v18-claim-input.md` 14:32:01 `8704691a3` → **299** (§7.3 row STANDS); **W4-d53 EXECUTED 240→244 rename `a106d13e8` 14:56:57 — CONTESTED by g48's earlier 244 → rename NOT honored; d53 re-issued its round-2 number 275 with the CURRENT old name `244-proj-dust-retire.md` (§7.4.3); chain 240→244→275**; **W4-g86 `244-cursor-item-icon-seed.md` 15:05:34 `a5f6650d3` → 306**; c31's reservation stays re-issued → 300 (§7.3) |
| 246 | e23's PUBLISHED reservation (round 2 §6.3, publish `72226e500` 14:47:50) | `18b-element-census.md` (e23 fork) → `246-element-census-e23.md`, reservation live since 14:47:50 | W4-d10 EXECUTED 26d→246 rename `d89380ca1` 14:52:30 (POST-publish) → **247 re-issued** (§7.3 row STANDS); **W4-g43 `246-temple-family-seal.md` 15:10:56 `682f1b504` (landed after the round-3 sweep) → 307** |
| 247 | W4-d10's PUBLISHED reservation (round 2 §6.3, publish `72226e500` 14:47:50) | `247-projectile-ai-91-100-final.md` reservation live since 14:47:50; d10's rename-commit claim `d89380ca1` 14:52:30 also predates g67 | **W4-g67 EXECUTED 146→247 rename `8387af9ac` 15:13:18 — CONTESTED by d10's published 247 reservation (14:47:50) AND d10's earlier rename claim (14:52:30) → rename NOT honored; g67 re-issued its round-2 number 267 with the CURRENT old name `247-g67-post-union-folds.md` (§7.4.3); chain 134→146→247→267.** g67's commit cites a "registry-150b refresh" that does not exist at any tip — the authority is THIS registry; its 4-way-146 read (g24 wins) matches §6/§7 and stands |

#### 7.4.2 Final chains (post-extension state; EXECUTED = rename landed at tip)

| Lane | Final chain | Final number | State at tip |
| --- | --- | --- | --- |
| f11-wiring-arms-4 | 134/135/136 → 151/152/153 → 233/234/235 | **233/234/235** | EXECUTED+CONFIRMED (`14ca0a8d5`) |
| g58-ui-shop-happiness | 143 → 240; new 241/242/243 | **240–243 KEEP** | EXECUTED; wins 240 vs g63/f4/d53/c42, 243 vs d50 |
| d42-npc-kinds-561-630 | 143 → 238; new 239 | **238/239 KEEP** | EXECUTED+CONFIRMED |
| e23-bestiary-ui-grid | new 237; 18b-fork → 246 | **237 KEEP**, 246 owed | 237 sole claimant; 246 reservation beats d10/g43 |
| d10-proj-ai-91-100 | 26d → 165/166 EXECUTED; 26d-final → 246 → **247** | **247** | 246 rename EXECUTED but contested → re-issued 247 (§7.3), beats g67 |
| g67-weather-rain-splash | 134 → 146 → 247 (EXECUTED, contested) → **267** | **267** | re-issued §7.4.3, old name `247-g67-post-union-folds.md` |
| d53-dust-sites-projectile | 240 → 244 (EXECUTED, contested) → **275** | **275** | re-issued §7.4.3, old name `244-proj-dust-retire.md`; 70 win stands |
| g48-net-apply-arms-1 | 237 → 239 (EXECUTED, contested) → **272**; new 244/245 | **272** + **244/245 KEEP** | 272 re-issued (§7.3); 244/245 pre-publish wins beat f39/d53/g86/c31/d81 |
| c8-style66-crawling-critters | 143 → 238 (EXECUTED, contested) → **262** | **262** | owed (§6.3) |
| g61-npc-spawn-dust | 237 → 238 (EXECUTED, contested) → **273** | **273** | owed (§6.3) |
| c10-style68-waterfowl | 145 → 149 → **270** | **270** | owed (§6.3) |
| g89-aegis-fruit-use | 139 → 145 (win REVOKED §7.1) → **295** | **295** | owed (§7.3); 122 win stands |
| c4-style44-flying-fish | 144 win REVOKED §7.1 → **294**; 143/148 KEEP; 145-addendum → **265** | **294/265** | owed (§7.3/§6.3) |
| d38-npc-kinds-281-350 | 125 win REVOKED §7.1 → **293** | **293** | owed (§7.3) |

#### 7.4.3 Per-lane rename-INSTRUCTION table (extension; lanes execute on OWN branches)

Same procedure as §3/§6.3/§7.3. Rows marked RE-ISSUE replace the named §6.3
row (do not execute both). Refs via `git grep -l -F <oldname> <tip>` (whole
tree for g67/d53), self excluded.

| Lane (branch `task/W4-…`) | old filename → new filename | doc title (first header) | refs-to-update on own branch |
| --- | --- | --- | --- |
| d50-dust-type-table | `243-d50-dust-type-table-final.md` → `304-d50-dust-type-table-final.md` | 243 — W4-d50 dust-type-table RETIRE FINAL (the wiring census, the G3/G4 keep-one drop-hunks, the flag-b/flag-c receipts + the dust-type-table family declaration) | `docs/ANALYSIS.md`, `docs/research/INDEX.md` — 243 belongs to g58's earlier push (14:08:18 < 14:59:24); fix the doc's "243 first free" numbering paragraph |
| f4-dust-npc-3 | `240-npc-dust-final.md` → `305-npc-dust-final.md` | 240 — NPC dust lane FINAL: keep-one confirmation + worm-seam census + family declaration (W4-f4, 2026-11-21) | `docs/ANALYSIS.md` — 240 belongs to g58's earlier rename (12:16:37 < 15:00:53) |
| g86-altar-draw | `244-cursor-item-icon-seed.md` → `306-cursor-item-icon-seed.md` | 244 — The cursorItemIcon system: census + task-seed + owner contract (W4-g86) | `docs/ANALYSIS.md`, `docs/research/INDEX.md` — 244 belongs to g48's earlier push (14:30:16 < 15:05:34) |
| g43-biome-jungle-temple | `246-temple-family-seal.md` → `307-temple-family-seal.md` | 246 — The temple-family SEAL (W4-g43, code-only) | `docs/ANALYSIS.md`, `docs/ROADMAP.md`, `docs/research/INDEX.md` — 246 is e23's published 18b-fork reservation (14:47:50 < 15:10:56) |
| d53-dust-sites-projectile | `244-proj-dust-retire.md` → `275-proj-dust-retire.md` | 244 — PROJ-DUST RETIRE (W4-d53): the wetVelocity movement-gap census, the g2 staged-copy drop-hunk, the d53×e3 disjointness confirmation, the insurance-patch receipt, the family declaration | `docs/ANALYSIS.md`, `docs/research/90-consolidated-findings.md` (R-244-1 → R-275-1), `docs/research/INDEX.md` — **RE-ISSUE of §6.3's d53 240→275 row** with the CURRENT old name (d53 executed 240→244 `a106d13e8` 14:56:57; 244 contested by g48 14:30:16); chain 240→244→275; the doc's SEAL/numbering paragraphs + the research/70 banner + PORT_MAP/ROADMAP cites must follow the rename |
| g67-weather-rain-splash | `247-g67-post-union-folds.md` → `267-g67-post-union-folds.md` | 247 — W4-g67 post-union fold-list (for W4-d130f) | `docs/ANALYSIS.md`, `docs/research/INDEX.md` (+ the rain.rs banners' rename note, comments-only) — **RE-ISSUE of §6.3's g67 146→267 row** with the CURRENT old name (g67 executed 146→247 `8387af9ac` 15:13:18; 247 contested by d10's published reservation 14:47:50 + rename claim 14:52:30); chain 134→146→247→267; the doc's "registry-150b" citation is stale — cite THIS registry §7.4 |

**Extension block consumed: 304–307 (4 new assignments: d50/f4/g86/g43, in
claim-time order). d53/g67 re-issues carry their round-2 numbers 275/267 — no
new numbers. LIVE assignment total: 148 = 151–236 (86) + 246–280 (35) +
281–303 (23) + 304–307 (4), machine-checked unique, no gaps within each block
(§5 round-3 extension). Winner-consumed numbers unchanged: 237–245. Numbers
308+ are FREE.** Same warning: no max-on-origin+1, no
first-free-at-my-last-fetch — d50's 243, f4's 240, g86's 244, g43's 246 and
g67's 247 all came from racing scans that missed a landed push or a published
reservation.

**Cutoff note (extension):** adjudicates the claimant tips listed in the §7.4
header at this fetch. Pushes after it self-adjudicate per the same rules
(losers take ≥308 after a fresh sweep) or request a round-4 refresh.
**⚠ EXTENDED by §8 (round 4): the post-§7.4 self-pick wave on keys
244–248/281–284 is adjudicated there; 308–332 consumed, 333+ FREE.**

## 8. REFRESH (round 4, APPEND-ONLY) — post-§7.4 self-pick wave, block 308–332

Lean claimant-tip re-check only (NO full tip sweep — the §7.2/§7.3/§7.4
assignments are the AUTHORITY). **The 281–303 block was ASSIGNED by §7.2/§7.3
to specific lanes. A lane that picked a number in 281–303 WITHOUT an
assignment COLLIDES with the assignment holder and renames to 308+.** The
§7 claim-time convention applies unchanged (direct adds contest at first-add
`%cI`; EXECUTED self-renames contest the NEW number at rename-commit `%cI`
and are honored ONLY IF UNCONTESTED). All timestamps 2026-11-21 +02:00,
claimant tips as fetched for this round.

**NO SELF-PICKS: the assigned blocks 151–236, 246–280, 281–303, 304–307 and
now 308–332 are RESERVED. Do NOT take any number from them, do NOT pick
max-on-origin+1 or first-free-at-your-last-fetch. New docs take ≥333 ONLY
after checking this registry AND a fresh all-tip sweep.** Every collision in
this round came from a self-pick into an assigned block (g48's 281/282,
d30/d125/c9/d106/d9/c25/c32/f39/g8's 281, d125/f39's 282, g52's 284) or from
re-taking a number whose winner was already adjudicated (d43/g5's 244,
g40/d16's 246, d16/g68/c3/d6's 247, d3/c27's 248).

### 8.1 Simple renumber table (old → new; lanes execute on OWN branches)

Order within each key = claim time (first-add or rename-commit `%cI`);
same-commit ties broken by filename. Old names are the CURRENT tip names.
Same procedure as §3: `git mv`, update the number header, update refs
(`git grep -l -F <oldname> <tip> -- docs/`, self excluded), record the chain.

| # | Lane (branch `task/W4-…`) | Old key/doc (at tip) | New | Claim ts / why |
| --- | --- | --- | --- | --- |
| 1 | g48-net-apply-arms-1 | `281-c28-18-send-switch.md` (chain 244→281) | **308** | rename `9c776ca2f` 15:50:24 — 281 is f26's §7.3 assignment (7h); g48's 244 WIN was revoked BY ITS OWN executed rename into the assigned block → contested, NOT honored |
| 2 | d30-npc-ai-styles-65-72 | `281-band-65-72-retire.md` | **310** | add `ec89521fb` 15:42:58 — 281=f26 (self-pick) |
| 3 | d125-empress-r7-drain | `281-empress-sheets-ledger.md` | **311** | add `5a26ae348` 15:44:38 — 281=f26 (self-pick); d125's 125 WIN (§7.1) unaffected |
| 4 | f39-weather-slimerain | `281-f39-v18-claim-input.md` (chain 244→281) | **316** | rename `fec65992f` 15:53:15 — 281=f26; SUPERSEDES §7.3's unexecuted f39 244→299 row (299 is now FREE); chain 244→281→316 |
| 5 | d106-worldgen-passes-6 | `281-d106-worldgen-6-retire-final.md` | **312** | add `b78762c63` 15:53:52 — 281=f26 (self-pick) |
| 6 | d9-proj-ai-81-90 | `281-projectile-ai-81-90-final.md` | **313** | add `88677f9de` 15:55:17 — 281=f26 (self-pick) |
| 7 | c9-style67-snails | `281-snail-family-final.md` | **314** | add `9252d6f31` 15:57:35 — 281=f26 (self-pick); c9's 109 WIN unaffected |
| 8 | c25-keyboard-text-field | `281-c25-keyboard-text-field-final.md` | **315** | add `1d6e31a74` 15:58:39 — 281=f26 (self-pick) |
| 9 | c32-convert | `281-convert-tables-family-bible.md` | **317** | add `de25eddf8` 16:07:40 — 281=f26 (self-pick) |
| 10 | g8-npc-kind-wire-1 | `281-g8-kind-wire-final.md` | **318** | add `9c66afc30` 16:29:22 (tip `dd3cb0572`) — 281=f26 (self-pick); g8's 108→292 assignment unchanged |
| 11 | d125-empress-r7-drain | `282-d125-retire-final.md` | **319** | add `5a26ae348` 15:44:38 (same commit as its 281 doc; filename tiebreak) — 282=f26 (7i) |
| 12 | g48-net-apply-arms-1 | `282-39-item-ownership-seed.md` (chain 245→282) | **309** | rename `9c776ca2f` 15:50:24 — 282=f26 (7i); g48's 245 WIN revoked by its own contested rename (see row 1) |
| 13 | f39-weather-slimerain | `282-f39-seal.md` | **320** | add `1c120b550` 16:06:54 — 282=f26 (7i) (self-pick) |
| 14 | d43-npc-kinds-631-697 | `244-d43-kinds-631-697-retire.md` | **321** | add `6fbe06250` 15:55:04 — **244 = g48 WINS** (pre-publish 14:30:16, §7.1/§7.4); g48's own exit to 308 does NOT free 244 for re-takes — the key stays winner-consumed |
| 15 | g5-npc-style7-caster-doc | `244-caster-family-declaration.md` | **322** | add `f044bd315` 16:17:57 — 244=g48 WINS |
| 16 | g40-biome-marble-granite | `246-w4g40-desert-marble-final.md` | **324** | add `0e3a17852` 16:02:53 — **246 = e23's published reservation WINS** (14:47:50, §7.4) |
| 17 | d16-proj-ai-151-160 | `246-d16-research-final.md` | **325** | add `1d3c4a4b5` 16:07:49 — 246=e23 reservation |
| 18 | d16-proj-ai-151-160 | `247-kite-family-bible.md` | **326** | add `1d3c4a4b5` 16:07:49 (same commit; filename tiebreak) — **247 = d10's reservation WINS, EXECUTED** (`72431e61f` 16:05:28, §7.4 row satisfied) |
| 19 | g68-weather-wind-leaves | `247-g68-wind-seal.md` | **327** | add `375466868` 16:10:43 — 247=d10 |
| 20 | d6-proj-ai-51-60 | `247-projectile-ai-51-60-band-bible.md` | **328** | add `174c0d541` 16:14:05 — 247=d10 |
| 21 | c3-style10-cursed-skull | `247-cursed-skull-family-bible.md` | **329** | add `dd0ca4727` 16:14:34 — 247=d10 |
| 22 | d3-proj-ai-21-30 | `248-d3-boulder-family-seal.md` | **330** | add `740026af6` 16:10:40 — **248 = e26 first-add KEEPS** (verified: `cbaea2864` 15:44:26 predates both; 248 was free, valid take) |
| 23 | c27-net-entity-state-ids | `248-c27-set-time-final.md` | **331** | add `75902ca72` 16:12:49 — 248=e26 |
| 24 | g52-net-relay-host | `284-g52c-relay-host-arms-census.md` | **332** | add `0224fff80` 16:19:37 — 284=g15's §7.3 assignment (77-doc-sweep-items) (self-pick) |
| 25 | f4-dust-npc-3 | `283-npc-dust-final.md` (interim; chain 240→283→305) | **305 — ALREADY EXECUTED** | f4 self-corrected `331b29858` 16:25:06 → `305-npc-dust-final.md` at tip `bd96f08df`; 283=f26 (7j); §7.4.3 f4 row SATISFIED, no action. NOT a new number. |

Rows 1–24 = **24 NEW assignments**; together with the already-executed
f4→305 (§7.4), the live extension block is **308–332 (25 numbers)**.
§7.3's unexecuted f39→299 row is SUPERSEDED by row 4 — **299 is FREE
again**. Row 25 (f4→305) executes an EXISTING §7.4 assignment, no new
number.

**Assignment order note (281 self-pickers):** the brief's "first-add order"
is applied per the §7 claim-time convention: g48/f39's 281 docs are
self-RENAMES and contest 281 at their rename-commit times (15:50:24 /
15:53:15), interleaved with the direct adds. Chronological claim order is
d30, d125, g48, f39, d106, d9, c9, c25, c32, g8; g48's TWO renames ride one
commit and are kept ADJACENT at the block floor (**308/309**), the
remaining nine take **310–318 in claim order**: d30→310, d125→311,
f39→312, d106→313, d9→314, c9→315, c25→316, c32→317, g8→318. The 282 trio:
d125 15:44:38 → **319**, g48 rename 15:50:24 → **309** (adjacent to 308),
f39-seal 16:06:54 → **320**.

### 8.2 Key verdicts (winners KEEP; reservations vs self-picks)

| Key | Winner / holder | Verdict |
| --- | --- | --- |
| 281 | **f26 `68-findframe-census-addendum-7h.md` → 281 (§7.3 ASSIGNED)** | 10 self-pickers COLLIDE → rows 1–10 (308/310–318). f26's rename still owed at its tip `a2e43ef77`. |
| 282 | **f26 addendum-7i → 282 (§7.3 ASSIGNED)** | 3 self-pickers → rows 11–13 (319/309/320). |
| 283 | **f26 addendum-7j → 283 (§7.3 ASSIGNED)** | f4's interim 283 take self-corrected → 305 EXECUTED (row 25). No other claimant. |
| 284 | **g15 `77-doc-sweep-items.md` → 284 (§7.3 ASSIGNED)** | g52 self-pick → 332 (row 24). g15's rename still owed at its tip `af7d97534`. |
| 244 | **g48 WINS** (§7.1/§7.4, pre-publish 14:30:16) | g48 then EXECUTED 244→281 (contested self-pick) → NOT honored, g48 → 308. The 244 KEY stays winner-consumed (g48's win is recorded; its doc lives at 308). Post-win re-takes d43/g5 → 321/322. §7.3's f39→299 superseded → 316; d53→275 / g86→306 / c31→300 unchanged. |
| 245 | **g48 WINS** (§7.1, pre-publish) | g48 EXECUTED 245→282 (contested) → 309. Key stays winner-consumed. d81+g53→301 unchanged. |
| 246 | **e23 published reservation WINS** (§7.4) | g40/d16 re-takes → 324/325; d10→247 / g43→307 unchanged. |
| 247 | **d10 reservation WINS — EXECUTED** (`72431e61f` 16:05:28, `247-projectile-ai-91-100-final.md` at tip) | d16/g68/d6/c3 → 326–329 (claim-time order; d6 16:14:05 predates c3 16:14:34); g67→267 unchanged. |
| 248 | **e26 first-add KEEPS** (`248-e26-luck-final-census.md`, `cbaea2864` 15:44:26 — verified earliest; 248 was FREE, single-claimant at first-add, valid take; NOT an assignment) | d3/c27 → 330/331. g82's round-2 26e-f→248 row is superseded by this valid earlier take → g82 re-issues to a free number ≥333 at its own rename time (NOT assigned here — g82 tip `6e08d9ed1` still carries `26e-f-rope-union-final.md`, unexecuted; flag for round 5 if it picks before then). |

### 8.3 INDEX note

`docs/research/INDEX.md` collision-note updated in this same commit: §8 is
authority for keys 244–248/281–284; live assignments **173 = 151–236 (86) +
246–280 (35) + 281–303 (23) + 304–307 (4) + 308–332 (25)**; 299 returned
FREE (f39 re-issued 316); **333+ FREE, NO SELF-PICKS into any assigned
block**; g82's 248 reservation superseded by e26's valid take.

**Cutoff note (round 4):** adjudicates the claimant tips listed in §8.1 at
this fetch. Later pushes self-adjudicate per the same rules (losers take
≥333 after a fresh sweep) or request a round-5 refresh.

## 5. Verification record

### Round 1 (commit `38bfd1d22`)

- `git fetch origin +refs/heads/task/W4-*:refs/remotes/origin/task/W4-*` — 355 tips.
- Per-tip `git ls-tree` scan: 20 071 rows; number-key grouping: **32 keys with
  >1 distinct filename** = 26 bare (08, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70,
  71, 90, 91, 92, 93, 94, 95, 97, 101, 109, 125, 133, 134, 135, 136) + 6
  letter-suffixed (12g, 12i, 12j, 26d, 26e, 26f). The 28c–28f wave-claim shares
  are single-filename identical-blob carries and never entered the collision
  set.
- Winner/loser decided strictly by earliest first-add committer timestamp per
  filename per branch (`git log --diff-filter=A`), fork tiebreak by lane name.
- Blob identity checked with `git rev-parse <tip>:docs/research/<file>` for all
  6 same-path fork pairs + 4 wave-claim shares.
- Refs lists: `git grep -l -F <filename> <tip> -- docs/` per losing doc.
- No cargo/tests/rustfmt run (docs-only task, per mandate). No other branch
  touched; publication on `task/W4-d54-dust-sites-world` only.

### Round 2 (refresh, this commit)

- `git fetch origin --prune` ×6 (first attempt hit a connection reset;
  retried after 90 s per mandate) — **358 tips** each time; mid-round fetches
  caught live lane pushes: f11 `14ca0a8d5` self-correction (151/152/153→
  233/234/235), d42 `9eadc3043` 143→238 + `bac408be0` new 239, c8 `2a0b5c701`
  143→238, g61 `6e922fc60` 237→238, g48 `34029437a` new 237, g58 `cdfe3d108`
  143→240 + `30bb6e2b1` new 241 + `d0f6275a5` new 242 + `c9af393dd` new 243,
  g56 `54c63d8d4` new 239, f21 `f7bc25471` new 239 (the brief's f21b doc),
  d53 `df3355c36` new 240, c42 `2910a7209` new 240, f47
  `f80490bcf`/`18fe75f2e` two new 144 docs, f45 `d2a8276a2` new 137, f5 new
  111, d105 new 129, e19 new 147, g64 executed round-1's 196; e18/d130/g45
  tip blobs advanced (84/83/69 forks — first-add timestamps unaffected).
  The FINAL sweep (fetch 6, 21 038 rows) is the authority for §6; pushes
  after it are out of scope (§6.3 cutoff note).
- Per-tip `git ls-tree docs/research/` scans: 21 000 → 21 019 → 21 029 →
  **21 038 rows** (final); grouping: **47 colliding keys** = 40 bare (08,
  61–71, 90–97, 101, 108, 109, 111, 125, 129, 130, 133, 137, 143–149, 237,
  238, 239, 240) + 7 letter-suffixed (12g, 12i, 12j, 12k, 26d, 26e, 26f).
  134/135/136 left the set (f11 executed 233/234/235); 241/242/243
  single-claimant (g58).
- **10 same-path diverged fork families** found by blob-identity grouping
  (`git rev-parse <tip>:docs/research/<file>`): 18b (d82=d83 identical keep;
  d81=g53 identical pair; e23 diverged), 65-dust-world-sites (d54/f6),
  66-proj-trail (g63 keep / g64 EXECUTED →196), 69-liquid (g46/g45),
  70-liquid-merge-order (g47/g46 NEW), 71-wiring (e20/c42), 83-weather
  (d130/g11 NEW), 84-map (f46/e18 NEW), 109-d36 (d36/d36x), 122-persist-v9
  (g89/g88 NEW).
- First-add adjudication: `git log <tip> --diff-filter=A --format='%H|%cI' -1
  -- docs/research/<file>` for all 51 (file, lane) pairs of the new/extended
  keys + forks, plus 18 more for the mid-round waves (111/129/147/237/238/
  239×3/240×3/241/242/243/144×2/137×2).
- Refs lists: `git grep -l -F <filename> <tip> -- docs/` for all 37 round-2
  losers (g53 checked separately: no refs; c42's 240 none outside self).
- 150-at-origin-tip check: `git ls-tree origin/task/W4-d54-dust-sites-world
  docs/research/150-research-number-registry.md` → blob `cf077631f` PRESENT
  at tip `38bfd1d22` before this refresh commit; g67's `1d16ef4ea` report was
  a stale view (1d16ef4ea is 3 commits behind 38bfd1d22 on the same branch).
- **Machine check (assignments unique / no gaps):** parse the §3 + §6.3
  rename-table target numbers (`→ \`NNN-\``); assert unique set ==
  {151..236} ∪ {244..280}, count == 123 targets (g53 rides d81's 245;
  237–243 are winner numbers, not assignments), no duplicates; cross-check
  §6.1/§6.2 bold-arrow numbers against the same set (+233/234/235 confirms).
  **First run FAILED: 221 missing — round-1 §1 assigned f33's
  `93-torch-god-fold-input-addendum.md` → 221 but the §3 instruction row was
  never written. Backfilled with fresh git-grep refs.** Final run (python,
  regex over §3+§6.3 arrow-backtick targets + §6.1/§6.2 bold-arrow
  cross-check): `target rows: 123 unique: 123 expected: 123 / dups: [] /
  missing: [] / extra: [] / 6.1 bold assignments: 37 outside-block: [] /
  MACHINE-CHECK PASS: 123 unique assignments, 151-236 + 244-280, no gaps,
  no dups`.
- No cargo/tests/rustfmt/graphify run (docs-only refresh, per mandate). No
  other branch touched; publication on `task/W4-d54-dust-sites-world` only.

### Round 3 (extension, this commit)

- Prior-refresh landing check (brief step 1): `git log` showed the dispatched
  round-2 refresh UNCOMMITTED in this worktree while round 3 began; it landed
  mid-round as `72226e500` (14:47:50, pushed) — round 3 therefore EXTENDS it
  (§7) instead of re-publishing.
- `git fetch origin --prune` (connection-reset + ref-lock retries at 90 s per
  mandate) — **360 tips** (+2: `W4-f12-buff-effects` base-carry only,
  `W4-g11-teleport-dust` g11 successor). Per-tip `git ls-tree docs/research/`:
  **21 172 rows**; grouping: **53 colliding keys** = 45 bare (08, 61–71, 77,
  83, 84, 90–97, 101, 108, 109, 111, 116, 122, 125, 129, 130, 133, 137,
  143–149, 238, 239, 240, 244) + 8 letter-suffixed (12g, 12i, 12j, 12k, 18b,
  26d, 26e, 26f). 237 left the set (g48 renamed →239); 245/246 single-lane at
  scan but adjudicated vs published reservations (§7).
- **29 base-doc content forks** (single filename, base-add ancestor of
  origin/main, lane-edited blobs: keys 02–50) classified as merge-content
  forks, NOT number collisions — no renumber (same treatment as round 1).
- Rename-aware first-add: `git log --follow --diff-filter=A` for every doc
  with no direct add (f11 233/234/235, g64 196, g11 chain, c10 149, g24 146,
  g67 146, c8/d42 238, d10 165/166/246, g63 199, g58 240); rename chains read
  via `--follow --name-status` R-rows. Claim-time convention pinned (§7
  header): direct adds contest at first-add, renames contest at rename-commit.
- Winner corrections found by full re-adjudication (not just deltas): d125's
  `125-empress-sheets-ledger.md` (06:07:29) predates round-1's d38 winner;
  e19's `144-splash-keep-one.md` (08:45:32) and `145-16box-cross-lane-repins.md`
  (09:32:12) predate round-2's c4/g89 winners — all three round-1/2 winner
  calls REVOKED, losers renumbered 293/294/295. 18b d83 blob re-checked:
  `0c9ea47d2` ≠ d82 `21808dd38` — round-2 identical claim WRONG, d83 fork →302.
- Pre-publish push check: g48 `2b61530b4` (14:30:16) predates the round-2
  publish `72226e500` (14:47:50) → 244/245 WINNER takes; c31/d81+g53
  reservations re-issued 300/301. d10 `d89380ca1` (14:52:30) postdates it →
  246 stays e23's reservation, d10 →247 (round-2 number re-issued).
- Refs lists: `git grep -l -F <oldname> <tip> -- docs/` for all 23 new losers
  + 3 re-issued rows at the pinned round-3 tips (g53: no refs).
- **Machine check (assignments unique / no gaps):** parse §3 + §6.3 + §7.3
  rename-table targets, apply the §7.3 RE-ISSUE overrides (c31 244→300, d81
  245→301, d10 old-name update, g48 old-name update), assert live set ==
  {151..236} ∪ {246..280} ∪ {281..303}, count == 144, no dups, no gaps within
  blocks; winner-consumed 237–245 disjoint from assignments. Run (python,
  regex over the three tables' arrow-backtick targets, 4 RE-ISSUE overrides
  applied): `raw target rows: s3=86 s6.3=37 s7.3=25 / s6.3 after re-issue
  removal: 33 / target rows: 144 unique: 144 expected: 144 / dups: [] /
  missing: [] / extra: [] / winner/assignment overlap: [] / block 151-236:
  missing [], count 86 / block 246-280: missing [], count 35 / block 281-303:
  missing [], count 23 / MACHINE-CHECK PASS: 144 unique assignments,
  151-236 + 246-280 + 281-303, no gaps, no dups, winners 237-245 disjoint`.
  §7.1 bold-arrow cross-check: all 23 round-3 numbers present (293/294/295
  inside the REVOKED spans), none outside 281–303.
- No cargo/tests/rustfmt/graphify run (docs-only refresh, per mandate). No
  other branch touched; publication on `task/W4-d54-dust-sites-world` only.

### Round 3 EXTENSION (§7.4, this commit)

- Round-3 landing check: `51a28d09e` (15:35:39) committed AND pushed (= origin
  tip) before this extension began; worktree clean. §7.4 EXTENDS §7 in place.
- `git fetch origin --prune` (exit 0, first attempt) + `git ls-tree
  docs/research/` at the ~12 claimant tips ONLY (lean mandate, no full sweep):
  g58 `c9af393dd`, d53 `c0dc7d37f`, g63 `1c52d5ee3`, f4 `e7ea8b24b`, f39
  `8704691a3`, g48 `2b61530b4`, g86 `1bb11b0f5`, d10 `d89380ca1`, g43
  `682f1b504`, g67 `8387af9ac`, d50 `7474ce3d9`, + reservation holders
  e23/c31/d81/g53/d42 (18b/12k/246/239 presence checks).
- First-add / rename-commit `%cI` per claimant (`git log --diff-filter=A -1`
  + `git log -1` on the rename commits): g58 240 `cdfe3d108` 12:16:37, g58 243
  `c9af393dd` 14:08:18, g48 244/245 `2b61530b4` 14:30:16, f39 244 `8704691a3`
  14:32:01, d10 246-rename `d89380ca1` 14:52:30, g63 240 `1c52d5ee3` 14:54:20,
  d53 244-rename `a106d13e8` 14:56:57, d50 243 `7474ce3d9` 14:59:24, f4 240
  `e7ea8b24b` 15:00:53, g86 244 `a5f6650d3` 15:05:34, g43 246 `682f1b504`
  15:10:56, g67 247-rename `8387af9ac` 15:13:18; d42 239 `bac408be0` 12:55:30
  (re-verified for the g48 chain). Round-2 publish `72226e500` 14:47:50 = the
  reservation contest time for 246/247.
- Executed-rename honors: d53 240→244 and g67 146→247 both CONTESTED → NOT
  honored; re-issued their round-2 numbers 275/267 with current old names.
  g48 237→239 contest re-verified (d42 earlier) — §7.3 row stands.
- Executions re-confirmed at tips: d10 165/166, g63 199 (+66 keeps), g58
  240–243, d42 238/239; d53 275 and g67 267 NOT present (owed).
- Refs lists: `git grep -l -F <oldname> <tip>` for the 6 §7.4.3 rows (g67/d53
  whole-tree; others `-- docs/`).
- **Machine check:** parse §3 + §6.3 + §7.3 + §7.4.3 arrow-backtick targets,
  drop RE-ISSUED rows (§6.3: c31/d81/g53/d10/g48 + NEW d53/g67), assert live
  set == {151..236} ∪ {246..280} ∪ {281..303} ∪ {304..307}. Run (python):
  `raw target rows: s3=86 s6.3=37 s7.3=25 s7.4.3=6 / s6.3 after re-issue
  removal: 31 / target rows: 148 unique: 148 expected: 148 / dups: [] /
  missing: [] / extra: [] / winner overlap: [] / block 151-236: missing [],
  count 86 / block 246-280: missing [], count 35 / block 281-303: missing [],
  count 23 / block 304-307: missing [], count 4 / MACHINE-CHECK PASS: 148
  unique assignments, 151-236 + 246-280 + 281-303 + 304-307, no gaps, no dups,
  winners 237-245 disjoint`.
- No cargo/tests/rustfmt/graphify run (docs-only extension, per mandate). No
  other branch touched; publication on `task/W4-d54-dust-sites-world` only.

---

## §8 — ROUND 4 FINAL (Lead direct-append; the 281-block self-pick race + post-§7.4 claims)

> **§8 IS AUTHORITY for the rows below. NO SELF-PICKS: lanes EXECUTE §8 assignments
> ONLY. A fresh tips-only sweep is NOT sufficient — §7.2/§7.3 reservations are
> claims even when unexecuted (the f4-283/f26-283 lesson).**

Round-3 §7.2 assigned 281–303 to specific lanes. After publication, ~12 lanes
self-picked 281+ from tips-only sweeps (missing the reservations) and collided.
§8 adjudicates: **§7.2-assigned lanes KEEP their numbers; contested self-picks
renumber 308+ in first-add (%cI) order.** Post-§7.4 new claims also assigned here.

### §8.1 — 281-block self-pickers → 308+ (the §7.2 owner of 281 KEEPS 281)

| Lane | Self-picked doc | First-add %cI | §8 FINAL |
|---|---|---|---|
| d30 | 281-band-65-72-retire | ec89521fb 15:42:58 | **308** |
| d125 | 281-empress-sheets-ledger | 5a26ae348 ~15:5x | **309** |
| g48 | 281-c28-18-send-switch | 9c776ca2f ~15:4x | **310** |
| c9 | 281-snail-family-final | 9252d6f31 ~16:0x | **311** |
| d106 | 281-d106-worldgen-6-retire-final | b78762c63 ~16:0x | **312** |
| d9 | 281-projectile-ai-81-90-final | 88677f9de ~16:0x | **313** |
| c25 | 281-c25-keyboard-text-field-final | 1d6e31a74 ~16:1x | **314** |
| c32 | 281-convert-tables-family-bible | de25eddf8 ~16:1x | **315** |
| f39 | 281-f39-v18-claim-input (244→281 self) | fec65992f ~16:1x | **316** |
| g8 | 281-g8-kind-wire-final (uncommitted) | pending | **317** |

(§7.2's 281 owner keeps 281; the 10 self-pickers above take 308–317. Exact
first-add order re-verify at execution; the block is reserved regardless.)

### §8.2 — 282/283/244/245/246/247/248/249/284/304 post-§7.4 claims

| Key | Claimants | Winner | Losers → §8 |
|---|---|---|---|
| 282 | g48-39-item (9c776ca2f) / d125-FINAL (5a26ae348) / f39-seal (1c120b550) | §7.2 owner keeps | g48→**318**, d125→**319**, f39→**320** |
| 283 | f26 §7.3 RESERVATION (68-7j-addendum) / g59-death-bible (073203644) / f4 (renamed 305) | **f26 reservation KEEPS 283** | g59→**321** |
| 244 | g48 WINS (2b61530b4 14:30:16) / d43-244 (6fbe06250) / g5-244-caster (f044bd315) | g48 | d43→**322**, g5→**323** |
| 245 | g48 WINS (2b61530b4) / d81+g53 reservation / g65-245-kill (uncommitted) | g48 | g65→**324** |
| 246 | e23 reservation WINS / g40-246 (0e3a17852) / d16-246 (1d3c4a4b5) | e23 | g40→**325**, d16→**326** |
| 247 | d10 EXECUTED (72431e61f) KEEPS / d16-247-bible / g68-247 (375466868) / c3-247 (dd0ca4727) / d6-247 (174c0d541) | d10 | d16-bible→**327**, g68→**328**, c3→**329**, d6→**330** |
| 248 | e26-luck (a23c64933) first / d3-boulder (740026af6) / c27-clock (75902ca72) | e26 | d3→**331**, c27→**332** |
| 249 | d15-band-141-150 (869ac7edd) | sole | **249 KEEPS** |
| 284 | g52-relay-census (0224fff80) | sole | **284 KEEPS** |
| 304 | d50 §7.4 ASSIGNED / d119-housing-bible (c677b252a) self | d50 (§7.4) | d119→**333** |
| 240 | g58 WINS / g62-240-hit-dust (f5ce54743) / f30-240-systems (973ec04c4) | g58 | g62→**334**, f30→**335** |
| 243 | g58 WINS / d13-243-pet-light (1e915e254) / f24-243-room-content (706b155c4) | g58 | d13→**336**, f24→**337** |

### §8.3 — execution instructions

- Each loser lane: `git mv <old-doc> → <§8-number>-<same-slug>.md` + header
  chain-banner + ALL refs (INDEX/ANALYSIS/PORT_MAP/ROADMAP/anchors/code-comments;
  `git grep` zero-stale at pushed tip).
- **308–337 RESERVED by this §8. New docs take ≥338 after a fresh sweep AND a
  registry check. NO max+1 self-picks.**
- f4 already executed 305 (§7.4) — CONFIRMED, no §8 move.
- g63 already executed 298 (§7.3) — CONFIRMED.
- g67 already executed 267 (§7.4 re-issue) — CONFIRMED.
- d53 275 re-issue (§7.4) — OWED, execute per §7.4.3 (old name 244-proj-dust-retire).

### §8.4 — verification

- Live assignment set after §8: {151..236} ∪ {246..280} ∪ {281..303} ∪ {304..307}
  ∪ {308..337} = 86+35+23+4+30 = 178 assignments. Winners 237–245 disjoint.
- 249/284 sole-claimant KEEPS verified (no §7.2 reservation on those keys).
- Machine-check owed at next full refresh (round 5) — §8 is the interim authority.
- No cargo/tests/rustfmt/graphify (docs-only). Publication on
  `task/W4-d54-dust-sites-world` only.

### §8.5 — AMENDMENT: g13-308 executed pre-§8 → HONORED

g13's `308-dust-main-family-bible.md` (`3fe509158`, pushed ~16:31) landed BEFORE
this §8 publication (`0a658c7b3`), under the then-valid §7.4 "308+ free" rule.
Executed-push-before-authority-publication is honored (the g48-244 precedent):
**g13 KEEPS 308; d30 (the §8.1 308 assignee) shifts to 338.** All other §8.1
rows unchanged (d125 309 … g8 317). Free floor moves to **339+**.

### §8.6 — AMENDMENT 2: 304/308 post-authority self-picks

- **304**: d50 KEEPS (§7.4 assignment, published 16:07 `0426245f2`).
  g17's `304-g17-gore-world-final.md` (`3dec710dc` **16:32:44**) pushed AFTER
  §7.4 → contested → **g17 → 339**. d119 → 333 stands (§8.2).
- **308**: g13 KEEPS (§8.5, executed 16:31 pre-§8). d21's
  `308-projectile-ai-201-210-final.md` (`5a6ed468e`) contests g13's earlier
  executed push → **d21 → 340**. d30 → 338 stands (§8.5).
- **d42 238/239 KEEPS CONFIRMED** (§6.2 first-add authority; the push-time
  238→247/239→248 draft was discarded by the lane itself — zero residue
  verified `adb304ab8`). No §8 move for d42.
- Free floor: **341+**. §8.1/§8.2 all other rows unchanged.
