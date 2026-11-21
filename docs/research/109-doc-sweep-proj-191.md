# 109 — DOC-SWEEP lane 28: proj aiStyle 191-200 skip census

> Wave 4 · researched 2026-11-21 · sources: decompiled + crates + sibling lanes
> (d19/d69/d80/c10) · branch `task/W4-d20-proj-ai-191-200` (base DONE `13de9ccf`)

## Scope

The W4-d20 lane ported projectile aiStyles **191/192/195/196/198/199/200**
verbatim into `crates/tr-app/src/projectile_ai_misc.rs` (+ rows in
`projectile/data.rs`, dispatch in `projectile.rs`, the gift drain in
`world_sim/projectiles.rs`). This DOC-SWEEP re-verifies the **six recorded
skips** against the C# source and the sibling lanes, decides keep-one for the
one duplicate, and documents the tracked-loop contract d80 owns.

Method: static cross-reading only (no cargo/tests/graphify), per
`docs/research/MODULE-TEMPLATE.md`. Every claim carries a `File.cs:line` or
`crates/...rs::symbol` cite. "Not found" is reported as such.

**Headline result:** all six skips are either *correctly skipped* (the consumer
lives in another lane), a *duplicate* (keep-one at merge), or expose a *real
gap that is cross-lane* (item-use / pet-equip / camera). **No clean in-lane code
port was warranted** — porting any of the real gaps here would duplicate d19's
rows or encroach on the item-use `editing.rs` hot file. They are recorded below
as prioritized task seeds with full C# anchors so the owning lane can land them
1:1. The d20 AI arms themselves re-verify FAITHFUL.

| # | Skip | Verdict | Real gap? | Owning lane |
| --- | --- | --- | --- | --- |
| 1 | JimsDrone `CameraTracker`/`CheckSection` | correctly skipped | camera-pan source unported (not a render target) | camera / `world_sim/camera.rs` |
| 2 | Flint `itemLocation.Y` jitter draw-side | correctly skipped | none (draw-side, `noMelee`) | — (RNG draw kept) |
| 3 | 200-droppings dup vs c10 | duplicate — keep-one | none (both faithful) | merge decision (c10 module) |
| 4 | Jumino gift 5276 item row | row exists; use/eat arm inert | pet persistence + 994 row missing | pet-equip / item-use |
| 5 | 191 TrueNightsEdge vs 190 NightsEdge | AI arms faithful; use-path missing | terra-chain spawn arms absent everywhere | item-use (`editing.rs`) + d19 rows |
| 6 | tracked-loops JimsDrone/FlintCharge | d80 owns | deferred census sites | d80 tracked-sound |

---

## Item 1 — JimsDrone `DroneCameraTracker` / `CheckSection` (aiStyle 195)

**C# source.** `DataStructures/DroneCameraTracker.cs:1-34` is a 3-method camera
bookkeeping class, NOT a render target:

- `Track(Projectile)` (`:11-15`) stores `_trackedProjectile` + `_lastTrackedType`.
- `WorldClear()` (`:17-21`) nulls both (called `WorldGen.cs:7000`).
- `TryTracking(out Vector2 cameraPosition)` (`:23-33`) returns the projectile's
  `Center` iff `active && type == _lastTrackedType && owner == Main.myPlayer &&
  Main.LocalPlayer.remoteVisionForDrone`; else nulls the ref and returns false.

The single consumer is `Main.GetObjectTrackingCameraPan`
(`Main.cs:62722-62746`): after the spectate arm (`:62727-62732`) and the dead
gate (`:62733-62736`), `Main.DroneCameraTracker.TryTracking(out trackedPosition)`
(`:62737`) feeds the camera pan target, ahead of the golf arm (`:62741`). The
field is `Main.cs:1809`, constructed `Main.cs:67041`.

`Track` is called from two AI arms: the drone `AI_195_JimsDrone`
(`Projectile.cs:40955`, inside the `owner == Main.myPlayer` block) and the RC car
`AI_205_RemoteControlCar` (`Projectile.cs:38125`, arm at `:38076`). The gate flag
`remoteVisionForDrone` (`Player.cs:2390`) is set by equipping the **Jims Drone
Visor** item 5452 (`Player.cs:14213` `ApplyEquipVanity` case 5452, and `:15461`),
reset each frame in `ResetEffects` (`Player.cs:18712`). Item ids:
`ItemID.cs:12585` `JimsDrone = 5451` (the transmitter), `:12587`
`JimsDroneVisor = 5452`; projectile `ProjectileID.cs:2594` `JimsDrone = 1020`.

`CheckSectionsInCaseOwnerIsWatching` (`Projectile.cs:20376-20389`, called from
the drone tail `:41066`) is **MP section loading**: `netMode == 2` →
`RemoteClient.CheckSection(owner, position)` (`RemoteClient.cs:126`), else
`owner == myPlayer` → `ActiveSections.CheckSection(position)`
(`ActiveSections.cs:15`). It moves no gameplay state.

**Port status.** `projectile_ai_misc.rs::ai_jims_drone` (`:1092`) ports the drone
AI verbatim and explicitly skips both (`:1076-1078` doc, `:1140`
`// Main.DroneCameraTracker.Track — skipped (camera bookkeeping)`). The camera
pan port `world_sim/camera.rs::update_camera_pan` (`:71`) keeps
`target_pan = Vec2::ZERO` and documents (`:66-68`) that "the C#'s tracking
sources (the spectate/drone/golf cameras, `:62722-62745`) have no ported
consumers". `remoteVisionForDrone` has no port field (grep: 0 hits in crates).
`ActiveSections`/`CheckSection` have no sim port (the netglue `recheck_sections`
at `app/net_proj_sync.rs:295` is the MP wire sync, a different subsystem).

