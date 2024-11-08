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

# FIXME: maybe pass lhs/rhs as parameters instead of globals?
def run_one_variant(variant, binary, binding, N, M, O, VARS):
    var = "var" if VARS else "novar"
    print(N, M, O, variant)
    os.system(f"/usr/bin/time -f '%E, %M Kbytes' timeout -v 5m {binary} '{lhs}' '{rhs}' '{binding}' ./outputs/{variant}-{N}-{M}-{O}-{var}.csv > ./outputs/{variant}-{N}-{M}-{O}-{var}.output 2>&1")
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

# TODO: ask confirmation
# os.system("rm -r outputs")
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
    if failed_before[0] is None:
        found = run_one_variant(variant, binary, binding, N, M, O, VARS)
        if not found:
            # print(variant, "failed_before:", failed_before[0])
            # print(variant, N, M, O)
            failed_before[0] = O

def carry_failure(failed_before_nested, nested_min, failed_before_outer, outer):
    if failed_before_nested[0] == nested_min:
        print("propagated failure outwards")
        failed_before_outer[0] = outer

def init_failure(nested_min, failed_outer):
    if failed_outer[0] is None:
        return [None]
    else:
        print("propagated failure inwards")
        return [nested_min]

if False:
    # THIS CODE SAMPLES MULTIPLE PLANES FROM THE GRID MORE EFFICIENTLY THAN SPERATE LOOPS, BUT ALSO DOES NOT COLLECT AS MUCH DATA ON FAILURE POINTS
    fdb_n = [None]
    fs_n = [None]
    fsdb_n = [None]
    for N in range(1, 11):
        min_m = 1
        fdb_m = init_failure(min_m, fdb_n)
        fs_m = init_failure(min_m, fs_n)
        fsdb_m = init_failure(min_m, fsdb_n)
        for M in range(min_m, 11):
            min_o = 0
            failed_db = init_failure(min_o, fdb_m)
            failed_slotted = init_failure(min_o, fs_m)
            failed_slotted_db = init_failure(min_o, fsdb_m)
            for O in range(min_o, 11):
                # TWEAK:
                should_do_that_one = (
                    M == 2 or M == 3 or M == 4 or
                    O == 2 or O == 6
                )
                if not should_do_that_one:
                    print("skipping", N, M, O)
                    break
                lhs, rhs = generate(N, M, O, VARS)
                may_run_one_variant(failed_db, "egg-db", "./egg-rise/target/release/egg-rise", "de-bruijn", N, M, O, VARS)
                may_run_one_variant(failed_slotted, "slotted", "./slotted-rise/target/release/slotted-rise", "slot", N, M, O, VARS)
                may_run_one_variant(failed_slotted_db, "slotted-db", "./slotted-rise/target/release/slotted-rise", "de-bruijn", N, M, O, VARS)
            carry_failure(failed_db, min_o, fdb_m, M)
            carry_failure(failed_slotted, min_o, fs_m, M)
            carry_failure(failed_slotted_db, min_o, fsdb_m, M)
        carry_failure(fdb_m, min_m, fdb_n, N)
        carry_failure(fs_m, min_m, fs_n, N)
        carry_failure(fsdb_m, min_m, fsdb_n, N)

fdb = [None]
fs = [None]
fsdb = [None]
for O in range(0, 11):
    N = 2
    M = 2
    lhs, rhs = generate(N, M, O, VARS)
    may_run_one_variant(fdb, "egg-db", "./egg-rise/target/release/egg-rise", "de-bruijn", N, M, O, VARS)
    may_run_one_variant(fs, "slotted", "./slotted-rise/target/release/slotted-rise", "slot", N, M, O, VARS)
    may_run_one_variant(fsdb, "slotted-db", "./slotted-rise/target/release/slotted-rise", "de-bruijn", N, M, O, VARS)

fdb = [None]
fs = [None]
fsdb = [None]
for M in range(1, 11):
    N = 2
    O = 2
    lhs, rhs = generate(N, M, O, VARS)
    may_run_one_variant(fdb, "egg-db", "./egg-rise/target/release/egg-rise", "de-bruijn", N, M, O, VARS)
    may_run_one_variant(fs, "slotted", "./slotted-rise/target/release/slotted-rise", "slot", N, M, O, VARS)
    may_run_one_variant(fsdb, "slotted-db", "./slotted-rise/target/release/slotted-rise", "de-bruijn", N, M, O, VARS)

fdb = [None]
fs = [None]
fsdb = [None]
for N in range(1, 11):
    M = 2
    O = 2
    lhs, rhs = generate(N, M, O, VARS)
    may_run_one_variant(fdb, "egg-db", "./egg-rise/target/release/egg-rise", "de-bruijn", N, M, O, VARS)
    may_run_one_variant(fs, "slotted", "./slotted-rise/target/release/slotted-rise", "slot", N, M, O, VARS)
    may_run_one_variant(fsdb, "slotted-db", "./slotted-rise/target/release/slotted-rise", "de-bruijn", N, M, O, VARS)

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
