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

## SHIMMER SEAL — per-lane EXECUTION instructions (each lane on its OWN branch)

The seal round re-verified every row below against the lanes' CURRENT pushed
tips (fresh fetch, byte-accurate `git show <tip>:<file>` line reads — the
line numbers here are the tip's, not the census round's; the g15 row's
census-round numbers were a few lines stale, corrected here). **Each lane executes its own repins
on its own branch** (the brief's instruction; do NOT wait for 141/master).
The canonical repinned forms to copy are e19's
`item.rs:2144-2151` (`the_grab_rectangle_is_the_expanded_hitbox`) and
`:2332-2352` (`s1_p_i52_the_magnet_pull_blends_at_the_csharp_speed_and_acc`)
at the seal tip.

**The shared inherited-block edit** (every lane whose row says "inherited
i47/i52 block" — the two witnesses are byte-identical main heritage):

1. In `the_grab_rectangle_is_the_expanded_hitbox`: the comment "A 14x14 drop
   at x 44 (44+14 = 58) TOUCHES … at 45 (45+14 = 59)" → "A 16x16 drop at x 42
   (42+16 = 58) TOUCHES … at 43 (43+16 = 59)"; the two `grab_rect_intersects`
   drop x literals `44.0` → `42.0`, `45.0` → `43.0`. The two
   `rects_intersect(..., 14.0, 14.0)` rows are EXPLICIT-size helper probes —
   leave them (they test the helper, not SIZE).
2. In `s1_p_i52_the_magnet_pull_blends_at_the_csharp_speed_and_acc`: the
   comment "pos = centre - 7 (the 14x14 box)" → "pos = centre - 8 (the 16x16
   box)"; both `ItemDrop::new` spawns `293.0, 193.0` → `292.0, 192.0`; the
   moved-x pin `heart.pos.x - 290.0` → `- 289.0`; the y pin
   `heart.pos.y, 193.0` → `192.0`. The blend assertions (−3 / −2.4 / −5.4)
   are centre-based — unchanged.

| Lane (own branch) | Tip verified | Edit sites (file:line at that tip) |
| --- | --- | --- |
| **g66** player-trail-dust | `248f0ff4d` | shared block: `item.rs:1799-1800` (grab), `:1987/:1995/:1996/:2001` (magnet); PLUS `world_sim/editing.rs:17856/:17863` `floor_top - 14.0` → `floor_top - crate::item::SIZE` (both the spawn and the `< 0.5` flush pin). `mount.rs` Rat `player_size_override(14,14)` NO repin (the PLAYER hitbox, `DelegateMethods.cs:190-194`) |
| **e1** dust-sites-player-2 | `3245d624c` | shared block: `item.rs:1799-1800`, `:1987/:1995/:1996/:2001`; PLUS `world_sim/editing.rs:17887/:17894` `floor_top - 14.0` → `floor_top - crate::item::SIZE`. The `pc−4` touch spawns (`editing.rs:17809/:17825` region) NO repin (rule 4) |
| **f6** dust-world-3 | `0f9fd642f` | shared block: `item.rs:1799-1800`, `:1987/:1995/:1996/:2001`; PLUS `world_sim/editing.rs:17856/:17863` floor−14 → floor−SIZE. No mount-grab witness exists (the f6-mount census row stands) |
| **f49** mana-regen-2 | `0d1f941af` | shared block: `item.rs:1802-1803`, `:1990/:1998/:1999/:2004`; PLUS the lane-UNIQUE nebula spawns `world_sim/items.rs:2519-2520/:2569-2570/:2586-2587`: `pc.0 + 93.0` → `pc.0 + 92.0`, `pc.1 - 7.0` → `pc.1 - 8.0` (centre−8; the +100 px centre offset keeps the 12/5 pull maths identical), the "pos = centre - 7 (the 14x14 drop box)" comment → centre−8/16x16. The `pc−4` pickup spawns (items.rs:1886ff, the chime witness `:2629`) NO repin |
| **g15** dust-item-sites | `7981aa5bf` | shared block ONLY: `item.rs:2386-2387` (grab), `:2574/:2582/:2583/:2588` (magnet). NO-repin rows re-verified at this tip: `:2003` `pos.x + SIZE <= 14*TILE_PX` (SIZE-relative wall flush), `:2389-2390` explicit 14×14 helper probes |
| **d79** sound-pitch-pool | `03ebefad7` | shared block: `item.rs:1800-1801`, `:1988/:1996/:1997/:2002` |
| **g56** ui-void-vault | `54c63d8d4` | shared block: `item.rs:1829-1830`, `:2017/:2025/:2026/:2031`. (g56's 239 retire receipt already accepts the merge-side repin; executing on its own branch per this seal supersedes the wait) |
| **f40** lantern-night-2 | `c4c6b4f4f` | shared block: `item.rs:1855-1856`, `:2043/:2051/:2052/:2057` |
| **g25** sounds-item-sites | `67d8a6987` | shared block: `item.rs:1917-1918`, `:2105/:2113/:2114/:2119` |
| **g79** tile-paint-draw | `0badeaeb0` | shared block: `item.rs:1915-1916`, `:2103/:2111/:2112/:2117` |
| **d98** net-ids-items-chests | `8c948e0e2` | shared block: `item.rs:1811-1812`, `:1999/:2007/:2008/:2013` |
| **g48** net-apply-arms-1 | `2b61530b4` (moved since the census) | shared block: `item.rs:1811-1812`, `:1999/:2007/:2008/:2013` (re-verified at the new tip) |
| **g52** net-relay-host | `856edb86c` | shared block: `item.rs:1813-1814`, `:2001/:2009/:2010/:2015` |
| **g20** addlight-item-sites | `b43297746` | NO repin (doc-comment only) |
| **f39** weather-slimerain | `8704691a3` | NO repin (no geometry pins) |
| **f47** liquid-shimmer-2 | `1997be092` | ALREADY DONE on its branch (its keep-one 144 phases 1-2 folded the 16-box + every re-pin); no instruction — the merge dedupe takes either side, the splash SHAPE follows research/144's seal ruling (e19 keeps) |

Witness-name anchors for the pin lines (in case a lane's tip moves again):
the x pin is the `"the blend moves the drop"` assertion, the y pin the
`assert_eq!(heart.pos.y, …)` right after it; both live inside
`s1_p_i52_the_magnet_pull_blends_at_the_csharp_speed_and_acc`. Grep
`44.0, 100.0` / `293.0, 193.0` / `floor_top - 14.0` / `pc.0 + 93.0` to find
every site mechanically; after the edit those greps must answer EMPTY on
the lane's branch (except the explicit-size helper probes, which carry
`14.0, 14.0` as trailing args, not spawn coords).

**Merge order note (unchanged)**: the first merge landing the 16-box repins
any straggler inherited block once; later merges take the 16-pinned side.
With every lane executing its own repins per this seal, the conflict faces
shrink to identical-vs-identical.
