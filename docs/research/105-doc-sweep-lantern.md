# 105 — DOC-SWEEP lane 25: lantern / events (W4-f40)

Code-only doc-sweep of the lantern-night slice against the sibling lanes that
touch it (c39 gates, d17 aiStyle-166, d130 wind, f45 persistence). Every C#
claim was re-read from the decompiled windows directly (≤100-line reads,
citations inline); every port claim was read from this branch's tree or the
sibling's `git show`. **No cargo/rustc/tests/graphify invoked** — the witnesses
are WRITTEN NOT RUN. This note records the REAL gaps ported here + the union
contract for the merger; it does not change any sibling lane's code.

Branch `task/W4-f40-lantern-night-2`, base wave-3 `a725acb0`. The lane's prior
work (phases 1-3 + ANALYSIS) landed the luck feed, the shop wish row, the
`on_game_event_cleared(id)` helper + 16 boss-death wires, and
`render/lantern_sky.rs`. This sweep adds the USE-side launch/draw, the invasion
triggers, the persistence-order fix, and the wind-drift witness.

## Lane tips read

| Lane | Tip | Scope (this slice) |
| --- | --- | --- |
| W4-f40 (this) | `8b4f87e9` | lantern second half + this sweep |
| W4-c39 | `a493e8f15` (research/114 master reconcile) | umbrella: gates + scheduler completion + both skies + hats + ai166 + cake |
| W4-d17 | `84126544` | aiStyle-166 `ai_dove`/`dove_movement`/`dove_bounce` in `projectile.rs` + 851/855 rows |
| W4-d130 | `50c845e5` | weather: `Wind::lanterns_up` freeze + rain/snow core |
| W4-f45 | `42b7363b` | .wld header/flags: the lantern quad save/restore |

c39's `docs/research/114-lantern-party-master-reconcile.md` is the authoritative
per-function KEEP/DROP sheet for the c39/f40/d17/f38/d120/f41/d55/f34 union.
This note folds f40's DOC-SWEEP additions into that sheet and confirms the
sweep's ports match its prescriptions.

---

## 1. ReleaseLantern item 4702 — the USE arm

**Census correction first.** `ItemID.cs:11087` `ReleaseLantern = 4702`;
`ProjectileID.cs:2264` `ReleaseLantern = 855`. The sibling `ReleaseDoves` is
item **4682** (`ItemID.cs:11047`) → proj **851** (`ProjectileID.cs:2256`). The
task's "spawn 851/855" refers to this pair.

**Item data** (`Item.cs:38644-38651`): `width 20, height 24`,
`DefaultToThrownWeapon(855, 25, 4f)` (`:48455-48466`: useStyle 1,
useTime/useAnimation 25, shoot 855, shootSpeed 4, noMelee, ranged, consumable),
then `value = buyPrice(0,0,1)` (1 silver), **`ranged = false`**, **`noUseGraphic
= true`**. The doves 4682 are byte-identical in shape (`:38555-38562`, shoot
851). The `itemdata.rs:4868` row already carries these (projectile 855,
use_time/anim 25, shoot_speed 4, consumable, ranged=false, class None) — data
is not the gap.

**The C# USE path has NO per-item arm.** `ItemCheck_Shoot`
(`Player.cs:47056-49468`) carries no `sItem.type == 4702` / `projToShoot == 855`
branch — the release items ride the **generic tail** (`:49437`
`NewProjectile(source, pointPosition, num4, num5, projToShoot, Damage,
KnockBack, i, ai7, ai8, ai9)`):
- `pointPosition = RotatedRelativePoint(MountedCenter)` (`:47196`) — the player
  centre, **no `+dir*10` lead**. `NewProjectile` centres the box on it
  (`Projectile.cs:10876-10877`, `position = X − width/2`).
- The aim is normalized to `shootSpeed` (`:47277-47307`); the **zero-aim
  fallback** (`:47291-47296`) sets `num4 = direction, num5 = 0` and still fires.
