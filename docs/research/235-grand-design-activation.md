# 235 — The Grand Design 3611 drag / proj-651: landed-verification + the apply activation (the kite seam)

> Renumbered 136 → 153 → **235** (2026-11-21, the INDEX-registry
> collision — g31's `136-tilepass-d-hunks` was first-pushed; the
> research/150 registry's key-136 row assigns 235; see research/233's
> header for the full note).
>
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
  (research/234) now makes the field reachable in-game.
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

---

## 4. OPEN-SEAMS-REMAINING — the updated table (retire phase, 2026-11-21)

Consumes **c42e-final** = `origin/task/W4-c42-wiring-device-arms` tip
`79b81ae77` (the union-exec + fold-133 EXECUTED; its open-seam audit is
research/95 §8's eleven seams + the §10/§10.1 deltas). Cross-checked
against `origin/task/W4-g48-net-apply-arms-1` (the 19 apply/relay LANDED
there: `net_apply.rs::host_door_toggle`/`client_door_toggle`/
`apply_door_action`, the netglue dispatch arms) and
`origin/task/W4-d13-proj-ai-121-130` (`462c86097`, the kite half).
The f11 five-seam wave (`b1b140979`/`11788ce9d5`/`9ae0edc97`/
`809d9a2cf`/`84ed6342b`) closed §8.1 + §8.2 + §8.5 on THIS branch and
landed the radial/kite halves — none of that is on c42 yet; the exact
fold-list is research/133 §7 (this branch).

**Numbering note**: the f11 wave's commit messages tagged the radial
"95-UI/95 open-seam 4" and the kite "95-5" — those are the f11 WORK
ids, NOT 95 §8 indices (§8.4 = the door-cue stage streams, closed by
the c42 drain union; §8.5 = `skip_wires=true`). The table below uses
the canonical 95 §8 indices.

| 95 §8 seam | State after the f11 wave | Where it stands / who closes it |
| --- | --- | --- |
| 8.1 `EmptyTile(ignoreTiles:true)` entity half | **CLOSED on f11** (`b1b140979`): `install/clear_entity_gate` + `entity_blocks`, the trapdoor/tall-gate arms read the snapshot, install/clear pairs at all 14 production flood entries (the census: research/133 §7.1) | c42 still passes `&\|_,_\| false` (its `wiring.rs:2617`/`:2630`/`:2656`); the fold-list = research/133 §7.1. Door 11 unaffected (`forced:true`). |
| 8.2 `SendData(19)` echoes | **CLOSED send-half on f11** (`11788ce9d5`): `num66`/`4+flag4`/`±num67` payloads → `door_toggles` → `send_door_toggle` (Off = the netMode-0 fold). **Apply-half LANDED on g48** (`net_apply.rs`) | **REMAINS: the union merge** — g48's `net.rs` id/struct/codec is byte-identical to f11's (keep-one, trivial dedupe); the netglue `C2s/S2c::DoorToggle(_)` no-op arms drop for g48's dispatch arms; f11's queue/drain folds onto c42 (research/133 §7.2). g48's 2/3 reader passes the REAL entity closure already — the 8.1 fold extends it to the wiring floods. |
| 8.3 SkipWire inside the WorldGen bodies | closed (the §1 Option seam, c42-exec `f982f1d4`) | nothing remains |
| 8.4 door-cue stage streams | closed (the c42 12-site `take_door_cues` drain union) | nothing remains; f11's single `door_sounds` drain is DEPRECATED-BY-UNION (133 §4b ruling stands) |
| 8.5 `try_toggle_light` `skip_wires=true` | **CLOSED structurally BOTH sides**: c42-exec item 6 (`6381b52d`, the free-fn family collects + discards); f11 (`9ae0edc97`, the method-side forced gates + campfire collector, research/233) | the residual is the LIGHT-FAMILY DEDUPE itself (c42's unified free fns vs f11's gated methods — never ship both); research/233's §1a/§1b hunks are SUPERSEDED by the c42 shape (the §1 header note), the surviving f11 unique = the forced-gate witness (research/133 §7.3) |
| 8.6 `SendTileSquare` tails in the Toggle* statics | OPEN (MP-only, all lanes agree) | the net sync wave; unchanged |
| 8.7 hopper `SyncItem`/`VisualizeChestTransfer` | OPEN (net/draw seams) | the particles/net tasks; unchanged |
| 8.8 `FrameTrack` non-pound + `flag4`/`KillTile(fail)` | OPEN (the hammer-placement half) | `edit/hammer.rs` `Consumed` follow-up, e20 assert convention; unchanged |
| 8.9 `WireState::any` unconsumed | OPEN (e20-unique helper) | the `IsHopperInRangeOf` follow-up reads it; unchanged |
| 8.10 boulder aiStyle-25 `proj_data` row | OPEN (e21's DeviceShot drops with the note) | the projectile family wave; unchanged |
| 8.11 `can_kill_tile` slice completeness | OPEN (the chest-inventory cases pass vacuously) | M7-TE-2; unchanged |

**The three post-wave remainders the merger executes** (the Lead's
open-seams-remaining shortlist):

1. **SendData-19 apply @ g48b** — the apply/relay dispatch is g48's
   (`net_apply.rs`, LANDED); what remains is the KEEP-ONE codec dedupe
   + the f11 send-half fold onto c42 (research/133 §7.2). No new code
   is invented at merge — both halves exist, byte-identical wire shape.
2. **Radial-draw @ the texture wave** — research/234 §2's ~450-line
   draw contract (`WireUi[12]` + `BuilderAcc` atlas loads, the centred
   two-sprite seam, `DrawCursorArea`, the gamepad folds). The LOGIC
   half is landed and reachable; the draw waits on the atlas-load
   wave (`Main.cs:31665-31692`'s cursor draws are the second consumer).
3. **Kite-drain @ the d13 union** — research/235 §2's published hunk
   at the projectiles pass tail (raw ToolMode, no Actuator strip, no
   cooldown gate, `owner == local_id`, `UpdateProjectiles` stream);
   one drain shape, never both (the buffer field drops if the direct
   drain wins). The f11 apply is stream-threaded and READY.

*Table verified by grep at the cited tips (c42 `79b81ae77`, g48, d13
`462c86097`); no cargo/tests/graphify run (code-only lane).*
