import subprocess, re
src = subprocess.run(["git","show","a725acb0:crates/tr-app/src/assets.rs"],
                     capture_output=True).stdout.decode("utf8","replace")
m = re.search(r"STATIC_SHEET_NAMES[^=]*=\s*&?\[(.*?)\n\];", src, re.S)
raw = m.group(1)
names_raw = re.findall(r'"(NPC_[^"]+)"', raw)
body = re.sub(r"//[^\n]*", "", raw)
names = re.findall(r'"(NPC_[^"]+)"', body)
print("raw incl comments:", len(names_raw), "comment-stripped:", len(names),
      "distinct:", len(set(names)))
# check whether comment-stripping removed any real rows (rows inside /* */?)
print("block comments present:", "/*" in raw)
