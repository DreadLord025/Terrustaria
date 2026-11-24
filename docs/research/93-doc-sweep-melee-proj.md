# 93 — DOC-SWEEP lane 17 (melee-proj): the item use-paths behind aiStyles 161/163/166/168/182 + the whip tag census

Branch `task/W4-d17-proj-ai-161-170` (DONE phase `84126544`, DOC-SWEEP continuation).
Code-only mode: no cargo/tests/graphify invoked; every witness below is WRITTEN NOT
RUN. All line numbers cite `decompiled/Terraria/` (the 1.4.5.8 decompilation) or the
port's `crates/` tree at this branch's tip.

The d17 machine phase landed the five AI arms (161 rapier stabs, 162 tiger pounce,
163 chum, 166 dove, 168 first fractal). This sweep censuses the ITEM side — the
`Player.ItemCheck_Shoot` launch arms that feed those machines — and ports the REAL
gaps it finds. Five commits, one per item:

| commit | item | scope |
|---|---|---|
| `43366016` | 161 | the rapier launch arm + jitter + style-13 aim + saber ×0.7 |
| `9b44445e` | 163 | the chum fishing book (helper, pond sum, reduction) + use-path witness |
| `8a9e035e` | 166 | the Release Doves three-bird arm + the 4713/peace-candle census |
| `b80205e9` | 168/182 | the Zenith/True-Copper launch arm + the deprecated-4722 census |
| `2c923acc` | whip | the tag-application summoner-speed buff grant + range/tag census |

---

## 1. Item 161 — the rapier item use-path (Iron Shortsword 6, useStyle 13, shoot 940)

### Census

