#!/usr/bin/env -S uv run --script
# /// script
# requires-python = "==3.12.*"
# dependencies = []
# ///

import sys
from pathlib import Path

def update_readme(readme_path: Path, version: str):
    if not readme_path.exists():
        return

    text = readme_path.read_text(encoding="utf-8")
    start_m, end_m = "<!-- # INSTALLATION_START -->", "<!-- # INSTALLATION_END -->"

    if start_m not in text or end_m not in text:
        sys.exit(1)

    install_block = (
        "## Installation\n\n"
        "Install the pre-compiled static binary:\n\n"
        "```sh\n"
        f"sudo curl -L -o /usr/local/bin/abyss-snaps https://github.com/abyss-stack/snaps/releases/download/v{version}/abyss-snaps \\\n"
        "  && sudo chmod +x /usr/local/bin/abyss-snaps\n"
        "```\n\n"
        "Verify the installation:\n"
        "```sh\n"
        "abyss-snaps --version\n"
        "```"
    )

    before, remainder = text.split(start_m, 1)
    _, after = remainder.split(end_m, 1)
    readme_path.write_text(f"{before}{start_m}\n{install_block}\n{end_m}{after}", encoding="utf-8")

def main():
    if sys.stdin.isatty():
        sys.exit(1)
        
    version = sys.stdin.read().strip()
    if not version:
        sys.exit(1)

    readme_path = Path(__file__).parent.parent / "README.md"
    update_readme(readme_path, version)

if __name__ == "__main__":
    main()
