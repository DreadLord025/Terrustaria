# 145 — Claims-table addendum: the d42 keep-d42 verdicts are SUPERSEDED (W4-c4 → d37d-E3)

Status: MERGE-CONTRACT ADDENDUM — input for the active d37d reconcile
(the research/108 EXTEND, tip `bdd59afba`, §E3 canonical table). Author
lane: W4-c4 (`task/W4-c4-style44-flying-fish`). Doc-only; no Rust
touched. Number 145 chosen free on the 357-tip sweep (the lead's
146/147 rename-fallbacks stay unassigned — 143/144 were KEPT, see INDEX).

## Why this addendum exists

c4's dup-check + proto-renumber phases (ANALYSIS "The style-16 dup-check"
/ "The proto-renumber EXECUTION" / "The FINAL style-44/16 band table")
published five **keep-d42-ids, retire-c4-rows-at-merge** verdicts against
d42's pushed tip `9b22c091a` (the port-local 354-404 band):

| Kind | cs_id | d42 id @`9b22c091a` | c4 id | old verdict |
|---|---|---|---|---|
| GoldGoldfish | 592 | 366 | 592 | keep d42@366, retire c4@592 |
| Pupfish | 607 | 381 | 607 | keep d42@381, retire c4@607 |
| Dolphin | 615 | 389 | 615 | keep d42@389, retire c4@615 |
| FlyingAntlion | 581 | 357 | 581 | keep d42@357, retire c4@581 |
| EyeballFlyingFish | 587 | 363 | 587 | keep d42@363, retire c4@587 |

