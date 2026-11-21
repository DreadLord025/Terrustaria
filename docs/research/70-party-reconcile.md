# 70 — Birthday-party 3-way reconcile (merger assist)

Code-only reconciliation of the four in-flight lanes that touch the
birthday-party slice. **No code changes** — this note is the merge plan's
per-function verdict sheet. Every C# claim below was re-read against the
decompiled windows in ≤100-line reads (citations inline); every port claim
was read from the lane's own diff (`git diff <base>..<tip>`), not memory.

| Lane | Tip | Scope (party-relevant) |
| --- | --- | --- |
| **W4-f38** | `7f647fda` | town naming + move-in chatter; keyed party broadcasts; `FreeCakeDialogue` chain (`check_dialogue` helper, `Npc.next_dialogue`, `get_chat` head, talk-seam gift) |
| **W4-d120** | `6381e43e` | `GetChat` re-pin (flag10..15 renames, real PartyGirl `Next(7)` ladder, mixed-chatter directions, event census, live feeds); FreeCake **collapsed to the talk**; dusk clear |
| **W4-f34** | `43c44837` | SugarRush 192 effect row (+ Dead Cells 366, honey); the seven station-buff dispatch/app arms (incl. cake 621) |
| **W4-f41** | `91f195fa` (this branch) | party monolith + cake right-click kind-literal arms; break records 455/621/457; SugarRush 192 effect row; freeCake gift chain (inline poll + talk arm) |

The four branches are siblings off wave-3 (`a725acb0`); none contains any
other's party work. Three of them (f38, d120, f41) each wrote a **complete,
mutually incompatible FreeCake chain**; two (f34, f41) each wrote the same
SugarRush effect arm; two (f34, f41) each wrote a cake-621 right-click
route. Exactly ONE of each may survive.

---

## 1. Per-function 4-way table

C# truth column first; then what each lane carries. "—" = lane does not
touch the function. Verdicts in §2.

### 1a. `NPC.freeCake` writers/clears (the world latch)

C# truth: static `NPC.freeCake` (`NPC.cs:6093`). Writers, complete census:
`BirthdayParty.NaturalAttempt` sets it (`BirthdayParty.cs:106`, after
`PartyDaysOnCooldown = Next(5,11)`, before the celebrant pick);
`UpdateTime_StartNight` clears it (`Main.cs:65222`, **before** the
`BirthdayParty.CheckNight()`/`LanternNight.CheckNight()` pair at
`:65225-65226`); `WorldGen.clearWorld` clears it (`WorldGen.cs:6945`).
`CheckNight` itself (`BirthdayParty.cs:40-58`) never touches it.

| Site | f38 | d120 | f34 | f41 |
| --- | --- | --- | --- | --- |
| field | `WorldEvents::free_cake` | same | — | same (field-placement deviation recorded: all writers are party-machine sites) |
| `:106` arm | `ev.free_cake = true` in `check_morning_party` after cooldown set | identical | — | identical |
| `:65222` dusk | `advance_time_with_events` StartNight, before event pair | identical | — | identical |
| `:6945` clearWorld | `world_clear` | identical | — | identical |

All three copies are line-for-line equivalent (comments differ only).
**KEEP one copy of each site** — d120's wording is the most precise
("EVERY dusk … the dawn keeps it"); the anchor rows
(`clock.rs|app.world_events.free_cake = false`, d120) survive any copy.

### 1b. The pending-dialogue field + registry stand-in

C# truth: `NPC.nextDialogue` (`NPC.cs:6539`), a `ConditionalDialogue`
object; `ConditionalDialogue.Init` (`ConditionalDialogue.cs:122-125`)
registers exactly ONE row: `FreeCakeDialogue` on type 208, condition
`(NPC _) => NPC.freeCake` (`:51-56`). Transient — never saved.

| | f38 | d120 | f34 | f41 |
| --- | --- | --- | --- | --- |
| field | `Npc.next_dialogue: bool` (one-row registry stand-in, doc'd transient) | **none** (collapsed away) | — | `Npc.next_dialogue_free_cake: bool` (same stand-in, longer name) |
| registry shape | `pending_dialogue(kind, free_cake)` + `conditions_met` fns in `npc_chat.rs` | — | — | inline `is_party_girl && free_cake` in the poll |

**KEEP f38's `next_dialogue` + helper fns** (its anchor row
`npc_chat.rs|pub fn check_dialogue` needs them). DROP f41's
`next_dialogue_free_cake` name — see §5 anchor re-point.