**d69 co-lane read.** `task/W4-d69-layer-target-consumers` (tip `9ba463ae`) ports
`crates/tr-render/src/targets.rs` — the `RenderToTargets` executor + skyTarget
clear + layer composite (`Main.cs:57277-57327/:62025-62026/:62196-63447`). It is
about the **full-screen render targets** (skyTarget/screenTarget/screenTargetSwap),
NOT the drone camera. Grep of d69 `targets.rs` for `drone|1020|remoteVision|camera`:
only generic `screen_size`/`offScreenRange` hits — **no drone-camera consumer**.

**Verdict — correctly skipped, NOT a render-target port.** The lane's port
condition ("port if self-contained render-target") does not hold:
`DroneCameraTracker` is a camera-pan *source*, and its consumer
(`GetObjectTrackingCameraPan`) is unported alongside the spectate and golf
tracking arms. Porting the tracker alone would be dead code (no pan consumer);
porting the pan consumer is the camera lane's scope (it must land spectate +
drone + golf together to keep the `:62722-62745` priority order). The
`CheckSection` half is MP-only. **Task seed T1** below.

---

## Item 2 — Flint `itemLocation.Y` jitter: draw-side vs location model (aiStyle 198)

**C# source.** `AI_198_Flint` (`Projectile.cs:52238-52315`), charge phase
(`ai[0] == 0`, `:52280-52314`):

```
player.itemAnimationMax = useAnimation;          // :52307
player.SetDummyItemTime(num5);                   // :52308
player.itemLocation.Y += Main.rand.Next(2);      // :52309  ← the jitter
if (Main.rand.Next(10) == 0) { ...dust 228... }  // :52310-52313
```

The jitter is **one `Main.rand.Next(2)` draw per charge tick** added to
`player.itemLocation.Y`. `itemLocation` (`Player.cs:1500`) is the held-item
draw anchor, recomputed every frame by `ItemCheck_ApplyHoldStyle`
(`Player.cs:50240`, the per-hold-style arms `:50279-50516` set
`itemLocation.X/.Y` from `position`/`MountedCenter`/`heldItemFrame`).

**Location-model census — is it gameplay or draw?** The readers of
`itemLocation` split cleanly:

- **Draw / light / dust** (the overwhelming majority): the hold-style placement
  itself (`:50279+`), `ItemCheck_EmitUseVisuals` dust/light (`:49904-50092`),
  the torch light (`:44916`), the melee-swing arc location (`:44751`).
- **Melee hitbox** (`ItemCheck_GetMeleeHitbox`, `Player.cs:45099-45126`): builds
  `itemRectangle` from `(int)itemLocation.X/.Y` (`:45102`/`:45126`). BUT its
  caller is gated `!sItem.noMelee` (`Player.cs:43648`:
  `((sItem.damage >= 0 && sItem.type > 0 && !sItem.noMelee) || ...)`) — and the
  **Dead Cells Flint item 5462 is `noMelee = true`** (`Item.cs:43095`). So the
  flint's `itemLocation` **never feeds a melee hitbox**; its damage rides the
  1043 slash / 1041 spikes the arm spawns, not the held-item rectangle.
- The bee-glove arm (`:43635`) reads it only for item 1124 (not the flint).

Therefore the `itemLocation.Y += Next(2)` jitter is **purely a draw-side
wobble** of the held flint sprite during the charge — it shifts no collision,
no spawn point (the slash spawns at `player.MountedCenter`, `:52254`, not
`itemLocation`), no damage. The only gameplay-observable effect is the **RNG
draw itself** on the `UpdateProjectiles` stream.

**Port status.** `projectile_ai_misc.rs::ai_flint` (`:1386`) keeps the draw and
documents the skip precisely:

- `:1376-1378` doc: "The `itemLocation.Y += Next(2)` jitter (`:52309`) keeps its
  draw on the stream; the draw-side item location is unmodelled (the thrust
  placement recomputes it each frame — a documented skip)."
- `:1478-1480`: `// itemLocation.Y += Next(2) — the draw stays; the location is
  draw-side (unmodelled).` then `let _ = rng.next_range(0, 2);` — **the draw is
  consumed**, preserving stream order ahead of the `Next(10)` dust gate
  (`:1482`) and the dust's three floats (`:1483-1485`).

The port has no persistent `player.itemLocation` field; the melee-swing draw
recomputes the anchor per frame (`melee_swing.rs:10` "render-side arc math
(`item_rotation`/`item_location`...)", `player.rs:410` `item_location: [f32; 2]`
is a draw-local). This matches the C# semantics: the jitter is a transient
draw offset that `ItemCheck_ApplyHoldStyle` overwrites next frame.

**Verdict — correctly skipped, draw kept.** The lane's note ("your draw kept —
location model census") is confirmed: the RNG draw is preserved (determinism
intact), the visual wobble is the documented draw-side skip, and the location
model census proves **no gameplay consumer** for the flint (`noMelee`). No gap,
no port needed. The only future consumer would be a held-item draw layer that
renders the flint sprite at `itemLocation` — at which point the +0/+1 px wobble
becomes visible and the (already-consumed) draw can feed it. **No task seed**
(cosmetic, blocked on the held-item draw layer).

---

## Item 3 — aiStyle 200 BirdDroppings: duplicate vs W4-c10 (keep-one)

