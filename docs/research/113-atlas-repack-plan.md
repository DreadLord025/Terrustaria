# 113 — Atlas repack plan: the W4 union overflows the 8192 strict-GPU bound

Status: ✅ SEALED (measurement + merger contract complete: §1-6 the
union crisis + 2-page contract, §7-9 the BG-STYLE extension, §10 the
NPC lazy-load census + the one-page lazy union, §11 the FINAL claimant
cross-read + 490-name union, §12 the merger checklist + witness matrix;
§13 the RETIRE phase — the step-7 lazy-load IMPLEMENTATION contract
(port-side load-path census + the exact merger code plan), §14 the
NEW-claimant delta since 490 (552-name union, worst case re-measured),
§15 the FINAL acceptance budgets + GPU-limit check + VRAM min-spec,
§16 the cross-receipts d38d/g72e/g31g; §17 the SEAL-VERIFY pass — the
retire receipt re-verified at 1afd327b, the NEW claimants since 562
(d42 NO-PACK 8382/d43 band/f29 equal-both-ways/e23 MapBGOverlay/g84/
c8/c9 + g10 pets), the **707-name census union** re-measured (lazy ONE
page 7256/7423/+Glow362 7447), and §17.5 the FINAL atlas-family bible —
the 7-step lazy-load merger contract with the SEAL acceptance numbers.
No further census pass is owed; the next atlas work is the merger).
Branch: `task/W4-d35-npc-kinds-71-140` (ATLAS-REPACK phase + extension
phase + FINAL phase + RETIRE phase). Base: `a725acb0` (wave-3 landing).
Code-only mode:
no cargo/tests run; every number below is measured by the pinned sim
harness over the REAL retail `.xnb` dims (LZX first-frame decoder,
`tools/_d35_atlas_sim_pinned.js`), with `verify()` (no overlap incl. PAD,
in-bounds) and double-run determinism on every layout. The bg extension
(§7-9) is measured by `tools/_d35_bg_inventory.js` →
`tools/_d35_bg_inventory.json` (commit `239108c2` + the underworld
own-page follow-up, 27 scenarios, same verify + determinism
discipline, boundary split recursed to N pages). The FINAL phase
(§10-12) is measured by `tools/_d35_npc_lazy.js` →
`tools/_d35_npc_lazy.json` (28 scenarios, same discipline; the
spawn-roster inputs `tools/_d35_npc_census.py` →
`_d35_npc_census.json`; the union re-extraction
`tools/_d35_pushed_extract.py` at the current tips →
`_d35_pushed_sheets.json`, the §1-9 base pinned in
`_d35_pushed_sheets_339.json`). The RETIRE phase (§13-16) re-extracts
at the latest tips (the comment-tolerant row regex recovers e23's 26
MapBG backdrops the FINAL regex silently dropped; 52 branches / 488
pushed names → the 552-name union) and re-measures via
`tools/_d35_retire_measure.js` → `_d35_retire_measure.json` (24
scenarios, same verify + determinism discipline) +
`_d35_retire_gpu.json` (the per-page VRAM caps + the GPU-limit check);
the font-array VRAM is measured by `tools/_d35_font_vram.js` →
`_d35_font_vram.json` (a full multi-frame LZX decompress of the five
retail font XNBs + the `FontAtlas::build` shelf-pack replica, 4 BC2
layers = 268.4 MB, page counts 159/159/159/160/160 matching the Rust
pins). The SEAL phase (§17) re-extracts once more (65 branches / 630
pushed names + d42's 29 NO-PACK forecast = the 707-name census union)
via `tools/_d35_seal_probe.py` / `_d35_seal_delta.py` /
`_d35_seal_union.py` → `_d35_seal_union.json` and re-measures via
`tools/_d35_seal_measure.js` → `_d35_seal_measure.json` (25 scenarios)
+ `_d35_seal_gpu.json`; the committed retire witnesses are preserved
untouched.

## 1. The crisis, re-measured

The sheets phase (`0668fba7`) measured the 17-branch union at **8530 px**
(+338 over 8192). Since then the branch set GREW: re-extraction
(`tools/_d35_pushed_extract.py`, base `a725acb0`) now finds **26 branches
adding 339 distinct sheet names** (was 17/198). New arrivals: d37 (46 NPC
sheets, 211-280 band), d35's own 21 (71-140 band, already on HEAD), d70
(4 sunflare lens sheets — three 1024×1024!), d85 (11 world-creation seed
previews, up to 434×960), e17 (105 map-overlay heads), c39/e18/e23/f40.

**W4-d40 (421-490) is data-only today** (12 batches, `8fe1d6d3`, no
`assets.rs` diff) — its sheets phase is still pending. Its 56 kinds' ids
were extracted from its `npc/data.rs` match arms and FORECAST into the
union (`NPC_430..488`, 56 names, all files present). The task brief's
"~90 more sheets from d38b/d40b" resolves to: d38's 35 are ALREADY pushed
and in the union; d40's 56 are the forecast. d35's 12 deferred sheets
(79/80/84/85/86/103/104/109/110/120/122/140) are measured as a worst-case
add-on.

### Measured heights (width 8192, PAD 2, strict-GPU bound 8192)

`tools/_d35_repack_measure.js` → `tools/_d35_repack_measure.json`.
"area bound" = ⌈padded area / 8190⌉ + 2 — the height NO packer can beat.

| scenario (no boot font) | sheets | padded area | area bound | skyline | maxrects BAF |
|---|---|---|---|---|---|
| base (HEAD a725acb0) | 9518 | 64.53 Mpx | 7881 | **8188** | **7988** |
| + pushed union (339 names, 317 new packs) | 9835 | 72.34 Mpx | **8835** | 9382 (+1190) | 9998 (+1806) |
| + d40 forecast (56) | 9891 | 74.05 Mpx | **9044** | 9443 (+1251) | 9158 (+966) |
| + d35 deferred (12) | 9903 | 74.71 Mpx | **9124** | 9478 (+1286) | 9254 (+1062) |
| + L8b mount (0 new — all 25 already in base) | 9903 | 74.71 Mpx | 9124 | 9478 | 9254 |

Boot-font variants (`font …` rows in the JSON) differ by ≤4 px; the font
sheet (768×8) packs into skyline valleys. All layouts verified + deterministic.

**The decisive fact: the union's padded AREA alone (72.34 Mpx → 8835 px
lower bound) exceeds 8192. No single-page packer — skyline, maxrects, or
any other — can fit the merged W4 wave. A second atlas page is mandatory,
not optional.** With d40 + d35-deferred the bound is 9124 px; even a
perfect packer needs ≥2 pages, and 2 pages (2×8192×8192 = 134 Mpx capacity)
hold the 74.7 Mpx set at ~56% occupancy — comfortable headroom for the
rest of wave 4 (d36/d39/d41+ bands, ~200 more NPC sheets forecast).

### Packer comparison (the promotion question)

- **Skyline (waste tie-break, the landed packer)**: base 8188, union 9382.
- **Maxrects BAF (promoted this phase)**: base **7988 (−200 px vs skyline)**,
  but union **9998 (+616 px WORSE than skyline)**. Maxrects' free-rect
  fragmentation interacts badly with the union's tall-strip shape (60+
  NPC sheets 600-1920 px tall): BAF buries short sheets in side-slivers
  that the skyline's valley-fill would have used. On the d40-forecast set
  maxrects recovers to 9158 (−285 vs skyline) — the ordering is chaotic,
  not monotone. **Verdict: do NOT swap the landed packer.** Maxrects stays
  a measured comparison candidate; skyline remains the production packer
  (its determinism witnesses and the `build_ref` contract are landed).
- The promotion itself: `maxrectsPack(sheets, cap = Infinity)` in the
  pinned harness now takes a height cap and returns `layout[i] = null`
  for sheets with no fitting free rect (spill) instead of throwing.
  `cap = Infinity` reproduces the original unbounded layout exactly
  (witness: `tools/_d35_repack_determinism.js`, `infDeterministic: true`).

### Two-page forms measured (all verified + deterministic)

| form | page 0 | page 1 | sheets on p1 | notes |
|---|---|---|---|---|
| **boundary skyline** (pack unbounded, cut at 8192, re-pack tail) — CHOSEN | 8163-8192 | 1232-1298 | 8868-8941 | below-boundary UVs stable; this IS the Rust contract (§3 step 1) |
| incremental skyline (page 0 = HEAD base packed ALONE) | 8188 | 1924 | 317-385 | UV-stable alternative; needs a pinned pre-W4 name set — rejected on maintainability |
| incremental maxrects | 7988 | 1924 | same | page-0 repack unnecessary |
| spill maxrects (cap 8192, overflow → fresh page) | 8192 | 700-1018 | 7031-7836 | smallest page 1, but maxrects is not the landed packer |

**Chosen fix: a SECOND ATLAS PAGE with the landed skyline packer, in the
boundary-split form** (pack unbounded as today, keep every rect entirely
under the 8192 cap on page 0, re-pack the straddling/overflow tail into
page 1). The incremental form (page 0 = HEAD base packed alone) is the
measured UV-stable alternative — it keeps the sheets-phase witness
numbers (8188) literally valid — but it requires pinning the pre-W4 name
set forever, which 26 interleaving branches make unmaintainable. UV
stability across builds is NOT a correctness requirement (every lookup
is rebuilt from the atlas each build; nothing persists UVs), so the
general form wins on maintainability. Its measured cost: page 0
8163-8192, page 1 1232-1298 px — both pages far under the bound, with
room for the rest of wave 4 (d36/d39/d41+ bands, ~200 more NPC sheets
forecast). The d35 deferred 12 + d40's 56 simply join the input list.

## 2. Census: the single-page assumption (what the merger must touch)

The font atlas ALREADY solved this problem — `font_atlas.rs` packs BC2
pages into 8192×8192 array layers, `SpriteInstance.layer` (u32, 0 = main
atlas, ≥1 = 1-based font layer) selects the texture in `sprite.wgsl`'s
`sample_tex` (sprite.wgsl:256-261). The world atlas is the ONLY single-page
texture left. Census (file:line):

### tr-render (the deep module — small, contained)
- `atlas.rs:46-54` — `SheetRect {x, y, width, height, plain_frame}`: NO
  page field. **Add `page: u32` (0-based).**
- `atlas.rs:56-61` — `Atlas {width, height, pixels, sheets}`: one pixel
  buffer. **Becomes `pages: Vec<AtlasPage>`** (`AtlasPage {height, pixels}`;
  width stays the shared 8192). `Atlas::build`/`build_ref` (atlas.rs:64-165)
  run the skyline pass UNBOUNDED exactly as today, then split: every rect
  entirely under the 8192 cap keeps its page-0 position; the straddling /
  overflow tail re-packs into page 1 (same sort order restricted to the
  tail, same skyline, fresh bin). This is the measured `boundaryTwoPage`
  form (§1 table: page 0 8163-8192, page 1 1232-1298). It is the general
  boundary-split shape — the merger cannot guarantee "HEAD first" ordering
  once 26 branches' const-list rows interleave, so no pinned pre-W4 name
  set is maintained. UV stability across builds is not a correctness
  requirement (every lookup is rebuilt from the atlas each build; nothing
  persists UVs). The tail set is a pure function of the input list, so the
  form is deterministic. The incremental alternative (page 0 = HEAD base
  alone, 8188/1924) is recorded in §1's table but rejected on
  maintainability.
- `atlas.rs:178-201` — `frame_is_opaque`/`rect_is_opaque` index
  `self.pixels` with `self.width` stride: **index `pages[sheet.page]`**.
- `sprite.rs:264-283` — ONE `create_texture_with_data` (D2,
  `depth_or_array_layers: 1`, Rgba8UnormSrgb). **Becomes a D2 ARRAY**
  (`depth_or_array_layers: pages.len()`, LayerMajor upload — the font
  path at sprite.rs:560-611 is the template). View dimension D2Array.
- `sprite.rs:346-355` — bind group layout entry 1: `TextureViewDimension::D2`
  → **D2Array** (entry 4, the font array, is the precedent at :374-383).
- `sprite.wgsl:12` — `var atlas: texture_2d<f32>` → **`texture_2d_array<f32>`**;
  `sample_tex` (sprite.wgsl:256-261) samples `atlas` with
  `i32(layer)` for the main path. **Layer encoding**: keep `layer == 0` =
  main atlas page 0 (every existing `layer: 0` push site — 20+ in
  sprite.rs — stays valid), extend to `layer < MAIN_PAGE_COUNT` = main
  page `layer`, `layer >= MAIN_PAGE_COUNT` = font layer
  `layer - MAIN_PAGE_COUNT`. **The font offset lands at ONE chokepoint**:
  `push_font_glyph` (sprite.rs:1088-1109) is the only producer of font
  instances (its `layer` arg is the 1-based `PageRect.layer` from
  font_atlas.rs:149-153); it adds `MAIN_PAGE_COUNT - 1` so font page 1
  maps to instance layer `MAIN_PAGE_COUNT`. No font_atlas.rs change, no
  caller change. Alternative (cleaner): a separate `page: u32` instance
  field + a `font` flag — but that grows the instance layout (11
  attributes today) and touches every push site; the layer-range
  encoding touches ONLY `sample_tex`, `push_font_glyph`, and the UV
  producers. **Recommended: layer-range encoding with a `MAIN_PAGES`
  uniform** (or a compile-time constant 2, bumped when page 3 arrives).
- `sprite.rs:494-522` — the `sheets` UV map, `font_uv0/1`, `white_uv`:
  divide by `atlas.width/height` — **per-page height**
  (`pages[rect.page].height`); `__white`/`__font` stay on page 0.
- `sprite.rs:23,1311-1329` — `Globals.texel` (half-texel inset,
  sprite.wgsl:112-117): one texel size for the whole atlas. Pages have
  DIFFERENT heights → **per-page texel**: either a `texels: array<vec2<f32>, N>`
  uniform indexed by layer, or (simpler) bake the inset into the UVs at
  push time per page and drop the shader inset for the main path (the
  font path already bakes its inset at build time — sprite.wgsl:110-111
  documents exactly this precedent). **Recommended: bake-at-push**,
  matching the font path; `Globals.texel` stays for the aberration
  offset (sprite.wgsl:281-283, page-0 texel — aberration is a world-pass
  effect and page-0 sheets dominate it; a per-page aberration texel is
  a follow-up if page-1 sprites show it).
- `sprite.rs:218,547,726-727` — `atlas_size: (u32, u32)` on the renderer:
  **becomes per-page** (`page_sizes: Vec<(u32,u32)>` or keep page-0 size
  + a page-height lookup).

### tr-app (the wide surface — mechanical, ~300 sites)
- `assets.rs:2311` — `Lookups.atlas_size: (u32, u32)`: **add
  `page_heights: Vec<u32>`** (or `atlas_sizes: Vec<(u32,u32)>`); every
  `uv_rect(rect, self.atlas_size, …)` call (23 sites in assets.rs, 30
  across tr-app; `uv_rect` def at :3354-3366) reads `page_heights[rect.page]`.
  `uv_rect`'s signature gains the page height (or takes `&Lookups`).
- `assets.rs:3347-3350` — the other UV helper (`sheet_frame_uv`-shaped,
  divides by `atlas.width/height`): same per-page fix.
- `render/sky.rs:430-450` — `sky_tex` → `BgTex {uv, w, h}`
  (render/mod.rs:32-36): **`BgTex` gains `page: u32`**; every consumer
  that pushes a sprite from a `BgTex` sets `layer: tex.page` instead of
  `layer: 0`. `BgTex` is `Copy`, 3 floats + uv today; +1 u32 is free.
- `render/entities.rs:442-448` — `npc_tex` wraps `sky_tex`: inherits the
  page automatically. `NpcAssets.npc_sheets` (HashMap<u16, BgTex>) and
  the ~120 named `Option<BgTex>` fields: no shape change beyond BgTex.
- `assets.rs:2163` — `bestiary_npc_sheets: Vec<Option<SheetRect>>`:
  SheetRect gains `page`, the portrait UV producers
  (`ui/bestiary.rs:369,718,2018`) read the per-page height.
- The `atlas_size` consumers (72 occurrences across tr-app):
  `ui/{mod,panel,slot,chrome,cards,tooltip,uistate,crafting,inventory,
  keyboard,emotes,notifications,npc_chat,achievements,char_creation,
  bestiary}.rs`, `render/{gore,pylons,screen_fx}.rs`, `projectile.rs`,
  `map.rs`/`map_layers.rs` — all follow the ONE pattern
  `let (aw, ah) = (lookups.atlas_size.0 as f32, lookups.atlas_size.1 as f32)`
  then divide a SheetRect's pixel coords. **Mechanical rewrite**: `ah`
  becomes `lookups.page_heights[rect.page]`. `aw` is unchanged (8192 on
  every page). ~30 distinct files, one pattern.
- `app/mod.rs:4692-4705` — the boot build + the `> 8192` warn: the warn
  becomes per-page. NOTE: the boundary form does NOT by itself guarantee
  page 1 ≤ 8192 (it guarantees page 0 ≤ 8192; page 1 is the re-packed
  tail, measured ≤ 1298 px for the whole W4 union). The build must ASSERT
  every page ≤ 8192 and, if a future tail ever exceeds it, recurse the
  split into page 2 (the array form makes N pages free). `app/mod.rs:4880,4904`
  index `atlas.pixels` for the opacity pre-pass → `pages[rect.page].pixels`.
- `app/loop.rs:71`, `app/mod.rs:2776` — `SpriteRenderer::new(gpu, atlas, …)`:
  signature unchanged (the Atlas carries its pages).
- `app/saveglue.rs:947` — `Atlas::build(Vec::new(), 8192)`: still valid
  (one empty page 0 with `__white`).

### Witnesses that must be updated (not deleted)
- `atlas.rs:445-473` (`build_ref_matches_build_bit_for_bit`),
  `:482-507` (determinism), `:513-539` (UV copy), `:545+` (no overlap),
  `:640-660` (the gore-band fit): extend to 2 pages; the ≤8192 assertion
  becomes per-page.
- `assets.rs:3764,3867` — the `atlas.height <= 8192` content witnesses:
  per-page; the full-set witness (`the_full_gore_and_mount_sheet_set_fits…`)
  now asserts page 0 ≤ 8192 AND page 1 ≤ 8192.
- `render/entities.rs:3758-3891` — the d35 packing audit: unchanged in
  spirit (the 21 packed / 12 deferred split), but the deferred 12 become
  PACKABLE once page 1 exists — the merger wave should admit them. In the
  chosen boundary-split form they join the sorted input; the measured worst
  case (+d35deferred scenario) is skyline 9478 unbounded → page 0 8192 /
  page 1 1298 (boundary form), both under the bound.
- `ui/bestiary.rs:2077,2197-2204`, `render/gore.rs:379-413` — synthetic
  `(8192, 8192)` atlas sizes in unit tests: stay valid for page-0 rects.

## 3. Implementation contract for the merger

Order of operations (each step independently mergeable):

1. **tr-render first** (the deep module): `SheetRect.page`, `Atlas.pages`,
   boundary-split `build_ref` (skyline unchanged + unbounded; rects entirely
   under the 8192 cap keep their page-0 position, the straddling/overflow
   tail re-packs page 1 with the same sort order restricted to the tail —
   deterministic: the tail set is a pure function of the input),
   array texture + D2Array binding + `sample_tex` layer-range encoding,
   bake-at-push half-texel inset, per-page UV normalization in the
   renderer's `sheets` map. All atlas.rs witnesses extended to 2 pages.
   **Determinism contract**: same input list → same page assignment →
   same per-page layout → same pixels. The world-swap `build_ref` path
   must produce identical pages (the existing bit-for-bit witness).
2. **tr-app Lookups**: `page_heights`, `uv_rect`/`sheet_frame_uv` per-page,
   `BgTex.page`, `sky_tex`/`npc_tex` propagate it.
3. **The ~30-file mechanical sweep**: every `(aw, ah)` destructure reads
   the rect's page height; every sprite push from a page-aware handle
   sets `layer: page`. The compiler finds every site (the `atlas_size`
   field rename to `page_sizes` forces the issue — recommended: rename
   so no site can silently keep the old single-page math).
4. **Admit the deferred sheets**: d35's 12 (79/80/84/85/86/103/104/109/
   110/120/122/140) move from the deferred list into `STATIC_SHEET_NAMES`
   / `D35_SHEETS`; d40's 56 land in its own sheets phase as normal. The
   union list (§4) is the merge checklist — every name must be packed on
   SOME page; the packing audit witness (`render/entities.rs`) extends
   to assert page membership.
5. **Re-measure**: `node tools/_d35_repack_measure.js` after the merge —
   every scenario must show page 0 ≤ 8192 AND page 1 ≤ 8192. The harness's
   `boundaryTwoPage` (skyline) is the measured reference: pack unbounded,
   keep every rect entirely under the cap on page 0, re-pack the tail.
   The Rust implementation may apply the cap DURING packing (a sheet
   whose placement would exceed 8192 goes to page 1) — a different page
   assignment whose heights are NOT yet measured; the boundary numbers
   (page 1 ≤ 1298 px) are the evidence-backed bound, and any during-
   packing variant must be re-measured in the harness before landing.

**What NOT to do**: do not swap skyline for maxrects (measured worse on
the union, chaotic ordering); do not compress the atlas to BC2/BC3 to
"fit" (the paint/dye/sRGB byte-multiply invariant — ANALYSIS S1-G-4 —
depends on exact RGBA8 texels; BC lossy compression breaks pixel
equality); do not raise the bound past 8192 (the strict-GPU limit is
real: `max_texture_dimension_2d` on the target adapters).

**Memory cost**: page 0 at 8192 px × 8192 × 4 B ≈ 268 MB; page 1 at
≤1298 px (the chosen skyline boundary-split form) ≈ 43 MB. Total
~311 MB — the font array already costs 256 MB/layer (BC2, 8192²), so
this is within the existing budget shape. (The incremental alternative
would be 268 + 63 MB; the maxrects spill form 268 + 33 MB — all the same
order.)

## 4. The union list (407 names, owner branches, priority order)

*(FINAL note: the list has since grown to **490 names** — §11's
re-extraction + the d125 delta; the full regenerated table is
`tools/_d35_union_table.md`, the delta is Appendix B. The 407-name
table below is the §1-9 measurement base, kept for the witness
numbers.)*

Full table: `tools/_d35_union_table.md` (generated by
`tools/_d35_union_table.py` from `tools/_d35_union_list.json`, which
`tools/_d35_union_list.py` builds from the re-extracted
`tools/_d35_pushed_sheets.json` + the d40 forecast + the d35 deferred).
Sorted by padded area DESC — the merge-priority view (biggest consumers
first; the packer itself sorts tallest-first). 339 pushed-union names +
56 d40-forecast (†) + 12 d35-deferred (‡) = 407. All 407 files exist and
decode (0 missing). Total padded area 10.47 Mpx ≈ 1278 px of bin width.

Owner-branch summary (name counts, deduped per branch):

| branch | sheets | note |
|---|---|---|
| W4-e17-map-overlay-layers | 105 | NPC_Head_* + Boss heads (tiny, 20×20-ish) |
| W4-f29-map-overlay-2 | 64 | overlaps e17 heavily (heads + Coin_0..3) |
| W4-d37-npc-kinds-211-280 | 46 | tall strips, NPC_243 78×1920 the tallest |
| W4-f15-glow-bind-tiledraw2 | 45 | Glow_* (small) |
| W4-d38-npc-kinds-281-350 | 35 | tall strips (NPC_291 68×1260) |
| W4-d35-npc-kinds-71-140 | 21 (+12‡) | on HEAD already; deferred 12 admitted by the repack |
| W4-d40-npc-kinds-421-490 | 56† | FORECAST — data-only branch today |
| W4-d85-ui-screens-2 | 11 | Seed_* previews, Seed_Everything 434×960 |
| W4-c24-journey-ui | 11 | Research_FramedPistons 124×1026 |
| W4-e23-bestiary-ui-grid | 10 | Icon_Tags_Shadow 480×150 |
| W4-d84-ui-screens-1 | 8 | Workshop hub icons |
| W4-c10-style68-waterfowl | 8 | NPC_362-365/602/603/608/609 |
| W4-c39-lantern-birthday | 5 | Extra_69-72/134 |
| W4-d34-npc-kinds-1-70 | 5 | NPC_31/34/37/43/46 |
| W4-c4-style44-flying-fish | 4 | NPC_509 138×522 |
| W4-c7-style64-fireflies | 4 | NPC_355/358/654/677 |
| W4-c9-style67-snails | 4 | NPC_359/360/655/677 (677 shared with c7) |
| W4-d70-moonlord-shake-screens | 4 | THREE 1024×1024 lens flares + one 512² — 3.42 Mpx, 33% of the union's area |
| W4-d127-birds-style24 | 3 | WallOfFlesh 190×420, Chain4/12 |
| W4-g71-bg-layers-2 | 2 | Background_26/27 (1024-wide) |
| W4-c6-style59-pumpking-blade | 2 | PumpkingArm, NPC_328 (shared with d38) |
| W4-g57-ui-reforge-anim | 2 | UI/Reforge_0/1 |
| W4-e18/f40/g55/g58/f23 | 1 each | Cursor_11, Extra_134 (shared c39), Extra_259, NPCHappiness, Inventory_Back24 |

