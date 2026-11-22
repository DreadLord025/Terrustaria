# Shared rep helper: line-ending aware, typographic-tolerant exact replace.
import re as _re

def make_rep(src_get, src_set):
    def rep(old, new, n=1):
        src = src_get()
        nl = "\r\n" if "\r\n" in src else "\n"
        old = old.replace("\n", nl); new = new.replace("\n", nl)
        # tokenize into escaped literal runs and raw non-ascii chars
        parts = _re.split(r"([\x80-\U0010ffff])", old)
        pat = "".join(_re.escape(p) if i % 2 == 0 else "[^\r\n]" for i, p in enumerate(parts))
        # hyphens tolerate en/em dashes; arrows already wildcarded if non-ascii
        pat = pat.replace(r"\-", "[-\u2013\u2014]").replace("s1", "(?:s1|\u00a71)")
        hits = list(_re.finditer(pat, src))
        assert len(hits) == n, f"count {len(hits)} != {n} for {old[:90]!r}"
        src_set(_re.sub(pat, lambda m: new, src))
    return rep
