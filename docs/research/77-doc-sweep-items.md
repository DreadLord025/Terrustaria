# 77 — DOC-SWEEP lane 5: WorldItem / world-drop items census + port

Status: **SEALED** (W4-g15 lane 5, `task/W4-g15-dust-item-sites`,
code-only 2026-11-21; seal round appended below — re-probe citations
only, the census scope never re-opens). Scope: the item-ENTITY mechanics
of `WorldItem.cs`
(the `Main.item[400]` world drop) + `Main.DrawItem` that the dust half
(research/65, a2911366) did not cover. Every mechanic below carries a C#
citation, a verdict (already-landed / ported-this-lane / documented gap /
other-lane skip) and the port site. Witnesses are WRITTEN, NOT RUN
(code-only swarm rule); no cargo/tests were executed this lane.

Sources read (direct file paths only, no junction recursion):
`decompiled/Terraria/WorldItem.cs` (1806 lines, full method sweep),
`Main.cs:38602-38788` (DrawItem), `Item.cs:49757-49760` (ownership arms),
`ItemID.cs:341/:1320/:1386`, `NPCID.cs:10120`, `WorldFile.cs:1415/:2440`,
`NetMessage.cs:652/:665-668`, `MessageBuffer.cs:1459/:1487-1493`,
`Wiring.cs:2610-2636/:3429`, `Player.cs:14551-14553/:15292/:35130-35166`.

## The six lane items (the brief's census list)

### 1. Ground merge — `TryCombiningIntoNearbyItems` (`WorldItem.cs:157-186`)

ALREADY LANDED (S1-P-i47): `item.rs::ItemDrop::combine_nearby`, drained in
`world_sim/items.rs`'s physics loop — the 30 px Manhattan passive merge,
the `moved/newStack` position+velocity lerp, the drained-source `TurnToAir`.
This lane added:

- The **reservation gate** (`:159` `playerIndexTheItemIsReservedFor !=
  Main.myPlayer` early-return): `combine_nearby` now takes the
  `reserved_elsewhere` answer from the caller's `net_reserved` read
  (`0fead49e4`). SP leaves the map empty, so the gate is vacuous there.
- The **`CanStack` citation** (`Item.cs:49865-49872` — type AND prefix
  equality) + the per-target reservation-equality term (`:167`, the MP
  gate) documented on the helper (`836f5fbd2`).

Known deliberate deviation (pre-existing, ANALYSIS S1-P-i47): the port runs
the merge after the movement half, one tick later than the C#'s
pre-`MoveInWorld` call; RNG-free, so no stream moves.

### 2. Bob/spin draw — `Main.DrawItem` (`Main.cs:38602-38788`)

