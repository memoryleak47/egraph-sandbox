#!/usr/bin/python3 -B

import matplotlib.pyplot as plt
import matplotlib
import numpy as np

plt.rcParams['font.size'] = 22

problems = ("Reduction", "Fission", "Binomial")
encodings = {
    'Named': (335, 1811085, 1001856),
    'DeBruijn': (574, 20820, 33177),
    'Slotted': (299, 184, 19132),
}

x = np.arange(len(problems))  # the label locations
width = 0.22  # the width of the bars
multiplier = 0

fig, ax = plt.subplots(layout='constrained')

for attribute, measurement in encodings.items():
    offset = width * multiplier
    rects = ax.bar(x + offset, measurement, width, label=attribute)
    ax.bar_label(rects, padding=3)
    multiplier += 1

# Add some text for labels, title and custom x-axis tick labels, etc.
ax.set_ylabel('Number of e-nodes (log-scale)')
ax.set_xlabel('Rewrite Problem')
ax.set_yscale('log')
ax.set_xticks(x + width, problems)
ax.set_ylim(0, 100000)
ax.legend(loc='upper left')

plt.show()
