# 243 — The seal verification round (W4-g58): 240-collision re-scan, f37-hunk re-verify, f22-seed delivery status

Date: 2026-11-21 ~14:00 +0200. Code-only — no cargo/tests run. This round
verifies the four pushed retire commits (`cdfe3d108`/`30bb6e2b1`/
`d2943e4aa`/`d0f6275a5`, now at origin) against the live branch tips.
Number 243 is free at every tip (the scan below covers 237–249).

## 1. The 240-rename re-check — NO COLLISION, 240 STANDS

A fresh 358-tip `git ls-tree docs/research/` scan (every `origin/task/W4-*`
branch, this round) for numbers 237–249 finds:

| Number | Claimants at tips | Verdict |
| --- | --- | --- |
| 237 | e23 `237-bestiary-ui-final.md` + g48 `237-f11-tail-fix.md` | COLLISION (2) — their lanes/merger resolve |
| 238 | c8 `238-c8-crawler-final-census.md` + d42 `238-d42-roster-sheets.md` + g61 `238-g61-spawn-family-final.md` | TRIPLE collision — their lanes/merger resolve |
| 239 | d42 `239-d42-final-keep-one-and-band.md` | single (d42) |
| 240 | g58 `240-happiness-final.md` ONLY | **g58's, no collision** |
| 241 | g58 `241-f37-latch-repoint-hunks.md` ONLY | g58's |
| 242 | g58 `242-chat-portrait-seed-happiness-sealed.md` ONLY | g58's |
| 243–249 | none | free (243 = this doc) |

The sibling lanes did NOT follow research/240's first-add-order suggestion
(d120→237 / d42→238 / c8→239): c8 took 238 (not 239), d42 took 238+239,
g61 took 238, e23/g48 took 237, and d120's `143-d120-chatter-retire.md`
is STILL 143 at its retired tip `2408233da` (never renamed). The
suggested assignments in 240's renumber note are therefore SUPERSEDED —
a correction is appended there. None of this touches 240: g58 remains
the sole 240 claimant, the rename stands, no renumber needed.

Registry status: `origin/task/W4-d54-dust-sites-world` tip is still
`38bfd1d22` (research/150, 10:22:14); NO 150b refresh exists at ANY tip
(a second 358-tip scan for `docs/research/150*` finds only d54's 150).
Registry-150 §2's rule ("late pushers MUST take free numbers 237+")
governed; the 237/238 collisions above are for the registry's next
refresh / the merger — NOT g58's to resolve (own-branch rule).

Also noted for the merger: f22 pushed `144-f22-ui-screens-retire.md`
(tip `8c1b40a46`) — 144 is c4's per registry-150 §2 (one-sided, c4 keeps);
f22's doc must take a free 237+ number on ITS branch.

## 2. The f37-repoint consumption — STILL PARKED, hunks re-verified EXACT

f37 tip probe: `908482d52` 2026-11-21T02:48:30+02:00 — UNCHANGED, now
~11.2 h parked (was 8.5 h at research/241's publish). The lane has not
consumed the hunks. Re-verification of research/241's five hunks against
the live f37 tip (`git show origin/task/W4-f37-shop-price-math:…` +
line scan): every cited line still holds EXACTLY —

- `crates/tr-app/src/app/mod.rs:1697` `buy_price_row(entry, self.discount_available, self.shop_mood)`;
- `:1759` sell payout args `self.shop_mood,`;
- `:1849` reforge cost args `self.shop_mood,`;
- `:2090` nurse args `self.shop_mood,`;
- `crates/tr-app/src/app/frame/draw.rs:1694` + `:1746` `self.shop_mood,`.

The field/capture/witness sites 241 says STAY also match (`mod.rs:211`
field, `:1961`/`:3604` init, `:5708`/`:5758`/`:5849` witness feeds).
research/241 is published at origin as of this round's push — the f37
lane (if it resumes) or the merger applies it verbatim; the prerequisite
(f37 has no `shopping_settings` field; g58's `happiness.rs` +
`set_talk_npc` carry it) is unchanged.

## 3. The f22 seed delivery — f22 RETIRED BEFORE the seed reached origin

f22 tip probe: ADVANCED `a2fdf86fb` (10:57:48) → `8c1b40a46` (12:04:59,
retire 5: research/144 + the UI-screens-family COMPLETE declaration,
"no f22 follow-up open"). Crucially, g58's retire commits (incl. the
research/242 §2 portrait task-seed) were LOCAL until this round's push
(~14:00) — f22's retire-5 predates the seed's publication and its docs
carry NO portrait-system claim (a `git grep -i portrait` over f22's tip
docs hits only base-doc/bestiary mentions; its 144 names g58 only for
the shop/happiness/housing BUTTONS remainder). Consequences:

- The chat-portrait DRAW (Detailed arm) + hop state are g58's, LANDED
  and now pushed — no f22 overlap, no compose conflict.
- The CloseUp/FullBodyRetro dummy-NPC remainder (research/242 §2) is
  UNCLAIMED again: f22 declared COMPLETE without it. The seed stands as
  published for a SUCCESSOR lane (the wiring contract is unchanged:
  `draw_portrait`'s preference-1/2 early return is the replace site).
- d120 (the chat-panel co-lane) is RETIRED (`2408233da`, the GetChat-40
  declaration; the sign-aware panel handed to f22, whose retire-5
  records the INTERACTION_LINES patch ported). No d120 compose owed.

## 4. The seal — CONFIRMED INTACT at origin

`git ls-remote origin refs/heads/task/W4-g58-ui-shop-happiness` →
`d0f6275a5eda3e1fbab862dc670a787ac401f4ee` (the 4 retire commits pushed,
`2ff3b7062..d0f6275a5`). research/240 carries the SEALED banner; the
one-paragraph happiness-family bible (g58-latch CANONICAL / f37-price-
MATH / d122-SHOPS) is research/242 §3; the ANALYSIS retire section +
PORT_MAP row + INDEX rows 240/241/242 all landed in the pushed tree.
No further happiness port work on any lane.

## 5. Verification record

- Push: attempt 1 `Recv failure: Connection was reset`, attempt 2
  succeeded (`2ff3b7062..d0f6275a5`), ls-remote confirms the tip.
- Scans: 358 `origin/task/W4-*` tips, `git ls-tree docs/research/` per
  tip, twice (237–249 claimants; `150*` registry refresh).
- f37 hunks: line-exact re-read of both files at `908482d52`.
- f22/d120: tip logs + doc greps at `8c1b40a46`/`2408233da`.
- Code-only mandate: no cargo/tests/rustfmt/graphify run.