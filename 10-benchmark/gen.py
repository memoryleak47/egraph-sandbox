#!/usr/bin/python3 -B

import os

def generate(n, m, VARS):
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

    # f1 ° ... ° fm
    def chained_fns(it):
        fresh = fresh_slot()
        it = [var_wrapper(f"fn{x}") for x in it]

        if len(it) == 1:
            return it[0]

        out = f"(var {fresh})"
        for i in it:
            out = f"(app {i} {out})"
        return f"(lam {fresh} {out})"

    def nested_maps(n, arg):
        out = arg;
        for _ in range(n):
            out = map_(out)
        return out

    # N = number of nested maps.
    # M = half amount of the chained functions.
    def generate_lhs(n, m):
        out = chained_fns(range(1, 2*m+1))
        out = nested_maps(n, out)
        return nest_lams(out, m)

    def generate_rhs(n, m):
        l = nested_maps(n, chained_fns(range(m+1, 2*m+1)))
        r = nested_maps(n, chained_fns(range(1, m+1)))
        out = comp(l, r)
        return nest_lams(out, m)

    def nest_lams(arg, m):
        if VARS:
            l = list(range(1, 2*m+1))
            for i in l[::-1]:
                arg = f"(lam $fn{i} {arg})"
        return arg

    lhs = generate_lhs(n, m)
    rhs = generate_rhs(n, m)
    return lhs, rhs
