# 362 — W4-g55 LOADOUT-REST FINAL: the loadout-family declaration + the persist/net compose census

**Status: FINAL (code-only retire round).** Branch `task/W4-g55-ui-loadout-save`,
base tip `ca1bc92d` (dev-2/3/4 net-147 LoadoutSync + clientCloneLoadouts +
pendingItemReuse landed). **Number 362 — rename chain 334→362 per research/150
§9.2 (UNIFIED FINAL, d54 tip b4c94432c):** the original self-pick **334** (taken
under §8 round-4's "334+ FREE" guidance after a fresh all-tip sweep found no
g55 assignment) landed in g62's assigned 334 (`334-g62-hit-dust-final`,
EXECUTED 87299d2c8); §9.2 re-assigns this doc **362**. This doc is the
loadout-family merger-bible: the remaining
loadout arms censused, the two unclaimed gameplay arms LANDED this round, the
`.plr`/`.trplayer` compose confirmed against f44/g88/g87, and the net-147
apply-side lane adjudicated.

## 1. The loadout-family declaration (what g55 owns end-to-end)

The three-loadout equipment swap (`Player.Loadouts[3]` / `CurrentLoadoutIndex`,
`Player.cs:3200-3207`) is ported across these surfaces, all on this branch:

| Surface | C# anchor | Port | Phase |
| --- | --- | --- | --- |
| the model + `Swap` | `EquipmentLoadout.cs:14-80` | `loadout.rs` (`EquipmentLoadout`, `swap`) | Ph2 |
| the two-swap switch + gate + next/prev wrap | `Player.cs:5651-5682` | `player.rs::try_switching_loadout`/`try_switching_to_next/previous_loadout` | Ph2 |
| the `:5663` gate's `pendingItemReuse` fourth member | `Player.cs:4074`, `:42597`/`:42819-42828` | `App.pending_item_reuse` (`world_sim/editing.rs` arm/clear) | dev-4 |
| `GetEffectiveArmor`/`GetEffectiveDye`/`CanShareArmor` | `Player.cs:5690-5764` | `player.rs::get_effective_armor`/`get_effective_dye`/`can_share_armor` | Ph4 |
| the loadout buttons + tint + flash | `Main.cs:41935-41991`, `ItemSlot.cs:2778-2837` | `ui/loadout.rs`, `ui/inventory.rs::equip_tint`/`loadout_flash` | Ph3 |
| the Alt+click share toggle + clear | `ItemSlot.cs:879-912` | `app/mod.rs::toggle_loadout_share`, `app/clicks.rs` cursor-3 arm | Ph4 |
| the `Extra_310-315` share badges | `ItemSlot.cs:2733-2746` | `ui/slot.rs` corner draw + `assets.rs` sheets | dev-1 |
| the `.plr` loadout carrier | `Player.cs:54919-54923`/`:55720-55727` | `player_file.rs` (`current_loadout_index`/`loadouts`) | Ph5 |
| the `.trplayer` v7 loadout block | (the sidecar's own format) | `persist.rs` (index/share line + 3 set lines) | Ph5 |
| net-147 send-on-swap + apply | `Player.cs:5674`, `MessageBuffer.cs:4328-4343` | `app/netglue.rs::send_sync_loadout` + the C2s/S2c arms | dev-2 |
| `clientCloneLoadouts` item-array half | `Player.cs:54522-54531` | `loadout.rs::LoadoutClone` + `App.loadout_clone` | dev-3 |
| **the death loadout sweep** | `Player.cs:54351-54355`, `EquipmentLoadout.cs:82-92` | `death.rs::drop_items` | **THIS ROUND** |
| **the angler reward loadout scan** | `Player.cs:56620-56627` | `world_sim/npcs.rs::give_angler_reward` | **THIS ROUND** |
| **the share-hint disable** | `ItemSlot.cs:886` → `Player.cs:394-401` | `settings.rs::show_loadout_share_hint` + the toggle's turn-ON arm | **THIS ROUND** |

## 2. The loadout-rest census (the dispatch's question: what remained?)

The dispatch was **loadout-rest** — census the remaining loadout arms after
dev-2/3/4. The full C# `Loadout`/`loadout` grep (`Player.cs`, `Main.cs`,
`ItemSlot.cs`, `EquipmentLoadout.cs`, `MessageBuffer.cs`, `NetMessage.cs`,
`ParticleOrchestrator.cs`, `TriggersSet.cs`) yields these arms, each adjudicated:

