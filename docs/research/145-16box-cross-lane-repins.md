# 145 — The 16-box cross-lane repin list (W4-e19 final, SHIMMER item 2)

**For 141/master.** `e2003946b` (this branch) re-pinned `item::SIZE` 14 → 16
— the C# `WorldItem` ctor is `base.width = (base.height = 16)`
(`WorldItem.cs:138`). The 14 was a stand-in; every probe (grab rect,
WetCollision/LavaCollision, conveyor, TileCollision, repulsion span, splash
`width+12`) now runs the 16-box. The DRAW quad keeps the 14 px visual
centred (`pos+1`, `SIZE-2`) so pixels are unchanged. **Any co-lane witness
that pins the OLD 14 geometry must repin at merge** — the list below is the
full census of the pushed branches (every `origin/task/W4-*` diffed against
main for `item.rs`/`world_sim/items.rs`/the dust lanes).

## The repin rules (hand-derived, C#-cited)

1. **Grab-rect edge** (`the_grab_rectangle_is_the_expanded_hitbox`,
   `item.rs`): player (100,100) 20×42, range 42 → rect x 58..162. A drop at
   x 44 (44+16=60 > 58) NOW INTERSECTS; the touching edge moves to **42**
   (42+16=58, strict `<` refuses) / **43** overlaps. Main's `44/45` pair
   repins to `42/43` (this branch's `item.rs:2137-2138` already carries it).
   The two `rects_intersect(..., 14.0, 14.0)` rows are direct-hitbox probes
   with EXPLICIT sizes — they stay valid as-is (they test the helper, not
   `SIZE`), but the comment should say 16 where it describes the drop box.
2. **Magnet centre spawns**: `pos = centre − SIZE/2` → centre (300,200)
   spawns **292/192** (was 293/193). The blend maths is centre-based and
   unchanged (−3 / −2.4 / −5.4); only the spawn literals and the
   `pos.y == 193.0` pin move (→ 192.0), and the moved-x pin 290.0 → 289.0.
3. **Resting fixtures**: flush on a floor at `F` means `pos.y = F − 16`
   (was `F − 14`). `floor_top - 14.0` literals repin to
   `floor_top - crate::item::SIZE` (this branch's `editing.rs:17856/:17863`
   pattern). A drop SPAWNED at `F−14` now lands one tick later at `F−16`;
   the `< 0.5` tolerance pins FAIL (2 px off) — repin the spawn, not the
   tolerance.
4. **`pc − 4.0` pickup spawns** (the touch-box fixtures): the 16-box still
   overlaps the player's 20×42 body at `pc−4` — these stay green, no repin.
5. **Splash geometry** (this branch only, but f47 co-lane): the dust anchor
   `pos.Y + height/2 − 8` is `pos.Y + 0` at 16 (was −1); the box roll
   `next_max(width+12−4)` is 24 (was 22). See research/144.

## Per-lane repin list (pushed tips as of this census)

| Lane | File:line (their tip) | Old pin | Repin to |
| --- | --- | --- | --- |
| **g66** player-trail-dust | `item.rs:1799-1800` | grab edge 44/45 | 42/43 (rule 1) |
| g66 | `item.rs:1987/:1996/:2001` | magnet 293/193, y==193 | 292/192, y==192 (rule 2) |
| g66 | `world_sim/editing.rs:17856/:17863` | `floor_top - 14.0` | `floor_top - crate::item::SIZE` (rule 3) |
| g66 | `mount.rs:1718` `player_size_override (14,14)` | — | **NO repin** — the Rat PLAYER hitbox is genuinely 14×14 (`DelegateMethods.cs:190-194`), unrelated to the item box |
| **e1** dust-sites-player-2 | `item.rs:1799-1800` | grab edge 44/45 | 42/43 |
| e1 | `item.rs:1987/:1996/:2001` | magnet 293/193 | 292/192 |
| e1 | `world_sim/editing.rs:17887/:17894` | `floor_top - 14.0` | `floor_top - crate::item::SIZE` |
| e1 | `world_sim/editing.rs:17809/:17825` `pc−4` spawns | — | NO repin (rule 4) |
| **f6** dust-world-3 | `item.rs:1799-1800` | grab edge 44/45 | 42/43 |
| f6 | `item.rs` magnet witness (same i52 block) | 293/193 | 292/192 |
| f6 | `world_sim/editing.rs:17856/:17863` | `floor_top - 14.0` | `floor_top - crate::item::SIZE` |
| **f6-mount-grab** (the mount grab seam) | no pushed branch carries a mount-specific grab witness (`mount.rs` has no `grab_rect`/`item::SIZE` use on any tip; the grab rect reads the LIVE `p.height`/`WIDTH` at `world_sim/items.rs:61/:583`) | — | the mounted grab rect follows the player's overridden hitbox automatically; the DROP side is the 16-box — any future mount-grab witness must spawn drops at `centre−8` and use the 42/43 edge |
| **f47** liquid-shimmer-2 | `item.rs:548` `SIZE = 14.0` | the constant itself | **retire** — keep-one verdict research/144: e19's 16 wins; f47's splash witnesses repin per 144 or retire with the dup-half |
| **f49** mana-regen-2 | `item.rs:1802-1803` | grab edge 44/45 | 42/43 |
| f49 | `item.rs:1990/:1999/:2004` | magnet 293/193 | 292/192 |
| f49 | `world_sim/items.rs:2519-2520/:2569-2570/:2586-2587` | `pc.0+93, pc.1−7` ("centre−7, the 14-box") | `pc.0+92, pc.1−8` (centre−8; the +100 px centre offset keeps the pull maths identical) |
| **g15** dust-item-sites | `item.rs:1592` `pos.x + SIZE <= 14*TILE_PX` | — | NO repin (SIZE-relative, the wall flush follows) |
| g15 | `item.rs:1938-1939` explicit 14×14 helper probes | — | NO repin (rule 1 tail) |
| g15 | `item.rs` i52 magnet block (293/193) | 293/193 | 292/192 |
| **d79** sound-pitch-pool | `item.rs:1800-1801/:1988/:1997` | 44/45 + 293/193 | 42/43 + 292/192 |
| **g25** sounds-item-sites | `item.rs:1917-1918/:2105/:2114` | 44/45 + 293/193 | 42/43 + 292/192 |
| **g79** tile-paint-draw | `item.rs:1915-1916/:2103/:2112` | 44/45 + 293/193 | 42/43 + 292/192 |
| **g56** ui-void-vault | `item.rs:1829-1830/:2017/:2026` | 44/45 + 293/193 | 42/43 + 292/192 |
| **f40** lantern-night-2 | `item.rs:1855-1856/:2043/:2052` | 44/45 + 293/193 | 42/43 + 292/192 |
| **d98** net-ids-items-chests | `item.rs:1811-1812/:1999/:2008` | 44/45 + 293/193 | 42/43 + 292/192 |
| **g48** net-apply-arms-1 | `item.rs:1811-1812/:1999/:2008` | 44/45 + 293/193 | 42/43 + 292/192 |
| **g52** net-relay-host | `item.rs:1813-1814/:2001/:2010` | 44/45 + 293/193 | 42/43 + 292/192 |
| **g20** addlight-item-sites | `item.rs` doc-comment only | — | NO repin |
| **f39** weather-slimerain | `world_sim/items.rs` (no geometry pins) | — | NO repin |

**Merge order note**: most of these lanes carry the SAME main-inherited
witness block (the i47/i52 fixtures) — the first merge that lands the 16-box
repins them once; later merges resolve the conflict by taking the
16-pinned side (this branch's `item.rs:2129-2141` + `:2325-2339` are the
canonical repinned forms). The lane-UNIQUE witnesses (f49's nebula pull
spawns, the editing.rs resting fixtures) need the per-lane edits above.

**yours-repinned confirmation**: this branch's own grab-rect witness reads
the 42/43 edge (`item.rs:2137-2138`), the magnet spawns 292/192
(`:2325/:2339`), the resting fixtures `19*16` flush, the wet-probe parking
−8, the repulsion span 16/8=2, the flush landing 303→304, the light anchors
168/184 + the −12 truncation — all re-pinned in `e2003946b`.
