# 91 — ADDENDUM FINAL RECEIPT: the d22r keep-both fold-list EXECUTED + the fighter-family merger instruction

W4-d22 retire phase R4 (FINAL), branch `task/W4-d22-npc-ai-styles-3-6`.
Code-only lane: no cargo/tests/rustfmt/graphify invoked. This is the
closing receipt for g4's `research/91-addendum-d22r-fold-list.md`
(blob on `origin/task/W4-g4-npc-style3-fighter` @ `a9369ec38`) — the
keep-both contract the g4 lane published for this branch's RESUME.

## 1. The addendum §7 merge checklist — every box CLOSED

| # | contract item (addendum §7) | status | evidence (this branch) |
| --- | --- | --- | --- |
| 1 | Keep `Npc::target_zone_graveyard` + the sim pre-pass (P5) | DONE | `npc.rs:1732`/`:3102`; `world_sim/npcs.rs:4389-4393` — reconcile phase 1 `0cfd08ac` |
| 2 | Thread `rng` into `fighter_obstacle_scan_g` + the zombie entry; `flag25 = false` → the pre-stamp && `next_max(60)` (P5) | DONE | `npc_ai_fighter_scan.rs:271` (the zombie entry, rng-threaded) / `:302` (the general entry) / `:473` (the ONE shared flag25 line) — `0cfd08ac` |
| 3 | Keep the `fighters_not_discouraged` pre-stamp read + the `update_zombie` chain term (P5); the core inherits them | DONE | `npc_ai.rs:294-302`; `npc.rs:9307`; `npc_ai_style3.rs:115` (the core calls the shared gate — no local re-fold) — `0cfd08ac` |
| 4 | Keep the P6 generic-walker confused arms (non-claimed riders) | DONE | `npc_ai.rs:100-102` (`walker_step`) / `:192-194` (`walker_step_ladder`) + the witness `:1400` — reconcile phase 2 `4150d374`; re-verified at the retire tip (R3 §2) |
| 5 | Keep the P9 `debuff_immune_ids` rows + `whip_immune` (do NOT re-fold to `&[]`); keep the corrected P3/P6 witnesses | DONE | `npc_buffs.rs:230`/`:277`/`:281`/`:408`/`:533-536` + the census witness `:1173`; the P3 shooter witness repointed to RayGunner 382 (`npc.rs:20988`), the P6 ladder twin to VortexSoldier 429 — reconcile phase 3 `e6c83bef` |
| 6 | Keep d22's GoblinPeon door-smash drain (distinct from the open) | DONE | `world_sim/npcs.rs` (the drain's `NpcKind::GoblinPeon` KillTile arm, `:60049-60056`) — d22 style-3 `60712154`, kept through the reconcile |
| 7 | Keep the residual blood-moon census correction + the 489 normal-night witness | DONE | `npc_ai_fighter_scan.rs` (the corrected DOOR_BASH_TYPES note) + `w4_g4_the_blood_zombie_opens_doors_on_a_normal_night` (`:803`) — `0cfd08ac` |
| 8 | Update d22's module header: the graveyard zone WIRED, draw-free outside / one conditional Next(60) inside | DONE | `npc_ai_style3.rs:22-31` + `npc_ai_fighter_scan.rs:30-51` — `0cfd08ac`, re-updated by R1 (the moan position) |
| 9 | The four P5 witnesses + the P9 census witness + the residual 489 witness all stay (written not run; the merge runs them) | DONE | all present at the retire tip (R3's KEEP table cites each); the graveyard end-to-end witness re-derived by R1 to the moan-first C# order |

The addendum's §1-§6 fold instructions beyond the checklist (the P4
tall-gate drain, the P1 banner lane, the P2 summoner phases, the P3
fighter arms, the §4 door census, the §5 knockback census) all landed
in reconcile phases 1-2 (`0cfd08ac`/`4150d374`); the §6 coin/star
cross-notes stay f35's lane (NOT wired here — the contract says
cross-note only). The full per-symbol adjudication:
`91-g4-deprecated-keep-drop.md` (R3).

## 2. The retire deltas (beyond the reconcile)

* **R1 `fb99a047`** — the S1-E-21e moan-position remainder CLOSED: the
  moan emitter moved from the sim tail to the pre-slot head block (the
  C# `:57338` position, BEFORE the machine's draws — the graveyard
  bash Next(60) included). The addendum §1's draw-position note ("the
  bash draw precedes the moan — the remainder extended by one
  conditional draw") is thereby RETIRED: the port now rolls
  moan-then-bash, the C# order. The end-to-end witness replays it.
