#!/usr/bin/python3 -B

import gen
import os

VARS = False
l, r = gen.generate(5, 5, VARS)
print(l)
print(r)
os.system(f"cd slotted-rise; cargo flamegraph -- '{l}' '{r}' flame.csv")
