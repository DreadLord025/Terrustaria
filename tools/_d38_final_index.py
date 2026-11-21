# W4-d38 FINAL - INDEX row 293 flip to FINAL (one-shot editor).
# RENUMBERED 125->293 per registry round 3 (research/150 s7: d125's empress
# ledger 06:07:29 wins key 125; d38 REVOKED -> 293). Historical one-shot,
# already executed at the pre-rename key; refs re-pinned for zero-stale grep.
import io
p = 'docs/research/INDEX.md'
s = io.open(p, encoding='utf-8').read()
old = ('| 293 | d38 crossnote ADDENDUM to research/113 (d35c atlas-repack input): '
       'BAF-alone dies WITHIN the d40b wave (8278 OVER +86), three-wave area bound '
       '8258 > 8192 \u2014 second page REQUIRED; d40\'s 56-sheet count confirmed vs '
       'the pushed tip; the d38 portrait-slot gap CLOSED (181\u2192216) + the '
       'rotation gate consumption witnessed | `293-d38-crossnote-atlas-addendum.md` '
       '| \U0001f4e3 published |')
new = ('| 293 | d38 crossnote ADDENDUM to research/113 \u2014 FINAL: the lazy-load '
       'alignment (d35 e5558b0a9 \u00a710-12: E35 reproduces the 8115 pre-d38 '
       'baseline, the 35+56 NPC-page slice packs 1554 px ONE page), the 35-kind '
       'FindFrame status (34 C# cases + the 328 no-case pin, 35/35 port arms, the '
       '7 town stand-ins the extra6/wave-B/extra7 modules supersede), the bestiary '
       'unlock census (all 35 banner chains agree, Paladin 25 the one non-default; '
       'GAP-1 critter sight-provider pre-existing, GAP-2 rarity stars display-only, '
       'GAP-3 the 328 Hide extra card), the 108-extend cross-ref (281-337 '
       'vanilla-free CONFIRMED, not 429-461) | `293-d38-crossnote-atlas-addendum.md` '
       '| \u2705 FINAL |')
assert old in s, 'INDEX row not found'
s = s.replace(old, new)
io.open(p, 'w', encoding='utf-8', newline='').write(s)
print('INDEX row updated')
