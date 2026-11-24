# 284 — g52c net final-audit: the relay-host arms census (W4-g52)

Status: CENSUS (code-only, g52c-net-final-audit RESUME round, branch
`task/W4-g52-net-relay-host`). Numbering: 284 = max-on-origin 283 + 1 (fresh
all-tip `docs/research/` sweep this round). **139 stays RESERVED for
g52c-net-master (the registry doc)** — research/150 (d54 tip) reserves it,
g89 renamed its own 139→145 (`41dabfbc8`), g50's cross-input note
(research/154 on its tip) does NOT take it, and THIS census does not take
it either. When the g52c-net-master sub-round lands its registry it takes
139 per the reservation; this 284 census is one of its feeds.

## 1. The relay-host arms census (this branch, tip lineage)

The g52 lane owns the SEND/RELAY side only; every client apply named below
stays g48-g51's lane. Arms landed across the lane's commits:

| arm | C# anchor | commit | port site |
| --- | --- | --- | --- |
| relay tail switch (34/69/20/23/28/13/27/default/targeted) | `NetMessage.cs:1692-1826` | `58a73a3b7` | `net.rs` `fanout_filtered`/`relay_*`, netglue wiring (ANALYSIS census table) |
| 21 pre-switch (instanced never syncs, IsAir→151) | `NetMessage.cs:89-100` | `c3fdfd543` | `item_send_gate` at the join burst + cadence diff/heartbeat |
| 162 damage-ack send (bodyless, at the 28 arm HEAD) | `MessageBuffer.cs:1768-1771` | `c3fdfd543` | `S2c::NpcStrikeAck` (d92 codec byte-verbatim) |
| 34 host place/destroy + relays | `MessageBuffer.cs:1995-2122` | `cdc8fa7b8` | `host_chest_updates`/`host_place_container`, `edit::place_chest`, `try_create_chest` |
| 34 destroy refinement (gate + store removal + drop) | `WorldGen.cs:63695`/`:63860`, `Chest.cs:609-645` | `e2a5968a4` | `host_destroy_container`: `can_kill_tile` + `chest_holds_items`, `remove_chest` + silent local close, object-drop spawn |
| join bundle tail: 23+54 per live NPC, 27 per pet/netImportant | `MessageBuffer.cs:851-866` | this round | `host_slot_join_complete` NPC/pet burst (cadence-store stamping) |
| SyncConnectedPlayer tail: 60 houses / 72 shop / 74 quest / 126 markers | `NetMessage.cs:2816-2819`, `:2824-2852`, `TownRoomManager.cs:130-142` | this round | same fn (the 140 credits push constants-off) |
| in-world 7 WorldInfo re-push apply (no spawn re-walk) | `MessageBuffer.cs:473-662` minus the `:655-659` State-3 gate | this round | `apply_world_info_repush` |
| live `world_info` fill (14 flag groups, bg block, ores, rain/wind/sandstorm) | `NetMessage.cs:224-412` | this round | `world_info` (port-owned bits only; the rest ride defaults) |
| 3600-tick header heartbeat | `Main.cs:64557-64561` | this round | `net_tick`'s `% 3600` arm broadcasts the LIVE header |
| dawn/dusk boundary re-push | `Main.cs:65359-65361`/`:65407-65409` | this round | `clock.rs` `advance_time_with_events` AFTER the event-flag clears |
| invasion-end re-push | `Main.cs:64357-64359` | this round | `npcs.rs` `update_invasion_state` |
| maxRaining-change re-push | `Main.cs:64952-64958` | this round | `weather.rs` `update_world_tick` (oldMaxRaining detector) |

## 2. The cross-lane keep-one feeds (for the g52c-net-master registry)

