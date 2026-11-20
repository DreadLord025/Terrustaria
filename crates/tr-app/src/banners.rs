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
    /// `AddNPCKillBy` (`BannerSystem.cs:231-258`): count the kill, and on the
    /// `KillsToBanner` milestone return the banner id whose banner item the
    /// player may claim (the port spawns the item at the death spot) plus
    /// the announcement text — `Game.EnemiesDefeatedByAnnouncement`
    /// (`:254`, `"{0} has defeated the {1}th {2}!"`, color 250,250,0 = the
    /// Command chat color). The NPC name rides `Lang.GetNPCName(netID).Key`
    /// (`:254`) = the `NPCName.*` language row, not the English bestiary
    /// column; the resource-less fallback keeps the C# English wording with
    /// the bestiary's English name (the `lang::item_name` fallback pattern).
    /// The player-less `Game.EnemiesDefeatedAnnouncement` arm (`:251`,
    /// `plr` outside 0..255) has no port caller — every kill pass names its
    /// local player.
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
        // `Lang.GetNPCName(netID)` (`Lang.cs:180-191`) — the localized row;
        // an unresolved key (no resources) falls back to the English column.
        let english = crate::bestiary::ENTRIES
            .iter()
            .find(|e| e.net_id == net)
            .map(|e| e.name)
            .unwrap_or("?");
        let name = crate::lang::npc_name_or(net, english);
        let key = "Game.EnemiesDefeatedByAnnouncement";
        let text = crate::lang::text_args(
            key,
            &[player_name.to_string(), count.to_string(), name.clone()],
        );
        let text = if text == key {
            // The manager answers a missing key with the key itself — keep
            // the C# en-US wording (`en-US.json:816`).
            format!("{player_name} has defeated the {count}th {name}!")
        } else {
            text
        };
        Some((banner, text))
    }

    /// The banner's kill count (the tracker display data).
    pub fn kills(&self, banner: i32) -> i32 {
        self.kills.get(&banner).copied().unwrap_or(0)
    }

    /// The kill pairs in ascending banner-id order (`BannerSystem.Save`'s
    /// per-banner counts, `BannerSystem.cs:172-186`) — the world-save
    /// section's payload; the port's sparse map where the vanilla writes a
    /// dense array.
    pub fn pairs(&self) -> impl Iterator<Item = (i32, i32)> + '_ {
        self.kills.iter().map(|(&banner, &kills)| (banner, kills))
    }

    /// `BannerSystem.Load` (`BannerSystem.cs:188-212`): the recorded counts
    /// replace the tally. The C# keeps only the entries its count array
    /// carries and the port's map only the recorded ones — a recorded 0
    /// stays in the map so a zeroed banner round-trips.
    pub fn load_pairs(&mut self, pairs: impl IntoIterator<Item = (i32, i32)>) {
        for (banner, kills) in pairs {
            self.kills.insert(banner, kills);
        }
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
        // `Game.EnemiesDefeatedByAnnouncement` (`en-US.json:816`): the
        // resource row and the resource-less fallback share the C# wording.
        assert!(
            text.contains("Belsin has defeated the 50th Blue Slime!"),
            "{text}"
        );
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
