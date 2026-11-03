//! `Terraria.ID.ItemID` subset: item definitions (`Item.cs` defaults per type) and
//! the tile/wall/projectile they place or fire.
//!
//! Inventory slots store the vanilla `ItemID`; the old ad-hoc encoding (tile kind,
//! `WALL_ITEM_BASE`, `WEAPON_ITEM_BASE`, `COIN_ITEM_BASE`) is migrated on load.

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum DamageClass {
    #[default]
    None,
    Melee,
    Ranged,
    Magic,
    Summon,
    Thrown,
}

pub struct ItemDef {
    pub id: u16,
    pub name: &'static str,
    pub max_stack: u32,
    /// `Item.createTile` (tile id), if this item places a tile.
    pub tile: Option<u16>,
    /// `Item.placeStyle` (`Item.cs:173`): the style variant this item places
    /// (the `TileObjectData` style that feeds `CalculatePlacementStyle`).
    pub place_style: i32,
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
    /// `Item.headSlot` (`Item.SetDefaults`): the `ArmorHead` texture slot; 0 =
    /// not a helmet.
    pub head_slot: u8,
    /// `Item.bodySlot`: the `ArmorBodyComposite` slot; 0 = not chest armor.
    pub body_slot: u8,
    /// `Item.legSlot`: the `ArmorLeg` slot; 0 = not leg armor.
    pub leg_slot: u8,
    /// `Item.defense` (`Item.SetDefaults`).
    pub defense: u8,
    /// The item’s damage class (`Item.melee`/`ranged`/`magic`/`summon`;
    /// classless items roll no class crit and pass the 1f multiplier).
    pub class: DamageClass,
    /// `Item.crit` — added to the player’s class crit (`Player.cs:2430`).
    pub crit: i32,
}

pub const COIN_VALUES: [u64; 4] = [1, 100, 10_000, 1_000_000];

