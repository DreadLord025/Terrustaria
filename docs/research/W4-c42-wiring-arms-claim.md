# W4-c42 — HitWireSingle missing-arms lane (claim doc)

> Code-only lane. Branch `task/W4-c42-wiring-device-arms`. Witnesses are
> WRITTEN NOT RUN (no cargo in this mode). Sources: `decompiled/Terraria/Wiring.cs`,
> `WorldGen.cs`, `Minecart.cs`, `Tile.cs`, `Sign.cs`, `Collision.cs` (1.4.5.8).

## Census — what is actually missing from `hit_wire_single` (re-read 2026-11-21)

The research-23 D8 closure list ("40 tile-mechanical arms restored") is
over-broad for two families it counted by name only. Verified against the live
`tr-core/src/wiring.rs` (de176ccd base):

| Arm | C# | Port state at claim time |
| --- | --- | --- |
| Campfire 215 (`TileID.Sets.Campfires` = {215}, `TileID.cs:616`) → `ToggleCampFire` | `Wiring.cs:1418-1421`, `:3000-3042` | **MISSING** (the D8 note claimed "the light family incl. the Toggle* helpers"; `ToggleCampFire` has no port — the 565 arm is a different tile, the campfire-alternative block) |
| Torch 4 (`TileID.Sets.Torches` = {4}, `TileID.cs:615`) → `ToggleTorch` | `Wiring.cs:1575-1579`, `:2903-2918` | **MISSING** (frameX ±66) |
| Active stone block 130 → 131 | `Wiring.cs:1423-1432` | **MISSING** (the above-cell `PreventsActuationUnder` + `CanKillTile` gate, swap + `SquareTileFrame`) |
| Inactive stone block 131 → 130 | `Wiring.cs:1433-1439` | **MISSING** (unconditional swap + wake) |
| Trapdoor pair 386/387 → `ShiftTrapdoor` | `Wiring.cs:1440-1453`, `WorldGen.cs:51447-51596` | **MISSING** (`ShiftTrapdoor` unported anywhere) |
| Tall gate pair 388/389 → `ShiftTallGate` | `Wiring.cs:1454-1460`, `WorldGen.cs:51723-51775` | **MISSING** (`ShiftTallGate` unported; `npc_ai_town.rs:742` seam comment confirms) |
| Open door 11 → `CloseDoor(forced: true)` | `Wiring.cs:1461-1468` | **MISSING** (`edit::close_door` exists, no wiring caller) |
| Closed door 10 → `Main.rand.Next(2)` + `OpenDoor` both directions | `Wiring.cs:1469-1488` | **MISSING** (`edit::open_door` exists, no wiring caller) |
| Hopper 21/467 → `Wiring.Hopper` | `Wiring.cs:1986-1989`, `:2638-2678` | **DEFERRED** — needs `WorldItem` hitboxes/reservation + `TryToPutItemInChest` coin arithmetic (`:2680-2799`) + `ItemSorting.SortInventory`; the app item store is `ItemDrop` (no reservation field) |
| Extractinator 219/642 → `Wiring.Extractinator` | `Wiring.cs:1990-1993`, `:2554-2608` | **DEFERRED** — the roll table IS ported (`extract.rs::roll_extractinator_drop`), but the arm needs the app chest store (`Chest.UsingChest` gate) and `world.chests` vs `app.chests` reconciliation |
| Switch track 314 → `CheckMech(5)` + `Minecart.FlipSwitchTrack` | `Wiring.cs:1744-1749`, `Minecart.cs:1296-1319` | **DEFERRED** — the type-0 front/back swap is portable, but the type-2 booster arm rides `FrameTrack(pound: true)` (`Minecart.cs:943-1160`), whose `_trackSwitchOptions` neighbour machinery is unported (`minecart.rs` module doc) |

## Claim — this lane (c42)

