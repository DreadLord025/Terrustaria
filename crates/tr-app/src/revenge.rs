//! The coin-loss revenge system — `CoinLossRevengeSystem.cs` + `RevengeMarker`
//! (`:36-58`) behind `NPC.RevengeManager` (`NPC.cs:6437`). The chain: the
//! player dies -> the coin items drop -> an enemy picks them up
//! (`WorldItem.cs:1125-1200`) into `NPC.extraValue` -> the enemy despawns
//! (`NPC.CheckActive` :78000-78002 caches the carrier when `extraValue >
//! 0`) -> the marker -> the player returns -> the spawn pass respawns the
//! carrier with the coins -> killing it drops them (`NPCLoot_DropMoney`
//! :79680 adds `extraValue`).
//!
//! The markers are SESSION-ONLY (network-synced, never world-saved;
//! `WorldGen.clearWorld` -> `RevengeManager.Reset()`, `WorldGen.cs:6980`) —
//! the port keeps a plain Vec on the App.

use crate::npc::{Npc, NpcKind};
use glam::Vec2;

/// `RevengeMarker.MinimumCoinsForCaching` (`:512`) — 10 silver.
pub(crate) const MIN_COINS_FOR_CACHING: u64 = 1_000;

/// `_hitbox`'s `EnemyBoxSize` (`:513`) — 2160x1440.
pub(crate) const ENEMY_BOX: [f32; 2] = [2160.0, 1440.0];

/// One cached carrier (`RevengeMarker`'s fields, `:36-58`).
#[derive(Clone, Debug)]
pub(crate) struct RevengeMarker {
    /// `_location` — the despawn spot, world pixels.
    pub location: Vec2,
    /// `_npcNetID` (the `RespawnEnemyID` remap is the vanilla pair-table;
    /// the port carries the kind straight).
    pub kind: NpcKind,
    /// `_npcHPPercent` — the despawn life percent.
    pub hp_percent: f32,
    /// `_baseValue` — the NPC's own loot value (`npc.value`).
    pub base_value: f32,
    /// `_coinsValue` — the picked-up coins (`npc.extraValue`).
    pub coins_value: u64,
    /// `_expirationTime` — the absolute tick the marker expires at.
    pub expiration_tick: u64,
    /// `_uniqueID`.
    pub unique_id: u64,
    /// `_forceExpire` (`SetToExpire` on the discouragement path).
    pub force_expire: bool,
}

impl RevengeMarker {
    /// `_hitbox` (`:513`) — the despawn-spot-centered 2160x1440 box.
    pub(crate) fn hitbox(&self) -> [f32; 4] {
        [
            self.location.x - ENEMY_BOX[0] * 0.5,
            self.location.y - ENEMY_BOX[1] * 0.5,
            ENEMY_BOX[0],
            ENEMY_BOX[1],
        ]
    }

    pub(crate) fn expired(&self, tick: u64) -> bool {
        self.force_expire || self.expiration_tick <= tick
    }
}

/// `_expirationCompCopper/Silver/Gold/Plat` (`:291-294`): 1c / 1s / 1g / 1p.
const COMP_COPPER: u64 = 1;
const COMP_SILVER: u64 = 100;
const COMP_GOLD: u64 = 10_000;
const COMP_PLAT: u64 = 1_000_000;

/// `Utils.GetLerpValue(from, to, v)` (`Utils.cs`): the 0-1 progress of `v`
/// between the bounds, clamped.
fn get_lerp_value(from: u64, to: u64, v: u64) -> f32 {
    if to <= from {
        return 1.0;
    }
    let t = (v.max(from) - from) as f32 / (to - from) as f32;
    t.clamp(0.0, 1.0)
}

