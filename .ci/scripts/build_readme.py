#!/usr/bin/env -S uv run --script
# /// script
# requires-python = "==3.12.*"
# dependencies = ["matplotlib==3.11.1"]
# ///
import json, sys
import matplotlib.pyplot as plt
import matplotlib.gridspec as gridspec
import matplotlib.colors as mcolors
from types import SimpleNamespace as NS

C = NS(
    gradient=mcolors.LinearSegmentedColormap.from_list("pie", [
        "#2d353b", "#4b564c", "#5c6a5e", "#7a8478", "#8da189", "#a7b09e"
    ]),
    main="#434f46", txt="#7a8478", edge="#4b564c",
)
MAX = 12

def main():
    if sys.stdin.isatty():
        return
    metrics = json.loads(sys.stdin.read())
    if not metrics:
        return

    items = list(metrics.items())
    top, rest = items[:MAX], items[MAX:]
    labels = [lbl for lbl, _ in top]
    sizes  = [d["bytes"] for _, d in top]
    locs   = [d["loc"] for _, d in top]
    if rest:
        labels.append(f"others ({len(rest)} files)")
        sizes.append(sum(d["bytes"] for _, d in rest))
        locs.append(sum(d["loc"] for _, d in rest))

    t_size = sum(sizes)
    t_loc  = sum(locs)
    colors = [C.gradient(i / max(len(sizes) - 1, 1)) for i in range(len(sizes))]

    fig = plt.figure(figsize=(14, 6), facecolor="none")
    gs = gridspec.GridSpec(1, 2, width_ratios=[0.85, 1.15], wspace=0.1)

    ax1 = fig.add_subplot(gs[0], aspect="equal", facecolor="none")
    ax1.pie(sizes, wedgeprops=dict(width=0.3, edgecolor=C.edge, lw=0.8),
            startangle=100, colors=colors)
    ax1.text(0, 0, f"Total\n{t_size/1024:.1f} KB\n{t_loc} LOC",
             ha="center", va="center", fontsize=15, color=C.main,
             fontfamily="monospace", weight="bold")

    ax2 = fig.add_subplot(gs[1], facecolor="none")
    ax2.set_xlim(0, 1); ax2.set_ylim(0, 1); ax2.axis("off")
    y, step = 0.85, 0.055
    ax2.text(0.05, y + 0.055, "FILE PATH", fontfamily="monospace",
             fontsize=10, color=C.main, weight="bold")
    ax2.text(0.55, y + 0.055, " SIZE | METRICS", fontfamily="monospace",
             fontsize=10, color=C.main, weight="bold")
    ax2.plot([0.05, 0.95], [y + 0.035, y + 0.035], color=C.txt, lw=1, alpha=0.6)

    for lbl, sz, lc, col in zip(labels, sizes, locs, colors):
        name = lbl if len(lbl) <= 35 else lbl[:32] + "..."
        ax2.text(0.05, y, name, fontfamily="monospace", fontsize=11,
                 color=col, va="center", weight="bold")
        ax2.text(0.55, y, f"{sz/1024:>7.1f} KB | {lc:>6} LOC",
                 fontfamily="monospace", fontsize=10, color=C.txt, va="center")
        y -= step

    plt.subplots_adjust(left=0.03, right=0.97, top=0.95, bottom=0.05)
    plt.savefig("codebase_size.png", dpi=150, bbox_inches="tight", transparent=True)
    plt.close()

if __name__ == "__main__":
    main()
