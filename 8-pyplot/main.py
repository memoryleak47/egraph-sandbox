
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

ok_symbol = "✔"
ko_symbol = "✘"
maxY = 10000

plt.plot(slotted, label="Slotted")
slottedMaxY = min(maxY, slotted[-1])
slottedMaxX = np.interp(slottedMaxY, slotted, range(len(slotted)))
plt.text(slottedMaxX, slottedMaxY, ok_symbol, color="green", horizontalalignment='left', verticalalignment='top', fontsize=16)
plt.text(slottedMaxX, slottedMaxY, f"{slotted[-1]}", horizontalalignment='right', verticalalignment='bottom')
plt.plot(baseline, label="Baseline")
baselineMaxY = min(maxY, baseline[-1])
baselineMaxX = np.interp(baselineMaxY, baseline, range(len(baseline)))
plt.text(baselineMaxX, baselineMaxY, ko_symbol, color="red", horizontalalignment='left', verticalalignment='top', fontsize=16)
plt.text(baselineMaxX, baselineMaxY, f"{baseline[-1]}", horizontalalignment='center', verticalalignment='bottom')
# plt.yscale('log')
plt.xlabel('# of iterations')
plt.ylabel('# of e-nodes')
plt.legend()
plt.axis([0, 25, 0, maxY])
#plt.annotate('☒: 2950801', xy=(19, 10000))
#plt.annotate('☑: 706', xy=(22, 706))


# plt.show()
plt.savefig("plot.svg")
