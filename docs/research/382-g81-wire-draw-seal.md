# 382 — W4-g81 WIRE-DRAW SEAL: the compose verdicts + the junction-box correction + the wire-draw-family bible

> **Chain: 283-self-contested → 382.** This doc was originally self-picked
> as 283 post-§10 (verified free across the 363 origin task tips at the
> time), but the research/150 registry §7.3 RESERVES 283 for f26
> (68-7j-addendum) — that reservation KEEPS. Per the Lead amendment the
> g81 self-pick is contested and renumbers to **382** (registry §9/§10
> micro-rename; slug kept, all references repointed).
>
> W4-g81 · 2026-11-21 · code-only (no cargo/tests/rustfmt/graphify run) ·
> branch `task/W4-g81-wire-draw-overlay`, seal tip `5394f5df3` (the
> pre-seal DONE tip `ebb5308a0`).
>
> Sources re-read at the cited lines in this worktree's `decompiled/`
> junction (direct paths, no recursion). Peer contracts read from
> `git show origin/task/<branch>:<file>` at the sealed tips:
> c42 `69148794c`, f11 `f8d07a546`, d13 `6850a19e5`, g78 `66cbf9983`,
> g86 `47cd14a87`.

## 0. What this lane owns (the wire-draw estate)

`Main.DrawWires` (`Main.cs:58218-58675`) verbatim — the four-colour
`WireNew` masks, the XNA byte tints (incl. alpha truncation), the pre-dim
strip colour, the actuator plate's raw `Lighting.Brightness` gate, the
tile-423 LightSeaGreen range rectangle (`:58660-58672` via
`Utils.DrawRectangle`/`DrawLine`). Plus the four residual visuals OUTSIDE
that region, all landed on this branch:

| Visual | C# | Port |
| --- | --- | --- |
| The WiresRadial (flower + cursor badges + ToolMode machine) | `WiresUI.cs:59-691` | `ui/wires_ui.rs` + `App.wire_tool_mode`/`wires_radial` + the DrawInterface_9 slot |
| The Wire Kite ghost (proj 651) | `Projectile.cs:6990-7000`/`:35697-35833`/`:72051-72067` + `Main.cs:31620-31907` | `projectile_ai_kite.rs` + `render/wire_kite.rs` + the 651 data row + the pool seams + the 3611 use gate/spawn |
| The builder-accessory toggle column | `Main.cs:47353-47590` + `:42518-42596` | `ui/builder_toggles.rs` + `Player.ruler_line/ruler_grid/auto_paint/sign` |
| The held-tool cursor icon + 10-frame cache | `Main.cs:45037-45115`/`:46515-46540`/`:17035` + `Player.cs:46752-46760`/`:45620-45624` | `ui/mod.rs` draw_hud icon pass + `app/mod.rs` cache + `app/frame.rs` clear + `world_sim/editing.rs` latch |

**37 witnesses WRITTEN NOT RUN** (kite 8, ghost 10, radial 14, toggles 5).
The seal added 1 kite witness (the negative-centre floor) + 3 kill-tick
tail pins; the 5 ghost colour-mix witnesses re-derived to the C# alpha.

## 1. The c42 compose (research/95 §8, the entity-boxes / door-echoes lane)

**c42 owns the wiring ARMS; g81 owns the wiring VISUALS. Disjoint.**
c42's `Wiring.cs` full-file census (its research/95 §11) ports every
gameplay-bearing fn; g81 ports zero `Wiring.cs` logic. The two estates
meet at exactly one symbol: `App.wire_tool_mode`.

- **`App.wire_tool_mode`** — BOTH branches add it (c42 fold-133 `05ae5cc6`
  for the 3625 use arm; g81 for the radial's XOR clicks). Same field, same
  Red default (`WiresUI.cs:92-95` static ctor), same two-ctor census
  (`App::new` + `flow_app`). **Keep-one: identical, the merge dedupes the
  field + the two inits.** g81's seal fix 1 added the `flow_app` init the
  pre-seal tip missed (E0063).
- **The radial** — c42 has NO `wires_radial`/`wires_ui` (grep-verified at
  `69148794c`). g81's `ui/wires_ui.rs` is the whole module. No conflict.
- **The mass-op** — see §4. c42 owns the apply; g81's kite queues the
  intent and drops it (the documented deferral). The merger wires g81's
  pool seam to c42's apply via the f11 §8.4b hunk.
- **The entity-boxes / door-echoes** (c42 §8.1/§8.2 closures) — pure
  wiring-tick logic, zero draw-side overlap with g81. No g81 hunk touches
  `Wiring.entity_boxes`/`door_echoes`.

## 2. The f11 compose (research/234 radial / 235 kite / 133 §8 seal)

**g81's `ui/wires_ui.rs` is the radial KEEP-ONE; f11's `ui/wires_radial.rs`
DROPS.** The verdict re-verified at f11 `f8d07a546`:

- f11's module is the LOGIC half only (the Settings gates, the
  flower/line state machines, the six-slot geometry + XOR toggles). g81's
  module is the SUPERSET: the same logic PLUS the `DrawFlower`/
  `DrawCursorArea` sprite passes PLUS the texture loads.