- **g48c-9460c85d FINAL-table feeds future-master.** The g48 lane's
  research/17 tail is SEALED at `9460c85d9` ("ids 1-40 wired per the FINAL
  census"); its `d3200ef3a` receipt records this lane's pushed-tip feed
  (the relay census deferring client applies to g48-g51). Later g48
  correction commits (`2b61530b4`, `9c776ca2f`) renumber docs only — the
  seal stands. The master registry consumes the 1-40 table AS the apply
  statuses feed; this 284 census is the SEND-side complement.
- **g52-e2a5968a destroy arm: merger takes g52's, keep-one.** Re-verified
  at both tips this round: g48's `net_apply.rs::host_chest_updates` destroy
  arms (`:920-957`) run the anchor decode + `find_chest` +
  `replicated_break` with NO `can_kill_tile`/`chest_holds_items` gate and
  NO store removal (the g48 `can_kill` closure at `:692` belongs to the
  trapdoor arm, not the 34). g52's `host_destroy_container` carries the
  full `Chest.CanDestroyChest` refusal + `remove_chest` + silent local
  close + the cascade object drop. KEEP g52's destroy arm.
- **34 codec byte-identical g48==g52.** Re-verified field-for-field at
  both tips: the `ChestUpdates` struct (action u8 + x/y/style/chest i16),
  the C2s write (`i16(0)` fifth short), the S2c write (`i16(c.chest)`),
  and both decode arms (host zeroes the fifth short, client keeps it) —
  `NetMessage.cs:913-927`. Only a doc-comment line-wrap differs. KEEP ONE
  copy at the union, either direction byte-safe.
- **162 lives g52+g51, NOT g48.** `S2c::NpcStrikeAck` (bodyless, id 162)
  exists on g52 (`c3fdfd543`) and g51 (`51d01bc87` pending-damage queue
  apply + FINAL `a8983fac2`); g48's net.rs has ZERO `NpcStrikeAck` hits.
  The merger keeps g51's apply arm with either codec copy.
- **g51 FINAL's 139 master input.** `a8983fac2` wires
  `DoesPlayerSlotCountAsAHost` (`NetMessage.cs:2876-2883`,
  `TcpAddress.cs:23-26`): `Server::slot_counts_as_host` (peer IP == IPv4
  loopback, State-10 gated) + the join-time 139 push. The g52 join arm's
  in-world `12` re-send comment (MessageBuffer.cs:903-948) marks the
  PlayerNet spawn fields as g48's keep-one — the 139 push rides g51's.
- **g50 cross-input (research/154 on g50's tip):** the 21 body's send-gate
  survivor is g52's; the 90 apply must read the landed `instanced` field;
  the 109 per-edit 20s must NOT ride the latched relay (the C# 17 path is
  ungated); 34 place/destroy vs 85 content-sync are disjoint ids.

## 3. This round's fix list (the frozen WIP completed + corrected)

The prior session's 4-file WIP (frozen 12:36) was verified line-by-line
against the C# and completed. Defects found and fixed this round:

1. **Dusk/dawn re-push ordering** (`clock.rs`): the WIP fired
   `net_world_repush` BEFORE the eclipse/blood-moon clears — the C#
   `stopMoonEvent` (`Main.cs:65246`/`:65392`) runs before its
   `SendData(7)`. Moved after the clears; the dusk witness now pins the
   CLEARED eclipse riding the header.
2. **Traveller flag gate** (`netglue.rs`): the WIP set `traveller_present`
   outside the `flag2` gate; the C# `:2841-2844` sets it INSIDE (type 368
   is aiStyle 7 so a live traveller always passes — behaviour equal, shape
   now verbatim).
3. **Pet-burst cadence stamping**: the WIP stamped `net_proj_prev` for
   EVERY pet/netImportant shot; the host cadence's live set is owner-gated
   (`Projectile.cs:17210`), so a foreign-owner pet stamped at join would
   read as "gone" next diff and fire a spurious `29`. Now stamped only for
   the host's own shots (the C# join burst has no owner gate `:861`, its
   cadence does — the split is the source's).
4. **Re-push apply's missing event homes**: added `Sandstorm.Happening`
   (`MessageBuffer.cs:576`), `BirthdayParty.ManualParty` (`:571`),
   `LanternNight.ManualLanterns` (`:583`) — the writer sends
   PartyIsUp/LanternsUp but the reader lands them on the MANUAL flags;
   the asymmetry is the source's, kept verbatim. The `flag8`
   StartSlimeRain/StopSlimeRain pair (`:641-648`) stays flag-only (the
   machines draw RNG; a re-push must not consume a stream — recorded
   deviation).
5. **Heartbeat witness loop count**: 620 frames ≈ 103 net ticks never
   reaches the 3600 counter; fixed to 3700 frames.
6. **Houses/shop/quest/marker witness**: drained an already-consumed
   channel (poll_net applies the frames through the no-op dispatcher arms
   first). Rewritten to observe a THIRD client's dispatcher input inline
   (drain-and-dispatch, watching until the last bundle message).
7. Citation re-pins: reader gate `:655-659` (was 654-658), raining derive
   `:530-531` (was 529-530), SyncConnectedPlayer tail `:2816-2819` (was
   2814-2818).

Verified-unchanged (the WIP's correct parts): the 14 flag-group bit maps
against `NetMessage.cs:285-389` (every port-owned bit matches), the
11-row bg block's 4/5/7/8/10 slots against `:246-258` + the port's
`set_surface_bg_style` 1/2/4/5/8 homes, the `!raining → wire 0`
(`:280-284`), the ore-tier order (`:392-398`), the State-3-only
windSpeedCurrent copy (`:655-659`), the house status 1/2 semantics
(`TownRoomManager.cs:130-142`: homeless=1, has-room=2), the join NPC/pet
loops (`:851-866`), and all four re-push site gates (`netMode == 2`).

## 4. Verification

Code-only: every arm re-read line-by-line against the C# anchors above;
the 34/162 codecs byte-compared at the g48/g51/g52 tips (git show +
field-for-field diff). 4 witness tests WRITTEN NOT RUN
(`netglue.rs::w4_g52_relay_tests`: the heartbeat, the dusk boundary, the
join NPC/pet burst, the houses/shop/quest/marker bundle) — **no compiler
invoked** (the lane's code-only contract). The lane's earlier rounds carry
13 + 9 witnesses WRITTEN NOT RUN (ANALYSIS).