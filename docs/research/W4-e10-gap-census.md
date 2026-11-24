# W4-e10 gap census — the 51 unclaimed voice sites, ported + task-seeded

Status: **CODE-ONLY 2026-11-21** (`task/W4-e10-player-sounds-wave`, over
the census-note tip `3738d4015`). No compiler invoked — witnesses
WRITTEN NOT RUN. The census script: `python tools/_e10_final_census.py`
(the context dump: `tools/_e10_census_ctx.py` over its `--json`).

## 1. The census re-run (step 1)

At the seal tip: **211 sites / 160 claimed / 51 unclaimed** — the 51
attributed in `W4-e10-voice-seal.md` §6 (census-only). This pass PORTED
the 4 self-contained sites (§3) and task-seeded the system-dependent
remainder (§4):

- after commit `91a266876`/`966d2b34f` (the pet grants): **162 / 49**
- after commit `158a5b529` (the sign tails): **164 / 47** — FINAL.

The 51 sites with their C# lines (the §6 attribution re-verified against
the context dump; ✔ = ported this pass):

| line | call | enclosing method | disposition |
| --- | --- | --- | --- |
| 5673 | `PlaySound(12)` | `TrySwitchingLoadout` | g55-owned (their `mod.rs:1401` plays it) |
| 15612 | `Item166, Center` | `ApplyEquipFunctional` 576 arm | seed: music-box record (§4.3) |
| 15991 | `Item60, Center` | `ApplyWilsonBeard` | seed: Wilson beard (§4.3) |
| 17116 | `Item197` | `HandleSpectatingControls` | seed: spectate (§4.3) |
| 17120 | `Item198` | `HandleSpectatingControls` | seed: spectate (§4.3) |
| ✔ 17306 | `miscEquips[0].UseSound, position` | `UpdatePet` | **PORTED** `91a266876` |
| ✔ 17338 | `miscEquips[1].UseSound, position` | `UpdatePetLight` | **PORTED** `91a266876` |
| 18306 | `19, x, y, 3` | `ResetEffects` shimmer-fade | seed: shimmer emergence (§4.3) |
| 22287 | `4, wofpos, 10` | `WOFTongue` | seed: WOF tongue (§4.3) |
| 26703/26710/26723/26730 | `Item8, position` ×4 | `Update` gravity flips | seed: gravDir (§4.3) |
| 28333 | `Item28, MountedCenter, −0.9, 0.04` | `Update_TimerCrit` | f3 65-seam (§6.3) |
| 28396 | `Item30, MountedCenter, −0.9, 0.08` | `Update_SnappingStone` | f3 65-seam |
| 28401 | `Item20, MountedCenter` | `Update_SnappingStone` Pyroclast | f3 65-seam |
| 28406 | `Item28, MountedCenter, −0.9, 0.08` | `Update_SnappingStone` ArmletOfRuin | f3 65-seam |
| 31637 | `PlaySound(type)` | `ItemCheck_ManageRightClickFeatures` | seed: held-item swap (§4.3) |
| 31724 | `25, Center` | `..._ShieldRaise` parry cooldown | seed: shield parry (§4.3) |
| 32134 | `Item11, position` | `TileInteractionsUse` 212 arm | seed: snowball launcher (§4.3) |
| 33354 | `PalChilletJoy, Center` | `PetMount` types 62-65 | seed: petting (§4.3) |
| ✔ 33537 | `PlaySound(11)` | `CloseSign` | **PORTED** `158a5b529` |
| ✔ 33547 | `PlaySound(10)` | `OpenSign` | **PORTED** `158a5b529` |
| 35047 | `7, (int)x, (int)y` | `PickupItem` Nebula arm | d79 keep-one (§6.2) |
| 35082 | `PlaySound(7)` | `PickupItem` 4143 arm | f49 keep-one (§6.2) |
| 43364 | `PalFoxparksAttack, Center` | `TryUsingFoxparksAbility` | seed: Foxparks (§4.3) |
| 43702/43706 | `Item197`/`Item198` | `ItemCheck_OwnerOnlyCode` spectate | seed: spectate (§4.3) |
| 46374/46435/46455/46475/46493 | `19, x, y` ×5 | `ItemCheck_UseBuckets` | seed: buckets (§4.3) |
| 46543 | `type (Item26/35), position` | `ItemCheck_PlayInstruments` | instrument seam (§4.1) |
| 46572 | `Item47, position` | `ItemCheck_PlayInstruments` | instrument seam (§4.1) |
| 46681/46685/46689/46693/46697/46701 | `49/48/47/51/52/50, Center, 1, pitchOffset` | `PlayGuitarChord` | d79-seed 47-52 (§4.1) |
| 46710/46714/46718/46722/46726/46730/46734/46738/46742/46746 | `59/58/53/57/62/61/54/56/55/60, Center` | `PlayDrums` | d79-seed 53-62 (§4.1) |

