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

---

## Task seeds (prioritized, with owning lane + dependencies)

| Seed | Size | Lane | Task | C# anchor | Depends on |
| --- | --- | --- | --- | --- | --- |
| **T5** | M | item-use (`editing.rs`) | Port the terra-chain use-path spawn arms: NightsEdge 273 / TrueNightsEdge 675 / TrueExcalibur 674 / TerraBlade 757 — paired slash+beam with `ai0=dir·gravDir`, `ai1=itemAnimMax`(slash)/`30/32/18`(beam), `ai2=GetAdjustedItemScale`; TrueNightsEdge beam dmg/2, TrueExcalibur ghost dmg 0, TerraBlade beam vel×5; spawn at MountedCenter | `Player.cs:48870-48937` | d19's 972/982/983/984 rows landing; `adjusted_item_scale` (`melee_swing.rs:132`) exists |
| **T4b** | M | pet-equip | Port `UpdatePet`/`UpdatePetLight` — per-tick `miscEquips[0..1].buffType` re-grant `AddBuff(num,3600)` + UseSound when `FindBuffIndex==-1 && !hideMisc`, the vanity/light-pet persistence mechanism (fixes ALL pet items, not just 5276) | `Player.cs:17298-17340` (call `:25573-25574`) | `is_vanity_pet`/`is_light_pet` (`buffs.rs:106/102`) exist |
| **T4a** | S | item-use (`editing.rs`) | Port the pet-buff shoot arm: kill owner's same-kind pets, then `NewProjectile(point, aim, projToShoot, 0, 0f, owner)` (damage 0, kb 0) for `projPet[shoot] && !summon` | `Player.cs:48561-48583` | `PROJ_PET` (`data.rs:4588`) exists |
| **T4c** | M | pet-AI | Add the 994 JunimoPet `proj_data` row (20×20, aiStyle 26, friendly, pen −1, netImportant, timeLeft×5) + extend the aiStyle-26 pet slice to cover 994 (keep-alive `petFlagJunimoPet`) | `Projectile.cs:9436-9445`, `:58845-58855` | T4b (the buff must persist for the keep-alive to matter) |
| **T6a** | S | d80 tracked-sound | Wire the JimsDrone loop: `ProjTrackedCue::Drone*` ensure+follow, re-derive num3/num4 from `local_ai[0]/[1]` (d20 maintains both), shared-atomic condition, handle in `local_ai[2]`; ZERO gameplay draws | `Projectile.cs:40983-40997` | d20's `ai_jims_drone` (landed); d80's `ProjTrackedCue` drain |
| **T6b** | S | d80 tracked-sound | Wire the FlintCharge case-42 one-shot at `ai[1]==30` via `play_use_sound` (style 296), id-42-aware drain (NOT the id-2 `wants_sound` façade); ZERO gameplay draws | `Projectile.cs:52303-52306` | d20's `ai_flint` (landed); `play_use_sound` case-42 (`audio.rs:1080`) exists |
| **T1** | M | camera (`world_sim/camera.rs`) | Port `GetObjectTrackingCameraPan`'s three sources (spectate/drone/golf, priority order) + `DroneCameraTracker` + `remoteVisionForDrone` (visor 5452 equip grant); the drone Track call-site is already in d20's arm (skipped) | `Main.cs:62722-62746`, `DroneCameraTracker.cs:1-34`, `Player.cs:14213/15461/18712` | golf/spectate camera state; d69 render targets are unrelated |
| **T3** | S | merge (c10+d20) | Keep-one: keep c10's `projectile_bird_droppings.rs` + dispatch/row; drop d20's `ai_bird_droppings` + `200 =>` arm + duplicate row/overlay/witnesses (or repoint d20's dispatch to c10's module) | `Projectile.cs:39985-40017` | c10 + d20 both landing |

