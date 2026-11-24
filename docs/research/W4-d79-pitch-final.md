# W4-d79 PITCH FINAL — the 3-way Nebula keep-one, the all-case census, the co-lanes (2026-11-21)

> **PITCH SEAL (2026-11-21, commits `b2c9aa1d2` + the seal docs commit):**
> the lane is SEALED. §S1 is the f49 mutual receipt (the owner-table
> adjudicated against f49 `0d1f941af` §RT2 — AGREED on every canonical
> home, four identical-twin survivorships resolved to f49's side, the
> bridge-ASSIGN-with-deletions one-commit contract restated); §S2 the
> per-lane rewire-table at the CURRENT sibling tips (g21 moved +5, e10
> moved +0-bare, the ledger 43 → **48**); §S3 the case-39 Drip PORTED
> (`b2c9aa1d2` — the F2 in-band gap CLOSED, the census now 43/43);
> §S4 the six out-of-band E21 task-seeds (owner-lanes named); §S5 the
> pitch-family bible + the SEALED banner. The F1-F5 tables below are the
> historical record — where the seal supersedes them, the §S row wins.

The final-phase record of `task/W4-d79-sound-pitch-pool` (base `a725acb04`,
sweep tip `b98130d66`). Code-only mode — witnesses WRITTEN NOT RUN, no
compiler invoked. All sibling tips re-fetched at census time:

| lane | tip at THIS census | note |
| --- | --- | --- |
| W4-f49-mana-regen-2 | `6c0ee2aa2` | R3 contract published, NOT executed (f12 closeout item 1 confirms the three scan arms still live) |
| W4-f12-buff-frame-wiring | `ee67786ec` | moved past the sweep-4 census tip (`61724db38`): the closeout wave publishes the 2-line `fold_orb` bridge hunk |
| W4-d80-tracked-sound-layer | `075b954d2` | unchanged since sweep 4 |
| W4-g21-sounds-main-1 | `7da02f439` | unchanged |
| W4-g22-sounds-main-2 | `250488c34` | moved past sweep 4 (`e0f811358`): the MENU-FINAL/doc-sweep/RETIRE waves; direction-B merger instruction published |
| W4-g23-sounds-main-3 | `945391e33` | unchanged (g22 RETIRE R1 deprecates the branch — g23 folds INTO g22) |
| W4-e10-player-sounds-wave | `432dd5ab1` | moved past sweep 4 (`dc55322a7`): the voice-picker/Hungry/mana-full/stoned/cart/floor-cue wave |

## F1. The Nebula 3-WAY keep-one — per-fn owner-table

Three trees carry a Nebula lifecycle port of the same C# block
(`Player.cs:35045-35058` pickup, `:57030-57059` NebulaLevelup,
`:10444-10461` scan arms + stat folds, `:12070-12079`
UpdateBuffs_NebulaBuffs, `:18755` chained reset, `:19421`/`:19565-19585`
ApplyNebulaBuffMana):

- **d79 (THIS branch)** — the full-arm sweep-2 port: fields + counter on
  `Player`, `nebula_levelup`, the inline `update_buffs_nebula_scan` (+ the
  `update_buffs_nebula` helper) called in `update_with_grapple` before the
  buff-stat loop, the life/damage folds in the stat loops,
  `apply_nebula_buff_mana` as a `Player` method at the `update_survival`
  head, the pickup arm in `world_sim/items.rs` (hardcoded buffType map +
  `audio.play_grab_sound` positioned).
- **f49 `6c0ee2aa2`** — the same full-arm shape with two differences: the
  mana reader lives in `regen.rs::apply_nebula_buff_mana` behind
  `RegenInputs.nebula_level_mana` (the counter is `RegenState::
  nebula_mana_counter`), and the pickup reads `registry::get(kind)
  .buff_type` + calls `audio.play_grab_sound_at`. Its research/85 R3
  (ce71ba61a) is the ACTIVE two-way contract: keep f12's writer, bridge
  the Player field, delete its own scan arms, lift `update_survival`.
- **f12 `ee67786ec`** — only the scan/consumer half: the g27-canonical
  `buff_effects_orbs.rs::apply_orb_arm` owns all 18 orb ids exclusively
  inside `buff_frame_wiring::update_buff_frame` (`is_orb_id` :321 covers
  95-100/170-181); tier derivation writes `Player.orb_scan.
  nebula_level_*` (`OrbScanState`, zeroed by `reset_effects`); `fold_orb`
  (:457) folds the Life/Damage consumers (`buff_life_regen += 6×tier`,
  the 0.15×tier four-class adds). NO pickup arm (its `items.rs` still
  carries the `IsAPickup` skip gate), NO `nebula_levelup`, NO Player
  nebula fields, NO mana reader (its `regen.rs` predates d117). Its
  closeout item 1 (d78e312e0) PUBLISHES the bridge hunk (the field exists
  only on the f49/d79 trees, so it lands with the merge).

### The owner-table (KEEP / DELETE per function)

| C# block | d79 (this branch) | f49 6c0ee2aa2 | f12 ee67786ec | MERGE VERDICT |
| --- | --- | --- | --- | --- |
| Pickup chain arm `:35045-35058` (`world_sim/items.rs:218`) | FULL arm, hardcoded 3453→179/3454→173/3455→176, `play_grab_sound(px.trunc(), py.trunc(), UpdatePlayers)` | same arm, registry `buff_type` read + `play_grab_sound_at` | ABSENT (skip gate) | **KEEP d79's** — the hardcoded map is the `Item.cs:31001-31018` rows verbatim; f49's arm is its duplicate (delete). The `(int)` casts agree (`trunc()` ≡ `as i32 as f32`). |
| Positioned case-7 façade | `audio.rs:2468 play_grab_sound` (works positioned; `play_chest_eat_sound` :2160 carries the same body + `replace_single`) | `audio.rs:1907 play_grab_sound_at` (emit-only, NO single-slot write) | absent | **KEEP d79's `play_grab_sound`** — it is the faithful `:718-722` Stop-then-recreate on the `SoundInstanceGrab` slot; f49's `_at` twin is the D4-era emit-only shape. f49 call sites (chest eat :1895, nebula pickup :305) re-point onto `play_grab_sound`/`play_chest_eat_sound`. |
| `NebulaLevelup` `:57030-57059` | `player.rs:3481 nebula_levelup` | `player.rs:3488 nebula_levelup` (identical logic, `buff_kind` param name) | ABSENT | **KEEP ONE — d79's** (the bodies are line-identical bar the param name; either survives, the merge dedupes). |
| Scan writer `:12070-12079` + `:10444-10461` arms | `player.rs:3519 update_buffs_nebula` + `:3557 update_buffs_nebula_scan`, called `:6578` inside `update_with_grapple` | `player.rs:3529 update_buffs_nebula_buffs` + three inline arms `:6864/:6869/:6874` | `buff_effects_orbs.rs:154 nebula_buffs` via `apply_orb_arm` :288 (the g27 canonical) | **KEEP f12's — DELETE d79's AND f49's.** Three writers on one buff bar would triple-step the `buffTime == 2` edge. The R3 contract (f49 research/85 §R3 step 1) + the g27 keep-one canon both name `apply_orb_arm` the ONE tier writer. |
| Life/Damage stat folds `:10448-10461` | `player.rs:6886` (damage ×0.15 four-class) + `:7113` (life +6 via `accessory_life_regen`) | fields only, no folds (R3 step 2: "never landed here") | `fold_orb` :457 (`buff_life_regen` + the four-class adds inside `apply_orb_arm`) | **KEEP f12's — DELETE d79's two fold arms.** Same C# lines, two homes; the wiring's fold is the canonical consumer (f12 closeout re-sweep afa786057). |
| Mana reader `:19565-19585` @ `:19421` | `player.rs:3533 apply_nebula_buff_mana` (Player method, `Player.nebula_mana_counter`), called `:11325` at the `update_survival` head | `regen.rs:374 apply_nebula_buff_mana` (`RegenState.nebula_mana_counter`, fed `RegenInputs.nebula_level_mana` at `player.rs:11306`) | ABSENT (pre-d117 regen) | **KEEP f49's engine form** — the counter belongs to `RegenState` (the regen machine's cadence half, the d117 anchor row). DELETE d79's Player-method twin + the `Player.nebula_mana_counter` field; the `update_survival` head call re-points at the engine (f49's `RegenInputs` feed line joins the merged inputs struct). Both bodies are the same single-`if` ≥6 grant + tier-0 wipe (`:19583` — NOT ResetEffects). |
| Player fields `:924-930` | `nebula_level_life/mana/damage` + `nebula_mana_counter` (`player.rs:1402-1405`) | `nebula_level_*` trio only (counter in RegenState) | `orb_scan.nebula_level_*` (OrbScanState) | **KEEP the trio on Player** (the levelup grant writes them, `:57047-57056`); the mana one survives because the regen engine reads it (R3: keeping the scan-local twin would strand the reader). DELETE d79's Player counter (moves to RegenState with f49's reader). |
| Chained reset `:18755` | `player.rs:6291-6293` | `player.rs:6157-6159` | `orb_scan.reset_effects()` (`player.rs:6199`) | **KEEP BOTH forms** — they clear different objects (Player fields vs scan state); both are the C#'s per-frame zero (R3 step 3). |
| Pull arm `:34953-34956` (12/5) | landed (sweep 2) | landed (lane 12) | absent | **KEEP ONE — d79's** (identical rows; f49's items.rs is otherwise superseded). |
| 4143 Mana Cloak arm `:35080-35097` | landed (phase 7: RAW-50 text + heat + cap-after-text) | landed (lane 12, same shape) | absent | **KEEP d79's** (R3 step 5 resolves items.rs to the full-arm tree; d79's phase-7 arm carries the text-font sizing convention). |

