# 64 — Town-spawn census (W4-f43, parallel to W4-c41)

> **SPLICED (W4-f43b):** the "UNWIRED" status below is the census wave's
> historical state. The wiring landed on this branch — research/352
> (`352-pet-arrival-family-final.md`): the arrival gates ride
> `town_arrival.rs`'s LADDER (21→32) + `condition_holds`, the exchange chain
> dispatches from a6's `use_pet_license` else-arm through
> `exchange_resident_pet`, the save flags widened (`.wld` + `.trwld` v18),
> the g10 kinds merged (`5dee28650`). The "Port gaps the wiring lane must
> close" section below is CLOSED except Santa 142 (no ported kind — d36)
> and the unlock-source writers (the bound-slime transforms / King Slime
> death / shimmer arms — their own lanes; the rainbow/red/shimmer flags
> already had live writers). The RNG-slot table's "genRand NOT Main.rand"
> reading is CORRECTED by 352: `WorldGen.genRand` aliases the active
> `Main.rand` (`WorldGen.cs:4445`).

Scope: `Main.UpdateTime_SpawnTownNPCs` (`Main.cs:65542-66116`) — the full
canSpawn table + priority ladder, arm by arm — plus the unlock sources of the
unclaimed arms. Code-only wave: ports WRITTEN NOT RUN, UNWIRED. The core
21-kind machine is already landed (`town_arrival.rs`, M4-TOWN-1b); this
census covers the tail arms it leaves closed and ports them verbatim into
`crates/tr-app/src/town_arrival_pets.rs`.

## Census correction (directive vs decompile)

The directive says "town pet slimes (701-707)". This decompile has
`NPCID.Count = 697` (`ID/NPCID.cs:12581`) — **no NPC ids 701-707 exist**. The
town slimes are **670 (Blue), 678 (Green), 679 (Old), 680 (Purple),
681 (Rainbow), 682 (Red), 683 (Yellow), 684 (Copper)**
(`NPCID.cs:12017-12045`); `Sets.IsTownSlime` = {670, 678-684}
(`NPCID.cs:10119`). The bound forms are 685/686/687
(`NPCID.cs:12047-12051`). The census ports the real ids.

## Arm-by-arm table (`Main.cs:65800-65956` canSpawn, `:65957-66114` ladder)

Status key: **LANDED** = in `town_arrival.rs` on main a725acb0;
**F43** = ported this wave in `town_arrival_pets.rs` (unwired);
**C41/OTHER** = belongs to the parallel c41 town-spawn-invaders lane or a
named follow-up (not touched here).