**Priority rationale.** T5 is the highest-impact behavioural gap (the entire
terra-chain — a core progression weapon line — fires inert single shots with a
collapsed envelope). T4b is the highest-leverage pet fix (one arm repairs every
vanity/light pet's persistence). T4a/T4c complete the 5276 gift chain. T6a/T6b
are small d80 follow-ups now unblocked by d20's AI arms. T1 is camera-lane scope
(the drone view). T3 is a merge-time dedupe, no new code.

**Cross-lane dependency canary.** The 1043 DeadCellsFlintSlash is the merge
canary: d20's 198 arm spawns it (`projectile.rs:17485`) but its 190 AI arm is
d19's; d19 has the 1043 row + 190 arm but not the 198 spawner. **Both must land
for the flint slash to fly correctly** — a lossy merge that drops either side
leaves the slash inert (spawned but no AI) or unspawned (AI but no source). The
`anchors.txt` canary should pin both the 198 spawn and the 190 dispatch.

---

## Verification performed

Code-only DOC-SWEEP (no cargo/tests/graphify per the lane constraint and
`MODULE-TEMPLATE.md`). Static cross-reading of:

- C#: `Projectile.cs` (AI_190/191/192/195/198/199/200, SetDefaults rows,
  HandleMovement tails), `Player.cs` (ItemCheck_Shoot use-path arms, UpdatePet,
  AddBuff, GetAdjustedItemScale, itemLocation readers), `Item.cs` (5276/273/675/
  674/757/5462 SetDefaults, DefaultToVanitypet, buyPrice), `Main.cs`
  (GetObjectTrackingCameraPan), `DroneCameraTracker.cs`, `SoundEngine.cs`,
  `SoundID.cs`, `BuffID.cs`, `ProjectileID.cs`, `ItemID.cs`.
- Rust (this branch): `projectile_ai_misc.rs`, `projectile.rs`,
  `projectile/data.rs`, `world_sim/projectiles.rs`, `world_sim/editing.rs`,
  `world_sim/camera.rs`, `world_sim/player.rs`, `player.rs`, `buffs.rs`,
  `buff_effects_accessory.rs`, `buff_effects_misc.rs`, `potions.rs`,
  `itemdata.rs`, `item.rs`, `audio.rs`, `soundid.rs`, `tracked_sound.rs`,
  `melee_swing.rs`, `projectile_ai_golf.rs`, `projectile_ai026_*.rs`,
  `registry.rs`, `prefix.rs`.
- Sibling lanes (git show, read-only): d19 `projectile_ai_sword_styles.rs` +
  `projectile/data.rs` + dispatch; d69 `tr-render/targets.rs`; d80
  `tracked_sound.rs` + `projectile.rs` + `world_sim/projectiles.rs` +
  `docs/research/64-tracked-sound-census.md` + `docs/ANALYSIS.md`; c10
  `projectile_bird_droppings.rs` + `projectile/data.rs` + dispatch.

All six d20 AI arms re-verify FAITHFUL against their C# sources. The six skips
resolve as tabulated above. No repo code was modified (research-only, per
`MODULE-TEMPLATE.md`); the real gaps are cross-lane and recorded as seeds.

## Deviations & risks

- **d20 doc imprecision (cosmetic):** `ai_flint`'s doc (`:1384`) calls the
  FlintCharge cue "a tracked sound"; it is a legacy case-42 one-shot (T6b note).
  `ai_jims_drone`'s doc (`:1078-1081`) correctly calls the drone loop a tracked
  loop. No behaviour impact (both skipped).
- **d80 census staleness (doc):** the 64-census deferred table claims the drone
  AI is unported; d20 has since landed it. The deferral *reason* is resolved;
  the wiring remains d80's (T6a).
- **T5 envelope-collapse risk (behavioural, determinism-adjacent):** until T5
  lands, the terra-chain swords spawn `ai[1]=0` projectiles whose 190/191 arms
  divide by zero — the NaN/∞ propagates into rotation/scale/opacity. This is
  inert-but-not-crashing (the num4/num≥1 kills still fire), but it is a real
  behavioural deviation from the C# paired slash+beam. NOT a d20 regression
  (the use-path was never ported on any branch).
- **T3 double-arm risk (compile):** if c10 and d20 both land without the
  keep-one resolution, the two `200 =>` dispatch arms are a duplicate-match
  compile error and the two `BIRD_DROPPINGS` rows a duplicate definition. The
  merge MUST resolve to one (T3).

---

# SEAL — the FINAL band-191-200 census + the sword-family declaration (registry 109)

Code-only seal (no cargo/tests/rustfmt/graphify per the lane constraint).
Every line number below is pinned at the seal tip's HEAD
(`9bc29bc7f`, the T4b commit) unless stated; d19 reads are at
`origin/task/W4-d19-proj-ai-181-190` tip `61f7638a5`. Witnesses WRITTEN NOT RUN.

## S1 — the git log census: what landed on band 191-200

`git log origin/task/W4-d20-proj-ai-191-200..HEAD` at seal time = 4 commits
ahead of the published tip `243f81672` (the tip d19 §7a monitored):