Cross-branch duplicates (the loader's name-dedup keeps one rect): NPC_224
(c4+d37), NPC_328 (c6+d38), NPC_355/358 (c7+d37 band), NPC_359/360
(c9+d37 band), NPC_362-365 (c10+d37 band), NPC_677 (c7+c9), Extra_134
(c39+f40), the e17/f29 head set (~50 names), UI/Bestiary/Button_Search
(c24+e23), d84/d85's internal list duplicates. The union is deduped by
name; the per-branch counts above are pre-dedup.

**Priority order for the merger**: (1) the tr-render page plumbing (no
sheet changes, all existing UVs valid on page 0); (2) the mechanical
tr-app sweep; (3) admit d35's deferred 12 + verify d37/d38's pushed sets
pack on page 1; (4) d40's sheets phase lands normally (its 56 go to page
1); (5) the remaining branches' additions append to page 1 as they merge.
The d70 sunflare set (3.42 Mpx across 4 sheets) is the single biggest area consumer —
if page 1 ever tightens, those three are the first candidates for a
dedicated effects page (they are screen-space lens flares, not world
sprites; a third page costs nothing architecturally once the array
exists).

## 5. Determinism evidence

- `tools/_d35_repack_determinism.js` → `_d35_repack_determinism.json`:
  784-sheet synthetic set (the content's shape: 1024² flares, 60 tall
  strips, 720 tiny icons). ALL PASSED: cap=Infinity reproduces the
  original maxrects layout (`infDeterministic: true`, height 1684);
  cap-bounded double-run identical (`capDeterministic: true`); tight-cap
  spill: 16 sheets overflow page 0 (1398 px), page 1 packs them all
  (1684 px, deterministic, verified); skyline unchanged + deterministic.
- `tools/_d35_repack_measure.js`: every scenario's skyline AND maxrects
  layouts double-run identical + `verify()` (no overlap incl. PAD,
  in-bounds); every 2-page form's both pages verified.
- The pinned harness's own `main()` scenarios still run unmodified
  (the maxrects promotion is backward-compatible at cap=Infinity).

## 6. What this branch ships vs documents

SHIPPED (code-side, small): the maxrects BAF promotion in the pinned
harness (cap + spill), the re-extraction of the 26-branch union, the
10-scenario measurement + the 407-name union list with owners, the
determinism witness. All under `tools/`, no Rust touched.

DOCUMENTED (code-side large, ~300 sites across ~30 files + the render
binding change): the page-index plumbing (§2-3). Per the task contract
("implement if code-side small — else document"), the Rust multi-page
implementation is the MERGER WAVE's job: it touches the hot render path
(sprite.rs/wgsl), the Lookups surface every UI module reads, and the
landed packing witnesses — a coordinated change that must land AFTER
the W4 sheet branches merge (or simultaneously with them), not on a
single NPC-kind branch. This doc is its implementation contract.

EXTENSION PHASE (commit `239108c2` + the doc commits after it): §7 the
bg-style demand (the g71/g72 UNWIRED blockers re-measured on the merged
base — the full 358-sheet inventory, per-style areas, realistic
per-world budgets), §8 the lazy-load architecture (the C#
`AssetInitializer`/`LoadBackground` census — vanilla loads ONLY the
drawn styles, the port contract is load-on-demand style sheets, NOT
all-407), §9 the g72 underworld escape (bg-target render-to-texture)
and the d38c cross-note (its maxrects-8170 fit is superseded).

FINAL PHASE: §10 the NPC-sheet lazy-load census (the C# `LoadNPC`
per-draw contract + the full-asset eager-vs-lazy family table + the
measured one-page lazy union + the per-world NPC page budgets), §11
the FINAL claimant cross-read (d38c/d40b/d125b at their current tips,
the 37-branch/424-name re-extraction + the d125 local-tip delta = the
490-name union, Appendix B), §12 the FINAL merger checklist (steps
1-7, the witness matrix, the realistic budgets, the runtime-uniform
and own-page-threshold contracts). New tools: `_d35_npc_census.py`
(the spawn-roster extraction), `_d35_npc_lazy.js` (the 28-scenario
measurement), `_d35_delta_table.py` (Appendix B), the pinned
`_d35_pushed_sheets_339.json` (the §1-9 base). No Rust touched.

## 7. BG-STYLE demand: the g71/g72 blockers re-measured on the merged base

The g71 doc-sweep (`origin/task/W4-g71-bg-layers-2`,
`docs/research/92-doc-sweep-bg.md` §2, commit `e2d085ae`) measured
non-default surface styles at **+336…+1503 px** over a **7847 px**
style-0 baseline (corrupt-4 alone **8193** — one pixel over; the
one-each rolled world **12345**). The g72 lane
(`origin/task/W4-g72-bg-layers-3`) measured the Underworld style-0 set
alone at **8424** over its **7530** baseline. **Both baselines predate
the merged W4 wave** — the same asset set on this branch's HEAD is
8188 px (§1), and the merged worst case is skyline 9478 / boundary
pages 8192+1298. The extension re-measures every style add-on on the
**merged base** (font + pushed-339 + d40-56 + d35-deferred-12, 9904
sheets, 74.72 Mpx): `tools/_d35_bg_inventory.js` →
`tools/_d35_bg_inventory.json` (27 scenarios, `verify()` + double-run
determinism on every layout, boundary split recursed to N pages).

### 7.1 The bg-sheet inventory (all style variants, REAL dims)

**358 sheets** — `Background_0..343` (344, `Main.maxBackgrounds`,
`Main.cs:66583`) + `Backgrounds/Underworld 0..13` (14,
`TextureAssets.cs:616`). **0 missing, all decode.** Total padded area
**200.70 Mpx → area bound 24508 px**: the bg corpus ALONE needs ≥4
pages (measured skyline 38974, boundary pages 7891/8049/7961/5894).
The style tables below are transcribed verbatim from `WorldGen.setBG`
(`WorldGen.cs:7215-7637`), `SetForestBGSet` (`:7639-7768`), the cavern
sets (`Main.cs:53812-54015`) and the animation families
(`Main.cs:19658-19735`, `:61371-61376`); the full per-sheet dims live
in the JSON's `inventory` + `styleRows` arrays.

Per-family style counts and padded areas (Mpx; the roll ranges are
`RandomizeBackgrounds` `WorldGen.cs:7947-8012` +
`RollRandomForestBGStyle` `:8034-8058`):

| family | styles (roll-reachable) | sheets/style | area/style Mpx | heaviest style |
|---|---|---|---|---|
| forest ×4 sets | 0-13 + 31/51/71/72/73 (19 rows) | 3-9 (anim families inflate 4/5/51 to 8-9, 6 to 7) | 1.77-7.14 | **10** (280/281/282, 7.14) |
| corrupt | 0-4, 51, 52 (7) | 3 | 1.28-2.06 | 4 (240/241/242, 2.06) |
| jungle | 0-6 (7) | 3 | 1.65-7.39 | **5** (271/272/273 animated, 7.39) |
| snow | 0-8 + 21/22/31/32/41/42 (15) | 2-5 (mnt+bg) | 1.04-2.84 | 5 (258/259/260, 2.84) |
| hallow | 0-5 (6) | 2-3 | 1.12-6.73 | **2** (219/220/221 animated, 6.73) |
| crimson | 0-6 (7) | 2-3 | 1.28-6.47 | **6** (339/338/337 animated, 6.47) |
| desert | 0-4, 51/52/53 (8) | 2-8 (51-53 = 4 variant rows ×2) | 0.71-**12.33** | **51/53** (8 sheets incl. three 2048×1200 animated, 12.33) |
| ocean | 0-7 (8) | 1 | 0.62-1.17 | 5 (283, 1.17) |
| mushroom | 0-4 (5) | 3 | 1.67-3.33 | 2 (234/235/236 animated, 3.33) |
| underworld | 0-2 (3) | 5 (9 shared between 1/2) | 6.14-**12.33** | **1** (three 2048×1716, 12.33) |
| cavern | 0-21 (22) + ice 0-3 / jungle 0-1 / hell 0-2 axes | 4-7 | 0.07-0.10 | all ≈0.10 (tiny 128-256 px tiles) |

The cavern sets are CHEAP (≤0.101 Mpx per style — the sheets are small
tiled backdrops); the demand is dominated by the surface tree-line
strips (1024-2048 wide × 600-1600 tall, the animated ones 2×2 frames)
and the Underworld parallax sheets. The single biggest bg sheet is
`Background_281` (forest style 10's animated hill, **2048×2048**,
4.20 Mpx); the biggest Underworld sheet is `Underworld 1`
(**2048×1900**, 3.90 Mpx).

### 7.2 The g71 single-style add-ons ON THE MERGED BASE

Same add-on sets as g71's table, packed into the merged worst case
(skyline unbounded → boundary pages; every row verified + deterministic):

