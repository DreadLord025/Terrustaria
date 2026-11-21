# 304 — The housing-relocation family bible (W4-d119 FINAL) + the seal receipts

> Wave 4 · researched 2026-11-21 · sources: decompiled + crates + ROADMAP/ANALYSIS/PORT_MAP
> Registry: research/150 round 3 (`51a28d09e`, §7) pins **304+ FREE**; a fresh
> `git log --all --name-only -- docs/research/` sweep at this commit's fetch
> finds NO doc numbered ≥304 at any tip (281–303 taken per §7). This doc takes
> **304** per the registry rule (next number ≥ the free floor after a fresh
> sweep — not max-on-origin+1). Code-only mode: no cargo/tests/rustfmt/graphify
> invoked; every witness cited is WRITTEN NOT RUN.

## Scope

The town-NPC housing family: the room validator (`WorldGen.StartRoomCheck`/
`CheckRoom`/`RoomNeeds`/`ScoreRoom`/`QuickFindHome`, `WorldGen.cs:5717-6243`),
the mid-game relocation driver (`UpdatePrioritizedTownNPC`/
`CheckForHousesNearAPlayer`, `:71627-71674`), the homeless arrival pair
(`SpawnHomelessNPC`/`UnspawnHomelessNPC`, `:5036-5249`), the full
`SpawnTownNPC` (`:5409-5615`) + `IsThereASpawnablePrioritizedTownNPC`
(`:4881-4917`) + `TrySpawningTownNPC` (`:74869-74882`), the
`UpdateTime_SpawnTownNPCs` cadence (`Main.cs:65542-66116`) and the msg-60
client apply (`MessageBuffer.cs:2695-2739`). One paragraph on responsibility:
a town NPC gets a home when a furnished room passes the validator and the
arrival machine picks its type; it keeps the home while the room survives the
7200-tick re-scan; it re-homes (relocation) or arrives homeless (arrival) when
it does not.

## §1 — The family in ONE paragraph (the merger's map)

