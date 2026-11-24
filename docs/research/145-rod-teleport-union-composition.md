# 145 — the RoD `teleport_style(1)` ⨉ TeleportEffect-union composition (W4-g89h)

Status: CROSS-NOTE for the g11 union merger + the g89 rod lane. Written by
W4-g89 (branch `task/W4-g89-aegis-fruit-use`, tip lineage `59864256` →
`e32be4cf`). Reads the EXECUTED union on `origin/task/W4-g11-dust-main-sites-1`
(tip `9a9ac1fd0` at the g89h probe; the exact hunks below re-pinned at the
FINAL tip `98283df08`, research/83+116 FINAL). Code-only: no cargo/tests/
rustfmt/graphify; the merger actions below are WRITTEN, NOT APPLIED (they
are the g11/g89 merger's worklist, not this lane's edits — the union code does
not exist on g89's base `a725acb04`).

> **Numbering note (W4-g89j FINAL):** this note was FIRST published as
> research/139 (`cdb70e5ed`). The g52c-net-master lane owns 139 (its
> 139-COLLISION report); this doc is RENAMED to **145** — 139a would still
> collide with the 139 family, and the pushed-branch census shows 140
> (f29 map-overlay), 141 (e9 npc-sounds), 142 (d122 savings), 143/144 (c4)
> taken, 145 free. Every "research/139" citation in the g89h-era commits
> (`cdb70e5ed` ANALYSIS/PORT_MAP/INDEX rows, `rod_teleport.rs` module doc)
> means THIS file.

## 1. The composition verdict: the rod call COMPOSES, zero rod-side change

g89f's rod arm calls `p.teleport_style(point.x, point.y, 1)`
(`rod_teleport.rs::rod_teleport`, the `Player.cs:45512` `Teleport(point, 1)`
leg). On g11's tip that method is the thin wrapper
`teleport_style_with_extra(new_x, new_y, style, 0)` (`player.rs:9498-9500`),
which:

1. `queue_teleport_effect` — captures the OLD box + `mult = Clamp(1 -
   teleportTime*0.99, 0.01, 1)` read PRE-move (`:37442`), `extra_info2 = 0`
   (style 1 is not the portal style 4), into `pending_teleport_effect`;
2. `teleport_move_core` — the hooks/vanity/shimmer strip + position +
   fallStart + oldPosition (`:37420-37431`/`:37446`/`:37455`/`:37488`);
3. the grants `teleport_time = 1`/`teleport_style_id = style`
   (`:37490-37491`; the replay re-writes them at the C# position, gap 6 —
   idempotent).

The player-tick drain (`world_sim/player.rs:1564`) takes the queue into a
`PlayerTeleportReq` on `RngKey::UpdatePlayers` and `main_dust::player_teleport`
replays Entry-at-OLD → move → Exit-at-NEW → grants. **Style 1 is a live arm of
the union body** (`teleport_dust.rs:90-129`, the `0 | 1` shared shape): the
Item8 cue FIRST at the rect centre (`Main.cs:13258`), then
`(Width*Height/5)*mult` INTEGER-truncated type-164 dust, each with the
`Next(20,70)*0.01` scale ASSIGN + the j<10 / j<5 +0.25 bumps
(`:13259-13272`). Both legs fire (Entry at the OLD box, Exit at the NEW box),
so the style-1 Entry/Exit dust is NOW-LIVE through the union — the g89f
module doc's "TeleportEffect bursts stay the unported system (recall
precedent)" citation is SUPERSEDED for the rod once the union lands: the
bursts arrive via the queue, no rod-side dust code needed.

The camera: g11's replay OWNS the myPlayer transition for BOTH callers
(`main_dust.rs:435-450`, gap 5 closure — style 1 → `lerpSpeedupDelay = 10`,
the OLD centre feeds the distance gate). See §3 for the dedup this forces on
the rod's inline call.

## 2. NEW GAP (not in research/116's six): the editing-phase caller drains a
tick late — the chaos-roll RNG order flips

