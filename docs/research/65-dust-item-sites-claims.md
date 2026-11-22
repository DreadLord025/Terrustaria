# 65 — World-item dust sites (W4-g15): claim + census correction

Status: CLAIM + CENSUS doc (W4-g15, `task/W4-g15-dust-item-sites`). Written
early per the lane's coordination rule — the branch list is the claim record.

## Census correction — the task title's file is wrong

The task brief says "Item.cs Dust.NewDust sites ALL (item use/pickup/spawn
dust)". A direct-file scan of `decompiled/Terraria/Item.cs` (49 981 lines)
finds **ZERO** `Dust.NewDust*` call sites — `Item.cs` is the item *definition*
class (SetDefaults / prefixes / tooltips), not the world-drop entity.

The item USE / PICKUP / SPAWN dust lives in **`WorldItem.cs`** — the
`Main.item[400]` world-drop entity — exactly as research/15's per-file
caller-coverage table records:

> `WorldItem.cs | 17 | 1 | none | MISSING`

(17 `Dust.NewDust*` sites + 1 `Gore.NewGore` sibling.) This lane ports those
17 dust sites. The `Item.cs` reading of the brief is a mislabel; the
`WorldItem.cs` row is the real target and the only one that matches "item
use/pickup/spawn dust".

## The 17 dust sites (line-ordered)

| # | Line | Family | Enclosing method | Dust |
| - | ---- | ------ | ---------------- | ---- |
| 1 | 461 | NewDust | `UpdateShimmer` (`:455`) | 309, 1/30 gate, shimmered sparkle |
| 2 | 649 | NewDust | `DespawnIfMeetingConditions` (`:639`) | 15 ×10, fallen-star (75) daytime burn |
| 3 | 661 | NewDust | `DespawnIfMeetingConditions` | 15 ×20, 4143 expiry, CornflowerBlue→Indigo lerp |
| 4 | 670 | NewDust | `DespawnIfMeetingConditions` | 61, 3822 DD2 despawn, Next(18,24) count |
| 5 | 892 | NewDust | `MoveInWorld` wet-entry (`:879`) | 308 ×10, shimmer splash, colour Next(6) |
| 6 | 919 | NewDust | `MoveInWorld` wet-entry | 152 ×5, honey splash |
| 7 | 932 | NewDust | `MoveInWorld` wet-entry | dustWater() ×10, water splash |
| 8 | 946 | NewDust | `MoveInWorld` wet-entry | 35 ×5, lava splash |
| 9 | 971 | NewDust | `MoveInWorld` wet-exit (`:959`) | 308 ×10, shimmer splash |
| 10 | 998 | NewDust | `MoveInWorld` wet-exit | 152 ×5, honey splash |
| 11 | 1011 | NewDust | `MoveInWorld` wet-exit | dustWater() ×10, water splash (box drops the −8) |
| 12 | 1025 | NewDust | `MoveInWorld` wet-exit | 35 ×5, lava splash |
| 13 | 1217 | NewDust | `UpdateItem_VisualEffects` (`:1204`) | 6, meteorite (116) 1/250 gate, scale Next(3) |
| 14 | 1299 | NewDust | `UpdateItem_VisualEffects` | 55, pixie dust (501) 1/6 gate, this.color |
| 15 | 1629 | NewDustPerfect | `UpdateItem_VisualEffects` | 228, fallen-star (75) %12 sparkle |
| 16 | 1641 | NewDustPerfect | `UpdateItem_VisualEffects` | 279, boss-bag %12 sparkle |
| 17 | 1671 | NewDust | `ShimmerEffect` (`:1661`) | 309 ×20, transform burst, colour Next(6) |

The one `Gore.NewGore` sibling (`:653`, the fallen-star burn's 16/17 gore ×3)
is the gore lane's (W4-d55/f10) — its `Main.rand.Next(16,18)` draws are a
documented skip here, exactly as the rest of the port treats unported gore.

## Port shape

- `crates/tr-app/src/item_dust.rs` — the 17 sites behind a `DustSink` seam
  (disjoint field borrows: the `UpdateItems` stream, the shared `DustPool`,
  the `World`, the `SpawnCtx`, the `Audio`). Every spawn rides the verbatim
  `DustPool::new_dust` / `new_dust_perfect` (W4-d50 contract, `Dust.cs:75-256`
  / `:54-63`); the sentinel (6000) is written through like the C#.
- `crates/tr-app/src/item.rs` — `ItemDrop` gains the wet-state fields
  (`wet_count`/`wet`/`lava_wet`/`honey_wet`/`age_ticks`/`splash_edge`) + the
  `SplashKind` enum; `update()` keeps the RNG-free flag/cooldown arithmetic
  (`wet_edges` arms the `splash_edge` marker), `tick_age()` is the
  `timeSinceItemSpawned` tick at its C# position (after the visual effects).
- `crates/tr-app/src/world_sim/items.rs` — the drains run in the physics half
  at their C# positions: sparkle (before movement), splash (inside
  MoveInWorld), despawn burns (after MoveInWorld, before visual effects),
  meteorite/pixie/%12 sparkles (in the visual-effects head).
- `crates/tr-app/src/shimmer.rs` — `ShimmerEffect` burst in `GetShimmered`'s
  shared tail (`:1846`).

## RNG contract (S1-A-10)

One slot per C# call-pass: every draw rides the `UpdateItems` stream
(`Main.SwapRandom("UpdateItems")` wraps `WorldItem.UpdateItem`,
`Main.cs:17779-17781`), in `Main.item` slot order. No new streams. The
`ShimmerEffect` burst runs from `GetShimmered` — still inside the same swap.

Pool contract: `origin/task/W4-d50-dust-type-table`
(`tr_core::particles::DustPool::new_dust`, `Dust.cs:75-256` verbatim).

## Witnesses

`item_dust.rs` carries `w4_g15_*` stream-mirror witnesses (gate draws, the
full NewDust draw sequence, the splash counts/types, the despawn gates, the
palette/lerp rows, the sentinel write-through). WRITTEN NOT RUN (code-only
lane — no cargo per the directive).