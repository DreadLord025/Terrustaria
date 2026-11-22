import re, json
doc = open(r'docs\research\113-atlas-repack-plan.md', encoding='utf-8').read()
a = doc.split('## Appendix A')[1].split('## Appendix B')[0]
b = doc.split('## Appendix B -')[1].split('## Appendix B-2')[0]
b2 = doc.split('## Appendix B-2')[1]
pat = re.compile(r'\| `([^`]+)` \|')
na = set(pat.findall(a)); nb = set(pat.findall(b)); nb2 = set(pat.findall(b2))
print('A:', len(na), 'B:', len(nb), 'B-2:', len(nb2))
print('A+B:', len(na | nb), 'A+B+B2:', len(na | nb | nb2))
print('B2 overlap with A+B:', len(nb2 & (na | nb)))
u = json.load(open(r'tools\_d35_union_list.json'))
un = set(r['name'] for r in u)
print('union_list.json:', len(un), '| matches A+B+B2:', un == (na | nb | nb2))
