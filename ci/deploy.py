import subprocess
from pathlib import Path
from datetime import datetime

BRANCH = "main"

if __name__ == "__main__":
    version = datetime.now().strftime("%y.%-m.%-d")
    print(f"Project version: {version}")

    # Requires 'cargo install cargo-edit'
    cargo_path = Path(__file__).parent
    subprocess.run(["cargo", "set-version", version], cwd=cargo_path)

    subprocess.run(["git", "add", "."])
    subprocess.run(["git", "commit", "-m", f"ci-{version}"], cwd=cargo_path)
    subprocess.run(["git", "push", "origin", BRANCH])