* **R2 `28866c7c`** — the 123-roster-gap table published
  (`91-d22-roster-gap-table.md`, the d37d-E6 input) with two census
  composition corrections (590+591 both rowed at the `:16666`
  ||-gate; 368's `:53412` is a runtime town-machine flip, not a roster
  row) and the 13-lane pushed-tip claim probe (75 ROWED / 48 open).
* **R3 `78386ab1`** — the g4 branch DEPRECATED as a merge source; the
  final KEEP/DROP symbol table; the P6 generic-walker flips verified
  landed; d22 canonical.

## 3. The fighter-family merger instruction (FINAL)

> **SUPERSEDED IN PART by the FIGHTER-CORRECTION** (the g4 seal
> `e5915bfab` consumed): the "merge d22 only" instruction below is
> corrected to **merge d22 + f35 (`1abae9ade`), never g4 wholesale,
> cherry g4's two unique hunks** — f35 owns the DropHeals heal seam
> (ported FIRST, superset `b72cdfbfc`/`00a5b27db`; g4's duplicate
> retired by its own seal); the R4 "g4 no-delta" probe was stale
> (g4's tip was `55840d9a`). The corrected record:
> `91-fighter-family-bible.md` §0/§2 (FINAL). Items 3-6 below stand.

For the Lead / the merging agent:

1. **Merge `task/W4-d22-npc-ai-styles-3-6` only.** It carries: the
   aiStyle-3 axis core (22 plain walkers, `npc_ai_style3.rs`), the
   generalised scan + the GoblinPeon smash, the styles 4/5/6 census
   fixes (`c1d26278`/`204e0651`/`b24977d9`), the whole g4 keep-both
   fold (`0cfd08ac`/`4150d374`/`e6c83bef`/`bcd42ddd`), and the retire
   (`fb99a047`/`28866c7c`/`78386ab1` + this receipt).
2. **Never merge `task/W4-g4-npc-style3-fighter` after d22.** Every
   KEEP symbol is already in d22 (R3's table); a g4 merge would
   conflict-revert the union scan signature, the axis dispatch and the
   wave-3-shaped files. The g4 branch's two unique docs
   (`91-doc-sweep-fighter.md`, `91-addendum-d22r-fold-list.md`) are
   audit trail — cherry-pick the DOC FILES only if wanted on main.
3. **Compose, don't re-fold:** g8 (`82c93cec`, 34 kinds ≤230 — its
   WallCreeper 164 / Nymph 196 style-3 rows keep `update_skeleton`
   kind arms, disjoint from `claims()`), g9 (`5d08fdf2`, ten
   kind-gated rows on the same machines), g10 (at base). The shared
   `npc.rs` kind-match region: disjoint kind sets — keep both sides.
4. **The d34-d43 kind lanes** take `91-d22-roster-gap-table.md` as
   their E6 input. The stream-parity obligation: every new moan-listed
   walker kind MUST join `fighter_walker_kind` (the extension list is
   in the table §2.1) or its UpdateNPCs stream shortfalls per tick.
   The plain-walker gap kinds should move onto `claims()` (the
   faithful axis core) and retire their `walker_step_fighter`
   stand-in arms; the SHOOTER-block types keep their own machines.
5. **Run the written-not-run witnesses at the merge** (the AGENTS.md
   verification gate): the P5 graveyard set, the P9 census, the 489
   normal-night pair, the R1-re-derived graveyard end-to-end +
   DW-3.1 replays, the style-4/5/6 stream-parity sets. All are
   code-only-written on this branch (no cargo invoked in this lane).
6. **After the merge is verified** (the step-6 ancestor + symbol
   checks), the Lead/owner deletes the g4 branch (local + remote +
   prune). This lane never deletes another agent's branch.

## 4. Branch receipt (the retire hashes)

| phase | commit | scope |
| --- | --- | --- |
| DONE base | `bcd42ddd` | the reconcile record (phases 1-4 complete) |
| R1 | `fb99a047` | the moan-position remainder CLOSED (code + witnesses + docs + anchor) |
| R2 | `28866c7c` | the 123-roster-gap table (d37d-E6 input) + INDEX |
| R3 | `78386ab1` | g4 DEPRECATED + the final KEEP/DROP symbol table + INDEX |
| R4 | (this commit) | the addendum FINAL receipt + the merger instruction |

Verification: code-only — the checklist rows re-grepped at the retire
tip; the g4 tip re-probed (`a9369ec38`, no delta); brace-balance
delta-checked on the R1 .rs edits (0/0); no cargo/tests/rustfmt/
graphify invoked. The witnesses are WRITTEN, NOT RUN — the merge gate
runs them.
