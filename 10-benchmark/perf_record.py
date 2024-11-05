#!/usr/bin/python3 -B

import gen
import os

os.system("cd slotted-rise; cargo b --release")

l, r = gen.generate(5, 5, 0, True)
print(l)
print(r)
os.system(f"perf record --call-graph dwarf ./slotted-rise/target/release/slotted-rise '{l}' '{r}' slot perf.csv")
