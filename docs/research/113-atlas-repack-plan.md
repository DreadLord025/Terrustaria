# 113 — Atlas repack plan: the W4 union overflows the 8192 strict-GPU bound

Status: 🔄 measurement DONE, implementation contract for the merger wave.
Branch: `task/W4-d35-npc-kinds-71-140` (ATLAS-REPACK phase). Base: `a725acb0`
(wave-3 landing). Code-only mode: no cargo/tests run; every number below is
measured by the pinned sim harness over the REAL retail `.xnb` dims (LZX
first-frame decoder, `tools/_d35_atlas_sim_pinned.js`), with `verify()`
(no overlap incl. PAD, in-bounds) and double-run determinism on every layout.

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
| **incremental skyline** (page 0 = HEAD base packed ALONE) | **8188** | **1924** | 317-385 | page-0 layout == today's, every existing UV unchanged |
| incremental maxrects | 7988 | 1924 | same | page-0 repack unnecessary; skyline form wins on UV stability |
| spill maxrects (cap 8192, overflow → fresh page) | 8192 | 700-1018 | 7031-7836 | most sheets move; page-0 UVs all change |
| boundary skyline (pack unbounded, cut at 8192, re-pack tail) | 8163-8192 | 1232-1298 | 8868-8941 | below-boundary UVs stable, but tail is huge |

**Chosen fix: the INCREMENTAL form with the landed skyline packer.**
Page 0 keeps HEAD's exact layout (8188 px, the sheets-phase witness
numbers stay valid); every W4 addition packs into page 1 (1924 px for the
full 385-sheet worst case — 23.5% of a page). The merger's per-branch
sheet additions are UV-neutral for everything already on main, the
`build_ref` world-swap contract is preserved per page, and the d35
deferred 12 + d40's 56 simply append to page 1's input list.

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
  pack page 0 exactly as today; sheets that don't fit the 8192 cap spill
  to page 1 (same sort, same skyline, fresh bin). The incremental contract:
  page 0's input is the HEAD enumeration, page 1's input is the additions —
  but the IMPLEMENTATION should be the general cap-spill form (sort once,
  fill page 0 to the cap, overflow → page 1), because the merger cannot
  guarantee "HEAD first" ordering once 26 branches' const-list rows
  interleave. **Measured consequence**: the general spill form moves most
  sheets to page 1 (7031+); the incremental form keeps them. The merger
  chooses: (a) general spill — simpler code, all UVs recomputed anyway
  (they always are, per build); or (b) two-pass incremental — page 0 =
  the pre-W4 name set pinned by a witness, page 1 = the rest. Both are
  deterministic; (a) is recommended (no pinned name list to maintain).
  The heights in §1's table bound both forms.
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
  `layer - MAIN_PAGE_COUNT`. Alternative (cleaner): a separate
  `page: u32` instance field + a `font` flag — but that grows the
  instance layout (11 attributes today) and touches every push site;
  the layer-range encoding touches ONLY `sample_tex` + the UV producers.
  **Recommended: layer-range encoding with a `MAIN_PAGES` uniform**
  (or a compile-time constant 2, bumped when page 3 arrives).
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
  `uv_rect(rect, self.atlas_size, …)` call (30 sites in assets.rs,
  `uv_rect` def at :3354-3366) reads `page_heights[rect.page]`.
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
  becomes per-page (a page over 8192 is now a BUG, not a warning — the
  spill form guarantees the cap); `app/mod.rs:4880,4904` index
  `atlas.pixels` for the opacity pre-pass → `pages[rect.page].pixels`.
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
  PACKABLE once page 1 exists — the merger wave should admit them (they
  cost page-1 height only; measured: +0 px on page 0, page 1 1924 → 1924
  in the incremental form because the deferred sheets are already in the
  385-addition worst case).
- `ui/bestiary.rs:2077,2197-2204`, `render/gore.rs:379-413` — synthetic
  `(8192, 8192)` atlas sizes in unit tests: stay valid for page-0 rects.

## 3. Implementation contract for the merger

Order of operations (each step independently mergeable):

1. **tr-render first** (the deep module): `SheetRect.page`, `Atlas.pages`,
   cap-spill `build_ref` (skyline unchanged, page 0 fills to 8192, overflow
   re-packs page 1 with the same sort order restricted to the overflow
   set — deterministic: the overflow set is a pure function of the input),
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
   the skyline column must show page 0 ≤ 8192 AND page 1 ≤ 8192 for every
   scenario. The harness's `spillTwoPage`/`boundaryTwoPage` forms are the
   reference; the Rust cap-spill form must match `spillTwoPage`'s page
   assignment shape (not necessarily its maxrects geometry — the Rust
   packer is skyline; the skyline spill form is the boundary form with
   the cap applied DURING packing, which the harness can simulate by
   capping the skyline bin height — a follow-up harness extension).

**What NOT to do**: do not swap skyline for maxrects (measured worse on
the union, chaotic ordering); do not compress the atlas to BC2/BC3 to
"fit" (the paint/dye/sRGB byte-multiply invariant — ANALYSIS S1-G-4 —
depends on exact RGBA8 texels; BC lossy compression breaks pixel
equality); do not raise the bound past 8192 (the strict-GPU limit is
real: `max_texture_dimension_2d` on the target adapters).

**Memory cost**: page 1 at 1924 px × 8192 × 4 B ≈ 63 MB (incremental
form) or ~33 MB (spill form, 1018 px). Page 0 stays ~268 MB. Total
~300-330 MB — the font array already costs 256 MB/layer (BC2, 8192²),
so this is within the existing budget shape. The spill form's smaller
page 1 is a secondary argument for it.

## 4. The union list (407 names, owner branches, priority order)

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

## Appendix A - the full 407-name union list (priority order: padded area desc)

Legend: branch slugs omit the W4- prefix; dagger = d40 FORECAST (data-only branch, sheets phase pending); double-dagger = d35 DEFERRED (admitted by the repack). Generated by tools/_d35_union_table.py from the REAL .xnb dims.

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
