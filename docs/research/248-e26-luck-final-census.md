# 248 — W4-e26 luck-system FINAL census: the critter sweep scope, the factor activation, the Frog/Duck keep-one claims-table, the luck-family declaration

Branch: `task/W4-e26-luck-system` (retire phase, 2026-11-21, code-only —
no cargo/tests/rustfmt/graphify invoked; every witness WRITTEN NOT RUN).
Base: `a725acb04` (origin/main). Phase commits: `69e977bca` (gnome sensor),
`47241f5f4` (equipment tail + writers), `c4a7dff1c` (factor census),
`048cca6cf` (the luck-critter roster), `04dbdb0d2` (the E8 keep-one
EXECUTION), this doc.

**Number 248 per registry-150.** The W4-d54 COLLISION REGISTRY
(`origin/task/W4-d54-dust-sites-world`, `docs/research/150-research-number-registry.md`
§2) frees 237+ for future lanes ("if those lanes later push them, they MUST
take free numbers 237+"). A full remote-tip `ls-tree docs/research/` sweep at
retire time (re-run after the sibling lanes advanced) shows **237-247 all
claimed**: 237 (e23), 238 (c8/d42), 239 (d42/f21/g48/g56), 240 (f4/g58/g63),
241/242 (g58), 243 (d50/g58), 244 (d53/f39/g48/g86), 245 (g48), 246 (d10),
247 (claimed); the scattered higher claims run 262/264/266/269/270/271/273/
274/276/279/280. **248 is the first free number at every tip** — e26 takes it.
(The doc was drafted at 243 when 243 was free; the d50/g58 pushes claimed 243
mid-retire, so the number moved to 248 — the registry's first-free rule, no
other branch touched. The 237-247 multi-claims are post-registry collisions the
registry's 151-236 block did not cover; they are the merger's to renumber, not
this lane's — recorded here as evidence.)

## 1. The e26c-luck-critter-sweep scope — the c8 SIGHT-gate auto-covers the critters

The dispatch's "census remaining luck-critter wiring" resolves to **zero new
provider wiring**: the W4-c8 generic critter provider contract
(`origin/task/W4-c8-style66-crawling-critters` tip `7c2de9c06`,
research/238 §1/§7a) auto-covers every e26 kind the moment the roster rows
merge. Verified at the c8 tip:

- `scan_world_for_critter_sights` (`bestiary.rs:2490`, run at the `update_npcs`
  head — `Main.cs:17914` before the `UpdateNPC` loop) has **no kind list**: its
  gate is the live `max_hp <= 5 && contact_damage() == 0` (the `CountsAsACritter`
  stand-in, `NPC.cs:6887-6897`).
- `counts_as_a_critter(kind)` (`bestiary.rs:2280`) reads the kind's `SetDefaults`
  rows + the `cs_id != 594/686` exclusions; `is_critter_entry` (`:2303`) adds the
  `DD2LanePortal` 549 Hide exclusion; `unlock_state_world` (`:2412`) dispatches
  on `net_id ∈ GOLD_PEER_TABLE` then the generic critter gate.
- **The e26 receipt (c8 research/238 §7a, verified at e26 tip `048cca6cf`)**:
  Frog/LadyBug/GoldLadyBug pass `counts_as_a_critter` (`npc/data.rs` max_hp 5 /
  contact_damage 0), the entries ride the scan + dispatch with **zero per-lane
  wiring**. The GoldLadyBug 605 → {604} row is ALREADY in `GOLD_PEER_TABLE`
  (`bestiary.rs:2332`, the 13-row table) — 605 auto-rides the gold provider.

**The keep-one deltas this retire phase landed (§3) do not break the contract**:
the Frog's `cs_id`/`net_id` stay 361 (only the wire id moved 361→744), so
`counts_as_a_critter` (which reads `max_hp`/`contact_damage`/`cs_id`) and the
bestiary `net_id` 361 entry are unchanged. The Duck drop removes the e26 Duck
entry; c10 lands its own Duck 362 entry (the c8 §7a receipt already lists c10's
Duck family as auto-covered). The 594/686 `cs_id` exclusions and the 549 Hide
exclusion stay in the c8 gate for every lane — e26 rows neither.

**Cross-note to c8**: no e26 sight scan, provider arm, or peer-table row is
needed (research/238 §1 contract). The e26 ladybug pair + Frog are pre-covered;
the gold 605 row is pre-tabled. The merge reconciles the e26 UI `unlock_state`
call sites onto `unlock_state_world` (the same fold c8 did for c4 — research/238
§1a/§6), a merge-time edit, not an e26 wiring task.

### 1a. The ladybug-rain-boost seam — g67 drops wholesale, d130 is canonical

The dispatch's "ladybug-rain-boost g67" resolves to a **merge-direction note, not
new e26 code**. Census at the tips:

- **e26** (`rain.rs`): the `lady_bug_rain_boost: i32` field (`:120`), the
  `add_lady_bug_rain_boost(add)` setter clamped at `crate::luck::LADYBUG_RAIN_CAP`
  (`:299`, `Main.ladyBugRainBoost += NPC.ladyBugRainTime`, `NPC.cs:81568-81572`),
  the `dayRate` drain (`:669`, `Main.cs:64833-64836`), the rain-start boost gate
  (`:713`, `Main.cs:64920`). Callers: `world_sim/npcs.rs:5083` (the `checkDead`
  ladybug kill arm) and `world_sim/editing.rs:2911` (the S1-P-i35 fishing-bait
  consume) — both feed `app.rain.add_lady_bug_rain_boost(add)`.
- **g67** (`origin/task/W4-g67-weather-rain-splash` tip `8b9a1ff46`): its `rain.rs`
  carries the SAME seam (field `:155`, setter `:404`, drain `:801`) — but g67's
  SEAL (research/146, `8b9a1ff46`/`c38d42d32`) declares **g67 drops rain.rs
  wholesale**; d130 is the canonical rain/snow core (Rain.cs verbatim + the
  `Main.snowing` global-stream fold). g67's only ladybug mention is the c39
  storm-deck graft note (research/146:36), not a seam claim.
- **d130** (`origin/task/W4-d130-weather-ambience` tip `af3ebc1a1`): its `rain.rs`
  ALREADY carries the full seam — field `:106`, setter `add_lady_bug_rain_boost`
  `:345` (the same `LADYBUG_RAIN_CAP` clamp), drain `:719`, rain-start gate `:771`,
  + the drain/gate witnesses (`:2649`/`:2666`/`:2739`/`:2754`). The setter-side
  comment (`:104`) says "the setter side (M4 ladybug deaths) is not ported" — that
  is the **e26 kill/bait callers**, which d130 does not carry.

**Merger action (keep-one)**: d130's `rain.rs` is canonical (the g67 SEAL
verdict); the e26 `add_lady_bug_rain_boost` **callers** (`world_sim/npcs.rs:5083`,
`world_sim/editing.rs:2911`) compose onto d130's setter unchanged (the signature
+ cap are identical). The e26 `rain.rs` seam body drops with the g67/d130 union
(the field/setter/drain/gate are d130's); the e26 luck-side constants
(`LADYBUG_RAIN_TIME`/`LADYBUG_RAIN_CAP`, `luck.rs:37-39`) stay e26's (d130's
setter reads `crate::luck::LADYBUG_RAIN_CAP`). No e26 rain.rs edit is needed at
retire — the seam is already correct; the union is the merger's.

