# 367 — Merger order DAG (Lead-assigned, W4-d54; the wave plan + gates + cleanup)

Status: FINAL PLAN — doc-number 367 per registry-150 §9.2 (Lead-assigned to
d54; reserved row honored here). Sources read LEAN, exactly three:
research/108 §X0–X5 FINAL (`7bfd5a450`, TR-W4-d37), research/113 §17.5 only
(d35 HEAD, the 7-step lazy-load contract), research/150 §9 table (d54 HEAD,
the doc-number dedupe authority; §10 rules compose per §10.3). Everything
else below is the Lead's dispatch brief (wave members, gate flags, hashes);
where a brief flag refines a 108 adjudication the flag is stated as the
merger-side instruction. Code-only lane: no cargo/tests/rustfmt/graphify.

---

## §1. Merge waves (the DAG)

### Wave 1 — the roster spine (108 §X5 order, MANDATORY sequence)

**d34 → d38 → d37 → d40 → d42 → d35-renumber-LAST → small lanes + new
claimants.** Per 108 §X2/X3 (scripted `_d37_e8s_plan.py`): **395 owned
kinds / 395 distinct final ids / max 827 (i16-safe)**, zero clash, zero
base clash. Tables: **59 moves / 336 keeps / 153 drops**. Post-merge pins
re-asserted at EACH splice: `ALL_KINDS`/`to_u16`/`from_u16`/`s1_f_3` =
**665**, ENTRIES = **583** (all net ids distinct), `STATIC_SHEET_NAMES`
NPC rows = **593**, `BESTIARY_NPC_SHEET_IDS` **440 surviving → 517
target** (77 real gaps, X4 fill owners), SFX cues 117. The bijection gate
+ `the_protocol_id_space_round_trips_every_kind` guard every splice.

- **d34** first (266-280 keep; Crab 282→791, DungeonGuardian 283→792;
  PinkJellyfish/Antlion/SpikeBall drop to g8@745/746/747). Its phase-4
  281-285 squats are resolved BEFORE d38 lands its anchor.
- **d38** second (281-337 anchor KEEPS — witness-pinned RaggedCaster..
  DiabolistRed).
- **d37** third (354-399 ZERO-MOVE, byte-identical).
- **d40** fourth (430-488 minus CrimsonGoldfish — c4's impl survives
  @465; MothronEgg 478 own-entry fix rides).
- **d42** fifth (executed 578-630 splices clean once its **11 drops**
  land: 7 dragonflies → d33@595-601, 4 fairy/balloon → d33@583/584/585/594).
- **d35 LAST** (E7): executes its **+269 order-preserving renumber
  429-461→698-730** against the settled roster; ENTRIES wholesale from
  its splice-repaired tip. LAST by design — never earlier.
- **Small lanes** splice with their X3 tables: c3 keeps WaterBoltMimic
  694; c4 keeps 406/465/509/581/587/592/607/615 (drop 5); c7/c8/c9/c10
  move contested kinds to 735-741/731-734; d23 AngryTrapper→743,
  SporeBat→634, QueenSlimeMinionPurple→660 (drop 19); d120 towns→763-766
  (lane retired, merge-wave); d125 executed 636/742 keep; g5 keeps
  LibrarianSkeleton 693 + RuneWizard 827; g8 executed 745-762 keep (drop
  17); d33 fairy/balloon re-pin 583-585/594; e21 PartyBunny→540; f42
  Tumbleweed→546; d9 keeps AncientCultistSquidhead 521; d36/d39/d41/e26/
  g10/f47 keep executed ids; c7b/d103/g62/d11 drop wholesale (wiring
  composes onto the canonical kinds).

**f18 = ZERO rows.** All 20 f18 ports DROP (verify lane, X1e/X3): the 7
critters to c7/c8/c9/e26/c10, ArmedZombie to d40@430, and the 7 d39-band
kinds to **d39@817-823** (Butterfly/ScorpionBlack/Scorpion/Grasshopper/
ChatteringTeethBomb/CultistArcherBlue/CultistArcherWhite — R7
self-consumed `048c74f8f`). **Splice f18's AI/wiring onto d39's kept
kinds** — the drop removes variant + to/from arms + ALL_KINDS + ENTRIES +
stat arms, NOT the AI modules. **style65 keep-one (Lead merger flag):
KEEP f18's module — it carries NO InWorld gate** (108 §X1e preferred
c10's `ai_065_butterflies` as the superset; the Lead's wave instruction
resolves the keep-one to f18's variant with the no-InWorld-gate
difference explicit — re-verify against `NPC.cs` aiStyle 65 at the
splice and record the verdict in the chain-banner).