| Type | Kind | canSpawn gate (cite) | Ladder position (cite) | Status |
| --- | --- | --- | --- | --- |
| 22 | Guide | `num6 < 1` (`:65800-65803`) | `:65966-65969` | LANDED |
| 17 | Merchant | `SpawnAllowed_Merchant && num2 < 1` (`:65804-65807`) | `:65970-65973` | LANDED |
| 18 | Nurse | `SpawnAllowed_Nurse && num3 < 1 && num2 > 0` (`:65808-65811`) | `:65974-65977` | LANDED |
| 19 | ArmsDealer | `SpawnAllowed_ArmsDealer && num5 < 1` (`:65812-65815`) | `:65978-65981` | LANDED |
| 20 | Dryad | `(infectedSeed \|\| downedBoss1/2/3) && num4 < 1` (`:65816-65819`) | infected-seed lead `:65958-65961`, boss arm `:65990-65993` | LANDED (infected-seed lead = C41/OTHER special-seed lane) |
| 38 | Demolitionist | `SpawnAllowed_Demolitionist && num2 > 0 && num8 < 1` (`:65820-65823`) | `:65994-65997` | LANDED |
| 353 | Stylist | `savedStylist && num22 < 1` (`:65824-65827`) | `:66026-66029` | LANDED |
| 369 | Angler | `savedAngler && num23 < 1` (`:65828-65831`) | `:66010-66013` | LANDED |
| 54 | Clothier | `downedBoss3 && num9 < 1` (`:65832-65835`) | `:66022-66025` | LANDED |
| 107 | GoblinTinkerer | `savedGoblin && num11 < 1` (`:65836-65839`) | `:65982-65985` | LANDED |
| 441 | TaxCollector | `savedTaxCollector && num24 < 1` (`:65840-65843`) | `:66054-66057` | LANDED |
| 108 | Wizard | `savedWizard && num10 < 1` (`:65844-65847`) | `:65986-65989` | LANDED |
| 124 | Mechanic | `savedMech && num12 < 1` (`:65848-65851`) | `:66006-66009` | LANDED |
| **142** | **SantaClaus** | `downedFrost && num13 < 1 && Main.xMas` (`:65852-65855`) | `:66042-66045` (after PartyGirl, before Bartender) | **F43** |
| 178 | Steampunker | `((tenthAnniversary && !getGoodWorld) \|\| downedMechBossAny) && num15 < 1` (`:65856-65859`) | `:66002-66005` (boss term only) | C41/OTHER (kind not in port LADDER) |
| 207 | DyeTrader | `SpawnAllowed_DyeTrader && num16 < 1 && num40 >= 4` (`:65860-65863`) | `:66030-66033` | LANDED (the `num40 >= 4` term is a known LANDED gap — c41 lane) |
| 228 | WitchDoctor | `downedQueenBee && num20 < 1` (`:65864-65867`) | `:65998-66001` | LANDED |
| 229 | Pirate | `downedPirates && num21 < 1` (`:65868-65871`) | `:66018-66021` | C41/OTHER (kind not in port LADDER) |
| 160 | Truffle | `num14 < 1 && hardMode` (`:65872-65875`) | `:66058-66061` | C41/OTHER (kind not in port LADDER) |
| 209 | Cyborg | `hardMode && downedPlantBoss && num18 < 1` (`:65876-65879`) | `:66014-66017` | C41/OTHER (kind not in port LADDER) |
| 227 | Painter | `num40 >= 8 && num19 < 1` (`:65880-65883`) | `:66034-66037` | LANDED |
| 208 | PartyGirl | `flag7 && num17 < 1` (`:65884-65887`); flag7 = `rand.Next(40)==0 && num40>=20` or unlock (`:65785-65794`) | `:66038-66041` | LANDED (the `Next(40)` slot pinned on `RngKey::UpdateTime`) |
| 550 | Tavernkeep | `savedBartender && num25 < 1` (`:65888-65891`) | `:66046-66049` | LANDED (DD2Bartender kind) |
| 588 | Golfer | `savedGolfer && num26 < 1` (`:65892-65895`) | `:66050-66053` | LANDED |
| 633 | Zoologist | `((vampireSeed && !infectedSeed) \|\| bestiary >= 0.1) && num27 < 1` (`:65896-65899`) | `:66062-66065` (bestiary term only) | LANDED-closed (bestiary gate = c37/c41 lane; vampire lead = special-seed lane) |
| **637** | **TownCat** | `NPC.boughtCat && num28 < 1` (`:65900-65903`) | `:66106-66109` | **F43** |
| **638** | **TownDog** | `NPC.boughtDog && num29 < 1` (`:65904-65907`) | `:66110-66113` (the LAST ladder arm) | **F43** |
| **656** | **TownBunny** | `NPC.boughtBunny && num30 < 1` (`:65908-65911`) | `:66102-66105` | **F43** |
| **670** | TownSlimeBlue | `unlockedSlimeBlueSpawn && num31 < 1` (`:65912-65915`) | `:66074-66077` | **F43** |
| **678** | TownSlimeGreen | `flag8 && num32 < 1` (`:65916-65919`); flag8 = `GenuineParty \|\| unlockedSlimeGreenSpawn` (`:65795-65799`) | `:66078-66081` | **F43** |
| **679** | TownSlimeOld | `unlockedSlimeOldSpawn && num33 < 1` (`:65920-65923`) | `:66082-66085` | **F43** |
| **680** | TownSlimePurple | `unlockedSlimePurpleSpawn && num34 < 1` (`:65924-65927`) | `:66086-66089` | **F43** |
| **681** | TownSlimeRainbow | `unlockedSlimeRainbowSpawn && num35 < 1` (`:65928-65931`) | `:66098-66101` | **F43** |
| **682** | TownSlimeRed | `unlockedSlimeRedSpawn && num36 < 1` (`:65932-65935`) | `:66090-66093` | **F43** |
| **683** | TownSlimeYellow | `unlockedSlimeYellowSpawn && num37 < 1` (`:65936-65939`) | `:66094-66097` | **F43** |
| **684** | TownSlimeCopper | `unlockedSlimeCopperSpawn && num38 < 1` (`:65940-65943`) | `:66070-66073` (first slime in the ladder) | **F43** |
| **663** | **Princess** | `flag9 && num39 < 1` (`:65953-65956`); flag9 = all 24 residents (`:65944`) or `tenthAnniversary && !getGoodWorld` (`:65945-65948`) or `unlockedPrincessSpawn` (`:65949-65952`) | `:66066-66069` (first tail arm) | **F43** (LANDED core keeps the unlock-only stub; this ports the full flag9) |

