import sh
import tomlkit
from pathlib import Path
from datetime import datetime

BRANCH = "main"

if __name__ == "__main__":
    now = datetime.now()
    version = now.strftime("%y.%-m.%-d")
    print(f"Project version: {version}")

    project_path = Path(__file__).parent

    print(project_path)

    with sh.pushd(project_path):
        
        sh.git("add", ".")
        sh.git("commit", "-m", f"ci-{now}")
        sh.git("push", "origin", BRANCH)
