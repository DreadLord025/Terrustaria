//! S1-I-4 wave C — the PROJECTILE `SendData` cadence (the send path).
//!
//! The C# fans projectile sync out over three cadences, all ported here as
//! pure functions over the pool + a per-slot bookkeeping state. This module
//! is deliberately disjoint from `netglue.rs` (wave A, world-item) and
//! `net_npc_sync.rs` (wave B, NPC): it does not touch the message-id table
//! (S1-I-1) — the intents below ride the existing `27`/`29` bodies.
//!
//! # Site census (decompiled/Terraria)
//!
//! | site | C# | cadence |
//! | --- | --- | --- |
//! | spawn | `Projectile.cs:11096` (`NewProjectile` tail, after `FinalizeProjectile` + modifier) | one `27` per spawn, no site guard — `SendData` itself returns at `netMode == 0` (`NetMessage.cs:83-86`) |
//! | kill | `Projectile.cs:77254-77257` (`Kill` tail) | one `29` per kill, guarded `owner == Main.myPlayer && Main.netMode != 0`; the position is the **entry** snapshot (`:70648` `Vector2 vector = base.position`), not the death position |
//! | per-update | `Projectile.cs:17210-17244` (`Update` tail) | the `netUpdate`/`netSpam` budget: owner gate `:17210`, `netUpdate2` promotion `:17214-17217`, inactive spam reset `:17218-17221`, budget `< 60` / cost `+5` `:17224-17233`, refill `-1` `:17235-17238`, `netUpdate = false` tail `:17244` (and the pre-AI clear `:16772`) |
//! | section recheck | `Projectile.cs:17246-17256` (`RecheckSectionsForSkippedUpdates`, host only `:17240-17243`) | one targeted `27` per player whose skipped flag is set and whose section became active |
//! | AI immediate | `:23744`/`:23760`/`:23780` (bounce chains 7/494/150/151), `:52472` (golf), `:57818` (charm/quiver/necklace retarget), `:73866` (1037/1049), `:74845` (bobbers 99/1013/727/1014/1021/1048/1053/1054/1055/1057) | one `27` at the AI moment, ahead of the update-tail cadence block |
//! | despawn mark | `Main.cs:37899-37906` (`TryDespawningProjectile_ModifyBeforeKill`) | `aiStyle == 16 && !hostile` → `ai[0] = 1000` + one `27`, before `Kill` |
//!
//! Writer bodies: `NetMessage.cs:776-841` (`case 27`: key, position, velocity,
//! `(short)type`, the two `BitsByte` flags, then the non-zero ai/damage/
//! knockBack/originalDamage fields in flag order), `:852-855` (`case 29`:
//! `number` = key, `Vector2(number2, number3)` = the entry position).
//! Broadcast tail `NetMessage.cs:1774-1800`: `flag3` (`type == 12 ||
//! Main.projPet[type] || aiStyle == 11 || netImportant`, `:1777`) exempts the
//! projectile from section culling and clears its skipped flags; otherwise a
//! client with an inactive section is skipped and remembered
//! (`netSyncSkippedForPlayer`, `Projectile.cs:386`). A targeted send clears
//! that player's flag (`NetMessage.cs:1821-1823`).
//!
//! # Wiring status (S1-I-4d SWAP)
//!
//! `netglue.rs::proj_sync_cadence` steps `ProjSync` per fixed step and
//! `drain_proj_outbox` sends the result through the existing `27`/`29` wires.
//! This REPLACED `netglue.rs::projectile_sync_plan`'s 10 Hz diff — the two send
//! paths never both run (the no-double-send witnesses pin it). The `netUpdate
//! = true` producers (337 C# AI sites) are still a follow-up: until they land,
//! an interim change-detect producer arms `mark_dirty` (recorded in ANALYSIS),
//! and the port's AI arms keep their "net code - SKIPPED" comments. The send
//! BUDGET itself is the faithful C# gate, not the diff.

// The cadence engine is wired through netglue; the event-site helpers
// (sync_now / despawn_mark_applies / recheck_sections) stay exercised by the
// per-site tests until their simulation callers land.
#![allow(dead_code)]

use crate::net;
use crate::projectile::{proj_data, ProjectilePool, MAX_PROJECTILES};

/// `Main.netMode` — 0 single-player, 1 client, 2 host.
pub(crate) const NET_SP: u8 = 0;