### 2a. LANDED this round (the unclaimed pure-loadout gameplay arms)

1. **The death loadout sweep** (`Player.cs:54351-54355`): mediumcore/hardcore
   `DropItems(gemsOnly: false)` walks `this.Loadouts` in set order 0/1/2, each
   `EquipmentLoadout.TryDroppingItems` (`EquipmentLoadout.cs:82-92`) dropping
   Armor[20] then Dye[10] through the same `TryDroppingSingleItem`
   (`:54369-54380`). The live set is NOT in `Loadouts` (it rides `App.equip`,
   dropped in the earlier armor/dye walk). `TurnToAir`'s `SetDefaults(0)`
   resets `favorited` (`Item.cs:49047`), so the stored set's share word clears
   with the items. **Port:** `death.rs::drop_items` — a snapshot-then-write-back
   walk (the drop ORDER preserved while `try_drop_single_item` borrows the whole
   App), two `UpdatePlayers` draws per stack BEFORE the starter re-grant's
   `Prefix(-1)` rolls. The base comment "the loadout sweep is unported (no
   loadout system)" is now stale and corrected. Witness
   `w4_g55_mediumcore_death_sweeps_the_stored_loadouts` (set-order drop
   90→1007→954, the slots air, the share word clears, the RNG mirror).
   **Unclaimed at every sibling tip** (c28/d89/g48/g51/f44 all carry the same
   stale "unported" comment — grep-verified).

2. **The angler reward loadout scan** (`Player.cs:56620-56627`):
   `DropAnglerAccByMissing` removes owned accessories from the reward pool over
   the inventory (`:56590-56594`), armor (`:56595-56599`), four banks
   (`:56600-56619`) and the three stored loadouts' **Armor** (`:56620-56627` —
   the Dye arrays are NOT scanned, `:56622`). **Port:**
   `world_sim/npcs.rs::give_angler_reward`'s `owned` collection now extends
   through the armor + banks + the three stored sets' Armor (the base scanned
   only the inventory). The `remove_acc_options` family filter
   (`Player.cs:56646-56683`) is unchanged. Witness
   `w4_g55_the_angler_pool_drops_a_stored_loadout_accessory` (a stored 3721
   removes the boot family 2373/2374/2375, via the `remove_acc_options_for_test`
   seam). The `angler.rs` doc comment + the PORT_MAP angler row's "armor/bank/
   loadout scans stay with their systems" note are corrected.

3. **The share-hint disable** (`ItemSlot.cs:886` → `Player.cs:394-401`):
   `ToggleLoadoutShare`'s turn-ON arm calls `Player.Settings.DisableLoadoutShareHint`,
   which clears `ShowLoadoutShareHint` (static-ctor default true, `:412`) and
   re-saves the config (`Main.SaveSettings`). **Port:** `settings.rs` gains
   `show_loadout_share_hint` (the `"ShowLoadoutShareHint"` config key, `Put`
   `Main.cs:4219`/`Get` `:4553`), and `app/mod.rs::toggle_loadout_share`'s
   turn-ON arm clears it + saves (only when still true). The hint LINE itself
   (`Main.cs:21312-21316`, `UI.ItemLoadoutShareHint`) rides the tooltip lane
   (e16/f28) — the FLAG + its disable are g55's. The existing
   `w4_g55_toggle_loadout_share_flips_and_clears_the_stored_sets` witness is
   extended (the hint disables on turn-ON, stays off on turn-OFF); the settings
   default + round-trip witnesses pin the new key.

