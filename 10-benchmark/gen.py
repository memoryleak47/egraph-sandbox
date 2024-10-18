#!/usr/bin/python3 -B

import os

def generate(n, m):
    FRESH = [0]

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
        it = [f"f{x}" for x in it]

        out = it[0]
        for i in it[1:]:
            out = comp(i, out)
        return out

    def nested_maps(n, arg):
        out = arg;
        for _ in range(n):
            out = map_(out)
        return out

    # N = number of nested maps.
    # M = half amount of the chained functions.
    def generate_lhs(n, m):
        l = chained_fns(range(1, m+1))
        r = chained_fns(range(m, 2*m+1))
        out = comp(l, r)
        out = nested_maps(n, out)
        return out

    def generate_rhs(n, m):
        l = nested_maps(n, chained_fns(range(1, m+1)))
        r = nested_maps(n, chained_fns(range(m, 2*m+1)))
        out = comp(l, r)
        return out

    lhs = generate_lhs(n, m)
    rhs = generate_rhs(n, m)
    return lhs, rhs