/// The expiration add-on (`:161-165`, verbatim brackets): base 18000 ticks
/// (5 min) plus the value-scaled bracket — <1s: 0-1 min extra; <1g: 10-30
/// min; >=1p: 120 min; else 30-60 min. The last bracket's lerp bounds read
/// the silver/gold comps in this decompile (the noted artifact — ported
/// verbatim).
pub(crate) fn expiration_addon(coin_value: u64) -> u64 {
    let extra: u64 = if coin_value < COMP_SILVER {
        (3600.0
            * get_lerp_value(COMP_COPPER, COMP_SILVER, coin_value.max(COMP_COPPER)))
            as u64
    } else if coin_value < COMP_GOLD {
        (36000.0f32
            + 72000.0 * get_lerp_value(COMP_SILVER, COMP_GOLD, coin_value))
            as u64
    } else if coin_value >= COMP_PLAT {
        432_000
    } else {
        // The artifact bracket: the lerp bounds read the silver/gold comps.
        (108000.0f32
            + 108000.0 * get_lerp_value(COMP_SILVER, COMP_GOLD, coin_value))
            as u64
    };
    extra + 18_000
}

/// `RevengeMarker`'s boss exclusion (`CacheEnemy` :347-371's `npc.boss`):
/// the port's boss kinds (the `NPC.boss` flag rows — the Eye 4, the King
/// Slime 50, the EoW head 13, the Brain 63, the Queen Bee 43, the
/// Skeletron 35, the Wall of Flesh 114, the mech trio 125/126/127).
const BOSS_KINDS: &[NpcKind] = &[
    NpcKind::EyeOfCthulhu,
    NpcKind::QueenBee,
    NpcKind::SkeletronHead,
    NpcKind::WallOfFlesh,
    NpcKind::Retinazer,
    NpcKind::Spazmatism,
    NpcKind::SkeletronPrime,
];

/// `CoinLossRevengeSystem.CacheEnemy`'s gates (`:347-371`): the bosses and
/// the sub-10-silver carriers skip; `CheckActive`'s gate (`:78000`) is the
/// `extraValue > 0` driver. The worm-tail (`realLife != -1`) and the rare
/// (`rarity > 0`) arms have no port mechanic yet.
pub(crate) fn should_cache(n: &Npc) -> bool {
    n.extra_value >= MIN_COINS_FOR_CACHING && !BOSS_KINDS.contains(&n.kind)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expiration_brackets_follow_the_lerp_table() {
        // `:161-165`: base 18000 (5 min) + the brackets — a sub-silver
        // carrier gets 0-1 min, the sub-gold 10-30 min, the sub-plat 30-60
        // min, the platinum 120 min.
        assert_eq!(expiration_addon(1), 18_000, "1 copper: no add-on");
        assert_eq!(expiration_addon(50), 18_000 + 1781, "50c: (50-1)/99 of the 3600 bracket");
        assert_eq!(
            expiration_addon(5_000),
            18_000 + 71_636,
            "50 silver: (5000-100)/9900 of the 36000..108000 lerp"
        );
        assert_eq!(
            expiration_addon(500_000),
            18_000 + 216_000,
            "50 gold: the lerp saturates at the bracket edge (the artifact)"
        );
        assert_eq!(expiration_addon(2_000_000), 18_000 + 432_000, "2 platinum");
    }

    #[test]
    fn cache_gate_needs_ten_silver_and_no_boss() {
        let mut n = Npc::new(NpcKind::Zombie, 0.0, 0.0, 1, 0.0);
        n.extra_value = 999;
        assert!(!should_cache(&n), "9.99 silver skips");
        n.extra_value = 1_000;
        assert!(should_cache(&n));
        n.kind = NpcKind::EyeOfCthulhu;
        assert!(!should_cache(&n), "bosses skip");
    }

    #[test]
    fn hitbox_is_the_2160x1440_box_centered_on_the_spot() {
        let m = RevengeMarker {
            location: Vec2::new(1000.0, 2000.0),
            kind: NpcKind::Zombie,
            hp_percent: 1.0,
            base_value: 0.0,
            coins_value: 1_000,
            expiration_tick: 0,
            unique_id: 1,
            force_expire: false,
        };
        assert_eq!(
            m.hitbox(),
            [1000.0 - 1080.0, 2000.0 - 720.0, 2160.0, 1440.0]
        );
    }
}
