# 98 — DOC-SWEEP lane 20: shields & accessories (W4-f48)

**Branch:** `task/W4-f48-dodge-2` · **Mode:** code-only (no cargo/tests/graphify
runs; every witness WRITTEN NOT RUN, the parry-wave convention) · **Date:**
2026-11-21.

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
hellfireTreads / default dust-16 arms) is unported — it rides the
particle/dust waves, out of this lane's scope. No new code this lane.

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
hurt-spawned stars carry ai2 = 0).

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