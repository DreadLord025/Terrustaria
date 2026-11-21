# W4-d120 kind claim (2026-11-21, code-only wave)

Branch `task/W4-d120-town-chatter` claims the small SetDefaults identity
rows + GetChat arms for the UNCLAIMED town kinds the chatter ladder needs
(census: `git rev-list a725acb0..origin/<lane>` — d36/d39/d41/d42/d43 all
0 ahead at claim time; the claim is this doc + the branch push).

| vanilla | kind | protocol id | SetDefaults cite | GetChat cite |
| --- | --- | --- | --- | --- |
| 142 | SantaClaus | 241 | NPC.cs:10791-10804 | NPC.cs:94279-94301 |
| 160 | Truffle | 242 | NPC.cs:11003-11016 | NPC.cs:94302-94341 |
| 178 | Steampunker | 243 | NPC.cs:11252-11265 | NPC.cs:94342-94382 |
| 209 | Cyborg | 244 | NPC.cs:11682-11695 | NPC.cs:94457-94501 |
| 579 | BartenderUnconscious | 245 | NPC.cs:16554-16568 | NPC.cs:94780-94783 |

NOT claimed here (the sibling lanes own them — d120 ports only their
GetChat arms, kind-agnostic until the merge):

- OldMan 37 — d34 phase 1 (30bfb41b, protocol 268).
- Pirate 229 — d37 (750d8e44, protocol 361).
- SkeletonMerchant 453 — d40 batch 10 (protocol 453 vanilla).
- TownCat 637 / TownDog 638 / TownBunny 656 / town slimes 670/678-684 —
  d43 enum rows + f43 spawn arms (research/64 §Port-gaps-2).

Protocol band 241-245: free in to_u16/from_u16/render_key on main AND on
every pushed sibling (d34 266-275, d37 354-399, d35 429-461, d40
430-488 all clear it). cs_id/net_id keep the vanilla types. Sheets
deferred (the ~73-px atlas headroom, the wyvern/d35 precedent).
AI: the town-wander stand-in (update_guide) for 142/160/178/209; the
bound still-stand for 579 (aiStyle 0). Deviations: docs/ANALYSIS.md
"W4-d120 kind census + the remaining GetChat arms".

## Final-phase addendum (2026-11-21, the d36 collision)

d36 pushed its band claim (65e481f96, 06:03 +0200 — an EMPTY commit)
AFTER this claim (05:48). Its 141-210 band covers 142/160/178/209.
KEEP-ONE: this lane's kinds are the landed truth (claim-first + code
pushed 2891b0bd8/e4e400629); d36 skips the four types. The
research/108 reserved-hole tension (237-249 "cs-owned") + the
renumber fallback are recorded in docs/ANALYSIS.md "W4-d120 final
census" — the d37c matrix resolves the protocol ids, the KINDS stay
here either way.
