# 60 — Item-use census: `Player.ItemCheck` vs the port's use path

Worker W3-i01 (task/S1-P-i01-item-use-census). Read-only census: no code changes,
no builds/tests run. All C# cites are `decompiled/Terraria/Player.cs` unless noted;
all Rust cites are `crates/tr-app/src/…` at `origin/main` aa147ce0.

## 0. Naming reconciliation (read first)

The task brief names `ItemCheck_ManageUseTime`, `ItemCheck_ManageShoot` and
`CheckItemText`. **None of these exist in this 1.4.5.8 decompilation** (grep over
`decompiled/` finds zero hits). The use-time/use-shoot logic lives inline in:

| Brief name | Real location |
| --- | --- |
| `ItemCheck_ManageUseTime` | `ItemCheck` inline countdowns (`:42656-42671`, `:42808-42845`) + `SetItemTime`/`ApplyItemTime`/`SetItemAnimation`/`ApplyItemAnimation` (`:4406-4497`) + `ApplyReuseDelay` (`:52835-52840`) + `ItemCheck_AutoReuseLogic` (`:52877-52903`) |
| `ItemCheck_ManageShoot` | `ItemCheck_OwnerOnlyCode`'s shot gate (`:43544-43571`) + `ItemCheck_Shoot` (`:47056-49770`) + `PickAmmo` (`:53458+`) |
| `CheckItemText` | closest match: `Main.MouseText`/`MouseTextInner` (`Main.cs:20138-20182+`), the held-item cursor tooltip. Port equivalent: `ui/tooltip.rs` (has use-time/knockback rows; tests at `:914-922`). Out of this census's sim scope beyond the row below. |

The port's whole use path is **`world_sim/editing.rs::update_editing`**
(`:284-2001`), driven once per fixed step from `app/frame/world.rs:496`. It is NOT
a port of `ItemCheck`'s structure — it is a cursor-driven action dispatcher with
seconds-float timers (`app.swing`, `app.item_time`, `app.attack_cd`,
`app.reuse_delay`; fields `app/mod.rs:225,284,905-926`) instead of the C#'s
integer `itemAnimation`/`itemTime`/`reuseDelay`/`attackCD`. The ammo-volley wire
(`:1702-1951`) reconstructs integer tick counts via `ticks_of` (`:199-201`) —
that part is deliberately faithful; everything else compares floats.

## 1. C# use-path skeleton (order is load-bearing)

`Player.Update` → `ItemCheckWrapped` (`:31798-31828`) → `ItemCheck` (`:42595`):

1. `pendingItemReuse = false`; **CCed early-return** (`:42597-42603`):
   `channel=false`, `itemAnimation=itemAnimationMax=0`, return. `CCed` =
   `frozen || webbed || stoned` (`:3377-3387`).
2. Local-player input helpers: fast-use, fishing-bait / solution-gun cursor icon
   (`:42608-42651`).
3. `ItemCheck_HandleMount` (`:42652`, body `:52922-52950`), `GetWeaponDamage`
   (`:42653`), **`ItemCheck_AutoReuseLogic`** (`:42654`), `HackHoldStyles`
   (`:42655`).
4. Negative-timer clamps; `itemAnimationMax=0` when anim 0; **`ApplyReuseDelay`**
   when `itemAnimation==0 && reuseDelay>0` (`:42656-42671`): copies `reuseDelay`
   into BOTH `itemAnimation` and `itemTime`, zeroes `reuseDelay` (`:52837-52839`).
5. Revolver crit-bonus decay (2269) — **draws `Main.rand.Next(3)` every tick for
   every player** (`:42677-42703`); gem-staff ArPenSpread mid-anim extra shot and
   snake-band whip extra shot (`:42704-42732`).
6. **Use-start gate** (`:42733`): `controlUseItem && releaseUseItem &&
   itemAnimation==0 && useStyle!=0 && !bufferedChange` →
   `ItemCheck_TryStartUse` (`:42743`, body `:52021-52040`) =
   `CheckCanUse_Inner` (`:52042-52321`) && `ActuallyPayMana` (`:52638-52655`) &&
   `KillDirtBlock` && `PullFishingBobbers`. On success: potion delay (`:42756`),
   mana-item buff grant (`:42760-42763`), pet buffs, mount (`:42768-42771`),
   `FreeUpPetsAndMinions` (`:42778-42781`), **`ItemCheck_StartActualUse`**
   (`:42784`, body `:51657-51699`).
