# 132 — The persist MASTER MAP: three layers, one owner per field

**Status: FINAL — THE PERSIST MASTER INDEX.** Written by W4-f44 crosspersist
(branch `task/W4-f44-save-plr-2`, base `180b7430` audit DONE + the
`d331ea7c` timestamp residual fix), closed by the FINAL phase (`63204f85`
slot-11 property fix + this update). This doc sits ABOVE the per-layer
contracts and is the **single entry point** for the persist merge:

| Layer | Canonical contract | Where it lives at merge time |
| --- | --- | --- |
| L1 `.plr` | **research/99** (71-row offset table + gate matrix) | THIS branch |
| L2 `.trplayer` | **research/103** (v10 superset FINAL) + **research/122** (line-8 addendum) | g88 tip `6998a4ecb` + its in-flight v10 crystal fold (worktree, unpushed at this probe) |
| L2 fold-inputs | **research/135** (g90 six-line v7 + token map) + **research/138** (a6b reconcile — NOT pushed; its content read off the a6 worktree ANALYSIS, §3-L2 below) | g90 tip `b3b82d071` / a6 tip `8ed96ee5d` |
| L3 `.trwld` union | **research/92-doc-sweep-bg §1** (g71c five-branch 21-byte head + dense variants, worktree state) re-extended by **research/64-addendum** (g72d, the 31-byte cave claim) + 135 §4 (g90 books) — the reconciled FINAL layout in §3-L3 below | g71 worktree over `e2d085ae4` |
| L3 `.wld` flags | `wld/sections/flags.rs` per-field C# gates; f45's second-half claims co-lane | f45 tip `6352d3840` |

Code-only lane: witnesses written, not run.

Inputs read at the FINAL probe (2026-11-21 — tips this doc describes;
worktree-only states are marked):

