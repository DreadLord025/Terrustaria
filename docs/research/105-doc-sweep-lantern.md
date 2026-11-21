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
| W4-f40 (this) | `3d7a9b77` (FINAL phase) | lantern second half + this sweep + the FINAL phase |
| W4-c39 | `8ecb288a` (research/114 §7 EXECUTED) | umbrella: gates + scheduler completion + both skies + hats + ai166 + cake |
| W4-d17 | `84126544` | aiStyle-166 `ai_dove`/`dove_movement`/`dove_bounce` in `projectile.rs` + 851/855 rows |
| W4-d130 | `50c845e5` | weather: `Wind::lanterns_up` freeze + rain/snow core |
| W4-f45 | `6352d384` | .wld header/flags: the lantern quad save/restore + Ph5-13 claims |

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

**FINAL census correction (§8.3): the DOVES 4682 DO have a per-item arm.**
`Player.cs:48383-48392`: `if (sItem.type == 4682)` fires THREE projectiles,
each `SpeedX = num4 + Next(-20, 21) * 0.1`, `SpeedY = num5 + Next(-20, 21) *
0.1` (six `Main.rand` draws per use, X before Y per shot), then `return` —
it never reaches the generic tail. The 4702 half of the "no per-item arm"
claim stands (the whole-file `4702` grep hits only ItemID/Item/Chest — no
Player arm); the 4682 half was missed at sweep time. The volley is ported
(`3d7a9b77`) into the same exact-centre release arm.

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

---

## 8. FINAL phase — the five closing items + the 114 cross-receipt

The FINAL phase re-read the sibling tips (c39 `8ecb288a` §7 EXECUTED, f45
`6352d384`) and closed the four remaining census questions. All witnesses
WRITTEN NOT RUN (code-only, no cargo/rustc/tests/rustfmt/graphify invoked).

### 8.1 c39 lantern-exec status (item 1) — verified + the seed graft taken

c39's research/114 §7 is EXECUTED on `task/W4-c39-lantern-birthday` (7 verdict
commits `b5ab90db`/`4d184569`/`912f0ae2`/`a493e8f1`/`bb19d79f`/`7aff2857`/
`ce288dac`, landing record `8ecb288a`). For THIS lane:

- **LanternSky-KEEP-f40 confirmed**: §1b/§2 name `render/lantern_sky.rs`
  (this module) the surviving LanternSky — the depth-band draw, the
  byte-truncation tint ladder, the unscaled view convention, the world-load
  reset all KEEP here.
- **c39 does NOT retire its lantern copy**: verdict 2 (`4d184569`) fixes the
  colour bug in-place (the tier² rgb drop + the premultiplied glow) and
  states "The lantern STATE fns stay interim (f40's module is the surviving
  LanternSky at the merge)" — `event_skies.rs`'s `generate_lanterns`/
  `reset_lantern`/`update_lanterns`/`draw_lanterns` still exist at `8ecb288a`
  (verified by `git show`). The retirement happens AT THE MERGE when this
  module folds wholesale, not on c39's branch. Cross-note: the merge drops
  c39's lantern half + dedupes `Extra_134` (§1b's assets note).