| add-on | new sheets | skyline | Δ vs merged 9478 | boundary pages | g71's old number |
|---|---|---|---|---|---|
| corrupt 4 (240/241/242) | +3 | **9900** | +422 | 8186 / 1700 | 8193 (+346 over 7847) |
| snow 7 (267/266/268+269/270) | +5 | 9895 | +417 | 8192 / 1798 | 8323 (+476) |
| hallow 5 (327/326/325) | +3 | 10050 | +572 | 8192 / 1848 | 8370 (+523) |
| snow 5 (258/259/260) | +3 | 9820 | +342 | 8192 / 1684 | 8410 (+563) |
| jungle 6 (302/301/300) | +3 | 10252 | +774 | 8178 / 2146 | 8478 (+631) |
| crimson 6 (339/338/337) | +3 | 10321 | +843 | 8184 / 2204 | 8729 (+882) |
| desert 51 (8 variant sheets) | +8 | **11089** | +1611 | 8164 / 2997 | 9350 (+1503) |
| desert 4 (oasis 248/249/250) | +3 | 9740 | +262 | 8188 / 1572 | 8186 (+339, "fits") |
| mushroom 4 (321/320/319) | +3 | 9770 | +292 | 8190 / 1619 | 8183 (+336, "fits") |
| ocean 5 (283) | +1 | 9777 | +299 | 8152 / 1642 | 8008 (+161, "fits") |
| one-each 8 biomes (g71's rolled world) | +29 | **14067** | +4589 | 8176 / 6247 | 12345 (+4498) |

**Reading**: on the merged base EVERY add-on overflows page 0 — even
g71's three "fits" rows (desert-4/mushroom-4/ocean-5) — because the
merged wave already consumes the headroom their 7847 baseline had.
But **every single-style and one-each world still fits in TWO pages**
(the boundary page 1 grows from 1298 to at most 6247 px). The
two-page contract of §1-3 survives the whole g71 surface demand.

### 7.3 Realistic per-world budgets (the rolls, not one-each-of-the-heaviest)

A real world rolls ONE style per family (four pairwise-distinct forest
styles, `WorldGen.cs:7949-7964`) plus the cavern machine's picks. Two
worlds measured, both including the underworld set and the full
reachable cavern set (4 cave bands + snow/jungle/glowshroom + beach +
deep rows + the boot style-0 set):

| world | new bg sheets | bg area | skyline | boundary pages (mixed) | own-bg-page form (§8.4) |
|---|---|---|---|---|---|
| **seed-42** (g71's Python-reference rolls: forest [2,7,51,11], corrupt 51, jungle 5, snow 5, hallow 4, crimson 4, desert 4, ocean 2, mushroom 2, uw 2; cave bands [1,4,3,0], ice 3, hell 2, jungle 0) | +93 | 42.77 Mpx | 15636 | 8152 / **7225** (2 pages) | pages 0/1 unchanged + **one bg page 7448 px** (3 total) |
| **worst-world** (heaviest rolled style per family: forest [10,5,51,4], corrupt 4, jungle 5, snow 5, hallow 2, crimson 6, desert 51, ocean 5, mushroom 2, uw 1; 4 heaviest cave bands + ALL beach/deep rows — the drunk-hardmode upper bound) | +101 | 70.57 Mpx | **19087** | 8188 / 8184 / **3099** (3 pages) | pages 0/1 unchanged + **two bg pages 8135+2138** (4 total) |
| ALL-VARIANTS (every style of every family + all cavern axes + all underworld — the load-everything upper bound, NOT the contract) | +307 | 181.48 Mpx | 34399 | 7828/8160/7985/8192/570 (5 pages) | 4 own bg pages (6 total) |

**The realistic per-world budget is 3 pages, 4 in the adversarial
worst case** — NOT the 5+ an all-styles loader would need. The C#
never loads all styles (§8); the port must not either. The mixed form
re-packs the union with the bg sheets interleaved (bg strips are tall,
they dominate page 0 and push NPC sheets down); the own-bg-page form
keeps the §1-3 union layout BIT-IDENTICAL on pages 0/1 and appends the
world's bg set as fresh pages — the recommended shape (§8.4).

### 7.4 The g72 underworld numbers on the merged base

| set | sheets | area | skyline | boundary pages | own bg page |
|---|---|---|---|---|---|
| style 0 (Underworld 0-4) | 5 | 6.14 Mpx | 10407 | 8192 / 2313 | 1904 px |
| style 1 (Underworld 5-9) | 5 | **12.33 Mpx** | **11195** | 8118 / 3312 | **1722 px** |
| style 2 (Underworld 10-13+9) | 5 | 7.06 Mpx | 10402 | 8191 / 2329 | 1722 px |
| all three (14 distinct) | 14 | 24.64 Mpx | 12544 | 8188 / 4744 | — |

g72's "style-0 set alone measures 8424" was against its 7530 baseline;
on the merged base style 0 adds +929 px (9478→10407) and the heaviest
style 1 adds +1717 (→11195). **Every single underworld style fits on
ONE extra page** (≤3312 px mixed, ≤1904 px own-page — style 1's three
2048×1716 sheets stack into a narrower-but-shorter column than style
0's 2048×1900 giant) — the atlas-block is a SINGLE-page artifact,
exactly as g72's module docs suspected.
The render-to-target escape (§9.1) remains the faithful alternative
for the underworld stack specifically.

## 8. Lazy-load architecture: C# loads ONLY the rolled/drawn styles

### 8.1 The AssetInitializer census (exact)

- `AssetInitializer.Load(asyncLoad)` (`Main.cs:10855`) →
  `LoadTextures(mode)` where mode = AsyncLoad(2)/ImmediateLoad(1)
  (`AssetInitializer.cs:78-85`).
- **ALL 344 background assets register `DoNotLoad`**:
  `AssetInitializer.cs:239-242` —
  `for (num25 = 0; num25 < TextureAssets.Background.Length; num25++)
  TextureAssets.Background[num25] = LoadAsset<Texture2D>(
  "Images/Background_" + num25, AssetRequestMode.DoNotLoad);`
  (`Background.Length = Main.maxBackgrounds = 344`,
  `TextureAssets.cs:683`, `Main.cs:66583`).
- **ALL 14 Underworld assets register `DoNotLoad`**:
  `AssetInitializer.cs:390-393` (`"Images/Backgrounds/Underworld " +
  num49`).
- The ONLY eager background loads at boot: **`LoadBackground(0)` and
  `LoadBackground(49)`** (`AssetInitializer.cs:422-423`).
- `LoadAsset` is a thin `Main.Assets.Request<T>(name, mode)`
  (`AssetInitializer.cs:656-659`); `Request`'s default mode is
  `ImmediateLoad` (`AssetRepository.cs:82`) — so every later
  `LoadBackground` call loads synchronously on first use.

### 8.2 The LoadBackground census (per-draw, idempotent)

`Main.LoadBackground(int i)` (`Main.cs:59039-59074`): guards
`i >= 0 && State == NotLoaded`, requests the asset, caches
`backgroundWidth/Height[i]`, and halves both for the 21 animated ids
(219-221/235/271-273/281/302/303/305/307/309/311/313/315/317/326/
337/338/341 — the 2×2 frame sheets, `GetBackgroundRect`
`Main.cs:61220-61262`). **Never called with a style the world did not
roll.** Call-site census (49 sites):

| caller | ids loaded | when |
|---|---|---|
| `DrawBackground_UpdateBackgroundStyles` (`:54009-54015`) | the 7-slot cavern set of the CURRENT + OLD underground style (`:53812-54008` tables) | every frame the cavern bg draws; the old set rides the `ugBackTransition` cross-fade |
| `OldDrawBackground` (`:54344-54349`) | the 6-slot legacy set (`BackgroundEnabled == false`) | same, legacy path |
| `DrawSurfaceBG` cloud arm (`:59376-59377`) | `cloudBG[0..1]` = **112/113** (fixed, `Main.cs:66676`) | `cloudBGAlpha > 0` |
| hallow shimmer (`:59487-59488`) | **18/19** (fixed) | `HolyTileCount > 0` |
| forest hill (`:59498`) | `treeMntBGSet1[1]` (rolled) | set > -1 |
| `DrawSurfaceBG_DrawBackMountainsLayer` (`:59729`) | any far-mountain id: the four sets' mnt rows, **23/24/25/42**, snowMnt, **246/247** (hallow-3) | per far layer with alpha > 0 (`:59804-59963`) |
| `DrawSurfaceBG_{Mushroom,Crimson,Snow,Hallow,Jungle,Desert,Corrupt,Forest}` (`:60038…:61451`) | the rolled 3-row style arrays (`setBG` tables) | per front layer with `bgAlphaFrontLayer[l] > 0` (`:59529-59599`) |
| `DrawSurfaceBG_GoodEvilDesert` (`:60885/:60907`) | **26/27** (fixed fallback) when the desert variant row is absent | evil/hallow desert cross-fade |
| forest 173 frame swap (`:61374`) | **251 + counter/10 % 4** | forest style 6's tree line |
| `DrawUnderworldBackgroundLayer` (`:52687-52691`) | `TextureAssets.Underworld[rolled]` — direct `Request`, NOT LoadBackground | the underworld pass runs |

**The vanilla working set at any instant**: ≤7 cavern ids ×2
(cross-fade) + ≤3 ids ×10 front layers + ≤10 far ids + 5 underworld
≈ **≤60 sheets**, and the lifetime set of a world is bounded by its
ROLLED styles (the §7.3 per-world numbers), never the 358-sheet
corpus. The four desert variant rows (51-53) are the one place a
single roll expands to 8 sheets — all four rows load because the
cross-fade can show any of them (`:60870-60884`).

### 8.3 What the port does today (the census of the gap)

- `load_world_sheets` (`assets.rs:128-223`) ALREADY loads per-world
  selectively — but only the **five owned surface families**
  (corrupt/jungle/hallow/crimson/mushroom, `world.rs:1442-1456`), from
  the world's persisted style bytes. g70's branch extends it to the
  four forest sets + animation families (its `load_world_sheets`
  `:62-79`); g71 adds snow/desert/ocean style bytes. The merged
  loader is the union of those arms.
- `STATIC_SHEET_NAMES` (`assets.rs:239-280`) pins the style-0/default
  bg ids (0/1/2/3/4/6/7/8/20/21/22/23/28/35-39/50-52/112/113/125-127/
  185-187) — the always-loaded core, matching the C#'s fixed ids.
- The cavern machine (`render/bg.rs`) pins `undergroundBackground = 0`
  and the underworld stack is UNWIRED (g72) — their sheet demand is
  NOT yet in the loader. When they wire, their ids join the world
  half, NOT the static half.
- The boot rebuild (`app/mod.rs:4642-4705`) re-packs
  `world ++ static ++ font` on every world swap with the static half
  cached — the per-world bg set is already the only world-dependent
  part of the pack. **The swap path is the natural lazy-load seam.**

### 8.4 The port contract: load-on-demand style sheets (extends §3)

The merger's 5-step contract (§3) gains a **step 6** and a page-
assignment rule:

6. **Per-world bg pages (the lazy-load form)**: the bg style sheets
   do NOT join the union input list. `Atlas::build_ref` packs the
   union (static + world tiles/walls/NPC/etc.) into pages 0..N exactly
   as §3 step 1, then appends the world's rolled bg set as FRESH
   pages (N+1..M), boundary-split recursively. Measured (§7.3): the
   union stays 8192/1298; a realistic world adds ONE page (7448 px),
   the adversarial worst adds TWO (8135+2138). On a world swap the
   bg pages are re-packed from the new world's rolls while pages
   0..N are layout-stable for a fixed union input — the same
   determinism contract as §3 step 1, per page.
   - **Which ids**: the merged `load_world_sheets` bg arm = the five
     owned families + g70's four forest sets (+94/180 animation
     families + 173 frames) + g71's snow/desert/ocean + g72's cavern
     picks (4 bands + snow/jungle/glowshroom + the reachable
     beach/deep rows for the world's evil/hallow state + hell rows)
     + the rolled underworld set. The fixed ids (18/19/24/25/42/
     246/247/26/27/112/113) stay wherever they are today (static or
     world) — they are ≤11 sheets, not the demand driver.
   - **NOT all-407**: the 358-sheet corpus is 200.70 Mpx (≥4 pages
     alone, §7.1); loading it would cost ~800 MB of atlas and pack
     styles no world ever shows. The C# census (§8.1-8.2) is the
     1:1 justification: `DoNotLoad` registration + per-draw
     `LoadBackground` IS load-on-demand.
   - **True per-draw laziness is NOT required**: the C# loads a
     style's sheets the first time its layer draws; the port may
     pack the whole ROLLED set at world load (a superset — the
     cross-fade can reach every rolled row within seconds). This
     keeps the atlas immutable during play (no mid-frame re-pack,
     no UV invalidation, the G6 cache untouched). A future
     memory-tightening pass could defer the cavern/underworld pages
     until first descent, but that is an optimisation, not the
     contract.
   - **`SheetRect.page` already carries the assignment** (§2): bg
     lookups (`sky_tex`, `BgTex`) read the page like every other
     sheet; no new lookup machinery. The `MAIN_PAGE_COUNT` constant
     becomes `atlas.pages.len()` at runtime (the layer-range
     encoding's font offset must use the ACTUAL page count, not a
     compile-time 2 — §2's "bumped when page 3 arrives" note is
     superseded: with per-world bg pages the count varies by world,
     so the uniform form is REQUIRED, not optional).
   - **Memory**: pages 0/1 ≈311 MB (§3) + one bg page ≤268 MB
     (7448 px ≈ 244 MB realistic) — ~555 MB realistic, ~640 MB
     worst-case (two bg pages). The font array's 256 MB/layer is
     the existing budget shape; if this tightens, the bg pages are
     the first candidates for the §9.1 render-to-target escape
     (they are exactly the sheets the C# draws outside the tile
     atlas anyway).

**What this does NOT change**: §3 steps 1-5 stand as written for the
union pages; the deferred-12 admission, the d40 forecast, the
mechanical sweep are all unaffected. The bg extension only adds
pages AFTER the union and the runtime page-count uniform.

## 9. Cross-lane notes

### 9.1 The g72 underworld escape: bg-target render-to-texture

g72's prep (read on `origin/task/W4-g72-bg-layers-3`, NOT touched):
`render/underworld_bg.rs` (the verbatim `DrawUnderworldBackground`/
`Layer` geometry, `Main.cs:52665-52796`, UNWIRED — its module doc
cites the 8424 single-page overflow), `render/bg_styles.rs` (the
22-case cavern style machine, `Main.cs:53812-54095`, UNWIRED), and
`render/bg_target.rs` (the capture/consume seams, commit `86a4471d`).
The bg_target census CORRECTS the lane's earlier routing claim: the
underworld stack lands in **screenTarget** (DrawBG under
`Filters.Scene.BeginCapture`, `Main.cs:62098-62100`/`:64049-64053`;
`FilterManager.EndCapture` composites skyTarget THEN screenTarget,
`FilterManager.cs:146-152`), the cavern tree in **backgroundTarget**
(`WorldSceneLayerTarget`, `Main.cs:57290`→`:55200-55218`).

**The escape, documented**: the C# draws the underworld stack
through `Main.spriteBatch` under the sky matrix into screenTarget —
the sheets are NEVER in the tile atlas. The port's faithful shape is
the same: draw the 5 rolled Underworld sheets as plain textures into
the d69/S1-G-13 target family (`tr_render::targets`, landed:
`RenderTargetFamily`/`LayerCamera`/`layer_composite_position`), NOT
as atlas pages. Cost: 5 standalone textures (≤12.33 Mpx ≈ 49 MB for
style 1) + one composite pass — vs +1717 px of atlas (§7.4). The
g72 modules are already pure-plan adapters (`UnderworldLayerPlan`
carries src/scale/positions); the wiring lane binds them to a
texture-array draw instead of `BgTex` lookups. **This escape is
underworld-specific**: the surface/cavern style sheets draw through
the SAME `Main.spriteBatch` but the port's sky pass already reads
them from the atlas — for them the §8.4 per-world pages are the
cheaper path (no new draw path, the existing sky_tex plumbing
works). The two forms compose: union pages 0/1 + per-world surface/
cavern bg pages + the underworld stack in screenTarget.

### 9.2 d38c cross-note: the maxrects-8170 fit is SUPERSEDED

*(FINAL re-read at d38's current tip `6f9dd02db`: §11 — d38 published
its own addendum confirming the supersession and endorsing the
boundary-split form.)*

d38's sheets phase (`origin/task/W4-d38-npc-kinds-281-350`, commits
`96a48386`/`518df928`) measured the cumulative d38+d35 set at skyline
**8452** (+260 over) and proposed **maxrects BAF 8170 (22 px
headroom)** as the single-page fix. **That fit is superseded by this
plan**: the re-extracted 26-branch union (339 names, §1) is 72.34
Mpx — area bound 8835 — so NO single-page packer fits the merged
wave, maxrects included (measured 9998 on the union, WORSE than
skyline's 9382, §1). d38's 35 sheets are already IN the union
(Appendix A); its deferred siblings ride the boundary split like
everyone else. The d38 branch's ANALYSIS note should point here at
merge time; the maxrects promotion it asked for landed in the pinned
harness (§1) as a measured comparison, not the production packer.

### 9.3 Ownership boundaries (unchanged)

The bg ROLLS stay g71's UNWIRED `worldgen/bg_rolls.rs` (the RNG-shift
pin-move is a worldgen lane's call, its §2); the cavern/underworld
DRAW machines stay g72's UNWIRED modules; the forest sets/animation
families stay g70's. This extension owns ONLY the measurement + the
merger contract — no Rust touched, no rolls wired, no pins moved.

## 10. The NPC-sheet lazy-load census: the union shrinks to ONE page

The bg census (§8) asked whether the C# loads NPC sheets eagerly too. It
does not. **Every `TextureAssets.Npc` entry registers `DoNotLoad`**
(`AssetInitializer.cs:127-129`), and `Main.LoadNPC` (`Main.cs:59084-59090`)
fires from `NPC.SetFrameSize` (`NPC.cs:66661-66663`) at the top of
`DrawNPCDirect` (`Main.cs:22513-22516`) — **per DRAWN NPC**, exactly the
bg contract. Measured: `tools/_d35_npc_lazy.js` →
`tools/_d35_npc_lazy.json` (28 scenarios, verify() + double-run
determinism on every layout, boundary split recursed to N pages; the
spawn-roster inputs extracted by `tools/_d35_npc_census.py` →
`_d35_npc_census.json`).

### 10.1 The LoadNPC census (per-draw, idempotent)

`LoadNPC(i)` guards `State == NotLoaded`, requests synchronously
(`AssetRepository.Request` default `ImmediateLoad`). Call sites:

| caller | ids loaded | when |
|---|---|---|
| `NPC.SetFrameSize` (`NPC.cs:66663`) ← `DrawNPCDirect` (`Main.cs:22516`) | the drawn NPC's own type | every frame an NPC draws (also the bestiary icon draw, `UnlockableNPCEntryIcon.cs:206`) |
| bestiary icon first hover (`UnlockableNPCEntryIcon.cs:185`) | the hovered entry's type | the bestiary grid opens — EVERY entry's sheet loads on hover, seen or not |
| stardust dragon arm (`Main.cs:24278-24279`) | 403/404 | the 402 arm draws |
| empress arm (`Main.cs:24683`) | 492 | the 636 arm draws |
| bestiary scutlix dummy (`Main.cs:25227`) | 391 | the 390 icon |
| mech-queen destroyer (`Main.cs:25509`) | 136 | 134 draws under `IsMechQueenUp` |
| probe projectile (`Main.cs:31967`) | 139 | proj 591 draws |
| eye-spawn projectile (`Main.cs:33179`) | 4 | proj 1130 draws |
| lunatic-cultist projectile (`Main.cs:34410`) | 594 | proj 1025 draws |
| leashed critters (`LeashedCritter.cs:199`, `ShimmerFlyLeashedCritter.cs:60`) | the leashed type | a critter is leashed/shimmered |
| credits roll (`CreditsRollComposer.cs:353/:615`) | 550/199 | the credits play |

**The vanilla NPC working set at any instant** = the on-screen NPC types
(the 200-slot roster, typically ≤20 distinct kinds) — the lifetime set
of a world is bounded by what it SPAWNS plus what the bestiary hovers,
never the 697-sheet corpus. The port packs its named NPC subset EAGERLY
today (`STATIC_SHEET_NAMES` rows → `npc_sheets`); that is a superset
deviation, not the vanilla contract.

### 10.2 The full-asset eager-vs-lazy table (every atlas family)

C# status from `AssetInitializer.LoadTextures` (mode = Async/Immediate
at boot); areas from REAL `.xnb` dims (`_d35_npc_lazy.json` `families`):

| family | sheets | area Mpx | C# boot | C# load-on-demand | port today |
|---|---|---|---|---|---|
| NPC | 697 | 29.62 | **DoNotLoad ALL** (`:127-129`) | `LoadNPC` per drawn NPC (`:59084`) | EAGER named rows — **the gap this section closes** |
| Item | 6134 | 5.44 | **DoNotLoad ALL** (`:115-126`, TextureCopyLoad aliases share) | `LoadItem` per drawn item/slot (`:59076`, `DrawItem :38608`) | EAGER enumerate-all (`assets.rs:1353`) |
| Projectile | 575 | 5.17 | **DoNotLoad ALL** (`:131-133`) | `LoadProjectile` per drawn proj (`:59092`, `DrawProjDirect :27935`) | EAGER spawnable_kinds+734 |
| Gore | 1402 | 3.07 | **DoNotLoad ALL** (`:135-137`) | `LoadGore` per active gore (`:59100`, `DrawGore :21888`) | EAGER enumerate-all (E-4b) |
| Wall | 366 | 40.12 | **DoNotLoad ALL** (`:139-141`) | `LoadWall` per drawn wall (`:59108`, `WallDrawing.cs:116`) | WORLD-SELECTIVE (usage scan) ✓ |
| Tiles | 754 | 47.70 | **DoNotLoad ALL** (`:143-145`) | `LoadTiles` per drawn tile (`:59116`, `TileDrawing.cs:223/:1482`) | WORLD-SELECTIVE (usage scan) ✓ |
| Background | 344 | 176.06 | **DoNotLoad ALL** (`:239-242`); only 0/49 eager (`:422-423`) | `LoadBackground` per drawn layer (§8.2) | WORLD-SELECTIVE 5 families + static defaults (§8.4 extends) |
| Underworld | 14 | 24.65 | **DoNotLoad ALL** (`:390-393`) | direct Request per drawn layer (`Main.cs:52687`) | UNWIRED (g72) |
| ItemFlame/Wings/hair/armor/acc | — | — | **DoNotLoad ALL** (`:147-237`) | `LoadItemFlames`/`LoadWings` per drawn player layer (`PlayerDrawLayers.cs:649/:3429`) | not packed (player layers partial) |
| Buff | 400 | 0.46 | EAGER (`:418-420`) | — | EAGER ✓ |
| NpcHead / NpcHeadBoss | 81/40 | 0.06/0.04 | EAGER (`:406-412`) | — | EAGER named rows ✓ (e17/f29 append) |
| Extra | 316 | 18.25 | EAGER (`:363-366`) | — | EAGER named rows ✓ |
| GlowMask | 384 | 13.11 | EAGER (`:352-355`) | — | EAGER GLOW_SHEET_NAMES ✓ |
| Cloud | 42 | 0.89 | EAGER (`:459-462`) | — | EAGER ✓ |
| TreeTop/Branch | 64 | 1.90 | EAGER (`:524-531`) | — | EAGER ✓ |
| Liquid+LiquidSlope | 30 | 0.11 | EAGER (`:467-471`) | — | EAGER ✓ |
| Misc/water | 15 | 1.15 | EAGER (LiquidRenderer) | — | EAGER ✓ |
| Waterfall | 28 | 0.55 | EAGER (`:472`) | — | EAGER ✓ |
| Flame | 18 | 1.57 | EAGER (`:382-385`) | — | EAGER ✓ |
| Fonts | 5 DynamicSpriteFont | — | EAGER (`:87-94`) | — | EAGER font array ✓ |

**Reading**: the C# eager set is the UI/HUD/sky furniture (small);
the four BIG corpora (NPC 29.6 + Item 5.4 + Projectile 5.2 + Gore 3.1
= 43.3 Mpx) plus tiles/walls/bg are ALL `DoNotLoad`. The port already
matches the contract for tiles/walls/bg; NPC/Item/Projectile/Gore are
eager supersets. Item/Projectile/Gore are cheap (13.7 Mpx combined,
~1.7 px of bin width per 0.01 Mpx) and their draw sites are scattered
across every slot/tooltip/projectile arm — **their eager packing stays**
(a documented superset, the E-4b precedent). NPC is the one family
where lazy loading changes the page budget materially (29.6 Mpx, the
tall strips that dominate page 0).

### 10.3 The measured shrink: union minus NPC = ONE page

| scenario | sheets | area Mpx | area bound | skyline | boundary pages |
|---|---|---|---|---|---|
| merged-407 base (§7, re-witnessed) | 9904 | 74.72 | 9125 | 9478 | 8192 / 1298 |
| **lazy union (407 base − all NPC_)** | 9463 | **54.17** | **6617** | **7059** | **ONE page 7059** |
| merged-final-424 (the FINAL re-extraction, §11) | 9979 | 75.91 | 9271 | 9601 | 8176 / 1439 |
| **lazy union (424 base − all NPC_)** | 9497 | **54.77** | **6690** | **7162** | **ONE page 7162** |
| NPC corpus alone (all 697 files) | 697 | 29.62 | 3619 | 3808 | one page 3808 |

The 441 NPC sheets in the 407 union carry 20.55 Mpx — pulling them out
drops the union from 9478 (2 pages) to **7059 (ONE page, 1133 px
headroom)**. Even the FINAL 424-union lazy form is one page (7162).
The whole NPC corpus would fit one own page (3808) — but the contract
is the per-world set, not the corpus:

### 10.4 The realistic per-world NPC sets (the port roster census)

Extracted from the landed code (`_d35_npc_census.py`): the ambient
spawner's 51 kinds (`spawner.rs` production arms), the world_sim spawn
sites' 128 kinds (invasions/dd2/boss summons/shimmer release), the 31
moon-ladder kinds (`pumpkin/frost_moon_defaults`), the 39 `townNPC=true`
vanilla ids (NPC.cs SetDefaults census, incl. the 637/638 + 678-684
town-slime OR chains), the bestiary ENTRIES' 270 net ids, and the 10
cross-draw ids (§10.1). Sheet id = `texture_net_id` override else the
bestiary fold + ENTRIES net id (the Hide kinds draw their own sheets).

| world stage | NPC sheets | area Mpx | own page(s) |
|---|---|---|---|
| fresh world (town 39 + ambient 51, deduped) | 87 | 3.52 | **1668 px** |
| long-lived (town + 141 spawn-site + 31 moon + 10 cross-draw) | 195 | 15.51 | **2152 px** |
| bestiary opened (every ENTRIES id hovered) | 289 | 19.73 | **2566 px** |
| ALL-NPC upper bound (NOT the contract) | 697 | 29.62 | 3808 px |

Every realistic set fits ONE own page with ≥5600 px headroom. The
bestiary is the lifetime upper bound of the lazy contract: hovering
every card loads every entry sheet — still one page.

### 10.5 The realistic total page budget (union + bg + NPC)

Stacking the §7.3 bg pages (seed-42 realistic 7448 px / worst-world
8135+2138) with the NPC own pages:

| form | union pages | bg pages | NPC page | total | atlas MB |
|---|---|---|---|---|---|
| eager union + seed-42 bg (§8.4 as written) | 8192/1298 | 7448 | — | 3 | ~555 |
| eager union + worst bg | 8192/1298 | 8135/2138 | — | 4 | ~648 |
| **lazy union + seed-42 bg + fresh NPC** | **7059** | 7448 | 1668 | **3** | **~530** |
| **lazy union + seed-42 bg + long-lived NPC** | **7059** | 7448 | 2152 | **3** | **~546** |
| lazy union + seed-42 bg + bestiary NPC | 7059 | 7448 | 2566 | 3 | ~560 |
| lazy union + worst bg + long-lived NPC | 7059 | 8135/2138 | 2152 | 4 | ~639 |
| lazy-424 union + seed-42 bg + fresh NPC | 7162 | 7448 | 1668 | 3 | ~534 |
| lazy-424 union + worst bg + bestiary NPC | 7162 | 8135/2138 | 2566 | 4 | ~656 |

**The page COUNT is unchanged (3 realistic / 4 adversarial) — the win
is page-0 headroom and memory**: the eager union's page 0 is pinned at
8192 with page 1 at 1298; the lazy union is ONE 7059-px page with 1133
px of headroom for the rest of wave 4 (d36/d39/d41+ bands) BEFORE a
second union page is needed, and the NPC demand rides its own page
that re-packs per world. The union layout is world-independent again
(the §3 determinism contract strengthens: pages 0..N are a pure
function of the merged branch set, never of the world's rolls).

### 10.6 The port contract amendment (step 7, extends §3/§8.4)

7. **NPC sheets join the per-world lazy half**: the `NPC_<id>` rows
   leave `STATIC_SHEET_NAMES`' eager pack and join the world-dependent
   input. At world load the port packs the world's RESIDENT set (the
   saved roster's kinds + the town roster) plus the SPAWNABLE set (the
   spawner/invasion/moon kind tables — all statically known, the
   `_d35_npc_census.json` extraction is the witness) as FRESH pages
   after the union, exactly the §8.4 bg form. The bestiary-opened set
   is the lifetime superset: packing it at world load (289 sheets,
   2566 px, one page) keeps the atlas immutable during play — the same
   rolled-set-superset argument as §8.4, and it is STILL one page.
   - **Which ids**: `texture_net_id(kind)` for every kind the world can
     spawn or house (the census tables), NOT `net_id()` — the Hide
     kinds draw their own sheets (the 96/97/99/100/112 precedent).
   - **The draw path is unchanged**: `npc_sheets` is built from the
     atlas by name at load; a kind whose sheet is absent keeps its
     box rest (the deferred-kind contract). No mid-play re-pack.
   - **Item/Projectile/Gore stay eager** (§10.2): 13.7 Mpx combined,
     scattered draw sites, the E-4b superset precedent. Their lazy
     form is a future memory pass, not this contract.
   - **`MAIN_PAGE_COUNT` runtime uniform** (§8.4) covers the NPC pages
     too — the page count now varies by world AND by bestiary state
     if the bestiary set is deferred; packing it at load avoids that.
   - **Memory**: ~530-560 MB realistic (vs ~555 eager), ~639-656 MB
     adversarial — the NPC page costs 55-84 MB but frees 20.6 Mpx
     (~84 MB) from the union pages; net ≈ −25 MB realistic.

**What this does NOT change**: §3 steps 1-6 stand; the boundary-split
form, the deferred-12 admission, the d40 forecast, the mechanical
sweep are unaffected. The NPC lazy form is an INPUT-LIST change (which
names go in the union vs the world half), not a packer or binding
change — it composes with the bg pages and the g72 screenTarget escape.

## 11. FINAL cross-lane re-read (the claimant tips, 2026-11-21)

The §9.2 d38c cross-note read d38 at `518df928`; the branch has since
pushed its crossnote phase. Re-read at the CURRENT tips:

- **d38c** (`task/W4-d38-npc-kinds-281-350`, tip `6f9dd02db`, pushed):
  published `docs/research/125-d38-crossnote-atlas-addendum.md` —
  measured with d40's REAL 56 sheets that **maxrects BAF dies WITHIN
  the d40b wave** (8278, +86 over; the three-wave area bound alone
  8258 > 8192), withdrew its BAF-promotion proposal, and ENDORSED the
  boundary-split skyline 2-page form. Its 8188-vs-8232 ledger
  reconciliation (different bases, both correct) agrees with §1.
  **Inventory-current: YES** — its 35 sheets are in the union
  (Appendix A), its numbers are a SMALLER-set independent reproduction
  of §1's verdict, no delta to the union list. Its bestiary pin
  (216 as-pushed + 10 d34 + 16 d35 + 43 d37 + 38 d40 = 323 final) is
  the portrait-slot merger checklist, unchanged.
- **d40b** (`task/W4-d40-npc-kinds-421-490`, tip `8fe1d6d3`, pushed):
  STILL data-only (12 batches, ALL_KINDS 326, no `assets.rs` sheet
  rows). The 56-id forecast stands verbatim (d38's addendum §3
  confirms the id list against the batch commits). **Inventory-current:
  YES as a forecast** — the 56 names ride Appendix A with the dagger;
  when d40b's sheets phase pushes, the re-extraction picks them up as
  real rows and the dagger retires.
- **d125b** (`task/W4-d125-empress-r7-drain`, LOCAL tip `2dfe026d`,
  ahead of its pushed origin tip `d8047a10`): the sheets phase packed
  `NPC_636`/`NPC_661` (skyline 8116/8117 SOLO) and the draw phase added
  `Extra_188` (the halo ring, 8141/8141) while DEFERRING the five tall
  overlay strips `Extra_157/158/159/160/187` (+0.985 Mpx) to this
  repack wave (its `research/125-empress-sheets-ledger.md` §2).
  **Inventory-current: NO — DELTA APPENDED**: the pushed-tip extraction
  misses all 8 names (the branch is ahead of origin). The union list
  now carries them explicitly: +3 tip names (636/661/Extra_188) +5
  deferred (‡). Measured on the merged base: +636/661 → skyline 9665
  (pages 8177/1492); +Extra_188 → 9687 (8177/1502); +deferred-5 →
  9880 (8152/1720). Under the §10 lazy form the NPC pair rides the
  per-world NPC page (both are spawnable kinds — 636 the boss, 661
  the summon critter) and the Extra strips ride the union's page-0
  headroom (1.0 Mpx ≈ 122 px, fits the 1133 px lazy headroom).
  d125's ledger note "page-1 append under the incremental form" is
  superseded by the boundary form (§1) like everyone else's.

**The FINAL re-extraction** (`tools/_d35_pushed_extract.py` re-run at
the current origin tips): **37 branches / 424 names** (was 26/339,
+85 new, 0 gone). New arrivals: g8 (34 NPC sheets, the kind-wire
band), d16 (15 Extra), c8 (9), g21 (9), d34 (+10), c4 (+8), g70 (8:
Sun2/Sun3/moons/sunflares), d23 (6), e26 (4), c3 (3), e10/g56/f46/
g55/etc (1-2 each). With the d125 local-tip delta the union list is
**490 names** (424 pushed + 56 d40-forecast + 12 d35-deferred + 3
d125-tip + 5 d125-deferred, deduped; the g8/d34/c4/d23 rows overlap
the d35/d37 bands heavily). Total padded area 12.92 Mpx. The merged
worst case re-measures: skyline **9601** (424) / **9880** (+d125 full
+deferred), boundary pages 8176/1439 → 8152/1720 — **the two-page
contract holds with ≥6400 px of page-1 headroom**, and the §10 lazy
form keeps the union itself at ONE page (7162).

## 12. The FINAL merger checklist (the witness matrix)

The merger wave executes §3 steps 1-7 in order. Each step's witness:

| step | work | witness (must pass before the next step) |
|---|---|---|
| 1 | tr-render: `SheetRect.page`, `Atlas.pages`, boundary-split `build_ref`, D2Array binding, layer-range encoding, bake-at-push inset | `atlas.rs` witnesses extended to 2 pages; `build_ref_matches_build_bit_for_bit`; the pinned harness `boundaryNPages` reproduces the Rust layout (same input → same pages, `_d35_repack_measure.json` reference) |
| 2 | tr-app Lookups: `page_heights`, per-page `uv_rect`/`sheet_frame_uv`, `BgTex.page` | the `content_lookups` test family; every `uv_rect` site reads the rect's page |
| 3 | the ~30-file mechanical sweep (`atlas_size` → `page_sizes` rename forces every site) | compiler clean; the packing audit (`render/entities.rs`) asserts page membership for every union name |
| 4 | admit d35's deferred 12 + d125's deferred 5; d40's 56 land in its sheets phase | the union list (Appendix A, 490 names) — every name packed on SOME page; `the_d35_strips_divide_into_their_vanilla_rows` + the d125/d38 strip witnesses |
| 5 | re-measure: `node tools/_d35_repack_measure.js` + `_d35_npc_lazy.js` | every scenario page ≤ 8192; the lazy union ONE page ≤ 7162; the per-world NPC/bg pages ≤ 2 |
| 6 | per-world bg pages (§8.4): the rolled-set loader arm, fresh pages after the union, `MAIN_PAGE_COUNT` runtime uniform | the seed-42 pageform (7448 px) + worst-world (8135+2138) reproduce; world-swap bit-stability witness (pages 0..N identical across swaps) |
| 7 | per-world NPC pages (§10.6): the NPC rows leave the eager pack, the resident+spawnable set packs per world | the census sets reproduce (fresh 1668 / long-lived 2152 / bestiary 2566 px, one page each); `npc_sheets` builds by name; the deferred-kind box rest survives |

**Realistic budgets (the merger's acceptance numbers)**: 3 pages
realistic (union 7059-7162 + bg 7448 + NPC 1668-2566), 4 adversarial
(worst bg 8135+2138); ~530-560 MB realistic, ~639-656 MB adversarial;
page-0 headroom ≥1000 px for the rest of wave 4. **Runtime uniform**:
`MAIN_PAGE_COUNT = atlas.pages.len()` at every font-offset site (the
`push_font_glyph` chokepoint, `sprite.rs:1088`) — REQUIRED, the count
varies by world. **Own-page thresholds**: a per-world set that ever
exceeds 8192 recurses the boundary split (the harness's
`boundaryNPages`); the measured sets are ≤2566 (NPC) / ≤8135 (bg), so
one page each today, but the recursion is the contract, not the
measurement.

**Determinism witnesses (all measured, this branch)**:
`_d35_repack_determinism.json` (784-sheet synthetic, cap/spill/inf),
`_d35_repack_measure.json` (10 scenarios × 2 packers × verify +
double-run), `_d35_bg_inventory.json` (27 scenarios, the canonical
re-run SHA256 `23E17DC5…`), `_d35_npc_lazy.json` (28 scenarios, verify
+ double-run, boundary recursion; the FINAL cross-run witness: a full
fresh re-run with no cache is semantically identical — 0 diffs,
families + npcCorpus + every scenario byte-equal — the JSON
re-serialized in canonical script order, cross-run SHA256 of the
canonical form
`A098AE10A2BC8B85AEF9AA82F7E68707B80EFC3930ED7B600699E42C7CD903BD`).
The BG-STYLE fresh re-run witness
(`6a2a155a`) proves the harness itself is deterministic across runs.

## 13. The step-7 lazy-load IMPLEMENTATION contract (the port-side census)

§10.6 published the step-7 amendment as a CONTRACT (which names leave
the eager pack, which set packs per world). This section makes it an
IMPLEMENTATION spec: the exact port load path today, the exact seam the
merger edits, and the load-on-first-draw semantics it reproduces. It is
the step-7 witness the §12 checklist's row 7 points at. Code-only: no
Rust touched; every citation is a read of the landed tree at this tip.

### 13.1 The port's NPC-sheet load path today (eager-STATIC_SHEET_NAMES)

The NPC corpus rides the EAGER static half, end to end:

| stage | site | what it does |
|---|---|---|
| name list | `assets.rs:239` `STATIC_SHEET_NAMES` | **269 `NPC_<id>` rows** (of the const's ~1070 rows) — the named NPC subset, packed eagerly |
| decode | `assets.rs:1315` `load_static_sheets` → `:1319-1324` | `for name in STATIC_SHEET_NAMES { load_png(dir, name, Raw) }` — every NPC row decoded at boot, world-independent |
| split | `assets.rs:113` `load_world_images` → `(world_half, static_half)` | the NPC rows are in the STATIC half (the `:99-108` doc: "backgrounds, NPCs, items, UI" are world-independent) |
| pack | `app/mod.rs:4692` `Atlas::build_ref(&images, 8192)` | one page today; the NPC strips are the tall consumers that pin page 0 |
| lookup build | `assets.rs:2324` `Lookups::build` → `:2510` `bestiary_npc_sheets` | `atlas.sheet("NPC_<id>").copied()` per `BESTIARY_NPC_SHEET_IDS` slot (`:1893`, 197 entries) |
| draw map | `render/entities.rs:450` `make_npc_assets` → `:505-522` `npc_sheets` | the 13 family lists (`PILLAR_SHEETS`…`D35_SHEETS`) chained, `npc_tex(atlas, "NPC_<id>")` per id → `HashMap<u16, BgTex>` (`:297`) |
| named fields | `render/entities.rs:452-655` | ~140 dedicated `npc_tex(atlas, "NPC_<id>")` fields (slime/eye/bosses/town…) alongside the map |
| draw | `render/entities.rs:2193` `draw_npcs` → `:3280` | `let Some(tex) = tex else { continue }` — an absent sheet SKIPS the draw (the deferred-kind contract) |

**The packing audit**: `npc_tex` (`entities.rs:442-448`) carries
`debug_assert!(STATIC_SHEET_NAMES.contains(&name))` — the M4-NPC-3v bug
guard (a name asked for but never packed is a silent no-draw). **This
assertion is the step-7 edit's tripwire**: when the NPC rows leave
`STATIC_SHEET_NAMES`, every `npc_tex` call for an NPC name fails the
assert in debug builds until the audit is re-pointed at the per-world
set (§13.3 step 7c).

**The bestiary path**: `ui/bestiary.rs:587` `entry_sheet` resolves
`lookups.bestiary_sheet_index(entry.net_id)` → `bestiary_npc_sheets[slot]`
(the `:579-593` doc: the literal-slot reads drifted twice, BUG_REPORT §15).
The grid icon + info-page portrait both draw through it (`:704`, `:802+`).
So the bestiary's NPC sheets are the SAME `NPC_<id>` atlas rects as the
in-world draw — one pack serves both, exactly the C# `LoadNPC` sharing.

### 13.2 The C# contract being reproduced (load-on-first-draw)

§10.1's census, restated as the semantics the port must match:

- **Registration is `DoNotLoad` for ALL 697** (`AssetInitializer.cs:127-129`)
  — nothing NPC decodes at boot.
- **`LoadNPC(i)` fires per DRAWN NPC** (`Main.cs:59084-59090`, guard
  `State == NotLoaded`, synchronous `ImmediateLoad`), from
  `NPC.SetFrameSize` (`NPC.cs:66661-66663`) at the top of `DrawNPCDirect`
  (`Main.cs:22513-22516`). The first frame a kind draws, its sheet loads;
  every later frame is a no-op (idempotent).
- **The bestiary hover loads EVERY entry** (`UnlockableNPCEntryIcon.cs:185`)
  — opening the grid and hovering a card loads that entry's sheet whether
  or not the kind was ever seen in-world. The bestiary-opened set is the
  lifetime superset of the lazy contract (§10.4).
- **10 cross-draw sites** load a sheet for a kind that is NOT the drawn
  NPC's own type (§10.1's table): the stardust dragon arm (403/404), the
  empress arm (492), the bestiary scutlix dummy (391), the mech-queen
  destroyer (136), the probe projectile (139), the eye-spawn projectile
  (4), the lunatic-cultist projectile (594), the leashed critters
  (`LeashedCritter.cs:199`, `ShimmerFlyLeashedCritter.cs:60`), the credits
  roll (550/199). These are why the per-world set must include the
  cross-draw ids, not just the spawnable kinds.

**The working-set bound**: at any instant the C# holds only the on-screen
NPC types (the 200-slot roster, typically ≤20 distinct kinds); over a
world's lifetime it holds what the world SPAWNS plus what the bestiary
HOVERS — never the 697-sheet corpus. The port's eager 269-row pack is a
superset deviation; step 7 closes it.

### 13.3 The exact port-contract edits (step 7, the merger's code plan)

The amendment is an INPUT-LIST change plus a per-world pack arm — NOT a
packer or binding change (§10.6). The merger edits, in order:

**7a. The NPC rows leave the eager pack.** Delete the 269 `NPC_<id>` rows
from `STATIC_SHEET_NAMES` (`assets.rs:239`). They join a new
`npc_sheet_names_for_world(world, roster) -> Vec<String>` helper that
returns the world's RESIDENT + SPAWNABLE + bestiary-superset ids as
`NPC_<texture_net_id>` names. **Which ids**: `texture_net_id(kind)`
(`entities.rs:1061`) for every kind the world can spawn or house — NOT
`net_id()` (the Hide kinds draw their own sheets, the 96/97/99/100/112
precedent). The roster census is `_d35_npc_census.json` (the spawner /
world_sim / moon-ladder / town / bestiary-ENTRIES / cross-draw tables,
§10.4); the bestiary-opened superset (289 sheets) is packed at world load
so the atlas stays immutable during play (§13.4).

**7b. The per-world NPC pack arm.** In `build_atlas_inner`
(`app/mod.rs:4615`), after the union pages pack, append the world's NPC
set as FRESH pages — exactly the §8.4 bg form. The seam is the existing
`load_world_sheets` world half (`assets.rs:128`): the NPC names decode
there (or in a sibling `load_world_npc_sheets`), concatenate AFTER the
static half, and `Atlas::build_ref` boundary-splits them onto their own
page(s). The union layout (pages 0..N) is a pure function of the merged
branch set, never of the world's rolls (§10.5's strengthened determinism
contract); the NPC pages (N+1..M) re-pack per world.

**7c. Re-point the packing audit.** `npc_tex`'s
`debug_assert!(STATIC_SHEET_NAMES.contains(&name))` (`entities.rs:444`)
becomes a check against the WORLD's packed NPC set (the union + the
per-world NPC names), not the static const. The `make_npc_assets` map
(`:505`) and the ~140 named fields (`:452-655`) build by name from the
atlas exactly as today — a kind whose sheet is absent keeps its
`let Some(tex) else { continue }` skip (`:3280`, the deferred-kind box
rest). **No draw-path change**: `npc_sheets` is still built from the
atlas by name at load; only WHICH names are packed changes.

**7d. `MAIN_PAGE_COUNT` runtime uniform.** The layer-range encoding's
font offset (`sprite.wgsl:256` `sample_tex`, the `push_font_glyph`
chokepoint `sprite.rs:1088`) must use `atlas.pages.len()` at runtime, not
a compile-time 2 — the page count now varies by world AND by bestiary
state if the bestiary set is deferred (§8.4's requirement, extended to
the NPC pages). Packing the bestiary superset at load avoids the
bestiary-state variation; the world variation remains, so the uniform is
REQUIRED.

**7e. The world-swap seam.** `swap_world` (`app/mod.rs:2682`) already
rebuilds the atlas via `build_atlas_reusing_static` (`:4601`, the static
half cached). The NPC pages ride the world half, so a swap re-packs them
from the new world's roster while pages 0..N stay bit-stable (the §3
step-1 determinism witness, per page). The static cache
(`App.static_sheets`) no longer carries the NPC rows — they move to the
world half, so the cache shrinks by the 269-row decode (the ~1.4 s boot
decode drops accordingly).

