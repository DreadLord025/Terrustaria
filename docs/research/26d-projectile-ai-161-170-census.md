# 26d — Projectile aiStyles 161-170 census (W4-d17)

Base: `a725acb0` (origin/main, wave-3 complete). Branch:
`task/W4-d17-proj-ai-161-170`. Code-only mode — witnesses WRITTEN NOT RUN.

The C# dispatch (`Projectile.cs:36607-36646`) arms ten styles in this band.
Verdicts against the port's `match data.ai_style` (`projectile.rs::update`)
and the kind-gated modules:

| Style | C# machine | C# lines | Port status at census | Action |
| --- | --- | --- | --- | --- |
| 161 | `AI_161_RapierStabs` | `:47612-47665` | MISSING — no rows (802/842/938-945/1127), no arm | ported this lane |
| 162 | `AI_162_TigerPounce` | `:46701-46768` | MISSING — no 818 row, no arm | ported this lane |
| 163 | `AI_163_Chum` | `:53474-53603` | MISSING — no 820 row, no arm (the bobber doc's "chum book unported" note stands for the HELPER only) | ported this lane |
| 164 | `AI_164_StormTigerGem` | `:46593-46699` | a9 `db50ff98` (`storm_tiger_gem_step` + `ai_164_home_location`, rows 831/970, `proj_frames` 6) — NOT in main yet | VERIFY-ONLY: re-read C# whole, port matches (keep-alive pin 2, 4-tick pulse wrap 6, upgrade latch, ring ladder, floored mount anchor). No re-port. |
| 165 | `AI_165_Whip` | `:47667-48270` | LANDED main — `ai_whip` (`projectile.rs:5573`), dispatch arm `165 =>` (`:18003`), the i24/i27 whip lane | none |
| 166 | `AI_166_Dove` | `:47540-47610` | MISSING — no 851/855 rows, no arm | ported this lane |
| 167 | `AI_167_SparkleGuitar` | `:46573-46591` | LANDED main — `ai_sparkle_guitar` (`projectile.rs:6962`), arm `167 =>` (`:17516`), 856 row (`data.rs:3829`). Re-read: the 89-tick kill, the CatmullRom control points and the `2π·t` rotation match; the HotPink light (`:46588`) is the recorded skip | none |
| 168 | `AI_168_FirstFractal` | `:46537-46571` | MISSING — no 857 row, no arm | ported this lane |
| 169 | `AI_169_Smolstars` | `:45718-45855` | a9 `dc35f100` (`smolstar_step`, 864 row, `proj_frames` 2) — NOT in main yet | VERIFY-ONLY: re-read C# whole, port matches (idle orbit/steer, 800-px acquire, −1 tumble's 2×Next(3)+1 NextFloat, 900-px leash, 16-speed chase, width·5 separation). No re-port. |
| 170 | `AI_170_FairyGlowstick` | `:45662-45716` | LANDED main — `fairy_glowstick_hover` (`projectile_ai_light_items.rs:423`), reached through the 870 style-flip (`local_ai[2]` latch, `glowstick_step`); the row keeps aiStyle 14 (the C# flips at runtime, `:24832-24835`). Re-read: the Item28 skip, the X-zero/`ai[1]=2` latch, the 4-frame machine, the ±10-cell repulsion, the ×0.95 drag + <1 snap match | none |

## In-flight probe (before taking the missing five)

`git fetch --prune` + every local/remote branch ahead of main diffed for
projectile files: a9 owns 156/164/169/206 (its `UNARMED_ROWED` census drops
them one per commit); d16 owns 151-154; NO branch claims 161/162/163/166/168.
The f19 census (`e7e3c723`, band 1-105) and its F2 finding (`c156903d`) hand
164/169 to "d17" by band, but a9's pushed commits supersede that claim —
verify-only here, per the Lead's instruction.

## Row census (the `SetDefaults` arms the missing styles need)

- 161: `DefaultToShortsword` (`:10773-10787`) — 18×18, pen −1,
  `tileCollide = false`, melee, `ownerHitCheck`, `extraUpdates = 1`,
  `usesOwnerLight`, drawLayer 7. Callers: 802 (`:7921-7924`), 842
  (`:8145-8148`), 938-945 (`:8917-8921`, `timeLeft = 360`), 1127
  (`:10559-10564`, + local immunity −1).
- 162: 818 (`:8005-8018`) — 132×132, alpha 255, `tileCollide = false`,
  `ignoreWater`, pen −1, `extraUpdates = 3`, local immunity −1.
  `UpdatePosition` skip list (`:20435`) names 818 — the CatmullRom arc IS
  the position. `TrailingMode[818] = 2` (`ProjectileID.cs:521`).
- 163: 820 (`:8019-8029`) — 12×12, `tileCollide = true`, `netImportant`,
  pen −1, `timeLeft = 86400`. `projFrames[820] = 4` (`Main.cs:8452`).
- 166: 851 (`:8173-8181`) — 4×4, pen −1, `timeLeft = 300`,
  `netImportant`; 855 (`:8197-8205`) — 12×12, pen −1, `timeLeft = 1000`,
  `netImportant`. `projFrames` 4 both (`Main.cs:8657`/`:8660`). The
  blocked-move bounces are the OUTER type chain (`:18336-18361`), ahead of
  the generic kill arm.
- 168: 857 (`:8220-8236`) — 32×32, alpha 255, melee, `tileCollide = false`,
  `ignoreWater`, `extraUpdates = 1`, local immunity 10,
  `manualDirectionChange`, pen −1, drawLayer 6. NO `projFrames[857]`
  override in `Main.cs:8444-8727` — the stock 1 stands even though the AI
  rolls `frame = Next(15)` (the draw reads `GetFirstFractalColor` +
  `DrawPrettyStarSparkle`, `Main.cs:33394-33401`, not a frame slice).
  `TrailingMode[857] = 0`.

## Damage-side riders the ports wire

- 161: `Colliding`'s stab arm (`:15955-15970`) — the AABB-vs-line from
  `Center` to `Center + velocity·6` (1127: ·28) at width `10·scale` (the
  `num38` 10/18 local is computed but the call passes `10f·scale` — a
  verbatim C# quirk, kept). `ownerHitCheck` (`:12637` →
  `CanHitWithMeleeWeapon` `:12400-12411`, the 1000-px default distance
  `:304`). 1127's `StatusNPC` Oiled-397 300-tick grant (`:11721-11724`).
  `CutTiles`' stab line (`:15229-15237`) rides the unported cut layer.
- 162: `Damage_CanDealDamage`'s `(type == 818 && ai[0] < 1)` refusal
  (`:12448`); the gem-count damage multiplier `1.5 + 0.4·max(0,
  ownedProjectileCounts[831] − 1)` (`:12861-12864`); `Kill()`'s 818
  re-`Damage()` (`:71631-71634`) + the two 3-iteration dust rings
  (`:71587-71596`, 30 explicit `NextFloat`s kept, the `NewDustPerfect`
  internals go with the dust pool).
- 163: the chum helper book (`ChumBucketProjectileHelper.AddChumLocation`,
  `:53601`) stays unported (the bobber module's `chumsInWater = 0` note);
  the `timeLeft` 60-cap + `netUpdate` (`:53594-53598`) is ported.
- 166: `GetAlpha`'s 855 gold lerp (`:78960-78963`) is draw-side, unported.
- 168: `GetFirstFractalColor` = `hslToRgb(ai[1], 1, 0.5)` (`:70517-70520`)
  feeds only the skipped dust + the draw sparkle.

## Deliberate skips (ANALYSIS conventions, per machine)

Dust/gore/light BODIES skip (no pool layer); every EXPLICIT call-site
`Main.rand` draw stays in C# order (the misc-module G3 standard); the
`Gore.NewGore` internal kicks (`Next(10,31)` Y then `Next(-20,21)` X,
`Gore.cs:1327-1328`) stay; `ChildSafety.Disabled = true` statically
(`ChildSafety.cs:33`) so the dangerous-gore re-roll never fires. Sounds ride
`wants_sound` where the pool has the seam (161's Item1 stab whoosh); the
1127 light write and the 855 lantern light skip. The chum's wet probe uses
the aiStyle-16 centre-cell stand-in (the i16 precedent — the slot's liquid
flags are written by the generic Update tail the port does not run for this
family); its wet integrate folds the `wetVelocity` ×0.5 water drag with the
blocked-axis restore (`:17642-17655`).
