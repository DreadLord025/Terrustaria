# 263 - W4-g87 closeout fold-input: the a6 dup adjudication, the dh-line g88f fold, the .trwld g71c union extension, the slot-8 consumer census

> **RENUMBERED 143 → 263** per the research/150 registry §6.3 (origin/task/W4-d54
> `72226e500`): key 143 became a full 6-way collision — c4's
> `143-d40-crimsongoldfish-retire-hunk.md` KEEPS 143 (earliest first-add); the
> other claimants renumbered (d42→238, g58→240, d120→261, this doc→263,
> c8→238→262). Every former research-143 citation on this branch now reads
> `research/263`.

**Lane:** `task/W4-g87-demon-heart-use` (base `dfbcaf94`, closeout commits
`26620538` use-arm deltas + `236c0d5a` slot-8 UI + this doc).
**Read at closeout:** `origin/task/W4-a6-crystals-powerups` tip `8ed96ee5`
(research/138 NOT pushed — `git ls-tree` shows no 13x/14x file),
`origin/task/W4-g88-life-fruit-use` tip `c8985b53` (research/103 v10 +
122), `origin/task/W4-g90-artisan-bread-use` tip `b3b82d07`
(research/135), `origin/task/W4-g71-bg-layers-2` tip `e2d085ae`
(research/92-doc-sweep-bg), `origin/task/W4-f45-wld-header-2` tip
`6352d384`, `origin/task/W4-f44-save-plr-2` (player_file.rs G7 block).
Code-only: every witness WRITTEN NOT RUN; no cargo per lane constraint.

---

## 1. The a6 dup: per-item KEEP verdicts (input to a6b / research/138)

W4-a6 independently ported the SAME three families this lane owns, on
`crates/tr-app/src/powerups.rs` (a single `powerup_item_check` dispatch
machine, `is_powerup` kind list `:65-71`). The closeout read both ports
arm-by-arm against the C# and **folded a6's four fidelity deltas into this
lane** (commit `26620538`); this lane's
`world_sim/editing/permanent_upgrades.rs` is now the strict superset.

### 1a. Demon Heart 3335 — KEEP g87, DROP a6's arm

C# `ItemCheck_UseDemonHeart` (`Player.cs:45192-45200`) + the START refusal
`ItemCheck_CheckCanUse_Inner` (`:52051-52054`, reached via
`ItemCheck_TryStartUse` `:52021-52023` BEFORE `StartActualUse`).