### Wave 2 — core systems (after the roster spine settles)

| Order | Family | Contract / pin |
|---|---|---|
| 2a | **persist** | 103 §L3-ARBITER **107-byte** record + 132 + 92-D5 compose; the arbiter verdict is the save-format authority — no lane re-opens it |
| 2b | **liquid** | 69 + 70 + 71-A9 (the g46 SEAL hunks doc renumbered →374 per registry §10; 69 win + 70-fork→252 unchanged) |
| 2c | **atlas — LAST in wave 2** | 113 §17.5 7-step lazy-load contract, MANDATORY: the **593-sheet NPC union OVERFLOWS one 8192 page** (base 7256 + Glow_362 → lazy union ≤7447 only AFTER the NPC rows leave `STATIC_SHEET_NAMES`). Steps in order, each witnessed: (1) tr-render multi-page `SheetRect.page`/`Atlas.pages`/boundary-split `build_ref`; (2) per-page `uv_rect`/`sheet_frame_uv`/`BgTex.page`; (3) `atlas_size`→`page_sizes` mechanical sweep; (4) admit the **707-name** deferred/forecast census; (5) re-measure every page ≤8192, lazy union one page ≤7447; (6) per-world bg pages + `MAIN_PAGE_COUNT` runtime uniform at the `push_font_glyph` chokepoint (`sprite.rs:1088`); (7) per-world NPC pages — the 269 `NPC_<id>` rows → `npc_sheet_names_for_world` keyed by **`texture_net_id` (NOT net_id)**, bestiary superset at world load, `NPC_Head_*` rows stay eager. The **1580 confirmed** sheet pin (d40 `f4355d380`) rides step 4's census; X4's 77 slot-gap appends ride this wave (d40's unpushed co-lane sheets phase 22 slots included) |

### Wave 3 — behavior families (parallel-safe within the wave; each splices keep-one per its bible)

| Family | Members / merger instruction |
|---|---|
| gore | 111 + 351 + 339 + 364 (gore-sites/gore-world/proj-gore finals compose) |
| kill | 26i + 346 — **29 hunks**; 503 fixed `16c7e9d8a` |
| wiring | 133 §8-c42 (`2910a720`) + e20-266 re-pick (`6d1754719`) |
| map | 84 §10.4 **steps 1-4 are MERGER-SIDE**; step 0 DONE `5c88f1901` |
| splash | 144-v2 + 151-shimmer (f47's v2 doc renumbered →379 per §10; 144 belongs to e19) |
| rope | 26e-f |
| pets | 138 §12.2 + 360 + 368-splice (g10 pets-final + f43 pet-arrival) |
| journey | 344 (c24 journey-family-final) |
| addlight | 366 + e6-ANALYSIS-86/21 |
| net | 359 (g51 net-te-family-seal) |
| loadout | 362 (g55 loadout-family-final) |
| dodge | d115 (`cc985584c`) |

### Wave 4 — UI / net-glue / small lanes

The remaining UI lanes, net glue and small single-doc lanes splice last,
each with its registry-deduped filename (§6) and its own keep-one
receipt. The d54 dust union (131 master status, the orch routers e4 +
d54j/370 — ONE composed dispatch, never both routers for one kind) rides
here per 131's keep-one symbol checklist.

---

## §2. Known merge-gate flags (check BEFORE the relevant splice)

1. **d43 Owl needs d42** — 343 §10.5: d43's Owl wiring depends on d42's
   band; d42 must be in before d43's rows splice.
2. **d32-85 arm unreachable** — extend per 467 + 348 (d32's band-85-93
   retire): the 85 arm is unreachable until the extension lands.
3. **:86532 eye-pop caller** — the eye-pop dust caller seam must be
   composed, not dropped, at the gore/kill splice.
4. **f25 hash RE-DERIVE** — f25's hash pins must be re-derived on the
   merged tree; do not carry the branch-local values.
5. **world-hash sequential re-pins** — `SEED42_512 = 124297949`,
   `DEFAULT = 3647283075`; earlier pins are **stale-by-design** (each
   wave re-pins sequentially; re-derive after the LAST world-affecting
   splice, not per-lane).
6. **anchors canary** — `crates/tr-core/build.rs` fails the build when a
   marker disappears; a canary failure = a LOSSY merge. Never delete
   `anchors.txt` rows to silence it; reland per AGENTS.md recovery.
7. **e21 15.5px centre-anchor** — KEEP d3's arm (the wiring-tail splice
   must not swap the centre-anchor maths).
8. **g79-g86 MouseOversClear** — keep ONE implementation.
9. **g81 dedupe** — `18e53dd0f` is the dedupe receipt; compose, don't
   re-port.

---

## §3. The single gate

```
cargo test --workspace -- --skip migrates_version_2_kinds_to_item_ids
```

on the MERGED main (judge only by the `test result: ok` lines — cargo may
exit 1 on success in this project). Baseline: **`051600da8`**. Local
primary **`a725acb0`** ff-syncs (`git merge --ff-only origin/main`) at
merger start. **NEVER force-push.** Merges are `git merge --no-ff` (the
banned forms — `commit-tree`, `read-tree`, `-s ours`, `-X ours/theirs` —
stay banned per MAIN-RULES). After EVERY push: `git fetch origin` +
`git merge-base --is-ancestor <merge> origin/main` (exit 0) + a SYMBOL
check from the merged change (`git show origin/main:<file>` grep), not
just the commit — the documented silent-revert failure mode.

---

## §4. Fix-sub-branches (size by error count)

When the gate goes red after a wave: capture the FULL error list, cluster
by subsystem/file, and cut ONE fix-sub-branch per error cluster (sized to
the cluster, not one mega-fix and not one-per-error). Each sub-branch:
fix → re-run the §3 gate → merge --no-ff → ancestry+symbol verify.
Re-gate after EVERY sub-branch lands; the wave is not closed until the
gate is green on the merged main and the wave's count pins (§1) re-assert.

---

## §5. Cleanup checklist (after the last wave is green)

1. **Delete task branches local + remote, ONE PER COMMAND** (a batch
   fails whole if any ref is already gone): verify
   `git log main..<branch> --oneline` = 0 first; then `git branch -d
   <branch>`; `git push origin --delete <branch>`; `git fetch --prune`.
   Never delete another agent's in-flight branch.
2. **Worktree junctions FIRST**: `cmd /c rmdir <worktree>\assets` (and
   `decompiled`, `decompiled-libs`) BEFORE `git worktree remove` — a
   recursive remove through a junction eats the primary's trees. Then
   `git worktree remove <path>` (from ANOTHER checkout) + `git worktree
   prune`.
3. **Desktop TR-\* dirs** — remove ALL, including **TR-W4-d34seal** (the
   369 seal worktree, registry §9.2 cleanup-phase note).
4. **Temp insurance patches** — the `*.patch` insurance copies are
   DELETE-AT-CLEANUP; no stray `.rs`/`.patch`/screenshots outside
   `docs/screenshots/` remain on the Desktop.
5. **Stray refs audit** — `git branch -a` + `git for-each-ref` sweep for
   leftover backup/stale-index/RELAND refs; delete only what is fully in
   main.
6. **Only-main final audit** — `git branch -a` shows only `main` +
   `origin/main`; `git worktree list` shows only the primary (on main,
   clean, pushed); primary `git status --porcelain` empty.

---

## §6. Doc-number dedupe — registry §9/§10 is THE authority

The merger dedupes `docs/research/NN-*.md` filenames by
**`150-research-number-registry.md` §9 (UNIFIED FINAL TABLE, Lead —
308-370, §9.4 EXECUTION COMPLETE 49/49) + §10 (round-5, 371-381)**, NOT
by per-lane claims: where any earlier registry section disagrees, §9/§10
win (§9.3: "Merger dedupes filenames by THIS table. Chain-banners must
cite §9"). Round-5 keys the merger must honor at splice time:
64=d64 (c26→376), 67=d54 (f3→378), 71=e20 (g46→374), 75=g37 (f32→380),
78=c40 (g78→375), 83=d130 (c14→371), 26d=d12 (g1→373, g2→377), 26f=f20
(d78→372), 144=e19 (f47-v2→379), 249=d15 KEEPS (f18 §6.3 assignment
VOID → 381 re-issue). Same-name shared docs follow registry §4's merge
rules. **Free floor 382+ — NO SELF-PICKS**; new docs need a Lead
assignment appended to §9. This doc: 367, Lead-assigned (§9.2 row
"367 | d54 | 367-merger-order-dag (Lead-assigned, in flight) | reserved"
— executed by this landing).

---

## §7. 40-bible citation index + lane corrections (append, W4-d54)

Recorded from the original DAG agent's verified 40-bible read
(`docs/research/40-port-done-verification.md` family coverage), then
independently re-pinned here via `git ls-tree`/`git show` on the exact
commits named below. APPEND ONLY — §1–§6 above stand unchanged; where
this section refines a §1 pin it is stated as a delta, not a rewrite.

### §7.1 Lane corrections (where a bible actually lives)

| Doc | Correction | Verified at |
|---|---|---|
| research/135 | Lives on **g90** (`task/W4-g90-artisan-bread-use`) — `135-g90-persist-folds.md` | tip **`43fee29a4`** |
| research/95 | Lives on **c42** (`task/W4-c42-wiring-device-arms`) — `95-wiring-master-reconcile.md` | tip **`69148794c`** |
| 26g | Lives on **d19** — `26g-clouds-keep-one.md` (NOT on the 26 audit's home lane) | `task/W4-d19-proj-ai-181-190` |
| 26h | Lives on **d18** — `26h-projectile-ai-171-180-census.md` | `task/W4-d18-proj-ai-171-180` |
| 92-D5 | = **g71**'s `92-doc-sweep-bg.md` **§1-SEAL.1** (the persist D5 fold rides the bg doc-sweep, not a standalone 92-D5 file) | `task/W4-g71-bg-layers-2` |
| A9 liquid folds | Live on **g45**'s copy of **69** (`69-liquid-reconcile.md`) — merge 69 from g45, not from g46/g47 | `task/W4-g45-liquid-lava-honey-delays` |
| f10-delete-receipt | = **f10's tip commit `afea4c0fc` ITSELF** — no standalone receipt file exists (ls-tree confirms); cite the commit | `task/W4-f10-gore-sites-3` tip `afea4c0fc` |

### §7.2 Re-pin deltas at merge (tips moved since the original DAG read)

| Lane | Old pin → NEW pin | Why |
|---|---|---|
| f46 | `77ff32040` → **`5c88f1901`** | map step-0 EXECUTED (84 §10.4 step 0 done, per §1 wave-3 map row) |
| d3 | `5ff9fe983` → **`c23b7b013`** | 331-rename landed on the lane |
| d125 | `5a26ae348` → **`5f8569945`** | 309/319-renames landed on the lane |

All six hashes resolve locally (`git cat-file -e` exit 0); the three NEW
pins are the current `origin/task/...` tips (rev-parse verified). The
merger uses the NEW pins; the old ones are cited for chain-history only.

### §7.3 Liquid docs-tail (the 70 union the merger must expect)

- **g46 `a4d49b355`**'s copy of `70-liquid-merge-order.md` ends at the
  **§7/R-12-5 settle-drivers addendum** (verified: last heading
  "R-12-5 settle drivers addendum (g47 continuation)").
- **g47 `e97b7cb22`**'s copy of 70 carries **§8–§16** (verified headings:
  §8 g46d re-fold obligation … §15 LIQUID SEAL … §16 SEAL-VERIFY), incl.
  the **A9 row-9** material.
- The **g46e tail WIP (+314 lines, §13–15)** is being committed by a
  SEPARATE agent — the merger expects the **UNION** of g46's tail + g47's
  §8–§16 + the g46e commit, not either copy alone. Do not pick-one the 70
  copies; compose per registry §4 shared-doc rules.
- Note: g46's tip has since moved to `4bcbdd645` (the 71→374 MICRO
  RENAME per registry §10.2 — `a4d49b355` is an ancestor, verified); the
  70-tail content pin above is at `a4d49b355` as recorded.

### §7.4 Independently re-verified gate flags (evidence pins for §2)

| Flag (§2 ref) | Evidence | Re-verified |
|---|---|---|
| d40 OVERFLOW **+1580** row | d40 **`f4355d380`** ANALYSIS section (commit stat: `docs/ANALYSIS.md` +174; message pins "+all-56 d40 9772 (+1580)") — rides wave-2c step 4 census | commit show ✓ |
| g62 **:82045 emit** | **`428e93ecb`** = g62 tip; "Emit the 667 live 3-gore twin at :82045 (the fifth hit-half site, e5 research/351 §4)" | commit show ✓ |
| c42 seal | **`2910a7209`** ancestor of c42 tip (`merge-base --is-ancestor` exit 0) — 133 §8-c42 per §1 wave-3 | ancestry ✓ |
| e20 repick | **`6d1754719`** ancestor of e20 tip (exit 0) — the 266 re-pick per §1 wave-3 | ancestry ✓ |
| Owl needs d42 (§2.1) | **343 §10.5** on d43 (`343-d43-kinds-631-697-final.md`, heading "10.5 The one compile-risk … merge gate") | doc show ✓ |
| 85-arm extend (§2.2) | **348 §2c-G2** on d32 (`348-band-85-93-retire.md`: the DeadlySphere **467** panic-hazard row — NOTE: 467 here is the KIND id, not a doc number) | doc show ✓ |
| :86532 eye-pop (§2.3) | **111 §7.3** (`111-gore-final-verification.md` on f5: the `:86532` 133 WanderingEye eye-pop latch row) | doc show ✓ |
| e21 15.5px (§2.7) | **365 §4** (`365-e21-wiring-tail-seal.md` on e21: "Receipt 4 — the boulder aiStyle-25 rows are d3's … 15.5px" keep-d3 seam) | doc show ✓ |

### §7.5 Family → bible index (the §1 waves, one row per family)

Filenames verified by lean `git ls-tree`/`git show` on the named lane
ONLY where §1 did not already name the file. Lanes whose tables live in
the roster master carry 108 as their bible (no per-lane doc exists —
ls-tree confirmed for d34/d37-band lanes).

| Wave | Family / lane | Bible doc(s) | Lane branch (origin/task/…) |
|---|---|---|---|
| 1 | roster master (X0–X5) | **108**-roster-master-reconcile | W4-d37-npc-kinds-211-280 |
| 1 | d34 (266-280 keep) | 108 §X tables (no per-lane bible) | W4-d34-npc-kinds-211-280 |
| 1 | d38 (281-337 anchor) | 108 §X + **293**-d38-crossnote-atlas-addendum | W4-d38-npc-kinds-281-350 |
| 1 | d37 (354-399 ZERO-MOVE) | **108** | W4-d37-npc-kinds-211-280 |
| 1 | d40 (430-488) | **345**-d40-band-421-490-seal + ANALYSIS @`f4355d380` | W4-d40-npc-kinds-421-490 |
| 1 | d42 (578-630) | **238**-d42-roster-sheets + **239**-d42-final-keep-one-and-band | W4-d42-npc-kinds-561-630 |
| 1 | d35 (renumber-LAST) | 108 §X (E7) + **113**-atlas-repack-plan | W4-d35-npc-kinds-71-140 |
| 1 | d43 (Owl gate) | **343**-d43-kinds-631-697-final §10.5 | W4-d43-npc-kinds-631-697 |
| 1 | d32 (85-93 retire) | **348**-band-85-93-retire §2c-G2 | W4-d32-npc-ai-styles-85-93 |
| 1 | small lanes c3/c4/c7-c10/d23/d120/d125/g5/g8/d33/d9/d36/d39/d41/e26/g10/f47/e21/f42 | 108 §X3 per-lane tables (keep/move/drop); e21 tail = **365** §4; d125 NEW pin `5f8569945` (§7.2) | respective W4-* lanes per §1 |
| 1 | f18 (ZERO rows) | 108 §X1e/X3 + **381**-npc-kinds-351-697-census (249 assignment VOID per §6) | W4-f18-npc-kinds-verify-2 |
| 2a | persist | **103**-persist-v7-layout §L3-ARBITER 107-byte (g88's copy) + **132**-persist-master-map (f44's copy) + **92-D5** = g71 92-doc-sweep-bg §1-SEAL.1 (§7.1); 135 folds on g90 (§7.1) | W4-g88-life-fruit-use / W4-f44-save-plr-2 / W4-g71-bg-layers-2 / W4-g90-artisan-bread-use |
| 2b | liquid | **69**-liquid-reconcile (A9 folds on **g45**, §7.1) + **70**-liquid-merge-order (UNION tail, §7.3) + **71**-liquid-seal-hunks → **374** per §10 (g46) | W4-g45-liquid-lava-honey-delays / W4-g46-liquid-evaporation / W4-g47-liquid-skip-settle |
| 2c | atlas (LAST in wave 2) | **113** §17.5 7-step lazy-load | W4-d35-npc-kinds-71-140 |
| 3 | gore | **111**-gore-final-verification (f5's copy; §7.3 :86532 row) + **351**-gore-sites-family-bible (e5) + **339**-g17-gore-world-final + **364**-proj-gore-family-final (g16) | W4-f5-dust-proj-3 / W4-e5-gore-sites-2 / W4-g17-gore-world-sites / W4-g16-gore-projectile-sites |
| 3 | kill | **26i**-g65-kill-keep-one + **346**-g65-kill-death-dust-family-final (29 hunks; 503 fixed `16c7e9d8a`) | W4-g65-proj-death-dust |
| 3 | wiring | **133**-fold-receipt §8-c42 (`2910a7209`) + **95**/280 seals (c42, §7.1) + **71**-wiring-device-reconcile (e20; 266 re-pick `6d1754719`) + **365** tail (e21); f11 = 133-f11-unique-folds | W4-c42-wiring-device-arms / W4-e20-wiring-device-arms-2 / W4-e21-wiring-device-arms-3 / W4-f11-wiring-arms-4 |
| 3 | map | **84**-map-reconcile §10.4 (steps 1-4 MERGER-SIDE; step 0 done `5c88f1901` = f46 tip, §7.2) | W4-e18-minimap-reveal-queue / W4-f46-minimap-reveal-2 |
| 3 | splash | **144**-splash-keep-one (e19) + **151**-shimmer-family-seal (f47) + f47-v2 → **379**-ARBITER-FINAL-v2 per §10 | W4-e19-liquid-quicksell-shimmer / W4-f47-liquid-shimmer-2 |
| 3 | rope | **26e-f**-rope-union-final (on g82) | W4-g82-rope-coil-draw |
| 3 | pets | **138**-powerup-master-reconcile §12.2 + **360**-g10-pets-final + **368**-pet-arrival-family-final | W4-g10-npc-kind-wire-3 / W4-f43-town-spawn-2 |
| 3 | journey | **344**-journey-family-final | W4-c24-journey-ui |
| 3 | addlight | **366**-addlight-family-bible (g18's copy) + e6-ANALYSIS-86/21 (ANALYSIS rows, no standalone file) | W4-g18-addlight-main-1 / W4-e6-addlight-proj-2 |
| 3 | net | **359**-g51-net-te-family-seal | W4-g51-net-apply-arms-4 |
| 3 | loadout | **362**-g55-loadout-family-final | W4-g55-ui-loadout-save |
| 3 | dodge | **97**-doc-sweep-dodge (d115 `cc985584c`) | W4-d115-dodge-family |
| 3 | hit-dust (g62) | **334**-g62-hit-dust-final (renamed 240→334 per registry §9; :82045 emit `428e93ecb`, §7.4) | W4-g62-npc-hit-dust |
| 4 | dust union (d54) | **131**-dust-master-status + this **367**; ONE composed router (e4 + d54j/370), never both | W4-d54-dust-sites-world |
| 4 | UI / net-glue / small | per-lane registry-deduped docs per §6; g81 dedupe `18e53dd0f` (§2.9) | respective W4-* lanes |

Citation discipline: every "lives on" row above was re-checked against
the named branch's tree or the named commit — not carried from memory.
The 40-bible read that seeded this index is recorded in the Lead's
dispatch; where this section and a lane's own chain-banner disagree on a
filename, registry-150 §9/§10 still wins (§6).
