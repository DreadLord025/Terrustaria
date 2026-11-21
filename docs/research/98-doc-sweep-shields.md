# 98 — DOC-SWEEP lane 20: shields & accessories (W4-f48) — SEALED

**Branch:** `task/W4-f48-dodge-2` · **Mode:** code-only (no cargo/tests/graphify
runs; every witness WRITTEN NOT RUN, the parry-wave convention) · **Date:**
2026-11-21 · **Sealed:** 2026-11-21 (SHIELDS RETIRE — the verdicts below).

The lane censused six items from the shields/accessories doc-sweep list and
ported the real gaps. This note records, per item: the C# source, the port
state BEFORE the lane, what landed (with the commit), and the census
negatives (things the sweep list named that do not exist in the C#, or were
already ported stand-ins).

## Verdict table

| # | Item | C# anchor | Before the lane | Landed |
|---|------|-----------|-----------------|--------|
| 1 | ShieldRaise keybind | `Player.cs:31706-31764` | input wiring already ported (`world_sim/player.rs:1689-1787`) | census-only (no `TriggersSet.ShieldRaise` trigger exists) |
| 2 | Shield draw | `PlayerDrawLayers.cs:2954-3024` | body row 10 read `shield_raised`; NO shield slot / draw layer | `7c121ccf7` — `Player.shield` + `shield_layers` + the 12 sheets |
| 3 | Held-4760 parry-reflect | `Projectile.cs:8312-8324` + AI/on-hit | raisable/shouldGuard gates ported; proj 866 INERT (no row) | `42ffb0759` — the 866 row + homing AI + on-hit spend |
| 4 | "Squire shield bash 866" | naming correction | 866 = Bouncing Shield, not a squire bash | folded into #3 (3823 = Brand of the Inferno) |
| 5 | Panic-necklace visuals | `Player.cs:10130-10133`/`:38237-38240` | buff-63 moveSpeed++ + grant already ported | census-only (no panic speed-lines exist in C#) |
| 6 | StarVeil star damage | `Player.cs:38432-38481` + `Projectile.cs:7646-7658` | spawn + row + AI already ported; NO damage-lands witness | `66d7ce51d` — the end-to-end witness |

Plus the raised-shield STAT effects (`41cc56f95`) and the projectile-lane
parry + ParryDamageBuff melee-projectile consumer (`08f55642e`), which the
census surfaced as the live gaps behind items 1/3.

## 1. ShieldRaise keybind — census negative

The sweep list named a "ShieldRaise keybind". There is **no
`TriggersSet.ShieldRaise` trigger** in the C# (`TriggersSet.cs:865-867` has
no such field). The input is `PlayerInput.Triggers.JustPressed.MouseRight`
(`Player.cs:31708`) folded into `theGeneralCheck` (`:31528`:
`selectedItem != 58 && controlUseTile && releaseUseItem && !controlUseItem
&& !mouseInterface`), and `shouldGuard` (`:31714-31718`) additionally
demands the held Brand of the Inferno (3823) / Bouncing Shield (4760), a
raisable shield, no mount, and (idle OR the right press).

**Already ported** at `world_sim/player.rs:1689-1787` (the parry wave,
commit `338a6e2d`): the `:28283` seam, the general-check stand-ins
(`place_down` / the previous tick's `mine_released` latch / `!mine_down` /
`!inventory_open`), the `drop_right_edge` press edge, the raise zeroing
swing/item_time/reuse_delay, the lower flooring `attack_cd` at 20/60, and
the MaxMana (legacy 25) + 10x dust-45 drains on `UpdatePlayers`. No new
code this lane — the item is a census confirmation, not a gap.

## 2. Shield draw — `7c121ccf7`

