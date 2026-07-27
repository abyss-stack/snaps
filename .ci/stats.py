# /// script
# requires-python = ">=3.11"
# dependencies = [
#     "matplotlib",
#     "click",
#     "structlog",
# ]
# ///

import os
from pathlib import Path
import matplotlib.pyplot as plt
import matplotlib.colors as mcolors
import matplotlib.gridspec as gridspec
import click
import structlog

log = structlog.get_logger()

def load_ignore_patterns(root_dir: Path) -> list[str]:
    ignore_file = root_dir / ".statsignore"
    if not ignore_file.exists():
        return []
    with open(ignore_file, "r", encoding="utf-8") as f:
        return [line.strip() for line in f if line.strip() and not line.startswith("#")]

def should_ignore(path: Path, root_dir: Path, patterns: list[str]) -> bool:
    rel_path = str(path.relative_to(root_dir))
    for pattern in patterns:
        if pattern.endswith("/"):
            if rel_path.startswith(pattern) or f"/{pattern}" in rel_path:
                return True
        elif pattern in rel_path or path.name == pattern:
            return True
    return False

def analyze_file_content(path: Path) -> tuple[int, dict[str, int]]:
    lines_count = 0
    markers = {"unwrap": 0, "expect": 0, "unsafe": 0}
    try:
        with open(path, "r", encoding="utf-8", errors="ignore") as f:
            for line in f:
                lines_count += 1
                line_strip = line.strip().lower().replace(" ", "")
                if "//unwrap" in line_strip:
                    markers["unwrap"] += 1
                elif "//expect" in line_strip:
                    markers["expect"] += 1
                elif "//unsafe" in line_strip:
                    markers["unsafe"] += 1
    except Exception:
        pass
    return lines_count, markers

def calculate_toxicity(loc: int, markers: dict) -> float:
    if loc == 0:
        return 0.0
    score = (markers["unwrap"] * 1.5) + (markers["expect"] * 1.0) + (markers["unsafe"] * 2.0)
    return (score / loc) * 100

def scan_files(root_dir: Path, extensions: set[str], output_image: str) -> dict:
    file_data = {}
    root_abs = root_dir.resolve()
    if not root_abs.exists():
        log.error("directory_not_found", path=str(root_abs))
        raise click.Abort()

    patterns = load_ignore_patterns(root_abs)
    for path in root_abs.rglob("*"):
        if path.is_file() and path.suffix.lower() in extensions:
            if path.name == output_image or should_ignore(path, root_abs, patterns):
                continue
            relative_path = path.relative_to(root_abs)
            loc, markers = analyze_file_content(path)
            toxicity = calculate_toxicity(loc, markers)
            file_data[str(relative_path)] = {
                "size": path.stat().st_size,
                "loc": loc,
                "markers": markers,
                "toxicity": toxicity
            }
    return file_data

