# 283 - FindFrame census addendum after f26 slice 7j (was key 68; renumbered per registry-150 §3/§6.3/§7.3, KEEPS §9.1: 7c->201 7e->203 7f->205 7g->251 7h->281 7i->282 7j->283; executed at the plan-B3 merge of task/W4-f26-findframe-extra-3)

> RENUMBER # 68 addendum — FindFrame census after f26 slice 7j (code-only)

Audit date: 2026-11-21. Branch `task/W4-f26-findframe-extra-3`, slice
7j, base tip `5af99f1cc` (slice 7i, PUSHED this run — attempt 5 of the
flaky-network retry, `899896d42..5af99f1cc`). **Code-only mode: no
cargo/rustc/tests/rustfmt/graphify invoked** — every claim is
git/file-read evidence against the PUSHED remote tips, re-probed at
census time (`git fetch origin --prune`).

The master wire-census lives at `docs/research/200-findframe-wire-status.md`
on `task/W4-d44-findframe-low-1` (§6's 130-row UNCLAIMED worklist base).

## The window re-probe (at the CURRENT tips)

The pool: 32 (the 7i addendum post-state). Per-tip re-probe:

* **f27 tip ADVANCED `1a24a133` → `77ebe4e93`** (slice 11 EARLY CLAIM):
  435/436 — a keep-one DUP vs OUR pushed 7g claims (no new independent
  id) — plus the 550/552-563 v1-arm FIXES (outside our window, no
  count change).
* **d42 tip ADVANCED `33b3cfe07` → `bac408be0`**: doc-only finals (the
  11-kind keep-one drop-hunks, the 593-fold re-point, the 578-630 band
  declaration) — no new FindFrame ids.
* **d41 `727578f2d` / d44 `ce2c3a24a` / c4 `0df6964b` / c8 `7c2de9c06`
  / e26 `048cca6c` / g10 `5dee2865`** UNCHANGED.
* **Mechanical claim grep**: all EIGHT lane tips grep ZERO for
  `GemBunnyRuby|GemBunnyDiamond|GemBunnyAmber|HellButterfly|Lavafly|
  MagmaSnail` + `65[0-5] =>` arms over `npc.rs` / `npc/` /
  `npc_findframe*.rs` — the ONLY d44 hits are wave-C's header
  cut-list comments naming 654/653/655 as deliberately OUT of its
  350-550 band; the advanced f27/d42 tips grep ZERO `=>` arms for the
  remaining worklist ids too. This branch's own tree: the ONLY
  kind-name hits are data rows (`creative_sacrifices` / `itemdata` /
  `lang_tables` / `bug_net` / the style-64/67 AI machines), no
  FindFrame arms.

## The census count

| step | subtraction | remaining |
| --- | --- | --- |
| the 7i post-state | — | 32 |
| f26 slice-7j THIS claim (650/651/652/653/654/655) | 6 | **26 unclaimed** |

The 26-id post-slice worklist: **634** (the RNG SporeBat — seam
contract in the extra7h header), **657-662** (6), **664** (1),
**667** (1), **669** (1), **671-677** (7), **687-689** (3),
**691-696** (6) = 1+6+1+1+1+7+3+6 = **26**.

## This slice's claim (6 kinds — the SMALL 400-storm batch)

The brief's "same hop-group-arm extension" premise holds ONLY for the
bunny tail; the hell-critter tail is three DIFFERENT arms (all
re-read from source, each window ≤60 lines):

* **650/651/652** GemBunnyRuby/Diamond/Amber
  (`NPCID.cs:11977/:11979/:11981`) — the hop group
  (`NPC.cs:76863-76865`, body `:76851-76904`): the VERBATIM 7i machine
  copy (the f27-540 ONE-group precedent; the wiring merge keeps one).
* **653** HellButterfly (`NPCID.cs:11983`) — the butterfly group
  `case 356/444/653/661` (`:72739-72773`, label `:72741`): the
  `vx*0.3f` tilt + the UNCONDITIONAL sprite copy + the velocity-fed
  `1 + (|vx|+|vy|)/2` accumulator, rows 0/1/2/1 reset at `7*4-1 = 27`;
  NO colour bank — the `:72768` gate excludes 653 (aiStyle 65
  `:17254`).
* **654** Lavafly (`NPCID.cs:11985`) — its own `case 654`
  (`:72690-72709`): the bare `++` <4/>=7 two-row blink + the
  `localAI[2] <= 0` glow-off +2-row bank shift. The timer is AI-side
  (the style-64 machine already on this branch; the roll `:34184`
  draws THERE) — it rides `FrameInput::local_ai2` as the RESOLVED
  value (the 7d resolved-draw seam precedent). aiStyle 64 `:17269`.
* **655** MagmaSnail (`NPCID.cs:11987`) — the snail body
  (`:72240-72295`, shared with 359/360): **ALREADY PORTED** by
  `npc_ai_style67.rs::snail_find_frame` (`:414`) on this branch — the
  new arm DELEGATES (the d44 wave-C Snail/GlowingSnail "one machine,
  no second copy" precedent). The claim is the id-keyed seam entry,
  not a second machine. aiStyle 67 `:17284`.

Zero-RNG + zero-world-read mechanical grep over the four cited ranges
(`:76851-76904`, `:72739-72773`, `:72690-72709`, `:72240-72295`) =
ZERO. Frame counts cross-checked `Main.cs:67082` vs
`NPC_FRAME_COUNT`: 650-652 = **7** (`:67149` entries 1-3), 653 = **3**
(entry 4), 654 = **4** (entry 5), 655 = **6** (entry 6); all 697 slots
re-diffed = ZERO mismatches (a mechanical text-parse diff).

Module `crates/tr-app/src/npc_findframe_extra7j.rs` +
`find_frame_kind_extra7j` seam + `main.rs` registration; 10 module
witnesses WRITTEN NOT RUN (code-only), every vector hand-re-derived
(the bunny vx2 +3.0/tick every-3rd step + the row-6 wrap edge; the
butterfly still +1.0/tick 27-reset tick SHOWS row 1; the vx2/vy2
+3.0/tick feed; the lavafly 7-reset + the `<= 0` glow boundary; the
snail fall 11-reset / walk 59-reset phase edges + the 180-combo
delegation-equality grid).