### The bridge hunk (f12 closeout item 1, verbatim — lands with the merge)

`crates/tr-app/src/buff_frame_wiring.rs::fold_orb` (:457 on f12):

```rust
 fn fold_orb(p: &mut Player, e: &OrbEffects) {
+    // R3 bridge (f49 research/85): the ONE tier writer is `nebula_buffs`
+    // inside apply_orb_arm; the regen engine reads the Player field.
+    p.nebula_level_mana = p.orb_scan.nebula_level_mana; // `:12072` ASSIGN
     p.beetle_buff |= e.beetle_buff;
     ...
```

The ASSIGN (not `+=`) is the C# `:12072` assign-derivation; `OrbScanState`
starts at 0 each tick, so a buff-less tick cannot leave a stale tier. The
Life/Damage Player twins may stay unbridged (nothing reads them once
d79's fold arms are deleted — R3 step 2 permits it). Applying the bridge
WITHOUT deleting the d79/f49 scan arms double-writes the field; deleting
the arms WITHOUT the bridge strands the regen engine at 0. Both halves
in ONE merge commit.

### C#-position-order single-chain assembly (the merged tick)

The C# order inside `Player.Update`: `ResetEffects` (`:25491`, both
zeroes) → `UpdateBuffs` (`:25566` — the scan) → `UpdateEquips` (`:25633`)
→ `UpdateLifeRegen` (`:25977`) → `UpdateManaRegen` (`:25995`, the nebula
mana head `:19421` first) → `GrabItems` (`:27428` — `NebulaLevelup`
increments the bridged field; the NEXT tick's scan re-derives it, so the
pickup's write is a same-tick hint only). The merged port chain, in order:

1. `buffs.update_buffs_tick` — the `:9871-9873` timer head (f12 keeps it
   in `update_with_grapple`; the d79 tree already runs it there).
2. `ResetEffects` — the Player trio zero (d79 `player.rs:6291`) AND
   `orb_scan.reset_effects()` (f12 `player.rs:6199`). Both kept.
3. `UpdateEquips` — the equip pass (unchanged).
4. `update_buff_frame` (f12 `world_sim/player.rs:1230`) — the
   `:9857-9864` frame head; the arm chain routes 173-181 to
   `apply_orb_arm` at the `:10444-10461` position (Mana → Life → Damage
   per slot, the `:12046-12067` tails); `fold_orb` folds the consumers +
   the mana bridge ASSIGN. DELETE the d79 `update_buffs_nebula_scan()`
   call at `player.rs:6578` (and the fn + helper) and the two stat-fold
   arms (`:6886`/`:7113`).
5. `update_survival` — **LIFTED** out of `update_with_grapple` (d79
   `player.rs:7786`) to the world-sim seam AFTER `update_buff_frame`
   (before the accessory pass is fine — the C# regen reads sit after
   UpdateEquips too). Without the lift the regen reads the PREVIOUS
   tick's tier (the reset zeroes it at the head). The lift also repairs
   f12's own `buff_life_regen` feed (its fold lands after the regen read
   and the next reset wipes it before any read — R3 step 4). The
   standalone `Player::update` wrapper re-adds the `update_survival`
   call AFTER `update_with_grapple` so the no-wiring callers (equip
   tests, potion rigs) keep their regen tick (tier stays 0 there — the
   wiring owns the scan, the f12 gap-3 convention). The nebula mana head
   runs FIRST inside the lifted call (f49's engine form).
6. projectiles phase → items phase — the d79 pickup arm's
   `nebula_levelup` increments the bridged Player field (same-tick hint).

### Witness re-targets (the merge's test surgery)

- d79 `w4_d79_the_nebula_pickup_levels_the_matching_buff_family` /
  `w4_d79_the_nebula_pickup_draws_one_pitch_and_no_text`
  (`world_sim/items.rs:2496/:2545`) — KEEP as-is (the pickup arm and the
  Player-field grant survive; the one-pitch order twin is stream-exact).
- f49's four lane-12 witnesses — the levelup/reset twins keep their
  Player-field assertions (the bridge keeps the fields live); the two
  scan-driving twins move to `update_buff_frame` (assert `orb_scan` +
  the bridged field), per R3 step 1. f49's
  `w4_f49_the_star_pickup_plays_the_unpositioned_grab_chime` keeps its
  count/pitch pins (the `play_pickup_sound` façade is shared).
- d79's `accessory_life_regen` stat-loop witnesses that add buffs
  173-175 must re-target to the wiring's `buff_life_regen` fold (none of
  the current d79 rigs grant nebula buffs through `Player::update` — the
  census at sweep 2 verified the folds only fire with a live nebula buff).
- The `anchors.txt` rows: d79 :903 (`fn nebula_levelup` — KEPT, marker
  survives), :904 (the pickup arm — KEPT); f49's :904-equivalent
  (`pub(crate) fn nebula_levelup` row) rewords its prose to cite
  `buff_effects_orbs.rs::nebula_buffs` as the tier-writer's home (never
  delete the row); the d117 `regen.rs|fn apply_nebula_buff_mana` row
  survives with f49's engine form.

## F2. The LegacySoundPlayer ALL-CASE census — FINAL

