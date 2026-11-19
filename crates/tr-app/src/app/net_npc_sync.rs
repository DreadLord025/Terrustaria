//! S1-I-4 wave B — the NPC-STATE sync subsystem's SendData cadence.
//!
//! The C# server does not broadcast the whole NPC set on a timer: every NPC
//! carries its own throttle (`NPC.netSpam`, `NPC.cs:6133-6149`) and the send
//! sites fan out from `NPC.UpdateNPC`'s tail (`NPC.cs:90513`) through
//! `UpdateNetworkCode` (`NPC.cs:90583-90631`), with the per-client culling in
//! `NetMessage.SendData`'s `case 23` / `case 28` arms
//! (`NetMessage.cs:1719-1762`). This module ports that cadence 1:1 as pure
//! decision functions over an outbox of [`NpcSend`]s, in the exact C# send
//! order.
//!
//! WIRED (S1-I-4d): `netglue.rs::npc_sync_cadence` steps `update_network_code`
//! per NPC per fixed step (host-only) and `flush_npc_outbox` drains the result
//! through the existing `S2c::NpcInfo` / `NpcHouse` / `NpcBuffs` wires. This
//! REPLACED the old whole-set 10 Hz broadcast — the two paths never both run.
//! No message-id table changes (I-1).
//!
//! Site map (the NPC life/ai/buff sync sites of the research/17 census):
//! - `23 SyncNPC` cadence gate: `NPC.cs:90602-90614` (netSpam throttle).
//! - `23` fan-out + section culling: `NetMessage.cs:1719-1749`; targeted
//!   `SendData(23, i)`: `NetMessage.cs:1814-1825`.
//! - `23` proximity stream: `NPC.StreamUpdatesToNearbyPlayers`
//!   (`NPC.cs:90633-90658`); skipped-section recheck (`NPC.cs:90660-90670`).
//! - `23` on death: `UpdateNPC`'s `life <= 0` arm (`NPC.cs:90385-90392`).
//! - `28 DamageNPC`: client strike (`NPC.cs:81218`), the server's
//!   `spawnNeedsSyncing` cascade (`NetMessage.cs:102-107`) and the `case 28`
//!   section-culled relay (`NetMessage.cs:1751-1762`).
//! - `53/54/137` buffs: `AddBuff` (`NPC.cs:89926-89983`), `DelBuff`
//!   (`NPC.cs:90002-90023`), `RequestBuffRemoval` (`NPC.cs:89985-90000`),
//!   the expired-buff sweep (`NPC.cs:91890-91909`), `Transform`'s pair
//!   (`NPC.cs:81131-81135`).
//! - `153 NPCHurtByDebuff`: `ApplyEelWhipDoT` (`NPC.cs:91843-91862`).
//! - `60 NPCHomeChange`: `UpdateNetworkCode`'s town tail
//!   (`NPC.cs:90621-90630`).
//! - The `Main.cs` UpdateNPCs loop itself sends nothing: `UpdateWorld_NPCs`
//!   (`Main.cs:17898-17969`) only calls `NPC.UpdateNPC(k)` per slot in index
//!   order — every NPC send rides the per-NPC tail above.

use glam::Vec2;

/// `NPC.netSpamPacketLimit` (`NPC.cs:6139`).
pub(crate) const NET_SPAM_PACKET_LIMIT: i32 = 3;
/// `NPC.netSpamTicksPerPacket` (`NPC.cs:6141`).
pub(crate) const NET_SPAM_TICKS_PER_PACKET: i32 = 30;
/// `NPC.netSpamTicksPerPacketForBosses` (`NPC.cs:6143`).
pub(crate) const NET_SPAM_TICKS_PER_PACKET_FOR_BOSSES: i32 = 5;
/// `Main.npcStreamSpeed`'s default (`Main.cs:66580`; the config override is
/// `Main.cs:5089`).
pub(crate) const NPC_STREAM_SPEED: i32 = 30;

/// `NPC.PlayerNetSyncState` (`NPC.cs:32-37`): the per-player stream/skip
/// counters, cleared by `Array.Clear` on a priority `23`
/// (`NetMessage.cs:1730`) and by a targeted send (`NetMessage.cs:1819`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct PlayerNetSyncState {
    pub skipped_syncs: u8,
    pub stream_counter: u8,
}

/// The per-NPC network fields (`NPC.cs:6133-6149`, `netStream`,
/// `spawnNeedsSyncing` `:6149`, `playerNetSyncState`). The reset values are
/// `SetDefaults`' (`NPC.cs:8531-8532` clears both pending flags).
#[derive(Clone, Debug)]
pub(crate) struct NpcNetState {
    /// `NPC.netUpdate` (`NPC.cs:6133`) — the AI sets it; `UpdateNPC`'s tail
    /// clears it after `UpdateNetworkCode` (`NPC.cs:90515`).
    pub net_update: bool,
    /// `NPC.netUpdatePendingSpamCooldown` (`NPC.cs:6135`).
    pub pending_spam_cooldown: bool,
    /// `NPC.netUpdatePendingFullSpamCooldown` (`NPC.cs:6137`).
    pub pending_full_spam_cooldown: bool,
    /// `NPC.netSpam` (`NPC.cs:6145`).
    pub net_spam: i32,
    /// `NPC.netStream` — the proximity-stream tick counter.
    pub net_stream: i32,
    /// `NPC.spawnNeedsSyncing` (`NPC.cs:6149`; set at `NPC.cs:80785`).
    pub spawn_needs_syncing: bool,
    /// `NPC.playerNetSyncState[256]`.
    pub sync: [PlayerNetSyncState; 256],
    /// `NPC.oldHomeless` / `oldHomeTileX` / `oldHomeTileY` — the `60`
    /// change-detect tail's previous values (`NPC.cs:90623-90629`).
    pub old_home: Option<(bool, i32, i32)>,
}

impl Default for NpcNetState {
    fn default() -> Self {
        Self {
            net_update: false,
            pending_spam_cooldown: false,
            pending_full_spam_cooldown: false,
            net_spam: 0,
            net_stream: 0,
            spawn_needs_syncing: false,
            sync: [PlayerNetSyncState::default(); 256],
            old_home: None,
        }
    }
}

