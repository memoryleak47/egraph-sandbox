#!/usr/bin/python3 -B

import matplotlib.pyplot as plt
import matplotlib.ticker as tkr
import matplotlib.colors as clr
import numpy as np
import pandas as pd
import math
import sys
import os

# call like `python3 plot.py outputs/`
workdir = sys.argv[1]

def lastIterationAndTotalFromCSV(path, columns, columns_from_last_iteration, columns_as_total):
  data = pd.read_csv(path, names=[ n for n, _ in columns.items() ], skipinitialspace=True)
  data = data.astype(columns)
  lastrow = data.iloc[data["iteration_number"].idxmax()]
  lastrow = lastrow[columns_from_last_iteration]
  total = data[columns_as_total].sum()
  return pd.concat([lastrow, total])

def summaryFromCSV(path):
  columns_from_last_iteration = ["iteration_number", "physical_memory", "virtual_memory", "e-nodes", "e-classes", "found"]
  columns_as_total = ["total_time"]
  if "slotted" in path:
    columns = {
      'iteration_number':'int',
      'physical_memory':'int',
      'virtual_memory':'int',
      'e-nodes':'int',
      'e-classes':'int',
      'total_time':'float',
      'found':'bool'
    }
    return lastIterationAndTotalFromCSV(path, columns, columns_from_last_iteration, columns_as_total)
  else:
    columns = {
      'iteration_number':'int',
      'physical_memory':'int',
      'virtual_memory':'int',
      'e-nodes-hashcons':'int',
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
    return lastIterationAndTotalFromCSV(path, columns, columns_from_last_iteration, columns_as_total)

# Collect summaries for all impl-n-m-o-variant.csv files
rows = []
for entry in os.scandir(workdir):
  (base, ext) = entry.name.split(".")
  if ext != "csv":
    continue
  (impl, n, m, o, variant) = base.rsplit("-", 4)
  row_data = summaryFromCSV(entry.path)
  # if m == "2" and o == "2":
  #  print("collected summary:", impl, n, m, o, variant)
  rows.append({
    "impl": impl, "n": int(n), "m": int(m), "o": int(o),
    "variant": variant,
    "iteration_number": row_data["iteration_number"],
    "physical_memory": row_data["physical_memory"],
    "virtual_memory": row_data["virtual_memory"],
    "e-nodes": row_data["e-nodes"],
    "e-classes": row_data["e-classes"],
    "total_time": row_data["total_time"],
    "found": row_data["found"]
  })

data = pd.DataFrame(rows, columns=["impl", "n", "m", "o", "variant", "iteration_number", "physical_memory", "virtual_memory", "e-nodes", "e-classes", "total_time", "found"])

# Plot a 3D surface for every implementation and variant
def plot_3d(zcol, zfmt, fixed_o):
  impls = data["impl"].unique()
  variants = data["variant"].unique()
  fig, axs = plt.subplots(len(impls), len(variants), squeeze=False, subplot_kw={"projection": "3d"})
  for impl_i, impl in enumerate(impls):
    for variant_i, variant in enumerate(variants):
      ax = axs[impl_i][variant_i]
      width = depth = height = 1
      ldata = data[(data["impl"] == impl) & (data["variant"] == variant) & (data["o"] == fixed_o)]
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

def plot_2d_plane_subplots(filename_str, plane_str, data, x_axis, y_axis, y_fmt):
  impls = np.sort(data["impl"].unique())
  variants = np.sort(data["variant"].unique())
  x_min = data[x_axis].min()
  x_max = data[x_axis].max()
  y_min = data[y_axis].min()
  y_max = data[y_axis].max()
  fig, axs = plt.subplots(len(impls), len(variants), squeeze=False, constrained_layout=True, sharex=True, sharey=True)
  for impl_i, impl in enumerate(impls):
    for variant_i, variant in enumerate(variants):
      ax = axs[impl_i][variant_i]
      ldata = data[(data["impl"] == impl) & (data["variant"] == variant)]
      cmap = clr.ListedColormap(['red', 'green'])
      ax.scatter(x_axis, y_axis, c="found", cmap=cmap, norm=None, vmin=False, vmax=True, data=ldata)

      ax.set_title("{}, {}, {}".format(plane_str, impl, variant))
      ax.set_xlim(x_min, x_max)
      ax.set_ylim(y_min, y_max)

      ax.set_ylabel(y_axis)
      ax.yaxis.set_major_formatter(y_fmt)
  # plt.show()
    if x_axis == "n":
      fig.supxlabel("N (#maps)")
    elif x_axis == "m":
      fig.supxlabel("M (#funcs / 2)")
    elif x_axis == "o":
      fig.supxlabel("O (#params)")
    else:
      fig.supxlabel(x_axis)
  plt.savefig(f"plots/{filename_str}-{x_axis}-{y_axis}.pdf")

def plot_2d_plane(filename_str, plane_str, data, x_axis, y_axis, y_fmt):
  impls = np.sort(data["impl"].unique())
  x_min = data[x_axis].min()
  x_max = data[x_axis].max()
  y_min = data[y_axis].min()
  y_max = data[y_axis].max()

  plt.figure(figsize=(3.2, 2.4))
  ax = plt.gca()

  for impl_i, impl in enumerate(impls):
    ldata = data[(data["impl"] == impl) & (data["variant"] == "var")]

    # update value for preemptive timeout
    tmp = ldata["total_time"].case_when([(
      (ldata["found"] == False) & (ldata["virtual_memory"] < 4000000000),
      5*60
    )])
    ldata.loc[tmp.index, "total_time"] = tmp

    # plot lines
    sdata = ldata.sort_values(by=x_axis)
    linestyle = "-"
    if impl == "egg-db":
      linestyle = "--"
    elif impl == "slotted":
      linestyle = ":"
    elif impl == "slotted-db":
      linestyle = "-."
    lines = plt.plot(x_axis, y_axis, data=sdata, label=impl, zorder=1, linestyle=linestyle)

    # plot points
    # cmap = clr.ListedColormap(['red', 'green'])
    # plt.scatter(x_axis, y_axis, c="found", cmap=cmap, norm=None, vmin=False, vmax=True, data=ldata, label=None, zorder=2)
    plt.scatter(x_axis, y_axis, c=lines[0].get_color(), marker="o", data=ldata[ldata["found"] == True], label=None, zorder=2)
    plt.scatter(x_axis, y_axis, c="red", marker="X", data=ldata[ldata["found"] == False], label=None, zorder=2)


  # plt.title("{}".format(plane_str))
  if y_axis == "total_time":
    plt.legend(loc='best')

  if y_axis == "total_time":
    ax.set_ylabel("time")
    #ax.set_ylim(y_min - 1.0, y_max)
  elif y_axis == "virtual_memory":
    ax.set_ylabel("memory")
    # ax.set_ylim(1, y_max)
  else:
    ax.set_ylabel(y_axis)
    #ax.set_ylim(y_min, y_max)
  ax.yaxis.set_major_formatter(y_fmt)

  ax.set_xlim(x_min, x_max)
  if x_axis == "n":
    ax.set_xlabel("N (#maps)")
  elif x_axis == "m":
    ax.set_xlabel("M (#funcs / 2)")
  elif x_axis == "o":
    ax.set_xlabel("number of function parameters")
  else:
    ax.set_xlabel(x_axis)

  plt.tight_layout()
  plt.savefig(f"plots/{filename_str}-{x_axis}-{y_axis}.pdf")
  plt.clf()

# 2D Plots:
# N=2, M=2, plot O
# N=2, plot M, O=2
# plot N, M=2, O=2
def plot_2d_planes(metric, metric_fmt):
  for (filename_str, plane_str, x_axis, filtered_data) in [
    ("n2-m2", "N = 2, M = 2", "o", data[(data["n"] == 2) & (data["m"] == 2)]),
    ("n2-o2", "N = 2, O = 2", "m", data[(data["n"] == 2) & (data["o"] == 2)]),
    ("m2-o2", "M = 2, O = 2", "n", data[(data["m"] == 2) & (data["o"] == 2)])
  ]:
    plot_2d_plane(filename_str, plane_str, filtered_data, x_axis, metric, metric_fmt)

def bytes_fmt_func(x, pos):
  s = '{} GB'.format(round(x / 1e9))
  return s

def nodes_fmt_func(x, pos):
  s = '{} M'.format(x / 1e6)
  return s

def sec_fmt_func(x, pos):
  s = '{} s'.format(round(x))
  return s

def plot_all_in_one():
  impls = np.sort(data["impl"].unique())
  for impl_i, impl in enumerate(impls):
    found_data = data[(data["found"] == True) & (data["impl"] == impl)]
    not_found_data = data[(data["found"] == False) & (data["impl"] == impl)]
    plt.scatter("total_time", "virtual_memory", data=found_data, label=impl, marker="o")
    plt.scatter("total_time", "virtual_memory", data=not_found_data, label=impl, marker="X")

  plt.legend(loc='best')
  ax = plt.gca()
  ax.set_xlim(data["total_time"].min(), data["total_time"].max())
  ax.set_ylim(data["virtual_memory"].min(), data["virtual_memory"].max())

  ax.set_xlabel("time")
  ax.set_ylabel("memory")
  ax.xaxis.set_major_formatter(tkr.FuncFormatter(sec_fmt_func))
  ax.yaxis.set_major_formatter(tkr.FuncFormatter(bytes_fmt_func))
  ax.set_xscale("log")
  ax.set_yscale("log")

  plt.savefig(f"plots/all_in_one.pdf")

print(data[(data["n"] == 2) & (data["m"] == 2)][["impl", "o", "iteration_number", "virtual_memory", "e-nodes", "e-classes", "total_time", "found"]].sort_values(by=["o", "impl"]))
# exit(1)

# Plot for each metric
for metric in ["total_time", "virtual_memory", "e-nodes", "e-classes", "iteration_number"]:
  fmt = None
  if metric == "virtual_memory":
    fmt = tkr.FuncFormatter(bytes_fmt_func)
  elif metric == "e-nodes" or metric == "e-classes":
    fmt = tkr.FuncFormatter(nodes_fmt_func)
  elif metric == "total_time":
    fmt = tkr.FuncFormatter(sec_fmt_func)
  else:
    fmt = tkr.ScalarFormatter()

  plot_2d_planes(metric, fmt)

plot_all_in_one()