- **f11 ports the DEAD `DrawLine` as a live `line_clicks` interaction**
  (`wires_radial.rs:140` dispatches to it when `line_opacity > 0`). The
  C# `WiresRadial.DrawLine` (`WiresUI.cs:195-340`) has NO caller —
  `Draw` (`:189-193`) calls only `DrawFlower` + `DrawCursorArea`
  (tree-grep verified: zero `.DrawLine(` hits). f11's `line_clicks`
  therefore adds a NON-VANILLA bottom-centre line-slot toggle. **g81
  correctly omits it** (the module doc records the dead-code verdict).
  The keep-one must drop f11's `line_clicks` + its dispatch arm, never
  ship both radials.
- **The texture-wave contract (f11 research/234 §2b) is FULFILLED on
  g81.** f11's census read g81 at `0017d32ab` (the census commit, BEFORE
  the residuals landed) and concluded "NO lane loads the radial UI
  textures". **That verdict is STALE**: g81's residuals load
  `UI/Wires_0..11` + `UI/BuilderIcons` (`assets.rs:1469`/`:1474`, the UI
  block — exactly the slot f11's contract named) and port the draw. g81
  IS the radial-draw executor. The 13-name claim attaches to d35's
  research/113 union as f11's §14 delta form (the measured 20 980 px² ≈ 3
  atlas rows stands; g81's loads are the claimant).
- **The kite** — f11's research/235 verified d13's projectile half; g81
  ports its OWN aiStyle-125 arm (`projectile_ai_kite.rs`) + the ghost
  draw. **The d13 pool seam is the keep-one** (§3): g81's
  `projectile_ai_kite.rs` and d13's `projectile_ai_wire_kite.rs` are
  two ports of the same C# arm — the merger keeps ONE (d13's is the
  pool-integrated shape; g81's ghost draw + use gate/spawn are unique and
  land regardless). The f11 apply (`mass_wire_operation` + `drop_rng`)
  is c42's at the union (§4).

## 3. The d13 compose (the kite pool seam keep-one)

d13 `6850a19e5` ports aiStyle-125 as `projectile_ai_wire_kite.rs`
(`ai_wire_kite`, a `Projectile` method) with the pool seam
(`wire_kite_inputs`/`take_mass_wire_ops`, the 5-tuple `(owner, psx, psy,
pex, pey)`). g81 ports it as `projectile_ai_kite.rs` (a free fn over a
`KiteOwner` snapshot) with `kite_ctx`/`kite_ticks`/`drain_mass_wire_ops`
(the 2-tuple `((psx,psy),(pex,pey))`, owner gated at the queue site).

**Keep-one: d13's pool seam wins** (the pool-integrated method shape, the
5-tuple carrying the owner for the drain gate). g81's unique estate that
lands regardless:
- the ghost DRAW (`render/wire_kite.rs` — d13 has NO ghost draw, its 651
  falls through the generic `DrawProj` sprite path, a WRONG g81 fixes);
- the 3611 use gate (`Player.cs:52063-52066` no-channel refusal) + the
  zero-velocity centre spawn + the `TryUpdateChannel` tail
  (`world_sim/editing.rs` — d13 has none);
- the 651 generic-draw skip (`projectile.rs:10281` — the C#
  `CacheProjDraws` routes drawLayer 5 out of the main walk).

The seal fixed g81's kite to match the C# where the pre-seal tip diverged
(the owner tail runs on the kill tick; the release cell uses `(int)X>>4`).
d13's arm already had both (its census noted them) — the keep-one merge
is a clean dedupe, g81's fixes confirm d13's shape.

## 4. The MassWireOperation-dropped deviation — CONFIRMED c42-owned