| commit | scope | files |
| --- | --- | --- |
| `7b4173846` | T5 rows: the four d19 aiStyle-190 slash rows copied VERBATIM (972/982+997/983/984) + the ignoreWater/melee overlay arms + consts | `projectile/data.rs` (+65/−1) |
| `23199067d` | T5 use-path: the four paired slash+beam arms (`Player.cs:48870/48914/48922/48930`) into `fire_non_ammo_shot` before the generic tail; 6 witnesses | `world_sim/editing.rs` (+313), `anchors.txt` (+1) |
| `23c075d7a` | T5-canary: the dart-spawn drain carries ai2 (11th tuple slot); the 1043 release slash's scale 1 survives; 1 merge-canary witness | `projectile.rs`, `projectile_ai_darts.rs`, `projectile_ai_misc.rs` (+81/−10) |
| `9bc29bc7f` | T4b pet persistence: `UpdatePet`/`UpdatePetLight` (`Player.cs:17298-17340`) as `update_pet_persistence`, wired after UpdateBuffs; 5 witnesses | `world_sim/player.rs`, `anchors.txt` |

**T5 terra-use-arms: LANDED locally (`23199067d`), NOT YET on origin at the
d19 §7a monitor read.** d19's "d20c-T5 NOT-landed" verdict was correct at
`243f81672` and is now stale by these four commits — this seal + push is the
receipt d19 §7a asked for. The earlier band arms (191/192/195/196/198/199/200)
landed pre-`243f81672` (`2b2b71a71`/`543bcbdf9`/`6e44ca3bd`/`f4e660a20`/
`75286d0e4`/`fe576990c`/`e239723b3`); the doc-sweep six items are
`a6211d389`…`243f81672` (items 1-6 + consolidation above).

## S2 — the d19 sword-seal contract (26g §7/§7a/§8) read against THIS tip

- **The envelope witness gate** (`w4_d19_the_terra_chain_payload_feeds_the_190_envelope`,
  d19 `projectile_ai_sword_styles.rs:1625` at `6b56e27c`, re-read intact at
  `61f7638a5`): the merged pool must spawn 972 row-gated with
  `ai = [dir·gravDir, itemAnimMax, scale, 0]`, finite `num = localAI[0]/ai[1]`,
  scale `(1 + num·0.2)·ai2`, progress-1 kill. d20's T5 feeds exactly this:
  `editing.rs:4093` gate → `:4150` per-arm table → `spawn_sword` closure
  (`:4191-4213` at HEAD) writes ai0/ai1 through `spawn_with_data_ai` + `set_ai2(scale)`
  — the ai2 third slot rides the same seam d19's witness drives.
- **slashes-ONLY-d19 HARD gate — DEVIATION, adjudicated here.** `7b4173846`
  copies d19's four slash rows onto d20, against the seed's "do NOT duplicate"
  instruction. Rationale (recorded in the commit + the data.rs comment): the
  row is the spawn GATE (`spawn_with_data_ai` declines rowless kinds), so
  without the copy T5's paired slashes never spawn on THIS base and the whole
  port is untestable pre-merge. The copies are **value-identical** to d19
  (`data.rs:3660-3682` at HEAD vs d19 `:3700-3723`, tuples re-read whole this
  seal): 972 pen2 / 982|997 pen3 / 983 pen6 / 984 pen3, all
  `16x16, 190, TIME_LEFT_DEFAULT, tileCollide false, LOCAL −1, alpha 0`.
  **Merge resolution (published for the merger): keep-ONE — keep d19's block,
  drop d20's copy** (consts `data.rs:4620-4636`, row block `:3652-3682`,
  ignoreWater arm `:4321-4325`, melee arm `:4366-4371` at HEAD; d19's overlay
  arms are supersets that already carry every kind — dropping the d20 arms
  loses nothing). This extends the 1043 keep-one pattern to the whole slash
  set; the §8 step-4 "1043 dedupe" becomes a five-kind dedupe.
