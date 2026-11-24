# 368 — Pet-arrival family FINAL + the f43b splice (W4-f43, a6f Success-leg contract)

> CHAIN-BANNER (registry-150 §9 rename): this report's post-§9 self-pick of
> **352 was CONTESTED** — 352 is d1's §9 assignment, EXECUTED at `194c66ce6`
> (`352-d1-falling-block-final`). Per the Lead amendment the f43 FINAL carries
> row **368**; every reference (INDEX/ANALYSIS/PORT_MAP/anchors/census-64)
> now points here. The pre-rename chain-history mentions of 352 stay as history.

Registry-150 §8.7 AMENDMENT-3 free-floor take (368, per the registry §9 Lead
amendment; the original 352 self-pick was contested — see the chain-banner
above). Chain: the f43 census
`64-town-spawn-census.md` (UNWIRED) → this FINAL (the SPLICE, leg 2+3 of the
research/138 §12.2 activation contract). Code-only wave: witnesses WRITTEN NOT
RUN — no cargo/rustc/rustfmt/tests/dotnet/graphify invoked.

## The activation contract (research/138 §12.2, UPDATED) — three legs

The `Misc.PetExchangeSuccess` chat (`NPC.cs:80910-80912`) fires only when
`RerollVariationForNPCType(npcType)` finds a resident of net id 637/638/656 AND
its `RerollVariation()` succeeds. No single lane could witness it alone; the
contract activates in dependency order:

1. **g10 (kinds) — DONE at `5dee28650`.** All eleven town pets
   (637/638/656/670/678-684) landed + WIRED: `NpcKind::TownCat/TownDog/
   TownBunny` + the eight town slimes at their vanilla ids, the kind-guarded
   `7 if is_town_pet` aiStyle-7 axis arm, `npc_findframe_pet`, the
   `is_town_pet`/`is_town_npc`/`is_town_npc_like`/`is_friendly` predicates, the
   bestiary net_ids. **This branch merges g10 `5dee28650` as-is** (the later
   FINAL-phase commits `77ec42303`/`4c4e56752`/`f34bd2715` — the slime-colour
   chat switch, the s1_f_3 coverage pin, the claims-table — stay with the
   merger; the brief pins the kinds premise at `5dee28650`).
2. **f43-splice (THIS lane, f43b).** The arrival gates + the exchange chain
   wire to the live machine (below).
