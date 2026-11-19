//! The cell-bucketed NPC spatial hash (S1-D-8b, research/39 "Phase A —
//! read-only world + spatial hash"). Built over the read-only
//! [`NpcSnapshot`] seam (`world_sim/npc_snapshot.rs`): each snapshot slot is
//! bucketed by its centre into a fixed [`CELL_PX`] grid, and the radius
//! scans of the saucer/scutlix/dutchman carrier family
//! (`world_sim/npcs.rs`'s pre-pass — the `AI_075` type-390 link, the
//! `AI_093` crew counts, the saucer mounts' core lookup) visit only the
//! cells that can intersect the query disc instead of re-borrowing
//! `npcs.iter()` for the whole pool.
//!
//! Behaviour is frozen: [`NpcSpatial::near`] yields the candidate slots in
//! ASCENDING SLOT ORDER — the exact order the linear `npcs.iter()` scan
//! visited them — and the callers keep the original predicates, the strict
//! `< 1600.0` reach and the first-wins tie-break untouched. The hash only
//! narrows WHICH slots are examined; a slot outside the visited cells
//! cannot pass the radius predicate (the cell window is conservative, see
//! [`NpcSpatial::near`]), so the candidate sequence after the predicate is
//! identical to the linear scan's. Proof: the A/B equivalence tests below
//! and `world_sim/npcs.rs`'s twin-seed carrier-family tests compare the
//! spatial path against the verbatim linear scan over twin pools.

use std::collections::HashMap;

use crate::npc::NpcKind;
use crate::world_sim::npc_snapshot::{NpcSnapshot, NpcView};

/// The bucket edge in pixels. The carrier family's only reach constant is
/// `1600.0` (`AI_075`/`AI_093`'s link + crew scans), so one cell covers a
/// query's whole reach in each axis: a radius-1600 disc touches at most a
/// 3×3 cell window. The constant is an index granularity ONLY — it never
/// enters a predicate, so it cannot change behaviour.
pub(crate) const CELL_PX: f32 = 1600.0;

/// The query point's cell coordinate (floor division — negative centres
/// bucket into negative cells, exactly mirroring the coordinate line).
fn cell_coord(v: f32) -> i32 {
    (v / CELL_PX).floor() as i32
}

/// The per-family spatial index over a [`NpcSnapshot`]. Buckets hold slot
/// indices in ascending order (the snapshot's `Main.npc[]` order); `hp` is
/// NOT indexed — the owning pass's kill mirror (`NpcSnapshot::kill`) stays
/// visible to every query without rebuilding the hash.
pub(crate) struct NpcSpatial {
    cells: HashMap<(i32, i32), Vec<usize>>,
}

impl NpcSpatial {
    /// Bucket every snapshot slot by its centre. Slots keep their order
    /// inside a bucket (the build walks the rows front-to-back).
    pub fn build(snap: &NpcSnapshot) -> Self {
        let mut cells: HashMap<(i32, i32), Vec<usize>> = HashMap::new();
        for (i, row) in snap.rows().iter().enumerate() {
            cells
                .entry((cell_coord(row.cx), cell_coord(row.cy)))
                .or_default()
                .push(i);
        }
        Self { cells }
    }

