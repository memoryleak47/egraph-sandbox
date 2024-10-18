#!/usr/bin/python3 -B

import os
from gen import generate

os.system("cd egg-rise; cargo b --release")
os.system("cd slotted-rise; cargo b --release")

for N in range(1, 20):
    for M in range(1, 20):
        lhs, rhs = generate(N, M, "egg")
        os.system(f"./egg-rise/target/release/egg-rise '{lhs}' '{rhs}' name")

        lhs, rhs = generate(N, M, "slotted")
        os.system(f"./slotted-rise/target/release/slotted-rise '{lhs}' '{rhs}'")
