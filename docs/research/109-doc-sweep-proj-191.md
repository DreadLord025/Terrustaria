# 109 — DOC-SWEEP lane 28: proj aiStyle 191-200 skip census

> Wave 4 · researched 2026-11-21 · sources: decompiled + crates + sibling lanes
> (d19/d69/d80/c10) · branch `task/W4-d20-proj-ai-191-200` (base DONE `13de9ccf`)

## Scope

The W4-d20 lane ported projectile aiStyles **191/192/195/196/198/199/200**
verbatim into `crates/tr-app/src/projectile_ai_misc.rs` (+ rows in
`projectile/data.rs`, dispatch in `projectile.rs`, the gift drain in
`world_sim/projectiles.rs`). This DOC-SWEEP re-verifies the **six recorded
skips** against the C# source and the sibling lanes, decides keep-one for the
one duplicate, and documents the tracked-loop contract d80 owns.

Method: static cross-reading only (no cargo/tests/graphify), per
`docs/research/MODULE-TEMPLATE.md`. Every claim carries a `File.cs:line` or
`crates/...rs::symbol` cite. "Not found" is reported as such.

**Headline result:** all six skips are either *correctly skipped* (the consumer
lives in another lane), a *duplicate* (keep-one at merge), or expose a *real
gap that is cross-lane* (item-use / pet-equip / camera). **No clean in-lane code
port was warranted** — porting any of the real gaps here would duplicate d19's
rows or encroach on the item-use `editing.rs` hot file. They are recorded below
as prioritized task seeds with full C# anchors so the owning lane can land them
1:1. The d20 AI arms themselves re-verify FAITHFUL.

| # | Skip | Verdict | Real gap? | Owning lane |
| --- | --- | --- | --- | --- |
| 1 | JimsDrone `CameraTracker`/`CheckSection` | correctly skipped | camera-pan source unported (not a render target) | camera / `world_sim/camera.rs` |
| 2 | Flint `itemLocation.Y` jitter draw-side | correctly skipped | none (draw-side, `noMelee`) | — (RNG draw kept) |
| 3 | 200-droppings dup vs c10 | duplicate — keep-one | none (both faithful) | merge decision (c10 module) |
| 4 | Jumino gift 5276 item row | row exists; use/eat arm inert | pet persistence + 994 row missing | pet-equip / item-use |
| 5 | 191 TrueNightsEdge vs 190 NightsEdge | AI arms faithful; use-path missing | terra-chain spawn arms absent everywhere | item-use (`editing.rs`) + d19 rows |
| 6 | tracked-loops JimsDrone/FlintCharge | d80 owns | deferred census sites | d80 tracked-sound |

---

## Item 1 — JimsDrone `DroneCameraTracker` / `CheckSection` (aiStyle 195)

**C# source.** `DataStructures/DroneCameraTracker.cs:1-34` is a 3-method camera
bookkeeping class, NOT a render target:

- `Track(Projectile)` (`:11-15`) stores `_trackedProjectile` + `_lastTrackedType`.
- `WorldClear()` (`:17-21`) nulls both (called `WorldGen.cs:7000`).
- `TryTracking(out Vector2 cameraPosition)` (`:23-33`) returns the projectile's
  `Center` iff `active && type == _lastTrackedType && owner == Main.myPlayer &&
  Main.LocalPlayer.remoteVisionForDrone`; else nulls the ref and returns false.

The single consumer is `Main.GetObjectTrackingCameraPan`
(`Main.cs:62722-62746`): after the spectate arm (`:62727-62732`) and the dead
gate (`:62733-62736`), `Main.DroneCameraTracker.TryTracking(out trackedPosition)`
(`:62737`) feeds the camera pan target, ahead of the golf arm (`:62741`). The
field is `Main.cs:1809`, constructed `Main.cs:67041`.

`Track` is called from two AI arms: the drone `AI_195_JimsDrone`
(`Projectile.cs:40955`, inside the `owner == Main.myPlayer` block) and the RC car
`AI_205_RemoteControlCar` (`Projectile.cs:38125`, arm at `:38076`). The gate flag
`remoteVisionForDrone` (`Player.cs:2390`) is set by equipping the **Jims Drone
Visor** item 5452 (`Player.cs:14213` `ApplyEquipVanity` case 5452, and `:15461`),
reset each frame in `ResetEffects` (`Player.cs:18712`). Item ids:
`ItemID.cs:12585` `JimsDrone = 5451` (the transmitter), `:12587`
`JimsDroneVisor = 5452`; projectile `ProjectileID.cs:2594` `JimsDrone = 1020`.

`CheckSectionsInCaseOwnerIsWatching` (`Projectile.cs:20376-20389`, called from
the drone tail `:41066`) is **MP section loading**: `netMode == 2` →
`RemoteClient.CheckSection(owner, position)` (`RemoteClient.cs:126`), else
`owner == myPlayer` → `ActiveSections.CheckSection(position)`
(`ActiveSections.cs:15`). It moves no gameplay state.

**Port status.** `projectile_ai_misc.rs::ai_jims_drone` (`:1092`) ports the drone
AI verbatim and explicitly skips both (`:1076-1078` doc, `:1140`
`// Main.DroneCameraTracker.Track — skipped (camera bookkeeping)`). The camera
pan port `world_sim/camera.rs::update_camera_pan` (`:71`) keeps
`target_pan = Vec2::ZERO` and documents (`:66-68`) that "the C#'s tracking
sources (the spectate/drone/golf cameras, `:62722-62745`) have no ported
consumers". `remoteVisionForDrone` has no port field (grep: 0 hits in crates).
`ActiveSections`/`CheckSection` have no sim port (the netglue `recheck_sections`
at `app/net_proj_sync.rs:295` is the MP wire sync, a different subsystem).