3. **a6 else-arm one-line dispatch (THIS lane).** The merged `use_pet_license`
   (a6's machine, research/138 §10.3/§11.3 keep-own) swaps its hardcoded Fail
   else-arm for the reroll result — the exact published hunk
   (`powerups.rs:441-450`), the `if success` key select.

## What the splice lands (leg 2 + leg 3)

### The arrival gates (`town_arrival.rs` + `world_sim/npcs.rs`)

- `LADDER` grows 21 → 32: the eleven g10 town-pet kinds join at the C# ladder
  tail seats (`Main.cs:66066-66113` — Princess, Copper, Blue, Green, Old,
  Purple, **Red, Yellow, Rainbow**, Bunny, Cat, Dog; the Red-before-Rainbow
  quirk is the C# order, NOT the canSpawn write order). Santa 142's seat
  (`:66042-66045`) stays CLOSED — no ported Santa kind (d36's 141-210 lane).
- `ArrivalContext` gains `tail: TailFlags` + `princess_flag9: bool`.
  `condition_holds` spells the pet/slime arms per-kind (`Main.cs:65900-65943`)
  and the Princess's FULL flag9 (`ctx.princess_flag9`, replacing the
  unlock-only stub).
- `world_sim/npcs.rs` assembles `tail_flags_from_world` (the bought trio + the
  eight slime unlocks + flag8's `GenuineParty` term + the season/anniversary
  overrides) and `princess_flag9` over the LIVE roster by vanilla id. The
  `checkXMas` season read (`Main.cs:13118-13139`) injects the wall-clock date
  ONCE PER CHECK (the `party_roll` boundary predicts the check tick — no
  per-tick clock read enters the sim); `forceXMasForever` is the
  endlessChristmas micro seed (`WorldGen.cs:572-578`).

### The arrival unique-data (`spawn_town_tail_unique_data`)

`NPC.NewNPC`'s `GiveTownUniqueDataToNPCsThatNeedIt` (`NPC.cs:80769` →
`:80839-80856`), wired at the `spawn_town_at` placement site:
1. `GivenName = getNewNPCName(Type)` — the plain-category draw (the pets draw
   their VARIANT-0 category, `NPC.cs:7312-7314`).
2. With a `TownNPCProfiles` entry: `townNpcVariationIndex =
   profile.RollVariation()` (the `Main.rand.Next(6)` VariantNPCProfile draw for
   the license pets; the constant 0 — ZERO draws — for every
   Legacy/Stacked/Transformable profile), then `GivenName =
   GetNameForVariant(npc)` (a SECOND draw on the rolled variant's category).
3. `ShimmeredTownNPCs[Type]` forces variation 1 AFTER the rolls.

**RNG census correction (f43b re-read):** `WorldGen.genRand` is an ALIAS for
`Main.rand` (`WorldGen.cs:4445` `public static UnifiedRandom genRand =>
Main.rand;`), NOT a separate generator. The f43 census note's "different
stream" reading is corrected: at the arrival site the active swap is
`UpdateTime` (`Main.cs:65153` inside the `UpdateTime` wrapper, `:17795`), so
BOTH the variation roll and the name draws ride ONE stream
(`RngKey::UpdateTime`). The pure fns keep them as separate injected closures;
the wiring passes the caller's active stage stream to both slots. (Placement-
cadence nuance: the C# normal placement pass runs under `UpdateWorldGen`
(`Main.cs:17802-17804`); the port's M4-TOWN-1b folded placement rides the
`UpdateTime`-gated block's stream — the recorded stand-in, ANALYSIS
"Pet-arrival SPLICE". The C# Old Man pre-arm DOES call `NewNPC` under
`UpdateTime`, `Main.cs:65780`.)

### The exchange dispatch (`exchange_resident_pet` + the a6 else-arm)

`exchange_resident_pet` is the live-Npc `RerollVariationForNPCType` adapter
(the scan/roll split research/138 §12.2 names): the FIRST active resident of
the type is snapshotted into a one-slot `PetVariationSlot` roster, the pure
`reroll_variation_for_npc_type` runs (first-resident-only, the no-profile
false, the legacy-0 roll, the ≤100-draw cap, the heal), and the result writes
back. On success the C# re-names (`GivenName = GetNameForVariant`, `:80949`)
and heals (`life = lifeMax`, `:80950`); the PetExchange particle +
`TrySendData(56)` (`:80951-80959`) are MP/FX seams the port folds out. The roll
rides `RngKey::UpdatePlayers` (research/138 §10.2's use-context precedent); the
name draw rides the SAME stream (the genRand alias).

The a6 `use_pet_license` else-arm (`powerups.rs`) dispatches to it and posts
`Misc.PetExchangeSuccess` on true / `Misc.PetExchangeFail` on false
(`NPC.cs:80910-80917`) — the C# ALWAYS chats on the exchange and consumes
either way (`:45442`); no flag re-raise, no license re-print. The LIVE
`AnyNPCs` scan now answers (the g10 kinds carry the net ids), so the exchange
leg is REACHABLE.

### The save-flag widening (the `flags.rs:608` load-gap fix)

- **`.wld` read/write** (`wld/sections/flags.rs`): the bought trio
  (`WorldFile.cs:1392-1394`, v217 — a6/g87 already owned these) + the SIX
  newly-owned slime flags (`:1398` blue v250 + `:1409-1415`
  green/old/purple/yellow/copper v261; the rainbow/red pair were S1-P-i35's).
  The old `r.skip(3)` bought-trio load gap is CLOSED (a6's read); the six
  slime `r.skip` gaps are closed here. `World` gains the six fields.
- **`.trwld` v18** (`save.rs`): VERSION 17 → 18. The tail carries the three
  license bytes (the g87 v18 claimant, keep-one) + ALL EIGHT slime bytes
  (f43b's own claim — the rainbow/red pair had no native-save twin, a
  pre-existing gap f43b closes since its live arrival arms read all eight)
  = 11 bytes; a pre-v18 file loads them false. The g88h-A3 union
  seats the license bytes at head offsets [57..60] of the SEALED 107-byte
  head (research/103 §L3-ARBITER FINAL A3 at `d09d7e10a`, research/263 §6.4);
  this branch keeps its own end-of-file tail until the g71c union lands once,
  forward-only. The eight slime bytes are f43b's union claim.
- **The per-record variation byte** (`save.rs` `NpcRecord.variation_index`,
  `SaveNPCs:1731`'s BitsByte payload): the v18 per-record tail byte persists
  the license pets' rolled variant; a pre-v18 record loads 0. The saveglue
  writer + the app load restore it.

## d43-town-pets DUP — keep-one (item 4)

d43's 631-697 band rowed the eleven town pets too (637/638/656/670/678-684).
**KEEP-ONE verdict: g10 canonical** — already adjudicated by d43 research/343
(§2c, →343 per registry-150 §8.7) and d37 research/250, and receipted g10-side
at `f34bd2715` (`tools/_g10_pet_claims.md`). The g10 superset is WIRED (the
kind-guarded 7-if axis + `npc_findframe_pet` + the `is_town_pet` gates + the
Friends.* bestiary override) vs d43's `update_guide` + `{}` FindFrame
stand-ins. **This branch's arrival uses the g10 kinds** (merged `5dee28650`);
the d43 rows DROP at the merge per its own published drop-hunks. The overlap
is ELEVEN, not the brief's two. No d43 code is merged here.

## The pet-arrival family declaration

| Member | C# home | Canonical owner | Ruling |
| --- | --- | --- | --- |
| Town-pet KINDS (637/638/656/670/678-684) | `NPCID.cs:11951-12045`, `SetDefaults` `NPC.cs:17207-17608` | **g10** `5dee28650` | WIRED superset; d43 rows drop |
| Arrival gates (canSpawn + ladder tail) | `Main.cs:65900-65943` / `:66066-66113` | **f43b** (this) | `town_arrival.rs` LADDER + `condition_holds` |
| Princess flag9 (full 24-roster) | `Main.cs:65944-65956` | **f43b** (this) | `princess_flag9` over the live roster |
| Arrival unique-data (name + variation) | `NPC.cs:80839-80856` | **f43b** (this) | `spawn_town_tail_unique_data` |
| License use-arm (gate + purchase + exchange) | `Player.cs:45352-45445`, `NPC.cs:80894-80918` | **a6** `use_pet_license` | keep-own; else-arm dispatches to f43b |
| Exchange reroll chain (≤100-draw, profiles, names) | `NPC.cs:80920-80961`, `Profiles.cs`, `TownNPCProfiles.cs` | **f43** pure fns + **f43b** live adapter | `exchange_resident_pet` |
| Santa 142 arrival | `Main.cs:65852-65855` / `:66042-66045` | **d36** (141-210 kinds) | CLOSED — no ported Santa kind; pure fns keep it verbatim |
| Save flags (bought trio + 8 slime unlocks) | `WorldFile.cs:1392-1394` / `:1398` / `:1409-1415` | **a6/g87** (trio) + **f43b** (6 slime) | `.wld` + `.trwld` v18 |
| Per-record variation index | `SaveNPCs:1731` | **f43b** (this) | `.trwld` v18 per-record byte |

## Witnesses (WRITTEN NOT RUN — code-only)

- `town_arrival_pets.rs::tests` — the 13 census witnesses STAND + 5 new splice
  witnesses: `the_tail_flags_read_the_world_fields`,
  `the_tail_kinds_map_to_the_g10_roster`,
  `the_arrival_unique_data_draws_the_csharp_shape` (the name/roll/name draw
  order + the legacy 2-draw + the shimmer override + the non-town no-draw),
  `the_live_exchange_rerolls_the_first_resident`,
  `the_live_exchange_caps_at_one_hundred_draws`.
- `powerups.rs::tests` — the a6 refusal witnesses STAND + 2 new positive
  witnesses: `a_resident_cat_exchanges_with_the_success_chat` (the Success
  leg, the heal, the consume), `a_legacy_slime_exchange_fails_without_a_draw`
  (the constant-0 LegacyNPCProfile fail, zero draws).
- `save.rs::tests` — `w4_f43_the_pet_arrival_flags_round_trip_in_v18` (the
  11-byte tail — 3 license + 8 slime — + the pre-v18 reset),
  `w4_f43_the_npc_variation_index_round_trips_in_v18`
  (the per-record byte + the pre-v18 0); the DW-1.4/DW-2.x truncation fixtures
  re-pinned to the 11-byte tail.
- `wld/mod.rs::tests` — the owned-fields sweep + the gate-V3 version-gate walk
  extended to the six newly-owned `.wld` slime bytes (blue@2027 v250,
  green/old/purple@2038-2040 + yellow/copper@2043-2044 v261; the rainbow/red
  pair were already owned, S1-P-i35).

## Verification

Code-only: every arm re-read against the cited C# windows in ≤100-line reads
(`Main.cs:65542-66116`, `NPC.cs:80700-80983`/`7280-7325`, `Player.cs:45352-45446`,
`Profiles.cs:1-243`, `TownNPCProfiles.cs:15-199`, `WorldGen.cs:4445`/`5409-5599`,
`WorldFile.cs:1385-1429`/`2360-2471`, `LanguageManager.cs:390-430`). The seam
call sites checked against the actual signatures by read/grep (`World` fields,
`NpcKind` predicates, `NamedRngs::stream`, `lang::category_size`/
`random_from_category`, `ShimmeredTownNpcs::is_shimmered`, `NpcRecord`). No
cargo/rustc/rustfmt/tests/dotnet/graphify invoked.
