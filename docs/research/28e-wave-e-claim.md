# S1-E-7 wave E (W4-d71) — disjoint-range claim + live-set census

> Worker W4-d71, branch `task/W4-d71-buff-effects-wave-e` @ main `a725acb0`.
> Code-only wave: tests WRITTEN NOT RUN (no cargo/rustc per the worker contract);
> compile-by-inspection. This note is the early claim commit — the range below
> is this worker's; c16 owns the first missing block (`:9914-10048`), c17 the
> second (`:10318-10461`, claim `fa1e8930`, `28c-wave-c-claim.md`), c18 the
> third (`:10462-10976`, claim `02d946e1`, `28d-wave-d-claim.md`).

## The live-set census (re-derived at `a725acb0`)

C# chain: `Player.UpdateBuffs` (`decompiled/Terraria/Player.cs:9855-12069`),
every `buffType[j] == N` head plus the six range heads (`95..97`, `170..172`,
`98..100`, `176..178`, `173..175`, `179..181`) = **270 distinct ids** (the
c17/c18 census, reproduced — counts match).

**Live-set key**: the ids with a LIVE effect owner — `player.rs`'s wave-A
effect scan (the `for bt in self.buffs.active_types()` match), the DoT
derivation (`derive_dot_inputs`'s `holding(N)` reads), `SUMMON_BUFF_ARMS` +
`update_summon_buff_arms`, the WIRED `buff_effects_accessory.rs`
(`update_accessory_buff_arms`, hooked in `world_sim/player.rs` after the
summon pass), the mount re-affirm (`mount.rs`), and the live potion arms
(`potions.rs`: 94 via `apply_mana_sickness_arm`, the fed-state 332/333/334
cleanup). The wave-B `buff_effects.rs` and wave-D `buff_effects_misc.rs`
tables are **test-only** (their wiring is the waves' merge step) — they are
NOT live, but their ids are wave-owned: this wave does not re-port them.

**61 chain ids have no live owner and no c16/c17/c18 claim.** In C# chain
order they form the FOURTH missing block plus the tail of wave B's range:

- **The fourth block (d71 — THIS WORKER)** — `Player.cs:11406-12012`
  (**37 ids**): the late pet family **84, 61, 154, 65, 66, 42, 45, 50, 51,
  52, 53, 54, 55, 56, 85, 81, 82, 57, 60, 127, 136** (21 ids — all tabled in
  wave D's `PET_ARMS` but unwired), then the DoT/status tail **80, 320, 353,
  163, 164, 194, 195, 196, 197, 198, 145, 43** (12), **28** (Werewolf), and
  the hunger tiers **333, 334, 336** (3). The exact per-id list and heads
  are in the table below; the block spans the first no-owner head after
  152 (`:11369`, live via the accessory module) to the last one before the
  imbues (`:12013`, live via `IMBUE_ARMS`).
- The remaining no-live ids sit inside wave B's landed range
  (`:10049-10317`: 36/93/58/306/88/398/399/400/63/106/108/109/110/150/348/
  366/111/112/113/115/116/117/119/124 — 24 ids, the unwired `DebuffEffects`
  rows). NOT mine: wave B owns the block; its wiring rides the waves' merge
  step, unchanged by this wave.

## The claimed block, exactly (37 heads, disjoint by C# line)

| id | head | arm | live today |
| --- | --- | --- | --- |
| 84 | `:11406` | Black Cat pet 319 | none (misc table only) |
| 61 | `:11420` | Dino pet 236 | none |
| 154 | `:11434` | Baby Face Monster pet 499 | none |
| 65 | `:11448` | Eye Spring pet 268 | none |
| 66 | `:11462` | Snowman pet 269 | none |
| 42 | `:11476` | Turtle pet 127 | none |
| 45 | `:11490` | Eater pet 175 | none |
| 50 | `:11504` | Skeletron pet 197 | none |
| 51 | `:11518` | Hornet pet 198 | none |
| 52 | `:11532` | Tiki pet 199 | none |
| 53 | `:11546` | Lizard pet 200 | none |
| 54 | `:11560` | Parrot pet 208 | none |
| 55 | `:11574` | Truffle pet 209 | none |
| 56 | `:11588` | Sapling pet 210 | none |
| 85 | `:11602` | Corrupt Sapling pet 324 | none |
| 81 | `:11616` | Spider pet 313 | none |
| 82 | `:11630` | Squashling pet 314 | none |
| 57 | `:11644` | Wisp pet 211 | none |
| 60 | `:11658` | Crystal Leaf 226 + dedupe walk | none |
| 127 | `:11679` | Zephyr Fish pet 380 | none |
| 136 | `:11693` | Mini Minotaur pet 398 | none |
| 80 | `:11727` | Blackout | none |
| 320 | `:11756` | Oozed (drippingSparkleSlime) | none |
| 353 | `:11784` | Shimmering | clear-only (`player.rs` shimmer state) |
| 163 | `:11826` | Obstructed | none |
| 164 | `:11831` | Distorted (vortexDebuff) | none |
| 194 | `:11835` | Wind Pushed | none |
| 195 | `:11839` | Withered Armor | none |
| 196 | `:11847` | Withered Weapon | none |
| 197 | `:11851` | Oozed/Slow (ogre spit) | none |
| 198 | `:11855` | Parry Damage | none |
| 145 | `:11859` | Moon Bite (moonLeech) | none |
| 43 | `:11882` | Paladin's Shield (defendedByPaladin) | none |
| 28 | `:11893` | Werewolf | none |
| 333 | `:11979` | Hungry | fed-state only (`potions.rs`) |
| 334 | `:11994` | Starving | fed-state only |
| 336 | `:12009` | Sugar Crash | none |

(205 Ballista Panic `:11843` is LIVE — `player.rs`'s match arm — excluded;
149 Webbed `:11863`, 29 `:11886`, 33 `:11911`, 25 `:11918`, 26 `:11931`,
206 `:11947`, 207 `:11963` are LIVE in the wave-A match — excluded. The
DoT flag arms 70/397/20/21/22/23/24/103/137/67/68/39/323/44/324 are LIVE via
`derive_dot_inputs`/the wave-A match — excluded.)

**Boundary**: c16 ⊂ `:9914-10048`, c17 = `:10318-10461`, c18 = `:10462-10976`,
d71 = `:11406-12012`. No shared C# line, no shared id.

## RNG census of `:11406-12012`

Direct `Main.rand` draws: **ZERO** (line-window scan). Indirect draws:
- The 21 pet spawns + Crystal Leaf's spawn (`Projectile.NewProjectile`
  `:11417` etc.) — the projectile init rolls ride the spawn seam
  (`spawn_with_data_ai_owned`), exactly as the wired accessory module's
  `pool.spawn` already models; the table records spawn REQUESTS.
- **Shimmering 353** (`:11784-11825`): no rand; the `SolidTile3` tile scan
  (`:11807-11816`) is deterministic; the absorbed counter is timer
  arithmetic. The 60/6 outcomes and the `DelBuff` are exact.
- **Werewolf 28** (`:11893-11910`): no rand; the day/charm/merman gate and
  the `DelBuff(j); j--` rescan are deterministic.
- **Crystal Leaf 60** (`:11658-11678`): no rand; the kill-every-226-past-the-
  first dedupe walk is a pool request.

## Shape

Pure accumulator table, the wave-B seam pattern:
`crates/tr-app/src/buff_effects_status.rs` (`apply_status_effect_arm` →
`StatusEffects`). One dispatch fn matches the 37 ids; the pet rows ride a
21-row table (buff → projectile kind + flag + the `:12227-12258` helper's
18000 pin/owned-count gate, wave D's `PET_ARMS` shape — disjoint module, no
edits to `buff_effects_misc.rs`). Slot-ordered driver
`run_status_buff_chain` over `&mut BuffSlots` mirrors the C# `for j` loop
with the `DelBuff(j); j--` rescans (353/28). Wiring into the live tick is
the merge step (not this wave): the module is test-only until the waves
merge — no runtime behaviour change, no RNG or fixed-step-order change.
Witnesses: one `s1_e_7e_*` test per id + the chain-order/rescan/RNG pins.