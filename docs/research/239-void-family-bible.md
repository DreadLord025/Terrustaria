# 239 — The VOID-FAMILY BIBLE + the g56 RETIRE verdicts (W4-g56 final, code-only)

Branch `task/W4-g56-ui-void-vault`, retire over DONE `3ab631923` (phase chain
`273814c51` → `6727fb9ad` → `21d01fa0a` → `d364a2f4c` → docs `3a07ce683` →
void-chain items 1-5 `b7a9cdcc0`/`85711f19e`/`cadec8ef8`/`9f6d2df51`/`ddea019d1`
→ docs `05b04d527` → witness/parity `3ab631923`).

Number per **registry-150** (`task/W4-d54-dust-sites-world`
`150-research-number-registry.md`, tip `38bfd1d22`): the free block starts at
237; the all-tip re-scan at this retire finds **237 already consumed** by
`task/W4-e23-bestiary-ui-grid` (`237-bestiary-ui-final.md`, tip `7a7779690`)
with **two same-key forks** (`task/W4-g48-net-apply-arms-1`
`237-f11-tail-fix.md` `9460c85d9`, `task/W4-g61-npc-spawn-dust`
`237-g61-spawn-family-final.md` `fd406ecba` — a NEW collision the registry
predates; per its first-timestamp rule the two later forks renumber on their
OWN branches), and **238 consumed** by `task/W4-c8-style66-crawling-critters`
(`238-c8-crawler-final-census.md`, tip `7c2de9c06`). **239 is free at every
`origin/task/W4-*` tip** (the `1[5-9][0-9]|2[0-9][0-9]` ls-tree scan over all
remote tips, re-fetched this session). This doc takes **239**.