/// The NPC fields the cadence reads. The caller (a future `net_tick` hook)
/// fills this from `crate::npc::Npc`; `boss`/`net_always` come from the
/// kind's `SetDefaults` (`NPC.cs:8716` etc.), `proximity_sync` from
/// `NPCID.Sets.UsesMultiplayerProximitySyncing` (`NPCID.cs:10454` — only
/// 396/397/398, the Martian saucer parts), `section` from
/// `NPC.NetSectionCoordinates` (`NPC.cs:6899`), `has_head_index` from
/// `NPC.TypeToDefaultHeadIndex(type) > 0` (`NPC.cs:90621`).
#[derive(Clone, Copy, Debug)]
pub(crate) struct NpcView {
    pub active: bool,
    pub life: i32,
    pub boss: bool,
    pub town_npc: bool,
    pub net_always: bool,
    pub proximity_sync: bool,
    pub vx: f32,
    pub vy: f32,
    pub center: Vec2,
    pub section: (i32, i32),
    pub has_head_index: bool,
    /// `(homeless, homeTileX, homeTileY)` — the town tail's current values.
    pub home: (bool, i32, i32),
}

/// One player slot as the cadence sees it: `Main.player[i].active`,
/// `ReportedCameraPosition` (`NPC.cs:90649`) and
/// `Netplay.Clients[i].IsSectionActive(npc.NetSectionCoordinates)`
/// (`NetMessage.cs:1740`). Index = the client id.
#[derive(Clone, Copy, Debug)]
pub(crate) struct PlayerView {
    pub active: bool,
    pub connected: bool,
    pub camera: Vec2,
    /// Precomputed per (player, NPC-section) by the caller.
    pub section_active: bool,
}

/// One queued send, in C# emission order. `to: None` is `remoteClient = -1`
/// (broadcast); `Some(i)` is the targeted `SendData(23, i, ...)` of the
/// stream/recheck sites. The bodies ride the existing `net.rs` messages.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum NpcSend {
    /// `23 SyncNPC` (`NetMessage.cs:686-748`).
    State { slot: i16, to: Option<u32> },
    /// `28 DamageNPC` (`NetMessage.cs:844-850`); `to: None` = the client's
    /// send-to-server, `Some(i)` = one leg of the server relay.
    Strike { slot: i16, to: Option<u32> },
    /// `54 NPCBuffs` — the full live list (`NetMessage.cs:1024-1039`).
    BuffList { slot: i16 },
    /// `53 AddNPCBuff` (`NetMessage.cs:1019-1022`).
    AddBuff { slot: i16, buff: u16, time: i16 },
    /// `137 RemoveNPCBuff` (`NetMessage.cs:1583`).
    RemoveBuff { slot: i16, buff: u16 },
    /// `153 NPCHurtByDebuff` (`NetMessage.cs:1646`).
    DebuffDot { slot: i16, amount: i32 },
    /// `60 NPCHomeChange` (`NetMessage.cs:1067-1071`).
    TownHome {
        slot: i16,
        homeless: bool,
        x: i32,
        y: i32,
    },
}

/// `NetMessage.SendData`'s `case 23` fan-out (`NetMessage.cs:1719-1749`)
/// and the targeted arm (`NetMessage.cs:1814-1825`). Mutates the NPC's
/// sync state exactly like the C#: a priority send (`flag4`) clears every
/// counter; a non-priority broadcast skips a client whose section is
/// inactive up to 4 times, then forces through; a targeted send always
/// lands and resets that client's state.
fn send_state(
    st: &mut NpcNetState,
    view: &NpcView,
    slot: i16,
    to: Option<u32>,
    players: &[PlayerView; 256],
    out: &mut Vec<NpcSend>,
) {
    if let Some(i) = to {
        // NetMessage.cs:1814-1825: targeted — reset the slot's state, send.
        if players[i as usize].connected {
            st.sync[i as usize] = PlayerNetSyncState::default();
            out.push(NpcSend::State { slot, to: Some(i) });
        }
        return;
    }
    // NetMessage.cs:1722: the priority test.
    let priority = view.boss
        || view.net_always
        || view.town_npc
        || !view.active
        || view.life <= 0
        || st.spawn_needs_syncing;
    if priority {
        // NetMessage.cs:1725-1730.
        st.spawn_needs_syncing = false;
        st.net_stream = 0;
        st.net_update = false;
        st.pending_spam_cooldown = false;
        st.pending_full_spam_cooldown = false;
        st.sync = [PlayerNetSyncState::default(); 256];
    }
    for (i, p) in players.iter().enumerate() {
        if !p.connected {
            continue; // NetMessage.cs:1734 (ignoreClient is -1 at every site here)
        }
        if !priority {
            // NetMessage.cs:1740-1744.
            if st.sync[i].skipped_syncs < 4 && !p.section_active {
                st.sync[i].skipped_syncs += 1;
                continue;
            }
            st.sync[i] = PlayerNetSyncState::default(); // NetMessage.cs:1745
        }
        out.push(NpcSend::State {
            slot,
            to: Some(i as u32),
        });
    }
}

/// `NPC.NetUpdateIgnoreSpamLimit` (`NPC.cs:90568-90576`).
pub(crate) fn net_update_ignore_spam_limit(st: &mut NpcNetState, boss: bool) {
    let num = if boss {
        NET_SPAM_TICKS_PER_PACKET_FOR_BOSSES
    } else {
        NET_SPAM_TICKS_PER_PACKET
    };
    if !st.net_update || st.net_spam > num * NET_SPAM_PACKET_LIMIT {
        st.net_spam -= num;
    }
    st.net_update = true;
}

/// `NPC.NetUpdateLowPriority` (`NPC.cs:90578-90581`).
pub(crate) fn net_update_low_priority(st: &mut NpcNetState) {
    st.pending_full_spam_cooldown = true;
}

/// `NPC.RecheckSectionsForSkippedUpdates` (`NPC.cs:90660-90670`): a client
/// that was skipped and whose section became active gets a targeted `23`.
fn recheck_sections(
    st: &mut NpcNetState,
    view: &NpcView,
    slot: i16,
    players: &[PlayerView; 256],
    out: &mut Vec<NpcSend>,
) {
    for (i, p) in players.iter().enumerate() {
        if st.sync[i].skipped_syncs > 0 && p.active && p.section_active {
            send_state(st, view, slot, Some(i as u32), players, out);
        }
    }
}

