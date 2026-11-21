# 238 - The NPCID 578-630 roster sheets: real-dims inventory + pack verdict (W4-d42)

Status: DONE + SEAL-CONFIRMED (measurement + inventory; code-only mode — the
packers run under node, no cargo/tests). Branch `task/W4-d42-npc-kinds-561-630`,
base commit `49346dba6` (the research/108 vanilla-id reconcile). Input to d35's
`research/113` atlas-final union-forecast (§§10-12 lazy-load contract).

**Number-238 SEAL receipt (registry-150 round 2, d54 `72226e500` §6.2):**
the TRIPLE 238 collision (c8-crawler / g61-spawn / d42-sheets) resolved in
d42's favour — d42's 143→238 rename (`9eadc3043`, first-add 12:15:55) is the
FIRST 238 at any tip and **WINS the key**; c8's 238-crawler-final-census
(`2a0b5c701` 12:24:19) renumbers →262 and g61's 238-g61-spawn-family-final
(`6e922fc60` 12:55:22) →273 — BOTH EXECUTED at their origin tips
(`262-c8-crawler-final-census.md` / `273-g61-spawn-family-final.md`,
re-fetched this round). Registry §6.2 row verbatim: "d42-npc-kinds-561-630 |
143 → **238 KEEPS**; new doc **239 KEEPS** … **CONFIRMED - no further
action.**" The adjudication key is FIRST-ADD %cI (registry §1/§6), NOT
push time — a concurrent seal-verify draft in this worktree proposed
238→247 on push-time ordering (c8 pushed 12:40:57 before d42's bac408be0
push); that draft was discarded unstaged: it contradicts the published
registry AND 247/248 are §6.1-assigned to d10's 26d doc / g82's 26e-f doc.
No renumber owed by d42; 238 stands.

## 1. Scope — the 43 roster kinds after the 108 reconcile

The reconcile (`49346dba6`) renumbered the roster to **protocol == NPCID**
(the vanilla-free 578-630 band) and dropped the eight sibling-owned kinds:
581/587 (c4 style-44), 592/607/615 (c4b style-16), 604/605 (e26b luck),
606 (c8 style-66). 588/589 were already packed before this wave. The
remaining **43 kinds** are this lane's sheet demand:

578 579 580 582 583 584 585 586 590 591 593 594 595 596 597 598 599 600
601 602 603 608 609 610 611 612 613 614 616 617 618 619 620 621 622 623
624 625 626 627 628 629 630.

Shared ownership (the keep-one inputs):
- **602/603/608/609** (Seagull/Seagull2/Grebe/Grebe2) — c10's tip packs all
  four and d35's 490-name union already rows them under
  `W4-c10-style68-waterfowl`. Our kinds are the same vanilla ids post-
  renumber: at merge, ONE `NpcKind` row + ONE sheet row survives (c10's
  wiring owns the AI; our SetDefaults rows dedupe against theirs).
- **595-601** (the seven dragonflies) — d33's tip ALREADY packs all seven
  (`assets.rs` STATIC_SHEET_NAMES, the style-114 wiring wave) but d35's §11
  re-extraction predates that push, so they are union-NEW with shared
  owners `d33 + d42`.

## 2. Method

`tools/_d42_sheets.js` reuses `tools/atlas_sim.js`'s REAL-dims LZX/XNB
reader (`sheetDims`) and its three packers WITHOUT touching that file —
the purged S1-L-8b git-show block is stubbed in memory (the d38
`_d38_atlas_cumulative.js` technique). Every scenario runs best-fit
shelves + skyline(waste) ×2 (determinism check) + maxrects BAF, and
`verify()` re-checks every layout (no overlap, in-bounds, all sheets
placed). Machine-readable output: `tools/_d42_sheet_inventory.json`
(the inventory rows ride the d35 union schema `{name, owners, w, h,
padded_area, missing}`; padded = (w+2)(h+2), the 2-px atlas inset).

**All 43 `NPC_<id>.xnb` decode; none missing.** Total padded area
**806,554 px² = 0.807 Mpx** (the 39-name union delta: 691,674 px²).

## 3. Measured heights (8192-px strict GPU bound, `tr-render/src/atlas.rs`)

