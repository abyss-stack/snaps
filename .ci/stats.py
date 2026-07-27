# /// script
# requires-python = ">=3.11"
# dependencies = ["matplotlib", "click", "structlog"]
# ///

import os
from pathlib import Path
from dataclasses import dataclass
from typing import Dict, List, Set
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
        for p in self.root.rglob("*"):
            if p.is_file() and p.suffix.lower() in exts and p.name != out and not self._skip(p):
                loc = self._parse(p)
                data[str(p.relative_to(self.root))] = Metrics(p.stat().st_size, loc)
        return data

class Chart:
    def __init__(self):
        self.cmap = mcolors.LinearSegmentedColormap.from_list(
            "nord_git", ['#eceff4', '#d8dee9', '#4c566a', '#3b4252', '#2e3440']
        )

        # Строгий монохромный градиент для круговой диаграммы (без розового и синего)
        # Плавно переходит от светлого арктического к глубокому угольному Nord0
        self.pie = mcolors.LinearSegmentedColormap.from_list(
            "nord_pie_mono", ['#eceff4', '#e5e9f0', '#d8dee9', '#4c566a', '#3b4252', '#2e3440']
        )

        # Базовый нейтральный цвет для текста и центральной метки (Nord3)
        self.col = '#4c566a'

    def save(self, data: Dict[str, Metrics], path: str, maxn: int = 12):
        if not data:
            return

        items = sorted(data.items(), key=lambda x: x[1].size, reverse=True)
        labels, sizes, locs = [], [], []
        totals = {"size": 0, "loc": 0, "cnt": 0}

        for i, (lbl, m) in enumerate(items):
            totals["size"] += m.size
            totals["loc"] += m.loc

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

        # Левая часть: Пончик-чарт
        ax = fig.add_subplot(gs[0], aspect="equal", facecolor='none')
        n = len(sizes)
        colors = [self.pie(i / max(n - 1, 1)) for i in range(n)]
        
        ax.pie(sizes, 
               wedgeprops=dict(width=0.3, edgecolor='none'), 
               startangle=100, 
               colors=colors)
        
        ax.text(0, 0, f"Total\n{totals['size']/1024:.1f} KB\n{totals['loc']} LOC",
                ha='center', va='center', fontsize=15, color=self.col, fontfamily='monospace', weight='bold')

        # Правая часть: Список файлов
        ax2 = fig.add_subplot(gs[1], facecolor='none')
        for s in ax2.spines.values():
            s.set_visible(False)
        ax2.set_xticks([])
        ax2.set_yticks([])

        n_items = len(labels)
        step = 0.045                    # чуть плотнее
        total_height = (n_items - 1) * step

        # Жёстко опускаем список, чтобы он сидел на одной высоте с пончиком
        # Верх списка ≈ 0.72, низ ≈ 0.28
        center_y = 0.50
        start_y = center_y + total_height / 2

        y = start_y

        for lbl, sz, lc, color in zip(labels, sizes, locs, colors):
            display_name = lbl if len(lbl) <= 35 else lbl[:32] + "..."
            
            # Имя файла — ярким цветом сегмента
            ax2.text(0.05, y, display_name, 
                    fontfamily='monospace', fontsize=11, 
                    color=color, va='center', weight='bold')
            
            # Размер и LOC — нейтральный серый
            ax2.text(0.55, y, f"{sz/1024:>7.1f} KB | {lc:>6} LOC", 
                    fontfamily='monospace', fontsize=10, 
                    color='#718096', va='center')
            y -= step

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