/// `NPC.StreamUpdatesToNearbyPlayers` (`NPC.cs:90633-90658`): the
/// distance-banded proximity stream for the saucer parts.
fn stream_updates_to_nearby_players(
    st: &mut NpcNetState,
    view: &NpcView,
    slot: i16,
    players: &[PlayerView; 256],
    npc_stream_speed: i32,
    out: &mut Vec<NpcSend>,
) {
    // NPC.cs:90635: the four-way gate (the C# double compare of the sum).
    if npc_stream_speed <= 0
        || view.town_npc
        || !view.proximity_sync
        || (view.vx.abs() + view.vy.abs()) as f64 <= 0.5
    {
        return;
    }
    st.net_stream += 1; // NPC.cs:90639
    if st.net_stream < npc_stream_speed {
        return;
    }
    st.net_stream = 0; // NPC.cs:90644
    for (i, p) in players.iter().enumerate() {
        if !p.active {
            continue; // NPC.cs:90647
        }
        let dist = (view.center - p.camera).length();
        // NPC.cs:90650: boss 8, else the 250/500/1000/1500 px bands.
        let add: u8 = if view.boss {
            8
        } else if dist < 250.0 {
            8
        } else if dist < 500.0 {
            4
        } else if dist < 1000.0 {
            2
        } else if dist < 1500.0 {
            1
        } else {
            0
        };
        st.sync[i].stream_counter += add; // NPC.cs:90651 (byte add)
        if st.sync[i].stream_counter >= 8 {
            // NPC.cs:90654 — the targeted send resets the counter via
            // NetMessage.cs:1819.
            send_state(st, view, slot, Some(i as u32), players, out);
        }
    }
}

/// `NPC.UpdateNetworkCode` (`NPC.cs:90583-90631`) — the per-tick cadence
/// gate. Call once per NPC per fixed step, in slot order (the
/// `Main.UpdateWorld_NPCs` loop, `Main.cs:17948-17952`); the caller clears
/// `net_update` afterwards (`NPC.cs:90515`, see [`npc_sync_tick`]).
pub(crate) fn update_network_code(
    st: &mut NpcNetState,
    view: &NpcView,
    slot: i16,
    net_mode: u8,
    players: &[PlayerView; 256],
    npc_stream_speed: i32,
    out: &mut Vec<NpcSend>,
) {
    // NPC.cs:90586-90589: an inactive slot always wants a (despawn) update.
    if !view.active {
        st.net_update = true;
    }
    if net_mode != 2 {
        return; // NPC.cs:90590-90593
    }
    if view.town_npc {
        // NPC.cs:90594-90597: town NPCs never bank more than one packet.
        st.net_spam = st.net_spam.min(NET_SPAM_TICKS_PER_PACKET);
    }
    if !view.active {
        st.net_spam = 0; // NPC.cs:90598-90601
    }
    // NPC.cs:90602-90614: the throttle.
    if st.net_update
        || st.pending_spam_cooldown
        || (st.pending_full_spam_cooldown && st.net_spam == 0)
    {
        let num = if view.boss {
            NET_SPAM_TICKS_PER_PACKET_FOR_BOSSES
        } else {
            NET_SPAM_TICKS_PER_PACKET
        };
        if st.net_spam <= NET_SPAM_PACKET_LIMIT * num {
            st.net_spam += num;
            send_state(st, view, slot, None, players, out); // NPC.cs:90608
        } else {
            st.pending_spam_cooldown = true; // NPC.cs:90612
        }
    }
    if st.net_spam > 0 {
        st.net_spam -= 1; // NPC.cs:90615-90618
    }
    // NPC.cs:90619-90620: the two per-player passes, in this order.
    recheck_sections(st, view, slot, players, out);
    stream_updates_to_nearby_players(st, view, slot, players, npc_stream_speed, out);
    // NPC.cs:90621-90630: the town home change-detect tail.
    if view.active && view.town_npc && view.has_head_index {
        let (homeless, hx, hy) = view.home;
        if st.old_home != Some((homeless, hx, hy)) {
            out.push(NpcSend::TownHome {
                slot,
                homeless,
                x: hx,
                y: hy,
            });
        }
        st.old_home = Some((homeless, hx, hy));
    }
}

/// The `UpdateNPC` tail hook `net_tick` can call: per slot, in index order,
/// `UpdateNetworkCode(i)` then `netUpdate = false` (`NPC.cs:90513-90515`).
/// The death arm (`NPC.cs:90385-90392`) is the same pair with `active`
/// already false — the caller marks the view inactive and this tick emits
/// the despawn `23` (priority via `!active`, `NetMessage.cs:1722`).
pub(crate) fn npc_sync_tick(
    states: &mut [NpcNetState],
    views: &[NpcView],
    net_mode: u8,
    players: &[PlayerView; 256],
    npc_stream_speed: i32,
) -> Vec<NpcSend> {
    let mut out = Vec::new();
    for (i, (st, view)) in states.iter_mut().zip(views.iter()).enumerate() {
        update_network_code(
            st,
            view,
            i as i16,
            net_mode,
            players,
            npc_stream_speed,
            &mut out,
        );
        st.net_update = false; // NPC.cs:90515 (and :90389 on the death arm)
    }
    out
}

/// `NPC.StrikeNPC`'s client send (`NPC.cs:81204-81218`): a non-simulated
/// strike goes to the server as `28` before the local damage applies.
pub(crate) fn client_strike(
    st: &mut NpcNetState,
    view: &NpcView,
    slot: i16,
    net_mode: u8,
    out: &mut Vec<NpcSend>,
) {
    let _ = st;
    if !view.active || view.life <= 0 {
        return; // NPC.cs:81199-81202
    }
    if net_mode != 0 {
        out.push(NpcSend::Strike { slot, to: None }); // NPC.cs:81218
    }
}

