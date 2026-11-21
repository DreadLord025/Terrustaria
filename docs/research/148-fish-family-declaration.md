# 148 — FISH-FAMILY declaration: style-44 + style-16 ALL LANDED (W4-c4 lane close)

Status: LANE-CLOSE DECLARATION + merger contract. Author lane: W4-c4
(`task/W4-c4-style44-flying-fish`). Doc-only retire phase. Companion
contracts: research/143 (the d40-465 retire hunk — number KEPT, see
INDEX collision verdict), research/144 (the Orca NPC_692 census-input —
number KEPT), research/145 (the claims-table addendum for d37d-E3).
Merge order + numbering authority: research/108 EXTEND §§E3/E7 (the
d37c-E plan, tip `bdd59afba`), as corrected by research/145.

## The declaration

**The fish family is COMPLETE on the pushed union.** aiStyle 44 (the
flying swimmers, `NPC.cs:24022-24340`) — all 4 kinds; aiStyle 16 (the
fish/swimmer machine, `NPC.cs:23492-24021`) — all 13 vanilla members
ported verbatim, 12 rostered by c4 + AnglerFish 102 rostered by d35.
No fish-family kind remains unported on the union; the residual work
is merge dedupe + the Orca sheet admission, itemized below.

### aiStyle 44 — 4 kinds (c4 phases 1-2, `5c4c8311`/`377660223`)

