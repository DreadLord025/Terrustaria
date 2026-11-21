# 149 — Devices-family bible: the Wiring.cs / Minecart.cs device estate — who owns what at retire

> **STATUS: BIBLE (ownership map) — 2026-11-21, branch
> `task/W4-e20-wiring-device-arms-2` (retire phase, the FINAL doc).**
> One-page map of the whole wiring-device family after the e20 × f11 ×
> c42 × e21 × d106 lanes: every subsystem, its canonical owner, the
> merge instruction, and the open contracts. Evidence lives in the cited
> research docs — this file re-derives nothing. Companion to the seal
> (research-71 §8). Code-only lane.

## 1. Ownership table

| Subsystem (C# anchor) | Canonical owner / state at retire | Merge instruction | Evidence |
| --- | --- | --- | --- |
| Tile 314 switch-track: `FlipSwitchTrack`/`FrameTrack` FULL (pound/hammer/place slices), `_trackSwitchOptions` OnceLock, `TRACK_FRAMES` superset table (`Minecart.cs:125-518/:943-1319`) | **e20** `tr-app/src/minecart.rs` + `tr-core` wiring case 314 request seam; state at `02ac64ab6` | take e20 wholesale; DROP c42's `e8e6b159c` halves (stale, non-compiling — research-146) and f11's `tr-core/src/minecart.rs` duplicate | 71 §1/§2/§6b, 146 |
| Track pieces: `PlaceTrack`/`GetTrackItem`/the placement wake/`cell_drops` 314 (`Minecart.cs:1486-1517`, `WorldGen.cs:60231/:64582/:85290-85294`) | **e20** `edit::place_track` + `minecart::track_item` (`865e9d83f`, mechanical `29d9dc224`) | take e20; items 2340/2492/2739 (3183 = Golden Bug Net, NOT a track item) | 71 §6c |
| Hammer 314 arm: `PoundOutcome::TrackHammer`, Item52 fail cue + wake drain (`Player.cs:46058`, `WorldGen.cs:66531/:63735`) | **e20** (`0cea13365`) | take e20; c42's `hammer.rs:417` still `Consumed` (pre-follow-up seam) | 71 §6b, 146 §3 |
| `IsPressurePlate` (`Minecart.cs:549-560`) | **e20** `minecart::is_pressure_plate` (`6d1754719`, app-side, dead_code) + **d106** inline predicate in `is_it_a_trigger` (tr-core) | keep ONE truth at merge — the seam resolution is research-148 §3c | 71 §7b, 148 |
| Hoppers 21/467: `Hopper`/`TryToPutItemInChest`/`TryMoveCoinsInChest`/`TryAddingToStack`/`TryAddingToEmptySlot`/`CoinsCount`/`CoinsSplit`/`SortInventory` (`Wiring.cs:2638-2798`, `Utils.cs:1019-1060`) | **e20** `world_sim/wiring.rs` (Vec signatures `a2d6cdad0`, PlaySound(7) `f167082ed`, strict-`<` box `474bc8b55`, remainder `dcbb29f82`) | take e20 wholesale; c42's slice+resize pair DOES NOT COMPILE (146 §2); f11's `wiring_devices.rs` duplicate dropped (its wins already merged in) | 71 §2/§3, 146 |
| Extractinators 219/642: `Extractinator`/`TryFindChestForExtractinator` + the roll (`Wiring.cs:2554-2608`, `ExtractinatorHelper.cs`) | **e20** (anchor-centre drop, wiring-stream RNG, all-kinds consume sweep `0055ac801`) | take e20; f11's tile-origin spawn is WRONG | 71 §2/§6e |
| Trap arms 137 + spiky-ball cap (`Wiring.cs:1819/:1898-1950`) | **e20** (`cb14ba4fb`: wall `speedY=0`, cap anchor+Center, `TrapRequest::Spear` `cap_x/cap_y` shape) | take e20's request shape — f11/c42 constructors must adapt, not the reverse | 71 §3a |
| Light family + 429 witnesses; b=0 neighbour pin; lantern fix | **e20** (`ee6df0f2d`, `44d7d20b7`, `2322af906`) | take e20 | 71 §3a.7 |
| Doors 10/11, trapdoors 386/387, tall gates 388/389, torch/campfire/slab, `validate_tile_square` | **c42** union-exec (`f982f1d4f`/`ed88ec3de`/`552a0a44a` + g4 witnesses `23411fa77`) — f11 originals folded | c42's own terms (non-device) | 71 §5/§6a |
| 144/421/422 wakes + actuator guard, 235/226 `downedPlantBoss`, 506 shared validator | **c42** `de176ccdd` (ANCESTOR of c42 tip; e20 carries pre-fix halves) | merger takes c42's fix round over e20's pre-fix halves | 71 §7a/§7c |
| `clear_all` + world-load hook, UpdateMech pop-not-remove, LogicGatePass tail, HitWire removed-clear, 216/fireworks SkipWire, mass-wire + 3625, anchor helper, 11 `Toggle*` unification | **c42** fold-133 (`e5569c2a7`…`79b81ae77`) + d6 (`6381b52d1`) | c42's own terms | 71 §7c, c42 research/133 |
| `Collision.EmptyTile` entity gate (§8.1) + `SendData(19)` door echoes (§8.2) | **c42 IN FLIGHT** (uncommitted at probe 10:10, research-147) | wait for c42's §8 commits; textual adjacency with e20's wiring.rs only | 147 |
| Pump `XferWater` gate, monolith nine-kinds, grate 546/557 wakes, boulder-statue 99 | **e21** (`12fb1a2e` tip; pump fix `7c9a67a2f`) | union the two adjacency regions; no semantic overlap with e20 | 71 §4 |
| Dart/geyser/explosives/mine/teleport/timers (`Wiring.cs:3074-3118` etc.) | **landed pre-e20** (S1-L-6 / D5), witnessed; no separate geyser/explosives timer fn exists — `CheckMech` args | nothing to do | 71 §7c |
| `BrokenTrapCleanup` worldgen pass (`WorldGen.cs:22121-22140/:26788-26958`) | **d106 R5** `e2c2609c5` (`worldgen/cleanup.rs` + pipeline slot after stalac) — ALREADY LANDED, written NOT run | do NOT re-port; the finish worklist (kill_tile no-item, BFS leftover edge, seam) is research-148 | 148 |
| Rail-sprite draw: `TrackColors`/`GetSourceRect`/`Draw*` (`Minecart.cs:1321-1532`) | **CONTRACT: M2-MOUNT-1/3 rail-draw lane** — `TRACK_FRAMES`/`TEXTURE_POSITION` feed ready | contracted, unclaimed | 71 §7b |
| Digger auto-fit: `GetAreExpectationsForSidesMet`/`TryFittingTileOrientation`/`ConvertOffsetY*` (`Minecart.cs:1534-1621`) | **CONTRACT: M2-MOUNT-4 mechanical-cart lane** (sole consumer `MinecartDiggerHelper.cs`) | contracted, unclaimed | 71 §7b |
| Ride physics (`TrackCollision` etc.) | **M2-PHYS-7 landed on main** pre-branch-point; f6 owns only the 27 Mount.cs dust sites | nothing to do | 71 §7b |
| MP-only: `SetCurrentUser`, `IsHopperInRangeOf` (`WorldItem.cs:237-247` netMode gate), `SendTileSquare`/net echoes | SP-discarded / MP contracts (g48b codec for the 19 echoes) | recorded deviations | 71 §6d/§7c, 147 |
| `ClearAllBrokenTraps` (`WorldGen.cs:26825-26838`) | dead C# API — ZERO callers | no port; discard | 148 §1 |

## 2. The three blocking rules for the merger

1. **c42's device halves CANNOT merge as-is** — slice+resize compile
   error + missing hammer/track-item census. Re-pick from the e20
   `6d1754719` chain (exact list: research-146 §1). Everything else c42
   owns merges on its own terms.
2. **One truth per table/predicate** — `TRACK_FRAMES` (never f11's four
   flat arrays too), `FIRST_PRESSURE_FRAME`, the `IsPressurePlate`
   predicate (148 §3c), the anchor helper (c42's `a4b8a6027` adoption).
3. **The §3a fixes are shape-breaking** — `TrapRequest::Spear`
   `cap_x/cap_y`: any lane constructing the old shape must adapt to e20's,
   never the reverse (71 §3a.2/§5.4).

## 3. Doc map (retire publications, this branch)

| Doc | Commit | Role |
| --- | --- | --- |
| `71-wiring-device-reconcile.md` §8 SEAL | (this phase) | the closed reconcile + census record |
| `146-c42-repick-critical.md` | `e47e1399f` | merger-BLOCKING escalation, standalone |
| `147-c42e-session-probe.md` | `038af46cf` | c42e ALIVE verdict + recommendation |
| `148-brokentrapcleanup-wg-seed.md` | `d818aee06` | wg-lane task-seed (dedupe: d106 landed) |
| `149-devices-family-bible.md` | (this file) | the ownership map |

*All rows trace to hand-verified evidence in the cited docs; no cargo/
tests/rustfmt/graphify run (code-only retire lane). Branch FINAL state:
the seal commit on top of `d818aee06`.*
