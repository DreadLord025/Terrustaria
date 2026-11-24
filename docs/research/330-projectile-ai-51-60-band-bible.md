# 330 — Projectile aiStyles 51–60: the band bible + the RETIRE seal (W4-d6, code-only)

Scan date: **2026-11-21** (worktree TR-W4-d6, branch `task/W4-d6-proj-ai-51-60`).

> **REGISTRY-RENAME 247 → 330 (this round).** The seal commit `174c0d541`
> claimed 247 from a tips-only sweep ("max claimed 246") — but 247 was
> **d10's published reservation, EXECUTED** (`72431e61f` 16:05:28,
> `247-projectile-ai-91-100-final.md`, 9 min BEFORE our 16:14:05 add).
> Registry-150 (`origin/task/W4-d54-dust-sites-world` tip `a15cb634b`)
> adjudicates the key twice: the round-4 §8.1 row 21 first assigned
> **328**, then the **§8 FINAL (Lead direct-append) §8.2** re-ordered the
> four 247 losers by claim time — d16-bible→327, g68→328, c3→329,
> **d6→330** — and declares "§8 IS AUTHORITY … lanes EXECUTE §8
> assignments ONLY". This rename executes the §8 FINAL row. Chain:
> 247→**330**. All on-branch refs updated (INDEX, ANALYSIS, PORT_MAP,
> anchors, the `projectile_gun_splash.rs` module doc); `git grep`
> zero-stale at the pushed tip.

Number assignment: **330** — the registry-150 §8 FINAL assignment (150 =
the collision registry, W4-d54). The original 247 self-pick is superseded;
no other key is claimed by this doc.

**Code-only mode: no cargo/rustc/rustfmt/tests/graphify invoked. Every
witness on the branch is WRITTEN NOT RUN; verification is deferred to the
merge gate per the task contract.**

C# cites are `decompiled/Terraria/Projectile.cs` (stable) unless named;
Rust cites are this branch's tree at the retire tips (§6).

## 1. The band census — 10/10 styles armed, 16/16 kinds rowed

The C# dispatch ladder (`AI()`, the `else if (this.aiStyle == …)` chain) and
the full `SetDefaults` membership (mechanical grep `aiStyle = 5[1-9]|60`,
16 assignment sites, every one re-read):

| Style | C# dispatch | Kinds (SetDefaults cites) | Port arm (this tree) | Owner |
| --- | --- | --- | --- | --- |
| 51 | `:28402-28503` | LostSoulHostile **293** (`:3826-3837`, hostile, pen −1, extraUpdates 1), LostSoulFriendly **297** (`:3870-3882`, friendly, pen 3, ID-static 10) | `projectile.rs` `ai_spectre_shot` (`:4138`) — dispatch `51 =>` (`:17924`); rows `data.rs:2850/2854` | base (M4-PROJ magic lane) |
| 52 | `:28504-28559` | SpiritHeal **298** (`:3883-3892`, extraUpdates 3), VampireHeal **305** (`:3965-3973`, extraUpdates 10) | `ai_heal_orb` (`:4670`) — dispatch `52 =>` (`:17934`); rows `data.rs:2864/2876` | **d6** `af75e6460` |
| 53 | `:28560-28563` → `AI_053_FrostHydra` `:37266-37490` (+ `AI_053_HandleSentryNPCTargeting` `:40194`) | FrostHydra **308** (`:3997-4008`, 80×74, sentry, life 36000, light 0.25, cold), SpiderHiver **377** (`:4615-4624`, 66×50), HoundiusShootius **966** (`:9143-9152`, 18×60) | inline dispatch `53 =>` (`:18576`) + `sentry_npc_targeting`; rows `data.rs:3604/3608/3612`, shots 309/378/967 rowed | base |
| 54 | `:28564-28772` | Raven **317** (`:4099-4111`, minion, `timeLeft *= 5` → 18000, local-immunity 10) | `ai_raven` (`:2226`) + `minion_separation` — dispatch `54 =>` (`:19985`); row `data.rs:3480` | base (S1-P-i25/i56) |
| 55 | `:28773-28844` | FlamingJack **321** (`:4139-4148`, 30×30 melee, tileCollide false, ignoreWater) | `ai_flaming_jack` (`:4731`) — dispatch `55 =>` (`:17942`); row `data.rs:2880` | **d6** `a91c25381` |
| 56 | `:28845-28866` | FlamingScythe **329** (`:4215-4225`, 80×80 hostile pen −1, life 420, light 0.25) | `ai_flaming_scythe` (`:4802`) — dispatch `56 =>` (`:17950`); row `data.rs:2886` | **d6** `b53ee70ab` + c6 dup (§3) |
| 57 | `:28867-28905` | NorthPole snowflake **343** (`:4326-4338`, pen 3, life 120, scale 1.1, cold); its spawn **344** (`:4339-4352`, aiStyle 1) rowed too | `projectile_ai_spear::ai_north_pole_flake` — dispatch `57 =>` (`:17686`), the 344 spawn drains via `spawn_with_data_ai_owned`; rows `data.rs:3992/3996` | base (spear lane) |
| 58 | `:28906-28937` | Present **351** (`:4411-4420`, hostile, pen −1, tileCollide false) | `projectile_ai_frostmoon::present` — dispatch `58 =>` kind-gated SANTA_PRESENT (`:17998`); row `data.rs:2562` | base (S1-D-4) |
| 59 | `:28938-28993` | SpectreWrath **356** (`:4464-4473`, 6×6 magic, extraUpdates 3) | `ai_spectre_wrath` (`:4590`) — dispatch `59 =>` (`:17976`); row `data.rs:2859` | base |
| 60 | `:28994-29309` | WaterGun **358** (`:4487-4496`), SlimeGun **406** (`:4868-4877`), ShimmerGunStream **1042** (`:9848-9857`) — all alpha 255, pen −1, extraUpdates 2, ignoreWater | `projectile_ai_guns::gun_stream` — dispatch `60 =>` (`:17962`); rows `data.rs:2892/2895/2899` | **d6** `5d81f2de0` |