### 13.4 Atlas-page-append-at-load, immutable-in-play

The contract packs the world's NPC set ONCE at world load (the
bestiary-opened superset, 289 sheets / 2566 px / one page, §10.4) and
NEVER re-packs during play. This is the same rolled-set-superset argument
as §8.4's bg pages:

- The C# loads a kind's sheet on its first draw; the port packs the whole
  spawnable+bestiary superset at load (a superset — every kind the world
  can show is already resident). The first-draw `LoadNPC` becomes a no-op
  lookup (`npc_sheet(id)` returns the packed rect).
- **No mid-frame re-pack, no UV invalidation, the G6 cache untouched** —
  the atlas is immutable from world-load to world-swap. A kind outside the
  superset (none, by construction) would keep its box rest.
- **The bestiary superset is the lifetime bound**: hovering every card
  loads every entry sheet in the C#; the port pre-packs them all, so the
  bestiary open is also a no-op lookup. Still ONE page (2566 px, ≥5600 px
  headroom).
- A future memory-tightening pass could defer the bestiary-only sheets
  until first hover (true per-draw laziness), but that re-introduces the
  bestiary-state page-count variation and a mid-play re-pack — it is an
  optimisation, NOT the contract (§8.4's identical note for the bg pages).

### 13.5 What step 7 does NOT change

§3 steps 1-6 stand; the boundary-split form, the deferred-12 admission,
the d40 forecast, the mechanical sweep, the bg pages, the g72 screenTarget
escape are all unaffected. Item/Projectile/Gore stay eager (§10.2: 13.7
Mpx combined, scattered draw sites, the E-4b superset precedent). The
`npc_sheets` map, the named fields, the draw arms, the bestiary
`entry_sheet` resolution are all unchanged — only the INPUT LIST (which
names pack eagerly vs per-world) and the audit's reference set move. The
NPC lazy form composes with the bg pages and the underworld escape; it is
the last input-list amendment of the repack wave.

## 14. The NEW-claimant delta since 490 (the RETIRE re-extraction)

The §11 FINAL re-extraction stood at 37 branches / 424 pushed names → the
490-name union. The RETIRE re-fetch (`git fetch origin --prune` +
`tools/_d35_pushed_extract.py` re-run, 2026-11-21) finds **54 branches /
504 pushed names** → the **562-name union** (504 pushed + 56 d40-forecast
+ 12 d35-deferred + 3 d125-tip + 5 d125-deferred, deduped; the d40/d35/
d125 sets overlap the pushed rows heavily). **+72 genuinely-new names, 0
gone** (Appendix B-2). Total padded area 13.48 Mpx ≈ 1646 px of bin width.

### 14.1 The extraction fix: the comment-tolerant row regex

The FINAL regex `^\+\s+"([^"{} ]+)",?\s*$` matched only bare `+  "Name",`
rows. e23's bestiary-backdrop set carries a trailing `// comment` on every
row (`+  "MapBG5",  // dungeon`), so the FINAL extraction **silently
dropped all 26 MapBG backdrops** — they were never in the 490 union. The
RETIRE regex `^\+\s+"([^"{} ]+)",?\s*(//.*)?$` tolerates the comment;
`tools/_d35_retire_probe.py` confirms e23 is the ONLY branch with
comment-row misses (26 names, all MapBG). This is a measurement-harness
correction, not a new claim: e23's backdrops were always packed on its
branch, the union table just never listed them.

### 14.2 The named claimants (the brief's five, at their current tips)

| claimant | tip | atlas delta | verdict |
|---|---|---|---|
| **g66c** (player-trail-dust) | `7213f3c71` | **0 sheet rows** | NO atlas claim — pure dust wiring (`player_trail_dust.rs`/`player_wing_dust.rs` etc., 7464 insertions, zero `assets.rs`/`atlas.rs` diff, zero `atlas.sheet`/`load_png` refs). The dust draws sample EXISTING sheets; nothing new packs. |
| **g84b** (plantera-bulb-draw) | `e6c2dcd0d` | **+2: `Extra_260` (90×36), `Glow_294` (36×432)** | the bulb's glow overlay + the plantera-bulb Extra strip. Both ride the union (Appendix B-2 #7/#46). `Glow_294` is a `GLOW_SHEET_NAMES`-family row (the f15 glow set), `Extra_260` an Extra row. |
| **e23b** (bestiary-ui-grid) | `a4b972bde` | **+30: 26 MapBG backdrops + `UI/Scrollbar` + `Stat_Gold/Platinum/Silver`** | the 26 MapBG are the §14.1 regex recovery (always packed, now listed); the 4 UI rows are the bestiary info-page's coin denominations + Blue-theme scrollbar (`NPCStatsReportInfoElement.cs:179-198`, `UIScrollbar.cs:61`). All small (≤7.8 kpx each). |
| **f42** (sandstorm-2) | `75f055fcf` | **+9 NPC: 546/61/69/78/79/80/580/581/630** | the desert family's sheets (the spawn arm `NPC.cs:3949-4018`). NPC_79/80 were d35-DEFERRED (now real on f42); the rest are new. Under the §10 lazy form ALL nine ride the per-world NPC page (desert spawnable kinds), NOT the union. |
| **g34e** (drawdata-wire-2) | `595debb8d` | **0 sheet rows** | NO atlas claim — pure drawdata/cage wiring (`drawdata_cases_a/b.rs`, `render/tiles.rs`, the cage-light `LightParams.cage_mode` feed). Zero `assets.rs`/`atlas.rs` diff, zero new sheet refs. |

**The other new arrivals** (the re-extraction's full +72, Appendix B-2):
d33's lunar band (NPC_595-601, 7), d39's critters (NPC_356/366/367/377,
4), d41's band (NPC_494-497, 4), d9's proj-NPCs (NPC_454-459/521, 7 —
**NPC_454-459 were d40-FORECAST, now packed by d9's proj-ai lane**, the
forecast dagger retires for those six), d23 (NPC_516/666, 2), g5's caster
(NPC_172/693, 2), d11 (NPC_619), f47's shimmer (NPC_676), g82's rope/coil
(Chain16/38/39 + Chains_4-7, 7), f41's birthday (Armor_Head_195 +
Player_HairAlt_1, 2), g12 (Shroom_Tops), c46 (Flame + Ghost), f22 (Chat2),
f28 (OneDropLogo), d13 (Extra_66).

### 14.3 The re-measured worst case (the 562-name union)

`tools/_d35_retire_measure.js` → `_d35_retire_measure.json` (24
scenarios, verify() + double-run determinism on every layout, boundary
split recursed to N pages):

| scenario | sheets | area Mpx | area bound | skyline | boundary pages | atlas MB |
|---|---|---|---|---|---|---|
| merged-407 base (§7, re-witnessed) | 9904 | 74.72 | 9125 | 9478 | 8192 / 1298 | 311.0 |
| merged-final-504 (the RETIRE pushed union) | 10051 | 76.46 | 9338 | 9757 | 8190 / 1602 | 320.9 |
| + d125 full tip (636/661/Extra_188) | 10054 | 76.55 | 9349 | 9739 | 8174 / 1612 | 320.7 |
| **+ d125 deferred 5 (the WORST case)** | 10059 | 77.73 | 9493 | 9922 | **8160 / 1754** | 324.9 |
| **lazy union (504 − all NPC_)** | 9544 | **55.10** | **6730** | **7140** | **ONE page 7140** | 234.0 |
| **lazy union (worst − all NPC_)** | 9550 | **56.30** | **6877** | **7305** | **ONE page 7305** | 239.4 |

**The two-page contract holds with ≥6400 px of page-1 headroom** (worst
page 1 = 1754, cap 8192). **The lazy union stays ONE page** (7140 px,
1052 px headroom). The +72 new names split 25 NPC-band kinds + 47 non-NPC
rows: the NPC kinds are pulled out by the lazy filter (they ride the
per-world NPC page), and the 47 non-NPC rows (the 26 MapBG recovery + the
small UI/Extra/Chain/Armor rows, ≤47 kpx each) add only 0.33 Mpx — the
lazy union's height moves 7162 → 7140 (the skyline's chaotic ordering,
§1; the area bound rises monotonically 6690 → 6730, the honest metric).
The lazy-union SHEET count is stable at 9544 across the retire snapshots
because the 47 non-NPC rows were already in the first retire extraction
(488); the latest +16 names (504) are all NPC-band kinds the filter
removes. The NPC-band growth (d33/d39/d41/f42/d9/d23/g5) rides the
per-world NPC page, NOT the union — exactly the §10 contract's intent.

### 14.4 The per-world NPC page under the new claimants

The new claimant kinds are NOT in the landed spawn-roster census
(`_d35_npc_census.json` reads THIS branch's spawner/world_sim/bestiary;
the new kinds live on their own branches). Post-merge their spawn arms
land, so their sheets join the per-world NPC page. The augmented sets
(the 31 new-claimant spawnable ids added to the census sets):

| world stage | NPC sheets | area Mpx | own page |
|---|---|---|---|
| long-lived + new claimants | 226 | 15.81 | **2160 px** (one page) |
| bestiary-opened + new claimants | 320 | 20.03 | **2566 px** (one page) |

The bestiary superset height is UNCHANGED (2566 px) — the new kinds are
small (≤256 px strips) and pack into the existing page's headroom. **Every
realistic set still fits ONE own page with ≥5600 px headroom.** The
realistic total budget is unchanged: 3 pages (~533-568 MB) / 4 adversarial
(~647-660 MB), §15.

## 15. The FINAL acceptance budgets (per-page VRAM caps + the min-spec)

§12's acceptance numbers, re-measured on the 562-name union and extended
with the GPU-limit check the brief asks for: is the ~530 MB realistic
atlas OK against the GPU limit, and what is the minimum spec? Measured by
`tools/_d35_retire_measure.js` → `_d35_retire_gpu.json` (the per-page
caps + the total-VRAM stack) and `tools/_d35_font_vram.js` →
`_d35_font_vram.json` (the font-array layer count).

### 15.1 The per-page VRAM caps (the strict-GPU bound is per-texture)

The atlas is a D2 ARRAY of RGBA8 pages (4 B/px). The strict-GPU limit is
`max_texture_dimension_2d = 8192` (wgpu `Limits`, the target adapters) —
a PER-TEXTURE dimension bound, so each page must be ≤8192×8192 =
**268.4 MB**. The boundary-split form guarantees every page ≤8192 px
tall (§3 step 1), so every page is under the cap BY CONSTRUCTION:

| form | pages (px) | per-page MB | max page | under 268.4 cap? |
|---|---|---|---|---|
| lazy union (504) | 7140 | 234.0 | 234.0 | ✓ |
| lazy union (worst) | 7305 | 239.4 | 239.4 | ✓ |
| seed-42 bg page | 7448 | 244.1 | 244.1 | ✓ |
| worst bg pages | 8135 / 2138 | 266.6 / 70.1 | 266.6 | ✓ |
| NPC own page (bestiary) | 2566 | 84.1 | 84.1 | ✓ |
| eager union page 0 | 8192 | 268.4 | 268.4 | ✓ (at the cap) |

**Every page of every realistic/adversarial form is under the per-texture
cap.** The eager union's page 0 sits AT 268.4 MB (8192 px pinned); the
lazy union's single page is 234-239 MB (the headroom the lazy form buys).
The worst bg page (8135 px = 266.6 MB) is the largest realistic page, still
1.8 MB under the cap.

### 15.2 The GPU-limit check: is ~530 MB OK?

**wgpu exposes NO total-VRAM limit.** `Limits` carries
`max_texture_dimension_2d` (8192), `max_texture_array_layers` (256), and
per-binding size bounds — but no aggregate memory cap. The total atlas
allocation is a budget the DRIVER must satisfy, not a hard API limit. So
"530 MB OK?" has two answers:

1. **Per-texture: YES, unconditionally.** Every page ≤8192² (268.4 MB),
   the array layer count ≤4 (well under `max_texture_array_layers` 256).
   No single texture exceeds any wgpu limit. The atlas binds as one
   `texture_2d_array` with ≤4 layers — valid on every adapter that
   supports 8192² textures at all.
2. **Total allocation: YES on the target spec, with headroom.** The
   realistic atlas is 533-568 MB; the adversarial worst is 647-660 MB.
   Adding the font array (268.4 MB, §15.3) and the render-target family
   (127.1 MB, §15.3) gives **928-963 MB realistic / 1042-1056 MB
   adversarial** of GPU texture memory. A 2 GB adapter (the min-spec,
   §15.4) holds this with ~1 GB spare; a 1 GB adapter does NOT (the
   realistic stack alone is ~930 MB before the backbuffer/depth/misc).

**The verdict**: the ~530 MB atlas is OK — it is not the binding
constraint. The binding constraint is the TOTAL texture stack (~930 MB
realistic), which sets the min-spec at 2 GB VRAM (§15.4). The lazy form's
win is real but modest at the total level: it saves ~31 MB vs eager
(533 vs 564 realistic) by freeing the NPC strips from page 0; the page
COUNT is unchanged (3 realistic / 4 adversarial).

### 15.3 The full GPU texture stack (atlas + font array + targets)

The atlas is not the only GPU texture allocation. The complete stack:

| component | size | source |
|---|---|---|
| main atlas (RGBA8 D2 array) | 533-660 MB | §15.1, the measured pages |
| **font array (BC2 D2 array)** | **268.4 MB** | `font_atlas.rs`: 4 × 8192² layers, 1 B/px (BC2). Measured by `_d35_font_vram.js`: the five retail fonts ship 797 pages (159/159/159/160/160, matching the Rust pins), shelf-packed into **4 layers** |
| render-target family (RGBA8) | 127.1 MB | `targets.rs`: 7 world-layer `WorldSceneLayerTarget` at 2304×1584 (1920×1200 view + 192 `offScreenRange` each side, HiDef) + 3 full-screen 1920×1080 |
| **TOTAL realistic** | **928-963 MB** | atlas 533-568 + font 268.4 + targets 127.1 |
| **TOTAL adversarial** | **1042-1056 MB** | atlas 647-660 + font 268.4 + targets 127.1 |

The font array is a FIXED 268.4 MB cost (the retail fonts are
locale-independent, always loaded — `load_fonts` `assets.rs:1627`). It is
BC2 (1 B/px), NOT RGBA8 — the port uploads the shipped compressed surfaces
verbatim like XNA (`font_atlas.rs:1-10`, the `Bc2RgbaUnormSrgb` sampling),
so it is 4× cheaper than a decompressed font atlas would be (~900 MB). The
render-target family is the d69/S1-G-13 `targets.rs` layout (the g72
underworld screenTarget escape composites into this family, §9.1, adding
no new allocation).

### 15.4 The min-spec (the documented recommendation)

**Minimum GPU: 2 GB VRAM, `max_texture_dimension_2d` ≥ 8192,
`max_texture_array_layers` ≥ 4.** Reasoning:

- The realistic texture stack is ~930 MB (§15.3); the adversarial worst is
  ~1056 MB. A 2 GB adapter holds the worst case with ~990 MB spare for the
  backbuffer, depth, the post-process targets, and driver overhead.
- A 1 GB adapter FAILS the realistic stack (~930 MB + backbuffer/misc
  exceeds 1 GB). It is below the min-spec.
