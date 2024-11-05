#!/usr/bin/python3 -B

import gen
import os

l, r = gen.generate(5, 5, 0, True)
print(l)
print(r)
os.system(f"cd slotted-rise; /usr/bin/time -f '%E, %M Kbytes' cargo run --release -- '{l}' '{r}' slot /dev/null")