**The pre-seal ANALYSIS recorded `Wiring.MassWireOperation` as UNPORTED
(the mass-op lane). The seal CONFIRMS: c42 owns the apply, it is NOT an
open gap.** At c42 `69148794c`:

- `world_sim/wiring.rs:1096` `mass_wire_operation(app, ps, pe, mode)` —
  the 58-slot stock census (530 wire / 849 actuator), the core L-walk
  (`mass_wire_operation_inner`), the ConsumeItem bill (the msg-110 MP
  bill folds to SP), the cut-cache `Item.DropCache` drops at
  `master.Center`. fold-133 `05ae5cc6`.
- `editing/wiring.rs:207` — the 3625 Multicolor Wrench use arm calls it
  with the Actuator-strip mask (`Player.cs:46908`).
- The core `mass_wire_operation_step`/`_inner` (`tr-core/wiring.rs`) —
  fold-133, the L-shaped rectangle walk + the dry-stock halt.

**The merger contract** (f11 research/133 §8.4, re-verified at the sealed
tips):
1. f11 threads `drop_rng: RngKey` onto c42's moved fn (`:1096`, the stream
   use at `:1143`); the 3625 arm passes `UpdatePlayers`, the kite drain
   passes `UpdateProjectiles` (the Kill's stage, `Main.cs:17775`).
2. The kite drain: ONE shape, the direct drain wins (f11 §8.4b). At the
   d13×c42 union, REPLACE d13's buffer park with the direct drain over
   `take_mass_wire_ops()` — RAW ToolMode (`Wiring.cs:3241`, no Actuator
   strip, that mask is the 3625 arm's), no cooldown gate (the Kill hook
   has none), `owner == local_id`, `UpdateProjectiles`. The
   `app.pending_mass_wire_ops` buffer field DROPS.
3. g81's `drain_mass_wire_ops` drop site (`world_sim/projectiles.rs`) is
   the same seam — the merger replaces the drop with the apply call. g81's
   pool seam dedupes to d13's (§3).

**g81's deviation is therefore a DEFERRAL, not a gap**: the ghost preview
+ the anchor/release machine are the ported visuals; the apply lands at
the c42×d13×f11 union. The ANALYSIS + the code notes re-worded
"UNPORTED ON THIS BRANCH" with the compose verdict.

## 5. The junction-box hover highlight — CORRECTED (no such C# exists)

**The pre-seal census called the junction-box highlight the GENERIC
smart-cursor tile-outline subsystem (`GetTileOutlineInfo`/
`InSmartCursorHighlightArea`) and recorded it MISSING. The seal re-read
CORRECTS the attribution:**

- The junction box is tile **424** (`TileID.WirePipe`, `TileID.cs:1477`);
  the Pixel Box is **445** (`:1519`); the LogicSensor is **423** (`:1475`).
- **NONE of 423/424/445 is in `TileID.Sets.HasOutlines`** (`TileID.cs:551`,
  the full 90-id set re-read — the three ids absent). The outline
  subsystem CANNOT highlight them: `GetTileOutlineInfo` is gated on
  `HasOutlines[typeCache]` (`TileDrawing.cs:893-895`).
- **NO 423/424/445 arm in `TileInteractionsMouseOver`**
  (`Player.cs:33587-34743`, the g86 research/244 101-arm site-list — the
  only 42x/44x arms are 441/440). So no cursor-icon hover either.
- **No junction-box hover highlight exists in the C#.** What DOES exist:
  - the tile-423 LogicSensor (style 2 "Above", `frameY == 36`) RANGE
    rectangle — the always-on LightSeaGreen box `DrawWires` draws for
    every such sensor (`Main.cs:58660-58672`), NOT a hover highlight.
    **Ported** on g81 (`render/wires.rs::special_tile_rect`/
    `draw_rectangle`, census commit `6f046998`).
  - the generic outline subsystem itself (the `HasOutlines` tiles: chests
    21/467, doors 10/11, altars 26/237, ...) — **g86 owns it**
    (`render/tile_outlines.rs` + `world_sim/smart_interact.rs`, g86
    `47cd14a87`): the draw pass, the HighlightMask sheets, the
    `SmartInteractLookup` feed, the pylon/void-lens selection glow. The
    g81 "needs its own lane" verdict stands, but the lane EXISTS and has
    landed. The PORT_MAP row re-points to g86, not left MISSING.

**The cursor-icon lane ownership** (the task-seed question): g86's
research/244 is the `cursorItemIcon-170-sites` seed (the full census: 357
writes / 169 ID-writes in Player.cs). **g81 owns the enabled latch + the
10-frame cache + the held-kind draw** (the wiring/mining in-reach arms);
**g86 owns the ID/Text/Push/Reversed override system** (the ~150
tile-hover arms + the layer-40 override draw). The two are disjoint: g81's
`cursor_item_icon_enabled` is the field g86's seed extends. The merge
keeps g81's bodies and ADDS g86's four fields + the override arm (g86
research/244 §4 owner contract). The wiring tools keep ID 0, so g81's
radial `:130` ID gate never trips — no conflict.

## 6. The g78 compose (tile-echo: spelunker / hunter-tint) — DISJOINT

g78 `66cbf9983` (the spelunker ore-spot dust + the NPC debuff-tint chain +
the Hunter floor + the show_invisible wiring + the ObjectDrop centre fix)
and g81 (wire-draw) are **DISJOINT — no shared symbol**. The four shared
files have non-overlapping hunks (verified by `git diff` hunk ranges):

| File | g78 hunks | g81 hunks |
| --- | --- | --- |
| `app/frame/draw.rs` | 362/390/1179/1226 (light params + show_invisible) | 1304/1562/1646/1780 (wires + ghost + radial + toggles + icon) |
| `main.rs` | 159/201 (npc_hunter_tint + spelunker_helper mod rows) | 195 (projectile_ai_kite mod row) |
| `player.rs` | 914/2954/4908/5001/5981/19703/21650 | 1002/2628 (ruler/auto_paint/sign fields) |
| `world_sim/editing.rs` | 5902 (break_drop_events) | 2165/2537/2749 (update_editing) |

g78's render files (`tiles.rs`/`entities.rs`/`pylons.rs`/`bg.rs`/
`vines.rs`) are NOT g81's (`wires.rs`/`wire_kite.rs`/`mod.rs`). The
`render/mod.rs` row: only g81 adds `wire_kite` (g78 doesn't touch it).
**No merge conflict; the two lanes compose cleanly.** g78's research/77
§3 monolith use-arm request was consumed by c42 (its research/280 §2) —
not a g81 concern.

## 7. The seal fixes (six, this branch)

1. **`flow_app` ctor E0063** — the exhaustive test literal
   (`app/flow.rs:2134`, no `..Default` spread) carried NONE of the six new
   App fields; 344 `flow_app` callers would not compile. The six fields
   now mirror `App::new` (the two-literal census). `87f7b159`.
2. **The kite's owner tail runs on the KILL tick** — the C# else-if chain
   (`Projectile.cs:35706-35723`) has NO return after the kill arms;
   `:35825-35832` follows it. The early returns dropped
   heldProj/SetDummyItemTime(2)/itemRotation on the release frame. 3
   kill-tick tail pins added. `87f7b159`.
3. **The release cell + ghost `pe` use `(int)X >> 4`** —
   `Utils.ToTileCoordinates` (`Utils.cs:1807-1810`) casts to int FIRST
   then arithmetic-shifts (floors negatives). 1 new witness. `87f7b159`.
4. **The draw pass's three whole-`self` method calls → disjoint-field
   forms** — the field destructure holds `renderer`/`lookups` mutably;
   `self.slot_at_mouse()`/`self.coin_ammo_at_mouse()`/
   `self.item_icon_cache_verification()` would E0502.
   `slot_at_mouse_fields`/`coin_ammo_at_mouse_fields` free fns in
   `clicks.rs` (the App methods delegate); the cache-verification body
   inlines. `b8c46275`.
5. **The callerless `App::item_icon_cache_verification` drops** — the
   inline is the single body (the C# has one caller, `:45062`).
   `77505fa0`.
6. **The ghost's alpha-255 override REMOVED** — the C# batch is
   `BlendState.AlphaBlend` (premultiplied `One, OneMinusSrcAlpha`), so
   `color47.A = 0` (`Main.cs:31657`) with non-zero RGB is an ADDITIVE term
   (`src + dst`), the visible ghost glow; `color46` keeps alpha 0 normally
   / 255 with the cutter. The port pipeline is the same equation
   (`sprite.wgsl` keeps rgb un-premultiplied, alpha = `tex.a * color.a`).
   The earlier override (a misread of the blend as non-premultiplied, the
   stale `liquids.rs` comment) turned it into a semi-transparent REPLACE.
   The 5 colour-mix witnesses re-derived to the C# alpha (RGB unchanged).
   `5394f5df3`.

The radial's icon_enabled doc-comments drop the stale "stand-in" wording
(the field IS the sim latch since `851a73e1`); the mass-op drop notes
re-point at the compose verdict (§4). `d17a9da7`.

## 8. The WIRE-DRAW-FAMILY bible (one paragraph)

**g81-canonical / c42-arms / f11-radial-drops / d13-pool-keep-one /
g86-outline+icon-override / g78-disjoint:**
`task/W4-g81-wire-draw-overlay` is the wire-DRAW canonical —
`Main.DrawWires` verbatim (the byte tints, the 423 range rect), the
WiresRadial (logic + draw + the `UI/Wires_0..11`/`UI/BuilderIcons` loads,
the f11 research/234 texture-wave executor), the Wire Kite ghost draw +
the 3611 use gate/spawn + the 651 generic-draw skip, the builder-toggle
column, the held-tool cursor icon + cache + the wiring/mining in-reach
latch — **37 dedicated-module witnesses**, ALL WRITTEN NOT RUN. **c42**
owns the wiring ARMS (`Wiring.cs` full census) + the `mass_wire_operation`
apply; the merger threads f11's `drop_rng` param and replaces g81's/d13's
kite-intent drop with the f11 §8.4b direct drain (RAW ToolMode, no
Actuator strip, no cooldown gate, `owner == local_id`,
`UpdateProjectiles`). **f11**'s `ui/wires_radial.rs` DROPS (g81's
`wires_ui.rs` is the superset keep-one; f11's `line_clicks` ports the DEAD
`DrawLine` as a non-vanilla interaction — never ship it). **d13**'s
`projectile_ai_wire_kite.rs` pool seam is the aiStyle-125 keep-one (g81's
`projectile_ai_kite.rs` dedupes to it; g81's ghost draw + use gate/spawn
are unique and land regardless). **g86** owns the smart-cursor
tile-outline subsystem (`tile_outlines.rs` + `smart_interact.rs`, the
`HasOutlines` tiles) + the cursorItemIcon ID/Text/Push/Reversed override
system (research/244); g81's enabled latch + cache + held-kind draw are
the disjoint base it extends. **g78** (tile-echo: spelunker/hunter-tint/
show_invisible/ObjectDrop) is DISJOINT — no shared symbol, the four
shared files have non-overlapping hunks. **The junction-box hover
highlight does NOT exist in the C#** (423/424/445 absent from
`HasOutlines` + no MouseOver arm); the 423 range rect is the always-on
`DrawWires` box (ported). **Gate-command** (the merger runs it; this lane
never did): `cargo test --workspace --skip
migrates_version_2_kinds_to_item_ids` — judge by the `test result: ok`
lines (cargo may exit 1 on success); canary `anchors.txt` (the g81 rows:
`wire_tint`, `ai_wire_kite`, `ghost_path`, `draw_tool_mode_ui`,
`draw_builder_acc_toggles`) via `tr-core/build.rs` + `cargo test -p tr-app
--test main_anchors`.

