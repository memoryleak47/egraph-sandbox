#!/usr/bin/python3 -B

import sys
import os
from gen import generate

def found_in_csv(path):
    with open(path) as file:
        contents = file.read()
        found = "true" in contents
        print("found:", found)
        return found

def run_one(N, M, VARS):
    lhs, rhs = generate(N, M, VARS)
    var = "var" if VARS else "novar"

    print(N, M, "egg-db")
    os.system(f"/usr/bin/time -f '%E, %M Kbytes' timeout -v 20m ./egg-rise/target/release/egg-rise '{lhs}' '{rhs}' de-bruijn ./outputs/egg-db-{N}-{M}-{var}.csv > ./outputs/egg-db-{N}-{M}-{var}.output 2>&1")
    found_db = found_in_csv(f"./outputs/egg-db-{N}-{M}-{var}.csv")

    print(N, M, "slotted")
    os.system(f"/usr/bin/time -f '%E, %M Kbytes' timeout -v 20m ./slotted-rise/target/release/slotted-rise '{lhs}' '{rhs}' ./outputs/slotted-{N}-{M}-{var}.csv > ./outputs/slotted-{N}-{M}-{var}.output 2>&1")
    found_slotted = found_in_csv(f"./outputs/slotted-{N}-{M}-{var}.csv")

    return found_db and found_slotted

os.system("cd egg-rise; cargo b --release")
os.system("cd slotted-rise; cargo b --release")

if len(sys.argv) == 3:
    run_one(int(sys.argv[1]), int(sys.argv[2]), True)
    exit()

os.system("rm -r outputs")
os.system("mkdir -p outputs")

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
        both_found = run_one(N, M, True)
        if not both_found:
            break

"""
DeBruijn saturates but fails to prove: \z. \x. ((\y. z) x) x = \z. z
DeBruijn does not saturate but can prove: λ x, (λ t y, t) (λ z, x z) = λ x y, x

First combination, not so bad:
(app (lam $a (lam $b (app (app (lam $c (var $a)) (var $b)) (var $b)))) (lam $x (app (lam $t (lam $y (var $t))) (lam $z (app (var $x) (var $z))))))
(λz. λx. ((λy. z) x) x) (λ x, (λ t y, t) (λ z, x z)) =
(λz. z) (λ x y, x) =
(λ x y, x)
= (lam $x (lam $y (var $x)))

Second combination, explosion?
(app (lam $x (app (lam $t (lam $y (var $t))) (lam $z (app (var $x) (var $z))))) (lam $a (lam $b (app (app (lam $c (var $a)) (var $b)) (var $b))))) =
(λ x, (λ t y, t) (λ z, x z)) (λz. λx. ((λy. z) x) x) =
(λ x y, x) (λz. z) =
(λ y, (λz. z))
= (lam $y (lam $z (var $z)))
"""