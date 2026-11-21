# 304 — W4-d13 PET-LIGHT RETIRE: the FINAL activation census + the pet-light-family declaration

> W4-d13 · 2026-11-21 · code-only (no cargo/tests/rustfmt/graphify run) ·
> branch `task/W4-d13-proj-ai-121-130`, base tip `462c86097` (activation
> 5/5, the pet item-use 3600 grant gap FOUND + FIXED).
> Renumbered **243 → 304** (2026-11-21, the SEAL round): the doc was
> first pushed as 243 (`1e915e254`, 15:20:29) — g58's
> `243-seal-verification.md` first-add `c9af393dd` (14:08:18) PREDATES
> it, and registry-150 round 3 (`51a28d09e`) confirms **243 = g58
> winner** (the g58 240–243 block), 246–303 are the round-2/3 assignment
> blocks, **304+ FREE** ("Do NOT pick max-on-origin+1; check this
> registry AND a fresh all-tip sweep first"). Fresh sweep executed:
> per-tip `git ls-tree docs/research/` over ALL origin + local refs AND
> the full-history `git log --all --name-only` scan — zero 304+ docs
> anywhere; 304 is the first free number per the registry's rule.
>
> This is the lane's FINAL research note: it composes the three
> activation cross-notes (3/5 wire-kite `ed631370d`, 4/5 PET-light
> `0f3cb1410`, 5/5 grant-gap `462c86097`) against the CONSUMED lanes'
> landed tips, records the one NEW gap the retire census found (the
> classic pet single-item arms — FIXED in this commit), and declares
> the pet-light family closed. Witnesses WRITTEN NOT RUN.

## 1. The c19 compose — the PET-light census cross-note CONFIRMED consumed

Consumed tip: `origin/task/W4-c19-addlight-projectiles` `a160ad9ff`
(round-2 accounting corrected: 44 ported / 53 lights / 21 absent).
Their doc is `65-addlight-proj-a.md` — registry-150's key-65 row assigns
it **189** on c19's OWN branch (rename not yet executed at the consumed
tip; cite as research/65→189).

**SEAL-round refresh (c19 round 3, tip `472ef867e`, read):** the four
rand-gated sites ARMED IN THE SIM (`pending_ai_lights` + the pass-tail
`drain_ai_lights`, the `item_lights.rs` named-stream precedent) — the
final accounting **58 ported sites (54 feed + 4 sim) / 64 contributions,
2 audit-landed, 7 absent** (502/1092/465/466/731/the phaseblade pair/
756). The zero-pet-site claim is UNCHANGED at the final tip: none of the
four sim-armed kinds (409/590/849/912) is a pet, the d9/d10/d11
activation list carries no pet kind, and the absent-7 carries none.
The compose below holds verbatim against BOTH tips.

The d13 4/5 claim ("the seven aiStyle-124 pets contribute ZERO sites to
the Projectile.cs AddLight family") is CONFIRMED against c19's landed
register, not just asserted:

* **No pet kind in any column.** The ported-44 list (346/732/533/580/
  644/604/617/641/the gem bolts/623/342/856/833-835/500/446/646-649/
  715-718/601 + the round-2 activations 602/612/953/614/636/658/855/
  1127/984/1126/933/1100/1078/1038/1116/1020/985/1115/1105/1098/1123/
  1091/1117 + the Chillet mount site 3) carries NONE of 650/882/888/
  895/896/898/957. The absent-21 list carries none either. e6's sites
  68-107 (`proj_ai_lights.rs`, tip `c8b072b69`) likewise: no pet kind.
  67 + 40 = 107 closed with ZERO pet contributions — the d13 census and
  the c19/e6 registers agree from both sides.
* **The d13 658 row ACTIVATED c19's site 28** (`:36150`, the
  SandnadoHostileMark alpha==0 white·0.5): c19's round-2 blocker-3 list
  cites "d13 (658)" explicitly (`proj_lights_a.rs:130`/`:461-464`, the
  conditional row invariant `:1453`). The collector matches `p.kind`
  directly — the arm goes live when d13's `SANDNADO_MARK` row
  (`projectile/data.rs:3599-3603`) merges; c19 copies no rows (the
  keep-one-dedupe convention).