research/116 §2's wiring contract drains the queue in the PLAYER tick
(`world_sim/player.rs`, the `UpdatePlayers` pass). That is correct for the
player-tick callers (the no-space / Shellphone style-11 fallback, the
Queen-Slime style-10 hook drained at the projectile tail). **The rod is an
EDITING-phase caller**: `rod_item_check` runs in `update_editing`
(`world_sim/editing.rs:2139`, g89 tip; g11 tip has no rod arm), which the
frame orchestrator runs in `editing_net_phase` (`app/frame/world.rs:658`) —
AFTER `players_phase` (`:633`) in the SAME fixed step. So the rod's queued
effect is NOT drained this tick; the NEXT tick's `players_phase` drains it.

The C# order within the one `ItemCheck_UseTeleportRod` call
(`Player.cs:45512-45533`, all under the `Main.cs:17754` `UpdatePlayers`
`Main.rand` swap):

```
Teleport(point, 1)
  ├─ TeleportEffect(Entry)   :37444  — Item8 cue + N dust draws
  ├─ move + camera           :37446-37463
  └─ TeleportEffect(Exit)    :37489  — Item8 cue + N dust draws
AddBuff / chaos block        :45518-45533
  └─ Main.rand.Next(2)       :45522  — the death-reason roll (ONE draw,
                                       ONLY when buff 88 was up)
```