Co-lane tips read FULL this retire (re-fetched): f44 `d35876af9`
(research/99 + 132 FINAL-FINAL), g88 `d09d7e10a` (research/103 SEALED +
ARBITRATED, over the brief's `fc5cf11ae`; the v11 code stands `a9f1c29d8`),
e19 `e7e466d46` (research/144-148 incl. the 145 repin list), g12 `2b1f8a999`,
d122 `962084663` (research/142 FINAL).

## 1. The `.trvault` sidecar vs the f44 `.plr` bank4 rows / the g88 `.trplayer` superset — THE FOLD CONTRACT

### 1a. The three carriers today

| Carrier | Owner | Shape | Scope |
| --- | --- | --- | --- |
| `.plr` rows 47/48 | **f44** `task/W4-f44-save-plr-2` `d35876af9` | `bank4[40]` × `(type i32, stack i32, prefix u8, favorited bool)` @ **2001** (400 B, write `Player.cs:54856-54862`, read gate 198 `:55488`, fav gate 255 `:55495-55498` — the fav byte rides at offset **2001 + 10·i + 9**) + the `voidVaultInfo` BitsByte @ **2401** (`:54863`, gate 199 `:55501-55503`; **bit 0 = `IsVoidVaultEnabled`**, `Player.cs:3900-3910`, the only bit vanilla uses) | the per-character file; f44's `PlayerFile.bank4: [(i32,i32,u8,bool); 40]` (`player_file.rs:284`) reads the fav bool verbatim (`:887`), the aired-slot policy keeps stack+fav (G1/G3, research/99 §3) |
| `.trplayer` v7-v11 | **g88** `task/W4-g88-life-fruit-use` `d09d7e10a` (code `a9f1c29d8`) | the live-session sidecar superset: unlock line 6 tokens, life/mana pairs, bread, loadout block, torch pair LAST | **NO bank4/vault line exists** in the v11 writer (`persist.rs` save arms read line-by-line this retire); the superset never claimed the vault |
| `<world>.trvault` | **g56** (this branch, `9f6d2df51`) | 40-char `0`/`1` string, one per bank4 slot, index 0 first (`persist.rs::save_vault_favorites`/`load_vault_favorites`/`vault_favorites_path`; wired `saveglue.rs` save + `App::new` load) | the interim favorites carrier — per-WORLD file (the port's session model), NOT per-character |

### 1b. What f44 actually carries today (the retire probe)

f44's writer emits bank4 at **fresh-ctor defaults** — the only `.plr` write
seams on its tip are `session.rs::save_created_character` (the creation save,
`..Default::default()` tail → bank4 all-air/all-false) and the golden/fixture
writers. **No live-session `.plr` re-save exists on ANY tip** (f44 research/132
§2c: "the `.plr` has no in-session re-save" — the character-load hydration and
the exit-save are the SHARED remainders, §2c/§4, "seams, not claims"). So
f44's bank4 fav bools are a **file-model claim, not a live carrier**: a player
who stars a vault slot mid-session has nowhere in f44's tip to persist it.

g88's v11 superset likewise has no vault line (1a), and g88's own research/103
§2b publishes the CONDITIONAL DROP rule for its dual-carrier flags: drop the
sidecar line when **character-load hydration AND the exit-save both land**;
restore precedence per f44 research/132 §2c — **character file first, sidecar
second, sidecar wins as the newer state**.

### 1c. THE VERDICT — keep-one: `.trvault` COEXISTS through the folds, retires only at the two-seam gate

**When f44 lands bank4: COEXIST — do NOT retire `.trvault`.** f44's landing
seats the byte-shape (reader + model + golden parity), not the live save path;
retiring the sidecar at that point would silently drop every mid-session star
(the same corruption class 132 §2b's conditional-drop rule exists to prevent).
The keep-one carrier for the favorites TODAY is **this branch's `.trvault`**;
the keep-one owner of the `.plr` bank4 byte-shape is **f44** (research/99 rows
47/48 canonical; g56 ports no `.plr` bytes — verified: g56's tip touches no
`player_file.rs`).

**The retire gate (both required):**
1. **Character-load hydration** — `App::new`'s player restore reads the `.plr`
   bank4 rows into `Player.banks[3]` + `Player.vault_favorites` (fav bool per
   slot) and `voidVaultInfo` bit 0 into the enabled flag's saved state;
2. **The exit-save** — a live-session `.plr` writer (f44's lane) emits the
   bank4 quadruples with the LIVE `vault_favorites` (fav byte @ 2001+10·i+9)
   and `voidVaultInfo` (bit 0 = the enabled latch) from the live player.

When BOTH land: the merger **drops the `.trvault` write** (delete the
`saveglue.rs` call + optionally the writer fn), keeps
`load_vault_favorites` as a **one-release back-read fallback** for pre-fold
saves (the `.trplayer` back-compat precedent), restore order **`.plr` first,
`.trvault` second** ONLY for pre-fold files (post-fold the sidecar is never
written, so the fallback is naturally inert). The `.trplayer` superset needs
NO vault line — the favorites are `.plr`-resident in vanilla; adding a v12
line would re-create the dual-carrier this contract retires. (If the merger
prefers a sidecar interim for the exit-save seam, g88's line-append rules
apply — but the recommendation is: no new carrier, wait for the `.plr`
exit-save.)

**The `voidVaultInfo` nuance (row 48):** bit 0 mirrors `IsVoidVaultEnabled`,
which vanilla RE-DERIVES per tick from `HasItem(4131)` (`Player.cs:24550-24553`,
the port's `void_vault_enabled` refresh, S1-P-i43). The saved bit is therefore
a one-tick-stale mirror in C# too — the fold writes it for byte parity (f44's
model already carries the byte) but the port must NOT hydrate
`void_vault_enabled` from it against the per-tick refresh (the refresh wins,
C#-identical). g56 keeps no runtime field for the BitsByte; the byte rides
f44's model untouched.

### 1d. Merger conflict notes (persist.rs / saveglue.rs / app/mod.rs)

- `persist.rs`: g56 inserts `vault_favorites_path` + the two fns between
  `player_path` and `map_path` (`@@ -33,6 +33,46`); g88 inserts `loadout_item`
  at the SAME seam (`@@ -33,6 +187,22`). Both are pure insertions — keep BOTH
  (g56's block first, g88's helper after, or either order; no shared lines).
  g88 rewrites `save`/`load` bodies; g56 touches NEITHER (verified: g56's
  persist.rs hunks are `@@ -33` + `@@ -274` tests only).
- `saveglue.rs`: g56 appends the `.trvault` write AFTER the `persist::save`
  error arm; g88 widens the `persist::save` call (the `equip_share_flags`
  arg). Same statement region — the merged form is g88's widened call + g56's
  appended block, verbatim both.
- `app/mod.rs::new`: g56's restore hunk sits after `loaded_kills = saved.kills`
  (`@@ -2954`); g88's restore hunks sit at `@@ -2869`/`@@ -2936` (before it).
  Pure insertions, no overlap; the restore ORDER at merge is g88's superset
  block, then g56's `.trvault` read — which already matches §1c's
  sidecar-second precedence.

## 2. The e19 co-lane auto-pickup — DISJOINT CONFIRMED (+ the 16-box repin receipt)

e19 tip `e7e466d46` (round 3 items 1-5 + the final research/144-148). The
brief's "e19c-lights-resume-active" is the SAME branch
(`task/W4-e19-liquid-quicksell-shimmer` — the only e19 remote; its
`entity_lights.rs`/`item_lights.rs` deltas are the 16-box test repins, not a
lights port).

- **`bank.rs`: e19 diff EMPTY** (`git diff origin/main...e19 -- bank.rs` —
  zero lines). `void_vault_take`/`can_void_vault_accept`/the `DoCoins`-reads-
  `this.inventory[i]` quirk note (`bank.rs:51`) untouched.
- **`world_sim/items.rs`**: e19's hunks are `@@ -117` (spawn_ctx snapshot),
  `@@ -668` (repulsion + splash drain), `@@ -721` (`item_splash` fn), then
  test modules. The void-bag arm region (`vault_enabled` :173, the GrabItems
  vault fills :481-:497/:550-:575, the Cofveve probe :602) carries **no e19
  hunk and no `void|vault|bank4|4131|5325` diff line** (grep over the full
  diff: zero matches). g56's own items.rs delta is comment-only (the item-3
  census note) + the `vault_favorites` closure arg at :603 — disjoint.
- **`item.rs`**: e19 rewrites the drop-physics half (`SIZE` 14→16, ItemSplash,
  repulsion, wet flags); it does NOT touch `item_space`/
  `can_item_slot_accept_pickup` (its tip still carries the 6-param
  `item_space`). g56's hunks are `@@ -394/-402/-437` (the Cofveve 7th param)
  + its own test block — no line overlap; the merger keeps g56's signature
  and e19's physics (the `vault_favorite` probe composes with the 16-box
  pull gate unchanged).
- **`chest.rs`/`editing.rs`/`clicks.rs`/`ui/*`**: e19's chest.rs hunk is
  `@@ -1399` (a hitbox test repin), editing.rs `@@ -17853` (resting-fixture
  repins), clicks.rs `@@ -5983` (appended shop-click tests) — all outside
  g56's hunks (chest `@@ -895..-2701`, editing `@@ -4647/-13586`, clicks
  `@@ -413..-4200`). `frame/draw.rs`: e19 `@@ -375/-1085/-1780` (shimmer
  alpha + cursor_override args) vs g56 `@@ -186/-568/-1329/-1727/-1754/-2209`
  (the lens head, the glow/favorites draw args, the lastMouseInterface tail)
  — the nearest pair (e19's cursor_override insert at :1780 vs g56's
  favorites/glow args at :1754-1816) lands in the same `draw_hud` call but on
  DIFFERENT argument lines; the merged call carries both. `assets.rs`: e19's
  cursor_overrides insert (:2412) vs g56's ui_back2/ui_back19 (:2417) — both
  pure insertions in `Lookups`, keep both.
- **The lights resume**: e19's `entity_lights.rs`/`item_lights.rs` changes are
  14→16 test repins only; g56's lens light feed
  (`render/pylons.rs::draw_void_lens_projectile` → `EntityLights.add`, the
  `AddLight(0.4,0.2,0.9)` one-frame-late convention) rides the draw-side proj
  loop, not the drop-lights path — disjoint.
- **The 16-box repin receipt (e19 research/145, the g56 row)**: e19's census
  lists g56's `item.rs:1829-1830/:2017/:2026` (grab edge 44/45 + magnet
  293/193) for repin to **42/43 + 292/192**. Verified: g56's tip still carries
  the main-inherited 14-geometry pins at exactly those lines (they are the
  SHARED i47/i52 fixture block, not g56-authored witnesses). Per 145's
  merge-order note the FIRST merge landing the 16-box repins them once —
  **g56 takes e19's canonical repinned side at merge** (e19's
  `item.rs:2129-2141`/`:2325-2339` forms); g56's own 27 void witnesses carry
  NO drop-geometry pins (no `item::SIZE` use in `void_lens.rs`/`ui/glow.rs`),
  so no lane-unique repin is owed. Receipt recorded; no code change here
  (the repin lands with the 16-box merge, not before — repinning against the
  still-14 constant on this branch would break the written witnesses).

