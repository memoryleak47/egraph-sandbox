import matplotlib.pyplot as plt
import matplotlib.ticker as tkr
import numpy as np
import pandas as pd
import math
import sys
import os

workdir = sys.argv[1]

def lastIterationFromCSV(path, columns):
  data = pd.read_csv(path, names=[ n for n, _ in columns.items() ], skipinitialspace=True)
  data = data.astype(columns)
  lastrow = data.iloc[data["iteration_number"].idxmax()]
  return lastrow

def summaryFromCSV(path):
  lastrow = None
  if "slotted" in path:
    columns = {
      'iteration_number':'int',
      'physical_memory':'int',
      'virtual_memory':'int',
      'e-nodes':'int',
      'e-nodes (computed)':'int',
      'e-classes':'int',
      'total_time':'float',
      'found':'bool'
    }
    lastrow = lastIterationFromCSV(path, columns)
  else:
    columns = {
      'iteration_number':'int',
      'physical_memory':'int',
      'virtual_memory':'int',
      'e-nodes':'int',
      'e-classes':'int',
      'applied_rules':'int',
      'total_time':'float',
      'hook_time':'float',
      'search_time':'float',
      'apply_time':'float',
      'rebuild_time':'float',
      'found':'bool'
    }
    lastrow = lastIterationFromCSV(path, columns)
  lastrow = lastrow[["iteration_number", "physical_memory", "virtual_memory", "e-nodes", "e-classes", "total_time", "found"]]
  return lastrow

rows = []
for entry in os.scandir(workdir):
  (base, ext) = entry.name.split(".")
  (impl, n, m, variant) = base.rsplit("-", 3)
  row_data = summaryFromCSV(entry.path)
  rows.append({
    "impl": impl, "n": int(n), "m": int(m),
    "variant": variant,
    "iteration_number": row_data["iteration_number"],
    "physical_memory": row_data["physical_memory"],
    "virtual_memory": row_data["virtual_memory"],
    "e-nodes": row_data["e-nodes"],
    "e-classes": row_data["e-classes"],
    "total_time": row_data["total_time"],
    "found": row_data["found"]
  })

data = pd.DataFrame(rows, columns=["impl", "n", "m", "variant", "iteration_number", "physical_memory", "virtual_memory", "e-nodes", "e-classes", "total_time", "found"])

def bytes_fmt_func(x, pos):
  s = '{} GB'.format(x / 1e9)
  return s

def nodes_fmt_func(x, pos):
  s = '{} M'.format(x / 1e6)
  return s

def sec_fmt_func(x, pos):
  s = '{} s'.format(x)
  return s

zcol = "total_time"
zfmt = None
if zcol == "virtual_memory":
  zfmt = tkr.FuncFormatter(bytes_fmt_func)
elif zcol == "e-nodes":
  zfmt = tkr.FuncFormatter(nodes_fmt_func)
elif zcol == "total_time":
  # NOTE: this is the time of the last iteration, not of entire process
  zfmt = tkr.FuncFormatter(sec_fmt_func)
else:
  raise Exception("???")

impls = data["impl"].unique()
variants = data["variant"].unique()
fig, axs = plt.subplots(len(impls), len(variants), squeeze=False, subplot_kw={"projection": "3d"})
for impl_i, impl in enumerate(impls):
  for variant_i, variant in enumerate(variants):
    ax = axs[impl_i][variant_i]
    width = depth = height = 1
    ldata = data[(data["impl"] == impl) & (data["variant"] == variant)]
    lns = sorted(ldata["n"].unique())
    lms = sorted(ldata["m"].unique())
    x, y = np.meshgrid(lns, lms)
    def ldata_map_or(n, m, col, f, default):
      row = ldata[
        (ldata["n"] == n) &
        (ldata["m"] == m)
      ]
      if len(row) == 0:
        return default
      elif len(row) == 1:
        return f(row[col].values[0])
      else:
        print(row)
        raise Exception("expected one row or less")
    z = np.array([[ldata_map_or(n, m, zcol, lambda x: x, 0) for n in lns] for m in lms])
    colors = np.array([[ldata_map_or(n, m, "found", lambda b: "green" if b else "red", "black") for n in lns] for m in lms])
    ax.plot_surface(x, y, z, facecolors=colors)
    ax.set_title("{}, {}".format(impl, variant))
    ax.set_xlabel("N (#maps)")
    ax.set_ylabel("M (#funcs / 2)")
    ax.set_zlabel(zcol)
    ax.set_zlim(z.min(), z.max())
    #ax.zaxis.set_major_locator(LinearLocator(10))
    ax.zaxis.set_major_formatter(zfmt)

plt.show()
