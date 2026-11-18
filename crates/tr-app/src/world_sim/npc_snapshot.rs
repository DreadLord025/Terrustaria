//! The read-only NPC snapshot seam (S1-D-8, research/39 "Phase A — read-only
//! world + spatial hash"). The port's NPC pass scans `app.npcs.iter()` per
//! scanning NPC (the carrier-link family: the Moon Lord parts' core lookup
//! `NPC.cs:41642`/`:42098` + the core's part check `:41251-41269`), re-borrowing
//! the mutable store for every read. This module is the first read-only view:
//! [`NpcSnapshot::build`] copies the scan-visible fields once per tick, and the
//! scan family reads the snapshot (`&NpcSnapshot`) while its writes land on the
//! scanning NPC alone — the borrow split Stage 2's read-only phase needs.
//!
//! Behaviour is frozen: the view mirrors the pass's own `hp = 0` writes through
//! [`NpcSnapshot::kill`] (the only mutation the family makes that a later scan
//! in the same family can observe — a dead part must vanish from the leech
//! blob's head fallback and the core's alive check, exactly as the live
//! `hp > 0` reads saw it). No scan predicate, order or constant changes.

use crate::npc::{Npc, NpcKind};

/// One NPC's scan-visible state: the stable identity (`NPC.whoAmI`,
/// `NPC.cs:6145`), the kind, the life, and the centre (`Npc::center`) the
/// carrier write-through mirrors.
pub(crate) struct NpcView {
    pub uid: u32,
    pub kind: NpcKind,
    pub hp: i32,
    pub cx: f32,
    pub cy: f32,
}

/// The per-tick read-only NPC snapshot. Built from `&[Npc]` before a scan
/// family runs; the family's scans take `&self`, so they cannot touch the
/// mutable store. The owning pass mirrors its own kill writes via [`Self::kill`].
pub(crate) struct NpcSnapshot {
    rows: Vec<NpcView>,
}

impl NpcSnapshot {
    /// Copy the scan-visible fields in slot order (the C#'s `Main.npc[]`
    /// order). The family never moves or resizes an NPC, so the centres stay
    /// valid for the whole family.
    pub fn build(npcs: &[Npc]) -> Self {
        let rows = npcs
            .iter()
            .map(|n| {
                let (cx, cy) = n.center();
                NpcView {
                    uid: n.uid,
                    kind: n.kind,
                    hp: n.hp,
                    cx,
                    cy,
                }
            })
            .collect();
        Self { rows }
    }

    pub fn rows(&self) -> &[NpcView] {
        &self.rows
    }

    /// The carrier lookup: `npcs.iter().position(|m| m.uid == uid &&
    /// m.uid != 0 && m.hp > 0)` — the pre-pass's core/head resolution.
    pub fn position_living_uid(&self, uid: u32) -> Option<usize> {
        self.rows
            .iter()
            .position(|m| m.uid == uid && m.uid != 0 && m.hp > 0)
    }

    /// The unlinked part's fallback: `npcs.iter().find(|m| m.kind == want &&
    /// m.hp > 0).map(|m| m.uid)` — the family's core (or the blob's head).
    pub fn first_living_kind_uid(&self, want: NpcKind) -> Option<u32> {
        self.rows
            .iter()
            .find(|m| m.kind == want && m.hp > 0)
            .map(|m| m.uid)
    }

    /// The core's part check (`NPC.cs:41251-41269`): `npcs.iter().any(|m|
    /// m.uid as i32 == uid && m.hp > 0)`.
    pub fn any_living_uid(&self, uid: i32) -> bool {
        self.rows.iter().any(|m| m.uid as i32 == uid && m.hp > 0)
    }

    /// Mirror the pass's own `hp = 0` write on slot `idx` so later scans in
    /// the same family see the death exactly as the live `hp > 0` reads did.
    /// This is the ONLY mutation the seam allows, and only from the owning
    /// pass — the scans themselves stay read-only.
    pub fn kill(&mut self, idx: usize) {
        self.rows[idx].hp = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pool() -> Vec<Npc> {
        // A core (uid 10), two living parts (uid 11/12), a dead head
        // (uid 13, hp 0) and an unlinked slot (uid 0).
        let mut core = Npc::new(NpcKind::MoonLordCore, 100.0, 200.0, 1, 0.0);
        core.uid = 10;
        core.hp = core.max_hp;
        let mut hand = Npc::new(NpcKind::MoonLordHand, 300.0, 400.0, 1, 0.0);
        hand.uid = 11;
        hand.hp = hand.max_hp;
        let mut head = Npc::new(NpcKind::MoonLordHead, 500.0, 600.0, 1, 0.0);
        head.uid = 12;
        head.hp = head.max_hp;
        let mut dead_head = Npc::new(NpcKind::MoonLordHead, 700.0, 800.0, 1, 0.0);
        dead_head.uid = 13;
        dead_head.hp = 0;
        let unlinked = Npc::new(NpcKind::Slime, 0.0, 0.0, 1, 0.0);
        vec![core, hand, head, dead_head, unlinked]
    }

    #[test]
    fn the_snapshot_scans_match_the_live_slice_scans() {
        let npcs = pool();
        let snap = NpcSnapshot::build(&npcs);
        // The carrier lookup, incl. the uid != 0 guard.
        for uid in [0u32, 10, 11, 12, 13, 99] {
            assert_eq!(
                snap.position_living_uid(uid),
                npcs.iter()
                    .position(|m| m.uid == uid && m.uid != 0 && m.hp > 0),
                "uid {uid}"
            );
        }
        // The fallback find + the core's alive check.
        for want in [NpcKind::MoonLordCore, NpcKind::MoonLordHead, NpcKind::Slime] {
            assert_eq!(
                snap.first_living_kind_uid(want),
                npcs.iter()
                    .find(|m| m.kind == want && m.hp > 0)
                    .map(|m| m.uid)
            );
        }
        for uid in [0i32, 10, 11, 12, 13, 99] {
            assert_eq!(
                snap.any_living_uid(uid),
                npcs.iter().any(|m| m.uid as i32 == uid && m.hp > 0),
                "uid {uid}"
            );
        }
        // The centres ride Npc::center at build time.
        for (i, n) in npcs.iter().enumerate() {
            assert_eq!((snap.rows()[i].cx, snap.rows()[i].cy), n.center());
        }
    }

    #[test]
    fn the_kill_mirror_matches_the_live_hp_write() {
        let mut npcs = pool();
        let mut snap = NpcSnapshot::build(&npcs);
        // The pre-pass kills the living head (slot 2): the blob's fallback
        // and the core's alive check must stop seeing it, exactly like the
        // live reads.
        npcs[2].hp = 0;
        snap.kill(2);
        assert_eq!(
            snap.first_living_kind_uid(NpcKind::MoonLordHead),
            npcs.iter()
                .find(|m| m.kind == NpcKind::MoonLordHead && m.hp > 0)
                .map(|m| m.uid)
        );
        assert_eq!(snap.position_living_uid(12), None);
        assert!(!snap.any_living_uid(12));
        // The core is untouched.
        assert_eq!(snap.position_living_uid(10), Some(0));
    }
}