- **PartySky-c39-unique + the depth-layering retrofit noted**: §1e verdict
  YES-unique; the §1e follow-up-1 retrofit LANDED in `4d184569` —
  `draw_balloons` → `draw_balloons_band` (the `PartySky.cs:185-198` num/num2
  index walk, unscaled screenPosition + integer-halved centre + zoom-scaled
  push + `>=` cull), threaded through `draw_sky`'s depth boundaries +
  `AmbiencePass`/`draw_front_trees` (this module's shape) + a strata
  witness. Follow-up-2 (the world-load reset seam extended to the party
  sky) still rides this lane's reset at the merge.
- **The §1b world-seed graft TAKEN here** (`73eb9060`): `LanternSky::new`
  now takes `world_seed` and seeds the private stream
  `world_seed ^ 0x1A2B_3C4D` (c39's `event_skies.rs:105` form — the sheet's
  preferred per-world variety, still deterministic). Both `App::new` sites
  feed it (`mod.rs` `world_seed`, `flow.rs` `world.seed`); the `Default`
  impl dropped with the ctor arg; 1 twin-seed witness (same-seed identical,
  cross-seed divergent, the xor-constant probe); the ANALYSIS fold
  re-pinned. The sheet's "RE-DERIVE one twin-seed assert if the seed swap
  is taken" obligation is discharged.

### 8.2 Lantern-night MUSIC (item 2) — AMBIENT-ONLY, no track, no port

Census (whole reads): `MusicID.cs` carries 105 tracks, **no Lantern entry**
(the closest event tracks are SlimeRain 48 / PumpkinMoon 30 / FrostMoon 32).
`Main.UpdateAudio_DecideOnNewMusic` (`:12175-12990`, the whole ladder) has
**zero** `LanternNight`/`LanternsUp` branches — the town arm `flag23`
(`:12199`) is `TownNPCCount >= 3 && !ZoneShadowCandle`, NOT the lanterns.
The `LanternNightActive` field (`Main.cs:216`) is fed only into
`SetBackColor` (`:56718`/`:61980`), which never reads it — write-only, dead
(the ANALYSIS fold already recorded it). `LanternNight.cs` whole carries no
sound/ambience call. **Verdict: a lantern night plays the ordinary night
track 3 (or town 47 with 3+ town NPCs) — nothing to port.** The census doc
rides `decide_new_music` + 1 negative witness (`2399a2ab`: night 3, town
47, `MAX_MUSIC = 105`).

### 8.3 Lantern CRAFT recipe (item 3) — 4702 shop-only; 5613 at the bench

Census: `Recipe.cs` whole-file grep for `4702`/`4682`/`ReleaseLantern`/
`ReleaseDoves` — **zero hits**. The release pair is shop-only
(`Chest.cs:2307`/`:2311`, the Party Girl wish rows — 4702 gated on
`LanternsUp`, this lane's `StockWhen::LanternNight` arm). **No workbench
recipe exists in the C#, so none is ported** — the generator cannot invent
one (`gen_recipes.py` splices only the C# bodies). The craftable lantern is
**5613 FallenStarLantern**: `AddFallenStarFurniture` (`Recipe.cs:16415-16418`)
— 6 Fallen Star (5622) + 1 Torch (8) at `SetCraftingStation(18)` — tile 18
is the **Work Bench** (`TileID.cs:665`), NOT by hand. The generator row
exists: `recipes_gen.rs` row 1548 (`:11021-11027`) carries the exact tuple
(the `AddFallenStarFurniture` body is spliced literal — the parametric
`AddStandardFurnitureSetRecipes` helper's `lantern` parameter `:16597-16600`
is a DIFFERENT family, the per-wood lanterns, not 5613). 1 witness
(`0d1b10f2`): the release pair recipe-less + the 5613 row's exact
ingredients/station + craft-at-bench/refuse-without.

### 8.4 Lantern PERSIST quad (item 4) — the f45 compose-contract verified

The order fix (`5a19073e`) verified in-tree: `swap_world` clears at
`mod.rs:2710-2711` BEFORE the `*self = App::new(` rebuild;
`client_install_world` clears at `netglue.rs:2017-2018` BEFORE
`self.world = world` (the tile install); `enter_loaded_world` carries no
clear. f45's shape re-read at tip `6352d384`:

| f45 piece | Site | C# |
| --- | --- | --- |
| save snapshot | `saveglue.rs:132-136` → `World.lantern_night_{cooldown,genuine,manual,next_night_is_genuine}` | `SetTempToOngoing`, `WorldFile.cs:1060-1063` |
| flags write | `flags.rs:211-214` (tail 1940..1946) | `:1381-1384` |
| flags read | `flags.rs:670-673` (version ≥ 207) | `:2326-2329` |
| restore | `App::new` `from_wld` block, `mod.rs:4123-4127` | `SetOngoingToTemps`, `:173-176` |

The four World fields (`lantern_night_cooldown: i32` + three bools) map 1:1
onto this branch's `WorldEvents` names/types
(`lantern_nights_on_cooldown`/`genuine_lanterns`/`manual_lanterns`/
`next_night_is_lantern_night`) — the restore lands cleanly, and the
clear-then-restore order keeps it. 1 compose-contract witness (`3e2277d7`):
the typed mirror of f45's assignments (compile-breaks on a rename/retype
here, before the merge silently breaks the restore) + the quad survives
the tail + the net-install clear-before-install source-order pin. The
existing `w4_f40_entering_a_loaded_world_keeps_the_restored_lantern_quad`
stays (the swap-order half).

### 8.5 The doves-volley census correction (FINAL find)

Re-reading `Player.cs:48360-48419` for the music/recipe census surfaced the
missed 4682 arm (`:48383-48392`): three projectiles, each
`aim + Next(-20, 21) * 0.1` per axis, then `return`. The sweep's §1 "no
per-item arm for them" was half-wrong — 4702's generic-tail half stands,
4682's volley was missed. Ported (`3d7a9b77`) into the release arm (the
same exact-centre spawn + facing flip; the six draws ride the shared
`loot_rng` UpdatePlayers borrow — the C# `Main.rand` stream); 1 witness
(the doves' six-draw probe + the lantern's zero-draw contrast); the USE-arm
comment + module doc + anchor row re-pinned. The §1 text above carries the
correction note.

### 8.6 FINAL commits + witness ledger

| Hash | Item | Mechanic |
| --- | --- | --- |
| `73eb9060` | 1 | the world-seed RNG graft (`LanternSky::new(world_seed)`, the xor form) + twin-seed witness + ANALYSIS re-pin |
| `2399a2ab` | 2 | the music census doc + negative witness (ambient-only) |
| `0d1b10f2` | 3 | the craft census witness (4702 recipe-less, 5613 at the bench) |
| `3e2277d7` | 4 | the f45 compose-contract witness (typed mirror + net-install order pin) |
| `3d7a9b77` | — | the doves-volley census correction + port + witness + anchor re-pin |
| (this) | 5 | research/105 FINAL + the 114 cross-receipt |

Witnesses this phase: 1 twin-seed + 1 music-negative + 1 craft + 1
compose-contract + 1 doves-volley = 5 new, all WRITTEN NOT RUN. The lane
total: 9 sweep + 5 FINAL = 14 new/extended.

### 8.7 The 114 cross-receipt (f40 → c39's sheet)

What c39's research/114 asks of f40, and where it stands on this branch:

| 114 obligation | Status here |
| --- | --- |
| §1a KEEP f40's id-table helper + fix the case-4 label | DONE pre-FINAL (`8b4f87e9`, the Mourning Wood label + the id-5/id-20 arm asserts) |
| §1a f40's EoC-integration witness survives the union | in-tree (`w4_f40_the_first_boss_kill_arms_the_next_lantern_night`) |
| §1b KEEP f40's lantern_sky module wholesale at the merge | the module is canonical + complete (depth bands, tint ladder, reset); c39's copy stays interim on ITS branch until the merge |
| §1b the world-seed graft (preferred) + the twin-seed re-derive | TAKEN (`73eb9060`) |
| §1b f40's 3 lantern-sky witnesses KEEP | in-tree (ladder/determinism, opacity/leaving, surface-band reset) + the wind-drift witness (`522d92bd`) |
| §1d KEEP f40's shop wish row + luck write | in-tree (the `StockWhen::LanternNight` row + the beside-zones luck snapshot); c39 retired its duplicate (`ce288dac`) |
| §1e follow-up-2: the party-sky reset rides f40's reset seam | f40's `reset()` + the clear sites are ready; the party half folds at the merge (c39's `SkyManager.Reset` extension) |
| §5 merge hot-spots: f40's weather hunk takes c39's gate line | noted — this branch keeps the base `!day_time` gate (`weather.rs:207`); the merge takes c39's `LanternGates::from_world` (the §3 table above) |
| §4 anchor survival: f40's 3 rows verified present | `fn on_game_event_cleared` ✓, `struct LanternSky` ✓, `shop.rs\|LanternNight,` ✓ (+ this lane's 4 DOC-SWEEP rows, all markers present) |

The union contract is unchanged by the FINAL phase except the two census
corrections (the doves' volley arm — this lane's own §1, no sibling
overlap; the music/recipe negatives — no port anywhere, so no merge
conflict). The remaining merge scope on c39's side (§7's tail) is
unchanged: this module + the shop row + the reset fold wholesale, the
world-seed graft is now IN this module so the merge takes it as-is.

### 8.8 FINAL verification

Code-only per the lane contract: **no cargo/rustc/tests/rustfmt/graphify
invoked**. Evidence: `MusicID.cs:1-229` whole; `Main.cs:12140-13019` (the
whole `UpdateAudio_DecideOnNewMusic` + `UpdateWindyDayState` head),
`:56690-56749`/`:61940-62019` (the two `SetBackColor` feeds),
`:63452-63551` (the `SetBackColor` body — no `LanternNightActive` reader),
`:65195-65249`; the whole-tree `LanternNightActive`/`ReleaseLantern`/
`4702`/`4682` greps (Main.cs 3 hits / ItemID+ProjectileID+Item+Chest+Player
only); `Recipe.cs:16363-16456` (`AddFallenStarFurniture` whole + the
parametric helper's call sites), `:16540-16639`
(`AddStandardFurnitureSetRecipes` whole), `:6398-6427` (the SetupRecipes
furniture call order); `TileID.cs:665`; `Player.cs:48360-48419` (the doves
arm) + `:49425-49464` (the generic tail); `LanternNight.cs:1-153` whole
(re-read — no sound call). Sibling claims read via `git show` at c39
`8ecb288a` (research/114 whole + `event_skies.rs` markers) and f45
`6352d384` (`saveglue.rs`/`flags.rs`/`app/mod.rs` lantern sites). Port
claims read from this branch's tree (`lantern_sky.rs`, `audio.rs`,
`craft.rs`/`recipes_gen.rs`, `frame.rs`, `editing.rs`, `world_events.rs`,
`mod.rs`/`flow.rs`/`netglue.rs`). No working-tree code outside this lane
was modified; every commit touches only this lane's files.

---

## 9. RETIRE phase — the keep-one publishes + the merge receipts

The retire phase re-fetched every sibling tip (d17 `f6e5d7d3`, c39
`4f72847e`, f41 `f21028f56` — all advanced past the FINAL phase's reads)
and executed the four retire items. Code-only throughout: **no
cargo/rustc/tests/rustfmt/graphify invoked**; every witness WRITTEN NOT
RUN. One commit per item.

### 9.1 The doves-volley duplicate — ADJUDICATED (d17 `8a9e035e` vs f40 `3d7a9b77`)

**Byte-compare: NOT identical — divergent ports of the same C# arm**
(`Player.cs:48383-48392`, re-read whole: `for (num131 = 0; num131 < 3;
num131++)` → `NewProjectile(SpeedX: num4 + Next(-20,21)*0.1f, SpeedY:
num5 + Next(-20,21)*0.1f, X: pointPosition.X, Y: pointPosition.Y, …)` →
`return`). Both agree on the volley core: 3 shots, X draw before Y draw
per shot, `Next(-20,21)*0.1` per axis, six `UpdatePlayers` draws, one
consumable pay, the 4702 generic-tail single shot. The differences:

| Axis | d17 `8a9e035e` | f40 `3d7a9b77` | C# truth |
| --- | --- | --- | --- |
| Arm shape | standalone `else if held.kind == 4682` in the per-item chain (the `:48383` position, after the Bat Scepter `:48344` arm) | merged into the `boomerang_use \|\| release_use` exact-centre arm with a `volley` count | the standalone arm; `return` before the tail |
| Spawn point | `spawn` = centre `+dir*10` (the M4-PROJ-3 generic deviation), **no half-box subtract** | exact centre (`release_use` drops the lead) `− bw/2, − bh/2` | `pointPosition` (no lead) + `NewProjectile`'s `X − width/2` centre (`Projectile.cs:10876-10877`) |
| Zero-aim | d17's gate refuses 4682 at zero aim (`!boomerang_use` only) | `release_use` escapes the gate; the facing fallback fires | `:47291-47296` fires at the facing — f40 correct |
| Facing flip | none for the doves (the boomerang-only arm) | `ChangeDir` runs (`release_use`) | `:47214-47223` runs for every kind but 3611 — f40 correct |
| Witness | count/band/consume (`thrown_app`, rows live on d17) | six-draw stream probe + lantern zero-draw contrast (`release_app`, rows absent here) | complementary — union keeps both |
| Defect | none | the FINAL splice left `w4_f40_the_release_consume_spends_no_rng_draw` **unclosed** (the new test nested inside it; the file's brace census read 2046/2045) | — |

**Verdict: KEEP d17's arm FORM (the standalone `else if` — it matches the
C#'s per-item-chain structure and rides d17's live 851 rows), GRAFT f40's
three corrections onto it** — the exact-centre spawn (`pcx − bw/2, pcy −
bh/2`, the i08 precedent), the zero-aim escape + facing flip (the shared
`release_use` prologue stays on this branch), and the six-draw witness.
The f40 merged-arm variant (`volley` count inside the boomerang arm) is
RETIRED; the boomerang arm returns to its i08 single-shot form. The
lantern 4702's exact-centre geometry moves to a `release_spawn` override
on the generic tail (`held.kind == 4702` → centre − half-box, other
kinds keep `leaf_spawn` untouched) — the tail IS its C# arm (`:49437`),
so the correction belongs there, not in the boomerang arm.

Landed here: the d17-form doves arm (with the graft) in `editing.rs`'s
per-item chain; the generic tail's `release_spawn` override; the
brace-defect fix (the no-RNG witness closed with its stream assert
restored); d17's `w4_d17_the_doves_release_three_and_the_lantern_one`
witness re-derived VERBATIM beside the bone-javelin rig as the keep-one
publish (UNION-SCOPED — its spawn-count asserts go green when d17's
851/855 rows fold; this branch's spawn seam no-ops, the f40 draw-count +
consume witnesses pin the branch-local behaviour); the module doc +
anchor row re-pinned. **Merge instruction: take THIS branch's doves arm
+ the d17 witness copy here; DROP d17's original arm body's spawn-point
half (replace with the grafted exact-centre form landed here) and its
original witness copy (the duplicate); d17's zero-aim gate needs f40's
`release_use` escape (this branch's gate line is the keeper).** The
4682/4702 flight machine (`ai_dove`/`dove_movement`/`dove_bounce`) +
the 851/855 rows stay d17's wholesale (c39 `bb19d79f` already retired
its duplicate onto them — no third copy).

### 9.2 The lantern WISH shop row — f41's eleven-row block dedupe-check

f41's final sweep 2 (`be4cb841`) landed the Party Girl's full `case 13`
rework: the eleven-row `BirthdayParty` gift block (`Chest.cs:2345-2358`,
`StockWhen::PartyUp`), the day/night Bubble/Fog pair fix (`:2285-2294`),
and — the overlap — **its own Release Lantern 4702 `LanternsUp` row**
(`shop.rs:539`, `row(4702, StockWhen::LanternsUp)`).

**Dedupe-check result: f40's row is NOT one of f41's eleven.** The
eleven are 3749/3746/3739/3740/3741/3737/3738/3736/3745/3744/3743 — all
`PartyUp`-gated gift rows at the shop TAIL (`:2345-2358`). The 4702 row
is a separate `LanternsUp`-gated row mid-list (`:2309-2312`, after 4681,
ahead of the always 4791 tail). Both branches ported it independently:

| Piece | f40 (this branch) | f41 `be4cb841` | Verdict |
| --- | --- | --- | --- |
| row position | after 4681, before 4791 (`shop.rs:524`) | identical (`shop.rs:539`) | same — either survives |
| gate variant | `StockWhen::LanternNight` | `StockWhen::LanternsUp` | KEEP f41's name (matches the C# property; f40's variant renames mechanically — the anchor row `shop.rs\|LanternNight,` re-points) |
| `ShopFlags.lanterns_up` + default | present (`shop.rs:30`/`:84`) | present (`:42`/`:94`) | ONE field (identical name/type) |
| gate eval | `LanternNight => flags.lanterns_up` (`:1282`) | `LanternsUp => flags.lanterns_up` (`:1341`) | one arm after the rename |
| live feed | `npcs.rs:922` `lanterns_up(&app.world_events)` | `npcs.rs:988` identical expression | ONE feed line (byte-identical modulo comment) |
| witnesses | gate-off/gate-on + num-order (`npc.rs:24168`) | row-order + prices inside its 11-row witness (`shop.rs:1834-1861`) | complementary — union keeps both (f41's tail-order witness covers 4702's position too) |

**Merge instruction: KEEP f41's shop.rs `case 13` wholesale (it is the
superset — the day/night fix + the eleven gift rows + the 4702 row);
f40's row/gate-variant/feed contributions fold as the mechanical
`LanternNight → LanternsUp` rename; keep ONE `lanterns_up` field + ONE
feed line; both witness sets survive (f40's gate witness re-pins its
variant name).** The Bubble/Fog day-gate fix and the eleven-row block
are f41-unique — f40 does not duplicate them (its `case 13` keeps the
old `Always`+`Night` pair; the merge takes f41's).

### 9.3 The 114 merge-scope FINAL receipts (c39 `4f72847e`)

c39's tip advanced past the FINAL phase's `8ecb288a` read: the final
phase landed §8 (the v18 9-way union input, `5cdf6da6`), §9 (the
event-flag census, `96b13e3b`), §10 (the wind/rain compose-verify,
`9ac39893`) and §11 (the MERGE-SCOPE FINAL checklist + the 105/70
cross-receipts, `4f72847e`). Its §11 cross-receipt of THIS note
(`a06b16e4b`) confirms the four KEEP-c39 gate rows, the wind-drift /
clear-order / case-4-label / invasion-id agreements — **no open
conflict**. The per-item receipts against this branch's FINAL tip:

| 114 §11 item | f40 receipt |
| --- | --- |
| 1 LanternSky module KEEP-f40 | **canonical here** — `render/lantern_sky.rs` complete (depth bands, the `:224` tint ladder, the reset, the world-seed graft `73eb9060`). c39's `event_skies.rs` lantern STATE fns verified STILL INTERIM at `4f72847e` (`activate_lanterns`/`deactivate_lanterns`/`generate_lanterns`/`reset_lantern`/`update_lanterns`/`draw_lanterns` all present, the module doc names this module the survivor). The merge folds this module wholesale + drops c39's lantern half + dedupes `Extra_134`. |
| 2 world-load reset order | **canonical here** — the clear at `swap_world` (`mod.rs:2710-2711`) / `client_install_world` (`netglue.rs:2017-2018`), NOT `enter_loaded_world` (`5a19073e`). This branch's `world_clear` (`world_events.rs:336-349`) leaves `next_night_is_lantern_night` alone — matches `LanternNight.WorldClear` + c39's `world_clear` body. The PartySky reset (§1e follow-up-2) extends this seam at the merge. |
| 6 TRWLD v18 union | **f40 claims NO v18 tail bytes.** This branch's `save.rs` stays `VERSION = 17` (verified); the lantern quad persists through f45's `.wld` flags-block sites (`flags.rs:211-214`/`:670-673`, the v≥207 arm — the quad is a `.wld`-native block, NOT a TRWLD tail field; §8.4's compose-contract witness pins it). The v18 union's 106-byte head (§8's table) carries no f40 row — the c43≡f45 identity block is f45's claim, keep-ONE there. f40's merge obligation: NONE on save.rs; the quad's restore rides f45's `from_wld` block. |
| 7 Empress death arm (d125) | not f40's code — noted: the merge routes d125's arm through this branch's `on_game_event_cleared(ev, 23, first)` (id 23 default-arm, ARMS). The helper is canonical here (c39 `b5ab90db` re-derived it in-place). |
| 9 d130 weather compose | f40's sky is flag-name-agnostic (§4's wind-drift verify — reads `speed_current` live). c39's §10a split-return shape makes the `lantern_night`/`lanterns_up` rename mechanical; the storm-deck clamp + full `LanternsCanPersist` graft are c39→d130 obligations, NOT f40 code. This branch keeps the base `!day_time` gate (`weather.rs:207`) — the merge takes c39's gate line (§3's table). |
| 10 anchors | f40's 7 rows verified present on this tip (3 phase rows + 4 DOC-SWEEP rows; the release-arm row re-pinned by §9.1). c39's 7 rows (`7aff2857`) name no f40-owned marker — no cross-branch anchor collision. |

**The your-module-canonical receipt:** c39 §7's remaining-scope list
("f40's `render/lantern_sky.rs` module wholesale + the world-seed RNG
graft + the shop wish row + the world-load reset") is discharged on this
branch: the module is complete + canonical, the graft is IN it
(`73eb9060`), the shop row lands per §9.2's keep-f41-superset verdict
(f40's gate/feed contributions fold), the reset order is landed +
witnessed. Nothing in c39's final phase changes f40's scope.

### 9.4 The LANTERN-FAMILY declaration (the unified merger order)

The family spans four lanes + the umbrella: **f40** (sky + use/luck —
this branch), **c39** (gates + triggers + the umbrella exec), **f41**
(party — the shop superset + the scheduler consumers), **d17** (the
aiStyle-166 flight + rows). The unified merger order, folding research/
105 §8 (this note) + c39 research/114 §7/§11 (its tip `4f72847e`):

1. **d17's projectile machine first** (no lantern dependency): the
   851/855 `proj_data` rows + `ai_dove`/`dove_movement`/`dove_bounce` +
   `proj_frames` 4-row entries + c39's entity-light graft (`draw.rs`,
   `bb19d79f`'s KEEP). The doves' spawn seam goes live here.
2. **f40's use-side second** (§9.1's keep-one): this branch's doves arm
   (the grafted d17 form) + the generic tail's `release_spawn` override
   + the zero-aim/facing prologue + BOTH witness sets (d17's copy here
   is the keeper). DROP d17's original arm spawn-point half + its
   original witness copy.
3. **c39's gates + triggers third**: the weather hunk takes c39's
   `LanternGates::from_world` line + the storm-deck clamp + the rain
   gate args (d130's landed order, §10b); the wind flag keeps ONE name
   (mechanical rename, §10a); the trigger call sites keep ONE set (f40's
   helper + ids — c39 re-derived the same signature, `b5ab90db`); the
   five moon-boss death arms + World flags + the v18 tail ride c39's
   §8 union layout (the 9-way 106-byte head, VERSION 18).
4. **f40's sky module fourth**: `render/lantern_sky.rs` wholesale (the
   seed graft included); DROP c39's `event_skies.rs` lantern half +
   dedupe `Extra_134`; the PartySky depth-band retrofit (`4d184569`)
   stays c39's; the world-load reset seam extends to the party sky
   (§1e follow-up-2) on f40's clear sites.
5. **f41's shop superset fifth**: `case 13` wholesale (the day/night
   pair + the 11 gift rows + the 4702 row); f40's `LanternNight` variant
   renames to `LanternsUp`; ONE `lanterns_up` field + feed line; both
   witness sets survive (§9.2).
6. **f45's persistence last** (the quad's `.wld` sites + the identity
   block keep-ONE with c43): f40's clear-then-restore order is already
   the contract (§8.4's typed-mirror witness compile-breaks on drift).

Determinism notes for the merger: the doves' six draws ride the shared
`loot_rng` UpdatePlayers borrow (both ports agree); the sky's private
stream is `world_seed ^ 0x1A2B_3C4D` (both branches now carry the same
graft — c39's interim copy seeds identically, `event_skies.rs:105`);
the luck write is ONE beside-auras/zones snapshot (c39 retired its
duplicate, `ce288dac`). All witnesses across the family remain WRITTEN
NOT RUN until the merged tree runs the catalog command.

### 9.5 RETIRE commits + verification

| Hash | Item |
| --- | --- |
| `85a9253c` | 1 — §9.1 the doves adjudication + keep-one publish + brace fix (editing.rs + anchor + this note's §9) |
| `40256123` | 2 — §9.2 the shop-row dedupe receipt (shop.rs + npcs.rs feed comments) |
| `0ec61e92` | 3 — §9.3 the 114 FINAL receipts (save.rs no-v18-claim + world_clear doc) |
| (this) | 4 — §9.4 the LANTERN-FAMILY declaration (lantern_sky.rs module doc + ANALYSIS 'The RETIRE phase' + INDEX + PORT_MAP rows) |

Evidence: `Player.cs:48360-48404` (the doves arm re-read whole),
`:47180-47253` (the anchor/facing prologue), `:47280-47309` (the
zero-aim fallback), `:49425-49454` (the generic tail);
`Projectile.cs:10860-10889` (`NewProjectile`'s half-box centre);
`Chest.cs:2270-2369` (the whole Party Girl case — the 4702 row at
`:2309-2312`, the eleven gift rows at `:2345-2358`); both doves commits
read via `git show` (d17 `8a9e035e`, f40 `3d7a9b77`); both branch-tip
`editing.rs`/`shop.rs`/`npcs.rs` reads via `git show`; c39's research/114
whole at `4f72847e` (§7-§11) + its `event_skies.rs`/`save.rs` markers;
f41's shop feed at `be4cb841`. The brace census (2046 open / 2045 close
before the fix, balanced after) verified by direct file read. Code-only:
no cargo/rustc/tests/rustfmt/graphify invoked.