## 2. The d79 out-of-band cross-check (step 2)

d79's PITCH FINAL (`03ebefad7`, research/W4-d79-pitch-final.md §F2)
names six out-of-band E21 gaps: **47-52 / 53-62 / 63 / 64 / 65 / 66**.
Against the 51 Player.cs sites:

| d79 gap | Player.cs sites | count |
| --- | --- | --- |
| 47-52 guitar chords | `PlayGuitarChord` 46681-46701 | **6** |
| 53-62 drums | `PlayDrums` 46710-46746 | **10** |
| 63/64 Journey research | NONE — the callers are `ItemSlot.cs:1293/:1296` (c24's lane) | 0 |
| 65 QueenSlime ambient | NONE — the caller is `NPC.cs:45759` (d33/d43's lane) | 0 |
| 66 type-75 bomb | NONE — the caller is `Projectile.cs:46941` (d8's lane) | 0 |

**16 of the 51 are strict d79-gap sites; 18 counting the two
`ItemCheck_PlayInstruments` use-cues (46543/46572) on the same
instrument-use seam.** The other four gaps contribute zero Player.cs
sites — they are other-file callers, seeded below with owner-contracts
(§4.2) so the owning tasks carry the audio contract when their machines
land. (Matches the §6 census-note attribution verbatim.)

## 3. The ports (step 3 — the self-contained small ones)

### 3.1 The pet/light-pet grant arms — `91a266876` + `966d2b34f`

`Player.UpdatePet`/`UpdatePetLight` (`Player.cs:17298-17340`, called at
`:25573-25574` right after `UpdateBuffs` in the `myPlayer` block) →
`Player::update_pet_grant`/`update_light_pet_grant` (player.rs), wired
in `world_sim/player.rs` after `update_accessory_buff_arms`:

- the `vanityPet || lightPet` table gate + `hideMisc[slot]` refusal +
  the absent-buff `FindBuffIndex` test; `AddBuff(num, 3600)` then the
  item's `UseSound` at `base.position` through the wave-A
  `play_use_sound` façade (the position overload's `pitchOffset`
  default **0** — NOT `useSoundPitch`; the case-2 pitch rides
  `UpdatePlayers` after the attenuation gate).
- the Fairy Bell's legacy `buffType == 27`: a live 102/101 fairy
  satisfies the alias ladder (`:17323-17330`, draw-free); a fresh grant
  picks one of {27, 102, 101} via ONE `Next(3)` (`Utils.SelectRandom`,
  `:17335`) BEFORE the sound's draws.
- **Deviation (recorded):** the Carrot-603 `Main.runningCollectorsEdition`
  gate folds TRUE — the retail copy ships the CE content; the port has
  no `CheckBunny` registry/DLC probe (`Main.cs:4229-4238`).
- The sound plays even when `AddBuff` itself refused (immune) — the C#
  call order (`:17305-17306` unconditional pair).
- 4 witnesses (world_sim/player.rs, WRITTEN NOT RUN): the grant-once +
  hide-refusal tick pair; the function-level fairy pick (one `Next(3)`
  maps the trio, the alias refusals draw-free, the plain-152 grant
  draw-free); the fairy tick's exact stream replay (pick → pitch → the
  i14 tick-tail decay off pristine seed 0) + the muted twin (the gate
  precedes the pitch) + the other-stream isolation; the plain-152 tick
  grant-once + `hideMisc[1]` refusal.
- The i53 `hide_misc` field doc re-stamped: the grants READ the flags;
  the `TogglePet`/`ToggleLight` re-show halves (`:17342-17363`) stay
  unported (nothing clears a hide once set — the pet-grant task's
  remaining seam).

### 3.2 The sign open/close menu-tails — `158a5b529`

`Player.OpenSign`/`CloseSign` (`Player.cs:33528-33551`) → `signs.rs`
`open_sign`/`close_sign`/`close_all_windows_quiet`/
`tick_sign_range_close` + the `Player.sign` latch (`Player.cs:1440`,
ctor −1):

- `OpenSign`: `IngameUIWindows.CloseAll(quiet: true)` (the port's live
  windows — map/bestiary/talk/chest-bare/inventory/shop/crafting, NO
  cues), then the unpositioned `PlaySound(10)` Menu_Open (`:33547`),
  the latch, the record's text into `npcChatText`.
- `CloseSign`: latch clears, text empties, `PlaySound(11)` (`:33537`)
  unless `quiet`; a closed latch is a cue-free no-op.
- Call sites (all seven C# `CloseSign` callers + the open paths):
  the doors.rs sign arm (`:32693-32723` — the clicked-tile `tileSign`
  gate, the same-sign re-click closes WITH the cue, SP opens via
  `Sign.ReadSign` create-if-missing, the MP client sends `46`); the
  netglue case-47 client tail (`MessageBuffer.cs:2400-2403` —
  sender==us + record-live + `!flags[0]` → open); the Escape
  `ToggleInv` sign arm (`:5017-5020`); the map-open prologue
  `CloseSign(quiet: true)` (`:29108`) + the inventory/talk clears
  (`:29107-29110`); the chest/dresser/bank open prologues
  (`:33087`/`:32873`, quiet=false WITH cue); the chat Close button
  (`Main.cs:40682`, quiet); the per-tick range close (`:27504-27519` —
  the Simple-region 32×32 intersect, the gone-record catch arm, cue on
  close).
- **Deviation (recorded):** the dense-store index probe — the port's
  `World.signs` splices dead-tile records (the C# keeps null holes), so
  the case-47 `Main.sign[num58] != null` test maps to "a record still
  sits at this tile" and the open uses THAT record's index.
- The sign EDITOR (`Main.editSign`/`InputTextSign`) and the chat
  panel's sign-draw half (`Main.cs:46713`'s `sign != -1` disjunct) stay
  **S1-E-26**'s — the latch + cues are the ported half.
- 6 witnesses (WRITTEN NOT RUN): signs.rs ×3 (open latch + one cue +
  the quiet prologue; close cue/quiet/no-op; range close
  in-reach/far/gone), doors.rs ×1 (open → re-click close, one cue
  each, draw-free), loop.rs ×1 (the Escape arm + the map quiet
  prologue), netglue.rs ×1 (the live-pair 46→47 open + the flags-bit
  refusal).

## 4. The task seeds (step 3 — the system-dependent ones, owner-contracts)

Every remaining site is an unported-CALLER deferral (the E21 class):
the cue maths is table-complete here, the machine that calls it is
another task's. The contracts below are what the owning task owes the
audio lane when its machine lands — publish-and-forget, no e10 code
owed.

### 4.1 The instrument-use seam (18 sites) — owner: the instrument-player task (research/60 item-use census; d79 §S4 seeds 47-52/53-62; e26 luck for the bad-luck roll)

The caller block: `ItemCheck_PlayInstruments` (`Player.cs:46506+`),
`PlayGuitarChord` (`:46671-46702`), `PlayDrums` (`:46705-46747`).
Contract:

1. **The use-cues (46543/46572)** are `SoundID.Item26` (default) /
   `Item35` (type 507) / `Item47` (the Stellar Tune arm) style plays at
   `base.position` — the wave-A `play_use_sound` façade ALREADY carries
   the case-2 music-pitch arm (`sound.rs:321-324`: styles 26/35/47 →
   ×0.75 volume, pitch = `Main.musicPitch`, the port's stand-in 0.0).
   The task owes: the `Main.musicPitch` producer (the
   `Math.Round(num * musicNotes) / musicNotes` quantize, `:46537`/
   `:46571` — banker's rounding), the `NetMessage.SendData(58)` pitch
   sync, and the re-point of the stand-in at the live value.
2. **The guitar chords (46681-46701, cases 47-52)**: the range ladder
   `range > (1/6)·{5,4,3,2,1}` picks id 49/48/47/51/52/50 (else 50);
   `LegacySoundPlayer.cs:1212-1232` — slot `133 + id − 47`, the FAMILY
   Stop sweep (all six `SoundInstanceItem[133..138]` stop before the
   play), ×0.45 volume, NO per-case pitch. The ONE pitch source is the
   caller's `pitchOffset`: `RollOnlyBadLuckExtreme(10) == 0` →
   `Main.rand.Next(-50, 51) * 0.01` (`:46675-46678`) — a LUCK roll
   (`Luck.RollOnlyBadLuckExtreme`, `Player.cs:18208-18211`), e26's
   lane; at neutral luck it is one `Next(10)` on `UpdatePlayers`
   (the use runs under the `Main.cs:17754` swap), the pitch draw only
   on the 1-in-10 hit. Attenuation BEFORE the draws (the E21e
   precedent); the int overload draws no `GetRandomPitch`.
3. **The drums (46710-46746, cases 53-62)**: the range ladder
   `range > (1/10)·{9,8,7,6,5,4,3,2,1}` picks id
   59/58/53/57/62/61/54/56/55/60 (else 60);
   `LegacySoundPlayer.cs:1234-1245` — slot `139 + id − 53`, the ONE
   slot's Stop, ×0.7 volume, NO pitch, NO caller pitchOffset. Draw-free
   past the gate.
4. The bank rows `Item_133..148` join when the seam lands (the
   `SoundItem[133..148]` assets); the D4 pool deviation stands (the
   family Stop sweep is the case's own contract, model it like the
   splash slots when the chords need the cut-off behaviour).

### 4.2 The other four d79 out-of-band gaps (0 Player.cs sites) — owner-contracts for the sibling lanes

- **63/64 Journey research — owner c24 (`task/W4-c24-journey-ui`,
  ALREADY PORTED there).** c24's `ui/journey.rs::tick_animation` plays
  the thunk at anim 56 (`Research_{Next(1,4)}` — the draw BEFORE the 15
  spark draws, `UICreativeInfiniteItemsDisplay.cs:349-364`) and the
  completion chime at 40 (`Research_0`), on the bare-`Main.rand`
  stand-in (the UI runs outside every swap). The bank rows
  `Research_0..3` (ids 64/63/63/63) are c24's. **Keep-one at the
  merge:** c24's animation-frame plays WIN over any future
  ItemSlot-click play (the C# plays from the animation, not the
  click); the case-63 façade contract when a positioned caller ever
  lands: attenuation gate → ONE `Next(1,4)` slot pick → the slot's
  Stop+re-create, no pitch (`LegacySoundPlayer.cs:480-492`); case 64
  draw-free (`:493-502`). e10 owes NOTHING.
- **65 QueenSlime ambient — owner d33/d43 (the AI_121 machine,
  `NPC.cs:45757-45759`).** Contract: the `Main.rand.Next(360) == 0`
  ambient roll rides `UpdateNPCs` (the NPC AI's own stream) and runs
  REGARDLESS of audibility (it is an AI roll, not a sound roll); on the
  hit, `PlaySound(65, base.position)` — case 65
  (`LegacySoundPlayer.cs:1382-1402`): the THREE zombie-slot guards
  (115/116/117 playing → refuse BEFORE any draw), then ONE
  `Next(115, 118)` slot pick, ×0.5 volume, NO pitch. Attenuation before
  the guards (the E21e precedent). The `Zombie_115..117` bank rows join
  with the machine.
- **66 type-75 cursed-flame bomb — owner d8 (the AI_016 slice,
  `Projectile.cs:46936-46941`).** Contract: `soundDelay == 0` → re-arm
  −1 + `PlaySound(66, (int)position.X, (int)position.Y)` on
  `UpdateProjectiles` (the E21h `sound_delay` machinery already
  exists). Case 66 (`LegacySoundPlayer.cs:1180-1199`) has a DRAW-ORDER
  QUIRK: the `Next(121, 124)` slot pick runs FIRST (before the three
  zombie-slot guards 121/122/123), so a guard refusal still consumes
  ONE draw; past the guards, ×0.45 volume + ONE `Next(-15, 16) * 0.01`
  pitch. Attenuation-gate refusal = zero draws; guard refusal = one;
  full play = two. The `Zombie_121..123` rows join with the machine.

### 4.3 The unseeded remainder (23 sites after the 4 ports) — e10-backlog / item-use-census candidates

Each row names the machine that must land first (the owner-lane) and
the cue contract the audio side already holds:

| sites | machine (owner-lane) | cue contract when it lands |
| --- | --- | --- |
| 5673 | loadout system — **g55 already plays it** (`mod.rs:1401` `Menu_Tick`); the merge cites `:5673` there | draw-free menu cue; no e10 code |
| 15612 | music-box record (`ApplyEquipFunctional` 576 arm: the `Next(540)==0` + `curMusic`/`CanBeRecorded` gates) | Item166 style at Center, wave-A façade; the record roll rides UpdatePlayers |
| 15991 | Wilson beard growth (`ApplyWilsonBeard`, the 5104→5105→5106 `SetDefaults` chain) | Item60 style at Center, wave-A façade |
| 17116/17120 + 43702/43706 | spectate system (`HandleSpectatingControls` + the OwnerOnlyCode arm — no spectate in the port) | Item197 (success) / Item198 (refusal) UNPOSITIONED style plays |
| 18306 | shimmer emergence (`ResetEffects`' `shimmerTransparency == 0.8f` edge — the player shimmer state is S1-E-16/e19's) | case 19 style 3 at `(int)position` — `play_splash_sound(3, …)` exists |
| 22287 | `WOFTongue` (the Horrified drag — `gross`/`wofNPCIndex` exist, the tongue machine does not) | case 4 style 10 at the WOF's `(int)position` — the NPC-death int façade |
| 26703/26710/26723/26730 | gravity flips (`gravDir` unported — the Gravitation potion) | Item8 style at position ×4 (the two flip directions × the two control paths) |
| 28333/28396/28401/28406 | DST accessory recharge machines — **f3's 65-seam owns the MountedCenter re-siting** (§6.3); the machines are i-lane | Item28 −0.9/0.04, Item30 −0.9/0.08, Item20 plain, Item28 −0.9/0.08 at the RE-SITED accessor |
| 31637 | the held-item right-click swap (`ItemCheck_ManageRightClickFeatures` — the port's `try_item_swap` is the ItemSlot UI path, a DIFFERENT site) | `type` = 22 (the fifteen kinds) / 7 (else), UNPOSITIONED int plays; the `releaseUseTile`/`mouseRightRelease` clears precede |
| 31724 | shield parry (`shield_parry_cooldown` — the port has `shield_raised` only) | case 25 MaxMana at Center — d79's `play_single_voice("MaxMana")` pool path (the seal §2 keep-one) |
| 32134 | snowball launcher (tile 212 interaction — unported) | Item11 style at position + the 7-tick cooldown |
| 33354 | petting system (`PetMount` types 62-65 — no petting in the port) | PalChilletJoy trackable at Center — the `play_chillet_sound` shape |
| 35047/35082 | **d79/f49 keep-one** (§6.2) — the pickup arms land on their branches | case 7 positioned/unpositioned — the façades exist |
| 43364 | Foxparks ability (`TryUsingFoxparksAbility` — the channel projectile family) | PalFoxparksAttack trackable at Center, the `soundDelay <= -320` → −21 gate |
| 46374/46435/46455/46475/46493 | buckets (`ItemCheck_UseBuckets` — unported) | case 19 style 1 (default) at `(int)position` ×5 — `play_splash_sound(1, …)` exists |

## 5. Verification log (this pass)

- `python tools/_e10_final_census.py` re-run at every commit:
  211/160/51 → 211/162/49 (`966d2b34f`) → **211/164/47** (`158a5b529`).
- The context dump read for all 51 (the enclosing methods above).
- d79 §F2/§S4 re-read at `03ebefad7`/`24ce8aa0d`; c24's
  `tick_animation` + bank rows re-read at `face9df72`; g55's
  `try_switching_loadout` re-read (the `:5673` cue already theirs).
- The C# re-read for every ported block: `UpdatePet`/`UpdatePetLight`
  (`:17298-17340`), the `:25567-25574` call site, `OpenSign`/`CloseSign`
  (`:33528-33551`), all seven `CloseSign` callers, `ToggleInv`
  (`:4969-5056`), `TryOpeningFullscreenMap` (`:29103-29119`),
  `IngameUIWindows.CloseAll` (the whole file), `MessageBuffer.cs:2366-
  2405`, `TileReachCheckSettings.cs` (the Simple settings),
  `Utils.SelectRandom` (`Utils.cs:2459-2462`), the case-47/52/53/62/
  63/64/65/66 legacy bodies.
- 10 witnesses WRITTEN NOT RUN (4 pet + 6 sign); no compiler invoked
  (code-only lane). The anchors rows: `update_pet_grant` +
  `open_sign` (the canary markers).