* **The pets' actual light is the SAME skipped layer c19 documents.**
  c19's register records the hooks' `PlotTileLine` rope spread
  (`:52613`/`:52644`) as "the light-map write with no port — skipped,
  the 870 FloodFillTile precedent". The pets' 650/895/896 casts
  (`Projectile.cs:35008-35043`) are exactly that layer: the standing
  light-map-WRITE deferral, now censused from the pet side (colours/
  widths/anchors in §4 below). When that layer lands, c19's hook rows
  and the pet rows land together — one task, both registers cite it.
* **No gap to file against c19/e6.** The `ProjLight` `light > 0f` gate
  (`proj_lights.rs:100-101`, `:16155-16158`) never fires for the pets:
  none of the seven SetDefaults rows sets `this.light` (re-read at
  `:6977-6989`/`:8461-8473`/`:8508-8520`/`:8553-8565`/`:8576-8588`/
  `:9034-9046`; the port's `set_defaults_light` has no pet arm, the
  `_ => 0.0` default holds). d2's player-side cross-note ("the light
  pets are not Player.cs sites") closed from the projectile side at
  4/5; c19's consumed tip closes the loop.

## 2. The f11 retire CONSUMED — the kite-drain contract SEALED

Consumed tip: `origin/task/W4-f11-wiring-arms-4` `14ca0a8d5` (retire
phase 1b — the registry renumber landed: f11's docs took the ASSIGNED
233/234/235; the kite contract is **research/235 §2**, renumbered
136→153→235). f11f-seal reads: research/235 §1-§4 + research/133 §7.5
(the fold-list's kite-param entry).

What changed since d13's 3/5 contract (`ed631370d`):

* **The STREAM NOTE is CLOSED on f11** (`84ed6342b`):
  `wiring_devices::mass_wire_operation` now takes
  `drop_rng: crate::app::RngKey` and rolls the cut-cache `Item.DropCache`
  drops on the CALLER's stage stream (verified at the f11 tip:
  `wiring_devices.rs:414-481` — the doc + the `app.rngs.stream(drop_rng)`
  call). The 3625 click arm passes `UpdatePlayers` (behaviour unchanged);
  the kite drain passes `UpdateProjectiles` (the Kill runs inside
  `Main.cs:17775`'s swap). d13's "thread the stage stream or record the
  fold" alternative resolved to the thread — NO deviation to record.
* **The union hunk is pinned verbatim** (research/235 §2, published on
  f11 at its `world_sim/projectiles.rs` pass tail, after
  `take_spear_hold`): the `owner as u32 != app.local_id` gate, the RAW
  `app.wire_tool_mode` (`Wiring.cs:3241` — no Actuator strip; the 3625
  arm's `Player.cs:46908` mask is NOT the kite's), no cooldown gate.
* **The one-drain-shape rule**: the merger keeps EITHER the direct
  drain OR the `app.pending_mass_wire_ops` buffer + a second-pass
  drain, never both (the buffer field drops if the direct drain wins).
* **d13's half VERIFIED LANDED by f11** (research/235 §1's table): the
  651 row, the aiStyle-125 drag + the two NO-OP L-scans, the release
  latch, the Kill-hook queue, the app buffer. Nothing for d13 to re-do.

