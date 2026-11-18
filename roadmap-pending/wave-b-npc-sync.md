# ROADMAP pending update — S1-I-4 wave B (task/S1-I-4b)

Fold into `docs/ROADMAP.md`'s S1-I-4 row (line ~454) as a wave-progress note;
`docs/ROADMAP.md` is not edited on this branch (wave A, task/S1-I-4a, is
in-flight on the same row).

## Row update

S1-I-4 — wave B (**DONE 2026-11-18**, `task/S1-I-4b`): the NPC-STATE sync
subsystem's SendData cadence is mapped and ported into the new
`crates/tr-app/src/app/net_npc_sync.rs` (disjoint from wave A's `netglue.rs`
world-item region). The C# server does NOT broadcast the NPC set on a timer:
each NPC carries a `netSpam` throttle (`NPC.cs:6133-6149`:
`netSpamPacketLimit = 3`, `netSpamTicksPerPacket = 30`, bosses 5) and the
sends fan out from `NPC.UpdateNPC`'s tail (`NPC.cs:90513` →
`UpdateNetworkCode`, `NPC.cs:90583-90631`) — `Main.UpdateWorld_NPCs`
(`Main.cs:17898-17969`) itself sends nothing, it only walks
`NPC.UpdateNPC(k)` in slot order. Ported sites:

| id | C# site | Rust fn (`net_npc_sync.rs`) |
| --- | --- | --- |
| 23 cadence gate | `NPC.cs:90602-90618` (flag → `netSpam <= 3*num` → send, else pending-cooldown; -1/tick decay; town clamp `:90596`, inactive zero `:90600`) | `update_network_code` |
| 23 fan-out/cull | `NetMessage.cs:1719-1749` (priority = boss/netAlways/townNPC/!active/life<=0/spawnNeedsSyncing clears all counters; per-client 4-skip section cull) + targeted arm `:1814-1825` | `send_state` |
| 23 stream | `NPC.StreamUpdatesToNearbyPlayers` `NPC.cs:90633-90658` (`Main.npcStreamSpeed = 30` `Main.cs:66580`; distance bands 250/500/1000/1500 → +8/4/2/1, fire at 8; only `NPCID.Sets.UsesMultiplayerProximitySyncing` = 396/397/398, `NPCID.cs:10454`) | `stream_updates_to_nearby_players` |
| 23 recheck | `NPC.cs:90660-90670` (skipped + section re-activated → targeted 23) | `recheck_sections` |
| 23 death | `NPC.cs:90385-90392` (`life <= 0` → active=false → UpdateNetworkCode → netUpdate=false) | `npc_sync_tick` (inactive view) |
| 28 client strike | `NPC.cs:81199-81218` | `client_strike` |
| 28 cascade + relay | `NetMessage.cs:102-107` (spawnNeedsSyncing → full 23 FIRST) + `:1751-1762` (section-culled relay; all clients when life<=0) | `server_relay_strike` |
| 53/54 AddBuff | `NPC.cs:89926-89983` (client 53 BEFORE the mutation/early-return; server 54 only when written) | `add_buff` |
| 54 DelBuff | `NPC.cs:90002-90023` (compaction then server 54) | `del_buff` |
| 137 removal | `NPC.cs:89985-90000`; `BuffID.Sets.CanBeRemovedByNetMessage` is created EMPTY (`BuffID.cs:48`) — the 137 is gated off for every id in this decompilation | `request_buff_removal` / `can_be_removed_by_net_message` |
| 54 expired sweep | `NPC.cs:91890-91909` (server-only, ONE 54 per tick when any expired; per-slot DelBuff quiet) | `expired_buff_sweep` |
| 23+54 Transform | `NPC.cs:81131-81135` (23 then 54, in order) | `transform_sends` |
| 153 eel-whip DoT | `NPC.cs:91843-91862` (every 60th tick, server TrySendData(153, whoAmI, 50)) | `eel_whip_dot` |
| 60 town home | `NPC.cs:90621-90630` (change-detect vs oldHomeless/oldHomeTileX/Y, after the 23/recheck/stream) | `update_network_code` tail |
| helpers | `NetUpdateIgnoreSpamLimit` `NPC.cs:90568-90576`, `NetUpdateLowPriority` `:90578-90581` | `net_update_ignore_spam_limit` / `net_update_low_priority` |

Send ORDER is preserved exactly (the C# emission order per tick: the gated
23 → recheck 23s → stream 23s → town 60; the strike cascade's 23 before the
28; AddBuff's 53 before the mutation, 54 after). The module is a pure
decision layer over an `NpcSend` outbox — `net_tick` can later drain it
through the existing `S2c::NpcInfo`/`NpcStrike`/`NpcBuffs` wires; the wiring
itself is the follow-up (the current `net_tick` 10 Hz whole-set broadcast,
`netglue.rs:2075-2110`, stays until then — research/17 D18's `netSpam` gap is
now half-closed: the cadence is ported, the wire hookup is not).
No message-id table changes (I-1 in flight). 24 per-site tests.

## ANALYSIS.md note (fold into the netcode deviations section)

**S1-I-4b — the NPC send cadence is ported but not yet wired.**
`app/net_npc_sync.rs` ports `NPC.UpdateNetworkCode` + the `NetMessage`
`case 23`/`case 28` fan-out 1:1 (citations above) as pure functions; the live
`net_tick` still sends the whole NPC set at 10 Hz (`netglue.rs:2075-2110`)
until a follow-up drains the `NpcSend` outbox. Deliberate shape choices:
(1) the per-client `broadcast`/`IsConnected` gates ride a caller-supplied
`PlayerView` array (the port's `Server::slot_ids` + section bookkeeping map
onto it); (2) `NPC.oldHomeless/oldHomeTileX/Y` start at the C# `false/0/0`
defaults — the port's `old_home: None` fires one extra `60` on the very
first observation of a `(false, 0, 0)` home, which no head-index town NPC
ever has; (3) `CanBeRemovedByNetMessage` is empty in this decompilation
(`BuffID.cs:48`), so `137` is ported as a gated-off no-op, not omitted.

## PORT_MAP.md rows (fold)

| C# | Rust |
| --- | --- |
| `NPC.UpdateNetworkCode` (`NPC.cs:90583-90631`) | `app/net_npc_sync.rs::update_network_code` |
| `NPC.StreamUpdatesToNearbyPlayers` (`NPC.cs:90633-90658`) | `app/net_npc_sync.rs::stream_updates_to_nearby_players` |
| `NPC.RecheckSectionsForSkippedUpdates` (`NPC.cs:90660-90670`) | `app/net_npc_sync.rs::recheck_sections` |
| `NetMessage.SendData` case 23/28 fan-out (`NetMessage.cs:1719-1762`, `:102-107`) | `app/net_npc_sync.rs::send_state` / `server_relay_strike` |
| `NPC.AddBuff`/`DelBuff`/`RequestBuffRemoval` send sites (`NPC.cs:89926-90023`) | `app/net_npc_sync.rs::add_buff`/`del_buff`/`request_buff_removal` |
| `NPC.ApplyEelWhipDoT` (`NPC.cs:91843-91862`) | `app/net_npc_sync.rs::eel_whip_dot` |

## Verification

- `cargo test -p tr-app net_npc_sync` — `test result: ok. 24 passed; 0 failed`
- `cargo test --workspace -- --skip migrates_version_2_kinds_to_item_ids` —
  all `test result: ok` lines green (judged by the ok lines per AGENTS.md).