| scenario | sheets | padded Mpx | shelves | skyline | maxrects |
|---|---|---|---|---|---|
| A: HEAD static-only | 8095 | 61.18 | 8628 (+436) | **7711** (481 free) | 7946 |
| B: HEAD + gore | 9497 | 64.24 | 9012 (+820) | **8115** (77 free) | 7956 |
| C: HEAD static + roster 43 | 8138 | 61.98 | 8722 (+530) | **8021** (171 free) | 7730 |
| D: HEAD + gore + roster 43 | 9540 | 65.05 | 9099 (+907) | **8382 (+190 OVER)** | 8058 |
| E: D + boot font | 9541 | 65.06 | 9103 (+911) | **8382 (+190 OVER)** | 8058 |
| F: HEAD + gore + delta 39 (c10's 4 excluded) | 9536 | 64.93 | 9089 (+897) | **8418 (+226 OVER)** | 8596 (+404) |
| G: roster-43 SOLO page | 43 | 0.81 | 1768 | **1768** | 1768 |

All skyline runs deterministic (two runs byte-equal layouts), all
skyline/maxrects layouts verified. Notes:
- The stale "~71 px headroom (8121/8192)" ANALYSIS figure re-measures as
  **77 px (8115)** on this base — same verdict, fresher number.
- Removing c10's four sheets from the pack (F) makes skyline WORSE
  (8418 > 8382): the packer is order-sensitive; sheet removal is not
  monotone. Both overflow — the verdict does not hinge on the four.
- Maxrects BAF "fits" D/E at 8058, but d38c's addendum (research/125)
  WITHDREW the BAF-promotion proposal (it dies within the d40b wave,
  8278 > 8192, and the three-wave area bound alone is 8258 > 8192).
  The skyline number is the contract's currency.

## 4. Verdict — None-skip; the sheets ride d35's lazy-load wave

**The 43 sheets are NOT packed on this branch.** The full runtime set
(gore + font included) overflows the landed skyline packer by **190 px**;
the static-only pack fits (8021) but pins the atlas at 171 px headroom
and would eat the shared budget the d35/d38/d40 waves already claim —
the boundary-split 2-page contract (research/113 §1) and the NPC
lazy-load amendment (§10.6: NPC rows LEAVE `STATIC_SHEET_NAMES` and ride
the per-world page) are the sanctioned home for this demand.

Under the §10 lazy form our demand is cheap: the roster-43 solo pack is
**1768 px** (G) — even the bestiary-opened lifetime superset grows by at
most that (real rolls dedup against the resident set; §10.4's 2566-px
bestiary page + 1768 still fits one 8192 page). The one-page lazy union
(7162) is untouched: NPC_ sheets are not union rows under §10.6.

The draw arms therefore keep the wyvern deferral: `draw_npcs` calls
`generic_anchor(assets.npc_sheet(id), …)`; `npc_sheet` returns None for
an unpacked name and the arm no-draws. No `STATIC_SHEET_NAMES` rows, no
`npc_tex` debug_assert risk. When the repack wave lands, the 43 names
below join the per-world NPC input list (`texture_net_id` keyed — the
folded kinds keep their OWN sheets).

## 5. The union-forecast delta (39 new names, padded-area DESC)

Append to d35's `tools/_d35_union_list.json` schema (sorted padded-area
descending; the four c10 rows already exist — merge adds `W4-d42` to
their owners arrays instead). † = also packed on d33's tip (post-§11
arrival). Under §10.6 these NPC names ultimately ride the per-world half,
not the eager union — the rows exist so the merger's census is complete.

