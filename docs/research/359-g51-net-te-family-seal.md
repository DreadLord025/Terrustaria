# 359 — W4-g51 SEAL: the net TE-family FINAL verification + the net-121-family bible

> **Chain-banner: 309→359 (registry §9.2, d125-KEEPS-309).** Per the collision
> registry (`150-research-number-registry.md`, W4-d54 tip `b4c94432c`, §9
> UNIFIED FINAL — THE SINGLE AUTHORITY), 309 is d125's executed empress
> sheets ledger (`309-empress-sheets-ledger`, DONE `5f8569945`); this doc's
> round-3 self-pick of 309 was a post-§9 contested pick → **g51 takes 359**
> (§9.2 row 359). The original pick history below is chain-history, kept
> verbatim.

Number per the collision registry (`150-research-number-registry.md`, W4-d54
tip `0426245f2`, round 3 + §7.4 extension): the assignment blocks end at 307,
**308+ FREE**; a fresh all-tip sweep (361 `task/W4-*` tips, this SEAL's fetch)
shows 304 contested (d119 `c677b252a` 16:22:58 vs g17), 305 executed (f4
`e7ea8b24b`), 306/307 published reservations (g86/g43, §7.4.3), 308 taken
(g13 `3fe509158` 16:31:01) — **309 is the next free number**, claimed by this
doc. Branch `task/W4-g51-net-apply-arms-4`, worktree TR-W4-g51, code-only
(no cargo/tests/rustfmt/graphify per lane mandate; witnesses WRITTEN NOT RUN).

The g51 lane's FINAL commit `a8983fac2` landed the 8 BIG apply arms
(121/122/123/124/133/144/149/156) + the 139 master input + the 145/148
census constants. This SEAL re-verifies every claim against the C# and the
sibling lane tips, fixes what the re-read caught, and publishes the union
keep-one hunks.

## 1. SEAL-1 — the 8 BIG ids + 139 + 145/148 landed (re-verified)

All at this branch tip (post-SEAL fixes below):

| id | codec (net.rs) | host arm (netglue.rs) | client arm | C# re-read |
| --- | --- | --- | --- | --- |
| 121 | `TE_DISPLAY_DOLL_SYNC` :317, `DisplayDollSync`/`DollSyncData` :2099-2123, `write_doll_sync`/`read_doll_sync` :3220/:3244 | `C2s::DisplayDollSync` :1178 (id-force `:3925`, `apply_doll_sync` :1442, relay-except via `doll_sync_relay` :1519) | `S2c::DisplayDollSync` :3183 (`apply_doll_sync`, no relay) | `NetMessage.cs:1458-1474`, `MessageBuffer.cs:3920-3941`, `TEDisplayDoll.cs:766-849` — command 2 pose, 0/1/3+unknowns equip/dye/misc, `itemIndex < Length` gate, no-entity `ReadDummySync` no-relay: ALL MATCH |
| 122 | `TE_INTERACTION_REQUEST` :322, `TileEntityInteraction` :2132 | `C2s::TileEntityInteraction` :1193 (id-force `:3948`, -1 Clear+relay, `IsOccupied` scan + `TryGet` gate + Set + relay) | :3190 (Clear / `SetInteractionAnchor`: chest −1, talk clear, local inventory-open + 10/12 cue, `was_clear` :3224) | `MessageBuffer.cs:3942-3977`, `TileEntity.cs:221-234` (`active && !dead && anchor==id`), `:289-313` — MATCH (the sound flag reads the PRE-set anchor state) |
| 123/133/149 (+89) | `TryPlacingItem` :2050, the four `*_TRY_PLACING` constants + arms | `host_try_placing` :1348 (one body, four dispatch arms :1154-1169) | — (C2s-only; the answer is the 86) | `MessageBuffer.cs:3377-3393`/`:3978-3994`/`:4147-4163`/`:4344-4360` (the `DefaultAssignNewItemsToPlayer(whoAmI)` scope), `TEItemFrame.cs:69-86` + the three twins — MATCH; the twins differ ONLY in `RangeFrame` rect + `DropItem` box (the port's edit chain owns those); the sender grab-delay rides `apply_grab_delay` (the `ReserveForLocalPlayer` 100-tick lock, `WorldItem.cs:295-309`) |
| 124 | `TE_HAT_RACK_ITEM_SYNC` :333, `HatRackItemSync` + `is_dye`/`slot_index` :2148-2169 | :1235 (id-force `:4000`, unfold, `TryGet<TEHatRack>` + `slot>=2` refusal, `ReadItem`, relay re-folded via `hat_rack_relay` :1601) | :3241 (unfold + refusal + `ReadItem`) | `MessageBuffer.cs:3995-4022`, `TEHatRack.cs:378-403` (STACK-BEFORE-PREFIX), `NetMessage.cs:1489-1508` (the +2 fold, the 5-byte air fallback) — MATCH |
| 144 | `REQUEST_QUEST_EFFECT` :344 (bodyless, C2s-only) | :1268 → `host_dryad_stardew` :1643 | — (host-only `netMode==2`) | `MessageBuffer.cs:4304-4309`, `NPC.cs:19184-19229` — the ai writes / probe / spawn re-read line-by-line; **one fix landed** (§4) |
| 156 | `LEASHED_ANCHOR_PLACE_ITEM` :356, `LeashedAnchorPlaceItem` :2178 | :1278 (`TryGetAt` + kite/critter type gate + `TeData::Anchor(itemType)`) | — (C2s-only) | `MessageBuffer.cs:4439-4449`, `TELeashedEntityAnchorWithItem.cs:32-36` — MATCH; `RespawnLeashedEntity` parks M7-TE-2 (documented) |
| 139 | `COUNTS_AS_HOST` :253, `CountsAsHost` :1960 (S2c-only) | the join arm :2111-2122 (`slot_counts_as_host` store + push-when-true) | :3089 (store on the named slot) | `NetMessage.cs:2876-2883` + `:1587-1590`, `MessageBuffer.cs:930-935` + `:4230-4237`, `TcpAddress.cs:23-26` — MATCH (§5) |
| 145/148 | `SHIMMER_ITEMS_DEPRECATED` :258 / `ITEM_ENEMY_LOCK_DEPRECATED` :271 | — | — | `MessageID.cs:304`/`:310`; zero writer cases, zero reader cases, zero `SendData(145|148` sites tree-wide (re-grepped this SEAL) — FINALIZED census constants, the 24 precedent |

The 86/87 seam codecs (`TileEntitySharing`/`TileEntityPlacement`) + the
client 86 reader (`add_net`/`remove_by_id`) + the host 87 placement arm are
the g50 lifts — byte-verified in §2.

## 2. SEAL-2 — the g50 TE-store keep-one: NOT byte-identical; g51's copy is the SUPERSET

Compared `origin/task/W4-g50-net-apply-arms-3` `549819cbb` (the lift source)
AND its tip `e2fd0637d` (both carry blob `737eb9c60`) vs this branch's
`crates/tr-core/src/tile_entities.rs` (blob `f8b59f48b` pre-SEAL):
`git diff` = **98 insertions / 34 deletions — NOT byte-identical**. Every
delta is a g51 extension ON TOP of the g50 seams; none rewrites a g50
behaviour except the one bug-fix below:

1. **The jar joined** `has_item_slot`/`read`/`write_net` comment/`place`/
   `place_unchecked` (`TEDeadCellsDisplayJar` — its `TryPlacing` writes
   `result.item`, `TEDeadCellsDisplayJar.cs:69-86`; g50's copy predates the
   149 arm).
2. **The doll ctor payload** in `place`/`place_unchecked` (the empty 9+9
   arrays + misc None + pose 0, `TEDisplayDoll.cs:101-122`; g50's copy left
   `TeData::None`, which would break the 121 arm's payload match).
3. **`remove_by_id` tombstone gate** (the behavioural fix): g50's copy
   re-removed an already-tombstoned id and returned `Some` with stale
   coordinates; the port's slot-tombstone book means the C# `TryGet` fails
   on the removed `ByID` entry (`TileEntity.cs:126`/`:159-167`) — the g51
   copy returns `None` when `by_position` no longer maps the cell to the
   id. Pinned by `w4_g51_add_net_forces_the_id_and_remove_tombstones` (the
   second `remove_by_id(5)` asserts `None`).
4. **Witness renames** `w4_g50_*` → `w4_g51_*` + the jar round-trip block
   appended to the net-codec witness (the g50 originals are the same tests
   minus the jar).

**Dedupe-hunk (union merge): KEEP g51's `tile_entities.rs` WHOLE; DROP the
g50 copy.** The g51 file is a strict superset of the g50 seams (the four
`write_net`/`read_net`/`add_net`/`remove_by_id` bodies are the g50 bodies
with the tombstone gate added and comments re-cited). The g50 netglue 86/87
arms + `broadcast_tile_entity` are byte-identical lifts on this branch
(host 87 :1100-1144 vs g50 :869-913; client 86 :3161-3179 vs g50
:3864-3882; `broadcast_tile_entity` :1306 vs g50 :2403 — all verified
line-for-line this SEAL) — keep-one either side, no conflict. The g50
`ItemTweaker` FINALIZE-PARK client arm (88) is g50-unique — KEEP g50's.
The 86/87 `net.rs` structs + all four codec arms per variant verified
code-identical (this SEAL's block-diff, §3 method).

## 3. SEAL-3 — the f21 codecs keep-one: BYTE-EQUAL, re-pinned

Compared this branch's `net.rs` vs `origin/task/W4-f21-net-ids-verify-tail`
`5274a0972` (the lift source), per-block (structs/impls/arms extracted and
compared raw, comments included):

- **Structs byte-identical**: `TryPlacingItem`, `DisplayDollSync`,
  `TileEntityInteraction`, `HatRackItemSync`, `LeashedAnchorPlaceItem`,
  `CountsAsHost` + `impl HatRackItemSync` (the unfold helpers).
- **Every codec arm byte-identical** (msg_id / encode / decode, both C2s
  and S2c where the variant exists): the TryPlacing quadruplet ×3 arms
  each, DisplayDollSync ×6, TileEntityInteraction ×6, HatRackItemSync ×6,
  RequestQuestEffect ×3, LeashedAnchorPlaceItem ×3, CountsAsHost ×3,
  TileEntitySharing/TileEntityPlacement ×4 each (vs g50).
- **The free helpers byte-identical**: `write_doll_sync`/`read_doll_sync`
  (18 lines each).
- **The id constants byte-identical**: all nine (`ITEM_FRAME_TRY_PLACING`
  89 … `LEASHED_ANCHOR_PLACE_ITEM` 156 + `COUNTS_AS_HOST` 139).
- **f21b tip drift check**: `5274a0972..f7bc25471` (and the rename tip
  `5dfc6a764`) touches net.rs by exactly +5 lines — the
  `PlayerNet.angler_quests_finished/golfer_score` fields (f21b's 76 arm).
  ZERO codec drift; the lift stays byte-equal at the f21 tip.

**Keep-one verdict** (confirms f21b's research/276 §4): the net.rs codec
bodies dedupe to ONE copy (either side — byte-equal); g51's netglue arms
survive (f21 parked its arms); f21b's 57/76/58 arms + the two PlayerNet
fields are f21b-unique — KEEP. The f21b audit doc renamed 239→276 at tip
`5dfc6a764` per registry §6.3 (executed, confirmed this SEAL) — cite
research/276, NOT 239 (239 = d42's kinds doc).

## 4. SEAL-4 — the 144 dryad: f30 System-5 compose (disjoint, keep-both) + the row-dup + TWO fixes

**Disjointness verified** against `origin/task/W4-f30-achievement-notify-2`
(tip `973ec04c4`; `dryad_stardew.rs` added `f2d4d469a` 07:21:10):

- **f30 owns the DECISION CORE + GIFT**: `can_dryad_play_stardew_animation`
  (`Main.cs:40449-40457`), `dryad_do_stardew_valley_bit`
  (`Main.cs:40280-40295`: the latches, `ConsumeItem(5275)`, the thank-you
  line, progression 46), `override_dryad_chat_text`
  (`NPCChatPanel.cs:195-213`), the cola button text, the joja latches —
  NONE of these exist on g51.
- **g51 owns the NET ARM**: `host_dryad_stardew` (the `NPC.cs:19184-19229`
  body behind `144`) + the 27 broadcast — f30 has no net arm.
- **The overlap is the animation body**: f30's
  `have_dryad_do_stardew_animation` (the local/host direct path) vs g51's
  `host_dryad_stardew`. Both port the same C# function; the bodies agree
  on the ai writes, the probe (`Bottom+(100,0)`, `Clamp(Y-10,10,
  maxTilesY-10)`, the 15-row scan, the first-row-solid own-bottom choice)
  and the spawn point. **KEEP BOTH** (f30's is the single-player/local
  path its gift calls; g51's is the host's 144 apply + relay). At union
  they may fold onto one shared helper — NOT required for correctness;
  the compose note goes to the merger, no drop-hunk.
- **f30's gift path sends 144 as a client** (`Main.cs:40285-40288`): f30's
  port runs the animation directly on the host path (documented in its
  comment); the client-side `C2s::RequestQuestEffect` send is the g51
  codec's — the f30 gift's client leg should call it at union (one-line
  compose note, f30's lane).

**STARDEW_JUMINO row-dup — CONFIRMED**: f30's `projectile/data.rs:3977`
carries a literal `995 =>` row with the IDENTICAL tuple `(240.0, 104.0,
192, -1, TIME_LEFT_DEFAULT * 5, 0, false, false, false, RESET_STATE)`;
g51's `STARDEW_JUMINO =>` row (data.rs:3986) is the same values behind the
named constant. `proj_data` is `#[deny(unreachable_patterns)]` — the union
MUST dedupe. **Keep g51's row** (the named constant + the two membership
folds f30 lacks: `net_important` data.rs:4157 and `ignore_water` :4237,
`Projectile.cs:9448`/`:9455`); **DROP f30's literal 995 row**. f30's
post-spawn `p.friendly/p.tile_collide/p.ignore_water` writes
(`dryad_stardew.rs:151-154`) become redundant with the row + the pool's
`friendly: true` spawn default (projectile.rs:787) — harmless if kept,
droppable at union.

**FIX 1 (this SEAL): the host spawn centre**. The C# `NewProjectile` X/Y
overload centres the box: `position = (X − width/2, Y − height/2)`
(`Projectile.cs:10876-10877`). g51's host arm passed the spawn point
UNCENTRED (the f30 local twin centres correctly, `spawn.x − 120, spawn.y −
52`). Fixed: `spawn.0 − 120.0, spawn.1 − 52.0` (netglue.rs:1708-1709). The
dryad witness now pins the exact position (96, 536) + friendly /
!tile_collide / ignore_water / net_important / time_left 18000 (the
fixture's probe maths re-derived: Bottom (109,640) → probe (209,640) →
tile (13,40) → spawn (216,588) → centred (96,536); the OLD witness comment
carried a wrong tile (6,40) — corrected).

**FIX 2 (this SEAL): the tombstoned-id `TryGet` gates**. The C# `TryGet`
fails on a removed `ByID` entry (`TileEntity.cs:126` removes the entry;
`:159-167` looks it up). The port's id book TOMBSTONES slots (`remove_at`
leaves `TeData::None` in place), so a bare `get_by_id` still hits a dead
id. The 121/122/124 host gates, the 121/124 relays and the 122/124 client
gates now filter on position-book liveness (the
`broadcast_tile_entity` pattern): a tombstoned id refuses the interaction
(no anchor, no relay), consume-and-drops the hat sync, and falls to the
doll `ReadDummySync`/`WriteDummySync` paths. New witness
`w4_g51_a_tombstoned_te_id_refuses_the_interaction_and_the_hat_sync`
(HatRack tombstoned on both sides — the kind still reads HatRack, ONLY the
liveness filter refuses). WRITTEN NOT RUN.

## 5. SEAL-5 — the 139 master input (g52c): verified, feeds the master

- **g52 tip check**: `origin/task/W4-g52-net-relay-host` tip `0224fff80`
  (the final-audit round) greps ZERO for `counts_as_host`/`CountsAsHost`
  in crates/ — the g52c master sub-round is still unpushed; its
  research/139 stays RESERVED (registry §7.2: "139 still EMPTY at all 360
  tips"). g51's join push is the master's INPUT, not a claim on 139-the-doc.
- **The C# chain re-read**: the State-3→10 join arm
  (`MessageBuffer.cs:925-935`) computes `DoesPlayerSlotCountAsAHost`
  (`NetMessage.cs:2876-2883`: `State == 10 &&
  Socket.GetRemoteAddress().IsLocalHost()`), stores
  `Main.countsAsHostForGameplay[whoAmI]` UNCONDITIONALLY, and sends 139 to
  the joiner ONLY when true (`:932-934`). `TcpAddress.IsLocalHost` is
  `Address.Equals(IPAddress.Loopback)` (`Net/TcpAddress.cs:23-26`) —
  exactly IPv4 127.0.0.1, NOT IPv6 ::1, NOT the 127/8 block.
- **The port matches**: `Server::slot_counts_as_host` (net.rs:4928) reads
  the accept-time `Slot::peer_ip` against `Ipv4Addr::LOCALHOST` gated on
  `slot_state::IN_WORLD` (10); the join arm (netglue.rs:2111-2122) stores
  host-side + pushes only-when-true; the client arm (:3089) stores on the
  named slot (`MessageBuffer.cs:4230-4237` — the array is GLOBAL per-slot,
  the push names the receiver's OWN slot; the port keeps it on the peer
  snapshot, the creative-power authority consumer is g52c's).
- **The g52c hand-off**: the master-final table (research/139 when pushed)
  reads `PlayerNet::counts_as_host` as the host-authority flag's join
  source; the writer `number2 == 1f` bool (`NetMessage.cs:1587-1590`) is
  the codec's `bool` field — byte-verified.

## 6. The NET-121-FAMILY bible (the TE/doll/anchor union's single entry point)

The 121-anchored family = the TE-state apply arms + their seams. Doc map
(registry round-3 numbers; sibling renames are THEIR lanes' to execute):

| doc | lane | role |
| --- | --- | --- |
| research/17 (base trunk) | g48 SEALed | the ids 1-162 census master (codec/writer/reader per id) |
| research/276 (f21b, 239→276 EXECUTED `5dfc6a764`) | f21b | the APPLY coverage census g48-g52 + the union keep-one notes + the f21b 57/76/58 survivors |
| research/271 (g50, 154→271 EXECUTED `e2fd0637d`) | g50b | the g52c cross-input of research/40's DONE statuses × the net lane; the 139-RESERVED provenance |
| research/23 (base trunk) | — | the wiring/TE audit (the family's C# inventory) |
| **359 (this doc)** | g51 | the TE-family SEAL: the 8 BIG arms verified, the g50/f21 keep-one hunks (§2/§3), the f30 compose (§4), the 139 input (§5) |
| research/139 (RESERVED, unpushed) | g52c | the master-final table; g51's `counts_as_host` join push is its input (§5) |

**Merge order for the family** (extends f21b 276 §7): codecs first (f21
base + g51 lifted keep-ones — byte-equal, §3), then the tr-core TE store
(g51's superset copy WHOLE, §2), then the g50/g51 netglue arms (86/87
byte-equal keep-one; the 8 BIG arms g51-unique), then f30's dryad decision
core (keep-both + the 995-row dedupe, §4), then g52's send-gating census,
then g52c's master (reads the 139 flag).

**Remaining family gaps** (owners unchanged from f21b 276 §3): the TE draw
halves → d57-TE-render; `RespawnLeashedEntity`/the LeashedEntity system →
M7-TE-2; the doll `_dollPlayer` render Player + `SupportedUseStylePoses` →
d57; the AI_192 Jumino frame machine + the Stardew draw
(`Main.cs:28170-28173`) → the animation lane; the 88 ItemTweaker producer
→ the loot lane (g50's FINALIZE-PARK stands).

## 7. Verification record

- Sibling tips fetched + pinned: g50 `e2fd0637d`, f21 `5dfc6a764`, g52
  `0224fff80`, f30 `973ec04c4`, d54 `0426245f2` (registry round 3 + §7.4).
- Block-diffs: PowerShell extraction of each struct/impl/arm (brace-count
  bounded), raw + code-only compare — all §3 rows `raw=True`.
- C# windows re-read ≤100 lines each: `NetMessage.cs:1458-1537`/`:1585-1592`/
  `:2860-2889`, `MessageBuffer.cs:920-944`/`:3266-3290`/`:3377-3421`/
  `:3920-4032`/`:4230-4249`/`:4304-4328`/`:4439-4483`, `TileEntity.cs:108-183`/
  `:215-239`/`:285-319`, `TEDisplayDoll.cs:93-193`/`:760-849`,
  `TEHatRack.cs:360-409`, `TEItemFrame.cs:40-94`, `TEWeaponsRack.cs:122-164`,
  `TEFoodPlatter.cs:55-89`, `TEDeadCellsDisplayJar.cs:60-89`,
  `TELeashedEntityAnchorWithItem.cs:30-64`, `NPC.cs:19184-19233`,
  `Projectile.cs:9442-9461`/`:10855-10934`/`:41501-41560`,
  `Item.cs:49649-49770`, `WorldItem.cs:273-321`, `Main.cs:40278-40297`,
  `Utils.cs:1762-1773`/`:1807-1814`, `TcpAddress.cs:20-29`.
- Fixes landed this SEAL (code): the host dryad spawn centre (§4 FIX 1) +
  the seven tombstone-liveness gates (§4 FIX 2) + 1 new witness + the
  dryad witness's exact-position/flag pins + its corrected probe maths.
  All WRITTEN NOT RUN (code-only lane).
- No cargo/tests/rustfmt/graphify invoked. No other branch touched.