/// The server's `28` path: `SendData`'s pre-switch cascade
/// (`NetMessage.cs:102-107` — a `spawnNeedsSyncing` NPC gets a full `23`
/// FIRST) then the `case 28` relay (`NetMessage.cs:1751-1762` — every
/// connected client whose section is active, or all when the NPC died).
pub(crate) fn server_relay_strike(
    st: &mut NpcNetState,
    view: &NpcView,
    slot: i16,
    players: &[PlayerView; 256],
    out: &mut Vec<NpcSend>,
) {
    if st.spawn_needs_syncing {
        send_state(st, view, slot, None, players, out); // NetMessage.cs:105
    }
    for (i, p) in players.iter().enumerate() {
        if p.connected && (view.life <= 0 || p.section_active) {
            out.push(NpcSend::Strike {
                slot,
                to: Some(i as u32),
            }); // NetMessage.cs:1756-1758
        }
    }
}

/// `NPC.AddBuff`'s send sites (`NPC.cs:89926-89983`). `apply` runs the buff
/// mutation between the two sends and returns whether the buff was actually
/// written (the C# early-returns on immunity / a longer existing time
/// BEFORE the server's `54`, but AFTER the client's `53`).
pub(crate) fn add_buff(
    slot: i16,
    buff: u16,
    time: i16,
    net_mode: u8,
    quiet: bool,
    out: &mut Vec<NpcSend>,
    apply: impl FnOnce() -> bool,
) {
    if !quiet && net_mode == 1 {
        out.push(NpcSend::AddBuff { slot, buff, time }); // NPC.cs:89932-89935
    }
    let written = apply();
    if written && !quiet && net_mode == 2 {
        out.push(NpcSend::BuffList { slot }); // NPC.cs:89979-89982
    }
}

/// `NPC.DelBuff`'s send site (`NPC.cs:90002-90023`): the compaction runs
/// first, then the server pushes the full list.
pub(crate) fn del_buff(
    slot: i16,
    net_mode: u8,
    quiet: bool,
    out: &mut Vec<NpcSend>,
    compact: impl FnOnce(),
) {
    compact();
    if !quiet && net_mode == 2 {
        out.push(NpcSend::BuffList { slot }); // NPC.cs:90019-90022
    }
}

/// `BuffID.Sets.CanBeRemovedByNetMessage` (`BuffID.cs:48`): the set is
/// created EMPTY in this decompilation and never filled — the `137` request
/// is gated off for every buff id.
pub(crate) fn can_be_removed_by_net_message(_buff: u16) -> bool {
    false
}

/// `NPC.RequestBuffRemoval` (`NPC.cs:89985-90000`): gate → find → `DelBuff`
/// (local, quiet-by-omission: the C# calls the non-quiet overload, so on a
/// server it would also push `54`) → the client's `137`.
pub(crate) fn request_buff_removal(
    slot: i16,
    buff: u16,
    net_mode: u8,
    found: bool,
    out: &mut Vec<NpcSend>,
    compact: impl FnOnce(),
) {
    if !can_be_removed_by_net_message(buff) {
        return; // NPC.cs:89987-89990
    }
    if found {
        del_buff(slot, net_mode, false, out, compact); // NPC.cs:89994
        if net_mode == 1 {
            out.push(NpcSend::RemoveBuff { slot, buff }); // NPC.cs:89995-89998
        }
    }
}

/// `UpdateNPC_BuffClearExpiredBuffs` (`NPC.cs:91890-91909`): the sweep runs
/// server-side only and pushes ONE `54` when at least one buff expired —
/// the per-slot `DelBuff`s are quiet.
pub(crate) fn expired_buff_sweep(
    slot: i16,
    net_mode: u8,
    any_expired: bool,
    out: &mut Vec<NpcSend>,
) {
    if net_mode == 1 {
        return; // NPC.cs:91892-91895
    }
    if any_expired && net_mode == 2 {
        out.push(NpcSend::BuffList { slot }); // NPC.cs:91905-91908
    }
}

/// `NPC.Transform`'s pair (`NPC.cs:81131-81135`): `23` then `54`, in that
/// order, server-only.
pub(crate) fn transform_sends(
    st: &mut NpcNetState,
    view: &NpcView,
    slot: i16,
    net_mode: u8,
    players: &[PlayerView; 256],
    out: &mut Vec<NpcSend>,
) {
    if net_mode == 2 {
        send_state(st, view, slot, None, players, out); // NPC.cs:81133
        out.push(NpcSend::BuffList { slot }); // NPC.cs:81134
    }
}

