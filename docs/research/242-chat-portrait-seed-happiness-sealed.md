# 242 — The chat-portrait system task-seed (W4-g58 retire → f22 lane) + the happiness-family SEALED bible

Date: 2026-11-21. Code-only. Two artifacts: (1) the task-seed for the
portrait-system remainder, addressed to the f22 ui-screens lane (ALIVE at
this round's probe: tip `a2fdf86fb` 10:57:48, retire-4 c25 compose-verify —
the sign-aware panel owner per d120's research/143→237 hand-off); (2) the
happiness-family SEALED declaration — the one-paragraph bible the merger
reads.

## 1. What g58 landed (the claim boundary)

The chat-portrait lane was UNCLAIMED (research/240 §4: d85/g53 carry only
the BESTIARY portraits — `UI/Bestiary/Portrait_Front` + the 22 NPC sheets,
a different system). g58's retire ports the SMALL faithful half on its own
branch (`d2943e4aa`):

- The hop STATE (earlier, `468db5928`): `npc_chat_portrait_frame_counter`
  + `ease_out_bounce` + `portrait_hop_lift` + `do_npc_portrait_hop`
  (`ui/npc_chat.rs`), the five ported trigger sites, the per-drawn-frame
  tick (`app/frame/draw.rs`, gated `talk_npc.is_some()`).
- The DRAW, Detailed arm (`ui/chat_portrait.rs` +
  `ui/chat_portrait_data.rs`): `Main.DrawNPCPortrait` (`Main.cs:40318-
  40447`) — the `Portrait_Window` backdrop + the provider-resolved sheet +
  the `GivenOrTypeName` label, riding the hop lift; the 21-kind
  `NPCPortraits` provider table (`NPCID.cs:5894-6073`) with the shimmer +
  Zoologist-lycanthrope selectors; 44 sheets packed; the `Disabled` gate.

## 2. The task-seed: the portrait-system remainder (f22 or successor)

**Scope — the two dummy-NPC modes + the mode cycle + the unported rows:**

1. **CloseUp** (`Main.cs:40379-40416`): `Main._portraitDummy`
   (`SetDefaults(type)` + `whoAmI`/`GivenName`/`townNpcVariationIndex`
   copies + `FindFrame()`), scale 3, position
   `anchor + (-width/2, 48 + 16)` (pets: `num11 = -20`,
   `num12 = -36 + (96 - frame.Width)/2/6*6`, `:40390-40395`), +
   `NPCPortraitsCloseUpOffsets[type]` (`NPCID.cs:10180+`);
   `IsABestiaryIconDummy`/`IsAPortraitDummy` true; a SCISSORED
   `DrawNPCDirect` — the scissor is `Utils.CenteredRectangle(anchor ·
   UIScale, (96,96) · UIScale)` with `Y -= 50; Height += 50`
   (`:40408-40410`), PointClamp sampler, batch restart around it. The
   port needs a scissor seam in the sprite pipeline (none today) + the
   portrait-dummy NPC draw (the bestiary-icon dummy precedent,
   `IsABestiaryIconDummy`, is the closest existing shape).
2. **FullBodyRetro** (`:40417-40443`): the same dummy at scale 2,
   position `anchor + (-width/2, 48 - height)` +
   `NPCPortraitsFullBodyRetroOffsets[type]` (`NPCID.cs:10343+`), NO
   scissor (the C# reads-then-discards the scissor rect, `:40438`),
   PointClamp, batch restart.
3. **The mode cycle**: `CycleNPCPortraitMode` (`:40297-40316`, the
   four-way 0→1→2→3→0) — the settings ROW already ports
   (`Settings.portrait_preference`, the menu/ingame-options labels read
   `UI.Portraits*`); only the in-chat cycle keybind (if any) is missing.
4. **The unported provider rows**: the pets (638/637/656, the
   `VariantPortraitCondition` six-way selectors), the town slimes
   (670/678-684), the bound/sleeping forms, OldMan 37, Santa 142,
   SkeletonMerchant 453, Steampunker 178, Truffle 160, Pirate 229,
   Tavernkeep 550, Cyborg 209 — each lands with its roster lane; extend
   `PORTRAIT_ROWS` + `packed_sheet_names` (the expansion is shared, one
   edit site).
5. **The shimmer-conditioned providers** read `Main.npc[talkNPC].
   IsShimmerVariant` LIVE (`NPCID.cs:383-391`) — g58's port reads the
   live NPC's `variation_index` (the same truth); the persisted
   `App.shimmered_npcs` bitmap is the save mirror, NOT the draw gate.

**Wiring contract (already in place):** `draw_portrait` returns false for
preferences 1/2 today — the f22 arm replaces that early return with the
dummy draws; the counter tick + the provider resolution + the anchor math
stay g58's. The `Lookups.chat_portraits` map + the
`npc_portrait_background` rect are the sheet feed. The hop witnesses
(`the_portrait_hop_lift_matches_the_bounce` etc.) pin the shared math.

## 3. The happiness-family bible — SEALED

**The happiness family is CLOSED for porting (W4-g58 canonical latch,
f37 price math, d122 shops).** The one canonical price path is the
TALK-OPEN LATCH: `Player.SetTalkNPC` (`Player.cs:4383-4390`) is the ONLY
write site of `currentShoppingSettings`, built by
`ShopHelper.GetShoppingSettings` → `ProcessMood` (the biome/neighbor/
crowd/homeless judges — g58's `happiness.rs` ports the full tables incl.
the Princess `Main.rand` loves and the ≤0.82 achievement); every price
reader — buy/sell (`Player.GetItemExpectedPrice` `:35382-35388`), reforge
(`Main.cs:41591`), nurse (`:40100`), tax (`:40184`, a DIVIDE), Angler
rarity (`Player.cs:56306`), hair (`:43799`, unported window), the shop
chrome (`:41793`) — reads that latch, never a per-frame re-scan and never
the shop-open capture; f37's `shop.rs` keeps the pure price MATH
(`get_item_expected_price`'s banker's-rounded buyOnce/plain arms, the ÷5
sell payout + sellback memo, `LimitAndRoundMultiplier`'s cents rounding)
with `shop_mood` as the merge-side capture field only (its five stale
readers repoint per research/241's exact hunks); d122's shop/restock lane
owns the stock rows + the medal wallet (`buy_with_currency`) and the
KEEP-winning both-arms savings module (research/142, g58's 142-input
vote: one `draw_savings`, d122's signature; g58's report-lane chrome gate
+ d122's chrome body); the report UI is TEXT in the chat panel (no
slider/neighbor-list widget exists in the C#) and is COMPLETE, the legacy
`DrawNPCChatButtons` is dead code; the portrait hop state + the
Detailed-arm draw are g58's (this family's last open half — the dummy
modes ride §2's seed). No further happiness port work remains on any
lane; the merger composes g58-latch + f37-math + d122-shops per
research/240 §5 + research/241 + research/142.

## 4. Verification record

- f22 tip probe: `git log -1 origin/task/W4-f22-ui-screens-4` →
  `a2fdf86fb` 2026-11-21T10:57:48+02:00 (alive, retire phase).
- f37 tip probe: `908482d52` 02:48:30 (8.5 h parked; research/68 winner).
- The 143-claimant scan: 356 `task/W4-*` tips, `git ls-tree
  docs/research/` per tip (research/240's renumber note).
- g58 code: `d2943e4aa` (the Detailed-arm port), witnesses WRITTEN NOT
  RUN (code-only mandate; no cargo/tests/rustfmt).