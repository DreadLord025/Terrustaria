//! The banner kill tally — `BannerSystem.AddNPCKillBy`
//! (`BannerSystem.cs:163-193`): the per-banner kill counts, the milestone
//! every `KillsToBanner[bannerItem]` kills (the claimable banner + the chat
//! announcement in (250, 250, 0)), and the strength the placed banners
//! grant (`ItemID.BannerEffect`'s classic 1.5/0.75 pair, `tr_core::bestiary`).

use tr_core::bestiary;

/// The `BannerSystem` state — the kill counts by banner id (the port's
/// `killCount[]`, keyed sparsely).
#[derive(Default)]
pub struct BannerSystem {
    kills: std::collections::BTreeMap<i32, i32>,
}

impl BannerSystem {
    /// `AddNPCKillBy` (`:163-193`): count the kill, and on the
    /// `KillsToBanner` milestone return the banner id whose banner item the
    /// player may claim (the port spawns the item at the death spot) plus
    /// the announcement text — `Game.EnemiesDefeatedByAnnouncement`
    /// (`{name} has defeated {count} {npc}!`, color 250,250,0 = the Command
    /// chat color).
    pub fn add_npc_kill_by(&mut self, banner_id: i32, player_name: &str) -> Option<(i32, String)> {
        let banner = bestiary::npc_to_banner(banner_id);
        if banner <= 0 {
            return None;
        }
        let count = self.kills.entry(banner).or_insert(0);
        *count += 1;
        let count = *count;
        let needed = bestiary::kills_needed_for(banner_id);
        if count % needed != 0 {
            return None;
        }
        let net = bestiary::banner_to_npc(banner);
        let name = crate::bestiary::ENTRIES
            .iter()
            .find(|e| e.net_id == net)
            .map(|e| e.name)
            .unwrap_or("?");
        Some((
            banner,
            format!("{player_name} has defeated {count} {name}!"),
        ))
    }

    /// The banner's kill count (the tracker display data).
    pub fn kills(&self, banner: i32) -> i32 {
        self.kills.get(&banner).copied().unwrap_or(0)
    }
}

/// `Player.GetBannerBuffEffect` (`Player.cs:20483-20493`) + the
/// `BannerEffect` classic sample: `Some((1.5, 0.75))` when the NPC's banner
/// stands in the scene window, `None` otherwise.
pub fn banner_buff_for(
    metrics: &tr_core::scene::SceneMetrics,
    net_id: i32,
) -> Option<(f32, f32)> {
    let banner = bestiary::npc_to_banner(net_id);
    if banner <= 0 {
        return None;
    }
    metrics
        .npc_banner_buff
        .get(banner as usize)
        .copied()
        .filter(|b| *b)
        .map(|_| (bestiary::BANNER_DAMAGE_DEALT, bestiary::BANNER_DAMAGE_RECEIVED))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_milestone_lands_every_kills_to_banner() {
        // The blue slime needs 50 kills for the banner milestone; the 50th
        // returns the announcement, the 49th returns nothing.
        let mut sys = BannerSystem::default();
        let mut last = None;
        for _ in 0..49 {
            last = sys.add_npc_kill_by(1, "Belsin");
        }
        assert_eq!(last, None);
        last = sys.add_npc_kill_by(1, "Belsin");
        let (banner, text) = last.take().expect("the 50th kill lands the milestone");
        assert_eq!(banner, 69);
        assert!(text.contains("Belsin has defeated 50 Blue Slime!"), "{text}");
        assert_eq!(sys.kills(69), 50);
        // The Pinky needs only 25 (`ItemID.cs:1304`'s override).
        let mut sys = BannerSystem::default();
        for _ in 0..25 {
            last = sys.add_npc_kill_by(-4, "Belsin");
        }
        assert!(last.is_some());
    }

    #[test]
    fn the_strength_needs_the_placed_banner() {
        let mut metrics = tr_core::scene::SceneMetrics::new();
        assert_eq!(banner_buff_for(&metrics, 1), None);
        metrics.npc_banner_buff[69] = true;
        assert_eq!(banner_buff_for(&metrics, 1), Some((1.5, 0.75)));
        // A net id without a banner never buffs.
        assert_eq!(banner_buff_for(&metrics, 50), None);
    }
}