**d69 co-lane read.** `task/W4-d69-layer-target-consumers` (tip `9ba463ae`) ports
`crates/tr-render/src/targets.rs` — the `RenderToTargets` executor + skyTarget
clear + layer composite (`Main.cs:57277-57327/:62025-62026/:62196-63447`). It is
about the **full-screen render targets** (skyTarget/screenTarget/screenTargetSwap),
NOT the drone camera. Grep of d69 `targets.rs` for `drone|1020|remoteVision|camera`:
only generic `screen_size`/`offScreenRange` hits — **no drone-camera consumer**.

**Verdict — correctly skipped, NOT a render-target port.** The lane's port
condition ("port if self-contained render-target") does not hold:
`DroneCameraTracker` is a camera-pan *source*, and its consumer
(`GetObjectTrackingCameraPan`) is unported alongside the spectate and golf
tracking arms. Porting the tracker alone would be dead code (no pan consumer);
porting the pan consumer is the camera lane's scope (it must land spectate +
drone + golf together to keep the `:62722-62745` priority order). The
`CheckSection` half is MP-only. **Task seed T1** below.

---

## Item 2 — Flint `itemLocation.Y` jitter: draw-side vs location model (aiStyle 198)

**C# source.** `AI_198_Flint` (`Projectile.cs:52238-52315`), charge phase
(`ai[0] == 0`, `:52280-52314`):

```
player.itemAnimationMax = useAnimation;          // :52307
player.SetDummyItemTime(num5);                   // :52308
player.itemLocation.Y += Main.rand.Next(2);      // :52309  ← the jitter
if (Main.rand.Next(10) == 0) { ...dust 228... }  // :52310-52313
```

The jitter is **one `Main.rand.Next(2)` draw per charge tick** added to
`player.itemLocation.Y`. `itemLocation` (`Player.cs:1500`) is the held-item
draw anchor, recomputed every frame by `ItemCheck_ApplyHoldStyle`
(`Player.cs:50240`, the per-hold-style arms `:50279-50516` set
`itemLocation.X/.Y` from `position`/`MountedCenter`/`heldItemFrame`).

**Location-model census — is it gameplay or draw?** The readers of
`itemLocation` split cleanly:

- **Draw / light / dust** (the overwhelming majority): the hold-style placement
  itself (`:50279+`), `ItemCheck_EmitUseVisuals` dust/light (`:49904-50092`),
  the torch light (`:44916`), the melee-swing arc location (`:44751`).
- **Melee hitbox** (`ItemCheck_GetMeleeHitbox`, `Player.cs:45099-45126`): builds
  `itemRectangle` from `(int)itemLocation.X/.Y` (`:45102`/`:45126`). BUT its
  caller is gated `!sItem.noMelee` (`Player.cs:43648`:
  `((sItem.damage >= 0 && sItem.type > 0 && !sItem.noMelee) || ...)`) — and the
  **Dead Cells Flint item 5462 is `noMelee = true`** (`Item.cs:43095`). So the
  flint's `itemLocation` **never feeds a melee hitbox**; its damage rides the
  1043 slash / 1041 spikes the arm spawns, not the held-item rectangle.
- The bee-glove arm (`:43635`) reads it only for item 1124 (not the flint).

Therefore the `itemLocation.Y += Next(2)` jitter is **purely a draw-side
wobble** of the held flint sprite during the charge — it shifts no collision,
no spawn point (the slash spawns at `player.MountedCenter`, `:52254`, not
`itemLocation`), no damage. The only gameplay-observable effect is the **RNG
draw itself** on the `UpdateProjectiles` stream.

**Port status.** `projectile_ai_misc.rs::ai_flint` (`:1386`) keeps the draw and
documents the skip precisely:

- `:1376-1378` doc: "The `itemLocation.Y += Next(2)` jitter (`:52309`) keeps its
  draw on the stream; the draw-side item location is unmodelled (the thrust
  placement recomputes it each frame — a documented skip)."
- `:1478-1480`: `// itemLocation.Y += Next(2) — the draw stays; the location is
  draw-side (unmodelled).` then `let _ = rng.next_range(0, 2);` — **the draw is
  consumed**, preserving stream order ahead of the `Next(10)` dust gate
  (`:1482`) and the dust's three floats (`:1483-1485`).

The port has no persistent `player.itemLocation` field; the melee-swing draw
recomputes the anchor per frame (`melee_swing.rs:10` "render-side arc math
(`item_rotation`/`item_location`...)", `player.rs:410` `item_location: [f32; 2]`
is a draw-local). This matches the C# semantics: the jitter is a transient
draw offset that `ItemCheck_ApplyHoldStyle` overwrites next frame.

**Verdict — correctly skipped, draw kept.** The lane's note ("your draw kept —
location model census") is confirmed: the RNG draw is preserved (determinism
intact), the visual wobble is the documented draw-side skip, and the location
model census proves **no gameplay consumer** for the flint (`noMelee`). No gap,
no port needed. The only future consumer would be a held-item draw layer that
renders the flint sprite at `itemLocation` — at which point the +0/+1 px wobble
becomes visible and the (already-consumed) draw can feed it. **No task seed**
(cosmetic, blocked on the held-item draw layer).