/// One send intent, in the C#'s emission order. The caller maps `Sync` to
/// `S2c::Projectile`/`C2s::Projectile` and `Kill` to `ProjectileDead` by its
/// role — the ids stay the I-1 table's `27`/`29`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum ProjSend {
    /// `SendData(27, -1, -1, null, whoAmI)` — broadcast.
    Sync(net::ProjectileState),
    /// `SendData(27, i, -1, null, whoAmI)` — targeted (the section recheck).
    SyncTo {
        state: net::ProjectileState,
        client: u8,
    },
    /// `SendData(29, -1, -1, null, key, x, y)` — the entry-position despawn.
    Kill { key: i32, x: f32, y: f32 },
}

/// The per-slot net bookkeeping the C# carries on each `Projectile`
/// (`Projectile.cs:766-769`'s `SetDefaults` clear, `:386`'s skipped array).
#[derive(Clone, Debug)]
struct SlotNet {
    /// `this.netUpdate` — an AI arm dirtied the slot this tick.
    net_update: bool,
    /// `this.netUpdate2` — the budget parked a send; retry next tick.
    net_update2: bool,
    /// `this.netSpam` — the send budget (cost 5, cap 60, refill 1/tick).
    net_spam: i32,
    /// `netSyncSkippedForPlayer[255]` (`Projectile.cs:386`).
    sync_skipped: [bool; 255],
}

impl Default for SlotNet {
    fn default() -> Self {
        Self {
            net_update: false,
            net_update2: false,
            net_spam: 0,
            sync_skipped: [false; 255],
        }
    }
}

/// The projectile sync engine: one [`SlotNet`] per pool slot, plus the
/// per-site send functions. The C# keeps the bookkeeping on the projectile
/// itself; the port keeps it beside the pool so `projectile.rs` stays
/// untouched by this wave.
pub(crate) struct ProjSync {
    slots: Vec<SlotNet>,
}

impl Default for ProjSync {
    fn default() -> Self {
        Self {
            slots: vec![SlotNet::default(); MAX_PROJECTILES],
        }
    }
}

/// The `case 27` body's field snapshot (`NetMessage.cs:776-841`): the key,
/// position/velocity, `(short)type`, and the flag-gated ai/damage fields.
/// The `(short)` casts are C# unchecked truncations — Rust `as i16` matches.
/// `bannerIdToRespondTo` (`:790-793`) has no pool field yet (the banner
/// response is unported); it rides as 0 until that lands.
fn state_of(pool: &ProjectilePool, slot: usize) -> net::ProjectileState {
    let p = &pool.slots()[slot];
    net::ProjectileState {
        key: pool.key_of(slot),
        x: p.pos.x,
        y: p.pos.y,
        vx: p.vel.x,
        vy: p.vel.y,
        kind: p.kind as i16,
        ai0: if p.ai[0] != 0.0 { Some(p.ai[0]) } else { None },
        ai1: if p.ai[1] != 0.0 { Some(p.ai[1]) } else { None },
        banner_id: 0,
        damage: p.damage as i16,
        knock_back: p.knock_back,
        original_damage: p.original_damage as i16,
        ai2: if p.ai[2] != 0.0 { Some(p.ai[2]) } else { None },
    }
}

/// `flag3` (`NetMessage.cs:1777`): the section-culling exemption —
/// `type == 12 || Main.projPet[type] || aiStyle == 11 || netImportant`.
/// `proj_pet` is a parameter because `Main.projPet` has no port table yet;
/// the caller passes the pet flag once it lands (false today).
pub(crate) fn sync_ignores_sections(
    kind: u16,
    proj_pet: bool,
    ai_style: u8,
    net_important: bool,
) -> bool {
    kind == 12 || proj_pet || ai_style == 11 || net_important
}

impl ProjSync {
    /// Marks a slot dirty — the port-side stand-in for the C# AI arms'
    /// `this.netUpdate = true` writes (337 sites). The AI-arm wiring is the
    /// follow-up; until then only tests and explicit callers set it.
    pub(crate) fn mark_dirty(&mut self, slot: usize) {
        if slot < self.slots.len() {
            self.slots[slot].net_update = true;
        }
    }