| # | sheet | w×h | padded kpx | owner branch(es) |
|---|-------|-----|-----------|------------------|
| 1 | `NPC_620` | 102×1764 | 183.7 | d42-npc-kinds-561-630 |
| 2 | `NPC_618` | 150×528 | 80.6 | d42-npc-kinds-561-630 |
| 3 | `NPC_628` | 70×782 | 56.4 | d42-npc-kinds-561-630 |
| 4 | `NPC_586` | 60×840 | 52.2 | d42-npc-kinds-561-630 |
| 5 | `NPC_591` | 88×476 | 43.0 | d42-npc-kinds-561-630 |
| 6 | `NPC_630` | 40×896 | 37.7 | d42-npc-kinds-561-630 |
| 7 | `NPC_619` | 48×444 | 22.3 | d42-npc-kinds-561-630 |
| 8 | `NPC_611` | 40×520 | 21.9 | d42-npc-kinds-561-630 |
| 9 | `NPC_625` | 56×360 | 21.0 | d42-npc-kinds-561-630 |
| 10 | `NPC_594` | 256×76 | 20.1 | d42-npc-kinds-561-630 |
| 11 | `NPC_616` | 50×364 | 19.0 | d42-npc-kinds-561-630 |
| 12 | `NPC_617` | 50×364 | 19.0 | d42-npc-kinds-561-630 |
| 13 | `NPC_624` | 32×462 | 15.8 | d42-npc-kinds-561-630 |
| 14 | `NPC_614` | 48×280 | 14.1 | d42-npc-kinds-561-630 |
| 15 | `NPC_580` | 66×192 | 13.2 | d42-npc-kinds-561-630 |
| 16 | `NPC_629` | 32×276 | 9.5 | d42-npc-kinds-561-630 |
| 17 | `NPC_578` | 48×160 | 8.1 | d42-npc-kinds-561-630 |
| 18 | `NPC_590` | 38×144 | 5.8 | d42-npc-kinds-561-630 |
| 19 | `NPC_582` | 34×144 | 5.3 | d42-npc-kinds-561-630 |
| 20 | `NPC_593` | 25×180 | 4.9 | d42-npc-kinds-561-630 |
| 21 | `NPC_621` | 38×74 | 3.0 | d42-npc-kinds-561-630 |
| 22 | `NPC_622` | 38×74 | 3.0 | d42-npc-kinds-561-630 |
| 23 | `NPC_623` | 38×74 | 3.0 | d42-npc-kinds-561-630 |
| 24 | `NPC_626` | 20×132 | 2.9 | d42-npc-kinds-561-630 |
| 25 | `NPC_627` | 20×132 | 2.9 | d42-npc-kinds-561-630 |
| 26 | `NPC_583` | 24×96 | 2.5 | d42-npc-kinds-561-630 |
| 27 | `NPC_584` | 24×96 | 2.5 | d42-npc-kinds-561-630 |
| 28 | `NPC_585` | 24×96 | 2.5 | d42-npc-kinds-561-630 |
| 29 | `NPC_612` | 28×70 | 2.2 | d42-npc-kinds-561-630 |
| 30 | `NPC_613` | 28×70 | 2.2 | d42-npc-kinds-561-630 |
| 31 | `NPC_610` | 20×96 | 2.2 | d42-npc-kinds-561-630 |
| 32 | `NPC_579` | 52×20 | 1.2 | d42-npc-kinds-561-630 |
| 33 | `NPC_595` | 20×48 | 1.1 | d33-npc-ai-styles-95-127†, d42-npc-kinds-561-630 |
| 34 | `NPC_596` | 20×48 | 1.1 | d33-npc-ai-styles-95-127†, d42-npc-kinds-561-630 |
| 35 | `NPC_597` | 20×48 | 1.1 | d33-npc-ai-styles-95-127†, d42-npc-kinds-561-630 |
| 36 | `NPC_598` | 20×48 | 1.1 | d33-npc-ai-styles-95-127†, d42-npc-kinds-561-630 |
| 37 | `NPC_599` | 20×48 | 1.1 | d33-npc-ai-styles-95-127†, d42-npc-kinds-561-630 |
| 38 | `NPC_600` | 20×48 | 1.1 | d33-npc-ai-styles-95-127†, d42-npc-kinds-561-630 |
| 39 | `NPC_601` | 20×48 | 1.1 | d33-npc-ai-styles-95-127†, d42-npc-kinds-561-630 |

The four EXISTING union rows (owner-append only, no new names):
`NPC_602` 44×540 (24.9k), `NPC_603` 44×540 (24.9k), `NPC_608` 52×600
(32.5k), `NPC_609` 52×600 (32.5k) — owners `W4-c10-style68-waterfowl`
+ `W4-d42-npc-kinds-561-630`.

