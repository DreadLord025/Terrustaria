# Golden re-pin procedure — the skyline atlas packer (S1-G-12b)

**Status: owed, not done.** The skyline packer landed on main (`548fc335`,
merge `4791ae7f`) in code-only mode. **Every sheet's atlas position moved**, so
every normalized UV moved with it — the committed golden screenshots in
`docs/screenshots/` are stale **by design** (same texels, new addresses; no
pixel-behaviour change is intended). This doc is the procedure the next
build-allowed window executes to re-pin them. Do not "fix" a golden mismatch
from this wave by changing draw code — re-take the golden first, then diff.

## What moved, and why the goldens must change

- `crates/tr-render/src/atlas.rs::skyline_pack` replaced the best-fit
  decreasing-height shelves. Placement order is still tallest-first (stable),
  but the skyline backfills the width tails the shelves stranded, so nearly
  every `SheetRect { x, y }` differs from the shelf layout.
- All UVs are computed **at draw time** from `SheetRect` through the sheet map
  (`Lookups::sheet_uv` / `uv_rect`, `assets.rs:3166-3174`; the sprite pipeline's
  per-sheet UV table, `sprite.rs:494-500`). No UV is hardcoded in draw code —
  the pixels sampled are the same texels at new normalized coordinates.
- The atlas height changed (main set ~7289 px skyline vs ~8117 px shelves,
  `tools/atlas_sim.js` measured), so even sheets whose `x` happens to survive
  get a different `y / atlas.height` normalization.
- Name dedup (first occurrence wins) removed 41 duplicate `Projectile_*`
  decodes — UV-neutral for lookups (the HashMap always kept one rect per name).

**Expected visual delta: zero** if the port is correct (nearest sampling,
`PAD = 2` separation, texel-exact rects). Any *persistent* pixel difference
after a re-take + twin-run check is a real bug (sampling phase, PAD bleed,
a sheet copied to the wrong rect) — investigate with `tools/pixel_diff.py`
before accepting the new golden.

## Which goldens are stale

All 61 committed PNGs in `docs/screenshots/` (world scenes, boss fixtures,
lighting-mode pairs, UI screens — anything drawn through the atlas). The
`assets/test/golden.wld` / `golden.plr` **byte fixtures are NOT affected**
(they pin the save format, not rendering; S1-J-3). The world-hash pins are
NOT affected (worldgen never reads the atlas).

Known per-file recipes (from the ANALYSIS entries that pinned them; re-use
the same env pins, only the baseline bytes change):

