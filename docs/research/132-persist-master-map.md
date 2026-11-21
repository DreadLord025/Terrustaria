# 132 — The persist MASTER MAP: three layers, one owner per field

**Status: SEALED + RE-PINNED — THE PERSIST-FAMILY BIBLE FINAL (seal
phase + the post-seal receipts pass).** Written
by W4-f44 crosspersist (branch `task/W4-f44-save-plr-2`, base `180b7430`
audit DONE + the `d331ea7c` timestamp residual fix), closed by the FINAL
phase (`63204f85` slot-11 property fix), re-opened and closed by the
RETIRE phase (`d35876af9`), and SEALED by the seal phase (`0d1ee2045`
docs over the `5c8bbb160` hydration port): (1) the 132↔92
convergence receipt — g71 `f2d8c2c67`'s independently-derived 107-byte
union byte-order-compared, the offsets adjudicated C#-write-order
definitive, the FINAL-SEAL layout published (§3-L3); (2) the g88h seal
receipt — `6a829a0e8`'s 100-byte step-10 superseded, the `dh` line
adjudicated DEFAULT-DROP with §2e as the conditional seat (§2f);
(3) the character-load hydration hunk CLOSED by the code port — the
e10c receipt (§4), the exit-save the one remaining shared remainder.
The post-seal receipts pass adds: (4) the **D5-A3 re-pin receipt** —
g88 `d09d7e10a`'s L3-ARBITER FINAL read FULL, its A4 132-row owed
re-pin EXECUTED (§3-L3 rule 1 — the FINAL-SEAL table already carried
A3's layout; the receipt + the historical Δ pins close the loop,
g71 `b27474a44`'s "owes" row now CLOSED); (5) the **g56 `.trvault`
fold-contract receipt** (§2g — the two-seam retire gate composed with
this doc's exit-save remainder: condition (a) MET by the hydration
port, condition (b) the ONE shared gate both contracts wait on).
This doc sits ABOVE the per-layer contracts and is the **single entry
point** for the persist merge:

| Layer | Canonical contract | Where it lives at merge time |
| --- | --- | --- |
| L1 `.plr` | **research/99** (71-row offset table + gate matrix) | THIS branch |
| L2 `.trplayer` | **research/103** (**v11** superset FINAL — the torch fold landed) + **research/122** (line-8 addendum + the v11 post-receipt) | g88 tip `a9f1c29d8` (PUSHED — the v10/v11 folds are no longer worktree-only) |
| L2 fold-inputs | **research/135** (g90 six-line v7 + token map — §2 rule LANDED in g88) + **research/138** (a6 4-way master reconcile — **PUSHED** at `d7de8bd53`; its verdicts seated in §3-L2 below) | g90 tip `94d33f2ca` / a6 tip `d7de8bd53` |
| L3 `.trwld` union | **THIS doc §3-L3 is THE authoritative union** (107-byte head + W×H dense, the FINAL-SEAL order = **103 L3-ARBITER A3 / D5-A3**, `d09d7e10a` — the byte-order authority of record, re-pin receipt §3-L3 rule 1). research/92-doc-sweep-bg **§1-FULL** (g71's 13-branch registry — the claimant registry of record; its §1-FULL.2 re-pinned to A3 at `8224c94f1`, receipts closed `b27474a44`) + research/64-v18-union-addendum (g72, PUSHED, D5-RE-PIN `aa1645481`) + 135 §4 (g90 books) + 138 §5 (f45 identity) are the per-claimant inputs; where they disagree on the total (21/53/65/97/98/100), §3-L3 wins | the union lands at the merge; no single branch carries it yet |
| L3 `.wld` flags | `wld/sections/flags.rs` per-field C# gates; **f45's flags file is the merge base** (138 §5's verdict, re-verified this pass); f39 owns the force-today pair (f45 `9c1a01a42` defers it) | f45 tip `6352d3840` |

Code-only lane: witnesses written, not run.

Inputs read at the RETIRE probe (2026-11-21 — every tip below is PUSHED;
the worktree-only states the FINAL probe described have all landed or
been superseded):

| Ref | Tip read | Persist state |
| --- | --- | --- |
| `origin/task/W4-g89-aegis-fruit-use` | `598642563` (g89f over g89e `e9ec8dcfc` / g89d `4344bd50`) | `.trplayer` **v9** 3-token unlock line (research/122); `.plr` fruit/gummy/ambrosia bools both halves; `.wld` book-two byte (g89e); `.trwld` stays **v17** |
| `origin/task/W4-g88-life-fruit-use` | **`d09d7e10a` (L3-ARBITER FINAL — PUSHED, the post-seal receipts pass)** over `fc5cf11ae` (L3-DEFINITIVE seal) / `6a829a0e8` (g88h receipts-phase FINAL-FINAL, the SEAL probe) / `a9f1c29d8` (g88f fold 7, the v11 code tip) | `.trplayer` **v11 SUPERSET** (research/103 FINAL — g89c+g90b+a6+f33 folded; the torch pair is LINE 16, both v7 recognition rules landed); carries research/122 + the v11 post-receipt; its four-receipt table + the dh-DROP + the 100-byte step-10 arithmetic adjudicated §2f; its **L3-ARBITER A1-A4** (the verbatim C# write rows, the ONE A3 table, the five-dependent notify list incl. THIS doc's one-move re-pin) consumed §3-L3 rule 1's D5-A3 receipt |
| `origin/task/W4-g90-artisan-bread-use` | **`94d33f2ca`** (reconcile over `b3b82d071` / lane 7 `73367e0af`) | `.trplayer` **v7** six 1-token lines + `.plr` bools 1/3/4/5/6 + `.trwld` **v18** 2-byte tail; research/**135** pushed (`135-g90-persist-folds.md`) — its §2 rule LANDED in g88f |
| `origin/task/W4-a6-crystals-powerups` | **`d7de8bd53` (the 4-way master reconcile — PUSHED)** over `8ed96ee5d` | use arms + consumers; NO sidecar writer, NO `.trwld` claim; `.wld` book-two/satchel/bought-pets bytes — **research/138 pushed** (`138-powerup-master-reconcile.md`), its verdicts seated §3-L2/L3 |
| `origin/task/W4-f33-torch-god-favor` | `8a000407f` (torch-favor v7-line: `6f2c122e8` use arm → `7954d6ed1` docs → `a429b37c0`/`8a000407f` shimmer+desert arms) | the v7 ONE 2-token pair-line fork — **FOLDED into g88f v11** (§2d's receipt); the event/favor/949 machines (`torch_god.rs`, keep-one over d19 per research/93) stay f33's |
| `origin/task/W4-g72-bg-layers-3` | **`aa1645481` (D5-RE-PIN — PUSHED, the post-seal receipts pass)** over `b069439eb` (retire) / `4a75715ef` (final item 4: the `:11340` order-contract) / `39479b551` (the cave DRAW consumer) / `2b02345ce` (g72c persist) | `.trwld` v18 **31 B** cave block + `.wld` i32 arrays + net header; research/**64-v18-union-addendum PUSHED** (the 53-byte five-claimant proposal — a §3-L3 subset, its prefix RETIRED by the D5 re-pin: underworld [58]→[56], sites 53→107); the underworld seat it flagged FIRED (§3-L3) |
| `origin/task/W4-g71-bg-layers-2` | **`b27474a44` (BG-SEAL RECEIPT — PUSHED, the post-seal receipts pass)** over `8224c94f1` (the 92-side A3 re-pin) / `f2d8c2c67` (V18 FINAL CONSOLIDATION, the SEAL probe) / `fc76a48b3` (keep-one FINAL) / `f08e8d41e` (WIRING docs) | `.trwld` v18 3-byte surface tail + the FIRED underworld seat; research/92-doc-sweep-bg **§1-FULL** (the 13-branch claimant registry + its own 107-byte union + §2.1-FINAL roll keep-one) — the convergence receipt + offset adjudication is §3-L3 rule 1; its §1-FULL.2 now re-pinned to A3 (`8224c94f1`), its §1-SEAL.5 records THIS doc's one-move re-pin as owed — EXECUTED by the D5-A3 receipt (§3-L3 rule 1) |
| `origin/task/W4-g56-ui-void-vault` | **`54c63d8d4` (RETIRE — PUSHED, the post-seal receipts pass)** over `3ab631923` (witness/parity fixes) / `9f6d2df51` (the `.trvault` sidecar) | research/**239 FINAL** (the void-family bible): the `<world>.trvault` interim favorites carrier + its **fold-contract vs THIS doc's `.plr` bank4 rows 47/48** — COEXIST through the folds, the sidecar write retires ONLY at the two-seam gate (character-load hydration AND exit-save); consumed §2g |
| `origin/task/W4-g87-demon-heart-use` | **`dfbcaf94b` (PUSHED — the NEW claimant this probe)** | `.trplayer` **v7 `dh` sentinel line** (extraAccessory + unlockedSuperCart — §2e, a NEW self-identifying v7 shape after the 5 historical numeric ones + g90b's six-line + f33's pair-line); `.trwld` v18 **3-byte pet trio**; `.wld` pets gate 217 flips; the use arms (Demon Heart/Minecart Powerup/Pet License) |
| `origin/task/W4-c39-lantern-birthday` | **`8ecb288aa` (exec — PUSHED — the NEW claimant)** | `.trwld` v18 **5-byte moon-boss downeds** (`:1347-1351`) + `.wld` five flips — **field-identical to f39's claim, dedup §3-L3**; the lantern/party/wind machine (research/114's KEEP pieces land at the merge) |
| `origin/task/W4-f39-weather-slimerain` | **`bd5a5a24a` (Phase B — PUSHED — the NEW claimant)** | the SAME `.trwld` 5-byte downeds + `.wld` flips (dedup with c39) **PLUS the force-today pair** (`.wld` gate 212 — f45 defers it); `tr-core/season.rs` (the FixtureClock-pinned date gates) + `World.refresh_seasons` (the re-derivation carrier — no `.trwld` season bytes) |
| `origin/task/W4-g70-bg-layers-1` | `6b3049e19` (persist at `bbe47378` lineage) | `.trwld` **v18** 16-byte forest tail |
| `origin/task/W4-g69-weather-clouds-parallax` | `9717c0bb` lineage (per g71 §1) | `.trwld` **v18** 1-byte crimson tail |
| `origin/task/W4-d125-empress-r7-drain` | `d8047a10` | `.trwld` **v18** 1-byte downedEmpress tail |
| `origin/task/W4-d109-framing-genrand-a3` | `81894095` | `.trwld` **v18** dense W×H framing-variants block (port-only, LAST; `variants_from_save` arms resetFrame:false) |
| `origin/task/W4-f45-wld-header-2` (+c43 `076970809` in lineage) | `6352d3840` | `.wld` header/flags second half OWNED (13 phases — the flags merge base per 138 §5); **c43's `.trwld` v18 44-byte identity tail** (generator version/GUID/worldId/CreationTime/LastPlayed) + the `.plr` `.bak`/revision-carry/UtcNow envelope — the L1 envelope co-lane |
| `origin/task/W4-g55-ui-loadout-save` | `cde865fc2` | `.plr` loadout family + `.trplayer` v7 loadout block (folded into g88's superset) |
| `origin/task/W4-e10-player-sounds-wave` | **`432dd5ab1`** over `693e3850a` (the voice-picker + persistence docs) | **voice-write LANDED** (the char-creation picker + the draft→`.plr` threading + its own i32 transport diff) — the keep-one vs THIS branch's u8 transport is §4; the live-Player hydration hunk is CLOSED by this branch's SEAL-phase port (§4's e10c receipt) |
| this branch | `5c8bbb160` (SEAL 3, the hydration port) over `d35876af9` (RETIRE) / `63204f85` (slot-11 property fix) | `.plr` complete (research/99) + the timestamp fix + the UsingBiomeTorches property seam + **the character-load hydration LIVE** (§4's e10c receipt); `.trplayer` stays main's **v6**; `.trwld` stays **v17** |

---

## 1. The three layers (what each file IS)

| Layer | File | Format | Writer / reader | Job |
| --- | --- | --- | --- | --- |
| **L1 `.plr`** | `Players/<name>.plr` | vanilla binary, AES-128-CBC, release **326** | `crates/tr-app/src/player_file.rs` | the CHARACTER: identity, appearance, stats, inventory/banks/loadouts, every per-player unlock/counter, the full `Player.Serialize` field set (`Player.cs:54750-54933`). Vanilla-interchangeable both ways. **Canonical field map: research/99's offset table** (71 rows, 3827-byte fresh layout). |
| **L2 `.trplayer`** | `<world>.trplayer` | port-invented versioned TEXT | `crates/tr-app/src/persist.rs` | the LIVE SESSION next to a `.trwld`: position/hp/coins + the fields the live App mutates and must restore on world reload. NOT a `.plr` mirror — a lossy projection (kinds-only equips, 50-slot inventory). **Canonical contract: research/103 (v11 superset FINAL — the torch fold landed) + research/122 (line-8 addendum + the v11 post-receipt); fold-inputs: research/135 (g90, its §2 rule LANDED) + research/138 (a6, PUSHED — verdicts seated §3-L2); the f33 v7 fork: §2d's receipt; the g87 `dh` fork: §2e below.** |
| **L3 world** | `<world>.trwld` (native) / `<name>.wld` (vanilla) | port binary v17→v18 union / vanilla sections release 326 | `tr-core/src/save.rs` / `tr-core/src/wld/` | the WORLD: tiles, chests, NPCs, world-global flags. **`.trwld` union contract: THIS doc §3-L3 — TEN claimants, the 107-byte fixed head + W×H dense (g70 forest 16 + g72 cave 31 + g71 surface 3 + g69 crimson 1 + c39⨉f39 moon-boss 5 + underworld 1 + g87 pets 3 + d125 empress 1 + g90 books 2 + c43/f45 identity 44 — the FINAL-SEAL order, mutual receipt with 92 §1-FULL).** The per-claimant docs (92 §1 / 64-add / 135 §4 / 138 §5) are subsets. `.wld` flags codec: `wld/sections/flags.rs` (the C# gates; f45's file is the merge base, f39 owns the force-today pair). |

Side satellites (not layers): `<stem>.trfav` (the character star —
vanilla's `favorites.json` store, `IO/FavoritesFile.cs:27-36`; the `.plr`
metadata bit is dead in vanilla too), `<world>.trvault` (g56's interim
bank4-favorites carrier — the fold-contract §2g, retires at the
exit-save gate, never an L2 line), `<world>.map` (explored map),
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
research pairs, creative powers, temp slots, super-cart's ENABLED bit
(row 64 bit 1 — the UNLOCK bit 0 moved to §2b at the retire probe, g87's
`dh` line), refunds, dialogues, hbLocked, the save timestamp, the
metadata block. Also **L1-only among the unlock family**:
`downedDD2EventAnyDifficulty`
(`:54787` — 103 §2a token 7 stays RESERVED; if a DD2 lane ever needs it
live, the token is there, but no pushed lane claims it), taxMoney, the
death counters.

**`extraAccessory` MOVED out of this list at the RETIRE probe**: g87
`dfbcaf94b` gives it (and `unlockedSuperCart`) a live sidecar carrier
(the v7 `dh` sentinel line, §2e) — they are §2b contested fields now.
The FINAL probe's "research/122 §4.4 already pins no sidecar line" note
is superseded: 122 §4.4 predates g87's push; the dual-carrier rule
(§2b) applies — the Demon Heart use-grant (`Player.cs:45197`) mutates
the live Player mid-world, so the sidecar is the only carrier that
survives a world reload until the exit-save lands.

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
| `usedAegisCrystal` | row 23 (`[0]`) | line 8 token 4 **LIVE at v10** | g90 lane 7 / a6 |
| `usedArcaneCrystal` | row 25 (`[2]`) | line 8 token 5 **LIVE at v10** | g90 lane 7 / a6 |
| `usedGalaxyPearl` | row 26 (`[3]`) | line 8 token 6 **LIVE at v10** | g90 lane 7 / a6 |
| `ateArtisanBread` | row 21 | line 11 (g90, folded g88) | g90 |
| `unlockedBiomeTorches` | row 19 (`:54777`, gate 229) | f33 v7 pair-line → **LANDED as line 16 at g88f v11** (§2d receipt) | f33 |
| `UsingBiomeTorches` (the slot-11 getter) | row 20 (`:54778`) + row 55 slot 11 | same line 16 token 2 | f33 (transport); the slot-11 array stays L1-only |
| `extraAccessory` | row 18 (`:54776`, gate 125) | g87 v7 `dh` line token 1 → fold per §2e | g87 (transport); was L1-only, §2a's note updated |
| `unlockedSuperCart` | row 64 (the `:54914-54918` BitsByte bit 0, gate 253) | g87 v7 `dh` line token 2 → fold per §2e | g87 (transport); bit 1 `enabledSuperCart` stays L1-only |
| `CurrentLoadoutIndex` + `Loadouts[3]` | rows 65-66 | lines 12-15 (g55, folded g88) | g55 |
| bank4 favorites (the fav bool per slot) | row 47 (the fav byte @ 2001+10·i+9, gate 255) | **`<world>.trvault`** — g56's SEPARATE 40-char sidecar, NOT a `.trplayer` line (239 §1; the superset never grows a vault line) | g56 interim carrier; the `.plr` byte-shape THIS lane — the retire gate §2g |
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
3. **Within L2, one token per field.** 103 §2a's token map is the
   single registry; g90's six-line v7 block folded into the v7
   recognition rule (135 §2 — LANDED in g88f). The token-order question
   for 4/5/6 is CLOSED: the pushed g88 v10 writer emits
   **crystal=4/arcane=5/pearl=6** (the vanilla `.plr` relative order,
   `Player.cs:54781/:54783/:54784`) — 103 §8's alignment note is the
   receipt; 135 §3's pearl-first recommendation and this doc's FINAL
   rule-3 parenthetical are superseded (138 §6 re-verified the written
   order is binding). No renumbering ever.
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
  byte carrier. The live `Player` on THIS branch now carries ALL SIX
  (the SEAL-phase hydration port declared the five siblings beside
  `used_gummy_worm` (i35) + `ate_artisan_bread` (reach formula) — the
  g90b declaration spots, so the g-lanes' consumers fold textually).
- **g89/g90 sidecar bools** carry the LIVE session's unlock state
  (the use arms set `Player.used_*`, the booster rows read them every
  tick) across world reloads, because the `.plr` is not re-saved
  in-session. Same field, same meaning — a second carrier for a
  different save cadence.
- **The merge boundary (LIVE since the SEAL-phase port):** the live
  `Player.used_*` fields are the single live truth; the `.plr` G7
  carrier restores them at CHARACTER load (`CharPlay` parks the
  card's path, `App::new` calls `Player::hydrate_from_file` — §4's
  e10c receipt); the sidecar restores them at world load (the
  `persist::load` block, which runs AFTER the hydration in the same
  `App::new`). The character load runs FIRST, the sidecar restore
  second — the sidecar wins for a world that was saved after the
  character file, which is the correct precedence (it is the newer
  state). A `.plr`-only field (extraAccessory etc.) has no sidecar
  line under the §2f default-DROP, so no precedence question arises.

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

**THE RECEIPT (retire probe — g88f `a9f1c29d8`, PUSHED): §2d CONSUMED.**
The fold landed this section's proposal with ONE deliberate deviation:

- **The recognition rule landed verbatim**: 103 §3's v7 arm now runs
  "exactly ONE non-empty line, a 2-token pair with BOTH tokens 0/1 →
  the biome-torch pair" (the value-range argument reproduced: the
  permanent halves start 100/20, no maxima pair can be all-0/1).
  Witness `f33s_torch_only_v7_loads_positionally`. 135 §2's six-1-token
  rule landed in the same arm (`g90bs_six_line_v7_loads_positionally`,
  the bounded-misread witness RETIRED).
- **The union seat landed at line 16** (the LAST post-dye line, after
  the loadout block) — §2d rule 2's seat exactly. **The version bumped
  to v11** instead of §2d's "writer stays v10" append-only proposal:
  g88 chose the explicit bump (103 §2b — v11 was free; the v10 files
  back-read the pair false, witness
  `the_v10_superset_still_loads_without_the_torch_pair`). The bump is
  the safer shape (a v10 reader meeting a line-16 file tolerates it via
  the shape walk's "any other shape → defaults kept", but the explicit
  version makes the back-read contract testable) — **v11 is the merge
  base, §2d's stay-v10 proposal is superseded**.
- **The restore landed via the setter**: `app/mod.rs` writes
  `Player.unlocked_biome_torches` + `set_using_biome_torches` (f33's
  exact accessor spot after `luck()`), no second bool field — §2d
  rule 3 / §2a's slot-11 semantics exactly.
- **The conditional DROP-recommendation published** (103 §2b rule 3):
  the line stays a deliberate dual-carrier until (a) the character-load
  hydration AND (b) the exit-save both land; then the writer drops
  line 16 and the reader keeps the arm as a back-read. Restore
  precedence per §2c (character first, sidecar second, sidecar wins).
- **The f44c-G7 redundancy adjudicated**: 103 §2b rule 4 pins f33's
  named `.plr` torch fields as a clean textual dedup onto THIS branch's
  identical named fields (the FINAL slot-11 property model is the
  superset — f33's raw-bool witness re-pins per §2d rule 4).

### 2e. The g87 `dh` sentinel line (a NEW v7 shape — retire-probe input; the CONDITIONAL fold — see §2f's default-DROP adjudication)

`origin/task/W4-g87-demon-heart-use` tip `dfbcaf94b` (PUSHED, read at
this probe — the claimant the FINAL probe missed). Its `persist.rs`
diff against main:

- Writer bumps **v6 → v7** and appends ONE line after the dye line:
  **`dh <extraAccessory> <unlockedSuperCart>`** — a SELF-IDENTIFYING
  sentinel prefix + two 0/1 tokens (`Player.cs:866`/`:2052`, the
  use-grants `:45197`/`:45209`). The sentinel follows 103 §6 rule 5's
  contract (a shape distinct from the numeric pair/flag/loadout lines).
- Reader: `"7"` joins the no-migrate arm; armor/kills/dye gates widened;
  the `dh` line is OPTIONAL at v7 (a missing/non-`dh` line keeps both
  false — token-tolerant).
- `SavedPlayer` gains `extra_accessory` + `unlocked_super_cart`; the
  `.plr` halves are explicitly deferred to THIS lane (research/99
  rows 18 + 64 — both already ride the complete walk).

**The collision:** under the g88 superset's v7 walk the `dh` line is a
3-token first post-dye line — the shape walk's "any other shape →
tolerated, defaults kept" arm would DROP both unlocks silently (not a
corruption, but a loss). Under g87's own reader a g88/g89/g90 v7 file's
first line is not `dh` and defaults stand — aligned but lossy. Neither
reader is the merge winner; the fold:

1. **Recognition rule (extends the v7 arm's):** a post-dye line whose
   FIRST token is the literal `dh` → the g87 unlock pair
   (`parts[1]`/`parts[2]` parse as 0/1). The sentinel is unambiguous —
   no numeric line can start with `dh`. Runs BEFORE the shape walk,
   beside the six-line and torch-pair rules.
2. **Union seat:** append as a NEW 3-token sentinel line AFTER the
   torch pair — **line 17, `dh <extra> <cart>`**. The version
   adjudication: 103 §6 rule 4 reserved v12 for the DD2 token-7
   activation, but that reservation is a TOKEN slot on the unlock
   line (§2a), not a hard version claim — and no pushed writer ever
   emitted v12. The `dh` line takes the next free version, **v12**;
   the DD2 version reservation MOVES to **v13** (a future token-7
   activation bumps v13 with the 7-token back-read). Zero back-compat
   cost (the same no-pushed-writer argument 103 §8 used for the
   crystal tokens). An append-only stay-v11 alternative exists (the
   shape walk tolerates the extra line) but the explicit bump is the
   safer shape — the same choice g88f made for the torch line (§2d's
   receipt). The v11 reader back-reads a missing line 17 as
   both-false.
3. **Restore:** `p.extra_accessory = line17[1];
   p.unlocked_super_cart = line17[2]` — plain bool fields (the C# has
   no property indirection here; `enabledSuperCart` is NOT carried —
   it is a session toggle, L1-only row 64 bit 1, and the sidecar must
   not grow a token for it).
4. **Witnesses:** g87's two (`w4_g87_the_demon_heart_and_super_cart_unlocks_round_trip`,
   `w4_g87_a_v6_sidecar_loads_the_unlocks_false`) re-pin onto the
   superset writer (the `7\n` header assertion becomes `12\n`; the
   `dh 1 1` line assertion stays verbatim — the sentinel shape is
   preserved at the new seat).
5. **The `.wld`/`.trwld` halves:** g87's pet-trio flips dedup onto
   f45's flags file (§3-L3 rule 3); its `.trwld` 3-byte claim seats in
   the 107-byte union (§3-L3 rule 1). The use arms (Demon Heart /
   Minecart Powerup / Pet License) are g87's — 138 §1 adjudicates the
   Pet License arm to a6 (`use_pet_license`, the sole full-fidelity
   claimant of the exchange legs); g87's simpler grant arm dedups onto
   it at the merge, the persist carriers are unaffected.

### 2f. The g88h seal receipt (the L2-definitive re-read, SEAL probe)

`origin/task/W4-g88-life-fruit-use` latest tip **`6a829a0e8`** (the
receipts-phase FINAL-FINAL, PUSHED over the v11 code tip `a9f1c29d8`)
read FULL — research/103's §1a f44-cross-receipt, §2a's DD2 census,
§5a's g87-triple-flag adjudication, §9's receipts-complete checklist.
The verdicts, item by item against THIS doc:

1. **The 107-receipt — g88h does NOT carry it (one pass stale, now
   closed by this seal).** Its §9 step 10 publishes a reconciled
   **100-byte** head ("132's 97 + g87's 3 pets at 52-54, books move
   55/56, identity 57..100") — computed against our FINAL `bdd1cc79a`
   (the arithmetic-slip era), before the RETIRE `d35876af9` corrected
   97→98 and seated the moon-boss 5 + underworld 1. Its own text
   flags the arithmetic as "the L3 merger's to verify against the
   final tips — L2 is independent of it". **This doc's §3-L3 rule 1
   (the 107-byte FINAL-SEAL table, mutual-received with 92 §1-FULL)
   supersedes g88h's step-10 100-byte arithmetic**; its pets-before-
   books rationale (`:1392-1394` < `:1407-1408`) is CONSUMED and
   holds in the FINAL-SEAL order (pets [57..60] < books [61..63]).
   Its step-11 `.wld` dedups (f45 base, pets keep-f45, d124's
   independent tier trio) match §3-L3 rule 3 verbatim.
2. **The `dh` line-17 question — g88h reads it as DROP, ours folds
   it; adjudicated: the DROP wins as the DEFAULT, our §2e fold stays
   the CONDITIONAL seat.** g88h §5a/§9 step 9 (consuming 138-FINAL
   §10.4): the `dh` sentinel line DROPS at merge — the v11 reader
   tolerates a g87 file lossy-not-corrupt (the 3-token `dh` line hits
   the `_ => {}` arm, `persist.rs:565` on its tip — verified in its
   census), both flags are L1 fields (`extraAccessory` row 18, the
   super-cart BitsByte row 64) that ride THIS branch's complete
   walk, so no L2 carrier is owed. **The SEAL adjudication:** the
   character-load hydration is THIS seal phase's code obligation
   (§4's hunk — the port lands in this phase's code commit), making
   the L1 carrier live on load — the
   `dh` line's only remaining value is the in-session dual-carrier
   BEFORE an exit-save exists (the same condition the torch line's
   drop rides, 103 §2b rule 3). So: **the merger DEFAULT is g88h's
   DROP** (no v12 bump, no line 17, the v11 writer stands; DD2 keeps
   its v12 token-7 reservation per g88h §2a — our §2e's v13 re-seat
   is VOID under the default). §2e's recognition rule + line-17 seat
   stay published as the CONDITIONAL alternative: they activate only
   if the merger chooses to carry the two unlocks in-session before
   the exit-save lands, in which case §2e's v12-line-17/DD2→v13
   arithmetic applies verbatim. The back-compat matrix's two `dh`
   rows (§5) hold under BOTH paths (tolerated-loss without the rule,
   lossless with it).
3. **The L2 base + token plan — AGREED, no delta.** g88h §9 step 6
   confirms writer v11 + the tolerant v7-v11 reader as the base (our
   §3-L2's "base = g88 `6998a4ecb` v9" staleness note matches its own
   correction); token 7 UNCLAIMED (d124 owns the WORLD tier latches
   `.wld` 1931-1933 gate 178, NO `.plr`/`.trplayer`/`save.rs` diff —
   its census re-verified `git diff --stat` empty); the C#
   `downedDD2EventAnyDifficulty` is a derived per-tick mirror
   (`Player.cs:24450-24453`) whose only consumer is the turret-use
   refusal `:52083` — the L1 byte stays file-verbatim transport
   (row 25), no live-field obligation. The use-arm keep-ones (§9
   step 8: g88 keeps 29/109/1291, a6's `use_shimmer_permanent`
   machine for the six, ONE `use_combat_book`, 5326 g90, 5043 a6 +
   f33 machine + c36 helper, g87's three arms DROP) match our
   §3-L2 rule 7's table — consumed verbatim.
4. **The conditional torch-line drop — condition (a) is this phase's
   port.** 103 §2b rule 3 gates the drop on
   (a) character-load hydration AND (b) the exit-save. The seal-phase
   port lands (a) (§4's hunk, this phase's code commit); (b) — a
   `.plr` re-save at world exit — remains
   open (the port's save funnel writes the sidecar only). The line
   stays until (b); the restore precedence (character first, sidecar
   second, sidecar wins) becomes the LIVE order in `App::new` (§4).

### 2g. The g56 `.trvault` fold-contract receipt (the post-seal receipts pass — the two-seam gate COMPOSED)

`origin/task/W4-g56-ui-void-vault` tip **`54c63d8d4`** (RETIRE, PUSHED
over `3ab631923`) read FULL — research/239 FINAL (the void-family
bible), its §1 fold-contract + §3 zero-touch compose + §5 family map.
The contract, consumed:

1. **The three carriers.** The `.plr` bank4 quadruples (row 47 —
   `Player.cs:54856-54862`, read gate 198 `:55488`, the fav bool gate
   255 `:55495-55498` at offset **2001 + 10·i + 9**) + the
   `voidVaultInfo` BitsByte (row 48 — `:54863`, gate 199
   `:55501-55503`, bit 0 = `IsVoidVaultEnabled`, `Player.cs:3900-3910`)
   are THIS lane's byte-shape (`PlayerFile.bank4`, the aired-slot
   policy keeps stack+fav). g88's v11 superset carries **NO vault
   line** (239 §1a's writer-arm census). g56's `<world>.trvault`
   (40-char 0/1, `persist.rs::save_vault_favorites`/
   `load_vault_favorites`, wired saveglue save + `App::new` load) is
   the interim LIVE carrier — per-world, not per-character (the
   port's session model).
2. **Keep-one TODAY = the sidecar; COEXIST through the folds.** This
   lane's bank4 model is a file-shape claim, not a live carrier: the
   only `.plr` write seams are the creation save (fresh-ctor bank4)
   and the fixtures — no live-session re-save exists (§1's routing
   rule). The SEAL-phase hydration (§4) applies stats/unlocks/voice/
   counters only; **the inventory/equip/bank payloads stay the
   App-level restore's scope** (`player.rs:2513-2515`'s own scope
   note). So a mid-session vault star STILL has only `.trvault` to
   ride — retiring the sidecar at the `.plr` fold would silently drop
   it (the same corruption class §2b's conditional-drop rule
   prevents). f44 owns the byte-shape, g56 owns the interim carrier;
   no dup-write today (different files, §2b rule 1 holds).
3. **The retire gate = THIS doc's exit-save remainder, COMPOSED.**
   239 §1c gates the `.trvault` write-retire on (a) character-load
   hydration applying bank4 + favorites AND (b) the exit-save writing
   the LIVE bank4 quadruples + `voidVaultInfo`. Condition (a)'s
   PLAYER-state half is MET by the SEAL port (`5c8bbb160`); its
   bank4-payload half rides the App-level restore extension that the
   exit-save lane lands WITH the payload writer — so **both contracts
   (239 §1c's vault gate + 103 §2b rule 3's torch-line gate + §2f's
   dh conditional) wait on the SAME one remaining seam: the
   exit-save** (§4's remainder, §8 step 4 item 3). When it lands: the
   merger drops the `.trvault` WRITE (the saveglue call + optionally
   the writer fn), KEEPS `load_vault_favorites` as a one-release
   back-read fallback (the `.trplayer` back-compat precedent),
   restore order `.plr` first / `.trvault` second for pre-fold files
   only (post-fold the sidecar is never written — the fallback is
   naturally inert). **NO g88 v12 vault line ever** — a new L2 token
   would re-create the dual-carrier the contract retires (the
   favorites are `.plr`-resident in vanilla).
4. **The `voidVaultInfo` bit-0 nuance (row 48).** The C# RE-DERIVES
   `IsVoidVaultEnabled` per tick from `HasItem(4131)`
   (`Player.cs:24550-24553`); the saved bit is a one-tick-stale
   mirror in vanilla too. The exit-save writes it for byte parity;
   the hydration must NEVER apply it against the per-tick refresh
   (the refresh wins, C#-identical). This lane's model carries the
   byte untouched; g56 keeps no runtime field for the BitsByte.
5. **The merger conflict notes (239 §1d/§3, consumed verbatim).**
   `persist.rs`: g56's `vault_favorites_path` + the two fns insert
   between `player_path` and `map_path` (`@@ -33`), g88's
   `loadout_item` at the SAME seam — both pure insertions, keep
   both; g56 touches NEITHER `save`/`load` body (its only other hunk
   is `@@ -274` tests). `saveglue.rs`: g88's widened `persist::save`
   call + g56's appended `.trvault` write after the error arm —
   verbatim both. `app/mod.rs::new`: g56's restore sits AFTER g88's
   superset block (`loaded_kills = saved.kills` seam) — pure
   insertions; with THIS lane's hydration FIRST (§2c/§4) the merged
   order is **hydration → g88 superset restore → g56 `.trvault`
   read**, already the contract's sidecar-second precedence. The
   e19 auto-pickup disjointness + the 16-box repin + the glow
   keep-one (239 §2/§4) are g56-lane verdicts — no `.plr`/`.trwld`
   byte involvement, nothing owed here.

## 3. Per-layer merge order (the FINAL merger persist-checklist)

**Order: L1 → L2 → L3.** L1 first because the `.plr` is the vanilla
interchange and the field-superset every other layer projects from; L2
second because its restore block reads the live `Player` fields L1's
fold keeps; L3 last because the world union is independent of both
(no player field rides it) and its arithmetic depends on the g-lane
tips being final.

### L1 `.plr` (player_file.rs — hot file; the g-lanes + e10 + c43/f45 touch it)

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
5. **Fold c43's envelope** (`076970809`, in f45's lineage — PUSHED,
   re-read at this probe): the `.bak`
   copy before a re-save (`InternalSavePlayerFile:54730-54733`), the
   metadata revision CARRY (`PlayerFileWrite::revision` — this branch's
   writer still hardcodes 0 at the prologue; the carry is the one
   remaining L1 envelope gap, byte-neutral for created characters),
   its UtcNow creation stamp + `with_mtime_fallback` dedup onto this
   branch's `utc_now_binary()` + `read_player_file` fallback (same
   semantics, same C# citations — keep one helper each; c43's
   `read_player_bytes_at` split is a compatible refactor shape, take
   whichever survives the textual merge, the semantics are identical).
   **g87's `.plr` half needs NO fold**: its use-grant fields
   (`extra_accessory` row 18, the super-cart BitsByte row 64) already
   ride this branch's complete walk — g87's diff explicitly defers the
   `.plr` to this lane (its persist.rs comments cite f44).
6. **Keep the timestamp fix whole** (`read_player_file`'s mtime
   fallback + `utc_now_binary`); g89/g90/f33's AND e10's
   `last_saved_binary: 0` literals in their session/test diffs dedup
   onto this branch's `utc_now_binary()` (session) / stay 0 (fixtures).
   e10's session hunk still carries the pre-fix "the port tracks no
   save clock" comment — this branch's session.rs is the newer shape.
7. **Fold e10's voice-write, keep THIS branch's transport** (§4's
   keep-one): e10's `player_file.rs` diff (the i32 `PlayerFile`/
   `PlayerFileWrite` fields + the `finish_player_file` voice parameter)
   DEDUPS onto this branch's u8 fields + `PlayerTail` walk — same
   slots, same gates, same clamp, same Male-derive; this branch's
   model additionally covers the inc1/inc2 depths through the shared
   tail. KEEP e10's char-creation picker (`UICharacterCreation.cs:
   452-528`), its `NewPlayerDraft.voice_variant/voice_pitch_offset`
   fields and its `session.rs` threading hunk — re-pointed at this
   branch's u8 writer fields (`voice_variant: draft.voice_variant as
   u8`). e10's three voice witnesses re-pin onto the u8 assertions
   (its `w4_e10_the_voice_pair_round_trips_at_the_csharp_offsets` ==
   this branch's `the_voice_pair_round_trips_clamps_and_derives` first
   arm — keep one; the clamp + gender-derive arms are already covered
   here). The live-Player hydration hunk is CLOSED on this branch
   (§4's e10c receipt — the SEAL-phase port: `CharPlay` parks the
   card's `.plr`, `App::new` hydrates the live Player; e10's fold
   keeps its consumers and drops its own hydration note).
8. Anchors: this lane's rows stay; the g-lanes' `.plr` witnesses
   re-pin onto the array fields (their assertions are slot-value
   assertions — mechanically portable).

### L2 `.trplayer` (persist.rs — the g88 superset wins)

Per research/103 §9 (the FINAL v11 checklist) + research/122 §4 +
research/135 §6 + §2d/§2e above, in order:

1. **Base = g88f's pushed tip `a9f1c29d8`** (writer **v11**, tokens 1-6
   LIVE, the torch pair at line 16, the tolerant v7-v11 reader with
   BOTH recognition rules landed, the loadout block, research/103+122
   in-tree). The FINAL probe's "worktree v10, re-apply if unpushed"
   contingency is RETIRED — the fold is pushed; take the file whole.
2. **The v7 legacy arm is DONE** (135 §2's six-1-token-line rule + §2d's
   one-line-2-token-0/1 rule both landed in g88f — 103 §3/§8's receipt).
   The `dh`-sentinel recognition is CONDITIONAL (§2f's adjudication:
   the default is g88h's DROP; the rule lands only with the line).
3. **Tokens 4/5/6 are LIVE at v10+** in the C#-verified order
   crystal=4/arcane=5/pearl=6 (103 §2a's FINAL map + §8's alignment
   note; 138 §6 re-verified the written order is binding). Short-line
   back-reads for v7/v8/v9/v10 kept. Token 7 (DD2) stays reserved.
4. **The torch line 16 is LANDED** (§2d's receipt — the setter restore,
   the conditional drop-recommendation, the v10 back-read witness).
   Nothing left to do except the §2b dual-carrier revisit when the two
   §2c seams land.
5. **The g87 `dh` line — DEFAULT DROP (§2f's g88h receipt).** The v11
   writer stands; the two unlocks ride L1 (rows 18/64) + the
   character-load hydration. ONLY if the merger needs the in-session
   carriers before the exit-save lands: §2e's conditional fold
   (line 17, writer bumps v12, DD2's version reservation moves to
   v13; the sentinel recognition in the v7 arm; the v11 back-read
   defaults both false).
6. **Dedup** g90's `SavedPlayer` flag fields + App restores onto the
   superset's (all folded at v10/v11); f33's pair fields join the same
   restore block (landed); g87's two fields need NO L2 seat under the
   default (the hydration restores them from L1).
7. **Seat research/138's adjudications — PUSHED now (`d7de8bd53`), the
   FINAL probe's "when it pushes" contingency is RETIRED.** 138 §1's
   per-item verdicts (the C# re-read adjudications, a6's five fidelity
   fixes landed):
   - 29/109/1291 (Life/Mana Crystal, Life Fruit): **g88 keep-own**
     (`crystals.rs`/`life_fruit.rs`); a6's three dups + c36's
     `use_permanent_powerup` DROP; the shared row-targeted
     `complete_condition` helper survives (c36's shape — NOT the
     `notify_custom("Use")` broadcast).
   - 5326 (Artisan Loaf): **g90 keep-own** (the skeleton-merchant shop
     consumer `Chest.cs:2958` is its exclusive); a6's dup DROP.
   - 4382/5336 (Combat Books): keep ONE — 138 recommends **a6**'s
     `use_combat_book` (the single cluster seam) or g89e (pushed
     precedent); the TRAINING_DAY consumer lands once.
   - 5043 (Torch God's Favor): **a6's use arm** (the faithful setter —
     BOTH `unlocked_biome_torches = true` AND
     `builder_acc_status[11] = 0`, superseding c36 research/30's
     keep-c36 note); **c36**'s `complete_condition` dispatch; **f33**'s
     event machine + flag reads. Three-way split, no overlap to delete.
   - 5343 (Satchel): keep ONE — a6 or g90; the shop +1 consumer
     identical on both.
   - 4829/4830/4910 (Pet Licenses): **a6** (`use_pet_license`, the sole
     claimant of the exchange legs); g87's grant arm dedups (§2e).
   - 5337-5342 (six shimmer permanents): **a6's
     `use_shimmer_permanent`** is the single use-arm owner; g89's three
     + g90's `use_shimmer_permanent_item` dedup onto it; **g90's Player
     field declarations + consumer shapes are the merge anchors** (the
     `update_luck` pearl mirror, the aegis-crystal f32 slot BETWEEN
     base +1 and crimson +1, the booster row order fruit→gummy→
     ambrosia — a6 now matches all three).
   These are use-arm/consumer dedups, NOT persist-layout changes — the
   sidecar contract is unaffected either way (103 §5a's keep-own for
   the CARRIERS stands; 138 adjudicates the ARMS).
8. This lane contributes NOTHING to L2 (main's v6 reader/writer is a
   strict prefix of the superset; the merge takes g88's file whole).

### L3 world (save.rs `.trwld` + wld/ `.wld`)

1. **`.trwld` base = THIS section's union — TEN claimants, 107-byte fixed
   head + W×H dense.** This is THE authoritative layout: research/92
   §1-FULL (g71's 13-branch registry + its own 107-byte union, pushed
   `f2d8c2c67` — the claimant registry of record, its offsets re-seated
   by the convergence receipt below), research/64-add
   (g72's 53-byte five-claimant proposal), 135 §4 (g90's 21), 138 §5
   (f45's 65) and this doc's own FINAL "97" are all STRICT SUBSETS or
   stale totals/orders — the ten claimants below seat them all.

   **The arithmetic correction (the FINAL doc's slip):** the FINAL phase
   published "Fixed head = 97 bytes: 16+31+3+1+1+2+44" — that sum is
   **98**, not 97. The seven-claimant head was always 98 bytes. The retire
   probe adds three more claims and one conditional seat that FIRED:

   - **g87's 3-byte pet trio** (`dfbcaf94b`, PUSHED): `boughtCat`/
     `boughtDog`/`boughtBunny` u8×3 (`WorldFile.cs:1392-1394`, the
     4829/4830/4910 Pet License use-unlocks). 132's FINAL probe knew the
     pets only as `.wld` flags (a6/f45 Ph11); g87 gives them a `.trwld`
     claim — the same dual-carrier pattern the flags half has.
   - **c39 ⨉ f39's 5-byte moon-boss downeds — ONE block, TWO claimants**
     (`8ecb288aa` c39 exec / `bd5a5a24a` f39 Phase B, both PUSHED): BOTH
     branches append the SAME five bytes in the SAME vanilla order
     (`downedHalloweenKing downedHalloweenTree downedChristmasIceQueen
     downedChristmasSantank downedChristmasTree`, `WorldFile.cs:
     1347-1351`; the `NPC.cs:79210-79238` DoDeathEvents arms). The
     save.rs/flags.rs hunks are field-identical — **dedup to ONE block**
     (five bytes, not ten). Keep-one for the DEATH-ARM side: f39's
     `DoDeathEvents` port is the season-lane owner (its Phase B wires
     every season consumer through the resolved `World.halloween/xmas`);
     c39's exec arms dedup onto it (research/114 §0.2's record stays the
     landing history). f39 ALSO owns the `.wld` force-today pair
     (`forceHalloweenForToday`/`forceXMasForToday`, `:1386-1387`, gate
     212 — f45 `9c1a01a42` explicitly defers those bytes to f39). The
     pair is `.wld`-only: the `.trwld` needs no force bytes because f39's
     `World.refresh_seasons` re-derives the resolved seasons at dawn/load
     (`Main.cs:65389-65391`, `WorldFile.cs:651-652`) — the runtime
     re-derivation is the carrier, witnessed by f39's
     `the season flags re-derive, not persist` arm.
   - **The underworld seat FIRED** (the FINAL doc's OPEN conditional):
     g71's pushed wiring rolls `RandomizeBackgrounds`' underworld
     `Next(3)` (`bg_rolls.rs` → `set_surface_bg_style(9, …)`) AND its
     flags.rs now writes/reads `world.underworld_bg_style`
     (`WorldFile.cs:1376`). A rolled-and-`.wld`-persisted style with NO
     `.trwld` byte would be lost on a native world reload — exactly the
     loss class the union exists to prevent. Per 64-add §3's flagged
     seat: **1 byte in the flags group at its C# position — after the
     moon-boss downeds, before the pets (`:1376` between `:1351` and
     `:1392`; the RETIRE phase's surface-adjacent seat is corrected by
     the convergence receipt below)**.

   **THE CONVERGENCE RECEIPT (the SEAL probe, mutual 132↔92):** g71's
   V18-FINAL-CONSOLIDATION `f2d8c2c67` (PUSHED — research/92-doc-sweep-bg
   §1-FULL, the 13-branch claimant registry) independently derived a
   **SAME-SIZED 107-byte head** from a full-tip sweep (13 branches /
   11 distinct blocks / 3 duplicate pairs). The seal probe byte-order-
   compared the two independently-derived layouts field by field. The
   claimant SETS are IDENTICAL (ten claimants after dedup; the same
   three dedup pairs: e19=g69, f39=c39, f45=c43). The block INTERNALS
   are byte-identical (forest tree_x 3×i32 LE + styles 4×u8; cave
   cave_back_x[..3] 3×i32 + cave_back_style 4×i32 + ice/jungle/hell
   3×u8, caveBackX[3] NEVER persisted `:1260-1262`; identity u64 + 16B
   GUID + i32 + 2×i64 = 44). The OFFSETS DIFFER in the flag-singles
   group — adjudicated below, **C#-write-order definitive**:

   | Block | 132 RETIRE (ours) | 92 §1-FULL (theirs) | Δ |
   | --- | --- | --- | --- |
   | forest / cave / surface | [0..16] / [16..47] / [47..50] | same | 0 |
   | crimson | [51] | [50] | −1 |
   | books ×2 | [61..63] | [51..53] | −10 |
   | moon-boss ×5 | [52..57] | [53..58] | +1 |
   | underworld | [50] | [58] | +8 |
   | pets ×3 | [57..60] | [59..62] | +2 |
   | empress | [60] | [62] | +2 |
   | identity 44 / dense | [63..107] / [107..] | same | 0 |

   (The Δ table is the HISTORICAL RETIRE-vs-92 audit trail — both
   columns superseded by the FINAL-SEAL layout below, which the D5-A3
   re-pin receipt confirms as 103 L3-ARBITER A3 verbatim.)

   **The adjudication** (`WorldFile.cs:1240-1414` re-read at the seal
   probe): the C# write order is crimson `:1281` < moon-boss
   `:1347-1351` < underworldBG `:1376` < pets `:1392-1394` < empress
   `:1395` < books `:1407-1408`. Their layout keeps g72's published
   64a 53-byte prefix verbatim (books at [51..53], underworld at [58])
   — a *published-offset-stability* rule for a proposal NO released
   file ever carried; ours claimed strict C# order but placed
   underworld BEFORE crimson (the RETIRE phase's "between the surface
   group and crimson" seat — itself a 64a §3 carry-over, not C#
   order). Under the definitive rule each layout is wrong in one
   group: theirs moves books ahead of moon-boss/underworld/pets/
   empress; ours moves underworld ahead of crimson/moon-boss. **The
   reconciled FINAL-SEAL layout below keeps the strict-C# rule and
   fixes the underworld seat** (their [50] crimson and our moon-boss-
   before-pets/empress/books survive; books ride the tail of the
   flags group where the C# writes them). Every move is free under
   the forward-only rule: no released v18 exists, all claimants are
   unmerged task work, and 92 §1-FULL.3's own back-compat matrix
   already declares every single-branch v18 mis-parsing — the 64a
   offsets carry no back-compat obligation.

   The reconciled FINAL-SEAL layout (one fixed shared order — 92 §1's
   forest-first rationale extended through 64-add §2's C# field order,
   the flag singles in strict vanilla `.wld` write order, identity
   closing the head, dense LAST):

   ```
   … v17 downedFishron byte (1)
   v18 UNION tail — fixed head (107 bytes) + dense variants (W×H):
     [0..16]    g70 forest block        (tree_x 3×i32 + styles 4×u8)
     [16..47]   g72 cave block          (cave_back_x 3×i32 + cave_back_style
                                         4×i32 + ice/jungle/hell u8)  — 64-add §1
     [47..50]   g71 surface bytes       (snow/desert/ocean u8)
     [50]       g69 crimson             u8   (:1281 — 92 §1-FULL's seat)
     [51..56]   c39⨉f39 moon-boss downeds u8×5 (king/tree/iceQueen/
                                         santank/xmasTree — :1347-1351,
                                         ONE deduped block)
     [56]       g71 underworld_bg_style u8   — the FIRED seat (:1376,
                                         AFTER moon-boss per C# order)
     [57..60]   g87 bought-pet trio     u8×3 (cat/dog/bunny — :1392-1394)
     [60]       d125 downedEmpress      u8   (:1395)
     [61..63]   g90 book-two + satchel  u8×2 (:1407-1408)  — 135 §4
     [63..107]  c43/f45 identity block  (u64 generator version + 16B GUID
                                         + i32 worldId + 2×i64 ToBinary
                                         stamps = 44 B; :1225-1227/:1250-1251)
     [107..107+W*H]  d109 dense variants (LAST, self-delimiting,
                                         variants_from_save arms the
                                         framing passes' resetFrame:false)
   ```

   **Fixed head = 107 bytes: 16+31+3+1+5+1+3+1+2+44. Total = 107 + W×H.**
   (The flag singles ride strict C# `.wld` write order — crimson
   `:1281` < moon-boss `:1347-1351` < underworld `:1376` < pets
   `:1392-1394` < empress `:1395` < books `:1407-1408`. The identity
   block is not flags-family — it rides `SaveWorldHeader:1225-1227` +
   the two time fields `:1250-1251` ahead of the whole flags stream —
   so as a self-contained header group it closes the head (both
   derivations agree); the dense block stays LAST per 92 §1.)
   VERSION stays 18; forward-only (no released v18 — every claimant is
   unmerged task work). `v18_tail_len` returns `107 + width*height`.
   **The mutual receipt:** 92 §1-FULL's registry (sizes, C# anchors,
   dedup pairs, pre-v18 defaults, the truncation-move table §1-FULL.4,
   the back-compat matrix §1-FULL.3) is CONSUMED VERBATIM — only its
   §1-FULL.2 offsets [50..63] re-seat to the table above; THIS table
   is the layout of record, 92 §1-FULL the claimant registry of
   record. Both docs describe ONE union.

   **THE D5-A3 RE-PIN RECEIPT (the post-seal receipts pass — 103 A4's
   132-row EXECUTED).** g88 `d09d7e10a` (L3-ARBITER FINAL, PUSHED over
   its L3-DEFINITIVE seal `fc5cf11ae`) read FULL: A1 quotes every
   disputed `writer.Write` verbatim (`WorldFile.cs` identity
   `:1225-1227`/`:1250-1251`, crimson `:1281`, moon quintet
   `:1347-1351`, underworldBG `:1376`, pets `:1392-1394`, empress
   `:1395` AFTER pets, books `:1407-1408` the LAST two flag writes;
   the order-proof interstitials — first `combatBookWasUsed` `:1380`
   already `.wld`-owned so books stays 2 B, force-today `:1386-1387`
   `.wld`-ONLY, journey/bread carriers NONE, `Main.GameMode` `:1240`
   pre-v18); A2 overrules BOTH consolidators' non-matching middles
   (92's books-in-prefix AND our RETIRE underworld-before-crimson)
   under the C#-actual-write-order-definitive brief; **A3 restates
   the ONE table — field-for-field IDENTICAL to the FINAL-SEAL table
   above** (this lane's convergence adjudication had already landed
   A3's exact layout; the arbiter confirms it independently). A4's
   notify table names THIS doc at `d35876af9` with the one-move
   delta (underworld [50]→[56], crimson [51]→[50], moon
   [52..57]→[51..56]) — that delta is the RETIRE table's, already
   fixed by the FINAL-SEAL layout; **this receipt closes the owed
   re-pin on the published record**. The Δ table above is the
   historical RETIRE-vs-92 comparison (audit trail), NOT a live
   layout. The cross-receipt state at the current tips: 92 re-pinned
   to A3 at g71 `8224c94f1`, its §1-SEAL.5 (`b27474a44`) recording
   this lane's re-pin as owed — now CLOSED; g72 `aa1645481` D5-RE-PIN
   (64a prefix retired, underworld [58]→[56], sites 53→107); a6 138
   `9a4acd729` D5-RE-PIN (pets 57-59, books 61/62) AGREES; g90 135
   `43fee29a4` ALREADY on A3 — **ALL FIVE L3 dependents now seat the
   identical strict-C#-row middle; the persist-order dispute is
   RESOLVED family-wide, zero open re-pins.** The arbiter's closing
   rule adopted: the C# rows (A1) are the tie-breaker for any later
   revision — not publication timestamp, not designation. The
   direct C# re-read this pass (`WorldFile.cs:1270-1414`) agrees
   line-for-line. Authority chain: THIS table = A3 = D3 = D5; 103
   L3-ARBITER the byte-order authority of record; 92 §1-FULL the
   claimant registry of record; this doc's L1/L2 + §8 merger-bible
   authority unchanged (A4's own 132-row pins that scope).
   The truncation-constant moves (every claimant's end-relative sites):
   g70/g71's 16/3 → 107 (92 §1.5's table), g69/d125's 1 → 107, g90's six
   sites 2 → 107 (135 §4's table), g72's eleven sites 31 → 107 (64-add
   §3's list), c43/f45's 44 → 107 (138 §5's +44 sites), g87's 3 → 107
   (its `EMPTY_V16_TAIL` family + the v12/v14/pre-v18 probes), c39/f39's
   5 → 107 (ONE set — the deduped block's witnesses keep one copy each);
   d109's end-relative sites need only the helper's new return value.
   Every claimant's round-trip witness keeps its assertions, only the
   tail lengths move. The stale totals published elsewhere (92 §1's 21,
   64-add's 53, 135 §4's 21, 138 §5's 65, this doc's FINAL 97/98) are
   superseded by THIS table — the merger seats all ten in one pass.
2. **The roll keep-one (g71 vs g72) — both PUSHED now.** g71's
   `f08e8d41e` wires `bg_rolls.rs` into `step_world_tail` (the surface
   rolls + the boot re-roll seam + the atlas budget guard) and g72's
   `4a75715ef`/`39479b551` push the cave roll + the `:11340`
   ORDER-CONTRACT + the cave-style DRAW consumer. The adjudication the
   FINAL phase owed: **keep ONE cave-roll implementation** — g72's
   `worldgen/cave_bg.rs` (the pushed `CaveRoll` trait driver with the
   boot-side Knuth dead-store seam, research/64 §2 correction) is the
   persistence-trio owner (`.wld` arrays / `.trwld` cave block / net
   header); g71's `bg_rolls.rs::randomize_cave_backgrounds` is the SAME
   port (`WorldGen.cs:8192-8249`) and dedups onto it. Keep g71's surface
   rolls + its `step_world_tail` wire; g72's order-contract pins the
   insert seat (after `apply_cave_backgrounds`, before the moon
   `Next(9)` — `WorldGen.cs:11339-11341` contiguous on genRand). The
   `world_hash` stale-by-design notes ride both; the moon-shift re-pin
   is g72's published pattern.
3. **`.wld` flags dedup — f45's flags file is the base (138 §5's verdict,
   re-verified against the pushed tips):**
   - `combatBookVolumeTwoWasUsed` (gate 259, tail 2036): g89e ⨉ g90 ⨉ a6
     ⨉ f45 ALL write it — **keep f45's** (the pushed `.wld`-specialist
     superset); the other three flips dedup.
   - `peddlersSatchelWasUsed` (gate 260, tail 2037): g90 ⨉ a6 ⨉ f45 —
     **keep f45's**.
   - bought-pets trio (gate 217, tails 2021-2023): a6 ⨉ f45 ⨉ g87 —
     **keep f45's** flips; g87's/a6's dedup (g87's `.trwld` claim is
     separate, §3-L3 rule 1).
   - the five moon-boss downeds (`:1347-1351`, v131+): c39 ⨉ f39 ⨉ f45
     all flip the same five `false`s — **keep f45's** writer half; the
     World-field declarations + death arms dedup per rule 1 (f39's
     season-lane `DoDeathEvents` owns the semantics).
   - the force-today pair (gate 212, tails 2002-2003): **f39 ONLY** —
     f45 `9c1a01a42` explicitly leaves those bytes to the f39 lane;
     take f39's `flags.rs` hunk verbatim.
   - g72's cave arrays (i32 widths, `WorldFile.cs:1260-1268`/
     `:2058-2066`) vs f45 Ph6's tree/cave claim — same fields, same
     gates; keep one codec half (f45's flags file is the superset;
     g72's arrays must survive it). g71's underworld/treeBG2..4/mushroom
     flips (`f08e8d41e` flags.rs) are independent bytes — they ride.
4. This lane contributes NOTHING to L3 (no save.rs/wld change).
5. **The g56 `.trvault` sidecar is NOT an L3 claimant** (§2g's
   receipt): a per-world PLAYER-favorites satellite (the `.trfav`
   precedent class), zero `.trwld`/`.wld` bytes — it never enters
   this union. Its write-retire rides the exit-save gate (§2g rule
   3), not the L3 merge; the back-read fallback stays inert
   post-fold.

## 4. The e10 voice co-lane (research/99 rows 67-68 — the retire receipt)

**RETIRE probe: `origin/task/W4-e10-player-sounds-wave` tip `432dd5ab1`
(over `693e3850a`, the voice-picker + persistence docs commit) — the
voice-write LANDED.** The FINAL probe's "NO `.plr` write" state is
superseded. What e10 carries now (the diff against main, re-read):

- **The char-creation picker** (`ui/char_creation.rs`, +866 lines):
  the voice row UI (`UICharacterCreation.cs:452-528` — the play button
  + prev/next cycling), `NewPlayerDraft.voice_variant: i32` +
  `voice_pitch_offset: f32`, the `is_male` helper (the
  `PlayerVariantID.Sets.Male` set `{0,1,2,3,8,10}` — identical to this
  branch's inline matches! arm).
- **The session threading** (`app/session.rs`): `save_created_character`
  passes `draft.voice_variant`/`draft.voice_pitch_offset` into the
  writer (its `last_saved_binary: 0` literal is the PRE-timestamp-fix
  shape — this branch's `utc_now_binary()` is the newer session, §3-L1
  rule 6).
- **Its own transport diff** (`player_file.rs`, +146 lines): i32
  `PlayerFile.voice_variant` + `PlayerFileWrite.voice_variant`, the
  `finish_player_file` `voice: Option<(i32, f32)>` parameter (None for
  the inc1/inc2 depths → the gender default stands in), the 1..4 clamp,
  the <280 Male-derive, the 279-subset gender default, three witnesses.
- `Player.voice_variant: i32` (ctor 1, `Player.cs:1346`/`:56045`),
  `voice_pitch_offset: f32` (`:1348`), `voice_override: i32` (`:1706`)
  — live fields feeding the ported `PlayHurtSound`/`PlayDeathSound`
  decision trees; its `693e3850a` player.rs note pins the remaining
  seam: "the live in-world Player's hydration from a loaded file is
  the f44c save-load audit's seam (`CharPlay` is routing-only, spec
  §9.4), so it stays at the ctor's 1 in play until that lands."
  **That seam is now LANDED here** (the e10c receipt below — this
  branch declares the same `voice_variant: i32`/`voice_pitch_offset:
  f32` live pair at the same ctor defaults, so e10's declaration hunk
  dedups textually; `voice_override` stays e10's — it is a per-tick
  equip-derived field, not a persisted one).

**Row 67-68 status (this lane):** COMPLETE on the `.plr` side —
`voice_variant: u8` + `voice_pitch_offset: f32` ride both models
(`player_file.rs:367/370`, write `:54924-54925` at `:2085-2086`, read
gates 280/281 at `:1159-1177` with the <280 Male-derive and the
`LastMinuteFixes` 1..4 clamp `:55831` in `finish_player_file`). The
golden fixture's fresh voice byte 1 + pitch 0f are pinned.

**THE KEEP-ONE FINAL (the transport collision the retire probe found):**
e10's diff and this branch's ph3 walk model THE SAME two slots with
different field types (i32 vs u8) and different plumbing (a
`finish_player_file` parameter vs the `PlayerTail` struct). **This
branch's u8 transport is the merge base** — it is the file-faithful
width (`(byte)voiceVariant`, `:54924`), it rides the complete-walk
superset (the inc1/inc2 depths get the gender default through
`PlayerTail::default` + the shared clamp; e10's None-parameter shape
achieves the same but only on its main-base reader), and its witness
(`the_voice_pair_round_trips_clamps_and_derives`) is the superset of
e10's three (round-trip + clamp + gender-derive in one). The fold
(§3-L1 rule 7): DELETE e10's `player_file.rs` diff, KEEP its picker +
draft fields + session threading re-pointed at the u8 writer
(`voice_variant: draft.voice_variant as u8` — the picker only offers
1..3, the cast is lossless; a hand-edited draft clamps on the next
read exactly like the C#). e10's `save_list.rs` test-fixture hunks
(the `voice_variant: 1, voice_pitch_offset: 0.0` literals) dedup onto
this branch's fixtures (same values, the u8 type).

**The hydration hunk — CLOSED on THIS branch (the e10c receipt, the
SEAL-phase port).** The RETIRE state ("STILL OPEN on both branches":
this branch's reader fed only the card list, `save_list.rs:294`; e10's
`693e3850a` note pinned `CharPlay` as routing-only) is superseded by
this phase's code commit. The census found the hunk SMALL (the reader
already models every field; only the apply-side was missing) and it
is ported on the canonical walk:

- **The park** (`app/flow.rs`, `apply_front_flow`): `CharPlay(i)` parks
  the card's `.plr` path on `App.play_player` — the
  `Main.SelectPlayer` → `data.SetAsActive` seam
  (`UICharacterListItem.cs:256-261`, `Main.cs:66208-66216`). A
  disk-less card (the session stand-in / a fake) parks `None`.
- **The carry** (`app/mod.rs`, `swap_world`): the park rides the
  `App::new` reset (`Main.ActivePlayerFileData` survives
  `Main.LoadWorld`); `App::new` gains the `play_player` parameter
  (boot/tests pass `None` — the fresh session spawn is untouched, the
  golden/screenshot determinism holds).
- **The apply** (`Player::hydrate_from_file`, `player.rs`): the read
  `PlayerFile`'s per-player state lands on the live Player — the
  stats (the reader's 500/200/400 clamps already applied), the G7
  unlock family (`extra_accessory`, the torch pair through the
  slot-11 SETTER then the builder-array copy in C# order — the
  `63204f85` property model, `ate_artisan_bread`, the six
  `used_upgrades` onto the five new sibling fields + the existing
  `used_gummy_worm`, the super-cart BitsByte split), the voice pair
  (`p.voice_variant = file.voice_variant as i32` — the 1..4 clamp
  already applied on the read side — + `p.voice_pitch_offset =
  file.voice_pitch_offset`, exactly the seam this section predicted),
  the pearl's `LuckState` mirror (the `:29142-29145` +0.03 term's
  input), and the counters (angler/golfer/deaths/team). The file's
  name becomes `App.player_name`; the `inv_favorites` bitfield
  hydrates `favorite_slots` (BUG-03's play-side half).
- **The precedence** (§2c, LIVE): the hydration runs FIRST in
  `App::new` (right after `Player::spawn`), the `.trplayer` sidecar
  restore runs SECOND (the existing block) and wins on the shared
  fields — the sidecar hp clamps against the HYDRATED max, not the
  fresh 100.
- **The new live fields** (`player.rs`): `voice_variant: i32` (ctor 1,
  `Player.cs:56045` — e10's live width, the u8 transport casts),
  `voice_pitch_offset: f32`, `used_aegis_crystal/used_aegis_fruit/
  used_arcane_crystal/used_galaxy_pearl/used_ambrosia` (the
  `Player.cs:852-862` siblings of the existing `used_gummy_worm` —
  the g90b declaration spots, so the powerup lanes' folds dedup
  textually). The voice CONSUMERS stay e10's (the hurt cue keeps its
  variant-1 stand-in until that fold); the upgrade CONSUMERS stay the
  powerup lanes' (the fields are the persistence owners).
- **The witnesses (written not run, code-only):**
  `w4_f44_the_character_load_hydrates_the_live_player` (player.rs —
  the full field set through a real write→read→hydrate round, both
  slot-11 arms: the array-overwrite disable + the setter-alone
  enable, the cart BitsByte split, the pearl mirror),
  `w4_f44_char_play_parks_the_cards_plr_path` (flow.rs — the disk
  card parks its path, the session stand-in parks `None`),
  `w4_f44_the_played_character_hydrates_before_the_sidecar_restore`
  (saveglue.rs — the real `App::new` e2e: the `.plr` hydrates, the
  sidecar hp 250 WINS over the file's 300-life at the hydrated max,
  the name/favourites land, the `None` park keeps the fresh spawn).

**The remaining shared remainder is the EXIT-SAVE alone** (the `.plr`
re-save at world exit — the save funnel writes the sidecar only):
103 §2b rule 3's condition (b). Condition (a) — this hunk — is MET.
The save direction casts back (`file.voice_variant = p.voice_variant
as u8`) when that lane lands. **The exit-save is now the ONE shared
gate for THREE published contracts** (the post-seal compose, §2g):
103 §2b rule 3's torch-line drop, §2f's `dh` conditional (its only
activation trigger is an in-session carrier need BEFORE the
exit-save), and g56 239 §1c's `.trvault` write-retire (+ its bank4/
favorites payload half of condition (a), which lands WITH the
exit-save's App-level payload writer). One seam, three dependents —
no lane blocks another; the exit-save lane owns the bank4 quadruples,
the `voidVaultInfo` bit-0 write (parity-only mirror, §2g rule 4) and
the bank-payload hydration extension in the same pass.
e10 owns the CONSUMERS (the voice
trees, the pitch remap `Utils.Remap(-1,1,-0.7,0.7)`); this lane owns
the TRANSPORT + the HYDRATION. The picker UI seam was CLOSED by e10's
push; the hydration seam is CLOSED by this port — the e10c receipt.

## 5. The back-compat matrix (cross-layer)

| File shape | Layer | Reader that loads it | Result |
| --- | --- | --- | --- |
| vanilla `.plr` 135-326 | L1 | this branch `read_player_fields` | full gate matrix (research/99 §2); 135-137 no playtime, <136 misc slot-1 skip |
| vanilla `.plr` <135 | L1 | refused (`pre-135`) | the legacy string-item layout is unmodelled (documented) |
| port 279 subset | L1 | `read_legacy_port_subset` | header + armor/dye/inv; tail defaults; G7 bools ride |
| port 326 inc1/inc2 | L1 | depth-1/2 retries | reached sections ride, rest defaults (research/99 §4) |
| port `.plr` 0-stamp | L1 | `read_player_file` | **mtime fallback fills the stamp** (`d331ea7c`, `LoadPlayer:55096-55099`); byte-level read keeps 0 |
| g89/g90/f33 `.plr` (main-base + their slots) | L1 | this branch's walk | their slots ARE G7/sweep slots — loads complete; their own readers load this branch's files too (same bytes; f33's raw torch bool reads through the slot-11 setter, §2a) |
| e10 `.plr` (main-base + the voice pair, i32 model) | L1 | this branch's walk | byte-identical slots (`:54924-54925` u8+f32 either way) — loads complete both directions; the i32/u8 difference is in-memory only (§4's keep-one) |
| `.plr` 229..229-only torch block (no builder array) | L1 | this branch's walk | the `:55242` setter's slot-11 value STANDS (the <230 count never reads slot 11) — the C# order, `63204f85` |
| `.trplayer` v1/v2 | L2 | any lane reader | kinds migrate (`migrate_kind`) |
| `.trplayer` v3-v6 | L2 | any lane reader | prefix rides, post-dye defaults |
| `.trplayer` v7 (5 historical numeric shapes + g90b six-line + f33 pair-line + g87 dh-sentinel) | L2 | g88f v11 reader (the numeric rules LANDED) | the six numeric/six-line shapes load losslessly (103 §4 matrix + `g90bs_six_line_v7_loads_positionally` + `f33s_torch_only_v7_loads_positionally`); the `dh` line needs the §2e rule (next row) |
| `.trplayer` v7 g87 `dh`-sentinel line | L2 | g88f v11 reader WITHOUT the §2e rule | **LOSS** (the 3-token `dh` line hits "any other shape → defaults kept" — both unlocks silently false, NOT corrupt; g88h's census verified `persist.rs:565`). Under the §2f DEFAULT-DROP this is the accepted merge behaviour — the flags ride L1 + the hydration; the §2e recognition is owed only at the conditional v12 fold |
| `.trplayer` v7 g87 `dh`-line | L2 | g87's own reader | loads (its native shape); the numeric v7 files default both false — aligned but lossy, NOT the merge winner |
| `.trplayer` v8/v9 short lines | L2 | g88 superset / g89 reader | tokens 1-3 back-read, rest false |
| `.trplayer` v9 g88-superset | L2 | g89's reader | line 8 reads; pairs/bread/loadout lines IGNORED (g89's reader stops after line 8 — lossy but aligned; the superset reader is the merge winner) |
| `.trplayer` v9 g89-file | L2 | g88 superset | unlock line positional, rest defaults |
| `.trplayer` v10 short unlock line (3-token) | L2 | g88 v10+ reader | tokens 1-3 read, crystal trio false (103 §7 back-read) |
| `.trplayer` v10 (no line 16) | L2 | g88f v11 reader | torch pair defaults false (`the_v10_superset_still_loads_without_the_torch_pair` — LANDED) |
| `.trplayer` v11 (no line 17) | L2 | the §2e-extended v12 reader | the `dh` pair defaults false (the append-only back-read) |
| `.trplayer` v7 g90-six-line | L2 | g89's reader | **CORRUPTS** (bread→aegis; 135 §2) — the recognition rule landed in g88f, the merge winner is safe |
| `.trplayer` v7 f33-pair-line | L2 | a superset reader WITHOUT the §2d rule | **CORRUPTS** (torch pair → life pair, hp feeds 1/1) — the value-range rule landed in g88f |
| `.trplayer` v7 f33-pair-line | L2 | f33's own reader | loads (its native shape); g90/g88 files' first line mis-reads as the pair — f33's reader is NOT the merge winner |
| `.trwld` ≤17 | L3 | any v18 reader | union tail unread, Reset defaults (each claimant's own: g72 the `World::new` zeros, g71 the setBG slot-0 defaults, c39/f39 the downs, g87/g90 false, c43/f45 the identity zeros + the re-stamp funnel) |
| `.trwld` v18 single-claimant (g70 16B / g71 3B / g69 1B / d125 1B / g90 2B / g72 31B / c43-f45 44B / g87 3B / c39-f39 5B / d109 dense-only) | L3 | the 107-byte-head union reader | **EOF/garbage — not back-compat** (92 §1 + 64-add §3: forward-only, no released v18; all ten are unmerged task work) |
| `.wld` any release | L3 | `wld::parse` gates | per-field gates; book-two 259 / satchel 260 / pets 217 / moon-boss v131+ ride f45's flags base; the force-today pair 212 rides f39's hunk; the cave arrays ride g72/f45-Ph6's one codec half; the underworld/treeBG2..4 flips ride g71's |
| `<world>.trvault` (g56 40-char 0/1) | satellite | `load_vault_favorites` | loads today (the interim carrier, §2g); post-exit-save it is the ONE-RELEASE back-read fallback for pre-fold saves — `.plr` first, `.trvault` second, naturally inert once the write retires (239 §1c) |
| no `.trvault` beside a post-fold save | satellite | the fallback | favorites come from the `.plr` bank4 fav bytes (row 47) through the payload hydration — no sidecar file expected (the write retired, §2g rule 3) |

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

**RETIRE-phase verification (this pass):** the pushed-tip re-reads —
g88f `a9f1c29d8` (research/103 FULL at v11 + research/122's v11
post-receipt), g71 `f08e8d41e` (research/92-doc-sweep-bg FULL + the
save.rs/flags.rs/bg_rolls.rs diffs — the underworld seat's trigger
verified in the pushed code), g72 `4a75715ef` (research/64-v18-union-
addendum FULL + the save.rs diff), g90 `94d33f2ca` (research/135 FULL +
the save.rs diff), a6 `d7de8bd53` (research/138 FULL — the per-item
verdict table + §4's byte-compare + §5's f45-collision + §6's
token-order), g87 `dfbcaf94b` (the FULL persist.rs/save.rs/wld diffs),
c39 `8ecb288aa` + f39 `bd5a5a24a` (the FULL save.rs/flags.rs/wld-mod
diffs — the five-byte block's field-identity verified line-by-line),
f45 `6352d3840` (the save.rs identity block + the flags.rs flips +
`9c1a01a42`'s force-today deferral note), e10 `432dd5ab1`/`693e3850a`
(the FULL player_file.rs/session.rs/save_list.rs diffs + the player.rs
hydration note), d109 `81894095` (the save.rs dense-block diff); the
C# `WorldFile.cs:1341-1410` flags-order re-read (the moon-boss/pets/
books/force-today write order pinned); this branch's player_file.rs
voice walk re-read (`:1159-1177`/`:1352-1353`/`:2085-2086` — the
keep-one's u8 base verified). No cargo/tests/rustfmt/graphify per the
lane constraint.

**SEAL-phase verification (this pass, item 1 — the 132↔92 convergence):**
g71 `f2d8c2c67` fetched + research/92-doc-sweep-bg §1-FULL read FULL
(the 13-row claimant registry, the §1-FULL.2 layout + writer/reader
code blocks, §1-FULL.3's back-compat matrix, §1-FULL.4's per-branch
truncation-move table); the byte-order comparison ran block-by-block
against this doc's RETIRE table (the Δ table in §3-L3 rule 1); the
adjudication re-read `WorldFile.cs:1240-1414` directly (the flags
write order: crimson `:1281`, moon-boss `:1347-1351`, underworldBG
`:1376`, pets `:1392-1394`, empress `:1395`, books `:1407-1408`; the
identity fields `:1225-1227`/`:1250-1251` confirmed ahead of the flags
stream). Both layouts' 107-byte totals verified by summation
(16+31+3+1+5+1+3+1+2+44). No cargo/tests/rustfmt/graphify per the lane
constraint.

**SEAL-phase verification (item 2 — the g88h receipt):** g88
`6a829a0e8` fetched + research/103 read FULL at that tip (§1a's
f44-FINAL cross-receipt, §2a's DD2 v12-contract-seat census, §2b's
torch-line drop conditions, §5a's g87-triple-flag verdicts, §9's
receipts-complete table + steps 10/11); its step-10 100-byte
arithmetic re-derived against our FINAL `bdd1cc79a` base (the
97-slip era) and superseded by the 107-byte FINAL-SEAL table; its
`dh`-DROP vs our §2e-fold collision adjudicated in §2f (default DROP,
conditional fold). No cargo/tests/rustfmt/graphify per the lane
constraint.

**SEAL-phase verification (item 3 — the hydration port):** the
census ran over every `read_player_file`/`PlayerFile` caller (only
`save_list.rs:294`'s card list + the creation writer consumed the
reader — no live-Player apply existed), the `CharPlay` routing
(`flow.rs`'s click gate + `next_flow_state`), the `swap_world` →
`App::new` reset and the sidecar restore block's position; the C#
anchors re-read (`Main.cs:66208-66251` SelectPlayer,
`UICharacterListItem.cs:256-262` PlayGame, `Player.cs:55216-55278`
the unlock reads, `:55710-55745` the cart/voice reads, `:3950-3964`
the property, `:852-862` the six field decls, `:56045` the voice
ctor, `:29142-29145` the pearl term — the +0.03 mirror verified
against `luck.rs:201-203`); e10's live-field shapes re-read at
`432dd5ab1` (player.rs `:2000-2010` — the i32/f32 pair matched). A
brace/paren balance sweep of the five touched files (the player.rs
−1 paren delta is pre-existing at HEAD, comment text) + a full
`git diff` re-read. 3 witnesses written not run; no cargo/tests/
rustfmt/graphify per the lane constraint. The golden `.plr` parity
is untouched (no codec change — the port is apply-side only; the
`None` park keeps every existing boot/screenshot path byte-identical).

**Post-seal receipts-pass verification (this pass — the D5-A3 re-pin +
the g56 compose):** g88 `d09d7e10a` fetched + research/103's
L3-ARBITER FINAL (A1-A4) read FULL — A3's table byte-compared against
this doc's FINAL-SEAL layout field-for-field (IDENTICAL: same claimant
order, same offsets, same 107 sum, dense LAST); A4's 132-row move list
re-derived against the RETIRE Δ table (the owed one-move delta is the
RETIRE column's, already fixed at the seal). g71 `b27474a44` fetched +
research/92 §1-FULL.2 (re-pinned `8224c94f1`) + §1-SEAL.5 read — its
132-owes row consumed and closed by §3-L3 rule 1's receipt; g72
`aa1645481`'s D5-RE-PIN + a6 `9a4acd729` + g90 `43fee29a4` states read
via 92 §1-SEAL.5's cross-receipt table (four-of-five closed there, the
fifth — this doc — closed here). g56 `54c63d8d4` fetched + research/239
FINAL read FULL (§1a-§1d, §2, §3, §5 — the fold-contract, the
zero-touch compose, the family map); its bank4 anchors re-verified
against research/99 rows 47/48 (offsets 2001/2401, gates 198/255/199)
and the hydration's scope note re-read in the landed code
(`player.rs:2503-2560` — stats/unlocks/voice/counters only, the bank
payloads explicitly deferred). The C# flags stream re-read directly
(`WorldFile.cs:1270-1414` — crimson `:1281`, quintet `:1347-1351`,
underworld `:1376`, pets `:1392-1394`, empress `:1395`, books
`:1407-1408`, the interstitials `:1380`/`:1386-1387` confirmed). The
landed hydration port re-verified at HEAD `0d1ee2045` (the `CharPlay`
park `flow.rs:1302-1304`, the `App::new` apply `app/mod.rs:2848-2857`
BEFORE the sidecar block, `player.rs::hydrate_from_file`, the
name/favourites landing `:3445`/`:3645`, `save.rs` VERSION still 17 —
no code touched by this pass). Docs-only pass; no cargo/tests/
rustfmt/graphify per the lane constraint; the golden `.plr` parity
untouched.

## 8. The SEALED merger persist-checklist (THE persist-family bible — all
recognition rules, all claimants, all dedup steps, in order)

**Order: L1 → L2 → L3** (§3's rationale). Every step below is a
mechanical instruction; every adjudication is already made — the
merger executes, it does not re-decide.

### Step 1 — L1 `.plr` (`player_file.rs`)

1. Take THIS branch's `player_file.rs` whole (`5c8bbb160` lineage: the
   71-row research/99 walk, G1-G7, the three shim depths, the
   timestamp fix, the slot-11 property fix) + the hydration port
   (`Player::hydrate_from_file` + the `CharPlay` park + the `App::new`
   apply — §4's e10c receipt; the g-lanes' live `Player.used_*`
   declarations dedup onto the five sibling fields at the same
   spots).
2. Fold g55's loadout semantics onto the `LoadoutData` transport
   (byte-identical, rows 65-66).
3. Dedup g89's 3 + g90's 5 named unlock bools onto `used_upgrades[6]`
   (indices: crystal 0, fruit 1, arcane 2, pearl 3, gummy 4, ambrosia
   5 — the `.plr` file order `:54781-54786`, NOT the sidecar token
   order); dedup g90's `ate_artisan_bread` onto the G7 field (same
   name/slot — textual).
4. Dedup f33's torch transport onto rows 19-20 (same named fields);
   KEEP f33's live `set_using_biome_torches` setter + use arm + event
   machines; never resurrect a stored `using_biome_torches` live
   field. The 5043 use-arm owner is a6's faithful setter (138 §1).
5. Fold c43/f45's envelope: the `.bak` copy, the revision CARRY
   (`PlayerFileWrite::revision` — the one remaining L1 gap), the
   UtcNow/mtime helpers dedup (keep one each).
6. Fold e10's voice-write per §4's keep-one: DELETE its player_file.rs
   diff (the u8 transport here is the base), KEEP the picker + draft
   fields + session threading (`as u8` cast), dedup its save_list.rs
   fixture literals + its round-trip witness onto
   `the_voice_pair_round_trips_clamps_and_derives`; its player.rs
   voice-field declaration hunk dedups onto the hydration port's
   pair (same i32/f32 shapes, same ctor defaults); its consumers
   (the voice trees + the pitch remap + `voice_override`) land as-is.
7. Keep the timestamp helpers whole; every lane's `last_saved_binary:
   0` session literal → `utc_now_binary()` (fixtures stay 0).
8. g87's `.plr` half needs NO fold (rows 18/64 already ride).
9. **Recognition rules: NONE** (binary, version-gated) — the gate
   matrix is research/99 §2; the accepted range [135, 326], the 279
   subset shim, the inc1/inc2 retries.

### Step 2 — L2 `.trplayer` (`persist.rs`)

1. Take g88f's pushed `persist.rs` whole (`a9f1c29d8`: writer v11,
   tokens 1-6 LIVE crystal=4/arcane=5/pearl=6, the torch line 16, the
   tolerant reader). 103 §9 is the lane-local checklist; this step
   list is the cross-layer one.
2. The v7 recognition rules ALREADY LANDED (verify, don't re-add):
   - **135 §2's six-line rule**: exactly six 1-token lines →
     positional bread/crystal/arcane/pearl/gummy/ambrosia.
   - **§2d's torch-pair rule**: exactly ONE 2-token line, both 0/1 →
     the biome-torch pair (the value-range argument: maxima start
     100/20).
   - **The lone-1-token aegis-first ambiguity** (103 §3): the ONLY
     surviving bounded misread (g90 bread-only → aegis slot); ends
     at the fold, documented.
3. The g87 `dh` line — **DEFAULT DROP** (§2f's g88h receipt; g88h
   §5a/§9 step 9 + 138-FINAL §10.4): the v11 reader tolerates a g87
   file lossy-not-corrupt (`persist.rs:565`'s `_ => {}` arm), both
   flags ride L1 + the hydration. ONLY IF the merger needs the
   in-session carriers before the exit-save lands: ADD **§2e's
   `dh`-sentinel rule** (first token `dh` → the g87 unlock pair) +
   append **line 17 `dh <extra> <cart>`**, writer bumps **v12** (the
   DD2 version reservation moves to v13; token 7 on the unlock line
   stays DD2-reserved). v11 back-read: missing line 17 → both false.
   Restore: plain bool fields, NOT `enabledSuperCart` (L1-only).
4. Dedup the `SavedPlayer`/App-restore fields (g90/f33 folded; g87's
   two need no L2 seat under the default — the hydration restores
   them from L1; they join only at the conditional v12 fold).
5. Seat 138 §1's use-arm verdicts (§3-L2 rule 7's table — g88 keeps
   the crystal/fruit carriers, a6 owns the shimmer/book/satchel/
   license/torch arms, g90 owns the loaf + the Player-field consumer
   anchors, c36's `complete_condition` is the shared dispatch).
6. The torch line's conditional DROP (103 §2b rule 3): revisit ONLY
   when the character-load hydration AND the exit-save both land —
   (a) is MET (the SEAL port), so the trigger is the exit-save alone
   (the same shared gate as §2f's dh conditional + §2g's vault
   retire; §4's compose).
7. **The g56 `.trvault` compose (§2g's receipt — execute at the
   folds, retire at the gate):** keep g56's three sidecar fns + its
   saveglue write + its `App::new` read alongside g88's superset
   (pure insertions at the shared seams, 239 §1d — hydration FIRST,
   superset restore SECOND, `.trvault` read THIRD); NO vault line in
   the `.trplayer` writer ever. AT the exit-save landing: drop the
   `.trvault` WRITE, keep `load_vault_favorites` as the one-release
   back-read, extend the hydration + the exit-save writer with the
   bank4 quadruples (LIVE favorites, fav byte @ 2001+10·i+9) +
   `voidVaultInfo` bit 0 (parity-only — never hydrate it against the
   per-tick `HasItem(4131)` refresh, §2g rule 4).
8. **Recognition rules summary (the complete registry)**: 103 §3's
   positional v8-v11 unlock line + the shape walk (1-token bread /
   2-token pairs / the pre-41 index line / 41-token sets / the v11
   last-2-token torch line) + 135 §2's six-line + §2d's all-0/1 pair +
   §2e's `dh` sentinel + the lone-flag ambiguity. World globals NEVER
   ride L2 (§2b rule 4).

### Step 3 — L3 world (`save.rs` `.trwld` + `wld/` `.wld`)

1. Seat the **107-byte-head union** (§3-L3 rule 1's table — THE
   authoritative layout, the FINAL-SEAL order after the 132↔92
   convergence receipt, CONFIRMED VERBATIM by 103 L3-ARBITER A3 /
   D5-A3 `d09d7e10a` — the re-pin receipt; ALL FIVE dependents
   re-pinned, zero open order disputes): g70 16 + g72 31 + g71 3 +
   g69 1 + c39⨉f39 5 (ONE deduped block) + underworld 1 + g87 3 +
   d125 1 + g90 2 + c43/f45 44, then d109's W×H dense LAST. VERSION
   stays 18, forward-only. `v18_tail_len` → `107 + width*height`.
   The `.trvault` satellite NEVER enters this union (§3-L3 rule 5).
2. Move EVERY claimant's truncation constants to 107 (§3-L3's list;
   the stale 21/53/65/97/98 totals in 92/64-add/135/138/this-doc's-
   FINAL are superseded).
3. The roll keep-one: g72's `cave_bg.rs` owns the cave roll + the
   persistence trio; g71's surface rolls + `step_world_tail` wire
   stay; g71's `randomize_cave_backgrounds` dedups (§3-L3 rule 2).
4. The `.wld` flags base = f45's file (138 §5); dedup book-two (g89e/
   g90/a6), satchel (g90/a6), pets (a6/g87), moon-boss (c39/f39) onto
   it; take f39's force-today hunk verbatim (f45 defers); g72's cave
   arrays + g71's underworld/treeBG flips ride (§3-L3 rule 3).
5. The moon-boss death-arm keep-one: f39's season-lane `DoDeathEvents`
   owns the semantics; c39's exec arms dedup; the resolved seasons
   re-derive (f39's `refresh_seasons`), never persist.
6. **Dedup steps (the invariants)**: one writer per file per save;
   one slot/token per field; world globals never ride L2; the
   dual-carrier fields keep BOTH carriers until the exit-save lands;
   character-load restore FIRST, sidecar restore SECOND (§2c).

### Step 4 — witness re-pins + acceptance

1. Every lane's round-trip witnesses keep their assertions; only tail
   lengths (L3 → 107), field names (L1 array dedup, L2 token map) and
   version headers (L2 → v12 at the dh fold) move.
2. The back-compat witness-matrix is §5 — run it as the merge
   acceptance list (every row is a written witness on some lane).
3. The character-load hydration is LANDED on this branch (§4's e10c
   receipt — the SEAL-phase port). The EXIT-SAVE (the `.plr` re-save
   at world exit) is the one remaining SHARED remainder (§2c/§4) —
   a seam, not a conflict; no lane blocks another. It is the SINGLE
   gate for the torch-line drop (103 §2b rule 3), the dh conditional
   (§2f) AND the `.trvault` write-retire (239 §1c, §2g's compose) —
   the exit-save pass also lands the bank4/favorites payload
   hydration + writer and the `voidVaultInfo` parity byte (§8 step 2
   item 7).

### Step 5 — canaries

1. Add/keep the anchors rows for the union reader, the superset
   writer and this branch's walk; never delete rows to silence a
   failure (AGENTS.md).
2. The golden `.plr` fixture stays byte-identical through every step
   (all fold defaults emit the old hardcoded bytes — the voice pair
   1/0f, the pets/books/downeds false, the identity zeros).