## 9. Verification record (this seal, code-only)

- The compose verdicts: `git show`/`grep` at the five sealed peer tips
  (c42 `69148794c`, f11 `f8d07a546`, d13 `6850a19e5`, g78 `66cbf9983`,
  g86 `47cd14a87`) — the mass-op fn signature, the radial module absence
  on c42, the d13 pool seam, the g86 outline/icon modules, the g78 hunk
  ranges.
- The junction-box correction: `TileID.cs:551` (the full HasOutlines set),
  `TileDrawing.cs:893-895` (the gate), `Player.cs:33587-34743` (the
  MouseOver arm list) — direct reads.
- The ghost alpha: `Main.cs:31626-31657` (the colour mix + the A=0 reset),
  `:38213`/`:62294` (the AlphaBlend batch), `sprite.wgsl:296-314` (the
  port's premultiplied equation) — direct reads.
- The kite fixes: `Projectile.cs:35697-35833` (the else-if chain, no
  return), `Utils.cs:1807-1810` (ToTileCoordinates) — direct reads.
- The borrow fixes: the base `draw.rs` destructure (the only self-method
  call sits outside the block) — `git show origin/main`.
- **No cargo/rustc/rustfmt/tests/graphify invoked.** The 37 witnesses are
  WRITTEN NOT RUN; the colour-mix maths re-verified by hand (the RGB bytes
  unchanged from the pre-seal python f32 emulation, only the alpha byte
  re-derived to the C# value).