**Ladder-order quirk (ported 1:1):** the canSpawn table writes the slimes in
id order (Blue, Green, Old, Purple, Rainbow, Red, Yellow, Copper) but the
ladder picks **Copper, Blue, Green, Old, Purple, Red, Yellow, Rainbow** —
Red before Yellow before Rainbow (`:66070-66101`). The pets close the ladder
in **Bunny, Cat, Dog** order (`:66102-66113`), the reverse of their canSpawn
write order (Cat, Dog, Bunny).

**The Old Man pre-arm** (`:65778-65784`): `!downedBoss3 && num7 == 0 &&
dungeonX/Y >= 0` spawns the Old Man (37) at the dungeon directly — no
canSpawn/ladder involvement. C41/OTHER lane (dungeon spawn).

## RNG slots (the whole census)

| Slot | Stream | Cite | Port seam |
| --- | --- | --- | --- |
| Party Girl `Next(40) == 0`, once per 7200-tick check | `Main.rand` | `Main.cs:65786` | LANDED — `RngKey::UpdateTime`, `world_sim/npcs.rs:4876-4877` |
| Pet variation reroll `Next(variants.Length)` = `Next(6)`, ≤100 draws while unchanged | `Main.rand` | `Profiles.cs:214-217`, loop `NPC.cs:80941-80944` | F43 — injected `roll` closure in `reroll_variation` / `reroll_variation_for_npc_type` |
| Pet/slime/Princess name pick `Next(category.Count)` | **`WorldGen.genRand`** (NOT Main.rand) | `NPC.cs:7311-7322` → `LanguageManager.cs:419`; variant names `Profiles.cs:219-222` | F43 — `name_category_for` returns the key; the wiring lane draws on the genRand stream |
| Everything else in the table/ladder | — | zero RNG | F43 pure fns take counts + flags only |

The license chain itself (`ItemCheck_UsePetLicenses` → `LicenseOrExchangePet`
→ `UnlockOrExchangePet`) draws NOTHING until the exchange arm reaches
`RerollVariation`. First use is flag-write + chat + `TrySendData(7)` only
(`NPC.cs:80904-80909`).

## Unlock sources (census of the flag writers)