Two branches ported the **same** `AI_200_BirdDroppings`
(`Projectile.cs:39985-40017`) + the 1080 `SetDefaults` row
(`Projectile.cs:10078-10086`):

- **d20 (this lane, `e239723b`)**: `projectile_ai_misc.rs::ai_bird_droppings`
  (`:725-761`) + dispatch `projectile.rs:17381-17389` (arm then `kill_step`) +
  row `projectile/data.rs:3611` + `ignoreWater` overlay `:4277`.
- **c10 (`task/W4-c10-style68-waterfowl`, `98a99fb1`)**: a dedicated module
  `projectile_bird_droppings.rs::bird_droppings_step` (245 lines) + dispatch
  `projectile.rs:19778-19806` (the module owns the whole body) + row
  `projectile/data.rs:3982` + `ignoreWater` overlay `:4201`.

**The C# arm** (`:39985-40017`): `velocity.Y += 0.3` capped 16 (`:39987-39991`);
TWO dust-329 iterations, each `NewDust(Center,1,1,329)` + `velocity *= 0.1` then
a `Main.rand.Next(2)` gate — hit: `scale = NextFloat*0.5+0.5`, `velocity +=
base.velocity`, `noGravity`; miss: `scale = NextFloat*0.4+0.4`, `velocity *=
0.5`, `velocity += base.velocity/2` (`:39992-40008`); then the int-truncated
10×10 rect vs every active-and-alive player's rect, XNA `Intersects` → `Kill()`
(`:40009-40016`). Movement is NOT in the arm — `HandleMovement`'s type-1080
branch (`:18921-18928`): after tile collision, `if (velocity != lastVelocity) {
position += velocity; Kill(); }` (the splat).

**Line-by-line fidelity comparison:**

| Aspect | C# | d20 `ai_bird_droppings` | c10 `bird_droppings_step` |
| --- | --- | --- | --- |
| 0.3 gravity / 16 cap | `:39987-39991` | `:731-734` ✓ | `:65-68` ✓ |
| dust RNG census | 2×(Next(2),NextFloat) | `:738-744` if/else, one float each ✓ | `:73-76` flat pair ✓ (equivalent draws) |
| player-intersect kill | int rects, strict edges, `!dead` | `:748-760` reads `pl.width/height` (live hitbox) ✓ | `:82-91` rect seam, dispatch passes `WIDTH/HEIGHT` consts |
| movement / splat | `:18921-18928` integrate-then-kill | `kill_step` (`projectile.rs:10088`) — per-axis probe, integrate-then-kill ✓ | inline `:97-107` — same probe shape ✓ |
| 1080 row | 10×10/200/α255/pen−1/ignoreWater/tileCollide-true/timeLeft-3600 | `data.rs:3611` ✓ | `data.rs:3982` ✓ **byte-identical tuple** |