Bestiary custom portraits: the band's two `CustomTexturePath` rows
(`NPCID.cs:5233` 594, `:5389` 621 → `UI/Bestiary/NPCs/NPC_594/621`) are
ALREADY packed on HEAD (`assets.rs` `bestiary_custom_portraits`) — no
delta.

## 6. Keep-one inputs for the merger (sheets axis)

- 595-601: d33 packs, d42 rows the kinds — one sheet row each, owners
  merged; the AI wiring is d33's style-114 machine (its tip wires
  kinds 595-599; 600/601 ride the same `FindFrame` case).
- 602/603/608/609: c10 packs + wires (style 68 axis + `ai_068_waterfowl`);
  d42's SetDefaults rows dedupe (same vanilla ids post-renumber).
- 578-580/582-586/590/591/593/594/610-614/616-630: d42-only demand.
- The eight dropped kinds' sheets (581/587/592/604/605/606/607/615) are
  NOT d42 demand — c4/c4b/e26b/c8 own them (all already in the union).

## 7. The rotation write-census (the whitelist input, Phase 1b)

Mechanical scan (`decompiled/Terraria/NPC.cs` FindFrame cases + the
`AI_<style>_*` functions, cross-checked against the port's `find_frame`
arms and `update_kind_ai`): which of the 43 kinds' machines write
`rotation`, and which of those the PORT writes today.

**IN — the ported machine writes rotation (10 kinds, the gate admits):**

| kind(s) | id | writer | C# cite | port site |
|---|---|---|---|---|
| DD2LightningBug | 578 | `rotation = vx*0.1` | FindFrame `:67876` | `npc.rs` find_frame arm |
| FairyCritterPink/Green/Blue | 583-585 | `rotation = vx*0.1` | FindFrame `:69838` | find_frame arm |
| GoldGoldfishWalker | 593 | grounded `= 0`; air `vy>4`: `-= vy*0.01` | FindFrame `:75926`/`:75959` | find_frame arm |
| Seagull2 | 603 | wet/stand `= 0`; fly `= vx*0.1` | FindFrame `:73860-73870` | find_frame arm |
| Grebe2 | 609 | same shape | FindFrame `:73792-73802` | find_frame arm |
| Owl | 611 | `rotation = vx*0.1` | FindFrame `:73684` | find_frame arm |
| BloodSquid | 619 | hover Aim tilt `atan2(aim)-1.57` | `AI_005` `:50643-50646` | `HoverParams::BLOOD_SQUID` → `hover_step` |
| GoblinShark | 620 | wet atan2 lerp `(rot*4+num43)/5` + wrap; dry decay `*0.9` | FindFrame `:68463-68503` | find_frame arm |

**OUT — C# writers exist but the port's machine does not write yet (join
with their axis-wiring commits):**

| kind(s) | id | C# writer | why OUT now |
|---|---|---|---|
| WindyBalloon | 594 | `AI_113` `:47919` `vx*0.05` | style-113 unwired (`{}` stand-in) |
| BloodNautilus | 618 | `AI_117` `:47248-47430` (AngleTowards/AngleLerp aim ladder) | style-117 unwired |
| Seahorse/GoldSeahorse | 626/627 | `AI_118` `:47082` `+= (vx+vy)/2*0.05` air spin, `:47107` `vx*0.1` | style-118 unwired |
| BloodEelHead/Body/Tail | 621-623 | `AI_006` `:51878`/`:52201` atan2+1.57 | `is_worm` membership deferred (ANALYSIS deviation 6) |