7. `flag5` input re-read; kite/minecart overrides; **`!flag5 → channel=false`**
   (`:42787-42807`).
8. **Anim countdown** (`:42808-42823`): if `itemAnimation>0`: mana item →
   `ItemCheck_ApplyManaRegenDelay` **every anim tick** (`:42810-42813`),
   `itemAnimation--`; on reaching 0 with `reuseDelay==0 && controlUseItem &&
   releaseUseItem` → `pendingItemReuse=true`.
9. `releaseUseItem = !controlUseItem` (`:42829`); **`itemTime--`** (`:42834-42836`)
   + star-in-a-bottle max-mana effect at zero (`:42837-42844`).
10. `AnimatePlayerAndGetItemFrame` (`:42846`) → held light/food/drink particles,
    then **`ItemCheck_OwnerOnlyCode`** (`:42854`).
11. Mid-swing consumable effects at `ItemTimeIsZero && itemAnimation>0`
    (`:42856-42975`): hair dye, **`ApplyLifeAndOrMana` + `ApplyItemTime`** for
    `healLife>0 || healMana>0` (`:42867-42875`), buff items (`:42876-42883`),
    celebrationmk2 buff shower (678). Then recall/mirror/conch/shellphone/
    gender-swap arms keyed on `itemTime == useTime/2` or fixed ticks
    (`:42976-43250`), wand consumption + **generic consumable consumption with
    the ammo-cost rolls** (`:43264-43326`), stack-empty → `itemTime =
    itemAnimation` (`:43321-43324`).

`ItemCheck_OwnerOnlyCode` (`:43452-43716`): per-item first-shot gates
(`ItemAnimationJustStarted` for the flails/swords `:43460-43483`), turret/minion
alt-use (`:43542-43543`), **shot gate `flag4 = itemAnimation>0 && ItemTimeIsZero`**
(`:43544`) + razorpine channel top-up (`:43545-43559`) + `shootsEveryUse`
(`:43560-43563`) → **`ItemCheck_Shoot`** (`:43568-43571`); wiring/lawnmower/
instruments/buckets (`:43572-43575`); **`toolTime = itemTime`** (non-channel) or
channel countdown re-arm (`:43576-43587`); mining tools (`:43593`), teleport rod,
life/mana crystals, permanent power-ups, event items, boss spawners
(`:43594-43608`); `PlaceThing` (`:43613`); critter release (`:43621-43624`);
**melee hitbox block** (`:43644-43686`): every anim tick, `damage>=0 && !noMelee`
→ `GetMeleeHitbox` → visuals, tile cutting, `MeleeHitNPCs` with glove/buff/stealth
knockback (`:43665-43679`).

