#!/usr/bin/python3 -B

import os

def generate(n, m, o, VARS):
    assert (n >= 1)
    assert (m >= 1)
    assert (o >= 0)
    FRESH = [0]

    def var_wrapper(x):
        if VARS:
            return f"(var ${x})"
        else:
            return x

    def fresh_slot():
        FRESH[0] += 1
        return "$" + str(FRESH[0])

    # (a ° b)
    def comp(a, b):
        x = fresh_slot()
        out = f"(lam {x} (app {a} (app {b} (var {x}))))"
        return out

    # (map x)
    def map_(x):
        return f"(app map {x})"

    # f p1 ... pO
    def fn_with_args(f, o):
        out = f
        for i in range(1, o+1):
            v = var_wrapper(f"p{i}")
            out = f"(app {out} {v})"
        return out

    # f1 ° ... ° fm
    def chained_fns(it, o):
        it = [ fn_with_args(var_wrapper(f"fn{i}"), o) for i in it ]
        if len(it) == 1:
            return it[0]

        fresh = fresh_slot()
        out = f"(var {fresh})"
        for f in it:
            out = f"(app {f} {out})"
        return f"(lam {fresh} {out})"

    def nested_maps(n, arg):
        out = arg
        for _ in range(n):
            out = map_(out)
        return out

    # N = number of nested maps.
    # M = half amount of the chained functions.
    # O = number of function parameters
    def generate_lhs(n, m, o):
        out = chained_fns(range(1, 2*m+1), o)
        out = nested_maps(n, out)
        return nest_lams(out, m, o)

    def generate_rhs(n, m, o):
        l = nested_maps(n, chained_fns(range(m+1, 2*m+1), o))
        r = nested_maps(n, chained_fns(range(1, m+1), o))
        out = comp(l, r)
        return nest_lams(out, m, o)

    def nest_lams(arg, m, o):
        if VARS:
            l = list(range(1, o+1))
            for i in l[::-1]:
                arg = f"(lam $p{i} {arg})"
            l = list(range(1, 2*m+1))
            for i in l[::-1]:
                arg = f"(lam $fn{i} {arg})"
        return arg

    lhs = generate_lhs(n, m, o)
    rhs = generate_rhs(n, m, o)
    return lhs, rhs

if __name__ == "__main__":
    import sys
    N = int(sys.argv[1])
    M = int(sys.argv[2])
    O = int(sys.argv[3])
    lhs, rhs = generate(N, M, O, True)
    print(lhs)
    print(rhs)
