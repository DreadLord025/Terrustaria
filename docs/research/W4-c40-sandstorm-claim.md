# W4-c40 — Sandstorm (sim halves) — CLAIM

Branch `task/W4-c40-sandstorm`, worktree `TR-W4-c40`, base `a725acb0` (main pinned).
Code-only mode: NO cargo/tests/graphify invoked; witnesses WRITTEN NOT RUN.

## In-flight probe (before claiming)

- `task/W4-f42-sandstorm-2` (TR-W4-f42): **0 commits, clean at base** — f42 owns
  the SECOND-HALF VISUALS (the brief's split). c40 does NOT touch
  `sand.rs`/`app/filters.rs`/`render/post.rs`/dust-268/269 draw code.
- `task/W4-c39-lantern-birthday`: 1 commit (`f5ae8fc0`, rain/wind lantern gates)
  — disjoint (lantern, not sandstorm). c40 does not touch `rain.rs`/`wind.rs`
  lantern arms.
- `task/W4-c41-town-spawn-invaders`: 2 commits (town-block gates + RNG census)
  — disjoint (town block `NPC.cs:794-923`); c40's spawn work is the
  sandstorm-desert ladder `NPC.cs:3949-4019`, not the town block.
- `task/W4-d33-npc-ai-styles-95-127`: 4 commits (census + styles 99/100/101).
  Its census claims MISSING 99-104 as unwired modules — **styles 102/103 are
  d33-claimed but UNDELIVERED**. c40's sand-elemental half ports the
  aiStyle-102 machine (its ONLY user is 541 SandElemental, `NPC.cs:15902`)
  in `npc_ai_sand_elemental.rs` UNWIRED (style73 precedent).
  **MERGER KEEP-ONE**: if d33 later lands `npc_ai_style102.rs`, keep ONE
  machine (C# decides); c40's module is kind-scoped 541-only.
- `task/W4-d41-npc-kinds-491-560` / `task/W4-d42-npc-kinds-561-630`: 0 commits.
  The kind ROWS for 541-546/580/581 belong to those lanes. c40 does NOT add
  `NpcKind` variants; the spawn ladder consumes the vanilla rolls and maps
  unported kinds per the spawner's documented consume-and-fallthrough
  precedent (`spawner.rs:874-902`).
- `task/W4-d130-weather-ambience`, `task/W4-f39-weather-slimerain`: 0 commits —
  no overlap with the sandstorm event machine.

## Scope (C# Main / Player / NPC / Sandstorm — the sim gaps)

Already landed (verify-only, NOT re-ported): `Sandstorm.UpdateTime` severity
machine + start/stop + `ChangeSeverityIntentions` + `UpdateSeverity`
(`scene.rs:986-1160`, R-11-5/WEATHER-2), `EmitDust` + dust-268 AI
(`sand.rs`, WEATHER-2/S1-E-1/E-3), the fg/bg filters (`app/filters.rs`,
BG-18), `ZoneSandstorm` derivation + rain suppression
(`scene.rs:786-790`, S1-L-1), the spawn-RATE override ×0.9/×1.2 / ×0.4/×1.5
(`spawner.rs:323-325`, M4-SPAWN-1), music 40 (`audio.rs:2726`), the
`TR_SANDSTORM` pin, the `.trwld` sandstorm quad (wld-roundtrip), the
start/stop F3 rows (S1-E-35), the scheduler walk (S1-F-14).

Real gaps this task ports:

1. **Severity — the hardMode start denominator** (`Sandstorm.cs:58-59`):
   `num = 21600 * (hardMode ? 2 : 3)`. The port pins `21600 * 3`
   (`scene.rs:1055`, "hardmode is not ported yet" — stale: `World.hard_mode`
   exists, `world.rs:368`). `update_time` gains the `hard_mode` input; the
   weather-tick caller (`world_sim/weather.rs:213`) feeds `world.hard_mode`.
   RNG order unchanged (same slot count, only the denominator).
2. **Desert override — the sandstorm spawn ladder** (`NPC.cs:3949-4019`):
   `Sandstorm.Happening && ZoneSandstorm && Conversion.Sand[tileType] &&
   Spawning_SandstoneCheck` → the pre-hardmode 546/61/69 ladder, the
   hardmode 541 (Next(20), !AnyNPCs(541)) / 510 DuneSplicer (noWorms,
   Next(3), CountNPCS(510) < 4, +10 tiles) / 542-545 sandshark
   (Corrupt/Crimson/Hallow tile swaps) / 78/79/630/80 mummy rows /
   546/580/581 tail. Plus `Spawning_SandstoneCheck` (`NPC.cs:5452-5491`,
   the 8-row ×(1+4+4) sand scan, ≥40 / remix ≥10) and
   `StrayMethods.CanSpawnSandstormHostile/Friendly`
   (`StrayMethods.cs:47-81`, the 3-column ExpandVertically ≥20/≥10 gates).
   Full vanilla roll order; unported kinds consume-and-map per the spawner
   precedent (d41/d42 own the rows).
3. **Sand elementals** — the 541 SandElemental: `SetDefaults` row
   (`NPC.cs:15898-15911`: 30×76, aiStyle 102, dmg 40, def 30, 5000 HP,
   kb 0.05, value 150 silver, rarity 2, NPCHit23/NPCDeath39), the
   aiStyle-102 machine (`NPC.cs:40147-40542` — the 541-only arm: the
   hp-scaled speed/accel, the <0.5-life kb-zero, the ai[0] 135→−300 attack
   cycle, the ai[0]==54 sandnado 658 spawn scan (3 columns, ≥10 apart,
   CanSpawnSandstormHostile), the 0.96 velocity damp, the tile-probe
   hover/step machine), and `FindFrame` case 541 (`NPC.cs:69734-69772`).
   UNWIRED module `npc_ai_sand_elemental.rs` (style73 precedent; d33
   keep-one note above). The 658 projectile + the dust-268/269 trails are
   REQUEST seams / f42-visuals — not drawn here.
4. **Push — the windPushed chain** (the buff-194 half):
   - the scene grant `!behindBackWall && ZoneSandstorm → AddBuff(194, 2)`
     (`Player.cs:25556-25559`) — missing from the port's scene-grant block
     (`player.rs:6974-6994`); rides `Auras` (new `sandstorm_wind` field,
     `scene.rs:249-256`/`:423-432`).
   - the live buff arm `194 → windPushed = true` (`Player.cs:11835-11838`)
     in the per-slot match (`player.rs:~6605`); the pure table
     (`buff_effects_misc.rs:660`) already carries it — keep-one, the live
     loop is the wired path.
   - `Player.windPushed` field + the `ResetEffects` clear
     (`Player.cs:16970`).
   - `CanBePushedByWind` (`Player.cs:19751-19770`: isLockedToATile false,
     roller-skates false, mounted-grounded-with-input false).
   - `HorizontalMovement`'s push arm (`Player.cs:19795-19817`: sign×0.06,
     |wind|>0.5 ×1.37, airborne ×1.5, moving ×0.8 + clamp ±0.072,
     facing-opposite run-threshold −|num2|×40) + the flag2 tail
     (`:20089-20107`: the one-sided velocity approach to num2).
     Wind speed rides the existing `wind: Option<&Wind>` channel
     (`update_with_grapple`), captured per tick.
5. **i12 sandStorm field reuse** — the Sandstorm-in-a-Bottle dash's run
   boost `sandStorm → runAcceleration ×1.5, maxRunSpeed ×2`
   (`Player.cs:26573-26577`) is missing after the floor-modifier chain
   (`player.rs:7025-7054`). Reuses the LANDED i12 `sand_storm` field
   (`player.rs:1220`, the `:19837` direction lock + the `:36298` leg pose
   already read it) — NO new field.

## RNG slots

- `UpdateTime`: the existing per-consumer storm stream (`scene.rs`
  `self.rng`); the hardMode fix changes only the `Next(num)` denominator,
  slot order/count identical (`Sandstorm.cs:62-74`).
- Spawn ladder: the `SpawnNPC` named stream (`Main.SwapRandom`,
  `spawner.rs` convention) — every conditional roll consumes exactly where
  the C# draws it, including unported-kind outcomes.
- Push + buff grant + run boost: ZERO draws (witnesses pin twin-tick
  stream equality).

## Out of scope (recorded)

- f42: second-half visuals (dust emit/draw, filters, 269 trail draw).
- d41/d42: NpcKind rows 541-546/580/581 (+ mummies/vulture/antlion rows).
- d33: unwired style modules 99-104/111+ (keep-one on 102 if both land).
- The 658 Sandnado projectile AI (projectile lanes), `SendData(7)`
  (multiplayer), `Main.remixWorld` arms (constants-off, cited).