**The housing-relocation family composes as: d118 owns the scorer/occupancy
seams, d119 owns the drivers, g44 owns the ONE wall table, f43 owns the tail
arms, and every join is a published hunk or a named splice.** d118
(`task/W4-d118-housing-scoreroom`, tip `91db266ea`) is canonical for
`tr-core/housing.rs`'s `QuickFindHome` pass/canSpawn shape, the occupancy
queries and the verbatim `Collision.SolidTiles` rewrite
(`solid_tiles_in_box`, `Collision.cs:3455-3488`) — its housing.rs blob
`2ecd7924e` differs from d119's landed `555bb04d6` ONLY in the `is_house_wall`
block + d119's dedupe witness (verified `git diff` at the tips: 2 hunks,
nothing else), so the merge keeps d118's file + the ONE delegation hunk below.
d119 (this branch) is canonical for the drivers: `town_rehome.rs` (the
prioritized park + the 300-pick house probe + `IsNPCEvictable`),
`town_spawn.rs` (the homeless pair + the FULL `SpawnTownNPC` + the spawnable
query + `TownSpawnStatics` + the walker drain), `town_arrival.rs`'s rate-aware
cadence + the DyeTrader/Zoologist gate terms + the flag8 predicate (§4),
`tr-core/growth.rs`'s `TownSpawnWalker` seam, the `world_sim` wiring
(weather.rs arrival slot / growth.rs head + drain / npcs.rs home-loss pass)
and the `netglue.rs` msg-60 client arm. g44 owns `edit::wall_house` — the ONE
full 279-entry `Main.wallHouse` copy (S1-P-i33); `housing::is_house_wall`
delegates to it (§3's receipt; NEVER take g44's housing.rs wholesale — d118
rewrote `solid_tiles_in_box`). f43 (`79a037f5e`) owns the UNWIRED
`town_arrival_pets.rs` tail (pet licenses 637/638/656, town slimes 670/678-684,
Santa 142, the Princess flag9 roster) — the wiring lane splices it at the
cited positions and folds d119's local flag8 predicate into f43's
`TailFlags`-fed twin (ONE gate, §4). f45 Ph4 (`8c28a7ae`) owns the seven
remaining unlock setters; f38 owns the `NewNPC` naming seam
(`give_town_unique_data`); d43 owns the town-slime kinds (678 has no ported
`NpcKind` yet — `kind_from_net_id(678)` is `None`); c41's claims are the
`spawner.rs` GetSpawnRate/invaders re-verification only (`72a93e062`, zero
overlap with this family's files). The host kickOut/moveRoom msg-60 arms stay
parked (their C2s senders — the housing-banner UI net legs — are unported;
`IsNPCEvictable` lands as the faithful gate for that surface). After the
merges: ONE wall table, ONE room validator, ONE set of drivers, ONE tail.

## §2 — Port status (the family's rows, tip reads)

| C# subsystem | Status | Rust location | Evidence / notes |
| --- | --- | --- | --- |
| `StartRoomCheck`/`CheckRoom`/`RoomNeeds` (`WorldGen.cs:6041-6243`/`:5717-5756`) | PORTED | `tr-core/src/housing.rs` | d118 canonical; the ±2 wall scan reads the delegated full table (§3) |
| `QuickFindHome` (`:5732-5794`) | PORTED | `tr-app/src/town.rs:40` | the last-successful-scan canSpawn shape + the `:5736` outer-guard NO-OP quirk; d118 seam |
| Occupancy queries (`:4770-4855` family) | PORTED | `town.rs:210/:225/:257/:286/:316` | `can_npcs_live_with_each_other` content-sample arm, the two occupancy fns, the mover/homeless finders — d118 |
| `Collision.SolidTiles` (`Collision.cs:3455-3488`) | PORTED ×2 | `tr-core/housing.rs:120` + `tr-app/town_spawn.rs:142` | d118's verbatim rewrite + d119's twin (`collision_solid_tiles`); keep d118's housing copy at merge, the app twin is a separate fn (no dedupe forced) |
| `UpdatePrioritizedTownNPC`/`CheckForHousesNearAPlayer` (`WorldGen.cs:71627-71674`) | PORTED | `town_rehome.rs:49/:140` | 300 picks × 2 `UpdateWorldGen` draws, the `>>4` shift + ClampToWorld fluff 10; ANALYSIS 'Mid-game housing relocation' |
| `IsNPCEvictable` (`:4857-4867`) | PORTED (no C# caller) | `town_rehome.rs` | the banner-draw surface is unported; `kick_out` + the timeout decay already live |
| Home loss (`Main.cs:65607-65614`) | PORTED | `world_sim/npcs.rs:4919-4960` | the 7200/rate-boundary `QuickFindHome(k)` re-scan; out-of-range home NO-OP kept |
| `SpawnHomelessNPC`/`UnspawnHomelessNPC` (`WorldGen.cs:5036-5249`) | PORTED | `town_spawn.rs` + `world_sim/weather.rs:344-380` | the `Next(27000/dayRate)` cadence after the travel block, `netMode != 1` |
| FULL `SpawnTownNPC` (`:5409-5615`) | PORTED | `town_spawn.rs:490` | sticky canSpawn, stinkbug exemption, alternate-spot recursion, the PRIORITIZED-id infi-housing gate, the local-num5/num6 spiral quirk |
| `IsThereASpawnablePrioritizedTownNPC` (`:4881-4917`) | PORTED | `town_spawn.rs:206` | occupants-first then type-order walk over the modelled LADDER |
| `TrySpawningTownNPC` (`:74869-74882`) | PORTED + WIRED | `tr-core/growth.rs:101/:123` + `world_sim/growth.rs:121/:181` | escalation 1 `171549649`: the walker seam pushes `(x,y)` intents, the app drains into the full `SpawnTownNPC` at the `UpdateWorldGen` slot; `tileSolid[379]` needs no toggle (base table carries 379) |
| `UpdateTime_SpawnTownNPCs` core (`Main.cs:65542-66116`) | PORTED (21-kind LADDER) | `town_arrival.rs:305` | the 7200/rate interval, the party `Next(40)` on the same boundary, DyeTrader ≥4 + Zoologist ≥0.1f wired (phase 2b) |
| flag8 → TownSlimeGreen 678 (`:65795-65799`/`:65916-65919`/`:66078-66081`) | PREDICATE LANDED, production waits on d43+f43 | `town_arrival.rs::slime_green_flag8/slime_green_can_spawn` | §4 — the f41e census's remaining tail; `ctx.genuine_party` fed from `world_events.genuine_party` |
| Pet/slime/Santa/Princess tail arms (`:65852-65956`/`:66042-66113`) | PORTED UNWIRED (f43's) | f43 `town_arrival_pets.rs` | the splice is the wiring lane's; d119 closes only the two unclaimed LANDED gaps + flag8 |
| msg-60 client apply (`MessageBuffer.cs:2695-2739`) | PORTED | `app/netglue.rs:1852` | the slot-keyed home write + `isLikeATownNPC`-gated KickOut/SetRoom book arms; host arms parked (C2s unported) |
| Unlock switch (`WorldGen.cs:5577-5608`) | PARTIAL (2 of 9) | `town_spawn.rs:683-687` | PartyGirl/Princess modelled; the other seven setters f45 Ph4's claim |

## §3 — The g44e-islands-retire §5 receipt: CONSUMED (mutual receipt)

**d119 side (this branch):** the ONE verbatim hunk from g44 research/87
RETIRE §5 (`2af46dec2`, seal `9ce512b4f`) is EXECUTED at `dd8a2f764`:
`housing.rs:18-27`'s narrow `is_house_wall` (blob `2ecd7924e`, byte-identical
at d118/d119/main) replaced by the delegation — the load-bearing body line
`crate::edit::wall_house(wall)` (the anchors marker, `anchors.txt:904` kept),
the doc reworded in d119's own voice (RETIRE §5 pins the body, not the doc),
plus d119's own full-table witness
`the_house_wall_gate_is_the_full_main_table` (the widened bands, the unsafe
walls, the 279 count over `WALL_COUNT`, a wood-4-wall room scans) instead of
g44's optional additive. The `wall.rs:379+`/`hammer.rs:158+` stale-subset
comments repointed to the one-copy state (comment-only; the `pub fn
wall_house` anchors row untouched). The FREE correctness gain is realized:
`town_spawn.rs:506` (was `:463` at `04d7f9fd2` — escalation-1 line drift)
reads `Main.wallHouse` directly in the C# (`WorldGen.cs:5411` re-read) and
now delegates to the same full table — a disc-wall-82 anchor arms `canSpawn`
exactly as vanilla, no hunk at that call site.

**g44 side (the mutual half):** g44's SEAL-VERIFY `f688ab142` (research/87
§SEAL-VERIFY item 3) re-read this consumption at the live tips and CONFIRMS
it: "d119 CONSUMED the dedupe at `dd8a2f764` … the fn body is the load-bearing
`crate::edit::wall_house(wall)` delegation (the anchors marker, row `:904`
kept) … `town_spawn.rs:506` … gains the `WorldGen.cs:5411` correctness with NO
hunk there". Both halves published — the receipt is MUTUAL and CLOSED on the
d119×g44 edge.

**d118 side (the remaining half — d118's own lane executes):** d118 tip
`91db266ea` still carries the narrow blob `2ecd7924e` (re-verified at this
fetch; g44's seal-verify agrees: "d118 still needs the ONE hunk … its
worktree carries unrelated in-flight town.rs / world_sim/npcs.rs edits").
The SAME-ONE-verbatim-hunk applies unchanged: replace d118 `housing.rs:18-27`
(the 2-line doc + 8-line narrow fn quoted in RETIRE §5) with the delegation
body `crate::edit::wall_house(wall)`; the delegation target exists identically
at d118's tip (`edit/wall.rs` blob `9f39cbde8` — d119's `38887d4c3` differs by
the comment refresh ONLY, the fn body identical). d118's
`TooManyUnsafeWalls` witness uses unsafe wall 3 (absent from the full table
too — stays green); no d118 test pins the narrow subset; the widening is
RNG-free. **NEVER take g44's housing.rs wholesale** — d118 rewrote
`solid_tiles_in_box` (`:118` at its tip) + the occupancy fns; g44's file lacks
them. Merge order: whichever housing lane lands SECOND sees the delegation
already present and keeps ONE copy (the bodies are identical; d119's witness
is additive). This branch does NOT edit d118's checkout — the hunk is
published, d118's lane executes it on its own branch.

## §4 — The town-spawn 678-flag8 gate (the f41e party census × f43 arrival compose)

**The claim chain:** f41's party-center-of-attention census (research/70 §8h,
`d118ff9d`; the PARTY SEAL `ef3a481a6` §9e keeps it a census lane) names
`Main.cs:65795`/`:65916`/`:66078` flag8 (`GenuineParty ||
unlockedSlimeGreenSpawn`) → TownSlimeGreen 678 as "town-spawn lane
(d119/f43/c41; d119's town_arrival.rs models the PartyGirl flag7 gate, the 678
arm is its lane's remaining tail)". c41's pushed claims are spawner.rs-only
(`72a93e062` — no 678/flag8 hit in its docs). f43's census (`79a037f5e`,
research/64) ports the arm verbatim inside its UNWIRED
`town_arrival_pets::tail_can_spawn` row `(678, flag8 && count(678) < 1)` +
`pet_slime_princess_ladder_tail` (`:66078-66081` slot, Red-before-Rainbow
order kept), fed `TailFlags.genuine_party || TailFlags.unlocked_slime_green`.

**What d119 lands (this commit):** the C# re-read (`Main.cs:65795-65799` —
`bool flag8 = BirthdayParty.GenuineParty; if (NPC.unlockedSlimeGreenSpawn)
flag8 = true;`; `:65916-65919` — `if (flag8 && num32 < 1)
Main.townNPCCanSpawn[678] = true;`; `:66078-66081` — `if (((num42 == 0) &
flag8) && num32 < 1) num42 = 678;`, the non-short-circuit `&` on two pure
sides) as `town_arrival.rs::slime_green_flag8` + `slime_green_can_spawn`,
with `ArrivalContext.genuine_party` fed in production from
`app.world_events.genuine_party` (`world_sim/npcs.rs`'s ctx build) — NOT
`party_is_up`: a manual monolith party does not open the gate
(`world_events.rs:61-62` folds `genuine_party || manual_party`; the C# reads
`GenuineParty` alone). ZERO RNG draws — the gate never touches the party
`Next(40)` slot. Two witnesses: the four-row flag8 truth table + the
`num32 < 1` resident veto, and the no-landed-arm-change census (a
`genuine_party: true` check leaves every LADDER arm + the winner identical;
flag7 stays its own gate).

**Why a predicate and not a LADDER row:** 678 has no ported `NpcKind` (d43's
kinds lane; `kind_from_net_id(678)` → `None`), the world carries no
`unlockedSlimeGreenSpawn` field (only rainbow/red, `world.rs:303-307` — f43's
census save-widening note), and the canSpawn/ladder tail splice is f43's
published claim. A LADDER entry now would be an unreachable row inventing a
kind. The compose contract: **your-walker vs f43-arrival** — d119's
`TownSpawnWalker` drain runs the FULL `SpawnTownNPC` for ANY prioritized type
(the walker seam is type-agnostic), so once d43 lands the kind + f43's tail
splices (folding this local predicate into `TailFlags` — ONE gate, delete
`slime_green_flag8`/`slime_green_can_spawn` at the splice or re-point them at
f43's fns), a prioritized 678 flows through the existing
query→`SpawnTownNPC`→`NewNPC` path with no new driver work. The
`is_town_pet(678)` early-return (`town.rs:573-575`, the `NPCID.Sets.IsTownPet`
set) is already landed — a town slime needs no `HasRoomQuick`.

## §5 — The walker seam + escalation-1 (EXECUTED, re-verified)

`tr-core/growth.rs:101` `TownSpawnWalker` (invasion/eclipse guard + the
`++npcSpawnDelay >= npcSpawnPeriod` counter reset-on-fire + the wall-34-only
`Next(4)` short-circuit, `WorldGen.cs:74869-74874` re-read) rides the
`UpdateWorldGen` stream INLINE with the growth picks (`_ts` walker variants
at `:766/:1017/:1871`; `None` keeps every landed pure walk). The fire pushes
`(x, y)` onto the shared queue (the `take_statue_spawns` event-queue
precedent); `world_sim/growth.rs:121` builds the walker, `:181` drains into
the FULL `SpawnTownNPC` (`:74878`, `canSpawnNewTownNPC: true`) — the inline
call deferred because the pure walkers have no NPC store. NO_HOBO(8) per
`Successful`/`RelocatedHomeless` after the borrow releases. The
`tileSolid[379]` window (`:74876-74879`) needs no toggle (the generated base
table carries 379). The statics' decay/period re-arm (`:71545-71549`) rides
the `UpdateWorld` head, rate-0 gated. Landed `171549649`; g44's seal-verify
confirms "escalation-1 … landed the TrySpawningTownNPC seam (the recorded
UNWIRED deviation closed)".

## §6 — RNG census (the family, exact slots — unchanged by this commit)

| Site | Stream | Draws | Cite |
| --- | --- | --- | --- |
| Homeless cadence roll | `UpdateTime` | 1 × `Next(27000/max(dayRate,1))` per morning tick | `Main.cs:65057` |
| `SpawnHomelessNPC` house pick / roofline / chat | `UpdateTime` | 1 + k(1..30) + 1 | `WorldGen.cs:5093`/`:5108-5124`/`:5200` |
| `UnspawnHomelessNPC` / `SpawnTownNPC` / the query | — | 0 (naming = f38's seam) | `:5206-5249`/`:5409-5615`/`:4881-4917` |
| `TrySpawningTownNPC` gate | `UpdateWorldGen` | 1 × `Next(4)` ONLY on wall 34 at the fire tick | `:74874` |
| `CheckForHousesNearAPlayer` | `UpdateWorldGen` | 2 per pick, ≤300 (600 on the no-house arm) | `:71661-71672`, `Utils.cs:1246-1249` |
| `UpdateTime_SpawnTownNPCs` | `UpdateTime` | 1 × `Next(40)` per check (the party slot) | `Main.cs:65786` |
| flag8 / the 678 arm / home-loss / prioritized / evictable / msg-60 | — | 0 | `:65795-65799`/`:65916`/`:66078`, `:65607-65614`, `WorldGen.cs:71627-71641`/`:4857-4867`, `MessageBuffer.cs:2695-2739` |

## §7 — Deviations (the family's open list, all recorded in ANALYSIS)

1. The homeless anchor-tile fold (`Npc.home` conflates homeless + homeTile;
   the arrival anchor feeds the immediate `QuickFindHome`, later re-scans
   ride the 2 s look pass).
2. The `BlockedTooManyNPCs` Vec cap (the push always succeeds; the enum arm
   kept).
3. The unlock switch: 2 of 9 setters modelled (f45 Ph4 owns the rest;
   `CheckAchievement_RealEstateAndTownSlimes` waits on d43's kinds).
4. The f43 tail splice stays open (pet/slime/Santa/Princess — f43's UNWIRED
   fns; flag8's predicate landed here folds in at the splice, §4).
5. The single-player fold + the `is_town_npc` subset + the Truffle-false
   pass-through (the relocation-driver trio, ANALYSIS 'Mid-game housing
   relocation').
6. The host msg-60 kickOut/moveRoom arms parked (C2s senders unported).
7. NEW (this commit): the 678 gate is a landed predicate with no production
   fire path until d43's kind + f43's splice (§4) — recorded, not invented.

## Tests

WRITTEN NOT RUN (code-only): `town_arrival.rs` 12 (incl. the 2 new flag8
witnesses `the_slime_green_flag8_gate_is_party_or_unlock_and_no_resident` +
`the_flag8_feed_changes_no_landed_ladder_arm`), `town_spawn.rs` 9,
`town_rehome.rs` 6, `tr-core/growth.rs` 14 (incl.
`the_walker_gate_counts_and_draws_only_on_wall_34`), `world_sim/growth.rs` 5,
`netglue.rs`'s 4 NpcHouse apply legs, `housing.rs`'s dedupe witness. The
flag8 witnesses are pure-predicate + table-equality — no RNG stream touched.

## Task seeds

- **S** — d118 executes the ONE §5 hunk on its own branch (or the merger
  applies it); second-lander keeps ONE copy (§3).
- **M** — the f43 tail splice (wiring lane): fold `slime_green_*` into
  `TailFlags`, widen the world save's bought-trio + slime flags (f43 census
  note), splice the 13 canSpawn rows + the ladder tail at the cited slots.
  Depends on d43's town-slime kinds for production fire.
- **S** — the composed 678 witness once d43+f43 land: a genuine party +
  a furnished room → the walker drain spawns a Green slime (the §4 contract).