- `ChangeDir` (`:47213-47223`): `Dot(UnitX, aim) > 0 → 1 else −1` (the zero aim
  takes −1). `flag` is true for the release items (only Void Vault 3611 skips).
- `GetWeaponDamage` (`:53421-53429`): `damage <= 0 → return damage` — the
  release items' `damage = -1` passes through **unscaled** (the class
  multiplier is 1 anyway, class None).
- The consume (`ItemCheck` `:43280-43326`): `ranged = false` **skips the four
  not-consume rolls** (huntress/chloro/ammoCost80/ammoCost75), `CanConsumeConsumableItem`
  is `return true` (`:5496-5499`), `ForceConsumption[4702]` is null
  (`ItemID.cs:413-427` — only 2350/4870/2351/2756/4343/4344), so **always
  consume**; the stack-empty latch `itemTime = itemAnimation` (`:43321-43324`).

**The gap on this branch.** The generic `fire_non_ammo_shot`
(`world_sim/editing.rs`) already launched + consumed 4702, but diverged on two
points: (a) the spawn carried the port's `+ dir * 10` lead (the M4-PROJ-3
deviation) instead of the exact centre, and (b) the zero-aim gate
`if dir == ZERO && !boomerang_use { return false }` **refused** the release
items (the C# fires them at the facing). The 851/855 `proj_data` rows are
**absent on this branch** (d17's claim, `data.rs:3925/3928` at d17's tip), so
`spawn_with_data_ai` no-ops — the launch decision + consume run, the projectile
does not yet exist.

**Ported** (`23a62996`): a `release_use = matches!(held.kind, 4682 | 4702)`
variant joins the boomerang's exact-centre arm — the zero-aim gate escape, the
no-lead spawn, the facing fallback + `ChangeDir` flip, the half-box subtract.
The consume tail already handled `ranged = false` (no roll chain) + the
stack-empty latch. 4 witnesses (`w4_f40_release_tests`): press arms the swing +
consumes, the last-release `itemTime = itemAnimation` latch, the zero-aim
facing fire, and the no-RNG consume (the four ammo-cost flags up spend zero
draws). The exact-centre spawn assertion rides d17's rows landing (the spawn
seam no-ops until then — recorded).

**Union with d17 (research/114 §1c):** d17's `ai_dove`/`dove_movement`/
`dove_bounce` + the 851/855 rows + `projFrames => 4` are the surviving machine
(the blocked-move `Next(3)` draw, the unconditional HandleMovement, the
direction write). c39's separate `projectile_ai166.rs` drops; its **lantern
entity-light feed** (`(1,0.6,0.1)*(1-alpha/255)` at the centre,
`Projectile.cs:47568-47569`, into `app/frame/draw.rs`) is the frame-side seam
the pool cannot reach — KEEP c39's feed grafted beside d17's machine. This
lane's USE arm is orthogonal to both (the launch decision, not the flight).

## 2. The "star-lantern variant (4703?)" — census correction

**4703 is NOT a lantern.** `ItemID.cs:11089` `QuadBarrelShotgun = 4703`;
`Item.cs:38652-38661` is `DefaultToRangedWeapon(14, AmmoID.Bullet, 55, 7f)`,
damage 14, knockback 6.5, UseSound Item36 — a shotgun. Its USE arm
(`Player.cs:47969-47982`: one straight shot + 7 spread pellets) is already
ported in `gun_launch.rs:218/330` (`quad_barrel`). **No gap, no port.**

**The real star lantern is `FallenStarLantern = 5613`** (`ItemID.cs:12909`):
`Item.cs:44142-44147` `DefaultToPlaceableTile(42, 50)` (tile 42 =
`HangingLanterns`, `TileID.cs:713`), `SetShopValues(White0, 150)`, 12×28. The
`itemdata.rs:5779` row already carries it (tile 42, place_style 50, value 150,
12×28, consumable, use_turn, auto_reuse). It is a **placeable furniture
lantern**, not a release/sky item — its placement rides the generic tile route.
**No gap.**

