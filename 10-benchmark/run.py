#!/usr/bin/python3 -B

import os
from gen import generate

os.system("mkdir -p outputs")
os.system("rm outputs/*")
os.system("cd egg-rise; cargo b --release")
os.system("cd slotted-rise; cargo b --release")

for N in range(1, 21):
    for M in range(1, 21):
        lhs, rhs = generate(N, M)

        # print(N, M, "egg-name")
        # os.system(f"/usr/bin/time -f '%E, %M Kbytes' timeout -v 20m ./egg-rise/target/release/egg-rise '{lhs}' '{rhs}' name &> ./outputs/egg-name-{N}-{M}")

        print(N, M, "egg-db")
        os.system(f"/usr/bin/time -f '%E, %M Kbytes' timeout -v 20m ./egg-rise/target/release/egg-rise '{lhs}' '{rhs}' de-bruijn &> ./outputs/egg-db-{N}-{M}")

        print(N, M, "slotted")
        os.system(f"/usr/bin/time -f '%E, %M Kbytes' timeout -v 20m ./slotted-rise/target/release/slotted-rise '{lhs}' '{rhs}' &> ./outputs/slotted-{N}-{M}")
