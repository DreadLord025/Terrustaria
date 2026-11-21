# 239 — Net APPLY audit (W4-f21b, code-only, 2026-11-21)

Worktree TR-W4-f21, branch `task/W4-f21-net-ids-verify-tail`. Resume of the
TIMEOUT-died f21b round. Number choice: max-on-origin at exec time is **238**
(`origin/task/W4-c8-style66-crawling-critters`); the collision registry
(`origin/task/W4-d54-dust-sites-world:docs/research/150`) consumed 151–236 and
left 237+ free; 237 is already triple-claimed (e23/g48/g61), 238 taken → **239**.
**139 stays RESERVED for g52c-net-master** (the g50b note, research/154).

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
