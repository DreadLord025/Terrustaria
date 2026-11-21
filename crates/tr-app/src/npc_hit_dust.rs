//! W4-g62 — the NPC HIT dust lane: `NPC.HitEffect`'s LIVE (`life > 0`)
//! per-kind dust arms (`NPC.cs:81654-89834`), reached from
//! `NPC.StrikeNPC_Inner`'s `HitEffect(hitDirection, num)` tail
//! (`NPC.cs:81476-81489`). The DEATH halves of the same arms are
//! W4-g59/g60's lane, the spawn-side dust g61's, the hit sounds c13's
//! (`strike_hit_cue`/`queue_hit_effect_sound`), the gore BODIES d55's —
//! where a gore call sits between dust draws its ARGUMENT draws are
//! consumed here in C# order (the W4-e2 split precedent).
//!
//! Contract (the S1-E-1 pool + d50's DustPool, `tr_core::particles`):
//! the strike captures a [`HitDustSnapshot`] at the C# `HitEffect` call
//! moment — AFTER the damage write and the knockback (`NPC.cs:81387-
//! 81475` precede `:81488`), so `life` is post-damage and `vx/vy` are
//! post-knockback. The sim drains the queue in `update_npcs` and every
//! draw rides the `UpdateNPCs` stream in C# statement order through the
//! verbatim `DustPool::new_dust` (`Dust.cs:75-256`): explicit `Main.rand`
//! argument/gate draws FIRST, then the pool internals, then the
//! post-spawn writes (the sentinel slot 6000 takes the writes exactly
//! like the C# when the pool refuses).
//!
//! RNG deviation (recorded, the W-10 gallop precedent): the C# runs
//! `HitEffect` inside the STRIKE's swap (`UpdatePlayers` for melee,
//! `UpdateProjectiles` for the projectile walk); the port's drain rides
//! `UpdateNPCs` after the tick — the draw SEQUENCE per strike is
//! identical, the stream placement is the documented pool-pass fold
//! (docs/ANALYSIS.md "The NPC hit-dust lane (W4-g62)").
//!
//! Batch ledger (C# `Dust.NewDust*` line = site id), batch 1:
//! - 84747 — the slime family's live gel (`:84721-84749`, types
//!   1/16/71/244/535): `dmg/lifeMax*100` × dust 4, alpha/color ride the
//!   NPC's own rows.
//! - 86455 — the demon-eye family (`:86449-86457`, types 2/190-194/
//!   317/318): ×100 dust 5.
//! - 86943 — the zombie/critter group (`:86937-86967`): ×100 dust 5,
//!   then the 186/632 gore-gate `Next(5)` draws (bodies d55's), the
//!   187 SlimedZombie extra loop and the 590/591 dry triple.
//! - 86957 — the 187 extra ×200 dust 4 (alpha 125, blue, height 24).
//! - 86964 — the 590/591 `!wet` 3 × dust 6 (same tint, height 24).
//! - 85803 — the bat group (`:85797-85805`, 49/51/93/150/152/226): ×30
//!   dust 5.
//! - 88533 — the skeleton group (`:88527-88535`): ×50 dust 26.
//! - 89556 — the goblin group (`:89550-89558`, 26/27/28/29/73/111):
//!   ×100 dust 5.
//! - 87508 — the Eater-of-Souls group (`:87502-87510`, 6/94/166): ×100
//!   dust 18, alpha/color/scale ride the NPC.
//! - 89708 — the hornet group (`:89702-89710`, 42/176/205/231-235):
//!   ×100 dust 18, alpha/color/scale ride the NPC.
//! Batch 2 (the slime-variant/jellyfish/lava/boss/sphere arms):
//! - 85415 — Crimslime 183 (`:85409-85417`): ×100 dust 136, (0,0),
//!   this.alpha/color.
//! - 85433 — CorruptSlime 81/121 (`:85427-85435`): ×100 dust 14, (0,0).
//! - 85484 — IlluminantSlime 120/137/138 (`:85478-85487`): ×50 dust 71,
//!   (0,0), alpha 200, velocity ×1.5.
//! - 85343/85345 — IceSlime 147/184 (`:85337-85347`): ×50 PAIRS — dust
//!   76 noGravity + dust 4 (97,200,255,100) alpha 100.
//! - 85549 — the jellyfish trio 63/64/103 (`:85534-85551`): ×50 dust 4,
//!   the per-type tint.
//! - 85568 — LavaSlime/Hellbat 59/60/151 (`:85562-85575`): ×80 dust 6,
//!   (2·dir,-1), this.alpha, scale 1.5, per-dust Next(8)!=0 → noGravity.
//! - 85676 — King Slime 50 (`:85670-85679`): ×300 dust 4, alpha 175,
//!   (0,80,255,100).
//! - 89531/89534 — the Burning Sphere 25 (`:89526-89537`): 20 PAIRS of
//!   dust 6 at (-vx·0.2,-vy·0.2), alpha 100 — scale 2 noGravity ×2, then
//!   scale 1 ×2 (the Item10 cue is c13's lane).
//! Batch 3 (the remaining spheres + the caster/boss live arms):
//! - 89543/89546 — the Water Sphere 33 (`:89538-89548`): the 20-pair
//!   dust-172 shape, scales 1.2/0.6.
//! - 89695/89698 — the Chaos Ball 30/665 (`:89690-89700`): the 20-pair
//!   dust-27 shape, scales 2/1.
//! - 89481/89482 — Meteor Head 23 (`:89470-89485`): ×100 — the Next(2)
//!   25/6 selector BEFORE the spawn, then the dust-6 velocity pair
//!   (vx·0.2, vy·0.2) alpha 100 scale 2 noGravity.
//! - 89509 — Fire Imp 24 (`:89503-89512`): ×100 dust 6 at (vx,vy),
//!   alpha 100 scale 2.5 noGravity.
//! - 84264 — Duke Fishron 370 (`:84258-84266`): ×100 dust 5.
//! - 84213 — Sharkron 372/373 (`:84207-84215`): ×100 dust 5.
//! - 86527 — Wandering Eye 133 (`:86521-86534`): ×100 dust 5 (the
//!   half-life gore latch `:86529-86533` is draw-free, d55's body).
//! Batch 4 (the boss/worm live arms):
//! - 87279 — Eye of Cthulhu 4 (`:87275-87284`, the 4/126/125 group):
//!   ×100 dust 5.
//! - 87345 — Servant of Cthulhu 5 (`:87341-87350`): ×50 dust 5.
//! - 87594 — the Giant Worm 10/11/12 (`:87590-87599`): ×50 dust 5.
//! - 87560 — the Devourer 7/8/9 (`:87556-87565`): ×100 dust 18,
//!   alpha/color/scale ride the NPC.
//! - 87626 — the Eater of Worlds 13/14/15 (`:87622-87631`): ×100 dust
//!   18, alpha/color/scale.
//! - 88913 — the Wyvern 87-92 (`:88909-88919`): ×50 dust 16, (0,0),
//!   scale 1.5, velocity ×1.5, noGravity.
//! - 87362 — the Wall of Flesh 113/114 (`:87358-87366`): FLAT 20 dust 5
//!   (the every-segment death fan `:81476-81484` is g60's).
//! - 87414 — the Hungry 115/116 (`:87410-87418`): FLAT 5 dust 5.
//! - 87440 — the Leech 117-119 (`:87436-87444`): FLAT 5 dust 5.
//! - 87485 — Queen Bee 222 (`:87481-87489`): ×100 dust 18,
//!   alpha/color/scale.
//! Batch 5 (the Martian/solar live arms — two shapes):
//! - 84308/84328/84368/84388/84456/84476/84508 — the martian grunts
//!   (382/390/383/385/386/389/388/399, `:84302-84533`): the
//!   `dmg/(lifeMax·50)` bound, dust 227 (dir,-1).
//! - 84348/84408/84431 — the ForceBubble 384 (`:84342-84351`, ×50), the
//!   Tesla Turret 387 (`:84402-84411`, ×20) and the Martian Walker 520
//!   (`:84425-84434`, ×20): dust 226 at (-dir,-1), post-spawn position
//!   Lerp(pos, Centre, 0.25) + scale 0.5.
//! Batch 6 (the solar pillar's gated-child live arms — ×20 dust 241 +
//! the Next-gated dust-6 child, noGravity/scale 1.5/fadeIn 1/vel ×3):
//! - 82791/82794 — 518/416 (`:82785-82800`), Next(4)==0.
//! - 82829/82832 — 419 (`:82823-82838`), Next(4)==0.
//! - 82890/82893 — 415 (`:82884-82899`), Next(4)==0.
//! - 82924/82927 — 417 (`:82918-82933`), Next(4)==0.
//! - 82965/82968 — 418 (`:82959-82974`), Next(2)==0 (the tighter gate).
//! Batch 7 (the Vortex pillar's gated-child live arms — dust 241 parent
//! + the Next-gated dust-229 child, the same four post-writes):
//! - 83045/83048 — Vortex Hornet Queen 426 (`:83039-83054`), ×50,
//!   Next(4)==0.
//! - 83092/83095 — Vortex Hornet 427 (`:83086-83101`), ×20, Next(2)==0.
//! - 83131/83134 — Vortex Soldier 429 (`:83125-83140`), ×20, Next(4)==0.
//! - 83213/83216 — Vortex Rifleman 425 (`:83207-83222`), ×20, Next(4)==0.
//! - 83253 — Vortex Larva 428 (`:83247-83254`), ×20, NO child gate live
//!   (the 229/240 selector is death-half only).
//! Batch 8 (the Nebula pillar's gated-child live arms — dust 249 parent
//! + the Next-gated dust-242 child, the same four post-writes):
//! - 83280/83283 — Nebula Soldier 424 (`:83274-83289`), ×20, Next(4)==0.
//! - 83318/83321 — Nebula Beast 423 (`:83312-83327`), ×50, Next(4)==0.
//! - 83351/83354 — Nebula Headcrab 421 (`:83345-83360`), ×20, Next(2)==0.
//! - 83384/83387 — Nebula Brain 420 (`:83378-83393`), ×20, Next(2)==0.
//! Batch 9 (the Stardust pillar's live arms):
//! - 83419 — the Stardust Cells 405/406 (`:83411-83433`): the
//!   `10 + dmg/10` ring, dust 229 in the width/4 centre box; the DEAD
//!   vector10's NextDouble + Next(24,41) draws are KEPT (computed never
//!   used — the d50 235 precedent), the position re-anchors on the
//!   normalized spawn offset × num161 × scale − 4, velocity ×2 (i<30)
//!   or the Next(45,91) draw, scale 0.7 + NextFloat.
//! - 83487/83490 — Stardust Spider Big 409 (`:83481-83492`): ×50 dust
//!   17 + Next(4)==0 → dust-229 noGravity child (NO other post-writes).
//! - 83514/83517 — Spider Small 410 (`:83508-83519`): ×10, same shape.
//! - 83538/83541 — Jellyfish Big 407 (`:83532-83543`): ×50 dust 17
//!   scale 0.75 Transparent + Next(2)==0 child.
//! - 83640/83643 — Stardust Soldier 411 (`:83634-83645`): ×20, the
//!   Next(4) shape.
//! RESOLVED (the g60 ring landed): the Stardust Worm 402's LIVE arm
//! (`:83565-83598`, batch 34) now rides `Npc.old_pos` — the head pair
//! per ×3 iteration, then the per-slot Next(4)-gated trail with the
//! zero-break INSIDE the gate; g60 owns the DEATH half (`:83599-83632`,
//! a different site — the keep-one merge composes).
//! Batch 10 (the desert/misc live arms):
//! - 83764/83767 — the sandwalker group 524-527 (`:83740-83773`): ×20
//!   dust 5 + Next(4)==0 && child>0 → the per-type child (524 has NONE —
//!   the gate draw still rides), the four post-writes.
//! - 83809 — 528/529 (`:83797-83810`): ×20 dust 5, gate-free.
//! - 83830 — 530/531 (`:83824-83831`): ×30 dust 250, alpha 100.
//! - 83852 — Desert Spirit 532 (`:83846-83853`): ×50 dust 5, alpha 100.
//! - 83873 — Desert Djinn 533 (`:83867-83876`): ×50 dust 27, (0,0),
//!   alpha 50 scale 1.5, velocity ×2 + noGravity (the death fadeIn is
//!   g60's).
//! - 83895 — 460 (`:83889-83896`): ×100 dust 5.
//! - 83917/83939 — 477/479 (`:83911-83940`): ×100 dust 237.
//! Batch 11 (the critter/zombie-variant live arms):
//! - 86648 — 252 (`:86642-86650`): ×100 dust 5.
//! - 86665 — 489/586 (`:86659-86667`): ×100 dust 5.
//! - 86696 — 534 (`:86690-86698`): ×100 dust 5.
//! - 86716 — 490 (`:86710-86718`): ×100 dust 5.
//! - 86736 — the shark group 65/542-545/692 (`:86730-86738`): ×150
//!   dust 5.
//! - 86793 — 195/196 (`:86787-86795`): ×150 dust 5.
//! - 86811 — 198/199 (`:86807-86815`): ×150 dust 5.
//! - 86834 — 206 (`:86828-86836`): ×150 dust 137.
//! - 86854 — 342 (`:86848-86856`): ×100 dust 0.
//! - 86874 — the Zombie Elf trio 338-340 (`:86868-86876`): ×100 dust 5.
//! Batch 12 (the town-NPC live arms — ×100 dust 5, (dir,-1); the death
//! halves' shimmer gore rows are g60/d55's):
//! - 87658 — Merchant 17 (`:87652-87660`).
//! - 88288 — Nurse 18 (`:88282-88291`).
//! - 88311 — Arms Dealer 19 (`:88305-88313`).
//! - 88331 — Demolitionist 38 (`:88325-88333`).
//! - 88354 — Dryad 20 (`:88348-88356`).
//! - 88374 — Dye Trader 207 (`:88368-88376`).
//! - 88397 — Party Girl 208 (`:88391-88399`).
//! - 88082 — Painter 227 (`:88076-88084`).
//! - 88105 — Witch Doctor 228 (`:88099-88107`).
//! - 87928 — Guide 22 (`:87922-87930`).
//! Batch 13 (the mimic/skeleton-variant/golem live arms):
//! - 88417 — Princess 663 (`:88411-88419`): ×100 dust 5 (her own arm —
//!   the `:88046` town-pet 663 is unreachable).
//! - 88667 — Santa 453 (`:88661-88669`): ×50 dust 26.
//! - 88690 — the pirate group 269-276/281-286 (`:88684-88692`): ×50
//!   dust 26.
//! - 88730 — the DD2 skeleton group 277-280 (`:88724-88733`): ×100
//!   dust 174, post-spawn scale 1.1 + noGravity.
//! - 88751 — 341 (`:88744-88753`): ×50 dust 196 at (0,0).
//! - 88781 — the Mimics 85/629 (`:88766-88783`): ×50, the ai[3]
//!   selector (2 → 10, 3 → 37, else 7), (0,0).
//! - 88803 — 473-476 (`:88796-88805`): ×50 dust 31 at (0,0).
//! - 88824 — 169 (`:88818-88827`): ×50 dust 92, (0,0), scale 1.5,
//!   velocity ×1.5 + noGravity.
//! - 88844 — 170/180 (`:88838-88847`): ×50 dust 54, same shape.
//! - 88868 — 171 (`:88862-88871`): ×50 dust 16, same shape.
//! Batch 14 (the mummy/wraith/spore/serpent live arms):
//! - 88940 — the mummies 78/79/80/630 (`:88934-88943`): ×50 dust 31,
//!   (0,0), scale 1.5, velocity ×2 + noGravity.
//! - 88965 — 82/158/159/250/253 (`:88959-88968`): ×50 dust 54, (0,0),
//!   alpha 50, scale 1.5, velocity ×2 + noGravity.
//! - 88990 — 316/330 (`:88984-88992`): ×100 dust 192, alpha 100,
//!   (100,100,100,100), noGravity.
//! - 89009 — 662 (`:89003-89013`): ×100 dust 180 at (vx,vy), velocity
//!   ×2 + noGravity + scale 1.4.
//! - 89032 — 315 (`:89026-89035`): ×50 dust 54, the 88965 shape.
//! - 89059 — the FungiSpore group 254-261/634/635 (`:89053-89062`):
//!   ×50 dust 165, (0,0), alpha 50, scale 1.5, velocity ×2 + noGravity.
//! - 89105 — the Bone Serpent 39/40/41 (`:89099-89107`): ×50 dust 26.
//! - 89121/89125 — Cursed Skull 34 (`:89115-89128`): ×30 PAIRS at
//!   (-vx·0.2,-vy·0.2) — dust 15 alpha 100 scale 1.8 noLight noGravity
//!   ×1.3, then dust 26 scale 0.9 noLight ×1.3.
//! - 89150 — 289 (`:89144-89154`): ×30 dust 181, the 15-shape single.
//! Batch 15 (the pillar/confetti/Pumpkin-Moon live arms):
//! - 84693 — the lunar pillars 333-336 (`:84670-84694`): ×100 dust 4,
//!   alpha 150, the per-type tint.
//! - 84710 — 537 (`:84704-84711`): ×100 dust 32.
//! - 84732 — the confetti slime (type 1, ai[1]==1345, `:84723-84741`):
//!   the LIVE 10-dust burst (Next(139,143) type arg FIRST, scale
//!   ×(1+Next(-30,31)·0.01), vel.X += Next(-50,51)·0.05) + the 5 gore
//!   siblings' ARGUMENT/post-write draws (Next(276,283), Next(-20,21),
//!   Next(-50,51) each — the bodies are d55's), BEFORE the gel loop.
//! - 84782 — 352 (`:84776-84784`): ×100 dust 67 noGravity.
//! - 84801 — 305-314 (`:84795-84802`): ×100 dust 194, scale 1.1.
//! - 84823 — Pumpking 327 (`:84817-84824`): ×100 dust 189, scale 1.1.
//! - 84844 — Mourning Wood 325 (`:84838-84845`): ×100 dust 191, 1.1.
//! - 84873 — 344 (`:84867-84874`): ×100 dust 196.
//! - 84898 — 326 (`:84892-84899`): ×100 dust 191, scale 1.1.
//! - 84915 — 304 (`:84909-84916`): ×100 dust 189, scale 1.3.
//! Batch 16 (the remaining simple live arms + the hook selector):
//! - 85503 — 122 (`:85497-85505`): ×50 dust 72, alpha 200, vel ×1.5.
//! - 85522 — Pixie 75 (`:85516-85523`): ×50 dust 55, alpha 200,
//!   this.color.
//! - 84932 — 302 (`:84926-84933`): ×100 dust 192, alpha 100, grey.
//! - 84950 — the Golem parts 245-249 (`:84944-84951`): ×100 dust 148.
//! - 85016 — 204 (`:85010-85017`): ×100 dust 4, alpha 100,
//!   (149,208,104).
//! - 85033 — 225 (`:85027-85034`): ×100 dust 4, alpha 100, blue.
//! - 85051 — 243 (`:85045-85052`): ×150 dust 161.
//! - 85384 — Toxic Sludge 141 (`:85378-85385`): ×100 dust 4,
//!   this.alpha, (210,230,140).
//! - 85364 — the Frost Legion 143/144/145 (`:85358-85366`): ×100 dust
//!   76 noGravity.
//! - 87173/87181 — the Plantera parts 262/263/264 (`:87165-87183`):
//!   ×100 — 262 above half life gates Next(3)!=0 → 166 else 167 (the
//!   draw ONLY when type==262 && life > lifeMax/2, integer half); the
//!   `:87175` type-265 arm is dead inside this group.
//! Batch 17 (the worm/demon/town-adjacent live arms):
//! - 85248 — the Truffle Worms 374/375 (`:85244-85251`): EVERY strike
//!   (no life gate), FLAT 6 dust 165, (0,0), alpha 50 scale 1.5,
//!   velocity ×2 + noGravity.
//! - 85321 — 154 (`:85315-85324`): ×50 dust 67 at (vx,vy), alpha 90,
//!   scale 1.5, noGravity + velocity ×0.2.
//! - 89748 — the Man Eaters 43/56 (`:89742-89750`): ×100 dust 40,
//!   this.alpha/color, scale 1.2.
//! - 89765 — 175 (`:89759-89767`): ×100 dust 40, same shape.
//! - 89782 — 48 (`:89776-89784`): ×100 dust 5.
//! - 89799 — the demons 62/66 (`:89793-89801`): ×100 dust 5.
//! - 87991 — Zoologist 633 (`:87985-87993`): ×100 dust 5.
//! - 88029 — Travelling Merchant 368 (`:88023-88031`): ×100 dust 5.
//! - 87681 — Tax Collector 441 (`:87675-87683`): ×100 dust 5.
//! - 87701 — 86 (`:87695-87703`): ×100 dust 5.
//! Batch 18 (the corruption/caster live arms):
//! - 87147 — 83/84/179 (`:87141-87149`): ×50 dust 31, (0,0), scale 1.5,
//!   noGravity (the LIVE arm has NO ×2 velocity — that's death-only).
//! - 87458 — 217/218/219 (`:87452-87459`): ×100 dust 18, NPC rows.
//! - 87543 — Clinger 101 (`:87537-87544`): ×100 dust 18, NPC rows.
//! - 87579 — the Seeker 98/99/100 (`:87573-87580`): ×100 dust 18, NPC
//!   rows.
//! - 87723 — 155 (`:87717-87725`): ×100 dust 5.
//! - 87741 — 329 (`:87735-87743`): ×100 dust 5.
//! - 87759 — 163/238 (`:87753-87760`): ×100 dust 18 (plain — NO NPC
//!   rows).
//! - 87779 — 164/165 (`:87773-87780`): ×100 dust 18, plain.
//! - 89613 — Goblin Summoner 471 (`:89607-89614`): ×100 dust 5.
//! - 89633 — 472 (`:89627-89639`): ×100 dust 5 + per-dust Next(3)!=0 →
//!   noGravity, then alpha 100 ALWAYS.
//! Batch 19 (the bound-NPC/corruption misc live arms):
//! - 87799 — 239/240 (`:87793-87800`): ×100 dust 5.
//! - 87819 — 236/237 (`:87813-87820`): ×100 dust 18, plain.
//! - 87839 — 241 (`:87833-87840`): ×100 dust 5.
//! - 87859 — 242 (`:87850-87857`): ×100 dust 5.
//! - 87876 — the bound trio + Wizard 105-108 (`:87870-87877`): ×100
//!   dust 5.
//! - 87908 — 123/124 (`:87902-87909`): ×100 dust 5.
//! - 87948 — 550 (`:87942-87949`): ×100 dust 5.
//! - 87968 — the Golfers 588/589 (`:87962-87969`): ×100 dust 5.
//! - 88125 — 229 (`:88119-88126`): ×100 dust 5.
//! - 88159 — 142 (`:88153-88160`): ×100 dust 5.
//! Batch 20 (the remaining town/misc live arms — ×100 dust 5):
//! - 88182 — 178 (`:88176-88183`).
//! - 88204 — the Stylists 353/354 (`:88198-88205`).
//! - 88228 — 37 (`:88222-88229`).
//! - 88248 — Clothier 54 (`:88242-88249`).
//! - 88268 — 160 (`:88262-88269`): ×100 dust 26 (NOT 5).
//! - 88434 — 209 (`:88428-88435`).
//! - 88454 — the pirates 212-216 (`:88448-88455`).
//! - 88507 — 220/221 (`:88503-88510`): ×50 dust 151, scale 0.7.
//! - 86613 — Vulture 61 (`:86607-86614`): ×100 dust 5.
//! - 86631 — 301 (`:86625-86632`): ×100 dust 5.
//! Batch 21 (the DD2 roster live arms — ×20):
//! - 82085 — 552-554 (`:82079-82086`): dust 5.
//! - 82116 — 561-563 (`:82110-82117`): dust 5.
//! - 82149 — 555-557 (`:82143-82150`): dust 5.
//! - 82182 — 551 (`:82172-82183`): dust 5 (the CheckProgress gate is
//!   draw-free).
//! - 82225 — 572/573 (`:82219-82226`): dust 5.
//! - 82250 — 574/575 (`:82244-82251`): dust 5.
//! - 82275 — 570/571 (`:82269-82276`): dust 5.
//! - 82298 — 568/569 (`:82292-82299`): dust 5.
//! - 82320 — 558-560 (`:82314-82321`): dust 5.
//! - 82353 — 566/567 (`:82347-82354`): dust 236.
//! (the same commit also lands 576/577 :82395 dust 273 + 564/565
//! :82432 + 578 :82461 dust 5 — the ×20 group's tail rows.)
//! Batch 22 (the DD2 boss/critter live arms):
//! - 82569/82575 — the Ogre 541 (`:82562-82581`): ×200 PAIRS of dust
//!   268, colour (222,108,48)×0.7 (XNA byte truncation → 155,75,33,178),
//!   noGravity, scale 1.5, fadeIn 0.7, velocity ×3.
//! - 82613/82619 — the Dark Mage 546 (`:82606-82625`): ×50 PAIRS, the
//!   (150,108,78)×0.7 tint (105,75,54,178), same writes.
//! - 81889 — 668 (`:81883-81890`): ×100 dust 5, scale 1.2.
//! - 81919/81923 — the Empress 636 (`:81911-81925`): ×100, the Next(2)
//!   selector BEFORE the spawn (0 → 67 else 69).
//! - 81786 — 690 (`:81782-81787`): EVERY strike, FLAT 10 dust 1, scale
//!   1.2.
//! - 82661 — 378 (`:82655-82662`): ×50 dust 7 at (0,0).
//! Batch 23 (the cultist/fairy live arms):
//! - 84113 — 379/438 (`:84107-84114`): the martian divisor
//!   `dmg/(lifeMax·50)`, dust 5.
//! - 84133 — the Cultist 439 (`:84127-84136`): ×100 dust 5, (0,0),
//!   scale 1.5, velocity ×1.5 + noGravity.
//! - 84186 — 454-459 (`:84180-84189`): ×50 dust 16, (0,0), scale 1.5,
//!   velocity ×1.5 + noGravity.
//! - 82715 — the fairies 583-585 (`:82696-82716`): ×50 dust 278
//!   NewDustDirect in the (num96+4) centre box, alpha 200,
//!   Color.Lerp(value, value2, NextFloat()) arg, scale 0.65, noGravity;
//!   the per-type HotPink/LightPink, LimeGreen/LightSeaGreen,
//!   RoyalBlue/LightBlue pairs.
//! - 82734 — 513-515 (`:82728-82735`): ×100 dust 250, NPC rows.
//! - 82753 — 510-512 (`:82747-82754`): ×100 dust 250, NPC rows.
//! - 82772 — 621-623 (`:82766-82773`): ×100 dust 5, NPC rows.
//! Batch 24 (the ×30/×20 critter groups):
//! - 85710 — 153 (`:85704-85711`): ×30 dust 5.
//! - 85729 — 177 (`:85723-85730`): ×30 dust 5.
//! - 85748 — 494/495 (`:85742-85749`): ×30 dust 5.
//! - 85766 — 496/497 (`:85760-85767`): ×30 dust 5.
//! - 85784 — 498-506 (`:85778-85785`): ×30 dust 5.
//! - 85848 — 611/689 (`:85842-85849`): ×30 dust 5.
//! - 85866 — 624 (`:85860-85867`): ×20 dust 5.
//! - 85885 — 628 (`:85879-85886`): ×20 dust 3.
//! - 85909 — 646-652 (`:85898-85910`): ×20, dust 86+(type-646) below
//!   652, else 138.
//! - 85933 — 639-645 (`:85921-85934`): ×20, dust 86+(type-639) below
//!   645, else 138.
//! Batch 25 (the critter ×20 groups):
//! - 85948 — the big critter group (`:85942-85949`, 46/55/67/74/102/
//!   224/230/297-300/303/337/538/540/587/610/614-617/625-627/615/
//!   671-675): ×20 dust 5.
//! - 86098 — 148/149/168/470 (`:86092-86099`): ×20 dust 5.
//! - 86134 — 361 (`:86128-86135`): ×20 dust 5.
//! - 86151 — 687 (`:86145-86152`): ×20 dust 16.
//! - 86165 — 685 (`:86159-86166`): ×20 dust 0.
//! - 86179 — 607 (`:86173-86180`): ×20 dust 45.
//! - 86196 — 688 (`:86190-86197`): ×20 dust 45.
//! - 86223 — the scorpions 366/367 (`:86217-86224`): ×20 dust 203.
//! - 86250 — the ducks 362-365 (`:86244-86251`): ×20 dust 5.
//! Batch 26 (the bird/holiday/antlion live arms):
//! - 86277 — the seagulls 602/603 (`:86271-86278`): ×20 dust 5.
//! - 86297 — the grebes 608/609 (`:86291-86298`): ×20 dust 5.
//! - 86318 — the Nutcrackers 348/349 (`:86312-86319`): ×20 dust 5.
//! - 86338 — Krampus 351 (`:86332-86339`): ×20 dust 5.
//! - 86358 — the Elf Archer 350 (`:86352-86359`): ×20 dust 5.
//! - 86377 — 47/57/58/464/465 (`:86371-86378`): ×20 dust 5.
//! - 86415 — 173/174/181/182/268 (`:86409-86416`): ×100 dust 5,
//!   this.alpha.
//! - 86510 — 157 (`:86504-86511`): ×100 dust 5.
//! - 86558 — the antlions 69/581/580/508/509 (`:86545-86559`): ×100
//!   dust 250.
//! - 86564 — the Larvae Antlion 582 (`:86549-86552`): ×30 dust 250
//!   (the num513 override).
//! Batch 27 (the remaining martian/saucer live arms):
//! - 84540 — the Scutlix 391 (`:84534-84541`): the martian divisor,
//!   dust 227.
//! - 84560 — the saucer parts 392-395 (`:84554-84563`): ×20 dust 228 at
//!   (-dir,-1), Centre-Lerp 0.25 + scale 0.5.
//! - 84598 — 482/483 (`:84592-84599`): the martian divisor, dust 240.
//! - 84622 — 631 (`:84616-84623`): the martian divisor, dust 1.
//! - 85399 — the Vile Spits 112/666 (`:85395-85407`): EVERY strike,
//!   FLAT 20 dust 18 at (0,0) y+2, alpha 100 scale 2; the post-spawn
//!   Next(2)==0 → scale ×0.6 + continue, else velocity ×1.4 +
//!   noGravity.
//! - 86904 — 343 (`:86898-86905`): ×100 dust 5.
//! - 86923 — 691 (`:86917-86924`): ×100 dust 5.
//! Batch 28 (the complex every-strike machines):
//! - 81822 — the Rainbow Slime 661 (`:81813-81836`): ×100 loop, the
//!   Next(5)==0 gate per iteration, hslToRgb(i/100%1,1,0.5)×0.5 colour
//!   arg, dust 267; position = Centre + NextVector2Circular(w,h) (TWO
//!   NextFloat draws), velocity = pos-Centre, noGravity, fadeIn
//!   0.6+NextFloat·0.7, scale 0.35; the CloneDust sibling (scale/2,
//!   fadeIn ×0.85, white×0.5) when the slot is not the sentinel.
//! - 81868 — 521 (`:81862-81871`): ×200 dust-228 NewDustDirect at
//!   (6·dir, vy), Transparent, scale 3, noGravity + velocity ×5.
//! - 82998/83003 — the Solar Flare 516 (`:82994-83006`): EVERY strike,
//!   FLAT 20 dust 6 at (2.5·dir,-2.5), velocity ×3, scale +=
//!   NextFloat; the Next(2)==0 → dust 259 noGravity sibling.
//! - 82018 — the Queen Slime minions 658-660 (`:81991-82023`): LIVE
//!   FLAT 7, dust 4 (659 → 243 default colour), (2·dir,-1), alpha 80,
//!   the per-type tint, post-spawn Next(3)!=0 → noGravity (the death
//!   40-count halves are g60's).
//! - 82051 — 667 (`:82025-82056`): LIVE FLAT 7 dust 10, alpha 80, scale
//!   1.1, the same gate; the LIVE 3-gore sibling's argument draws
//!   (Next(1,10) + Next(4) each, `:82043-82047`) ride BEFORE the dust
//!   loop (bodies d55's).
//! - 82070 — 676 (`:82058-82077`): LIVE FLAT 8 dust 310, alpha 80,
//!   scale 1.1, the gate, then velocity ×1.5 + base.velocity·0.1.
//! Batch 29 (the Queen Slime herself + the crystal ring):
//! - 81980 — the Queen Slime 657 (`:81974-81988`): LIVE FLAT 12 dust 4,
//!   (2·dir,-1), alpha 50, the GetDustColor double-Lerp (TWO NextFloat
//!   draws: inner grey↔pink first, outer blue↔inner second,
//!   `NPC.cs:45121-45124`), A=150; velocity ×0.3 + base.velocity·0.3;
//!   Next(3)==0 → noGravity + scale ×1.2.
//! - 83031 — the Eternia Crystal 522 (`:83024-83037`): EVERY strike,
//!   the RotatedByRandom(π) spinning point (TWO NextDouble draws,
//!   Utils.cs:1697-1700), then FLAT 24 dust 66 in the 0×0 Centre box,
//!   position = Centre, velocity = spin.RotatedBy(2π·i/24) × (1.05,1)
//!   × (0.8+NextFloat·0.4) × 2, SkyBlue, noGravity, scale +=
//!   0.5+NextFloat.
//! RESOLVED (batch 33/34): the 519 oldPos trail (`:83008-83022`) and the
//! 402 oldPos trail (`:83579-83596`) ride the g60-adopted `Npc.old_pos`
//! ring now (`update_trails`'s mode-7 shift; both 519 and 402 map to
//! mode 7, `NPCID.cs:10430`). The 661/657 DEATH halves, the 371 bubble ring
//! (`:84239`), the 686/594 gore-only deaths and the daybreak prologue
//! (`:81735`) are g59/g60's death lane.
//! Batch 30 (the census tail — the last simple live rows):
//! - 89660 — 480 (`:89654-89661`): ×100 dust 5.
//! - 89680 — 185 (`:89674-89681`): ×100 dust 5.
//! - 89821 — the chain's final else 156 (`:89811-89822`): ×100 dust 5.
//! - 87244 — the Brain of Cthulhu 266 (`:87238-87245`): ×100 dust 5.
//! - 87265 — the Creeper 267 (`:87259-87266`): ×50 dust 5.
//! - 84643 — the gold critters (`NPCID.Sets.IsGoldCritter`,
//!   `NPCID.cs:10126`: 442-448/539/592/593/601/605/613/627,
//!   `:84637-84644`): FLAT 10, the Next(232,234) type arg per dust.
//! Batch 31 (the DD2 beast Direct bursts + the solar/critter tail):
//! - 82482 — 620 (`:82476-82485`): ×150 dust-5 NewDustDirect, velocity
//!   ×2.5, scale 1.7.
//! - 82508 — 619 (`:82502-82511`): ×150 Direct, velocity ×2.5, fadeIn
//!   1.7 (NOT scale).
//! - 82534 — 618 (`:82528-82537`): ×200 Direct, velocity ×2.5, scale 2
//!   (the death half's 2.5 is g60's).
//! - 82868 — the Solar Crawltipedes 412-414 (`:82862-82869`): ×50 dust
//!   241 (dir,-1), no child gate live.
//! - 83959 — 478 (`:83953-83961`): FLAT 10 dust 238, noGravity.
//! - 83981 — 466 (`:83975-83982`): ×100 dust 5.
//! - 84003 — 468 (`:83997-84004`): ×100 dust 5.
//! - 84025 — 469 (`:84019-84026`): ×100 dust 5.
//! - 84047 — 463 (`:84041-84048`): ×100 dust 5.
//! - 84069 — 461 (`:84063-84070`): ×100 dust 5.
//! Batch 32 (the final live census tail):
//! - 84091 — 462 (`:84085-84092`): ×100 dust 5.
//! - 84287 — 381 (`:84281-84288`): the martian divisor, dust 227.
//! - 87612 — the EoW segments 95/96/97 (`:87606-87613`): ×50 dust 5.
//! - 89173 — the Skeletron hands 35/36/68 (`:89167-89174`): ×100 dust
//!   26.
//! - 88648 — the Deerclops 694 (`:88641-88650`): NO dust — the LIVE
//!   arm's gore siblings' ARGUMENT draws ride here (the clamped
//!   `max(1,min(5,(int)(dmg/lifeMax·3)))` count, 2 NextFloat per gore;
//!   the bodies are d55's).
//! Batch 33 (the unblocked oldPos trails — the g60 ring landed):
//! - 83017 — the SolarGoop 519 (`:83008-83023`): EVERY strike (a
//!   standalone if, NO life gate), the hitbox walks every 3rd oldPos
//!   slot (`+= 3`, (int) truncation, NO zero-break — an unfilled slot
//!   spawns at (0,0)), FLAT 5 per slot: the SelectRandom(6,259,158)
//!   Next(3) arg FIRST (Utils.cs:2459-2462), (2.5·dir,-2.5), alpha 200,
//!   velocity ×2.4, scale += NextFloat. Mode census: 519→7, the SAME
//!   TrailingMode row as 402 (`NPCID.cs:10430`).
//! Batch 34 (the 402 LIVE hit arm — the death half is g60's :83611):
//! - 83573/83577/83589/83593 — the Stardust Worm Head 402 (`:83565-
//!   83598`): ×(dmg/lifeMax·3); per iteration the Next(3)==0 head 17
//!   (dir,-1, Transparent, scale 0.75) + Next(2)==0 head 229 noGravity,
//!   then EVERY oldPos slot draws Next(4); on ==0 a zero slot BREAKS
//!   (inside the gate — later slots draw nothing), else the same
//!   17/229 gated pair AT the slot (raw float position, no (int) cast).
//! WRITTEN NOT RUN (code-only mode): the witnesses below are compiled
//! by the merge-verify wave, not executed here.