**Verdict: DISJOINT — no keep-one needed; the auto-pickup arm stays
S1-P-i43/i47/i50's, g56's census note stands (cadec8ef8).**

## 3. The g88 persist-compose — ZERO-TOUCH RECEIPT

g56's persist.rs delta vs main is **purely additive**: `@@ -33,6 +33,46`
(the sidecar doc + `vault_favorites_path`/`save_vault_favorites`/
`load_vault_favorites`, inserted between `player_path` and `map_path`) and
`@@ -274,4 +314,31` (the round-trip witness appended to `mod tests`). The
`SavedPlayer` struct, `save`, `load`, the version ladder — **zero lines
changed** (the full `+/-` diff read this retire: no `-` line outside the test
tail context).

g88's persist.rs delta (1258 insertions) rewrites `save` (v11 writer +
`equip_share_flags` param), `load` (the v7-v11 ladder), the module header and
the tests — its hunks `@@ -48/-69/-78/-107/-115/-130` all sit INSIDE the fns
g56 never touches. The only shared seam is the post-`player_path` insertion
point (§1d — both pure insertions, keep both). **Compose is clean: g88's v11
superset + g56's sidecar fns coexist with zero semantic overlap; the v11
round-trip witnesses are untouched by g56 and vice versa.** Receipt: the
`.trvault` carrier neither reads nor writes any `.trplayer` byte.

