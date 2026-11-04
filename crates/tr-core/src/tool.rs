//! The tool tiers — `Player.GetPickaxeDamage`'s pick-power gates
//! (`Player.cs:53902-53980`, the pre-hardmode subset the port's world can
//! hit) plus the `Main.tileAxe`/`tileDungeon` sets (`Main.cs:7208-7213`,
//! `:7992-7997`).
//!
//! The vanilla damage model (each swing deals `pickPower`-ish damage into
//! the hit buffer) folds into the port's time-per-tile pacing: the gate
//! decides whether the tile mines at all, and the speed scales as
//! `baseline * 35 / pick` (35 = the copper pick the baseline timings were
//! tuned against).

/// `Main.tileAxe` (`Main.cs:7208-7213`) — the tiles an axe chops.
pub fn is_axe_tile(kind: u16) -> bool {
    matches!(kind, 5 | 72 | 80 | 323 | 488 | 596 | 616 | 704)
}

/// `Main.tileDungeon` (`Main.cs:7992-7997`) — the bricks the dungeon gate
/// covers.
fn is_dungeon_tile(kind: u16) -> bool {
    matches!(kind, 41 | 43 | 44 | 677..=679)
}

/// `GetPickaxeDamage`'s zero-damage gates (`Player.cs:53933-53974`,
/// pre-hardmode): `true` = the pick CAN mine the tile.
pub fn pick_can_mine(kind: u16, pick_power: i32, below_surface: bool) -> bool {
    let p = pick_power;
    let _ = below_surface;
    match kind {
        // `:53941-53942`: the meteorite needs 50.
        37 => p >= 50,
        // `:53943-53945`: ebonstone/crimstone need 65 (the worm-food
        // conversion of `infectedSeed` is not ported).
        25 | 203 => p >= 65,
        // `:53946-53948`: the shadow orb region's 117 needs 65.
        117 => p >= 65,
        // `:53949-53951`: the demonite/crimtane gates only BELOW the
        // surface (`y > Main.worldSurface`).
        22 | 204 => !below_surface || p >= 55,
        // `:53952-53954`: the obsidian needs 55.
        56 => p >= 55,
        // `:53955-53957`: the hellstone needs 65.
        58 => p >= 65,
        // `:53962-53970`: the dungeon bricks need 100 below the surface
        // (the middle-band x check rides the zone; the port gates on the
        // depth alone — deviation until the dungeon band lands).
        k if is_dungeon_tile(k) => !below_surface || p >= 100,
        // `:53971-53973`: the cobalt-family 107/221 need 100.
        107 | 221 => p >= 100,
        _ => true,
    }
}

/// The mining-time multiplier for a pick on a minable tile — the baseline
/// timings assume the copper pick's 35; stronger picks scale linearly.
pub fn pick_speed_scale(pick_power: i32) -> f32 {
    35.0 / pick_power.max(1) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_copper_pick_mines_the_world_but_not_the_gated_tiles() {
        // The copper pick's 35: the plain ores/dirt/stone pass.
        for kind in [0u16, 1, 2, 6, 7, 8, 9, 5] {
            assert!(pick_can_mine(kind, 35, true), "kind {kind}");
        }
        // The gates: the meteorite 50, the ebonstone 65, the hellstone 65,
        // the obsidian 55, the dungeon 100 below the surface.
        assert!(!pick_can_mine(37, 35, false));
        assert!(pick_can_mine(37, 50, false));
        assert!(!pick_can_mine(25, 35, true));
        assert!(pick_can_mine(25, 65, true));
        assert!(!pick_can_mine(58, 35, false));
        assert!(pick_can_mine(58, 65, false));
        assert!(!pick_can_mine(56, 35, true));
        assert!(!pick_can_mine(41, 35, true));
        assert!(pick_can_mine(41, 100, true));
        // The demonite gate only bites below the surface.
        assert!(pick_can_mine(22, 35, false));
        assert!(!pick_can_mine(22, 35, true));
        assert!(pick_can_mine(22, 55, true));
    }

    #[test]
    fn the_axe_and_speed_helpers() {
        assert!(is_axe_tile(5));
        assert!(!is_axe_tile(1));
        // The baseline: the copper pick's 35 scales to 1.0.
        assert!((pick_speed_scale(35) - 1.0).abs() < 1e-6);
        assert!((pick_speed_scale(70) - 0.5).abs() < 1e-6);
    }
}