So the C# draws **Entry-dust, Exit-dust, THEN the chaos Next(2)** on the one
stream. With the next-tick drain the port draws **the chaos Next(2) (this
tick, in `chaos_state_tax`) THEN Entry/Exit-dust (next tick's drain)** — the
chaos roll and the dust draws SWAP positions on `UpdatePlayers`. Every
downstream `UpdatePlayers` draw shifts; the death-reason roll reads a
different value than the C# for the same seed. This is a determinism break
(the AGENTS.md hard invariant), invisible to the g89f witnesses (they pin the
chaos roll on a cloned stream with NO dust present — correct for the pre-union
base, wrong once the union's dust draws share the stream).

**Fix (the merger's, forward-only):** drain the pending teleport effect at the
EDITING-phase tail too, on the same `UpdatePlayers` key, immediately after
`rod_item_check` returns (the `recall_teleport` post-step convention — the
rod's teleport + chaos tax both run inside `update_editing`, so a drain right
after the rod arm restores the C# Entry→Exit→chaos order within the tick).
The player-tick drain stays for the player-tick callers; the two drains are
disjoint per tick (a caller queues in exactly one phase). The same-tick
deferral of the MOVE is already observation-identical (g11's contract —
nothing probes the dust between); only the RNG ORDER needs the editing-tail
drain. The EXACT drain-move hunks are published in §2a below (W4-g89j).

### 2a. GAP-7 FINAL — the EXACT drain-move hunks (g89j, for the g11-union merger)

Target: `origin/task/W4-g11-dust-main-sites-1` tip **`98283df08`** (the g11
FINAL — research/83+116 FINAL docs; the union code is unchanged from
`9a9ac1fd0`/`cc8c767b0`). The g89 side is `task/W4-g89-aegis-fruit-use`
(g89f `598642563` rod machine). **Placement refinement over the g89h prose:**
the drain must sit BETWEEN the teleport leg (`rod_teleport`, `Player.cs:
45512`) and the chaos block (`chaos_state_tax`, `:45518`) — INSIDE
`run_rod_arm`, not after `rod_item_check` returns (the chaos `Next(2)` runs
inside the same call; a drain after the whole arm would still leave the roll
ahead of the dust). Three hunks, all merger-applied, NONE applied on either
branch today (the seam is unreachable pre-union: g89's base has no
`pending_teleport_effect`/`take_teleport_effect`/`PlayerTeleportReq`/
`main_dust::player_teleport` — `teleport_style` on g89 is the bare move core,
`player.rs:9496`, no queue; g11's tip has no rod arm — `git grep rod_item_check`
empty).

**Hunk 1 — `crates/tr-app/src/rod_teleport.rs` (the g89 file, at the union),
`run_rod_arm` tail (g89 tip `:257-263`).** OLD:

```rust
    // `:45512` Teleport(point, 1). (`:45513` SendData(65) — no SP port.)
    rod_teleport(app, point);
    // `:45514-45517` — the Rod of Harmony skips the chaos block entirely.
    if held.kind != ROD_OF_DISCORD {
        return;
    }
    chaos_state_tax(app);
```

NEW (the drain fires for BOTH rods — the harmony 5335 teleports style 1 too,
its dust draws precede everything downstream even though it rolls no chaos):

```rust
    // `:45512` Teleport(point, 1). (`:45513` SendData(65) — no SP port.)
    rod_teleport(app, point);
    // W4-g89j GAP-7 (research/145 §2a) — the EDITING-tail drain: the rod
    // queued the Entry/Exit pair in `teleport_style` (g11's wrapper); the
    // player-tick drain already ran this fixed step (`players_phase` before
    // `editing_net_phase`, `app/frame/world.rs:633`/`:658`), so WITHOUT this
    // drain the pair fires NEXT tick — AFTER the chaos `Next(2)` below,
    // reversing the C# order (dust `:37444`/`:37489` inside Teleport BEFORE
    // the chaos block `:45518`). Drain HERE on the same UpdatePlayers key:
    // Entry draws → move re-apply (idempotent) → camera → Exit draws, then
    // the chaos roll — the C# stream order restored. The player-tick drain
    // (`world_sim/player.rs:1564`→`:1691-1693` at g11 tip 98283df08) stays
    // for the player-tick callers; the queues are disjoint per tick.
    if let Some((ox, oy, nx, ny, style, e2, ei, mult)) =
        app.player.as_mut().and_then(|p| p.take_teleport_effect())
    {
        crate::main_dust::player_teleport(
            app,
            crate::main_dust::PlayerTeleportReq {
                old_x: ox,
                old_y: oy,
                new_x: nx,
                new_y: ny,
                style,
                extra_info2: e2,
                extra_info: ei,
                mult,
                key: crate::app::RngKey::UpdatePlayers,
            },
        );
    }
    // `:45514-45517` — the Rod of Harmony skips the chaos block entirely.
    if held.kind != ROD_OF_DISCORD {
        return;
    }
    chaos_state_tax(app);
```

(The req shape is the g11 player-tick drain's verbatim — `world_sim/player.rs:
1564-1576` at `98283df08`, the tuple order `(old_x, old_y, new_x, new_y, style,
extra_info2, extra_info, mult)` from `Player::take_teleport_effect`,
`player.rs:3431`; `PlayerTeleportReq` is `main_dust.rs:405-419`, the replay
`:433-497`.)

**Hunk 2 — the gap-5 camera dedup, SAME commit (research/145 §3).**
`rod_teleport.rs::rod_teleport` (g89 tip `:143-155`). OLD:

```rust
fn rod_teleport(app: &mut App, point: Vec2) {
    let Some(p) = app.player.as_mut() else {
        return;
    };
    let old = Vec2::new(p.x, p.y);
    p.teleport_style(point.x, point.y, 1);
    // `:45463` StartCameraTransitionForPlayerTeleport(vector, 0.1f, 10).
    let mut state = std::mem::take(&mut app.camera_state);
    crate::world_sim::camera::start_camera_transition_for_player_teleport(
        &mut state, old, 0.1, 10,
    );
    app.camera_state = state;
}
```

NEW (the replay owns the camera for BOTH callers — `main_dust.rs:463-478` at
g11 tip, style 1 → delay 10, the OLD centre feeds the distance gate; keeping
the inline call would `set_camera_lerp` TWICE and re-arm the 10-tick speedup
delay a tick late):

```rust
fn rod_teleport(app: &mut App, point: Vec2) {
    let Some(p) = app.player.as_mut() else {
        return;
    };
    // `:45512` — the g11 wrapper queues the Entry/Exit pair + runs the move
    // core; the GAP-7 drain in `run_rod_arm` replays the effects AND owns
    // the `:45456-45463` camera transition (`main_dust::player_teleport`,
    // gap 5 — the inline call retired with the drain, research/145 §3).
    p.teleport_style(point.x, point.y, 1);
}
```

Hunk 2 MUST land with hunk 1 (same merger commit): without the drain the
inline retirement would drop the camera entirely; without the retirement the
drain double-sets it.

**Hunk 3 — the re-pinned chaos witness (the g89f
`w4_g89f_the_chaos_tax_books_the_teleport_death_at_low_life`,
`rod_teleport.rs:426-453`).** Its cloned-stream tail OLD:

```rust
        // Exactly ONE Next(2) draw on the stream (the reason roll).
        let _ = witness.next_max(2);
        assert_eq!(
            *app.rngs.stream(crate::app::RngKey::UpdatePlayers),
            witness,
            "one Next(2) draw, no others"
        );
```

NEW (the dust draws now sit AHEAD of the roll on the same stream — the
style-1 shape, `teleport_dust.rs:93-131` / `Main.cs:13258-13272`; the Item8
cue is `play_sound(2, 8, …)` and its pitch draws ride the same stream through
the App audio façade, exactly as the replay invokes it):

```rust
        // The GAP-7 order: Entry dust → Exit dust → the chaos Next(2)
        // (research/145 §2a). Each leg: the Item8 cue's pitch draws (the
        // audio façade's case-2 shape) then count = (W*H/5)*mult INTEGER
        // dust, each a NewDust allocation draw-set + one Next(20,70) scale.
        // Replay BOTH legs on the clone through the SAME seams the drain
        // uses (the g11 teleport_dust witness rig), THEN the one Next(2).
        let (w, h) = {
            let p = app.player.as_ref().unwrap();
            (p.width as i32, p.height as i32)
        };
        let count = ((w * h / 5) as f32 * 1.0) as i32; // mult 1: teleport_time 0 pre-move
        for _leg in 0..2 {
            // the Item8 cue's pitch draws (case 2 — the audio façade)
            // + count × (NewDust draw-set + Next(20,70)) — the rig helper
            // `teleport_dust` witnesses use; pin via the shared helper.
            crate::main_dust::tests::replay_style1_leg_draws(&mut witness, count);
        }
        let _ = witness.next_max(2);
        assert_eq!(
            *app.rngs.stream(crate::app::RngKey::UpdatePlayers),
            witness,
            "Entry dust + Exit dust AHEAD of the one Next(2) — GAP-7 closed"
        );
```

(`replay_style1_leg_draws` is the witness-rig helper the merger adds beside
the g11 `main_dust.rs:1156-1180` drain witnesses — it must call the SAME
`DustPool::new_dust` + audio pitch seams the live replay calls, not a
hand-counted approximation; the g11 style-1 witnesses already pin the
per-dust draw-set, reuse that rig. The mult is 1.0 in the rig: the player
never teleported before, `teleport_time = 0`.)

**Witness the merger must ALSO add (new, not a re-pin):** a two-phase rig —
run `players_phase`'s drain with an EMPTY queue, then the rod arm in
`update_editing`, and assert (a) the queue is empty at the editing tail (the
drain consumed it THIS tick), (b) a following `players_phase` draws NOTHING
teleport-related (no double-fire), (c) the death-reason value equals the C#
seeded recording where the dust draws precede the roll.

## 3. The camera double-set dedup (gap-5 closure ⨉ the rod's inline call)

g89f's `rod_teleport` starts the style-1 camera transition INLINE
(`rod_teleport.rs:121-126` at the g89f probe; `:149-154` at the g89i tip,
`start_camera_transition_for_player_teleport(old, 0.1, 10)` — the
`Player.cs:45463` leg). g11's gap-5 closure moved that into the replay
(`main_dust.rs:435-450` at the g89h probe; `:463-478` at the FINAL tip
`98283df08`) and RETIRED the wiring pad's inline arm. At the union the rod's
inline call would set the camera TWICE (inline this tick + replay next tick)
— the second `set_camera_lerp` resets `camera_lerp_timer = 0`
(`camera.rs:57-60`), re-arming the 10-tick speedup delay one tick late.

**Fix (the merger's):** RETIRE the rod's inline camera call — the replay
owns it for the rod too (the gap-5 "replay owns the camera for BOTH callers"
contract). The rod keeps only `p.teleport_style(point.x, point.y, 1)`. This
is a g89-side edit the merger applies when the union lands; it is NOT applied
on this branch (the replay does not exist on g89's base, so retiring the
inline call now would drop the camera entirely pre-union). The EXACT hunk is
§2a hunk 2 — it lands in the SAME merger commit as the editing-tail drain.

## 4. What stays g89's (no union interaction)

- The chaos-state tax block (`chaos_state_tax`, `Player.cs:45518-45533`) —
  pure player state + the ONE `Next(2)` draw; the union adds dust draws AHEAD
  of it on the stream (§2), the block itself is unchanged.
- The 5335 chaos-SKIP (`:45514-45517`) — the harmony rod teleports (style 1,
  dust fires) but never arms buff 88 / never pays the tax / never rolls the
  reason. The union's dust is identical for both rods (same style 1); only
  the chaos block differs. (The §2a drain sits BEFORE the 5335 early return
  so the harmony rod's dust also drains this tick.)
- The aim gates / cadence / refusals (`:45478-45511`) — upstream of the
  teleport, no dust/RNG interaction.

## 5. Cross-references

- research/116 (g11): the 4-way union plan + the EXECUTED record (§4 of its
  tip) — the six gaps it closed; §2 above is the SEVENTH (the editing-phase
  caller), §3 the camera dedup the gap-5 closure forces on the rod. g11
  FINAL (`98283df08`) marks 116 EXECUTED+FINAL — GAP-7 is NOT in its six;
  this note is the standing g11-union worklist item.
- `teleport_dust.rs:90-129` (g11 tip): the style-1 arm (the `0 | 1` shape).
- `main_dust.rs:405-497` (g11 tip `98283df08`): `PlayerTeleportReq` +
  `player_teleport` — the replay the rod's queue drains through; `:463-478`
  the replay-owned camera.
- `player.rs:9498-9546` (g11 tip): `teleport_style` /
  `teleport_style_with_extra` / `queue_teleport_effect`; `:3431`
  `take_teleport_effect`.
- `world_sim/player.rs:1564-1576`/`:1691-1693` (g11 tip): the player-tick
  drain (the model for the editing-tail drain §2a).
- g89f: `rod_teleport.rs` (the rod machine), ANALYSIS "W4-g89f" (the cited
  TeleportEffect skip this note supersedes for the rod).

## Verification

Code-only: re-read `Main.cs:13233-13274` (the style-0/1 arms),
`Player.cs:37414-37496` (Teleport) / `:45476-45534` (the rod),
`app/frame/world.rs:627-670` (the phase order: players `:633` before editing
`:658`), and the g11-tip bodies cited above (`git show
origin/task/W4-g11-dust-main-sites-1:<path>` — re-pinned at `98283df08` for
§2a: `main_dust.rs:405/433/463-478`, `teleport_dust.rs:92-131`,
`player.rs:3431/9498-9546`, `world_sim/player.rs:1564/1691`,
`world_sim/editing.rs:1998` the recall arm — g11 tip has NO rod arm, `git
grep rod_item_check` empty). **No cargo/rustc/rustfmt/tests/graphify
invoked.** The §2a hunks (the editing-tail drain + the camera dedup + the
re-pinned chaos witness) are WRITTEN, NOT APPLIED — they are the g11/g89
merger's worklist; the teleport-dust seam is unreachable on g89's base (no
queue, no replay), so wiring on this branch is not possible without porting
g11's union (out of scope, g11's lane).