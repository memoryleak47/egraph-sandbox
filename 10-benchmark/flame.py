#!/usr/bin/python3 -B

import gen
import os

l, r = gen.generate(8, 8)
print(l)
print(r)
os.system(f"cd slotted-rise; cargo flamegraph -- '{l}' '{r}'")
