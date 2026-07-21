import sh
import tomlkit
from pathlib import Path
from datetime import datetime

BRANCH = "main"

if __name__ == "__main__":
    now = datetime.now()
    version = now.strftime("%y.%-m.%-d")
    project_path = Path(__file__).parent.parent
    cargo_toml = project_path / "Cargo.toml"

    print(f"Project version: {version}")
    print(f"Project path: {project_path}")

    doc = tomlkit.parse(cargo_toml.read_text(encoding="utf-8"))
    doc["package"]["version"] = version
    cargo_toml.write_text(tomlkit.dumps(doc), encoding="utf-8")

    sh.cargo("build", "--release")
    
    with sh.pushd(project_path):
        sh.git("add", ".")
        sh.git("commit", "-m", f"ci-{now}")
        sh.git("push", "origin", BRANCH)