def save_github_chart(file_data: dict, output_path: str, max_pieces: int = 10):
    if not file_data:
        log.warning("no_files_found")
        return

    sorted_data = sorted(file_data.items(), key=lambda x: x[1]["size"], reverse=True)
    final_labels, final_sizes, final_locs, final_toxicities = [], [], [], []
    total_unwrap = total_expect = total_unsafe = total_loc = 0
    others_size = others_loc = others_count = others_toxicity_sum = 0

    for i, (lbl, metrics) in enumerate(sorted_data):
        total_unwrap += metrics["markers"]["unwrap"]
        total_expect += metrics["markers"]["expect"]
        total_unsafe += metrics["markers"]["unsafe"]
        total_loc += metrics["loc"]
        
        if i < max_pieces:
            final_labels.append(lbl)
            final_sizes.append(metrics["size"])
            final_locs.append(metrics["loc"])
            final_toxicities.append(metrics["toxicity"])
        else:
            others_size += metrics["size"]
            others_loc += metrics["loc"]
            others_toxicity_sum += metrics["toxicity"]
            others_count += 1

    if others_size > 0:
        final_labels.append(f"others ({others_count} files)")
        final_sizes.append(others_size)
        final_locs.append(others_loc)
        final_toxicities.append(others_toxicity_sum / others_count)

    total_kb = sum(final_sizes) / 1024
    
    text_color = '#718096'
    metrics_color = '#4a5568' 
    border_color = '#3d4868' 

    text_cmap = mcolors.LinearSegmentedColormap.from_list("text_toxicity", ['#84a0c6', '#526685', '#2a3147', '#161821'])

    fig = plt.figure(figsize=(14, 7.5), facecolor='none')
    gs = gridspec.GridSpec(1, 2, width_ratios=[1.1, 1.0], wspace=0.15)

    ax_pie = fig.add_subplot(gs[0, 0], aspect="equal")
    ax_pie.set_facecolor('none')
    
    pie_cmap = mcolors.LinearSegmentedColormap.from_list("adaptive_slate", ['#4a5568', '#718096', '#a0aec0', '#cbd5e0', '#4eb3d3', '#3182ce'])
    pie_colors = [pie_cmap(i / (len(final_sizes) - 1 if len(final_sizes) > 1 else 1)) for i in range(len(final_sizes))]

    wedges, _ = ax_pie.pie(final_sizes, wedgeprops=dict(width=0.3, edgecolor='none'), startangle=100, colors=pie_colors)
    ax_pie.text(0, 0, f"Total\n{total_kb:.1f} KB\n{total_loc} LOC", ha='center', va='center', fontsize=10.5, color=text_color, fontfamily='monospace', weight='bold')

    ax_text = fig.add_subplot(gs[0, 1])
    ax_text.set_facecolor('none')
    
    for spine in ax_text.spines.values():
        spine.set_edgecolor(border_color)
        spine.set_linewidth(1)
    
    ax_text.set_xticks([])
    ax_text.set_yticks([])

    pad_left = 0.05
    pad_right = 0.95

    ax_text.text(pad_left, 0.94, "REPOSITORY FILE SIZES", fontsize=10.5, fontfamily='monospace', fontweight='bold', color=text_color, va='top')
    
    y_offset = 0.84
    for lbl, sz, lc, tox in zip(final_labels, final_sizes, final_locs, final_toxicities):
        norm_tox = min(tox / 10.0, 1.0)
        current_name_color = text_cmap(norm_tox)
        
        name_part = f"{lbl:<28}"
        ax_text.text(pad_left, y_offset, name_part, fontfamily='monospace', fontsize=9.5, color=current_name_color, va='top')
        
        metrics_part = f" {sz / 1024:>6.1f} KB | {lc:>5} LOC"
        ax_text.text(0.49, y_offset, metrics_part, fontfamily='monospace', fontsize=9.5, color=metrics_color, va='top')
        
        y_offset -= 0.055

    y_offset -= 0.03
    ax_text.text(pad_left, y_offset, "TOXICITY GRADIENT MAP\n[ pristine -> debt / unsafe ]", fontsize=9.5, fontfamily='monospace', color=text_color, weight='bold', va='top')
    
    cax = ax_text.inset_axes([pad_left, y_offset - 0.09, pad_right - pad_left, 0.035])
    norm = mcolors.Normalize(vmin=0, vmax=10)
    cb = fig.colorbar(plt.cm.ScalarMappable(norm=norm, cmap=text_cmap), cax=cax, orientation='horizontal', drawedges=False)
    cb.outline.set_visible(False)
    cb.set_ticks([])

    y_offset -= 0.14
    sanity_text = (
        f"CODE SANITY REPORT\n"
        f"---------------------\n"
        f"// 'unwrap' : {total_unwrap:<4}\n"
        f"// 'expect' : {total_expect:<4}\n"
        f"// 'unsafe' : {total_unsafe:<4}"
    )
    ax_text.text(pad_left, y_offset, sanity_text, fontsize=9.5, fontfamily='monospace', color=text_color, weight='bold', va='top')

    ax_pie.set_title("Codebase Stats", fontsize=13, pad=5, color=text_color, fontweight='bold', fontfamily='sans-serif')
    
    plt.subplots_adjust(left=0.05, right=0.95, top=0.90, bottom=0.08)
    
    plt.savefig(output_path, dpi=150, bbox_inches='tight', transparent=True)
    plt.close()
    log.info("chart_generated", output=output_path)

@click.command()
@click.option('--path', '-p', default='.', type=click.Path(exists=True))
@click.option('--target', '-t', default='.py,.rs')
@click.option('--output', '-o', default='disk_analysis.png')
def main(path, target, output):
    structlog.configure(processors=[structlog.processors.TimeStamper(fmt="iso"), structlog.processors.KeyValueRenderer()])
    target_dir = Path(path)
    extensions = {ext.strip().lower() if ext.strip().startswith('.') else f".{ext.strip().lower()}" for ext in target.split(',')}

    absolute_output = Path(os.getcwd()) / output

    log.info("scan_started", directory=str(target_dir.resolve()), targets=list(extensions))
    data = scan_files(target_dir, extensions, output)
    save_github_chart(data, str(absolute_output))

if __name__ == "__main__":
    main()
