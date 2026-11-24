# W4-f15 — TILE-DRAW-2 glow-bind wiring census (claim + plan)

In-flight probe (git rev-list vs main a725acb0, worktree TR-W4-f15):
- co-lane `task/W4-g35-glow-records-wire`, `task/W4-g29-tile-draw-pass-2`,
  `task/W4-g33/g34-drawdata-wire-*`: **0 commits** — no in-flight code to
  collide with; the record producer (`drawdata_cases_a.rs`) is already on
  main via S1-F-9d.
- `task/W4-d61..d64-drawdata-cases-*`: audit-only branches (research docs).
  d64's remainder summary (15bcb93b) confirms the gap is WIRING, not arms:
  371/371 GetTileDrawData case ids ported; glow records produced but
  dropped at `render/tiles.rs:731` (`let _glow = …apply_cases_a(…)`).

## The gap (C# order)

`DrawSingleTile` (`TileDrawing.cs:864-1409`):
1. `GetTileDrawData` (:876) fills `glowTexture/glowSourceRect/glowColor`
   (the SECOND switch :6049-6507 — ported in `drawdata_cases_a::glow_switch`).
2. `flag` includes `glowTexture != null` (:971) — a glow record alone keeps
   a fully dark cell out of the black-merge (self-lit).
3. `DrawBasicTile` (:1176) draws the cell sprite (+ platform side-fills).
4. `Main.tileGlowMask[type]` table pass (:1178-1342) — separate wide task,
   explicitly out of scope (flame.rs module doc names it).
5. **The glow bind (:1343-1403)** ← THIS TASK:
   - 412 cross-fade (:1345-1357): `num20 = tileFrame[412]/60`,
     `num21 = (num20+1)%4`, `num22 = (tileFrame[412]%60)/60f`; two draws at
     `rect.Y += num20*54` / `num21*54`, colours `glowColor*(1-num22)` /
     `glowColor*num22` (XNA per-byte truncating float scale).
   - else (:1358-1402): one draw at
     `pos2 = (tileX*16 - (tileWidth-16)/2, tileY*16 + tileTop)`
     (**no** halfBrickHeight, **no** draw_dx/dy, **no** no_width_adjust
     exemption — 711's glow centres unlike its sprite), `Layer_Tiles` 1.
   - Platforms + slope (:1361-1401): pos2 = `vector` (the :1013 sprite
     anchor); a second glow side-fill at `pos2+(0,16)` with the SAME
     gates/X picks as `DrawBasicTile`'s side-fill (:1524/:1534 →
     `special_tiles::platform_side_fill`) but keeping the glow rect's
     Y/size (`:1375-1383`/`:1389-1397`), `Layer_BehindTiles` 1.
6. `DrawSingleTile_Flames` runs INSIDE DrawBasicTile (:1723) — i.e. the
   glow bind comes AFTER the port's `draw_tile_flames` hook.

## Seeded residuals folded in (from d61/d64 audits)

- **711** (d64 R7): first-switch glow `glowTexture = Tile[711]`, framed
  rect, colour stays `Color.Transparent` (:4931-4939, init :4538). Record
  producer: `apply_cases_a` 711 arm (mask sentinel = painted/tile sheet).
  Pixel note: transparent tint → invisible draw in both engines; binding
  it still flips the dark-merge gate (:971) exactly like C#.
- **637** (d61 batch 8): painted-sheet bind deviation stays (port draws
  the raw `Tiles_637` sheet; no paint system on glow) — recorded.
- **184 col 220** (d64 R11): disco colour now routable — `params.disco`
  already carries `Main.DiscoR/G/B`; wire it at the draw site.
- **564** (d64 R15): glow rect must read the PRE-clear animation row
  (`tileFrame*36`, C# :6109 reads addFrY before :6112 clears it). Fix in
  `apply_cases_a`'s 564 arm (producer-side, one arm).
- **mouseTextColor** (656/701/564): pass `params.mouse_text_color` instead
  of the parked 255 (flame pass already reads it).

## Deliberate deviations (to record in ANALYSIS)

- `Layer_Tiles`/`Layer_BehindTiles` targets unported (S1-G-13 family):
  glow quads ride the same row-major world stream as the cell sprite.
- Alpha-zero glow tints keep alpha 0: `sprite.wgsl:292-315` implements
  XNA premultiplied AlphaBlend (rgb factor One) natively — unlike the
  older flame note, no alpha-1 lift is needed here.
- 637 glow sheet unpainted; martianLight stays 0 (event unported).
- Glow on sway/sunflower arms draws at the un-swayed C# anchor (the C#
  glow bind never sees the grass/sunflower transforms).

## Phases (commit each)

1. Claim + census (this doc).
2. Sheet binding: add the 44 missing `Glow_{id}` sheets to
   `assets.rs GLOW_SHEET_NAMES` (mask set of `glow_switch`; all verified
   present as `assets/Content/Images/Glow_*.xnb`).
3. Producer residuals: 711 record arm, 564 pre-clear rect, 184 disco
   param, mouse-text pulse param; update the module's own tests.
4. Draw pass `render/tile_glow.rs` (412 cross-fade + plain bind +
   platform side-fill reusing `special_tiles::platform_side_fill` gates),
   wired in `tiles.rs` after `draw_tile_flames`; `_glow` bound; dark-merge
   `flame_tile` gate extended with the glow record; witnesses.
5. Docs (ANALYSIS/PORT_MAP/ROADMAP row touch) + anchors.txt row + push.
