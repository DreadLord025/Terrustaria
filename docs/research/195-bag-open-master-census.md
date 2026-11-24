//! W4-f35 DOC-SWEEP — the bag OPEN mechanics + the master-mode bag
//! drop rates: the census.
//!
//! Companion to `184-expert-drop-trees-census.md` (the per-boss grant
//! trees). This doc sweeps the MECHANICS around the open — the
//! right-click dispatch, the luck question, the death-side rule gates
//! and the master-mode rates — and records the three stream-fidelity
//! fixes the sweep landed.
//!
//! ## 1. The open dispatch (the right-click mechanics)
//!
//! `ItemSlot.RightClick` (`UI/ItemSlot.cs:1636-1715`) reaches the open
//! through the `OpenableBag` arm (`:1660-1667`):
//!
//! ```text
//! if (Main.mouseItem.IsAir || !Item.CanStack(inv[slot], Main.mouseItem))
//!     if (!PlayerInput.UsingGamepadUI && context == 0
//!         && ItemID.Sets.OpenableBag[inv[slot].type])
//!         if (Main.mouseRightRelease)
//!             ItemSlot.TryOpenContainer(inv, context, slot, player);
//!         return;
//! ```
//!
//! - the gate is an AIR cursor OR one that cannot stack with the cell
//!   (`Item.CanStack` = same kind AND same prefix, `Item.cs:49865`);
//! - context 0 only (the 10x5 grid; the coin/ammo/equip rows do not
//!   open bags);
//! - the PRESS EDGE (`Main.mouseRightRelease`) — a held button does not
//!   re-open; the arm RETURNS on every held frame too (no split pickup
//!   while a bag sits under the pointer);
//! - the arm runs BEFORE the `HasItemSwap` cycle (`:1668`) and the
//!   `SwapEquip` ladder (`:1685`), so a bag right-click opens instead of
//!   swapping/equipping;
//! - the gamepad path (`:3983-3991`, the Grapple trigger) reaches the
//!   same `TryOpenContainer` — unported (no gamepad UI).
//!
//! `TryOpenContainer` (`:1786-1804`): `TryOpenContainer_GrantItems`
//! FIRST, then the bag stack decrements (`:1795-1799`, at 0 →
//! `SetDefaults(0)`), sound 7 (`:1800`), `stackSplit = 30` (`:1801`),
//! `mouseRightRelease = false` (`:1802`). A refused key gate (the Lock
//! Box / Shadow Lockbox) returns false → NO decrement, NO sound, NO
//! cooldown.
//!
//! `TryOpenContainer_GrantItems` (`:1806-1865`) is the else-if dispatch
//! over `ItemID.Sets.OpenableBag` (`ItemID.cs:1384`, 58 kinds):
//!
//! | Gate | Kinds | Opener | C# |
//! | --- | --- | --- | --- |
//! | `ItemID.Sets.BossBag` | 21 (3318-3332, 3860/3861/3862, 4782, 4957, 5111) | `Player.OpenBossBag` | `Player.cs:7202-7883` |
//! | `ItemID.Sets.IsFishingCrate` | 26 (2334-2336, 3203-3208, 4405/4407, 4877, 5002 + the hardmode 3979-3987, 4406/4408, 4878, 5003) | `Player.OpenFishingCrate` | `:8028-9250` |
//! | `type == 3093` | Herb Bag | `OpenHerbBag` | `:9367-9441` |
//! | `type == 4345` | Can of Worms | `OpenCanofWorms` | `:9443-9455` |
//! | `type == 4410` | Oyster | `OpenOyster` | `:9457-9476` |
//! | `type == 1774` | Goodie Bag | `OpenGoodieBag` | `:9502-9585` |
//! | `type == 6142` | Chillet Egg | `OpenChilletEgg` | `:9491-9500` |
//! | `type == 3085` | Lock Box (CONSUMES a Golden Key 327) | `OpenLockBox` | `:9330-9365` |
//! | `type == 4879` | Shadow Lockbox (needs a Shadow Key 329 PRESENT) | `OpenShadowLockbox` | `:9310-9328` |
//! | `type == 1869` | Present | `OpenPresent` | `:7031-7150` |
//! | `type == 599/600/601` | legacy Presents | `OpenLegacyPresent` | `:7152-7168` |
//!
//! The port: `app/clicks.rs` — the `OpenableBag` arm
//! (`open_boss_bag_click` for the 21 boss bags, `open_container_click`
//! for the other 37, the else-if order above), the shared
//! `fill_quick_spawn_grants` (the `Player.GetItem` fill) +
//! `finish_container_open` (the `TryOpenContainer` tail). The four
//! family modules (`boss_bags.rs`, `fishing_crates.rs`, `presents.rs`,
//! `misc_bags.rs`) carry the pure grant trees over the shared
//! `BagGrant`/`quick_spawn` seam.
//!
//! ## 2. The luck census — NO luck at open time
//!
//! The task's premise ("right-click treasure bag → loot roll at
//! open-time with luck") is REFUTED by the source. Every open-time roll
//! is a PLAIN `Main.rand.Next(...)`:
//!
//! | Window | C# | `luck`/`RollLuck` reads |
//! | --- | --- | --- |
//! | `OpenBossBag` + `TryGettingDevArmor` + the money tail | `Player.cs:7202-8026` | **0** |
//! | `OpenFishingCrate` | `:8028-9250` | **0** |
//! | `OpenPresent`/`OpenLegacyPresent` | `:7031-7168` | **0** |
//! | `OpenHerbBag`/`OpenCanofWorms`/`OpenOyster`/`OpenChilletEgg`/`OpenGoodieBag`/`OpenLockBox`/`OpenShadowLockbox` | `:9310-9585` | **0** |
//! | `QuickSpawnItem` → `Item.Prefix(-1)` (the natural prefix roll) | `:7170-7191`, `Item.cs:504-618` | **0** |
//! | `Player.GetItem` (the fill) | `:39330-39580` | **0** |
//! | `ItemSlot.RightClick`/`TryOpenContainer`/`TryOpenContainer_GrantItems` | `UI/ItemSlot.cs:1636-1865` | **0** |
//!
//! There is NO `ItemCheck_OpenBag`/`OpenBag` method in `Player.cs` —
//! the bag open is a UI event (`ItemSlot`), NOT an `ItemCheck` use-arm.
//! The i52/i47 pickup lanes (`Player.GetItem`/`PickupItem`) read no luck
//! either. Luck enters ONLY on the DEATH side (the drop rules +
//! `NPCLoot_DropMoney`), never at open time.
//!
//! `Luck.RollLuck` (`GameContent/Luck.cs:5-16`): `luck > 0 &&
//! NextFloat() < luck` → `Next(Next(range/2, range))` (the shrunk,
//! better-odds path); `luck < 0 && NextFloat() < -luck` →
//! `Next(Next(range, range*2))` (the widened, worse-odds path); else
//! `Next(range)`. The port: `tr_core::loot::roll_luck`.
//!
//! ## 3. The death-side rule gates (the bag + the master pet/mount)
//!
//! The death side (`NPC.NPCLoot` → `NPCLoot_DropItems` →
//! `ItemDropResolver`) reads the killer's luck through
//! `DropAttemptInfo.player` — but NOT every rule uses it:
//!
//! | Rule | Gate | Luck? | C# |
//! | --- | --- | --- | --- |
//! | `CommonDrop` (the NotExpert weapons, the relics) | `info.player.RollLuck(denominator) < numerator` | YES | `CommonDrop.cs:36` |
//! | `DropLocalPerClientAndResetsNPCMoneyTo0` (the BOSS BAG) | `info.rng.Next(denominator) < numerator` | **NO** (overrides the base) | `DropLocalPerClient...cs:24` |
//! | `DropPerPlayerOnThePlayer` (the master PET/MOUNT) | `rng.Next(denominator) < numerator` per interacting player | **NO** | `CommonCode.cs:45/:51` |
//! | `OneFromOptionsDropRule` | `info.player.RollLuck(denominator)` | YES | `OneFromOptionsDropRule.cs:30` |
//! | `OneFromOptionsNotScaledWithLuckDropRule` | `info.rng.Next(denominator)` | **NO** | `OneFromOptionsNotScaledWithLuckDropRule.cs:30` |
//! | `NPCLoot_DropMoney` | the `NextFloat() < |luck|` double-roll (best/worst) | YES | `NPC.cs:79622-79626` |
//!
//! ### Fix 1 — the boss-bag gate is luck-free (`e0e5b53d`)
//!
//! The port's `DropLocalPerClientAndResetsMoney::try_dropping_item`
//! rolled `roll_luck(info.rng, info.luck, denominator)` — the C#
//! override (`DropLocalPerClient...cs:24`) uses the PLAIN
//! `info.rng.Next(denominator)`. At luck ≠ 0 the port added a
//! luck-dependent float gate + shrunk/widened extra draws to the
//! `UpdateNPCs` stream on every expert boss death. Fixed to the plain
//! `next_range`; witness `w4_f35_boss_bag_gate_is_luck_free` pins the
//! two-draw consumption (gate `Next(0,1)` + stack `Next(1,2)`) at luck
//! 0/1/-0.7 (an empty float script panics on any `RollLuck` gate).
//!
//! ### Fix 2 — the master pet/mount denominator (`8f17966d`)
//!
//! Every `MasterModeDropOnAllPlayers(item, this._masterModeDropRng)`
//! call passes the private field `_masterModeDropRng = 4`
//! (`ItemDropDatabase.cs:15`) as the chanceDenominator — the master
//! pet/mount drops at **1/4**, not 1/1. `gen_loot.py`'s `sugar()` could
//! not parse the `this._masterModeDropRng` field ref (it fell to the
//! `I(1)` default), so all 32 generated rows emitted denominator 1.
//! Fixed: `sugar()` resolves the field ref to its literal 4;
//! `loot_db_gen.rs` regenerated (32 rows 1→4, the only diff; `--check`
//! ok, 1599 registrations).
//!
//! ### Fix 3 — the master pet/mount draw order + money reset (`ecb02e49`)
//!
//! `DropPerPlayerOnThePlayer.TryDroppingItem`
//! (`DropPerPlayerOnThePlayer.cs:24`) passes
//! `info.rng.Next(amountDroppedMinimum, amountDroppedMaximum + 1)` as a
//! CALL ARGUMENT to `CommonCode.DropItemForEachInteractingPlayerOnThePlayer`
//! — the C# argument-evaluation order draws the stack ONCE, BEFORE the
//! per-player gate, even when every gate misses. The port drew it
//! gate-first and only on a hit. The `CommonCode` tail also zeroes
//! `npc.value` UNCONDITIONALLY (`CommonCode.cs:55`). Fixed: the stack
//! draws once before the gate loop, the gate is the plain
//! `next_range(0, denominator) < 1`, and the `resets_money` flag is set.
//! Witness `w4_f35_master_pet_draws_the_stack_before_the_gate` pins the
//! stack-before-gate order on hit AND miss + the classic `DropNothing`
//! no-draw arm.
//!
//! ## 4. The master-mode bag drop rates (the census)
//!
//! The master-mode relics/pets ride the DEATH side (not the bag open).
//! Per boss (`ItemDropDatabase.cs:312-676`):
//!
//! - `MasterModeCommonDrop(relic)` = `ByCondition(IsMasterMode, relic,
//!   1, 1, 1)` — the relic drops at **1/1** in master, once, at the NPC
//!   (`ItemDropRule.cs:25-28`, `CommonDrop`'s luck gate at denominator 1
//!   always passes). The relic ids: 4924 (EoC) … 4950 (Queen Slime),
//!   5110 (Deerclops), 4948 (Betsy), 4949 (Empress), 4946/4947
//!   (Dark Mage/Ogre).
//! - `MasterModeDropOnAllPlayers(pet, _masterModeDropRng)` =
//!   `DropBasedOnMasterMode(DropNothing, DropPerPlayerOnThePlayer(pet,
//!   4, 1, 1, IsMasterMode))` — the pet/mount drops at **1/4** per
//!   interacting player in master (`ItemDropRule.cs:30-33`). The pet
//!   ids: 4795 (WoF goat) … 4817 (Betsy), 4960 (Queen Slime), 5090
//!   (Deerclops), 4796 (Dark Mage book), 4816 (Ogre).
//!
//! The bag ITSELF drops at **1/1 in expert** (`ItemDropRule.BossBag` =
//! `DropBasedOnExpertMode(DropNothing, DropLocalPerClientAndResetsNPCMoneyTo0(bag,
//! 1, 1, 1))`, `ItemDropRule.cs:10-13`) — the denominator-1 gate always
//! passes, and the `CommonCode` zeroes `npc.value`
//! (`CommonCode.cs:29-39`). In master the bag ALSO drops (master ⊇
//! expert, `Main.expertMode => Difficulty >= Expert`, `Main.cs:2700`).
//!
//! The bag OPEN grants (the expert-exclusive items) are ALWAYS granted
//! (no chance roll) — the Worm Scarf 3224, Brain of Confusion 3223, Hive
//! Backpack 3333, Bone Glove 3245, Demon Heart 3335 (gated
//! `!extraAccessory`), the mech cart pieces 3354-3356, Spore Sac 3336,
//! Shiny Stone 3337, Shrimpy Truffle 3367, Empress Flight Booster 4989,
//! Volatile Gelatin 4987, Bone Helm 5100. The master-mode stack bumps
//! (the EoW/BoC ore/ingot counts) ride `Main.masterMode` at open time.
//!
//! ## 5. The e25 boundary (the money ×2.5 seam)
//!
//! The bag money tail builds a FRESH `NPC.SetDefaults(num9)`
//! (`Player.cs:7833-7834`) — whose `ScaleStats` tail (`NPC.cs:17856`)
//! scales `value` by the difficulty money curve
//! (`GameDifficultyData.EnemyMoneyDropMultiplier`,
//! `DataStructures/GameDifficultyData.cs:87`: Classic 1, Expert 2.5,
//! Master 2.5, Legendary 3.5) with the `(int)` truncation of
//! `ScaleStats_ByDifficulty` (`NPC.cs:18176`). `BagEnv::value_multiplier`
//! is that seam: the wiring passes 1.0 (the classic world the port
//! models today); the e25 scaling module stays UNWIRED and untouched.
//! The bag money reads the unscaled `SetDefaults` value exactly like the
//! C# classic path — the expert `value ×2.5` rides the DEATH-side
//! `ScaleStats`, not the bag.
//!
//! ## Verification
//!
//! Code-only: every window re-read against the cited C#; the three
//! fixes carry same-seed/scripted witnesses (WRITTEN NOT RUN — no
//! cargo/rustc/rustfmt/tests/dotnet/graphify invoked). The `gen_loot.py
//! --check` passes (1599 registrations, the 32-row denominator diff the
//! only change).
