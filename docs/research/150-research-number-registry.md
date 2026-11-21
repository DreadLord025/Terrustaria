# 150 — Research-number COLLISION REGISTRY (W4-d54 doc-integrity, code-only)

Scan date: **2026-11-21** (worktree TR-W4-d54, branch `task/W4-d54-dust-sites-world`).
Method: fetched **all 355 `task/W4-*` remote tips**, `git ls-tree docs/research/`
per tip (20 071 file rows), grouped by leading number key (`NN` or `NNx`),
blob-identity checked, and first-add commit per filename per branch via
`git log --diff-filter=A --format=%H %cI --name-only`. Timestamps below are
**committer dates (%cI, +02:00)** of the doc's first add on the branch — the
first-push proxy; ties broken by lane name.

**Result: the 7-collision brief undercounts. The full scan finds 32 colliding
number keys** (26 bare numbers 08, 61–71, 90–95, 97, 101, 109, 125, 133–136 plus
6 letter-suffixed keys 12g, 12i, 12j, 26d, 26e, 26f), **86 rename assignments
(81 distinct losing docs + 5 same-path fork copies) across 64 lanes**. Free
renumber block assigned: **151–236, sequential** (150 = this registry; max
number seen at any tip was 144).

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
| 125 | W4-d38 | `125-d38-crossnote-atlas-addendum.md` 11-21T07:00:21 `d728002aa` | W4-d39 d39-band-351-420-census 07:18:50 → **230**; W4-e3 proj-dust-coverage 07:42:27 → **231** |
| 133 | W4-f11 | `133-f11-unique-folds.md` 11-21T05:53:19 `7f307d171` | W4-c42 fold-receipt 07:24:20 → **232** |
| 134 | W4-g67 | `134-g67-post-union-folds.md` 11-21T07:50:35 `30b39fceb` | W4-f11 light-family-unification 08:33:23 → **233** |
| 135 | W4-g90 | `135-g90-persist-folds.md` 11-21T06:39:48 `b3b82d071` | W4-f11 wires-radial 09:03:29 → **234** |
| 136 | W4-g31 | `136-tilepass-d-hunks.md` 11-21T06:47:18 `5b8899541` | W4-f11 grand-design-activation 09:21:26 → **235** |

**Free block actually consumed: 151–236 (86 unique assignments = 81 distinct
losing docs + 5 same-path fork copies).** Numbers 237+ stay FREE for future
lanes. 150 is this registry.

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
push them, they MUST take free numbers 237+ (107/139/143/144 are taken at the
tips). The scan found **29 additional colliding keys** the brief missed.

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
| f33-torch-god-favor | `93-torch-god-949-keep-one.md` → `219-torch-god-949-keep-one.md` | 93 — The 949 Torch God duplicate: W4-f33 vs W4-d19 keep-one | `docs/ANALYSIS.md`, `docs/research/93-torch-god-fold-input-addendum.md` (→`221-…`), `docs/research/INDEX.md` |
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

## 5. Verification record

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
