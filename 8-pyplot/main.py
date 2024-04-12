import matplotlib
from matplotlib import pyplot as plt
import numpy as np

matplotlib.rcParams.update({'font.size': 14})

def load(fname):
    l = []
    with open(fname) as f:
        for x in f:
            x = int(x.strip())
            l.append(x)
    return l

slotted = load("slotted.data")
baseline = load("baseline.data")

plt.plot(slotted, label="Slotted")
plt.plot(baseline, label="Baseline")
# plt.yscale('log')
plt.xlabel('# of iterations')
plt.ylabel('# of e-nodes')
plt.legend()
plt.axis([0, 25, 0, 10000])
plt.annotate('☒: 2950801', xy=(19, 10000))
plt.annotate('☑: 706', xy=(22, 706))


plt.show()
plt.savefig("plot.svg")