| Ref | Tip read | Persist state |
| --- | --- | --- |
| `origin/task/W4-g89-aegis-fruit-use` | `598642563` (g89f over g89e `e9ec8dcfc` / g89d `4344bd50`) | `.trplayer` **v9** 3-token unlock line (research/122); `.plr` fruit/gummy/ambrosia bools both halves; `.wld` book-two byte (g89e); `.trwld` stays **v17** |
| `origin/task/W4-g88-life-fruit-use` | `6998a4ecb` (fold 5 docs) + **worktree v10 fold UNPUSHED** (research/103 updated in-tree: writer v10, tokens 4/5/6 LIVE crystal/arcane/pearl, §5a keep-own pending 138) | `.trplayer` **v10 SUPERSET** (research/103 FINAL — g89c+g90b+a6 folded); carries research/122 blob-verbatim |
| `origin/task/W4-g90-artisan-bread-use` | `b3b82d071` (reconcile-input over lane 7 `73367e0af`) | `.trplayer` **v7** six 1-token lines + `.plr` bools 1/3/4/5/6 + `.trwld` **v18** 2-byte tail; research/**135** is its mechanical fold-input |
| `origin/task/W4-a6-crystals-powerups` | `8ed96ee5d` (moved past `d46221b2b`: use arms 5337-5342/5043/5343/pets/books/drones) + **worktree a6b reconcile UNPUSHED** (the research/138 content: row-targeted achievements, pearl Player-field ownership, aegis f32 order, booster row order — ANALYSIS §W4-a6) | use arms + consumers; NO sidecar writer; `.wld` book-two/satchel/bought-pets bytes — overlaps g90 lane 7 / g89e / f45 |
| `origin/task/W4-f33-torch-god-favor` | `8a000407f` (torch-favor v7-line: `6f2c122e8` use arm → `7954d6ed1` docs → `a429b37c0`/`8a000407f` shimmer+desert arms) | **ANOTHER v7 fork**: `.trplayer` v7 ONE 2-token line (the biome-torch pair) + `.plr` `:54777-54778` both halves + the 279-subset bools 2-3 capture; the event/favor/949 machines (`torch_god.rs`, keep-one over d19 per research/93) |
| `origin/task/W4-g72-bg-layers-3` | `2b02345ce` (g72c: phases 7-8 DONE — the roll OWNED, `.wld` flags + `.trwld` v18 **31-byte cave-tail** + net header) + **worktree g72d UNPUSHED** (`108cc0934` research/64-addendum: the 53-byte five-claimant union layout; `7039b0638` d35 atlas cross-note) | `.trwld` v18 31 B (`cave_back_x[0..3]` 3×i32 + `cave_back_style[0..4]` 4×i32 + ice/jungle/hell u8); `.wld` flags arrays at C# i32 widths |
| `origin/task/W4-g71-bg-layers-2` | `e2d085ae4` + **worktree g71c UNPUSHED** (rolls WIRED into `step_world_tail`; research/92 §1 re-extended to FIVE branches: g70/g71/g69/d125/d109, **21-byte fixed head + W×H dense variants LAST**) | `.trwld` v18 union owner-to-be; the g72/g90 claims NOT yet seated in its §1 — the re-extend obligation, §3-L3 |
| `origin/task/W4-g70-bg-layers-1` | `6b3049e19` (persist at `bbe47378` lineage) | `.trwld` **v18** 16-byte forest tail |
| `origin/task/W4-g69-weather-clouds-parallax` | `9717c0bb` lineage (per g71c §1) | `.trwld` **v18** 1-byte crimson tail |
| `origin/task/W4-d125-empress-r7-drain` | `d8047a10` | `.trwld` **v18** 1-byte downedEmpress tail |
| `origin/task/W4-d109-framing-genrand-a3` | `81894095` | `.trwld` **v18** dense W×H framing-variants block (port-only, LAST) |
| `origin/task/W4-f45-wld-header-2` (+c43 `076970809` in lineage) | `6352d3840` | `.wld` header/flags second half OWNED (13 phases); **c43's `.trwld` v18 44-byte identity tail** (generator version/GUID/worldId/CreationTime/LastPlayed) + the `.plr` `.bak`/revision-carry/UtcNow envelope — the seventh v18 claimant + the L1 envelope co-lane |
| `origin/task/W4-g55-ui-loadout-save` | `cde865fc2` | `.plr` loadout family + `.trplayer` v7 loadout block (folded into g88's superset) |
| `origin/task/W4-e10-player-sounds-wave` | `2c024e444` | NO persist change — `voice_variant`/`voice_pitch_offset` are Player fields only (§4 below) |
| this branch | `63204f85` (FINAL: the slot-11 property fix over `05e4020b` crosspersist) | `.plr` complete (research/99) + the timestamp fix + the UsingBiomeTorches property seam; `.trplayer` stays main's **v6**; `.trwld` stays **v17** |

---

## 1. The three layers (what each file IS)

| Layer | File | Format | Writer / reader | Job |
| --- | --- | --- | --- | --- |
| **L1 `.plr`** | `Players/<name>.plr` | vanilla binary, AES-128-CBC, release **326** | `crates/tr-app/src/player_file.rs` | the CHARACTER: identity, appearance, stats, inventory/banks/loadouts, every per-player unlock/counter, the full `Player.Serialize` field set (`Player.cs:54750-54933`). Vanilla-interchangeable both ways. **Canonical field map: research/99's offset table** (71 rows, 3827-byte fresh layout). |
| **L2 `.trplayer`** | `<world>.trplayer` | port-invented versioned TEXT | `crates/tr-app/src/persist.rs` | the LIVE SESSION next to a `.trwld`: position/hp/coins + the fields the live App mutates and must restore on world reload. NOT a `.plr` mirror — a lossy projection (kinds-only equips, 50-slot inventory). **Canonical contract: research/103 (v10 superset FINAL) + research/122 (line-8 addendum); fold-inputs: research/135 (g90) + research/138 (a6b, unpushed — content in the a6 worktree ANALYSIS); the f33 v7 fork: §2d below.** |
| **L3 world** | `<world>.trwld` (native) / `<name>.wld` (vanilla) | port binary v17→v18 union / vanilla sections release 326 | `tr-core/src/save.rs` / `tr-core/src/wld/` | the WORLD: tiles, chests, NPCs, world-global flags. **`.trwld` union contract: research/92 §1 (g71c five-branch 21-byte head + dense) re-extended per §3-L3 below with g72d's 31-byte cave claim (research/64-addendum), 135 §4's book pair and c43's 44-byte identity block — the reconciled head is 97 bytes + W×H dense.** `.wld` flags codec: `wld/sections/flags.rs` (the C# gates). |

Side satellites (not layers): `<stem>.trfav` (the character star —
vanilla's `favorites.json` store, `IO/FavoritesFile.cs:27-36`; the `.plr`
metadata bit is dead in vanilla too), `<world>.map` (explored map),
`achievements.trdat`.

**The routing rule that keeps the layers apart** (`saveglue::save_world`,
`app/saveglue.rs:88-296`): a `.trwld` world saves native + the
`.trplayer`/`.map` sidecars; a `.wld` world saves vanilla-shaped and
writes NO player sidecar (`.wld` player state has no port format —
recorded ANALYSIS deviation). The `.plr` is written ONLY by the
character-creation path (`session.rs::save_created_character`) — there is
no in-session `.plr` re-save yet (the vanilla `Player.SavePlayer` per
world-exit is the recorded remainder). Consequence for this map: today
**no code path writes the same field to two layers in one save** — but
the g-lanes' sidecar extensions overlap the `.plr`'s field set, so the
ownership table below is what the merged save funnel must honour.

## 2. Field ownership — which fields live where (no dup-writes)

### 2a. L1-only (the `.plr` carries them; the sidecar must NOT grow lines for them)

Everything in research/99's table EXCEPT the §2b rows. The families the
sidecar has no business mirroring: name/difficulty/appearance/colors
(the character card), playtime, hairDye/team, the hide flags, banks
1-4 + voidVaultInfo, buffs, spawn points, the quest/death counters
(angler/dpad/builderAcc/bartender/golfer), dead+respawnTimer, the
research pairs, creative powers, temp slots, super-cart, refunds,
dialogues, hbLocked, the save timestamp, the metadata block. Also
**L1-only among the unlock family**: `extraAccessory` (`:54776`, g87's
demon-heart lane — research/122 §4.4 already pins "no sidecar line"),
`downedDD2EventAnyDifficulty`
(`:54787` — 103 §2a token 7 stays RESERVED; if a DD2 lane ever needs it
live, the token is there, but no pushed lane claims it), taxMoney, the
death counters.

**The biome-torch pair MOVED out of this list at the FINAL phase**: f33
`8a000407f` gives `unlockedBiomeTorches`/`UsingBiomeTorches` a live
sidecar carrier (its v7 fork, §2d) — they are §2b contested fields now.
**The `UsingBiomeTorches` half is a PROPERTY, not a field** (the FINAL
re-diff fix `63204f85`): the `.plr` `:54778` byte is the getter
`unlocked && builderAccStatus[11] == 0` (`Player.cs:3954-3958`) on write
and the setter `builderAccStatus[11] = !value` (`:3960-3963`) on read —
the effective toggle state lives in the L1-only `builderAccStatus[12]`
array (row 55), which the sidecar must NOT mirror wholesale; only the
derived bool rides L2. f33's live `Player::set_using_biome_torches`
already writes slot 11 (its ANALYSIS pins the setter); the merge dedups
the transport, not the semantics.

### 2b. The contested fields — BOTH layers carry them today (the boundary)

The g-lanes extended the sidecar with fields the `.plr` also carries.
**This is a deliberate dual-carrier, not an accident** — the two files
have different lifetimes: the `.plr` is written at character creation
(and, once the exit-save lands, at world exit), the sidecar at every
world autosave. A field the live session mutates (an upgrade eaten
mid-world) must ride the sidecar or it is lost between `.plr` writes;
the same field must ride the `.plr` or a vanilla round-trip loses it.

| Field | `.plr` slot (research/99 row) | `.trplayer` carrier (union) | Live owner |
| --- | --- | --- | --- |
| `statLifeMax`/`statLifeMax2` | rows 15 (+ effective hp in L2 line 2) | line 9 life pair (g88) | g88 |
| `statManaMax`/`statManaMax2` | row 17 | line 10 mana pair (g88) | g88 |
| `usedAegisFruit` | row 24 (`used_upgrades[1]`) | line 8 token 1 (g89) | g89 |
| `usedGummyWorm` | row 27 (`[4]`) | line 8 token 2 (g89b) | g89b (field pre-existed i35) |
| `usedAmbrosia` | row 28 (`[5]`) | line 8 token 3 (g89c) | g89c |
| `usedGalaxyPearl` | row 26 (`[3]`) | line 8 token 4 RESERVED→g90 fold | g90 lane 7 |
| `usedArcaneCrystal` | row 25 (`[2]`) | line 8 token 5 RESERVED→g90 fold | g90 lane 7 |
| `usedAegisCrystal` | row 23 (`[0]`) | line 8 token 6 RESERVED→g90 fold | g90 lane 7 |
| `ateArtisanBread` | row 21 | line 11 (g90, folded g88) | g90 |
| `unlockedBiomeTorches` | row 19 (`:54777`, gate 229) | f33 v7 pair-line token 1 → fold to a NEW post-loadout line (§2d) | f33 |
| `UsingBiomeTorches` (the slot-11 getter) | row 20 (`:54778`) + row 55 slot 11 | f33 v7 pair-line token 2 → same new line | f33 (transport); the slot-11 array stays L1-only |
| `CurrentLoadoutIndex` + `Loadouts[3]` | rows 65-66 | lines 12-15 (g55, folded g88) | g55 |
| inventory/armor/dye/coins/hp/pos | rows 40-42 (+ stats) | lines 2-7 (v1-v6 prefix) | pre-existing |

**The no-dup-write rules for the merged save funnel:**

1. **One writer per file per save.** `player_file.rs` writes L1;
   `persist.rs` writes L2; `save.rs`/`wld` write L3. No lane's writer
   opens another layer's file (today: `persist.rs` never touches the
   `.plr`; `player_file.rs` never touches the sidecar — verified by the
   module reads). Keep it that way.
2. **Within L1, one field slot per field.** The g89/g90 `.plr` halves
   and this lane's G7 `used_upgrades[6]` array model THE SAME SIX BYTES
   (`:54781-54786`). At the merge, keep ONE model: this lane's
   `used_upgrades[6]` (file-order array, both models, all three shim
   depths ride it) is the superset — g89's three named bools map to
   indices 1/4/5, g90's five to 0/2/3/4/5. The named-field lanes'
   `PlayerFile`/`PlayerFileWrite` fields dedup onto the array (or the
   array onto six named fields — either way ONE set, never both).
   Same for `ate_artisan_bread` (g90's field == this lane's G7 field,
   same name, same slot — a clean textual dedup).
3. **Within L2, one token per field.** 103 §2a's reserved positions are
   the single registry; g90's six-line v7 block folds into tokens per
   research/135 §2-3 (its lines are a LEGACY READ shape, never a second
   write shape). The 103-vs-122 token-order conflict for 4/5/6 is
   resolved by 135 §3: **103 §2a wins** (pearl=4/arcane=5/crystal=6) —
   no pushed writer ever emitted token 4+, so no back-compat obligation
   attaches to either order.
4. **L3 world globals never ride L2.** `combatBookWasUsed` /
   `combatBookVolumeTwoWasUsed` / `peddlersSatchelWasUsed` /
   bought-pets are WORLD flags (`NPC.cs:6339-6343`, `WorldFile.cs:1392-1394`/
   `:1407-1408`) — `.wld` flags block + the `.trwld` union tail only.
   Never a sidecar token (122 §5 already pins this; g90's use arms write
   the World fields, not Player fields).

### 2c. The G7-vs-sidecar semantics census (the brief's question)

**Is the sidecar a port-invention for pre-unlock state?** No — the
sidecar bools and the `.plr` G7 bools are THE SAME vanilla fields
(`Player.usedAegisFruit` etc., ctor `:56078-56083`), captured at
different times. The semantic difference is the WRITE MOMENT, not the
meaning:

- **G7 (this lane)** models the `.plr` slots so a VANILLA character's
  already-used upgrades survive a port read→re-save (the audit's loss
  case). It invents no state; `PlayerFile.used_upgrades` is a verbatim
  byte carrier. The live `Player` on THIS branch has only
  `used_gummy_worm` (i35) + `ate_artisan_bread` (reach formula) — the
  other four G7 slots have no live consumer here; the g-lanes own them.
- **g89/g90 sidecar bools** carry the LIVE session's unlock state
  (the use arms set `Player.used_*`, the booster rows read them every
  tick) across world reloads, because the `.plr` is not re-saved
  in-session. Same field, same meaning — a second carrier for a
  different save cadence.
- **The merge boundary:** the live `Player.used_*` fields (g89/g90's
  declarations) are the single live truth; the sidecar restores them at
  world load (`app/mod.rs`'s `persist::load` block); the `.plr` G7
  carrier restores them at CHARACTER load (once the character-load path
  hydrates a live Player — today the `.plr` reader feeds only the card
  list, `save_list.rs:294`). When both restores exist, the character
  load runs FIRST (App::new spawns the Player), the sidecar restore
  second (world load) — the sidecar wins for a world that was saved
  after the character file, which is the correct precedence (it is the
  newer state). A `.plr`-only field (extraAccessory etc.) has no sidecar
  line, so no precedence question arises.

### 2d. The f33 torch-favor v7 fork (the EIGHTH v7 collision — FINAL-phase input)

`origin/task/W4-f33-torch-god-favor` tip `8a000407f` (read at the FINAL
probe; the persist half landed at `6f2c122e8`, docs `7954d6ed1`). Its
`persist.rs` diff against main:

- Writer bumps **v6 → v7** and appends ONE line after the dye line:
  `unlockedBiomeTorches UsingBiomeTorches` — **TWO 0/1 tokens**
  (`Player.cs:54777-54778`; the second token is
  `player.using_biome_torches()`, the slot-11 GETTER, not a stored bool).
- Reader: `"7"` joins the no-migrate arm; armor/kills/dye gates widened
  to `"7"`; the pair line is REQUIRED at v7
  (`lines.next()?`), tokens read `is_some_and(|v| v == "1")` — a missing
  second token reads false (token-count tolerant).
- `SavedPlayer` gains `unlocked_biome_torches` + `using_biome_torches`;
  the App restore rides `enter_loaded_world`.
- Its `.plr` half captures the 229-block's first two bools into named
  fields on the main-base reader + the 279-subset's bools 2-3, and flips
  the writer's two hardcoded `false`s to the fields. **All of that is
  already covered by THIS branch's G7 walk + the FINAL slot-11 fix** —
  the fold deletes f33's `.plr` diff except its live
  `set_using_biome_torches` setter (which this branch's `Player` lacks;
  keep f33's).

**The collision:** f33's 2-token v7 line is a NEW shape in 103 §1's
collision table (which knew g89 1-token / g88 2-token LIFE PAIR / g90
1-token bread / g55 loadout / g90b six-line). Under the superset's v7
walk a 2-token first post-dye line reads as the **life pair** — f33's
`(1,1)` would land as `statLifeMax=1, statLifeMax2=1` (the 103 §5
pairs-before-clamp corruption class, worse than the flag swaps: it
feeds hp). f33's own ANALYSIS proposed a v8 both-line; that proposal is
**SUPERSEDED** — v8/v9/v10 are taken (g89b/g89c/g88-crystal). The fold:

1. **Recognition rule (extends 135 §2's):** a v7 post-dye region of
   EXACTLY ONE line with EXACTLY 2 tokens whose values are both 0/1 is
   AMBIGUOUS with g88's life-pair line `(statLifeMax, statLifeMax2)` —
   resolve by VALUE RANGE: a life pair's first token is ≥ 100 for any
   real character (the ctor 100, clamped ≤ 500 on read); f33's tokens
   are 0/1 only. Rule: `post.len() == 1 && post[0].len() == 2 &&
   post[0].iter().all(|t| t == "0" || t == "1")` → f33 torch pair;
   otherwise the 103 §3 walk. (A hypothetical 1-hp life pair cannot
   exist — `statLifeMax` never drops below 100 in vanilla.)
2. **Union seat:** the torch pair appends as a NEW 2-token line AFTER
   the loadout block (103 lines 13-15) — line 16, writer stays **v10**
   (append-only, the short-line back-read defaults it false for every
   pre-fold file; no version bump needed since no pushed v10 writer
   emits line 16 yet — the same forward-only argument as the crystal
   tokens). Alternatively tokens 8/9 on the unlock line — NOT
   recommended: the unlock line is the upgrade-bool registry, the torch
   pair is an unlock+toggle, and 103 §2a's token 7 is DD2-reserved.
3. **Restore:** `p.unlocked_biome_torches = line16[0];
   p.set_using_biome_torches(line16[1])` — the setter writes slot 11,
   matching the `.plr` read side's setter semantics (§2a).
4. f33's two sidecar witnesses (`w4_f33_the_biome_torch_pair_round_trips_the_sidecar`,
   `w4_f33_version_6_loads_without_the_torch_unlock`) re-pin onto the
   superset writer; its `.plr` witness
   (`w4_f33_the_biome_torch_pair_round_trips_the_plr`) dedups onto this
   branch's `the_using_biome_torches_byte_is_the_slot11_property` (the
   f33 version asserts the raw bool round-trip — TRUE under the slot-11
   model only when unlocked; the property witness is the superset).

**The g88e-fold-input pointer:** research/103's §1 collision table +
§4 back-compat matrix gain the f33 row at the g88 fold (the same edit
that seats 135's six-line rule); this §2d is the mechanical input.

## 3. Per-layer merge order (the FINAL merger persist-checklist)

**Order: L1 → L2 → L3.** L1 first because the `.plr` is the vanilla
interchange and the field-superset every other layer projects from; L2
second because its restore block reads the live `Player` fields L1's
fold keeps; L3 last because the world union is independent of both
(no player field rides it) and its arithmetic depends on the g-lane
tips being final.

### L1 `.plr` (player_file.rs — hot file, five lanes touch it)

1. **Base = THIS branch** (`63204f85` FINAL): the complete 71-row walk,
   the G1-G7 fixes, the three shim depths, the timestamp fix, the
   slot-11 property fix. It is the only branch whose `player_file.rs`
   models EVERY field; the g-lanes' versions are main-base + their own
   slots.
2. **Fold g55's loadout semantics** (the swap machine's
   `set_shared`/fix-loaded-data hooks ride this branch's
   `LoadoutData` transport; byte layout identical — g55's
   `EquipmentLoadout.cs:31-45` walk == research/99 rows 65-66).
3. **Dedup g89's + g90's named unlock fields onto `used_upgrades[6]`**
   (§2b rule 2). Their read arms (`:55255-55260`) and write arms
   (`:54781-54786`) are already covered by the G7 array walk — the
   fold DELETES their duplicate reads/writes, keeps their live
   `Player.used_*` fields + use arms + booster rows. g90's legacy-279
   indices 4/6/7/8/9 already ride this branch's subset capture.
4. **Dedup f33's torch-pair fields** (§2d): its `.plr` named-bool
   capture == this branch's rows 19-20; DELETE its reader/writer diff,
   KEEP its live `Player::set_using_biome_torches` (the slot-11 setter)
   + the use arm + the event machines. Its raw-bool round-trip witness
   re-pins onto the property witness. **Do NOT resurrect a stored
   `using_biome_torches` live field** — the C# has none; the toggle is
   `builderAccStatus[11]`.
5. **Fold c43's envelope** (`076970809`, in f45's lineage): the `.bak`
   copy before a re-save (`InternalSavePlayerFile:54730-54733`), the
   metadata revision CARRY (`PlayerFileWrite::revision` — this branch's
   writer still hardcodes 0 at the prologue; the carry is the one
   remaining L1 envelope gap, byte-neutral for created characters),
   its UtcNow creation stamp dedups onto this branch's
   `utc_now_binary()` (same semantics, keep one helper).
6. **Keep the timestamp fix whole** (`read_player_file`'s mtime
   fallback + `utc_now_binary`); g89/g90/f33's `last_saved_binary: 0`
   literals in their session/test diffs dedup onto this branch's
   `utc_now_binary()` (session) / stay 0 (fixtures).
7. Anchors: this lane's rows stay; the g-lanes' `.plr` witnesses
   re-pin onto the array fields (their assertions are slot-value
   assertions — mechanically portable).

### L2 `.trplayer` (persist.rs — the g88 superset wins)

Per research/103 §6 (FINAL v10) + research/122 §4 + research/135 §6 +
§2d above, in order:

1. **Base = g88's v10 fold** (worktree over `6998a4ecb` at this probe —
   the writer v10 superset, tokens 1-6 LIVE, the tolerant v7/v8/v9/v10
   reader, the loadout block, research/103+122 in-tree). If the fold is
   still unpushed at merge time, the pushed v9 tip + 103's §7 receipt
   is the mechanical spec to re-apply it.
2. **Extend the v7 legacy arm** with 135 §2's six-1-token-line rule
   (g90 lane-7 files load positionally) AND §2d's one-line-2-token-0/1
   rule (f33 torch pair; the value-range disambiguation vs the life
   pair). Every other v7 shape keeps 103 §3's walk.
3. **Tokens 4/5/6 are LIVE at v10** in the corrected vanilla-relative
   order crystal=4/arcane=5/pearl=6 (103 §2a's FINAL map — the pre-v10
   reservation listed pearl=4; no pushed writer ever emitted token 4+,
   so the correction carried no back-compat cost). Short-line
   back-reads for v7/v8/v9 kept.
4. **Append the f33 torch-pair line** (line 16, after the loadout
   block; §2d rule 2) — writer stays v10, short-line back-read defaults
   false. Restore via the slot-11 setter.
5. **Dedup** g90's `SavedPlayer` flag fields + App restores onto the
   superset's (gummy/ambrosia/bread/crystals already folded); f33's
   pair fields join the same restore block.
6. **Seat research/138's adjudications when it pushes** (103 §5a's
   keep-own is provisional): the a6b worktree ANALYSIS already records
   the four reconcile fixes — row-targeted `complete_condition` (c36's
   shape, NOT the `notify_custom("Use")` broadcast), the galaxy-pearl
   Player-field ownership (`update_luck` mirrors into `LuckState`),
   the aegis-crystal +0.2 f32 slot BETWEEN base +1 and crimson +1, the
   booster row order fruit→gummy→ambrosia. These are use-arm/consumer
   dedups, NOT persist-layout changes — the sidecar contract is
   unaffected either way.
7. This lane contributes NOTHING to L2 (main's v6 reader/writer is a
   strict prefix of the superset; the merge takes g88's file whole).

### L3 world (save.rs `.trwld` + wld/ `.wld`)

1. **`.trwld` base = the g71c union RE-EXTENDED to seven claimants.**
   The g71c worktree's research/92 §1 seats five (g70 16 + g71 3 +
   g69 1 + d125 1 = 21-byte head, d109 dense LAST). Two more claims
   must be seated before the union lands:
   - **g72d's 31-byte cave block** (research/64-addendum, worktree
     `108cc0934` over `2b02345ce`): its §2 recommends the cave block
     BETWEEN g70's forest block and g71's surface bytes (the C#
     `:1260-1269` field order). The g71c §1 layout does NOT carry it
     yet — **the re-extend obligation this FINAL phase records**.
   - **g90's 2-byte book pair** (135 §4): after the flag singles.
   - **c43's 44-byte identity block** (`076970809`: u64 generator
     version + 16 GUID + i32 worldId + 2×i64 ToBinary stamps) — the
     seventh claimant, discovered at this FINAL probe; identity fields
     are not bg-family, so they close the fixed head after the books.
   The reconciled FINAL layout (one fixed shared order, extending both
   92 §1's and 64-addendum §2's rationale):

   ```
   … v17 downedFishron byte (1)
   v18 UNION tail — fixed head (97 bytes) + dense variants (W×H):
     [0..16]   g70 forest block        (tree_x 3×i32 + styles 4×u8)
     [16..47]  g72 cave block          (3×i32 bands + 4×i32 styles
                                        + ice/jungle/hell u8)  — 64-add §1
     [47..50]  g71 surface bytes       (snow/desert/ocean u8)
     [50]      g69 crimson             u8
     [51]      d125 downedEmpress      u8
     [52..54]  g90 book-two + satchel  u8×2               — 135 §4
     [54..97]  c43 identity block      (u64 + 16B GUID + i32 + 2×i64 = 44 B)
     [97..97+W*H]  d109 dense variants (LAST, self-delimiting)
   ```

   **Fixed head = 97 bytes: 16+31+3+1+1+2+44. Total = 97 + W×H.**
   VERSION stays 18; forward-only (no released v18 — every claimant is
   unmerged task work). `v18_tail_len` returns `97 + width*height`.
   The truncation-constant moves: g70/g71's 16/3 → 97 (92 §1.5's table),
   g69/d125's 1 → 97, g90's six sites 2 → 97 (135 §4's table), g72's
   eleven sites 31 → 97 (64-add §3's list), c43's 44 → 97; d109's
   end-relative sites need only the helper's new return value. Every
   claimant's round-trip witness keeps its assertions, only the tail
   lengths move. The `underworldBG` seat (64-add §3's flag, C# `:1376`)
   stays OPEN — if g71c's roll-wire persists it, insert 1 byte between
   the surface group and crimson → 98-byte head, re-pin once.
2. **The roll keep-one (g71c vs g72c):** BOTH branches now port
   `RandomizeCaveBackgrounds` — g71c's `bg_rolls.rs` (WIRED into
   `step_world_tail`, worktree) and g72c's `worldgen/cave_bg.rs`
   (pushed, the `CaveRoll` trait driver). g71c's 92 §2 seam contract
   claims canonical status; g72c's is pushed and carries the boot-side
   Knuth driver + the width-arm drawn-count fix. **Adjudication owed at
   the merge** (the docs disagree; both are faithful ports of
   `WorldGen.cs:8192-8249`): keep ONE roll implementation, keep g72c's
   boot dead-store seam (`App::new`, research/64 §2 correction) and
   g71c's surface rolls; the persistence trio (`.wld` flags / `.trwld`
   union / net header) is g72c's pushed shape either way. The
   `world_hash` stale-by-design notes ride both.
3. **`.wld` flags dedup**: g89e, g90 lane 7, a6 AND f45 Ph12 ALL write
   `combatBookVolumeTwoWasUsed` (gate 259) — keep ONE (f45's claim is
   the pushed `.wld`-specialist lane; g90's pair also owns the satchel
   byte, gate 260; a6/f45 Ph11 add the bought-pets trio, gates 217,
   tail 2021-2023 — independent bytes, no conflict). g72c's cave arrays
   (i32 widths, `WorldFile.cs:1260-1268`/`:2058-2066`) vs f45 Ph6's
   tree/cave claim — same fields, same gates; keep one codec half
   (f45's flags file is the superset; g72c's arrays must survive it).
4. This lane contributes NOTHING to L3 (no save.rs/wld change).

## 4. The e10b voice co-lane (research/99 row 67 cross-note)

**`origin/task/W4-e10-player-sounds-wave` tip `2c024e444`: NO `.plr`
write landed.** The branch's diff against main touches neither
`player_file.rs` nor `persist.rs` (verified `git diff --stat` — empty
for both). What it DOES carry:

- `Player.voice_variant: i32` (ctor 1, `Player.cs:1346`/`:56045`),
  `voice_pitch_offset: f32` (`:1348`), `voice_override: i32` (`:1706`)
  — live fields feeding the ported `PlayHurtSound`/`PlayDeathSound`
  decision trees (`player.rs:1933-1948` at its tip).
- Its ANALYSIS deviation note: "**No voice-picker / save-load
  surface.** `voiceVariant` stays at the ctor's 1 ... until a
  character-creation/save surface lands."

**Row 67 status (this lane):** COMPLETE on the `.plr` side —
`voice_variant: u8` + `voice_pitch_offset: f32` ride both models
(`player_file.rs:361/364`, write `:54924-54925` slots, read gates
280/281 with the <280 Male-derive `{0,1,2,3,8,10}` and the
`LastMinuteFixes` 1..4 clamp `:55831`). The golden fixture's fresh
voice byte 1 + pitch 0f are pinned.

**The merge seam (clean, no conflict):** e10's live field is `i32`,
this lane's file field is `u8` — the hydration is
`p.voice_variant = file.voice_variant as i32` (the 1..4 clamp already
applied on the read side) and `p.voice_pitch_offset =
file.voice_pitch_offset`, at the future character-load restore block.
The save direction casts back (`file.voice_variant = p.voice_variant
as u8`). e10 owns the CONSUMERS (the voice trees, the pitch remap
`Utils.Remap(-1,1,-0.7,0.7)`); this lane owns the TRANSPORT. Neither
branch needs a change for the other to land — the fields are declared
at different layers with no textual overlap. The character-creation
voice picker (S1-H-11's clipboard note: the creator writes the gender
default `Male ? 1 : 2`) is the remaining UI seam, not a persist gap.

## 5. The back-compat matrix (cross-layer)

| File shape | Layer | Reader that loads it | Result |
| --- | --- | --- | --- |
| vanilla `.plr` 135-326 | L1 | this branch `read_player_fields` | full gate matrix (research/99 §2); 135-137 no playtime, <136 misc slot-1 skip |
| vanilla `.plr` <135 | L1 | refused (`pre-135`) | the legacy string-item layout is unmodelled (documented) |
| port 279 subset | L1 | `read_legacy_port_subset` | header + armor/dye/inv; tail defaults; G7 bools ride |
| port 326 inc1/inc2 | L1 | depth-1/2 retries | reached sections ride, rest defaults (research/99 §4) |
| port `.plr` 0-stamp | L1 | `read_player_file` | **mtime fallback fills the stamp** (`d331ea7c`, `LoadPlayer:55096-55099`); byte-level read keeps 0 |
| g89/g90/f33 `.plr` (main-base + their slots) | L1 | this branch's walk | their slots ARE G7/sweep slots — loads complete; their own readers load this branch's files too (same bytes; f33's raw torch bool reads through the slot-11 setter, §2a) |
| `.plr` 229..229-only torch block (no builder array) | L1 | this branch's walk | the `:55242` setter's slot-11 value STANDS (the <230 count never reads slot 11) — the C# order, `63204f85` |
| `.trplayer` v1/v2 | L2 | any lane reader | kinds migrate (`migrate_kind`) |
| `.trplayer` v3-v6 | L2 | any lane reader | prefix rides, post-dye defaults |
| `.trplayer` v7 (5 historical shapes + g90 six-line + f33 pair-line) | L2 | g88 superset + 135 §2 rule + §2d rule | all load (103 §4 matrix + the six-line row + the torch-pair row) |
| `.trplayer` v8/v9 short lines | L2 | g88 superset / g89 reader | tokens 1-3 back-read, rest false |
| `.trplayer` v9 g88-superset | L2 | g89's reader | line 8 reads; pairs/bread/loadout lines IGNORED (g89's reader stops after line 8 — lossy but aligned; the superset reader is the merge winner) |
| `.trplayer` v9 g89-file | L2 | g88 superset | unlock line positional, rest defaults |
| `.trplayer` v10 short unlock line (3-token) | L2 | g88 v10 reader | tokens 1-3 read, crystal trio false (103 §7 back-read) |
| `.trplayer` v10 pre-torch (no line 16) | L2 | the §2d-extended reader | torch pair defaults false (append-only back-read) |
| `.trplayer` v7 g90-six-line | L2 | g89's reader | **CORRUPTS** (bread→aegis; 135 §2) — the fold's recognition rule is mandatory |
| `.trplayer` v7 f33-pair-line | L2 | g88 superset WITHOUT the §2d rule | **CORRUPTS** (torch pair → life pair, hp feeds 1/1) — the value-range rule is mandatory |
| `.trplayer` v7 f33-pair-line | L2 | f33's own reader | loads (its native shape); g90/g88 files' first line mis-reads as the pair — f33's reader is NOT the merge winner |
| `.trwld` ≤17 | L3 | any v18 reader | union tail unread, Reset defaults |
| `.trwld` v18 single-claimant (g70 16B / g71 3B / g69 1B / d125 1B / g90 2B / g72 31B / c43 44B / d109 dense-only) | L3 | the 97-byte-head union reader | **EOF/garbage — not back-compat** (92 §1 + 64-add §3: forward-only, no released v18; all eight are unmerged task work) |
| `.wld` any release | L3 | `wld::parse` gates | per-field gates; book-two 259 / satchel 260 / pets 217 ride after the g90/a6/g89e/f45 dedup; the cave arrays ride g72c/f45-Ph6's one codec half |

## 6. The six pre-existing divergences — re-verified (C# re-read)

Research/99 §5's list, each re-checked against the decompile this pass:

1. **Write timestamp caller-stamped vs UtcNow** — **NARROWED, partially
   fixed (`d331ea7c`).** C# re-read: `Serialize:54909` writes
   `DateTime.UtcNow.ToBinary()` unconditionally; `LoadPlayer:55096-55099`
   falls back to the file mtime on a zero stamp. The port now: (a) the
   creation save stamps the real clock (`utc_now_binary`), (b) the disk
   read ports the mtime fallback. REMAINING deliberate residue: the
   byte-level writer stays caller-stamped (the golden fixture pins
   FixtureClock — a clock inside the codec would break byte-parity),
   and the stamps carry whole seconds, not the C#'s sub-second ticks
   (no consumer reads finer than a date). Witnesses:
   `the_utc_binary_encoding_pins_the_epoch_and_the_fixture_clock`,
   `a_zero_stamp_falls_back_to_the_file_mtime_on_the_disk_read`.
2. **Metadata favorites u64 always 0** — **STILL DELIBERATE, now
   provably C#-identical for the port's write path.** C# re-read:
   `FileMetadata.Write` (`FileMetadata.cs:22-27`) writes
   `IsFavorite.ToInt()`, but `IsFavorite` is only ever set false
   (`FromCurrentSettings:41`); the live star lives in `favorites.json`
   (`FileData.cs:35` reads `LocalFavoriteData.IsFavorite`, NOT the
   metadata). `CreateAndSave` (`PlayerFileData.cs:53`) builds fresh
   false metadata. So a vanilla-created `.plr` ALSO carries 0 here —
   the port's `.trfav` sidecar is the faithful analogue of
   `favorites.json`, not a divergence from the file bytes. The reader
   still ORs the header bit (`save_list.rs:300`) for foreign tools.
3. **Revision u32 always 0** — **STILL DELIBERATE, now provably
   C#-identical.** C# re-read: `InternalSavePlayerFile:54739` calls
   `playerFile.Metadata.Write` — NOT `IncrementAndWrite`. The ONLY
   `IncrementAndWrite` call in the decompile is `WorldFile.cs:1175`
   (the WORLD file). A vanilla `.plr` re-save keeps its revision
   forever (0 for a created character). The port's constant 0 matches
   every vanilla `.plr` the port can produce. (The `.wld` writer DOES
   increment — `wld/mod.rs`, witnessed by
   `test_DW_7_3_..._increments_the_revision`.)
4. **Legacy -1..-48 ids ride verbatim** — **STILL DELIBERATE.** C#
   re-read: `netDefaults` (`Item.cs:1211-1361`) maps -1..-48 to real
   items (3521..3480 etc.); the port keeps the negative id so a re-save
   round-trips the bytes. The mapping needs the item-defaults tables
   in the card model (the card carries no item stats) — deferred, the
   byte transport is lossless. The below-(-48) band airs correctly
   (G3, `Item.cs:1213`'s switch has no case).
5. **<253 cart-derive equip stand-in** — **STILL DELIBERATE.** C#
   re-read: `HasItemInAnyInventory(3353)` (`Player.cs:55931-55970`)
   walks inventory/armor/dye/miscEquips/miscDyes/banks via `HasItem`
   (`:55906-55916`, `type == && stack > 0`). The port's derive
   (`player_file.rs:1062-1069`) covers ALL nine collections in the C#
   order; the equip arrays carry no stack in the card model, so a
   non-air equip slot stands in for stack>0. A file that actually
   parks a cart in an equip slot with stack 0 is the only divergence
   case — vanishingly rare, documented.
6. **Dialogues first-occurrence order** — **STILL DELIBERATE.** C#
   re-read: `oneTimeDialoguesSeen` is a `HashSet<string>`
   (`Player.cs:3209`); `Serialize:54927` snapshots it into a List
   (insertion-order in practice for an add-only set), `Deserialize:55760`
   `.Add`s (dedupes). The port's Vec keeps first occurrences on both
   sides — the practical HashSet enumeration order, byte-identical for
   every add-only history. A remove-then-readd history could reorder
   the C# set; no port code removes dialogues (`OneTimeDialogue.cs:23`
   is the only writer, add-only).

**No new divergence introduced by the crosspersist pass** — the
timestamp fix moves the port TOWARD the C# on both the write and read
sides; the golden-fixture witnesses are untouched (the codec's
caller-stamped contract is preserved).

**FINAL-phase addendum (`63204f85`): a SEVENTH gap found and fixed by
the last full re-diff** — the `:54778`/`:55242` `UsingBiomeTorches`
byte was modelled as a stored bool on both sides, but the C# member is
a PROPERTY over `builderAccStatus[11]` (`Player.cs:3950-3963`): the
write emits the GETTER (`unlocked && slot == 0`), the read lands on the
SETTER (`slot = !value`) with the later >=230 builder-array read
overwriting the slot in C# order. The G7 sweep's raw-bool transport
round-tripped the byte but LOST the slot-11 link (a locked character
with a stray `using=true` would have written a true byte the C# never
writes; an unlocked+disabled character's toggle state had no carrier).
Fixed on the full walk + the 279 subset; the raw field stays as the
file-verbatim carry for the f33 fold's shape parity. This was the ONLY
remaining unmodelled-semantics field after the re-diff — every other
Serialize/Deserialize line maps 1:1 (research/99's table re-verified
against `Player.cs:54750-54933`/`:55134-55764` once more at this phase;
the buffNoSave zeroing, the spawn sentinel, the dead-gated respawn
timer, the temp-slot BitsByte, the powers reset-by-difficulty and the
dialogues dedup all already ride).

## 7. Witnesses (this pass — written, code-only, not run)

- `the_utc_binary_encoding_pins_the_epoch_and_the_fixture_clock` —
  the DateTime binary encoding pins, independently derived via .NET
  `DateTime.ToBinary` (epoch `0x489F7FF5F7B58000`, FixtureClock
  `0x48DF4506E391C000` = Unix 1794960000).
- `a_zero_stamp_falls_back_to_the_file_mtime_on_the_disk_read` — the
  0-stamp file surfaces its mtime through `read_player_file`, the raw
  byte walk keeps 0, a nonzero stamp rides verbatim.
- FINAL phase: `the_using_biome_torches_byte_is_the_slot11_property` —
  locked-writes-false / using-true / disabled-decouple / read-restore /
  byte-identical rewrite (offsets 67-68 + 2836); the header round-trip
  witness re-pinned onto the slot-11 assertion.

**Verification performed:** direct C# re-reads (`Player.cs:54680-54939`,
`:55060-55134`, `:55216-55339`, `:55686-55974`, `:3945-3974` (the
property), `:54770-54794`/`:55235-55279` (the unlock block re-diff);
`Item.cs:1211-1364`, `:48660-49069`, `:49875-49939`;
`IO/FileMetadata.cs` whole; `IO/FavoritesFile.cs:1-60`;
`IO/PlayerFileData.cs:1-75`), the g-lane persist diffs at their fetched
tips (f33 `8a000407f` full persist/player_file diffs; g72 `2b02345ce`
save.rs; c43 `076970809` save.rs identity block), the worktree-only
doc states (g71c research/92 five-branch §1, g72d research/64-addendum,
g88 research/103 v10, a6b ANALYSIS reconcile), a code-only brace/paren
balance sweep of the touched file, and a full `git diff` re-read of
this phase's change. No cargo/tests/rustfmt/graphify per the lane
constraint.

## 8. The FINAL merger persist-checklist (condensed, in order)

1. **L1** (§3-L1): take THIS branch's `player_file.rs` whole → fold
   g55 semantics → dedup g89/g90 named bools onto `used_upgrades[6]` →
   dedup f33's torch transport (keep its live setter) → fold c43's
   `.bak`/revision-carry envelope → keep the timestamp helpers whole.
   Recognition rules: none needed (binary, version-gated); the gate
   matrix is research/99 §2.
2. **L2** (§3-L2): take g88's v10 superset `persist.rs` whole → seat
   135 §2's six-line rule + §2d's torch-pair value-range rule in the
   v7 arm → append the torch line 16 → dedup the `SavedPlayer`/restore
   fields → seat 138's use-arm adjudications when pushed. Recognition
   rules: 103 §3 (positional v8/v9/v10) + 135 §2 (six 1-token lines) +
   §2d (one 2-token 0/1 line) + the lone-1-token aegis-first ambiguity
   (103 §3, unchanged).
3. **L3** (§3-L3): re-extend the g71c union to the 97-byte head +
   dense layout above (seat g72d's 31 + g90's 2 + c43's 44) → move
   every claimant's truncation constants → adjudicate the g71c/g72c
   roll keep-one → dedup the `.wld` book-two/satchel/pets/cave-arrays
   across g89e/g90/a6/f45/g72c. Dedup steps: one writer per file, one
   slot/token per field, world globals never ride L2 (§2b rules 1-4).
4. **Witness re-pins**: every lane's round-trip witnesses keep their
   assertions; only tail lengths (L3) and field names (L1 array dedup,
   L2 token map) move. The back-compat witness-matrix is §5 — run it
   as the merge acceptance list.
5. **Canaries**: add/keep the anchors rows for the union reader, the
   superset writer and this branch's walk; the golden `.plr` fixture
   stays byte-identical through every step (all fold defaults emit the
   old hardcoded bytes).