### 2b. CLAIMED elsewhere (the merge-time folds, NOT g55 gaps)

4. **net-147 apply-side** — see §4. g51's band (121+) carries the arm, but its
   base predates g55's loadout system, so g51's arm is the stored-not-simulated
   stub; g55's dev-2 already landed the FULL apply (the local swap core +
   visibility unpack). The merger keeps g55's.

5. **The `.plr` loadout transport** — f44's complete walk (research/99 rows
   65-66, the `LoadoutData` transport) is the L1 merge base; g55's
   `EquipmentLoadout` semantics fold onto it (132 §3-L1 rule 2). See §3.

6. **The `.trplayer` loadout block** — g88's v11 superset (research/103 lines
   12-15) already carries g55's block byte-identically. See §3.

7. **The `clientCloneLoadouts` scalar half + the `TrySyncingMyPlayer` array
   diff** (`Main.cs:17557-17575`) — c28's `ClientClone` carries the scalar half;
   g55's dev-3 landed the item-array half (`LoadoutClone`). The array-diff SEND
   (`TrySyncingItemArray` over `Loadouts[i].Armor/Dye`) is c28's send-cadence
   lane (its `net_player_sync.rs` records "the loadout arrays have no port
   loadouts" — stale at g55's tip; the merger wires g55's `loadout_clone` into
   c28's diff). NOT a g55 gap — the clone snapshot is g55's, the diff cadence is
   c28's.

8. **The join-push loadout arrays** (`MessageBuffer.cs:267-272`,
   `NetMessage.cs:2923-2928`): the client join push + the host's `SyncOnePlayer`
   send the three stored sets' Armor/Dye as `5` inventory messages at
   `PlayerItemSlotID.Loadout1_Armor_0..Loadout3_Dye_0`. The slot ledger
   (`id_tables.rs::PLAYER_ITEM_SLOT_WALK`) already carries the six loadout blocks
   (canNetRelay true); g55's join push (`client_send_identity`) sends the 147 +
   the live arrays but NOT the stored-set arrays. **This is the c28/d89
   join-lobby lane's** (the peer-inventory map widening to i16 is c28's — its
   `net.rs` already carries `HashMap<i16, NetItem>`; g55's base is `u8`). The
   stored-set array push folds onto c28's widened map at the merge. NOT a g55
   gap — recorded as a merge-time dependency.

### 2c. RECORDED SKIPS (the visual/system tails, deliberately deferred)

9. **The `ParticleOrchestraType.LoadoutChange` tail** (`Player.cs:5675-5679` →
   `ParticleOrchestrator.Spawn_LoadOutChange`, `ParticleOrchestrator.cs:2370-2399`):
   40 dust-16 + 5 dust-43 (hslToRgb random hue) over the player hitbox, the
   `RequestParticleSpawn(clientOnly: false)` net relay through
   `NetParticlesModule`. The port's particle orchestration is unported
   (`netmodules.rs`'s PARTICLES arm "round-trips and drops"); the dust pool
   (`tr-core/particles.rs`) exists but the orchestra dispatch does not. This is
   the **particle-system lane's** (the `Spawn_LoadOutChange` body is a
   self-contained 30-line dust spawn the particle lane folds). g55's
   `try_switching_loadout` comment records the skip. NOT a g55 gap.

10. **The F1/F2/F3 = Loadout1/2/3 keys** (`PlayerInput.cs:2088-2090` →
    `TriggersSet.cs:805-824`): the vanilla binds stay the port's debug bindings
    (F1 debug overlay, F2 light mode, F3 workbench — the S1-E-29 deviation). The
    `releaseLoadout1/2/3`/`releaseNextLoadout`/`releasePreviousLoadout` fields
    (`Player.cs:1782-1790`) + the `JustBecameTrue` switch arms are the input
    lane's; the swap is reachable through the button UI + the
    `try_switching_loadout` API. Recorded in ANALYSIS. NOT a g55 gap.

11. **The `Extra_310-315` share badges** — dev-1 LANDED them (the dispatch's
    "recorded skip" note in the Ph5 ANALYSIS is stale; the badges are packed +
    drawn). The ANALYSIS deviation bullet is corrected this round.

12. **The `CopyVisuals` loadout half** (`Player.cs:54429-54431` →
    `EquipmentLoadout.CopyVisuals`, `:107-117`): the `Main.playerVisualClone`
    after-image path (`Main.cs:38257`, the First Fractal / terraprisma visual).
    The visual-clone system is unported; this is the render lane's. NOT a g55
    gap.

13. **The `FixLoadedData` loadout half** (`Player.cs:55791-55794` →
    `EquipmentLoadout.FixLoadedData`, `:94-105`: `FixAgainstExploit` per slot +
    `FixLoadedData_EliminiateDuplicateAccessories`): the `.plr` load-time
    sanitisation. f44's complete walk is the L1 base; the loadout
    `FixLoadedData` fold rides f44's `LoadPlayer_LastMinuteFixes` seam
    (`:55859`). The port's `chest.rs::fix_against_exploit` is the shared
    per-slot helper. NOT a g55 gap — the merge-time fold onto f44's walk.

14. **The tooltip share lines** (`Main.cs:20638-20648`
    `UI.ItemLoadoutSharedFrom`/`UI.ItemLoadoutShared`, `:21312-21316`
    `UI.ItemLoadoutShareHint`) + the `MouseHover` shared-item substitution
    (`ItemSlot.cs:3171-3173`): the tooltip lane's (e16/f28). g55 owns the
    `GetEffectiveArmor`/`GetEffectiveDye` scan they read + the hint FLAG (§2a.3).
    NOT a g55 gap.

## 3. The f44-plr compose (the dispatch's question: loadout-block position)

**CONFIRMED — g55's loadout block composes cleanly with f44's plr-walk and the
g88 superset; the g87 dh-sentinel seats AFTER it per 103 §6 rule 5.**

### 3a. The `.plr` (L1, `player_file.rs`)

- **f44's complete walk is the base** (research/132 §3-L1 rule 1, tip
  `d35876af9`): the 71-row field map (research/99), rows 65-66 =
  `CurrentLoadoutIndex` (`:54919`, i32, offset 2880, gate 262, clamp 0..2) +
  `Loadouts[3].Serialize` (`:54920-54923`, 3×(30×10+10)=930 bytes, offset 2884,
  gate 262, fav 322). f44's `LoadoutData` transport
  (`armor: [(i32,i32,u8,bool); 20]`, `dye: [...; 10]`, `hide: [bool; 10]`) is
  byte-identical to g55's `EquipmentLoadout.cs:31-45` walk.
