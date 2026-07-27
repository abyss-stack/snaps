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
    purity: float  # Индекс чистоты кода в процентах (0.0 - 100.0)

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
    def _parse(path: Path) -> tuple[int, float]:
        loc = 0
        penalty_score = 0.0
        
        try:
            with open(path, errors="ignore") as f:
                for line in f:
                    loc += 1
                    line_upper = line.upper()
                    if "// UNSAFE" in line_upper:
                        penalty_score += 2.0
                    if "// UNWRAP" in line_upper:
                        penalty_score += 1.5
                    if "// EXPECT" in line_upper:
                        penalty_score += 1.0
        except Exception:
            pass
            
        if loc > 0:
            purity = max(0.0, 100.0 - (penalty_score / loc) * 100.0)
        else:
            purity = 100.0
            
        return loc, purity

    def scan(self, exts: Set[str], out: str) -> Dict[str, Metrics]:
        data = {}
        for p in self.root.rglob("*"):
            if p.is_file() and p.suffix.lower() in exts and p.name != out and not self._skip(p):
                loc, purity = self._parse(p)
                data[str(p.relative_to(self.root))] = Metrics(p.stat().st_size, loc, purity)
        return data

class Chart:
    def __init__(self):
        # Монохромный градиент для Git/Heatmap
        self.cmap = mcolors.LinearSegmentedColormap.from_list("ef_git", ['#e4e8dc', '#b8c0ab', '#7a8478', '#4b564c', '#2d353b'])
        
        # Настоящий плавный градиент Everforest от темного к светлому
        self.pie_gradient = mcolors.LinearSegmentedColormap.from_list(
            "ef_pie_dark_to_light", ['#2d353b', '#4b564c', '#5c6a5e', '#7a8478', '#8da189', '#a7b09e']
        )
        
        # Базовый цвет для центрального текста, заголовков и итогов (темный лесной Everforest)
        self.col = '#434f46'

    def save(self, data: Dict[str, Metrics], path: str, maxn: int = 12):
        if not data: return

        # Сортировка по размеру файла из объекта Metrics
        items = sorted(data.items(), key=lambda x: x[1].size, reverse=True)
        labels, sizes, locs, purities = [], [], [], []
        totals = {"size": sum(m.size for _, m in items), "loc": sum(m.loc for _, m in items), "cnt": 0}

        # Считаем средневзвешенную чистоту вообще для всего проекта (total)
        if totals["loc"] > 0:
            total_project_purity = sum(m.purity * m.loc for _, m in items) / totals["loc"]
        else:
            total_project_purity = 100.0

        for i, (lbl, m) in enumerate(items):
            if i < maxn:
                labels.append(lbl)
                sizes.append(m.size)
                locs.append(m.loc)
                purities.append(m.purity)
            else:
                totals["cnt"] += 1

        if totals["cnt"]:
            labels.append(f"others ({totals['cnt']} files)")
            sizes.append(sum(m.size for _, m in items[maxn:]))
            locs.append(sum(m.loc for _, m in items[maxn:]))
            
            remaining = items[maxn:]
            total_rem_loc = sum(m.loc for _, m in remaining)
            if total_rem_loc > 0:
                avg_purity = sum(m.purity * m.loc for _, m in remaining) / total_rem_loc
            else:
                avg_purity = 100.0
            purities.append(avg_purity)

        fig = plt.figure(figsize=(14, 6), facecolor='none')
        gs = gridspec.GridSpec(1, 2, width_ratios=[0.80, 1.20], wspace=0.1)

        # 1. Левая часть: Пончик-чарт
        ax = fig.add_subplot(gs[0], aspect="equal", facecolor='none')
        n = len(sizes)
        
        pie_colors = [self.pie_gradient(i / max(n - 1, 1)) for i in range(n)]
        
        ax.pie(sizes, wedgeprops=dict(width=0.3, edgecolor='#4b564c', linewidth=0.8), startangle=100, colors=pie_colors)
        ax.text(0, 0, f"Total\n{totals['size']/1024:.1f} KB\n{totals['loc']} LOC", ha='center', va='center', fontsize=15, color=self.col, fontfamily='monospace', weight='bold')

        # 2. Правая часть: Список файлов + Header + Footer
        ax2 = fig.add_subplot(gs[1], facecolor='none')
        ax2.set_xlim(0, 1)
        ax2.set_ylim(0, 1)
        ax2.get_xaxis().set_visible(False)
        ax2.get_yaxis().set_visible(False)
        for s in ax2.spines.values(): s.set_visible(False)

        # Учитываем +1 дополнительную строчку под итоги в расчете высоты
        y = 0.50 + ((len(labels) * 0.045) / 2)
        
        # Вывод заголовков колонок
        ax2.text(0.02, y + 0.055, "FILE PATH", fontfamily='monospace', fontsize=10, color=self.col, weight='bold')
        ax2.text(0.48, y + 0.055, "      SIZE |    METRICS | PURITY", fontfamily='monospace', fontsize=10, color=self.col, weight='bold')
        ax2.plot([0.02, 0.98], [y + 0.035, y + 0.035], color='#7a8478', linewidth=1, alpha=0.6)

        # Вывод строк с данными файлов
        for lbl, sz, lc, pur, color in zip(labels, sizes, locs, purities, pie_colors):
            display_name = lbl if len(lbl) <= 32 else lbl[:29] + "..."
            ax2.text(0.02, y, display_name, fontfamily='monospace', fontsize=11, color=color, va='center', weight='bold')
            ax2.text(0.48, y, f"{sz/1024:>7.1f} KB | {lc:>6} LOC | {pur:>5.1f}%", fontfamily='monospace', fontsize=10, color='#7a8478', va='center')
            y -= 0.045

        # --- ДОБАВЛЕНИЕ FOOTER (ИТОГИ И СРЕДНЕЕ) ---
        # Тонкая разделительная линия перед итогами
        ax2.plot([0.02, 0.98], [y + 0.02, y + 0.02], color='#7a8478', linewidth=0.8, linestyle='--', alpha=0.5)
        y -= 0.015
        
        # Вывод итоговой строки цветом заголовка (self.col)
        ax2.text(0.02, y, "CODEBASE", fontfamily='monospace', fontsize=10, color=self.col, weight='bold', va='center')
        ax2.text(0.48, y, f"{totals['size']/1024:>7.1f} KB | {totals['loc']:>6} LOC | {total_project_purity:>5.1f}%", 
                 fontfamily='monospace', fontsize=10, color=self.col, weight='bold', va='center')

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