| Leg | C# | a6 `ea478936` (`powerups.rs`) | g87 pre-closeout | g87 post-closeout |
| --- | --- | --- | --- | --- |
| START refusal (`extraAccessory \|\| !expertMode` → no swing/sound/consume) | `:52051-52054` | **YES** — `use_start_refused` `:77-85`, called at the press edge `:586` | NO (grant gate only — a refused heart still swung + played the UseSound) | **YES** — `use_start_refused` `permanent_upgrades.rs:127`, called `:176` (a6's shape folded) |
| Grant gate `!extraAccessory && Main.expertMode && ItemTimeIsZero` | `:45194` | `use_demon_heart` `:187-196` (the double gate kept) | `try_use_demon_heart` `:201` (same) | same |
| `ApplyItemTime` BEFORE the flag raise | `:45196-45197` | **order deviation**: `run_powerup_arm` sets `item_time` AFTER the arm returns (`:112`), i.e. after `extraAccessory = true` | faithful (item_time `:214` then flag `:219`) | faithful |
| `reuseDelay = sItem.reuseDelay` tail | `:4496` (via ApplyItemAnimation) | YES `:597-598` | NO | **YES** (folded, `:185-188`) |
| Consumable pay + TurnToAir | `:43280-43326`/`:42824` | `consume_one` `:123-131` (identical body) | `consume_one` `:109` (identical) | identical |
| `SendData(4)` | `:45198` | MP-only drop | MP-only drop | same |

**Verdict: KEEP g87's `demon_heart_item_check` + `try_use_demon_heart` +
`use_start_refused`.** Post-closeout it carries every a6 leg plus the
faithful ApplyItemTime order. **DROP from a6:** `use_demon_heart`
(`powerups.rs:187-196`), `use_start_refused` (`:77-85` — g87's copy is the
survivor; a6's is kind-3335-only, identical semantics), the `3335 =>`
dispatch arm (`:99`), the `3335` entry in `is_powerup` (`:68`), and a6's
three heart witnesses (`demon_heart_unlocks_the_extra_slot_in_expert`
`:860`, `demon_heart_outside_expert_refuses` `:873`,
`a_second_demon_heart_refuses` `:886` — g87's 3 heart witnesses cover the
same legs incl. the no-swing START assertions). Also repoint a6's
`player.rs` doc-comment edit (`extra_accessory` "rides
`powerups::use_demon_heart`") to `permanent_upgrades.rs` at merge — g87's
`player.rs:4871-4877` comment already says LIVE.

### 1b. Minecart Powerup 5289 — KEEP g87, DROP a6's arm; the "flag-gate" hypothesis is REFUTED

C# `ItemCheck_UseMinecartPowerUp` (`Player.cs:45202-45214`): the gate
`:45204` is `type == 5289 && itemAnimation > 0 && ItemTimeIsZero` — **NO
`!unlockedSuperCart` leg**. The unlock raises ONCE (`:45207-45211`), the
`QuickSpawnItem(3353)` runs ALWAYS (`:45212`).

**Both lanes are ALWAYS-spawn.** a6 `bb36e6ee`'s `use_minecart_powerup`
(`powerups.rs:203-211`) has no flag gate either (its own doc-comment: "NO
flag gate — every granted use ... grants the Minecart Mech"). The brief's
"ALWAYS-spawn vs flag-gate" contrast does not exist in the pushed code;
C# `:45204` re-verified at closeout. g87's re-use witness now asserts the
SECOND Mech explicitly (`w4_g87_the_minecart_powerup_unlocks_the_cart_and_spawns_the_mech`,
the re-use leg).

The real delta was **QuickSpawnItem depth**:

| Leg | C# | a6 `bb36e6ee` | g87 pre-closeout | g87 post-closeout |
| --- | --- | --- | --- | --- |
| `Prefix(-1)` on the fresh instance, use's `Main.rand` = UpdatePlayers swap | `:7176`, `Item.cs:528` | YES `:241-242` (UpdatePlayers) | NO — rolled on UpdateItems at the overflow only (recorded deviation) | **YES** (folded, `:294-296`) |
| `GetItem` full deposit walk (ammo pre-pass, occupied merge, hotbar, reverse empty) + per-slot chime + NotifyItemPickup | `:39330-39399`, `:39491`/`:39505` | YES — `item::deposit_pickup` + the closure `:244-258` | NO — bare `slot_for_pickup` loop, no chime/notify | **YES** (folded, `:304-318`) |
| Void vault LAST resort (`CanGoIntoVoidVault: true`, `GetItemSettings.cs:57`) | `:39395-39398` | YES `:260-272` | NO | **YES** (folded, `:320-332`) |
| Overflow at `base.Center`, prefix PRESERVED (no fresh roll) | `:7196-7199` | YES `:275-290` (reforge-return triplet) | partial — `spawn_item_drop_px` re-rolled the prefix | **YES** (folded, `:336-349`) |

RNG census: the Mech 3353 is in NO prefixable set (`itemdata.rs:3519`:
`accessory: false`, no weapon class, `mount_type: 11`;
`prefix::rollable_prefixes` → None), so `Prefix(-1)` draws NOTHING on
either stream — the stream move is census-identical for this kind. The
per-slot chimes draw only when audible (witnesses mute). Both lanes use
the port's pinned reforge velocity triplet (±1.2 / −1.5−r / TAU·r) for the
overflow, not the C# `NewItem` integer scatter (`Item.cs:49744-49750`,
`Next(-30,31)*0.1` / `Next(-40,-15)*0.1`) — the SAME recorded port-wide
deviation on both sides, no adjudication needed.

**Verdict: KEEP g87's `minecart_powerup_item_check` +
`try_use_minecart_powerup` + `quick_spawn_item`** (post-closeout superset).
**DROP from a6:** `use_minecart_powerup` (`:203-211`), `quick_spawn_item`
(`:227-291`), the `5289 =>` arm (`:100`), the `5289` entry in `is_powerup`,
and a6's two cart witnesses (`:908`, `:923` — g87's extended witness covers
both legs).

### 1c. Pet Licenses 4829/4830/4910 — KEEP g87, DROP a6's arm

C# `ItemCheck_UsePetLicenses` (`:45352-45366`) → `LicenseOrExchangePet`
(`:45438-45445`) → `NPC.UnlockOrExchangePet` (`NPC.cs:80894-80918`).

| Leg | C# | a6 `d46221b2` | g87 pre-closeout | g87 post-closeout |
| --- | --- | --- | --- | --- |
| Gate `ItemTimeIsZero && (!bought \|\| AnyNPCs(npcType))` | `:45440` | YES `:415` | YES (same) | same |
| `AnyNPCs` = active && net-id match | `NPC.cs:89873-89883` | **LIVE scan** `:411-414` (`n.hp > 0 && n.kind.net_id() == npc_type`) | hardcoded `false` | **LIVE scan** (folded, `:362-365`) |
| `!bought` guard on the flag raise + license text | `NPC.cs:80904-80909` | YES `:418-426` | NO (re-raised + re-printed on the exchange pass) | **YES** (folded, `:380-389`) |
| Exchange fail line `Misc.PetExchangeFail` | `NPC.cs:80914-80917` | **NO** (returns true silently) | NO | **YES** (`:390-395` — g87 is the superset here) |
| ApplyItemTime + consumable pay on the exchange pass | `:45442` + `:43280` | YES (via run_powerup_arm) | YES | YES |

**Verdict: KEEP g87's `pet_license_item_check` + `try_use_pet_license`.**
**DROP from a6:** `use_pet_license` (`:396-428`), the `4829 | 4830 | 4910
=>` arm (`:105`), the three ids in `is_powerup`, a6's 4 license witnesses.
The exchange POSITIVE path (a resident pet) is unwitnessable on either
lane until a TownCat/Dog/Bunny kind (637/638/656) ships — g87's
`w4_g87_a_resident_non_pet_does_not_open_the_exchange` pins the negative
(a slime does not open the gate).

### 1d. The `.wld` carriers — triple-claimed, byte-identical, dedupe at merge

g87 (`dfbcaf94`), a6 (`d46221b2`) AND f45 (`9c1a01a4`'s comment refresh +
the Ph-sweep) all write `World.bought_cat/dog/bunny`:

* `flags.rs` write: the three `w.boolean(false)` literals →
  `w.boolean(world.bought_*)` — g87 `:212-214`, a6 same lines, f45 `:230-232`
  (same semantics, different base offsets).
* `flags.rs` read gate 217: `r.skip(3)` → three `r.bool()` — g87 `:609-611`,
  a6/f45 same.
* `world.rs` fields + ctor false: g87 `:316-322`/`:695-697`, a6 `:321+`,
  f45 `:405-413`/`:910-912`.
* Golden-fixture tail pins 2021/2022/2023 (`wld/mod.rs`): g87
  `:2064-2066`, a6 same offsets (both cite `WorldFile.cs:1392-1394`;
  re-verified: cat@2021 dog@2022 bunny@2023, empress@2024, queenSlime@2025).

**Verdict: keep ONE copy** (any — they are semantically identical; the
merger takes whichever base lands first and drops the other two's hunks).
g87's `owned_world` round-trip extension + a6's gate-217 synthetic-file
witness (`d46221b2`'s `wld/mod.rs:3676+` hunk) are COMPLEMENTARY — keep
both witnesses on the surviving carrier.

### 1e. What a6 keeps (NOT this lane's call, recorded for 138)

* `drone_destroy_check` (`powerups.rs:532+`, tip commit `8ed96ee5`): Jim's
  Drone 5451 + Remote Control Car 5738 destroy arms (`Player.cs:45245-45272`,
  the `:52067-52074` CheckCanUse legs) — **unique to a6, KEEP**.
* The crystal/fruit/loaf/torch/book/satchel/shimmer arms: g88's
  research/103 §5a already adjudicated the crystal/fruit keep-own; the
  shimmer keep-one is 138's call (103 §5a defers to it). g87's verdict
  extends the same pattern: **the merger drops a6's `powerups.rs` use-arm
  cluster wholesale EXCEPT `drone_destroy_check`**, keeps g87's
  `permanent_upgrades.rs` (3 families), g88's `crystals.rs`/`life_fruit.rs`,
  g90's loaf/book/satchel/shimmer, f33's torch. a6's `powerup_item_check`
  machine + `is_powerup` + `run_powerup_arm` + `consume_one`/`ticks_of`/
  `use_time_secs` twins all drop with the cluster (g87/g88 carry the same
  helpers; the merge dedupes onto ONE shared machine per 103 §6 rule 1's
  spirit).
* a6's `editing.rs` dispatch (`:2013` `powerup_item_check` call + `:2006`
  `drone_destroy_check`): keep ONLY the drone call; g87's three calls
  (`editing.rs:2019/:2022/:2025`, the `:43598/:43599/:43606` C#-order
  slots) survive. The full `:43595-43607` ladder reassembles at merge from
  g88 (crystals/fruit) + g87 (heart/cart) + f33 (torch) + g90 (loaf/books/
  satchel/shimmer) + g87 (licenses).

---

## 2. The `dh` sentinel line → g88f fold-input (research/103 §6 rule 5)

**My line (g87 `persist.rs`):** writer v7 (`:64` header `"7\n"`), ONE new
post-dye line appended directly after the dye line (`:84-95`):

```
dh <extraAccessory> <unlockedSuperCart>      (3 tokens: sentinel + u8 + u8)
```

`SavedPlayer.extra_accessory` (`:36`) / `.unlocked_super_cart` (`:41`);
reader `:169-179` (optional, sentinel-checked, missing/non-dh tolerated →
false); App restore `app/mod.rs:2944-2945` (after the hp line, before
coins). The `.plr` halves are f44's G7 lane (`player_file.rs:737` gate 125
extraAccessory, `:1099` super-cart BitsByte gate 253) — NOT the sidecar.

**The g88 superset (tip `c8985b53`, writer v10):** unlock line (6 tokens) →
life pair → mana pair → bread → loadout index + 3×41-token sets. Per 103
§6 rule 5 a new post-dye line appends AFTER the loadout block with a
self-identifying shape. `dh` qualifies: 3 tokens whose first is the
non-numeric sentinel `dh` — distinct from the 1/2/41 numeric shapes AND
from any future numeric 3-token line.

### The exact fold hunks (g88f)

1. **`SavedPlayer`**: append the two bool fields (g87 `persist.rs:30-41`
   doc-comments + decls verbatim).
2. **Writer** (g88 `persist.rs`, after the loadout `for` loop ending
   `:256`, before `std::fs::write` `:257`):
   ```rust
   // W4-g87: the Demon Heart / Super Cart unlock line — the self-
   // identifying `dh` sentinel (research/103 §6 rule 5, research/263 §2).
   s.push_str(&format!(
       "dh {} {}\n",
       u8::from(player.extra_accessory),
       u8::from(player.unlocked_super_cart),
   ));
   ```
   **VERSION stays 10.** The line is optional on read and self-identifying;
   103's v11 reservation is the unlock line's DD2 token 7, a different
   axis. No released v10 file carries a `dh` line, so no back-compat cost.
3. **Reader** (g88 `persist.rs`, the shape walk `match toks.len()`
   `:409-448`): add a sentinel arm BEFORE the catch-all, in BOTH the v7
   and v8+ walks (the sentinel is unambiguous at any version):
   ```rust
   3 if toks[0] == "dh" => {
       extra_accessory = toks.get(1).is_some_and(|t| *t == "1");
       unlocked_super_cart = toks.get(2).is_some_and(|t| *t == "1");
   }
   ```
   (declare the two `let mut` false defaults beside `:358-367`; return
   them in the `Some(SavedPlayer{..})` literal `:451+`).
   **Without this arm g88's CURRENT reader tolerates-but-drops the line**
   (3 tokens → `_ => {}` `:447`) — a g87-branch v7 file loaded by the
   un-folded superset loses both flags silently. The arm is the fold.
4. **App restore** (g88 `app/mod.rs`, the `persist::load` block): insert
   g87's two lines (`p.extra_accessory = saved.extra_accessory;
   p.unlocked_super_cart = saved.unlocked_super_cart;`) after the loadout
   restore (`:3004`) and BEFORE the hp clamp (`:3007`) — the flags do not
   touch hp, but the block's order convention keeps every restore ahead
   of the clamp.
5. **Back-compat rows for 103 §4** (witnesses to port from g87
   `persist.rs::tests`):
   * g87 dh-only v7 (`w4_g87_the_demon_heart_and_super_cart_unlocks_round_trip`):
     loads under the folded v7 walk, both flags read; pairs None, bread
     false, loadouts empty.
   * v6 no-dh (`w4_g87_a_v6_sidecar_loads_the_unlocks_false`): both false.
   * the full v10 superset + dh round-trip: extend
     `the_superset_round_trips_the_full_block` with the trailing `dh 1 1`
     line assertion.
   * a v10 file WITHOUT the dh line (every pre-fold superset file): both
     false — the optional-read contract.
6. **103 §2 layout table**: add row `16 | the dh unlock line | 3 | dh
   extraAccessory unlockedSuperCart | Player.cs:54776/:55235 (gate 125) +
   :54914-54918/:55710-55715 (gate 253) | LIVE (g87, folded)`. §5 carrier
   table: `Player.extra_accessory` (pre-existing decl, g87 writer) /
   `Player.unlocked_super_cart` (pre-existing `player.rs:1792`, g87
   writer); consumers: the slot-8 gate (`player.rs:4878` +
   `equip_swap.rs:104`) + the Super Cart ride (`player.rs:9501`).

---

## 3. The `.trwld` v18 3-byte tail → g71c union-input

**My block (g87 `save.rs`):** `VERSION = 18` (`:71`); writer `:438-442`
appends **3 bytes** directly after the v17 `downedFishron` byte (`:433`);
reader `:826-830` (`if version >= 18`):

| Offset in MY v18 block | Byte | Field | C# `.wld` anchor |
| --- | --- | --- | --- |
| 0 | u8 0/1 | `World.bought_cat` | `WorldFile.cs:1392` write / `:2358` read gate 217 |
| 1 | u8 0/1 | `World.bought_dog` | `:1393` / `:2359` |
| 2 | u8 0/1 | `World.bought_bunny` | `:1394` / `:2360` |

Pre-v18 loads all three false (the vanilla's own reset arm
`WorldFile.cs:2364-2366`).

### The union extension (input to g71c / the g70⨉g71⨉g90⨉g87⨉f45 merge)

Research/92 §1 fixes the g70⨉g71 union at **19 bytes** (forest-first);
research/135 §4 appends g90's book/satchel pair at [19]/[20] → 21 bytes.
**This lane's 3 bytes slot BETWEEN them by C# field order**: the pets
(`WorldFile.cs:1392-1394`) precede the book-two/satchel pair
(`:1407-1408`) in `SaveWorldFlags`, and both follow the setBG/treeBG bytes
(`:1318-1379`). 92 §1's "least-surprising order = C# field order" rationale
therefore moves 135's two offsets:

```
… v17 downedFishron byte (1)
v18 UNION tail (24 bytes), offsets from the start of the v18 block:
  [0..16]  g70 forest-set block   (tree_x 3×i32 LE + forest_bg_styles 4×u8)  — 92 §1
  [16]     snow_bg_style    u8    ┐ g71 surface bytes                        — 92 §1
  [17]     desert_bg_style  u8    │
  [18]     ocean_bg_style   u8    ┘
  [19]     bought_cat       u8    ┐ g87 pet licenses (THIS DOC)
  [20]     bought_dog       u8    │
  [21]     bought_bunny     u8    ┘
  [22]     combat_book_volume_two_was_used  u8   ← g90 (135 §4's [19] MOVES here)
  [23]     peddlers_satchel_was_used        u8   ← g90 (135 §4's [20] MOVES here)
```

**No back-compat cost**: no released v18 exists (92's forward-only rule —
every v18 claimant is an unmerged branch), so the reorder is free. 135 §4's
table is superseded on offsets [19]/[20] → [22]/[23]; its writer/reader
snippets stay verbatim, just appended after the three pet reads.

**The FOURTH v18 claimant the 92/135 contracts do not know: f45's
world-identity block (44 bytes).** `origin/task/W4-f45-wld-header-2`
`save.rs:438-446` appends generator_version u64 + unique_id [u8;16] +
world_id i32 + creation_time i64 + last_played i64 (the `.wld` header
fields `WorldFile.cs:1225-1227`/`:1250-1251`, W4-c43's port). It is
header-shaped, not flags-shaped; the vanilla writes it at the file HEAD,
but the port's `.trwld` tails are forward-only append. **Recommendation
for g71c: identity LAST** (it is the only non-flags block; keeping the
flags-shaped bytes contiguous preserves the 92/135/263 offset tables):

```
  [24..68] f45 identity block (8 + 16 + 4 + 8 + 8 = 44 bytes)
```

**Full union: 68 bytes** (19 bg + 3 pets + 2 book/satchel + 44 identity).
`VERSION` stays **18** across all five claimants.

Writer (after the 92 §1 union writes + this doc's pet trio + 135's pair):

```rust
w.write_all(&[
    u8::from(world.bought_cat),
    u8::from(world.bought_dog),
    u8::from(world.bought_bunny),
])?;
```

Reader (inside the same `if version >= 18` arm, after the 19 union reads,
before 135's pair):

```rust
world.bought_cat = r.u8()? != 0;
world.bought_dog = r.u8()? != 0;
world.bought_bunny = r.u8()? != 0;
```

### Truncation-constant moves on THIS branch at the union merge

Every site currently counts my `+3`; the full union is `+68` (or `+24`
if g71c lands the flags union before f45's identity — the table gives
both):

| Site (`save.rs`) | Current (g87) | Flags union (24) | Full union (68) |
| --- | --- | --- | --- |
| `:1300` v12 `let tail = …` | `… + 5 + 1 + 3` | `… + 5 + 1 + 24` | `… + 5 + 1 + 68` |
| `:1329` `EMPTY_V16_TAIL` | `EMPTY_V13_TAIL + 9 + 1 + 5 + 1 + 3` | `… + 1 + 24` | `… + 1 + 68` |
| `:1518` v14 queen-slime probe | `len - 1 - 5 - 1 - 3` | `len - 1 - 5 - 1 - 24` | `len - 1 - 5 - 1 - 68` |
| `:1542` pre-v18 probe in `w4_g87_the_pet_license_flags_round_trip_in_v18` (`:1528`) | `len - 3` | `len - 24` | `len - 68` |
| `:1238` v11 probe (`- 10 - EMPTY_V16_TAIL`) | rides the constant | rides | rides |
| `back_flags`/`back_micro` (`:1331-1341`) | ride `EMPTY_V16_TAIL` | ride | ride |

The lenient v6/v5 downgrade probes (`:1188`, `:1564`) over-keep trailing
bytes and rely on the version gate (the reader stops at the v6/v5 block);
they pass unchanged — but g71c MAY tighten them to the exact union length
for hygiene (135 §4's `:1194`/`:1576` rows assume the strict form; on THIS
base the two probes are the lenient form, recorded so the merger does not
"fix" a non-bug). The version-gate fixture
(`dw_1_4_version_above_4_rejected`, `:1106` region) keeps its 18→19 repin;
the writer-emits-18 pins stay. g70's/g71's/g90's own round-trip witnesses
move 16/3/2 → 24 (or 68) per the same rule.

**`.wld` side (independent of the union, already landed here):** §1d's
carriers — write `flags.rs:212-214`, read gate 217 `:609-611`, tail pins
`wld/mod.rs:2064-2066` (2021/2022/2023), the `owned_world` round-trip
extension. The golden fixture's divergence set is unchanged (the three
bytes are 0 in the golden world).

---

## 4. The slot-8 consumer census (the extraAccessory writer is live — who draws it?)

**Census verdict at closeout base (`dfbcaf94`):** the SIM consumers were
live, the DRAW consumer was hardcoded dead.

| Consumer | C# anchor | Port site (base) | Status at base | Closeout |
| --- | --- | --- | --- | --- |
| `UpdateEquips` functional-accessory grant, slot 8 | `Player.cs:12946-12952` + `:12587` | `player.rs:4878` `is_item_slot_unlocked_and_usable` (8\|18 → extra && expert) + `:4896` loop | LIVE (S1-H-5/i48) | unchanged (made `pub` for the UI) |
| Equip-swap empty-slot scan row 8 | `ItemSlot.cs` ladder | `equip_swap.rs:104` (same gate) | LIVE | unchanged |
| Equipment-column DRAW (armor 8 / vanity 18 / dye 8) | `Main.cs:41381-41382` flag4/flag5 + `:41398`/`:41471`/`:41504` skip | `ui/inventory.rs` `DEMON_HEART_SLOT: bool = false` / `MASTER_SLOT = false` | **DEAD — hardcoded hidden** | **PORTED** (`236c0d5a`) |
| Row count `num19 = 8 + GetAmountOfExtraAccessorySlotsToShow()` | `Main.cs:41145`, `Player.cs:4294-4306` | `EXTRA_ACCESSORY_SLOTS: i32 = 0` | **DEAD** | **PORTED** — `ExtraAccessoryShow::count()` |
| Show predicate (unlock OR stored armor[8]/armor[18]/dye[8]) | `Player.cs:4276-4283` | none | **MISSING** | **PORTED** — `Player::can_demon_heart_accessory_be_shown` (`player.rs:4891`) + the master twin `:4904` |
| Shown-but-locked dim background `(80,80,80,80)` | `Main.cs:41393` color2 + `:41452`/`:41493`/`:41521` | none (every row drew `inventoryBack`) | **MISSING** | **PORTED** — `ExtraAccessoryShow::slot_unlocked` + the per-column `back` |
| Locked-row interact gate `flag6 \|\| mouseItem.IsAir` | `Main.cs:41450`/`:41491`/`:41519` (`ItemSlot.Handle` 4th arg, `ItemSlot.cs:588-597`) | none (every hit clicked) | **MISSING** | **PORTED** — `clicks.rs` equip-cell dispatch |
| Hide-visual tick row 8 | `Main.cs:41425-41460` | `accessory_tick_rects` skipped idx 8 | **DEAD** | **PORTED** — the 6th tick; `hideVisibleAccessory[8]` flips live |
| Defense counter anchor (last accessory row) | `Main.cs:41421-41424` (`num35 = num19 - 1`) | pinned to row 7 | **STALE** | **PORTED** — drops to row 8 when shown |
| Settings-button avoidance count | `Main.cs:45508` | `extra_accessory_slots()` → 0 | **DEAD** | **PORTED** — `frame.rs` reads the LIVE count |
| `over_ui_slot` probe / housing base_y | `Main.cs:41145` | `equip_cell_at`/`equip_page_base_y` | geometry-only | re-threaded through the snapshot |
| Super Cart ride gate | `Player.cs` mount speed | `player.rs:9501` (`unlocked_super_cart && enabled_super_cart`) | LIVE | unchanged |
| World-space accessory VISUALS (wings/balloons/shields on the sprite) | `PlayerDrawLayers.cs` acc layers | `render/player_layers.rs` — the composite machine draws head/body/legs + mount ONLY, for EVERY accessory slot | **UNPORTED system-wide** | **NOT a slot-8 gap** — no per-slot draw port exists to extend; recorded, out of scope |
| `.plr` halves | `Serialize:54776` / `:54914-54918` | f44 `player_file.rs:737` (gate 125) / `:1099` (gate 253) | f44's lane | unchanged |
| Net `hideVisibleAccessory` | `NetMessage` player-info | `net.rs:919` decode-only u16 | pre-existing | unchanged (10 bits fit) |

**The port (commit `236c0d5a`):** `Player.CanDemonHeartAccessoryBeShown` /
`CanMasterModeAccessoryBeShown` / `GetAmountOfExtraAccessorySlotsToShow`
(`Player.cs:4276-4306` verbatim; the C# `type <= 0` reads map to the
port's kind-0-empty `equipped`/`dye_slots` snapshots) +
`ui::inventory::ExtraAccessoryShow` (flag4/flag5 + the flag6/7/8 unlock
bits, `of(player)` per-frame snapshot) threaded through
`equip_page_base_y` / `accessory_tick_rects` / `accessory_tick_click` /
`equip_cell_at` / `draw` and every caller (`ui/mod.rs`, `app/frame.rs`,
`app/clicks.rs` incl. the tick + equip-cell dispatch, `app/frame/world.rs`
over_ui_slot, the housing base_y). Master mode stays unported: slot 9
shows only while a stored item occupies it (the C# predicate's item legs,
faithful). 3 new witnesses + 6 re-threaded geometry witnesses, written
not run.

---

## 5. Witness ledger (this closeout, all WRITTEN NOT RUN)

* `permanent_upgrades.rs` (8): the expert grant + once-ever START refusal;
  the classic no-swing refusal (rewritten); the used-heart START refusal
  (new); the cart unlock + ALWAYS re-use second Mech (extended); the cat
  license once + text; the dog/bunny dispatch; the resident-non-pet
  exchange refusal (new); the zero-draw RNG census incl. the cart
  (extended).
* `player.rs` (1 new): `w4_g87_the_demon_heart_row_show_predicate` (every
  leg of `:4276-4292`).
* `ui/inventory.rs` (2 new + 4 re-threaded):
  `w4_g87_the_demon_heart_row_joins_the_columns` (row 8 in all three
  columns, the 6th tick at y=556, slot 9 stays hidden),
  `w4_g87_a_shown_locked_row_dims`; the four geometry witnesses carry the
  `NO_EXTRA` snapshot (values unchanged).
* Persist/.wld/.trwld witnesses: unchanged from `dfbcaf94` (3 + the
  owned-field round-trip + the v18 round-trip); §2/§3 give the fold-side
  witness rows for g88f/g71c.

Verification: `rustfmt --edition 2024 --check` on every touched file —
the closeout's diff-count vs HEAD is non-positive per file (the repo
carries pre-existing fmt drift; the lane does not widen it). No cargo, no
tests run, no graphify (lane constraint).

---

## 6. RETIRE (g87c, 2026-11-21) — the post-closeout receipts, the renumber, the FINAL demonheart-family bible

**Status: RETIRE/FINAL.** §1-§5 above stand AS PUBLISHED at `e3f81d0b`
(the closeout record); this section carries every later authority that
supersedes their fold tables, the registry renumber, the Success-leg
g87-side hunk, and the family declaration. Tips read at THIS pass (fetched
2026-11-21, `git fetch origin --prune`):

| Lane | Tip read | What was read |
| --- | --- | --- |
| a6 `task/W4-a6-crystals-powerups` | `0766c8df9` (ARBITER receipt) | research/138 FULL (§10.1-10.5, §11.3, §12.1-12.5) + `powerups.rs` fail-chat arm (`:447`) |
| g88 `task/W4-g88-life-fruit-use` | `5469d5872` (SEAL-VERIFY, A5) | research/103 §5a/§9 + L3-ARBITER A1-A4 + the SEAL bible; `persist.rs` `_ => {}` tolerate arm verified at the v11 tip |
| f44 `task/W4-f44-save-plr-2` | `a46e798f3` (SEAL-VERIFY) | research/132 §2e/§2f/§8; `Player::hydrate_from_file` (`player.rs:2516`, `extra_accessory` `:2524`, `super_cart & 1` `:2548`) — the L1 carrier is LIVE on load |
| g71 `task/W4-g71-bg-layers-2` | `b27474a44` (BG-SEAL RECEIPT) | research/92 §1-FULL.2 (A3-corrected) + §1-FULL.4 site table (g87 column) |
| f45 `task/W4-f45-wld-header-2` | `82a006dcf` (REGISTRY-RENAME) | research/137→**279**-wld-final-audit (the .wld ownership table + SEALED banner) |
| g90 `task/W4-g90-artisan-bread-use` | `43fee29a4` (BOOSTER SEAL) | research/135 s10g (books [61]/[62] on A3) |
| f43 `task/W4-f43-town-spawn-2` | `79a037f5e` (census, UNCHANGED) | `town_arrival_pets.rs` full re-read: `license_gate`/`unlock_or_exchange_pet`/`reroll_variation`/`reroll_variation_for_npc_type`/`PetVariationSlot` |
| g10 `task/W4-g10-npc-kind-wire-3` | `5dee28650` | the ALL-11 town-pet kinds WIRED (138 §12.2's census receipted) |
| d54 `task/W4-d54-dust-sites-world` | `72226e500` (registry round 2) | research/150 §6.1 key-143 row + §6.3's g87 rename instruction |

C# re-reads this pass (direct paths, ≤100-line windows, no junction
recursion): `NPC.cs:80880-80969` (`UnlockOrExchangePet` +
`RerollVariationForNPCType` + `RerollVariation` verbatim),
`IO/WorldFile.cs:1385-1410` (the pets :1392-1394 → empress :1395 → books
:1407-1408 write rows). No cargo/tests/rustfmt/graphify (lane constraint);
witnesses WRITTEN NOT RUN.

### 6.1 The renumber receipt (143 → 263)

The registry (research/150) round 1 (`38bfd1d22`) ruled key 143 ONE-SIDED
for c4; round 2 (`72226e500`, §6.1) re-scanned it as a FULL 6-way collision
and assigned **g87 fold-input → 263** (§6.3's instruction row, refs
`docs/ANALYSIS.md` + `docs/research/INDEX.md`). This pass EXECUTES it: the
file renamed (`git mv`, the RENUMBERED banner at the head), and every
citation re-pinned — ANALYSIS ×4 (incl. the filename cite), PORT_MAP ×1,
INDEX row, `permanent_upgrades.rs` ×7 doc/witness comments. The doc's own
internal §-cites ride the banner. d120's 143→261 and c8's 143→238→262 are
those lanes' own executions (registry rule: own branch only). **263 is
this doc's FINAL number.**

### 6.2 The dh sentinel line — g88h DEFAULT-DROP CONFIRMED (supersedes §2's fold hunks)

§2 published the g88f fold (line 17 / sentinel arm / VERSION stays 10) at
the g88 v10 tip `c8985b53`. The authorities have since RULED:

* **a6 138 §10.4: DROP** — the version collision (g88 owns v7-v11) + the
  L1-only ruling (132 §2a rows 18/64).
* **f44 132 §2f item 2 (SEALED): the DROP is the merger DEFAULT.** The
  §2e line-17-at-v12 fold survives only as the CONDITIONAL seat (in-session
  dual-carrier before the exit-save); under the default the v11 writer
  stands and **DD2 keeps its v12 token-7 reservation** (132 §2e's v13
  re-seat VOID). g88 103 §5a/§9 receipt the same.
* **The DROP is SAFE on the published record:** f44's hydration port
  landed (`5c8bbb160`, `Player::hydrate_from_file` applies
  `extra_accessory` + the super-cart BitsByte split at load — condition
  (a) of 103 §2b MET); the v11 superset reader tolerates this branch's v7
  `dh` line lossy-not-corrupt (the 3-token line falls to the `_ => {}`
  arm — re-verified at the g88 tip this pass); the remaining condition (b)
  (the exit-save `.plr` re-write) is f44's recorded open remainder, NOT
  this lane's.

**This branch keeps its v7 writer/reader AS LANDED** (`persist.rs:64`/
`:84-95`/`:169-179` + the two round-trip witnesses) — they are the
in-session carrier until the merge; at the merge the whole `dh` cluster
(SavedPlayer fields, the v7 bump, the reader arm, both witnesses, the
`app/mod.rs:2944-2945` restore pair) DROPS per 138 §10.4 / 132 §2f, and
the flags ride f44's hydration (character-first, sidecar-second
precedence, 132 §2c). §2's hunk list is SUPERSEDED — the merger does NOT
apply it under the default. The dh-fold witnesses §2 item 5 lists are
VOID under the default (they activate only with the conditional seat).

### 6.3 The pet-license Success leg — the g87-side hunk (a6 138 §12.2's contract)

§1c's "unwitnessable until a TownCat/Dog/Bunny kind ships" boundary is
STALE at the merged horizon: **g10 `5dee28650` landed ALL-11 town pets
WIRED** (637/638/656/670/678-684; the live `AnyNPCs` scan ANSWERS on g10's
tip). The one remaining use-arm debt is the **Success dispatch**
(`NPC.cs:80910-80912`, `Misc.PetExchangeSuccess`), shared a6 × g87, with
the three-leg activation contract (138 §12.2):

1. **g10 kinds — DONE** (`5dee28650`).
2. **f43-splice (the f43b lane)** — splice `tail_can_spawn`'s pet arms +
   expose a wired `reroll_variation_for_npc_type`
   (`town_arrival_pets.rs:150-184`) over the live `Npc` roster
   (`PetVariationSlot` adapter: active/npc_type/variation_index/hp/max_hp);
   the variant roll on `Main.rand` = `RngKey::UpdatePlayers`
   (`Profiles.cs:216` `Next(6)` — the ONLY Main.rand slot in the chain),
   the name draw on `WorldGen.genRand` (a DIFFERENT stream).
3. **The else-arm one-line dispatch** — a6's published hunk (138 §12.2)
   swaps the hardcoded Fail chat for the reroll result.

**g87's arm takes the SAME hunk** — the exact g87-side seat is now recorded
IN CODE (`permanent_upgrades.rs`, the `try_use_pet_license` else-arm
comment): `success = reroll_variation_for_npc_type(&mut slots, npc_type,
&mut |n| roll_rng.next_max(n))` then the
`if success { "Misc.PetExchangeSuccess" } else { "Misc.PetExchangeFail" }`
key select, World colour, consume either way (`:45442`), no flag re-raise,
no license re-print. **g87 is ALREADY the PetExchangeFail superset** (§1c:
the Fail chat a6 lacked landed here first at `26620538`; a6 reciprocally
folded it, 138 §11.3 — verified at `0766c8df9` `powerups.rs:447`), so the
Success hunk is the ONLY behavioural delta left on either arm. NOT wired
on this branch (f43's chain is UNWIRED pure fns at `79a037f5e`; the splice
is f43b's lane — wiring here would duplicate it). The positive Success
witness lands WITH the f43b splice + this hunk (138 §12.2's boundary);
this branch's `w4_g87_a_resident_non_pet_does_not_open_the_exchange`
stays the negative pin. The C# legs re-verified verbatim this pass
(`NPC.cs:80894-80961`: the first-resident-only scan `:80922-80930`, the
no-profile false `:80935-80938`, the ≤100-draw cap `:80941-80944`, the
`GetNameForVariant` + `life = lifeMax` heal `:80949-80950`, the
PetExchange particle + `TrySendData(56)` `:80951-80959`).

### 6.4 The .trwld pets seat — 107-A3 RE-PIN (supersedes §3's 24/68-byte tables)

§3's union extension (pets [19..22], books [22]/[23], identity LAST,
68-byte full union) was the FOUR-claimant view at closeout. The union has
since been arbitrated at THIRTEEN claimants: **g88 research/103
§L3-ARBITER FINAL A3 (`d09d7e10a`, A5-verified `5469d5872`) is THE
authority** — the 107-byte fixed head + W×H dense LAST, VERSION 18
forward-only, the flag middle in STRICT C# row order. This lane's pets
seat:

```
[57] bought_cat   u8   (WorldFile.cs:1392)
[58] bought_dog   u8   (:1393)
[59] bought_bunny u8   (:1394)
```

— after crimson [50] / moon quintet [51..56] / underworld [56], BEFORE
empress [60] and books [61..63]; identity 44 B [63..107] closes the head.
The C# rows re-read verbatim this pass (`:1385-1410`). §3's [19..22] seat
and its "135's offsets move to [22]/[23]" instruction are SUPERSEDED: g90's
pair sits [61]/[62] (135 s10g, on A3), f45's identity is NOT last-flag-
shaped but head-closing [63..107] — the four-claimant arithmetic did not
know the crimson/moon/underworld/cave blocks. **The truncation-move table
(§3) re-pins 3 → 107** per 92 §1-FULL.4's g87 column (order-agnostic, the
sites stand): `save.rs:1300` (`+3`→`+107`), `:1329` EMPTY_V16_TAIL
(`+3`→`+107`), `:1518` (`-3`→`-107`), `:1542` (`len-3`→`len-107`); the
mid-record cuts `:1093`/`:1700`/`:2163` re-arm `-3`→`-3-107` per the same
table; `:1238`/back_flags/back_micro ride EMPTY_V16_TAIL. **NOT executed
on this branch** — the branch keeps its own 3-byte tail (every claimant
does; the union lands ONCE at the g71c merge, forward-only, zero
back-compat cost). The `.wld` side (§1d/§3 tail) is UNCHANGED: f45's
flags superset keeps the write/read-gate-217 bytes (137→279's ownership
table), this lane's flips dedup, the 2021/2022/2023 golden pins stand.

### 6.5 The a6 keep-one direction — 138's verdict RECEIPTED (supersedes §1's KEEP-g87 direction)

§1 adjudicated KEEP-g87/DROP-a6 per family from THIS lane's side. a6's
138 (§10.1-10.3, §11.3, §12.3 SEALED + ARBITER) adjudicated the mirror:
**a6's single-seam `powerups.rs` machine owns the use-arm cluster; g87's
three arms DROP at merge.** Both verdicts agree on the SEMANTICS (the
closeout made all three families shape-identical — 138 §11.3's
cross-receipt confirms every fold leg by leg); they differ only on WHICH
copy survives, and the SEALED merger authorities (103 §9 step 8 / 132 §8
step 2 item 3 / 138 §12.3) all seat the survivor on **a6's machine**.
This lane receipts that direction: at the merge
`permanent_upgrades.rs`'s three use arms + their 8 witnesses dedup onto
a6's `powerups.rs` (g87's witness ASSERTIONS survive on the surviving
arm — the no-swing START pins, the second-Mech re-use leg, the
resident-non-pet refusal, the zero-draw census); the `editing.rs:2019-
2025` dispatch calls dedup onto a6's `:2013` seam at the same C#-order
slots. **What g87 UNIQUELY keeps** (no a6 twin): the slot-8 UI/draw port
(`236c0d5a` — `Player.can_demon_heart_accessory_be_shown` +
`ui::inventory::ExtraAccessoryShow` + the 3 UI witnesses; 138 has no UI
scope), the `.wld`/`.trwld` carrier flips (dedup per §1d/§6.4), and the
dh-sidecar cluster until its §6.2 DROP. The §1e "a6 keeps
drone_destroy_check" note stands (138 §12.3's bible seats the drones on
a6 too).

### 6.6 The demonheart-family bible (FINAL, one paragraph)

The Demon Heart family seals as: **use arms** — a6's `powerups.rs`
machine owns 3335 (with the CheckCanUse START refusal g87 folded), 5289
(ALWAYS-spawn + the full GetItem QuickSpawnItem walk, `Prefix(-1)` on
UpdatePlayers, zero draws — the flag-gate hypothesis REFUTED on both
lanes) and 4829/4830/4910 (the live AnyNPCs scan + `!bought` guard + the
PetExchangeFail chat both lanes carry; the Success leg is the ONE
remaining debt, activation-contracted g10-DONE → f43b-splice → the
one-line dispatch, §6.3); **flags** — `Player.extra_accessory` /
`unlocked_super_cart` are L1 `.plr` fields (f44's complete walk +
hydration LIVE, gate 125 / super-cart BitsByte 253), the g87 v7 `dh`
sidecar line DROPS at merge under the 132 §2f DEFAULT (the conditional
line-17-at-v12 seat stays published, DD2 keeps v12); **world flags** —
`bought_cat/dog/bunny` ride f45's `.wld` flags superset (gate 217, golden
pins 2021-2023) + the 107-byte `.trwld` v18 union at [57..60] under 103
L3-ARBITER A3; **consumers** — the slot-8 sim gate (`player.rs:4880` +
`equip_swap.rs:104`, S1-H-5/i48) and the slot-8 UI row (g87's `236c0d5a`
port, `Player.cs:4276-4306` + `Main.cs:41381-41524`) are LIVE; the Super
Cart ride gate (`player.rs:9501`) is LIVE; the world-space accessory
visuals stay unported SYSTEM-WIDE (not a slot-8 gap). Every carrier has
one owner, every arm one survivor, one debt one contract — the family is
RETIRED on this branch; nothing further is owed here.

### 6.7 Retire verification

* Registry: research/150 round 2 (`72226e500`) §6.1/§6.3 read FULL; the
  rename executed per its instruction row (file + banner + ANALYSIS ×4 +
  PORT_MAP ×1 + INDEX ×1 + code comments ×7); the stale-name grep
  (the old `143-` prefix joined to this doc's slug) returns ZERO after
  the pass (the banner's historical chain mentions excepted).
* The §6.2/§6.4/§6.5 supersessions verified against the CURRENT tips
  (table above), not the closeout-era tips; the C# rows re-read directly.
* The §6.3 hunk is comment-only in code (the else-arm's contract note);
  no behaviour changed on this branch this pass — the 8+3+3 witnesses
  and every landed pin stand as written (WRITTEN NOT RUN).
* No cargo/tests/rustfmt/graphify invoked (lane constraint). Comment-only
  code delta; docs + the rename are the pass's whole diff.