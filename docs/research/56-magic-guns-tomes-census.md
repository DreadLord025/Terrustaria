# S1-P-i22 — magic guns + tomes use census (`Player.ItemCheck_Shoot`)

Scope: the 14 listed items only. C# = `decompiled/Terraria/Player.cs`
`ItemCheck_Shoot` (`:47056-49468`) unless noted. Port = the
`world_sim/editing.rs` use pass + `projectile/data.rs` rows + `projectile.rs`
AI arms. Code-only mode: witnesses WRITTEN, NOT RUN.

Item ids (`ID/ItemID.cs`): Starfury 65, StarCannon 197, MagicalHarp 494,
LaserRifle 514, NettleBurst 788, LeafBlower 1178, HeatRay 1295, PoisonStaff
1308, ElectrosphereLauncher 2796, ChargedBlasterCannon 2882, CrystalSerpent
3209, SpiritFlame 3779, SkyFracture 3787, SparkleGuitar 4715.

## Verdicts (before this task)

| item | shoot | C# arm | port before | verdict |
| --- | --- | --- | --- | --- |
| LaserRifle 514 | 88 PurpleLaser | generic + `ai0 = Next(3,6)` `:49379-49382` | generic, ai0=0 | WRONG (ai0 missing) |
| HeatRay 1295 | 260 | generic, no special | no row 260 → no spawn | MISSING |
| ChargedBlasterCannon 2882 | 460 | generic spawn; AI_075 `:66360-66538` | no row 460 → no spawn | MISSING |
| ElectrosphereLauncher 2796 | 442 | `projToShoot = 442` `:47169-47172`; `ai0/ai1 = tileTarget` `:49388-49392` | ammo path, ai0/ai1=0, no row 442 | WRONG + MISSING |
| LeafBlower 1178 | 206 Leaf | jitter `:49426-49429` | ported (`editing.rs` generic LEAF arm) | OK |
| PoisonStaff 1308 | 265 | 3-or-4 volley `:48115-48134` | generic single, no row 265 | WRONG + MISSING |
| NettleBurst 788 | 150 | generic; AI_004 chain `:23694-23828` | no row 150 → no spawn | MISSING |
| MagicalHarp 494 | 76 | note arm `:47392-47423` | generic single, no rows 76-78 | WRONG + MISSING |
| SparkleGuitar 4715 | 856 | guitar arm `:47994-48025` | generic single, no row 856 | WRONG + MISSING |
| StarCannon 197 | 955 | ammo path (FallenStar 75); AI_005 `:23830-24099` | ammo path, no row 955 | MISSING (row) |
| Starfury 65 (fallback) | 9 | `projToShoot==9` `:47251-47266` + type-65 arm `:49095-49108` | generic single, no row 9 | WRONG + MISSING |
| CrystalSerpent 3209 | 521 | generic; AI_029's 521 arm `:37705-37741` | no row 521 → no spawn | MISSING |
| SkyFracture 3787 | 660 | ring arm `:48808-48827` | ported but WRONG RNG + lerp + CanHitLine | WRONG |
| SpiritFlame 3779 | 659 | retry arm `:48793-48807` | ported, correct draws, but CanHitLine not CanHit | WRONG (probe) |

## The C# arms, verbatim order + RNG

All draws ride `Main.rand` under `Player.ItemCheck` → the `UpdatePlayers`
stream (`Main.cs:17754`). The port's use pass draws on
`app.rngs.stream(RngKey::UpdatePlayers)`. The crit roll
(`tr_core::damage::crit_roll`, `Next(1,101)`) is the port's spawn-time stand-in
for the C#'s hit-time crit (`Projectile.cs:12890-12898`); it precedes the
per-item draws exactly as the existing gem/blaze arms do.

### projToShoot == 9 (Starfury star, `:47251-47266`) — shared by 65
```
num2 = mouseX + screenPosition.X            // the cursor world X
num3 = -1
if num2 < player.Left.X: num3 = 1
else if num2 <= player.Right.X && rand.Next(2)==0: num3 = 1
pointPosition = (position.X + width*0.5 + rand.Next(201)*num3
                 + (mouseX+screenPos.X - position.X), MountedCenter.Y - 600)
KnockBack = 0; Damage = (int)(Damage * 1.5f)
```
RNG: `Next(2)` ONLY when `Left.X <= num2 <= Right.X`; then `Next(201)` always.
The type-65 arm (`:49095-49108`) then spawns at that point with the aim
velocity and `ai1 = vec.Y`, where `vec` walks from MouseWorld toward the
spawn (`SafeNormalize(0,-1)`) while `vec.Y > pointPosition.Y &&
WorldGen.SolidTile(vec.ToTileCoordinates())`, stepping `vector70*16`.

