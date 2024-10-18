#!/usr/bin/python3 -B

import os

FRESH = 0

def fresh_slot():
    global FRESH, SLOTTED
    FRESH += 1

    if SLOTTED:
        return "$" + str(FRESH)
    else:
        return "s" + str(FRESH)

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

N = 3
M = 3

SLOTTED = False
lhs = generate_lhs(N, M)
rhs = generate_rhs(N, M)
os.system(f"cd egg-rise; cargo r --release '{lhs}' '{rhs}' name")

SLOTTED = True
lhs = generate_lhs(N, M)
rhs = generate_rhs(N, M)
os.system(f"cd slotted-rise; cargo r --release '{lhs}' '{rhs}'")
