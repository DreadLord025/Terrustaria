# 89 — DOC-SWEEP lane 13: Projectile OnHit / ModifyHit / CanDamage hook census (W4-g3)

> Researched 2026-11-21 against `a467726f` (the g3 hooks-DONE tip). C# cites
> are `decompiled/Terraria/Projectile.cs` (stable); Rust cites are
> `crates/tr-app/src/projectile.rs` at this revision. The lane's contract:
> (1) the g3 Kill 16/34/79 vs d78 `5fe6d940` duplicate — arm-by-arm keep-one;
> (2) re-census the REMAINING `Damage_PVE_Inner` OnHitNPC arms (`:13000-15000`)
> vs landed, take the still-unclaimed rowed+armed kinds; (3) the
> ModifyHitNPC/ModifyHitByNPC damage-modifier gaps (the whip tag damage/i24
> landed); (4) the CanDamage/CanHitNpc special gates. Port verbatim +
> witnesses, code-only.

## 1. The Kill 16/34/79 duplicate — g3 `9ee2a7a1`/`778573ca` vs d78 `5fe6d940`

d78 (`task/W4-d78-ui-sounds-menus`, tip `5fe6d940`, "Kill death-effects part
1") ports the SAME friendly magic resize-blast family as g3's `9ee2a7a1`
(aimed-spell 16/34/79) + `778573ca` (nebula/spirit/onyx 634/635/659/661).
Arm-by-arm:

| kind | C# Kill arm | g3 port | d78 port | verdict |
| --- | --- | --- | --- | --- |
| 79 RainbowRod | `:71257-71328` Resize(128,128)+pen−1+Damage+restore+Item10 | `aimed_spell_kill` (`projectile.rs:21789`) + `rainbow_rod_death_draws` | `magic_death_blast` (`5fe6d940:21371`) Resize(128,128)+pen−1+Damage+restore, NO cue/dust | KEEP g3 (superset: cue + dust census) |
| 16 MagicMissile | `:71329-71408` Resize(128,128)+pen−1+Damage+restore+Item10 | `aimed_spell_kill` + `magic_missile_death_draws` | `magic_death_blast` same shape, NO cue/dust | KEEP g3 |
| 34 Flamelash | `:71409-71489` Resize(96,96)+pen−1+Damage+restore+Item10 | `aimed_spell_kill` + `flamelash_death_draws` | `magic_death_blast` same shape, NO cue/dust | KEEP g3 |
| 634/635 NebulaBlaze | `:71995-72050` recentre 50+pen−1+Damage+Item14 | `nebula_spirit_onyx_kill` (`:21853`) + `nebula_blaze_death_draws` | `magic_death_blast` recentre 50+pen−1+Damage, NO cue/dust | KEEP g3 |
| 661 OnyxBolt | `:72226-72277` recentre 160+pen−1+Damage+Item14 | `nebula_spirit_onyx_kill` + `onyx_bolt_death_draws` | `magic_death_blast` recentre 160+pen−1+Damage, NO cue/dust | KEEP g3 |
| 659 SpiritFlame | `:72503-72542` ai0≥0 gate, recentre 80+Damage+Item14 | `nebula_spirit_onyx_kill` (ai0 gate, pen kept) + `spirit_flame_death_draws` | NOT in d78 (d78 covers 79/16/34/634/635/661 only) | g3 UNIQUE |

**Keep-one: g3 wins every shared arm.** g3's `aimed_spell_kill` /
`nebula_spirit_onyx_kill` are strict supersets of d78's `magic_death_blast`:
same resize/penetrate/Damage/restore state, PLUS the `SoundID.Item10/14` cue
(queued through `pending_proj_sounds`, the S1-E-21h seam) and the trail-death
dust census (the explicit `Main.rand` draws kept in C# order, bodies skipped).
d78's `magic_death_blast` carries neither. At merge, DROP d78's
`magic_death_blast` + its death-hook call; KEEP g3's two helpers + calls.
The 659 SpiritFlame arm is g3-unique (d78 never ported it).

**d78b active on 452/454 — read claims.** d78's ANALYSIS "Deferred" paragraph
names the hostile 452/454/448 EVP blasts as the NEXT batch (not yet ported in
`5fe6d940`). g1's census (`26d`, `5bd68ce2`) claims 452/454/459/521/448/80 and
its in-flight worktree (`TR-W4-g1`) carries `projectile_kill_hooks_a.rs`
(452/454/448 hostile EVP bursts + 459/521/80). So the hostile resize-blast
family is g1's claim, NOT d78's — d78 only PORTED the friendly magic family
(the duplicate above). No 452/454 code exists in d78's pushed tip; the "d78b
active on 452/454" note refers to d78's DEFERRED intent, which g1 already
claimed and is porting. Keep-one: g1 owns 452/454/448; g3 owns the friendly
magic family; d78's `magic_death_blast` is the duplicate to drop.

## 2. Re-census: REMAINING OnHitNPC arms (`Damage_PVE_Inner` :13000-15000)

The g3 hooks-DONE census (`26e`) ported 9 hooks (Kill 16/34/79/634/635/659/661
+ OnHit 659/aiStyle68/aiStyle50). After the d-lane pushes (d1-d21 aiStyles
1-210, a9 156/164/169/206, f20 ai001_d overlays), the AI-arm axis is exhausted.
The OnHit/modifier axis still has UNCLAIMED rowed+armed kinds. Method: walk
every `this.type == N` / `this.aiStyle == N` arm in `:13000-15000`, check the
kind's `proj_data` row + dispatch arm on main, then grep every pushed branch's
`projectile.rs` for the arm's C# line cite. A hook is UNCLAIMED iff no pushed
branch ports it AND its kind is rowed+armed on main.

### Landed OnHit arms (the seam, `damage_pve` :22654-23428)

- aiStyle-3 boomerang reverse (`:13068`, `:23090`), aiStyle-16 bomb fuse+face
  (`:13120`, `:23238`), aiStyle-68 Molotov (`:13149`, `:23255`), aiStyle-50
  Inferno face (`:13164`, `:23269`), aiStyle-99 yoyo (`:13256`, `:22991`),
  aiStyle-113 javelin pin (`:13210`, `:22953`), 41 Hellfire clamp (`:13252`,
  `:22966`), 659 SpiritFlame timeLeft=0 (`:13223`, `:22977`), 477 crystal-dart
  retarget (`:13534`, `:23306`), 294 Shadowbeam decay (`:13411`, `:23419`),
  638/639/640 moonlord decay (`:14177`, `:23409`), the immunity chain
  (`:13967-14119`, `:23186`), the whip debuffs/tag/on-hit (i24/i27,
  `:23107-23394`), the yoyo spawns (`:13707-13809`, `yoyo_on_hit`), the
  StatusNPC debuff families (arrows/bullets/darts/flares/thrown).
- a4 (`task/W4-a4-hit-time-multipliers`, `1b484c89`) owns the FULL hit-time
  damage-decay switch (`:13409-13533`, `hit_time_damage`) — 40 cases, all
  kinds. NOT re-ported here.
- d2 (`15e5448a`) owns the chain-weapon return arms (`:14148-14164`,
  aiStyle-13 + harpoon 23). d17 (`84126544`) owns the 162 pounce. d10
  (`07115c55`) owns the 93 Nail stick+multipliers (`:13334`).

### UNCLAIMED rowed+armed OnHit/modifier gaps (ported here)

| # | hook | kind(s) | C# span | effect | row/arm |
| --- | --- | --- | --- | --- | --- |
| A | OnHit friendly=false+ai1=1000 | 604 TerrarianBeam | `:13001-13005` | the beam spends on first hit | 115/`ai_yoyo_beam` |
| B | OnHit EoW ×0.65+pen−− | 400/401/402/1107/1108/1109 | `:13006-13013` | molotov/obsidian fire vs EoW segments | 14/`ai_spear` |
| C | OnHit air ×1.5 | 710 BetsyArrow | `:13014-13017` | no solid tile 12 below → ×1.5 | 1/`ai_arrow` |
| O | OnHit ai0+=(60−ai0)/2 | 954 Spark | `:13018-13022` | the spark's hit-age jump (504/979 unrowed) | 2/`ai_style2` |
| D | ModifyHit stellar ×0.75 | 856 SparkleGuitar | `:13302-13305`/`:15186-15192` | no LOS to owner → ×0.75 | 167/`ai_sparkle_guitar` |
| E | ModifyHit expert EoW | bee family + 280 GoldenShower | `:13357-13367` | bee/grenade ÷5 vs EoW; 280 ×0.75 vs Destroyer | 16/12/36 |
| F | ModifyHit mult table | 968 PewMaticHorn | `:14470-14548` | the 24-case ai[1] damage multiplier | 2/`ai_style2` |
| G | ModifyHit ×0.5 | 533 DeadlySphere | `:14549-14552` | localAI[2]≥40 (the dash wind-up) → ×0.5 | 66/`ai_twins_minion` |
| H | CanDamage armorPenPercent=1 | 688/689/690 LightningAura | `:13232-13235` | full armor pen vs non-Guardian def<999 | 137/lightning |
| I | OnHit ai1=−1 | 317 Raven | `:13593-13597` | the raven re-arms its dive on hit | 54/`ai_raven` |
| J | OnHit Kill | 1088 Acorn | `:13959-13962` | the acorn dies on first NPC hit | 16/`ai_bomb` |
| K | OnHit immune=8 | 443 Electrosphere | `:14008-14011` | the sphere's 8-tick per-owner window | 77/`sphere_pulse` |
| L | OnHit ai0=0+ai1=−1 | 645 PoisonDart | `:14193-14198` | the flying dart places its cloud on hit | 1/`ai001_c` |
| M | OnHit ai1=−1 | 34 Flamelash / 79 RainbowRod | `:14219-14233` | the aimed spell re-arms when ai0==−1 | 9/`ai_magic_missile` |
| N | OnHit decay+sharpnel | 357 PulseBolt / 710 BetsyArrow | `:14121-14131` | 357 ×0.8 per pen hit; 710 5-shard fan | 1/`ai_arrow` |

All 14 are UNCLAIMED (no pushed branch's `projectile.rs` cites their C# lines)
and rowed+armed on main. The 390-392 spiders' `localAI[1]=20` (`:13963`) is
rowed but UNARMED (aiStyle-26's `ai026_c` covers only SLIME_MINION) — deferred
with its AI arm. The 323 Stake's vampire ×10 (`:13381`) needs NPC 158/159
(Vampire/VampireBat, unported kinds) — deferred. The 961/436/681 EVP tails
(`:14850`/`:14887`/`:14891`) need `damage_player`'s unconditional-kill rework
(the port kills every hostile shot on contact; the C# EVP arms write
penetrate/timeLeft instead) — deferred to the EVP lane.

## 3. ModifyHitNPC/ModifyHitByNPC damage-modifier census

The whip tag damage (`tag_damage_for`, `:12919-12933`, i24) and the hit-time
decay switch (a4 `hit_time_damage`, `:13409-13533`) are LANDED. The remaining
`Damage_GetProjectileSpecificDamageMultiplier` (`:14432-14566`) gaps:

- rocket ×2 (`:14435`, `IsARocketThatDealsDoubleDamageToPrimaryEnemy`) and mine
  ×3 (`:14439`, `IsAMineThatDealsTripleDamageWhenStationary`) — the rocket/mine
  kinds (134/137/140/143/776/780/793/796/799/784/787/790/246 and the mine set)
  are rowed; the flail ×1.2/×2 (`:14447-14465`) is LANDED (`flail_damage_adjust`).
  The rocket/mine multipliers are UNCLAIMED but their kinds' primary-enemy
  gate (`timeLeft>3` / `velocity.Length()<0.5`) is a per-pass read — deferred
  to the rocket lane (the sets are large, the gate is simple).
- 877/878/879 jousting-lance velocity mult (`:14466-14469`) — rowed (aiStyle
  19), UNCLAIMED; the `0.1 + player.velocity.Length()/7*0.9` needs the owner
  velocity snapshot. Deferred (the lance AI arm is the spear stand-in).
- 968 PewMatic 24-case table (`:14470-14548`) — PORTED here (F).
- 533 DeadlySphere localAI[2]≥40 ×0.5 (`:14549`) — PORTED here (G).
- 1024/1023/1091 (`:14553-14564`) — 1024/1023 unrowed, 1091 unrowed. Deferred.
- 856 stellar (`:13302`/`:15186`) — PORTED here (D).
- 688/689/690 armorPenPercent (`:13232`) — PORTED here (H).
- bee/grenade expert ÷5 + 280 expert ×0.75 (`:13357-13367`) — PORTED here (E).

## 4. CanDamage/CanHitNpc special gates

`Damage_CanDealDamage` (`:12446-12485`) is the per-kind no-damage gate. The
port carries it piecemeal: the flail hold window (`:12448` aiStyle15,
`flail_damage_adjust`), the 919/923 empress telegraph (`:12448`,
`projectile_ai_empress`), the 818 tiger ai0<1 (`:12448`, d17). The full gate
is a large per-kind list; the rowed+armed kinds' gates are mostly LANDED via
their AI arms' friendly/penetrate writes. The 688/689/690 armorPenPercent
(`:13232`) is the CanDamage-adjacent gate PORTED here (H). The
`CanHitWithMeleeWeapon`/`CanHitWithOwnBody` (`:12400-12445`) owner-LOS gates
are the spear family's (`owner_can_hit`, i06) — LANDED. The `Colliding`
(`:15336`) per-kind hitbox gates (flail ellipse, gem-staff, 454/452 hostile)
are LANDED (flail) or g1's claim (452/454). No new CanDamage gate is unclaimed
for a rowed+armed kind beyond H.

## 5. Port contract (code-only)

Every arm re-read against its C# block; the explicit `Main.rand` draws kept in
C# order (the 710 sharpnel fan is RNG-free — the 5 velocities are deterministic
rotations; the 968/533/856/688/bee/280 modifiers are RNG-free). The dust/gore/
particle bodies are SKIPPED (the sibling lanes'). Witnesses WRITTEN, NOT RUN
(no cargo/rustc/rustfmt/tests/graphify invoked — the Lead's constraint). The
ports extend `damage_pve`'s existing seam (per-NPC damage mult, pre-strike
modifiers, the immunity chain, the post-strike OnHit arms), no new module.
