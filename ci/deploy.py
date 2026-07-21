import sh
import tomlkit
from pathlib import Path
from datetime import datetime

BRANCH = "main"

if __name__ == "__main__":
    now = datetime.now()
    version = now.strftime("%y.%-m.%-d")
    project_path = Path(__file__).parent.parent

    print(f"Project version: {version}")
    print(f"Project path: {project_path}")

    with sh.pushd(project_path):
        sh.git("add", ".")
        sh.git("commit", "-m", f"ci-{now}")
        sh.git("push", "origin", BRANCH)
