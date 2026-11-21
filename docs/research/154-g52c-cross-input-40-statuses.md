# 154 — g52c cross-input: research/40's DONE statuses × the g50 net lane (81-120)

Status: CROSS-NOTE (code-only, W4-g50 re-probe round, branch
`task/W4-g50-net-apply-arms-3`). Input A: `docs/research/40-port-done-verification.md`
(the 34-row DONE hot-rows re-verification, 2026-11-12 — read at this branch's
base, byte-identical to `origin/task/W4-g52-net-relay-host`'s copy). Input B:
the g50 lane's 40 ids (81-120, batches 1-19 `ee28e7aef..549819cbb` + the
re-probe round `f8bacbc18/1c023f0ad/950f71a7e/31e33a81e`). Numbering: 154 =
max-on-origin 153 (f11's renumber wave `b03d53f1d`) + 1; 139 is RESERVED for
g52c-net-master (the g89h note renamed itself 139→145 for exactly this,
`41dabfbc8`) — this note does NOT take 139.

## 1. The 40-statuses census (input A, verbatim verdicts)

34 task rows: **26 PASS** (12 of them structural), **3 PARTIAL**
(M2-COMBAT-1 the player damage floor, M2-COMBAT-2 the bow damage branch,
M4-BOSS-6 `behindTiles`), **3 FAIL** (all one row: M4-NPC-4 the Hornet hover
constants + its self-invalidating citations + the partial shape), **2 STALE**
(the M4-NPC-3c/3h..3r display-row snapshot disagreement, the M1.5 Phase 1/2
measurement tables). The 40 module's own scope note: M6 net/save rows are
**NOT covered** ("~180 further ✅ rows remain unverified claims") — the g50
lane's ids live in that uncovered band, so no 40-row verdict directly
certifies or contradicts a g50 arm. The cross-input is therefore about
SHARED SURFACES, not verdict overlap.

## 2. The shared-surface intersections (input A × input B)

| 40 row (verdict) | g50 arm touching the same surface | Cross-input |
| --- | --- | --- |
| M4-DAMAGE (PASS) — `damage_var`/`calculate_damage_npcs_take`/`strike_npc_damage` | 117 PlayerHurtV2 / 118 PlayerDeathV2 (batch 16 `5b3cea587`) | The 117 host arm rides `hurt_inner` — the SAME fn whose missing player-side floor is 40's M2-COMBAT-1 PARTIAL. The net arm inherits the defect verbatim (a 0-damage PvP hit stays 0 on the wire path); the fix is the combat lane's, NOT a net-side clamp. No new deviation. |
| M2-COMBAT-1 (PARTIAL) — the `num2 < 1 → 1` floor (`Player.cs:38378-38381`) | 117 client quiet-hurt replay | Same inheritance: the client replay calls the same `hurt_inner`. When the floor lands, both legs fix together — the g50 witnesses pin the CURRENT (floorless) behaviour and must re-pin then. |
| M4-LOOT-1/2/3 (PASS/structural) — the drop-rule engine + coin chain | 88 ItemTweaker (re-probe `950f71a7e` FINALIZE-PARK) | The 88 census closes the loop: the ONLY vanilla 88 producer is `CommonCode.ModifyItemDropFromNPC` (gel 23 / mimic-banner 319 COLOR tints) — a loot-lane surface. 40's M4-LOOT rows certify the rule ENGINE, not the tint post-pass; the tint producer is unported on every branch, so the 88 park is consistent with the LOOT verdicts (no claim conflict). |
| M4-PROJ-1 (PASS structural) — the 1000-slot pool | 108 WiredCannonShot (batch 12), 112 SpecialFx, 95 MurderPortal (batch 6) | The net replays spawn through the same pool (`spawn_device_kind`/`spawn_with_data_ai`); 40's structural PASS covers the pool shape the replays ride. The 94 `/randomizeprojslots` shared static (re-probe `31e33a81e`) stores but the pool's slot-scan randomizer arm (`Projectile.cs:10831`) stays default-off — the store does NOT change the pool behaviour 40 certified. |
| M4-EVENT-1 (PASS) — the invasion framework | 113 CrystalInvasionStart / 114 Wipe / 116 Wait (batches 14-15) | The DD2 arms ride `World.start_invasion`-adjacent fields + the dd2 pools; 40's M4-EVENT-1 PASS is the GOBLIN framework (`Main.cs:64435`), the DD2 family is the `tr-core/dd2.rs` 6-test band (M4-EVENT-5d..5q, listed in 40's witness ledger but not a main-table row). No verdict overlap; the g50 114 wipe's `belongs_to_ooa` sweep reads the same dd2 tables the ledger certifies. |
| M4-TOWN-2/3 (PASS) — happiness + shop engine | 104 ShopOverride (batch 10), 92 SyncExtraValue moneyPing (batch 2) | The 104 client rebuild feeds `ShopEntry.value` into the same `resolve_stock` path 40's M4-TOWN-3 certifies; the override is a price INPUT, not an engine change. The 92 moneyPing is the `NPC.cs:90052` visual, outside the shop engine. No conflict. |
| M4-NPC-2/5/6 (PASS) — worm/caster/swim machines | 100 NpcPortalTeleport (batch 8), 96 PlayerPortalTeleport (batch 6) | The 100 snap writes the slot-keyed `net_npcs` store, not the AI machines; the 96 `teleport_style(4)` is the portal arm. NOTE for the g89/g11 union: 40's PASS rows do NOT cover the teleport-dust RNG order — that is research/145's GAP-7 (the editing-phase drain), still the merger's worklist. The g50 96 arm queues nothing (style 4 is not the dust style), so it is disjoint from GAP-7. |
| M3-CRAFT-6 (PASS structural) — the recipe table | 85 QuickStackChests (re-probe `1c023f0ad`) | The 85 host scan reuses `craft::CHEST_RANGE_PX` (the `NearbyChests` 600px constant the crafting AdjTiles scan reads) but NOT the recipe table; the g21 engine's category scores read `item_sorting`'s 52 layers (the Sort-button engine, S1-H-8), not the recipes. Disjoint. |

