# 276 — Net APPLY audit (W4-f21b, code-only, 2026-11-21)

> **RENUMBERED 239 → 276** (research/150 round-2 refresh, d54 tip `72226e500`,
> §6.2/§6.3): this doc's first add `f7bc25471` 2026-11-21T14:11:45+02:00 loses
> key 239 to d42's `239-d42-final-keep-one-and-band.md` (`bac408be0`
> 12:55:30 — first-pushed-wins); g56's `239-void-family-bible.md`
> (`54c63d8d4` 13:57:53) loses too → **274**. The registry's §6.3 row for
> `f21-net-ids-verify-tail` assigns **276**; refs re-pointed on this branch
> (research/17, INDEX, ANALYSIS ×2, PORT_MAP, anchors.txt). Body below is
> content-preserving except this block, the header, the number-choice
> paragraph and the §6/§7 seal additions.

Worktree TR-W4-f21, branch `task/W4-f21-net-ids-verify-tail`. Resume of the
TIMEOUT-died f21b round. Number choice (SUPERSEDED — historical record):
max-on-origin at exec time was **238**
(`origin/task/W4-c8-style66-crawling-critters`); the collision registry
(`origin/task/W4-d54-dust-sites-world:docs/research/150`) consumed 151–236 and
left 237+ free; 237 was already triple-claimed (e23/g48/g61), 238 taken → picked
**239** — which d42 had already pushed (see the renumber block above; the
registry now bans max-on-origin picks, losers take ≥281 after a fresh sweep).
**139 stays RESERVED for g52c-net-master** — re-confirmed at the registry
round-2 refresh (§4-REFRESH: g89 self-renamed its rod doc 139→145 at
`41dabfbc8`, g50's cross-note — itself assigned 154→271 — explicitly does not
take 139; if g52c pushes a 139 doc it keeps 139, single claimant).

## 1. Git log vs origin — what landed

- `origin/main` = `a725acb04` (wave-3 magic+summon, 56/56). **Nothing from the
  net lanes is in main**: `git merge-base --is-ancestor` exits 1 for f21
  (`5274a097`), g48, g49, g50, g51 and g52 tips alike.