### 1c. `NPC.CheckDialogue` — the 10-tick poll

C# truth (`NPC.cs:90519-90532`, called from `UpdateNPC` `:90510`):
`netMode != 2` gate; on `(GameUpdateCount + whoAmI) % 10 == 0` the probe
**REPLACES** `nextDialogue` via `TryGetPendingDialogue`
(`ConditionalDialogue.cs:85-102` — null when no condition holds); on the
other nine ticks a pending row whose `ConditionsMet` failed clears
immediately. The predicate itself is ONLY `NPC.freeCake` — the "is alive"
gate comes from the caller: `UpdateNPC` never runs for an inactive NPC.

| | f38 | d120 | f34 | f41 |
| --- | --- | --- | --- | --- |
| shape | `pub fn check_dialogue(npc, count, who, free_cake)` in `npc_chat.rs`, called per-NPC from `update_npcs` phase head | **absent** — "collapses to the talk" (recorded deviation) | — | inline block in `update_npcs` phase head |
| cadence | `(count.wrapping_add(who)) % 10`, who = stable `uid` (compaction deviation recorded) | — | — | identical (`count.wrapping_add(n.uid)`) |
| replace-on-poll | yes (`= pending_dialogue(...)`) | — | — | yes (`= is_party_girl && free_cake`) |
| between-poll fall clear | yes | — | — | yes |
| alive gate | **missing** (polls corpses until the death retain) | — | — | `hp > 0` term inside both arms |

**KEEP f38's helper + call loop, fold f41's alive gate in as a CALLER
gate**: `for n in ... { if n.hp > 0 { check_dialogue(n, count, n.uid,
free_cake) } }`. That matches the C# split (predicate = freeCake only;
active gate = `UpdateNPC`'s caller). f41's in-predicate `hp > 0` is
behaviourally the same for the one row but structurally misplaces the
gate; f38's unguarded loop is the one real fidelity bug here (a girl who
died this tick would still poll/clear before the death retain at
`world_sim/npcs.rs` `retain(|n| n.hp > 0)`).

### 1d. `GetChat`'s cake head + the talk seam's gift

