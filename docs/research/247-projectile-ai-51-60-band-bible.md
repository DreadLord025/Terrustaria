# 247 — Projectile aiStyles 51–60: the band bible + the RETIRE seal (W4-d6, code-only)

Scan date: **2026-11-21** (worktree TR-W4-d6, branch `task/W4-d6-proj-ai-51-60`).
Number assignment: **247** — the fresh all-tips `git ls-tree docs/research/`
scan (every `origin/task/W4-*` tip, leading-number keys ≥ 150) finds the max
claimed bare number **246** (`246-projectile-ai-91-100-final.md`, W4-d10 tip
`e1519df75`); 247 is the first free number per registry-150's "237+ stay FREE"
rule (150 = `38bfd1d22`, W4-d54). The known 237–244 collisions belong to their
owners (registry-150 §1 + the g58 243 seal-verification); this doc claims
exactly one free key, no letter suffix.

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
| 51 | `:28402-28503` | LostSoulHostile **293** (`:3826-3837`, hostile, pen −1, extraUpdates 1), LostSoulFriendly **297** (`:3870-3882`, friendly, pen 3, ID-static 10) | `projectile.rs` `ai_spectre_shot` — dispatch `51 =>` (`:17924`) | base (M4-PROJ magic lane) |
| 52 | `:28504-28559` | SpiritHeal **298** (`:3883-3892`, extraUpdates 3), VampireHeal **305** (`:3965-3973`, extraUpdates 10) | `ai_heal_orb` — dispatch `52 =>` (`:17934`) | **d6** `af75e6460` |
| 53 | `:28560-28563` → `AI_053_FrostHydra` `:37266-37490` (+ `AI_053_HandleSentryNPCTargeting` `:40194`) | FrostHydra **308** (`:3997-4008`, 80×74, sentry, life 36000, light 0.25, cold), SpiderHiver **377** (`:4615-4624`, 66×50), HoundiusShootius **966** (`:9143-9152`, 18×60) | inline dispatch `53 =>` (`:18576`) + `sentry_npc_targeting`; shots 309/378/967 rowed (`data.rs:3585/3589/3592`) | base |
| 54 | `:28564-28772` | Raven **317** (`:4099-4111`, minion, `timeLeft *= 5` → 18000, local-immunity 10) | `ai_raven` + `minion_separation` — dispatch `54 =>` (`:19985`) | base (S1-P-i25/i56) |
| 55 | `:28773-28844` | FlamingJack **321** (`:4139-4148`, 30×30 melee, tileCollide false, ignoreWater) | `ai_flaming_jack` — dispatch `55 =>` (`:17942`) | **d6** `a91c25381` |
| 56 | `:28845-28866` | FlamingScythe **329** (`:4215-4225`, 80×80 hostile pen −1, life 420, light 0.25) | `ai_flaming_scythe` — dispatch `56 =>` (`:17950`) | **d6** `b53ee70ab` + c6 dup (§3) |
| 57 | `:28867-28905` | NorthPole snowflake **343** (`:4326-4338`, pen 3, life 120, scale 1.1, cold); its spawn **344** (`:4339-4352`, aiStyle 1) rowed too | `projectile_ai_spear::ai_north_pole_flake` — dispatch `57 =>` (`:17686`), the 344 spawn drains via `spawn_with_data_ai_owned` | base (spear lane) |
| 58 | `:28906-28937` | Present **351** (`:4411-4420`, hostile, pen −1, tileCollide false) | `projectile_ai_frostmoon::present` — dispatch `58 =>` kind-gated SANTA_PRESENT (`:17998`) | base (S1-D-4) |
| 59 | `:28938-28993` | SpectreWrath **356** (`:4464-4473`, 6×6 magic, extraUpdates 3) | `ai_spectre_wrath` — dispatch `59 =>` (`:17976`) | base |
| 60 | `:28994-29309` | WaterGun **358** (`:4487-4496`), SlimeGun **406** (`:4868-4877`), ShimmerGunStream **1042** (`:9848-9857`) — all alpha 255, pen −1, extraUpdates 2, ignoreWater | `projectile_ai_guns::gun_stream` — dispatch `60 =>` (`:17962`) | **d6** `5d81f2de0` |

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

## 5. ShimmerImmunity × f47 — the COMPOSE contract (no dup)

The f47 tip `18fe75f2e` was re-read whole on the buff path:

* f47 CONSUMES `buff_immune[353]`: `npc_buffs.rs:700` (the buff-scan's
  immune branch — DelBuff mid-scan, `NPC.cs:92133-92143`) and
  `npc.rs:5082` (the shimmer-wet contact's guard). Its own tests SET the
  flag by hand (`:1356`/`:1389`).
* f47 NEVER WRITES the flag from the kind: `for_kind` on its tip has no
  `SHIMMER_IMMUNITY`/353 application (grep: zero hits for
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
adjustments (the 353 exclusion) stay; the f47 hand-set-flag tests stay
(they test the scan, not the writer). The 1042 gun contact
(`AddBuff(353,100)` through the immune guard, `projectile_ai_guns.rs`)
and the f47 `shimmer_wet_contact_pass` are disjoint sites — the gun
NEVER writes `NPC.shimmerWet` (the census answer NO stands, ANALYSIS
residual 9).

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
| retire | `388530671` + this doc's commits | the 601 row fix (§6), the g65 keep-one (§4), the f47 compose (§5), this seal |

**SEALED + band-51-60 declaration:** 10/10 styles armed, 16/16 kinds
rowed (+ the 344 rider), the boundaries closed both ends (§1), every
spawn source ported or owned (§2), the four keep-one/compose contracts
published (§3–5), the 601 row corrected and its teleport remainder
SEEDED (§6). Remaining owned elsewhere: the style-56 merge composition
(c6/d6 merger), the light-toggle dedupe (c42/e20 merger), the g65 D1–D3
drop (g65 lane/merger), the d18 601-witness re-point (merger), the
portal-gun subsystem (the §6 seed). **No census/arm/row work is owed on
51–60.**

**Witness register (WRITTEN NOT RUN, code-only):** the band's ~40
witnesses across `projectile.rs` (the 52/55/56 arms + the PVE-walk
ghost/vampire chain + the row pins), `projectile_ai_guns.rs` (8),
`projectile_gun_splash.rs` (4), `world_sim/editing.rs` (4 use-path),
`world_sim/projectiles.rs` (1 pass-chain), `armor_sets.rs` (1),
`player.rs` (2 budgets), `npc_buffs.rs` (1 set gate),
`render/player_layers.rs` (1 vanity), `projectile/data.rs` (the retire
row-shape witness). Verification: direct C#/Rust re-reads + mechanical
greps only; **no compiler invoked** — the merge gate owns the run.