| Kind | cs_id | FINAL proto | Owner | Merger action |
|---|---|---|---|---|
| FlyingFish | 224 | **359** | c4 (id adopted from d37's frozen band, `91293a3d2`) | identical-arm dedupe vs d37@359; KEEP c4's world-sim style-44 pre-pass (sibling scan + materialize), drop d37's solo `update_flying_fish` wrapper |
| GiantFlyingAntlion | 509 | **509** | c4 | STANDS (d41 claim-only band 491-560 skips it) |
| FlyingAntlion | 581 | **581** | c4 (d42 ceded, `49346dba6`) | STANDS — research/145 withdrew the retire verdict; `style44_find_frame` + the `num46=-6` nudge are the keepers |
| EyeballFlyingFish | 587 | **587** | c4 (d42 ceded) | STANDS; the 587 materialize pre-pass (shared-pool dust) is the keeper |

### aiStyle 16 — 13 kinds (c4 phases 3-4 `22cec16f7`; base 55/58/65 frozen since S1-D-3)

| Kind | cs_id | FINAL proto | Owner | Merger action |
|---|---|---|---|---|
| Goldfish | 55 | 55 | base | behaviour-frozen, untouched |
| CorruptGoldfish | 57 | **277** (d34) | d34 `5f3435a87` | dup-kind: keep d34@277 per E3's kinds-lane rule, retire c4@405 AT MERGE (research/145 §flags; d37d-E3 final call). c4's `style16_find_frame` case group survives (drop d34's inline case-57 copy — behaviour-identical bands) |
| Piranha | 58 | 58 | base | frozen |
| Shark | 65 | 65 | base | frozen |
| AnglerFish | 102 | 450 @d35 tip `e5558b0a9` → E3 renumber band **698-730** | d35 | not c4's row; the family module's case-102 arm already serves it (`vanilla_type` on d35's side) |
| Arapaima | 157 | **406** (was 237) | c4, renumber EXECUTED `91293a3d2` | STANDS — no rival (d125 owns only the vacated 237 hole) |
| BloodFeeder | 241 | **373** (d37) | d37 `750d8e44` | keep d37@373, retire c4@241 AT MERGE (the dup-check verdict; c4's row stays on-branch so exhaustive matches compile); FindFrame dedupes onto c4's helper |
| CrimsonGoldfish | 465 | **465** (shared id) | c4 IMPLEMENTATION | KEEP c4's rows (kb 1.0 + `vanilla_type` 465), RETIRE d40's defective rows — exact drop-lines: **research/143**; E3's `keep d40@465` row corrected by research/145 |
| GoldGoldfish | 592 | **592** | c4 (d42 ceded) | STANDS (research/145); catchItem 4274; d42's GoldGoldfishWalker 593 interim bestiary entry re-points its fold onto this entry at merge (drop the interim row, ENTRIES pin −1) |
| Pupfish | 607 | **607** | c4 (d42 ceded) | STANDS; catchItem 4373 |
| Dolphin | 615 | **615** | c4 (d42 ceded) | STANDS; the leap-tilt + idle-timer `Next(300,1200)`/`Next(1,3)`/`Next(2)` RNG census on the caller's UpdateNPCs stream |
| Pufferfish | 688 | **688** | c4 — GENUINELY UNCLAIMED (d43 conditional below) | STANDS; the puff one-tick `just_hit` fix `17a241f6a` + catchItem 5511 + the `num46 += 10` anchor |
| Orca | 692 | **692** | c4 — GENUINELY UNCLAIMED (d43 conditional below) | STANDS; AI/stats/FindFrame/wet-anchor fully wired; the NPC_692 sheet (136×992) atlas-deferred — admission via **research/144** (d35e lazy NPC page, FINAL rank #16) |

## The d43 conditional (688/692 — the active band watch)

Re-verified at retire time (2026-11-21): `origin/task/W4-d43-npc-kinds-631-697`
tip == merge-base `a725acb04` (ZERO commits of its own); the `TR-W4-d43`
worktree is CLEAN at base (no staged/uncommitted rows; probe: `git -C …
log/status/diff`, read-only). The only 688 mentions on d43's tree are
the BASE's "the unported Pufferfish 688" hurt_cd comments. g10's
uncommitted town-pet work (637/638/656/670/678-683) touches neither id.
**688/692 are genuinely unclaimed — c4's rows STAND.**

**Conditional retire-at-merge note (for d43r + the merger):** 688/692
sit INSIDE d43's assigned vanilla-free band (E4 guidance: "d41/d43
vanilla-free 491-560 / 631-697, skip the g10/d33 cs ids"). d43r is
ACTIVE. If d43r resumes and rows Pufferfish 688 / Orca 692 before the
merge wave, the E3 ownership rule applies (kinds-lane d43 owns over
wiring-lane c4): **c4's 688/692 variants + wire arms + bestiary entries
retire AT MERGE onto d43's rows** — the BloodFeeder treatment (c4 keeps
its rows on-branch until then so the exhaustive matches compile). The
keeper implementations c4 carries across in that case: the puff-flag
one-tick fix (d43 must read `just_hit`, NOT the hurt_cd stand-in — the
`17a241f6a` witness is the regression pin), `style16_find_frame`'s
688/692 arms, the catchItem 5511 row, the Orca wet-anchor
`num46 = wet ? 6 : 10`, and research/144's sheet census (the NPC_692
admission request survives regardless of which lane owns the kind).
d43r should read this note BEFORE rowing 688/692 — porting them
duplicate-free (skip, c4 owns) is equally valid under E3 and cheaper.

## The merger checklist (per the d37c-E plan, E7 order)

E7: d34 → d38 → d37 → d40 → d42(vanilla-free) → d35(698-730, last) →
small lanes (c4 among them). At c4's merge:

1. Drop c4's BloodFeeder@241 + CorruptGoldfish@405 variants/arms/entries
   (canonical d37@373 / d34@277); keep the helpers + wiring keyed on the
   canonical kinds (E3 ownership rule).
2. Execute research/143's drop-lines on d40's side (CrimsonGoldfish —
   the kb-0.4 + vanilla_type-missing defects die there).
3. FlyingFish@359: identical-arm dedupe vs d37; keep c4's world-sim
   pre-pass, drop d37's solo wrapper.
4. 509/581/587/406/592/607/615/688/692: splice c4's rows as-is (d42
   ceded the five; d43 conditional above for 688/692).
5. Re-point d42's GoldGoldfishWalker 593 bestiary fold onto c4's 592
   entry; drop the interim row (E4 ENTRIES pin re-derive: −1).
6. Orca sheet: d35e admits NPC_692 per research/144 (the lazy NPC page,
   NOT the eager union); then `STYLE16_SHEETS` += 692 + the
   `BESTIARY_NPC_SHEET_IDS`/`bestiary_npc_sheets` slot pair — the ONLY
   code change (the Some/None draw seam already handles it).
7. Count pins: the E4 550/550/550/489 union re-derives with research/145's
   corrections (d42's five cessions ADD c4's rows back as the only
   ports; the walker-fold interim row drops).

## Witness register (all WRITTEN, NOT RUN — code-only lane)

style-44 module 15 + 2 wiring (npc.rs axis claim + flap cadence with
the 359 pin; world_sim/npcs.rs separation + materialize); style-16
defaults pin (405/406) + strip-walk + routing + Arapaima/Dolphin/
Pufferfish/Orca FindFrame (npc_ai_style16.rs); the catch census
(bug_net.rs); the puff one-tick flag (npc.rs); the 283-kind round-trip
bijection guard. The renumber execution `91293a3d2` verified
mechanically: 283↔283 arms, zero dups, 229/237/354 decode None.

## Lane commit chain (the hashes)

`5c4c8311`/`377660223` style-44 phases 1-2 · `22cec16f7` style-16
phases 3-4 · `318e28709` dup-check verdicts · `4e7bcc9a9` Orca overflow
note · `17a241f6a` puff-flag fix · `91293a3d2` proto-renumber EXECUTED ·
`90fd242d9` research/143 · `9156d83a7` FINAL band table · `479d70ad2`
research/144 · retire phase: `cb4fec9c3` collision verdict (143/144
KEPT) · `ff8026327` research/145 addendum · this declaration.

Code-only: claims re-verified via git-show/git-grep at the named pushed
tips (`49346dba6`, `bdd59afba`, `5f3435a87`, `e5558b0a9`, `a725acb04`);
read-only worktree probes (d43/g10); no compiler invoked, no junction
recursion, no foreign writes.
