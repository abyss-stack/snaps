import sh
from pathlib import Path
from datetime import datetime

BRANCH = "main"

if __name__ == "__main__":
    now = datetime.now()
    version = now.strftime("%y.%-m.%-d")
    print(f"Project version: {version}")

    project_path = Path(__file__).parent

    with sh.pushd(project_path):
        # Requires 'cargo install cargo-edit'
        sh.cargo("set-version", version)

        sh.git("add", ".")
        sh.git("commit", "-m", f"ci-{now}")
        sh.git("push", "origin", BRANCH)