use glam::Vec2;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::{UnifiedRandom, World};

use crate::npc::{Npc, NpcKind};

/// The strike-moment snapshot of one `HitEffect(hitDirection, num)` call
/// (`NPC.cs:81488`): every field the dust arms read, captured AFTER the
/// damage + knockback writes. `life <= 0` snapshots are kept in the
/// queue (the g59/g60 death lane reads the same capture shape) but the
/// LIVE replay below emits nothing for them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HitDustSnapshot {
    /// `this.type` — the C# NPCID (via `NpcKind::cs_id`).
    pub npc_type: i32,
    /// `this.life` at the call (post-damage; the carrier mirror already
    /// landed, `NPC.cs:81374-81378`).
    pub life: i32,
    pub life_max: i32,
    /// The `dmg` argument — `StrikeNPC_Inner`'s final `num`.
    pub dmg: f64,
    pub hit_dir: i32,
    /// `base.position` / `base.width` / `base.height` (the C# ints).
    pub x: f32,
    pub y: f32,
    pub w: i32,
    pub h: i32,
    /// `base.Center` (the martian Lerp arms read it, `NPC.cs:84349`).
    pub cx: f32,
    pub cy: f32,
    /// `base.velocity` post-knockback (the FireImp/sphere arms read it).
    pub vx: f32,
    pub vy: f32,
    /// `this.alpha` / `this.color` / `this.scale` / `base.wet`.
    pub alpha: i32,
    pub color: Color,
    pub scale: f32,
    pub wet: bool,
    /// `this.ai[3]` — the Mimic's chest-type selector (`NPC.cs:88769-
    /// 88776`: 2 → dust 10, 3 → dust 37, else 7).
    pub ai3: f32,
    /// `this.ai[1]` — the confetti-slime gate (`NPC.cs:84723`: the
    /// type-1 slime with ai[1] == 1345 bursts confetti BEFORE its gel).
    pub ai1: f32,
    /// `this.oldPos` (`NPC.cs:6119`) — the trail ring the 519 SolarGoop
    /// arm walks (`:83011-83014`, every 3rd slot, no zero-break) and the
    /// 402 worm head's hit loop gates on (`:83579-83596`). Captured at
    /// strike time like every other field; `Npc::update_trails`'s mode-7
    /// shift fills it (`NPCID.cs:10430`: 402→7 AND 519→7). The g60
    /// 10-slot shape verbatim — the C#'s per-type TrailCacheLength
    /// (402→36, 519→20, `NPCID.cs:10450`) is the recorded keep-one
    /// deviation (ANALYSIS).
    pub old_pos: [Vec2; 10],
}

impl HitDustSnapshot {
    /// Capture at the strike tail (the three `StrikeNPC` entry points'
    /// shared position — after the knockback and the bubble re-arm).
    pub fn capture(n: &Npc, dmg: i32, hit_dir: i32) -> Self {
        let (w, h) = n.size();
        let (cx, cy) = n.center();
        Self {
            npc_type: n.kind.cs_id() as i32,
            life: n.hp,
            life_max: n.max_hp,
            dmg: dmg as f64,
            hit_dir,
            x: n.x,
            y: n.y,
            w: w as i32,
            h: h as i32,
            cx,
            cy,
            vx: n.vx,
            vy: n.vy,
            alpha: n.alpha as i32,
            color: hit_color(n),
            scale: n.scale,
            wet: n.wet,
            ai3: n.ai[3],
            ai1: n.ai[1],
            old_pos: n.old_pos,
        }
    }
}

/// `this.color` for the ported kinds — the `SetDefaults` census: the
/// only writers are the slime-variant rows (`NPC.cs:7685-7753` +
/// `:8583`), the MotherSlime's `:8975` and the unported 537's `:15869`;
/// every other kind keeps `this.color = default` (`NPC.cs:8526`, XNA
/// `default(Color)` = all-zero).
fn hit_color(n: &Npc) -> Color {
    match n.kind {
        NpcKind::Slime => {
            let c = n.slime.stats().color;
            Color::new(c[0], c[1], c[2], c[3])
        }
        // `NPC.cs:8975` — the MotherSlime's own row.
        NpcKind::MotherSlime => Color::new(0, 0, 0, 50),
        _ => Color::default(),
    }
}

/// `Main.hslToRgb`-free helper: `Color.Lerp` (MonoGame float lerp + the
/// truncating byte cast, the dust_table convention).
fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let ch = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t) as u8;
    Color::new(ch(a.r, b.r), ch(a.g, b.g), ch(a.b, b.b), ch(a.a, b.a))
}

/// `NPC.AI_121_QueenSlime_GetDustColor` (`NPC.cs:45121-45124`):
/// `Lerp((0,160,255), Lerp((200,200,200), (255,80,255), NextFloat()),
/// NextFloat())` — the INNER lerp's draw rides FIRST (argument
/// evaluation), then the outer's.
fn queen_slime_dust_color(rng: &mut UnifiedRandom) -> Color {
    let inner_t = rng.next_float();
    let outer_t = rng.next_float();
    let inner = lerp_color(Color::new(200, 200, 200, 255), Color::new(255, 80, 255, 255), inner_t);
    lerp_color(Color::new(0, 160, 255, 255), inner, outer_t)
}

/// The C# live-loop bound: `for (int i = 0; (double)i < dmg /
/// (double)lifeMax * mult; i++)` — f64 division FIRST, then the ×mult,
/// counted exactly as the `for` runs it (the spore-plan precedent,
/// `npc_ai_style50.rs`).
fn hit_count(dmg: f64, life_max: i32, mult: f64) -> u32 {
    let bound = dmg / life_max as f64 * mult;
    let mut count = 0u32;
    while (count as f64) < bound {
        count += 1;
    }
    count
}

/// One `Dust.NewDust(base.position, width, height, Type, hitDirection,
/// -1f)`-shaped spawn (the common live-arm body). Post-spawn writes are
/// the caller's; the sentinel takes them like the C#.
#[allow(clippy::too_many_arguments)]
fn live_dust(
    s: &HitDustSnapshot,
    dust_type: i32,
    speed_x: f32,
    speed_y: f32,
    alpha: i32,
    color: Color,
    scale: f32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) -> usize {
    dust.new_dust(
        rng,
        ctx,
        world,
        Vec2::new(s.x, s.y),
        s.w,
        s.h,
        dust_type,
        speed_x,
        speed_y,
        alpha,
        color,
        scale,
    )
}