- f21 tip at resume: `5274a0972` (docs+anchors, the 43-id tail census in
  research/17) — pushed, HEAD == origin. The tail census lives in
  **research/17's "The W4-f21 tail census" section**, not a research/107 file
  (g48's cross-note `a546fd8ee` already recorded that 107 does not exist on any
  pushed branch; the registry's §2 confirms 107 belongs to g24).
- This round adds `93df2fbb1` (the small-arm port) + `5230cb99b` (the witness
  splice repair + the 76 host own-echo gate) — pushed.

## 2. Apply-arm coverage census (MessageBuffer ids 1–162, per lane tip)

Vocabulary: **WIRED** = the arm runs the C# body; **PARKED** = codec exists,
arm no-op, owner named; **DEAD/IGNORE** = the f21 census verdicts; **N/A** =
join-FSM-owned. Sources: g48's research/17 status table (`a546fd8ee`), the
g49/g50/g51 netglue arm scans at their tips, g51's f21 cross-check commit
(`fcfb92f26`), g52's ANALYSIS relay census (`15a7b500b`), research/154.

| band | lane | WIRED at tip | remaining (owner) |
| --- | --- | --- | --- |
| 1–40 | g48 (`74e1ffa3c`) | 2,4–14,16,17 (incl. kinds 15/21),19 (incl. 2–5),20–23,27–36,40; N/A 1,3,37,38; DEAD 15,24,25,26 | **18** clock lane (internal 205); **39** M6-NET-5 (FindOwner/22 re-announce) |
| 41–80 | g49 (`af1f172f5`) | 42,45,56,60,61,63,64,65,66,68,72,73,74,75,77 (client no-op — Animation unported),78; pre-g49: 46,47,49,50,53,54,59,69,70,71,80 | **41/43** parked on g48 (S1-I-4); **48/52/79** unclaimed (below); **51/55/62** claimed by `W4-d115-dodge-family` (arms at its tip); **57/76/58-host** WIRED by this f21b round; 44/67 IGNORE (f21); 58-client parked (audio) |
| 81–120 | g50 (`714f1f736`) | 81,83–87,89–92,93,95,96,99–118,119,120; **g50b re-probes wired 109/85/94** (`f8bacbc18`/`1c023f0ad`/`31e33a81e`) | **88 FINALIZE-PARK** (`950f71a7e`: the only vanilla producer is CommonCode.ModifyItemDropFromNPC's six COLOR-only sends — the loot lane's unported half; ItemDrop has no inner stats); 85 client blocked leg FINALIZE-PARK (pure RoomCheckParticle pulse); 94 /quickload-* probe PARKED (QuickLoad serializer) |
| 121–162 | g51 (`fcfb92f26`) | 125–128,130,131,134,135-host,136,137,140–143,147,153,157,159,162; f21 cross-check lifted+wired 129,138,139,145/148 (census),146,152,160,161; pre: 151,154,155 | **121/122/123/124/133/149/156** TE families apply-owner-needed; **144** dryad Stardew (NPC.cs:19184-19229 + proj 995); **150** spectating unported; **158** DOCUMENTED SKIP (netMode+seed-gated TeamSwap spawn); 135-client ghost fade render-parked; **132** dead skip (zero PlayNetSound sites) |
| relay/send | g52 (`590b93565`) | the SendData gating census: broadcast flag, 20 latch filter, 27 flag3 + projPet table, 28 cascade order, 21 send-gate (instanced/IsAir→151), 162 ack send, 34 host place/destroy pass | client 34 direct pass is g48's (disjoint halves, research/154) |

## 3. The remaining-gap list (unclaimed apply arms after the lane union)

These have codecs (f21/d-lanes) but **no lane owns the apply arm**:

| id | arm | C# reader | what it needs | suggested owner |
| --- | --- | --- | --- | --- |
| 39 | host+client ReleaseItemOwnership | `MessageBuffer.cs:2232-2255` | WorldItem.FindOwner + the 22 re-announce; g48's table already assigns M6-NET-5 | item lane (M6-NET-5) |
| 41 | host+client ItemAnim | `:2271-2289` | peer `itemRotation/itemAnimation/channel` store (PlayerNet has no fields) + relay; the render reads `swing_anim` locally today | S1-I-4 / render-peer lane |
| 43 | host+client ManaEffect | `:2307-2324` | `Player.ManaEffect` star pickup FX (unported) + relay | S1-I-4 / mana-FX lane |
| 48 | host+client LiquidUpdate | `:2407-2446` | the ±10px SpamCheck box, tile liquid/liquidType store, host SquareTileFrame + the b2==0 drained re-push | liquid lane (g46/g47 adjacency) |
| 52 | host LockAndUnlock | `:2531-2564` | Chest.Unlock/Lock + WorldGen.UnlockDoor + relay + SendTileSquare(2) | chest/edit lane |
| 79 | host+client PlaceObject | `:3192-3215` | the spam/InWorld(x,y,10)/section gate + WorldGen.PlaceObject + SendObjectPlacement relay (sbyte random + direction bool) | edit/object lane |
| 58 | client instrument switch | `:2652-2679` | PlayGuitarChord (4057/4372/4715) / PlayDrums (4673) / musicPitch + Item26/35/47 cue | audio lane (host relay WIRED f21b) |
| 121/122/123/124/133/149/156 | TE doll/anchor/rack/hat-rack/platter/jar/leash | `:3920-4022`, `:4147-4163`, `:4344-4360`, `:4439-4449` | the TE entity consumers (g51 apply-owner-needed; the TryPlacing quadruplets share one body, f21 byte-equality pin) | TE lane (research/23 wiring adjacency) |
| 144 | host RequestQuestEffect | `:4304-4309` | NPC.HaveDryadDoStardewAnimation (NPC.cs:19184-19229) + the 995 projectile | dryad/Stardew lane |
| 150 | host+client SpectatePlayer | `:4361-4384` | the spectating system (`Player.spectating`) | S1-I-4 |

Documented non-gaps (deliberate verdicts, NOT missing work): 18 (clock rides
205), 88 (no producer exists to apply), 85-client (FX-only), 77-client
(Animation unported, no-op is the arm), 135-client (render ghost fade), 158
(seed-gated large spawn), 132 (dead), 24/25/26/44/67/83/93/138/145/148
(dead/ignore/bodyless census verdicts).

## 4. Keep-one notes for the union merge

- **PlayerNet fields are disjoint across lanes**: g49 adds `mana/max_mana/team`,
  g51 adds `host/counts_as_host`, f21b adds `angler_quests_finished/golfer_score`.
  Keep-both; every ctor in netglue uses `..Default::default()` (verified all 10
  sites), so the field additions compose without ctor edits.
- **World fields disjoint**: g49's `angler_who_finished_today` Vec vs f21b's
  `t_good/t_evil/t_blood` — keep-both, both Default-safe (`World::new` literals).
- **57/76/58**: no other lane touches these arms (g49/g50/g51 netglue scans show
  zero hits). At the f21↔f21b union the f21b WIRED arms replace f21's parked
  `C2s::InstrumentSound(_) | C2s::QuestsCountSync(_)` / `S2c::WorldEvilBytes(_)`
  lines — keep-one, f21b survivor. The 57 join push sits immediately before
  `S2c::InitialSpawn` in `host_stream_sections` (the `:867` position); g48's
  join-arm rework keeps that send line — re-verify at union.
- **g51's lifted f21 codecs** (129/138/139/145/146/148/152/160/161): keep-one the
  net.rs bodies (byte-identical lifts), g51's netglue arms survive. f21b does
  not touch those ids — no three-way conflict.
- **g50b re-probes** (109/85/94) supersede g50 batch-12/18/19's parked arms on
  the same branch lineage — already keep-one; the c42/g21 engine copies carry
  their own keep-one notes (`f8bacbc18`, `1c023f0ad`).
- **research/154's three union notes stand**: the 21 body g52 send-gate survivor
  + 90 instanced-field read; 34 place/destroy (g52 host) vs 85 content-sync
  disjoint; the 109 per-edit 20s must NOT ride the latched relay.
- **Witness namespaces disjoint**: `w4_f21b_*` (5, this round) vs `w4_g51_f21_*`
  vs `w4_f21_*` — no name collisions.

## 5. What f21b ported this round (the small unclaimed arms)

- **57 WorldEvilBytes** (`MessageBuffer.cs:2631-2638`, writer
  `NetMessage.cs:1054-1058`): client stores the three bytes into new
  `World.t_good/t_evil/t_blood` (`WorldGen.cs:4180-4184`); the join push wired
  at `MessageBuffer.cs:867`'s position (ahead of 49). The `CountTiles` recount
  producer (`WorldGen.cs:71148-71173`, banker's Round + the nonzero-total
  floor-of-1) parks with the tile-count pass — documented on the fields.
- **76 QuestsCountSync** (`:3164-3182`): host own-echo gate (`:3167` BEFORE the
  force), id-force (`:3171`), peer store (`:3174-3175`), relay-except (`:3178`);
  client own-echo skip + peer store. `PlayerNet.angler_quests_finished/
  golfer_score` new (the local Player already had both, `player.rs:782/:785`).
- **58 InstrumentSound host half** (`:2642-2650`): id-force + relay-except, raw
  float pitch, no host sound. The client instrument switch stays PARKED (§3).

Witnesses: `netglue::tests::w4_f21b_world_evil_bytes_join_push_stores_the_three_bytes`,
`_quests_count_sync_host_forces_id_stores_and_relays` (3-app, bogus-id-99 force
pin), `_quests_count_sync_client_skips_own_echo_stores_peer`,
`_quests_count_sync_host_drops_own_slot_echo`,
`_instrument_sound_host_forces_id_and_relays_pitch` (raw-frame drain — the
client apply parks, the relay is the observable). **WRITTEN NOT RUN**
(code-only lane; no cargo per the task).

Commits: `93df2fbb1` (port + 4 witnesses), `5230cb99b` (splice repair + the
host echo gate + 5th witness). Both pushed to
`origin/task/W4-f21-net-ids-verify-tail`.

## 6. SEAL verification (f21c round, 2026-11-21, code-only)

The f21b WIRED arms re-verified at this branch tip `f7bc25471` (pre-rename):

- **57**: client store `netglue.rs:2005-2009` (`t_good/t_evil/t_blood`,
  `:2634-2636` cites), join push `:962-973` (the `:867` position, ahead of 49),
  fields `tr-core/world.rs:572+` (Default-safe `:811`).
- **76**: host arm `netglue.rs:852-867` — own-echo gate `:853-855` runs BEFORE
  the store/relay (the `:3167`-before-`:3169` order), peer store `:857-858`,
  relay-except `:859-866`; client arm `:2019-2027` (own-echo skip + peer
  store); `PlayerNet.angler_quests_finished/golfer_score` in `tr-app/net.rs`.
- **58 host**: `netglue.rs:835-843` (id-force `:838`, raw pitch, relay-except
  `:842`); client `:2014` stays the documented park.
- All 5 `w4_f21b_*` witnesses present (`netglue.rs:5439/:5457/:5519/:5553/:5602`);
  anchors.txt row `w4_f21b_quests_count_sync_host_drops_own_slot_echo` intact.
- **Keep-one re-probe at the ADVANCED sibling tips**: g48 moved
  `74e1ffa3c`→`2b61530b4` (docs-only correction phase — the diff touches zero
  crates/ files) and g52 `590b93565`→`856edb86c` (join-bundle WIP checkpoint);
  `git grep WorldEvilBytes|QuestsCountSync|InstrumentSound` over crates/ at
  g48/g49/g50/g51/g52 tips is EMPTY on all five — the §4 keep-one verdicts
  (f21b survivor, no other lane touches 57/76/58) STAND.

## 7. NET-TAIL-FAMILY bible (the union merger's single entry point)

Doc map after the registry round-2 renumbers (research/150 §6, d54 tip
`72226e500`; sibling renames are THEIR lanes' to execute — cited here at
assigned numbers):

| doc | lane | role |
| --- | --- | --- |
| research/17 (base trunk, SEALed by g48 `9460c85d9`) | g48 | the ids 1–162 census master: per-id codec/writer/reader table + the f21 tail-census section + the f21b cross-note; g48's correction receipts append AFTER the seal (re-probe citations only) |
| **276 (this doc)** | f21b | the APPLY coverage census (g48–g52 tips), the remaining-gap list (§3), the union keep-one notes (§4), the SEAL (§6) |
| 271 (g50, `154→271` assigned) | g50b | g52c cross-input: research/40 DONE statuses × the net lane; the 139-RESERVED provenance |
| 272 (g48, `237→239→272` assigned) | g48 | the f11 minecart fail-tile tail-fix hunk (upstream publish; NOT a net id — rides the family INDEX only) |
| g48's 18-send-switch publish (`244-c28-18-send-switch.md` at tip `2b61530b4` 14:30:16 — POST-CUTOFF pick; 244 is inside the registry's round-2 assigned block (244=c31 draft) and f39 pushed its own 244 at 14:32:01; g48/f39/c31/d81 must self-adjudicate per the §6.3 cutoff note or request a round-3 refresh — NOT this lane's call) | g48→c28 | id 18 has ZERO SendData sites; the clock rides id 7 WorldData (event edges + the 3600-tick heartbeat); the exact 5-step internal-205→id-7 switch handed to `W4-c28-net-send-cadences` |
| g48's 39 seed (`245-39-item-ownership-seed.md` at tip — same POST-CUTOFF caveat: 245=d81 draft in the round-2 block) | g48→item lane | 39 ReleaseItemOwnership adjudicated to the ITEM lane (d98 region), NOT M6-NET-5-as-rowed (that ATLAS row is the landed 27/28/29 sync); FindOwner/ReserveFor scope `WorldItem.cs:188-285` + the 151 slot-reuse remainder |
| 139 (RESERVED, unpushed) | g52c-net-master | the g52c master-final table; g52 tip `856edb86c` carries NO 139 doc — the sub-round is UNPUSHED (g48's receipt `d3200ef3a`); single claimant keeps 139 when pushed |

Owner lanes for the §3 remaining gaps (task-seeds; the registry's ≥281 rule
applies to any new docs they publish):

- **39** → item lane (d98 region) per g48's seed — supersedes §3's
  "M6-NET-5" suggestion (the seed's adjudication: the ATLAS M6-NET-5 row is
  the landed 27/28/29 sync, the ROADMAP pointer is a forward-reference).
- **18** → c28 cadence lane per g48's send-switch publish — supersedes §2's
  "internal 205" framing (the switch target is id **7**, not 18).
- **41/43** → g48-parked (S1-I-4 render-peer/mana-FX); **48** → liquid lane
  (g46/g47 adjacency); **52** → chest/edit lane; **79** → edit/object lane;
  **58-client** → audio lane; **121/122/123/124/133/149/156** → TE lane
  (research/23 adjacency); **144** → dryad/Stardew lane; **150** → S1-I-4.
- Merge order for the net union: codecs first (f21 base + g51 lifted
  keep-ones), then the g48–g52 apply arms per §2, then f21b's 57/76/58-host
  survivors per §4, g52's send-gating census last (it reads the arms).