**C# source:** `DrawPlayer_25_Shield` (`PlayerDrawLayers.cs:2954-3024`),
slotted after `DrawPlayer_24_Pulley` / `32_FrontAcc_BackPart` and before
the BehindFrontArm held item (`LegacyPlayerRenderer.cs:228-238`). The slot
derivation is `UpdateVisibleAccessories` (`Player.cs:36753-36833`): the
equipped `shieldSlot` rows, the eocDash/raised fallbacks (`:36762-36777`),
the held-4760 override (`:36806-36810`, slot 9), reset by
`ResetVisibleAccessories` (`:31134`, from `Player.Update` `:25624` and
`UpdateDead` `:16937`).

**Before:** the port's body-frame chain already read `shield_raised` for
row 10 (`player.rs:8174`), but there was NO `Player.shield` slot field and
NO shield draw layer — the raised shield showed the pose but no shield
sprite.

**Landed:**
- `Player.shield: i16` (−1 = none) + `update_visible_shield` (the 11
  `shieldSlot` ids → `ArmorIDs.Shield` slots: 156→1, 938→2, 397→3, 1613→4,
  3097→5, 3810→6, 3997→7, 3998→8, 6183→10, 6177→11, 6188→12; the
  eocDash/raised fallbacks fill an unset slot ahead of the visibility
  gate; `ItemIsVisuallyIncompatible`'s shield arms — an enabled composite
  back arm or a held food (`sortdata::IS_FOOD`) — hide the worn row; the
  held 4760 with no live 866 overrides to slot 9). The per-tick reset
  rides the `ResetEffects` block; `update_dead` clears it too.
