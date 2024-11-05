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

def run_one_variant(variant, binary, binding, N, M, O, VARS):
    var = "var" if VARS else "novar"
    print(N, M, O, variant)
    os.system(f"/usr/bin/time -f '%E, %M Kbytes' timeout -v 20m {binary} '{lhs}' '{rhs}' '{binding}' ./outputs/{variant}-{N}-{M}-{O}-{var}.csv > ./outputs/{variant}-{N}-{M}-{var}.output 2>&1")
    return found_in_csv(f"./outputs/{variant}-{N}-{M}-{O}-{var}.csv")

os.system("cd egg-rise; cargo b --release")
os.system("cd slotted-rise; cargo b --release")

VARS = True

if len(sys.argv) == 4:
    N = int(sys.argv[1])
    M = int(sys.argv[2])
    O = int(sys.argv[3])
    lhs, rhs = generate(N, M, O, VARS)
    run_one_variant("egg-db", "./egg-rise/target/release/egg-rise", "de-bruijn", N, M, O, VARS)
    run_one_variant("slotted", "./slotted-rise/target/release/slotted-rise", "slot", N, M, O, VARS)
    run_one_variant("slotted-db", "./slotted-rise/target/release/slotted-rise", "de-bruijn", N, M, O, VARS)
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

def may_run_one_variant(failed_before, variant, binary, binding, N, M, O, VARS):
    if failed_before[0] == False:
        found = run_one_variant(variant, binary, binding, N, M, O, VARS)
        if not found:
            failed_before[0] = O

def carry_failure(failed_before_nested, nested_min, failed_before_outer, outer):
    if failed_before_nested[0] == nested_min:
        failed_before_outer[0] = outer

fdb_n = [False]
fs_n = [False]
fsdb_n = [False]
for N in range(1, 4): # 21
    fdb_m = [False]
    fs_m = [False]
    fsdb_m = [False]
    for M in range(1, 4): # 21
        failed_db = [False]
        failed_slotted = [False]
        failed_slotted_db = [False]
        for O in range(0, 4): # 21
            lhs, rhs = generate(N, M, O, VARS)
            # print(N, M, O)
            # print(lhs)
            # print(rhs)
            # print("----")
            may_run_one_variant(failed_db, "egg-db", "./egg-rise/target/release/egg-rise", "de-bruijn", N, M, O, VARS)
            may_run_one_variant(failed_slotted, "slotted", "./slotted-rise/target/release/slotted-rise", "slot", N, M, O, VARS)
            may_run_one_variant(failed_slotted_db, "slotted-db", "./slotted-rise/target/release/slotted-rise", "de-bruijn", N, M, O, VARS)
        carry_failure(failed_db, 0, fdb_m, M)
        carry_failure(failed_slotted, 0, fs_m, M)
        carry_failure(failed_slotted_db, 0, fsdb_m, M)
    carry_failure(fdb_m, 1, fdb_n, N)
    carry_failure(fs_m, 1, fs_n, N)
    carry_failure(fsdb_m, 1, fsdb_n, N)

"""
DeBruijn saturates but fails to prove: λz. λx. ((λy. z) x) x = λz. z
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