static ITEMS: &[ItemDef] = &[
    // Terraria.ID.ItemID.Gel (23): a crafting material — `SetDefaults1`'s
    // case 23 sets nothing, so the `ResetStats` defaults apply (useStyle 0,
    // no use timings; `Item.cs:49020-49022`).
    ItemDef { id: 23, name: "Gel", max_stack: 999, tile: None, place_style: 0, wall: None, projectile: None, damage: 0, rare: 0, use_time: 0, use_animation: 0, use_style: 0, hold_style: 0, auto_reuse: false, head_slot: 0, body_slot: 0, leg_slot: 0, defense: 0 , class: DamageClass::None, crit: 0 },
    // Terraria.ID.ItemID.PinkGel (3111, Pinky's drop).
    ItemDef { id: 3111, name: "Pink Gel", max_stack: 999, tile: None, place_style: 0, wall: None, projectile: None, damage: 0, rare: 1, use_time: 0, use_animation: 0, use_style: 0, hold_style: 0, auto_reuse: false, head_slot: 0, body_slot: 0, leg_slot: 0, defense: 0 , class: DamageClass::None, crit: 0 },
    // Terraria.ID.ItemID.DirtBlock
    ItemDef {
        id: 2,
        name: "Dirt Block",
        max_stack: 999,
        tile: Some(0),
        place_style: 0,
        wall: None,
        projectile: None,
        damage: 0,
        rare: 0,
        use_time: 10,
        use_animation: 15,
        use_style: 1,
        hold_style: 0,
        auto_reuse: true,
        head_slot: 0,
        body_slot: 0,
        leg_slot: 0,
        defense: 0,
        class: DamageClass::None,
        crit: 0,
    },
    // StoneBlock
    ItemDef {
        id: 3,
        name: "Stone Block",
        max_stack: 999,
        tile: Some(1),
        place_style: 0,
        wall: None,
        projectile: None,
        damage: 0,
        rare: 0,
        use_time: 10,
        use_animation: 15,
        use_style: 1,
        hold_style: 0,
        auto_reuse: true,
        head_slot: 0,
        body_slot: 0,
        leg_slot: 0,
        defense: 0,
        class: DamageClass::None,
        crit: 0,
    },
    // Torch
    ItemDef {
        id: 8,
        name: "Torch",
        max_stack: 999,
        tile: Some(4),
        place_style: 0,
        wall: None,
        projectile: None,
        damage: 0,
        rare: 0,
        use_time: 10,
        use_animation: 15,
        use_style: 1,
        hold_style: 1,
        auto_reuse: true,
        head_slot: 0,
        body_slot: 0,
        leg_slot: 0,
        defense: 0,
        class: DamageClass::None,
        crit: 0,
    },
    // Wood
    ItemDef {
        id: 9,
        name: "Wood",
        max_stack: 999,
        tile: Some(5),
        place_style: 0,
        wall: None,
        projectile: None,
        damage: 0,
        rare: 0,
        use_time: 10,
        use_animation: 15,
        use_style: 1,
        hold_style: 0,
        auto_reuse: true,
        head_slot: 0,
        body_slot: 0,
        leg_slot: 0,
        defense: 0,
        class: DamageClass::None,
        crit: 0,
    },
    ItemDef {
        id: 11,
        name: "Iron Ore",
        max_stack: 999,
        tile: Some(6),
        place_style: 0,
        wall: None,
        projectile: None,
        damage: 0,
        rare: 0,
        use_time: 10,
        use_animation: 15,
        use_style: 1,
        hold_style: 0,
        auto_reuse: true,
        head_slot: 0,
        body_slot: 0,
        leg_slot: 0,
        defense: 0,
        class: DamageClass::None,
        crit: 0,
    },
    ItemDef {
        id: 12,
        name: "Copper Ore",
        max_stack: 999,
        tile: Some(7),
        place_style: 0,
        wall: None,
        projectile: None,
        damage: 0,
        rare: 0,
        use_time: 10,
        use_animation: 15,
        use_style: 1,
        hold_style: 0,
        auto_reuse: true,
        head_slot: 0,
        body_slot: 0,
        leg_slot: 0,
        defense: 0,
        class: DamageClass::None,
        crit: 0,
    },
    ItemDef {
        id: 13,
        name: "Gold Ore",
        max_stack: 999,
        tile: Some(8),
        place_style: 0,
        wall: None,
        projectile: None,
        damage: 0,
        rare: 0,
        use_time: 10,
        use_animation: 15,
        use_style: 1,
        hold_style: 0,
        auto_reuse: true,
        head_slot: 0,
        body_slot: 0,
        leg_slot: 0,
        defense: 0,
        class: DamageClass::None,
        crit: 0,
    },
    ItemDef {
        id: 14,
        name: "Silver Ore",
        max_stack: 999,
        tile: Some(9),
        place_style: 0,
        wall: None,
        projectile: None,
        damage: 0,
        rare: 0,
        use_time: 10,
        use_animation: 15,
        use_style: 1,
        hold_style: 0,
        auto_reuse: true,
        head_slot: 0,
        body_slot: 0,
        leg_slot: 0,
        defense: 0,
        class: DamageClass::None,
        crit: 0,
    },
    ItemDef {
        id: 26,
        name: "Stone Wall",
        max_stack: 999,
        tile: None,
        place_style: 0,
        wall: Some(1),
        projectile: None,
        damage: 0,
        rare: 0,
        use_time: 7,
        use_animation: 15,
        use_style: 1,
        hold_style: 0,
        auto_reuse: true,
        head_slot: 0,
        body_slot: 0,
        leg_slot: 0,
        defense: 0,
        class: DamageClass::None,
        crit: 0,
    },
    ItemDef {
        id: 30,
        name: "Dirt Wall",
        max_stack: 999,
        tile: None,
        place_style: 0,
        wall: Some(2),
        projectile: None,
        damage: 0,
        rare: 0,
        use_time: 7,
        use_animation: 15,
        use_style: 1,
        hold_style: 0,
        auto_reuse: true,
        head_slot: 0,
        body_slot: 0,
        leg_slot: 0,
        defense: 0,
        class: DamageClass::None,
        crit: 0,
    },
    // Shuriken: vanilla `Item.shoot = 3`; our projectile table has a single id.
    ItemDef {
        id: 42,
        name: "Shuriken",
        max_stack: 999,
        tile: None,
        place_style: 0,
        wall: None,
        projectile: Some(crate::projectile::SHURIKEN),
        damage: 15,
        rare: 0,
        use_time: 15,
        use_animation: 15,
        use_style: 1,
        hold_style: 0,
        auto_reuse: false,
        head_slot: 0,
        body_slot: 0,
        leg_slot: 0,
        defense: 0,
        class: DamageClass::Ranged,
        crit: 0,
    },
    // Coins (`Item.maxStack = 100`)
    ItemDef {
        id: 71,
        name: "Copper Coin",
        max_stack: 100,
        tile: None,
        place_style: 0,
        wall: None,
        projectile: None,
        damage: 0,
        rare: 0,
        use_time: 10,
        use_animation: 15,
        use_style: 1,
        hold_style: 0,
        auto_reuse: true,
        head_slot: 0,
        body_slot: 0,
        leg_slot: 0,
        defense: 0,
        class: DamageClass::None,
        crit: 0,
    },
    ItemDef {
        id: 72,
        name: "Silver Coin",
        max_stack: 100,
        tile: None,
        place_style: 0,
        wall: None,
        projectile: None,
        damage: 0,
        rare: 0,
        use_time: 10,
        use_animation: 15,
        use_style: 1,
        hold_style: 0,
        auto_reuse: true,
        head_slot: 0,
        body_slot: 0,
        leg_slot: 0,
        defense: 0,
        class: DamageClass::None,
        crit: 0,
    },
    ItemDef {
        id: 73,
        name: "Gold Coin",
        max_stack: 100,
        tile: None,
        place_style: 0,
        wall: None,
        projectile: None,
        damage: 0,
        rare: 0,
        use_time: 10,
        use_animation: 15,
        use_style: 1,
        hold_style: 0,
        auto_reuse: true,
        head_slot: 0,
        body_slot: 0,
        leg_slot: 0,
        defense: 0,
        class: DamageClass::None,
        crit: 0,
    },
    ItemDef {
        id: 74,
        name: "Platinum Coin",
        max_stack: 100,
        tile: None,
        place_style: 0,
        wall: None,
        projectile: None,
        damage: 0,
        rare: 0,
        use_time: 10,
        use_animation: 15,
        use_style: 1,
        hold_style: 0,
        auto_reuse: true,
        head_slot: 0,
        body_slot: 0,
        leg_slot: 0,
        defense: 0,
        class: DamageClass::None,
        crit: 0,
    },
    ItemDef {
        id: 169,
        name: "Sand Block",
        max_stack: 999,
        tile: Some(53),
        place_style: 0,
        wall: None,
        projectile: None,
        damage: 0,
        rare: 0,
        use_time: 10,
        use_animation: 15,
        use_style: 1,
        hold_style: 0,
        auto_reuse: true,
        head_slot: 0,
        body_slot: 0,
        leg_slot: 0,
        defense: 0,
        class: DamageClass::None,
        crit: 0,
    },
    ItemDef {
        id: 172,
        name: "Ash Block",
        max_stack: 999,
        tile: Some(57),
        place_style: 0,
        wall: None,
        projectile: None,
        damage: 0,
        rare: 0,
        use_time: 10,
        use_animation: 15,
        use_style: 1,
        hold_style: 0,
        auto_reuse: true,
        head_slot: 0,
        body_slot: 0,
        leg_slot: 0,
        defense: 0,
        class: DamageClass::None,
        crit: 0,
    },
    ItemDef {
        id: 174,
        name: "Hellstone",
        max_stack: 999,
        tile: Some(58),
        place_style: 0,
        wall: None,
        projectile: None,
        damage: 0,
        rare: 0,
        use_time: 10,
        use_animation: 15,
        use_style: 1,
        hold_style: 0,
        auto_reuse: true,
        head_slot: 0,
        body_slot: 0,
        leg_slot: 0,
        defense: 0,
        class: DamageClass::None,
        crit: 0,
    },
    ItemDef {
        id: 276,
        name: "Cactus",
        max_stack: 999,
        tile: Some(80),
        place_style: 0,
        wall: None,
        projectile: None,
        damage: 0,
        rare: 0,
        use_time: 10,
        use_animation: 15,
        use_style: 1,
        hold_style: 0,
        auto_reuse: true,
        head_slot: 0,
        body_slot: 0,
        leg_slot: 0,
        defense: 0,
        class: DamageClass::None,
        crit: 0,
    },
    ItemDef {
        id: 593,
        name: "Snow Block",
        max_stack: 999,
        tile: Some(147),
        place_style: 0,
        wall: None,
        projectile: None,
        damage: 0,
        rare: 0,
        use_time: 10,
        use_animation: 15,
        use_style: 1,
        hold_style: 0,
        auto_reuse: true,
        head_slot: 0,
        body_slot: 0,
        leg_slot: 0,
        defense: 0,
        class: DamageClass::None,
        crit: 0,
    },
    // Terraria.ID.ItemID.Gel (`ItemID.cs:1729`) — slime drops, torch ingredient.
    ItemDef {
        id: 23,
        name: "Gel",
        max_stack: 999,
        tile: None,
        place_style: 0,
        wall: None,
        projectile: None,
        damage: 0,
        rare: 0,
        use_time: 10,
        use_animation: 15,
        use_style: 1,
        hold_style: 0,
        auto_reuse: false,
        head_slot: 0,
        body_slot: 0,
        leg_slot: 0,
        defense: 0,
        class: DamageClass::None,
        crit: 0,
    },
    // IronBar (`ItemID.cs:1727`).
    ItemDef {
        id: 22,
        name: "Iron Bar",
        max_stack: 999,
        tile: None,
        place_style: 0,
        wall: None,
        projectile: None,
        damage: 0,
        rare: 0,
        use_time: 10,
        use_animation: 15,
        use_style: 1,
        hold_style: 0,
        auto_reuse: false,
        head_slot: 0,
        body_slot: 0,
        leg_slot: 0,
        defense: 0,
        class: DamageClass::None,
        crit: 0,
    },
    // WorkBench (`ItemID.cs:1755`) — places the WorkBench tile 18 (`TileID.cs:665`).
    ItemDef {
        id: 36,
        name: "Work Bench",
        max_stack: 999,
        tile: Some(18),
        place_style: 0,
        wall: None,
        projectile: None,
        damage: 0,
        rare: 0,
        use_time: 10,
        use_animation: 15,
        use_style: 1,
        hold_style: 0,
        auto_reuse: true,
        head_slot: 0,
        body_slot: 0,
        leg_slot: 0,
        defense: 0,
        class: DamageClass::None,
        crit: 0,
    },
    // Furnace (`ItemID.cs:1749`) — places the Furnace tile 17.
    ItemDef {
        id: 33,
        name: "Furnace",
        max_stack: 999,
        tile: Some(17),
        place_style: 0,
        wall: None,
        projectile: None,
        damage: 0,
        rare: 0,
        use_time: 10,
        use_animation: 15,
        use_style: 1,
        hold_style: 0,
        auto_reuse: true,
        head_slot: 0,
        body_slot: 0,
        leg_slot: 0,
        defense: 0,
        class: DamageClass::None,
        crit: 0,
    },
    // IronAnvil (`ItemID.cs:1753`) — places the Anvils tile 16.
    ItemDef {
        id: 35,
        name: "Iron Anvil",
        max_stack: 999,
        tile: Some(16),
        place_style: 0,
        wall: None,
        projectile: None,
        damage: 0,
        rare: 0,
        use_time: 10,
        use_animation: 15,
        use_style: 1,
        hold_style: 0,
        auto_reuse: false,
        head_slot: 0,
        body_slot: 0,
        leg_slot: 0,
        defense: 0,
        class: DamageClass::None,
        crit: 0,
    },
    // WoodenDoor (`ItemID.cs:1733`) — places the closed door tile 10
    // (`Item.cs:1677-1687`; no `autoReuse`, `placeStyle` unset → 0).
    ItemDef {
        id: 25,
        name: "Wooden Door",
        max_stack: 999,
        tile: Some(10),
        place_style: 0,
        wall: None,
        projectile: None,
        damage: 0,
        rare: 0,
        use_time: 10,
        use_animation: 15,
        use_style: 1,
        hold_style: 0,
        auto_reuse: false,
        head_slot: 0,
        body_slot: 0,
        leg_slot: 0,
        defense: 0,
        class: DamageClass::None,
        crit: 0,
    },
    // WoodenTable (`ItemID.cs:1747`) — places the table tile 14
    // (`Item.cs:1758-1768`).
    ItemDef {
        id: 32,
        name: "Wooden Table",
        max_stack: 999,
        tile: Some(14),
        place_style: 0,
        wall: None,
        projectile: None,
        damage: 0,
        rare: 0,
        use_time: 10,
        use_animation: 15,
        use_style: 1,
        hold_style: 0,
        auto_reuse: true,
        head_slot: 0,
        body_slot: 0,
        leg_slot: 0,
        defense: 0,
        class: DamageClass::None,
        crit: 0,
    },
    // WoodenChair (`ItemID.cs:1751`) — places the chair tile 15
    // (`Item.cs:1784-1794`).
    ItemDef {
        id: 34,
        name: "Wooden Chair",
        max_stack: 999,
        tile: Some(15),
        place_style: 0,
        wall: None,
        projectile: None,
        damage: 0,
        rare: 0,
        use_time: 10,
        use_animation: 15,
        use_style: 1,
        hold_style: 0,
        auto_reuse: true,
        head_slot: 0,
        body_slot: 0,
        leg_slot: 0,
        defense: 0,
        class: DamageClass::None,
        crit: 0,
    },
    // Chest (`ItemID.cs:1779`) — places the chest tile 21 (`Item.cs:1958-1968`);
    // a pure tile object here — no Chest entity/storage (M3-CHEST-UI seam).
    ItemDef {
        id: 48,
        name: "Chest",
        max_stack: 999,
        tile: Some(21),
        place_style: 0,
        wall: None,
        projectile: None,
        damage: 0,
        rare: 0,
        use_time: 10,
        use_animation: 15,
        use_style: 1,
        hold_style: 0,
        auto_reuse: true,
        head_slot: 0,
        body_slot: 0,
        leg_slot: 0,
        defense: 0,
        class: DamageClass::None,
        crit: 0,
    },
    // The early-metal armor sets (`Item.SetDefaults`): equipSlot ids 1..4 index
    // the `Armor_Head_{n}` / `Armor/Armor_{n}` / `Armor_Legs_{n}` sheets and
    // `Item.defense` feeds `Player.statDefense`. Copper helmet/chainmail/
    // greaves = 89/80/76, iron = 90/81/77, silver = 91/82/78, gold = 92/83/79.
    ItemDef { id: 89, name: "Copper Helmet", max_stack: 1, tile: None, place_style: 0, wall: None, projectile: None, damage: 0, rare: 0, use_time: 0, use_animation: 0, use_style: 0, hold_style: 0, auto_reuse: false, head_slot: 1, body_slot: 0, leg_slot: 0, defense: 1 , class: DamageClass::None, crit: 0 },
    ItemDef { id: 80, name: "Copper Chainmail", max_stack: 1, tile: None, place_style: 0, wall: None, projectile: None, damage: 0, rare: 0, use_time: 0, use_animation: 0, use_style: 0, hold_style: 0, auto_reuse: false, head_slot: 0, body_slot: 1, leg_slot: 0, defense: 2 , class: DamageClass::None, crit: 0 },
    ItemDef { id: 76, name: "Copper Greaves", max_stack: 1, tile: None, place_style: 0, wall: None, projectile: None, damage: 0, rare: 0, use_time: 0, use_animation: 0, use_style: 0, hold_style: 0, auto_reuse: false, head_slot: 0, body_slot: 0, leg_slot: 1, defense: 1 , class: DamageClass::None, crit: 0 },
    ItemDef { id: 90, name: "Iron Helmet", max_stack: 1, tile: None, place_style: 0, wall: None, projectile: None, damage: 0, rare: 0, use_time: 0, use_animation: 0, use_style: 0, hold_style: 0, auto_reuse: false, head_slot: 2, body_slot: 0, leg_slot: 0, defense: 2 , class: DamageClass::None, crit: 0 },
    ItemDef { id: 81, name: "Iron Chainmail", max_stack: 1, tile: None, place_style: 0, wall: None, projectile: None, damage: 0, rare: 0, use_time: 0, use_animation: 0, use_style: 0, hold_style: 0, auto_reuse: false, head_slot: 0, body_slot: 2, leg_slot: 0, defense: 3 , class: DamageClass::None, crit: 0 },
    ItemDef { id: 77, name: "Iron Greaves", max_stack: 1, tile: None, place_style: 0, wall: None, projectile: None, damage: 0, rare: 0, use_time: 0, use_animation: 0, use_style: 0, hold_style: 0, auto_reuse: false, head_slot: 0, body_slot: 0, leg_slot: 2, defense: 2 , class: DamageClass::None, crit: 0 },
    ItemDef { id: 91, name: "Silver Helmet", max_stack: 1, tile: None, place_style: 0, wall: None, projectile: None, damage: 0, rare: 1, use_time: 0, use_animation: 0, use_style: 0, hold_style: 0, auto_reuse: false, head_slot: 3, body_slot: 0, leg_slot: 0, defense: 3 , class: DamageClass::None, crit: 0 },
    ItemDef { id: 82, name: "Silver Chainmail", max_stack: 1, tile: None, place_style: 0, wall: None, projectile: None, damage: 0, rare: 1, use_time: 0, use_animation: 0, use_style: 0, hold_style: 0, auto_reuse: false, head_slot: 0, body_slot: 3, leg_slot: 0, defense: 4 , class: DamageClass::None, crit: 0 },
    ItemDef { id: 78, name: "Silver Greaves", max_stack: 1, tile: None, place_style: 0, wall: None, projectile: None, damage: 0, rare: 1, use_time: 0, use_animation: 0, use_style: 0, hold_style: 0, auto_reuse: false, head_slot: 0, body_slot: 0, leg_slot: 3, defense: 3 , class: DamageClass::None, crit: 0 },
    ItemDef { id: 92, name: "Gold Helmet", max_stack: 1, tile: None, place_style: 0, wall: None, projectile: None, damage: 0, rare: 1, use_time: 0, use_animation: 0, use_style: 0, hold_style: 0, auto_reuse: false, head_slot: 4, body_slot: 0, leg_slot: 0, defense: 4 , class: DamageClass::None, crit: 0 },
    ItemDef { id: 83, name: "Gold Chainmail", max_stack: 1, tile: None, place_style: 0, wall: None, projectile: None, damage: 0, rare: 1, use_time: 0, use_animation: 0, use_style: 0, hold_style: 0, auto_reuse: false, head_slot: 0, body_slot: 4, leg_slot: 0, defense: 5 , class: DamageClass::None, crit: 0 },
    ItemDef { id: 79, name: "Gold Greaves", max_stack: 1, tile: None, place_style: 0, wall: None, projectile: None, damage: 0, rare: 1, use_time: 0, use_animation: 0, use_style: 0, hold_style: 0, auto_reuse: false, head_slot: 0, body_slot: 0, leg_slot: 4, defense: 4 , class: DamageClass::None, crit: 0 },
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
/// Multi-tile objects drop through the kill cascade's object record; the
/// per-kind mapping below is the style-0 slice of the vanilla drop tables
/// (`GetDoorItem`/`GetItemDrop_Tables`/`GetItemDrop_Chair`/
/// `GetItemDrop_Workbenches`/`GetItemDrop_Chests`) — for 10/14/15/18/21 it
/// equals `by_tile` (the placing item), and the open door (11, which no item
/// places) drops the closed-door item (`GetDoorItem(0)` = 25).
pub fn tile_drop(tile: u16) -> Option<u16> {
    match tile {
        2 => Some(2),
        11 => Some(25),
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

/// `Item.placeStyle` (`Item.cs:173`, field default 0).
pub fn place_style(id: u16) -> i32 {
    get(id).map_or(0, |d| d.place_style)
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
    #![allow(non_snake_case)] // DW-IDs are part of the test names (traceability)
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

    #[test]
    fn test_DW_2_2_wooden_set_item_data_matches_item_cs() {
        // (id, createTile, autoReuse) — `Item.cs:1677-1687` (WoodenDoor, the
        // one placement without `autoReuse`), `:1758-1768` (WoodenTable),
        // `:1784-1794` (WoodenChair), `:1958-1968` (Chest), `:1806-1816`
        // (WorkBench). Every SetDefaults case leaves `placeStyle` unset → 0,
        // and `maxStack` stays `Item.CommonMaxStack` (999, `Item.cs:49017`).
        let cases = [
            (25u16, 10u16, false),
            (32, 14, true),
            (34, 15, true),
            (48, 21, true),
            (36, 18, true),
        ];
        for (id, tile, auto_reuse) in cases {
            let d = get(id).unwrap_or_else(|| panic!("item {id} missing"));
            assert_eq!(d.tile, Some(tile), "id {id}");
            assert_eq!(d.place_style, 0, "id {id}");
            assert_eq!(
                (d.use_time, d.use_animation, d.use_style),
                (10, 15, 1),
                "id {id}"
            );
            assert_eq!(d.auto_reuse, auto_reuse, "id {id}");
            assert_eq!(d.max_stack, 999, "id {id}");
        }
        // The item↔tile mapping is bidirectional where the set is complete.
        assert_eq!(by_tile(10), Some(25));
        assert_eq!(by_tile(14), Some(32));
        assert_eq!(by_tile(15), Some(34));
        assert_eq!(by_tile(21), Some(48));
        assert_eq!(place_style(9999), 0, "unknown items default to style 0");
    }

    #[test]
    fn test_DW_3_2_tile_drop_wooden_set() {
        // The kill-cascade drop mapping — the style-0 slice of the vanilla
        // drop tables: `GetDoorItem(0)` = 25 (`WorldGen.cs:69200-69204`),
        // `GetItemDrop_Tables(0)` = 32 (:42194), `GetItemDrop_Chair(0)` = 34
        // (:42635), `GetItemDrop_Workbenches(0)` = 36 (:42447),
        // `GetItemDrop_Chests(0, false)` = 48 (:41677-41681). The open door
        // (11) has no placing item and needs the explicit row — it drops the
        // closed-door item (`CheckDoorOpen` → `DropDoorItem` :86415).
        assert_eq!(tile_drop(10), Some(25));
        assert_eq!(tile_drop(11), Some(25));
        assert_eq!(tile_drop(14), Some(32));
        assert_eq!(tile_drop(15), Some(34));
        assert_eq!(tile_drop(18), Some(36));
        assert_eq!(tile_drop(21), Some(48));
        // Non-object kinds keep their existing rows.
        assert_eq!(tile_drop(2), Some(2), "grass drops dirt");
        assert_eq!(tile_drop(1), Some(3), "stone drops via its item");
    }
}
