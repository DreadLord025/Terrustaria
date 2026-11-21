# 137 — The g84 inline arms vs the g31 drains: keep-one seam-mapping (W4-g84 final phase)

Worker: code-only subagent on `task/W4-g84-plantera-bulb-draw` (base `8717e938`).
Read-only census of `origin/task/W4-g31-tile-draw-pass-4` (tip `766a5a5a8` —
the d-hunk fold prep `research/136` + the UNION-CONTRACT §8 attach addendum).
No builds/tests run. Rust cites on the g31 branch are `git show
origin/task/W4-g31-tile-draw-pass-4:<path>` line numbers; g84 cites are this
branch's files.

## 0. Verdict — g31 ports BOTH bucket bodies; the g84 inline arms DROP at the split

The g31 union's `render/tile_drains.rs` (2299 ln) ports the **complete**
`DrawMultiTileGrass`/`DrawMultiTileGrassInWind` and
`DrawMultiTileVines`/`DrawMultiTileVinesInWind` bodies as pure
`DrainSprite`-plan drains over the g30 `SpecialPoints.buckets` — including
EVERY kind the g84 follow-up ported per-cell inline. The UNION-CONTRACT §8.3
step 3 (lines 675-678 of the contract on that tip) already rules:

> "The inline grass/vine/sunflower arms DROP (the drains own them); the
> inline 192-leaf arm DROPS onto d60's battery arm with g68's foliage.rs leaf
> body as the keep-one (research/136 C2)."

**Keep-one: g31's drains.** The g84 inline arms (`tiles.rs` `tile.kind == 27`
arm :915-977, the `236 | 238 | 702 | …` family arm :978-1085, the
`is_multi_tile_vine` arm :1086-1122 + the inline-flame gate :1274-1277) are
the interim per-cell z-order approximation until the §8.3 split lands; the
merger deletes them and binds the drains. The g84 sway modules
(`multi_tile_grass.rs`, `multi_tile_vines.rs`) keep their PURE-MATHS helpers
where the drains re-ported them privately (table below) — the merger folds
the duplicates onto ONE copy per the contract's keep-one convention
(lowest-crate / already-wired copy wins; the drains' copies are
`pub(crate)`-private so either direction compiles).

## 1. The drain-pass map (research/136 §0's 16-step table is CAD, NOT the drains)

