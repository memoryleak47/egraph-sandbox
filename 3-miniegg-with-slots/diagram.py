#!/usr/bin/python3 -B

import matplotlib.pyplot as plt
import matplotlib
import numpy as np

etaexp=False

plt.rcParams['font.size'] = 28

problems = ("Reduction", "Binomial", "Fission")

oom = 10**10

encodings1 = {
    'Named (egg)': (335, oom, oom),
    'DeBruijn (egg)': (574, 33177, 20820),
    'Slotted': (299, 19132, 184),
}

# with eta-expansion
encodings2 = {
    'Named (egg)': (oom, oom, oom),
    'DeBruijn (egg)': (oom, oom, oom),
    'Slotted': (4304, 96523, 1334),
}

x = np.arange(len(problems))  # the label locations
width = 0.22  # the width of the bars
multiplier = 0

fig, ax = plt.subplots(layout='constrained')

colors = ["#ff7700", "#aa5500", "#aaaa00"]

for i, (attribute, measurement) in enumerate(encodings1.items()):
    offset = width * multiplier
    m2 = encodings2[attribute]
    if etaexp:
        delta = tuple(m2[j] - measurement[j] for j in range(3))
        rects = ax.bar(x + offset, delta, width, bottom=measurement, color=(colors[i], 0.5))
        ax.bar_label(rects, padding=3)
    rects = ax.bar(x + offset, measurement, width, label=attribute, color=colors[i])
    ax.bar_label(rects, padding=3)
    multiplier += 1

# Add some text for labels, title and custom x-axis tick labels, etc.
ax.set_ylabel('Number of e-nodes (log-scale)')
#ax.set_xlabel('Rewrite Problem')
ax.set_yscale('log')
ax.set_xticks(x + width, problems)
ax.set_ylim(1, 20_000_000)
ax.legend(loc="upper center", ncols=3, bbox_to_anchor=(0.5, 1.2))
plt.axhline(y=8_000_000, color="r", linestyle="-", linewidth=8)

plt.show()