## 4. `SetGlowForChest` — the g12/d122 co-lane census: KEEP-ONE = g56's `ui/glow.rs`

The all-tip census (every `origin/task/W4-*` tip's `crates/tr-app/src/ui/`
ls-tree'd for `glow.rs`, plus the `SetGlowForChest|chest_glow|GlowState|
set_glow|glow_time` grep over g12/d122/g58/f35's crates + research):

- **`ui/glow.rs` exists on exactly ONE tip: g56's** (the 355-tip scan).
- g12 `2b1f8a999` and d122 `962084663` each carry ONE `SetGlowForChest`
  mention — the SAME inherited base comment `chest.rs:870` ("the
  `ItemSlot.SetGlowForChest`/input-navigator arms are UI state the port does
  not carry"), which PREDATES g56's port. Neither lane ported any glow arm
  (g12's lane is dust/render sweeps — its `DrawSavings` copy b467699f is
  d122's keep-one per research/142, unrelated; d122's lane is the shop/savings
  superset — `ui/slot.rs`/`item_sorting.rs` diff EMPTY on its tip).
- g58 (shop-happiness) touches no slot/glow file; f35's bag-open census
  (research/66 → 195 per registry-150) claims the bag DROP rates, no glow.

**Verdict: KEEP g56's `ui/glow.rs` package (ddea019d1) — no duplicate exists.**
The stale base comment at `chest.rs:870-871` is corrected on THIS branch (the
glow half now lands two screens below it; the input-navigator half stays
unported). At merge, g12/d122's inherited copies of the old comment resolve
to g56's corrected form (a comment-only conflict, take g56's).

## 5. The VOID-FAMILY map (the bible)

The complete void-bag/void-vault family, C# → port, owner per row:

| Mechanic | C# anchor | Port | Owner / status |
| --- | --- | --- | --- |
| The items 4131 Void Lens / 5325 Closed Void Bag (shoot 734, useStyle 1, 28/28, Item130) | `Item.cs:35875-35888` | itemdata rows + the generic ItemCheck_Shoot | base (S1); NO bag-open spawn-suppression exists (g56 item-1 census) |
| The held-item right-click swap 4131↔5325 | `Player.cs:31560-31565` (gate `:31528-31534`, sound 7) | `world_sim/editing.rs` tile-interact dispatch + `hovering_lens` probe | **g56** `b7a9cdcc0` |
| The slot-context swap cycle + SwapEquip ladder | `ItemSlot.cs:1668-1697` | `app/clicks.rs` ctx-32 right-click arms | **g56** `6727fb9ad` |
| Lens projectile 734 (row, AI_154, frames, fade, cull) | `Projectile.cs:7726-7735`/`:52154-52196`, `Main.cs:8504` | `projectile/data.rs` + `projectile.rs` 154 arm | **g56** `273814c51` |
| Lens draw + light | `VoidLensHelper.cs:19-24`/`:77-104`/`:34-38` | `render/pylons.rs::void_lens_sprites_at`/`draw_void_lens_projectile` (shared with the S1-G-1 wave-D tile pass) | **g56** `273814c51` (tile body S1-G-1) |
| Lens open/close interaction (draw-time) | `Main.cs:37673-37745`, `:27924` | `void_lens.rs::try_interacting`/`update_selection` | **g56** `273814c51` |
| Lens tracker (ProjectileKey pack, death/range close) | `Player.cs:31019-31097`, `ProjectileKey.cs:27-32` | `void_lens.rs::LensTracker`/`tick_tracker` + `chest.rs::tick_open` | **g56** `273814c51` |
| `lastMouseInterface` latch | `Player.cs:1454`, `Main.cs:61811-61812` | `App.last_mouse_interface` + `mouse_interface_now` | **g56** `273814c51` |
| Tile vault 491 placement/kill/open | `TileObjectData` Style3x3 `:3797-3800`, `Player.cs:33127-33139` | tileobject registry + `interact_at`/`toggle_open` | base (M3-STORE-1); the lock guard **g56** `85711f19e` |
| The banks (piggy/safe/forge/vault, 40 slots) | `Player.cs:1580-1591`, `Chest.cs:178-189` | `bank.rs::BankStore`/`bank_for_chest` | base (M3-STORE-1) |
| `disableVoidBag` net-lock (MP-only) | `Player.cs:3100`/`:23123-23133`, `QuickStacking.cs:293`, `MessageBuffer.cs:434-437` | `Player.disable_void_bag` + `lock_net_slot`/`clear_void_bag_lock_on_echo` + the two open guards | **g56** `85711f19e` (SP-inert, no producer) |
| Auto-pickup routing (GetItem void arm, per-tick 4131 refresh, occupied-then-empty fills) | `Player.cs:39395-39398`/`:24550-24553`/`:39402-39423` | `world_sim/items.rs` + `bank.rs::void_vault_take` | base (S1-P-i43/i47/i50); g56 census `cadec8ef8`; e19 disjoint (§2) |
| Cofveve favorite probe | `Player.cs:39199-39229` (`:39212`) | `item.rs::item_space` 7th param | **g56** `21d01fa0a` |
| ctx-32 grid + Back2/Back19 frames | `ChestUI.cs:356-359`, `ItemSlot.cs:2218-2221`/`:2155-2158` | `ui/inventory.rs`/`ui/slot.rs`/`assets.rs` | **g56** `6727fb9ad` |
| Vault favorites (Alt-toggle, LootAll park, emptied-cell clear) | `ItemSlot.cs:386-394`/`:611-620`/`:2152`, `ChestUI.cs:447-460` | `Player.vault_favorites` + `app/clicks.rs` | **g56** `6727fb9ad`/`d364a2f4c` |
| Favorites PERSIST | `.plr` `Player.cs:54856-54863` gates 198/255/199 | `.trvault` sidecar (interim) — the fold contract §1 | **g56** `9f6d2df51`; `.plr` shape **f44**; superset **g88** (no vault line) |
| `SetGlowForChest` glow package (clear/writer/decay/tint + Sort hue ramp) | `ItemSlot.cs:480-514`/`:522-544`/`:2305-2326`, `ItemSorting.cs:1579-1593`, `Player.cs:28771` | `ui/glow.rs::GlowState`/`chest_glow_tint` + the sort/open/frame/ChestChangeEvents wires | **g56** `ddea019d1` — keep-one (§4) |
| Void-bag key fallback / useVoidBag scan | `Player.cs:33200-33221` | `chest::use_void_bag` | base (ANALYSIS :11342) |
| Lens dust rolls | `VoidLensHelper.Update` dust half | SKIPPED — the S1-G-1 wave-D RNG-order rule | deliberate (ANALYSIS deviation 4) |
| Smart-cursor/gamepad selection, cursorItemIcon, gravDir mirror | `Main.cs:37686`/`:37699-37701` | unported (selection always 0; noThrow=2 lands) | deliberate (ANALYSIS deviations 2/3/8) |
| Bank CONTENTS persist (all four banks) | `.plr` rows 44-47 | unported — f44's exit-save seam (§1c gate 2) | f44 lane; the favorites ride ahead via `.trvault` |

### The retire summary (the five verdicts)

1. **`.trvault` vs f44/g88**: keep-one TODAY = `.trvault` (the only live
   carrier); f44 owns the `.plr` byte-shape, g88's superset has no vault line.
   **COEXIST through the folds; RETIRE the sidecar write only when
   character-load hydration AND the exit-save both land** (§1c), reader keeps
   the one-release back-read, `.plr`-first restore. No g88 v12 vault line.
2. **e19 auto-pickup**: DISJOINT confirmed (bank.rs empty diff, zero
   void-line overlap in items.rs/item.rs); the 16-box repin receipt recorded
   (§2) — g56 takes e19's canonical repins at merge.
3. **g88 persist-compose**: ZERO-TOUCH verified (§3) — g56's persist delta is
   two pure-insertion hunks; the v11 bodies untouched; both keep at merge.
4. **SetGlowForChest**: KEEP-ONE = g56's `ui/glow.rs` — the only glow port on
   any of the 355 tips (§4); the stale base comment corrected on this branch.
5. **This doc = research/239** per registry-150 (237/238 consumed at the
   tips, the 237 three-way fork flagged for its owners; 239 free everywhere).

Code-only retire: no cargo/tests/rustfmt/graphify run; the one code edit is
the stale `chest.rs:870-871` comment correction (§4). The 27 written-not-run
witnesses stand as recorded in ANALYSIS "The void vault UI (W4-g56)".
