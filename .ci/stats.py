#!/usr/bin/env -S uv run
# /// script
# requires-python = "==3.12.*"
# dependencies = [
#     "matplotlib==3.11.1",
#     "click==8.4.2",
#     "structlog==26.1.0",
# ]
# ///

import os
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, List, Set

import click
import matplotlib.colors as mcolors
import matplotlib.gridspec as gridspec
import matplotlib.pyplot as plt
import structlog

log = structlog.get_logger()

@dataclass
class Metrics:
    size: int
    loc: int

class Analyzer:
    def __init__(self, root: Path):
        self.root = root.resolve()

    @staticmethod
    def _parse(path: Path) -> int:
        loc = 0
        try:
            with open(path, errors="ignore") as f:
                for _ in f:
                    loc += 1
        except Exception:
            pass
        return loc

    def scan(self, exts: Set[str], out: str) -> Dict[str, Metrics]:
        data = {}
        ignored_parts = {"target", ".git"}

        for p in self.root.rglob("*"):
            if p.is_file() and p.suffix.lower() in exts and p.name != out:
                if any(part in p.parts for part in ignored_parts):
                    continue
                    
                loc = self._parse(p)
                data[str(p.relative_to(self.root))] = Metrics(p.stat().st_size, loc)
        return data


class Chart:
    def __init__(self):
        self.pie_gradient = mcolors.LinearSegmentedColormap.from_list(
            "ef_pie_dark_to_light", ['#2d353b', '#4b564c', '#5c6a5e', '#7a8478', '#8da189', '#a7b09e']
        )
        self.col = '#434f46'

    def save(self, data: Dict[str, Metrics], path: str, maxn: int = 12):
        if not data: 
            return

        items = sorted(data.items(), key=lambda x: x[1].size, reverse=True)
        labels, sizes, locs = [], [], []
        totals = {"size": sum(m.size for _, m in items), "loc": sum(m.loc for _, m in items), "cnt": 0}

        for i, (lbl, m) in enumerate(items):
            if i < maxn:
                labels.append(lbl)
                sizes.append(m.size)
                locs.append(m.loc)
            else:
                totals["cnt"] += 1

        if totals["cnt"]:
            labels.append(f"others ({totals['cnt']} files)")
            sizes.append(sum(m.size for _, m in items[maxn:]))
            locs.append(sum(m.loc for _, m in items[maxn:]))

        fig = plt.figure(figsize=(14, 6), facecolor='none')
        gs = gridspec.GridSpec(1, 2, width_ratios=[0.85, 1.15], wspace=0.1)

        ax = fig.add_subplot(gs[0], aspect="equal", facecolor='none')
        n = len(sizes)
        pie_colors = [self.pie_gradient(i / max(n - 1, 1)) for i in range(n)]
        
        ax.pie(sizes, wedgeprops=dict(width=0.3, edgecolor='#4b564c', linewidth=0.8), startangle=100, colors=pie_colors)
        ax.text(0, 0, f"Total\n{totals['size']/1024:.1f} KB\n{totals['loc']} LOC", ha='center', va='center', fontsize=15, color=self.col, fontfamily='monospace', weight='bold')

        ax2 = fig.add_subplot(gs[1], facecolor='none')
        
        ax2.set_xlim(0, 1)
        ax2.set_ylim(0, 1)
        
        ax2.get_xaxis().set_visible(False)
        ax2.get_yaxis().set_visible(False)
        for s in ax2.spines.values(): 
            s.set_visible(False)

        y = 0.85 
        y_step = 0.055

        ax2.text(0.05, y + 0.055, "FILE PATH", fontfamily='monospace', fontsize=10, color=self.col, weight='bold')
        ax2.text(0.55, y + 0.055, "      SIZE |    METRICS", fontfamily='monospace', fontsize=10, color=self.col, weight='bold')
        ax2.plot([0.05, 0.95], [y + 0.035, y + 0.035], color='#7a8478', linewidth=1, alpha=0.6)

        for lbl, sz, lc, color in zip(labels, sizes, locs, pie_colors):
            display_name = lbl if len(lbl) <= 35 else lbl[:32] + "..."
            ax2.text(0.05, y, display_name, fontfamily='monospace', fontsize=11, color=color, va='center', weight='bold')
            ax2.text(0.55, y, f"{sz/1024:>7.1f} KB | {lc:>6} LOC", fontfamily='monospace', fontsize=10, color='#7a8478', va='center')
            y -= y_step

        plt.subplots_adjust(left=0.03, right=0.97, top=0.95, bottom=0.05)
        plt.savefig(path, dpi=150, bbox_inches='tight', transparent=True)
        plt.close()

@click.command()
@click.option('-p', '--path', default='.')
@click.option('-t', '--target', default='.py,.rs')
@click.option('-o', '--output', default='codebase_size.png')
def main(path, target, output):
    structlog.configure(processors=[structlog.processors.TimeStamper(fmt="iso"), structlog.processors.KeyValueRenderer()])
    exts = {e.strip().lower() if e.startswith('.') else f".{e.strip().lower()}" for e in target.split(',')}
    data = Analyzer(Path(path)).scan(exts, output)
    Chart().save(data, str(Path(os.getcwd()) / output))

if __name__ == "__main__":
    main()
