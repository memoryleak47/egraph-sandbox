#!/usr/bin/python3 -B

import os
from gen import generate

os.system("mkdir -p outputs")
os.system("cd egg-rise; cargo b --release")
os.system("cd slotted-rise; cargo b --release")

"""
# much better for slotted.
lhs = "(lam $v1 (lam $v2 (lam $v3 (lam $v4 (lam $v5 (app map (lam $x11 (app (var $v5) (app (var $v4) (app (var $v3) (app (var $v2) (app (var $v1) (var $x11)))))))))))))";
rhs = "(lam $v1 (lam $v2 (lam $v3 (lam $v4 (lam $v5 (lam $x7 (app (app map (lam $x6 (app (var $v5) (app (var $v4) (app (var $v3) (var $x6)))))) (app (app map (lam $x4 (app (var $v2) (app (var $v1) (var $x4))))) (var $x7)))))))))";

# complete toss-up between egg-db & slotted.
lhs = "(app map (lam $x11 (app v5 (app v4 (app v3 (app v2 (app v1 (var $x11))))))))";
rhs = "(lam $x7 (app (app map (lam $x6 (app v5 (app v4 (app v3 (var $x6)))))) (app (app map (lam $x4 (app v2 (app v1 (var $x4))))) (var $x7))))";

# complete-toss up (II)
lhs = "(app map (lam $42 (app f5 (app f4 (app f3 (app f2 (app f1 (var $42))))))))"
rhs = "(lam $1 (app (app map (lam $42 (app f5 (app f4 (app f3 (var $42)))))) (app (app map (lam $42 (app f2 (app f1 (var $42))))) (var $1))))"
"""

GRID = []
for N in range(1, 21):
    for M in range(1, 21):
        GRID.append((N, M))

GRID = sorted(GRID, key=lambda xy: xy[0]+xy[1]*1.0001)
VARS = True

for (N, M) in GRID:
    lhs, rhs = generate(N, M, VARS)
    var = "var" if VARS else "novar"

    # print(N, M, "egg-name")
    # os.system(f"/usr/bin/time -f '%E, %M Kbytes' timeout -v 20m ./egg-rise/target/release/egg-rise '{lhs}' '{rhs}' name &> ./outputs/egg-name-{N}-{M}")

    print(N, M, "egg-db")
    os.system(f"/usr/bin/time -f '%E, %M Kbytes' timeout -v 20m ./egg-rise/target/release/egg-rise '{lhs}' '{rhs}' de-bruijn &> ./outputs/egg-db-{N}-{M}-{var}")

    print(N, M, "slotted")
    os.system(f"/usr/bin/time -f '%E, %M Kbytes' timeout -v 20m ./slotted-rise/target/release/slotted-rise '{lhs}' '{rhs}' &> ./outputs/slotted-{N}-{M}-{var}")
