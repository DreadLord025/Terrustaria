//! `Terraria.ID.ItemID` subset: item definitions (`Item.cs` defaults per type) and
//! the tile/wall/projectile they place or fire.
//!
//! Inventory slots store the vanilla `ItemID`; the old ad-hoc encoding (tile kind,
//! `WALL_ITEM_BASE`, `WEAPON_ITEM_BASE`, `COIN_ITEM_BASE`) is migrated on load.

#[derive(Clone, Copy, Debug)]
pub struct ItemDef {
    pub id: u16,
    pub name: &'static str,
    pub max_stack: u32,
    /// `Item.createTile` (tile id), if this item places a tile.
    pub tile: Option<u16>,
    /// `Item.createWall` (wall id), if this item places a wall.
    pub wall: Option<u16>,
    /// `Item.shoot` (projectile id), if this item fires one.
    pub projectile: Option<u16>,
    pub damage: i32,
    /// `Item.rare` (`Item.cs`); drives the tooltip name colour
    /// (`Main.MouseText_DrawItemTooltip_GetItemNameColor`). 0 until the item-data
    /// port fills it in.
    pub rare: i32,
    /// `Item.useTime` (ticks, `Item.SetDefaults`).
    pub use_time: u8,
    /// `Item.useAnimation` (ticks per swing, `Item.SetDefaults`).
    pub use_animation: u8,
    /// `Item.useStyle` (`Item.SetDefaults`; 1 = swing).
    pub use_style: u8,
    /// `Item.holdStyle` (`Item.SetDefaults`; 1 = torch-style held item).
    pub hold_style: u8,
    /// `Item.autoReuse` (`Item.SetDefaults`): holding the use button restarts
    /// the use once `itemAnimation` runs out.
    pub auto_reuse: bool,
}

pub const COIN_VALUES: [u64; 4] = [1, 100, 10_000, 1_000_000];

