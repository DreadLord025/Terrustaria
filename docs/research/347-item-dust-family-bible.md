# 347 — The item-dust-family bible (W4-g15 SEAL round)

> **RENAME CHAIN: 309 (self-pick at `131961a73` 16:48:33, contested) →
> 347 (registry-150 §8.7 FROZEN ASSIGNMENT).** The d54 collision
> registry (`150-research-number-registry.md`, round-4 §8.7 AMENDMENT 3,
> read at `origin/task/W4-d54` `a15cb634b`) renumbers every post-§8
> self-pick to 341+: g15's 309 pick → **347**; the
> key 309 collision stands with d125 (`281-empress-sheets-ledger.md` →
> 309 per §8.1 — d125-309 stands). All refs re-pointed this commit;
> NO SELF-PICKS — §8/§8.7 assignments only.

Number **347** per registry-150 §8.7 (the original **309** pick used
the research/150 free-block rule — a fresh full-remote `ls-tree` scan
at the seal round found highest claimed key 308, 309+ unclaimed; that
tips-only sweep missed §8's reservations, so the pick was contested;
78 already forked twice — g78-sight/c40-weather).
Published with the research/358 SEAL (`task/W4-g15-dust-item-sites`;
the census doc renumbered 77→358 per registry-150 §9.2 — the §7.3 284
target was voided, 284=g52).
**This is the merger's single page for the WorldItem/Item
dust+entity surface** — who owns which half, the keep-one rulings,
the merge order. The per-lane records stay canonical for their own
scope: research/190 (the dust-half census correction), research/358
(the lane-5 entity census, SEALED), e19's research/144/145/148 (the
splash keep-one + the 16-box seal), g45's research/69/104 (the
liquid/bucket compose), g48's research/245 (the 39 ownership seed).
All probes fresh this round, direct paths only; code-only — no
cargo/tests/graphify invoked.

## 1. The family map (C# surface → port owner)