    /// The spawn site (`Projectile.cs:11096`): `NewProjectile`'s tail sends
    /// one `27` for the fresh slot, unguarded at the site — the only gate is
    /// `SendData`'s own `netMode == 0` return (`NetMessage.cs:83-86`). The
    /// send rides **after** `FinalizeProjectile` and the modifier, so the
    /// caller invokes this once the slot is fully set up.
    pub(crate) fn spawn_sync(
        &mut self,
        pool: &ProjectilePool,
        slot: usize,
        net_mode: u8,
    ) -> Option<ProjSend> {
        if net_mode == NET_SP {
            return None;
        }
        // A flag3 projectile clears its skipped flags on every broadcast
        // (`NetMessage.cs:1779-1781`); a fresh slot has none set, but the
        // recycled index may carry stale flags from its previous occupant.
        let p = &pool.slots()[slot];
        let ai_style = proj_data(p.kind).map_or(255, |d| d.ai_style);
        if sync_ignores_sections(p.kind, false, ai_style, p.net_important) {
            self.slots[slot].sync_skipped = [false; 255];
        }
        Some(ProjSend::Sync(state_of(pool, slot)))
    }

    /// The kill site (`Projectile.cs:77254-77257`): `Kill`'s tail sends one
    /// `29` when `owner == Main.myPlayer && Main.netMode != 0`. `entry_pos`
    /// is the position snapshot taken at `Kill`'s entry (`:70648`) — the
    /// death effects may move the projectile before the send, and the wire
    /// carries the **entry** position regardless.
    pub(crate) fn kill_sync(
        &mut self,
        pool: &ProjectilePool,
        slot: usize,
        entry_pos: (f32, f32),
        local_id: u8,
        net_mode: u8,
    ) -> Option<ProjSend> {
        let p = &pool.slots()[slot];
        if p.owner != local_id || net_mode == NET_SP {
            return None;
        }
        // The slot's bookkeeping dies with it (`SetDefaults` clears on the
        // next spawn, `Projectile.cs:766-769`); the C# leaves the fields on
        // the dead object, but nothing reads them before the next defaults.
        self.slots[slot] = SlotNet::default();
        Some(ProjSend::Kill {
            key: pool.key_of(slot),
            x: entry_pos.0,
            y: entry_pos.1,
        })
    }

    /// The per-update cadence block (`Projectile.cs:17210-17244`), run once
    /// per fixed step **after** the AI pass, in ascending slot order (the
    /// C# `Main.projectile` loop). The block sits inside the sub-step
    /// `while` (`:16676`), so it runs `1 + extraUpdates` times per tick;
    /// per sub-step, in the source's order: owner gate → `netUpdate2`
    /// promotion → inactive spam reset → the budget gate (`< 60` send /
    /// cost 5, else park) → the refill decrement. The `netUpdate = false`
    /// tail (`:17244`) is **outside** the loop and the owner gate — it
    /// clears every slot's flag every tick, remote ones included (the
    /// pre-AI clear `:16772` is the same flag's head-of-update reset).
    pub(crate) fn update_cadence(
        &mut self,
        pool: &ProjectilePool,
        local_id: u8,
        net_mode: u8,
    ) -> Vec<ProjSend> {
        let mut sends = Vec::new();
        let online = net_mode != NET_SP;
        for slot in 0..MAX_PROJECTILES {
            let p = &pool.slots()[slot];
            // `SendData`'s early return still lets the bookkeeping run in
            // the C# (the fields tick in single-player too); the port keeps
            // the same state motion, minus the sends.
            if p.active && p.owner == local_id {
                for _ in 0..(1 + p.extra_updates) {
                    self.cadence_slot(pool, slot, local_id, online, &mut sends);
                }
            }
            // `:17244` — the tail clear, every slot, once per update.
            self.slots[slot].net_update = false;
        }
        sends
    }

    fn cadence_slot(
        &mut self,
        pool: &ProjectilePool,
        slot: usize,
        local_id: u8,
        online: bool,
        sends: &mut Vec<ProjSend>,
    ) {
        let p = &pool.slots()[slot];
        // `:17210` — `if (!this.active || this.owner != Main.myPlayer)
        // continue;`: a remote or dead slot pays nothing, not even the
        // spam refill.
        if !p.active || p.owner != local_id {
            return;
        }
        let sn = &mut self.slots[slot];
        // `:17214-17217` — the parked retry promotes first.
        if sn.net_update2 {
            sn.net_update = true;
        }
        // `:17218-17221` — `if (!this.active) this.netSpam = 0;` (dead code
        // past the `:17210` gate in the C#; kept for shape parity).
        if !p.active {
            sn.net_spam = 0;
        }
        // `:17222-17234` — the budget gate.
        if sn.net_update {
            if sn.net_spam < 60 {
                sn.net_spam += 5;
                if online {
                    sends.push(ProjSend::Sync(state_of(pool, slot)));
                }
                sn.net_update2 = false;
            } else {
                sn.net_update2 = true;
            }
        }
        // `:17235-17238` — the refill, after the send decision.
        if sn.net_spam > 0 {
            sn.net_spam -= 1;
        }
        // The `netUpdate = false` tail (`:17244`) lives outside the
        // sub-step loop — `update_cadence` clears it once per tick.
    }

