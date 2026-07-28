#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.12.0, <3.13.0"
# dependencies = [
#     "sh==2.2.2",
# ]
# ///

from contextlib import chdir
from pathlib import Path
import sys
import sh

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

    run_stats = sh.Command(str(stats_script))
    try:
        run_stats("--path", project_root, "--output", output_img, _fg=True)
        print(f"Chart saved to: {output_img}")
    except sh.ErrorReturnCode as e:
        print(f"Stats error: {e}", file=sys.stderr)
        sys.exit(e.exit_code)

    if not dockerfile.exists():
        print(f"Error: Dockerfile not found at {dockerfile}", file=sys.stderr)
        sys.exit(1)

    artifact_dir.mkdir(exist_ok=True)
    try:
        with chdir(project_root):
            sh.docker.buildx.build("-f", dockerfile, "--target", "exporter", "-o", artifact_dir, ".", _fg=True)
        print(f"Binary in: {artifact_dir}")
    except sh.ErrorReturnCode as e:
        print(f"Docker build error: {e}", file=sys.stderr)
        sys.exit(e.exit_code)

if __name__ == "__main__":
    main()
