# 77 — DOC-SWEEP lane 5: WorldItem / world-drop items census + port

Status: CENSUS + PORT doc (W4-g15 lane 5, `task/W4-g15-dust-item-sites`,
code-only 2026-11-21). Scope: the item-ENTITY mechanics of `WorldItem.cs`
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

Plus this census doc's commit (see the branch tip).
