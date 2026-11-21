# 75 — SWEEP-LUCK-LOOT: the Player.cs luck-factor / loot-drop census (W4-f32 resume)

Scope: the code-only RESUME sweep after the f32b DOC-SWEEP timeout. Three
jobs: (1) the dirty-file in-flight read (none — the tree was clean-frozen),
(2) the census of every C# `Player.cs` luck-factor / item-drop-luck site
against the port, porting the unclaimed small arms, (3) the e26-luck-system
compose check (factor census vs consumer lanes — disjoint?).

Code-only round: no `cargo`/test/rustfmt/graphify invoked. Witnesses WRITTEN
NOT RUN, verified by reading.

## 1. Alive-status (the resume probe)

`git status` clean at resume; the two mtime probes 92 s apart
(13:35:25 / 13:37:51 local) both read the same newest tracked file
(`docs/research/INDEX.md`, 12:57:19) — **dirty-frozen/dead**, the prior f32b
session's DOC-SWEEP tip `198b8dd71` was its last write. Resume proceeded.
The five unpushed DOC-SWEEP commits were pushed EARLY (attempt 4 of 5 after
three connection resets, 90 s retries): `c05ef86f7..198b8dd71` landed,
remote tip == local tip.

## 2. The Player.cs luck census (every `luck`/`RollLuck`/`GetLuck` site)

The full `Select-String` sweep of `Player.cs` (58 hits) classified:

### 2a. The VALUE side (e26's lane — NOT consumers)

| C# site | What | Port status |
| --- | --- | --- |
| `:4112-4118` | `NormalizedLuck` (the luck-bar UI value) | landed (M2-BUFF-6, `luck.rs`) |
| `:9924`/`:12047-12050`/`:13095`/`:16728`/`:18099`/`:29045-29047`/`:29131`/`:29184` | `luckNeedsSync` writes (packet 134) | net-only, skipped (recorded M2-BUFF-6 deviation) |
| `:10291-10299`/`:18814` | `luckPotion` tier set/clear (buff 257) | landed (e26 phase 2, the i37 drink seam) |
| `:17735-17914` | `luckyTorchCounter` / the torch-luck scan | landed (e27 torch lane) |
| `:18160-18166` | `LadyBugLuck` normalized getter | landed (`luck.rs`) |
| `:29043`/`:29121-29184` | `UpdateLuckFactors` / `RecalculateLuck` | landed (M2-BUFF-6 + e26 factor census) |
| `:54471` | `clonePlayer.luck` (the death ghost copy) | net/clone-only |

### 2b. The CONSUMER side (f32's lane)

| C# site | What | Port status |
| --- | --- | --- |
| `:18188-18210` | the five `RollLuck`/`RollBadLuck`/`RollOnlyBadLuck*` wrappers | landed (`loot.rs` five fns, e26 core) |
| `:18213-18229` | `GetPlayerWithHighestLuck` | landed (single player; `rain.rs`/`shop.rs` read it) |
| `:18231-18239` | `GetClosestRollLuck` (Vector2 + tile) | landed via `DropCtx.luck` (the pot Coin Portal, `WorldGen.cs:57176`) |
| `:18169-18177` | `GetClosestPlayersLuck` (Point + Vector2) | **DEAD in C#** — zero callers outside the declaration (whole-tree grep). Not ported, nothing to port. |
| `:18241-18249` | `GetClosestRollBadLuck` (Vector2 + tile) | **DEAD in C#** — zero callers outside the declaration. Not ported. |
| `:18928` | `TryToPoop` `RollOnlyBadLuckExtreme(10)` | landed (f32, `luck.rs::try_to_poop_stack`) |
| `:46675` | `PlayGuitarChord` `RollOnlyBadLuckExtreme(10)` | landed (f32, `luck.rs::guitar_chord_cue`) |
| `:44370` | Classy Cane `RollBadLuck(25)` coin shower | landed (f32, `editing.rs`) |
| `:44537-44549` | the melee Lucky Coin proc (`Next(5)` gate) | landed (S1-P-i50, `editing.rs`) — NOT a luck roll, a plain `Next(5)`; the coin FAMILY's luck twin is the +0.05 equip term (e26) |
| `:28981` | tile-contact `DamageVar(num, -luck)` | landed (`touch_damage.rs:256`) |
| `:31387` | NPC-contact `DamageVar(dmg, -luck)` | landed (`npc.rs:18290`) |
| `:44342` | melee-hit `DamageVar(dmg, luck)` | landed (`editing.rs:7812`) |
| `:45723` | the Don't-Starve living-tree chop `DamageVar(6, -luck)` | landed (`editing.rs:1610`) |
| `:29247/:29274/:29277/:29282` | `GetMinecartDamage`'s four `DamageVar(…, luck)` | **UNPORTED — the whole minecart NPC-strike pass** (`Player.cs:27539-27605`) is absent: the cart-speed hitbox, the max-class crit roll, `TryConsumingTimerCrit`, `GetMinecartDamage`, the knockback-resist divide, `ApplyDamageToNPC(…, 2343)`, `immune[i]=30`. NOT a small arm — a combat-pass gap (the crit + timer-crit + ApplyDamageToNPC banner/OnHit/armor-pen chain). Recorded, deferred to the mount/combat lane. |
| `:44094` | PVP melee `DamageVar(damage, luck)` | PVP — unreachable single-player (recorded) |