## 2. The luck-factor activation census — torch-god f33 / garden-gnome

The dispatch's "which remaining-neutral factors activate on torch-god f33 /
garden-gnome?" resolves to a **per-factor activation table**. The e26 phase-3
census (ANALYSIS "The W4-e26 factor census") wired the equipment tail, the
lantern-night + gnome copies, the ladybug/town writers, the potion factor; the
**remaining neutral-default inputs** are `usedGalaxyPearl`, the broken-mirror
writer (projectile 1085), and the kite items (`IsAKite`). Census:

| factor | C# writer | e26 state | activates on f33 torch-god? | activates on garden-gnome? |
|---|---|---|---|---|
| `torchLuck` (the scan) | `TryRecalculatingTorchLuck` `Player.cs:17731-18088` | WIRED (the scan runs; `nearbyTorches` counts) | **YES — f33 ports the head/tail** (`torch_scan_step_event`, the `happyFunTorchTime`/`torchGodCooldown` head `:17733-17742` + the `:18066-18086` trigger). f33's `torch_god_trigger_check` (`torch_god.rs:390`) gates on `!unlocked_biome_torches && nearby_torches > 100`; the 5043 favor use (`ItemCheck_UseTorchGodsFavor` `:45226-45243`) sets `unlockedBiomeTorches` PERMANENT, so the trigger never fires again once the favor is consumed. The scan's `torchLuck` term (`RecalculateLuck` `:29139` `torchLuck * 0.2`) stays live either way | no (the gnome is a separate `+0.2` term) |
| `has_garden_gnome_nearby` (`+0.2`) | `UpdateBiomes` `Player.cs:16726-16730` ← `SceneMetrics.HasGardenGnome` (tile 567) | WIRED (phase 1 sensor + the per-tick copy `world_sim/player.rs:590`) | no | **YES — already live** (the tile-567 placement is generic, item 4609 `D_567`; the gnome NPC 624 petrification `AI_003_Gnomes_ShouldTurnToStone` `NPC.cs:55843-55900` places tile 567 — that NPC kind is d42's `Gnome@624`, not e26's; the e26 sensor reads the placed tile regardless of who placed it) |
| `usedGalaxyPearl` (`+0.03`) | `ItemCheck_UseGalaxyPearl` `Player.cs:45405-45410` (item 5340) | NEUTRAL (no writer; the item 5340 row exists `itemdata.rs:5506` but no use-arm sets the flag; not persisted — no player-file writer) | **NO** — f33 does not touch `usedGalaxyPearl` (its diff is the torch scan + `torch_god.rs` + the 5043 favor; `git diff origin/main..f33 -- luck.rs` shows only the `nearby_torches` doc + the scan head/tail seam) | no |
| `brokenMirrorBadLuck` (`−0.25`) | PrettyMirror projectile 1085 `Projectile.cs:70919-70922` (`brokenMirrorBadLuckTime = 25200`) | NEUTRAL (the decay runs `luck.rs:188`; the projectile 1085 rotation arm exists `projectile.rs:2629` but no kill-writer sets the timer) | **NO** — f33 does not touch the mirror | no |
| `kiteLuckLevel` (`+0.1/3 · tier`) | buff 383 `Player.cs:9920-9924` (`buffTime/10800` clamp 0..3); the kite items `ItemID.Sets.IsAKite` | NEUTRAL (the buff-383 tier arm is wired `player.rs:6830`, but no kite item grants buff 383 — `IsAKite` has no port; the field stays stale-until-recomputed, the C# never zeroes it) | **NO** — f33 does not touch kites | no |
| `lantern_night` (`+0.3`) | `LanternNight.LanternsUp` `LanternNight.cs:17-27` | WIRED (the S1-F-14 scheduler copy `world_sim/player.rs:595`) | no | no |
| equipment tail (`clover/feather/horseshoe/coin`) | `UpdateEquips` `Player.cs:13050-13069` | WIRED (phase 2) | no | no |
| ladybug windows / coin / potion / stinky | `NPC.cs:81566-81652` / `:18115-18153` / buff 257 / buff 120 | WIRED (phases 2-4) | no | no |

**The verdict**: f33's torch-god event activates the **torch-luck scan's
head/tail** (the `happyFunTorchTime` event replaces the scan while it runs; the
trigger tail gates on the 5043 favor) — it does **NOT** activate any of the three
remaining-neutral factors (`usedGalaxyPearl`/broken-mirror/kite). Those stay
neutral until their own writers land (the galaxy-pearl use-arm + player-file
persistence, the PrettyMirror kill-writer, the kite-item family). The
garden-gnome `+0.2` term is **already live** (e26 phase 1) and is independent of
f33. **No e26 factor-activation code is owed to f33 or the gnome** — the f33
merge composes the torch-scan seam (e26's `torch_scan_step` is the no-event entry;
f33's `torch_scan_step_event` is the event-carrying one — keep f33's, the e26
`LuckState` fields are the shared input), and the gnome sensor is e26's own
landed work.

**Cross-note to f33**: the e26 `luck.rs` torch-scan deviation note ("the
`happyFunTorchTime`/`torchGodCooldown` head … not ported") is SUPERSEDED by f33's
`torch_scan_step_event` seam at merge — f33's tip already re-documents
`nearby_torches` as "the event rides `torch_god.rs`, W4-f33" and ports the head
(`:17733-17742`) + trigger tail (`:18066-18086`) in C# order. The e26
`torch_scan_step` (no-event) stays as the test entry; the player-context caller
(`player.rs::update_luck`) routes through f33's event entry at merge. The e26
ANALYSIS deviation bullet is updated to point at f33 (this retire phase).

## 3. The Frog 361 / Duck 362 DEDUPE — the claims-table (d37-E8 keep-one EXECUTED)

The dispatch's "Frog361/Duck362 DEDUPE (d39r/f18/c10-also-361/362 — d37d-E8
keep-one; publish claims-table)" resolves to the **d37-E8 work order, executed
on this branch** (`04dbdb0d2`). The claimants, verified at each pushed tip:

| kind | cs/net | claimant lanes (tip) | pushed wire id | E8/SEAL verdict | e26 action (EXECUTED) |
|---|---|---|---|---|---|
| **Frog** | 361 | e26 (`048cca6cf`), f18 (`a832d38ea`, batch 2) | e26 361, f18 243 | **e26 KEEPS** (research/108 E8 "e26 — 1 owned, 1 renumber": Frog 361→**744**, cs 361); f18's Frog drops to e26 | wire id 361→**744** (d37's Pirate owns proto 361, E1 HARD); cs/net stay 361; the kind/stats/FindFrame/bestiary/catch chain unchanged |
| **Duck** | 362 | e26 (`048cca6cf`), c10 (`5fcc68ae6`), f18 (`a832d38ea`, batch 2) | e26 362, c10 362, f18 244 | **c10 KEEPS** (research/108 E8 "Duck: drop e26@362 → keep c10@731"; SEAL S1 c10 moves four ducks→731-734); e26 + f18 DROP | the Duck variant + every arm DROPPED (enum/to_u16/from_u16/render_key/ALL_KINDS/CS_STYLES/data.rs tables/bestiary ENTRIES/netglue/entities/ui/assets); c10 lands its Duck@731 at merge |
| **LadyBug** | 604 | e26 (`048cca6cf`), d42 (`bac408be0`, dropped) | e26 604 | **e26 KEEPS** (the d42 ownership flip — d42 dropped its row at `49346dba6`, SEAL S1 accepts; E8's original "drop e26@604 → keep d42@604" is RETIRED) | unchanged (604, vanilla wire id kept) |
| **GoldLadyBug** | 605 | e26 (`048cca6cf`), d42 (`bac408be0`, dropped) | e26 605 | **e26 KEEPS** (the same d42 flip) | unchanged (605); the c8 `GOLD_PEER_TABLE` {604} row pre-tabled |

**The d39r/f18/c10 cross-reads (the dispatch's named claimants):**
- **d39** (`origin/task/W4-d39-npc-kinds-351-420` tip `a0d49dc1d`, research/125
  §351-420 census): d39 is **census-only** for 361/362 — its doc rows
  (`125-d39-band-351-420-census.md:61-62`) record "361 Frog → e26-luck-system
  (`048cca6c`)" and "362 Duck → e26 AND c10 … both claim 362, merge-wave dedupe".
  d39 rows NEITHER in `npc.rs` (its 7 kinds are 356/366/367/377/378/379/380 at
  port-local 817-823, the R7 re-pin). **d39 is not a Frog/Duck claimant in code** —
  it defers to e26/c10. No d39 dedupe action.
- **f18** (`origin/task/W4-f18-npc-kinds-verify-2` tip `a832d38ea`, research/64):
  f18 rows Frog (proto 243, cs 361) + Duck (proto 244, cs 362) in its first-20
  batch (`npc/data.rs:601`/`:960`, the "batch 2 360-364" comment). f18's
  research/64 scoreboard marks 361/362 "✓ W4-f18". **f18 is a same-kind duplicate
  port** (the E1 "49 same-kind duplicate ports" class) — at merge f18's Frog/Duck
  rows drop to the canonical owners (e26 Frog, c10 Duck). f18's protocol ids
  243/244 are port-local and do not collide with e26's 744/604/605. **No e26
  action against f18** — the merge dedupes f18's rows (the E8/SEAL keep-one rule:
  kinds-lane owns, verify-lane drops).
- **c10** (`origin/task/W4-c10-style68-waterfowl` tip `5fcc68ae6`, research/149
  SEALED): c10 rows the eight waterfowl kinds 362-365/602/603/608/609 (Duck 362
  proto 362, `npc.rs:19596`) + the style-68 swim machine + the `Transform(type-1)`
  dry-landing chain. c10 is the **canonical Duck owner** (E8/SEAL). c10's SEAL
  (research/149 §2.5) already receipts the e26 Duck drop direction. **The e26
  Duck drop composes with c10's roster** — c10's Duck@731 (the E8 move) lands at
  merge; the e26 `npc_ai_style68.rs` module (UNWIRED on e26, the c10 swim-form
  `Transform` landing target) stays as the shared seam (c10's 1080-module
  supersedes both d20/d76 per its SEAL §2.6 — the e26 style68 module is the
  unwired base, c10 wires it).

**The execution receipts (scripted, code-only — no cargo):**
- `to_u16`/`from_u16` bijection: **273/273 arms, zero duplicate ids, to-id-set ==
  from-id-set**, max id **744** (i16-safe). Frog 744/744; Duck arms 0/0.
- `ALL_KINDS`: decl `[NpcKind; 273]`, **273 rows**, Duck absent, Frog present.
- `BESTIARY_NPC_SHEET_IDS`: decl `[u16; 184]`, **184 distinct ids**, 362 absent,
  361/604/605 present; `bestiary_npc_sheets` vec **184 `atlas.sheet` rows** (the
  `every_bestiary_sheet_slot_opens_its_page_without_panicking` len-equality pin
  holds).
- `ENTRIES`: **252 rows** (the Duck credit dropped); `kind_from_net_id(361)` →
  Frog (net unchanged), `kind_from_net_id(362)` → None (the Duck entry gone,
  c10 lands it).
- The re-pinned witness `w4_e26_the_luck_critter_protocol_ids_round_trip` asserts
  `from_u16(362) == None`, `from_u16(731) == None` (c10's slot, not e26's),
  `from_u16(361) == None` (the Frog moved) — the dropped/moved slots. WRITTEN NOT
  RUN.

**Cross-note to d37**: the e26 E8 work order (1 move + 1 drop, the LadyBug drops
RETIRED by the d42 flip) is EXECUTED at `04dbdb0d2` — the SEAL S1 "e26 NOT
EXECUTED" receipt is now stale; e26's Frog is at 744, the Duck dropped, the
ladybug pair kept. The merger's bijection gate (552/583 grand pins) absorbs the
e26 273 (was 274) — the Duck row's removal is the −1 the SEAL's "61 drops owed"
counts.

## 4. The luck-family declaration (retire phase)

**The luck family is COMPLETE on this branch** (code-only; witnesses written not
run). The full `RecalculateLuck` formula (`Player.cs:29137-29164`) runs in C#
order with every term live or neutral-default:

- **The value + decay timers** (`luck.rs`): `LuckState` carries all 11 terms +
  the three `dayRate` decays (coinLuck `:18103-18110`, ladyBugLuckTimeLeft
  `:18836-18854`, brokenMirrorBadLuckTime `:29166-29182`); the value is never
  clamped (the −0.7/1.0 caps are UI-only `NormalizedLuck`).
- **The wired inputs** (phases 1-4): the torch-luck scan (`:17731-18088`, the
  13-verdict biome table, `torchLuck` floored at 0), the coin tiers
  (`:18115-18153` incl. the duplicated unreachable `>24900` branch), the ladybug
  kill/nearby windows (`NPC.cs:81566-81652`), the buff-driven inputs (257 potion
  tier, 120 stinky, 383 kite tier), the equipment tail (`:13050-13069`, the four
  `hasLuck_*` flags + coin, once per family), the carried clover scan
  (`:12723-12734`), the lantern-night + gnome per-tick copies, the town-death
  NPCLuck + CheckActive ladybug wires.
- **The roster** (E8 keep-one EXECUTED): Frog 361 (wire 744, data-only aiStyle-7
  walker) + LadyBug 604 / GoldLadyBug 605 (aiStyle 115, the faithful
  `AI_115_LadyBugs` machine `npc_ai_style115.rs`, `NPC.cs:47580-47714`); the Duck
  362 dropped to c10. The ladybug seams (`checkDead` kill + `CheckActive` nearby)
  are LIVE; the c8 generic sight provider auto-covers the bestiary unlocks at
  merge (§1).
- **The remaining neutral-default inputs** (their own tasks, not e26's):
  `usedGalaxyPearl` (the 5340 use-arm + player-file persistence), the
  broken-mirror writer (PrettyMirror projectile 1085 kill), the kite items
  (`IsAKite` + buff 383 grant). f33's torch-god event does NOT activate these
  (§2) — it ports the torch-scan head/tail seam only.
- **The consumers**: the gel drop rules + `drop_money` (`CommonDrop.cs:36`,
  `NPC.cs:79622`), the melee `DamageVar` (`Player.cs:44342`), `start_rain`'s coin
  gate (`Main.cs:64668`) read `player.luck()`. The ladybug-rain-boost seam feeds
  d130's canonical rain.rs at merge (§1a).

**Residual work is merge-time only**: the c8 UI `unlock_state_world` fold (§1),
the d130 rain.rs union (§1a), the f33 torch-scan event-entry compose (§2), the
c10 Duck@731 + style68 wiring (§3), the d37 bijection-gate re-pin (§3). The
galaxy-pearl/mirror/kite writers are their own lanes (the neutral-default inputs
stay live terms, the M2-BUFF-6 pattern). **No further e26 commits planned.**

## 5. Witness register (all WRITTEN NOT RUN — code-only mode)

| witness | file | pins |
|---|---|---|
| `w4_e26_the_luck_critter_setdefaults_rows_match_the_csharp` (re-pinned) | npc.rs | the three kinds' rows (Frog 361 / LadyBug 604 / GoldLadyBug 605), the Duck dropped |
| `w4_e26_the_luck_critter_protocol_ids_round_trip` (re-pinned) | npc.rs | Frog wire 744 / cs-net 361, ladybugs 604/605, `from_u16(362/731/361) == None` |
| `w4_e26_the_luck_critter_catch_and_release_chain_resolves` (re-pinned) | npc.rs | the Frog 2121 + ladybug 4361/4362 chains; the Duck 2122 chain waits for c10 |
| `w4_e26_the_luck_critter_bestiary_entries` (re-pinned) | npc.rs | the three entries' credits/net/names; the Duck entry gone |
| `w4_e26_the_luck_critters_have_no_debuff_immunity` (re-pinned) | npc.rs | the three null-rows |
| `w4_e26_the_waterfowl_hold_still_and_the_ladybugs_fly` (re-pinned) | npc.rs | the Frog data-only hold + the ladybug aiStyle-115 flight |
| `w4_e26_the_luck_tail_accumulates_in_the_csharp_order` | player.rs | the equipment-tail f32 order (phase 2) |
| `w4_e26_gnome_placement_tests` | world_sim/editing.rs | the tile-567 item-4609 placement + mid-air refusal (phase 4) |
| the aiStyle-115 RNG witnesses | npc_ai_style115.rs | the rearm draws on the caller's stream (phase 4) |
| the potion/ladybug/equipment-order witnesses | luck.rs / player.rs / potions.rs | phases 2-3 |

## 6. Verification record

- `git fetch origin --prune` + per-tip `git ls-tree docs/research/` sweep: 243
  first-free (237-242 claimed, §header).
- The claimant tips read via `git show <ref>:<path>` / `git grep <ref>` (direct
  paths, no junction recursion): c8 `7c2de9c06`, c10 `5fcc68ae6`, f18
  `a832d38ea`, d39 `a0d49dc1d`, d42 `bac408be0`, f33 `bffee350f`, g67
  `8b9a1ff46`, d130 `b8f655708`, d37 `2ea22ce3e` (the f33/d37/d130 tips advanced
  mid-retire; the census facts re-verified at the new tips — the d130 ladybug
  seam, the f33 torch-god trigger, the d37 E8 e26 receipt all unchanged).
- The E8 keep-one execution scripted pins (§3): bijection 273/273, ALL_KINDS 273,
  sheet tables 184/184, ENTRIES 252, Duck arms 0, Frog 744/744, max 744 i16-safe.
- The C# anchors re-read: `Player.cs:17731-18088` (torch scan), `:29137-29164`
  (RecalculateLuck), `:16726-16730` (gnome), `:45226-45243` (5043 favor),
  `:45405-45410` (galaxy pearl), `:9920-9924` (kite); `NPC.cs:55843-55900` (gnome
  petrification), `:81566-81652` (ladybug windows); `Projectile.cs:70919-70922`
  (mirror).
- No cargo/tests/rustfmt/graphify run (code-only mandate). No other branch
  touched; publication on `task/W4-e26-luck-system` only.