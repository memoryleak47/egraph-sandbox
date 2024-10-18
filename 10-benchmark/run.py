#!/usr/bin/python3 -B

import os
from gen import generate

N, M = 3, 3

lhs, rhs = generate(N, M, "egg")
os.system(f"cd egg-rise; cargo r --release '{lhs}' '{rhs}' name")

lhs, rhs = generate(N, M, "slotted")
os.system(f"cd slotted-rise; cargo r --release '{lhs}' '{rhs}'")
