# AGENTS.md — working rules for this repository

Terraria-Rust is a **1:1 reimplementation of Terraria 1.4.5.8** in Rust: same behaviour,
same constants, same pixels, different engine. These rules are mandatory for every change.

## The golden rule: port the C# first

Before writing or changing **any** game logic, function or constant:

1. Find the matching implementation in the decompiled sources under `decompiled/`
   (`Terraria.sln` namespaces: `Terraria`, `Terraria.GameContent.*`, `Terraria.ID`, …).
   Data tables (tile ids, frame counts, merge sets, colours) also come from there.
2. Read the whole relevant function and its callers. Note constants, update order,
   rounding (`Math.Round` is banker's rounding), RNG order, and special cases.
3. Port it **faithfully** to Rust first — same structure, same maths, same order.
   Name it after the original (`Liquid.Update`, `DrawBasicTile`, `AI_003_Fighters`, …)
   and reference it in a one-line comment when the mapping is not obvious.
4. Verify the port against the source: unit-test the maths, compare screenshots with the
   original assets, compare an original `.wld` where possible.
5. Only after steps 1–4 may you optimise. Optimisations must produce identical output
   (pixel-identical screenshots, identical state) and be measured with `TR_BENCH` /
   `TR_STATS`.

If you are unsure about a behaviour, **search the C# source again**; never guess, never
invent a plausible-looking shortcut. If something is deliberately approximate, write it
down in `docs/ANALYSIS.md` and `docs/ROADMAP.md`.

## Assets

- `decompiled/`, `decompiled-libs/` and `assets/Images/` are derived from the user's own
  retail copy.

## Rendering invariants

- Tiles are 16 px on an 18 px sheet grid (2 px padding); tree trunks are 20×20 on a 22 px
  grid; walls use 36 px cells with an 8 px inset. Keep source rects on those grids.
- The renderer draws one atlas through one instanced sprite pipeline. Sprites carry an
  optional rotation; tint is pre-linearised in WGSL to match XNA byte multiplies.
- Lighting uses the original `TileLightScanner` seeds and `LightMap` blur; the 9-slice
  block path (`GetColor9Slice`) is what makes Color/White mode smooth.
- Optimisation policy (Milestone 1.5): the in-game zoom range stays 1.0–2.0; **no LOD,
  no reduced detail, no skipped frames of world state**. Any cached range must key on
  every input that affects its pixels.

## Verification before committing

- `cargo test --workspace` must pass.
- For visual work, take deterministic screenshots (`TR_SHOT`, `TR_CAM`, `TR_DAY`,
  `TR_NOPLAYER`, `TR_DEBUG=0`) and compare with `tools/pixel_diff.py`. Check zoom 1.0,
  1.5 and 2.0 when background/parallax changes.
- For performance work, A/B against the previous binary with `TR_BENCH=600` and
  `TR_STATS=1`; include before/after numbers.
- Update `README.md`, `docs/ARCHITECTURE.md`, `docs/ANALYSIS.md` and `docs/ROADMAP.md`
  when a feature lands. Commit one roadmap item per commit with an imperative message.

## Housekeeping

- Do not run `cargo fmt` on the generated `crates/tr-core/src/tiledata.rs` or `wld.rs`;
  use `rustfmt` on the touched files only.
- `assets/test/`, `target/`, `decompiled/`, `assets/Images/` and `.qoder_tmp/` are local.
- Prefer extending the existing systems (world model, `framing`, `lighting`, `LiquidSystem`,
  `Rain`, `Projectile`, `Npc`) over adding parallel implementations.
