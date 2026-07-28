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

    if not stats_script.exists():
        print(f"Error: Script not found at {stats_script}", file=sys.stderr)
        sys.exit(1)

    cmd = f'"{stats_script}" --path "{project_root}" --output "{output_img}"'

    try:
        subprocess.run(cmd, shell=True, check=True)
        print(f"Analysis completed successfully. Chart saved to: {output_img}")
    except subprocess.CalledProcessError as e:
        print(f"Error executing statistics script: {e}", file=sys.stderr)
        sys.exit(e.returncode)

if __name__ == "__main__":
    main()