**OUT — no rotation writer anywhere in the kind's C# machine:**
579 (style 0 bound stand), 580/582/586/590/591/624/630 (style 3 — the
`AI_003` rotation arms `:56507`/`:58627-58640`/`:58824-58840` are
type-471/427/185/428-local, none of ours), 595-601 (style 114 —
`AI_114` + FindFrame `:72722-72738` write none), 602/608 (style-7 walk
FindFrame `:73819`/`:73747`), 610/614/616/617/625 (style 7), 612/613
(style 116 — `AI_116` + FindFrame `:69773-69834` write none), 628
(style 119), 629 (style 25 chest — the inline block `:25538-25630`
writes none). The inline style-7/24/68 blocks (`:20812`/`:25313`/
`:34623`) write none either (the Owl's tilt is FindFrame-side).

The draw gate is extracted as `render::entities::npc_rotation_allowed`
(the d38c pattern — behaviour unchanged for the pre-existing members)
and gains exactly the ten IN kinds. Witnesses (WRITTEN NOT RUN,
code-only): `render::entities::anchor_tests::w4_d42_the_rotation_gate_consumes_the_writers`
(the FindFrame writers run the real arms; the 33 OUT kinds refused) +
`npc_ai::tests::w4_d42_the_blood_squid_hover_writes_the_aim_tilt` (the
hover Aim writer's snapped-vector value).

## 8. Reproduce

```
node tools/_d42_sheets.js --json tools/_d42_sheet_inventory.json
```

Deterministic: the skyline layout is byte-equal across two runs in every
scenario (the tool asserts it); `verify()` passes on every skyline and
maxrects layout. Node-only; no cargo/tests invoked (code-only mode).

## 9. SEAL: the 43-in-562-union confirm (d35 `1afd327b9` research/113 §14-16)

Re-read of d35's RETIRE tip (`1afd327b9`, the 562-name union: 504 pushed +
56 d40-forecast + 12 d35-deferred + 3 d125-tip + 5 d125-deferred, deduped)
against this doc's 43-name inventory (`tools/_d42_sheet_inventory.json`,
mechanical `\bNPC_<id>\b` scan of the 113 text):

- **14 of 43 are ALREADY union-named — but NONE under d42's owner tag.**
  595-601 ride d33's lunar-band rows (§14.2 "d33's lunar band (NPC_595-601,
  7)", Appendix B-2 #52-58); 602/603/608/609 ride c10's rows (§4
  `W4-c10-style68-waterfowl | 8 | NPC_362-365/602/603/608/609`); 580/630
  ride f42's desert nine (§14.2 f42 `75f055fcf`); 619 rides **d11**'s row
  (§14.2 "d11 (NPC_619)" — the proj-ai lane packed the sheet). The §5
  owner-append contract stands for these: the merge adds `W4-d42` to the
  owners arrays (keep-one name, no new row).
- **29 of 43 are union-ABSENT — d42-only demand, the extraction gap.**
  578 579 582 583 584 585 586 590 591 593 594 610 611 612 613 614 616 617
  618 620 621 622 623 624 625 626 627 628 629. Padded area **610,766 px
  (0.611 Mpx, ~74.6 px of bin width)**. Cause: d42 rows the kinds but packs
  NO sheet rows (the §4 NO-pack verdict — the 8382 overflow means the 43
  never enter `STATIC_SHEET_NAMES`), and `_d35_pushed_extract.py` reads
  packed-name diffs, not kind rosters. d42's own `STATIC_SHEET_NAMES`
  (783 rows / 248 NPC rows at `bac408be0`) contains none of the 43 —
  mechanical check, zero hits.
- **The confirm verdict: the 43 ride §10's lazy-load wave BY CONTRACT, not
  by union presence.** Under §10.6/§13.3 step 7 every `NPC_<id>` row leaves
  the eager pack and joins the per-world NPC page built from the landed
  spawn/bestiary census — so the union-absent 29 join the per-world page
  when d42's kinds land (the §14.4 "new claimant kinds are NOT in the
  landed spawn-roster census… post-merge their spawn arms land, so their
  sheets join the per-world NPC page" mechanism, verbatim). No eager-union
  row is owed; the 29-name list above + the §5 dims table are the d35
  merger's append input (the roster-solo page stays 1768 px, scenario G).
  The bestiary grid needs the 29 (+ the 14's slots where the owners' rows
  don't already cover them) in `BESTIARY_NPC_SHEET_IDS` at merge — d42's
  own table is `[u16; 181]` (`assets.rs:1863`), the 43-kind slot append is
  merger-mechanical (the d33 182→183 precedent).
- Cross-check vs the round's other receipts: the 8 dropped kinds' sheets
  (581/587/592/604/605/606/607/615) are ALL union-present under their
  owners (c4/e26/c8 rows, §14.2/Appendix B) — the drops lose no sheet.