static ITEMS: &[ItemDef] = &[
    // Terraria.ID.ItemID.Gel (23): a crafting material — `SetDefaults1`'s
    // case 23 sets nothing, so the `ResetStats` defaults apply (useStyle 0,
    // no use timings; `Item.cs:49020-49022`).
    ItemDef { id: 23, name: "Gel", max_stack: 999, tile: None, wall: None, projectile: None, damage: 0, rare: 0, use_time: 0, use_animation: 0, use_style: 0, hold_style: 0, auto_reuse: false },
    // Terraria.ID.ItemID.PinkGel (3111, Pinky's drop).
    ItemDef { id: 3111, name: "Pink Gel", max_stack: 999, tile: None, wall: None, projectile: None, damage: 0, rare: 1, use_time: 0, use_animation: 0, use_style: 0, hold_style: 0, auto_reuse: false },
    // Terraria.ID.ItemID.DirtBlock
    ItemDef {
        id: 2,
        name: "Dirt Block",
        max_stack: 999,
        tile: Some(0),
        wall: None,
        projectile: None,
        damage: 0,
        rare: 0,
        use_time: 10,
        use_animation: 15,
        use_style: 1,
        hold_style: 0,
        auto_reuse: true,
    },
    // StoneBlock
    ItemDef {
        id: 3,
        name: "Stone Block",
        max_stack: 999,
        tile: Some(1),
        wall: None,
        projectile: None,
        damage: 0,
        rare: 0,
        use_time: 10,
        use_animation: 15,
        use_style: 1,
        hold_style: 0,
        auto_reuse: true,
    },
    // Torch
    ItemDef {
        id: 8,
        name: "Torch",
        max_stack: 999,
        tile: Some(4),
        wall: None,
        projectile: None,
        damage: 0,
        rare: 0,
        use_time: 10,
        use_animation: 15,
        use_style: 1,
        hold_style: 1,
        auto_reuse: true,
    },
    // Wood
    ItemDef {
        id: 9,
        name: "Wood",
        max_stack: 999,
        tile: Some(5),
        wall: None,
        projectile: None,
        damage: 0,
        rare: 0,
        use_time: 10,
        use_animation: 15,
        use_style: 1,
        hold_style: 0,
        auto_reuse: true,
    },
    ItemDef {
        id: 11,
        name: "Iron Ore",
        max_stack: 999,
        tile: Some(6),
        wall: None,
        projectile: None,
        damage: 0,
        rare: 0,
        use_time: 10,
        use_animation: 15,
        use_style: 1,
        hold_style: 0,
        auto_reuse: true,
    },
    ItemDef {
        id: 12,
        name: "Copper Ore",
        max_stack: 999,
        tile: Some(7),
        wall: None,
        projectile: None,
        damage: 0,
        rare: 0,
        use_time: 10,
        use_animation: 15,
        use_style: 1,
        hold_style: 0,
        auto_reuse: true,
    },
    ItemDef {
        id: 13,
        name: "Gold Ore",
        max_stack: 999,
        tile: Some(8),
        wall: None,
        projectile: None,
        damage: 0,
        rare: 0,
        use_time: 10,
        use_animation: 15,
        use_style: 1,
        hold_style: 0,
        auto_reuse: true,
    },
    ItemDef {
        id: 14,
        name: "Silver Ore",
        max_stack: 999,
        tile: Some(9),
        wall: None,
        projectile: None,
        damage: 0,
        rare: 0,
        use_time: 10,
        use_animation: 15,
        use_style: 1,
        hold_style: 0,
        auto_reuse: true,
    },
    ItemDef {
        id: 26,
        name: "Stone Wall",
        max_stack: 999,
        tile: None,
        wall: Some(1),
        projectile: None,
        damage: 0,
        rare: 0,
        use_time: 7,
        use_animation: 15,
        use_style: 1,
        hold_style: 0,
        auto_reuse: true,
    },
    ItemDef {
        id: 30,
        name: "Dirt Wall",
        max_stack: 999,
        tile: None,
        wall: Some(2),
        projectile: None,
        damage: 0,
        rare: 0,
        use_time: 7,
        use_animation: 15,
        use_style: 1,
        hold_style: 0,
        auto_reuse: true,
    },
    // Shuriken: vanilla `Item.shoot = 3`; our projectile table has a single id.
    ItemDef {
        id: 42,
        name: "Shuriken",
        max_stack: 999,
        tile: None,
        wall: None,
        projectile: Some(crate::projectile::SHURIKEN),
        damage: 15,
        rare: 0,
        use_time: 15,
        use_animation: 15,
        use_style: 1,
        hold_style: 0,
        auto_reuse: false,
    },
    // Coins (`Item.maxStack = 100`)
    ItemDef {
        id: 71,
        name: "Copper Coin",
        max_stack: 100,
        tile: None,
        wall: None,
        projectile: None,
        damage: 0,
        rare: 0,
        use_time: 10,
        use_animation: 15,
        use_style: 1,
        hold_style: 0,
        auto_reuse: true,
    },
    ItemDef {
        id: 72,
        name: "Silver Coin",
        max_stack: 100,
        tile: None,
        wall: None,
        projectile: None,
        damage: 0,
        rare: 0,
        use_time: 10,
        use_animation: 15,
        use_style: 1,
        hold_style: 0,
        auto_reuse: true,
    },
    ItemDef {
        id: 73,
        name: "Gold Coin",
        max_stack: 100,
        tile: None,
        wall: None,
        projectile: None,
        damage: 0,
        rare: 0,
        use_time: 10,
        use_animation: 15,
        use_style: 1,
        hold_style: 0,
        auto_reuse: true,
    },
    ItemDef {
        id: 74,
        name: "Platinum Coin",
        max_stack: 100,
        tile: None,
        wall: None,
        projectile: None,
        damage: 0,
        rare: 0,
        use_time: 10,
        use_animation: 15,
        use_style: 1,
        hold_style: 0,
        auto_reuse: true,
    },
    // SandBlock / AshBlock / Hellstone / Cactus / SnowBlock
    ItemDef {
        id: 169,
        name: "Sand Block",
        max_stack: 999,
        tile: Some(53),
        wall: None,
        projectile: None,
        damage: 0,
        rare: 0,
        use_time: 10,
        use_animation: 15,
        use_style: 1,
        hold_style: 0,
        auto_reuse: true,
    },
    ItemDef {
        id: 172,
        name: "Ash Block",
        max_stack: 999,
        tile: Some(57),
        wall: None,
        projectile: None,
        damage: 0,
        rare: 0,
        use_time: 10,
        use_animation: 15,
        use_style: 1,
        hold_style: 0,
        auto_reuse: true,
    },
    ItemDef {
        id: 174,
        name: "Hellstone",
        max_stack: 999,
        tile: Some(58),
        wall: None,
        projectile: None,
        damage: 0,
        rare: 0,
        use_time: 10,
        use_animation: 15,
        use_style: 1,
        hold_style: 0,
        auto_reuse: true,
    },
    ItemDef {
        id: 276,
        name: "Cactus",
        max_stack: 999,
        tile: Some(80),
        wall: None,
        projectile: None,
        damage: 0,
        rare: 0,
        use_time: 10,
        use_animation: 15,
        use_style: 1,
        hold_style: 0,
        auto_reuse: true,
    },
    ItemDef {
        id: 593,
        name: "Snow Block",
        max_stack: 999,
        tile: Some(147),
        wall: None,
        projectile: None,
        damage: 0,
        rare: 0,
        use_time: 10,
        use_animation: 15,
        use_style: 1,
        hold_style: 0,
        auto_reuse: true,
    },
];