- **1043 dup value-identical keep-one:** re-confirmed at HEAD — d20
  `data.rs:3651` vs d19 `:3725-3729`, both
  `(16.0, 16.0, 190, 2, TIME_LEFT_DEFAULT, 0, false, false, false, (true, -1, false, -1, 0, 1.0))`.
  Keep either (d19's rides the band set). The merge canary stands: 1043 needs
  d20's 198 spawner (`projectile.rs:17467` arm + drain `:17488-17493`) AND
  d19's 190 dispatch; the ai2 drain fix (`23c075d7a`) is REQUIRED for the
  slash to fly with scale 1 — a merge that keeps the old `_ai2` drain
  collapses `scale *= ai[2]` to 0 (`:42222`).
- **§8 step 5 (T5 LAST against merged rows):** satisfied by construction —
  T5 spawns through the row-gated seam and carries no 190 AI (d19's stays
  sole owner); after the keep-one dedupe the merged rows are d19's and the
  §7 envelope witness must pass against them (written not run).

## S3 — terra-use-arms: all four branches ported (the ledger gap CLOSED on d20)

The ledger's major-fidelity bug ("use-path spawn arms missing on EVERY
branch", doc-sweep item 5) is closed on this lane by `23199067d`. The four
arms at HEAD `world_sim/editing.rs` (`fire_non_ammo_shot`, gate `:4093`,
table `:4150`, spawns `:4191-4230`), each vs its C# arm re-read:

| item | C# | slash | partner | pinned |
| --- | --- | --- | --- | --- |
| 273 NightsEdge | `:48870-48877` | 972 vel(dir,0) ai1=animMax | 972 beam vel=aim ai0=dir×0.1 ai1=30 | `w4_d20_t5_nights_edge_spawns_the_paired_972` (`:18930`) |
| 675 TrueNightsEdge | `:48914-48921` | 972 (LITERAL, `:48917`) | 973 beam dmg/2 ai1=32 | `w4_d20_t5_true_nights_edge_halves_the_beam` (`:18958`) |
| 674 TrueExcalibur | `:48922-48929` | 983 | 982 ghost dmg 0 vel(dir,0) ai1=animMax (`:48926`) | `w4_d20_t5_true_excalibur_ghost_is_damage_zero` (`:18980`) |
| 757 TerraBlade | `:48930-48937` | 984 (`:48933`) | 985 beam vel×5 ai1=18 (`:48934`) | `w4_d20_t5_terra_blade_beam_is_times_five` (`:19002`) |

Shared shape: `ai0 = dir·gravDir` (gravDir=1 upright fold), `ai1 =
itemAnimationMax` (prefixed melee-speed int, `scaled_anim_secs`), `ai2 =
GetAdjustedItemScale` via `set_ai2` (glove folds false, documented),
MountedCenter half-box spawns, `GetWeaponDamage` epsilon via
`whip_weapon_damage`, arpen + revolver-crit stamps on both slots, ZERO
`Main.rand` draws (`w4_d20_t5_the_sword_arms_spend_no_draw`, `:19025`).
Copied-rows pin: `w4_d20_t5_the_copied_slash_rows_carry_style_190` (`:18890`).
Canary: `w4_d20_t5_the_flint_slash_canary_spawns_with_the_scale`
(`projectile_ai_misc.rs:2967`) — dmg 89, vel(dir,0), ai0=dir ai1=6 ai2=1 +
the d19-twin row pin. All WRITTEN NOT RUN. Residual: 997 HORSEMANS_BLADE
rides the copied rows for chain completeness but has NO use-arm this pass
(no C# terra-chain arm for it; its pumpkin spawner is `AI_194`-lane scope).

## S4 — band 191-200 census (the FINAL table)

Dispatch lines at HEAD `projectile.rs`; rows at HEAD `projectile/data.rs`:

| aiStyle | C# | arm | row(s) | dispatch | owner / status |
| --- | --- | --- | --- | --- | --- |
| 191 | `AI_191_TrueNightsEdge` `:41640-41808` | `ai_true_nights_edge` + `terra_blade_move` (`projectile_ai_misc.rs:1533`) | 973 `:3687`, 985 `:3693` | `:17527` | d20 ARMED+ROWED (`2b2b71a71`) |
| 192 | `AI_192_JuminoAnimation` `:41501-41638` | `ai_jumino` (+gift drain `drain_gift_items`) | 995 `:3632` | `:17433` | d20 ARMED+ROWED (`543bcbdf9`) |
| 193 | `AI_193_Flamethrower` `:41337-41401` | pre-existing arm | pre-existing rows | `:17371` | PRE-D20 base (`a725acb04` already carries it) — no d20 work owed |
| 194 | `AI_194_HorsemanPumpkin` `:41293-41336` | `ai_horseman_pumpkin` (`projectile.rs:10810`) | pre-existing | `:19701` | PRE-D20 (S1-D-4 census W-10) — no d20 work owed |
| 195 | `AI_195_JimsDrone` `:40923-41067` | `ai_jims_drone` + `drone_move` | 1020 `:3637` | `:17449` | d20 ARMED+ROWED (`6e44ca3bd`); camera T1 + sound T6a seeded cross-lane |
| 196 | `AI_196_Petal` `:40466-40560` | `ai_petal` + `petal_storm_move` | 1038 `:3623`, 1116 `:3628` | `:17414` | d20 ARMED+ROWED (`f4e660a20`) |
| 197 | `AI_197_CeilingAndHoverTurret` `:40307-40479` | pre-existing arm | pre-existing rows | `:18445` | PRE-D20 (S1-D-4b-2) — no d20 work owed |
| 198 | `AI_198_Flint` `:52238-52415` | `ai_flint` + EmitSpikes + FindBlockSurface/Ceiling | 1040 `:3642`, 1041 `:3646`, 1043 `:3651` | `:17467` | d20 ARMED+ROWED (`75286d0e4`); ai2 drain fix `23c075d7a`; sound T6b seeded |
| 199 | `AI_199_MeteorOre` `:40019-40041` | `ai_meteor_ore` | 1078 `:3617` | `:17400` | d20 ARMED+ROWED (`fe576990c`) |
| 200 | `AI_200_BirdDroppings` `:39985-40017` | `ai_bird_droppings` | 1080 `:3611` | `:17385` | d20 ARMED+ROWED (`e239723b3`) — DUP vs c10, keep-c10 at merge (T3, item 3) |

**ALL TEN styles ARMED+ROWED at this tip.** Seven are d20 ports; 193/194/197
were already armed on the pre-d20 base (`a725acb04`, verified by git-show this
seal). The band's residuals are merge-order + cross-lane seeds only: the 190
AI for 1043/972/982/983/984/997 is d19's (compose at merge), the 200 dedupe is
T3, camera T1 / sounds T6a-b / pet-chain T4a-c remain seeded (T4b landed
`9bc29bc7f`; T4a in flight in the co-lane's uncommitted work).

## S5 — the FINAL declaration (registry 109)

**The 191-200 band is COMPLETE on task/W4-d20-proj-ai-191-200.** Seven
verbatim AI arms + rows + dispatch + RNG-census witnesses (all WRITTEN NOT
RUN), the terra-chain use-path T5 (four arms + copied slash rows + ai2 drain
fix + 7 witnesses), the T4b pet persistence, and the six-item doc-sweep with
cross-lane seeds. The sword family composes per d19 26g §8's five ordered
steps with ONE amendment published here: step 4's dedupe covers the whole
slash set (972/982/983/984/997 + 1043), keep-d19, drop d20's `7b4173846`
copies at the four exact hunks (S2); step 5's T5 is LANDED and feeds the §7
envelope contract exactly — the merger runs d19's
`w4_d19_the_terra_chain_payload_feeds_the_190_envelope` plus d20's
`w4_d20_t5_*` witnesses against the merged pool as the gate. No further d20
rounds are owed on the band; the open seeds (T1/T3/T4a/T4c/T6a/T6b) belong to
their named lanes.

### Seal verification performed (code-only)

- `git status` co-lane check: dirty-2 (`projectile/data.rs` T4a pet rows +31,
  `world_sim/editing.rs` T4a pet shoot arm +71) — file hashes stable over a
  20 s re-read, last write ~2 h before seal → co-lane IDLE, not advancing;
  its uncommitted work was NOT staged or committed by this seal.
- `git log`/`git show --stat` census of the four ahead commits (S1).
- d19 26g §7/§7a/§8 re-read whole at `61f7638a5`; the envelope witness
  re-read at `projectile_ai_sword_styles.rs:1625`.
- Row dup compare: d20 HEAD `data.rs` slash block + 1043 vs d19 tip
  `data.rs:3700-3729` — tuples value-identical (S2); overlay arms compared
  (d19 supersets confirmed).
- T5 arms re-read whole at HEAD `editing.rs:4093-4230` vs the four C# arms
  (S3 table); all 7 T5 witnesses located by name + line.
- Band dispatch grep at HEAD (ten styles, S4) + pre-existence check of
  193/194/197 at base `a725acb04`.
- NO cargo/tests/rustfmt/graphify run (lane constraint); witnesses WRITTEN
  NOT RUN; commit + push only, no merge, primary untouched.