C# truth: `GetChat` (`NPC.cs:93766`) head `:93768-93771` —
`if (nextDialogue != null) return nextDialogue.GetChatAndClearCondition(this);`
**before** the Joja-cola/Dryad re-roll resets and the flags scan.
`FreeCakeDialogue.GetChatAndClearCondition`
(`ConditionalDialogue.cs:58-66`) in order:
1. `NPC.freeCake = false` (`:60`)
2. `NetMessage.SendData(51, …, 10f)` (`:61` — net lane's, all lanes record it unported)
3. `new Item(); SetDefaults(3750)` (`:62-63` — **no prefix roll**; maxStack
   = `CommonMaxStack` 9999, case 3750 writes none, `Item.cs:33214-33226` → `:49017`)
4. `QuickSpawnItem(EntitySource_Gift, item, GiftRecieved)` (`:64` →
   `Player.cs:7187-7191` → `GetOrDropItem` `:7193-7200`: `GetItem` fills
   inventory, the remainder drops via `Item.RequestNewItem` at
   `base.Center` keeping prefix)
5. **then** `PartyGirlSpecialText.Cake{Main.rand.Next(1,4)}` (`:65`).

`GetChat`'s only caller is `Main.cs:43082` (census: one hit repo-wide),
outside every `SwapRandom` stage — both the gift's `NewItem` draws and the
text draw ride the bare `Main.rand` (the port's `rngs.global()`, S1-A-10).
The C# does NOT clear `nextDialogue` at talk time — the next
`CheckDialogue` tick clears it (condition fell); the same-NPC re-talk
decline (`Main.cs:43077`) makes the one-shot observable either way.

| arm | f38 | d120 | f34 | f41 |
| --- | --- | --- | --- | --- |
| head placement | inside `get_chat` (structural 1:1), text via `cake_chat(ctx)` | in `talk_to_npc` before ctx build | — | in `talk_to_npc` before ctx build |
| fire condition | `npc.next_dialogue` (the latched row) | **`kind == PartyGirl && free_cake` directly** (no latch) | — | `npc.next_dialogue_free_cake` (the latched row) |
| gift-vs-text order | **text FIRST** (inside `get_chat`), side effects drained after | gift first, then `Next(1,4)` | — | gift first, then `Next(1,4)` |
| inventory fill | multi-slot `while pending > 0` loop | single `slot_for_pickup` + `count += 1` | — | single `slot_for_pickup` + `.min(max_stack)` clamp |
| overflow stream | **`RngKey::UpdateItems`** | `rngs.global()` | — | `rngs.global()` |
| overflow site | player centre, `if let Some(p)` | player centre, `if let Some(p)` | — | player centre, `(0,0)` fallback when no player |
| latch clear | after text | n/a (no latch) | — | before gift |
| `freeCake` clear | after text | before gift | — | before gift |

**KEEP: ONE chain = f38's structure with d120/f41's ordering.** Arm-by-arm
faithfulness verdict in §3.

### 1e. SugarRush 192 — the `UpdateBuffs` effect row

C# truth (`Player.cs:9935-9939`): `buffType[j] == 192` →
`pickSpeed -= 0.2f; moveSpeed += 0.2f`, rebased every tick by
`ResetEffects`.

| | f38 | d120 | f34 | f41 |
| --- | --- | --- | --- | --- |
| arm | — | — | `192 => { pick_speed_bonus += 0.2; move_speed += 0.2 }` in the buff match, beside its 366 row + the `ResetEffects` `dead_cells_potion_station` clear (`:18362`) | **byte-identical arm**, same match, same spot (after 159) |
| witness | — | — | `w4_f34_the_sugar_rush_buff_arm_is_verbatim` (grant + the 7199 decrement; 192 NOT in `TimeLeftDoesNotDecrease`) | `w4_f41_sugar_rush_speeds_the_pick_and_the_run` (base-relative +0.2/−0.2, the rebase-never-doubles tick, the expiry drop) |

**KEEP f34's arm** (identical code; f34's context carries 366 + the
ResetEffects clear and its anchor row names the honey witness in the same
file). DROP f41's duplicate arm — two `192 =>` arms in one match would not
compile. **Witness union: keep BOTH** (disjoint aspects, distinct names).

### 1f. Cake 621 right-click (dispatch + app arm)

C# truth (`Player.cs:32600-32605`): `flag2 = true; AddBuff(192, 7200);
PlaySound(SoundID.Item2, base.position)`. `SoundID.Item2 =
LegacySoundStyle(2, 2)` (`SoundID.cs`), so the cue is the case-2 item-use
path: the `GetRandomPitch` NextFloat (`SoundEngine.cs:99`), then the
default-arm pitch `Main.rand.Next(-6,7)*0.01`
(`LegacySoundPlayer.cs:632`), at the PLAYER's position (façade truncates).

| | f38 | d120 | f34 | f41 |
| --- | --- | --- | --- | --- |
| dispatch | — | — | `StationBuff { buff_type, buff_time }` — the seven-row kind match (`125/621/464/699/377/354/287`), `621 => Some((192, 7_200))`, before the dials; supersedes the old station-tail craft-filter click | `CakeSlice` kind-literal variant + `if t.kind == 621` arm after the dials |
| app arm | — | — | shared `StationBuff` arm: `add_buff` THEN `play_at(cue, p.x, p.y, volume)` — the **unpitched e28 stand-in** (recorded deviation), cue map `192 → Item_2` | dedicated arm: `add_buff(192,7200)` + `play_npc_item_cue(2, cx, cy, UpdatePlayers rng)` — the **faithful pitched case-2 play** |
| anchors | — | — | `edit.rs\|621 => Some((192, 7_200))` | none for CakeSlice |

**KEEP f34's `StationBuff` dispatch + app arm** (anchor-bearing, covers
all seven C# arms in chain order; f41's route is a strict subset). DROP
f41's `CakeSlice` variant, dispatch arm, `interaction_events` arm, app
arm, `TR_TOGGLE` log arm and its two witnesses (§4). **Fidelity note for
the merger**: f41's `play_npc_item_cue` is the faithful case-2 draw shape;
f34's `play_at` keeps the e28-recorded unpitched stand-in for all seven
rows. Folding the pitched seam into the `StationBuff` arm (at least the
192 row) is a legitimate follow-up, but it changes f34's recorded
deviation — do it as its own noted change, not silently in the merge.

### 1g. Party monolith 455 right-click — f41 UNIQUE

C# truth (`Player.cs:32278-32284`): `flag2 = true; PlaySound(28, myX*16,
myY*16, 0); GamepadEnableGrappleCooldown(); ToggleManualParty()`. No other
lane touches 455's interaction. **KEEP f41 wholesale**: the
`PartyMonolithToggle` variant + dispatch, the app arm (mech case-28 cue on
the UpdatePlayers stream, `manual_party` flip, THROW_A_PARTY up-edge),
the `TR_TOGGLE` log arm, both witnesses, the anchor row. (The gamepad
cooldown is the recorded input-census skip.)

### 1h. Break records 455/621/457 — f41 UNIQUE

No other lane touches `edit/kill.rs`. **KEEP f41 wholesale**: 455 in
`check_3x3_bottom`'s support group (drop 3747, `WorldGen.cs:53224`); 621
in `check_2x2` (drop 3750, `:48609`); 457's `check_2x2_style` verbatim
(the `:52300-52314` style decode, the **frameX-ONLY** `:52323` integrity
scan, `:52328` floor gate, drop 3749 `:52395`, the 4x4 tail ring); the
census (457 has NO right-click arm; 3749 not in `OpenableBag`,
`ItemID.cs:1384` — the present is place/break-only); all 6 witnesses; the
anchor row.

### 1i. Party broadcasts — f38 UNIQUE (and a stale-literal fix)

C# truth: the dawn start rows `Game.BirthdayParty_{1,2,3}` over the
celebrants' `GetGivenOrTypeNetName` (`BirthdayParty.cs:127-139`), colour
`new Color(255,0,160)`, before the achievement tail; the dusk farewell
`NetworkText.FromKey(Lang.misc[99].Key)` (`:56`) —
`LegacyMisc.99 = "Party time's over!"` (verified in
`Terraria.Localization.Content.en-US.Legacy.json`).

| | f38 | d120 | f34 | f41 |
| --- | --- | --- | --- | --- |
| dawn | keyed `Game.BirthdayParty_{n}` via `npc_names::given_or_type_name` + `lang::text_args`, in the clock's dawn arm before `notify_progression_net(25)` | — (no broadcast work) | — | — (base shape) |
| dusk | `lang::misc(99)` keyed farewell | **stale literal** `"Everyone has gone home!"` (base) | — | **stale literal** (base) |

**KEEP f38's broadcasts + naming layer wholesale** (`give_town_unique_data`,
`given_or_type_name`, the housing-hover FullName, the move-in/departure
announcements — all f38-unique, anchor-bearing). The f41/d120 copies of
`check_night_party` carry the pre-1.4.5 literal; f38's keyed row wins
(the literal is a bug, not a variant).

### 1j. `GetChat` body (the ladder itself) — d120 CANONICAL

d120's re-pin is the audited body: flag10..15 renames (160 Truffle /
229 Pirate / 208 PartyGirl / 353 Stylist), the real PartyGirl
`Next(7)`-up-front ladder (`NPC.cs:94410-94456`), the FindAll-forward
`mixed_chatter` vs SelectRandom-reverse picks, the event-text census
(count draw paid even at 0, `HasNoPartyText` AFTER the party roll),
`normalized_luck`, and the live feeds (rain latch, defense, head slot,
male, cola HasItem, graveyard/storm/windy/party/lanterns/moonlord/dd2-t3).
f38's diff sits on the OLD ladder (its `party_girl` fn is the mislabelled
Truffle ladder with `next(6)`); f41 does not touch the ladder.
**KEEP d120's `npc_chat.rs` body + `lang.rs` dialog helpers +
`world_sim/npcs.rs` live-feed ctx wholesale**; graft f38's head +
`check_dialogue` + `cake_chat` onto it (§3). f38's live-feed subset
(party/lanterns/moonlord) is subsumed by d120's superset — drop the
subset hunks, keep d120's.

---

## 2. KEEP sheet (per function, one winner)

| Function | KEEP | Source |
| --- | --- | --- |
| `free_cake` field + `:106`/`:65222`/`:6945` writers | one copy (identical everywhere; d120 wording) | any of f38/d120/f41 |
| pending-row field | `Npc.next_dialogue` | f38 |
| registry stand-in + poll helper | `check_dialogue` / `pending_dialogue` / `conditions_met` | f38 |
| poll call loop | f38's `update_npcs` loop **+ f41's `hp > 0` caller gate** | f38 ⊕ f41 |
| `get_chat` cake head | f38's head + `cake_chat(ctx)` | f38 |
| talk-seam gift | latch-gated, **side effects BEFORE the text draw**, single-slot fill, overflow + text on `rngs.global()` | d120/f41 ordering in f38's structure (§3) |
| SugarRush 192 effect arm | the single `192 =>` arm | f34 |
| cake 621 right-click | `StationBuff` dispatch + app arm (unpitched cue = f34's recorded deviation; f41's pitched seam noted as follow-up) | f34 |
| monolith 455 right-click | `PartyMonolithToggle`全套 (dispatch/app/log/witnesses/anchor) | f41 |
| break records 455/621/457 | `check_3x3_bottom`/`check_2x2`/`check_2x2_style` rows + census | f41 |
| party broadcasts + naming | keyed dawn/dusk rows, `given_or_type_name`, naming layer | f38 |
| `GetChat` ladder + live feeds | the re-pinned body | d120 |

## 3. The ONE FreeCake chain — f41 vs f38 arm-by-arm

Which arms of each are faithful to `ConditionalDialogue.cs:58-66` +
`NPC.cs:90519-90532`/`:93768-93771` (all re-read):

| Arm | Faithful winner | Why |
| --- | --- | --- |
| latch exists at all | **f38/f41 (tie)** | d120's collapse fires on `kind && free_cake` at talk time — the C# fires only on a row latched by a poll. Up to 9 ticks after the dawn party (and after a mid-day re-arm) d120 gifts where the C# answers chatter. d120 recorded the collapse; the un-collapsed shape exists now, so the deviation need not survive. |
| poll cadence + replace + fall-clear | **tie** (identical modulo the field name) | both `(count + uid) % 10`, replace-on-poll, immediate fall-clear; uid-for-whoAmI is the shared recorded compaction deviation. |
| alive gate | **f41** | the C# predicate is freeCake-only, but `UpdateNPC` (hence `CheckDialogue`) never runs for a dead NPC. f41's `hp > 0` models that; f38 polls corpses until the death retain. Fold as a caller gate (§1c). |
| head placement | **f38** | `:93768-93771` is inside `GetChat`, before the re-roll resets + flags scan. f41/d120's talk-side placement is observationally equivalent (single caller, census'd) but f38 is the structural 1:1 — and f38's pure-`get_chat` witness needs it. |
| gift-before-text order | **f41/d120** | the C# gifts (`:64`) THEN draws `Next(1,4)` (`:65`). f38 drains side effects AFTER `get_chat` returned the text — on the overflow path the drop's velocity/phase triplet and the Cake draw swap places, changing the pinned-stream outcome. d120's overflow witness pins the faithful order (triplet, then `Next(1,4)`, seed-0 global). |
| overflow/text RNG stream | **f41/d120** | the talk runs outside every `SwapRandom` stage (`Main.cs:43082`, the S1-A-10 contract) — `Item.NewItem`'s draws ride the bare `Main.rand` = `rngs.global()`. f38's `RngKey::UpdateItems` models a swap that is not active here. |
| inventory fill | **f41/d120 (single-slot)** | the gift is exactly ONE item: `GetItem` (`Player.cs:39330-39399`) merges into the first occupied slot with room, else the first empty hotbar slot (useStyle 1, `:39365-39374`), else reverse-empty — one `slot_for_pickup` call is exact (`count < max_stack` ⇒ `+1 ≤ max_stack`; f41's `.min()` clamp is vacuous, d120's bare `+= 1` is the clean form). f38's multi-slot while-loop is equivalent for stack 1 but over-general. |
| overflow guard | **f38/d120** | `if let Some(p)` — the talk gate already requires a live player, so the no-player branch is unreachable; f41's `(0,0)` fallback invents a spawn site the C# never has. |
| prefix | **tie** | all three: no prefix roll on the gift (`SetDefaults` + the Item-variant `QuickSpawnItem`); the port's `spawn_item_drop_px` → `roll_prefix(3750, -1)` draws nothing (not prefixable) — d120's witness pins prefix 0 + the triplet-only draws. |
| latch clear timing | **tie (all slightly early)** | the C# leaves `nextDialogue` set until the next `CheckDialogue` tick; all three ports clear at talk. Unobservable: the same-NPC re-talk declines (`:43077`) and the poll clears next tick anyway. Keep the port form, note it. |
| `SendData(51)` | **tie** | all record it unported (net lane). |
| pickup sound 7 + PopupText | **none** | `GetItem_FillIntoOccupiedSlot`/`FillEmptyInventorySlot` (`Player.cs:39489-39500`/`:39557-39566`) play `SoundID 7` + popup text on the gift; the port's `slot_for_pickup` seam carries neither (the shared S1-C-9 pickup deviation). Census note, not a lane differentiator. |

**The reconciled chain (merge recipe, no code here):**
1. f38's `Npc.next_dialogue` field + `check_dialogue`/`pending_dialogue`/
   `conditions_met` + `get_chat` head + `cake_chat` — verbatim.
2. `update_npcs` poll loop: f38's call shape, body gated `if n.hp > 0`
   (f41's gate, moved to the caller).
3. `talk_to_npc`: after `app.talk_npc = Some(i)`,
   `if app.npcs[i].next_dialogue { free_cake = false; gift ONE 3750 via
   slot_for_pickup (d120 form) with the global-stream overflow drop at
   the player centre }` — **then** the normal ctx build + `get_chat`
   (whose head draws the Cake text on the same global stream) — **then**
   `app.npcs[i].next_dialogue = false`. This keeps f38's structure and
   d120/f41's draw order; the early-return shape of f41/d120 is dropped.
4. d120's anchor marker `app.world_events.free_cake = false` still exists
   in step 3; f38's `pub fn check_dialogue` in step 1.

## 4. Witness union (all WRITTEN NOT RUN — code-only lanes)

| Witness | Lane | Verdict |
| --- | --- | --- |
| `the_dialogue_probe_walks_the_ten_tick_cadence` | f38 | KEEP (helper-level cadence/replace/fall/other-kind) |
| `w4_f41_the_dialogue_poll_latches_and_clears` | f41 | KEEP, rename field → `next_dialogue` (integration-level: uid phase, `update_npcs` wiring) |
| `the_pending_cake_dialogue_wins_over_the_ladder` | f38 | KEEP (pure `get_chat` head; passes over d120's ladder) |
| `the_talk_action_hands_over_the_cake_and_clears_the_condition` | f38 | KEEP (one-shot + clears; order-agnostic asserts) |
| `w4_f41_the_party_girl_gifts_the_cake_once` | f41 | KEEP, rename field (count==1, prefix==0, no-second-gift total — stricter than f38's `any`) |
| `the_party_girl_gifts_the_cake_then_draws_the_text` | d120 | **RE-DERIVE**: seed-0 pinned text assumes the collapse (no poll); under the kept chain the test must latch the row first (set `next_dialogue` or run a poll tick). Keep the pinned-draw asserts after the re-pin. |
| `a_full_inventory_overflows_the_cake_to_a_world_drop` | d120 | **RE-DERIVE** likewise (latch first); the triplet-then-text pinned order is the canonical overflow witness — KEEP its draws/pos/prefix asserts |
| `w4_f41_the_free_cake_flag_arms_on_the_fresh_party_only` | f41 | KEEP (gate-miss/cooldown/roll-miss arms + CheckNight-never-touches) |
| `dusk_clears_the_free_cake_latch` | d120 | KEEP (dusk clear + dawn keeps) |
| `w4_f41_the_dusk_boundary_clears_the_free_cake_gift` | f41 | KEEP (mid-day keeps — disjoint from d120's dawn arm) |
| `s1_f_14_the_party_machine_follows_natural_attempt` (f38's extended form) | f38 | KEEP f38's version (adds the freeCake arm + keyed-farewell asserts; superset of the base test both other lanes carry) |
| `the_party_broadcasts_name_the_celebrants_and_clear_the_cake_at_dusk` | f38 | KEEP (broadcasts unique) |
| `w4_f34_the_sugar_rush_buff_arm_is_verbatim` | f34 | KEEP |
| `w4_f41_sugar_rush_speeds_the_pick_and_the_run` | f41 | KEEP (rebase + expiry — disjoint) |
| `w4_f34_the_cake_war_table_and_dead_cells_station_grant_their_buffs` / `…sharpening_and_bewitching…` / `…draw_no_rng_and_refresh_not_stack` | f34 | KEEP (the surviving StationBuff route) |
| `w4_f41_the_cake_dispatches_the_slice` | f41 | DROP (expects `CakeSlice`; f34's edit.rs witnesses pin `621 → StationBuff{192,7200}`) |
| `w4_f41_the_cake_click_grants_sugar_rush` | f41 | DROP (f34's grant + RNG-free refresh witnesses cover it on the surviving route) |
| `w4_f41_the_party_monolith_click_toggles_the_party` / `…dispatches_the_toggle` | f41 | KEEP (unique) |
| all 6 f41 break-record witnesses (`…monolith_survives…`, `…support_break_drops_3747/3750/3749`, `…cake_stands_on_a_table`, `…present_style_zero_and_the_frame_x_only_check`) | f41 | KEEP (unique) |
| d120's 13 census witnesses + f38's naming witnesses + f34's honey/366/Item179 witnesses | resp. | KEEP (no overlap with the dropped arms) |

## 5. DROP list (with anchor reconciliation)

1. **f41 `Npc.next_dialogue_free_cake`** (field + init) → f38's
   `next_dialogue`. ⚠ The f41 anchor row
   `world_sim/npcs.rs|next_dialogue_free_cake` loses its marker —
   **re-point the row** (same feature text) at a surviving marker, e.g.
   `crate::npc_chat::check_dialogue` (the poll call in `update_npcs`).
   Update the row, never delete it.
2. **f41's inline poll block** in `update_npcs` → f38's helper loop +
   the `hp > 0` caller gate.
3. **d120's collapsed talk arm** (`kind == PartyGirl && free_cake`
   direct-fire) → the latched chain (§3). Its ANALYSIS "collapses to the
   talk" deviation paragraph is superseded — record the un-collapse.
4. **f38's talk-seam side-effect block** as written (post-text ordering,
   `RngKey::UpdateItems` overflow stream, multi-slot while-loop) → the
   §3 recipe (pre-text side effects, global stream, single-slot fill).
   f38's `get_chat` head/check_dialogue/field stay.
5. **f41's duplicate `192 =>` SugarRush arm** in `player.rs` → f34's arm
   (two identical arms in one match do not compile).
6. **f41's `CakeSlice`**: the `TileInteraction` variant, the
   `if t.kind == 621` dispatch arm, the `interaction_events` arm, the
   `interact_tile` app arm, the `app/mod.rs` `TR_TOGGLE` log arm, and the
   two witnesses named in §4 → f34's `StationBuff` route (anchor
   `621 => Some((192, 7_200))`). No anchor names `CakeSlice` — safe drop.
   The pitched-cue fidelity note (§1f) is a recorded follow-up, not part
   of the merge.
7. **f41/d120's stale `"Everyone has gone home!"` literal** in
   `check_night_party` → f38's `lang::misc(99)` keyed farewell (the
   literal is the pre-1.4.5 text; the C# keys `Lang.misc[99]`).
8. **f38's live-feed hunks** in `talk_to_npc` (party/lanterns/moonlord
   only) → d120's superset ctx (rain latch, defense, head, male, luck,
   cola, graveyard/storm/windy, dd2-t3, angler).
9. **Duplicate copies** of the three `free_cake` writer lines and the
   `WorldEvents` field doc (three lanes, identical code) — keep one.

Anchor survivors after the drops: f38 `npc_names.rs|pub fn
give_town_unique_data` ✓, f38 `npc_chat.rs|pub fn check_dialogue` ✓,
d120 `npc_chat.rs|fn mixed_chatter` ✓, d120 `npc_chat.rs|fn party_girl` ✓,
d120 `world_sim/npcs.rs|app.world_events.free_cake = false` ✓ (§3 step 3),
d120 `clock.rs|app.world_events.free_cake = false` ✓, d120
`localization/mod.rs|pub fn select_random_from_dialog` ✓, f34
`edit.rs|621 => Some((192, 7_200))` ✓, f34 `player.rs|w4_f34_honey_contact…` ✓,
f41 `kill.rs|fn check_2x2_style` ✓, f41
`doors.rs|edit::TileInteraction::PartyMonolithToggle` ✓, f41
`npcs.rs|next_dialogue_free_cake` — **RE-POINT** (drop 1).

## 6. Verification

Code-only, per the lane contract: **no cargo/rustc/tests/graphify
invoked**. Evidence: every C# window cited above re-read directly
(`ConditionalDialogue.cs` full 141 lines; `NPC.cs:90495-90539`/`:93760-93794`;
`Player.cs:7175-7204`/`:9930-9944`/`:32268-32292`/`:32580-32639`/
`:39230-39274`/`:39310-39399`/`:39485-39574`; `Main.cs:65212-65231`;
`WorldGen.cs:6940-6951`; `BirthdayParty.cs:40-139`;
`LegacySoundPlayer.cs:510-674`; `Item.cs:33190-33229`/`:49012-49021`;
`GetItemSettings.cs` GiftRecieved row; `SoundID.cs` Item2/Item179 rows;
`LegacyMisc.99` + `PartyGirlSpecialText.Cake1..3` from the en-US JSON).
Every port claim read from the lane diffs (`git diff <base>..<tip>`) and
`git show <tip>:<file>` — no working-tree code was modified by this note.

---

## 7. EXECUTED — the f41 union-exec (branch `task/W4-f41-birthday-party-2`)

The §3 recipe + the f41-scoped §5 drops are EXECUTED on this branch
(commits `0ab54f5e` phase 1 chain, `1449fdd5` phase 2 drops). What landed:

1. **§3 step 1** — f38's `Npc.next_dialogue` field (+ init),
   `check_dialogue`/`pending_dialogue`/`conditions_met`, the `get_chat`
   head + `cake_chat`, verbatim (one comment corrected: `cake_chat`'s
   doc now states the side effects run BEFORE the text draw — f38's
   "drains them after" wording described the dropped ordering).
2. **§3 step 2** — the `update_npcs` poll loop is f38's call shape with
   the body gated `if n.hp > 0` (f41's alive gate moved to the caller;
   the predicate stays freeCake-only).
3. **§3 step 3** — `talk_to_npc`: after `app.talk_npc = Some(i)`,
   `if app.npcs[i].next_dialogue { app.world_events.free_cake = false;
   ONE 3750 via single slot_for_pickup (d120's bare += 1 form), overflow
   drop at the player centre on rngs.global() behind if let Some(p) }` —
   then the normal ctx build + `get_chat` (the head draws the Cake text
   on the same global stream) — then `app.npcs[i].next_dialogue = false`
   (the recorded early-clear timing note). f41's early-return arm and
   d120's collapse shape are gone.
4. **§5 drops 1/2/5/6 executed** — the `next_dialogue_free_cake` field,
   the inline poll block, the duplicate `192 =>` player.rs arm, and the
   whole `CakeSlice` route (edit.rs variant + dispatch, doors.rs app +
   interaction_events arms, app/mod.rs TR_TOGGLE arm) are removed; the
   two CakeSlice witnesses dropped; the pitched-cue follow-up recorded
   in-code at the doors.rs seam (§1f).
5. **§5 anchor re-point executed** — `anchors.txt` row now reads
   `crates/tr-app/src/world_sim/npcs.rs|crate::npc_chat::check_dialogue`
   (same feature text, updated to the reconciled chain). Never deleted.
6. **§4 witness union on this branch (18 WRITTEN NOT RUN)** — f38's 3
   chain witnesses kept verbatim (`the_dialogue_probe_walks_the_ten_tick_cadence`,
   `the_pending_cake_dialogue_wins_over_the_ladder`,
   `the_talk_action_hands_over_the_cake_and_clears_the_condition`);
   f41's 2 renamed to `next_dialogue`
   (`w4_f41_the_party_girl_gifts_the_cake_once`,
   `w4_f41_the_dialogue_poll_latches_and_clears` — doc updated to the
   caller gate); d120's 2 **RE-DERIVED**
   (`the_party_girl_gifts_the_cake_then_draws_the_text`,
   `a_full_inventory_overflows_the_cake_to_a_world_drop` — each now
   latches `pg.next_dialogue = true` before the talk; ALL pinned-draw
   asserts kept: seed-0 `Next(1,4)` text, the triplet-then-text overflow
   order, prefix 0, the centre+(1,0) spawn pos). The remaining 11 f41
   witnesses (SugarRush rebase/expiry, flag-arm, dusk, monolith ×2,
   break records ×6) untouched.

**Merger's remaining scope** (NOT executable on this branch — the
sibling lanes' code is not here): d120's canonical `GetChat` ladder body
+ live-feed ctx + lang helpers (§1j), f38's naming/broadcast layer +
keyed dusk farewell (§1i, drops 7/8), f34's `StationBuff` route + the
single 192 arm (§1e/§1f), one copy of the three `free_cake` writer sites
(drop 9 — this branch's copies are line-equivalent to d120's; keep
d120's wording). This branch KEEPS its monolith + break records
wholesale (§1g/§1h). Post-union witness expectation: this branch's 18 +
f38's naming/broadcast/extended-s1_f_14 witnesses + d120's 13 census
witnesses + `dusk_clears_the_free_cake_latch` + f34's buff/station/honey
witnesses.

Verification (code-only): no cargo/rustc/tests/graphify invoked; the
chain re-read against `ConditionalDialogue.cs:58-66` +
`NPC.cs:90519-90532`/`:93768-93771` via the §6 evidence; every folded
piece read from `git diff a725acb0..7f647fda` / `..6381e43e` and
`git show <tip>:<file>` before the edit; `git grep` confirms zero
remaining `next_dialogue_free_cake` / `CakeSlice` references in
`crates/` and the re-pointed anchor marker present.
