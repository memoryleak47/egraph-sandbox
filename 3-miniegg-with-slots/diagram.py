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

colors = [(8,8,156), (49,163,84), (166,54,3)]
colors2 = [(107,174,214), (116,196,118), (253,141,60)]

div = lambda x: (x[0]/255, x[1]/255, x[2]/255)

for i, (attribute, measurement) in enumerate(encodings1.items()):
    offset = width * multiplier
    if etaexp:
        m2 = encodings2[attribute]
        rects = ax.bar(x + offset, m2, width, bottom=measurement, color=div(colors2[i]))
        ax.bar_label(rects, padding=3)
    rects = ax.bar(x + offset, measurement, width, label=attribute, color=div(colors[i]))
    ax.bar_label(rects, padding=3)
    multiplier += 1

# Add some text for labels, title and custom x-axis tick labels, etc.
ax.set_ylabel('Number of e-nodes (log-scale)')
#ax.set_xlabel('Rewrite Problem')
ax.set_yscale('log')
ax.set_xticks(x + width, problems)
ax.set_ylim(1, 20_000_000)
ax.legend(loc="upper center", ncols=3, bbox_to_anchor=(0.5, 1.2))
plt.axhline(y=8_000_000, color="r", linestyle="-", linewidth=4)

plt.show()
