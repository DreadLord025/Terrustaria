# INTENT: ROADMAP rows for the item-mechanics slice (task/items)

Append to `docs/ROADMAP.md` → "Task catalog" (the same table shape as the
`S1-C-*` rows):

| ✅ ITEMS-1 | `KillTile` must drop the tile's item exactly like `KillTile_GetItemDrops` (dirt/grass → Dirt Block 2, stone → 3, torch → 8, …) and spawn it through `KillTile_DropItems` at `x*16, y*16, 16, 16` | S | B | research/07-item-core.md | — | break-a-tile test pinning the item id — **DONE 2026-11-17** (`task/items`): `registry::tile_drop` already carried the style-0 map; `break_tile` → `break_drop_events` → `items::spawn_item_drop` spawns it. Test `item_mechanics::breaking_dirt_drops_dirt_item` pins `tile_drop(0, 0) == Some(2)` and the one-block drop |
| ✅ ITEMS-2 | Pickup must clamp a merge at `Item.CommonMaxStack` (9999) and leave the surplus on the ground: `update_items` used to add the whole stack into the slot (`world_sim/items.rs`, `count += drop_count`) | S | B+P | research/07-item-core.md | — | merge test pinning `maxStack` — **DONE 2026-11-17** (`task/items`): the slot walk mirrors `Player.GetItem`'s `GetItem_FillIntoOccupiedSlot` (`Player.cs:39493-39518`), taking `maxStack - count` per slot and writing the leftover back into the drop (`PickupItem`'s `ReplaceWith`, `:35085-35087`). Test `item_mechanics::pickup_merges_to_maxstack` (9999 cap + a 7-stack remainder drop) |
| ✅ ITEMS-3 | Placing a tile item must consume exactly one from the held stack and write the tile (the torch: item 8 → tile 4) | S | B | research/07-item-core.md | — | place test pinning the tile kind and the stack — **DONE 2026-11-17** (`task/items`): the route `use_tile_item` → `place_tile` → `place_item_and_consume` already held; test `item_mechanics::placing_torch_consumes_item_and_spawns_tile` |
| ✅ ITEMS-4 | Dropped-item physics must use the `WorldItem.cs:363-364` constants (`gravity = 0.1f`, `maxFallSpeed = 7f`), the `MoveInWorld` friction `velocity.X *= 0.95f` with the `< 0.1` snap (`:854-860`) and the `Collision.TileCollision` flush stop (no restitution) — the port ran 0.3 / 6.0 and bounced (`-v*0.35`, `-v*0.3`) | S | B+P | research/07-item-core.md | — | gravity/friction/rest test — **DONE 2026-11-17** (`task/items`): `item.rs`'s `GRAVITY`/`MAX_FALL`/`FRICTION_X`/`FRICTION_EPSILON` + the zeroed collision axes; test `item_mechanics::dropped_item_lies_on_ground_with_csharp_physics`. The longer 0.1-gravity arc retuned the `test_store_1_the_void_lens_routes_pickups_into_bank4` window (40 → 200 ticks) |

## NOT PORTED (out of this slice — no row claimed here)

Doors (`OpenDoor`/`CloseDoor`/`ChangeDoorState`), chest opening + the chest UI
(`Chest.OpenChest`/`ChestUI`), torch light emission and the wall-torch styles,
MP item ownership/`grabDelay`, the `goldRing`/`manaMagnet`/`lifeMagnet`/
`treasureMagnet` grab-range rows, death drops (`Player.DropItems`/
`DropItemsOnDeath`/`DropCoins`), the `Item.NewItem` velocity spread
(`Item.cs:49740-49746`), slopes/wet items in the drop physics, and the
`favorited`/`uniqueStack`/coin-merge slot order.
