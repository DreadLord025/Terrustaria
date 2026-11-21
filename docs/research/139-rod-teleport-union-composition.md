# 139 — the RoD `teleport_style(1)` ⨉ TeleportEffect-union composition (W4-g89h)

Status: CROSS-NOTE for the g11 union merger + the g89 rod lane. Written by
W4-g89 (branch `task/W4-g89-aegis-fruit-use`, tip lineage `59864256` →
`e32be4cf`). Reads the EXECUTED union on `origin/task/W4-g11-dust-main-sites-1`
(tip `9a9ac1fd0`, execution `56afc0c01` + `8540a8ad5` + `9a9ac1fd0`, the
research/116 keep-one: g61's widest-11 body adapted into `teleport_dust.rs` +
g11's App-level `main_dust` seams dispatcher). Code-only: no cargo/tests/
rustfmt/graphify; the two merger actions below are WRITTEN, NOT APPLIED (they
are the g11/g89 merger's worklist, not this lane's edits — the union code does
not exist on g89's base `a725acb04`).

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
(`world_sim/editing.rs:2139`), which the frame orchestrator runs in
`editing_net_phase` (`app/frame/world.rs:658`) — AFTER `players_phase`
(`:633`) in the SAME fixed step. So the rod's queued effect is NOT drained
this tick; the NEXT tick's `players_phase` drains it.

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
Concretely: after the `rod_teleport::rod_item_check` call site
(`editing.rs:2139`), take `p.take_teleport_effect()` and run
`main_dust::player_teleport(app, req)` with `key = RngKey::UpdatePlayers`
(the same req shape the player-tick drain builds). The player-tick drain
stays for the player-tick callers; the two drains are disjoint per tick (a
caller queues in exactly one phase). The same-tick deferral of the MOVE is
already observation-identical (g11's contract — nothing probes the dust
between); only the RNG ORDER needs the editing-tail drain.

**Witness the merger must add:** a rod teleport with buff 88 up, pinning the
`UpdatePlayers` stream against a recording of [Entry-dust draws, Exit-dust
draws, Next(2)] in that order (the g89f chaos witness
`w4_g89f_the_chaos_tax_books_the_teleport_death_at_low_life` re-pins its
cloned-stream expectation to include the dust draws AHEAD of the Next(2)).

## 3. The camera double-set dedup (gap-5 closure ⨉ the rod's inline call)

g89f's `rod_teleport` starts the style-1 camera transition INLINE
(`rod_teleport.rs:121-126`, `start_camera_transition_for_player_teleport(old,
0.1, 10)` — the `Player.cs:45463` leg). g11's gap-5 closure moved that into
the replay (`main_dust.rs:435-450`) and RETIRED the wiring pad's inline arm.
At the union the rod's inline call would set the camera TWICE (inline this
tick + replay next tick) — the second `set_camera_lerp` resets
`camera_lerp_timer = 0` (`camera.rs:57-60`), re-arming the 10-tick speedup
delay one tick late.

**Fix (the merger's):** RETIRE the rod's inline camera call
(`rod_teleport.rs:121-126`) — the replay owns it for the rod too (the gap-5
"replay owns the camera for BOTH callers" contract). The rod keeps only
`p.teleport_style(point.x, point.y, 1)`. This is a g89-side edit the merger
applies when the union lands; it is NOT applied on this branch (the replay
does not exist on g89's base, so retiring the inline call now would drop the
camera entirely pre-union). Recorded here so the merger does it in the same
commit that wires the editing-tail drain (§2).

## 4. What stays g89's (no union interaction)

- The chaos-state tax block (`chaos_state_tax`, `Player.cs:45518-45533`) —
  pure player state + the ONE `Next(2)` draw; the union adds dust draws AHEAD
  of it on the stream (§2), the block itself is unchanged.
- The 5335 chaos-SKIP (`:45514-45517`) — the harmony rod teleports (style 1,
  dust fires) but never arms buff 88 / never pays the tax / never rolls the
  reason. The union's dust is identical for both rods (same style 1); only
  the chaos block differs.
- The aim gates / cadence / refusals (`:45478-45511`) — upstream of the
  teleport, no dust/RNG interaction.

## 5. Cross-references

- research/116 (g11): the 4-way union plan + the EXECUTED record (§4 of its
  tip) — the six gaps it closed; §2 above is the SEVENTH (the editing-phase
  caller), §3 the camera dedup the gap-5 closure forces on the rod.
- `teleport_dust.rs:90-129` (g11 tip): the style-1 arm (the `0 | 1` shape).
- `main_dust.rs:405-468` (g11 tip): `player_teleport` — the replay the rod's
  queue drains through; `:435-450` the replay-owned camera.
- `player.rs:9498-9538` (g11 tip): `teleport_style` /
  `teleport_style_with_extra` / `queue_teleport_effect`.
- `world_sim/player.rs:1564-1576`/`:1691-1693` (g11 tip): the player-tick
  drain (the model for the editing-tail drain §2).
- g89f: `rod_teleport.rs` (the rod machine), ANALYSIS "W4-g89f" (the cited
  TeleportEffect skip this note supersedes for the rod).

## Verification

Code-only: re-read `Main.cs:13233-13274` (the style-0/1 arms),
`Player.cs:37414-37496` (Teleport) / `:45476-45534` (the rod),
`app/frame/world.rs:627-670` (the phase order: players `:633` before editing
`:658`), and the g11-tip bodies cited above (`git show
origin/task/W4-g11-dust-main-sites-1:<path>`). **No cargo/rustc/rustfmt/tests/
graphify invoked.** The two merger actions (§2 editing-tail drain, §3 camera
dedup) + the re-pinned chaos witness are WRITTEN, NOT APPLIED — they are the
g11/g89 merger's worklist.