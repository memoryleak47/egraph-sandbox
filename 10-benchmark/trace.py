#!/usr/bin/python3 -B

import gen
import os

os.system("cd slotted-rise; cargo b --release --features tracing")

l, r = gen.generate(5, 5, True)
print(l)
print(r)
os.system(f"cd slotted-rise; ./target/release/slotted-rise '{l}' '{r}' /dev/null")