### 2c. The Projectile.cs luck sites (the loot-drop adjacency)

| C# site | What | Port status |
| --- | --- | --- |
| `:12938` | PVE `DamageVar(num10, owner.luck)` | landed (`projectile.rs:22805`) |
| `:14649` | PVP `DamageVar(…, owner.luck)` | PVP — unreachable |
| `:14831` | hostile-shot `DamageVar(damage, -owner.luck)` | the port's `damage_player` passes raw `p.damage` (no variance) — a recorded hostile-shot stand-in, NOT a luck-only gap (the whole `Damage_EVP` variance/banner/cold/scaling chain is the stand-in) |
| `:15039` | `SelfHurtPlayers` `DamageVar(value, -luck)` | the rocket self-damage is the documented M2-DEATH deviation (`rocket_fixed_self_damage` ported for the follow-up) |
| `:20673-20683` | the fishing-level luck rolls | landed (`tr-core/fishing.rs:383`) |
| `:13685-13697` | **the projectile-side Lucky Coin proc** | **WAS the one unclaimed small arm — PORTED this sweep** (see §3) |

## 3. The ported arm: the projectile-side Lucky Coin proc

`Projectile.cs:13685-13697`, inside the `ownedBySomeone & flag8` block
(`:13673`). The melee twin landed in S1-P-i50; the projectile twin was
explicitly recorded as "its own unported row" (ANALYSIS "The coins lane"
deviations, PORT_MAP:617) and no branch claimed it.

The C# shape, verbatim:
- Gate `(flag9 || targetNPC.value > 0f) && Main.player[owner].hasLuckyCoin &&
  Main.rand.Next(5) == 0`. `flag9` is the Target-Dummy debug override
  (`:12818-12821`, type 488 unported) → false.
- Denomination: copper 71, `Next(10)==0` → silver 72, `Next(100)==0` → gold 73
  (independent rolls, gold wins).
- `Main.rand.Next(1, 11)` — **drawn and DISCARDED** (`:13696`); the spawn is
  `stack: 1`. The draw still consumes its stream slot.
- `Item.RequestNewItem(velocity: (Next(10,31)*0.2*num23, Next(-20,1)*0.2),
  center: npc.Center, type, prefix: 0, ReserveForLocalPlayer)`.
- `num23` (`:13598-13628`): the slot's `direction`, overridden to the
  owner-centre side for kinds 697/699/707/708/759/1118 (the monk staves, the
  Baby Bird, the Clay Pot Minion), aiStyles 15/188-191, and the WaterStream 22.
  Of those only 759/1118/22 carry port `proj_data` rows; the full list is
  transcribed verbatim regardless.

