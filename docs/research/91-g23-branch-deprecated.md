# 91 — g23-branch-DEPRECATED: merge-direction recommendation (MENU RETIRE)

**Status: `task/W4-g23-sounds-main-3` is DEPRECATED AS A MERGE BASE.**
Its content survives; the branch must NOT be the canonical lane tip.
The merger takes **`task/W4-g22-sounds-main-2` as the superset base** and
folds g23's unique wired sites into it — the smaller-fold direction.

## 1. The probe (g23d-session liveness)

* 2026-11-21, `git ls-remote origin refs/heads/task/W4-g23-sounds-main-3`
  → **`945391e33df214eb6332bbb97c4926fa6a1dc579`** — UNCHANGED since the
  MENU-FINAL phase-1 probe (`0cdc6084a`). g23d was dispatched to re-fold
  g22's doc-sweep; it has not pushed. Treat g23d as dead or stalled: the
  research/91 re-fold list (`91-g23-refold-g22-doc-sweep.md`) **stands as
  the record**, but its fold TARGET is superseded by §3 below.
* g22 tip at this note: `3b7949ae` (MENU-FINAL close-out), 20 commits
  past the common base `e0f81135`.
* `git merge-base task/W4-g22-sounds-main-2 origin/task/W4-g23-sounds-main-3`
  → `e0f811358` — g23's union merged ONLY the 90/90 window; none of the
  20 later g22 commits are in g23.

## 2. The two merge directions, measured

| Direction | Base | Fold payload | Commits | Diff vs base | Hot-file reconciliation |
| --- | --- | --- | --- | --- | --- |
| A — g23-union base | `945391e3` | g22's post-base lane | **20** | 17 files, +1865/-31 | 20-commit replay through the union's rewritten `actions.rs`/`mod.rs`/`frame.rs`; the 91-doc's adjacent-hunk map applies per commit |
| B — g22-superset base | `3b7949ae` (g22 tip) | g23's unique content | **4** (3 content + 1 union) | g23-unique ≈ 13 files, +1755/-31 vs base; the union's re-fold of g22's OLD window is ALREADY in g22's history and drops out | ONE reconciliation pass over the 91-doc's hot-file map |

**Pick B.** Fold-size: 4 commits vs 20; one hot-file pass vs a 20-commit
replay. Direction B also keeps g22's later corrections (the connecting
guard BEFORE the header gate `ba5355c19`, the EquipArmor census fix, the
three music arms) as the base truth — direction A would have to re-land
each of them through g23's rewritten menu files.

**Consequence (the banner question):** because the direction FLIPS (g22
canonical), **no DEPRECATED-BY-UNION banners are added to g22's files.**
The deprecation is on the g23 BRANCH as a base, recorded here — not on
g22's code. g23's own files carry their own scope notes already.

## 3. Merger instruction (direction B, concrete)

1. Base: `task/W4-g22-sounds-main-2` tip (this branch, MENU RETIRE tip).
2. Fold `e0f811358..945391e33` (g23's 4 commits). A plain
   `git merge origin/task/W4-g23-sounds-main-3` works — the merge base is
   `e0f81135`, so g23's union-side copies of g22's OLD window resolve
   against g22's newer versions of the same hunks; keep g22's side for
   everything the doc-sweep/MENU-FINAL rewrote, take g23's side for its
   unique content (list below). Never `-X ours/theirs` (MAIN-RULES).
3. g23-unique content to KEEP (the "5 wired sites" + census):
   * **Colour-focus tick `:51276`** — `Menu::pending_ticks` (DRAW-TIME
     chrome queue) + the `focusColor` capture/reset + the
     `draw_color_sliders -> [bool; 4]` signature change (`ui/sliders.rs`)
     + the drain after `Menu::draw` in `app/frame/draw.rs` (~:1830).
   * **Volume-bar hover tick `:51575`** — `volume_drag_pass`'s persistent
     `rightHover` mirror, enter/move/leave edges.
   * **IP-submit `:52059`** — `Act::Nav` Accept-row `ip_submit` arm +
     Enter `submit_join_ip`, BOTH on `pending_cue` (the union already
     moved it off `pending_ticks`; keep that shape).
   * **Password-cancel `:51952`** — JoinPassword Back row, `pending_cue`.
   * **NewText tails `:64635`/`:64641`** — centralized in
     `ChatMonitor::new_text` (`chat.rs`) + `drain_new_text_cues` in
     `frame.rs` (:478 region) and the achievement subscriber
     (`app/mod.rs` :2158 region — replaces the inline `Menu_Tick`).
   * Plus: `main_sounds_batch3.rs` (the 75-site `DRAW_MENU_CUE_CENSUS` +
     14 tail halves), the star-pair dormant seam (`audio.rs`), the 9
     replayed g22-missed rows (Video Parallax/Effects forward-11
     `:50002`/`:50075`, ResetKeys ×2 `:50458`/`:50563`, Resolution
     cycle/borderless/Apply `:50181`/`:50201`/`:50225`,
     ManageControls/Steam-join `:50686`/`:50726`), the two g22-arm
     expectation fixes (password_back tuple JoinPort→JoinIp,
     JoinIp-Accept None→Menu_Tick), `play_sound_lines_g23.txt`, the 4
     anchor rows, g23's ANALYSIS section.
4. Hot-file reconciliation — the 91-refold doc's compose-check map
   (`91-g23-refold-g22-doc-sweep.md` §"Disjoint-population contract")
   applies IN REVERSE: g22's hunks are the base now, g23's are incoming.
   Keep BOTH at every adjacent pair: `actions.rs` Act-tail (g22's three
   Action arms + g23's ResetKeys/Hint cue writes), `mod.rs` struct fields
   (`load_failed_path` + `pending_ticks`) and the three test-tail groups,
   `anchors.txt` both EOF blocks. The connecting guard stays BEFORE
   `if item.header`. `frame/draw.rs` arms are disjoint (front `audio`
   field vs menu drain). `audio.rs` hunks are disjoint (g23: SFX_CUES
   :286 / star seam :1978 / tests :4574; g22: MusicEnv :2529 / menu-music
   :2715 / tests :3391+:4343).
5. The disjoint-queue contract MUST survive: `pending_cue` = activation
   only; `pending_ticks` = draw-time legacy chrome only (`:51276`,
   `:51575`); the doc-sweep's HSL front cues ride `FrontDraw.audio`, not
   either menu queue. g23's union witnesses + g22's 19 doc-sweep + 2
   MENU-FINAL witnesses all stay (written, not run — code-only lanes).
6. After the fold: verify per MAIN-RULES (cargo suite on the merged
   branch, anchors canary), then merge to main by the ref-update
   procedure. Delete BOTH lane branches only after
   `git log main..<branch>` is empty for each.

## 4. If g23d wakes up

If `origin/task/W4-g23-sounds-main-3` advances past `945391e3` before the
merger runs, re-probe: if the new tip already contains g22's
`3b7949ae` (or later), direction A became free — take that tip as base
instead and skip the fold. Otherwise this note stands; g23d's new
commits join the direction-B fold payload.