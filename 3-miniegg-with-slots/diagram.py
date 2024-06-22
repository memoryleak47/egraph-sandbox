#!/usr/bin/python3 -B

import matplotlib.pyplot as plt
import numpy as np

problems = ("Reduction", "Fission", "Binomial")
encodings = {
    'Named': (335, 1811085, 1001856),
    'DeBruijn': (574, 20820, 33177),
    'Slotted': (299, 184, 19132),
}

x = np.arange(len(problems))  # the label locations
width = 0.25  # the width of the bars
multiplier = 0

fig, ax = plt.subplots(layout='constrained')

for attribute, measurement in encodings.items():
    offset = width * multiplier
    rects = ax.bar(x + offset, measurement, width, label=attribute)
    ax.bar_label(rects, padding=3)
    multiplier += 1

# Add some text for labels, title and custom x-axis tick labels, etc.
ax.set_ylabel('Length (mm)')
ax.set_yscale('log')
ax.set_title('Number of e-nodes by Rewrite Problems')
ax.set_xticks(x + width, problems)
# ax.set_ylim(40000)
ax.legend(loc='upper left', ncols=3)

plt.show()