### projToShoot == 76 (MagicalHarp, `:47392-47423`)
```
projToShoot += rand.Next(3)                  // 76/77/78
num7 /= Camera.SmallerScaledAxis / 2; clamp num7 <= 1   // num7 = cursor dist
num10 = num4 + rand.Next(-40,41)*0.01
num11 = num5 + rand.Next(-40,41)*0.01
num7 *= 1.75; num10 *= num7+0.1; num11 *= num7+0.1
vector8 = MouseWorld - Center + rand.NextVector2Circular(40,40)  // 2 draws
num12 = Remap(vector8.Length(), 540, 960, 20, 60)   // clamped
vector9 = vector8 / num12 * 2
spawn(pointPosition, vector9, projToShoot, Damage, KnockBack, ai0=0, ai1=1,
      ai2 = vector9.Length()/num12)
num7 = clamp(num7*2-1, -1, 1); projectile.ai[0] = num7   // overrides ai0
```
RNG order: Next(3), Next(-40,41), Next(-40,41), circular(angle,radial).
`SmallerScaledAxis` = min(ScaledSize.X, ScaledSize.Y) (`Camera.cs:29-40`),
ScaledSize = screen − 2·GameViewMatrix.Translation (`Camera.cs:14`).

### type 4715 (SparkleGuitar, `:47994-48025`)
```
vector36 = MouseWorld
if GetSparkleGuitarTarget(out targets):       // :46626-46643
    npc = targets[rand.Next(targets.Count)]
    vector36 = npc.Center + npc.velocity*20
vector37 = vector36 - Center
if !target:
    vector36 += rand.NextVector2Circular(24,24)   // 2 draws
    if vector37.Length() > 700: vector37 *= 700/len; vector36 = Center+vector37
vector38 = rand.NextVector2CircularEdge(1,1)      // 1 draw
if vector38.Y > 0: vector38 *= -1
if |vector38.Y| < 0.5: vector38.Y = (0-rand.NextFloat())*0.5-0.5  // 1 draw
vector38 *= vector37.Length()*2
spawn(pointPosition, vector38, 856, Damage, KnockBack, ai0=vector36.X, ai1=vector36.Y)
```
`GetSparkleGuitarTarget`: chaseable NPCs whose Hitbox intersects the
1000×800 box centred on the player (`Utils.CenteredRectangle`).

### type 1308 (PoisonStaff, `:48115-48134`)
```
num84 = 3; if rand.Next(3)==0: num84++
for num85 in 0..num84:
    num86 = num4 + rand.Next(-35,36) * (0.05*num85)
    num87 = num5 + rand.Next(-35,36) * (0.05*num85)
    num6 = speed / sqrt(num86²+num87²)
    spawn(pointPosition, (num86*num6, num87*num6), 265, Damage, KnockBack)
```
RNG: Next(3), then 2·num84 draws. num85=0 multiplies by 0 but STILL draws.

### type 3787 (SkyFracture, `:48808-48827`) — the RNG fix
```
f = rand.NextFloat()*2π                       // draw 1
vector60 = pointPosition + f.ToRotationVector2()*Lerp(20,60,rand.NextFloat())  // draw 2 (PRE-LOOP, overwritten)
for 0..50:
    vector60 = pointPosition + f.ToRotationVector2()*Lerp(20,60,rand.NextFloat())  // radial draw
    if CanHit(pointPosition, vector60 + (vector60-pointPosition).SafeNormalize(UnitX)*8): break
    f = rand.NextFloat()*2π                   // angle re-roll
v4 = MouseWorld - vector60
vector61 = (num4,num5).SafeNormalize(UnitY)*speed
spawn(vector60, Lerp(v4.SafeNormalize(vector61)*speed, vector61, 0.25), 660, ...)
```
The port's pre-fix loop DROPPED the pre-loop radial draw (`:48813`) and
re-rolled the angle before the first probe; the C# draws angle+radial once
before the loop (the radial result discarded), then per iteration radial →
probe → angle. The lerp fallback is `vector61` (not zero) and the probe
fallback `UnitX` (not zero). The probe is `Collision.CanHit` (`:48817`) —
the port used `can_hit_line`, fixed to `can_hit`.

### type 3779 (SpiritFlame, `:48793-48807`) — draws correct, probe fixed
angle = NextFloat·2π; 10 retries (probe = spawn + shot.rotate(angle)·100,
`Collision.CanHit` `:48798` — the port used `can_hit_line`, fixed to
`can_hit`); speed = 0.95+NextFloat·0.3 AFTER the loop; spawn at
pointPosition + v·30, velocity 0, ai0 = −2.

OUT OF SCOPE (recorded, not fixed): the Nebula Blaze 3542's probe
(`:48783`) is the same `Collision.CanHit` but the port uses `can_hit_line`
— the same bug class; 3542 is not one of the 14 listed items, so it is left
for its own task (the gem-staff / nebula census).

### type 65 (Starfury sword, `:49095-49108`) — the fallback
After the projToShoot==9 block above, spawns `(num4,num5)` at pointPosition
with `ai1 = vec.Y` (the SolidTile walk). The port wires the type-65 arm;
the projToShoot==9 block is shared.