    /// The candidate slots whose cell can intersect the axis-aligned box
    /// `[cx - radius, cx + radius] × [cy - radius, cy + radius]`, yielded in
    /// ASCENDING SLOT ORDER with their read-only views.
    ///
    /// Coverage is conservative: with `k = ceil(radius / CELL_PX)` the box
    /// spans cells `qx - k ..= qx + k` (the query centre sits inside cell
    /// `qx`, and the box reaches at most `k` cells past it on each side),
    /// so every slot within `radius` of the centre is in the window. The
    /// window is a superset of the disc — the caller's own distance
    /// predicate does the final cut, exactly as the linear scan did.
    pub fn near<'a>(
        &'a self,
        snap: &'a NpcSnapshot,
        cx: f32,
        cy: f32,
        radius: f32,
    ) -> impl Iterator<Item = (usize, &'a NpcView)> + 'a {
        let k = (radius / CELL_PX).ceil() as i32;
        let (qx, qy) = (cell_coord(cx), cell_coord(cy));
        let mut idxs: Vec<usize> = Vec::new();
        for dy in -k..=k {
            for dx in -k..=k {
                if let Some(bucket) = self.cells.get(&(qx + dx, qy + dy)) {
                    idxs.extend_from_slice(bucket);
                }
            }
        }
        // Restore the linear scan's visit order: the buckets were gathered
        // window-row by window-row, the scan walked slot 0..len.
        idxs.sort_unstable();
        idxs.into_iter().map(move |i| (i, &snap.rows()[i]))
    }

    /// The carrier family's nearest-link scan, verbatim predicate: the
    /// first living `want` strictly within `radius`, ties keeping the
    /// earlier slot (`d < bd` — the linear scan's `best.is_none_or`).
    /// Returns `(slot, distance)` exactly as the linear `for (j, m)` loop's
    /// `best`.
    pub fn nearest_living_kind(
        &self,
        snap: &NpcSnapshot,
        cx: f32,
        cy: f32,
        want: NpcKind,
        radius: f32,
    ) -> Option<(usize, f32)> {
        let mut best: Option<(usize, f32)> = None;
        for (j, m) in self.near(snap, cx, cy, radius) {
            if m.kind != want || m.hp <= 0 {
                continue;
            }
            let d = (m.cx - cx).hypot(m.cy - cy);
            if d < radius && best.is_none_or(|(_, bd)| d < bd) {
                best = Some((j, d));
            }
        }
        best
    }

    /// The crew-count scan, verbatim predicate: living `want` slots whose
    /// centre is strictly within `radius` (`AI_093`'s Dutchman cannons, the
    /// saucer core's turrets/cannons). Count order is irrelevant to the
    /// result, but the visit order still matches the linear scan.
    pub fn count_living_kind_within(
        &self,
        snap: &NpcSnapshot,
        cx: f32,
        cy: f32,
        want: NpcKind,
        radius: f32,
    ) -> i32 {
        self.near(snap, cx, cy, radius)
            .filter(|(_, m)| m.kind == want && m.hp > 0 && (m.cx - cx).hypot(m.cy - cy) < radius)
            .count() as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::Npc;

    /// A tiny deterministic LCG (the tests' twin pools must be reproducible
    /// without touching any game RNG stream).
    struct Lcg(u64);
    impl Lcg {
        fn next(&mut self) -> u64 {
            self.0 = self
                .0
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            self.0 >> 33
        }
        fn f32(&mut self, lo: f32, hi: f32) -> f32 {
            lo + (self.next() % 10_000) as f32 / 10_000.0 * (hi - lo)
        }
    }

    const KINDS: [NpcKind; 8] = [
        NpcKind::Scutlix,
        NpcKind::ScutlixRider,
        NpcKind::MartianSaucerCore,
        NpcKind::MartianSaucerTurret,
        NpcKind::MartianSaucerCannon,
        NpcKind::FlyingDutchman,
        NpcKind::FlyingDutchmanCannon,
        NpcKind::Slime,
    ];

    /// A twin-seed pool: `count` NPCs spread over ±4800 px (several cells),
    /// a third dead, kinds cycling. The SAME seed builds byte-identical
    /// twins for the A/B comparison.
    fn pool(seed: u64, count: usize) -> Vec<Npc> {
        let mut rng = Lcg(seed);
        (0..count)
            .map(|i| {
                let kind = KINDS[i % KINDS.len()];
                let x = rng.f32(-4800.0, 4800.0);
                let y = rng.f32(-4800.0, 4800.0);
                let mut n = Npc::new(kind, x, y, 1, 0.0);
                n.uid = (i + 1) as u32;
                n.hp = if rng.next() % 3 == 0 { 0 } else { n.max_hp };
                n
            })
            .collect()
    }

    /// The linear reference: the family's exact `for (j, m)` scan over the
    /// live slice, returning the surviving `(slot, distance)` sequence.
    fn linear_candidates(
        npcs: &[Npc],
        cx: f32,
        cy: f32,
        want: NpcKind,
        radius: f32,
    ) -> Vec<(usize, f32)> {
        let mut out = Vec::new();
        for (j, m) in npcs.iter().enumerate() {
            if m.kind != want || m.hp <= 0 {
                continue;
            }
            let (mcx, mcy) = m.center();
            let d = (mcx - cx).hypot(mcy - cy);
            if d < radius {
                out.push((j, d));
            }
        }
        out
    }

    fn spatial_candidates(
        snap: &NpcSnapshot,
        spatial: &NpcSpatial,
        cx: f32,
        cy: f32,
        want: NpcKind,
        radius: f32,
    ) -> Vec<(usize, f32)> {
        spatial
            .near(snap, cx, cy, radius)
            .filter(|(_, m)| m.kind == want && m.hp > 0)
            .map(|(j, m)| (j, (m.cx - cx).hypot(m.cy - cy)))
            .filter(|(_, d)| *d < radius)
            .collect()
    }

    #[test]
    fn the_spatial_candidates_match_the_linear_scan_over_twin_seeds() {
        for seed in [1u64, 42, 512, 20261118] {
            let npcs = pool(seed, 200);
            let snap = NpcSnapshot::build(&npcs);
            let spatial = NpcSpatial::build(&snap);
            let mut rng = Lcg(seed ^ 0x9E3779B9);
            for _ in 0..60 {
                let cx = rng.f32(-5000.0, 5000.0);
                let cy = rng.f32(-5000.0, 5000.0);
                for want in KINDS {
                    let lin = linear_candidates(&npcs, cx, cy, want, 1600.0);
                    let spa = spatial_candidates(&snap, &spatial, cx, cy, want, 1600.0);
                    assert_eq!(lin.len(), spa.len(), "seed {seed} centre ({cx},{cy})");
                    for ((li, ld), (si, sd)) in lin.iter().zip(&spa) {
                        assert_eq!(li, si, "slot order, seed {seed}");
                        assert_eq!(ld.to_bits(), sd.to_bits(), "distance bits, seed {seed}");
                    }
                    // The two family queries derived from the sequence.
                    let lin_best = lin.iter().fold(None::<(usize, f32)>, |best, &(j, d)| {
                        if best.is_none_or(|(_, bd)| d < bd) {
                            Some((j, d))
                        } else {
                            best
                        }
                    });
                    assert_eq!(
                        spatial.nearest_living_kind(&snap, cx, cy, want, 1600.0),
                        lin_best
                    );
                    assert_eq!(
                        spatial.count_living_kind_within(&snap, cx, cy, want, 1600.0),
                        lin.len() as i32
                    );
                }
            }
        }
    }

    #[test]
    fn the_cell_boundaries_and_the_strict_reach_match_the_linear_scan() {
        // Centres exactly on cell edges (±1600 multiples), a cannon exactly
        // 1600 px away (the strict `<` must exclude it), one just inside,
        // negative coordinates and a dead slot.
        let mut npcs = Vec::new();
        for (i, (x, y)) in [
            (1600.0f32, 1600.0f32),
            (-1600.0, -1600.0),
            (0.0, 0.0),
            (3200.0, 0.0), // exactly 1600 from (1600,0): excluded
            (3199.9, 0.0), // just inside
            (-0.1, 1599.9),
            (1600.0, 0.0),
        ]
        .into_iter()
        .enumerate()
        {
            let mut n = Npc::new(NpcKind::FlyingDutchmanCannon, x, y, 1, 0.0);
            // Place the CENTRE (not the top-left) on the intended point —
            // `Npc::center` adds half the sprite size.
            let (ccx, ccy) = n.center();
            n.x += x - ccx;
            n.y += y - ccy;
            n.uid = (i + 1) as u32;
            n.hp = if i == 5 { 0 } else { n.max_hp };
            npcs.push(n);
        }
        let snap = NpcSnapshot::build(&npcs);
        let spatial = NpcSpatial::build(&snap);
        for (cx, cy) in [
            (1600.0f32, 0.0f32),
            (0.0, 0.0),
            (-1600.0, -1600.0),
            (1599.9999, 1599.9999),
            (-3200.0, 3200.0),
        ] {
            let lin = linear_candidates(&npcs, cx, cy, NpcKind::FlyingDutchmanCannon, 1600.0);
            let spa = spatial_candidates(
                &snap,
                &spatial,
                cx,
                cy,
                NpcKind::FlyingDutchmanCannon,
                1600.0,
            );
            assert_eq!(lin, spa, "centre ({cx},{cy})");
        }
        // The exact-1600 slot (3) is out, the just-inside slot (4) is in.
        let got = spatial_candidates(
            &snap,
            &spatial,
            1600.0,
            0.0,
            NpcKind::FlyingDutchmanCannon,
            1600.0,
        );
        assert!(got.iter().all(|&(j, _)| j != 3));
        assert!(got.iter().any(|&(j, _)| j == 4));
    }

    #[test]
    fn the_kill_mirror_stays_visible_without_a_rebuild() {
        let mut npcs = pool(7, 64);
        let mut snap = NpcSnapshot::build(&npcs);
        let spatial = NpcSpatial::build(&snap);
        // Kill every living core through the seam's mirror — the buckets
        // are untouched, but the queries must stop seeing the deaths.
        for i in 0..npcs.len() {
            if npcs[i].kind == NpcKind::MartianSaucerCore && npcs[i].hp > 0 {
                npcs[i].hp = 0;
                snap.kill(i);
            }
        }
        let mut rng = Lcg(99);
        for _ in 0..30 {
            let cx = rng.f32(-5000.0, 5000.0);
            let cy = rng.f32(-5000.0, 5000.0);
            assert_eq!(
                spatial.nearest_living_kind(&snap, cx, cy, NpcKind::MartianSaucerCore, 1600.0),
                None
            );
            assert_eq!(
                spatial.count_living_kind_within(
                    &snap,
                    cx,
                    cy,
                    NpcKind::MartianSaucerTurret,
                    1600.0
                ),
                linear_candidates(&npcs, cx, cy, NpcKind::MartianSaucerTurret, 1600.0).len() as i32
            );
        }
    }

    #[test]
    fn the_empty_and_all_dead_pools_match() {
        let empty: Vec<Npc> = Vec::new();
        let snap = NpcSnapshot::build(&empty);
        let spatial = NpcSpatial::build(&snap);
        assert_eq!(
            spatial.nearest_living_kind(&snap, 0.0, 0.0, NpcKind::Scutlix, 1600.0),
            None
        );
        assert_eq!(
            spatial.count_living_kind_within(&snap, 0.0, 0.0, NpcKind::Scutlix, 1600.0),
            0
        );
        let mut npcs = pool(3, 32);
        for n in npcs.iter_mut() {
            n.hp = 0;
        }
        let snap = NpcSnapshot::build(&npcs);
        let spatial = NpcSpatial::build(&snap);
        for want in KINDS {
            assert_eq!(
                spatial.nearest_living_kind(&snap, 100.0, -100.0, want, 1600.0),
                None
            );
            assert_eq!(
                spatial.count_living_kind_within(&snap, 100.0, -100.0, want, 1600.0),
                0
            );
        }
    }
}