    /// The section recheck (`Projectile.cs:17246-17256`, host only per
    /// `:17240-17243`): for each player whose skipped flag is set, an active
    /// player with an active section gets one targeted `27`, and the flag
    /// clears (`NetMessage.cs:1821-1823`). `player_active` and
    /// `section_active` are the `Main.player[i].active` and
    /// `Netplay.Clients[i].IsSectionActive(NetSectionCoordinates)` reads.
    pub(crate) fn recheck_sections(
        &mut self,
        pool: &ProjectilePool,
        slot: usize,
        player_active: &[bool; 255],
        section_active: impl Fn(u8) -> bool,
    ) -> Vec<ProjSend> {
        let mut sends = Vec::new();
        let p = &pool.slots()[slot];
        if !p.active {
            return sends;
        }
        let state = state_of(pool, slot);
        for i in 0..255usize {
            if self.slots[slot].sync_skipped[i] && player_active[i] && section_active(i as u8) {
                self.slots[slot].sync_skipped[i] = false;
                sends.push(ProjSend::SyncTo {
                    state,
                    client: i as u8,
                });
            }
        }
        sends
    }

    /// The broadcast tail's section bookkeeping (`NetMessage.cs:1782-1799`):
    /// a flag3 projectile clears all skipped flags; otherwise each client
    /// with an inactive section is remembered as skipped. The caller runs
    /// this when it emits a broadcast `Sync` (the actual packet fan-out is
    /// the server's, this is the C#'s per-projectile flag motion).
    pub(crate) fn note_broadcast(
        &mut self,
        pool: &ProjectilePool,
        slot: usize,
        section_active: impl Fn(u8) -> bool,
    ) {
        let p = &pool.slots()[slot];
        let ai_style = proj_data(p.kind).map_or(255, |d| d.ai_style);
        if sync_ignores_sections(p.kind, false, ai_style, p.net_important) {
            self.slots[slot].sync_skipped = [false; 255];
            return;
        }
        for i in 0..255usize {
            self.slots[slot].sync_skipped[i] = !section_active(i as u8);
        }
    }

    /// The despawn-mark site (`Main.cs:37899-37906`): a non-hostile
    /// aiStyle-16 (yoyo) projectile gets `ai[0] = 1000` and one `27` before
    /// `Kill`. Returns whether the mark applies; the caller writes
    /// `ai[0] = 1000` through the pool and then emits the `Sync` (the send
    /// is unguarded at the site — only `SendData`'s netMode gate applies).
    pub(crate) fn despawn_mark_applies(ai_style: u8, hostile: bool) -> bool {
        ai_style == 16 && !hostile
    }

    /// The AI immediate sites (`Projectile.cs:23744`/`:23760`/`:23780`,
    /// `:52472`, `:57818`, `:73866`, `:74845`): one `27` at the AI moment.
    /// The site guards differ (the bounce chains and golf gate on
    /// `owner == Main.myPlayer`; `:57818` is unguarded; `:73866`/`:74845`
    /// gate on `owner == Main.myPlayer && netMode != 0`) — the caller
    /// applies its site's guard and then calls this, which carries only
    /// `SendData`'s own netMode gate (`NetMessage.cs:83-86`).
    pub(crate) fn sync_now(
        &mut self,
        pool: &ProjectilePool,
        slot: usize,
        net_mode: u8,
    ) -> Option<ProjSend> {
        if net_mode == NET_SP {
            return None;
        }
        Some(ProjSend::Sync(state_of(pool, slot)))
    }