**The sky "variant" is the 3-across sheet column.** `LanternSky.ResetLantern`
(`LanternSky.cs:108`) draws `Variant = _random.Next(3)`; the source rect is
`FrameWidth * Variant` with `FrameWidth = texture.Width / 3` (`:48, :55-58`) —
three lantern sprites on `Extra_134`. `render/lantern_sky.rs` already ports this
(`reset_lantern`'s `next_range(0, 3)`, the variant-column source rect, the
`assets.rs:816/2409` `Extra_134` → `Lookups.lantern_sky` lookup). **No gap.**
There is no separate "gold/star" release-lantern item in 1.4.5.8 — the three
sky variants are the only lantern visual variants.

## 3. Lantern-night natural spawn gates — the c39 union contract

c39 (`f5ae8fc0` + the research/114 sheet) owns the gates; this lane does NOT
duplicate them. The union for the merger (research/114 §1f/§2):

| Gate | C# | c39 port | Verdict |
| --- | --- | --- | --- |
| `LanternsCanPersist` full gate | `LanternNight.cs:49-65` (`!dayTime && LanternsCanStart()`) | `weather.rs` `LanternGates::from_world` + live `lanterns_can_start` (boss/invasion mid-night drops GENUINE, manual persists) | **KEEP c39** — this branch keeps the base `!day_time`-only gate; the merged weather hunk takes c39's gate line |
| wind lantern freeze | `Main.cs:58856-58859` (`if LanternsUp return` before `windCounter--`) | `Wind::lantern_night` own flag (`tr-core/wind.rs`), fed in `weather.rs` | **KEEP c39's flag** (d130 has the identical `Wind::lanterns_up` — one survives, see §4) |
| rain stop + start-arm block | `Main.cs:64868-64871` (StopRain), `:64890` (`!slimeRain && !LanternsUp && !NextNightIsLanternNight`) | `Rain::update_time` 3 new gate args + the stop-rain arm | **KEEP c39** (unique; this branch's `rain.rs` has the base 5-arg signature) |
| storm-deck clamp | `Main.cs:64825-64831` (`cloudBGActive = 0`, numClouds cap 30) | `Rain::update_time` head `cloud_bg = 0` | **KEEP c39** (the numClouds cap is outcome-neutral at the 100 stand-in — recorded) |
| natural spawn (`NaturalAttempt`) | `LanternNight.cs:80-104` (cooldown decrement, post-MoonLord `Next(14)==0`, the `NextNightIsLanternNight` override, `Next(5,11)` cooldown) | base S1-F-14 `check_night_lanterns` (`world_events.rs:130`), wired in `clock.rs:156` | already on this branch (the base scheduler); c39 adds no new natural-spawn arm |

**The triggers — this lane's contribution.** research/114 §1a KEEP f40's
id-parameterised `on_game_event_cleared(ev, id, first)` (the structural 1:1
that survives moon-boss death arms) + c39's invasion-clear wires re-pointed at
it. This sweep ports exactly that: the four invasion clears
(`update_invasion_state`, `Main.cs:64335-64354` → ids goblins 0 / frost 1 /
pirates 2 / martians 3, all default-arm) now call the helper on the was-down
edge (`a5547cfc`), matching c39's `a17b4a7b` invasion half onto f40's helper
signature. 1 witness (all four types, first-vs-repeat).

**The case-4 label fix (research/114 §0.1).** Re-read `NPC.cs:79210-79237`:
Pumpking (type 327) → `SetEventFlagCleared(ref downedHalloweenKing, 5)` — id
**5**, the DEFAULT arm, so Pumpking's first kill **ARMS** the lantern night.
Mourning Wood (type 325) → id **4** (a skip). Ice Queen (345) → id 20 (default,
arms); Everscream (344) → 21, SantaNK1 (346) → 22 (skips). This branch's id
table (skip 4/21/22) was already correct; the "Pumpking 4" label in the helper
doc / witness / ANALYSIS / anchor was wrong — fixed (`8b4f87e9`), witness gains
the id-5/id-20 arm asserts.

## 4. Wind lantern drift — d130 `lanterns_up` + this lane's sky

**The sky consumes the live wind.** `LanternSky.Update` (`LanternSky.cs:126-135`)
reads `Main.windSpeedCurrent` per lantern: `num = windSpeedCurrent`, `num == 0 →
0.1` fallback; X drift `(0.1 + num) * (3 - speed) * 0.5 * (i/len + 1.5) / 2.5`;
rotation `num2 * sign(num) * 0.5`. `render/lantern_sky.rs:197-210` ports this
verbatim; the update seam (`app/frame/world.rs:523`) reads
`self.wind.speed_current` **live** (not a frozen snapshot). Witness
(`522d92bd`): three wind cases pinned (+0.4 drift 0.3, the −0.4 rotation sign,
the 0→0.1 calm fallback drift 0.12).

**d130's freeze does not stop the drift.** `Wind::update_rain`
(`tr-core/wind.rs`, d130 tip `:208-213`): the `lanterns_up` early-out sits
**AFTER** the `speed_current` ease-toward-target (`:185-200`) and **BEFORE** the
`windCounter--` target walk (`:216+`). So a lantern night freezes the target
walk but the current speed still eases — the sky's drift survives. This matches
the C# (`Main.cs:58856-58859`'s return is after the drift, before the walk).
**Verified: this lane's sky consumes d130's wind correctly.**

**c39 vs d130 wind-flag union.** Both add the same `Main.cs:58856-58859`
early-out: c39 `Wind::lantern_night` (`f5ae8fc0`), d130 `Wind::lanterns_up`
(`50c845e5`'s weather lane). They are behaviourally identical (own flag, after
the drift, before the walk); the merger keeps ONE (research/114 §1f names c39's
`lantern_night`; d130's is the same gate under a different name — a mechanical
rename at merge, not a semantic conflict). This lane's sky reads
`wind.speed_current`, agnostic to which flag name survives.

## 5. Event-cleared persistence — f45 lantern quad + the order fix

**f45 owns the save/restore.** `31996439` (Ph3) claims the lantern quad:
`SetTempToOngoing` (`WorldFile.cs:1060-1063`) snapshots
`LanternNightsOnCooldown/GenuineLanterns/ManualLanterns/NextNightIsLanternNight`
into `World.lantern_night_*` at save (`saveglue.rs:132-136`); the .wld flags
block writes them at tail 1940..1946 (`WorldFile.cs:1381-1384`) and reads them
back at version ≥ 207 (`:2326-2329`, below 207 the four temps reset);
`SetOngoingToTemps` (`:173-176`) restores them into `App.world_events` in
`App::new`'s `from_wld` block. The field names match this branch's
`WorldEvents` (`lantern_nights_on_cooldown/genuine_lanterns/manual_lanterns/
next_night_is_lantern_night`) — the restore lands cleanly.

**The REAL gap this sweep fixed: the clear-then-restore ORDER.** The C#
`LoadWorld` runs `WorldGen.clearWorld()` (`WorldFile.cs:1977`, which calls
`LanternNight.WorldClear`, `WorldGen.cs:6996`) **BEFORE** `LoadWorldFlags` reads
the temps and `SetOngoingToTemps` restores them. This branch's phase-3 put the
`world_clear` + `lantern_sky.reset()` in `enter_loaded_world` — which runs
**AFTER** `App::new` (the restore). Merged with f45, that tail clear would
**clobber the restored lantern quad on every load** (a saved lantern night dies
on reload). Fixed (`5a19073e`): the clear moves to `swap_world` (after
`load_world`, ahead of the `App::new` rebuild) and `client_install_world`
(ahead of the tile install, the `Netplay.cs:500` state-4 arm); the
`enter_loaded_world` tail carries no clear. The fresh `App::new` builds
`WorldEvents`/`LanternSky` from defaults anyway, so the single-player swap and
the net install start cleared by construction; the explicit clear pins the
order for any future restore-onto-self path. 1 witness
(`w4_f40_entering_a_loaded_world_keeps_the_restored_lantern_quad`): the quad
survives the tail + a source-order pin (swap clears before the rebuild, the
tail has no clear).

**The sky reset.** `LanternSky.Reset` (`LanternSky.cs:278-281`, `_active =
false`) rides the same clear sites; `App::new` builds a fresh inactive
`LanternSky::new()`. The merged App carries f40's LanternSky + c39's PartySky
half on ONE cadence stepper (research/114 §1b/§5); the `SkyManager.Reset`
family covers both — the party-sky reset is c39's follow-up, not this lane's.

**The `NextNightIsLanternNight` survival.** The flag is NOT in
`LanternNight.WorldClear` (`:119-125` clears Manual/Genuine/Cooldown/_was only)
— it survives a world clear by design (only `CheckNight`'s override consumes
it). The f45 quad restores it; the clear-then-restore order keeps it. The
trigger witness (`w4_f40_the_first_event_clear_arms_the_next_lantern_night`)
pins the persist-through-moon-boss behaviour.

---

## 6. Commits (this sweep)

| Hash | Mechanic |
| --- | --- |
| `5a19073e` | persistence: the lantern world-load clear moves to the clear-then-restore sites (item 5) |
| `23a62996` | Release Doves/Lantern USE arm: exact-centre + zero-aim + consume (item 1) |
| `5033e533` | noUseGraphic held-item draw gate, 391-kind census (item 1 visual companion) |
| `a5547cfc` | first invasion clear arms the lantern night, ids 0/1/2/3 (item 3) |
| `522d92bd` | LanternSky wind-drift witness (item 4) |
| `8b4f87e9` | case-4 label fix per c39 research/114 §0.1 (Mourning Wood, not Pumpking) |

Witnesses: 4 release + 1 noUseGraphic + 1 invasion + 1 wind-drift + 1
persistence + the extended trigger-table = 9 new/extended, all WRITTEN NOT RUN.

## 7. Verification

Code-only per the lane contract: **no cargo/rustc/tests/graphify invoked**.
Evidence: C# windows re-read directly — `Item.cs:38548-38679`/`:44130-44169`/
`:48455-48499`/`:48975-49004` + the `noUseGraphic` census (SetDefaults1-5 +
the six DefaultTo* helpers, 391 kinds, two independent extractors agree);
`Player.cs:47056-47307`/`:47930-48029`/`:49410-49468`/`:43240-43339`/
`:53340-53439`/`:5496-5540`; `Projectile.cs:8150-8229`/`:47540-47614`/
`:18320-18374`; `NPC.cs:78755-78804`/`:79205-79244`; `Main.cs:64330-64359`;
`LanternSky.cs:13-281`; `LanternNight.cs:1-153`; `WorldFile.cs:152-211`/
`:1040-1066`/`:1955-2014`/`:3430-3489`; `WorldGen.cs:6900-7009`;
`Chest.cs:2290-2334`; `ItemID.cs:413-432`/`:11047-11101`/`:12909`;
`TileID.cs:713`. Port claims read from this branch's tree + the siblings'
`git show` (c39 research/114, d17 `data.rs`/`projectile.rs`, d130 `wind.rs`,
f45 `app/mod.rs`/`saveglue.rs`/`wld`). The `noUseGraphic` array was
programmatically extracted + cross-checked (count 391, sorted, no dupes,
exact set-match against the C#). No working-tree code outside this lane was
modified.