| C# surface | Port half | Owner (keep at merge) | Status |
| --- | --- | --- | --- |
| `WorldItem.cs` 17 `Dust.NewDust*` sites (sparkle `:461`, burns `:649/:661/:670`, visuals `:1217/:1299/:1629/:1641`, ShimmerEffect `:1671`) | `item_dust.rs` behind the `DustSink` seam, drains in `world_sim/items.rs` | **g15** (`a29113669`) | landed+pushed, 16/16 canary rows OK |
| `MoveInWorld` wet entry/exit splashes (`:879-1035`) + wetCount cooldown + dry clear | THREE forms exist: e19 `ItemSplash`/`pending_splash`/`item_splash`, f47 `Splash`/`splash_due`/`ItemDrop::splash`, g15 `splash_edge: Option<(bool, SplashKind)>` + `item_dust::splash_edge` | **e19** per research/144 SHIMMER SEAL (the g67 free-fn drain contract) | e19/f47 ruled; the g15 third form retires with f47 (§3) |
| `MoveInWorld` repulsion scan (`:790-847`) + the ×0.8 tail (`:859-862`) | `ItemDrop::repel_shimmered` + `repulsed` field, wired pre-update | **e19** (`6d4982240`, tip `c7a38bea5` `items.rs:689`) | g15 skip CONFIRMED (358 §5) |
| `MoveInWorld` ItemNoGravity float (`:774-786`, `ItemID.cs:1320` 16 kinds) | BOTH g15 (`0ebca131a`, `item.rs:683/:1307`) and e19 (`item.rs:708/:1325`) carry it — the sets byte-equal | **keep-one**, either side faithful; e19's form rides its `move_in_world(world, gravity, max_fall, wet_step)` signature | dedupe at merge (§3) |
| Wet gravity heads (`:379-396`: shimmer 0.065/4/×0.375, honey 0.05/3/×0.25, water 0.08/5, air ×0.5) + wetVelocity integration (`:1046-1084`) | `wet_step` threading in e19's `move_in_world` | **e19** (g15 has air-only constants) | e19-unique, keep |
| The 16-px drop box (`WorldItem.cs:138`) | `item::SIZE` 14→16 + every geometry repin | **e19** (`e2003946b`); f47 folded it too; **g15 executed its shared-block repin this seal** (`8f04df01b`) | 145 SHIMMER SEAL consumed on all three branches |
| `TryCombiningIntoNearbyItems` (`:157-186`) | `combine_nearby` (i47) + the reservation gate + CanStack cites | **g15** (`0fead49e4`/`836f5fbd2`) | landed (the post-move position is the standing i47 deviation) |
| `Main.DrawItem` family (`:38602-38788`) | `shimmer_fade_color`/`aura_echo_offsets`/`mana_crystal_pulse` (int /2)/A=0 copy + render_key coverage | **g15** (`b45a3dfbd`/`67cf45768`) | landed; glowmask/GetItemLight-draw/frame-anim stay render-lane gaps |
| `CheckLavaDeath` (`:681-770`) | i47/i50 burn + LAVA_IMMUNE byte-verify + reservation gate | **g15** gate (`0fead49e4`) on the i47/i50 base | landed |
| `GetItemGrabRange` grants + client flags (`Player.cs:14551/:15292/:35130-35166`) | treasureMagnet grants + both-half live GrabFlags | **g15** (`291fdac09`/`51cfd534f`) | landed |
| `TryGrantingMakeAWishSet` (`:590-610`) + SpawnShimmeredItem + RainbowBoulder cue | post-tick scan AFTER apply_due_transforms + `play_rainbow_boulder` | **g15** (`509c29ab3`, ordering `652f08753`) | landed (the post-tick deviation recorded) |
| `CheckInWorld` (`:573-588`) | fluff air-out + boss-bag recover (bed override) | **g15** (`aebc5f223`) | landed |
| `TryFillBucket` (`:548-571`) | `try_fill_bucket` on the UpdateItems stream, gate `!was_grabbed && 205 && !reserved_elsewhere` | **g15** (`d43b490f3`, PUSHED — g45 A9.4 discharged) | landed; g45 compose contract 104 §6c verified (358 S2) |
| `GetPickedUpByMonsters_Special` (`:1086-1116`) + the flag save byte (`WorldFile.cs:1415/:2440`) | flag + `copper_slime_pickup` scan + wld tail 2044 | **g15** (`1a2067354`/`063b772f7`) | landed; the Transform(684) half is the post-merge seed (§4) |
| `enemyGrabDelayTime` clock (`:28/:449-452`, `Item.cs:49757-49760`) + msg-21 bit-3 byte | `enemy_grab_delay_time` + `apply_enemy_grab_delay` + `ItemState.enemy_grab_delay` | clock+100-arm **g15-unique** (`e8e1080ab`); the wire byte **g15==d98** (`dbfbbec48` vs `662c884cc`, field-for-field equal) — keep-one either side | landed both tips |
| `Wiring.IsHopperInRangeOf` (`:2610-2636`) | `is_hopper_in_range_of` + `WireState::any_at` + `banned_in_inventory` | **g15** (`a4b560389`) | landed as the MP FindOwner input seam |
| `timeToKeepReservation` decay (`:445-448`) | net_reserved hold −6 per 10 Hz cadence tick | **g15** (`7f122e224`) | landed (batched stand-in recorded) |
| `FindOwner`/`ReserveFor` (`:188-319`) + msg 39 | UNPORTED — the seed | **d98 region** per g48 research/245 | task-seed published, NOT g15/g48 work |
| `MakeInstanced` (`:321-336`) + msg 90 instanced + 151/159 | UNPORTED — pre-switch | **g52b** | skip re-confirmed at g52 tip `856edb86c` (grep empty) |
| `Main.cs:38637` rand500 draw sparkle | `main_dust.rs` (landed, UNWIRED) | **g13** (`dca09bed` lineage, tip `eef0202cc`) | g13's wiring contract, not this family's |

## 2. The keep-one rulings (already rendered, cited)

1. **Splash**: e19 keeps (research/144 SHIMMER SEAL, the f47
   counter-verdict consumed arm-by-arm). The g15 `splash_edge`
   form is the third copy of the same window and retires with
   f47's at the three-way merge — its known deltas vs the keeper:
   the sticky entry-lava read (behaviourally void per 144), the
   box-union honey/shimmer probes (the standing g15 deviation,
   ANALYSIS), the un-truncated sound-position floats (the C#
   `(int)` casts — e19 truncates). The g15 drain site
   (`world_sim/items.rs:827-837`) + the marker field
   (`item.rs:927`) + `item_dust::splash_edge` are the retire
   hunks; the OTHER 16 dust sites are g15-unique and STAY.
2. **16-box**: e19 canonical (`e2003946b`); every lane executes
   its own shared-block repin per 145's SHIMMER SEAL — g15 DONE
   this seal (`8f04df01b`: grab edge 42/43, magnet 292/192, x-pin
   289, y-pin 192; the explicit 14×14 helper probes stay). The
   first merge landing SIZE 16 repins any straggler once; later
   merges take the 16-pinned side.
3. **ItemNoGravity**: g15 == e19 byte-equal sets + same maths;
   keep-one trivially, e19's form wins by riding the wet-gravity
   signature the merger keeps anyway.
