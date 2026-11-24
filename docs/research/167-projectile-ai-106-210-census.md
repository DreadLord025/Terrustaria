# 167 — Projectile aiStyles 106-210 COVERAGE census + AI_001 overlay tail (W4-f20)

> RENAME CHAIN: 26e → 167 (registry-150 §1/§3 round-1 assignment; 26e =
> W4-d7 `26e-projectile-ai-61-70-census.md` wins). Executed at the
> integration/batch-O merge (merger dedupe per §9/§10); refs re-pointed
> (main.rs/projectile.rs/projectile_ai001_d.rs census cites, anchors).

> Researched 2026-11-21 against `a725acb0` (wave-3 complete; main == base).
> C# cites are `decompiled/Terraria/Projectile.cs` (stable); Rust cites are
> `crates/tr-app/src/projectile.rs` / `projectile/data.rs` at this revision.
> Lane 2 of the Proj-AI census: f19 (`26c`, `e7e3c723` + findings
> `049cc8e2`/`c156903d`/`4a6b2419`/`807da87f`/`ec5dec82`) covered styles
> 1-105; this report covers **106-210** plus the **per-kind AI_001 overlay
> tail** (the rowed aiStyle-1 kinds whose per-type branches were never
> ported). Task contract: port only UNCLAIMED missing behaviour.

## Method

- Dispatch census: the `match data.ai_style` block in `ProjectilePool::update`
  (`projectile.rs:16731-19779`) — the exact-24-space arm scan (the
  `s1_d_4c_every_rowed_style_has_exactly_one_live_arm` pin's shape,
  `:23770-23856`), plus the kind-gated catch-arms
  (`projectile_frost_shots::handles`, `projectile_ai_empress::handles`
  `:19584`) and the aiStyle-14-internal fairy flip (170, below).
- C# arm spans: `AI()`'s if/else chain (`:23199-36815`); kind lists parsed
  from `SetDefaults`' `else if (this.type == N)` chain (`:697-10670`)
  INCLUDING the `type >= a && type <= b` ranges and the `DefaultToWhip`
  (`:10704-10718`, aiStyle 165) / `DefaultToKite` (`:10744-10751`, 160) /
  `DefaultToShortsword` (`:10773-10787`, 161) helper call sites.
- Claim census: `git branch -a` + `git worktree list` + sibling-worktree
  `git status --porcelain` (in-flight probe, 2026-11-21): the branch IS the
  claim (AGENTS.md). Probed d11-d21 + g1/g2/g3:
  - d12: untracked census doc `26d` (111-120 port plan: 111/112-sub/114/116/117).
  - d13: uncommitted `data.rs` rows (650/882/888/895/896/898/957 — the 124 pets).
  - d15: untracked `projectile_ai_dd2.rs` (141-146 + 148).
  - d16: pushed `a03fbcbd`/`939816d4`/`0581aab1`/`37cbc001` (151-154) +
    uncommitted 155 (`ai_mystic_snake_coil` + dispatch arm).
  - d18: uncommitted 172 (`ai_pellet_storm` + the 871 row).
  - d11/d14/d17/d19/d20/d21/g1/g2/g3: clean at base, branches live.
  - **a9** (`task/W4-a9-minion-four-machines`, tip `8c4fded8`, NOT in main):
    156/164/169/206 machines — f19's F2 claim attribution corrected below.

## Verdict (styles 106-210)

