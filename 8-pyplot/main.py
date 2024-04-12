from matplotlib import pyplot as plt
import numpy as np

def load(fname):
    l = []
    with open(fname) as f:
        for x in f:
            x = int(x.strip())
            l.append(x)
    if "baseline" in fname:
        l = l[:20]
    return l

slotted = load("slotted.data")
baseline = load("baseline.data")

plt.plot(slotted, label="Slotted")
plt.plot(baseline, label="Baseline")
# plt.yscale('log')
plt.xlabel('# of iterations')
plt.ylabel('# of e-nodes')
plt.legend()
plt.show()