The i05 note ("the Iron Shortsword is useStyle 13 since the rapier rework — its
lunge is a projectile, not the Thrust arm") was correct but left the LAUNCH side
unported. The C# chain for a useStyle-13 item:

* `ItemCheck_Shoot`'s aim jitter — `Player.cs:47335-47348`: ONLY the proj kinds
  **802 (Gladius) / 842 (Ruler) / 1127 (Chlorophyte Saber)** get the
  `SafeNormalize(Zero).RotatedBy(num8·(NextFloat−0.5)) · (len − NextFloatDirection·num9)`
  jitter. `num8 = π/4`, `num9 = 0.7`; the 1127 takes `num9 = 0.5` and
  `num8 = 0.65 · Remap(meleeSpeed, 1, 1/3, 1, 0.25)` (clamped, descending range).
  The ore shortswords' 938-945 kinds are NOT named — they fly the plain aim.
* The launch arm — `Player.cs:48840-48844`: `sItem.type == 4463 || 486 || 1227`
  spawns at the EXACT `pointPosition` (`RotatedRelativePoint(MountedCenter)`,
  `:47196` — no `+dir*10`) with `ai1 = sItem.shootSpeed / speed`. `speed` already
  divided by `meleeSpeed` at `:47061-47063`, so the quotient is the INVERTED
  melee-speed multiplier — exactly the value `AI_161_RapierStabs`' 1127 stretch
  reads (`Projectile.cs:47621-47626`, the d17 machine). The arm `return`s.
* The 1227 damage pre-pass — `Player.cs:47143-47146`:
  `Damage = (int)((double)Damage * 0.7)` runs BEFORE every spawn arm (80 → 56).
* The style-13 `itemRotation` write — `Player.cs:47376-47381`:
  `atan2(num5·direction, num4·direction) − fullRotation` off the (jittered) aim,
  ONCE per shot — NOT per tick (the style-5 per-tick gate does not cover 13).
* The use-start sound flag — `Player.cs:51684`: `flag3 = useStyle==5 ||
  useStyle==13 || shoot>0` — the single-player arm always plays (`:51694`), so
  the port's `play_use_sound` seam already covers it. No gap.
* The body rows — `Player.cs:36400-36414` (the thirds walk) and the composite
  arm — `:51174-51196` (the Quarter/ThreeQuarters/Full stretch ladder): both
  landed in M3-ITEM-2 / S1-G-18 (ATLAS `M3-ITEM-2`, ANALYSIS "The throw and
  rapier item placement"). No gap.

### Ported (commit `43366016`)

`world_sim/editing.rs::fire_non_ammo_shot`:
* the `4463 | 486 | 1227` arm — the jitter (two `UpdatePlayers` draws: the angle
  `NextFloat`, then the length `NextDouble·2−1`), the exact-anchor spawn
  (half-box subtracted, the i08 precedent), `ai1 = prefixed shootSpeed / fx_spd`;
* the 1227 ×0.7 pre-pass on `raw` (f64 multiply, the C#'s double cast);
* the style-13 `swing_aim` write off the shot velocity (the generic path for
  938-945, the jittered overwrite in the rapier arm);
* `start_swing_extras` skips the style-13 mouse-aim overwrite (the C#
  `ApplyItemAnimation` never touches `itemRotation`; the shot-time write owns it).

Witnesses (WRITTEN NOT RUN, `s1_p_i08_throw_tests`):
`w4_d17_the_rapier_launch_spawns_at_the_anchor_with_the_speed_ratio` (the trio's
anchor + ai1 = 1 at stock speed; the Iron Shortsword 6 keeps ai1 = 0),
`w4_d17_the_rapier_jitter_draws_two_and_the_saber_cuts_damage` (the two-draw
census, the ±0.7/±π/8 bands, the jittered aim write, 80 → 56, the +0.2-bonus
ai1 = 1/1.2).

### Remaining (out of scope, recorded)

* The 1127's `Lighting.AddLight` + dust bodies (`Projectile.cs:47648-47657`) —
  the d17 machine's cited skip stands (the two NextFloat draws stay).
* The `meleeSpeed` remap's `Utils.Remap` is clamped (`remap_clamped`); the C#
  default `clamped: true` matches.

---

## 2. Item 163 — the chum fishing-blood interaction (Chum Bucket 4608 → 820)

### Census

The i35 fishing lane landed the bobber + the attempt builder, but the chum book
was explicitly deferred (`projectile_bobber.rs` module doc: "the chum-bucket
projectile book is unported, so chumsInWater is 0"). The C# chain:

* The use — `Item.cs:38233-38245`: 4608 is useStyle 1, shoot 820, shootSpeed 7,
  consumable, noMelee. `ItemCheck_UseBuckets` (`Player.cs:46320`) names ONLY the
  liquid buckets (205-207/1128/3031/3032/4820/4872/5302-5304/5364) — the chum
  bucket rides the GENERIC thrown shot. The port's generic path already fires it
  (the witness pins it); no use-arm gap.
* The helper — `GameContent/ChumBucketProjectileHelper.cs:6-33`: two
  `Dictionary<Point,int>` maps; `OnPreUpdateAllProjectiles` (`Main.cs:17538`)
  swaps pending→last + clears pending; `AddChumLocation(spot)` keys
  `spot.ToTileCoordinates()` (`Utils.cs:1807-1810`: `(int)X >> 4`, arithmetic
  shift); `GetChumsInLocation` reads LAST frame's map (the one-tick lag is the
  C#'s own).
* The feed — `AI_163_Chum` `Projectile.cs:53593-53602`: `spot = Center + (0,16)`;
  `ai[0] >= 10 && timeLeft > 60` caps the life to 60 + netUpdate; while
  `ai[0] < 10 && timeLeft > 60` the bait calls `AddChumLocation(spot)`.
* The bonus — `GetFishingPondState` `Projectile.cs:21597-21599` sums
  `GetChumsInLocation` per scanned WATER cell into `chumCount`; the attempt
  builder adds **+11 (>0), +6 (>1), +3 (>2)** (`:20638-20649`). The port's
  `tr_core::fishing` ladder already existed — only the count was 0.
* The reduction — `ReduceRemainingChumsInPool` `Projectile.cs:53605-53656`,
  called from the bobber's reel head at the `frameCounter == 0` latch
  (`:53709-53713`): every active local-owner 820 with `timeLeft > 60`
  contributes its `Center+16` tile; the pond-width column walk steps Y BEFORE
  matching (`:53631` — a chum on the pond's TOP row is stepped past), removes
  matched chums, and ages each `ai[0]++` + netUpdate (one bite of the 10-use
  budget). Note the bobber's tile read is `(int)Center.X / 16`
  (truncation-then-divide, `:53607-53608`) — NOT the shift form.

### Ported (commit `9b44445e`)

* `ProjectilePool` gains `chum_pending`/`chum_last` maps; the swap+clear runs at
  the `update` head (the `Main.cs:17538` site); `chum_step`'s tail feeds the
  pending map under the C#'s gate.
* `tr_core::fishing::get_fishing_pond_state_chums` — the pond scan with the
  per-cell lookup closure (`PondState.chums_in_water`); the plain
  `get_fishing_pond_state` delegates with a zero closure (every existing caller
  unchanged). `get_fishing_pond_width` + `pond_cell_solid` exposed for the
  reduction walk.
* `projectile_bobber::fishing_check` computes the count off the pool's last-frame
  map and feeds `AttemptInputs.chums_in_water` (the +11/+6/+3 ladder now live).
* `reduce_remaining_chums_in_pool` (pool-side) rides the bobber's new
  `wants_chum_reduce` intent (the `wants_splash` pattern — the arm cannot see the
  sibling slots).

Witnesses (WRITTEN NOT RUN): `w4_d17_the_chum_book_swaps_pending_to_last`,
`w4_d17_the_reel_ages_the_water_chums_only` (the top-row step-past quirk pinned),
`tr_core::fishing::the_pond_scan_sums_the_chum_book` (the +20 ladder),
`w4_d17_the_chum_bucket_throws_the_bait_and_consumes` (the use path).

### Remaining (out of scope, recorded)

* The gore bodies (1201/1208) stay skipped (the d17 census — the gate draws ride
  the stream). The `Gore.ChumFloatingChunk_GetWaterLine` draw layer is unported.

---

## 3. Item 166 — the Dove item (the 4713 / peace-candle question)

### Census — the id question resolved

* **Item 4713 is `Fake_DungeonDesertChest`** (`ItemID.cs:11109`) — a tile placer
  (tile 468, placeStyle 12), NOT a dove item. The port's row is faithful.
* **PeaceCandle is item 3117** (`ItemID.cs:7917`) — a placeable candle
  (`Player.cs:50017`'s hold arm), no projectile. Not in this lane's scope.
* The dove pair is **4682 Release Doves → 851** and **4702 Release Lantern → 855**
  (`Item.cs:38555-38562` / `:38644-38651`, both `DefaultToThrownWeapon(851|855,
  25, 4f)` + `ranged = false` + `noUseGraphic`; `ItemID.cs:11047`). The d17
  machine phase already rowed 851/855 + the AI_166 arm.

### The real gap — the three-dove launch

`Player.cs:48383-48392`: ONLY `sItem.type == 4682` has a special arm — THREE
doves per use, each at the aim velocity plus `Next(-20,21)·0.1` per axis (X draw
then Y draw — six `UpdatePlayers` draws), the C# `return`s. The lantern 4702 has
NO special arm (the generic single shot). The port fired ONE dove for 4682.

### Ported (commit `8a9e035e`)

The `held.kind == 4682` arm in `fire_non_ammo_shot` (three spawns, the per-axis
jitter, the shared consumable pay runs once per use). Witness
`w4_d17_the_doves_release_three_and_the_lantern_one` (WRITTEN NOT RUN).

---

## 4. Items 168/182 — FirstFractal vs FinalFractal (the Terra Blade progression)

### Census — both serve the correct items

* **FirstFractal ITEM 4722 is DEPRECATED** — `ItemID.Sets.Deprecated`
  (`ItemID.cs:1316`) names 4722; `Item.SetDefaults`' dispatcher epilogue runs
  `TurnToAir` → `SetDefaults(0)` for deprecated ids, so the port's generated
  placeholder row (`itemdata.rs:4888`, name "None", projectile None) is FAITHFUL
  — `tools/GenItemData/gen.py --check` passes (6196 rows). The item is
  unobtainable in 1.4.5.8; its `Item.cs:38788-38808` body (shoot 857, channel,
  useStyle 5) never survives the epilogue. **No item-side gap for 4722.**
* **Projectile 857 FirstFractal** (aiStyle 168, `Projectile.cs:8226-8240`) is the
  d17 machine (`ai_first_fractal`, commit `2f50bd72`). Its only live spawner in
  the C# is the deprecated 4722 — the proj row + AI stay for completeness (the
  Zenith's component-sword DISPLAY reads the profile table, not this proj).
* **FinalFractal 933 / TrueCopperShortsword 1100** (aiStyle 182,
  `Projectile.cs:8857-8873` / `:10265-10281`) — the rows + `AI_182_FinalFractal`
  (`:43111-43176`) are **d19's lane** (`task/W4-d19-proj-ai-181-190`, commit
  `a4185f89`, `projectile_ai_sword_styles.rs::ai_final_fractal`). NOT duplicated
  here (the branch-per-task rule).
* **The Zenith 4956 → 933 and True Copper Shortsword 5669 → 1100** item rows are
  faithful (`Item.cs:40000-40019` / `:44546-44566`: useStyle 1, 30/10, shootSpeed
  16, damage 190). The volley window (`Player.cs:43502`, `useTime*3`) was already
  ported (`editing.rs` `window_blocked`). **The LAUNCH arm was missing in every
  in-flight branch** (d19/d20 carry only the window) — the real gap.
* The Terra Blade 757 → 985/984 double-spawn (`Player.cs:48930-48937`) and the
  sword family (273/368/674/675/1226/1826) launch arms are the aiStyle-190/191
  lanes (d19/d20 row 972/982/983/984/985/997/1043/1126); their item-side arms
  belong to those lanes' wiring, not this sweep. Recorded, not ported.

### Ported (commit `b80205e9`)

The `4956 | 5669` launch arm (`Player.cs:48691-48730`) in `fire_non_ammo_shot`:
* the shot ordinal `(itemAnimationMax − itemAnimation) / itemTime` off the 60 Hz
  integer counts (the prefixed useTime, NOT the swing fallback);
* `random_fractal_profile` — `GetRandomProfileIndex`
  (`Graphics/FinalFractalHelper.cs:78-88`): one `Next(21)` over the 21
  insertion-order keys (`:128-214`, verbatim `FRACTAL_PROFILE_KEYS`); a 4956 draw
  removes it and re-rolls `Next(20)` (the short-circuit shape). Shot 0 forces the
  4956 Zenith profile; 5669 pins 3507 (the copper);
* the cursor clamp (`limit_point_to_player_reachable_area`, the i23 port),
  `zenith_target` (`GetZenithTarget` `Player.cs:46645-46669`, the strict
  `!(best <= d)` nearest-chaseable scan) at 400 px for shots 1/2, the
  shot-1-miss/shot-2 `NextVector2Circular(150,150)` scatter;
* the spawn at the exact `pointPosition` anchor, velocity `(target −
  MountedCenter)/2`, `ai0 = Next(-100,101)`, `ai1 = profile`.

Witness `w4_d17_the_fractal_profile_rerolls_the_zenith_key` (WRITTEN NOT RUN —
the one-draw/two-draw stream census, both arms exercised). The spawn-side
witness waits on d19's 933/1100 rows (the `proj_data` gate would no-op the spawn
on this branch — recorded, not faked).

---

## 5. Whip 165 — the range/tag census (i24 landed; the remaining gaps)

### Census — what i24/i27 already landed (verified, no gap)

* `GetWhipSettings` (`Projectile.cs:48337-48419`) — `tr_core::whip::
  get_whip_settings` is VERBATIM, including the `*=` vs `=` quirks (848/849/1033
  assign, 1034/1035 multiply) and the 1035 arm's `30·MaxUpdates` override.
* `whipRangeMultiplier` — the player field (`player.rs:797`, the Tiki/Stardust/
  Obsidian Outlaw/Mobius grants `:6418`/`:6799`) feeds the collision curve
  (`projectile.rs:23300-23314`, `FillWhipControlPoints` `:48301`). Wired.
* The tag damage math — `tag_damage_for` (`projectile.rs:16462`) gates on
  `CanRunHitEffects` (`WhipTagEffect.cs:75-86`, the MinionShot/SentryShot sets
  verbatim `ProjectileID.cs:431-432`), applies `SummonTagDamageMultiplier` +
  the forced crit (`:12919-12933`). The proc arms (SwordWhip +30, Firecracker
  ×2.75, Eel 50+50n) + the per-type switch (`:13904-13956`) landed in i24/i27.
* The tag table — `TAG_EFFECTS` (`tr_core::whip.rs:347`) matches
  `ItemID.Sets.UniqueTagEffects` (`ItemID.cs:1473-1580`) row for row.

### The real gap — the tag-application buff grant

`WhipTagEffect.OnTagAppliedToNPC` (`WhipTagEffect.cs:45-59`): applying a tag
grants the effect's `PlayerBuffId` for `(int)(PlayerBuffTime · tagEffectDuration)`
ticks — the summoner-speed buffs **308 SwordWhip +0.25 / 311 DarkHarvest +0.35 /
314 ThornWhip +0.12** (`Player.cs:10110-10121`, the port's buff pass already
reads them, `buff_effects.rs:144-146`). The port's tag write
(`projectile.rs:23858`) never granted them — the minion attack-speed bonus was
dead. The `PlayerBuffAppliedManually` rows (365 CobWhip, 312 CoolWhip) keep their
per-type-switch grants (`:13923`/`:13934`, already ported `:16772`/`:16779`).

### Ported (commit `2c923acc`)

The grant rides the tag write, gated by the same `ImmuneToWhips` refusal
(`CanApplyTagToNPC`) and the manual flag. Witness
`w4_d17_the_tag_application_grants_the_summoner_speed_buff` (WRITTEN NOT RUN —
the SwordWhip hit grants buff 308 for 180 ticks).

### Remaining (out of scope, recorded)

* The proc SPAWN bodies (the DarkHarvest 916, Firecracker 918, Starcrash 1037,
  FlowerWhip 1038, Constellation 1039 fan, Eel chain, CoolWhip 917 snowflake,
  CobWhip 1036 spider) — the deferred follow-ups the tag module doc names; the
  917/1036 rows + AI_177/67 belong to d18/d20's lanes.
* `AI_165_Whip`'s per-type dust/sound switch (`:47713-48270`) — the cosmetic
  layer, the standing skip convention.
* `MatchItemTimeToItemAnimation` (`:47697`, `Player.cs:4499-4502`) — the whip's
  held-item time pin; the port's swing/item timers already run in lockstep for
  the whip (the i24 census), no observable gap.

---

## Verification

Code-only mode (the lane's standing rule): **no cargo, no tests, no graphify
invoked.** Every witness above is WRITTEN NOT RUN, hand-derived from the cited
C# lines. The generated item table was check-run once
(`python tools/GenItemData/gen.py --check` → "check ok (6196 rows)") to confirm
the 4722 placeholder is faithful — no regeneration needed, no generated file
touched. Four `anchors.txt` canary rows added (the markers verified present in
the source). The branch diff is confined to this lane's files:
`world_sim/editing.rs`, `projectile.rs`, `projectile_bobber.rs`,
`projectile/data.rs`, `tr-core/fishing.rs`, `anchors.txt`, this doc.