**The first 8 missing arms in C# case order**: 130, 131, 386/387 (one arm),
388/389 (one arm), 11, 10, plus the two set-gated families the C# evaluates
between the switches — `Campfires` (215) after case 212 (`:1418`) and
`Torches` (4) after case 235 (`:1575`). All eight are tile-mechanical: no
entity store, no chest store, no unported subsystem beyond the seams named
below. The dispatch structure keeps the C#'s three-switch shape: the arms
sit in a **pre-check block between the 356/663 dial arm and the 425
announcement arm's switch** — actually the port folds all switches into one
`match`, so the arms are ordered by their C# evaluation position and the
`return` semantics are preserved (every C# `if (type == …) { … return; }`
arm ends the dispatch; the port arm returns from `hit_wire_single`).

Batch plan (5 arms per commit):
1. **Batch 1**: 215 campfire, 4 torch, 130, 131, 11 close-door (5 arms).
2. **Batch 2**: 10 open-door (the rand roll + retry), 386/387 trapdoor,
   388/389 tall gate (3 arms + the `edit::shift_trapdoor`/`shift_tall_gate`
   ports).

## Explicitly NOT claimed here (later lanes' scope — do not touch)

- **W4-e20 (wiring-device-arms-2)**: 21/467 Hopper (needs the WorldItem
  reservation + coin arithmetic), 219/642 Extractinator (app chest store).
- **W4-e21 (wiring-device-arms-3)**: 314 FlipSwitchTrack (needs
  `FrameTrack`/`_trackSwitchOptions`), the `ToggleTorch`-family forced-state
  variants if any consumer appears.
- **W4-f11 (wiring-arms-4)**: the residual `HitSwitch`/`UpdateMech` audits,
  the `SendData(19)` net echoes (Stage-2), any remaining case-order fixes.

## Seams reused (i34 precedent — no re-invention)

- `edit::open_door`/`close_door` (`WorldGen.cs:37630-37784`/`:31805-31909`)
  — the door bodies exist; the wiring arms add only the caller semantics
  (forced close, the rand-direction open + retry).
- `edit::validate_tile_square` (pub(crate), de176ccd) — the trapdoor/gate
  validators.
- `edit::square_tile_frame_default` — the post-swap wake (130/131).
- `edit::can_kill_tile(world, x, y, chest_holds_items)` — the 130 gate and
  the trapdoor close guard (`WorldGen.cs:62219` body).
- `tileobject::{tile_cut, is_drip_tile, def_for_tile, get_tile_data}` — the
  trapdoor sweep predicate and the tall-gate def decode.
- `framing::reframe_area` — the trailing `TileFrame` loops (documented
  radius-superset deviation, same as `open_door`'s).
- The wiring RNG stand-in `Wiring::rand_next` (`Main.rand` at the
  `HitWireSingle` site — the door-direction roll `:1472` joins it, the
  S1-L-6c contract).

## Known seams left open (documented, not invented)

- `Collision.EmptyTile(ignoreTiles: true)` entity half: tr-core has no
  entity store; the trapdoor/gate ports take a `cell_blocked` closure
  exactly like `edit::close_door` (the app wires
  `doors.rs::cell_blocked_by_entity`; the wiring-tick drain passes it).
- `Wiring.SkipWire` inside `WorldGen.OpenDoor`/`CloseDoor`/`ShiftTrapdoor`/
  `ShiftTallGate` (`Wiring.running`-gated): the edit-layer ports have no
  wiring handle; the arms collect the returned skip cells and insert them
  into `self.skip` after the call (order-identical: the C# skips happen
  before the trailing reframe, which reads no skip state).
- `SoundEngine.PlaySound(8/9)` door/trapdoor/gate cues: queued as
  `door_cues` events, drained app-side on the caller's stage stream
  (the `switch_clicks` seam).
- The `NetMessage.SendData(19)` door/gate echoes: Stage-2, not ported
  (single-player host).
