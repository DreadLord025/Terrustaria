# 78 — W4-g78 SIGHT-RETIRE seal (the sight family closed; the retire hunks)

> W4 lane g78 · sight-retire (code-only RESUME) · 2026-11-21 · base
> `82bc71ea8` (research/77, the wiring attach contracts) · no
> cargo/tests/rustfmt/graphify. This document SEALS the sight family:
> §1 the c42 wiring-seal (research/77 §3.1 CONSUMED at `f173fb88d`),
> §2 the ObjectDrop centre-gap FIX-ONCE (core LANDED here, the contested
> drains' one-fix hunk locations + lane ownership), §3 the held-3002
> attach hunks (the counter+scan half LANDED here), §4 the d9/d13/d56
> keep-one dedupe (the EXACT delete line-ranges), §5 the sight-family
> bible (every sight mechanic → its final owner + status).

## §1 — the c42 wiring-seal (research/77 §3 CONSUMED / REMAINING)

`origin/task/W4-c42-wiring-device-arms` `f173fb88d` ("final seams 2 (g78
contested-seam request)") landed research/77 **§3.1 in full**, with three
name/shape deltas the merger must use (c42's names WIN — they are landed):

| research/77 §3.1 hunk | c42 landed shape | Delta |
| --- | --- | --- |
| `TileInteraction::MonolithSwitch { x, y }` | `TileInteraction::MonolithToggle { x, y }` (`tr-core/src/edit.rs`, the nine-kind literal arm BEFORE the registry-def lookup — the dial precedent) | RENAME only; the nine kinds 410/480/509/657/658/720/721/725/733 verbatim |
| `app.wiring.switch_monolith_use(…)` | `Wiring::use_switch_monolith(&mut self, world, i, j)` (`tr-core/src/wiring.rs`) — decodes kind/frameX/frameY, calls the kept private `switch_monolith` (e21's `fdbf37d9` nine-kind gate) | RENAME only; the `running == false` SkipWire-inert contract kept |
| the doors.rs arm: Mech-28 + grapple-cooldown comment | the arm plays `play_switch_click` on UpdatePlayers + `p.gamepad_enable_grapple_cooldown()` — c42 ported the REAL `Player.quick_grapple_cooldown` (field/setter/Update decrement + the map-open re-arm `Player.cs:24439-24448`) | research/77's "no gamepad layer (recorded deviation)" note is SUPERSEDED — c42 closed it |

The `interaction_events` mapping carries `MonolithToggle { .. } => (None,
Vec::new())` (consumed in `interact_tile` ahead of the mapping, like
`HitSwitch`). c42's later tip `15810df4` is witness fixes only (the
plate-break trapdoor fixture, `Option::as_ref`, an unused import) — no
further §3 consumption.

**§3.2 (the 657 Check2xX break check) is NOT consumed** — c42's commit
message explicitly keeps it "the break-lane contract (research/76 co-lane
list)". The `check_monolith` fn + the `check_object` dispatch arm +
the witness contract stay research/77 §3.2/§3.3 VERBATIM for the break
lane (c31/f41/g17/c14/c29/g42/g45/g46/g85/f25/c36… — the `edit/kill.rs`
contest list). Its drop rides the §2 FIX-ONCE (the origin-cell
`(ox*16, j*16)` 32×32 rect → centre `(ox*16+16, j*16+16)`).

## §2 — the ObjectDrop 32×32-centre-drop-gap FIX-ONCE

### §2.1 — the C# centre math (verified this pass)

Every `Check*` routine spawns through the OBSOLETE rect overload
(`Item.cs:49649-49652`):

```csharp
return Item.NewItem(source, new Vector2(X + Width / 2, Y + Height / 2), …);
```

The centre overload sets `worldItem.Center = center` (`Item.cs:49736`);
`Entity.Center`'s setter is `position = value − (width/2, height/2)`
(`Entity.cs:56-59`) and the `WorldItem` box is **16×16**
(`WorldItem.cs:138`: `base.width = (base.height = 16)`). So a 32×32 rect
at tile `(x, y)` lands the C# item's top-left at `(x*16+8, y*16+8)` —
the box CENTRE at `(x*16+16, y*16+16)`. The spawn VELOCITY is
box-independent (`Next(-30,31)*0.1` / `Next(-40,-15)*0.1`,
`Item.cs:49749-49750`).

The port fed the tile TOP-LEFT (`spawn_item_drop`, `items.rs:745`'s
`tx*TILE_PX` + `ItemDrop::new`'s `pos.x + 1.0`) — the port's 14×14 box
(`item.rs:548` `SIZE = 14.0`) centred at `(x*16+8, y*16+7)`: **8 px left /
9 px up** of the C# box centre (the "~15 px" of research/77 §3.2 is the
top-left-to-centre distance; the box-centre delta is (8, 9)). The gap is
PRE-EXISTING and shared by EVERY `ObjectDrop` consumer — NOT
monolith-specific.

### §2.2 — the rect census (which lane owns the core: g78, LANDED)

The per-producer C# rect (every `ObjectDrop` producer in
`tr-core/src/edit/kill.rs` + the door interactions):