/// Replay one captured strike's LIVE dust arm on the caller's stream.
/// The arm match mirrors the C# chain's exclusivity: every ported type
/// belongs to exactly one arm (the giant `else if` chain, `NPC.cs:85560-
/// 89832`, plus the standalone slime `if` at `:84721`), so a per-type
/// match is faithful. Types outside the ledger emit nothing — their
/// arms land with the later g62 batches or the g59/g60 death lane.
pub fn replay_hit_dust(
    s: &HitDustSnapshot,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let dir = s.hit_dir as f32;
    // `:89526-89548`/`:89690-89700` — the three spheres: EVERY strike
    // (live or dead — the arms carry NO life gate, so the killing blow's
    // HitEffect still bursts; they are strike arms, not death-event
    // arms). 20 pairs at the reflected velocity; the Item10 cues
    // (`:89528`/`:89540`/`:89692`) are c13's sound lane. Pair shape:
    // first noGravity ×2 at scale A, second ×2 at scale B.
    let sphere = match s.npc_type {
        25 => Some((6, 2.0, 1.0)),    // `:89531`/`:89534`
        33 => Some((172, 1.2, 0.6)),  // `:89543`/`:89546`
        30 | 665 => Some((27, 2.0, 1.0)), // `:89695`/`:89698`
        _ => None,
    };
    if let Some((dust_type, scale_a, scale_b)) = sphere {
        for _ in 0..20 {
            let idx = live_dust(
                s, dust_type, -s.vx * 0.2, -s.vy * 0.2, 100, Color::default(), scale_a,
                rng, dust, ctx, world,
            );
            let d = dust.slot_mut(idx);
            d.no_gravity = true;
            d.velocity *= 2.0;
            let idx = live_dust(
                s, dust_type, -s.vx * 0.2, -s.vy * 0.2, 100, Color::default(), scale_b,
                rng, dust, ctx, world,
            );
            dust.slot_mut(idx).velocity *= 2.0;
        }
        return;
    }
    // `:85244-85251` — the Truffle Worms: EVERY strike (no life gate),
    // FLAT 6 dust 165, (0,0), alpha 50 scale 1.5, velocity ×2 +
    // noGravity.
    if s.npc_type == 374 || s.npc_type == 375 {
        for _ in 0..6 {
            let idx = live_dust(s, 165, 0.0, 0.0, 50, Color::default(), 1.5, rng, dust, ctx, world);
            let d = dust.slot_mut(idx);
            d.velocity *= 2.0;
            d.no_gravity = true;
        }
        return;
    }
    // `:81782-81787` — 690: EVERY strike, FLAT 10 dust 1, (dir,-1),
    // scale 1.2.
    if s.npc_type == 690 {
        for _ in 0..10 {
            live_dust(s, 1, dir, -1.0, 0, Color::default(), 1.2, rng, dust, ctx, world);
        }
        return;
    }
    // `:85395-85407` — the Vile Spits: EVERY strike, FLAT 20 dust 18 at
    // (x, y+2), (0,0), alpha 100 scale 2; the post-spawn Next(2): 0 →
    // scale ×0.6 + continue, else velocity ×1.4 + noGravity.
    if s.npc_type == 112 || s.npc_type == 666 {
        for _ in 0..20 {
            let idx = dust.new_dust(
                rng, ctx, world, Vec2::new(s.x, s.y + 2.0), s.w, s.h, 18,
                0.0, 0.0, 100, Color::default(), 2.0,
            );
            let d = dust.slot_mut(idx);
            if rng.next_max(2) == 0 {
                d.scale *= 0.6; // `:85402`
                continue; // `:85403`
            }
            d.velocity *= 1.4; // `:85405`
            d.no_gravity = true; // `:85406`
        }
        return;
    }
    // `:82994-83006` — the Solar Flare: EVERY strike, FLAT 20 dust 6 at
    // (2.5·dir, -2.5), velocity ×3, scale += NextFloat; the Next(2)==0
    // sibling spawns dust 259 noGravity.
    if s.npc_type == 516 {
        for _ in 0..20 {
            let idx = live_dust(s, 6, 2.5 * dir, -2.5, 0, Color::default(), 1.0, rng, dust, ctx, world);
            let d = dust.slot_mut(idx);
            d.velocity *= 3.0; // `:82999`
            d.scale += rng.next_float(); // `:83000`
            if rng.next_max(2) == 0 {
                // `:83003` — the sibling's own internals roll.
                let sib = live_dust(s, 259, 2.5 * dir, -2.5, 0, Color::default(), 1.0, rng, dust, ctx, world);
                dust.slot_mut(sib).no_gravity = true; // `:83004`
            }
        }
        return;
    }
    // `:83008-83023` — the SolarGoop 519: EVERY strike (a standalone if,
    // NO life gate). The hitbox walks every 3rd oldPos slot (`+= 3` over
    // the ring's length), X/Y take the C# `(int)` truncation of the
    // slot; NO zero-break — an unfilled slot spawns at (0,0) exactly
    // like the C#. Per slot FLAT 5: the Type argument's SelectRandom
    // (6, 259, 158) draws Next(3) FIRST (Utils.cs:2459-2462), then the
    // spawn at hitbox2.TopLeft() with (2.5·dir, -2.5), alpha 200,
    // velocity ×2.4, scale += NextFloat.
    if s.npc_type == 519 {
        for slot_idx in (0..s.old_pos.len()).step_by(3) {
            let slot = s.old_pos[slot_idx];
            // `:83013-83014` — `hitbox2.X = (int)oldPos[i].X` (truncate).
            let pos = Vec2::new(slot.x as i32 as f32, slot.y as i32 as f32);
            for _ in 0..5 {
                // `:83017` — the Type argument draw rides FIRST.
                let t = [6, 259, 158][rng.next_max(3) as usize];
                let idx = dust.new_dust(
                    rng, ctx, world, pos, s.w, s.h, t,
                    2.5 * dir, -2.5, 0, Color::default(), 1.0,
                );
                let d = dust.slot_mut(idx);
                d.alpha = 200; // `:83018`
                d.velocity *= 2.4; // `:83019`
                d.scale += rng.next_float(); // `:83020`
            }
        }
        return;
    }
    // `:81991-82077` — the Queen Slime minion trio + 667 + 676: the
    // LIVE halves (FLAT 7/7/8, alpha 80, the post-spawn Next(3)!=0 →
    // noGravity). The death halves' 40-count bursts are g60's lane, so
    // this block gates on life > 0 (unlike the life-independent sphere/
    // flare arms above). 667's LIVE 3-gore siblings' ARGUMENT draws ride
    // BEFORE the dust loop (`:82043-82047`, the bodies d55's).
    if s.life > 0 && matches!(s.npc_type, 658..=660 | 667 | 676) {
        let (count, t, tint, sc) = match s.npc_type {
            658 => (7, 4, Color::new(78, 136, 255, 150), 1.0),
            659 => (7, 243, Color::default(), 1.0),
            660 => (7, 4, Color::new(150, 100, 200, 150), 1.0),
            667 => (7, 10, Color::default(), 1.1),
            _ => (8, 310, Color::default(), 1.1), // 676
        };
        if s.npc_type == 667 {
            // `:82043-82047` — 3 × (Next(1,10) then Next(4)); the
            // velocity expressions consume them in argument order.
            for _ in 0..3 {
                let _ = rng.next_range(1, 10);
                let _ = rng.next_max(4);
            }
        }
        for _ in 0..count {
            let idx = live_dust(s, t, 2.0 * dir, -1.0, 80, tint, sc, rng, dust, ctx, world);
            let d = dust.slot_mut(idx);
            if rng.next_max(3) != 0 {
                d.no_gravity = true; // `:82019-82022` etc.
            }
            if s.npc_type == 676 {
                d.velocity *= 1.5; // `:82075`
                d.velocity += Vec2::new(s.vx, s.vy) * 0.1; // `:82076`
            }
        }
        return;
    }
    // `:83024-83037` — the Eternia Crystal's ring: EVERY strike. The
    // spinning point's RotatedByRandom(π) draws TWO NextDoubles (left
    // operand first, Utils.cs:1697-1700); then FLAT 24 dust 66 in the
    // 0×0 Centre box, position = Centre, the per-i rotated velocity,
    // SkyBlue, noGravity, scale += 0.5 + NextFloat.
    if s.npc_type == 522 {
        let max = std::f64::consts::PI;
        let angle = rng.next_double() * max - rng.next_double() * max;
        let spin = crate::bow_launch::rotated_by(Vec2::new(0.0, -3.0), angle);
        for i in 0..24u32 {
            let idx = dust.new_dust(
                rng, ctx, world, Vec2::new(s.cx, s.cy), 0, 0, 66,
                0.0, 0.0, 0, Color::TRANSPARENT, 1.0,
            );
            let d = dust.slot_mut(idx);
            d.position = Vec2::new(s.cx, s.cy); // `:83032`
            // `:83033` — RotatedBy((float)(π·2·i/24)).
            let a = (std::f32::consts::PI * 2.0 * i as f32 / 24.0) as f64;
            let v = crate::bow_launch::rotated_by(spin, a);
            let f = 0.8 + rng.next_float() * 0.4;
            d.velocity = v * Vec2::new(1.05, 1.0) * f * 2.0;
            d.color = Color::new(135, 206, 235, 255); // Color.SkyBlue `:83034`
            d.no_gravity = true; // `:83035`
            d.scale += 0.5 + rng.next_float(); // `:83036`
        }
        return;
    }
    // `:81974-81988` — the Queen Slime's LIVE shimmer: FLAT 12 dust 4,
    // (2·dir,-1), alpha 50, the GetDustColor double-Lerp (A=150),
    // velocity ×0.3 + base.velocity·0.3, Next(3)==0 → noGravity +
    // scale ×1.2. The death half's 150-count burst is g60's.
    if s.npc_type == 657 && s.life > 0 {
        for _ in 0..12 {
            let mut tint = queen_slime_dust_color(rng);
            tint.a = 150; // `:81979`
            let idx = live_dust(s, 4, 2.0 * dir, -1.0, 50, tint, 1.0, rng, dust, ctx, world);
            let d = dust.slot_mut(idx);
            d.velocity *= 0.3; // `:81981`
            d.velocity += Vec2::new(s.vx, s.vy) * 0.3; // `:81982`
            if rng.next_max(3) == 0 {
                d.no_gravity = true; // `:81985`
                d.scale *= 1.2; // `:81986`
            }
        }
        return;
    }
    // The LIVE lane's gate: every remaining arm sits under `if (this.life
    // > 0)` and RETURNS; the death halves are g59/g60's.
    if s.life <= 0 {
        return;
    }
    // `:84637-84644` — the gold critters (`NPCID.Sets.IsGoldCritter`,
    // `NPCID.cs:10126`): a STANDALONE if that runs BEFORE the giant
    // else-if chain — FLAT 10, the Next(232,234) type argument draws
    // before each spawn. Type 627 ALSO sits in the chain's `:85942`
    // group, so it falls through (both arms fire, gold first); every
    // other gold type has no chain arm and returns.
    if matches!(s.npc_type, 442..=448 | 539 | 592 | 593 | 601 | 605 | 613 | 627) {
        for _ in 0..10 {
            let t = rng.next_range(232, 234);
            live_dust(s, t, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
        }
        if s.npc_type != 627 {
            return;
        }
    }
    match s.npc_type {
        // `:84721-84749` — the slime family (1/16/71/244/535).
        1 | 16 | 71 | 244 | 535 => {
            // `:84723-84741` — the confetti slime's burst rides BEFORE
            // the gel loop (live num314 = 5 → 10 dusts + 5 gore siblings).
            if s.npc_type == 1 && s.ai1 == 1345.0 {
                for _ in 0..10 {
                    // `:84732` — the Type argument draws FIRST.
                    let t = rng.next_range(139, 143);
                    let idx = live_dust(s, t, 0.0, -10.0, 0, Color::default(), 1.2, rng, dust, ctx, world);
                    let d = dust.slot_mut(idx);
                    d.scale *= 1.0 + rng.next_range(-30, 31) as f32 * 0.01; // `:84733`
                    d.velocity.x += rng.next_range(-50, 51) as f32 * 0.05; // `:84734`
                }
                // `:84736-84741` — the 5 gore siblings: the ARGUMENT
                // draw (Next(276,283)) + the two post-write draws ride
                // here in order; the NewGore bodies are d55's.
                for _ in 0..5 {
                    let _ = rng.next_range(276, 283); // `:84738`
                    let _ = rng.next_range(-20, 21); // `:84739`
                    let _ = rng.next_range(-50, 51); // `:84740`
                }
            }
            // `:84745-84748` — ×100 dust 4, this.alpha, this.color.
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 4, dir, -1.0, s.alpha, s.color, 1.0, rng, dust, ctx, world);
            }
        }
        // `:86449-86457` — the demon-eye family.
        2 | 190 | 191 | 192 | 193 | 194 | 317 | 318 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 5, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:86937-86967` — the zombie/critter group.
        430..=436 | 590 | 591 | 3 | 52 | 53 | 536 | 104 | 109 | 331 | 332 | 132 | 161
        | 162 | 186 | 187 | 188 | 189 | 200 | 223 | 251 | 319 | 320 | 321 | 632 => {
            // `:86941-86944` — ×100 dust 5.
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 5, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
            // `:86945-86948` — the PincushionZombie's gore gate: the
            // Next(5) ALWAYS draws for 186 (the gore body is d55's).
            if s.npc_type == 186 {
                let _ = rng.next_max(5);
            }
            // `:86949-86952` — the same gate for 632.
            if s.npc_type == 632 {
                let _ = rng.next_max(5);
            }
            // `:86953-86959` — the SlimedZombie's extra ×200 blue gel,
            // height 24 (NOT base.height).
            if s.npc_type == 187 {
                for _ in 0..hit_count(s.dmg, s.life_max, 200.0) {
                    dust.new_dust(
                        rng, ctx, world, Vec2::new(s.x, s.y), s.w, 24, 4, dir, -1.0,
                        125, Color::new(0, 80, 255, 100), 1.0,
                    );
                }
            }
            // `:86960-86966` — the zombie goldfish pair's dry triple.
            if (s.npc_type == 590 || s.npc_type == 591) && !s.wet {
                for _ in 0..3 {
                    dust.new_dust(
                        rng, ctx, world, Vec2::new(s.x, s.y), s.w, 24, 6, dir, -1.0,
                        125, Color::new(0, 80, 255, 100), 1.0,
                    );
                }
            }
        }
        // `:85797-85805` — the bat group.
        49 | 51 | 93 | 150 | 152 | 226 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 30.0) {
                live_dust(s, 5, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:88527-88535` — the skeleton group (the full C# list).
        21 | 31 | 294 | 295 | 296 | 32 | 44 | 45 | 77 | 110 | 167 | 197 | 201 | 202
        | 203 | 287 | 291 | 292 | 293 | 322..=324 | 481 | 449..=452 | 693 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 50.0) {
                live_dust(s, 26, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:89550-89558` — the goblin group.
        26 | 27 | 28 | 29 | 73 | 111 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 5, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:87502-87510` — the Eater-of-Souls group.
        6 | 94 | 166 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 18, dir, -1.0, s.alpha, s.color, s.scale, rng, dust, ctx, world);
            }
        }
        // `:89702-89710` — the hornet group.
        42 | 176 | 205 | 231..=235 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 18, dir, -1.0, s.alpha, s.color, s.scale, rng, dust, ctx, world);
            }
        }
        // `:85409-85417` — the Crimslime's live crimson gel (the death
        // half's ×2 velocity is g59/g60's).
        183 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 136, 0.0, 0.0, s.alpha, s.color, 1.0, rng, dust, ctx, world);
            }
        }
        // `:85427-85435` — the CorruptSlime pair (81/121).
        81 | 121 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 14, 0.0, 0.0, s.alpha, s.color, 1.0, rng, dust, ctx, world);
            }
        }
        // `:85478-85487` — the hallow trio (120/137/138): ×50 dust 71,
        // alpha 200, post-spawn velocity ×1.5.
        120 | 137 | 138 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 50.0) {
                let idx = live_dust(s, 71, 0.0, 0.0, 200, Color::default(), 1.0, rng, dust, ctx, world);
                dust.slot_mut(idx).velocity *= 1.5; // `:85485`
            }
        }
        // `:85337-85347` — the IceSlime/UmbrellaSlime pair: ×50 PAIRS,
        // dust 76 noGravity then dust 4 with the frost tint alpha 100.
        147 | 184 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 50.0) {
                let idx = live_dust(s, 76, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
                dust.slot_mut(idx).no_gravity = true; // `:85344`
                live_dust(
                    s, 4, dir, -1.0, 100, Color::new(97, 200, 255, 100), 1.0,
                    rng, dust, ctx, world,
                ); // `:85345`
            }
        }
        // `:85534-85551` — the jellyfish trio: ×50 dust 4, the per-type
        // tint (`:85536-85544`).
        63 | 64 | 103 => {
            let tint = match s.npc_type {
                64 => Color::new(225, 70, 140, 100),
                103 => Color::new(70, 225, 140, 100),
                _ => Color::new(50, 120, 255, 100),
            };
            for _ in 0..hit_count(s.dmg, s.life_max, 50.0) {
                live_dust(s, 4, dir, -1.0, 0, tint, 1.0, rng, dust, ctx, world);
            }
        }
        // `:85562-85575` — LavaSlime/Hellbat/SpikedJungleSlime: ×80 dust
        // 6 at (2·dir, -1), this.alpha, scale 1.5; the per-dust Next(8)
        // rides AFTER the spawn internals (`:85569`), != 0 → noGravity.
        // The death half's lava-tile write (`:85584-85668`) is g59/g60's.
        59 | 60 | 151 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 80.0) {
                let idx = live_dust(
                    s, 6, 2.0 * dir, -1.0, s.alpha, Color::default(), 1.5,
                    rng, dust, ctx, world,
                );
                if rng.next_max(8) != 0 {
                    dust.slot_mut(idx).no_gravity = true; // `:85571`
                }
            }
        }
        // `:85670-85679` — King Slime's live gel: ×300 dust 4, alpha 175,
        // the blue tint (the death half's minion batch is g60's).
        50 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 300.0) {
                live_dust(
                    s, 4, dir, -1.0, 175, Color::new(0, 80, 255, 100), 1.0,
                    rng, dust, ctx, world,
                );
            }
        }
        // `:89470-89485` — Meteor Head: ×100, the Next(2) selector
        // BEFORE the spawn (0 → dust 6, else 25), then the dust-6
        // velocity pair (alpha 100, scale 2, noGravity). The death
        // halves' second 50-loop is g60's.
        23 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                let mut t = 25;
                if rng.next_max(2) == 0 {
                    t = 6; // `:89477-89480`
                }
                live_dust(s, t, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world); // `:89481`
                let idx = live_dust(
                    s, 6, s.vx * 0.2, s.vy * 0.2, 100, Color::default(), 2.0,
                    rng, dust, ctx, world,
                ); // `:89482`
                dust.slot_mut(idx).no_gravity = true; // `:89483`
            }
        }
        // `:89503-89512` — Fire Imp: ×100 dust 6 at the NPC's own
        // velocity, alpha 100 scale 2.5 noGravity.
        24 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                let idx = live_dust(
                    s, 6, s.vx, s.vy, 100, Color::default(), 2.5,
                    rng, dust, ctx, world,
                );
                dust.slot_mut(idx).no_gravity = true; // `:89510`
            }
        }
        // `:84258-84266` — Duke Fishron's live blood (the death half's
        // 150-loop + the six centre gores are g60's).
        370 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 5, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:84207-84215` — the Sharkrons' live blood (the death half's
        // 75-dust centre burst is g60's).
        372 | 373 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 5, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:86521-86534` — the Wandering Eye: ×100 dust 5; the half-life
        // gore latch (`:86529-86533`) draws NOTHING (the body is d55's).
        133 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 5, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:87275-87284` — the Eye of Cthulhu group (4/126/125 — the
        // twins share the EoC blood arm): ×100 dust 5. The death half's
        // 150-loop + the gore argument pairs are g60/d55's.
        4 | 126 | 125 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 5, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:87341-87350` — the Servant of Cthulhu: ×50 dust 5.
        5 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 50.0) {
                live_dust(s, 5, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:87590-87599` — the Giant Worm segments: ×50 dust 5.
        10 | 11 | 12 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 50.0) {
                live_dust(s, 5, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:87556-87565` — the Devourer segments: ×100 dust 18, the
        // NPC's alpha/color/scale.
        7 | 8 | 9 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 18, dir, -1.0, s.alpha, s.color, s.scale, rng, dust, ctx, world);
            }
        }
        // `:87622-87631` — the Eater of Worlds segments: ×100 dust 18.
        13 | 14 | 15 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 18, dir, -1.0, s.alpha, s.color, s.scale, rng, dust, ctx, world);
            }
        }
        // `:88909-88919` — the Wyvern segments: ×50 dust 16 at (0,0),
        // scale 1.5, post-spawn velocity ×1.5 + noGravity (the death
        // half's ×2 + the Next(1,4) gore batch are g60/d55's).
        87..=92 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 50.0) {
                let idx = live_dust(s, 16, 0.0, 0.0, 0, Color::default(), 1.5, rng, dust, ctx, world);
                let d = dust.slot_mut(idx);
                d.velocity *= 1.5; // `:88916`
                d.no_gravity = true; // `:88917`
            }
        }
        // `:87358-87366` — the Wall of Flesh + Eye: FLAT 20 dust 5 (no
        // dmg bound). The killing blow's every-segment fan
        // (`:81476-81484`) is g60's death lane.
        113 | 114 => {
            for _ in 0..20 {
                live_dust(s, 5, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:87410-87418` — the Hungry: FLAT 5 dust 5.
        115 | 116 => {
            for _ in 0..5 {
                live_dust(s, 5, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:87436-87444` — the Leech segments: FLAT 5 dust 5.
        117..=119 => {
            for _ in 0..5 {
                live_dust(s, 5, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:87481-87489` — Queen Bee: ×100 dust 18, alpha/color/scale.
        222 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 18, dir, -1.0, s.alpha, s.color, s.scale, rng, dust, ctx, world);
            }
        }
        // `:84302-84533` — the martian grunts' dust 227: the bound is
        // `dmg / (lifeMax * 50)` (the C# `(double)(this.lifeMax * 50)` —
        // the ×50 sits INSIDE the divisor, unlike the ×mult arms).
        382 | 390 | 383 | 385 | 386 | 389 | 388 | 399 => {
            let bound = s.dmg / (s.life_max * 50) as f64;
            let mut count = 0u32;
            while (count as f64) < bound {
                live_dust(s, 227, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
                count += 1;
            }
        }
        // `:84342-84351`/`:84402-84411`/`:84425-84434` — the ForceBubble
        // (×50), the Tesla Turret and the Walker (×20): dust 226 at
        // (-dir, -1), post-spawn position Lerp toward Centre 0.25 +
        // scale 0.5 (the XNA float lerp, per axis).
        384 | 387 | 520 => {
            let mult = if s.npc_type == 384 { 50.0 } else { 20.0 };
            for _ in 0..hit_count(s.dmg, s.life_max, mult) {
                let idx = live_dust(s, 226, -dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
                let d = dust.slot_mut(idx);
                d.position.x += (s.cx - d.position.x) * 0.25; // `:84349`
                d.position.y += (s.cy - d.position.y) * 0.25;
                d.scale = 0.5; // `:84350`
            }
        }
        // `:82785-82974`/`:83039-83222` — the solar/Vortex gated-child
        // arms: the dust-241 parent, then a Next(N)==0 gate per iteration
        // spawning the child (solar 6, Vortex 229) with noGravity, scale
        // 1.5, fadeIn 1, velocity ×3. Gates: Next(4) default; Next(2) for
        // 418 (`:82966`) and 427 (`:83093`); 426 rides ×50 (`:83043`).
        518 | 416 | 419 | 415 | 417 | 418 | 426 | 427 | 429 | 425 => {
            let (gate, child, mult) = match s.npc_type {
                418 => (2, 6, 20.0),
                426 => (4, 229, 50.0),
                427 => (2, 229, 20.0),
                429 | 425 => (4, 229, 20.0),
                _ => (4, 6, 20.0),
            };
            for _ in 0..hit_count(s.dmg, s.life_max, mult) {
                live_dust(s, 241, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
                if rng.next_max(gate) == 0 {
                    let idx = live_dust(s, child, 0.0, 0.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
                    let d = dust.slot_mut(idx);
                    d.no_gravity = true;
                    d.scale = 1.5;
                    d.fade_in = 1.0;
                    d.velocity *= 3.0;
                }
            }
        }
        // `:83247-83254` — the Vortex Larva: ×20 dust 241, NO live child
        // gate (the 229/240 selector lives in the death half only).
        428 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 20.0) {
                live_dust(s, 241, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:83274-83393` — the Nebula gated-child arms: dust 249 parent +
        // the dust-242 child (the same four post-writes). 423 rides ×50
        // (`:83316`); 421/420 gate Next(2) (`:83352`/`:83385`).
        424 | 423 | 421 | 420 => {
            let (gate, mult) = match s.npc_type {
                423 => (4, 50.0),
                421 | 420 => (2, 20.0),
                _ => (4, 20.0),
            };
            for _ in 0..hit_count(s.dmg, s.life_max, mult) {
                live_dust(s, 249, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
                if rng.next_max(gate) == 0 {
                    let idx = live_dust(s, 242, 0.0, 0.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
                    let d = dust.slot_mut(idx);
                    d.no_gravity = true;
                    d.scale = 1.5;
                    d.fade_in = 1.0;
                    d.velocity *= 3.0;
                }
            }
        }
        // `:83411-83433` — the Stardust Cells' live ring: the bound is
        // `10.0 + dmg / 10.0` (NO lifeMax term). Per iteration: the DEAD
        // vector10 draws FIRST (NextDouble + Next(24,41), computed never
        // used — the d50 235 precedent), then the dust-229 spawn in the
        // width/4 centre box, the normalized-offset re-anchor, the
        // velocity arm and the scale draw.
        405 | 406 => {
            let bound = 10.0 + s.dmg / 10.0;
            let mut i = 0u32;
            while (i as f64) < bound {
                let num161 = s.w / 4; // `:83417` — INTEGER division
                // `:83418` — the dead vector10: NextDouble then
                // Next(24,41), both consumed, the result never read.
                let _ = rng.next_double();
                let _ = rng.next_range(24, 41);
                let idx = dust.new_dust(
                    rng, ctx, world,
                    Vec2::new(s.cx - num161 as f32, s.cy - num161 as f32),
                    num161 * 2, num161 * 2, 229, 0.0, 0.0, 0, Color::default(), 1.0,
                ); // `:83419`
                let d = dust.slot_mut(idx);
                // `:83421-83422` — Normalize(pos - Centre); the zero
                // vector normalizes to zero (XNA).
                let off = Vec2::new(d.position.x - s.cx, d.position.y - s.cy);
                let len = off.length();
                let n = if len > 0.0 { off / len } else { Vec2::ZERO };
                d.position = Vec2::new(
                    s.cx + n.x * num161 as f32 * s.scale - 4.0,
                    s.cy + n.y * num161 as f32 * s.scale - 4.0,
                );
                if i < 30 {
                    // `:83425` — the spawn velocity's length, ×2.
                    let vlen = d.velocity.length();
                    d.velocity = n * vlen * 2.0;
                } else {
                    // `:83429` — `2f * vector11 * Next(45,91) / 10f`:
                    // the C# multiplies the vector FIRST, then divides.
                    d.velocity = (n * 2.0) * rng.next_range(45, 91) as f32 / 10.0;
                }
                d.no_gravity = true; // `:83431`
                d.scale = 0.7 + rng.next_float(); // `:83432`
                i += 1;
            }
        }
        // `:83481-83645` — the Stardust dust-17 arms: parent then the
        // Next(N)==0 dust-229 child with ONLY noGravity (no scale/fade/
        // velocity writes — unlike the solar/Vortex/Nebula children).
        // 409 ×50 Next(4), 410 ×10 Next(4), 407 ×50 Next(2) scale-0.75
        // Transparent parent, 411 ×20 Next(4).
        409 | 410 | 407 | 411 => {
            let (mult, gate) = match s.npc_type {
                409 => (50.0, 4),
                410 => (10.0, 4),
                407 => (50.0, 2),
                _ => (20.0, 4),
            };
            let parent_scale = if s.npc_type == 407 { 0.75 } else { 1.0 };
            for _ in 0..hit_count(s.dmg, s.life_max, mult) {
                live_dust(
                    s, 17, dir, -1.0, 0, Color::TRANSPARENT, parent_scale,
                    rng, dust, ctx, world,
                );
                if rng.next_max(gate) == 0 {
                    let idx = live_dust(s, 229, 0.0, 0.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
                    dust.slot_mut(idx).no_gravity = true;
                }
            }
        }
        // `:83565-83598` — the Stardust Worm Head's LIVE arm (g60 owns
        // the DEATH half `:83599-83632`, a different site): ×(dmg/
        // lifeMax·3), per iteration the Next(3)==0 head dust 17 (dir,
        // -1, Transparent, scale 0.75) + the Next(2)==0 head dust-229
        // noGravity child, THEN the oldPos trail loop — EVERY slot
        // draws Next(4) FIRST (`:83581`); on == 0 a zero slot BREAKS
        // (`:83583-83586`, inside the gate — later slots draw nothing),
        // else the SAME 17/229 gated pair spawns AT the slot
        // (`:83587-83594`, raw float position, no (int) cast here).
        402 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 3.0) {
                if rng.next_max(3) == 0 {
                    live_dust(
                        s, 17, dir, -1.0, 0, Color::TRANSPARENT, 0.75,
                        rng, dust, ctx, world,
                    ); // `:83573`
                }
                if rng.next_max(2) == 0 {
                    let idx = live_dust(s, 229, 0.0, 0.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
                    dust.slot_mut(idx).no_gravity = true; // `:83577`
                }
                for &slot in &s.old_pos {
                    if rng.next_max(4) == 0 {
                        if slot == Vec2::ZERO {
                            break; // `:83583-83586` — an unfilled slot
                        }
                        if rng.next_max(3) == 0 {
                            dust.new_dust(
                                rng, ctx, world, slot, s.w, s.h, 17,
                                dir, -1.0, 0, Color::TRANSPARENT, 0.75,
                            ); // `:83589`
                        }
                        if rng.next_max(2) == 0 {
                            let idx = dust.new_dust(
                                rng, ctx, world, slot, s.w, s.h, 229,
                                0.0, 0.0, 0, Color::default(), 1.0,
                            ); // `:83593`
                            dust.slot_mut(idx).no_gravity = true;
                        }
                    }
                }
            }
        }
        // `:83740-83773` — the sandwalker group: ×20 dust 5 + the
        // Next(4)==0 && child>0 gate. The gate draw rides for EVERY type
        // (the `&&` short-circuits on the DRAW result, not before it);
        // 524's child id is 0 → the draw happens, no spawn.
        524..=527 => {
            let child = match s.npc_type {
                525 => 75,
                526 => 170,
                527 => 67,
                _ => 0, // `:83742` — 524 has no child
            };
            for _ in 0..hit_count(s.dmg, s.life_max, 20.0) {
                live_dust(s, 5, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
                if rng.next_max(4) == 0 && child > 0 {
                    let idx = live_dust(s, child, 0.0, 0.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
                    let d = dust.slot_mut(idx);
                    d.no_gravity = true;
                    d.scale = 1.5;
                    d.fade_in = 1.0;
                    d.velocity *= 3.0;
                }
            }
        }
        // `:83797-83810` — 528/529: ×20 dust 5, no gate.
        528 | 529 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 20.0) {
                live_dust(s, 5, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:83824-83831` — the sand sharks 530/531: ×30 dust 250, alpha
        // 100 (the ChildSafety re-type lives inside new_dust).
        530 | 531 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 30.0) {
                live_dust(s, 250, dir, -1.0, 100, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:83846-83853` — the Desert Spirit: ×50 dust 5, alpha 100.
        532 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 50.0) {
                live_dust(s, 5, dir, -1.0, 100, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:83867-83876` — the Desert Djinn: ×50 dust 27 at (0,0), alpha
        // 50 scale 1.5, velocity ×2 + noGravity (the death half's fadeIn
        // 1 is g60's — the LIVE arm writes none).
        533 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 50.0) {
                let idx = live_dust(s, 27, 0.0, 0.0, 50, Color::default(), 1.5, rng, dust, ctx, world);
                let d = dust.slot_mut(idx);
                d.velocity *= 2.0; // `:83874`
                d.no_gravity = true; // `:83875`
            }
        }
        // `:83889-83896` — 460: ×100 dust 5.
        460 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 5, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:83911-83940` — the tomb crawlers 477/479: ×100 dust 237.
        477 | 479 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 237, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:86642-86718` — the ×100 dust-5 critter/zombie arms (252,
        // 489/586, 534, 490): the death halves' gore batches are g60/d55's.
        252 | 489 | 586 | 534 | 490 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 5, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:86730-86815` — the ×150 dust-5 groups: the shark family
        // (65/542-545/692) and the blood-zombie pairs (195/196, 198/199).
        65 | 542..=545 | 692 | 195 | 196 | 198 | 199 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 150.0) {
                live_dust(s, 5, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:86828-86836` — 206: ×150 dust 137.
        206 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 150.0) {
                live_dust(s, 137, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:86848-86856` — 342: ×100 dust 0 (the plain smoke).
        342 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 0, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:86868-86876` — the Zombie Elf trio: ×100 dust 5.
        338..=340 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 5, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // The town-NPC live arms (`:87658`/`:88288`/`:88311`/`:88331`/
        // `:88354`/`:88374`/`:88397`/`:88082`/`:88105`/`:87928`): ×100
        // dust 5 — the death halves' 2.5× shimmer gore rows are g60/d55's.
        // NOTE 663 (Princess) has its OWN arm at `:88411` (no early
        // return) — the `:88046` town-pet arm's 663 is unreachable.
        17 | 18 | 19 | 38 | 20 | 207 | 208 | 227 | 228 | 22 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 5, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:88411-88419` — the Princess's own arm (×100 dust 5).
        663 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 5, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:88661-88692` — Santa + the pirate group: ×50 dust 26.
        453 | 269..=276 | 281..=286 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 50.0) {
                live_dust(s, 26, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:88724-88733` — the DD2 skeletons: ×100 dust 174, post-spawn
        // scale 1.1 + noGravity (the death half's 1.5·dir/-0.5 ×2/1.4 is
        // g60's).
        277..=280 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                let idx = live_dust(s, 174, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
                let d = dust.slot_mut(idx);
                d.scale = 1.1; // `:88731`
                d.no_gravity = true; // `:88732`
            }
        }
        // `:88744-88753` — 341: ×50 dust 196 at (0,0).
        341 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 50.0) {
                live_dust(s, 196, 0.0, 0.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:88766-88783` — the Mimics: ×50, the ai[3] chest-type selector
        // (`:88768-88776`), (0,0).
        85 | 629 => {
            let mut t = 7;
            if s.ai3 == 2.0 {
                t = 10;
            }
            if s.ai3 == 3.0 {
                t = 37;
            }
            for _ in 0..hit_count(s.dmg, s.life_max, 50.0) {
                live_dust(s, t, 0.0, 0.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:88796-88805` — the 473-476 group: ×50 dust 31 at (0,0).
        473..=476 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 50.0) {
                live_dust(s, 31, 0.0, 0.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:88818-88871` — the golem-part trio: ×50 at (0,0), scale 1.5,
        // velocity ×1.5 + noGravity (169 → 92, 170/180 → 54, 171 → 16).
        169 | 170 | 180 | 171 => {
            let t = match s.npc_type {
                169 => 92,
                170 | 180 => 54,
                _ => 16,
            };
            for _ in 0..hit_count(s.dmg, s.life_max, 50.0) {
                let idx = live_dust(s, t, 0.0, 0.0, 0, Color::default(), 1.5, rng, dust, ctx, world);
                let d = dust.slot_mut(idx);
                d.velocity *= 1.5;
                d.no_gravity = true;
            }
        }
        // `:88934-88943` — the mummies: ×50 dust 31, (0,0), scale 1.5,
        // velocity ×2 + noGravity.
        78 | 79 | 80 | 630 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 50.0) {
                let idx = live_dust(s, 31, 0.0, 0.0, 0, Color::default(), 1.5, rng, dust, ctx, world);
                let d = dust.slot_mut(idx);
                d.velocity *= 2.0;
                d.no_gravity = true;
            }
        }
        // `:88959-88968`/`:89026-89035` — the wraith group + 315: ×50
        // dust 54, (0,0), alpha 50, scale 1.5, velocity ×2 + noGravity.
        82 | 158 | 159 | 250 | 253 | 315 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 50.0) {
                let idx = live_dust(s, 54, 0.0, 0.0, 50, Color::default(), 1.5, rng, dust, ctx, world);
                let d = dust.slot_mut(idx);
                d.velocity *= 2.0;
                d.no_gravity = true;
            }
        }
        // `:88984-88992` — 316/330: ×100 dust 192, alpha 100, the grey
        // tint, noGravity.
        316 | 330 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                let idx = live_dust(
                    s, 192, dir, -1.0, 100, Color::new(100, 100, 100, 100), 1.0,
                    rng, dust, ctx, world,
                );
                dust.slot_mut(idx).no_gravity = true; // `:88991`
            }
        }
        // `:89003-89013` — 662: ×100 dust 180 at the NPC's own velocity,
        // velocity ×2 + noGravity + scale 1.4.
        662 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                let idx = live_dust(s, 180, s.vx, s.vy, 0, Color::default(), 1.0, rng, dust, ctx, world);
                let d = dust.slot_mut(idx);
                d.velocity *= 2.0;
                d.no_gravity = true;
                d.scale = 1.4;
            }
        }
        // `:89053-89062` — the FungiSpore group: ×50 dust 165, (0,0),
        // alpha 50, scale 1.5, velocity ×2 + noGravity.
        254..=261 | 634 | 635 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 50.0) {
                let idx = live_dust(s, 165, 0.0, 0.0, 50, Color::default(), 1.5, rng, dust, ctx, world);
                let d = dust.slot_mut(idx);
                d.velocity *= 2.0;
                d.no_gravity = true;
            }
        }
        // `:89099-89107` — the Bone Serpent segments: ×50 dust 26.
        39 | 40 | 41 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 50.0) {
                live_dust(s, 26, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:89115-89128` — the Cursed Skull: ×30 PAIRS at the reflected
        // velocity — dust 15 (alpha 100, scale 1.8, noLight+noGravity,
        // ×1.3) then dust 26 (scale 0.9, noLight, ×1.3).
        34 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 30.0) {
                let idx = live_dust(
                    s, 15, -s.vx * 0.2, -s.vy * 0.2, 100, Color::default(), 1.8,
                    rng, dust, ctx, world,
                );
                let d = dust.slot_mut(idx);
                d.no_light = true; // `:89122`
                d.no_gravity = true; // `:89123`
                d.velocity *= 1.3; // `:89124`
                let idx = live_dust(
                    s, 26, -s.vx * 0.2, -s.vy * 0.2, 0, Color::default(), 0.9,
                    rng, dust, ctx, world,
                ); // `:89125`
                let d = dust.slot_mut(idx);
                d.no_light = true; // `:89126`
                d.velocity *= 1.3; // `:89127`
            }
        }
        // `:89144-89154` — 289: ×30 dust 181, the 15-shape single.
        289 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 30.0) {
                let idx = live_dust(
                    s, 181, -s.vx * 0.2, -s.vy * 0.2, 100, Color::default(), 1.8,
                    rng, dust, ctx, world,
                );
                let d = dust.slot_mut(idx);
                d.no_light = true;
                d.no_gravity = true;
                d.velocity *= 1.3;
            }
        }
        // `:84670-84694` — the lunar pillars: ×100 dust 4, alpha 150,
        // the per-type tint (`:84672-84688`).
        333..=336 => {
            let tint = match s.npc_type {
                333 => Color::new(73, 147, 221, 200),
                334 => Color::new(24, 146, 79, 200),
                335 => Color::new(194, 46, 50, 200),
                _ => Color::new(228, 200, 1, 200),
            };
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 4, dir, -1.0, 150, tint, 1.0, rng, dust, ctx, world);
            }
        }
        // `:84704-84711` — 537: ×100 dust 32.
        537 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 32, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:84776-84784` — 352: ×100 dust 67, noGravity.
        352 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                let idx = live_dust(s, 67, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
                dust.slot_mut(idx).no_gravity = true; // `:84783`
            }
        }
        // `:84795-84916` — the Pumpkin/Frost Moon boss parts: ×100,
        // alpha 0 default colour; 305-314 → 194 scale 1.1, 327 → 189
        // scale 1.1, 325/326 → 191 scale 1.1, 344 → 196 scale 1, 304 →
        // 189 scale 1.3.
        305..=314 | 327 | 325 | 344 | 326 | 304 => {
            let (t, sc) = match s.npc_type {
                305..=314 => (194, 1.1),
                327 => (189, 1.1),
                325 | 326 => (191, 1.1),
                344 => (196, 1.0),
                _ => (189, 1.3), // 304
            };
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, t, dir, -1.0, 0, Color::default(), sc, rng, dust, ctx, world);
            }
        }
        // `:85497-85505` — 122: ×50 dust 72, alpha 200, vel ×1.5.
        122 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 50.0) {
                let idx = live_dust(s, 72, 0.0, 0.0, 200, Color::default(), 1.0, rng, dust, ctx, world);
                dust.slot_mut(idx).velocity *= 1.5; // `:85504`
            }
        }
        // `:85516-85523` — the Pixie: ×50 dust 55, alpha 200, this.color.
        75 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 50.0) {
                live_dust(s, 55, 0.0, 0.0, 200, s.color, 1.0, rng, dust, ctx, world);
            }
        }
        // `:84926-84933` — 302: ×100 dust 192, alpha 100, the grey tint.
        302 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(
                    s, 192, dir, -1.0, 100, Color::new(100, 100, 100, 100), 1.0,
                    rng, dust, ctx, world,
                );
            }
        }
        // `:84944-84951` — the Golem parts: ×100 dust 148.
        245..=249 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 148, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:85010-85017` — 204: ×100 dust 4, alpha 100, the green tint.
        204 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(
                    s, 4, dir, -1.0, 100, Color::new(149, 208, 104), 1.0,
                    rng, dust, ctx, world,
                );
            }
        }
        // `:89654-89681`/`:89811-89822`/`:87238-87245` — the ×100 dust-5
        // tail rows: 480, 185, the chain's final else 156, the Brain 266.
        480 | 185 | 156 | 266 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 5, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:87259-87266` — the Creeper: ×50 dust 5.
        267 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 50.0) {
                live_dust(s, 5, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:82476-82537` — the DD2 beast Direct bursts: dust 5 at
        // (dir,-1), velocity ×2.5; 620 scale 1.7, 619 fadeIn 1.7, 618
        // scale 2 (the death half's 2.5 is g60's).
        620 | 619 | 618 => {
            let mult = if s.npc_type == 618 { 200.0 } else { 150.0 };
            for _ in 0..hit_count(s.dmg, s.life_max, mult) {
                let idx = dust.new_dust_direct(
                    rng, ctx, world, Vec2::new(s.x, s.y), s.w, s.h, 5,
                    dir, -1.0, 0, Color::default(), 1.0,
                );
                let d = dust.slot_mut(idx);
                d.velocity *= 2.5;
                match s.npc_type {
                    620 => d.scale = 1.7,   // `:82484`
                    619 => d.fade_in = 1.7, // `:82510`
                    _ => d.scale = 2.0,     // `:82536`
                }
            }
        }
        // `:82862-82869` — the Solar Crawltipedes: ×50 dust 241, no
        // live child gate (the 6-dust pairs are death-half only).
        412..=414 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 50.0) {
                live_dust(s, 241, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:83953-83961` — 478: FLAT 10 dust 238, noGravity.
        478 => {
            for _ in 0..10 {
                let idx = live_dust(s, 238, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
                dust.slot_mut(idx).no_gravity = true; // `:83960`
            }
        }
        // `:83975-84070` — the blood-critter ×100 dust-5 rows:
        // 466/468/469/463/461.
        466 | 468 | 469 | 463 | 461 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 5, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:84085-84092` — 462: ×100 dust 5.
        462 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 5, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:84281-84288` — 381: the martian divisor, dust 227.
        381 => {
            let bound = s.dmg / (s.life_max * 50) as f64;
            let mut count = 0u32;
            while (count as f64) < bound {
                live_dust(s, 227, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
                count += 1;
            }
        }
        // `:87606-87613` — the EoW segments 95/96/97: ×50 dust 5.
        95 | 96 | 97 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 50.0) {
                live_dust(s, 5, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:89167-89174` — the Skeletron hands (35/36/68): ×100 dust 26.
        35 | 36 | 68 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 26, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:88641-88650` — the Deerclops: NO live dust; the gore
        // siblings' ARGUMENT draws ride here — the clamped count
        // max(1, min(5, (int)(dmg/lifeMax·3))), 2 NextFloat per gore
        // (the position expression, `:88648`; the bodies are d55's).
        694 => {
            let raw = (s.dmg / s.life_max as f64 * 3.0) as i32;
            let count = raw.clamp(1, 5);
            for _ in 0..count {
                let _ = rng.next_float();
                let _ = rng.next_float();
            }
        }
        // `:85027-85034` — 225: ×100 dust 4, alpha 100, the blue tint.
        225 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(
                    s, 4, dir, -1.0, 100, Color::new(0, 80, 255, 100), 1.0,
                    rng, dust, ctx, world,
                );
            }
        }
        // `:85045-85052` — 243: ×150 dust 161.
        243 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 150.0) {
                live_dust(s, 161, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:85378-85385` — the Toxic Sludge: ×100 dust 4, this.alpha,
        // the (210,230,140) tint.
        141 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(
                    s, 4, dir, -1.0, s.alpha, Color::new(210, 230, 140), 1.0,
                    rng, dust, ctx, world,
                );
            }
        }
        // `:85358-85366` — the Frost Legion trio: ×100 dust 76,
        // noGravity (the death half's ×1.2 scale is g60's).
        143 | 144 | 145 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                let idx = live_dust(s, 76, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
                dust.slot_mut(idx).no_gravity = true; // `:85365`
            }
        }
        // `:87165-87183` — the Plantera parts: ×100. The 262 arm gates
        // Next(3)!=0 → 166 ONLY above half life (`this.life >
        // this.lifeMax / 2`, INTEGER half); below half — and for
        // 263/264 — the else spawns 167 with NO draw (the `:87175`
        // type-265 condition is dead inside this group).
        262 | 263 | 264 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                if s.npc_type == 262 && s.life > s.life_max / 2 && rng.next_max(3) != 0 {
                    live_dust(s, 166, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
                } else {
                    live_dust(s, 167, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
                }
            }
        }
        // `:85315-85324` — 154: ×50 dust 67 at the NPC's own velocity,
        // alpha 90 scale 1.5, noGravity + velocity ×0.2.
        154 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 50.0) {
                let idx = live_dust(s, 67, s.vx, s.vy, 90, Color::default(), 1.5, rng, dust, ctx, world);
                let d = dust.slot_mut(idx);
                d.no_gravity = true; // `:85322`
                d.velocity *= 0.2; // `:85323`
            }
        }
        // `:89742-89767` — the Man Eaters (43/56) + 175: ×100 dust 40,
        // this.alpha/color, scale 1.2.
        43 | 56 | 175 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 40, dir, -1.0, s.alpha, s.color, 1.2, rng, dust, ctx, world);
            }
        }
        // `:89776-89801`/`:87985-88031`/`:87675-87703` — the ×100 dust-5
        // group: 48, the demons 62/66, the Zoologist, the Travelling
        // Merchant, the Tax Collector, 86.
        48 | 62 | 66 | 633 | 368 | 441 | 86 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 5, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:87141-87149` — the corruption trio 83/84/179: ×50 dust 31,
        // (0,0), scale 1.5, noGravity — the LIVE arm has NO velocity ×2
        // (the `:87155` write is death-half only).
        83 | 84 | 179 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 50.0) {
                let idx = live_dust(s, 31, 0.0, 0.0, 0, Color::default(), 1.5, rng, dust, ctx, world);
                dust.slot_mut(idx).no_gravity = true; // `:87148`
            }
        }
        // `:87452-87459`/`:87537-87544`/`:87573-87580` — the corruption
        // floaters (217-219), the Clinger and the Seeker segments: ×100
        // dust 18 with the NPC's alpha/color/scale.
        217 | 218 | 219 | 101 | 98 | 99 | 100 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 18, dir, -1.0, s.alpha, s.color, s.scale, rng, dust, ctx, world);
            }
        }
        // `:87717-87743` — 155/329: ×100 dust 5.
        155 | 329 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 5, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:87753-87780` — 163/238/164/165: ×100 dust 18, PLAIN (no NPC
        // alpha/color/scale rows — unlike the :87458 group).
        163 | 238 | 164 | 165 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 18, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:89607-89614` — the Goblin Summoner: ×100 dust 5.
        471 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 5, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:89627-89639` — 472: ×100 dust 5, the per-dust Next(3)!=0 →
        // noGravity AFTER the spawn internals, then alpha 100 ALWAYS.
        472 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                let idx = live_dust(s, 5, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
                let d = dust.slot_mut(idx);
                if rng.next_max(3) != 0 {
                    d.no_gravity = true; // `:89636`
                }
                d.alpha = 100; // `:89638`
            }
        }
        // `:87793-87969`/`:88119-88160` — the ×100 dust-5 group: the
        // corruption misc (239/240, 241, 242), the bound trio + Wizard
        // (105-108), the bound Mechanic (123/124), 550, the Golfers
        // (588/589), 229, 142.
        239 | 240 | 241 | 242 | 105..=108 | 123 | 124 | 550 | 588 | 589 | 229 | 142 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 5, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:87813-87820` — 236/237: ×100 dust 18, plain.
        236 | 237 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 18, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:88176-88455`/`:86607-86632` — the ×100 dust-5 group: 178,
        // the Stylists, 37, the Clothier, 209, the pirates 212-216, the
        // Vulture, 301.
        178 | 353 | 354 | 37 | 54 | 209 | 212..=216 | 61 | 301 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 5, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:88262-88269` — 160: ×100 dust 26 (the skeleton dust, not 5).
        160 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 26, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:88503-88510` — 220/221: ×50 dust 151, scale 0.7.
        220 | 221 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 50.0) {
                live_dust(s, 151, dir, -1.0, 0, Color::default(), 0.7, rng, dust, ctx, world);
            }
        }
        // The DD2 roster's ×20 dust-5 live arms (`:82085`/`:82116`/
        // `:82149`/`:82182`/`:82225`/`:82250`/`:82275`/`:82298`/`:82320`/
        // `:82432`/`:82461`): the goblins/kobolds/drakin/wither beasts.
        // The 551 CheckProgress gate (`:82174`) draws nothing.
        552..=554 | 561..=563 | 555..=557 | 551 | 572 | 573 | 574 | 575 | 570 | 571
        | 568 | 569 | 558..=560 | 564 | 565 | 578 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 20.0) {
                live_dust(s, 5, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:82347-82354` — 566/567: ×20 dust 236.
        566 | 567 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 20.0) {
                live_dust(s, 236, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:82389-82396` — 576/577: ×20 dust 273 (the ChildSafety
        // re-type lives inside new_dust).
        576 | 577 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 20.0) {
                live_dust(s, 273, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:82562-82625` — the Ogre/Dark Mage PAIRS: dust 268 twice per
        // iteration, the XNA `new Color(r,g,b) * 0.7f` byte truncation
        // ((222,108,48,255)·0.7 → 155,75,33,178; (150,108,78,255)·0.7 →
        // 105,75,54,178), noGravity, scale 1.5, fadeIn 0.7, velocity ×3.
        // The 268 spawn is the 4-arg NewDust (speeds 0, alpha 0, scale 1).
        541 | 546 => {
            let (mult, tint) = if s.npc_type == 541 {
                (200.0, Color::new(155, 75, 33, 178))
            } else {
                (50.0, Color::new(105, 75, 54, 178))
            };
            for _ in 0..hit_count(s.dmg, s.life_max, mult) {
                for _ in 0..2 {
                    let idx = live_dust(s, 268, 0.0, 0.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
                    let d = dust.slot_mut(idx);
                    d.color = tint;
                    d.no_gravity = true;
                    d.scale = 1.5;
                    d.fade_in = 0.7;
                    d.velocity *= 3.0;
                }
            }
        }
        // `:81883-81890` — 668: ×100 dust 5, scale 1.2.
        668 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 5, dir, -1.0, 0, Color::default(), 1.2, rng, dust, ctx, world);
            }
        }
        // `:81911-81925` — the Empress: ×100, the Next(2) selector BEFORE
        // the spawn (0 → dust 67, else 69).
        636 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                let t = if rng.next_max(2) == 0 { 67 } else { 69 };
                live_dust(s, t, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:82655-82662` — 378: ×50 dust 7 at (0,0) (the 4-arg NewDust).
        378 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 50.0) {
                live_dust(s, 7, 0.0, 0.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:84107-84114` — 379/438: the martian divisor bound, dust 5.
        379 | 438 => {
            let bound = s.dmg / (s.life_max * 50) as f64;
            let mut count = 0u32;
            while (count as f64) < bound {
                live_dust(s, 5, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
                count += 1;
            }
        }
        // `:84127-84136` — the Cultist: ×100 dust 5, (0,0), scale 1.5,
        // velocity ×1.5 + noGravity.
        439 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                let idx = live_dust(s, 5, 0.0, 0.0, 0, Color::default(), 1.5, rng, dust, ctx, world);
                let d = dust.slot_mut(idx);
                d.velocity *= 1.5;
                d.no_gravity = true;
            }
        }
        // `:84180-84189` — 454-459: ×50 dust 16, the same shape.
        454..=459 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 50.0) {
                let idx = live_dust(s, 16, 0.0, 0.0, 0, Color::default(), 1.5, rng, dust, ctx, world);
                let d = dust.slot_mut(idx);
                d.velocity *= 1.5;
                d.no_gravity = true;
            }
        }
        // `:82696-82716` — the fairies: ×50 dust 278 NewDustDirect in the
        // (4+4) centre box, alpha 200, the Lerp(arg NextFloat) tint,
        // scale 0.65, noGravity. The colour pairs are the MonoGame named
        // colours (`:82698-82709`).
        583 | 584 | 585 => {
            let (value, value2) = match s.npc_type {
                584 => (Color::new(50, 205, 50, 255), Color::new(32, 178, 170, 255)),
                585 => (Color::new(65, 105, 225, 255), Color::new(173, 216, 230, 255)),
                _ => (Color::new(255, 105, 180, 255), Color::new(255, 182, 193, 255)),
            };
            for _ in 0..hit_count(s.dmg, s.life_max, 50.0) {
                // The Lerp amount draws BEFORE the spawn (argument order).
                let t = rng.next_float();
                let ch = |x: u8, y: u8| {
                    (x as f32 + (y as f32 - x as f32) * t) as u8
                };
                let tint = Color::new(
                    ch(value.r, value2.r), ch(value.g, value2.g),
                    ch(value.b, value2.b), ch(value.a, value2.a),
                );
                let idx = dust.new_dust_direct(
                    rng, ctx, world,
                    Vec2::new(s.cx - 2.0, s.cy - 2.0), 8, 8, 278,
                    0.0, 0.0, 200, tint, 0.65,
                );
                dust.slot_mut(idx).no_gravity = true; // `:82715`
            }
        }
        // `:82728-82773` — the blood-critter groups: ×100 with the NPC
        // rows (513-515/510-512 → dust 250, 621-623 → dust 5).
        513..=515 | 510..=512 | 621..=623 => {
            let t = if s.npc_type >= 621 { 5 } else { 250 };
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, t, dir, -1.0, s.alpha, s.color, s.scale, rng, dust, ctx, world);
            }
        }
        // `:85704-85849` — the ×30 dust-5 critter groups: 153, 177,
        // 494/495, 496/497, 498-506, 611/689.
        153 | 177 | 494 | 495 | 496 | 497 | 498..=506 | 611 | 689 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 30.0) {
                live_dust(s, 5, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:85860-85867` — 624: ×20 dust 5.
        624 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 20.0) {
                live_dust(s, 5, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:85879-85886` — 628: ×20 dust 3.
        628 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 20.0) {
                live_dust(s, 3, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:85898-85934` — the butterfly/moth groups: ×20, the per-type
        // dust 86+offset below the group's last id, else 138.
        646..=652 | 639..=645 => {
            let t = match s.npc_type {
                646..=651 => 86 + (s.npc_type - 646), // `:85901-85904`
                639..=644 => 86 + (s.npc_type - 639), // `:85924-85927`
                _ => 138, // 652 / 645
            };
            for _ in 0..hit_count(s.dmg, s.life_max, 20.0) {
                live_dust(s, t, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:85942-85949`/`:86092-86135`/`:86244-86251` — the ×20 dust-5
        // critter groups: the big list, the penguins/168/470, 361, the
        // ducks.
        46 | 55 | 67 | 74 | 102 | 224 | 230 | 297 | 298 | 299 | 300 | 303 | 337 | 538
        | 540 | 587 | 610 | 614 | 616 | 617 | 625 | 626 | 627 | 615 | 671..=675
        | 148 | 149 | 168 | 470 | 361 | 362..=365 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 20.0) {
                live_dust(s, 5, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:86145-86166` — the bound town slimes: 687 → dust 16, 685 →
        // dust 0 (the death PoofOfSmoke is g60's).
        687 | 685 => {
            let t = if s.npc_type == 687 { 16 } else { 0 };
            for _ in 0..hit_count(s.dmg, s.life_max, 20.0) {
                live_dust(s, t, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:86173-86197` — the fish pair 607/688: ×20 dust 45.
        607 | 688 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 20.0) {
                live_dust(s, 45, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:86217-86224` — the scorpions: ×20 dust 203.
        366 | 367 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 20.0) {
                live_dust(s, 203, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:86271-86378` — the ×20 dust-5 groups: the seagulls, grebes,
        // Nutcrackers, Krampus, Elf Archer, the corruption critters
        // (47/57/58/464/465).
        602 | 603 | 608 | 609 | 348 | 349 | 351 | 350 | 47 | 57 | 58 | 464 | 465 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 20.0) {
                live_dust(s, 5, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:86409-86416` — the spectral group: ×100 dust 5, this.alpha.
        173 | 174 | 181 | 182 | 268 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 5, dir, -1.0, s.alpha, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:86504-86511` — 157: ×100 dust 5.
        157 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 5, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:86545-86559` — the antlions: dust 250; the Larvae (582)
        // overrides the live multiplier to 30 (`:86549-86552`).
        69 | 581 | 580 | 508 | 509 | 582 => {
            let mult = if s.npc_type == 582 { 30.0 } else { 100.0 };
            for _ in 0..hit_count(s.dmg, s.life_max, mult) {
                live_dust(s, 250, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:84534-84541`/`:84592-84623` — the martian-divisor rows:
        // Scutlix 227, 482/483 dust 240, 631 dust 1.
        391 | 482 | 483 | 631 => {
            let t = match s.npc_type {
                391 => 227,
                482 | 483 => 240,
                _ => 1,
            };
            let bound = s.dmg / (s.life_max * 50) as f64;
            let mut count = 0u32;
            while (count as f64) < bound {
                live_dust(s, t, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
                count += 1;
            }
        }
        // `:84554-84563` — the saucer parts: ×20 dust 228 at (-dir,-1),
        // the Centre-Lerp 0.25 + scale 0.5 (the 384 shape).
        392..=395 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 20.0) {
                let idx = live_dust(s, 228, -dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
                let d = dust.slot_mut(idx);
                d.position.x += (s.cx - d.position.x) * 0.25;
                d.position.y += (s.cy - d.position.y) * 0.25;
                d.scale = 0.5;
            }
        }
        // `:86898-86924` — 343/691: ×100 dust 5.
        343 | 691 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 100.0) {
                live_dust(s, 5, dir, -1.0, 0, Color::default(), 1.0, rng, dust, ctx, world);
            }
        }
        // `:81813-81836` — the Rainbow Slime's live shimmer: ×100 loop,
        // the Next(5)==0 gate per iteration; the hsl colour arg (×0.5
        // byte truncation, alpha 255·0.5 → 127), dust 267; the position
        // re-anchor = Centre + NextVector2Circular(w,h) (NextFloat unit
        // angle then NextFloat radius), velocity = pos − Centre,
        // noGravity, fadeIn 0.6 + NextFloat·0.7, scale 0.35; the
        // CloneDust sibling when the slot is not the sentinel.
        661 => {
            let bound = s.dmg / s.life_max as f64 * 100.0;
            let mut i = 0u32;
            while (i as f64) < bound {
                if rng.next_max(5) == 0 {
                    // `:81822` — hslToRgb(i/100 % 1, 1, 0.5) * 0.5f.
                    let hsl = tr_core::tile_light::hsl_to_rgb(i as f32 / 100.0 % 1.0, 1.0, 0.5);
                    let tint = Color::new(
                        (hsl[0] * 255.0 * 0.5) as u8,
                        (hsl[1] * 255.0 * 0.5) as u8,
                        (hsl[2] * 255.0 * 0.5) as u8,
                        (255.0 * 0.5) as u8,
                    );
                    let idx = live_dust(s, 267, 0.0, 0.0, 0, tint, 1.0, rng, dust, ctx, world);
                    // `:81823` — NextVector2Circular: unit angle draw,
                    // then the radius draw (Utils.cs:1261-1268).
                    let angle = (std::f32::consts::TAU * rng.next_float()) as f64;
                    let unit = Vec2::new(angle.cos() as f32, angle.sin() as f32);
                    let circ = unit * Vec2::new(s.w as f32, s.h as f32) * rng.next_float();
                    let d = dust.slot_mut(idx);
                    d.position = Vec2::new(s.cx + circ.x, s.cy + circ.y);
                    d.velocity = d.position - Vec2::new(s.cx, s.cy); // `:81824`
                    d.no_gravity = true; // `:81825`
                    d.fade_in = 0.6 + rng.next_float() * 0.7; // `:81826`
                    d.scale = 0.35; // `:81827`
                    if idx != 6000 {
                        // `:81830-81833` — the CloneDust sibling.
                        let sib = dust.clone_dust(rng, ctx, world, idx);
                        let sd = dust.slot_mut(sib);
                        sd.scale /= 2.0;
                        sd.fade_in *= 0.85;
                        sd.color = Color::new(127, 127, 127, 127); // white ×0.5
                    }
                }
                i += 1;
            }
        }
        // `:81862-81871` — 521: ×200 dust-228 NewDustDirect at (6·dir,
        // vy), Transparent, scale 3, noGravity + velocity ×5.
        521 => {
            for _ in 0..hit_count(s.dmg, s.life_max, 200.0) {
                let idx = dust.new_dust_direct(
                    rng, ctx, world, Vec2::new(s.x, s.y), s.w, s.h, 228,
                    6.0 * dir, s.vy, 0, Color::TRANSPARENT, 3.0,
                );
                let d = dust.slot_mut(idx);
                d.no_gravity = true; // `:81869`
                d.velocity *= 5.0; // `:81870`
            }
        }
        // `:89526-89548`/`:89690-89700` — the three spheres are handled
        // ABOVE the life gate (the every-strike arms); this match never
        // sees 25/33/30/665.
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::SlimeVariant;

    fn air() -> World {
        World::new("t".into(), 200, 200)
    }

    /// The whole world on-screen, the full budget — the pool never
    /// culls, every spawn lands (the `update_rng` permissive ctx).
    fn ctx() -> SpawnCtx {
        SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: Vec2::ZERO,
            screen_size: Vec2::new(200.0 * 16.0, 200.0 * 16.0),
            max_dust_to_draw: 6000,
        }
    }

    fn snap(npc_type: i32, life: i32, life_max: i32, dmg: f64) -> HitDustSnapshot {
        HitDustSnapshot {
            npc_type,
            life,
            life_max,
            dmg,
            hit_dir: 1,
            x: 100.0,
            y: 100.0,
            w: 24,
            h: 18,
            cx: 112.0,
            cy: 109.0,
            vx: 0.0,
            vy: 0.0,
            alpha: 0,
            color: Color::default(),
            scale: 1.0,
            wet: false,
            ai3: 0.0,
            ai1: 0.0,
            old_pos: [Vec2::ZERO; 10],
        }
    }

    /// W4-g62 — the f64 loop bound (`(double)i < dmg / lifeMax * mult`):
    /// the division happens FIRST, so a half-HP 1-damage hit on a
    /// 100-HP NPC draws exactly 1 (0.01 × 100), and a 0.5-damage hit
    /// draws 0 (0.005 × 100 = 0.5 < 1).
    #[test]
    fn w4_g62_the_live_loop_bound_is_f64_division_first() {
        assert_eq!(hit_count(1.0, 100, 100.0), 1);
        assert_eq!(hit_count(0.5, 100, 100.0), 0);
        assert_eq!(hit_count(10.0, 100, 100.0), 10);
        // The bat ×30 and skeleton ×50 multipliers.
        assert_eq!(hit_count(1.0, 100, 30.0), 0);
        assert_eq!(hit_count(4.0, 100, 30.0), 1);
        assert_eq!(hit_count(2.0, 100, 50.0), 1);
        // An overkill on a live snapshot still counts (the C# does not
        // clamp dmg to life).
        assert_eq!(hit_count(200.0, 100, 100.0), 200);
    }

    /// W4-g62 — the death gate: a `life <= 0` snapshot emits nothing
    /// and draws NOTHING (the death halves are g59/g60's lane).
    #[test]
    fn w4_g62_a_dead_snapshot_draws_nothing() {
        let world = air();
        let ctx = ctx();
        for life in [0, -5] {
            let s = snap(1, life, 25, 10.0);
            let mut rng = UnifiedRandom::new(7);
            let twin = UnifiedRandom::new(7);
            let mut dust = DustPool::new();
            replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
            assert_eq!(rng, twin, "no draws for life {life}");
            assert!(dust.slots().iter().all(|d| !d.active));
        }
    }

    /// W4-g62 — the slime arm's draw mirror (`NPC.cs:84745-84748`):
    /// ×(dmg/lifeMax·100) dust 4 with the NPC's alpha/color, each spawn
    /// the verbatim pool internals — replay equals a hand-rolled twin
    /// pool call-for-call, stream included.
    #[test]
    fn w4_g62_the_slime_live_arm_mirrors_the_pool_draws() {
        let world = air();
        let ctx = ctx();
        let mut n = Npc::new(NpcKind::Slime, 100.0, 100.0, 1, 0.0);
        n.slime = SlimeVariant::Blue;
        n.hp = 20;
        n.max_hp = 25;
        let s = HitDustSnapshot::capture(&n, 5, 1);
        assert_eq!(s.color, Color::new(0, 80, 255, 100), "the blue row");
        let mut rng = UnifiedRandom::new(11);
        let mut dust = DustPool::new();
        replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
        // The twin: 5/25 × 100 = 20 spawns, dust 4, (1, -1), alpha 0,
        // the blue tint, scale 1.
        let mut twin_rng = UnifiedRandom::new(11);
        let mut twin = DustPool::new();
        for _ in 0..20 {
            twin.new_dust(
                &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 4,
                1.0, -1.0, 0, Color::new(0, 80, 255, 100), 1.0,
            );
        }
        assert_eq!(rng, twin_rng, "the stream matches call-for-call");
        assert_eq!(dust.slots(), twin.slots());
    }

    /// W4-g62 — the zombie group's tail draws (`NPC.cs:86945-86966`):
    /// the 186 gore gate consumes exactly ONE Next(5) after the dust
    /// loop (the body is d55's); a plain zombie draws none.
    #[test]
    fn w4_g62_the_pincushion_gore_gate_draws_once() {
        let world = air();
        let ctx = ctx();
        let s = snap(186, 40, 40, 4.0); // ×100 → 10 dusts
        let mut rng = UnifiedRandom::new(3);
        let mut dust = DustPool::new();
        replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
        let mut twin_rng = UnifiedRandom::new(3);
        let mut twin = DustPool::new();
        for _ in 0..10 {
            twin.new_dust(
                &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 5,
                1.0, -1.0, 0, Color::default(), 1.0,
            );
        }
        let _ = twin_rng.next_max(5); // `:86945`
        assert_eq!(rng, twin_rng);
        assert_eq!(dust.slots(), twin.slots());
        // The plain zombie (3) has no gate draw.
        let p = snap(3, 40, 40, 4.0);
        let mut rng2 = UnifiedRandom::new(3);
        let mut dust2 = DustPool::new();
        replay_hit_dust(&p, &mut rng2, &mut dust2, &ctx, &world);
        let mut plain_rng = UnifiedRandom::new(3);
        let mut plain = DustPool::new();
        for _ in 0..10 {
            plain.new_dust(
                &mut plain_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 5,
                1.0, -1.0, 0, Color::default(), 1.0,
            );
        }
        assert_eq!(rng2, plain_rng);
        assert_eq!(dust2.slots(), plain.slots());
    }

    /// W4-g62 — the SlimedZombie's extra loop (`NPC.cs:86953-86959`):
    /// ×200 dust 4 at height 24 (NOT the NPC's 18), alpha 125, the blue
    /// tint — after the ×100 dust-5 main loop.
    #[test]
    fn w4_g62_the_slimed_zombie_extra_loop_rides_height_24() {
        let world = air();
        let ctx = ctx();
        let s = snap(187, 50, 50, 1.0); // main ×2, extra ×4
        let mut rng = UnifiedRandom::new(5);
        let mut dust = DustPool::new();
        replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
        let mut twin_rng = UnifiedRandom::new(5);
        let mut twin = DustPool::new();
        for _ in 0..2 {
            twin.new_dust(
                &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 5,
                1.0, -1.0, 0, Color::default(), 1.0,
            );
        }
        for _ in 0..4 {
            twin.new_dust(
                &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 24, 4,
                1.0, -1.0, 125, Color::new(0, 80, 255, 100), 1.0,
            );
        }
        assert_eq!(rng, twin_rng);
        assert_eq!(dust.slots(), twin.slots());
    }

    /// W4-g62 — the zombie-goldfish dry triple (`NPC.cs:86960-86966`):
    /// 590/591 spawn 3 dust 6 ONLY when `!base.wet`; the wet snapshot
    /// stops after the main loop.
    #[test]
    fn w4_g62_the_zombie_goldfish_triple_is_wet_gated() {
        let world = air();
        let ctx = ctx();
        let mut s = snap(590, 30, 30, 3.0); // main ×10
        let mut rng = UnifiedRandom::new(9);
        let mut dust = DustPool::new();
        replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
        let dry_count = dust.slots().iter().filter(|d| d.active).count();
        assert_eq!(dry_count, 13, "10 main + 3 dry dust 6");
        s.wet = true;
        let mut rng2 = UnifiedRandom::new(9);
        let mut dust2 = DustPool::new();
        replay_hit_dust(&s, &mut rng2, &mut dust2, &ctx, &world);
        assert_eq!(dust2.slots().iter().filter(|d| d.active).count(), 10);
        assert_ne!(rng, rng2, "the dry triple's internals drew");
    }

    /// W4-g62 — the EoS/hornet arms pass the NPC's alpha/color/scale
    /// through (`NPC.cs:87508`/`:89708`): dust 18, scale = this.scale.
    #[test]
    fn w4_g62_the_eos_arm_passes_the_npc_rows() {
        let world = air();
        let ctx = ctx();
        let mut s = snap(6, 40, 40, 2.0); // ×5
        s.alpha = 55;
        s.color = Color::new(1, 2, 3, 4);
        s.scale = 1.1;
        let mut rng = UnifiedRandom::new(13);
        let mut dust = DustPool::new();
        replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
        let mut twin_rng = UnifiedRandom::new(13);
        let mut twin = DustPool::new();
        for _ in 0..5 {
            twin.new_dust(
                &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 18,
                1.0, -1.0, 55, Color::new(1, 2, 3, 4), 1.1,
            );
        }
        assert_eq!(rng, twin_rng);
        assert_eq!(dust.slots(), twin.slots());
        // The hornet group shares the shape (dust 18, ×100).
        let h = snap(42, 40, 40, 2.0);
        let mut rng3 = UnifiedRandom::new(13);
        let mut dust3 = DustPool::new();
        replay_hit_dust(&h, &mut rng3, &mut dust3, &ctx, &world);
        let live: Vec<_> = dust3.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 5);
        assert!(live.iter().all(|d| d.dust_type == 18));
    }

    /// W4-g62 — the skeleton arm's dust 26 and ×50 bound
    /// (`NPC.cs:88531-88534`), the bat arm's ×30 (`:85801`).
    #[test]
    fn w4_g62_the_skeleton_and_bat_arms_match_their_rows() {
        let world = air();
        let ctx = ctx();
        let sk = snap(21, 40, 40, 4.0); // ×50 → 5
        let mut rng = UnifiedRandom::new(17);
        let mut dust = DustPool::new();
        replay_hit_dust(&sk, &mut rng, &mut dust, &ctx, &world);
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 5);
        assert!(live.iter().all(|d| d.dust_type == 26));
        let bat = snap(49, 45, 45, 3.0); // ×30 → 2
        let mut rng2 = UnifiedRandom::new(17);
        let mut dust2 = DustPool::new();
        replay_hit_dust(&bat, &mut rng2, &mut dust2, &ctx, &world);
        assert_eq!(dust2.slots().iter().filter(|d| d.active).count(), 2);
    }

    /// W4-g62 — the capture reads the POST-strike state: the snapshot's
    /// life is the hp the strike left, the velocity the knockback wrote
    /// (the FireImp/sphere arms read it), the box the C# int truncation.
    #[test]
    fn w4_g62_the_capture_reads_the_post_strike_state() {
        let mut n = Npc::new(NpcKind::MotherSlime, 100.0, 100.0, 1, 0.0);
        n.hp = 70;
        n.max_hp = 90;
        n.vx = 2.5;
        n.vy = -3.0;
        let s = HitDustSnapshot::capture(&n, 20, -1);
        assert_eq!(s.npc_type, 16);
        assert_eq!(s.life, 70);
        assert_eq!(s.dmg, 20.0);
        assert_eq!(s.hit_dir, -1);
        assert_eq!((s.vx, s.vy), (2.5, -3.0));
        assert_eq!(s.color, Color::new(0, 0, 0, 50), "the :8975 row");
        assert_eq!((s.w, s.h), (36, 24), "the MotherSlime box");
    }

    /// W4-g62 batch 2 — the jellyfish trio's per-type tint
    /// (`NPC.cs:85536-85544`): ×50 dust 4, the blue/pink/green rows.
    #[test]
    fn w4_g62_the_jellyfish_tints_match_the_rows() {
        let world = air();
        let ctx = ctx();
        for (t, tint) in [
            (63, Color::new(50, 120, 255, 100)),
            (64, Color::new(225, 70, 140, 100)),
            (103, Color::new(70, 225, 140, 100)),
        ] {
            let s = snap(t, 60, 60, 3.0); // ×50 → 2 (3/60·50 = 2.5 → 3? no: 2.5 → 3)
            let mut rng = UnifiedRandom::new(21);
            let mut dust = DustPool::new();
            replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
            let mut twin_rng = UnifiedRandom::new(21);
            let mut twin = DustPool::new();
            for _ in 0..hit_count(3.0, 60, 50.0) {
                twin.new_dust(
                    &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 4,
                    1.0, -1.0, 0, tint, 1.0,
                );
            }
            assert_eq!(rng, twin_rng, "type {t}");
            assert_eq!(dust.slots(), twin.slots(), "type {t}");
        }
    }

    /// W4-g62 batch 2 — the lava arm's per-dust Next(8) (`NPC.cs:85568-
    /// 85572`): the roll rides AFTER the spawn internals, != 0 sets
    /// noGravity; the whole ×80 sequence mirrors call-for-call.
    #[test]
    fn w4_g62_the_lava_arm_rolls_next8_after_each_spawn() {
        let world = air();
        let ctx = ctx();
        let mut s = snap(59, 50, 50, 1.0); // ×80 → 1
        s.alpha = 50;
        let mut rng = UnifiedRandom::new(23);
        let mut dust = DustPool::new();
        replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
        let mut twin_rng = UnifiedRandom::new(23);
        let mut twin = DustPool::new();
        for _ in 0..1 {
            let idx = twin.new_dust(
                &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 6,
                2.0, -1.0, 50, Color::default(), 1.5,
            );
            if twin_rng.next_max(8) != 0 {
                twin.slot_mut(idx).no_gravity = true;
            }
        }
        assert_eq!(rng, twin_rng);
        assert_eq!(dust.slots(), twin.slots());
    }

    /// W4-g62 batch 2 — the IceSlime pair (`NPC.cs:85343-85345`): each
    /// iteration spawns dust 76 (noGravity) THEN dust 4 (frost tint,
    /// alpha 100) — the pair order is the stream order.
    #[test]
    fn w4_g62_the_ice_slime_pair_order_is_76_then_4() {
        let world = air();
        let ctx = ctx();
        let s = snap(147, 30, 30, 1.0); // ×50 → 1 pair
        let mut rng = UnifiedRandom::new(29);
        let mut dust = DustPool::new();
        replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 2);
        assert_eq!(live[0].dust_type, 76);
        assert!(live[0].no_gravity);
        assert_eq!(live[1].dust_type, 4);
        assert_eq!(live[1].alpha, 100);
        assert_eq!(live[1].color, Color::new(97, 200, 255, 100));
    }

    /// W4-g62 batch 2 — the Burning Sphere's every-strike burst
    /// (`NPC.cs:89529-89536`): 40 dusts even on the KILLING blow (no
    /// life gate), the reflected-velocity pair shape, the stream
    /// mirrors the 20 pairs exactly.
    #[test]
    fn w4_g62_the_burning_sphere_bursts_even_on_death() {
        let world = air();
        let ctx = ctx();
        let mut s = snap(25, -3, 30, 33.0);
        s.vx = 4.0;
        s.vy = -2.0;
        let mut rng = UnifiedRandom::new(31);
        let mut dust = DustPool::new();
        replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
        let mut twin_rng = UnifiedRandom::new(31);
        let mut twin = DustPool::new();
        for _ in 0..20 {
            let idx = twin.new_dust(
                &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 6,
                -0.8, 0.4, 100, Color::default(), 2.0,
            );
            let d = twin.slot_mut(idx);
            d.no_gravity = true;
            d.velocity *= 2.0;
            let idx = twin.new_dust(
                &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 6,
                -0.8, 0.4, 100, Color::default(), 1.0,
            );
            twin.slot_mut(idx).velocity *= 2.0;
        }
        assert_eq!(rng, twin_rng);
        assert_eq!(dust.slots(), twin.slots());
        assert_eq!(dust.slots().iter().filter(|d| d.active).count(), 40);
    }

    /// W4-g62 batch 2 — King Slime's ×300 bound (`NPC.cs:85674-85677`)
    /// and the hallow trio's velocity ×1.5 post-write (`:85484-85485`).
    #[test]
    fn w4_g62_king_slime_and_hallow_rows() {
        let world = air();
        let ctx = ctx();
        let ks = snap(50, 2000, 2000, 20.0); // ×300 → 3
        let mut rng = UnifiedRandom::new(37);
        let mut dust = DustPool::new();
        replay_hit_dust(&ks, &mut rng, &mut dust, &ctx, &world);
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 3);
        assert!(live.iter().all(|d| d.dust_type == 4 && d.alpha == 175));
        let il = snap(138, 180, 180, 36.0); // ×50 → 10
        let mut rng2 = UnifiedRandom::new(37);
        let mut dust2 = DustPool::new();
        replay_hit_dust(&il, &mut rng2, &mut dust2, &ctx, &world);
        let mut twin_rng = UnifiedRandom::new(37);
        let mut twin = DustPool::new();
        for _ in 0..10 {
            let idx = twin.new_dust(
                &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 71,
                0.0, 0.0, 200, Color::default(), 1.0,
            );
            twin.slot_mut(idx).velocity *= 1.5;
        }
        assert_eq!(rng2, twin_rng);
        assert_eq!(dust2.slots(), twin.slots());
    }

    /// W4-g62 batch 3 — the Water Sphere/Chaos Ball pair shapes
    /// (`NPC.cs:89543-89547`/`:89695-89699`): dust 172 at 1.2/0.6 and
    /// dust 27 at 2/1, every strike, the stream mirrors all 20 pairs.
    #[test]
    fn w4_g62_the_water_and_chaos_sphere_pairs() {
        let world = air();
        let ctx = ctx();
        for (t, dt, sa, sb) in [(33, 172, 1.2f32, 0.6f32), (30, 27, 2.0, 1.0), (665, 27, 2.0, 1.0)] {
            let mut s = snap(t, 1, 30, 30.0);
            s.vx = -3.0;
            s.vy = 1.5;
            let mut rng = UnifiedRandom::new(41);
            let mut dust = DustPool::new();
            replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
            let mut twin_rng = UnifiedRandom::new(41);
            let mut twin = DustPool::new();
            for _ in 0..20 {
                let idx = twin.new_dust(
                    &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, dt,
                    0.6, -0.3, 100, Color::default(), sa,
                );
                let d = twin.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 2.0;
                let idx = twin.new_dust(
                    &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, dt,
                    0.6, -0.3, 100, Color::default(), sb,
                );
                twin.slot_mut(idx).velocity *= 2.0;
            }
            assert_eq!(rng, twin_rng, "type {t}");
            assert_eq!(dust.slots(), twin.slots(), "type {t}");
        }
    }

    /// W4-g62 batch 3 — the Meteor Head's selector-then-pair order
    /// (`NPC.cs:89476-89483`): Next(2) BEFORE the first spawn, the
    /// dust-6 pair after; one iteration mirrors exactly.
    #[test]
    fn w4_g62_the_meteor_head_selector_rides_before_the_spawn() {
        let world = air();
        let ctx = ctx();
        let mut s = snap(23, 50, 50, 1.0); // ×100 → 2
        s.vx = 5.0;
        s.vy = 2.0;
        let mut rng = UnifiedRandom::new(43);
        let mut dust = DustPool::new();
        replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
        let mut twin_rng = UnifiedRandom::new(43);
        let mut twin = DustPool::new();
        for _ in 0..2 {
            let mut t = 25;
            if twin_rng.next_max(2) == 0 {
                t = 6;
            }
            twin.new_dust(
                &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, t,
                1.0, -1.0, 0, Color::default(), 1.0,
            );
            let idx = twin.new_dust(
                &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 6,
                1.0, 0.4, 100, Color::default(), 2.0,
            );
            twin.slot_mut(idx).no_gravity = true;
        }
        assert_eq!(rng, twin_rng);
        assert_eq!(dust.slots(), twin.slots());
    }

    /// W4-g62 batch 3 — the Fire Imp's own-velocity dust 6
    /// (`NPC.cs:89509-89510`): scale 2.5, alpha 100, noGravity.
    #[test]
    fn w4_g62_the_fire_imp_rides_its_own_velocity() {
        let world = air();
        let ctx = ctx();
        let mut s = snap(24, 60, 60, 3.0); // ×100 → 5
        s.vx = -2.0;
        s.vy = 0.5;
        let mut rng = UnifiedRandom::new(47);
        let mut dust = DustPool::new();
        replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
        let mut twin_rng = UnifiedRandom::new(47);
        let mut twin = DustPool::new();
        for _ in 0..5 {
            let idx = twin.new_dust(
                &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 6,
                -2.0, 0.5, 100, Color::default(), 2.5,
            );
            twin.slot_mut(idx).no_gravity = true;
        }
        assert_eq!(rng, twin_rng);
        assert_eq!(dust.slots(), twin.slots());
    }

    /// W4-g62 batch 3 — the Fishron/Sharkron/WanderingEye live arms are
    /// the plain ×100 dust-5 shape (`NPC.cs:84264`/`:84213`/`:86527`),
    /// and the Wandering Eye's half-life gore latch draws NOTHING.
    #[test]
    fn w4_g62_the_blood_arms_share_the_dust5_shape() {
        let world = air();
        let ctx = ctx();
        for t in [370, 372, 373, 133] {
            let mut s = snap(t, 20, 100, 10.0); // ×100 → 10
            if t == 133 {
                s.life = 20; // below half — the gore latch arm, no draws
            }
            let mut rng = UnifiedRandom::new(53);
            let mut dust = DustPool::new();
            replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
            let mut twin_rng = UnifiedRandom::new(53);
            let mut twin = DustPool::new();
            for _ in 0..10 {
                twin.new_dust(
                    &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 5,
                    1.0, -1.0, 0, Color::default(), 1.0,
                );
            }
            assert_eq!(rng, twin_rng, "type {t}");
            assert_eq!(dust.slots(), twin.slots(), "type {t}");
        }
    }

    /// W4-g62 batch 4 — the WoF/Hungry/Leech FLAT counts (`NPC.cs:87362`
    /// 20, `:87414`/`:87440` 5): no dmg bound — a 1-damage hit and an
    /// overkill hit draw the same count.
    #[test]
    fn w4_g62_the_wof_family_counts_are_flat() {
        let world = air();
        let ctx = ctx();
        for (t, want) in [(113, 20), (114, 20), (115, 5), (116, 5), (117, 5), (118, 5), (119, 5)] {
            for dmg in [1.0, 9999.0] {
                let s = snap(t, 100, 8000, dmg);
                let mut rng = UnifiedRandom::new(59);
                let mut dust = DustPool::new();
                replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
                assert_eq!(
                    dust.slots().iter().filter(|d| d.active).count(),
                    want,
                    "type {t} dmg {dmg}"
                );
            }
        }
    }

    /// W4-g62 batch 4 — the Wyvern's dust-16 post-writes (`NPC.cs:88915-
    /// 88917`): scale 1.5 at spawn, velocity ×1.5, noGravity — the LIVE
    /// multiplier is 1.5, not the death half's 2.
    #[test]
    fn w4_g62_the_wyvern_live_multiplier_is_1_5() {
        let world = air();
        let ctx = ctx();
        let s = snap(87, 4000, 4000, 80.0); // ×50 → 1
        let mut rng = UnifiedRandom::new(61);
        let mut dust = DustPool::new();
        replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
        let mut twin_rng = UnifiedRandom::new(61);
        let mut twin = DustPool::new();
        let idx = twin.new_dust(
            &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 16,
            0.0, 0.0, 0, Color::default(), 1.5,
        );
        let d = twin.slot_mut(idx);
        d.velocity *= 1.5;
        d.no_gravity = true;
        assert_eq!(rng, twin_rng);
        assert_eq!(dust.slots(), twin.slots());
    }

    /// W4-g62 batch 4 — the worm/boss dust-18 arms pass the NPC rows and
    /// the ×100/×50 bounds (`NPC.cs:87560`/`:87626`/`:87485`/`:87594`/
    /// `:87345`/`:87279`).
    #[test]
    fn w4_g62_the_worm_and_boss_bounds() {
        let world = air();
        let ctx = ctx();
        // (type, mult, dust type)
        for (t, mult, dt) in [
            (4, 100.0, 5), (5, 50.0, 5), (10, 50.0, 5),
            (7, 100.0, 18), (13, 100.0, 18), (222, 100.0, 18),
        ] {
            let s = snap(t, 100, 100, 4.0);
            let want = hit_count(4.0, 100, mult);
            let mut rng = UnifiedRandom::new(67);
            let mut dust = DustPool::new();
            replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len() as u32, want, "type {t}");
            assert!(live.iter().all(|d| d.dust_type == dt), "type {t}");
        }
    }

    /// W4-g62 batch 5 — the martian divisor (`NPC.cs:84306`): the ×50
    /// sits INSIDE the divisor — a full-lifeMax hit draws 0, a 50×
    /// lifeMax hit draws exactly 1 (NOT the ×mult arms' shape).
    #[test]
    fn w4_g62_the_martian_bound_divides_by_lifemax_times_50() {
        let world = air();
        let ctx = ctx();
        for t in [382, 390, 383, 385, 386, 389, 388, 399] {
            let s = snap(t, 300, 300, 300.0); // 300/(300·50) = 0.02 → 0
            let mut rng = UnifiedRandom::new(71);
            let twin = UnifiedRandom::new(71);
            let mut dust = DustPool::new();
            replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
            assert_eq!(rng, twin, "type {t}: a full-life hit draws nothing");
            let big = snap(t, 300, 300, 15000.0); // 15000/15000 = 1 → 1
            let mut rng2 = UnifiedRandom::new(71);
            let mut dust2 = DustPool::new();
            replay_hit_dust(&big, &mut rng2, &mut dust2, &ctx, &world);
            assert_eq!(dust2.slots().iter().filter(|d| d.active).count(), 1, "type {t}");
            assert!(dust2.slots().iter().filter(|d| d.active).all(|d| d.dust_type == 227));
        }
    }

    /// W4-g62 batch 5 — the ForceBubble/Turret/Walker Lerp arm
    /// (`NPC.cs:84348-84350`): dust 226 at (-dir,-1), the position lerps
    /// 0.25 toward Centre, scale pinned 0.5 — the mirror replays the
    /// exact post-writes on the twin pool.
    #[test]
    fn w4_g62_the_martian_lerp_arm_mirrors_the_post_writes() {
        let world = air();
        let ctx = ctx();
        for (t, mult) in [(384, 50.0), (387, 20.0), (520, 20.0)] {
            let s = snap(t, 1000, 1000, 40.0 * mult); // → 40 dusts
            let mut rng = UnifiedRandom::new(73);
            let mut dust = DustPool::new();
            replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
            let mut twin_rng = UnifiedRandom::new(73);
            let mut twin = DustPool::new();
            for _ in 0..40 {
                let idx = twin.new_dust(
                    &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 226,
                    -1.0, -1.0, 0, Color::default(), 1.0,
                );
                let d = twin.slot_mut(idx);
                d.position.x += (112.0 - d.position.x) * 0.25;
                d.position.y += (109.0 - d.position.y) * 0.25;
                d.scale = 0.5;
            }
            assert_eq!(rng, twin_rng, "type {t}");
            assert_eq!(dust.slots(), twin.slots(), "type {t}");
        }
    }

    /// W4-g62 batch 6 — the solar gated-child order (`NPC.cs:82791-
    /// 82799`): the parent 241's internals draw FIRST, then the Next(4)
    /// gate; on a hit the child 6 spawns with the four post-writes. The
    /// 418 arm's gate is Next(2) (`:82966`) — twice the child rate.
    #[test]
    fn w4_g62_the_solar_child_gate_rides_after_the_parent() {
        let world = air();
        let ctx = ctx();
        for (t, gate) in [(518, 4), (416, 4), (419, 4), (415, 4), (417, 4), (418, 2)] {
            let s = snap(t, 100, 100, 50.0); // ×20 → 10 iterations
            let mut rng = UnifiedRandom::new(79);
            let mut dust = DustPool::new();
            replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
            let mut twin_rng = UnifiedRandom::new(79);
            let mut twin = DustPool::new();
            let mut children = 0;
            for _ in 0..10 {
                twin.new_dust(
                    &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 241,
                    1.0, -1.0, 0, Color::default(), 1.0,
                );
                if twin_rng.next_max(gate) == 0 {
                    children += 1;
                    let idx = twin.new_dust(
                        &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 6,
                        0.0, 0.0, 0, Color::default(), 1.0,
                    );
                    let d = twin.slot_mut(idx);
                    d.no_gravity = true;
                    d.scale = 1.5;
                    d.fade_in = 1.0;
                    d.velocity *= 3.0;
                }
            }
            assert_eq!(rng, twin_rng, "type {t}");
            assert_eq!(dust.slots(), twin.slots(), "type {t}");
            assert_eq!(
                dust.slots().iter().filter(|d| d.active && d.dust_type == 6).count(),
                children,
                "type {t}"
            );
        }
    }

    /// W4-g62 batch 7 — the Vortex rows (`NPC.cs:83043-83053`/`:83090-
    /// 83100`/`:83129-83139`/`:83211-83221`/`:83251-83254`): the 229
    /// child, 426's ×50, 427's Next(2), and the Larva's gate-free live
    /// loop (a full-life hit draws exactly the parent count).
    #[test]
    fn w4_g62_the_vortex_rows_and_the_gate_free_larva() {
        let world = air();
        let ctx = ctx();
        for (t, gate, mult) in [(426, 4, 50.0), (427, 2, 20.0), (429, 4, 20.0), (425, 4, 20.0)] {
            let s = snap(t, 100, 100, 50.0); // bound = 50/100·mult = mult/2 iterations
            let iters = hit_count(s.dmg, s.life_max, mult);
            let mut rng = UnifiedRandom::new(83);
            let mut dust = DustPool::new();
            replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
            let mut twin_rng = UnifiedRandom::new(83);
            let mut twin = DustPool::new();
            for _ in 0..iters {
                twin.new_dust(
                    &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 241,
                    1.0, -1.0, 0, Color::default(), 1.0,
                );
                if twin_rng.next_max(gate) == 0 {
                    let idx = twin.new_dust(
                        &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 229,
                        0.0, 0.0, 0, Color::default(), 1.0,
                    );
                    let d = twin.slot_mut(idx);
                    d.no_gravity = true;
                    d.scale = 1.5;
                    d.fade_in = 1.0;
                    d.velocity *= 3.0;
                }
            }
            assert_eq!(rng, twin_rng, "type {t}");
            assert_eq!(dust.slots(), twin.slots(), "type {t}");
        }
        // The Larva: NO gate draw — the stream is exactly 10 parent spawns.
        let larva = snap(428, 100, 100, 50.0); // ×20 → 10
        let mut rng = UnifiedRandom::new(83);
        let mut dust = DustPool::new();
        replay_hit_dust(&larva, &mut rng, &mut dust, &ctx, &world);
        let mut twin_rng = UnifiedRandom::new(83);
        let mut twin = DustPool::new();
        for _ in 0..10 {
            twin.new_dust(
                &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 241,
                1.0, -1.0, 0, Color::default(), 1.0,
            );
        }
        assert_eq!(rng, twin_rng);
        assert_eq!(dust.slots(), twin.slots());
    }

    /// W4-g62 batch 8 — the Nebula rows (`NPC.cs:83280-83288`/`:83318-
    /// 83326`/`:83351-83359`/`:83384-83392`): dust 249 parent + 242
    /// child; 423 ×50 Next(4), 421/420 ×20 Next(2).
    #[test]
    fn w4_g62_the_nebula_rows() {
        let world = air();
        let ctx = ctx();
        for (t, gate, mult) in [(424, 4, 20.0), (423, 4, 50.0), (421, 2, 20.0), (420, 2, 20.0)] {
            let s = snap(t, 100, 100, 50.0); // mult/2 iterations
            let iters = hit_count(s.dmg, s.life_max, mult);
            let mut rng = UnifiedRandom::new(89);
            let mut dust = DustPool::new();
            replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
            let mut twin_rng = UnifiedRandom::new(89);
            let mut twin = DustPool::new();
            for _ in 0..iters {
                twin.new_dust(
                    &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 249,
                    1.0, -1.0, 0, Color::default(), 1.0,
                );
                if twin_rng.next_max(gate) == 0 {
                    let idx = twin.new_dust(
                        &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 242,
                        0.0, 0.0, 0, Color::default(), 1.0,
                    );
                    let d = twin.slot_mut(idx);
                    d.no_gravity = true;
                    d.scale = 1.5;
                    d.fade_in = 1.0;
                    d.velocity *= 3.0;
                }
            }
            assert_eq!(rng, twin_rng, "type {t}");
            assert_eq!(dust.slots(), twin.slots(), "type {t}");
        }
    }

    /// W4-g62 batch 9 — the Stardust Cell ring (`NPC.cs:83415-83432`):
    /// bound 10 + dmg/10 (no lifeMax), the DEAD vector draws consumed
    /// per iteration, the re-anchor + velocity + scale draws mirror.
    #[test]
    fn w4_g62_the_stardust_cell_ring_mirrors_the_dead_draws() {
        let world = air();
        let ctx = ctx();
        let mut s = snap(405, 100, 100, 25.0); // bound 12.5 → 13 iterations
        s.w = 40;
        s.cx = 120.0;
        s.cy = 110.0;
        s.scale = 1.3;
        let mut rng = UnifiedRandom::new(97);
        let mut dust = DustPool::new();
        replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
        let mut twin_rng = UnifiedRandom::new(97);
        let mut twin = DustPool::new();
        for i in 0..13u32 {
            let num161 = 40 / 4;
            let _ = twin_rng.next_double();
            let _ = twin_rng.next_range(24, 41);
            let idx = twin.new_dust(
                &mut twin_rng, &ctx, &world, Vec2::new(120.0 - 10.0, 110.0 - 10.0),
                20, 20, 229, 0.0, 0.0, 0, Color::default(), 1.0,
            );
            let d = twin.slot_mut(idx);
            let off = Vec2::new(d.position.x - 120.0, d.position.y - 110.0);
            let len = off.length();
            let n = if len > 0.0 { off / len } else { Vec2::ZERO };
            d.position = Vec2::new(120.0 + n.x * 10.0 * 1.3 - 4.0, 110.0 + n.y * 10.0 * 1.3 - 4.0);
            if i < 30 {
                let vlen = d.velocity.length();
                d.velocity = n * vlen * 2.0;
            } else {
                d.velocity = (n * 2.0) * twin_rng.next_range(45, 91) as f32 / 10.0;
            }
            d.no_gravity = true;
            d.scale = 0.7 + twin_rng.next_float();
        }
        assert_eq!(rng, twin_rng);
        assert_eq!(dust.slots(), twin.slots());
    }

    /// W4-g62 batch 9 — the Stardust dust-17 arms (`NPC.cs:83487-83491`
    /// etc.): the child carries ONLY noGravity; 407's parent is scale
    /// 0.75 Transparent, its gate Next(2); 410's bound ×10.
    #[test]
    fn w4_g62_the_stardust_17_arms() {
        let world = air();
        let ctx = ctx();
        for (t, mult, gate, pscale) in
            [(409, 50.0, 4, 1.0f32), (410, 10.0, 4, 1.0), (407, 50.0, 2, 0.75), (411, 20.0, 4, 1.0)]
        {
            let s = snap(t, 100, 100, 50.0);
            let iters = hit_count(s.dmg, s.life_max, mult);
            let mut rng = UnifiedRandom::new(101);
            let mut dust = DustPool::new();
            replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
            let mut twin_rng = UnifiedRandom::new(101);
            let mut twin = DustPool::new();
            for _ in 0..iters {
                twin.new_dust(
                    &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 17,
                    1.0, -1.0, 0, Color::TRANSPARENT, pscale,
                );
                if twin_rng.next_max(gate) == 0 {
                    let idx = twin.new_dust(
                        &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 229,
                        0.0, 0.0, 0, Color::default(), 1.0,
                    );
                    twin.slot_mut(idx).no_gravity = true;
                }
            }
            assert_eq!(rng, twin_rng, "type {t}");
            assert_eq!(dust.slots(), twin.slots(), "type {t}");
        }
    }

    /// W4-g62 batch 10 — the sandwalker gate (`NPC.cs:83765`): the
    /// Next(4) draws for 524 TOO (the `&& child > 0` short-circuits
    /// AFTER the draw) — 524's stream carries the gate but no child.
    #[test]
    fn w4_g62_the_524_gate_draws_without_a_child() {
        let world = air();
        let ctx = ctx();
        let s = snap(524, 100, 100, 50.0); // ×20 → 10
        let mut rng = UnifiedRandom::new(103);
        let mut dust = DustPool::new();
        replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
        let mut twin_rng = UnifiedRandom::new(103);
        let mut twin = DustPool::new();
        for _ in 0..10 {
            twin.new_dust(
                &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 5,
                1.0, -1.0, 0, Color::default(), 1.0,
            );
            let _ = twin_rng.next_max(4); // drawn, never spawns for 524
        }
        assert_eq!(rng, twin_rng);
        assert_eq!(dust.slots(), twin.slots());
        assert!(dust.slots().iter().filter(|d| d.active).all(|d| d.dust_type == 5));
        // 525's child (75) rides the same gate with the four writes.
        let c = snap(525, 100, 100, 50.0);
        let mut rng2 = UnifiedRandom::new(103);
        let mut dust2 = DustPool::new();
        replay_hit_dust(&c, &mut rng2, &mut dust2, &ctx, &world);
        let mut twin_rng2 = UnifiedRandom::new(103);
        let mut twin2 = DustPool::new();
        for _ in 0..10 {
            twin2.new_dust(
                &mut twin_rng2, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 5,
                1.0, -1.0, 0, Color::default(), 1.0,
            );
            if twin_rng2.next_max(4) == 0 {
                let idx = twin2.new_dust(
                    &mut twin_rng2, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 75,
                    0.0, 0.0, 0, Color::default(), 1.0,
                );
                let d = twin2.slot_mut(idx);
                d.no_gravity = true;
                d.scale = 1.5;
                d.fade_in = 1.0;
                d.velocity *= 3.0;
            }
        }
        assert_eq!(rng2, twin_rng2);
        assert_eq!(dust2.slots(), twin2.slots());
    }

    /// W4-g62 batch 10 — the Djinn's live post-writes (`NPC.cs:83873-
    /// 83875`): ×2 NOT the death ×2+fadeIn; the sharks' alpha-100 dust
    /// 250 ×30; the tomb crawlers' dust 237 ×100.
    #[test]
    fn w4_g62_the_djinn_shark_and_crawler_rows() {
        let world = air();
        let ctx = ctx();
        let dj = snap(533, 100, 100, 4.0); // ×50 → 2
        let mut rng = UnifiedRandom::new(107);
        let mut dust = DustPool::new();
        replay_hit_dust(&dj, &mut rng, &mut dust, &ctx, &world);
        let mut twin_rng = UnifiedRandom::new(107);
        let mut twin = DustPool::new();
        for _ in 0..2 {
            let idx = twin.new_dust(
                &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 27,
                0.0, 0.0, 50, Color::default(), 1.5,
            );
            let d = twin.slot_mut(idx);
            d.velocity *= 2.0;
            d.no_gravity = true;
        }
        assert_eq!(rng, twin_rng);
        assert_eq!(dust.slots(), twin.slots());
        for (t, dt, mult, alpha) in [(530, 250, 30.0, 100), (532, 5, 50.0, 100), (477, 237, 100.0, 0), (460, 5, 100.0, 0)] {
            let s = snap(t, 100, 100, 10.0);
            let want = hit_count(10.0, 100, mult);
            let mut r = UnifiedRandom::new(109);
            let mut p = DustPool::new();
            replay_hit_dust(&s, &mut r, &mut p, &ctx, &world);
            let live: Vec<_> = p.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len() as u32, want, "type {t}");
            assert!(live.iter().all(|d| d.dust_type == dt && d.alpha == alpha), "type {t}");
        }
    }

    /// W4-g62 batch 11 — the ×150 groups and the dust-137/0 rows
    /// (`NPC.cs:86734`/`:86791`/`:86811`/`:86832`/`:86852`/`:86872`).
    #[test]
    fn w4_g62_the_batch11_rows() {
        let world = air();
        let ctx = ctx();
        for (t, dt, mult) in [
            (252, 5, 100.0), (489, 5, 100.0), (534, 5, 100.0), (490, 5, 100.0),
            (65, 5, 150.0), (542, 5, 150.0), (692, 5, 150.0),
            (195, 5, 150.0), (198, 5, 150.0),
            (206, 137, 150.0), (342, 0, 100.0), (338, 5, 100.0), (340, 5, 100.0),
        ] {
            let s = snap(t, 100, 100, 10.0);
            let want = hit_count(10.0, 100, mult);
            let mut r = UnifiedRandom::new(113);
            let mut p = DustPool::new();
            replay_hit_dust(&s, &mut r, &mut p, &ctx, &world);
            let live: Vec<_> = p.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len() as u32, want, "type {t}");
            assert!(live.iter().all(|d| d.dust_type == dt), "type {t}");
        }
    }

    /// W4-g62 batch 12 — the town-NPC arms are the ×100 dust-5 shape
    /// (`NPC.cs:87658` etc.); the Princess 663 is NOT in this group (her
    /// own `:88411` arm lands with the next batch — a live 663 snapshot
    /// currently emits nothing, the documented pending row).
    #[test]
    fn w4_g62_the_town_arms_share_the_dust5_shape() {
        let world = air();
        let ctx = ctx();
        for t in [17, 18, 19, 38, 20, 207, 208, 227, 228, 22] {
            let s = snap(t, 250, 250, 25.0); // ×100 → 10
            let mut r = UnifiedRandom::new(127);
            let mut p = DustPool::new();
            replay_hit_dust(&s, &mut r, &mut p, &ctx, &world);
            let live: Vec<_> = p.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), 10, "type {t}");
            assert!(live.iter().all(|d| d.dust_type == 5), "type {t}");
        }
    }

    /// W4-g62 batch 13 — the Mimic's ai[3] selector (`NPC.cs:88768-
    /// 88776`): 2 → dust 10, 3 → dust 37, else 7; the DD2 skeletons'
    /// scale-1.1 post-write; the golem trio's ×1.5 velocity.
    #[test]
    fn w4_g62_the_mimic_selector_and_the_batch13_rows() {
        let world = air();
        let ctx = ctx();
        for (ai3, want) in [(0.0f32, 7), (2.0, 10), (3.0, 37), (1.0, 7)] {
            let mut s = snap(85, 500, 500, 20.0); // ×50 → 2
            s.ai3 = ai3;
            let mut r = UnifiedRandom::new(131);
            let mut p = DustPool::new();
            replay_hit_dust(&s, &mut r, &mut p, &ctx, &world);
            let live: Vec<_> = p.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), 2, "ai3 {ai3}");
            assert!(live.iter().all(|d| d.dust_type == want), "ai3 {ai3}");
        }
        // The DD2 skeleton: scale pinned 1.1, noGravity.
        let sk = snap(277, 100, 100, 10.0); // ×100 → 10
        let mut r = UnifiedRandom::new(131);
        let mut p = DustPool::new();
        replay_hit_dust(&sk, &mut r, &mut p, &ctx, &world);
        let live: Vec<_> = p.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 10);
        assert!(live.iter().all(|d| d.dust_type == 174 && d.scale == 1.1 && d.no_gravity));
        // The golem trio: (type, dust) rows, ×50, scale 1.5 spawn.
        for (t, dt) in [(169, 92), (170, 54), (180, 54), (171, 16)] {
            let s = snap(t, 100, 100, 10.0); // ×50 → 5
            let mut r2 = UnifiedRandom::new(131);
            let mut p2 = DustPool::new();
            replay_hit_dust(&s, &mut r2, &mut p2, &ctx, &world);
            let live2: Vec<_> = p2.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live2.len(), 5, "type {t}");
            assert!(live2.iter().all(|d| d.dust_type == dt && d.no_gravity), "type {t}");
        }
    }

    /// W4-g62 batch 14 — the Cursed Skull pair (`NPC.cs:89121-89127`):
    /// dust 15 then dust 26 per iteration, both noLight ×1.3, the first
    /// also noGravity — the full mirror on one iteration.
    #[test]
    fn w4_g62_the_cursed_skull_pair_mirrors() {
        let world = air();
        let ctx = ctx();
        let mut s = snap(34, 100, 100, 10.0 / 3.0); // ×30 → 1
        s.vx = 3.0;
        s.vy = -1.0;
        let mut rng = UnifiedRandom::new(137);
        let mut dust = DustPool::new();
        replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
        let mut twin_rng = UnifiedRandom::new(137);
        let mut twin = DustPool::new();
        for _ in 0..1 {
            let idx = twin.new_dust(
                &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 15,
                -0.6, 0.2, 100, Color::default(), 1.8,
            );
            let d = twin.slot_mut(idx);
            d.no_light = true;
            d.no_gravity = true;
            d.velocity *= 1.3;
            let idx = twin.new_dust(
                &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 26,
                -0.6, 0.2, 0, Color::default(), 0.9,
            );
            let d = twin.slot_mut(idx);
            d.no_light = true;
            d.velocity *= 1.3;
        }
        assert_eq!(rng, twin_rng);
        assert_eq!(dust.slots(), twin.slots());
    }

    /// W4-g62 batch 14 — the mummy/wraith/spore ×2-velocity rows and the
    /// 316/330 grey noGravity, 662's own-velocity 1.4 scale.
    #[test]
    fn w4_g62_the_batch14_rows() {
        let world = air();
        let ctx = ctx();
        for (t, dt, mult, alpha) in [
            (78, 31, 50.0, 0), (82, 54, 50.0, 50), (250, 54, 50.0, 50),
            (315, 54, 50.0, 50), (254, 165, 50.0, 50), (261, 165, 50.0, 50),
            (316, 192, 100.0, 100), (39, 26, 50.0, 0),
        ] {
            let s = snap(t, 100, 100, 10.0);
            let want = hit_count(10.0, 100, mult);
            let mut r = UnifiedRandom::new(139);
            let mut p = DustPool::new();
            replay_hit_dust(&s, &mut r, &mut p, &ctx, &world);
            let live: Vec<_> = p.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len() as u32, want, "type {t}");
            assert!(live.iter().all(|d| d.dust_type == dt && d.alpha == alpha), "type {t}");
            if dt != 26 {
                assert!(live.iter().all(|d| d.no_gravity), "type {t}");
            }
        }
        // 662: own velocity, scale pinned 1.4.
        let mut s = snap(662, 100, 100, 5.0); // ×100 → 5
        s.vx = 1.5;
        s.vy = -0.5;
        let mut r = UnifiedRandom::new(139);
        let mut p = DustPool::new();
        replay_hit_dust(&s, &mut r, &mut p, &ctx, &world);
        let live: Vec<_> = p.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 5);
        assert!(live.iter().all(|d| d.dust_type == 180 && d.scale == 1.4 && d.no_gravity));
    }

    /// W4-g62 batch 15 — the confetti slime's full draw sequence
    /// (`NPC.cs:84730-84741`): 10 × (type arg + internals + 2 jitter
    /// draws) then 5 × 3 gore-sibling draws, THEN the gel loop.
    #[test]
    fn w4_g62_the_confetti_burst_precedes_the_gel() {
        let world = air();
        let ctx = ctx();
        let mut s = snap(1, 25, 25, 2.5); // gel ×100 → 10
        s.ai1 = 1345.0;
        s.color = Color::new(0, 80, 255, 100);
        let mut rng = UnifiedRandom::new(149);
        let mut dust = DustPool::new();
        replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
        let mut twin_rng = UnifiedRandom::new(149);
        let mut twin = DustPool::new();
        for _ in 0..10 {
            let t = twin_rng.next_range(139, 143);
            let idx = twin.new_dust(
                &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, t,
                0.0, -10.0, 0, Color::default(), 1.2,
            );
            let d = twin.slot_mut(idx);
            d.scale *= 1.0 + twin_rng.next_range(-30, 31) as f32 * 0.01;
            d.velocity.x += twin_rng.next_range(-50, 51) as f32 * 0.05;
        }
        for _ in 0..5 {
            let _ = twin_rng.next_range(276, 283);
            let _ = twin_rng.next_range(-20, 21);
            let _ = twin_rng.next_range(-50, 51);
        }
        for _ in 0..10 {
            twin.new_dust(
                &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 4,
                1.0, -1.0, 0, Color::new(0, 80, 255, 100), 1.0,
            );
        }
        assert_eq!(rng, twin_rng);
        assert_eq!(dust.slots(), twin.slots());
        // A plain slime (ai1 != 1345) draws NO confetti/gore draws.
        let plain = snap(1, 25, 25, 2.5);
        let mut r2 = UnifiedRandom::new(149);
        let mut p2 = DustPool::new();
        replay_hit_dust(&plain, &mut r2, &mut p2, &ctx, &world);
        let mut t2 = UnifiedRandom::new(149);
        let mut q2 = DustPool::new();
        for _ in 0..10 {
            q2.new_dust(
                &mut t2, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 4,
                1.0, -1.0, 0, Color::default(), 1.0,
            );
        }
        assert_eq!(r2, t2);
        assert_eq!(p2.slots(), q2.slots());
    }

    /// W4-g62 batch 15 — the pillar tints and the moon-boss scale rows
    /// (`NPC.cs:84693`/`:84801`/`:84823`/`:84844`/`:84873`/`:84898`/
    /// `:84915`/`:84782`).
    #[test]
    fn w4_g62_the_pillar_tints_and_moon_scales() {
        let world = air();
        let ctx = ctx();
        for (t, tint) in [
            (333, Color::new(73, 147, 221, 200)),
            (334, Color::new(24, 146, 79, 200)),
            (335, Color::new(194, 46, 50, 200)),
            (336, Color::new(228, 200, 1, 200)),
        ] {
            let s = snap(t, 100, 100, 5.0); // ×100 → 5
            let mut r = UnifiedRandom::new(151);
            let mut p = DustPool::new();
            replay_hit_dust(&s, &mut r, &mut p, &ctx, &world);
            let live: Vec<_> = p.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), 5, "type {t}");
            assert!(live.iter().all(|d| d.dust_type == 4 && d.alpha == 150 && d.color == tint), "type {t}");
        }
        // (type, dust type) — the scale rides the pool's jitter × row.
        for (t, dt) in [(305, 194), (327, 189), (325, 191), (344, 196), (326, 191), (304, 189), (352, 67), (537, 32)] {
            let s = snap(t, 100, 100, 5.0);
            let mut r = UnifiedRandom::new(151);
            let mut p = DustPool::new();
            replay_hit_dust(&s, &mut r, &mut p, &ctx, &world);
            let live: Vec<_> = p.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), 5, "type {t}");
            assert!(live.iter().all(|d| d.dust_type == dt), "type {t}");
            if t == 352 {
                assert!(live.iter().all(|d| d.no_gravity));
            }
        }
    }

    /// W4-g62 batch 16 — the Plantera-hook selector (`NPC.cs:87171-
    /// 87182`): 262 above half life draws Next(3) per iteration (!=0 →
    /// 166, else 167); at/below half NO draw, always 167; 263/264 never
    /// draw the gate. The half is INTEGER (`lifeMax / 2`).
    #[test]
    fn w4_g62_the_plantera_hook_gate_is_half_life_integer() {
        let world = air();
        let ctx = ctx();
        // Above half: the gate draws.
        let s = snap(262, 60, 100, 5.0); // ×100 → 5
        let mut rng = UnifiedRandom::new(157);
        let mut dust = DustPool::new();
        replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
        let mut twin_rng = UnifiedRandom::new(157);
        let mut twin = DustPool::new();
        for _ in 0..5 {
            let t = if twin_rng.next_max(3) != 0 { 166 } else { 167 };
            twin.new_dust(
                &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, t,
                1.0, -1.0, 0, Color::default(), 1.0,
            );
        }
        assert_eq!(rng, twin_rng);
        assert_eq!(dust.slots(), twin.slots());
        // At exactly half (50 of 100): NO gate draw, all 167.
        let half = snap(262, 50, 100, 5.0);
        let mut r2 = UnifiedRandom::new(157);
        let mut p2 = DustPool::new();
        replay_hit_dust(&half, &mut r2, &mut p2, &ctx, &world);
        let mut t2 = UnifiedRandom::new(157);
        let mut q2 = DustPool::new();
        for _ in 0..5 {
            q2.new_dust(
                &mut t2, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 167,
                1.0, -1.0, 0, Color::default(), 1.0,
            );
        }
        assert_eq!(r2, t2);
        assert_eq!(p2.slots(), q2.slots());
        // 263/264: same gate-free shape.
        for t in [263, 264] {
            let s3 = snap(t, 90, 100, 5.0);
            let mut r3 = UnifiedRandom::new(157);
            let mut p3 = DustPool::new();
            replay_hit_dust(&s3, &mut r3, &mut p3, &ctx, &world);
            assert_eq!(r3, t2, "type {t} draws no gate");
        }
    }

    /// W4-g62 batch 16 — the tinted/scaled simple rows.
    #[test]
    fn w4_g62_the_batch16_rows() {
        let world = air();
        let ctx = ctx();
        for (t, dt, mult) in [
            (122, 72, 50.0), (75, 55, 50.0), (302, 192, 100.0),
            (245, 148, 100.0), (204, 4, 100.0), (225, 4, 100.0),
            (243, 161, 150.0), (141, 4, 100.0), (143, 76, 100.0),
        ] {
            let s = snap(t, 100, 100, 10.0);
            let want = hit_count(10.0, 100, mult);
            let mut r = UnifiedRandom::new(163);
            let mut p = DustPool::new();
            replay_hit_dust(&s, &mut r, &mut p, &ctx, &world);
            let live: Vec<_> = p.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len() as u32, want, "type {t}");
            assert!(live.iter().all(|d| d.dust_type == dt), "type {t}");
        }
    }

    /// W4-g62 batch 17 — the Truffle Worm bursts on EVERY strike (no
    /// life gate, `NPC.cs:85246`), FLAT 6 regardless of dmg; the 154
    /// arm's ×0.2 velocity brake.
    #[test]
    fn w4_g62_the_truffle_worm_and_154_rows() {
        let world = air();
        let ctx = ctx();
        for life in [5, 0, -3] {
            let s = snap(374, life, 5, 1.0);
            let mut r = UnifiedRandom::new(167);
            let mut p = DustPool::new();
            replay_hit_dust(&s, &mut r, &mut p, &ctx, &world);
            let live: Vec<_> = p.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), 6, "life {life}");
            assert!(live.iter().all(|d| d.dust_type == 165 && d.no_gravity && d.alpha == 50));
        }
        let mut s = snap(154, 100, 100, 10.0); // ×50 → 5
        s.vx = 2.0;
        s.vy = -1.0;
        let mut r = UnifiedRandom::new(167);
        let mut p = DustPool::new();
        replay_hit_dust(&s, &mut r, &mut p, &ctx, &world);
        let mut tr = UnifiedRandom::new(167);
        let mut tp = DustPool::new();
        for _ in 0..5 {
            let idx = tp.new_dust(
                &mut tr, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 67,
                2.0, -1.0, 90, Color::default(), 1.5,
            );
            let d = tp.slot_mut(idx);
            d.no_gravity = true;
            d.velocity *= 0.2;
        }
        assert_eq!(r, tr);
        assert_eq!(p.slots(), tp.slots());
        // The Man Eater row: dust 40 scale 1.2 with the NPC rows.
        let me = snap(43, 100, 100, 5.0);
        let mut r2 = UnifiedRandom::new(167);
        let mut p2 = DustPool::new();
        replay_hit_dust(&me, &mut r2, &mut p2, &ctx, &world);
        let live: Vec<_> = p2.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 5);
        assert!(live.iter().all(|d| d.dust_type == 40));
    }

    /// W4-g62 batch 18 — the 472 per-dust gate (`NPC.cs:89633-89638`):
    /// Next(3) AFTER the internals, alpha 100 written unconditionally;
    /// the corruption trio's LIVE arm has no velocity ×2.
    #[test]
    fn w4_g62_the_472_gate_and_the_corruption_trio() {
        let world = air();
        let ctx = ctx();
        let s = snap(472, 100, 100, 5.0); // ×100 → 5
        let mut rng = UnifiedRandom::new(173);
        let mut dust = DustPool::new();
        replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
        let mut twin_rng = UnifiedRandom::new(173);
        let mut twin = DustPool::new();
        for _ in 0..5 {
            let idx = twin.new_dust(
                &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 5,
                1.0, -1.0, 0, Color::default(), 1.0,
            );
            let d = twin.slot_mut(idx);
            if twin_rng.next_max(3) != 0 {
                d.no_gravity = true;
            }
            d.alpha = 100;
        }
        assert_eq!(rng, twin_rng);
        assert_eq!(dust.slots(), twin.slots());
        // The corruption trio: dust 31 scale 1.5 noGravity, velocity
        // UNMULTIPLIED (the spawn jitter only).
        let c = snap(83, 100, 100, 10.0); // ×50 → 5
        let mut r2 = UnifiedRandom::new(173);
        let mut p2 = DustPool::new();
        replay_hit_dust(&c, &mut r2, &mut p2, &ctx, &world);
        let mut t2 = UnifiedRandom::new(173);
        let mut q2 = DustPool::new();
        for _ in 0..5 {
            let idx = q2.new_dust(
                &mut t2, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 31,
                0.0, 0.0, 0, Color::default(), 1.5,
            );
            q2.slot_mut(idx).no_gravity = true;
        }
        assert_eq!(r2, t2);
        assert_eq!(p2.slots(), q2.slots());
    }

    /// W4-g62 batch 19 — the bound-NPC/misc rows are the ×100 dust-5
    /// shape; 236/237 are the plain dust-18 shape.
    #[test]
    fn w4_g62_the_batch19_rows() {
        let world = air();
        let ctx = ctx();
        for t in [239, 240, 241, 242, 105, 106, 107, 108, 123, 124, 550, 588, 589, 229, 142] {
            let s = snap(t, 250, 250, 25.0);
            let mut r = UnifiedRandom::new(179);
            let mut p = DustPool::new();
            replay_hit_dust(&s, &mut r, &mut p, &ctx, &world);
            let live: Vec<_> = p.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), 10, "type {t}");
            assert!(live.iter().all(|d| d.dust_type == 5), "type {t}");
        }
        for t in [236, 237] {
            let s = snap(t, 250, 250, 25.0);
            let mut r = UnifiedRandom::new(179);
            let mut p = DustPool::new();
            replay_hit_dust(&s, &mut r, &mut p, &ctx, &world);
            let live: Vec<_> = p.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), 10, "type {t}");
            assert!(live.iter().all(|d| d.dust_type == 18 && d.alpha == 0), "type {t}");
        }
    }

    /// W4-g62 batch 20 — the last simple rows: 160 is dust 26, 220/221
    /// are ×50 dust 151 scale 0.7, the rest the ×100 dust-5 shape.
    #[test]
    fn w4_g62_the_batch20_rows() {
        let world = air();
        let ctx = ctx();
        for t in [178, 353, 354, 37, 54, 209, 212, 216, 61, 301] {
            let s = snap(t, 250, 250, 25.0);
            let mut r = UnifiedRandom::new(181);
            let mut p = DustPool::new();
            replay_hit_dust(&s, &mut r, &mut p, &ctx, &world);
            let live: Vec<_> = p.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), 10, "type {t}");
            assert!(live.iter().all(|d| d.dust_type == 5), "type {t}");
        }
        let s = snap(160, 250, 250, 25.0);
        let mut r = UnifiedRandom::new(181);
        let mut p = DustPool::new();
        replay_hit_dust(&s, &mut r, &mut p, &ctx, &world);
        assert!(p.slots().iter().filter(|d| d.active).all(|d| d.dust_type == 26));
        let s2 = snap(220, 100, 100, 10.0); // ×50 → 5
        let mut r2 = UnifiedRandom::new(181);
        let mut p2 = DustPool::new();
        replay_hit_dust(&s2, &mut r2, &mut p2, &ctx, &world);
        let live: Vec<_> = p2.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 5);
        assert!(live.iter().all(|d| d.dust_type == 151));
    }

    /// W4-g62 batch 21 — the DD2 roster's ×20 rows: dust 5 for the
    /// goblins/kobolds, 236 for 566/567, 273 for 576/577.
    #[test]
    fn w4_g62_the_dd2_roster_rows() {
        let world = air();
        let ctx = ctx();
        for t in [552, 561, 555, 551, 572, 574, 570, 568, 558, 564, 578] {
            let s = snap(t, 100, 100, 25.0); // ×20 → 5
            let mut r = UnifiedRandom::new(191);
            let mut p = DustPool::new();
            replay_hit_dust(&s, &mut r, &mut p, &ctx, &world);
            let live: Vec<_> = p.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), 5, "type {t}");
            assert!(live.iter().all(|d| d.dust_type == 5), "type {t}");
        }
        for (t, dt) in [(566, 236), (576, 273)] {
            let s = snap(t, 100, 100, 25.0);
            let mut r = UnifiedRandom::new(191);
            let mut p = DustPool::new();
            replay_hit_dust(&s, &mut r, &mut p, &ctx, &world);
            let live: Vec<_> = p.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), 5, "type {t}");
            assert!(live.iter().all(|d| d.dust_type == dt), "type {t}");
        }
    }

    /// W4-g62 batch 22 — the Ogre pair (`NPC.cs:82569-82580`): two 268
    /// spawns per iteration, the ×0.7 byte-truncated tint, the five
    /// post-writes; the Empress's Next(2) rides BEFORE the spawn; 690
    /// bursts FLAT 10 even on death.
    #[test]
    fn w4_g62_the_ogre_pairs_and_the_empress_selector() {
        let world = air();
        let ctx = ctx();
        let s = snap(541, 100, 100, 0.5); // ×200 → 1 iteration = 2 dusts
        let mut rng = UnifiedRandom::new(193);
        let mut dust = DustPool::new();
        replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
        let mut twin_rng = UnifiedRandom::new(193);
        let mut twin = DustPool::new();
        for _ in 0..2 {
            let idx = twin.new_dust(
                &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 268,
                0.0, 0.0, 0, Color::default(), 1.0,
            );
            let d = twin.slot_mut(idx);
            d.color = Color::new(155, 75, 33, 178);
            d.no_gravity = true;
            d.scale = 1.5;
            d.fade_in = 0.7;
            d.velocity *= 3.0;
        }
        assert_eq!(rng, twin_rng);
        assert_eq!(dust.slots(), twin.slots());
        // The Empress: the selector draw precedes each spawn.
        let e = snap(636, 100, 100, 5.0); // ×100 → 5
        let mut r2 = UnifiedRandom::new(193);
        let mut p2 = DustPool::new();
        replay_hit_dust(&e, &mut r2, &mut p2, &ctx, &world);
        let mut t2 = UnifiedRandom::new(193);
        let mut q2 = DustPool::new();
        for _ in 0..5 {
            let t = if t2.next_max(2) == 0 { 67 } else { 69 };
            q2.new_dust(
                &mut t2, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, t,
                1.0, -1.0, 0, Color::default(), 1.0,
            );
        }
        assert_eq!(r2, t2);
        assert_eq!(p2.slots(), q2.slots());
        // 690: FLAT 10 dust 1 scale 1.2, even on a dead snapshot.
        let d690 = snap(690, -1, 10, 1.0);
        let mut r3 = UnifiedRandom::new(193);
        let mut p3 = DustPool::new();
        replay_hit_dust(&d690, &mut r3, &mut p3, &ctx, &world);
        let live: Vec<_> = p3.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 10);
        assert!(live.iter().all(|d| d.dust_type == 1));
    }

    /// W4-g62 batch 23 — the fairy Lerp-arg draw (`NPC.cs:82715`): the
    /// NextFloat rides BEFORE the spawn internals, the tint is the
    /// per-type named-colour lerp, the box is the 8×8 centre box.
    #[test]
    fn w4_g62_the_fairy_lerp_arg_precedes_the_spawn() {
        let world = air();
        let ctx = ctx();
        let s = snap(583, 100, 100, 2.0); // ×50 → 1
        let mut rng = UnifiedRandom::new(197);
        let mut dust = DustPool::new();
        replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
        let mut twin_rng = UnifiedRandom::new(197);
        let mut twin = DustPool::new();
        let t = twin_rng.next_float();
        let ch = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t) as u8;
        let tint = Color::new(
            ch(255, 255), ch(105, 182), ch(180, 193), ch(255, 255),
        );
        let idx = twin.new_dust_direct(
            &mut twin_rng, &ctx, &world, Vec2::new(112.0 - 2.0, 109.0 - 2.0), 8, 8, 278,
            0.0, 0.0, 200, tint, 0.65,
        );
        twin.slot_mut(idx).no_gravity = true;
        assert_eq!(rng, twin_rng);
        assert_eq!(dust.slots(), twin.slots());
        // The Cultist: dust 5 scale 1.5 ×1.5 noGravity; 454-459 dust 16.
        for (t, dt, mult) in [(439, 5, 100.0), (454, 16, 50.0)] {
            let s2 = snap(t, 100, 100, 10.0);
            let want = hit_count(10.0, 100, mult);
            let mut r2 = UnifiedRandom::new(197);
            let mut p2 = DustPool::new();
            replay_hit_dust(&s2, &mut r2, &mut p2, &ctx, &world);
            let live: Vec<_> = p2.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len() as u32, want, "type {t}");
            assert!(live.iter().all(|d| d.dust_type == dt && d.no_gravity), "type {t}");
        }
    }

    /// W4-g62 batch 24 — the butterfly offset rows (`NPC.cs:85901-
    /// 85904`/`:85924-85927`): 86+(type-base) below the last id, 138 at
    /// it; the ×30/×20 bounds.
    #[test]
    fn w4_g62_the_butterfly_offsets_and_critter_bounds() {
        let world = air();
        let ctx = ctx();
        for (t, dt) in [(646, 86), (647, 87), (651, 91), (652, 138), (639, 86), (644, 91), (645, 138)] {
            let s = snap(t, 100, 100, 10.0); // ×20 → 2
            let mut r = UnifiedRandom::new(199);
            let mut p = DustPool::new();
            replay_hit_dust(&s, &mut r, &mut p, &ctx, &world);
            let live: Vec<_> = p.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), 2, "type {t}");
            assert!(live.iter().all(|d| d.dust_type == dt), "type {t}");
        }
        for (t, dt, mult) in [(153, 5, 30.0), (177, 5, 30.0), (494, 5, 30.0), (498, 5, 30.0), (611, 5, 30.0), (624, 5, 20.0), (628, 3, 20.0)] {
            let s = snap(t, 100, 100, 10.0);
            let want = hit_count(10.0, 100, mult);
            let mut r = UnifiedRandom::new(199);
            let mut p = DustPool::new();
            replay_hit_dust(&s, &mut r, &mut p, &ctx, &world);
            let live: Vec<_> = p.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len() as u32, want, "type {t}");
            assert!(live.iter().all(|d| d.dust_type == dt), "type {t}");
        }
    }

    /// W4-g62 batch 25 — the critter ×20 rows: the big group dust 5,
    /// 687/685 the 16/0 pair, 607/688 dust 45, scorpions dust 203.
    #[test]
    fn w4_g62_the_batch25_rows() {
        let world = air();
        let ctx = ctx();
        for (t, dt) in [
            (46, 5), (303, 5), (540, 5), (671, 5), (148, 5), (470, 5),
            (361, 5), (362, 5), (687, 16), (685, 0), (607, 45), (688, 45),
            (366, 203), (367, 203),
        ] {
            let s = snap(t, 100, 100, 25.0); // ×20 → 5
            let mut r = UnifiedRandom::new(211);
            let mut p = DustPool::new();
            replay_hit_dust(&s, &mut r, &mut p, &ctx, &world);
            let live: Vec<_> = p.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), 5, "type {t}");
            assert!(live.iter().all(|d| d.dust_type == dt), "type {t}");
        }
    }

    /// W4-g62 batch 26 — the antlion multiplier override (`NPC.cs:86549-
    /// 86552`): 582 rides ×30, the rest ×100; the spectral group passes
    /// this.alpha.
    #[test]
    fn w4_g62_the_antlion_override_and_spectral_alpha() {
        let world = air();
        let ctx = ctx();
        for (t, want) in [(69, 10u32), (582, 3)] {
            let s = snap(t, 100, 100, 10.0);
            let mut r = UnifiedRandom::new(223);
            let mut p = DustPool::new();
            replay_hit_dust(&s, &mut r, &mut p, &ctx, &world);
            let live: Vec<_> = p.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len() as u32, want, "type {t}");
            assert!(live.iter().all(|d| d.dust_type == 250), "type {t}");
        }
        let mut s = snap(173, 100, 100, 5.0);
        s.alpha = 77;
        let mut r = UnifiedRandom::new(223);
        let mut p = DustPool::new();
        replay_hit_dust(&s, &mut r, &mut p, &ctx, &world);
        assert!(p.slots().iter().filter(|d| d.active).all(|d| d.alpha == 77 && d.dust_type == 5));
        // The bird/holiday ×20 rows.
        for t in [602, 608, 348, 351, 350, 47, 464] {
            let s2 = snap(t, 100, 100, 25.0);
            let mut r2 = UnifiedRandom::new(223);
            let mut p2 = DustPool::new();
            replay_hit_dust(&s2, &mut r2, &mut p2, &ctx, &world);
            assert_eq!(p2.slots().iter().filter(|d| d.active).count(), 5, "type {t}");
        }
    }

    /// W4-g62 batch 27 — the Vile Spit's post-spawn Next(2) branch
    /// (`NPC.cs:85400-85406`): 0 → scale ×0.6 ONLY (continue), else
    /// velocity ×1.4 + noGravity; FLAT 20 even on death, the y+2 box.
    #[test]
    fn w4_g62_the_vile_spit_branch_mirrors() {
        let world = air();
        let ctx = ctx();
        let s = snap(112, -5, 100, 10.0);
        let mut rng = UnifiedRandom::new(227);
        let mut dust = DustPool::new();
        replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
        let mut twin_rng = UnifiedRandom::new(227);
        let mut twin = DustPool::new();
        for _ in 0..20 {
            let idx = twin.new_dust(
                &mut twin_rng, &ctx, &world, Vec2::new(100.0, 102.0), 24, 18, 18,
                0.0, 0.0, 100, Color::default(), 2.0,
            );
            let d = twin.slot_mut(idx);
            if twin_rng.next_max(2) == 0 {
                d.scale *= 0.6;
                continue;
            }
            d.velocity *= 1.4;
            d.no_gravity = true;
        }
        assert_eq!(rng, twin_rng);
        assert_eq!(dust.slots(), twin.slots());
        // The saucer Lerp arm + the martian-divisor rows.
        let sc = snap(392, 100, 100, 50.0); // ×20 → 10
        let mut r2 = UnifiedRandom::new(227);
        let mut p2 = DustPool::new();
        replay_hit_dust(&sc, &mut r2, &mut p2, &ctx, &world);
        let live: Vec<_> = p2.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 10);
        assert!(live.iter().all(|d| d.dust_type == 228 && d.scale == 0.5));
        let sx = snap(391, 300, 300, 15000.0); // divisor → 1
        let mut r3 = UnifiedRandom::new(227);
        let mut p3 = DustPool::new();
        replay_hit_dust(&sx, &mut r3, &mut p3, &ctx, &world);
        assert_eq!(p3.slots().iter().filter(|d| d.active).count(), 1);
        assert!(p3.slots().iter().filter(|d| d.active).all(|d| d.dust_type == 227));
    }

    /// W4-g62 batch 28 — the Solar Flare's every-strike sequence
    /// (`NPC.cs:82996-83005`): 20 × (spawn, ×3, NextFloat scale add,
    /// Next(2) gate, optional 259 sibling), even on death.
    #[test]
    fn w4_g62_the_solar_flare_sequence_mirrors() {
        let world = air();
        let ctx = ctx();
        let s = snap(516, -2, 100, 5.0);
        let mut rng = UnifiedRandom::new(229);
        let mut dust = DustPool::new();
        replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
        let mut twin_rng = UnifiedRandom::new(229);
        let mut twin = DustPool::new();
        for _ in 0..20 {
            let idx = twin.new_dust(
                &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 6,
                2.5, -2.5, 0, Color::default(), 1.0,
            );
            let d = twin.slot_mut(idx);
            d.velocity *= 3.0;
            d.scale += twin_rng.next_float();
            if twin_rng.next_max(2) == 0 {
                let sib = twin.new_dust(
                    &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 259,
                    2.5, -2.5, 0, Color::default(), 1.0,
                );
                twin.slot_mut(sib).no_gravity = true;
            }
        }
        assert_eq!(rng, twin_rng);
        assert_eq!(dust.slots(), twin.slots());
    }

    /// W4-g62 batch 28 — the Rainbow Slime's gated shimmer
    /// (`NPC.cs:81817-81834`): the Next(5) gate per iteration, the hsl
    /// colour, the two circular draws, the clone sibling. A seed whose
    /// first Next(5) hits pins the full sequence.
    #[test]
    fn w4_g62_the_rainbow_slime_gated_shimmer() {
        let world = air();
        let ctx = ctx();
        // One iteration: dmg/lifeMax·100 = 1.
        let s = snap(661, 100, 100, 1.0);
        // Find a seed whose first Next(5) == 0.
        let seed = (1..500).find(|&v| UnifiedRandom::new(v).next_max(5) == 0).unwrap();
        let mut rng = UnifiedRandom::new(seed);
        let mut dust = DustPool::new();
        replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
        let mut twin_rng = UnifiedRandom::new(seed);
        let mut twin = DustPool::new();
        assert_eq!(twin_rng.next_max(5), 0);
        let hsl = tr_core::tile_light::hsl_to_rgb(0.0, 1.0, 0.5);
        let tint = Color::new(
            (hsl[0] * 255.0 * 0.5) as u8, (hsl[1] * 255.0 * 0.5) as u8,
            (hsl[2] * 255.0 * 0.5) as u8, 127,
        );
        let idx = twin.new_dust(
            &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 267,
            0.0, 0.0, 0, tint, 1.0,
        );
        let angle = (std::f32::consts::TAU * twin_rng.next_float()) as f64;
        let unit = Vec2::new(angle.cos() as f32, angle.sin() as f32);
        let circ = unit * Vec2::new(24.0, 18.0) * twin_rng.next_float();
        let d = twin.slot_mut(idx);
        d.position = Vec2::new(112.0 + circ.x, 109.0 + circ.y);
        d.velocity = d.position - Vec2::new(112.0, 109.0);
        d.no_gravity = true;
        d.fade_in = 0.6 + twin_rng.next_float() * 0.7;
        d.scale = 0.35;
        let sib = twin.clone_dust(&mut twin_rng, &ctx, &world, idx);
        let sd = twin.slot_mut(sib);
        sd.scale /= 2.0;
        sd.fade_in *= 0.85;
        sd.color = Color::new(127, 127, 127, 127);
        assert_eq!(rng, twin_rng);
        assert_eq!(dust.slots(), twin.slots());
    }

    /// W4-g62 batch 28 — the Queen minion LIVE gate: a dead snapshot
    /// draws NOTHING (the 40-count death halves are g60's); 667's live
    /// gore-argument draws precede the dust loop; 676's velocity adds.
    #[test]
    fn w4_g62_the_queen_minion_live_gate_and_667_draws() {
        let world = air();
        let ctx = ctx();
        let dead = snap(658, 0, 100, 10.0);
        let mut rng = UnifiedRandom::new(233);
        let twin = UnifiedRandom::new(233);
        let mut dust = DustPool::new();
        replay_hit_dust(&dead, &mut rng, &mut dust, &ctx, &world);
        assert_eq!(rng, twin, "the death halves are g60's");
        assert!(dust.slots().iter().all(|d| !d.active));
        // 667 live: 3 × 2 gore-arg draws then 7 × (spawn + Next(3)).
        let s = snap(667, 100, 100, 10.0);
        let mut r2 = UnifiedRandom::new(233);
        let mut p2 = DustPool::new();
        replay_hit_dust(&s, &mut r2, &mut p2, &ctx, &world);
        let mut t2 = UnifiedRandom::new(233);
        let mut q2 = DustPool::new();
        for _ in 0..3 {
            let _ = t2.next_range(1, 10);
            let _ = t2.next_max(4);
        }
        for _ in 0..7 {
            let idx = q2.new_dust(
                &mut t2, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 10,
                2.0, -1.0, 80, Color::default(), 1.1,
            );
            if t2.next_max(3) != 0 {
                q2.slot_mut(idx).no_gravity = true;
            }
        }
        assert_eq!(r2, t2);
        assert_eq!(p2.slots(), q2.slots());
        // 676 live: FLAT 8 dust 310, the velocity ×1.5 + 0.1·vel add.
        let mut s3 = snap(676, 100, 100, 10.0);
        s3.vx = 2.0;
        s3.vy = -1.0;
        let mut r3 = UnifiedRandom::new(233);
        let mut p3 = DustPool::new();
        replay_hit_dust(&s3, &mut r3, &mut p3, &ctx, &world);
        let live: Vec<_> = p3.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 8);
        assert!(live.iter().all(|d| d.dust_type == 310 && d.alpha == 80));
    }

    /// W4-g62 batch 28 — 521's Direct burst (`NPC.cs:81866-81871`):
    /// ×200 dust 228 at (6·dir, vy), Transparent, scale 3, ×5 noGravity.
    #[test]
    fn w4_g62_the_521_direct_burst() {
        let world = air();
        let ctx = ctx();
        let mut s = snap(521, 100, 100, 0.5); // ×200 → 1
        s.vy = 1.5;
        let mut rng = UnifiedRandom::new(239);
        let mut dust = DustPool::new();
        replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
        let mut twin_rng = UnifiedRandom::new(239);
        let mut twin = DustPool::new();
        let idx = twin.new_dust_direct(
            &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 228,
            6.0, 1.5, 0, Color::TRANSPARENT, 3.0,
        );
        let d = twin.slot_mut(idx);
        d.no_gravity = true;
        d.velocity *= 5.0;
        assert_eq!(rng, twin_rng);
        assert_eq!(dust.slots(), twin.slots());
    }

    /// W4-g62 batch 29 — the Queen Slime's live double-Lerp colour
    /// (`NPC.cs:81978-81987`): the INNER NextFloat rides first, A=150,
    /// the ×0.3 velocity + 0.3·vel add, the Next(3)==0 gate.
    #[test]
    fn w4_g62_the_queen_slime_live_shimmer() {
        let world = air();
        let ctx = ctx();
        let mut s = snap(657, 100, 100, 10.0);
        s.vx = 1.0;
        s.vy = -2.0;
        let mut rng = UnifiedRandom::new(241);
        let mut dust = DustPool::new();
        replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
        let mut twin_rng = UnifiedRandom::new(241);
        let mut twin = DustPool::new();
        for _ in 0..12 {
            let inner_t = twin_rng.next_float();
            let outer_t = twin_rng.next_float();
            let inner = lerp_color(Color::new(200, 200, 200, 255), Color::new(255, 80, 255, 255), inner_t);
            let mut tint = lerp_color(Color::new(0, 160, 255, 255), inner, outer_t);
            tint.a = 150;
            let idx = twin.new_dust(
                &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 4,
                2.0, -1.0, 50, tint, 1.0,
            );
            let d = twin.slot_mut(idx);
            d.velocity *= 0.3;
            d.velocity += Vec2::new(1.0, -2.0) * 0.3;
            if twin_rng.next_max(3) == 0 {
                d.no_gravity = true;
                d.scale *= 1.2;
            }
        }
        assert_eq!(rng, twin_rng);
        assert_eq!(dust.slots(), twin.slots());
        // A dead Queen draws nothing (the 150-burst is g60's).
        let dead = snap(657, 0, 100, 10.0);
        let mut r2 = UnifiedRandom::new(241);
        let t2 = UnifiedRandom::new(241);
        let mut p2 = DustPool::new();
        replay_hit_dust(&dead, &mut r2, &mut p2, &ctx, &world);
        assert_eq!(r2, t2);
    }

    /// W4-g62 batch 29 — the crystal ring (`NPC.cs:83026-83036`): the
    /// two NextDouble spin draws FIRST, then 24 × (spawn, rotated
    /// velocity with one NextFloat, SkyBlue, scale add NextFloat).
    #[test]
    fn w4_g62_the_crystal_ring_mirrors() {
        let world = air();
        let ctx = ctx();
        let s = snap(522, -1, 1000, 5.0); // every strike
        let mut rng = UnifiedRandom::new(251);
        let mut dust = DustPool::new();
        replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
        let mut t2 = UnifiedRandom::new(251);
        let mut twin = DustPool::new();
        let max = std::f64::consts::PI;
        let angle = t2.next_double() * max - t2.next_double() * max;
        let spin = crate::bow_launch::rotated_by(Vec2::new(0.0, -3.0), angle);
        for i in 0..24u32 {
            let idx = twin.new_dust(
                &mut t2, &ctx, &world, Vec2::new(112.0, 109.0), 0, 0, 66,
                0.0, 0.0, 0, Color::TRANSPARENT, 1.0,
            );
            let d = twin.slot_mut(idx);
            d.position = Vec2::new(112.0, 109.0);
            let a = (std::f32::consts::PI * 2.0 * i as f32 / 24.0) as f64;
            let v = crate::bow_launch::rotated_by(spin, a);
            let f = 0.8 + t2.next_float() * 0.4;
            d.velocity = v * Vec2::new(1.05, 1.0) * f * 2.0;
            d.color = Color::new(135, 206, 235, 255);
            d.no_gravity = true;
            d.scale += 0.5 + t2.next_float();
        }
        assert_eq!(rng, t2);
        assert_eq!(dust.slots(), twin.slots());
    }

    /// W4-g62 batch 30 — the gold-critter block runs BEFORE the chain
    /// (`NPC.cs:84637`): FLAT 10 with the Next(232,234) arg per dust;
    /// 627 falls through to the `:85948` ×20 dust-5 arm AFTER it.
    #[test]
    fn w4_g62_the_gold_critter_block_and_the_627_fallthrough() {
        let world = air();
        let ctx = ctx();
        let s = snap(442, 10, 10, 1.0);
        let mut rng = UnifiedRandom::new(257);
        let mut dust = DustPool::new();
        replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
        let mut twin_rng = UnifiedRandom::new(257);
        let mut twin = DustPool::new();
        for _ in 0..10 {
            let t = twin_rng.next_range(232, 234);
            twin.new_dust(
                &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, t,
                1.0, -1.0, 0, Color::default(), 1.0,
            );
        }
        assert_eq!(rng, twin_rng);
        assert_eq!(dust.slots(), twin.slots());
        // 627: the gold block THEN the ×20 chain arm (dmg/lifeMax·20 = 2).
        let g = snap(627, 100, 100, 10.0);
        let mut r2 = UnifiedRandom::new(257);
        let mut p2 = DustPool::new();
        replay_hit_dust(&g, &mut r2, &mut p2, &ctx, &world);
        let mut t2 = UnifiedRandom::new(257);
        let mut q2 = DustPool::new();
        for _ in 0..10 {
            let t = t2.next_range(232, 234);
            q2.new_dust(
                &mut t2, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, t,
                1.0, -1.0, 0, Color::default(), 1.0,
            );
        }
        for _ in 0..2 {
            q2.new_dust(
                &mut t2, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 5,
                1.0, -1.0, 0, Color::default(), 1.0,
            );
        }
        assert_eq!(r2, t2);
        assert_eq!(p2.slots(), q2.slots());
        // The tail rows: 480/185/156/266 ×100 dust 5, 267 ×50.
        for (t, mult) in [(480, 100.0), (185, 100.0), (156, 100.0), (266, 100.0), (267, 50.0)] {
            let s3 = snap(t, 100, 100, 10.0);
            let want = hit_count(10.0, 100, mult);
            let mut r3 = UnifiedRandom::new(257);
            let mut p3 = DustPool::new();
            replay_hit_dust(&s3, &mut r3, &mut p3, &ctx, &world);
            assert_eq!(p3.slots().iter().filter(|d| d.active).count() as u32, want, "type {t}");
        }
    }

    /// W4-g62 batch 31 — the DD2 Direct bursts (`NPC.cs:82482-82484`
    /// etc.): the per-type post-write (620 scale 1.7, 619 fadeIn 1.7,
    /// 618 scale 2), velocity ×2.5; 478's FLAT 10; the crawltipede ×50.
    #[test]
    fn w4_g62_the_dd2_direct_bursts_and_the_tail() {
        let world = air();
        let ctx = ctx();
        for (t, mult) in [(620, 150.0), (619, 150.0), (618, 200.0)] {
            let s = snap(t, 100, 100, 10.0 / mult * 100.0); // → 10 iterations
            let mut rng = UnifiedRandom::new(263);
            let mut dust = DustPool::new();
            replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
            let mut twin_rng = UnifiedRandom::new(263);
            let mut twin = DustPool::new();
            for _ in 0..10 {
                let idx = twin.new_dust_direct(
                    &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 5,
                    1.0, -1.0, 0, Color::default(), 1.0,
                );
                let d = twin.slot_mut(idx);
                d.velocity *= 2.5;
                match t {
                    620 => d.scale = 1.7,
                    619 => d.fade_in = 1.7,
                    _ => d.scale = 2.0,
                }
            }
            assert_eq!(rng, twin_rng, "type {t}");
            assert_eq!(dust.slots(), twin.slots(), "type {t}");
        }
        let f = snap(478, 100, 100, 1.0);
        let mut r = UnifiedRandom::new(263);
        let mut p = DustPool::new();
        replay_hit_dust(&f, &mut r, &mut p, &ctx, &world);
        let live: Vec<_> = p.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 10);
        assert!(live.iter().all(|d| d.dust_type == 238 && d.no_gravity));
        for t in [412, 466, 468, 469, 463, 461] {
            let s2 = snap(t, 100, 100, 10.0);
            let mult = if t == 412 { 50.0 } else { 100.0 };
            let want = hit_count(10.0, 100, mult);
            let mut r2 = UnifiedRandom::new(263);
            let mut p2 = DustPool::new();
            replay_hit_dust(&s2, &mut r2, &mut p2, &ctx, &world);
            assert_eq!(p2.slots().iter().filter(|d| d.active).count() as u32, want, "type {t}");
        }
    }

    /// W4-g62 batch 32 — the Deerclops live gore-arg clamp (`NPC.cs:
    /// 88645-88648`): max(1,min(5,(int)(dmg/lifeMax·3))) × 2 NextFloat,
    /// NO dust; the Skeletron hands ×100 dust 26; 381's martian divisor.
    #[test]
    fn w4_g62_the_deerclops_clamp_and_the_final_tail() {
        let world = air();
        let ctx = ctx();
        // A tiny hit still draws ONE gore's 2 floats (the max(1,..)).
        let tiny = snap(694, 100, 100, 1.0);
        let mut rng = UnifiedRandom::new(269);
        let mut dust = DustPool::new();
        replay_hit_dust(&tiny, &mut rng, &mut dust, &ctx, &world);
        let mut twin = UnifiedRandom::new(269);
        let _ = twin.next_float();
        let _ = twin.next_float();
        assert_eq!(rng, twin);
        assert!(dust.slots().iter().all(|d| !d.active), "no live dust");
        // An overkill clamps at 5 gores = 10 floats.
        let big = snap(694, 100, 100, 9999.0);
        let mut r2 = UnifiedRandom::new(269);
        let mut p2 = DustPool::new();
        replay_hit_dust(&big, &mut r2, &mut p2, &ctx, &world);
        let mut t2 = UnifiedRandom::new(269);
        for _ in 0..10 {
            let _ = t2.next_float();
        }
        assert_eq!(r2, t2);
        // The hands: ×100 dust 26; the EoW segments ×50 dust 5.
        for (t, dt, mult) in [(35, 26, 100.0), (36, 26, 100.0), (68, 26, 100.0), (95, 5, 50.0), (462, 5, 100.0)] {
            let s = snap(t, 100, 100, 10.0);
            let want = hit_count(10.0, 100, mult);
            let mut r3 = UnifiedRandom::new(269);
            let mut p3 = DustPool::new();
            replay_hit_dust(&s, &mut r3, &mut p3, &ctx, &world);
            let live: Vec<_> = p3.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len() as u32, want, "type {t}");
            assert!(live.iter().all(|d| d.dust_type == dt), "type {t}");
        }
        let m = snap(381, 300, 300, 15000.0); // divisor → 1
        let mut r4 = UnifiedRandom::new(269);
        let mut p4 = DustPool::new();
        replay_hit_dust(&m, &mut r4, &mut p4, &ctx, &world);
        assert_eq!(p4.slots().iter().filter(|d| d.active).count(), 1);
        assert!(p4.slots().iter().filter(|d| d.active).all(|d| d.dust_type == 227));
    }

    /// W4-g62 batch 33 — the SolarGoop's trail walk (`NPC.cs:83008-
    /// 83023`): EVERY strike (even a killing blow — no life gate), slots
    /// 0/3/6/9 of the 10-ring, (int)-truncated positions, NO zero-break
    /// (an all-zero fresh ring spawns its 4×5 dusts AT (0,0), the C#'s
    /// fresh-NPC shape). Per dust: the SelectRandom Next(3) arg FIRST,
    /// then the pool internals, then alpha 200 / ×2.4 / scale +=
    /// NextFloat. Draw-for-draw twin.
    #[test]
    fn w4_g62_the_goop_trail_walks_every_third_slot_without_a_break() {
        let world = air();
        let ctx = ctx();
        // A filled ring with fractional slots: the (int) cast truncates.
        let mut s = snap(519, 1, 1, 9999.0); // the goop's lifeMax 1; dead
        s.old_pos = [
            Vec2::new(100.9, 200.9),
            Vec2::new(1.0, 1.0),
            Vec2::new(2.0, 2.0),
            Vec2::new(88.5, 190.25),
            Vec2::new(3.0, 3.0),
            Vec2::new(4.0, 4.0),
            Vec2::new(76.0, 180.0),
            Vec2::new(5.0, 5.0),
            Vec2::new(6.0, 6.0),
            Vec2::new(64.0, 170.0),
        ];
        let mut rng = UnifiedRandom::new(271);
        let mut dust = DustPool::new();
        replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
        let mut twin_rng = UnifiedRandom::new(271);
        let mut twin = DustPool::new();
        for slot in [
            Vec2::new(100.0, 200.0), // (int)100.9 / (int)200.9
            Vec2::new(88.0, 190.0),
            Vec2::new(76.0, 180.0),
            Vec2::new(64.0, 170.0),
        ] {
            for _ in 0..5 {
                let t = [6, 259, 158][twin_rng.next_max(3) as usize];
                let idx = twin.new_dust(
                    &mut twin_rng, &ctx, &world, slot, 24, 18, t,
                    2.5, -2.5, 0, Color::default(), 1.0,
                );
                let d = twin.slot_mut(idx);
                d.alpha = 200;
                d.velocity *= 2.4;
                d.scale += twin_rng.next_float();
            }
        }
        assert_eq!(rng, twin_rng, "the stream matches call-for-call");
        assert_eq!(dust.slots(), twin.slots());
        // The fresh all-zero ring still fires all four slot groups —
        // at (0,0) (NO zero-break, unlike the 402 arm).
        let z = snap(519, 1, 1, 0.0);
        let mut rz = UnifiedRandom::new(271);
        let mut pz = DustPool::new();
        replay_hit_dust(&z, &mut rz, &mut pz, &ctx, &world);
        let mut tz = UnifiedRandom::new(271);
        let mut qz = DustPool::new();
        for _ in 0..4 {
            for _ in 0..5 {
                let t = [6, 259, 158][tz.next_max(3) as usize];
                let idx = qz.new_dust(
                    &mut tz, &ctx, &world, Vec2::ZERO, 24, 18, t,
                    2.5, -2.5, 0, Color::default(), 1.0,
                );
                let d = qz.slot_mut(idx);
                d.alpha = 200;
                d.velocity *= 2.4;
                d.scale += tz.next_float();
            }
        }
        assert_eq!(rz, tz);
        assert_eq!(pz.slots(), qz.slots());
    }

    /// W4-g62 batch 34 — the Stardust Worm Head's LIVE arm (`NPC.cs:
    /// 83565-83598`): ×(dmg/lifeMax·3) iterations of (Next(3) head 17 +
    /// Next(2) head 229 + the per-slot Next(4) trail gate). The LIVE
    /// ring: slots 0-2 filled, slot 3 zero — a passing gate on slot 3
    /// BREAKS (slots 4-9 draw nothing THAT iteration; the next
    /// iteration starts its walk over). The twin mirrors every draw.
    #[test]
    fn w4_g62_the_worm_head_live_trail_gates_per_slot() {
        let world = air();
        let ctx = ctx();
        let mut s = snap(402, 700, 700, 700.0); // ×3 → 3 iterations
        s.old_pos = [
            Vec2::new(100.0, 100.0),
            Vec2::new(94.0, 101.0),
            Vec2::new(88.0, 103.0),
            Vec2::ZERO,
            Vec2::ZERO,
            Vec2::ZERO,
            Vec2::ZERO,
            Vec2::ZERO,
            Vec2::ZERO,
            Vec2::ZERO,
        ];
        let mut rng = UnifiedRandom::new(277);
        let mut dust = DustPool::new();
        replay_hit_dust(&s, &mut rng, &mut dust, &ctx, &world);
        let mut twin_rng = UnifiedRandom::new(277);
        let mut twin = DustPool::new();
        for _ in 0..3 {
            if twin_rng.next_max(3) == 0 {
                twin.new_dust(
                    &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 17,
                    1.0, -1.0, 0, Color::TRANSPARENT, 0.75,
                );
            }
            if twin_rng.next_max(2) == 0 {
                let idx = twin.new_dust(
                    &mut twin_rng, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 229,
                    0.0, 0.0, 0, Color::default(), 1.0,
                );
                twin.slot_mut(idx).no_gravity = true;
            }
            for &slot in &s.old_pos {
                if twin_rng.next_max(4) == 0 {
                    if slot == Vec2::ZERO {
                        break;
                    }
                    if twin_rng.next_max(3) == 0 {
                        twin.new_dust(
                            &mut twin_rng, &ctx, &world, slot, 24, 18, 17,
                            1.0, -1.0, 0, Color::TRANSPARENT, 0.75,
                        );
                    }
                    if twin_rng.next_max(2) == 0 {
                        let idx = twin.new_dust(
                            &mut twin_rng, &ctx, &world, slot, 24, 18, 229,
                            0.0, 0.0, 0, Color::default(), 1.0,
                        );
                        twin.slot_mut(idx).no_gravity = true;
                    }
                }
            }
        }
        assert_eq!(rng, twin_rng, "the stream matches call-for-call");
        assert_eq!(dust.slots(), twin.slots());
        // The fresh all-zero ring: each iteration's walk draws gates
        // until the FIRST passing one breaks at slot 0 (the C#'s
        // never-updated shape — g60's death witness precedent).
        let z = snap(402, 700, 700, 700.0);
        let mut rz = UnifiedRandom::new(277);
        let mut pz = DustPool::new();
        replay_hit_dust(&z, &mut rz, &mut pz, &ctx, &world);
        let mut tz = UnifiedRandom::new(277);
        let mut qz = DustPool::new();
        for _ in 0..3 {
            if tz.next_max(3) == 0 {
                qz.new_dust(
                    &mut tz, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 17,
                    1.0, -1.0, 0, Color::TRANSPARENT, 0.75,
                );
            }
            if tz.next_max(2) == 0 {
                let idx = qz.new_dust(
                    &mut tz, &ctx, &world, Vec2::new(100.0, 100.0), 24, 18, 229,
                    0.0, 0.0, 0, Color::default(), 1.0,
                );
                qz.slot_mut(idx).no_gravity = true;
            }
            for _ in 0..10 {
                if tz.next_max(4) == 0 {
                    break; // slot 0 is zero
                }
            }
        }
        assert_eq!(rz, tz);
        assert_eq!(pz.slots(), qz.slots());
        // A dead snapshot draws NOTHING (the death half :83599-83632
        // is g60's site — this lane's replay stays silent).
        let d = snap(402, 0, 700, 700.0);
        let mut rd = UnifiedRandom::new(277);
        let peek = rd.clone();
        let mut pd = DustPool::new();
        replay_hit_dust(&d, &mut rd, &mut pd, &ctx, &world);
        assert_eq!(rd, peek, "no draws on the death side");
        assert!(pd.slots().iter().all(|x| !x.active));
    }
}