Both arms are **FAITHFUL** and behaviourally equivalent: the dust census draws
the same `2×(Next(2),NextFloat)` in order (d20's if/else and c10's flat pair both
consume exactly one float per branch — the C# draws one scale float either way);
the kill is the same int-truncated strict-edge rect test; the splat is the same
integrate-then-kill probe. The rows are byte-identical
`(10.0, 10.0, 200, -1, TIME_LEFT_DEFAULT, 0, true, false, false, (false, -2,
false, -1, 255, 1.0))`.

**Two micro-differences (neither a fidelity loss):**

1. **Player rect source.** d20 reads the live `pl.width`/`pl.height`
   (`player.rs:1703/1711`, mount-shrinkable); c10's dispatch passes the
   `WIDTH`/`HEIGHT` constants (`player.rs:26-27`). For the unmounted session
   player they are equal (20×42). d20 is marginally more general (a mounted
   player's shrunk hitbox would intersect correctly); c10 matches the C#
   `Main.player[j].width/height` read only for the stock size. **Edge-case
   only**, no observable difference in the single-player unmounted port.
2. **Module shape.** c10 isolates the drop in its own module (the aiStyle-19
   spear precedent) and owns the movement inline; d20 folds it into
   `projectile_ai_misc.rs` and reuses the pool's shared `kill_step`. Both are
   house-style; c10's is self-contained, d20's shares the movement helper.

**Keep-one decision — KEEP c10's module at merge.** Rationale:

- c10's `projectile_bird_droppings.rs` is the **dedicated, self-contained**
  module with the fuller doc (the `HandleMovement` splat cite, the damage-free
  `Kill()` note, the `ignoreWater`/`wet=false` spawn cite) and **7 hand-derived
  witnesses** (`:165-256`: the row, the gravity+census, the player-rect kill
  incl. the strict edge + dead-player, the blocked-step splat). d20's arm has
  the row + gravity/census + kill witnesses but no dedicated splat witness.
- c10 is the **waterfowl lane** — the 1080 drop is the waterfowl/bird's own
  shot (`NPC.cs:34632`/`:25322`), so the drop belongs with its spawner's lane.
- The d20 arm is a **verbatim duplicate** that adds no fidelity over c10.

**Merge instruction (T3):** when c10 and d20 both land, **keep
`projectile_bird_droppings.rs` + c10's dispatch/row**, and **drop d20's
`ai_bird_droppings` (`projectile_ai_misc.rs:711-761`), its dispatch arm
(`projectile.rs:17381-17389`) and its duplicate row/overlay** — OR repoint d20's
dispatch to call c10's module. The rows are identical so no data conflict; the
only conflict is the two `200 =>` dispatch arms and the two `BIRD_DROPPINGS`
const/row definitions (resolve to c10's). **Do not keep both arms** (a double
`200 =>` match arm is a compile error; a double row is a duplicate-match
warning). The d20 witnesses (`projectile_ai_misc.rs:2268-2330`) should be
dropped with the arm; c10's cover the same ground.

---

## Item 4 — Jumino gift item 5276: row census + use/eat arm (aiStyle 192)

**C# source.** `AI_192_JuminoAnimation` (`Projectile.cs:41501-41638`), the
completion gift (`:41632-41637`):

```
if (this.ai[0] >= num5) {
    int num12 = Item.NewItem(new EntitySource_Gift(this),
        base.Center + new Vector2(0f, 12f), 5276);   // :41634
    Main.item[num12].velocity = new Vector2(0f, 0f);  // :41635
    this.Kill();                                      // :41636
}
```

The gift is item **5276**. Its `SetDefaults` (`Item.cs:41867-41872`):

```
case 5276:
    this.DefaultToVanitypet(994, 345);   // shoot 994 (JunimoPet), buff 345
    this.width = 32; this.height = 32;
    this.SetShopValues(ItemRarityColor.Orange3, Item.buyPrice(0, 1));
    break;
```

`DefaultToVanitypet` (`Item.cs:47936-47950`): `damage 0, useStyle 1, width 16,
height 30, UseSound = SoundID.Item2, useAnimation/useTime 20, rare 3, noMelee,
value sellPrice(0,2), buffType = buffID, shoot = projId`. So 5276 is a **vanity
pet item** (NOT consumable/food): use-style 1, `buffType 345` (JunimoPet),
`shoot 994`, `buffTime 0` (the pet buff is persistent, re-granted by `UpdatePet`).
`ItemID.cs` name `JunimoPetItem`; projectile `ProjectileID.cs` `JunimoPet = 994`.

**Row census — does 5276 exist?** YES. `itemdata.rs:5442`:
`ItemDef { id: 5276, name: "Junimo Pet Item", projectile: Some(994),
use_style: 1, use_time: 20, use_animation: 20, buff_type: 345, buff_time: 0,
consumable: false, potion: false, heal_life: 0, heal_mana: 0, use_sound:
Some(127), width: 32, height: 32, rare: 3, value: 10000, ... }`.

Field-by-field vs C#: `projectile 994` ✓, `buff_type 345` ✓, `use_style 1` ✓,
`use_time/animation 20` ✓, `consumable false` ✓ (vanity pet, not food),
`width/height 32` ✓ (the case override), `rare 3` ✓, `value 10000` =
`buyPrice(0,1)` = 1 gold = 10000 copper ✓ (`Item.cs:49417-49419`).
`use_sound Some(127)` = `SOUND_STYLES[127]` = `Item2` (sound_id 2, style 2,
`soundid.rs:154`) = `SoundID.Item2` (`SoundID.cs:1385`) ✓ — the
`DefaultToVanitypet` UseSound. **The row is FAITHFUL.**

**The gift drain (d20's port).** `ai_jumino` (`projectile_ai_misc.rs:964`)
queues `p.wants_gift_item = Some((5276, center.x, center.y + 12.0))` (`:1060`);
the pool drains it (`projectile.rs::drain_gift_items`, `:15040-15048`); the sim
resolves it (`world_sim/projectiles.rs:612-629`): three `NextFloat` draws (the
`Item.NewItem` velocity triplet, `Main.cs:17775` `UpdateProjectiles` swap) then
`ItemDrop::new(5276, 1, 0, x - SIZE/2, y - SIZE/2, 0, 0, 0)` — the explicit
velocity zeroing (`:41635`) overrides the rolled triplet. **The gift drop is
FAITHFUL** (the item spawns at `Center + (0,12)`, velocity 0, prefix 0).

**Use/eat arm census — what happens when the player USES 5276?** This is where
the real gap lives, and it is **cross-lane** (the item-use path + pet-equip):

1. **The use-path drink gate refuses it.** `world_sim/editing.rs:2313-2350`:
   the food arm needs `consumable && buff_type ∈ {26,206,207}` (5276 is not);
   `potions::use_potion_item` (`:2348`) needs `drinkable(def)` (`potions.rs:358`:
   `consumable && (potion || heal>0 || buff_type>0 && use_style ∈ {2,9})`) —
   5276 is `consumable: false`, `use_style 1`, so **`drinkable` is false**. The
   pet item is NOT a drink/food. Correct — the C# pet item rides the **shoot
   path**, not the consumable path.
2. **The shoot path grants the buff but the pet spawn arm is missing.** The
   generic shoot (`editing.rs:2737-2799`) grants `buff_type 345` at
   `:2773-2774` (`AddBuff(345, buff_time=0)`) and calls `free_up_pets_and_minions`
   (`:2787`, 994 ∈ `PROJ_PET`, `data.rs:4595`). BUT the C# **pet-buff shoot arm**
   (`Player.cs:48561-48583`: kill the owner's same-kind pets, then
   `NewProjectile(pointPosition, num4, num5, projToShoot, 0, 0f, i)` — damage 0,
   kb 0) has **no port** (grep `48561|48582` in `editing.rs`: 0 hits). The
   generic `fire_non_ammo_shot` tail (`:4093+`) would fire 994 as a *generic
   shot* with the item's damage/kb, not the pet arm's damage-0/kb-0 spawn.
3. **The buff is granted with time 0 → it expires immediately.** `AddBuff(345,
   0)` (`buffs.rs:192`) stores `buff_time = 0`; `update_buffs_tick` (`:388`)
   skips `buff_time <= 0` slots, and `remove_expired_buffs` (`player.rs:5728`)
   deletes them. The C# avoids this via **`UpdatePet`** (`Player.cs:17298-17309`,
   called `:25573`): every tick, if `miscEquips[0].buffType >= 1 && stack >= 1`
   and the buff is a vanity/light pet and `!hideMisc[0]` and `FindBuffIndex(num)
   == -1`, it re-grants `AddBuff(num, 3600)` + plays the UseSound. **`UpdatePet`
   has no port** (grep `UpdatePet|update_pet` in crates: 0 hits). So even if the
   buff were granted, nothing re-grants it → the pet never persists.
4. **The pet projectile 994 has no `proj_data` row.** Grep `994 =>|JUNIMO_PET =>`
   in `projectile/data.rs`: 0 hits (994 appears only in the `PROJ_PET` id list
   `:4595`). The C# 994 row (`Projectile.cs:9436-9445`): netImportant, 20×20,
   aiStyle 26, friendly, penetrate −1, `timeLeft *= 5`. The pet AI
   (`AI_026`, `:58102`, the 994 keep-alive `:58845-58855` reads
   `petFlagJunimoPet`) is partially ported (`projectile_ai026_c.rs` lists 994 in
   its doc `:25/:955/:968` but `AI_026_C_KINDS = [SLIME_MINION]` only, `:120` —
   **994 is NOT in any ported aiStyle-26 slice**). The buff-effect spawn
   (`buff_effects_accessory.rs:86` `(345, 994)` in `PET_BUFF_ARMS`, dispatched
   `:308-310` → `spawn_pet_if_needed`) DOES spawn 994 via `pool.spawn` when buff
   345 is active — but `spawn_pet_if_needed` (`:121-151`) documents "the port's
   data-row table has no pet rows yet (`proj_data` returns None), so the plain
   spawn carries the slot; the pet AI/box lands with the pet-AI task".

**Verdict — row FAITHFUL, gift drop FAITHFUL; the use/eat arm is a cross-lane
real gap.** The d20 lane's responsibility (the 192 AI + the gift drop) is
complete and correct. The pet item's *use* → *persistent pet* chain is broken
across three unported pieces, all outside the proj-191-200 lane:

- **T4a (item-use lane):** the pet-buff shoot arm `Player.cs:48561-48583`
  (kill same-kind + damage-0/kb-0 spawn) in `editing.rs::fire_non_ammo_shot`.
- **T4b (pet-equip lane):** `Player.UpdatePet`/`UpdatePetLight`
  (`Player.cs:17298-17340`) — the per-tick `miscEquips[0..1]` buff re-grant
  (3600) + UseSound, the persistence mechanism for ALL vanity/light pets.
- **T4c (pet-AI lane):** the 994 `proj_data` row (`Projectile.cs:9436-9445`) +
  the aiStyle-26 pet slice covering 994 (the keep-alive `:58845-58855`).

These are **not d20 ports** — they belong to the item-use, pet-equip and pet-AI
lanes respectively. The gift item 5276 itself is correctly rowed and dropped;
picking it up and using it currently grants a buff that expires the same tick
(no `UpdatePet`) and spawns no persistent pet (no 994 row/AI). **Recorded as
seeds, not ported here** (cross-lane; porting would touch `editing.rs` and the
pet-equip hot files outside this lane's scope).

---

## Item 5 — 191 TrueNightsEdge vs 190 NightsEdge: terra-chain progression

**The chain (item → projectile → aiStyle).** Verified against `Item.cs`
SetDefaults + `Projectile.cs` SetDefaults + `ProjectileID.cs`/`ItemID.cs`:

| Item | id | `shoot` | C# use-path arm | spawns (kind, velocity, damage, ai0/ai1/ai2) |
| --- | --- | --- | --- | --- |
| Nights Edge | 273 (`Item.cs:4578`) | 972 | `Player.cs:48870-48877` | **972** slash `(dir,0)` dmg `dir·gravDir / itemAnimMax / scale`; **972** beam `(num4,num5)` dmg `dir·gravDir·0.1 / 30 / scale` |
| True Nights Edge | 675 (`Item.cs:9299`) | 973 | `Player.cs:48914-48921` | **972** slash `(dir,0)` dmg `dir·gravDir / itemAnimMax / scale`; **973** beam `(num4,num5)` **dmg/2** `dir·gravDir / 32 / scale` |
| True Excalibur | 674 (`Item.cs:9280`) | 983 | `Player.cs:48922-48929` | **983** slash `(dir,0)` dmg `dir·gravDir / itemAnimMax / scale`; **982** ghost `(dir,0)` **dmg 0** `dir·gravDir / itemAnimMax / scale` |
| Terra Blade | 757 (`Item.cs:10212`) | 985 | `Player.cs:48930-48937` | **984** slash `(dir,0)` dmg `dir·gravDir / itemAnimMax / scale`; **985** beam `(num4,num5)·5` dmg `dir·gravDir / 18 / scale` |

(`num4,num5` = the aim normalized to `shootSpeed`, `Player.cs:47277-47307`;
`scale` = `GetAdjustedItemScale`, `:42566-42574` = `item.scale × 1.1` iff
melee+glove; `itemAnimMax` = the prefixed `itemAnimationMax`.)

**Projectile rows + aiStyle:**

| proj | id | aiStyle | C# SetDefaults | d19 row | d20 row |
| --- | --- | --- | --- | --- | --- |
| NightsEdge | 972 | 190 | `:9298-9314` | ✓ `data.rs:3700` | **absent** |
| TrueNightsEdge | 973 | 191 | `:9315-9329` | — | ✓ `data.rs:3656` |
| Excalibur | 982 | 190 | `:9354-9370` | ✓ `:3707` | **absent** |
| TrueExcalibur | 983 | 190 | `:9371-9387` | ✓ `:3713` | **absent** |
| TerraBlade2 (slash) | 984 | 190 | `:9388-9404` | ✓ `:3719` | **absent** |
| TerraBlade2Shot (beam) | 985 | 191 | `:9405-9421` | — | ✓ `data.rs:3662` |
| HorsemansBlade | 997 | 190 | `:9466-9482` | ✓ `:3707` | absent |
| DeadCellsFlintSlash | 1043 | 190 | `:9858-9874` | ✓ `:3725` | ✓ `data.rs:3651` (dup) |
| ChlorophyteClaymoreBlade | 1126 | 190 | `:10543-10558` | ✓ `:3734` | absent |

**AI-arm fidelity — BOTH faithful.**

- **190 `AI_190_NightsEdge`** (`Projectile.cs:41972-42227`): d19's
  `projectile_ai_sword_styles.rs::ai_nights_edge` (`:1035`) ports the spine
  verbatim — the first-tick 984 Item60 cue (`:41978`, port `:1048`) + 1126
  vel-y latch (`:41986`, port `:1051`); the age/progress/rotation
  (`:41991-41994`, port `:1057-1061`); the per-kind scale pair
  (`:41997-42016`, port `:1065-1076`); the centre pin + progress scale
  (`:42017-42018`, port `:1085-1086`); the 972/1043 thrust extension
  (`:42019-42049`, port `:1089+`); the per-kind dust gates; the 1126 spore
  machine (`:42078-42182`) queuing 1128 clouds; the `scale *= ai[2]` + num≥1
  kill (`:42222-42226`). Dispatch wired (`projectile.rs:17423` on d19).
- **191 `AI_191_TrueNightsEdge`** (`Projectile.cs:41640-41808`): d20's
  `projectile_ai_misc.rs::ai_true_nights_edge` (`:1533`) ports it verbatim —
  the envelope constants + 985 swap (`:41642-41653`, port `:1539-1551`); the
  973 first-tick Item8 cue (`:41654-41657`, port `:1553-1554`); the age +
  fast-forwards (`:41658-41666`, port `:1556-1563`); the opacity envelope
  (`:41667`, port `:1565-1567`); the num4 kill (`:41668-41673`, port
  `:1569-1575`); the 973 spin/LOS/centre/dust (`:41678-41725`, port
  `:1583-1634`); the 985 ownerHitCheck window + damage-zero + drag ladder +
  retract (`:41730-41807`, port `:1641-1703`); the 985 blocked-move retract
  (`:18104-18110`, `terra_blade_move` `:1710`). Dispatch wired
  (`projectile.rs:17524`). Witnesses pin the rows, the cue, the num4 kill, the
  ownerHitCheck window, the damage-zero, the retract and the dispatch
  (`projectile_ai_misc.rs:2964-3089`).

**Both arms read `ai[1]` as the lifetime envelope** (190: `num = localAI[0] /
ai[1]`, `:41992`; 191: `num3 = ai[1] + 50`, `fromValue = localAI[0] / ai[1]`,
`:41644/:41675`) and `ai[2]` as the scale (`:42222` / `:41710/:41737`). The
witnesses spawn with `ai[1] = 30, ai[2] = 1` (`:2988/:3070`) — the **correct
use-path payload**.

**THE REAL GAP — the use-path spawn arms are missing on EVERY branch.** The
four C# arms (`Player.cs:48870/:48914/:48922/:48930`) that spawn the paired
slash+beam with the `ai0/ai1/ai2` payload have **no port** (all-branch grep
`48870|48914|48930|kind == 273|kind == 675|kind == 757` in `editing.rs`: 0 hits).
The swords fall through to the **generic `Item.shoot` tail**
(`editing.rs::fire_non_ammo_shot`, `:4093-4225`), which spawns ONE projectile
(the item's `shoot`) at the aim velocity with **`ai0 = ai1 = 0`** (`:4113-4114`)
and `ai2 = 0` (the spawn seam carries ai0/ai1 only, `projectile.rs:16034`).

Consequences (behavioural, not cosmetic):

1. **The envelope collapses.** With `ai[1] = 0`, the 190/191 arms divide by zero
   (`localAI[0] / ai[1]` → ∞/NaN). The 191 num4 kill (`localAI[0] >= ai[1]+65 =
   65`) fires after 65 ticks regardless; the opacity/scale remaps read NaN. The
   190 `num >= 1` kill (`:42223`) fires immediately (∞ ≥ 1). The slash/beam
   choreography never plays as designed.
2. **The paired slash is never spawned.** TrueNightsEdge should spawn a 972
   slash + a 973 beam (half damage); the port spawns only a 973. Terra Blade
   should spawn a 984 slash + a 985 beam (×5 velocity); the port spawns only a
   985. TrueExcalibur should spawn 983 + a 982 ghost (damage 0); the port spawns
   only 983. NightsEdge should spawn 972 slash + 972 beam; the port spawns one
   972.
3. **The 972/984/982/983 rows are absent on d20** (they live only on d19), so
   even a naive use-path port on d20 could not spawn the slash kinds.

**Verdict — AI arms FAITHFUL (both 190 and 191 shoot the correct child types
*when given the correct payload*); the use-path spawner is a cross-lane real
gap.** The d20 lane's 191 arm is correct and complete; the d19 lane's 190 arm is
correct and complete. The missing piece is the **item-use lane's** per-item
spawn arms (`Player.cs:48870-48937`), which must:

- spawn the paired slash+beam with `ai0 = dir·gravDir` (slash) / `dir·gravDir`
  or `·0.1` (beam), `ai1 = itemAnimMax` (slash) / `30/32/18` (beam), `ai2 =
  GetAdjustedItemScale`;
- spawn at `MountedCenter` (slash, velocity `(dir,0)`) and the aim velocity
  (beam, `(num4,num5)`, Terra Blade ×5);
- halve the TrueNightsEdge beam damage, zero the TrueExcalibur ghost damage.

**T5 (item-use lane, depends on d19's rows landing):** port the four arms into
`editing.rs::fire_non_ammo_shot` ahead of the generic tail, reading the prefixed
`use_animation`/`scale` and `p.melee_speed`-adjusted `itemAnimationMax`. The
972/984/982/983 rows come from d19 (do NOT duplicate them on d20). The
`GetAdjustedItemScale` helper already exists (`melee_swing.rs:132`,
`adjusted_item_scale`); the glove flag is unported (`accessory_combat.rs:18`) so
it folds to `item.scale` (×1.0) until the glove lands. **Not a d20 port** —
it touches the item-use hot file and depends on d19's data; recorded as a seed.

**Progression verify (the lane's question): do both item shoots land correctly?**
At the AI level YES — 973 (191) and 985 (191) on d20, 972/982/983/984/997/1043/
1126 (190) on d19, all faithful, all reading the same `ai[1]`/`ai[2]` envelope.
At the use-path level NO — neither branch spawns the paired slash+beam with the
payload, so the terra-chain progression (NightsEdge → TrueNightsEdge →
TerraBlade) is **AI-complete but use-path-inert** until T5 lands. The 1043 flint
slash (d20's 198 arm spawns it via `wants_dart_spawns`, `projectile.rs:17485`)
is the ONE chain member whose spawn IS wired — but its 190 AI arm lives on d19,
so on d20 alone the 1043 spawns and flies inert (no 190 dispatch); on d19 alone
the 1043 row exists but the 198 flint arm that spawns it does not. **The flint
slash is the merge canary for the 190/198 cross-lane dependency.**

---

## Item 6 — tracked-loops JimsDrone / FlintCharge: d80 owns (contract doc)

**Ownership.** The tracked-sound layer (S1-E-23, `tracked_sound.rs` +
`audio.rs::play_tracked*`) and its per-entity wiring are **d80's lane**
(`task/W4-d80-tracked-sound-layer`, tip `075b954d`). d20 correctly skips the
drone/flint sound call-sites and documents them as d80's. This item reads d80's
claims and documents the contract d20's arms must honour when d80 wires them.

**The two call-sites (C#):**

1. **JimsDrone loop** — `AI_195_JimsDrone` (`Projectile.cs:40983-40997`):
   ```
   localAI[1] = Lerp(localAI[1], num2, 0.2f);                    // :40983 throttle
   num3 = Remap(localAI[0],0,5,0,1) * Remap(localAI[0],5,15,1,0); // :40984 steer envelope
   num4 = Clamp(Max(Remap(localAI[1],0,100,0,25), num3*12),0,100); // :40985 volume
   activeSound = GetActiveSound(SlotId.FromFloat(localAI[2]));    // :40986
   if (activeSound == null && num4 != 0f)                         // :40987 ensure
       localAI[2] = PlayTrackedLoopedSound(SoundID.JimsDrone, Center,
           new ProjectileAudioTracker(this).IsActiveAndInGame).ToFloat(); // :40989
   if (activeSound != null) {                                     // :40992 follow
       activeSound.Volume = num4;                                 // :40994
       activeSound.Position = Center;                             // :40995
       activeSound.Pitch = Clamp(Remap(localAI[1],0,100,-1,1)+num3,-1,1); // :40996
   }
   ```
   This is a **`PlayTrackedLoopedSound`** with a live volume/pitch/position
   follow and a `ProjectileAudioTracker` stop condition. `SoundID.JimsDrone` =
   `CreateTrackable("Drone").WithVolume(0.1f)` (`SoundID.cs:1706`), port row
   `soundid.rs:474` (sound_id 42, style 271, vol 0.1, trackable "Drone"). The
   handle lives in `localAI[2]`. The RC car (`AI_205`, `:38253-38267`) is the
   identical shape with `SoundID.RCCar` (`soundid.rs:475`).

2. **FlintCharge cue** — `AI_198_Flint` (`Projectile.cs:52303-52306`):
   ```
   if (this.ai[1] == 30f)
       SoundEngine.PlaySound(SoundID.DeadCellsFlintCharge, player.Center); // :52305
   ```
   This is a **one-shot `PlaySound(LegacySoundStyle, Vector2)`**
   (`SoundEngine.cs:88-99`), NOT a tracked loop — it fires once at the 30-tick
   charge mark and plays out. `SoundID.DeadCellsFlintCharge` =
   `CreateTrackable("deadcells_flint_charge").WithVolume(0.5f)`
   (`SoundID.cs:1733`), port row `soundid.rs:501` (sound_id 42, style 296,
   vol 0.5). Despite the `CreateTrackable` asset family, the **call-site is the
   legacy one-shot path** (`SoundEngine.PlaySound`, not `PlayTrackedSound`), so
   it rides the `wants_sound` case-2 façade, not the tracked pool. (The
   `DeadCellsFlintRelease`/`Wave` cues, `soundid.rs:502-503`, are the release
   one-shots — also legacy.)

**d80's claims (read from the branch):**

- The census (`docs/research/64-tracked-sound-census.md`, d80 tip) lists the
  drone loop as **DEFERRED**: "Jim's Drone loop | `Projectile.cs:40983-40997` |
  the 1020 drone AI has no port arm (id-table row only)" (`:78`), and the RC car
  likewise (`:77`). **This claim is now STALE** — d20 landed `ai_jims_drone`
  (`projectile_ai_misc.rs:1092`) after d80's base (`a725acb0`). d80's branch has
  no `ai_jims_drone`/`ai_flint` (grep: 0 hits), so d80 could not wire them.
- d80's `ProjTrackedCue` enum (`projectile.rs:477-489` on d80) has ONLY
  `BombFuseEnsure`/`BombFuseStop` — **no drone/flint variants**. The drone loop
  was deferred precisely because its owning AI was unported at d80's base.
- The ANALYSIS contract (d80 `docs/ANALYSIS.md:2511-2545`): the tracked entity
  loops queue intents (`NpcSoundCue`/`ProjTrackedCue`), the sim drain
  (`apply_proj_tracked_cue`) applies them and writes the `SlotId` float back into
  `local_ai[2]`; **RNG contract: NO tracked play draws `Main.rand`** — every
  `PlayTracked*` consumes exactly two AUDIO-ONLY static streams (the variation
  pick `LegacySoundStyle.Random` drawn only when `Variations != 1`, and the
  always-drawn `GetRandomPitch` NextFloat `SoundStyle._random`), port stand-ins
  `Audio::style_random`/`sound_style_random` (seeded 1). The looped-play
  conditions fold to shared `Arc<AtomicBool>` (the pool cannot hand a live
  entity ref to a `Send` closure); `ProjectileAudioTracker.IsActiveAndInGame`
  (`ProjectileAudioTracker.cs:15-27`: `!gameMenu && active && type ==
  expectedType`) folds to the atomic the slot's death flips false.

**The contract d20's arms must honour (when d80 wires them):**

- **JimsDrone (T6a):** d20's `ai_jims_drone` already computes the throttle
  `local_ai[1]` lerp (`:1167`) and keeps `local_ai[2] = 0` (`:1080-1081` doc,
  `:1168-1169` "the sound volume/pitch envelope feeds only the skipped
  tracked-loop sound... localAI[2] stays 0"). To wire: d80 adds a
  `ProjTrackedCue::DroneEnsure`/`DroneFollow` (or a per-tick follow intent),
  the drain plays `TrackedStyle::by_name("JimsDrone")` looped with the
  shared-atomic condition, writes the handle to `local_ai[2]`, and sets
  `Volume = num4`, `Position = Center`, `Pitch = clamp(remap(local_ai[1],0,100,
  -1,1)+num3,-1,1)` each tick. **The num3/num4 envelope math is currently
  SKIPPED in d20** (`:1168-1169`) — d80 must re-derive it from `local_ai[0]`
  (steer age) and `local_ai[1]` (throttle), both of which d20 DOES maintain.
  ZERO gameplay RNG draws (the census contract) — the drone's dust draws
  (`:1083-1087`) stay on the gameplay stream, the sound draws ride the audio
  statics.
- **FlintCharge (T6b):** d20's `ai_flint` skips the cue (`:1384-1385` doc,
  `:1474-1475` "The 30-tick charge cue is a tracked sound — skipped (the state
  still gates the spike wave above)"). **Correction for d80:** the cue is a
  legacy ONE-SHOT (`PlaySound(LegacySoundStyle, Vector2)`, `SoundEngine.cs:88-99`,
  `:52305`), not a tracked loop. It is a **case-42 custom-trackable one-shot**,
  NOT the id-2 item family — so it does NOT ride `play_proj_sound`
  (`audio.rs:1969`, which refuses `id != 2`, `:1977`). It rides
  `audio.rs::play_use_sound(style_index, ...)` (`:1053`), whose **case-42 arm**
  (`:1080-1089`) plays the trackable asset (`deadcells_flint_charge`) at
  `style.volume` with only the caller's pitch offset clamped — **NO gameplay
  `Main.rand` draw** (the id-2 case-2 pitch roll at `:1092-1093` is skipped for
  id 42; the single `GetRandomPitch` NextFloat at `:1064` rides the audio-only
  `sound_style_random` static). d80 wires it by queueing the cue at the
  `ai[1] == 30` mark and draining through `play_use_sound` with the
  `DeadCellsFlintCharge` row's `SOUND_STYLES` index (`soundid.rs:501`, style 296,
  sound_id 42) — NO tracked pool, NO `local_ai[2]` handle, NO `wants_sound`
  id-2 tuple (that façade is id-2-only). The existing `wants_sound` seam
  (`projectile.rs:716`) would need an id-42-aware drain arm, or a dedicated
  one-shot intent. **T6b note:** d20's doc calling it "a tracked sound" is
  imprecise — it is a trackable-*asset* played via the legacy one-shot case-42
  path; the gameplay RNG contract is ZERO draws (unlike the id-2 case-2 cues).

**Verdict — d80 owns both; d20's skips are correct and the contract is
documented.** The drone loop is a genuine tracked-loop deferred site (d80's
census claim is stale only in that d20 has now landed the owning AI — the
*reason* for deferral is resolved, the wiring is still d80's). The flint charge
is a legacy case-42 one-shot d80 wires through `play_use_sound` (d20's doc
calling it "a tracked sound" is imprecise — it is a trackable-*asset* played via
the legacy one-shot path, ZERO gameplay RNG draws; **T6b note**). Neither is a
d20 port. **Task seeds T6a/T6b** below, owned by d80.
