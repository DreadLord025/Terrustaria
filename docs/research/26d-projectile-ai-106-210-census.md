# 26d — Projectile aiStyles 106-210 COVERAGE census + the 211+ void (W4-g2)

> Researched 2026-11-21 against `a725acb0` (wave-3 complete), lane b of the
> proj-ai-missing pool. C# cites are `decompiled/Terraria/Projectile.cs`
> (stable); Rust cites are `crates/tr-app/src/projectile.rs` /
> `projectile_ai_*.rs` at this revision. Inputs: f19's 26c census
> (`origin/task/W4-f19-proj-ai-verify-1`, tip `ec5dec82`) — its 1-105 verdict
> (42 missing, ALL claimed d1-d11) is re-confirmed, not re-derived. Verify-only
> outcome: every missing tail style is CLAIMED, and C# has NO aiStyle above
> 210 — no unclaimed arm exists to port from this task.

## Method

- Dispatch census: the `match data.ai_style` block in `ProjectilePool::update`
  (`projectile.rs:16731-19779`) parsed arm-by-arm at 24-space indent: **96
  numeric arms** (f19's count holds) plus the kind-guarded empress arm
  (`_ if projectile_ai_empress::handles(kind)`, `:19584`). Of the 96, **33 sit
  in 106-210**; five more tail styles ride KIND gates (below).
- Claim census: `git fetch --prune` + `git branch -r` (ALL pushed branches
  read for projectile-file commits since `a725acb0`) + `git worktree list`
  (the branch IS the claim, AGENTS.md). d11-d21 all exist as live worktrees
  `TR-W4-d11` … `TR-W4-d21`; pushed among them: d13, d15 (both at `a725acb0`,
  no commits) and d16 (`37cbc001`, four arms landed). g1/g3 (the sibling
  missing-lanes) are fresh at `a725acb0` with no commits and no docs.
- C# arm spans: `AI()`'s if/else chain (`:23199-36807`); kind lists parsed
  from `SetDefaults`' `else if (this.type …)` chain (`:697-10674`) INCLUDING
  the `DefaultTo*` helper calls (finding G4 — a naive literal-only scan
  misattributes the kite/shortsword families).
- 211+ probe: every `.cs` under `decompiled/Terraria` (1546 files) and
  `decompiled-libs` (168 files) scanned for `aiStyle` assignments/comparisons.

## Verdict

**Tail 106-210: 38 of 105 styles armed (33 numeric arms + 5 kind-gated);
67 missing. All 67 missing styles sit inside claimed d-wave ranges (d11-d21).
UNCLAIMED missing arms in 106-210: ZERO.**

**211+: the style space ENDS at 210.** Every `aiStyle` write in the entire
decompiled tree is a literal; the maximum is 210 (`Projectile.cs:10601`,
`:10657`). `AI()`'s chain ends at `else if (this.aiStyle == 210)`
(`:36803-36806`); the highest `AI_<n>` method is `AI_210_YoyoShots`
(`:36809`). No compound (`+=`/`++`) or non-literal `aiStyle` write exists
anywhere; `NPC.cs` tops out at 127, ReLogic has zero `aiStyle` hits. There is
no style 211+ in C# to port — the "beyond f19 range" pool is exactly 106-210,
and it is fully claimed. No port lands from this task.

## Coverage table (106-210)

P = ported (dispatch line in `projectile.rs`), M = missing (C# body span +
SetDefaults kinds), claim = the d-wave branch that owns the style's decade.
"gate" = kind-gated arm outside the numeric match. "†" = armed on an
UNMERGED in-flight branch (finding G1), not on `a725acb0`.

| style | state | Rust dispatch / C# body | kinds | claim |
| --- | --- | --- | --- | --- |
| 106 | M | :33516-33552 (rotation ±vel×0.02, ×0.98 drag, ai0≥60 alpha/Kill) | 569,570,571 | d11 |
| 107 | M | :33553-33678 (575/596 type-split fade machine) | 575,596 | d11 |
| 108 | P | :17943 (:33679-34025) | 578,579,813 | d11 |
| 109 | M | :34026-34065 | 582 | d11 |
| 110 | M | :34066-34109 (Betsy fireball family) | 584 | d11 |
| 111 | M | :34110-34113 → `AI_111_DryadsWard` :41134 | 586 | d12 |
| 112 | P | :18803 (:34114-34363) | 590,644,836 | d12 |
| 113 | P | :17584 (:34364-34367, `projectile_ai_thrown` sticker) | 598,614,636,971,975,1024 | d12 |
| 114 | M | :34368-34413 | 602 | d12 |
| 115 | P | :17807 (:34414-34437 yoyo beam) | 604 | d12 |
| 116 | M | :34438-34458 | 607 | d12 |
| 117 | M | :34459-34637 | 608,612,624,953,1044 | d12 |
| 118 | P | :17679 (:34638-34808) | 617 | d12 |
| 119 | P | :17699 (:34809-34887) | 618 | d12 |
| 120 | P | :17354 (:34888-34891 stardust guardian) | 623 | d12 |
| 121 | P | :17388 (:34892-34895) | 626,627 | d13 |
| 122 | P | :17888 (:34896-34965) | 629,631 | d13 |
| 123 | P | :18719 (:34966-34969) | 641,643 | d13 |
| 124 | M | :34970-35696 | 650,882,888,895,896,898,957 | d13 |
| 125 | M | :35697-35833 | 651 | d13 |
| 126 | M | :35834-35997 | 654,670 | d13 |
| 127 | M | :35998-36085 | 656,657 | d13 |
| 128 | M | :36086-36197 | 658 | d13 |
| 129 | P | :17792 (:36198-36309) | 659 | d13 |
| 130 | P | :18403 (:36310-36313 flame burst tower) | 663,665,667 | d13 |
| 131 | P | :18663 (:36314-36317) | 664,666,668 | d14 |
| 132 | M | :36318-36434 | 672 | d14 |
| 133 | P | :17549 (:36435-36498 dark mage raise) | 673,674 | d14 |
| 134 | P | :18827 (:36499-36502 ballista) | 677,678,679 | d14 |
| 135 | M | :36503-36506 → `AI_135_OgreStomp` :68671 | 683,922 | d14 |
| 136 | M | :36507-36510 → `AI_136_BetsyBreath` :68789 (row deviation — 26c F5) | 687 | d14 |
| 137 | P | :19005 (:36511-36514) | 688,689,690 | d14 |
| 138 | P | :19173 (:36515-36518 explosive trap) | 691,692,693 | d14 |
| 139 | P | :19318 (:36519-36522) | 694,695,696 | d14 |
| 140 | M | :36523-36526 → `AI_140_MonkStaffT1` :69416 | 697,707 | d14 |
| 141 | M | :36527-36530 → `AI_141_MonkStaffT1Explosion` :69602 | 698 | d15 |
| 142 | M | :36531-36534 → `AI_142_MonkStaffT2And3` :69817 | 699,708 | d15 |
| 143 | M | :36535-36538 → `AI_143_MonkStaffT2Ghast` :69970 | 700 | d15 |
| 144 | M | :36539-36542 → `AI_144_DD2Pet` :70038 | 701,702,703,764,1050 | d15 |
| 145 | M | :36543-36546 → `AI_145_BookStaffStorm` :70326 | 704 | d15 |
| 146 | M | :36547-36550 → `AI_146_DD2Victory` :53960 | 713 | d15 |
| 147 | P | :17309 (:36551-36554 celeb2 rocket, `projectile_ai_misc`) | 715,716,717,718 | d15 |
| 148 | M | :36555-36558 → `AI_148_StarSpawner` :53094 | 720 | d15 |
| 149 | P | :19635 (:36559-36562 golf ball, `projectile_ai_golf`) | 721,739-752 | d15 |
| 150 | P | :19701 (:36563-36566 golf club helper) | 722 | d15 |
| 151 | M† | :36567-36570 → `AI_151_SuperStar` :22138-22168 — d16 `a03fbcbd` | 728 | d16 |
| 152 | M† | :36571-36574 → `AI_152_SuperStarSlash` :22230-22351 — d16 `939816d4` | 729,976,977,1045 | d16 |
| 153 | M† | :36575-36578 → `AI_153_ToiletPop` :52198-52236 — d16 `0581aab1` | 733 | d16 |
| 154 | M† | :36579-36582 → `AI_154_VoidLens` :52154-52196 — d16 `37cbc001` | 734 | d16 |
| 155 | M | :36583-36586 → `AI_155_MysticSnakeCoil` :51939 | 754 | d16 |
| 156 | M† | :36587-36590 → `AI_156_BatOfLight` — a9 `0ee3a5cd` | 755,946 | d16/a9 |
| 157 | P | :17478 (:36591-36594 sharp tears) | 756,961,1041 | d16 |
| 158 | P | :17614 (:36595-36598) | 759 | d16 |
| 159 | M | :36599-36602 → `AI_159_PaperAirplanes` :48695 | 761,762 | d16 |
| 160 | M | :36603-36606 → `AI_160_Kites` :48421 (`DefaultToKite` :10744-10751) | 19 kite kinds (G4) | d16 |
| 161 | M | :36607-36610 → `AI_161_RapierStabs` :47612 (`DefaultToShortsword` :10773-10787) | 11 shortsword kinds (G4) | d17 |
| 162 | M | :36611-36614 → `AI_162_TigerPounce` :46701 | 818 | d17 |
| 163 | M | :36615-36618 → `AI_163_Chum` :53474 | 820 | d17 |
| 164 | M† | :36619-36622 → `AI_164_StormTigerGem` — a9 `db50ff98` | 831,970 | d17/a9 |
| 165 | P | :18003 (:36623-36626 whip, `DefaultToWhip` :10704-10718) | 18 whip kinds | d17 |
| 166 | M | :36627-36630 → `AI_166_Dove` :47540 | 851,855 | d17 |
| 167 | P | :17516 (:36631-36634 sparkle guitar) | 856 | d17 |
| 168 | M | :36635-36638 → `AI_168_FirstFractal` :46537 | 857 | d17 |
| 169 | M† | :36639-36642 → `AI_169_Smolstars` — a9 `dc35f100` | 864 | d17/a9 |
| 170 | P gate | `projectile_ai_light_items.rs:226` (:36643-36646 → `AI_170_FairyGlowstick` :45662-45716; G2) | 870 (flips from 14) | d17 |
| 171 | P gate | :19584 empress (hostile 873 only; weapon variants 931/1039 excluded — 26c F2) | 873 | d18 |
| 172 | M | :36651-36654 → `AI_172_HallowBossRainbowPelletStorm` :45569-45625 (G5: unspawned) | 871 | d18 |
| 173 | P gate | :19584 empress (:36655-36658) | 872 | d18 |
| 174 | M | :36659-36662 → `AI_174_MultisegmentPet` :45345 | 883,887,893 | d18 |
| 175 | M | :36663-36666 → `AI_175_TitaniumStormShards` :45319 | 908 | d18 |
| 176 | M | :36667-36670 → `AI_176_EdgyLightning` :45058 | 916 | d18 |
| 177 | M | :36671-36674 → `AI_177_IceWhipSlicer` :45117 | 917,969 | d18 |
| 178 | M | :36675-36678 → `AI_178_FireExplosion` :45267 | 918 | d18 |
| 179 | P gate | :19584 empress (:36679-36682) | 919 | d18 |
| 180 | P gate | :19584 empress (:36683-36686) | 923 | d18 |
| 181 | P | :17324 (:36687-36690 fairy queen shot, `projectile_ai_misc`) | 932 | d19 |
| 182 | M | :36691-36694 → `AI_182_FinalFractal` :43111 | 933,1100 | d19 |
| 183 | M | :36695-36698 → `AI_183_ZoologistStrike` :43095 | 880,929 | d19 |
| 184 | M | :36699-36702 → `AI_184_BadTorchLuck` :42787 | 949 | d19 |
| 185 | M | :36703-36706 → `AI_185_LifeDrain` :42631 | 476 | d19 |
| 186 | M | :36707-36710 → `AI_186_PrincessWeapon` :42667 | 950 | d19 |
| 187 | P | :17553 (:36711-36714 insanity shadow) | 964,965 | d19 |
| 188 | P | :17557 (:36715-36718 light's bane) | 974 | d19 |
| 189 | P | :17561 (:36719-36722 volcano) | 978,1125 | d19 |
| 190 | M | :36723-36726 → `AI_190_NightsEdge` :41972 | 972,982,983,984,997,1043,1126 | d19 |
| 191 | M | :36727-36730 → `AI_191_TrueNightsEdge` :41640 | 973,985 | d20 |
| 192 | M | :36731-36734 → `AI_192_JuminoAnimation` :41516 | 995 | d20 |
| 193 | P | :17336 (:36735-36738 flamethrower, `projectile_ai_misc`) | 85,1106 | d20 |
| 194 | P | :19504 (:36739-36742 horseman pumpkin) | 1001 | d20 |
| 195 | M | :36743-36746 → `AI_195_JimsDrone` :40923 | 1020 | d20 |
| 196 | M | :36747-36750 → `AI_196_Petal` :40466 | 1038,1116 | d20 |
| 197 | P | :18248 (:36751-36754) | 1025 | d20 |
| 198 | M | :36755-36758 → `AI_198_Flint` :52238 | 1040 | d20 |
| 199 | M | :36759-36762 → `AI_199_MeteorOre` :40019 | 1078 | d20 |
| 200 | M | :36763-36766 → `AI_200_BirdDroppings` :39985 | 1080 | d20 |
| 201 | M | :36767-36770 → `AI_201_ThrownMelee` :39954 | 1082,1083 | d21 |
| 202 | M | :36771-36774 → `AI_202_TorchGodHelper` :39912 | 1084 | d21 |
| 203 | M | :36775-36778 → `AI_203_StormLightning` :39641 | 1091,1117,1122 | d21 |
| 204 | M | :36779-36782 → `AI_204_Digtoise` :38463 | 1098,1123 | d21 |
| 205 | M | :36783-36786 → `AI_205_RemoteControlCar` :38076 | 1105 | d21 |
| 206 | M† | :36787-36790 → `AI_206_ForbiddenMinion` :37501 — a9 `8c4fded8` | 1119 (+1120 sign shot) | d21/a9 |
| 207 | M | :36791-36794 → `AI_207_SnappingStone` :37492-37494 (EMPTY body — G3) | none (dead style) | d21 |
| 208 | M | :36795-36798 → `AI_208_GlacierFangShot` :36903 | 1115 | d21 |
| 209 | M | :36799-36802 → `AI_209_ChlorophyteClaymoreBladeSlam` :69622 | 1129 | d21 |
| 210 | P | :17808 (:36803-36806 → `AI_210_YoyoShots` :36809) | 1130,1135 | d21 |

Missing styles (67): 106, 107, 109, 110, 111, 114, 116, 117, 124, 125, 126,
127, 128, 132, 135, 136, 140, 141, 142, 143, 144, 145, 146, 148, 151, 152,
153, 154, 155, 156, 159, 160, 161, 162, 163, 164, 166, 168, 169, 172, 174,
175, 176, 177, 178, 182, 183, 184, 185, 186, 190, 191, 192, 195, 196, 198,
199, 200, 201, 202, 203, 204, 205, 206, 207, 208, 209. Claim map: d11:
106/107/109/110; d12: 111/114/116/117; d13: 124-128; d14: 132/135/136/140;
d15: 141-146/148; d16: 151-156/159/160; d17: 161-164/166/168/169; d18:
172/174-178; d19: 182-186/190; d20: 191/192/195/196/198-200; d21:
201-209. Eight of the 67 (†) are already armed on in-flight branches (G1).

## Finding G1 — eight tail styles are armed IN FLIGHT (do not re-port)

Two unmerged branches carry dispatch arms absent from `a725acb0`:

- `origin/task/W4-a9-minion-four-machines` (tip `8c4fded8`): styles **156**
  (`0ee3a5cd`), **164** (`db50ff98`), **169** (`dc35f100`), **206**
  (`8c4fded8`) — pool-side steps + dispatch arms; its `UNARMED_ROWED` pin is
  `[u8; 0] = []` (EMPTY). This retires 26c's F2 rowed-but-unarmed four.
- `origin/task/W4-d16-proj-ai-151-160` (tip `37cbc001`): styles **151**
  (`a03fbcbd`), **152** (`939816d4`), **153** (`0581aab1`), **154**
  (`37cbc001`) — verbatim arms in `projectile_ai_misc.rs` + rows + dispatch;
  its `UNARMED_ROWED` still lists `[156, 164, 169, 206]` (a9's arms not yet
  merged into d16's base).

On `a725acb0` the pin is `const UNARMED_ROWED: [u8; 4] = [156, 164, 169, 206]`
(`projectile.rs:23837`). After BOTH land the pin should be EMPTY; the merge
owner must reconcile the two constants (the a9 shape wins). The g-lanes and
d16/d17/d21 must NOT re-port these eight styles — the arms exist, they are
just unmerged. d16 still owes 155/159/160; d17 owes 161-163/166/168 (164/169
arrive via a9); d21 owes 201-205/207-209 (206 arrives via a9).

## Finding G2 — style 170 is armed via the fairy-glowstick KIND gate, not a numeric arm

C# assigns 870 (FairyGlowstick) aiStyle **14** in `SetDefaults` and flips it
to **170** inside the style-14 arm when `ai[1] > 0`
(`Projectile.cs:24832-24835`). The port mirrors this: the style-14 dispatch
(`projectile.rs:17083`) gates `projectile_ai_light_items::GLOWSTICK_KINDS`
(`:17091-17094`) and the module's aiStyle-170 hover lives at
`projectile_ai_light_items.rs:226` (`AI_170_FairyGlowstick`
`Projectile.cs:45662-45716`). A numeric-arm-only census would wrongly list
170 as missing — it is PORTED. No `SetDefaults` kind carries 170 directly
(the flip is the only writer), so its kind list is "870 (flips from 14)".

## Finding G3 — style 207 is DEAD CODE in C# (empty body, zero kinds)

`AI_207_SnappingStone` (`Projectile.cs:37492-37494`) is an EMPTY method, and
NO `SetDefaults` type arm assigns aiStyle 207 (the full 697-10674 parse
returns zero kinds; the only `== 207` in the file is the dispatch itself,
`:36791`). The style is unreachable in the shipped game. d21 owns it by
range (201-210); the faithful port is a no-op — d21 must NOT invent behaviour
for it, and a future coverage pin may legitimately keep 207 unarmed forever
(the `_ => {}` fallthrough IS the 1:1 port). Recorded so no lane "fixes" the
empty arm with a guess.

## Finding G4 — styles 160/161 (and 165) get their kinds from DefaultTo* helpers

A naive `this.aiStyle = N` kind scan MISSES the kite/shortsword/whip
families: their `SetDefaults` arms call helpers instead of assigning
literally —

- `DefaultToKite()` (`:10744-10751`, aiStyle **160**, 4×4, penetrate −1,
  extraUpdates 60): kinds **765, 822, 823, 824, 826, 827, 828, 829, 830,
  838, 839, 840, 843, 844, 845, 846, 850, 852, 853** (19).
- `DefaultToShortsword()` (`:10773-10787`, aiStyle **161**, 18×18, melee,
  extraUpdates 1): kinds **802, 842, 938, 939, 940, 941, 942, 943, 944, 945,
  1127** (11).
- `DefaultToWhip()` (`:10704-10718`, aiStyle **165**, already armed): 18
  kinds (841, 847-849, 912-915, 952, 1028-1035, 1104).

The literal-only scan also misattributes 1135 (Terrarian's yoyo projectile,
aiStyle **210**, `:10653-10666`) to the kite family — 1135's own type arm
sits AFTER the helper calls and assigns 210 directly. d16 (160) and d17
(161) must row the full helper-resolved kind lists, not the literal-scan
subset. (851/855 are style **166** doves, NOT kites — the kite range arms
skip them.)

## Finding G5 — style 172's only kind (871) has no spawner; 924 has no row

871 (HallowBossSplitShotCore, aiStyle **172**, `:8380-8392`) is never passed
to `NewProjectile` in `NPC.cs`/`Player.cs`/`Main.cs` (the empress module doc,
`projectile_ai_empress.rs:20-25`, records the same finding; `NPC.cs:83753`'s
"871" is a dust id). 924 (FairyQueenHymn) has no `SetDefaults` arm at all.
Both stay unspawned; d18 owns 172 by range and may port the arm for
completeness, but no live gameplay rides it. The empress gate deliberately
excludes the style-171 weapon variants 931/1039 (`:8826`/`:9807`) — their
`AI_171` parameter overrides belong to the player-weapon port (26c F2
stands).

## Claim ledger (tail decades, at census time)

| branch | range | tip | state |
| --- | --- | --- | --- |
| task/W4-d11-proj-ai-101-110 | 101-110 | a725acb0 | live worktree, no commits, unpushed |
| task/W4-d12-proj-ai-111-120 | 111-120 | a725acb0 | live worktree, no commits, unpushed |
| task/W4-d13-proj-ai-121-130 | 121-130 | a725acb0 | pushed, no commits |
| task/W4-d14-proj-ai-131-140 | 131-140 | a725acb0 | live worktree, no commits, unpushed |
| task/W4-d15-proj-ai-141-150 | 141-150 | a725acb0 | pushed, no commits |
| task/W4-d16-proj-ai-151-160 | 151-160 | 37cbc001 | pushed, 151-154 armed (G1) |
| task/W4-d17-proj-ai-161-170 | 161-170 | a725acb0 | live worktree, no commits, unpushed |
| task/W4-d18-proj-ai-171-180 | 171-180 | a725acb0 | live worktree, no commits, unpushed |
| task/W4-d19-proj-ai-181-190 | 181-190 | a725acb0 | live worktree, no commits, unpushed |
| task/W4-d20-proj-ai-191-200 | 191-200 | a725acb0 | live worktree, no commits, unpushed |
| task/W4-d21-proj-ai-201-210 | 201-210 | a725acb0 | live worktree, no commits, unpushed |
| task/W4-a9-minion-four-machines | 156/164/169/206 | 8c4fded8 | pushed, four machines armed (G1) |

The 26c 1-105 claim map (d1-d11) is unchanged: all 42 missing low styles
still claimed, d3's style-25 boulder still owed (26c F4 stands — d3's tip
`57abe80a` is still doc-only). No pushed branch since `a725acb0` adds a
projectile AI arm outside a9/d16 (the other projectile-file touches — a3/a4
pen+damage folds, c11's 435 row, d57's display-doll draw, d80/d91/d93
sound/net seams — add no dispatch arms).
