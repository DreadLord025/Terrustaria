# 144 — The Orca NPC_692 census-input (W4-c4 → d35e atlas repack)

Status: CENSUS INPUT / MERGE CONTRACT — the dims-row + admission
request for the repack lane. Author lane: W4-c4
(`task/W4-c4-style44-flying-fish`, parent `9156d83a7`). No Rust
touched; the measurement re-run on the current HEAD.

## The dims-row (the deliverable)

| sheet | w×h | padded w×h | padded kpx | frames | owner | admission |
|---|---|---|---|---|---|---|
| `NPC_692` | **136×992** | 138×994 | **137.2** | 16 × 62 px | `W4-c4-style44-flying-fish` | the world's NPC page (the §10.6 lazy form), NOT the eager union |

Re-measured with `tools/_w4c4_headroom.js` (the skyline packer over the
real `.xnb` dims, `assets/Content/Images/NPC_692.xnb`):

- baseline main **7762** / full gore+font **8150** (bound 8192);
- the eight packed style-16 sheets (57/157/241/465/592/607/615/688)
  are already in the baseline (+0 each);
- **NPC_692 alone**: main 7762→**7844** (+82, fits) but full
  8150→**8226** (+76, **OVERFLOW by 34 px**).

The Orca is the SINGLE sheet that tips the full eager set over the
strict-GPU bound — the wyvern-precedent deferral stands on THIS
branch: `STYLE16_SHEETS` omits 692, `npc_sheet(692)` returns None, the
draw arm + the bestiary portrait skip until admission. The AI + stats
+ FindFrame (`NPC.cs:74693-74762` two-half glide) + the wet anchor
nudge (`num46 = wet ? 6 : 10`, `Main.cs:23072-23074`) are fully wired.

## The rank — CORRECTED against the FINAL 490-name union

The lane's earlier note (ANALYSIS "The Orca sheet overflow", commit
`4e7bcc9a9`) said "rank #10, just above NPC_109 (130.2) and below
NPC_243 (153.8, #9)". That was against the **407-name** Appendix A
list. d35's ATLAS FINAL (`e5558b0a9`, research/113 §§10-12 + Appendix
B) regenerated the union to **490 names** (`tools/_d35_union_table.md`
on its tip) — the d125 empress extras + the g70 flares interleaved.
Against the FINAL list the padded-area-DESC neighbours are:

| FINAL rank | sheet | padded kpx |
|---|---|---|
| #15 | `NPC_243` | 153.8 |
| **(insert)** | **`NPC_692`** | **137.2** |
| #16 | `NPC_109` | 130.2 |

So the Orca inserts at **FINAL rank #16** (between NPC_243 and
NPC_109), shifting every row ≥16 down one. It remains a mid-upper
consumer — well below the d70 sunflares (1052.7 kpx each) and the
d125 Extra_157/159 strips (240.9). **NPC_692 is NOT in the 490-name
union** (verified: zero hits in `_d35_union_table.md` /
`_d35_union_list.json` at `e5558b0a9`) — the census predates this
lane's sheets phase. The repack must ADD it.

## The admission form — the §10.6 lazy contract, not "page 1"

The earlier note asked for "page-1 admission" under the two-page
eager form. The FINAL plan's §10 amendment SUPERSEDES that: NPC
sheets leave `STATIC_SHEET_NAMES`' eager pack and join the
world-dependent lazy half (the resident + spawnable + bestiary
superset, packed as fresh pages after the union). The corrected
request:

1. **Add `NPC_692` to the NPC census input** (`_d35_npc_census.py`'s
   extraction surfaces): the Orca is an Ocean-spawn fish kind AND a
   bestiary ENTRIES row (net_id 692, c4's entry) — it belongs to the
   long-lived + bestiary-opened sets. Its 136×992 strip adds at most
   ~994 px of skyline to the NPC page (the packer's call — the §10.4
   sets have ≥5600 px headroom, so ONE NPC page still holds).
2. **Do NOT add it to the eager union** — that re-creates the 8226
   overflow the deferral exists to avoid.
3. **On admission, re-enable on c4's side** (the merger's splice, no
   behavioural change): `STYLE16_SHEETS` gains 692, the
   `BESTIARY_NPC_SHEET_IDS` + `bestiary_npc_sheets` pair gains the
   692 slot (the portrait rests the sheet's own 62-px row height).
   The draw arm already handles the Some/None seam — the sheet
   admission is the ONLY code change needed.
4. The per-world page contract (§10.5: 3 pages realistic / 4
   adversarial) is unchanged by one 137.2-kpx sheet inside the NPC
   page's headroom.

## Cross-references

- ANALYSIS "The Orca sheet overflow — repack priority note" (c4
  `4e7bcc9a9`) — the original measurement; this doc corrects its
  rank (#10→#16) + admission form (page-1→the lazy NPC page).
- ANALYSIS "The FINAL style-44/16 band table" — the Orca row's
  STANDS verdict + the deferral pointer.
- research/113 §§10.3-10.6 (d35 tip `e5558b0a9`) — the lazy union
  7162 px, the per-world NPC budgets, the step-7 amendment.
- The c4 sheet set already in the union: NPC_224/509/581/587 (the
  44 family) + NPC_57/157/241/465/592/607/615/688 (the 16 family,
  Appendix B rows 28/29/33/45/46/47 + the Appendix A rows) — the
  Orca is the lane's ONLY missing name.

Code-only: measurement re-run (`node tools/_w4c4_headroom.js`), the
union tables read via git-show at the named tips; no compiler invoked.
