# 91 — THE fighter_walker_kind EXTENSION OBLIGATION (exact hunk): the d37e-E8-seal / d34-d43 E6 stream-parity contract

W4-d22 SEAL phase (S2), branch `task/W4-d22-npc-ai-styles-3-6` (retire
tip `a1a21235`). Code-only lane. This is the exact-hunk publication of
the roster-gap table's §2.1 obligation (`91-d22-roster-gap-table.md`,
R2 `28866c7c`) — the input d37e-E8-seal-active reads.

## 1. The gate + why the extension is mandatory

`ambient_sound.rs:603` — `pub fn fighter_walker_kind(kind: NpcKind) ->
bool` is a `matches!` list (:605-665). The sim's PRE-SLOT head block
(`world_sim/npcs.rs:4559-4623`, R1 `fb99a047`) runs, INSIDE that gate:
the C# idle counter (`fighter_idle_counter`, `:57120-57149`) then the
seven ambient-moan arms (`fighter_ambient_moans`, `:57336-57367`) on
the UpdateNPCs stream, BEFORE the machine's draws (the graveyard bash
Next(60) included). A rowed moan-listed kind MISSING from the gate:

* never ticks `ai[3]` (the C# idle counter — the `:57336` chase-gate
  input the axis core reads), and
* never rolls its C# moan arm — a per-tick UpdateNPCs stream
  shortfall that desyncs every later draw for every NPC in the slot
  order.

The emitter table (`fighter_ambient_moans`, `ambient_sound.rs:883`)
already names EVERY id in the extension list (the 1000-roll family
list :901-938, the 489/586 800-roll :946, the mummy 500-roll :950, the
vampire/Frankenstein/face-monster arms :954-963, the 269-280 range
:966). **The extension is the KIND GATE only — no emitter edit.**

## 2. The exact hunk

Append the new kinds as `|` arms inside the `matches!` list, after
the last current arm `| SolarSpearman` (`ambient_sound.rs:664`),
before the closing `)` (:665). Shape (one arm per lane batch, with
the lane's citation comment — the house convention the zombie-variant
block :608-612 models):

```rust
            | SolarSpearman
            // W4-d3x E6 — the <lane>'s moan-listed walkers (NPC.cs
            // :57340-57367 arms; the roster-gap table research/91 §1).
            | AngryBones            // 31, d34 (tip 5f3435a8)
            | ArmoredSkeleton       // 77, d35 (tip e5558b0a)
            ...
    )
}
```

**The ai[3]-free precondition** (the gate's doc comment :595-602): a
kind joins ONLY when its port machine leaves `ai[3]` free for the C#
idle counter. The plain walkers (`update_zombie`/`update_skeleton`/
the axis core) qualify on landing. A SHOOTER-block port that claims
`ai[3]` for its own state joins WITH its full machine migration (the
moan draw lands then — not before). Do not add a kind whose machine
still owns `ai[3]`: the counter would fight the machine state.

## 3. The per-lane extension list (variant names probed at the pushed tips)

The moan-listed gap ids (table §2.1, with the seal correction — the
`:57340` list rows 294-296 too, `fighter_ambient_moans` :910-912):
**31, 77, 78, 79, 80, 110, 159, 162 (DOUBLE-arm), 167, 181, 197, 201,
202, 203, 269-280, 291, 292, 293, 294-296, 449-452, 481, 586
(800-arm), 590, 591, 630, 632, 635, 691.**

| lane | tip probed | NpcKind variants to append (name verified at tip) | moan arm |
| --- | --- | --- | --- |
| d34 | `5f3435a8` | `AngryBones` (31) | 1000 |
| d35 | `e5558b0a` | `ArmoredSkeleton` (77), `Mummy` (78), `DarkMummy` (79), `LightMummy` (80); `SkeletonArcher` (110) WITH its shooter migration (ai[3] precondition §2) | 1000 / 500 (mummies) |
| d36 | E4 in flight (kinds unpushed) | 159 Vampire (500 case 29/7), 162 Frankenstein (BOTH 1000 + 500 case 29/6), 167 UndeadViking (1000), 181 FaceMonster (500 case 29/8), 197 ArmoredViking (1000), 201/202/203 the skeleton trio (1000) — names the lane's enum rows choose; the 163-no-entry / 198->199 fold decisions ride the same pass | per-id |
| d37 | `a341daa1` | `RustyArmoredBonesAxe`/`RustyArmoredBonesFlail`/`RustyArmoredBonesSword`/`RustyArmoredBonesSwordNoArmor` (269-272), `BlueArmoredBones`/`BlueArmoredBonesMace`/`BlueArmoredBonesNoPants`/`BlueArmoredBonesSword` (273-276), `HellArmoredBones`/`HellArmoredBonesSpikeShield`/`HellArmoredBonesMace`/`HellArmoredBonesSword` (277-280) | 1000 (the 269-280 range arm :966) |
| d38 | `1ba0395c` | `SkeletonSniper` (291), `TacticalSkeleton` (292), `SkeletonCommando` (293) — all three WITH their shooter migrations (ai[3] precondition); `AngryBonesBig`/`AngryBonesBigMuscle`/`AngryBonesBigHelmet` (294-296 — the SEAL correction: the `:57340` 1000-list rows them, emitter :910-912; plain walkers, qualify on landing) | 1000 |
| d40 | `8fe1d6d3` | `BoneThrowingSkeleton`/`2`/`3`/`4` (449-452, SHOOTER-excluded ladder — plain walkers, qualify on landing; 449 unclaimed at tip, the lane rows it), `GreekSkeleton` (481, WITH its shooter migration) | 1000 (449-452 range + 481 row) |
| d42 | `2d7ac08c` | `ZombieMerman` (586 — the 800-roll, style = the type), `TorchZombie` (590), `ArmedTorchZombie` (591 — 1000-list row, NOT flag24), `BloodMummy` (630 — 500 mummy arm) | 800 / 1000 / 500 |
| d43 | at base | 632 MaggotZombie, 635 SporeSkeleton, 691 MossZombie — names the lane's enum rows choose | 1000 |

NOT extended: the axis-claimed counter-only kinds already in the gate
(the scarecrows, Splinterling, the goblins/pirates, GrayGrunt/Scutlix/
VortexSoldier/SolarSpearman — draw-free, no moan-list id); the
SHOOTER machines that never migrate (their moan share is the cited
skip until migration); 534 DemonTaxCollector / 536 TheBride (no moan
arm — the default ladder only, but they DO join for the idle counter
when rowed: the gate serves the counter too).

## 4. The witness obligation (each lane's E6 pass)

Each appending lane extends its stream-parity witness: a fixture with
the new kind on a graveyard night rolls moan-then-machine-draw in the
C# order (the R1-re-derived
`w4_g4_the_graveyard_bash_draws_on_the_update_npcs_stream` shape,
`world_sim/npcs.rs`), and a draw-count witness pins the per-tick
Next(N) consumption for the kind's arm (the 162 DOUBLE-arm consumes
TWO rolls). Written-not-run in the code-only lanes; the merge gate
runs them.

## 5. Provenance

The gate/emitter/sim-block line cites re-read at the seal tip
(`a1a21235`); the lane variant names probed by `git show <tip>:npc.rs`
enum parse (no checkout). The obligation's id list is the R2 table
§2.1 verbatim. Code-only: no cargo/tests/rustfmt/graphify invoked.