| Flag | Writer | Cite |
| --- | --- | --- |
| `boughtCat` / `boughtDog` | Cat/Dog License use (items 4829/4830) | `Player.cs:45354-45361` → `NPC.cs:80906` |
| `boughtBunny` | Bunny License use (item 4910) | `Player.cs:45362-45365`; also the tenth-anniversary world-gen spawn (`WorldGen.cs:19813-19815`, variation pinned 1 = Angora) |
| `unlockedSlimeBlueSpawn` | King Slime (50) death → `ViolentlySpawnNerdySlime` spawns 670 directly at the corpse with `(NextFloatDirection()*3, -10)` velocity | `NPC.cs:79411-79415`, `:79516-79529` |
| `unlockedSlimeGreenSpawn` | Green slime (678) moves into a house (`SpawnTownNPC` success switch) | `WorldGen.cs:5605-5607`; flag8 also opens on any genuine party (`Main.cs:65795`) |
| `unlockedSlimeOldSpawn` | `TransformElderSlime`: bound 685 → 679 | `NPC.cs:19136-19158` |
| `unlockedSlimePurpleSpawn` | bound 686 → 680 (`NPC.cs:81768-81779`); drunk-world gen spawn (`WorldGen.cs:19805-19810`); `WorldGen.cs:19809` shimmer path | cited |
| `unlockedSlimeRainbowSpawn` | shimmer transform of a town slime (`WorldItem.GetShimmered` 4986 arm — LANDED on `world.rs:300-303`); remix-world gen spawn (`WorldGen.cs:19799-19804`) | cited |
| `unlockedSlimeRedSpawn` | fished-out crate arm (LANDED on `world.rs:304-307`, `Player.cs:52429-52432`) | cited |
| `unlockedSlimeYellowSpawn` | bound 687 hit by the sparkler projectile → 683 | `Projectile.cs:14737-14755` |
| `unlockedSlimeCopperSpawn` | `TransformCopperSlime`: any of {1, 302, 335, 336, 333, 334} (`NPCID.cs:10120` `CanConvertIntoCopperSlimeTownNPC`) shimmered → 684 | `NPC.cs:19160-19182` |
| `unlockedPrincessSpawn` | Princess (663) moves in (`WorldGen.cs:5602-5604`); tenth-anniversary gen spawn (`WorldGen.cs:19766-19773`, name pinned "Yorai"); save-load fixup `AnyNPCs(663)` (`WorldFile.cs:2952`) | cited |
| `Main.xMas` | `checkXMas`: wall-clock Dec 15-31, `forceXMasForToday`/`Forever` override | `Main.cs:13118-13139` — F43 `check_xmas` (date injected) |

## Port gaps the wiring lane must close (recorded, not fixed here)

1. **Save flags**: the port SKIPS `boughtCat/Dog/Bunny` on load
   (`wld/sections/flags.rs:608` `r.skip(3)`, write-side `:212-214` writes
   false) and carries only the rainbow/red slime flags
   (`world.rs:300-307`). The other six slime flags + the bought trio need
   World fields + the flags-section read/write before the F43 arms can fire
   in production. (Save-header lanes f44/f45/c43 own the section surgery.)
2. **Kinds**: `NpcKind` has no TownCat/TownDog/TownBunny/TownSlime*/SantaClaus
   variants yet (the d43 npc-kinds-631-697 lane owns the enum rows; Santa 142
   is d36's 141-210 lane). The F43 module is id-keyed (`i32`) so it splices
   without them.
3. **Ladder splice points**: Santa after the Party Girl arm
   (`town_arrival.rs` LADDER position 15/16 boundary), the pet/slime/Princess
   tail after the Zoologist arm (LADDER tail). The F43 fns take/return
   `num42` so the splice is a fold, not a rewrite.
4. **Princess flag9 in the LANDED core**: `town_arrival.rs:217` keeps the
   unlock-only stub; `princess_flag9` here is the full 24-resident
   conjunction — the wiring lane replaces the stub.

## Witnesses (WRITTEN NOT RUN — code-only wave)

`town_arrival_pets.rs::tests` — 13 tests:
`the_xmas_window_is_december_fifteen_through_thirty_one`,
`the_license_gate_wants_zero_item_time_and_a_resident_for_exchanges`,
`the_unlock_or_exchange_branches_follow_the_csharp`,
`the_reroll_loop_caps_at_one_hundred_draws` (the 100-draw cap + the
immediate/last-draw success arms),
`the_variant_rosters_and_head_ids_match_the_profiles`,
`the_name_categories_key_on_the_variation`,
`the_exchange_rerolls_the_first_resident_only` (incl. the Legacy-0 roll and
the no-profile no-roll arm),
`the_pet_arms_open_on_the_bought_flags`,
`the_green_slime_rides_the_party_or_its_unlock`,
`santa_needs_the_frost_legion_and_the_season`,
`the_princess_wants_every_town_type_or_an_override`,
`the_santa_ladder_arm_passes_a_live_pick_through`,
`the_ladder_tail_order_is_the_csharp_one` (the Red-before-Rainbow quirk +
the Bunny-Cat-Dog pet order).