    /// The per-fixed-step hook `netglue.rs::proj_sync_cadence` calls (S1-I-4d,
    /// replacing `netglue.rs::projectile_sync_plan`): the update-tail cadence
    /// pass over the whole pool, in slot order. Spawn/kill/AI immediate sends
    /// are event-driven at their simulation sites and are NOT re-emitted here
    /// — that split is what keeps the C#'s send order.
    pub(crate) fn step(
        &mut self,
        pool: &ProjectilePool,
        local_id: u8,
        net_mode: u8,
    ) -> Vec<ProjSend> {
        self.update_cadence(pool, local_id, net_mode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::projectile::ProjectilePool;

    /// A pool with one local-owned arrow (kind 1, aiStyle 1) at slot 0.
    fn pool_with_arrow(owner: u8) -> ProjectilePool {
        let mut pool = ProjectilePool::new();
        pool.spawn_with_data_owned(1, owner, 100.0, 200.0, 4.0, -1.0, 10, 2.0, 0.0, 0.0)
            .expect("a free slot");
        pool
    }

    /// The spawn site (`Projectile.cs:11096`): one `27` per spawn online,
    /// nothing in single-player (`NetMessage.cs:83-86`).
    #[test]
    fn the_spawn_site_sends_one_27_online_and_nothing_offline() {
        let pool = pool_with_arrow(0);
        let mut sync = ProjSync::default();
        let send = sync.spawn_sync(&pool, 0, 2).expect("the spawn send");
        match send {
            ProjSend::Sync(st) => {
                assert_eq!(st.key, pool.key_of(0));
                assert_eq!((st.x, st.y), (100.0, 200.0));
                assert_eq!((st.vx, st.vy), (4.0, -1.0));
                assert_eq!(st.kind, 1);
                assert_eq!(st.damage, 10);
                assert_eq!(st.knock_back, 2.0);
                assert_eq!(st.ai0, None, "ai[0] == 0 rides no flag");
            }
            other => panic!("expected a 27 Sync, got {other:?}"),
        }
        assert_eq!(sync.spawn_sync(&pool, 0, NET_SP), None);
        // The client role sends too — the site has no role guard.
        assert!(sync.spawn_sync(&pool, 0, 1).is_some());
    }

    /// The kill site (`Projectile.cs:77254-77257`): the owner gate, the
    /// netMode gate, and the **entry** position (`:70648`), not the current.
    #[test]
    fn the_kill_site_sends_29_with_the_entry_position_and_owner_gate() {
        let mut pool = pool_with_arrow(0);
        let mut sync = ProjSync::default();
        // The death effects moved the slot after Kill's entry snapshot.
        let entry = (pool.slots()[0].pos.x, pool.slots()[0].pos.y);
        let mut moved = state_of(&pool, 0);
        moved.x = 999.0;
        moved.y = 999.0;
        pool.upsert_remote(&moved);
        assert_eq!(pool.slots()[0].pos.x, 999.0, "the slot moved post-entry");
        let send = sync
            .kill_sync(&pool, 0, entry, 0, 2)
            .expect("the kill send");
        assert_eq!(
            send,
            ProjSend::Kill {
                key: pool.key_of(0),
                x: 100.0,
                y: 200.0,
            },
            "the wire carries the entry snapshot"
        );
        // A remote owner's kill sends nothing (the owner's machine sends).
        let remote = pool_with_arrow(1);
        assert_eq!(sync.kill_sync(&remote, 0, entry, 0, 2), None);
        // Single-player sends nothing.
        assert_eq!(sync.kill_sync(&pool, 0, entry, 0, NET_SP), None);
    }

    /// The budget arithmetic (`Projectile.cs:17224-17238`): cost 5, cap 60,
    /// refill 1/tick — a continuously-dirty slot sends 15 ticks, parks on
    /// the 16th, and resumes when the refill drops the budget under 60.
    #[test]
    fn the_update_cadence_budget_matches_the_csharp_arithmetic() {
        let pool = pool_with_arrow(0);
        let mut sync = ProjSync::default();
        let mut sent = 0usize;
        // Ticks 1..=15: spam before the gate is 4*(k-1) < 60 → send.
        for k in 1..=15usize {
            sync.mark_dirty(0);
            let sends = sync.update_cadence(&pool, 0, 2);
            assert_eq!(sends.len(), 1, "tick {k} sends");
            sent += sends.len();
            assert_eq!(sync.slots[0].net_spam, 4 * k as i32, "tick {k} budget");
            assert!(!sync.slots[0].net_update2);
        }
        // Tick 16: spam 60 is not < 60 → park (netUpdate2), refill to 59.
        sync.mark_dirty(0);
        assert!(sync.update_cadence(&pool, 0, 2).is_empty(), "tick 16 parks");
        assert!(sync.slots[0].net_update2);
        assert_eq!(sync.slots[0].net_spam, 59);
        // Tick 17: the promotion retries, 59 < 60 → send again.
        let sends = sync.update_cadence(&pool, 0, 2);
        assert_eq!(
            sends.len(),
            1,
            "the parked retry sends once the budget frees"
        );
        assert!(!sync.slots[0].net_update2);
        assert_eq!(sent, 15);
    }

    /// The owner gate and the tail clear (`Projectile.cs:17210`, `:17244`):
    /// a remote slot never sends; a clean local slot never sends; the dirty
    /// flag clears every tick whether or not the send fired.
    #[test]
    fn the_update_cadence_gates_on_owner_and_clears_the_flag() {
        let pool = pool_with_arrow(1); // remote-owned
        let mut sync = ProjSync::default();
        sync.mark_dirty(0);
        assert!(sync.update_cadence(&pool, 0, 2).is_empty());
        assert_eq!(sync.slots[0].net_spam, 0, "a remote slot pays nothing");
        assert!(
            !sync.slots[0].net_update,
            "the :17244 tail clears even a remote slot's flag"
        );
        // Local, but clean.
        let pool = pool_with_arrow(0);
        assert!(sync.update_cadence(&pool, 0, 2).is_empty());
        // Dirty once → one send, then the flag is gone.
        sync.mark_dirty(0);
        assert_eq!(sync.update_cadence(&pool, 0, 2).len(), 1);
        assert!(!sync.slots[0].net_update);
        assert!(sync.update_cadence(&pool, 0, 2).is_empty());
        // Single-player keeps the state motion but emits nothing.
        let mut sync = ProjSync::default();
        sync.mark_dirty(0);
        assert!(sync.update_cadence(&pool, 0, NET_SP).is_empty());
        assert_eq!(sync.slots[0].net_spam, 4, "the budget still ticks offline");
    }

    /// The section recheck (`Projectile.cs:17246-17256`): only the skipped,
    /// active players with an active section get the targeted `27`, and the
    /// flag clears on send (`NetMessage.cs:1821-1823`).
    #[test]
    fn the_section_recheck_targets_only_skipped_active_players() {
        let pool = pool_with_arrow(0);
        let mut sync = ProjSync::default();
        // The broadcast skipped clients 1 and 2 (inactive sections).
        sync.note_broadcast(&pool, 0, |c| c != 1 && c != 2);
        assert!(sync.slots[0].sync_skipped[1]);
        assert!(sync.slots[0].sync_skipped[2]);
        let mut active = [false; 255];
        active[1] = true;
        active[2] = true;
        active[3] = true;
        // Client 1's section came back; client 2's did not.
        let sends = sync.recheck_sections(&pool, 0, &active, |c| c == 1);
        assert_eq!(sends.len(), 1);
        match sends[0] {
            ProjSend::SyncTo { client, state } => {
                assert_eq!(client, 1);
                assert_eq!(state.key, pool.key_of(0));
            }
            other => panic!("expected a targeted 27, got {other:?}"),
        }
        assert!(!sync.slots[0].sync_skipped[1], "the send clears the flag");
        assert!(
            sync.slots[0].sync_skipped[2],
            "the still-inactive stays parked"
        );
        // Client 3 was never skipped — no send even though active.
        assert!(sync.recheck_sections(&pool, 0, &active, |_| true).len() == 1);
    }

    /// `flag3` (`NetMessage.cs:1777`): type 12, pets, aiStyle 11 and
    /// netImportant projectiles bypass section culling and clear the flags.
    #[test]
    fn flag3_projectiles_ignore_sections() {
        assert!(sync_ignores_sections(12, false, 0, false));
        assert!(sync_ignores_sections(1, true, 0, false));
        assert!(sync_ignores_sections(1, false, 11, false));
        assert!(sync_ignores_sections(1, false, 0, true));
        assert!(!sync_ignores_sections(1, false, 1, false));
        // A netImportant spawn clears stale skipped flags on the recycled
        // index; the broadcast note keeps them clear. The Hornet minion
        // (373) carries netImportant in proj_data (`minion || sentry`).
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn_with_data_owned(
                crate::projectile::HORNET,
                0,
                0.0,
                0.0,
                0.0,
                0.0,
                10,
                0.0,
                0.0,
                0.0,
            )
            .expect("a free slot");
        assert!(
            pool.slots()[slot].net_important,
            "the Hornet is netImportant"
        );
        let mut sync = ProjSync::default();
        sync.slots[slot].sync_skipped[1] = true;
        sync.spawn_sync(&pool, slot, 2).expect("the spawn send");
        assert!(!sync.slots[slot].sync_skipped[1], "flag3 cleared the flags");
        sync.note_broadcast(&pool, slot, |_| false);
        assert!(
            !sync.slots[slot].sync_skipped[1],
            "flag3 never re-parks a client"
        );
    }

    /// The despawn mark (`Main.cs:37899-37906`): only non-hostile aiStyle-16
    /// yoyos get the `ai[0] = 1000` + `27` before the kill.
    #[test]
    fn the_despawn_mark_applies_only_to_friendly_yoyos() {
        assert!(ProjSync::despawn_mark_applies(16, false));
        assert!(!ProjSync::despawn_mark_applies(16, true));
        assert!(!ProjSync::despawn_mark_applies(1, false));
    }

    /// The AI immediate sites (`Projectile.cs:52472` etc.): one `27` at the
    /// AI moment, gated only by `SendData`'s netMode return.
    #[test]
    fn the_ai_immediate_site_sends_one_27_online() {
        let pool = pool_with_arrow(0);
        let mut sync = ProjSync::default();
        let send = sync.sync_now(&pool, 0, 1).expect("the immediate send");
        assert!(matches!(send, ProjSend::Sync(st) if st.key == pool.key_of(0)));
        assert_eq!(sync.sync_now(&pool, 0, NET_SP), None);
        // The immediate send does not touch the budget — the C# site is a
        // bare SendData, outside the netSpam block.
        assert_eq!(sync.slots[0].net_spam, 0);
    }

    /// The block sits inside the sub-step `while` (`Projectile.cs:16676`):
    /// a slot with `extraUpdates = 1` runs the budget body twice per tick —
    /// two sends while dirty, spam `5-1 + 5-1 = 8`, not `4`. The Musket
    /// Ball (14) carries `extraUpdates = 1` in its data row.
    #[test]
    fn the_update_cadence_runs_per_sub_step() {
        let mut pool = ProjectilePool::new();
        pool.spawn_with_data_owned(
            crate::projectile::MUSKET_BALL,
            0,
            0.0,
            0.0,
            0.0,
            0.0,
            10,
            0.0,
            0.0,
            0.0,
        )
        .expect("a free slot");
        assert_eq!(pool.slots()[0].extra_updates, 1);
        let mut sync = ProjSync::default();
        // The flag stays set across the sub-steps (the tail clear is
        // outside the loop), so both send — the double-cost arithmetic.
        sync.mark_dirty(0);
        let sends = sync.update_cadence(&pool, 0, 2);
        assert_eq!(sends.len(), 2, "both sub-steps send while dirty");
        assert_eq!(sync.slots[0].net_spam, 5 - 1 + 5 - 1);
        assert!(!sync.slots[0].net_update, "the tail clear runs once");
        // The next clean tick pays only the two refills.
        assert!(sync.update_cadence(&pool, 0, 2).is_empty());
        assert_eq!(sync.slots[0].net_spam, 6);
    }

    /// The `case 27` field snapshot (`NetMessage.cs:776-841`): the non-zero
    /// ai flags, the `(short)` truncations, the originalDamage field.
    #[test]
    fn the_state_snapshot_matches_the_case27_body() {
        let mut pool = pool_with_arrow(0);
        let slot = pool
            .spawn_with_data_owned(1, 0, 0.0, 0.0, 0.0, 0.0, 70000, 0.0, 1.5, 2.5)
            .expect("a free slot");
        pool.set_original_damage(slot, 70000);
        let st = state_of(&pool, slot);
        assert_eq!(st.ai0, Some(1.5));
        assert_eq!(st.ai1, Some(2.5));
        assert_eq!(st.ai2, None);
        // `(short)70000` truncates to 4464 in the C# unchecked cast.
        assert_eq!(st.damage, 70000i32 as i16);
        assert_eq!(st.original_damage, 70000i32 as i16);
    }
}
