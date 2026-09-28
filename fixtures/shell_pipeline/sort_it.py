# Broken: should write sorted ascending integers from nums.txt into sorted.txt
# (one number per line, no trailing blank line required beyond final newline)
with open("nums.txt") as f:
    nums = [line.strip() for line in f if line.strip()]
# BUG: writes unsorted and to wrong file
with open("out.txt", "w") as f:
    f.write("\n".join(nums) + "\n")