**38 of 105 styles armed; 67 missing. ALL 67 missing styles are CLAIMED by
live branches (d11-d21 decade claims + a9's four machines). UNCLAIMED
missing style arms: ZERO.** The unclaimed gap is on the OTHER axis — the
AI_001 per-kind overlays (below), ported by this lane as slice D.

## Coverage table (106-210)

P = ported (dispatch line), M = missing (C# span + SetDefaults kinds),
part = partial. "claim" = the live branch owning the style.

| style | state | Rust dispatch / C# body | kinds (M rows) | claim |
| --- | --- | --- | --- | --- |
| 106 | M | :33516-33552 (spin + ×0.98 drag + 60-tick fade/kill) | 569,570,571 | d11 |
| 107 | M | :33553-33678 (sand-tornado lift) | 575,596 | d11 |
| 108 | P | :17943 (:33679-34025 vortex ritual) | 578,579,813 | — |
| 109 | M | :34026-34065 (Dark Mage bolt arc) | 582 | d11 |
| 110 | M | :34066-34109 (Betsy fireball phases) | 584 | d11 |
| 111 | M | `AI_111_DryadsWard` :41134-41240 (:34110-34113) | 586 | d12 (26d plan 1) |
| 112 | part | :18803 serves 644 only (:34114-34363) | 590,836 missing | d12 (26d plan 2) |
| 113 | P | :17584 (`bone_javelin_step`; :34364-34367) | 598; 614/636/971/975/1024 unrowed (26d F1) | — |
| 114 | M | :34368-34413 (portal gate) | 602 | d12 (26d plan 3) |
| 115 | P | :17807 (`ai_yoyo_beam`; :34414-34437) | 604 | — |
| 116 | M | :34438-34458 (solar flare ray) | 607 | d12 (26d plan 4) |
| 117 | M | :34459-34637 (explosions) | 608,612,624,953,1044 | d12 (26d plan 5) |
| 118 | P | :17679 (`ai_nebula_arcanum`; :34638-34808) | 617 | — |
| 119 | P | :17699 (`ai_nebula_arm`; :34809-34887) | 618 | — |
| 120 | P | :17354 (`ai_stardust_guardian`; :34888-34891) | 623 | — |
| 121 | P | :17388 (`dragon_step`; :34892-34895) | 626,627 (+625/628 head/tail) | — |
| 122 | P | :17888 (:34896-34965 tower bolt) | 629,631 | — |
| 123 | P | :18719 (`AI_123_FloatingTurrets`; :34966-34969) | 641,643 (+642/644 shots) | — |
| 124 | M | :34970-35696 (light pet + flying pets) | 650,882,888,895,896,898,957 | d13 (rows in flight) |
| 125 | M | :35697-35833 (Grand Design drag) | 651 | d13 |
| 126 | M | :35834-35997 (mechanic piranha/laser) | 654,670 | d13 |
| 127 | M | :35998-36085 (portal gun bolt) | 656,657 | d13 |
| 128 | M | :36086-36197 (portal gun gate return) | 658 | d13 |
| 129 | P | :17792 (:36198-36309 spirit flame) | 659 | — |
| 130 | P | :18403 (`AI_130_FlameBurstTower`; :36310-36313) | 663,665,667 | — |
| 131 | P | :18663 (`AI_131_FlameBurstShot`; :36314-36317) | 664,666,668 | — |
| 132 | M | :36318-36434 (Houndius Shootius) | 672 | d14 |
| 133 | P | :17549 (`ai_dark_mage_raise`; :36435-36498) | 673,674 | — |
| 134 | P | :18827 (`AI_134_Ballista`; :36499-36502) | 677,678,679 | — |
| 135 | M | `AI_135_OgreStomp` :68671-68788 (:36503-36506) | 683,922 | d14 |
| 136 | M | `AI_136_BetsyBreath` :68789-68853 (:36507-36510) | 687 (row deviation — f19 F5, d14 owes the fix) | d14 |
| 137 | P | :19005 (`AI_137_LightningAura`; :36511-36514) | 688,689,690 | — |
| 138 | P | :19173 (`AI_138_ExplosiveTrap`; :36515-36518) | 691,692,693 | — |
| 139 | P | :19318 (`AI_139_ExplosiveTrapExplosion`; :36519-36522) | 694,695,696 | — |
| 140 | M | `AI_140_MonkStaffT1` :69416-69601 (:36523-36526) | 697,707 | d14 |
| 141 | M | `AI_141_MonkStaffT1Explosion` :69602-69620 (:36527-36530) | 698 | d15 (dd2 module in flight) |
| 142 | M | `AI_142_MonkStaffT2And3` :69817-69968 (:36531-36534) | 699,708 | d15 |
| 143 | M | `AI_143_MonkStaffT2Ghast` :69970-70036 (:36535-36538) | 700 | d15 |
| 144 | M | `AI_144_DD2Pet` :70038-70324 (:36539-36542) | 701,702,703,764,1050 | d15 |
| 145 | M | `AI_145_BookStaffStorm` :70326-70468 (:36543-36546) | 704 | d15 |
| 146 | M | `AI_146_DD2Victory` :53960-53962 — EMPTY C# body (:36547-36550) | 713 | d15 |
| 147 | P | :17309 (`ai_celeb2_rocket` + explosion; :36551-36554) | 715,716,717,718 | — |
| 148 | M | `AI_148_StarSpawner` :53094-53128 (:36555-36558) | 720 | d15 |
| 149 | P | :19635 (`AI_149_GolfBall`; :36559-36562) | 721,739-752 | — |
| 150 | P | :19701 (`ai_golf_club_helper`; :36563-36566) | 722 | — |
| 151 | P d16 | `AI_151_SuperStar` :22138-22168 (d16 `a03fbcbd`, :36567-36570) | 728 | d16 (landed on branch) |
| 152 | P d16 | `AI_152_SuperStarSlash` :22230-22351 (d16 `939816d4`) | 729,976,977,1045 | d16 |
| 153 | P d16 | `AI_153_ToiletPop` :52198-52236 (d16 `0581aab1`) | 733 | d16 |
| 154 | P d16 | `AI_154_VoidLens` :52154-52196 (d16 `37cbc001`) | 734 | d16 |
| 155 | M→d16 | `AI_155_MysticSnakeCoil` :51939-52112 (uncommitted arm in TR-W4-d16) | 754 | d16 (in flight) |
| 156 | M→a9 | `AI_156_BatOfLight` :51516-51911 (a9 `0ee3a5cd`/`76bb85e2`; UNARMED_ROWED[0]) | 755,946 | a9 (NOT d16 — finding F1) |
| 157 | P | :17478 (`ai_sharp_tears`; :36591-36594) | 756,961,1041 | — |
| 158 | P | :17614 (`AI_158_BabyBird`; :36595-36598) | 759 | — |
| 159 | M | `AI_159_PaperAirplanes` :48695-48756 (:36599-36602) | 761,762 | d16 |
| 160 | M | `AI_160_Kites` :48421-48694 (:36603-36606) | 765,822-824,826-830,838-840,843-846,850,852,853 (DefaultToKite) | d16 |
| 161 | M | `AI_161_RapierStabs` :47612-47666 (:36607-36610) | 802,842,938-945,1127 (DefaultToShortsword) | d17 |
| 162 | M | `AI_162_TigerPounce` :46701-47539 (:36611-36614) | 818 | d17 |
| 163 | M | `AI_163_Chum` :53474-53959 (:36615-36618) | 820 | d17 |
| 164 | M→a9 | `AI_164_StormTigerGem` :46593-46699 (a9 `db50ff98`; UNARMED_ROWED[1]) | 831,970 | a9 (NOT d17 — F1) |
| 165 | P | :18003 (`ai_whip`; :36623-36626) | 841,847-849,912-915,952,1028-1035,1104 (DefaultToWhip) | — |
| 166 | M | `AI_166_Dove` :47540-47611 (:36627-36630) | 851,855 | d17 |
| 167 | P | :17516 (`ai_sparkle_guitar`; :36631-36634) | 856 | — |
| 168 | M | `AI_168_FirstFractal` :46537-46572 (:36635-36638) | 857 | d17 |
| 169 | M→a9 | `AI_169_Smolstars` :45718-45853 (a9 `dc35f100`; UNARMED_ROWED[2]) | 864 | a9 (NOT d17 — F1) |
| 170 | P gate | kind-internal: 870's flip inside the 14 arm (`projectile_ai_light_items::glowstick_head` + hover, :24832-24835 → `AI_170_FairyGlowstick` :45662-45716); NO rowed kind carries aiStyle 170 | 870 (rowed 14) | — |
| 171 | P gate | :19584 empress catch-arm (`AI_171_HallowBossRainbowStreak` :45451-45568) | 873,931,1039 | — |
| 172 | M→d18 | `AI_172_HallowBossRainbowPelletStorm` :45569-45625 (uncommitted `ai_pellet_storm` in TR-W4-d18) | 871 | d18 (in flight) |
| 173 | P gate | :19584 empress (`AI_173_HallowBossRainbowTrail` :45429-45450) | 872 | — |
| 174 | M | `AI_174_MultisegmentPet` :45345-45428 (:36659-36662) | 883,887,893 | d18 |
| 175 | M | `AI_175_TitaniumStormShards` :45319-45344 (:36663-36666) | 908 | d18 |
| 176 | M | `AI_176_EdgyLightning` :45058-45116 (:36667-36670) | 916 | d18 |
| 177 | M | `AI_177_IceWhipSlicer` :45117-45266 (:36671-36674) | 917,969 | d18 |
| 178 | M | `AI_178_FireExplosion` :45267-45318 (:36675-36678) | 918 | d18 |
| 179 | P gate | :19584 empress (`AI_179_FairyQueenLance` :45032-45057) | 919 | — |
| 180 | P gate | :19584 empress (`AI_180_FairyQueenSunDance` :44984-45031) | 923 | — |
| 181 | P | :17324 (`AI_181_FairyQueenRangedItemShot` :43178-44475; :36687-36690) | 932 | — |
| 182 | M | `AI_182_FinalFractal` :43111-43177 (:36691-36694) | 933,1100 | d19 |
| 183 | M | `AI_183_ZoologistStrike` :43095-43110 (:36695-36698) | 880,929 | d19 |
| 184 | M | `AI_184_BadTorchLuck` :42787-42823 (:36699-36702) | 949 | d19 |
| 185 | M | `AI_185_LifeDrain` :42631-42666 (:36703-36706) | 476 | d19 |
| 186 | M | `AI_186_PrincessWeapon` :42667-42786 (:36707-36710) | 950 | d19 |
| 187 | P | :17553 (`ai_insanity_shadow`; :36711-36714) | 964,965 | — |
| 188 | P | :17557 (`ai_lights_bane`; :36715-36718) | 974 | — |
| 189 | P | :17561 (`ai_volcano`; :36719-36722) | 978,1125 | — |
| 190 | M | `AI_190_NightsEdge` :41972-42228 (:36723-36726) | 972,982,983,984,997,1043,1126 | d19 |
| 191 | M | `AI_191_TrueNightsEdge` :41640-41971 (:36727-36730) | 973,985 | d20 |
| 192 | M | `AI_192_JuminoAnimation` :41516-41639 (:36731-36734) | 995 | d20 |
| 193 | P | :17336 (`AI_193_Flamethrower`; :36735-36738) | 85,1106 | — |
| 194 | P | :19504 (`ai_horseman_pumpkin`; :36739-36742) | 1001 | — |
| 195 | M | `AI_195_JimsDrone` :40923-41133 (:36743-36746) | 1020 | d20 |
| 196 | M | `AI_196_Petal` :40466-40561 (:36747-36750) | 1038,1116 | d20 |
| 197 | P | :18248 (`AI_197_CeilingAndHoverTurret`; :36751-36754) | 1025 | — |
| 198 | M | `AI_198_Flint` :52238-52316 (:36755-36758) | 1040 | d20 |
| 199 | M | `AI_199_MeteorOre` :40019-40042 (:36759-36762) | 1078 | d20 |
| 200 | M | `AI_200_BirdDroppings` :39985-40018 (:36763-36766) | 1080 | d20 |
| 201 | M | `AI_201_ThrownMelee` :39954-39984 (:36767-36770) | 1082,1083 | d21 |
| 202 | M | `AI_202_TorchGodHelper` :39912-39953 (:36771-36774) | 1084 | d21 |
| 203 | M | `AI_203_StormLightning` :39641-39911 (:36775-36778) | 1091,1117,1122 | d21 |
| 204 | M | `AI_204_Digtoise` :38463-39640 (:36779-36782) | 1098,1123 | d21 |
| 205 | M | `AI_205_RemoteControlCar` :38076-38462 (:36783-36786) | 1105 | d21 |
| 206 | M→a9 | `AI_206_ForbiddenMinion` :37501-37641 (a9 `8c4fded8`; UNARMED_ROWED[3]) | 1119 (+1120 sign shot) | a9 (NOT d21 — F1) |
| 207 | M | `AI_207_SnappingStone` :37492-37500 (:36791-36794) | — (no SetDefaults kind binds 207) | d21 |
| 208 | M | `AI_208_GlacierFangShot` :36903-37050 (:36795-36798) | 1115 | d21 |
| 209 | M | `AI_209_ChlorophyteClaymoreBladeSlam` :69622-69816 (:36799-36802) | 1129 | d21 |
| 210 | P | :17808 (`ai_yoyo_shot`; :36803-36806) | 1130 (+1135 the Terrarian row) | — |

Armed (38): 108, 113, 115, 118, 119, 120, 121, 122, 123, 129, 130, 131, 133,
134, 137, 138, 139, 147, 149, 150, 157, 158, 165, 167, 170 (kind-internal),
171/173/179/180 (empress gate), 181, 187, 188, 189, 193, 194, 197, 210
(+112 partial). Missing (67): 106, 107, 109, 110, 111, 114, 116, 117, 124,
125, 126, 127, 128, 132, 135, 136, 140, 141, 142, 143, 144, 145, 146, 148,
155, 156, 159, 160, 161, 162, 163, 164, 166, 168, 169, 172, 174, 175, 176,
177, 178, 182, 183, 184, 185, 186, 190, 191, 192, 195, 196, 198, 199, 200,
201, 202, 203, 204, 205, 206, 207, 208, 209 (+112's 590/836 sub-arms).
Claim map: d11: 106/107/109/110; d12: 111/112-sub/114/116/117; d13:
124/125/126/127/128; d14: 132/135/136/140; d15: 141-146/148; d16:
155/159/160 (+151-154 landed); d17: 161/162/163/166/168; d18:
172/174/175/176/177/178; d19: 182/183/184/185/186/190; d20:
191/192/195/196/198/199/200; d21: 201-205/207/208/209; a9:
156/164/169/206. No port lands on this axis from this lane.

## AI_001 per-kind overlay tail (the second census axis)

The rowed aiStyle-1 kinds (`proj_data`, 97 kinds) vs the C# `AI_001`
(`:54002-57798`) per-type branches: slices A (16 kinds), B (9), C (9), the
riders (20/88/355/660/110 + 265/336/337/442/459), the frostmoon four
(345/348/349/350), the moonlord trio (638/639/640), the darts five
(267/477/478/479/442), the special arrows (15), the special bullets (11),
the javelin (507), the North Pole flake (344) and the Book of Skulls (837)
cover every rowed kind EXCEPT the seven below, which fell through to the
generic `ai_arrow` despite carrying per-type C# state:

| kind | name | C# overlay blocks (state-bearing) | ported by |
| --- | --- | --- | --- |
| 323 | Stake | :54142-54149 (alpha −50 fade) | slice D |
| 357 | PulseBolt | :56753-56777 (alpha −25 fade; the <170 trail is dust-only) | slice D |
| 485 | Hellwing | :57708-57736 (the (vel×39+ai)/40 homing blend + the 2-tick 5-frame machine + the direction-split atan2 rotation) | slice D |
| 661 | OnyxBlackBolt | :55734-55766 (alpha −55 fade + scale 1.3; the dust rings skipped) | slice D |
| 710 | DD2BetsyArrow | :55382-55414 (alpha −25 fade; the Next(4) dust block skipped WITH its draws — the nested wet Kill :55409-55412 is unreachable: the row sets ignoreWater :7529 so base.wet stays false, :16790) + :57549-57567 (the ai0≥0 double-increment 20-tick gravity arm, +0.2 / ×0.98 / 12 cap) | slice D |
| 719 | QueenBeeStinger | :55495-55517 (the localAI[0] one-shot latch; both dust bursts skipped) | slice D |
| 819 | BloodArrow | :56388-56422 (the localAI[0] one-shot latch + alpha −20 fade; the 8-dust burst / 5-dust trail / Next(2) red dust / light skipped) — the S1-P-i17 DOCUMENTED deferral ("819 keeps the generic flight … ANALYSIS records the deferral") is closed here | slice D |

All seven take the generic gravity tail (`:57568-57579`, pin 15 / +0.1)
except 710 (its own :57549 arm) and **357 (NO gravity — finding F7)** —
none is named in the :56888-57548 else-if arms; all seven take the default
rotation (`:57750`) except 485 (its own :57726-57735 tail). flag3 skip-list (`:55947-56036`) membership:
323 (`:55983`), 485 (`:56006`), 661 (`:56021`), 710 (`:56026`) REST the
counter; 357/719/819 count. The 485 ai0/ai1 carry the Hellwing Bow's
launch vector (`Player.cs:47573-47578` — the RotatedByRandom(π/8) twin).
RNG: every `Main.rand` draw in the seven blocks feeds dust/light only and
is SKIPPED with the layer (the ai001_a/b/c convention) — the module takes
no rng, the `UpdateProjectiles` stream untouched BY CONSTRUCTION. The
719/819 latches gate only skipped cues, so their state rides alone.

## Findings

- **F1 (claim attribution).** f19's F2 (`c156903d`) mapped the rowed-but-
  unarmed 156/164/169/206 to "d16/d17/d21" by decade; the live claims are
  **a9** (`task/W4-a9-minion-four-machines`, `0ee3a5cd`/`db50ff98`/
  `dc35f100`/`8c4fded8` — all four machines committed on that branch,
  not yet in main). d16/d17/d21 must NOT re-port them; the a9 merge drops
  the four UNARMED_ROWED entries.
- **F2 (207 has no kind).** `AI_207_SnappingStone` (:37492-37500) exists
  but NO `SetDefaults` arm binds aiStyle 207 (the whole-tree census finds
  zero `this.aiStyle = 207` writes) — the style is dead code in 1.4.5.8;
  d21's port is verify-only documentation, no row can exist.
- **F3 (170 is kind-internal).** No rowed kind carries aiStyle 170: the
  fairy glowstick 870 is rowed 14 and flips at runtime (:24832-24835);
  the port models the flip in `local_ai[2]` inside the 14 arm
  (`projectile_ai_light_items`), so the style-axis census counts 170
  armed-by-gate, not missing.
- **F4 (146 is empty).** `AI_146_DD2Victory` (:53960-53962) is an empty
  body — d15's arm is a documented no-op + row, not a machine.
- **F5 (f19 F5 — CLOSED on d14, retire-round re-probe).** The 687
  BetsyBreath row (12×12 aiStyle 1 vs C# 30×30 aiStyle 136, :7315-7324)
  and the 686 box (22×22 vs 30×30, :7301-7313) deviations stood at this
  census; d14 owed the row fix WITH the 136 arm. RETIRE-round re-read
  of `origin/task/W4-d14-proj-ai-131-140` (tip `eaa05ae9`): **landed** —
  `97c5a9847` ("style 136: BetsyBreath arm + f19 F5 row fixes") rows
  687 as 30×30 aiStyle 136 alpha 255 penetrate −1 and 686 as 30×30
  aiStyle 1 alpha 255 (both `Projectile.cs:7305-7324` verbatim, the
  row comments cite "W4-d14 / f19 F5"), the `136 =>` dispatch arm is
  live (d14-tree `projectile.rs:19623`) with the
  `w4_d14_betsy_breath_tests` module, and the `NPC.cs:62026` spawn
  wiring rides ai1=whoAmI. The 686 slice-B flight takes the corrected
  box at that merge (the A/B pins ride d14's commit). NOT yet in main
  (`a725acb0` still carries the deviant rows) — the closure is
  branch-landed, merge-pending; nothing further owed by any verify
  lane.
- **F6 (the slice-D gap).** The seven overlays above were rowed by
  S1-P-i13/i14/i17 AFTER the S1-D-5 slices were declared complete; no
  live branch claims them (d1's decade covers the style-1 AXIS, already
  armed — the per-kind overlay axis is the S1-D-5 line, closed for its
  declared kinds). This lane ports them as `projectile_ai001_d.rs`
  (slice D), gated after slice C, kind-disjoint from every existing gate.
- **F7 (the gravity chain's head is the 357 fade).** The `AI_001` gravity
  else-if chain is NOT headed by the 81/91 arm (`:56888`) — it is headed by
  the **357 fade block** (`if (this.type == 357)` `:56753`, then
  `else if 207||837` `:56778`, …, `else if 81||91` `:56888`, …,
  `else if type != 239 && type != 264` `:57187` wrapping the inner
  per-kind chain + the generic `ai[0] >= 15f` tail `:57568`). So the
  Pulse Bolt 357 matches the FIRST arm (fade only) and takes **no gravity
  at all** — its flight counter counts (`flag3` true) but never pins; the
  generic `ai_arrow`'s +0.1-after-15 fall is WRONG for it. The slice-D
  port gives 357 `Gravity::None`. (323/485/661/719/819 pass every arm and
  reach the generic tail; 710 takes its own `:57549` arm inside the
  `:57520` guard.)

## Verification

Code-only round (task contract: no cargo/tests/graphify). The slice-D
witness tests are WRITTEN NOT RUN — every expected value hand-derived
from the cited C# lines (derivations in the test doc-comments). A
follow-up build must run `cargo test -p tr-app projectile_ai001_d` before
the merge. The census tables were derived by direct file reads (no
junction recursion): the dispatch arm scan, the SetDefaults parse
(including range arms + DefaultTo* helpers), and the AI_001 type-branch
scan (184 `type ==` lines over :54002-57824).