- The 8192² dimension bound is the strict-GPU floor (§1: the target
  adapters' `max_texture_dimension_2d`); an adapter below 8192 cannot hold
  ANY page and is unsupported (the atlas would need a smaller page width,
  a different contract).
- The array-layer bound (≥4) is trivially met (every modern adapter
  supports ≥256 layers); it is listed for completeness.

**Recommended GPU: 4 GB VRAM** — the adversarial stack (~1056 MB) plus a
comfortable margin for higher resolutions (the render-target family scales
with the backbuffer; a 4K backbuffer raises the 3 full-screen targets from
1920×1080 to 3840×2160, +~75 MB) and future wave content. The page-0
headroom (≥1000 px lazy, §12) absorbs the rest of wave 4 without a new
page; the VRAM headroom absorbs the resolution scaling.

**This is a RECOMMENDATION, not a hard gate**: wgpu does not expose a
total-VRAM limit, so the port cannot refuse to run on a 1 GB adapter at
the API level — the allocation would simply fail (or the driver would
spill to system memory, a performance cliff). The merger should log the
atlas + font + target budget at boot (`log_atlas_diagnostics`,
`app/mod.rs`) and warn if the adapter's reported budget is below the
realistic stack. The min-spec is the documented floor for a playable
experience, not an API enforced bound.

### 15.5 The acceptance numbers (the merger's gate, FINAL)

Restating §12's gate on the 562-name union, with the VRAM caps added:

- **Page count**: 3 realistic (lazy union 7140-7305 + bg 7448 + NPC
  1668-2566), 4 adversarial (worst bg 8135+2138). UNCHANGED from §12.
- **Per-page**: every page ≤8192 px (≤268.4 MB), the boundary-split
  guarantee. The largest realistic page is the worst bg page (8135 px =
  266.6 MB).
- **Atlas VRAM**: ~533-568 MB realistic, ~647-660 MB adversarial.
- **Total texture VRAM**: ~928-963 MB realistic, ~1042-1056 MB
  adversarial (atlas + 268.4 MB font array + 127.1 MB targets).
- **Page-0 headroom**: ≥1000 px lazy (7140 vs 8192 = 1052 px) for the rest
  of wave 4.
- **Min-spec**: 2 GB VRAM / 8192² / ≥4 array layers (§15.4); recommended
  4 GB.
- **Runtime uniform**: `MAIN_PAGE_COUNT = atlas.pages.len()` at every
  font-offset site (the `push_font_glyph` chokepoint, `sprite.rs:1088`) —
  REQUIRED, the count varies by world (§13.3 step 7d).
- **Own-page thresholds**: a per-world set that ever exceeds 8192 recurses
  the boundary split (the harness's `boundaryNPages`); the measured sets
  are ≤2566 (NPC) / ≤8135 (bg), one page each today, but the recursion is
  the contract, not the measurement.

## 16. The cross-receipts (d38d / g72e / g31g atlas obligations)

The three lanes that publish atlas obligations INTO this repack, re-read
at their CURRENT tips (the retire re-fetch). Each receipt records what the
lane owes the merger and what this plan owes the lane — the §11 cross-read
extended to the retire tips.

### 16.1 d38d (`task/W4-d38-npc-kinds-281-350`, tip `1ba0395c7`)

d38 published `docs/research/125-d38-crossnote-atlas-addendum.md` (read at
`6f9dd02db` in §11; the FINAL phase added §6-10). Its atlas obligations:

- **The eager-form verdicts STAND** (§1-3): the BAF-promotion withdrawal
  (maxrects dies WITHIN the d40b wave, 8278 +86 over), the boundary-split
  skyline endorsement, the 8188-vs-8232 ledger reconciliation (different
  bases, both correct). §11's "Inventory-current: YES" holds — d38's 35
  sheets are in the union (Appendix A), no delta.
- **The lazy alignment** (§6, `tools/_d38_final_lazy.js` RUN): d38
  re-measured the lazy form independently — E35 (HEAD eager minus d38's 35
  NPC rows) reproduces the pre-d38 baseline **8115 exactly** (the 35
  sheets' eager contribution was the +121 that pushed HEAD to 8236); L91
  (d38's 35 + d40's 56 forecast alone) packs **1554 px on its own page**,
  inside §10.4's per-world budgets. **d38's draw path already resolves
  every sheet by name** (§4a, `npc_sheet(texture_net_id(kind))`), so step 7
  needs NO draw-side change on d38 — the rows simply leave
  `STATIC_SHEET_NAMES` and ride the per-world NPC page.
- **The count pins** (§9 E4): the union is 550 kinds / 489 ENTRIES / ≤357
  slots / **446 static NPC sheets** / 103 cues / max proto 761. **The 446
  pin is the EAGER-form count** — under §10.6 step 7 the NPC rows leave it
  (the lazy union's NPC count is the per-world set, not 446). The merger's
  packing audit (§12 step 4) must reconcile against the EAGER 446 if step 7
  is skipped, or the per-world census sets if step 7 lands.
- **The bestiary slot pin** (§4b/§6): 216 as-pushed (d38 closed its +35,
  181→216) + 10 d34 + 16 d35 + 43 d37 + 38 d40 = **323 final**. Slots are a
  LOOKUP table (`BESTIARY_NPC_SHEET_IDS`), NOT atlas input — the lazy form
  does not change the pin. The 328 PumpkingBlade Hide=true exclusion
  (§8 GAP-3) is a bestiary-populator question deferred to the wiring round,
  not an atlas obligation.
- **The two dedupes** (§9 E3): GiantCursedSkull keep d38@289 (drop c3@289),
  PumpkingBlade keep d38@328 (drop c6@328) — the dropped lanes keep their
  AI/draw/sound work; the union dedups by name so the atlas packs one rect
  per id regardless.

**This plan owes d38**: the step-7 contract (§13) keeps d38's by-name draw
path unchanged; the per-world NPC page includes d38's 35 ids (they are
spawnable kinds, in the census); the 323 slot pin rides §12's checklist.

### 16.2 g72e (`task/W4-g72-bg-layers-3`, origin `4a75715ef`, local `6cb4e933f`)

g72's atlas obligations (the underworld escape + the bg-roll order):

- **The underworld screenTarget escape** (§9.1, re-confirmed at the local
  closeout tip): the underworld stack draws through `Main.spriteBatch`
  into **screenTarget** (NOT backgroundTarget — the bg_target census,
  `Main.cs:62098-62100`/`:64049-64053`, `FilterManager.cs:146-152`). The
  port's faithful shape draws the 5 rolled Underworld sheets as plain
  textures into the d69/S1-G-13 `targets.rs` screenTarget family, NOT as
  atlas pages — costing ≤49 MB instead of +1717 atlas px (§7.4). The
  closeout item 1 (`ba6da402e`) wired the `ugBackTransition` cross-fade
  overlays into the live draw chain (the old-style sheet table resolves
  once per frame, `Main.cs:53804-53810`); the escape composes with the
  cross-fade (the overlay draws into the same target family).
- **The surface-bg draw ownership** (research/64c, closeout item 3
  `6cb4e933f`): the five v16-owned variants (corrupt/jungle/hallow/
  crimson/mushroom) READ the g71-shape `surface_bg_texture_ids` expansion
  (`world.rs:1400`, the setBG slots 1/2/4/5/8) — LIVE, not style-0 pins,
  g71-compatible (the merge keeps ONE table). The forest/desert/snow/ocean
  chains are STYLE-0 PINNED on g72 (`[50,51,52]`/`[21,20,-1]`/`[37,38,39]`/
  `Background_28`); their style tables are g70/g71's to port — the pins
  lift to those lanes' reads at the merge. **Atlas consequence**: the
  per-world bg page (§8.4) packs the rolled style set; the surface-bg
  draw-table ownership (g70/g71) determines WHICH ids roll, not whether
  they pack. The underground-desert cavern style stays g72's (no desert
  arm in `PickUndergroundBackgroundStyle`).
- **The :11340 ORDER-CONTRACT** (research/64 §6, origin tip `4a75715ef`):
  g71c's `RandomizeBackgrounds` inserts BETWEEN g72's cave block
  (`pipeline.rs:2746`) and the moon draw (`:2750`) — the C#
  `:11339→:11340→:11341` sequence is contiguous on genRand. This is a
  WORLDGEN RNG-order obligation, not an atlas one, but it determines the
  rolled bg set the per-world page packs: the insert's internal draw order
  (4 forest rolls + corrupt/jungle/snow/hallow/crimson/desert/ocean/
  mushroom/underworld) is g71's `bg_rolls.rs` port. The underworld
  `Next(3)` (`:8011`) feeds `underworldBG` (the v18 seat open, research/64b).
- **The v18 claimant registry** (research/64b, closeout item 2): the
  consolidated 106-byte fixed head (16+31+3+1+5+3+1+2+44) + d109 dense
  LAST; g72's 31-byte cave claim is seated. This is SAVE-format, not
  atlas — recorded for the merger's world-load seam (the rolled styles
  persist in the v18 head, the per-world bg page reads them at load).

**This plan owes g72**: the §9.1 escape is documented as the underworld
path (no atlas pages for the 5 Underworld sheets); the §8.4 per-world bg
page packs the rolled surface/cavern set (the g70/g71 draw-tables determine
the ids); the screenTarget family is the d69 `targets.rs` (already in the
§15.3 VRAM stack, no new allocation).

### 16.3 g31g (`task/W4-g31-tile-draw-pass-4`, tip `9d1e331db`)

g31's atlas obligations (the TILEPASS retire phase, research/136 §7-8):

- **The Glow_362 load obligation** (§7/§9.3): the 717 lava glowmask bind
  needs `Glow_362` (mask id 362, `Main.cs:10056`). The asset census
  answered: **`Glow_362.xnb` EXISTS (288×270, 78.9 kpx padded) but is NOT
  in f15's `GLOW_SHEET_NAMES` set** (the table-pass masks are a different
  census from f15's flame/glow branches). **The load + push + `:971` gate
  coupling + G6 clock keying stay the table-pass lane's obligations.**
  Atlas consequence: `Glow_362` is a NEW union claim the repack must admit
  (it is not in the 562-name union today — f15 never packed it). Measured:
  adding it to the lazy worst union raises the skyline 7305 → **7313**
  (+8 px, fits the 887 px headroom). The merger admits it via the
  table-pass lane's `GLOW_SHEET_NAMES` append (or the union list), and the
  `:971` dark-merge gate (`tile_draw_gate(glow_mask_id: 362)`) lands
  together with the bind (C3, §8.2 — the two binds must land together or
  the dark-cell pixels diverge).
- **The g84 keep-one** (§8/§8.6): g84's grass/vines inline arms DROP at the
  pixel merge (g31's drains own them); **KEEP from g84: the `Extra_260`/
  `Glow_294` asset loads** (the drain plans need them). These are exactly
  the two sheets g84b adds (§14.2, Appendix B-2 #7/#46) — the repack packs
  them in the union, the g31 drains consume them. The two g84 render
  modules DROP wholesale; their 19 witnesses re-pin onto the drain bodies.
- **The C3/C4/C7 fold obligations** (§8.2, all OPEN at retire): C3 (f15's
  fused `flame_tile || glow.is_some()` term → d56's `tile_draw_gate`
  params, the 717 `glow_mask_id: 362` half binds with the table-pass),
  C4 (f15's `neighbor()` rebinds to `FillNeighbor::from_world`), C7 (d58's
  chest rewrite MOVE to CAD step 8). These are DRAW-ORDER obligations, not
  atlas-input ones — they do not change the union list, but the Glow_362
  load (C3's asset half) is the atlas-relevant piece.
- **The 85-witness inventory** (§10.2, retire item 3): the g-family 70 +
  tile_ctor 6 + tile_attach 6 + tile_visibility 3 = 85 witnesses pinned by
  name. The `is_visible` keep-one (executed, `tr_render::tile_visibility`)
  and the ONE-_rand invariant (§10.5) are merge obligations, not atlas.

**This plan owes g31**: admit `Glow_362` to the union (the +8 px measured
fits); keep g84's `Extra_260`/`Glow_294` packed (the drains consume them);
the Glow_362 load couples with the `:971` gate (C3) at the table-pass bind.
The TILEPASS draw-order folds (C3/C4/C7, the keep-ones, the ONE-_rand) are
g31's merger checklist (research/136 §10), not this plan's — recorded here
so the atlas merger and the tile-pass merger coordinate the Glow_362 bind.

### 16.4 The retire cross-read verdict

All three lanes are **inventory-current at their retire tips**: d38's 35
sheets + the 446/323 pins stand (the lazy form re-partitions, no draw
change); g72's underworld escape + surface-bg ownership + bg-roll order
stand (the per-world bg page packs the rolled set, the underworld rides
screenTarget); g31's Glow_362 load obligation is the ONE new atlas claim
(+8 px, admitted) and the g84 Extra_260/Glow_294 keep is already in the
union (§14). **No lane's obligation contradicts the §13 step-7 contract or
the §15 budgets.** The merger executes §3 steps 1-7 + the Glow_362 admission
+ the g31 tile-pass folds (research/136 §10) + the g72 bg-roll wire
(research/64 §6) as coordinated worklists.

## 17. The SEAL-VERIFY pass (the retire receipt + the 707-name union + the FINAL bible)

The seal re-fetch (`git fetch origin --prune` +
`tools/_d35_pushed_extract.py` re-run, 2026-11-21, after the RETIRE tip
`1afd327b9`) re-verifies the retire receipt, admits the NEW claimants
since 562, re-measures the union, and closes the plan with the FINAL
atlas-family bible (§17.5). Tools: `tools/_d35_seal_probe.py` (the named
claimants at their tips), `_d35_seal_delta.py` (the delta since 562),
`_d35_seal_union.py` → `_d35_seal_union.json` (the 707-name table with
dims), `_d35_seal_measure.js` → `_d35_seal_measure.json` +
`_d35_seal_gpu.json` (25 scenarios, verify() + double-run on every
layout, boundary split recursed). The committed retire witnesses
(`_d35_union_list.json` 562, `_d35_retire_measure.json` 24) are NOT
overwritten — the seal witnesses are new files.

### 17.1 The retire receipt: 1afd327b landed, every pin re-read

`git log` on this branch confirms `1afd327b9` is the pushed tip
(origin == local, working tree clean at read). The retire pins all
re-verified against the committed files:

| pin | where | verified |
|---|---|---|
| the step-7 implementation contract 7a-7e | §13.3 (NPC rows leave the eager pack / per-world pack arm / audit re-point / MAIN_PAGE_COUNT uniform / swap seam) | present, intact |
| the 562-name union | `tools/_d35_union_list.json` = 562 rows, §14 | ✓ |
| VRAM min-spec 2 GB | §15.4 (1 GB FAILS the realistic stack) | ✓ |
| font array 268.4 MB | §15.3, `_d35_font_vram.json` (4 BC2 8192² layers) | ✓ |
| lazy ONE page 7140 / worst 7305 | §14.3, `_d35_retire_measure.json` | ✓ |
| Glow_362 +8 px (7305→7313) | §16.3, the `lazy-union-worst+Glow362` scenario | ✓ |

### 17.2 The NEW claimants since 562 (the seal re-extraction)

65 pushed branches / **630 pushed names** (was 54 / 504): +126 names,
10 already forecast/deferred in the 562 union (d125's tip
NPC_636/661/Extra_188 is now PUSHED at `7a0482302` — the local-tip
dagger retires — plus NPC_430/442-447 now pushed by f18/c10), so
**+116 genuinely-new pushed names, 0 gone** (Appendix C). The brief's
named claimants at their CURRENT tips (`_d35_seal_probe.json`):

| claimant | tip | sheet rows | verdict |
|---|---|---|---|
| **d42** (npc-kinds-561-630) | `bac408be0` | **0 rows — NO-PACK confirmed** | research/238 §3: the full runtime set (gore+font) skyline **8382 = +190 OVER** the 8192 bound (scenarios D/E; static-only 8021 pins 171 px headroom); §4 verdict None-skip, the sheets ride this plan's lazy wave. research/239 §4 declares the FINAL **32-kind band** (578-630 minus 8 ceded minus 11 keep-one-dropped); 580/630 are pushed by f42 and 619 by d11, the other **29 names are FORECAST rows** admitted to the census union exactly like d40's 56 (they ride the per-world NPC page under §10.6). |
| **d43** (npc-kinds-631-697) | `0253bde90` | **+63 rows, 53 union-new** | the 631-697 band's sheets phase LANDED (13/13 batches); 10 rows already union-owned (c7/c9/d23/f47/g5 etc.). The big strips: NPC_657 360×1952 (707 kpx), NPC_696 384×1242 (480 kpx), NPC_692 136×992, NPC_631 60×1554. ALL are `NPC_<id>` rows → the lazy filter pulls them onto the per-world NPC page. |
| **f29** (map-overlay-2) | `ff819421d` | **109 rows, ALL in the union — EQUAL both ways** | re-verified by set comparison: the 562-union rows owned e17/f29 number exactly 109 and equal f29's `a725acb0..tip` assets.rs additions (0 missing, 0 extra) — f29's own §11.3 receipt (research/140) reproduces at the seal. The rows are eager `NPC_Head_*`/UI names (60 heads + 40 boss heads + MapDeath + Extra_173/175/176 + Coin_0..3 + UI_quickicon1), NOT `NPC_<id>` kinds — the lazy filter keeps them (§10's regex is `^NPC_\d+$`). |
| **e23** (bestiary-ui-grid) | `7a7779690` | **+9: MapBGOverlay1..9** (49 total, 40 already in) | the weather/event map overlays (115×65, 7.8 kpx each, 70.6 kpx total; sandstorm/rain/sun/moon/eclipse/blizzard/meteor/pumpkin-moon/frost-moon). e23's own measurement (research/237): skyline 7753→**7759** (+6 px) under 8192 on ITS branch. NEW union claims — admitted (Appendix C #88-96); non-NPC rows, they ride the lazy union page. |
| **g84** (plantera-bulb-draw) | `e6c2dcd0d` | **2 rows, tip UNMOVED** | Extra_260/Glow_294 in the union since RETIRE (§14.2); the g31 keep-one (§16.3) stands. |
| **c8** (style66-crawling-critters) | `7c2de9c06` | **9 rows, ALL in union** | NPC_357/374/375/448/484-487/606 — already packed by c8's earlier pushes (the 484-487 crawlers shared with d40's forecast, now also c10/f18). |
| **c9** (style67-snails) | `2042d77f1` | **4 rows, ALL in union** | NPC_359/360/655/677 (655 shared c7, 677 c9-owned since the retire extraction). |

The other arrivals (Appendix C owners): d41's band completion (+19:
NPC_498-506/508-515/519), d36 (+5: 141/148/149/153/154), d39/f18
(NPC_378-380), g8 (+NPC_159), e21 (+NPC_540), f48's dodge shields
(Acc_Shield_1..12, the 44×1120 strips — 12 rows, 578 kpx), d6's
Armor_Head_101/156, e19's cursors (UI/Cursor_3/6/7/8/9/10), g21/g81's
Extra_211 + Lock_0/1 + UI/BlockReplace_0 + UI/InfoIcon_13, g58's
Portrait_Window, f50's BeetleOrb.

### 17.3 The in-union lanes (the brief's question 3)

| lane | tip | in the 562-union? |
|---|---|---|
| **d38** (npc-kinds-281-350) | `9c0e3bc54` (moved from the retire receipt's `1ba0395c7`) | **YES — all 35 rows**, unchanged by the tip move (the probe re-diffed: same 35 names). The §16.1 obligations stand. |
| **d40** (npc-kinds-421-490) | `8fe1d6d31` (UNMOVED, still data-only) | **YES via the 56-id FORECAST** — 0 pushed sheet rows; NPC_430/442-448/454-459/484-487 are now ALSO pushed by f18/c10/d9 (the forecast dagger retires for those, the census keeps one row per name). |
| **g10** (npc-kind-wire-3, the pets) | `5dee28650` | **NO at 562 — 11 NEW rows** (NPC_637/638/656/670/678-684, the town pets incl. TownSlimeCopper 684), every one shared-owner with d43's band. Admitted at the seal (Appendix C); they ride the per-world NPC page (town-resident kinds, in the census superset post-merge). |
| **g11** (dust-main-sites-1) | `2e978b279` | **YES trivially — 0 sheet rows**, pure dust wiring/retire docs (the research/116 s7 merger instruction). NO atlas claim. |

### 17.4 The re-measured union: 707 names (`_d35_seal_measure.json`)

The census union is **707 = 562 + 116 pushed-new + 29 d42-forecast**
(0 dropped; total padded area **17.57 Mpx ≈ 2145 px** of bin width).
The +116 split **80 NPC-band + 36 non-NPC** (the non-NPC 36 add only
0.78 Mpx: 12 shields + 9 MapBGOverlay + 2 armor heads + 6 cursors +
small UI/Extra/Lock rows); the d42 forecast adds 0.61 Mpx, all NPC.

| scenario | sheets | area Mpx | bound | skyline | boundary pages | atlas MB |
|---|---|---|---|---|---|---|
| merged-407 base (re-witnessed) | 9904 | 74.72 | 9125 | 9478 | 8192 / 1298 | 311.0 |
| merged-final-630 | 10170 | 80.03 | 9775 | 10232 | 8190 / 2135 | 338.3 |
| + d42 forecast 29 | 10199 | 80.65 | 9849 | 10325 | 8156 / 2201 | 339.4 |
| **+ d125 deferred 5 (WORST)** | 10204 | 81.82 | 9993 | 10424 | **8191 / 2377** | 346.3 |
| **lazy union (630 − all NPC_)** | 9581 | 55.91 | 6829 | **7256** | **ONE page** | 237.8 |
| **lazy union (worst − all NPC_)** | 9586 | 57.08 | 6972 | **7423** | **ONE page** | 243.2 |
| lazy worst + Glow_362 | 9587 | 57.16 | 6982 | **7447** (+24) | ONE page | 244.0 |

**The contracts hold**: eager worst 2 pages with **5815 px page-1
headroom**; the lazy union stays **ONE page** (7256 px, 936 px headroom;
worst 7423, 769 px; +Glow_362 7447, 745 px — the Glow_362 admission is
+24 px on the SEAL ordering, not the retire's +8: the skyline is
order-chaotic (§1), the honest bound moves 6972→6982 = +10). The
per-world NPC own pages are UNCHANGED at the census sets (fresh 1668 /
long-lived 2152 / bestiary 2566 px); the post-merge augmented roster
(retire-31 + the SEAL census-new kinds = 286 names) packs long-lived
**481 sheets / 2728 px** and bestiary-opened **575 sheets / 3286 px** —
still ONE own page each, ≥4900 px headroom. Realistic totals: **3 pages
536.5-595 MB**; adversarial (worst bg 8135+2138): **4 pages 650.4-687.5
MB**. The GPU-limit check re-runs green: every page ≤268.4 MB (max
realistic 244.1, adversarial 266.6, eager page-0 at 268.4); full VRAM
stack **932-990.5 MB realistic / 1045.9-1083 MB adversarial** (font
268.4 + targets 127.1 unchanged) — **the 2 GB min-spec STANDS** (worst
1083 MB, ~965 MB spare; 1 GB still FAILS).

**Determinism**: 25 scenarios verify() + double-run; a full fresh
cross-run re-run is semantically identical (0 diffs) and byte-identical
(`_d35_seal_measure.json` SHA256
`67C3F25D2496892916FB151E2E914C18294E25FB515F0E34D1F59EA53AAECEAF`);
canonical-form cross-run SHA256
`939BFD003E48797C6229A6B29F21604157AF6DE056432546D9EE6B51033AFA53`.
Witness hashes: `_d35_seal_union.json`
`BB5CBC3412F868CBEB9C9B51F5069BF790EBBE7389D69CAF2F704A0BF2D8FC39`,
`_d35_seal_delta.json`
`7FB7842F604BE1914B8330A36258BA9CA82A74628EE24120A1150078D7E91A90`,
`_d35_pushed_sheets.json` (the 630-name re-extraction)
`F1BA84D8E824089191605685D1A76AAE62D5E41AE4D74062CA1E1EACF79929D1`.

### 17.5 The FINAL atlas-family bible (the 7-step lazy-load merger contract)

This is the atlas merger's LAST WORD — the §12 checklist + §13.3 code
plan re-affirmed with the SEAL acceptance numbers. The merger executes
IN ORDER; each step's witness must pass before the next:

1. **tr-render multi-page** (§3 step 1): `SheetRect.page`,
   `Atlas.pages`, boundary-split `build_ref` (skyline-unbounded then
   split at 8192, tail re-packs recursively), D2Array binding,
   layer-range encoding, bake-at-push inset. Witness: the atlas.rs
   2-page tests + `build_ref_matches_build_bit_for_bit` + the harness
   `boundaryNPages` reproducing the Rust layout.
2. **Lookups per-page rects** (§3 step 2): `page_heights`, per-page
   `uv_rect`/`sheet_frame_uv`, `BgTex.page`; every uv site reads the
   rect's page. Witness: the `content_lookups` family.
3. **The mechanical sweep** (§3 step 3): `atlas_size` → `page_sizes`
   forces every site; the packing audit asserts page membership for
   every union name. Witness: compiler clean + audit.
4. **Admit the deferred/forecast census** (§3 step 4, SEAL form): the
   **707-name union** (`_d35_seal_union.json`) — d35's deferred 12,
   d125's tip 3 + deferred 5, d40's forecast 56, **d42's forecast 29**
   (the NO-PACK verdict, research/238), **Glow_362** (the g31 table-pass
   load, §16.3 — couples with the `:971` gate at the bind). Every name
   packed on SOME page; one rect per name (the dedupes stand: 289 d38,
   328 d38, the Coin/keep-one rows).
5. **Re-measure** (§3 step 5, SEAL budgets): every page ≤8192; the lazy
   union ONE page ≤**7447** (worst+Glow_362; base 7256); the per-world
   NPC/bg pages ≤2. The retire numbers (7140/7305/7313) are superseded
   by the SEAL table (§17.4); the acceptance FORM is unchanged.
6. **Per-world bg pages** (§8.4): the rolled-set loader arm, fresh
   pages after the union, `MAIN_PAGE_COUNT` runtime uniform
   (`atlas.pages.len()` at the `push_font_glyph` chokepoint,
   `sprite.rs:1088` — REQUIRED). Witness: seed-42 7448 px + worst
   8135+2138 reproduce; world-swap bit-stability. The g72 underworld
   escape (§16.2): the 5 Underworld sheets draw as plain textures into
   the d69 screenTarget family, NOT atlas pages.
7. **Per-world NPC pages** (§13.3 7a-7e): the 269 `NPC_<id>` rows leave
   `STATIC_SHEET_NAMES` → `npc_sheet_names_for_world` via
   `texture_net_id` (NOT net_id); the per-world pack arm in
   `build_atlas_inner` after the union; the `npc_tex` audit
   (`entities.rs:444`) re-points to the world set; the bestiary
   superset packs at world load (immutable-in-play, §13.4 — post-merge
   575 sheets / 3286 px / one page); the swap seam re-packs the world
   half only. **No draw-path change** — d38's by-name path (§16.1), the
   `npc_sheets` map, the ~140 named fields, the bestiary `entry_sheet`
   all stand. The `NPC_Head_*`/`NPC_Head_Boss_*` rows are NOT NPC kinds
   — they stay eager (f29's §11.3 receipt).

**Acceptance (FINAL)**: 3 pages realistic (~537-595 MB atlas) / 4
adversarial (~650-688 MB); per-page ≤268.4 MB by construction; full
VRAM 932-990.5 / 1045.9-1083 MB; **min-spec 2 GB VRAM,
max_texture_dimension_2d ≥ 8192, ≥4 array layers** (a recommendation —
wgpu exposes no total-VRAM limit; `log_atlas_diagnostics` should warn
below it). Item/Projectile/Gore stay eager supersets (13.7 Mpx, E-4b).
The G6 cache, the RNG streams, the fixed-step order are untouched
throughout. **Status: SEALED — no further census pass is owed; the next
atlas-family work is the merger itself.**

## Appendix A - the original 407-name union list (priority order: padded area desc)

Legend: branch slugs omit the W4- prefix; dagger = d40 FORECAST (data-only branch, sheets phase pending); double-dagger = d35 DEFERRED (admitted by the repack). Generated by tools/_d35_union_table.py from the REAL .xnb dims. **This is the §1-9 measurement base (26 branches / 339 pushed + 56 forecast + 12 deferred). The FINAL 490-name list lives in `tools/_d35_union_table.md` (regenerated); the 83-name delta is Appendix B below.**

| # | sheet | w×h | padded kpx | owner branch(es) |
|---|-------|-----|-----------|------------------|
| 1 | `Misc/Sunflare/Lens/Flare1` | 1024×1024 | 1052.7 | d70-moonlord-shake-screens |
| 2 | `Misc/Sunflare/Lens/Flare2` | 1024×1024 | 1052.7 | d70-moonlord-shake-screens |
| 3 | `Misc/Sunflare/Lens/PointSharp` | 1024×1024 | 1052.7 | d70-moonlord-shake-screens |
| 4 | `Background_27` | 1024×500 | 515.1 | g71-bg-layers-2 |
| 5 | `UI/WorldCreation/Seed_Everything` | 434×960 | 419.4 | d85-ui-screens-2 |
| 6 | `Misc/Sunflare/Lens/PointBlurry` | 512×512 | 264.2 | d70-moonlord-shake-screens |
| 7 | `Background_26` | 1024×227 | 235.0 | g71-bg-layers-2 |
| 8 | `NPC_86` | 126×1312 | 168.2 | d35-71-140‡ |
| 9 | `NPC_243` | 78×1920 | 153.8 | d37-211-280 |
| 10 | `NPC_109` | 84×1512 | 130.2 | d35-71-140‡ |
| 11 | `UI/Creative/Research_FramedPistons` | 124×1026 | 129.5 | c24-journey-ui |
| 12 | `NPC_477` | 156×720 | 114.1 | d40-421-490† |
| 13 | `NPC_291` | 68×1260 | 88.3 | d38-281-350 |
| 14 | `NPC_483` | 60×1320 | 82.0 | d40-421-490† |
| 15 | `WallOfFlesh` | 190×420 | 81.0 | d127-birds-style24 |
| 16 | `NPC_292` | 60×1260 | 78.2 | d38-281-350 |
| 17 | `NPC_110` | 60×1200 | 74.5 | d35-71-140‡ |
| 18 | `NPC_293` | 60×1200 | 74.5 | d38-281-350 |
| 19 | `NPC_480` | 46×1550 | 74.5 | d40-421-490† |
| 20 | `NPC_509` | 138×522 | 73.4 | c4-style44-flying-fish |
| 21 | `UI/Bestiary/Icon_Tags_Shadow` | 480×150 | 73.3 | e23-bestiary-ui-grid |
| 22 | `NPC_453` | 44×1456 | 67.1 | d40-421-490† |
| 23 | `NPC_229` | 40×1456 | 61.2 | d37-211-280 |
| 24 | `NPC_482` | 48×1120 | 56.1 | d40-421-490† |
| 25 | `NPC_290` | 66×816 | 55.6 | d38-281-350 |
| 26 | `NPC_466` | 64×828 | 54.8 | d40-421-490† |
| 27 | `Glow_127` | 242×216 | 53.2 | f15-glow-bind-tiledraw2 |
| 28 | `NPC_481` | 40×1254 | 52.8 | d40-421-490† |
| 29 | `NPC_473` | 60×840 | 52.2 | d40-421-490† |
| 30 | `NPC_474` | 60×840 | 52.2 | d40-421-490† |
| 31 | `NPC_475` | 60×840 | 52.2 | d40-421-490† |
| 32 | `NPC_476` | 60×840 | 52.2 | d40-421-490† |
| 33 | `NPC_461` | 46×1050 | 50.5 | d40-421-490† |
| 34 | `NPC_37` | 40×1176 | 49.5 | d34-1-70 |
| 35 | `Glow_267` | 36×1296 | 49.3 | f15-glow-bind-tiledraw2 |
| 36 | `NPC_440` | 46×1024 | 49.2 | d40-421-490† |
| 37 | `NPC_275` | 46×992 | 47.7 | d37-211-280 |
| 38 | `Glow_315` | 176×264 | 47.3 | f15-glow-bind-tiledraw2 |
| 39 | `NPC_452` | 40×1120 | 47.1 | d40-421-490† |
| 40 | `NPC_279` | 52×870 | 47.1 | d37-211-280 |
| 41 | `NPC_450` | 42×1064 | 46.9 | d40-421-490† |
| 42 | `NPC_449` | 40×1064 | 44.8 | d40-421-490† |
| 43 | `NPC_451` | 40×1064 | 44.8 | d40-421-490† |
| 44 | `NPC_277` | 50×842 | 43.9 | d37-211-280 |
| 45 | `NPC_276` | 50×840 | 43.8 | d37-211-280 |
| 46 | `NPC_468` | 40×1040 | 43.8 | d40-421-490† |
| 47 | `NPC_430` | 88×476 | 43.0 | d40-421-490† |
| 48 | `NPC_431` | 88×476 | 43.0 | d40-421-490† |
| 49 | `NPC_432` | 88×476 | 43.0 | d40-421-490† |
| 50 | `NPC_433` | 88×476 | 43.0 | d40-421-490† |
| 51 | `NPC_434` | 88×476 | 43.0 | d40-421-490† |
| 52 | `NPC_435` | 88×476 | 43.0 | d40-421-490† |
| 53 | `NPC_436` | 88×476 | 43.0 | d40-421-490† |
| 54 | `NPC_272` | 46×870 | 41.9 | d37-211-280 |
| 55 | `NPC_278` | 46×870 | 41.9 | d37-211-280 |
| 56 | `NPC_460` | 48×832 | 41.7 | d40-421-490† |
| 57 | `NPC_83` | 82×492 | 41.5 | d35-71-140 |
| 58 | `NPC_84` | 82×492 | 41.5 | d35-71-140‡ |
| 59 | `NPC_287` | 44×896 | 41.3 | d38-281-350 |
| 60 | `NPC_269` | 46×842 | 40.5 | d37-211-280 |
| 61 | `NPC_270` | 46×842 | 40.5 | d37-211-280 |
| 62 | `NPC_280` | 44×870 | 40.1 | d37-211-280 |
| 63 | `NPC_469` | 52×720 | 39.0 | d40-421-490† |
| 64 | `NPC_581` | 100×378 | 38.8 | c4-style44-flying-fish |
| 65 | `NPC_104` | 40×896 | 37.7 | d35-71-140‡ |
| 66 | `NPC_294` | 40×896 | 37.7 | d38-281-350 |
| 67 | `NPC_73` | 40×896 | 37.7 | d35-71-140 |
| 68 | `NPC_78` | 40×896 | 37.7 | d35-71-140 |
| 69 | `NPC_79` | 40×896 | 37.7 | d35-71-140‡ |
| 70 | `NPC_80` | 40×896 | 37.7 | d35-71-140‡ |
| 71 | `NPC_323` | 42×840 | 37.0 | d38-281-350 |
| 72 | `NPC_271` | 40×872 | 36.7 | d37-211-280 |
| 73 | `NPC_274` | 40×872 | 36.7 | d37-211-280 |
| 74 | `NPC_140` | 40×840 | 35.4 | d35-71-140‡ |
| 75 | `NPC_273` | 40×840 | 35.4 | d37-211-280 |
| 76 | `NPC_295` | 40×840 | 35.4 | d38-281-350 |
| 77 | `NPC_296` | 40×840 | 35.4 | d38-281-350 |
| 78 | `NPC_31` | 40×840 | 35.4 | d34-1-70 |
| 79 | `NPC_77` | 40×840 | 35.4 | d35-71-140 |
| 80 | `NPC_324` | 36×896 | 34.1 | d38-281-350 |
| 81 | `NPC_116` | 110×300 | 33.8 | d35-71-140 |
| 82 | `NPC_362` | 52×600 | 32.5 | c10-style68-waterfowl |
| 83 | `NPC_363` | 52×600 | 32.5 | c10-style68-waterfowl |
| 84 | `NPC_364` | 52×600 | 32.5 | c10-style68-waterfowl |
| 85 | `NPC_365` | 52×600 | 32.5 | c10-style68-waterfowl |
| 86 | `NPC_608` | 52×600 | 32.5 | c10-style68-waterfowl |
| 87 | `NPC_609` | 52×600 | 32.5 | c10-style68-waterfowl |
| 88 | `NPC_322` | 32×896 | 30.5 | d38-281-350 |
| 89 | `Extra_72` | 800×36 | 30.5 | c39-lantern-birthday |
| 90 | `NPC_120` | 38×720 | 28.9 | d35-71-140‡ |
| 91 | `UI/Creative/Infinite_Powers` | 756×36 | 28.8 | c24-journey-ui |
| 92 | `NPC_467` | 46×598 | 28.8 | d40-421-490† |
| 93 | `NPC_85` | 32×828 | 28.2 | d35-71-140‡ |
| 94 | `NPC_463` | 32×800 | 27.3 | d40-421-490† |
| 95 | `NPC_437` | 66×384 | 26.2 | d40-421-490† |
| 96 | `NPC_454` | 80×312 | 25.7 | d40-421-490† |
| 97 | `NPC_268` | 72×344 | 25.6 | d37-211-280 |
| 98 | `NPC_602` | 44×540 | 24.9 | c10-style68-waterfowl |
| 99 | `NPC_603` | 44×540 | 24.9 | c10-style68-waterfowl |
| 100 | `Glow_243` | 54×432 | 24.3 | f15-glow-bind-tiledraw2 |
| 101 | `NPC_122` | 48×462 | 23.2 | d35-71-140‡ |
| 102 | `Glow_289` | 36×576 | 22.0 | f15-glow-bind-tiledraw2 |
| 103 | `NPC_289` | 52×384 | 20.8 | d38-281-350 |
| 104 | `NPC_242` | 50×392 | 20.5 | d37-211-280 |
| 105 | `NPC_240` | 68×272 | 19.2 | d37-211-280 |
| 106 | `NPC_103` | 56×322 | 18.8 | d35-71-140‡ |
| 107 | `NPC_488` | 32×550 | 18.8 | d40-421-490† |
| 108 | `NPC_94` | 64×282 | 18.7 | d35-71-140 |
| 109 | `Glow_330` | 36×486 | 18.5 | f15-glow-bind-tiledraw2 |
| 110 | `Extra_71` | 102×168 | 17.7 | c39-lantern-birthday |
| 111 | `UI/Creative/Infinite_Tabs_B` | 84×192 | 16.7 | c24-journey-ui |
| 112 | `NPC_226` | 64×248 | 16.5 | d37-211-280 |
| 113 | `Glow_268` | 36×432 | 16.5 | f15-glow-bind-tiledraw2 |
| 114 | `Glow_269` | 36×432 | 16.5 | f15-glow-bind-tiledraw2 |
| 115 | `Glow_270` | 36×432 | 16.5 | f15-glow-bind-tiledraw2 |
| 116 | `NPC_244` | 74×208 | 16.0 | d37-211-280 |
| 117 | `NPC_121` | 90×168 | 15.6 | d35-71-140 |
| 118 | `NPC_237` | 60×248 | 15.5 | d37-211-280 |
| 119 | `NPC_238` | 60×248 | 15.5 | d37-211-280 |
| 120 | `NPC_470` | 28×504 | 15.2 | d40-421-490† |
| 121 | `NPC_221` | 40×352 | 14.9 | d37-211-280 |
| 122 | `NPC_303` | 48×280 | 14.1 | d38-281-350 |
| 123 | `NPC_337` | 48×280 | 14.1 | d38-281-350 |
| 124 | `NPC_443` | 48×280 | 14.1 | d40-421-490† |
| 125 | `NPC_46` | 48×280 | 14.1 | d34-1-70 |
| 126 | `NPC_472` | 52×252 | 13.7 | d40-421-490† |
| 127 | `NPC_101` | 62×210 | 13.6 | d35-71-140 |
| 128 | `NPC_479` | 62×204 | 13.2 | d40-421-490† |
| 129 | `NPC_239` | 72×170 | 12.7 | d37-211-280 |
| 130 | `NPC_301` | 44×270 | 12.5 | d38-281-350 |
| 131 | `NPC_462` | 34×342 | 12.4 | d40-421-490† |
| 132 | `NPC_236` | 84×140 | 12.2 | d37-211-280 |
| 133 | `Glow_202` | 54×216 | 12.2 | f15-glow-bind-tiledraw2 |
| 134 | `Extra_259` | 114×102 | 12.1 | g55-ui-loadout-save |
| 135 | `NPC_257` | 54×210 | 11.9 | d37-211-280 |
| 136 | `NPC_258` | 46×228 | 11.0 | d37-211-280 |
| 137 | `UI/Creative/Research_Slots` | 104×96 | 10.4 | c24-journey-ui |
| 138 | `NPC_252` | 44×220 | 10.2 | d37-211-280 |
| 139 | `NPC_464` | 34×280 | 10.2 | d40-421-490† |
| 140 | `NPC_299` | 50×192 | 10.1 | d38-281-350 |
| 141 | `NPC_677` | 88×110 | 10.1 | c7-style64-fireflies, c9-style67-snails |
| 142 | `Glow_112` | 486×18 | 9.8 | f15-glow-bind-tiledraw2 |
| 143 | `Glow_65` | 486×18 | 9.8 | f15-glow-bind-tiledraw2 |
| 144 | `NPC_328` | 106×86 | 9.5 | c6-style59-pumpking-blade, d38-281-350 |
| 145 | `NPC_316` | 40×224 | 9.5 | d38-281-350 |
| 146 | `NPC_438` | 40×216 | 9.2 | d40-421-490† |
| 147 | `NPC_455` | 80×104 | 8.7 | d40-421-490† |
| 148 | `NPC_456` | 80×104 | 8.7 | d40-421-490† |
| 149 | `NPC_457` | 80×104 | 8.7 | d40-421-490† |
| 150 | `NPC_458` | 80×104 | 8.7 | d40-421-490† |
| 151 | `NPC_459` | 80×104 | 8.7 | d40-421-490† |
| 152 | `NPC_102` | 48×168 | 8.5 | d35-71-140 |
| 153 | `NPC_253` | 34×234 | 8.5 | d37-211-280 |
| 154 | `NPC_445` | 26×286 | 8.1 | d40-421-490† |
| 155 | `Extra_70` | 54×138 | 7.8 | c39-lantern-birthday |
| 156 | `NPC_284` | 44×168 | 7.8 | d38-281-350 |
| 157 | `NPC_75` | 54×136 | 7.7 | d35-71-140 |
| 158 | `NPC_285` | 42×168 | 7.5 | d38-281-350 |
| 159 | `NPC_286` | 40×170 | 7.2 | d38-281-350 |
| 160 | `NPC_281` | 40×168 | 7.1 | d38-281-350 |
| 161 | `NPC_282` | 40×168 | 7.1 | d38-281-350 |
| 162 | `NPC_283` | 40×168 | 7.1 | d38-281-350 |
| 163 | `NPC_241` | 36×180 | 6.9 | d37-211-280 |
| 164 | `NPC_465` | 38×168 | 6.8 | d40-421-490† |
| 165 | `NPC_233` | 44×144 | 6.7 | d37-211-280 |
| 166 | `NPC_360` | 42×144 | 6.4 | c9-style67-snails |
| 167 | `NPC_655` | 42×144 | 6.4 | c9-style67-snails |
| 168 | `NPC_255` | 38×156 | 6.3 | d37-211-280 |
| 169 | `NPC_256` | 34×172 | 6.3 | d37-211-280 |
| 170 | `NPC_254` | 42×138 | 6.2 | d37-211-280 |
| 171 | `NPC_234` | 46×126 | 6.1 | d37-211-280 |
| 172 | `NPC_235` | 48×120 | 6.1 | d37-211-280 |
| 173 | `Glow_55` | 106×54 | 6.0 | f15-glow-bind-tiledraw2 |
| 174 | `NPC_137` | 44×128 | 6.0 | d35-71-140 |
| 175 | `NPC_93` | 44×128 | 6.0 | d35-71-140 |
| 176 | `NPC_43` | 52×108 | 5.9 | d34-1-70 |
| 177 | `NPC_231` | 46×120 | 5.9 | d37-211-280 |
| 178 | `Glow_114` | 144×38 | 5.8 | f15-glow-bind-tiledraw2 |
| 179 | `Glow_53` | 144×38 | 5.8 | f15-glow-bind-tiledraw2 |
| 180 | `NPC_82` | 26×200 | 5.7 | d35-71-140 |
| 181 | `Glow_113` | 142×36 | 5.5 | f15-glow-bind-tiledraw2 |
| 182 | `Glow_52` | 142×36 | 5.5 | f15-glow-bind-tiledraw2 |
| 183 | `NPC_224` | 40×128 | 5.5 | c4-style44-flying-fish, d37-211-280 |
| 184 | `NPC_587` | 40×128 | 5.5 | c4-style44-flying-fish |
| 185 | `NPC_232` | 44×114 | 5.3 | d37-211-280 |
| 186 | `NPC_251` | 34×144 | 5.3 | d37-211-280 |
| 187 | `NPC_230` | 25×180 | 4.9 | d37-211-280 |
| 188 | `NPC_260` | 46×92 | 4.5 | d37-211-280 |
| 189 | `Glow_117` | 36×114 | 4.4 | f15-glow-bind-tiledraw2 |
| 190 | `UI/TexturePackButtons` | 64×64 | 4.4 | d84-ui-screens-1 |
| 191 | `NPC_304` | 48×80 | 4.1 | d38-281-350 |
| 192 | `Extra_69` | 42×90 | 4.0 | c39-lantern-birthday |
| 193 | `Glow_115` | 54×70 | 4.0 | f15-glow-bind-tiledraw2 |
| 194 | `Glow_119` | 70×54 | 4.0 | f15-glow-bind-tiledraw2 |
| 195 | `Glow_58` | 70×54 | 4.0 | f15-glow-bind-tiledraw2 |
| 196 | `Glow_60` | 54×70 | 4.0 | f15-glow-bind-tiledraw2 |
| 197 | `NPC_297` | 30×120 | 3.9 | d38-281-350 |
| 198 | `NPC_298` | 30×120 | 3.9 | d38-281-350 |
| 199 | `NPC_359` | 30×120 | 3.9 | c9-style67-snails |
| 200 | `NPC_442` | 30×120 | 3.9 | d40-421-490† |
| 201 | `NPC_74` | 30×120 | 3.9 | d35-71-140 |
| 202 | `NPC_259` | 28×128 | 3.9 | d37-211-280 |
| 203 | `Glow_56` | 32×112 | 3.9 | f15-glow-bind-tiledraw2 |
| 204 | `UI/DefaultResourcePackIcon` | 60×60 | 3.8 | d84-ui-screens-1 |
| 205 | `UI/WorldCreation/Seed_Celebration` | 60×60 | 3.8 | d85-ui-screens-2 |
| 206 | `UI/WorldCreation/Seed_Drunk` | 60×60 | 3.8 | d85-ui-screens-2 |
| 207 | `UI/WorldCreation/Seed_ForTheWorthy` | 60×60 | 3.8 | d85-ui-screens-2 |
| 208 | `UI/WorldCreation/Seed_NoTraps` | 60×60 | 3.8 | d85-ui-screens-2 |
| 209 | `UI/WorldCreation/Seed_Normal` | 60×60 | 3.8 | d85-ui-screens-2 |
| 210 | `UI/WorldCreation/Seed_NotTheBees` | 60×60 | 3.8 | d85-ui-screens-2 |
| 211 | `UI/WorldCreation/Seed_Remix` | 60×60 | 3.8 | d85-ui-screens-2 |
| 212 | `UI/WorldCreation/Seed_Secret` | 60×60 | 3.8 | d85-ui-screens-2 |
| 213 | `UI/WorldCreation/Seed_Skyblock` | 60×60 | 3.8 | d85-ui-screens-2 |
| 214 | `UI/WorldCreation/Seed_TheConstant` | 60×60 | 3.8 | d85-ui-screens-2 |
| 215 | `UI/Workshop/HubPublishResourcepacks` | 60×58 | 3.7 | d84-ui-screens-1 |
| 216 | `UI/Workshop/HubPublishWorlds` | 60×58 | 3.7 | d84-ui-screens-1 |
| 217 | `UI/Workshop/HubResourcepacks` | 60×58 | 3.7 | d84-ui-screens-1 |
| 218 | `UI/Workshop/HubWorlds` | 60×58 | 3.7 | d84-ui-screens-1 |
| 219 | `UI/Bestiary/Button_Filtering` | 114×28 | 3.5 | e23-bestiary-ui-grid |
| 220 | `UI/Bestiary/Button_Sorting` | 114×28 | 3.5 | e23-bestiary-ui-grid |
| 221 | `UI/Bestiary/Button_Wide_Border` | 114×28 | 3.5 | e23-bestiary-ui-grid |
| 222 | `Glow_118` | 36×88 | 3.4 | f15-glow-bind-tiledraw2 |
| 223 | `Glow_51` | 36×88 | 3.4 | f15-glow-bind-tiledraw2 |
| 224 | `NPC_317` | 48×64 | 3.3 | d38-281-350 |
| 225 | `NPC_333` | 38×76 | 3.1 | d38-281-350 |
| 226 | `NPC_334` | 38×76 | 3.1 | d38-281-350 |
| 227 | `NPC_335` | 38×76 | 3.1 | d38-281-350 |
| 228 | `NPC_336` | 38×76 | 3.1 | d38-281-350 |
| 229 | `Glow_57` | 52×54 | 3.0 | f15-glow-bind-tiledraw2 |
| 230 | `Inventory_Back24` | 52×52 | 2.9 | f23-ui-elements-4 |
| 231 | `NPC_220` | 46×56 | 2.8 | d37-211-280 |
| 232 | `NPC_318` | 40×64 | 2.8 | d38-281-350 |
| 233 | `NPC_34` | 28×90 | 2.8 | d34-1-70 |
| 234 | `PumpkingArm` | 26×96 | 2.7 | c6-style59-pumpking-blade |
| 235 | `NPC_Head_Boss_39` | 50×48 | 2.6 | e17-map-overlay-layers, f29-map-overlay-2 |
| 236 | `Glow_68` | 70×34 | 2.6 | f15-glow-bind-tiledraw2 |
| 237 | `UI/NPCHappiness` | 96×24 | 2.5 | g58-ui-shop-happiness |
| 238 | `NPC_302` | 40×56 | 2.4 | d38-281-350 |
| 239 | `Coin_3` | 14×144 | 2.3 | f29-map-overlay-2 |
| 240 | `NPC_300` | 20×96 | 2.2 | d38-281-350 |
| 241 | `NPC_447` | 20×96 | 2.2 | d40-421-490† |
| 242 | `Glow_124` | 54×36 | 2.1 | f15-glow-bind-tiledraw2 |
| 243 | `Glow_62` | 36×54 | 2.1 | f15-glow-bind-tiledraw2 |
| 244 | `Glow_67` | 54×36 | 2.1 | f15-glow-bind-tiledraw2 |
| 245 | `Coin_2` | 12×144 | 2.0 | f29-map-overlay-2 |
| 246 | `Glow_120` | 54×34 | 2.0 | f15-glow-bind-tiledraw2 |
| 247 | `Glow_121` | 54×34 | 2.0 | f15-glow-bind-tiledraw2 |
| 248 | `Glow_123` | 54×34 | 2.0 | f15-glow-bind-tiledraw2 |
| 249 | `Glow_59` | 54×34 | 2.0 | f15-glow-bind-tiledraw2 |
| 250 | `Glow_64` | 54×34 | 2.0 | f15-glow-bind-tiledraw2 |
| 251 | `Glow_66` | 54×34 | 2.0 | f15-glow-bind-tiledraw2 |
| 252 | `NPC_444` | 24×74 | 2.0 | d40-421-490† |
| 253 | `NPC_219` | 32×56 | 2.0 | d37-211-280 |
| 254 | `Coin_1` | 12×128 | 1.8 | f29-map-overlay-2 |
| 255 | `NPC_100` | 34×48 | 1.8 | d35-71-140 |
| 256 | `NPC_217` | 34×48 | 1.8 | d37-211-280 |
| 257 | `NPC_98` | 34×48 | 1.8 | d35-71-140 |
| 258 | `NPC_99` | 34×48 | 1.8 | d35-71-140 |
| 259 | `NPC_478` | 38×42 | 1.8 | d40-421-490† |
| 260 | `NPC_Head_Boss_34` | 42×36 | 1.7 | e17-map-overlay-layers, f29-map-overlay-2 |
| 261 | `NPC_218` | 34×44 | 1.7 | d37-211-280 |
| 262 | `NPC_95` | 30×48 | 1.6 | d35-71-140 |
| 263 | `NPC_96` | 30×48 | 1.6 | d35-71-140 |
| 264 | `NPC_97` | 30×48 | 1.6 | d35-71-140 |
| 265 | `Coin_0` | 12×112 | 1.6 | f29-map-overlay-2 |
| 266 | `Glow_116` | 36×40 | 1.6 | f15-glow-bind-tiledraw2 |
| 267 | `Glow_54` | 36×40 | 1.6 | f15-glow-bind-tiledraw2 |
| 268 | `Glow_122` | 36×38 | 1.5 | f15-glow-bind-tiledraw2 |
| 269 | `Glow_88` | 36×38 | 1.5 | f15-glow-bind-tiledraw2 |
| 270 | `NPC_485` | 18×72 | 1.5 | d40-421-490† |
| 271 | `NPC_486` | 18×72 | 1.5 | d40-421-490† |
| 272 | `NPC_487` | 18×72 | 1.5 | d40-421-490† |
| 273 | `Glow_63` | 34×36 | 1.4 | f15-glow-bind-tiledraw2 |
| 274 | `NPC_Head_Boss_8` | 34×36 | 1.4 | e17-map-overlay-layers, f29-map-overlay-2 |
| 275 | `UI/Creative/Research_Spark` | 33×37 | 1.4 | c24-journey-ui |
| 276 | `Extra_134` | 60×20 | 1.4 | c39-lantern-birthday, f40-lantern-night-2 |
| 277 | `NPC_Head_Boss_35` | 40×30 | 1.3 | e17-map-overlay-layers, f29-map-overlay-2 |
| 278 | `NPC_Head_71` | 34×34 | 1.3 | e17-map-overlay-layers |
| 279 | `UI/Creative/Journey_Toggle` | 34×32 | 1.2 | c24-journey-ui |
| 280 | `UI/Creative/Journey_Toggle_MouseOver` | 34×32 | 1.2 | c24-journey-ui |
| 281 | `NPC_Head_67` | 26×40 | 1.2 | e17-map-overlay-layers |
| 282 | `NPC_Head_Boss_32` | 32×32 | 1.2 | e17-map-overlay-layers, f29-map-overlay-2 |
| 283 | `UI/Creative/Research_GearA` | 32×32 | 1.2 | c24-journey-ui |
| 284 | `NPC_Head_76` | 34×30 | 1.2 | e17-map-overlay-layers |
| 285 | `NPC_Head_54` | 28×36 | 1.1 | e17-map-overlay-layers |
| 286 | `UI/Bestiary/Stat_Panel` | 31×31 | 1.1 | e23-bestiary-ui-grid |
| 287 | `NPC_Head_12` | 32×30 | 1.1 | e17-map-overlay-layers, f29-map-overlay-2 |
| 288 | `NPC_Head_63` | 32×30 | 1.1 | e17-map-overlay-layers |
| 289 | `NPC_Head_69` | 32×30 | 1.1 | e17-map-overlay-layers |
| 290 | `NPC_Head_Boss_37` | 30×32 | 1.1 | e17-map-overlay-layers, f29-map-overlay-2 |
| 291 | `Glow_329` | 26×36 | 1.1 | f15-glow-bind-tiledraw2 |
| 292 | `NPC_358` | 14×64 | 1.1 | c7-style64-fireflies |
| 293 | `NPC_654` | 14×64 | 1.1 | c7-style64-fireflies |
| 294 | `NPC_Head_Boss_10` | 30×30 | 1.0 | e17-map-overlay-layers, f29-map-overlay-2 |
| 295 | `NPC_Head_Boss_11` | 30×30 | 1.0 | e17-map-overlay-layers, f29-map-overlay-2 |
| 296 | `NPC_Head_Boss_12` | 30×30 | 1.0 | e17-map-overlay-layers, f29-map-overlay-2 |
| 297 | `NPC_Head_Boss_13` | 30×30 | 1.0 | e17-map-overlay-layers, f29-map-overlay-2 |
| 298 | `NPC_Head_Boss_17` | 30×30 | 1.0 | e17-map-overlay-layers, f29-map-overlay-2 |
| 299 | `NPC_Head_Boss_22` | 30×30 | 1.0 | e17-map-overlay-layers, f29-map-overlay-2 |
| 300 | `NPC_Head_Boss_23` | 30×30 | 1.0 | e17-map-overlay-layers, f29-map-overlay-2 |
| 301 | `NPC_Head_Boss_24` | 30×30 | 1.0 | e17-map-overlay-layers, f29-map-overlay-2 |
| 302 | `NPC_Head_Boss_3` | 30×30 | 1.0 | e17-map-overlay-layers, f29-map-overlay-2 |
| 303 | `NPC_Head_Boss_31` | 30×30 | 1.0 | e17-map-overlay-layers |
| 304 | `NPC_Head_Boss_4` | 30×30 | 1.0 | e17-map-overlay-layers, f29-map-overlay-2 |
| 305 | `NPC_Head_Boss_6` | 30×30 | 1.0 | e17-map-overlay-layers, f29-map-overlay-2 |
| 306 | `NPC_Head_Boss_7` | 30×30 | 1.0 | e17-map-overlay-layers |
| 307 | `UI/Reforge_0` | 30×30 | 1.0 | g57-ui-reforge-anim |
| 308 | `UI/Reforge_1` | 30×30 | 1.0 | g57-ui-reforge-anim |
| 309 | `NPC_Head_56` | 32×28 | 1.0 | e17-map-overlay-layers |
| 310 | `NPC_Head_77` | 32×28 | 1.0 | e17-map-overlay-layers |
| 311 | `NPC_Head_78` | 32×28 | 1.0 | e17-map-overlay-layers |
| 312 | `NPC_Head_19` | 26×34 | 1.0 | e17-map-overlay-layers, f29-map-overlay-2 |
| 313 | `NPC_Head_66` | 26×34 | 1.0 | e17-map-overlay-layers |
| 314 | `Extra_173` | 28×30 | 1.0 | e17-map-overlay-layers, f29-map-overlay-2 |
| 315 | `Extra_175` | 28×30 | 1.0 | e17-map-overlay-layers, f29-map-overlay-2 |
| 316 | `NPC_Head_79` | 28×30 | 1.0 | e17-map-overlay-layers |
| 317 | `NPC_Head_Boss_14` | 28×30 | 1.0 | e17-map-overlay-layers, f29-map-overlay-2 |
| 318 | `NPC_Head_Boss_15` | 30×28 | 1.0 | e17-map-overlay-layers, f29-map-overlay-2 |
| 319 | `NPC_Head_Boss_16` | 30×28 | 1.0 | e17-map-overlay-layers, f29-map-overlay-2 |
| 320 | `NPC_Head_Boss_20` | 30×28 | 1.0 | e17-map-overlay-layers, f29-map-overlay-2 |
| 321 | `NPC_Head_Boss_21` | 30×28 | 1.0 | e17-map-overlay-layers, f29-map-overlay-2 |
| 322 | `NPC_Head_Boss_26` | 30×28 | 1.0 | e17-map-overlay-layers, f29-map-overlay-2 |
| 323 | `NPC_Head_Boss_38` | 28×30 | 1.0 | e17-map-overlay-layers, f29-map-overlay-2 |
| 324 | `NPC_Head_Boss_5` | 28×30 | 1.0 | e17-map-overlay-layers, f29-map-overlay-2 |
| 325 | `NPC_Head_52` | 32×26 | 1.0 | e17-map-overlay-layers, f29-map-overlay-2 |
| 326 | `NPC_Head_80` | 32×26 | 1.0 | e17-map-overlay-layers |
| 327 | `Glow_61` | 36×22 | 0.9 | f15-glow-bind-tiledraw2 |
| 328 | `NPC_Head_26` | 28×28 | 0.9 | e17-map-overlay-layers, f29-map-overlay-2 |
| 329 | `UI/Bestiary/Button_Border` | 28×28 | 0.9 | e23-bestiary-ui-grid |
| 330 | `UI/Bestiary/Button_Forward` | 28×28 | 0.9 | e23-bestiary-ui-grid |
| 331 | `UI/Creative/Research_GearB` | 28×28 | 0.9 | c24-journey-ui |
| 332 | `NPC_Head_47` | 30×26 | 0.9 | e17-map-overlay-layers, f29-map-overlay-2 |
| 333 | `NPC_Head_49` | 30×26 | 0.9 | e17-map-overlay-layers, f29-map-overlay-2 |
| 334 | `NPC_Head_64` | 26×30 | 0.9 | e17-map-overlay-layers |
| 335 | `NPC_Head_70` | 26×30 | 0.9 | e17-map-overlay-layers |
| 336 | `NPC_Head_Boss_18` | 26×30 | 0.9 | e17-map-overlay-layers, f29-map-overlay-2 |
| 337 | `NPC_Head_Boss_19` | 26×30 | 0.9 | e17-map-overlay-layers, f29-map-overlay-2 |
| 338 | `NPC_Head_Boss_27` | 26×30 | 0.9 | e17-map-overlay-layers, f29-map-overlay-2 |
| 339 | `NPC_Head_Boss_28` | 26×30 | 0.9 | e17-map-overlay-layers, f29-map-overlay-2 |
| 340 | `NPC_Head_Boss_29` | 26×30 | 0.9 | e17-map-overlay-layers, f29-map-overlay-2 |
| 341 | `NPC_Head_Boss_30` | 26×30 | 0.9 | e17-map-overlay-layers, f29-map-overlay-2 |
| 342 | `NPC_Head_Boss_33` | 26×30 | 0.9 | e17-map-overlay-layers, f29-map-overlay-2 |
| 343 | `UI/Workshop/Publish` | 30×26 | 0.9 | d84-ui-screens-1 |
| 344 | `NPC_Head_18` | 32×24 | 0.9 | e17-map-overlay-layers, f29-map-overlay-2 |
| 345 | `NPC_Head_73` | 32×24 | 0.9 | e17-map-overlay-layers |
| 346 | `NPC_Head_59` | 22×34 | 0.9 | e17-map-overlay-layers |
| 347 | `NPC_Head_48` | 26×28 | 0.8 | e17-map-overlay-layers, f29-map-overlay-2 |
| 348 | `NPC_Head_57` | 28×26 | 0.8 | e17-map-overlay-layers |
| 349 | `NPC_Head_62` | 28×26 | 0.8 | e17-map-overlay-layers |
| 350 | `NPC_Head_72` | 28×26 | 0.8 | e17-map-overlay-layers |
| 351 | `NPC_Head_75` | 28×26 | 0.8 | e17-map-overlay-layers |
| 352 | `NPC_Head_Boss_0` | 26×28 | 0.8 | e17-map-overlay-layers, f29-map-overlay-2 |
| 353 | `NPC_Head_Boss_1` | 26×28 | 0.8 | e17-map-overlay-layers, f29-map-overlay-2 |
| 354 | `Glow_125` | 36×20 | 0.8 | f15-glow-bind-tiledraw2 |
| 355 | `Glow_69` | 36×20 | 0.8 | f15-glow-bind-tiledraw2 |
| 356 | `NPC_Head_Boss_2` | 24×30 | 0.8 | e17-map-overlay-layers, f29-map-overlay-2 |
| 357 | `NPC_Head_Boss_25` | 24×30 | 0.8 | e17-map-overlay-layers, f29-map-overlay-2 |
| 358 | `NPC_Head_Boss_9` | 30×24 | 0.8 | e17-map-overlay-layers, f29-map-overlay-2 |
| 359 | `NPC_355` | 10×64 | 0.8 | c7-style64-fireflies |
| 360 | `NPC_Head_68` | 26×26 | 0.8 | e17-map-overlay-layers |
| 361 | `NPC_Head_53` | 28×24 | 0.8 | e17-map-overlay-layers, f29-map-overlay-2 |
| 362 | `MapDeath` | 24×26 | 0.7 | e17-map-overlay-layers, f29-map-overlay-2 |
| 363 | `NPC_Head_51` | 24×26 | 0.7 | e17-map-overlay-layers, f29-map-overlay-2 |
| 364 | `NPC_Head_50` | 28×22 | 0.7 | e17-map-overlay-layers, f29-map-overlay-2 |
| 365 | `NPC_Head_60` | 28×22 | 0.7 | e17-map-overlay-layers |
| 366 | `NPC_Head_74` | 28×22 | 0.7 | e17-map-overlay-layers |
| 367 | `Extra_243` | 24×24 | 0.7 | d84-ui-screens-1 |
| 368 | `NPC_Head_13` | 22×24 | 0.6 | e17-map-overlay-layers, f29-map-overlay-2 |
| 369 | `NPC_Head_16` | 22×24 | 0.6 | e17-map-overlay-layers, f29-map-overlay-2 |
| 370 | `NPC_Head_46` | 26×20 | 0.6 | e17-map-overlay-layers, f29-map-overlay-2 |
| 371 | `NPC_Head_36` | 22×22 | 0.6 | e17-map-overlay-layers |
| 372 | `NPC_Head_38` | 22×22 | 0.6 | e17-map-overlay-layers |
| 373 | `UI/Bestiary/Button_Search` | 22×22 | 0.6 | c24-journey-ui, e23-bestiary-ui-grid |
| 374 | `NPC_Head_58` | 20×24 | 0.6 | e17-map-overlay-layers |
| 375 | `Chain12` | 16×28 | 0.5 | d127-birds-style24 |
| 376 | `Chain4` | 16×28 | 0.5 | d127-birds-style24 |
| 377 | `NPC_Head_34` | 22×20 | 0.5 | e17-map-overlay-layers |
| 378 | `NPC_Head_55` | 22×20 | 0.5 | e17-map-overlay-layers |
| 379 | `NPC_Head_65` | 20×22 | 0.5 | e17-map-overlay-layers |
| 380 | `NPC_446` | 18×24 | 0.5 | d40-421-490† |
| 381 | `NPC_Head_33` | 22×18 | 0.5 | e17-map-overlay-layers, f29-map-overlay-2 |
| 382 | `NPC_Head_35` | 22×18 | 0.5 | e17-map-overlay-layers |
| 383 | `NPC_Head_37` | 22×18 | 0.5 | e17-map-overlay-layers |
| 384 | `NPC_Head_Boss_36` | 22×18 | 0.5 | e17-map-overlay-layers, f29-map-overlay-2 |
| 385 | `NPC_Head_0` | 16×24 | 0.5 | e17-map-overlay-layers |
| 386 | `NPC_Head_40` | 16×24 | 0.5 | e17-map-overlay-layers |
| 387 | `NPC_484` | 18×20 | 0.4 | d40-421-490† |
| 388 | `NPC_Head_31` | 18×20 | 0.4 | e17-map-overlay-layers |
| 389 | `NPC_Head_61` | 18×20 | 0.4 | e17-map-overlay-layers |
| 390 | `UI/Cursor_11` | 18×18 | 0.4 | e18-minimap-reveal-queue |
| 391 | `NPC_Head_27` | 16×18 | 0.4 | e17-map-overlay-layers, f29-map-overlay-2 |
| 392 | `NPC_Head_28` | 16×18 | 0.4 | e17-map-overlay-layers |
| 393 | `NPC_Head_29` | 16×18 | 0.4 | e17-map-overlay-layers |
| 394 | `NPC_Head_30` | 16×18 | 0.4 | e17-map-overlay-layers |
| 395 | `NPC_Head_32` | 16×18 | 0.4 | e17-map-overlay-layers |
| 396 | `NPC_Head_39` | 14×20 | 0.4 | e17-map-overlay-layers, f29-map-overlay-2 |
| 397 | `NPC_Head_41` | 14×20 | 0.4 | e17-map-overlay-layers |
| 398 | `NPC_Head_42` | 14×20 | 0.4 | e17-map-overlay-layers |
| 399 | `NPC_Head_44` | 14×20 | 0.4 | e17-map-overlay-layers |
| 400 | `Extra_176` | 16×16 | 0.3 | e17-map-overlay-layers, f29-map-overlay-2 |
| 401 | `NPC_112` | 16×16 | 0.3 | d35-71-140 |
| 402 | `UI/Creative/Research_GearC` | 16×16 | 0.3 | c24-journey-ui |
| 403 | `NPC_448` | 14×16 | 0.3 | d40-421-490† |
| 404 | `NPC_Head_43` | 16×14 | 0.3 | e17-map-overlay-layers |
| 405 | `UI/Bestiary/Icon_Rank_Dim` | 14×14 | 0.3 | e23-bestiary-ui-grid |
| 406 | `UI/Bestiary/Icon_Rank_Light` | 14×14 | 0.3 | e23-bestiary-ui-grid |
| 407 | `UI/UI_quickicon1` | 6×16 | 0.1 | e17-map-overlay-layers, f29-map-overlay-2 |

## Appendix B - the FINAL delta (83 new names, priority order: padded area desc)

The §11 re-extraction delta: 37 pushed branches / 424 names (+85 vs the
339 base, 0 gone) + the d125 local-tip delta (+3 tip, +5 deferred),
deduped against Appendix A. Generated by `tools/_d35_delta_table.py`
from `tools/_d35_union_list.json` (the FULL 490-name table is
`tools/_d35_union_table.md`). Legend: † d40 forecast; ‡ d35/d125
deferred; "(tip 2dfe026d, local)" = d125's unpushed sheets tip.

| # | sheet | w×h | padded kpx | owner branch(es) |
|---|-------|-----|-----------|------------------|
| 1 | `Misc/Sunflare/flare2` | 1166×254 | 299.0 | g70-bg-layers-1 |
| 2 | `Extra_187` | 178×1488 | 268.2 | d125-empress-r7-drain‡ |
| 3 | `Extra_157` | 195×1221 | 240.9 | d125-empress-r7-drain‡ |
| 4 | `Extra_159` | 195×1221 | 240.9 | d125-empress-r7-drain‡ |
| 5 | `Extra_158` | 174×1204 | 212.3 | d125-empress-r7-drain‡ |
| 6 | `Extra_160` | 174×1204 | 212.3 | d125-empress-r7-drain‡ |
| 7 | `Misc/Sunflare/flare1` | 532×215 | 115.9 | g70-bg-layers-1 |
| 8 | `NPC_636` | 166×352 | 59.5 | d125-empress-r7-drain (tip 2dfe026d, local) |
| 9 | `NPC_694` | 46×1200 | 57.7 | c3-style10-cursed-skull |
| 10 | `NPC_156` | 104×500 | 53.2 | g8-npc-kind-wire-1 |
| 11 | `NPC_48` | 100×516 | 52.8 | d34-1-70, g8-npc-kind-wire-1 |
| 12 | `NPC_196` | 38×784 | 31.4 | g8-npc-kind-wire-1 |
| 13 | `Extra_188` | 166×174 | 29.6 | d125-empress-r7-drain (tip 2dfe026d, local) |
| 14 | `NPC_61` | 78×360 | 29.0 | d34-1-70, g8-npc-kind-wire-1 |
| 15 | `NPC_62` | 86×310 | 27.5 | d34-1-70, g8-npc-kind-wire-1 |
| 16 | `NPC_179` | 68×330 | 23.2 | g8-npc-kind-wire-1 |
| 17 | `Moon_Pumpkin` | 50×400 | 20.9 | g70-bg-layers-1 |
| 18 | `Moon_Snow` | 50×400 | 20.9 | g70-bg-layers-1 |
| 19 | `Extra_138` | 198×102 | 20.8 | d16-proj-ai-151-160 |
| 20 | `Extra_139` | 198×102 | 20.8 | d16-proj-ai-151-160 |
| 21 | `NPC_63` | 56×322 | 18.8 | d34-1-70, g8-npc-kind-wire-1 |
| 22 | `NPC_64` | 56×322 | 18.8 | g8-npc-kind-wire-1 |
| 23 | `NPC_177` | 76×228 | 17.9 | g8-npc-kind-wire-1 |
| 24 | `NPC_174` | 70×230 | 16.7 | g8-npc-kind-wire-1 |
| 25 | `NPC_165` | 62×248 | 16.0 | g8-npc-kind-wire-1 |
| 26 | `NPC_182` | 34×440 | 15.9 | g8-npc-kind-wire-1 |
| 27 | `NPC_152` | 80×184 | 15.3 | g8-npc-kind-wire-1 |
| 28 | `NPC_157` | 84×168 | 14.6 | c4-style44-flying-fish |
| 29 | `NPC_615` | 66×210 | 14.4 | c4-style44-flying-fish |
| 30 | `NPC_47` | 48×280 | 14.1 | d34-1-70 |
| 31 | `Extra_104` | 264×48 | 13.3 | d16-proj-ai-151-160 |
| 32 | `Extra_133` | 228×52 | 12.4 | d16-proj-ai-151-160 |
| 33 | `NPC_688` | 46×252 | 12.2 | c4-style44-flying-fish |
| 34 | `NPC_164` | 70×150 | 10.9 | g8-npc-kind-wire-1 |
| 35 | `NPC_150` | 52×192 | 10.5 | g8-npc-kind-wire-1 |
| 36 | `NPC_151` | 52×192 | 10.5 | g8-npc-kind-wire-1 |
| 37 | `NPC_158` | 52×192 | 10.5 | g8-npc-kind-wire-1 |
| 38 | `Extra_145` | 186×52 | 10.2 | d16-proj-ai-151-160 |
| 39 | `Extra_146` | 228×36 | 8.7 | d16-proj-ai-151-160 |
| 40 | `NPC_175` | 56×141 | 8.3 | d23-npc-ai-styles-9-14 |
| 41 | `NPC_361` | 26×286 | 8.1 | e26-luck-system |
| 42 | `Extra_132` | 154×48 | 7.8 | d16-proj-ai-151-160 |
| 43 | `NPC_69` | 36×200 | 7.7 | g8-npc-kind-wire-1 |
| 44 | `NPC_195` | 20×322 | 7.1 | g8-npc-kind-wire-1 |
| 45 | `NPC_57` | 38×168 | 6.8 | c4-style44-flying-fish, d34-1-70 |
| 46 | `NPC_592` | 38×168 | 6.8 | c4-style44-flying-fish |
| 47 | `NPC_607` | 38×168 | 6.8 | c4-style44-flying-fish |
| 48 | `UI/House_Banner` | 68×84 | 6.0 | g21-sounds-main-1 |
| 49 | `NPC_56` | 52×108 | 5.9 | d23-npc-ai-styles-9-14, d34-1-70 |
| 50 | `NPC_374` | 38×144 | 5.8 | c8-style66-crawling-critters |
| 51 | `NPC_375` | 38×144 | 5.8 | c8-style66-crawling-critters |
| 52 | `NPC_169` | 22×216 | 5.2 | g8-npc-kind-wire-1 |
| 53 | `Extra_147` | 304×14 | 4.9 | d16-proj-ai-151-160 |
| 54 | `NPC_604` | 26×160 | 4.5 | e26-luck-system |
| 55 | `NPC_605` | 26×160 | 4.5 | e26-luck-system |
| 56 | `Extra_280` | 64×64 | 4.4 | g70-bg-layers-1 |
| 57 | `Sun2` | 64×64 | 4.4 | g70-bg-layers-1 |
| 58 | `Sun3` | 64×64 | 4.4 | g70-bg-layers-1 |
| 59 | `UI/TexturePackButtonsOutline` | 64×64 | 4.4 | e10-player-sounds-wave |
| 60 | `NPC_39` | 42×90 | 4.0 | d34-1-70 |
| 61 | `NPC_40` | 42×90 | 4.0 | d34-1-70 |
| 62 | `NPC_41` | 42×90 | 4.0 | d34-1-70 |
| 63 | `Moon_Smiley` | 56×66 | 3.9 | g70-bg-layers-1 |
| 64 | `Inventory_Back19` | 52×52 | 2.9 | g56-ui-void-vault |
| 65 | `Extra_103` | 210×10 | 2.5 | d16-proj-ai-151-160 |
| 66 | `Extra_140` | 64×34 | 2.4 | d16-proj-ai-151-160 |
| 67 | `NPC_661` | 24×72 | 1.9 | d125-empress-r7-drain (tip 2dfe026d, local) |
| 68 | `NPC_70` | 34×34 | 1.3 | g8-npc-kind-wire-1 |
| 69 | `UI/ChestStack_0` | 32×30 | 1.1 | g21-sounds-main-1 |
| 70 | `UI/ChestStack_1` | 32×30 | 1.1 | g21-sounds-main-1 |
| 71 | `UI/ChestStack_2` | 32×30 | 1.1 | g21-sounds-main-1 |
| 72 | `UI/ChestStack_3` | 32×30 | 1.1 | g21-sounds-main-1 |
| 73 | `UI/Sort_0` | 30×30 | 1.0 | g21-sounds-main-1 |
| 74 | `UI/Sort_1` | 30×30 | 1.0 | g21-sounds-main-1 |
| 75 | `Extra_135` | 32×16 | 0.6 | d16-proj-ai-151-160 |
| 76 | `Extra_144` | 32×16 | 0.6 | d16-proj-ai-151-160 |
| 77 | `Extra_136` | 28×12 | 0.4 | d16-proj-ai-151-160 |
| 78 | `Extra_137` | 28×12 | 0.4 | d16-proj-ai-151-160 |
| 79 | `Extra_148` | 28×12 | 0.4 | d16-proj-ai-151-160 |
| 80 | `Extra_255` | 18×18 | 0.4 | g21-sounds-main-1 |
| 81 | `Extra_256` | 18×18 | 0.4 | g21-sounds-main-1 |
| 82 | `NPC_357` | 14×16 | 0.3 | c8-style66-crawling-critters |
| 83 | `NPC_606` | 14×16 | 0.3 | c8-style66-crawling-critters |

## Appendix B-2 - the RETIRE delta (72 new names, priority order: padded area desc)

The §14 re-extraction delta: 54 pushed branches / 504 names (+80 vs the
424 base, 0 gone; 8 of the 80 were already forecast/deferred in the 490
union — NPC_79/80 + NPC_454-459 — so +72 genuinely-new), deduped against
Appendix A + B (the 490-name union). Generated by
`tools/_d35_delta_table.py` from `tools/_d35_union_list.json` (the FULL
562-name table is `tools/_d35_union_table.md`). Legend: the 26 MapBG rows
are e23's bestiary backdrops the FINAL regex dropped (§14.1, always
packed, now listed); NPC_454-459 were d40-forecast, now packed by d9's
proj-ai lane (the dagger retires for those six). The NPC-band rows ride
the per-world NPC page under the §10 lazy form, NOT the union.

| # | sheet | w×h | padded kpx | owner branch(es) |
|---|-------|-----|-----------|------------------|
| 1 | `Armor_Head_195` | 40×1120 | 47.1 | f41-birthday-party-2 |
| 2 | `NPC_630` | 40×896 | 37.7 | f42-sandstorm-2 |
| 3 | `NPC_521` | 100×352 | 36.1 | d9-proj-ai-81-90 |
| 4 | `Player_HairAlt_1` | 40×784 | 33.0 | f41-birthday-party-2 |
| 5 | `NPC_619` | 48×444 | 22.3 | d11-proj-ai-101-110 |
| 6 | `NPC_494` | 62×256 | 16.5 | d41-491-560 |
| 7 | `Glow_294` | 36×432 | 16.5 | g84-plantera-bulb-draw |
| 8 | `NPC_496` | 40×360 | 15.2 | d41-491-560 |
| 9 | `NPC_497` | 40×360 | 15.2 | d41-491-560 |
| 10 | `NPC_356` | 24×576 | 15.0 | d39-351-420 |
| 11 | `NPC_495` | 54×256 | 14.4 | d41-491-560 |
| 12 | `NPC_580` | 66×192 | 13.2 | f42-sandstorm-2 |
| 13 | `NPC_172` | 40×210 | 8.9 | g5-npc-style7-caster-doc |
| 14 | `Shroom_Tops` | 186×44 | 8.6 | g12-dust-main-sites-2 |
| 15 | `MapBG13` | 115×65 | 7.8 | e23-bestiary-ui-grid |
| 16 | `MapBG22` | 115×65 | 7.8 | e23-bestiary-ui-grid |
| 17 | `MapBG23` | 115×65 | 7.8 | e23-bestiary-ui-grid |
| 18 | `MapBG24` | 115×65 | 7.8 | e23-bestiary-ui-grid |
| 19 | `MapBG26` | 115×65 | 7.8 | e23-bestiary-ui-grid |
| 20 | `MapBG27` | 115×65 | 7.8 | e23-bestiary-ui-grid |
| 21 | `MapBG28` | 115×65 | 7.8 | e23-bestiary-ui-grid |
| 22 | `MapBG29` | 115×65 | 7.8 | e23-bestiary-ui-grid |
| 23 | `MapBG30` | 115×65 | 7.8 | e23-bestiary-ui-grid |
| 24 | `MapBG31` | 115×65 | 7.8 | e23-bestiary-ui-grid |
| 25 | `MapBG34` | 115×65 | 7.8 | e23-bestiary-ui-grid |
| 26 | `MapBG35` | 115×65 | 7.8 | e23-bestiary-ui-grid |
| 27 | `MapBG36` | 115×65 | 7.8 | e23-bestiary-ui-grid |
| 28 | `MapBG37` | 115×65 | 7.8 | e23-bestiary-ui-grid |
| 29 | `MapBG38` | 115×65 | 7.8 | e23-bestiary-ui-grid |
| 30 | `MapBG39` | 115×65 | 7.8 | e23-bestiary-ui-grid |
| 31 | `MapBG40` | 115×65 | 7.8 | e23-bestiary-ui-grid |
| 32 | `MapBG41` | 115×65 | 7.8 | e23-bestiary-ui-grid |
| 33 | `MapBG42` | 115×65 | 7.8 | e23-bestiary-ui-grid |
| 34 | `MapBG5` | 115×65 | 7.8 | e23-bestiary-ui-grid |
| 35 | `MapBG6` | 115×65 | 7.8 | e23-bestiary-ui-grid |
| 36 | `MapBG7` | 115×65 | 7.8 | e23-bestiary-ui-grid |
| 37 | `MapBG8` | 115×65 | 7.8 | e23-bestiary-ui-grid |
| 38 | `MapBG9` | 115×65 | 7.8 | e23-bestiary-ui-grid |
| 39 | `MapBG20` | 115×61 | 7.4 | e23-bestiary-ui-grid |
| 40 | `MapBG21` | 115×61 | 7.4 | e23-bestiary-ui-grid |
| 41 | `NPC_693` | 40×168 | 7.1 | g5-npc-style7-caster-doc |
| 42 | `Ghost` | 34×176 | 6.4 | c46-respawn-breath-drown |
| 43 | `NPC_516` | 38×152 | 6.2 | d23-npc-ai-styles-9-14 |
| 44 | `NPC_366` | 38×112 | 4.6 | d39-351-420 |
| 45 | `NPC_367` | 34×104 | 3.8 | d39-351-420 |
| 46 | `Extra_260` | 90×36 | 3.5 | g84-plantera-bulb-draw |
| 47 | `NPC_676` | 34×52 | 1.9 | f47-liquid-shimmer-2 |
| 48 | `NPC_546` | 42×42 | 1.9 | f42-sandstorm-2 |
| 49 | `UI/Bestiary/Stat_Gold` | 47×31 | 1.6 | e23-bestiary-ui-grid |
| 50 | `UI/Bestiary/Stat_Platinum` | 47×31 | 1.6 | e23-bestiary-ui-grid |
| 51 | `UI/Bestiary/Stat_Silver` | 47×31 | 1.6 | e23-bestiary-ui-grid |
| 52 | `NPC_595` | 20×48 | 1.1 | d33-npc-ai-styles-95-127 |
| 53 | `NPC_596` | 20×48 | 1.1 | d33-npc-ai-styles-95-127 |
| 54 | `NPC_597` | 20×48 | 1.1 | d33-npc-ai-styles-95-127 |
| 55 | `NPC_598` | 20×48 | 1.1 | d33-npc-ai-styles-95-127 |
| 56 | `NPC_599` | 20×48 | 1.1 | d33-npc-ai-styles-95-127 |
| 57 | `NPC_600` | 20×48 | 1.1 | d33-npc-ai-styles-95-127 |
| 58 | `NPC_601` | 20×48 | 1.1 | d33-npc-ai-styles-95-127 |
| 59 | `Chat2` | 32×28 | 1.0 | f22-ui-screens-4 |
| 60 | `Flame` | 22×22 | 0.6 | c46-respawn-breath-drown |
| 61 | `NPC_377` | 18×24 | 0.5 | d39-351-420 |
| 62 | `OneDropLogo` | 19×19 | 0.4 | f28-tooltip-lines-2 |
| 63 | `UI/Scrollbar` | 20×16 | 0.4 | e23-bestiary-ui-grid |
| 64 | `NPC_666` | 16×16 | 0.3 | d23-npc-ai-styles-9-14 |
| 65 | `Extra_66` | 12×12 | 0.2 | d13-proj-ai-121-130 |
| 66 | `Chain38` | 16×8 | 0.2 | g82-rope-coil-draw |
| 67 | `Chain39` | 16×8 | 0.2 | g82-rope-coil-draw |
| 68 | `Chains_4` | 16×8 | 0.2 | g82-rope-coil-draw |
| 69 | `Chains_5` | 16×8 | 0.2 | g82-rope-coil-draw |
| 70 | `Chains_6` | 16×8 | 0.2 | g82-rope-coil-draw |
| 71 | `Chains_7` | 16×8 | 0.2 | g82-rope-coil-draw |
| 72 | `Chain16` | 8×8 | 0.1 | g82-rope-coil-draw |

## Appendix C - the SEAL delta (145 new names, priority order: padded area desc)

The §17.2 seal re-extraction delta: 65 pushed branches / 630 names
(+126 vs the 504 retire base, 0 gone; 10 of the 126 were already
forecast/deferred in the 562 union — d125's tip 3 now PUSHED + NPC_430/
442-447 — so +116 genuinely-new pushed) PLUS d42's 29 FORECAST names
(the research/238 NO-PACK verdict, marked FORECAST below), deduped
against Appendix A + B + B-2 = the **707-name census union**. Generated
by `tools/_d35_seal_union.py` → `_d35_seal_union.json` (the full table;
the delta body `tools/_d35_seal_delta_table.md`). The NPC-band rows
(109 of the 145) ride the per-world NPC page under the §10 lazy form,
NOT the eager union; the 36 non-NPC rows ride the lazy union page.

| # | name | w x h | padded kpx | owners |
|---|---|---|---|---|
| 1 | `NPC_657` | 360x1952 | 707.3 | d43-npc-kinds-631-697 |
| 2 | `NPC_696` | 384x1242 | 480.2 | d43-npc-kinds-631-697 |
| 3 | `NPC_620` | 102x1764 | 183.7 | d42 (FORECAST: NO-PACK, research/238) |
| 4 | `NPC_692` | 136x992 | 137.2 | d43-npc-kinds-631-697 |
| 5 | `NPC_631` | 60x1554 | 96.5 | d43-npc-kinds-631-697 |
| 6 | `NPC_618` | 150x528 | 80.6 | d42 (FORECAST: NO-PACK, research/238) |
| 7 | `NPC_638` | 70x1064 | 76.8 | d43-npc-kinds-631-697, g10-npc-kind-wire-3 |
| 8 | `NPC_628` | 70x782 | 56.4 | d42 (FORECAST: NO-PACK, research/238) |
| 9 | `NPC_586` | 60x840 | 52.2 | d42 (FORECAST: NO-PACK, research/238) |
| 10 | `NPC_656` | 46x1080 | 51.9 | d43-npc-kinds-631-697, g10-npc-kind-wire-3 |
| 11 | `Acc_Shield_10` | 44x1120 | 51.6 | f48-dodge-2 |
| 12 | `Acc_Shield_11` | 44x1120 | 51.6 | f48-dodge-2 |
| 13 | `Acc_Shield_5` | 44x1120 | 51.6 | f48-dodge-2 |
| 14 | `Acc_Shield_9` | 44x1120 | 51.6 | f48-dodge-2 |
| 15 | `NPC_637` | 44x1120 | 51.6 | d43-npc-kinds-631-697, g10-npc-kind-wire-3 |
| 16 | `Acc_Shield_2` | 42x1120 | 49.4 | f48-dodge-2 |
| 17 | `Acc_Shield_7` | 42x1120 | 49.4 | f48-dodge-2 |
| 18 | `Acc_Shield_8` | 42x1120 | 49.4 | f48-dodge-2 |
| 19 | `Acc_Shield_1` | 40x1120 | 47.1 | f48-dodge-2 |
| 20 | `Acc_Shield_12` | 40x1120 | 47.1 | f48-dodge-2 |
| 21 | `Acc_Shield_3` | 40x1120 | 47.1 | f48-dodge-2 |
| 22 | `Acc_Shield_4` | 40x1120 | 47.1 | f48-dodge-2 |
| 23 | `Acc_Shield_6` | 40x1120 | 47.1 | f48-dodge-2 |
| 24 | `Armor_Head_101` | 40x1118 | 47.0 | d6-proj-ai-51-60 |
| 25 | `Armor_Head_156` | 40x1118 | 47.0 | d6-proj-ai-51-60 |
| 26 | `NPC_379` | 60x696 | 43.3 | d39-npc-kinds-351-420, f18-npc-kinds-verify-2 |
| 27 | `NPC_380` | 60x696 | 43.3 | d39-npc-kinds-351-420, f18-npc-kinds-verify-2 |
| 28 | `NPC_591` | 88x476 | 43.0 | d42 (FORECAST: NO-PACK, research/238) |
| 29 | `NPC_684` | 72x532 | 39.5 | d43-npc-kinds-631-697, g10-npc-kind-wire-3 |
| 30 | `NPC_635` | 40x896 | 37.7 | d43-npc-kinds-631-697 |
| 31 | `NPC_159` | 40x840 | 35.4 | g8-npc-kind-wire-1 |
| 32 | `NPC_632` | 36x800 | 30.5 | d43-npc-kinds-631-697 |
| 33 | `NPC_683` | 46x616 | 29.7 | d43-npc-kinds-631-697, g10-npc-kind-wire-3 |
| 34 | `NPC_153` | 72x384 | 28.6 | d36-npc-kinds-141-210 |
| 35 | `NPC_154` | 72x384 | 28.6 | d36-npc-kinds-141-210 |
| 36 | `NPC_678` | 44x616 | 28.4 | d43-npc-kinds-631-697, g10-npc-kind-wire-3 |
| 37 | `NPC_680` | 46x588 | 28.3 | d43-npc-kinds-631-697, g10-npc-kind-wire-3 |
| 38 | `NPC_498` | 40x594 | 25.0 | d41-npc-kinds-491-560 |
| 39 | `NPC_499` | 40x594 | 25.0 | d41-npc-kinds-491-560 |
| 40 | `NPC_500` | 40x594 | 25.0 | d41-npc-kinds-491-560 |
| 41 | `NPC_501` | 40x594 | 25.0 | d41-npc-kinds-491-560 |
| 42 | `NPC_502` | 40x594 | 25.0 | d41-npc-kinds-491-560 |
| 43 | `NPC_503` | 40x594 | 25.0 | d41-npc-kinds-491-560 |
| 44 | `NPC_504` | 40x594 | 25.0 | d41-npc-kinds-491-560 |
| 45 | `NPC_505` | 40x594 | 25.0 | d41-npc-kinds-491-560 |
| 46 | `NPC_506` | 40x594 | 25.0 | d41-npc-kinds-491-560 |
| 47 | `NPC_679` | 38x616 | 24.7 | d43-npc-kinds-631-697, g10-npc-kind-wire-3 |
| 48 | `NPC_611` | 40x520 | 21.9 | d42 (FORECAST: NO-PACK, research/238) |
| 49 | `NPC_689` | 40x520 | 21.9 | d43-npc-kinds-631-697 |
| 50 | `NPC_682` | 36x560 | 21.4 | d43-npc-kinds-631-697, g10-npc-kind-wire-3 |
| 51 | `NPC_625` | 56x360 | 21.0 | d42 (FORECAST: NO-PACK, research/238) |
| 52 | `NPC_508` | 84x240 | 20.8 | d41-npc-kinds-491-560 |
| 53 | `NPC_594` | 256x76 | 20.1 | d42 (FORECAST: NO-PACK, research/238) |
| 54 | `NPC_670` | 38x476 | 19.1 | d43-npc-kinds-631-697, g10-npc-kind-wire-3 |
| 55 | `NPC_681` | 38x476 | 19.1 | d43-npc-kinds-631-697, g10-npc-kind-wire-3 |
| 56 | `NPC_616` | 50x364 | 19.0 | d42 (FORECAST: NO-PACK, research/238) |
| 57 | `NPC_617` | 50x364 | 19.0 | d42 (FORECAST: NO-PACK, research/238) |
| 58 | `NPC_624` | 32x462 | 15.8 | d42 (FORECAST: NO-PACK, research/238) |
| 59 | `NPC_660` | 90x168 | 15.6 | d43-npc-kinds-631-697 |
| 60 | `NPC_148` | 28x504 | 15.2 | d36-npc-kinds-141-210 |
| 61 | `NPC_149` | 28x504 | 15.2 | d36-npc-kinds-141-210 |
| 62 | `NPC_540` | 48x280 | 14.1 | e21-wiring-device-arms-3 |
| 63 | `NPC_614` | 48x280 | 14.1 | d42 (FORECAST: NO-PACK, research/238) |
| 64 | `NPC_646` | 48x280 | 14.1 | d43-npc-kinds-631-697 |
| 65 | `NPC_647` | 48x280 | 14.1 | d43-npc-kinds-631-697 |
| 66 | `NPC_648` | 48x280 | 14.1 | d43-npc-kinds-631-697 |
| 67 | `NPC_649` | 48x280 | 14.1 | d43-npc-kinds-631-697 |
| 68 | `NPC_650` | 48x280 | 14.1 | d43-npc-kinds-631-697 |
| 69 | `NPC_651` | 48x280 | 14.1 | d43-npc-kinds-631-697 |
| 70 | `NPC_652` | 48x280 | 14.1 | d43-npc-kinds-631-697 |
| 71 | `TownNPCs/Portraits/Portrait_Window` | 112x112 | 13.0 | g58-ui-shop-happiness |
| 72 | `NPC_662` | 50x232 | 12.2 | d43-npc-kinds-631-697 |
| 73 | `NPC_671` | 46x240 | 11.6 | d43-npc-kinds-631-697 |
| 74 | `NPC_672` | 46x240 | 11.6 | d43-npc-kinds-631-697 |
| 75 | `NPC_673` | 46x240 | 11.6 | d43-npc-kinds-631-697 |
| 76 | `NPC_674` | 46x240 | 11.6 | d43-npc-kinds-631-697 |
| 77 | `NPC_675` | 46x240 | 11.6 | d43-npc-kinds-631-697 |
| 78 | `NPC_687` | 30x338 | 10.9 | d43-npc-kinds-631-697 |
| 79 | `NPC_639` | 50x192 | 10.1 | d43-npc-kinds-631-697 |
| 80 | `NPC_640` | 50x192 | 10.1 | d43-npc-kinds-631-697 |
| 81 | `NPC_641` | 50x192 | 10.1 | d43-npc-kinds-631-697 |
| 82 | `NPC_642` | 50x192 | 10.1 | d43-npc-kinds-631-697 |
| 83 | `NPC_643` | 50x192 | 10.1 | d43-npc-kinds-631-697 |
| 84 | `NPC_644` | 50x192 | 10.1 | d43-npc-kinds-631-697 |
| 85 | `NPC_645` | 50x192 | 10.1 | d43-npc-kinds-631-697 |
| 86 | `NPC_629` | 32x276 | 9.5 | d42 (FORECAST: NO-PACK, research/238) |
| 87 | `NPC_578` | 48x160 | 8.1 | d42 (FORECAST: NO-PACK, research/238) |
| 88 | `MapBGOverlay1` | 115x65 | 7.8 | e23-bestiary-ui-grid |
| 89 | `MapBGOverlay2` | 115x65 | 7.8 | e23-bestiary-ui-grid |
| 90 | `MapBGOverlay3` | 115x65 | 7.8 | e23-bestiary-ui-grid |
| 91 | `MapBGOverlay4` | 115x65 | 7.8 | e23-bestiary-ui-grid |
| 92 | `MapBGOverlay5` | 115x65 | 7.8 | e23-bestiary-ui-grid |
| 93 | `MapBGOverlay6` | 115x65 | 7.8 | e23-bestiary-ui-grid |
| 94 | `MapBGOverlay7` | 115x65 | 7.8 | e23-bestiary-ui-grid |
| 95 | `MapBGOverlay8` | 115x65 | 7.8 | e23-bestiary-ui-grid |
| 96 | `MapBGOverlay9` | 115x65 | 7.8 | e23-bestiary-ui-grid |
| 97 | `NPC_634` | 44x160 | 7.5 | d43-npc-kinds-631-697 |
| 98 | `NPC_141` | 60x108 | 6.8 | d36-npc-kinds-141-210 |
| 99 | `NPC_590` | 38x144 | 5.8 | d42 (FORECAST: NO-PACK, research/238) |
| 100 | `NPC_691` | 38x144 | 5.8 | d43-npc-kinds-631-697 |
| 101 | `NPC_695` | 44x114 | 5.3 | d43-npc-kinds-631-697 |
| 102 | `NPC_582` | 34x144 | 5.3 | d42 (FORECAST: NO-PACK, research/238) |
| 103 | `NPC_593` | 25x180 | 4.9 | d42 (FORECAST: NO-PACK, research/238) |
| 104 | `NPC_669` | 26x160 | 4.5 | d43-npc-kinds-631-697 |
| 105 | `NPC_378` | 28x128 | 3.9 | d39-npc-kinds-351-420, f18-npc-kinds-verify-2 |
| 106 | `NPC_658` | 48x64 | 3.3 | d43-npc-kinds-631-697 |
| 107 | `NPC_659` | 48x64 | 3.3 | d43-npc-kinds-631-697 |
| 108 | `NPC_686` | 40x76 | 3.3 | d43-npc-kinds-631-697 |
| 109 | `NPC_621` | 38x74 | 3.0 | d42 (FORECAST: NO-PACK, research/238) |
| 110 | `NPC_622` | 38x74 | 3.0 | d42 (FORECAST: NO-PACK, research/238) |
| 111 | `NPC_623` | 38x74 | 3.0 | d42 (FORECAST: NO-PACK, research/238) |
| 112 | `NPC_626` | 20x132 | 2.9 | d42 (FORECAST: NO-PACK, research/238) |
| 113 | `NPC_627` | 20x132 | 2.9 | d42 (FORECAST: NO-PACK, research/238) |
| 114 | `NPC_664` | 14x180 | 2.9 | d43-npc-kinds-631-697 |
| 115 | `Extra_211` | 104x24 | 2.8 | g21-sounds-main-1, g81-wire-draw-overlay |
| 116 | `NPC_510` | 50x48 | 2.6 | d41-npc-kinds-491-560 |
| 117 | `NPC_511` | 50x48 | 2.6 | d41-npc-kinds-491-560 |
| 118 | `NPC_512` | 50x48 | 2.6 | d41-npc-kinds-491-560 |
| 119 | `NPC_583` | 24x96 | 2.5 | d42 (FORECAST: NO-PACK, research/238) |
| 120 | `NPC_584` | 24x96 | 2.5 | d42 (FORECAST: NO-PACK, research/238) |
| 121 | `NPC_585` | 24x96 | 2.5 | d42 (FORECAST: NO-PACK, research/238) |
| 122 | `NPC_612` | 28x70 | 2.2 | d42 (FORECAST: NO-PACK, research/238) |
| 123 | `NPC_613` | 28x70 | 2.2 | d42 (FORECAST: NO-PACK, research/238) |
| 124 | `NPC_610` | 20x96 | 2.2 | d42 (FORECAST: NO-PACK, research/238) |
| 125 | `BeetleOrb` | 26x68 | 2.0 | f50-hurt-reactions |
| 126 | `NPC_653` | 24x72 | 1.9 | d43-npc-kinds-631-697 |
| 127 | `NPC_667` | 32x52 | 1.8 | d43-npc-kinds-631-697 |
| 128 | `UI/BlockReplace_0` | 72x22 | 1.8 | g21-sounds-main-1, g81-wire-draw-overlay |
| 129 | `NPC_690` | 32x42 | 1.5 | d43-npc-kinds-631-697 |
| 130 | `Lock_0` | 50x22 | 1.2 | g21-sounds-main-1 |
| 131 | `Lock_1` | 50x22 | 1.2 | g21-sounds-main-1 |
| 132 | `NPC_579` | 52x20 | 1.2 | d42 (FORECAST: NO-PACK, research/238) |
| 133 | `NPC_519` | 30x30 | 1.0 | d41-npc-kinds-491-560 |
| 134 | `NPC_685` | 32x28 | 1.0 | d43-npc-kinds-631-697 |
| 135 | `NPC_513` | 26x32 | 1.0 | d41-npc-kinds-491-560 |
| 136 | `NPC_514` | 26x32 | 1.0 | d41-npc-kinds-491-560 |
| 137 | `NPC_515` | 26x32 | 1.0 | d41-npc-kinds-491-560 |
| 138 | `UI/Cursor_10` | 22x22 | 0.6 | e19-liquid-quicksell-shimmer |
| 139 | `UI/Cursor_8` | 22x22 | 0.6 | e19-liquid-quicksell-shimmer |
| 140 | `UI/Cursor_9` | 22x22 | 0.6 | e19-liquid-quicksell-shimmer |
| 141 | `UI/Cursor_3` | 20x20 | 0.5 | e19-liquid-quicksell-shimmer |
| 142 | `UI/Cursor_6` | 20x20 | 0.5 | e19-liquid-quicksell-shimmer |
| 143 | `UI/Cursor_7` | 20x20 | 0.5 | e19-liquid-quicksell-shimmer |
| 144 | `UI/InfoIcon_13` | 18x18 | 0.4 | g81-wire-draw-overlay |
| 145 | `NPC_665` | 16x16 | 0.3 | d43-npc-kinds-631-697 |
