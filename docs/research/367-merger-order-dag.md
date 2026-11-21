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