- **g55's fold** (132 §3-L1 rule 2): g55's `current_loadout_index: usize` /
  `loadouts: [EquipmentLoadout; 3]` dedup onto f44's `current_loadout: i32` /
  `loadouts: [LoadoutData; 3]` — same slots, same gates, same clamp. g55's
  swap-machine semantics (`set_shared`/`is_shared`/`item_at`, the
  `FixLoadedData` hooks) ride f44's transport. The byte layout is identical
  (g55's writer emits `(kind, count, prefix, favorited)` per slot then 10 hide
  bools; f44's `LoadoutData` stores the same tuple). The golden 310-zero-byte
  fresh block is preserved on both (g55's `w4_g55_a_fresh_character_loadout_block_is_all_zero`).
- **The loadout-block POSITION**: after the inventory/banks/misc/temp/refunds,
  before the voice pair (`:54924`) — the C# `Serialize` order. f44's walk seats
  it at offset 2880/2884; g55's writer emits it at the same point (after the
  super-cart BitsByte `:54918`, before voice `:54924`). **No position conflict.**

### 3b. The `.trplayer` (L2, `persist.rs`)

- **g88's v11 superset is the base** (research/103 §2, tip `a9f1c29d8`): the
  post-dye block lines 8-16. g55's loadout block is **lines 12-15**:
  - line 12: `currentLoadoutIndex equipShareFlags` (2 tokens, index clamps 0..2)
  - lines 13-15: 3 stored-set lines (41 tokens each: 20 armor kinds + 10 dye
    kinds + 10 hide 0/1 + the 30-bit share word)
- **g55's block is byte-identical in the superset** (verified: g88's writer
  emits `format!("{} {}\n", player.current_loadout_index, equip_share_flags)`
  then per set `20 armor kinds + 10 dye kinds + 10 hide + favorited` — the same
  shape g55's `persist.rs` emits). The superset's v7 reader recognises the block
  by its 41-token set lines ("the two-token line directly BEFORE the first
  41-token set line the loadout index line — g55's block self-identifies by its
  set lines").
- **The g87 dh-sentinel seats AFTER the loadout block** (103 §6 rule 5, 132
  §2e): the `dh <extraAccessory> <unlockedSuperCart>` 3-token sentinel line is
  **line 17 at v12** (the DD2 token-7 version reservation moves to v13). The
  loadout block (lines 12-15) + the torch pair (line 16) precede it. The
  sentinel recognition runs BEFORE the shape walk (a post-dye line whose first
  token is the literal `dh`). **g55's block is unaffected by the dh fold** — it
  sits at lines 12-15 regardless; the dh line appends after the torch pair.
- **The restore** (`app/mod.rs`): g88's superset restore is identical to g55's
  (`p.loadouts = saved.loadouts; p.current_loadout_index = saved.current_loadout_index;
  loaded_share_flags = saved.equip_share_flags`). **No restore conflict.**

### 3c. The compose verdict

The three layers compose without a g55-side change:
- L1: f44's `LoadoutData` transport is the base; g55's `EquipmentLoadout`
  semantics fold onto it (the merger takes f44's walk, re-points g55's
  swap/share/fix hooks). The `.plr` byte layout is identical.
- L2: g88's v11 superset already carries g55's block byte-identically at lines
  12-15; the g87 dh-sentinel appends at line 17 (v12) AFTER the loadout block +
  torch pair, per 103 §6 rule 5. The merger takes g88's `persist.rs` whole
  (132 §3-L2 rule 1); g55's `persist.rs` diff dedups onto it.
- L3: no loadout field rides the world file — the `.trwld`/`.wld` union is
  independent (132 §3-L3).

**The g55 `persist.rs`/`player_file.rs` diffs are merge-time DEDUPS, not
conflicts** — the superset/complete-walk bases already model every loadout byte.

## 4. The net-147 g48/g50-apply census (the dispatch's question: which lane?)

**The 147 apply-side is g51's band (121+), but g55's dev-2 already landed the
FULL apply; the merger keeps g55's.**

The net-apply lanes partition the message-id space:
- **g48** (NET-APPLY-1): ids **1-40** (research/17's FINAL table, SEALED). 147
  is NOT in g48's band. g48's `netglue.rs` carries `C2s::SyncLoadout(_) => {}` /
  `S2c::SyncLoadout(_) => {}` — the empty stubs (its base predates g55).
- **g49** (NET-APPLY-2): ids **41-80**. 147 NOT in band; same empty stubs.
- **g50** (NET-APPLY-3): ids **81-120** (research/271). 147 NOT in band; same
  empty stubs.
- **g51** (NET-APPLY-4): ids **121+** (the BIG apply arms, tip `a8983fac2`).
  **147 IS in g51's band.** g51's `netglue.rs:1014-1025` (C2s) + `:2808-2820`
  (S2c) carry the 147 arm — BUT g51's base predates g55's loadout system, so
  its arm is the **stored-not-simulated stub**: the C2s arm forces the subject +
  stores the peer loadout + relays (no host swap); the S2c arm unpacks ONLY the
  visibility bits onto the local player (no `TrySwitchingLoadout` call — "the
  loadout swap is unported (no loadout system)").
- **g55 dev-2** (this branch, `ff0f2653`): the FULL 147 — the send-on-swap
  (`send_sync_loadout`), the host C2s arm (force subject + store + relay), and
  the client S2c arm that runs the **real swap core** (`p.try_switching_loadout`)
  + unpacks visibility (`netglue.rs:1866-1891`). The join push's 147
  (`MessageBuffer.cs:253`) + `WriteAccessoryVisibility`'s bool[10]→ushort mask
  (`NetMessage.cs:1882-1893`).

**The adjudication:** g51's 147 arm and g55's dev-2 147 arm are the SAME message
in the SAME band overlap (g51's 121+ includes 147; g55's loadout lane owns the
147 semantics). g51's is the stub (its base has no loadout system); g55's is the
full apply (the swap core + visibility). **The merger keeps g55's 147 arms**
(the C2s host arm + the S2c client arm), deduping g51's stubs onto them. This is
the same keep-one pattern as the 34 destroy arm (g52's refinement over g48's
stub, research/272). g55's dev-2 commit message already records "g51 claims the
same arm, take-if-unclaimed" — the census confirms g51's claim is the STUB, so
g55's full apply is the keep-one winner. **No g55-side change needed** — the
apply is landed; the merge dedups g51's stub.

The 147 codec (`net.rs::SyncLoadout`, the `s1_i_1_sync_loadout_round_trips`
witness) is base-trunk (S1-I-1 wave A), shared by all lanes — no conflict.

## 5. The witness ledger (this round)

Two NEW `w4_g55_*` witnesses written not run (code-only):
- `w4_g55_mediumcore_death_sweeps_the_stored_loadouts` (`death.rs`) — the
  set-order drop, the slot air-out, the share-word clear, the RNG mirror.
- `w4_g55_the_angler_pool_drops_a_stored_loadout_accessory` (`world_sim/npcs.rs`)
  — the stored-loadout owned-ids feed the `remove_acc_options` family filter.

Two EXTENDED witnesses:
- `w4_g55_toggle_loadout_share_flips_and_clears_the_stored_sets` (`app/clicks.rs`)
  — the hint-disable on turn-ON, stays off on turn-OFF.
- the settings `defaults_match_the_vanilla_options` + `json_round_trips`
  (`settings.rs`) — the `show_loadout_share_hint` default true + round-trip.

The branch's total `w4_g55_*` count is **29** (27 at `ca1bc92d` + 2 new). The
Ph5 ANALYSIS "19 witnesses" note is the Ph5-phase count; the dev-1/2/3/4 rounds
added 8 (badges 2 + net 3 + clone 1 + reuse 2), and this round adds 2.

## 6. The retire verdict

The loadout family is **COMPLETE for g55's scope**: the model, the swap machine
+ gate (incl. the `pendingItemReuse` fourth member), the effective-item share
scan, the UI (buttons/tint/flash/badges), the share toggle (+ the hint disable),
the `.plr`/`.trplayer` carriers, the net-147 send + apply, the
`clientCloneLoadouts` item-array half, the death sweep, and the angler scan.

The remaining arms are **merge-time folds onto their owning lanes** (§2b: f44's
`.plr` transport, g88's `.trplayer` superset, c28's array-diff cadence + join
push, g51's 147 stub dedup) or **recorded system skips** (§2c: the particle
orchestra, the F1/F2/F3 input binds, the visual-clone, the tooltip lines, the
`FixLoadedData` sanitisation). None is a g55 gap.

**Compose confirmed** (§3): the loadout block seats at `.plr` offset 2880/2884
(f44's walk) and `.trplayer` lines 12-15 (g88's superset); the g87 dh-sentinel
appends at line 17 (v12) AFTER the loadout block + torch pair per 103 §6 rule 5.
**The net-147 apply is g55's keep-one** (§4) over g51's stub.

Code-only retire — no cargo/tests/rustfmt run; the witnesses are WRITTEN NOT
RUN. The golden `.plr` byte-parity is untouched (the fresh loadout block stays
310 zero bytes; the settings key is additive).