# /// script
# requires-python = ">=3.11"
# dependencies = ["matplotlib", "click", "structlog"]
# ///

import os
from pathlib import Path
from dataclasses import dataclass
from typing import Dict, List, Set, Tuple
import matplotlib.pyplot as plt
import matplotlib.colors as mcolors
import matplotlib.gridspec as gridspec
import click
import structlog

log = structlog.get_logger()

@dataclass
class Metrics:
    size: int
    loc: int
    markers: Dict[str, int]
    toxicity: float

class Analyzer:
    def __init__(self, root: Path):
        self.root = root.resolve()
        self.ignore = self._load_ignore()

    def _load_ignore(self) -> List[str]:
        f = self.root / ".statsignore"
        return [l.strip() for l in f.read_text().splitlines() if l.strip() and not l.startswith("#")] if f.exists() else []

    def _skip(self, p: Path) -> bool:
        rel = str(p.relative_to(self.root))
        return any(
            (pat.endswith("/") and (rel.startswith(pat) or f"/{pat}" in rel)) or pat in rel or p.name == pat
            for pat in self.ignore
        )

    @staticmethod
    def _parse(path: Path) -> Tuple[int, Dict[str, int]]:
        m = {"unwrap": 0, "expect": 0, "unsafe": 0}
        loc = 0
        try:
            with open(path, errors="ignore") as f:
                for line in f:
                    loc += 1
                    s = line.strip().lower().replace(" ", "")
                    for marker in m:
                        if f"//{marker}" in s:
                            m[marker] += 1
        except Exception:
            pass
        return loc, m

    def scan(self, exts: Set[str], out: str) -> Dict[str, Metrics]:
        data = {}
        for p in self.root.rglob("*"):
            if p.is_file() and p.suffix.lower() in exts and p.name != out and not self._skip(p):
                loc, m = self._parse(p)
                tox = ((m["unwrap"] * 1.5 + m["expect"] * 1.0 + m["unsafe"] * 2.0) / loc * 100) if loc else 0
                data[str(p.relative_to(self.root))] = Metrics(p.stat().st_size, loc, m, tox)
        return data

class Chart:
    def __init__(self):
        self.cmap = mcolors.LinearSegmentedColormap.from_list("c", ['#84a0c6', '#526685', '#2a3147', '#161821'])
        self.pie = mcolors.LinearSegmentedColormap.from_list("p", ['#4a5568', '#718096', '#a0aec0', '#cbd5e0', '#4eb3d3', '#3182ce'])
        self.col = '#718096'

    def save(self, data: Dict[str, Metrics], path: str, maxn: int = 12):
        if not data:
            return

        items = sorted(data.items(), key=lambda x: x[1].size, reverse=True)
        labels, sizes, locs, toxs = [], [], [], []
        totals = {"size": 0, "loc": 0, "unwrap": 0, "expect": 0, "unsafe": 0, "tox": 0, "cnt": 0}

        for i, (lbl, m) in enumerate(items):
            totals["size"] += m.size
            totals["loc"] += m.loc
            totals["tox"] += m.toxicity
            for k in ["unwrap", "expect", "unsafe"]:
                totals[k] += m.markers[k]

            if i < maxn:
                labels.append(lbl)
                sizes.append(m.size)
                locs.append(m.loc)
                toxs.append(m.toxicity)
            else:
                totals["cnt"] += 1

        if totals["cnt"]:
            labels.append(f"others ({totals['cnt']} files)")
            sizes.append(sum(m.size for _, m in items[maxn:]))
            locs.append(sum(m.loc for _, m in items[maxn:]))
            toxs.append(sum(m.toxicity for _, m in items[maxn:]) / totals["cnt"])

        fig = plt.figure(figsize=(14, 6), facecolor='none')
        gs = gridspec.GridSpec(1, 2, width_ratios=[0.85, 1.15], wspace=0.1)

        # Левая часть: Пончик-чарт
        ax = fig.add_subplot(gs[0], aspect="equal", facecolor='none')
        colors = [self.pie(i / (len(sizes) - 1 or 1)) for i in range(len(sizes))]
        ax.pie(sizes, wedgeprops=dict(width=0.3, edgecolor='none'), startangle=100, colors=colors)
        ax.text(0, 0, f"Total\n{totals['size']/1024:.1f} KB\n{totals['loc']} LOC",
                ha='center', va='center', fontsize=15, color=self.col, fontfamily='monospace', weight='bold')

        # Правая часть: Спецификация и метрики
        ax2 = fig.add_subplot(gs[1], facecolor='none')
        for s in ax2.spines.values():
            s.set_visible(False)
        ax2.set_xticks([])
        ax2.set_yticks([])

        y = 0.95
        for lbl, sz, lc, tox in zip(labels, sizes, locs, toxs):
            c = self.cmap(min(tox / 10, 1))
            ax2.text(0.02, y, f"{lbl:<30}", fontfamily='monospace', fontsize=12, color=c, va='top')
            ax2.text(0.52, y, f"{sz/1024:>7.1f} KB | {lc:>6} LOC", fontfamily='monospace', fontsize=12, color='#4a5568', va='top')
            y -= 0.048

        y -= 0.04
        ax2.text(0.02, y, "TOXICITY MAP", fontsize=11, fontfamily='monospace', color=self.col, weight='bold', va='top')
        cax = ax2.inset_axes([0.02, y - 0.055, 0.82, 0.025])
        cb = fig.colorbar(plt.cm.ScalarMappable(norm=mcolors.Normalize(0, 10), cmap=self.cmap), cax=cax, orientation='horizontal')
        cb.outline.set_visible(False)
        cb.set_ticks([])

        y -= 0.10
        for marker in ["unwrap", "expect", "unsafe"]:
            ax2.text(
                0.02, y, f"{marker.upper()}: {totals[marker]}",
                fontsize=13, fontfamily='monospace', color=self.col, weight='bold', va='top'
            )
            y -= 0.045
                
        plt.subplots_adjust(left=0.03, right=0.97, top=0.95, bottom=0.05)
        plt.savefig(path, dpi=150, bbox_inches='tight', transparent=True)
        plt.close()

@click.command()
@click.option('-p', '--path', default='.')
@click.option('-t', '--target', default='.py,.rs')
@click.option('-o', '--output', default='disk_analysis.png')
def main(path, target, output):
    structlog.configure(processors=[structlog.processors.TimeStamper(fmt="iso"), structlog.processors.KeyValueRenderer()])
    exts = {e.strip().lower() if e.startswith('.') else f".{e.strip().lower()}" for e in target.split(',')}
    data = Analyzer(Path(path)).scan(exts, output)
    Chart().save(data, str(Path(os.getcwd()) / output))

if __name__ == "__main__":
    main()