| golden | recipe pins (besides the common set below) |
| --- | --- |
| `s1-g-3-lights-bane-{base,blend}.png` | `golden.wld`, `TR_DAY=1 TR_SHOT_FRAME=20 TR_DEBUG=0`, player present (ANALYSIS ~:1112/:1155) |
| `s1-e-19b-{color,retro,trippy,white}-{cave,surface}.png` | the lighting-mode pins of S1-E-19b (`TR_MAPSTYLE`/lighting mode per the row's ANALYSIS entry) |
| `bug-player-smear-post-z{1,1.5,2}.png` | `TR_ZOOM` per suffix; the `-pre-` pair is HISTORICAL evidence — never re-take it |
| `temple_altar_z{1,2}*.png` | `TR_CAM=1836,703`, seed 1, `TR_ZOOM` per suffix (M5-TEMPLE entry) |
| `m7-ui-4-ingame-options.png` | `TR_OPTIONS=1` |
| boss/NPC fixtures (`golem_*`, `king_slime`, `eoc_eye`, …) | `TR_BOSS=<name>` / `TR_NPC=<kinds>` per the fixture's ANALYSIS/ROADMAP entry |

For any golden whose recipe is not recorded, recover it from the ANALYSIS /
ROADMAP / ATLAS row that introduced the PNG (grep the file name).

## The harness (deterministic capture)

Binary: `cargo run -p tr-app --bin terraria-rust` (or the release build).
Env pins (`crates/tr-app/src/env.rs`):

- `TR_WORLD=<path>` — the `.trwld` to load (the golden scenes use the pristine
  `golden.wld`-derived save; **copy a pristine `.trwld` per run** — the autosave
  rewrites the loaded world, ATLAS B10 caveat).
- `TR_SHOT=<path.png>` — capture and exit; pins `dt` to exactly 1/60.
- `TR_SHOT_FRAME=<n>` — capture frame (default 20).
- `TR_CAM=x,y`, `TR_ZOOM=f`, `TR_DAY=f` — camera/zoom/time pins.
- `TR_NOPLAYER=1`, `TR_NO_BG=1`, `TR_NOHUD=1`, `TR_DEBUG=0` — strip the
  non-deterministic/overlay content for world-only shots.
- `TR_UI=<screen>`, `TR_MOUSE=x,y` — front-end screens / pinned cursor.
  Without `TR_MOUSE` the OS pointer is parked offscreen
  (`deterministic_pointer_park`, env.rs:1102) — do not move the mouse during
  a capture.
- Fixture pins as needed: `TR_BOSS`, `TR_NPC`, `TR_OPTIONS`, `TR_PVP`, …

## Re-pin steps (build-allowed window)

1. **Sync + build.** `git fetch origin --prune; git merge --ff-only origin/main`
   in your worktree; `cargo build --release -p tr-app`. Run the catalog suite
   first (`cargo test --workspace -- --skip migrates_version_2_kinds_to_item_ids`,
   judge by the `test result: ok` lines) — the atlas witnesses
   (`skyline_*`, `build_ref_matches_build_bit_for_bit`,
   `the_full_gore_and_mount_sheet_set_fits_the_strict_gpu_bound`) must pass
   before any golden work; they were WRITTEN NOT RUN in the code-only waves.
2. **Twin-run every recipe.** Run each capture command TWICE into two temp
   paths and require SHA256 equality (`Get-FileHash` / `sha256sum`). A twin
   mismatch is a determinism bug, not a re-pin — stop and diagnose
   (pointer park, autosave world rewrite, timing-sensitive hooks; ANALYSIS
   "Deterministic screenshots").
3. **Diff old vs new** with `python tools/pixel_diff.py <old> <new>`. Expect
   zero or near-zero differing pixels (UV phase is texel-exact). A large or
   structured diff = a real regression — do NOT overwrite the golden; file it.
   Check zoom 1.0 / 1.5 / 2.0 for background/parallax scenes (AGENTS.md).
4. **Overwrite the goldens** in `docs/screenshots/` with the twin-verified
   captures; keep the historical `-pre-` evidence pairs untouched.
5. **Record the re-pin** in `docs/ANALYSIS.md` (new SHA256 prefixes + the
   recipe, the "skyline repack moves every UV" note at ~:1170 is the pattern),
   update the owning ROADMAP/ATLAS rows' evidence, and commit on the re-pin
   branch. The E-4b lane owes the full-set re-take (merge `4791ae7f` message).
6. **Unblock note:** with the goldens re-pinned, the sheet-blocked lanes
   (N16 wyvern sheets, S1-L-8b mounts, gore draw refinements) can add their
   sheets — each addition moves UVs AGAIN, so each such lane re-takes the
   goldens it touches in its own merge window (same procedure).

## UV unit expectations — audit result (this wave)

Audited for absolute atlas-coordinate pins that the packer change would
stale: **none exist outside the packer's own witnesses.**

- `atlas.rs::tests` — all six skyline witnesses + `build_ref_matches_build_bit_for_bit`
  are invariant or hand-traced: `skyline_backfills_the_tall_sheets_tail` pins
  absolute positions, but they were hand-computed FROM the skyline algorithm
  (bin 100, PAD 2: Tall 40×80 → (2,2); Wide 54×20 → (44,2); Small 30×10 →
  (44,24); height 84) — re-verified by trace in this wave, correct.
- `assets.rs::item_icon_uvs_are_normalized_and_atlas_fits` — normalized-range
  + height-bound assertions only (no absolute UVs).
- `assets.rs::the_full_gore_and_mount_sheet_set_fits_the_strict_gpu_bound` —
  height bound + addressability only. NOTE: E-4b landed after this test was
  written, so its "simulated E-4b additions" now double-enumerate the gore
  set; the name dedup makes the second copy a no-op (first wins, identical
  pixels) — the test stays correct, the comment is historical.
- `projectile.rs::the_batch_layout_matches_the_csharp_draw_order` — pins draw
  ORDER, not atlas addresses (re-pinned in its own wave).
- No test reads a golden PNG or asserts a screenshot SHA256 in Rust — the
  goldens are compared out-of-band (`tools/pixel_diff.py`, recorded hashes).

## Determinism argument (why the re-pin is stable)

The layout is a pure function of the registration list: first-occurrence name
dedup (HashSet used only as a membership filter over the ordered input — no
HashMap iteration feeds placement), a **stable** sort (height desc, width
desc; ties keep registration order), then `skyline_pack`'s fixed left-to-right
node scan with the integer key `(y, waste, x)` — no floats, no randomness, no
map iteration. `build` delegates to `build_ref`, so the owned and borrowed
(world-swap) paths pack bit-identically. The only input that changes the
layout is the sheet list itself (content enumeration order is sorted/deduped,
`test_DW_4_1_enumeration_sorted_deduped_deterministic`).