# INTENT: PORT_MAP rows for the item-mechanics slice (task/items)

Append the rows below to `docs/PORT_MAP.md` → section **Game / presentation
(`crates/tr-app`)** — next to the existing `Item.cs, GetItemSettings.cs,
NewItemOwnership.cs` row (`crates/tr-app/src/item.rs`, `partial`), which these
rows refine. Do not duplicate that row; replace or extend it.

| C# source (under `Terraria/`) | Rust target | State |
| --- | --- | --- |
| `Player.GrabItems` + `Player.GetItemGrabRange` (`Player.cs:34905-34960`, `:35128-35168`) — the `defaultItemGrabRange = 42` px box, the magnet pull and the `Hitbox.Intersects` direct pickup | `crates/tr-app/src/item.rs` (`ItemDrop::update`'s `MAGNET_RADIUS = 42.0` + `PICKUP_DELAY`), `crates/tr-app/src/world_sim/items.rs::update_items` | partial (ITEMS-2 — the port is SP-only: no `playerIndexTheItemIsReservedFor`/`goldRing`/`manaMagnet`/`lifeMagnet`/`treasureMagnet` grab-range bonuses, and the `PICKUP_DELAY = 0.35` s stand-in replaces `WorldItem.DefaultGrabDelay = 100` ticks. The MP reservation rides `net_reserved`) |
| `Player.GetItem` + `Player.GetItem_FillIntoOccupiedSlot` + `Player.PickupItem` (`Player.cs:35043-35098`, `:39330-39385`, `:39485-39518`) — the 50-slot walk, `Item.CanStack` (same type AND prefix), the `maxStack` clamp and the surplus left as a world drop | `crates/tr-app/src/item.rs::slot_for_pickup`, `crates/tr-app/src/world_sim/items.rs::update_items` (the pending/moved slot walk) | done (ITEMS-2 — `Item.CommonMaxStack = 9999`, `Item.cs:49966`, is what `registry::max_stack` carries; the coins/ammo-slot routing and the `favorited` slot order stay app-side) |
| `WorldItem.UpdateItem` + `WorldItem.MoveInWorld` (`WorldItem.cs:338-419`, `:772-1065`) — the air physics (`gravity = 0.1f`, `maxFallSpeed = 7f`), the `velocity.X *= 0.95f` friction with the `< 0.1` rest snap, the `Collision.TileCollision` flush stop | `crates/tr-app/src/item.rs` (`GRAVITY`/`MAX_FALL`/`FRICTION_X`/`FRICTION_EPSILON`, `ItemDrop::update`) | partial (ITEMS-4 — the air case only: the wet/honey/shimmer gravity rows, slopes and conveyor belts are their own rows; the shimmer half is ported in `ItemDrop::update_shimmer`) |
| `WorldGen.KillTile_DropItems` + `WorldGen.KillTile_GetItemDrops` (`WorldGen.cs:63947-63958`, `:63964-66090`) — the per-tile drop table, `dropItemStack`, the `Item.NewItem(x*16, y*16, 16, 16)` spawn | `crates/tr-app/src/registry.rs::tile_drop`, `crates/tr-app/src/world_sim/editing.rs::break_tile` → `break_drop_events` → `items::spawn_item_drop` | partial (ITEMS-1 — the soil/blocks/plants slice with the `tile_drop` style-0 map; the tree family rides `KillTile_GetTreeDrops`, the pot loot rides `SpawnThingsFromPot` — both separate rows) |
| `Player.PlaceThing` + `Item.createTile`/`Item.placeStyle` (`Player.cs:39580-41470`; `Item.cs:33745-33754`, field `:173`) + the one-item consume on a successful place | `crates/tr-app/src/registry.rs::place_tile`/`place_style`, `crates/tr-app/src/world_sim/editing.rs::use_tile_item` → `place_tile` → `place_item_and_consume` | partial (ITEMS-3 — the tile route incl. the torch (8 → tile 4); the `TileObject` dispatcher styles and the wall/paint routes are their own rows) |

## Ported by this slice (do not re-implement)

- `Player.PickupItem`'s stacking, the `maxStack` clamp and the world-surplus
  rule (`world_sim/items.rs`).
- `WorldItem`'s air gravity/friction/rest (`item.rs`).
- `KillTile` → item drop for the soil/block/plant tables (`registry::tile_drop`
  + `break_tile`).
- Item → tile placement with the one-item consume (`use_tile_item`).

## NOT PORTED (this slice's out-of-slice list, each its own row/task)

1. **Doors** — `WorldGen.OpenDoor`/`CloseDoor`/`ChangeDoorState`
   (`WorldGen.cs:37710-37890`), `Player.OpenDoor`/`HitDoor`, the `TileID.Sets.OpenDoorID`
   pair and the door `Check*` cascade: no door state machine exists in the port;
   tiles 10/11 only break/place.
2. **Chests** — `Chest.OpenChest`/`ChestUI` (`Chest.cs:100-260`, `Main.cs:20300+`):
   `chest_open`/loot windows, the `chest` field write on click and the
   `chest_holds_items` break refusal are present as a stub, the UI is not.
3. **Torch presentation** — `Player.ItemCheck_EmitHeldItemLight`, the wall-torch
   variants (`ItemID.Sets.TorchStyle`), `TileObjectData` torch support checks and
   the flicker light rows: the placed tile carries no light source in the port.
4. **Item ownership / grab delay in MP** — `NewItemOwnership`,
   `playerIndexTheItemIsReservedFor`, `WorldItem.DefaultGrabDelay = 100`
   (`WorldItem.cs:110`): the port has the host-side reservation map only.
5. **Grab-range accessories** — `goldRing`/`manaMagnet`/`lifeMagnet`/
   `treasureMagnet` and the star/heart/mana pickup arms of `PickupItem`
   (`Player.cs:35055-35099`).
6. **Death drops** — `Player.DropItems`/`DropItemsOnDeath` (`Player.cs:54302+`),
   `DropCoins`, the tombstone spawn: death currently drops nothing.
7. **Item.NewItem's spawn velocity spread** — the port pops at
   `vx ∈ [-1.4, 1.4]`, `vy ∈ [-3.4, -2.2]` instead of the C#'s
   `rand(-30..31)*0.1` / `rand(-40..-15)*0.1` (`Item.cs:49740-49746`).
8. **Slopes, conveyors and wet items** — `Collision.SlopeCollision`, the
   conveyor push (ported), the wet/honey/shimmer gravity rows
   (`WorldItem.cs:380-396`): the drop physics is the air case.
9. **Prefix merge refinement** — `Item.CanStack` is prefix equality; the
   `favorited`/`uniqueStack`/coin-merge (`DoCoins`) and the emergency
   stacking of `PickupReplacementTime` are app-side or unported.
10. **Sell/buy of drops** — `Player.SellItem`'s per-item math lives in
    `shop.rs` (S1-C-11); the world-drop `GetItemExpectedPrice` path is untouched.
