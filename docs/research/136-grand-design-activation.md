# 136 — The Grand Design 3611 drag / proj-651: landed-verification + the apply activation (95 open-seam 5)

> W4-f11 · 2026-11-21 · code-only (no cargo/tests/graphify run) · sources:
> `decompiled/Terraria/Projectile.cs:6990-7000` (the SetDefaults row),
> `:35697-35833` (aiStyle 125), `:72051-72067` (the Kill hook),
> `Wiring.cs:419-453`/`:3241` (MassWireOperation + the raw ToolMode read);
> tips: `origin/task/W4-d13-proj-ai-121-130` (latest `462c86097`, the
> activation contract at `ed631370d`), this branch.

## 1. VERIFIED LANDED (d13 — the projectile half, stays with proj)

The f11 note "the Grand Design 3611 drag stays with projectile 651" is
CONFIRMED against d13's tip:

| Piece | C# anchor | d13 landing |
| --- | --- | --- |
| The 651 SetDefaults row | `Projectile.cs:6990-7000` (10×10, aiStyle 125, friendly, ignoreWater, tileCollide false, penetrate −1, drawLayer 5) | `projectile/data.rs:1042` `WIRE_KITE = 651` + `:3580` the row |
| The item 3611 → proj 651 link | `Item.cs`'s `projectile: 651, channel: true` | `itemdata.rs:3777` (BASE — already on f11) |
| aiStyle 125 (the drag) | `:35697-35833` | `projectile_ai_wire_kite.rs` (`ai_wire_kite`) + the dispatch arm (`projectile.rs:17601` region); the two L-scans are C# NO-OPS (the `flag69` break never sets — d13's census, verified against the source) |
| The release latch | `:35720` `localAI[0] = 1` on the channel-drop | `ai_wire_kite` sets `local_ai[0] = 1.0` |
| The Kill hook | `:72051-72067`: `localAI[0] == 1 && owner == myPlayer` → `MassWireOperation(ps, pe, master)` (SP) / `SendData(109)` (MP) | the pool's death hook queues `(owner, ps_x, ps_y, pe_x, pe_y)` (`pending_mass_wire_ops` → `take_mass_wire_ops`); `ps = (ai[0], ai[1])`, `pe = Center.ToTileCoordinates()` (`(int)X >> 4`, `Utils.cs:1807-1810`) |
| The app buffer | — | `app.pending_mass_wire_ops` parked at the `world_sim/projectiles.rs` pass tail (`:456` on d13) — UNCONSUMED there, the documented deferral |

The MP `SendData(109)` half: d93 landed the 109/110 codecs
(`MASS_WIRE_OPERATION`/`_PAY`, `MessageBuffer.cs:3774-3803`); the apply
dispatch is the g48-g51 wave's (unclaimed by them per the f11 anchors
row — the 109 host arm runs the operation under a temporary ToolMode swap).

## 2. The f11 apply half — READY, the activation contract

f11's `wiring_devices::mass_wire_operation(app, ps, pe, mode)` landed
(`24fa39f14`). THIS commit closes the d13 contract's one open item — the
STREAM NOTE: the fn now takes `drop_rng: RngKey` and rolls the cut-cache
`Item.DropCache` drops on the CALLER's stage stream:

- the 3625 click arm passes `RngKey::UpdatePlayers` (the use path's stage —
  behaviour unchanged, the same stream as before);
- the kite drain passes `RngKey::UpdateProjectiles` (the Kill runs inside
  that swap, `Main.cs:17775`).

The drain itself is a UNION-MERGE hunk (d13's pool seam is unmerged on
f11 — a direct call would not compile); the exact hunk is published at
`world_sim/projectiles.rs`'s pass tail (after `take_spear_hold`):

```rust
for (owner, psx, psy, pex, pey) in app.projectiles.take_mass_wire_ops() {
    if owner as u32 != app.local_id { continue; } // the Kill gate
    crate::world_sim::wiring_devices::mass_wire_operation(
        app, (psx, psy), (pex, pey),
        app.wire_tool_mode,                    // RAW ToolMode (:3241)
        crate::app::RngKey::UpdateProjectiles, // the Kill's stage
    );
}
```

Contract pins (d13 `ed631370d`, re-verified against the C# here):
- **RAW ToolMode** — `MassWireOperationInner` reads `Settings.ToolMode`
  unmasked (`Wiring.cs:3241`); the Actuator strip is the 3625 arm's own
  (`Player.cs:46908`), NOT the kite's. The WiresRadial logic half
  (research/135) now makes the field reachable in-game.
- **No cooldown gate** — the Kill hook has none; the
  `wire_operations_cooldown` throttle is `CanUseItem`'s (the use arm's).
- **Tile coords both sides** — `(int)X >> 4` on the pool side, the core
  L-walk takes tile points; the shapes compose verbatim.
- **One drain shape** — d13 parks the buffer on `app.pending_mass_wire_ops`;
  the merger keeps EITHER the direct drain above OR the buffer + a
  second-pass drain, never both (the buffer field drops if the direct
  drain wins).

Witness (written not run): `w4_f11_the_kite_apply_keeps_the_actuator_bit_and_the_proj_stream`
(the raw Red+Actuator mode places both + bills both stocks through the
`UpdateProjectiles` stream param).

## 3. Status

- d13 projectile half: LANDED (its branch) — stays with proj, no f11 copy.
- f11 apply half: LANDED + stream-threaded (this branch).
- The drain: the published union hunk (§2) — executes at the d13×f11 merge.
- The 95 §8 open-seam list's "3611 stays with proj" note: CLOSED as a
  verification (the seam is no longer open — both halves exist, the merge
  hunk is pinned).

*All C# anchors re-read at the cited lines in this worktree's
`decompiled/Terraria/` checkout; the d13 comparisons read from
`git show origin/task/W4-d13-proj-ai-121-130:<file>`. No cargo/tests/
graphify run (code-only lane); the witness is WRITTEN NOT RUN.*