| Producer (port fn) | Kind(s) | C# spawn | Rect |
| --- | --- | --- | --- |
| `check_orb` (:1096) | 12/31/639 | `CheckOrb` `:54229-54288` | 32×32 |
| `check_piggy` (:1191) | 29 | `Check2x1` `:47232-47235` | 32×32 |
| `check_lamp` (:1232) | 93 | `Check1xX` `:37864` | 32×32 |
| `check_clock` (:1283) | 104 | `Check2xX` `:38154` | 32×32 |
| `check_bookcase` (:1345) | 101 | `Check3x4` `:50141` | 32×32 |
| `check_lantern` (:1403) | 42 | `Check1x2Top` `:46790` | 32×32 |
| `check_bed_bathtub` (:1458) | 79/90 | `Check4x2` `:47486/:47494` | 32×32 |
| `check_candelabra_sink` (:1524) | 100/172 | `Check2x2` default `:48750` | 32×32 |
| `check_3x3_bottom` (:1593) | 219/642 | `Check3x3` `:53171`-area | 32×32 |
| `check_chandelier` (:1657) | 34 | `CheckChand` `:52915` | 32×32 (the coin arms' 52×52-at-(−10,−10) `:52913/:52929` centre at +16 anyway — `PileLootEvent`, not `ObjectDrop`) |
| `check_door_closed/open` (:1751/:1786) | 10/11 | `DropDoorItem` `:69409-69416` | **16×16** |
| `check_3x2` (:1850) | 14/87/88 | `Check3x2` `:49434-49439` | 32×32 |
| `check_chair` (:2013) | 15 | `Check1x2` `:43494` | 32×32 |
| `check_bench` (:2053) | 18 | `Check2x1` `:47130` | 32×32 |
| `check_chest` (:2093) | 21 | `CheckChest` `:57991` | 32×32 |
| `check_sunflower` (:2183) | 27 | `CheckSunflower` `:53787` | 32×32 |
| `check_on_table_1x1` (:2615) | 13/33/49/78 | `CheckOnTable1x1` `:43503-43546` spawns NOTHING itself — the drop rides `KillTile`'s per-cell `:63955` | **16×16** |
| the door interactions (`edit.rs:457-477`) | 10 (+cascade) | `OpenDoor`/`CloseDoor`/`SmashDoor` `:86415/:86508` → `DropDoorItem` | **16×16** |

**The core (g78 owns it — LANDED this pass, uncontested):**

* `ObjectDrop::spawn_centre_px(&self) -> (f32, f32)`
  (`tr-core/src/edit/kill.rs`, after the struct): `half = 8.0` for
  10/11/13/33/49/78, else `16.0`; returns
  `(x*16 + half, y*16 + half)`. The struct window's census: only g34
  touches `kill.rs` :63+ (the `HerbEvent` region — a clean union with
  this :34 insertion).
* `spawn_item_drop_centre(items, rng, kind, count, centre)`
  (`world_sim/items.rs`, beside the px entry): spawns the 14×14 box at
  `centre − SIZE/2` through `spawn_item_drop_px` — the draws are
  byte-identical (only the position differs). The items.rs spawn-family
  window (:724-820) census: ZERO lanes edit it (d79/e19/f47/f49/g15
  insert AROUND it; f39 edits :1104 `spawn_player_drop`; g56 :546/:591).
* WIRED at the ONE uncontested consumer: the actuation drain
  (`world_sim/wiring.rs:351` — every wiring.rs lane inserts around the
  loop, none edits its body). The seeded-stream contract unchanged.
* 3 witnesses WRITTEN NOT RUN: the rect→centre table (kill.rs), the
  centre box placement + draw-stream twin (items.rs), the actuated
  bench-pair pop centres at (176, 336) (wiring.rs).

### §2.3 — the contested drains (the one-fix hunk locations)

The fix is ONE conversion (`spawn_centre_px`) + ONE spawn entry
(`spawn_item_drop_centre`) — both landed above. Each remaining drain
swaps its object-drop arm ONLY (the per-cell/pot/tree/wall events keep
the tile feed — their 16×16-family gap is the separate residual, §2.4):

**(a) `editing.rs::break_drop_events` (:5909-5917) + the `break_tile`
drain (:5678-5685)** — CONTESTED: g85 inserts the orb's Musket-Ball pair
INSIDE the loop (after the push), g48 makes the fn `pub(crate)` + copies
the drain into its net-apply arm (its :5846 call + the copied
`for (id, count, x, y)` loop), f32/g46/g47/g48 edit `DropCtx`/`drop_ctx`
below. The hunk: tag the object events' provenance (a parallel
`Vec<(f32, f32)>` of centres pushed beside each object push — g85's
ball pair pushes the SAME centre, the C# spawns both at the same rect),
then the drain routes tagged events through `spawn_item_drop_centre`.
The g48 COPY must take the same shape (its stream is `rngs.global()`,
unchanged). The comment marker is landed at :5905-5908.

**(b) `doors.rs::interaction_events` (:451-455) + the drain
(:383-388)** — CONTESTED: d79 rewrites the cue block above (:379-380),
e10/g24 insert after :377/:393, c28 after :360; c42/e10/e28/f34/f41 add
match arms in `interaction_events`. The hunk: the map's tuple gains the
centre (`(id, cx, cy)` from `d.spawn_centre_px()` — every door drop is
16×16, half 8), the drain calls `spawn_item_drop_centre`. The
`test_DW_4_1` expectations change `(25, 10, 18)` → `(25, 168.0, 296.0)`
(the door's 16×16) and `(32, 10, 17)` → `(32, 176.0, 288.0)` (the
fixture's kind 14 is a TABLE — 32×32, half 16).

**(c) `recall_teleport.rs::spawn_force_clear_area` (:425-437) +
`editing.rs::spawn_force_clear_area` (:6933-6948)** — CONTESTED: g83
widens `square_tile_frame_default`'s signature (adds the UpdatePlayers
stream param) at the SAME call; c46/f37/g58/g73 edit `spawn_recall`
below. The hunk: `spawn_item_drop(…, d.x, d.y)` →
`spawn_item_drop_centre(…, Vec2::from(d.spawn_centre_px()))` in both
copies (the two are the same C# `Player.Spawn_ForceClearArea` — the
merge should keep ONE, the recall lane's call).

### §2.4 — the residual (NOT this fix)

The per-cell `KillTile` drops (`:63955` 16×16), the pot loot
(`SpawnThingsFromPot` 16×16), the tree shakes (`:63225` 16×16) and the
wall breaks (`KillWall_DropItems` `:60448` 16×16) ALSO ride the obsolete
rect overload — their C# centre is `(x*16+8, y*16+8)` vs the port's
top-left feed. That is the SAME class of gap on the 16×16 family, but
those events flow through the shared `(id, count, x, y)` tuple with the
tile drops (whose C# `Player.DropItem`-family spawns are centre-based
too) — fixing them is the item-spawn lane's whole-feed audit, NOT the
ObjectDrop one-fix. Recorded here so the merger does not "fix" the
ObjectDrop drains twice.

## §3 — the held-3002 attach (the counter+scan half LANDED)

`SpelunkerProjectileHelper::held_3002_tick(&mut self, world, center,
spelunker_timer: &mut u8, rng, out)` landed in `spelunker_helper.rs`
(g78's own module — uncontested): the `Player.cs:50094-50098` counter
wrap (`wrapping_add` = the C# `byte++`; the `>= 10` reset + ONE
`add_spot_to_check(base.Center)` on the SHARED instance). NO distance
gate, NO netMode gate. 3 witnesses WRITTEN NOT RUN (the ten-wrap
cadence, the shared dedupe + shared clear, the byte wrap 255→0).

**The caller hunks (CONTESTED — the merger applies):**

**(a) the `Player` field** (`player.rs`, the `:810` C# field): add
`pub spelunker_timer: u8` (init 0; NEVER reset in `ResetEffects` — the
C# has no reset there; `:50094-50097` is its only writer). player.rs is
touched by 53 branches but the field-block insertions union trivially
(the c42 `quick_grapple_cooldown` precedent — same shape, same pass).

**(b) the tick call** (`world_sim/player.rs`, the `take_sticky_dust`
dust-drain cluster :1366): CONTESTED — 18 lanes edit the :1280-1450
window (c14/d79/e10 rewrite :1355; c42/f11 insert :1322+; g83 edits
:1364; c28/d130/e1/e4/f42/f50/g17/g67 insert :1394+; c46 :1379; f32
:1424). The hunk (after the held-light emit decision, the C# runs it
inside `ItemCheck_EmitHeldItemLight` — the port's light pass is
draw-side, so the tick rides the player's fixed tick behind the SAME
gates):

```rust
    // W4-g78 — `Player.cs:50081-50099`: the held 3002's ore-spot counter.
    // The gates: `sItem.type == 3002 && !pulley` (:50081) + the ItemCheck
    // pair `!JustDroppedAnItem && IsAllowedToHoldItems` (:42847). The
    // helper is the POOL's shared instance (§1.1 of research/77) — the
    // player tick runs BEFORE the projectile pass (Main.cs:17754 vs
    // :17775), so the Next(4) draws ride UpdatePlayers here.
    if app.inventory[app.selected].kind == 3002
        && !p.pulley
        && !p.just_dropped_an_item
    {
        let (cx, cy) = p.center();
        let mut out = Vec::new();
        app.projectiles.spelunker.held_3002_tick(
            &app.world,
            glam::Vec2::new(cx, cy),
            &mut p.spelunker_timer,
            app.rngs.stream(crate::app::RngKey::UpdatePlayers),
            &mut out,
        );
        // the dust drain: the take_sticky_dust precedent — the spawn's
        // internal rolls draw on UpdatePlayers at THIS drain (the
        // S1-E-21c deferred-spawn deviation, research/77 §1.2).
        for d in out { /* new_dust_direct + fadeIn/velocity/noLight */ }
    }
```

(`IsAllowedToHoldItems` — the port's `ItemCheck` gate already folds it;
if the tick site sits behind the same gate, drop the third clause. The
borrow: `p` is the destructured player binding — the helper call needs
`app.projectiles` + `app.world` + `app.rngs` as siblings; the
`take_sticky_dust` cluster already solved this shape.)

The pool fields + the pass head/tail + the 473/1008 dispatch swaps stay
research/77 §1.1 VERBATIM (contested by d9/d13/d16/f5/g11/g16/g63/g64/
g65) — the held tick depends on the SAME `spelunker` pool field.

## §4 — the IsTileSpelunkable keep-one (the EXACT delete line-ranges)

The canonical copy: **d9's `projectile_ai_style90.rs`** at
`origin/task/W4-d9-proj-ai-81-90` (tip `7a418c636`): `pub static
TILE_SPELUNKER: TileSet` :69-82 (the 12 words, the doc :64-68) + `pub fn
is_tile_spelunkable` :86-99 (the bitset + the 185/186 windows). Both
`pub` — the re-export needs no visibility change.

**(1) g78's copy deletes** (`spelunker_helper.rs`, THIS branch): the
`static TILE_SPELUNKER` :65-78 + `const TILE_SPELUNKER_IDS` :82-86 +
`fn is_tile_spelunkable` :94-105 replace with:

```rust
pub use crate::projectile_ai_style90::{TILE_SPELUNKER, is_tile_spelunkable};
```

The `TILE_SPELUNKER_IDS` census list moves into the census witness (or
the witness keeps a local copy — it is test-only). The `check_spot`
call site is unchanged. NOTE: d9's module does not exist on THIS branch
(both `projectile_ai_style90.rs` and `projectile_ai_pets124.rs` are
branch-local files) — the re-export lands ONLY at the union merge.

**(2) d13's inline table deletes** (`projectile_ai_pets124.rs` at
`origin/task/W4-d13-proj-ai-121-130`, tip `462c8609`): **lines 59-80**
(the doc comment :59-62 + `fn is_tile_spelunkable` :63-80, the inline
`&[u16]` 49-id table) replace with:

```rust
use crate::projectile_ai_style90::is_tile_spelunkable;
```

The call site :583 (`t.active() && is_tile_spelunkable(t.kind,
t.frame_x, t.frame_y)`) is unchanged. **The g64 union note:** g64's
branch (`origin/task/W4-g64-proj-trail-dust-2`) carries its OWN whole
`projectile_ai_pets124.rs` (the file is new on both branches — the diff
is `@@ -0,0 +1,989 @@`); its copy's fn is at **:57-78** (the doc :57-60,
the fn :61-78 — the 2-line offset is g64's shorter header). Whichever
copy survives the d13/g64 file-union takes the SAME delete (the fn body
is byte-identical). No test in either copy references the fn by name
beyond the :583 call site (the :825 doc mention is prose).

**(3) d56's tr-render copy — the crate-boundary EXCEPTION (stays):**
`tr_render::animated_tiles` at `origin/task/W4-d56-tile-draw-animated`
(tip `5d80b38a0`): `const TILE_SPELUNKER: [u16; 49]` :68-72 + `pub fn
is_tile_spelunkable` :165-176. tr-render's Cargo.toml has NO tr-core /
tr-app dependency (verified: anyhow/bytemuck/log/wgpu/font8x8/winit
only) — it CANNOT reach d9's bitset. The copy stays (the
`SWAYS_IN_WIND_BASIC` hand-transcription precedent); moving the words
to a shared tr-core module would ADD a tr-render→tr-core dependency —
the merger's call, NOT required. The equivalence proof is g78's
`w4_g78_the_spelunker_table_census` (bitset == 49-id-list for EVERY id
in 0..TILE_COUNT) + d56's own :799-820 window witnesses — the id lists
are byte-identical across all four copies (verified this pass).

The 185/186 frame windows are IDENTICAL in all four copies (`frameY ==
18 && frameX in 576..=882` / `frameX in 864..=1170` inclusive,
`Main.cs:21789-21796`) — the dedupe is lossless.

## §5 — the sight-family bible (the final owner + status of every mechanic)

| Mechanic | C# anchor | Final owner | Status at this seal |
| --- | --- | --- | --- |
| Spelunker flag 9 (`findTreasure`) | `Player.cs:2786/:9981/:18370` | player.rs (landed pre-W4) | PORTED+WIRED |
| Hunter flag 17 (`detectCreature`) | `:2792/:10021/:18380` | player.rs | PORTED+WIRED |
| Dangersense flag 111 (`dangerSense`) | `:1132/:10189/:18496` | buff_effects.rs accumulator; **f12** owns the live-field fold | PORTED-UNWIRED (f12's lane) |
| Biome Sight flag 343 (`biomeSight`) | `:2788/:9985/:18371` | player.rs | PORTED+WIRED |
| The 4 potion items (296/304/2329/5211) | `Item.cs:4914/:5026/:23373/:41132` | itemdata.rs + potions.rs (S1-P-i37) | LANDED pre-W4 |
| The 3 vision floors + dust (DrawSingleTile / AdjustForVisionChangers) | `TileDrawing.cs:897-963/:8481-8552` | **d56** (`tr_render::animated_tiles`) | PORTED-UNWIRED (d56's tiles.rs merge) |
| `IsTileDangerous` | `TileDrawing.cs:4435-4467` | d56 | PORTED-UNWIRED |
| `IsTileSpelunkable` + the 49-id table | `Main.cs:21783-21798/:9420-9467/:9651` | **d9 canonical** (bitset); g78 re-export, d13 delete, d56 exception (§4) | 4 copies today → 2 at the merges (d9 + d56) |
| `IsTileBiomeSightable` | `Main.cs:21800-21818` | d56 | PORTED-UNWIRED |
| SpelunkerProjectileHelper (the shared instance) | `SpelunkerProjectileHelper.cs:6-67`, `Main.cs:2233` | **g78** (`spelunker_helper.rs`) | PORTED; dispatch attach = research/77 §1.1 (contested) |
| the 473 glowstick head attach | `Projectile.cs:24836-24847` | g78's `glowstick_head_inner` (attach-ready) | LANDED here; the dispatch swap = 77 §1.1(c) |
| the 1008 flare stuck-arm attach | `Projectile.cs:27124-27127` | g78's `flare_step` split (attach-ready) | LANDED here; the swap = 77 §1.1(d) |
| the HELD 3002 `spelunkerTimer` | `Player.cs:810/:50094-50099` | g78's `held_3002_tick` (attach-ready, THIS pass) | LANDED here; the caller = §3 (contested) |
| the caller distance gate | `Projectile.cs:24842/:27124` | g78's `spot_in_range` | LANDED (7254154a3) |
| the 14-arm NPC debuff tint + the black-out scan | `Main.cs:22922-22938/:22939`, `NPC.cs:91024-91141` | **g78** (`npc_hunter_tint.rs` → `render/entities.rs` colour site) | WIRED (3f1e13855/cad4ce3be) |
| the G6 npc_key tint-widen | the render_key free fn | g78 (`tint_key`) | WIRED (3f1e13855) |
| the Hunter floor (`hunter_active`/`hunter_friendly`) | `NPC.cs:91110-91139/:91019-91022` | g78's fold; the threading = 77 §2 (c39/d70-contested); the friendly predicate stays **f47**'s `shimmer::appears_friendly_to_hunter_potion` (the 689 override `NPCID.cs:10128-10132`) | PORTED; inputs false until 77 §2 lands |
| the draw-order swap (hunter-lit NPCs in front of solid tiles) | `Main.cs:63290-63299` | 77 §2(e) — ONE unit with the threading | PUBLISHED (contested) |
| the Echo-sight chain (`ShouldShowInvisibleBlocksAndWalls` / `CanSeeInvisibleBlocks`) | `Main.cs:58023-58030`, `Player.cs:18742/:14548-14550/:14215-14217` | **g78** (`scene::should_show_invisible_blocks_and_walls`) | WIRED (db4794b36) |
| the 657 monolith lit-row scan (frameY≥54) | `SceneMetrics` | g78 | WIRED (db4794b36) |
| Spectre Goggles 4409 (functional-only grant) | `Player.cs:14548-14550/:12946-12952` | g78 | WIRED; the vanity refusal pinned (1d67338ec) |
| `IsVisible` / `tile_is_visible` / wall_visible coating bit | `TileDrawing.cs:1411-1432`, `WallDrawing.cs:91/:258-261/:100-103` | g78 (`scene::tile_is_visible`) | WIRED (db4794b36) |
| `WallLightAt` | `Main.cs:55169-55181` | g78 (`scene::wall_light_at`) | PORTED-UNWIRED (g70's bg.rs consumer) |
| the 657 right-click use arm | `Player.cs:32271-32277` | **c42** (`MonolithToggle` + `use_switch_monolith`, f173fb88d) | CONSUMED (§1) — e21's nine-kind gate is the single flip source |
| the 657 Check2xX break check + the per-kind drop table | `WorldGen.cs:37993-38287/:85738` | the break lane (77 §3.2 contract) | PUBLISHED (contested) |
| the ObjectDrop centre gap | `Item.cs:49649-49652` | **g78** core (§2.2 LANDED); the drains = §2.3 hunks | CORE WIRED (1 drain); 3 drains published |
| the 5345 vanity-monolith grant (the mirror) | `Player.cs:14215-14217` | g78 | WIRED (the vanity-only grant, db4794b36) |

**The retire verdict:** every sight mechanic has a named owner and a
landed-or-published hunk. g78's remaining merge dependencies are exactly
research/77 §1.1 (the spelunker dispatch), §2 (the Hunter threading),
§3.2 (the break check), §4 (the dedupe) + this doc's §2.3 (the drains)
and §3 (the held caller). NOTHING in the sight family is unowned.

## Witnesses (WRITTEN NOT RUN — code-only)

This pass adds 6: `spelunker_helper.rs` 3 (the held counter cadence, the
shared dedupe/clear, the byte wrap), `edit/kill.rs` 1 (the rect→centre
table), `items.rs` 1 (the centre placement + draw twin), `wiring.rs` 1
(the actuation drain centre). The §2.3/§3 hunk witnesses are the
merger's (the contracts above pin the expected values).