- `render/player_layers.rs::shield_layers`: the −4y raised lift
  (`gravDir` always 1), the 4 alpha-0 ghost copies orbiting at radius
  `2.5 + 1.5·sin(2π·GlobalTimeWrappedHourly)` quarter-turn apart with tint
  `×(0.45 − 0.15·sin)`, the body sprite, and the parry echo (push
  `×1.5·(parryTimeLeft/20)` away from the hitbox centre, scale
  `1 + 1.5·fraction`, RGB fade `×(1−fraction)`, alpha `×(1−fraction)²` —
  the C# `A *= f` THEN `color *= f` double multiply). The cart's last-two
  reverse (`:3020-3023`). The anchor collapses to the plain 40×56 body box
  (`height − bodyFrame.Height + 4 = −10` + the `bodyFrame/2` origin shift).
- `PlayerAssets.acc_shield: [Option<BgTex>; 13]` + the `Acc_Shield_1..12`
  atlas rows (`assets.rs` `STATIC_SHEET_NAMES`; the sheets exist under
  `assets/Content/Images/`, `AssetInitializer.cs:219-221`).
- The layer slots before the BehindFrontArm held item in `assemble_layers`.

**Census finding (dead code):** the `sin(π·t)` pulse overlay
(`:2995-3003`) builds a `DrawData` but **never `DrawDataCache.Add`s it** —
the block ends at `:3002` with no Add. The vanilla draws nothing there;
the port skips it faithfully (5 raised sprites, not 6).

**Deviations:** `bodyPosition`/`bodyRotation` are stock zero for the alive
player (`PlayerDrawSet.cs:1706-1711`); the `cShield` dye has no port
producer (every copy draws plain); the vanity slots 13..20 have no port
loadout (only the functional 3..10 scan).

## 3. Held-4760 parry-reflect / proj 866 — `42ffb0759`

**Naming correction (item 4):** the sweep's "Squire shield bash 866" is a
misnomer. **866 = `ProjectileID.BouncingShield`**, the shot of item 4760
(Bouncing Shield). **3823 = Brand of the Inferno**
(`DD2SquireDemonSword`), the OTHER raisable held item. There is no
"GreatShield" and no squire-bash projectile 866. The squire's actual
shield is item 3810 (`shieldSlot` 6); its "bash" is the contact-lane
parry reflect (already ported, `npc.rs:18246-18309`) plus the
`AddBuff(205, 300)` Hurt twin (`player.rs`, the parry wave).

**Before:** the raisable-shield gate (`player.rs:5108-5112`) and the
shouldGuard held-4760 arm (`world_sim/player.rs:1721-1726`) were ported,
and the boomerang NPC-hit reverse already excluded 866
(`projectile.rs:23027-23029`) and the wall-bounce trio already included it
(`:5522`). But **proj 866 had NO `proj_data` row**, so
`spawn_with_data_ai_owned` returned `None` — the thrown shield was inert.

**Landed:**
- The 866 row (`Projectile.cs:8312-8324`): 30×30, aiStyle 3, friendly,
  penetrate 5, melee, extraUpdates 1, light 0.15, `usesLocalNPCImmunity` +
  `localNPCHitCooldown = -1` (one hit per NPC per throw).
- `ai_boomerang`'s 866 arms: the outbound NEVER counts `ai[1]`
  (`:39027-39030`); the homing seek on `localAI[0]` ∈ [10, 360] through
  `FindTargetWithLineOfSight` (the slot's local-immunity slice is the
  `:45645` gate), else `ai[1] = -1`; the no-target 30-tick return (60 for
  a remote owner, `:39249-39259`); the 12/len accel-0.25 steer with the
  sign-crossing DOUBLE boost (`:39261-39299`); the 16/1.2 return seek
  (`:39427-39431`); rotation = velocity angle + the `Next(2)` dust-212
  gate (`:39560-39568`, the dust itself skipped, its gate draw kept).
- On-hit in `damage_pve`: the ×0.8 STORED-damage decay (`:13443-13445`,
  pre-strike, this hit keeps full damage — the Shadowbeam precedent);
  `immune[owner] = 0` + the local −1 stamp + `penetrate--`; at 0 the inert
  reset (`penetrate = 1`, `damage = 0`, `ai[1] = -1`, walk stops) else the
  local-owner retarget + velocity redirect keeping speed (`:14077-14104`).
  The generic post-strike `penetrate -= 1` now EXCLUDES 866 (`:14125`).
- `ai_boomerang` widened with `npcs` / `local_immunity` / `rng` (the other
  kinds draw nothing; the existing callers pass empty slices + a default
  stream).

## 4. (folded into 3 — the naming correction)

See item 3. No separate code.

## 5. Panic-necklace visuals — census negative

The sweep list named "panic-necklace visuals" (speed-lines). **No panic
speed-lines exist in the C#.** The Panic Necklace family (1290/1578/6187)
grants buff 63 on an accepted hit (`Player.cs:38237-38240`,
`AddBuff(63, 480)` post-dodge) and buff 63's only effect is `moveSpeed++`
(`UpdateBuffs` `:10130-10133`). Both were **already ported**
(`player.rs:4514-4531` the grant, the buff-scan `63 => move_speed += 1.0`
arm; the `panic` field + the three equip arms + the `ResetEffects` clear,
commit `5dfd6ae1`).

The fast-run DUST (`SpawnFastRunParticles`, `Player.cs:20205-20299`) is a
separate visual seam: the port's `spawn_fast_run_particles`
(`player.rs:9040`) carries only the SOUND half (the hermes step cue); the
dust half (the wings-3 / sailDash / desertDash / coldDash / fairyBoots /
hellfireTreads / default dust-16 arms) is unported ON THIS TREE — it rides
the particle/dust waves, out of this lane's scope. No new code this lane.
**RETIRE CORRECTION (2026-11-21):** the dust half is NOT unported
globally — W4-d51 `0d0cf7c42` (batch 3) landed the full `FastRunArm`
picker + the boot-run visual grants + the `player_dust::drain_fast_run`
replay; the body is byte-identical at d51/f3/g66 tips (d51 is an ancestor
of both f3 75f3f7347 and g66 248f0ff4d; not on main). KEEP ONE: the d51
copy rides whichever of f3/g66 merges; f48's sound-only body drops at
merge. See ANALYSIS "SHIELDS RETIRE" item 5.

## 6. StarVeil star damage — `66d7ce51d`

**Before:** the five cloak stars (723/724/725/726/1133) already had the
shared `proj_data` row (`Projectile.cs:7646-7658`: 24×24, aiStyle 5,
penetrate 2, alpha 50, local immunity 10), the `ai_cloak_star` AI arm, and
the retaliate-drain spawn through `spawn_with_data_ai_owned` (damage 75,
×2 expert, arpen 25; `hurt_modifiers.rs:153-214`). The star's `friendly`
defaults true (`projectile.rs:787`) and `damage_pve` gates only
`damage > 0`, so the star DOES damage — but no witness pinned the
end-to-end "the star hurts an NPC".

**Landed:** `w4_f48_the_star_veil_star_lands_damage` — the drain spawns
kind 725 (the starVeil override), the star's contact lands through
`damage_pve` and the NPC's life drops; the expert spawn carries 150. One
witness, written not run.

**Out of scope (noted):** the ManaCloak MAGIC-HIT star proc
(`Projectile.cs:13831-13852`: a magic projectile hit with the manaCloak
override + `starCloakCooldown == 0` spawns ONE 723 with `ai2 = 1`, damage
100, cooldown 300) is unported — it is a projectile-on-hit accessory proc,
not the hurt-retaliate lane this item covers. The 723's `ai2 = 1`
fall-through half is already coded inert in `ai_cloak_star` (the
hurt-spawned stars carry ai2 = 0). **RETIRE (2026-11-21): TASK-SEEDED**
to the projectile on-hit accessory lane (the `Damage_PVE_Inner` flag10
family — ghostHeal/ghostHurt/vampireHeal `:13812-13825` + the nebula roll
`:13826-13830` + this proc; no origin/task branch carries any of them,
censused). Prereqs: the `starCloakCooldown` field (`Player.cs:1108` + the
`:24626-24628` decrement, unported) + the accessory-source spawn seam.
ANALYSIS "SHIELDS RETIRE" item 4.

## The two live gaps the census surfaced

### Raised-shield stat effects — `41cc56f95`

`Player.cs:25714-25717` (`shieldRaised → statDefense += 20`, after the
equip pass, before the defense floor) and `:25955-25962`
(`strongestMoveSpeedDebuff = min(1/3)` + grounded `|vx| > 3 → vx /= 2`).
Ported at the defense-floor site (`player.rs:7170`) and beside the
slow/chilled debuff arms (`:6983-6988`), before the run-speed merge
(`:7256-7259`). The sibling `slowOgreSpit`/`dazed`/`burned` 1/3-and-0.5
caps (`:25931-25946`) are unported rows — only the shield arm lands.
3 witnesses written not run.

### Projectile-lane parry + ParryDamageBuff consumer — `08f55642e`

- **`Damage_EVP`'s parry** (`Projectile.cs:14822-14828`):
  `flag2 = CanParryAgainst(player.Hitbox, base.Hitbox, base.velocity)`.
  On a parry the whole hurt block (`:14828-14880`) is skipped — no
  DamageVar draw, no Hurt, no StatusPlayer — and the C# does NOT kill the
  shot (`:14881-14899` spends penetrate only for 435/682/436/437, never a
  parry term), so the parried shot keeps flying. **No reflect / immunity /
  buff is granted here** — those live ONLY in the contact lane's
  `Update_NPCCollision` (`Player.cs:31366-31431`, already ported). The
  port's `damage_player` returns early on a parry (the shot survives).
- **The ParryDamageBuff ×5 melee-PROJECTILE consumer**
  (`Projectile.cs:13573-13578`): an owned melee projectile ×5s the rolled
  damage while buff 198's flag is live, then clears the flag AND
  `ClearBuff(198)`. The `DontApplyParryDamageBuff` set
  (`ProjectileID.cs:187`) = {524, 321, 181, 566} refuses the boost. The
  port folds it post-DamageVar pre-strike (the draw order untouched — a
  pure multiply). This is the projectile twin of the already-ported
  melee-ITEM consumer (`world_sim/editing.rs:7734-7753`,
  `Player.cs:44307-44312`).
2 witnesses written not run.

## Commit ledger (this lane)

| Commit | Scope |
|--------|-------|
| `41cc56f95` | raised-shield stat arms (+20 def, 1/3 speed cap, grounded vx halve) |
| `7c121ccf7` | shield draw layer + `Player.shield` derivation + the 12 sheets |
| `42ffb0759` | proj 866 Bouncing Shield: row, homing AI, on-hit spend |
| `08f55642e` | projectile-lane parry + ParryDamageBuff melee-projectile consumer |
| `66d7ce51d` | StarVeil damage-lands witness |
| (this) | the research note + PORT_MAP/ANALYSIS/anchors updates |

Prior parry-wave commits on the branch: `64ca5fad` / `56746bae` /
`5dfd6ae1` / `338a6e2d` / `69d499c6`.

## Verification

Code-only mode: **no compiler invoked, no tests run, no graphify**. Every
witness above is WRITTEN NOT RUN (the parry-wave convention). The anchors
rows are added per landed feature; the canary (`crates/tr-core/build.rs` +
`main_anchors`) will check the markers on the next build.

---

## SHIELDS RETIRE (2026-11-21) — the seal

The lane's five port commits + the parry wave are the lane's whole scope;
this retire pass executes the cross-branch keep-one rulings, publishes
the receipts, task-seeds the two out-of-scope seams and seals this note.
Branch tip at retire start: `e5b66bb06`. Code-only: no cargo/tests/
rustfmt/graphify; the retargeted witnesses are WRITTEN NOT RUN.

### Receipt 1 — the f50 Panic-63 THIRD-copy drop: EXECUTED (`f4539c27e`)

f50's retire (51eefdf84, `W4-f50-closeout.md` "Sub-receipt — the f48
THIRD copy") ruled f50-canonical for `hurt_inner`'s reaction head and
published f48's drop-hunks at 69d499c6b. This pass re-pinned every range
at the lane-20 tip e5b66bb06 (the shield commits shifted the file) and
EXECUTED the drop in `crates/tr-app/src/player.rs`:

| f50 pin (69d499c6b) | e5b66bb06 re-pin | Action |
|---|---|---|
| grants :4514-4531 (comment + both ifs) | :4528-4545 | DROP both `AddBuff(63,480)` + `AddBuff(205,300)`; a keep-one comment cites f50's head |
| field :1454-1462 | :1463-1471 | DROP the `panic` field + doc (a retire note replaces it) |
| spawn :2808 | :2818 | DROP `panic: false` |
| equip :5649-5664 panic halves | :5661-5678 | 1578 keeps `honey_comb`, 6187 keeps `long_invince`, the 1290 arm drops whole |
| clear :6391-6392 | :6411-6412 | DROP the `:18616` clear |
| inline 63 row :6809-6814 | :6829-6834 | DROP `63 => self.move_speed += 1.0` (f12-canonical, Receipt 2) |

STAY (f50's receipt, verified): the dodge block (:4497-4527 — f48's own
lane, already re-verified on main), the DD2 crystal-lane
`add_buff(205, 3600)` (:12554 post-drop — the S1-D-4b-2 set-flags
witness, a DIFFERENT site from the hurt twin). Witness retargets: the
dodged-hit pin → `w4_f48_a_dodged_hit_draws_only_the_belt_roll` (the
grant assertions ride f50's head — same post-dodge site, composes 1:1);
the necklaces witness → `w4_f48_the_necklace_fold_halves_survive_the_
panic_keep_one`; the shield-cap witness's Panic-63 `2×1/3` pin →
Swiftness-3 `1.25×1/3` (the multiply-before-merge proof survives without
the dropped reader); the 480-tick / 205-twin / expiry witnesses retire
onto f50's `w4_f50_the_panic_necklace_grants_the_hurt_buff_and_arms_from_
all_three_sources` (:16118 at 18eeb4f44) +
`w4_f50_the_squire_t2_set_grants_ballista_panic_on_hurt` (:16154) +
f12's `w4_f12_closeout_panic_63_grants_move_speed_once_through_the_
wiring` (`buff_frame_wiring.rs:804` at 22ce2a41f).

**CENSUS NEGATIVE — the THIRD `rederive_beetle_solar_buffs` claim is
misattributed.** f50's receipt head names "f48-69d499c6b THIRD
rederive_beetle_solar_buffs+63-row copy"; the sub-receipt's own hunk list
carries NO rederive hunk, and the census confirms why: `git grep rederive
69d499c6b -- crates` → empty; the same at HEAD → only the unrelated wld
ore-tier tests + the biomes band test; `git log -S
rederive_beetle_solar_buffs` over the branch → zero commits. **f48 never
carried a beetle/solar ladder copy** — the panic family + the 63 row are
the whole third copy. The g27-canonical ladder ruling (f12/f50's ledgers)
is untouched; there is no third ladder to drop on this branch.

### Receipt 2 — the f12 Panic-63 canonical: PUBLISHED (keep-one f12)

f12's retire dispatch (22ce2a41f, `W4-f12-closeout.md` RETIRE) + f50's
re-pin (51eefdf84) census THREE inline-63 copies: f12's
`DEBUFF_DISPATCH` (the set at `buff_frame_wiring.rs:113` carries 63; the
`buff_effects.rs:149` table row `63 => e.move_speed += 1` — the canonical
wiring), f50's inline row (:7168 at 18eeb4f44 — f50's own drop-hunk
:7165-7168, NOT landed here), f48's inline row (:6834 at e5b66bb06 —
DROPPED this pass, Receipt 1). KEEP ONE: **f12**. Merged with either
inline row surviving, buff 63 grants +2 moveSpeed/tick (f12's ruling).
The 205 hurt grant: f12's tree has no writer (its sweep), so after the
f50/f48 fold exactly ONE `AddBuff(205, 300)` hurt grant survives (f50's
:4904-4906); f48's DD2 `add_buff(205, 3600)` is a different site and
lands clean.

### Receipt 3 — the 866 d18/d12 boomerang keep-one: COMPOSES (no cut)

The task named d18-8248a54b / d12-c98bd893 boomerang-arms compose. The
census at the FINAL tips (d18 6e8083751 — 8248a54be's band census is its
ancestor; d12 c98bd8937):

* the `ai_boomerang` fn body is **byte-identical** at d18 tip, d12 tip
  and f48's pre-866 base 42ffb0759^ (SHA256 head `2553EB9D1F0D` over the
  extracted body, all three) — neither lane touched the machine;
* their 866 mentions are the pre-existing wall-bounce trio 33/106/866
  (`Projectile.cs:19550-19560`) + the 301/866/902 NPC-hit-reverse
  exclusion — both already on main, both kept verbatim by f48's commit;
* NO 866 `proj_data` row exists on d18/d12 (`projectile/data.rs` grep:
  zero) — the row + the homing arms + the on-hit spend are f48's alone
  (42ffb0759).

**Ruling: f48-canonical for the 866 family; the widened `ai_boomerang`
signature composes onto either tip.** f48's widening added
`npcs`/`local_immunity`/`rng` (the 866 seek + the dust-212 gate draw; the
other kinds draw nothing) and updated all 13 call sites (1 live dispatch
`projectile.rs:17084` + 12 witnesses). At merge the widened signature
wins; d18/d12's call-site lines take the three new arguments (empty
slices + the pass stream). No keep-one deletion is needed on any branch
— the compose is mechanical, and the non-866 kinds' behaviour is
unchanged (the identity census above proves the base bodies match).

### Receipt 4 — the ManaCloak magic-hit proc: TASK SEED

The lane-20 out-of-scope note (item 6 above) is promoted to a seed:
`Projectile.cs:13831-13852`, the `Damage_PVE_Inner` flag10 block
(`:13810`) — a magic hit with the manaCloak override +
`starCloakCooldown == 0` sets the cooldown 300 and spawns ONE 723
(x + Next(-400,400), y − Next(500,800), the centre-seeking 23/len speed
with the Next(-100,101) x-jitter, damage 100, kb 5, ai2 = 1 — the
fall-through half `ai_cloak_star` already codes inert). **Lane: the
projectile on-hit accessory-proc family** (ghostHeal/ghostHurt/
vampireHeal `:13812-13825` + the nebula-boost roll `:13826-13830` + this
proc) — censused: no origin/task branch carries any of them. Prereqs:
the `starCloakCooldown` field (`Player.cs:1108`, the `:24626-24628`
decrement — unported) + the accessory-source spawn seam
(`spawn_with_data_ai_owned`, the StarVeil precedent). NOT the
hurt-retaliate lane (research/04's seal covers that family).

### Receipt 5 — SpawnFastRunParticles dust half: TASK SEED → d51 keep-one

The lane-20 seam note (item 5 above) called the dust half
(`Player.cs:20217-20298`) unported. The retire census finds it **LANDED
on W4-d51** (`0d0cf7c42`, batch 3: the `FastRunArm` picker — wings-3/
rocket-6 two-puff, sailDash 4×253, desertDash NewDustDirect 32, coldDash
2×76, fairyBoots NextFromList one-roll, hellfireTreads 6, plain 16 — +
`UpdateBootVisualEffects`/`CancelAllBootRunVisualEffects` + the
`player_dust::drain_fast_run` replay after the footstep sound). The
`spawn_fast_run_particles` body is byte-identical (SHA256 head
`653BEED04AEC`) at d51 tip 0d0cf7c42, f3 tip 75f3f7347 and g66 tip
248f0ff4d; d51 is an ancestor of both f3 and g66 (`merge-base
--is-ancestor` exit 0), NOT on main. **KEEP ONE: the d51 copy**, riding
whichever of f3/g66 merges — g66's RNG-order contract already names
"fast-run (W4-d51)" first in the `UpdatePlayers` drain order. f48's
sound-only body drops at merge in favour of the d51 shape; no f48 code
change this pass. The d51 port's documented deviation: the gravDir == −1
foot-offset fold (`:20207-20211`) collapses to 0 (no gravity flip).

### The shields-family bible (C# → port → owner → ruling)

| Machine | C# anchor | Port | Owner | Retire ruling |
|---|---|---|---|---|
| ShieldRaise input | `Player.cs:31706-31738` + the `:31528` general check | `world_sim/player.rs` (`shield_raise_countdowns` wiring, 338a6e2d) | f48 | LANDED, no dup (no `TriggersSet.ShieldRaise` exists) |
| TryTogglingShield / CanParryAgainst / GiveImmuneTimeForCollisionAttack | `:31741-31764` / `:31437-31444` / `:20494-20511` | `player.rs` (the parry machine, 64ca5fad/56746bae) | f48 | LANDED, no dup |
| Contact-lane parry reflect | `Update_NPCCollision` `:31366-31431` | `npc.rs` (the verdict before thorns, 2x/kb5, 60/30 + AddBuff(198,300)) | f48 | LANDED, no dup |
| Projectile-lane parry | `Damage_EVP` `Projectile.cs:14822-14828` | `projectile.rs::damage_player` (the hurt block skipped, the shot survives) | f48 | LANDED (08f55642e), no dup |
| ParryDamageBuff ×5 melee item / projectile | `Player.cs:44307-44312` / `Projectile.cs:13573-13578` | `world_sim/editing.rs` / `projectile.rs::damage_pve` (post-DamageVar pre-strike, flag + ClearBuff(198)) | f48 | LANDED, no dup |
| Raised-shield stats | `Player.cs:25714-25717` / `:25955-25962` | `player.rs` (+20 def, min(1/3) cap, grounded vx halve; 41cc56f95) | f48 | LANDED; the cap witness retargeted (Swiftness-3) |
| Shield draw | `PlayerDrawLayers.cs:2954-3024` + `Player.cs:36762-36810` | `render/player_layers.rs::shield_layers` + `Player.shield` (7c121ccf7) | f48 | LANDED; the pulse overlay is dead code (no cache Add) |
| Proj 866 Bouncing Shield | `Projectile.cs:8312-8324` + AI_003's 866 arms + on-hit `:13443-13445`/`:14077-14104`/`:14125` | `projectile/data.rs` row + `ai_boomerang` widened + `damage_pve` spend (42ffb0759) | f48 | LANDED; d18/d12 compose (Receipt 3 — byte-identical base bodies) |
| Cloak stars 723-726/1133 | `Projectile.cs:7646-7658` + `:23845-23953` + `Player.cs:38432-38481` | the shared row + `ai_cloak_star` + the retaliate drain | f48 (S1-C-7 wave B base) | LANDED; the StarVeil damage witness 66d7ce51d |
| Panic family (field/grant/arms/clear) | `Player.cs:1154`/`:38237-38240`/`:14892-14895`/`:15076-15080`/`:14625-14628`/`:18616` | f50's `hurt_inner` head | **f50** | f48's copies DROPPED (f4539c27e, Receipt 1) |
| Squire Hurt twin AddBuff(205,300) | `Player.cs:38241-38244` | f50's head (:4904-4906 at 18eeb4f44) | **f50** | f48's copy DROPPED; the DD2 3600 grant STAYS (different site) |
| Buff-63 moveSpeed++ reader | `UpdateBuffs` `Player.cs:10130-10133` | f12's DEBUFF_DISPATCH (`buff_frame_wiring.rs`/`buff_effects.rs`) | **f12** | f48's inline row DROPPED; f50's row is f50's own drop-hunk |
| Beetle/solar ladder rederive | `Player.cs:10342-10443` | g27 `apply_orb_arm` | **g27** | f48 carries NO copy (census negative, Receipt 1) |
| ManaCloak magic-hit proc | `Projectile.cs:13831-13852` | — | unclaimed | TASK SEED → the projectile on-hit accessory lane (Receipt 4) |
| Fast-run dust half | `Player.cs:20217-20298` | d51 `FastRunArm` + `player_dust::drain_fast_run` | **d51** (rides f3/g66) | KEEP ONE d51; f48's sound-only body drops at merge (Receipt 5) |
| Fast-run footstep sound | `Player.cs:20205-20216` | `player.rs::spawn_fast_run_particles` (S1-E-21 wave D) | main | already on main; the d51 body keeps it verbatim |

### Seal verdicts

* **SEALED**: items 1-6 + the two live gaps — every landed machine has
  its commit, its witnesses (written not run) and its anchors row; the
  cross-branch keep-one rulings are executed (f48's halves) or published
  (f50's/f12's own halves ride their ledgers).
* **OPEN (task-seeded, NOT this lane)**: the ManaCloak magic-hit proc
  (Receipt 4), the fast-run dust merge keep-one (Receipt 5 — a merger
  instruction, no code gap), the Seraph Necklace 6180 halves, the
  slowOgreSpit/dazed/burned speed caps, the cShield dye producer, the
  vanity shield loadout.
* **The ONE-paragraph SHIELDS merger-instruction**: at merge take f50's
  `hurt_inner` reaction head (the panic + squire grants + the field +
  the equip arms + the clear — f48's copies are already dropped at
  f4539c27e), f12's DEBUFF_DISPATCH 63 wiring (f50's inline row drops
  per its own ledger), f48's dodge block + the 1578/6187 fold halves +
  the DD2 3600 grant + the whole shields family (the parry machine, the
  stat arms, the draw layer, the 866 row + the widened `ai_boomerang` —
  d18/d12's call sites take the three new arguments mechanically), and
  the d51 fast-run body (via f3 or g66, whichever merges); then re-run
  the f12 sweep + the anchors canary. No beetle/solar ladder exists on
  f48 — the g27 canonical needs no third drop.

### Retire commit ledger

| Commit | Scope |
|--------|-------|
| `f4539c27e` | the f50 receipt's drop-hunks executed + the witness retargets (player.rs, +66/−147) |
| (this) | the receipts + the bible + the seal (research/98, ANALYSIS, PORT_MAP, ROADMAP, INDEX, anchors) |