**d42 then pushed its own reconcile `49346dba6`** (2026-11-21 09:32,
AFTER c4's last census): "renumber the 578-630 roster to the 108 union
plan's VANILLA-FREE band + drop the 8 sibling-owned kinds". Verified
against `origin/task/W4-d42-npc-kinds-561-630` at that tip (git-show /
git-grep, read-only):

- `to_u16`/`from_u16` now ride protocol==NPCID (the vanilla 578-630
  slots, 313 arms bijective per its commit message);
- **all five kinds above are GONE from d42** — zero `GoldGoldfish,` /
  `Pupfish,` / `Dolphin,` / `FlyingAntlion,` / `EyeballFlyingFish,` enum
  variants and zero 357/363/366/381/389 or 581/587/592/607/615 wire arms
  in its `npc.rs`/`npc/data.rs` (grep exit 1 on both patterns);
- its commit message names the cession explicitly: "Dropped
  FlyingAntlion(581)/EyeballFlyingFish(587) [c4 style-44],
  GoldGoldfish(592)/Pupfish(607)/Dolphin(615) [c4b style-16] … Their ids
  stay free for those branches."

## The corrected claims table (what d37d-E3 must read)

| Kind | cs_id | FINAL owner + id | E3 row @`bdd59afba` says | Correction |
|---|---|---|---|---|
| GoldGoldfish | 592 | **c4 @592 STANDS** | `keep d42@592 drop c4@592` | d42 dropped the kind — nothing to keep on its side; c4's variant/rows are the ONLY port |
| Pupfish | 607 | **c4 @607 STANDS** | `keep d42@607 drop c4@607` | same |
| Dolphin | 615 | **c4 @615 STANDS** | `keep d42@615 drop c4@615` | same |
| FlyingAntlion | 581 | **c4 @581 STANDS** | `keep d42@581 drop c4@581` | same |
| EyeballFlyingFish | 587 | **c4 @587 STANDS** | `keep d42@587 drop c4@587` | same |

E3's five rows were derived from d42's OLD tip (`9b22c091a`, the E0
census read); `49346dba6` supersedes them. The five c4 retire-at-merge
actions in c4's own ANALYSIS band table are hereby WITHDRAWN — c4 keeps
its variants, wire arms, bestiary entries and catchItem wiring outright.
The keeper implementations c4 flagged for the merger (the
`style16_find_frame`/`style44_find_frame` helpers, the world-sim
pre-passes, the catchItem 4274/4373 rows, the Dolphin idle-timer RNG
census) now merge with NO dedupe partner on d42's side.

### The GoldGoldfishWalker 593 fold (the one remaining d42×c4 seam)

d42 keeps GoldGoldfishWalker at 593 with an **interim OWN bestiary
entry** (`bestiary.rs:2244-2252` at `49346dba6`: "its vanilla credit
folds onto GoldGoldfish(592, ContentSamples.cs:1049) but 592 is c4b's
UNPORTED kind → interim OWN entry … the merge re-points the fold once
c4b lands"). c4b == this lane, and 592 IS ported here (bestiary entry
`credit_id: "GoldGoldfish"`, net_id 592). **Merge action: re-point d42's
walker fold onto c4's GoldGoldfish entry and drop the interim row** —
the ENTRIES union count loses one row (the E4 489 pin was computed with
d42's interim entry; d37d must re-derive).

## Two more stale-E3 flags for d37d (found by this retire-phase sweep)

1. **CorruptGoldfish — c4 @405 × d34 @277 (dup-kind, E3-contested).**
   E3 says `keep d34@277 drop c4@229`; c4's renumber phase moved 229→405
   (the d125 hard-collision fix) BEFORE reading d34's phase-3 push
   `5f3435a87` (06:12), so c4's FINAL table says "STANDS at 405". Facts
   at d34's tip: full kind at 277/cs 57 — kb 1.0, def 6, hp 100, dmg 30,
   value 500, `{31}` immunity, `vanilla_type` 57 present, routes
   `update_swimmer`, inline case-57 FindFrame (wet 6/12/18/24, dry
   6/12 — behaviour-identical to c4's helper group). No wire collision
   (277 ≠ 405); this is the BloodFeeder dup-kind class. Per E3's
   ownership rule (kinds-lane d34 owns over wiring-lane c4) the verdict
   is **keep d34@277, retire c4@405 at merge** — c4's
   `style16_find_frame` case group is the FindFrame keeper (drop d34's
   inline copy). c4's ANALYSIS "STANDS at 405" row is corrected by this
   addendum; flagged CONTESTED only insofar as d37d owns the final call.
2. **CrimsonGoldfish 465 — implementation conflict, not id conflict.**
   E3 says `keep d40@465 drop c4@465`; c4's research/143 says KEEP c4's
   implementation at the shared 465 because d40's row carries two
   defects vs the C# (kb 0.4 vs the `:8533` reset 1f; `vanilla_type`
   missing 465 → `unreachable!` PANIC in the swimmer arm). The id is
   465 either way; the question is WHICH ROWS SURVIVE. c4's position
   stands: the exact drop-lines in **research/143** retire d40's
   defective rows and keep c4's. d37d's E3 row should read "keep the
   465 id, keep c4's implementation (research/143), drop d40's rows".
   Also stale in E3's per-lane table: c4's row still lists
   "229/237/354" — c4's pushed tip now rides **405/406/359** (the
   renumber execution `91293a3d2`); Arapaima @406 has NO rival (d125
   owns only the 237 hole c4 vacated), FlyingFish @359 is the
   identical-arm adoption of d37's frozen row (E3's `drop c4@354`
   already satisfied — the arms are now equal, trivial dedupe).

## Cross-references

- research/108 EXTEND §§E0/E3/E4/E7 (d37 tip `bdd59afba`) — the table
  this addendum corrects; E7's merge order (d34→d38→d37→d40→d42→d35→
  small lanes) is UNAFFECTED (d42's vanilla-free renumber is exactly
  what E7 assumed).
- research/143 — the d40-465 retire hunk (unchanged, still the contract).
- research/144 — the Orca NPC_692 census-input (unchanged).
- c4 ANALYSIS "The FINAL style-44/16 band table" — the five withdrawn
  retire rows + the CorruptGoldfish correction are annotated there in
  this lane's retire phase.
- The 688/692 d43-band watch: see research/148 (the FISH-FAMILY
  declaration) §"The d43 conditional".

Code-only: every claim read via git-show/git-grep at the named pushed
tips (`49346dba6`, `bdd59afba`, `5f3435a87`, `9b22c091a`); no compiler
invoked, no foreign working-tree writes, no junction recursion.