pub fn get(id: u16) -> Option<&'static ItemDef> {
    ITEMS.iter().find(|d| d.id == id)
}

pub fn name(id: u16) -> &'static str {
    get(id).map_or("Item", |d| d.name)
}

pub fn max_stack(id: u16) -> u32 {
    get(id).map_or(999, |d| d.max_stack)
}

/// Item dropped by breaking `tile` (`WorldGen.KillTile` drops dirt for grass).
pub fn tile_drop(tile: u16) -> Option<u16> {
    match tile {
        2 => Some(2),
        _ => by_tile(tile),
    }
}

/// Item that places `tile`, if the registry has one.
pub fn by_tile(tile: u16) -> Option<u16> {
    ITEMS.iter().find(|d| d.tile == Some(tile)).map(|d| d.id)
}

/// Item that places `wall`, if the registry has one.
pub fn by_wall(wall: u16) -> Option<u16> {
    ITEMS.iter().find(|d| d.wall == Some(wall)).map(|d| d.id)
}

pub fn place_tile(id: u16) -> Option<u16> {
    get(id)?.tile
}

pub fn place_wall(id: u16) -> Option<u16> {
    get(id)?.wall
}

pub fn damage(id: u16) -> i32 {
    get(id).map_or(0, |d| d.damage)
}

/// `Item.rare` (`Item.cs`), used by `Main.MouseText_DrawItemTooltip_GetItemNameColor`.
pub fn rare(id: u16) -> i32 {
    get(id).map_or(0, |d| d.rare)
}

pub fn projectile(id: u16) -> Option<u16> {
    get(id)?.projectile
}

pub fn is_weapon(id: u16) -> bool {
    get(id).is_some_and(|d| d.projectile.is_some())
}

pub fn is_coin(id: u16) -> bool {
    (71..=74).contains(&id)
}

pub fn coin_index(id: u16) -> Option<usize> {
    is_coin(id).then(|| (id - 71) as usize)
}

pub fn coin_value(id: u16) -> u64 {
    coin_index(id)
        .and_then(|i| COIN_VALUES.get(i))
        .copied()
        .unwrap_or(0)
}

/// Coin item id for a denomination (0 copper .. 3 platinum).
pub fn coin_id(denom: usize) -> u16 {
    71 + denom.min(3) as u16
}

/// Migrates a version-1/2 slot kind (tile kind, wall/weapon/coin ranges) to `ItemID`.
pub fn migrate_kind(old: u16) -> u16 {
    match old {
        0..=999 => by_tile(old).unwrap_or(0),
        10_000..=19_999 => by_wall(old - 10_000).unwrap_or(0),
        20_000..=29_999 => 42,
        _ => coin_id((old - 30_000) as usize),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn use_timings_match_item_setdefaults() {
        // Item.cs SetDefaults1: every ported kind swings with useStyle 1 over
        // 15 ticks; blocks take 10 ticks of useTime, the grass seeds 7, the
        // shuriken 15 with autoReuse off.
        for id in [2, 3, 8, 9, 11, 12, 13, 14, 169, 172, 174, 276, 593] {
            let d = get(id).unwrap_or_else(|| panic!("id {id}"));
            assert_eq!(
                (d.use_time, d.use_animation, d.use_style),
                (10, 15, 1),
                "id {id}"
            );
            assert!(d.auto_reuse, "id {id}");
        }
        for id in [26, 30] {
            let d = get(id).unwrap();
            assert_eq!(
                (d.use_time, d.use_animation, d.use_style),
                (7, 15, 1),
                "id {id}"
            );
        }
        let shuriken = get(42).unwrap();
        assert_eq!((shuriken.use_time, shuriken.use_animation), (15, 15));
        assert!(!shuriken.auto_reuse, "the shuriken does not auto-repeat");
        // The torch is the one holdStyle-1 item of the set.
        let torch = get(8).unwrap();
        assert_eq!((torch.hold_style, torch.use_style), (1, 1));
    }

    #[test]
    fn unregistered_kinds_fall_back_to_the_vanilla_default() {
        assert!((crate::item::use_animation_secs(9999) - 0.25).abs() < 1e-6);
        assert_eq!(crate::item::hold_style(9999), 0);
        assert!((crate::item::use_animation_secs(8) - 0.25).abs() < 1e-6);
    }
}