The 16-step table the brief refers to is research/136 §0's per-cell
`DrawSingleTile` order (CAD steps 1-16: dd dispatch → vision floors → d60
emission → override → gate/black → TE anchors → chest rewrite → shroom cap →
rect/anchor → gate return → final light → slope family → glowmask table →
f15 glow bind → highlight). **Neither grass nor vines appears in it** — the
MultiTileGrass/MultiTileVine kinds never reach `DrawSingleTile` (the `Draw`
switch registers a special point and `continue`s). Their drains are
**post-step** lanes (research/136 §1 vocabulary: "post-step = the ten
non-solid nature steps (`POST_DRAW_NON_SOLID`, drains)"):

`tile_drains.rs::POST_DRAW_DRAIN_ORDER` (:1378-1389, `TileDrawing.cs:421-430`
C# order) + `drain_pass_bucket` (:1394-1407):

| # | DrainPass | bucket (TileCounterType) | C# body | g84 inline counterpart |
|---|---|---|---|---|
| 1 | MultiTileVines | 3 MultiTileVine | `DrawMultiTileVines` :8829-8888 → `…InWind` :9238-9603 | `tiles.rs:1086-1122` + `multi_tile_vines.rs` |
| 2 | MultiTileGrass | 2 MultiTileGrass | `DrawMultiTileGrass` :8738-8805 → `…InWind` :8916-9032 | `tiles.rs:915-977` (27) + :978-1085 (family) |
| 3 | VoidLenses | 6 | :8708-8736 (PortedBodyDrain seam) | — |
| 4 | TeleportationPylons | 8 | :8639-8706 (seam → pylons.rs) | — |
| 5 | MasterTrophies | 9 | :8597-8637 (seam) | — |
| 6 | Grass (WindyGrass) | 1 | `DrawGrass` :8326-8381 | `tiles.rs:882-896` sway + :1123+ generic sway arm |
| 7 | AnyDirectionalGrass | 10 | :8383-8479 | not ported inline (research/09 row) |
| 8 | Trees | 0 | :8052-8304 (seam → foliage.rs) | inline trunk/canopy arms |
| 9 | Vines | 4 | `DrawVines` :8890-8901 (seam → vines.rs) | `vines.rs` strip pass (already the ported body) |
| 10 | ReverseVines | 7 | :8903-8914 + `DrawRisingVineStrip` :9134-9193 | MISSING on both (research/09 row 12) — g31's drain is the first port |

`drain_pass_for_post_step` (:1435-1455) maps the g29 `PostDrawStep` 1:1 onto
these; `EndBatch`/`NatureAfterAllObjects`/the two TE passes map `None`.

## 2. Per-kind seam-mapping — g84 inline arm ↔ g31 DrainPass bucket

### MultiTileGrass (bucket 2) — `drain_multi_tile_grass` (:775-813) + `multi_tile_grass_in_wind` (:657-770)

| Kind(s) | g84 inline (tiles.rs / multi_tile_grass.rs) | g31 drain coverage | keep-one note |
|---|---|---|---|
| 27 sunflower | `sunflower_sway` tiles.rs:928 + the +74 glow band :952-976 | `multi_tile_grass_size` 27→(2,5) :310; the `flag2/num2=74` glow latch :688-691 + `DrainGlow` :744-751 (`SUNFLOWER_GLOW_Y` :647) | drain owns; g84 `sunflower_sway` + its tests DROP/re-pin |
| 236/238/702 bulb + Life Fruit | `grass_sway_2x2` (num 0.15) tiles.rs:1049 | size :311 (2,2); default arm num 0.15 :699-701 | drain owns |
| 485/490/652 | `grass_sway_2x2` tiles.rs:1049 | size :318 (2,2); default 0.15 | drain owns |
| 521-527 dragonfly jars | `grass_sway_2x2` freeze tiles.rs:1047-1051 | size :318; freeze arm num=0 flag=false :695-698 | drain owns |
| 530/651/705 oasis/echo | `grass_sway_3x2` tiles.rs:1008 | size :317 (3,2) | drain owns |
| 489 pinwheel | `grass_sway_2x3` tiles.rs:1017 | size :319 (2,3) | drain owns |
| 493 golf flag | `golf_flag_sway` (flip_xor, ±6 dx) tiles.rs:1025 | size :320 (1,2); the wind-sign flip + ±6 `vector2[0]` shift :729-738 | drain owns |
| 233 detritus | `detritus_sway` (split 3/2 width) tiles.rs:1031 | size :313-316 (`fy != 0 ? 2 : 3`) | drain owns |
| 519 cattail | `climb_cat_tail` + `cattail_sway` tiles.rs:1040 | size :321 `(1, climb_cat_tail)`; the anchor shift `top_y -= size_y - 1` :793-797; the top-row wind gate :692-694; `climb_cat_tail` :265-283 | **`climb_cat_tail` is a duplicate port** (g84 multi_tile_grass.rs:324 vs g31 tile_drains.rs:265) — merger keeps ONE (g31's is private; either direction) |

The num3 row ladder + 0.1 guard + `!flag→0` (:714-720), the bottom-centre
anchor (:672-676), the per-cell `_rand.Next(4)` (:721) and the
`rot = windCycle*num*num3` (:761) match the g84 per-cell maths term-for-term.

### MultiTileVines (bucket 3) — `drain_multi_tile_vines` (:1321) + `multi_tile_vines_in_wind` (:970-1319)

| Kind(s) | g84 inline | g31 drain coverage | keep-one note |
|---|---|---|---|
| 34 chandeliers | `vine_top_left`/`vine_sway`/`draw_vine_cell` | size :294 (3,3); planter_style :836-855 | **`planter_style` duplicate** (g84 multi_tile_vines.rs:102 vs g31 :836) — identical tuples incl. the 9/11/23/37 rows |
| 95/126/444 lanterns/disco/hive | same arm | size :298 (2,2); 126 rides planter_style | drain owns |
| 454 pigronata | same arm | size :295 (4,3) | drain owns |
| 42/270/271/572/581/660 bottles+jars | same arm | size :296 (1,2); bottle_style :859-874 | **`bottle_style` duplicate** (g84 :125 vs g31 :859) — identical incl. the `:9400-9408` dead-write note |
| 91 banners | same arm | size :297 (1,3) | drain owns |
| 465/591/592 war-table/pots/brazier | same arm (the both-columns platform family) | size :299 (2,3); `wide_family` both-columns lift :1007-1011; the 592 `Glow_294` latch :1030-1058 | drain owns |
| 698 dead-cells jar | same arm (lids + icon + wind-cancel) | size :300 (1,1); `DrainJarItem` :881-888; `jar_rarity_color` :913-933 + `rgb_to_hsl` :936-957 | drain owns; the d57 icon colour chain stays d57's on BOTH |
| shared flame tail | `draw_vine_cell`'s tail (shared top-left seed) | `DrainFlame`/`DrainFlameSheet` :890-908 (FireflyJar/LightningBugJar/Glow291 sheets) | drain owns; the g84 inline-flame gate (tiles.rs:1274-1277) DROPS with the arm |
| platform probe | `is_below_a_non_hammered_platform` g84 :88 | g31 :821-829 — identical | **duplicate helper** — keep one |
| size table | `multi_tile_vine_size` g84 :44 | g31 :292-303 — identical | **duplicate table** — keep one |

## 3. What the drains do NOT own (stays inline / other lanes at the merge)

* **The animator row**: g31's `drain_draw_data` (:334-345) feeds
  `Main.tileFrame[type]` = 0 (the `vines.rs` convention, module doc :71-72)
  until S1-G-1f binds it. The g84 inline arms read the LIVE animator
  (`drawdata` + `drawdata_anim`, incl. the 238 `tileFrame*36` row and the
  485/490/489/493 `drawdata_cases_b` arms). At the split the merger must bind
  the animator feed into `drain_draw_data` or the 238/485/490 rows regress —
  recorded here as the ONE feed the g84 side contributes to the merge.
* **The vision-changer chain + light override**: g31's `DrainCellLight` seam
  (:347-352 base light) binds d56's `vision_changer_light`/`light_override`;
  the g84 inline arms use raw `Lighting.GetColor` + the fullbright/Glow-coat
  White override (tiles.rs:1064-1068/:1111). d56's seam supersedes both.
* **The `_rand` dust rolls**: the drains CONSUME `Next(4)` per cell (:721,
  :386) in pass order on the ONE shared `UnifiedRandom` (§8.3 step 4); the
  g84 inline arms roll NOTHING (documented deviation). The merged stream is
  the drains' — the inline arms' zero-roll behaviour must not be carried over.
* **The 238 spore particle** (`TileDrawing.cs:6956`, `_rand.Next(10)==0`):
  CAD step 3, d60's battery (`draw_tiles_emit_particles`, research/136's
  :471-801 row) — NOT a drain, NOT g84's.
* **The z-order**: the inline arms draw in the fused row-major walk (the
  documented approximation); the drains draw at the `PostDrawTiles(false)`
  position in `POST_DRAW_DRAIN_ORDER`. The split is what fixes the order —
  no g84 witness pins the inline position as canonical (the
  `w4_g84_the_vine_arm_draws_before_the_generic_raised_arm` source-order
  witness pins the ROSTER order inside the fused loop and DROPS with it).

## 4. Merger checklist (this doc's actionable output)

1. At the §8.3 F0 split: DELETE tiles.rs :915-977 (27), :978-1085 (family),
   :1086-1122 (vine) + the :1274-1277 inline-flame gate; the g30 registry
   records the special points instead (the `Draw` switch :549-599/:606-687).
2. Bind `drain_multi_tile_grass`/`drain_multi_tile_vines` through
   `drain_one_pass` on the ONE `_rand`; `DrainCellLight` → d56.
3. Keep-one the four duplicate helpers/tables (§2): `climb_cat_tail`,
   `planter_style`, `bottle_style`, `multi_tile_vine_size`,
   `is_below_a_non_hammered_platform` — g31's copies are the drain-private
   ones; the g84 modules' `pub(crate)` copies may be the surviving ones if
   the drains rebind (either direction, ONE copy).
4. Bind the animator row into `drain_draw_data` (the g84 contribution, §3).
5. Re-pin the g84 pure-maths witnesses (grass_sway/golf/detritus/cattail/vine
   sway/style-table tests) onto the surviving helper copies; the
   tiles.rs-scanner witnesses (arm presence/order) DROP with the arms.
6. `bulb_sway` (cfg(test)) + the `sunflower_sway` tests keep their maths
   pins; the anchors.txt rows :899-901 re-point at the drain bodies when the
   arms drop (never delete the rows to silence the canary — re-point).

## 5. Final-phase verification receipt (2026-11-22, code-only)

The three final phases are CLOSED on this branch and the last-3-commit
claims were re-verified against the tree at tip `325034628` by direct
symbol check: phase 1 (`e6c2dcd0d`, doc-only) — this doc + the INDEX row
137 + the §4 merger checklist stand as published; phase 2 (`856a57263`) —
`kill.rs:2305 fn check_jungle_plant` + `BulbSummonEvent :43` /
`TileKill.bulb_summons :179`, `editing.rs:5561 bulb_summon_plantera`,
`spawner.rs:799 spawn_on_player_tile` + `:857 stamp_boss_spawn`, the
`boot_statics` jungle-plant bits (`pipeline.rs:2827-2829`) and all 10
`w4_g84_*` witness fns exist exactly as the message claims; phase 3
(`325034628`, doc-only ALREADY-LANDED) — `scanner.rs:945-953` (the 238
accumulated R/B floor at 0.5, G untouched), `drawdata_anim.rs:303`
(`step_gt_ge(238, 20, 4)`), `drawdata.rs:744-747` (`top = 2`,
`add_fr_y = tile_frame * 36`), the both-engine seeds
(`lighting/mod.rs:531` + `legacy.rs:1260`), the 3 witnesses
(`scanner.rs:1262` / `drawdata_anim.rs:640` / `drawdata.rs:1655`) and the
`anchors.txt` canary row (`scanner.rs | 238 => {`) all verified present.
Nothing owed on THIS branch: the §4 checklist above is merger-owned (the
UNION-CONTRACT §8.3 split), the d60 sparkle / d56 vision / d57 icon lanes
are g31/other-lane-owned per §3, and the reverse-vines 549 gap is g31's
`drain_reverse_vines` (first port there, research/09 row 12) — dropping
the inline arms here would regress to static-generic with no drain wired
(the phase-1 verdict). No builds/tests run (code-only mode); the
witnesses remain WRITTEN NOT RUN.