`ItemCheck_Shoot` (`:47056`): melee-speed velocity divide (`:47061-47064`);
`PickAmmo` for `useAmmo>0` (`:47069-47071`), gunProj stat reset (`:47077-47082`);
phaseblade/coin/per-item damage tweaks (`:47083-47146`); **`!canShoot` returns
BEFORE `ApplyItemTime`** (`:47147-47150`); `GetWeaponKnockback` (`:47151`,
body `:53346-53369`); **`ApplyItemTime(sItem)`** re-arm (`:47185-47195`); spawn at
`RotatedRelativePoint(MountedCenter)` (`:47196`); aim normalize → `speed`
(`:47277-47307`); **`ChangeDir` by aim dot** (`:47214-47224`); `itemRotation`
write for styles 5/13 (`:47349-47381`); dozens of per-item spawn arms; generic
`Projectile.NewProjectile` tail. No sound inside `ItemCheck_Shoot` itself (the
bow/gun cue is the item's `UseSound` at use start; `PickAmmo` plays nothing).

## 2. Census table

Status: FAITHFUL / WRONG / MISSING (no code anywhere) / UNWIRED (fields/logic
exist but the use path never reads them).

| # | Step | C# cite | Port cite | Status |
| --- | --- | --- | --- | --- |
| U1 | CCed early return incl. **stoned** | `:42598-42603`, `:3377-3387` | `editing.rs:356-363` gates `frozen || webbed` only; no `stoned` field (`player.rs:641-642` has just the two) | WRONG (stoned missing; also doesn't zero swing/channel) |
| U2 | Negative-timer clamps, `itemAnimationMax` reset | `:42656-42667` | floats clamp via `.max(0.0)` at decay (`editing.rs:289,323,1091`) | FAITHFUL (equivalent) |
| U3 | `ApplyReuseDelay`: reuseDelay → anim+time | `:42668-42671`, `:52835-52840` | `app.reuse_delay` decays as its own timer (`editing.rs:708,1090-1092`); gates use-start but never becomes swing/itemTime | WRONG (shape: reuse delay blocks but doesn't animate; `ItemCheck_Shoot`'s dry-fire windows can't see it) |
| U4 | autoReuse forced `releaseUseItem=true`; anim==1 → 0 early cut | `:52877-52903` | `throw_ready = auto_reuse \|\| press-edge` (`editing.rs:605-606`, `:1642`) | FAITHFUL-in-effect for weapons; the anim==1 early cut (saves 1 tick/swing) is absent → swings run 1 tick long (WRONG, minor) |
| U5 | FeralClaws/stressBall/autoReuseAllWeapons reuse grants | `TryAllowingItemReuse` `:52905-52920` | `accessory_combat.rs:18` documents autoReuseGlove unported; no stressBall/autoReuseAllWeapons | MISSING |
| U6 | Use-start edge `controlUseItem && releaseUseItem && anim==0` | `:42733` | weapon arms gate `attack_cd<=0 && reuse_delay<=0 && throw_ready` (`editing.rs:615,666,1128-1133`); place arm `starting \|\| inside_swing` (`:1641-1644`) | FAITHFUL-in-effect (different mechanism: `attack_cd` mirrors swing length, C# gates on anim itself) |
| U7 | `CheckCanUse_Inner` battery: noItems, potionDelay, silence, mount CanMount, HasAmmo, per-item caps (vilethorn 6, crystal storm 10, …), water/torch rules, pulley+kite, DD2 sentry spots | `:52042-52321` | port has only: silence (`editing.rs:682-684`), potion-sickness buff 21 (`:1374-1379`), mount `can_mount` (`:1410`), ammo pick refusal (`:1734-1742`); **no HasAmmo pre-gate** (C# refuses use-start when out of ammo `:53431-53446`; port starts the swing and dry-fires), no per-projectile caps, no `noItems` | MISSING (bulk); ammo pre-gate WRONG (dry swing starts + UseSound plays when C# refuses silently) |
| U8 | Mana **predict** (`CheckManaPredictWithoutUse`, quick-mana aware) then **pay** (`CheckMana`, manaFlower QuickMana auto-drink) | `:52549-52617`, `:52624-52655` | flat `p.mana < cost → return` + pay (`editing.rs:698-705`) | WRONG (no mana-flower QuickMana; `CheckMana`'s flower arm `:52588-52609` absent; QuickMana key exists in `input_map.rs:53` but has no consumer) |
| U9 | spaceGun skip-mana + skip-regen-delay | `:52695-52698`, `:50589-50592` | `armor_sets::space_gun_skips_mana` (`editing.rs:692`; `armor_sets.rs:393-397`) | FAITHFUL |
| U10 | Mana cost `(int)(mana * manaCost)`; alt-fire 3852 ×2; free-usage 2795/3006 | `:52579`, `:52679-52687`, `:52665-52677` | `editing.rs:696` (base ×cost) | FAITHFUL for the base arm; alt-fire/free-usage arms MISSING (no alt-fire use path anyway) |
| U11 | Mana regen delay **every anim tick** while mana item swings | `:42810-42813` → `:19396-19416` | applied ONCE at pay (`editing.rs:704` → `regen.rs:268-274`) | WRONG (regen resumes mid-swing in port; vanilla keeps pushing the delay until the swing ends) |
| U12 | `StartActualUse`: toolTime=1 for tools, grapple pulley release, StartChanneling, attackCD=0, ResetMeleeHitCooldowns, ApplyItemAnimation, UseSound | `:51657-51699` | `start_swing_extras` (`editing.rs:206-238`) does aim + UseSound only | PARTIAL: sound FAITHFUL (`item.rs:73-79` incl. SkipsInitialUseSound {2350,4870}; case-2 draws `sound.rs:267`, audio gate `audio.rs:1040-1080`); toolTime/pulley-release/ResetMeleeHitCooldowns MISSING; attackCD inverted (port sets `attack_cd = swing_max`, C# zeroes it and re-arms only after a melee hit `:44428-44431`) |
| U13 | `ApplyItemAnimation` multipliers: meleeSpeed, summoner/whip speed, tileSpeed, wallSpeed, style-5 volley `+1` tick | `:4463-4497` | `item::use_animation_secs_of` (`item.rs:41-50`) = raw prefixed ticks only; `p.melee_speed()` used for shot VELOCITY (`editing.rs:743`) but never swing length; `tile_speed`/`wall_speed` computed (`buff_effects.rs:76-77,158-159`) never consumed | UNWIRED (all five multiplier arms; melee-speed weapons swing too slow, mining/building buffs don't speed the swing) |
| U14 | `reuseDelay = sItem.reuseDelay` at every use start | `:4496` | `editing.rs:708` sets it — but only in the NON-ammo shoot arm; melee/food/potion/mount/place arms never set it | WRONG (partial coverage; registry `reuse_delay` exists, e.g. `registry.rs:519` sky fracture 18) |
| U15 | `ApplyItemTime` = useTime; multiplier variant clamps ≥1 | `:4406-4430` | place arm `app.item_time = action_secs` (`editing.rs:1997`); shooter re-arm `:1950`; food/potion/melee arms never set `item_time` | PARTIAL (see U16/U17) |
| U16 | Potion: heal at `ItemTimeIsZero && anim>0` (mid-swing), potionDelay via `ApplyPotionDelay` (3001 random 2400-4200, restoration 2700, pStone ×0.75), healMana + ManaSickness buff 94, HealEffect | `:42756-42759`, `:42867-42875`, `:51853-51886`, `:51888-51952`; `Item.cs:49968-49969`, `:25478-25484` | heal ON PRESS + flat buff 21 3600 (`editing.rs:540-565` first arm — **computes `sick` but never checks it**; second arm `:1372-1400` checks). No healMana potions, no buff 94, no restoration/pStone durations, no HealEffect text | WRONG (duplicate arm: the first drink ignores sickness entirely — potion-sickness is bypassable by holding the button over open sky; heal timing on-press vs mid-swing; healMana MISSING) |
| U17 | Food (buff 26/206/207) mid-swing consume + `ApplyItemTime` | `:42876-42883` + `CanUseItem` wellFed `:14353` | on-press consume (`editing.rs:506-533`) | WRONG (timing only: consumes/heals on press, C# at itemTime zero mid-anim; minor) |
| U18 | Generic consumable decrement with huntress/chloro/ammoCost rolls, ForceConsumption, stack-empty → itemTime=anim | `:43280-43326` | place arm decrements per placement (`use_tile_item` `:3425+`); shoot arm decrements per shot (`editing.rs:1067-1070`); thrown consumables consume per swing | WRONG (no ammo-cost-80/75/90 rolls on thrown ranged consumables — shuriken/throwing knives drain 1-per-throw always; `ammo.rs` not-consume rolls cover ammo weapons only) |
| U19 | Channel: set at StartChanneling (item.channel), cleared on `!controlUseItem` / CC / mouse-item / projectile death (`TryCancelChannel`) | `:52958-52968`, `:42804-42807`, `:5037-5039`, `:52975-52981` | `p.channel/channel_item/channel_target` maintained per-tick from held-button state (`editing.rs:578-600`), consumed by aiStyle-9 owner block | FAITHFUL-in-effect for magic missiles (aim clamp `limit_point_to_player_reachable_area` `:260-282` mirrors `:45447-45473`); MISSING: projectile-death cancel (`TryCancelChannel`), mouse-item cancel, razorpine top-up (`:43545-43559`) |
| U20 | Channel anim hold: C# has NO "hold animation" — the anim runs out; channel only gates `toolTime` re-arm (`:43576-43587`) and reuse grants (`:52912`) | as cited | port fabricates `app.swing = 0.2` + `attack_cd.max(0.2)` hold for non-weapon channel items (`editing.rs:1635-1640`), citing `:43576` which says no such thing | WRONG (invented mechanic + wrong cite; also `channels` requires `!held_weapon` so actual channel WEAPONS never take this arm — dead-ish code) |
| U21 | Shot gate `itemAnimation>0 && ItemTimeIsZero`, per-item first-tick gates, `shootsEveryUse` | `:43544-43571`, `:43460-43518` | ammo shooters: integer-tick wire `:1702-1717` (just_started/anim windows) — FAITHFUL; non-ammo shooters fire ONCE on press (`editing.rs:666-1078`) | WRONG for non-ammo multi-shot items (crystal storm/vilethorn etc. fire once per swing in port; C# re-fires at every itemTime zero); per-item `ItemAnimationJustStarted` weapon gates MISSING |
| U22 | `PickAmmo` chain: specific pairs, cycling (+cooldown anim+120), not-consume roll windows, stat fuse, gunProj reset | `:53458-53682`, `:47069-47082` | `ammo.rs` (`pick/resolve_projectile/not_consume/drain`) + `editing.rs:1719-1908` incl. VolleyWindows (`:1820-1829`) | FAITHFUL (the deepest-ported part of the path; witnesses in `ammo.rs:797-1435`) |
| U23 | Dry fire: `!canShoot` returns before `ApplyItemTime`, swing still plays | `:47147-47150` | `editing.rs:1734-1742` returns, item_time stays 0 | FAITHFUL (but see U7: C# wouldn't have started the swing at all without ammo) |
| U24 | Shot spawn point `RotatedRelativePoint(MountedCenter)` + per-item offsets | `:47196`, `:47225-47276` | `center + dir*10` (`editing.rs:750-754,1914`) | WRONG (documented stand-in? not in ANALYSIS as such — the +10px along-aim offset is invented; C# spawns at the rotated mounted centre, no +10; per-item offsets mostly MISSING) |
| U25 | `ChangeDir` toward aim on shoot (`useTurn` handled in movement `:19837`) | `:47214-47224` | no facing write anywhere in the shoot arms | MISSING (player keeps facing walk-direction while shooting backwards) |
| U26 | `itemRotation` write styles 5/13 at shoot | `:47349-47381` | per-tick `swing_aim` atan2 for style 5 (`editing.rs:292-305`); style 13 not handled | PARTIAL (style 5 equivalent; 13 rapier aim MISSING) |
| U27 | `GetWeaponKnockback`: minionKB, kbGlove ×2 melee, kbBuff ×1.5, shroomite/vortex stealth | `:53346-53369` | fields exist (`player.rs:1424-1425`, `buff_effects.rs:56`, `buff_effects_misc.rs:337`) but shoot/melee arms pass raw `fx_kb` (`editing.rs:1278`, `:1872-1907`); melee glove multiplier comment admits "port's 1f" (`:1275-1277`) | UNWIRED (all four arms) |
| U28 | Melee hitbox: EVERY anim tick vs rotated item rectangle, immune cooldowns, tile cutting, PVP | `:43644-43686`, `GetMeleeHitbox` `:45099`, `MeleeHitNPCs` `:44241-44420` | one strike per swing when cursor point is inside an NPC box within 4 tiles (`editing.rs:1128-1365`) | WRONG (fundamental shape: cursor-point hit vs swing-arc hitbox; no per-tick multi-hit, no `ResetMeleeHitCooldowns`/`SetMeleeHitCooldown` `:44414`, no tile cutting `:43680`, no PVP) — largest combat deviation |
| U29 | Melee `attackCD = max(1, animMax*0.33)` after each hit; per-NPC immune[i] | `:44428-44431`, `:44246` | `attack_cd = swing_max` after a hit (`editing.rs:1361`); strike gate has no NPC immunity check visible in the arm | WRONG (cooldown formula + semantics: C# attackCD gates the NEXT melee hit within a swing, port uses it as a whole-swing lock) |
| U30 | Mining: `toolTime==0 && anim>0 && controlUseItem` cadence; pick damage per hit; wall hammer fallback | `:45584-45644`, `:43576-43587` | continuous `mine_timer` seconds accumulator vs hardness (`editing.rs:1451-1614`), swing restarts freely when `swing<=0` (`:1477-1481`) | WRONG (documented stand-in cadence — dig speed folds pickSpeed but not the per-swing hit rhythm; `toolTime` doesn't exist) |
| U31 | Placement: `PlaceThing` inside OwnerOnlyCode at itemTime-zero cadence | `:43613`, `:39586+` | `starting || inside_swing` place arm (`editing.rs:1952-2000`), item_time re-arm `:1997` | FAITHFUL-in-effect (dirt 10-in-15 volley documented `:1616-1623`) |
| U32 | Recall/mirror/conch/shellphone teleports at `itemTime == useTime/2` (or 20/2) mid-swing | `:42976-43250` | none (grep: no recall/mirror use arm anywhere; 2350/4870 only appear as loot rows + the sound skip) | MISSING (recall potion, magic mirror, conches, shellphone all inert) |
| U33 | Life/Mana crystal + permanent power-ups (demon heart, life fruit, mana crystal, combat book, …) | `:43595-43608`, bodies `:45192-45424` | none in use path (`player.rs:4838` mana_max+20 is buff 29 MagicPower, not the crystal) | MISSING |
| U34 | Boss spawners/event items mid-swing (`ItemTimeIsZero && anim>0` + per-item gates) | `:43602-43603`, `:43751-44056` | on-press arms BEFORE the reach gate (`editing.rs:378-446`) | PARTIAL (items work; timing/gates simplified — e.g. C# consumes mid-swing, port on press; day/biome gates partially mirrored in `CheckCanUse` equivalents) |
| U35 | Revolver (2269) crit-bonus tick + **per-tick `Main.rand.Next(3)` draw for ALL players** | `:42677-42703` | absent | MISSING — also an RNG-stream alignment hazard: any future port must place this draw in the UpdatePlayers stream at the same position |
| U36 | Hair dye, star-in-bottle max-mana effect at itemTime zero, food/drink particles | `:42837-42866`, `:43717-43750` | food particles documented at `player.rs:4131`; hair dye / star bottle absent | PARTIAL/MISSING |
| U37 | `pendingItemReuse` + `UsingOrReusingItem` (mount/gfx consumers) | `:42597`, `:42819-42822`, `:4068-4078` | absent | MISSING (low impact until mounts/gfx need it) |
| U38 | Held-item tooltip use-time row (`Main.MouseText` family) | `Main.cs:20138-20182` | `ui/tooltip.rs` (rows + tests `:914-922`) | FAITHFUL (spot-checked row presence only) |
| U39 | Net sync of use attempt (msg 13), UseSound sync (152), rotation (41) | `:42744-42755`, `:51690-51692`, `:47373-47374` | documented no-port (`editing.rs:224-226`) | MISSING (acknowledged single-player fold) |

## 3. Top-10 findings (severity order)

1. **Potion-sickness bypass** — `editing.rs:540-565` computes `sick` and never
   reads it; the arm fires before the melee block, so any healing potion drunk
   with the cursor away from NPCs ignores buff 21 entirely (C# gate
   `:52217-52220`). Also heals on press instead of mid-swing, flat 3600 delay
   (no restoration 2700 / pStone / 3001 random), no healMana/ManaSickness(94).
   Two near-duplicate arms (`:540` vs `:1372`) should collapse into one.
2. **Melee is cursor-point, not swing-arc** — no `GetMeleeHitbox`/per-tick
   hitbox, no immune cooldowns, no tile cutting, no glove/buff knockback
   (U27-U29). Every broadsword behaves like a 4-tile-range click attack.
3. **`ApplyItemAnimation` multiplier arms all unwired** — meleeSpeed, tileSpeed,
   wallSpeed, whip/summoner speed, style-5 `+1` volley tick (U13). Reforge
   speed prefixes change `useTime`-paced actions but swing length only via the
   prefix table, never via player melee speed; mining-haste buffs do nothing.
4. **Non-ammo shooters fire once per press-swing** — crystal storm, vilethorn,
   razorpine, etc. lose the `ItemTimeIsZero` volley (U21); only `useAmmo>0`
   weapons got the faithful tick wire.
5. **Mana regen delay applied once, not per anim tick** (U11) + no mana-flower
   auto QuickMana (U8): magic sustain economy diverges.
6. **Recall/mirror/conch/shellphone and life/mana crystals are inert** (U32,
   U33) — whole item families with registry rows but no use arm.
7. **`reuseDelay` never becomes the animation** (U3) and is only armed on one of
   six use-start sites (U14) — sky-fracture-style pacing items mispace.
8. **Invented channel anim-hold** (`editing.rs:1635-1640`, U20) with a citation
   (`:43576`) that doesn't support it — and gated `!held_weapon` so real channel
   weapons bypass it; dead-wrong code that should be deleted, not fixed.
9. **Shoot spawn `center + dir*10` + no ChangeDir** (U24, U25) — invented spawn
   offset, projectiles emerge off-centre; player never turns toward the aim.
10. **HasAmmo use-start pre-gate missing** (U7) — out-of-ammo weapons still
    swing + play UseSound; C# refuses the use silently. Plus stoned missing
    from the CC gate (U1) and the per-tick revolver `Next(3)` stream draw (U35)
    as determinism follow-ups.

## 4. Proposed fix slices (for other workers)

Each slice is independently mergeable; none requires the melee-arc rewrite.

- **S1-P-1 potions** (fixes #1): delete the first potion arm
  (`editing.rs:540-565`), keep/extend the second; port `ApplyPotionDelay`
  (`:51853-51886`) + `ApplyLifeAndOrMana` (`:51888-51952`) incl. healMana +
  buff 94 + `HealEffect`; move the heal to the mid-swing `item_time<=0` seam.
  Tests: sickness-gated re-drink, restoration duration, mana-potion sickness.
- **S1-P-2 anim multipliers** (#3): route `use_animation_secs_of` through a
  player-aware `ApplyItemAnimation` port (`:4463-4497`); consume
  `melee_speed()`, `tile_speed`, `wall_speed`, whip multiplier; add the
  style-5 `num=1` tick. Touches `item.rs`, `editing.rs` call sites; witness
  the five C# arms with per-arm unit tests.
- **S1-P-3 magic economy** (#5): per-anim-tick `ApplyManaRegenDelay` while a
  mana item swings; `CheckMana`'s manaFlower QuickMana arm (`:52588-52609`)
  + `QuickMana_GetItemToUse`; wire the existing QuickMana keybind.
- **S1-P-4 non-ammo volley** (#4): generalize the `:1702-1951` tick wire to
  `shoot>0 && useAmmo==0` items; port the `:43460-43518` first-tick gates and
  `shootsEveryUse`; keep the per-item spawn arms already in `:787-1055`.
- **S1-P-5 reuse delay** (#7): one shared use-start tail that arms
  `reuse_delay` for every arm (U14) + `ApplyReuseDelay` semantics (delay runs
  as animation, blocks itemTime) — pairs naturally with S1-P-2.
- **S1-P-6 teleport items** (#6a): recall 2350 / return 4870 / mirror 50 /
  conches 4263/4819 / shellphone 5359 mid-swing arms (`:42976-43250`), reusing
  the mount/spawn seam; consumes at the teleport tick, not on press.
- **S1-P-7 crystals & power-ups** (#6b): `:45192-45424` family (life crystal,
  life fruit, mana crystal, demon heart, combat book, artisan loaf, torch
  god, peddler satchel, shimmer items) — mostly stat bumps + consume.
- **S1-P-8 shoot geometry** (#9): spawn at rotated mounted centre (drop the
  invented +10), `ChangeDir` aim-dot arm (`:47214-47224`), `GetWeaponKnockback`
  wiring (fields already exist). Determinism note: pixel diffs will move —
  needs screenshot re-pin in the same task.
- **S1-P-9 channel cleanup** (#8): delete `editing.rs:1635-1640`; port
  `TryCancelChannel`/mouse-item cancel; razorpine top-up rides S1-P-4.
- **S1-P-10 gates & stream** (#10): HasAmmo pre-gate at use start; `stoned`
  field + CC gate; revolver per-tick `Next(3)` draw placed in the
  UpdatePlayers stream (must land WITH a stream-order witness, cf. U35).
- **S1-P-11 melee arc** (#2, largest): `GetMeleeHitbox` (`:45099-45190`) +
  per-tick `MeleeHitNPCs` (`:44241-44420`) + immune cooldowns +
  `ApplyAttackCooldown` formula + tile cutting. Recommend splitting into
  hitbox-geometry / cooldown / cutting sub-slices; blocks nothing above.

## 5. Verification

Read-only census — no cargo/rustc/tests run (worker constraint). Evidence is
the cited line reads on both sides at aa147ce0; every WRONG/MISSING claim was
checked against a grep of the whole `crates/` tree for a consumer before being
recorded (e.g. `tile_speed` consumers: only `buff_effects.rs` writes; recall
2350: only loot tables + the sound-skip row).