/// `NPC.ApplyEelWhipDoT` (`NPC.cs:91843-91862`): every 60th tick the
/// server applies 50 debuff damage and sends `153`. Returns the damage
/// amount when the tick fires (the caller runs `GetHurtByDebuff`).
pub(crate) fn eel_whip_dot(
    counter: &mut i32,
    slot: i16,
    net_mode: u8,
    out: &mut Vec<NpcSend>,
) -> Option<i32> {
    *counter += 1; // NPC.cs:91847 (++electricEelCounter)
    if *counter < 60 {
        return None;
    }
    *counter = 0; // NPC.cs:91851
    if net_mode == 1 {
        return None; // NPC.cs:91853
    }
    if net_mode == 2 {
        out.push(NpcSend::DebuffDot { slot, amount: 50 }); // NPC.cs:91857-91860
    }
    Some(50)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn players_none() -> [PlayerView; 256] {
        [PlayerView {
            active: false,
            connected: false,
            camera: Vec2::ZERO,
            section_active: false,
        }; 256]
    }

    fn players_two() -> [PlayerView; 256] {
        let mut p = players_none();
        p[0] = PlayerView {
            active: true,
            connected: true,
            camera: Vec2::ZERO,
            section_active: true,
        };
        p[1] = PlayerView {
            active: true,
            connected: true,
            camera: Vec2::ZERO,
            section_active: true,
        };
        p
    }

    fn mob() -> NpcView {
        NpcView {
            active: true,
            life: 50,
            boss: false,
            town_npc: false,
            net_always: false,
            proximity_sync: false,
            vx: 1.0,
            vy: 0.0,
            center: Vec2::ZERO,
            section: (0, 0),
            has_head_index: false,
            home: (true, 0, 0),
        }
    }

    fn states(n: usize) -> Vec<NpcNetState> {
        (0..n).map(|_| NpcNetState::default()).collect()
    }

    // --- the 23 cadence gate (NPC.cs:90602-90618) ---

    #[test]
    fn a_single_flag_sends_one_23_then_the_spam_decays_silently() {
        let p = players_two();
        let mut st = NpcNetState::default();
        let v = mob();
        let mut out = Vec::new();
        st.net_update = true;
        update_network_code(&mut st, &v, 0, 2, &p, NPC_STREAM_SPEED, &mut out);
        // One 23 per connected client, netSpam = 30 - 1 (NPC.cs:90607/90617).
        assert_eq!(
            out,
            vec![
                NpcSend::State {
                    slot: 0,
                    to: Some(0)
                },
                NpcSend::State {
                    slot: 0,
                    to: Some(1)
                },
            ]
        );
        assert_eq!(st.net_spam, 29);
        // No further flags: the tail clears netUpdate (NPC.cs:90515), the
        // counter decays 1/tick, nothing sends.
        for _ in 0..29 {
            out.clear();
            st.net_update = false; // the UpdateNPC tail, NPC.cs:90515
            update_network_code(&mut st, &v, 0, 2, &p, NPC_STREAM_SPEED, &mut out);
            assert!(
                out.is_empty(),
                "sent without a flag (netSpam {})",
                st.net_spam
            );
        }
        assert_eq!(st.net_spam, 0);
    }

    #[test]
    fn a_continuously_flagged_mob_bursts_four_packets_then_throttles_to_one_per_30() {
        let p = players_two();
        let mut st = NpcNetState::default();
        let v = mob();
        let mut out = Vec::new();
        // The gate is netSpam <= 3*30 (NPC.cs:90605): four back-to-back
        // flagged ticks all send (spam 29/58/87/116 after the decrement).
        for k in 0..4 {
            out.clear();
            st.net_update = true;
            update_network_code(&mut st, &v, 0, 2, &p, NPC_STREAM_SPEED, &mut out);
            assert_eq!(out.len(), 2, "burst send {}", k + 1);
        }
        assert_eq!(st.net_spam, 116);
        // Tick 5 defers (116 > 90) and arms the pending cooldown.
        out.clear();
        st.net_update = true;
        update_network_code(&mut st, &v, 0, 2, &p, NPC_STREAM_SPEED, &mut out);
        assert!(out.is_empty());
        assert!(st.pending_spam_cooldown);
        // The pending flag retries every tick; the next send lands when the
        // decayed counter reaches 90 again — 26 ticks later (tick 31).
        let mut next_send = None;
        for t in 6..=40 {
            out.clear();
            st.net_update = true;
            update_network_code(&mut st, &v, 0, 2, &p, NPC_STREAM_SPEED, &mut out);
            if !out.is_empty() {
                next_send = Some(t);
                break;
            }
        }
        assert_eq!(
            next_send,
            Some(31),
            "sustained rate is one packet per 30 ticks"
        );
    }

    #[test]
    fn a_boss_uses_the_5_tick_packet_cost() {
        let p = players_two();
        let mut st = NpcNetState::default();
        let mut v = mob();
        v.boss = true;
        let mut out = Vec::new();
        st.net_update = true;
        update_network_code(&mut st, &v, 0, 2, &p, NPC_STREAM_SPEED, &mut out);
        assert_eq!(st.net_spam, 4); // +5 then -1 (NPC.cs:90604)
        assert_eq!(out.len(), 2);
        // Continuously flagged: the 3*5 gate lets 4 packets burst, then the
        // sustained rate is one per 5 ticks.
        for k in 0..3 {
            out.clear();
            st.net_update = true;
            update_network_code(&mut st, &v, 0, 2, &p, NPC_STREAM_SPEED, &mut out);
            assert_eq!(out.len(), 2, "boss burst send {}", k + 2);
        }
        assert_eq!(st.net_spam, 16); // 4 packets * 5 - 4 decrements
        out.clear();
        st.net_update = true;
        update_network_code(&mut st, &v, 0, 2, &p, NPC_STREAM_SPEED, &mut out);
        assert!(out.is_empty(), "16 > 15 defers");
        out.clear();
        st.net_update = true;
        update_network_code(&mut st, &v, 0, 2, &p, NPC_STREAM_SPEED, &mut out);
        assert_eq!(out.len(), 2, "spam decayed to 15 -> the 6th tick sends");
    }

    #[test]
    fn the_spam_limit_defers_a_burst_until_the_cooldown_drains() {
        let p = players_two();
        let mut st = NpcNetState::default();
        let v = mob();
        let mut out = Vec::new();
        // Four back-to-back forced sends bank 4*30; the 4th exceeds the
        // limit*30 gate AFTER the send (netSpam 90 > 90 is false at 90, so
        // the 4th still sends; the 5th defers). NPC.cs:90605.
        for k in 0..4 {
            st.net_update = true;
            update_network_code(&mut st, &v, 0, 2, &p, NPC_STREAM_SPEED, &mut out);
            assert_eq!(out.len(), 2 * (k + 1), "send {} went through", k + 1);
        }
        assert_eq!(st.net_spam, 4 * 30 - 4);
        out.clear();
        st.net_update = true;
        update_network_code(&mut st, &v, 0, 2, &p, NPC_STREAM_SPEED, &mut out);
        assert!(out.is_empty(), "the 5th burst send is deferred");
        assert!(st.pending_spam_cooldown);
        // The pending flag keeps trying every tick until the gate opens.
        let mut sent = false;
        for _ in 0..40 {
            out.clear();
            update_network_code(&mut st, &v, 0, 2, &p, NPC_STREAM_SPEED, &mut out);
            if !out.is_empty() {
                sent = true;
                break;
            }
        }
        assert!(sent, "the deferred update lands once netSpam drains");
    }

    #[test]
    fn ignore_spam_limit_and_low_priority_match_the_csharp_helpers() {
        let mut st = NpcNetState::default();
        // NPC.cs:90571: netUpdate false -> subtract one packet's worth.
        st.net_spam = 30;
        net_update_ignore_spam_limit(&mut st, false);
        assert_eq!(st.net_spam, 0);
        assert!(st.net_update);
        // netUpdate true and under the limit -> no subtraction.
        st.net_spam = 30;
        net_update_ignore_spam_limit(&mut st, false);
        assert_eq!(st.net_spam, 30);
        // Over the limit -> subtract even when flagged (boss window 5).
        st.net_spam = 16;
        net_update_ignore_spam_limit(&mut st, true);
        assert_eq!(st.net_spam, 11);
        // Low priority only arms the full-cooldown flag (NPC.cs:90580).
        let mut st = NpcNetState::default();
        net_update_low_priority(&mut st);
        assert!(st.pending_full_spam_cooldown && !st.net_update);
        // It fires only at netSpam == 0 (NPC.cs:90602).
        let p = players_two();
        let v = mob();
        let mut out = Vec::new();
        st.net_spam = 3;
        update_network_code(&mut st, &v, 0, 2, &p, NPC_STREAM_SPEED, &mut out);
        assert!(out.is_empty());
        st.net_spam = 0;
        update_network_code(&mut st, &v, 0, 2, &p, NPC_STREAM_SPEED, &mut out);
        assert_eq!(out.len(), 2, "low-priority update fires at netSpam 0");
    }

    #[test]
    fn single_player_and_client_run_no_server_cadence() {
        let p = players_two();
        let mut st = NpcNetState::default();
        let v = mob();
        let mut out = Vec::new();
        st.net_update = true;
        update_network_code(&mut st, &v, 0, 0, &p, NPC_STREAM_SPEED, &mut out);
        update_network_code(&mut st, &v, 0, 1, &p, NPC_STREAM_SPEED, &mut out);
        assert!(out.is_empty(), "netMode 0/1 never run the server gate");
        // The inactive flag still arms on the client (NPC.cs:90586-90589).
        let mut dead = mob();
        dead.active = false;
        st.net_update = false;
        update_network_code(&mut st, &dead, 0, 1, &p, NPC_STREAM_SPEED, &mut out);
        assert!(st.net_update);
    }

    #[test]
    fn a_dead_npc_despawns_immediately_and_clears_the_throttle() {
        let p = players_two();
        let mut st = NpcNetState::default();
        st.net_spam = 87; // mid-window
        let mut v = mob();
        v.active = false;
        v.life = 0;
        let mut states = vec![st.clone()];
        let out = npc_sync_tick(&mut states, &[v], 2, &p, NPC_STREAM_SPEED);
        // NPC.cs:90598-90601 zeroes netSpam; the priority arm sends at once.
        assert_eq!(
            out,
            vec![
                NpcSend::State {
                    slot: 0,
                    to: Some(0)
                },
                NpcSend::State {
                    slot: 0,
                    to: Some(1)
                },
            ]
        );
        assert_eq!(states[0].net_spam, 29); // 0 + 30 - 1
        assert!(
            !states[0].net_update,
            "the tail clears netUpdate (NPC.cs:90389/90515)"
        );
    }

    // --- the 23 fan-out culling (NetMessage.cs:1719-1749) ---

    #[test]
    fn an_inactive_section_is_skipped_four_times_then_forced() {
        let mut p = players_two();
        p[1].section_active = false;
        let mut st = NpcNetState::default();
        let v = mob();
        let mut out = Vec::new();
        for k in 0..4 {
            out.clear();
            st.net_update = true;
            st.net_spam = 0;
            update_network_code(&mut st, &v, 0, 2, &p, NPC_STREAM_SPEED, &mut out);
            assert_eq!(
                out,
                vec![NpcSend::State {
                    slot: 0,
                    to: Some(0)
                }],
                "tick {k}"
            );
            assert_eq!(st.sync[1].skipped_syncs, (k + 1) as u8);
        }
        out.clear();
        st.net_update = true;
        st.net_spam = 0;
        update_network_code(&mut st, &v, 0, 2, &p, NPC_STREAM_SPEED, &mut out);
        // The 5th send forces through and resets the counter (NetMessage.cs:1745).
        assert_eq!(
            out,
            vec![
                NpcSend::State {
                    slot: 0,
                    to: Some(0)
                },
                NpcSend::State {
                    slot: 0,
                    to: Some(1)
                },
            ]
        );
        assert_eq!(st.sync[1], PlayerNetSyncState::default());
    }

    #[test]
    fn a_priority_npc_bypasses_the_section_cull_and_clears_the_counters() {
        let mut p = players_two();
        p[1].section_active = false;
        let mut st = NpcNetState::default();
        st.sync[1].skipped_syncs = 3;
        st.spawn_needs_syncing = true;
        st.pending_spam_cooldown = true;
        let v = mob();
        let mut out = Vec::new();
        st.net_update = true;
        update_network_code(&mut st, &v, 0, 2, &p, NPC_STREAM_SPEED, &mut out);
        assert_eq!(out.len(), 2, "spawnNeedsSyncing forces both clients");
        assert!(!st.spawn_needs_syncing && !st.pending_spam_cooldown);
        assert_eq!(st.sync[1], PlayerNetSyncState::default());
    }

    #[test]
    fn a_skipped_client_gets_a_targeted_23_when_its_section_activates() {
        let mut p = players_two();
        p[1].section_active = false;
        let mut st = NpcNetState::default();
        st.sync[1].skipped_syncs = 2;
        let v = mob();
        let mut out = Vec::new();
        // No cadence send this tick; only the recheck fires (NPC.cs:90665).
        p[1].section_active = true;
        update_network_code(&mut st, &v, 0, 2, &p, NPC_STREAM_SPEED, &mut out);
        assert_eq!(
            out,
            vec![NpcSend::State {
                slot: 0,
                to: Some(1)
            }]
        );
        assert_eq!(st.sync[1], PlayerNetSyncState::default());
    }

    // --- the proximity stream (NPC.cs:90633-90658) ---

    #[test]
    fn the_stream_fires_every_30_ticks_with_distance_banded_counters() {
        let mut p = players_none();
        // near (<250): +8 -> fires on the first stream tick;
        // mid (<500): +4 -> every 2nd; far (<1000): +2 -> every 4th.
        p[0] = PlayerView {
            active: true,
            connected: true,
            camera: Vec2::ZERO,
            section_active: true,
        };
        p[1] = PlayerView {
            active: true,
            connected: true,
            camera: Vec2::new(300.0, 0.0),
            section_active: true,
        };
        p[2] = PlayerView {
            active: true,
            connected: true,
            camera: Vec2::new(700.0, 0.0),
            section_active: true,
        };
        let mut v = mob();
        v.proximity_sync = true; // NPCID.cs:10454 (396/397/398)
        let mut st = NpcNetState::default();
        let mut out = Vec::new();
        for _ in 0..29 {
            update_network_code(&mut st, &v, 0, 2, &p, NPC_STREAM_SPEED, &mut out);
        }
        assert!(out.is_empty(), "no stream send before tick 30");
        update_network_code(&mut st, &v, 0, 2, &p, NPC_STREAM_SPEED, &mut out);
        assert_eq!(
            out,
            vec![NpcSend::State {
                slot: 0,
                to: Some(0)
            }]
        );
        assert_eq!(st.sync[1].stream_counter, 4);
        assert_eq!(st.sync[2].stream_counter, 2);
        // Second stream tick (tick 60): the mid player fires.
        out.clear();
        for _ in 0..30 {
            update_network_code(&mut st, &v, 0, 2, &p, NPC_STREAM_SPEED, &mut out);
        }
        assert_eq!(
            out,
            vec![
                NpcSend::State {
                    slot: 0,
                    to: Some(0)
                },
                NpcSend::State {
                    slot: 0,
                    to: Some(1)
                },
            ]
        );
        assert_eq!(st.sync[2].stream_counter, 4);
    }

    #[test]
    fn the_stream_gate_needs_speed_non_town_proximity_and_motion() {
        let mut p = players_two();
        p[0].camera = Vec2::ZERO;
        let mut st = NpcNetState::default();
        st.net_stream = 29; // one tick from firing
        let mut out = Vec::new();
        let mut v = mob();
        v.proximity_sync = true;
        v.vx = 0.25;
        v.vy = 0.25; // sum 0.5 -> the <= 0.5 gate blocks (NPC.cs:90635)
        update_network_code(&mut st, &v, 0, 2, &p, NPC_STREAM_SPEED, &mut out);
        assert!(out.is_empty() && st.net_stream == 29);
        v.vx = 0.6;
        v.town_npc = true; // town NPCs never stream
        update_network_code(&mut st, &v, 0, 2, &p, NPC_STREAM_SPEED, &mut out);
        assert!(out.is_empty());
        v.town_npc = false;
        update_network_code(&mut st, &v, 0, 2, &p, 0, &mut out);
        assert!(out.is_empty(), "npcStreamSpeed 0 disables the stream");
        update_network_code(&mut st, &v, 0, 2, &p, NPC_STREAM_SPEED, &mut out);
        assert_eq!(
            out.len(),
            2,
            "moving proximity NPC streams to both near players"
        );
        assert_eq!(st.net_stream, 0);
    }

    // --- the 28 strike path (NPC.cs:81218, NetMessage.cs:102-107/1751-1762) ---

    #[test]
    fn a_client_strike_sends_28_and_a_dead_npc_sends_nothing() {
        let mut st = NpcNetState::default();
        let v = mob();
        let mut out = Vec::new();
        client_strike(&mut st, &v, 3, 1, &mut out);
        assert_eq!(out, vec![NpcSend::Strike { slot: 3, to: None }]);
        out.clear();
        client_strike(&mut st, &v, 3, 0, &mut out);
        assert!(out.is_empty(), "single-player never sends");
        let mut dead = mob();
        dead.life = 0;
        client_strike(&mut st, &dead, 3, 1, &mut out);
        assert!(out.is_empty(), "NPC.cs:81199-81202 blocks a dead target");
    }

    #[test]
    fn the_server_cascades_a_full_23_before_the_28_when_spawn_needs_syncing() {
        let mut p = players_two();
        p[1].section_active = false;
        let mut st = NpcNetState::default();
        st.spawn_needs_syncing = true;
        let v = mob();
        let mut out = Vec::new();
        server_relay_strike(&mut st, &v, 5, &p, &mut out);
        // Order: the priority 23 to BOTH clients (NetMessage.cs:105), then
        // the 28 only to the section-active client (NetMessage.cs:1756).
        assert_eq!(
            out,
            vec![
                NpcSend::State {
                    slot: 5,
                    to: Some(0)
                },
                NpcSend::State {
                    slot: 5,
                    to: Some(1)
                },
                NpcSend::Strike {
                    slot: 5,
                    to: Some(0)
                },
            ]
        );
        assert!(!st.spawn_needs_syncing);
    }

    #[test]
    fn a_killing_strike_relays_to_every_client_regardless_of_section() {
        let mut p = players_two();
        p[1].section_active = false;
        let mut st = NpcNetState::default();
        let mut v = mob();
        v.life = 0;
        let mut out = Vec::new();
        server_relay_strike(&mut st, &v, 5, &p, &mut out);
        assert_eq!(
            out,
            vec![
                NpcSend::Strike {
                    slot: 5,
                    to: Some(0)
                },
                NpcSend::Strike {
                    slot: 5,
                    to: Some(1)
                },
            ]
        );
    }

    // --- the buff sites (NPC.cs:89926-90023, 91890-91909, 81131-81135) ---

    #[test]
    fn add_buff_sends_53_before_the_mutation_on_a_client_and_54_after_on_a_server() {
        let mut log: Vec<&str> = Vec::new();
        let mut out = Vec::new();
        add_buff(2, 20, 300, 1, false, &mut out, || {
            log.push("apply");
            true
        });
        assert_eq!(
            out,
            vec![NpcSend::AddBuff {
                slot: 2,
                buff: 20,
                time: 300
            }]
        );
        assert_eq!(log, vec!["apply"]);
        out.clear();
        add_buff(2, 20, 300, 2, false, &mut out, || true);
        assert_eq!(out, vec![NpcSend::BuffList { slot: 2 }]);
        // The C# early-return (a longer existing buff) skips the server 54
        // but NOT the client 53 (NPC.cs:89941-89944 sits after :89934).
        out.clear();
        add_buff(2, 20, 100, 2, false, &mut out, || false);
        assert!(out.is_empty());
        out.clear();
        add_buff(2, 20, 100, 1, false, &mut out, || false);
        assert_eq!(out.len(), 1);
        // quiet suppresses both (NPC.cs:89932/89979).
        out.clear();
        add_buff(2, 20, 100, 2, true, &mut out, || true);
        add_buff(2, 20, 100, 1, true, &mut out, || true);
        assert!(out.is_empty());
    }

    #[test]
    fn del_buff_pushes_the_full_list_after_the_compaction_on_the_server() {
        let mut order: Vec<&str> = Vec::new();
        let mut out = Vec::new();
        del_buff(4, 2, false, &mut out, || order.push("compact"));
        assert_eq!(order, vec!["compact"]);
        assert_eq!(out, vec![NpcSend::BuffList { slot: 4 }]);
        out.clear();
        del_buff(4, 1, false, &mut out, || {});
        del_buff(4, 2, true, &mut out, || {});
        assert!(out.is_empty(), "client and quiet deletions send nothing");
    }

    #[test]
    fn the_expired_sweep_sends_one_54_per_tick_only_when_something_expired() {
        let mut out = Vec::new();
        expired_buff_sweep(6, 2, true, &mut out);
        assert_eq!(out, vec![NpcSend::BuffList { slot: 6 }]);
        out.clear();
        expired_buff_sweep(6, 2, false, &mut out);
        expired_buff_sweep(6, 1, true, &mut out);
        expired_buff_sweep(6, 0, true, &mut out);
        assert!(out.is_empty());
    }

    #[test]
    fn the_137_removal_request_is_gated_off_by_the_empty_buffid_set() {
        // BuffID.cs:48 creates CanBeRemovedByNetMessage empty; the request
        // never reaches the wire in this decompilation.
        let mut out = Vec::new();
        let mut compacted = false;
        request_buff_removal(1, 20, 1, true, &mut out, || compacted = true);
        assert!(out.is_empty() && !compacted);
        assert!(!can_be_removed_by_net_message(20));
    }

    #[test]
    fn transform_sends_the_23_then_the_54_in_order() {
        let p = players_two();
        let mut st = NpcNetState::default();
        let v = mob();
        let mut out = Vec::new();
        transform_sends(&mut st, &v, 7, 2, &p, &mut out);
        assert_eq!(
            out,
            vec![
                NpcSend::State {
                    slot: 7,
                    to: Some(0)
                },
                NpcSend::State {
                    slot: 7,
                    to: Some(1)
                },
                NpcSend::BuffList { slot: 7 },
            ]
        );
        out.clear();
        transform_sends(&mut st, &v, 7, 1, &p, &mut out);
        assert!(out.is_empty());
    }

    // --- the 153 DoT site (NPC.cs:91843-91862) ---

    #[test]
    fn the_eel_whip_dot_fires_every_60th_tick_on_the_server() {
        let mut counter = 0;
        let mut out = Vec::new();
        for _ in 0..59 {
            assert_eq!(eel_whip_dot(&mut counter, 8, 2, &mut out), None);
        }
        assert!(out.is_empty());
        assert_eq!(eel_whip_dot(&mut counter, 8, 2, &mut out), Some(50));
        assert_eq!(
            out,
            vec![NpcSend::DebuffDot {
                slot: 8,
                amount: 50
            }]
        );
        assert_eq!(counter, 0);
        // The client counts but neither damages nor sends (NPC.cs:91853).
        out.clear();
        for _ in 0..60 {
            assert_eq!(eel_whip_dot(&mut counter, 8, 1, &mut out), None);
        }
        assert!(out.is_empty() && counter == 0);
    }

    // --- the town home tail (NPC.cs:90621-90630) ---

    #[test]
    fn the_town_home_change_sends_60_once_per_change_after_the_23() {
        let p = players_two();
        let mut st = NpcNetState::default();
        let mut v = mob();
        v.town_npc = true;
        v.has_head_index = true;
        v.home = (false, 100, 200);
        let mut out = Vec::new();
        st.net_update = true;
        update_network_code(&mut st, &v, 0, 2, &p, NPC_STREAM_SPEED, &mut out);
        // First observation counts as a change (old* start at the C#
        // defaults 0/false; the port's None differs only on the very first
        // tick of a homeless-at-(0,0) NPC, which never has a head index).
        assert_eq!(
            out.last(),
            Some(&NpcSend::TownHome {
                slot: 0,
                homeless: false,
                x: 100,
                y: 200
            })
        );
        out.clear();
        st.net_update = true;
        st.net_spam = 0;
        update_network_code(&mut st, &v, 0, 2, &p, NPC_STREAM_SPEED, &mut out);
        assert_eq!(
            out,
            vec![
                NpcSend::State {
                    slot: 0,
                    to: Some(0)
                },
                NpcSend::State {
                    slot: 0,
                    to: Some(1)
                },
            ],
            "no 60 while the home is unchanged"
        );
        v.home = (true, 0, 0);
        out.clear();
        st.net_spam = 0;
        update_network_code(&mut st, &v, 0, 2, &p, NPC_STREAM_SPEED, &mut out);
        assert_eq!(
            out,
            vec![NpcSend::TownHome {
                slot: 0,
                homeless: true,
                x: 0,
                y: 0
            }]
        );
    }

    #[test]
    fn a_town_npc_never_banks_more_than_one_packet_of_spam() {
        let p = players_two();
        let mut st = NpcNetState::default();
        st.net_spam = 90;
        let mut v = mob();
        v.town_npc = true;
        let mut out = Vec::new();
        st.net_update = true;
        update_network_code(&mut st, &v, 0, 2, &p, NPC_STREAM_SPEED, &mut out);
        // Clamped to 30 first (NPC.cs:90596), so the gate passes.
        assert_eq!(out.len(), 2);
        assert_eq!(st.net_spam, 59); // 30 + 30 - 1
    }

    // --- the tick hook: slot order + the netUpdate reset ---

    #[test]
    fn the_tick_walks_slots_in_order_and_clears_every_net_update_flag() {
        let p = players_two();
        let mut sts = states(3);
        for st in sts.iter_mut() {
            st.net_update = true;
        }
        let views = [mob(), mob(), mob()];
        let out = npc_sync_tick(&mut sts, &views, 2, &p, NPC_STREAM_SPEED);
        // Main.cs:17948-17952: slot order; two clients per slot.
        let slots: Vec<i16> = out
            .iter()
            .map(|s| match s {
                NpcSend::State { slot, .. } => *slot,
                _ => panic!("unexpected send"),
            })
            .collect();
        assert_eq!(slots, vec![0, 0, 1, 1, 2, 2]);
        assert!(sts.iter().all(|st| !st.net_update), "NPC.cs:90515");
    }
}