### generic tail ai0/ai1/ai2 (`:49376-49437`)
- projToShoot==88 (LaserRifle): `ai0 = rand.Next(3,6)` (`:49379-49382`).
- projToShoot==442 (Electrosphere): `ai0 = tileTargetX; ai1 = tileTargetY`
  (`:49388-49392`). tileTarget = cursor tile clamped to [5, max−5]
  (`Player.cs:25389-25410`).
- projToShoot==206 (LeafBlower): jitter (`:49426-49429`) — ported.
- Everything else in scope: ai0=ai1=ai2=0.

### ammo shooters (StarCannon 197, Electrosphere 2796)
The `useAmmo > 0` path (`editing.rs` ~1712) runs PickAmmo + the stat fuse.
StarCannon: useAmmo 75 (FallenStar), shoot 955; the FallenStar has shoot 0
(`Item.cs:2301-2314`) so projToShoot stays 955. Electrosphere: useAmmo 771
(Rocket), shoot 442; `PickAmmo`'s rocket arm ADDS the ammo shoot
(`Player.cs:53483-53486`), then `:47169-47172` forces 442. The port's ammo
path spawns `resolved.projectile` with ai0/ai1=0 — Electrosphere needs the
tileTarget ai0/ai1.

## Projectile rows + AI (SetDefaults cites)

| kind | SetDefaults | aiStyle | AI arm |
| --- | --- | --- | --- |
| 9 Starfury | `:886-896` | 5 | `:23830-24099` (tileCollide = Bottom.Y>=ai1 `:23865-23868`) |
| 76/77/78 notes | `:1556-1593` | 21 | `:25585-25606` (decel + bounce `:19299-19309`) |
| 150/151/152 nettle | `:2398-2410` | 4 | `:23694-23828` (chain `:23762-23781`, flip `:23709-23713`) |
| 260 HeatRay | `:3441-3451` | 48 | `:28204-28242` (delay 3 `:53990-53992`) |
| 265 PoisonFang | `:3503-3515` | 1 | AI_001 straight (flag3 skip `:55973`) |
| 442 missile | `:5126-5136` | 1 | `:54726-54771` (fly to ai0/ai1·16+8, Kill≤16) |
| 443 sphere | `:5137-5149` | 77 | `:30671-30761` (pulse box, 120-tick) |
| 459 orb | `:5292-5303` | 1 | `:54651-54695` (scale=clamp(ai1,.7,1.6), box 22·scale) |
| 460 holder | `:5304-5316` | 75 | `:66360-66538` (charge machine) |
| 461 beam | `:5317-5328` | 84 | `:31405-31410` + `:31486-31499` + `:31572-31618` |
| 521 CrystalPulse | `:5908-5918` | 29 | `:37705-37741` (alpha −50 fade) |
| 856 SparkleGuitar | `:8206-8219` | 167 | `:46573-46591` (CatmullRom arc) |
| 955 StarCannonStar | `:9013-9023` | 5 | `:23830-24099` (generic ai1 latch `:23871-23879`) |

Hit-time damage multipliers (`Damage_PVE_Inner`'s per-type switch
`:13409+` and `Damage_GetProjectileSpecificDamageMultiplier` `:14432+`):
265 ×0.75 (`:13414-13416`, a `this.damage` mutation that COMPOUNDS per hit),
76/77/78 ×0.95 (`:13432-13436`), 9 ×0.75 (`:14502-14504`). These are
HIT-time folds in the projectile damage system, NOT the use path — the
port's `damage_pve` applies no per-kind hit multiplier today (only the
flail family), so they are left unported and recorded here as a gap. The
spawn carries the raw weapon damage. The Starfury's ×1.5 (`Player.cs:47265`)
IS a use-path fold and IS applied at spawn.

## Deliberate deviations (this task)

1. **Dust RNG skipped pool-wide.** The C# AI arms draw `Main.rand` for
   dust/gore/sound (e.g. AI_005's `:23883/:23973/:23983/:24035/:24039`,
   AI_021's `:25594`, the 459 orb's `:54678`). The port's pool keeps ZERO
   RNG (the S1-D-5 slice-A precedent, `projectile_ai001_a.rs` module doc),
   so these draws are NOT consumed — the `UpdateProjectiles` stream stays
   untouched by construction. The shoot-path draws (above) ARE consumed in
   C# order on `UpdatePlayers`.
2. **Crit at spawn.** The port rolls crit at spawn (the existing use-pass
   convention); the C# rolls at hit. Same stream, same count per shot.
3. **Hit-time multipliers at spawn.** 265/76-78/9 damage scaled at spawn
   (the gem-staff `damage_multiplier` precedent), not at hit.
4. **`SmallerScaledAxis`** computed from the port's screen + zoom
   (`GameViewMatrix.Translation`, the `draw.rs:735-746` formula).
5. **ChargedBlasterCannon 460/461 + Electrosphere 443**: the channel charge
   machine + the sphere pulse are ported; the dust/sound/light visuals skip
   (no pool layer). The 460's `GetSlowMagicUseRate` = 1 (no ManaV2).