THIS commit consumes the seal: the drain-site contract at
`world_sim/projectiles.rs` + the `app/mod.rs` field doc + the kite
module doc all re-pinned to the five-param signature with
`RngKey::UpdateProjectiles`; the shape-composition witness
(`w4_d13_the_queued_tuple_composes_with_the_f11_apply_shape`) extended
with the stream pin (`drop_rng.name() == "UpdateProjectiles"`, ≠ the
3625 arm's `UpdatePlayers`). WRITTEN NOT RUN. The drain itself stays
the d13×f11 merge hunk — nothing calls the unmerged fn on this branch.

Cross-lane note (no action for d13): g48's research/237-f11-tail-fix
(the minecart pound `kill_tile(no_item:true)` tail) targets f11e/the
merger, NOT the kite seam — the kite drain and that fix compose
independently at the merge.

## 3. The 3600 grant — does it ACTIVATE g10/d43? Census: NO (different lanes) + one NEW gap FIXED

Consumed tips: `origin/task/W4-g10-npc-kind-wire-3` `5dee28650` (the
11 town pets wired: 637/638/656/670/678-683 batch 1 `2d0403324` +
TownSlimeCopper 684 batch 2) and `origin/task/W4-d43-npc-kinds-631-697`
`0253bde90` (batch 13/13, the distressed pals 695/696 among the last).

**g10's town pets do NOT consume the 3600 grant — a different use-arm.**
`NPCID.Sets.IsTownPet` (`NPCID.cs:10118`: 637/638/656/670/678-684) are
NPCs, not projectiles. Their acquisition lanes:

* the licenses 4829/4830/4910 (`ItemID.cs:11341/:11343/:11503`) ride
  `ItemCheck_UsePetLicenses` (`Player.cs:45352-45366`, the `:43606`
  call) → `LicenseOrExchangePet` (`:45438-45445`) →
  `NPC.UnlockOrExchangePet` (`NPC.cs:80894-80918`): the boughtCat/Dog/
  Bunny flags + the chat broadcast + the variation reroll. NO buff, NO
  projectile — the item rows carry `buffType` unset (`Item.cs:39538-
  39548`/`:39821-39830`, useStyle 4 consumable; the port's itemdata
  rows: buff_type 0, projectile None). The town machine spawns the
  pet NPC on housing (g10's landed lane).
* the town slimes 678-684 unlock through the bound-slime finds
  (685/686/687, `NPC.cs:1415`/`:2092`/`:5611`'s spawn rolls) — again
  NPC-side, no buff.

The license USE-arm itself (`UsePetLicenses`) is unported on every
consumed tip (g10 wires the NPC machine, not the item use; d13's grant
table correctly excludes the licenses — witness-pinned below). That is
g10's/f43's town lane residual, NOT a 3600-grant gap.

**d43's distressed pals do NOT consume the grant either.** 695/696
(`NPC.cs:17752-17779`, aiStyle 127 NPC rows) spawn through the wild
spawn roll (`NPC.cs:4371-4384` — the `HasItem(5663/5664)` +
`RollLuck(100) < 40` kind pick), run `AI_127_Pal`
(`NPC.cs:42929-43024`, unported on d43 — the kind holds, their
`npc.rs:6740` note), and on rescue drop the reward STAFF
(`AI_127_Pal_GiveRewerd` `:43026-43034`: 5663 Cattiva / 5664 Foxsparks).
Those staves are the i25 `MINION_STAFF_PET_BUFF_ITEMS` rows (`Player.cs:
53272-53275`'s summon-staff switch — ALREADY granted 3600 by the i25
wave; the keep-alives are the summon arms `Player.cs:10745-10760` etc.,
ported as `SummonBuffArm` rows in `player.rs`). The Chillets 5665/5666
are MOUNTS (`Item.cs:44509-44532`, mountType 62/63 — the c19 mount-light
lane, no buff). No d43 kind touches `ApplyPetBuffs`.

**The NEW gap the retire census found — the CLASSIC pet single-item
arms, FIXED.** Re-reading `ItemCheck_ApplyPetBuffs` WHOLE
(`Player.cs:53091-53344`) for the retire exposed that the 5/5 fix
covered the 3577 arm + the 54-case vanity switch but NOT the function's
remaining single-item arms — the CLASSIC pets/light pets:

| C# arm | item(s) | buff → pet |
| --- | --- | --- |
| `:53097-53100` | 669 Fish | 41 → 112 Penguin |
| `:53101-53104` | 115 Shadow Orb | 19 → 18 (light pet) |
| `:53105-53108` | 3060 Bone Rattle | 154 → 499 |
| `:53109-53112` | 3628 Companion Cube | 191 → 653 |
| `:53113-53116` | 3062 Crimson Heart | 155 → 500 (light pet) |
| `:53146-53153` | 753 Seaweed / 994 Eater's Bone | 42 → 127 / 45 → 175 |
| `:53154-53189` | 1169-1172 / 1180-1183 / 1242 | 50-57 / 61 → the Bone-Key..Wisp/Dino pets |
| `:53198-53233` | 1311 / 1837 / 1312 / 1798 / 1799 / 1810 / 1927 / 1959 | 65 / 85 / 66 / 81 / 82 / 84 / 91 / 92 |
| `:53242-53249` | 3043 Magic Lantern / 2420 Zephyr Fish | 152 → 492 (light pet) / 127 → 380 |

ALL carry `buffTime = 0` (registry-verified) — the SAME `:42880` ghost
the 5/5 fix documented — and ALL their keep-alives are already ported
(`buff_effects_misc.rs`'s pet table `:11406-11706` rows +
`buff_effects_accessory.rs`'s 19/40/41/152/91/92/154/155/191 arms). The
port granted them NOTHING at the use start → every classic pet
summoned through its item despawned, exactly the 5/5 failure mode.
FIXED: the arms join `PET_ITEM_BUFF_GRANTS` verbatim (the grant site is
unchanged — one `contains` check serves all rows). TWO exclusions,
witness-pinned:

* **603 Carrot** — the `:53093-53096` arm gates on
  `Main.runningCollectorsEdition`, a flag the port does not carry (the
  non-CE default false keeps the arm dead; adding the row would grant
  a buff the C# refuses). The `UpdatePet` CE gate (`:17303`) is the
  same flag — the whole CE lane stays out until a CE flag lands.
* **425 Fairy Bell** — the `:53121-53145` arm is NOT a plain
  `buffType` grant: `Next(3)` picks 27/101/102, dels the rival fairy
  buffs, then `AddBuff(num, 3600)`. d2's light-pets lane owns it (the
  fairy pick + the `UpdatePetLight` `SelectRandom` twin `:17335`);
  a plain row here would grant buff 27 unconditionally — WRONG.

The standing `UpdatePet`/`UpdatePetLight` deferral (the equipped-pet
3600 re-grant, `Player.cs:17298-17340` — ANALYSIS's i53 note "the
pet-grant task's seam") is UNCHANGED: this fix is the USE arm
(`ItemCheck`), not the equip arm (`UpdateEquips`). Both grant 3600;
the equip lane's task keeps its seam.

Witnesses (WRITTEN NOT RUN): the Shadow Orb use → the 190 eviction +
buff 19 at 3600 → 18000 + the 18 flag; the classic-row/registry
agreement + the nine exclusions (603/425/the three licenses/the two
Chillets) + the pal-staff i25-table membership.

## 4. The PET-LIGHT-FAMILY DECLARATION (final)

The aiStyle-124 family (`Projectile.cs:34970-35696`; the port
`projectile_ai_pets124.rs` + the `STYLE124_PETS` rows) is DECLARED
CLOSED for the light lanes, with the one standing deferral named:

| Kind | Pet | AddLight sites | `this.light` | PlotTileLine casts (`:35008-35043`) |
| --- | --- | --- | --- | --- |
| 650 | Suspicious Tentacle | 0 | 0 | `(0.5,0.9,1)*2` — self 20/20, owner 40/40 (`:35010-35014`) |
| 882 | Eye of Cthulhu | 0 | 0 | NONE — dark in C# too |
| 888 | Twins | 0 | 0 | NONE — dark |
| 895 | Fairy Queen | 0 | 0 | `(1,0.6,1)*1.5`, `!NotRetro` ×0.8 (`:35019-35022`); self ×1.5 30/20, owner ×0.75 40/40 |
| 896 | Pumpking | 0 | 0 | `(1,0.7,0.05)*1.5`, same ladder + retro gate (`:35030-35043`) |
| 898 | Ice Queen | 0 | 0 | NONE — dark |
| 957 | Glommer | 0 | 0 | NONE — dark |

* The `LightPet` set membership (`ProjectileID.cs:523`: 650/895/896
  among the 12) is the FREE-UP/eviction lane, NOT a light amount — the
  port's `LIGHT_PET_PROJ` carries it verbatim.
* The 107-site AddLight family (c19 67 + e6 40): ZERO pet sites (§1).
* The `ProjLight` chain: the `light > 0f` gate never fires (§1).
* The pets' ONLY light is the PlotTileLine/CastLightOpen light-map
  WRITE layer — the SAME standing deferral as the 870 FloodFillTile
  spread and c19's hook rope spreads. The rows above are that layer's
  census: colours, the retro gate, the width pairs (velocity-lead
  `Center + velocity*6` / `Left→Right`), the self/owner anchor split.
  When the light-map-write task lands it consumes THIS table + c19's
  hook rows; no pet-side work remains before it.
* The item-use lane: the 3600 grants complete (§3 + the 5/5 fix); the
  keep-alives, the eviction, FreeUpPetsAndMinions all ported. The
  equip-arm (`UpdatePet`/`UpdatePetLight`) + CE + fairy-bell lanes are
  named owners' seams, not gaps of this family.

## 5. Verification record

* Consumed tips read via `git show origin/task/W4-…:<file>` (c19
  `a160ad9ff`, e6 `c8b072b69`, f11 `14ca0a8d5`, g10 `5dee28650`, d43
  `0253bde90`, d54's registry-150); every C# anchor re-read in this
  worktree's `decompiled/Terraria/` at the cited lines.
* Number freeness (the SEAL round re-scan): 243 LOST to g58
  (`c9af393dd` 14:08:18 < this lane's `1e915e254` 15:20:29; registry-150
  round 3 `51a28d09e` confirms 243 = g58 winner, 304+ FREE). Fresh
  per-tip `git ls-tree docs/research/` over ALL `refs/remotes/origin/*`
  + `refs/heads/*` AND the full-history `git log --all --name-only`
  scan — zero 304/305 docs anywhere; the rename 243→304 + the band
  bible's 305 execute on THIS branch only.
* This commit's code changes: the drain-contract doc trio (projectiles
  pass tail, app field, kite module), the kite witness's stream pin,
  `PET_ITEM_BUFF_GRANTS` + the grant-site comment, 2 new witnesses.
  Witnesses WRITTEN NOT RUN (code-only mandate; no cargo/tests/
  rustfmt/graphify invoked).

## 6. The SEAL round (this rename commit) — the remaining receipts

**(a) The g64 895/898/957 DUP — keep-d13-superset CONFIRMED at the
current tips.** g64's FINAL 1 (`c82a17e19`) adjudicated the three row
tuples BYTE-EQUAL against d13 `b56db900`/`0f3cb141`; re-verified here
against d13 `1e915e254` × g64 `3cdc64236`:

* the row tuple: d13 `data.rs:3572-3575` (the seven-kind match arm) vs
  g64 `data.rs:3584-3586` (the three-kind arm) — both
  `(20.0, 20.0, 124, -1, 18000, 0, false, false, false, RESET_STATE)`,
  byte-equal;
* the flag lists: d13's `STYLE124_PETS.contains` (`:4226`/`:4269`,
  netImportant + ignoreWater) ⊇ g64's
  `matches!(kind, FAIRY_QUEEN_PET | ICE_QUEEN_PET | GLOMMER_PET)`
  (`:4216`/`:4262`) — value-identical on the three shared kinds;
* the frame counts: d13 `:4745-4747` (895→6, 898→16 via the
  PUMPKING_PET-shared arm, 957→12) = g64 `:4721-4723` verbatim;
* `LIGHT_PET_PROJ` byte-equal (both `[18, 500, 72, 87, 86, 211, 492,
  650, 702, 891, 896, 895]`).

The machine differs by DESIGN: g64's `ai_pets_style124` takes
`(…, dust, dust_ctx)` (the trail-dust emitters wired), d13's takes the
5-arg census shape. **Merge keep-one: d13's seven-kind arm + rows +
flag lists (the superset); g64's dust-wired machine body folds INTO it**
(the g64 FINAL union-table's own ruling — `813/895/898/957/650 → the
g64 dust-wired ai_vortex_ritual/ai_pets_style124` — reads d13's rows,
g64's emitters). No d13 change owed; the dup is merge-side only.

**(b) The f11 235 §2 seal re-target ACKNOWLEDGED.** f11's SEAL banner
(research/235 header, tip `14ca0a8d5`) re-targets the union to c42
`15810df4a`: the moved `mass_wire_operation` sits at
`world_sim/wiring.rs:1089` (still 4-arg, the hard-coded
`RngKey::UpdatePlayers` at `:1143` — re-read at the c42 tip `2910a7209`,
unchanged); the `drop_rng` param diff + the direct-drain replacement
(the buffer field DROPS) are research/133 §8.4(a)/(b). d13's seam
comment keeps the five-param CONTRACT shape (the post-param-diff
signature); the merger executes 133 §8.4 against the moved fn — the
call path becomes `crate::world_sim::wiring::mass_wire_operation`, NOT
`wiring_devices::` (f11's module drops per 133 §4a). Nothing on this
branch calls either; the one-drain-shape rule stands.

**(c) The registry renumber EXECUTED** (243→304, this commit): the doc
file + header, `anchors.txt` (2 rows), ANALYSIS (2 refs), PORT_MAP
(2 refs), `data.rs`/`projectile_ai_pets124.rs`/`editing.rs` doc comments
(5 refs), the INDEX row. The band bible takes **305**
(`305-projectile-ai-121-130-band-final.md`, the next free number, the
same fresh-sweep receipt).