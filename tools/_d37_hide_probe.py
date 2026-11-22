# Refine the slot-gap census: read each entry's hide/fold flags from the
# owning tip's bestiary.rs so deliberate Hide folds are not "gaps".
import subprocess, re, json
REPO = r"C:\Users\DreadLord\Desktop\TR-W4-d37"
def show(rev, path):
    return subprocess.run(["git","-C",REPO,"show",f"{rev}:{path}"],
                          capture_output=True).stdout.decode("utf8","replace")
TIPS={"c4":"17a241f6a","c7":"c2cc6ec6","c8":"3705670d","d120":"c5e68a80",
      "d33":"b08acf463","d35":"6a2a155a","d40":"8fe1d6d3","d42":"9b22c091"}
counts=json.load(open(REPO+r"\tools\_d37_receipts_counts.json"))
# sample one entry struct to see fields
src=show("b08acf463","crates/tr-app/src/bestiary.rs")
i=src.index('credit_id: "BlackDragonfly"')
print(src[i-40:i+420])