## 3. The g52 host-relay lane's own cross-points (input B × g52 tip 590b93565)

- **21 pre-switch × 90 SpawnInstancedItem**: g52's `item_send_gate`
  (`c3fdfd543`) never syncs instanced drops and rewrites IsAir→151; g50's 90
  client apply (batch 5 `631fcf458`) shares the 21 body via
  `apply_item_state` + the `net_reserved` local reservation. The g52
  `ItemDrop.instanced` field lands on THEIR branch; the g50 base has no
  field (the 90 arm's recorded deviation). At the union the 90 apply must
  read the landed field — keep-one on the 21 body, the g52 send gate is the
  survivor (it owns the pre-switch).
- **34 ChestUpdates × 85 SyncSlot**: g52's `host_chest_updates`
  (`cdc8fa7b8`) owns the place/destroy relay; the g50 85 arm's per-slot 32
  broadcasts are the CONTENT sync (`QuickStacking.SyncSlot`,
  `QuickStacking.cs:91-97`), a different message id — no overlap, both
  survive.
- **20 SectionRange latch × 109 per-edit 20s**: g52's latch filter
  (`58a73a3b7`) gates the 20 SENDS by the streamed-section latch; the g50
  109 host arm broadcasts one 1×1 20 per changed cell WITHOUT the latch
  filter (the C# `MassWireOperationStep` sends `SendData(17)` unconditionally
  per edit, `Wiring.cs:3353` etc. — the 17 path has no section gate). At the
  union the 109 arm's 20s ride the plain broadcast (the 17→20 fold is the
  port's recorded convention), NOT the latched `relay_tile_square` — the
  merger must not route them through the latch.

## 4. Verdict

Zero claim conflicts: no 40-row verdict is invalidated by a g50 arm, and no
g50 park is contradicted by a 40 PASS. The two inheritance notes (§2 rows
1-2: the hurt floor) and the three union keep-one notes (§3) are the
actionable cross-input. The 40 module's uncovered M6 band stays uncovered —
this note is a cross-input, not a re-verification of the net rows.
