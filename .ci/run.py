#!/usr/bin/env -S uv run
# /// script
# requires-python = "==3.12.*"
# dependencies = []
# ///

import subprocess
import sys
from pathlib import Path

def main():
    ci_dir = Path(__file__).parent.resolve()
    project_root = ci_dir.parent

    stats_script = ci_dir / "stats.py"
    output_img = ci_dir / "artifact" / "codebase_size.png"
    dockerfile = ci_dir / "Dockerfile"
    artifact_dir = ci_dir / "artifact"

    if not stats_script.exists():
        print(f"Error: Script not found at {stats_script}", file=sys.stderr)
        sys.exit(1)

    cmd_stats = f'"{stats_script}" --path "{project_root}" --output "{output_img}"'
    try:
        subprocess.run(cmd_stats, shell=True, check=True)
        print(f"Chart saved to: {output_img}")
    except subprocess.CalledProcessError as e:
        print(f"Stats error: {e}", file=sys.stderr)
        sys.exit(e.returncode)

    if not dockerfile.exists():
        print(f"Error: Dockerfile not found at {dockerfile}", file=sys.stderr)
        sys.exit(1)

    artifact_dir.mkdir(exist_ok=True)
    cmd_build = f"docker buildx build -f {dockerfile} --target exporter -o {artifact_dir} {project_root}"

    try:
        subprocess.run(cmd_build, shell=True, check=True)
        print(f"Binary in: {artifact_dir}")
    except subprocess.CalledProcessError as e:
        print(f"Docker build error: {e}", file=sys.stderr)
        sys.exit(e.returncode)

if __name__ == "__main__":
    main()