The velocity spin ALREADY LANDED (S1-P-i47): vanilla has NO positional bob
(the port's invented `sin(t*3+phase)*1.5` bob was retired then);
`rotation = velocity.X * 0.2f`, suppressed by `onConveyor`/`shimmered`
(`:38613-38617`). This lane ported the REST of the draw family
(`b45a3dfbd`, precision follow-up `67cf45768`), all in `item.rs`:

- `shimmer_fade_color` (`:38621-38634`): the shimmer-time colour fade with
  the per-channel `(byte)(255f * f)` truncations.
- `aura_echo_offsets` (`:38667-38721`): the boss-bag echo rings (4 copies
  at 0.25 alpha, radius 8 + 3 copies at 0.34, radius 4, the `% 4` triangle
  pulse), the fallen-star 75 family (same shape, `% 5`), the ManaCloak star
  4143 (two 0.34 rings `% 5` + the white-128 main copy `:38720`); the
  `age/240 + Main.GlobalTimeWrappedHourly*0.04` phase, the f32
  k-accumulation, `RotatedBy`'s double-angle f32 cos/sin.
- `mana_crystal_pulse` (`:38722-38726`, kinds 1522-1527/3643): the
  `Main.mouseTextColor` pulse — INTEGER `mouseTextColor / 2` alpha BEFORE
  the shimmer scale (`67cf45768` fixed the float-half deviation),
  `mouseTextColor/1000f + 0.8` scale. `ItemDrop::draw` gained the
  `mouse_text_color: u8` parameter (`render/tiles.rs:1281` passes
  `params.mouse_text_color`).
- The shimmered additive `A = 0` copy (`:38732-38735`).
- G6 `render_key(items, anim_step, anim_time)` now covers `shimmer_time`
  + the aura age/clock ONLY when an aura/shimmer drop is present
  (zero-when-absent — the plain-drop key does not widen). Caller
  `app/frame/draw.rs:961`.

The `:38637` `rand.Next(500)` sparkle is g13's landed dust site
(dca09bed, `main_dust.rs`, UNWIRED — not this lane's). Documented draw
gaps: the glowmask pass, `GetItemLight`'s draw-side light, and the
`frameCounter` ground frame-animation are unported (see Gaps below).

### 3. Lava burn — `CheckLavaDeath` (`WorldItem.cs:681-770`, the `:419` call)

ALREADY LANDED (S1-P-i47/i50): the rare-0 non-immune burn, the
`LAVA_IMMUNE` set, the voodoo-doll sacrifice arm (M4-BOSS-6c). This lane:

- Verified the port's `LAVA_IMMUNE` set **byte-equal** to
  `ItemID.cs:341`'s 18 rows (`0fead49e4`).
- Added the **reservation gate** (`:683`): a drop reserved for another
  peer does not burn — the `net_reserved` read gates the burn AND the
  voodoo arm (`0fead49e4`). Witness:
  `w4_g15_a_foreign_reserved_drop_neither_burns_nor_combines`.

### 4. Grab-range magnet — `Player.GetItemGrabRange` (`Player.cs:35125-35166`)

The range math ALREADY LANDED (i47/i50/i52: the 42 px base rectangle, the
Gold Ring +350 coin row, the heart +250 / star +300 rows, the Journey
far-range arm). Real gaps found and ported:

- **`treasureMagnet` grants** (`Player.cs:14551-14553` the 5010 Treasure
  Magnet, `:15292` the 5126 Hand Of Creation, `ResetEffects :18493`
  clears): the `GrabFlags.treasure_magnet` seam + the +150 range row
  (`:35149-35152`) existed with NO grant writer. `player.rs` now arms the
  flag on both accessories (the 5126 arm merges with the chisel +0.25
  fold — keep-one, a second arm would be unreachable) (`291fdac09`).
- **Client live GrabFlags** (`51cfd534f`): the client half of
  `update_items` used `GrabFlags::default()`, so the optimistic grab
  refused every accessory range extension; both halves now read
  `gold_ring`/`mana_magnet`/`life_magnet`/`treasure_magnet`/
  `far_placement_range` from the local player.
- **`Wiring.IsHopperInRangeOf`** (`Wiring.cs:2610-2636`, the
  `HopperGrabHitboxSize = 192` inflate-96/axis hitbox, `:3429`): the
  drop's inflated box scanned for an active wired Hopper (tile 21/467 +
  any wire via the new `WireState::any_at` = `Tile.anyWire`'s
  four-channel OR); the `chest::banned_in_inventory` kinds refuse
  (`ItemID.cs:345` set). `world_sim/wiring.rs::is_hopper_in_range_of`
  (`a4b560389`). The C# `FindOwner` (`:243`) uses it to route the MP
  reservation; the SP host owns every drop, so the predicate is the seam
  the MP `FindOwner` reads when it lands.
- **Client reservation-hold expiry** (`WorldItem.cs:445-448`
  `timeToKeepReservation` decrement): the `net_reserved` hold ticks down
  6 per 10 Hz cadence tick (the 60 Hz clock's batched stand-in, the
  `net_play_counter` precedent); an expired hold releases the
  reservation, re-opening the grab gate (`7f122e224`).
- **The `enemyGrabDelayTime` gate on the revenge coin-suck**
  (`WorldItem.cs:405` gates BOTH monster grabs): the port's pickup-time
  revenge arm (the standing coin-wallet deviation, i50) now reads the
  clock, so a freshly-dropped lucky-coin revenge coin is not re-sucked
  for 100 ticks (`dd6fa11ac`).

The doc rows claiming the magnet grants were missing were corrected
(`ed05754d1`: ANALYSIS S1-P-i47 deviations, PORT_MAP gap 5, the `item.rs`
GrabFlags doc).

### 5. Shimmer repulsion — SKIP (e19b owns)

`MoveInWorld`'s shimmered-drop repulsion loop (`WorldItem.cs:788-870`) is
e19b's lane. This lane only CONFIRMED the loop's ownership + recorded the
shimmer census notes in ANALYSIS M7-SHIMMER-1 (`a5c4b7751`): the
shimmered dust half landed in `item_dust`, the ramp's missing
`Item.CanShimmer` gate is cosmetic for non-shimmerable kinds (the apply
gates refuse), the `over_shimmer` probe is an approximation. NOT ported
here.

### 6. Instanced loot pre-switch — SKIP (g52b owns), item-side census only

`MakeInstanced` (`WorldItem.cs:321-336`), the message-90 instanced
`ItemDrop` variant (`instanced = true` + `reservedFor = myPlayer`,
`MessageBuffer.cs:1489-1493`) and the 151/159 slot-reuse/section-request
messages stay g52b's pre-switch lane. This lane's item-side census
contribution: the message-21 `enemyGrabDelayTime` conditional byte
(`NetMessage.cs:652` flag bit 3 + `:665-668` clamp 0..255 byte) —
`net.rs::ItemState.enemy_grab_delay`, the writer/reader carry it,
`netglue.rs::item_state_of` clamps the live clock, the client 21 handler
writes it (`dbfbbec48`; doc fix `7981aa5bf`). NOT the instanced switch
itself.

## Extra ports (the full-method sweep's real gaps)

### 5x. `TryGrantingMakeAWishSet` (`WorldItem.cs:590-610`, the `:414-417` call)

A wet, single-stack, `grabDelayTime > 0`, named-player-delayed
(`!= 255`), non-shimmer-wet platinum coin (74) whose closest player is in
the desert (`ZoneDesert`) grants the five Heroicis pieces
(5655/5656/5657/5658/5661) as shimmered items with the alternating
`vx = num * (num % 2 * 2 - 1)` X spread (`SpawnShimmeredItem :623-637`,
draw-free explicit velocity) + the RainbowBoulder cue
(`ParticleOrchestrator.cs:897-900` — the cue IS the whole
HeroicisSetSpawnSound effect; `SoundID.cs:1709`). Port:
`world_sim/items.rs::try_granting_make_a_wish_set` (post-tick scan — the
closure holds the world borrow; draw-free so the scan order moves no
stream; handles multiple coins without stale indices) +
`audio.rs::play_rainbow_boulder` (`509c29ab3`). Ordering: the grant runs
AFTER `apply_due_transforms` — the C#'s `UpdateShimmer` (`:403`) precedes
the grant (`:414`), and the grant's `swap_remove` would invalidate the
`shimmer_due` indices (`652f08753`). The two passes are disjoint on the
grant's input (a coin in WATER never rides the shimmer ramp).

### 6x. `CheckInWorld` (`WorldItem.cs:573-588`, the `:422` call)

A drop beyond the 20-tile world fluff airs out UNLESS it is a
`RecoverableImportantItem` (`ItemID.cs:1386` — the set is exactly the 21
boss bags, the port's `BOSS_BAG`), which recovers to the spawn cell centre
(`+8/+8` `ToWorldCoordinates`) with zero velocity. The SP target is the
world spawn; a validated bed (`world_sim::editing::bed_spawn_valid`)
overrides. The port runs it after the despawn burns (disjoint kinds —
observationally identical) (`aebc5f223`).

### 7x. `TryFillBucket` (`WorldItem.cs:548-571`, the `:399-402` call)

An empty bucket (205) in the rain fills to a water bucket (206) KEEPING
its stack (the `SetDefaults(206)` + stack-restore shape; kind-swap only).
Gates: `raining` && (`worldSurface > 50` || remix) &&
`IsSurfaceForAtmospherics` (the top-left tile; the remix arm reads
`y > rockLayer && y < height - 350`) && the centre tile's wall allows
wind. The `Next(600/dayRate) == 0 && NextFloat() < maxRaining` pair rides
`UpdateItems` IN ORDER; a refused gate draws NOTHING (the C# returns
before the draws). Port: `world_sim/items.rs::try_fill_bucket` — takes
the `UnifiedRandom` stream directly (the in-closure borrow convention)
(`d43b490f3`, signature repair `652f08753`). The in-place kind swap lets
the drop continue the rest of the update as a water bucket, the C#'s
`this.inner` mutation.

### 8x. `MoveInWorld`'s `ItemNoGravity` arm (`WorldItem.cs:774-786`)

The 16-kind `ItemID.Sets.ItemNoGravity` set (`ItemID.cs:1320` — the
souls/essences 520/521/575/549/548/547, the nebula 3453-3459, the
fragments 3580/3581, the ManaCloak star 4143) damps BOTH axes `*= 0.95`
with the `< 0.1` rest snap — no gravity, no fall cap. The port's
`move_in_world` applied the air gravity+cap to EVERY non-shimmered kind,
so the float kinds fell. `item.rs::ITEM_NO_GRAVITY` + the move arm
(`0ebca131a`); the shimmered case still routes to the float branch.

### 9x. The copper-slime unlock (`GetPickedUpByMonsters_Special`)

Three commits, one mechanic:

- **9a — the world flag + save/load** (`1a2067354`):
  `World.unlocked_slime_copper_spawn` (the `Main`-global standing-in, the
  rainbow/red precedent); `WorldFile.cs:1415` (write) / `:2440` (read) —
  the LAST byte of the v261 7-slime block, wld tail offset **2044**
  (yellow@2043 stays unowned). The flags writer/reader own the byte; the
  owned-world round-trip + v261 read + v326 raw-offset witnesses extended.
- **9b — the pickup scan** (`063b772f7`): `WorldItem.cs:1086-1116` (the
  `:405-407` call) — a dropped Copper Shortsword (89) / Chain Knife (3507)
  overlapping a `CanConvertIntoCopperSlimeTownNPC` hostile
  (`NPCID.cs:10120` = {1, 302, 335, 336, 333, 334}) while the unlock is
  locked is consumed and raises the flag. The gate reads the reservation
  + the enemy-delay clock (`:405`). The `NPC.TransformCopperSlime`
  (`NPC.cs:19160-19182`) target — NPC 684 TownSlimeCopper — has no
  `NpcKind` row: the town-slime family is the NPC roster gap, so the port
  raises the flag and leaves the NPC (the `spawn_shimmer_release`
  precedent: the accounting runs, the spawn waits for the roster). The
  `TownSlimeTransform` particle broadcast is the orchestrator lane's.
- **9c — the `enemyGrabDelayTime` clock** (`e8e1080ab`):
  `WorldItem.cs:28` (field) / `:449-452` (decrement);
  `Item.cs:49757-49760` — the NAMED-player ownership arm sets it to 100,
  the all-players 255 arm does NOT. `item.rs::ItemDrop.enemy_grab_delay_time`
  + `apply_enemy_grab_delay` + the per-update decrement; the named-player
  spawn arms (`spawn_player_drop` + the lucky-coin revenge spawn) set it;
  the `:405` monster-grab gates (copper-slime + revenge `dd6fa11ac`) read
  it; the two `wiring.rs` `ItemDrop` literals gained the field.

Borrow repair (`652f08753`): inside `item_step.tick`'s closure the live
aliases (`world = &app.world`, `items = &mut app.items`, `player`, …)
forbid any helper taking `app` wholesale — `copper_slime_pickup` became
the disjoint `&[Npc]` scan (the `DustSink` convention), the flag write
rides the post-tick `copper_unlocked` latch (a second coin this tick sees
the lock the C#'s in-loop write would show), and `try_fill_bucket` takes
the `UpdateItems` stream directly.

## Documented gaps (NOT ported, each with its reason)

| Gap | C# | Reason |
| --- | -- | ------ |
| `FindOwner` / `ReserveFor` | `WorldItem.cs:188-319`, `:243` | MP-only ownership routing; the SP host owns every drop. `is_hopper_in_range_of` is the landed seam the MP `FindOwner` reads. PORT_MAP gap 4. |
| Message 90 instanced variant | `MessageBuffer.cs:1489-1493` | g52b's pre-switch lane (instanced loot). |
| Messages 151/159 | `NetMessage.SendData` cases 151/159 | The slot-reuse / section-request pair rides g52b's switch. |
| `MakeInstanced` | `WorldItem.cs:321-336` | g52b. |
| Glowmask draw pass | `Main.cs:38737+` (DrawItem tail) | The glowmask atlas pass is the render lane's; no port glow layer for items exists. |
| `GetItemLight` draw-side | `Main.cs` DrawItem light block | The light half lives in `entity_lights::item_light` (S1-E-5); the draw-side call shape is unported. |
| Ground frame-animation | `Main.cs:38602-38612` (`frameCounter`) | The port draws the static frame; the animated ground frames are a render-lane gap. |
| `Item.CanShimmer` ramp gate | `Item.cs` CanShimmer | Cosmetic for non-shimmerable kinds (the apply gates refuse); ANALYSIS M7-SHIMMER-1 (`a5c4b7751`). |
| Shimmer repulsion loop | `WorldItem.cs:788-870` | e19b owns. |
| NPC 684 transform | `NPC.cs:19160-19182` | The town-slime roster gap; the flag is the observable half. |
| `TownSlimeTransform` particles | `ParticleOrchestrator` | The orchestrator lane's. |

## Witnesses (written, NOT run)

~15 new/extended witnesses across `world_sim/items.rs` (5 new test mods:
`make_a_wish_tests`, `check_in_world_tests`, `fill_bucket_tests`,
`copper_slime_tests`, + the reservation-gate row in `lava_burn_tests`),
`world_sim/wiring.rs` (`is_hopper_in_range_of`), `app/netglue.rs` (the
hold-expiry + `ItemState` round-trip rows), `player.rs:14823` (the
treasure-magnet grants), `item.rs` (the draw-family key/pulse rows),
`wld/mod.rs` (the copper byte round-trip + raw offsets). The enemy-clock
gate is exercised end-to-end through `update_items` (`652f08753`). All
await the code-only swarm's test phase.

## Commits (this lane, oldest first)

| Hash | Mechanic |
| ---- | -------- |
| `291fdac09` | 1 — treasureMagnet grants |
| `b45a3dfbd` | 2 — DrawItem family + render_key |
| `0fead49e4` | 3 — reservation gates (lava/combine) + LAVA_IMMUNE verify |
| `51cfd534f` | 4 — client live GrabFlags |
| `ed05754d1` | docs — GrabFlags grant-status rows |
| `11de0f3f7` | docs — research/15 row flip |
| `509c29ab3` | 5x — MakeAWish + play_rainbow_boulder |
| `aebc5f223` | 6x — CheckInWorld |
| `d43b490f3` | 7x — TryFillBucket |
| `a5c4b7751` | docs — ANALYSIS shimmer notes |
| `0ebca131a` | 8x — ItemNoGravity arm |
| `1a2067354` | 9a — copper flag save/load |
| `063b772f7` | 9b — copper_slime_pickup |
| `e8e1080ab` | 9c — enemy_grab_delay clock |
| `dbfbbec48` | 6a — ItemState.enemy_grab_delay wire byte |
| `a4b560389` | 4a — is_hopper_in_range_of + any_at + banned_in_inventory |
| `836f5fbd2` | docs — CanStack citation |
| `7f122e224` | 4b — client hold expiry |
| `dd6fa11ac` | 4c — revenge enemy-delay gate |
| `7981aa5bf` | docs — item_state_of doc fix |
| `652f08753` | borrow repair + wish/shimmer ordering |
| `67cf45768` | mana_crystal_pulse integer-division precision |
| `527904546` | anchors — 15 canary rows |

Plus this census doc's commit (`af7d97534` — the 24-commit lane:
`291fdac09^..af7d97534`, `git rev-list --count` = 24).

---

## SEAL (2026-11-21, code-only — re-probe citations only, scope frozen)

The lane is COMPLETE for its scope and SEALED. All probes below ran
fresh at the seal round (direct file paths / `git show <tip>:<file>`
only, no junction recursion; no cargo/rustc/tests/graphify invoked).

### S1 — the 24-commit lane landed + pushed

- Tip `af7d97534` (census doc) == `origin/task/W4-g15-dust-item-sites`
  (`git rev-parse` both = `af7d97534e0e6015a73c9ba1a4e78e94e652c000`);
  working tree clean at seal entry. The 24 lane commits
  (`git rev-list --count 291fdac09^..HEAD` = 24) sit on base
  `a725acb04` (the 26-commit `origin/main..HEAD` span includes the two
  dust-half commits `a29113669`/`f8f35d799` beneath the lane's first).
- The borrow repair (`652f08753`), the mana-pulse integer division
  (`67cf45768` — `item.rs::mana_crystal_pulse` reads
  `(mouse_text_color / 2) as f32` BEFORE the fade, `Main.cs:38724`),
  the 15 canary rows (`527904546`) and this census (`af7d97534`) are
  all in remote history (`d43b490f3` ancestor-of-remote exit 0).
- **Canary re-probe**: all 16 `W4-g15` rows in `anchors.txt`
  (15 lane-5 + 1 dust-half) verified marker-present by substring at
  the seal — 16/16 OK, 0 missing.

### S2 — g45 buckets compose (the g45f-A9 push-owed debt DISCHARGED)

- **Your TryFillBucket `d43b490f3` is PUSHED** — the exact probe g45's
  A9.4 owed: `git merge-base --is-ancestor d43b490f3
  origin/task/W4-g15-dust-item-sites` exit **0**. g45 monitor pass 2
  (`695e1ec5e`, research/69 A10) already recorded the discharge at the
  then-tip `7981aa5bf`; the tip has since advanced to `af7d97534`
  (same lineage), so the A9 fold-list needs NO re-pin — the merger
  takes the remote tip directly.
- **Rain-fill gate verified** (`world_sim/items.rs:768`):
  `!was_grabbed && drop.kind == 205 && !reserved_elsewhere` guards
  `try_fill_bucket` — the C# `:397` `!beingGrabbed` head + `:399`
  `type == 205` + `:550` `playerIndexTheItemIsReservedFor !=
  Main.myPlayer` refusal, arm-for-arm. Inside the helper the `:550`
  raining/surface row, the `IsSurfaceForAtmospherics` top-left probe
  (the remix `rockLayer`/`height-350` branch), the centre-tile
  `AllowsWind` wall and the `Next(600/dayRate)==0 &&
  NextFloat()<maxRaining` pair all re-read faithful (`:548-571`
  re-read this round, window ≤100 lines).
- **Disjoint-union editing**: g15's ONLY `world_sim/editing.rs` hunk
  is the lucky-coin `coin.apply_enemy_grab_delay()` in
  `resolve_melee_strike` (`@@ -7964` region, comment + one call);
  g45's hunks are `@@ -19` (the buckets mod), `@@ -1931`
  (`wake_around(&mut …)`), `@@ -4317` (the `use_buckets` arm) —
  **zero overlap**, exactly as g45's 104 §6c-compose axis 4 pinned.
  The union re-fold `9bc722e5f`'s editing hunks (`:1931/:5729/:5810/
  :6947/:7541`) also miss g15's hunk. The compose loop holds: g45's
  full-inventory hand-back spawns a world 205 via
  `spawn_item_drop_px` (buckets.rs `:394`), g15's gate admits it,
  rain can fill it back; the spawn signatures are byte-identical at
  both tips (re-diffed this round — `spawn_item_drop_px` body equal),
  and buckets.rs carries ZERO `ItemDrop` literals (grep exit 1), so
  the +7 ctor-initialized fields need no buckets re-pin.
- **Compose pins** (`tr-core/wiring.rs` `:3922` +
  `world_sim/wiring.rs` `:670`, g45 research/69 A9.3): re-probed on
  the union `9bc722e5f` — `the_pump_moves_water_and_adopts_the_type`
  at core `:3922` and
  `r_12_5_the_pump_transfer_wakes_the_settle_and_the_water_flows` at
  app `:670` both present. g15's wiring adds are DISJOINT from both:
  core `WireState::any_at` inserts at `:114` (the union's core hunks
  start `:250`), the app-side `is_hopper_in_range_of` + its witness
  insert at `:26-66`/`:238`/`:652`/`:863` (the union's app hunks are
  `:241`/`:647` — adjacent, not overlapping; the merger keeps both
  families, the pin line-numbers shift by the g15 insertions, the
  markers survive).

### S3 — gaps → task seeds (the census gap rows now have owners)

- **`FindOwner`/`ReserveFor` → the 39 item-lane seed is PUBLISHED**:
  g48's research/245 (`245-39-item-ownership-seed.md`, tip
  `2b61530b4`) adjudicates the 39 ReleaseItemOwnership row to the
  ITEM lane — the d98 region (`task/W4-d98-net-ids-items-chests`,
  tip `8c948e0e2`, the 21/22/30-34/80/81 census scope) owns it, NOT
  g48/g49/g50/g51 (grep-empty at their tips) and NOT the landed
  M6-NET-5 row (27/28/29, a ROADMAP forward-reference). The
  `WorldItem.cs:188-285` FindOwner/ReserveFor scope pin + the
  unrowed reservation-fields census ride that seed. g15's
  `is_hopper_in_range_of` (`a4b560389`) is the landed input seam the
  seed's `:243` call reads. The gap row in the table above stands as
  g15's census record; the WORK is d98-region's per 245.
- **Messages 90/151/159** stay g52b's pre-switch lane — re-probed:
  the pushed g52 tip `856edb86c` (net-relay-host) carries NO
  `MakeInstanced` port (only the `npcs.rs:2729` constants-off note)
  and no 151/159 codec arms in net.rs/netglue.rs (grep empty). The
  g15 `enemyGrabDelayTime` byte (`dbfbbec48`) is the item-side
  census contribution, NOT the switch. d98's `662c884cc` landed the
  SAME 21 bit-3 byte independently (net.rs `:818` field, writer/
  reader flag-3 arms byte-equivalent to g15's `:800` forms — both
  re-read this round); the merger dedupes keep-one (either side
  faithful, the `ItemState` shapes match field-for-field). d98 does
  NOT carry the named-player 100-arm (`apply_enemy_grab_delay` grep
  empty on its tip) — that half is g15-unique (`e8e1080ab`).
- **Glowmask / GetItemLight draw-side / frame-anim** stay render-lane
  gaps — re-probed empty on both g15 HEAD and the e19 tip (no item
  glowmask pass, no `frame_counter` in item.rs). The light HALF
  lives in `entity_lights::item_light` (`:153`, S1-E-5) +
  `item_lights::rand_light`; the draw-side call shape is the gap.
- **NPC 684 roster → d43 LANDED the row; the transform itself does
  NOT activate** (the brief's question answered): d43's batch 11
  `fd6a544fc` (tip `0253bde90`) lands `NpcKind::TownSlimeCopper`
  (npc.rs `:1124`, the SetDefaults row, the netglue spawn arm
  `:3600` `spawn_cave_walker`, bestiary/sheet/lang rows). BUT a
  full-tip grep for `TransformCopperSlime`/`transform_copper`/
  `unlocked_slime_copper` on d43 answers EMPTY — no consumer reads
  g15's flag, and `NPC.TransformCopperSlime` (`NPC.cs:19160-19182`:
  the netMode-1 SendData(140) arm, the flag raise + SendData(7),
  `nPC.Transform(684)`, the TownSlimeTransform broadcast) is
  unported. The flag's OTHER C# consumers are the town-arrival
  gates (`Main.cs:65940` `townNPCCanSpawn[684]` + `:66070` the
  spawn-pool pick) — g15's `town_arrival.rs` has no slime gates
  either. So the roster gap NARROWS to a wiring gap: the flag +
  save byte (g15) and the NpcKind row (d43) both exist post-merge;
  the missing halves are (a) the pickup's `Transform(684)` call
  (g15's scan deliberately leaves the NPC — the
  `spawn_shimmer_release` precedent), (b) the town-arrival
  flag→canSpawn gates, (c) msg 140 + the particle broadcast
  (orchestrator lane). The `transform_to_kind` method exists
  (`npc.rs:5080`) as the seam (a) reads. Recorded as the post-merge
  seed, NOT ported here (the seal freezes scope).

### S4 — e19b shimmer-repulsion / g52b instanced-loot skip receipts

- **e19b repulsion CONFIRMED LANDED on its branch**: the e19 tip
  `c7a38bea5` carries `ItemDrop::repel_shimmered`
  (`item.rs:1187`, the `WorldItem.cs:790-847` slice-level 50-budget
  centre-span push, the ±3 bounds + equal-X tie-break, the `flag`
  → `repulsed` field → `velocity.X *= 0.8` tail `:859-862`), wired
  at `world_sim/items.rs:689` BEFORE `drop.update` (the C#'s
  MoveInWorld-head position). g15's skip stands — no repulsion code
  on this branch (`repulsed` grep empty). **Merge notes**: e19's
  round-2/3 also ports the ItemNoGravity float (`item.rs:1325`,
  the SAME 16-kind set as g15's `:683` — byte-equal lists) and the
  wet-gravity heads (`wet_step` 0.5/0.375/0.25, the `:379-396`
  window) which g15 does NOT carry; the merger keeps e19's
  `move_in_world(world, gravity, max_fall, wet_step)` signature and
  folds g15's disjoint adds (the draw family, the enemy clock,
  `tick_age`, the reservation gates) onto it. The splash keep-one
  is RULED (e19 research/144, SHIMMER SEAL consumed): e19's
  `ItemSplash`/`pending_splash`/`take_splash` + free-fn
  `item_splash` (`world_sim/items.rs:777`) survive; f47's
  `Splash`/`splash_due`/`ItemDrop::splash` retire. **g15's
  `item_dust::splash_edge` + the `splash_edge: Option<(bool,
  SplashKind)>` marker are a THIRD form of the same window** — at
  the three-way merge the e19 keeper wins per 144's ruling; g15's
  drain site (`world_sim/items.rs:827-837`) and marker field
  retire with the f47 dup-half, the OTHER 16 dust sites
  (sparkle/burns/visuals/ShimmerEffect) are g15-unique and stay.
  The g15 splash form's known deltas vs the keeper: the sticky
  entry-lava read (behaviourally void per 144), the box-union
  honey/shimmer probes (the standing deviation), the un-truncated
  sound-position floats (e19 truncates `as i32 as f32` per the C#
  `(int)` casts — the keeper's form wins).
- **The 16-box repin EXECUTED on this branch** (`8f04df01b`, this
  seal round): the research/145 SHIMMER SEAL g15 row — shared
  inherited i47/i52 block ONLY: grab-rect edge 44/45 → 42/43,
  magnet spawns 293/193 → 292/192, moved-x 290 → 289, y-pin 193 →
  192; the explicit 14×14 helper probes stay (rule 1 tail); the
  `44.0, 100.0`/`293.0, 193.0` greps answer EMPTY after the edit.
  The lane-unique geometry (the `resting_drop`/`resting` fixtures'
  `19*16+2` spawns, the `item_dust` splash box) is SIZE-relative
  or overlap-tolerant — the seal's g15 row lists NO lane-unique
  repin, confirmed by re-read (the +2 spawns still overlap the
  shimmer row at SIZE 16; the wall-flush pins follow the constant).
- **g52b instanced loot CONFIRMED SKIPPED**: `MakeInstanced`
  (`WorldItem.cs:321-336`), the 90 instanced variant
  (`MessageBuffer.cs:1489-1493`) and 151/159 are absent from g52's
  pushed tip `856edb86c` (grep: only the npcs.rs constants-off
  note) — the pre-switch lane owns them; g15's census rows stand.

### S5 — the item-dust-family bible

The cross-lane family fold-table (who owns which half of the
WorldItem/Item dust+entity surface, the keep-one rulings, the
merge order) lives in **research/347** (`347-item-dust-family-bible.md`,
this seal round — number 347 per registry-150 §8.7 FROZEN assignment:
the original 309 self-pick (`131961a73`, the free-block scan found the
highest claimed key 308, 309+ unclaimed) missed §8's reservations and
was contested — chain 309→347, d125-309 stands; 78 was NOT used — two
prior forks already carry it, g78-sight + c40-weather). research/65 (the
dust-half census correction) and this doc stay the per-lane
records; 347 is the merger's single page.

### Seal-round commits

| Hash | What |
| ---- | ---- |
| `8f04df01b` | the 145 SHIMMER-SEAL 16-box repin (shared block only) |
| (this doc's commit) | research/77 SEALED + research/347 family bible (309 self-pick → 347 per registry-150 §8.7) + INDEX/PORT_MAP/ANALYSIS receipts |
