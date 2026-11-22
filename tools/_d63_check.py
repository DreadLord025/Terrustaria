import io, re
t = io.open(r"C:\Users\DreadLord\Desktop\TR-W4-d63\docs\research\173-drawdata-cases-3.md", encoding="utf-8").read()
m = re.search(r"\*\*d63 owns these 93 ids\*\*.*?```(.*?)```", t, re.S)
claim = m.group(1)
ids = [int(x.group(1)) for x in re.finditer(r"(\d+)@", claim)]
print("claimed:", len(ids), "distinct:", len(set(ids)))
union = [tuple(map(int, ln.split())) for ln in io.open(r"C:\Users\DreadLord\Desktop\TR-W4-d63\tools\_d63_union.txt", encoding="utf-8")]
q = [u[1] for u in union[186:279]]
print("matches census quarter:", ids == q)
# batch coverage: every claimed id appears in an audit table row or verbatim section
missing = [i for i in ids if not re.search(r"\| %d \|" % i, t) and not re.search(r"\*\*%d " % i, t)]
print("ids without a table row or bold header:", missing)