4. **Msg-21 enemy byte**: g15 == d98 field-for-field
   (`ItemState.enemy_grab_delay`, flag bit 3, clamp 0..255,
   reader/writer arms re-read equal this round); keep-one either
   side. The named-player 100-arm + the `:405` gates are
   g15-unique — they ride whichever ItemState survives.
5. **World.crimson / shimmer decraft**: g69-canonical per e19
   round 3 — outside this family's scope, cited for the merge
   order only.

## 3. The merge order (this family)

1. Land the **16-box + e19 movement half** first (SIZE 16,
   `move_in_world(world, gravity, max_fall, wet_step)`,
   `repel_shimmered` + `repulsed`, `ItemSplash`/`take_splash` +
   `item_splash`). It is the widest geometry/signature change;
   every later fold resolves onto it.
2. Fold **g15's entity adds** (the draw family, reservation
   gates, MakeAWish/CheckInWorld/TryFillBucket, the copper flag +
   scan, the enemy clock + 100-arm, hopper predicate, hold
   expiry, `tick_age`/`age_ticks`): all disjoint from e19's
   movement half EXCEPT (a) the ItemNoGravity dup — drop g15's
   arm, keep e19's; (b) the splash third form — drop g15's
   marker/drain/`splash_edge` fn per §2.1; (c) `update`'s
   shimmered routing — g15's `pos += vel` float branch and
   e19's shimmered-float scan (`:493-504`) are the same C#
   window, reconcile onto e19's two-cell probe (the g15
   `over_shimmer` approximation retires there).
3. Fold **f47's keep-hunks** (the 676 ShimmerSlime light/dust
   arm, the NPC-side transforms, the collision wrappers) per
   144's published table; its splash dup-half + witnesses retire.
4. **g45 buckets** ride the liquid union fold (g46e per 69 A9);
   the g15 compose needs NO re-pin (358 S2 — disjoint editing
   hunks, byte-identical spawn signatures, zero buckets ItemDrop
   literals, the two compose pins present on the union).
5. **d98's wire region** dedupes the msg-21 byte keep-one; its
   35/36/33/34/80/81 codecs are disjoint from g15's netglue adds
   (the hold-expiry block + the ItemState round-trip rows).
6. **d43's NPC roster** lands the TownSlimeCopper row; the
   flag→transform wiring is the post-merge seed (§4), NOT a
   merge blocker — the flag is the observable half.

## 4. The post-merge seeds (NOT ported, owners named)

| Seed | C# | Owner | Evidence |
| --- | -- | ----- | -------- |
| 39 ReleaseItemOwnership + FindOwner/ReserveFor | `WorldItem.cs:188-285`, `NetMessage` 39 | d98 region (item lane) | g48 research/245 (tip `2b61530b4`), the g15 hopper seam is the `:243` input |
| The copper Transform(684) wiring: pickup → `transform_to_kind` + the town-arrival gates + msg 140 | `NPC.cs:19160-19182`, `Main.cs:65940/:66070` | post-merge item/NPC seam (the roster row is d43's `fd6a544fc`) | d43 full-tip grep EMPTY for TransformCopperSlime/unlocked_slime_copper; g15 `town_arrival.rs` has no slime gates; `npc.rs:5080` `transform_to_kind` is the seam |
| TownSlimeTransform particles | `ParticleOrchestrator` | orchestrator lane | 358 gap table |
| Instanced loot pre-switch (90/151/159, MakeInstanced) | `WorldItem.cs:321-336`, `MessageBuffer.cs:1489-1493` | g52b | g52 tip `856edb86c` grep empty |
| Item glowmask / GetItemLight draw-side / ground frame-anim | `Main.cs:38737+`/DrawItem light/`:38602-38612` | render lane | both g15/e19 tips grep empty; the light half lives in `entity_lights::item_light` |
| The `Item.CanShimmer` ramp gate | `Item.cs:49448-49464` | cosmetic-only deviation (the apply gates refuse) | ANALYSIS M7-SHIMMER-1 (`a5c4b7751`) |

## 5. Verification ledger (this seal round)

- 16/16 g15 anchors rows marker-present (substring probe).
- `d43b490f3` ancestor-of-remote exit 0; local == remote tip
  `af7d97534` at seal entry.
- The 145 repin greps (`44.0, 100.0` / `45.0, 100.0` /
  `293.0, 193.0`) answer EMPTY on this branch after `8f04df01b`.
- The g45 compose pins present on union `9bc722e5f` (core `:3922`,
  app `:670`); the g15 wiring inserts disjoint from the union
  hunks.
- All witnesses WRITTEN, NOT RUN (code-only swarm rule); the
  repin's assertions await the test phase with the rest.