# Reads the wrong file / wrong format — should print "Hello, Ada"
with open("names.txt") as f:
    name = f.read().strip().upper()
print(f"Hi {name}")