Membership total: 2+2+3+1+1+1+1+1+1+3 = **16 SetDefaults rows over 16 band
kinds + the 344 aiStyle-1 spawn rider** (343's `NewProjectile` `:28902`).
Every row exists in `projectile/data.rs` at this tip (mechanical grep, cites
above); every dispatch arm is FIRST-match live (the S1-D-5 dead-arm purge
precedent — no duplicate 51–60 labels exist in the match).

**Boundary closure.** Style 50 (the Inferno Fork family) is the d5 band
(`156-projectile-ai-41-50-census.md`, W4-d5); style 61+ starts the d7 band
(`26e-projectile-ai-61-70-census.md`, W4-d7). The 51 arm's head `:28402`
sits directly below the 50 arm's tail `:28401`, the 60 arm's tail `:29309`
directly above the 61 head — no gap, no overlap.

## 2. The spawn sources + the use-path census (the d6 residuals, all landed)

| Chain | C# anchor | Port | Commit |
| --- | --- | --- | --- |
| Spectre Hood/Mask set bonuses → ghostHeal/ghostHurt flags | `ArmorSetBonuses.cs:439-440`/`:253-262` | `armor_sets.rs` rows + `Player.ghost_heal/ghost_hurt/life_steal` | `bd0865edd` |
| ghostHeal → the 298 intent | `Projectile.cs:12064-12096` via PVE `:13816-13821` | `ghost_heal_on_hit` (share decay, lifeSteal budget, folded scan) | `bd0865edd` |
| ghostHurt → the 356 intent | `:12012-12062` via PVE `:13822-13825` | `ghost_hurt_on_hit` (1500 budget, Manhattan-800, the `:12039` additive-halfwidth quirk) | `42cfa6d8a` |
| vampireHeal → the 305 intent | `:12098-12106` via PVE `:13812-13815` (OUTSIDE canGhostHeal) | the PVE-walk 7.5% share + `pending_vampire_orbs` drain | `42cfa6d8a` |
| the 304 knives (aiStyle 2, the heal SOURCE) | `:3952-3964` + volley `Player.cs:48157-48188` + probe `:20007-20011` | `VAMPIRE_KNIFE` row + the knives volley use arm + `ghostDmg` decay `Player.cs:24460-24467` | `42cfa6d8a` |
| the 298 contact-heal pass chain | `:28513-28526` | hood hit → intent → centred at-rest 298 → next-pass blend → contact heal + Kill (witness `w4_d6_the_hit_intent_drains_into_a_flying_orb_that_heals_on_contact`) | `5f12a2c48` |
| the hood/mask head vanity | `PlayerDrawLayers.cs:2267` (normal-hat branch, 52px crop) | `Armor_Head_101` (Hood 1503) / `Armor_Head_156` (Mask 2189) packed + helmet-push resolve | `671128ae1` |
| the gun items' use path | items **2272/2610/5495** (`Item.cs:22876-22889`/`:25504-25517`/`:43313-43325`), the ai1 spawn rolls `Projectile.cs:10914-10919` (358 Next(10,31)×0.1 / 406 Next(10,21)×0.1), the 5495 ai1=1 use arm `Player.cs:49124-49127` | `world_sim/editing.rs` payloads; 3059 is PaintingTheSeason (`ItemID.cs:7801`), NOT a gun | `861db5c84` |
| the gun Kill-splash dust | `:74029-74147` | `projectile_gun_splash.rs` (§4) | `1b54dcbbd` |
| the ShimmerGun 5495 buff-353 chain | use arm → contact `AddBuff(353,100)` `:29039` → the f47 flag/ramp tail | `SHIMMER_IMMUNITY` + `for_kind` (§5) | `e2561efe1` |
| the light-toggle family | `Wiring.cs:2800-3072` + `WorldGen.TryToggleLight` `WorldGen.cs:72970-73024` | `tr-core/src/wiring.rs` free-fn family (§3) | `5d81f2de0` |

## 3. The keep-one adjudications (merge-time records, no code change)

1. **The light-toggle family** (ANALYSIS "W4-d6 residual — the light-toggle
   family keep-one", `ec72de402`): the d6 free-function family is the
   canonical superset (the only `TryToggleLight` dispatch + forced-state
   Torch/CampFire ports). KEEP the base `Wiring` methods + the e20
   `ee6df0f2` witnesses for the `HitWireSingle` lane (they carry the
   `SkipWire` bookkeeping); c42 `6b68736a`'s case-4 torch + campfire arms
   dedupe onto the d6 statics with a skip-set parameter at merge. The
   `kind != 215` vs `SET_CAMPFIRES.has` gate pair is equivalent ({215},
   `TileID.cs:616`); prefer the set read at the dedupe.
2. **The aiStyle-56 duplicate** (ANALYSIS "the aiStyle-56 duplicate
   keep-one", `451cace15`): d6 `b53ee70a` vs c6 `040b33b5` arm-by-arm
   identical maths/rows. Recommended composition: the c6 `kill_step`
   signature + the d6 friendly-hold; all FIVE shared sites listed in the
   ANALYSIS note; the witness sets are complementary; c6's NPC chain is
   the spawn source. The merger keeps ONE arm + both witness sets.
3. **The gun-splash Kill dust**: §4 below (the g65 late dup, KEEP d6).
4. **ShimmerImmunity vs f47**: §5 below (COMPOSE, not a dup).

## 4. The gun-splash keep-one — KEEP d6, the g65 late-dup DROP-HUNKS

**Timeline (committer dates, +02:00, first-pushed-wins per registry-150):**

* `1b54dcbbd` **09:38:32** — this lane ports `projectile_gun_splash.rs`
  after censusing the g65 tip `7f0dc8649` (its batch-6 range spans
  `:73952-74333` but its kind list SKIPPED 358/406/1042).
* `1bbeec5b5` **10:22:59** — the g65 lane's batch 18 ("the smoke/cursed-
  flame Kill death-dust bursts (1037/1049/399/**358/406/1042**/323/589/
  311/316)") re-claims the three sites without a fresh d6-tip read; the
  claim is live at its tip `f6e07bb90`.

**The verdict: KEEP the d6 module** — first claim AND the faithful
velocity base. The g65 copy's `smoke_puff` reads
`s.old_vel * 0.2` where `DeathSnapshot::of` sets `old_vel: p.vel` (its own
doc: "the snapshot takes the slot's current velocity as the stand-in …
Kinds that need the true last-sub-step velocity are deferred"). Two
behaviour deltas vs the C# (`:74031`/`:74067`/`:74101`
`velocity = oldVelocity * 0.2f`):

1. **The scale death** (the common gun death — the 0.015 decay): the C#
   latches `oldVelocity = velocity` AFTER the ×5 kick
   (`:28999-29000`), so the burst base is `kicked × 0.2`. The d6 hook
   reads exactly that (`projectile.rs` death hook: `let base = if
   p.scale <= 0.0 { p.vel } else { old_velocity };`). The g65 snapshot
   is taken BEFORE the d6 hook's velocity write in its own tree (no d6
   hook exists there), so its base is the UNKICKED velocity — a ×5
   velocity error on every dust of every scale death.
2. **Every other death path**: the C# reads the sub-step-start capture
   (`:16766`); the g65 stand-in reads the LIVE velocity (documented
   deferred on its side). The d6 snapshot carries the true
   `old_velocity` latch.

The dust bodies themselves are draw-for-draw identical (the same 50/40/50
counts, the 3×Next(2) alpha gates, the scale/noGravity pick, the jitter
chain, the 1042 Next(6) palette LAST, the ×0.8 add) — verified arm-by-arm
against `:74029-74147` at both tips; only the base differs. Both lanes
use the same gallop queue→drain seam and the same live
`UpdateProjectiles` stream, so the drop is stream-neutral.

**The exact g65 drop-hunks (at tip `f6e07bb90`, for the g65 lane/merger —
this lane cannot touch other branches):**

| # | File | Lines | Content |
| --- | --- | --- | --- |
| D1 | `crates/tr-app/src/projectile_death_dust.rs` | `:2786-2794` | the three match arms `358 => smoke_puff(…)` / `406 => …` / `1042 => …` + their three comment pairs |
| D2 | same | `:4834-4885` | `fn smoke_puff` + its doc comment (no other caller — grep-verified: exactly the three D1 sites) |
| D3 | same | `:6616-6634` | the `w4_g65_the_smoke_puffs` witness + its doc comment |

The g65 anchors rows keep their markers (`pub(crate) fn drain`,
`620 | 618 =>` — neither touches the dropped hunks). The g65 26i
keep-one receipt's site accounting stays valid (the three sites are
COVERED — by d6, not g65; the census row gains the cross-reference).
The d6 side KEEPS: `projectile_gun_splash.rs` whole, the pool's
`pending_gun_splashes` + `drain_gun_splashes`, the death hook
(`projectile.rs`, the `!died_out_of_world` + kind gate at the C# body
position above `special_arrow_kill`), the pass-tail drain
(`world_sim/projectiles.rs`), the four mirror witnesses, the anchors row.

**Merge order:** either order is safe — the two modules never share a
function name; if g65 lands first the double-burst window exists until
the D1–D3 drop executes (the merger MUST run the drop, not defer it: two
50-dust bursts on one death is a visible + stream-length regression).

## 5. ShimmerImmunity × f47 — the COMPOSE contract (+ the c9 THIRD-copy keep-one)

The f47 tip was re-read whole on the buff path at `18fe75f2e` and
RE-VERIFIED at the moved tip `057f44a68` (the splash-arbiter rounds are
item-lane docs; the buff path is byte-unchanged):

* f47 CONSUMES `buff_immune[353]`: `npc_buffs.rs:694-705` (the buff-scan's
  `353 =>` immune branch — DelBuff mid-scan, `NPC.cs:92133-92143`) and
  `npc.rs:5078-5083` (`npc_shimmer_transparency_tick`'s immune argument —
  the ramp) + `npc.rs:5072` (the fresh-per-tick `shimmering` flag). Its own
  tests SET the flag by hand (`npc_buffs.rs:1356`/`:1389`).
* f47 NEVER WRITES the flag from the kind: `for_kind` (`npc_buffs.rs:515`)
  on its tip has no `SHIMMER_IMMUNITY`/353 application (grep: zero hits for
  `ShimmerImmunity|SHIMMER_IMMUNITY` in its whole `crates/` tree). Every
  real NPC would take buff 353 unguarded until a writer lands.
* **d6 residual 9 (`e2561efe1`) IS that writer**: the verbatim 131-id
  `SHIMMER_IMMUNITY` set (`NPCID.cs:10467`) applied at
  `NpcBuffs::for_kind` (`npc_buffs.rs:542`, the `NPC.cs:17838` line),
  keyed on `cs_id`.

**Contract: COMPOSE at merge — keep BOTH halves, no drop on either
side.** d6's `for_kind` write feeds f47's scan/contact reads; the two
hunks are in different functions of the same file (`for_kind` vs the
buff-scan arm), a clean textual union. The d6 four null-row witness
adjustments (the 353 exclusion — `npc.rs:30658`, `:31699`, `:31813`,
`:32041`, each `enumerate()`-gated `i == 353`/`i != 353`) stay; the f47
hand-set-flag tests stay (they test the scan, not the writer). The 1042
gun contact (`AddBuff(353,100)` through the immune guard,
`projectile_ai_guns.rs:132`, the witness `:491`) and the f47
`shimmer_wet_contact_pass` (`world_sim/npcs.rs:3517-3521`, the
`try_adding_repeated_buff(353,100,10)` at `NPC.cs:93071-93074`) are
disjoint sites — the gun NEVER writes `NPC.shimmerWet` (the census answer
NO stands, ANALYSIS residual 9).

### 5a. The THIRD writer — c9's `shimmer_immune` (KEEP-ONE, this round's find)

The §5 compose above missed a third claimant: **W4-c9**
(`task/W4-c9-style67-snails`, the `2042d77f1` residual round, 07:13:30 —
THREE HOURS BEFORE d6's `e2561efe1` 10:11:37) ports the SAME
`NPC.cs:17838` write: `fn shimmer_immune(cs_id: u16)` (its
`npc_buffs.rs:505-547`, the 131 ids as a range-compressed `matches!`) +
the identical `buffs.buff_immune[353] = shimmer_immune(kind.cs_id())`
line inside `for_kind` (`:586`). Its FINAL (`349-snail-family-final.md`
§5 "Immunity + shimmer compose", tip `2267c93e8`) names f47 but NOT d6 —
neither lane saw the other's writer. **Two `for_kind` 353-writes is a
merge conflict in the SAME function tail; the sets are semantically
identical** (both verbatim `NPCID.cs:10467`; c9's compression covers
245..=249 / 381..=421 / 423..=429 / 516..=523 / 551..=578 / 657..=660 /
678..=683 + the 24 singletons — hand-checked against the d6 flat list:
131 ids, zero drift, the 422/550 gaps preserved in both).

**KEEP-ONE verdict: KEEP c9's `shimmer_immune(cs_id)` + its `for_kind`
write (first-claim 07:13:30, registry-150 first-pushed-wins); DROP d6's
`SHIMMER_IMMUNITY` const + the d6 `for_kind` write line at merge.**
Rationale beyond the timestamp: c9's predicate shape serves its
Shimmerfly-677 census witness directly and its range compression is
documented set-exact; d6's flat list carries the same ids. The d6
DROP-hunks (this branch, for the merger):

| # | File | Lines (this tip) | Content |
| --- | --- | --- | --- |
| S1 | `crates/tr-app/src/npc_buffs.rs` | `:149-169` | the `SHIMMER_IMMUNITY` const + doc (incl. the RETIRE KEEP-ONE note; c9's `shimmer_immune` survives) |
| S2 | same | `:543-548` | the d6 `for_kind` 353-write block (comment + line; c9's `:586` write survives) |
| S3 | same | `:1135-1163` | `w4_d6_the_shimmer_immunity_set_gates_buff_353` — FOLD, not blind-drop: its KingSlime/ForceBubble/MartianWalker/Hungry AddBuff-refusal rows + the 422/550 gap pins re-point at `shimmer_immune` (c9's own witness `:1194` covers Snail-trio/Shimmerfly/BlazingWheel/ForceBubble/Zombie — the union covers every d6 pin EXCEPT the 131-length + 422/550 set-shape asserts, which must move onto c9's fn) |

The d6 gun-contact witness (`projectile_ai_guns.rs:491`, reads
`buff_immune[353]` through `for_kind`) is writer-agnostic — it stays
unchanged under either writer. The four d6 null-row `i == 353` witness
exclusions stay (they test `for_kind`'s output, not the set). The
`anchors.txt` row keyed `const SHIMMER_IMMUNITY` must re-point to
`fn shimmer_immune` at the merge (the marker moves with the keeper).
**f47 composes with the c9 keeper exactly as §5 describes — the compose
contract itself is unchanged, only the writer's identity flips.**

## 6. The 601 correction + the teleport-AI TASK SEED

**The row-shape fix LANDED this retire** (`388530671`): the d18 phase-3
finding (`dd7b1b0a`, cross-noted in its SEALED 26h) — 601 is C#
`PortalGunBolt` (`ProjectileID.cs:1756`), the `SetDefaults` block
`Projectile.cs:6502-6510` re-read whole: 10×10, aiStyle **1** (`:6506`,
NOT 0), alpha 255 (`:6507`), friendly (`:6508`), extraUpdates 30
(`:6509`); the stock penetrate 1 (`:774`), timeLeft 3600 (`:787`),
tileCollide (`:775`) stand. The M7-WIRE-2 row (blame `85e1d416e`) had
carried penetrate −1 / alpha 0 under a cannon-ammo comment; 178 is
ConfettiGun (`ProjectileID.cs:910`, row `:2641-2649`, UNTOUCHED). The
`CANNON_EXPLOSIVE` name deviation is recorded at the const (the
`POISON_DART`/645 precedent — value/row/arm correct, rename deferred).
The row leaves the `BASELINE_ROWS` reset sweep (39→38). The d18
gate-refusal witness (`w4_d18_the_601_portal_bolt_gate_refusal_cross_note`,
on the d18 branch) pins the OLD row shape (`penetrate == -1`, `alpha ==
0`) — **it must be re-pointed at merge** to the fixed shape (the two
asserts flip to `1`/`255`; the gate refusal `:12448` itself is
row-shape-independent and stays). One new row-shape witness
(`w4_d6_retire_the_601_row_is_the_portal_bolt_not_cannon_ammo`) WRITTEN
NOT RUN.

**The teleport AI: TASK-SEED, not a small port.** The d6 gap census
(`Projectile.cs:18099` → `GameContent/PortalHelper.cs:216`) re-read whole
finds a subsystem, not a function:

* `TryPlacingPortal` (`PortalHelper.cs:216-244`) needs `FindCollision`
  (`:311-322`, a `Utils.PlotLine` tile walk), `FindValidLine`/
  `IsValidLine`/`CanPlacePortalOn`/`DoesTileTypeSupportPortals`/
  `BlockPortals` (`:246-309`, the 3-cell slope-matched line scan + the
  tileCut/BreakableWhenPlacing/tileSolid gates), `SupportedTilesAreFine`
  (`:470+`, the 8-way angle-quantised support probe), `AddPortal`
  (`:324-336`: the 602 `NewProjectile` + direction/netUpdate),
  `RemoveMyOldPortal` (`:338-349`, the per-owner 2-portal budget) and
  `RemoveIntersectingPortals` (`:351-371`, the ±22px `GetPortalEdges`
  line-vs-line `Collision.CheckLinevLine`).
* The statics: `FoundPortals[256,2]`, `PortalCooldownForPlayers[256]`,
  `PortalCooldownForNPCs[maxNPCs]`, the normalized `SLOPE_EDGES`,
  `anyPortalAtAll` (`:10-62`), refreshed per tick by
  `UpdatePortalPoints` (`:64-98`).
* The traversal: `TryGoingThroughPortals` (`:105-214`) — the player
  gravDir-aware entry, `GetPortalOutingPoint`'s 8-way exit anchor
  (`:401-431`), `Player.Teleport`/`NPC.Teleport(…,4,…)` + the −3 rise,
  the cooldown writes, the MP `SendData(100/23/95)`.
* The 602 gate projectile itself (aiStyle 114, `Projectile.cs:6511-6520`,
  the `:18079-18083` movement arm: `position += velocity*0.99`,
  `velocity = lastVelocity.SafeNormalize(Zero)*0.0001`) is UNROWED and
  UNARMED; the 600 holder (aiStyle 75, `:6490-6501`, the `AI_075` 600
  arm `:66791-66830` — the mouse-aim + the speed-3 601 spawn + the
  30-tick kill) shares its style with the Last Prism/Blaster/Drill but
  has no ported arm (the `75 =>` dispatch's else-branch would misroute
  it to `ai_last_prism` — a kind gate is owed with the row).
* The 601 collision arm (`HandleMovement` `:18084-18103`): the
  Item114/115 cue by ai1, the owner-gated `TryPlacingPortal(this,
  lastVelocity, velocity)`, `position += velocity`, `Kill()` — plus the
  Kill dust (`:72454-72468`: 6× dust 263, the `GetPortalColor` tint
  A=255, `RotatedByRandom(2π)×3×NextFloat` + `velocity/5`, scale 2,
  noLight/noGravity) and `GetPortalColor` (`:373-392`, the SP hsl pair
  0.12/0.52 vs the MP per-player hue ladder, A=66).
* The use arm: item 3384 (`ItemID.cs:8451`, `Player.cs:48643-48647` —
  `ai1 = (altFunctionUse == 2) ? 1 : 0`, the right-click portal pair),
  the `portalPhysicsFlag` movement hook (`Player.cs:25652`) and the
  gamepad grapple exclusion (`:6315`).

**Seed scope (one task, suggested slug `task/W4-d6e-portal-gun` or the
next free d-lane row):** the 600/601/602 rows + the 75-arm kind gate +
the 601 HandleMovement arm + the PortalHelper placement/traversal
statics on the World/Player seam + the Kill dust + the draw colours;
the MP messages ride the net lane's reservation convention. Estimated
~600 C# lines + the draw hook — beyond a retire-round port, hence the
seed. NOTHING of it lands on this branch beyond the row fix + the
census; the 601 slice-B flight/fade machine
(`projectile_ai001_b.rs:153-166`, `:437-443`) is already correct and
stays.

## 7. The RETIRE seal — the ledger + the band declaration

The d6 lane's full commit ledger (origin/main..tip, oldest first):

| Phase | Commits | Content |
| --- | --- | --- |
| main slice | `af75e6460`, `a91c25381`, `b53ee70ab`, `5d81f2de0`, `5ae2978c9` | styles 52/55/56/60 + rows + the TryToggleLight family + the hostile-lane witness fix |
| residuals 1–9 | `ec72de402`, `861db5c84`, `bd0865edd`, `451cace15`, `42cfa6d8a`, `671128ae1`, `5f12a2c48`, `1b54dcbbd`, `e2561efe1` | §2's table + the two keep-one notes |
| witness fixes | `e422878e0`, `ae5a5964d`, `7edfcd495` | the at-rest epsilon, the leech same-cell re-hit + the chain mid-assert drop, the doc cross-ref |
| retire | `388530671`, `174c0d541` + this round's commits | the 601 row fix (§6), the g65 keep-one (§4), the f47 compose (§5), the seal; this round: the registry rename 247→330, the c9 third-writer keep-one (§5a), the d53/g64 disjoint receipt (§8), the gun/heal trail activation contract (§9), the g65 D1–D3 re-pin at the moved tip (§4a) |

**SEALED + band-51-60 declaration:** 10/10 styles armed, 16/16 kinds
rowed (+ the 344 rider), the boundaries closed both ends (§1), every
spawn source ported or owned (§2), the keep-one/compose contracts
published (§3–5 + §5a), the 601 row corrected and its teleport remainder
SEEDED (§6), the co-lane disjointness pinned to exact line ranges (§8),
the dust-lane activation contract published (§9). Remaining owned
elsewhere: the style-56 merge composition (c6/d6 merger), the
light-toggle dedupe (c42/e20 merger), the g65 D1–D3 drop (g65
lane/merger), the d6 S1–S3 ShimmerImmunity drop (merger, §5a), the d18
601-witness re-point (merger), the trail-dust wiring (§9, the dust-lane
merger), the portal-gun subsystem (the §6 seed). **No census/arm/row
work is owed on 51–60.**

**Witness register (WRITTEN NOT RUN, code-only):** the band's ~40
witnesses across `projectile.rs` (the 52/55/56 arms + the PVE-walk
ghost/vampire chain + the row pins), `projectile_ai_guns.rs` (8),
`projectile_gun_splash.rs` (4), `world_sim/editing.rs` (4 use-path),
`world_sim/projectiles.rs` (1 pass-chain), `armor_sets.rs` (1),
`player.rs` (2 budgets), `npc_buffs.rs` (1 set gate),
`render/player_layers.rs` (1 vanity), `projectile/data.rs` (the retire
row-shape witness). Verification: direct C#/Rust re-reads + mechanical
greps only; **no compiler invoked** — the merge gate owns the run.

## 8. The ghostHurt/vampireHeal × d53-wet / g64-trail DISJOINT receipt (exact line-ranges for the merger)

The residual-5 census said "disjoint"; this section pins the EXACT
ranges at the current pushed tips so the merger can verify mechanically.
Tips read: d53 `c0dc7d37f`, g64 `3e581c8ce`, this branch `174c0d541`+.

### 8a. The C# regions are disjoint by construction

| Piece | C# region | Phase |
| --- | --- | --- |
| d6 ghostHurt | `Projectile.cs:12012-12062` (the fn), called at `:13822-13825` | `Damage_PVE_Inner` — the HIT walk |
| d6 vampireHeal | `:12098-12106`, called at `:13812-13815` | the HIT walk, OUTSIDE canGhostHeal |
| d6 ghostHeal | `:12064-12096`, called at `:13818-13821` | the HIT walk |
| d53 wet machine | `:16786-17007` (the 8 splash sites `:16851-16987`) | `Update` — post-AI, pre-HandleMovement |
| g64 gun trail (406) | `:29197-29233` (sites `:29205`/`:29229`) | `AI()` aiStyle-60 — inside the arm |

`:12012-13825` (damage) < `:16786-17007` (wet) < `:29197-29233` (AI
dust): three different functions, zero shared C# lines. The d53 FINAL
item-3 census (its `8f5692786`) and the g64 claim doc both exclude the
damage-walk region; neither lane's tree contains `ghost_hurt_on_hit` /
`vampire_heal_on_hit` / `ghost_heal_on_hit` (git grep at both tips: zero
hits — the fns exist ONLY on this branch).

### 8b. The Rust seams are disjoint too (the exact ranges)

**d6 side (this tip, `crates/tr-app/src/projectile.rs`):**

| Piece | Lines |
| --- | --- |
| `ghost_heal_on_hit` | `:16753-16789` |
| `ghost_hurt_on_hit` (doc `:16791-16816`) | `:16817-16885` |
| `vampire_heal_on_hit` (doc `:16887-16894`) | `:16895-16913` |
| the PVE-walk call block (`flag10` `:23768`, the 304 arm `:23778-23785`, the canGhostHeal gate `:23788`, the hood `:23790`, the mask `:23794-23806`) | `:23759-23808` inside `damage_pve` (`:23052`) |
| the spawn-intent queues (`pending_heal_orbs`/`pending_wraths`/`pending_vampire_orbs` fields `:14740/:14746/:14751`, inits `:15158-15160`, drains `:15271/:15279/:15286`) | the pool struct |
| the intent drains → `spawn_with_data_ai_owned` | `world_sim/projectiles.rs:621/:644/:664` |

**d53 side (tip `c0dc7d37f`):** `projectile_wet.rs` — `wet_factor`
`:91-102` (the S1-P-i57 staged pick), `wet_block` `:131-246`; the
dispatch call `projectile.rs:20184-20190` (the kill-latch arm) + the
`wants_wet_splash` forward `:20215-20230`; the slot field
`wants_wet_splash` `:773`, the pool fields `pending_dust_bursts`
`:14806` + `water_style` `:14811`. **Zero overlap with the d6 ranges**:
the d6 fns sit at `:16753-16913` (d53's file has no code there — its
`ai_spectre_wrath` doc at `:4614/:4639` only CITES `:12060`, no body),
the d6 PVE block at `:23759-23808` (d53's damage-walk hunks are
elsewhere), the pool fields are DIFFERENT names at different offsets.
The one adjacent seam: d53's wet-block kill (`kill_time_left` latch)
feeds the SAME end-of-tick death hook the d6 gun-splash uses — a
composition, not a conflict (a gun killed by water still splashes; the
C# `Kill()` runs the same `:74029-74147` arms).

**g64 side (tip `3e581c8ce`):** `proj_trail_dust.rs` — `trail_406`
`:134-205` (doc `:127-133`). UNWIRED at its tip (zero call sites outside
the module — the 406 row is d6's, unmerged there). NOTE the g64 doc's
"Nebula Blaze" label on `trail_406` is a census typo: 406 =
`ProjectileID.SlimeGun` (`ProjectileID.cs:1366`), the C# block
`:29197-29233` is the SlimeGun's blue slime — the body itself is
correct. **Zero overlap**: the g64 lane touches no damage-walk code, no
`projectile_wet`, no pool intent queue.

### 8c. The merge-order note

All three land in `projectile.rs` but in disjoint hunks (the d6
`:16753-16913` insert, the d6 `:23759-23808` damage-walk insert, the d53
`:20184-20230` dispatch insert, the g64 trail call-site insert inside
the d6 gun arm — see §9). The pool-struct inserts are all append-shape
at the same `pending_catches` neighbourhood: d6 adds five fields
(`:14733-14757`), d53 adds two (`pending_dust_bursts`/`water_style`) —
textual-adjacent, mechanically resolvable, no semantic interaction.

## 9. The dust-lane ACTIVATION contract (the d6 rows arm the co-lane emitters)

The g63/g64/e3 trail lanes wrote their gun/heal-orb emitters UNWIRED
(their docs: "the row is W4-d6's, unmerged"). This branch's rows +
machines are the activation gate. The merger wires each emitter INTO
the d6 machine at the C# position — the d6 draw-count census stubs are
the exact replacement targets:

| Emitter (co-lane tip) | C# sites | d6 replacement target (this tip) | Wiring |
| --- | --- | --- | --- |
| g63 `vampire_heal_trail` (`projectile_trail_dust.rs:2101-2121`, tip `10775b366`) | `:28533-28545` (3× dust 183) | `ai_heal_orb`'s "dust trails — SKIPPED" comment (`projectile.rs:4709-4710`) | the 305 arm's per-tick trail, after the contact/blend, before the integrate |
| g63 `spirit_heal_trail` (`:2128-2148`) | `:28546-28558` (5× dust 175) | same site, the else (298) arm | same |
| g63 `water_gun_stream` (`:2159-2203`) | `:29158-29196` (6× dust 211 + 3 alpha gates + the Next(4) extra) | `gun_stream`'s `WATER_GUN \| SLIME_GUN` census arm (`projectile_ai_guns.rs:227-236`) — the 358 half | replace the draw-count loop with the emitter call |
| g64 `trail_406` (`proj_trail_dust.rs:134-205`, tip `3e581c8ce`) | `:29197-29233` (6× dust 4 (0,80,255,100) + the Next(4) seventh) | the same census arm — the 406 half | same (the SlimeGun, §8b's typo note) |
| e3 `style60_shimmer_gun_trail_1042` (`projectile_dust.rs:1652-1700` + `shimmer_gun_color` `:1703-1710`, tip `d7bffece4`) | `:29238-29303` (6× dust 324 + 3 alpha gates + Next(6) colour + the Next(4) extra with its OWN Next(6)) | `gun_stream`'s `SHIMMER_GUN_STREAM` census arm (`projectile_ai_guns.rs:237-249`) | same |

**The stream argument:** each d6 census stub draws EXACTLY the explicit
`Main.rand` gates its emitter draws, in the C# order (the 358's 6×3
Next(2) + Next(4) verified against `:29158-29196` this round). The stubs
are a documented SKIP of the `Dust.NewDust` internals (the module doc:
"the bodies and their internal rolls go with the dust pool"); the
emitters draw those internals through `DustDraw(rng)` at the C# position
(the spawn precedes its three alpha gates, `:29167` before `:29168`).
Replacing the stub with the emitter call therefore RESTORES the C#
stream (internals + gates interleaved per dust) — it is not a no-op on
the d6-only tree, it is the owed fidelity completion, and the
explicit-gate ORDER survives because the emitter draws the same gates
in the same relative slots. The d6 gun witnesses that pin the gate
INDICES on a recording rng (`rng.ranges[18] == (0,4)`,
`projectile_ai_guns.rs:394`; the 1042's `:473-475`) must be re-pointed at
the wiring — the emitter's internals interleave additional draws, so
the index pins fold into the co-lane mirror witnesses (g63
`w4_g63_the_water_gun_alpha_gates` `:3315` +
`w4_g63_the_heal_orb_trails` `:3295`, g64
`w4_g64_406_mirrors_the_alpha_ladder_and_seventh_gate` `:2262`, e3
`w4_e3_style60_shimmer_gun_1042_trail_mirror` `:4150`), which replay the
FULL stream; the d6 witnesses' non-dust assertions (the buff scans, the
tile walks, the accumulator) stay untouched. The g63 doc-66 census-error
correction (the `:29245/:29284` pair is the 1042's, NOT the 358's — e3
research/231 + d53 `c0dc7d37f` both record it) is already reflected in
the table above. **No d6 code change is owed** — the stubs are the
documented seams; the wiring is the dust-lane merger's drop-in (the e3
`super_star_dust` precedent).

## 4a. The g65 D1–D3 re-pin at the moved tip (this round)

The g65 tip moved `f6e07bb90` → `37ec2f3aa` (the 346 registry rename +
the family-final doc). The three drop-hunks RE-VERIFIED byte-identical
at `37ec2f3aa`: D1 the arms `projectile_death_dust.rs:2786-2794`, D2
`fn smoke_puff` `:4834-4885`, D3 `w4_g65_the_smoke_puffs` `:6616-6634`
(the fn body `:6620-6634`, doc `:6616-6618`). The g65 lane's own FINAL
(`346-g65-kill-death-dust-family-final.md` §2b, hunks 15–17) ADOPTED the
d6-SURVIVES verdict and publishes the same three ranges — the keep-one
is now MUTUAL (both lanes' sealed docs agree; the merger executes either
copy of the instruction). The g65 26i accounting's "393 covered" becomes
390 + the 3 gun sites other-lane-claimed (d6) + 6 + 7 = 406, zero
unclaimed — unchanged by the rename.