The C# `LegacySoundPlayer.PlaySound` switch surface
(`LegacySoundPlayer.cs:438`/`:1092`/`:1247` + the nested style switches):
cases **0-42** plus the out-of-band 43/44/45/46/47-52/53-62/63/64/65/66/
67/68/69. Script-diff of the C# case list vs the port façades at d79 HEAD
(`b98130d66` + this phase's witness):

| case | C# rows | port home (d79 HEAD) | verdict |
| --- | --- | --- | --- |
| 0 Dig | `:440-452` Next(3) + Next(-10,11) | `play_no_space_sound` (audio.rs:1274) + `dig_slots[3]`; the kill-table path pins Dig_1 (`editing.rs:5527`) | PORTED |
| 1 PlayerHit | `:503-515` Next(3), no pitch | `play_hurt_sound` (:1961) + `player_hit_case1` + `player_hit_slots[3]` | PORTED |
| 2 Item | `:516-644` style windows/guards | `play_use_sound` (:1160) / `play_proj_sound` (:2249) / `play_npc_item_cue` (:1798) + `item_use_case2`/`item_case2_pool` + `item_slots` map | PORTED |
| 3 NPCHit | `:646-672` | `play_npc_hit_sound`/`play_npc_hit_int` (:1390/:1417) + `npc_hit_case3` | PORTED |
| 4 NPCDeath | `:673-695` | `play_npc_death_sound`/`_style`/`_int` (:1492-1540) + `npc_killed_case4` | PORTED |
| 5 PlayerKilled | `:696-705` no draws | `play_death_sound` (:1997), SINGLE_SLOT row | PORTED |
| 6 Grass | `:706-716` Next(-30,31) | `play_step_sound` grass arm (:1913) + `legacy_family_pitch` | PORTED |
| 7 Grab | `:717-727` Next(-10,11) | `play_grab_sound` (:2468) / `play_pickup_sound` (:2107) / `play_chest_eat_sound` (:2160) | PORTED |
| 8/9 Door | `:728-749` Next(-10,11) | `play_door_sound` (:2346) | PORTED |
| 10/11/12 Menu | `:750-782` no draws | `play_menu` (:2452) → `play_single_voice` (:2426) | PORTED |
| 13 Shatter | `:783-792` no draws | `play_wall_break_sound` (:2291) shatter arm | PORTED |
| 14 Zombie moan | `:793-825` slot draw, no pitch | `play_fighter_moan` (:1051) → `critter_legacy_voice` 14 arm | PORTED |
| 15 Roar | `:826-842` guard, no draws | `play_roar` (:1615) + `roar_case15` | PORTED |
| 16 DoubleJump | `:843-853` | `play_double_jump_sound` (:2021) | PORTED |
| 17 Run | `:854-864` | `play_step_sound` run arm | PORTED |
| 18 Coins | `:865-870` no draws, no Stop | `play_single_voice("Coins")` (pool false flag) | PORTED |
| 19 Splash | `:871-910` guard + windows | `play_splash_sound` (:2054) + `splash_case19` + `splash_slots[6]`; the liquid-merge shimmer arm (:1090) | PORTED |
| **20 FemaleHit** | `:911-923` Next(3), no pitch | **NOT on d79** (the `player.rs:4579` comment records variant-1-only) — **e10 `432dd5ab1` ports it**: `Female_Hit_0..2` bank rows + `play_female_hurt_sound` (e10 audio.rs:1822) + the `hurt_voice` C#-priority picker | **GAP → e10's lane** (keep e10's) |
| 21 Tink | `:924-936` Next(3), no pitch | `play_tink_sound` (:2492) + `tink_slots[3]` | PORTED |
| 22 Unlock | `:937-946` no draws | `play_single_voice("Unlock")` (positioned callers pass x/y) | PORTED |
| 23 Drown | `:947-956` no draws | `play_drown_sound` (:2221) | PORTED |
| 24 Chat | `:957-962` no draws, no Stop | `play_single_voice("Chat")` — 3 seams wired (talk :43083, Angler :40243, DyeTrader :40016) | PORTED |
| 25 MaxMana | `:963-968` no draws, no Stop | `play_single_voice("MaxMana")` + the phase-6 refill drain | PORTED (e10's `play_max_mana_sound` twin is the merge-dedupe — see F4) |
| 26 Zombie | `:969-978` Next(3,5)+pitch | `play_fighter_moan` 26 arm | PORTED |
| 27 Pixie | `:979-996` live-instance re-draw | `play_pixie` (:1686) + dedicated `pixie_slot` (OUT of the draw-free pool) | PORTED (orphaned emitter, E21 class) |
| 28 Mech | `:997-1007` guard + pitch | `play_switch_click` (:2390) + `mech_case28` + `mech_slots` | PORTED |
| 29 Zombie style-slot | `:1008-1050` | `play_fighter_moan` 29 arm (the per-style volume table) | PORTED |
| 30/31/32/33 Critters | `:1102-1143`/`:1200-1211` | `critter_legacy_voice` 30/31/32/33 arms + `play_critter` (:1000) + `zombie_playing` guards | PORTED |
| 34/35 Liquid loops | `:1249-1307` | `set_liquid_loop` (:2565) — the persistent Volume/Pan channels | PORTED |
| 36 DashRoar | `:1309-1325` no guard/draws | `play_dash_roar` (:1651) + `roar_case36` | PORTED |
| **37 Star hit** | `:1326-1336` Next(57,59)+pitch | **NOT on d79** — **g23 `945391e33` ports it**: `play_star_hit_pair` (g23 audio.rs:2011), Item_57/58 bank rows under id 37, the case38-then-37 order (`Main.cs:63168-63169`); dormant (no falling-star consumer) | **GAP → g23's lane** (keep g23's, folds via g22 direction-B) |
| 38 Coin pickup | `:1337-1350` Next(5)+Next(-40,41)*0.002 | `play_pickup_sound(true)` / `play_coin_ping` (:2185) + `coin_pickup_case38` + `coin_slots[5]` no-Stop | PORTED |
| **39 Drip** | `:1351-1358` Style-slot + Next(-30,31) | **NOT ported** — the gore pool's drip sound is a DOCUMENTED drop (`tr-core/gore.rs:627-630`: "the drip's sound and ripple (`:953`, `:965-968`)"); the C# draws `Main.rand.Next(2)` INSIDE the call (`Gore.cs:953`) — the drop note's "none of them draws" is wrong for the tile-hit arm | **GAP — documented, E21-class deferred** (the gore pool has no sound seam; the wet arm `:965` is draw-free, the tile arm draws one Next(2) on the gore stream) |
| 40 Camera | `:1359-1368` no draws | `play_single_voice("Camera")` | PORTED |
| 41 MoonlordCry | `:1369-1375` distance volume + pitch | `play_moonlord_cry` (:1729) + dedicated `moonlord_cry_slot` (sweep 3) | PORTED (orphaned emitter) |
| 42 Trackable | `:1376-1381` no gameplay draws | `play_tracked`/`play_tracked_global`/`play_tracked_looped` (:2608-2639) + `tracked_sound.rs`; the style façades' NextFloat rides the audio-only stream | PORTED |
| 43 Thunder | `:454-479` re-roll loop + pitch | `thunder_cue_pick` (ambient_sound.rs:310) + `play_thunder` (:956) + `thunder_slots[6]` | PORTED |
| 44/45/46 Critters | `:1051-1091` | `critter_legacy_voice` 44/45/46 arms (45 table-complete, no duck kind) | PORTED |
| **47-52 Guitar chords** | `:1212-1232` slots 133-138, family Stop sweep, no pitch | **NOT ported** — `Player.ItemCheck_PlayInstruments` (`Player.cs:46506`) / `PlayGuitarChord` (`:46671-46702`) unported (the Sparkle Guitar's NOTE projectile is i22's, the chord cues are a different block) | **GAP — E21-class deferred** (no instrument-use seam; the six chord ids + the 1-in-10 bad-luck pitch `Next(-50,51)` ride `Main.rand` at the use site) |
| **53-62 Drum family** | `:1234-1245` slots 139-148, Stop, ×0.7, no pitch | **NOT ported** — `PlayDrums` (`Player.cs:46704+`) unported, same instrument block | **GAP — E21-class deferred** (same seam as 47-52) |
| **63/64 Research** | `:480-502` Next(1,4)/slot 0, Stop, no draws | **NOT ported** — the Journey sacrifice UI arm (`ItemSlot.cs:1293/:1296`) is unported (`clicks.rs:1818` records "the Journey research arm `:940-943`" deferred); `creative_sacrifices.rs` is the DATA table, no sacrifice-click seam | **GAP — E21-class deferred** (draw-free when it lands: 63 draws one Next(1,4), 64 none) |
| **65 QueenSlime ambient** | `:1382-1403` 3-slot guard, Next(115,118), NO pitch | **NOT ported** — the caller `NPC.cs:45759` (`AI_121_QueenSlime`, the `Next(360) == 0` ambient roll) has no ported NPC AI body (the Queen Slime NPC is spawn/data-only on this tree) | **GAP — E21-class deferred** (the roll rides UpdateNPCs; the case itself draws one slot pick, no pitch) |
| **66 Cursed-flame ambient** | `:1180-1199` 3-slot guard, Next(121,124), Next(-15,16) | **NOT ported** — the caller `Projectile.cs:46941` is the type-75 arm inside `AI_016_Bombs`; `projectile.rs:314` records "147/75/10/181 stay unported stand-ins" | **GAP — E21-class deferred** (two draws when the type-75 body lands) |
| 67/68/69 Critters | `:1144-1179` | `critter_legacy_voice` 67/68/69 arms | PORTED |

**Verdict: all 43 in-band cases (0-42) accounted for.** 41 ported on the
d79 HEAD or its sibling lanes (20 → e10, 37 → g23); **one in-band gap
(39 Drip)** — a documented gore-pool drop, deferred with its E21-class
note (the tile-hit arm's Next(2) draw must join the gore stream when the
sound seam lands). Out-of-band: 43/44/45/46/67/68/69 ported; **six
out-of-band gaps (47-52, 53-62, 63, 64, 65, 66)** — every one an
unported-CALLER deferral (instruments, Journey sacrifice UI, Queen Slime
AI, type-75 bomb arm), NOT a bare-play regression: none has a ported
emitter playing through the retired `audio.play()` path. The 19-field
single-instance census stays closed (17 SINGLE_SLOT_CUES rows +
`pixie_slot` + `moonlord_cry_slot`, sweep 3). The bank-id pins are
witnessed (`w4_d79_final_the_case_bank_pins_cover_the_in_band_surface`,
written not run).

**Bare-play re-census (this phase, per-tip `git grep` + the brace-depth
`#[cfg(test)]`-module classification, every ambiguous site hand-verified
— the exact figures supersede sweep-4's filter):**

| tip | prod sites | test-rig sites | total | Δ prod vs base |
| --- | --- | --- | --- | --- |
| base `a725acb04` | 88 | 2 | 90 | — |
| **d79 HEAD** | **0** | **0** | **0** | **−88** |
| g21 `7da02f439` | 109 | 2 | 111 | +21 |
| g22 `250488c34` | 96 | 5 | 101 | +8 net (+9 prod added, −1 blanket tick; +3 test rigs) |
| g23 `945391e33` | 92 | 2 | 94 | +4 net (+6 added, −2) |
| e10 `432dd5ab1` | 91 | 2 | 93 | +3 net (+10 prod menu added, −6 editing.rs Roar, −1 doors.rs Roar; the cart drain's `play_at` is a REWRITE of the base site, not an addition — the 7th Roar removal) |
| d80 `075b954d2` | 88 | 2 | 90 | 0 |
| f49 `6c0ee2aa2` | 88 | 2 | 90 | 0 |
| f12 `ee67786ec` | 89 | 2 | 91 | +1 (the `play("hungry")` drain, F4) |

The base's 2 test-rig sites are the `app/flow.rs` Bug-17 hover witness's
two plays; the 6 `editing.rs` Roar plays at :16041-16558 are PROD
(the summon-item helpers live after the tests module — the naive
first-`mod tests` split miscounts them). g22's +3 test rigs are the
doc-sweep witnesses (clicks.rs:3372, flow.rs:1789/:1967 — the latter two
join the base Bug-17 pair's file).

d79 HEAD has **0** `audio.play(`/`play_at(` sites — the only `.play()`
left tree-wide is `tracked_sound.rs:191` `sink.play()` (the
`ActiveSound.Resume` seam, `ActiveSound.cs:117-125` — NOT a bare SFX
play). The sibling deltas are F4's rewrite-table.

## F3. The d80 co-lane — tracked loops × the Resume-only-bare verdict

d80 `075b954d2` (6 phases) adds the tracked-sound ENTITY LOOPS on the
same `tracked_sound.rs` layer d79's case-42 façades ride:

- `TrackedStyle::by_name` (d80 tracked_sound.rs:+65) — resolves a
  `SoundID` field name through the generated `SOUND_STYLES` table
  (`soundid.rs`, the `SoundID.cs` static-ctor port); every d80 style is a
  plain `CreateTrackable` (`SoundID.cs:1205-1213`, uncapped).
- The `test_keep_alive` seam (`#[cfg(test)]`) — keeps headless slots
  addressable across pool updates (the ensure/follow witnesses).
- Entity loops: drill/chainsaw aiStyle-20 cadence (Item189/190, the
  20/15-tick `soundDelay`), Laser Drill 445 (Item132), the DD2 NPC loops
  (Betsy wind/breath, portal open/idle, kobold ignite/explode, wither
  aura — `NpcSoundCue` tracked arms + `local_ai` SlotId write-backs),
  the bomb fuse (`ProjTrackedCue` queue/drain + the shared-atomic
  `ProjectileAudioTracker` condition fold), the vampire sun-scorch
  sizzle (`VampireSizzleTracker`).

**Compose-note (the merge contract):**

1. **Zero conflict on the bare-play surface.** d80 adds NO
   `audio.play(`/`play_at(` sites (its tip count equals the base's 90).
   Its tracked plays ride `play_tracked_looped`/`play_tracked` — the
   d79 case-42 façades — and its drill/chainsaw one-shots ride
   `play_proj_sound` (the case-2 style path, d79's façade). The d80
   `audio.rs` diff vs d79 HEAD is the ABSENCE of d79's façade wave (d80
   branched from the base), not a competing rewrite: the merge keeps
   d79's audio.rs wholesale and adds d80's bank rows (`Item_132/189/190`
   under id 2, the DD2/fuse/sizzle custom cues) + the three test seams.
2. **The `ActiveSound.Resume` sink.play() stays.** d79's sweep-4 verdict
   ("only the tracked_sound.rs sink.play() stays") and d80's layer are
   the SAME seam: `resume()` (`:117-125`) un-pauses a live tracked
   instance — it never creates a voice, draws no RNG, and is not a
   `PlaySound` case. d80's `test_keep_alive` extends `is_playing`'s test
   arm only; the production `resume` body is untouched by both lanes.
3. **RNG contract.** d80's research/64 census: zero `Main.rand` draws on
   the tracked path (the variation pick rides the engine's own
   `LegacySoundStyle.Random`, the port's `style_random`). The d79
   pitch/pool contract holds: no new gameplay-stream draws from the
   tracked lane; the cadence counters (`soundDelay`, `local_ai`) are
   gameplay state, not RNG.
4. **Ordering.** The d80 drains write SlotId floats back into
   `Npc.local_ai`/projectile slots at the same pass positions the C#
   reads them; the d79 façade wave did not move those passes. The
   vampire sizzle's `update_dead`/`UpdateSunScorch` arms ride the player
   pass — after the d79 mana-refill drain (`:25995` < `:28580`), no
   interleave conflict.

## F4. The sibling-rewire hunk-list — exact per-site rewrite-table

Every NEW bare `audio.play(`/`play_at(` site on the sibling tips (the
per-file `git diff a725acb04 <tip>` added-line census, this phase) and
its d79-façade rewrite. Line numbers are the Sibling tip's; the merger
locates the hunk by the cited C# anchor, not the line. All menu rewrites
are **draw-free** (cases 10/11/12 draw nothing — the rewire changes the
instance-pool behaviour, not the RNG); the Chat rewrites are draw-free
too. `play_menu` = `audio.rs:2452`; `play_single_voice` = `:2426`;
`chest::play_menu_sound(app, id)` = the id-keyed wrapper (`chest.rs:965`).

### g21 `7da02f439` — +21 (19 menu + 2 Chat)

| site (g21 tip) | C# anchor | bare call | REWRITE |
| --- | --- | --- | --- |
| clicks.rs:1142 | `Main.cs:42458` bestiary icon | `play("Menu_Tick")` | `play_menu("Menu_Tick")` |
| clicks.rs:1175 | `:42403` emotes wheel | `play("Menu_Tick")` | `play_menu("Menu_Tick")` |
| clicks.rs:1314 | `:40828` | `play("Menu_Tick")` | `play_menu("Menu_Tick")` |
| clicks.rs:1596 | `:41100` map icon open | `play("Menu_Open")` | `play_menu("Menu_Open")` |
| clicks.rs:1605 | `:41109/:41114/:41119` mapStyle | `play("Menu_Tick")` | `play_menu("Menu_Tick")` |
| clicks.rs:1904 | `:41249/:41275` hideMisc/superCart | `play("Menu_Tick")` | `play_menu("Menu_Tick")` |
| clicks.rs:2660 | `:41849` quick-stack transfer | `play("Menu_Tick")` | `play_menu("Menu_Tick")` |
| clicks.rs:2738 | `:41855` | `play("Menu_Open")` | `play_menu("Menu_Open")` |
| clicks.rs:2796 | `:41887` sort | `play("Menu_Tick")` | `play_menu("Menu_Tick")` |
| loop.rs:671 | `:18370` chat close | `play("Menu_Close")` | `play_menu("Menu_Close")` |
| loop.rs:686 | `:18347-18350` chat change tick | `play("Menu_Tick")` | `play_menu("Menu_Tick")` |
| loop.rs:702 | `:18337` append change tick | `play("Menu_Tick")` | `play_menu("Menu_Tick")` |
| loop.rs:712 | `:18270` chat open | `play("Menu_Open")` | `play_menu("Menu_Open")` |
| loop.rs:743 | `:18416` F11 UI toggle | `play("Menu_Tick")` | `play_menu("Menu_Tick")` |
| loop.rs:754 | `:18432` F7 graphics diag | `play("Menu_Tick")` | `play_menu("Menu_Tick")` |
| loop.rs:778 | `:18460` F8 net diag | `play("Menu_Tick")` | `play_menu("Menu_Tick")` |
| loop.rs:793 | `:18498` F10 FPS | `play("Menu_Tick")` | `play_menu("Menu_Tick")` |
| loop.rs:848 | `:18481` F9 lighting cycle | `play("Menu_Tick")` | `play_menu("Menu_Tick")` |
| ui/mod.rs:892 | `:3244` chest-button hover ticks | `chest_panel.audio.play("Menu_Tick")` | `chest_panel.audio.play_menu("Menu_Tick")` (the FrontDraw audio borrow — same façade) |
| world_sim/npcs.rs:287 | `:43083` talk open | `play("Chat")` | **DELETE — d79 phase 7 already wires this seam** through `play_single_voice("Chat", -1, -1)` (`talk_to_npc`) |
| world_sim/npcs.rs:645 | `:40243` Angler turn-in | `play("Chat")` | **DELETE — d79's `angler_turn_in` seam wins** (same pool path) |

Non-new g21 sites that CONFLICT with d79 rewires (keep d79's side): the
`world_sim/npcs.rs:1953` `play_at("Roar_0", …)` Destroyer roar — d79
`:2072` plays `play_roar(1, …)`: the C# `NPC.cs:50093`
`PlaySound(15, x, y)` takes the int overload's **default Style = 1**
(`SoundEngine.cs:102`), so `Roar_1` is faithful and g21's bare `Roar_0`
is wrong twice (no guard, wrong slot). The `:4733` kill-table
`play_at(name, …)` — d79's `play_kill_cue` path wins (the per-case
draws). The g21 Grab/Coins click sites duplicate d79's phase-3/5
rewires one-for-one (keep d79's `play_grab_sound(-1,-1, rngs.global())`
/ `play_single_voice` forms).

### g22 `250488c34` — +12 raw (+9 prod, +3 test; the RETIRE tip supersedes e0f811358's +3)

| site (g22 tip) | C# anchor | bare call | REWRITE |
| --- | --- | --- | --- |
| clicks.rs:2832 | `:48843` corrupt-world Load Backup | `play("Menu_Open")` | `play_menu("Menu_Open")` |
| clicks.rs:2843 | `:48849` backup cancel | `play("Menu_Close")` | `play_menu("Menu_Close")` |
| clicks.rs:3372 | test rig (`doc_sweep_cancel_connect…`) | `play(cue.unwrap())` | `play_menu(cue.unwrap())` (test-only) |
| flow.rs:466 | HSL slider drag cues | `d.audio.play("Menu_Tick")` | `d.audio.play_menu("Menu_Tick")` (FrontDraw seam) |
| flow.rs:1067 | `UIVirtualKeyboard.cs:252/:286/:525/:499/:394` on-screen keys | `play("Menu_Tick")` | `play_menu("Menu_Tick")` |
| flow.rs:1789 | test rig (`doc_sweep_creator_hover_edge…`) | `play("Menu_Tick")` | `play_menu("Menu_Tick")` (test-only) |
| flow.rs:1967 | test rig (card-body enter) | `play("Menu_Tick")` | `play_menu("Menu_Tick")` (test-only) |
| frame.rs:209 | `IngameOptions.cs:157-199/:421-436` update drain | `play(cue)` | `play_menu(cue)` — the cue is always a menu name (the ingame_options::update contract) |
| loop.rs:278 | `:50794-50797` text-edit change | `play(cue)` (m.take_cue) | `play_menu(cue)` |
| loop.rs:294 | key-arm per-site cue | `play(cue)` (m.take_cue) | `play_menu(cue)` |
| loop.rs:481 | `:48489-49727` activation drain | `play(cue)` (m.take_cue) | `play_menu(cue)` |
| mod.rs:2596 | `UIWorldCreation.cs:834` seed-option Chat | `play("Chat")` | `play_single_voice("Chat", -1.0, -1.0)` — the front-end site d79 phase 7 left out of the sim census; the pool path is the same |

The g22 `frame.rs:174` `IngameOptions.update` drain is pre-existing on
the g22 line (the sweep-4 +1) — same rewrite (`play_menu(cue)`).
**g22↔g23 compose:** g22 RETIRE R3's direction-B instruction stands
(g22 base + the 4-commit g23 merge, keep-g22/take-g23-unique, the
disjoint-queue contract `pending_cue` activation / `pending_ticks`
draw-chrome / `FrontDraw.audio` front cues). The d79 rewire applies to
the UNION's drain sites — every `play(cue)`/`play(match id …)` in the
merged tree becomes `play_menu(…)` (the id-keyed arms map 10/11/12 →
the cue name first, or route through `chest::play_menu_sound`).

### g23 `945391e33` — +6 raw (4 unique after the g22 union)

| site (g23 tip) | C# anchor | bare call | REWRITE |
| --- | --- | --- | --- |
| frame/draw.rs:1843 | `:51276/:51575` draw-chrome drain | `play(match id {10/11/_})` | `play_menu(match id {…})` (the pending_ticks chrome queue) |
| chat.rs:117 | `:64635/:64641` NewText tail | `play(match id {…})` | `play_menu(match id {…})` (drain_new_text_cues) |
| frame.rs:209, loop.rs:278/:294/:481 | shared with g22 (the union) | `play(cue)` | same as g22's rows — ONE rewrite after the direction-B merge |

### e10 `432dd5ab1` — +10 prod menu, −7 Roar (6 editing + 1 doors), the cart rewrite (the wave past dc55322a7)

| site (e10 tip) | C# anchor | bare call | REWRITE |
| --- | --- | --- | --- |
| flow.rs:493 | `UICharacterCreation.cs:1517-1527` voice pitch bar | `d.audio.play("Menu_Tick")` | `d.audio.play_menu("Menu_Tick")` |
| flow.rs:712 | `:1288` voice-cycle | `play("Menu_Tick")` | `play_menu("Menu_Tick")` |
| flow.rs:743 | `:1042` voice pitch click | `play("Menu_Tick")` | `play_menu("Menu_Tick")` |
| flow.rs:802 | `:1547` randomize player | `play("Menu_Tick")` | `play_menu("Menu_Tick")` |
| loop.rs:711 | `Player.cs:4976` ToggleInv map-close | `play("Menu_Close")` | `play_menu("Menu_Close")` |
| loop.rs:720 | `:5010` ToggleInv talk-close | `play("Menu_Close")` | `play_menu("Menu_Close")` |
| loop.rs:727 | `:5056/:5087` ToggleInv toggle | `play(if open {…})` | `play_menu(if …)` |
| loop.rs:766 | `:24973/:29111` fullscreen map | `play(if open {…})` | `play_menu(if …)` |
| loop.rs:779 | `:24953` map-style cycle | `play("Menu_Tick")` | `play_menu("Menu_Tick")` |
| loop.rs:830 | `:5087/:5056` rebound cInv | `play(if open {…})` | `play_menu(if …)` |
| world_sim/player.rs:1371 | cart drain fallback arm | `play_at(cue.cue, …)` | **DELETE the fallback** — d79's drain (`world_sim/player.rs:1364`) matches Item_53/55/56 → `play_npc_item_cue` and `_ => continue`s (the queue only ever carries those three; e10's `play_proj_sound` routing is the same case-2 path — keep ONE, d79's `play_npc_item_cue` form) |

The e10 Roar rewires (−7: six `editing.rs` summon-item sites + the
`doors.rs` golem-altar one) land on `play_roar(0, plx, ply)` — the SAME
façade d79's phase-3 rewire used (`editing.rs:16059…16591`,
`doors.rs:58`). **Keep-one: identical target, either side's hunk
survives; d79's positions are the C#-order-verified ones (the roar
BEFORE ApplyItemTime / SpawnOnPlayer).** The e10 tip also carries the
loop.rs:779 smart-cursor tick (`:28818`) inside the same hunk window —
same `play_menu` rewrite.

### e10's NON-bare duplicates (the façade keep-one rulings)

- **MaxMana (case 25):** e10's `play_max_mana_sound` (audio.rs:2112,
  emit-only) + `take_mana_full` drain + the `EmitMaxManaEffect`
  mana-blade expiry arm (`editing.rs:1807`) vs d79's
  `play_single_voice("MaxMana")` pool path + `pending_mana_refill`
  drain. **KEEP d79's pool path** (the no-Stop single-slot overwrite is
  the C# `:964`; e10's emit-only twin stacks voices). **KEEP e10's
  mana-blade expiry PRODUCER** (`Player.cs:42837-42844` → `:49798-49808`
  — a seam d79 does not have): re-point its play at
  `play_single_voice("MaxMana", -1, -1)`; the 5×dust-45 burst body is
  identical on both lanes (one Next(20,26) scale before each NewDust).
  The full-bar refill producer is ONE queue (d79's `pending_mana_refill`
  carries the live box; e10's `take_mana_full` bool is its duplicate —
  delete e10's queue + drain, keep d79's).
- **Hungry (case 42):** e10's `play_voice_style("Hungry", -1, -1, 0, rng)`
  on the three drink callers (`potions.rs:454/:767/:850`, the 333-expiry
  flag from `apply_potion_delay`/`try_to_reset_hunger_to_neutral`) vs
  f12's `pending_hungry_sounds` counter drained as `play("hungry")`
  (world_sim/player.rs:1242, fed by the wiring's 333-arm
  `buff_frame_wiring.rs:314`). **The C# plays at TWO sites**: the
  UpdateBuffs 333-expiry (`Player.cs:12138` — f12's wiring arm) AND the
  drink-path `TryToResetHungerToNeutral` → `UpdateHungerBuffs` re-entry
  (`:5524`/`:51873`/`:51879` — e10's callers). KEEP BOTH producers;
  **the play is ONE façade**: `play_voice_style("Hungry", …)` (e10's —
  the style path draws the NextFloat on the audio-only stream, zero
  gameplay draws, the case-42 contract). f12's drain re-points:
  `for _ in 0..p.pending_hungry_sounds { app.audio.play_voice_style("Hungry", -1.0, -1.0, 0.0, rng); }`.
  The bare `play("hungry")` bypasses the tracked-style volume/variance —
  delete it.
- **The voice picker (case 20 + the hurt family):** e10's `hurt_voice`
  C#-priority chain + `play_female_hurt_sound`/`play_hurt_npc_int`/
  `play_voice_style`/`play_no_space_sound` (stoned) — all d79-compatible
  façades (the stoned arm rides d79's case-0 `play_no_space_sound`).
  KEEP e10's wholesale; d79's `pending_hurt_sound = Some("Player_Hit_0")`
  default-variant stand-in (`player.rs:4582`) is superseded by e10's
  picker (the merge keeps e10's drain).

### The added-bare ledger (the merger's checklist)

Raw added sites: g21 +21, g22 +12, g23 +6, e10 +10 (the cart `play_at`
is a rewrite of the base site, not an addition) = **49**. Dedupe:
−4 (the g23 rows shared with g22's drains: frame.rs:209,
loop.rs:278/:294/:481) = **45 unique added sites**; of those, −2 (the
g21 Chat pair — d79's phase-7 seams already play them, DELETE not
rewire) = **43 rewire targets** (40 prod + 3 g22 doc-sweep test rigs).
The 2 base Bug-17 flow.rs test plays are NOT additions (they ride every
tip; d79's HEAD retired them with the hover path — the merged tree
rewires them too if the sibling hover hunk survives). Every added site
is a case-10/11/12 menu cue or the case-24 Chat trio. ZERO new
drawing-case bare plays on any sibling tip (the pitch/pool contract
holds tree-wide). After the rewrites above, the merged tree's bare-play
count returns to **0** (the `tracked_sound.rs:191` Resume seam
excepted).

## F5. Research FINAL — the residual register

1. **The three-way Nebula merge is a SINGLE commit** (F1): keep f12's
   writer + bridge, delete the d79/f49 scan twins + d79's fold arms +
   d79's Player-counter reader, keep d79's pickup/levelup/reset/pull,
   keep f49's regen-engine reader, lift `update_survival`. The bridge
   hunk and the deletions MUST land together (either half alone is a
   behaviour bug).
2. **The in-band case surface is closed** (F2): 0-42 all accounted;
   the only in-band gap (39 Drip) is a documented gore-pool drop with
   its RNG caveat corrected here (the tile-hit arm DOES draw one
   Next(2)). The six out-of-band gaps are unported-caller E21 deferrals
   — table-complete specs recorded in F2's rows for the future seams.
3. **The d80 co-lane composes clean** (F3): no bare-play overlap, the
   Resume seam is shared, the tracked RNG contract holds.
4. **The sibling rewire is mechanical** (F4): every added bare site is
   a draw-free pool cue; the rewrite-table is the merger's exact
   checklist. The three façade keep-one rulings (MaxMana, Hungry, the
   voice picker) are the only semantic merges in the sound family.
5. **Witnesses (WRITTEN NOT RUN):**
   `w4_d79_final_the_case_bank_pins_cover_the_in_band_surface`
   (audio.rs — the bank-id pins for every ported in-band case + the
   17-row pool table + the two dedicated slots),
   `w4_d79_final_the_nebula_owner_table_holds_on_this_tree`
   (player.rs — the d79-side pre-merge contract: the scan derives the
   tier, the levelup sweeps + climbs, the mana counter pays per-6, the
   reset zeroes — the pins the merge's re-targets must preserve).
   Verification = the whole-block C# re-reads cited per row + the
   per-tip `git diff`/`git grep` censuses (the commands' outputs are
   the tables above).

---

# PITCH SEAL — the receipts (2026-11-21, after 03ebefad7)

The seal phase. Code-only mode — witnesses WRITTEN NOT RUN, no compiler
invoked. Tip reads at seal time (`git fetch origin --prune`, one
connection-reset retried after 90 s per the retry rule):

| lane | tip at THIS seal | vs the F-phase census |
| --- | --- | --- |
| W4-d79-sound-pitch-pool (THIS) | `b2c9aa1d2` (seal 1, the Drip port) | past `03ebefad7` |
| W4-f49-mana-regen-2 | **`0d1f941af`** | past `6c0ee2aa2`: the MANA FINAL (`cded67142`, R3 EXECUTED `8eb20d8b`) + the RETIRE receipts RT1-RT4 (research/85 SEALED) |
| W4-f12-buff-frame-wiring | **`22ce2a41f`** | past `ee67786ec`: the closeout/RETIRE wave |
| W4-g21-sounds-main-1 | **`7596fb262`** | past `7da02f439`: batch 5 (hotbar-lock / block-replacement / torch-biome icons, hosts 71/72/73) — **+5 new bare sites** |
| W4-g22-sounds-main-2 | `250488c34` | unchanged (the MENU RETIRE R3 tip; direction-B canonical) |
| W4-g23-sounds-main-3 | `945391e33` | unchanged (DEPRECATED as a base by g22's R1; folds INTO g22) |
| W4-e10-player-sounds-wave | **`8e0d951fc`** | past `432dd5ab1`: the buff-station wave + the census scripts — **+0 new bare sites** (the stations ride `play_use_sound`) |
| W4-d80-tracked-sound-layer | **`a6bcf736a`** (origin; the worktree rides `f0da72ead`) | past `075b954d2`: phases through 12 (the DD2 riders round). Re-censused at the origin tip: **90 bare sites = the base** — the F3 compose-note's zero-bare-overlap verdict HOLDS (the riders' cues ride the tracked arms + the machine streams) |

## S1. The f49 mutual receipt — F1 vs RT2 (`0d1f941af` research/85 §RT2, the 13-row keep-one)

**The question the Lead posed: does d79's §F1 owner-table agree with
f49's §RT2?** Read in full at `0d1f941af` (RT1 `12d9414d3`, RT2
`b2c9a71fa`, RT3 `fb771bfb8`, RT4 the seal `0d1f941af`). **Verdict:
AGREED on every canonical-home ruling; four rows name a different
surviving TWIN of an identical port — this receipt resolves them to
f49's side (the adjudication below), so the merged tree has ONE answer
per row.** The functional split the Lead named — pickup-cue = d79,
scan/folds = f12-g27, mana = f49 — holds on BOTH tables; the difference
is only WHICH TREE'S COPY of the pickup ARM / levelup / reset / pull
survives the dedupe (the cue façade is d79's on both).

### The row-by-row concordance (F1 row → RT2 row)

| C# block | F1 (this doc) said | RT2 (f49) says | SEAL verdict |
| --- | --- | --- | --- |
| Scan writer `:12070-12079` + arms `:10444-10461` | KEEP f12's `apply_orb_arm`; DELETE d79's AND f49's | row 8: KEEP f12/g27 `nebula_buffs` (blob `c6bcb017b`); d79's drops, f49's ALREADY deleted (`8eb20d8b`) | **AGREE.** f49's deletion is EXECUTED (RT1: the tombstones `player.rs:6874`/`:3523`); d79's `update_buffs_nebula`/`_scan` + the `:6578` call still live on THIS tree — the merge deletes them (the pre-merge witness pins them until then). |
| Life/Damage folds `:10448-10461` | KEEP f12's `fold_orb`; DELETE d79's two arms | rows 9/10: same (the double-add argument verbatim) | **AGREE.** |
| Mana reader `:19565-19585` | KEEP f49's `regen.rs` engine form; DELETE d79's Player-method twin + `Player.nebula_mana_counter` | row 11: KEEP d117/f49 `regen.rs:374` (`RegenState` the ONE counter); d79's twin + field + head call drop | **AGREE.** |
| Positioned case-7 cue | KEEP d79's `play_grab_sound` (Stop-then-recreate); f49's `_at` emit-only twin retires, its 2 call sites re-point | row 3: KEEP d79's `play_grab_sound` :2468; f49's `play_grab_sound_at` retires (chest-eat :1895 rewires too) | **AGREE** — the Lead's "pickup-cue = d79". |
| Pickup arm `:35045-35058` | KEEP d79's (hardcoded map = the `Item.cs` rows verbatim); f49's registry-read arm is the duplicate | row 2: KEEP **f49's** arm (registry `buff_type` read), REWIRED onto d79's façade; d79's hardcoded arm drops | **RESOLVED → f49's arm.** Both maps are the same values (registry rows verified: `itemdata.rs:3619-3621` buff_type 179/173/176 = `Item.cs:31001-31018`); RT2's wholesale-items.rs resolution (RT1 item 3) makes f49's arm the survivor with ONE rewire (`play_grab_sound_at` → `play_grab_sound`, the `(int)` casts already equivalent). F1's "KEEP d79's" is superseded — the behaviour is identical either way, but ONE table must name ONE tree and RT1's items.rs-wholesale instruction already commits to f49's file. |
| `NebulaLevelup` `:57030-57059` | KEEP ONE — d79's ("either survives, the merge dedupes") | row 6: KEEP **f49's** `player.rs:3492` (the anchors:904-equivalent marker); d79's :3481 drops | **RESOLVED → f49's.** F1 explicitly permitted either; RT2 names f49's because its anchors row + witness set ride it. d79's `fn nebula_levelup` anchors row (:903) REWORDS its prose to cite f49's home at the merge (never deleted — the marker `fn nebula_levelup` matches f49's `pub(crate) fn nebula_levelup` too, so the canary survives the swap). |
| Chained reset `:18755` | KEEP BOTH forms (Player trio zero + `orb_scan.reset_effects`) | row 7: KEEP **f49's** Player-zero lines + f12's orb_scan reset; d79's :6291-6293 drops | **RESOLVED → f49's Player-zero copy.** The BOTH-OBJECTS ruling agrees; only the Player-zero twin's tree changes (identical three lines). |
| Pull arm `:34953-34956` | KEEP ONE — d79's ("identical rows") | row 1: KEEP **f49's** `items.rs:685-688`; d79's :700-704 drops | **RESOLVED → f49's** (the same wholesale-items.rs argument; F1 permitted either). |
| 4143 Mana Cloak arm `:35080-35097` | KEEP d79's (the phase-7 text-font sizing convention) | row 4: KEEP **f49's** arm (`items.rs:324+`); d79's drops | **RESOLVED → f49's arm.** Re-read at `0d1f941af`: f49's arm carries the SAME RAW-50-before-cap text with the SAME font-sizing match (`combat_text_font` width/line_spacing, the 8.0/10.0 fallback) — F1's "d79's carries the convention" premise is stale, BOTH carry it. The bodies are line-equivalent; the wholesale resolution keeps f49's. |
| `IsAPickup` gate | (F1 implicit: d79's gate-with-exclusions) | row 13: KEEP f49's gate-REMOVED shape; d79's exclusions AND f12's stale gate drop | **AGREE with RT2** — the gate removal is the faithful shape (every `IsAPickup` kind has a consume arm); d79's exclusion list is the same admission expressed as a skip. |
| Player fields `:924-930` | KEEP the trio on Player; DELETE d79's counter | RT3: the tier FIELDS + bridge = f49's (`Player.nebula_level_*` + `fold_orb_nebula_bridge`) | **AGREE.** |
| Full-mana refill cue `:19550-19560` | (d79's phase 6, no f49 counterpart) | row 12: KEEP d79's queue/capture/drain; lands on the LIFTED `update_survival` (f49's engine already returns `refilled`) | **AGREE** — d79's refill witnesses land clean. |
| Star restock cue `:35067` | (d79's `play_pickup_sound` shared) | row 5: KEEP d79's pool path (f49's arms already call it) | **AGREE.** |

### The bridge-ASSIGN-with-deletions ONE-COMMIT contract (restated, mutual)

Both tables carry the same invariant, now with f49's executed half:

1. **f49's side is DONE** (RT1, verified at `0d1f941af`): the three scan
   arms deleted (`8eb20d8b`), the `fold_orb_nebula_bridge(mana, life,
   damage)` ASSIGN stub landed (`player.rs:3547`), `update_survival`
   LIFTED to the wiring seam (`world_sim/player.rs:1204`) + the wrapper
   re-add (`player.rs:5763`).
2. **The merge's one commit** lands: f12's `update_buff_frame` at the
   marked seam + the `fold_orb` bridge (f12's published 2-line hunk
   compiles as-is — `fold_orb(p, e)` reads `p.orb_scan` directly; the
   3-line stub-call form is the all-tiers equivalent, both ASSIGN-shaped
   per RT1) + the d79-side deletions (the `update_buffs_nebula`/`_scan`
   fns + the `:6578` call + the two stat-fold arms + the Player-method
   mana reader + `Player.nebula_mana_counter`) + the items.rs-wholesale
   resolution with the ONE cue rewire (`play_grab_sound_at` →
   `play_grab_sound`). **Either half alone is a behaviour bug**
   (double-write / stranded reader) — F1 and RT1/RT2 agree verbatim.
3. **The extra_regen sum** (RT1 item 4): the merge keeps f12's
   `accessory_life_regen + buff_life_regen` read — f49's lifted read
   lacks the sum; without it the Nebula Life add never lands. d79's
   tree carries the accessory-only read too; the sum is the merge's.
4. **The witness re-targets** (F1's list, updated): d79's two pickup
   witnesses fold into f49's set (RT2's instruction — keep ONE set per
   behaviour, f49's anchors marker survives, its audio assertions
   re-target to the d79 façade); d79's scan/levelup/counter pre-merge
   witness (`w4_d79_final_the_nebula_owner_table_holds_on_this_tree`)
   stays valid ON THIS TREE until the merge, then its scan/counter
   assertions re-target to `orb_scan` + `RegenState` (the f49 re-target
   pattern); the refill witnesses land clean on the lifted seam.

**Mutual-receipt status: PUBLISHED from the d79 side.** f49's RT2 read
d79 at `b98130d6` (sweep 4) — before the PITCH FINAL owner-table existed;
this section is the d79 answer at f49's SEALED tip. No f49 re-read is
needed: every RT2 row is either agreed or resolved-to-f49 above, and the
resolutions only SIMPLIFY the merger's work (one wholesale file + one
cue rewire instead of a per-arm splice).

## S2. The sibling-rewire execution record — per-lane tables at the CURRENT tips

The Lead's instruction: the 43-rewire-target execution happens on the
SIBLING lanes' own branches (g21/g22/g23/e10 execute their own sites);
d79 publishes the per-lane rewire-table they execute against. **The
sibling resume sessions (g21c-chat-resume-active, g22-canonical,
e10c-voice-resume-active) were probed: no such branches/worktrees/docs
exist yet** (`git branch -a` + the three worktrees' research dirs +
`--grep` over all refs — zero hits). The tables below are the contract
those sessions read; the F4 tables remain the base record, this section
is the DELTA at the moved tips.

### g21 `7596fb262` (was `7da02f439`) — +26 raw now (21 menu/Unlock + 2 Chat + 3 batch-5)

The batch-5 delta (`git diff a725acb04 7596fb262` added-line census,
total 116 = base 90 + 26):

| site (g21 NEW tip) | C# anchor | bare call | REWRITE |
| --- | --- | --- | --- |
| clicks.rs:1953 | `Main.cs:42506` hotbar-lock click | `play("Unlock")` | `play_single_voice("Unlock", -1.0, -1.0)` — case 22 IS in the 17-row pool (Stop-then-recreate, draw-free); the unpositioned int overload |
| clicks.rs:1962 | `:42544` block-replacement click | `play("Unlock")` | same |
| clicks.rs:1973 | `:42584` torch-biome click | `play("Unlock")` | same |
| ui/mod.rs:1018 | `:42515` → `:3244` hotbar-lock hover edge | `chest_panel.audio.play("Menu_Tick")` | `chest_panel.audio.play_menu("Menu_Tick")` (the FrontDraw borrow) |
| ui/mod.rs:1042 | `:47588` → `:3256` toggleAccMouseOver latch | `chest_panel.audio.play("Menu_Tick")` | same |

The F4 g21 table's 21 rows stand (the clicks.rs line numbers shift
non-uniformly — −1 early, +40 past the batch-3/4 insertions; loop.rs −1;
ui/mod.rs +9 — the table locates by C# anchor, not the line). The 2
Chat DELETEs stand
(npcs.rs:286/:644 at the new tip — d79's phase-7 seams win). The
non-new conflict rulings stand: the Destroyer `Roar_0` (npcs.rs:1953)
→ d79's `play_roar(1, …)` (the int overload's default Style 1), the
kill-table `play_at` (:4733) → d79's `play_kill_cue`, the Grab/Coins
click duplicates → d79's phase-3/5 forms. **g21's rewire count: 24
(19 menu + 3 Unlock + 2 hover) + 2 Chat DELETEs.**

### g22 `250488c34` — unchanged, the CANONICAL menu base (direction B)

The g22-canonical receipt: g22's MENU RETIRE R1-R3
(`docs/research/91-g23-branch-deprecated.md` at `250488c34`) deprecates
g23 AS A BASE — the merger takes g22's tip + the 4-commit g23 fold
(keep-g22/take-g23-unique, the disjoint-queue contract: `pending_cue`
activation / `pending_ticks` draw-chrome / `FrontDraw.audio` front
cues). d79's F4 g22 table (12 rows: 9 prod + 3 test rigs) applies to the
UNION's drains — every `play(cue)`/`play(match id …)` becomes
`play_menu(…)`; the `mod.rs:2596` seed Chat →
`play_single_voice("Chat", -1.0, -1.0)`. **No delta at this seal** (the
tip did not move). g23's 4-unique rows (frame/draw.rs:1843, chat.rs:117
+ the 4 shared drains) fold via direction B — ONE rewrite after the
union.

### e10 `8e0d951fc` (was `432dd5ab1`) — +0 new bare sites (the buff-station wave)

The delta (`git diff 432dd5ab1 8e0d951fc`): the seven-tile BuffStation
family (`Player.cs:32594-32634`, `world_sim/editing/doors.rs:402`) plays
through `play_use_sound` (the case-2 STYLE façade — the audio-only
NextFloat + the per-style pitch on `UpdatePlayers`), the `Item_149`/
`Item_179` bank rows join, the smart-cursor comment cites both press
arms (`Player.cs:28820`/:28827 — the single press-edge play covers the
pair). **Zero new `audio.play(`/`play_at(` sites** (the tip's total
stays 93 = 91 prod + 2 test). The F4 e10 table stands with a +4-line
shift in loop.rs (the comment hunk): the 10 prod menu rows + the cart
fallback DELETE (now `world_sim/player.rs:1370`) + the −7 Roar keep-one
+ the three façade rulings (MaxMana/Hungry/voice-picker) unchanged.
**e10's rewire count: 10 prod menu + 1 cart-fallback DELETE.**

### The updated added-bare ledger (the merger's checklist, FINAL)

Raw added: g21 **+26**, g22 +12, g23 +6, e10 +10 = **54**. Dedupe −4
(the g22/g23-shared drains) = **50 unique**; −2 (the g21 Chat pair,
DELETE not rewire) = **48 rewire targets** (45 prod + 3 g22 test rigs).
Every added site is a draw-free pool cue (cases 10/11/12/22/24) — ZERO
new drawing-case bare plays on any sibling tip (the pitch/pool contract
holds tree-wide). After the rewrites the merged tree's bare count
returns to **0** (the `tracked_sound.rs` Resume seam excepted). The
per-lane execution split: **g21 24 rewires + 2 deletes, g22 12 (9 prod
+ 3 test), g23 2 unique (post-union), e10 10 + 1 delete** = 48 + 3
deletes. The base Bug-17 flow.rs pair rides every tip (d79 retired it;
the merged tree rewires it too if the sibling hover hunk survives).

## S3. The case-39 Drip gap — CLOSED (ported, `b2c9aa1d2`)

The Lead's option "port small" was taken (the gore-pool drop was the
F2 census's ONE in-band gap; the task-seed alternative is moot). The
port, verbatim against the C#:

- **`tr-core/src/gore.rs`** — the `GoreSound` seam (the `GoreCollision`
  injection pattern; `tr-core` cannot borrow `tr-app`'s Audio).
  `update_drip_collision` (`Gore.cs:915-977`) now: the `flag4` quiet
  kinds (716/717/943/1160-1162, `:942`) skip both arms; the tile-hit
  arm (`:953`) draws ONE `Next(2)` STYLE pick **at the C# call-argument
  position** (mid-walk, BEFORE the PlaySound gate — a refused play
  still spends it) and calls the seam; the wet arm (`:965`) passes
  style 2 with NO style draw. `NoGoreSound` keeps the pool's own tests
  stream-exact. **An after-walk intent queue was rejected mid-port** —
  it would move the case's pitch draw out of the inter-slot stream
  order (the `Main.cs:17771` `SwapRandom("UpdateGores")` wraps the walk
  AND the plays).
- **`tr-app/src/audio.rs`** — `Audio::play_drip`
  (`LegacySoundPlayer.cs:1351-1358`): the gate first (case 39 is
  ambient-scaled, `DoesSoundScaleWithAmbientVolume` `:1516`; the `:433`
  `type != 39` post-scale zero-volume exemption means a menu-muted
  drip STILL draws its pitch at volume 0 — the TOP `soundVolume == 0`
  gate `:369` refuses draw-free), ONE `Next(-30, 31) * 0.01` pitch on
  the caller's stream, `num2 * 0.5` volume (`:1354`, via the gate's
  `volume_scale`), the no-Stop `SoundInstanceDrip[3]` overwrite
  (`:1353` — the Coins/Chat/MaxMana detach family). The `Drip_0/1/2`
  bank rows pin id 39 (`SoundID.cs:100`; the three `.xnb` assets ship).
- **`tr-app/src/gore_seam.rs`** — `AppGoreSound` forwards the seam;
  **`app/frame/world.rs::gores_phase`** threads it (the one prod
  caller).
- **Witnesses (4, WRITTEN NOT RUN):** the tile-hit style-then-pitch
  order twin + the wet style-2 no-style-draw twin + the six quiet
  kinds' zero-play/zero-draw refusal (gore.rs); the façade's
  gate/exemption/no-Stop/bank pins (audio.rs). The bank-pin census
  witness re-pins `Drip_0/1/2 → 39`.
- **Stays dropped (draw-free):** the `:967` water-shader ripple + the
  `:917-941` shimmer light (the `LiquidRenderer` dependency).

**The F2 census verdict updates: all 43 in-band cases (0-42) PORTED**
(41 on d79 HEAD incl. the new 39; 20 → e10, 37 → g23 at the merge). The
gore-lane co-ownership note: f5's research/111 gore-family census
(853 sites) is SPAWN-site scoped — the drip SOUND is the pool's Update
walk, not a spawn site; no f5/g62 conflict (g62's hit-half is the
`:86532`-family wiring, not the drip). The six out-of-band gaps stay
(§S4).

## S4. The six out-of-band E21 gaps — the task-seeds (owner-lanes named)

Every one is an unported-CALLER deferral (the case specs are
table-complete in F2's rows). The seeds, with the owner-lane the Lead's
catalog assigns:

| gap | C# caller (unported) | case spec (F2) | owner-lane / seed |
| --- | --- | --- | --- |
| 47-52 guitar chords | `Player.ItemCheck_PlayInstruments` (`Player.cs:46506`) → `PlayGuitarChord` (`:46671-46702`): the 1-in-10 `RollOnlyBadLuckExtreme(10) == 0` bad-luck roll (`:18208`, the luck lane e26's machine) then `Next(-50, 51) * 0.01` pitchOffset; the range ladder 1/6-steps picks 49/48/47/51/52/50 | slots 133-138, the family Stop sweep `:1220-1226`, ×0.45 volume, NO case pitch (the offset rides `:1410`) | **the instrument-use seam** — i22's NOTE projectile is a different block; seed rides the item-use census (research/60) + e26 luck. d79's façade contract: the six chord ids join the bank under their cases when the seam lands |
| 53-62 drum family | `PlayDrums` (`Player.cs:46705-46747`): the 1/10-step range ladder picks 59/58/53/57/62/61/54/56/55/60, NO luck roll | slots 139-148, per-slot Stop, ×0.7 volume, no pitch | **same seam as 47-52** (one instrument lane) |
| 63/64 Journey research | `ItemSlot.cs:1288-1297` (the sacrifice-click arm): `SacrificedAndDone` → 64, default → 63; `clicks.rs:1818` records the deferral | 63 draws ONE `Next(1, 4)` slot pick (`:482`), 64 fixed slot 0; both Stop, no pitch | **c24-journey-ui's lane** (`ff99f1486`, the Journey UI wiring) — the sacrifice-click seam is its scope; `creative_sacrifices.rs` is the DATA table |
| 65 QueenSlime ambient | `NPC.cs:45757-45759` (`AI_121_QueenSlime`): the `Next(360) == 0` ambient roll on UpdateNPCs; the Queen Slime NPC (kind 657) is spawn/data-only | 3-slot playing guard (115/116/117), ONE `Next(115, 118)` slot pick, ×0.5 volume, NO pitch | **the d33/d43 NPC-AI/kind lanes** — the AI_121 body is the aiStyle-121 machine's (d33's 95-127 band covers the style; the kind 657 row is d43's 631-697 band); the roll rides UpdateNPCs when the body lands |
| 66 type-75 bomb ambient | `Projectile.cs:46936-46941` (`AI_016_Bombs`' type-75 arm): `soundDelay == 0` → −1 + the play; `projectile.rs:314` records 147/75/10/181 as unported stand-ins | 3-slot guard (121/122/123), ONE `Next(121, 124)` pick + ONE `Next(-15, 16)` pitch, ×0.45 volume | **d8-proj-ai-71-80's lane** (`e9de71683`, the band 71-80 all-armed table — the type-75 body is its scope); TWO draws when it lands |

**No bare-play regression anywhere** — none of the six has a ported
emitter playing through the retired `audio.play()` path (the F2 verdict
stands). The seeds are recorded here + the ROADMAP S1-E-21 row; the
owner-lanes execute on their own branches.

## S5. The pitch-family bible + SEALED

**The pitch-family declaration (the merger-bible, one paragraph):** the
LegacySoundPlayer pitch/pool family's ONE canonical home per member —
the per-case façades + the bank + the 17-row SINGLE_SLOT_CUES pool + the
dedicated slots (pixie/moonlord_cry/drip) + the `play_menu`/
`play_single_voice` draw-free paths are **d79's `audio.rs` wholesale**
(the merge keeps it; d80's bank rows + test seams join, e10's
Female_Hit/voice-picker/Hungry-style façades join per the F4 rulings,
g23's star-hit pair joins via the g22 direction-B union); the tier
WRITER is **f12/g27's `apply_orb_arm`** (blob `c6bcb017b`), the mana
cadence **d117/f49's `regen.rs`** (`RegenState` the ONE counter), the
pickup/levelup/reset/pull **f49's copies** (the S1 adjudication — the
cue on d79's façade), the bridge **f49's stub + f12's `fold_orb`
ASSIGN** in ONE commit with the deletions; the sibling bare sites rewire
onto the d79 façades per the S2 per-lane tables (48 targets + 3
deletes); the tracked layer's `ActiveSound.Resume` `sink.play()` is the
ONE surviving bare `.play()` (not a PlaySound case, no RNG). The
pitch/pool contract: every gameplay draw rides its C# pass's named
stream at the C# statement position; the menu/Chat/Unlock pool cues are
draw-free; the style façades' NextFloat rides the audio-only stream.

**SEALED.** No further d79 commits planned after the seal docs commit.
The lane's record: the sweep phase (`7f3c29da4`/`b78b02022`/`5a719d90e`/
`b98130d66`), the PITCH FINAL phase (`2edd40547`/`ef4fc427b`/`e917c0664`/
`03ebefad7`), the PITCH SEAL (`b2c9aa1d2` the Drip port + this docs
commit). The merger-facing contracts: §S1 (the f49 mutual receipt), §S2
(the per-lane rewire tables), F3 (the d80 compose-note), F4 (the façade
keep-one rulings), §S4 (the E21 task-seeds). Witnesses tree-wide:
WRITTEN NOT RUN (code-only mode; the catalog command runs at the merge).

### SEAL files touched

| File | Change |
| --- | --- |
| `crates/tr-core/src/gore.rs` | the GoreSound seam + the drip-collision port + 3 witnesses (seal 1) |
| `crates/tr-app/src/audio.rs` | `play_drip` + the Drip bank rows + the drip_slots + the witness re-pins + 1 witness (seal 1) |
| `crates/tr-app/src/gore_seam.rs` | AppGoreSound (seal 1) |
| `crates/tr-app/src/app/frame/world.rs` | gores_phase threads the seam (seal 1) |
| `anchors.txt` | the play_drip canary row (seal 1) |
| `docs/research/W4-d79-pitch-final.md` | the SEAL banner + §S1-§S5 (this commit) |
| `docs/research/INDEX.md` | the W4-d79-final row → SEALED (this commit) |
| `docs/ANALYSIS.md` | the PITCH SEAL note (this commit) |
| `docs/PORT_MAP.md` | the case-surface row's 39-closed update + the nebula row's S1 pointer (this commit) |
| `docs/ROADMAP.md` | the S1-E-21 row's seal note + the E21 task-seed pointer (this commit) |