Port: `projectile.rs::damage_pve` — the proc sits inside the landed block
after the javelin arm, before the whip/yoyo arms (the C# draw order: after
StatusNPC's dart/flare/thrown debuffs, before the whip switch / yoyo spawns).
The C# gate is `flag8` (not the strike result), but in this PVE walk every NPC
reaching the strike has `hp > 0` and no melee `hurt_cd` gate, so `landed` is
always true (`strike_pooled` returns true unconditionally) — the placement is
equivalent. The six rolls draw inline on the `UpdateProjectiles` stream; the
fully-rolled drop queues on `pending_lucky_coin_drops` and the world-sim pass
tail (`world_sim/projectiles.rs`, after the hit-sound drain) spawns the
`ItemDrop` (stack 1, prefix 0, the 14×14 box centred on the NPC). The spawn
draws NOTHING — the explicit velocity skips `Item.NewItem`'s spread draws
(`Item.cs:49738-49740`) — so the pass-tail placement is stream-exact and the
`UpdateItems` stream stays pristine.

The boxed walk (`damage_pve_boxed`) does NOT carry the proc: it is the
resize-burst seam (the gem-bolt AoE / the bombs), a separate C# `Damage()`
path whose `Damage_PVE_Inner` call is the same — but the port's boxed walk is
a reduced stand-in (no player borrow, class crits read 0). The proc needs the
owner's `hasLuckyCoin`, which the boxed seam does not thread; recorded as a
boxed-walk follow-up (the same seam's crit stand-in).

Witnesses (WRITTEN NOT RUN): `f32_lucky_coin_proj_tests` (6 — the six-draw
census + silver/gold/copper denominations, the gate refusal's one draw, the
clay-pot `num23` owner-side override, the immortal refusal, the no-coin
short-circuit) + `f32_lucky_coin_drain_tests` (1 — the pass-tail drain spawns
the draw-free drop, `UpdateItems` pristine).

## 4. The e26 compose check (factor census vs consumer lanes)

**Disjoint by construction.** e26 (`task/W4-e26-luck-system`, tip `048cca6cf`)
owns the luck VALUE — the `RecalculateLuck` INPUTS: the equipment tail
(`Player.cs:13050-13069`, the four `hasLuck_*` flags + coin), the carried
clover/feather scan (`:12723-12734`), the lantern-night + gnome per-tick
copies, the town-death NPCLuck + CheckActive ladybug writers, the potion
factor, and the luck-critter roster (361/362/604/605). Its ANALYSIS section
"The W4-e26 factor census (luck inputs wired)" is explicitly the input side;
every added path is draw-free (bool flags, f32 adds, i32 timers).

f32 (this branch) owns the luck CONSUMERS — the `RollLuck`/`DamageVar` READ
sites that spend the value e26 computes. The two meet only at the
`player.luck()` / `LuckState.luck` accessor: e26 writes it, f32 reads it. No
file overlap in the roll sites (e26 touches `luck.rs`'s factor accumulation +
`npc.rs`/`player.rs` equip arms; f32 touches `spawner.rs`/`world_sim/*`/
`projectile.rs` consumer seams). The compose is clean: e26's factor census and
f32's consumer census are complementary halves of the one luck system, not
overlapping claims.

The one shared field is `has_lucky_coin`: e26's equip tail arms the +0.05 luck
twin (`hasLuck_LuckyCoin`), f32/S1-P-i50's proc reads the `hasLuckyCoin` bool.
The port carries ONE bool (the flags are never set apart, ANALYSIS recorded) —
both lanes read the same field, no conflict.

## 5. Remaining unclaimed arms (recorded, NOT ported)

- **The minecart NPC-strike pass** (`Player.cs:27539-27605` + `GetMinecartDamage`
  `:29242-29293`): the four `DamageVar(…, luck)` calls are consumers, but the
  whole pass (the cart-speed hitbox inflate, the max-class crit, the
  timer-crit consume, the knockback-resist divide, `ApplyDamageToNPC`'s
  banner/OnHit/armor-pen chain, `immune[i]=30`) is unported. A combat-pass
  task, not a small luck arm — deferred to the mount/combat lane.
- **The hostile-shot `Damage_EVP` variance** (`Projectile.cs:14831`): the
  port's `damage_player` is a raw-damage stand-in; the luck term rides the
  whole variance/banner/cold/scaling chain, not a luck-only fix.
- **`GetClosestPlayersLuck` / `GetClosestRollBadLuck`**: DEAD in C# (zero
  callers) — nothing to port.
- **The boxed-walk Lucky Coin proc**: the `damage_pve_boxed` seam does not
  thread the owner's `hasLuckyCoin` (its class crits read 0 too) — a follow-up
  with that seam's player-borrow reconcile.

## 6. Verification

Code-only: no compiler/test/graphify invoked. Every ported block re-read
against the cited C# (the gate order, the six-draw census, the `num23`
override list, the discarded stack draw, the explicit-velocity draw-free
spawn). The `landed`-always-true claim verified by reading `strike_pooled`
(returns true unconditionally) + the PVE walk's `hp > 0` guard. The witnesses'
no-extra-draw assumptions verified by reading `tag_damage_for` (no active
tags → no draw), `yoyo_on_hit`/`whip_debuffs`/`special_bullet_status_npc`/
`thrown_on_hit`/`probe_launch_on_hit` (all kind-gated, disjoint from the
wooden-arrow / clay-pot rigs). The drain's `UpdateItems`-pristine claim
verified by reading the pass-tail drains (only `drain_catches` draws on
`UpdateItems`, and only on overflow — empty in the rig).